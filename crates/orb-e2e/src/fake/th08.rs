//! A th08 that is not the real one: the part of the game orb has read, played over the address space
//! [`orb_core::game::th08::image`] lays out.
//!
//! **What it plays is as much of the game's part as `Th08` reads**, which is more than
//! [`th07`](super::th07)'s frame and less than [`th06`](super::th06)'s whole run: the frame, in this
//! game's own update-then-draw order, with the queue of quads its drawing fills; the supervisor, whose
//! scene the frame's jobs ask to change; the front end's three screens, which a recipe of this game walks
//! by their own numbers; the build a decide at the team select starts; and a stage with a clock, a player
//! the word moves and bullets aimed at it.
//!
//! **And a chapter over that**, `Th08::rewinds` answering true: orb's own question stands at the title,
//! a snapshot is taken per chapter and a death is answered by one, so an e2e test here can ask for the
//! mode as well as the seam — see [`a_pointdevice_run_of_th08`](crate::a_pointdevice_run_of_th08) and the
//! third of the cases `orb_real::cases_of(Game::Th08)` holds. Which is also the reason this file exists:
//! the cases asked those questions of the real game first, and cost a launch every time.
//!
//! **Where a number here is th08's it says so**, and where it is this file's own it says that: the frame's
//! own order, the chain walk's answers and the input fold are the exe's, read at the addresses written
//! beside each; the stage's script — how long a stage lasts, what it fires and how often — is this file's,
//! a laid-out stage being a stage of its own. Over [`Launch`], which is the same half of a launch every
//! other game's e2e tests run on.

use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_void};
use std::path::PathBuf;

use orb_config::Config;
use orb_core::game::Game;
use orb_core::game::th08::Th08;
use orb_core::game::th08::image::{Image, Scene, Screen};

use orb_sim::keys;

use super::{
    DRAW, Display, Launch, Launched, PRESENT, Presented, SOUND, UPDATE, WINDOW, Work, scratch,
};

/// **What this game's two chain walks answer**, read off 0x43ca50 and 0x43cb60: the count of the jobs
/// they ran, or one of three numbers a job of its own asked for.
///
/// A job's own answer is switched on — the table at 0x43cb38 for the calc list's seven cases and the one
/// at 0x43cc46 for the draw list's six — and three of those cases leave the walk with a number instead of
/// the count: case 3 leaves 1 (0x43caf7), case 4 leaves 0 (0x43caee) and case 5 leaves `-1` (0x43cb00).
/// What the frame does with them is [`FRAME_LEFT`]'s doc.
///
/// [`CHAIN_CARRIED_ON`] is the count of one job, which is what a walk of this laid-out game runs: its
/// whole update is one job, orb standing in front of the walk rather than of an element.
const CHAIN_CARRIED_ON: i32 = 1;
pub const CHAIN_LEFT: i32 = 0;
pub const CHAIN_FAILED: i32 = -1;

/// And what the whole frame answers, which the message pump reads: nothing to be called again, and the
/// two above nothing to leave.
///
/// Read off 0x441f5f through 0x441f8e and 0x44204c: the calc walk's answer is kept, and a **0** from it
/// makes the frame answer 1 while a **-1** makes it answer 2. So the walk's two ways out become the
/// frame's two, which is th07's mapping at th08's addresses.
pub const FRAME_KEPT_RUNNING: i32 = 0;
pub const FRAME_LEFT: i32 = 1;
pub const FRAME_FAILED: i32 = 2;

/// Which key is which bit of the word this game's own read folds the keyboard into, as its defaults have
/// it — the seven an e2e test here presses.
///
/// Every bit read off the fold at 0x43d970: `Z` gives 0x1 at 0x43dbd7, `X` 0x2 at 0x43dbf9, `VK_SHIFT`
/// 0x4 at 0x43dc1d, `VK_ESCAPE` 0x8 at 0x43dc41, and the four arrows 0x10, 0x20, 0x40 and 0x80 at
/// 0x43dacf through 0x43db3d. `VK_RETURN`'s 0x1000 is in because the decide mask holds it — see
/// [`MENU_DECIDE`] — and the skip's 0x100 is out, no e2e test here pressing it.
const MAP: [(u8, u16); 8] = [
    (keys::Z, BUTTON_SHOT),
    (keys::X, BUTTON_BOMB),
    (SHIFT, BUTTON_FOCUS),
    (keys::ESCAPE, BUTTON_MENU),
    (keys::UP, BUTTON_UP),
    (keys::DOWN, BUTTON_DOWN),
    (LEFT, BUTTON_LEFT),
    (RIGHT, BUTTON_RIGHT),
];

/// The bits of that word this file names, as the folds above write them.
const BUTTON_SHOT: u16 = 0x0001;
const BUTTON_BOMB: u16 = 0x0002;
const BUTTON_FOCUS: u16 = 0x0004;
const BUTTON_MENU: u16 = 0x0008;
const BUTTON_UP: u16 = 0x0010;
const BUTTON_DOWN: u16 = 0x0020;
const BUTTON_LEFT: u16 = 0x0040;
const BUTTON_RIGHT: u16 = 0x0080;

/// **The mask every screen of this game's front end tests against last frame's to find a decide**:
/// `0x1001` at 105 sites, the shot button and return.
///
/// This file's own literal, for the reason `th08::image`'s module doc gives: a mask shared between the
/// writer and the reader is silent when wrong.
///
/// **And the cancel's**, `0xa` at 74 sites, which the title alone reads here: what a cancel does on the
/// other screens this game plays the part of has not been read, so a press of it does nothing there rather
/// than something guessed at.
const MENU_DECIDE: u16 = 0x1001;
const MENU_CANCEL: u16 = 0x000a;

/// The two arrow keys and the modifier `orb_sim::keys` does not name, orb's own menus reading none of the
/// three — which is what makes an e2e test pressing `Z` at a question of orb's and this game reading its
/// shot key the same key by construction.
const LEFT: u8 = 0x25;
const RIGHT: u8 = 0x27;
const SHIFT: u8 = 0x10;

/// How many frames a launch left to itself spends before its title menu is up: the real game unpacks
/// `th08.dat`, makes its window and reads `score.dat` first, and a recipe's first press finds the title a
/// moment later.
///
/// This file's own number, as th07's is.
const TITLE_COMES_UP_AFTER: u32 = 60;

/// **How many updates the title counts with nothing pressed before it starts the attract demo**: `0x5dc`,
/// the compare at 0x467ac4 against the count 0x467a9c steps and 0x467aba zeroes where the input word is
/// anything at all. The game's own number.
const THE_TITLE_WAITS: u32 = 0x5dc;

/// **The team select's phase its decide is read in, and the settled count it goes into it on**: 0x46a939
/// writes the 1 on the update 0x46a92d reads 8. This file's own literals, for the reason the module doc
/// gives.
const TEAM_PHASE_CHOOSING: i32 = 1;
const TEAM_SETTLES_AT: i32 = 8;

/// **The difficulty the attract demo starts on**: Easy, which is what a watched attempt of the real game
/// read the frame the demo took it — `diff=0` where every frame before read 1.
const DIFFICULTY_OF_THE_DEMO: i32 = 0;

/// **The items of the title menu an e2e test here points at**, by the indices `Th08::menu_pointed_at` reads
/// out of the front end's `+0x0`: a run is behind item 0, and the item after it is what says a cursor moved.
pub const TITLE_ITEM_START: i32 = 0;
pub const TITLE_ITEM_EXTRA: i32 = 1;
/// And the one after it, the spell practice, which the title leaves out with the Extra's where its records
/// have no clear — see [`Records::lights`].
pub const TITLE_ITEM_SPELL_PRACTICE: i32 = 2;
/// And the first item a title with nothing cleared steps down to from the first, the stage practice.
pub const TITLE_ITEM_PRACTICE: i32 = 3;
/// And the item the game's own replays are behind, which a launch of the real game decided on and read
/// `front end: screen 7, cursor on item 4` straight after.
pub const TITLE_ITEM_REPLAYS: i32 = 4;
/// And the last, which ends the game and is where a cancel at the title puts the cursor (0x4681f3).
pub const TITLE_ITEM_QUIT: i32 = 8;

/// How many items each screen of the walk has a cursor over, so that a direction pressed at one wraps
/// where the game's own list wraps: the title's nine, the four difficulties a whole run offers, the
/// three teams the select lists, and the one replay the list here holds.
///
/// **The title's is the game's own**: 0x467960 hands 0x47030b a 9, which takes up below nought to 8
/// (0x47037a through 0x470386) and down at nine to nought (0x4703df through 0x470416). The rest are this
/// file's own counts — what a walk needs of a list is a cursor with an item under it.
const TITLE_ITEMS: i32 = 9;

/// **The updates the title builds itself over, and the count its decide is read from**: one item's sprite
/// an update while its count at `+0x1442c` is below [`TITLE_ITEMS`] (0x4678ce through 0x46792f), the phase
/// its items are read in written on the tenth with the count put back to nothing (0x467934 through
/// 0x46794f), the count stepped at every update's tail (0x468333), and the decide and the cancel read only
/// once it has reached ten (0x467cac). So neither is read on the title's first nineteen updates, and a
/// direction is from the tenth on — 0x47030b running ahead of that gate.
const TITLE_PHASE_BUILDING: i32 = 0;
const TITLE_PHASE_ITEMS: i32 = 1;
const TITLE_READS_A_DECIDE_AT: i32 = 10;
const DIFFICULTIES_OFFERED: i32 = 4;
const TEAMS: i32 = 3;
const REPLAYS_LISTED: i32 = 1;

/// **The difficulty the select comes up on**, which is the one a run this file's front end starts is
/// played on: Normal, the second of the five this game's own table names.
///
/// Public because an e2e test about such a run holds what it reads against it — and because
/// `orb_real::checks::a_normal_mode_run_of_th08_played_without_a_bomb` asks the real game for the same
/// number.
pub const DIFFICULTY_NORMAL: i32 = 1;

/// And the stage such a run starts in, counted from nothing the way this game counts it.
pub const FIRST_STAGE: i32 = 0;

/// **The run's own numbers a stage of this laid-out game starts with**: the lives and bombs this game's
/// own front end puts in place, a shot part way up, and a score with digits in it.
///
/// Public because they are what an e2e test reading a run's numbers back holds them against.
pub const LIVES: f32 = 2.0;
pub const BOMBS: f32 = 3.0;
pub const POWER: f32 = 96.0;
pub const SCORE: u32 = 1_234_560;
pub const POINT_ITEMS: u32 = 12;

/// The seed the first stage of a launch is built out of, so that one is built out of a seed like every
/// other — this file's own number, the generator's word being sixteen bits of whatever the game has drawn.
const A_SEED: u16 = 0x4d2f;

/// The field a stage of this laid-out game is played in and where its build puts the player, in the units
/// `th08::sight` reads them in: this game's 384x448 field measured from its own corner, with the player
/// low in the middle of it.
///
/// The same numbers `th08::image` lays out, named again here because what moves the player is this file's
/// and what clamps it has to be the field the game's own tests clip against.
const FIELD: (f32, f32) = (384.0, 448.0);
/// Public because an e2e test about a stage holds the player it started against it.
pub const PLAYER_STARTS_AT: (f32, f32) = (192.0, 384.0);

/// **The four speeds one frame of holding a direction moves the player by**, as the active team member's
/// own `.sht` holds them and one launch of the real game measured them: straight, diagonal, focused
/// straight and focused diagonal.
///
/// The same four `th08::image` writes into the spec, for the same reason the field is written twice: what
/// reads them is orb, and what moves the player here is this file.
const A_STEP: (f32, f32, f32, f32) = (4.0, 2.83, 2.0, 1.41);

/// **How long a stage of this laid-out game lasts**, and what it fires: a bullet aimed at where the player
/// is, from the top of the field, every so many frames from the first — slow enough that a player who
/// steps aside is missed and one who does not is hit.
///
/// This file's own script. What a stage that shot nothing would be is a stage the rules have nothing to do
/// in, and what the rules do is the whole of what a case about a run played asks.
///
/// **The first shot where the real stage 1 fires its first**, and not a second in: the game's own replay of
/// a Normal run has stage 1's first bullet out on frame 483, stage 3's on 430 and 4b's on 381, and every
/// case's walk into a run ends in three hundred frames that press nothing. Fired at 60, this stage's first
/// bullet reached a player standing on its spawn inside that wait, and a chapter the real game has never
/// retried there was retried here.
const FIRST_BULLET_AT: u32 = 480;
const A_BULLET_EVERY: u32 = 120;
const BULLETS_FROM: (f32, f32) = (192.0, 0.0);
const BULLET_SPEED: f32 = 2.0;
const BULLET_SIZE: f32 = 8.0;
/// How many of the manager's slots this stage uses, from the first.
const BULLET_SLOTS_USED: usize = 16;

/// **What a conversation costs the run's clock**, which is the number instruction 148 of a real stage's
/// script adds to it: `0x708` at 0x41d6cd, on the update the line goes up. The 1,800 keys it skips are keys
/// nothing is ever written down against, and the frames the conversation itself runs are written down
/// against the number it landed on — see `resume::Again` and `Th08`'s own `STAGE_FRAMES`.
const THE_CLOCK_A_LINE_COSTS: u32 = 1800;

/// **How often the enemy manager checks the run's seal**: where its own timer is a multiple of this — the
/// `push 0xc8` at 0x42de49 — and so on a stage's first frame too, the timer starting from nothing.
const THE_SEAL_IS_CHECKED_EVERY: u32 = 200;

/// **The player's own update, by the state its byte is in** — `asm.md`'s *The player*, off the exe:
///
/// - the count under it at `[player+0xe2af4+0x8]` goes up by one an update in states 0, 1 and 2 (0x44d409)
///   and down by one in state 3 (0x44d35c);
/// - **a hit, state 2, stands until that count has reached [`A_COUNT_RUNS_TO`]**: then 0x44cbf0 takes the life
///   (0x44d100) and puts the player back at the start in state 1 with the count at nothing (0x44d012 through
///   0x44d07a);
/// - **state 1 does the same**, and then 0x44d180 puts the player in state 3 with [`UNTOUCHABLE_FOR`] under it
///   (0x44d252, 0x44d29f) — which at a stage's start, whose build leaves [`A_BUILD_LEAVES`], is the first update
///   the player runs;
/// - **state 3 runs its count down**, and on the update it reaches nothing the state goes back to 0 (0x44d39f);
/// - and **states 1 and 2 move nothing, shoot nothing and focus nothing** (0x44c4b5 through 0x44c4cb).
///
/// So a hit is 31 updates of state 2, 30 of state 1 and then 239 of state 3; a stage's start, state 1 for as
/// long as its build is held and then 240 updates of state 3.
const PLAYER_PLAYING: u8 = 0;
const PLAYER_COMING_IN: u8 = 1;
const PLAYER_HIT: u8 = 2;
const PLAYER_UNTOUCHABLE: u8 = 3;
const A_COUNT_RUNS_TO: i32 = 0x1e;
const UNTOUCHABLE_FOR: i32 = 0xf0;
const A_BUILD_LEAVES: i32 = 120;

/// One bullet of that stage's own: which slot it is written down in, where it is and what it moves by a
/// frame.
#[derive(Clone, Copy)]
struct Bullet {
    slot: usize,
    at: (f32, f32),
    step: (f32, f32),
}

