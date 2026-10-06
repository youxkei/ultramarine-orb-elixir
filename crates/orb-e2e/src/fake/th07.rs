//! A th07 that is not the real one, and far less of one than [`th06`](super::th06) is.
//!
//! **It plays as much of the game's part as `Th07` reads.** Which is a frame and nothing else: its
//! memory holds the device orb draws through, the window orb asks about, the chain the two frame calls are
//! made at and the queue of quads its drawing fills, and its own frame does what th07's does in
//! th07's own draw-then-update order. Everything a run is made of is missing because `Th07` answers all
//! of it without looking — no scene to be in, no stage, no pad, and a score file that is one name and no
//! contents — so there is nothing for this game to be doing.
//!
//! What that makes it good for is the two things worth asking of a second game: that the seam holds, and
//! that the frame orb composes is that game's frame. orb attached to a game it has read a frame of gets
//! in, paces it, and has its update and draw reached inside orb's own frame — and draws nothing, there
//! being no font beside the exe to build an overlay from. **Its draw chain queues a quad**, which is why
//! the second of those can be asked at all: a frame missing what the game's own does around its drawing
//! faults here as it faulted on the machine. Over `Launch`, which is the same half of a launch th06's e2e
//! tests run on.
//!
//! Where a number here is th07's it says so. There are two: the answers its chain walk gives, and what
//! its own frame answers to the loop above it.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, CString, c_void};

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th07::Th07;
use orb_core::game::th07::image::{Image, Track};
use orb_core::mode::Mode;

use orb_sim::keys;

use super::{
    DRAW, Display, Launch, Launched, PRESENT, Presented, READS_KEYS_AFTER, SOUND, UPDATE, WINDOW,
    Work, scratch,
};

/// What the game's own chain walk answers while the game is running, and what it answers when the game
/// is leaving: nothing, and a walk that failed. All three read off `th07.exe` — 0x4347f1 turns a zero
/// into the frame answering 1, and 0x434801 turns a `-1` into 2.
const CHAIN_CARRIED_ON: i32 = 1;
pub const CHAIN_LEFT: i32 = 0;
pub const CHAIN_FAILED: i32 = -1;

/// The score file th07 asks for, which is the one name this game passes `CreateFileA`: `score.dat`,
/// beside its exe, opened twice for reading and once for writing in a measured launch.
///
/// The name the game asks for and not the name an open lands in — which of the two files that is is
/// orb's decision, and a game that named the forked file itself would be answering the question the e2e
/// test over it asks.
const SCORE_FILE: &CStr = c"score.dat";

/// `GENERIC_WRITE`, which is the one bit of `CreateFileA`'s access an open here varies: orb's fork reads
/// it to say which of the game's opens this is.
const GENERIC_WRITE: u32 = 0x4000_0000;

/// What the keyboard's half of the read answers on a frame with nothing held, which is what the word the
/// pad's bits are added to starts as — see [`input`].
const NO_KEY_HELD: u16 = 0;

/// Which key is which bit of the word th07's own read builds, as its defaults have it: the six an e2e test
/// here presses.
///
/// The order the game's own read fills them in is shoot, bomb, focus, menu, up, down, left, right and skip,
/// one bit apiece from 0x1 — which is what `Th07`'s own `PAD_UP` and the two menu masks are read off. Focus
/// and the skip are left out because no e2e test here holds either: what these have to answer is a press of
/// orb's own menus, which is the shot button, and a direction.
///
/// **The menu button is in, and it is the one that does something to a stage**: the manager takes its edge off
/// the word the read left and puts the pause menu up — see [`Fake::follows_its_manager`].
const MAP: [(u8, u16); 7] = [
    (keys::Z, 0x1),
    (keys::X, 0x2),
    (keys::ESCAPE, MENU_BUTTON),
    (keys::UP, 0x10),
    (keys::DOWN, 0x20),
    (LEFT, 0x40),
    (RIGHT, 0x80),
];

/// The bit that puts the game's own pause menu up, and the shot bit that takes it down again — its first item
/// being 続ける.
const MENU_BUTTON: u16 = 0x8;
const MENU_DECIDE: u16 = 0x1;
/// The pause menu's three states this game's e2e tests walk — setting up, on 続ける, and going down — and the
/// timer its first item and its going down wait on. The menu's own numbers.
const PAUSE_MENU_SETTING_UP: i32 = 0;
const PAUSE_MENU_ON_ITS_FIRST_ITEM: i32 = 1;
const PAUSE_MENU_CLOSING: i32 = 4;
const PAUSE_MENU_TAKES_A_DECIDE_FROM: i32 = 4;
const PAUSE_MENU_CLOSES_AT: i32 = 0x14;
/// And what its front end reads as *back* — the bomb and the menu button, which is `Th07`'s `MENU_CANCEL` —
/// and the four directions, as the word's own bits.
const MENU_CANCEL: u16 = 0x2 | MENU_BUTTON;
const BUTTON_BOMB: u16 = 0x2;
const BUTTON_UP: u16 = 0x10;
const BUTTON_DOWN: u16 = 0x20;
const BUTTON_LEFT: u16 = 0x40;
const BUTTON_RIGHT: u16 = 0x80;

/// How many frames a launch left to itself spends before its title menu is up: the real game reads its
/// archives and makes its window first, and a recipe's first press finds the title a moment later.
const TITLE_COMES_UP_AFTER: u32 = 60;

/// The four screens Start's own flow walks, by the ids `Th07` reads at `+0xd0f8`: the title, and then the three
/// the table at 0x4555ce dispatches 4, 5 and 6 to — a difficulty select, a character select and a shot select.
/// Extra Start's flow — 12, 13, 14 — is not walked here.
const SCREEN_TITLE: i32 = 0;
const SCREEN_DIFFICULTY: i32 = 4;
const SCREEN_CHARACTER: i32 = 5;
const SCREEN_SHOT_SELECT: i32 = 6;

/// **And Practice Start's own flow, which is the same three screens**: that table sends 8, 9 and 10 to the
/// indices 4, 5 and 6 go to, so what tells the two walks apart is the screen the title item wrote — Start 4 at
/// 0x455d20 and Practice Start 8 at 0x455dc7. Behind them is the select the stage is chosen at, screen 11,
/// whose handler at 0x45a583 is the one the flow does not share and the other place the practice bit is set.
const SCREEN_PRACTICE_DIFFICULTY: i32 = 8;
const SCREEN_PRACTICE_CHARACTER: i32 = 9;
const SCREEN_PRACTICE_SHOT_SELECT: i32 = 10;
const SCREEN_PRACTICE_STAGE: i32 = 11;

/// **And Extra Start's, which is the third triple through those same handlers**: 12, 13 and 14, the first of
/// them written by that item at 0x455e77.
const SCREEN_EXTRA_DIFFICULTY: i32 = 12;
const SCREEN_EXTRA_CHARACTER: i32 = 13;
const SCREEN_EXTRA_SHOT_SELECT: i32 = 14;

/// The difficulty the Extra is and the stage it is played in, neither of which any select decides: the fifth
/// of the six this game keeps a record per, and the stage after the six a run plays.
const DIFFICULTY_EXTRA: i32 = 4;
const THE_EXTRAS_STAGE: i32 = STAGES_A_RUN_PLAYS;

/// How many items each of those screens has a cursor over, and which of the title's items are Start and
/// Practice Start: the eight th07's title menu lists, the four difficulties a whole run offers, the three
/// characters and their two shots apiece, and the stages the practice select lists — that last being this
/// laid-out game's own, what a walk needs of it being a cursor with an item under it.
const TITLE_ITEMS: i32 = 8;
const DIFFICULTIES_OFFERED: i32 = 4;
const CHARACTERS: i32 = 3;
const SHOTS: i32 = 2;
const STAGES_TO_PRACTISE: i32 = 6;
const TITLE_ITEM_START: i32 = 0;
const TITLE_ITEM_EXTRA: i32 = 1;
const TITLE_ITEM_PRACTICE: i32 = 2;
/// And the title's Score item, which asks for the ranking rather than walking to a screen of its own.
const TITLE_ITEM_SCORE: i32 = 4;

/// The two phases a screen of the front end is in, as `Th07` reads them: the one its build runs in and the one
/// its items are read in — and how long each screen is up before it moves to the second. The title's gate is the
/// count itself against ten (0x455c6d, `SETTLED_FRAMES`) and the shot select's is the phase written at thirty
/// (0x459bbb) — see `Th07::menu_takes_a_press`.
const PHASE_BUILDING: i32 = 0;
const PHASE_ITEMS: i32 = 1;
const TITLE_TAKES_A_PRESS_AFTER: i32 = 10;
const A_SCREEN_TAKES_A_PRESS_AFTER: i32 = 30;
/// And the phase the title's last item sends the front end to, the updates it waits in before it asks for the
/// end of the game (its timer passing 0x3b), and that scene, the supervisor's -1.
const PHASE_ENDING_THE_GAME: i32 = 2;
const THE_WAY_DOWN_WAITS_FOR: i32 = 0x3b;
const SCENE_THE_GAME_ENDS: i32 = -1;

/// **The script of a stage this game plays on its own**, which is a stage a run started from its own front end
/// gets and a stage an e2e test placed does not: the frames of the stage its midboss arrives on, puts its card
/// up on, moves on to its next attack on and goes down on. The same shape th06's laid-out stage has, since what
/// a recipe's `play_until: { stage: 1, chapter: 2 }` needs is a second chapter arriving on its own.
///
/// Public because an e2e test about such a run holds what it reads against them.
pub const MIDBOSS_ARRIVES: u32 = 400;
pub const CARD_STARTS: u32 = 500;
pub const ATTACK_CHANGES: u32 = 700;
pub const MIDBOSS_BEATEN: u32 = 900;
/// The card that midboss puts up, by its own record, and the name the game puts on the plate for it.
const SCRIPTED_CARD: usize = 96;
const SCRIPTED_CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The field such a stage is played in and where its player starts, in the game's own units — th07's 384x448
/// field, with the player low in the middle of it — and what one frame of holding a direction moves them by,
/// straight and then diagonally. The same numbers `the_bot_playing_th07.rs` lays its stages out with.
const FIELD: (f32, f32, f32, f32) = (0.0, 0.0, 384.0, 448.0);
const PLAYER_STARTS_AT: (f32, f32) = (192.0, 384.0);
const A_STEP: f32 = 4.0;
/// Half of the box the game's own hit test gives the player, on each axis.
const PLAYER_HALF: f32 = 4.0;

/// **And what that stage shoots**: a bullet aimed at where the player is, from the top of the field, every so
/// many frames from the first — with a size of its own, and slow enough that a player who steps aside is
/// missed and one who does not is hit. A stage that shot nothing is a stage no `take_hits` could die in.
const FIRST_BULLET_AT: u32 = 60;
const A_BULLET_EVERY: u32 = 120;
const BULLETS_FROM: (f32, f32) = (192.0, 0.0);
const BULLET_SPEED: f32 = 3.0;
const BULLET_SIZE: f32 = 8.0;
/// How many of the bullet manager's slots this stage uses, from the first.
const BULLET_SLOTS_USED: usize = 16;

/// How many updates a hit stands before it is a death, which is the window th07 counts down at 0x440cf0 with the
/// state at 2 and a bomb pressed inside it a deathbomb — and how long a bomb that went off is going off for.
const A_BOMB_IS_TAKEN_FOR: i32 = 8;
const A_BOMB_LASTS: u32 = 60;

/// One bullet of that stage's own: which slot it is written down in, where it is and what it moves by a frame.
#[derive(Clone, Copy)]
struct Bullet {
    slot: usize,
    at: (f32, f32),
    step: (f32, f32),
}

/// **How many stages a run plays through before its ending**, which is the six this game's own runs have:
/// `Th07`'s own `STAGES` is eight, the two past these being the Extra and the Phantasm, and a run walks into
/// neither.
const STAGES_A_RUN_PLAYS: i32 = 6;

/// And how many updates the ending's own script and the staff roll after it run for, where a run is played to
/// its clear. This laid-out game's own numbers: the lengths are the game's, and what a case that has just
/// played six stages needs of them is that they end.
const ENDING_UPDATES: i32 = 10;
const ROLL_UPDATES: i32 = 10;

/// Which update of a run this game does not advance it on, where it is slowing itself down: 0x42e186 divides
/// the manager's own count by five and skips the one that leaves nothing over — see
/// [`Fake::slows_its_own_run_down`].
///
/// Public because an e2e test about what such an update does has to run enough of them to reach one.
pub const SLOWED_EVERY: u32 = 5;

/// The frames a play the game was asked for takes to leave the player's own queue, a run of the sound a frame:
/// 0x44c9c0's kind 2 walks one phase a run and the eighth run takes it out — which is what orb waits on before it
/// does anything more about the sound of a chapter it has put back.
pub const A_PLAY_TAKES: u32 = 8;
/// And the live bullets that slowing starts at, the `cmp` against 0x80 at 0x42e17a.
const BULLETS_THE_SLOWDOWN_STARTS_AT: i32 = 0x80;

/// The two arrow keys orb's own menus do not read, so `orb_sim::keys` does not name them — the four it does
/// read come from there, which is what makes an e2e test pressing `Z` at one of orb's questions and this game
/// reading its shot key the same key by construction.
const LEFT: u8 = 0x25;
const RIGHT: u8 = 0x27;

/// The run every e2e test over a chapter of this game plays, and what a stage of it starts with: Normal from
/// the first stage, with the lives and bombs th07's own front end puts in place and a shot part way up.
///
/// A score with digits in it rather than a round number, so that a chapter putting one number back where
/// another goes shows up as the wrong digits rather than as nothing.
///
/// Public because they are what an e2e test reading a run's numbers back holds them against: the numbers laid
/// out here and the numbers asserted there are one thing said once — see `the_run_of_th07_read_back.rs`.
pub const DIFFICULTY: i32 = 1;
pub const FIRST_STAGE: i32 = 0;
/// And whose run it is, which is what names the file a chapter of it is written down in: the first character
/// and their first shot, which is what the game's own front end leaves under its cursor.
pub const CHARACTER: u8 = 0;
pub const SHOT: u8 = 0;
pub const LIVES: f32 = 2.0;
pub const BOMBS: f32 = 3.0;
pub const POWER: f32 = 96.0;
pub const SCORE: u32 = 1_234_560;

/// **What a build leaves the rank at**, which is the starting figure the table by difficulty holds for the
/// difficulty above: what a launch of th07 read out of a stage 2 its resume had built. A stage the run walked into carries
/// its own instead — see `Fake::builds_the_stage`.
pub const A_BUILDS_OWN_RANK: u32 = 0x10;

/// How long the title menu has been up when an e2e test presses at it: past the ten frames the game's own
/// update refuses a press over, which is what `Th07::menu_takes_a_press` reads.
const SETTLED: i32 = 60;

/// What a boss of this game is put up with, which is a number orb reads and does nothing with: the life it has
/// left, in the units the game keeps it in.
const BOSS_LIFE: i32 = 2000;

/// The two scripts an ending reads, as the addresses the files they were read into would be at: its own, and the
/// staff roll's it hands over to.
///
/// Two numbers that can be told apart is the whole of what they have to be — what orb reads of either is the
/// pointer and never a byte through it, the script changing being the signal. See [`Fake::an_ending_is_running`].
const ENDING_SCRIPT: usize = 0x0700_0000;
const ROLLS_SCRIPT: usize = 0x0700_1000;

/// The tracks `thbgm.dat` holds, as its own records describe them: the name each carries, where its sound
/// starts in that file, where it loops back to, and how long it is.
///
/// Three of them because three is what a chapter's music can come back as: the stage's own track, the boss's
/// that a fight puts on instead, and **the title screen's, which is the file's first record** — the play of a
/// slot nothing was loaded into comes out as that one, so a test that reads the track back can tell it from the
/// stage's own. Small enough that a whole track fits in what a test can read back, and no two of them share a
/// place in the file, so a restore putting one where another goes shows up.
pub const TITLE_TRACK: Track = Track {
    name: c"th07_01.wav",
    at: 0x0,
    loops_at: 0x80,
    bytes: 0x200,
};
pub const TRACK: Track = Track {
    name: c"th07_02.wav",
    at: 0x1000,
    loops_at: 0x40,
    bytes: 0x400,
};
pub const FIGHT_TRACK: Track = Track {
    name: c"th07_03.wav",
    at: 0x1400,
    loops_at: 0x40,
    bytes: 0x400,
};
/// And the one a fight's second phase brings, whose name the game carries itself rather than in the stage's
/// data — see `Th07`'s `SECOND_PHASE_SONG`. `th07_13b` and not `th07_13`, which is why the two names that data
/// holds cannot answer for it.
pub const SECOND_PHASE_TRACK: Track = Track {
    name: c"th07_13b.wav",
    at: 0x1800,
    loops_at: 0x40,
    bytes: 0x400,
};

/// And the slots a stage's build loads the last two into: its own song in 0 and its boss's in 1, which is the
/// order 0x42f1d5 and 0x42f1ec load them in.
pub const STAGE_SLOT: u32 = 0;
pub const FIGHT_SLOT: u32 = 1;

/// And the slot the front end leaves its own song in, which is one of the two 0x439ec1's other call sites ask
/// for — and what a th07 session found in a chapter of a stage: `th07_01.wav`, still in slot 8, frames into a
/// stage whose own song had not started yet.
pub const FRONT_END_SLOT: u32 = 8;

/// The name a track is loaded into a slot under, which is not the name its record carries: the game hands the
/// load a path out of the stage's own table and the record holds the file's name, so what the play compares is
/// this with its directory taken off.
fn loaded_as(track: Track) -> CString {
    let mut path = b"bgm/".to_vec();
    path.extend_from_slice(track.name.to_bytes());
    CString::new(path).expect("a name with no zero in it")
}

/// And the name the stage's own data carries for it, which is that path before the load wrote `wav` over its
/// extension: the `.mid` the table holds.
fn named_in_the_table(track: Track) -> CString {
    let path = loaded_as(track);
    let path = path.to_bytes();
    let stem = path
        .split(|byte| *byte == b'.')
        .next()
        .expect("a name with an extension");
    CString::new([stem, b".mid"].concat()).expect("a name with no zero in it")
}

/// The buffer that track is played out of, and how much of it the streaming thread writes at a time: a
/// fraction of the track, so that a chapter's worth of it is a few refills rather than one.
pub const STREAM_BUFFER: usize = 0x80;
pub const STREAM_NOTIFY: u32 = 0x20;

