//! A 東方紅魔郷 that is not the real one: it owns its own memory, advances its own state, and calls
//! orb's hooks where the real game's code calls them.
//!
//! **It plays the game's part, not the address space's.** Nothing here hands orb an opinion about
//! the run. Its state is the memory laid out by `image`, so `read_state` is how orb learns what the
//! run is — as in production — and a chapter restored underneath it takes the run back with it,
//! because there is nothing else for the run to be. Anything kept beside that memory would be a
//! second source of truth, which is the mistake this replaced: a scenario that told orb what the
//! state was while the memory said something else.
//!
//! **A scenario's whole vocabulary is the host and the input.** It says which window is in front,
//! presses keys, and runs frames. Everything else it reads back: the game's own memory, the game's
//! own records, and what orb put in the log.
//!
//! **Its own loop calls orb's frame loop, as the real game's does.** The `Present` and the sound that
//! loop makes are addresses the game hands over — see `Game::frame_calls` — so this game hands over two
//! of its own, and its `present` is where a scenario counts a frame handed over. A launch started
//! `--no-frame-loop` is the game's own draw-then-update order instead, with orb's update and draw hooks
//! in the middle of it, which is the other configuration orb ships.
//!
//! **Each scenario runs in a process of its own**, which is what lets every one of them run at once —
//! see [`in_its_own_process`]. A launch is a process, and orb is written that way.
//!
//! Where a number here is 紅魔郷's it says so. Where it is this game's own — how far the player moves,
//! when its boss arrives — it says that too: what a scenario is about is that the same buttons from
//! the same seed arrive at the same place, not how fast Reimu is.

// Shared by every scenario in `tests/`, and each drives the part of a game it is about — so what one
// file does not touch is not dead code, it is another file's. Nothing can see that: `dead_code` is
// worked out per binary, and this module is compiled into one per scenario.
#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::path::{Path, PathBuf};

use orb::recording::{Drawn, Quad, Screen as Recorded};
use orb_api::Hwnd;
use orb_config::Config;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{
    Boss, FrontEnd, Image, Mapping, Player, Playing, Pushed, Reproducing, Scene, Screen,
    Supervising, item, joy_state,
};
use orb_core::game::{Game, RunStart};
use orb_sim::{Compose, Log, keys};

/// The game's window. Any handle: what it is for is that the host says it is the one in front, which
/// is what makes a key down a key the game and orb both read.
pub const WINDOW: Hwnd = Hwnd(0x1234);

/// What the game's own chain walk answers while the game is running.
///
/// Neither of the two below, which orb reads as the game leaving. A run whose resume played 7476
/// updates in inside one frame (`DONE.md`) is measured proof that the real walk does not answer zero
/// while a stage is running, since orb's playback stops on that.
const CHAIN_CARRIED_ON: i32 = 1;

/// And what it answers when the game is leaving: zero, which orb reads as the game having asked to stop,
/// and `-1`, which is the walk having failed. Both of 紅魔郷's own.
pub const CHAIN_LEFT: i32 = 0;
pub const CHAIN_FAILED: i32 = -1;

/// What a whole frame answers while the game is running, which is 紅魔郷's own `Render` answering that
/// the loop above it should call it again. Zero, and the two above zero are the game leaving — which is
/// what orb's frame loop turns the chain's two exits into.
pub const FRAME_KEPT_RUNNING: i32 = 0;
pub const FRAME_LEFT: i32 = 1;
pub const FRAME_FAILED: i32 = 2;

/// The names a frame's own record uses for the calls a loop makes into the game — see [`Fake::asked`].
pub const UPDATE: &str = "update";
pub const SOUND: &str = "sound";
pub const DRAW: &str = "draw";
pub const PRESENT: &str = "present";

/// The bits of the word the game's own input read hands back, which are 紅魔郷's own: the three masks
/// orb reads them through are made of these — `menu_decide` is `SHOOT | ENTER`, `menu_cancel` is
/// `BOMB | MENU`, and `run_input` is every one of these but those two. [`Fake::attach`] holds them to
/// exactly that.
mod button {
    pub const SHOOT: u16 = 0x0001;
    pub const BOMB: u16 = 0x0002;
    pub const MENU: u16 = 0x0008;
    pub const UP: u16 = 0x0010;
    pub const DOWN: u16 = 0x0020;
    pub const LEFT: u16 = 0x0040;
    pub const RIGHT: u16 = 0x0080;
    pub const ENTER: u16 = 0x1000;
}

/// The two arrow keys orb's own menus do not read, so `orb_sim::keys` does not name them.
///
/// The four it does read come from there, which is what makes a scenario pressing `Z` at one of orb's
/// questions and this game reading its shot key the same key by construction rather than by two
/// numbers that happen to agree.
const LEFT: u8 = 0x25;
const RIGHT: u8 = 0x27;

/// Which key is which button, as the game's own configuration maps them.
const MAP: [(u8, u16); 6] = [
    (keys::Z, button::SHOOT),
    (keys::X, button::BOMB),
    (keys::UP, button::UP),
    (keys::DOWN, button::DOWN),
    (LEFT, button::LEFT),
    (RIGHT, button::RIGHT),
];

/// The lives, bombs and power a run starts a stage with.
const FRESH: (i8, i8, u16) = (2, 3, 0);