/// **The stage words a run of this game's first team goes through**, which is six stages out of a table
/// of nine and not the first six of it: this game splits its fourth stage into `stage4a` and `stage4b` and
/// its last into `stage6` and `stage7`, so the words a run walks skip one of each pair.
///
/// **The chooser is 0x43c4b3**, `__thiscall` on the game manager, which the run manager calls at 0x439e20
/// as a stage ends: 0 goes to 1, 1 to 2, 2 to either 3 or 4 by the character — the byte table at 0x43c5d5
/// indexed by `[0x164d0b1]` and the jump table at 0x43c5c5 behind it, where the first team's 0 writes **4**
/// — 3 and 4 both go to 5, and 5 goes to 7 where `(0x164d0b4 >> 0xb) & 3` is anything and to **6** where it
/// is nothing. The run manager ends the run instead of choosing where the stage is already 6, 7 or 8
/// (0x439d4d).
///
/// This file's own literals, for the reason the module doc gives.
const A_RUNS_STAGES: [i32; 6] = [0, 1, 2, 4, 5, 6];
/// **The two last stages and the Extra's**, by their words: Eirin's, Kaguya's — which 5 goes to where the
/// records hold Eirin's stage passed, see [`Records`] — and the Extra's, which its team select writes as
/// the difficulty and four (0x46ac08).
const EIRINS_STAGE: i32 = 6;
const KAGUYAS_STAGE: i32 = 7;
const THE_EXTRAS_STAGE: i32 = 8;
/// And the Extra's difficulty, the fifth of the five `0x4c7fd4` names.
pub const THE_EXTRAS_DIFFICULTY: i32 = 4;
/// And how many stage words a practice's select lists, which is where its cursor wraps (0x46b045).
const STAGES_A_PRACTICE_LISTS: i32 = 8;

/// **The stage word a run goes to after `stage`**, and nothing where the run ends there instead — see
/// [`A_RUNS_STAGES`], and [`Records`] for what sends 5 to Kaguya.
fn the_stage_after(stage: i32, to_kaguya: bool) -> Option<i32> {
    match stage {
        5 if to_kaguya => Some(KAGUYAS_STAGE),
        EIRINS_STAGE | KAGUYAS_STAGE | THE_EXTRAS_STAGE => None,
        _ => {
            let at = A_RUNS_STAGES.iter().position(|word| *word == stage)?;
            A_RUNS_STAGES.get(at + 1).copied()
        }
    }
}

/// **What this game's `CLRD` records say that a walk of its front end turns on**, kept as the three facts
/// and not as the records: orb reads none of the records, so what this file owes is what they decide —
/// which items the title draws, and which last stage a run goes to. See `asm.md`'s *What a clear writes
/// into a `CLRD` record*.
///
/// - `eirins_stage_passed`: bit 6 in a Normal, Hard or Lunatic `+0x16` word, which the run manager ORs in
///   as stage word 6 ends (0x439c50 through 0x439d02) and 0x439856 asks for at 0x433b47 — and a run after
///   it goes from 5 to 7.
/// - `cleared`: `0x8000` in a `+0x16` word, which the run manager ORs in as a last stage ends (0x439eaa
///   through 0x439ed5) and 0x43c2c5 lights the title's third item, the spell practice, out of.
/// - `extra_lit`: `0x4000` in a `+0xc` word, which only Kaguya's ending writes (0x429a00 through 0x429c07,
///   the stage word 7) and 0x43c1e9 lights the second, the Extra, out of.
///
/// - `stages_passed`: the stage bits of that `+0x16` word, `1 << stage` for each stage word a run has
///   ended, which a stage practice's own select steps its cursor over (0x46afb1 through 0x46b061) — with the
///   first stage offered where the word is nought.
///
/// **In a file of this file's own format** — see [`Records::bytes`] — under the name the game's own open of
/// `score.dat` landed on, which is what makes a record a Pointdevice run writes one the title does not read.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct Records {
    eirins_stage_passed: bool,
    cleared: bool,
    extra_lit: bool,
    stages_passed: u16,
}

impl Records {
    fn bytes(self) -> [u8; 5] {
        let [low, high] = self.stages_passed.to_le_bytes();
        [
            u8::from(self.eirins_stage_passed),
            u8::from(self.cleared),
            u8::from(self.extra_lit),
            low,
            high,
        ]
    }

    fn read(bytes: &[u8]) -> Self {
        let set = |at: usize| bytes.get(at) == Some(&1);
        let byte = |at: usize| bytes.get(at).copied().unwrap_or(0);
        Records {
            eirins_stage_passed: set(0),
            cleared: set(1),
            extra_lit: set(2),
            stages_passed: u16::from_le_bytes([byte(3), byte(4)]),
        }
    }

    /// Whether a stage practice's select offers stage word `stage`.
    fn offers_a_practice_of(self, stage: i32) -> bool {
        let passed = if self.stages_passed == 0 {
            1
        } else {
            self.stages_passed
        };
        u32::try_from(stage).is_ok_and(|stage| stage < 16 && passed & (1 << stage) != 0)
    }

    /// Whether the title draws its item `item` — the second and the third being the two it can leave out
    /// (0x46799c, 0x467977).
    fn lights(self, item: i32) -> bool {
        match item {
            TITLE_ITEM_EXTRA => self.extra_lit,
            TITLE_ITEM_SPELL_PRACTICE => self.cleared,
            _ => true,
        }
    }
}

/// **The Last Spell the last of those stages ends on**, which every run that beats that stage's boss meets:
/// its norm of time orbs is nought, so Eirin answers with `禁薬「蓬莱の薬」` whatever a run picked up — card
/// 144, the index of its own `CATK` record, which is what a whole run of the real game took. Up for
/// [`THE_LAST_SPELL_LASTS`] frames here, and gone [`AFTER_THE_LAST_SPELL`] frames before the stage is.
const THE_LAST_SPELL: usize = 144;
const THE_LAST_SPELL_LASTS: u32 = 600;
/// **And the frames between that card going and the ending**, which on the real game are the boss going
/// down and the conversation after her fight: a card held up to the ending's own frame was one a
/// whole-run check read as never over, and so never taken. This file's own number — what it has to be
/// is more than nothing.
const AFTER_THE_LAST_SPELL: u32 = 120;

/// **The stage's own boss, up for that many of a stage's last frames** where an e2e test said how long one
/// lasts: in the enemy manager's first slot, at the top of the field's middle and with a life that lasts
/// the fight. **What a recipe waits for her by is her arriving**, `play_until: { boss: true }`, which a
/// real stage answers at its end — and a laid-out stage with none walked on through every stage after it
/// to the end of the run, the chapter it was asked to be left in never reached. Long enough to hold
/// Eirin's Last Spell and the frames after it, which are hers. This file's own numbers.
const THE_BOSS_FIGHTS_FOR: u32 = 900;
/// **And her fight a real boss's: a nonspell, a card, and the nonspell after it**, the attack timer put back to
/// nought as each attack begins — which is what orb reads a fight's chapters by, the card ending saying nothing
/// of its own. A fight with no card in it had no nonspell after a card to die in, and
/// `a_death_in_the_nonspell_after_a_card_answered_by_that_nonspell` nothing to be about. Card 3 is the one
/// stage 1's boss put up first on the real game, `spell=3`; the frames are this file's own, each past the
/// shortest a chapter may be.
const HER_CARD: usize = 3;
const HER_CARD_AFTER: u32 = 240;
const HER_CARD_LASTS: u32 = 240;
const THE_BOSSS_SLOT: usize = 0;
const THE_BOSS_STANDS_AT: (f32, f32) = (192.0, 96.0);
const THE_BOSSS_LIFE: i32 = 60_000;

/// **And the ending the last stage's end goes into**, scene 9, for as many updates as this: the real one's
/// text and staff roll run tens of thousands, which orb's own skip runs out, and what this file owes is the
/// scene and its end — the screen a run ends at after it.
const THE_ENDING_LASTS: u32 = 300;

/// **The updates the pause menu closes over** once a menu press has put it in its phase 4: 0x4041d5 compares
/// its timer against `0x14` before 0x4041e4 lets the run go.
const THE_PAUSE_CLOSES_OVER: u32 = 0x14;

/// **The phases of the screen a run ends at that this file walks**, by the numbers 0x4584bc switches on:
/// the name entry, the score after it and the pause before the offer, the offer of a replay and the three
/// steps after its first answer, and the way out. See `Th08`'s `RESULT_PHASE` for where each was read —
/// this file's own literals, for the reason the module doc gives.
const RESULT_TYPING_A_NAME: i32 = 9;
const RESULT_SHOWING_THE_SCORE: i32 = 0xf;
const RESULT_BEFORE_THE_OFFER: i32 = 0x10;
const RESULT_OFFERING_A_REPLAY: i32 = 0xa;
const RESULT_SAVING_A_REPLAY: std::ops::RangeInclusive<i32> = 0xc..=0xe;
const RESULT_LEAVING: i32 = 2;
/// **And the updates each waits**, on the timer every update of that screen steps: the entry's ten before
/// it takes a key (0x456493), the score's ninety before a decide (0x45782c), the pause's thirty and the
/// nine it hands the offer (0x45788c, 0x457895), the offer's twelve before a key (0x456b0e), and the way
/// out's twenty (0x458589).
const THE_ENTRY_WAITS: i32 = 0xa;
const THE_SCORE_WAITS: i32 = 0x5a;
const THE_PAUSE_LASTS: i32 = 0x1e;
const THE_PAUSE_HANDS_THE_OFFER: i32 = 9;
const THE_OFFER_WAITS: i32 = 0xc;
const THE_WAY_OUT_LASTS: i32 = 0x14;
/// **The entry's grid**: `0x60` items sixteen to a row, the space at 0x5e and the item that ends the
/// entry at 0x5f, and eight characters a name — 0x4564e5 through 0x4566c2 and 0x456764 through 0x4567c9.
/// The characters the other 0x5e items stand for are this file's own, the printable ASCII from `!` on:
/// orb reads none of them, and what they have to be is distinct.
const THE_GRID: i32 = 0x60;
const A_ROW_OF_IT: i32 = 0x10;
const THE_SPACE_ITEM: i32 = 0x5e;
const THE_END_ITEM: i32 = 0x5f;
const A_NAMES_CHARACTERS: i32 = 8;

/// **The thread a stage's own track is streamed on**, as `PlayBGM`'s `CreateThread` would have written it
/// down: this file's own number, and what it has to be is something rather than nothing — orb reads it to
/// know which thread a restore leaves running.
const A_MUSIC_THREAD: u32 = 0x3b09;

/// **The tracks the laid-out `thbgm.dat` holds, and the three every laid-out stage names**: the stage's
/// own first and its boss's second, in the shapes the real names take — a record the file's name alone,
/// the stage's own file a name with a directory and an extension the load writes `wav` over (0x448019),
/// and a space for a slot the stage has no track in (0x43b763). The names are this file's own.
const TRACKS: [&CStr; 2] = [c"th08_01.wav", c"th08_02.wav"];
const A_STAGES_TRACKS: [&CStr; 3] = [c"bgm/th08_01.mid", c"bgm/th08_02.mid", c" "];
/// Which of [`TRACKS`] a stage's boss is fought to, which the conversation before her plays on the real
/// game (0x434dce) and an e2e test plays here.
pub const THE_FIGHTS_TRACK: usize = 1;

/// **The stream's own numbers as a stage's track starts**: a chunk, the buffer four of them round, and the
/// whole of a track before its loop. This file's own — what they have to be is a countdown a refill moves.
const A_MUSIC_CHUNK: u32 = 0x4000;
const CHUNKS_THE_BUFFER_HOLDS: u32 = 4;
const A_TRACK_BEFORE_ITS_LOOP: u32 = 0x40_0000;

/// The class th08 registers its window with, which is th06's and th07's too: `BASE`. What orb recognises
/// the window it plays in by — see `orb_core::window::is_game_class`.
const WINDOW_CLASS: &CStr = c"BASE";

/// And a class of some other window, which is what an ask orb hands straight on looks like: any name but the
/// one above — see [`Fake::creates_another_window`].
const OTHER_WINDOW_CLASS: &CStr = c"th08-notify";

/// And the ask itself, which is what a pass-through leaves standing and what orb rewrites every argument
/// of: a position, a size and a style with a caption on it.
///
/// This file's own three, and deliberately nothing like what orb lays out — so that a window read back at
/// these numbers is one orb decided nothing about.
pub const ASKED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x1000_0000;
pub const ASKED_AT: (i32, i32) = (17, 23);
pub const ASKED_SIZE: (i32, i32) = (646, 505);

/// **Where the supervisor's thread has got to with a run's build** — see [`Fake::its_loader_goes_on`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Build {
    /// No build, or the one asked for not yet begun.
    Idle,
    /// The stage built and its jobs on the chain, and the build's last wait not yet over: `[gm+0x3c]` under
    /// what it stands for.
    WaitingOutItsFrames,
    /// The count up, and the thread asleep for this many more frames before it wakes and clears its word.
    Asleep(u32),
}

