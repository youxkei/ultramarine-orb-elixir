//! A th06nc that is not the real one: it owns its own memory, advances its own state, and calls orb's
//! hooks where the real game's code calls them.
//!
//! **It plays the game's part, not the address space's**, for the reason [`th06c`](super::th06c)'s does:
//! its state is the memory `orb_core::game::th06nc::image` lays out, `read_state` is how orb learns what
//! the run is, and a chapter restored underneath it takes the run back with it because there is nothing
//! else for the run to be.
//!
//! **Its frame is orb's, the way the real one's is.** th06nc's frame is the function `main`'s loop calls,
//! which orb stands in for whole — see `orb_core::game::th06nc`'s `THE_FRAME` — so a frame here is
//! `orb_core::runtime::render`, whose update is th06nc's own walk of the calc chain calling *this* game's
//! chain, whose draw is th06nc's own walk of *this* game's draw chain, and whose calls into the game — the
//! walk, the camera, the draw's own, the frame's end and the three of the music's catch-up — are functions
//! of this file's handed over through `Image::hands_over_the_frames_own_calls` and
//! `Image::hands_over_the_draws_own_calls`.
//!
//! **And it reads its keyboard the way the real one does**: a `GetKeyboardState` it looked up, which is
//! orb's [`orb_core::runtime::keyboard_state`], translated through the game's own table into the 256 keys
//! it plays a run off, and the word built out of those keys. So the word a frame is played with here is
//! the word orb wrote into that snapshot — and nothing else, which is the whole of how a recipe drives
//! this game.
//!
//! **The block of key bindings the real one builds its word through is not laid out here.** The real read
//! goes keys → a copy of them one bit wide at `0x140c6e190` → the word, each binding byte in
//! `0x140c6e07c`–`0x140c6e158` naming the scan code one action is on; which byte is which action is
//! `th06.cfg`'s and orb reads none of it. What is read off the exe and kept here is [`DIK_BITS`] — the
//! bits each key's press puts in the word — so the word this game builds is the word the real one builds
//! for a default configuration, through one step instead of three.
//!
//! **And it presents through a swapchain it keeps**, which is what orb draws its overlay over: the object
//! is among the renderer's own, and [`ends_the_frame`] calls its `Present` as the real `0x14003db50`
//! does — `0x140097150`, which is `jmp 0x140094620`. A fake that presented only through the frame loop's
//! own call would never reach orb's stand-in in that slot, and no e2e test could see a quad of a canvas.
//!
//! Where a number here is th06nc's it says so. Where it is this game's own — how far the player moves,
//! when its midboss arrives — it says that too.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_void};
use std::path::PathBuf;

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th06nc::Th06nc;
use orb_core::game::th06nc::image::{
    Image, MENU_ITEM_EXTRA, MENU_ITEM_PRACTICE, MENU_ITEM_RUN, RESULT_LEAVING, RESULT_NAME_BYTES,
    RESULT_SAVE_REPLAY_QUESTION, RESULT_THE_END_ITEM, RESULT_TYPING_A_NAME, SCREEN_CHARACTER,
    SCREEN_DIFFICULTY, SCREEN_LOGO, SCREEN_MENU, SCREEN_ON_ITS_OWN, SCREEN_SHOT_SELECT,
    SCREEN_TITLE, chain_job, chain_result,
};
use orb_core::game::th06nc::score_file::{self, CLEARED, Record};

use super::{DRAW, Display, FLUSH, Launch, Launched, PRESENT, SOUND, UPDATE, Work, scratch};

/// The exe this game is running as, and the `Game` orb is attached to found out of the table by it.
pub const EXE: &str = "th06nc.exe";

/// **The two names this game asks the host for**, which orb answers — `Hooks::input` and
/// `Hooks::makes_its_renderer` under `orb_core::game::th06nc`, whose own constants these are the same
/// strings as. Written out here rather than read from there for the reason every number in this game's
/// `image` is: a name shared between the asker and the answerer agrees with itself however wrong it is.
const KEYBOARD_READ: &str = "GetKeyboardState";
const RENDERER_MADE: &str = "CreateDXGIFactory2";
/// And the two the window is made and moved through, which this game asks for by name as well —
/// `Hooks::makes_its_window` and `Hooks::moves_its_window`. See [`Fake::creates_its_window`].
const WINDOW_MADE: &str = "CreateWindowExW";
const WINDOW_MOVED: &str = "MoveWindow";
/// And the two it puts that window on the screen with, which it looks up at its startup as well —
/// `Hooks::shows_its_window` and `Hooks::places_its_window`. See [`Fake::shows_its_window`].
const WINDOW_SHOWN: &str = "ShowWindow";
const WINDOW_PLACED: &str = "SetWindowPos";
/// And the one it writes that window's styles with when it enters its window mode —
/// `Hooks::restyles_its_window`. See [`Fake::restyles_its_window`].
const WINDOW_RESTYLED: &str = "SetWindowLongW";

/// `GWL_STYLE` and `GWL_EXSTYLE`, the two indices this game's window mode writes, in the order it writes
/// them: the extended style at `0x1400befee` and the style at `0x1400bf011`.
const GWL_STYLE: i32 = -16;
const GWL_EXSTYLE: i32 = -20;

/// **The styles the window mode writes**: `WS_EX_WINDOWEDGE` alone — what th06c's table at `0x1402bf888`
/// holds for its windowed modes, and what its window was measured holding, `exstyle 0x00000100` — and a
/// captioned style with `WS_VISIBLE` added, the way th06c adds it at `0x14009d0ee`.
const THE_WINDOW_MODES_EX_STYLE: u32 = 0x0000_0100;
const THE_WINDOW_MODES_STYLE: u32 = 0x00ca_0000 | 0x1000_0000;

/// `SW_SHOW`, and the flags a place carries: `SWP_SHOWWINDOW`, and `SWP_NOSIZE | SWP_NOZORDER` for a call
/// that moves the window and leaves the rest of it alone. Written out the way `orb_core::window`'s own
/// numbers are.
const SW_SHOW: i32 = 5;
const SWP_SHOWWINDOW: u32 = 0x0040;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;

/// **The window this game asks the host for**: the size out of its own `th06.cfg` and a position of its
/// own, in a style with a caption and no frame to resize by.
///
/// Numbers of this fake's own, the way every other ask an e2e test reads back is: what a launch that plays
/// itself does with them is take the position and leave the size, so the position being in the middle of
/// nowhere is the point of it — see `orb_core::window::create_window_ex_w`.
const ASKS_FOR: (i32, i32) = (1280, 720);
const ASKS_AT: (i32, i32) = (320, 180);
/// `WS_CAPTION | WS_SYSMENU | WS_VISIBLE`, written out the way `orb_core::window`'s own styles are.
const ASKED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x1000_0000;

/// **The bits of the word this game's own read builds out of its 256 keys**, by the DirectInput code of
/// the key — read off the twenty-seven binding reads at `0x140013499` onwards, each
/// `neg al; sbb ecx,ecx; and ecx,<bits>; or r8d,ecx`: `Z` carries the shot with the two shortcut bits
/// beside it, `X` the bomb with one, `Escape` two shortcut bits and no button of a run's, `Enter` the
/// decide's own bit, `Left Shift` the focus and the four arrows their four. The scan codes are this
/// build's own table at `0x1402f6060`.
const DIK_BITS: [(usize, u16); 9] = [
    (0x2c, SHOT | ENTER | 0x8000), // Z
    (0x2d, BOMB | 0x200),          // X
    (0x2a, FOCUS),                 // Left Shift
    (0x01, ESCAPE),                // Escape
    (0x1c, ENTER),                 // Enter
    (0xc8, UP),                    // Up
    (0xd0, DOWN),                  // Down
    (0xcb, LEFT),                  // Left
    (0xcd, RIGHT),                 // Right
];
const SHOT: u16 = 0x0001;
const BOMB: u16 = 0x0002;
const FOCUS: u16 = 0x0004;
const UP: u16 = 0x0010;
const DOWN: u16 = 0x0020;
const LEFT: u16 = 0x0040;
const RIGHT: u16 = 0x0080;
/// **The decide, which is `0x100` and not orb's own `0x1000`**: the front end writes this bit into the
/// word itself at `0x1400491cc` where a screen is to be skipped, and its edge check at `0x140049aa5`
/// reads it.
const ENTER: u16 = 0x0100;
/// The two bits `Escape` sets, of which the pause reads the second — this laid-out game's own reading,
/// the real one's pause bit being unread.
const ESCAPE: u16 = 0x0600;
const PAUSE: u16 = 0x0400;
/// What the front end reads as its decide and its cancel: the shot or the enter, and the bomb.
const MENU_DECIDE: u16 = SHOT | ENTER;
const MENU_CANCEL: u16 = BOMB;

/// **The pad's half of the word, `0x140012f00`**, which the compose ends in: `XInputGetState` through the name
/// it looked up (`0x1400abd40`) — orb's own answer, `runtime::xinput_get_state` — and its buttons ORed into the
/// keyboard's bits from `0x140012fd3` on. `A` carries the bits `Z` does and `B` those `X` does, `X` the focus,
/// `Start` the pause and the d-pad the four directions: what `Th06nc::press_into_pad` presses each with, which a
/// real case's run has played on. The focus the half makes of a pad's held one (`[0x140abad70]`) is the game's
/// own count and not laid out here.
fn the_pads_word() -> u16 {
    let mut pad = [0u8; 16];
    if unsafe { orb_core::runtime::xinput_get_state(0, pad.as_mut_ptr()) } != 0 {
        return 0;
    }
    let buttons = u16::from_le_bytes([pad[4], pad[5]]);
    [
        (PAD_A, SHOT | ENTER | 0x8000),
        (PAD_B, BOMB | 0x200),
        (PAD_X, FOCUS),
        (PAD_START, PAUSE),
        (PAD_UP, UP),
        (PAD_DOWN, DOWN),
        (PAD_LEFT, LEFT),
        (PAD_RIGHT, RIGHT),
    ]
    .into_iter()
    .filter(|(button, _)| buttons & button != 0)
    .fold(0, |word, (_, bits)| word | bits)
}

/// XInput's own bits for the buttons [`the_pads_word`] reads.
const PAD_A: u16 = 0x1000;
const PAD_B: u16 = 0x2000;
const PAD_X: u16 = 0x4000;
const PAD_START: u16 = 0x0010;
const PAD_UP: u16 = 0x0001;
const PAD_DOWN: u16 = 0x0002;
const PAD_LEFT: u16 = 0x0004;
const PAD_RIGHT: u16 = 0x0008;

/// **The main menu's ten items**, of which orb has a question over the first two — see `asm.md`'s own
/// table, read off launches: a fresh launch's taps of down went `0, 2, 3 … 9`, the Extra stepped over
/// while it is locked.
const MENU_ITEMS: i32 = 10;

/// The items each select offers: four difficulties, two characters, two shots.
const DIFFICULTIES_OFFERED: i32 = 4;
const CHARACTERS: i32 = 2;
const SHOTS: i32 = 2;

/// **How many frames the main menu ignores its own decide for**, `0x140049a8d` against `0x14`.
const THE_MENU_TAKES_A_PRESS_AFTER: i32 = 0x14;
/// **And the frame the four self-advancing screens go by on**: `0x1400491af` writes the decide into the
/// word itself where `[front end + 0x9638]` is exactly `0xf`, so the screen after that is the next one.
const A_SELECT_GOES_BY_ON_ITS_OWN_AFTER: i32 = 0xf;
/// The frames the logo and the screen after the menu's first item last, this laid-out game's own numbers:
/// long enough for a recipe's `wait_until` to see each.
const LOGO_LEAVES_ON_ITS_OWN_AFTER: i32 = 120;
const ON_ITS_OWN_LASTS: i32 = 60;
/// And how long a launch is up before its logo comes, which is what a fresh process reads as no scene.
const LOGO_COMES_UP_AFTER: u32 = 60;
/// The screens between the shot select and the stage. **14 and 24 take a press and 25 does not**: 24's
/// case is `0x14004d981`, which leaves for anything under [`THE_LAST_SCREENS_GRACE`] frames and then
/// edge-checks the decide, where 25's `0x14004dbe9` reads no word at all and calls the run's own start as
/// soon as the object at `[0x140c6e378]` is ready.
const SCREEN_AFTER_THE_SHOT: i32 = 14;
const THE_SCREEN_THAT_TAKES_THE_LAST_PRESS: i32 = 24;
const THE_SCREEN_THAT_STARTS_THE_RUN: i32 = 25;
/// And how long that press is ignored for, `0x14004d9a3` against `0xf`.
const THE_LAST_SCREENS_GRACE: i32 = 0xf;
/// **And the three the front end walks on its way back from a run** — `16, 17, 18`, read off the eighth
/// launch's own `front end: screen N` lines and timed by the launch of `the-front-end-after-a-run.yaml`:
/// 16 came 16ms after `leave: true`, 17 one frame after 16 and 18 seven frames after 17, and **18 is
/// where it comes to rest** — 21,600 frames of that launch sat on it with nothing pressed. They are the
/// attract cycle rather than a way back to the menu, and 18 is the replay list; what leaves it is the
/// cancel.
const SCREEN_BACK_FROM_A_RUN: i32 = 16;
const SCREEN_BEFORE_THE_REPLAY_LIST: i32 = 17;
const THE_REPLAY_LIST: i32 = 18;
/// And the frames each of the first two stands for, off those same timings at sixty frames a second.
const SCREEN_BACK_FROM_A_RUN_LASTS: i32 = 1;
const SCREEN_BEFORE_THE_REPLAY_LIST_LASTS: i32 = 7;

/// The scene a launch reads before its front end has been built at all, which is what tells a first build
/// from a rebuild after a run.
const NO_SCENE: i32 = 0;

/// **The character every decide at the name entry types**, which is the one that entry's own cursor starts
/// on: a launch of the real game whose rules held the shot through that screen registered `AAAAAAAA`.
const THE_CHARACTER_THE_ENTRYS_CURSOR_STARTS_ON: char = 'A';

/// What the stage's own script does and when, this laid-out game's own schedule — a boss that arrives, a
/// card it casts, an attack that changes and a boss beaten, which is what a chapter table is collected
/// off.
pub const MIDBOSS_ARRIVES: u32 = 400;
pub const CARD_STARTS: u32 = 500;
pub const ATTACK_CHANGES: u32 = 700;
pub const MIDBOSS_BEATEN: u32 = 900;
const BOSS_LIFE: i32 = 1500;
/// **And the frame her life is read as nearly out on**, a second before she goes, with the life she goes
/// with beside it — see [`Fake::plays_the_stage`], which has why that is a frame of its own.
const A_BOSS_IS_NEARLY_OUT_AT: u32 = MIDBOSS_BEATEN - 60;
const A_BEATEN_BOSSES_LIFE: i32 = 5;

