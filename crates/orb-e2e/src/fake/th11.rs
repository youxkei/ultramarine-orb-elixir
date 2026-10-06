//! A th11 that is not the real one: the part of the game orb has read, played over the address space
//! [`orb_core::game::th11::image`] lays out.
//!
//! **What it plays is as much of the game's part as the e2e tests over it ask for**, and each piece of it is
//! `th11.exe`'s own at the addresses written beside it: the frame at 0x446420 in its own order, with the queue of
//! quads, the camera and the fog it sets around its two walks; the walks themselves, the calc walk entered with its
//! chain in `ebx` and the draw walk with nothing, and the calc walk's jobs in their priority's order; the
//! supervisor, which carries the sound's commands out, reads the keys through the pad's half of the read and
//! switches the scene; the title's main menu with the Extra's item and the way out, the three selects a run is
//! chosen through and the practice's stage select; the stage's build on a loader thread, with the replay's seam for
//! the seed inside it and its wait for the sound's queue to empty, and the stage after one won with the run's
//! numbers carried and its `.std` fading in under the last one's; the player's states, its move and its shot; aimed
//! fire, a midboss of two attacks and a boss on a stage script of this file's own; the pause object's menu and the two
//! ends of a run, with the ranking's name entry and the continue; the ending, and the clear's name entry and the
//! replay's slots after it; the score object read at the startup and written at each run's teardown; the window it
//! makes and its device; and the sound manager's queue of commands.
//!
//! **What it does not play, and says so where a piece would be**: a bomb of its own — one is let go and put out
//! where an e2e test says — the lasers, the items, the lines of a conversation, the pause menu's yes or no, the
//! replay's slots and its name, the options, and the attract demo coming up by itself. A screen or a state this file
//! has no handler for stands where it was put.
//!
//! **Where a number here is th11's it says so**, and where it is this file's own it says that. Over [`Launch`],
//! which is the same half of a launch every other game's e2e tests run on.

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th11::image::{
    A_BLANK_NAME, A_PLAY, A_SONGS_PATH, A_STOP, DEVICE_ACQUIRE, DEVICE_GET_STATE, DEVICE_POLL,
    Image, Pause, Player, STEP_BUILT, STEP_DECIDED, STEP_LEFT, STEP_READING, STEP_WAITING, Scene,
    Screen, THE_PAUSES_COMMAND, THE_RESUMES_COMMAND, TheTitlesEntry,
};
use orb_core::game::th11::{Th11, score_file};
use orb_sim::keys;

use super::{DRAW, Display, Launch, Launched, PRESENT, SOUND, UPDATE, WINDOW, Work, scratch};

/// **The names this game's own record of a frame uses past the four every game's has** — see
/// [`Launched::asked`]: the queue drawn by the frame (0x44fd10, at 0x44649f and 0x44657c), the camera set
/// (0x42a810 at 0x4464a9) and the fog set (0x42a300 at 0x44656c).
pub const QUEUE_DRAWN: &str = "queue drawn";
pub const CAMERA_SET: &str = "camera set";
pub const FOG_SET: &str = "fog set";

/// **The calls of one frame, in the order the game's own frame at 0x446420 makes them** — `asm.md`'s *The frame at
/// `0x446420`, in order*: the queue drawn, the camera set, the calc walk, the fog set inside the scene, the draw
/// walk, the queue drawn again and the present. **No sounds**: this game carries its sound's commands out inside the
/// calc walk, from the supervisor's job (0x429058).
pub const THE_FRAMES_OWN_ORDER: [&str; 7] = [
    QUEUE_DRAWN,
    CAMERA_SET,
    UPDATE,
    FOG_SET,
    DRAW,
    QUEUE_DRAWN,
    PRESENT,
];

/// **Which key is which bit of the word the read folds the keyboard into** — the `GetKeyboardState` half of
/// 0x4576b0 (0x4576ee through 0x457848) and its DirectInput half (0x457883 through 0x4579e9): `Z` 0x1, `X` 0x2,
/// Shift 0x8, the arrows 0x10 through 0x80, Esc 0x100, Ctrl 0x200, `Q` 0x10000, Enter 0x80000 and `R` 0x200000 —
/// the keys an e2e test here presses. The numeric keypad, `S`, `D`, Home, `P` and F10 are out, no e2e test here
/// pressing them.
const MAP: [(u8, u32); 12] = [
    (keys::Z, 0x0000_0001),
    (keys::X, 0x0000_0002),
    (SHIFT, 0x0000_0008),
    (keys::UP, 0x0000_0010),
    (keys::DOWN, 0x0000_0020),
    (keys::LEFT, 0x0000_0040),
    (keys::RIGHT, 0x0000_0080),
    (keys::ESCAPE, 0x0000_0100),
    (CONTROL, 0x0000_0200),
    (Q, 0x0001_0000),
    (keys::RETURN, 0x0008_0000),
    (R, 0x0020_0000),
];

/// The keys `orb_sim::keys` does not name: Shift, Ctrl, `Q` and `R`.
const SHIFT: u8 = 0x10;
const CONTROL: u8 = 0x11;
pub const Q: u8 = 0x51;
/// `R`, which the pause menu takes as its own way to a start-over (0x42cb38).
pub const R: u8 = 0x52;

/// **The masks the game's screens read the buttons gone down against**: decide `0x80001` and cancel `0x102` on every
/// menu (0x42ca55, 0x42d1a5), up `0x10`, down `0x20`, left `0x40` and right `0x80`, the pause `0x100` (0x42bf53), and
/// the pause menu's own start-over `0x200000` and title `0x10000` (0x42cb38, 0x42cba6). This file's own literals, for
/// the reason `th11::image`'s module doc gives.
const DECIDE: u32 = 0x0008_0001;
const CANCEL: u32 = 0x0000_0102;
const UP: u32 = 0x0000_0010;
const DOWN: u32 = 0x0000_0020;
const TO_THE_LEFT: u32 = 0x0000_0040;
const TO_THE_RIGHT: u32 = 0x0000_0080;
/// And the focus, `>> 3 & 1` of the run's word (0x430373).
const FOCUS: u32 = 0x0000_0008;
/// And the shot, which with the focus is what Yukari's gap asks is let go (`test [0x4c93c0], 9`, 0x430e79).
const SHOT: u32 = 0x0000_0001;
/// **Yukari's gap's states** (0x430e50): against the left wall and the direction let go there, against the right
/// and let go there, and carried out of the left and of the right.
const GAP_AGAINST_THE_LEFT: i32 = 1;
const GAP_LET_GO_AT_THE_LEFT: i32 = 2;
const GAP_AGAINST_THE_RIGHT: i32 = 3;
const GAP_LET_GO_AT_THE_RIGHT: i32 = 4;
const GAP_OUT_OF_THE_LEFT: i32 = 99;
const GAP_OUT_OF_THE_RIGHT: i32 = 100;
/// The frames after letting go the wall is to be pressed into again within, more than 8 being too late (0x430ee5).
const THE_GAP_IS_TAKEN_WITHIN: i32 = 8;
/// What the gap carries the player by a frame, in 128ths (0x430396, 0x4303a5).
const THE_GAP_CARRIES_BY: i32 = 0x96;
/// Where it is put across the field, past `0x6800` from the middle by `0xd000` (0x430ff2 through 0x431012).
const THE_GAP_COMES_OUT_PAST: i32 = 0x6800;
const THE_GAP_COMES_IN_BY: i32 = 0xd000;
/// How many frames it lasts (0x43104d).
const THE_GAP_LASTS: i32 = 0x2d;
const THE_PAUSE: u32 = 0x0000_0100;
const THE_PAUSES_START_OVER_KEY: u32 = 0x0020_0000;
const THE_PAUSES_TITLE: u32 = 0x0001_0000;

/// **How many frames a launch left to itself spends before its title is wanted**: the loading object's job that
/// writes scene 4 once the start-up's files are in (0x428f10). This file's own number, as th10's is.
const TITLE_COMES_UP_AFTER: u32 = 60;

/// **The frame the title's thread unloads the loading screen's file on**: th11's thread waits for the loading
/// screen's counter `[[0x4a8e98]+0x474]` to reach 180 (0x439521 through 0x43954c) and then unloads slot 1 through
/// 0x4549e0 (0x439564) while the frames go on. Counted here from the launch's first frame, which is this file's own
/// stand-in for that counter.
const THE_LOADING_SCREENS_FILE_GOES_AT: u32 = 180;

/// **How long the walk the unload is asked on gives the title's thread**, which is the frame going on beside the
/// thread rather than ahead of it: a thread let run takes far less, and one that is waiting on the frame never
/// finishes inside it. This file's own bound.
const THE_TITLES_THREAD_IS_GIVEN: std::time::Duration = std::time::Duration::from_millis(20);

/// **Whether the loading screen's file is in**, from the launch until the title's thread unloads it — a static and
/// not a field, the thread being the game's other one.
static THE_LOADING_SCREENS_FILE_IS_IN: AtomicBool = AtomicBool::new(true);

/// **The one window th11's LZSS decode and encode share**, `0x4c0ee8` — 8KB there, as many bytes here as the one
/// instruction this file decodes through it.
static THE_WINDOW: [AtomicU8; TITLE_ANM_SCRIPT_0.len()] =
    [const { AtomicU8::new(0) }; TITLE_ANM_SCRIPT_0.len()];

/// **`title.anm`'s script 0 as the archive holds it**: its first instruction, opcode `0x55`, length `0xc`, its
/// argument 1 — the words every launch held for a debugger read at the load (`000c0055 00000000 00000001`).
const TITLE_ANM_SCRIPT_0: [u8; 12] = [0x55, 0, 0x0c, 0, 0, 0, 0, 0, 1, 0, 0, 0];

/// **The score file's bytes the loading's thread decodes**, which this file never reads back: what the decode is
/// for here is the window it writes over.
const THE_SCORE_FILES_BYTES: [u8; 12] = [0x54, 0x48, 0x31, 0x31, 0, 0, 0, 0, 0, 0, 0, 0];

/// **How long the first decode holds its bytes in the window for another to come in beside it**, which is this
/// file's stand-in for a decode of a whole file: the other decode, let in, finishes long before; kept out, it never
/// does inside this, and the first copies out what it put in. Short, the frames being held for it.
const A_DECODE_LASTS: std::time::Duration = std::time::Duration::from_millis(20);

/// **Whether a decode has its bytes in the window**, which the loading's thread waits for before it decodes the
/// score file — the order a launch held for a debugger logged them in: the score's decode still running on the
/// loading's thread as the title's thread began on `title.anm` (0x437952, 0x439660).
static A_DECODE_IS_UNDER_WAY: AtomicBool = AtomicBool::new(false);

/// **How many decodes have copied out**, which the first one waits on to go up.
static DECODES_DONE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// **`title.anm`'s script 0 as the title's thread decoded it**, for the main menu to make its VM out of.
static TITLE_ANM_DECODED: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

/// **The frames each screen of the title ignores a key for, and the frames a decide takes to act**: past `0x50` on
/// the main menu (0x43a22d) and past 6 on the three selects (0x43c8b6, 0x43cd27, 0x43d3a3) and 10 on practice's stage
/// (0x43d72f); the main menu's decide carried out past `0x14` (0x43a4d0), the difficulty's and the character's past
/// `0xd` (0x43caee, 0x43cf85), the partner's on frame 10 for a practice and past `0x27` for a run (0x43d50f,
/// 0x43d57a), practice's stage past `0x27` (0x43d9e8); and a cancel's step 4 back a screen past 5.
const THE_MAIN_MENU_READS_PAST: i32 = 0x50;

/// **The main menu's frame that makes its VM of `title.anm`'s script 0 on layer 0x16**, `0x455a00` at 0x43a1d4 —
/// which runs the VM's script as it makes it (0x4569b5 into 0x44ae40).
const THE_MAIN_MENU_MAKES_ITS_VM_AT: i32 = 0x46;
const A_SELECT_READS_PAST: i32 = 6;
const THE_PRACTICE_SELECT_READS_PAST: i32 = 10;
const THE_MAIN_MENU_ACTS_PAST: i32 = 0x14;
const A_SELECT_MOVES_ON_PAST: i32 = 0xd;
const A_PRACTICE_IS_CHOSEN_AT: i32 = 10;
const THE_PARTNER_STARTS_THE_RUN_PAST: i32 = 0x27;
const A_PRACTICE_STARTS_PAST: i32 = 0x27;
const A_CANCEL_GOES_BACK_PAST: i32 = 5;
/// **The counts of items each screen's cursor moves over**: the main menu's eight, four difficulties outside the
/// Extra, two characters, three partners and six practice stages.
const MAIN_MENU_ITEMS: i32 = 8;
const DIFFICULTIES: i32 = 4;
const CHARACTERS: i32 = 2;
const PARTNERS: i32 = 3;
const PRACTICE_STAGES: i32 = 6;
/// **The main menu's items this file plays**: a run, the Extra, a practice, and the way out, the last.
pub const THE_ITEM_FOR_A_RUN: i32 = 0;
const THE_ITEM_FOR_THE_EXTRA: i32 = 1;
pub const THE_ITEM_FOR_A_PRACTICE: i32 = 2;
const THE_LAST_ITEM: i32 = 7;

/// **The frames a stage's build runs on its thread**, from the frame the game object is made to the one its songs
/// are queued: th10's 55, the length of th11's not having been read off a launch.
const A_BUILD_RUNS_FOR: u32 = 55;
/// **The run's flag that keeps its managers for the next build**, `0x2`, which the teardown for scene 12 sets
/// (0x41fe73) and the build reads to go through 0x436f30 rather than the replay's seam (0x41fc98).
const THE_RUN_KEEPS_ITS_MANAGERS: u32 = 0x2;

/// **The frames the player's states run for**: sixty coming back in (0x431188), eight for a bomb to take a hit back
/// (0x43122d), the power taken on the dying state's third (0x4312c6) and the run going on or ending from its
/// thirtieth (0x4313e6).
const COMING_BACK_IN_FOR: i32 = 60;
const A_HIT_WAITS_FOR: i32 = 8;
const THE_DYING_TAKES_THE_POWER_ON: i32 = 3;
const DYING_FOR: i32 = 30;
/// **The update's frames the focus is held off for** (0x430290).
const THE_FOCUS_IS_HELD_OFF_FOR: i32 = 4;

/// **The pause object's frames**: its opening waits until the game object is thirty frames in (0x42bf77) and opens
/// over ten (0x42c8fa); its item is carried out twelve frames on (0x42d460); the two tallies wait ten (0x42da94,
/// 0x42e774), the ranking's name entries read past ten, and a run's end carries its item out twelve frames on
/// (0x42e582, 0x42f2b2).
const A_PAUSE_OPENS_PAST: i32 = 30;
const THE_PAUSE_OPENS_OVER: i32 = 10;
const THE_PAUSE_CLOSES_OVER: i32 = 12;
const THE_END_TALLIES_FOR: i32 = 10;
const THE_ENTRY_READS_PAST: i32 = 10;
const THE_END_LEAVES_AFTER: i32 = 12;
/// **The pause menu's items**: four — the resume, the title, the replay's save and the start-over (0x42c8fa) — the
/// replay's save left out for a replay played (0x42c91f).
const THE_PAUSES_ITEMS: i32 = 4;
const THE_PAUSES_RESUME: i32 = 0;
const THE_PAUSES_WAY_TO_THE_TITLE: i32 = 1;
const THE_PAUSES_SAVE: i32 = 2;
const THE_PAUSES_START_OVER: i32 = 3;
/// **A lost run's menu**: four — the continue, the title, the replay's save and the start-over (0x42db4e); and a won
/// run's, three — the title, the replay's save and the start-over (0x42e7f7).
const A_LOST_RUNS_ITEMS: i32 = 4;
const A_LOST_RUNS_CONTINUE: i32 = 0;
const A_LOST_RUNS_SAVE: i32 = 2;
const A_LOST_RUNS_START_OVER: i32 = 3;
const A_WON_RUNS_ITEMS: i32 = 3;
const A_WON_RUNS_SAVE: i32 = 1;
const A_WON_RUNS_START_OVER: i32 = 2;

