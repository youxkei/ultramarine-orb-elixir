//! The pads the machine has, read off the game's thread, and what a sample of one means.
//!
//! `Controller::GetControllerInput` asks winmm for joystick 0 once a frame, and where no joystick
//! answers, that call can take long enough to cost the frame its blank — measured as a large part of a
//! frame, spent looking for a device rather than waiting on one.
//!
//! Because it is work rather than waiting, moving it to a quieter part of the frame buys nothing. So a
//! thread of orb's own takes the samples and the game's call is answered out of the last one. What a
//! sample means is left to the game: which button is shot, where an axis becomes a direction, the
//! auto-repeat behind holding one — none of that is orb's to reimplement, and all of it is downstream
//! of the call this replaces.
//!
//! The setting that turned the joystick off was never the answer to this and is gone: the call is only
//! expensive where no device answers, which is also the only case DirectInput leaves the game in this
//! branch for.
//!
//! **Every pad is read and not the one device the game asks about.** 紅魔郷 holds exactly one
//! DirectInput device, settled at startup and never again, and asks winmm about joystick 0 alone — so a
//! second pad, a pad plugged in after the game started, and a pad in XInput's second slot are all pads
//! the game cannot read. This is a one-player game, so each of them is that one player's, and each is
//! read here: every index winmm has room for, and every one of XInput's four slots.
//!
//! **And one of them is handed over: the pad last pushed.** See [`reading`], which is what
//! [`Game::pad_word`](crate::game::Game::pad_word) puts into the word the game's own read handed back.
//!
//! **What each socket costs is what decides how often it is read.** The empty one is the expensive one
//! — the read this module exists to keep out of the frame — so a socket with a pad in it is read four
//! times a frame and every socket is looked at once a second, which is also what bounds how late a pad
//! plugged in mid-run is picked up. What a look at all of them costs is measured beside
//! [`CYCLES_BETWEEN_SWEEPS`].
//!
//! **Every pad carries the bounds its axes are to be measured against**, in its own [`Reading`], which is
//! what a `JOYCAPSA` read beside its position is for. The game kept one of those and filled it once, in
//! its startup check and only where a joystick answered there — so a pad that turned up later was measured
//! against zeros, its centred axes both reading as far over, and the run spent the rest of itself with two
//! directions held. Watched happening, and watched not happening once each pad was measured against its
//! own.
//!
//! **What is left in `orb::joystick` is the write over the import table entry**, which is the whole of
//! what needs a process to patch. The replacement that entry is pointed at is [`answer`] and it is here,
//! being a hook body like the eleven in [`crate::runtime`]. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use orb_api::{JoyCaps, JoyInfo, XinputPad, joyerr};

use crate::game::{Axis, Reading};
use crate::{detail, log, profile};

/// `JOY_RETURNALL`, which is what a sample is taken with and what 紅魔郷 asks a read for.
pub const RETURN_ALL: u32 = 0xff;

/// The joystick the game asks about. It only ever asks about this one.
const DEVICE: u32 = 0;

/// Between reads while a joystick is answering. Four samples a frame, so what the game
/// is handed is never a frame behind what the stick is doing.
const ATTACHED_MS: u32 = 4;
/// And while none is. Asking every frame to be told again that there is nothing there is
/// what this module exists to stop, and doing it on another thread would only move the
/// cost onto another core. A joystick plugged in mid-session is picked up within this.
const DETACHED_MS: u32 = 1000;
/// How many reads of the pads that are answering stand between two looks at every socket, which is
/// [`DETACHED_MS`] worth of them: a socket with nothing in it costs what this module exists to keep out
/// of the frame, and a pad that turns up is found within the second either way.
///
/// What a look at every socket costs is `scripts/joystick-scan.c`, which times each of them apiece —
/// what to run the day this sweep is the thing being blamed, rather than a number here that was one
/// machine's on one day.
const CYCLES_BETWEEN_SWEEPS: u32 = DETACHED_MS / ATTACHED_MS;
/// How many reads a `verbose` line covers: a second's worth while a joystick answers, and
/// far rarer than that while none does, which is when there is least to say about it.
///
/// **Which is why no e2e test reaches that line.** A real host waits the four milliseconds, so an
/// e2e test waiting for one waits a second of wall clock — for a line about what the reads cost.
/// Reachable, and not worth a second of the suite.
const REPORT_READS: u32 = 250;

/// The last sample of joystick 0, which is what the game's own startup check is answered out of — the one
/// read of a joystick it still makes for itself.
static SAMPLE: Mutex<Option<Sample>> = Mutex::new(None);

