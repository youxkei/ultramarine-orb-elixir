//! A 東方紅魔郷 that is not the real one: it owns its own memory, advances its own state, and calls
//! orb's hooks where the real game's code calls them.
//!
//! **It plays the game's part, not the address space's.** Nothing here hands orb an opinion about
//! the run. Its state is the memory laid out by `image`, so `read_state` is how orb learns what the
//! run is — as in production — and a chapter restored underneath it takes the run back with it,
//! because there is nothing else for the run to be. Anything kept beside that memory would be a
//! second source of truth, which is the mistake this replaced: an e2e test that told orb what the
//! state was while the memory said something else.
//!
//! **Its own loop calls orb's frame loop, as the real game's does.** The `Present` and the sound that
//! loop makes are addresses the game hands over — see `Game::frame_calls` — so this game hands over two
//! of its own, and its `present` is where an e2e test counts a frame handed over. A launch started
//! `--no-frame-loop` is the game's own draw-then-update order instead, with orb's update and draw hooks
//! in the middle of it, which is the other configuration orb ships.
//!
//! **Everything a launch has that is not this game is [`Launch`]** — the display, the device orb draws
//! through, what a frame's own work costs, and the frames themselves. This file is the half a second
//! game would bring one of its own of.
//!
//! Where a number here is 紅魔郷's it says so. Where it is this game's own — how far the player moves,
//! when its boss arrives — it says that too: what an e2e test is about is that the same buttons from
//! the same seed arrive at the same place, not how fast Reimu is.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{CStr, c_void};

use orb_config::Config;
use orb_core::game::th06::Th06;
use orb_core::game::th06::image::{
    Boss, FrontEnd, Image, Mapping, Player, Playing, Pushed, Reproducing, Scene, Screen,
    Supervising, Track, chain_job, chain_result, item, joy_state, result_state,
};
use orb_core::game::{Game, RunStart};
use orb_sim::Quad;
use orb_sim::keys;

use super::{
    DRAW, Display, Launch, Launched, PRESENT, Panel, READS_KEYS_AFTER, SOUND, UPDATE, WINDOW, Work,
    scratch,
};

/// What the game's own chain walk answers while the game is running.
///
/// Neither of the two below, which orb reads as the game leaving. A real run whose resume played 7476
/// updates in inside one frame is measured proof that the real walk does not answer zero
/// while a stage is running, since orb's playback stops on that — and
/// `pointdevice_run.rs` plays 700 in one frame the same way.
const CHAIN_CARRIED_ON: i32 = 1;

/// And what it answers when the game is leaving: zero, which orb reads as the game having asked to stop,
/// and `-1`, which is the walk having failed. Both of 紅魔郷's own.
pub const CHAIN_LEFT: i32 = 0;
pub const CHAIN_FAILED: i32 = -1;
/// And what a walk a job broke out of answers, which is 紅魔郷's own **1**: `Chain::RunCalcChain` returns
/// that for `CHAIN_CALLBACK_RESULT_BREAK` and the count of jobs it ran otherwise — so a walk that ran one job
/// and a walk a job broke are the same number to whatever is above, and both are a game carrying on.
const CHAIN_BROKE: i32 = 1;

/// More jobs than the chain has, which is what a walk that has run this many says: 紅魔郷 registers one per
/// priority from 0 to 0x10, and this game registers six of those. A list whose links do not end is a walk
/// that does not either, and saying so names the registration that made it — orb's own walks bound
/// themselves the same way, at `CHAIN_LINKS`.
const CHAIN_JOBS: i32 = 64;

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
pub(crate) mod button {
    pub const SHOOT: u16 = 0x0001;
    pub const BOMB: u16 = 0x0002;
    /// What holds the player still, and the one of these the game sets from something other than a
    /// button where the mapping puts focus and shoot on one — see `Th06::buttons_of`.
    pub const FOCUS: u16 = 0x0004;
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

/// And a class that is not it, for the other windows a game makes — see
/// [`creates_another_window`](Fake::creates_another_window). Any name but the one above; what an e2e test
/// asks of it is that orb left the ask alone.
const OTHER_WINDOW_CLASS: &CStr = c"th06-notify";

/// The score file 紅魔郷 asks for, which is the only name this game ever passes `CreateFileA`.
///
/// Which file the open *lands* in is orb's answer and not this one's — the whole subject of
/// `the_score_file.rs` — so a game that named the forked file itself would be answering the
/// question being asked.
const SCORE_FILE: &CStr = c"score.dat";

/// `GENERIC_WRITE`, which is the one bit of `CreateFileA`'s access this game varies: the score file is read
/// in three places and written in one, and orb reads that bit to tell a refused write from a read.
const GENERIC_WRITE: u32 = 0x4000_0000;

/// What `CreateFileA` answers where it could not open the file — `INVALID_HANDLE_VALUE`, which is what
/// orb's own refusal of a write answers with and what this game reads as a first launch.
const NO_HANDLE: isize = -1;

/// What this game's score file holds in its `clrd` chunk where the game has been cleared: one `Clrd` record
/// for the shot this game's runs are played with, cleared the **99** times `HasReachedMaxClears` compares
/// against.
///
/// A whole record and not a magic, because the gate is a count and not a flag: a record with the right magic
/// and its clear counts at anything else is a record the front end reads as a game nobody has cleared, which
/// is exactly what a failed read leaves behind — see `Image::parses_the_unlocks`. What the bytes *are* is
/// `Image::cleared_record`'s, the layout being that file's business.
fn cleared() -> Vec<u8> {
    Image::cleared_record(the_run().shot_type)
}

/// One open of the score file: the name it landed in, and whether it was for writing.
///
/// Which is the whole of what crosses `CreateFileA` and the whole of what an e2e test about that file reads
/// back. There is no file on any disk — see [`Fake::opens`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Open {
    pub path: String,
    pub write: bool,
}

/// What one of this game's score files holds: the record about spell cards, and beside it what has
/// been cleared.
///
/// Two of the four chunks a real one holds, and they are here because the three reads of the file are
/// told apart by which chunks each parses: the front end's own takes the unlocks and nothing else,
/// and the stage's and the ranking's take both. `hscr`, the ranking itself, is not modelled — nothing
/// above the game reads it.
#[derive(Clone, Default)]
struct ScoreFile {
    /// The bytes `Game::captures` and `Game::set_captures` are the two halves of.
    captures: Vec<u8>,
    unlocks: Vec<u8>,
}

/// What the game asks `CreateWindowExA` for, and every one of these is replaced.
///
/// This game's own numbers rather than 紅魔郷's, and deliberately nothing like the answer: what an
/// e2e test asserts is the window that came out, so an ask that already resembled it would be an ask
/// that could not tell a rewrite from a pass-through. The style is a caption and a system menu, which
/// is a window with a frame — the one thing about the ask that is real, since it is what the game
/// would have got.
pub const ASKED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x1000_0000;
pub const ASKED_AT: (i32, i32) = (17, 23);
pub const ASKED_SIZE: (i32, i32) = (646, 505);

/// The two arrow keys orb's own menus do not read, so `orb_sim::keys` does not name them.
///
/// The four it does read come from there, which is what makes an e2e test pressing `Z` at one of orb's
/// questions and this game reading its shot key the same key by construction rather than by two
/// numbers that happen to agree.
const LEFT: u8 = 0x25;
const RIGHT: u8 = 0x27;

/// Which key is which button, which in 紅魔郷 is **not configurable at all**.
///
/// `Controller::GetInput` names every key in its own code — `VK_UP`, `'Z'`, `'X'`, `VK_SHIFT`, `VK_ESCAPE`
/// and the numpad beside them on the `GetKeyboardState` branch, and the `DIK_` equivalents on the
/// DirectInput one. `GameConfiguration`'s `controllerMapping` is the **pad's**, and the options screen that
/// writes it offers nothing about the keyboard.
///
/// So this is the game's own map and not a configuration of it, and the six here are the six a run is played
/// with. What the arrangement buys is that an e2e test pressing `Z` at one of orb's questions and this game
/// reading its shot key are the same key by construction — see [`LEFT`] for the two `orb_sim::keys` does not
/// name.
const MAP: [(u8, u16); 6] = [
    (keys::Z, button::SHOOT),
    (keys::X, button::BOMB),
    (keys::UP, button::UP),
    (keys::DOWN, button::DOWN),
    (LEFT, button::LEFT),
    (RIGHT, button::RIGHT),
];

/// The lives, bombs and power a **run** starts with, which is 紅魔郷's own: the two and the three are
/// `g_Supervisor.defaultConfig`'s, and the power a run begins with nothing of.
///
/// A run and not a stage. `GameManager::AddedCallback` writes `currentPower = 0` inside the branch it takes
/// only when it is *not* reinitialising, and writes `livesRemaining` and `bombsRemaining` nowhere at all —
/// what puts those two in place is the front end. So a stage transition carries all three, which is what
/// [`stage_numbers_in_place`](Fake::stage_numbers_in_place) is split on.
pub const FRESH: (i8, i8, u16) = (2, 3, 0);

/// And where the power a run collects stops, which is 紅魔郷's own **128**: `ItemManager`'s own collection
/// raises it one at a time and clamps there.
const FULL_POWER: u16 = 128;

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

/// And when the fight the stage *ends* with arrives, which is where the second of the two songs a stage's
/// data names begins to play.
///
/// This game's own number, past the midboss and its card so that the two fights are two. What it is for is
/// the one thing that tells them apart: **the music**, since an STD names the stage's song and the boss's
/// and the game plays the second for the boss it ends with — what the timeline is doing says nothing,
/// stage 3 parking on the same wait for its *midboss*.
pub const STAGE_BOSS_ARRIVES: u32 = 900;

/// And that fight fought out, where an e2e test asked for it — see [`Fake::fights_its_boss_out`]. A card
/// first, and then the attack after it.
///
/// This game's own numbers, each far enough from the last that a chapter is due at it: the shortest a
/// chapter may be is `chapter.rs`'s `MIN_CHAPTER_FRAMES`, a second.
pub const STAGE_BOSS_CARD_STARTS: u32 = 1100;
pub const STAGE_BOSS_ATTACK_CHANGES: u32 = 1300;

/// And the frame that attack is *named* on, which is not the frame it began on.
///
/// **Two frames behind, which 紅魔郷 really does**: the boss's own timer resets first and the spellcard is
/// declared after, and where those fall on different updates the chapter is already there and called a
/// nonspell. Patchouli's first card and Flandre's last, in stage 7, are the two it was measured on — see
/// `chapter::Chapters::due`, whose comment carries that measurement and whose answer is that the chapter
/// takes the name rather than a second chapter starting for it.
pub const STAGE_BOSS_NAMES_ITS_CARD: u32 = STAGE_BOSS_ATTACK_CHANGES + 2;

/// The two spell card records those attacks are, and what each is called.
///
/// Apart from [`CARD`] so that an e2e test reading a count back says which card it counted, the fight the
/// stage ends with and the midboss's being two fights.
pub const STAGE_BOSS_CARD: i32 = 4;
pub const STAGE_BOSS_CARD_NAME: &str = "BOSS CARD";
pub const STAGE_BOSS_LATE_CARD: i32 = 5;
pub const STAGE_BOSS_LATE_CARD_NAME: &str = "BOSS CARD NAMED LATE";

/// What rank a run is started at, which is 紅魔郷's own **16**.
///
/// `g_DifficultyInfo` at `src/GameManager.cpp` holds `{rank, minRank, maxRank}` per difficulty and every one
/// of the five has 16 as its rank — Easy's bounds are `{12, 20}` and Extra's `{14, 18}` where the other three
/// are `{10, 32}`, so the difficulty decides how far rank can move and not where it starts. One number
/// therefore covers whichever run an e2e test declares.
///
/// Written in the branch `GameManager::AddedCallback` takes only when it is *not* reinitialising — after an
/// earlier `rank = 8` in the same branch, which nothing between the two reads — so a stage transition carries
/// the rank the run had reached. `minRank` and `maxRank` are written beside it and are not laid out: nothing
/// above the game reads either.
pub const RANK_AT_A_RUNS_START: i32 = 16;

/// Which of the game's 64 spell card records its boss's card is. Any of them; what an e2e test reads
/// back is the count of attempts against this one.
pub const CARD: i32 = 3;

/// What that card is called, which the game copies into the record when the card starts and its ranking
/// screen draws against the row from then on.
pub const CARD_NAME: &str = "MIDBOSS CARD";

/// And one no card of this game's is, for a row an e2e test reads to say that nothing named it: every
/// record carries a name from the moment a run fills the block, and the count is what decides whether the
/// screen draws it — see `Image::fills_the_card_records`.
pub const UNNAMED_CARD: i32 = 0;

/// How many stages a run goes through, which is 紅魔郷's own six.
pub const STAGES: i32 = 6;

/// How long the title menu sits with nothing pressed at it before it falls into its attract demo, in
/// frames.
///
/// This game's own number, and far past the frames the title's own opening animation ignores a press for —
/// `MENU_TITLE_GRACE_FRAMES` — so the two moments a press can be spent on nothing are two moments and not
/// one. What an e2e test is about is that each of them costs a press, not how patient 紅魔郷 is.
pub const DEMO_AFTER: i32 = 90;

/// How long the game lays its panel over the start of a stage, which is 紅魔郷's own: the vm's script
/// sets all five of `GuiFlags`' two-bit fields to 2 itself over these frames — 0x41a2b6 — and stops
/// where the script reaches `ExitHide`. Over them, what orb writes into the lowest pair decides
/// nothing.
pub const PANEL_FRAMES: u32 = 250;

/// How long the ending stands before the result screen follows it, where no e2e test has laid an ending
/// out.
///
/// This game's own. An ending is a script and a staff roll — see [`Fake::lays_out_an_ending`] — and this
/// is the scene with nothing in it, which is what every e2e test that only has to pass a cleared run
/// through the ending wants.
const ENDING_FRAMES: i32 = 10;

/// What an ending is, as an e2e test lays one out.
///
/// Two scripts and two tracks: the ending's own, which runs out and hands over to `@Fdata/staff00.end`,
/// and the roll's, which plays on afterwards. Both are the game's — see [`Fake::lays_out_an_ending`] for
/// where the frames come from — and the tracks are this game's own numbers, since what an e2e test reads
/// of them is that the boundary changed both.
#[derive(Clone, Copy)]
struct Ending {
    /// The frames of waits in the ending's own script, and in the roll's after it.
    waits: i32,
    roll_waits: i32,
    /// Whether orb can find the script at all: an ending whose job is not in the chain is one
    /// `chain_argument` answers nothing for, which is what an ending already torn down looks like.
    found: bool,
}

/// The tracks this game plays, as the numbers orb tells one track from another by.
///
/// Two for an ending: `@mbgm/th06_16.mid` is the one an ending plays, and `staff00.end` starts
/// `bgm/th06_17` for the roll — what an e2e test reads off those two is that the script handing over and
/// the track changing happen on the same update. And two for a stage, which is what an STD names: the
/// stage's own song, which plays through the midstage and the midboss alike, and the boss's, which is the
/// second one and begins with the fight the stage ends with.
///
/// This game's own numbers, and their lengths are what a position inside one is measured against.
const ENDING_TRACK: Track = Track {
    length: 4_200_000,
    loop_start: 44,
    loop_end: 4_100_000,
};
const ROLL_TRACK: Track = Track {
    length: 2_600_000,
    loop_start: 44,
    loop_end: 2_500_000,
};
const STAGE_TRACK: Track = Track {
    length: 8_000_000,
    loop_start: 44,
    loop_end: 7_900_000,
};
/// And one more for an e2e test about the stream *itself* rather than about which chapters rewind: the same
/// shape and small enough to hold the sound of in memory, since what is read of it is arithmetic over its
/// own numbers. See [`streams_its_song`](Fake::streams_its_song).
const STREAMED_TRACK: Track = Track {
    length: 400_000,
    loop_start: 44,
    loop_end: 380_000,
};
/// How big the buffer that track is streamed into is, and how much the streaming thread writes each time it
/// is woken. This game's own numbers: a buffer holds a fraction of a second of sound and is topped up in
/// chunks, which is the shape the arithmetic is over.
pub const STREAM_BUFFER: usize = 32_768;
pub const STREAM_NOTIFY: u32 = 8_192;

