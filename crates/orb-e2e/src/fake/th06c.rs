//! A th06c that is not the real one: it owns its own memory, advances its own state, and calls orb's hooks
//! where the real game's code calls them.
//!
//! **It plays the game's part, not the address space's**, for the reason [`th06`](super::th06)'s does: its
//! state is the memory `orb_core::game::th06c::image` lays out, `read_state` is how orb learns what the run
//! is, and a chapter restored underneath it takes the run back with it because there is nothing else for the
//! run to be.
//!
//! **Its frame is orb's, the way the real one's is.** th06c hands its two chain walks and the whole frame
//! around them over — see `orb_core::game::th06c` — so a frame here is `orb_core::runtime::render`, whose
//! update is th06c's own walk of the calc chain calling *this* game's chain, whose draw is th06c's own walk of
//! the draw chain over an element this game registered, and whose eleven calls into the game — the walk, the
//! take-out, the viewport, the matrix, the sound's catch-up and the picture — are functions of this file's
//! handed over through `Image::hands_over_the_frames_own_calls`.
//!
//! **And it reads its keyboard the way the real one does**: a `GetKeyboardState` it looked up, which is orb's
//! [`orb_core::runtime::keyboard_state`], translated through the game's own table into the 256 keys it plays
//! a run off, and a pad through the `XInputGetState` orb answers. So the word a frame is played with here is
//! the word orb wrote into that snapshot and that pad — and nothing else, which is the whole of how a recipe
//! drives this game.
//!
//! **And it presents through a swapchain it keeps**, which is what orb draws its overlay over: the object
//! is among the renderer's own, `finishes_the_picture` calls its `Present` as the real composite does, and
//! orb's stand-in in that slot is where a canvas of orb's is drawn — see `Image::keeps_a_swapchain` and
//! `the_mode_question_over_th06c.rs`. A question of orb's is answered by a recipe or by a key, both being
//! driven here.
//!
//! Where a number here is th06c's it says so. Where it is this game's own — how far the player moves, when
//! its midboss arrives — it says that too.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_void};
use std::path::PathBuf;

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th06c::Th06c;
use orb_core::game::th06c::image::{
    Image, RESULT_GRID, RESULT_GRID_ACROSS, RESULT_LEAVING, RESULT_NAME_BYTES,
    RESULT_PLACES_RANKED, RESULT_QUESTION_COMES_UP_AFTER, RESULT_RANKING,
    RESULT_RANKING_COMES_UP_AFTER, RESULT_RANKING_COMING_UP, RESULT_RANKING_GOES_AFTER,
    RESULT_RANKING_GOING, RESULT_RANKING_ITEMS, RESULT_RANKING_THE_WAY_OUT,
    RESULT_SAVE_REPLAY_ASKED, RESULT_SAVE_REPLAY_QUESTION, RESULT_STATS, RESULT_STATS_FADE_FOR,
    RESULT_STATS_FADING, RESULT_STATS_TAKE_A_PRESS_AFTER, RESULT_THE_END_ITEM,
    RESULT_THE_ENTRY_TAKES_A_KEY_AFTER, RESULT_TYPES_A_SPACE, RESULT_TYPING_A_NAME,
    SCREEN_CHARACTER, SCREEN_DIFFICULTY, SCREEN_LOGO, SCREEN_ON_ITS_OWN, SCREEN_RANKING,
    SCREEN_SHOT_SELECT, SCREEN_TITLE, chain_job, chain_result,
};
use orb_core::game::th06c::score_file::{self, CLEARED, Record};

use super::{DRAW, Display, FLUSH, Launch, Launched, PRESENT, SOUND, UPDATE, Work, scratch};

/// The exe this game is running as, and the `Game` orb is attached to found out of the table by it.
pub const EXE: &str = "th06c.exe";

/// **The walk from the logo to a run, as a recipe**, which is the real recipes' own — see
/// `orb-real/recipes/th06c/a_run_left_in_its_second_chapter.yaml` for why each step is shaped so.
///
/// **Here rather than in one of the e2e modules that plays it**, because it is this game's screens and every
/// module over a run of it starts by walking them: `a_pointdevice_run_of_th06c` and
/// `the_run_of_th06c_left_behind` would otherwise carry the same twenty lines and a screen number changed in
/// one of them would leave the other walking a game that no longer exists. What a module writes out for
/// itself is what it *asserts* — see `the_run_of_th06c_left_behind`'s own `SLOT`.
pub(crate) const TO_A_RUN: &str = "\
- wait_until: { screen: 1 }
  give_up_after: 7200
- press: [decide]
  frames: 6
- wait_until: { screen: 2 }
  give_up_after: 36000
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 7 }
  give_up_after: 1800
- press: [down]
  frames: 6
  until: { screen: 7, cursor: 1 }
  give_up_after: 900
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
- wait_until: { screen: 9 }
  give_up_after: 1800
- press: [up]
  frames: 6
  until: { screen: 9, cursor: 0 }
  give_up_after: 900
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
- wait_until: { screen: 11 }
  give_up_after: 1800
- wait_until: { frames: 120 }
";