/// **The difficulty a placed run is played on**, Normal, and the stage it starts in, as orb counts one.
pub const DIFFICULTY_NORMAL: i32 = 1;
pub const FIRST_STAGE: i32 = 0;

/// **The stage this file plays, which is its own script and not a stage of `th11.dat`'s** — th10's fake's, the same
/// numbers: a bullet aimed at the player every so many frames from the first, a midboss of two attacks part way in,
/// and the stage's boss for its last frames to a song of her own — a nonspell, a card and a nonspell — beaten a little
/// before the stage ends.
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
/// The cards the two fights put up, by the number the card object keeps: numbers of this file's own among the 175.
pub const THE_MIDBOSSS_CARD: i32 = 2;
pub const THE_BOSSS_CARD: i32 = 7;
/// **The tracks of `thbgm.fmt` the stage is played to**: the stage's own in slot 0, which the stage's first frame
/// plays (0x420525), and the boss's in slot 1, which the conversation before her plays (0x41e9d7) — by their records'
/// names `th11_01.wav` and `th11_02.wav`, stage 1's own two, whichever stage it is.
pub const THE_STAGES_TRACK: usize = 1;
pub const THE_BOSSS_TRACK: usize = 2;
const THE_STAGES_SLOT: u32 = 0;
const THE_FIGHTS_SLOT: u32 = 1;
/// **The last stage of a run**, whose end has the panel ask for the ending 380 frames on (0x41b3fc through
/// 0x41b430), and the Extra's stage word, 7, whose end is the run's (0x41ed54).
const THE_LAST_STAGE: i32 = 6;
const THE_ENDING_IS_ASKED_FOR_AFTER: u32 = 380;
const THE_EXTRAS_STAGE: i32 = 7;
/// **The Extra's difficulty**, 4, which its item of the main menu writes (0x43a539).
const DIFFICULTY_EXTRA: i32 = 4;
/// **How many frames the ending runs before its last line**, which then stands until the decide goes down: this
/// file's own count, th10's fake's, the ending's own length not having been read.
const THE_ENDING_LASTS: u32 = 600;

/// **What a stage won adds to the run's score**, in the score's own tenths. This file's own number: what it has to be
/// is a score a whole run of six puts in the ranking this file's records start with.
pub(crate) const A_STAGE_WON_SCORES: u32 = 200_000;

/// **The grid every name entry walks**: 88 characters and then the three places for a space, a step back and the end,
/// `0x58`, `0x59` and `0x5a` (0x42d078), in rows of thirteen.
const THE_GRID: &[u8; 91] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-=.,!?@:;[]()_/{}|~^#$%&*   ";
const A_GRIDS_ROW: i32 = 13;
/// **The frames the title's name entry's steps and the replay's slots wait**: its first step's, which is not read
/// (th10's 6); its third's five on to the slots (`asm.md`'s *The name entries*); the slots' own wait, not read (th10's
/// 6); and a cancel's six frames out of them (`+0x2b4 > 5`).
const THE_TITLES_ENTRY_READS_PAST: i32 = 6;
const THE_TITLES_ENTRY_LEAVES_PAST: i32 = 5;
const THE_SLOTS_READ_PAST: i32 = 6;
const A_CANCEL_LEAVES_THE_SLOTS_PAST: i32 = 5;
const THE_REPLAYS_SLOTS: i32 = 25;
/// **The frame of the game object's timer the stage after one won begins on**, its stage before faded out
/// (0x4205b1).
const A_FADE_BEGINS_THE_STAGE_AT: i32 = 0x1e;

/// **The class 0x446ae0 registers (0x446b5a) and makes its window on**: `BASE`, at `0x49688c` — which is what orb
/// recognises the window it plays in by: see `orb_core::window::is_game_class`.
const WINDOW_CLASS: &std::ffi::CStr = c"BASE";

/// **One of the windows 0x446ae0 asks `CreateWindowExA` for**: the style, where, and the whole size.
#[derive(Clone, Copy)]
pub struct Ask {
    pub style: u32,
    pub at: (i32, i32),
    pub size: (i32, i32),
}

/// **The one where `th11.cfg` says the whole screen** (0x446c28): `WS_POPUP | WS_VISIBLE` at the origin, 640x480.
pub const ASKED_FOR_FULL_SCREEN: Ask = Ask {
    style: 0x9000_0000,
    at: (0, 0),
    size: (640, 480),
};

/// **And where it says a window of one of its three sizes** (0x446cb5): `WS_VISIBLE | WS_SYSMENU | WS_MINIMIZEBOX |
/// WS_MAXIMIZEBOX` at the configuration's `+0x28`/`+0x2c`, `CW_USEDEFAULT` by default (0x420e01), the size with
/// twice `SM_CXFIXEDFRAME`/`SM_CYFIXEDFRAME` and `SM_CYCAPTION` round it. The three metrics are this file's own, the
/// laid-out host answering none.
const A_WINDOWS_STYLE: u32 = 0x100b_0000;
const THE_WINDOW_SIZES: [(i32, i32); 3] = [(640, 480), (960, 720), (1280, 960)];
const FIXED_FRAME: (i32, i32) = (3, 3);
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

/// **Where the stage's build on its thread stands**.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Build {
    None,
    /// The frames it has left before it queues the stage's songs.
    Running(u32),
    /// **Its songs queued, and `Sleep(16)` until the sound's queue is empty** (0x41fd87 through 0x41fd9e) — which the
    /// supervisor's job on the main thread empties, one command a walk.
    WaitingOnTheSound,
}

/// A th11 laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// Whether orb stands in front of this game's pad half of the read.
    reads_the_keys: bool,
    /// How many frames this launch has run, for the title wanted on its own — see [`TITLE_COMES_UP_AFTER`].
    frames_since_launch: Cell<u32>,
    /// **Where the stage's build stands** — see [`Build`].
    building: Cell<Build>,
    /// **Whether the stage's own jobs are on the chain**, which the game object's first frame of a stage built fresh
    /// turns them on for and a run's teardown takes off — and which the stage after one won finds on, its managers
    /// kept.
    the_stage_is_on: Cell<bool>,
    /// **What the replay's seam saw each time the build reached it**: the game object's flags and the stage's
    /// frames — see [`Fake::what_the_build_seeded_on`].
    seeded_on: RefCell<Vec<(u32, u32)>>,
    /// How many times the options were made for the power, orb's calls of it only — see [`the_options_are_made`].
    options_made: Cell<u32>,
    /// How many times orb stepped the player's shot through the game's own call — see [`the_shot_is_stepped`].
    shot_stepped: Cell<u32>,
    /// **Whether the player's shot sits out this update**, which is the update a stage built fresh begins on — th10's
    /// measurement, and th11's own reading of the same: the pick-up's shot timer one behind the run's from frame 2.
    the_shot_waits: Cell<bool>,
    /// **How many updates the stage's `.std` stepped its script on** — see `Image::the_background_steps_its_script`.
    background_stepped: Cell<u32>,
    unpaused: Cell<u32>,
    fog_told: Cell<u32>,
    /// **The behaviour flags the host's `IDirect3D9::CreateDevice` was last handed**, and the presentation interval in
    /// the parameters — see [`Fake::makes_its_direct3d_and_its_device`].
    device_made_with: Cell<Option<u32>>,
    device_presents_with: Cell<Option<u32>>,
    /// The word the pad's half of the read answered on the last update.
    word: Cell<u32>,
    /// **Whether the run's stage is played by this file's own script** — see [`FIRST_BULLET_AT`] — and how long
    /// a stage of it lasts, where an e2e test said; and whether it fires.
    on_its_own: Cell<bool>,
    stage_lasts: Cell<Option<u32>>,
    fires: Cell<bool>,
    /// Where the stage's fight stands.
    fight: Cell<Fight>,
    /// **The frames the panel has counted towards the ending**, where the last stage has been won.
    towards_the_ending: Cell<Option<u32>>,
    /// The frames the ending has run.
    the_ending_has_run_for: Cell<u32>,
    /// **The items the title's screen up leaves out**, which its cursor passes over. Where th11 keeps such a list was
    /// not read, so it is this file's and not the address space's: nothing of orb's reads it.
    left_out: RefCell<Vec<i32>>,
    /// **Whether the startup's read of `scoreth11.dat` found a file**.
    read_a_score_file: Cell<bool>,
    /// **The names each `CreateFileA` of this game's landed on**, in order.
    opens: RefCell<Vec<String>>,
    /// **Whether the game is leaving**, its switch having built scene 3.
    leaving: Cell<bool>,
    /// **The DirectInput pad this file holds**, where the launch plugged one in.
    directinput_pad: Cell<DirectInputPad>,
    /// **What the pad's half of the read made of the pads alone** on the last read it made.
    pads_share: Cell<u32>,
    /// **Whether the update unloads through orb's hook** — see `Hooks::unloads_beside_its_frame` — which a launch
    /// installs only beside orb's own frame.
    unloads_through_orb: bool,
    /// **Whether the title's thread has been started on its unload and the file not yet seen gone between frames**.
    the_unload_is_under_way: Cell<bool>,
    /// **Whether the title was built out of the loading screen**, which is the title whose thread unloads that screen's
    /// file — a title placed rather than walked to having no loading screen behind it to unload.
    the_title_came_out_of_the_loading: Cell<bool>,
    /// **Whether the decodes go through orb's hook** — see `Hooks::decodes_through_one_window`, which a launch
    /// installs whatever it is for.
    decodes_through_orb: bool,
    /// **The two threads decoding through the window as the first title is built**: the title's on `title.anm` and
    /// the loading's on the score file — joined as the main menu makes its VM out of what the first decoded.
    decoding: RefCell<Vec<std::thread::JoinHandle<()>>>,
    /// **Whether the walks stand spinning**: the loading screen's file unloaded under a walk that was running its
    /// VMs, whose scripts are then memory that holds no script — th11's interpreter on an instruction of size
    /// nought (0x44b5c0, 0x44c558). Nothing of the walk runs again, which is this file's way of a walk that never
    /// returns.
    stands_spinning: Cell<bool>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory.
    _installed: orb_api::Installed,
}

/// **The facts about the machine a launch is declared with** beside its display.
struct Host {
    ui_language: Option<u16>,
    monitor: bool,
    device: bool,
    pad: PluggedIn,
}

impl Host {
    const ORDINARY: Self = Self {
        ui_language: None,
        monitor: true,
        device: true,
        pad: PluggedIn::Nothing,
    };
}