/// **The card that schedule casts, and the name the game's own card start writes into its record** —
/// `0x140027950`'s loop, which copies the name in at `+0x18` with the low byte of its bytes' sum at
/// `+0x12`, and `0x140027a2a`, which reads the attempts at `+0x3c` and writes them back a step up.
///
/// Both, because the count orb adds for a retry is only added beside a name: `0x1400279a0` holds the sum
/// against the name and `0x1400279aa` clears the record where they disagree — so a laid-out game that
/// cast a card its record was not named for would be one no attempt could be counted against. Which is
/// what [`the_cards_a_run_of_th06nc_counted`](crate::the_cards_a_run_of_th06nc_counted) is about.
///
/// A card of its own rather than the one that file's tests name, so that a test setting a card by hand is
/// not a test this schedule answers.
pub const THE_CARD_THE_STAGE_CASTS: usize = 0;
const ITS_NAME: &str = "「ムーンライトレイ」";

/// The field a run is played in, and where a stage starts the player — the game's own 384 by 448, and the
/// middle of the bottom.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
const PLAYER_STARTS_AT: (f32, f32) = (192.0, 384.0);
/// How far a frame moves the player unfocused, and half the box a hit is tested against — this laid-out
/// game's own numbers.
const A_STEP: f32 = 4.0;
const PLAYER_HALF: f32 = 4.0;

/// The stage's bullets: when the first comes, how often, from where, how fast and how big, and how many
/// slots of the manager's are used.
const FIRST_BULLET_AT: u32 = 60;
const A_BULLET_EVERY: u32 = 120;
const BULLETS_FROM: (f32, f32) = (192.0, 0.0);
const BULLET_SPEED: f32 = 3.0;
const BULLET_SIZE: f32 = 8.0;
const BULLET_SLOTS_USED: usize = 16;

/// How long a hit may still be bombed out of, and how long a bomb goes off for.
const A_BOMB_IS_TAKEN_FOR: i32 = 8;
pub const A_BOMB_LASTS: u32 = 60;

/// **How long the walk-in takes and what it hands the count over to**, which is the 270 frames of a
/// stage nothing reaches the player in — `0x14006a4b7`'s `cmp ecx,0x1e` and the `0xf0` at `0x14006a6cf`.
/// `th06nc/asm.md`'s *The 270 frames of a stage nothing can reach its player in* is the reading.
const RESPAWN_TAKES: u32 = 0x1e;
const INVULNERABLE_FOR: i32 = 0xf0;
/// And what a stage's own register leaves on the count, `0x78` at `0x140069b3b` — which no state but the
/// invulnerability's reads, so what it is here is a number an e2e test holds the build to.
const THE_REGISTER_LEAVES: i32 = 0x78;

/// **How many updates a firing cycle lasts**: `0x14006b39e` holds the phase against thirty and the reset
/// beside it puts the pair back to [`NOT_FIRING`], so the update after the thirtieth starts another.
const A_FIRING_CYCLE_LASTS: i32 = 30;
/// **The pair a player firing nothing holds**, phase and phase before — the qword `0x14006b3b4` writes
/// and the one a stage's own register writes at `0x140069e46`.
const NOT_FIRING: (i32, i32) = (-1, -999);
/// **And the phase a press starts at**, `0x14006ae30`. The `-999` `0x14006ae36` writes beside it is not
/// here because nothing outside that update ever reads it: the advance runs in the same one and puts what
/// this held into the phase before.
const THE_FIRST_PHASE: i32 = 0;

#[derive(Clone, Copy)]
struct Bullet {
    slot: usize,
    at: (f32, f32),
    step: (f32, f32),
}

/// The stages a run plays and the difficulty the Extra is, which is what says a stage is the last.
const STAGES_A_RUN_PLAYS: i32 = 6;
const DIFFICULTY_EXTRA: i32 = 4;
/// **And the stage the Extra's own run starts in**, as the supervisor's case writes it: six, which the
/// stage's own build then counts on to seven — `0x14007bc9c`.
const THE_EXTRAS_STAGE: i32 = 6;

/// What a run starts with: the lives and bombs `th06.cfg` gives, and the Extra's own.
const CONFIG_LIVES: i8 = 2;
const CONFIG_BOMBS: i8 = 3;
const EXTRA_LIVES: i8 = 2;
const EXTRA_BOMBS: i8 = 3;

/// The updates the ending's script and the roll after it take.
const ENDING_UPDATES: i32 = 10;
const ROLL_UPDATES: i32 = 10;

/// The score file this game opens, the file orb's fork sends the mode's runs to, and what `fopen`'s two
/// modes come to at the `CreateFileW` the exe's own CRT ends in: `rb` asks to read a file that is there,
/// `wb` to write one made afresh — Win32's own numbers, which are the game's arguments here.
const SCORE_FILE: &CStr = c"score.dat";
const THE_MODES_FILE: &str = "pointdevice_score.dat";
const READ: (u32, u32) = (0x8000_0000, 3);
const WRITE: (u32, u32) = (0x4000_0000, 2);

/// What this game's calc walk answers where the frame is to carry on.
pub const FRAME_KEPT_RUNNING: i32 = 0;

/// The run a test walks into by default: Normal, Reimu, shot A — the slot
/// `orb-real/recipes/th06nc` names.
pub const DIFFICULTY: i32 = 1;
pub const CHARACTER: u8 = 0;
pub const SHOT_TYPE: u8 = 0;

/// **The walk from the title to a run, as a recipe**, which is the real recipes' own — see the eighth
/// launch of `into-a-chapter.yaml`, whose thirty-three steps walked it with none given up.
///
/// **Here rather than in one of the e2e modules that plays it**, because it is this game's screens and
/// every module over a run of it starts by walking them.
pub(crate) const TO_A_RUN: &str = "\
- wait_until: { screen: 2 }
  give_up_after: 36000
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
- wait_until: { screen: 3 }
  give_up_after: 1800
- press: [up]
  frames: 6
  until: { screen: 3, cursor: 0 }
  give_up_after: 900
- wait_until: { frames: 30 }
";

/// **And the walk on from that menu to the last screen before a stage**, which is the rest of what every
/// module over a run of this game makes: orb's own mode question answered over the menu's first item,
/// Normal taken at the difficulty, and the three selects pressed through to
/// `front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS`.
///
/// **It stops before that screen's own decide**, because that is where the modules differ: one presses it
/// and starts a run, and one finds orb's question about picking one up standing in front of it — see
/// `Th06nc::run_pointed_at`, which reads the run there.
pub(crate) const TO_THE_LAST_SCREEN: &str = "\
- press: [decide]
  frames: 6
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 9 }
  give_up_after: 1800
- press: [down]
  frames: 6
  until: { screen: 9, cursor: 1 }
  give_up_after: 900
- wait_until: { frames: 30 }
- press: [decide]
  frames: 6
- wait_until: { screen: 24 }
  give_up_after: 3600
- wait_until: { frames: 30 }
";

/// A th06nc laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// Whether this game's chain walk says the game is leaving — see [`Fake::leaves`].
    leaving: Cell<bool>,
    /// How many frames this launch has run, for the logo that comes up on its own.
    frames_since_launch: Cell<u32>,
    /// How many frames the screen the menu's first item opens has been going by on its own.
    on_its_own_for: Cell<i32>,
    /// Which item the menu was left on, which is the item the front end comes back on.
    left_the_menu_on: Cell<i32>,
    /// Whether the run being chosen is the Extra.
    extra: Cell<bool>,
    /// How long a stage lasts before the next is built. `None` is a stage that never ends.
    stage_frames: Cell<Option<u32>>,
    /// The stage frame the script last acted on, the bullets the stage has out, the frames a respawn has left
    /// and the frames a bomb has left to go off for. **Not the frames a hit has left**, which are the game's
    /// own field — `Image::a_bomb_still_takes_the_hit_back_for` — and what orb reads a deathbomb off.
    last_stage_frame: Cell<u32>,
    bullets: RefCell<Vec<Bullet>>,
    respawn_left: Cell<u32>,
    bomb_left: Cell<u32>,
    /// Whether the stage fires at the player — see [`Fake::fires_at_the_player`].
    fires: Cell<bool>,
    /// How many of the stage's own enemies are alive beside whatever the fight has.
    enemies: Cell<i32>,
    /// Whether an e2e test said the player is hit on the next update.
    hit: Cell<bool>,
    /// The updates the ending's script and the roll after it have left, where one is running.
    ending_updates: Cell<i32>,
    roll_updates: Cell<i32>,
    /// Every open of the score file, by the name it landed in and whether it was for writing.
    opens: RefCell<Vec<(String, bool)>>,
    /// Every seek the sound's catch-up made, for a test about the frame.
    seeks: RefCell<(Vec<(i64, i32)>,)>,
    /// How many times the frame ended and how many references to the swapchain have been given back.
    pictures: Cell<u32>,
    swapchain_releases: Cell<u32>,
    /// Whether the game's own retry menu — the continue question a run out of lives is asked — is up.
    out_of_lives: Cell<bool>,
    /// Whether the window's own mode has been entered, and the size the picture was last set to — what
    /// `Game::prepare_frame` does to this game's window on a launch's first frame.
    windows_remade: Cell<bool>,
    picture_sized_to: Cell<Option<(u32, u32)>>,
    /// The window the host made for this game, and [`super::WINDOW`] until one was asked for — see
    /// [`Fake::creates_its_window`], which every test but the ones about the window itself leaves alone.
    window: Cell<orb_api::Hwnd>,
    /// The size the swapchain's buffers are, and the window's client the frame last made them for — see
    /// [`ends_the_frame`].
    buffers: Cell<(u32, u32)>,
    client_was: Cell<Option<orb_api::Rect>>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory.
    _installed: orb_api::Installed,
}

thread_local! {
    static RUNNING: Cell<*const Fake> = const { Cell::new(std::ptr::null()) };
}

fn running() -> &'static Fake {
    let fake = RUNNING.get();
    assert!(!fake.is_null(), "no game has been attached on this thread");
    unsafe { &*fake }
}

impl Launched for Fake {
    /// The object the frame is called on, which for this game reaches nothing here: the supervisor's
    /// address, which is inside the space.
    fn frame_window(&self) -> *mut c_void {
        std::ptr::without_provenance_mut(orb_core::game::th06nc::image::SUPERVISOR_OBJECT)
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
    /// Lays the game out on a display an e2e test says the whole of and attaches orb to it the way a
    /// launch is attached: **orb's own frame loop**, which for this game is the only frame there is.
    ///
    /// Boxed and never moved, for the reason th06's is: the hooks find it through a pointer.
    pub fn attach(display: Display, name: &str, work: Work) -> Box<Self> {
        Self::attach_configured(display, name, work, |_| {})
    }

    pub fn attach_configured(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, name, &[], work, settings)
    }