/// **The bits of the word this game's own read builds out of its 256 keys**, by the DirectInput code of the
/// key — read off `0x14000fc24` onwards, see `th06c/asm.md`: `Z` carries the shot with the two shortcut bits
/// beside it, `X` the bomb with one, `Escape` two shortcut bits and no button of a run's, `Enter` the decide's
/// own bit and the shot's, and the four arrows their four.
///
/// **`Enter`'s shot bit is measured and not read**: a real recording of the Extra pressed orb's decide alone
/// across the stage's first four frames — `chose=0x1000`, which [`orb_core::game::th06c`] presses as `Enter`
/// and nothing else — and the word the run acted on was `input=0x0101` on every one of them, with the shot's
/// cycle stepping from the first.
const DIK_BITS: [(usize, u16); 9] = [
    (0x2c, SHOT | ENTER | 0x8000), // Z
    (0x2d, BOMB | 0x200),          // X
    (0x2a, FOCUS),                 // Left Shift
    (0x01, ESCAPE),                // Escape
    (0x1c, SHOT | ENTER),          // Enter
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
const ENTER: u16 = 0x0100;
/// The two bits `Escape` sets, which this game's own pause reads the second of — this laid-out game's own
/// reading, the real one's pause bit being unread.
const ESCAPE: u16 = 0x0600;
const PAUSE: u16 = 0x0400;
/// What the front end reads as its decide and its cancel: the shot or the enter, and the `0x200` the bomb's
/// keys set beside it (`and $0x200` at 0x14002f98e).
const MENU_DECIDE: u16 = SHOT | ENTER;
const MENU_CANCEL: u16 = 0x200;
/// What the logo takes as a press, `and $0x1d02` in its case of 0x14002ccf0, and the updates it takes none
/// for, `0x1e`.
const LOGO_PRESSES: u16 = 0x1d02;
const THE_LOGO_TAKES_A_PRESS_AFTER: i32 = 0x1e;
/// **The read's repeat** — see [`Fake::reads_the_keys_and_the_pad`]: past `0x1d` unchanged reads, every
/// eighth, the count put back to `0x1e` past `0x25`.
const REPEATS_PAST: u32 = 0x1d;
const REPEATS_EVERY: u32 = 8;
const REPEATS_BACK_PAST: u32 = 0x25;
const REPEATS_BACK_TO: u32 = 0x1e;

/// **The title's items**, nine — `0x1400364b0(menu, 9)` and the jump at `0x14002d657` over 0 to 8, the last
/// the one a cancel puts the cursor on (`movl $0x8` at `0x14002f9a5`) — of which `0x14002d592` steps the
/// cursor over the second where no clear unlocked the Extra.
const TITLE_ITEMS: i32 = 9;
const ITEM_START: i32 = 0;
const ITEM_EXTRA: i32 = 1;
const ITEM_PRACTICE: i32 = 2;
const ITEM_REPLAY: i32 = 3;
const ITEM_SCORE: i32 = 4;
const ITEM_QUIT: i32 = 8;
/// How many items the three selects and the practice stage select list: the four difficulties a whole run
/// offers, the two characters, their two shots, and the six stages.
const DIFFICULTIES_OFFERED: i32 = 4;
const CHARACTERS: i32 = 2;
const SHOTS: i32 = 2;
const STAGES_TO_PRACTISE: i32 = 6;
/// The screen the practice stage select is, read off a launch's own `front end:` line — `screen 19` on the
/// frame after the shot select's decide of a run started from Practice Start.
const SCREEN_PRACTICE_STAGE: i32 = 19;

/// How many frames the title ignores a decide for on arrival, and how many the selects do: `0x14` at
/// `0x14002d614` and `0x1e` at `0x1400354d8` — see `Th06c::menu_takes_a_press`.
const TITLE_TAKES_A_PRESS_AFTER: i32 = 0x14;
const A_SELECT_TAKES_A_PRESS_AFTER: i32 = 0x1e;
/// How long the screen the title's first items open goes by on its own before the difficulty select: the real
/// one's `front end: screen 6` line arrives and its `screen 7` line follows sixty frames later.
const ON_ITS_OWN_LASTS: i32 = 60;
/// How long the Score item's screen waits before it asks the supervisor for the ranking: `0x3c` at
/// `0x1400362c4`.
const RANKING_ASKS_AFTER: i32 = 0x3c;
/// How many frames a launch spends before its logo is up, which is the archive read and the window made.
const LOGO_COMES_UP_AFTER: u32 = 60;
/// And how long the logo stands before it goes to the title on its own, as the real one's does after a run —
/// a launch of it left the result screen for `screen 1` and reached `screen 2` with nothing pressed. This
/// laid-out game's own number; the real one's idle is `front end + 0x9454`, uncounted.
const LOGO_LEAVES_ON_ITS_OWN_AFTER: i32 = 120;

/// **The script of a stage this game plays**: the frames its midboss arrives on, puts its card up on, moves
/// on to its next attack on and goes down on. The same shape th06's and th07's laid-out stages have, since
/// what a recipe's `play_until: { stage: 1, chapter: 2 }` needs is a second chapter arriving on its own.
pub const MIDBOSS_ARRIVES: u32 = 400;
pub const CARD_STARTS: u32 = 500;
pub const ATTACK_CHANGES: u32 = 700;
pub const MIDBOSS_BEATEN: u32 = 900;
/// The card that midboss puts up, by its own record, and the name the game puts on the plate for it.
pub const SCRIPTED_CARD: u32 = 1;
const SCRIPTED_CARD_NAME: &str = "月符「ムーンライトレイ」";
/// What a boss of this game is put up with, in the units the game keeps its life in.
const BOSS_LIFE: i32 = 1500;
/// **And the frame her life is read as nearly out on**, a second before she goes, with the life she goes
/// with beside it — see [`Fake::plays_the_stage`], which has why that is a frame of its own.
const A_BOSS_IS_NEARLY_OUT_AT: u32 = MIDBOSS_BEATEN - 60;
const A_BEATEN_BOSSES_LIFE: i32 = 5;

/// The field such a stage is played in and where its player starts, in the game's own units — th06c's
/// 384x448 field, with the player low in the middle of it — and what one frame of holding a direction moves
/// them by, straight and then diagonally.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
/// **And the box inside it the player is held to**, left, top, width and height: what the stage callback writes
/// at `0x1403ee7f4` (0x140024345 through 0x140024363), so the player moves over x 8 to 376 and y 16 to 432.
const PLAYER_AREA: (f32, f32, f32, f32) = (8.0, 16.0, 368.0, 416.0);
const PLAYER_STARTS_AT: (f32, f32) = (192.0, 384.0);
const A_STEP: f32 = 4.0;
/// Half of the box the game's own hit test gives the player, on each axis.
const PLAYER_HALF: f32 = 4.0;

/// **And what that stage shoots**: a bullet aimed at where the player is, from the top of the field, every so
/// many frames from the first — slow enough that a player who steps aside is missed and one who does not is
/// hit.
const FIRST_BULLET_AT: u32 = 60;
const A_BULLET_EVERY: u32 = 120;
const BULLETS_FROM: (f32, f32) = (192.0, 0.0);
const BULLET_SPEED: f32 = 3.0;
const BULLET_SIZE: f32 = 8.0;
const BULLET_SLOTS_USED: usize = 16;

/// How many updates a hit stands before it is a death — the frames a bomb is still taken in, this game's
/// own `+0x7254` counted down at `0x14004a7ae` — the count a walk-in ends at, how long the invulnerability
/// after one lasts, and how long a bomb goes off for. The game's own numbers but the last: the 6 the walk-in
/// leaves on `+0x7254` as it hands over (0x14004abb4, the register's 8 having been put to nought at
/// 0x14004ab32), `0x14004ab7a`'s `cmp r8d,0x1e` and the `0xf0` `0x14004ab96` writes.
const A_BOMB_IS_TAKEN_FOR: i32 = 6;
const RESPAWN_TAKES: i32 = 0x1e;
const INVULNERABLE_FOR: i32 = 0xf0;
pub const A_BOMB_LASTS: u32 = 60;
/// The count a bomb of the first shot type leaves the player untouchable for, `movl $0x168` at 0x140009685.
const A_BOMB_KEEPS_THE_PLAYER_UNTOUCHABLE_FOR: i32 = 0x168;

/// **And what the player's own register leaves on that count at a stage's build**, `0x14004a14c`'s 120,
/// which the walk-in reads against its thirty (`0x14004aaeb`) and is already past — the state beside it is
/// what matters, and it is the walk-in's. See `the_player_a_stage_of_th06c_starts`.
const THE_REGISTER_LEAVES_UNTOUCHABLE: i32 = 120;

/// **How many updates a firing cycle lasts**: `0x14004b8be` holds the phase against thirty and `0x14004b8d3`
/// puts the pair back to [`NOT_FIRING`], so the update after the thirtieth starts another.
const A_FIRING_CYCLE_LASTS: i32 = 30;
/// **The pair a player firing nothing holds**, phase and phase before — the qword `0x14004b8d3` writes and
/// the one the player's own register writes at `0x14004a3ca`.
const NOT_FIRING: (i32, i32) = (-1, -999);
/// **And the phase a press starts at**, `0x14004b35c`. The `-999` `0x14004b362` writes beside it is not here
/// because nothing outside that update reads it: the advance runs in the same one.
const THE_FIRST_PHASE: i32 = 0;

/// One bullet of that stage's own: which slot it is written down in, where it is and what it moves by a frame.
#[derive(Clone, Copy)]
struct Bullet {
    slot: usize,
    at: (f32, f32),
    step: (f32, f32),
}

/// The stage word the shot select writes for the Extra, which the stage callback raises to seven.
const THE_EXTRAS_STAGE_WORD: i32 = 6;
const DIFFICULTY_EASY: i32 = 0;
/// The count of extends an Extra run starts with as paid for, `[gm+0x17f6]` at 0x1400242d0's run start.
const THE_EXTRAS_EXTENDS_PAID_FOR: i8 = 4;
const DIFFICULTY_EXTRA: i32 = 4;
/// The stage word an Easy run ends on, the fifth stage's: `cmp $0x5` against `[0x1403ed030]` in 0x140028410's
/// case 0xb, which every other difficulty goes on past.
const THE_LAST_STAGE_WORD_OF_EASY: i32 = 5;
/// The rank a Normal run starts at, which is what a launch of the real game read as its first stage began:
/// `rank: 16` in `resume: stage 1 begins with`.
const RANK_AT_A_RUNS_START: i32 = 16;
/// What a run starts with out of the configuration, `02 03` at `0x140059935`, and what the Extra or a
/// practice run starts with instead, `0x302` at `0x14002cc30`.
const CONFIG_LIVES: u8 = 2;
const CONFIG_BOMBS: u8 = 3;
const EXTRA_LIVES: i8 = 2;
const EXTRA_BOMBS: i8 = 3;

/// How many updates the ending's own script runs for where a run is played to its clear, and the staff roll
/// after it. This laid-out game's own numbers: what a case that has played six stages needs of them is that
/// they end.
const ENDING_UPDATES: i32 = 10;
const ROLL_UPDATES: i32 = 10;

/// The score file th06c asks for, which is the one name this game passes `fopen` for its records — and so
/// the one its `CreateFileW` is handed, widened, the CRT being the exe's own.
const SCORE_FILE: &CStr = c"score.dat";
/// And the name orb's fork sends it to while the mode is pointdevice — relative, as the game's own is.
const THE_MODES_FILE: &str = "pointdevice_score.dat";
/// **What `fopen`'s two modes come to at `CreateFileW`**: `rb` asks to read a file that is there, `wb` to
/// write one made afresh — Win32's own numbers, which are the game's arguments here.
const READ: (u32, u32) = (0x8000_0000, 3);
const WRITE: (u32, u32) = (0x4000_0000, 2);

/// What a whole frame answers, which the game's own loop reads: nought to be called again.
pub const FRAME_KEPT_RUNNING: i32 = 0;

/// The run every e2e test over a chapter of this game plays, and what a stage of it starts with: Normal from
/// the first stage, with Reimu's first shot.
pub const DIFFICULTY: i32 = 1;
pub const CHARACTER: u8 = 0;
pub const SHOT_TYPE: u8 = 0;

/// A th06c laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// Whether this game's chain walk says the game is leaving — see [`Fake::leaves`].
    leaving: Cell<bool>,
    /// How many frames this launch has run, for the logo that comes up on its own.
    frames_since_launch: Cell<u32>,
    /// How many frames the screen the title's items open has been going by on its own.
    on_its_own_for: Cell<i32>,
    /// Which item the title was left on, which is the item the front end comes back on.
    left_the_title_on: Cell<i32>,
    /// Whether the run being chosen practises one stage, and whether it is the Extra.
    practising: Cell<bool>,
    extra: Cell<bool>,
    /// **The read's repeat**, 0x140058330's: how many reads the word has stood unchanged for
    /// (`[0x1409553ac]`), and whether this read is one the held word counts as pressed again
    /// (`[0x140955398]`).
    unchanged_for: Cell<u32>,
    repeated: Cell<bool>,
    /// How long a stage lasts before the next is built. `None` is a stage that never ends.
    stage_frames: Cell<Option<u32>>,
    /// The stage frame the script last acted on, the bullets the stage has out, the frames a hit has left and
    /// the frames a bomb has left to go off for.
    last_stage_frame: Cell<u32>,
    bullets: RefCell<Vec<Bullet>>,
    hit_window: Cell<i32>,
    bomb_left: Cell<u32>,
    /// Whether the stage fires at the player — see [`Fake::fires_at_the_player`].
    fires: Cell<bool>,
    /// How many of the stage's own enemies are alive beside whatever the fight has — see
    /// [`Fake::the_stage_has_enemies`].
    enemies: Cell<i32>,
    /// Whether an e2e test said the player is hit on the next update.
    hit: Cell<bool>,
    /// The updates the ending's script and the roll after it have left, where one is running.
    ending_updates: Cell<i32>,
    roll_updates: Cell<i32>,
    /// **The shot — `character * 2 + shot` — and the difficulty of a run that reached the ending**, which the
    /// file written after it holds as a clear: the stage callback raises both of the shot's `CLRD` arrays to the
    /// stage just cleared, the first only for a run with no retry (`0x140024d3c` through `0x140024da9`), and the
    /// last stage's is [`CLEARED`].
    cleared: Cell<Option<(u8, usize)>>,
    /// Every open of the score file, by the name it landed in and whether it was for writing — see
    /// [`create_file_w`].
    opens: RefCell<Vec<(String, bool)>>,
    /// Every viewport the frame set and every seek the sound's catch-up made, for a test about the frame.
    viewports: RefCell<Vec<[u32; 4]>>,
    seeks: RefCell<Vec<(i64, i32)>>,
    /// How many times the frame finished its picture — see [`finishes_the_picture`].
    pictures: Cell<u32>,
    /// And how many references to the swapchain have been given back, which is what says orb's ask about
    /// what that object is did not hold one — see [`swapchain_release`].
    swapchain_releases: Cell<u32>,
    /// Whether the game's own retry menu — the continue question a run out of lives is asked — is up.
    out_of_lives: Cell<bool>,
    /// The state the screen a run ends at was last seen in, and how many of its updates it has been in it:
    /// its stats and its fades are timed, as the real one's are — see `Image::RESULT_STATS`.
    result_state_seen: Cell<i32>,
    result_state_frames: Cell<i32>,
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
    /// The object the frame is called on, which for this game is the window's own and reaches nothing here:
    /// the supervisor's address, which is inside the space.
    fn frame_window(&self) -> *mut c_void {
        std::ptr::without_provenance_mut(orb_core::game::th06c::image::SUPERVISOR_OBJECT)
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
    /// Lays the game out on a display an e2e test says the whole of and attaches orb to it the way a launch
    /// is attached: **orb's own frame loop**, which for this game is the only frame there is.
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

    /// And on a machine whose own windows are in the language `ui_language` names — `orb_sim::langid` — with
    /// nothing in `orb.yaml` about it, which is where orb's screens take their language from when the file
    /// says none. See `the_screens_of_th06c_in_english`.
    pub fn attach_on_a_machine_in(
        ui_language: u16,
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, Some(ui_language), name, &[], work, settings)
    }

    /// And with files a launch before left under orb's directory, as paths inside it and their bytes — the
    /// chapter file and the replay a run leaves being bytes and not text.
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
        // The ten calls orb's frame makes into this game.
        image.hands_over_the_frames_own_calls(
            run_calc_chain as *const () as usize,
            takes_an_element_out as *const () as usize,
            sets_the_frames_view as *const () as usize,
            flushes_the_sprites as *const () as usize,
            the_sounds_rate as *const () as usize,
            where_the_music_is as *const () as usize,
            how_long_the_track_is as *const () as usize,
            seeks_the_music as *const () as usize,
            puts_the_rate_together as *const () as usize,
            finishes_the_picture as *const () as usize,
        );
        // **The keyboard read this game looked up, which is orb's, with the host's behind it** — where a
        // real launch's `GetProcAddress` answer puts it. The host's is the simulated keyboard.
        orb_core::runtime::GET_KEYBOARD_STATE.store(
            host_keyboard_state as *const () as usize,
            std::sync::atomic::Ordering::Relaxed,
        );
        // And the file open this game's records go through, which orb's fork stands in front of —
        // `CreateFileW`, the CRT's `fopen` being the exe's own and ending there; see
        // `Hooks::opens_its_files`. With it the heap the copy of that file is taken off, which orb's own
        // copy goes through too.
        unsafe { orb_core::score::install_over_create_file_w(create_file_w) };
        image.hands_over_its_heap(malloc as *const () as usize, free as *const () as usize);
        image.hands_over_the_run_start(the_games_run_start as *const () as usize);
        image.hands_over_the_capture(the_games_capture as *const () as usize);
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
        // Where the game starts: the supervisor's job registered and nothing else — the state a fresh process
        // reads as `scene 0` — with the draw chain's one element and the configuration's lives and bombs.
        image.the_configuration_gives(CONFIG_LIVES, CONFIG_BOMBS);
        image.registers_the_supervisor();
        image.registers_in_the_draw_chain(draw_job as *const () as usize, 0);
        // **The swapchain this game presents through, as the real one has one from its first frame.** It is
        // what orb's own frame hands a frame over through and what orb's overlay is drawn on — a laid-out
        // game without one is a launch with no overlay at all, and every e2e test over what orb puts on
        // this game's screen would be asserting about a drawer that never came up.
        image.keeps_a_swapchain(
            swapchain_query_interface as *const () as usize,
            swapchain_release as *const () as usize,
            swapchain_describes_itself as *const () as usize,
            swapchain_presents as *const () as usize,
            swapchain_presents1 as *const () as usize,
        );

        let fake = Box::new(Self {
            image,
            launch,
            leaving: Cell::new(false),
            frames_since_launch: Cell::new(0),
            on_its_own_for: Cell::new(0),
            left_the_title_on: Cell::new(ITEM_START),
            practising: Cell::new(false),
            extra: Cell::new(false),
            unchanged_for: Cell::new(0),
            repeated: Cell::new(false),
            stage_frames: Cell::new(None),
            last_stage_frame: Cell::new(0),
            bullets: RefCell::new(Vec::new()),
            hit_window: Cell::new(0),
            bomb_left: Cell::new(0),
            fires: Cell::new(true),
            enemies: Cell::new(0),
            hit: Cell::new(false),
            ending_updates: Cell::new(0),
            roll_updates: Cell::new(0),
            cleared: Cell::new(None),
            opens: RefCell::new(Vec::new()),
            viewports: RefCell::new(Vec::new()),
            seeks: RefCell::new(Vec::new()),
            pictures: Cell::new(0),
            swapchain_releases: Cell::new(0),
            out_of_lives: Cell::new(false),
            result_state_seen: Cell::new(-1),
            result_state_frames: Cell::new(0),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        let walks = the_game_this_is()
            .hooks()
            .walks
            .expect("th06c hands its two walks over");
        unsafe {
            orb_core::runtime::attach_to(
                the_game_this_is(),
                config,
                fake.image.data(),
                orb_core::runtime::Originals {
                    // **The game's own two walks, which are orb's**: nothing of this game's calls either, the
                    // frame that did being orb's — see `Hooks::walks`.
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
        if let Some(recipe) = &recipe {
            unsafe { orb_bot::install(recipe, the_game_this_is()) };
        } else {
            // **And a launch with no recipe takes the last one's bot out**, which a real launch never has
            // to: `orb_bot`'s own static and the hook it sets are the process's, and an e2e test plays two
            // launches in one — measured, a launch configured with no recipe at all said
            // `autoplay: step 21 of 21 closes the game` on every frame of a walk a keyboard was making, the
            // recipe being the one the launch before it had left there. See
            // `the_run_of_th06c_left_behind`'s cancel, which is walked by hand for want of a step that
            // presses the key answering no item.
            unsafe { orb_core::runtime::plays_the_run(None) };
        }
        // The game's own file is there, holding a clear, as an installation somebody has played is; orb's own
        // is not until a ranking has written it. And the one read of the score file this game makes, at its
        // startup — `0x14002c239` — lands in whichever orb's fork says, and the copy every screen parses is
        // what that read found.
        fake.puts_the_file(
            &SCORE_FILE.to_string_lossy(),
            &score_file::encode(&records_holding(true), 0x11),
        );
        fake.reads_the_score_file_at_startup();
        fake.frame_takes(work);
        fake
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th06c.read_state() }
    }

    /// Whether the stage fires at the player, which it does unless a test says otherwise.
    pub fn fires_at_the_player(&self, fires: bool) {
        self.fires.set(fires);
    }

    /// **How many of the stage's own enemies are alive**, beside whatever the fight has.
    ///
    /// A wave is that count going up and a gap is it going back to nothing, which is the whole of what the
    /// chapter detector reads of a stage's script — see `a_chapter_table_collected_for_th06c`. Driven from a
    /// test rather than scripted here for the reason th07's fake has it so: what this game lays out of a
    /// stage is its fight, and a wave is one number.
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

    /// The game's own records holding no clear, which takes `Extra Start` off the title.
    pub fn its_records_hold_no_clear(&self) {
        self.its_records_are(&records_holding(false));
    }

    /// The game's own file holding these records — as the installation was before the launch, so the copy
    /// the startup read made of it is made again where that read landed there.
    pub fn its_records_are(&self, records: &[Record]) {
        let name = SCORE_FILE.to_string_lossy().into_owned();
        self.puts_the_file(&name, &score_file::encode(records, 0x11));
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
        self.puts_the_file(THE_MODES_FILE, &score_file::encode(records, 0x22));
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

    /// Every viewport orb's frame set, every seek it asked the music for, and how many pictures it finished.
    pub fn viewports_set(&self) -> Vec<[u32; 4]> {
        self.viewports.borrow().clone()
    }

    pub fn seeks_asked_for(&self) -> Vec<(i64, i32)> {
        self.seeks.borrow().clone()
    }

    pub fn pictures_finished(&self) -> u32 {
        self.pictures.get()
    }

    /// How many references to this game's swapchain orb has given back — see [`swapchain_release`].
    pub fn swapchain_references_given_back(&self) -> u32 {
        self.swapchain_releases.get()
    }

    /// **Whether the title menu is up, on the item that starts a run, and past the frames it ignores a
    /// press for** — which is the one screen orb has a question about and the one state a press at it is
    /// acted on in. See [`TITLE_TAKES_A_PRESS_AFTER`], the number the game's own decide check reads.
    pub fn the_title_is_up_and_takes_a_press(&self) -> bool {
        self.image.the_screen_the_front_end_is_on() == SCREEN_TITLE
            && self.image.the_item_the_cursor_is_on() == ITEM_START
            && self.image.frames_the_screen_has_been_up() >= TITLE_TAKES_A_PRESS_AFTER
    }

    /// **The title put up on the item that starts a run, past its grace** — the state an e2e test about
    /// orb's question over that screen wants, said rather than walked to.
    ///
    /// **Said, because walking there twice in one launch is a walk this fake does not owe.** What the
    /// question is asked over is the title with the cursor on Start, and a test that asks twice would
    /// otherwise have to play a whole run out and wait for the front end to be rebuilt to get back to it —
    /// which is `a_pointdevice_run_of_th06c.rs`'s subject and not this one's. th07's own question test says
    /// its title the same way, through `Image::the_front_end_is_up`.
    /// **Cut before it is registered**, which is the order the supervisor's own rebuild has and the reason
    /// this is not two lines: `registers_the_front_end` puts an element into the chain and one put in twice
    /// links to itself, which is a walk of that chain that never ends — `runs_the_calc_chain` asserted its
    /// way out of one at 65 jobs.
    pub fn puts_its_title_up(&self) {
        self.image.cuts_the_front_end();
        self.image.registers_the_front_end();
        self.image.the_front_end_is_on(SCREEN_TITLE, ITEM_START);
        self.image
            .the_screen_has_been_up_for(TITLE_TAKES_A_PRESS_AFTER * 3);
    }

    // ---- the calc chain ----

    /// `0x1400256b0`: every job of the calc list in turn, and what the walk does with each answer — answering
    /// nought to carry the frame on and one to leave it, which is what the real walk answers.
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
            chain_job::RESULT_SCREEN => self.result(),
            chain_job::ENDING => self.ending(),
            other => {
                panic!("a job of this chain has {other:#x} as its callback, which is no job of it")
            }
        }
    }

    /// The supervisor's own update at chain priority 0: the read of the frame's keys and pad into the word the
    /// run acts on, the scene that has been asked for built, and the copy that makes the next one a change
    /// nobody has acted on.
    fn supervisor(&self) -> i32 {
        let frames = self.frames_since_launch.get() + 1;
        self.frames_since_launch.set(frames);
        if frames == LOGO_COMES_UP_AFTER && self.image.the_scene_now() == 0 {
            self.image
                .is_in_scene(self.image.scene_of_the_front_end(), 0);
        }
        self.reads_the_keys_and_the_pad();
        let now = self.image.the_scene_now();
        let last = self.image.the_scene_last_acted_on();
        if now != last {
            self.builds(now, last);
        }
        self.image
            .is_in_scene(self.image.the_scene_now(), self.image.the_scene_now());
        chain_result::CONTINUES
    }

    /// `0x14000fa80`: the keyboard read this game looked up, translated through its table into the 256 keys
    /// it plays off, the word built out of them, and the pad's bits added.
    fn reads_the_keys_and_the_pad(&self) {
        let mut snapshot = [0u8; 256];
        unsafe { orb_core::runtime::keyboard_state(snapshot.as_mut_ptr()) };
        let mut keys = [0u8; 256];
        for (key, byte) in snapshot.iter().enumerate() {
            if let Some(index) = self.image.the_index_of_the_key(key as u8) {
                keys[index] = *byte;
            }
        }
        self.image.the_game_reads_the_keys(&keys);
        let mut word = DIK_BITS
            .iter()
            .filter(|(index, _)| keys[*index] & 0x80 != 0)
            .fold(0u16, |word, (_, bits)| word | bits);
        let mut pad = [0u8; 16];
        // **`0x14000f710`, the pad's half**, through `th06.cfg`'s own bindings: see
        // `super::the_word_of_a_bound_pad`.
        if unsafe { orb_core::runtime::xinput_get_state(0, pad.as_mut_ptr()) } == 0 {
            let (bound, swapped) = self.image.the_pad_bindings();
            word |= super::the_word_of_a_bound_pad(
                u16::from_le_bytes([pad[4], pad[5]]),
                bound,
                swapped,
            );
        }
        // **The repeat**, 0x140058330 beside the read: a word that stood for more than 0x1d reads is pressed again
        // on every one whose count is a multiple of eight, the count going back to 0x1e past 0x25 — so the
        // thirty-third read of a held key and every eighth after it.
        let unchanged = self.image.the_word_the_run_acts_on() as u16 == word;
        let (count, repeated) = match unchanged {
            false => (0, false),
            true => {
                let mut count = self.unchanged_for.get();
                let repeated = count > REPEATS_PAST && count.is_multiple_of(REPEATS_EVERY);
                if count > REPEATS_BACK_PAST {
                    count = REPEATS_BACK_TO;
                }
                (count + 1, repeated)
            }
        };
        self.unchanged_for.set(count);
        self.repeated.set(repeated);
        self.image.the_run_acts_on_the_word(u32::from(word));
    }

    fn pressed(&self, mask: u16) -> bool {
        let word = self.image.the_word_the_run_acts_on() as u16;
        let before = self.image.the_word_before() as u16;
        word & mask != 0 && before & mask == 0
    }

    /// Pressed, or held on a read the repeat counts as a press — what the cursors and the logo take.
    fn pressed_or_repeated(&self, mask: u16) -> bool {
        let word = self.image.the_word_the_run_acts_on() as u16;
        self.pressed(mask) || (word & mask != 0 && self.repeated.get())
    }

    /// What the supervisor does with a state that has been asked for: takes down what was running and builds
    /// the one wanted.
    fn builds(&self, now: i32, last: i32) {
        let front_end = self.image.scene_of_the_front_end();
        let run = self.image.scene_of_a_run();
        let rebuilding = self.image.scene_of_a_stage_being_rebuilt();
        let ranking = self.image.scene_of_the_ranking();
        let run_over = self.image.scene_of_a_run_over();
        if now == self.image.scene_of_a_replay_over() {
            // The title, as 0x140058330's move from 2 to 8 builds it, and the supervisor's state put to it.
            self.image.is_in_scene(front_end, now);
            self.builds(front_end, run);
        } else if now == front_end {
            // A run left for the title — the pause menu's quit, the demo cut short, a replay played out — puts
            // the playback flag down (0x140058330's moves from 2), where one from any other state leaves it.
            if last == run {
                self.image.the_playback_flag_is_down();
            }
            self.tears_the_stage_down();
            self.image.the_screen_a_run_ends_at_is_down();
            self.image.the_ending_is_over();
            self.image.the_replay_manager_is_gone();
            self.image.registers_the_front_end();
            self.image
                .the_front_end_is_on(SCREEN_LOGO, self.left_the_title_on.get());
            self.image.is_not_paused();
            self.out_of_lives.set(false);
        } else if now == run && last == front_end {
            self.image.cuts_the_front_end();
            self.image.the_replay_manager_is_up();
            self.registers_the_stage();
        } else if now == rebuilding {
            // The stage's own jobs cut and the manager registered again, which is what a stage move is: the
            // callback reads the state as three to know it from a run's start, and the supervisor puts two back.
            self.tears_the_stage_down();
            self.registers_the_stage();
            self.image.is_in_scene(run, run);
        } else if now == ranking {
            self.image.cuts_the_front_end();
            self.image
                .the_screen_a_run_ends_at_is_up(RESULT_RANKING_COMING_UP);
            self.image.the_characters_typed_are(0);
        } else if now == run_over {
            self.tears_the_stage_down();
            self.image.the_ending_is_over();
            self.image
                .the_screen_a_run_ends_at_is_up(RESULT_TYPING_A_NAME);
        }
    }

    /// **A stage of this game's own choosing built**, counted from zero the way everything above `Game`
    /// counts, for an e2e test about a stage coming up that is not the one a resume asked for.
    ///
    /// The word first and the callback second, which is the order the game has it in: the stage callback is
    /// what orb's hook stands over and it reads the stage out of the manager, so a word written after it would
    /// be a stage orb never saw.
    pub fn builds_the_stage(&self, stage: i32) {
        self.image.the_games_stage_word_is(stage);
        self.registers_the_stage();
    }

    /// **A run of this game in progress in one of its stages**, said rather than walked to: the supervisor in
    /// the run's own scene and that stage built.
    ///
    /// **Said, because the walk to it is a recipe's** — `a_pointdevice_run_of_th06c.rs` is what that walk is
    /// the subject of — and an e2e test about what a stage does with a frame owes no walk through the menus.
    ///
    /// **And the scene before the stage**, which is not an ordering to swap: the supervisor builds whatever
    /// scene it has been asked for, so a stage registered while the launch is still on its logo's scene is a
    /// stage the front end's own build tears down sixty frames later.
    ///
    /// **With the lives the configuration gives**, which is the game's own run start and not a detail of the
    /// scene: a stage entered with none of them is a stage whose first hit is the run's last, and the run is
    /// over before a test that walked into it has asked anything.
    pub fn in_a_run_in_the_stage(&self, stage: i32) {
        let run = self.image.scene_of_a_run();
        self.the_games_own_run_start();
        self.image.is_in_scene(run, run);
        self.builds_the_stage(stage);
    }

    /// **And that same stage with the game's own playback in charge of it**, which is what a judging pass
    /// watches: the byte the replay list's own decide sets, over a stage being played through.
    ///
    /// **Set after the run's start and not before**, which is the order the game has it in: that start
    /// clears the byte (`0x14002cc77`, the same function that writes `0x302` over the lives and the bombs),
    /// so a flag set first is one the run start takes away again.
    pub fn watches_a_replay_of_the_stage(&self, stage: i32) {
        self.in_a_run_in_the_stage(stage);
        self.image.a_replay_is_playing(false);
    }

    /// **A stage asked for that never arrives**: the supervisor already in the run's own scene, so nothing is
    /// built and no stage callback is entered. What orb must not do is hold a run's numbers for ever waiting
    /// for a moment that is not coming.
    pub fn the_stage_never_comes_up(&self) {
        let run = self.image.scene_of_a_run();
        self.image.is_in_scene(run, run);
    }

    /// The game manager's element registered, whose init callback is the stage callback orb's hook stands over
    /// — entered with the manager, the way the chain enters it.
    fn registers_the_stage(&self) {
        self.image.registers_the_gameplay_scene();
        // **And the shot's phase back to idle**, which is what the player's own register leaves at
        // `0x14004a3ca`: a stage carries none of its shots in. See [`Fake::fires_a_shot`].
        self.image.the_shot_phase_is(NOT_FIRING.0, NOT_FIRING.1);
        let manager =
            std::ptr::without_provenance_mut(orb_core::game::th06c::image::GAME_MANAGER_OBJECT);
        orb_core::runtime::stage_begun_in_ecx(manager);
    }

    fn tears_the_stage_down(&self) {
        self.image.cuts_the_gameplay_scene();
        self.bullets.borrow_mut().clear();
        self.writes_the_bullets_down();
        self.image.no_boss_is_fighting();
        self.image.no_spellcard_is_running();
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
        let role = self.role_of(screen);
        let items = match role {
            SCREEN_TITLE => TITLE_ITEMS,
            SCREEN_DIFFICULTY => DIFFICULTIES_OFFERED,
            SCREEN_CHARACTER => CHARACTERS,
            SCREEN_SHOT_SELECT => SHOTS,
            SCREEN_PRACTICE_STAGE => STAGES_TO_PRACTISE,
            _ => 0,
        };
        // The cursor, round, up before down and on the repeat as well as the press (0x1400364b0): the title
        // steps over the Extra where the records hold no clear.
        if items > 0 {
            let step = match (self.pressed_or_repeated(UP), self.pressed_or_repeated(DOWN)) {
                (true, _) => -1,
                (false, true) => 1,
                (false, false) => 0,
            };
            let mut moved = (cursor + step).rem_euclid(items);
            if screen == SCREEN_TITLE && !self.a_clear_is_in_the_copy() && moved == ITEM_EXTRA {
                moved = (moved + step).rem_euclid(items);
            }
            if moved != cursor {
                self.image.the_cursor_is_on(moved);
            }
        }
        let cursor = self.image.the_item_the_cursor_is_on();
        match role {
            // The count the logo and the title hold a press against is raised at their update's end, so the
            // update reads it one short of `up_for`.
            SCREEN_LOGO
                if (self.pressed_or_repeated(LOGO_PRESSES)
                    && up_for > THE_LOGO_TAKES_A_PRESS_AFTER)
                    || up_for >= LOGO_LEAVES_ON_ITS_OWN_AFTER =>
            {
                self.image.the_front_end_is_on(SCREEN_TITLE, cursor)
            }
            SCREEN_TITLE if decide && up_for > TITLE_TAKES_A_PRESS_AFTER => {
                self.left_the_title_on.set(cursor);
                match cursor {
                    ITEM_START | ITEM_EXTRA | ITEM_PRACTICE => {
                        self.practising.set(cursor == ITEM_PRACTICE);
                        self.extra.set(cursor == ITEM_EXTRA);
                        // The item writes the practice flag itself, as th06 1.02h's does, and the shot select
                        // then writes it clear or the stage select sets it again: what orb reads at the shot
                        // select is a run with no slot, and one peeked at for a run left unfinished would be
                        // a full run's file looked up for a stage practised on its own.
                        if cursor == ITEM_PRACTICE {
                            self.image.is_practising();
                        } else {
                            self.image.is_not_practising();
                        }
                        self.on_its_own_for.set(0);
                        self.image.the_front_end_is_on(SCREEN_ON_ITS_OWN, cursor);
                    }
                    ITEM_SCORE => self.image.the_front_end_is_on(SCREEN_RANKING, cursor),
                    ITEM_QUIT => orb_api::process::exit(0),
                    // The replay list, which orb's own stands over: a screen this game does not lay out.
                    _ => {}
                }
            }
            SCREEN_TITLE if cancel && up_for > TITLE_TAKES_A_PRESS_AFTER => {
                self.image.the_cursor_is_on(ITEM_QUIT)
            }
            SCREEN_ON_ITS_OWN => {
                let on_its_own = self.on_its_own_for.get() + 1;
                self.on_its_own_for.set(on_its_own);
                if on_its_own >= ON_ITS_OWN_LASTS {
                    if self.extra.get() {
                        self.image.the_difficulty_is(DIFFICULTY_EXTRA);
                        self.image.the_front_end_is_on(
                            self.shown_as(SCREEN_CHARACTER),
                            i32::from(CHARACTER),
                        );
                    } else {
                        self.image
                            .the_front_end_is_on(SCREEN_DIFFICULTY, DIFFICULTY);
                    }
                }
            }
            SCREEN_DIFFICULTY if decide && up_for >= A_SELECT_TAKES_A_PRESS_AFTER => {
                self.image.the_difficulty_is(cursor);
                self.image
                    .the_front_end_is_on(SCREEN_CHARACTER, i32::from(CHARACTER));
            }
            SCREEN_DIFFICULTY if cancel => self.image.the_front_end_is_on(SCREEN_TITLE, ITEM_START),
            SCREEN_CHARACTER if decide && up_for >= A_SELECT_TAKES_A_PRESS_AFTER => {
                let character = u8::try_from(cursor).expect("a cursor over the characters");
                self.image.the_shot_select_decided(
                    self.image.the_difficulty_now(),
                    character,
                    SHOT_TYPE,
                );
                self.image
                    .the_front_end_is_on(self.shown_as(SCREEN_SHOT_SELECT), i32::from(SHOT_TYPE));
            }
            SCREEN_CHARACTER if cancel && !self.extra.get() => {
                self.image
                    .the_front_end_is_on(SCREEN_DIFFICULTY, self.image.the_difficulty_now());
            }
            SCREEN_SHOT_SELECT if decide && up_for >= A_SELECT_TAKES_A_PRESS_AFTER => {
                let shot = u8::try_from(cursor).expect("a cursor over the shots");
                self.image.the_shot_select_decided(
                    self.image.the_difficulty_now(),
                    self.image.the_character_chosen(),
                    shot,
                );
                if self.practising.get() {
                    self.image.the_front_end_is_on(SCREEN_PRACTICE_STAGE, 0);
                } else {
                    self.starts_the_run(if self.extra.get() {
                        THE_EXTRAS_STAGE_WORD
                    } else {
                        0
                    });
                }
            }
            SCREEN_SHOT_SELECT if cancel => {
                self.image.the_front_end_is_on(
                    self.shown_as(SCREEN_CHARACTER),
                    i32::from(self.image.the_character_chosen()),
                );
            }
            SCREEN_PRACTICE_STAGE if decide && up_for >= A_SELECT_TAKES_A_PRESS_AFTER => {
                self.image.is_practising();
                self.starts_the_run(cursor);
            }
            SCREEN_RANKING if up_for >= RANKING_ASKS_AFTER => {
                self.image.is_in_scene(
                    self.image.scene_of_the_ranking(),
                    self.image.the_scene_last_acted_on(),
                );
            }
            _ => {}
        }
        chain_result::CONTINUES
    }

    /// `0x14002cc00`: what the item that starts a run writes — the lives and the bombs out of the configuration
    /// or the Extra's own, the playback flags cleared, the stage word, and the supervisor's state, which its
    /// update acts on next.
    /// **Which select a screen number is, which for the Extra is not what it is for a full run**: the Extra
    /// has no difficulty select, and its two selects carry the numbers a full run's first two do — seven is
    /// its character select and nine its shot select, where a full run's are the difficulty and the character.
    /// Read off a launch of the real one walking into the Extra: `screen 6`, `screen 7, cursor on item 0`,
    /// `screen 9`, and `run: stage 7 on difficulty 4` off the decide there.
    fn role_of(&self, screen: i32) -> i32 {
        match (self.extra.get(), screen) {
            (true, SCREEN_DIFFICULTY) => SCREEN_CHARACTER,
            (true, SCREEN_CHARACTER) => SCREEN_SHOT_SELECT,
            (_, screen) => screen,
        }
    }

    /// And the number a select is shown under, the other way round.
    fn shown_as(&self, role: i32) -> i32 {
        match (self.extra.get(), role) {
            (true, SCREEN_CHARACTER) => SCREEN_DIFFICULTY,
            (true, SCREEN_SHOT_SELECT) => SCREEN_CHARACTER,
            (_, role) => role,
        }
    }

    /// What the shot select's and the practice's decide do: the stage word written, and the game's own run
    /// start called — which orb calls too, through [`the_games_run_start`].
    fn starts_the_run(&self, stage_word: i32) {
        self.image.the_games_stage_word_is(stage_word);
        self.the_games_own_run_start();
    }

    /// `0x14002cc00`: the lives and the bombs out of the configuration — or two and three where the
    /// difficulty is the Extra's or the run is practice (0x14002cc1b) — the playback flag cleared, the music
    /// stopped and the track put to `-1` where the configuration plays the wav tracks (0x14002cc70), and the
    /// supervisor's state to the run's.
    fn the_games_own_run_start(&self) {
        if self.image.the_difficulty_now() == DIFFICULTY_EXTRA || self.state().practice {
            self.image.has_lives_and_bombs(EXTRA_LIVES, EXTRA_BOMBS);
        } else {
            let (lives, bombs) = self.image.the_configuration_now();
            self.image.has_lives_and_bombs(lives as i8, bombs as i8);
        }
        self.image.no_replay_is_playing();
        if self.image.the_music_is_played_as_wav() {
            self.image.plays_the_track(-1);
        }
        self.image.is_in_scene(
            self.image.scene_of_a_run(),
            self.image.the_scene_last_acted_on(),
        );
    }

    /// The game manager's own update at chain priority 4: one frame of the stage.
    fn stage(&self) -> i32 {
        let state = self.state();
        // The pause menu put up and taken down off the button's edge, and the run's update held while it is.
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
        // Out of lives: the continue question, which nothing here answers yes to — a decide gives the run up.
        if self.out_of_lives.get() {
            if self.pressed(MENU_DECIDE) {
                self.image.is_not_paused();
                self.image.is_in_scene(
                    self.image.scene_of_a_run_over(),
                    self.image.the_scene_last_acted_on(),
                );
            }
            return chain_result::CONTINUES;
        }
        let frames = self.image.frames_in_the_stage() + 1;
        self.image.has_been_in_the_stage_for(frames);
        self.image
            .the_script_has_run_for(state.script_frames.saturating_add(1));
        self.image.set_replay_clock(frames as i32);
        // One number a frame out of the generator, which is why a stage played again from another seed is
        // another stage.
        self.image.draws_from_the_generator();
        self.plays_the_stage(frames);
        if Some(frames) == self.stage_frames.get() {
            self.the_stage_is_over();
        }
        chain_result::CONTINUES
    }

    /// **The message script's end of a stage**, 0x140028410's case 0xb: the next stage built through the state
    /// the game rebuilds its manager in for a run not in practice below its stage word 5, or 5 on any difficulty
    /// but Easy; and past it the title for a replay, the ending for every difficulty but the Extra's, and the
    /// screen a run ends at for the Extra and practice. The stage word stays the stage's until the callback
    /// raises it, nothing writing it nought.
    fn the_stage_is_over(&self) {
        let word = self.image.the_games_stage_word();
        let easy = self.image.the_difficulty_now() == DIFFICULTY_EASY;
        let last = self.image.the_scene_last_acted_on();
        let wanted = if self.state().practice {
            self.image.scene_of_a_run_over()
        } else if word < THE_LAST_STAGE_WORD_OF_EASY
            || (!easy && word == THE_LAST_STAGE_WORD_OF_EASY)
        {
            self.image.scene_of_a_stage_being_rebuilt()
        } else if self.image.is_a_replay_playing() {
            self.image.scene_of_a_replay_over()
        } else if self.image.the_difficulty_now() != DIFFICULTY_EXTRA {
            self.enters_the_ending();
            return;
        } else {
            self.image.scene_of_a_run_over()
        };
        self.image.is_in_scene(wanted, last);
    }

    /// The script's events for the frame the stage has reached, the bullets, the player and the hit test.
    fn plays_the_stage(&self, now: u32) {
        // The game manager's update raising the run's score to the panel's where the panel is ahead of it —
        // `0x140023f71` — which is what gives a stage the run walked into its score back.
        if self.image.the_score_the_panel_shows() > self.image.the_score() {
            self.image
                .the_runs_own_score_is(self.image.the_score_the_panel_shows());
        }
        let last = self.last_stage_frame.get();
        if now < last {
            // A stage put back is a stage whose bullets are gone: what a restore could not put back is dropped
            // to agree with the memory it did.
            self.bullets.borrow_mut().clear();
            self.hit_window.set(0);
            self.bomb_left.set(0);
        } else {
            let reached = |at: u32| last < at && at <= now;
            if reached(MIDBOSS_ARRIVES) {
                self.image.a_boss_is_fighting(BOSS_LIFE, 0);
            }
            if reached(CARD_STARTS) {
                self.image.a_boss_is_fighting(BOSS_LIFE, 0);
                self.image
                    .starts_the_card(SCRIPTED_CARD, SCRIPTED_CARD_NAME);
            }
            if reached(ATTACK_CHANGES) {
                self.image.no_spellcard_is_running();
                self.image.a_boss_is_fighting(BOSS_LIFE, 0);
            }
            if reached(MIDBOSS_BEATEN) {
                self.image.no_boss_is_fighting();
                self.image.no_spellcard_is_running();
            }
            if self.fires.get()
                && now > last
                && now >= FIRST_BULLET_AT
                && (now - FIRST_BULLET_AT).is_multiple_of(A_BULLET_EVERY)
            {
                self.shoots_at_the_player();
            }
            // **Nearly out of life a second before she goes**, which is how the real game's bosses leave:
            // the Extra's read `1` on the last frame she stood on where one a run was lost under stood at
            // whatever her attack held — see `orb_real`'s
            // `the_extra_of_th06c_played_to_its_clear_with_no_hit_and_no_bomb`, whose check is that
            // difference. **On frames of its own before the one she leaves on**, which are the frames a trace
            // could read it at all.
            if self.state().boss_present && now > last {
                let life = match now >= A_BOSS_IS_NEARLY_OUT_AT {
                    true => A_BEATEN_BOSSES_LIFE,
                    false => BOSS_LIFE,
                };
                self.image
                    .a_boss_is_fighting(life, self.image.the_bosses_attack_has_run_for() + 1);
            }
        }
        self.last_stage_frame.set(now);

        let word = self.image.the_word_the_run_acts_on() as u16;
        self.comes_back_and_is_untouchable();
        self.fires_a_shot(word);
        self.moves_the_player(word);
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
                    self.image.the_player_has_been_hit(A_BOMB_IS_TAKEN_FOR);
                    self.hit_window.set(A_BOMB_IS_TAKEN_FOR);
                }
            }
            // Hit, inside the window: a bomb pressed takes the hit back, and the window running out is the
            // death made final, which orb reads off the state still the hit's with no frames left under it
            // (see `Th06c::where_a_counted_death_stands`). **The state stands on after it** until the count
            // the hit put to nought has reached thirty (0x14004a93e, the `cmp $0x1e` at 0x14004a9c1), the
            // count going on every update the player is not untouchable (0x14004acdd) — and only then the life.
            2 => {
                let (_, bombs) = self.image.lives_and_bombs();
                let count = self.image.the_frames_the_player_is_untouchable_for();
                if word & BOMB != 0 && bombs > 0 && self.hit_window.get() > 0 {
                    self.a_bomb_goes_off();
                    self.hit_window.set(0);
                } else if self.hit_window.get() > 0 {
                    let left = self.hit_window.get() - 1;
                    self.hit_window.set(left);
                    self.image.the_frames_a_bomb_is_still_taken_in(left);
                    self.image.the_players_count_is(count + 1);
                } else if count >= RESPAWN_TAKES {
                    self.dies();
                } else {
                    self.image.the_players_count_is(count + 1);
                }
            }
            _ => {}
        }
        if self.bomb_left.get() > 0 {
            self.bomb_left.set(self.bomb_left.get() - 1);
            if self.bomb_left.get() == 0 {
                self.image.a_bomb_is_going_off(false);
            }
        }
        self.image.the_players_box_is((PLAYER_HALF, PLAYER_HALF));
        self.writes_the_bullets_down();
    }

    /// **The death's end, 0x14004a9d8 on**: the walk-in begun from the field's middle and 64 up from its foot,
    /// with nothing on its count, and the life taken with the bombs put to the run's starting count — the
    /// configuration's, or three for the Extra and practice (0x14004aa79 through 0x14004aaa1); or, with no life
    /// to take, the continue question put up over the walk-in's state (0x14004aa5a). Nothing held: the byte that
    /// holds everything is an enemy script's alone.
    fn dies(&self) {
        let (lives, _) = self.image.lives_and_bombs();
        self.image.the_player_is_coming_back(0);
        self.image
            .the_player_is_at(PLAYER_STARTS_AT.0, PLAYER_STARTS_AT.1);
        if lives <= 0 {
            self.out_of_lives.set(true);
            self.image.the_continue_question_is_up();
            return;
        }
        // The run's starting count being the copy the stage callback keeps of the bombs a run's start gave
        // (`[0x140bc28ed]`, 0x1400242d0), which for a run on the configuration's numbers is the configuration's.
        let bombs =
            match self.image.the_difficulty_now() >= DIFFICULTY_EXTRA || self.state().practice {
                true => EXTRA_BOMBS,
                false => self.image.the_configuration_now().1 as i8,
            };
        self.image.has_lives_and_bombs(lives - 1, bombs);
        // And into the walk-in's first update on the same one, as the jump to 0x14004aae8 goes.
        self.comes_back_and_is_untouchable();
    }

    /// A bomb pressed: one fewer, the bullets swept off the field for as long as it goes off, and the player
    /// untouchable under it — the state 3 its callback writes (0x140009a5f) with the count its first update
    /// gives (0x140009685), whatever state the bomb was pressed in.
    fn a_bomb_goes_off(&self) {
        let (lives, bombs) = self.image.lives_and_bombs();
        self.image.has_lives_and_bombs(lives, bombs - 1);
        self.image
            .the_player_is_invulnerable_for(A_BOMB_KEEPS_THE_PLAYER_UNTOUCHABLE_FOR);
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

    /// **The walk-in and the invulnerability after it**, as the player's own update runs them and before the
    /// shot's press, which is the order the update has them in.
    ///
    /// **The walk-in ends on the count and not on frames of its own**: `0x14004ab7a` holds the count at
    /// `+0x7378` — `0x14004aaf2` loads it — against thirty, and at thirty `0x14004ab80` onwards writes the
    /// state 3 and `0xf0`. A death leaves nought there (`0x14004aa1c`), so a respawn walks in for thirty
    /// updates; a stage's register leaves 120 (`0x14004a14c`), so a stage's walk-in ends on its first update.
    /// **Measured**: a real Extra read `state=3` on its stage frame 1 and every frame to 239 and `state=0`
    /// from 240, with the shot's cycle begun on frame 1 — which a thirty-frame walk-in, skipping the press,
    /// could not have. And state 3 counts down in the same update (`0x14004ac4e`).
    fn comes_back_and_is_untouchable(&self) {
        if self.image.the_players_state() == 1 {
            let count = self.image.the_frames_the_player_is_untouchable_for();
            if count >= RESPAWN_TAKES {
                self.image.the_player_is_invulnerable_for(INVULNERABLE_FOR);
            } else {
                self.image.the_player_is_coming_back(count + 1);
            }
        }
        if self.image.the_players_state() == 3 {
            let left = self.image.the_frames_the_player_is_untouchable_for() - 1;
            if left <= 0 {
                self.image.the_player_is_playing();
            } else {
                self.image.the_player_is_invulnerable_for(left);
            }
        }
    }

    /// **The shot's firing cycle stepped**, as the player's own update steps it: a press of the shot starts
    /// one where the pair is idle — `0x14004b340` — and every update a cycle is out moves it one on —
    /// `0x14004b8aa` — to its end at thirty, `0x14004b8be`. What a cycle emits is not laid out here: the
    /// pair is what a pick-up's playback is held to the run by, and the part of it a fake can be faithful
    /// about.
    ///
    /// **And the press is the half a walk-in and a hit skip, where the advance is not**: `0x14004ad0e` and
    /// `0x14004ad17` send states 2 and 1 to `0x14004b37a`, past the press and before the advance.
    fn fires_a_shot(&self, word: u16) {
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

    /// The player moved by the directions in the word, by the step a frame moves them — halved where the focus
    /// is held — and held inside the field.
    fn moves_the_player(&self, word: u16) {
        // Dying or walking in: 0x14004ad0e and 0x14004ad17 send both past the move.
        if matches!(self.image.the_players_state(), 1 | 2) {
            return;
        }
        // **The first of a pair and not the two netted**: 0x14004a4b0 tests up before down and right before left
        // and takes the first it finds, so both of a pair held are the first of it.
        let across = match (word & RIGHT != 0, word & LEFT != 0) {
            (true, _) => 1,
            (false, true) => -1,
            (false, false) => 0,
        };
        let down = match (word & UP != 0, word & DOWN != 0) {
            (true, _) => -1,
            (false, true) => 1,
            (false, false) => 0,
        };
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
            (x + across as f32 * step).clamp(PLAYER_AREA.0, PLAYER_AREA.0 + PLAYER_AREA.2),
            (y + down as f32 * step).clamp(PLAYER_AREA.1, PLAYER_AREA.1 + PLAYER_AREA.3),
        );
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
        // **The boss and the wave both**, which is what the game's own manager leaves: it walks its slots and
        // counts every one that is alive, and a boss is one of them. The wave is what an e2e test drives —
        // see [`Fake::the_stage_has_enemies`] — because what this fake scripts of a stage is the fight.
        self.image
            .enemies_alive(i32::from(self.state().boss_present) + self.enemies.get());
    }

    /// The ending a cleared run hands over to: the stage's jobs gone, the ending's own in their place, with
    /// the supervisor's state still a stage's — see `orb_core::game::th06c::ENDING_UPDATE`.
    fn enters_the_ending(&self) {
        self.cleared.set(Some((
            self.image.the_character_chosen() * 2 + self.image.the_shot_chosen(),
            self.image.the_difficulty_now() as usize,
        )));
        self.tears_the_stage_down();
        self.image.an_ending_is_running();
        self.ending_updates.set(ENDING_UPDATES);
        self.roll_updates.set(ROLL_UPDATES);
    }

    /// The ending's own job: its script and then the roll, and the screen a run ends at after both — which is
    /// what `0x140018f05` writes seven for.
    fn ending(&self) -> i32 {
        if self.ending_updates.get() > 0 {
            self.ending_updates.set(self.ending_updates.get() - 1);
        } else if self.roll_updates.get() > 0 {
            self.roll_updates.set(self.roll_updates.get() - 1);
        } else {
            self.image.is_in_scene(
                self.image.scene_of_a_run_over(),
                self.image.the_scene_last_acted_on(),
            );
            return chain_result::REMOVED;
        }
        chain_result::CONTINUES
    }

    /// The screen a run ends at, walked by the presses whoever finished the run makes, as the real one's
    /// states are walked — see `Image::RESULT_STATS` for each: the name entry on to the stats, the stats on
    /// a decide of their own to a fade, the fade to the question about saving a replay, the question to its
    /// cursor, and the cursor — on いいえ — out on a decide. Three decides from the name to the title, which
    /// is why a recipe that pressed two sat in the question, and why the real one's are tapped. The state it
    /// leaves in puts the supervisor back and writes the file — `0x140054ab0`, its deinit.
    fn result(&self) -> i32 {
        let state = self.image.the_state_the_screen_a_run_ends_at_is_in();
        if state != self.result_state_seen.get() {
            self.result_state_seen.set(state);
            self.result_state_frames.set(0);
        }
        let up_for = self.result_state_frames.get() + 1;
        self.result_state_frames.set(up_for);
        if state == RESULT_LEAVING {
            self.writes_the_score_file();
            self.image.is_in_scene(
                self.image.scene_of_the_front_end(),
                self.image.the_scene_last_acted_on(),
            );
            return chain_result::REMOVED;
        }
        if state == RESULT_TYPING_A_NAME {
            self.types_its_name(up_for);
            return chain_result::CONTINUES;
        }
        if matches!(
            state,
            RESULT_RANKING_COMING_UP | RESULT_RANKING | RESULT_RANKING_GOING
        ) {
            return self.walks_its_ranking(state, up_for);
        }
        let decide = self.pressed(MENU_DECIDE);
        let next = match state {
            RESULT_STATS if decide && up_for >= RESULT_STATS_TAKE_A_PRESS_AFTER => {
                RESULT_STATS_FADING
            }
            RESULT_STATS_FADING if up_for >= RESULT_STATS_FADE_FOR => RESULT_SAVE_REPLAY_QUESTION,
            RESULT_SAVE_REPLAY_QUESTION if up_for >= RESULT_QUESTION_COMES_UP_AFTER => {
                RESULT_SAVE_REPLAY_ASKED
            }
            // The question comes up on いいえ, so its decide is the way out — as a launch of the real one
            // tapping decide from the name showed, the title back 200 frames on.
            RESULT_SAVE_REPLAY_ASKED if decide => RESULT_LEAVING,
            _ => return chain_result::CONTINUES,
        };
        self.image.the_screen_a_run_ends_at_is_in(next);
        chain_result::CONTINUES
    }

    /// **The name entry, case 9 of 0x14004ff40**: on its first update eight spaces, the cursor on the grid's
    /// first cell and the run placed among the records the file holds — below ten of them, the stats with no
    /// name asked for (0x14004e750's answer against 9); then, past its first `0x1d` updates, the four
    /// directions walking the grid round over its spaces, and a decide typing the cell under the cursor at the
    /// count typed (capped at the eighth byte), a space for the cell `0x5e`, and leaving for the stats on the
    /// end item. Each key is taken on its edge or on the read's repeat.
    fn types_its_name(&self, up_for: i32) {
        if up_for == 1 {
            self.image.the_entry_holds("");
            self.image.the_characters_typed_are(0);
            self.image.the_entrys_cursor_is_on(0);
            if self.the_place_the_run_takes() >= RESULT_PLACES_RANKED {
                self.image.the_screen_a_run_ends_at_is_in(RESULT_STATS);
            }
            return;
        }
        if up_for - 1 <= RESULT_THE_ENTRY_TAKES_A_KEY_AFTER {
            return;
        }
        let cells = RESULT_GRID.len() as i32;
        let blank = |cell: i32| RESULT_GRID[cell as usize] == b' ';
        let walked = |step: &dyn Fn(i32) -> i32| {
            let mut cell = self.image.the_entrys_cursor();
            loop {
                cell = step(cell);
                if !blank(cell) {
                    break cell;
                }
            }
        };
        let across = RESULT_GRID_ACROSS;
        if self.pressed_or_repeated(UP) {
            let cell = walked(&|cell| {
                if cell - across < 0 {
                    cell - across + cells
                } else {
                    cell - across
                }
            });
            self.image.the_entrys_cursor_is_on(cell);
        }
        if self.pressed_or_repeated(DOWN) {
            let cell = walked(&|cell| {
                if cell + across >= cells {
                    cell + across - cells
                } else {
                    cell + across
                }
            });
            self.image.the_entrys_cursor_is_on(cell);
        }
        if self.pressed_or_repeated(LEFT) {
            let cell = walked(&|cell| {
                if cell % across == 0 {
                    cell + across - 1
                } else {
                    cell - 1
                }
            });
            self.image.the_entrys_cursor_is_on(cell);
        }
        if self.pressed_or_repeated(RIGHT) {
            let cell = walked(&|cell| {
                if (cell + 1) % across == 0 {
                    cell + 1 - across
                } else {
                    cell + 1
                }
            });
            self.image.the_entrys_cursor_is_on(cell);
        }
        if !self.pressed_or_repeated(ENTER) {
            return;
        }
        let cursor = self.image.the_entrys_cursor();
        if cursor > RESULT_TYPES_A_SPACE {
            self.image.the_screen_a_run_ends_at_is_in(RESULT_STATS);
            return;
        }
        let byte = match cursor == RESULT_TYPES_A_SPACE {
            true => b' ',
            false => RESULT_GRID[cursor as usize],
        };
        let typed = self.image.the_characters_typed();
        self.image
            .the_entrys_byte_is(typed.min(RESULT_NAME_BYTES - 1) as usize, byte);
        let typed = (typed + 1).min(RESULT_NAME_BYTES);
        self.image.the_characters_typed_are(typed);
        if typed == RESULT_NAME_BYTES {
            self.image.the_entrys_cursor_is_on(RESULT_THE_END_ITEM);
        }
    }

    /// **The ranking the Score item builds, states 0 to 2 of 0x14004ff40**: up for `0x14` updates before its
    /// menu, then a menu of seven on the count beside the name entry's — up and down round it on their edge or
    /// the repeat (0x14004f200), a cancel putting the cursor on the seventh, a decide there going to state 2 —
    /// and state 2 asking the supervisor for the front end once past `0x3b` updates, the deinit writing the file
    /// as every way out of this screen does. A decide on the five pages or the sixth item opens what this
    /// laid-out game does not draw, and stays.
    fn walks_its_ranking(&self, state: i32, up_for: i32) -> i32 {
        match state {
            RESULT_RANKING_COMING_UP => {
                if up_for >= RESULT_RANKING_COMES_UP_AFTER {
                    self.image.the_screen_a_run_ends_at_is_in(RESULT_RANKING);
                }
            }
            RESULT_RANKING => {
                let items = RESULT_RANKING_ITEMS;
                let mut cursor = self.image.the_characters_typed();
                if self.pressed_or_repeated(UP) {
                    cursor = (cursor - 1).rem_euclid(items);
                }
                if self.pressed_or_repeated(DOWN) {
                    cursor = (cursor + 1).rem_euclid(items);
                }
                if self.pressed(ENTER) && cursor == RESULT_RANKING_THE_WAY_OUT {
                    self.image
                        .the_screen_a_run_ends_at_is_in(RESULT_RANKING_GOING);
                }
                if self.pressed(MENU_CANCEL) {
                    cursor = RESULT_RANKING_THE_WAY_OUT;
                }
                self.image.the_characters_typed_are(cursor);
            }
            _ => {
                if up_for > RESULT_RANKING_GOES_AFTER {
                    self.writes_the_score_file();
                    self.image.is_in_scene(
                        self.image.scene_of_the_front_end(),
                        self.image.the_scene_last_acted_on(),
                    );
                    return chain_result::REMOVED;
                }
            }
        }
        chain_result::CONTINUES
    }

    /// Where the run's score falls among the `HSCR` records of the copy the startup read: how many hold more.
    fn the_place_the_run_takes(&self) -> usize {
        let score = self.image.the_score();
        self.the_records_the_game_parses()
            .unwrap_or_default()
            .iter()
            .filter(|record| record.magic() == b"HSCR")
            .filter_map(|record| record.bytes.get(0xc..0x10))
            .filter(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) > score)
            .count()
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
        (handle as isize != orb_core::score::INVALID_HANDLE).then(|| {
            self.opens
                .borrow()
                .last()
                .expect("an open just made")
                .0
                .clone()
        })
    }

    /// The startup's read, `0x14002c239` on: the copy is what the file the open landed in holds, and nothing
    /// where the open failed — a first launch.
    fn reads_the_score_file_at_startup(&self) {
        if let Some(name) = self.opens_the_score_file(false) {
            self.loads_the_copy_out_of(&name);
        }
    }

    fn loads_the_copy_out_of(&self, name: &str) {
        if let Some(bytes) = self.sim().files().get(name) {
            self.image.the_score_file_copy_is(&bytes);
        }
    }

    /// A file of this game's, where orb's own read of it — `orb_api::fs::read`, by the same relative name the
    /// game opens it by — finds it too.
    fn puts_the_file(&self, name: &str, bytes: &[u8]) {
        self.sim().files().put(name, bytes);
    }

    /// What the title lights `Extra Start` from: the `CLRD` records of the copy, as `0x14002d526` reads them.
    fn a_clear_is_in_the_copy(&self) -> bool {
        self.the_records_the_game_parses()
            .is_some_and(|records| score_file::a_clear_is_among(&records))
    }

    /// The result screen's deinit, `0x140054ab0` on: the file built again out of the records the copy held
    /// and the run's own, put in a block off the heap in the copy's place (`0x140055555`..`0x140055583`) and
    /// written whole.
    fn writes_the_score_file(&self) {
        let mut records = self.the_records_the_game_parses().unwrap_or_default();
        if records.is_empty() {
            records.push(score_file::th6k());
        }
        if let Some((shot, difficulty)) = self.cleared.take() {
            records = with_the_clear(records, shot, difficulty);
        }
        records.push(score_file::record(
            b"HSCR",
            &self.image.the_score().to_le_bytes(),
        ));
        let file = score_file::encode(&records, 0x33);
        self.image.the_score_file_copy_is(&file);
        if let Some(name) = self.opens_the_score_file(true) {
            self.puts_the_file(&name, &file);
        }
    }
}