pub use super::{A_WINMM_PADS_BUTTONS, A_WINMM_PADS_TRAVEL, DirectInputPad, PluggedIn};

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into.
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

    /// **One frame of the game, entered the way this game's loop enters its frame**: pushed, `push 0x4c3d88` and one
    /// of the three frames at 0x44587e, 0x44589b or 0x4458a7 — which is `runtime::render_on_the_stack` where orb's
    /// loop stands over the prologue of each.
    ///
    /// **And the process ended where the game is leaving**: a frame answering 1 leaves WinMain's loop (0x4458b2).
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
    /// way a launch there is attached.
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

    /// And with a pad plugged in as the game starts.
    pub fn attach_with_a_pad(
        pad: PluggedIn,
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let host = Host {
            pad,
            ..Host::ORDINARY
        };
        Self::attach_declaring(display, host, name, &[], work, settings)
    }

    /// And on a host that will not say where its panel is.
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

    /// And on a machine whose own windows are in the language `ui_language` names.
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

    /// And in a directory that already holds `left`, each file at its path inside the directory.
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
    /// before WinMain's `0x446d30` (called at 0x445750) has `IDirect3D9::CreateDevice` write the device to `0x4c3288`
    /// (0x446f5d). See [`makes_its_device`](Fake::makes_its_device).
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
        let caps_through_orb = config.through_a_pad && config.autoplay.is_some();
        // **No `font.ttf` beside this game's exe**, th11 keeping its own inside `th11.dat`.
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
            makes_the_options_in_ebx as *const () as usize,
            the_run_is_unpaused as *const () as usize,
        );
        image.hands_over_the_landings_own_calls(
            steps_the_shot_in_eax as *const () as usize,
            the_shots_are_moved as *const () as usize,
        );
        image.hands_over_the_score_objects_own_call(empties_a_record_in_eax as *const () as usize);
        // **The configuration's defaults**, which 0x420d80 writes where no `th11.cfg` was read.
        image.the_configurations_defaults_are_in();
        // **The generator seeded as the supervisor's job is added**, from `timeGetTime` (0x429582) — a number of this
        // file's own.
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

        let unloads_through_orb = launch.own_frame_loop
            && the_game_this_is()
                .hooks()
                .unloads_beside_its_frame
                .is_some();
        THE_LOADING_SCREENS_FILE_IS_IN.store(true, Ordering::Relaxed);
        let fake = Box::new(Self {
            image,
            launch,
            reads_the_keys: the_game_this_is().hooks().input.is_some(),
            frames_since_launch: Cell::new(0),
            building: Cell::new(Build::None),
            the_stage_is_on: Cell::new(false),
            seeded_on: RefCell::new(Vec::new()),
            options_made: Cell::new(0),
            shot_stepped: Cell::new(0),
            the_shot_waits: Cell::new(false),
            background_stepped: Cell::new(0),
            unpaused: Cell::new(0),
            fog_told: Cell::new(0),
            device_made_with: Cell::new(None),
            device_presents_with: Cell::new(None),
            word: Cell::new(0),
            on_its_own: Cell::new(false),
            stage_lasts: Cell::new(None),
            fires: Cell::new(true),
            fight: Cell::new(Fight::None),
            towards_the_ending: Cell::new(None),
            the_ending_has_run_for: Cell::new(0),
            left_out: RefCell::new(Vec::new()),
            read_a_score_file: Cell::new(false),
            opens: RefCell::new(Vec::new()),
            leaving: Cell::new(false),
            directinput_pad: Cell::new(DirectInputPad::default()),
            pads_share: Cell::new(0),
            unloads_through_orb,
            the_unload_is_under_way: Cell::new(false),
            the_title_came_out_of_the_loading: Cell::new(false),
            decodes_through_orb: the_game_this_is()
                .hooks()
                .decodes_through_one_window
                .is_some(),
            decoding: RefCell::new(Vec::new()),
            stands_spinning: Cell::new(false),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        // **The three trampolines a launch of this game leaves in the slots of its own**, which is what `orb`'s
        // install writes for a game whose calc walk takes its chain in `ebx`, whose draw walk takes nothing and whose
        // frame takes the window on the stack — see `orb_core::runtime::run_calc_chain_in_ebx`.
        orb_core::runtime::RUN_CALC_CHAIN_IN_EBX
            .store(update_in_ebx as *const () as usize, Ordering::Relaxed);
        orb_core::runtime::RUN_DRAW_CHAIN_TAKING_NOTHING
            .store(draw as *const () as usize, Ordering::Relaxed);
        orb_core::runtime::RENDER_ON_THE_STACK
            .store(own_render as *const () as usize, Ordering::Relaxed);
        orb_core::runtime::UNLOADS_IN_EDI.store(
            the_loading_screens_file_is_unloaded as *const () as usize,
            Ordering::Relaxed,
        );
        orb_core::runtime::DECODE_ALONE.store(decode as *const () as usize, Ordering::Relaxed);
        A_DECODE_IS_UNDER_WAY.store(false, Ordering::Relaxed);
        DECODES_DONE.store(0, Ordering::Relaxed);
        TITLE_ANM_DECODED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        unsafe {
            orb_core::runtime::attach_to(
                the_game_this_is(),
                config,
                fake.image.data(),
                orb_core::runtime::Originals {
                    update: calls_the_calc_walk_in_ebx,
                    draw: calls_the_draw_walk_taking_nothing,
                    // **The pad's half, in the one slot every game's read goes in**: `runtime::
                    // get_input_with_the_keys_in_ecx` takes it back out as the `fastcall` of one it is.
                    input: std::mem::transmute::<
                        extern "fastcall" fn(u32) -> u32,
                        extern "system" fn() -> u16,
                    >(pad_half),
                    // And the replay's seam the same way, a `stdcall` of one that
                    // `runtime::stage_building_of_one` calls through.
                    stage_building: std::mem::transmute::<
                        extern "stdcall" fn(u32) -> u32,
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
            unsafe { orb_core::runtime::plays_the_run(None) };
        }
        // **The input's set-up, in WinMain on the main thread and before `Direct3DCreate9`** (0x4456aa through
        // 0x445702): the joystick probed, and the keyboard and the pad DirectInput finds made — so a launch's first
        // frame already reads through them, where th10's set-up raced its frames on a thread of its own.
        if host.pad == PluggedIn::OnWinmm {
            fake.sim().joystick().attach(
                A_WINMM_PADS_BUTTONS,
                A_WINMM_PADS_TRAVEL.0,
                A_WINMM_PADS_TRAVEL.1,
            );
        }
        fake.probes_for_a_joystick(caps_through_orb);
        fake.image.the_input_set_up_makes_a_keyboard(
            its_keyboard_is_acquired as *const () as usize,
            its_keyboard_is_let_go_of as *const () as usize,
        );
        KEYBOARD_ACQUIRED.store(true, Ordering::Relaxed);
        if host.pad == PluggedIn::ThroughDirectInput {
            fake.image.a_pad_is_found_through_directinput(
                its_pad_acquires as *const () as usize,
                its_pad_is_read as *const () as usize,
                its_pad_is_polled as *const () as usize,
            );
        }
        fake.frame_takes(work);
        fake
    }

    /// **The set-up's probe for a joystick, 0x4571a0** (0x4456aa): `joyGetPosEx` of joystick 0 and then of joystick 1
    /// through the import orb writes over, and where either answers, `joyGetDevCapsA(0, 0x4c2f08, 0x194)` — orb's
    /// where orb writes over that entry too.
    fn probes_for_a_joystick(&self, caps_through_orb: bool) {
        let asked = || orb_api::JoyInfo {
            size: size_of::<orb_api::JoyInfo>() as u32,
            flags: orb_core::joystick::RETURN_ALL,
            ..orb_api::JoyInfo::default()
        };
        let found = [0, 1].into_iter().any(|device| {
            let mut info = asked();
            unsafe { orb_core::joystick::answer(device, &mut info) == orb_api::joyerr::NOERROR }
        });
        if !found {
            return;
        }
        let caps = match caps_through_orb {
            true => {
                let mut caps = orb_api::JoyCaps::default();
                let answered = unsafe {
                    orb_core::joystick::caps_answer(
                        0,
                        &mut caps,
                        size_of::<orb_api::JoyCaps>() as u32,
                    )
                };
                (answered == orb_api::joyerr::NOERROR).then_some(caps)
            }
            false => orb_api::joystick::caps(0),
        };
        if let Some(caps) = caps {
            self.image.the_joysticks_caps_are(&caps);
        }
    }

    /// **Pushes the DirectInput pad this file holds**, which the game's own read answers from on its next frame.
    pub fn pushes_its_directinput_pad(&self, pad: DirectInputPad) {
        self.directinput_pad.set(pad);
    }

    /// **What the game's own read made of the pads on its last read**, the keyboard's word left out.
    pub fn the_pads_share_of_the_read(&self) -> u32 {
        self.pads_share.get()
    }

    /// **The game bringing the window it plays in to the front**, as WinMain does before its loop:
    /// `SetForegroundWindow([0x4c3d88])` through the import at `0x48b234` (0x4457d8).
    pub fn brings_its_window_forward(&self, window: orb_api::Hwnd) -> i32 {
        unsafe { orb_core::window::set_foreground_window(window.0 as *mut c_void) }
    }

    /// **The game making its Direct3D object and its device**, as WinMain and `0x446d30` do: `Direct3DCreate9(0x20)`
    /// through its import (0x44570e), and `CreateDevice` through the object's own vtable, `+0x40`, on the window with
    /// `D3DCREATE_HARDWARE_VERTEXPROCESSING` (0x446f5d) and its present parameters — the ones copied to `0x4c3374`,
    /// which every `Reset` is handed.
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

    pub fn the_device_was_made_with(&self) -> Option<u32> {
        self.device_made_with.get()
    }

    pub fn the_device_presents_with(&self) -> Option<u32> {
        self.device_presents_with.get()
    }

    /// This game's memory, for an e2e test that reads back what a frame of orb's did to it.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// **The game making the window it plays in**, as 0x446ae0 does once its class is registered: one
    /// `CreateWindowExA` on `BASE`, by the configuration's window size — and with nothing of orb's in front of that
    /// read, `Th11::hooks` declaring no `create_window`.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        let ask = match self.image.the_window_size() {
            0 => ASKED_FOR_FULL_SCREEN,
            size => {
                let (width, height) =
                    THE_WINDOW_SIZES[usize::from(size - 1).min(THE_WINDOW_SIZES.len() - 1)];
                Ask {
                    style: A_WINDOWS_STYLE,
                    at: (i32::MIN, i32::MIN),
                    size: (
                        width + 2 * FIXED_FRAME.0,
                        height + 2 * FIXED_FRAME.1 + CAPTION,
                    ),
                }
            }
        };
        let window = unsafe {
            orb_core::window::create_window_ex_a(
                0,
                WINDOW_CLASS.as_ptr().cast(),
                c"th11".as_ptr().cast(),
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

    /// **The device made, as `0x446d30` makes it** — WinMain calls it (0x445750) before its message loop.
    pub fn makes_its_device(&self) {
        self.image.shows_through(self.launch.device(), WINDOW);
    }

    /// The state orb reads of this game, for an e2e test that asks what a frame left.
    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th11.read_state() }
    }

    pub fn times_the_fog_was_put_out(&self) -> u32 {
        self.fog_told.get()
    }

    /// How many times orb made the options for the power through the game's own call, which only a stage's numbers
    /// put back does.
    pub fn times_the_options_were_made(&self) -> u32 {
        self.options_made.get()
    }

    pub fn times_orb_stepped_the_shot(&self) -> u32 {
        self.shot_stepped.get()
    }

    pub fn the_shot_timer(&self) -> i32 {
        self.image.the_shot_timer()
    }

    pub fn times_the_run_was_unpaused(&self) -> u32 {
        self.unpaused.get()
    }

    /// **How many updates the stage's `.std` stepped its script on** — which a stage begun out of a fade holds back
    /// on its first thirty and on the stage's own first.
    pub fn times_the_background_stepped(&self) -> u32 {
        self.background_stepped.get()
    }

    /// **The game object's flags and the stage's frames as the replay's seam saw them**, once for every build that
    /// reached it.
    pub fn what_the_build_seeded_on(&self) -> Vec<(u32, u32)> {
        self.seeded_on.borrow().clone()
    }

    /// **The title up on its main menu and reading a key**: the scene the supervisor has built it on, the first title
    /// passed, and the main menu past the frames it ignores a key for.
    pub fn its_title_menu_is_up(&self) {
        // **The score object the loading reads before any title comes up** (0x428b28 into 0x437800), where the
        // title is placed ahead of the frame the loading's job would have read it on and nothing has put one in.
        if self.frames_since_launch.get() < TITLE_COMES_UP_AFTER && !self.read_a_score_file.get() {
            self.the_loading_reads_the_score_object();
        }
        self.frames_since_launch.set(TITLE_COMES_UP_AFTER + 1);
        self.image.is_in_scene(Scene::Title, Scene::Title);
        self.image.the_title_is_built();
        self.image.the_title_is_to_come_up(1);
        self.image.the_title_is_on(Screen::MainMenu);
        self.the_main_menu_is_built();
        self.image.the_titles_step_is(STEP_READING);
        for _ in 0..=THE_MAIN_MENU_READS_PAST {
            self.image.the_title_counts_a_frame();
        }
    }

    /// **A run in its first stage, placed rather than walked to**: what the partner select writes for a Normal run of
    /// the first shot type, the switch's build of it, and the build run to its end — its songs' commands carried out
    /// as the supervisor carries them — so the next frame is the stage's first.
    pub fn is_in_a_run(&self) {
        if !self.image.the_title_is_up() {
            self.its_title_menu_is_up();
        }
        self.image.the_difficulty_is(DIFFICULTY_NORMAL);
        self.image.the_character_is(0);
        self.image.the_shot_is(0);
        self.image.the_run_starts_at_stage_1();
        self.the_run_placed_is_built();
    }

    /// **And a run of the Extra placed the same way**: what the partner select writes for difficulty 4 (0x43d58d
    /// through 0x43d5fc), its stage words at 7.
    pub fn is_in_the_extra(&self) {
        if !self.image.the_title_is_up() {
            self.its_title_menu_is_up();
        }
        self.image.the_difficulty_is(DIFFICULTY_EXTRA);
        self.image.the_character_is(0);
        self.image.the_shot_is(0);
        self.image.the_run_starts_at_the_extra();
        self.the_run_placed_is_built();
    }

    fn the_run_placed_is_built(&self) {
        self.image.asks_for_the_scene(Scene::Run);
        self.switches_the_scene();
        while self.building.get() != Build::None {
            self.its_sound_goes_on();
            self.its_loader_goes_on();
        }
    }

    /// **The player hit**, the state and its timer 0x432a90 writes — and not the six frames nothing can hit it for
    /// that it writes as well — for an e2e test over what orb reads of a hit.
    pub fn the_player_is_hit(&self) {
        self.image.the_player_is_in(Player::Hit);
    }

    /// **A bomb let go**, which is what the player's own bomb leaves behind.
    pub fn a_bomb_goes_off(&self) {
        self.image.a_bomb_is_going_off();
    }

    pub fn the_bomb_is_over(&self) {
        self.image.no_bomb_is_going_off();
    }

    /// **The run's stage played by this file's own script from here on** — see [`FIRST_BULLET_AT`].
    pub fn plays_its_own_stage(&self) {
        self.on_its_own.set(true);
    }

    pub fn stages_last(&self, frames: u32) {
        self.stage_lasts.set(Some(frames));
    }

    pub fn fires_at_the_player(&self, fires: bool) {
        self.fires.set(fires);
    }

    /// **The word the pad's half of the read answered on the last update it made**.
    pub fn word_the_input_read_answered(&self) -> u32 {
        self.word.get()
    }

    /// **The two score files a launch finds beside the exe, and the score object the startup read made of the
    /// game's own**: `scoreth11.dat` (`0x4957b8`) holding `own` and orb's fork of it holding `modes`.
    pub fn its_score_files_hold(&self, own: &score_file::Records, modes: &score_file::Records) {
        let files = self.image.sim().files();
        files.put(THE_GAMES_SCORE_FILE, score_file::encode(own));
        files.put(THE_MODES_SCORE_FILE, score_file::encode(modes));
        self.image.the_score_object_is_read_from(own);
        self.read_a_score_file.set(true);
    }

    /// **The score object read at the startup**, 0x437800 called last by the loading (0x428b28): `scoreth11.dat` out
    /// of the directory, or the records 0x437550 and 0x437660 make empty where it does not read.
    fn the_loading_reads_the_score_object(&self) {
        let file = self.image.sim().files().get(THE_GAMES_SCORE_FILE);
        self.read_a_score_file.set(file.is_some());
        let own = file
            .and_then(|file| score_file::decode(&file))
            .unwrap_or_default();
        self.image.the_score_object_is_read_from(&own);
    }

    /// **The score object written back to `scoreth11.dat`, 0x437a70** — the game object's teardown makes it first
    /// (0x41fe17), and the loading object's teardown as the game leaves (0x428d81). Nothing where the startup read
    /// found no file; otherwise packed and opened by the game's own name through whatever stands in front of its
    /// `CreateFileA`, the bytes going where that open landed.
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

    /// **Another stage than the one asked for built**, which a resume must not write a run over.
    pub fn builds_the_stage_instead(&self, stage: i32) {
        self.image.the_stage_word_is(stage + 1);
    }

    /// **And the stage asked for built inside the attract demo**.
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

    /// **The stage after this one asked for**, which a stage's end writes: the stage stepped on and the scene wanted
    /// at 12 (0x41eda3).
    pub fn moves_on_to_the_next_stage(&self) {
        self.image.asks_for_the_scene(Scene::NextStage);
        self.image.the_run_moves_on_a_stage();
    }

    /// **The calc walk, 0x456cb0**: every job of the chain in its priority order — the supervisor at 1, the ending,
    /// the title at 6, the pause object at 9, the game object at `0xa`, the replay at `0xb`, the `.std` at `0xc`, the
    /// player at `0x10`, the enemy manager at `0x12`, the bullets at `0x14` and the card at `0x16` — and what it answers: the count of the
    /// jobs it reached, `0` where the supervisor answered 4 (the game leaving), and `1` where the game object answered
    /// 3 (paused).
    fn walks_its_calc_list(&self) -> i32 {
        if self.stands_spinning.get() {
            return 1;
        }
        let mut reached = 1;
        if !self.its_supervisor() {
            return 0;
        }
        if self.stands_spinning.get() {
            return reached;
        }
        // **The ending's job**, 0x40f120, which asks for the title after it once its last line is pressed past.
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
        if self.the_stage_is_on.get() && self.image.the_pause_is_up() {
            reached += 1;
            self.its_pause_object();
        }
        if !self.image.the_game_object_is_up() {
            return reached;
        }
        reached += 1;
        // **The game object's job answering 1 having stepped nothing while its build runs** (0x4206c4 through
        // 0x4206de).
        if self.building.get() != Build::None || self.image.the_stage_is_being_built() {
            self.image.the_game_object_waits_on_its_build();
            return reached;
        }
        if !self.its_game_object() {
            return 1;
        }
        if !self.the_stage_is_on.get() {
            return reached;
        }
        reached += 1;
        self.image.the_run_copies_the_word();
        reached += 1;
        self.its_background();
        reached += 1;
        self.its_player();
        reached += 1;
        self.its_enemy_manager();
        // Last of each enemy's update, after the next attack it may have been handed (0x412756 after 0x417b40).
        self.image.the_enemies_attacks_go_on();
        reached += 1;
        self.its_bullets();
        reached += 1;
        self.image.the_card_runs_a_frame();
        // **The count towards the ending**, from a stage 6 won (0x41ecb0) to the scene wanted at 15 (0x41b430).
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

    /// **The `.std`'s calc job, 0x403900 into 0x402aa0** (priority `0xc`): its script stepped where its fade lets it.
    fn its_background(&self) {
        if self.image.the_background_steps_its_script() {
            self.background_stepped
                .set(self.background_stepped.get() + 1);
        }
    }

    /// **The enemy manager's calc job** (0x4111a0, priority `0x12`): every enemy's scripts a step on, each body tested
    /// against the player, and the stage's own script where this file plays one.
    fn its_enemy_manager(&self) {
        self.image
            .the_enemies_scripts_step(matches!(self.fight.get(), Fight::Up { boss: true, .. }));
        self.its_enemies_bodies_touch_the_player();
        // **The generator drawn from once a frame**, which the enemies' scripts do on every stage: how often is this
        // file's own, what it has to be is a word that moves with the stage. **And not over the fade a stage after
        // one won comes in under**: its `main` is spawned on the fade's frame 30 and not before (0x4205b1, 0x4205d2),
        // so no script of the stage's runs to draw while the stage before fades out.
        if !self.image.the_stage_before_fades_out() {
            self.image.the_generator_draws();
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

    /// **Whether a test of the player's answers at all**: nothing while a conversation is up, while the state is 2, 3
    /// or 4, or while every test refuses the player (0x431e90 through 0x431ece).
    fn the_player_can_be_touched(&self) -> bool {
        let image = &self.image;
        let (state, _) = image.what_the_player_is_doing();
        [Player::Playing as i32, Player::ComingBackIn as i32].contains(&state)
            && !image.a_conversation_stands()
            && !image.the_player_is_untouchable()
    }

    /// **Each enemy's body tested against the player**, through the circle test (0x411e48 into 0x431f70): `d² < h² +
    /// r²`, `h` the `.sht`'s whole hitbox and `r` half the body — and the hit taken where nothing holds the player
    /// unhittable (0x432049 through 0x432059).
    fn its_enemies_bodies_touch_the_player(&self) {
        let image = &self.image;
        if !self.the_player_can_be_touched() {
            return;
        }
        let player = image.where_the_player_is();
        let whole = image.the_players_whole_hitbox();
        let touches = image
            .the_enemies_on_the_list()
            .into_iter()
            .filter_map(|enemy| image.the_body_of(enemy))
            .any(|((x, y), radius)| {
                let (dx, dy) = (x - player.0, y - player.1);
                dx * dx + dy * dy < whole * whole + radius * radius
            });
        if touches && image.the_frames_nothing_hits_the_player_for() < 1 {
            image.the_player_takes_the_hit();
        }
    }

    /// **The stage's fight, by the stage's own frame** — th10's fake's, its song played into slot 1 as the
    /// conversation before th11's boss plays it.
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
                // 0x42a150(1, [rec+0x38]) from the conversation's script (0x41e9cf through 0x41e9dd).
                image.a_sound_command_is_queued_with(A_PLAY, THE_FIGHTS_SLOT, b"");
                image.a_fight_is_up(A_FIGHT_STANDS_AT, A_FIGHTS_LIFE);
                the_attack(true, 0);
            }
            Fight::Up {
                boss: false,
                attack,
            } => {
                if frame == THE_MIDBOSS_ARRIVES_AT + (attack + 1) * AN_ATTACK_LASTS {
                    match attack {
                        0 => the_attack(false, 1),
                        _ => the_fights_over(false),
                    }
                }
            }
            Fight::Up { boss: true, attack } => {
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

    /// **A bullet aimed at the player**, from the fight where one is up and from the top of the field otherwise.
    fn fires_a_bullet(&self) {
        let image = &self.image;
        let Some(slot) = (0..BULLET_SLOTS_USED).find(|slot| {
            !image.the_bullet_is_active(*slot) && !image.the_bullet_is_cancelled(*slot)
        }) else {
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

    /// **The bullets' calc job** (0x408d00, priority `0x14`): each active one a step on, freed off the field, and
    /// tested against the player with the box test (0x408bb0 into 0x431e00) — **which answers 1 on a touch whether or
    /// not the hit is taken**, the bullet cancelled either way (0x408bda) and the hit taken only where nothing holds
    /// the player unhittable (0x431ed4 through 0x431ee4). A bullet cancelled is freed on the next walk — the frames of
    /// its cancel are not played. And the count of those left.
    fn its_bullets(&self) {
        let image = &self.image;
        let mut left = 0;
        let touchable = self.the_player_can_be_touched();
        let player = image.where_the_player_is();
        let half = image.the_players_half_box();
        for slot in 0..BULLET_SLOTS_USED {
            if image.the_bullet_is_cancelled(slot) {
                image.the_bullet_is_gone(slot);
                continue;
            }
            if !image.the_bullet_is_active(slot) {
                continue;
            }
            image.the_bullet_moves(slot);
            let ((x, y), (w, h)) = image.where_the_bullet_is(slot);
            if x.abs() > THE_FIELD_ACROSS || y < THE_FIELD_DOWN.0 || y > THE_FIELD_DOWN.1 {
                image.the_bullet_is_gone(slot);
                continue;
            }
            if touchable
                && (x - player.0).abs() < w / 2.0 + half
                && (y - player.1).abs() < h / 2.0 + half
            {
                if image.the_frames_nothing_hits_the_player_for() < 1 {
                    image.the_player_takes_the_hit();
                }
                image.the_bullet_is_cancelled_now(slot);
                continue;
            }
            left += 1;
        }
        image.the_bullet_manager_counts(left);
    }

    /// **The stage's end**, which the stage's script carries out: a practice ends its run cleared (0x41eba3), the last
    /// stage has the panel hold the shot and count towards the ending (0x41ecb0), the Extra ends its run cleared
    /// (0x41ed54), and any other asks for the stage after it (0x41eda3). Every stage won not in a replay offers its
    /// practice first (0x41eb39 through 0x41eb92).
    fn the_stage_ends(&self) {
        let image = &self.image;
        let (score, power, lives) = image.the_runs_numbers();
        image.the_run_has(score + A_STAGE_WON_SCORES, power, lives);
        image.a_stage_won_offers_its_practice();
        if self.state().practice {
            image.the_run_is_over_cleared();
            return;
        }
        match image.the_stage_word() {
            THE_LAST_STAGE => {
                image.the_panel_holds_the_shot();
                self.towards_the_ending.set(Some(0));
            }
            THE_EXTRAS_STAGE => image.the_run_is_over_cleared(),
            _ => self.moves_on_to_the_next_stage(),
        }
    }

    /// **The sound's commands carried out once**, 0x44a340, and the stream put on the record a play reached.
    fn its_sound_goes_on(&self) {
        if let Some(track) = self.image.the_head_of_the_sound_queue_is_carried_out() {
            self.image.the_stream_plays_the_track(track);
        }
    }

    /// **The supervisor's calc job, 0x429030**: the sound's commands carried out (0x429058), the keys read
    /// (0x429062), and the scene switched (0x4290cd). Answers false where the switch answered 4, the game leaving.
    fn its_supervisor(&self) -> bool {
        let frames = self.frames_since_launch.get() + 1;
        self.frames_since_launch.set(frames);
        // **The time an e2e test says this frame's sounds cost**, spent inside the walk where this game spends it.
        self.sim().clock().advance_micros(self.sound_this_frame());
        self.its_sound_goes_on();
        // **The loading object's job**, which wants the title once the start-up's files are in (0x428f10) — the score
        // object the last of them, read out of `scoreth11.dat` (0x428b28 into 0x437800).
        if frames == TITLE_COMES_UP_AFTER && self.image.the_scene_now() == Scene::Loading as i32 {
            self.the_loading_reads_the_score_object();
            self.image.asks_for_the_scene(Scene::Title);
        }
        if frames == THE_LOADING_SCREENS_FILE_GOES_AT
            && self.the_title_came_out_of_the_loading.get()
        {
            self.the_titles_thread_unloads_the_loading_screens_file();
        }
        let keys = self.the_keyboards_word();
        // **The pad's half, through orb's hook where a launch installs it** — `mov ecx,esi; call 0x457270` at
        // 0x4579eb, the keyboard's word in `ecx` and nothing pushed.
        let word = match self.reads_the_keys {
            true => orb_core::runtime::get_input_with_the_keys_in_ecx(keys),
            false => pad_half(keys),
        };
        self.word.set(word);
        self.image.the_keys_read(word);
        if self.image.the_scene_now() != self.image.the_scene_wanted() {
            self.switches_the_scene();
        }
        !self.leaving.get()
    }

    /// **The title's thread let go of the loading screen's file** (0x439564), through orb's hook where a launch
    /// installs it — and the walk this is asked on going on beside the thread, the loading screen's two VMs of that
    /// file run after the supervisor's job: a file gone inside the walk is a walk that stands spinning.
    fn the_titles_thread_unloads_the_loading_screens_file(&self) {
        let through_orb = self.unloads_through_orb;
        let installed = orb_api::installed();
        std::thread::spawn(move || {
            let _installed = installed.as_ref().map(orb_api::install);
            match through_orb {
                true => unsafe { orb_core::runtime::unloads_in_edi() },
                false => the_loading_screens_file_is_unloaded(),
            }
        });
        self.the_unload_is_under_way.set(true);
        self.the_titles_thread_is_given_a_while();
    }

    /// **The title's thread given a while inside a walk or a present**, its unload pending: the loading screen's file
    /// gone in it is a VM's script freed under the walk that runs it, or a texture released under the present the
    /// driver is making of it — the walk spinning on an instruction of size nought, or `Present` waiting inside
    /// `nvd3dum.dll` (`orb-real` 1791788493, launch 3) — which this file has as the walks standing spinning. The file
    /// gone between the two is the unload the frame was kept apart from.
    fn the_titles_thread_is_given_a_while(&self) {
        if !self.the_unload_is_under_way.get() {
            return;
        }
        if !THE_LOADING_SCREENS_FILE_IS_IN.load(Ordering::Relaxed) {
            self.the_unload_is_under_way.set(false);
            return;
        }
        let given = std::time::Instant::now();
        while THE_LOADING_SCREENS_FILE_IS_IN.load(Ordering::Relaxed)
            && given.elapsed() < THE_TITLES_THREAD_IS_GIVEN
        {
            std::thread::yield_now();
        }
        if !THE_LOADING_SCREENS_FILE_IS_IN.load(Ordering::Relaxed) {
            self.stands_spinning.set(true);
        }
    }

    /// **The title's thread decoding `title.anm` and the loading's thread the score file, beside each other** — through
    /// orb's hook where a launch installs it — as the first title is built (`title.anm` at 0x439660 on the thread
    /// the title starts, the score's 0x437952 on the loading object's).
    fn the_title_and_the_loading_decode_beside_each_other(&self) {
        let through_orb = self.decodes_through_orb;
        let decodes: extern "stdcall" fn(usize, usize, usize) -> u32 = match through_orb {
            true => unsafe {
                std::mem::transmute::<
                    unsafe extern "C" fn(),
                    extern "stdcall" fn(usize, usize, usize) -> u32,
                >(orb_core::runtime::decodes_alone)
            },
            false => decode,
        };
        let installed = orb_api::installed();
        let the_titles = {
            let installed = installed.clone();
            std::thread::spawn(move || {
                let _installed = installed.as_ref().map(orb_api::install);
                let mut out = vec![0u8; TITLE_ANM_SCRIPT_0.len()];
                decodes(
                    TITLE_ANM_SCRIPT_0.as_ptr() as usize,
                    TITLE_ANM_SCRIPT_0.len(),
                    out.as_mut_ptr() as usize,
                );
                *TITLE_ANM_DECODED
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = out;
            })
        };
        let the_loadings = std::thread::spawn(move || {
            let _installed = installed.as_ref().map(orb_api::install);
            let given = std::time::Instant::now();
            while !A_DECODE_IS_UNDER_WAY.load(Ordering::Relaxed) && given.elapsed() < A_DECODE_LASTS
            {
                std::thread::yield_now();
            }
            let mut out = vec![0u8; THE_SCORE_FILES_BYTES.len()];
            decodes(
                THE_SCORE_FILES_BYTES.as_ptr() as usize,
                THE_SCORE_FILES_BYTES.len(),
                out.as_mut_ptr() as usize,
            );
        });
        self.decoding
            .borrow_mut()
            .extend([the_titles, the_loadings]);
    }

    /// **The main menu's VM of `title.anm`'s script 0 made and run as it is made** (0x43a1d4): a script whose first
    /// instruction is not the archive's is one whose length is nought where it copied noughts, and the walk stands
    /// spinning on it. Nothing where no thread decoded — a title placed rather than walked to.
    fn the_main_menu_makes_its_vm(&self) {
        let threads = std::mem::take(&mut *self.decoding.borrow_mut());
        if threads.is_empty() {
            return;
        }
        for thread in threads {
            thread
                .join()
                .expect("a decoding thread of the laid-out game panicked");
        }
        let decoded = TITLE_ANM_DECODED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if decoded != TITLE_ANM_SCRIPT_0 {
            self.stands_spinning.set(true);
        }
    }

    /// **The keyboard folded into one word**, the keyboard's half of 0x4576b0 — see [`MAP`] — read only while
    /// `[0x4c3d94]`, the window's `WM_ACTIVATEAPP`, is set (0x4576cd), which is the window in front; through the
    /// device the input's set-up made wherever bit `0x400` of its flags is set (0x4576d3) and `GetKeyboardState`
    /// (0x4576e8) where it is clear. **The device is the system's to take away**, held `DISCL_FOREGROUND` (0x447b7b):
    /// gone while the window is behind, so the first read in front again fails, and the read's answer to that is a
    /// word of nought and one `Acquire` (0x457864 through 0x45787e).
    fn the_keyboards_word(&self) -> u32 {
        if self.sim().display().foreground() != WINDOW {
            KEYBOARD_ACQUIRED.store(false, Ordering::Relaxed);
            return 0;
        }
        let keyboard = self.sim().keyboard();
        let state = keyboard.state();
        let through_the_device = self.image.reads_its_keyboard_through_directinput();
        if through_the_device && !KEYBOARD_ACQUIRED.swap(true, Ordering::Relaxed) {
            return 0;
        }
        MAP.iter()
            .filter(|(key, _)| match through_the_device {
                true => keyboard.held(*key),
                false => state
                    .as_ref()
                    .is_some_and(|state| state[*key as usize] & 0x80 != 0),
            })
            .fold(0, |word, (_, bit)| word | bit)
    }

    /// **The scene switch, 0x42b0e0**, for the scenes this file builds (the table at `0x42b35c`), and then the scene it
    /// is on is the scene it wanted (0x42b1bb, 0x42b1c1). A scene this file builds nothing for stands asked for.
    fn switches_the_scene(&self) {
        let now = self.image.the_scene_now();
        let wanted = self.image.the_scene_wanted();
        match wanted {
            w if w == Scene::Startup as i32 => self.image.asks_for_the_scene(Scene::Loading),
            // 0x42b177: `0x429830` tears every object down, the loading object's teardown writing the score object
            // (0x428d81) — and the switch answers 4, which leaves the frame and the game with it.
            w if w == Scene::TheGameLeaves as i32 => {
                if self.image.the_game_object_is_up() {
                    self.the_run_is_taken_down();
                }
                self.image.the_title_is_freed();
                self.writes_its_score_file();
                self.leaving.set(true);
                return;
            }
            // 0x42b18d: from a run the game object goes first; from the ending the ending goes; from the loading
            // nothing; and from any other scene **the title is not built** (the byte table at `0x42b3b4`).
            w if w == Scene::Title as i32 => {
                let builds = match now {
                    n if n == Scene::Run as i32 => {
                        self.the_run_is_taken_down();
                        true
                    }
                    n if n == Scene::Ending as i32 || n == Scene::Loading as i32 || n == 2 => true,
                    _ => false,
                };
                if builds {
                    if now == Scene::Loading as i32 {
                        self.the_title_came_out_of_the_loading.set(true);
                        self.the_title_and_the_loading_decode_beside_each_other();
                    }
                    self.image.the_title_is_built();
                    self.image.the_title_is_on(Screen::Entry);
                }
            }
            // 0x42b220: the title torn down where it came from 4, `+0x564` at 1, and the game object built by
            // `0x4202d0(0)`.
            w if w == Scene::Run as i32 => {
                if now == Scene::Title as i32 {
                    self.image.the_title_is_freed();
                }
                self.image.the_runs_numbers_are_started_afresh(true);
                self.builds_a_game_object(false);
            }
            // 0x42b29f: the game object torn down, `+0x564` at 1, 7 wanted, the stage put back to the one the run started
            // at, and `0x4202d0(0)`.
            w if w == Scene::StartOver as i32 => {
                self.writes_its_score_file();
                self.the_stage_is_on.set(false);
                self.building.set(Build::None);
                self.image.the_run_is_taken_down_to_start_over();
                self.image.the_runs_numbers_are_started_afresh(true);
                self.image.asks_for_the_scene(Scene::Run);
                self.builds_a_game_object(false);
            }
            // 0x42b269: `+0x564` at nought, the game object torn down where it came from 7 — the score object written
            // first (0x41fe17) — 7 wanted, and `0x4202d0` with the replay's flag carried.
            w if w == Scene::NextStage as i32 => {
                let replaying = self.image.the_run_plays_a_replay();
                self.image.the_runs_numbers_are_started_afresh(false);
                if now == Scene::Run as i32 {
                    self.writes_its_score_file();
                    self.image.the_run_is_taken_down_for_the_next_stage();
                }
                self.image.asks_for_the_scene(Scene::Run);
                self.builds_a_game_object(replaying);
            }
            // 0x42b30e: the game object torn down, `+0x564` at 1, 7 wanted, and `0x4202d0(0)` — the stage left where it is.
            w if w == Scene::Continue as i32 => {
                self.writes_its_score_file();
                self.the_stage_is_on.set(false);
                self.building.set(Build::None);
                self.image.the_run_is_taken_down_for_a_continue();
                self.image.the_runs_numbers_are_started_afresh(true);
                self.image.asks_for_the_scene(Scene::Run);
                self.builds_a_game_object(false);
            }
            // 0x42b32f: the game object torn down where it came from 7, and the ending built, whose set-up writes the
            // ending seen into the score object (0x40ec90).
            w if w == Scene::Ending as i32 => {
                if now == Scene::Run as i32 {
                    self.the_run_is_taken_down();
                }
                self.image.the_ending_writes_its_clear();
                self.the_ending_has_run_for.set(0);
            }
            // 0x42b1e9: from 7 the game object goes, from 15 the ending; 4 wanted, `[0x4c3250] = 3`, and the title built.
            w if w == Scene::AfterTheEnding as i32 => {
                if now == Scene::Run as i32 {
                    self.the_run_is_taken_down();
                }
                self.image.asks_for_the_scene(Scene::Title);
                self.image.the_title_is_to_come_up(3);
                self.image.the_title_is_built();
                self.image.the_title_is_on(Screen::Entry);
            }
            _ => return,
        }
        self.image.the_switch_is_on_the_scene_it_wanted();
    }

    /// **`0x4202d0`**: the game object built with its argument and its build thread started.
    fn builds_a_game_object(&self, replaying: bool) {
        self.image.the_game_object_is_built(replaying);
        self.building.set(Build::Running(A_BUILD_RUNS_FOR));
    }

    /// **The game object's teardown, 0x41fe10**, which writes the score object first (0x41fe17).
    fn the_run_is_taken_down(&self) {
        self.writes_its_score_file();
        self.the_stage_is_on.set(false);
        self.building.set(Build::None);
        self.image.the_run_is_taken_down();
    }

    /// **The stage's build, 0x41f8f0, on its thread**: nothing a frame of it the main thread can see but the game
    /// object's flag, and on its last the whole of it — the run's numbers where `+0x564` is set (0x41f932 through
    /// 0x41fb01), the replay through its seam (0x41fc2b) where the run's managers are not kept and through 0x436f30
    /// where they are (0x41fc98), the stage's objects, and the stage's two songs queued (0x41fd45 through 0x41fd68) —
    /// and then `Sleep(16)` until the sound's queue is empty (0x41fd87 through 0x41fd9e) before the flag comes off
    /// (0x41fdbe).
    ///
    /// **A frame of it goes by between two of the main thread's frames**, as the present hands one over — th10's
    /// reason.
    fn its_loader_goes_on(&self) {
        match self.building.get() {
            Build::None => {}
            Build::Running(left) if left > 1 => self.building.set(Build::Running(left - 1)),
            Build::Running(_) => {
                self.fight.set(Fight::None);
                let replaying = self.image.the_run_plays_a_replay();
                if self.image.the_run_is_fresh() {
                    self.image.the_builds_defaults_are_in();
                    // 0x41fa99: the options made for the power, the game's own call on the player it built.
                    self.image.the_options_are_made();
                }
                if self.image.the_runs_flags() & THE_RUN_KEEPS_ITS_MANAGERS == 0 {
                    match the_game_this_is().hooks().stage_building.is_some() {
                        true => orb_core::runtime::stage_building_of_one(u32::from(replaying)),
                        false => replay_built(u32::from(replaying)),
                    };
                    self.image.the_stages_objects_are_made();
                } else {
                    // 0x436f30's recording: the seed into a new record and the draws from nought (0x436f79, 0x436f8b).
                    self.image.the_generators_draws_start_over();
                    self.image.the_next_stages_objects_are_made();
                }
                self.loads_the_stages_songs();
                self.building.set(Build::WaitingOnTheSound);
                self.its_loader_goes_on();
            }
            Build::WaitingOnTheSound => {
                if !self.image.the_sound_queue_holds_a_command() {
                    self.building.set(Build::None);
                    self.image.the_build_is_over();
                }
            }
        }
    }

    /// **The stage's two songs loaded**, unless the attract demo (0x41fd3c): the stream stopped (`0x42a240`, command 3
    /// with bit `0x10` of the configuration's `+0x38` clear, as its defaults leave it), and the stage record's two
    /// paths queued into slots 0 and 1 (`0x42a0d0`).
    fn loads_the_stages_songs(&self) {
        let image = &self.image;
        if image.the_run_is_the_demo() {
            return;
        }
        image.a_sound_command_is_queued(A_STOP);
        for (slot, track) in [
            (THE_STAGES_SLOT, THE_STAGES_TRACK),
            (THE_FIGHTS_SLOT, THE_BOSSS_TRACK),
        ] {
            image.a_sound_command_is_queued_with(
                A_SONGS_PATH,
                slot,
                &image.the_path_of_the_track(track),
            );
        }
    }

    /// **The game object's calc job, 0x420390**, once the build has ended: its frame 0 switches the new `.std` on and
    /// either begins the stage (fresh) or starts the two fades and the flag `0x800` (out of a fade); frame 30 of a fade
    /// begins it and starts the timer over; the kept `.std` freed once faded; a pause stops the walk here, answering
    /// false; and every frame past it steps the stage's clocks (0x4207ee through 0x4207fd).
    fn its_game_object(&self) -> bool {
        let image = &self.image;
        match image.the_games_frames() {
            0 if image.the_stage_before_is_kept() && !image.the_stage_before_fades_out() => {
                image.the_background_is_switched_on();
                image.the_backgrounds_fade();
                image.the_stage_before_fades(true);
            }
            0 if !image.the_stage_before_fades_out() => {
                image.the_background_is_switched_on();
                self.begins_the_stage();
                self.the_shot_waits.set(true);
            }
            A_FADE_BEGINS_THE_STAGE_AT if image.the_stage_before_fades_out() => {
                image.the_stage_before_fades(false);
                self.begins_the_stage();
                image.the_games_timer_starts_over();
            }
            _ => {}
        }
        image.the_stage_before_is_freed_once_faded();
        if image.the_game_stands_paused() {
            return false;
        }
        image.a_frame_of_the_stage_goes_by();
        true
    }

    /// **The stage begun** (0x420480 on, or 0x4205bd out of a fade): the player reset and its options made for the
    /// power through 0x432cc0 (0x42fec0), the items zeroed, `main` spawned onto an emptied list, the stage's jobs on,
    /// and its own song played into slot 0 unless the demo (0x420525).
    fn begins_the_stage(&self) {
        self.image.the_stage_begins();
        self.image.the_options_are_made();
        self.the_stage_is_on.set(true);
        self.fight.set(Fight::None);
        if !self.image.the_run_is_the_demo() {
            self.image
                .a_sound_command_is_queued_with(A_PLAY, THE_STAGES_SLOT, b"");
        }
    }

    /// **The pause object's calc job, 0x42c600 into 0x42bf30**, its states by its three handlers — see [`Pause`] — and
    /// its timer one frame on at the end.
    fn its_pause_object(&self) {
        let image = &self.image;
        let (state, frames, cursor) = image.where_the_pause_is();
        let pressed = image.the_buttons_gone_down();
        let moving = image.the_buttons_moving_the_cursor();
        let items = image.the_pauses_items();
        let left_out = image.the_items_the_pause_leaves_out();
        let moved = |by: i32| moved(cursor, by, items, &left_out);
        let moves_its_cursor = || {
            if moving & UP != 0 {
                image.the_pauses_cursor_is(moved(-1), items);
            }
            if moving & DOWN != 0 {
                image.the_pauses_cursor_is(moved(1), items);
            }
        };
        match state {
            // 0x42bf4a: not the demo, the pause key gone down, and the game object 30 frames in — 0x42c620: the state,
            // the game object's flag, the music's command and the speed held back.
            s if s == Pause::Nothing as i32 => {
                if pressed & THE_PAUSE != 0
                    && !image.the_run_is_the_demo()
                    && image.the_games_frames() >= A_PAUSE_OPENS_PAST
                {
                    image.the_pause_is_in(Pause::Opening);
                    image.the_game_is_paused(true);
                    image.a_sound_command_is_queued(THE_PAUSES_COMMAND);
                    image.the_pause_holds_the_speed_back();
                }
            }
            // 0x42c8fa: the menu, four items, the cursor on the resume, the replay's save left out for a replay.
            s if s == Pause::Opening as i32 => {
                if frames >= THE_PAUSE_OPENS_OVER {
                    image.the_pause_is_in(Pause::Menu);
                    image.the_pause_leaves_nothing_out();
                    if image.the_run_plays_a_replay() {
                        image.the_pause_leaves_out(THE_PAUSES_SAVE);
                    }
                    image.the_pauses_cursor_is(THE_PAUSES_RESUME, THE_PAUSES_ITEMS);
                }
            }
            // 0x42c9f2: the arrows, the decide, and the three keys of its own.
            s if s == Pause::Menu as i32 => {
                moves_its_cursor();
                let cursor = image.where_the_pause_is().2;
                let replaying = image.the_run_plays_a_replay();
                let its_own = !image.the_pause_is_a_replays_own();
                let closing_on = if its_own && pressed & THE_PAUSES_START_OVER_KEY != 0 {
                    Some(THE_PAUSES_START_OVER)
                } else if its_own && pressed & THE_PAUSES_TITLE != 0 {
                    Some(THE_PAUSES_WAY_TO_THE_TITLE)
                } else if its_own && pressed & THE_PAUSE != 0 {
                    Some(THE_PAUSES_RESUME)
                } else if pressed & DECIDE != 0 {
                    match cursor {
                        THE_PAUSES_RESUME => Some(THE_PAUSES_RESUME),
                        THE_PAUSES_WAY_TO_THE_TITLE | THE_PAUSES_START_OVER if replaying => {
                            Some(cursor)
                        }
                        // The yes or no the other items ask, which this file does not play.
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
                    image.the_pauses_cursor_is(item, THE_PAUSES_ITEMS);
                }
            }
            // 0x42d460: the item carried out twelve frames on, the state at nought.
            s if s == Pause::Closing as i32 => {
                if frames == THE_PAUSE_CLOSES_OVER {
                    image.the_pause_is_in(Pause::Nothing);
                    match cursor {
                        THE_PAUSES_RESUME => the_resume_is_made(),
                        THE_PAUSES_WAY_TO_THE_TITLE => image.asks_for_the_scene(Scene::Title),
                        THE_PAUSES_START_OVER => match image.the_run_plays_a_replay() {
                            true => image.asks_for_the_scene(Scene::AReplayStartedOver),
                            false => image.asks_for_the_scene(Scene::StartOver),
                        },
                        _ => {}
                    }
                }
            }
            // 0x42da94: the high score; for a run not practised the ranking, and its name entry where the score placed
            // — the menu of four on its first item otherwise.
            s if s == Pause::ALostRunTallies as i32 => {
                if frames == THE_END_TALLIES_FOR {
                    image.the_high_score_is_raised();
                    let practice = self.state().practice;
                    if !practice {
                        image.the_pause_places_the_run(THE_GRID.len() as i32);
                    }
                    if practice || image.the_run_is_out_of_the_ranking() {
                        image.the_pause_is_in(Pause::ALostRunsMenu);
                        image.the_pause_leaves_nothing_out();
                        image.the_pauses_cursor_is(A_LOST_RUNS_CONTINUE, A_LOST_RUNS_ITEMS);
                    } else {
                        image.the_pause_is_in(Pause::ALostRunNamesTheRanking);
                    }
                }
            }
            // 0x42dbaf: the arrows, a decide carrying the item out — the replay's save to its slots — and a cancel to
            // the title's item.
            s if s == Pause::ALostRunsMenu as i32 => {
                moves_its_cursor();
                let cursor = image.where_the_pause_is().2;
                if pressed & DECIDE != 0 {
                    match cursor {
                        A_LOST_RUNS_SAVE => image.the_pause_is_in(Pause::ALostRunSavesAReplay),
                        _ => image.the_pause_is_in(Pause::ALostRunLeaves),
                    }
                } else if pressed & CANCEL != 0 {
                    image.the_pauses_cursor_is(1, items);
                }
            }
            // The replay's slots, a cancel back to the menu (0x42df44) and nothing else played.
            s if s == Pause::ALostRunSavesAReplay as i32 => {
                if frames > THE_ENTRY_READS_PAST && pressed & CANCEL != 0 {
                    image.the_pause_is_in(Pause::ALostRunsMenu);
                }
            }
            s if s == Pause::ALostRunNamesTheRanking as i32 && frames >= THE_ENTRY_READS_PAST => {
                self.names_the_ranking(pressed, Pause::ALostRunsMenu, A_LOST_RUNS_ITEMS);
            }
            // 0x42e582: twelve frames on, state 27 and the item: the continue — or the start-over for a run cleared —
            // the title, and the start-over.
            s if s == Pause::ALostRunLeaves as i32 && frames == THE_END_LEAVES_AFTER => {
                image.the_pause_is_in(Pause::Done);
                match cursor {
                    A_LOST_RUNS_CONTINUE if image.the_run_was_cleared() => {
                        image.asks_for_the_scene(Scene::StartOver)
                    }
                    A_LOST_RUNS_CONTINUE => image.asks_for_the_scene(Scene::Continue),
                    A_LOST_RUNS_START_OVER => image.asks_for_the_scene(Scene::StartOver),
                    _ => image.asks_for_the_scene(Scene::Title),
                }
            }
            // 0x42e774: the high score; a practice's menu of three; otherwise the ranking, its name entry where the
            // score placed, and the lost run's menu of three where it did not.
            s if s == Pause::AWonRunTallies as i32 => {
                if frames == THE_END_TALLIES_FOR {
                    image.the_high_score_is_raised();
                    if self.state().practice {
                        image.the_pause_is_in(Pause::AWonRunsMenu);
                        image.the_pauses_cursor_is(0, A_WON_RUNS_ITEMS);
                    } else {
                        image.a_cleared_extra_is_moved_past();
                        image.the_pause_places_the_run(THE_GRID.len() as i32);
                        if image.the_run_is_out_of_the_ranking() {
                            image.the_pause_is_in(Pause::ALostRunsMenu);
                            image.the_pause_leaves_nothing_out();
                            image.the_pauses_cursor_is(0, A_WON_RUNS_ITEMS);
                        } else {
                            image.the_pause_is_in(Pause::AWonRunNamesTheRanking);
                        }
                    }
                }
            }
            s if s == Pause::AWonRunsMenu as i32 => {
                moves_its_cursor();
                let cursor = image.where_the_pause_is().2;
                if pressed & DECIDE != 0 {
                    match cursor {
                        A_WON_RUNS_SAVE => image.the_pause_is_in(Pause::AWonRunSavesAReplay),
                        _ => image.the_pause_is_in(Pause::AWonRunLeaves),
                    }
                } else if pressed & CANCEL != 0 {
                    image.the_pauses_cursor_is(0, items);
                }
            }
            s if s == Pause::AWonRunSavesAReplay as i32 => {
                if frames > THE_ENTRY_READS_PAST && pressed & CANCEL != 0 {
                    image.the_pause_is_in(Pause::AWonRunsMenu);
                }
            }
            s if s == Pause::AWonRunNamesTheRanking as i32 && frames >= THE_ENTRY_READS_PAST => {
                self.names_the_ranking(pressed, Pause::AWonRunsMenu, A_WON_RUNS_ITEMS);
            }
            // 0x42f2b2: twelve frames on, state 27 and the item: the title, or the start-over.
            s if s == Pause::AWonRunLeaves as i32 && frames == THE_END_LEAVES_AFTER => {
                image.the_pause_is_in(Pause::Done);
                match cursor {
                    A_WON_RUNS_START_OVER => image.asks_for_the_scene(Scene::StartOver),
                    _ => image.asks_for_the_scene(Scene::Title),
                }
            }
            _ => {}
        }
        image.the_pause_counts_a_frame();
    }

    /// **The player's calc job, 0x431c50 into 0x431070**, its states by the switch at 0x43108e — see [`Player`] — the
    /// shot after them (0x431af9 through 0x431b90), and its timers one frame on.
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
                if frames >= A_HIT_WAITS_FOR {
                    image.the_player_dies();
                }
            }
            s if s == Player::Dying as i32 && frames == THE_DYING_TAKES_THE_POWER_ON => {
                image.the_dying_takes_the_power();
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

    /// **The player moved by the run's copy of the word**, 0x430290: whether it is focused — the word's `0x8` while an
    /// enemy is up and the update's timer is 4 or more (0x430368 through 0x43037b) — the direction out of its four
    /// bits, the speed of that direction, and the step times the game's speed truncated to 128ths. **Which of two
    /// opposite directions held together wins was not read for th11**; th10's order is kept.
    fn moves_its_player(&self) {
        let image = &self.image;
        let word = image.the_run_word();
        let focused = image.an_enemy_is_up()
            && image.the_players_update_timer() >= THE_FOCUS_IS_HELD_OFF_FOR
            && word & FOCUS != 0;
        image.the_player_is_focused(focused);
        let holds = |bits: u32| word & bits == bits;
        let (across, down) = [
            (UP | TO_THE_LEFT, (-1, -1)),
            (DOWN | TO_THE_LEFT, (-1, 1)),
            (UP | TO_THE_RIGHT, (1, -1)),
            (DOWN | TO_THE_RIGHT, (1, 1)),
            (DOWN, (0, 1)),
            (UP, (0, -1)),
            (TO_THE_LEFT, (-1, 0)),
            (TO_THE_RIGHT, (1, 0)),
        ]
        .into_iter()
        .find(|(bits, _)| holds(*bits))
        .map_or((0, 0), |(_, direction)| direction);
        // **Carried through Yukari's gap, whatever is held** (0x43038b through 0x4303a8): `0x96` 128ths a frame out of
        // the wall it was taken against, and nothing down.
        let (gap, _) = image.the_players_gap();
        let scaled = |by: i32| (by as f32 * image.the_games_speed()) as i32;
        match gap {
            GAP_OUT_OF_THE_LEFT => image.the_player_moves_by(-scaled(THE_GAP_CARRIES_BY), 0),
            GAP_OUT_OF_THE_RIGHT => image.the_player_moves_by(scaled(THE_GAP_CARRIES_BY), 0),
            _ if across == 0 && down == 0 => {}
            _ => {
                let [straight, focused_straight, diagonal, focused_diagonal] =
                    image.the_players_speeds();
                let speed = match (focused, across != 0 && down != 0) {
                    (false, false) => straight,
                    (true, false) => focused_straight,
                    (false, true) => diagonal,
                    (true, true) => focused_diagonal,
                };
                let step = scaled(speed);
                image.the_player_moves_by(across * step, down * step);
            }
        }
        if image.the_character() * 3 + image.the_shot() == 0 {
            self.goes_on_through_its_gap(word, (across, down));
        }
        image.the_player_is_held_inside_the_field();
    }

    /// **Yukari's gap gone on with** (0x430e50, called for shot type 0 between the step and the clamp, 0x430760): from
    /// against a wall the direction let go, and back into that wall within 8 frames taken through it — nothing of it
    /// with the shot or the focus held — and carried out through it, put across the field past `0x6800`, for 45 frames.
    /// **The two other things it asks before going on, `[[0x4a8d84]+0x4438]` nought and `[[0x4a8d7c]+0x70]` not, are
    /// not read for what they are**, and owed nothing here: a run of the fake has neither up.
    fn goes_on_through_its_gap(&self, word: u32, (across, down): (i32, i32)) {
        let image = &self.image;
        let (state, frames) = image.the_players_gap();
        let toward = |wall: i32| across == wall && down == 0;
        let let_go = across == 0 && down == 0;
        let (state, frames) = match state {
            0 => return,
            _ if state < GAP_OUT_OF_THE_LEFT && word & (SHOT | FOCUS) != 0 => (0, 1),
            GAP_AGAINST_THE_LEFT | GAP_AGAINST_THE_RIGHT if let_go => (state + 1, 1),
            GAP_AGAINST_THE_LEFT if toward(-1) => (state, frames + 1),
            GAP_AGAINST_THE_RIGHT if toward(1) => (state, frames + 1),
            GAP_AGAINST_THE_LEFT | GAP_AGAINST_THE_RIGHT => (0, frames + 1),
            GAP_LET_GO_AT_THE_LEFT | GAP_LET_GO_AT_THE_RIGHT
                if frames > THE_GAP_IS_TAKEN_WITHIN =>
            {
                (0, 1)
            }
            GAP_LET_GO_AT_THE_LEFT if toward(-1) => (GAP_OUT_OF_THE_LEFT, 1),
            GAP_LET_GO_AT_THE_RIGHT if toward(1) => (GAP_OUT_OF_THE_RIGHT, 1),
            GAP_LET_GO_AT_THE_LEFT | GAP_LET_GO_AT_THE_RIGHT => (state, frames + 1),
            GAP_OUT_OF_THE_LEFT | GAP_OUT_OF_THE_RIGHT => {
                let (x, y) = image.where_the_player_is_in_128ths();
                match state {
                    GAP_OUT_OF_THE_LEFT if x <= -THE_GAP_COMES_OUT_PAST => {
                        image.the_player_is_at_128ths(x + THE_GAP_COMES_IN_BY, y);
                    }
                    GAP_OUT_OF_THE_RIGHT if x >= THE_GAP_COMES_OUT_PAST => {
                        image.the_player_is_at_128ths(x - THE_GAP_COMES_IN_BY, y);
                    }
                    _ => {}
                }
                match frames >= THE_GAP_LASTS {
                    true => (0, 1),
                    false => (state, frames + 1),
                }
            }
            _ => (state, frames + 1),
        };
        image.the_players_gap_is(state, frames);
    }

    /// **The title's calc job, 0x439f10 into 0x4399b0**: the screen up by the switch at 0x439a6e, and its frames one on.
    fn walks_its_title(&self) {
        let image = &self.image;
        let (screen, step, cursor, frames) = image.where_the_title_is();
        let pressed = image.the_buttons_gone_down();
        let moving = image.the_buttons_moving_the_cursor();
        let (_, items) = image.the_titles_cursor();
        let left_out = self.left_out.borrow().clone();
        let moved = |by: i32| moved(cursor, by, items, &left_out);
        let arrows = |back: u32, on: u32| {
            if moving & back != 0 {
                image.the_titles_cursor_is(moved(-1), items);
            }
            if moving & on != 0 {
                image.the_titles_cursor_is(moved(1), items);
            }
        };
        match screen {
            // Screen 0's set-up by `[0x4c3250]`: 3 the name entry after the ending, and the main menu otherwise — the
            // cursor on the Extra where the difficulty is the Extra's — every title after the first coming back to it.
            s if s == Screen::Entry as i32 => {
                match image.how_the_title_comes_up() {
                    3 => image.the_title_is_on(Screen::AfterTheEnding),
                    how => {
                        image.the_title_is_on(Screen::MainMenu);
                        if how == 1 && image.the_difficulty() == DIFFICULTY_EXTRA {
                            image.the_titles_cursor_is(THE_ITEM_FOR_THE_EXTRA, MAIN_MENU_ITEMS);
                        }
                    }
                }
                image.the_title_is_to_come_up(1);
            }
            s if s == Screen::MainMenu as i32 => match step {
                STEP_BUILT => {
                    self.the_main_menu_is_built();
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames == THE_MAIN_MENU_MAKES_ITS_VM_AT => {
                    self.the_main_menu_makes_its_vm();
                }
                STEP_WAITING if frames > THE_MAIN_MENU_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => {
                    arrows(UP, DOWN);
                    let (cursor, _) = image.the_titles_cursor();
                    // **A cancel on the last item taken as a decide** (0x43a336, 0x43a4ad through 0x43a4be), and a
                    // cancel elsewhere putting the cursor on the last, the count less one (0x43a34a through 0x43a366).
                    if pressed & CANCEL != 0 && cursor != THE_LAST_ITEM {
                        image.the_titles_cursor_is(THE_LAST_ITEM, MAIN_MENU_ITEMS);
                    } else if pressed & (DECIDE | CANCEL) != 0 {
                        image.the_titles_step_is(STEP_LEFT);
                    }
                }
                STEP_LEFT if frames > THE_MAIN_MENU_ACTS_PAST => match cursor {
                    // 0x43a4f3: practice off, and the difficulty — **with its cursor put on the difficulty the
                    // title keeps on the same update** (0x43a5a2, 0x40df10 at 0x43a5c2), and not on the select's own
                    // first one.
                    THE_ITEM_FOR_A_RUN => {
                        image.the_run_is_no_practice();
                        image.the_title_is_on(Screen::Difficulty);
                        image.the_titles_cursor_is(image.the_titles_difficulty(), MAIN_MENU_ITEMS);
                    }
                    // 0x43a52e through 0x43a539: the difficulty kept, 4 written into the title's own and the run's.
                    THE_ITEM_FOR_THE_EXTRA => {
                        image.the_run_is_no_practice();
                        image.the_title_keeps_the_difficulty();
                        image.the_titles_difficulty_is(DIFFICULTY_EXTRA);
                        image.the_difficulty_is(DIFFICULTY_EXTRA);
                        image.the_title_is_on(Screen::Difficulty);
                    }
                    // 0x43a57e through 0x43a5b8: practice on, and the difficulty the title keeps, 1 where it is the
                    // Extra's.
                    THE_ITEM_FOR_A_PRACTICE => {
                        image.the_run_becomes_a_practice();
                        let kept = image.the_titles_difficulty();
                        let difficulty = match kept > 3 {
                            true => DIFFICULTY_NORMAL,
                            false => kept,
                        };
                        image.the_difficulty_is(difficulty);
                        image.the_title_is_on(Screen::Difficulty);
                        image.the_titles_cursor_is(difficulty, MAIN_MENU_ITEMS);
                    }
                    THE_LAST_ITEM => image.the_title_is_on(Screen::Leaving),
                    // **Only a run, the Extra, a practice and the way out are played of what the menu's items open**.
                    _ => {}
                },
                _ => {}
            },
            // Screen 5, the difficulty: one item over the Extra, the cursor on the difficulty the title keeps
            // otherwise.
            s if s == Screen::Difficulty as i32 => match step {
                STEP_BUILT => {
                    self.left_out.borrow_mut().clear();
                    match image.the_difficulty() >= DIFFICULTY_EXTRA {
                        true => image.the_titles_cursor_is(0, 1),
                        false => image.the_titles_cursor_is(
                            image.the_titles_difficulty().clamp(0, DIFFICULTIES - 1),
                            DIFFICULTIES,
                        ),
                    }
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => self.a_select_reads(pressed, UP, DOWN, arrows),
                // 0x43cb11 through 0x43cb51: the difficulty unless the Extra's, the title's own, and the character with
                // the cursor on the one remembered.
                STEP_DECIDED if frames > A_SELECT_MOVES_ON_PAST => {
                    if image.the_difficulty() < DIFFICULTY_EXTRA {
                        image.the_difficulty_is(cursor);
                        image.the_titles_difficulty_is(cursor);
                    }
                    image.the_title_is_on(Screen::Character);
                    let (character, _) = image.the_selects_remember();
                    image.the_titles_cursor_is(character.clamp(0, CHARACTERS - 1), CHARACTERS);
                }
                STEP_LEFT if frames > A_CANCEL_GOES_BACK_PAST => {
                    image.the_title_is_on(Screen::MainMenu)
                }
                _ => {}
            },
            s if s == Screen::Character as i32 => match step {
                STEP_BUILT => {
                    self.left_out.borrow_mut().clear();
                    image.the_titles_cursor_is(cursor, CHARACTERS);
                    if image.the_difficulty() == DIFFICULTY_EXTRA {
                        self.leaves_out_what_the_extra_is_not_open_for(CHARACTERS, |character| {
                            (0..PARTNERS).any(|partner| {
                                image.the_extra_is_open_for(character * PARTNERS + partner)
                            })
                        });
                    }
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => self.a_select_reads(pressed, TO_THE_LEFT, TO_THE_RIGHT, arrows),
                // 0x43cfb6 through 0x43d036: the character, remembered, and the partner's cursor at nought for a
                // character changed or on the one remembered.
                STEP_DECIDED if frames > A_SELECT_MOVES_ON_PAST => {
                    let before = image.the_character();
                    image.the_character_is(cursor);
                    image.the_character_select_remembers(cursor);
                    image.the_title_is_on(Screen::Partner);
                    let partner = match before == cursor {
                        true => image.the_selects_remember().1,
                        false => {
                            image.the_partner_select_remembers(0);
                            0
                        }
                    };
                    image.the_titles_cursor_is(partner.clamp(0, PARTNERS - 1), PARTNERS);
                }
                STEP_LEFT if frames > A_CANCEL_GOES_BACK_PAST => {
                    image.the_title_is_on(Screen::Difficulty)
                }
                _ => {}
            },
            s if s == Screen::Partner as i32 => match step {
                STEP_BUILT => {
                    self.left_out.borrow_mut().clear();
                    image.the_titles_cursor_is(cursor, PARTNERS);
                    if image.the_difficulty() == DIFFICULTY_EXTRA {
                        let character = image.the_character();
                        self.leaves_out_what_the_extra_is_not_open_for(PARTNERS, |partner| {
                            image.the_extra_is_open_for(character * PARTNERS + partner)
                        });
                    }
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > A_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => self.a_select_reads(pressed, UP, DOWN, arrows),
                // 0x43d553 through 0x43d575: a practice's partner, and its stage select.
                STEP_DECIDED if frames == A_PRACTICE_IS_CHOSEN_AT && self.state().practice => {
                    image.the_shot_is(cursor);
                    image.the_partner_select_remembers(cursor);
                    image.the_title_is_on(Screen::Practice);
                }
                // 0x43d58d through 0x43d5fc: the partner, the title's screen 2, stage 1 or the Extra's by the
                // difficulty, and the run wanted.
                STEP_DECIDED
                    if frames > THE_PARTNER_STARTS_THE_RUN_PAST && !self.state().practice =>
                {
                    image.the_shot_is(cursor);
                    image.the_partner_select_remembers(cursor);
                    image.the_title_is_on(Screen::Leaving);
                    match image.the_difficulty() >= DIFFICULTY_EXTRA {
                        true => image.the_run_starts_at_the_extra(),
                        false => image.the_run_starts_at_stage_1(),
                    }
                    image.asks_for_the_scene(Scene::Run);
                }
                STEP_LEFT if frames > A_CANCEL_GOES_BACK_PAST => {
                    image.the_title_is_on(Screen::Character)
                }
                _ => {}
            },
            s if s == Screen::Practice as i32 => match step {
                // Six stages, the cursor on the first — **whether the select remembers its cursor was not read**.
                STEP_BUILT => {
                    image.the_titles_cursor_is(0, PRACTICE_STAGES);
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > THE_PRACTICE_SELECT_READS_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                STEP_READING => {
                    arrows(UP, DOWN);
                    let (cursor, _) = image.the_titles_cursor();
                    // A cancel back towards the partner, and a decide that takes only a stage the shot type's record
                    // offers (0x43d80a).
                    if pressed & CANCEL != 0 {
                        image.the_titles_step_is(STEP_LEFT);
                    } else if pressed & DECIDE != 0
                        && image.the_practice_is_offered(image.the_difficulty(), cursor)
                    {
                        image.the_titles_step_is(STEP_DECIDED);
                    }
                }
                // 0x43da11 through 0x43da3a: the title on its way out, the stage under the cursor and the run wanted.
                STEP_DECIDED if frames > A_PRACTICE_STARTS_PAST => {
                    image.the_title_is_on(Screen::Leaving);
                    image.the_practice_starts_at(cursor + 1);
                    image.asks_for_the_scene(Scene::Run);
                }
                STEP_LEFT if frames > A_CANCEL_GOES_BACK_PAST => {
                    image.the_title_is_on(Screen::Partner);
                    image.the_titles_cursor_is(image.the_shot().clamp(0, PARTNERS - 1), PARTNERS);
                }
                _ => {}
            },
            // Screen 2: the game's leaving asked for, scene 3 (0x439d61). A run chosen leaves through this screen too,
            // but the switch builds the run and frees the title before the title's job runs on it again.
            s if s == Screen::Leaving as i32 => image.asks_for_the_scene(Scene::TheGameLeaves),
            s if s == Screen::AfterTheEnding as i32 => self.names_the_clear(step, frames, pressed),
            s if s == Screen::TheReplaysSlots as i32 => match step {
                STEP_BUILT => {
                    image.the_titles_cursor_is(0, THE_REPLAYS_SLOTS);
                    image.the_titles_step_is(STEP_WAITING);
                }
                STEP_WAITING if frames > THE_SLOTS_READ_PAST => {
                    image.the_titles_step_is(STEP_READING)
                }
                // The arrows over the slots, and a cancel to step 4. A decide's own name entry is not played.
                STEP_READING => {
                    arrows(UP, DOWN);
                    if pressed & CANCEL != 0 {
                        image.the_titles_step_is(STEP_LEFT);
                    }
                }
                // `FUN_004390e0(1)` past `+0x2b4 > 5`: the main menu.
                STEP_LEFT if frames > A_CANCEL_LEAVES_THE_SLOTS_PAST => {
                    image.the_titles_cursor_is(THE_ITEM_FOR_A_RUN, MAIN_MENU_ITEMS);
                    image.the_title_is_on(Screen::MainMenu);
                }
                _ => {}
            },
            _ => {}
        }
        image.the_title_counts_a_frame();
    }

    /// **The title's name entry for a clear, 0x43f6b0**: its first step placing the clear in the ranking with the
    /// stage at 8 and the name started from the last one typed, or nothing asked where it has no place; its second
    /// reading the grid; and its third, five frames on, the replay's slots.
    fn names_the_clear(&self, step: i32, frames: i32, pressed: u32) {
        let image = &self.image;
        let the_end = THE_GRID.len() as i32 - 1;
        let mut entry = image.the_titles_entry();
        match step {
            STEP_BUILT => {
                image.the_titles_step_is(STEP_WAITING);
                match image.the_title_places_the_clear() {
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
            STEP_WAITING if frames > THE_TITLES_ENTRY_READS_PAST => {
                image.the_titles_step_is(STEP_READING)
            }
            STEP_READING => {
                if pressed & (DECIDE | CANCEL) != 0 && entry.not_asked {
                    image.the_titles_step_is(STEP_DECIDED);
                    return;
                }
                match typed_into(&mut entry, pressed) {
                    Typed::Ended => {
                        let (ranked, _) = image.the_titles_cursor();
                        image.the_ranked_place_is_named(ranked.max(0) as usize, &entry.name);
                        image.the_last_name_typed_is(&entry.name);
                        image.the_titles_entry_is(&entry);
                        image.the_titles_step_is(STEP_DECIDED);
                    }
                    Typed::Going => image.the_titles_entry_is(&entry),
                }
            }
            STEP_DECIDED if frames > THE_TITLES_ENTRY_LEAVES_PAST => {
                image.the_title_is_on(Screen::TheReplaysSlots)
            }
            _ => {}
        }
    }

    /// **A select over the Extra leaving out the items nobody has opened it for**: each item the Extra is not open for
    /// put on the list left out, and the cursor on it moved to the first of the others the Extra is open for.
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
            self.left_out.borrow_mut().push(item);
        }
    }

    /// **The ranking's name entry of a run's end, states 18 and 25** (0x42e299, 0x42efc9): the grid walked, and on its
    /// end the name copied into the ranked place under the pause's cursor and into the last one typed, and the menu
    /// the run ends on put up standing on its first item (0x42e4d4 through 0x42e54f, 0x42f204 through 0x42f27f).
    fn names_the_ranking(&self, pressed: u32, menu: Pause, items: i32) {
        let image = &self.image;
        let mut entry = image.the_pauses_entry();
        match typed_into(&mut entry, pressed) {
            Typed::Ended => {
                let (_, _, ranked) = image.where_the_pause_is();
                image.the_ranked_place_is_named(ranked.max(0) as usize, &entry.name);
                image.the_last_name_typed_is(&entry.name);
                image.the_pauses_entry_is(&entry);
                image.the_pause_is_in(menu);
                image.the_pause_leaves_nothing_out();
                image.the_pauses_cursor_is(0, items);
            }
            Typed::Going => image.the_pauses_entry_is(&entry),
        }
    }

    /// **A select reading its keys**: its two directions moving the cursor, a decide putting it in its step 3 and a
    /// cancel in its step 4.
    fn a_select_reads(&self, pressed: u32, back: u32, on: u32, arrows: impl Fn(u32, u32)) {
        arrows(back, on);
        if pressed & DECIDE != 0 {
            self.image.the_titles_step_is(STEP_DECIDED);
        } else if pressed & CANCEL != 0 {
            self.image.the_titles_step_is(STEP_LEFT);
        }
    }

    /// **The main menu's step 0**: its eight items, the Extra left out where no ending opens it (0x43a0c3 into
    /// 0x439350), and a menu come back to from a practice put on the practice's item with the run's flag for one
    /// taken off — th10's, the same not having been read for th11.
    fn the_main_menu_is_built(&self) {
        let image = &self.image;
        let (cursor, _) = image.the_titles_cursor();
        image.the_titles_cursor_is(cursor, MAIN_MENU_ITEMS);
        let mut left_out = self.left_out.borrow_mut();
        left_out.clear();
        if !image.the_extra_is_open() {
            left_out.push(THE_ITEM_FOR_THE_EXTRA);
        }
        drop(left_out);
        if self.state().practice {
            image.the_titles_cursor_is(THE_ITEM_FOR_A_PRACTICE, MAIN_MENU_ITEMS);
            image.the_run_is_no_practice();
        }
    }

    /// **The draw walk, 0x456e10**: one quad queued, which is what drawing anything in this game is, and the `.std`s'
    /// two draw jobs at 2 and 5 (0x403130, 0x4035a0) — and the count of the jobs it reached.
    fn walks_its_draw_list(&self) -> i32 {
        self.image.queues_a_quad();
        self.image.the_backgrounds_draw();
        1
    }
}

/// **What a press does to a name entry**, the grid's (0x42d078 and its kin): the arrows over the grid; a decide on a
/// character or the space putting it at the place and the place on — the grid's cursor on the end once the name is
/// full, the last character written over where it already is — on the step back taking the last one off, and on the
/// end ending the entry; and a cancel taking the last character off.
enum Typed {
    Going,
    Ended,
}

fn typed_into(entry: &mut TheTitlesEntry, pressed: u32) -> Typed {
    let the_end = THE_GRID.len() as i32 - 1;
    entry.grid = moved_over_the_grid(entry.grid, pressed);
    if pressed & DECIDE != 0 {
        let place = entry.typed.max(0) as usize;
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
            _ => return Typed::Ended,
        }
    } else if pressed & CANCEL != 0 && entry.typed > 0 {
        entry.typed -= 1;
        entry.name[entry.typed as usize] = b' ';
    }
    Typed::Going
}

/// **A cursor moved**: round past either end, and over the items the screen leaves out.
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

/// **The cursor of a name entry's grid moved by the arrows**: up and down a row of [`A_GRIDS_ROW`], left and right a
/// cell round the row's ends, and round the grid past either of its ends — th10's grid's walk, th11's not read.
fn moved_over_the_grid(at: i32, pressed: u32) -> i32 {
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
    /// **The game closing, which takes orb down with it** — the runtime first, then the fields.
    fn drop(&mut self) {
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The exe this game is, which is how `orb_core::game::KNOWN` is asked for it.
pub const EXE: &str = "th11.exe";

/// **The score file's name the game opens**, `0x4957b8`, and the name orb's fork of it is.
pub(crate) const THE_GAMES_SCORE_FILE: &str = "scoreth11.dat";
const THE_MODES_SCORE_FILE: &str = "pointdevice_scoreth11.dat";

/// **The walk from the title to a run of the mode, as a recipe**, which is the real recipes' own — see
/// `orb-real/recipes/th11/a_run_started_and_given_up.yaml` — with the mode's question answered where the main menu's
/// decide puts it, up to the partner select's decide that asks for a Normal run of Reimu and her first partner.
pub(crate) const TO_THE_SHOT: &str = "\
- wait_until: { screen: 1 }
  give_up_after: 3600
- press: [up]
  frames: 10
  until: { screen: 1, cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { question: mode }
  give_up_after: 900
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 5 }
  give_up_after: 1800
- press: [up]
  frames: 10
  until: { screen: 5, cursor: 0 }
  give_up_after: 900
- press: [down]
  frames: 10
  until: { screen: 5, cursor: 1 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { screen: 6 }
  give_up_after: 900
- press: [up]
  frames: 10
  until: { screen: 6, cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 10
  until: { screen: 7 }
  give_up_after: 900
- press: [up]
  frames: 10
  until: { screen: 7, cursor: 0 }
  give_up_after: 900
- bombs: false
";

/// **And the partner select's decide that asks for the run, played up to the run's first frame**.
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

/// **The calling calls a launch of this game leaves as the originals of its two walks and its frame** —
/// `orb_core::runtime::calls_the_calc_chain_in_ebx`, `calls_the_draw_chain_taking_nothing` and
/// `calls_the_frame_on_the_stack` — in the type `Originals` holds an original in.
/// **The `.anm` file's unload, 0x4549e0, as far as this file has one**: the loading screen's file gone, which is what
/// orb's hook over the prologue calls through to.
extern "C" fn the_loading_screens_file_is_unloaded() {
    THE_LOADING_SCREENS_FILE_IS_IN.store(false, Ordering::Relaxed);
}

/// **The LZSS decode, 0x4426c0, as far as this file has one**: the window cleared as the decode starts, the bytes
/// put through it, and copied out — the first decode under way only once another has copied out or
/// [`A_DECODE_LASTS`] has gone, so what that other one did to the window in between is what this one copies out.
/// `stdcall` of three, as orb's hook over the prologue calls through to it.
extern "stdcall" fn decode(source: usize, bytes: usize, out: usize) -> u32 {
    for cell in &THE_WINDOW {
        cell.store(0, Ordering::Relaxed);
    }
    let source = unsafe { std::slice::from_raw_parts(source as *const u8, bytes) };
    for (cell, byte) in THE_WINDOW.iter().zip(source) {
        cell.store(*byte, Ordering::Relaxed);
    }
    let done = DECODES_DONE.load(Ordering::SeqCst);
    if !A_DECODE_IS_UNDER_WAY.swap(true, Ordering::SeqCst) {
        let given = std::time::Instant::now();
        while DECODES_DONE.load(Ordering::SeqCst) == done && given.elapsed() < A_DECODE_LASTS {
            std::thread::yield_now();
        }
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out as *mut u8, bytes) };
    for (byte, cell) in out.iter_mut().zip(&THE_WINDOW) {
        *byte = cell.load(Ordering::Relaxed);
    }
    DECODES_DONE.fetch_add(1, Ordering::SeqCst);
    0
}

extern "fastcall" fn calls_the_calc_walk_in_ebx(chain: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::calls_the_calc_chain_in_ebx(chain) }
}

extern "fastcall" fn calls_the_draw_walk_taking_nothing(chain: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::calls_the_draw_chain_taking_nothing(chain) }
}

extern "fastcall" fn calls_the_frame_on_the_stack(window: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::calls_the_frame_on_the_stack(window) }
}

/// **`0x456cb0`, the calc walk, entered with the chain in `ebx`** and ending in a plain `ret` — moved into the
/// argument of [`update`], which is where a function of this file's own can read it.
///
/// **Naked, because the register is the whole of the call**: the compiler keeps `ebx` for itself, so a function it
/// writes has no way to be entered with an argument there.
#[unsafe(naked)]
unsafe extern "C" fn update_in_ebx() {
    std::arch::naked_asm!("mov ecx, ebx", "jmp {body}", body = sym update)
}

extern "fastcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.walks_its_calc_list()
}

/// `0x456e10`: the draw walk, which takes nothing and reads the chain itself (0x456e25).
extern "C" fn draw() -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.walks_its_draw_list()
}

/// **The calc walk called the way the frame calls it**, `mov ebx,[0x4c3234]; call 0x456cb0` (0x4464d1, 0x4464e1) —
/// which reaches `runtime::run_calc_chain_in_ebx` where orb's hook stands over the walk's prologue.
///
/// **`ebx` saved and put back inside the asm**, which LLVM keeps for itself on x86 — `Th11`'s `with_ebx` has why.
fn the_calc_walk_as_the_frame_calls_it(chain: *mut c_void) -> i32 {
    let answer: i32;
    unsafe {
        std::arch::asm!(
            "push ebx",
            "mov ebx, ecx",
            "call {walk}",
            "pop ebx",
            walk = sym orb_core::runtime::run_calc_chain_in_ebx,
            in("ecx") chain,
            lateout("eax") answer,
            clobber_abi("C"),
        )
    };
    answer
}

/// **And the draw walk**, `call 0x456e10` with nothing (0x446571).
fn the_draw_walk_as_the_frame_calls_it() -> i32 {
    let answer: i32;
    unsafe {
        std::arch::asm!(
            "call {walk}",
            walk = sym orb_core::runtime::run_draw_chain_taking_nothing,
            lateout("eax") answer,
            clobber_abi("C"),
        )
    };
    answer
}

/// **The game's own frame, 0x446420**, `stdcall` on the window — `asm.md`'s *The frame at `0x446420`, in order*, less
/// the limiter and the frame skip, a frame here being one update and one draw: the queue drawn, the camera set with
/// its viewport, the calc walk and its two ways out (0x4464ec, 0x446505), `BeginScene`, the queue put back, the fog
/// forced and set, the draw walk, the queue drawn, `SetTexture(0, null)`, `EndScene` and the present. **Which of the
/// three frames WinMain picks is not played**: all three take the same steps around the walks, and orb's own loop
/// stands in front of each.
extern "stdcall" fn own_render(_window: *mut c_void) -> i32 {
    let fake = running();
    let image = &fake.image;
    let device = fake.launch.device();
    let chain = Th11.chain();
    the_queue_is_drawn(0);
    image.the_frame_names_its_camera();
    the_camera_is_set(image.the_camera());
    orb_api::d3d9::set_viewport(device, image.the_cameras_viewport());
    image.the_frame_has_set_its_camera();
    let walked = the_calc_walk_as_the_frame_calls_it(chain);
    if walked == 0 {
        return 1;
    }
    if walked == -1 {
        return 2;
    }
    orb_api::d3d9::begin_scene(device);
    image.the_queue_is_put_back_at_its_start();
    image.forces_the_fog_call_through();
    the_fog_is_set(image.the_supervisor());
    the_draw_walk_as_the_frame_calls_it();
    the_queue_is_drawn(0);
    orb_api::d3d9::set_texture(device, 0, None);
    orb_api::d3d9::end_scene(device);
    unsafe { present(0) };
    0
}

/// **`0x44fd10`, entered with the sprite manager in `esi`** — moved into the argument of [`the_queue_is_drawn`].
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

/// **`0x42a810`, entered with the camera in `edi`** — see [`draws_the_queue_in_esi`].
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

/// **`0x42a300`, entered with the supervisor in `edi`** — see [`draws_the_queue_in_esi`].
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

/// **`0x432cc0`, entered with the player in `ebx`** — see [`draws_the_queue_in_esi`].
#[unsafe(naked)]
unsafe extern "C" fn makes_the_options_in_ebx() {
    std::arch::naked_asm!(
        "push ebx",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym the_options_are_made,
    )
}

/// **`0x434380`, entered with the player in `eax`** — see [`draws_the_queue_in_esi`].
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

/// **`0x437550`, entered with the record in `eax`** — see [`draws_the_queue_in_esi`].
#[unsafe(naked)]
unsafe extern "C" fn empties_a_record_in_eax() {
    std::arch::naked_asm!(
        "push eax",
        "call {body}",
        "add esp, 4",
        "ret",
        body = sym a_record_is_emptied,
    )
}

/// The shot's step, as the image writes it — see `Image::the_shot_steps`.
extern "C" fn the_shot_is_stepped(_player: usize) {
    let fake = running();
    fake.image.the_shot_steps();
    fake.shot_stepped.set(fake.shot_stepped.get() + 1);
}

/// **The shots in flight moved, 0x434440**, `stdcall` on the player — and none are laid out to move.
extern "stdcall" fn the_shots_are_moved(_player: usize) {}

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

/// **The fog set**, and the device told where the dword asked it to be — counted here rather than at the device.
extern "C" fn the_fog_is_set(_supervisor: usize) {
    let fake = running();
    fake.launch.asked_for(FOG_SET);
    if fake.image.sets_the_fog() {
        fake.fog_told.set(fake.fog_told.get() + 1);
    }
}

/// **The pause menu's resume, 0x42c880**, which takes nothing: the game object's pause flag off (0x42c885), the
/// music's command and the speed put back — as orb calls it.
extern "C" fn the_run_is_unpaused() {
    let fake = running();
    the_resume_is_made();
    fake.unpaused.set(fake.unpaused.get() + 1);
}

/// **And the same resume as the pause menu's own item makes it** (0x42d508), which is counted too.
fn the_resume_is_made() {
    let fake = running();
    fake.image.the_game_is_paused(false);
    fake.image.a_sound_command_is_queued(THE_RESUMES_COMMAND);
    fake.image.the_pause_puts_the_speed_back();
}

/// **The options made for the power, 0x432cc0** — see `Image::the_options_are_made` — as orb calls it.
extern "C" fn the_options_are_made(_player: usize) {
    let fake = running();
    fake.image.the_options_are_made();
    fake.options_made.set(fake.options_made.get() + 1);
}

/// **A shot type's record made empty, 0x437550** — see `Image::a_record_is_emptied`.
extern "C" fn a_record_is_emptied(record: usize) {
    running().image.a_record_is_emptied(record);
}

/// **The pad's half of the read, 0x457270**: the keyboard's word in `ecx` with joystick 0's in it, one of two ways.
/// Where the input's set-up found a DirectInput pad, through the device's own vtable — `Poll` (0x457407), an `Acquire`
/// where that fails and the keys alone for the frame (0x45741d), and `GetDeviceState` of `0x110` (0x457476 through
/// 0x45749f), each held button's byte its top bit, the two axes against the configuration's thresholds (0x45752b
/// through 0x457582). Otherwise `joyGetPosEx(0)` through the import orb writes over (0x4572bf), each axis past a quarter
/// of the travel the set-up's caps give it from its middle (0x457384 through 0x4573e8). The mapping's five buttons
/// each into its bit either way (0x4572e8 through 0x457382); the hat never looked at.
extern "fastcall" fn pad_half(keys: u32) -> u32 {
    let pads = the_pads_share();
    running().pads_share.set(pads);
    keys | pads
}

/// The pads' share of [`pad_half`]: what it makes of them with no keys in front.
fn the_pads_share() -> u32 {
    let image = &running().image;
    let held_on = |held: &dyn Fn(u32) -> bool| {
        let mapping = image.the_pads_mapping();
        [(0, 0x1), (1, 0x2), (2, 0x8), (3, 0x100), (8, 0x200)]
            .into_iter()
            .filter(|(at, _)| u32::try_from(mapping[*at]).is_ok_and(held))
            .fold(0u32, |word, (_, bit)| word | bit)
    };
    let directions = |up: bool, down: bool, left: bool, right: bool| {
        [(up, 0x10), (down, 0x20), (left, 0x40), (right, 0x80)]
            .into_iter()
            .filter(|(held, _)| *held)
            .fold(0u32, |word, (_, bit)| word | bit)
    };
    if let Some((device, across, down)) = image.the_directinput_pad() {
        let slot = |index: usize| {
            let vtable = image.space().read::<usize>(device);
            image
                .space()
                .read::<usize>(vtable + index * size_of::<usize>())
        };
        let poll: unsafe extern "system" fn(usize) -> i32 =
            unsafe { std::mem::transmute(slot(DEVICE_POLL)) };
        if unsafe { poll(device) } < 0 {
            let acquire: unsafe extern "system" fn(usize) -> i32 =
                unsafe { std::mem::transmute(slot(DEVICE_ACQUIRE)) };
            unsafe { acquire(device) };
            return 0;
        }
        let read: unsafe extern "system" fn(usize, u32, *mut u8) -> i32 =
            unsafe { std::mem::transmute(slot(DEVICE_GET_STATE)) };
        let mut state = [0u8; 0x110];
        if unsafe { read(device, 0x110, state.as_mut_ptr()) } < 0 {
            return 0;
        }
        let axis = |at: usize| {
            i32::from_le_bytes([state[at], state[at + 1], state[at + 2], state[at + 3]])
        };
        let (x, y) = (axis(0), axis(4));
        let (across, down) = (i32::from(across), i32::from(down));
        let buttons = held_on(&|button| {
            usize::try_from(button).is_ok_and(|at| at < 0x80 && state[0x30 + at] & 0x80 != 0)
        });
        return buttons | directions(y < -down, y > down, x < -across, x > across);
    }
    let mut pad = orb_api::JoyInfo {
        size: size_of::<orb_api::JoyInfo>() as u32,
        flags: orb_core::joystick::RETURN_ALL,
        ..orb_api::JoyInfo::default()
    };
    if unsafe { orb_core::joystick::answer(0, &mut pad) } != orb_api::joyerr::NOERROR {
        return 0;
    }
    let (x_min, x_max, y_min, y_max) = image.the_joysticks_axes();
    let buttons = held_on(&|button| {
        1u32.checked_shl(button)
            .is_some_and(|bit| pad.buttons & bit != 0)
    });
    let pushed = |at: u32, low: u32, high: u32| {
        let quarter = high.wrapping_sub(low) >> 2;
        let middle = low.wrapping_add(high) >> 1;
        (
            at < middle.wrapping_sub(quarter),
            middle.wrapping_add(quarter) < at,
        )
    };
    let (left, right) = pushed(pad.x, x_min, x_max);
    let (up, down) = pushed(pad.y, y_min, y_max);
    buttons | directions(up, down, left, right)
}

/// **Whether the keyboard device is acquired**, which the system takes away from a foreground device whose window
/// goes behind — see [`Fake::the_keyboards_word`].
static KEYBOARD_ACQUIRED: AtomicBool = AtomicBool::new(false);

/// **The keyboard device's own `Acquire` and `Unacquire`**, the two of it orb calls.
unsafe extern "system" fn its_keyboard_is_acquired(_device: usize) -> i32 {
    KEYBOARD_ACQUIRED.store(true, Ordering::Relaxed);
    0
}

unsafe extern "system" fn its_keyboard_is_let_go_of(_device: usize) -> i32 {
    KEYBOARD_ACQUIRED.store(false, Ordering::Relaxed);
    0
}

/// **The DirectInput pad this file holds**: acquired whenever asked, polled as a pad that is there, and its state
/// what an e2e test last pushed.
unsafe extern "system" fn its_pad_acquires(_device: usize) -> i32 {
    0
}

unsafe extern "system" fn its_pad_is_polled(_device: usize) -> i32 {
    0
}

unsafe extern "system" fn its_pad_is_read(_device: usize, size: u32, into: *mut u8) -> i32 {
    let state = super::the_state_of(running().directinput_pad.get(), size);
    unsafe { std::ptr::copy_nonoverlapping(state.as_ptr(), into, state.len()) };
    0
}

/// **The replay built for a stage built fresh, 0x435e00**, `stdcall` on the game object's `+0x74`: the replay's
/// object made in the mode it names — recording's seed taken and its draws counted from nought (0x43582a) — and what
/// the build stood at as it reached the seam written down.
extern "stdcall" fn replay_built(replaying: u32) -> u32 {
    let fake = running();
    fake.image.the_replay_is_built(replaying != 0);
    fake.seeded_on
        .borrow_mut()
        .push((fake.image.the_games_flags(), fake.image.the_stages_frames()));
    0
}

/// **Where orb's own frame calls the game's sounds**, which on th11 is `Th11`'s own `plays_no_sounds` and nothing of
/// the game's — the sound's commands are carried out in the supervisor's job. Written down as asked, so that a frame
/// of orb's own still says where it made the call, and nothing else.
unsafe extern "fastcall" fn play_sounds(_manager: usize) {
    running().launch.asked_for(SOUND);
}

/// **The present, 0x4465d0**, which from here is the compositor's: where an e2e test counts a frame handed over — and
/// where the build's thread gets a frame of its own.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
    fake.the_titles_thread_is_given_a_while();
    fake.its_loader_goes_on();
}

/// The hooks `Th11::hooks` declines, handed over to be a valid `Originals` and for nothing more.
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

/// **winmm's own `joyGetPosEx`, which the import held before orb wrote over it**.
unsafe extern "system" fn joystick_position(device: u32, into: *mut orb_api::JoyInfo) -> u32 {
    if into.is_null() {
        return orb_api::joyerr::PARMS;
    }
    let (result, info) = orb_api::joystick::position(device, unsafe { (*into).flags });
    unsafe { *into = info };
    result
}

extern "C" fn pad_buttons() -> *mut c_void {
    std::ptr::null_mut()
}

extern "C" fn create_game_window(_instance: *mut c_void) -> i32 {
    0
}

extern "C" fn init_d3d_device() -> i32 {
    0
}

/// **Its `CreateFileA`**, which the score file's fork calls through: the name the call landed on written down, and a
/// handle that is its place in that list.
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

/// **An open of `scoreth11.dat` for its write through whatever stands in front of this game's `CreateFileA`**.
fn its_score_file_opened_for_writing() {
    const GENERIC_WRITE: u32 = 0x4000_0000;
    let open = match orb_core::score::installed() {
        true => orb_core::score::create_file_a,
        false => create_file,
    };
    unsafe {
        open(
            c"scoreth11.dat".as_ptr().cast(),
            GENERIC_WRITE,
            0,
            std::ptr::null(),
            0,
            0,
            std::ptr::null_mut(),
        )
    };
}

/// **Its `CreateWindowExA`**, which the window's rewrite calls through with the arguments orb decided.
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

/// `D3D_SDK_VERSION` as `d3d9.h` has it, which th11 hands `Direct3DCreate9` (`0x20`, 0x44570e).
const D3D_SDK_VERSION: u32 = 32;
/// `D3DDEVTYPE_HAL`.
const D3DDEVTYPE_HAL: u32 = 1;
/// `D3DCREATE_HARDWARE_VERTEXPROCESSING`, the flag th11 tries its device with first (0x446f5d).
const D3DCREATE_HARDWARE_VERTEXPROCESSING: u32 = 0x40;
/// **Where th11 keeps the present parameters** every `Reset` is handed (0x4470ca).
const PRESENT_PARAMETERS: usize = 0x004c_3374;
/// **The presentation interval the parameters hold on the configuration's defaults**: `D3DPRESENT_INTERVAL_IMMEDIATE`,
/// which bit `0x10` of `0x4c3dc0` asks for and the defaults set (0x446e40 through 0x446e63, 0x446b85 through 0x446ba2)
/// — a present that does not wait for the display's blank, the default frame 0x446080 pacing in its present instead.
const THE_GAMES_PRESENTATION_INTERVAL: u32 = 0x8000_0000;

/// **Where the host's Direct3D object goes**, its vtable right behind it: an address past every range th11 lays out.
const DIRECT3D: usize = 0x2100_0000;
const DIRECT3D_VTABLE: usize = DIRECT3D + size_of::<usize>();
const DIRECT3D_BYTES: usize = size_of::<usize>() * (orb_api::d3d9::CREATE_DEVICE_SLOT + 2);

/// **And its `Direct3DCreate9`**, which orb's rewrite calls through.
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

/// **And its `IDirect3D9::CreateDevice`**, which writes down the behaviour flags and the interval it was handed.
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

/// **And its `SetForegroundWindow`**, which orb's rewrite calls through for a window it leaves to the game.
unsafe extern "system" fn host_brings_a_window_forward(window: *mut c_void) -> i32 {
    i32::from(
        running()
            .sim()
            .windows()
            .bring_forward(orb_api::Hwnd(window as usize)),
    )
}