/// What the stream looks like from outside the game's memory: the bytes in the buffer, the cursor the mixer is
/// at, how far the file has been read and whether it is playing.
///
/// The four together, because what a restore has to put back is the four together — see
/// `the_music_across_a_restore_of_th07.rs`.
#[derive(PartialEq, Eq, Debug)]
pub struct Stream {
    pub buffered: Vec<u8>,
    pub play_cursor: u32,
    pub position: i32,
    pub playing: bool,
    /// Whether the buffer is going round, which for a streaming one is the only way it can be played — see
    /// `orb_sim::Sound::looping`.
    pub looping: bool,
}

/// The class th07 registers its window with, which is th06's too: `BASE`, at 0x497bd0 in this exe. What
/// orb recognises the window it plays in by — see `orb_core::window::is_game_class`, and the size the game
/// asks for is [`Th07::content_size`]'s, orb deciding what it gets instead.
const WINDOW_CLASS: &CStr = c"BASE";

/// And a class of some other window this game makes, which is what an ask orb hands straight on looks like: any
/// name but the one above — see [`Fake::creates_another_window`].
const OTHER_WINDOW_CLASS: &CStr = c"th07-notify";

/// The ask itself, which is what a pass-through leaves standing and what orb rewrites every argument of: a
/// position, a size and a style with a caption on it.
///
/// The three are this file's own stand-in for the game's own arguments — what they have to be is *nothing like*
/// what orb lays out, so that a window read back at these numbers is one orb decided nothing about. Which of
/// them the real th07 passes is not read here: orb keys on the class, and the size a window comes out is
/// [`Th07::content_size`]'s ratio and `orb.yaml`'s.
pub const ASKED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x1000_0000;
pub const ASKED_AT: (i32, i32) = (17, 23);
pub const ASKED_SIZE: (i32, i32) = (646, 505);

/// And what a whole frame answers, which the message pump reads: zero to be called again, and the two
/// above zero to leave. th07's own, at 0x4346f2, 0x4347f7 and 0x434807.
pub const FRAME_KEPT_RUNNING: i32 = 0;
pub const FRAME_LEFT: i32 = 1;
pub const FRAME_FAILED: i32 = 2;