    /// And on a machine whose own windows are in the language `ui_language` names — `orb_sim::langid` —
    /// with nothing in `orb.yaml` about it, which is where orb's screens take their language from when
    /// the file says none. See `the_screens_of_th06nc_in_english`.
    pub fn attach_on_a_machine_in(
        ui_language: u16,
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, Some(ui_language), name, &[], work, settings)
    }

    /// And with files a launch before left under orb's directory, as paths inside it and their bytes.
    pub fn attach_finding(
        display: Display,
        name: &str,
        left: &[(PathBuf, Vec<u8>)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, name, left, work, settings)
    }

    fn attach_declaring(
        display: Display,
        ui_language: Option<u16>,
        name: &str,
        left: &[(PathBuf, Vec<u8>)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let dir = scratch(name);
        let image = Image::laid_out_seeded(display.seed);
        let installed = image.enter();
        image.sim().log().waits_first(orb_core::log::written_out);
        // Before the attach, that being where orb settles which language its screens are in: a machine
        // declared after it is one nothing asked.
        if let Some(ui_language) = ui_language {
            image.sim().set_ui_language(ui_language);
        }
        image.sim().files().make(&dir);
        for (file, contents) in left {
            image.sim().files().put(dir.join(file), contents);
        }
        let mut config =
            Config::load_beside(&dir.join(EXE)).expect("a directory with no orb.yaml in it");
        // The memory hooks patch an import table there is none of, for the reason th06's says.
        config.track_memory = false;
        settings(&mut config);
        let recipe = config.autoplay.clone();
        // No font beside this exe: what orb would draw it has no device of here anyway.
        let font = dir.join("font.ttf");
        image.sim().text().install_font(&font);
        let launch = Launch::new(
            image.sim(),
            dir,
            display.seed,
            super::orbs_own_loop(&config, the_game_this_is()),
        );
        image.sim().text().remove_font(&font);
        // The calls orb's frame makes into this game, and the draw's own among them.
        image.hands_over_the_frames_own_calls(
            run_calc_chain as *const () as usize,
            moves_the_camera as *const () as usize,
            ends_the_frame as *const () as usize,
            where_the_music_is as *const () as usize,
            how_long_the_track_is as *const () as usize,
            seeks_the_music as *const () as usize,
            makes_a_window as *const () as usize,
            covers_the_monitor as *const () as usize,
            sizes_the_picture as *const () as usize,
        );
        image.hands_over_the_draws_own_calls(
            begins_the_draw as *const () as usize,
            reads_the_viewport as *const () as usize,
            sets_the_viewport as *const () as usize,
            clears as *const () as usize,
            takes_an_element_out as *const () as usize,
            flushes_the_sprites as *const () as usize,
            takes_the_screenshot as *const () as usize,
        );
        // **The host's own `GetProcAddress`, which orb's stands in front of** — see [`proc_address`].
        // What a launch of this game asks for through it is its keyboard read and its factory, and what
        // orb answers each with is what the game then calls.
        orb_core::runtime::GET_PROC_ADDRESS.store(
            proc_address as *const () as usize,
            std::sync::atomic::Ordering::Relaxed,
        );
        // **And the names orb answers through it**, which in a real launch `orb::lookup::install` fills
        // out of this game's `Hooks` — nothing of `attach_to`'s does, so a laid-out game that did not say
        // this is one whose asks all reach the host and none of them orb. The four the fake drives: the
        // keyboard read, the factory whose swapchains orb takes, and the two the window is made and moved
        // through — see [`Fake::creates_its_window`], which is what asks for those. The pad is left out
        // for the reason th06c's fake leaves it out, answering it through `runtime::xinput_get_state`
        // itself — see [`the_pads_word`].
        unsafe {
            orb_core::runtime::answers_the_lookup_of(orb_core::runtime::Lookups {
                read: Some(KEYBOARD_READ),
                renderer: Some(RENDERER_MADE),
                window_made: Some(WINDOW_MADE),
                window_moved: Some(WINDOW_MOVED),
                window_shown: Some(WINDOW_SHOWN),
                window_placed: Some(WINDOW_PLACED),
                window_restyled: Some(WINDOW_RESTYLED),
                ..Default::default()
            })
        };
        // And the file open this game's records go through, which orb's fork stands in front of —
        // `CreateFileW`, the CRT's `fopen` being the exe's own and ending there; see
        // `Hooks::opens_its_files`. With it the heap the copy of that file is taken off, which orb's own
        // copy goes through too.
        unsafe { orb_core::score::install_over_create_file_w(create_file_w) };
        image.hands_over_its_heap(malloc as *const () as usize, free as *const () as usize);
        image.hands_over_the_run_start(the_games_run_start as *const () as usize);
        image.sim().set_host_exe(launch.dir().join(EXE));
        let sim = image.sim();
        sim.display().set_monitor_hz(display.monitor_hz);
        sim.display().set_desktop_hz(display.monitor_hz);
        sim.display().set_foreground(super::WINDOW);
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
        sim.windows()
            .set_monitor(orb_sim::Monitor::scaled(), orb_sim::Frame::SOME);
        // Where the game starts: the supervisor's job registered and nothing else — the state a fresh
        // process reads as no scene at all — with the draw chain's one element.
        image.registers_the_supervisor();
        image.registers_in_the_draw_chain(draw_job as *const () as usize, 0);
        // **The swapchain this game presents through, as the real one has one from its first frame** —
        // laid out here and *made* after the attach, which is when orb has its answer to give. See
        // [`makes_its_renderer`].
        image.keeps_a_swapchain(
            swapchain_query_interface as *const () as usize,
            swapchain_release as *const () as usize,
            swapchain_describes_itself as *const () as usize,
            swapchain_presents as *const () as usize,
            swapchain_presents1 as *const () as usize,
            swapchain_resizes_its_buffers as *const () as usize,
        );
        image.keeps_a_factory(factory_creates_a_swapchain as *const () as usize);
        // The card records as the stage's own build lays them out, so that a capture read back is a record
        // this game wrote.
        image.fills_the_card_records();

        let fake = Box::new(Self {
            image,
            launch,
            leaving: Cell::new(false),
            frames_since_launch: Cell::new(0),
            on_its_own_for: Cell::new(0),
            left_the_menu_on: Cell::new(MENU_ITEM_RUN),
            extra: Cell::new(false),
            stage_frames: Cell::new(None),
            last_stage_frame: Cell::new(0),
            bullets: RefCell::new(Vec::new()),
            respawn_left: Cell::new(0),
            bomb_left: Cell::new(0),
            fires: Cell::new(true),
            enemies: Cell::new(0),
            hit: Cell::new(false),
            ending_updates: Cell::new(0),
            roll_updates: Cell::new(0),
            opens: RefCell::new(Vec::new()),
            seeks: RefCell::new((Vec::new(),)),
            pictures: Cell::new(0),
            swapchain_releases: Cell::new(0),
            out_of_lives: Cell::new(false),
            windows_remade: Cell::new(false),
            picture_sized_to: Cell::new(None),
            window: Cell::new(super::WINDOW),
            buffers: Cell::new(super::BACK_BUFFER),
            client_was: Cell::new(None),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        let walks = the_game_this_is()
            .hooks()
            .walks
            .expect("th06nc hands its two walks over");
        unsafe {
            orb_core::runtime::attach_to(
                the_game_this_is(),
                config,
                fake.image.data(),
                orb_core::runtime::Originals {
                    // **The game's own two walks, which are orb's**: nothing of this game's calls either,
                    // the frame that did being orb's — see `Hooks::walks`.
                    update: std::mem::transmute::<usize, orb_core::runtime::OnItsThis>(
                        walks.update,
                    ),
                    draw: std::mem::transmute::<usize, orb_core::runtime::OnItsThis>(walks.draw),
                    input,
                    stage_building,
                    stage_begun,
                    unlocks_read,
                    ranking_read,
                    card_records_read,
                    render: own_render,
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
                },
            )
        };
        // **The two names this game asks for and the renderer it then makes**, after the attach because
        // that is what fills the table of names orb answers: the keyboard read, and the factory whose
        // swapchains orb takes — and then the swapchain itself, which is what orb's overlay is drawn on.
        fake.makes_its_renderer();
        if let Some(recipe) = &recipe {
            unsafe { orb_bot::install(recipe, the_game_this_is()) };
        } else {
            // **And a launch with no recipe takes the last one's bot out**, which a real launch never has
            // to: `orb_bot`'s own static and the hook it sets are the process's, and an e2e test plays two
            // launches in one.
            unsafe { orb_core::runtime::plays_the_run(None) };
        }
        // The game's own file is there, holding a clear, as an installation somebody has played is; orb's
        // own is not until a ranking has written it. And the one read of the score file this game makes,
        // at its startup, lands in whichever orb's fork says, and the copy every screen parses is what
        // that read found.
        fake.puts_the_file(
            &SCORE_FILE.to_string_lossy(),
            &score_file::encode(&records_holding(true), THE_GAMES_OWN_SEED),
        );
        fake.reads_the_score_file_at_startup();
        fake.frame_takes(work);
        fake
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    /// **The names this game asks for at its startup, and the renderer it makes out of one of them.**
    ///
    /// The real one resolves `GetKeyboardState` and `CreateDXGIFactory2` through `GetProcAddress` and
    /// calls what it got — see `Hooks::input` and `Hooks::makes_its_renderer`. orb answers both, and its
    /// answer to the factory is the host's own with `CreateSwapChainForHwnd` written over, so the
    /// swapchain made through it is one orb presents through.
    ///
    /// **Which is the whole of how orb's own screens reach this game.** A laid-out game that skipped this
    /// is one nothing of orb's is ever drawn on: no canvas, no overlay, and a question raised over a
    /// screen nobody could see it on.
    fn makes_its_renderer(&self) {
        let keyboard =
            unsafe { orb_core::runtime::get_proc_address(0, c"GetKeyboardState".as_ptr().cast()) };
        assert_eq!(
            keyboard,
            orb_core::runtime::keyboard_state as *const () as usize,
            "orb did not answer this game's ask for its keyboard read with its own",
        );
        let factory_made = unsafe {
            orb_core::runtime::get_proc_address(0, c"CreateDXGIFactory2".as_ptr().cast())
        };
        assert_ne!(
            factory_made, 0,
            "orb answered nothing for this game's factory"
        );
        let factory_made: unsafe extern "system" fn(u32, *const u8, *mut usize) -> i32 =
            unsafe { std::mem::transmute(factory_made) };
        let mut factory = 0usize;
        let answered = unsafe { factory_made(0, std::ptr::null(), &mut factory) };
        assert_eq!(answered, 0, "the factory was not made");
        // The swapchain made through that factory's own slot, which orb has written over by now.
        let vtable: usize = self.image.space().read(factory);
        let creates: usize = self
            .image
            .space()
            .read(vtable + self.image.where_a_factory_makes_a_swapchain() * size_of::<usize>());
        let creates: unsafe extern "system" fn(
            usize,
            usize,
            usize,
            *const u8,
            *const u8,
            usize,
            *mut usize,
        ) -> i32 = unsafe { std::mem::transmute(creates) };
        let mut swapchain = 0usize;
        // **The `DXGI_SWAP_CHAIN_DESC1` the real one hands over**: its size, two buffers, and the scaling
        // and swap effect both nought — th06c's own call was read as `640x480 with 2 buffer(s), scaling 0
        // and swap effect 0`, and this one's size is this fake's own back buffer.
        let mut description = [0u8; 48];
        description[..4].copy_from_slice(&super::BACK_BUFFER.0.to_le_bytes());
        description[4..8].copy_from_slice(&super::BACK_BUFFER.1.to_le_bytes());
        description[28..32].copy_from_slice(&2u32.to_le_bytes());
        let answered = unsafe {
            creates(
                factory,
                0,
                super::WINDOW.0,
                description.as_ptr(),
                std::ptr::null(),
                0,
                &mut swapchain,
            )
        };
        assert_eq!(answered, 0, "the swapchain was not made");
    }

    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th06nc.read_state() }
    }

    /// The size the swapchain's buffers are now.
    pub fn the_swapchains_buffers(&self) -> (u32, u32) {
        self.buffers.get()
    }

    /// Whether the stage fires at the player, which it does unless a test says otherwise.
    pub fn fires_at_the_player(&self, fires: bool) {
        self.fires.set(fires);
    }

    /// **How many of the stage's own enemies are alive**, beside whatever the fight has. A wave is that
    /// count going up and a gap is it going back to nothing, which is the whole of what the chapter
    /// detector reads of a stage's script.
    pub fn the_stage_has_enemies(&self, alive: i32) {
        self.enemies.set(alive);
    }

    /// The player hit on the next update, which is a test saying nothing was dodged.
    pub fn hit(&self) {
        self.hit.set(true);
    }

    /// How long a stage of this game lasts before the next begins.
    pub fn stages_last(&self, frames: u32) {
        self.stage_frames.set(Some(frames));
    }

    /// The game's own records holding a clear, which is what offers the Extra on the main menu.
    pub fn its_records_hold_a_clear(&self) {
        self.its_records_are(&records_holding(true));
    }

    pub fn its_records_hold_no_clear(&self) {
        self.its_records_are(&records_holding(false));
    }

    /// The game's own file holding these records — as the installation was before the launch, so the copy
    /// the startup read made of it is made again where that read landed there.
    pub fn its_records_are(&self, records: &[Record]) {
        let name = SCORE_FILE.to_string_lossy().into_owned();
        self.puts_the_file(&name, &score_file::encode(records, THE_GAMES_OWN_SEED));
        let landed_there = self
            .opens
            .borrow()
            .first()
            .is_some_and(|(landed, write)| *landed == name && !write);
        if landed_there {
            self.loads_the_copy_out_of(&name);
        }
    }

    /// The file orb's fork sends the mode's runs to, as a launch before left it.
    pub fn the_modes_file_holds(&self, records: &[Record]) {
        self.puts_the_file(THE_MODES_FILE, &score_file::encode(records, THE_MODES_SEED));
    }

    /// The records of the copy the game parses its screens out of, or `None` where it read no file.
    pub fn the_records_the_game_parses(&self) -> Option<Vec<Record>> {
        score_file::decode(&self.image.the_score_file_copy()?)
    }

    /// The records a file the game wrote holds, or `None` for a name it never wrote.
    pub fn the_records_written_as(&self, name: &str) -> Option<Vec<Record>> {
        score_file::decode(&self.sim().files().get(name)?)
    }

    /// The game leaving on its next update, which is what its own quit does.
    pub fn leaves(&self) {
        self.leaving.set(true);
    }

    /// Every open of the score file: the name it landed in, and whether it was for writing.
    pub fn score_file_opens(&self) -> Vec<(String, bool)> {
        self.opens.borrow().clone()
    }

    pub fn score_file_written_as(&self, name: &str) -> bool {
        self.sim().files().holds(name)
    }

    /// Every seek the frame asked the music for, and how many frames it ended.
    pub fn seeks_asked_for(&self) -> Vec<(i64, i32)> {
        self.seeks.borrow().0.clone()
    }

    pub fn pictures_finished(&self) -> u32 {
        self.pictures.get()
    }

    /// How many references to this game's swapchain orb has given back.
    pub fn swapchain_references_given_back(&self) -> u32 {
        self.swapchain_releases.get()
    }

    /// **The window this game makes, through the name it asks the host for it by.**
    ///
    /// The real one looks `CreateWindowExW` up and calls what it got — see `Hooks::makes_its_window` —
    /// which is how orb decides the window a launch that plays itself is played in, and it asks for
    /// `MoveWindow` beside it, that being what such a window is put back into its corner with. Both are
    /// asked for here, in that order, because a launch that asked for neither is one
    /// `orb_core::window::keeps_its_corner` does nothing in.
    ///
    /// Apart from [`Self::makes_its_renderer`], which the attach runs for every launch: a window is the
    /// subject of the tests about *where* the window goes and of nothing else, and a game whose window
    /// the host never made is what every other test over this game has always had.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        let moved =
            unsafe { orb_core::runtime::get_proc_address(0, c"MoveWindow".as_ptr().cast()) };
        assert_eq!(
            moved,
            orb_core::window::move_window as *const () as usize,
            "orb did not answer this game's ask for its window move with its own",
        );
        let made =
            unsafe { orb_core::runtime::get_proc_address(0, c"CreateWindowExW".as_ptr().cast()) };
        assert_eq!(
            made,
            orb_core::window::create_window_ex_w as *const () as usize,
            "orb did not answer this game's ask for its window with its own",
        );
        let made: orb_core::window::CreateWindowExW = unsafe { std::mem::transmute(made) };
        // Neither string, because neither is read: the class and the title are this game's own, orb's
        // rewrite of the wide call looks at neither — the class is what the *other* rewrite decides a
        // window by, `CreateWindowExA`'s — and the simulated host that makes the window looks at neither
        // either. A title invented here would be a string of this fake's that no launch has ever shown.
        let window = unsafe {
            made(
                0,
                std::ptr::null(),
                std::ptr::null(),
                ASKED_STYLE,
                ASKS_AT.0,
                ASKS_AT.1,
                ASKS_FOR.0,
                ASKS_FOR.1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        let window = orb_api::Hwnd(window as usize);
        self.window.set(window);
        window
    }

    /// **The window this game puts on the screen itself**, through the name it asks the host for it by.
    ///
    /// The real one looks `ShowWindow` up at its startup — a launch's log has
    /// `the game looked ShowWindow up` a millisecond before its window is made — and a window orb had
    /// made hidden was on the screen one frame later, which is what this stands for. What orb does with
    /// the ask is `orb_core::window::show_window`'s, and what it answers is what this hands back.
    pub fn shows_its_window(&self) -> i32 {
        let shows =
            unsafe { orb_core::runtime::get_proc_address(0, c"ShowWindow".as_ptr().cast()) };
        assert_eq!(
            shows,
            orb_core::window::show_window as *const () as usize,
            "orb did not answer this game's ask for its window's show with its own",
        );
        let shows: orb_core::window::ShowWindow = unsafe { std::mem::transmute(shows) };
        unsafe { shows(self.window.get().0 as *mut c_void, SW_SHOW) }
    }

    /// **And the window placed**, `SetWindowPos` being the call that moves it and shows it at once — with
    /// `showing` saying whether this one carries `SWP_SHOWWINDOW`, which is the half orb strips while the
    /// window is being kept off the screen.
    pub fn places_its_window(&self, at: (i32, i32), showing: bool) -> i32 {
        let places =
            unsafe { orb_core::runtime::get_proc_address(0, c"SetWindowPos".as_ptr().cast()) };
        assert_eq!(
            places,
            orb_core::window::set_window_pos as *const () as usize,
            "orb did not answer this game's ask for its window's placing with its own",
        );
        let places: orb_core::window::SetWindowPos = unsafe { std::mem::transmute(places) };
        let flags = SWP_NOSIZE | SWP_NOZORDER | if showing { SWP_SHOWWINDOW } else { 0 };
        unsafe {
            places(
                self.window.get().0 as *mut c_void,
                std::ptr::null_mut(),
                at.0,
                at.1,
                0,
                0,
                flags,
            )
        }
    }

    /// **And the window's styles written as its window mode writes them**, through the name it asks the
    /// host for — the extended style first and then the style, the pair th06nc makes at
    /// `0x1400befee`/`0x1400bf011` and th06c at `0x14009d12e`/`0x14009d151`.
    pub fn restyles_its_window(&self) {
        let restyles =
            unsafe { orb_core::runtime::get_proc_address(0, c"SetWindowLongW".as_ptr().cast()) };
        assert_eq!(
            restyles,
            orb_core::window::set_window_long_w as *const () as usize,
            "orb did not answer this game's ask for its window's styles with its own",
        );
        let restyles: orb_core::window::SetWindowLongW = unsafe { std::mem::transmute(restyles) };
        let window = self.window.get().0 as *mut c_void;
        unsafe {
            restyles(window, GWL_EXSTYLE, THE_WINDOW_MODES_EX_STYLE as i32);
            restyles(window, GWL_STYLE, THE_WINDOW_MODES_STYLE as i32);
        }
    }

    /// Whether the window's own mode has been entered, and the size the picture was last set to — what
    /// `Th06nc::prepare_frame` does to this game's window on the first frame of a launch.
    pub fn its_window_was_remade(&self) -> bool {
        self.windows_remade.get()
    }

    pub fn the_picture_was_sized_to(&self) -> Option<(u32, u32)> {
        self.picture_sized_to.get()
    }

    /// **Whether the main menu is up, on the item that starts a run, and past the frames it ignores a
    /// press for** — which is the one screen orb has a question about and the one state a press at it is
    /// acted on in.
    pub fn the_menu_is_up_and_takes_a_press(&self) -> bool {
        self.image.the_screen_the_front_end_is_on() == SCREEN_MENU
            && self.image.the_item_the_cursor_is_on() == MENU_ITEM_RUN
            && self.image.frames_the_screen_has_been_up() >= THE_MENU_TAKES_A_PRESS_AFTER
    }

    /// **The main menu put up on the item that starts a run, past its grace** — the state an e2e test
    /// about orb's question over that screen wants, said rather than walked to.
    ///
    /// **Cut before it is registered**, which is the order the supervisor's own rebuild has and the reason
    /// this is not two lines: an element put into the chain twice links to itself, which is a walk of that
    /// chain that never ends.
    pub fn puts_its_menu_up(&self) {
        self.image.cuts_the_front_end();
        self.image.registers_the_front_end();
        self.image.the_front_end_is_on(SCREEN_MENU, MENU_ITEM_RUN);
        self.image
            .the_screen_has_been_up_for(THE_MENU_TAKES_A_PRESS_AFTER * 3);
    }

    // ---- the calc chain ----

    /// `0x14003d700`: every job of the calc list in turn, and what the walk does with each answer —
    /// answering nought to carry the frame on and one to leave it, which is what the real walk answers.
    fn runs_the_calc_chain(&self) -> u8 {
        let mut at = self.image.calc_chain_head();
        let mut ran = 0;
        while at != 0 {
            assert!(
                ran <= 64,
                "this game's calc chain has run {ran} job(s), which is more than it has"
            );
            let callback = self.image.chain_callback(at);
            if callback == 0 {
                at = self.image.chain_next(at);
                continue;
            }
            let answered = loop {
                match self.job(callback) {
                    chain_result::AGAIN => continue,
                    answer => break answer,
                }
            };
            let next = self.image.chain_next(at);
            if answered == chain_result::REMOVED {
                self.image.cuts_from_the_chain(at);
            } else if answered == chain_result::OVER {
                break;
            }
            ran += 1;
            at = next;
        }
        u8::from(self.leaving.get())
    }

    fn job(&self, callback: usize) -> i32 {
        match callback {
            chain_job::SUPERVISOR => self.supervisor(),
            chain_job::FRONT_END => self.front_end(),
            chain_job::GAME_MANAGER => self.stage(),
            // **The player's own job, which is where a stage's frame is already done.** What the real
            // one's does — the word read, the player moved, the hit test — this game does inside the
            // manager's above, one frame of a stage being one call here. What it is in the chain for is
            // what orb reads it for: its being there is what says a stage is running.
            chain_job::PLAYER => chain_result::CONTINUES,
            chain_job::RESULT_SCREEN => self.result(),
            chain_job::ENDING => self.ending(),
            other => {
                panic!("a job of this chain has {other:#x} as its callback, which is no job of it")
            }
        }
    }

    /// The supervisor's own update at chain priority 0: the read of the frame's keys into the word the run
    /// acts on, with the frame before's written from it first — `0x14007b8a6` onwards — and the state that
    /// has been asked for built.
    fn supervisor(&self) -> i32 {
        let frames = self.frames_since_launch.get() + 1;
        self.frames_since_launch.set(frames);
        if frames == LOGO_COMES_UP_AFTER && self.image.the_scene_now() == 0 {
            self.image
                .is_in_scene(0, self.image.scene_of_the_front_end());
        }
        self.reads_the_keys();
        let wanted = self.image.the_scene_wanted();
        let acted_on = self.image.the_scene_now();
        if wanted != acted_on {
            self.builds(wanted, acted_on);
        }
        self.image
            .is_in_scene(self.image.the_scene_wanted(), self.image.the_scene_wanted());
        chain_result::CONTINUES
    }

    /// `0x1400abc90` and `0x140013310`: the keyboard read this game looked up, translated through its
    /// table into the 256 keys it plays off, and the word built out of them — with the frame before's
    /// written from the word first, which is the order the supervisor has it in.
    fn reads_the_keys(&self) {
        self.image
            .the_word_before_is(self.image.the_word_the_run_acts_on());
        let mut snapshot = [0u8; 256];
        unsafe { orb_core::runtime::keyboard_state(snapshot.as_mut_ptr()) };
        let mut keys = [0u8; 256];
        for (key, byte) in snapshot.iter().enumerate() {
            if let Some(index) = self.image.the_index_of_the_key(key as u8) {
                keys[index] = *byte;
            }
        }
        self.image.the_game_reads_the_keys(&keys);
        let word = DIK_BITS
            .iter()
            .filter(|(index, _)| keys[*index] & 0x80 != 0)
            .fold(0u16, |word, (_, bits)| word | bits);
        self.image
            .the_run_acts_on_the_word(u32::from(word | the_pads_word()));
    }

    fn pressed(&self, mask: u16) -> bool {
        let word = self.image.the_word_the_run_acts_on() as u16;
        let before = self.image.the_word_before() as u16;
        word & mask != 0 && before & mask == 0
    }

    /// What the supervisor does with a state that has been asked for: takes down what was running and
    /// builds the one wanted.
    fn builds(&self, wanted: i32, acted_on: i32) {
        let front_end = self.image.scene_of_the_front_end();
        let run = self.image.scene_of_a_run();
        let rebuilding = self.image.scene_of_a_stage_being_rebuilt();
        let run_over = self.image.scene_of_a_run_over();
        // **The front end, and the state a run given up asks for with it**: `0x14007bb13`'s whole work is
        // the stage's two elements out, the two replay flags cleared and 1 — the front end — asked for
        // itself, so the two arrive at the same place. **And neither builds the screen a run ends at**,
        // which is `game::th06nc::STATE_THE_RUNS_OWN_END`'s own: that is what makes `Game::run_finished`
        // answer false for a run given up and true for one that finished.
        if wanted == front_end || wanted == run_over {
            // **And the front end asked for, which is what `0x14007bb13` does with its own state**: the
            // state left standing is the one the next transition is read against, and a run given up that
            // left it on the run-over's own number is a launch where the run after it never starts —
            // `wanted == run && acted_on == front_end` is the branch that cuts the front end and
            // registers a stage, and 8 is not the front end. A second run in one launch is what found
            // it: `an_attempt_at_a_card_counted_and_kept`.
            self.image.is_in_scene(acted_on, front_end);
            self.tears_the_stage_down();
            self.image.the_screen_a_run_ends_at_is_down();
            self.image.the_ending_is_over();
            self.image.registers_the_front_end();
            // **Three screens a rebuilt front end comes up on, and `acted_on` is what tells them apart** —
            // each measured on the real game:
            //
            // - **its logo** on a launch with no scene yet, `front end: screen 1` on the first lines of
            //   every launch;
            // - **the title** where what it is rebuilt from is the screen a run that *finished* ends at:
            //   the launch of `the-offer-written-away.yaml` read `result: the screen a run ends at is
            //   down` and then `front end: screen 2` a quarter of a second later, with no 16 between them;
            // - **and 16, the attract cycle**, where it is rebuilt from a run — `the-front-end-after-a-run
            //   .yaml` read `front end: screen 16` 16ms after `leave: true`, 17 one frame later and 18
            //   seven frames after that.
            let coming_up_on = if acted_on == NO_SCENE {
                SCREEN_LOGO
            } else if acted_on == self.image.scene_of_the_runs_own_end() {
                SCREEN_TITLE
            } else {
                SCREEN_BACK_FROM_A_RUN
            };
            self.goes_to(coming_up_on, self.left_the_menu_on.get());
            self.image.is_not_paused();
            self.out_of_lives.set(false);
        } else if wanted == run && acted_on == front_end {
            // **And the word a run acts on emptied with the frame before's**, which is what the front
            // end's own case does as a run starts: `0x140049234` writes nought into `0x140abae80`,
            // `0x140abae84`, `0x140abae68` and `0x140abae7c` together, and `0x140049649` is a second case
            // of that table doing the same. The two repeat statics are not laid out here, nothing of orb
            // reading them; the pair an edge is taken from is, and starting a run with a key still in it
            // is a press inside the run that nobody made — see
            // [`the_frame_a_scene_of_th06nc_is_built_on`](crate::the_frame_a_scene_of_th06nc_is_built_on).
            self.image.the_run_acts_on_the_word(0);
            self.image.the_word_before_is(0);
            self.image.cuts_the_front_end();
            self.registers_the_stage();
        } else if wanted == rebuilding {
            // The stage's own jobs cut and the manager registered again, which is what a stage move is.
            //
            // **And the stage itself written nowhere here**: what raises it is the build's own
            // `0x14003c7c0`, which both of its paths reach — see [`stage_begun`]. Written at the
            // transition as well, the stage would go up twice a boundary.
            self.tears_the_stage_down();
            self.registers_the_stage();
            self.image.is_in_scene(run, run);
        } else if wanted == self.image.scene_of_the_runs_own_end() {
            // **A run that finished**, which is the one of the two states that builds the screen:
            // `0x14007b9e3` takes the stage's elements out, builds the screen and writes the name entry
            // into it — see `game::th06nc::STATE_THE_RUNS_OWN_END`.
            self.tears_the_stage_down();
            self.image.the_ending_is_over();
            self.image
                .the_screen_a_run_ends_at_is_up(RESULT_TYPING_A_NAME);
            // **With nothing typed into it yet**, which is how the real screen comes up: the `AAAAAAAA`
            // a launch registers is what the *rules* typed there, a character a decide while the shot is
            // held — see [`Fake::result`]. Built with eight already in it, the entry left on the first
            // decide and `play_until: { name: true }` had a state that stood for less than a frame.
            self.image.the_entry_holds("");
            self.image.the_entrys_cursor_is_on(0);
        }
    }

    /// **A stage of this game's own choosing built**, counted the way everything above `Game` counts, for
    /// an e2e test about a stage coming up that is not the one a resume asked for.
    ///
    /// The stage first and the callback second, which is the order the game has it in: the stage's build
    /// is what orb's hook stands over and it reads the stage out of the manager, so a stage written after
    /// it would be one orb never saw.
    ///
    /// **`stage` is counted from nought, the way `State::stage` is**, and not as the manager's own field
    /// holds it: the build counts that field on — `0x14003c7c0`, reached by both of its paths — so what
    /// goes in before it is the nought-counted number and what comes out is that number plus one. See
    /// [`stage_begun`], and `Th06nc::start_stage`, which writes the same number for the same reason.
    pub fn builds_the_stage(&self, stage: i32) {
        self.image
            .is_playing(stage, self.image.the_difficulty_now());
        self.registers_the_stage();
    }

    /// **A run of this game in progress in one of its stages**, said rather than walked to: the supervisor
    /// in the run's own scene and that stage built.
    ///
    /// **With the lives the configuration gives**, which is the game's own run start and not a detail of
    /// the scene: a stage entered with none of them is a stage whose first hit is the run's last.
    pub fn in_a_run_in_the_stage(&self, stage: i32) {
        let run = self.image.scene_of_a_run();
        self.the_games_own_run_start();
        self.image.is_in_scene(run, run);
        self.builds_the_stage(stage);
    }

    /// **And that same stage with the game's own playback in charge of it**, which is what a judging pass
    /// watches.
    pub fn watches_a_replay_of_the_stage(&self, stage: i32) {
        self.in_a_run_in_the_stage(stage);
        self.image.a_replay_is_playing(true);
    }

    /// **A stage asked for that never arrives**: the supervisor already in the run's own scene, so nothing
    /// is built and no stage build is entered.
    pub fn the_stage_never_comes_up(&self) {
        let run = self.image.scene_of_a_run();
        self.image.is_in_scene(run, run);
    }

    /// The game manager's element registered, whose build is the seam orb's hook stands over — entered
    /// with the manager, the way the chain enters it.
    fn registers_the_stage(&self) {
        self.image.registers_the_gameplay_scene();
        // **And the shot's phase back to idle**, which is what the game's own register leaves at
        // `0x140069e46` beside the eighty slots it clears: a stage carries none of its shots in, so a
        // stage played into from the one before it starts firing where a stage played into from the front
        // end does. See [`Fake::fires_a_shot`].
        self.image.the_shot_phase_is(NOT_FIRING.0, NOT_FIRING.1);
        let manager =
            std::ptr::without_provenance_mut(orb_core::game::th06nc::image::GAME_MANAGER_OBJECT);
        orb_core::runtime::stage_begun_in_ecx(manager);
    }

    fn tears_the_stage_down(&self) {
        self.image.cuts_the_gameplay_scene();
        self.bullets.borrow_mut().clear();
        self.writes_the_bullets_down();
        self.image.no_boss_is_fighting();
        self.image.everything_is_held(false);
    }

    /// The front end's own update at chain priority 2: its screens, walked by the word the read left.
    fn front_end(&self) -> i32 {
        let up_for = self.image.frames_the_screen_has_been_up().saturating_add(1);
        self.image.the_screen_has_been_up_for(up_for);
        let screen = self.image.the_screen_the_front_end_is_on();
        let cursor = self.image.the_item_the_cursor_is_on();
        let decide = self.pressed(MENU_DECIDE);
        let cancel = self.pressed(MENU_CANCEL);
        let items = match screen {
            SCREEN_MENU => MENU_ITEMS,
            SCREEN_DIFFICULTY => DIFFICULTIES_OFFERED,
            SCREEN_CHARACTER => CHARACTERS,
            SCREEN_SHOT_SELECT => SHOTS,
            _ => 0,
        };
        // The cursor, round: the menu steps over the Extra where the records hold no clear.
        if items > 0 {
            let step = i32::from(self.pressed(DOWN)) - i32::from(self.pressed(UP));
            let mut moved = (cursor + step).rem_euclid(items);
            if screen == SCREEN_MENU && !self.a_clear_is_in_the_copy() && moved == MENU_ITEM_EXTRA {
                moved = (moved + step).rem_euclid(items);
            }
            if moved != cursor {
                self.image.the_cursor_is_on(moved);
            }
        }
        let cursor = self.image.the_item_the_cursor_is_on();
        // **The four screens that press themselves through**, `0x1400491af`: the difficulty, the
        // character, the shot and the screen after them go by on their own fifteenth frame whatever
        // anybody presses.
        let goes_by_on_its_own = matches!(
            screen,
            SCREEN_DIFFICULTY | SCREEN_CHARACTER | SCREEN_SHOT_SELECT
        ) && up_for > A_SELECT_GOES_BY_ON_ITS_OWN_AFTER;
        match screen {
            SCREEN_LOGO if decide || up_for >= LOGO_LEAVES_ON_ITS_OWN_AFTER => {
                self.goes_to(SCREEN_TITLE, cursor);
            }
            SCREEN_TITLE if decide => self.goes_to(SCREEN_MENU, cursor),
            SCREEN_MENU if decide && up_for >= THE_MENU_TAKES_A_PRESS_AFTER => {
                self.left_the_menu_on.set(cursor);
                match cursor {
                    // **Three items walk into a run** — a plain one, the Extra and a stage practised
                    // on its own — and each of the three goes by way of the screen on its own, which
                    // is the state that loads the banner saying which kind of run it is
                    // (`0x14004c278`, inside state 8's case).
                    MENU_ITEM_RUN | MENU_ITEM_EXTRA | MENU_ITEM_PRACTICE => {
                        self.extra.set(cursor == MENU_ITEM_EXTRA);
                        // **And two of the three write the practice byte**: item 2 sets it and item 0
                        // clears it, which is what is read off them; the Extra's own case has no write
                        // of it, so this leaves it as it finds it.
                        if cursor == MENU_ITEM_RUN || cursor == MENU_ITEM_PRACTICE {
                            self.image
                                .the_item_asks_for_a_practice(cursor == MENU_ITEM_PRACTICE);
                        }
                        self.on_its_own_for.set(0);
                        self.goes_to(SCREEN_ON_ITS_OWN, cursor);
                    }
                    // **Every other item, and none of them opens a ranking**: three launches pressed
                    // the seven that do not start a run, and of the ten only 6, 7, 8 and 9 write
                    // anything — see `th06nc/asm.md`'s *The main menu's ten items*. The screen a run
                    // ends at is a run's own here and not a menu item's, which is why this game's
                    // `show_ranking` declines.
                    _ => {}
                }
            }
            // **A cancel at the menu moves the cursor onto the item that ends the game and leaves the menu
            // standing**, which is what a launch found: the recipe of
            // `an_attempt_at_a_card_counted_and_kept` taps the cancel until `screen: 3` is up and then
            // re-aims the cursor at item 0, and it would have reached no menu at all if the cancel had
            // gone back to the title.
            //
            // **The cursor alone and not the screen again**, which is what keeps the count of the
            // screen's own frames running: put through `goes_to` the menu would come back up inside its
            // own twenty-frame grace every time a cancel was tapped at it.
            SCREEN_MENU if cancel => self.image.the_cursor_is_on(MENU_ITEMS - 1),
            SCREEN_ON_ITS_OWN => {
                let on_its_own = self.on_its_own_for.get() + 1;
                self.on_its_own_for.set(on_its_own);
                if on_its_own >= ON_ITS_OWN_LASTS {
                    if self.extra.get() {
                        self.image.the_difficulty_is(DIFFICULTY_EXTRA);
                    }
                    self.goes_to(SCREEN_DIFFICULTY, DIFFICULTY);
                }
            }
            SCREEN_DIFFICULTY if decide || goes_by_on_its_own => {
                // **And an Extra run's difficulty is not written over by this screen's own decide**,
                // which a launch of `the_extra_played_to_its_clear` says: its recipe presses the decide
                // at screen 9 without moving the cursor — which stands on Normal — and the run that
                // started read `stage=6 diff=4`. The screen the menu's item opened is what wrote the
                // Extra's four, `0x14007bc97` being where the four is read.
                if !self.extra.get() {
                    self.image.the_difficulty_is(cursor);
                }
                self.goes_to(SCREEN_CHARACTER, i32::from(CHARACTER));
            }
            SCREEN_CHARACTER if decide || goes_by_on_its_own => {
                let character = u8::try_from(cursor).expect("a cursor over the characters");
                self.image.the_shot_select_decided(
                    self.image.the_difficulty_now(),
                    character,
                    SHOT_TYPE,
                );
                self.goes_to(SCREEN_SHOT_SELECT, i32::from(SHOT_TYPE));
            }
            SCREEN_SHOT_SELECT if decide || goes_by_on_its_own => {
                let shot = u8::try_from(cursor).expect("a cursor over the shots");
                self.image.the_shot_select_decided(
                    self.image.the_difficulty_now(),
                    self.image.the_character_chosen(),
                    shot,
                );
                self.goes_to(SCREEN_AFTER_THE_SHOT, 0);
            }
            SCREEN_AFTER_THE_SHOT if decide || up_for > A_SELECT_GOES_BY_ON_ITS_OWN_AFTER => {
                self.goes_to(THE_SCREEN_THAT_TAKES_THE_LAST_PRESS, 0);
            }
            // **The one screen before a stage that takes a press, and its own grace** — where orb's
            // question about picking a run up stands. A press inside the grace does nothing, which is
            // what makes the hold `Th06nc::menu_takes_a_press` answers here mean anything.
            THE_SCREEN_THAT_TAKES_THE_LAST_PRESS if decide && up_for >= THE_LAST_SCREENS_GRACE => {
                self.goes_to(THE_SCREEN_THAT_STARTS_THE_RUN, 0);
            }
            // **And its cancel is the way back one screen**, measured: a launch of
            // `the-cancel-at-the-last-screen.yaml` tapped the cancel at 24 and read
            // `front end: screen 14`, then tapped again and read `13`. Which is what
            // `the_run_of_th06nc_left_behind`'s own cancel test is about — orb holds that key back for
            // its question, and a key that reached the screen underneath would walk somebody answering
            // "neither" back out of the run they asked for.
            THE_SCREEN_THAT_TAKES_THE_LAST_PRESS if cancel => {
                self.goes_to(SCREEN_AFTER_THE_SHOT, 0);
            }
            SCREEN_AFTER_THE_SHOT if cancel => {
                self.goes_to(SCREEN_SHOT_SELECT, i32::from(SHOT_TYPE));
            }
            // **And the screen after it hands over on its own**, reading no word at all: the real one
            // calls the run's start as soon as the object it waits on is ready.
            THE_SCREEN_THAT_STARTS_THE_RUN => self.starts_the_run(),
            // **The two the front end passes through on its way back from a run**, each standing for the
            // frames the real one's stood for — a frame and seven of them, where a screen of its own lasts
            // seconds. **Counted rather than handed straight on**, because the supervisor's build and the
            // front end's own update are two jobs of one chain walk: handed on with no count, 16 would
            // have been written and left again inside the frame it came up on, and a walk of the screens
            // would never see it.
            SCREEN_BACK_FROM_A_RUN if up_for > SCREEN_BACK_FROM_A_RUN_LASTS => {
                self.goes_to(SCREEN_BEFORE_THE_REPLAY_LIST, cursor);
            }
            SCREEN_BEFORE_THE_REPLAY_LIST if up_for > SCREEN_BEFORE_THE_REPLAY_LIST_LASTS => {
                self.goes_to(THE_REPLAY_LIST, cursor);
            }
            // **And the replay list it comes to rest on**, which is where a launch that presses nothing
            // stays: six minutes of frames on it brought neither another screen nor a playback. What
            // leaves it is the cancel, and the menu it puts up has its cursor on the item that ends the
            // game — measured, which is why `an_attempt_at_a_card_counted_and_kept` re-aims the cursor
            // after tapping the cancel back to the menu.
            //
            // **And its own decide is declined here**, where the real one starts a replay of the game's
            // own: `front end: screen 20`, then `26`, then a playback orb keeps nothing of. Nothing of
            // orb calls through that, `Th06nc::menu_pointed_at` holding the press at the *item* this
            // screen is behind, so what a tap of the decide here is measured on is the real game.
            THE_REPLAY_LIST if cancel => self.goes_to(SCREEN_MENU, MENU_ITEMS - 1),
            _ => {}
        }
        chain_result::CONTINUES
    }

    /// **The front end moved to another of its screens**, which is the state's own case coming up: the
    /// state, the cursor, and the count of the screen's own frames back to nought.
    ///
    /// **That last is not a detail**: every state of the real one writes nought into `[front end + 0x9638]`
    /// as it comes up, and that count is what the main menu holds its twenty-frame grace against. A fake
    /// that carried the count over from the screen before puts the menu up already past its grace, and
    /// then a decide on the menu's first frame opens a run — which is the opposite of what
    /// `Th06nc::menu_takes_a_press` exists for, and what
    /// `the_main_menu_of_th06nc_walked::a_decide_in_the_first_twenty_frames_of_th06ncs_menu_is_not_acted_on`
    /// caught.
    fn goes_to(&self, screen: i32, cursor: i32) {
        self.image.the_front_end_is_on(screen, cursor);
        self.image.the_screen_has_been_up_for(0);
    }

    /// What the last screen before the stage decides: the game's own run start, which orb calls too.
    fn starts_the_run(&self) {
        self.the_games_own_run_start();
    }

    /// `0x14004f320`: the lives and the bombs out of the configuration — or the Extra's where the
    /// difficulty is its — the playback flag cleared, the track put to `-1`, the practice byte written
    /// from the front end's own, and the supervisor asked for the run's own state.
    fn the_games_own_run_start(&self) {
        if self.image.the_difficulty_now() == DIFFICULTY_EXTRA {
            self.image.has_lives_and_bombs(EXTRA_LIVES, EXTRA_BOMBS);
        } else {
            self.image.has_lives_and_bombs(CONFIG_LIVES, CONFIG_BOMBS);
        }
        // **The one field a run carries out of the item it came from**: `0x14004f320` copies the front
        // end's `+0x168d1` into `0x14053d404`, which is what `Th06nc::read_state` answers `practice`
        // off once the run is under way.
        self.image
            .a_run_is_practice(self.image.does_the_item_ask_for_a_practice());
        self.image.a_replay_is_playing(false);
        self.image.plays_the_track(-1);
        // **And the stage the run starts in, which the supervisor's own case writes and not the front
        // end**: `0x14007bc9c` writes **6** where the difficulty is the Extra's — the build then counts it
        // on to seven — and `0x14007bccb` nought for every other run. (Its third path, `0x14007bcd3`,
        // decrements instead where the practice byte or the scene's is set, so that the build lands on the
        // stage the front end chose; nothing here lays that screen out.)
        let extra = self.image.the_difficulty_now() == DIFFICULTY_EXTRA;
        self.image.is_playing(
            if extra { THE_EXTRAS_STAGE } else { 0 },
            self.image.the_difficulty_now(),
        );
        self.image
            .is_in_scene(self.image.the_scene_now(), self.image.scene_of_a_run());
    }

    /// The game manager's own update at chain priority 4: one frame of the stage.
    fn stage(&self) -> i32 {
        let state = self.state();
        // The pause menu put up and taken down off the button's edge, and the run's update held while it
        // is.
        if self.image.is_it_paused() {
            if self.pressed(MENU_DECIDE) {
                self.image.is_not_paused();
            }
            return chain_result::CONTINUES;
        }
        if self.pressed(PAUSE) && !self.out_of_lives.get() {
            self.image.is_paused();
            return chain_result::CONTINUES;
        }
        // Out of lives: the game's own menu, which nothing here answers yes to — a decide ends the run.
        // **Into the run's own end and not into the state a give-up writes**: this build offers no
        // continue, so a run out of lives is a run finished and the screen it ends at comes up.
        if self.out_of_lives.get() {
            if self.pressed(MENU_DECIDE) {
                self.image.is_not_paused();
                self.image.is_in_scene(
                    self.image.the_scene_now(),
                    self.image.scene_of_the_runs_own_end(),
                );
            }
            return chain_result::CONTINUES;
        }
        let frames = self.image.frames_in_the_stage() + 1;
        self.image.has_been_in_the_stage_for(frames);
        // And the stage's own script, which the enemies' job steps at its own end — the clock a midstage
        // boundary is counted in.
        self.image
            .the_script_has_run_for(state.script_frames.saturating_add(1));
        // One number a frame out of the generator, which is why a stage played again from another seed is
        // another stage.
        self.image.draws_from_the_generator();
        self.plays_the_stage(frames);
        // The stage over: the next built through the state the game rebuilds its manager in, and the last
        // hands over to the ending.
        if Some(frames) == self.stage_frames.get() {
            // **`State::stage` is counted from nought and the manager's own from one**, which is what
            // the count below is in: `Th06nc::read_state` takes one off the number it reads, so a run is
            // on its last stage where `state.stage + 1` has reached the count. Nothing here writes the
            // stage — the build is what raises it, see [`stage_begun`].
            if state.stage + 1 < STAGES_A_RUN_PLAYS && !self.extra.get() {
                self.image.is_in_scene(
                    self.image.the_scene_now(),
                    self.image.scene_of_a_stage_being_rebuilt(),
                );
            } else if self.extra.get() {
                // **The Extra ends into the screen a run ends at and not into an ending**, which is what
                // the real game does: the numbered stages' sixth hands over to the ending's own job and
                // the Extra reaches its clear screen and the result instead — see `orb_real`'s
                // `the_extra_of_th06nc_played_to_its_clear`, whose check is written around there being no
                // ending to read.
                self.image.is_in_scene(
                    self.image.the_scene_now(),
                    self.image.scene_of_the_runs_own_end(),
                );
            } else {
                self.enters_the_ending();
            }
        }
        chain_result::CONTINUES
    }

    /// The script's events for the frame the stage has reached, the bullets, the player and the hit test.
    fn plays_the_stage(&self, now: u32) {
        let last = self.last_stage_frame.get();
        if now < last {
            // A stage put back is a stage whose bullets are gone: what a restore could not put back is
            // dropped to agree with the memory it did.
            self.bullets.borrow_mut().clear();
            self.respawn_left.set(0);
            self.bomb_left.set(0);
            self.image.a_bomb_is_going_off(false);
        } else {
            let reached = |at: u32| last < at && at <= now;
            // **Each attack from nought on the boss's own timer**, which is what the game says the fight moved
            // on by — see `enemy::TIMER` in `th06nc/mod.rs`.
            if reached(MIDBOSS_ARRIVES) || reached(CARD_STARTS) || reached(ATTACK_CHANGES) {
                self.image.a_boss_is_fighting(BOSS_LIFE, 0);
            } else if self.state().boss_present && now > last {
                self.image.the_bosses_attack_runs_on();
            }
            // **The card cast, named and counted, which is the whole of what a card's own start does** —
            // see [`THE_CARD_THE_STAGE_CASTS`]. The attack after it is a card no longer.
            if reached(CARD_STARTS) {
                self.image
                    .the_card_is_named(THE_CARD_THE_STAGE_CASTS, ITS_NAME);
                self.image.the_card_has_been_attempted(
                    THE_CARD_THE_STAGE_CASTS,
                    self.image.card_attempts(THE_CARD_THE_STAGE_CASTS) + 1,
                );
                self.image.the_spellcard_is(
                    u32::try_from(THE_CARD_THE_STAGE_CASTS)
                        .expect("a card of a hundred and thirty-four"),
                );
            }
            if reached(ATTACK_CHANGES) {
                self.image.no_spellcard_is_running();
            }
            // **Nearly out of life a second before she goes**, which is how the real game's bosses leave:
            // one that was beaten read `5` on the last frame she stood on where one a run was lost under
            // stood at whatever her attack held — see `orb_real`'s
            // `the_extra_of_th06nc_played_to_its_clear`, whose check is that difference. **On its own
            // frame and not the frame she leaves on**, which is the frame a trace could read it at all.
            if reached(A_BOSS_IS_NEARLY_OUT_AT) {
                self.image.a_boss_is_fighting(
                    A_BEATEN_BOSSES_LIFE,
                    self.image.the_bosses_attack_has_run_for(),
                );
            }
            if reached(MIDBOSS_BEATEN) {
                self.image.no_boss_is_fighting();
            }
            if self.fires.get()
                && now > last
                && now >= FIRST_BULLET_AT
                && (now - FIRST_BULLET_AT).is_multiple_of(A_BULLET_EVERY)
            {
                self.shoots_at_the_player();
            }
        }
        self.last_stage_frame.set(now);

        let word = self.image.the_word_the_run_acts_on() as u16;
        self.moves_the_player(word);
        self.fires_a_shot(word);
        let player = self.image.where_the_player_is();
        {
            let mut bullets = self.bullets.borrow_mut();
            for bullet in bullets.iter_mut() {
                bullet.at = (bullet.at.0 + bullet.step.0, bullet.at.1 + bullet.step.1);
            }
            bullets.retain(|bullet| {
                bullet.at.0 >= FIELD.0 - BULLET_SIZE
                    && bullet.at.0 <= FIELD.0 + FIELD.2 + BULLET_SIZE
                    && bullet.at.1 >= FIELD.1 - BULLET_SIZE
                    && bullet.at.1 <= FIELD.1 + FIELD.3 + BULLET_SIZE
            });
        }
        match self.image.the_players_state() {
            // Playing: the hit test, and the hit that starts the window a bomb may still be taken in.
            0 => {
                let reach = PLAYER_HALF + BULLET_SIZE / 2.0;
                let hit = self.bullets.borrow().iter().position(|bullet| {
                    (bullet.at.0 - player.0).abs() <= reach
                        && (bullet.at.1 - player.1).abs() <= reach
                });
                if let Some(hit) = hit {
                    self.bullets.borrow_mut().remove(hit);
                }
                if hit.is_some() || self.hit.take() {
                    self.image.the_player_has_been_hit();
                    self.image
                        .a_bomb_still_takes_the_hit_back_for(A_BOMB_IS_TAKEN_FOR);
                }
            }
            // Hit, inside the window: a bomb pressed takes the hit back, and the window running out is the
            // death.
            2 => {
                let (_, bombs) = self.image.lives_and_bombs();
                let window = self.image.the_frames_a_bomb_still_takes_the_hit_back_in();
                if word & BOMB != 0 && bombs > 0 && window > 0 {
                    self.a_bomb_goes_off();
                    self.image.the_player_is_playing();
                    self.image.a_bomb_still_takes_the_hit_back_for(0);
                } else if window <= 0 {
                    self.dies();
                } else {
                    self.image.a_bomb_still_takes_the_hit_back_for(window - 1);
                }
            }
            // Coming back, which is what a stage's own register and a death both leave: the walk-in's
            // thirty frames, and then the invulnerability the gate at `0x14006a6a9` hands it over to.
            1 => {
                let left = self.respawn_left.get().saturating_sub(1);
                self.respawn_left.set(left);
                if left == 0 {
                    self.image.the_player_is_invulnerable_for(INVULNERABLE_FOR);
                    self.image.everything_is_held(false);
                    self.image
                        .the_player_is_at(PLAYER_STARTS_AT.0, PLAYER_STARTS_AT.1);
                }
            }
            // And the invulnerability counting itself down to nothing, which is the state the player is
            // played in and no hit lands in.
            3 => {
                let left = self.image.the_frames_the_player_is_untouchable_for() - 1;
                if left <= 0 {
                    self.image.the_player_is_playing();
                } else {
                    self.image.the_player_is_invulnerable_for(left);
                }
            }
            _ => {}
        }
        if self.bomb_left.get() > 0 {
            self.bomb_left.set(self.bomb_left.get() - 1);
            // The bomb over, which is what each of the game's own four ends does to that byte.
            if self.bomb_left.get() == 0 {
                self.image.a_bomb_is_going_off(false);
            }
        }
        self.writes_the_bullets_down();
    }

    /// The death made final: the life taken, and the player's respawn begun with everything held; or the
    /// retry menu where there is no life to take.
    fn dies(&self) {
        let (lives, bombs) = self.image.lives_and_bombs();
        if lives <= 0 {
            self.out_of_lives.set(true);
            self.image.is_in_the_retry_menu();
            self.image.the_player_is_playing();
            return;
        }
        self.image
            .has_lives_and_bombs(lives - 1, bombs.max(CONFIG_BOMBS));
        // Nothing on the count, which is what no state but the invulnerability's reads: the walk-in runs
        // on `respawn_left` and hands the count over itself.
        self.image.the_player_is_coming_back(0);
        self.image.everything_is_held(true);
        self.respawn_left.set(RESPAWN_TAKES);
    }

    /// A bomb pressed: one fewer, and the bullets swept off the field for as long as it goes off.
    fn a_bomb_goes_off(&self) {
        let (lives, bombs) = self.image.lives_and_bombs();
        self.image.has_lives_and_bombs(lives, bombs - 1);
        self.image.a_bomb_is_going_off(true);
        self.bomb_left.set(A_BOMB_LASTS);
        self.bullets.borrow_mut().clear();
    }

    fn shoots_at_the_player(&self) {
        let player = self.image.where_the_player_is();
        let (dx, dy) = (player.0 - BULLETS_FROM.0, player.1 - BULLETS_FROM.1);
        let length = (dx * dx + dy * dy).sqrt().max(1.0);
        let mut bullets = self.bullets.borrow_mut();
        let Some(slot) =
            (0..BULLET_SLOTS_USED).find(|slot| bullets.iter().all(|bullet| bullet.slot != *slot))
        else {
            return;
        };
        bullets.push(Bullet {
            slot,
            at: BULLETS_FROM,
            step: (dx / length * BULLET_SPEED, dy / length * BULLET_SPEED),
        });
    }

    /// The player moved by the directions in the word, by the step a frame moves them — halved where the
    /// focus is held — and held inside the field.
    fn moves_the_player(&self, word: u16) {
        if self.respawn_left.get() > 0 {
            return;
        }
        let across = i32::from(word & RIGHT != 0) - i32::from(word & LEFT != 0);
        let down = i32::from(word & DOWN != 0) - i32::from(word & UP != 0);
        if across == 0 && down == 0 {
            return;
        }
        let mut step = if across != 0 && down != 0 {
            A_STEP / 2.0_f32.sqrt()
        } else {
            A_STEP
        };
        if word & FOCUS != 0 {
            step /= 2.0;
        }
        let (x, y) = self.image.where_the_player_is();
        self.image.the_player_is_at(
            (x + across as f32 * step).clamp(FIELD.0, FIELD.0 + FIELD.2),
            (y + down as f32 * step).clamp(FIELD.1, FIELD.1 + FIELD.3),
        );
    }

    /// **The phase of the shot the player is firing, stepped the way the game's own update steps it**:
    /// `0x14006ae14` starts a cycle where the composed word's `SHOT` is held and the pair is idle,
    /// `0x14006b2c0` makes an idle pair an update nothing comes out of, and `0x14006b38a` puts the phase
    /// up one with what it was beside it, ending the cycle at [`A_FIRING_CYCLE_LASTS`].
    ///
    /// **The pair and not the shots it fills.** What goes into the eighty slots at `player::SHOTS` is
    /// decided by the shot type's own function — `[player+0x7720]` or `[player+0x7728]`, picked by the
    /// focus — which is handed this phase and answers whether to fire at it; which phases it fires on is
    /// not read off the exe, so a fake filling those slots would be inventing a rate. What is modelled is
    /// the number every one of those calls is made with, and that is the field two passes of one stage
    /// can disagree about.
    ///
    /// **And that is a divergence this names rather than hides**, of the class `CLAUDE.md` calls a defect:
    /// an e2e test it makes pass that the real game would fail. `Th06nc::the_worlds_fingerprint` folds the
    /// eighty slots into `shots=`, `fired=` and `shot_at=`, and over this fake all three are nought on
    /// every frame — so `a_pointdevice_run_of_th06nc`'s comparison of a playback to its run agrees about
    /// them for want of anything to disagree about, where on the real game a chapter of stage 3 parts in
    /// exactly what those folds are of. **What it would take to close it is a rate read off the exe**: the
    /// two shot functions this phase is handed to, at `[player+0x7720]` and `[player+0x7728]`, and which
    /// of a cycle's thirty updates each of them answers `>= 0` at. Until that reading exists the folds are
    /// an instrument for the real game and not for this one, and the phase above is the part of it a fake
    /// can be faithful about.
    ///
    /// **And the press is the half a walk-in and a hit skip, where the advance is not.** The branch at
    /// `0x14006a753` sends states 1 and 2 — the walk-in and the hit — to `0x14006ae58`, which is past the
    /// press at `0x14006ae14` and before the advance at `0x14006b2b8`: so a player in either starts no
    /// cycle and still carries one already out to its end.
    ///
    /// **And not the conversation's own gate either** — `0x14006ae20` skips a press while
    /// `[[0x140abae28]+0x36b0]` is nought or more — this game's fake having no conversation to be in.
    fn fires_a_shot(&self, word: u16) {
        // The walk-in and the hit, the two states the press is skipped in.
        let pressing = !matches!(self.image.the_players_state(), 1 | 2);
        let phase = match self.image.the_shot_phase().0 {
            phase if phase < 0 && pressing && word & SHOT != 0 => THE_FIRST_PHASE,
            phase => phase,
        };
        if phase < 0 {
            return;
        }
        let (phase, before) = match phase + 1 >= A_FIRING_CYCLE_LASTS {
            true => NOT_FIRING,
            false => (phase + 1, phase),
        };
        self.image.the_shot_phase_is(phase, before);
    }

    fn writes_the_bullets_down(&self) {
        let bullets = self.bullets.borrow();
        for slot in 0..BULLET_SLOTS_USED {
            match bullets.iter().find(|bullet| bullet.slot == slot) {
                Some(bullet) => self.image.a_bullet_of_size_is_at(
                    slot,
                    bullet.at.0,
                    bullet.at.1,
                    bullet.step,
                    (BULLET_SIZE, BULLET_SIZE),
                ),
                None => self.image.the_bullet_is_gone(slot),
            }
        }
        self.image
            .bullets_alive(i32::try_from(bullets.len()).expect("a handful of bullets"));
        // **The boss and the wave both**, which is what the game's own manager leaves: it walks its slots
        // and counts every one that is alive, and a boss is one of them.
        self.image
            .enemies_alive(i32::from(self.state().boss_present) + self.enemies.get());
    }

    /// The ending a cleared run hands over to: the stage's jobs gone, the ending's own in their place,
    /// with the supervisor's state still a stage's.
    fn enters_the_ending(&self) {
        self.tears_the_stage_down();
        self.image.an_ending_is_running();
        self.ending_updates.set(ENDING_UPDATES);
        self.roll_updates.set(ROLL_UPDATES);
    }

    /// The ending's own job: its script and then the roll, and the screen a run ends at after both.
    fn ending(&self) -> i32 {
        if self.ending_updates.get() > 0 {
            self.ending_updates.set(self.ending_updates.get() - 1);
        } else if self.roll_updates.get() > 0 {
            self.roll_updates.set(self.roll_updates.get() - 1);
        } else {
            self.image.is_in_scene(
                self.image.the_scene_now(),
                self.image.scene_of_the_runs_own_end(),
            );
            return chain_result::REMOVED;
        }
        chain_result::CONTINUES
    }

    /// The screen a run ends at, walked by the presses whoever finished the run makes: the name entry on
    /// to the way out, which is the state orb's `leave_the_screen_a_run_ends_at` writes itself. Leaving
    /// takes the job out and puts the supervisor back.
    ///
    /// **The file is written by the entry's own decide and not by the leaving**, which is the game's
    /// shape: `0x14007301f` hands the name at `+0x9e88` and a record built out of it to `0x14006db10`,
    /// and that is the one call of the seven that writes — see `th06nc/asm.md`. So orb writing the way
    /// out over this screen writes nothing, which is what it does on the real game too.
    ///
    /// **One entry where the game has two**, and that is a divergence this owes: the game's other entry,
    /// at `+0x9e70`, copies its eight bytes into `th06.cfg`'s object instead, and which of the two a
    /// clear of this game arrives at has not been driven — `asm.md`'s *What is not established* carries
    /// it. `Th06nc::types_its_name` writes `+0x9e70`, and this entry is the one that writes.
    ///
    /// **And the offer to save a replay stands between the entry and the way out**, which is where the
    /// real one puts it: a launch that held every state of a run that ended walked `9, 15, 16, 10, 12,
    /// 14` and the question **「今回のリプレイを保存しますか？」** was photographed at fifteen. That is
    /// the state `Th06nc::skip_replay_prompt` writes the way out over, so a run of orb's is never asked.
    ///
    /// **Four states of it are not laid out**, and a test that pressed through them would be asserting
    /// this file rather than the game: sixteen, ten and twelve are the phases the real screen animates
    /// the question through, and fourteen is where the answer is taken — `0x1400734f3` raising it over a
    /// replay slot that already holds one. What orb reaches is fifteen, which every path to the others
    /// goes through, so the one state is what this game owes.
    fn result(&self) -> i32 {
        let state = self.image.the_state_the_screen_a_run_ends_at_is_in();
        if state == RESULT_LEAVING {
            self.image.is_in_scene(
                self.image.the_scene_now(),
                self.image.scene_of_the_front_end(),
            );
            return chain_result::REMOVED;
        }
        if self.pressed(MENU_DECIDE) {
            if state == RESULT_SAVE_REPLAY_QUESTION {
                self.image.the_screen_a_run_ends_at_is_in(RESULT_LEAVING);
            } else if self.image.the_entrys_cursor() == RESULT_THE_END_ITEM {
                // **The entry's own end item**, which is the decide that copies the eight bytes out and
                // writes fifteen — `0x1400727d5`.
                self.writes_the_score_file();
                self.image
                    .the_screen_a_run_ends_at_is_in(RESULT_SAVE_REPLAY_QUESTION);
            } else {
                // **And every other decide at the entry types a character rather than leaving it**, which
                // is what the real one does: `0x1400726f1` writes the character the cursor is on into
                // `+0x9e70 + typed`, `0x140072703` steps the count and `0x1400726fe` stops it at eight,
                // and `0x140072710` puts the cursor on the end item once eight are typed. **Which is why
                // a recipe's `play_until: { name: true }` has a state to see at all**: the rules hold the
                // shot while this screen is up, so an entry that left on the first decide was one whose
                // name entry stood for less than a frame — the whole game's own chain waited out its
                // bound against that.
                let typed = self.image.the_characters_typed();
                if typed < RESULT_NAME_BYTES {
                    let mut name = self.image.the_name_the_entry_holds();
                    name.push(THE_CHARACTER_THE_ENTRYS_CURSOR_STARTS_ON);
                    self.image.the_entry_holds(&name);
                    if self.image.the_characters_typed() >= RESULT_NAME_BYTES {
                        self.image.the_entrys_cursor_is_on(RESULT_THE_END_ITEM);
                    }
                }
            }
        }
        chain_result::CONTINUES
    }

    // ---- the score file ----

    /// `fopen("score.dat", ...)`, which reaches orb's fork as the `CreateFileW` it ends in: the fork
    /// decides which file the name lands in.
    fn opens_the_score_file(&self, write: bool) -> Option<String> {
        let (access, disposition) = if write { WRITE } else { READ };
        let name: Vec<u16> = SCORE_FILE
            .to_string_lossy()
            .encode_utf16()
            .chain([0])
            .collect();
        let handle = unsafe {
            orb_core::score::create_file_w(
                name.as_ptr(),
                access,
                0,
                std::ptr::null(),
                disposition,
                0,
                std::ptr::null_mut(),
            )
        };
        // The name is [`create_file_w`]'s own note of where the open landed, which orb's fork decided —
        // see `Fake::score_file_opens`, whose first entry is this read.
        (handle as isize != orb_core::score::INVALID_HANDLE).then(|| {
            self.opens
                .borrow()
                .last()
                .expect("an open just made")
                .0
                .clone()
        })
    }

    /// The one read of the score file this game makes, at its startup — and the copy every screen parses
    /// put together out of whichever files orb's fork says.
    fn reads_the_score_file_at_startup(&self) {
        let Some(landed) = self.opens_the_score_file(false) else {
            return;
        };
        self.loads_the_copy_out_of(&landed);
    }

    fn loads_the_copy_out_of(&self, name: &str) {
        if let Some(bytes) = self.sim().files().get(name) {
            self.image.the_score_file_copy_is(&bytes);
        }
    }

    /// A file of this game's, where orb's own read of it — `orb_api::fs::read`, by the same relative name
    /// the game opens it by — finds it too.
    fn puts_the_file(&self, name: &str, bytes: &[u8]) {
        self.sim().files().put(name, bytes);
    }

    /// Whether the copy the game parses holds a clear, which is what offers the Extra.
    fn a_clear_is_in_the_copy(&self) -> bool {
        self.the_records_the_game_parses()
            .is_some_and(|records| score_file::a_clear_is_among(&records))
    }

    /// What the screen a run ends at writes as it leaves: the records it parsed, back out through orb's
    /// fork.
    fn writes_the_score_file(&self) {
        let Some(landed) = self.opens_the_score_file(true) else {
            return;
        };
        let records = self
            .the_records_the_game_parses()
            .unwrap_or_else(|| records_holding(true));
        self.puts_the_file(&landed, &score_file::encode(&records, THE_GAMES_OWN_SEED));
    }
}

/// The records an installation somebody has played holds, with a clear among them or without one: a
/// `CLRD` for Normal whose two arrays of five say what each shot reached, [`CLEARED`] being a clear.
fn records_holding(a_clear: bool) -> Vec<Record> {
    let reached = match a_clear {
        true => [CLEARED; 5],
        false => [1; 5],
    };
    vec![
        score_file::th6k(),
        score_file::clrd(
            u8::try_from(DIFFICULTY).expect("a difficulty of four"),
            reached,
            reached,
        ),
    ]
}

/// The seeds the two files are written with, which are this laid-out game's own: the cipher takes any,
/// and two different ones are what says a read landed in the file it was meant to.
const THE_GAMES_OWN_SEED: u8 = 0x11;
const THE_MODES_SEED: u8 = 0x22;

/// `0x14004f320`: the game's own run start, which orb calls where it writes a run into being — entered
/// with the front end, the way the last screen before a stage calls it.
unsafe extern "win64" fn the_games_run_start(_front_end: usize) {
    running().the_games_own_run_start();
}

/// The game's `malloc`, as its import table holds it: a block off the laid-out heap.
unsafe extern "win64" fn malloc(len: usize) -> usize {
    running().image.takes_a_block_off_its_heap(len)
}

/// And its `free`: nothing, the heap never taking a block back.
unsafe extern "win64" fn free(_block: usize) {}

impl Drop for Fake {
    fn drop(&mut self) {
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

fn the_game_this_is() -> &'static dyn orb_core::game::Game {
    orb_core::game::found(EXE)
        .unwrap_or_else(|| panic!("{EXE} is a game orb knows"))
        .game
}

// ---- the six calls handed over ----

/// `0x14003d700`: the calc chain's walk, entered with the pointer the frame keeps its exit code in.
unsafe extern "win64" fn run_calc_chain(_leaving_with: *mut i32) -> u8 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.sim().clock().advance_micros(fake.work_this_frame());
    fake.runs_the_calc_chain()
}

/// `0x14007ccb0`: the camera, on the game's own object. Nothing a laid-out game draws turns on it.
unsafe extern "win64" fn moves_the_camera(_game: usize, _x: f32, _y: f32) {}

/// `0x1400019d0`: the draw begun, which puts the screenshot's two flags down.
unsafe extern "win64" fn begins_the_draw() {}

/// `0x1400855f0`: the viewport read, into the four ints handed over — the whole output here.
unsafe extern "win64" fn reads_the_viewport(
    x: *mut i32,
    y: *mut i32,
    width: *mut i32,
    height: *mut i32,
) {
    for (into, value) in [(x, 0), (y, 0), (width, 640), (height, 480)] {
        if !into.is_null() {
            unsafe { *into = value };
        }
    }
}

/// `0x140097160`: the viewport set, which the draw makes twice — once for the clear and once back.
unsafe extern "win64" fn sets_the_viewport(_x: i32, _y: i32, _width: i32, _height: i32) {}

/// `0x140081330`: the target cleared.
unsafe extern "win64" fn clears(_what: i32) {}

/// `0x140012e10`: the element taken out of whichever chain holds it.
unsafe extern "win64" fn takes_an_element_out(_: usize, _elem: usize) {}

/// `0x140083eb0`: the renderer's batch drawn, which is the one call a sprite queued is ever drawn by.
unsafe extern "win64" fn flushes_the_sprites(
    _sprites: usize,
    _count: i32,
    _topology: i32,
    _flushed_as: i32,
    _one: i32,
) {
    running().launch.asked_for(FLUSH);
}

/// `0x140001c70`: the screenshot's own path, which does nothing where nobody asked for one.
unsafe extern "win64" fn takes_the_screenshot() {}

/// The draw chain's one job, which is what drawing anything here is: a quad's worth of sprite queued into
/// the renderer's batch, as every draw of the real one queues its own.
unsafe extern "win64" fn draw_job(_on: usize) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.image.queues_sprites(4);
    chain_result::CONTINUES
}

/// `0x14003db50`: the frame put on the screen — **and the swapchain's own `Present` called from inside
/// it**, `0x140097150` being `jmp 0x140094620`. That call is the whole reason orb's overlay reaches this
/// game's screen: after `dxgi::take` the slot holds orb's own stand-in, which draws the overlay over the
/// finished frame and then presents.
unsafe extern "win64" fn ends_the_frame() {
    let fake = running();
    fake.pictures.set(fake.pictures.get() + 1);
    let swapchain = fake.image.the_swapchain();
    let vtable = fake.image.space().read::<usize>(swapchain);
    // **And the buffers made the window's size again where its client has changed**, which the real game
    // does as its window is resized — th06c's was measured asking `320x240` on every move orb made of the
    // window it plays in. Here rather than in a message, this fake having none.
    let client = fake.sim().windows().client(fake.window.get());
    if let Some(client) = client.filter(|client| Some(*client) != fake.client_was.get()) {
        fake.client_was.set(Some(client));
        const RESIZE_BUFFERS_SLOT: usize = 13;
        let resize = fake
            .image
            .space()
            .read::<usize>(vtable + size_of::<usize>() * RESIZE_BUFFERS_SLOT);
        if resize != 0 {
            let resize: unsafe extern "system" fn(usize, u32, u32, u32, u32, u32) -> i32 =
                unsafe { std::mem::transmute(resize) };
            unsafe {
                resize(
                    swapchain,
                    0,
                    client.width() as u32,
                    client.height() as u32,
                    0,
                    0,
                )
            };
        }
    }
    // The `Present` slot, counted in pointers as `orb_core::dxgi` counts it: `IUnknown`'s three,
    // `IDXGIObject`'s four and `IDXGIDeviceSubObject`'s one before it.
    const PRESENT_SLOT: usize = 8;
    let present = fake
        .image
        .space()
        .read::<usize>(vtable + size_of::<usize>() * PRESENT_SLOT);
    if present != 0 {
        let present: unsafe extern "system" fn(usize, u32, u32) -> i32 =
            unsafe { std::mem::transmute(present) };
        unsafe { present(swapchain, 0, 0) };
    }
}

/// **The first thing the frame's sound catch-up asks this game**, so this is where the sound's place in
/// the frame is noted — and it lands only on a frame that has music behind it, that catch-up doing nothing
/// on any other.
unsafe extern "win64" fn where_the_music_is() -> i64 {
    running().launch.asked_for(SOUND);
    0
}

unsafe extern "win64" fn how_long_the_track_is(_track: i32) -> i64 {
    0x10_0000
}

unsafe extern "win64" fn seeks_the_music(to: i64, track: i32) {
    running().seeks.borrow_mut().0.push((to, track));
}

/// **The window made over at the mode the settings asked for**, `0x14007e1b0` — which this game's frame
/// reaches on its first frame, once. What the real one does is destroy its window and make another; what
/// a laid-out game owes is the sizes an e2e test reads back, and the host's window is already the size
/// the display said.
unsafe extern "win64" fn makes_a_window(_game: usize) {
    running().windows_remade.set(true);
}

/// And the mode that covers the monitor, `0x14007df70`, which is the borderless window orb means by
/// fullscreen.
unsafe extern "win64" fn covers_the_monitor(_game: usize) {
    running().windows_remade.set(true);
}

/// And the picture sized again inside that window, `0x1400966cd0` — the call a launch that plays itself
/// makes with orb's own size after the mode has put the window back.
unsafe extern "win64" fn sizes_the_picture(width: u32, height: u32, _bits: u32, _hertz: u32) {
    running().picture_sized_to.set(Some((width, height)));
}

// ---- the swapchain ----

/// **`IDXGISwapChain::QueryInterface`, answering that this object is one.**
///
/// The IID is not compared against `orb_core`'s, which is the rule this file's numbers keep: any IID
/// answers, because the only ask orb makes of a swapchain is whether it is a swapchain.
unsafe extern "system" fn swapchain_query_interface(
    object: usize,
    _iid: *const u8,
    out: *mut usize,
) -> i32 {
    if !out.is_null() {
        unsafe { out.write(object) };
    }
    0
}

/// And the `Release` that gives that reference straight back, counted so that a test can say orb's ask
/// held none.
unsafe extern "system" fn swapchain_release(_object: usize) -> u32 {
    let fake = running();
    fake.swapchain_releases
        .set(fake.swapchain_releases.get() + 1);
    1
}

/// `IDXGISwapChain::GetDesc`, out of which orb reads the window this game plays in — the window the
/// simulated host made — and the size its buffers are.
unsafe extern "system" fn swapchain_describes_itself(_swapchain: usize, into: *mut u8) -> i32 {
    // `DXGI_SWAP_CHAIN_DESC`, 72 bytes: the mode's own 28 — of which the first two dwords are the
    // buffer's width and height, which `orb_core::dxgi::the_size_of` reads — the sample's 8, the usage
    // and the buffer count, and the window at 48, where `orb_core::dxgi::the_window` reads it.
    let buffers = running().buffers.get();
    let mut description = [0u8; 72];
    description[..size_of::<u32>()].copy_from_slice(&buffers.0.to_le_bytes());
    description[size_of::<u32>()..2 * size_of::<u32>()].copy_from_slice(&buffers.1.to_le_bytes());
    unsafe { std::ptr::copy_nonoverlapping(description.as_ptr(), into, description.len()) };
    const WINDOW_AT: usize = 48;
    unsafe {
        std::ptr::copy_nonoverlapping(
            super::WINDOW.0.to_le_bytes().as_ptr(),
            into.add(WINDOW_AT),
            size_of::<usize>(),
        )
    };
    0
}

/// **`IDXGISwapChain::ResizeBuffers`, the buffers made that size** — written down, so that a test can say
/// what size they are after the game asked.
unsafe extern "system" fn swapchain_resizes_its_buffers(
    _swapchain: usize,
    _buffers: u32,
    width: u32,
    height: u32,
    _format: u32,
    _flags: u32,
) -> i32 {
    running().buffers.set((width, height));
    0
}

/// **The swapchain's own `Present`, which is the frame handed over.** The same three things the game's own
/// present does in this fake: the call noted, when it happened written down, and the simulated display
/// told a frame arrived.
///
/// The moment is written down here and not only in [`present`] because this is the path orb's loop takes:
/// `Th06nc::end_drawing` calls `0x14003db50` whole, and that function presents through the swapchain. A
/// fake that noted the call and dropped the moment left `Launched::handovers_us` empty for every frame of
/// a run orb paced, so nothing could be read off a rate — which is what `the_pacing_of_th06nc.rs` asks.
unsafe extern "system" fn swapchain_presents(
    _swapchain: usize,
    _interval: u32,
    _flags: u32,
) -> i32 {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
    0
}

/// And `Present1`, which this game never calls itself — so nothing reaches it unless orb decides its
/// swapchain presents that way. Noted rather than left empty, so an e2e test can say which of the two
/// orb chose.
unsafe extern "system" fn swapchain_presents1(
    swapchain: usize,
    interval: u32,
    flags: u32,
    _parameters: *const u8,
) -> i32 {
    unsafe { swapchain_presents(swapchain, interval, flags) }
}

// ---- the host ----

/// **The host's own `GetProcAddress`**, which orb's stands in front of: the four names this game asks for
/// and nothing else.
unsafe extern "system" fn proc_address(_module: usize, name: *const u8) -> usize {
    let asked = unsafe { CStr::from_ptr(name.cast()) };
    match asked.to_bytes() {
        b"GetKeyboardState" => host_keyboard_state as *const () as usize,
        b"CreateDXGIFactory2" => host_creates_a_factory as *const () as usize,
        b"CreateWindowExW" => host_creates_a_window as *const () as usize,
        b"MoveWindow" => host_moves_a_window as *const () as usize,
        b"ShowWindow" => host_shows_a_window as *const () as usize,
        b"SetWindowPos" => host_places_a_window as *const () as usize,
        b"SetWindowLongW" => host_restyles_a_window as *const () as usize,
        _ => 0,
    }
}

/// **The host's own `CreateWindowExW`**, which orb's rewrite calls through: the simulated host makes the
/// window, of the whole size and at the position orb decided.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn host_creates_a_window(
    ex_style: u32,
    _class_name: *const u16,
    _window_name: *const u16,
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

/// **The host's own `ShowWindow`**, which puts the window on the screen: a show that reaches here is one
/// orb let through, and the whole of what a laid-out host does with it is write it down.
unsafe extern "system" fn host_shows_a_window(window: *mut c_void, _how: i32) -> i32 {
    i32::from(
        running()
            .sim()
            .windows()
            .show(orb_api::Hwnd(window as usize)),
    )
}

/// **And the host's own `SetWindowLongW`** for the two indices this game writes, answering what was there
/// the way the real one does; any other index writes nothing and answers nought, which is the real one's
/// answer for a failure.
unsafe extern "system" fn host_restyles_a_window(
    window: *mut c_void,
    index: i32,
    value: i32,
) -> i32 {
    let windows = running().sim().windows();
    let window = orb_api::Hwnd(window as usize);
    match index {
        GWL_STYLE => {
            let had = windows.style(window);
            windows.set_style(window, value as u32);
            had as i32
        }
        GWL_EXSTYLE => {
            let had = windows.ex_style(window);
            windows.set_ex_style(window, value as u32);
            had as i32
        }
        _ => 0,
    }
}

/// **And the host's own `SetWindowPos`**, which moves the window and — with `SWP_SHOWWINDOW` — shows it,
/// the two things that call does of which orb reads the position and that flag.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn host_places_a_window(
    window: *mut c_void,
    _after: *mut c_void,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    flags: u32,
) -> i32 {
    let windows = running().sim().windows();
    let window = orb_api::Hwnd(window as usize);
    let moved = if flags & SWP_NOSIZE == 0 {
        orb_api::Rect {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    } else {
        // The size kept, which is what `SWP_NOSIZE` asks for: the window's own, wherever it is going.
        let was = windows
            .asked_for(window)
            .unwrap_or(orb_api::Rect::sized(0, 0));
        orb_api::Rect {
            left: x,
            top: y,
            right: x + was.width(),
            bottom: y + was.height(),
        }
    };
    let placed = windows.move_window(window, moved);
    if flags & SWP_SHOWWINDOW != 0 {
        windows.show(window);
    }
    i32::from(placed)
}

/// **And the host's own `MoveWindow`**, which orb's rewrite calls through when it puts the window back in
/// its corner.
unsafe extern "system" fn host_moves_a_window(
    window: *mut c_void,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    _repaint: i32,
) -> i32 {
    let to = orb_api::Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };
    i32::from(
        running()
            .sim()
            .windows()
            .move_window(orb_api::Hwnd(window as usize), to),
    )
}