/// And the pad the run is being played with, which is the one last pushed of the pads orb sampled: what
/// is added to the word the game reads, and what a menu of orb's is driven by.
static PAD: Mutex<Option<Reading>> = Mutex::new(None);

/// winmm's own `joyGetPosEx`, which the game's import table pointed at — or the function a game laid out
/// by hand handed over in place of it.
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
/// Whether the thread has been asked for. Never cleared: a failed spawn leaves every
/// call going to winmm, which is the behaviour orb replaced and not a broken one.
static POLLING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy)]
pub struct Sample {
    pub result: u32,
    pub info: JoyInfo,
    /// The device's own, taken when one starts answering and kept for as long as it does.
    ///
    /// Not read again beside every position: the caps belong to the device and not to the read, and a
    /// second call per read is a second call per read. A pad swapped for another without one failing read
    /// in between would keep the first one's, which at a read every four milliseconds is not a way pads
    /// are swapped.
    pub caps: Option<JoyCaps>,
}

impl Sample {
    /// Whether what answered is a pad, which is not the same as something having answered.
    ///
    /// A device with no buttons and no axes has nothing to say, and winmm's joystick 0 can be one of
    /// those: a pad sitting in XInput's second slot leaves a device at index 0 that answers
    /// `joyGetPosEx` successfully with every field zero, while the pad itself is somewhere further
    /// along. So a successful read is not a pad, and believing it costs a line in the log claiming one
    /// answered and a run played with the axes of a device that has none: bounds of zero, which no position
    /// is inside.
    pub fn is_a_pad(&self) -> bool {
        self.result == joyerr::NOERROR
            && self
                .caps
                .is_some_and(|caps| caps.buttons > 0 || caps.axes > 0)
    }

    /// It as a pad, in the numbers the game's own read of one works in — `None` for what is no pad.
    fn reading(&self) -> Option<Reading> {
        let caps = self.caps.filter(|_| self.is_a_pad())?;
        Some(Reading {
            buttons: self.info.buttons,
            x: Axis {
                at: self.info.x,
                min: caps.x_min,
                max: caps.x_max,
            },
            y: Axis {
                at: self.info.y,
                min: caps.y_min,
                max: caps.y_max,
            },
            pov: self.info.pov,
        })
    }
}

/// Writes down what the sampling thread just read of joystick 0, which is what the game's own read is
/// answered out of.
fn sampled(sample: Sample) {
    if let Ok(mut held) = SAMPLE.lock() {
        *held = Some(sample);
    }
}

/// And which pad the run is being played with, which is what the word and orb's own menus are answered
/// out of.
fn sampled_pad(reading: Option<Reading>) {
    if let Ok(mut held) = PAD.lock() {
        *held = reading;
    }
}

/// Whether a sample taken with `RETURN_ALL` says everything this caller asked for.
fn describes_a_sample(asked: &JoyInfo) -> bool {
    asked.size as usize == size_of::<JoyInfo>() && asked.flags & !RETURN_ALL == 0
}

fn latest() -> Option<Sample> {
    *SAMPLE.lock().ok()?
}

/// `joyGetPosEx`, as the game's own import table holds it. The struct it fills is
/// [`orb_api::JoyInfo`], which is `JOYINFOEX`'s own layout — see there — so this is the signature
/// winmm exports and not a shape of orb's.
pub type JoyGetPosEx = unsafe extern "system" fn(u32, *mut JoyInfo) -> u32;

/// Says which function [`answer`] falls through to, which `orb::joystick::install` does with what it took
/// out of the import table — and a game laid out by hand with its own `joyGetPosEx`, there being no import
/// table to take one out of.
///
/// The same answer [`crate::window::install_over`] and [`crate::score::install_over`] are — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
pub fn install_over(original: JoyGetPosEx) {
    ORIGINAL.store(original as usize, Ordering::Relaxed);
}

/// Replaces `joyGetPosEx` for the game's own reads of it, which are its startup check — is there a
/// joystick at all — and the caps read behind that. The per-frame one was the third, and orb answers the
/// whole of that read now rather than the call inside it.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// read would have gone through the patched entry, there being no import table to reach it through.
///
/// # Safety
/// `into` must be null or a writable [`JoyInfo`], which is what winmm's own contract for that argument
/// is.
pub unsafe extern "system" fn answer(device: u32, into: *mut JoyInfo) -> u32 {
    sampling();
    if let Some(result) = unsafe { answered(device, into) } {
        return result;
    }
    let original: JoyGetPosEx = unsafe { std::mem::transmute(ORIGINAL.load(Ordering::Relaxed)) };
    unsafe { original(device, into) }
}