/// A th08 laid out, with orb attached to it.
pub struct Fake {
    image: Image,
    launch: Launch,
    /// What this game's chain walk answers. [`CHAIN_CARRIED_ON`] for the whole of a launch here, no e2e
    /// test over this game asking it to leave — a `Cell` all the same, because what reads it is a bare
    /// `extern` function with the ABI's arguments and nothing else.
    answers: Cell<i32>,
    /// How many times the game has been made to tell its device the fog is off — see
    /// [`puts_the_fog_out`].
    fog_told: Cell<u32>,
    /// Every present the game's device was asked for — see [`device_present`].
    presents: RefCell<Vec<Presented>>,
    /// Whether orb stands in front of this game's keyboard read, and the word that read last answered.
    reads_the_keys: bool,
    word: Cell<u16>,
    /// How many frames this launch has run, for the title menu that comes up on its own — see
    /// [`Fake::comes_up_at_its_title`].
    frames_since_launch: Cell<u32>,
    /// **Whether the run being played is one this game's own front end started**, which is the run that
    /// gets the stage's script, its bullets and a player the word moves. A run an e2e test placed with the
    /// helpers gets none of that, every such test driving its stage itself.
    on_its_own: Cell<bool>,
    /// How long a stage of this game lasts before the next one is built, where an e2e test says. Nothing
    /// by default, which is a stage that never ends.
    stage_frames: Cell<Option<u32>>,
    /// **The run's clock as the stage now played was built**, which is what that length is counted from.
    stage_built_at: Cell<u32>,
    /// How many updates the ending the run is in has left — see [`Fake::the_run_is_over`].
    ending_left: Cell<u32>,
    /// **Which frame of a stage a conversation begins on and how many updates it lasts**, where an e2e test
    /// asks for one — see [`Fake::has_a_conversation`]. Nothing by default, which is a stage with no line in
    /// it: a real stage has three and every one of them holds the run's clock while the run plays on, so a
    /// test that says nothing here gets a stage whose clock counts every update.
    a_conversation_at: Cell<Option<(u32, u32)>>,
    /// **Which frame of a stage the clock is put 1,800 on with no line**, where an e2e test asks — see
    /// [`Fake::skips_its_clock_at`].
    a_skip_at: Cell<Option<u32>>,
    /// **The pair the player's own job read on the last update it ran**: the word the run acts on and the one it
    /// acted on, as they stood at priority 9 — see [`Fake::what_the_player_read`].
    the_player_read: Cell<(u16, u16)>,
    /// **The enemy manager's own timer**, `[enemies+0x9dced0]`, which its update steps last and which the run's
    /// seal is checked on every two hundred of — see [`THE_SEAL_IS_CHECKED_EVERY`].
    enemy_frames: Cell<u32>,
    /// And how many updates the one now up has left, 0 being none up.
    conversation: Cell<u32>,
    /// **Every file this game has asked `CreateFileA` for, by the name the call landed on** — the one orb's
    /// hook handed on, which is what says which of the two score files an open of this game's went to.
    opens: RefCell<Vec<String>>,
    /// The bullets it has out, and the frames a hit has left before it is a death.
    bullets: RefCell<Vec<Bullet>>,
    /// Whether the stage's boss has arrived — see [`THE_BOSS_FIGHTS_FOR`].
    the_boss_is_up: Cell<bool>,
    /// **How many updates the pause menu has been closing for**, where it is — see [`Fake::its_pause_menu`].
    the_pause_closing: Cell<Option<u32>>,
    /// Whether the stage's own script fires at the player — see [`Fake::fires_at_the_player`].
    fires: Cell<bool>,
    /// The seed the last build settled on for its stage, which is where the next one comes from.
    seed: Cell<u16>,
    /// The item the title menu was last left on, which is the item the front end comes back on when the
    /// run behind it is over.
    left_the_title_on: Cell<i32>,
    /// **Whether the front end's element is on the chain**, which is what runs its screens — whatever the
    /// scene is. It comes off where its own update answers nought, which is a screen's own start of a run
    /// and nothing else: see [`Fake::the_front_end_leaves_the_chain`].
    front_end_on_the_chain: Cell<bool>,
    /// And how many updates the title has had with nothing pressed — see [`THE_TITLE_WAITS`].
    title_idle: Cell<u32>,
    /// **What the records the game has in memory say**, which is what its title lights its items from and
    /// what its chooser routes a run by — see [`Records`].
    records: Cell<Records>,
    /// **The name the last entry was left holding**, which the next one comes up holding — the `LSNM`
    /// record the real screen fills its field out of on its first update (0x4563d0).
    last_name: Cell<[u8; 8]>,
    /// Whether the load the supervisor's thread is on is a run starting rather than a stage carrying one
    /// on — the branch 0x43aaf4 takes at 0x43aafd, which the build on the thread puts the run's starting
    /// numbers in on.
    the_load_starts_a_run: Cell<bool>,
    /// **How many frames the supervisor's thread sleeps on after the count it waits for is up**, where an e2e
    /// test asks — see [`Fake::goes_on_building_for`]. Nought by default, which is a thread that wakes on the
    /// frame the count comes up.
    goes_on_building_for: Cell<u32>,
    /// **Where that thread has got to** — see [`Fake::its_loader_goes_on`].
    build: Cell<Build>,
    /// Whether the driver behind this game's device refuses to stretch a present — see
    /// [`Fake::refuses_to_stretch_on_a_present`].
    refuses_to_stretch: Cell<bool>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory.
    _installed: orb_api::Installed,
}

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into — plain `extern` functions with
    /// nothing but the ABI's arguments, so where the real game would reach its own globals this reaches
    /// the game.
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
}

/// **The facts about the machine a launch is declared with** beside its display: the language its windows
/// are in, whether it says where its panel is, and whether the game has made its device yet.
struct Host {
    ui_language: Option<u16>,
    monitor: bool,
    device: bool,
}

impl Host {
    /// What an ordinary launch has: the machine's own language, a panel to lay a window out against, and a
    /// device made before orb is attached.
    const ORDINARY: Self = Self {
        ui_language: None,
        monitor: true,
        device: true,
    };
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