/// **The host's own `CreateDXGIFactory2`**, which orb's answer calls through to: the factory this game was
/// laid out with.
unsafe extern "system" fn host_creates_a_factory(
    _flags: u32,
    _iid: *const u8,
    out: *mut usize,
) -> i32 {
    if out.is_null() {
        return -1;
    }
    unsafe { out.write(running().image.the_factory()) };
    0
}

/// **And that factory's own `CreateSwapChainForHwnd`**, which orb's stand-in calls through to: the
/// swapchain this game was laid out with, which orb then takes the present of.
unsafe extern "system" fn factory_creates_a_swapchain(
    _factory: usize,
    _device: usize,
    _window: usize,
    _description: *const u8,
    _fullscreen: *const u8,
    _restrict: usize,
    out: *mut usize,
) -> i32 {
    if out.is_null() {
        return -1;
    }
    unsafe { out.write(running().image.the_swapchain()) };
    0
}

/// **The keyboard read this game looked up**, with the simulated host's keys behind it — which is what
/// orb's own read calls through to.
unsafe extern "system" fn host_keyboard_state(state: *mut u8) -> i32 {
    match orb_api::keyboard::state() {
        Some(keys) => {
            unsafe { std::ptr::copy_nonoverlapping(keys.as_ptr(), state, keys.len()) };
            1
        }
        None => 0,
    }
}

