//! A th10 that is not the real one: the part of the game orb has read, played over the address space
//! [`orb_core::game::th10::image`] lays out.
//!
//! **What it plays is as much of the game's part as the e2e tests over it ask for**, and each piece of it is
//! `th10.exe`'s own at the addresses written beside it: the frame at 0x439390 in its own order, with the queue of
//! quads, the camera and the fog it sets around its two walks; the walks themselves, entered on the stack as the
//! game's are, and the calc walk's jobs in their priority's order; the supervisor, which reads the keys through the
//! pad's half of the read and switches the scene; the title's entry, the screen that waits for a key, the main menu
//! with the Extra's item and the way out, the three selects a run is chosen through and the practice's stage select;
//! the stage's build on a loader thread, with the replay's seam for the seed inside it, and the stage after one won
//! with the run's numbers carried; the player's states, its move and its shot; aimed fire, a midboss of two attacks
//! and a boss on a stage script of this file's own; the pause object's menu and the end of a run's, with the
//! ranking's name entry; the ending, and the clear's name entry and the replay's slots after it; the score object
//! read at the startup and written at each run's teardown; the window it makes and its device; and the sound
//! manager's queue of commands.
//!
//! **What it does not play, and says so where a piece would be**: a bomb of its own — one is let go and put out
//! where an e2e test says — the lasers, the lines of a conversation,
//! the pause menu's confirmation, the difficulty select's cancel, and the attract demo coming up by itself. A screen
//! or a state this file has no handler for stands where it was put.
//!
//! **Where a number here is th10's it says so**, and where it is this file's own it says that. Over [`Launch`],
//! which is the same half of a launch every other game's e2e tests run on.

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::sync::atomic::Ordering;

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th10::image::{
    A_BLANK_NAME, Image, Pause, Player, STEP_BUILT, STEP_DECIDED, STEP_LEFT, STEP_READING,
    STEP_WAITING, Scene, Screen, THE_PAUSES_COMMAND, THE_RESUMES_COMMAND, TheTitlesEntry,
};
use orb_core::game::th10::{Th10, score_file};
use orb_sim::keys;

use super::{DRAW, Display, Launch, Launched, PRESENT, SOUND, UPDATE, WINDOW, Work, scratch};

/// **The names this game's own record of a frame uses past the four every game's has** — see
/// [`Launched::asked`]: the queue drawn by the frame (0x442f50, at 0x439401 and 0x4394e8), the camera set
/// (0x421480 at 0x43940c) and the fog set (0x420cd0 at 0x4394cc).
pub const QUEUE_DRAWN: &str = "queue drawn";
pub const CAMERA_SET: &str = "camera set";
pub const FOG_SET: &str = "fog set";

/// **The calls of one frame, in the order the game's own frame at 0x439390 makes them** — `asm.md`'s *The frame*:
/// the queue drawn, the camera set, the calc walk, the sounds, the fog set inside the scene, the draw walk, the
/// queue drawn again and the present.
pub const THE_FRAMES_OWN_ORDER: [&str; 8] = [
    QUEUE_DRAWN,
    CAMERA_SET,
    UPDATE,
    SOUND,
    FOG_SET,
    DRAW,
    QUEUE_DRAWN,
    PRESENT,
];

/// **Which key is which bit of the word the read folds the keyboard into** — the `GetKeyboardState` half of
/// 0x44a5f0 (0x44a63c through 0x44a74f): `Z` 0x1, `X` 0x2, Shift 0x4, Esc 0x8, the arrows 0x10 through 0x80,
/// `Q` 0x200, Enter 0x1000 and `R` 0x4000 — the keys an e2e test here presses. Ctrl's 0x100 and the rest are out,
/// no e2e test here pressing them.
const MAP: [(u8, u16); 11] = [
    (keys::Z, 0x0001),
    (keys::X, 0x0002),
    (SHIFT, 0x0004),
    (keys::ESCAPE, 0x0008),
    (keys::UP, 0x0010),
    (keys::DOWN, 0x0020),
    (LEFT, 0x0040),
    (RIGHT, 0x0080),
    (Q, 0x0200),
    (keys::RETURN, 0x1000),
    (R, 0x4000),
];

/// The keys `orb_sim::keys` does not name: the two arrows across, Shift, `Q` and `R`.
const LEFT: u8 = 0x25;
const RIGHT: u8 = 0x27;
const SHIFT: u8 = 0x10;
const Q: u8 = 0x51;
/// `R`, which the pause menu takes as its own way to a retry (0x422e28).
pub const R: u8 = 0x52;

/// **The masks the game's screens read the buttons gone down against**: decide `0x1001` and cancel `0xa` on every
/// menu (0x42d5ff, 0x42d59c), any key `0x160b` on the screen that waits for one (0x42d3a0), up `0x10`, down `0x20`,
/// left `0x40` and right `0x80` (0x42d525, 0x42d542, 0x430808, 0x430825), the pause `0x8` (0x42240f), and the
/// pause menu's own retry `0x4000` and title `0x200` (0x422e28, 0x422e9b). This file's own literals, for the
/// reason `th10::image`'s module doc gives.
const DECIDE: u16 = 0x1001;
const CANCEL: u16 = 0x000a;
const ANY_KEY: u16 = 0x160b;
const UP: u16 = 0x0010;
const DOWN: u16 = 0x0020;
const TO_THE_LEFT: u16 = 0x0040;
const TO_THE_RIGHT: u16 = 0x0080;
/// And the focus, `& 0x4` of the run's word (0x4251a5).
const FOCUS: u16 = 0x0004;
const THE_PAUSE: u16 = 0x0008;
const THE_PAUSES_RETRY: u16 = 0x4000;
const THE_PAUSES_TITLE: u16 = 0x0200;

/// **How many frames a launch left to itself spends before its title is wanted**: the loading object's job that
/// writes scene 4 once `th10.dat`'s files are in (0x41fdbc). This file's own number, as th08's is.
const TITLE_COMES_UP_AFTER: u32 = 60;

/// **The frames each screen of the title ignores a key for, and the frames a decide takes to act**: past ten on
/// the screen that waits for a key (0x42d383) and the main menu (0x42d4ca), past six on the three selects
/// (0x4303fd, 0x4307d4, 0x430d3e); thirty after the key (0x42d3ef), twenty after the main menu's decide
/// (0x42d726), fourteen after the difficulty's and the character's (0x43059a, 0x430966) and forty after the
/// shot's (0x430ee3).
const A_KEY_IS_READ_PAST: i32 = 10;
const A_SELECT_READS_PAST: i32 = 6;
const THE_KEY_ACTS_AT: i32 = 30;
const THE_MAIN_MENU_ACTS_AT: i32 = 20;
const A_SELECT_MOVES_ON_AT: i32 = 14;
const THE_SHOT_STARTS_THE_RUN_AT: i32 = 40;
/// **The counts of items each screen's cursor moves over**: the main menu's eight (0x42d43b), four difficulties
/// outside the Extra (0x4303a6 through 0x4303ab), two characters (0x4306bb) and three shots (0x430a7b).
const MAIN_MENU_ITEMS: i32 = 8;
const DIFFICULTIES: i32 = 4;
const CHARACTERS: i32 = 2;
const SHOTS: i32 = 3;
/// **The main menu's items this file plays**: a run (0x42d749), and the Extra (0x42d757), which the menu leaves out
/// where the score object says nobody has opened it (0x42d442 through 0x42d45c).
pub const THE_ITEM_FOR_A_RUN: i32 = 0;
const THE_ITEM_FOR_THE_EXTRA: i32 = 1;
/// And a practice, which sets the run's flag for one and goes the same way to the difficulty (0x42d7ce through
/// 0x42d7eb).
pub const THE_ITEM_FOR_A_PRACTICE: i32 = 2;
/// **The practice's stage select** (0x430ff0): six stages (0x43100b), read past ten frames (0x431060), and a
/// decide on an offered stage starting it forty frames on (0x431352) — the shot select going to it ten frames
/// after its own decide (0x430e7d) where the run is a practice — and a cancel back to the shot after six
/// (0x4313b4).
const PRACTICE_STAGES: i32 = 6;
const THE_PRACTICE_SELECT_READS_PAST: i32 = 10;
const A_PRACTICE_IS_CHOSEN_AT: i32 = 10;
const A_PRACTICE_STARTS_AT: i32 = 40;
const A_PRACTICE_CANCELS_AT: i32 = 6;
/// And the item a cancel puts the cursor on, the last (0x42d5c0 through 0x42d5dc).
const THE_LAST_ITEM: i32 = 7;

/// **The frames a stage's build runs on its thread**, from the frame the game object is made to the one its
/// flag is taken off: a launch read the stage's frames at nought through all of them, 55 before a run's first
/// stage (`asm.md`'s *The stage's script*).
const A_BUILD_RUNS_FOR: u32 = 55;
/// **The replay's record the build hands the seam**, `push 0x477710` beside the game object's `+0x5c` (0x417a86).
const THE_REPLAYS_RECORD: u32 = 0x0047_7710;

/// **The frames the player's states run for**: sixty coming back in (0x425902 through 0x42590f), eight for a
/// bomb to take a hit back (0x425a96), and thirty dying before the run goes on or ends (0x425b9d).
const COMING_BACK_IN_FOR: i32 = 60;
const A_HIT_WAITS_FOR: i32 = 8;
const DYING_FOR: i32 = 30;

/// **The pause object's frames**: its opening waits until the game object is thirty frames in (0x422406), opens
/// over ten (0x422c9f), closes over twelve (0x423143); the end of a run reads a decide past ten (0x42371c), tallies
/// for ten (0x42377e) and leaves twelve after its last decide (0x42431d).
const A_PAUSE_OPENS_PAST: i32 = 30;
const THE_PAUSE_OPENS_OVER: i32 = 10;
const THE_PAUSE_CLOSES_OVER: i32 = 12;
const THE_END_READS_PAST: i32 = 10;
const THE_END_TALLIES_FOR: i32 = 10;
const THE_END_LEAVES_AFTER: i32 = 12;
/// **The pause menu's items**: three, or two in a replay (0x422cb5 through 0x422cc3), the retry the third; and the
/// end of a run's three, whose second is the replay's save (0x423991).
const THE_PAUSES_ITEMS: i32 = 3;
const A_REPLAYS_PAUSE_ITEMS: i32 = 2;
const THE_PAUSES_RESUME: i32 = 0;
const THE_PAUSES_WAY_TO_THE_TITLE: i32 = 1;
const THE_PAUSES_WAY_TO_A_RETRY: i32 = 2;
const THE_ENDS_ITEMS: i32 = 3;
const THE_ENDS_SAVE: i32 = 1;
const THE_ENDS_LAST_ITEM: i32 = 2;

/// **The difficulty a placed run is played on**, Normal — the one `.data`'s file part holds at `0x474c74` and so
/// the one the difficulty select comes up on — and the stage it starts in, as orb counts one.
pub const DIFFICULTY_NORMAL: i32 = 1;
pub const FIRST_STAGE: i32 = 0;

/// **The stage this file plays, which is its own script and not a stage of `th10.dat`'s**: a bullet aimed at the
/// player every so many frames from the first, a midboss of two attacks part way in to the stage's own song, and
/// the stage's boss for its last frames to a song of her own — a nonspell, a card and a nonspell — beaten a little
/// before the stage ends. What it has to be is a stage the rules have something to do in and a fight of the shape
/// orb cuts chapters at, each attack longer than the shortest a chapter may be.
const FIRST_BULLET_AT: u32 = 480;
const A_BULLET_EVERY: u32 = 120;
const BULLETS_FROM: (f32, f32) = (0.0, 32.0);
const BULLET_SPEED: f32 = 2.0;
const BULLET_SIZE: f32 = 8.0;
/// How many of the manager's `0x7d0` slots this stage uses, from the first.
const BULLET_SLOTS_USED: usize = 16;
/// **Where a bullet is off the field and freed**, past the field's `384x448` from its top middle by a bullet.
const THE_FIELD_ACROSS: f32 = 192.0 + BULLET_SIZE;
const THE_FIELD_DOWN: (f32, f32) = (-BULLET_SIZE, 448.0 + BULLET_SIZE);
/// The seed the generator starts a launch from — see [`Fake::attach_finding`].
const A_SEED: u16 = 0x4d2f;
const THE_MIDBOSS_ARRIVES_AT: u32 = 1_200;
const AN_ATTACK_LASTS: u32 = 300;
const THE_BOSS_FIGHTS_FOR: u32 = 900;
const AFTER_THE_BOSS: u32 = 120;
const A_FIGHT_STANDS_AT: (f32, f32) = (0.0, 96.0);
const A_FIGHTS_LIFE: i32 = 10_000;
/// The cards the two fights put up, by the number the card object keeps: the midboss's and the boss's, numbers
/// of this file's own among the 110.
pub const THE_MIDBOSSS_CARD: i32 = 2;
pub const THE_BOSSS_CARD: i32 = 7;
/// **The tracks of `thbgm.fmt` the stage is played to**: the stage's own, and the boss's, which the conversation
/// before her puts on (0x416a91 through 0x416a9c) — by their records' names `th10_00.wav` and `th10_01.wav`, stage
/// 1's own two, whichever stage it is.
pub const THE_STAGES_TRACK: usize = 0;
pub const THE_BOSSS_TRACK: usize = 1;
/// **The last stage of a run**, after which the panel asks for the ending instead of the next stage (0x416eef), and
/// the frames it waits first: `0x78` (0x414918).
const THE_LAST_STAGE: i32 = 6;
const THE_ENDING_IS_ASKED_FOR_AFTER: u32 = 0x78;
/// **The Extra's stage word**, 7, which the shot select writes where the difficulty is the Extra's (0x430f3f) and
/// whose end is the run's (0x416de8).
const THE_EXTRAS_STAGE: i32 = 7;
/// **The Extra's difficulty**, 4, which its item of the main menu writes (0x42d790) and the selects after it read
/// to take only what the Extra offers (0x430394, 0x4306c2, 0x430a82).
const DIFFICULTY_EXTRA: i32 = 4;
/// **How many frames the ending runs before its last line**, which then stands until the decide goes down and asks
/// for the title after it (scene 15, 0x4219bb): the script's wait of `-1` puts its timer back at 999 on every
/// update (0x40befa through 0x40bf2b) and goes on only on a press of `0x1001` (0x40bf30) — the ending a skip of
/// 54,000 updates with no key in them never ended on the real game, its timer read at 999 throughout (2026-10-02).
/// The count is this file's own, the ending's own length not having been read: what it has to be is a count orb's
/// skip of an ending reaches.
const THE_ENDING_LASTS: u32 = 600;