    /// And the same launch with a setting of its own answered — the recipe a bot plays to, or the screen
    /// a client of a given shape gets the game in.
    ///
    /// A closure rather than a `Config`, so that what an e2e test says is the one thing it is about and
    /// everything else is what a launch in a directory with no `orb.yaml` brings.
    pub fn attach_configured(
        display: Display,
        name: &str,
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, Host::ORDINARY, name, &[], work, settings)
    }

    /// And on a host that will not say where its panel is — **no monitor declared**, which is a window orb has
    /// nothing to lay out against: see `the_windows_orb_leaves_alone_of_th08.rs`.
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

    /// And in a directory that already holds `left`, each file at its path inside the directory — what an
    /// earlier launch left there, **found at the attach and not put in afterwards**: what a launch has to offer
    /// it reads as it comes up.
    pub fn attach_finding(
        display: Display,
        name: &str,
        left: &[(PathBuf, Vec<u8>)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, Host::ORDINARY, name, left, work, settings)
    }

    /// And on a machine whose own windows are in the language `ui_language` names — `orb_sim::langid` — with
    /// the launch otherwise the one [`Fake::attach_configured`] makes.
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

    /// And laid out with **no device**, which is what a process is before its Direct3D setup has run: orb
    /// attaches there in a real launch and finds the device afterwards, through its hook over
    /// `GameWindow::InitD3dDevice` at 0x435810. A game that had one already is a game that hook is never
    /// reached in — see [`finds_its_device`](Fake::finds_its_device), which is the other half of this.
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

    /// Every host fact a launch is declared with, in one place: the wrappers above are what an e2e test
    /// calls, each naming the one it is about and passing the rest as what an ordinary launch has.
    fn attach_declaring(
        display: Display,
        host: Host,
        name: &str,
        left: &[(PathBuf, Vec<u8>)],
        work: Work,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        let dir = scratch(name);
        // The simulated Windows first, `orb.yaml` being a file of orb's own and so read through the file
        // seam — the same order th06's and th07's attach have.
        let image = Image::laid_out_seeded(display.seed);
        let installed = image.enter();
        // And the language its windows are in, before the attach settles which language orb's screens are.
        if let Some(ui_language) = host.ui_language {
            image.sim().set_ui_language(ui_language);
        }
        // What the log says is read after what has been said is written — see `orb_sim::Log::waits_first`.
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
        // Kept before the attach takes the rest of it: what plays the run is installed after the attach —
        // the log being what a refused recipe has to speak through — and by then the config has moved.
        let recipe = config.autoplay.clone();
        // **No `font.ttf` beside this game's exe**, th08 keeping its fonts inside `th08.dat` as th07 does:
        // the face is there for the harness's own device and gone before orb reaches for one.
        let font = dir.join("font.ttf");
        image.sim().text().install_font(&font);
        let launch = Launch::new(
            image.sim(),
            dir,
            display.seed,
            super::orbs_own_loop(&config, the_game_this_is()),
        );
        image.sim().text().remove_font(&font);
        // The device where the launch has one already, and a null where it is attached before the game's
        // own setup has made one — which is every real launch.
        image.shows_through(
            if host.device {
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
        image.hands_over_the_runs_checksum(sums_the_run as *const () as usize);
        image.hands_over_the_supervisors_update(supervisor_update as *const () as usize);
        // Where the game is installed, which is where orb writes its log.
        image.sim().set_host_exe(launch.dir().join(EXE));
        // The display, in front of orb being attached: `configure` reads the desktop's own rate before
        // there is a window to ask about, and a rate written down after that would be read a second late.
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
        // attach is where orb says this process reads sizes as real pixels. Declared for every launch but the
        // one that asks for a host with no panel to read.
        if host.monitor {
            sim.windows()
                .set_monitor(orb_sim::Monitor::scaled(), orb_sim::Frame::SOME);
        }

        let fake = Box::new(Self {
            image,
            launch,
            answers: Cell::new(CHAIN_CARRIED_ON),
            fog_told: Cell::new(0),
            presents: RefCell::new(Vec::new()),
            reads_the_keys: the_game_this_is().hooks().input.is_some(),
            word: Cell::new(0),
            frames_since_launch: Cell::new(0),
            on_its_own: Cell::new(false),
            stage_frames: Cell::new(None),
            stage_built_at: Cell::new(0),
            ending_left: Cell::new(0),
            a_conversation_at: Cell::new(None),
            a_skip_at: Cell::new(None),
            the_player_read: Cell::new((0, 0)),
            enemy_frames: Cell::new(0),
            conversation: Cell::new(0),
            opens: RefCell::new(Vec::new()),
            bullets: RefCell::new(Vec::new()),
            seed: Cell::new(A_SEED),
            // The item that starts a run, which is where a launch of the real game reads the cursor
            // before anything has been left: `front end: screen 0, cursor on item 0` on the first frame
            // of every case's trace.
            left_the_title_on: Cell::new(TITLE_ITEM_START),
            front_end_on_the_chain: Cell::new(false),
            title_idle: Cell::new(0),
            records: Cell::new(Records::default()),
            last_name: Cell::new(*b"        "),
            the_boss_is_up: Cell::new(false),
            the_pause_closing: Cell::new(None),
            fires: Cell::new(true),
            the_load_starts_a_run: Cell::new(false),
            goes_on_building_for: Cell::new(0),
            build: Cell::new(Build::Idle),
            refuses_to_stretch: Cell::new(false),
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
                    tops_up_the_music,
                    stops_the_music,
                },
            )
        };
        // **Whoever is playing this run, installed where a real launch installs it**: `orb`'s attach
        // reaches `orb_bot::install` and nothing of orb-core does, so an e2e test over a bot has to reach
        // it too. After the attach, as a real launch's is: what a refused recipe has to say goes in the
        // log, and the log is not there to be written to until the attach has settled it.
        if let Some(recipe) = &recipe {
            unsafe { orb_bot::install(recipe, the_game_this_is()) };
        } else {
            // **And a launch with no recipe takes the last one's bot out**, for th06c's fake's reason: the bot
            // and its hook are the process's, and a test that plays two launches in one had a keyboard walk
            // say `autoplay: step 14 of 14 closes the game` on every frame, the recipe being the launch
            // before's — `the_run_of_th08_left_behind`'s cancel.
            unsafe { orb_core::runtime::plays_the_run(None) };
        }
        fake.frame_takes(work);
        fake
    }

    /// This game's memory, for an e2e test that reads back what a frame of orb's did to it.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// The game creating the window it plays in, which is one `CreateWindowExA` on the class orb
    /// recognises.
    ///
    /// `GameWindow::CreateGameWindow` first, which is the call everything about this game's window is
    /// decided inside: the game reads its own configuration in there, and orb's one chance to overrule a
    /// display taken exclusively is in front of that read — see `Th08::hooks`' `create_window`.
    pub fn creates_its_window(&self) -> orb_api::Hwnd {
        orb_core::runtime::create_game_window(std::ptr::null_mut());
        self.asks_for_a_window_of_the_class(WINDOW_CLASS)
    }

    /// And a window of another class, which is an ask orb has to hand straight on: the same call on a class
    /// that is not the one the game plays in.
    pub fn creates_another_window(&self) -> orb_api::Hwnd {
        self.asks_for_a_window_of_the_class(OTHER_WINDOW_CLASS)
    }

    fn asks_for_a_window_of_the_class(&self, class: &CStr) -> orb_api::Hwnd {
        let window = unsafe {
            orb_core::window::create_window_ex_a(
                0,
                class.as_ptr().cast(),
                c"th08".as_ptr().cast(),
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
    /// A step of its own because the launch is attached before the game has a device, as a real one is.
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
        orb_core::runtime::init_d3d_device();
    }

    /// The driver behind that device refusing to stretch a present, which is what `Present` answering a
    /// failure for a destination rectangle is.
    pub fn refuses_to_stretch_on_a_present(&self) {
        self.refuses_to_stretch.set(true);
    }

    /// Every present the game's device has been asked for since the last
    /// [`forget_presents`](Fake::forget_presents), which is what an e2e test reads orb's letterbox back
    /// through.
    pub fn presented(&self) -> Vec<Presented> {
        self.presents.borrow().clone()
    }

    pub fn forget_presents(&self) {
        self.presents.borrow_mut().clear();
    }

    /// How many times the game has told its device the fog is out, which is the one render state a frame
    /// of this game sets around its drawing — see [`puts_the_fog_out`].
    pub fn times_the_fog_was_put_out(&self) -> u32 {
        self.fog_told.get()
    }

    /// **What the player's own job read on the frame just run**: the word it moved on and the one it holds that
    /// against for a bomb — which is the pair as it stood at priority 9, the replay's element at 0x11 having
    /// copied both on by the time the frame is over.
    pub fn what_the_player_read(&self) -> (u16, u16) {
        self.the_player_read.get()
    }

    /// **The word the game's own read of the keys answered on the last update it made**, which is where a press
    /// orb held back and then handed over shows up.
    pub fn word_the_input_read_answered(&self) -> u16 {
        self.word.get()
    }

    /// The state orb reads of this game, for an e2e test that asks what a frame left.
    pub fn state(&self) -> orb_core::game::State {
        unsafe { Th08.read_state() }
    }

    /// **The front end up on its title menu**, which is where a launch left to itself arrives: the object
    /// where the game's own build would have left it, the title's own screen number, and the cursor on the
    /// item a run is behind.
    pub fn comes_up_at_its_title(&self) {
        if self.frames_since_launch.get() != TITLE_COMES_UP_AFTER {
            self.frames_since_launch
                .set(self.frames_since_launch.get() + 1);
            return;
        }
        self.frames_since_launch
            .set(self.frames_since_launch.get() + 1);
        self.the_front_end_is_up_on(self.left_the_title_on.get());
    }

    /// The front end up on an item of the title, with the scene the supervisor's own switch is in while it
    /// is: the one field a run's build frees and the one it allocates, either way round.
    ///
    /// **Coming up in the phase it builds itself in, with its count at nothing** — see
    /// [`TITLE_READS_A_DECIDE_AT`] for the updates the walk then counts through before a decide is read,
    /// which is what `Th08::menu_takes_a_press` reads the same two fields for.
    fn the_front_end_is_up_on(&self, item: i32) {
        self.image.is_in_scene(Scene::FrontEnd, Scene::FrontEnd);
        // The pause the front end clears as it comes up (0x4714d0).
        self.image.the_run_is_paused(false);
        self.the_pause_closing.set(None);
        self.image.the_front_end_comes_up_on(Screen::Title, item);
        self.front_end_on_the_chain.set(true);
        self.title_idle.set(0);
        self.image.its_runs_numbers_are_freed();
        // **And the front end's own added callback, which reads the score file for what the title lights** —
        // `Th08`'s `FRONT_END_ADDED` — made where the screen comes up, through orb's bracket where a launch
        // installs it, as the game's own chain makes it.
        let landed = self.reads_what_its_title_offers();
        self.loads_its_records(&landed);
    }

    /// **The title's own read of the score file, and the name it landed on** — the front end's added
    /// callback, through orb's bracket where a launch declares it, which is where orb puts that one read on
    /// the game's own file whichever mode it is in.
    pub fn reads_what_its_title_offers(&self) -> String {
        let front = self.image.front_end_object() as *mut c_void;
        let opened = self.opens.borrow().len();
        if the_game_this_is().hooks().unlocks_read.is_some() {
            orb_core::runtime::unlocks_read_in_ecx(front);
        } else {
            unlocks_read(front);
        }
        let opens = self.opens.borrow();
        assert_eq!(opens.len(), opened + 1, "the title's read opened no file");
        opens[opened].clone()
    }

    /// **And an open of the score file anywhere else**, which is every open that follows the mode: the save
    /// every run's start makes, the result screen's and the title's after a run.
    pub fn opens_its_score_file(&self, write: bool) -> String {
        let opened = self.opens.borrow().len();
        its_score_file_opened(write);
        let opens = self.opens.borrow();
        assert_eq!(opens.len(), opened + 1, "the score file was not opened");
        opens[opened].clone()
    }

    /// **One refill of its music by the game's streaming thread**, through orb's hook over the refill as the
    /// real thread reaches it — see `Th08`'s `TOPS_UP_THE_MUSIC`.
    pub fn refills_its_music(&self) {
        orb_core::runtime::tops_up_the_music(self.image.the_music_stream(), 1);
    }

    /// **The records a clear and Kaguya's ending leave**, said rather than played: the title's every item lit,
    /// for an e2e test about the title's own walk and not about the file the records are read out of. Said
    /// after the title has come up, its coming up being what reads that file.
    pub fn its_records_light_every_item(&self) {
        self.records.set(Records {
            cleared: true,
            extra_lit: true,
            ..self.records.get()
        });
    }

    /// **The title menu up straight away**, for an e2e test that is not about the frames a launch spends
    /// before it: the same screen [`comes_up_at_its_title`](Fake::comes_up_at_its_title) puts up, without
    /// the wait.
    pub fn its_title_menu_is_up(&self) {
        self.frames_since_launch.set(TITLE_COMES_UP_AFTER + 1);
        self.the_front_end_is_up_on(TITLE_ITEM_START);
        // **Past the updates it builds itself over and the ten after them**, said rather than counted through:
        // what an e2e test placing the title asks for is a title that reads a press.
        self.image
            .the_front_end_shows(Screen::Title, TITLE_ITEM_START);
    }

    /// And a run in its first stage, placed rather than walked to — for an e2e test about a stage rather
    /// than about the front end that reaches one.
    ///
    /// Everything `Th08::read_state` reads of a run: the scene, the stage and the difficulty, the run's own
    /// numbers behind the pointer a build allocates, the kind, the seed, the player and an empty field.
    pub fn is_in_a_run(&self) {
        // Placed as though its own team select had started it, which takes the front end off — and which the
        // title came up before, so it does not come up over the run sixty frames in.
        self.frames_since_launch.set(TITLE_COMES_UP_AFTER + 1);
        self.the_front_end_leaves_the_chain();
        self.image.is_playing(FIRST_STAGE, DIFFICULTY_NORMAL);
        self.image.is_in_scene(Scene::Run, Scene::Run);
        self.image.the_run_is_its_own();
        self.the_numbers_a_run_starts_with_are_in_place();
        self.builds_the_stage();
    }

    /// **The front end taken off the chain**, which is what its own update answering nought does — a screen's
    /// own start of a run, the team select's at 0x46ac47 and the replay list's at 0x46ee4f, and the attract
    /// demo's at 0x467c54 — and its object freed with it, the chain's own deleted callback 0x471662 handing it
    /// to the allocator with the static left pointing at it.
    ///
    /// **And nothing else takes it off**: the supervisor's switch out of the front end only registers the run
    /// (0x44578d), so a run's scene written from outside is a run with the front end still running under it.
    fn the_front_end_leaves_the_chain(&self) {
        self.front_end_on_the_chain.set(false);
        self.image.the_front_end_is_freed();
    }

    /// **The numbers a run starts with**, which this game's own front end puts in place: the lives and
    /// bombs, a shot part way up, and a score with digits in it.
    ///
    /// Apart from the build, because the two are apart in the game: a stage of a run already under way
    /// carries the numbers the last one left, and putting these in at every build would be a run that
    /// starts over at each stage.
    fn the_numbers_a_run_starts_with_are_in_place(&self) {
        // The run's own clock back to nothing, which is the one thing here a stage carrying a run on does
        // not do — 0x43ae88, on the branch 0x43aaf4 takes for a run starting.
        self.image.has_been_in_the_run_for(0);
        self.image.the_stage_script_has_run_for(0);
        self.image.has_a_runs_numbers();
        self.image.has_lives_and_bombs(LIVES, BOMBS);
        self.image.shot_is_at_power(POWER);
        self.image.has_died(0.0);
        self.image.the_run_has_scored(SCORE, POINT_ITEMS);
        // And the word the player acts on put to nothing, which is what registering the run's replay does
        // (0x451fbb) — a run starting and not one carried into its next stage, whose replay is the one it had.
        self.image.the_runs_replay_is_registered();
    }

    /// And what the build of one stage leaves: the run's clock where the stage before left it, the seed the
    /// stage was built out of, the player where a stage starts one, and nothing on the field. **Not the
    /// front end gone**, which is its own update's to do — see [`Fake::the_front_end_leaves_the_chain`].
    ///
    /// **The clock is not put back, which is the real game's own behaviour**: 0x43aaf4 branches on the
    /// wanted scene and only a run *starting* reaches the `and [gm+0x3de04],0x0` at 0x43ae88 — a stage
    /// carrying a run on jumps past it. A launch measured the same thing from outside, reading
    /// `stage=0 frames=18218` and `stage=1 frames=18218` either side of a stage 1 cleared.
    fn the_stage_is_laid_out(&self) {
        self.image.the_run_wants_no_scene();
        // **And the stage's own track playing**, which is what a build of the real game leaves: `PlayBGM`
        // opens the stream over `thbgm.dat` and hangs it off the sound manager's `+0x5208`, so a stage
        // with none would be a laid-out game whose `Th08::audio_state` answers one range where the real
        // one answers four — and a chapter taken here would hold back less than a chapter taken there.
        // **And the track the stage's own**, the first its info block names: which song is playing is
        // what `Th08::the_fights_own_song_is_playing` tells a boss from a midboss by.
        self.image.a_track_is_playing();
        self.image.holds_its_music_file(&TRACKS);
        self.image.names_its_tracks(A_STAGES_TRACKS);
        self.image.the_stream_reads_the_track(0);
        self.image
            .the_stream_is_at(0, A_MUSIC_CHUNK, A_TRACK_BEFORE_ITS_LOOP);
        self.image.the_music_is_streamed_on_thread(A_MUSIC_THREAD);
        self.image.the_player_is_playing();
        self.image.no_dialogue_is_up();
        self.image.no_boss_is_fighting();
        self.the_boss_is_up.set(false);
        // **And no card up**, which the build leaves too: 0x43b4c8 registers the spell card through 0x417f60,
        // whose 0x414590 zeroes the whole object — `0x991` dwords from `0x4ea670`, at 0x4145b1 and at 0x4147ae
        // on its other branch. A card the run before left up was a boss arriving as her card on the next build.
        self.image.no_card_is_up();
        // And the pause down, which the build clears (0x43b1cf).
        self.image.the_run_is_paused(false);
        self.the_pause_closing.set(None);
        self.image.enemies_alive(0);
        self.image.bullets_alive(0);
        self.image.lasers_alive(0);
        self.image.items_alive(0);
        self.bullets.borrow_mut().clear();
        // The enemy manager a stage's build makes, with its own timer from nothing.
        self.enemy_frames.set(0);
        // And the conversation over, the line each stage's script starts being its own — see
        // [`Fake::has_a_conversation`], which is what says where in a stage one is.
        self.conversation.set(0);
    }

    /// **And that run played by this game's own rules from here on**, which is what a run its front end
    /// started is: the stage's clock, its script, its bullets and a player the word moves.
    pub fn plays_its_own_stage(&self) {
        self.on_its_own.set(true);
    }

    /// How long one of those stages lasts before the next is built, where an e2e test says.
    pub fn stages_last(&self, frames: u32) {
        self.stage_frames.set(Some(frames));
    }

    /// **Whether that stage fires at the player**, which it does unless an e2e test says not: a pass
    /// stepping around a replay is no run to be hit in, and a hit there is a run it loses.
    pub fn fires_at_the_player(&self, fires: bool) {
        self.fires.set(fires);
    }

    /// **A conversation on that frame of every stage, lasting that many updates**, which is what instruction
    /// 148 of a real stage's script starts: the run's clock is put on [`THE_CLOCK_A_LINE_COSTS`] and then
    /// stands still until the line goes, while the stage's script, its bullets, its generator and the player
    /// carry on. Three of them in a real stage, 103 updates on the one measured — see the update's own
    /// comment.
    pub fn has_a_conversation(&self, at: u32, lasts: u32) {
        self.a_conversation_at.set(Some((at, lasts)));
    }

    /// **And the run's clock put 1,800 on at that frame of every stage with no line up**, which is what the real
    /// game's clock does as a boss's card ends: session 11's no-bomb clear read the record's last frame
    /// `16603` and the next `18404` as stage 1's boss card was taken, and `27112` then `28913` in stage 6 —
    /// 1,800 and the frame's own step, and a chapter beginning on the frame it landed on each time. What puts
    /// it there is not read; the 1,800 is the conversation's own number.
    pub fn skips_its_clock_at(&self, at: u32) {
        self.a_skip_at.set(Some(at));
    }

    /// **The supervisor's thread asleep `frames` past the count its build waits for**, which is the real build's
    /// own last act: 0x43b86e sleeps seventeen milliseconds at a time until `[gm+0x3c]` is up, so the frames the
    /// main thread fits into the last sleep are frames of the walk running a stage whose build is not over.
    /// Three pick-ups of one file on the real game began their stage at `frames=5` with `script=5`, `7` and `8`.
    pub fn goes_on_building_for(&self, frames: u32) {
        self.goes_on_building_for.set(frames);
    }

    /// One frame of the game's own update, which is what the calc chain's walk is here.
    ///
    /// **The read first, then the front end, then the supervisor, then the run** — which is the order this
    /// game's own jobs are in: the supervisor's read is the chain's first job (0x4455f5), the scene switch
    /// is the same job's tail, and everything a run is made of is registered above it.
    fn update(&self) {
        self.comes_up_at_its_title();
        // **A frame the supervisor sits out**: while its loader's word is set, its own update — the
        // chain's first job — serves the thread and answers 1 before the sound and the input read — see
        // [`Fake::the_supervisor_sits_out_a_build`]. **And 1 is not a stop**: the walk takes it to the next job
        // (0x43cb0e), so the run manager steps the frames the build waits out (0x439fdf) and whatever of the
        // stage the thread has put on the chain runs too — **from the count being up**, which is what three
        // pick-ups of the real game say, each beginning its stage with `script=5` after thirty frames of build
        // and two of them `7` and `8` where the thread slept on past the count.
        if self.the_supervisor_sits_out_a_build().is_some() {
            let stood = self.image.how_long_the_build_has_stood();
            let up = stood >= self.image.what_the_build_stands_for();
            self.image.the_build_has_stood_for(stood + 1);
            if up && self.build.get() != Build::Idle {
                self.the_stages_jobs_run(self.image.the_word_the_run_acts_on());
            }
            return;
        }
        // **Last frame's word copied out before the read**, which is where this game copies it: 0x4455e4,
        // in front of the read at 0x4455f5. Which is what makes every screen's `(cur & mask) != (last &
        // mask)` an edge — and what `Th08::swallow_input` has to write both halves of.
        let last = self.image.the_word_the_keys_read();
        // The read the game's own update makes for a frame's buttons, which orb stands in front of: the
        // whole of 0x43d970, whose answer is one word. Through the hook only where a launch installs it,
        // for the reason `orbs_own_loop` is.
        let word = match self.reads_the_keys {
            true => orb_core::runtime::get_input(),
            false => input(),
        };
        self.word.set(word);
        self.image.the_keys_read(word, last);
        // **The switch before the front end, and so a frame after the ask**: a scene the front end
        // writes into `+0x15c` on this update is one the supervisor's own job — already run this update
        // — switches to on the next, which is the frame orb reads `1 -> 2` on and offers a run left
        // behind. Switched on the same update, the laid-out game went from the front end to a built
        // stage between two of orb's reads and no run was ever asked for.
        self.follows_its_supervisor(word, last);
        self.walks_its_front_end(word, last);
        if self.image.the_scene_now() == Scene::Ending as i32 {
            self.plays_the_ending();
        }
        if self.image.the_scene_now() == Scene::RunOver as i32 {
            self.walks_the_screen_a_run_ends_at(word, last);
        }
        // **The player moved by the word the run acts on**, `[0x164d52c]`, which the player's job reads at
        // priority 9 (0x44aee8) and the replay's element at 0x11 writes the read into at the walk's tail
        // (0x452340) — so left to the game the player moves on the word of the frame before, which is what a
        // whole run of the real game measured: the move from one frame to the next followed the word orb chose
        // on the earlier of the two 5,104 times and contradicted it 16. And where the walk stops short of that
        // element the player goes on acting on what it was last handed, which a run picked up in stage 2 of the
        // real game did for every one of 7,318 frames.
        // **The pause menu at priority 1 and the run manager's own pause at 2**, both ahead of the stage's
        // jobs, and the walk stopped there while the pause stands — see [`Fake::its_pause_menu`].
        if self.its_pause_menu(word, last) {
            return;
        }
        // **The run manager's own update past its pause**, which is where the score is held at the panel's
        // copy — see `Image::the_run_manager_holds_the_score_at_the_panels_copy`.
        if self.image.a_run_is_on() {
            self.image
                .the_run_manager_holds_the_score_at_the_panels_copy();
        }
        let reached_the_copy = self.plays_the_stage_it_is_in(self.image.the_word_the_run_acts_on());
        if reached_the_copy {
            self.image.the_runs_replay_copies_the_read();
        }
    }

    /// **The game's own pause, over a run**: the menu's job at priority 1 (0x402200, 0x4037b0) and the run
    /// manager's at 2 (0x43a0e8) — see `Th08`'s `G_PAUSED`. With the pause up, a menu press puts the menu in
    /// its closing phase and [`THE_PAUSE_CLOSES_OVER`] updates later the byte goes back to nothing; with it
    /// down, a menu press in a run somebody plays puts it up. **And answers whether the walk stops here**,
    /// which it does on every update the byte is set on — the run manager answering 3 — the stage's jobs and
    /// the replay's copy of the word going unrun. The menu's own items are not walked, orb reading none of
    /// them.
    fn its_pause_menu(&self, word: u16, last: u16) -> bool {
        if !self.image.a_run_is_on() || self.image.the_loader_is_running() {
            return false;
        }
        let edge = word & BUTTON_MENU != 0 && word & BUTTON_MENU != last & BUTTON_MENU;
        if self.image.is_paused() {
            match self.the_pause_closing.get() {
                // **And the run on again on the update the menu lets go on**, the run manager reading the byte
                // after the menu's job has cleared it.
                Some(closing) if closing + 1 >= THE_PAUSE_CLOSES_OVER => {
                    self.the_pause_closing.set(None);
                    self.image.the_run_is_paused(false);
                    return false;
                }
                Some(closing) => self.the_pause_closing.set(Some(closing + 1)),
                None if edge => self.the_pause_closing.set(Some(0)),
                None => {}
            }
            return true;
        }
        if edge && !self.image.the_run_is_the_demos() {
            self.image.the_run_is_paused(true);
            return true;
        }
        false
    }

    /// **The screens of the front end a recipe of this game walks**, by the numbers
    /// `Th08::front_end_screen` reads out of `+0x14428`: the title menu at 0, the difficulty select at 4,
    /// the team select at 5 and the Extra's at 0xd, and the replay list at 7.
    ///
    /// **Run for as long as the front end's element is on the chain, whatever the scene is** — see
    /// [`Fake::the_front_end_leaves_the_chain`], which is the one way off it. **The team select reads one
    /// decide, in the phase after the eight updates it comes up over** — see `Th08`'s
    /// `TEAM_PHASE_CHOOSING`, which is why a recipe presses it twice and why a launch that pressed once
    /// stayed on it. **And the title counts the updates nothing is pressed on**, to the attract demo — see
    /// [`THE_TITLE_WAITS`].
    fn walks_its_front_end(&self, word: u16, last: u16) {
        if !self.front_end_on_the_chain.get() {
            return;
        }
        let (Some(screen), Some(mut cursor)) = (
            self.image.the_screen_the_front_end_is_on(),
            self.image.the_item_the_cursor_is_on(),
        ) else {
            return;
        };
        let edge = |mask: u16| word & mask != last & mask && word & mask != 0;
        let Some(up) = the_screen(screen) else { return };
        // **And the stage select a practice goes through comes up the same way**, the 1 written on the update
        // its count reads 8 (0x46af99).
        if matches!(
            up,
            Screen::Team
                | Screen::TeamOfTheExtra
                | Screen::TeamOfAPractice
                | Screen::StageOfAPractice
        ) {
            let (phase, settled) = self.image.where_the_front_end_is();
            if phase != TEAM_PHASE_CHOOSING {
                let phase = if settled == TEAM_SETTLES_AT {
                    TEAM_PHASE_CHOOSING
                } else {
                    phase
                };
                self.image.the_front_end_is_at(phase, settled + 1);
                return;
            }
        }
        // **The title built over its first updates, and its decide gated on the count after** — see
        // [`TITLE_READS_A_DECIDE_AT`].
        let reads_a_decide = if up == Screen::Title {
            let (phase, count) = self.image.where_the_front_end_is();
            if phase == TITLE_PHASE_BUILDING && count < TITLE_ITEMS {
                self.image
                    .the_front_end_is_at(TITLE_PHASE_BUILDING, count + 1);
                return;
            }
            let count = if phase == TITLE_PHASE_BUILDING {
                0
            } else {
                count
            };
            self.image.the_front_end_is_at(TITLE_PHASE_ITEMS, count + 1);
            count >= TITLE_READS_A_DECIDE_AT
        } else {
            true
        };
        if up == Screen::Title {
            let idle = if word == 0 {
                self.title_idle.get() + 1
            } else {
                0
            };
            self.title_idle.set(idle);
            if idle > THE_TITLE_WAITS {
                self.starts_its_demo();
                return;
            }
        }
        let items = match up {
            Screen::Title => TITLE_ITEMS,
            Screen::Difficulty | Screen::DifficultyOfAPractice => DIFFICULTIES_OFFERED,
            Screen::DifficultyOfTheExtra => 1,
            Screen::Team | Screen::TeamOfTheExtra | Screen::TeamOfAPractice => TEAMS,
            Screen::StageOfAPractice => STAGES_A_PRACTICE_LISTS,
            Screen::Replays => REPLAYS_LISTED,
        };
        // The cursor first, a direction being what moves it: the list wraps, which is what makes a press
        // at a list of three land on another item every time — **and on the title past an item its records
        // have not lit**, which that screen steps over rather than draws (0x467977 through 0x4679bf), **and
        // on a practice's stage select past a stage they have no pass of** (0x46b021 through 0x46b061).
        if edge(BUTTON_UP) || edge(BUTTON_DOWN) {
            let by = if edge(BUTTON_UP) { items - 1 } else { 1 };
            let records = self.records.get();
            let lit = |item: i32| match up {
                Screen::Title => records.lights(item),
                Screen::StageOfAPractice => records.offers_a_practice_of(item),
                _ => true,
            };
            let mut moved = (cursor + by).rem_euclid(items);
            while !lit(moved) {
                moved = (moved + by).rem_euclid(items);
            }
            self.image.the_front_end_cursor_is(moved);
            // **And on the title the same update's decide reads the item moved to**, 0x47030b being called
            // ahead of it; what the other screens do after a move is not read, so nothing more is done there.
            if up != Screen::Title {
                return;
            }
            self.left_the_title_on.set(moved);
            cursor = moved;
        }
        if !reads_a_decide {
            return;
        }
        // **A cancel at the title puts the cursor on the item that ends the game** (0x4681f3), and does it
        // again on that item: nothing asks where the cursor already is.
        if up == Screen::Title && edge(MENU_CANCEL) && !edge(MENU_DECIDE) {
            self.image.the_front_end_cursor_is(TITLE_ITEM_QUIT);
            self.left_the_title_on.set(TITLE_ITEM_QUIT);
            return;
        }
        if !edge(MENU_DECIDE) {
            return;
        }
        match up {
            // Item 0, which is the item a run is behind, and item 4, which the replays are: what the six
            // others do is not written here, so nothing is what a decide on one of them does.
            Screen::Title if cursor == TITLE_ITEM_START => {
                self.left_the_title_on.set(cursor);
                self.image
                    .the_front_end_shows(Screen::Difficulty, DIFFICULTY_NORMAL);
            }
            Screen::Title if cursor == TITLE_ITEM_REPLAYS => {
                self.left_the_title_on.set(cursor);
                self.image.the_front_end_shows(Screen::Replays, 0);
            }
            // **And a stage practice's**, which 0x467dd9 starts on screen 8 with the kind's bit 0 set.
            Screen::Title if cursor == TITLE_ITEM_PRACTICE => {
                self.left_the_title_on.set(cursor);
                self.image.the_run_is_a_practice();
                self.image
                    .the_front_end_shows(Screen::DifficultyOfAPractice, DIFFICULTY_NORMAL);
            }
            Screen::DifficultyOfAPractice => {
                self.image.the_difficulty_chosen_is(cursor as u8);
                self.image
                    .the_front_end_comes_up_on(Screen::TeamOfAPractice, 0);
            }
            // Whose decide writes the team and puts the stage select up rather than asking for a run
            // (0x46ab98 through 0x46abb9).
            Screen::TeamOfAPractice => {
                self.image.the_character_playing_is(cursor as u8);
                self.image.the_run_is_a_practice();
                self.image
                    .the_front_end_comes_up_on(Screen::StageOfAPractice, FIRST_STAGE);
            }
            // And the stage select's own, which asks for the run: the difficulty the select settled, the
            // stage under the cursor, the kind's recording bit cleared and the run's scene (0x46b09e through
            // 0x46b0e6).
            Screen::StageOfAPractice => {
                self.image
                    .is_playing(cursor, self.image.the_difficulty_chosen());
                self.image.the_run_plays_no_recording();
                self.image.asks_for_the_scene(Scene::Run);
                self.the_front_end_leaves_the_chain();
            }
            // **And the Extra's own flow**, which 0x467e8f starts on screen 0xc.
            Screen::Title if cursor == TITLE_ITEM_EXTRA => {
                self.left_the_title_on.set(cursor);
                self.image
                    .the_front_end_shows(Screen::DifficultyOfTheExtra, 0);
            }
            Screen::Title => {}
            // Whose one difficulty is the Extra, settled as `cursor + 4` (0x46a48d).
            Screen::DifficultyOfTheExtra => {
                self.image
                    .the_difficulty_chosen_is(THE_EXTRAS_DIFFICULTY as u8);
                self.image
                    .the_front_end_comes_up_on(Screen::TeamOfTheExtra, 0);
            }
            // **The list's own decide, which starts the replay under the cursor** — 0x46ed24 through
            // 0x46ee2e: bit 3 of the run kind written as 1 and nothing else of it, the difficulty, the team
            // and the stage out of the replay's header, and the run's scene asked for. The one replay here
            // is a Normal run of the first team from stage 1.
            Screen::Replays => {
                self.image.the_run_reads_its_own_input();
                self.image.is_playing(FIRST_STAGE, DIFFICULTY_NORMAL);
                self.image.asks_for_the_scene(Scene::Run);
                self.the_front_end_leaves_the_chain();
            }
            // **And the difficulty the select was left on is the difficulty the run is played on**, which
            // is what `checks::a_normal_mode_run_of_th08_played_without_a_bomb` reads out of the trace:
            // the cursor's own item, which a recipe that presses nothing leaves on Normal — written where
            // the select writes it, and copied into the run's own by the team select's decide.
            Screen::Difficulty => {
                self.image.the_difficulty_chosen_is(cursor as u8);
                self.image.the_front_end_comes_up_on(Screen::Team, 0);
            }
            // **And the kind cleared as it asks**, which is every start screen's own — 0x46ac1f, 0x46b0be and
            // 0x46ca8d write bit 3 as nothing before the run's scene goes in — and not the build's: a
            // build that cleared it would be a replay turned into a run the frame it is built. The scene
            // wanted goes to the run's while the scene now is still the front end's, which is what
            // `Th08::run_chosen` reads, and the stage is the first or the Extra's (0x46abf6 through
            // 0x46ac10).
            Screen::Team | Screen::TeamOfTheExtra => {
                let difficulty = self.image.the_difficulty_chosen();
                let stage = if up == Screen::TeamOfTheExtra {
                    difficulty + 4
                } else {
                    FIRST_STAGE
                };
                self.image.is_playing(stage, difficulty);
                self.image.the_run_is_its_own();
                // **And the records saved as the run starts**, which is what this game does at every run's
                // start — and through the mode's own file where orb has forked the opens.
                self.saves_its_records();
                self.image.asks_for_the_scene(Scene::Run);
                self.the_front_end_leaves_the_chain();
            }
        }
    }

    /// **The attract demo, started by the title over whatever it is under** — 0x467ad4 through 0x467c54: the
    /// next of the demos' replays read, the bits of a recording played and of the demo set, the difficulty,
    /// the team and the stage out of that replay's header, the run's scene asked for, and the title's own
    /// update answering nought. What this file owes of it is what orb reads: the run's kind and the
    /// difficulty, Easy — the `diff=0` a watched attempt of the real game read the frame the demo took it.
    fn starts_its_demo(&self) {
        self.image.the_run_is_the_demo();
        self.image.is_playing(FIRST_STAGE, DIFFICULTY_OF_THE_DEMO);
        self.image.asks_for_the_scene(Scene::Run);
        self.the_front_end_leaves_the_chain();
    }

    /// **The supervisor's own switch**, which is what carries the game from one scene to the next: the
    /// scene wanted becoming the scene now, and the build that scene's arrival is.
    ///
    /// And the other way a scene changes, which is a run asking: `[gm+0x3db9c]` is `-1` while a run goes
    /// on and 0x439fb6 is the one place that copies it across — the hand-over `Th08::leave_run` writes,
    /// which waits for a decide of its own.
    fn follows_its_supervisor(&self, word: u16, last: u16) {
        let (scene, wanted) = (self.image.the_scene_now(), self.image.the_scene_wanted());
        if scene != wanted {
            // **The run's own scene and the run built again are two different builds**, which is this
            // game's own distinction: the first is a run starting, so the numbers its front end put in
            // place go in with the stage, and the second is a run carrying into its next stage, which
            // keeps the numbers the last one left. A build that put them in both times would be a run
            // that starts over at every stage.
            //
            // **And the switch starts a thread rather than building anything**, which is the real
            // supervisor's own shape: the case for the scene being left registers the run manager
            // (0x44578d out of the front end, 0x4458f4 out of a run), its added callback sets the
            // loader's word and hands the build to `CreateThread`, and `scene = wanted` is the update's
            // own tail at 0x445b0b — see `Th08::a_stage_has_just_been_built`. So the scene after this
            // update is the run's and nothing of the run is in place, which is the frame the real game's
            // reproduction line read bytes nothing had written on.
            if wanted == Scene::Run as i32 {
                self.the_load_starts_a_run.set(true);
                self.image.is_loading(true);
                self.image.is_in_scene(Scene::Run, Scene::Run);
            } else if wanted == Scene::RunAgain as i32 {
                self.the_load_starts_a_run.set(false);
                self.image.is_loading(true);
                self.image.is_in_scene(Scene::Run, Scene::Run);
            } else if wanted == Scene::FrontEnd as i32 {
                // **Out of the screen a run ends at, its teardown first**, which is 0x4531f0 on the way out of
                // scene 6 (0x445a2e) and what writes the records — so a launch that stops short of the title
                // leaves a clear unwritten, the Extra it lit unlit — and its update off the chain, wherever the
                // way out was asked from.
                if scene == Scene::RunOver as i32 {
                    self.image.the_screen_a_run_ends_at_is_off_the_chain();
                    self.saves_its_records();
                }
                self.the_front_end_is_up_on(self.left_the_title_on.get());
            }
            // **And the keys' words put to nothing at the switch's tail**, every case of it coming through
            // 0x445adf — see `Image::the_switch_puts_the_keys_to_nothing`.
            self.image.the_switch_puts_the_keys_to_nothing();
            return;
        }
        if scene != Scene::Run as i32 {
            return;
        }
        // **The hand-over the run asked for**, where it asked and where what it waits for has come — which is
        // 0x439f3c's own three ways: a press, the two bits of `menu_decide` down this frame and not the same
        // as last frame's (0x439f3c through 0x439f67); or nothing pressed at all where the run plays a
        // recording, bit 3 of its kind (0x439f69), or its stage word is 6, 7 or 8 (0x439f78 through
        // 0x439f91). **A press and not a key held**, which is what this file took until a give-up the rules'
        // own held shot answered here and the real game would have stood behind.
        let asked = self.image.the_scene_the_run_wants();
        if asked < 0 || self.image.the_run_is_ending() == 0 {
            return;
        }
        let pressed = word & MENU_DECIDE != 0 && word & MENU_DECIDE != last & MENU_DECIDE;
        let a_recording = self.image.the_run_plays_a_recording();
        let the_last_stages = (6..=8).contains(&self.state().stage);
        if !(pressed || a_recording || the_last_stages) {
            return;
        }
        self.image.the_run_wants_no_scene();
        self.image.the_run_is_its_own();
        if asked == Scene::FrontEnd as i32 {
            self.the_front_end_is_up_on(self.left_the_title_on.get());
        } else {
            self.image.is_in_scene(Scene::FrontEnd, Scene::FrontEnd);
        }
    }

    /// **The build a run's own scene arrives by**, which is where the two seams of a resume go in: the
    /// seed settled inside it and the numbers put in place at its end.
    ///
    /// The order is the game's own — [`stage_building`] is the job the build calls three instructions past
    /// its seed's copy (0x409b20) and [`stage_begun`] is the run manager's own added callback (0x43aaf4),
    /// which is the chain job the build registers — so the seed goes in first and the numbers after.
    fn builds_the_stage(&self) {
        self.the_stage_is_laid_out();
        self.stage_built_at
            .set(self.image.the_frames_the_run_has_been_going());
        // The seed this build settled on, through the seam orb reads it at: the game's own word goes in
        // first and orb's answer, where it has one, goes in over it. The step is this file's own — what
        // it has to be is a seed that differs stage by stage.
        let seed = self.seed.get().wrapping_mul(0x5bd1).wrapping_add(0x2f19);
        self.seed.set(seed);
        self.image.the_stage_began_with_the_seed(seed);
        // **Through orb's own patch over the seam, and not straight to this file's callback**, which is
        // where the build's call lands in the real game once the prologue is written over. Called
        // straight, a run of the laid-out game was built with the seam never run.
        orb_core::runtime::stage_building(self.image.the_games_stage_word());
    }

    /// **The supervisor's update on a frame its loader's word is set**, which is where it stops short of the
    /// sound and the input read: the thread's requests served — [`Fake::its_loader_goes_on`] — and 1 answered
    /// at 0x4455bc. Nothing where the word is clear, which is a frame the rest of [`Fake::update`] is.
    fn the_supervisor_sits_out_a_build(&self) -> Option<i32> {
        if !self.image.the_loader_is_running() {
            return None;
        }
        self.its_loader_goes_on();
        Some(CHAIN_CARRIED_ON)
    }

    /// **The supervisor's thread, a step of it for every update of the supervisor's own** — which is what the
    /// real thread waits on: the player's added callback, run on the thread as its job goes up, asks the object
    /// at `[0x18bdc90]` for a file and sleeps until 0x466000 has served it, and 0x466000 is called at 0x445570
    /// of the supervisor's update. Its first step builds the stage and puts its jobs on the chain; then it waits
    /// out the frames its build stands for, and clears its word [`Fake::goes_on_building_for`] steps after they
    /// are up. The word is what `Th08::a_stage_has_just_been_built` reads the moment off; what has to hold is
    /// that the numbers are in before any job of the stage runs.
    fn its_loader_goes_on(&self) {
        if !self.image.the_loader_is_running() {
            return;
        }
        let asleep_for = match self.build.get() {
            Build::Idle => {
                let state = self.state();
                if self.the_load_starts_a_run.get() {
                    self.the_numbers_a_run_starts_with_are_in_place();
                } else if !state.practice && state.difficulty < THE_EXTRAS_DIFFICULTY {
                    self.image.a_carried_builds_hands_the_score_to_the_panel();
                }
                self.image.the_build_has_stood_for(0);
                self.builds_the_stage();
                // **And the player's own register, run on the thread with the rest of the build**: 0x44c230
                // on both of the build's paths, its added callback at once (0x43c8a8) — state 1 with 120 under
                // it, or 10 in a spell practice (0x44d886 through 0x44d8a7). A run placed rather than built is
                // placed playing, which is what an e2e test placing one asks for.
                let count = if self.image.the_run_practises_a_card() {
                    10
                } else {
                    A_BUILD_LEAVES
                };
                self.image.the_player_comes_in(count);
                self.build.set(Build::WaitingOutItsFrames);
                return;
            }
            // **The build's last wait**, 0x43b86e: its frames counted by the run manager's own step, see
            // [`Fake::update`], and the thread asleep for as long after they are up as a test asks.
            Build::WaitingOutItsFrames => {
                if self.image.how_long_the_build_has_stood()
                    < self.image.what_the_build_stands_for()
                {
                    return;
                }
                self.goes_on_building_for.get()
            }
            Build::Asleep(left) => left,
        };
        match asleep_for {
            0 => {
                self.build.set(Build::Idle);
                self.image.is_loading(false);
            }
            left => self.build.set(Build::Asleep(left - 1)),
        }
    }

    /// **Another stage than the one asked for built**, which is the one thing a resume must not write a run
    /// over: the stage word the build reads, written with `stage` — orb's own 0-based number for it — after the
    /// run has been asked for and before the build reads it.
    pub fn builds_the_stage_instead(&self, stage: i32) {
        self.image.is_playing(stage, self.state().difficulty);
    }

    /// **And the stage asked for built inside the attract demo**, which is a run orb keeps nothing of: the two
    /// bits of the run's kind the demo's own start writes — see `Image::the_run_is_the_demo`.
    pub fn builds_it_inside_the_demo(&self) {
        self.image.the_run_is_the_demo();
    }

    /// **A stage asked for that never arrives**: the front end off the chain and the supervisor already in the
    /// run's own scene with nothing asked of it, so no build is started and none is entered. What orb must not
    /// do is hold a run's numbers for ever waiting for a moment that is not coming.
    pub fn the_stage_never_comes_up(&self) {
        self.the_front_end_leaves_the_chain();
        self.image.is_in_scene(Scene::Run, Scene::Run);
    }

    /// And the stage after it, which is how this game moves a run on: the run built again, which is the
    /// scene its manager asks for at 0x439ddc.
    pub fn moves_on_to_the_stage(&self, stage: i32) {
        self.image.is_playing(stage, self.state().difficulty);
        self.image.is_in_scene(Scene::Run, Scene::RunAgain);
    }

    /// **And the run over where there is no stage after the one just played**, which is the run manager's
    /// own answer to that (0x439d4d): the ending that last stage's clear goes into — scene 9, for
    /// [`THE_ENDING_LASTS`] updates — and then the front end up again, on the item the title was left on,
    /// with nothing of a run's own scene wanted and this run's stages no longer played.
    ///
    /// **What orb reads off it is the scene**, `Th08::read_state`'s `in_run` going false and `in_ending`
    /// holding for the ending's own updates — the first is what a recipe's `play` step ends on, and the
    /// second what a case about a clear is held to. Nothing here registers a score.
    fn the_run_is_over(&self, stage: i32) {
        // **Kaguya's ending writes the bit the title lights the Extra out of** — `0x4000` where the stage
        // word is 7, at the ending's own start (0x429a00 through 0x429c07) — and a recording played writes
        // nothing.
        if stage == KAGUYAS_STAGE && !self.image.the_run_plays_a_recording() {
            let mut records = self.records.get();
            records.extra_lit = true;
            self.records.set(records);
        }
        self.image.the_run_wants_no_scene();
        self.image.the_run_is_its_own();
        self.image.no_card_is_up();
        self.on_its_own.set(false);
        self.image.is_in_scene(Scene::Ending, Scene::Ending);
        self.ending_left.set(THE_ENDING_LASTS);
    }

    /// **One update of the ending**, and the front end once it has run out.
    fn plays_the_ending(&self) {
        let left = self.ending_left.get().saturating_sub(1);
        self.ending_left.set(left);
        // **The screen a run ends at after it**, which 0x4582a0 registers out of scene 9 as out of a run —
        // in its name entry, bit 0 of the run kind being clear for a run of the whole game (0x458316).
        if left == 0 {
            self.image.is_in_scene(Scene::RunOver, Scene::RunOver);
            self.image
                .the_screen_a_run_ends_at_is_up_in(RESULT_TYPING_A_NAME);
        }
    }

    /// **One update of the screen a run ends at**, which is 0x4584b0's: the handler of the phase it is in, and
    /// then the timer stepped at the tail every one of them falls through to (0x458646) — see
    /// [`RESULT_TYPING_A_NAME`] and the phases after it. **Where 0x4584b0 answers nought the screen goes**:
    /// the way out writes the front end into the scene wanted (0x458591) and leaves the chain.
    fn walks_the_screen_a_run_ends_at(&self, word: u16, last: u16) {
        if !self.image.the_screen_a_run_ends_at_is_on_the_chain() {
            return;
        }
        let edge = |mask: u16| word & mask != 0 && word & mask != last & mask;
        let (phase, timer) = self.image.where_the_screen_a_run_ends_at_is();
        let to = |phase: i32, timer: i32| self.image.the_screen_a_run_ends_at_is_at(phase, timer);
        match phase {
            RESULT_TYPING_A_NAME => self.walks_the_name_entry(timer, &edge),
            RESULT_SHOWING_THE_SCORE if timer >= THE_SCORE_WAITS && edge(MENU_DECIDE) => {
                to(RESULT_BEFORE_THE_OFFER, 0);
            }
            RESULT_BEFORE_THE_OFFER if timer >= THE_PAUSE_LASTS => {
                to(RESULT_OFFERING_A_REPLAY, THE_PAUSE_HANDS_THE_OFFER);
            }
            // **The question**, its cursor in the entry's count word: put at the first answer on the offer's
            // tenth update, moved by either direction over its two answers (0x456b1b), and a decide on the
            // first going on to save one where one on the second — or a cancel, or the menu key — goes out.
            RESULT_OFFERING_A_REPLAY => {
                let (name, answer, cursor) = self.image.what_the_name_entry_holds();
                if timer == THE_PAUSE_HANDS_THE_OFFER + 1 {
                    self.image.the_name_entry_holds(name, 0, cursor);
                } else if timer >= THE_OFFER_WAITS {
                    if edge(BUTTON_UP) || edge(BUTTON_DOWN) {
                        self.image.the_name_entry_holds(name, 1 - answer, cursor);
                    } else if edge(MENU_DECIDE) && answer == 0 {
                        to(*RESULT_SAVING_A_REPLAY.start(), 0);
                    } else if edge(MENU_DECIDE) || edge(BUTTON_BOMB) || edge(BUTTON_MENU) {
                        to(RESULT_LEAVING, 0);
                    }
                }
            }
            // **And the steps a replay the game keeps is saved in**, walked on a decide apiece and out at the
            // last: this file owes nothing of what they save, orb writing the whole group away in the mode
            // whose replays are its own — see `Th08::skip_replay_prompt`.
            saving if RESULT_SAVING_A_REPLAY.contains(&saving) && edge(MENU_DECIDE) => {
                match saving == *RESULT_SAVING_A_REPLAY.end() {
                    true => to(RESULT_LEAVING, 0),
                    false => to(saving + 1, 0),
                }
            }
            RESULT_LEAVING if timer >= THE_WAY_OUT_LASTS => {
                self.image.asks_for_the_scene(Scene::FrontEnd);
                self.image.the_screen_a_run_ends_at_is_off_the_chain();
                return;
            }
            _ => {}
        }
        let (phase, timer) = self.image.where_the_screen_a_run_ends_at_is();
        self.image.the_screen_a_run_ends_at_is_at(phase, timer + 1);
    }

    /// **The name entry's own update**, 0x45621e: the field filled with the last name typed on its first
    /// update, and from its tenth the cursor walked over the grid by the directions — a row a step up and
    /// down and one item left and right, round within its row — and a decide writing the character under it,
    /// a space on the space item and leaving the entry on the one past that, and a bomb taking the last one
    /// back. The count at eight puts the cursor on that last item (0x4567c9), and the menu key leaves as
    /// well (0x45688c).
    fn walks_the_name_entry(&self, timer: i32, edge: &dyn Fn(u16) -> bool) {
        if timer == 0 {
            self.image.the_name_entry_holds(self.last_name.get(), 0, 0);
            return;
        }
        if timer < THE_ENTRY_WAITS {
            return;
        }
        let (mut name, typed, cursor) = self.image.what_the_name_entry_holds();
        let moved = if edge(BUTTON_UP) {
            Some((cursor - A_ROW_OF_IT).rem_euclid(THE_GRID))
        } else if edge(BUTTON_DOWN) {
            Some((cursor + A_ROW_OF_IT).rem_euclid(THE_GRID))
        } else if edge(BUTTON_LEFT) {
            Some(cursor - cursor % A_ROW_OF_IT + (cursor - 1).rem_euclid(A_ROW_OF_IT))
        } else if edge(BUTTON_RIGHT) {
            Some(cursor - cursor % A_ROW_OF_IT + (cursor + 1).rem_euclid(A_ROW_OF_IT))
        } else {
            None
        };
        if let Some(moved) = moved {
            self.image.the_name_entry_holds(name, typed, moved);
            return;
        }
        let at = typed.min(A_NAMES_CHARACTERS - 1) as usize;
        if edge(MENU_DECIDE) {
            if cursor > THE_SPACE_ITEM {
                return self.the_name_entry_is_left(name);
            }
            name[at] = match cursor == THE_SPACE_ITEM {
                true => b' ',
                false => b'!' + cursor as u8,
            };
            let typed = (typed + 1).min(A_NAMES_CHARACTERS);
            let cursor = match typed == A_NAMES_CHARACTERS {
                true => THE_END_ITEM,
                false => cursor,
            };
            self.image.the_name_entry_holds(name, typed, cursor);
        } else if edge(BUTTON_BOMB) && typed > 0 {
            name[(typed - 1) as usize] = b' ';
            self.image.the_name_entry_holds(name, typed - 1, cursor);
        } else if edge(BUTTON_MENU) {
            self.the_name_entry_is_left(name);
        }
    }

    /// **The entry left**, which is the score after it and the name kept for the next one (0x4568bc
    /// through 0x45692b).
    fn the_name_entry_is_left(&self, name: [u8; 8]) {
        self.last_name.set(name);
        self.image
            .the_screen_a_run_ends_at_is_at(RESULT_SHOWING_THE_SCORE, 0);
    }

    /// **What the run manager writes into the records as a stage ends** — that stage's bit into the
    /// difficulty's `+0x16` word (0x439c50 through 0x439d02), and `0x8000` with it where the stage is a last
    /// one (0x439eaa through 0x439ed5); nothing for a recording played, which that code steps past — see
    /// [`Records`]. Answers the records as they now stand.
    fn writes_down_the_stage(&self, stage: i32) -> Records {
        let mut records = self.records.get();
        if !self.image.the_run_plays_a_recording() {
            let normal_to_lunatic = (1..=3).contains(&self.state().difficulty);
            records.eirins_stage_passed |= stage == EIRINS_STAGE && normal_to_lunatic;
            records.cleared |= matches!(stage, EIRINS_STAGE | KAGUYAS_STAGE);
            if let Ok(stage) = u32::try_from(stage)
                && stage < 16
            {
                records.stages_passed |= 1 << stage;
            }
        }
        self.records.set(records);
        records
    }

    /// **The records out of the file the title's own read landed on**, and none where there is no such file
    /// — which is a copy nothing has been cleared on.
    fn loads_its_records(&self, landed: &str) {
        let bytes = self.sim().files().get(self.launch().dir().join(landed));
        self.records
            .set(bytes.map_or_else(Records::default, |bytes| Records::read(&bytes)));
    }

    /// **And the records written back**, through whichever file an open of the game's own lands on — which is
    /// the mode's own where orb has forked the opens, so what a Pointdevice run writes is not what the title
    /// reads.
    ///
    /// **And nothing where the open was refused**, which orb does to every write of the score file while an
    /// attempt is watched or a run is cleared on demand: the game's own save then fails and writes nothing.
    fn saves_its_records(&self) {
        let opened = self.opens.borrow().len();
        its_score_file_opened(true);
        let landed = self.opens.borrow().get(opened).cloned();
        if let Some(landed) = landed {
            self.sim()
                .files()
                .put(self.launch().dir().join(landed), self.records.get().bytes());
        }
    }

    /// One update of the stage the run is in, where the run is one this game's own front end started: the
    /// clock, the script, the bullets and the player the word moves.
    ///
    /// **And whether the chain's walk got as far as the replay's own element**, which is what copies the next
    /// frame's word — see [`Fake::update`].
    fn plays_the_stage_it_is_in(&self, word: u16) -> bool {
        // **And nothing on the update a build is asked on**, whose stage is the one being left: the switch
        // takes that stage's jobs down and the thread has put none of the next one's up yet. Played on that
        // update instead, the stage ended a second time on the count the stage before had left, and the
        // laid-out run went from stage 1 straight to stage 3.
        if self.image.the_loader_is_running() {
            return false;
        }
        self.the_stages_jobs_run(word)
    }

    /// **The jobs of the stage the run is in, one update of them** — on every frame the walk reaches them,
    /// which is every frame of a stage and every frame its build goes on for once the thread has put them on
    /// the chain: see [`Fake::its_loader_goes_on`].
    fn the_stages_jobs_run(&self, word: u16) -> bool {
        if !self.image.a_run_is_on() {
            return true;
        }
        // **The player moved in every run, placed or played**: the real game moves its player by the word
        // whatever the stage is doing, and a laid-out run whose player stood still under the word was one
        // the rules read as never arriving — judging each frame from the step they had just chosen, they
        // pressed left and right by turns to get back to a place they had never left.
        self.the_player_read
            .set((word, self.image.the_word_the_run_acted_on()));
        if self.the_players_state_goes_on() {
            self.moves_its_player(word);
        }
        // **And the timer every live enemy carries, in every run and not only in a stage this file drives**:
        // the enemy manager's update steps it once a frame wherever a run is on, and the count orb reads on a
        // frame is the one the enemy had before that step — the manager's job being registered under the
        // supervisor's, which is where orb's read sits. Above the gate below for that reason: a run whose
        // enemies never aged is one whose every body reads as a slot given away since last frame, so nothing
        // of a body's own motion is read at all — see `Hazard::frames` and `Image::an_enemy_is_new_in_its_slot`.
        self.image.its_enemies_are_a_frame_older();
        self.image.its_bullets_are_a_frame_older();
        if !self.on_its_own.get() {
            return true;
        }
        let state = self.state();
        // **The run manager's sum of the run, every frame**, which is the half of the seal a check holds the
        // kept sum against — see `Image::the_run_manager_takes_its_sum`.
        self.image.the_run_manager_takes_its_sum();
        // **The stage over, where an e2e test said how long one lasts**, and the stage after it the one the
        // game's own chooser would pick — see [`A_RUNS_STAGES`]. **And the run over where there is no stage
        // after it**, which is what the run manager does with the last of them: 0x439d4d ends the run
        // instead of choosing where the stage word is already 6, 7 or 8.
        //
        // **A laid-out run that played on past its last stage was the one thing keeping the whole-run case
        // from finishing.** Every `play` step ends the moment the run is over, so a run that never ends is
        // a recipe whose every remaining step sits out its own bound: that case spent all hundred thousand
        // frames the laid-out game gives it and never reached its own `quit`.
        //
        // **The stage's own frames counted from its own build**: the clock the game steps is the run's and
        // carries on across a build — see `the_stage_is_laid_out` — so this file keeps what it read as the
        // stage was built, the way `Th08::read_state` keeps its own. **Its own and not orb's**, orb's being
        // a reading of this file's stage and this being the stage: a laid-out game that ended its stages on
        // what orb read of them would be orb's reading tested against itself.
        // **At or past it, rather than on it**: the run's clock is put *forward* by the instruction that
        // starts a conversation — 1,800 in one write, see below — so a stage whose end this tested for
        // equality would be a stage a conversation's own jump could step over and never end.
        let into_the_stage = self
            .image
            .the_frames_the_run_has_been_going()
            .wrapping_sub(self.stage_built_at.get());
        // **The stage's boss arriving for its end** — see [`THE_BOSS_FIGHTS_FOR`] — **to her own track**,
        // which the conversation before her puts on (0x434dce) and which is what tells her from a midboss —
        // see `Th08::the_fights_own_song_is_playing` — and gone with the stage, whose next build takes her off.
        if let Some(last) = self.stage_frames.get()
            && into_the_stage + THE_BOSS_FIGHTS_FOR >= last
            && into_the_stage + AFTER_THE_LAST_SPELL < last
            && !self.the_boss_is_up.get()
        {
            self.the_boss_is_up.set(true);
            // **The count first and the boss over it**: `enemies_alive` writes its slots' flags whole, and
            // written after, it took the boss's own bit off the slot she is in — so no fight of this file's
            // stages had a boss orb could read, `State::boss_life` and `boss_attack_frames` both `None`.
            self.image.enemies_alive(1);
            self.image
                .a_boss_is_fighting(THE_BOSSS_SLOT, THE_BOSS_STANDS_AT, THE_BOSSS_LIFE);
            self.image.the_stream_reads_the_track(THE_FIGHTS_TRACK);
        }
        // **And beaten before the stage ends**, the frames between being the stage's own clear — which is
        // what a real stage's boss going down leaves before its next stage is built, and what says the stage
        // is won: a boss up to the build's own frame was a stage no step inside it could come to rest at the
        // end of.
        if let Some(last) = self.stage_frames.get()
            && into_the_stage + AFTER_THE_LAST_SPELL >= last
            && self.image.a_boss_is_up()
        {
            self.image.no_boss_is_fighting();
            self.image.enemies_alive(0);
        }
        // **And her fight a real boss's** — see [`HER_CARD`]. By the card being up or not rather than by one
        // frame, the run's clock being one a conversation can step over.
        if state.stage != EIRINS_STAGE
            && let Some(last) = self.stage_frames.get()
            && self.image.a_boss_is_up()
        {
            let arrived = last.saturating_sub(THE_BOSS_FIGHTS_FOR);
            let card_up = self.state().spellcard.is_some();
            let from = arrived + HER_CARD_AFTER;
            let to = from + HER_CARD_LASTS;
            if (from..to).contains(&into_the_stage) && !card_up {
                self.image.a_card_is_up(HER_CARD);
                self.image.the_boss_has_attacked_for(THE_BOSSS_SLOT, 0);
            } else if into_the_stage >= to && card_up {
                self.image.no_card_is_up();
                self.image.the_boss_has_attacked_for(THE_BOSSS_SLOT, 0);
            }
        }
        // **And Eirin's Last Spell up for her stage's end** — see [`THE_LAST_SPELL`].
        if state.stage == EIRINS_STAGE
            && let Some(last) = self.stage_frames.get()
        {
            let over = last.saturating_sub(AFTER_THE_LAST_SPELL);
            if into_the_stage >= over {
                self.image.no_card_is_up();
            } else if into_the_stage + THE_LAST_SPELL_LASTS >= over {
                self.image.a_card_is_up(THE_LAST_SPELL);
            }
        }
        if self
            .stage_frames
            .get()
            .is_some_and(|last| into_the_stage >= last)
        {
            let records = self.writes_down_the_stage(state.stage);
            match the_stage_after(state.stage, records.eirins_stage_passed) {
                Some(next) => self.moves_on_to_the_stage(next),
                None => self.the_run_is_over(state.stage),
            }
            return true;
        }
        // **The conversation, which is the one thing that holds the run's clock while the run plays on.**
        // Instruction 148 of the real stage script hands its line to the conversation and adds `0x708` —
        // 1,800 — to the run clock on the spot (0x41d6cd), and the step below is gated on no line being up
        // (0x42c6a1 against 0x4358bb): so the clock takes a number, skips 1,800 of them and then stands
        // still for as long as the line is shown. **What does not stand still is the run**, which is what a
        // launch of the real game read off the 103 updates it spent on frame 7,179 — the player walking from
        // `293,368` to `253,352`, four items collected and the midboss arriving from off the field — and it
        // is why `resume::Again` keeps every read of a frame and not the last.
        //
        // The script's own clock is not held: it went 4223 to 4283 across a real conversation while the run
        // clock stood at 17,132 — see `asm.md`'s *The dialogue does not stop it*.
        let carrying = match self.conversation.get() {
            0 => match self.a_conversation_at.get() {
                Some((at, lasts)) if state.stage_frames + 1 == at => {
                    self.conversation.set(lasts);
                    self.image.a_line_of_dialogue_is_up(0);
                    self.image.has_been_in_the_run_for(
                        self.image.the_frames_the_run_has_been_going() + THE_CLOCK_A_LINE_COSTS,
                    );
                    true
                }
                _ => false,
            },
            left => {
                self.conversation.set(left - 1);
                if left == 1 {
                    self.image.no_dialogue_is_up();
                }
                true
            }
        };
        // **The two clocks a frame of a run steps**, both inside the one update: the run's own at 0x42c6ad,
        // behind the gate above, and the stage script's first context at 0x42ad57, which no line holds.
        // Off the game's own clock and not off the state, whose `stage_frames` is orb's count from the
        // stage's build: written back from that, the run's clock would start again with every stage.
        // And the skip with no line, which is no gate on the step: 1,800 and the frame's own one.
        if self.a_skip_at.get() == Some(state.stage_frames + 1) {
            self.image.has_been_in_the_run_for(
                self.image.the_frames_the_run_has_been_going() + THE_CLOCK_A_LINE_COSTS,
            );
        }
        if !carrying {
            self.image
                .has_been_in_the_run_for(self.image.the_frames_the_run_has_been_going() + 1);
        }
        self.image
            .the_stage_script_has_run_for(u32::try_from(state.script_frames).unwrap_or(0) + 1);
        // And the generator drawn from, which is what every frame of a run does: a stage whose word never
        // moved is one orb reads as a frame nothing happened on. The step is this file's own — what it has
        // to be is a word that moves, this game's own draw being 0x4425c0 and unread.
        self.image.the_generator_has_drawn(
            self.image
                .the_generators_word()
                .wrapping_mul(0x343f)
                .wrapping_add(0x269e),
        );
        // **And the run's seal checked on every two hundredth frame of the enemy manager's own timer**, the
        // manager's update answering 4 where it is broken — which leaves the chain's walk behind it, the
        // bullets' job and the replay's copy of the word among what goes unrun — and not stepping that timer
        // on the way out, so the next frame is the same check (0x42de49 through 0x42de78). See `Th08`'s
        // `RUN_SEAL_KEY`.
        let timer = self.enemy_frames.get();
        if timer.is_multiple_of(THE_SEAL_IS_CHECKED_EVERY)
            && !self.image.the_run_is_as_it_was_sealed()
        {
            return false;
        }
        self.enemy_frames.set(timer + 1);
        // The pattern counted from the stage's own first frame and not the run's, so that every stage of a
        // run fires the same one — where an e2e test has not asked for a stage that fires nothing.
        if self.fires.get() {
            self.fires_at_the_player_on(state.stage_frames + 1);
        }
        self.moves_its_bullets();
        self.takes_the_hit_it_is_in();
        true
    }

    /// The stage's own script: a bullet aimed at where the player is, from the top of the field, every
    /// [`A_BULLET_EVERY`] frames from [`FIRST_BULLET_AT`].
    fn fires_at_the_player_on(&self, frame: u32) {
        if frame < FIRST_BULLET_AT {
            return;
        }
        if !(frame - FIRST_BULLET_AT).is_multiple_of(A_BULLET_EVERY) {
            return;
        }
        let at = self.image.where_the_player_is();
        let (dx, dy) = (at.0 - BULLETS_FROM.0, at.1 - BULLETS_FROM.1);
        let far = (dx * dx + dy * dy).sqrt().max(1.0);
        let step = (dx / far * BULLET_SPEED, dy / far * BULLET_SPEED);
        let mut bullets = self.bullets.borrow_mut();
        // **The first slot no bullet of this stage is in**, as the manager's own allocation takes one, and not
        // the count of those in flight: with one up, that count named that bullet's own slot and took it off
        // the field — a bullet is `185` frames from the top to where the player stands and one came every
        // `120`, so none ever reached a player past the stage's start.
        let Some(slot) =
            (0..BULLET_SLOTS_USED).find(|slot| bullets.iter().all(|bullet| bullet.slot != *slot))
        else {
            return;
        };
        bullets.push(Bullet {
            slot,
            at: BULLETS_FROM,
            step,
        });
    }

    /// And every bullet it has out one step further on, with the ones past the bottom of the field taken
    /// off — which is what the manager's own walk does.
    fn moves_its_bullets(&self) {
        let mut bullets = self.bullets.borrow_mut();
        for bullet in bullets.iter_mut() {
            bullet.at = (bullet.at.0 + bullet.step.0, bullet.at.1 + bullet.step.1);
        }
        let gone: Vec<usize> = bullets
            .iter()
            .filter(|bullet| bullet.at.1 > FIELD.1)
            .map(|bullet| bullet.slot)
            .collect();
        bullets.retain(|bullet| bullet.at.1 <= FIELD.1);
        for slot in gone {
            self.image.the_bullet_is_gone(slot);
        }
        for bullet in bullets.iter() {
            self.image.a_bullet_is_at(
                bullet.slot,
                bullet.at,
                bullet.step,
                (BULLET_SIZE, BULLET_SIZE),
            );
        }
        self.image
            .the_bullet_manager_counts(i32::try_from(bullets.len()).expect("a field of bullets"));
    }

    /// The player moved by the word, at the speed the team's own `.sht` holds for the direction and the
    /// focus — [`A_STEP`], which is the four one launch of the real game measured.
    fn moves_its_player(&self, word: u16) {
        let across = i32::from(word & BUTTON_RIGHT != 0) - i32::from(word & BUTTON_LEFT != 0);
        let down = i32::from(word & BUTTON_DOWN != 0) - i32::from(word & BUTTON_UP != 0);
        if across == 0 && down == 0 {
            return;
        }
        let focused = word & BUTTON_FOCUS != 0;
        let diagonal = across != 0 && down != 0;
        let step = match (focused, diagonal) {
            (false, false) => A_STEP.0,
            (false, true) => A_STEP.1,
            (true, false) => A_STEP.2,
            (true, true) => A_STEP.3,
        };
        let (x, y) = self.image.where_the_player_is();
        let at = (
            // Held inside the four the game holds its player in — 8 and 16 in from the area's own
            // corner, 368 by 416 — and not the area: the real player's update puts a position past them
            // back (0x44baf8 through 0x44bbc9), so its floor is 432 on a field 448 deep.
            (x + across as f32 * step).clamp(8.0, 8.0 + 368.0),
            (y + down as f32 * step).clamp(16.0, 16.0 + 416.0),
        );
        self.image.the_player_is_at(at);
    }

    /// **Whether the player is standing in a bullet**, which is asked of a player that can be hit and of no
    /// other: the game's three box tests decline while the state byte is not nothing (0x44a333, 0x44a445,
    /// 0x44a8f2). A hit is state 2 with its count from nothing — see [`PLAYER_PLAYING`] for what the player's
    /// own update makes of it.
    fn takes_the_hit_it_is_in(&self) {
        if self.image.what_the_player_is_doing() != PLAYER_PLAYING {
            return;
        }
        let (x, y) = self.image.where_the_player_is();
        let half = BULLET_SIZE / 2.0;
        let hit = self
            .bullets
            .borrow()
            .iter()
            .any(|bullet| (bullet.at.0 - x).abs() < half && (bullet.at.1 - y).abs() < half);
        if hit {
            self.image.the_player_has_been_hit();
        }
    }

    /// **The part of the player's own update its state byte decides**, and whether the player moves on this
    /// update — see [`PLAYER_PLAYING`], where each step is read off the exe.
    ///
    /// **And the life a hit run out takes**, with the field cleared of this file's own bullets as the stage
    /// puts the player back — which is the whole of what a run with the bomb forbidden has to do about one.
    fn the_players_state_goes_on(&self) -> bool {
        let mut state = self.image.what_the_player_is_doing();
        let count = self.image.the_frames_the_hit_has_counted();
        if state == PLAYER_HIT && count >= A_COUNT_RUNS_TO {
            let numbers = self.state();
            self.image
                .has_lives_and_bombs((f32::from(numbers.lives) - 1.0).max(0.0), BOMBS);
            self.image.has_died(numbers.deaths as f32 + 1.0);
            self.image.the_player_comes_in(0);
            self.bullets.borrow_mut().clear();
            self.image.bullets_alive(0);
            state = PLAYER_COMING_IN;
        } else if state == PLAYER_COMING_IN && count >= A_COUNT_RUNS_TO {
            // A spell practice keeps the count it came in with (0x44d28f skipping the write).
            let untouchable_for = if self.image.the_run_practises_a_card() {
                count
            } else {
                UNTOUCHABLE_FOR
            };
            self.image.the_player_is_untouchable_for(untouchable_for);
            state = PLAYER_UNTOUCHABLE;
        }
        let count = self.image.the_frames_the_hit_has_counted();
        if state == PLAYER_UNTOUCHABLE {
            if count - 1 > 0 {
                self.image.the_player_is_untouchable_for(count - 1);
            } else {
                self.image.the_player_is_playing_where_it_stands();
            }
        } else {
            self.image.the_hit_has_counted(count + 1);
        }
        state == PLAYER_PLAYING || state == PLAYER_UNTOUCHABLE
    }
}

/// Which of the nine screens this walk knows a number is, `None` for one it does not — see
/// [`Fake::walks_its_front_end`], which declines a screen rather than guessing at it.
///
/// Nine because nine is what a recipe of this game walks — three to a run, one to a replay, the Extra's own
/// two and a stage practice's three: the other nine the game's own switch covers are screens no e2e test here
/// reaches, and a walk that guessed at one would be this game claiming to know something the exe has not been
/// read for.
fn the_screen(number: i32) -> Option<Screen> {
    [
        Screen::Title,
        Screen::Difficulty,
        Screen::Team,
        Screen::Replays,
        Screen::DifficultyOfAPractice,
        Screen::TeamOfAPractice,
        Screen::StageOfAPractice,
        Screen::DifficultyOfTheExtra,
        Screen::TeamOfTheExtra,
    ]
    .into_iter()
    .find(|screen| *screen as i32 == number)
}

impl Drop for Fake {
    /// **The game closing, which takes orb down with it** — the runtime first, then the fields in the order
    /// they are declared, as every other game's `Fake` closes. Without it the thread orb writes its own
    /// files on outlives the launch and carries that launch's simulated Windows into the next — see
    /// `orb_core::writer::stop`: a run started over in a second launch read `2 chapter(s), 2 attempt(s)
    /// written to …pending` and left no such file in its own directory.
    fn drop(&mut self) {
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The exe this game is, which is how `orb_core::game::KNOWN` is asked for it.
///
/// # Panics
/// Where no entry holds that name.
pub const EXE: &str = "th08.exe";

/// **The walk from the title to a run of the mode, as a recipe**, which is the real recipes' own — see
/// `orb-real/recipes/th08/a_death_answered_by_the_chapter_again.yaml` for why each step is shaped so — and the
/// three hundred frames of the run's opening it presses nothing through.
///
/// **Here rather than in one of the e2e modules that plays it**, because it is this game's screens and every
/// module over a run of it starts by walking them: a screen number changed in one copy of it would leave the
/// others walking a game that no longer exists. What a module writes out for itself is what it *asserts*.
pub(crate) const TO_A_RUN: &str = "\
- wait_until: { screen: 0 }
  give_up_after: 1800
- press: [up]
  frames: 30
  until: { cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- answer: { question: mode, item: 0 }
  give_up_after: 300
- wait_until: { screen: 4 }
  give_up_after: 1800
- press: [decide]
  frames: 30
  until: { screen: 5 }
  give_up_after: 900
- press: [decide]
  frames: 12
- wait_until: { frames: 120 }
- press: [decide]
  frames: 12
- wait_until: { frames: 300 }
";

fn the_game_this_is() -> &'static dyn orb_core::game::Game {
    orb_core::game::found(EXE)
        .unwrap_or_else(|| panic!("{EXE} is a game orb knows"))
        .game
}

/// `Chain::RunCalcChain` at 0x43ca50: every job of the frame's update, in one call.
///
/// What orb stands in front of is the walk and not an element of it — see `Th08::hooks` — so a laid-out
/// game's whole update is what this answers for, and [`CHAIN_CARRIED_ON`] is the count of the one job it
/// ran.
extern "fastcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.update();
    fake.answers.get()
}

/// `Chain::RunDrawChain` at 0x43cb60: one quad queued, which is what drawing anything in this game is.
///
/// **The whole of why this game has a queue laid out.** A real draw chain reaches the sprite writer for
/// every sprite on the screen and that writer fills where `[queue+0x2a2524]` points — so a frame that
/// never emptied the queue writes its first quad past the end of it, which is the fault orb's first loop
/// over th07's frame took.
extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.image.queues_a_quad();
    CHAIN_CARRIED_ON
}

/// `GameWindow::Render` at 0x441e70: the game's own whole frame, in this game's own order.
///
/// **Which is the update before the draw**, unlike th07's: read off 0x441e70 to 0x442051 — the time, the
/// limiter, the queue drawn once (0x441f02), the whole-output viewport, the calc chain (0x441f4d), the
/// sound queue (0x441f5a), the frame-skip counter, `BeginScene`, the queue emptied (0x441fce), the fog
/// flag forced and put out (0x441fe2), the draw chain (0x441fec), the queue drawn (0x441ff7),
/// `SetTexture(0, null)`, `EndScene`, the time again and the present (0x44203d).
///
/// What orb's own loop replaces, and what that loop hands a frame back to on each of the three ways out of
/// it that return: no runtime, no device, and a chain target that is null.
///
/// The chain's two exits become the frame's own two here as they do in the real one, that mapping being
/// this game's — see [`FRAME_LEFT`]. Without the frame-skip counter the real one has: what orb's loop
/// replacing that frame does is take the skipping out, so a frame here is one update and one draw.
extern "fastcall" fn own_render(_window: *mut c_void) -> i32 {
    let fake = running();
    let chain = Th08.chain();
    // The queue drawn before anything of the frame, which is where this game's own frame draws it: 0x441f02,
    // ahead of the whole-output viewport and of the update.
    unsafe { draws_the_queue(0) };
    let walked = unsafe { orb_core::runtime::run_calc_chain(chain) };
    unsafe { play_sounds(0) };
    if walked == CHAIN_LEFT {
        return FRAME_LEFT;
    }
    if walked == CHAIN_FAILED {
        return FRAME_FAILED;
    }
    unsafe { empties_the_queue(0) };
    fake.image.forces_the_fog_call_through();
    unsafe { puts_the_fog_out(0) };
    unsafe { orb_core::runtime::run_draw_chain(chain) };
    unsafe { draws_the_queue(0) };
    unsafe { present(0) };
    FRAME_KEPT_RUNNING
}

/// `SoundPlayer::PlaySounds` at 0x45d790: the commands standing in the player's own queue carried out, and
/// the time an e2e test says this frame's sounds cost.
///
/// Here rather than left out because the frame loop calls it where the game's own loop did, and a frame
/// that skipped it would be one span of the pacing's breakdown short. Nothing of the commands, this game's
/// sound system being a reading not yet made — `Th08::music` answers nothing.
unsafe extern "fastcall" fn play_sounds(_player: usize) {
    let fake = running();
    fake.launch.asked_for(SOUND);
    fake.sim().clock().advance_micros(fake.sound_this_frame());
}

/// The frame handed over at 0x442060, which from here is the compositor's.
///
/// Where an e2e test counts a frame — the tick it was handed over at, which is what a rate is read off —
/// and where the host is told, since the next flush is what waits for this frame to be composed.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
    // And the device's own `Present`, which is what that function does in the game — four nulls through
    // `+0x3c` at 0x44207f — and the one path orb's letterbox is on. The slot is read out of the vtable
    // rather than remembered, because what is in it is whatever orb last put there.
    //
    // Only where the vtable is laid out, which is a launch whose device has been found.
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
/// the two rectangles the ask carried, and whichever answer an e2e test declared.
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
/// `D3DERR_DRIVERINTERNALERROR`, which is what such a refusal really comes back as — orb reads the sign
/// and nothing else.
const S_OK: orb_api::Hresult = 0;
const STRETCH_REFUSED: orb_api::Hresult = 0x8876_0827u32 as i32;

/// The three the frame makes on that queue and on the fog, `__thiscall` on what the game keeps each of
/// them in: 0x462e00, 0x462e40 and 0x448229.
///
/// What they do to this game's memory is the image's, the block being laid out there — see
/// `Image::empties_the_queue`. Handed over rather than reached at their own addresses, for the reason
/// every call out of the seam is: an address space holds no code.
unsafe extern "thiscall" fn empties_the_queue(_queue: usize) {
    running().image.empties_the_queue();
}

unsafe extern "thiscall" fn draws_the_queue(_queue: usize) {
    running().image.draws_the_queue();
}

/// **The run's sum**, 0x439a2e, `__thiscall` on the game manager — what orb takes again over the numbers a
/// resume puts back. See `Image::the_runs_checksum`.
unsafe extern "thiscall" fn sums_the_run(_manager: usize) -> u32 {
    running().image.the_runs_checksum()
}

/// **The supervisor's update**, 0x445453, `__thiscall` on the supervisor — which orb calls alone on a frame a
/// build is going on, see `Th08::walks_a_frame_of_its_build`. **As far as such a frame reaches**, the thread
/// served and 1 answered: orb calls nothing else of this laid-out game's that way.
unsafe extern "thiscall" fn supervisor_update(_supervisor: usize) -> i32 {
    running()
        .the_supervisor_sits_out_a_build()
        .unwrap_or(CHAIN_CARRIED_ON)
}

/// And the fog, which draws the queue first as the game's own does: what was queued under fog is drawn
/// under it, and putting the fog out before drawing it would draw it without.
///
/// **The field is what decides whether the device is told**, which is this game's own shape and the whole
/// of why its frame writes that field first — a call whose field already says the fog is out sets no
/// render state at all. What reaches the device is counted here rather than at the device itself: the
/// `SetRenderState` is not laid out, the device in an e2e test being a real object with a vtable of Rust
/// functions.
unsafe extern "thiscall" fn puts_the_fog_out(_supervisor: usize) {
    let fake = running();
    fake.image.draws_the_queue();
    if fake.image.the_fog_flag() != 0 {
        fake.image.puts_the_fog_out();
        fake.fog_told.set(fake.fog_told.get() + 1);
    }
}

/// The game's own read of a frame's buttons, which orb calls through: the keyboard folded into one word,
/// the way 0x43d970 folds it.
///
/// `GetKeyboardState` is the read this game really makes — 0x43d996 calls the second entry of the USER32
/// import table at 0x4b41c8 with a 0x100-byte buffer and tests each key's byte against 0x80 — so a key
/// another program sent is a key this game reads, and a read the host refuses leaves the game with nothing
/// held rather than with what a stack buffer happened to hold.
///
/// **And no pad half after it**, unlike th07's: `Th08::hooks` declines the pad, this game's own read being
/// DirectInput8 and not read.
extern "system" fn input() -> u16 {
    let fake = running();
    let state = fake.sim().keyboard().state();
    MAP.iter()
        .filter(|(key, _)| {
            state
                .as_ref()
                .is_some_and(|state| state[*key as usize] & 0x80 != 0)
        })
        .fold(0, |word, (_, bit)| word | bit)
}

/// **The job the build calls three instructions past its seed's copy**, at 0x409b20: nothing of its own,
/// the seed the stage is built with being `Fake::builds_the_stage`'s write, and success is what the real
/// callback answers.
///
/// What orb does in front of it is `Th08::seed`, which is the whole reason the seam is inside the build
/// rather than in front of it.
extern "C" fn stage_building(_stage: i32) -> i32 {
    0
}

/// **The run manager's own added callback**, at 0x43aaf4, entered with the game manager in `ecx` the way
/// the chain calls it: nothing of its own, a stage's numbers being `Fake::is_in_a_run`'s writes.
///
/// What orb does behind it is `Th08::put_the_numbers_back`, which is the other half of a resume.
extern "C" fn stage_begun(_manager: *mut c_void) -> i32 {
    0
}

/// The name this game opens its score file by, which is its own and not orb's.
const SCORE_FILE: &CStr = c"score.dat";

/// **An open of the score file through whatever stands in front of this game's `CreateFileA`** — orb's hook
/// where a launch installs it, and this file's own otherwise.
fn its_score_file_opened(write: bool) {
    const GENERIC_READ: u32 = 0x8000_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    let open = if orb_core::score::installed() {
        orb_core::score::create_file_a
    } else {
        create_file
    };
    unsafe {
        open(
            SCORE_FILE.as_ptr().cast(),
            if write { GENERIC_WRITE } else { GENERIC_READ },
            0,
            std::ptr::null(),
            0,
            0,
            std::ptr::null_mut(),
        )
    };
}

/// **The front end's own added callback**, `Th08`'s `FRONT_END_ADDED`: the title's load, whose one act that
/// matters to orb is its read of the score file — the records the title lights its items from.
extern "C" fn unlocks_read(_menu: *mut c_void) -> i32 {
    its_score_file_opened(false);
    0
}

/// The ranking's read and the card records' seam, which `Th08::hooks` declines: handed over to be a valid
/// `Originals` and for nothing more.
extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    0
}

unsafe extern "fastcall" fn card_records_read(_file: usize, _into: usize) -> i32 {
    0
}

/// And its recorder, its replay write and its music, which are the same story: nothing of any of them has
/// been read, `Th08::rewinds` answers false, and no launch here reaches one.
extern "C" fn stop_recording() {}

extern "C" fn save_replay(_path: *const u8, _name: *const u8) {}

/// **The game's own refill**, which [`Fake::refills_its_music`] reaches through orb's hook over it — see `Th08`'s
/// `TOPS_UP_THE_MUSIC`: one chunk in at the next write, the next write a chunk on round the buffer, and the
/// countdown to the loop a chunk down.
extern "thiscall" fn tops_up_the_music(_stream: usize, _argument: i32) -> i32 {
    let image = &running().image;
    let (write, chunk, left) = image.where_the_stream_is();
    image.the_stream_is_at(
        (write + chunk) % (A_MUSIC_CHUNK * CHUNKS_THE_BUFFER_HOLDS),
        chunk,
        left.saturating_sub(chunk),
    );
    0
}

extern "thiscall" fn stops_the_music(_stream: usize) -> i32 {
    0
}

/// And its own `joyGetPosEx`, which is never called: `Th08::hooks` declines the pad, so nothing here ever
/// reads one.
unsafe extern "system" fn joystick_position(_device: u32, _into: *mut orb_api::JoyInfo) -> u32 {
    orb_api::joyerr::PARMS
}

/// And its read of the pad's buttons by number, declined for the same reason.
extern "C" fn pad_buttons() -> *mut c_void {
    std::ptr::null_mut()
}

/// **`GameWindow::CreateGameWindow` at 0x435640**, which is the call everything about this game's window is
/// decided inside: nothing of its own, a laid-out launch asking for no window until
/// [`Fake::creates_its_window`] does.
///
/// What orb's hook in front of it is for is the windowed flag underneath it — see `Th08::force_windowed`.
extern "C" fn create_game_window(_instance: *mut c_void) {}

/// And its device setup at 0x435810, which orb gets in front of to redirect `Present` before anything is
/// presented through it: nothing of its own, the device and its vtable being
/// [`Fake::finds_its_device`]'s writes.
extern "C" fn init_d3d_device() {}

/// And its `CreateFileA`, which every file this game opens goes through: the name the call landed on written
/// down, and a handle that is its place in that list.
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
/// Windows has: where it is, how big, and the client area inside it, which is what orb's letterbox is
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
