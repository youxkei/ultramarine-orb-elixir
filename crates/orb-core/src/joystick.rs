//! The joystick read, off the game's thread, and what a sample of it means.
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
//! One thing the game cannot be left to do for itself. `GetControllerInput` places the centre of each
//! axis at `(wXmin + wXmax) / 2` with a dead zone of a quarter of the travel, read out of the
//! `JOYCAPSA` at 0x69d760 — an address that appears exactly once in the whole exe, in the
//! `joyGetDevCapsA` call the startup check makes, and that check only reads it where a joystick
//! answered `joyGetPosEx` first. A pad that turns up after that
//! is measured against zeros — its centred axes, 32767 of a 65535 travel, both read as far
//! over, and the game spends the rest of the run with two directions held. So the
//! calibration is handed over with the sample it belongs to.
//!
//! Watched doing it: a run that started with the pad asleep, had it wake mid-run, took the calibration
//! on the next frame — `the game's axis calibration was not this device's` — and was then driven
//! through the menus with nothing drifting.
//!
//! **What is left in `orb::joystick` is the write over the import table entry**, which is the whole of
//! what needs a process to patch. The replacement that entry is pointed at is [`answer`] and it is here,
//! being a hook body like the eleven in [`crate::runtime`]. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use orb_api::{JoyCaps, JoyInfo, joyerr};

use crate::game::Reading;
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
/// How many reads a `verbose` line covers: a second's worth while a joystick answers, and
/// far rarer than that while none does, which is when there is least to say about it.
///
/// **Which is why no scenario reaches that line.** A real host waits the four milliseconds, so a
/// scenario waiting for one waits a second of wall clock — for a line about what the reads cost.
/// Reachable, and not worth a second of the suite.
const REPORT_READS: u32 = 250;

/// The last sample, which is what the game's own read and orb's own menus are both answered out of.
static SAMPLE: Mutex<Option<Sample>> = Mutex::new(None);

/// winmm's own `joyGetPosEx`, which the game's import table pointed at — or the function a game laid out
/// by hand handed over in place of it.
static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
/// Where the game keeps the caps it measures axes against. Zero for a game that keeps none.
static CALIBRATION: AtomicUsize = AtomicUsize::new(0);
/// Whether the thread has been asked for. Never cleared: a failed spawn leaves every
/// call going to winmm, which is the behaviour orb replaced and not a broken one.
static POLLING: AtomicBool = AtomicBool::new(false);
/// Where the game's calibration last differed from the device's, plus one so that zero means
/// nothing yet.
static REPORTED_DIFFERENCE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
pub struct Sample {
    pub result: u32,
    pub info: JoyInfo,
    /// The device's own, taken when one starts answering and kept for as long as it does.
    ///
    /// Not read again beside every position: the caps belong to the device and not to the
    /// read, and what the game is handed must not change under it from one read to the next
    /// — that is a write into the game's memory each time it does. A pad swapped for another
    /// without one failing read in between would keep the first one's, which at a read every
    /// four milliseconds is not a way pads are swapped.
    pub caps: Option<JoyCaps>,
}

impl Sample {
    /// Whether what answered is a pad, which is not the same as something having answered.
    ///
    /// A device with no buttons and no axes has nothing to say, and winmm's joystick 0 can be one of
    /// those: a pad sitting in XInput's second slot leaves a device at index 0 that answers
    /// `joyGetPosEx` successfully with every field zero, while the pad itself is somewhere further
    /// along. So a successful read is not a pad, and believing it costs a line in the log claiming one
    /// answered and the game's axis calibration written from a device that has no axes.
    pub fn is_a_pad(&self) -> bool {
        self.result == joyerr::NOERROR
            && self
                .caps
                .is_some_and(|caps| caps.buttons > 0 || caps.axes > 0)
    }
}