/// When the fight this game's stage one has arrives, and when its boss puts a card up, in the stage's
/// own frames.
///
/// This game's own numbers, and far enough apart that each is a chapter of its own: `chapter.rs`'s
/// floor on how short a chapter may be would otherwise fold the card into the attack before it, which
/// is what happens to a card 紅魔郷 declares a frame or two after the attack begins.
pub const BOSS_ARRIVES: u32 = 400;
pub const CARD_STARTS: u32 = 500;
/// And where it moves on to its next attack, which is the card being over: a fight's own boundaries
/// are in no table, so this is the fourth chapter of a stage whose table has one entry at script frame
/// 4472 and nothing before it.
pub const ATTACK_CHANGES: u32 = 700;

/// Which of the game's 64 spell card records its boss's card is. Any of them; what a scenario reads
/// back is the count of attempts against this one.
pub const CARD: i32 = 3;

/// How long a scenario waits before it presses a *direction* at a menu of orb's that has just come up.
///
/// Every one of them holds its keys off for frames of its own — the press that opened it is an edge
/// already spent, and the keys somebody was playing with belong to the run — and none of those counts
/// is something a scenario can see. The retry menu's 24 frames is the longest, so waiting past it
/// waits past all of them, and a scenario that presses after it is somebody who read the screen before
/// answering.
///
/// A `decide` needs no such wait: pressing it again costs nothing where the item under the cursor is
/// the one wanted, which is what [`Fake::press_until`] does. A direction cannot be repeated that way —
/// a list of three that wraps is on another item every press — so this is what the two-key answers
/// wait out.
pub const READS_KEYS_AFTER: u32 = 30;

/// How many items the title menu's cursor walks, which is the eight the game bounds it to.
const TITLE_ITEMS: i32 = 8;

/// Where this game's ranking screen puts its rows, and the colour it writes them in. Its own layout:
/// what a scenario reads off it is which text is at which of these, and the numbers themselves are
/// nothing but somewhere to put them.
const RANKING_TOP: f32 = 96.0;
const RANKING_LINE: f32 = 24.0;
const RANKING_LEFT: f32 = 64.0;
const RANKING_ATTEMPTS: f32 = 400.0;
const INK: u32 = 0xffff_ffff;

/// How long the result screen a run ends into stands before it leaves for the title.
const RESULT_FRAMES: i32 = 10;

/// How many presses a scenario makes before it gives up on a menu answering.
///
/// Well past the longest any of orb's own menus holds its keys off for — the retry menu's 24 frames,
/// against two frames a press — so a menu that has not answered by here is not going to. A scenario
/// counts presses rather than frames because how long a menu holds them off is the menu's business.
const PRESSES: u32 = 30;

/// How far the player moves in a frame, and the field it is held inside — the field being 紅魔郷's own
/// play area and the speed this game's.
const SPEED: f32 = 4.0;

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
    /// must *not* do is buy the compositor anything — `pacing_load.rs` is that claim; this is the
    /// other half of it, which is that the rate comes back.
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

/// A game laid out, with orb attached to it.
///
/// Its own memory is the `Image`; everything else here is what a process has and an address space
/// does not — the device orb draws through, and which run its front end is offering.
pub struct Fake {
    image: Image,
    /// The device orb draws through, with an overlay of this game's own on it: one for what orb draws
    /// and reads back — see [`Fake::says`] — and one for the screens this game draws itself.
    screen: Recorded,
    run: RunStart,
    /// The directory the game is installed in, which is where orb reads `orb.yaml`, writes the runs
    /// left unfinished, and finds `font.ttf`.
    dir: PathBuf,
    /// The score file this game keeps, as the bytes of the record it holds about spell cards — which
    /// is what `Game::captures` and `Game::set_captures` are the two halves of.
    ///
    /// A field rather than a file on disk: what a scenario reads is the number, and *which* file a
    /// mode's ranking is written to is decided by an import hook on `CreateFileW` that no test can
    /// install. What matters here is that there is a file at all, because the game reads it in two
    /// places — building a run and building the ranking — and writes it where the ranking goes down.
    /// Outside the memory, so a chapter restored does not rewind it, which is true of a real file too.
    file: RefCell<Vec<u8>>,
    /// What the pad is doing, as a scenario is pushing it.
    ///
    /// Beside the memory rather than in it, and that is what a device is: the game's own read asks the
    /// controller every frame and the answer is never anywhere a snapshot could rewind.
    pushed: Cell<Pushed>,
    /// Set by a scenario for the frame the player is hit on.
    ///
    /// The one thing about a run that a laid-out game cannot do for itself: there are no bullets
    /// here, so being hit is a scenario saying so where in a real run it is a scenario dodging badly.
    hit: Cell<bool>,
    /// Whether this launch has orb's own frame loop on, which is what decides whether [`Fake::frame`]
    /// calls that loop or the game's own.
    own_frame_loop: bool,
    /// How long the game's own work takes in a frame — see [`Fake::frame_takes`].
    work: Cell<Work>,
    /// The stream the frame's own unevenness is drawn from, which is the host's: a scenario names one
    /// seed and everything drawn in the run follows from it.
    noise: RefCell<orb_sim::Noise>,
    /// The tick each frame was handed over at, as this game's own `Present` records it.
    ///
    /// Beside the memory for the same reason the pad's own state is: a hand-over is not a fact about
    /// the run, it is one about the display, and no chapter restored underneath it rewinds what the
    /// screen has already been shown.
    handovers: RefCell<Vec<i64>>,
    /// What the loop running this game has asked of it, in the order it asked — see [`Fake::asked`].
    asked: RefCell<Vec<&'static str>>,
    /// What this game's chain walk answers, which a scenario changes to say the game is leaving.
    answers: Cell<i32>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory. Last,
    /// and never dropped: see [`Fake::attach`].
    _installed: orb_api::Installed,
}

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into.
    ///
    /// Those are plain `extern` functions with nothing but the ABI's arguments — the same reason
    /// `recording.rs` keeps its record in a static — so where the real game would reach its own
    /// globals, this reaches the game.
    static RUNNING: Cell<*const Fake> = const { Cell::new(std::ptr::null()) };
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