/// The sound in that track's file: a byte per offset, and no two offsets alike.
///
/// A wave of zeros would say nothing. What an e2e test reads off the buffer has to name *where in the
/// file* those bytes were read from, or a restore that put back a buffer full of silence would pass the
/// same assertion as one that put the music back — so the byte at each offset is that offset's own,
/// Knuth's multiplicative hash of it, whose period is longer than any file laid out here.
fn sound_of(length: u32) -> Vec<u8> {
    (0..length)
        .map(|at| (at.wrapping_mul(0x9e37_79b9) >> 24) as u8)
        .collect()
}

/// What the stream a track is playing looks like from outside the game's memory: the bytes in the
/// buffer, where the play cursor is in them, where the file handle is, and whether it is playing.
///
/// **Which is exactly what no snapshot of that memory holds** — the buffer is DirectSound's, the cursor
/// is the mixer's and the position is winmm's — and so exactly what a chapter's restore has to put back
/// by calling them. One value rather than four reads, so an e2e test asks whether the *stream* came back.
#[derive(Clone, PartialEq, Eq)]
pub struct Stream {
    pub buffered: Vec<u8>,
    pub play_cursor: u32,
    pub position: i32,
    pub playing: bool,
}

impl std::fmt::Debug for Stream {
    /// The buffer as how much of it and what it adds up to, because a failure naming 32,768 bytes is a
    /// failure nobody reads: what an e2e test is asking is whether these are the same bytes, and a sum
    /// that differs says they are not.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Stream {{ {} byte(s) adding to {}, cursor {}, file at {}, {} }}",
            self.buffered.len(),
            self.buffered
                .iter()
                .map(|byte| u64::from(*byte))
                .sum::<u64>(),
            self.play_cursor,
            self.position,
            if self.playing { "playing" } else { "stopped" },
        )
    }
}
const BOSS_TRACK: Track = Track {
    length: 6_400_000,
    loop_start: 44,
    loop_end: 6_300_000,
};

/// How many items the title menu's cursor walks, which is the eight the game bounds it to.
const TITLE_ITEMS: i32 = 8;

/// What those eight are, in the order the cursor counts them — `image::item` names the three that
/// anything above has a question about.
///
/// Drawn, because **what a menu offers is the one thing about the front end no log can see**: `Extra
/// Start` is on it where the score file's `clrd` chunk is there and off it where the read that fills
/// that chunk found nothing, and reading that off the screen is the only way to ask.
const TITLE_MENU: [&str; TITLE_ITEMS as usize] = [
    "Game Start",
    "Extra Start",
    "Practice Start",
    "Replay",
    "Score",
    "Music Room",
    "Option",
    "Quit",
];

/// Where this game's title menu puts them. Its own layout, the way the ranking's rows are its own:
/// what an e2e test reads off it is which of the items are there at all.
const TITLE_TOP: f32 = 152.0;
const TITLE_LINE: f32 = 20.0;
const TITLE_LEFT: f32 = 224.0;

/// Where this game's ranking screen puts its rows, and the colour it writes them in. Its own layout:
/// what an e2e test reads off it is which text is at which of these, and the numbers themselves are
/// nothing but somewhere to put them.
const RANKING_TOP: f32 = 96.0;
const RANKING_LINE: f32 = 24.0;
const RANKING_LEFT: f32 = 64.0;
const RANKING_NAME: f32 = 160.0;
const RANKING_ATTEMPTS: f32 = 400.0;
const INK: u32 = 0xffff_ffff;

/// What the screen draws where a card's name would go for as long as nobody has tried it: 「？？？？？」
/// at 0x46bcdc, five full-width question marks, pushed at 0x42e270.
pub const NOT_TRIED: &str = "？？？？？";

/// How long the high-score name entry a finished run arrives on stands before it is answered.
///
/// Not at once, because the real one is a screen somebody types eight characters into, with the stats screen
/// after it.
///
/// **Past the 240 updates `runtime`'s `COMMIT_FRAME_LIMIT` allows a ranking to be built in**, which is the
/// property that matters rather than the number: the real screen never advances on its own at all, so a
/// game whose screen walked itself out inside that allowance would answer for orb what a player answers —
/// and a ranking asked for where this screen is up would come out looking like one that came up.
const NAME_ENTRY_FRAMES: i32 = 300;

/// And how long the question about saving a replay stands after it, where nothing writes past it.
///
/// Past [`REPLAY_QUESTION_DRAWN_AT`], so that a screen left to itself really would draw the question orb
/// writes past — which is what the half of that measurement taken on a launch orb asks nothing of needs.
const REPLAY_QUESTION_FRAMES: i32 = 90;

/// And how many frames into that state the question begins to be drawn, which is 紅魔郷's own **60**: orb
/// writes the state past the question before the frame timer reaches that, so no part of that screen is
/// ever drawn. See `Th06::skip_replay_prompt`.
const REPLAY_QUESTION_DRAWN_AT: i32 = 60;

/// How far the player moves in a frame, which is this game's own.
pub const SPEED: f32 = 4.0;

/// And the box they are held inside, which is 紅魔郷's own `(8, 16)` and `(368, 416)` —
/// `playerMovementAreaTopLeftPos` and `playerMovementAreaSize`, written in the branch
/// `GameManager::AddedCallback` takes only when it is not reinitialising.
///
/// **Not the arcade region**, whose `(32, 16)` and `(384, 448)` sit beside them in the same branch and are a
/// different rectangle: `Player::HandlePlayerInputs` clamps `positionCenter` to this one and
/// `Player::AddedCallback` measures a stage's first position from *that* one. Which is why a screen shake can
/// move where a stage starts the player and cannot move where they are allowed to go.
pub const PLAYER_AREA_TOP_LEFT: (f32, f32) = (8.0, 16.0);
pub const PLAYER_AREA_SIZE: (f32, f32) = (368.0, 416.0);

/// How long a bomb's screen shake runs here, which is **one of the six 紅魔郷 has**.
///
/// `ScreenEffect::RegisterChain(SCREEN_EFFECT_SHAKE, …)` is called from six places in `src/BombData.cpp`
/// with 16, 60, 80, 120, 60 and 200 frames — one bomb registers two of them at different moments of itself —
/// so there is no such thing as *the* length of a bomb's shake. This is 80 because 80 is one of them and an
/// e2e test has to wait out whichever it gets; which bomb went off is the player's own code and none of it is
/// here.
pub const SHAKE_FRAMES: i32 = 80;

/// How far it moves the arcade region, in pixels. This game's own: what an e2e test reads of it is that
/// the region is not where a stage left it, not how violent a bomb looks.
const SHAKE_PIXELS: u16 = 8;

/// What `Player::AddedCallback` leaves on the invulnerability count, what the spawning state ends at, and
/// what the update that ends it puts there instead. All three 紅魔郷's own, `src/Player.cpp`.
///
/// **The 120 is not how long anything lasts.** `Player::OnUpdate`'s spawning branch tests
/// `30 <= invulnerabilityTimer.AsFrames()`, so a stage's own 120 satisfies it on the very first update and
/// the state is flipped there — with `SetCurrent(240)` under it, which is the count a stage is really played
/// with. What the 120 does is make that flip happen immediately rather than 30 updates later, which is what
/// a *death* gets: the timer is zeroed on the way into spawning there and ticks up to 30 first.
///
/// The 240 is `PLAYER_INVULNERABLE_FRAMES` read from the other end — orb writes the same number under the
/// same state every update of a `--clear` run, and the measurement for it is beside that constant.
const SPAWNS_WITH: i32 = 120;
const SPAWNING_ENDS_AT: i32 = 30;
pub const INVULNERABLE_AFTER_SPAWNING: i32 = 240;

/// How far above the arcade region's foot a stage puts the player, which is 紅魔郷's own:
/// `Player::AddedCallback` measures `arcadeRegionSize.x / 2` across and `arcadeRegionSize.y - 64` down.
///
/// Measured from the *arcade region* and not from the box the player is held inside, which is the whole
/// reason a screen shake can reach the stage after the one that started it: a shake writes that region
/// from the generator every frame, and nothing on the way into a stage puts it back.
pub const PLAYER_STARTS_ABOVE: f32 = 64.0;

/// The run a launch is started for: Normal, Reimu A, from stage one.
///
/// What an e2e test that is not about a run declares, a launch having to be started for one: the game
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
    /// The score files this game keeps, by the name each open landed in — see [`ScoreFile`] for the
    /// chunks one holds.
    ///
    /// A map rather than files on disk: what an e2e test reads is which name an open landed in and the
    /// number behind it, and both of those cross `CreateFileA` — the path and the access. Outside the
    /// game's memory, so a chapter restored does not rewind them, which is true of real files too.
    ///
    /// A name with no entry is a file that is not there, and its open fails: which is what orb's own file
    /// is before a ranking has ever written it.
    files: RefCell<HashMap<String, ScoreFile>>,
    /// Every open of the score file, in the order they happened — see [`Open`].
    opens: RefCell<Vec<Open>>,
    /// What the pad is doing, as an e2e test is pushing it.
    ///
    /// Beside the memory rather than in it, and that is what a device is: the game's own read asks the
    /// controller every frame and the answer is never anywhere a snapshot could rewind.
    pushed: Cell<Pushed>,
    /// Set by an e2e test for the frame the player is hit on.
    ///
    /// The one thing about a run that a laid-out game cannot do for itself: there are no bullets
    /// here, so being hit is an e2e test saying so where in a real run it is an e2e test dodging badly.
    hit: Cell<bool>,
    /// And a bullet sitting on the player from here on, which is a hit every frame rather than one.
    ///
    /// Apart from `hit` because what it is for is the other question: `hit` is an e2e test saying the
    /// player died, and this is an e2e test saying nothing is stopping them dying — so what happens is
    /// the hit test's answer and not the e2e test's. See [`puts_a_bullet_on_the_player`](Fake::puts_a_bullet_on_the_player).
    bullet: Cell<bool>,
    /// How long each of this game's stages runs before the next begins. `None` is a stage that never
    /// ends — see [`stages_last`](Fake::stages_last).
    stage_frames: Cell<Option<u32>>,
    /// The ending a cleared run reaches, or `None` for the scene with nothing in it — see
    /// [`lays_out_an_ending`](Fake::lays_out_an_ending).
    ending: Cell<Option<Ending>>,
    /// Whether the front end draws its own items — see
    /// [`draws_its_title_menu`](Fake::draws_its_title_menu).
    draws_the_menu: Cell<bool>,
    /// How many frames of a bomb's screen shake are left to run — see [`bombs`](Fake::bombs).
    shake_frames: Cell<i32>,
    /// Whether its stages stream the two songs their data names — see
    /// [`plays_its_songs`](Fake::plays_its_songs).
    plays_songs: Cell<bool>,
    /// Whether the fight the stage ends with goes on past its arrival — see
    /// [`fights_its_boss_out`](Fake::fights_its_boss_out).
    fights_boss_out: Cell<bool>,
    /// Whether a stage asked for is one that never arrives — see
    /// [`never_builds_the_stage_it_is_asked_for`](Fake::never_builds_the_stage_it_is_asked_for).
    never_builds: Cell<bool>,
    /// And whether its front end sits on a ranking it was asked for — see
    /// [`never_builds_the_ranking_it_is_asked_for`](Fake::never_builds_the_ranking_it_is_asked_for).
    never_builds_the_ranking: Cell<bool>,
    /// Whether this game's controller has been lost — see
    /// [`its_controller_poll_fails`](Fake::its_controller_poll_fails) — and how many times orb has had it
    /// acquired again.
    controller_poll_fails: Cell<bool>,
    controller_acquires: Cell<u32>,
    /// What comes up where a stage is asked for, or `None` for the stage that was asked for — see
    /// [`comes_up_as`](Fake::comes_up_as).
    comes_up_as: Cell<Option<ComesUpAs>>,
    /// Every present the game's device was asked for, in the order it was asked — see [`Presented`].
    presents: RefCell<Vec<Presented>>,
    /// Whether this device's driver will not stretch on a present — see
    /// [`refuses_to_stretch_on_a_present`](Fake::refuses_to_stretch_on_a_present).
    refuses_to_stretch: Cell<bool>,
    /// The sound a track is streamed through, where an e2e test asked for one — see
    /// [`streams_its_song`](Fake::streams_its_song).
    ///
    /// Held here because orb reaches it by dereferencing the pointer the game's memory holds: the object
    /// has to outlive the last frame that reads the music, which is the game closing.
    sound: RefCell<Option<Box<orb_sim::Sound>>>,
    /// Set by an e2e test for the run to be given up at the game's own pause — see
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
    /// The window this game asked the host for, as the host answered — see
    /// [`creates_its_window`](Fake::creates_its_window). Null until it has asked.
    window: Cell<orb_api::Hwnd>,
    /// How many times orb has taken this game's music down through its own `StopBGM`, and the paths it
    /// has started a track by through its own `PlayAudio` — see
    /// [`music_stops`](Fake::music_stops) and [`music_started`](Fake::music_started).
    ///
    /// Beside the memory rather than in it: what the game was *called* is not a fact about the run, and
    /// the restore these happen either side of would rewind the record of them.
    music_stops: Cell<u32>,
    music_starts: RefCell<Vec<String>>,
    /// How many times orb has asked this game's keyboard device to be acquired again, which is what an
    /// e2e test reads instead of a log line — see [`keyboard_acquires`](Fake::keyboard_acquires).
    ///
    /// Beside the memory rather than in it: what a device was asked is not a fact about the run, and no
    /// chapter restored underneath it takes an acquire back.
    keyboard_acquires: Cell<u32>,
    /// And whether that device refuses, which is what one whose window is not in front does — see
    /// [`refuses_the_keyboard_acquire`](Fake::refuses_the_keyboard_acquire).
    refuses_the_acquire: Cell<bool>,
    /// What this game's chain walk answers, which an e2e test changes to say the game is leaving.
    answers: Cell<i32>,
    /// Every replay this game has written, as the file name and the name inside it — see
    /// [`saves_its_replay`](Fake::saves_its_replay).
    ///
    /// Beside the memory rather than in it, for the same reason the score files are: what an e2e test reads
    /// back is whether a write happened and under which name, and both of those cross the call.
    replays_written: RefCell<Vec<(String, String)>>,
    /// Held for the process's life, so that every read orb makes lands in this game's memory. Last,
    /// and never dropped: see [`Fake::attach`].
    _installed: orb_api::Installed,
}

