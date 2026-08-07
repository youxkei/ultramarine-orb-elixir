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
//! **Its own loop calls orb's frame loop, as the real game's does.** The `Present` and the sound that
//! loop makes are addresses the game hands over — see `Game::frame_calls` — so this game hands over two
//! of its own, and its `present` is where a scenario counts a frame handed over. A launch started
//! `--no-frame-loop` is the game's own draw-then-update order instead, with orb's update and draw hooks
//! in the middle of it, which is the other configuration orb ships.
//!
//! **Everything a launch has that is not this game is [`Launch`]** — the display, the device orb draws
//! through, what a frame's own work costs, and the frames themselves. This file is the half a second
//! game would bring one of its own of.
//!
//! Where a number here is 紅魔郷's it says so. Where it is this game's own — how far the player moves,
//! when its boss arrives — it says that too: what a scenario is about is that the same buttons from
//! the same seed arrive at the same place, not how fast Reimu is.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{CStr, c_void};

use orb::recording::Quad;
use orb_config::Config;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{
    Boss, FrontEnd, Image, Mapping, Player, Playing, Pushed, Reproducing, Scene, Screen,
    Supervising, item, joy_state, result_state,
};
use orb_core::game::{Game, RunStart};
use orb_sim::keys;

use super::{
    DRAW, Display, Launch, Launched, PRESENT, Panel, SOUND, UPDATE, WINDOW, Work, scratch,
};

/// What the game's own chain walk answers while the game is running.
///
/// Neither of the two below, which orb reads as the game leaving. A real run whose resume played 7476
/// updates in inside one frame is measured proof that the real walk does not answer zero
/// while a stage is running, since orb's playback stops on that — and
/// `scenario_pointdevice_run.rs` plays 700 in one frame the same way.
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

/// The class 紅魔郷 registers and creates its window with, which is what orb's rewrite matches on: a
/// window of any other class is one orb leaves the size the game asked for.
const WINDOW_CLASS: &CStr = c"BASE";

/// The score file 紅魔郷 asks for, which is the only name this game ever passes `CreateFileA`.
///
/// Which file the open *lands* in is orb's answer and not this one's — the whole subject of
/// `scenario_the_score_file.rs` — so a game that named the forked file itself would be answering the
/// question being asked.
const SCORE_FILE: &CStr = c"score.dat";

/// `GENERIC_WRITE`, which is the one bit of `CreateFileA`'s access this game varies: the score file is read
/// in three places and written in one, and orb reads that bit to tell a refused write from a read.
const GENERIC_WRITE: u32 = 0x4000_0000;

/// What `CreateFileA` answers where it could not open the file — `INVALID_HANDLE_VALUE`, which is what
/// orb's own refusal of a write answers with and what this game reads as a first launch.
const NO_HANDLE: isize = -1;

/// One open of the score file: the name it landed in, and whether it was for writing.
///
/// Which is the whole of what crosses `CreateFileA` and the whole of what a scenario about that file reads
/// back. There is no file on any disk — see [`Fake::opens`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Open {
    pub path: String,
    pub write: bool,
}

/// What the game asks `CreateWindowExA` for, and every one of these is replaced.
///
/// This game's own numbers rather than 紅魔郷's, and deliberately nothing like the answer: what a
/// scenario asserts is the window that came out, so an ask that already resembled it would be an ask
/// that could not tell a rewrite from a pass-through. The style is a caption and a system menu, which
/// is a window with a frame — the one thing about the ask that is real, since it is what the game
/// would have got.
const ASKED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x1000_0000;
const ASKED_AT: (i32, i32) = (17, 23);
const ASKED_SIZE: (i32, i32) = (646, 505);

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

/// How many stages a run goes through, which is 紅魔郷's own six.
pub const STAGES: i32 = 6;

/// How long the title menu sits with nothing pressed at it before it falls into its attract demo, in
/// frames.
///
/// This game's own number, and far past the frames the title's own opening animation ignores a press for —
/// `MENU_TITLE_GRACE_FRAMES` — so the two moments a press can be spent on nothing are two moments and not
/// one. What a scenario is about is that each of them costs a press, not how patient 紅魔郷 is.
pub const DEMO_AFTER: i32 = 90;

/// How long the game lays its panel over the start of a stage, which is 紅魔郷's own: the vm's script
/// sets all five of `GuiFlags`' two-bit fields to 2 itself over these frames — 0x41a2b6 — and stops
/// where the script reaches `ExitHide`. Over them, what orb writes into the lowest pair decides
/// nothing.
pub const PANEL_FRAMES: u32 = 250;