/// A th07 laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// What this game's chain walk answers. [`CHAIN_CARRIED_ON`] for the whole of a launch here: no
    /// e2e test over this game asks it to leave, th07's half being a frame and nothing else. A `Cell`
    /// all the same, because what reads it is a bare `extern` function with the ABI's arguments and
    /// nothing else — the same shape th06's is, where an e2e test does move it.
    answers: Cell<i32>,
    /// How many times the game has been made to tell its device the fog is off — see
    /// [`puts_the_fog_out`].
    fog_told: Cell<u32>,
    /// And how many times its own transition has put the screen a run ends on the chain, which is a screen
    /// somebody sees — see [`Fake::result_screens`].
    result_screens: Cell<u32>,
    /// The game's own pause menu's state and timer, `[menu+0x0]` and `[menu+0x4]` of 0x402780's object — see
    /// [`Fake::walks_its_pause_menu`]. Kept here and not laid out, nothing of orb's reading that object.
    pause_menu: Cell<(i32, i32)>,
    /// The word the player's own job found in the word the run acts on, which is what a frame is played with —
    /// see [`Fake::the_player_read`].
    the_player_read: Cell<u16>,
    /// And last frame's word beside it, as that job found it — see [`Fake::word_the_run_acted_on`].
    the_player_read_before: Cell<u16>,
    /// The seed the last build settled on for its stage — see [`Fake::settles_a_seed_for_the_stage`], which is
    /// where the next one comes from.
    seed: Cell<u16>,
    /// How many frames of a run being played back into place the sound was heard on — see
    /// [`Fake::frames_the_sound_was_heard_on_while_it_played_back`].
    heard_playing_back: Cell<u32>,
    /// The name every open of this game's score file landed in, in the order they happened — see
    /// [`create_file`].
    opens: RefCell<Vec<String>>,
    /// Every present the game's device was asked for — see [`device_present`].
    presents: RefCell<Vec<Presented>>,
    /// Whether orb stands in front of this game's pad read and in front of the read that read is inside,
    /// and the word that read last answered — see [`Fake::update`].
    reads_the_pad: bool,
    reads_the_keys: bool,
    /// The sound a track is streamed through, where an e2e test asked for one — see
    /// [`Fake::streams_its_song`].
    sound: RefCell<Option<Box<orb_sim::Sound>>>,
    /// Whether the panel its build lays has a sprite on the vm its background is drawn with — see
    /// [`Fake::has_no_panel_tile`].
    panel_tile: Cell<bool>,
    /// Whether a boss is fighting, which is what has its own timer moved on per frame — see
    /// [`Fake::the_boss_arrives`].
    boss: Cell<bool>,
    /// Whether a run of this game goes on by itself: its two clocks one frame further on per frame — see
    /// [`Fake::the_run_goes_on`].
    run_goes_on: Cell<bool>,
    /// Whether this game is slowing its own run down, and how many updates of one have gone by — see
    /// [`Fake::slows_its_own_run_down`], which is the update in [`SLOWED_EVERY`] that moves neither clock.
    slowed: Cell<bool>,
    updates_of_the_run: Cell<u32>,
    /// How long a stage of this game lasts before the next one is built, where an e2e test says.
    ///
    /// Nothing by default, which is a stage that never ends: every test about one stage wants that, and one
    /// about what happens *at* a stage's end says how long it is.
    stage_frames: Cell<Option<u32>>,
    /// The frame the fight this stage ends with is won on, where an e2e test says: the slot emptied, which
    /// is what a defeat is. Apart from the stage's own length because the two are apart in the game —
    /// th07 runs on for 281 to 496 frames after its boss goes down.
    fight_won_at: Cell<Option<u32>>,
    /// How many updates the ending's own script has left, and whether what it does then is leave its scene
    /// rather than hand over to the roll — see [`Fake::an_ending_is_running`].
    ending_updates: Cell<i32>,
    ending_leaves: Cell<bool>,
    /// And how many updates the roll's own script has left once the ending has handed over to it, where an
    /// e2e test says — see [`Fake::the_roll_lasts`].
    roll_updates: Cell<Option<i32>>,
    /// Whether the driver behind this game's device refuses to stretch a present — see
    /// [`Fake::refuses_to_stretch_on_a_present`].
    refuses_to_stretch: Cell<bool>,
    /// How many frames this launch has run, for the title menu that comes up on its own — see
    /// [`Fake::comes_up_at_its_title`].
    frames_since_launch: Cell<u32>,
    /// **Whether the run being played is one this game's own front end started**, which is the run that gets
    /// the stage's own script, its bullets and a player the word moves — see [`Fake::plays_its_own_stage`]. A
    /// run an e2e test placed with the helpers gets none of that, every such test driving its stage itself.
    on_its_own: Cell<bool>,
    /// Whether the shot select decided a run on this very update, which is the one frame a run has been chosen
    /// and not built — see [`Fake::the_run_is_chosen`] for why that frame is one of its own.
    just_chose: Cell<bool>,
    /// **Whether the front end got where it is by walking**, from Start on its own title: the shot select's
    /// decide starts a run only then. A shot select an e2e test placed is one that test drives — it chooses the
    /// run and builds the stage itself — and a game that started one under it would build the same stage twice.
    walking: Cell<bool>,
    /// Whether a decide at the shot select starts a run wherever that screen came from — see
    /// [`Fake::its_shot_select_starts_a_run_however_it_came_up`].
    starts_a_run_it_did_not_walk_to: Cell<bool>,
    /// Whether the ending now running is one a run cleared its way into, which is what the staff roll after
    /// it hands over to the screen a run's own end arrives on — see [`Fake::enters_the_ending`]. An ending an
    /// e2e test laid out itself hands over to the front end, that test being about the roll and not about
    /// what a run leaves behind.
    cleared: Cell<bool>,
    /// Whether the game's own records hold a clear, which is what its title offers `Extra Start` out of —
    /// see [`its_records_hold_no_clear`](Fake::its_records_hold_no_clear).
    clear_in_its_records: Cell<bool>,
    /// **Which item the title menu was last left on**, which is the item the front end comes back on when the
    /// run behind it is over — and, where that item practises a stage, the flow it comes back through. See
    /// `the_front_end_of_th07_walked`.
    ///
    /// Kept here because the cursor is one field for every screen of this front end,
    /// `FRONT_END_CURSOR`, and every screen between an item and a run writes its own list's item into it: the
    /// item the title was left on is gone by the time a run ends, and the real game reads it again there.
    left_the_title_on: Cell<i32>,
    /// The updates the front end has spent on its way down to the end of the game, once the title's last item
    /// was decided — see [`Fake::walks_its_front_end`].
    ending_the_game_for: Cell<Option<i32>>,
    /// And whether the front end is still walking back into the practice flow, which takes a screen a frame —
    /// see [`Fake::walks_back_into_the_practice_flow`].
    coming_back_through_the_practice_screens: Cell<bool>,
    /// The difficulty a stage is built on and the character the front end settled: [`DIFFICULTY`] and
    /// [`CHARACTER`] for a run an e2e test placed, and whatever the difficulty and character selects were left on
    /// for one the front end walked to.
    difficulty: Cell<i32>,
    character_chosen: Cell<u8>,
    /// The stage frame the script last acted on, the bullets that stage has out, the frames a hit has left before
    /// it is a death, and the frames a bomb has left to go off for — see [`Fake::plays_its_own_stage`].
    last_stage_frame: Cell<u32>,
    bullets: RefCell<Vec<Bullet>>,
    hit_window: Cell<i32>,
    bomb_left: Cell<u32>,
    /// And whether it stands in front of the read that screen makes — see [`Fake::reads_the_pad_buttons`].
    reads_the_pad_by_number: bool,
    word: Cell<u16>,
    /// Whether the last update was one the game skipped, which is 0x62627d — see [`Fake::update`] — and the word
    /// before the one the read left, 0x4b9e54, which that skip leaves where it was.
    read_held_over: Cell<bool>,
    read_before: Cell<u16>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory.
    _installed: orb_api::Installed,
}

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into — plain `extern` functions
    /// with nothing but the ABI's arguments, so where the real game would reach its own globals this
    /// reaches the game.
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

    fn frame(&self) -> i32 {
        if self.launch.own_frame_loop {
            unsafe { orb_core::runtime::render(self.frame_window()) }
        } else {
            self.own_render()
        }
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
    /// Lays the game out on a display an e2e test says the whole of, gives it a device, and attaches orb
    /// to it the way a launch there is attached: **orb's own frame loop, with the game's update and draw
    /// inside it.**
    ///
    /// `work` is what the game's own frame costs, which is what the clock moves by.
    ///
    /// Boxed and never moved for the reason th06's is: the hooks find it through a pointer.
    pub fn attach(display: Display, name: &str, work: Work) -> Box<Self> {
        Self::attach_configured(display, name, work, |_| ())
    }

    /// And the same launch with a setting of its own answered — the screen, for an e2e test about the
    /// shape a client of that shape gets the game in.
    ///
    /// A closure rather than a `Config`, so that what an e2e test says is the one thing it is about and
    /// everything else is what a launch in a directory with no `orb.yaml` brings.
    pub fn attach_configured(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, true, true, name, &[], work, settings)
    }

    /// And on a machine whose own windows are in the language `ui_language` names — `orb_sim::langid` — with
    /// nothing in `orb.yaml` about the language, so the answer comes from the only place left.
    pub fn attach_on_a_machine_in(
        ui_language: u16,
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(
            display,
            Some(ui_language),
            true,
            true,
            name,
            &[],
            work,
            settings,
        )
    }

    /// And on a machine that will not say where its panel is, which is a host with nothing for orb to lay a
    /// window out against: the window is then the one the game asked for, which is the window every launch
    /// before orb had.
    pub fn attach_without_a_monitor(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, false, true, name, &[], work, settings)
    }

    /// And laid out with **no device**, which is what a process is before its Direct3D setup has run: orb
    /// attaches there in a real launch and finds the device afterwards, through its hook over
    /// `GameWindow::InitD3dDevice` at 0x4356a0. A game that had one already is a game that hook is never
    /// reached in — see [`finds_its_device`](Fake::finds_its_device), which is the other half of this.
    pub fn attach_before_its_device(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, true, false, name, &[], work, settings)
    }

    /// And with files already beside the exe when orb is attached, which is what a sitting after another one
    /// finds: `left` is each file's name under the game's directory and what it holds.
    pub fn attach_finding(
        display: Display,
        name: &str,
        left: &[(&str, &str)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, true, true, name, left, work, settings)
    }

    /// Every host fact a launch is declared with, in one place: the wrappers above are what an e2e test calls,
    /// each naming the one it is about and passing the rest as what an ordinary launch has. So the argument
    /// count is the number of things a launch can be declared to be, which is what the lint is about.
    #[allow(clippy::too_many_arguments)]
    fn attach_declaring(
        display: Display,
        ui_language: Option<u16>,
        monitor: bool,
        device: bool,
        name: &str,
        left: &[(&str, &str)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let dir = scratch(name);
        // The simulated Windows first, `orb.yaml` being a file of orb's own and so read through the file
        // seam — the same order th06's has, and for the same reason.
        let image = Image::laid_out_seeded(display.seed);
        let installed = image.enter();
        // What the log says is read after what has been said is written — see th06's own attach, and
        // `orb_sim::Log::waits_first`.
        image.sim().log().waits_first(orb_core::log::written_out);
        // Before the attach, that being where orb settles which language its screens are in: a machine declared
        // after it is one nothing asked.
        if let Some(ui_language) = ui_language {
            image.sim().set_ui_language(ui_language);
        }
        image.sim().files().make(&dir);
        for (file, contents) in left {
            image.sim().files().put(dir.join(file), contents);
        }
        let mut config =
            Config::load_beside(&dir.join(EXE)).expect("a directory with no orb.yaml in it");
        // The memory hooks patch an import table there is none of.
        config.track_memory = false;
        settings(&mut config);
        // Kept before the attach takes the rest of it: what plays the run is installed after the attach — the
        // log being what a refused recipe has to speak through — and by then the config has moved.
        let recipe = config.autoplay.clone();
        // The font is there for the harness's own device and gone before orb reaches for one: **a th07
        // install has no `font.ttf` beside its exe.** th06 ships one and th07 keeps its fonts inside
        // `th07.dat`, so what orb draws here it draws in a face of the machine's own. A launch with a font
        // here would be a launch of the other game's directory.
        let font = dir.join("font.ttf");
        image.sim().text().install_font(&font);
        let launch = Launch::new(
            image.sim(),
            dir,
            display.seed,
            super::orbs_own_loop(&config, the_game_this_is()),
        );
        image.sim().text().remove_font(&font);
        // The device where the launch has one already, and a null where it is attached before the game's own
        // setup has made one — which is every real launch, and the state `finds_its_device` is the way out of.
        image.shows_through(
            if device {
                launch.device()
            } else {
                orb_api::Device::NULL
            },
            WINDOW,
        );
        // The three calls orb's frame makes into this game, which is what a game laid out by hand has to
        // hand over: code is the one thing an address space cannot hold.
        image.hands_over_the_frames_own_calls(
            empties_the_queue as *const () as usize,
            draws_the_queue as *const () as usize,
            puts_the_fog_out as *const () as usize,
        );
        // And every other call of the game's own that orb makes, for the same reason: code is the one thing
        // an address space cannot hold, and a launch that left one of these unanswered would jump into the
        // real game's addresses — which in a laid-out one is memory nothing has mapped. The read of the score
        // file is the one a mode chosen at the menu makes on the frame it is chosen, and the rest are a
        // chapter's: the seal after the numbers, the border the cherry+ earns, the two music calls, and the
        // baking a card's name goes back on the plate through.
        //
        // Answered here rather than by the e2e tests that reach them, because a launch is what has them: a
        // game whose own calls half work is not a game.
        let _ = image.hands_over_the_score_file();
        let _ = image.hands_over_the_seal();
        let _ = image.hands_over_the_border();
        let _ = image.hands_over_the_music();
        image.hands_over_the_text_baking();
        image.lays_out_its_score_files_image();
        image.its_play_records_are_tagged_at_its_start();
        // Where the game is installed, which is where orb writes its log.
        image.sim().set_host_exe(launch.dir().join(EXE));
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
        // And the monitor the window goes on, before orb is attached and for the reason the rate is: the
        // attach is where orb says this process reads sizes as real pixels. Declared for every launch here
        // rather than by the e2e tests that want one — th07 makes a window on any machine — except where one
        // asks for a host that will not say where its panel is, which is a window orb has nothing to lay out
        // against: see `the_windows_orb_leaves_alone_of_th07.rs`.
        if monitor {
            sim.windows()
                .set_monitor(orb_sim::Monitor::scaled(), orb_sim::Frame::SOME);
        }

        let fake = Box::new(Self {
            image,
            launch,
            answers: Cell::new(CHAIN_CARRIED_ON),
            fog_told: Cell::new(0),
            result_screens: Cell::new(0),
            pause_menu: Cell::new((PAUSE_MENU_SETTING_UP, 0)),
            the_player_read: Cell::new(0),
            the_player_read_before: Cell::new(0),
            // Not nothing, so the first stage of a launch is built out of a seed like every other one.
            seed: Cell::new(0x4d2f),
            heard_playing_back: Cell::new(0),
            opens: RefCell::new(Vec::new()),
            presents: RefCell::new(Vec::new()),
            reads_the_pad: the_game_this_is().hooks().joystick.is_some(),
            reads_the_keys: the_game_this_is().hooks().input.is_some(),
            reads_the_pad_by_number: the_game_this_is().hooks().pad_buttons.is_some(),
            run_goes_on: Cell::new(false),
            slowed: Cell::new(false),
            updates_of_the_run: Cell::new(0),
            stage_frames: Cell::new(None),
            fight_won_at: Cell::new(None),
            ending_updates: Cell::new(0),
            ending_leaves: Cell::new(false),
            roll_updates: Cell::new(None),
            refuses_to_stretch: Cell::new(false),
            frames_since_launch: Cell::new(0),
            on_its_own: Cell::new(false),
            just_chose: Cell::new(false),
            walking: Cell::new(false),
            starts_a_run_it_did_not_walk_to: Cell::new(false),
            cleared: Cell::new(false),
            clear_in_its_records: Cell::new(true),
            // The item that starts a run, which is where a launch of the real game reads the cursor before
            // anything has been left: `front end: screen 0, cursor on item 0` on the first frame of every
            // case's trace.
            left_the_title_on: Cell::new(TITLE_ITEM_START),
            ending_the_game_for: Cell::new(None),
            coming_back_through_the_practice_screens: Cell::new(false),
            difficulty: Cell::new(DIFFICULTY),
            character_chosen: Cell::new(CHARACTER),
            last_stage_frame: Cell::new(0),
            bullets: RefCell::new(Vec::new()),
            hit_window: Cell::new(0),
            bomb_left: Cell::new(0),
            sound: RefCell::new(None),
            panel_tile: Cell::new(true),
            boss: Cell::new(false),
            word: Cell::new(0),
            read_held_over: Cell::new(false),
            read_before: Cell::new(0),
            _installed: installed,
        });
        RUNNING.set(&raw const *fake);
        unsafe {
            orb_core::runtime::attach_to(
                the_game_this_is(),
                config,
                fake.image.data(),
                orb_core::runtime::Originals {
                    update,
                    draw,
                    input,
                    // A `fastcall` of one, which `runtime::stage_building_in_ecx` calls through as one — the
                    // field's own type being the one every other game's build has.
                    stage_building: std::mem::transmute::<
                        extern "fastcall" fn(i32) -> i32,
                        extern "C" fn(i32) -> i32,
                    >(stage_building),
                    stage_begun,
                    unlocks_read,
                    ranking_read: fake.image.its_score_file_gather(),
                    card_records_read: fake.image.its_card_record_parse(),
                    render: own_render,
                    play_sounds,
                    present,
                    create_window,
                    create_file,
                    stop_recording,
                    // Handed its instance in `ecx`, as `runtime::create_game_window_in_ecx` calls it.
                    create_game_window: std::mem::transmute::<
                        extern "fastcall" fn(*mut c_void) -> i32,
                        extern "C" fn(*mut c_void) -> i32,
                    >(create_game_window),
                    joystick_position,
                    pad_buttons,
                    save_replay,
                    init_d3d_device,
                    tops_up_the_music,
                    stops_the_music: orb_core::game::th07::image::stops_the_stream,
                },
            )
        };
        // **And the host's `SetActiveWindow` handed over where orb writes over its import entry** — see
        // [`Fake::activates_its_window`].
        unsafe {
            orb_core::window::answers_the_window_activated(
                host_activates_a_window as *const () as usize,
            )
        };
        // **Whoever is playing this run, installed where a real launch installs it**: `orb`'s attach reaches
        // `orb_bot::install` and nothing of orb-core does, so an e2e test over a bot has to reach it too — the
        // slot is what both fill, and a test that left it empty would be testing orb without one.
        //
        // After the attach, as a real launch's is: what a refused recipe has to say goes in the log, and the
        // log is not there to be written to until the attach has settled it.
        if let Some(recipe) = &recipe {
            unsafe { orb_bot::install(recipe, the_game_this_is()) };
        }
        fake.frame_takes(work);
        fake
    }

    /// This game's memory, for an e2e test that reads back what a frame of orb's did to it — the queue of
    /// quads and the fog, which are the whole of what a frame here touches.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// The game creating the window it plays in, which is one `CreateWindowExA` on the class orb
    /// recognises.
    ///
    /// Straight through orb's rewrite of that import rather than through the game's own creation, which orb
    /// also stands in front of: what that hook is for is the windowed flag, and a launch here asks for no
    /// window of its own. What orb does with the call is decide the window's shape and place — see
    /// `orb_core::window`.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        // `GameWindow::CreateGameWindow` first, which is the call everything about this game's window is decided
        // inside: the game reads its own configuration in there, and orb's one chance to overrule a display taken
        // exclusively is in front of that read — see `Th07::hooks`' `create_window`. Handed the instance in `ecx`.
        orb_core::runtime::create_game_window_in_ecx(std::ptr::null_mut());
        self.asks_for_a_window_of_the_class(WINDOW_CLASS)
    }

    /// **The game activating the window it plays in**, as 0x435e30 does straight after making it — th08's 0x4436a0
    /// over again: its input attached to the foreground window's thread (0x435e64), the foreground lock's timeout
    /// taken to nought (0x435e88), and `SetActiveWindow` on the window through the import at `0x48d1d0` (0x435e92) —
    /// which reaches orb's `orb_core::window::set_active_window` where orb wrote over that entry. What the host makes
    /// of it is `orb_sim`'s windows brought forward.
    pub fn activates_its_window(&self, window: orb_api::Hwnd) {
        unsafe { orb_core::window::set_active_window(window.0 as *mut c_void) };
    }

    /// And another window of its own, which is an ask orb has to hand straight on: the same call on a class that
    /// is not the one the game plays in.
    pub fn creates_another_window(&self) -> orb_api::Hwnd {
        self.asks_for_a_window_of_the_class(OTHER_WINDOW_CLASS)
    }

    /// The game answering `WM_SETCURSOR` the way it does, which is `ShowCursor(TRUE)` on every movement of the
    /// pointer over its window: straight through the import orb patched, since what that patch is for is exactly
    /// this call — see `the_mouse_pointer_over_th07.rs`.
    pub fn answers_wm_setcursor(&self) -> i32 {
        orb_core::mouse::show_cursor(1)
    }

    /// The ask itself, which is [`ASKED_STYLE`] at [`ASKED_AT`] sized [`ASKED_SIZE`]: nothing like any window orb
    /// lays out, so what the host was really asked for says by itself which of the two happened.
    fn asks_for_a_window_of_the_class(&self, class: &CStr) -> orb_api::Hwnd {
        let window = unsafe {
            orb_core::window::create_window_ex_a(
                0,
                class.as_ptr().cast(),
                c"th07".as_ptr().cast(),
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

    /// And the device it shows through, with the vtable orb redirects the `Present` slot of laid out under
    /// it — then orb's own device setup, where that redirection happens.
    ///
    /// The device object and its vtable are laid out here because orb reaches through them:
    /// `window::hook_device` reads the object for its vtable and swaps the slot, so a device that was only
    /// a pointer is a read of an address nothing has mapped. Which is also why this is a step of its own —
    /// the launch is attached before the game has a device, as a real one is — and for a launch that said it
    /// had none to begin with, see [`attach_before_its_device`](Fake::attach_before_its_device), this is where
    /// the pointer orb reads goes in as well.
    pub fn finds_its_device(&self) {
        let device = self.launch.device();
        let vtable = self.launch.vtable();
        let space = self.image.space();
        space.map(device.0, size_of::<usize>(), orb_api::Kind::Private);
        space.write::<usize>(device.0, vtable);
        space.map(vtable, super::DEVICE_VTABLE_BYTES, orb_api::Kind::Image);
        space.write_bytes(
            vtable,
            &self
                .launch
                .vtable_bytes(device_present as *const () as usize),
        );
        self.image.shows_through(device, WINDOW);
        // orb's own device setup only where a launch installs it over the game's, which is what
        // `Hooks::init_device` says: a game that declines it is one orb never gets in front of, so the
        // `Present` slot stays the one Direct3D put there and the frames are stretched over the client.
        // Calling it anyway would be this game driving a hook production does not install — the same
        // thing `orbs_own_loop` is for.
        if the_game_this_is().hooks().init_device.is_some() {
            orb_core::runtime::init_d3d_device();
        }
    }

    /// Every present the game's device has been asked for, in the order it was asked.
    pub fn presented(&self) -> Vec<Presented> {
        self.presents.borrow().clone()
    }

    pub fn forget_presents(&self) {
        self.presents.borrow_mut().clear();
    }

    /// Has this device's driver refuse to stretch on a present, which some do: the call that asks for a
    /// destination rectangle answers a failure and the one that asks for none is answered.
    ///
    /// An e2e test saying so, the way th06's does. What it is for is what orb does about it — the game's own
    /// call again, which leaves the run playable and stretched — since a run that stopped being presented at
    /// all would be a black window. The fallback is orb's and not a game's, so what asking it of this game adds
    /// is the ask th07 makes: four nulls, where th06's e2e tests hand over a source rectangle to see dropped.
    pub fn refuses_to_stretch_on_a_present(&self) {
        self.refuses_to_stretch.set(true);
    }

    /// How many frames have taken the fog off to the device, which is not the same as how many put the
    /// field out: see [`puts_the_fog_out`].
    pub fn fog_told_to_the_device(&self) -> u32 {
        self.fog_told.get()
    }

    /// The game opening its own score file, which is one `CreateFileA` on the name it asks for — and
    /// which th07 does at its front end and again on the way out, measured as three opens of `score.dat`
    /// in a measured launch.
    ///
    /// Answers the name the open landed in, which is orb's decision and not this game's.
    ///
    /// Through orb's rewrite where the launch installed one over the game's import entry, and straight
    /// into this game's own where it did not — a launch with nothing to fork patches nothing, and an
    /// unpatched entry is the one thing a game with no import table cannot have.
    pub fn opens_its_score_file(&self, write: bool) -> String {
        self.tries_to_open_its_score_file(write)
            .expect("an open of the score file this e2e test asked for")
    }

    /// And the same open where it may be refused, which is what a write is in a session whose run is not a
    /// score: `INVALID_HANDLE_VALUE` back, and the game left with nothing to write into.
    pub fn tries_to_open_its_score_file(&self, write: bool) -> Option<String> {
        let access = if write { GENERIC_WRITE } else { 0 };
        let open = if orb_core::score::installed() {
            orb_core::score::create_file_a
        } else {
            create_file
        };
        let handle = unsafe {
            open(
                SCORE_FILE.as_ptr().cast(),
                access,
                0,
                std::ptr::null(),
                0,
                0,
                std::ptr::null_mut(),
            )
        };
        if handle.is_null() || handle as isize == -1 {
            return None;
        }
        Some(self.opens.borrow()[handle as usize - 1].clone())
    }

    /// The read of the score file its front end makes for what its own menus offer, made where that screen
    /// makes it — inside the added callback orb brackets — and answering which of the two files it landed in.
    ///
    /// Through orb's hook only where a launch installs it, for the reason `orbs_own_loop` is: a launch without
    /// chapters has one file and nothing to bracket, so what its own read lands in is the game's file by there
    /// being no other.
    /// The front end's own added callback, made where that screen is built rather than where a test asks for it:
    /// the read itself, and nothing said about which file it landed in — see [`Fake::reads_what_its_menus_offer`]
    /// for the one that answers that.
    fn the_front_end_reads_the_file_for_its_menus(&self) {
        let front = self.image.front_end_object() as *mut c_void;
        if the_game_this_is().hooks().unlocks_read.is_some() {
            orb_core::runtime::unlocks_read_in_ecx(front);
        } else {
            unlocks_read(front);
        }
    }

    pub fn reads_what_its_menus_offer(&self) -> String {
        let front = self.image.front_end_object() as *mut c_void;
        let opened = self.opens.borrow().len();
        if the_game_this_is().hooks().unlocks_read.is_some() {
            orb_core::runtime::unlocks_read_in_ecx(front);
        } else {
            unlocks_read(front);
        }
        let opens = self.opens.borrow();
        assert_eq!(
            opens.len(),
            opened + 1,
            "the front end's own callback opened no score file",
        );
        opens[opens.len() - 1].clone()
    }

    /// The game's own key config read, made where that screen makes it: the array's address, with whatever
    /// the game found in it and whatever orb added.
    ///
    /// Through orb's hook only where a launch installs it, for the reason `orbs_own_loop` is: a game whose
    /// `hooks` declines that read is one orb is not in front of, and what its screen then walks is what its
    /// own read left — which here is an empty array.
    pub fn reads_the_pad_buttons(&self) -> usize {
        if self.reads_the_pad_by_number {
            orb_core::runtime::pad_buttons() as usize
        } else {
            pad_buttons() as usize
        }
    }

    /// Has its stage stream a track through a sound of the host's, with the sound now audible beginning
    /// `heard_at` bytes into `thbgm.dat`.
    ///
    /// A real buffer and a real pair of kernel32 functions, because orb reaches those by *calling* them rather
    /// than by reading them: th07 opens that one file itself and reads every track out of it, so what a
    /// position in it means is an offset in the archive — see `orb_core::audio::Track`.
    ///
    /// The countdown goes in with the position, they being the pair a loop is taken on: the track loops where
    /// it runs out, so what is left from here is the track's own length less how far in the file is.
    pub fn streams_its_song(&self, heard_at: i32) {
        let sound = orb_sim::Sound::of(
            (0..TRACK.bytes).map(|byte| byte as u8).collect(),
            STREAM_BUFFER,
            STREAM_NOTIFY,
        );
        sound.heard_at(heard_at);
        let left = TRACK.bytes - heard_at as u32;
        // The file's own records, the two names the stage's data carries, and the two slots its build loaded
        // out of them, before the stream: a track is started by naming a slot, and which slot is the stage's
        // music comes from those names.
        self.image
            .holds_its_music_file(&[TITLE_TRACK, TRACK, FIGHT_TRACK, SECOND_PHASE_TRACK]);
        self.image
            .names_its_songs(&named_in_the_table(TRACK), &named_in_the_table(FIGHT_TRACK));
        self.image.loads_a_slot(STAGE_SLOT, &loaded_as(TRACK));
        self.image.loads_a_slot(FIGHT_SLOT, &loaded_as(FIGHT_TRACK));
        self.image.streams_a_track(&sound, TRACK, left);
        *self.sound.borrow_mut() = Some(sound);
    }

    /// Another track playing, which is what a boss arriving leaves: the same file and the same buffer with
    /// another of `thbgm.dat`'s records on the source, so the track the chapter was taken under is gone.
    pub fn plays_another_song(&self) {
        let sound = self.sound.borrow();
        let sound = sound.as_ref().expect("a sound this e2e test asked for");
        self.image
            .streams_it_again(sound, FIGHT_TRACK, FIGHT_TRACK.bytes);
    }

    /// And **the stage taken into frames before its own song starts**, which is what a stage's first chapter
    /// falls in: the song of the scene before it — the front end's own, [`TITLE_TRACK`] — taken down at the
    /// transition, and the stage's own not asked for yet. So the stream, its source and its buffer are all
    /// still there, on that song's record, with nothing coming out of them.
    ///
    /// **Stopped rather than playing**, which is the whole of why this is its own call: a launch of th07 was
    /// watched taking a chapter here, and what is audible then is nothing. A laid-out game left playing lets an
    /// answer through that that launch heard fail — the stage's own song started early, and again from its
    /// beginning where the stage's script asked for it.
    ///
    /// The song is left in the slot the front end played it from, [`FRONT_END_SLOT`], the stage's build having
    /// loaded its own two names over slots 0 and 1: a track in no slot at all is a state this game is never in.
    pub fn took_down_the_song_it_came_in_with(&self, heard_at: i32) {
        self.streams_its_song(heard_at);
        self.image
            .loads_a_slot(FRONT_END_SLOT, &loaded_as(TITLE_TRACK));
        let sound = self.sound.borrow();
        let sound = sound.as_ref().expect("a sound this e2e test asked for");
        self.image
            .streams_it_again(sound, TITLE_TRACK, TITLE_TRACK.bytes);
        sound.stops();
    }

    /// And the stage's own song starting a few frames in, which is what replaces it: the same file and buffer
    /// with the stage's record on the source, as its ECL's own play of the slot the build loaded leaves it.
    pub fn its_stage_song_starts(&self) {
        let sound = self.sound.borrow();
        let sound = sound.as_ref().expect("a sound this e2e test asked for");
        self.image.streams_it_again(sound, TRACK, TRACK.bytes);
    }

    /// The screen a run ends at brought up from the title, which is the front end's own ranking screen: the screen on
    /// the chain with the phase that way up leaves, and its added callback — the same read of the score file a
    /// save makes — run where the chain would run it.
    ///
    /// **A read of the file that no write follows**, which is what makes it worth laying out: what a session
    /// counted must not be put back here, the records going nowhere afterwards and the note being spent.
    pub fn its_ranking_screen_is_brought_up(&self) {
        self.image.the_ranking_is_up();
        self.the_screen_a_run_ends_at_reads_the_file();
    }

    /// And how many times that screen has been put on the chain by the game's own transition, which is what a
    /// launch sees a frame or two of: its background, on the way out of a run — see
    /// [`Fake::follows_its_supervisor`].
    pub fn result_screens(&self) -> u32 {
        self.result_screens.get()
    }

    /// The word the player's own job found in the word the run acts on, which is **what the last frame was
    /// played with**: this game's own jobs read the keys at chain priority 0, move the player at 8 and copy the
    /// read's word into the run's at 0x10, so the word at 8 is the whole of what a frame's play is decided by.
    pub fn the_player_read(&self) -> u16 {
        self.the_player_read.get()
    }

    /// The read that screen's own build makes, which is the same read a save makes and the one the ranking screen
    /// shows what it shows out of.
    fn the_screen_a_run_ends_at_reads_the_file(&self) {
        let screen = self.image.the_screen_a_run_ends_at();
        // Which object this read is made on, the stand-in for it taking no `this` — and it is the whole of what
        // this step is about: the phase that screen carries is what says no write follows.
        self.image.its_next_read_is_made_on(screen);
        let from = self.opens_its_score_file(false);
        self.image.saves_nothing_reading(&from);
        let screen = screen as *mut c_void;
        if the_game_this_is().hooks().ranking_read.is_some() {
            unsafe { orb_core::runtime::ranking_read_in_ecx(screen) };
        } else {
            self.image.its_score_file_gather()(screen);
        }
    }

    /// Which tracks the game has been asked to start, as `thbgm.dat`'s own names for them: what a restore that
    /// took the music down put back, out of the record the play it asked for came out on.
    pub fn music_started(&self) -> Vec<String> {
        self.image.music_started()
    }

    /// And how many times it was asked to take the music down, which is the call a restore makes before it
    /// writes over the stream's own object.
    pub fn music_stops(&self) -> u32 {
        self.image.music_calls().0
    }

    /// `StreamingSound::ServiceBuffer`: one chunk of the file into the buffer at the offset the game keeps for
    /// it, with that offset and the countdown moved on by what was read — which is what the music thread's own
    /// refill does at 0x45db60.
    ///
    /// An e2e test saying the streaming thread ran, the way it says the player was hit: that thread is a
    /// thread, and what it would have done over a few seconds of a track is not something an e2e test can wait
    /// for.
    /// **Through orb's hook over it**, the way every other call of the game's own goes through the hook that
    /// stands in front of it: what that hook says is what the refill is about to feed the buffer, and a test
    /// that called the body instead would be a test of a launch with the hook left out.
    pub fn services_the_buffer(&self) {
        let (stream, _) = self
            .the_stream_its_player_names()
            .expect("a stream this e2e test asked for");
        orb_core::runtime::tops_up_the_music(stream, 1);
    }

    fn the_refill_itself(&self) {
        let at = self.image.next_write_offset();
        let read = self.with_the_sound(|sound| sound.tops_the_buffer_up(at));
        assert_eq!(
            read, STREAM_NOTIFY,
            "the stream was serviced past the end of the track's own sound",
        );
        self.image
            .set_next_write_offset((at + read) % STREAM_BUFFER as u32);
        self.image.set_bytes_left(self.image.bytes_left() - read);
    }

    /// What the stream's own countdown before the loop stands at, which is the number the game's read takes
    /// every byte it hands over off — see `Th07`'s `SOURCE_LEFT`.
    pub fn the_countdown(&self) -> u32 {
        self.image.bytes_left()
    }

    /// And the mixer playing what is in the buffer: the play cursor moved on by `bytes`.
    pub fn plays_the_buffer_on(&self, bytes: u32) {
        self.with_the_sound(|sound| sound.plays_on(bytes));
    }

    /// The whole of what the stream looks like from outside the game's memory, which is what a restore has to
    /// put back and what no snapshot of that memory holds.
    ///
    /// # Panics
    /// Where no e2e test asked for a sound.
    pub fn stream_now(&self) -> Stream {
        self.with_the_sound(|sound| Stream {
            buffered: sound.buffered(),
            play_cursor: sound.play_cursor(),
            position: sound.position(),
            playing: sound.playing(),
            looping: sound.looping(),
        })
    }

    /// The buffer playing without going round, which is the state a chapter of a resumed run of th07 was
    /// measured being taken in — see `orb_sim::Sound::plays_without_looping`.
    pub fn plays_without_looping(&self) {
        self.with_the_sound(orb_sim::Sound::plays_without_looping);
    }

    /// # Panics
    /// Where no e2e test asked for a sound, there being nothing for any of the above to be about.
    fn with_the_sound<T>(&self, body: impl FnOnce(&orb_sim::Sound) -> T) -> T {
        let sound = self.sound.borrow();
        body(sound.as_ref().expect("a sound this e2e test asked for"))
    }

    /// The boss of a fight arriving, which is a chapter of its own: the manager's own slot filled, with the
    /// attack it is on at its first frame.
    ///
    /// A *midboss* while the stage's own track is playing, which is what says which fight this is — the music
    /// and not what the script is doing, for the reason `Chapters::due` gives. `the_stages_own_boss_arrives`
    /// is the other one.
    pub fn the_boss_arrives(&self) {
        self.at_the_start_of_an_attack();
    }

    /// And the fight the stage ends with, which is the boss arriving *and* the game loading the boss's own
    /// track: th07 names two songs per stage and plays the second for that fight.
    pub fn the_stages_own_boss_arrives(&self) {
        self.plays_another_song();
        self.the_boss_arrives();
    }

    /// And its second phase, which is another song again: the one the game loads out of a name of its own — see
    /// [`SECOND_PHASE_TRACK`]. The fight carries on, so nothing about the boss changes here.
    ///
    /// **Through objects made somewhere else**, which is what a track change leaves: the stream and its source
    /// are deleted and allocated again, so the pair the player names is not the pair it named before.
    pub fn its_second_phase_begins(&self) {
        let sound = self.sound.borrow();
        let sound = sound.as_ref().expect("a sound this e2e test asked for");
        self.image
            .streams_it_again_elsewhere(sound, SECOND_PHASE_TRACK, SECOND_PHASE_TRACK.bytes);
    }

    /// Which stream the player names now and the countdown that one is on — see
    /// `Image::the_stream_its_player_names`.
    pub fn the_stream_its_player_names(&self) -> Option<(usize, u32)> {
        self.image.the_stream_its_player_names()
    }

    /// The fight moving on to its next attack, which the game says by putting the boss's own timer back to
    /// nothing — spell or not, that reset is the signal.
    pub fn the_attack_changes(&self) {
        self.at_the_start_of_an_attack();
    }

    /// The boss on the first frame of an attack, which is the one shape all three of those are: the manager's
    /// slot filled, the timer of the attack at nothing, and that timer moving on with every frame from here —
    /// a fight whose timer never went forward is one orb can never see move on.
    fn at_the_start_of_an_attack(&self) {
        self.image.a_boss_is_fighting(BOSS_LIFE, 0);
        self.boss.set(true);
    }

    /// And the boss beaten: the slot emptied, which is what the manager's own walk leaves.
    pub fn the_boss_is_beaten(&self) {
        self.image.no_boss_is_fighting();
        self.boss.set(false);
    }

    /// A bomb spent, which is what the player's own update leaves behind: one fewer on the count, and the
    /// flag orb reads a bomb off raised.
    pub fn a_bomb_goes_off(&self) {
        let state = self.state();
        self.image
            .has_lives_and_bombs(f32::from(state.lives), f32::from(state.bombs - 1));
        self.image.a_bomb_is_going_off();
    }

    /// And the bomb over, which is that flag lowered: the count stays where the bomb left it.
    pub fn the_bomb_is_over(&self) {
        self.image.no_bomb_is_going_off();
    }

    /// The card over, which is the flag the manager clears as the attack ends. The boss fights on: what a
    /// card ending says is that the fight has moved on to another attack or is already won, and neither is
    /// this.
    pub fn the_card_is_over(&self) {
        self.image.no_spellcard_is_running();
    }

    /// A card put up the way the game puts one up: the record named — which is the copy 0x40ffb4 makes — the
    /// attempt that start counts, the card declared on the boss, and its name baked onto the plate.
    ///
    /// The boss goes up with it where none is fighting yet, a card being an attack of a fight.
    pub fn a_card_starts(&self, card: usize, name: &str) {
        self.at_the_start_of_an_attack();
        self.image.the_card_is_named(card, name.as_bytes());
        self.image.a_card_is_being_attempted(card);
        self.image
            .the_spellcard_is(u32::try_from(card).expect("a card of the game's own"));
        self.image.the_game_names_the_card(card);
    }

    /// The player dying, which is the game counting one and taking the life: the count is what orb reads a
    /// death off, a deathbomb being a hit that never reaches one.
    pub fn dies(&self) {
        let state = self.state();
        self.image.has_died(f32::from(state.deaths as i16) + 1.0);
        self.image
            .has_lives_and_bombs(f32::from(state.lives - 1), f32::from(state.bombs));
        self.image.the_player_is_dying();
    }

    /// A Pointdevice mode run of this game, in progress and going on: the mode answered at its own title menu, a
    /// stage built with the numbers a run of it starts with, and the stage's first chapter taken.
    ///
    /// What every e2e test about a chapter of this game needs before it can be about anything else, and it is
    /// the same sequence `a_pointdevice_run_of_th07.rs` walks a step at a time — that one is *about* the
    /// sequence, so it says each step out loud where the others ask for it in one line.
    ///
    /// # Panics
    /// Naming whichever step the run did not get past.
    pub fn in_a_pointdevice_run(&self) {
        self.the_mode_is_answered_at_the_title_menu();
        self.the_run_is_chosen();
        self.builds_a_stage();
        self.takes_the_stages_first_chapter();
    }

    /// The one frame a run has been chosen and not built: the front end has asked for the run scene and the
    /// supervisor has not acted on it — see `Th07::run_chosen`.
    ///
    /// **A frame of its own and not a state the build carries in**, which is what it was: the build sets the
    /// same pair, but nothing of orb's looks between the two, so a run that only ever passed through it
    /// *inside* a build was a run orb was never handed. Which is the one frame two things happen on — a
    /// chapter pointed at the stage about to be built, and the replay this run writes into settled — so a
    /// laid-out game that skipped it was a game where neither could.
    fn the_run_is_chosen(&self) {
        self.image.a_run_has_been_chosen();
        self.frame();
    }

    /// And one stage of it practised rather than the whole game played, which is the same walk with the bit
    /// this game's own title menu sets for that item: all three of the items that start something are runs, so
    /// the question over them is the one question, and the chapters after it are a stage's like any other's.
    ///
    /// # Panics
    /// Naming whichever step the run did not get past.
    pub fn practises_a_stage(&self) {
        self.the_mode_is_answered_at_the_title_menu();
        self.image.is_practising();
        self.builds_a_stage();
        self.takes_the_stages_first_chapter();
    }

    /// And in a legacy-mode run of it, which is the same walk with the question answered the other way and no
    /// chapter at the end of it: the item the question's cursor starts on is Pointdevice mode, so the other mode is
    /// one press down.
    ///
    /// # Panics
    /// Naming whichever step the run did not get past.
    pub fn in_a_legacy_run(&self) {
        self.the_mode_is_answered_at_the_title_menu_as(Mode::Normal);
        self.builds_a_stage();
    }

    /// The title menu up with the cursor on an item that starts something, and orb's own question over it
    /// answered on the keyboard: the overlay first, which is orb's own rule about asking anything.
    fn the_mode_is_answered_at_the_title_menu(&self) {
        self.the_mode_is_answered_at_the_title_menu_as(Mode::Pointdevice);
    }

    fn the_mode_is_answered_at_the_title_menu_as(&self, mode: Mode) {
        let log = self.log();
        self.frames_until("an overlay", 8, || log.said("overlay: ready"));
        self.image.the_front_end_is_up();
        // The read that screen's own build makes, which is where this game reads its file for what its menus
        // offer — and lays that file's card records over the ones in memory while it is there.
        self.the_front_end_reads_the_file_for_its_menus();
        self.answers_the_mode(mode);
    }

    /// And the same at the title menu a run comes *back* to, which the game builds again rather than
    /// allocating a second front end: what an address space has one of, it has one of.
    fn the_mode_is_answered_at_the_title_menu_again(&self) {
        self.image.the_front_end_is_up_again();
        self.the_front_end_reads_the_file_for_its_menus();
        self.answers_the_mode(Mode::Pointdevice);
    }

    /// Held against what the log has said since this was entered rather than against the whole of it, because
    /// a launch that starts a second run asks the same question again: a line already there is a press this
    /// would never make.
    ///
    /// The other mode is one press down after the frames the question reads nothing over, a direction being a key
    /// that cannot be pressed again for nothing — see [`READS_KEYS_AFTER`].
    fn answers_the_mode(&self, mode: Mode) {
        let log = self.log();
        let from = log.written();
        self.image.the_title_menu_points_at_a_run();
        self.image.the_screen_has_been_up_for(SETTLED);
        // A frame with the cursor there before the press: whether orb holds a press back for a question is
        // decided in the frame hook and read by the input hook on the frame *after*, so a press on the frame the
        // cursor arrives on is one the game acts on itself — and this game's title menu now acts on it, which
        // is Start's flow walked and no title left to ask over.
        self.frame();
        self.press_until(keys::Z, "the mode question", || {
            log.said_since(from, "menu: Run is under the cursor, asking which mode")
        });
        if mode == Mode::Normal {
            self.frames(READS_KEYS_AFTER);
            self.press(keys::DOWN);
        }
        self.press_until(keys::Z, "the mode question answered", || {
            log.said_since(from, "mode: answered on the keyboard")
        });
    }

    /// The run given up at orb's own retry menu, which is the way out that leaves the chapter behind for the
    /// next launch to offer: タイトルに戻る is one press up from the item the cursor starts on, and the question
    /// it asks before it acts is answered はい one press up from the いいえ its own cursor starts on.
    ///
    /// The game is then where its own quit takes it. Which is two scenes — the run's teardown asks for the one
    /// a run ends in and the front end comes after it, and what these e2e tests want of the pair is the second:
    /// `the_screen_a_finished_run_ends_at_of_th07.rs` is where the first is the subject.
    ///
    /// # Panics
    /// Naming whichever step the give-up did not get past.
    pub fn gives_the_run_up(&self) {
        let log = self.log();
        let from = log.written();
        self.dies();
        self.frames_until("the death seen", 8, || {
            log.said_since(from, "died in chapter")
        });
        self.frames(READS_KEYS_AFTER);
        self.press(keys::UP);
        self.press_until(keys::Z, "the give-up asking", || {
            log.said_since(from, "retry: asking about the run given up")
        });
        self.frames(READS_KEYS_AFTER);
        self.press(keys::UP);
        self.press_until(keys::Z, "the run given up", || {
            log.said_since(from, "retry: the run is given up")
        });
        self.frames_until("the front end the run went back to", 600, || {
            self.image.the_scene_now() == self.image.scene_of_the_front_end()
                && self.image.the_scene_now() == self.image.the_scene_wanted()
        });
        // **And the run's own music gone with it**, which is what the teardown does: the stream the run was
        // playing through is deleted at 0x44cebd and the player left naming nothing. A laid-out game that kept
        // the song playing across a run being given up is one where the next run's music was already sounding
        // before that run had a stage.
        self.image.plays_no_track();
        orb_sim::Sound::taken_down();
        self.the_mode_is_answered_at_the_title_menu_again();
    }

    /// And the run left behind picked up again: the shot select this run is under the cursor of, orb's own
    /// question over it answered つづきから, the stage that answer points the build at, and the buttons written
    /// down played into it.
    ///
    /// The title menu is *not* walked again here — [`Fake::gives_the_run_up`] leaves the game standing on it
    /// with the mode answered, which is where a hand that has just given a run up is.
    ///
    /// # Panics
    /// Naming whichever step the resume did not get past.
    pub fn picks_the_run_up(&self) {
        self.picks_the_run_up_at(FIRST_STAGE);
    }

    /// And the same at a stage other than the one a run starts on, which is what a file written past the first
    /// asks the build for: the numbers a stage starts with are read where that stage starts, so a run put back
    /// with a *second* stage's is a run whose file was written in one.
    pub fn picks_the_run_up_at(&self, stage: i32) {
        self.picks_the_run_up_holding(stage, None);
    }

    /// And the same pick-up landing with **the front end's own song on the stream**, which is the state a
    /// stage's first chapter is really picked up into: that chapter falls frames into the stage, the build has
    /// loaded the stage's two slots and its script has asked for neither, so what the stream holds is the
    /// record of the track the front end was playing — the file's first. A chapter written down while the
    /// stage's own song played holds a position in *that* track, and the two are not the same wav.
    pub fn picks_the_run_up_with_the_front_ends_song_on(&self, stage: i32) {
        self.picks_the_run_up_holding(stage, Some(TITLE_TRACK));
    }

    fn picks_the_run_up_holding(&self, stage: i32, track: Option<Track>) {
        let log = self.log();
        let from = log.written();
        self.at_the_shot_select();
        self.press_until(keys::Z, "the question about the run left behind", || {
            log.said_since(from, "asking where to start")
        });
        self.press_until(keys::Z, "つづきから answered", || {
            log.said_since(
                from,
                "resume: from where it stopped, answered on the keyboard",
            )
        });
        // The press that answered it is handed back to the shot select, and it is that screen's own decide
        // which chooses the run: the frame the front end asks for the run scene is the one frame the game will
        // take a stage for a run it has not built yet — see `Th07::run_chosen`.
        self.image.a_run_has_been_chosen();
        self.frames_until("the stage the run was left in asked for", 8, || {
            log.said_since(from, "the run is being built at that stage")
        });
        self.builds_the_stage(stage);
        // **And the stage's own song playing, which is what that build leaves**: it loads its two slots and asks
        // for the first of them, and by the stage's first frame the play has been carried out. Said here rather
        // than in the build itself, because a stage is also built for a chapter that was taken before its song
        // began — see [`Fake::took_down_the_song_it_came_in_with`] — and this is the launch where the song is
        // what a run comes back to.
        if let Some(sound) = self.sound.borrow().as_ref() {
            self.image.names_the_stream_it_had();
            // **The record left as the run left it where this is the stage's own song**, and written where it
            // is another track's: what `names_the_stream_it_had` puts back is the word naming the stream, and
            // the record under it is still the one the stage was playing through.
            if let Some(track) = track {
                self.image.streams_it_again(sound, track, track.bytes);
            }
            orb_sim::Sound::started_again();
        }
        // **And what was audible on the way**, which is the whole of what somebody waiting for a run to come
        // back hears: the stage's own song was started by the build a moment ago, so a playback that leaves it
        // alone plays the stage's opening over and over the seconds the buttons take — a th07 session heard
        // exactly that, and then the song jump to the chapter's place when the run landed.
        self.heard_playing_back.set(0);
        self.frames_until("the run played back into place", 600, || {
            // The landing asked about first, because the frame it happens on is the frame the song is put back
            // on: counted as sound heard on the way, the answer would be yes however quiet the playback was.
            if log.said_since(from, "resume: the landing is") {
                return true;
            }
            if self
                .sound
                .borrow()
                .as_ref()
                .is_some_and(|sound| sound.playing())
            {
                self.heard_playing_back
                    .set(self.heard_playing_back.get() + 1);
            }
            false
        });
    }

    /// How many frames of the run being played back into place the sound was heard on — see
    /// [`Fake::picks_the_run_up`], which is where it is watched for.
    pub fn frames_the_sound_was_heard_on_while_it_played_back(&self) -> u32 {
        self.heard_playing_back.get()
    }

    /// The shot select up with this run under its cursor, and long enough up for a press: the screen orb asks
    /// about a run left unfinished over, and the screen whose own decide chooses the run.
    pub fn at_the_shot_select(&self) {
        self.at_the_shot_select_of_the_shot(SHOT);
    }

    /// And with another shot under it, which is another run with a file of its own — the buttons written down
    /// are the shot's.
    pub fn at_the_shot_select_of_the_shot(&self, shot: u8) {
        self.image.the_shot_select_of_a_run_is_up(
            u8::try_from(DIFFICULTY).expect("a difficulty of the game's own"),
            CHARACTER,
            shot,
        );
        self.image.the_screen_has_been_up_for(SETTLED);
        // A screen an e2e test placed rather than one this game walked to: its decide is the test's to act on —
        // see [`Fake::walks_its_front_end`] — and a frame with the run under the cursor goes before any press,
        // orb deciding what to hold back at the end of an update and acting on it in the next.
        self.walking.set(false);
        self.frame();
    }

    /// A launch taken as far as a run gets before the press that starts one: the mode answered at the title
    /// menu, and the shot select up under this run.
    ///
    /// Which is where the file behind orb's own question is read, so it is where an e2e test about that file
    /// stops — `Fake::picks_the_run_up` is this walk with the question answered and the stage built.
    ///
    /// # Panics
    /// Naming whichever step the launch did not get past.
    pub fn up_to_the_shot_select(&self) {
        self.the_mode_is_answered_at_the_title_menu();
        self.at_the_shot_select();
    }

    /// And the same with another shot under the cursor there.
    ///
    /// # Panics
    /// Naming whichever step the launch did not get past.
    pub fn up_to_the_shot_select_of_the_shot(&self, shot: u8) {
        self.the_mode_is_answered_at_the_title_menu();
        self.at_the_shot_select_of_the_shot(shot);
    }

    fn takes_the_stages_first_chapter(&self) {
        let log = self.log();
        self.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
    }

    /// The stage a run starts on, as the game's own build leaves it: the scene, the stage, the numbers behind
    /// the pointer that build allocates, the panel over them, the added callback the game ends that build with,
    /// and the run going on from there.
    ///
    /// The callback last, and on the manager the game's own chain would call it on: what orb does there is
    /// read the numbers this build has just put in place — so a stage whose callback ran first is a stage
    /// written down with the numbers of the one before it.
    pub fn builds_a_stage(&self) {
        self.builds_the_stage(FIRST_STAGE);
    }

    /// And a stage of this game other than the one a run starts on, which is what a resume asking for one and
    /// getting another is: the same build, and orb's own 0-based number for the stage.
    pub fn builds_the_stage(&self, stage: i32) {
        self.lays_the_stage_out(stage);
        self.image.has_a_runs_numbers();
        // **And the rank, out of the table by difficulty**, which 0x42e3a4 does inside the block 0x42e89b jumps
        // over: a run's first stage gets the table's starting figure and a stage the run walks into carries
        // whatever the stage before it left. A laid-out game that left this alone would be a game where
        // a resume gets the recorded number for free, which is the difference two launches measured — 0x20
        // carried against 0x10 built.
        self.image.the_rank_is(A_BUILDS_OWN_RANK);
        self.image.has_lives_and_bombs(LIVES, BOMBS);
        self.image.shot_is_at_power(POWER);
        self.image.the_run_has_scored(SCORE, 0);
        // And no deaths, these being the numbers a run *starts* with: the count is the run's own rather than the
        // stage's, so a launch that starts a second run and carried the first's deaths into it would be a run
        // orb reads as one somebody has already lost lives in.
        self.image.has_died(0.0);
        // **And the scene the supervisor came from, still, while the build runs.** 0x437f1d calls the build from
        // the branch for the scene being *left* — the front end's — and the write of `scene = wanted` is the
        // update's own tail at 0x4382c0, after that call returns. So everything the build reaches, orb's hook over
        // the added callback included, reads the old scene with the run's asked for; a laid-out game that had
        // arrived already is one no launch can disagree with, and a th07 session found that out: a run picked up
        // out of its file came up with the game's own numbers — `resume: not put back` — because `in_run` was read
        // off a scene the supervisor had not written yet.
        self.image.is_in_scene(
            self.image.scene_of_the_front_end(),
            self.image.scene_of_a_run(),
        );
        self.settles_a_seed_for_the_stage(true);
        self.builds_the_stage_out_of_that_seed(stage);
        self.the_stages_numbers_are_in_place();
        self.image
            .is_in_scene(self.image.scene_of_a_run(), self.image.scene_of_a_run());
        self.the_run_goes_on();
    }

    /// **The seed the build settles on for the stage it is about to lay out**, which the game takes out of
    /// wherever the generator had got to: where a replay is playing, 0x42f07e saves it, 0x42f08b seals the run's
    /// numbers over the top of it — two draws — and 0x42f094 puts it back, and on every build 0x42f0a4 writes it
    /// down as the stage's own.
    ///
    /// Another number every build, the way the game's is: the generator has moved on by then, so a stage built a
    /// second time is a different stage unless somebody puts the first one's seed back. What the step itself is
    /// does not matter — orb only ever holds this number against another of them — so it is a step of this file's
    /// own, said here rather than pretending to be the game's arithmetic.
    fn settles_a_seed_for_the_stage(&self, the_runs_first: bool) {
        let next = self.seed.get().rotate_left(3).wrapping_add(0x2b1d) | 1;
        self.seed.set(next);
        self.image.generator_is_at(next);
        // **The run's own eight numbers drawn out of it first, and only where the run's numbers are filled in at
        // all**, which is the order the game's own callback has them in: 0x42e951 draws them near its head,
        // 0x42f0a4 writes the stage's seed down after that, and the call to `STAGE_BUILD` — the one moment a
        // resume writes a seed — is at 0x42f0b4, after both. So these eight come off a word no resume can put
        // back, and a stage built a second time draws eight others.
        //
        // **The stage a run walks into draws none of them**: 0x42e89b jumps to 0x42eee6, which is past this call
        // as much as it is past the lives and the power, so the eight and the two counts the same block puts back
        // to nothing are the first stage's alone. The seed below is the other side of that — 0x42f0a4 and
        // 0x42f0b4 stand after the landing, so both ways in reach them.
        if the_runs_first {
            self.image.the_stage_draws_the_runs_numbers();
        }
        // And the seed the stage is written down as having begun with, which is the generator as it stands once
        // those draws have moved it — the number 0x42f0a4 copies, and so the number a resume puts back.
        self.image
            .the_stage_began_with_the_seed(self.image.the_generator_now());
    }

    /// And the stage laid out **out of that seed**, which is what makes the seed matter: the frames of the stage
    /// run from where the build's own draws left the generator, so a build from another seed is a stage whose
    /// every frame draws differently.
    ///
    /// `stage_building` is where a resumed run's seed goes in, and it goes in *here* — before these draws and
    /// after the seed above is settled. A game that declines that hook gets the draws with its own seed, which is
    /// what a th07 launch did: a run picked up out of its file parted from the run at the stage's first frame,
    /// its generator still holding the seed orb wrote at the end of the build where the run had it already drawn
    /// from.
    fn builds_the_stage_out_of_that_seed(&self, stage: i32) {
        // **Entered as the supervisor enters the build**, the stage in `ecx` and nothing on the stack (0x42f0ae) —
        // which is the hook `Building::InEcx` installs.
        if the_game_this_is().hooks().stage_building.is_some() {
            orb_core::runtime::stage_building_in_ecx(stage);
        }
        // The draws themselves, as one step: what a test asks of them is that the number the stage runs from is a
        // function of the seed the build had, and not the seed itself.
        let drawn = self.image.the_generator_now();
        self.image
            .generator_is_at(drawn.rotate_left(5) ^ 0x9e37 | 1);
    }

    /// The stage a run *moves on* to, which is the one frame a run spends between two stages and the build
    /// inside it: the stage's own fields, and the run's numbers left exactly as the stage before it left them.
    ///
    /// **The game's own added callback is where that difference is.** 0x42e894 compares the scene the supervisor
    /// has been asked for against the rebuild scene and jumps to 0x42eee6 where it is that one — and what it
    /// jumps over is the whole block that fills a run's numbers in: the lives at 0x42ea1c, the power at 0x42ea51
    /// and the deaths at 0x42eb08 are all inside it. So a run is one set of numbers walked through its stages,
    /// and a transition that put them back would hand a run six fresh lives.
    ///
    /// **All of it inside one update**, which is where the supervisor does it: the stage's end at 0x427ea5 has
    /// written the rebuild scene into the one *asked for*, and the supervisor's case for it tears the stage down
    /// through 0x42f45d at 0x438123, registers the manager for the next one at 0x438128 — which is where the added
    /// callback runs and reads that field — and puts it back to the run's own scene at 0x43813c before the update
    /// returns. So no frame of a transition is a frame outside the scene a run is played
    /// in, and a laid-out game that spent one would be a run orb reads as over.
    /// **And the last thing that block does is zero the input words**, at 0x438294 — 0x4b9e5c cleared, copied to
    /// 0x4b9e54 and from there to 0x4b9e4c — which every transition reaches, 0x4381cc jumping straight to it. So
    /// the scene just built is updated with nothing held, and a button still down on the frame after reads as a
    /// fresh press rather than as one already spent.
    pub fn moves_on_to_the_stage(&self, stage: i32) {
        self.image.the_next_stage_is_being_asked_for();
        self.image.the_stage_being_rebuilt_is(stage);
        self.the_stages_own_fields();
        // **The build itself, which a transition runs as much as a run's first stage does.** What 0x42e894 jumps
        // over is the block filling a run's numbers in and nothing after it: the jump lands at 0x42eee6, and the
        // seed the stage is written down as at 0x42f0a4 and built out of at 0x42f0b4 — settled first at 0x42f07e
        // where a replay is playing — are all past that. So the stage a run walks into is a stage `stage_building` is reached for, with the scene
        // *asked for* still the rebuild one — which is how orb tells the two ways in apart.
        self.settles_a_seed_for_the_stage(false);
        self.builds_the_stage_out_of_that_seed(stage);
        self.the_stages_numbers_are_in_place();
        self.image.the_read_left_the_word(0);
        // And both of the run's words, which 0x443aa0 zeroes on every stage build.
        self.image.the_run_acts_on_the_word(0);
        self.image.the_run_acted_on_the_word(0);
        self.image.is_playing(stage, self.difficulty.get());
    }

    /// What both of those write, which is the stage's own and nothing of the run's: the scene, the stage, its two
    /// clocks at nothing, a player playing, no fight, and the panel over them.
    fn lays_the_stage_out(&self, stage: i32) {
        self.image.is_playing(stage, self.difficulty.get());
        self.the_stages_own_fields();
    }

    fn the_stages_own_fields(&self) {
        // Both clocks at nothing and the player's own register run again, which is what a build leaves however the
        // stage before it ended: the frames a stage has run are that stage's own, and a run started again out of a
        // file plays its buttons in from the first of them. What that register leaves the player as — playing,
        // with nothing on the timer under it — is `the_player_a_stage_of_th07_starts`.
        self.image.has_been_in_the_stage_for(0);
        self.image.the_script_has_run_for(0);
        self.image.its_player_is_registered_again();
        // And no fight, which is the same thing said of the enemies: a boss is something a stage's script puts
        // up, so a build that left one standing would be a stage beginning half way through a fight.
        self.image.no_boss_is_fighting();
        self.image.no_spellcard_is_running();
        self.boss.set(false);
        self.image.has_a_panel();
        if !self.panel_tile.get() {
            self.image.the_panels_tile_has_no_sprite();
        }
        self.image.paints_its_panel();
    }

    /// `GameManager::AddedCallback`, which is the moment a stage's numbers are in place and that stage has not
    /// been updated yet: the frame a run is written down from, and the frame a resumed run's own numbers go in
    /// on.
    ///
    /// Entered in `ecx` where the launch installed it that way, which is how this game's chain calls every
    /// callback — the same shape [`Fake::reads_what_its_menus_offer`] hands the front end's own over in.
    pub fn the_stages_numbers_are_in_place(&self) {
        let manager = self.image.game_manager_object() as *mut c_void;
        if the_game_this_is().hooks().stage_begun.is_some() {
            orb_core::runtime::stage_begun_in_ecx(manager);
        } else {
            stage_begun(manager);
        }
    }

    /// The screen a run's own end arrives on, brought up the way that end brings it: the scene a run's teardown
    /// goes to, the screen on the chain in the phase a whole run reaches it in, and that screen's own read of the
    /// file — which is what takes a practice run's end on to the replay offer.
    ///
    /// The scene is asked for as well as arrived at, because a scene with another asked for is one this game
    /// moves on from — see [`Fake::follows_its_supervisor`]. What moves this screen on is its own walk, which is
    /// [`Fake::the_screen_offers_a_replay`] here.
    pub fn the_run_ends_at_its_own_screen(&self) {
        let over = self.image.scene_of_a_run_over();
        self.image.is_in_scene(over, over);
        self.image.the_screen_a_run_ends_at_is_up();
        self.the_screen_a_run_ends_at_reads_the_file();
    }

    /// And that screen's own walk reaching the question about saving a replay, which is the one state of it orb
    /// writes: a run with chapters has nothing to put in a replay.
    pub fn the_screen_offers_a_replay(&self) {
        self.image.the_screen_a_run_ends_at_offers_a_replay();
    }

    /// An ending running, as the game's own register leaves it: the scene, the object with the script it is
    /// reading, and the chain element carrying that object — which is the only place its address is kept, so the
    /// walk for it is the whole of how orb finds one.
    ///
    /// `updates` is how many of its own updates that script has left before the ending reads the *roll's* file
    /// over it, which is the one signal that says where a part of an ending ends: the flag stays set and the
    /// scene stays the ending's through the roll. The countdown runs inside [`Fake::update`], because the frames
    /// it is counted in are the ones orb runs itself while it takes the ending out of a frame's way.
    ///
    /// The roll's own track goes in with the handover where an e2e test asked for a sound, the roll starting one
    /// of its own — a second thing saying this is where one ends and the other begins.
    pub fn an_ending_is_running(&self, updates: i32) {
        self.image.an_ending_is_running(ENDING_SCRIPT);
        self.ending_updates.set(updates);
    }

    /// And an ending that leaves its scene rather than handing over, which is the other way orb's own run-out of
    /// one stops: after `updates` of its own the scene under it changes, and what follows an ending is reached and
    /// then left alone.
    pub fn an_ending_that_leaves_its_scene_is_running(&self, updates: i32) {
        self.ending_leaves.set(true);
        self.an_ending_is_running(updates);
    }

    /// And how many updates the roll's own script runs once the ending has handed over to it, after which the
    /// ending's scene is left for the front end's.
    ///
    /// Nothing by default, which is a roll that plays for ever: every e2e test about the handover wants that,
    /// and one about the roll being *played* says how long it is. **Which scene th07's roll hands over to is
    /// not read here** — what such a test is about is the frames of the roll and that they are the game's to
    /// play, and the scene after them is the one signal an e2e test has that the roll is over.
    pub fn the_roll_lasts(&self, updates: i32) {
        self.roll_updates.set(Some(updates));
    }

    /// The scene a run is played in entered with nothing of a run built in it, which is what a stage the game
    /// cannot load looks like from outside: the frame a run was chosen is over, so nothing is asking for one
    /// any more, and no stage ever arrives for a resume to be put back into.
    pub fn the_stage_never_comes_up(&self) {
        let run = self.image.scene_of_a_run();
        self.image.is_in_scene(run, run);
    }

    /// A replay being watched rather than a run being played: the bit the game sets as it loads one, and then
    /// the same stage a run gets.
    ///
    /// Which is a run — it has stages, and chapters could be taken over it — and not one orb acts on, there
    /// being nobody there to answer a retry menu.
    pub fn watches_a_replay(&self) {
        self.image.is_watching_a_replay();
        self.builds_a_stage();
    }

    /// Has its build leave no sprite on the vm the panel's background is drawn with, which is what every
    /// frame before that build really is: orb finds no tile then, and the strips either side of the mark over
    /// the lives are painted flat.
    ///
    /// Before the run rather than during it, because the line orb says about which of the two it got is said
    /// once: a tile taken away after the mark has been drawn is a launch that already found one.
    pub fn has_no_panel_tile(&self) {
        self.panel_tile.set(false);
    }

    /// How long each of this game's stages lasts before the next one is built.
    ///
    /// Nothing by default, which is a stage that never ends. **What ends one here is what ends one in
    /// th07: a single update**, the transition being the supervisor's own whole job — see
    /// [`Fake::moves_on_to_the_stage`] — so no frame of it is a frame outside the scene a run is played in
    /// and nothing watching the scene sees a stage end at all.
    pub fn stages_last(&self, frames: u32) {
        self.stage_frames.set(Some(frames));
    }

    /// And the frame the fight the stage ends with is won on, which is the slot emptied.
    ///
    /// Beside [`stages_last`](Fake::stages_last) rather than inside it, because the two are apart in the
    /// game: th07 runs on for 281 to 496 frames after its boss goes down — its death, the items it
    /// leaves and whatever is said over them — and what a step forward has to stop at is the first of the
    /// two.
    pub fn the_fight_is_won_at(&self, frames: u32) {
        self.fight_won_at.set(Some(frames));
    }

    /// Has the run go on by itself from here: its two clocks one frame further on with every frame, which is
    /// what a stage does and what the mode is read off — see [`Fake::update`].
    ///
    /// For an e2e test about a run rather than about a field: the ones that write a number and read the state
    /// line back want the number they wrote, so a laid-out run stands still until this says otherwise.
    pub fn the_run_goes_on(&self) {
        self.run_goes_on.set(true);
    }

    /// **A decide at the shot select starts a run whatever brought that screen up**, which is what the real
    /// screen does: the title's Start walks to it, and orb puts it back up on a run's own shot to ask the
    /// front end for one — see `Th07::front_end_starts_a_run`, which is how the attempt out of a replay gets
    /// a run to be watched in.
    ///
    /// Asked for rather than always done, the way the stage's own fire is: every other e2e test that places
    /// that screen drives it itself — it chooses the run and builds the stage — and a game that started one
    /// under it would build the same stage twice.
    /// **The game's own records holding no clear**, which is what a copy whose records were taken aside has:
    /// the title menu stops resting its cursor on `Extra Start`, the item it is not offering.
    ///
    /// A flag and not a record, this game keeping no `clrd` of its own — what its front end's own callback
    /// does here is make the read, the file it lands in being what an e2e test about that read asks about.
    ///
    /// **Settable rather than the default**, which is a game that has been cleared: every e2e test that
    /// walks to the Extra needs one, and what needs this is `the_cases_the_real_game_is_tested_with`, whose
    /// launches start from records `cargo xtask real` has taken aside.
    pub fn its_records_hold_no_clear(&self) {
        self.clear_in_its_records.set(false);
    }

    pub fn its_shot_select_starts_a_run_however_it_came_up(&self) {
        self.starts_a_run_it_did_not_walk_to.set(true);
    }

    /// **And this game slowing its own run down, which is what makes an update of it not a frame of it.**
    /// Where the run's header has its `+0x25` set, the manager's update divides its own count of them by five,
    /// four or three — by whether the live bullets at `[0x9a9a80]` have reached 0x80, 0xe0 or 0x140
    /// (0x42e122–0x42e186) — and, where nothing is left over, sets the byte at `G_GAME_MANAGER + 0xd` and
    /// answers 3, the same 3 that skips both clocks. Under 0x80 bullets the count goes back to nothing. Asked
    /// for by an e2e test rather than every laid-out run having it, and with the bullets at 0x80, which is one
    /// update in [`SLOWED_EVERY`] — a test that wants more of them puts more bullets up.
    ///
    /// **What it is here for is the arithmetic a playback is bounded by**: orb drives updates itself to put a
    /// run back, so a stage of 24119 frames is not 24119 updates and a bound counting them as frames runs out
    /// with the run still going. Stage 5 is where this is on and stage 6 is not, so it is the way a session
    /// has *not* shown yet — the one it did show is a clock that stopped, which `resume`'s own bound is
    /// written against.
    ///
    /// The byte itself is not written: nothing of orb reads it — see `Th07`'s `GAME_MENU_UP`, where it is
    /// written down beside its neighbour — and what a test can ask about is the clocks.
    pub fn slows_its_own_run_down(&self) {
        self.slowed.set(true);
        self.image.bullets_alive(BULLETS_THE_SLOWDOWN_STARTS_AT);
    }

    /// What orb reads of the run, for an e2e test that asks what a chapter put back: the game's own memory
    /// through the same reads orb's own frame makes.
    pub fn state(&self) -> orb_core::game::State {
        unsafe { orb_core::game::th07::Th07.read_state() }
    }

    /// The word the game's own input read came back with on the frame just run, for an e2e test that
    /// pushed a pad.
    /// The word the run **last** acted on, which is the half of the pair a press is an edge against: a bomb is
    /// a press rather than a hold, so what says one happened is the two words together.
    ///
    /// The other half is [`Fake::the_player_read`], which is what the player's own job read on the frame just
    /// run — and this one is what that job read beside it, the recorder below it moving the word on before
    /// the update is over.
    pub fn word_the_run_acted_on(&self) -> u16 {
        self.the_player_read_before.get()
    }

    pub fn word_the_input_read_answered(&self) -> u16 {
        self.word.get()
    }

    /// The whole of what this game does in one update: spend the frame's own work.
    ///
    /// Nothing else, because nothing else is read. `Th07::read_state` answers that no run is in
    /// progress without looking at the memory, so a scene written here would be a scene nothing could
    /// see — and inventing one would be this game claiming to know something the exe has not been read
    /// for.
    fn update(&self) {
        self.comes_up_at_its_title();
        // The read the game's own update makes for a frame's buttons, which orb stands in front of: the whole
        // read, whose own tail call is the pad half — so what a frame asks for once is the keys and the pads
        // together, the way th07's 0x430b50 asks.
        //
        // Each half only where a launch installs that hook, for the reason `orbs_own_loop` is: a read orb
        // does not stand in front of is one the game's own answers, which for the keys is [`input`] and for
        // the pads is a machine this game finds no device on.
        if self.reads_the_keys {
            self.word.set(orb_core::runtime::get_input());
        } else if self.reads_the_pad {
            self.word.set(input());
        }
        // And that word into both of the places the game's own read puts it: the word the read left, which is what
        // its front end and its pause read, and the word the run acts on, which is what its stage reads. A resumed
        // run's landing is held against the second of those, so a game that never wrote one down has nothing for
        // that comparison to be about — and a scene built with either still standing is a press nobody made being
        // spent, which is what a transition clearing them is for.
        //
        // The word the read left is kept first, because the manager below reads the pause off the pair of them.
        //
        // **Except on the update after one the game skipped**, where 0x62627d is up: 0x437c70 then ors the read
        // into the word the last one left and shifts nothing, so what that skipped update read is still pressed
        // and the edge is against the word before it.
        let read_before = if self.read_held_over.get() {
            self.image
                .the_read_left_the_word(self.image.the_word_the_read_left() | self.word.get());
            self.read_before.get()
        } else {
            let read_before = self.image.the_word_the_read_left();
            self.image.the_read_left_the_word(self.word.get());
            read_before
        };
        self.read_before.set(read_before);
        self.follows_its_manager(read_before);
        self.walks_its_front_end(read_before);
        self.walks_the_screen_a_run_ends_at(read_before);
        // **And the word the run acts on written where this game writes it, which is not here.** The read is
        // the supervisor's own job at chain priority 0; what copies it into the run's word is the recorder, at
        // priority 0x10 (0x443b9a) — and the player's own job is priority 8 (0x442ab6), which reads that word
        // at 0x43ee8c. So a frame of this game has the player acting on the word the recorder left *last*
        // frame, which is the frame of input lag th07 is known for. A laid-out game that wrote both words in
        // one breath is one no e2e test could ask about it.
        //
        // So a frame here walks those three in that order: the read above, then what the player's own job
        // reads, then the copy. The middle one is what an e2e test has to be able to ask about — the word a
        // frame is *played* with — so it is kept rather than acted on, this game having no player to move.
        self.follows_its_supervisor();
        // **And the manager's own 3 stops that walk**, which is what an update it does not advance the run on
        // is: the player's job and the recorder's are both below it, so neither the word the run acts on nor
        // last frame's of it is touched — and a press made on such a frame is still a press when the update
        // that does advance the run reads the pair. See [`Fake::slows_its_own_run_down`].
        let state = self.state();
        let advances = !state.in_run || (!state.paused && self.advances_the_run());
        self.read_held_over
            .set(state.in_run && !state.paused && !advances);
        if advances {
            self.the_player_read
                .set(self.image.the_word_the_run_acts_on());
            self.the_player_read_before
                .set(self.image.the_word_the_run_acted_on());
            // And the copy the recorder makes, out of the word the read left: the same value orb's own
            // hand-over put there, which is why the lag is the order and not the copy. **With the word the run
            // acts on moved into last frame's first** (0x442cf2), so that what orb wrote there lasts as far as
            // the player's job and no further.
            self.image
                .the_run_acted_on_the_word(self.image.the_word_the_run_acts_on());
            self.image
                .the_run_acts_on_the_word(self.image.the_word_the_read_left());
        }
        // And the run's own two clocks, where an e2e test asked for a run that goes on: the frames the stage
        // has run and the frames its script has, each one further on. **A stage that stands still is not a
        // stage** — orb waits a stage's own settling out before it takes the first chapter of one, judges
        // every boundary on the script's clock, and writes a frame's buttons down against the stage's — so a
        // laid-out run whose clocks never moved is a run nothing of the mode ever reaches.
        //
        // Asked for rather than always, which is the rule this whole file keeps: a laid-out game does nothing
        // no e2e test said, and the ones that read a state line back want the numbers they wrote.
        //
        // Every run and not only one somebody is playing: a replay's stage runs its own clocks the same way,
        // and a replay whose clocks stood still is one no e2e test could tell from a run orb had stopped.
        //
        // The stage over, where an e2e test said how long one lasts: the next one built inside this update,
        // which is what a transition of this game is. Instead of the clocks below rather than beside them —
        // the stage that comes out of it is at its own first frame, and one more would be the frame after.
        if state.in_run && Some(state.stage_frames) == self.fight_won_at.get() {
            self.the_boss_is_beaten();
        }
        if state.in_run && Some(state.stage_frames) == self.stage_frames.get() {
            // **And the last stage a run plays hands over to the ending rather than to a seventh**, which is
            // what clearing the game is — see [`STAGES_A_RUN_PLAYS`].
            if state.stage + 1 < STAGES_A_RUN_PLAYS {
                self.moves_on_to_the_stage(state.stage + 1);
            } else {
                self.enters_the_ending();
            }
        } else if self.run_goes_on.get() && state.in_run && advances {
            // Not while the game's own menu is up, which is what that flag *is*: the manager returns before any
            // job of the run runs — see `Th07`'s `GAME_MENU_UP` — so neither clock moves and the frame the game
            // did not advance the run on is what `resume::Record::note` writes down twice.
            self.image.has_been_in_the_stage_for(state.stage_frames + 1);
            self.image
                .the_script_has_run_for(state.script_frames.saturating_add(1));
            // And the boss's own timer with them, where one is fighting: what says a fight has moved on to
            // its next attack is that timer going *back*, so a timer that never went forward is a fight
            // orb can never see move.
            if self.boss.get() {
                self.image.a_boss_is_fighting(
                    BOSS_LIFE,
                    state.boss_attack_frames.unwrap_or(0).saturating_add(1),
                );
            }
            // And the stage's own script, bullets and player, where the run is one this game's front end
            // started — see [`Fake::plays_its_own_stage`].
            if self.on_its_own.get() {
                self.plays_its_own_stage();
            }
        }
        // And the ending's own script one update further on, where one is running: the updates an ending takes
        // are the game's own, and what says a part of it has ended is the next file read over the one it was
        // running — see [`Fake::an_ending_is_running`].
        if state.in_ending && self.ending_updates.get() > 0 {
            let left = self.ending_updates.get() - 1;
            self.ending_updates.set(left);
            if left == 0 {
                if self.ending_leaves.get() {
                    self.image.the_front_end_is_up();
                } else {
                    self.image.the_ending_reads_the_script(ROLLS_SCRIPT);
                    if self.sound.borrow().is_some() {
                        self.plays_another_song();
                    }
                }
            }
        } else if state.in_ending && !self.ending_leaves.get() {
            // And the roll's own updates after the handover, where an e2e test said how long it is: the
            // ending's scene left when they run out — see [`Fake::the_roll_lasts`].
            if let Some(left) = self.roll_updates.get().filter(|left| *left > 0) {
                self.roll_updates.set(Some(left - 1));
                if left == 1 {
                    // **A cleared run's roll hands over to the screen its own end arrives on**, in that
                    // screen's own first phase — see [`Fake::enters_the_ending`]. A roll an e2e test laid out
                    // goes to the front end, that test being about the roll rather than about what a run
                    // leaves behind.
                    //
                    // **And not to the offer of a replay, which is a phase along from it.** That phase is
                    // where the name a ranking is registered under is typed — `Th07::types_its_name` refuses
                    // any other — and `RESULT_FROM_A_RUN` and `RESULT_TYPING_A_NAME` are the same `0xa`, so
                    // offering the replay in the same update wrote over the one phase a name can be typed in.
                    // What reaches the offer is the decide the recipe presses there, which is
                    // [`Fake::walks_the_screen_a_run_ends_at`].
                    if self.cleared.replace(false) {
                        self.the_run_ends_at_its_own_screen();
                    } else {
                        self.image.the_front_end_is_up();
                    }
                }
            }
        }
        self.sim().clock().advance_micros(self.work_this_frame());
    }

    /// **The pause menu put up and taken down, which the manager does off the word the read left.** 0x42d927
    /// takes the menu bit out of that word and 0x42d939 holds it against last frame's of the same word, so what
    /// puts the menu up is the button's own edge — and what the flag then does is stop the run's update, which
    /// is why the clocks below stand still while it is up.
    ///
    /// **And the music paused with it and started again on the way out**, the manager sending the one (0x42d9b1)
    /// and the menu the other (0x403496): a game that paused nothing could not tell orb's own un-pause from
    /// nothing at all — see `Image::the_manager_pauses_the_music`.
    ///
    /// **Here because it is what makes a run's own buttons dangerous to feed back.** th07's replay never
    /// reaches this: the recorder writes a watched replay's entry into the word the *run* acts on, and the word
    /// the read left keeps whatever the keyboard did. A resume of orb's goes in through the read instead, so
    /// every bit it feeds is a bit this reads — and a laid-out game that left the pause to a test's own
    /// `is_paused` could not tell a resume that opens the menu from one that does not.
    ///
    /// Only in a run, the manager's update being what runs there: the mode question's own cancel is this key at
    /// the front end, where nothing of a stage is standing.
    /// Whether this update moves the run's clocks at all — see [`Fake::slows_its_own_run_down`], which is the
    /// game's own answer of 3 to one update in [`SLOWED_EVERY`].
    ///
    /// Counted per update of a run rather than per frame of the host, because that is what the game counts:
    /// every update orb drives itself is one of them, which is the whole point of it.
    fn advances_the_run(&self) -> bool {
        if !self.slowed.get() {
            return true;
        }
        let updates = self.updates_of_the_run.get().wrapping_add(1);
        self.updates_of_the_run.set(updates);
        let every = match self.image.the_bullets_alive() {
            0x140.. => 3,
            0xe0.. => 4,
            BULLETS_THE_SLOWDOWN_STARTS_AT.. => SLOWED_EVERY,
            _ => {
                self.updates_of_the_run.set(0);
                return true;
            }
        };
        !updates.is_multiple_of(every)
    }

    fn follows_its_manager(&self, read_before: u16) {
        if !self.state().in_run {
            return;
        }
        let word = self.image.the_word_the_read_left();
        if self.image.is_it_paused() {
            self.walks_its_pause_menu(word, read_before);
        } else if word & MENU_BUTTON != 0 && read_before & MENU_BUTTON == 0 {
            self.image.is_paused();
            self.image.the_manager_pauses_the_music();
            self.pause_menu.set((PAUSE_MENU_SETTING_UP, 0));
        }
    }

    /// **One update of the game's own pause menu, 0x402780**, which the text manager's job runs from the
    /// update after the flag goes up (0x4017e0, under `[0x62f64c]`): its first item, 続ける, is the one this
    /// game's e2e tests decide, and the menu button's edge closes it from any state but the closing itself.
    ///
    /// **A decide is not the menu going down.** The first item takes one only once the timer has reached 4
    /// (0x402e2e), and sends the menu to its closing state, which puts the flag down — and the music back —
    /// once the timer has reached 0x14 (0x403416). The timer goes up at the function's end (0x4039d0).
    fn walks_its_pause_menu(&self, word: u16, read_before: u16) {
        let edge = |bits: u16| word & bits != 0 && word & bits != read_before & bits;
        let (mut state, mut timer) = self.pause_menu.get();
        if edge(MENU_BUTTON) && state != PAUSE_MENU_CLOSING {
            (state, timer) = (PAUSE_MENU_CLOSING, 0);
        }
        if state == PAUSE_MENU_SETTING_UP {
            (state, timer) = (PAUSE_MENU_ON_ITS_FIRST_ITEM, 0);
        }
        if state == PAUSE_MENU_ON_ITS_FIRST_ITEM {
            if timer >= PAUSE_MENU_TAKES_A_DECIDE_FROM && edge(MENU_DECIDE) {
                (state, timer) = (PAUSE_MENU_CLOSING, 0);
            }
        } else if state == PAUSE_MENU_CLOSING && timer >= PAUSE_MENU_CLOSES_AT {
            self.image.is_not_paused();
            self.image.the_menu_un_pauses_the_music();
            state = PAUSE_MENU_SETTING_UP;
        }
        self.pause_menu.set((state, timer + 1));
    }

    /// **The screen a run's own end arrives on, walked by the presses whoever finished the run makes**: the
    /// name entry on to the offer of a replay, and that offer to the phase it leaves in — after which the game
    /// puts the scene back itself, which is [`Fake::the_front_end_comes_back`].
    ///
    /// Measured on the case that plays a whole game: the real th07 read `front end: screen 0, cursor on item
    /// 0` after the run ended, and the recipe's own tap is what got it there — where a laid-out game that took
    /// no press at that screen sat on it for the whole of the case's bound, the step aiming at the title
    /// giving up and every step after it pressing at nothing.
    ///
    /// **The two phases this screen is read by and not every phase it has.** The entry's own walk goes through
    /// 0x10 and 0x11 on the way to the offer — 0x44808d — and nothing above the game reads either, so a decide
    /// at the entry arrives at the offer here. What orb does about the offer is its own business: a run with
    /// chapters has it written past the frame it comes up, which is the rest of the way down.
    fn walks_the_screen_a_run_ends_at(&self, read_before: u16) {
        if self.image.the_scene_now() == self.image.scene_of_the_ranking() {
            return self.image.walks_the_rankings_build();
        }
        if self.image.the_scene_now() != self.image.scene_of_a_run_over() {
            return;
        }
        let word = self.image.the_word_the_read_left();
        if word & MENU_DECIDE == 0 || read_before & MENU_DECIDE != 0 {
            return;
        }
        let phase = self.image.the_phase_the_screen_is_in();
        if phase == self.image.phase_of_a_screen_from_a_run() {
            self.image.the_screen_a_run_ends_at_offers_a_replay();
        } else if phase == self.image.phase_of_a_screen_offering_a_replay() {
            self.image.the_screen_a_run_ends_at_leaves();
        }
    }

    /// The half of a scene change that is the game's: orb asks for the ranking after a run has ended by writing
    /// the supervisor's own `wanted` — see `Th07::show_ranking` — and it is the game that builds that screen,
    /// and the screen that puts the scene back where it has been told to leave.
    ///
    /// Here rather than in a step an e2e test takes because both happen inside orb's own frame: `commit_records`
    /// calls the game's update again and again while it waits for that screen, so a game that only moved
    /// between frames would be a screen that never comes up.
    ///
    /// **The ranking and the front end after it, and no other scene.** The run's own is asked for by the front
    /// end rather than by orb, so a laid-out game that followed that one too would build a stage no e2e test
    /// asked for — see [`Fake::builds_a_stage`], which is where a stage comes from here.
    fn follows_its_supervisor(&self) {
        let front_end = self.image.scene_of_the_front_end();
        if self.image.the_screen_a_run_ends_at_is_leaving() {
            return self.the_front_end_comes_back();
        }
        if self.image.the_scene_now() == self.image.the_scene_wanted() {
            return;
        }
        let wanted = self.image.the_scene_wanted();
        // **The end of the game, which is the loop's**: the frame answers that it is over, and the way out of the
        // loop saves the score file where the play records are tagged (0x434277) before the process goes. The
        // host's own call rather than orb's `close_the_game`: what ends the process here is the game, and a
        // laid-out host answers the call and returns instead of ending a test binary.
        if wanted == SCENE_THE_GAME_ENDS {
            if self.image.its_play_records_are_tagged() {
                let from = self.opens_its_score_file(false);
                let into = self.tries_to_open_its_score_file(true);
                self.image.saves_its_score_file(&from, into.as_deref());
            }
            return orb_api::process::exit(0);
        }
        // **The run this game's own shot select asked for, built** — on the update after the one it was asked
        // on, that being the one frame a run has been chosen and not built (see [`Fake::the_run_is_chosen`]) and
        // the frame a resume points the build at a stage on. Out of the stage word as the build reads it, which
        // is what that resume writes.
        if wanted == self.image.scene_of_a_run() && self.image.the_scene_now() == front_end {
            if self.on_its_own.get() && !self.just_chose.replace(false) {
                self.starts_its_own_stage(self.image.the_games_stage_word());
            }
            return;
        }
        if wanted == self.image.scene_of_the_ranking() {
            // `0x44a302(0)` (0x437f3a), whose added callback reads the file the ranking shows.
            self.image.the_ranking_is_up();
            self.the_screen_a_run_ends_at_reads_the_file();
            self.image.is_in_scene(wanted, wanted);
        } else if wanted == self.image.scene_of_a_run_over() {
            // The run torn down and **the screen a run ends on registered**, which is what this game's own
            // transition does for that scene: 0x43803c calls the teardown at 0x42f45d and then `0x44a302(1)`,
            // which puts that screen on the chain with the phase a run ending leaves. So it is drawn — the
            // background of it for a frame or two, which is what a launch of th07 showed — and then it asks
            // for the title itself.
            self.image.the_screen_a_run_ends_at_is_up();
            self.result_screens.set(self.result_screens.get() + 1);
            self.the_screen_a_run_ends_at_reads_the_file();
            self.image.is_in_scene(wanted, front_end);
        } else if wanted == front_end {
            if self.on_its_own.get() {
                self.leaves_its_own_run();
            }
            self.the_front_end_comes_back();
        }
    }
}

impl Fake {
    /// **The front end up again on the item the title was left on**, which is where the real game leaves a
    /// run that is over: `front end: screen 0, cursor on item 1` after a run started from `Extra Start`, and
    /// `item 0` after one started from `Start`.
    ///
    /// **And in the practice flow rather than at the title where that item practises a stage**, which is the
    /// select whoever left the run is offered another stage at: the real game read `screen 8, cursor on item
    /// 2` on the frame a left practice run came back — the item the title was left on, still in the one
    /// cursor this front end has — and screens 9, 10 and 11 on the three frames after it. See
    /// [`walks_back_into_the_practice_flow`](Fake::walks_back_into_the_practice_flow) for those three.
    ///
    /// The read of the score file goes with it, which is where that screen reads what its menus offer — its
    /// records over the ones in memory included. **A run given up walks through here before orb's own save**,
    /// so a laid-out game that left it out would be one no e2e test could ask what a read between the two
    /// costs.
    fn the_front_end_comes_back(&self) {
        self.image.the_front_end_is_up_again();
        let item = self.left_the_title_on.get();
        self.image.the_title_menu_points_at(item);
        if item == TITLE_ITEM_PRACTICE {
            self.enters_the_screen(SCREEN_PRACTICE_DIFFICULTY, item);
            self.coming_back_through_the_practice_screens.set(true);
        }
        self.the_front_end_reads_the_file_for_its_menus();
    }

    /// And the rest of that flow walked, one screen a frame: the character, the shot and the stage, each on
    /// its own first item, which is what the real game's three frames after it read.
    ///
    /// A walk of the game's own and not one a press makes, so it happens before the gate a press is taken on:
    /// nothing is pressed on those frames and the screens come up all the same.
    fn walks_back_into_the_practice_flow(&self, screen: i32) {
        match screen {
            SCREEN_PRACTICE_DIFFICULTY => {
                self.enters_the_screen(SCREEN_PRACTICE_CHARACTER, i32::from(CHARACTER));
            }
            SCREEN_PRACTICE_CHARACTER => {
                self.enters_the_screen(SCREEN_PRACTICE_SHOT_SELECT, i32::from(SHOT));
            }
            SCREEN_PRACTICE_SHOT_SELECT => {
                self.enters_the_screen(SCREEN_PRACTICE_STAGE, FIRST_STAGE);
                self.coming_back_through_the_practice_screens.set(false);
            }
            _ => self.coming_back_through_the_practice_screens.set(false),
        }
    }

    /// **The title menu up on its own**, [`TITLE_COMES_UP_AFTER`] frames into a launch nothing else has placed a
    /// scene in: the real game comes up at its title, and a recipe's first press is made at it.
    ///
    /// Only where no e2e test has laid a scene out — the supervisor still at the pair a fresh process has, which
    /// is neither the front end's nor a run's — so a test that places the game itself is left with what it placed.
    /// The count of frames the screen has been up starts from nothing, the game's own gate on a press being that
    /// count against ten.
    fn comes_up_at_its_title(&self) {
        let frames = self.frames_since_launch.get() + 1;
        self.frames_since_launch.set(frames);
        if frames == TITLE_COMES_UP_AFTER
            && self.image.the_scene_now() == 0
            && self.image.the_scene_wanted() == 0
        {
            self.image.the_front_end_is_up();
            self.image.the_screen_has_been_up_for(0);
        }
    }

    /// **The front end walked by the word the read left**, the way the game's own screens walk on it: the count
    /// of frames the screen has been up moved on, the cursor moved on a direction's edge, and a decide or a cancel
    /// on the screen's own gate taking it to the screen after or the one before — Start's flow of three and the
    /// run it ends in. See [`SCREEN_DIFFICULTY`] for which screens those are.
    ///
    /// **Edges and not holds**, which is what the game's own reads are: a button still down from the screen before
    /// is a press already spent, and the frame of input lag between the read and the run does not apply here, the
    /// front end reading the word the read left directly.
    fn walks_its_front_end(&self, read_before: u16) {
        let front_end = self.image.scene_of_the_front_end();
        if self.image.the_scene_now() != front_end || self.image.the_scene_wanted() != front_end {
            return;
        }
        // **On its way down to the end of the game**, which takes no press: the front end's phase 2 waits for
        // its timer to pass 0x3b and then asks the supervisor for the scene that ends it (0x4554d6's own case).
        if let Some(updates) = self.ending_the_game_for.get() {
            if updates > THE_WAY_DOWN_WAITS_FOR {
                self.image.is_in_scene(front_end, SCENE_THE_GAME_ENDS);
            } else {
                self.ending_the_game_for.set(Some(updates + 1));
            }
            return;
        }
        let up_for = self.image.frames_the_screen_has_been_up().saturating_add(1);
        self.image.the_screen_has_been_up_for(up_for);
        let screen = self.image.the_screen_the_front_end_is_on();
        if self.coming_back_through_the_practice_screens.get() {
            return self.walks_back_into_the_practice_flow(screen);
        }
        let items = match screen {
            SCREEN_TITLE => TITLE_ITEMS,
            SCREEN_DIFFICULTY | SCREEN_PRACTICE_DIFFICULTY | SCREEN_EXTRA_DIFFICULTY => {
                DIFFICULTIES_OFFERED
            }
            SCREEN_CHARACTER | SCREEN_PRACTICE_CHARACTER | SCREEN_EXTRA_CHARACTER => CHARACTERS,
            SCREEN_SHOT_SELECT | SCREEN_PRACTICE_SHOT_SELECT | SCREEN_EXTRA_SHOT_SELECT => SHOTS,
            SCREEN_PRACTICE_STAGE => STAGES_TO_PRACTISE,
            _ => return,
        };
        // The gate: the title's is the count itself, and every other screen's is the phase it moves to once it
        // has been up for thirty — see [`A_SCREEN_TAKES_A_PRESS_AFTER`].
        let takes_a_press = if screen == SCREEN_TITLE {
            up_for >= TITLE_TAKES_A_PRESS_AFTER
        } else {
            if up_for == A_SCREEN_TAKES_A_PRESS_AFTER {
                self.image.the_screen_is_in_phase(PHASE_ITEMS);
            }
            up_for >= A_SCREEN_TAKES_A_PRESS_AFTER
        };
        if !takes_a_press {
            return;
        }
        let word = self.image.the_word_the_read_left();
        let pressed = |bit: u16| word & bit != 0 && read_before & bit == 0;
        // The cursor, which goes round: th07's title menu wraps from its first item to its last, which is what a
        // stray up at it costs — see `Th07`'s `sight::in_a_stage`.
        let cursor = self.image.the_item_the_title_menu_points_at();
        let step = i32::from(pressed(BUTTON_DOWN)) - i32::from(pressed(BUTTON_UP));
        let moved = (cursor + step).rem_euclid(items);
        // **And the title steps over the item it is not offering**, which is the Extra's where the records hold
        // no clear: measured on a copy whose score file had none, the cursor going 0 to 2 pressing down and 0 to
        // 7 pressing up — see [`its_records_hold_no_clear`](Fake::its_records_hold_no_clear). The title is the
        // only screen with such an item, so what is stepped over is that item's own number.
        let locked =
            screen == SCREEN_TITLE && !self.clear_in_its_records.get() && moved == TITLE_ITEM_EXTRA;
        let moved = match locked {
            true => (moved + step).rem_euclid(items),
            false => moved,
        };
        if moved != cursor {
            self.image.the_title_menu_points_at(moved);
        }
        if pressed(MENU_DECIDE) {
            // **The item the title is left on is what the front end comes back on**, and it is remembered
            // here because every screen between an item and a run writes its own list's item into the one
            // cursor this front end has. The real game read `cursor on item 1` where the run before it was
            // started from `Extra Start` and `item 0` where one was started from `Start` — see
            // `the_front_end_of_th07_walked`.
            if screen == SCREEN_TITLE {
                self.left_the_title_on.set(moved);
            }
            match screen {
                SCREEN_TITLE if moved == TITLE_ITEM_START => {
                    self.walking.set(true);
                    self.enters_the_screen(SCREEN_DIFFICULTY, DIFFICULTY);
                }
                // **And the item that practises one stage, which is a run as much as Start's is**: its own
                // three screens, and the bit this item sets — the select behind them sets it again, which is
                // what 0x45a583 does and what makes the shot select's own write in between harmless.
                SCREEN_TITLE if moved == TITLE_ITEM_PRACTICE => {
                    self.walking.set(true);
                    self.image.is_practising();
                    self.enters_the_screen(SCREEN_PRACTICE_DIFFICULTY, DIFFICULTY);
                }
                // **And the item that starts the Extra**, whose own three screens are the third triple
                // through the same handlers: what it decides is written by the item and by the select behind
                // it rather than chosen anywhere — the difficulty is the Extra's and so is the stage.
                // **The item that ends the game, which is this menu's last**: the way somebody leaves one,
                // and what the case that clears the game walks to — the real game's trace stops a few frames
                // past the step before the decide made there, the process gone with it. Not at once: the item
                // sends the front end to its phase 2 (0x456000), and the way down from there is the walk's own
                // — see the head of this function and [`Fake::follows_its_supervisor`].
                SCREEN_TITLE if moved == TITLE_ITEMS - 1 => {
                    self.image.the_screen_is_in_phase(PHASE_ENDING_THE_GAME);
                    self.ending_the_game_for.set(Some(0));
                }
                // **The Score item, which is a scene and not a screen**: `[0x575aa8] = 5` at 0x455f95, the
                // supervisor's own `wanted` — the same write `Th07::show_ranking` makes after a run — and the
                // ranking then built by the supervisor, see [`Fake::follows_its_supervisor`].
                SCREEN_TITLE if moved == TITLE_ITEM_SCORE => {
                    self.image
                        .is_in_scene(front_end, self.image.scene_of_the_ranking());
                }
                SCREEN_TITLE if moved == TITLE_ITEM_EXTRA => {
                    self.walking.set(true);
                    // On its own first item, which is what the real one came up on: `front end: screen 12,
                    // cursor on item 0`, and no reading of that screen on any other item. The difficulty is
                    // the Extra's and the item wrote it — a cursor put on that difficulty here is a number
                    // this select has no row for, `DIFFICULTIES_OFFERED` being four.
                    self.enters_the_screen(SCREEN_EXTRA_DIFFICULTY, 0);
                }
                SCREEN_DIFFICULTY | SCREEN_PRACTICE_DIFFICULTY => {
                    self.difficulty.set(moved);
                    self.enters_the_screen(
                        match screen {
                            SCREEN_DIFFICULTY => SCREEN_CHARACTER,
                            _ => SCREEN_PRACTICE_CHARACTER,
                        },
                        i32::from(CHARACTER),
                    );
                }
                // The Extra's own first screen, whose cursor decides nothing: its difficulty is the Extra
                // and no item of that select is another.
                SCREEN_EXTRA_DIFFICULTY => {
                    self.difficulty.set(DIFFICULTY_EXTRA);
                    self.enters_the_screen(SCREEN_EXTRA_CHARACTER, i32::from(CHARACTER));
                }
                SCREEN_CHARACTER | SCREEN_PRACTICE_CHARACTER | SCREEN_EXTRA_CHARACTER => {
                    let character = u8::try_from(moved).expect("a cursor over the characters");
                    self.character_chosen.set(character);
                    // The two bytes the screens before wrote and the shot under the cursor, which is what
                    // `Th07::run_pointed_at` reads a run left unfinished's mark off — and the phase the build
                    // runs in, thirty frames before the screen takes a press. The shot select of whichever
                    // flow this walk is in, the three sharing that screen's handler.
                    self.image.the_shot_select_points_at(
                        match screen {
                            SCREEN_CHARACTER => SCREEN_SHOT_SELECT,
                            SCREEN_EXTRA_CHARACTER => SCREEN_EXTRA_SHOT_SELECT,
                            _ => SCREEN_PRACTICE_SHOT_SELECT,
                        },
                        u8::try_from(self.difficulty.get())
                            .expect("a difficulty of the game's own"),
                        character,
                        SHOT,
                    );
                    self.image.the_screen_is_in_phase(PHASE_BUILDING);
                    self.image.the_screen_has_been_up_for(0);
                }
                // The run started: the three numbers written where the run keeps them, the stage word at the
                // first stage, and the run scene asked for — which the supervisor acts on next update. Only
                // where this game walked here itself, or where a test says this screen starts a run whatever
                // brought it up — see [`Fake::walking`] and
                // [`Fake::its_shot_select_starts_a_run_however_it_came_up`].
                SCREEN_SHOT_SELECT
                    if self.walking.get() || self.starts_a_run_it_did_not_walk_to.get() =>
                {
                    let shot = u8::try_from(moved).expect("a cursor over the shots");
                    self.image.the_shot_select_decided(
                        self.difficulty.get(),
                        self.character_chosen.get(),
                        shot,
                    );
                    self.image.the_games_stage_word_is(FIRST_STAGE);
                    self.on_its_own.set(true);
                    self.just_chose.set(true);
                    self.image.a_run_has_been_chosen();
                }
                // And the Extra's, which is a run starting like Start's own — with the difficulty and the
                // stage the item decided rather than any select.
                SCREEN_EXTRA_SHOT_SELECT if self.walking.get() => {
                    let shot = u8::try_from(moved).expect("a cursor over the shots");
                    self.image.the_shot_select_decided(
                        DIFFICULTY_EXTRA,
                        self.character_chosen.get(),
                        shot,
                    );
                    self.image.the_games_stage_word_is(THE_EXTRAS_STAGE);
                    self.on_its_own.set(true);
                    self.just_chose.set(true);
                    self.image.a_run_has_been_chosen();
                }
                // The practice flow's own way out of that screen, which is not a run starting: the stage it
                // is played in is asked for at the select behind it.
                SCREEN_PRACTICE_SHOT_SELECT if self.walking.get() => {
                    let shot = u8::try_from(moved).expect("a cursor over the shots");
                    self.image.the_shot_select_decided(
                        self.difficulty.get(),
                        self.character_chosen.get(),
                        shot,
                    );
                    self.enters_the_screen(SCREEN_PRACTICE_STAGE, FIRST_STAGE);
                }
                // And the stage under the cursor practised, which is the last answer a practice run needs:
                // the bit that select's own handler sets, the stage word the build reads, and the run scene
                // asked for.
                SCREEN_PRACTICE_STAGE if self.walking.get() => {
                    self.image.is_practising();
                    self.image.the_games_stage_word_is(moved);
                    self.on_its_own.set(true);
                    self.just_chose.set(true);
                    self.image.a_run_has_been_chosen();
                }
                _ => {}
            }
        } else if pressed(MENU_CANCEL) {
            match screen {
                // **A cancel at the title puts the cursor on its last item**, which is the one that ends the
                // game: measured on the real game, where the six-frame cancel a recipe presses to take orb's
                // own question away read `front end: screen 0, cursor on item 7` on the frame it went in —
                // and every walk the recipes make afterwards starts from there. th06 does the same.
                SCREEN_TITLE => self.image.the_title_menu_points_at(TITLE_ITEMS - 1),
                SCREEN_DIFFICULTY => self.enters_the_screen(SCREEN_TITLE, TITLE_ITEM_START),
                SCREEN_CHARACTER => {
                    self.enters_the_screen(SCREEN_DIFFICULTY, self.difficulty.get())
                }
                SCREEN_SHOT_SELECT => {
                    self.enters_the_screen(
                        SCREEN_CHARACTER,
                        i32::from(self.character_chosen.get()),
                    );
                }
                _ => {}
            }
        }
    }

    /// One of the front end's screens brought up with its cursor on `cursor`: in the phase its build runs in and
    /// up for no frames yet, which is the state a press there is not taken in — the title excepted, whose gate is
    /// the count alone.
    fn enters_the_screen(&self, screen: i32, cursor: i32) {
        self.image.the_front_end_is_on_the_screen(screen);
        self.image.the_title_menu_points_at(cursor);
        self.image
            .the_screen_is_in_phase(if screen == SCREEN_TITLE {
                PHASE_ITEMS
            } else {
                PHASE_BUILDING
            });
        self.image.the_screen_has_been_up_for(0);
    }

    /// The stage a run of this game's own starting asked for, built and laid out to be played: the build every
    /// stage gets, and then the field, the player's box and step and where they start — which is what the rules a
    /// bot plays by read — with nothing of a stage before it left over.
    ///
    /// And the song the run comes back to, where an e2e test laid one out: a run given up took its stream down,
    /// and the build of the next one starts it again.
    fn starts_its_own_stage(&self, stage: i32) {
        self.builds_the_stage(stage);
        self.image
            .the_play_field_is(FIELD.0, FIELD.1, FIELD.2, FIELD.3);
        self.image.the_players_box_is((PLAYER_HALF, PLAYER_HALF));
        self.image
            .the_player_steps_by(A_STEP, A_STEP / 2.0_f32.sqrt());
        self.image
            .the_player_is_at(PLAYER_STARTS_AT.0, PLAYER_STARTS_AT.1);
        self.bullets.borrow_mut().clear();
        self.writes_the_bullets_down();
        self.last_stage_frame.set(0);
        self.hit_window.set(0);
        self.bomb_left.set(0);
        if self.sound.borrow().is_some() {
            self.image.names_the_stream_it_had();
            orb_sim::Sound::started_again();
        }
    }

    /// **The ending a cleared run hands over to**: the run torn down the way leaving one tears it down, the
    /// ending's own script running in its place, and the staff roll behind that — with which of the two
    /// lengths is which left to [`Fake::update`], where the updates are counted down.
    ///
    /// The run is what asks for this rather than an e2e test, because clearing the game is what a case that
    /// plays one through measures: `an_ending_is_running` is the same arrangement made from outside, for a
    /// test about the ending alone.
    fn enters_the_ending(&self) {
        self.leaves_its_own_run();
        self.cleared.set(true);
        // **And the clear in the records in memory**, which is where the title reads `Extra Start` from on the same
        // launch: the ending writes `99` into the shot's `CLRD` byte for the difficulty (0x41e590) and the title
        // asks those bytes (0x42f7df).
        self.clear_in_its_records.set(true);
        self.an_ending_is_running(ENDING_UPDATES);
        self.the_roll_lasts(ROLL_UPDATES);
    }

    /// And that run left for the title, which is what orb's own give-up asks the supervisor for: the stream the
    /// run was playing through deleted, as the teardown at 0x44cebd does — see [`Fake::gives_the_run_up`] — and
    /// the bullets gone with the stage.
    fn leaves_its_own_run(&self) {
        self.on_its_own.set(false);
        self.bullets.borrow_mut().clear();
        self.writes_the_bullets_down();
        if self.sound.borrow().is_some() {
            self.image.plays_no_track();
            orb_sim::Sound::taken_down();
        }
    }

    /// **One update of a stage this game plays on its own**: the script's events for the frame the stage has
    /// reached, a bullet aimed at the player where the schedule says, every bullet moved on, the player moved by
    /// the word their job read, and the hit test — with th07's own window between a hit and the death, which a
    /// bomb pressed inside is a deathbomb.
    ///
    /// **A stage put back is a stage whose bullets are gone**: the frames this stage has run going backwards is a
    /// chapter restored, the memory then holding what the chapter had, and what this game kept beside that memory
    /// — its bullets, a hit's window — is what the restore could not put back, so it is dropped to agree.
    fn plays_its_own_stage(&self) {
        let state = self.state();
        let now = state.stage_frames;
        let last = self.last_stage_frame.get();
        if now < last {
            self.bullets.borrow_mut().clear();
            self.hit_window.set(0);
            self.bomb_left.set(0);
        } else {
            let reached = |at: u32| last < at && at <= now;
            if reached(MIDBOSS_ARRIVES) {
                self.the_boss_arrives();
            }
            if reached(CARD_STARTS) {
                self.a_card_starts(SCRIPTED_CARD, SCRIPTED_CARD_NAME);
            }
            if reached(ATTACK_CHANGES) {
                self.the_card_is_over();
                self.the_attack_changes();
            }
            if reached(MIDBOSS_BEATEN) {
                self.the_boss_is_beaten();
            }
            if now > last
                && now >= FIRST_BULLET_AT
                && (now - FIRST_BULLET_AT).is_multiple_of(A_BULLET_EVERY)
            {
                self.shoots_at_the_player();
            }
        }
        self.last_stage_frame.set(now);

        let word = self.the_player_read.get();
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
            // Playing, and so something a bullet can hit: the one whose box reaches the player's puts them into
            // the state the hit test leaves at 0x43ee0e, and is spent doing it.
            0 => {
                let reach = PLAYER_HALF + BULLET_SIZE / 2.0;
                let hit = self.bullets.borrow().iter().position(|bullet| {
                    (bullet.at.0 - player.0).abs() <= reach
                        && (bullet.at.1 - player.1).abs() <= reach
                });
                if let Some(hit) = hit {
                    self.bullets.borrow_mut().remove(hit);
                    self.image.the_player_has_been_hit();
                    self.hit_window.set(A_BOMB_IS_TAKEN_FOR);
                }
            }
            // Hit, inside the window: a bomb pressed takes the hit back, and the window running out is the death.
            2 => {
                if word & BUTTON_BOMB != 0 && state.bombs > 0 {
                    self.a_bomb_goes_off();
                    self.image.the_player_is_playing();
                    self.bomb_left.set(A_BOMB_LASTS);
                    self.hit_window.set(0);
                } else {
                    let left = self.hit_window.get() - 1;
                    self.hit_window.set(left);
                    if left <= 0 {
                        self.dies();
                    }
                }
            }
            _ => {}
        }
        if self.bomb_left.get() > 0 {
            self.bomb_left.set(self.bomb_left.get() - 1);
            if self.bomb_left.get() == 0 {
                self.the_bomb_is_over();
            }
        }
        self.writes_the_bullets_down();
    }

    /// A bullet from the top of the field aimed at where the player is now, in the lowest slot nothing is in.
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

    /// The player moved by the directions in the word, by the step a frame moves them, and held inside the field.
    fn moves_the_player(&self, word: u16) {
        let across = i32::from(word & BUTTON_RIGHT != 0) - i32::from(word & BUTTON_LEFT != 0);
        let down = i32::from(word & BUTTON_DOWN != 0) - i32::from(word & BUTTON_UP != 0);
        if across == 0 && down == 0 {
            return;
        }
        let step = if across != 0 && down != 0 {
            A_STEP / 2.0_f32.sqrt()
        } else {
            A_STEP
        };
        let (x, y) = self.image.where_the_player_is();
        self.image.the_player_is_at(
            (x + across as f32 * step).clamp(FIELD.0, FIELD.0 + FIELD.2),
            (y + down as f32 * step).clamp(FIELD.1, FIELD.1 + FIELD.3),
        );
    }

    /// Every slot this stage uses written as it stands: the bullets that are out, alive with their position, step
    /// and size, and every other slot empty — and the count their manager keeps.
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
    }
}