/// **What a stage won adds to the run's score**, in the score's own tenths. This file's own number, nothing of
/// the stage's scoring having been read: what it has to be is a score a whole run of six puts in the ranking the
/// game's own code makes empty — ten places from 1,000,000 down by 100,000 — so that the name entry after a clear
/// asks for a name, as it does after a real one.
pub(crate) const A_STAGE_WON_SCORES: u32 = 200_000;

/// **The grid the title's name entry for a clear walks**, the string `[0x4746d8]` points at (`0x46e2f8`): 88
/// characters and then the three cells for a space, a step back and the end (0x433015, 0x433087, 0x4330ac), in
/// rows of thirteen the arrows move over (0x432f36, 0x432f50). Read with `objdump -s --start-address=0x46e2f8
/// --stop-address=0x46e353 th10.exe`.
const THE_GRID: &[u8; 91] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=.,!?@:;[]()_/{}|~^#$%&*   ";
const A_GRIDS_ROW: i32 = 13;
/// **The frames the name entry's first step, its last, and the replay's slots each wait** past or out (0x432eda,
/// 0x4331ac, 0x43362f, 0x433a33), and the slots' count (0x433599).
const THE_ENTRY_READS_PAST: i32 = 6;
const THE_ENTRY_LEAVES_AT: i32 = 6;
const THE_SLOTS_READ_PAST: i32 = 6;
const A_CANCEL_LEAVES_THE_SLOTS_AT: i32 = 6;
const THE_REPLAYS_SLOTS: i32 = 25;
/// **The frame of the game object's timer the stage after one won begins on**, its stage before faded out
/// (0x41840b).
const A_FADE_BEGINS_THE_STAGE_AT: i32 = 0x1e;

/// **The class 0x439730 registers (0x4397a7) and makes its window on**: `BASE`, at `0x46f698` — th06's class too,
/// which is what orb recognises the window it plays in by: see `orb_core::window::is_game_class`.
const WINDOW_CLASS: &std::ffi::CStr = c"BASE";

/// **One of the two windows 0x439730 asks `CreateWindowExA` for**: the style, where, and the whole size.
#[derive(Clone, Copy)]
pub struct Ask {
    pub style: u32,
    pub at: (i32, i32),
    pub size: (i32, i32),
}

/// **The one where `th10.cfg` says the whole screen** (0x4397b6 through 0x4397dc): `WS_OVERLAPPEDWINDOW` at the
/// origin, the back buffer's own 640x480.
pub const ASKED_FOR_FULL_SCREEN: Ask = Ask {
    style: 0x00cf_0000,
    at: (0, 0),
    size: (640, 480),
};

/// **And where it says a window** (0x4397e4 through 0x43982c): `WS_VISIBLE | WS_SYSMENU | WS_MINIMIZEBOX` at
/// `CW_USEDEFAULT`, the back buffer with `GetSystemMetrics`' dialog frame either side (`SM_CXDLGFRAME`,
/// `SM_CYDLGFRAME`) and its caption (`SM_CYCAPTION`) above. The three metrics are this file's own, the laid-out
/// host answering none.
const ASKED_FOR_A_WINDOW: Ask = Ask {
    style: 0x100a_0000,
    at: (i32::MIN, i32::MIN),
    size: (640 + 2 * DIALOG_FRAME.0, 480 + 2 * DIALOG_FRAME.1 + CAPTION),
};
const DIALOG_FRAME: (i32, i32) = (3, 3);
const CAPTION: i32 = 23;

/// **Where the fight of a stage stands**, which this file's own script moves on frame by frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Fight {
    /// Nobody up.
    None,
    /// The midboss or the boss, on the attack counted from nought.
    Up { boss: bool, attack: u32 },
    /// Beaten, which the midboss is once and the boss is for the rest of the stage.
    Beaten { boss: bool },
}

/// A th10 laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// Whether orb stands in front of this game's pad half of the read.
    reads_the_keys: bool,
    /// How many frames this launch has run, for the title wanted on its own — see [`TITLE_COMES_UP_AFTER`].
    frames_since_launch: Cell<u32>,
    /// **The frames the stage's build has left on its thread**, where one is running — see [`A_BUILD_RUNS_FOR`].
    building: Cell<Option<u32>>,
    /// **Whether the stage's own jobs are on the chain**, which the game object's frame 0 turns them on for
    /// (0x4182a9 through 0x4183ca) and a run's teardown takes off.
    the_stage_is_on: Cell<bool>,
    /// **What the replay's seam saw each time the build reached it**: the game object's flags and the stage's
    /// frames — see [`Fake::what_the_build_seeded_on`].
    seeded_on: RefCell<Vec<(u32, u32)>>,
    /// How many times the options were made for the power, the pause menu's resume was made, and the device told
    /// the fog is off.
    options_made: Cell<u32>,
    /// How many times orb stepped the player's shot through the game's own call — see [`the_shot_is_stepped`].
    shot_stepped: Cell<u32>,
    /// **Whether the player's shot sits out this update**, which is the update a stage built from the menu begins
    /// on: measured on the real game, a pick-up of stage 2 built fresh fired its first shots an update after the
    /// stage's clock went back to 1, where the run that came into that stage out of the last one's fade fired them
    /// on that very update — the shot timer 1 against 2 on frame 2, everything else of the world agreeing.
    the_shot_waits: Cell<bool>,
    unpaused: Cell<u32>,
    fog_told: Cell<u32>,
    /// **The behaviour flags the host's `IDirect3D9::CreateDevice` was last handed**, and nothing before a device is
    /// made — see [`Fake::makes_its_direct3d_and_its_device`].
    device_made_with: Cell<Option<u32>>,
    /// And the presentation interval in the parameters it was handed — see [`Fake::the_device_presents_with`].
    device_presents_with: Cell<Option<u32>>,
    /// The word the pad's half of the read answered on the last update — see
    /// [`Fake::word_the_input_read_answered`].
    word: Cell<u16>,
    /// **Whether the run's stage is played by this file's own script** — see [`FIRST_BULLET_AT`] — and how long
    /// a stage of it lasts, where an e2e test said; and whether it fires.
    on_its_own: Cell<bool>,
    stage_lasts: Cell<Option<u32>>,
    fires: Cell<bool>,
    /// **Whether a fight laid out by hand has its attack's timer stepped every update**, as the game's enemy
    /// update steps it (0x40e572) — see [`Fake::steps_the_fight_laid_out`].
    steps_the_fight_laid_out: Cell<bool>,
    /// Where the stage's fight stands.
    fight: Cell<Fight>,
    /// **The frames the panel has counted towards the ending**, where the last stage has been won (0x41490f).
    towards_the_ending: Cell<Option<u32>>,
    /// The frames the ending has run, which it asks for the scene after it at the end of — see
    /// [`THE_ENDING_LASTS`].
    the_ending_has_run_for: Cell<u32>,
    /// **Whether the startup's read of `scoreth10.dat` found a file**, the buffer it keeps at the score object's
    /// `+0x0` (0x42ae88) — which the object's write asks before anything else, writing nothing where it is null
    /// (0x42b1e0 through 0x42b1ef).
    read_a_score_file: Cell<bool>,
    /// **The names each `CreateFileA` of this game's landed on**, in order — which is what says which of the two
    /// score files a write of the game's went to, where orb's hook stands in front of the call.
    opens: RefCell<Vec<String>>,
    /// **Whether the game is leaving**, its switch having built scene 3 — see [`Fake::switches_the_scene`].
    leaving: Cell<bool>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory.
    _installed: orb_api::Installed,
}

/// **The facts about the machine a launch is declared with** beside its display: the language its windows are in,
/// whether it says where its panel is, and whether the game has made its device yet.
struct Host {
    ui_language: Option<u16>,
    monitor: bool,
    device: bool,
}

impl Host {
    /// What an ordinary launch has: the machine's own language, a panel to lay a window out against, and a device
    /// made before orb is attached.
    const ORDINARY: Self = Self {
        ui_language: None,
        monitor: true,
        device: true,
    };
}

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into — plain functions with nothing but the
    /// ABI's arguments.
    static RUNNING: Cell<*const Fake> = const { Cell::new(std::ptr::null()) };
}

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

    /// **One frame of the game, entered the way this game's loop enters its frame**: pushed, `push 0x4924f0; call
    /// 0x439390` at 0x438d2c — which is `runtime::render_on_the_stack` where orb's loop stands over the prologue.
    ///
    /// **And the process ended where the game is leaving**: a frame that answers 1 leaves WinMain's loop
    /// (0x438d48), and its teardown (0x438dab) and the runtime's exit after it are the end of the process.
    fn frame(&self) -> i32 {
        let answered = if self.launch.own_frame_loop {
            unsafe { orb_core::runtime::render_on_the_stack(self.frame_window()) }
        } else {
            self.own_render()
        };
        if self.leaving.get() {
            orb_api::process::exit(0);
        }
        answered
    }
}

impl Fake {
    /// Lays the game out on a display an e2e test says the whole of, gives it a device, and attaches orb to it the
    /// way a launch there is attached. Boxed and never moved for the reason th06's is: the hooks find it through a
    /// pointer.
    pub fn attach(display: Display, name: &str, work: Work) -> Box<Self> {
        Self::attach_configured(display, name, work, |_| ())
    }

    /// And the same launch with a setting of its own answered.
    pub fn attach_configured(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_finding(display, name, &[], work, settings)
    }

    /// And on a host that will not say where its panel is — **no monitor declared**, which is a window orb has
    /// nothing to lay out against: see `the_windows_orb_leaves_alone_of_th10.rs`.
    pub fn attach_without_a_monitor(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let host = Host {
            monitor: false,
            ..Host::ORDINARY
        };
        Self::attach_declaring(display, host, name, &[], work, settings)
    }

    /// And on a machine whose own windows are in the language `ui_language` names — `orb_sim::langid` — with the
    /// launch otherwise the one [`Fake::attach_configured`] makes.
    pub fn attach_on_a_machine_in(
        ui_language: u16,
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let host = Host {
            ui_language: Some(ui_language),
            ..Host::ORDINARY
        };
        Self::attach_declaring(display, host, name, &[], work, settings)
    }

    /// And in a directory that already holds `left`, each file at its path inside the directory — what an earlier
    /// launch left there, found at the attach.
    pub fn attach_finding(
        display: Display,
        name: &str,
        left: &[(std::path::PathBuf, Vec<u8>)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, Host::ORDINARY, name, left, work, settings)
    }

    /// And laid out with **no device**, which is what a process is when orb attaches to it: `DllMain` runs long
    /// before WinMain's `0x439890` (called at 0x438c2c) has `IDirect3D9::CreateDevice` write the device to
    /// `0x491c30` (0x439a06, 0x439a44, 0x439a83). See [`makes_its_device`](Fake::makes_its_device), which is
    /// the other half of this.
    pub fn attach_before_its_device(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let host = Host {
            device: false,
            ..Host::ORDINARY
        };
        Self::attach_declaring(display, host, name, &[], work, settings)
    }