// ---- the originals ----

/// The game's own whole frame, for a launch with orb's loop off — which no launch of this game is, its
/// frame being a stretch of `main` that orb replaces outright. The pieces in the frame's own order all the
/// same.
///
/// **`win64` and not `fastcall`, as every one of this game's own functions here is**: this exe is 64-bit,
/// and `"fastcall"` is an x86 convention rustc refuses for a 64-bit target.
extern "win64" fn own_render(_window: *mut c_void) -> i32 {
    let chain = Th06nc.chain();
    unsafe { orb_core::runtime::run_calc_chain(chain) };
    unsafe { play_sounds(0) };
    unsafe { orb_core::runtime::run_draw_chain(chain) };
    unsafe { present(0) };
    FRAME_KEPT_RUNNING
}

/// The sound's own catch-up, as the frame has it: the time an e2e test says this frame's sounds cost.
unsafe extern "win64" fn play_sounds(_player: usize) {
    let fake = running();
    fake.launch.asked_for(SOUND);
    fake.sim().clock().advance_micros(fake.sound_this_frame());
}

/// The frame handed over, which from here is the compositor's.
///
/// **This is the frame-off path's, and [`swapchain_presents`] the frame loop's.** orb's own loop presents
/// through this game's swapchain — `FrameCalls::orbs_own`, and `dxgi::present` behind it — so what reaches
/// here is [`own_render`] and nothing else.
unsafe extern "win64" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
}