/// **`records` with `shot`'s `CLRD` holding a clear of `difficulty` in both arrays**, the record made where there is
/// none — the four the game makes with no file hold `1` in every byte.
fn with_the_clear(mut records: Vec<Record>, shot: u8, difficulty: usize) -> Vec<Record> {
    const NO_RETRY: usize = 0xc;
    const AT_ALL: usize = 0x11;
    const SHOT: usize = 0x16;
    match records
        .iter_mut()
        .find(|record| record.magic() == b"CLRD" && record.bytes.get(SHOT) == Some(&shot))
    {
        Some(record) => {
            record.bytes[NO_RETRY + difficulty] = CLEARED;
            record.bytes[AT_ALL + difficulty] = CLEARED;
        }
        None => {
            let mut reached = [1; 5];
            reached[difficulty] = CLEARED;
            records.push(score_file::clrd(shot, reached, reached));
        }
    }
    records
}

/// The game's own records as an installation somebody has played leaves them: the header, and Reimu's
/// first shot having cleared Normal or reached stage 1 alone — what `Extra Start` is lit from.
fn records_holding(a_clear: bool) -> Vec<Record> {
    let normal = if a_clear { CLEARED } else { 1 };
    vec![
        score_file::th6k(),
        score_file::clrd(0, [1, normal, 1, 1, 1], [1; 5]),
    ]
}