    /// Every launch above, with the host facts it is declared with.
    fn attach_declaring(
        display: Display,
        host: Host,
        name: &str,
        left: &[(std::path::PathBuf, Vec<u8>)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let dir = scratch(name);
        let image = Image::laid_out_seeded(display.seed);
        let installed = image.enter();
        // And the language its windows are in, before the attach settles which language orb's screens are.
        if let Some(ui_language) = host.ui_language {
            image.sim().set_ui_language(ui_language);
        }
        image.sim().log().waits_first(orb_core::log::written_out);
        image.sim().files().make(&dir);
        for (file, contents) in left {
            image.sim().files().put(dir.join(file), contents);
        }
        let mut config =
            Config::load_beside(&dir.join(EXE)).expect("a directory with no orb.yaml in it");
        // The memory hooks patch an import table there is none of.
        config.track_memory = false;
        settings(&mut config);
        let recipe = config.autoplay.clone();
        // **No `font.ttf` beside this game's exe**, th10 keeping its own inside `th10.dat`: the face is there for
        // the harness's own half of the launch and gone before orb reaches for one.
        let font = dir.join("font.ttf");
        image.sim().text().install_font(&font);
        let launch = Launch::new(
            image.sim(),
            dir,
            display.seed,
            super::orbs_own_loop(&config, the_game_this_is()),
        );
        image.sim().text().remove_font(&font);
        image.shows_through(
            match host.device {
                true => launch.device(),
                false => orb_api::Device::NULL,
            },
            WINDOW,
        );
        image.hands_over_the_frames_own_calls(
            draws_the_queue_in_esi as *const () as usize,
            sets_the_camera_in_edi as *const () as usize,
            sets_the_fog_in_edi as *const () as usize,
            the_options_are_made as *const () as usize,
            unpauses_in_esi as *const () as usize,
        );
        image.hands_over_the_landings_own_calls(
            steps_the_shot_in_eax as *const () as usize,
            the_shots_are_moved as *const () as usize,
            the_background_runs as *const () as usize,
        );
        image.hands_over_the_score_objects_own_call(a_record_is_emptied as *const () as usize);
        // **The generator seeded as the supervisor starts**, from `timeGetTime` (0x4201d0) — a number of this
        // file's own, the laid-out clock having none worth seeding from.
        image.the_generator_is_seeded_with(A_SEED);
        image.sim().set_host_exe(launch.dir().join(EXE));
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
        if host.monitor {
            sim.windows()
                .set_monitor(orb_sim::Monitor::scaled(), orb_sim::Frame::SOME);
        }

        let fake = Box::new(Self {
            image,
            launch,
            reads_the_keys: the_game_this_is().hooks().input.is_some(),
            frames_since_launch: Cell::new(0),
            building: Cell::new(None),
            the_stage_is_on: Cell::new(false),
            seeded_on: RefCell::new(Vec::new()),
            options_made: Cell::new(0),
            shot_stepped: Cell::new(0),
            the_shot_waits: Cell::new(false),
            unpaused: Cell::new(0),
            fog_told: Cell::new(0),
            device_made_with: Cell::new(None),
            device_presents_with: Cell::new(None),
            word: Cell::new(0),
            on_its_own: Cell::new(false),
            stage_lasts: Cell::new(None),
            fires: Cell::new(true),
            steps_the_fight_laid_out: Cell::new(false),
            fight: Cell::new(Fight::None),
            towards_the_ending: Cell::new(None),
            the_ending_has_run_for: Cell::new(0),
            read_a_score_file: Cell::new(false),
            opens: RefCell::new(Vec::new()),
            leaving: Cell::new(false),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        // **The three trampolines a launch of this game leaves in the slots of its own**, which is what `orb`'s
        // install writes for a game whose walks and frame take their argument on the stack — see
        // `orb_core::runtime::run_calc_chain_on_the_stack` — with the pushing calls in front of them as the
        // originals `attach_to` is handed.
        orb_core::runtime::RUN_CALC_CHAIN_ON_THE_STACK
            .store(update as *const () as usize, Ordering::Relaxed);
        orb_core::runtime::RUN_DRAW_CHAIN_ON_THE_STACK
            .store(draw as *const () as usize, Ordering::Relaxed);
        orb_core::runtime::RENDER_ON_THE_STACK
            .store(own_render as *const () as usize, Ordering::Relaxed);
        unsafe {
            orb_core::runtime::attach_to(
                the_game_this_is(),
                config,
                fake.image.data(),
                orb_core::runtime::Originals {
                    update: calls_the_calc_walk_on_the_stack,
                    draw: calls_the_draw_walk_on_the_stack,
                    // **The pad's half, in the one slot every game's read goes in**, whose type is the read of
                    // nothing th06's is: `runtime::get_input_with_the_keys` takes it back out as the `fastcall`
                    // it is.
                    input: std::mem::transmute::<
                        extern "fastcall" fn(u32, u32, u32, u32) -> u16,
                        extern "system" fn() -> u16,
                    >(pad_half),
                    // And the replay's seam the same way, a `stdcall` of two that
                    // `runtime::stage_building_of_two` calls through as one.
                    stage_building: std::mem::transmute::<
                        extern "stdcall" fn(u32, u32) -> u32,
                        extern "C" fn(i32) -> i32,
                    >(replay_built),
                    stage_begun,
                    unlocks_read,
                    ranking_read,
                    card_records_read,
                    render: calls_the_frame_on_the_stack,
                    play_sounds,
                    present,
                    create_window,
                    create_file,
                    stop_recording,
                    create_game_window,
                    joystick_position,
                    pad_buttons,
                    save_replay,
                    init_d3d_device,
                    tops_up_the_music,
                    stops_the_music,
                },
            )
        };
        // **And the host's `SetForegroundWindow` handed over where orb writes over its import entry**, for the one
        // call of it the game makes — see [`Fake::brings_its_window_forward`] — and its `Direct3DCreate9` the same
        // way, see [`Fake::makes_its_direct3d_and_its_device`].
        unsafe {
            orb_core::window::answers_the_window_brought_forward(
                host_brings_a_window_forward as *const () as usize,
            );
            orb_core::window::answers_the_direct3d_made(
                host_makes_a_direct3d as *const () as usize,
            );
        };
        if let Some(recipe) = &recipe {
            unsafe { orb_bot::install(recipe, the_game_this_is()) };
        } else {
            // **And a launch with no recipe takes the last one's bot out**, for th06c's fake's reason.
            unsafe { orb_core::runtime::plays_the_run(None) };
        }
        fake.frame_takes(work);
        fake
    }

    /// **The game bringing the window it plays in to the front**, as WinMain does once the window is made and
    /// before its message loop: `SetForegroundWindow` on `0x4924f0` through the import at `0x4661f0` (0x438cb1) —
    /// which reaches orb's `orb_core::window::set_foreground_window` where orb wrote over that entry.
    pub fn brings_its_window_forward(&self, window: orb_api::Hwnd) -> i32 {
        unsafe { orb_core::window::set_foreground_window(window.0 as *mut c_void) }
    }

    /// **The game making its Direct3D object and its device**, as `0x439890` does: `Direct3DCreate9` through its import
    /// — which reaches orb's `orb_core::window::direct3d_create9` where orb wrote over that entry — kept at
    /// `0x491c2c`, and `CreateDevice` through the object's own vtable, `call [ecx+0x40]` (0x439a06), on the window with
    /// `D3DCREATE_HARDWARE_VERTEXPROCESSING` (`push 0x40`) and its present parameters at `0x491d0c`, the ones its
    /// `Reset` is handed too (0x439232). What the host's `CreateDevice` was handed is
    /// [`Fake::the_device_was_made_with`] and [`Fake::the_device_presents_with`].
    pub fn makes_its_direct3d_and_its_device(&self, window: orb_api::Hwnd) {
        let direct3d = unsafe { orb_core::window::direct3d_create9(D3D_SDK_VERSION) };
        let space = self.sim().space();
        space.write::<u32>(
            PRESENT_PARAMETERS + orb_api::d3d9::PRESENTATION_INTERVAL,
            THE_GAMES_PRESENTATION_INTERVAL,
        );
        let vtable = space.read::<usize>(direct3d as usize);
        let create_device: orb_core::window::CreateDevice9 = unsafe {
            std::mem::transmute(
                space
                    .read::<usize>(vtable + orb_api::d3d9::CREATE_DEVICE_SLOT * size_of::<usize>()),
            )
        };
        let mut device = std::ptr::null_mut();
        unsafe {
            create_device(
                direct3d,
                0,
                D3DDEVTYPE_HAL,
                window.0 as *mut c_void,
                D3DCREATE_HARDWARE_VERTEXPROCESSING,
                PRESENT_PARAMETERS as *mut c_void,
                &mut device,
            )
        };
    }

    /// The behaviour flags the host's `CreateDevice` was handed, and nothing before a device was made.
    pub fn the_device_was_made_with(&self) -> Option<u32> {
        self.device_made_with.get()
    }

    /// **The presentation interval in the parameters the host's `CreateDevice` was handed**, which is what says
    /// whether the device's `Present` waits for the display's blank — and nothing before a device was made.
    pub fn the_device_presents_with(&self) -> Option<u32> {
        self.device_presents_with.get()
    }

    /// This game's memory, for an e2e test that reads back what a frame of orb's did to it.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// **The game making the window it plays in**, as 0x439730 does once its class is registered: one
    /// `CreateWindowExA` on `BASE`, `ASKED_FOR_A_WINDOW` or [`ASKED_FOR_FULL_SCREEN`] by the byte
    /// `th10.cfg` was read into (0x4397ad) — and with nothing of orb's in front of that read, `Th10::hooks`
    /// declaring no `create_window`.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        let ask = match self.image.is_windowed() {
            true => ASKED_FOR_A_WINDOW,
            false => ASKED_FOR_FULL_SCREEN,
        };
        let window = unsafe {
            orb_core::window::create_window_ex_a(
                0,
                WINDOW_CLASS.as_ptr().cast(),
                c"th10".as_ptr().cast(),
                ask.style,
                ask.at.0,
                ask.at.1,
                ask.size.0,
                ask.size.1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        orb_api::Hwnd(window as usize)
    }

    /// **The device made, as `0x439890` makes it**: `IDirect3D9::CreateDevice` writing it to `0x491c30` (0x439a06,
    /// 0x439a44, 0x439a83) — and with nothing of orb's in front of the call, `Th10::hooks` declaring no
    /// `init_device`. WinMain calls it (0x438c2c) before its message loop (0x438cf1), which is the one caller of
    /// the frame, so a launch's first frame already has it.
    pub fn makes_its_device(&self) {
        self.image.shows_through(self.launch.device(), WINDOW);
    }

    /// The state orb reads of this game, for an e2e test that asks what a frame left.
    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th10.read_state() }
    }

    /// How many times the game has told its device the fog is off — see [`the_fog_is_set`].
    pub fn times_the_fog_was_put_out(&self) -> u32 {
        self.fog_told.get()
    }

    /// How many times the options were made for the power, which only a stage's numbers put back does here.
    pub fn times_the_options_were_made(&self) -> u32 {
        self.options_made.get()
    }

    /// How many times orb stepped the player's shot through the game's own call, which only a pick-up landing
    /// does.
    pub fn times_orb_stepped_the_shot(&self) -> u32 {
        self.shot_stepped.get()
    }

    /// The player's shot timer as it stands.
    pub fn the_shot_timer(&self) -> i32 {
        self.image.the_shot_timer()
    }

    /// How many times the pause menu's resume was made, the game's own or orb's.
    pub fn times_the_run_was_unpaused(&self) -> u32 {
        self.unpaused.get()
    }

    /// **The game object's flags and the stage's frames as the replay's seam saw them**, once for every build that
    /// reached it — which is what says the seed goes in on the build's thread and before its stage has a frame.
    pub fn what_the_build_seeded_on(&self) -> Vec<(u32, u32)> {
        self.seeded_on.borrow().clone()
    }

    /// **The title up on its main menu and reading a key**, for an e2e test that is not about the frames a launch
    /// spends before it: the scene the supervisor has built it on, the first title passed, and the main menu past
    /// the frames it ignores a key for.
    pub fn its_title_menu_is_up(&self) {
        self.frames_since_launch.set(TITLE_COMES_UP_AFTER + 1);
        self.image.is_in_scene(Scene::Title, Scene::Title);
        self.image.the_title_is_built();
        self.image.the_title_has_been_passed();
        self.image.the_title_is_on(Screen::MainMenu);
        self.the_main_menu_is_built();
        self.image.the_titles_step_is(STEP_READING);
        for _ in 0..=A_KEY_IS_READ_PAST {
            self.image.the_title_counts_a_frame();
        }
    }

    /// **A run in its first stage, placed rather than walked to**: what the shot select writes for a Normal run of
    /// the first shot type, the switch's build of it, and the build run to its end — so the next frame is the
    /// stage's first.
    pub fn is_in_a_run(&self) {
        if !self.image.the_title_is_up() {
            self.its_title_menu_is_up();
        }
        self.image.the_difficulty_is(DIFFICULTY_NORMAL);
        self.image.the_character_is(0);
        self.image.the_shot_is(0);
        self.image.the_run_starts_at_stage_1();
        self.image.asks_for_the_scene(Scene::Run);
        self.switches_the_scene();
        while self.building.get().is_some() {
            self.its_loader_goes_on();
        }
    }

    /// **The player hit**, which is what 0x426cf0 writes — the state and its timer (0x426cff) — for an e2e test
    /// over what orb reads of a hit; this laid-out stage fires nothing of its own.
    pub fn the_player_is_hit(&self) {
        self.image.the_player_is_in(Player::Hit);
    }

    /// **A bomb let go**, which is what the player's own bomb leaves behind (0x425998 through 0x4259d4) — for an
    /// e2e test over what orb reads of a bomb, X being nothing this file's player answers.
    pub fn a_bomb_goes_off(&self) {
        self.image.a_bomb_is_going_off();
    }

    /// And the bomb over: what an e2e test says, the bomb's own update being nothing this file plays.
    pub fn the_bomb_is_over(&self) {
        self.image.no_bomb_is_going_off();
    }

    /// **The run's stage played by this file's own script from here on** — see [`FIRST_BULLET_AT`].
    pub fn plays_its_own_stage(&self) {
        self.on_its_own.set(true);
    }

    /// How long one of those stages lasts before the next is asked for, where an e2e test says: a stage of no
    /// length otherwise, which never ends and has no fight.
    pub fn stages_last(&self, frames: u32) {
        self.stage_lasts.set(Some(frames));
    }

    /// **Whether that stage fires at the player**, which it does unless an e2e test says not.
    pub fn fires_at_the_player(&self, fires: bool) {
        self.fires.set(fires);
    }

    /// **A fight laid out by hand with its attack's timer stepped by the enemy manager from here on**, every
    /// update as the game's own enemy update steps it — where a test steps it a frame of its own otherwise, one
    /// for each of the frames it runs.
    pub fn steps_the_fight_laid_out(&self) {
        self.steps_the_fight_laid_out.set(true);
    }

    /// **The word the pad's half of the read answered on the last update it made**, which is where a press orb
    /// held back and then handed over shows up.
    pub fn word_the_input_read_answered(&self) -> u16 {
        self.word.get()
    }

    /// **The two score files a launch finds beside the exe, and the score object the startup read made of the
    /// game's own**: `scoreth10.dat` (`0x46e848`, read at 0x42ae7d) holding `own` and orb's fork of it holding
    /// `modes`. Where orb's own read of either — `orb_api::fs::read`, by the same relative name the game opens it
    /// by — finds it. The startup read is laid rather than made through the fork: that read landing in the game's
    /// own file is the bracket's to decide, which other games' e2e tests are about.
    pub fn its_score_files_hold(&self, own: &score_file::Records, modes: &score_file::Records) {
        let files = self.image.sim().files();
        files.put(THE_GAMES_SCORE_FILE, score_file::encode(own));
        files.put(THE_MODES_SCORE_FILE, score_file::encode(modes));
        self.image.the_score_object_is_read_from(own);
        self.read_a_score_file.set(true);
    }

    /// **The score object written back to `scoreth10.dat`, 0x42b1e0** — which every teardown of a run makes
    /// (0x417c8d), and the game's own way out (0x41fc6e), and nothing else: not the ending, whose clear goes into
    /// the object after the run's teardown has written it (0x421af2 through 0x421b02), nor the title's name entry,
    /// nor orb's `quit`, whose `ExitProcess` runs none of the game's teardown. Nothing where the startup read
    /// found no file (0x42b1e0 through 0x42b1ef); otherwise packed (see
    /// `Image::the_score_object_is_packed_for_its_write`) and opened by the game's own name (`push 0x46e848` at
    /// 0x42b33a) through whatever stands in front of its `CreateFileA`, the bytes going where that open landed —
    /// orb's fork in Pointdevice mode, and nowhere where orb refused the open.
    fn writes_its_score_file(&self) {
        if !self.read_a_score_file.get() {
            return;
        }
        let records = self.image.the_score_object_is_packed_for_its_write();
        let opened = self.opens.borrow().len();
        its_score_file_opened_for_writing();
        let landed = self.opens.borrow().get(opened).cloned();
        if let Some(landed) = landed {
            self.sim().files().put(landed, score_file::encode(&records));
        }
    }

    /// **Another stage than the one asked for built**, which a resume must not write a run over: the stage word
    /// the build reads written with `stage` — orb's own count from nought — after the run has been asked for.
    pub fn builds_the_stage_instead(&self, stage: i32) {
        self.image.the_stage_word_is(stage + 1);
    }

    /// **And the stage asked for built inside the attract demo**, a run orb keeps nothing of — the run's flag the
    /// demo's own start writes.
    pub fn builds_it_inside_the_demo(&self) {
        self.image.the_run_becomes_the_demo();
    }

    /// **A stage asked for that never arrives**: the supervisor on the run's scene and the build that was started
    /// taken down with nothing in its place.
    pub fn the_stage_never_comes_up(&self) {
        self.the_run_is_taken_down();
        self.image.the_title_is_freed();
        self.image.is_in_scene(Scene::Run, Scene::Run);
    }

    /// **The stage after this one asked for**, which is what the conversation after a stage's boss writes: the
    /// stage stepped on and the scene wanted at 11 (0x416ed5 through 0x416ee8).
    pub fn moves_on_to_the_next_stage(&self) {
        self.image.asks_for_the_scene(Scene::NextStage);
        self.image.the_run_moves_on_a_stage();
    }

    /// **The calc walk, 0x449c00**: every job of the chain in its priority order — the supervisor at 1, the title
    /// at 6, the pause object at 9, the game object at `0xa`, the replay at `0xb`, the player at `0x10`, the enemy
    /// manager at `0x12` and the bullets at `0x14` — and what it answers, the count of the jobs it reached (`inc
    /// [esp+0x10]` at 0x449cc7), the game object's `3` while paused stopping it where it stands (0x4186e2 through
    /// 0x4186fd).
    fn walks_its_calc_list(&self) -> i32 {
        let mut reached = 1;
        self.its_supervisor();
        // The supervisor's `4` out of the switch to scene 3 (0x421960), which stops the walk answering nought.
        if self.leaving.get() {
            return 0;
        }
        // **The ending's job** (0x40b940's), which asks for the title after it once its last line is pressed past.
        if self.image.the_scene_now() == Scene::Ending as i32 {
            reached += 1;
            let ran = (self.the_ending_has_run_for.get() + 1).min(THE_ENDING_LASTS);
            self.the_ending_has_run_for.set(ran);
            if ran == THE_ENDING_LASTS && self.image.the_buttons_gone_down() & DECIDE != 0 {
                self.image.asks_for_the_scene(Scene::AfterTheEnding);
            }
        }
        if self.image.the_title_is_up() {
            reached += 1;
            self.walks_its_title();
        }
        if self.the_stage_is_on.get() {
            reached += 1;
            self.its_pause_object();
        }
        // **The game object's job, on the chain from the build's end** (0x409e20 at 0x417c53).
        if !self.image.the_game_object_is_up()
            || self.building.get().is_some()
            || self.image.the_stage_is_being_built()
        {
            return reached;
        }
        reached += 1;
        if !self.its_game_object() {
            return reached;
        }
        if !self.the_stage_is_on.get() {
            return reached;
        }
        reached += 1;
        self.image.the_run_copies_the_word();
        reached += 1;
        self.its_player();
        reached += 1;
        self.its_enemy_manager();
        reached += 1;
        self.its_bullets();
        // **The panel's count towards the ending**, the panel's job (0x414900) — its flag and its `0x78` frames.
        if let Some(counted) = self.towards_the_ending.get() {
            match counted + 1 >= THE_ENDING_IS_ASKED_FOR_AFTER {
                true => {
                    self.towards_the_ending.set(None);
                    self.image.asks_for_the_scene(Scene::Ending);
                }
                false => self.towards_the_ending.set(Some(counted + 1)),
            }
        }
        reached
    }

    /// **The enemy manager's calc job** (0x40d810, priority `0x12`): every enemy's scripts a step on, and the
    /// stage's own script where this file plays one — its fight and its fire, and its end.
    fn its_enemy_manager(&self) {
        self.image.the_enemies_scripts_step();
        // **The generator drawn from once a frame**, which the enemies' scripts do on every stage (0x40e770,
        // 0x412300): how often is this file's own, what it has to be is a word that moves with the stage.
        self.image.the_generator_draws();
        if self.steps_the_fight_laid_out.get() {
            self.image.the_fight_goes_on();
        }
        if !self.on_its_own.get() {
            return;
        }
        let Some(lasts) = self.stage_lasts.get() else {
            return;
        };
        let frame = self.image.the_stages_frames();
        self.the_fight_goes_on(frame, lasts);
        if self.fires.get()
            && frame >= FIRST_BULLET_AT
            && (frame - FIRST_BULLET_AT).is_multiple_of(A_BULLET_EVERY)
        {
            self.fires_a_bullet();
        }
        if frame == lasts {
            self.the_stage_ends();
        }
    }

    /// **The stage's fight, by the stage's own frame**: the midboss from [`THE_MIDBOSS_ARRIVES_AT`] for two
    /// attacks under the stage's own song, and the boss for the stage's last [`THE_BOSS_FIGHTS_FOR`] frames under
    /// hers — each attack its timer from nought and its own card where it is one — gone [`AFTER_THE_BOSS`] before
    /// the stage's end. The midboss only where the stage has room for her before the boss.
    fn the_fight_goes_on(&self, frame: u32, lasts: u32) {
        let image = &self.image;
        let boss_from = lasts.saturating_sub(THE_BOSS_FIGHTS_FOR);
        let midboss_until = THE_MIDBOSS_ARRIVES_AT + 2 * AN_ATTACK_LASTS;
        let a_midboss = midboss_until + AN_ATTACK_LASTS <= boss_from;
        let the_attack = |boss: bool, attack: u32| {
            image.the_fight_attacks_again();
            let card = |number: i32| {
                image.a_card_is_up(number);
                image.a_cards_start_counts_an_attempt(number as usize);
            };
            match (boss, attack) {
                (false, 1) => card(THE_MIDBOSSS_CARD),
                (true, 1) => card(THE_BOSSS_CARD),
                _ => image.no_card_is_up(),
            }
            self.fight.set(Fight::Up { boss, attack });
        };
        let the_fights_over = |boss: bool| {
            image.no_card_is_up();
            image.the_fight_is_over();
            self.fight.set(Fight::Beaten { boss });
        };
        match self.fight.get() {
            Fight::None if a_midboss && frame == THE_MIDBOSS_ARRIVES_AT => {
                image.a_fight_is_up(A_FIGHT_STANDS_AT, A_FIGHTS_LIFE);
                the_attack(false, 0);
            }
            Fight::None | Fight::Beaten { boss: false } if frame == boss_from => {
                image.the_stream_plays_the_track(THE_BOSSS_TRACK);
                image.a_fight_is_up(A_FIGHT_STANDS_AT, A_FIGHTS_LIFE);
                the_attack(true, 0);
            }
            Fight::Up {
                boss: false,
                attack,
            } => {
                image.the_fight_goes_on();
                if frame == THE_MIDBOSS_ARRIVES_AT + (attack + 1) * AN_ATTACK_LASTS {
                    match attack {
                        0 => the_attack(false, 1),
                        _ => the_fights_over(false),
                    }
                }
            }
            Fight::Up { boss: true, attack } => {
                image.the_fight_goes_on();
                let next = boss_from + (attack + 1) * AN_ATTACK_LASTS;
                if attack < 2 && frame == next {
                    the_attack(true, attack + 1);
                } else if frame + AFTER_THE_BOSS == lasts {
                    the_fights_over(true);
                }
            }
            _ => {}
        }
    }

    /// **A bullet aimed at the player**, from the fight where one is up and from the top of the field otherwise,
    /// into the first slot of this stage's that holds none.
    fn fires_a_bullet(&self) {
        let image = &self.image;
        let Some(slot) = (0..BULLET_SLOTS_USED).find(|slot| !image.the_bullet_is_active(*slot))
        else {
            return;
        };
        let from = match self.fight.get() {
            Fight::Up { .. } => A_FIGHT_STANDS_AT,
            _ => BULLETS_FROM,
        };
        let at = image.where_the_player_is();
        let (dx, dy) = (at.0 - from.0, at.1 - from.1);
        let far = (dx * dx + dy * dy).sqrt().max(1.0);
        image.a_bullet_is_at(
            slot,
            from,
            (dx / far * BULLET_SPEED, dy / far * BULLET_SPEED),
            (BULLET_SIZE, BULLET_SIZE),
        );
    }

    /// **The bullets' calc job** (0x4065c0, priority `0x14`): each active one a step on, freed off the field, and
    /// tested against the player where the frame's own move put it — the two boxes, which 0x4266b0 takes nothing
    /// of while the player is dying, hit or out of reach (0x426735 through 0x426750); a bullet that hits goes. And
    /// the count of those left.
    fn its_bullets(&self) {
        let image = &self.image;
        let mut left = 0;
        let (state, _) = image.what_the_player_is_doing();
        let open = [Player::Playing as i32, Player::ComingBackIn as i32].contains(&state)
            && image.the_frames_nothing_hits_the_player_for() <= 0;
        let player = image.where_the_player_is();
        let half = image.the_players_half_box();
        for slot in 0..BULLET_SLOTS_USED {
            if !image.the_bullet_is_active(slot) {
                continue;
            }
            image.the_bullet_moves(slot);
            let ((x, y), (w, h)) = image.where_the_bullet_is(slot);
            if x.abs() > THE_FIELD_ACROSS || y < THE_FIELD_DOWN.0 || y > THE_FIELD_DOWN.1 {
                image.the_bullet_is_gone(slot);
                continue;
            }
            if open
                && (x - player.0).abs() < w / 2.0 + half
                && (y - player.1).abs() < h / 2.0 + half
            {
                image.the_player_is_in(Player::Hit);
                image.the_bullet_is_gone(slot);
                continue;
            }
            left += 1;
        }
        image.the_bullet_manager_counts(left);
    }

    /// **The stage's end**, which the conversation after its boss carries out: a practice ends its run cleared
    /// (0x423370 from 0x416c4c), the last stage has the panel count towards the ending (0x416c56 through 0x416c5e,
    /// 0x416eef), the Extra ends its run cleared as a practice does (0x416de8 through 0x416e55), and any other asks
    /// for the stage after it (0x416ed5 through 0x416ee8).
    fn the_stage_ends(&self) {
        let image = &self.image;
        let (score, power, lives) = image.the_runs_numbers();
        image.the_run_has(score + A_STAGE_WON_SCORES, power, lives);
        if self.state().practice {
            image.the_run_is_over_cleared();
            return;
        }
        match image.the_stage_word() {
            THE_LAST_STAGE => self.towards_the_ending.set(Some(0)),
            THE_EXTRAS_STAGE => image.the_run_is_over_cleared(),
            _ => self.moves_on_to_the_next_stage(),
        }
    }

    /// **The supervisor's calc job, 0x41ff80**: the keys read (0x41ffad), the stage's build going on beside it,
    /// and the scene switched (0x4218d0).
    fn its_supervisor(&self) {
        let frames = self.frames_since_launch.get() + 1;
        self.frames_since_launch.set(frames);
        // **The loading object's job**, which wants the title once the start-up's files are in (0x41fdbc) — the
        // score object among them, read out of `scoreth10.dat` (0x42af20 through 0x42af57). By the game's own name
        // and not through the fork: a read at startup landing in the game's own file is the bracket's to decide,
        // which other games' e2e tests are about.
        if frames == TITLE_COMES_UP_AFTER && self.image.the_scene_now() == Scene::Loading as i32 {
            let file = self.image.sim().files().get(THE_GAMES_SCORE_FILE);
            self.read_a_score_file.set(file.is_some());
            let own = file
                .and_then(|file| score_file::decode(&file))
                .unwrap_or_default();
            self.image.the_score_object_is_read_from(&own);
            self.image.asks_for_the_scene(Scene::Title);
        }
        let keys = self.the_keyboards_word();
        // **The pad's half, through orb's hook where a launch installs it** — `push 0; push ebx; mov ecx,edi;
        // call 0x44a190` at 0x44a8b3, the keyboard's word in `ecx`, the player and the pad pushed.
        let word = match self.reads_the_keys {
            true => orb_core::runtime::get_input_with_the_keys(u32::from(keys), 0, 0, 0),
            false => pad_half(u32::from(keys), 0, 0, 0),
        };
        self.word.set(word);
        self.image.the_keys_read(word);
        if self.image.the_scene_now() != self.image.the_scene_wanted() {
            self.switches_the_scene();
        }
    }

    /// **The keyboard folded into one word**, the `GetKeyboardState` half of 0x44a5f0 — see [`MAP`].
    fn the_keyboards_word(&self) -> u16 {
        let state = self.sim().keyboard().state();
        MAP.iter()
            .filter(|(key, _)| {
                state
                    .as_ref()
                    .is_some_and(|state| state[*key as usize] & 0x80 != 0)
            })
            .fold(0, |word, (_, bit)| word | bit)
    }

    /// **The scene switch, 0x4218d0**, for the scenes this file builds: the start-up's loading (0x42192b), the
    /// title — the run taken down first where it is from one (0x42197b) — a run, the title taken down and the game
    /// object built with its build started (0x4219f8 through 0x421a14), the stage after the one won, the run taken
    /// down but for what the next stage keeps and 7 wanted (0x421a4e through 0x421a76), and the ending, the run
    /// taken down (0x4219a0 on). Then the scene it is on is the scene it wanted (0x42198b through 0x421992). A scene
    /// this file builds nothing for stands asked for.
    fn switches_the_scene(&self) {
        let now = self.image.the_scene_now();
        let wanted = self.image.the_scene_wanted();
        match wanted {
            w if w == Scene::Startup as i32 => self.image.asks_for_the_scene(Scene::Loading),
            // 0x42194e, 0x4203f0: the run taken down where there is one, the title freed, and the loading
            // object's teardown writing the score object (0x41fc6e from 0x42042e) — the one write of it that is
            // no run's — and the supervisor answering 4, which leaves the frame and the game with it.
            w if w == Scene::TheGameLeaves as i32 => {
                if self.image.the_game_object_is_up() {
                    self.the_run_is_taken_down();
                }
                self.image.the_title_is_freed();
                self.writes_its_score_file();
                self.leaving.set(true);
                return;
            }
            w if w == Scene::Title as i32 => {
                if now == Scene::Run as i32 {
                    self.the_run_is_taken_down();
                }
                self.image.the_title_is_built();
                self.image.the_title_is_on(Screen::Entry);
            }
            w if w == Scene::Run as i32 => {
                if now == Scene::Title as i32 {
                    self.image.the_title_is_freed();
                }
                self.image.the_game_object_is_built_for_a_fresh_run();
                self.the_stage_is_on.set(false);
                self.building.set(Some(A_BUILD_RUNS_FOR));
            }
            w if w == Scene::NextStage as i32 => {
                let replaying = self.image.the_run_plays_a_replay();
                self.the_stage_is_on.set(false);
                // The teardown the switch makes of the game object out of a run (0x418150 at 0x421a66), which
                // writes the score object as every teardown does (0x417c8d).
                self.writes_its_score_file();
                self.image.the_run_is_taken_down_for_the_next_stage();
                self.image
                    .the_game_object_is_built_for_the_next_stage(replaying);
                self.building.set(Some(A_BUILD_RUNS_FOR));
                self.image.asks_for_the_scene(Scene::Run);
            }
            // 0x421af2 through 0x421b02: the run taken down, its score object written with it, and then the ending
            // built, whose set-up writes the clear into the object (0x40b97f).
            w if w == Scene::Ending as i32 => {
                self.the_run_is_taken_down();
                self.image.the_ending_writes_its_clear();
                self.the_ending_has_run_for.set(0);
            }
            // 0x4219bb through 0x4219f6: the ending freed, the title wanted and built to come up on the screen
            // after the ending.
            w if w == Scene::AfterTheEnding as i32 => {
                self.image.asks_for_the_scene(Scene::Title);
                self.image.the_title_is_to_come_up_after_the_ending();
                self.image.the_title_is_built();
                self.image.the_title_is_on(Screen::Entry);
            }
            _ => return,
        }
        self.image.the_switch_is_on_the_scene_it_wanted();
    }

    /// **The game object's teardown, 0x417c80**, which writes the score object first (0x417c8d).
    fn the_run_is_taken_down(&self) {
        self.writes_its_score_file();
        self.the_stage_is_on.set(false);
        self.building.set(None);
        self.image.the_run_is_taken_down();
    }

    /// **The stage's build, 0x417870, on its thread**: nothing a frame of it the main thread can see but the game
    /// object's flag and the stage's frames at nought, and on its last the whole of it — the defaults for a fresh
    /// run (0x4178ab through 0x4179e2), the replay's object for the stage through its seam (0x417a86), the stage's
    /// objects (0x417aab through 0x417ba2) and the flag off (0x417c30).
    ///
    /// **A frame of it goes by between two of the main thread's frames**, as the present hands one over: the walk
    /// after the build's last is the first to find the game object's job on the chain (0x409e20 at 0x417c53), so
    /// the top of that frame is one where the build is over and the stage has no frame yet — which is what
    /// `Th10::a_stage_has_just_been_built` is read off on the real game. Ended inside a walk, the game object's
    /// job ran on the walk that ended it, and no frame stood between the two.
    fn its_loader_goes_on(&self) {
        let Some(left) = self.building.get() else {
            return;
        };
        if left > 1 {
            self.building.set(Some(left - 1));
            return;
        }
        self.building.set(None);
        self.fight.set(Fight::None);
        // **The stage after the one won**, whose numbers are carried and whose replay goes through 0x42a6a0
        // rather than the seam (0x417afb) — the player and the rest kept, the enemies, the card and the bomb made
        // again.
        if !self.image.the_run_is_fresh() {
            // 0x42a6a0's recording: the seed into a new header and the draws from nought (0x42a6dd through
            // 0x42a6ef).
            self.image.the_generators_draws_start_over();
            self.image.the_next_stages_objects_are_made();
            self.loads_the_stages_songs();
            self.image.the_build_is_over();
            return;
        }
        self.image.the_builds_defaults_are_in();
        let replaying = u32::from(self.image.the_run_plays_a_replay());
        match the_game_this_is().hooks().stage_building.is_some() {
            true => orb_core::runtime::stage_building_of_two(replaying, THE_REPLAYS_RECORD),
            false => replay_built(replaying, THE_REPLAYS_RECORD),
        };
        self.image.the_stages_objects_are_made();
        self.loads_the_stages_songs();
        self.image.the_build_is_over();
    }

    /// **The stage's two songs loaded after its objects**, either way the stage was built, and not for the attract
    /// demo (0x417ba6 through 0x417bd1): the stage's own and the fight's, which every stage here plays.
    fn loads_the_stages_songs(&self) {
        if !self.image.the_run_is_the_demo() {
            self.image
                .the_stages_songs_are_loaded(THE_STAGES_TRACK, THE_BOSSS_TRACK);
        }
    }

    /// **The game object's calc job, 0x418190**, once the build has turned it on (0x417c53): its frame 0 begins
    /// the stage and turns the stage's jobs on (0x41821c through 0x4183ca), which run behind it on the same walk;
    /// a pause stops the walk here, answering false; and every frame past it steps the stage's clocks (0x418769
    /// through 0x418784).
    fn its_game_object(&self) -> bool {
        let image = &self.image;
        match image.the_games_frames() {
            // A stage after one won fades the stage before out first (0x4181f2 through 0x418217).
            0 if image.the_stage_before_is_kept() => image.the_stage_before_fades(true),
            0 if !self.the_stage_is_on.get() => {
                self.begins_the_stage();
                self.the_shot_waits.set(true);
            }
            A_FADE_BEGINS_THE_STAGE_AT if image.the_stage_before_fades_out() => {
                image.the_stage_before_fades(false);
                self.begins_the_stage();
                image.the_games_timer_starts_over();
                image.the_stage_before_is_freed();
            }
            _ => {}
        }
        if image.the_game_stands_paused() {
            return false;
        }
        image.a_frame_of_the_stage_goes_by();
        true
    }

    /// **The stage begun** (0x41821c through 0x4183e1): its frames at nought, the player reset, the items zeroed,
    /// `main` spawned, the options made for the power (0x4182e0), the stage's jobs on, and its own song started.
    fn begins_the_stage(&self) {
        self.image.the_stage_begins();
        // The player's reset, 0x424d90, which a stage begun either way runs: its shot held (0x426301).
        self.image.the_shot_timer_is(-1);
        the_options_are_made(self.image.the_player_object());
        self.the_stage_is_on.set(true);
        self.fight.set(Fight::None);
        self.image.the_stream_plays_the_track(THE_STAGES_TRACK);
    }

    /// **The pause object's calc job, 0x4223f0**, its states by the tables at 0x4224b0 and 0x42438c — see
    /// [`Pause`] — and its timer one frame on at the end.
    fn its_pause_object(&self) {
        let image = &self.image;
        let (state, frames, cursor) = image.where_the_pause_is();
        let pressed = image.the_buttons_gone_down();
        match state {
            s if s == Pause::Nothing as i32 => {
                if pressed & THE_PAUSE != 0
                    && !image.the_run_is_the_demo()
                    && image.the_games_frames() >= A_PAUSE_OPENS_PAST
                {
                    // 0x422ab0: the state, the game object's flag, the music's command and the speed held back.
                    image.the_pause_is_in(Pause::Opening);
                    image.the_game_is_paused(true);
                    image.a_sound_command_is_queued(THE_PAUSES_COMMAND);
                    image.the_pause_holds_the_speed_back();
                }
            }
            s if s == Pause::Opening as i32 => {
                if frames >= THE_PAUSE_OPENS_OVER {
                    image.the_pause_is_in(Pause::Menu);
                    let items = match image.the_run_plays_a_replay() {
                        true => A_REPLAYS_PAUSE_ITEMS,
                        false => THE_PAUSES_ITEMS,
                    };
                    image.the_pauses_cursor_is(THE_PAUSES_RESUME, items);
                }
            }
            s if s == Pause::Menu as i32 => {
                let items = image.the_pauses_items();
                let closing_on = if pressed & THE_PAUSES_RETRY != 0 {
                    Some(THE_PAUSES_WAY_TO_A_RETRY.min(items - 1))
                } else if pressed & THE_PAUSES_TITLE != 0 {
                    Some(THE_PAUSES_WAY_TO_THE_TITLE.min(items - 1))
                } else if pressed & THE_PAUSE != 0 {
                    Some(THE_PAUSES_RESUME)
                } else if pressed & DECIDE != 0 {
                    match cursor {
                        THE_PAUSES_RESUME => Some(THE_PAUSES_RESUME),
                        // The confirmation the other two ask for, which this file does not play.
                        _ => {
                            image.the_pause_is_in(Pause::Confirming);
                            None
                        }
                    }
                } else {
                    None
                };
                if let Some(item) = closing_on {
                    image.the_pause_is_in(Pause::Closing);
                    image.the_pauses_cursor_is(item, items);
                }
            }
            s if s == Pause::Closing as i32 => {
                if frames == THE_PAUSE_CLOSES_OVER {
                    image.the_pause_is_in(Pause::Nothing);
                    match cursor {
                        THE_PAUSES_RESUME => the_run_is_unpaused(image.the_pause_object()),
                        THE_PAUSES_WAY_TO_THE_TITLE => image.asks_for_the_scene(Scene::Title),
                        _ => image.asks_for_the_scene(Scene::StartOver),
                    }
                }
            }
            s if s == Pause::TheRunIsOver as i32 => {
                if frames > THE_END_READS_PAST && pressed & DECIDE != 0 {
                    image.the_pause_is_in(Pause::Tallying);
                }
            }
            // 0x42377e through 0x42388e: a practice's score kept and the menu; any other run's score placed
            // (0x423570, a cleared Extra moved past first) and the menu where it ranks nowhere, the ranking's name
            // entry where it does. The menu's three items with the cursor on the last.
            s if s == Pause::Tallying as i32 => {
                if frames == THE_END_TALLIES_FOR {
                    let practice = self.state().practice;
                    if !practice {
                        image.a_cleared_extra_is_moved_past();
                        image.the_pause_places_the_run(THE_GRID.len() as i32);
                    }
                    if practice || image.the_run_is_out_of_the_ranking() {
                        image.the_pause_is_in(Pause::TheEndsMenu);
                        image.the_pauses_cursor_is(THE_ENDS_LAST_ITEM, THE_ENDS_ITEMS);
                    } else {
                        image.the_pause_is_in(Pause::NamingTheRanking);
                    }
                }
            }
            s if s == Pause::NamingTheRanking as i32 && frames >= THE_END_READS_PAST => {
                self.names_the_ranking(pressed);
            }
            s if s == Pause::TheEndsMenu as i32 => {
                if pressed & DECIDE != 0 {
                    match cursor {
                        THE_ENDS_SAVE => image.the_pause_is_in(Pause::SavingAReplay),
                        _ => image.the_pause_is_in(Pause::Leaving),
                    }
                } else if pressed & CANCEL != 0 {
                    image.the_pauses_cursor_is(THE_ENDS_LAST_ITEM, THE_ENDS_ITEMS);
                }
            }
            s if s == Pause::Leaving as i32 && frames == THE_END_LEAVES_AFTER => {
                match (cursor, image.the_run_was_cleared()) {
                    // Item 0 plays on, which is scenes this file does not build.
                    (0, true) => image.asks_for_the_scene(Scene::StartOver),
                    (0, false) => {}
                    _ => image.asks_for_the_scene(Scene::Title),
                }
            }
            _ => {}
        }
        image.the_pause_counts_a_frame();
    }

    /// **The player's calc job, 0x425730**, its states by the switch at 0x425739 — see [`Player`] — and its
    /// timers one frame on at the end (0x42618e, 0x425e46).
    fn its_player(&self) {
        let image = &self.image;
        let (state, frames) = image.what_the_player_is_doing();
        match state {
            s if s == Player::ComingBackIn as i32 => {
                image.the_player_comes_up_to(frames);
                if frames == COMING_BACK_IN_FOR {
                    image.the_player_is_in(Player::Playing);
                }
            }
            s if s == Player::Playing as i32 => self.moves_its_player(),
            // A bomb would take the hit back inside these frames, which this file does not play.
            s if s == Player::Hit as i32 => {
                if frames == A_HIT_WAITS_FOR {
                    image.the_player_dies();
                }
            }
            s if s == Player::Dying as i32 && frames == DYING_FOR => {
                let (_, _, lives) = image.the_runs_numbers();
                match lives < 0 {
                    true => image.the_run_is_over_with_a_game_over(),
                    false => image.the_player_comes_back_in(),
                }
            }
            _ => {}
        }
        if !self.the_shot_waits.replace(false) {
            image.the_players_shot_goes_on();
        }
        image.the_player_counts_a_frame();
    }

    /// **The player moved by the run's copy of the word**, 0x4250b0: the direction out of its four bits — two of
    /// them a diagonal (0x4250c0 through 0x425163) — the speed of that direction, focused where bit `0x4` is down,
    /// and the step times the game's speed truncated to hundredths (0x425404 through 0x42541f).
    fn moves_its_player(&self) {
        let image = &self.image;
        let word = image.the_run_word();
        let across = i32::from(word & TO_THE_RIGHT != 0) - i32::from(word & TO_THE_LEFT != 0);
        let down = i32::from(word & DOWN != 0) - i32::from(word & UP != 0);
        if across == 0 && down == 0 {
            return;
        }
        let [straight, focused, diagonal, focused_diagonal] = image.the_players_speeds();
        let speed = match (word & FOCUS != 0, across != 0 && down != 0) {
            (false, false) => straight,
            (true, false) => focused,
            (false, true) => diagonal,
            (true, true) => focused_diagonal,
        };
        let step = (speed as f32 * image.the_games_speed()) as i32;
        image.the_player_steps_by(across * step, down * step);
    }

    /// **The title's calc job, 0x42cdf0**: the screen up by the switch at 0x42ce37, and its frames one on at its
    /// tail (0x42d1dc).
    fn walks_its_title(&self) {
        let image = &self.image;
        let (screen, step, cursor, frames) = image.where_the_title_is();
        let pressed = image.the_buttons_gone_down();
        let (_, items) = image.the_titles_cursor();
        let left_out = image.the_items_the_title_leaves_out();
        let moved = |by: i32| moved(cursor, by, items, &left_out);
        match screen {
            s if s == Screen::Entry as i32 => {
                // 0x42cf90 through 0x42cff2: a title after the ending up on the screen after the ending, and
                // every title after it on the main menu.
                if image.the_title_comes_up_after_the_ending() {
                    image.the_title_is_on(Screen::AfterTheEnding);
                    image.the_title_has_been_passed();
                } else if image.the_title_comes_up_on_its_main_menu() {
                    image.the_title_is_on(Screen::MainMenu);
                } else {
                    image.the_title_has_been_passed();
                    image.the_title_is_on(Screen::PressAKey);
                }
            }
            s if s == Screen::PressAKey as i32 => match step {
                STEP_BUILT => image.the_titles_step_is(STEP_WAITING),
                STEP_WAITING if frames > A_KEY_IS_READ_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING if pressed & ANY_KEY != 0 => image.the_titles_step_is(STEP_LEFT),
                STEP_LEFT if frames >= THE_KEY_ACTS_AT => image.the_title_is_on(Screen::MainMenu),
                _ => {}
            },
            s if s == Screen::MainMenu as i32 => match step {
                STEP_BUILT => {
                    self.the_main_menu_is_built();
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_KEY_IS_READ_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => {
                    if pressed & UP != 0 {
                        image.the_titles_cursor_is(moved(-1), items);
                    }
                    if pressed & DOWN != 0 {
                        image.the_titles_cursor_is(moved(1), items);
                    }
                    // A cancel on the last item takes it as a decide does (0x42d5a5 through 0x42d718).
                    let (cursor, _) = image.the_titles_cursor();
                    if pressed & CANCEL != 0 && cursor == THE_LAST_ITEM {
                        image.the_titles_step_is(STEP_LEFT);
                    } else if pressed & CANCEL != 0 {
                        image.the_titles_cursor_is(THE_LAST_ITEM, items);
                    } else if pressed & DECIDE != 0 {
                        image.the_titles_step_is(STEP_LEFT);
                    }
                }
                // 0x42d757 through 0x42d7cd: the Extra — the run's flag for a practice off, the difficulty, its
                // cursor's push, the difficulty before it kept at the title's `+0x58f0` and the difficulty put at 4,
                // and the cursor on the first item. What is kept is read only by the difficulty select's cancel
                // (0x430660), which this file does not play, so it is not laid.
                STEP_LEFT
                    if frames >= THE_MAIN_MENU_ACTS_AT && cursor == THE_ITEM_FOR_THE_EXTRA =>
                {
                    image.the_run_is_no_practice();
                    image.the_title_is_on(Screen::Difficulty);
                    image.the_title_leaves_nothing_out();
                    image.the_difficulty_is(DIFFICULTY_EXTRA);
                    image.the_titles_cursor_is(0, items);
                }
                // 0x42d8b6: the last item, the title on its way out — see `Screen::Leaving` below.
                STEP_LEFT if frames >= THE_MAIN_MENU_ACTS_AT && cursor == THE_LAST_ITEM => {
                    image.the_title_is_on(Screen::Leaving)
                }
                // **Only a run, the Extra, a practice and the way out are played of what the menu's items open**
                // — see the module doc.
                STEP_LEFT
                    if frames >= THE_MAIN_MENU_ACTS_AT
                        && [THE_ITEM_FOR_A_RUN, THE_ITEM_FOR_A_PRACTICE].contains(&cursor) =>
                {
                    match cursor == THE_ITEM_FOR_A_PRACTICE {
                        true => image.the_run_becomes_a_practice(),
                        false => image.the_run_is_no_practice(),
                    }
                    image.the_title_is_on(Screen::Difficulty);
                    // The cursor's push (0x42d7f5), which starts the list of items left out over (0x44be4b).
                    image.the_title_leaves_nothing_out();
                    // The Extra's difficulty a run before left put back to Normal (0x42d7fc through 0x42d80c).
                    if image.the_difficulty() >= DIFFICULTY_EXTRA {
                        image.the_difficulty_is(DIFFICULTY_NORMAL);
                    }
                    // 0x40ad20 at 0x42d814: the difficulty under the cursor, held to the count still standing.
                    image.the_titles_cursor_is(image.the_difficulty().clamp(0, items - 1), items);
                }
                _ => {}
            },
            // Over the Extra one item, which the arrows do not move over (0x430394 through 0x4303ab, 0x430423),
            // and the difficulty left at the Extra's as the select moves on (0x4305bf through 0x4305cb).
            s if s == Screen::Difficulty as i32 => match step {
                STEP_BUILT => {
                    let count = match image.the_difficulty() >= DIFFICULTY_EXTRA {
                        true => 1,
                        false => DIFFICULTIES,
                    };
                    image.the_titles_cursor_is(cursor, count);
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING if image.the_difficulty() >= DIFFICULTY_EXTRA => {
                    self.a_select_reads(pressed, 0, 0, moved)
                }
                STEP_READING => self.a_select_reads(pressed, UP, DOWN, moved),
                STEP_DECIDED if frames >= A_SELECT_MOVES_ON_AT => {
                    image.the_title_is_on(Screen::Character);
                    if image.the_difficulty() < DIFFICULTY_EXTRA {
                        image.the_difficulty_is(cursor);
                    }
                    image.the_title_leaves_nothing_out();
                    image.the_titles_cursor_is(
                        image.the_character().clamp(0, CHARACTERS - 1),
                        CHARACTERS,
                    );
                }
                _ => {}
            },
            s if s == Screen::Character as i32 => match step {
                STEP_BUILT => {
                    image.the_titles_cursor_is(cursor, CHARACTERS);
                    if image.the_difficulty() == DIFFICULTY_EXTRA {
                        self.leaves_out_what_the_extra_is_not_open_for(CHARACTERS, |character| {
                            (0..SHOTS)
                                .any(|shot| image.the_extra_is_open_for(character * SHOTS + shot))
                        });
                    }
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => self.a_select_reads(pressed, TO_THE_LEFT, TO_THE_RIGHT, moved),
                STEP_DECIDED if frames >= A_SELECT_MOVES_ON_AT => {
                    let before = image.the_character();
                    image.the_title_is_on(Screen::Shot);
                    image.the_character_is(cursor);
                    image.the_title_leaves_nothing_out();
                    // A character changed starts the shot's cursor at nought (0x4309a2 through 0x4309d8).
                    let shot = match before == cursor {
                        true => image.the_shot().clamp(0, SHOTS - 1),
                        false => 0,
                    };
                    image.the_titles_cursor_is(shot, SHOTS);
                }
                _ => {}
            },
            s if s == Screen::Shot as i32 => match step {
                STEP_BUILT => {
                    image.the_titles_cursor_is(cursor, SHOTS);
                    if image.the_difficulty() == DIFFICULTY_EXTRA {
                        let character = image.the_character();
                        self.leaves_out_what_the_extra_is_not_open_for(SHOTS, |shot| {
                            image.the_extra_is_open_for(character * SHOTS + shot)
                        });
                    }
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => self.a_select_reads(pressed, UP, DOWN, moved),
                // 0x430eba through 0x430ede: a practice's shot, and its stage select.
                STEP_DECIDED if frames == A_PRACTICE_IS_CHOSEN_AT && self.state().practice => {
                    image.the_shot_is(cursor);
                    image.the_title_is_on(Screen::Practice);
                }
                // 0x430ef0 through 0x430f5a: the shot, the title on its way out, stage 1 or the Extra's by the
                // difficulty, and the run wanted.
                STEP_DECIDED if frames == THE_SHOT_STARTS_THE_RUN_AT => {
                    image.the_shot_is(cursor);
                    image.the_title_is_on(Screen::Leaving);
                    match image.the_difficulty() >= DIFFICULTY_EXTRA {
                        true => image.the_run_starts_at_the_extra(),
                        false => image.the_run_starts_at_stage_1(),
                    }
                    image.asks_for_the_scene(Scene::Run);
                }
                _ => {}
            },
            s if s == Screen::Practice as i32 => match step {
                // 0x43100b through 0x43105b: six stages, the cursor where the select last left it.
                STEP_BUILT => {
                    let kept = image.the_practice_select_remembers();
                    image.the_titles_cursor_is(kept.clamp(0, PRACTICE_STAGES - 1), PRACTICE_STAGES);
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > THE_PRACTICE_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => {
                    if pressed & UP != 0 {
                        image.the_titles_cursor_is(moved(-1), items);
                    }
                    if pressed & DOWN != 0 {
                        image.the_titles_cursor_is(moved(1), items);
                    }
                    let (cursor, _) = image.the_titles_cursor();
                    // A cancel back towards the shot (0x4310e0 through 0x43110b), and a decide that takes only a
                    // stage the shot type's record offers (0x431119 through 0x43116a).
                    if pressed & CANCEL != 0 {
                        image.the_titles_step_is(STEP_LEFT);
                        image.the_practice_select_keeps(cursor);
                    } else if pressed & DECIDE != 0
                        && image.the_practice_is_offered(image.the_difficulty(), cursor)
                    {
                        image.the_titles_step_is(STEP_DECIDED);
                        image.the_practice_select_keeps(cursor);
                    }
                }
                // 0x431352 through 0x4313a1: the title on its way out, the stage under the cursor and the run
                // wanted.
                STEP_DECIDED if frames >= A_PRACTICE_STARTS_AT => {
                    image.the_title_is_on(Screen::Leaving);
                    image.the_practice_starts_at(cursor + 1);
                    image.asks_for_the_scene(Scene::Run);
                }
                // 0x4313b4 through 0x4313e6: back to the shot, its cursor on the shot it had.
                STEP_LEFT if frames >= A_PRACTICE_CANCELS_AT => {
                    image.the_title_is_on(Screen::Shot);
                    image.the_titles_cursor_is(image.the_shot().clamp(0, SHOTS - 1), SHOTS);
                }
                _ => {}
            },
            // 0x42d14d through 0x42d15d: the game's leaving asked for, scene 3 — or 2 where bit `0x1000` of
            // `[0x491ff4]` stands, which the loading object takes off (0x41fdb2, 0x41fed2) and this file never sets.
            // A run chosen leaves through this screen too, but the switch builds the run and frees the title before
            // the title's job runs on it again (0x421a03).
            s if s == Screen::Leaving as i32 => image.asks_for_the_scene(Scene::TheGameLeaves),
            s if s == Screen::AfterTheEnding as i32 => self.names_the_clear(step, frames, pressed),
            s if s == Screen::TheReplaysSlots as i32 => match step {
                // 0x433599 through 0x43362a: the twenty-five slots, the cursor on the first.
                STEP_BUILT => {
                    image.the_titles_cursor_is(0, THE_REPLAYS_SLOTS);
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > THE_SLOTS_READ_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                // 0x43364d through 0x4336cc: the arrows over the slots, and a cancel to step 4. A decide's step 3,
                // the replay's own name entry over the slot taken, is not played.
                STEP_READING => {
                    if pressed & UP != 0 {
                        image.the_titles_cursor_is(moved(-1), items);
                    }
                    if pressed & DOWN != 0 {
                        image.the_titles_cursor_is(moved(1), items);
                    }
                    if pressed & CANCEL != 0 {
                        image.the_titles_step_is(STEP_LEFT);
                    }
                }
                // 0x433a33 through 0x433a8d: the main menu, its cursor put on its first item — what the pop at
                // 0x433a8d puts back not having been read, nor which push it is.
                STEP_LEFT if frames >= A_CANCEL_LEAVES_THE_SLOTS_AT => {
                    image.the_titles_cursor_is(THE_ITEM_FOR_A_RUN, MAIN_MENU_ITEMS);
                    image.the_title_is_on(Screen::MainMenu);
                }
                _ => {}
            },
            _ => {}
        }
        image.the_title_counts_a_frame();
    }

    /// **The title's name entry for a clear, 0x432cb0**, by its steps (the table at 0x433218).
    fn names_the_clear(&self, step: i32, frames: i32, pressed: u16) {
        let image = &self.image;
        let the_end = THE_GRID.len() as i32 - 1;
        let mut entry = image.the_titles_entry();
        match step {
            // 0x432ccc through 0x432ed0: the run's score placed in the ranking, and where it has a place the name
            // started from the last one typed, the grid's cursor on the end where that was a name at all
            // (0x432e6a through 0x432e7e), and the place in it after its last character that is not a space
            // (0x432e83 through 0x432e9e). Where it has none, nothing is asked.
            STEP_BUILT => {
                image.the_titles_step_is(STEP_WAITING);
                image.the_titles_grid_is_this_long(THE_GRID.len() as i32);
                match image.the_ranking_places_the_run() {
                    Some(place) => {
                        let (_, items) = image.the_titles_cursor();
                        image.the_titles_cursor_is(place as i32, items);
                        let name = image.the_last_name_typed();
                        entry = TheTitlesEntry {
                            name,
                            typed: name.iter().rposition(|c| *c != b' ').map_or(0, |at| at + 1)
                                as i32,
                            not_asked: false,
                            grid: if name == A_BLANK_NAME { 0 } else { the_end },
                        };
                    }
                    None => entry.not_asked = true,
                }
                image.the_titles_entry_is(&entry);
            }
            STEP_WAITING if frames > THE_ENTRY_READS_PAST => image.the_titles_step_is(STEP_READING),
            STEP_READING => {
                // 0x432f13 through 0x432fb3 — see [`moved_over_the_grid`].
                if !entry.not_asked {
                    entry.grid = moved_over_the_grid(entry.grid, pressed);
                }
                if pressed & DECIDE != 0 {
                    // 0x432fe0 and 0x43310e: a clear not asked about decided straight past.
                    if entry.not_asked {
                        image.the_titles_step_is(STEP_DECIDED);
                        return;
                    }
                    let place = entry.typed as usize;
                    match entry.grid {
                        // 0x43301c through 0x43306e, and the space at 0x433042 — the grid's own cell for it being
                        // a space too: a character at the place, the place on, and the grid's cursor on the end
                        // once the name is full; the last character over where it already is.
                        at if at < the_end - 1 => {
                            let character = THE_GRID[at as usize];
                            if place < A_BLANK_NAME.len() {
                                entry.name[place] = character;
                                entry.typed += 1;
                                if entry.typed as usize >= A_BLANK_NAME.len() {
                                    entry.grid = the_end;
                                }
                            } else {
                                entry.name[A_BLANK_NAME.len() - 1] = character;
                            }
                        }
                        // 0x433087 through 0x4330aa: a step back over the last character.
                        at if at == the_end - 1 => {
                            if place > 0 {
                                entry.typed -= 1;
                                entry.name[place - 1] = b' ';
                            }
                        }
                        // 0x4330b1 through 0x433115: the name into the ranked place and the last name typed, and
                        // the entry's last step.
                        _ => {
                            let (ranked, _) = image.the_titles_cursor();
                            image.the_ranked_place_is_named(ranked as usize, &entry.name);
                            image.the_last_name_typed_is(&entry.name);
                            image.the_titles_entry_is(&entry);
                            image.the_titles_step_is(STEP_DECIDED);
                            return;
                        }
                    }
                } else if pressed & CANCEL != 0 {
                    // 0x433138 through 0x433189: a step back, or straight past where nothing was asked.
                    if entry.not_asked {
                        image.the_titles_step_is(STEP_DECIDED);
                        return;
                    }
                    if entry.typed > 0 {
                        entry.typed -= 1;
                        entry.name[entry.typed as usize] = b' ';
                    }
                }
                image.the_titles_entry_is(&entry);
            }
            // 0x4331ac through 0x433204: the replay's slots.
            STEP_DECIDED if frames >= THE_ENTRY_LEAVES_AT => {
                image.the_title_is_on(Screen::TheReplaysSlots)
            }
            _ => {}
        }
    }

    /// **A select over the Extra leaving out the items nobody has opened it for**, the character select's step 0
    /// over a character's three bytes (0x4306c2 through 0x430776) and the shot select's over the shot's own
    /// (0x430a82 through 0x430bf6): each item the Extra is not open for put on the list left out, and the cursor
    /// on it moved to the first of the others the Extra is open for — or the last of the others where none is.
    fn leaves_out_what_the_extra_is_not_open_for(&self, items: i32, open: impl Fn(i32) -> bool) {
        let image = &self.image;
        for item in (0..items).filter(|item| !open(*item)) {
            let (cursor, count) = image.the_titles_cursor();
            if cursor == item {
                let others: Vec<i32> = (0..items).filter(|other| *other != item).collect();
                let to = others
                    .iter()
                    .copied()
                    .find(|other| open(*other))
                    .or_else(|| others.last().copied())
                    .unwrap_or(item);
                image.the_titles_cursor_is(to, count);
            }
            image.the_title_leaves_out(item);
        }
    }

    /// **The ranking's name entry, state 12 of the end of a run, past its ten frames** (0x423f93 through
    /// 0x4242ca), over the grid the title's entry walks (`[0x4746d8]` at 0x42408d): the arrows (0x423fb0 through
    /// 0x424057); a decide on a character or the space putting it at the place and the place on — the grid's cursor
    /// on the end once the name is full (0x424107), the last character written over where it already is
    /// (0x4240cb) — on the step back taking the last one off (0x424121 through 0x42413d), and on the end the name
    /// copied into the ranked place under the pause's cursor and into the last one typed, and the menu put up with
    /// its cursor on its first item (0x424153 through 0x4241ec); and a cancel taking the last character off
    /// (0x42427a through 0x4242b0).
    fn names_the_ranking(&self, pressed: u16) {
        let image = &self.image;
        let the_end = THE_GRID.len() as i32 - 1;
        let mut entry = image.the_pauses_entry();
        entry.grid = moved_over_the_grid(entry.grid, pressed);
        if pressed & DECIDE != 0 {
            let place = entry.typed as usize;
            match entry.grid {
                at if at < the_end - 1 => {
                    let character = THE_GRID[at as usize];
                    if place < A_BLANK_NAME.len() {
                        entry.name[place] = character;
                        entry.typed += 1;
                        if entry.typed as usize >= A_BLANK_NAME.len() {
                            entry.grid = the_end;
                        }
                    } else {
                        entry.name[A_BLANK_NAME.len() - 1] = character;
                    }
                }
                at if at == the_end - 1 => {
                    if place > 0 {
                        entry.typed -= 1;
                        entry.name[place - 1] = b' ';
                    }
                }
                _ => {
                    let (_, _, ranked) = image.where_the_pause_is();
                    image.the_ranked_place_is_named(ranked.max(0) as usize, &entry.name);
                    image.the_last_name_typed_is(&entry.name);
                    image.the_pauses_entry_is(&entry);
                    image.the_pause_is_in(Pause::TheEndsMenu);
                    image.the_pauses_cursor_is(0, THE_ENDS_ITEMS);
                    return;
                }
            }
        } else if pressed & CANCEL != 0 && entry.typed > 0 {
            entry.typed -= 1;
            entry.name[entry.typed as usize] = b' ';
        }
        image.the_pauses_entry_is(&entry);
    }

    /// **A select reading its keys**: its two directions moving the cursor, and a decide putting it in its step 3
    /// (0x43056b, 0x4308f3, 0x430e2a) — a cancel's step 4 not being played.
    fn a_select_reads(&self, pressed: u16, back: u16, on: u16, moved: impl Fn(i32) -> i32) {
        let image = &self.image;
        let (_, items) = image.the_titles_cursor();
        if pressed & back != 0 {
            image.the_titles_cursor_is(moved(-1), items);
        }
        if pressed & on != 0 {
            image.the_titles_cursor_is(moved(1), items);
        }
        if pressed & DECIDE != 0 {
            image.the_titles_step_is(STEP_DECIDED);
        }
    }

    /// **The main menu's step 0** (0x42d436 through 0x42d48e): its eight items, the Extra left out where nobody
    /// has opened it, and a menu come back to from a practice put on the practice's item with the run's flag
    /// for one taken off.
    fn the_main_menu_is_built(&self) {
        let image = &self.image;
        let (cursor, _) = image.the_titles_cursor();
        image.the_titles_cursor_is(cursor, MAIN_MENU_ITEMS);
        image.the_title_leaves_nothing_out();
        if !image.the_extra_is_open() {
            image.the_title_leaves_out(THE_ITEM_FOR_THE_EXTRA);
        }
        if self.state().practice {
            image.the_titles_cursor_is(THE_ITEM_FOR_A_PRACTICE, MAIN_MENU_ITEMS);
            image.the_run_is_no_practice();
        }
    }

    /// **The draw walk, 0x449d40**: one quad queued, which is what drawing anything in this game is, and the count
    /// of the jobs it reached.
    fn walks_its_draw_list(&self) -> i32 {
        self.image.queues_a_quad();
        1
    }
}

/// **A cursor moved**, 0x44bea0 with the wrap its struct's `+0xd0` holds, which the title's constructor sets
/// (0x42cb49): round past either end, and over the items the screen leaves out.
fn moved(cursor: i32, by: i32, items: i32, left_out: &[i32]) -> i32 {
    if items <= 0 {
        return cursor;
    }
    let mut at = cursor;
    for _ in 0..items {
        at = (at + by).rem_euclid(items);
        if !left_out.contains(&at) {
            return at;
        }
    }
    cursor
}

/// **The cursor of a name entry's grid moved by the arrows**, the title's (0x432f13 through 0x432fb3) and the end of
/// a run's (0x423fb0 through 0x424057) alike: up and down a row of [`A_GRIDS_ROW`], left and right a cell round the
/// row's ends, and round the grid past either of its ends.
fn moved_over_the_grid(at: i32, pressed: u16) -> i32 {
    let by = |pressed_it: bool, by: i32| if pressed_it { by } else { 0 };
    let across = match at % A_GRIDS_ROW {
        0 if pressed & TO_THE_LEFT != 0 => A_GRIDS_ROW - 1,
        _ if pressed & TO_THE_LEFT != 0 => -1,
        r if r == A_GRIDS_ROW - 1 && pressed & TO_THE_RIGHT != 0 => 1 - A_GRIDS_ROW,
        _ if pressed & TO_THE_RIGHT != 0 => 1,
        _ => 0,
    };
    let moved =
        at - by(pressed & UP != 0, A_GRIDS_ROW) + by(pressed & DOWN != 0, A_GRIDS_ROW) + across;
    moved.rem_euclid(THE_GRID.len() as i32)
}

impl Drop for Fake {
    /// **The game closing, which takes orb down with it** — the runtime first, then the fields, as every other
    /// game's `Fake` closes.
    fn drop(&mut self) {
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The exe this game is, which is how `orb_core::game::KNOWN` is asked for it.
pub const EXE: &str = "th10.exe";

/// **The score file's name the game opens**, `0x46e848`, and the name orb's fork of it is — this file's own
/// literals, as every name here is.
pub(crate) const THE_GAMES_SCORE_FILE: &str = "scoreth10.dat";
const THE_MODES_SCORE_FILE: &str = "pointdevice_scoreth10.dat";

/// **The walk from the title to a run of the mode, as a recipe**, which is the real recipes' own — see
/// `orb-real/recipes/th10/a_run_started_and_given_up.yaml` for why each step is shaped so — up to the shot
/// select's decide that asks for a Normal run of Reimu A.
///
/// **Here rather than in one of the e2e modules that plays it**, because it is this game's screens and every
/// module over a run of it starts by walking them.
pub(crate) const TO_THE_SHOT: &str = "\
- press: [decide]
  frames: 10
  until: { screen: 2 }
  give_up_after: 3600
- press: [up]
  frames: 10
  until: { screen: 2, cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { question: mode }
  give_up_after: 900
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 6 }
  give_up_after: 1800
- press: [up]
  frames: 10
  until: { screen: 6, cursor: 0 }
  give_up_after: 900
- press: [down]
  frames: 10
  until: { screen: 6, cursor: 1 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { screen: 7 }
  give_up_after: 900
- press: [up]
  frames: 10
  until: { screen: 7, cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { screen: 8 }
  give_up_after: 900
- press: [up]
  frames: 10
  until: { screen: 8, cursor: 0 }
  give_up_after: 900
- bombs: false
";

/// **And the shot select's decide that asks for the run, played up to the run's first frame**, which is the
/// end of every real recipe's walk.
pub(crate) const TO_A_RUN: &str = "\
- press: [decide]
  frames: 10
  until: { stage: 1, chapter: 0 }
  give_up_after: 900
";

fn the_game_this_is() -> &'static dyn orb_core::game::Game {
    orb_core::game::found(EXE)
        .unwrap_or_else(|| panic!("{EXE} is a game orb knows"))
        .game
}

/// **The pushing calls a launch of this game leaves as the originals of its two walks and its frame** —
/// `orb_core::runtime::calls_the_calc_chain_on_the_stack` and its two neighbours — in the type `Originals` holds
/// an original in, which asks nothing of its caller: what they ask is that the three slots hold this game's own,
/// and [`Fake::attach_configured`] fills those before anything can call one.
extern "fastcall" fn calls_the_calc_walk_on_the_stack(chain: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::calls_the_calc_chain_on_the_stack(chain) }
}

extern "fastcall" fn calls_the_draw_walk_on_the_stack(chain: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::calls_the_draw_chain_on_the_stack(chain) }
}

extern "fastcall" fn calls_the_frame_on_the_stack(window: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::calls_the_frame_on_the_stack(window) }
}

/// `0x449c00`: the calc walk, `stdcall` on the chain.
extern "stdcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.walks_its_calc_list()
}

/// `0x449d40`: the draw walk, `stdcall` on the chain.
extern "stdcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.walks_its_draw_list()
}

/// **The game's own frame, 0x439390**, `stdcall` on the window — `asm.md`'s *The frame*, in its order and less the
/// limiter and the frame skip, a frame here being one update and one draw: the queue drawn, the camera set with its
/// viewport, the calc walk and its two ways out (0x439450, 0x43946a), the sounds, `BeginScene`, the queue put back,
/// the fog forced and set, the draw walk, the queue drawn, `SetTexture(0, null)`, `EndScene` and the present.
extern "stdcall" fn own_render(_window: *mut c_void) -> i32 {
    let fake = running();
    let image = &fake.image;
    let device = fake.launch.device();
    let chain = Th10.chain();
    the_queue_is_drawn(0);
    image.the_frame_names_its_camera();
    the_camera_is_set(image.the_camera());
    orb_api::d3d9::set_viewport(device, image.the_cameras_viewport());
    image.the_frame_has_set_its_camera();
    let walked = unsafe { orb_core::runtime::run_calc_chain_on_the_stack(chain) };
    if walked == 0 {
        return 1;
    }
    if walked == -1 {
        return 2;
    }
    unsafe { play_sounds(image.the_sound_manager()) };
    orb_api::d3d9::begin_scene(device);
    image.the_queue_is_put_back_at_its_start();
    image.forces_the_fog_call_through();
    the_fog_is_set(image.the_supervisor());
    unsafe { orb_core::runtime::run_draw_chain_on_the_stack(chain) };
    the_queue_is_drawn(0);
    orb_api::d3d9::set_texture(device, 0, None);
    orb_api::d3d9::end_scene(device);
    unsafe { present(0) };
    0
}

/// **`0x442f50`, entered with the sprite manager in `esi`** — moved into the argument of [`the_queue_is_drawn`],
/// which is where a function of this file's own can read it.
///
/// **Naked, because the register is the whole of the call**: the compiler keeps `esi` for itself, so a function it
/// writes has no way to be entered with an argument there — see `Th10`'s `with_esi`, which is the other side of
/// the same call.
#[unsafe(naked)]
unsafe extern "C" fn draws_the_queue_in_esi() {
    std::arch::naked_asm!(
        "push esi",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym the_queue_is_drawn,
    )
}

/// **`0x421480`, entered with the camera in `edi`** — see [`draws_the_queue_in_esi`].
#[unsafe(naked)]
unsafe extern "C" fn sets_the_camera_in_edi() {
    std::arch::naked_asm!(
        "push edi",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym the_camera_is_set,
    )
}

/// **`0x420cd0`, entered with the supervisor in `edi`** — see [`draws_the_queue_in_esi`].
#[unsafe(naked)]
unsafe extern "C" fn sets_the_fog_in_edi() {
    std::arch::naked_asm!(
        "push edi",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym the_fog_is_set,
    )
}

/// **`0x422c30`, entered with the pause object in `esi`** — see [`draws_the_queue_in_esi`].
#[unsafe(naked)]
unsafe extern "C" fn unpauses_in_esi() {
    std::arch::naked_asm!(
        "push esi",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym the_run_is_unpaused,
    )
}

/// **`0x4281d0`, entered with the player in `eax`** — see [`draws_the_queue_in_esi`].
#[unsafe(naked)]
unsafe extern "C" fn steps_the_shot_in_eax() {
    std::arch::naked_asm!(
        "push eax",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym the_shot_is_stepped,
    )
}

/// The shot's step, as the image writes it — see `Image::the_shot_steps`.
extern "C" fn the_shot_is_stepped(_player: usize) {
    let fake = running();
    fake.image.the_shot_steps();
    fake.shot_stepped.set(fake.shot_stepped.get() + 1);
}

/// **The shots in flight moved, 0x428280**, `stdcall` on the player — and none are laid out to move.
extern "stdcall" fn the_shots_are_moved(_player: usize) {}

/// **The background's calc, 0x402720**, entered with its object in `eax` — and no background is laid out, the
/// stage's `.std` being nothing orb reads but through the enemies it places.
extern "C" fn the_background_runs() {}

/// The queue drawn, as the image writes it — see `Image::draws_the_queue`.
extern "C" fn the_queue_is_drawn(_sprites: usize) {
    let fake = running();
    fake.launch.asked_for(QUEUE_DRAWN);
    fake.image.draws_the_queue();
}

/// The camera set, as the image writes it — see `Image::sets_the_camera`.
extern "C" fn the_camera_is_set(camera: usize) {
    let fake = running();
    fake.launch.asked_for(CAMERA_SET);
    fake.image.sets_the_camera(camera);
}

/// **The fog set**, and the device told where the word asked it to be — which is counted here rather than at the
/// device, the device in an e2e test being a real object whose render states are not laid out.
extern "C" fn the_fog_is_set(_supervisor: usize) {
    let fake = running();
    fake.launch.asked_for(FOG_SET);
    if fake.image.sets_the_fog() {
        fake.fog_told.set(fake.fog_told.get() + 1);
    }
}

/// **The pause menu's resume, 0x422c30**: the game object's pause flag off (0x422c38), the music's command
/// (0x422c4c) and the speed put back (0x422c6d through 0x422c73).
extern "C" fn the_run_is_unpaused(pause: usize) {
    let fake = running();
    fake.image.the_game_is_paused(false);
    fake.image.a_sound_command_is_queued(THE_RESUMES_COMMAND);
    fake.image.the_pause_puts_the_speed_back(pause);
    fake.unpaused.set(fake.unpaused.get() + 1);
}

/// **The options made for the power, 0x426f70**, `stdcall` on the player — see `Image::the_options_are_made`.
extern "stdcall" fn the_options_are_made(_player: usize) {
    let fake = running();
    fake.image.the_options_are_made();
    fake.options_made.set(fake.options_made.get() + 1);
}

/// **A shot type's record made empty, 0x42acb0**, `stdcall` on the record — see `Image::a_record_is_emptied`.
extern "stdcall" fn a_record_is_emptied(record: usize) {
    running().image.a_record_is_emptied(record);
}

/// **The pad's half of the read, 0x44a190**: the keyboard's word with joystick 0's in it, read the way the read
/// reads a pad DirectInput did not find — `joyGetPosEx` with every field asked for (0x44a1f7), the mapping's
/// five buttons each into its bit (0x44a205 through 0x44a291), and each axis past a quarter of the travel
/// `joyGetDevCapsA` gave it from its middle (0x44a293 through 0x44a315). The hat is read and never looked at.
/// **The DirectInput branch is never this one's**, the laid-out machine having no DirectInput for the game to
/// find a pad through.
extern "fastcall" fn pad_half(keys: u32, _edx: u32, _player: u32, _pad: u32) -> u16 {
    let keys = keys as u16;
    let (result, pad) = orb_api::joystick::position(0, 0xff);
    let Some(caps) = orb_api::joystick::caps(0).filter(|_| result == orb_api::joyerr::NOERROR)
    else {
        return keys;
    };
    let mapping = running().image.the_pads_mapping();
    let buttons = [(0, 0x1), (1, 0x2), (2, 0x4), (3, 0x8), (8, 0x100)]
        .into_iter()
        .filter(|(at, _)| {
            u32::try_from(mapping[*at])
                .ok()
                .and_then(|button| 1u32.checked_shl(button))
                .is_some_and(|bit| pad.buttons & bit != 0)
        })
        .fold(0u16, |word, (_, bit)| word | bit);
    // Low and high, the way the read works an axis out: its middle halfway between the bounds, a quarter of the
    // travel either side of it.
    let pushed = |at: u32, low: u32, high: u32| {
        let quarter = high.wrapping_sub(low) >> 2;
        let middle = low.wrapping_add(high) >> 1;
        (
            at < middle.wrapping_sub(quarter),
            middle.wrapping_add(quarter) < at,
        )
    };
    let (left, right) = pushed(pad.x, caps.x_min, caps.x_max);
    let (up, down) = pushed(pad.y, caps.y_min, caps.y_max);
    let stick = [(up, 0x10), (down, 0x20), (left, 0x40), (right, 0x80)]
        .into_iter()
        .filter(|(held, _)| *held)
        .fold(0u16, |word, (_, bit)| word | bit);
    keys | buttons | stick
}

/// **The replay's object for a stage built fresh, 0x429610**, `stdcall` on two: the recording's seed taken and its
/// draws counted from nought (0x429081 through 0x429092) — the playback of a replay not being played — and what the
/// build stood at as it reached the seam written down.
extern "stdcall" fn replay_built(_replaying: u32, _record: u32) -> u32 {
    let fake = running();
    fake.image.the_generators_draws_start_over();
    fake.seeded_on
        .borrow_mut()
        .push((fake.image.the_games_flags(), fake.image.the_stages_frames()));
    0
}

/// **`0x43ddf0`, `stdcall` on the sound manager**: the head of its queue carried out, and the time an e2e test says
/// this frame's sounds cost.
unsafe extern "fastcall" fn play_sounds(_manager: usize) {
    let fake = running();
    fake.launch.asked_for(SOUND);
    fake.sim().clock().advance_micros(fake.sound_this_frame());
    fake.image.the_head_of_the_sound_queue_is_carried_out();
}

/// **The present, 0x4391f0**, which from here is the compositor's: where an e2e test counts a frame handed over.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
    fake.its_loader_goes_on();
}

/// The hooks `Th10::hooks` declines, handed over to be a valid `Originals` and for nothing more: no launch here
/// reaches one.
extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    0
}

extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    0
}

extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    0
}

unsafe extern "fastcall" fn card_records_read(_file: usize, _into: usize) -> i32 {
    0
}

extern "C" fn stop_recording() {}

extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

extern "thiscall" fn tops_up_the_music(_stream: usize, _argument: i32) -> i32 {
    0
}

extern "thiscall" fn stops_the_music(_stream: usize) -> i32 {
    0
}

unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

extern "C" fn pad_buttons() -> *mut c_void {
    std::ptr::null_mut()
}

extern "C" fn create_game_window(_instance: *mut c_void) {}

extern "C" fn init_d3d_device() {}

/// **Its `CreateFileA`**, which the score file's fork calls through: the name the call landed on written down, and a
/// handle that is its place in that list — the bytes of a write being the caller's to put there, see
/// [`Fake::writes_its_score_file`].
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_file(
    name: *const u8,
    _access: u32,
    _share: u32,
    _security: *const c_void,
    _disposition: u32,
    _flags: u32,
    _template: *mut c_void,
) -> *mut c_void {
    let path = unsafe { std::ffi::CStr::from_ptr(name.cast()) }
        .to_string_lossy()
        .into_owned();
    let mut opens = running().opens.borrow_mut();
    opens.push(path);
    std::ptr::without_provenance_mut(opens.len())
}

/// **An open of `scoreth10.dat` for its write through whatever stands in front of this game's `CreateFileA`** —
/// orb's hook where a launch installs it, and this file's own otherwise — by the game's own name, `0x46e848`.
fn its_score_file_opened_for_writing() {
    const GENERIC_WRITE: u32 = 0x4000_0000;
    let open = match orb_core::score::installed() {
        true => orb_core::score::create_file_a,
        false => create_file,
    };
    unsafe {
        open(
            c"scoreth10.dat".as_ptr().cast(),
            GENERIC_WRITE,
            0,
            std::ptr::null(),
            0,
            0,
            std::ptr::null_mut(),
        )
    };
}

/// **Its `CreateWindowExA`**, which the window's rewrite calls through with the arguments orb decided: the host
/// makes the window, on the class this file names.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_window(
    ex_style: u32,
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
    running()
        .sim()
        .windows()
        .create_window(asked, style, ex_style)
        .0 as *mut c_void
}