/// Starts the sampling thread, if the first read of the run has not already started it.
///
/// Not from `DllMain`, where the rest of orb's hooks go in: a thread created while the
/// loader lock is held cannot run its own attach notifications, and the game is
/// suspended there, so nothing would be answered anyway. What starts it is whichever comes first of the
/// game's own startup check and the first frame that asks for the pads — the second of those being the
/// only one there is where the game holds a controller and never asks winmm again.
fn sampling() {
    if POLLING
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    // Through the seam, which carries whatever simulated Windows this thread reads through onto the
    // one being made: a thread spawned plainly would read the machine's own winmm — see
    // `orb_api::thread::spawn`.
    if orb_api::thread::spawn(|| poll()).is_err() {
        log!("joystick: cannot start a thread; the read stays in the game's frame");
    }
}

/// The whole of what the game's own `joyGetPosEx` is answered with, and `None` for a read no sample
/// answers — another joystick, a struct a sample does not describe, or the very first call, which is
/// the game's startup check and arrives before the thread has anything.
///
/// **Nothing is written into the game's memory here**, and there is a deletion behind that: the caps of the
/// answering device used to be copied into the `JOYCAPSA` at 0x69d760, because the game measured an axis
/// against that struct and only ever filled it at startup. Every read of it was in
/// `Controller::GetControllerInput`'s winmm branch, 0x41d18b to 0x41d22f, and orb answers that whole
/// function now — each pad measured against the bounds its own caps report, which is what a `Reading`
/// carries. So the struct has no reader left and the write went with it.
///
/// # Safety
/// `into` must be null or a writable [`JoyInfo`], which is winmm's own contract for that argument.
unsafe fn answered(device: u32, into: *mut JoyInfo) -> Option<u32> {
    if device != DEVICE || into.is_null() || !unsafe { describes_a_sample(&*into) } {
        return None;
    }
    let sample = latest()?;
    unsafe { *into = sample.info };
    Some(sample.result)
}

fn poll() -> ! {
    // Below the game's, because a sample a millisecond old costs nobody anything and a
    // late frame costs the thing orb was written for.
    orb_api::thread::below_normal();
    let mut sockets = sockets();
    let devices = sockets.iter().filter(|socket| socket.is_winmm()).count();
    let slots = sockets.len() - devices;
    let mut since_a_sweep = CYCLES_BETWEEN_SWEEPS;
    let mut said_there_is_no_pad = false;
    // How many pushes this thread has seen, which is what says which socket saw the last of them.
    let mut pushes = 0;
    let mut reading_from = None;
    let mut polls = 0;
    let mut total = 0;
    let mut swept = 0;
    loop {
        // Every socket once a second, which is what finds a pad that has just been plugged in — and
        // that is every cycle while nothing answers, the wait between two of those being a second
        // already.
        let sweeping =
            since_a_sweep >= CYCLES_BETWEEN_SWEEPS || !sockets.iter().any(Socket::has_a_pad);
        let started = profile::now();
        // One read of the clock apiece rather than two, since what each read costs is the gap between
        // this socket's own finishing and the last one's: the counter is the game's, and a thread of
        // orb's reading it twenty times a cycle is a thread charging the frame for its own sampling.
        let mut at = started;
        for socket in &mut sockets {
            if sweeping || socket.has_a_pad() {
                let answered = socket.read();
                // Which of them somebody is holding, as far as anything here can tell: the one they
                // last did something to. Counted rather than timed, one number for the whole thread —
                // what is asked of it is only which socket's is the highest.
                if socket.was_pushed() {
                    pushes += 1;
                    socket.pushed_at = pushes;
                }
                let now = profile::now();
                socket.report(answered, now - at);
                at = now;
            }
        }
        let cost = at - started;
        let pad = last_pushed(&sockets);
        sampled_pad(pad.and_then(|socket| socket.reading));
        // And a line where that is a different pad from the one before, which is somebody putting one
        // down and picking another up.
        let read_from = pad.map(|socket| socket.which);
        if reading_from != read_from {
            reading_from = read_from;
            if let Some(which) = read_from {
                log!(
                    "joystick: {which} is the pad the run is played with, it being the last pushed"
                );
            }
        }

        let answering = pad.is_some();
        // The one line a machine with no pad at all gets. Every other line is a socket's own and is
        // written where what that socket answers changes — a socket that has never had anything in it
        // has never changed, and sixteen of those would be sixteen lines saying nothing.
        if !answering && !said_there_is_no_pad {
            log!("joystick: no pad on any of winmm's {devices} devices or XInput's {slots} slots");
        }
        said_there_is_no_pad = !answering;
        if sweeping {
            swept = cost;
            since_a_sweep = 0;
        } else {
            since_a_sweep += 1;
            polls += 1;
            total += cost;
        }
        // Each line covers its own reads and no others: one that averaged everything since
        // the thread started would spend a long time walking the first read down — the slow one,
        // where winmm found the device — and read as though the reads were getting faster.
        // What device they are of stands in the lines above rather than being asked again,
        // since naming it means another read of the thing this stops reading often.
        if polls == REPORT_READS {
            detail!(
                "joystick: {polls} reads of what answers, {}us each; every socket, {swept}us",
                total / i64::from(polls)
            );
            (polls, total) = (0, 0);
        }

        let wait = if answering { ATTACHED_MS } else { DETACHED_MS };
        // Never sooner than the read itself took, so a device slow enough to cost what no
        // device at all costs cannot hold a core of its own.
        orb_api::clock::sleep(wait.max((cost.max(0) / 1000) as u32));
    }
}