/// The game this thread is running.
fn running() -> &'static Fake {
    let fake = RUNNING.get();
    assert!(!fake.is_null(), "no game has been attached on this thread");
    unsafe { &*fake }
}

impl Fake {
    /// Lays the game out, gives it a window and a device, and attaches orb to it.
    ///
    /// `name` names the directory that stands in for the one the game is installed in: `orb.yaml` is
    /// read from it, the runs left unfinished are kept under it, and `font.ttf` is copied into it.
    /// `settings` is where a scenario says what this launch was started with.
    ///
    /// Boxed and owned by whoever asked for it, so that a scenario file can hold more than one game
    /// over its lifetime: what serialises them is the recording device's own lock, which the game
    /// before has to be dropped to release — see [`Recording::new`](orb::recording::Recording::new).
    ///
    /// Boxed rather than returned by value because the hooks find it through a pointer, and a value
    /// moved out of this function would leave that pointer behind. Its `Drop` is the game closing, in
    /// the one order that works: the runtime first, so orb's overlay is released through a device that
    /// is still there, then the device, then the simulated Windows it was all read through.
    pub fn attach(name: &str, run: RunStart, settings: impl FnOnce(&mut Config)) -> Box<Self> {
        Self::attach_to_display(Display::ordinary(), name, run, settings)
    }

