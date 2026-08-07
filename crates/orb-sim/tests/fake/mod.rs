//! What a launch is with none of the game in it: the display it runs on, the device orb draws
//! through, what a frame's own work costs, and the vocabulary a scenario drives any game by.
//!
//! **A scenario's whole vocabulary is the host and the input.** It says which window is in front,
//! presses keys, and runs frames. Everything else it reads back: the game's own memory, the game's
//! own records, and what orb put in the log.
//!
//! **Each scenario runs in a process of its own**, which is what lets every one of them run at once —
//! see [`in_its_own_process`]. A launch is a process, and orb is written that way.
//!
//! **Playing a game's part is that game's own half.** [`th06`] is 紅魔郷's — the address space, the
//! scenes it walks, the buttons it is played with and the file it keeps its scores in — and [`th07`] is
//! 妖々夢's, which is a frame and nothing else, that being all of the exe orb has read. Each brings its
//! own half and nothing of the other's; what they share is here. See
//! [docs/adr/0004](../../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md).

// Shared by every `scenario_*.rs` beside this module, and each drives the part of a game it is about —
// so what one file does not touch is not dead code, it is another file's. Nothing can see that:
// `dead_code` is worked out per binary, and this module is compiled into one per `scenario_*.rs`.
#![allow(dead_code)]

pub mod th06;
pub mod th07;

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::path::{Path, PathBuf};

use orb::recording::{Drawn, Quad, Screen as Recorded};
use orb_api::Hwnd;
use orb_core::profile;
use orb_sim::{Clock, Compose, Log};

/// The game's window. Any handle: what it is for is that the host says it is the one in front, which
/// is what makes a key down a key the game and orb both read.
pub const WINDOW: Hwnd = Hwnd(0x1234);

/// The names a frame's own record uses for the calls a loop makes into the game — see
/// [`Launched::asked`].
///
/// Four whichever loop is running: an update, the sounds handed over after it, a draw, and the frame
/// handed to the display. What differs between the two is the order, which is what a scenario reads
/// these back for.
pub const UPDATE: &str = "update";
pub const SOUND: &str = "sound";
pub const DRAW: &str = "draw";
pub const PRESENT: &str = "present";

/// How long a scenario waits before it presses a *direction* at a menu of orb's that has just come up.
///
/// Every one of them holds its keys off for frames of its own — the press that opened it is an edge
/// already spent, and the keys somebody was playing with belong to the run — and none of those counts
/// is something a scenario can see. The retry menu's 24 frames is the longest, so waiting past it
/// waits past all of them, and a scenario that presses after it is somebody who read the screen before
/// answering.
///
/// A `decide` needs no such wait: pressing it again costs nothing where the item under the cursor is
/// the one wanted, which is what [`Launched::press_until`] does. A direction cannot be repeated that
/// way — a list of three that wraps is on another item every press — so this is what the two-key
/// answers wait out.
pub const READS_KEYS_AFTER: u32 = 30;

/// How many presses a scenario makes before it gives up on a menu answering.
///
/// Well past the longest any of orb's own menus holds its keys off for — the retry menu's 24 frames,
/// against two frames a press — so a menu that has not answered by here is not going to. A scenario
/// counts presses rather than frames because how long a menu holds them off is the menu's business.
const PRESSES: u32 = 30;

/// How long the game's own work takes in a frame, as a scenario declares it.
///
/// The same shape as [`Compose`], and for the same reason: a frame's own update and draw are not one
/// number either. What is on screen decides them — a title menu and a stage 3 boss fight with 524
/// bullets up are not the same work — and now and then one frame costs far more than any of them, a
/// stage load being a quarter of a second of it. So a scenario says the middle, how far above it an
/// ordinary frame wanders, and what the occasional one costs.
///
/// A `usual_us` of nothing is a laid-out game left to itself: its update walks a few writes and no
/// simulated time passes at all, which is what every scenario that is not about the pacing wants.
#[derive(Clone, Copy)]
pub struct Work {
    pub usual_us: i64,
    /// How far over `usual_us` an ordinary frame may take, drawn per frame from the host's own stream.
    pub jitter_us: i64,
    pub spike_us: i64,
    /// One frame in this many. Zero for a game whose frames never cost more than the usual.
    pub spike_one_in: i64,
}

impl Work {
    /// The same every frame, for a scenario about the arithmetic around it.
    ///
    /// 694µs is what a real run's report line said the game's own drawing took — "(694us to draw + …)"
    /// — which is where the 700 every pacing scenario uses comes from.
    pub fn flat(us: i64) -> Self {
        Self {
            usual_us: us,
            jitter_us: 0,
            spike_us: us,
            spike_one_in: 0,
        }
    }