impl Drop for Fake {
    /// The game closing: the runtime first, so orb's overlay is released through a device that is still
    /// there, then the fields in the order they are declared.
    fn drop(&mut self) {
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The exe this game is running as, which is the file name th07 ships under, and the `Game` orb is
/// attached to found out of the table by it.
///
/// Asked rather than named for the same reason th06's is — see `fake::th06::the_game_this_is`: what says
/// which game a process is, in a launch and here, is the exe's own name.
///
/// # Panics
/// Where no entry holds that name.
pub const EXE: &str = "th07.exe";

/// **The walk from the title to a run of the mode, as a recipe**, which is the real recipes' own — see
/// `orb-real/recipes/th07/a_run_left_in_its_second_chapter.yaml` for why each step is shaped so — ending
/// on the run's first chapter.
///
/// **Here rather than in the e2e module that plays it**, for the reason `fake::th08::TO_A_RUN` is: it is
/// this game's screens, and a screen number changed in one copy of it would leave the others walking a game
/// that no longer exists.
pub(crate) const TO_A_RUN: &str = "\
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { frames: 60 }
- press: [down]
  frames: 6
  until: { screen: 4, cursor: 1 }
  give_up_after: 600
- press: [decide]
  frames: 6
  until: { stage: 1, chapter: 1 }
  give_up_after: 1800
";

fn the_game_this_is() -> &'static dyn orb_core::game::Game {
    orb_core::game::found(EXE)
        .unwrap_or_else(|| panic!("{EXE} is a game orb knows"))
        .game
}

extern "fastcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.update();
    fake.answers.get()
}