/// `0x140007620`: the game's own capture of the field into the texture its pause draws behind it — nothing of
/// its own here, there being no picture to capture; what is under test is that it is asked for and the
/// request put back, which is the caller's.
unsafe extern "win64" fn the_games_capture(
    _renderer: usize,
    _request: i32,
    _left: i32,
    _top: i32,
    _width: i32,
    _height: i32,
) {
}

/// `0x14002cc00`: the game's own run start, which orb calls where it writes a run into being.
unsafe extern "win64" fn the_games_run_start() -> i32 {
    running().the_games_own_run_start();
    0
}

/// The game's `malloc`, a function of its own exe: a block off the laid-out heap.
unsafe extern "win64" fn malloc(len: usize) -> usize {
    running().image.takes_a_block_off_its_heap(len)
}

/// And its `free`: nothing, the heap never taking a block back — see `Image::takes_a_block_off_its_heap`.
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

// ---- the ten calls handed over ----

/// `0x1400256b0`: the calc chain's walk, entered with the pointer the frame keeps its exit code in.
unsafe extern "win64" fn run_calc_chain(_leaving_with: *mut i32) -> u8 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.sim().clock().advance_micros(fake.work_this_frame());
    fake.runs_the_calc_chain()
}

/// `0x14000f620`: the element taken out of whichever chain holds it.
unsafe extern "win64" fn takes_an_element_out(_: usize, elem: usize) {
    running().image.cuts_from_the_chain(elem);
}