/// `D3D_SDK_VERSION` as `d3d9.h` has it, which th10 hands `Direct3DCreate9`.
const D3D_SDK_VERSION: u32 = 32;
/// `D3DDEVTYPE_HAL`.
const D3DDEVTYPE_HAL: u32 = 1;
/// `D3DCREATE_HARDWARE_VERTEXPROCESSING`, the flag th10 makes its device with (`push 0x40`, 0x439a00).
const D3DCREATE_HARDWARE_VERTEXPROCESSING: u32 = 0x40;
/// **Where th10 keeps the present parameters** its `CreateDevice` and its `Reset` are handed (0x439232).
const PRESENT_PARAMETERS: usize = 0x491d0c;
/// **The presentation interval the fake's parameters hold**, `D3DPRESENT_INTERVAL_ONE`: a present that waits for
/// the display's blank, which is what held a launch of th10 at `--pace=8` to one frame a refresh, `8309us apart`
/// on a 120Hz display. The game's own number is not read; what matters to a test is that it waits.
const THE_GAMES_PRESENTATION_INTERVAL: u32 = 1;

/// **Where the host's Direct3D object goes**, its vtable right behind it: an address past every range th10 lays
/// out and past the device vtable, chosen rather than allocated for the reason `fake::DEVICE_VTABLE` gives — orb
/// reads the vtable and swaps its slot through the seam, which answers out of the space.
const DIRECT3D: usize = 0x2100_0000;
const DIRECT3D_VTABLE: usize = DIRECT3D + size_of::<usize>();
const DIRECT3D_BYTES: usize = size_of::<usize>() * (orb_api::d3d9::CREATE_DEVICE_SLOT + 2);