/// `GameWindow::Render` at 0x4346e0: the game's own whole frame, in th07's own draw-then-update order.
///
/// What orb's frame loop replaces — doing the update first is the frame of input lag removed — and what
/// that loop hands a frame back to on each of the three ways out of it that return: no runtime, no
/// device, and a chain target that is null.
///
/// **Its own drawing is here in full**, because what a frame of th07 does around its draw chain is the
/// whole subject of `../th07.rs`: the queue of quads emptied at 0x434734, the word at 0x434739, the fog
/// put out at 0x434748, and the queue drawn at 0x43475d. A laid-out frame with those left out would fault
/// on its own first quad, which is what orb's loop did to the real game.
///
/// The chain's two exits become the frame's own two here as they do in the real one, that mapping being
/// th07's and not orb's. Without the frame-skip loop the real one has around them: what orb's loop
/// replacing that frame does is take the skipping out, so a frame here is one draw and one update.
///
/// No wait in it, for the reason th06's has none: the real one paces itself and this one is called a
/// frame at a time by whatever is driving the game.
extern "fastcall" fn own_render(_window: *mut c_void) -> i32 {
    let fake = running();
    let chain = Th07.chain();
    unsafe { empties_the_queue(0) };
    fake.image.forces_the_fog_call_through();
    unsafe { puts_the_fog_out(0) };
    unsafe { orb_core::runtime::run_draw_chain(chain) };
    unsafe { draws_the_queue(0) };
    let walked = unsafe { orb_core::runtime::run_calc_chain(chain) };
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

/// `SoundPlayer::PlaySounds` at 0x44c9c0: the commands standing in the player's own queue carried out, and the
/// time an e2e test says this frame's sounds cost.
///
/// Here rather than left out because the frame loop calls it where the game's own loop did, and a frame
/// that skipped it would be one span of the pacing's breakdown short. What it costs is spent here and
/// not with the rest of the frame's work, as in th06's — see [`Work::sound_us`]. The commands are the half a
/// laid-out game can answer for: this is the one place a queued play of the music becomes a track playing,
/// whether the frame reached it or orb ran it itself — see `Image::runs_the_sound_commands`.
unsafe extern "fastcall" fn play_sounds(_player: usize) {
    let fake = running();
    fake.image.runs_the_sound_commands();
    fake.launch.asked_for(SOUND);
    fake.sim().clock().advance_micros(fake.sound_this_frame());
}

/// `GameWindow::Present` at 0x4345c0: the frame handed over, which from here is the compositor's.
///
/// Where an e2e test counts a frame — the tick it was handed over at, which is what a rate is read off —
/// and where the host is told, since the next flush is what waits for this frame to be composed.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
    // And the device's own `Present`, which is what this function does in the game — four nulls through
    // +0x3c at 0x4345df — and the one path orb's letterbox is on. The slot is read out of the vtable
    // rather than remembered, because what is in it is whatever orb last put there.
    //
    // Only where the vtable is laid out, which is a launch whose device has been found: orb's own frame
    // reaches this call from the first frame, and a game that never had a device presents through nothing.
    let device = fake.launch.device();
    if fake
        .image
        .space()
        .read_committed::<usize>(device.0)
        .is_some()
    {
        let slot: usize = fake.image.space().read(fake.launch.present_slot());
        let through: DevicePresent = unsafe { std::mem::transmute(slot) };
        unsafe {
            through(
                device.0 as *mut c_void,
                std::ptr::null(),
                std::ptr::null(),
                0,
                std::ptr::null(),
            )
        };
    }
}