/// `0x1400591d0`: the frame's view — **the sprites a draw left queued flushed first, as the game's own does**,
/// and then the viewport out of the block the frame wrote it into, and the matrix after it, which nothing here
/// is about.
unsafe extern "win64" fn sets_the_frames_view() {
    let fake = running();
    let queued = fake.image.the_sprites_queued();
    if queued > 0 {
        let (sprites, flushed_as) = fake.image.the_batch();
        unsafe { flushes_the_sprites(sprites, queued, 4, flushed_as, 1) };
        fake.image.queues_sprites(-queued);
    }
    fake.viewports
        .borrow_mut()
        .push(fake.image.the_frames_view());
}

/// `0x140062330`: the renderer's batch drawn, which is the one call a sprite queued is ever drawn by.
unsafe extern "win64" fn flushes_the_sprites(
    _sprites: usize,
    _count: i32,
    _topology: i32,
    _flushed_as: i32,
    _one: i32,
) {
    running().launch.asked_for(FLUSH);
}

/// **The first thing the frame's sound catch-up asks this game**, so this is where the sound's place in the
/// frame is noted: `tops_up_the_sound` is orb's own function here and not the game's, so there is no call
/// of the game's own for the frame loop to reach at all — see `FrameCalls::orbs_own`. Which also means the
/// note lands only on a frame that has music behind it, that catch-up doing nothing on any other.
unsafe extern "win64" fn the_sounds_rate() -> i32 {
    running().launch.asked_for(SOUND);
    44_100
}