/// How long the ending stands before the result screen follows it.
///
/// This game's own, and a placeholder for what an ending is: the script that runs out inside one frame
/// and the staff roll after it are `scenario_the_ending.rs`'s, and until they are here this is the scene
/// a cleared run passes through on the way to its result.
const ENDING_FRAMES: i32 = 10;

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
///
/// Not at once, because the real one is a screen somebody reads — **9.5 seconds** of it measured between
/// the two scene lines of a cleared run — and orb's trip through the ranking has to fit inside the frames
/// one takes: what that budget is measured against is `COMMIT_FRAME_LIMIT`. Past
/// [`REPLAY_QUESTION_DRAWN_AT`], so that a screen left to itself really would draw the question orb writes
/// past.
const RESULT_FRAMES: i32 = 120;

/// And how many frames into that screen the question about saving a replay begins to be drawn, which is
/// 紅魔郷's own **60**: orb writes the state past the question before the frame timer reaches that, so no
/// part of that screen is ever drawn. See `Th06::skip_replay_prompt`.
const REPLAY_QUESTION_DRAWN_AT: i32 = 60;

/// How far the player moves in a frame, and the field it is held inside — the field being 紅魔郷's own
/// play area and the speed this game's.
const SPEED: f32 = 4.0;

/// The run a launch is started for: Normal, Reimu A, from stage one.
///
/// What a scenario that is not about a run declares, a launch having to be started for one: the game
/// sits on its title menu and none of this is played.
pub fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// A 紅魔郷 laid out, with orb attached to it.
///
/// Its own memory is the `Image`; the half of a launch that is any game's is [`Launch`], and what is
/// left here is what a process has and an address space does not — which run its front end is
/// offering, and the file it keeps its scores in.
pub struct Fake {
    image: Image,
    launch: Launch,
    run: RunStart,
    /// The score files this game keeps, by the name each open landed in, as the bytes of the record they
    /// hold about spell cards — which is what `Game::captures` and `Game::set_captures` are the two
    /// halves of.
    ///
    /// A map rather than files on disk: what a scenario reads is which name an open landed in and the
    /// number behind it, and both of those cross `CreateFileA` — the path and the access. Outside the
    /// game's memory, so a chapter restored does not rewind them, which is true of real files too.
    ///
    /// A name with no entry is a file that is not there, and its open fails: which is what orb's own file
    /// is before a ranking has ever written it.
    files: RefCell<HashMap<String, Vec<u8>>>,
    /// Every open of the score file, in the order they happened — see [`Open`].
    opens: RefCell<Vec<Open>>,
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
    /// And a bullet sitting on the player from here on, which is a hit every frame rather than one.
    ///
    /// Apart from `hit` because what it is for is the other question: `hit` is a scenario saying the
    /// player died, and this is a scenario saying nothing is stopping them dying — so what happens is
    /// the hit test's answer and not the scenario's. See [`puts_a_bullet_on_the_player`](Fake::puts_a_bullet_on_the_player).
    bullet: Cell<bool>,
    /// How long each of this game's stages runs before the next begins. `None` is a stage that never
    /// ends — see [`stages_last`](Fake::stages_last).
    stage_frames: Cell<Option<u32>>,
    /// Set by a scenario for the run to be given up at the game's own pause — see
    /// [`gives_the_run_up_at_its_own_pause`](Fake::gives_the_run_up_at_its_own_pause).
    given_up: Cell<bool>,
    /// Whether the title screen falls into its attract demo when nothing is pressed at it — see
    /// [`demos_when_idle`](Fake::demos_when_idle).
    demos: Cell<bool>,
    /// Whether the result screen ever reached the frame its question about saving a replay is drawn
    /// from — see [`the_replay_question_was_drawn`](Fake::the_replay_question_was_drawn).
    ///
    /// Beside the memory rather than in it, because it is not a fact about the run: it is one about what
    /// reached the screen, which no chapter restored underneath it takes back.
    replay_question_drawn: Cell<bool>,
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

/// The game this thread is running.
fn running() -> &'static Fake {
    let fake = RUNNING.get();
    assert!(!fake.is_null(), "no game has been attached on this thread");
    unsafe { &*fake }
}

impl Launched for Fake {
    fn frame_window(&self) -> *mut c_void {
        self.image.game_window_object() as *mut c_void
    }

    fn own_render(&self) -> i32 {
        own_render(self.frame_window())
    }

    fn sim(&self) -> &orb_sim::Sim {
        self.image.sim()
    }