/// The pad the run is played with, and the only one handed over — see [`reading`].
///
/// The one somebody last did something to, and the first socket that answers until anything has been
/// pushed at all, which is what a machine with one pad in it has all along. An earlier socket on a tie so
/// that a run with two pads plugged in and neither touched is played with the same one from frame to
/// frame.
fn last_pushed(sockets: &[Socket]) -> Option<&Socket> {
    sockets
        .iter()
        .filter(|socket| socket.has_a_pad())
        .reduce(|held, socket| {
            if socket.pushed_at > held.pushed_at {
                socket
            } else {
                held
            }
        })
}

/// Every place a pad can be, as of the launch: winmm's own count of the indices it will answer about,
/// and XInput's four slots.
///
/// Asked once. `joyGetNumDevs` is how many devices winmm has *room* for rather than how many are
/// plugged in — it answers 16 on this machine with one pad attached — so it is a property of the
/// installation and not of what is in it.
fn sockets() -> Vec<Socket> {
    let winmm = (0..orb_api::joystick::count()).map(Which::Winmm);
    let xinput = (0..orb_api::xinput::SLOTS).map(Which::Xinput);
    winmm.chain(xinput).map(Socket::new).collect()
}

/// One place a pad can be, and what orb last read there.
struct Socket {
    which: Which,
    /// The winmm caps of the device in it, read once when one starts answering — see [`Sample::caps`].
    /// Never anything for an XInput slot, whose pad answers in XInput's own units and needs no bounds
    /// read for it.
    caps: Option<JoyCaps>,
    /// What is in it, as the game's own read of a pad works in — `None` for a socket with no pad in it,
    /// the phantom included.
    reading: Option<Reading>,
    /// What it was last read as while somebody was doing something to it, which is what the next read is
    /// held against to see whether they still are — see [`Socket::was_pushed`].
    pushed: Option<Reading>,
    /// Which push of the thread's count that was, so that the highest of them is the pad in somebody's
    /// hands. Zero for a socket nobody has touched this run.
    pushed_at: u64,
    /// What the log was last told about it, so that a line is written where that changes rather than
    /// per read. Not the reading, which changes with every push and says nothing new.
    said: Answered,
}

/// Which of the two interfaces, and where on it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Which {
    Winmm(u32),
    Xinput(u32),
}

impl std::fmt::Display for Which {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Winmm(device) => write!(out, "winmm {device}"),
            Self::Xinput(slot) => write!(out, "xinput {slot}"),
        }
    }
}

/// What a socket last answered, in the terms a line about it is written from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Answered {
    /// Never read, which is what every socket starts as — and what keeps a machine's empty ones out of
    /// the log, a line being written where what a socket answers *changes*.
    Unread,
    /// A pad.
    Pad,
    /// A device with no buttons and no axes, which is no pad however much it answered.
    NotAPad,
    /// Nothing in it. winmm says which — `JOYERR_PARMS` for an index its driver has nothing behind, and
    /// `JOYERR_UNPLUGGED` for a socket whose device has gone — where an XInput slot only ever says that
    /// nobody is in it.
    Empty(Option<u32>),
}

impl Socket {
    fn new(which: Which) -> Self {
        Self {
            which,
            caps: None,
            reading: None,
            pushed: None,
            pushed_at: 0,
            said: Answered::Unread,
        }
    }

    fn has_a_pad(&self) -> bool {
        self.reading.is_some()
    }

    fn is_winmm(&self) -> bool {
        matches!(self.which, Which::Winmm(_))
    }