    /// And on a display a scenario says the whole of, for the ones the frame loop's pacing is about.
    pub fn attach_to_display(
        display: Display,
        name: &str,
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        // The map above is only the game's if orb reads the same bits through its own masks.
        assert_eq!(
            Th06.menu_decide(),
            button::SHOOT | button::ENTER,
            "the game's decide is not the bits this game hands back",
        );
        assert_eq!(
            Th06.menu_cancel(),
            button::BOMB | button::MENU,
            "the game's back is not the bits this game hands back",
        );
        // And every button this game is played with is one a run's own record keeps: the pause button
        // is deliberately outside those, so a map that grew to include it would fail here — which is
        // what should happen, since a resume feeding it back would open a menu instead of playing.
        let played_with = MAP.iter().fold(0, |word, (_, button)| word | button);
        assert_eq!(
            Th06.run_input() & played_with,
            played_with,
            "a button this game is played with is not one a run is written down with",
        );

        let dir = scratch(name);
        let mut config =
            Config::load_beside(&dir.join("th06.exe")).expect("a directory with no orb.yaml in it");
        // The memory hooks patch an import table there is none of.
        config.track_memory = false;
        settings(&mut config);

        let image = Image::laid_out_seeded(display.seed);
        let screen = Recorded::over(&dir.join("font.ttf"));
        let installed = image.enter();
        image.shows_through(screen.device(), WINDOW);
        // Where the game is installed, which is where orb writes its log: beside the exe, because that
        // is where `orb.yaml` and the launcher are.
        image.sim().set_host_exe(dir.join("th06.exe"));
        // The display, in front of orb being attached: `configure` reads the desktop's own rate before
        // there is a window to ask about, and a rate written down after that would be read a second
        // late — the first second of the run paced against nothing.
        let sim = image.sim();
        sim.display().set_monitor_hz(display.monitor_hz);
        sim.display().set_desktop_hz(display.monitor_hz);
        sim.display().set_foreground(WINDOW);
        if let Some(hz) = display.compositor_hz {
            sim.display().attach_compositor(
                sim.clock().peek(),
                sim.clock().frequency() / i64::from(hz),
                display.compose,
            );
        }
        if display.metronome {
            sim.display().as_a_metronome();
        }
        // A controller, mapped the way this game's configuration maps one. The numbers are its own —
        // a real one's come out of the file the game's own options screen writes — and what a scenario
        // needs of them is that orb reads a pad's buttons through this mapping and not around it.
        image.controller(
            poll as *const () as usize,
            acquire as *const () as usize,
            read_state as *const () as usize,
        );
        image.maps_the_pad(MAPPING);
        // What its score file holds to begin with: a record of the card its boss is on, never tried.
        // Without the record `count_card_attempt` refuses to count, a zeroed one being a card that is
        // not there rather than a card nobody has reached.
        image.card_record(CARD, 0);
        let file = RefCell::new(unsafe { Th06.captures() });
        // Where the game starts: its front end, on the title menu, on the item that starts a run.
        image.supervising(Supervising {
            running: Scene::FrontEnd,
            wanted: Scene::FrontEnd,
        });
        image.front_end(FrontEnd {
            screen: Screen::Title,
            cursor: item::GAME_START,
            frames: 0,
        });

        let fake = Box::new(Self {
            image,
            screen,
            run,
            dir,
            file,
            pushed: Cell::new(Pushed::none()),
            hit: Cell::new(false),
            own_frame_loop: config.own_frame_loop,
            work: Cell::new(Work::flat(0)),
            // The host's own seed, so a failure names one number and everything the run drew follows.
            noise: RefCell::new(orb_sim::Noise::seeded(display.seed)),
            handovers: RefCell::new(Vec::new()),
            asked: RefCell::new(Vec::new()),
            answers: Cell::new(CHAIN_CARRIED_ON),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        unsafe {
            orb::attach_to(
                &TH06,
                config,
                fake.image.data(),
                orb::Originals {
                    update,
                    draw,
                    input,
                    stage_building,
                    stage_begun,
                    unlocks_read,
                    ranking_read,
                    render: own_render,
                    play_sounds,
                    present,
                },
            )
        };
        fake
    }

    /// One frame of the game, as its own loop runs one, and what that frame answered.
    ///
    /// Where orb's own frame loop is on, that loop *is* the frame: the game's loop calls it and it runs
    /// the update before the draw, which is the frame of input lag removed. Where the launch turned it
    /// off, the game's own draw-then-update order is what runs — see [`own_render`].
    pub fn frame(&self) -> i32 {
        let window = self.image.game_window_object() as *mut c_void;
        if self.own_frame_loop {
            unsafe { orb::render(window) }
        } else {
            own_render(window)
        }
    }

    pub fn frames(&self, count: u32) {
        for _ in 0..count {
            self.frame();
        }
    }

    /// How long the game's own work takes in a frame: its update, its sounds and its draw, as one span.
    ///
    /// A scenario saying so, the way it says the player was hit. A laid-out game's update walks a few
    /// writes and would take no time at all, and what a rate is judged against is a frame whose work
    /// takes as long as 紅魔郷's does — the whole question the pacing answers is where the wait goes
    /// around work of that size, and how unevenly it comes.
    pub fn frame_takes(&self, work: Work) {
        self.work.set(work);
    }

    /// What this frame's own work costs, drawn: the usual, or now and then the spike.
    fn work_this_frame(&self) -> i64 {
        let work = self.work.get();
        let mut noise = self.noise.borrow_mut();
        if work.spike_one_in > 0 && noise.up_to(work.spike_one_in - 1) == 0 {
            return work.spike_us;
        }
        if work.jitter_us > 0 {
            return work.usual_us + noise.up_to(work.jitter_us);
        }
        work.usual_us
    }

    /// The tick each frame so far was handed over at, which is what a rate is read off.
    pub fn handovers(&self) -> Vec<i64> {
        self.handovers.borrow().clone()
    }

    /// What the loop running this game has asked of it since the last [`forget_asked`](Self::forget_asked),
    /// in the order it asked: one of [`UPDATE`], [`SOUND`], [`DRAW`] or [`PRESENT`] per call it made in.
    ///
    /// Which is the difference between the two loops rather than a detail of either. 紅魔郷's own order is
    /// the draw before the update, so everything on screen is one update behind the input that produced
    /// it; orb's is the other way round, and that is the frame of input lag removed.
    pub fn asked(&self) -> Vec<&'static str> {
        self.asked.borrow().clone()
    }

    /// Forgets it, so that what a scenario reads is the frame it means rather than every frame since the
    /// game started.
    pub fn forget_asked(&self) {
        self.asked.borrow_mut().clear();
    }

    /// What this game's chain walk answers from here on: [`CHAIN_CARRIED_ON`] until a scenario says
    /// otherwise, and [`CHAIN_LEFT`] or [`CHAIN_FAILED`] to say the game is going.
    pub fn chain_answers(&self, result: i32) {
        self.answers.set(result);
    }

    /// Runs frames until `done`.
    ///
    /// # Panics
    /// After `limit` frames, naming what was being waited for: a scenario that waits for something
    /// which never happens should say what it was waiting for rather than time out.
    pub fn frames_until(&self, what: &str, limit: u32, done: impl Fn() -> bool) {
        for _ in 0..limit {
            if done() {
                return;
            }
            self.frame();
        }
        panic!("{what} did not happen in {limit} frame(s)");
    }

    /// Presses a key for one frame and lets it go, which is what a press is: an edge.
    ///
    /// The frame is run with the key down and the next with it up, so nothing that reads the
    /// keyboard — orb's own menus or the game's input read — sees it as held.
    pub fn press(&self, key: u8) {
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
    pub fn press_until(&self, key: u8, what: &str, done: impl Fn() -> bool) {
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

    /// Pushes the pad, or lets it go: what the game's own read of its controller will answer with from
    /// here on.
    pub fn push(&self, pushed: Pushed) {
        self.pushed.set(pushed);
    }

    /// Says the run on screen is a replay being watched rather than one somebody is playing, and asks
    /// for it.
    ///
    /// A scenario saying so, the way it says the player was hit: a replay is started from the game's
    /// own replay menu, and this game's front end has the two screens orb asks a question over and
    /// nothing else. What it stands in for is the flag the game sets, which is what orb reads.
    pub fn watches_a_replay(&self) {
        self.image.watching_a_replay();
        self.image.chose(&self.run);
        self.image.supervising(Supervising {
            running: Scene::Playing,
            wanted: Scene::FrontEnd,
        });
    }

    /// Says the player was hit, for the next frame of the stage.
    pub fn hit(&self) {
        self.hit.set(true);
    }

    /// What the game's memory says the run is, read the way the frame hook reads it: every field
    /// parsed back out of the memory rather than off the addresses this game wrote.
    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th06.read_state() }
    }

    /// Where the game is installed, which is where the runs it has left unfinished are kept.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    /// The host this game is laid out in, for a scenario that says which window is in front.
    pub fn sim(&self) -> &orb_sim::Sim {
        self.image.sim()
    }

    pub fn keyboard(&self) -> &orb_sim::Keyboard {
        self.image.sim().keyboard()
    }

    pub fn log(&self) -> &Log {
        self.image.sim().log()
    }

    /// What has been asked of the device since the last [`forget`](Self::forget).
    pub fn drawn(&self) -> Drawn {
        self.screen.recording().drawn()
    }

    /// The quads that drew `text`, and the colour each was drawn in — which for an item of one of
    /// orb's menus is what says whether the cursor was on it.
    pub fn says(&self, text: &str) -> Vec<Quad> {
        self.screen.says(text)
    }

    /// Forgets what has been drawn, so that what a scenario asks about is the frames it means rather
    /// than every frame since the game started.
    pub fn forget(&self) {
        self.screen.recording().clear();
    }

    /// One frame with nothing remembered before it, so that what [`says`](Self::says) answers is the
    /// screen as it is now rather than everything that has been on it.
    pub fn one_frame(&self) {
        self.forget();
        self.frame();
    }

    /// The game at its title menu, past the frames its own screen ignores a press for and with an
    /// overlay for orb to draw a question with — which is where every scenario about the question that
    /// chooses a mode starts, and where one comes back to after answering.
    ///
    /// # Panics
    /// If the game does not get there, naming which half was missing.
    pub fn at_the_title_menu(&self) {
        self.frames_until("an overlay", 8, || self.log().said("overlay: ready"));
        self.frames_until("the title menu ready to act on a press", 120, || {
            let front = self.image.front_end_now();
            self.image.scene() == Scene::FrontEnd
                && front.screen == Screen::Title
                && front.acts_on_a_press()
        });
    }

    /// The whole of what the game does in one update, in the order 紅魔郷 does it: its supervisor
    /// first, which is the only reader of the keyboard, and then the jobs that supervisor registered.
    fn update(&self) {
        // What the frame's own work costs, which is the game's and not orb's: the update, the sounds
        // handed over after it and the draw, as one span, since what the pacing is judged on is how long
        // a frame took between its turn and being handed over. Nothing by default — a laid-out game
        // walks a few writes — and a scenario about the rate says the size and its unevenness. See
        // `frame_takes`.
        self.sim().clock().advance_micros(self.work_this_frame());
        // `Supervisor::OnUpdate`'s first act: `g_LastFrameInput = g_CurFrameInput; g_CurFrameInput =
        // GetInput()`. Through orb's hook, which is where a run's buttons are written down and where
        // a run being played back into place is handed the ones it pressed.
        let last = self.image.input_now();
        let word = orb::get_input();
        self.image.input(word);
        let pressed = |mask: u16| word & mask != 0 && word & mask != last & mask;

        // What has been asked for and not built yet, which the supervisor builds here.
        let supervisor = self.image.supervising_now();
        let building = supervisor.wanted != supervisor.running;
        if building {
            self.build(supervisor.running);
        }
        // And its last act, before any other job runs: the copy that makes a scene written by one of
        // those jobs a change that has been asked for and not acted on. Every one-frame window orb
        // watches for is that gap — see `Th06::run_chosen`.
        self.image.supervising(Supervising {
            running: self.image.scene(),
            wanted: self.image.scene(),
        });
        // The jobs the supervisor has registered — but not the ones it registered on this very frame:
        // what it has just built has not been updated yet, which is what `resume::stage_begun` rests
        // on and what makes that moment the one place a stage's numbers can be written over.
        if building {
            return;
        }

        match self.image.scene() {
            Scene::FrontEnd => self.front_end(&pressed),
            Scene::Playing => self.stage(word),
            Scene::Result => self.result(),
            Scene::Ranking => self.ranking(&pressed),
            Scene::Other(_) => {}
        }
    }

    /// What the supervisor does with a scene that has been asked for: takes down what was running and
    /// builds the one wanted.
    fn build(&self, scene: Scene) {
        match scene {
            // `MainMenu::RegisterChain`, which starts the screen it comes up on from nothing.
            Scene::FrontEnd => {
                let front = self.image.front_end_now();
                self.image.front_end(FrontEnd { frames: 0, ..front });
                // Its read of the score file for what the front end offers — which stages, whether
                // there is an Extra — the one read of it whose answer is the game's own file whichever
                // mode orb is in.
                orb::unlocks_read(self.image.front_end_object() as *mut c_void);
            }
            // `GameManager::AddedCallback`, which is where a stage's numbers are put in place and
            // inside which the stage itself is built.
            Scene::Playing => {
                orb::stage_begun(self.image.game_manager_object() as *mut c_void);
            }
            // The result screen registering itself, whose frame timer starts at nothing.
            Scene::Result => {
                let front = self.image.front_end_now();
                self.image.front_end(FrontEnd { frames: 0, ..front });
            }
            // `Ranking::AddedCallback`, whose read of the score file fills the record of captures.
            Scene::Ranking => {
                unsafe { orb::ranking_read(self.image.ranking_screen() as *mut c_void) };
            }
            Scene::Other(_) => {}
        }
    }

    /// The front end, on whichever of its screens it is.
    ///
    /// Only the two orb has a question over. The difficulty and the character select are between them
    /// in the real game and orb asks nothing at either, so choosing `Game Start` here arrives at the
    /// shot type select with both already answered — which is what `chose` writes.
    fn front_end(&self, pressed: &impl Fn(u16) -> bool) {
        // Its cursor, which every one of its screens has and only the title menu's is walked here:
        // `image::item` names the two items orb has a question about, and the ranking is one of them.
        let stepped = |cursor: i32| {
            let moved = cursor - i32::from(pressed(button::UP)) + i32::from(pressed(button::DOWN));
            moved.clamp(0, TITLE_ITEMS - 1)
        };
        // Its screens draw from the generator too, which is why the seed a stage is built with is
        // never the same twice — and so why a run played again from the beginning is a different run,
        // and why a resumed one has to be given the seed that was written down.
        let moving = self.image.reproducing_now();
        self.image.reproducing(Reproducing {
            seed: drawn_from(moving.seed),
            randoms: moving.randoms + 1,
            ..moving
        });

        let front = self.image.front_end_now();
        let decide = pressed(Th06.menu_decide()) && front.acts_on_a_press();
        let (front, asked) = match front.screen {
            // The three items that start a run go through the difficulty and character selects; the
            // ranking is a state of the front end's own, which is also what orb asks for on the way
            // out of a run — see `Th06::show_ranking`.
            Screen::Title if decide && front.cursor == item::SCORE => (
                FrontEnd {
                    screen: Screen::Ranking,
                    frames: 0,
                    ..front
                },
                None,
            ),
            Screen::Title if decide => {
                self.image.chose(&self.run);
                (
                    FrontEnd {
                        screen: Screen::ShotType,
                        cursor: self.run.shot_type,
                        frames: 0,
                    },
                    None,
                )
            }
            Screen::Title => (
                FrontEnd {
                    cursor: stepped(front.cursor),
                    frames: front.frames + 1,
                    ..front
                },
                None,
            ),
            // What the item that starts a run writes: the shot under the cursor, and the scene it
            // wants. The supervisor has already made its copy this frame, so this update ends with
            // the two disagreeing and nothing of the run built.
            Screen::ShotType if decide => (front, Some(Scene::Playing)),
            // And back where it came from, which for the real screen is the character select and for
            // this game is the title menu: the two screens between them are ones orb asks nothing at.
            Screen::ShotType if pressed(Th06.menu_cancel()) => (
                FrontEnd {
                    screen: Screen::Title,
                    cursor: item::GAME_START,
                    frames: 0,
                },
                None,
            ),
            // The ranking asked for, which orb does on the way out of a run so that what the run
            // counted is written. Its own scene, which the front end asks for the same way.
            Screen::Ranking => (front, Some(Scene::Ranking)),
            _ => (
                FrontEnd {
                    frames: front.frames + 1,
                    ..front
                },
                None,
            ),
        };
        self.image.front_end(front);
        if let Some(scene) = asked {
            self.image.supervising(Supervising {
                running: scene,
                wanted: Scene::FrontEnd,
            });
        }
    }

    /// One frame of a stage: its clocks, its generator, its player, and whatever its script has
    /// arranged for that frame.
    fn stage(&self, word: u16) {
        let mut run = self.image.playing_now();
        let mut moving = self.image.reproducing_now();

        run.frames += 1;
        // The clock the enemy script runs on, which advances with the stage however the player is
        // doing — which is why the midstage table is written in it.
        run.script_frames += 1;
        run.enemies = waves(run.script_frames);
        // One number a frame out of the generator, which is the whole of why a stage played again
        // from a different seed is a different stage.
        moving.seed = drawn_from(moving.seed);
        moving.randoms += 1;
        moving.score += u32::from(moving.seed & 0xf);
        moving.player = moved(moving.player, word);

        // The frames after a death, which the real game spends on the animation. Here it is one
        // frame: orb freezes the game on the frame the death is noticed, so nothing but a retry or a
        // run given up ever follows one — and a retry puts the player back with the rest of `.data`.
        self.image.player(Player::Normal);
        if self.hit.take() {
            run.deaths += 1;
            run.lives -= 1;
            self.image.player(Player::Dying);
        }

        // The fight, and the card it puts up. Counted where the card *starts*, which is where the
        // game counts one and the only place it can: a chapter that begins inside a card never starts
        // it, which is why orb counts the retries itself.
        match run.frames {
            BOSS_ARRIVES => self.image.boss(Some(Boss {
                life: 1500,
                attack_frames: 0,
            })),
            CARD_STARTS => {
                self.image.boss(Some(Boss {
                    life: 1200,
                    attack_frames: 0,
                }));
                self.image.card(Some(CARD));
                self.image
                    .card_record(CARD, self.image.card_attempts(CARD) + 1);
            }
            // The card over and the fight going on: the clock back to nothing with no card up, which
            // is a nonspell.
            ATTACK_CHANGES => {
                self.image.boss(Some(Boss {
                    life: 900,
                    attack_frames: 0,
                }));
                self.image.card(None);
            }
            // The attack's own clock, which the two frames above put back to nothing: a reset is what
            // says the fight has moved on, so it must not be reset by anything else.
            _ => self.advance_the_attack(),
        }

        self.image.playing(run);
        self.image.reproducing(moving);
        // Out of lives is the run over, which every game ends at the same screen: the one that shows
        // what the run came to. What says a run *finished* rather than being left is that it went
        // through this — see `Game::run_finished` — so a game over is not the same thing as the retry
        // menu's third item, however alike they look from the title screen.
        if run.lives < 0 {
            self.image.supervising(Supervising {
                running: Scene::Result,
                wanted: Scene::Playing,
            });
        }
    }

    /// The result screen, which walks itself out and leaves for the title.
    ///
    /// `RESULT_FRAMES` rather than at once because the real one is a screen somebody reads, and orb's
    /// trip through the ranking has to fit inside the frames one takes: what that budget is measured
    /// against is `COMMIT_FRAME_LIMIT`.
    fn result(&self) {
        let front = self.image.front_end_now();
        self.image.front_end(FrontEnd {
            frames: front.frames + 1,
            ..front
        });
        if front.frames >= RESULT_FRAMES {
            self.image.supervising(Supervising {
                running: Scene::FrontEnd,
                wanted: Scene::Result,
            });
        }
    }

    fn advance_the_attack(&self) {
        let boss = self.image.boss_now();
        if let Some(boss) = boss {
            self.image.boss(Some(Boss {
                attack_frames: boss.attack_frames + 1,
                ..boss
            }));
        }
    }

    /// The ranking screen: it leaves when it is told to, or when somebody presses back, and putting
    /// the scene and the front end's own state back is its own doing. Going down is what writes the
    /// file.
    fn ranking(&self, pressed: &impl Fn(u16) -> bool) {
        if !self.image.ranking_screen_leaving() && !pressed(Th06.menu_cancel()) {
            return;
        }
        // Going down is what writes the file, which is the whole reason orb walks a run through this
        // screen: what the run counted is in memory and nowhere else until this happens.
        *self.file.borrow_mut() = unsafe { Th06.captures() };
        let front = self.image.front_end_now();
        self.image.front_end(FrontEnd {
            screen: Screen::Title,
            frames: 0,
            ..front
        });
        self.image.supervising(Supervising {
            running: Scene::FrontEnd,
            wanted: Scene::Ranking,
        });
    }

    /// What the game draws of its own.
    ///
    /// Nothing of a stage: a laid-out game has no sprites and a scenario reads a run out of its memory
    /// rather than off the screen. The ranking is the one screen worth drawing, because what it shows
    /// is the record orb has been writing into — one row per spell card the game holds a record of,
    /// with the attempts against it, which is the number 完全無欠 counts.
    fn draw(&self) {
        if self.image.scene() != Scene::Ranking {
            return;
        }
        for (row, (card, attempts)) in self.image.card_records().into_iter().enumerate() {
            let y = RANKING_TOP + row as f32 * RANKING_LINE;
            self.screen
                .writes(&format!("CARD {card}"), RANKING_LEFT, y, INK);
            self.screen
                .writes(&attempts.to_string(), RANKING_ATTEMPTS, y, INK);
        }
    }

    /// `GameManager::AddedCallback`: the stage's numbers in place, and the stage built out of them.
    ///
    /// Its read of the score file's record of spell cards is here too, where the real one parses
    /// `catk` — which is why orb holds that record across a run played back into place: playing a
    /// stage in again starts every card the run had passed, and this read is what a landing would
    /// otherwise be left with. See `resume::hold_captures`.
    fn stage_numbers_in_place(&self) {
        unsafe { Th06.set_captures(&self.file.borrow()) };
        let stage = self.image.stage_built();
        let previous = self.image.playing_now();
        let moving = self.image.reproducing_now();
        self.image.playing(Playing {
            stage,
            difficulty: self.run.difficulty,
            frames: 0,
            script_frames: 0,
            // The generator's own seed, copied where the callback copies it: this is what a stage
            // written down is read back from, and it stays put once the stage draws from it.
            seed: moving.seed,
            // The run's, not the stage's: a run carries its deaths from stage to stage and only the
            // result screen reads them.
            deaths: previous.deaths,
            lives: FRESH.0,
            bombs: FRESH.1,
            power: FRESH.2,
            enemies: 0,
        });
        // Nothing of the last stage's fight is left standing, and the player is where a stage starts
        // them: the field's own middle, which is what a resume has to arrive back at.
        self.image.boss(None);
        self.image.card(None);
        self.image.player(Player::Normal);
        let field = Th06.play_area();
        self.image.play_field(field.top, field.height);
        self.image.reproducing(Reproducing {
            // The count of numbers drawn zeroed as the callback zeroes it, before the stage's own
            // build draws the two that follow.
            randoms: 0,
            player: (field.center_x(), field.top + field.height * 0.75),
            ..moving
        });
        // `Stage::RegisterChain`, called from inside this callback: the one moment a resumed run's
        // seed can go in, since building the stage is what draws from it.
        orb::stage_building(stage);
    }

    /// `Stage::RegisterChain`: the stage built, which draws from the generator as it goes.
    fn stage_built(&self) {
        let moving = self.image.reproducing_now();
        self.image.reproducing(Reproducing {
            seed: drawn_from(drawn_from(moving.seed)),
            randoms: moving.randoms + 2,
            ..moving
        });
    }

    /// The keys the game sees, as its own read hands them back.
    ///
    /// Whatever the host says is down, with no question about which window is in front: orb's hook
    /// over this read is what answers that, and it only calls through when the game's window is the
    /// one in front.
    fn read_the_keyboard(&self) -> u16 {
        let state = orb_api::keyboard::state().unwrap_or([0; 256]);
        MAP.iter()
            .filter(|(key, _)| state[*key as usize] & 0x80 != 0)
            .fold(0, |word, (_, button)| word | button)
    }
}

impl Drop for Fake {
    /// The game closing. The runtime goes first — orb's overlay is released through the device, which
    /// is still here — and then the fields, in the order they are declared: the screen, and last the
    /// installation everything was read through.
    fn drop(&mut self) {
        unsafe { orb::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The device's own three functions, which is all of a controller the game's read calls through.
///
/// Real functions rather than anything in the laid-out memory, because code cannot be laid out: the
/// vtable in that memory holds their addresses, the same way the game's own memory holds the address
/// of the Direct3D device orb draws through.
unsafe extern "system" fn poll(_device: usize) -> i32 {
    0
}

unsafe extern "system" fn acquire(_device: usize) -> i32 {
    0
}

unsafe extern "system" fn read_state(_device: usize, size: u32, state: *mut u8) -> i32 {
    // Said rather than filled: a size that is not this device's format is the game asking for
    // something else, and writing its own idea of one into a buffer of another size is how a test
    // scribbles on a caller's stack.
    assert_eq!(
        size as usize,
        orb_core::game::th06::image::JOY_STATE_BYTES,
        "the game asked its controller for a state of another size",
    );
    unsafe { joy_state(state, running().pushed.get()) };
    0
}

/// Which of this game's buttons is which. Its own numbers, in the order the game's options screen
/// lists them, and a scenario names them through [`MAPPING`] rather than by number.
pub const MAPPING: Mapping = Mapping {
    shoot: 0,
    bomb: 1,
    menu: 2,
    up: 3,
    down: 4,
    // A quarter of the ±1000 the game gives an axis, which is far enough that the middle is not it.
    y_axis: 250,
};

/// The one game orb is attached to here, which has to outlive the runtime that holds it.
static TH06: Th06 = Th06;

extern "fastcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.asked.borrow_mut().push(UPDATE);
    fake.update();
    fake.answers.get()
}

/// `GameWindow::Render`: the game's own whole frame, in 紅魔郷's own draw-then-update order.
///
/// What orb's frame loop replaced — doing the update first is the frame of input lag removed — and what
/// that loop hands a frame back to on each of the three ways out of it that return: no runtime, no
/// device, and a chain target that is null.
///
/// The chain's two exits become the frame's own two here as they do in orb's loop, since that mapping is
/// 紅魔郷's and not orb's: a walk that answered nothing is the game asking to stop, and the frame above it
/// says so to the loop above that.
///
/// No wait in it. The real one paces itself and this one is called a frame at a time by whatever is
/// driving the game, so there is nothing here for a scenario to be held up by.
extern "fastcall" fn own_render(_window: *mut c_void) -> i32 {
    unsafe { orb::run_draw_chain(Th06.chain()) };
    let walked = unsafe { orb::run_calc_chain(Th06.chain()) };
    unsafe { play_sounds(0) };
    if walked == CHAIN_LEFT {
        return FRAME_LEFT;
    }
    if walked == CHAIN_FAILED {
        return FRAME_FAILED;
    }
    unsafe { present(0) };
    FRAME_KEPT_RUNNING
}

/// `SoundPlayer::PlaySounds`: nothing, a laid-out game having no sound system.
///
/// Here rather than left out because the frame loop calls it where the game's own loop did, and a frame
/// that skipped it would be one span of the pacing's breakdown short.
unsafe extern "fastcall" fn play_sounds(_player: usize) {
    running().asked.borrow_mut().push(SOUND);
}

/// `GameWindow::Present`: the frame handed over, which from here is the compositor's.
///
/// Where a scenario counts a frame — the tick it was handed over at, which is what a rate is read off —
/// and where the host is told, since the next flush is what waits for this frame to be composed.
///
/// The tick is peeked rather than read: a scenario writing down when something happened should not be
/// what moves the clock on, and every other read of this counter in a frame is orb's own.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.asked.borrow_mut().push(PRESENT);
    fake.handovers.borrow_mut().push(fake.sim().clock().peek());
    fake.sim().presented();
}

extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.asked.borrow_mut().push(DRAW);
    fake.draw();
    CHAIN_CARRIED_ON
}

extern "system" fn input() -> u16 {
    running().read_the_keyboard()
}

extern "C" fn stage_building(_stage: i32) -> i32 {
    running().stage_built();
    0
}

extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    running().stage_numbers_in_place();
    // Nothing, which is the callback saying it built the stage it was asked for: orb reads anything
    // else as a stage the game could not build.
    0
}

extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    0
}

/// `Ranking::AddedCallback`: the screen built, with the records it shows read out of the score file.
///
/// orb gets in front of this to empty what is in memory first, so that a ranking read defines the
/// history rather than adding to it — see `Game::forget_captures`.
extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    let fake = running();
    unsafe { Th06.set_captures(&fake.file.borrow()) };
    fake.image.ranking_screen_shown();
    0
}

/// The part of the status panel the game counts the lives in, as a rectangle a drawn quad can be held
/// against: what a pointdevice run paints a brush stroke over and a legacy run is left holding.
pub fn lives_row() -> Quad {
    let row = Th06.lives_row();
    Quad {
        x: row.left,
        y: row.top,
        width: row.width,
        height: row.height,
        color: 0,
        texture: 0,
    }
}

/// A stage's waves, in the only terms the boundary detector reads them: two hundred frames with
/// enemies on and two hundred without.
pub fn waves(script: i32) -> i32 {
    if (script / 200) % 2 == 0 { 3 } else { 0 }
}

/// The next number out of the generator.
///
/// A generator of this game's own, not 紅魔郷's: what a scenario is about is that a stage seeded the
/// same way draws the same numbers, which any generator answers and which is what a resume rests on.
fn drawn_from(seed: u16) -> u16 {
    seed.wrapping_mul(0x9d5d).wrapping_add(0x6f7f)
}

/// Where the buttons of a frame leave the player, held inside the play field.
fn moved((x, y): (f32, f32), word: u16) -> (f32, f32) {
    let field = Th06.play_area();
    let moved = |at: f32, less: bool, more: bool, from: f32, span: f32| {
        let at = at - if less { SPEED } else { 0.0 } + if more { SPEED } else { 0.0 };
        at.clamp(from, from + span)
    };
    (
        moved(
            x,
            word & button::LEFT != 0,
            word & button::RIGHT != 0,
            field.left,
            field.width,
        ),
        moved(
            y,
            word & button::UP != 0,
            word & button::DOWN != 0,
            field.top,
            field.height,
        ),
    )
}

/// A directory standing in for the one the game is installed in, empty of everything a run left
/// behind and holding the one file orb reads out of it.
fn scratch(name: &str) -> PathBuf {
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