unsafe extern "win64" fn where_the_music_is() -> i64 {
    0
}

unsafe extern "win64" fn how_long_the_track_is(_track: i32) -> i64 {
    0x10_0000
}

unsafe extern "win64" fn seeks_the_music(to: i64, track: i32) {
    running().seeks.borrow_mut().push((to, track));
}

unsafe extern "win64" fn puts_the_rate_together(_on: usize) {}

/// **The picture finished, which for this game is the composite and the hand-over in one call** — and so
/// the swapchain's own `Present` called from inside it, as the real `FINISHES_THE_PICTURE` does.
///
/// **That call is the whole reason orb's overlay reaches this game's screen.** After `dxgi::take` the slot
/// holds orb's own stand-in, which draws the overlay over the finished frame and then presents; a fake that
/// presented only through `dxgi::present` — the frame loop's own call, made after this — would never reach
/// that stand-in, and no e2e test could see a single quad of a canvas. Which is what happened: the five
/// tests over the question went red the moment the drawing moved to the present, because the fake was the
/// one game in the world whose renderer presented nowhere.
unsafe extern "win64" fn finishes_the_picture() {
    let fake = running();
    fake.pictures.set(fake.pictures.get() + 1);
    let swapchain = fake.image.the_swapchain();
    let vtable = fake.image.space().read::<usize>(swapchain);
    // The `Present` slot, counted in pointers as `orb_core::dxgi` counts it: `IUnknown`'s three,
    // `IDXGIObject`'s four and `IDXGIDeviceSubObject`'s one before it.
    //
    // **A named slot and not `8 * size_of`**, which clippy's `manual_bits` reads as a count of bits in a
    // `usize` — it is eight pointers into a vtable, and the multiplication says so this way round.
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

/// **`IDXGISwapChain::QueryInterface`, answering that this object is one.**
///
/// The IID is written here rather than compared against `orb_core`'s, which is the rule this file's
/// numbers keep: one taken from the code under test is right on both sides however wrong it is. Any IID
/// answers, because the only ask orb makes of a swapchain is whether it is a swapchain — a laid-out game
/// that refused the rest would be modelling an interface nothing here asks for.
unsafe extern "system" fn swapchain_query_interface(
    object: usize,
    _iid: *const u8,
    answered: *mut usize,
) -> i32 {
    unsafe { *answered = object };
    0
}

/// And the `Release` that gives back what the ask above added, which is counted and no more: what the
/// object *is* is this game's memory, and nothing here frees it.
unsafe extern "system" fn swapchain_release(_object: usize) -> u32 {
    let fake = running();
    fake.swapchain_releases
        .set(fake.swapchain_releases.get() + 1);
    0
}

/// **`IDXGISwapChain::GetDesc`, whose window is this game's own** — which is the one field of that
/// description orb reads, see `orb_core::dxgi::the_window`.
///
/// The offset is written here rather than taken from the code under test, as every number in this fake is:
/// `DXGI_MODE_DESC`'s seven words come to 28 bytes, `DXGI_SAMPLE_DESC`'s two make 36, the buffer's usage
/// and count make 44, and the `HWND` after them is eight-aligned. The rest is left at nought, orb reading
/// none of it.
unsafe extern "system" fn swapchain_describes_itself(_swapchain: usize, into: *mut u8) -> i32 {
    // The first two dwords are the buffer's own width and height — see `super::BACK_BUFFER`, which
    // `orb_core::dxgi::the_size_of` reads and the canvas sets its viewport to — and the window is at 48.
    let mut description = [0u8; 72];
    description[..size_of::<u32>()].copy_from_slice(&super::BACK_BUFFER.0.to_le_bytes());
    description[size_of::<u32>()..2 * size_of::<u32>()]
        .copy_from_slice(&super::BACK_BUFFER.1.to_le_bytes());
    unsafe { std::ptr::copy_nonoverlapping(description.as_ptr(), into, description.len()) };
    unsafe {
        std::ptr::copy_nonoverlapping(
            super::WINDOW.0.to_le_bytes().as_ptr(),
            into.add(48),
            size_of::<usize>(),
        )
    };
    0
}

/// **The swapchain's own `Present`, which is the frame handed over.** The same three things the game's own
/// present does in this fake: the call noted, when it happened written down, and the simulated display
/// told a frame arrived.
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
    _parameters: usize,
) -> i32 {
    unsafe { swapchain_presents(swapchain, interval, flags) }
}