    /// Whether what it now says is somebody doing something to the pad in it, rather than the pad
    /// sitting where it was left — which is what decides whose pad the run is played with.
    ///
    /// **A button or the hat exactly, and a stick only past a step.** A stick reports a few counts of
    /// noise while nobody is touching it, and a pad whose noise counted as somebody's hand would be the
    /// last-pushed pad for ever — which on a machine with two pads is the other one never working
    /// again. The step is a sixteenth of the travel, which is well inside the quarter of it the game's
    /// own read needs before it calls an axis a direction, so nothing that would move the player fails
    /// to count.
    ///
    /// Held against the last reading that counted rather than against the last read, so that a stick
    /// pushed slowly counts once it has gone a step from where it was resting instead of never.
    fn was_pushed(&mut self) -> bool {
        /// How much of an axis' travel a stick has to have moved.
        const STEP: u32 = 16;
        let Some(now) = self.reading else {
            self.pushed = None;
            return false;
        };
        let Some(before) = self.pushed else {
            // The first read of a pad is not somebody pushing it: what it says is where it was left, and
            // a run that started with two pads plugged in would otherwise be played with whichever
            // socket was read last.
            self.pushed = Some(now);
            return false;
        };
        let moved = |now: Axis, before: Axis| {
            now.max > now.min && now.at.abs_diff(before.at) > (now.max - now.min) / STEP
        };
        let pushed = now.buttons != before.buttons
            || now.pov != before.pov
            || moved(now.x, before.x)
            || moved(now.y, before.y);
        if pushed {
            self.pushed = Some(now);
        }
        pushed
    }

    /// Reads it, and answers what it now has in it.
    fn read(&mut self) -> Answered {
        match self.which {
            Which::Winmm(device) => self.read_winmm(device),
            Which::Xinput(slot) => self.read_xinput(slot),
        }
    }

    /// One winmm device: where it is, and what it is where that is not already known.
    fn read_winmm(&mut self, device: u32) -> Answered {
        // The host's own winmm rather than the entry `orb::joystick::install` redirected, and the cost
        // this module exists to move is right here.
        let (result, info) = orb_api::joystick::position(device, RETURN_ALL);
        // Only where one answered, and only where the last read did not: the caps of a
        // device that is not there cost what its position costs, and a device that is there
        // has the caps it had a moment ago.
        self.caps = match (result, self.caps) {
            (joyerr::NOERROR, None) => orb_api::joystick::caps(device),
            (joyerr::NOERROR, known) => known,
            _ => None,
        };
        let sample = Sample {
            result,
            info,
            caps: self.caps,
        };
        // The game's own startup check is answered out of joystick 0's sample, that being the one device it
        // asks about.
        if device == DEVICE {
            sampled(sample);
        }
        self.reading = sample.reading();
        match (sample.is_a_pad(), result) {
            (true, _) => Answered::Pad,
            (false, joyerr::NOERROR) => Answered::NotAPad,
            (false, result) => Answered::Empty(Some(result)),
        }
    }

    /// And one XInput slot, whose pad describes itself: no caps to read, XInput reporting every pad in
    /// its own units.
    fn read_xinput(&mut self, slot: u32) -> Answered {
        self.reading = orb_api::xinput::state(slot).map(|pad| reading_of(&pad));
        match self.reading {
            Some(_) => Answered::Pad,
            None => Answered::Empty(None),
        }
    }

    /// Says in the log what it now has in it, where that has changed since the last line about it, with
    /// what this read cost.
    fn report(&mut self, answered: Answered, cost: i64) {
        if self.said == answered {
            return;
        }
        // A socket that has never had anything in it says nothing: winmm has room for sixteen and a
        // machine has a pad in one of them, so the other fifteen would be fifteen lines of a launch's
        // log saying that a socket nobody has used is still unused. What a machine with no pad at all
        // gets instead is the one line [`poll`] writes.
        let unused = self.said == Answered::Unread && matches!(answered, Answered::Empty(_));
        self.said = answered;
        if !unused {
            log!(
                "joystick: {} {}, read in {cost}us",
                self.which,
                self.says(answered),
            );
        }
    }