    /// A frame whose own work wanders, which is what one really does: half as much again over the
    /// quiet frames, which is about what a stage full of bullets is against a menu.
    pub fn wandering(us: i64) -> Self {
        Self {
            jitter_us: us / 2,
            ..Self::flat(us)
        }
    }

    /// And one that now and then costs a quarter of a second, which is a stage load.
    ///
    /// `one_in` rather than a count, so a scenario says how many its length will see. What the load
    /// must *not* do is buy the compositor anything — `scenario_pacing.rs`'s `load` section is that claim;
    /// this is the other half of it, which is that the rate comes back.
    pub fn loading(us: i64, one_in: i64) -> Self {
        Self {
            spike_us: 250_000,
            spike_one_in: one_in,
            ..Self::wandering(us)
        }
    }
}

/// The display the game's window is on, as a scenario declares it.
///
/// What the pacing is paced against, and the whole of what a scenario says about the host it is
/// running on: everything else the frame loop reads it reads through orb's own code.
pub struct Display {
    /// What the monitor the window is on reports, in whole Hz. `None` is one that will not say.
    pub monitor_hz: Option<u32>,
    /// What the compositor is timing, which need not be the same monitor. `None` is no compositor.
    pub compositor_hz: Option<u32>,
    /// What the compositor takes over a frame.
    pub compose: Compose,
    /// Which stream of wake delays the host has. Named in a failure so the run can be replayed.
    pub seed: u64,
    /// Whether to take the host's non-determinism away, leaving a metronome. Only for a scenario
    /// making a claim about arithmetic — no machine is one.
    pub metronome: bool,
}

/// The monitor the game's window goes on, as a scenario declares it — apart from [`Display`] because
/// this is what the *layout* reads and that is what the pacing reads.
///
/// Two numbers a scenario cannot otherwise move: how many pixels the panel has and will admit to, and
/// the frame this host costs to get a client area of a given size. See [`orb_sim::Windows`].
pub struct Panel {
    pub monitor: orb_sim::Monitor,
    pub frame: orb_sim::Frame,
    /// Whether the host refuses `SetProcessDPIAware`, which leaves every size scaled behind the game's
    /// back for the whole launch.
    pub refuses_dpi_awareness: bool,
}

impl Panel {
    /// This machine's: a 3840x2160 monitor that reads as 2560x1440 to a process that has not asked
    /// otherwise, with a 6x40 frame round a window of a chosen size.
    pub fn measured() -> Self {
        Self {
            monitor: orb_sim::Monitor::measured(),
            frame: orb_sim::Frame::MEASURED,
            refuses_dpi_awareness: false,
        }
    }
}

impl Display {
    /// One display, with the compositor timing it — what almost every machine has.
    pub fn agreed(hz: u32) -> Self {
        Self {
            monitor_hz: Some(hz),
            compositor_hz: Some(hz),
            compose: Compose::measured(),
            seed: 0,
            metronome: false,
        }
    }

    /// The game's window on one monitor while the compositor times another.
    pub fn split(monitor_hz: u32, compositor_hz: u32) -> Self {
        Self {
            monitor_hz: Some(monitor_hz),
            compositor_hz: Some(compositor_hz),
            compose: Compose::measured(),
            seed: 0,
            metronome: false,
        }
    }

    /// Nothing will say the rate, which is the one case paced by the clock.
    pub fn unknown() -> Self {
        Self {
            monitor_hz: None,
            compositor_hz: None,
            compose: Compose::measured(),
            seed: 0,
            metronome: false,
        }
    }

    /// The display every scenario that is not about the pacing runs on: one monitor at the rate the
    /// game was written for, with the compositor timing it. A whole multiple of sixty, so the frames
    /// go on its blanks and the loop paces the way a shipped run paces.
    pub fn ordinary() -> Self {
        Self::agreed(60)
    }
}