    fn launch(&self) -> &Launch {
        &self.launch
    }
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
        Self::attach_declaring(display, None, name, run, settings)
    }

    /// And on a monitor a scenario says the whole of too, for the ones about the window orb makes.
    ///
    /// The panel goes in before orb is attached, because attaching is when orb says the process reads
    /// real pixels — a monitor written down after that would be one nothing had asked the truth of.
    pub fn attach_to_a_panel(
        panel: Panel,
        name: &str,
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(Display::ordinary(), Some(panel), name, run, settings)
    }

    fn attach_declaring(
        display: Display,
        panel: Option<Panel>,
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
        let launch = Launch::new(dir, display.seed, config.own_frame_loop);
        let installed = image.enter();
        image.shows_through(launch.device(), WINDOW);
        // Where the game is installed, which is where orb writes its log: beside the exe, because that
        // is where `orb.yaml` and the launcher are.
        image.sim().set_host_exe(launch.dir().join("th06.exe"));
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
        // And the monitor the window goes on, for a scenario about the window: in front of orb being
        // attached for the same reason the rate is, since attaching is where orb says this process reads
        // sizes as real pixels and a panel written down afterwards would never have been asked the
        // scaled question at all. A scenario that declares none has no monitor, which is the launch orb
        // leaves the window as the game made it.
        if let Some(panel) = &panel {
            sim.windows().set_monitor(panel.monitor, panel.frame);
            if panel.refuses_dpi_awareness {
                sim.windows().refuse_dpi_awareness();
            }
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
        // And the keyboard device the game takes exclusively, which is what its own read goes through until
        // orb lets it go: a launch that has not asked for `--sent-keys` keeps it for its whole life, which
        // is every other scenario here and is why the keys they press are pressed rather than sent.
        image.keyboard_device(
            keyboard_acquire as *const () as usize,
            keyboard_unacquire as *const () as usize,
            keyboard_release as *const () as usize,
        );
        // What its score file holds to begin with: a record of the card its boss is on, never tried.
        // Without the record `count_card_attempt` refuses to count, a zeroed one being a card that is
        // not there rather than a card nobody has reached.
        image.card_record(CARD, 0);
        // And the file it is in, under the name the game asks for. Only that one: orb's own is a file that
        // is not there until a ranking screen has written it, which is what makes its first open fail the
        // way a first launch's does.
        let files = RefCell::new(HashMap::from([(
            SCORE_FILE.to_string_lossy().into_owned(),
            unsafe { Th06.captures() },
        )]));
        // Where the game starts: its front end, on the title menu, on the item that starts a run — and not
        // built yet, which is the supervisor's own first frame. What that buys is the front end's own read
        // of the score file: `MainMenu::AddedCallback` is where it happens and being built is what calls it,
        // so a game whose menu was there from nothing would never make the one open whose answer is the
        // game's own file whatever the mode.
        image.supervising(Supervising {
            running: Scene::FrontEnd,
            wanted: Scene::Other(0),
        });
        image.front_end(FrontEnd {
            screen: Screen::Title,
            cursor: item::GAME_START,
            frames: 0,
        });

        let fake = Box::new(Self {
            image,
            launch,
            run,
            files,
            opens: RefCell::new(Vec::new()),
            pushed: Cell::new(Pushed::none()),
            hit: Cell::new(false),
            bullet: Cell::new(false),
            stage_frames: Cell::new(None),
            given_up: Cell::new(false),
            demos: Cell::new(false),
            replay_question_drawn: Cell::new(false),
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
                    create_window,
                    create_file,
                },
            )
        };
        fake
    }

    /// And the same with orb's own account of the pacing being written where a scenario can read it
    /// back, which is every scenario about the frame loop — see `scenario_pacing.rs`.
    ///
    /// `work` is what the game's own frame costs, since that is the whole of what the pacing has to put a
    /// wait around — see [`Work`]. The run is [`the_run`] and the game sits on its title menu throughout:
    /// a stage being played is a load and a draw rather than another question about the cadence, which is
    /// what `work` stands for.
    pub fn attach_watching_the_pacing(display: Display, name: &str, work: Work) -> Box<Self> {
        let game = Self::attach_to_display(display, name, the_run(), |config| {
            config.pacing_log = true;
        });
        game.frame_takes(work);
        game
    }

    /// What this game's chain walk answers from here on: [`CHAIN_CARRIED_ON`] until a scenario says
    /// otherwise, and [`CHAIN_LEFT`] or [`CHAIN_FAILED`] to say the game is going.
    pub fn chain_answers(&self, result: i32) {
        self.answers.set(result);
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

    /// Puts a bullet where the player is and leaves it there, so that every update from here on runs its
    /// hit test against a live bullet.
    ///
    /// Which is what a stage really is, and the difference from [`hit`](Fake::hit) is the whole point of
    /// it: a scenario saying "the player died" cannot show that something *stopped* them dying. The test
    /// runs where the game runs it — after `Player::OnUpdate`, which is chain priority 7 against the
    /// bullets' 11 — so an invulnerability written for this update has to still be there when it runs.
    pub fn puts_a_bullet_on_the_player(&self) {
        self.bullet.set(true);
    }

    /// How long each of this game's stages runs before the next begins.
    ///
    /// Nothing by default, which is a stage that never ends: every scenario about one stage wants that,
    /// and one about a run through six says how long each of them is. A run's last stage hands over to
    /// the ending rather than to a seventh.
    pub fn stages_last(&self, frames: u32) {
        self.stage_frames.set(Some(frames));
    }

    /// Has the title screen fall into its attract demo after [`DEMO_AFTER`] frames with nothing pressed at
    /// it, and leave it on the first press.
    ///
    /// Off by default, and that is not tidiness: a title screen that started a run of its own would change
    /// every scenario that sits on one, and the pacing's sit there for thousands of frames apiece — a demo
    /// under them would be building stages, taking snapshots and costing the frames they are measuring. So
    /// the one scenario about a press being eaten asks for it.
    pub fn demos_when_idle(&self) {
        self.demos.set(true);
    }

    /// `esc` and then やめる: the game's own way out of a run, which is one write.
    ///
    /// A scenario saying so, the way it says the player was hit — the pause menu's two screens are not
    /// here, and `StageMenu::OnUpdateGameMenu` ends them by writing the scene the front end runs in. The
    /// panel, and with it `g_Gui`'s job in the draw chain, stands until the front end is built on the
    /// frame after: that one frame is what this exists to make reachable.
    ///
    /// Not a key, deliberately. The pause button is outside [`MAP`] on purpose — a resume feeding it back
    /// would open a menu instead of playing — and the cancel the front end reads is the bomb key, which
    /// during a stage is a bomb and not a way out.
    ///
    /// Acted on inside the next update of the stage rather than here, because *when* the write happens is
    /// the whole of what the frame after a run is: the pause menu is a job of the stage's own, so the
    /// supervisor's copy for that frame has already been made and the scene it wrote is one that has been
    /// asked for and not acted on. Written between frames instead, the supervisor would take the scene
    /// down in the same update and there would be no such frame at all.
    pub fn gives_the_run_up_at_its_own_pause(&self) {
        self.given_up.set(true);
    }

    /// Every open of the score file this game has made, in the order they happened.
    ///
    /// Which name each landed in is orb's answer — the fork follows the mode chosen inside the game — and
    /// this is where a scenario reads it back. There is no file on any disk: see [`Fake::files`].
    pub fn score_file_opens(&self) -> Vec<Open> {
        self.opens.borrow().clone()
    }

    /// Forgets them, so that what a scenario reads is the opens it means rather than every one since the
    /// game started.
    pub fn forget_score_file_opens(&self) {
        self.opens.borrow_mut().clear();
    }

    /// What the file of that name holds, or `None` for one that is not there — which is orb's own file
    /// until a ranking screen has written it.
    pub fn score_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.borrow().get(path).cloned()
    }

    /// Whether the result screen ever got as far as drawing the question about saving a replay.
    ///
    /// Which is what "no replay is offered" has to mean if it means anything: orb writes that screen's
    /// state past the question rather than answering it, and a state written a frame too late is a
    /// question somebody has to answer.
    pub fn the_replay_question_was_drawn(&self) -> bool {
        self.replay_question_drawn.get()
    }

    /// The game in a 完全無欠モード run with its first chapter taken, which is where every scenario about
    /// a run being played from further on starts.
    ///
    /// The walk somebody makes to get there: the mode question answered over the title menu, the shot
    /// answered at the game's own select, and then the frames a stage spends settling before its first
    /// snapshot — 248 of them, `STAGE_SETTLE_FRAMES` and the whole of `MUSIC_WAIT_FRAMES`, since a
    /// laid-out game has no track for that wait to find. `scenario_pointdevice_run.rs` takes every step
    /// of it and asserts on each; this is the same walk with nothing asserted.
    ///
    /// # Panics
    /// Naming whichever step the run did not get past.
    pub fn in_a_pointdevice_run(&self) {
        let log = self.log();
        self.at_the_title_menu();
        self.press(keys::Z);
        self.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        self.started_at_the_games_own_screens();
        self.frames_until("the stage's first chapter", 400, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
    }

    /// And in a run nobody was asked about, which is what a launch that fixes the mode is: `--clear` and
    /// a pass over a replay take the mode they are given, so the press at the title menu is never held
    /// back and goes straight to the game's own screens.
    ///
    /// # Panics
    /// Naming whichever step the run did not get past.
    pub fn in_a_run_nobody_was_asked_about(&self) {
        self.at_the_title_menu();
        self.press(keys::Z);
        self.started_at_the_games_own_screens();
    }

    /// The shot answered at the game's own select, and the stage built out of it.
    fn started_at_the_games_own_screens(&self) {
        self.frames_until("the shot type select ready to act on a press", 90, || {
            let front = self.image.front_end_now();
            front.screen == Screen::ShotType && front.acts_on_a_press()
        });
        self.press(keys::Z);
        self.frames_until("the stage built", 8, || self.state().playing);
    }

    /// The game creating its window, which is the call orb's rewrite is reached through.
    ///
    /// Once, where the real game calls it once: everything about the window — where it goes, how big it
    /// is and whether it has a frame — is decided inside that one call, which is why there is nothing to
    /// flash on the screen and nothing to resize afterwards. The handle is the host's, and it is the one
    /// orb wrote down as the game's window.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        let window = unsafe {
            orb::window::create_window_ex_a(
                0,
                WINDOW_CLASS.as_ptr().cast(),
                c"th06".as_ptr().cast(),
                ASKED_STYLE,
                ASKED_AT.0,
                ASKED_AT.1,
                ASKED_SIZE.0,
                ASKED_SIZE.1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        orb_api::Hwnd(window as usize)
    }

    /// What the game's memory says the run is, read the way the frame hook reads it: every field
    /// parsed back out of the memory rather than off the addresses this game wrote.
    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th06.read_state() }
    }

    pub fn image(&self) -> &Image {
        &self.image
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
            Scene::Playing => self.stage(word, &pressed),
            Scene::Ending => self.ending(),
            Scene::Result => self.result(),
            Scene::Ranking => self.ranking(&pressed),
            // The one frame a stage transition takes is spent building the stage after, which is the
            // arm above this: the scene is only ever read here on a frame `build` has already returned
            // early from.
            Scene::Rebuilding | Scene::Other(_) => {}
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
                // Whatever run was on screen is over, the attract demo among them: the flag that tells one
                // apart from a played run belongs to the run and not to the front end.
                self.image.demo_mode(false);
                // The gameplay scene's own draw jobs go with it, `g_Gui`'s among them — which is what
                // takes the panel off the screen, and so the last frame the mark can be drawn on. Here
                // rather than on the frame the run ended, because that is where it is in the game: the
                // panel stays up until the front end has something of its own to draw.
                self.image.cuts_gui_from_the_draw_chain();
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
            // `GameManager::Reinit`: the same callback for the stage after this one, and the whole of
            // what a transition is — one frame, with the next stage built inside it, which is why the
            // scene goes straight back to a stage being played.
            Scene::Rebuilding => {
                orb::stage_begun(self.image.game_manager_object() as *mut c_void);
                self.image.supervising(Supervising {
                    running: Scene::Playing,
                    wanted: Scene::Playing,
                });
            }
            // `Ending::RegisterChain`, whose own frame timer starts at nothing.
            Scene::Ending => {
                let front = self.image.front_end_now();
                self.image.front_end(FrontEnd { frames: 0, ..front });
                self.image.cuts_gui_from_the_draw_chain();
            }
            // `ResultScreen::RegisterChain`: the screen's own job in the chain, and its frame timer at
            // nothing. It comes up on the question about saving a replay, which is what the screen does
            // with a run that finished — and the state orb writes over for a run that has chapters.
            Scene::Result => {
                let front = self.image.front_end_now();
                self.image.front_end(FrontEnd { frames: 0, ..front });
                self.image
                    .registers_the_result_screen(result_state::SAVE_REPLAY_QUESTION);
                self.image.cuts_gui_from_the_draw_chain();
            }
            // `Ranking::AddedCallback`, whose read of the score file fills the record of captures.
            Scene::Ranking => {
                unsafe { orb::ranking_read(self.image.ranking_screen() as *mut c_void) };
                self.image.cuts_gui_from_the_draw_chain();
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
            // Nothing pressed at it for long enough, and the title screen starts its attract demo: a run in
            // every respect but the flag that tells one apart, which is what makes it eat the press that
            // ends it — the press goes on leaving the demo and never reaches the menu underneath.
            Screen::Title if self.demos.get() && front.frames >= DEMO_AFTER => {
                self.image.chose(&self.run);
                self.image.demo_mode(true);
                (FrontEnd { frames: 0, ..front }, Some(Scene::Playing))
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
    ///
    /// **In the order the chain runs the jobs**, because one of the claims is about that order and
    /// nothing else can carry it: `Player::OnUpdate` is priority 7 and the bullets are checked at 11, so
    /// an invulnerability written for this update has to survive the player's own update to be there
    /// when the hit test runs. See [`update_the_player`](Fake::update_the_player).
    fn stage(&self, word: u16, pressed: &impl Fn(u16) -> bool) {
        // The attract demo, which any press leaves — and the press is spent on leaving it, never reaching
        // the menu underneath. Before anything else this update does, since a demo somebody has pressed a
        // key at is not a stage that goes on being played.
        if self.state().demo && pressed(Th06.menu_decide() | Th06.menu_cancel()) {
            self.image.supervising(Supervising {
                running: Scene::FrontEnd,
                wanted: Scene::Playing,
            });
            return;
        }
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

        // The panel laid over the stage's first frames, which writes every one of `GuiFlags`' five
        // fields itself — so over these the pair orb writes is not the pair the game draws from.
        if run.frames <= PANEL_FRAMES {
            self.image.repaints_the_whole_panel();
        }

        // `Player::OnUpdate`, at chain priority 7.
        self.update_the_player();
        // And the bullets, at 11: the hit test, which is the one thing an invulnerability written for
        // this update has to still be true at.
        let killable = self.image.player_now() == Player::Normal;
        if (self.hit.take() || self.bullet.get()) && killable {
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
            return;
        }
        // `esc` and then やめる, as a job of the stage's own: `StageMenu::OnUpdateGameMenu` writes the
        // scene the front end runs in, and the supervisor has already made its copy for this frame — so
        // the run is over on this one with the panel, and `g_Gui`'s job in the draw chain, still standing.
        if self.given_up.take() {
            self.image.supervising(Supervising {
                running: Scene::FrontEnd,
                wanted: Scene::Playing,
            });
            return;
        }
        // The stage over, where a scenario said how long one is. A transition goes through the scene the
        // game rebuilds its manager in and the last stage hands over to the ending instead — never to a
        // seventh stage.
        if Some(run.frames) == self.stage_frames.get() {
            let next = if run.stage + 1 < STAGES {
                Scene::Rebuilding
            } else {
                Scene::Ending
            };
            self.image.supervising(Supervising {
                running: next,
                wanted: Scene::Playing,
            });
        }
    }

    /// `Player::OnUpdate`, at chain priority 7: the player's state moved on one frame.
    ///
    /// The invulnerable state is the one that expires, and it expires *here* — at the end of the same
    /// update, 0x428... onwards — which is why `make_invulnerable` writes the frames left and not only
    /// the state: a state written with the frames the last respawn left under it is a player who is
    /// killable again by the time the bullets are checked at priority 11.
    fn update_the_player(&self) {
        match self.image.player_now() {
            Player::Invulnerable => {
                let left = self.image.invulnerable_frames() - 1;
                self.image.set_invulnerable_frames(left.max(0));
                if left <= 0 {
                    self.image.player(Player::Normal);
                }
            }
            // The frames after a death, which the real game spends on the animation. Here it is one
            // frame: orb freezes the game on the frame the death is noticed, so nothing but a retry or a
            // run given up ever follows one — and a retry puts the player back with the rest of `.data`.
            Player::Dying | Player::Spawning => self.image.player(Player::Normal),
            Player::Normal => {}
        }
    }

    /// The ending, which walks itself out and hands over to the result screen.
    ///
    /// What an ending *is* — the script that runs out inside one frame and the staff roll after it — is
    /// `scenario_the_ending.rs`'s and is not here yet. What is here is the scene, so that a cleared run
    /// goes the way a cleared run goes: six stages, the ending, and then the result.
    fn ending(&self) {
        let front = self.image.front_end_now();
        self.image.front_end(FrontEnd {
            frames: front.frames + 1,
            ..front
        });
        if front.frames >= ENDING_FRAMES {
            self.image.supervising(Supervising {
                running: Scene::Result,
                wanted: Scene::Ending,
            });
        }
    }

    /// The result screen, which walks itself out and leaves for the title.
    ///
    /// `RESULT_FRAMES` rather than at once because the real one is a screen somebody reads, and orb's
    /// trip through the ranking has to fit inside the frames one takes: what that budget is measured
    /// against is `COMMIT_FRAME_LIMIT`.
    ///
    /// **The screen a replay is saved from is a state of this one and not a scene of its own.** So what
    /// "no replay is offered" is, is this screen's state having been written past the question — orb does
    /// that rather than answering the question, because answering it means playing out a fade — and what
    /// says the question was never put to anybody is that no frame of it was ever drawn.
    fn result(&self) {
        let front = self.image.front_end_now();
        self.image.front_end(FrontEnd {
            frames: front.frames + 1,
            ..front
        });
        // The question about saving a replay, drawn from the frame its own animation starts on. Where the
        // state has been written past it there is nothing to draw, which is the whole of what orb writing
        // that state buys.
        if self.image.result_screen_state() == result_state::SAVE_REPLAY_QUESTION
            && front.frames >= REPLAY_QUESTION_DRAWN_AT
        {
            self.replay_question_drawn.set(true);
        }
        if front.frames >= RESULT_FRAMES {
            self.image.cuts_the_result_screen();
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
        // screen: what the run counted is in memory and nowhere else until this happens. Written whether a
        // score was entered into the ranking or the ranking was only looked at, as the deleted callback
        // writes it — 0x42f5cd, the one caller of the write in the whole exe.
        self.writes_the_score_file();
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
    ///
    /// One thing of the panel, and it is not drawing: `Gui::OnDraw` takes one off each of `GuiFlags`'
    /// five two-bit fields at the end of the draw it repainted that row in — 0x41acdb. Which is what
    /// makes a field decide anything at all: a field nothing writes again is a row the game stops
    /// repainting two draws later, so whether orb's write is what is keeping the lives' row painted is
    /// answerable rather than assumed.
    fn draw(&self) {
        if self.image.gui_in_the_draw_chain() {
            self.spends_the_panels_flags();
        }
        if self.image.scene() != Scene::Ranking {
            return;
        }
        for (row, (card, attempts)) in self.image.card_records().into_iter().enumerate() {
            let y = RANKING_TOP + row as f32 * RANKING_LINE;
            self.launch
                .writes(&format!("CARD {card}"), RANKING_LEFT, y, INK);
            self.launch
                .writes(&attempts.to_string(), RANKING_ATTEMPTS, y, INK);
        }
    }

    /// The game opening its score file, which is one `CreateFileA` on the name it asks for.
    ///
    /// Answers the name the open landed in, which is orb's decision and not this game's, or `None` where
    /// the open failed — a file that is not there, or orb refusing a write.
    fn opens_the_score_file(&self, write: bool) -> Option<String> {
        let access = if write { GENERIC_WRITE } else { 0 };
        let handle = unsafe {
            orb::score::create_file_a(
                SCORE_FILE.as_ptr().cast(),
                access,
                0,
                std::ptr::null(),
                0,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle as isize == NO_HANDLE {
            return None;
        }
        // The handle is an index into the opens above, one-based: a file this game keeps rather than one on
        // any disk, so what it is for is finding the name orb chose.
        let opens = self.opens.borrow();
        opens.get(handle as usize - 1).map(|open| open.path.clone())
    }

    /// The game's read of that file, at each of the three places the exe does it.
    ///
    /// A failed open is not a no-op: `clrd`'s parse at 0x42b502 clears its destination before it looks for
    /// the chunk — four records memset at 0x42b535 — so what the game is left with is an empty record and
    /// not the one it had.
    fn reads_the_score_file(&self) {
        let bytes = self
            .opens_the_score_file(false)
            .and_then(|path| self.files.borrow().get(&path).cloned())
            .unwrap_or_default();
        unsafe { Th06.set_captures(&bytes) };
    }

    /// And its write, which has one caller in the whole exe: 0x42f5cd, in the ranking screen's deleted
    /// callback.
    ///
    /// A refused open leaves the file as it was and the game carries on — `WriteDataToFile` checks its open
    /// and its caller drops the answer — which is what makes refusing the write a run that is not written
    /// rather than a game that stops.
    fn writes_the_score_file(&self) {
        let taken = unsafe { Th06.captures() };
        if let Some(path) = self.opens_the_score_file(true) {
            self.files.borrow_mut().insert(path, taken);
        }
    }

    /// One off each of `GuiFlags`' five fields, as `Gui::OnDraw` does at 0x41acdb — every field that is
    /// not already nothing, and none of them below it.
    fn spends_the_panels_flags(&self) {
        let flags = self.image.gui_flags();
        let spent = (0..5).fold(0, |spent: u32, field| {
            let left = (flags >> (field * 2)) & 0b11;
            spent | left.saturating_sub(1) << (field * 2)
        });
        self.image.sets_gui_flags(spent);
    }

    /// `GameManager::AddedCallback`: the stage's numbers in place, and the stage built out of them.
    ///
    /// Its read of the score file's record of spell cards is here too, where the real one parses
    /// `catk` — which is why orb holds that record across a run played back into place: playing a
    /// stage in again starts every card the run had passed, and this read is what a landing would
    /// otherwise be left with. See `resume::hold_captures`.
    fn stage_numbers_in_place(&self) {
        self.reads_the_score_file();
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
        // `Gui::RegisterChain`, which puts `g_Gui`'s own draw job in the chain: the panel is on the
        // screen from here until the scene is taken down, and that job being in the draw list is the
        // whole of what `Th06::draws_lives_row` asks. Registered per stage, as the real one is — a
        // second registration of the same static element is what the element's own links check out
        // against.
        self.image.registers_gui_in_the_draw_chain();
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

    /// The keys the game sees, as its own read hands them back — `Controller::GetInput` and both of its
    /// branches.
    ///
    /// **Which branch is which is the whole subject of `scenario_keys_from_another_program.rs`.** While the
    /// game holds a keyboard device it took `DISCL_EXCLUSIVE | DISCL_FOREGROUND`, that device is what
    /// answers, and such a device does not see a key another program sent — measured. Once orb has let it
    /// go the read is `GetKeyboardState`, the game's own other way, which does see one.
    ///
    /// Either way there is no question about which window is in front: orb's hook over this read is what
    /// answers that, and it only calls through when the game's window is the one in front.
    fn read_the_keyboard(&self) -> u16 {
        let down: Box<dyn Fn(u8) -> bool> = if self.image.holds_a_keyboard_device() {
            let keyboard = self.sim().keyboard();
            Box::new(move |key| keyboard.held(key))
        } else {
            let state = orb_api::keyboard::state().unwrap_or([0; 256]);
            Box::new(move |key| state[key as usize] & 0x80 != 0)
        };
        MAP.iter()
            .filter(|(key, _)| down(*key))
            .fold(0, |word, (_, button)| word | button)
    }
}

impl Drop for Fake {
    /// The game closing. The runtime goes first — orb's overlay is released through the device, which
    /// is still here — and then the fields, in the order they are declared: the game's memory, the
    /// launch's own device, and last the installation everything was read through.
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

/// And the keyboard device's own three, which is all of one orb ever calls: the acquire it makes after the
/// window has been away, and the `Unacquire` and `Release` it makes to let the device go.
///
/// Nothing to do in any of them. What being held *means* is read off the pointer the game keeps —
/// `Image::holds_a_keyboard_device` — because that is what the game's own read branches on and what orb
/// clears: a device object that remembered its own acquired state would be a second answer to the one
/// question.
unsafe extern "system" fn keyboard_acquire(_device: usize) -> i32 {
    0
}

unsafe extern "system" fn keyboard_unacquire(_device: usize) -> i32 {
    0
}

unsafe extern "system" fn keyboard_release(_device: usize) -> i32 {
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
    fake.launch.asked_for(UPDATE);
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

/// The game's own `CreateWindowExA`, which orb's rewrite calls through with the arguments it decided.
///
/// The host makes the window, not this: how thick a frame is and therefore what client a window of that
/// size comes out with belongs to Windows — see `orb_sim::Windows` — and a game that decided its own
/// client would be answering the question orb's arithmetic is being asked.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_window(
    _ex_style: u32,
    _class_name: *const u8,
    _window_name: *const u8,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    _parent: *mut c_void,
    _menu: *mut c_void,
    _instance: *mut c_void,
    _param: *const c_void,
) -> *mut c_void {
    let asked = orb_api::Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    running().sim().windows().create_window(asked, style).0 as *mut c_void
}

/// `SoundPlayer::PlaySounds`: nothing, a laid-out game having no sound system.
///
/// Here rather than left out because the frame loop calls it where the game's own loop did, and a frame
/// that skipped it would be one span of the pacing's breakdown short.
unsafe extern "fastcall" fn play_sounds(_player: usize) {
    running().launch.asked_for(SOUND);
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
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
}

extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
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

/// `MainMenu::AddedCallback` (0x43a5c0): the front end's own read of the score file.
///
/// It fills `g_GameManager` at 0x69ccd0 and 0x69cd30 with `clrd` and `pscr` — which stages the menu offers
/// and whether there is an Extra — and parses nothing else, so the record of spell cards is not what this
/// read is for and nothing of it is put back here. What it *is* for is the open: this is the one read of
/// the file whose answer is the game's own file whichever mode orb is in.
extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    running().opens_the_score_file(false);
    0
}

/// `Ranking::AddedCallback`: the screen built, with the records it shows read out of the score file.
///
/// orb gets in front of this to empty what is in memory first, so that a ranking read defines the
/// history rather than adding to it — see `Game::forget_captures`.
extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    let fake = running();
    fake.reads_the_score_file();
    fake.image.ranking_screen_shown();
    0
}

/// The game's own `CreateFileA`, which the score file's fork calls through with the name it decided.
///
/// Writes down that name and the access, which is the whole of what crosses this call and the whole of what
/// a scenario about that file reads back. The handle is an index into those, one-based so it is never the
/// null the game reads as a failure: there is no file on any disk here, and what a scenario asks is which
/// name the open landed in.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_file(
    name: *const u8,
    access: u32,
    _share: u32,
    _security: *const c_void,
    _disposition: u32,
    _flags: u32,
    _template: *mut c_void,
) -> *mut c_void {
    let fake = running();
    let path = unsafe { CStr::from_ptr(name.cast()) }
        .to_string_lossy()
        .into_owned();
    let write = access & GENERIC_WRITE != 0;
    // Written down before the open is answered, and whether it succeeds or not: an open that failed is an
    // open that happened, and which name it landed in is exactly what a scenario reads back.
    let index = {
        let mut opens = fake.opens.borrow_mut();
        opens.push(Open {
            path: path.clone(),
            write,
        });
        opens.len()
    };
    // A read of a file this game does not keep fails, as a real read of one that is not there does: orb's
    // own file is that until a ranking screen has written it. A write makes the file, so it never fails
    // here — what refuses one is orb, before this is ever reached.
    if !write && !fake.files.borrow().contains_key(&path) {
        return std::ptr::without_provenance_mut(NO_HANDLE as usize);
    }
    std::ptr::without_provenance_mut(index)
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