    /// What a line about it says, built where one is going to be written and not per read: `format!`
    /// on every read of every socket is an allocation four times a frame for a line nobody asked for.
    fn says(&self, answered: Answered) -> String {
        let named = |caps: &JoyCaps| {
            let (mid, pid) = (caps.manufacturer, caps.product);
            format!("mid={mid:04x} pid={pid:04x} \"{}\"", name(caps.name))
        };
        // An XInput slot has one thing to say either way: its pad describes itself in XInput's own
        // units, so there are no caps to name it by and nothing to fail to read.
        if let Which::Xinput(_) = self.which {
            return match answered {
                Answered::Pad => "has a pad",
                _ => "has nobody in it",
            }
            .to_owned();
        }
        match (answered, &self.caps) {
            (Answered::Pad, Some(caps)) => {
                let (buttons, axes) = (caps.buttons, caps.axes);
                let (low, high) = (caps.x_min, caps.x_max);
                format!(
                    "is {}, {buttons} buttons, {axes} axes, X {low}..{high}",
                    named(caps),
                )
            }
            // Named all the same, since which phantom it is is the thing to look up: 413d:2104 is
            // what Windows leaves on joystick 0 while the pad it has is in XInput's second slot.
            (Answered::NotAPad, Some(caps)) => format!(
                "is {} with no buttons and no axes, which is no pad; nothing of orb's will be \
                 driven from it",
                named(caps),
            ),
            (Answered::Empty(Some(joyerr::PARMS)), _) => "has nothing behind it".to_owned(),
            (Answered::Empty(Some(joyerr::UNPLUGGED)), _) => "is unplugged".to_owned(),
            (Answered::Empty(Some(other)), _) => format!("answers joyGetPosEx with {other}"),
            // A device that answered and whose caps did not read, which orb can neither describe nor
            // measure an axis of: the bounds are what a position is held against.
            _ => "answers, and its caps do not read".to_owned(),
        }
    }
}

/// An XInput pad in the numbers the game's own read of a pad works in.
///
/// **A translation and not a decision**, which is the whole of what may happen here: where an axis
/// becomes a direction and which button shoots are the game's, so what this does is put XInput's answer
/// into the units the game's arithmetic is written for — the buttons into DirectInput's numbering, the
/// stick into a winmm axis' travel, and the d-pad into the angle a hat reports.
fn reading_of(pad: &XinputPad) -> Reading {
    // The travel a winmm pad of this shape reports, which is what the game's quarter-of-the-travel dead
    // zone is worked out from — 0x41d18b, `(wXmax - wXmin) / 4`. XInput's own dead zone is not used, nor
    // is `cfg.padYAxis`: whether a stick is pushed is the game's answer to give, and the branch it would
    // have given it on for a pad it holds no device for is the winmm one, which gives it out of these two
    // numbers. The threshold beside the mapping is the *other* branch's, for the device it does hold.
    const TRAVEL: (u32, u32) = (0, u16::MAX as u32);
    let axis = |at: i16, upwards: bool| {
        // XInput measures both of its axes from the middle, and its Y upwards where every axis winmm
        // reports is measured downwards.
        let at = if upwards {
            -i32::from(at)
        } else {
            i32::from(at)
        };
        Axis {
            at: (at + i32::from(i16::MAX)).clamp(0, TRAVEL.1 as i32) as u32,
            min: TRAVEL.0,
            max: TRAVEL.1,
        }
    };
    Reading {
        buttons: buttons_of(pad.buttons),
        x: axis(pad.left_x, false),
        y: axis(pad.left_y, true),
        pov: pov_of(pad.buttons),
    }
}

/// XInput reports its buttons in its own order, and the game's configuration names them in
/// DirectInput's — which is the order the game's own controller reports an XInput pad in, and the order
/// winmm reports one in on the occasions it has it: A, B, X, Y, the two shoulders, Back, Start, and the
/// two thumbs. So the mask is translated into that numbering rather than the mapping being
/// second-guessed, and *shoot decides* stays true whatever the player mapped.
///
/// The launcher's settings dialog holds the same table, for the pad that answers it before the game
/// starts — see `launcher::pad`.
const BUTTONS: [(u16, u32); 10] = [
    (0x1000, 0), // A
    (0x2000, 1), // B
    (0x4000, 2), // X
    (0x8000, 3), // Y
    (0x0100, 4), // left shoulder
    (0x0200, 5), // right shoulder
    (0x0020, 6), // Back
    (0x0010, 7), // Start
    (0x0040, 8), // left thumb
    (0x0080, 9), // right thumb
];
/// And its d-pad, which is not a button in that numbering: a winmm pad reports one in its hat and the
/// game's mapping names no button for a direction.
const DPAD_UP: u16 = 0x0001;
const DPAD_DOWN: u16 = 0x0002;
const DPAD_LEFT: u16 = 0x0004;
const DPAD_RIGHT: u16 = 0x0008;

/// XInput's button mask in the numbering the game's configuration names buttons by.
fn buttons_of(mask: u16) -> u32 {
    BUTTONS
        .iter()
        .filter(|(bit, _)| mask & bit != 0)
        .fold(0, |buttons, (_, button)| buttons | 1 << button)
}