/// Never reached: this game's keyboard is read through the lookup orb answers, and its word is built in
/// [`Fake::reads_the_keys`].
extern "system" fn input() -> u16 {
    0
}

/// `0x14007a4a0`: the job registered three instructions past the seed's copy, which draws from the
/// generator as the stage is built.
extern "C" fn stage_building(_stage: i32) -> i32 {
    running().image.draws_from_the_generator();
    0
}

/// `0x14003c250`: the stage's own build, entered with the game manager in `rcx` — the stage raised, a
/// run's numbers where this is a run's start, the seed copied out of the generator with the count cleared,
/// and the job inside it registered through orb's hook, which is where a resumed run's seed goes in.
extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    let fake = running();
    let image = &fake.image;
    // **The state *asked for* and not the state acted on**, which is what the real build reads:
    // `0x14003c27c` takes the stage-following path where `[0x140c6dfbc]` — the supervisor's `+0x42c` — is
    // 3, and the supervisor's own job has not caught `+0x428` up to it yet when the build runs. Reading
    // the acted-on one took every transition down the run-start path instead, so a second stage began
    // with the score, the power, the point items and the extra lives all back to nought — which
    // `the_numbers_a_run_of_th06nc_was_made_with` is what caught.
    let starting = image.the_scene_wanted() != image.scene_of_a_stage_being_rebuilt();
    // **The stage counted on, whichever of the two paths ran**, which is what a launch of the real game
    // had to say: `0x14003c78b` jumps the run-start path into `0x14003c7a2`, so `mov r8d,[rdi+0x24]; lea
    // edx,[r8+1]; mov [rdi+0x24],edx` — the stage read and written back one higher — is in the code
    // *both* paths share and not in the following path alone. It is why a run's first stage comes up as
    // one out of the nought the front end leaves, and why the number a resume writes is the stage counted
    // from nought — see `Th06nc::start_stage`.
    //
    // **Laid out without it, a run picked up in its second stage came up in its second here and in its
    // third on the real game**: `resume: not put back — stage 3 came up where stage 2 was asked for`,
    // with the supervisor on 1 wanting 2 the whole way through the build — which is what says the branch
    // at `0x14003c27c` is not what raises it.
    let stage = image.the_stage_the_run_is_in() + 1;
    image.is_playing(stage, image.the_difficulty_now());
    if starting {
        image.fills_the_card_records();
        // **And the file's own card records over them**, which is the `0x14006e310(the parsed file,
        // manager + 0x968)` at `0x14003c5e7`: the layout above is what a run loses an attempt to and this
        // is what a file holding one gives it back. On the run-start branch alone, as that call is — a
        // stage a run walks into reads nothing.
        if let Some(records) = fake.the_records_the_game_parses() {
            image.the_files_card_records_are_laid_over(&records);
        }
        image.the_run_has_scored(0, 0);
        image.a_point_item_is_worth(0);
        image.shot_is_at_power(0);
        image.the_score_has_paid_for(0);
    }
    // **And what the stage-following path leaves alone, which is everything a chapter records**: the
    // score, the lives, the bombs, the power, the point items, what a point item is worth, the seed and
    // the extra lives are written nowhere in `0x14003c78d`, which is what this game's resume set was
    // read off. Its one write of the
    // score is the copy into the manager's `+0x938` at `0x14003c794`, a field two launches of the real
    // game read as nought across a boundary — so there is nothing here for it to do and no second score
    // field to do it to. See `game::th06nc::game_manager::SCORE`.
    //
    // What a point item is worth is `0x140549d2d`, and the write the branch above copies is
    // `0x14003c36d` — on the run-start path alone, the same as the three beside it.
    let (seed, _) = image.the_generator();
    image.the_stage_began_with_the_seed(seed);
    image.the_generator_is(seed, 0);
    image.has_been_in_the_stage_for(0);
    image.the_script_has_run_for(0);
    image.no_boss_is_fighting();
    image.everything_is_held(false);
    fake.bullets.borrow_mut().clear();
    fake.writes_the_bullets_down();
    fake.last_stage_frame.set(0);
    image.a_bomb_still_takes_the_hit_back_for(0);
    fake.bomb_left.set(0);
    image.a_bomb_is_going_off(false);
    orb_core::runtime::stage_building(stage);
    // And the player, after the stage's own build: where the stage starts them, walking back in.
    image.the_play_field_is(FIELD.0, FIELD.1, FIELD.2, FIELD.3);
    image.the_player_is_at(PLAYER_STARTS_AT.0, PLAYER_STARTS_AT.1);
    image.the_player_steps_by(A_STEP, A_STEP / 2.0_f32.sqrt());
    image.the_players_box_is(PLAYER_HALF);
    // **Coming back with the register's own count under it**, which is what the game's own register at
    // `0x140069b2c` leaves: the walk-in's thirty frames and then the invulnerability's `0xf0`, which is
    // the 270 frames of a stage nothing reaches the player in.
    image.the_player_is_coming_back(THE_REGISTER_LEAVES);
    fake.respawn_left.set(RESPAWN_TAKES);
    0
}