thread_local! {
    /// The game running on this thread, for the hooks orb calls back into.
    ///
    /// Those are plain `extern` functions with nothing but the ABI's arguments — the same reason the sound
    /// an e2e test installed is a thread's rather than a field of the simulated host — so where the real game
    /// would reach its own globals, this reaches the game.
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
    /// read from it, and the runs left unfinished are kept under it. `settings` is where an e2e test says
    /// what this launch was started with.
    ///
    /// Boxed and owned by whoever asked for it, so that an e2e test file can hold more than one game over
    /// its lifetime — **and the one before has to be dropped first**. What a launch's device and its record
    /// are is the simulated Windows', which goes with the game; what does not is orb's own state, the
    /// runtime and the pacing and the log's handle being the process's. `Drop` is the handover: it calls
    /// `orb_core::runtime::detached`, which takes the runtime down and closes the log so that the next game opens its
    /// own.
    ///
    /// Boxed rather than returned by value because the hooks find it through a pointer, and a value
    /// moved out of this function would leave that pointer behind. Its `Drop` is the game closing, in
    /// the one order that works: the runtime first, so orb's overlay is released through a device that
    /// is still there, then the device, then the simulated Windows it was all read through.
    pub fn attach(name: &str, run: RunStart, settings: impl FnOnce(&mut Config)) -> Box<Self> {
        Self::attach_to_display(Display::ordinary(), name, run, settings)
    }

    /// And with that directory already holding what an earlier launch left in it: `left` is a file name
    /// and its contents, written before orb is attached.
    ///
    /// Before, because that is the only moment it can be: orb reads what it finds there while it is being
    /// attached — a tuning pass reads `tuning.txt` inside `Chapters::new` — and a file written after that
    /// is one nothing will ever look at. Which is also why the directory is emptied at every attach: a
    /// launch finds what an e2e test says it finds and nothing another e2e test left behind.
    pub fn attach_finding(
        name: &str,
        left: &[(&str, &str)],
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(Display::ordinary(), None, name, left, run, true, settings)
    }

    /// And on a display an e2e test says the whole of, for the ones the frame loop's pacing is about.
    pub fn attach_to_display(
        display: Display,
        name: &str,
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(display, None, name, &[], run, true, settings)
    }

    /// And on a monitor an e2e test says the whole of too, for the ones about the window orb makes.
    ///
    /// The panel goes in before orb is attached, because attaching is when orb says the process reads
    /// real pixels — a monitor written down after that would be one nothing had asked the truth of.
    pub fn attach_to_a_panel(
        panel: Panel,
        name: &str,
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(
            Display::ordinary(),
            Some(panel),
            name,
            &[],
            run,
            true,
            settings,
        )
    }

    /// And laid out with **no device**, which is what a process is before its Direct3D setup has run: orb
    /// attaches there in a real launch and finds the device afterwards, through its hook over
    /// `GameWindow::InitD3dDevice`. A game that had one already is a game that hook is never reached in — see
    /// [`finds_its_device`](Fake::finds_its_device), which is the other half of this.
    pub fn attach_before_its_device(
        name: &str,
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(Display::ordinary(), None, name, &[], run, false, settings)
    }

    /// And the same with a monitor declared, which is the launch a letterbox exists in: the window is laid
    /// out against a panel, and the device whose `Present` that letterbox is presented through turns up
    /// afterwards, through the hook orb redirects the slot inside.
    pub fn attach_to_a_panel_before_its_device(
        panel: Panel,
        name: &str,
        run: RunStart,
        settings: impl FnOnce(&mut Config),
    ) -> Box<Self> {
        Self::attach_declaring(
            Display::ordinary(),
            Some(panel),
            name,
            &[],
            run,
            false,
            settings,
        )
    }

    fn attach_declaring(
        display: Display,
        panel: Option<Panel>,
        name: &str,
        left: &[(&str, &str)],
        run: RunStart,
        device: bool,
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
        // The simulated Windows first, because everything below reads through it: `orb.yaml` and the runs an
        // earlier session left are files of orb's own, and files of orb's own go through the file seam into
        // `orb_sim::Files`. Which is why `left` is put in *here* rather than on a disk — see
        // [`scratch`](super::scratch).
        let image = Image::laid_out_seeded(display.seed);
        let installed = image.enter();
        // The directory the game is installed in, which is there because the game is: orb writes a tuning
        // pass's two files straight into it, and a write wants its directory here as on a real host.
        image.sim().files().make(&dir);
        for (file, contents) in left {
            image.sim().files().put(dir.join(file), contents);
        }
        let mut config =
            Config::load_beside(&dir.join(EXE)).expect("a directory with no orb.yaml in it");
        // The memory hooks patch an import table there is none of, and no e2e test can turn them on: a game
        // laid out by hand has no PE headers for `pe::import_slot` to walk, and one that grew a synthesized
        // set would be a game standing in for a loader. What stands behind them instead is `hook.rs`'s own
        // `an_imports_slot_is_swapped_and_what_was_there_comes_back`, over headers written out by hand.
        config.track_memory = false;
        settings(&mut config);

        // The font beside the game's exe, which is what orb builds its overlay from — 紅魔郷 ships one
        // and this says so. In front of the launch and not after it, because the launch's own device
        // bakes through the same seam and a face made before this is one nothing declared.
        image.sim().text().install_font(dir.join("font.ttf"));
        let launch = Launch::new(image.sim(), dir, display.seed, config.own_frame_loop);
        // The window the game has made, and the device it shows through where an e2e test says the setup has
        // already run. A launch that says otherwise gets the window and no device, which is what a process is
        // between `GameWindow::Create` and `InitD3dDevice`.
        image.shows_through(
            if device {
                launch.device()
            } else {
                orb_api::Device::NULL
            },
            WINDOW,
        );
        // Where the game is installed, which is where orb writes its log: beside the exe, because that
        // is where `orb.yaml` and the launcher are.
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
        // And the monitor the window goes on, for an e2e test about the window: in front of orb being
        // attached for the same reason the rate is, since attaching is where orb says this process reads
        // sizes as real pixels and a panel written down afterwards would never have been asked the
        // scaled question at all. An e2e test that declares none has no monitor, which is the launch orb
        // leaves the window as the game made it.
        if let Some(panel) = &panel {
            sim.windows().set_monitor(panel.monitor, panel.frame);
            if panel.refuses_dpi_awareness {
                sim.windows().refuse_dpi_awareness();
            }
        }
        // And the game's own calls orb makes that an e2e test reaches: a shake still running at a stage
        // move is taken down through `Chain::Cut`, and a track whose chapter has been left behind is
        // stopped and started again through `StopBGM` and `PlayAudio`.
        image.hands_over_chain_cut(chain_cut as *const () as usize);
        image.hands_over_the_music_calls(
            stop_bgm as *const () as usize,
            play_audio as *const () as usize,
        );
        // A controller, mapped the way this game's configuration maps one. The numbers are its own —
        // a real one's come out of the file the game's own options screen writes — and what an e2e test
        // needs of them is that orb reads a pad's buttons through this mapping and not around it.
        image.controller(
            poll as *const () as usize,
            acquire as *const () as usize,
            read_state as *const () as usize,
        );
        image.maps_the_pad(MAPPING);
        // And the keyboard device the game takes exclusively, which is what its own read goes through until
        // orb lets it go: a launch that has not asked for `--sent-keys` keeps it for its whole life, which
        // is every other e2e test here and is why the keys they press are pressed rather than sent.
        image.keyboard_device(
            keyboard_acquire as *const () as usize,
            keyboard_unacquire as *const () as usize,
            keyboard_release as *const () as usize,
        );
        // What its front end lights `Extra Start` from: a game that has been cleared, which is what an
        // installation somebody has played is. In memory as well as in the file, because the read that
        // fills it is one the front end makes for itself — a menu built before any read would otherwise
        // be lit from a destination nothing had written.
        image.parses_the_unlocks(&cleared());
        // And the file both are in, under the name the game asks for. Only that one: orb's own is a file
        // that is not there until a ranking screen has written it, which is what makes its first open fail
        // the way a first launch's does.
        //
        // With no record of any spell card in it, which is a `catk` chunk holding none: that parse has no
        // clear of its own — 0x42b466 against `clrd`'s memset at 0x42b502 — so what it leaves standing is
        // `GameManager::AddedCallback`'s own fill, and a card this file does not name is one the ranking
        // would draw the fill's bytes for. See `Image::fills_the_card_records`.
        let files = RefCell::new(HashMap::from([(
            SCORE_FILE.to_string_lossy().into_owned(),
            ScoreFile {
                captures: Vec::new(),
                unlocks: cleared(),
            },
        )]));
        // Where the game starts: its front end, on the title menu, on the item that starts a run — and not
        // built yet, which is the supervisor's own first frame. What that buys is the front end's own read
        // of the score file: `MainMenu::AddedCallback` is where it happens and being built is what calls it,
        // so a game whose menu was there from nothing would never make the one open whose answer is the
        // game's own file whatever the mode.
        // `Supervisor::RegisterChain`, which is the chain's first job and the one every other goes in above:
        // a game with none of it is a game whose walk runs nothing at all.
        image.registers_the_supervisor();
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
            ending: Cell::new(None),
            draws_the_menu: Cell::new(false),
            shake_frames: Cell::new(0),
            plays_songs: Cell::new(false),
            fights_boss_out: Cell::new(false),
            never_builds: Cell::new(false),
            never_builds_the_ranking: Cell::new(false),
            controller_poll_fails: Cell::new(false),
            controller_acquires: Cell::new(0),
            comes_up_as: Cell::new(None),
            presents: RefCell::new(Vec::new()),
            refuses_to_stretch: Cell::new(false),
            sound: RefCell::new(None),
            given_up: Cell::new(false),
            demos: Cell::new(false),
            replay_question_drawn: Cell::new(false),
            window: Cell::new(orb_api::Hwnd::NULL),
            music_stops: Cell::new(0),
            music_starts: RefCell::new(Vec::new()),
            keyboard_acquires: Cell::new(0),
            refuses_the_acquire: Cell::new(false),
            answers: Cell::new(CHAIN_CARRIED_ON),
            replays_written: RefCell::new(Vec::new()),
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
                    render: own_render,
                    play_sounds,
                    present,
                    create_window,
                    create_file,
                    stop_recording,
                    create_game_window: game_window_create,
                    joystick_position,
                    save_replay,
                    init_d3d_device,
                },
            )
        };
        // And the game's startup check, which is the one read of a joystick it does for itself — after the
        // attach, orb's replacement of that import entry being what answers it.
        fake.checks_for_a_joystick();
        fake
    }

    /// And the same with orb's own account of the pacing being written where an e2e test can read it
    /// back, which is every e2e test about the frame loop — see `pacing.rs`.
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

    /// What this game's chain walk answers from here on: [`CHAIN_CARRIED_ON`] until an e2e test says
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
    /// An e2e test saying so, the way it says the player was hit: a replay is started from the game's
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

    /// And the same with the replay's own record under it: a manager with a replay loaded, and one
    /// record of inputs per stage of this game's six.
    ///
    /// Which is what makes the run play *itself*: the manager's own job overwrites the word the input
    /// read handed back, so the player moves on the buttons that were recorded rather than on the ones an
    /// e2e test is pressing — and the stepping keys an e2e test does press move between the stages the
    /// replay covers instead.
    ///
    /// Apart from [`watches_a_replay`](Fake::watches_a_replay) because a record is what a teardown can
    /// write over and a stage move needs: an e2e test about neither wants a replay that plays nothing.
    pub fn watches_a_replay_of_its_stages(&self) {
        let stages: Vec<i32> = (0..STAGES).collect();
        self.image.loads_a_replay(&stages);
        for stage in &stages {
            self.image
                .records_the_inputs(*stage, RECORDED_SEED, &recorded());
        }
        self.watches_a_replay();
    }

    /// Says the player was hit, for the next frame of the stage.
    pub fn hit(&self) {
        self.hit.set(true);
    }

    /// The game writing its replay out, which is what `ResultScreen`'s save screen does once somebody has
    /// named one: `ReplayManager::SaveReplay(replayPath, this->replayName)` at `src/ResultScreen.cpp`.
    ///
    /// An e2e test saying so, the way it says the player was hit. The screen itself is not here — orb writes
    /// that screen's state *past* the question rather than answering it, which is
    /// `a_clear_on_demand.rs`'s subject — so the one thing an e2e test needs of it is the call, and
    /// this is the call.
    pub fn saves_its_replay(&self) {
        orb_core::runtime::save_replay(REPLAY_FILE.as_ptr().cast(), REPLAY_NAME.as_ptr().cast());
    }

    /// Every replay this game has written, as the file name and the name inside it.
    pub fn replays_written(&self) -> Vec<(String, String)> {
        self.replays_written.borrow().clone()
    }

    /// The game's Direct3D setup finished: the device in its memory, and `GameWindow::InitD3dDevice` called
    /// over it.
    ///
    /// Through orb's hook over that function, which is what redirects the device's `Present` before anything
    /// is presented through it — and the device goes in *first*, because that is the order the game has:
    /// `InitD3dDevice` only ever sets render states on a device that already exists, called from
    /// `GameWindow::Create`'s tail and again after every `Reset`.
    ///
    /// Only for a launch that said it had no device to begin with — see
    /// [`attach_before_its_device`](Fake::attach_before_its_device).
    pub fn finds_its_device(&self) {
        let device = self.launch.device();
        // The device object and the vtable under it, laid out here because orb redirects the `Present`
        // slot inside that hook: `orb_core::window::hook_device` reads the object for its vtable and swaps
        // the slot through `orb_api::mem`, so a device that was only a handle is a read of an address
        // nothing has mapped.
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

    /// Every present the game's device has been asked for since the last
    /// [`forget_presents`](Fake::forget_presents), in the order it was asked.
    pub fn presented(&self) -> Vec<Presented> {
        self.presents.borrow().clone()
    }

    pub fn forget_presents(&self) {
        self.presents.borrow_mut().clear();
    }

    /// Has this device's driver refuse to stretch on a present, which some do: the call that asks for a
    /// destination rectangle answers a failure and the one that asks for none is answered.
    ///
    /// An e2e test saying so, the way it says the host will not turn display scaling off. What it is for is
    /// what orb does about it — the game's own call again, which leaves the run playable and stretched —
    /// since a run that stopped being presented at all would be a black window.
    pub fn refuses_to_stretch_on_a_present(&self) {
        self.refuses_to_stretch.set(true);
    }

    /// How many times orb has taken this game's music down through its own `StopBGM`.
    pub fn music_stops(&self) -> u32 {
        self.music_stops.get()
    }

    /// And the paths it has started a track by through its own `PlayAudio`, in the order it did.
    ///
    /// The path and not a count, because which path is the question: it is read out of the memory the
    /// restore has just put back, so a wrong one is a chapter's music restarted from another stage's song.
    pub fn music_started(&self) -> Vec<String> {
        self.music_starts.borrow().clone()
    }

    /// How many times orb has asked this game's keyboard device to be acquired again.
    ///
    /// The device is a `DISCL_EXCLUSIVE | DISCL_FOREGROUND` one, so the system takes it away the moment
    /// the window goes behind and every read after that fails — which is what orb asks for it back
    /// before. A count rather than the log line, because what the claim is about is the *call*.
    pub fn keyboard_acquires(&self) -> u32 {
        self.keyboard_acquires.get()
    }

    /// Has that device refuse, which is what one whose window is not in front really does:
    /// `DIERR_OTHERAPPHASPRIO`.
    ///
    /// An e2e test saying so, the way it says which window is in front — the two are the same fact from
    /// two sides, and a device that worked that out for itself would be answering the question orb is
    /// being asked here.
    pub fn refuses_the_keyboard_acquire(&self, refuses: bool) {
        self.refuses_the_acquire.set(refuses);
    }

    /// Has the game's controller answer its poll with `DIERR_INPUTLOST`, which is what a DirectInput device
    /// does once it has been lost — the window went away, or another process took it.
    ///
    /// An e2e test saying so, the way it says the keyboard device refuses its acquire. What it is for is the
    /// one thing orb does about it: `Th06::controller_pad` acquires the device again and answers nothing for
    /// that frame, so a pad that came back is a pad the next frame reads.
    pub fn its_controller_poll_fails(&self, fails: bool) {
        self.controller_poll_fails.set(fails);
    }

    /// How many times the game's controller has been acquired again after a poll that failed.
    pub fn controller_acquires(&self) -> u32 {
        self.controller_acquires.get()
    }

    /// Has what comes up where a stage was asked for be something other than that stage.
    ///
    /// **Which is the one thing a resume cannot check for itself.** It points the run the game is about to
    /// build at a stage and holds that stage's numbers for the moment it is ready for them; if what arrives
    /// is another stage, or a run orb keeps nothing of, then writing those numbers over it would be a run
    /// put back with the wrong lives and the wrong seed and nothing about the file saying so. So orb
    /// compares, and an e2e test is what makes there be something to compare — see
    /// `resume::stage_begun`.
    ///
    /// From the frame this is said until it is said otherwise, which is per stage the game builds.
    pub fn comes_up_as(&self, what: Option<ComesUpAs>) {
        self.comes_up_as.set(what);
    }

    /// A bomb, which is here for the one thing about one that outlives the stage it went off in: the
    /// screen shake it starts.
    ///
    /// An e2e test saying so, the way it says the player was hit — the four bombs are the player's own code
    /// and none of it is here. What is here is `ScreenEffect::ShakeScreen` registered as a job of the
    /// chain's, the [`SHAKE_FRAMES`] it writes the arcade region from the generator over, and the bomb the
    /// run has one fewer of: `Player::OnUpdate` spends one where it starts one, and only where there is one
    /// to spend.
    ///
    /// And `Player::bombInUse`, which is the flag orb reads a bomb by. **A bomb here is its shake and
    /// nothing else**, so the flag stands for exactly the shake's own frames and the frame the shake takes
    /// itself down is the frame it goes out; a real bomb's four are the player's code, where the two lengths
    /// are the same effect's and not one number. What an e2e test asks of it is that a bomb is *not* a frame
    /// orb keeps away from — see `chapter::guarded`, whose comment says so and says the `sync:` line is
    /// where a bomb is read back.
    pub fn bombs(&self) {
        let left = self.image.playing_now().bombs;
        if left > 0 {
            self.image.set_bombs(left - 1);
        }
        self.shake_frames.set(SHAKE_FRAMES);
        self.image.bombing(true);
        self.image.shakes_the_screen();
    }

    /// A power item collected, which raises the run's power by one and stops at the game's own
    /// [`FULL_POWER`].
    ///
    /// An e2e test saying so, the way it says the player was hit: there are no items here, so the
    /// collection is an e2e test's word for it where in a real run it is an e2e test flying over one.
    pub fn collects_a_power_item(&self) {
        let power = self.image.playing_now().power;
        self.image.set_power((power + 1).min(FULL_POWER));
    }

    /// Puts a bullet where the player is and leaves it there, so that every update from here on runs its
    /// hit test against a live bullet.
    ///
    /// Which is what a stage really is, and the difference from [`hit`](Fake::hit) is the whole point of
    /// it: an e2e test saying "the player died" cannot show that something *stopped* them dying. The test
    /// runs where the game runs it — after `Player::OnUpdate`, which is chain priority 7 against the
    /// bullets' 11 — so an invulnerability written for this update has to still be there when it runs.
    pub fn puts_a_bullet_on_the_player(&self) {
        self.bullet.set(true);
    }

    /// The fight over: the boss off the screen and its card with it, which is what the enemy manager
    /// holds none of once one has been beaten.
    ///
    /// An e2e test saying so, the way it says the player was hit — there is nothing shooting here, so a
    /// boss's life comes down because an e2e test says the fight is won where in a real run it comes down
    /// because somebody shot it. What it is for is the stage *after* a fight: a fight underway outranks
    /// the midstage table, so a stage whose boss never goes down is one where nothing of that table can
    /// begin a chapter.
    ///
    /// Whatever the stage's own script has arranged for a later frame still happens — the fight this game
    /// has runs from [`BOSS_ARRIVES`] to [`ATTACK_CHANGES`] — so beaten inside those frames is a boss that
    /// comes back on the next of them.
    pub fn beats_its_boss(&self) {
        self.image.boss(None);
        self.image.card(None);
    }

    /// How long each of this game's stages runs before the next begins.
    ///
    /// Nothing by default, which is a stage that never ends: every e2e test about one stage wants that,
    /// and one about a run through six says how long each of them is. A run's last stage hands over to
    /// the ending rather than to a seventh.
    pub fn stages_last(&self, frames: u32) {
        self.stage_frames.set(Some(frames));
    }

    /// Has its stages stream the two songs their data names: the stage's own from the frame the stage is
    /// built, and the boss's from [`STAGE_BOSS_ARRIVES`].
    ///
    /// Nothing by default, which is what every e2e test that is not about the music wants and is why
    /// `pointdevice_run.rs` takes its first chapter at frame 248 — the whole of `MUSIC_WAIT_FRAMES`
    /// spent waiting for a track that is never coming. A stage with a song under it is also a stage whose
    /// chapters have sound to put back, which is work on every one of them.
    pub fn plays_its_songs(&self) {
        self.plays_songs.set(true);
        self.image.plays_a_track(STAGE_TRACK);
    }

    /// Has the fight the stage ends with go on past its arrival: a card at [`STAGE_BOSS_CARD_STARTS`], the
    /// attack after it at [`STAGE_BOSS_ATTACK_CHANGES`], and that attack's *name* two frames behind it.
    ///
    /// **Which fight those attacks belong to this does not say**, and that is what it is for: with the two
    /// songs a stage's data names laid out they are the stage's own boss's, the boss track having started at
    /// [`STAGE_BOSS_ARRIVES`], and without them they are still the midboss's. One script, and the music is
    /// the whole of the difference — which is the claim `chapter::Chapters::due` rests a boss's chapters on.
    ///
    /// Nothing by default, and that is not tidiness: these are three more chapters in every stage that runs
    /// past frame 1100, and an e2e test counting the chapters before a boundary of the table would count
    /// them.
    pub fn fights_its_boss_out(&self) {
        self.fights_boss_out.set(true);
    }

    /// Has a stage this game is asked for never arrive: the scene a run is played in, with nothing of the
    /// run built inside it and `GameManager::AddedCallback` never reached.
    ///
    /// Which is what a stage the game cannot load looks like from orb's side — the run was chosen, the
    /// game went somewhere else with it, and the moment orb writes a resumed run's numbers over a stage's
    /// never comes. What it is for is the bound orb puts on waiting for one; see
    /// `runtime::RESUME_START_FRAMES`, which is ten seconds of frames.
    pub fn never_builds_the_stage_it_is_asked_for(&self) {
        self.never_builds.set(true);
    }

    /// And have its front end sit on the ranking it is asked for: the item is written into
    /// `MainMenu.gameState`, and the screen behind it never comes up.
    ///
    /// Which is what a ranking orb asked for looks like when it does not come up, and the bound orb puts on
    /// waiting is `runtime::COMMIT_FRAME_LIMIT` — 240 updates inside the one frame it asks in. What that
    /// path has to do with a game that answers slowly rather than not at all is the same thing: undo the
    /// request, write nothing, and touch no screen.
    pub fn never_builds_the_ranking_it_is_asked_for(&self) {
        self.never_builds_the_ranking.set(true);
    }

    /// Has its stage stream a track through a sound of the host's, with the sound now audible beginning
    /// `heard_at` bytes into the file.
    ///
    /// Which is more of one than [`plays_its_songs`](Fake::plays_its_songs) lays out, and a different
    /// question: that one is about *which* chapters put their music back, and needs nothing but the numbers
    /// a track is told apart by. This is about the seek itself — the buffer the game plays and the file
    /// handle it reads — and orb reaches those by calling them rather than by reading them, so they are a
    /// real object and a real pair of functions. See [`orb_sim::Sound`].
    ///
    /// The countdown goes in with the position, because they are the pair a loop is taken on: the track
    /// loops where the countdown runs out, so what is left before it from here is the loop point less where
    /// the file is.
    pub fn streams_its_song(&self, heard_at: i32) {
        let sound = orb_sim::Sound::of(
            sound_of(STREAMED_TRACK.length),
            STREAM_BUFFER,
            STREAM_NOTIFY,
        );
        sound.heard_at(heard_at);
        let left = STREAMED_TRACK.loop_end - heard_at as u32;
        self.image.streams_a_track(STREAMED_TRACK, &sound, left);
        *self.sound.borrow_mut() = Some(sound);
    }

    /// Where the track loops, as the pair the game reads it off: where the file is now, plus what the
    /// countdown says is left before it.
    ///
    /// Which is the number a seek must not move — the loop is taken in the same place afterwards — and the
    /// one the countdown being left behind would have moved.
    ///
    /// # Panics
    /// Where no e2e test asked for a sound, there being no file to have a position in.
    pub fn loop_point(&self) -> u32 {
        self.with_the_sound(|sound| sound.position() as u32) + self.image.bytes_left()
    }

    /// `StreamingSound::ServiceBuffer`: one chunk of the file into the buffer at the offset the game
    /// keeps for it, with that offset and the countdown moved on by what was read.
    ///
    /// An e2e test saying the streaming thread ran, the way it says the player was hit. That thread is a
    /// thread, and what it would have done over a few seconds of a track is not something an e2e test can
    /// wait for — so an e2e test that has to have the stream *somewhere else than the chapter left it*
    /// says how far.
    ///
    /// The countdown goes down by what the read took, because that is what the game's own `WaveFile::Read`
    /// does with it (0x43c1be) and it is what makes the loop point hold still while the file moves.
    ///
    /// # Panics
    /// Where the read came up short, which is the end of the track's sound: taking the loop there is the
    /// game's own answer and no e2e test here needs one, so an e2e test that has run its stream that far
    /// has run further than it meant to.
    pub fn services_the_buffer(&self) {
        let at = self.image.next_write_offset();
        let (read, size) =
            self.with_the_sound(|sound| (sound.tops_the_buffer_up(at), sound.buffer_size()));
        assert_eq!(
            read, STREAM_NOTIFY,
            "the stream was serviced past the end of the track's own sound",
        );
        self.image.set_next_write_offset((at + read) % size);
        self.image.set_bytes_left(self.image.bytes_left() - read);
    }

    /// And the mixer playing what is in the buffer: the play cursor moved on by `bytes`.
    ///
    /// Apart from the servicing above because they are two clocks, and the distance between them is what
    /// a margin *is* — see [`orb_core::audio::Margin`]. An e2e test says how far apart they are for the
    /// same reason it says the stream ran at all.
    pub fn plays_the_buffer_on(&self, bytes: u32) {
        self.with_the_sound(|sound| sound.plays_on(bytes));
    }

    /// The whole of what the stream a track is playing looks like from outside the game's memory, which
    /// is what a restore has to put back and what no snapshot of that memory holds.
    ///
    /// # Panics
    /// Where no e2e test asked for a sound.
    pub fn stream_now(&self) -> Stream {
        self.with_the_sound(|sound| Stream {
            buffered: sound.buffered(),
            play_cursor: sound.play_cursor(),
            position: sound.position(),
            playing: sound.playing(),
        })
    }

    /// # Panics
    /// Where no e2e test asked for a sound, there being nothing for any of the above to be about.
    fn with_the_sound<T>(&self, body: impl FnOnce(&orb_sim::Sound) -> T) -> T {
        let sound = self.sound.borrow();
        body(sound.as_ref().expect("a sound this e2e test asked for"))
    }

    /// Has the front end draw its own items, which is how an e2e test reads back what the score file
    /// unlocked: `Extra Start` is on the menu or it is not, and no log can see a menu.
    ///
    /// Off unless an e2e test asks, and that is not tidiness. Baking eight labels a frame is real work, and
    /// the runs that sit on the title menu for thousands of frames are the ones measuring what a frame
    /// costs: with the menu drawn on every one of them, `pacing.rs` went from 18 seconds to 616.
    pub fn draws_its_title_menu(&self) {
        self.draws_the_menu.set(true);
    }

    /// Gives the run an ending to reach: a `.end` script of that many frames of waits, the staff roll's
    /// own after it, and the track each of them plays.
    ///
    /// Nothing by default, which is the scene with nothing in it — and that is not tidiness. An ending orb
    /// can find a script in is one its skip stops at the roll of, and an ending with none is one it runs
    /// the whole scene out of: the two are different lines in the log and different numbers, so which of
    /// them an e2e test is about is the e2e test's to say. See
    /// [`lays_out_an_ending_orb_cannot_find`](Fake::lays_out_an_ending_orb_cannot_find).
    pub fn lays_out_an_ending(&self, waits: i32, roll_waits: i32) {
        self.ending.set(Some(Ending {
            waits,
            roll_waits,
            found: true,
        }));
    }

    /// And the same ending with its job left out of the chain, which is an ending orb has no script to
    /// read: `chain_argument` walks that chain for the object and there is nothing in it to find.
    ///
    /// What an ending already torn down looks like, and the case the skip runs the whole scene out for —
    /// which is how the ending and its roll come to be measured together.
    pub fn lays_out_an_ending_orb_cannot_find(&self, waits: i32, roll_waits: i32) {
        self.ending.set(Some(Ending {
            waits,
            roll_waits,
            found: false,
        }));
    }

    /// Has the title screen fall into its attract demo after [`DEMO_AFTER`] frames with nothing pressed at
    /// it, and leave it on the first press.
    ///
    /// Off by default, and that is not tidiness: a title screen that started a run of its own would change
    /// every e2e test that sits on one, and the pacing's sit there for thousands of frames apiece — a demo
    /// under them would be building stages, taking snapshots and costing the frames they are measuring. So
    /// the one e2e test about a press being eaten asks for it.
    pub fn demos_when_idle(&self) {
        self.demos.set(true);
    }

    /// `esc` and then やめる: the game's own way out of a run, which is one write.
    ///
    /// An e2e test saying so, the way it says the player was hit — the pause menu's two screens are not
    /// here, and `StageMenu::OnUpdateGameMenu` ends them by writing the scene the front end runs in. The
    /// panel, and with it `g_Gui`'s job in the draw chain, stands until the front end is built on the
    /// frame after: that one frame is what this exists to make reachable.
    ///
    /// **Those two screens are refused rather than deferred**, and this is where somebody would be tempted
    /// back to them. Having them would make `GameManager::OnUpdate`'s early `CHAIN_CALLBACK_RESULT_BREAK`
    /// reachable — `isInGameMenu` or `isInRetryMenu` set is a walk that stops at chain priority 4, before
    /// the stage, the player and the bullets — and that break arrives for free the day `Fake::update` is a
    /// walk over priority-ordered jobs. Which is where it belongs: a menu of the game's own laid out here
    /// would be two screens of cursor arithmetic standing in for two screens of cursor arithmetic, and the
    /// one thing above the game that reads either is the pair of flags `read_state` turns into `paused`.
    /// `docs/adr/0008`'s *What was weighed and rejected* is the whole of the reasoning.
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
    /// this is where an e2e test reads it back. There is no file on any disk: see [`Fake::files`].
    pub fn score_file_opens(&self) -> Vec<Open> {
        self.opens.borrow().clone()
    }

    /// Forgets them, so that what an e2e test reads is the opens it means rather than every one since the
    /// game started.
    pub fn forget_score_file_opens(&self) {
        self.opens.borrow_mut().clear();
    }

    /// Takes a score file away. The game's own is what a fresh installation looks like: every read of it
    /// fails, the front end's own among them — and that one has to be before the launch's first frame, which
    /// is the only moment it can be, the front end being built on it and its read inside that build.
    ///
    /// orb's own can go at any point, and what taking it away stands for is somebody moving it aside: the
    /// records it held are gone and every read of it fails the way a first launch's does, while the run
    /// written down under `pointdevice_resume/` is still there to be picked up.
    pub fn has_no_score_file(&self, path: &str) {
        self.files.borrow_mut().remove(path);
    }

    /// The record of spell cards the file of that name holds, or `None` for a file that is not there —
    /// which is orb's own until a ranking screen has written it.
    pub fn score_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files
            .borrow()
            .get(path)
            .map(|file| file.captures.clone())
    }

    /// Whether the result screen ever got as far as drawing the question about saving a replay.
    ///
    /// Which is what "no replay is offered" has to mean if it means anything: orb writes that screen's
    /// state past the question rather than answering it, and a state written a frame too late is a
    /// question somebody has to answer.
    pub fn the_replay_question_was_drawn(&self) -> bool {
        self.replay_question_drawn.get()
    }

    /// The game in a 完全無欠モード run with its first chapter taken, which is where every e2e test about
    /// a run being played from further on starts.
    ///
    /// The walk somebody makes to get there: the mode question answered over the title menu, the shot
    /// answered at the game's own select, and then the frames a stage spends settling before its first
    /// snapshot — 248 of them, `STAGE_SETTLE_FRAMES` and the whole of `MUSIC_WAIT_FRAMES`, since a
    /// laid-out game has no track for that wait to find. `pointdevice_run.rs` takes every step
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

    /// And the same run started again and answered つづきから, which is what plays the buttons it pressed
    /// back into the stage the game has just built.
    ///
    /// # Panics
    /// Naming whichever step the pick-up did not get past.
    pub fn picks_the_run_up(&self) {
        let log = self.log();
        self.frames_until("the title menu ready to act on a press", 300, || {
            let front = self.image().front_end_now();
            self.image().scene() == Scene::FrontEnd
                && front.screen == Screen::Title
                && front.acts_on_a_press()
        });
        self.press(keys::Z);
        self.press_until(keys::Z, "the mode question answered again", || {
            self.image().front_end_now().screen == Screen::ShotType
        });
        self.frames_until("the shot type select ready to act on a press", 90, || {
            self.image().front_end_now().acts_on_a_press()
        });
        self.press(keys::Z);
        self.press_until(keys::Z, "つづきから answered", || {
            log.said("resume: from where it stopped, answered on the keyboard")
        });
        self.frames_until("the run played back into place", 60, || {
            log.said("resume: the landing is")
        });
    }

    /// And in a レガシーモード run, which is the same walk with the question answered the other way: the
    /// item the cursor starts on is 完全無欠モード, so the other mode is one press down — after the frames
    /// the question reads nothing over, a direction being a key that cannot be pressed again for nothing.
    ///
    /// No chapter to wait for at the end of it, which is the whole of what answering this way costs.
    ///
    /// # Panics
    /// Naming whichever step the run did not get past.
    pub fn in_a_legacy_run(&self) {
        let log = self.log();
        self.at_the_title_menu();
        self.press(keys::Z);
        self.frames(READS_KEYS_AFTER);
        self.press(keys::DOWN);
        self.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        self.started_at_the_games_own_screens();
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
        // Through orb's hook over `GameWindow::Create`, which is where the display setting this game was
        // configured with is overruled: the call underneath it is this game's own, and that is what
        // reaches the rewrite.
        orb_core::runtime::create_game_window(std::ptr::null_mut());
        self.window.get()
    }

    /// And a window of the game's that is **not** the one it plays in: another class, which the game does
    /// make — a message-only window for its own device notifications among them.
    ///
    /// Straight through orb's rewrite rather than through `GameWindow::Create`, because that function is the
    /// one call the *play* window is decided inside: any other window the game makes reaches the patched
    /// import on its own. What orb has to do with one is nothing, and the class is what settles that — it is
    /// looked at before the monitor is read, so a window that is not the game's is not even the reason the
    /// host was asked about its panel.
    pub fn creates_another_window(&self) -> orb_api::Hwnd {
        let window = unsafe {
            orb_core::window::create_window_ex_a(
                0,
                OTHER_WINDOW_CLASS.as_ptr().cast(),
                c"th06 notifications".as_ptr().cast(),
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

    /// The game answering `WM_SETCURSOR`, which is the ask orb's rewrite over its `ShowCursor` import
    /// stands in front of.
    ///
    /// Its window procedure's case for that message as 1.02h has it — 0x420d40 dispatching `0x20` to
    /// 0x420dc0 — where a game in a window loads the arrow, sets it, and asks for the pointer to be
    /// drawn. Only the ask is here, that being the half orb has anything to do with; and one of these
    /// arrives per movement of the pointer over the window, which is what makes it something a launch
    /// meets thousands of times.
    ///
    /// Straight through the rewrite, there being no import table in a laid-out game to reach it through.
    pub fn answers_wm_setcursor(&self) -> i32 {
        orb_core::mouse::show_cursor(1)
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
    /// overlay for orb to draw a question with — which is where every e2e test about the question that
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

    /// The whole of what the game does in one update: `Chain::RunCalcChain` over the jobs its scenes have
    /// registered, in the order their priorities put them in.
    fn update(&self) -> i32 {
        // What the frame's own work costs, which is the game's and not orb's: the update, the sounds
        // handed over after it and the draw, as one span, since what the pacing is judged on is how long
        // a frame took between its turn and being handed over. Nothing by default — a laid-out game
        // walks a few writes — and an e2e test about the rate says the size and its unevenness. See
        // `frame_takes`.
        self.sim().clock().advance_micros(self.work_this_frame());
        self.runs_the_calc_chain()
    }

    /// `Chain::RunCalcChain`: every job of the calc list in turn, and what the walk does with each answer.
    ///
    /// **The list is walked live out of the game's own memory** rather than collected first, and that is the
    /// whole of what makes a scene's first update fall on the frame it was built: the supervisor is the
    /// first job and everything it registers goes in above its own priority, so a job registered from
    /// inside this loop is linked behind the position the loop has reached and the loop goes on to it. See
    /// `the_frame_a_scene_is_built_on.rs`.
    ///
    /// Answers the count of jobs it ran, as the game's own does, and the three ends of a walk answer what
    /// 紅魔郷 answers: zero for a job asking the game to stop, one for a break, `-1` for a failure. Which is
    /// the mapping orb reads — see [`CHAIN_LEFT`] and [`CHAIN_FAILED`], and [`CHAIN_CARRIED_ON`] for why a
    /// count above zero is a walk that carried on.
    fn runs_the_calc_chain(&self) -> i32 {
        'restart: loop {
            let mut ran = 0;
            let mut at = self.image.calc_chain_head();
            while at != 0 {
                assert!(
                    ran <= CHAIN_JOBS,
                    "this game's calc chain has run {ran} job(s), which is more than it has",
                );
                let callback = self.image.chain_callback(at);
                if callback == 0 {
                    at = self.image.chain_next(at);
                    continue;
                }
                // `EXECUTE_AGAIN` is answered by no job here and by none in 紅魔郷 either — only the walk's
                // own switch names it — so this loop is here because the walk has it and not because
                // anything needs it.
                let answered = loop {
                    match self.job(callback) {
                        chain_result::AGAIN => continue,
                        answer => break answer,
                    }
                };
                match answered {
                    chain_result::REMOVED => {
                        // The game's own order: the one after is read before the element is cut, cutting it
                        // being what clears its links.
                        let next = self.image.chain_next(at);
                        self.image.cuts_from_the_chain(at);
                        ran += 1;
                        at = next;
                        continue;
                    }
                    chain_result::EXITS => return CHAIN_LEFT,
                    chain_result::BREAKS => return CHAIN_BROKE,
                    chain_result::FAILED => return CHAIN_FAILED,
                    // `ReplayManager::OnUpdateDemo` is the one job in 紅魔郷 that answers this, and the
                    // demo's own job is not laid out here.
                    chain_result::RESTARTS => continue 'restart,
                    _ => {}
                }
                ran += 1;
                at = self.image.chain_next(at);
            }
            return ran;
        }
    }

    /// Which job an element's callback names, and that job run.
    ///
    /// The callbacks are 紅魔郷's own addresses, because that is what orb matches on: `cut_screen_shake`
    /// finds a shake by `ScreenEffect::ShakeScreen`'s address and `chain_argument` finds the ending and the
    /// result screen by theirs. So the element holds the game's address and this is where it becomes one of
    /// this game's own functions.
    /// The argument the walk would hand a job is not passed on: what each of these reaches the game
    /// through is `self`, and what the element's `arg` is *for* is orb finding the object by it.
    fn job(&self, callback: usize) -> i32 {
        match callback {
            chain_job::SUPERVISOR => self.supervisor(),
            chain_job::FRONT_END => self.front_end(),
            chain_job::GAMEPLAY => self.stage(),
            chain_job::ENDING => self.ending(),
            // One job for the two, because in 紅魔郷 they are one screen: the *Score* item and a run's own
            // end both reach `SUPERVISOR_STATE_RESULTSCREEN`, and what tells them apart here is the scene.
            chain_job::RESULT_SCREEN => match self.image.scene() {
                Scene::Ranking => self.ranking(),
                _ => self.result(),
            },
            chain_job::SCREEN_EFFECT => self.shakes_the_screen(),
            other => {
                panic!(
                    "a job of this chain has {other:#010x} as its callback, which is no job of it"
                )
            }
        }
    }

    /// `Supervisor::OnUpdate` at chain priority 0: the keyboard read, the scene that has been asked for
    /// built, and the copy that makes the next one a change nobody has acted on.
    fn supervisor(&self) -> i32 {
        // Its first act: `g_LastFrameInput = g_CurFrameInput; g_CurFrameInput = Controller::GetInput()`.
        // Through orb's hook over that read, which is where a run's buttons are written down and where a
        // run being played back into place is handed the ones it pressed.
        self.image.last_input(self.image.input_now());
        // The pad is read inside that, where the game reads it: `Controller::GetInput` tail-calls
        // `GetControllerInput` and orb's hook over the read is what decides whether either happens at all —
        // a frame whose word orb drops is a frame the joystick is not asked on, in a real launch too.
        let word = orb_core::runtime::get_input();
        // And the replay manager's record over the top of it, where one is playing back. **Here rather than
        // as a job of its own**, which is where 紅魔郷 has it: `ReplayManager::OnUpdate` is priority 15 and
        // `OnUpdateDemo` 5 or 16. A declared divergence, and the reason is that nothing can tell: the word
        // the stage acts on is the record's either way, and the stage's job is the only reader of it.
        let word = match self.replayed_input() {
            Some(recorded) => recorded,
            None => word,
        };
        self.image.input(word);

        // What has been asked for and not built yet, which the supervisor builds here.
        let supervisor = self.image.supervising_now();
        if supervisor.wanted != supervisor.running {
            // `GameManager::DeletedCallback` first, where what is being replaced is a stage: whatever
            // scene comes next, the stage's own jobs are cut and the run's recording is ended.
            if supervisor.wanted == Scene::Playing {
                self.tears_the_stage_down();
            }
            // And `ReplayManager::SaveReplay(NULL, NULL)` on the way from a run to the front end, which is
            // the teardown half of that function — the record's blocks freed and the manager's own job cut.
            // `Supervisor::OnUpdate` has it on each of those paths and on none of the others.
            if supervisor.running == Scene::FrontEnd
                && matches!(supervisor.wanted, Scene::Playing | Scene::Result)
            {
                orb_core::runtime::save_replay(std::ptr::null(), std::ptr::null());
            }
            self.build(supervisor.running);
            // And the word the scene built on this frame is updated with: **nothing**, which is the last act
            // of the state switch — `g_CurFrameInput = g_LastFrameInput = g_IsEigthFrameOfHeldInput = 0`, and
            // only where a transition happened. Both halves of it, because a word zeroed with the frame
            // before it left standing would make the same button a press already spent on the frame after.
            self.image.input(0);
            self.image.last_input(0);
        }
        // And its last act, before any other job runs: the copy that makes a scene written by one of
        // those jobs a change that has been asked for and not acted on. Every one-frame window orb
        // watches for is that gap — see `Th06::run_chosen`.
        self.image.supervising(Supervising {
            running: self.image.scene(),
            wanted: self.image.scene(),
        });
        // The game leaving, where an e2e test said so: the two ways out of a walk are answers a *job* gives,
        // and this is the job that gives them — `CHAIN_CALLBACK_RESULT_EXIT_GAME_SUCCESS` where the
        // supervisor has nothing left to run, and the error beside it.
        match self.answers.get() {
            CHAIN_LEFT => chain_result::EXITS,
            CHAIN_FAILED => chain_result::FAILED,
            _ => chain_result::CONTINUES,
        }
    }

    /// `WAS_PRESSED`: a button in the word this frame was handed and not in the one before it.
    ///
    /// Read out of `g_CurFrameInput` and `g_LastFrameInput` rather than carried into each job, because that
    /// is what they are — two globals every job of the chain works its own presses out against.
    fn pressed(&self, mask: u16) -> bool {
        let word = self.image.input_now();
        let last = self.image.last_input_now();
        word & mask != 0 && word & mask != last & mask
    }

    /// What the supervisor does with a scene that has been asked for: takes down what was running and
    /// builds the one wanted.
    fn build(&self, scene: Scene) {
        match scene {
            // `MainMenu::RegisterChain`, which starts the screen it comes up on from nothing.
            Scene::FrontEnd => {
                // The gameplay scene's own job out of the chain — `GameManager::CutChain`, which
                // `Supervisor::OnUpdate` calls on every path from a run back to the menu — and the menu's own
                // in, at priority 2.
                self.image.cuts_the_gameplay_scene();
                self.image.registers_the_front_end();
                // On its own first screen, which is what "from nothing" is: the memset inside
                // `MainMenu::RegisterChain` takes `gameState` with it, and the menu walks up to its title
                // from there. So whichever of its screens a run was started from is not the one it comes
                // back to.
                let front = self.image.front_end_now();
                self.image.front_end(FrontEnd {
                    screen: Screen::Title,
                    frames: 0,
                    ..front
                });
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
                orb_core::runtime::unlocks_read(self.image.front_end_object() as *mut c_void);
            }
            // A stage asked for that never arrives, where an e2e test said so — see
            // [`never_builds_the_stage_it_is_asked_for`](Fake::never_builds_the_stage_it_is_asked_for).
            // The scene is the one a run is played in and nothing of the run is in it.
            Scene::Playing if self.never_builds.get() => {}
            // `GameManager::AddedCallback`, which is where a stage's numbers are put in place and
            // inside which the stage itself is built.
            Scene::Playing => {
                // `MainMenu::DeletedCallback` first, which is the front end's job cut, and then
                // `GameManager::RegisterChain` linking `g_GameManagerCalcChain` at priority 4 — the added
                // callback below runs from inside `AddToCalcChain`, which is why it is registered first.
                self.image.cuts_the_front_end();
                self.image.registers_the_gameplay_scene();
                orb_core::runtime::stage_begun(self.image.game_manager_object() as *mut c_void);
            }
            // `GameManager::Reinit`: the same callback for the stage after this one, and the whole of
            // what a transition is — one frame, with the next stage built inside it, which is why the
            // scene goes straight back to a stage being played.
            Scene::Rebuilding => {
                // `GameManager::CutChain` and then `GameManager::RegisterChain` again, which is the same
                // static element registered a second time — and what its own links are checked out against.
                self.image.cuts_the_gameplay_scene();
                self.image.registers_the_gameplay_scene();
                orb_core::runtime::stage_begun(self.image.game_manager_object() as *mut c_void);
                self.image.supervising(Supervising {
                    running: Scene::Playing,
                    wanted: Scene::Playing,
                });
            }
            // Not the ending, which this game enters in one act rather than asking for it and building it
            // a frame later — see [`enters_the_ending`](Fake::enters_the_ending).
            Scene::Ending => {}
            // `ResultScreen::RegisterChain(this)`: the screen's own job in the chain, and its frame timer
            // at nothing. It comes up on the high-score name entry, which is the state that call writes
            // for every run but a practice one — a practice run's goes straight to the way out.
            Scene::Result => {
                self.image.cuts_the_gameplay_scene();
                let front = self.image.front_end_now();
                self.image.front_end(FrontEnd { frames: 0, ..front });
                self.image
                    .registers_the_result_screen(if self.run.practice {
                        result_state::EXIT
                    } else {
                        result_state::WRITING_HIGHSCORE_NAME
                    });
                // `ResultScreen::AddedCallback`, the same function the ranking's build runs: one screen,
                // one added callback, and orb's hook over it therefore runs at a finished run's end too.
                unsafe {
                    orb_core::runtime::ranking_read(self.image.result_screen() as *mut c_void)
                };
                self.image.cuts_gui_from_the_draw_chain();
            }
            // The same screen the *Score* item asks for, and the same `ResultScreen::AddedCallback` with
            // it — whose read of the score file fills the record of captures.
            Scene::Ranking => {
                // The front end's job out and the screen's own in, which is the same element and the same
                // callback a finished run's result screen registers: `ResultScreen::RegisterChain(NULL)`,
                // one screen in 紅魔郷 reached two ways.
                self.image.cuts_the_front_end();
                self.image.registers_the_ranking();
                unsafe {
                    orb_core::runtime::ranking_read(self.image.result_screen() as *mut c_void)
                };
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
    fn front_end(&self) -> i32 {
        // Its cursor, which every one of its screens has and only the title menu's is walked here:
        // `image::item` names the two items orb has a question about, and the ranking is one of them.
        let stepped = |cursor: i32| {
            let moved = cursor - i32::from(self.pressed(button::UP))
                + i32::from(self.pressed(button::DOWN));
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
        let decide = self.pressed(Th06.menu_decide()) && front.acts_on_a_press();
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
            Screen::ShotType if self.pressed(Th06.menu_cancel()) => (
                FrontEnd {
                    screen: Screen::Title,
                    cursor: item::GAME_START,
                    frames: 0,
                },
                None,
            ),
            // A ranking asked for that never comes up, where an e2e test said so — see
            // [`never_builds_the_ranking_it_is_asked_for`](Fake::never_builds_the_ranking_it_is_asked_for).
            // The item is in the front end's own state and no scene is asked for.
            Screen::Ranking if self.never_builds_the_ranking.get() => (front, None),
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
        chain_result::CONTINUES
    }

    /// One frame of a stage: its clocks, its generator, its player, and whatever its script has
    /// arranged for that frame.
    ///
    /// **In the order the chain runs the jobs**, because one of the claims is about that order and
    /// nothing else can carry it: `Player::OnUpdate` is priority 7 and the bullets are checked at 11, so
    /// an invulnerability written for this update has to survive the player's own update to be there
    /// when the hit test runs. See [`update_the_player`](Fake::update_the_player).
    fn stage(&self) -> i32 {
        let word = self.image.input_now();
        // The attract demo, which any press leaves — and the press is spent on leaving it, never reaching
        // the menu underneath. Before anything else this update does, since a demo somebody has pressed a
        // key at is not a stage that goes on being played.
        if self.state().demo && self.pressed(Th06.menu_decide() | Th06.menu_cancel()) {
            self.image.supervising(Supervising {
                running: Scene::FrontEnd,
                wanted: Scene::Playing,
            });
            return chain_result::CONTINUES;
        }
        let mut run = self.image.playing_now();
        let mut moving = self.image.reproducing_now();

        run.frames += 1;
        // The clock the enemy script runs on, which advances with the stage however the player is
        // doing — which is why the midstage table is written in it.
        run.script_frames += 1;
        // And the replay's own, which is the field orb reads a playback's position out of.
        self.image.set_replay_clock(run.frames as i32);
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
                self.image.starts_the_card(CARD, CARD_NAME);
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
            // And the fight the stage ends with, which brings the second of the two songs its data names:
            // the track changing is the only thing that tells this fight from the midboss.
            STAGE_BOSS_ARRIVES if self.plays_songs.get() => {
                self.image.plays_a_track(BOSS_TRACK);
                self.image.boss(Some(Boss {
                    life: 2400,
                    attack_frames: 0,
                }));
            }
            // That fight fought out: a card, and the attack after it. Which fight these belong to is the
            // music's to say and nothing else's — see [`Fake::fights_its_boss_out`].
            STAGE_BOSS_CARD_STARTS if self.fights_boss_out.get() => {
                self.image.boss(Some(Boss {
                    life: 2000,
                    attack_frames: 0,
                }));
                self.image.card(Some(STAGE_BOSS_CARD));
                self.image
                    .starts_the_card(STAGE_BOSS_CARD, STAGE_BOSS_CARD_NAME);
            }
            STAGE_BOSS_ATTACK_CHANGES if self.fights_boss_out.get() => {
                self.image.boss(Some(Boss {
                    life: 1600,
                    attack_frames: 0,
                }));
                self.image.card(None);
            }
            // And its name, two frames behind the attack itself. The attack's own clock is advanced here
            // as on any other frame: nothing began on this one, and a reset would say something did.
            STAGE_BOSS_NAMES_ITS_CARD if self.fights_boss_out.get() => {
                self.image.card(Some(STAGE_BOSS_LATE_CARD));
                self.image
                    .starts_the_card(STAGE_BOSS_LATE_CARD, STAGE_BOSS_LATE_CARD_NAME);
                self.advance_the_attack();
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
            return chain_result::CONTINUES;
        }
        // `esc` and then やめる, as a job of the stage's own: `StageMenu::OnUpdateGameMenu` writes the
        // scene the front end runs in, and the supervisor has already made its copy for this frame — so
        // the run is over on this one with the panel, and `g_Gui`'s job in the draw chain, still standing.
        if self.given_up.take() {
            self.image.supervising(Supervising {
                running: Scene::FrontEnd,
                wanted: Scene::Playing,
            });
            return chain_result::CONTINUES;
        }
        // The stage over, where an e2e test said how long one is. A transition goes through the scene the
        // game rebuilds its manager in and the last stage hands over to the ending instead — never to a
        // seventh stage.
        if Some(run.frames) == self.stage_frames.get() {
            if run.stage + 1 < STAGES {
                self.image.supervising(Supervising {
                    running: Scene::Rebuilding,
                    wanted: Scene::Playing,
                });
            } else {
                self.enters_the_ending();
            }
        }
        chain_result::CONTINUES
    }

    /// What the replay's own record says was held on the frame this update is about to play, or `None`
    /// where no replay is playing back and the keyboard is what the run is played on.
    ///
    /// Asked of the record by the stage's own frame counter, which is what the recording was written
    /// against: a replay started at a stage begins that stage at frame nothing, so the same entries are
    /// fed however the stage was reached.
    fn replayed_input(&self) -> Option<u16> {
        let playing_back = unsafe { Th06.replaying() } && self.image.scene() == Scene::Playing;
        playing_back.then(|| {
            let run = self.image.playing_now();
            self.image
                .plays_back_the_inputs(run.stage, run.frames as i32)
        })
    }

    /// `GameManager::DeletedCallback`: the stage taken down, and with it `ReplayManager::StopRecording`
    /// ending the run's recording.
    ///
    /// Through orb's hook over that function, which is what holds the write back while a replay is being
    /// watched — the record it would land in during playback is the replay's own, at the entry playback
    /// has reached.
    fn tears_the_stage_down(&self) {
        orb_core::runtime::stop_recording();
    }

    /// `ScreenEffect::ShakeScreen`: the arcade region written from **two numbers out of the generator
    /// every frame**, for as long as the shake has frames left.
    ///
    /// Which is the whole of what makes a shake more than drawing: those two numbers come out of the
    /// stream a replay has to match, and the region they land in is what a stage's first position for the
    /// player is measured from. The region goes back where it belongs on the frame the shake takes itself
    /// down — which is exactly what a shake removed early has not done yet.
    fn shakes_the_screen(&self) -> i32 {
        // A job of the chain reached because it is *in* the chain, which is the whole of what a shake still
        // running is: orb takes one down at a stage move through `Chain::Cut`, and the frames it had left
        // are not what decides whether the walk gets here.
        let left = self.shake_frames.get();
        if left <= 0 {
            return chain_result::CONTINUES;
        }
        self.shake_frames.set(left - 1);
        let field = Th06.play_area();
        // The frame it removes itself on, which draws nothing: `timer >= effectLength` puts the region back
        // and returns `CHAIN_CALLBACK_RESULT_CONTINUE_AND_REMOVE_JOB` before the offset is worked out at
        // all. **The walk is what cuts it**, not the job — which is why nothing here calls `Chain::Cut`.
        if left == 1 {
            self.image
                .sets_the_arcade_region((field.left, field.top), (field.width, field.height));
            self.image.bombing(false);
            return chain_result::REMOVED;
        }
        // And every other frame: **one of three cases per axis**, chosen by `GetRandomU32InRange(3)`. Two
        // numbers each, because `GetRandomU32` is two `GetRandomU16`s and each of those raises
        // `generationCount` — so four a frame, which is what a replay's stream has to match.
        let mut moving = self.image.reproducing_now();
        let case = |moving: &mut Reproducing| {
            let mut drawn = 0u32;
            for _ in 0..2 {
                moving.seed = drawn_from(moving.seed);
                moving.randoms += 1;
                drawn = (drawn << 16) | u32::from(moving.seed);
            }
            drawn % 3
        };
        let across = case(&mut moving);
        let down = case(&mut moving);
        self.image.reproducing(moving);
        // How far each case moves it. The game ramps this from one of the effect's parameters to the other
        // over the shake's own frames; this is a constant of this game's own — see [`SHAKE_PIXELS`] — since
        // what an e2e test reads of it is that the region is not where a stage left it.
        let offset = f32::from(SHAKE_PIXELS);
        let (left_edge, width) = match across {
            0 => (field.left, field.width),
            1 => (field.left + offset, field.width - offset),
            _ => (field.left, field.width - offset),
        };
        let (top_edge, height) = match down {
            0 => (field.top, field.height),
            1 => (field.top + offset, field.height - offset),
            _ => (field.top, field.height - offset),
        };
        self.image
            .sets_the_arcade_region((left_edge, top_edge), (width, height));
        chain_result::CONTINUES
    }

    /// `Player::OnUpdate`, at chain priority 7: the player's state moved on one frame.
    ///
    /// The invulnerable state is the one that expires, and it expires *here* — at the end of the same
    /// update, 0x428... onwards — which is why `make_invulnerable` writes the frames left and not only
    /// the state: a state written with the frames the last respawn left under it is a player who is
    /// killable again by the time the bullets are checked at priority 11.
    ///
    /// **Two `if`s and not two arms of one match**, because that is what the game has and the difference is
    /// a whole frame of the count: the spawning branch flips the state and the countdown below it runs in
    /// the same update, so a stage's own first update ends on `INVULNERABLE_AFTER_SPAWNING - 1`.
    fn update_the_player(&self) {
        // The frames after a death, which the real game spends on the animation — 30 of them dying, 30
        // spawning, then the same count as below. Here it is one frame: orb freezes the game on the frame
        // the death is noticed, so nothing but a retry or a run given up ever follows one — and a retry
        // puts the player back with the rest of `.data`.
        if self.image.player_now() == Player::Dying {
            self.image.player(Player::Normal);
        }
        // The spawning branch, whose own test is `30 <= invulnerabilityTimer.AsFrames()`: a stage's start
        // satisfies it at once, so the state lasts this one update and what it leaves under the next one is
        // [`INVULNERABLE_AFTER_SPAWNING`] rather than the count the callback wrote.
        if self.image.player_now() == Player::Spawning {
            if self.image.invulnerable_frames() >= SPAWNING_ENDS_AT {
                self.image.player(Player::Invulnerable);
                self.image
                    .set_invulnerable_frames(INVULNERABLE_AFTER_SPAWNING);
            } else {
                self.image
                    .set_invulnerable_frames(self.image.invulnerable_frames() + 1);
            }
        }
        if self.image.player_now() == Player::Invulnerable {
            let left = self.image.invulnerable_frames() - 1;
            self.image.set_invulnerable_frames(left.max(0));
            if left <= 0 {
                self.image.player(Player::Normal);
            }
        }
    }

    /// The ending entered, which this game does in one act: the scene, the job the ending is found
    /// through, the script that job's object holds, and the track it plays, all on the frame the last
    /// stage ends.
    ///
    /// **The measurement is what fixes that.** A stage 6 clear ran the ending out in 29,040 updates and
    /// stopped where the script handed over, which means orb had a script to compare on the very first
    /// read — the frame the scene became the ending's. An ending whose job went into the chain a frame
    /// later would have left that read with nothing, and the skip would have run the roll out too, which
    /// is the case [`lays_out_an_ending_orb_cannot_find`](Fake::lays_out_an_ending_orb_cannot_find) is
    /// here for.
    ///
    /// So the scene is written settled rather than asked for, and there is no `Scene::Ending` for the
    /// supervisor to build.
    fn enters_the_ending(&self) {
        let front = self.image.front_end_now();
        self.image.front_end(FrontEnd { frames: 0, ..front });
        // The gameplay scene's draw jobs go with it, `g_Gui`'s among them, which is the panel off the
        // screen.
        self.image.cuts_gui_from_the_draw_chain();
        // `GameManager::CutChain` and `Ending::RegisterChain`: the run's job out and the ending's in at
        // priority 3. **Always**, an ending that is running being an ending with a job — what
        // `lays_out_an_ending_orb_cannot_find` leaves out is the *script* in the object, which is what
        // `ending_script` reads through and what an ending already torn down has none of.
        self.image.cuts_the_gameplay_scene();
        if let Some(ending) = self.ending.get() {
            if ending.found {
                self.image.registers_the_ending();
            } else {
                self.image.registers_the_ending_without_its_script();
            }
            self.image.plays_a_track(ENDING_TRACK);
        } else {
            self.image.registers_the_ending_without_its_script();
        }
        self.image.supervising(Supervising {
            running: Scene::Ending,
            wanted: Scene::Ending,
        });
    }

    /// The ending, which walks its script out, hands over to the staff roll, and leaves for the result
    /// screen when the roll is done.
    ///
    /// **One update per frame of waits**, which is what makes the count an e2e test reads off the log the
    /// script's own: an `.end` is one-character instructions and waits between them, and every frame of
    /// those waits is an update of this scene whether anything is drawn on it or not.
    ///
    /// Where no ending has been laid out, the scene and nothing in it: a cleared run passes through in
    /// [`ENDING_FRAMES`] frames on the way to its result.
    fn ending(&self) -> i32 {
        let front = self.image.front_end_now();
        let frames = front.frames + 1;
        self.image.front_end(FrontEnd { frames, ..front });
        let Some(ending) = self.ending.get() else {
            if frames >= ENDING_FRAMES {
                self.leaves_the_ending();
            }
            return chain_result::CONTINUES;
        };
        // The ending's last act: `@Fdata/staff00.end` read over the script that was running, with the
        // roll's own track started on the same update. Both on one update because that is where they are
        // in the game — the `F` instruction at 0x40fc06 reads the file and carries straight on into it —
        // and the two agreeing is the whole of what says the roll begins here.
        if frames == ending.waits {
            if ending.found {
                self.image.hands_over_to_the_roll();
            }
            self.image.plays_a_track(ROLL_TRACK);
        }
        if frames >= ending.waits + ending.roll_waits {
            self.leaves_the_ending();
        }
        chain_result::CONTINUES
    }

    /// The scene taken down and the result screen asked for, which is what follows an ending either way.
    fn leaves_the_ending(&self) {
        self.image.cuts_the_ending();
        self.image.takes_the_music_down();
        self.image.supervising(Supervising {
            running: Scene::Result,
            wanted: Scene::Ending,
        });
    }

    /// The result screen a run's own end arrives on, which walks its states out and leaves for the title.
    ///
    /// **The name entry and the screen a replay is saved from are states of this one screen and not scenes
    /// of their own**, which is what makes the difference between them orb's to make: the name entry is a
    /// screen somebody types into and orb has nothing to say to it, and the question is one a pointdevice
    /// run cannot answer, which orb writes the state past rather than answering — answering means playing
    /// out a fade. So what "no replay is offered" is, is that write; what says the question was never put
    /// to anybody is that no frame of it was ever drawn; and what says the name entry was left alone is
    /// that the screen is still standing in that state.
    ///
    /// The frames each state stands for are this game's own, the real ones standing until they are
    /// answered. The stats screen between the two is not laid out at all — orb does nothing at that one
    /// either, and what these e2e tests are about is the screens it decides between.
    fn result(&self) -> i32 {
        let front = self.image.front_end_now();
        let frames = front.frames + 1;
        self.image.front_end(FrontEnd { frames, ..front });
        // The frame timer back to nothing with the state, which every one of the game's own transitions
        // does: each state waits out a count of its own.
        let moves_on = |state: i32| {
            self.image.set_result_screen_state(state);
            self.image.front_end(FrontEnd { frames: 0, ..front });
        };
        match self.image.result_screen_state() {
            // Either way out. `EXIT` is the game's own case for it — the scene put back and the job taken
            // out — and `EXITING` is where its own menus go to play the fade that ends in the same thing.
            result_state::EXIT | result_state::EXITING => self.leaves_the_result_screen(),
            result_state::WRITING_HIGHSCORE_NAME if frames >= NAME_ENTRY_FRAMES => {
                moves_on(result_state::SAVE_REPLAY_QUESTION);
            }
            result_state::SAVE_REPLAY_QUESTION => {
                // Drawn from the frame its own animation starts on. Where the state has been written past
                // it there is nothing to draw.
                if frames >= REPLAY_QUESTION_DRAWN_AT {
                    self.replay_question_drawn.set(true);
                }
                if frames >= REPLAY_QUESTION_FRAMES {
                    moves_on(result_state::EXITING);
                }
            }
            _ => {}
        }
        chain_result::CONTINUES
    }

    /// That screen going down: the score file written by its deleted callback, its job out of the chain,
    /// and the scene put back — `ResultScreen.cpp:1527-1535`, which is how it leaves.
    ///
    /// The write is the whole reason orb has this screen's other half built for a run given up: what the
    /// run counted is in memory and nowhere else until this happens. Written whether a score was entered
    /// or the screen was only read, as the deleted callback writes it — 0x42f5cd, the one caller of the
    /// write in the whole exe.
    fn leaves_the_result_screen(&self) {
        self.writes_the_score_file();
        self.image.cuts_the_result_screen();
        self.image.supervising(Supervising {
            running: Scene::FrontEnd,
            wanted: Scene::Result,
        });
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

    /// The ranking screen: it comes up out of the state its build left, it leaves when it is told to or
    /// when somebody presses back, and putting the scene and the front end's own state back is its own
    /// doing. Going down is what writes the file.
    fn ranking(&self) -> i32 {
        // `OnUpdate`'s `INIT` case, which is the screen showing what it read: the state `RegisterChain(NULL)`
        // left is nothing, and this is the first one orb reads as a ranking that is up.
        if self.image.result_screen_state() == result_state::INIT {
            self.image.shows_the_ranking();
            return chain_result::CONTINUES;
        }
        if !self.image.result_screen_leaving() && !self.pressed(Th06.menu_cancel()) {
            return chain_result::CONTINUES;
        }
        // Going down is what writes the file, which is the whole reason orb has this screen built for a
        // run: what the run counted is in memory and nowhere else until this happens. Written whether a
        // score was entered into the ranking or the ranking was only looked at, as the deleted callback
        // writes it — 0x42f5cd, the one caller of the write in the whole exe.
        self.writes_the_score_file();
        let front = self.image.front_end_now();
        self.image.front_end(FrontEnd {
            screen: Screen::Title,
            frames: 0,
            ..front
        });
        // And the screen's own job out of the chain with it, which is the same element a finished run's
        // screen registers: one screen in 紅魔郷 and so one element here.
        self.image.cuts_the_result_screen();
        self.image.supervising(Supervising {
            running: Scene::FrontEnd,
            wanted: Scene::Ranking,
        });
        chain_result::CONTINUES
    }

    /// What the game draws of its own.
    ///
    /// Nothing of a stage: a laid-out game has no sprites and an e2e test reads a run out of its memory
    /// rather than off the screen. Its two screens whose *contents* are a fact about what the game read
    /// are drawn, because for those the memory is not where the answer is — the ranking, which shows the
    /// record orb has been writing into, and the title menu, whose items are what the read that lights
    /// them found.
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
        match self.image.scene() {
            Scene::FrontEnd => self.draws_the_title_menu(),
            Scene::Ranking => self.draws_the_ranking(),
            _ => {}
        }
    }

    /// The title menu's own items, each on a row of its own, where an e2e test asked for them — see
    /// [`draws_its_title_menu`](Fake::draws_its_title_menu).
    ///
    /// `Extra Start` among them only where the score file's `clrd` chunk left the front end something to
    /// light it from, which is what makes a read that failed cost something an e2e test can see: the
    /// destination is cleared before the chunk is looked for, so a menu built after one offers a stage
    /// nobody can reach.
    fn draws_the_title_menu(&self) {
        if !self.draws_the_menu.get() || self.image.front_end_now().screen != Screen::Title {
            return;
        }
        // `GameManager::HasReachedMaxClears`, asked of every record rather than of one: what the item says
        // is that there is an Extra to reach at all, and the game's own Extra character select asks the same
        // question of each of the two shots a character has — `MainMenu.cpp`'s
        // `HasReachedMaxClears(character, 0) || HasReachedMaxClears(character, 1)`.
        let unlocked = (0..2)
            .any(|character| (0..2).any(|shot| self.image.has_reached_max_clears(character, shot)));
        for (row, name) in TITLE_MENU.iter().enumerate() {
            if row as i32 == item::EXTRA && !unlocked {
                continue;
            }
            let y = TITLE_TOP + row as f32 * TITLE_LINE;
            self.launch.writes(name, TITLE_LEFT, y, INK);
        }
    }

    /// The ranking's own rows: one per spell card the game holds a record of, with the name it holds and
    /// the attempts against it, which is the number 完全無欠 counts.
    ///
    /// **The name is drawn only while the attempts are not zero**, which is the branch at 0x42e26e: the
    /// count is read at 0x42e265 and a row whose count is nothing gets [`NOT_TRIED`] at 0x46bcdc instead of
    /// the name at 0x42e2ac. So a count against a record nobody named is what puts the bytes a fill left
    /// there on the screen.
    fn draws_the_ranking(&self) {
        for (row, (card, attempts)) in self.image.card_records().into_iter().enumerate() {
            let y = RANKING_TOP + row as f32 * RANKING_LINE;
            self.launch
                .writes(&format!("CARD {card}"), RANKING_LEFT, y, INK);
            let name = if attempts == 0 {
                NOT_TRIED.to_owned()
            } else {
                self.image.card_name(card).unwrap_or_default()
            };
            self.launch.writes(&name, RANKING_NAME, y, INK);
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
            orb_core::score::create_file_a(
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

    /// The game's read of that file at `GameManager::AddedCallback` and at the ranking screen's, which
    /// are the two that parse every chunk of it.
    ///
    /// A failed open is not a no-op for all of them alike: `clrd`'s parse at 0x42b502 clears its
    /// destination before it looks for the chunk, so what the front end is left with is an empty record
    /// and not the one it had, while `catk`'s at 0x42b466 has no clear of its own and leaves the record
    /// in memory standing.
    fn reads_the_score_file(&self) {
        let read = self.reads_the_unlocks();
        unsafe { Th06.set_captures(&read.captures) };
    }

    /// And its read at `MainMenu::AddedCallback`, which parses `clrd` and `pscr` and nothing else: this
    /// is the one read the front end's own items are lit from, and the record of spell cards is not what
    /// it is for.
    ///
    /// Answers what the open found, so that the read above can take the captures out of the same one.
    fn reads_the_unlocks(&self) -> ScoreFile {
        let read = self
            .opens_the_score_file(false)
            .and_then(|path| self.files.borrow().get(&path).cloned())
            .unwrap_or_default();
        self.image.parses_the_unlocks(&read.unlocks);
        read
    }

    /// And its write, which has one caller in the whole exe: 0x42f5cd, in the ranking screen's deleted
    /// callback.
    ///
    /// A refused open leaves the file as it was and the game carries on — `WriteDataToFile` checks its open
    /// and its caller drops the answer — which is what makes refusing the write a run that is not written
    /// rather than a game that stops.
    fn writes_the_score_file(&self) {
        let written = ScoreFile {
            captures: unsafe { Th06.captures() },
            unlocks: self.image.unlocks(),
        };
        if let Some(path) = self.opens_the_score_file(true) {
            self.files.borrow_mut().insert(path, written);
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
    /// **One function reached two ways**, and the whole of the difference is its own first condition:
    /// `g_Supervisor.curState != SUPERVISOR_STATE_GAMEMANAGER_REINIT`. A run's first stage takes that
    /// branch and a transition takes the two-line `else` the game's own is — `guiScore = score`, which is
    /// one number here, and `nextScoreIncrement = 0`, which is not modelled — so what a transition does is
    /// carry the run. See [`a_run_starts_here`](Fake::a_run_starts_here) for how the two are told apart.
    ///
    /// Its read of the score file's record of spell cards is in that branch, where the real one parses
    /// `catk` — which is why orb holds that record across a run played back into place: playing a
    /// stage in again starts every card the run had passed, and this read is what a landing would
    /// otherwise be left with. See `resume::hold_captures`.
    fn stage_numbers_in_place(&self) {
        // A stage other than the one asked for, or a run orb keeps nothing of, where an e2e test said so —
        // see [`comes_up_as`](Fake::comes_up_as). Before the number is raised and the run's own fields are
        // written, which is where the game itself settles both.
        match self.comes_up_as.get() {
            Some(ComesUpAs::AnotherStage(stage)) => self.image.builds_stage(stage),
            Some(ComesUpAs::TheDemo) => self.image.demo_mode(true),
            None => {}
        }
        let starting = self.a_run_starts_here();
        if starting {
            // The fill first and the read over the top of it, which is the order inside that branch: the
            // records are the generator's until the file's own are copied in, and a card the file holds no
            // record of is left named nothing anybody wrote.
            self.image.fills_the_card_records();
            self.reads_the_score_file();
        }
        // The stage's own song, which its data names first and which plays through the midstage and the
        // midboss alike. Per stage, as the game loads it per stage — outside the condition, where the
        // game's own two `LoadPbg3` calls and its `ReadMidiFile`/`PlayAudio` pair are.
        if self.plays_songs.get() {
            self.image.plays_a_track(STAGE_TRACK);
        }
        let stage = self.image.stage_built();
        let previous = self.image.playing_now();
        let mut moving = self.image.reproducing_now();
        // `ReplayManager::AddedCallbackDemo`, where one is playing back: the stage's seed comes out of the
        // replay's record rather than out of wherever the generator was left. Which is what makes a stage
        // reached by moving the same stage as one reached by playing into it — the seed is the one number
        // a wrong stage start can be wrong in while every other field is right.
        if unsafe { Th06.replaying() } {
            moving.seed = self.image.recorded_seed(stage);
        }
        // The numbers a run is played with, which are the run's and not the stage's: `livesRemaining` and
        // `bombsRemaining` are written nowhere in this callback on either branch — the front end puts them
        // there out of `g_Supervisor.defaultConfig` — and `currentPower = 0` and `deaths = 0` are inside
        // the branch a transition does not take.
        let (deaths, lives, bombs, power) = if starting {
            (0, FRESH.0, FRESH.1, FRESH.2)
        } else {
            (
                previous.deaths,
                previous.lives,
                previous.bombs,
                previous.power,
            )
        };
        self.image.playing(Playing {
            stage,
            difficulty: self.run.difficulty,
            frames: 0,
            script_frames: 0,
            // The generator's own seed, copied where the callback copies it: this is what a stage
            // written down is read back from, and it stays put once the stage draws from it.
            seed: moving.seed,
            deaths,
            lives,
            bombs,
            power,
            enemies: 0,
        });
        // Nothing of the last stage's fight is left standing.
        self.image.boss(None);
        self.image.card(None);
        // The arcade region and the box the player is held inside, both in that same branch: a run puts
        // them where the game has them and a transition leaves them alone, which is the whole reason a
        // bomb's screen shake can reach the stage after the one that started it.
        if starting {
            let field = Th06.play_area();
            self.image
                .sets_the_arcade_region((field.left, field.top), (field.width, field.height));
            self.image
                .play_field(PLAYER_AREA_TOP_LEFT.1, PLAYER_AREA_SIZE.1);
            // `mgr->rank = g_DifficultyInfo[difficulty].rank`, which is in that branch too and so is what
            // a transition carries rather than resets.
            self.image.set_rank(RANK_AT_A_RUNS_START);
        }
        // And `mgr->subRank = 0`, which is outside the condition: every stage starts from nothing there.
        self.image.set_sub_rank(0);
        // `g_Rng.generationCount = 0; mgr->randomSeed = g_Rng.seed;` — the two lines immediately before
        // `Stage::RegisterChain`, which is why the count of numbers drawn goes in *here* and not with the
        // player's position below.
        self.image.reproducing(Reproducing {
            randoms: 0,
            ..moving
        });
        // `Stage::RegisterChain`, called from inside this callback: the one moment a resumed run's
        // seed can go in, since building the stage is what draws from it.
        orb_core::runtime::stage_building(stage);
        // `Player::RegisterChain` and the `Player::AddedCallback` inside it, **after** the stage's own
        // build and not before it, which is the order `GameManager::AddedCallback` has.
        //
        // No e2e test can fail on the order today and that is the finding rather than an omission:
        // `resume::stage_building` writes the generator's seed and nothing else, so nothing reads a player
        // or a draw chain that was put in place too early. What it would cost is a debugging session on the
        // day that call grows a second write — which is why the order is right here now rather than then.
        //
        // Spawning, with [`SPAWNS_WITH`] on the invulnerability count. Neither is what the stage is played
        // with — the stage's own first update flips the state and replaces the count in the same frame —
        // but they are what is written here, and the count is why that flip is immediate. See
        // [`update_the_player`](Fake::update_the_player).
        self.image.player(Player::Spawning);
        self.image.set_invulnerable_frames(SPAWNS_WITH);
        // And where the player starts, measured from the *arcade region* and not from the box they are held
        // inside — see [`PLAYER_STARTS_ABOVE`]. Read back rather than carried in `moving`, because the
        // stage's own build has drawn from the generator since: writing the fields that call moved would put
        // the numbers it drew back.
        let (across, down) = self.image.arcade_region_size();
        let drawn = self.image.reproducing_now();
        self.image.reproducing(Reproducing {
            player: (across / 2.0, down - PLAYER_STARTS_ABOVE),
            ..drawn
        });
        // `Gui::RegisterChain`, which puts `g_Gui`'s own draw job in the chain: the panel is on the
        // screen from here until the scene is taken down, and that job being in the draw list is the
        // whole of what `Th06::draws_lives_row` asks. Registered per stage, as the real one is — a
        // second registration of the same static element is what the element's own links check out
        // against.
        self.image.registers_gui_in_the_draw_chain();
    }

    /// Whether the callback putting a stage's numbers in place is doing it for the **start of a run**
    /// rather than for a transition to the next stage, which is the game's own
    /// `g_Supervisor.curState != SUPERVISOR_STATE_GAMEMANAGER_REINIT`.
    ///
    /// Read out of the supervisor's own state and nothing kept beside it, which needs no new plumbing:
    /// [`build`](Fake::build) calls `orb_core::runtime::stage_begun` *before* it writes the supervisor's copy, so the
    /// scene here is still the one the supervisor is on — `Scene::Rebuilding` where the game is
    /// reinitialising its manager for the next stage, and already `Scene::Playing` where a run is being
    /// started.
    fn a_run_starts_here(&self) -> bool {
        self.image.scene() != Scene::Rebuilding
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

    /// The game's startup check: `joyGetPosEx(0, JOY_RETURNALL)` once, to find out whether there is a
    /// joystick at all.
    ///
    /// **Through orb's replacement of that import entry**, which is what that read is for: with nothing
    /// plugged in the call takes most of a frame and spends nearly all of it on the CPU, and orb answers it
    /// out of a sample taken on a thread of its own.
    ///
    /// Once and not once a frame, which is what the real game does with it now that orb answers the pad
    /// half of the input read itself: `Controller::GetControllerInput` is where the per-frame read was, and
    /// that function is hooked and not called through. The startup check is the game's own and stays.
    fn checks_for_a_joystick(&self) {
        let mut info = orb_api::JoyInfo {
            size: size_of::<orb_api::JoyInfo>() as u32,
            flags: RETURN_ALL,
            ..orb_api::JoyInfo::default()
        };
        unsafe { orb_core::joystick::answer(JOYSTICK_0, &mut info) };
    }

    /// The keys the game sees, as its own read hands them back — `Controller::GetInput` and both of its
    /// branches.
    ///
    /// **Which branch is which is the whole subject of `keys_from_another_program.rs`.** While the
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
        unsafe { orb_core::runtime::detached() };
        RUNNING.set(std::ptr::null());
    }
}

/// The device's own three functions, which is all of a controller the game's read calls through.
///
/// Real functions rather than anything in the laid-out memory, because code cannot be laid out: the
/// vtable in that memory holds their addresses, the same way the game's own memory holds the address
/// of the Direct3D device orb draws through.
/// `IDirectInputDevice8::Poll` on the controller, which answers what an e2e test declared — see
/// [`its_controller_poll_fails`](Fake::its_controller_poll_fails).
unsafe extern "system" fn poll(_device: usize) -> i32 {
    if running().controller_poll_fails.get() {
        INPUT_LOST
    } else {
        0
    }
}

/// `DIERR_INPUTLOST`, which is what a device answers once it has been lost — the window went away, or
/// another process took it. Negative as an `HRESULT` is, which is what orb reads.
const INPUT_LOST: i32 = 0x8007_001eu32 as i32;

/// And its `Acquire`, which is what the game calls after a poll that failed. Nothing to do but say it was
/// called: what a device being acquired *means* is nothing this game keeps, the same way the keyboard
/// device's acquire keeps only the count — see [`keyboard_acquire`].
unsafe extern "system" fn acquire(_device: usize) -> i32 {
    let fake = running();
    fake.controller_acquires
        .set(fake.controller_acquires.get() + 1);
    0
}

/// And the keyboard device's own three, which is all of one orb ever calls: the acquire it makes after the
/// window has been away, and the `Unacquire` and `Release` it makes to let the device go.
///
/// What being held *means* is read off the pointer the game keeps — `Image::holds_a_keyboard_device` —
/// because that is what the game's own read branches on and what orb clears: a device object that
/// remembered its own acquired state would be a second answer to the one question.
///
/// So the only thing the acquire itself keeps is that it was called, which is what an e2e test reads
/// instead of a log line, and whether this device is refusing — see
/// [`refuses_the_keyboard_acquire`](Fake::refuses_the_keyboard_acquire).
unsafe extern "system" fn keyboard_acquire(_device: usize) -> i32 {
    let fake = running();
    fake.keyboard_acquires.set(fake.keyboard_acquires.get() + 1);
    if fake.refuses_the_acquire.get() {
        OTHER_APP_HAS_PRIORITY
    } else {
        0
    }
}

/// `DIERR_OTHERAPPHASPRIO`, which is what DirectInput answers an acquire of a `DISCL_FOREGROUND` device
/// whose window is not the one in front. Negative as an `HRESULT` is, which is what orb reads.
const OTHER_APP_HAS_PRIORITY: i32 = 0x8007_0005u32 as i32;

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
/// lists them, and an e2e test names them through [`MAPPING`] rather than by number.
pub const MAPPING: Mapping = Mapping {
    shoot: 0,
    bomb: 1,
    menu: 2,
    up: 3,
    down: 4,
    // A button of its own, which is the configuration the game's own defaults are: shoot 0, bomb 1,
    // focus 2, skip 3, menu 4. What the *other* configuration does — focus on the shoot button — is
    // `a_pad_the_game_has_no_device_for.rs`'s, which maps the two together itself.
    focus: 5,
    // A quarter of the ±1000 the game gives an axis, which is far enough that the middle is not it. The
    // two differ so that a test pushing one axis past the other's threshold would be seen doing it.
    x_axis: 250,
    y_axis: 400,
};

/// What comes up where a stage was asked for, as an e2e test declares it — see [`Fake::comes_up_as`].
///
/// The two things orb tells from the stage it asked about, and it tells them apart: a run it keeps nothing
/// of is one thing and the wrong stage of one it does keep is another, and the line it writes says which.
#[derive(Clone, Copy)]
pub enum ComesUpAs {
    /// A stage of this run other than the one asked for, counted from zero.
    AnotherStage(i32),
    /// The attract demo, which is a run orb keeps nothing of — and the plainest of the three the log line
    /// names, the others being a replay and a launch in a mode that keeps nothing.
    TheDemo,
}

/// The exe this game is running as, which is 紅魔郷's own file name.
///
/// Which is what the process orb wakes up inside is recognised by, and how this game's `Game` is found:
/// nothing else of a game is readable before the game it is has been settled, so the name is the whole of
/// what `game::found` is given — see [`the_game_this_is`].
pub const EXE: &str = "東方紅魔郷.exe";

/// The `Game` orb is attached to here, out of the table by [`EXE`] rather than named.
///
/// **Named twice is where the two sides could disagree.** `orb::attach` reads the exe's own file name and
/// asks the table which game that is; a game laid out by hand is running under a name too, and an e2e test
/// that wrote `&Th06` here would be one where the table could hold anything at all and no launch would
/// notice. So this asks the same question the attach asks, and `the_process_orb_woke_up_in.rs` is the other
/// answer to it.
///
/// # Panics
/// Where no entry holds that name, which is the table having lost this game.
fn the_game_this_is() -> &'static dyn orb_core::game::Game {
    orb_core::game::found(EXE)
        .unwrap_or_else(|| panic!("{EXE} is a game orb knows"))
        .game
}

extern "fastcall" fn update(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(UPDATE);
    fake.update()
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
/// driving the game, so there is nothing here for an e2e test to be held up by.
extern "fastcall" fn own_render(_window: *mut c_void) -> i32 {
    unsafe { orb_core::runtime::run_draw_chain(Th06.chain()) };
    let walked = unsafe { orb_core::runtime::run_calc_chain(Th06.chain()) };
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

/// `GameWindow::Create` (0x420c10): the one call everything about the game's window is decided inside,
/// which is why there is nothing to flash on the screen and nothing to resize afterwards.
///
/// What this game does in it is ask for a window — through orb's own rewrite, which is where the ask it
/// makes is replaced. Reached from orb's hook over this function, so the setting the game was configured
/// with has already been overruled by the time the ask happens.
extern "C" fn game_window_create(_instance: *mut c_void) {
    let fake = running();
    let window = unsafe {
        orb_core::window::create_window_ex_a(
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
    fake.window.set(orb_api::Hwnd(window as usize));
}

/// The game's own `joyGetPosEx`, as its import table held it before orb replaced the entry: winmm's,
/// which behind the seam is the host's.
///
/// What orb calls when it has no sample of its own to answer with, which is the game's startup check and
/// the caps read behind it.
unsafe extern "system" fn joystick_position(device: u32, into: *mut orb_api::JoyInfo) -> u32 {
    let flags = unsafe { (*into).flags };
    let (result, info) = orb_api::joystick::position(device, flags);
    unsafe { *into = info };
    result
}

/// The joystick the game asks about, and what it asks for: `JOY_RETURNALL`, which is every field.
const JOYSTICK_0: u32 = 0;
const RETURN_ALL: u32 = 0xff;

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

/// `SoundPlayer::PlaySounds`: the time an e2e test says this frame's sounds cost, a laid-out game having
/// no sound system to spend it in.
///
/// Here rather than left out because the frame loop calls it where the game's own loop did, and a frame
/// that skipped it would be one span of the pacing's breakdown short. What it costs is declared apart
/// from the rest of the frame's work — see [`Work::sound_us`] — because it is spent *here*, and where
/// that is against the handover is the whole of what an e2e test about this call can ask.
unsafe extern "fastcall" fn play_sounds(_player: usize) {
    let fake = running();
    fake.launch.asked_for(SOUND);
    fake.sim().clock().advance_micros(fake.sound_this_frame());
}

/// `GameWindow::Present`: the frame handed over, which from here is the compositor's.
///
/// Where an e2e test counts a frame — the tick it was handed over at, which is what a rate is read off —
/// and where the host is told, since the next flush is what waits for this frame to be composed.
///
/// The tick is peeked rather than read: an e2e test writing down when something happened should not be
/// what moves the clock on, and every other read of this counter in a frame is orb's own.
unsafe extern "fastcall" fn present(_window: usize) {
    let fake = running();
    fake.launch.asked_for(PRESENT);
    fake.launch.handed_over(fake.sim().clock().peek());
    fake.sim().presented();
    // And the device's own `Present`, which is what this function does in the game and the one path orb's
    // letterbox is on: the slot is read out of the vtable rather than remembered, because what is in it is
    // whatever orb last put there.
    //
    // Only where the vtable is laid out, which is a launch whose device has been found: a game that never
    // had one presents through nothing, and that is the launch `attach_before_its_device` starts in.
    let device = fake.launch.device();
    if fake
        .image
        .space()
        .read_committed::<usize>(device.0)
        .is_some()
    {
        let slot: usize = fake.image.space().read(fake.launch.present_slot());
        let through: DevicePresent = unsafe { std::mem::transmute(slot) };
        let source = ASKS_TO_PRESENT;
        unsafe {
            through(
                device.0 as *mut c_void,
                &raw const source,
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

/// What this game asks its device to present: the whole of its back buffer, and no destination — which is
/// the stretch over the whole client that orb's replacement is there to narrow.
///
/// **A source rectangle where the real game may pass none.** What 紅魔郷 hands its device has not been read,
/// and this is a rectangle rather than a null because it is the thing orb *drops*: a game that passed
/// nothing there would leave nothing to see dropped, and the claim is that the letterbox is presented from
/// the whole back buffer whatever the caller asked for. Its size is the game's own content size.
pub const ASKS_TO_PRESENT: orb_api::Rect = orb_api::Rect {
    left: 0,
    top: 0,
    right: 640,
    bottom: 480,
};

/// What the game's device does with a present: writes down the two rectangles it was given, and answers
/// whichever an e2e test declared — `S_OK`, or the refusal a driver that will not stretch gives.
///
/// The one slot of the device's vtable anything is reached through; see `Launch::vtable_bytes`.
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

/// What a present that worked answers, and what a driver that will not stretch on one answers instead.
///
/// `D3DERR_DRIVERINTERNALERROR`, which is what such a refusal really comes back as: any negative number
/// would do — orb reads the sign and nothing else — and this is the one a driver gives.
const S_OK: orb_api::Hresult = 0;
const STRETCH_REFUSED: orb_api::Hresult = 0x8876_0827u32 as i32;

/// One present the game's device was asked for: the two rectangles, with `None` for a null.
///
/// Which is the whole of what orb's replacement of that slot decides — the back buffer's whole surface into
/// a rectangle of the game's own ratio — and what no other instrument can see: the rectangle is handed
/// straight to Direct3D, so an e2e test that only read `letterbox()` would be reading orb's arithmetic
/// rather than what reached the device.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Presented {
    pub source: Option<orb_api::Rect>,
    pub destination: Option<orb_api::Rect>,
}

extern "fastcall" fn draw(_chain: *mut c_void) -> i32 {
    let fake = running();
    fake.launch.asked_for(DRAW);
    fake.draw();
    CHAIN_CARRIED_ON
}

extern "system" fn input() -> u16 {
    // `Controller::GetInput`'s own tail call — `Controller::GetControllerInput` (0x41cfc0) — which orb hooks
    // and does not call through: the keyboard's word goes in and what every pad does to it comes back. So
    // this game has no pad arithmetic of its own to be right or wrong, which is what leaves
    // `the_pad_half_of_the_input_read.rs` asserting about orb.
    orb_core::runtime::get_controller_input(u32::from(running().read_the_keyboard()))
}

/// `ReplayManager::SaveReplay` (0x42a5c0): the record written out under the name it was given.
///
/// Reached through orb's hook over it, which is what drops the write — and only the write: the same function
/// called with no path is the teardown every way out of a run goes through, and the hook calls that through.
///
/// **Where the path is null there is nothing here**, and that is not an omission. What the real one does
/// there is free each stage's block of recorded inputs and cut the manager's own job out of the chain, and
/// nothing above the game reads either — the record an e2e test reads back is the one `Image::loads_a_replay`
/// laid out, and the blocks it would free are not in it.
extern "C" fn save_replay(path: *const u8, name: *const u8) {
    if path.is_null() {
        return;
    }
    let said = |at: *const u8| {
        unsafe { CStr::from_ptr(at.cast()) }
            .to_string_lossy()
            .into_owned()
    };
    running()
        .replays_written
        .borrow_mut()
        .push((said(path), said(name)));
}

/// The file this game saves a replay into and the name it puts in one, which are the only two it ever passes
/// `SaveReplay`.
///
/// This game's own, and pointers into this process rather than into its laid-out memory: what crosses the
/// call is a `char *` each, and the whole of what an e2e test reads back is which name a write landed under.
const REPLAY_FILE: &CStr = c"replay/th6_ud0000.rpy";
const REPLAY_NAME: &CStr = c"ORB";

/// `GameWindow::InitD3dDevice` (0x421420): the render states the game sets on a device that has just been
/// made, and again after every reset.
///
/// **Nothing**, a laid-out game having no render states — every one of them is a `SetRenderState` on
/// `g_Supervisor.d3dDevice`. Here rather than left out because orb gets in front of this to redirect the
/// device's `Present` before anything is presented through it, and the hook is the whole reason the call is
/// in `Originals`: a game whose device was already there when orb attached would never reach it. See
/// [`Fake::finds_its_device`].
extern "C" fn init_d3d_device() {}

/// `Chain::Cut` (0x41cde0): the element unlinked, which is one of the three calls into the game orb makes
/// that an e2e test reaches — a screen shake still running at a stage move is taken down through it.
///
/// A real function rather than anything in the laid-out memory, for the same reason the controller's three
/// are: code cannot be laid out.
unsafe extern "thiscall" fn chain_cut(_chain: usize, elem: usize) {
    running().image.cuts_from_the_chain(elem);
}

/// `SoundPlayer::StopBGM` (0x430f80): the whole teardown of a track — the buffer stopped, the streaming
/// thread joined, the stream deleted and the pointer cleared.
///
/// The pointer cleared is the half of it anything above the game can see, and it is the half that matters:
/// every read orb makes of the music goes through it, so a game with it cleared is a game with nothing
/// playing.
unsafe extern "fastcall" fn stop_bgm(_player: usize) {
    let fake = running();
    fake.image.takes_the_music_down();
    fake.music_stops.set(fake.music_stops.get() + 1);
}

/// `Supervisor::PlayAudio` (0x424b5d): the path's extension swapped for `.wav`, the file loaded, and the
/// track played.
///
/// The track laid out again is what the game's own load leaves behind, and it is the stage's own song —
/// this is the call a restore makes with the path read out of the memory it has just put back.
unsafe extern "thiscall" fn play_audio(_supervisor: usize, path: usize) -> i32 {
    let fake = running();
    fake.music_starts.borrow_mut().push(path_at(path));
    fake.image.plays_a_track(STAGE_TRACK);
    0
}

/// The path at an address in the game's own memory, as the game reads one: bytes up to the terminator.
fn path_at(address: usize) -> String {
    let bytes = unsafe { orb_api::mem::read::<[u8; SONG_PATH_BYTES]>(address) };
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// How long the game keeps a song path in: `char[128]`, which is what `Image::names_its_song` writes.
const SONG_PATH_BYTES: usize = 128;

/// `ReplayManager::StopRecording` (0x42aab0): the two entries that finish an input record off, written
/// into it at the entry playback has reached.
///
/// Which is right for a recording and wrong for a replay, and orb's hook over it is what tells the two
/// apart — so an e2e test reads the record back to find out whether this ran.
extern "C" fn stop_recording() {
    let fake = running();
    let run = fake.image.playing_now();
    fake.image.stops_recording(run.stage, run.frames as i32);
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
    running().reads_the_unlocks();
    0
}

/// `ResultScreen::AddedCallback`: the screen built, with the score file opened and read.
///
/// orb gets in front of this to empty what is in memory first, so that a ranking read defines the
/// history rather than adding to it — see `Game::forget_captures`.
///
/// **One callback for the two screens the class is**, which is why the state decides what the read is
/// parsed into: `ParseCatk`, `ParseClrd` and `ParsePscr` sit inside `resultScreenState !=
/// WRITING_HIGHSCORE_NAME && != EXIT` (compared at 0x42f4e5 and 0x42f4f1, jumped over at 0x42f4f7), so a
/// finished run's own record is what stands in memory when that screen goes down and writes. The open
/// happens either way, which is what a run's end shows in the log as a read of the mode's file.
extern "C" fn ranking_read(_screen: *mut c_void) -> i32 {
    let fake = running();
    match fake.image.result_screen_state() {
        result_state::WRITING_HIGHSCORE_NAME | result_state::EXIT => {
            fake.opens_the_score_file(false);
        }
        _ => fake.reads_the_score_file(),
    }
    0
}

/// The game's own `CreateFileA`, which the score file's fork calls through with the name it decided.
///
/// Writes down that name and the access, which is the whole of what crosses this call and the whole of what
/// an e2e test about that file reads back. The handle is an index into those, one-based so it is never the
/// null the game reads as a failure: there is no file on any disk here, and what an e2e test asks is which
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
    // open that happened, and which name it landed in is exactly what an e2e test reads back.
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

/// The record a replay of one of this game's stages holds: a change of what was held every
/// [`RECORDED_EVERY`] frames, cycling through the directions with the shot key down throughout.
///
/// This game's own — what an e2e test is about is that the same record played twice puts the player in the
/// same place to the last digit, not which buttons Reimu was pressing. Entries rather than a word per
/// frame because that is what a recording is: the frame each change happened on, and what was held from
/// then on.
const RECORDED_EVERY: i32 = 7;
const RECORDED_ENTRIES: usize = 512;
const RECORDED_DIRECTIONS: [u16; 4] = [button::LEFT, button::DOWN, button::RIGHT, button::UP];

/// And the seed each of those stages was drawn with, which the record carries too: a stage reached by
/// moving is seeded from the record and not from wherever the generator was left, which is the whole of
/// why two passes over one stage can agree at all.
const RECORDED_SEED: u16 = 0x1234;

fn recorded() -> Vec<(i32, u16)> {
    (0..RECORDED_ENTRIES)
        .map(|entry| {
            let direction = RECORDED_DIRECTIONS[entry % RECORDED_DIRECTIONS.len()];
            (entry as i32 * RECORDED_EVERY, direction | button::SHOOT)
        })
        .collect()
}

/// A stage's waves, in the only terms the boundary detector reads them: two hundred frames with
/// enemies on and two hundred without.
pub fn waves(script: i32) -> i32 {
    if (script / 200) % 2 == 0 { 3 } else { 0 }
}

/// The next number out of the generator.
///
/// A generator of this game's own, not 紅魔郷's: what an e2e test is about is that a stage seeded the
/// same way draws the same numbers, which any generator answers and which is what a resume rests on.
fn drawn_from(seed: u16) -> u16 {
    seed.wrapping_mul(0x9d5d).wrapping_add(0x6f7f)
}

/// Where the buttons of a frame leave the player, held inside the box the game holds them in.
///
/// [`PLAYER_AREA_TOP_LEFT`] and [`PLAYER_AREA_SIZE`] and not the arcade region's, which is what
/// `Player::HandlePlayerInputs` clamps `positionCenter` to.
fn moved((x, y): (f32, f32), word: u16) -> (f32, f32) {
    let moved = |at: f32, less: bool, more: bool, from: f32, span: f32| {
        let at = at - if less { SPEED } else { 0.0 } + if more { SPEED } else { 0.0 };
        at.clamp(from, from + span)
    };
    (
        moved(
            x,
            word & button::LEFT != 0,
            word & button::RIGHT != 0,
            PLAYER_AREA_TOP_LEFT.0,
            PLAYER_AREA_SIZE.0,
        ),
        moved(
            y,
            word & button::UP != 0,
            word & button::DOWN != 0,
            PLAYER_AREA_TOP_LEFT.1,
            PLAYER_AREA_SIZE.1,
        ),
    )
}