/// The `Present` slot's own signature, which is `orb_core::window`'s private one written out again: a
/// laid-out game calling through that slot has to call it the way Direct3D would.
type DevicePresent = unsafe extern "system" fn(
    *mut c_void,
    *const orb_api::Rect,
    *const orb_api::Rect,
    isize,
    *const c_void,
) -> orb_api::Hresult;

/// What is in that slot until orb redirects it, and what an e2e test reads the redirection back through:
/// the two rectangles the ask carried, and whichever answer an e2e test declared — `S_OK`, or the refusal a
/// driver that will not stretch gives.
unsafe extern "system" fn device_present(
    _device: *mut c_void,
    source: *const orb_api::Rect,
    destination: *const orb_api::Rect,
    _window_override: isize,
    _dirty: *const c_void,
) -> orb_api::Hresult {
    let fake = running();
    let read = |at: *const orb_api::Rect| (!at.is_null()).then(|| unsafe { *at });
    fake.presents.borrow_mut().push(Presented {
        source: read(source),
        destination: read(destination),
    });
    // Only the present that asks for one: what such a driver refuses is the *stretch*, so the game's own
    // call with no destination in it is one it answers.
    if fake.refuses_to_stretch.get() && !destination.is_null() {
        return STRETCH_REFUSED;
    }
    S_OK
}