/// The half of a launch that is not the game: the device orb draws through, the directory it was
/// installed in, what a frame's own work costs and how unevenly it comes, and what the loop running
/// the game has asked of it.
///
/// Held by whichever game is being played rather than holding one — see [`Launched`] — because a
/// second game brings its own half and nothing of this.
pub struct Launch {
    /// The device orb draws through, with an overlay of the game's own on it: one for what orb draws
    /// and reads back — see [`Launched::says`] — and one for the screens the game draws itself.
    screen: Recorded,
    /// The directory the game is installed in, which is where orb reads `orb.yaml`, writes the runs
    /// left unfinished, and finds `font.ttf`.
    dir: PathBuf,
    /// Whether this launch has orb's own frame loop on, which is what decides whether
    /// [`Launched::frame`] calls that loop or the game's own.
    own_frame_loop: bool,
    /// How long the game's own work takes in a frame — see [`Launched::frame_takes`].
    work: Cell<Work>,
    /// The stream the frame's own unevenness is drawn from, which is the host's: a scenario names one
    /// seed and everything drawn in the run follows from it.
    noise: RefCell<orb_sim::Noise>,
    /// The tick each frame was handed over at, as the game's own `Present` records it.
    ///
    /// Beside the game's memory rather than in it, because a hand-over is not a fact about the run: it
    /// is one about the display, and no chapter restored underneath it rewinds what the screen has
    /// already been shown.
    handovers: RefCell<Vec<i64>>,
    /// What the loop running the game has asked of it, in the order it asked — see [`Launched::asked`].
    asked: RefCell<Vec<&'static str>>,
}

impl Launch {
    /// The device, the directory it was found in, and the run's own unevenness.
    ///
    /// `dir` is the directory standing in for the one the game is installed in — see [`scratch`] — and
    /// `seed` is the one number a failure names, everything the host draws following from it.
    pub fn new(dir: PathBuf, seed: u64, own_frame_loop: bool) -> Self {
        Self {
            screen: Recorded::over(&dir.join("font.ttf")),
            dir,
            own_frame_loop,
            work: Cell::new(Work::flat(0)),
            noise: RefCell::new(orb_sim::Noise::seeded(seed)),
            handovers: RefCell::new(Vec::new()),
            asked: RefCell::new(Vec::new()),
        }
    }

    /// The device the game shows through and orb draws its own on.
    pub fn device(&self) -> *mut orb_core::d3d8::Device {
        self.screen.device()
    }

    /// Where the game is installed, which is where orb reads `orb.yaml` and writes its log.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Writes down that the loop asked the game for this, in the order it asked.
    pub fn asked_for(&self, what: &'static str) {
        self.asked.borrow_mut().push(what);
    }

    /// Writes down the tick a frame was handed over at.
    pub fn handed_over(&self, tick: i64) {
        self.handovers.borrow_mut().push(tick);
    }

    /// What the game writes of its own, which for a laid-out one is whichever screen is worth reading
    /// back off the recording.
    pub fn writes(&self, text: &str, x: f32, y: f32, color: u32) {
        self.screen.writes(text, x, y, color);
    }
}

/// What a scenario does to whatever game orb is attached to: run a frame of it, and read back the host
/// it is laid out in, the device it draws through and the log orb wrote.
///
/// The four below are the game's own to answer — its window, its own frame, the host its memory is in,
/// and the half of a launch it holds. Everything under them is written once and is every game's.
pub trait Launched {
    /// The object the game calls its own whole frame on, which is what a loop is handed.
    fn frame_window(&self) -> *mut c_void;

    /// The game's own whole frame, in its own order, and what that frame answered.
    fn own_render(&self) -> i32;

    /// The host the game is laid out in, for a scenario that says which window is in front.
    fn sim(&self) -> &orb_sim::Sim;

    fn launch(&self) -> &Launch;

    /// One frame of the game, as its own loop runs one, and what that frame answered.
    ///
    /// Where orb's own frame loop is on, that loop *is* the frame: the game's loop calls it and it runs
    /// the update before the draw, which is the frame of input lag removed. Where the launch turned it
    /// off, the game's own order is what runs — see [`own_render`](Launched::own_render).
    fn frame(&self) -> i32 {
        if self.launch().own_frame_loop {
            unsafe { orb::render(self.frame_window()) }
        } else {
            self.own_render()
        }
    }

    fn frames(&self, count: u32) {
        for _ in 0..count {
            self.frame();
        }
    }

    /// How long the game's own work takes in a frame: its update, its sounds and its draw, as one span.
    ///
    /// A scenario saying so, the way it says the player was hit. A laid-out game's update walks a few
    /// writes and would take no time at all, and what a rate is judged against is a frame whose work
    /// takes as long as a real game's does — the whole question the pacing answers is where the wait
    /// goes around work of that size, and how unevenly it comes.
    fn frame_takes(&self, work: Work) {
        self.launch().work.set(work);
    }