/// The draw chain's one job, which is what drawing anything here is: a quad's worth of sprite queued into the
/// renderer's batch, as every draw of the real one queues its own.
unsafe extern "win64" fn draw_job(_on: usize) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.image.queues_sprites(4);
    chain_result::CONTINUES
}

/// The host's `GetKeyboardState`, which is what orb's own read calls through to: the simulated keyboard.
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

/// The game's own whole frame, for a launch with orb's loop off — which no launch of this game is, its frame
/// being what orb replaces outright. The pieces in the frame's own order all the same.
///
/// **`win64` and not `fastcall`, as every one of this game's own functions here is**: this exe is
/// 64-bit, and `"fastcall"` is an x86 convention rustc warns about for a 64-bit target and will
/// refuse — see `orb_core::runtime::OnItsThis`, whose spelling these have to match.
extern "win64" fn own_render(_window: *mut c_void) -> i32 {
    let chain = Th06c.chain();
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
/// here is [`own_render`] and nothing else. The two do the same three things because a frame handed over is
/// a frame handed over whichever of them did it.
unsafe extern "win64" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
}

/// Never reached: this game's keyboard is read through the lookup orb answers, and its word is built here.
extern "system" fn input() -> u16 {
    0
}

/// `0x1400572d0`: the stage built, which draws from the generator as it goes.
extern "C" fn stage_building(_stage: i32) -> i32 {
    let fake = running();
    fake.image.draws_from_the_generator();
    fake.image.draws_from_the_generator();
    0
}