/// What a present that worked answers, and what a driver that will not stretch on one answers instead:
/// `D3DERR_DRIVERINTERNALERROR`, which is what such a refusal really comes back as — orb reads the sign and
/// nothing else.
const S_OK: orb_api::Hresult = 0;
const STRETCH_REFUSED: orb_api::Hresult = 0x8876_0827u32 as i32;

/// `Chain::RunDrawChain` at 0x42fe20: one quad queued, which is what drawing anything in th07 is.
///
/// **The whole of why this game has a queue laid out.** A real draw chain reaches 0x44f690 for every
/// sprite on the screen, and that function writes where `[queue + 0x17e534]` points — so a frame that
/// never emptied the queue writes its first quad to address zero, which is the fault orb's first launch of
/// this game took. A laid-out game whose draw chain only said it had been asked would have passed
/// that frame and every frame after it.
extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.image.queues_a_quad();
    CHAIN_CARRIED_ON
}

/// The three the frame makes on that queue and on the fog, `__thiscall` on what the game keeps each of
/// them in: 0x44f580, 0x44f5c0 and 0x43a207.
///
/// What they do to this game's memory is `image`'s, the block being laid out there — see
/// [`Image::empties_the_queue`](orb_core::game::th07::image::Image::empties_the_queue). Handed over
/// rather than reached at their own addresses, for the reason every call out of the seam is: an address
/// space holds no code.
unsafe extern "thiscall" fn empties_the_queue(_queue: usize) {
    running().image.empties_the_queue();
}