    /// What this frame's own work costs, drawn: the usual, or now and then the spike.
    fn work_this_frame(&self) -> i64 {
        let launch = self.launch();
        let work = launch.work.get();
        let mut noise = launch.noise.borrow_mut();
        if work.spike_one_in > 0 && noise.up_to(work.spike_one_in - 1) == 0 {
            return work.spike_us;
        }
        if work.jitter_us > 0 {
            return work.usual_us + noise.up_to(work.jitter_us);
        }
        work.usual_us
    }

    /// When each frame so far was handed over, in microseconds, which is what a rate is read off.
    ///
    /// Microseconds rather than the counter's own ticks, because what reads these is arithmetic over
    /// values and a tick is a fact about the host they were counted on.
    fn handovers_us(&self) -> Vec<i64> {
        self.launch()
            .handovers
            .borrow()
            .iter()
            .map(|tick| Clock::micros_for_ticks(*tick))
            .collect()
    }

    /// How far apart the blanks of the compositor this scenario declared are, in microseconds.
    ///
    /// # Panics
    /// On a display declared without a compositor, there being no blanks to be apart.
    fn refresh_period_us(&self) -> i64 {
        let period = self
            .sim()
            .display()
            .compositor_period()
            .expect("a display this scenario declared a compositor for");
        Clock::micros_for_ticks(period)
    }

    /// What the loop running this game has asked of it since the last
    /// [`forget_asked`](Launched::forget_asked), in the order it asked: one of [`UPDATE`], [`SOUND`],
    /// [`DRAW`] or [`PRESENT`] per call it made in.
    ///
    /// Which is the difference between the two loops rather than a detail of either. The game's own
    /// order is the draw before the update, so everything on screen is one update behind the input that
    /// produced it; orb's is the other way round, and that is the frame of input lag removed.
    fn asked(&self) -> Vec<&'static str> {
        self.launch().asked.borrow().clone()
    }

    /// Forgets it, so that what a scenario reads is the frame it means rather than every frame since the
    /// game started.
    fn forget_asked(&self) {
        self.launch().asked.borrow_mut().clear();
    }

    /// Runs frames until `done`.
    ///
    /// # Panics
    /// After `limit` frames, naming what was being waited for: a scenario that waits for something
    /// which never happens should say what it was waiting for rather than time out.
    fn frames_until(&self, what: &str, limit: u32, done: impl Fn() -> bool) {
        for _ in 0..limit {
            if done() {
                return;
            }
            self.frame();
        }
        panic!("{what} did not happen in {limit} frame(s)");
    }

    /// Runs frames until orb has written one more line holding `what`.
    ///
    /// What a scenario does instead of asking the pacing for a number: the `Pacing` a run is paced by is
    /// the frame loop's own and nothing outside orb holds it, so the line is waited for — a reporting
    /// period at the most, and every frame of the waiting is an ordinary frame of the run.
    ///
    /// # Panics
    /// After a reporting period, with the whole log: a scenario waiting for a line orb never wrote is
    /// about to assert on a number that was never said.
    fn frames_until_the_log_holds_another(&self, what: &str) {
        let holding = || {
            self.log()
                .lines()
                .iter()
                .filter(|line| line.contains(what))
                .count()
        };
        let before = holding();
        for _ in 0..=profile::INTERVAL {
            self.frame();
            if holding() > before {
                return;
            }
        }
        panic!(
            "orb wrote no {what:?} line in {} frames:\n  {}",
            profile::INTERVAL,
            self.log().lines().join("\n  ")
        );
    }

    /// Presses a key for one frame and lets it go, which is what a press is: an edge.
    ///
    /// The frame is run with the key down and the next with it up, so nothing that reads the
    /// keyboard — orb's own menus or the game's input read — sees it as held.
    fn press(&self, key: u8) {
        self.keyboard().set(key, true);
        self.frame();
        self.keyboard().set(key, false);
        self.frame();
    }

    /// Presses `key` again and again until `done`, and says how many presses that took.
    ///
    /// What somebody sitting at the keyboard does, and what a scenario has to do rather than count
    /// frames: every menu orb puts up holds its keys off for frames of its own — the press that opened
    /// it is an edge already spent — and how many is the menu's business and not the scenario's.
    ///
    /// Stops on the frame it happens on, which is what a scenario reading the game's memory
    /// afterwards needs: the frame a menu is answered on is the frame a chapter is put back on, and
    /// one more frame of the game is one frame past it.
    ///
    /// # Panics
    /// After [`PRESSES`], naming what was being waited for.
    fn press_until(&self, key: u8, what: &str, done: impl Fn() -> bool) {
        for _ in 0..PRESSES {
            if done() {
                return;
            }
            self.keyboard().set(key, true);
            self.frame();
            self.keyboard().set(key, false);
            if done() {
                return;
            }
            self.frame();
        }
        panic!("{what} did not happen in {PRESSES} press(es) of {key:#04x}");
    }

    /// Where the game is installed, which is where the runs it has left unfinished are kept.
    fn dir(&self) -> &Path {
        self.launch().dir()
    }

    fn keyboard(&self) -> &orb_sim::Keyboard {
        self.sim().keyboard()
    }

    fn log(&self) -> &Log {
        self.sim().log()
    }

    /// What has been asked of the device since the last [`forget`](Launched::forget).
    fn drawn(&self) -> Drawn {
        self.launch().screen.recording().drawn()
    }

    /// The quads that drew `text`, and the colour each was drawn in — which for an item of one of
    /// orb's menus is what says whether the cursor was on it.
    fn says(&self, text: &str) -> Vec<Quad> {
        self.launch().screen.says(text)
    }

    /// Forgets what has been drawn, so that what a scenario asks about is the frames it means rather
    /// than every frame since the game started.
    fn forget(&self) {
        self.launch().screen.recording().clear();
    }

    /// One frame with nothing remembered before it, so that what [`says`](Launched::says) answers is
    /// the screen as it is now rather than everything that has been on it.
    fn one_frame(&self) {
        self.forget();
        self.frame();
    }
}