/// And its d-pad as a hat's own field: hundredths of a degree clockwise from straight up, and
/// `JOY_POVCENTERED` for pushed nowhere.
///
/// An angle because that is what a `Reading` carries and what the game's own reading of a hat is written
/// against — the four bits name eight of them and the middle, so nothing is lost either way.
fn pov_of(mask: u16) -> u32 {
    /// And what winmm reports for a hat pushed nowhere, which is past a full circle.
    const CENTERED: u32 = 0xffff;
    let pushed = |bit: u16| mask & bit != 0;
    let (up, down) = (pushed(DPAD_UP), pushed(DPAD_DOWN));
    let (left, right) = (pushed(DPAD_LEFT), pushed(DPAD_RIGHT));
    // Opposite sides cancel: a hat says one thing, and a d-pad with both sides of an axis held says
    // nothing about that axis.
    let (up, down) = (up && !down, down && !up);
    let (left, right) = (left && !right, right && !left);
    // The eight a hat points in and the middle, clockwise from straight up.
    match (up, down, left, right) {
        (true, false, false, false) => 0,
        (true, false, false, true) => 4500,
        (false, false, false, true) => 9000,
        (false, true, false, true) => 13500,
        (false, true, false, false) => 18000,
        (false, true, true, false) => 22500,
        (false, false, true, false) => 27000,
        (true, false, true, false) => 31500,
        _ => CENTERED,
    }
}

/// `szPname`, which winmm gives in the machine's code page and the log is written in UTF-8:
/// the name here read `Microsoft PC �W���C�X�e�B�b�N` before this converted it, and a line
/// nobody can read is no answer to which device it is.
fn name(field: [u8; 32]) -> String {
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(32);
    orb_api::codepage::text(&field[..end])
}