unsafe extern "thiscall" fn draws_the_queue(_queue: usize) {
    running().image.draws_the_queue();
}

/// And the fog, which draws the queue first as the game's own does: what was queued under fog is drawn
/// under it, and putting the fog out before drawing it would draw it without.
///
/// **The field is what decides whether the device is told**, which is the game's own shape and the whole
/// of why its frame writes that field first — a call whose field already says the fog is out sets no
/// render state at all. What reaches the device is counted here rather than at the device itself: the
/// `SetRenderState` is not laid out, the device in an e2e test being a real object with a vtable of Rust
/// functions.
unsafe extern "thiscall" fn puts_the_fog_out(_supervisor: usize) {
    let fake = running();
    fake.image.draws_the_queue();
    if fake.image.fog_is_on() {
        fake.image.puts_the_fog_out();
        fake.fog_told.set(fake.fog_told.get() + 1);
    }
}

/// The game's own read of a frame's buttons, which orb calls through: no key held, and the pad half orb
/// answers on top of it — which is what th07's own read does with its tail call at 0x430f20.
///
/// The pad's own half only where orb stands in front of it, for the reason [`Fake::update`] gives: a launch
/// that declines it is one whose pads nothing reads, and the word is then the keys' alone.
extern "system" fn input() -> u16 {
    let fake = running();
    // `GetKeyboardState`, which is the read this game really makes: 0x430b76 calls the second entry of the USER32
    // import table at 0x48d1ac with a 0x100-byte buffer and tests each key's byte against 0x80. So a key another
    // program sent is a key this game reads — see `orb_sim::Keyboard::state` — and a read the host refuses leaves
    // the game with nothing held rather than with what a stack buffer happened to hold.
    let state = fake.sim().keyboard().state();
    let keys = MAP
        .iter()
        .filter(|(key, _)| {
            state
                .as_ref()
                .is_some_and(|state| state[*key as usize] & 0x80 != 0)
        })
        .fold(NO_KEY_HELD, |word, (_, bit)| word | bit);
    if fake.reads_the_pad {
        orb_core::runtime::get_controller_input_in_ecx(keys)
    } else {
        keys
    }
}

/// **The stage's build at 0x4074c0**, which orb's hook stands in front of to put a resumed run's seed in: what
/// the real build answers for success, and the stage it was handed **in `ecx`** — the supervisor's
/// `mov ecx,[eax+0x95ec]` at 0x42f0ae — kept where 0x4074fd keeps it.
extern "fastcall" fn stage_building(stage: i32) -> i32 {
    running().image.the_build_keeps_the_stage(stage);
    0
}

/// `GameManager::AddedCallback`, which the game's own chain calls where a stage's numbers have just been put in
/// place: nothing, that build being [`Fake::builds_a_stage`]'s own writes, and success is what a build that
/// worked answers.
/// `GameManager::AddedCallback`'s own body, as far as the score file goes: the save it makes before anything
/// else (0x42e646), every card record emptied (0x42e657), and the file read straight back into them (0x42e735).
///
/// **Which is what makes a run's counts the file's plus that run's**, and the whole reason it is laid out: a
/// laid-out build that left the records alone would leave them holding this run's counts alone, and a fix that
/// adds them to the file's could not be told from one that puts them back.
/// The game's own refill, which [`Fake::services_the_buffer`] reaches through orb's hook over it — see
/// `Th07`'s own `TOPS_UP_THE_MUSIC`, and `orb_core::runtime::tops_up_the_music` for what the hook says.
extern "thiscall" fn tops_up_the_music(_stream: usize, _argument: i32) -> i32 {
    running().the_refill_itself();
    0
}

extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    let fake = running();
    // **None of it for a stage the run walks into**: 0x42e634 is called at 0x42eae7, inside the block 0x42e894
    // jumps over where the scene asked for is the rebuild one.
    if fake.image.the_next_stage_is_asked_for() {
        return 0;
    }
    // Every one of the three opens the file, so every one of them asks orb's own fork which file that is: a
    // session orb can rewind sends these to its own and the read below to the same one.
    let from = fake.opens_its_score_file(false);
    let into = fake.tries_to_open_its_score_file(true);
    fake.image.saves_its_score_file(&from, into.as_deref());
    fake.image.empties_its_card_records();
    fake.image.reads_its_card_records_out_of_the_file(&from);
    0
}

/// `MainMenu`'s own added callback, which is where the front end reads the score file for what its menus
/// offer: the read itself, since that open is the whole of what orb brackets it for.
///
/// Read and not write, as the game's own read is: 0x431330 opens with `GENERIC_READ` and `OPEN_EXISTING`, so
/// a file answered here is a file left exactly as it was.
extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    // **And the file's own card records laid over the ones in memory with it**, which that read does at 0x45bff3
    // — out of whichever file this open lands in, which for this one read is the game's own whatever the mode.
    // What that costs is the whole of `Th07::read_practice_scores`'s second half: a record of runs read out of a
    // file the mode does not point at.
    let fake = running();
    let from = fake.opens_its_score_file(false);
    fake.image.reads_its_card_records_out_of_the_file(&from);
    0
}

extern "C" fn stop_recording() {}

/// And its own `GameWindow::CreateGameWindow`, which is never called: no launch here goes through the game's
/// own startup, so nothing reaches the hook whose only job is the windowed flag underneath it.
extern "fastcall" fn create_game_window(_instance: *mut c_void) -> i32 {
    0
}

/// **And its `SetActiveWindow`**, which orb's rewrite calls through for a window it leaves to the game: the host
/// writes the window down as brought forward, an active window being the one in front.
unsafe extern "system" fn host_activates_a_window(window: *mut c_void) -> *mut c_void {
    running()
        .sim()
        .windows()
        .bring_forward(orb_api::Hwnd(window as usize));
    std::ptr::null_mut()
}

/// The game's own read of the pad's buttons by number, which its key config screen makes: the array
/// emptied and answered, and nothing put in it.
///
/// Nothing, because this game has no device of its own to fill it from — which is the launch the machine
/// really is, th07 finding no pad there. What goes in is what orb adds, and that is the whole subject of
/// the e2e test over it.
extern "C" fn pad_buttons() -> *mut c_void {
    let fake = running();
    fake.image.empties_the_pad_buttons();
    std::ptr::without_provenance_mut(fake.image.pad_buttons())
}

/// And its own `joyGetPosEx`, which is never called either: `Th07` declines the pad along with everything
/// else about a run, so nothing here ever reads one.
unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

/// And its own `ReplayManager::SaveReplay` and device setup, neither of which is ever called: this game is a
/// frame and nothing else — see this file's own head — so the whole of what it hands over past the frame's
/// four calls is handed over to be a valid `Originals` and for nothing more.
extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

extern "C" fn init_d3d_device() -> i32 {
    0
}

/// And its `CreateFileA`, which is what the game's own opens of its score file go through.
///
/// It writes down the name the open landed in and answers a handle that is an index into that list: a
/// file this game keeps on no disk, so what an open is for here is which of the two names orb sent it
/// to. Every open succeeds — what a read of a file that is not there answers is nothing this game is
/// asked, `Th07` reading no score file of its own.
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
    let fake = running();
    let path = unsafe { CStr::from_ptr(name.cast()) }
        .to_string_lossy()
        .into_owned();
    let mut opens = fake.opens.borrow_mut();
    opens.push(path);
    std::ptr::without_provenance_mut(opens.len())
}

/// And its `CreateWindowExA`, which is what the window the game plays in is made through — with the
/// arguments orb decided rather than the ones this game asked for.
///
/// The host makes the window, so what an e2e test can then read of it is what any window of the simulated
/// Windows has: where it is, how big, and the client area inside it — which is what orb's letterbox is
/// worked out from.
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