/// What names the process a scenario runs in as the one that is *being* the launch, and which scenario
/// it is: read by the child, set by the parent.
const SCENARIO: &str = "ORB_SCENARIO";

/// Runs `scenario` in a process of its own, so that every one of these can run beside every other.
///
/// **A launch is a process, and orb is written that way.** Its runtime, the record of what a run has
/// pressed, which of the two files a score goes to and the device it draws through are one apiece, the
/// way they are in the game — and the runtime cannot be made one per thread even for a test's sake:
/// `DllMain` writes it on the thread the launcher's remote `LoadLibraryW` runs on, and the frame hook
/// reads it on the game's main thread. So two scenarios in one process have to take turns, which they
/// did on the recording device's lock. Taking turns is not being able to run at once.
///
/// So a scenario spawns this same binary again, told to run this one test and nothing else, and reports
/// what the child made of it. The child is the launch and this side is the harness, which is also what
/// makes a scenario that hangs or takes its process down name itself.
///
/// Where the harness is not naming its threads — `--test-threads=1`, which puts every test on `main` —
/// there is nothing to tell the child to run, so the scenario runs here. Serially, which is what asking
/// for one thread asked for.
pub fn in_its_own_process(scenario: impl FnOnce()) {
    if std::env::var_os(SCENARIO).is_some() {
        return scenario();
    }
    let named = std::thread::current().name().map(str::to_owned);
    let Some(name) = named.filter(|name| name != "main") else {
        return scenario();
    };
    let exe = std::env::current_exe().expect("the binary this scenario is in");
    let run = std::process::Command::new(&exe)
        // One thread in the child, so the test it runs is the whole of what that process does.
        .args(["--exact", &name, "--nocapture", "--test-threads=1"])
        .env(SCENARIO, &name)
        .output()
        .unwrap_or_else(|error| panic!("running {name} in a process of its own: {error}"));
    let said = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();
    // Said out loud rather than trusted: a filter that matched nothing is a child that passes without
    // running anything, which would be a scenario reported as green for never having happened.
    assert!(
        said(&run.stdout).contains("1 passed"),
        "{name} did not run in the process it was given:\n{}{}",
        said(&run.stdout),
        said(&run.stderr),
    );
    assert!(
        run.status.success(),
        "{name} failed in its own process:\n{}{}",
        said(&run.stdout),
        said(&run.stderr),
    );
}

/// A directory standing in for the one the game is installed in, empty of everything a run left
/// behind and holding the one file orb reads out of it.
pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("orb-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    // `font.ttf`, which orb builds its overlay from and without which it asks none of its questions:
    // `Font::load` hands the path to `AddFontResourceExW`, so it has to be a font that is there.
    // Windows' own, under the name the game's is installed as — GDI substituting a face is something
    // `Font::load` already survives, and a path that is not a font is not.
    let windows = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_owned());
    let font = Path::new(&windows).join("Fonts").join("arial.ttf");
    std::fs::copy(&font, dir.join("font.ttf"))
        .unwrap_or_else(|error| panic!("{}: {error}", font.display()));
    dir
}