/// The three entry points only a 32-bit game reaches — see `orb`'s own, gated to `x86`.
extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    0
}

extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    0
}

unsafe extern "win64" fn card_records_read(_image: usize, _records: usize) -> i32 {
    0
}

extern "C" fn stop_recording() {}

extern "C" fn create_game_window(_instance: *mut c_void) {}

unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

extern "C" fn pad_buttons() -> *mut c_void {
    std::ptr::null_mut()
}

extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

extern "C" fn init_d3d_device() {}

/// The game's own `CreateFileW`, as its import table holds it: the name the open landed in written down,
/// and a handle answered where the file is there — or where it is being written — and
/// `INVALID_HANDLE_VALUE` where it is not.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_file_w(
    name: *const u16,
    access: u32,
    _share: u32,
    _security: *const c_void,
    _disposition: u32,
    _flags: u32,
    _template: *mut c_void,
) -> *mut c_void {
    let fake = running();
    let length = (0..)
        .take_while(|at| unsafe { *name.add(*at) } != 0)
        .count();
    let path = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(name, length) });
    let write = access & WRITE.0 != 0;
    let there = fake.sim().files().holds(&path);
    fake.opens.borrow_mut().push((path, write));
    if write || there {
        std::ptr::without_provenance_mut(fake.opens.borrow().len())
    } else {
        std::ptr::without_provenance_mut(orb_core::score::INVALID_HANDLE as usize)
    }
}

/// Never reached: this game's records go through [`create_file_w`] and not `CreateFileA`.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_file(
    _name: *const u8,
    _access: u32,
    _share: u32,
    _security: *const c_void,
    _disposition: u32,
    _flags: u32,
    _template: *mut c_void,
) -> *mut c_void {
    std::ptr::without_provenance_mut(usize::MAX)
}

/// The game's own window made, which orb's rewrite calls through: the host makes it.
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