/// `0x1400242d0`: the game manager's stage callback — the stage word raised, a run's numbers where this is a
/// run's start, the seed copied out of the generator with the count cleared, and the stage built out of them
/// through orb's hook over the build, which is where a resumed run's seed goes in.
extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    let fake = running();
    let image = &fake.image;
    let starting = image.the_scene_now() != image.scene_of_a_stage_being_rebuilt();
    image.the_games_stage_word_is(image.the_games_stage_word() + 1);
    if starting {
        image.fills_the_card_records();
        image.the_run_has_scored(0, 0);
        image.shot_is_at_power(0);
        // The Extra's count of extends starts at four, so the score pays it none: 0x1400242d0 puts the
        // difficulty to the Extra's where the word, not yet raised there, is past 5, and the count to four
        // where the difficulty is the Extra's. The word here has been raised already.
        if image.the_games_stage_word() > THE_EXTRAS_STAGE_WORD {
            image.the_difficulty_is(DIFFICULTY_EXTRA);
        }
        image.the_score_has_paid_for(match image.the_difficulty_now() >= DIFFICULTY_EXTRA {
            true => THE_EXTRAS_EXTENDS_PAID_FOR,
            false => 0,
        });
        image.the_power_items_towards_the_score(0);
        image.the_rank_is(RANK_AT_A_RUNS_START, 0);
    } else {
        // **What a transition does to the numbers, which is the whole of it**: the panel's own copy of the
        // score put back to the run's and the step towards the next rank put back to nothing —
        // `0x140024cf5`'s first instruction and `0x140024d10`. The lives, the bombs, the power and the rank
        // are written nowhere in that branch, which is what `a_stage_transition_of_th06c` is about.
        image.the_panel_shows(image.the_score());
        image.the_rank_is(image.the_rank(), 0);
    }
    // **And the run's own score field left at nought, whichever way the callback came** — `0x14002548d`, in
    // the tail both branches reach — the score standing in the panel's alone until the stage's first update
    // raises the run's back to it. A chapter written down as the callback returns reads it there.
    image.the_runs_own_score_is(0);
    let (seed, _) = image.the_generator();
    image.the_stage_began_with_the_seed(seed);
    image.the_generator_is(seed, 0);
    image.has_been_in_the_stage_for(0);
    image.the_script_has_run_for(0);
    image.set_replay_clock(0);
    image.no_boss_is_fighting();
    image.no_spellcard_is_running();
    image.no_dialogue_is_up();
    image.everything_is_held(false);
    fake.bullets.borrow_mut().clear();
    fake.writes_the_bullets_down();
    fake.last_stage_frame.set(0);
    fake.hit_window.set(0);
    fake.bomb_left.set(0);
    // `Stage::RegisterChain`, through orb's hook over it.
    orb_core::runtime::stage_building(image.the_games_stage_word());
    // And the player, after the stage's own build: where the stage starts them, with the step and the box
    // the rules a bot plays by read — and **walking back in**, which is what this game's own register at
    // `0x140049ea0` leaves them as. Its three writes are the state 1 at `0x14004a132`, the count 120 at
    // `0x14004a14c` and a timer orb does not read; a count already past the walk-in's thirty ends it on the
    // stage's first update, and the `0xf0` of invulnerability behind it is why nothing can reach a player for
    // the first 240 frames of a stage. See `the_player_a_stage_of_th06c_starts`, which is the reading written
    // down, and [`Fake::comes_back_and_is_untouchable`].
    image.the_play_field_is(PLAYER_AREA.0, PLAYER_AREA.1, PLAYER_AREA.2, PLAYER_AREA.3);
    image.the_player_is_at(PLAYER_STARTS_AT.0, PLAYER_STARTS_AT.1);
    image.the_player_steps_by(A_STEP, A_STEP / 2.0_f32.sqrt());
    image.the_players_box_is((PLAYER_HALF, PLAYER_HALF));
    image.the_player_is_coming_back(THE_REGISTER_LEAVES_UNTOUCHABLE);
    image.the_frames_a_bomb_is_still_taken_in(0);
    image.a_bomb_is_going_off(false);
    0
}

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

extern "C" fn create_game_window(_instance: *mut c_void) -> i32 {
    0
}

unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

extern "C" fn pad_buttons() -> *mut c_void {
    std::ptr::null_mut()
}

extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

extern "C" fn init_d3d_device() -> i32 {
    0
}

/// The game's own `CreateFileW`, as its import table holds it: the name the open landed in written down, and
/// a handle answered where the file is there — or where it is being written — and `INVALID_HANDLE_VALUE`
/// where it is not.
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

/// Never reached: this game's `CreateFileA` is not what its records go through — see [`create_file_w`].
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

/// The game's own `CreateWindowExA`, which orb's rewrite calls through: the host makes the window.
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