/// The pad the run is being played with, as it was last sampled — `None` while no pad is answering
/// anywhere.
///
/// **One pad and not every pad**, that one being the last of them pushed: two pads plugged into a
/// one-player game is one in somebody's hands and one on the floor, and the one on the floor is the one
/// whose stick may be resting past its own dead zone. Merged, that is a direction held down for the rest
/// of the run; read this way, it is a pad that stops being read the moment the other is touched. See
/// [`Socket::was_pushed`] for what counts as touching one.
///
/// For the word the game's own read handed back, and for the menus orb puts up itself. Those freeze the
/// game, so the game's own reading of a pad is not running either and a pad would do nothing at all on
/// them; the samples this thread takes every few milliseconds are there to be read.
///
/// Starts the sampling thread where nothing has yet, which is every launch whose game holds a
/// DirectInput controller: such a game asks winmm nothing after its startup check, so the first frame
/// that wants a pad is what has to start it.
pub fn reading() -> Option<Reading> {
    sampling();
    *PAD.lock().ok()?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A device answering with no buttons and no axes is not a pad, and what makes that worth
    /// testing is that Windows leaves exactly one of those on joystick 0 while the pad it has
    /// sits in XInput's second slot.
    #[test]
    fn a_device_with_nothing_on_it_is_not_a_pad() {
        let mut caps = JoyCaps {
            x_max: 65535,
            ..JoyCaps::default()
        };
        let phantom = Sample {
            result: joyerr::NOERROR,
            info: JoyInfo::default(),
            caps: Some(caps),
        };
        assert!(!phantom.is_a_pad());
        assert!(phantom.reading().is_none());

        // A stick with axes and no buttons is still a pad, and so is a wheel with buttons and
        // one axis: either can say something.
        caps.axes = 2;
        let stick = Sample {
            caps: Some(caps),
            ..phantom
        };
        assert!(stick.is_a_pad());

        // And nothing at all answering is not one either, whatever it left in the caps.
        let nothing = Sample {
            result: joyerr::UNPLUGGED,
            ..stick
        };
        assert!(!nothing.is_a_pad());
    }

    /// A winmm pad's reading carries the bounds its axes are to be measured against, which are that
    /// device's own and not the ones the game keeps: the game's are joystick 0's, so a second pad
    /// measured against them would be measured against a device it is not.
    #[test]
    fn a_winmm_pads_reading_carries_its_own_travel() {
        let sample = Sample {
            result: joyerr::NOERROR,
            info: JoyInfo {
                x: 1000,
                y: 2000,
                buttons: 0b101,
                pov: 9000,
                ..JoyInfo::default()
            },
            caps: Some(JoyCaps {
                x_min: 100,
                x_max: 4000,
                y_min: 200,
                y_max: 3000,
                buttons: 12,
                ..JoyCaps::default()
            }),
        };
        let reading = sample.reading().expect("a pad");
        assert_eq!(
            (reading.x.at, reading.x.min, reading.x.max),
            (1000, 100, 4000)
        );
        assert_eq!(
            (reading.y.at, reading.y.min, reading.y.max),
            (2000, 200, 3000)
        );
        assert_eq!((reading.buttons, reading.pov), (0b101, 9000));
    }

    /// XInput's own button order is not the order the game's configuration names buttons in, and the
    /// game's is DirectInput's: A first, then B, X, Y, the shoulders, Back, Start, the thumbs. A mapping
    /// that says shoot is button 0 therefore has to be answered by A.
    #[test]
    fn an_xinput_pads_buttons_come_out_in_the_order_the_game_names_them() {
        assert_eq!(buttons_of(0x1000), 1 << 0, "A shoots");
        assert_eq!(buttons_of(0x2000), 1 << 1, "B bombs");
        assert_eq!(buttons_of(0x0010), 1 << 7, "Start is the eighth");
        assert_eq!(buttons_of(0x1000 | 0x0020), 1 << 0 | 1 << 6, "A and Back");
        // The d-pad is not a button here: it is read as a direction, the way a hat is.
        assert_eq!(buttons_of(0x000f), 0);
    }

    /// And its d-pad comes out as the angle a hat reports, since that is what the game's own reading of
    /// a hat is written against.
    #[test]
    fn an_xinput_pads_dpad_comes_out_as_a_hats_angle() {
        assert_eq!(pov_of(DPAD_UP), 0);
        assert_eq!(pov_of(DPAD_RIGHT), 9000);
        assert_eq!(pov_of(DPAD_DOWN), 18000);
        assert_eq!(pov_of(DPAD_LEFT), 27000);
        // The four diagonals, each between its two.
        assert_eq!(pov_of(DPAD_UP | DPAD_RIGHT), 4500);
        assert_eq!(pov_of(DPAD_DOWN | DPAD_RIGHT), 13500);
        assert_eq!(pov_of(DPAD_DOWN | DPAD_LEFT), 22500);
        assert_eq!(pov_of(DPAD_UP | DPAD_LEFT), 31500);
        // Nothing pushed, and both sides of one axis held at once, are a hat pushed nowhere.
        assert_eq!(pov_of(0), 0xffff);
        assert_eq!(pov_of(DPAD_UP | DPAD_DOWN), 0xffff);
        assert_eq!(pov_of(DPAD_LEFT | DPAD_RIGHT), 0xffff);
        // And a button that is not a direction says nothing about where the hat points.
        assert_eq!(pov_of(0x1000), 0xffff);
    }

    /// An XInput stick comes out in the travel a winmm axis reports, measured downwards: its Y is
    /// measured upwards, which is the opposite of every axis the game's arithmetic was written for.
    #[test]
    fn an_xinput_pads_stick_comes_out_in_a_winmm_axis_travel() {
        let centred = reading_of(&XinputPad::default());
        assert_eq!((centred.x.min, centred.x.max), (0, 65535));
        assert_eq!(centred.x.at, 32767, "a stick nobody is touching is centred");
        assert_eq!(centred.y.at, 32767);

        let pushed = reading_of(&XinputPad {
            left_x: i16::MIN,
            left_y: i16::MAX,
            ..XinputPad::default()
        });
        assert_eq!(pushed.x.at, 0, "the stick pushed left is the low side of X");
        assert_eq!(pushed.y.at, 0, "and pushed up is the low side of Y");

        let pushed = reading_of(&XinputPad {
            left_x: i16::MAX,
            left_y: i16::MIN,
            ..XinputPad::default()
        });
        assert_eq!((pushed.x.at, pushed.y.at), (65534, 65535));
    }

    /// A sample is taken with `JOY_RETURNALL` into the current `JOYINFOEX`, which is what
    /// 紅魔郷 asks for. Anything asking for more than that has to go to winmm.
    #[test]
    fn a_sample_answers_what_the_game_asks_and_no_more() {
        let mut asked = JoyInfo {
            size: size_of::<JoyInfo>() as u32,
            flags: RETURN_ALL,
            ..JoyInfo::default()
        };
        assert!(describes_a_sample(&asked));

        // `JOY_RETURNRAWDATA`, which a sample does not carry.
        asked.flags = RETURN_ALL | 0x100;
        assert!(!describes_a_sample(&asked));

        // The struct before `JOYINFOEX` grew, which is not the one a sample fills.
        asked.flags = RETURN_ALL;
        asked.size -= 4;
        assert!(!describes_a_sample(&asked));
    }
}