/// Writes down what the sampling thread just read, which is what everything below answers out of.
fn sampled(sample: Sample) {
    if let Ok(mut held) = SAMPLE.lock() {
        *held = Some(sample);
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

/// Says which function [`answer`] falls through to and where the game keeps the `JOYCAPSA` it measures
/// its axes against, which `orb::joystick::install` does with what it took out of the import table — and
/// a game laid out by hand with its own `joyGetPosEx`, there being no import table to take one out of.
///
/// The same answer [`crate::window::install_over`] and [`crate::score::install_over`] are — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
pub fn install_over(original: JoyGetPosEx, calibration: Option<usize>) {
    ORIGINAL.store(original as usize, Ordering::Relaxed);
    CALIBRATION.store(calibration.unwrap_or(0), Ordering::Relaxed);
}

/// Replaces `joyGetPosEx` for the game's three callers of it: the "is there a pad"
/// check at startup, the caps read behind it, and the per-frame one.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// read would have gone through the patched entry, there being no import table to reach it through.
///
/// # Safety
/// `into` must be null or a writable [`JoyInfo`], which is what winmm's own contract for that argument
/// is, and this must run on the thread that reads the calibration it may write — the game's.
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
/// suspended there, so nothing would be answered anyway. The first call the game makes
/// is the startup check, which is a fine place to start from.
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
/// # Safety
/// `into` must be null or a writable [`JoyInfo`], which is winmm's own contract for that argument, and
/// this must run on the thread that reads the calibration it may write — the game's.
unsafe fn answered(device: u32, into: *mut JoyInfo) -> Option<u32> {
    if device != DEVICE || into.is_null() || !unsafe { describes_a_sample(&*into) } {
        return None;
    }
    let sample = latest()?;
    // Only a device that is a pad writes the game's calibration: the axes of one that has
    // none describe nothing, and this write lands in the game's own memory.
    if let Some(caps) = sample.caps.filter(|_| sample.is_a_pad()) {
        unsafe { calibrate(&caps) };
    }
    unsafe { *into = sample.info };
    Some(sample.result)
}

/// Puts the answering device's caps where the game reads them, so the axes it is about to
/// place the centre of are the axes it has.
///
/// Every read rather than once when the device appeared: a chapter restored from before it
/// appeared rewinds the game's `.data`, and the caps are in there.
///
/// # Safety
/// Must run on the thread that reads them, which is the game's: the write is not atomic and
/// the game reads it a few instructions after this returns.
unsafe fn calibrate(caps: &JoyCaps) {
    let at = CALIBRATION.load(Ordering::Relaxed);
    if at == 0 {
        return;
    }
    // Through the seam and not a raw pointer, which is what it was: the address is the game's — 0x69d760
    // — and in a game laid out by hand there is nothing at that number in this process. A copy through it
    // read the test binary's own image and wrote over it.
    //
    // As bytes rather than as a `JoyCaps`, which is what the copy it replaces was: what is being asked is
    // where two of them first differ, and 0x194 bytes is not a value to read and write as one.
    let ours = bytes_of(caps);
    let theirs = unsafe { orb_api::mem::read_bytes(at, ours.len()) };
    let Some(differs) = ours
        .iter()
        .zip(&theirs)
        .position(|(ours, theirs)| ours != theirs)
    else {
        return;
    };
    unsafe { orb_api::mem::write_bytes(at, ours) };
    // Where it differed, and only when that is somewhere new. A write that has to happen
    // again says something is putting the game's copy back — a restored chapter is the
    // expected way, anything else is a finding — and one line says it as well as a line a
    // frame would.
    if REPORTED_DIFFERENCE.swap(differs + 1, Ordering::Relaxed) != differs + 1 {
        log!(
            "joystick: the game's axis calibration was not this device's from +{differs:#x}; set from its caps"
        );
    }
}

/// One of these as the bytes it is, for the one question orb asks of a whole `JOYCAPSA`: where two of
/// them first differ, which is what the line about the calibration reports.
fn bytes_of(caps: &JoyCaps) -> &[u8] {
    unsafe { std::slice::from_raw_parts(std::ptr::from_ref(caps).cast(), size_of::<JoyCaps>()) }
}

fn poll() -> ! {
    // Below the game's, because a sample a millisecond old costs nobody anything and a
    // late frame costs the thing orb was written for.
    orb_api::thread::below_normal();
    let mut reported = None;
    let mut polls = 0;
    let mut total = 0;
    let mut caps = None;
    loop {
        let started = profile::now();
        // The host's own winmm rather than the entry `orb::joystick::install` redirected, and the cost
        // this module exists to move is right here.
        let (result, info) = orb_api::joystick::position(DEVICE, RETURN_ALL);
        // Only where one answered, and only where the last read did not: the caps of a
        // device that is not there cost what its position costs, and a device that is there
        // has the caps it had a moment ago.
        caps = match (result, caps) {
            (joyerr::NOERROR, None) => orb_api::joystick::caps(DEVICE),
            (joyerr::NOERROR, known) => known,
            _ => None,
        };
        let cost = profile::since(started);
        let sample = Sample { result, info, caps };
        sampled(sample);

        if reported != Some(result) {
            reported = Some(result);
            (polls, total) = (0, 0);
            log!("joystick: {}, read in {cost}us", describe(&sample));
        }
        polls += 1;
        total += cost;
        // Each line covers its own reads and no others: one that averaged everything since
        // the thread started would spend a long time walking the first read down — the slow one,
        // where winmm found the device — and read as though the reads were getting faster.
        // What device they are of stands in the line above rather than being asked again,
        // since naming it means another read of the thing this stops reading often.
        if polls == REPORT_READS {
            detail!(
                "joystick: {polls} reads, {}us each",
                total / i64::from(polls)
            );
            (polls, total) = (0, 0);
        }

        let wait = if result == joyerr::NOERROR {
            ATTACHED_MS
        } else {
            DETACHED_MS
        };
        // Never sooner than the read itself took, so a device slow enough to cost what no
        // device at all costs cannot hold a core of its own.
        orb_api::clock::sleep(wait.max((cost.max(0) / 1000) as u32));
    }
}

/// What a sample says, with the device named where one answered.
fn describe(sample: &Sample) -> String {
    match (sample.result, &sample.caps) {
        // Named all the same, since which phantom it is is the thing to look up: 413d:2104 is
        // what Windows leaves on joystick 0 while the pad it has is in XInput's second slot.
        (joyerr::NOERROR, Some(caps)) if !sample.is_a_pad() => {
            let (mid, pid) = (caps.manufacturer, caps.product);
            format!(
                "joystick 0 is mid={mid:04x} pid={pid:04x} \"{}\" with no buttons and no axes, \
                 which is no pad; orb's own menus will not be driven from it",
                name(caps.name),
            )
        }
        (joyerr::NOERROR, Some(caps)) => {
            let (mid, pid) = (caps.manufacturer, caps.product);
            let (buttons, axes) = (caps.buttons, caps.axes);
            let (low, high) = (caps.x_min, caps.x_max);
            format!(
                "mid={mid:04x} pid={pid:04x} \"{}\", {buttons} buttons, {axes} axes, X {low}..{high}",
                name(caps.name),
            )
        }
        (joyerr::NOERROR, None) => "a joystick whose caps do not read".to_owned(),
        (joyerr::PARMS, _) => "there is no joystick 0".to_owned(),
        (joyerr::UNPLUGGED, _) => "joystick 0 is unplugged".to_owned(),
        (other, _) => format!("joyGetPosEx returns {other}"),
    }
}

/// `szPname`, which winmm gives in the machine's code page and the log is written in UTF-8:
/// the name here read `Microsoft PC �W���C�X�e�B�b�N` before this converted it, and a line
/// nobody can read is no answer to which device it is.
fn name(field: [u8; 32]) -> String {
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(32);
    orb_api::codepage::text(&field[..end])
}

/// The pad as it was last sampled, and `None` while none is answering.
///
/// For the menus orb puts up itself. Those freeze the game, so the game's own reading of the pad
/// is not running either and a pad would do nothing at all on them; the sample this thread already
/// takes every few milliseconds is there to be read.
pub fn reading() -> Option<Reading> {
    let sample = latest().filter(Sample::is_a_pad)?;
    Some(Reading {
        buttons: sample.info.buttons,
        y: sample.info.y,
        pov: sample.info.pov,
    })
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

    /// The caps are 0x194 bytes and the game keeps its auto-repeat counter in the four
    /// immediately after them — `G_JOY_CAPS` + 0x194 is 0x69d8f4, which
    /// `GetControllerInput` counts a held direction in. A write that runs over lands there.
    #[test]
    fn a_calibration_write_stops_where_the_caps_do() {
        #[repr(C)]
        struct AsTheGameHasThem {
            caps: JoyCaps,
            auto_repeat: u32,
        }
        let mut theirs = AsTheGameHasThem {
            caps: JoyCaps::default(),
            auto_repeat: 0x0102_0304,
        };
        let ours = JoyCaps {
            x_max: 65535,
            buttons: 16,
            ..JoyCaps::default()
        };

        CALIBRATION.store(
            std::ptr::from_mut(&mut theirs.caps) as usize,
            Ordering::Relaxed,
        );
        unsafe { calibrate(&ours) };

        assert_eq!((theirs.caps.x_max, theirs.caps.buttons), (65535, 16));
        assert_eq!(theirs.auto_repeat, 0x0102_0304);

        // What the caps are put back over is a restored chapter's, which is why the handover
        // happens with every read and not once when the device turned up.
        theirs.caps.x_max = 0;
        unsafe { calibrate(&ours) };
        assert_eq!(theirs.caps.x_max, 65535);
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