/// **And its `Direct3DCreate9`**, which orb's rewrite calls through: an object whose vtable has the host's
/// `CreateDevice` in its slot and nothing the game calls in any other, laid out once for the launch and left.
unsafe extern "system" fn host_makes_a_direct3d(_sdk: u32) -> *mut c_void {
    let space = running().sim().space();
    assert!(
        space.has_room(DIRECT3D, DIRECT3D_BYTES),
        "{DIRECT3D:#010x} is inside a range this game is laid out at",
    );
    space.map(DIRECT3D, DIRECT3D_BYTES, orb_api::Kind::Private);
    space.write::<usize>(DIRECT3D, DIRECT3D_VTABLE);
    space.write::<usize>(
        DIRECT3D_VTABLE + orb_api::d3d9::CREATE_DEVICE_SLOT * size_of::<usize>(),
        host_creates_a_device as *const () as usize,
    );
    DIRECT3D as *mut c_void
}

/// **And its `IDirect3D9::CreateDevice`**, which writes down the behaviour flags it was handed and makes nothing — the
/// device a test draws through being the one the fake was laid out with.
unsafe extern "system" fn host_creates_a_device(
    _this: *mut c_void,
    _adapter: u32,
    _kind: u32,
    _focus: *mut c_void,
    behaviour: u32,
    parameters: *mut c_void,
    _device: *mut *mut c_void,
) -> i32 {
    let game = running();
    game.device_made_with.set(Some(behaviour));
    game.device_presents_with.set(Some(
        game.sim()
            .space()
            .read::<u32>(parameters as usize + orb_api::d3d9::PRESENTATION_INTERVAL),
    ));
    0
}

/// **And its `SetForegroundWindow`**, which orb's rewrite calls through for a window it leaves to the game: the host
/// writes the window down as brought forward.
unsafe extern "system" fn host_brings_a_window_forward(window: *mut c_void) -> i32 {
    i32::from(
        running()
            .sim()
            .windows()
            .bring_forward(orb_api::Hwnd(window as usize)),
    )
}
