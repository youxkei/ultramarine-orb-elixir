//! th06nc — 「東方紅魔郷: New Classic」, the other Steam release of 東方紅魔郷 — as a [`Game`].
//!
//! **The same engine th06c is built on, and not the same game.** Both exes look their host up by name
//! through one `GetProcAddress`, import the same nine names of `steam_api64.dll`, read the keyboard
//! through one `GetKeyboardState` translated through a table of the same shape, and walk the same two
//! `ChainElem` lists a frame — the host layer matches th06c's byte for byte in places, its keyboard read
//! being a function of the same `0x26f` bytes. What sits on top is another game: the front end has
//! thirty-two states where th06c's has twenty-one
//! and leaves the chain for the whole of a run, the bullet is a different size, and a hit is a circle
//! about the player where th06c's is a box. `th06nc/asm.md` is where what has been read off this exe is
//! written down, and nothing of `game::th06c` is read here.
//!
//! **Two seams and no dialog.** The keyboard read is answered by name, and there is no startup dialog to
//! click through: this exe imports no `DialogBoxIndirectParamW`, and what window it makes is
//! `th06.env`'s business — which orb has a word in on the first frame, see
//! [`size_the_window_to_the_settings`].
//!
//! **And the frame is orb's**, stood in over the function `main`'s loop calls once round, the way th06c's
//! is — see [`THE_FRAME`]. Off orb's frame a stage is read as well: [`sight`] answers
//! [`Game::what_a_bot_sees`] out of this game's own statics, so the rules play it.
//!
//! **And a run of it is chaptered**: [`Th06nc::rewinds`] answers true, [`GAME_MANAGER`] is the static
//! every number of a run is a field of, [`SUPERVISOR`] is what a run being started and given up goes
//! through, [`resume`] is the file one is picked up from and [`score_file`] this game's own file as
//! bytes. What the mode still declines is the ranking it shows — read, as the main menu's item 5, and not
//! driven — and what no reading settles yet: the card a fight is on, and the line one pass of a stage is
//! held against another's.
//!
//! **Neither seam is an address**, for the reason th06c's are not: this exe is built with ASLR on. The
//! addresses below are the reading of a run, and every one of them goes through [`at`], which adds the
//! distance [`Game::set_image_base`] was told.
//!
//! **And every call into this game is `extern "win64"`**, the same four registers th06c's are.
#![doc = include_str!("asm.md")]

use std::ffi::c_void;
use std::ops::Range;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering};

use orb_api::mem;

use crate::game::{
    Boundary, Building, Call, CountedDeath, Entered, FileOpen, FrameCalls, FrameEntered, Game,
    Hooks, Lookup, Memory, Menu, Pad, PanelTile, Patch, Read, Reading, Rect, RunStart, State,
    Threads, Walks,
};

pub mod chapters;
/// Reachable from this crate's own tests and, through the `sim` feature, from the tests of the crates that
/// drive it — which is where the scenarios live.
#[cfg(any(test, feature = "sim"))]
pub mod image;
pub mod resume;
pub mod score_file;
mod sight;

/// Where the exe asks to be loaded, which is what the addresses below are written against.
const IMAGE_BASE: usize = 0x1_4000_0000;

/// How far it turned out to be from that ask — see [`Game::set_image_base`], which writes it, and
/// [`at`], which is the only place it is read.
static SLIDE: AtomicUsize = AtomicUsize::new(0);

/// An address of this game's own where its exe actually is.
fn at(address: usize) -> usize {
    address.wrapping_add(SLIDE.load(Ordering::Relaxed))
}

/// `handed_over!` for a game whose addresses go through [`at`], which is th06c's macro of the same name
/// and for its reason: that macro's slot starts out holding the address itself, a constant there, where
/// here the address is the constant plus the slide the loader chose and no static initialiser can hold
/// it. So the slot starts at nought, nought reads as the game's own function at [`at`] of the constant,
/// and a laid-out game — which is code no address space of this game's can hold — writes its own function
/// over it. Outside a build with the simulator in it the slot does not exist and the read is [`at`] of the
/// constant.
macro_rules! handed_over_at {
    ($slot:ident, $at:ident, $set:ident, $real:ident) => {
        #[cfg(any(test, feature = "sim"))]
        static $slot: AtomicUsize = AtomicUsize::new(0);

        #[cfg(any(test, feature = "sim"))]
        fn $at() -> usize {
            match $slot.load(Ordering::Relaxed) {
                0 => at($real),
                handed => handed,
            }
        }

        #[cfg(not(any(test, feature = "sim")))]
        fn $at() -> usize {
            at($real)
        }

        #[cfg(any(test, feature = "sim"))]
        fn $set(address: usize) {
            $slot.store(address, Ordering::Relaxed);
        }
    };
}

// **Every call orb makes into this game's code, one slot apiece.** A laid-out th06nc fills them with
// functions of its own through [`image`]; a real launch reads each as the game's own address. The walk,
// the camera, the draw's own six, the frame's own end, and the three of the music's catch-up — the calls
// this game's frame makes, and so the calls orb's frame makes in its place.
handed_over_at!(
    RUN_CALC_CHAIN_AT,
    run_calc_chain,
    set_run_calc_chain,
    RUN_CALC_CHAIN
);
handed_over_at!(
    MOVES_THE_CAMERA_AT,
    moves_the_camera,
    set_moves_the_camera,
    MOVES_THE_CAMERA
);
handed_over_at!(
    BEGINS_THE_DRAW_AT,
    begins_the_draw,
    set_begins_the_draw,
    BEGINS_THE_DRAW
);
handed_over_at!(
    READS_THE_VIEWPORT_AT,
    reads_the_viewport,
    set_reads_the_viewport,
    READS_THE_VIEWPORT
);
handed_over_at!(
    SETS_THE_VIEWPORT_AT,
    sets_the_viewport,
    set_sets_the_viewport,
    SETS_THE_VIEWPORT
);
handed_over_at!(CLEARS_AT, clears, set_clears, CLEARS);
handed_over_at!(
    TAKES_AN_ELEMENT_OUT_AT,
    takes_an_element_out,
    set_takes_an_element_out,
    TAKES_AN_ELEMENT_OUT
);
handed_over_at!(
    FLUSHES_THE_SPRITES_AT,
    flushes_the_sprites,
    set_flushes_the_sprites,
    FLUSHES_THE_SPRITES
);
handed_over_at!(
    TAKES_THE_SCREENSHOT_AT,
    takes_the_screenshot,
    set_takes_the_screenshot,
    TAKES_THE_SCREENSHOT
);
handed_over_at!(
    ENDS_THE_FRAME_AT,
    ends_the_frame,
    set_ends_the_frame,
    ENDS_THE_FRAME
);
handed_over_at!(
    WHERE_THE_MUSIC_IS_AT,
    where_the_music_is,
    set_where_the_music_is,
    WHERE_THE_MUSIC_IS
);
handed_over_at!(
    HOW_LONG_THE_TRACK_IS_AT,
    how_long_the_track_is,
    set_how_long_the_track_is,
    HOW_LONG_THE_TRACK_IS
);
handed_over_at!(
    SEEKS_THE_MUSIC_AT,
    seeks_the_music,
    set_seeks_the_music,
    SEEKS_THE_MUSIC
);
// **And the three the window's own sizing makes**, which `Game::prepare_frame` reaches on a launch's
// first frame — see [`size_the_window_to_the_settings`]. Left out for one change, on the reading that a
// laid-out game makes its window through the host and never gets here; what that cost was an access
// violation on the first frame of every e2e test over this game, `0x14007e1b0` being an address no
// laid-out address space has anything at.
handed_over_at!(
    MAKES_A_WINDOW_AT,
    makes_a_window,
    set_makes_a_window,
    MAKES_A_WINDOW
);
handed_over_at!(
    COVERS_THE_MONITOR_AT,
    covers_the_monitor,
    set_covers_the_monitor,
    COVERS_THE_MONITOR
);
handed_over_at!(
    SIZES_THE_PICTURE_AT,
    sizes_the_picture,
    set_sizes_the_picture,
    SIZES_THE_PICTURE
);
handed_over_at!(RUN_START_AT, run_start, set_run_start, RUN_START);
handed_over_at!(MALLOC_AT, mallocs, set_mallocs, MALLOC);
handed_over_at!(FREE_AT, frees, set_frees, FREE);

/// **The DirectInput keyboard the game reads a run with**, which orb takes away — the same shape th06c
/// has: the read at `0x1400afef0` takes the device where there is one, `GetDeviceState` straight into
/// [`THE_KEYS_THE_GAME_READS`] at `0x1400affc8`, and the translation from virtual keys orb's answer feeds
/// is on the other branch, reached where `[0x1405a7a44]` or `[0x1405a7a84]` is 1 or this is nought.
const DEVICE: usize = 0x1_405a_7a70;

/// **The read the game asks for by name**, resolved with `GetProcAddress` at startup into the slot at
/// `0x1405a83a8` and called at `0x1400b002c` and `0x1400b0085` — see [`Read::LookedUp`].
const KEYBOARD_READ: &str = "GetKeyboardState";

/// **And the name it asks for to make its window**: `CreateWindowExW` is in `.rdata` at `0x1402f8740`
/// and in no import table.
const WINDOW_MADE: &str = "CreateWindowExW";

/// **And the name it reads a pad with**, which orb answers by being the pad — see
/// [`Game::press_into_pad`]. This exe walks `xinput1_4.dll`, `xinput1_3.dll` and `xinput9_1_0.dll`, the
/// three wide strings at `0x1402f71f0` onwards, the same as th06c.
const PAD_READ: &str = "XInputGetState";

/// **And the name it makes its renderer through**, at `0x140303820` in `.rdata` with the two older
/// names beside it — see [`crate::dxgi`], which takes the swapchain the factory then makes.
const RENDERER_MADE: &str = "CreateDXGIFactory2";

/// **And the name it moves that window with afterwards**, `0x1402f8840`.
const WINDOW_MOVED: &str = "MoveWindow";

/// **And the two it puts that window on the screen with**, both looked up at startup: a launch's log has
/// `the game looked ShowWindow up` and `the game looked SetWindowPos up` one millisecond apart, before
/// the window is made — which is why a window made without `WS_VISIBLE` was on the screen a frame later
/// anyway. `SetWindowPos` is also in this exe's `USER32` import table, so the lookup is a second way to
/// the same call and the one orb can answer.
const WINDOW_SHOWN: &str = "ShowWindow";
const WINDOW_PLACED: &str = "SetWindowPos";
const WINDOW_RESTYLED: &str = "SetWindowLongW";

/// **Which virtual key each of this game's buttons is pressed with** — the keys 東方紅魔郷 has always
/// been played with; nothing has been read out of `th06.cfg`.
mod key {
    pub const SHOT: u8 = 0x5a; // Z
    pub const BOMB: u8 = 0x58; // X
    pub const FOCUS: u8 = 0x10; // Shift
    pub const MENU: u8 = 0x1b; // Escape
    pub const DECIDE: u8 = 0x0d; // Enter
    pub const UP: u8 = 0x26;
    pub const DOWN: u8 = 0x28;
    pub const LEFT: u8 = 0x25;
    pub const RIGHT: u8 = 0x27;

    /// **And the focus again as this build's own table holds it**: `VK_LSHIFT`, which the table at
    /// `0x1402f6060` keeps at scan `0x2a` beside `VK_RSHIFT` at `0x36` — and it holds no [`FOCUS`] at
    /// all, as th06c's own at `0x1402bc0b0` holds none. See
    /// [`the_key_the_games_table_holds`](super::the_key_the_games_table_holds).
    pub const FOCUS_IN_THE_GAMES_TABLE: u8 = 0xa0;
}

/// **The bits orb's own word carries for this game**, the same shape th06 1.02h's word has so that a
/// recipe reads the same for every game — see [`Th06nc::press_into`].
///
/// **And this game's own word has the same bits for the five a run is played with**: the player's own
/// walk at `0x14006a790` onwards tests `0x10`, `0x20`, `0x40` and the sign bit for the four directions
/// and `0x4` for the focus at `0x14006a7c3`, and the fast-forward at `0x14003d92b` tests `0x8000` of
/// [`INPUT_THE_RUN_ACTS_ON`].
///
/// **The decide is not one of them, though**: the game's own is `0x100` — the front end writes that bit
/// into the word at `0x1400491cc` where a screen is to be skipped, and `Enter`'s press gives it alone at
/// `0x1400136e0`, `Z`'s and `J`'s inside their `0x8101` — where [`DECIDE`] below is `0x1000`. Nothing needs the two to agree: what orb does with
/// its own word is press keys with it — see [`Th06nc::press_into`], which walks
/// [`Th06nc::BUTTONS`] — and the game builds its own out of those keys. A bit of orb's compared against
/// the game's word would be the thing this note exists to stop.
mod button {
    pub const SHOT: u16 = 0x0001;
    pub const BOMB: u16 = 0x0002;
    pub const FOCUS: u16 = 0x0004;
    pub const MENU: u16 = 0x0008;
    pub const UP: u16 = 0x0010;
    pub const DOWN: u16 = 0x0020;
    pub const LEFT: u16 = 0x0040;
    pub const RIGHT: u16 = 0x0080;
    pub const DECIDE: u16 = 0x1000;
}

/// **The word the game's own read leaves behind, which a run is played with**, and `0x140abae84` is the
/// frame before it: the front end's state 0 clears both at `0x140049234` as a run starts, the player
/// reads it at `0x14006a0fd` and five more places, and the frame's own fast-forward reads `0x8000` of it
/// at `0x14003d92b`.
const INPUT_THE_RUN_ACTS_ON: usize = 0x1_40ab_ae80;

/// **The same word as the frame before left it**, which every screen of the front end edge-checks its own
/// buttons against — `0x14007b8ac` copies one into the other in the supervisor's own job, before
/// `0x14007b8b7` writes the keyboard's new answer, see [`Game::swallow_input`]'s own note about that copy.
const THE_WORD_THE_FRAME_BEFORE: usize = 0x1_40ab_ae84;

/// **The one bit of that word the game reads as a level rather than as an edge**: `0x14003d92b` tests
/// `0x8000` of it inside the frame loop and walks the calc chain up to seven more times while
/// `[0x14053d3dc]` says a replay is playing, `[0x14053d400]` is clear and the state asked for is a
/// run's — each extra walk counted into `[0x1405542e0]`. Which is what [`Game::swallow_input`] has to
/// leave alone: every other bit of a filled word is a bit that cannot be an edge, and this one is a
/// frame played eight times.
const THE_FAST_FORWARD: u32 = 0x8000;

/// **The byte that says everything is held**, which the bullet manager's update tests first at
/// `0x140010faf` and jumps past the whole of itself on, and the player's at `0x140069f3d`, the enemies'
/// at `0x140039a7c` — seven readers in all, and one writer at `0x140034d3d`.
const EVERYTHING_HELD: usize = 0x1_4053_d409;

/// **The calc chain's head**, a static `ChainElem` the walk at [`RUN_CALC_CHAIN`] starts from and the
/// take-out at `0x140012e26` searches first; the draw chain's is [`THE_DRAW_CHAIN`], walked inline in the
/// frame at `0x14003da73`.
const THE_CALC_CHAIN: usize = 0x1_4053_c090;

/// The fields of a `ChainElem` a walk reads — the same `0x40` bytes th06c's is, read off the
/// registrations at `0x14006952e` and `0x1400573bd`.
mod chain {
    /// **The callback a frame runs**, called with [`ON`] in `rcx`.
    pub const CALLBACK: usize = 0x8;
    /// The link a walk follows.
    pub const NEXT: usize = 0x28;
    /// What that callback is called on.
    pub const ON: usize = 0x38;
}

/// Longer than any chain this game builds, so that a chain caught in a state whose links do not end is a
/// walk that answers nothing rather than one that does not return.
const CHAIN_LINKS: usize = 64;

/// **One frame of a run as the numbers a restore has to reproduce** — see
/// [`Game::reproduction`](Th06nc), which is what says which of them parted.
struct Reproduction {
    /// The word the update acted on, in the sixteen bits orb reads of it. **The whole low word and not
    /// the bits the game's own record keeps**, which is a reading this game has not got: th06c masks
    /// with `0x3ff` off `0x14004eecf`, and where this build masks is not established.
    input: u16,
    player: (f32, f32),
    /// The top and the height of the box the player is held inside.
    player_area: (f32, f32),
    /// How many numbers have come out of the generator since the stage seeded it.
    randoms: u32,
    /// The generator itself, which is the one number a wrong stage start shows in.
    seed: u16,
    /// Items in the air, which nothing about a stage's start puts back.
    items: u32,
    /// The score as shown, which is what an extra life is measured against.
    score: u64,
    extra_lives: i8,
    /// **How many enemies are alive**, out of the manager's own count — see [`enemy_manager::LIVE`].
    ///
    /// **Here because the fields before it left the enemies as the only place two passes could differ.**
    /// A pair of real launches of one chapter agreed on every one of them — the player, the play field,
    /// the items, the score, the three fingerprints of the bullet array, the script's clock and its
    /// cursor — through the frame before the generator parted, and the draws that parted are a spawn's.
    /// And the generator moves nowhere else: measured with a line either side of the drawing and another
    /// across the frames of `runtime::render`, a stage played out drew **nothing** in the drawing half
    /// and nothing outside orb's own frame.
    enemies: i32,
    /// **Where the stage's script has got to**, as the cursor's own distance from the timeline it points
    /// into — see [`enemy_manager::TIMELINE_CURSOR`] and [`THE_STAGES_TIMELINE`]. **A distance and not
    /// the cursor**, which is an address and a different one every launch, and `-1` where either is
    /// nought, which is every frame before a stage's script has one.
    ///
    /// **Here because two passes of one stage parted in a spawn and in nothing else**: a pair of real
    /// launches of a chapter of stage 3 agreed on every other field of every frame to 217 and then drew
    /// the same group of five numbers an update apart, with the script's own *clock* equal across it —
    /// and a spawn fires where that clock equals the time of the entry this points at, so the entry is
    /// the only thing left that the two could disagree about. See this change's description, whose `TODO`
    /// about a playback that does not reproduce the run is the whole of that measurement.
    timeline_cursor: i64,
    /// **The phase of the shot the player is firing, and the phase the update before it had** — see
    /// [`player::SHOT_PHASE`].
    ///
    /// **Here because the field before it left the player's shots as the only place two passes could
    /// differ.** The pair of real launches that took the enemies' own fingerprint had the same enemies in
    /// the same places with different life — the run's shots taking one down two frames before the
    /// playback's — and the player's shots are in neither array the line folds. What decides when one is
    /// emitted is this pair, so a playback whose cycle stands where the run's does not says so here on
    /// the frame it happens.
    shot_phase: (i32, i32),
}

impl std::fmt::Display for Reproduction {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            out,
            "input={:#06x} player={:.2},{:.2} area={:.2}+{:.2} randoms={} rng={:#06x} items={} \
             score={} extras={} enemies={} timeline={} shot={},{}",
            self.input,
            self.player.0,
            self.player.1,
            self.player_area.0,
            self.player_area.1,
            self.randoms,
            self.seed,
            self.items,
            self.score,
            self.extra_lives,
            self.enemies,
            self.timeline_cursor,
            self.shot_phase.0,
            self.shot_phase.1,
        )
    }
}

/// **The front end's frame callback**, which is the way to the front end: it is allocated rather than
/// static and **is taken out of the chain as a run starts** — a launch's `chain:` lines went from four
/// jobs with this one among them to three without it, and then to the stage's eleven — so the element
/// `0x140057411` keeps at `0x140c53468` points at freed memory for the whole of a run. What holds is the
/// chain itself: the element whose [`chain::CALLBACK`] is this has the front end at its [`chain::ON`].
/// The wrapper calls `0x140049140` once — or up to six hundred times where `[front end + 0x168d0]` says
/// to skip.
///
/// **The front end and not the game manager**, which is what it was first taken for: its thirty-two
/// states are this game's *screens* — the logo, the title, the menu, the difficulty, the character, the
/// shot — and none of them is a stage. What a run is made of is [`GAME_MANAGER`], a static that stays
/// where it is, which is th06c's split too.
const THE_FRONT_ENDS_CALLBACK: usize = 0x1_4004_90a0;

/// **The player's frame callback**, registered at `0x14006952e`, which is what says a stage is running:
/// the stage's own jobs — this, the enemies' `0x140038cd0`, the bullets' `0x140010fa0` and eight more —
/// arrive together as the front end leaves, and go together as it comes back.
const THE_PLAYERS_CALLBACK: usize = 0x1_4006_9f30;

/// **The job an ending runs as**, which is what says one is running: the frame the sixth stage's jobs
/// left, one launch's `chain:` line went from the stage's eleven to five — the three that are always
/// there, `0x14006d070` of the stage's, and this one, registered at `0x14002c2c6`. It reads the input
/// word at `0x14002c4a1` the way a screen that is pressed through does, and the front end is not back
/// while it runs, so [`Game::read_state`] answers `State::in_ending` by asking whether the chain holds it.
const ENDING_UPDATE: usize = 0x1_4002_c450;

/// **The job the screen a run ends at registers**, which is the one way to that screen: it is built on
/// the heap — `0x14007bd98` allocates its `0xbb48` bytes, `0x14006f5d0` builds it — and `0x14006f3f0`
/// puts it at the [`chain::ON`] of the element it registers with this callback, the same way th06c's is.
/// It arrived on the frame the ending's job left, and the front end came back after it.
const RESULT_SCREEN_UPDATE: usize = 0x1_4006_fa70;

mod result_screen {
    /// **The screen's own state**, which its update reads at `0x14006faa8` and jumps through a table of
    /// eighteen at `0x140073ef4` on — eight handlers, four of them shared, which `th06nc/asm.md`'s *The
    /// entry a name is typed into* lists with the script that dumps them again.
    pub const STATE: usize = 0x9e94;

    /// **The one of those eighteen the name is typed in.** `0x14007bdde` writes it as a run's own screen
    /// is built — nine where `[0x14053d404]` is clear and seventeen where it is not, which is the run that
    /// is asked for no name.
    ///
    /// **And twelve is the list of replay slots**, which `0x14000cdc9` writes as the machine at
    /// `0x14000c960` — the menu a scene's end puts up — builds that screen for its item 1, the replay's
    /// save: a scene's own path and no item of the main menu's, see [`Game::show_ranking`].
    pub const TYPING_A_NAME: i32 = 9;

    /// **The field the entry types into**, eight bytes: `0x1400726f1` writes the character at
    /// `[this + 0x9e70 + index]`, the index being [`TYPED`] capped at seven by the `cmovge` at
    /// `0x1400726cd`. The same shape th06c has at `+0x5173`.
    pub const NAME: usize = 0x9e70;
    pub const NAME_BYTES: usize = 8;

    /// **How many characters have been typed**, which `0x140072703` steps up and `0x1400726fe` stops at
    /// eight — and reaching eight is what `0x140072710` puts the cursor on [`THE_END_ITEM`] for.
    pub const TYPED: usize = 0x9e7c;

    /// **Which character of the grid the cursor is on**, the first dword of the screen's own object:
    /// `0x1400726d0` reads it, `0x1400726d8` takes the character at `0x140351f40 + it` — the grid of 96,
    /// sixteen to a row, reached off the image base in `r13` — and `0x1400726e9` writes a space where it
    /// is `0x5e`.
    pub const CURSOR: usize = 0x0;

    /// **The screen's own copy of the difficulty**, which `0x140071ecc` writes from `[0x14053d410]`
    /// inside state 9's setup — behind `0x140071e9c`'s test of `[+0x4450]` — and `0x1400722a2` copies
    /// into the `HSCR` record it builds, at that record's `+0x15`.
    pub const THE_DIFFICULTY: usize = 0xa8;

    /// **The item that ends the entry**, the one past the space at `0x5e`: `0x1400726e3` leaves the typing
    /// for it, and it is what `0x140072710` puts the cursor on once eight characters have been typed.
    pub const THE_END_ITEM: i32 = 0x5f;

    /// **The state the screen asks whether to save a replay in**, which is the one a run of orb's is
    /// taken past — a replay of a rewound run plays back as a run that dies where this one carried on.
    ///
    /// **A picture says so.** `0x1400727d5` writes fifteen here once the name entry's end item has
    /// copied the eight bytes out, and a launch that held the screen there — an Extra run stood still
    /// until it was out of lives, the one run this game asks no continue about — photographed
    /// **「今回のリプレイを保存しますか？」** over `はい` and `いいえ`. The same launch walked on with a
    /// decide each: `9, 15, 16, 10, 12, 14`, and the question was still drawn at ten.
    ///
    /// **And twelve is which slot to save into**, photographed on a later launch: fifteen rows under
    /// `No.`, `Name`, `Date`, `Player`, `Mode` and `Detail`, one of them holding this copy's own
    /// replay. So the three the walk passes are whether to save, which slot, and — where the slot is
    /// taken — this screen's own fourteen.
    ///
    /// **And the answer is taken later than this**, at fourteen, whose decide with the cursor at nought
    /// reaches the state that writes `replay/th6_%.2d.rpy` — see `th06nc/asm.md`'s *The entry a name is
    /// typed into*. Fifteen is caught rather than fourteen because it is the state the entry hands on
    /// to, so the screen is taken off the offer before the question can be answered at all.
    pub const SAVE_REPLAY_QUESTION: i32 = 15;

    /// **And the state the screen leaves in**: `0x14006fb0e` fades the object at `[0x140c6e378]` over
    /// thirty frames and then writes **1** — the front end — into the supervisor's wanted state at
    /// `0x140c6dfbc`. The same number th06c's own way out is, and the state `0x14007bdde` opens the
    /// screen in for a run asked for no name, which is a run that leaves without being asked anything.
    ///
    /// **And not fifteen, which this was.** That reading took the shared handler of fifteen and sixteen
    /// for a way out; what the picture above shows is that fifteen is the offer, so writing it here put
    /// the screen *onto* the question instead of taking it off.
    pub const EXIT: i32 = 17;

    /// **Whether the screen is a run's own**, the byte `0x14007bdc3` writes 1 into as the supervisor
    /// builds it for [`super::STATE_THE_RUNS_OWN_END`]. The ranking browser's build — the supervisor's 6,
    /// which the main menu's item 5 asks for — leaves it clear, and the menu a scene's end puts up sets
    /// `+0x9e9a` at `0x14000cdc2` instead — see [`super::Th06nc::show_ranking`] — so this is what tells a
    /// run that finished from a ranking put up over one.
    pub const A_RUNS_OWN: usize = 0x9e99;
}

/// Whether the ending was running the last time [`Game::read_state`] looked, so that its arrival is one
/// line.
static ENDING: AtomicBool = AtomicBool::new(false);

/// **How many times a run has been hit, counted by orb rather than read off the game.**
///
/// **The game keeps a count of its own at `0x14053cab0`**, the game manager's first dword: `0x14006a22a`
/// steps it up on every death, whether lives are left or `0x14006a581` puts the run in the retry menu,
/// and in a Challenge run — where a death takes no life — the 1up steps it down instead at
/// `0x140044cbf`. This one is orb's, counted off the player's own state — th06c's answer for an exe that
/// keeps none at all.
static DEATHS: AtomicI32 = AtomicI32::new(0);
/// Whether the player's state was [`player::HIT`] on the frame before, which is what makes the count
/// above move once per hit rather than once per frame of one.
static WAS_HIT: AtomicBool = AtomicBool::new(false);

/// What [`Game::menu_pointed_at`] saw: whether the front end was in the chain, and its state and its
/// cursor where it was.
type MenuSeen = (bool, Option<i32>, Option<i32>);

/// **The two words the front end's own screens read**, said once each time either changes while the
/// front end is in the chain — [`INPUT_THE_RUN_ACTS_ON`] and the frame before it, which every one of
/// those screens edge-checks its buttons against.
///
/// **The instrument for a screen whose press orb cannot see arriving.** What it was added for is the
/// screen a run's end leaves the front end on — 18, the replay list — which three launches could not get
/// off: the decide there starts a replay of the game's own and the cancel is read at `0x14004a5d7` as
/// `0x200` against `0x140abae84`, and a launch that tapped the bomb every twelve frames for 1,200 of
/// them moved nothing. Whether the bit arrives at all, and whether it arrives as an *edge*, is what these
/// two words say and nothing else does.
static THE_WORDS_LAST_SEEN: std::sync::Mutex<Option<(u32, u32)>> = std::sync::Mutex::new(None);

/// The two words above, said once each time either changes.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn say_the_words_the_front_end_reads_when_they_change() {
    if unsafe { the_front_end() }.is_none() {
        *THE_WORDS_LAST_SEEN.lock().unwrap() = None;
        return;
    }
    // Both raw, for the reason [`Th06nc::the_worlds_fingerprint`] gives: each is a static of the exe's
    // own image and committed from the moment the module is mapped, so a committed read of one is a
    // `VirtualQuery` about an address that can never be anywhere else.
    let seen = (
        unsafe { mem::read::<u32>(at(INPUT_THE_RUN_ACTS_ON)) },
        unsafe { mem::read::<u32>(at(THE_WORD_THE_FRAME_BEFORE)) },
    );
    if THE_WORDS_LAST_SEEN.lock().unwrap().replace(seen) != Some(seen) {
        crate::detail!(
            "front end: the word is {:#06x} and the frame before it {:#06x}",
            seen.0,
            seen.1,
        );
    }
}

/// And what it last saw, so that its own line is said once each time any of the three changes rather
/// than once a frame. See that method, whose doc has the launches this is the instrument for.
static MENU_LAST_SEEN: std::sync::Mutex<Option<MenuSeen>> = std::sync::Mutex::new(None);

/// **Which state the screen a run ends at was last in**, `None` inside where the screen is down — the
/// instrument the eighteen states of that screen are read with. A picture of this game is taken from
/// outside, orb having no back buffer to draw one from, so what a picture cannot say is *which* state it
/// caught: this line says that, and the two together are how a state's own screen is read. See
/// `th06nc/asm.md`'s *The entry a name is typed into*, whose table it fills in.
static THE_END_LAST_SEEN: std::sync::Mutex<Option<Option<i32>>> = std::sync::Mutex::new(None);

/// The line above, said once each time the state changes rather than once a frame.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn say_the_screen_a_run_ends_at_when_it_changes() {
    let screen = unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) };
    let state = screen.and_then(|screen| mem::read_committed::<i32>(screen + result_screen::STATE));
    if THE_END_LAST_SEEN.lock().unwrap().replace(state) != Some(state) {
        match state {
            // **And the two numbers a record's own difficulty comes out of, beside the state** — the
            // run's `DIFFICULTY` and the screen's own copy of it, which `0x140071ecc` writes inside
            // state 9's setup and `0x1400722a2` puts into the record it builds. An Extra run's record
            // came out with 1 where the run had played on 4, and these two said at which of the two
            // moments that happened — see `th06nc/asm.md`'s *What is not established*.
            Some(state) => crate::detail!(
                "result: the screen a run ends at is in state {state}, difficulty {} and its own {}",
                unsafe { mem::read::<i32>(at(DIFFICULTY)) },
                screen
                    .and_then(|screen| mem::read_committed::<i32>(
                        screen + result_screen::THE_DIFFICULTY
                    ))
                    .unwrap_or(-1),
            ),
            None => crate::detail!("result: the screen a run ends at is down"),
        }
    }
}

/// **The count and the grace `Game::menu_takes_a_press` last read**, for the other half of the hold: the
/// screen the count is of is `MENU_LAST_SEEN`'s line, and what this adds is the two numbers that answer
/// turns on.
static TAKES_A_PRESS_LAST_SEEN: std::sync::Mutex<Option<(i32, i32)>> = std::sync::Mutex::new(None);

/// How many frames running a boss has been up for, which paces the line that says how its fight is going.
static FRAMES_ON_A_BOSS: AtomicU32 = AtomicU32::new(0);
/// And the life it last had, for the line that says how it left.
static THE_BOSSES_LAST_LIFE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

mod front_end {
    /// **Which of thirty-two states it is in**: `0x140049178` reads it and `0x140049210` jumps through
    /// the table of thirty-two at `0x14004e848`, of which fifteen to twenty and twenty-six share one
    /// case. Which is which screen is `th06nc/asm.md`'s table, read off launches.
    pub const STATE: usize = 0x168b0;
    /// **Which item of the front end the cursor is on**: the difficulty's own state compares it with 0,
    /// 1, 2 and 3 at `0x14004c872`–`0x14004c934`, one per difficulty, and lights the item it names. The
    /// same offset th06c keeps its cursor at.
    ///
    /// **And not `+0x30`**, which every state writes nought into as it comes up and which a launch read
    /// climbing once a frame — a counter of the state's own frames, and what the difficulty state hands
    /// `0x140049040` to blink the lit item with.
    pub const CURSOR: usize = 0x28;

    /// **The state the main menu is**, whose ten items orb has a question over one of — see
    /// `th06nc/asm.md`'s table of the screens, read off launches: 2 is the title and a decide there
    /// opens this.
    pub const MENU: i32 = 3;

    /// **And the three items of it that start a run**: 0 a run, 1 the Extra — skipped while it is locked,
    /// a fresh launch's taps of down going 0, 2, 3 … 9 — and **2 a stage practised on its own**, which a
    /// picture of the screen after it settled: `PRACTICE` across the difficulty select. What each of the
    /// other seven does is the table at `0x14004e8c8`, which `th06nc/asm.md` walks: 4 reaches the replay
    /// list through states 15 and 16, 6 the music room, and 3, 5, 7 and 8 are read only as far as the
    /// state each asks for.
    ///
    /// **The practice is one of the three because it is a run**, and a run is what the mode is a question
    /// about: without it a launch's practice would start in whatever mode the launch came up in, which is
    /// nobody's answer. `orb_real`'s `a_stage_practised_on_its_own` is the case that wants it.
    pub const ITEM_RUN: i32 = 0;
    pub const ITEM_EXTRA: i32 = 1;
    pub const ITEM_PRACTICE: i32 = 2;

    /// **And the item the replays are behind, which orb's own list stands over for a pointdevice run** —
    /// the game recording none that such a run would fit. Item 4's own case at `0x140049d8d` fills the
    /// screen's table and `0x140049db0` writes `0xf` into the state; state 15's case writes 16, and 16
    /// is the list — `0x14004a042` formats `replay/th6_%.2d.rpy` sixty times into it, the loop stepping
    /// by `0x78` while it is under `0x1c20`. See `th06nc/asm.md`'s *The second table, and the replay
    /// list behind it*.
    ///
    /// **Which is why orb asks over the item and not over that screen**: the press that would open it is
    /// held back, so for a pointdevice run the game never reaches its own list at all.
    pub const ITEM_REPLAYS: i32 = 4;

    /// **The byte [`ITEM_PRACTICE`] answers the practice with**, which [`super::RUN_START`] copies into
    /// [`super::A_RUN_IS_PRACTICE`] and the screen after the item picks its banner by: item 2's
    /// `0x140049cbc` writes 1 here and item 0's `0x140049af2` clears it, so the front end carries the
    /// answer across the four screens between the menu and the run's own start. What
    /// [`super::Th06nc::run_pointed_at`] reads, a run being decided a screen before that start.
    ///
    /// **What the Extra's item does with it is not established** — the writes read off item 1 are the
    /// clear records it leaves on and nothing of this byte — so the game laid out in [`super::image`]
    /// leaves it as it finds it there.
    pub const THE_RUN_IS_PRACTICE: usize = 0x168d1;
    /// **The byte beside it that says the front end is skipping**, which the wrapper calls the front end's
    /// update up to six hundred times a frame under, and on which `0x14004918c` lets four of its screens
    /// press themselves through (`0x1400491af`).
    #[allow(
        dead_code,
        reason = "read by `image`, which is behind `feature = \"sim\"`"
    )]
    pub const IS_SKIPPING: usize = 0x168d0;

    /// **And the title**, the screen a decide opens the menu from — one of the two a run's count of
    /// deaths is put back to nought on, see `super::DEATHS`.
    pub const TITLE: i32 = 2;

    /// **The last screen before a stage that takes a press**, which is where the run about to start is
    /// fully decided and where the question about picking one up stands — see `Th06nc::run_pointed_at`.
    /// Its case is `0x14004d981`, and what it does after its grace is the ordinary edge check:
    /// `0x14004d9d7` tests the decide's own bits of `0x140abae80` against `0x140abae84`'s, and
    /// `0x14004d9bf` the cancel's `0x200` beside it.
    ///
    /// **And 25 is not it.** That screen's case, `0x14004dbe9`, reads no word at all: it calls the run's
    /// own start as soon as `[0x140c6e378]`'s `+0x34` is 2 (`0x14004dbf9`), so a press at it is a press
    /// at a screen already on its way out — which is what `asm.md` had written down as a screen that
    /// takes one, off launches whose presses landed there anyway.
    pub const THE_LAST_SCREEN_THAT_TAKES_A_PRESS: i32 = 24;

    /// **How many frames that screen ignores its own decide for**: `0x14004d9a3` compares
    /// [`SCREEN_FRAMES`] with `0xf` and leaves the case for anything under it — fifteen where the main
    /// menu's own is twenty.
    pub const THE_LAST_SCREENS_GRACE_FRAMES: i32 = 0xf;

    /// **The frames a screen of the front end has been up**, which is the counter the menu's own case
    /// holds its grace against — and not `+0x30`, which is another of the same shape.
    pub const SCREEN_FRAMES: usize = 0x9638;

    /// **How many frames the main menu ignores its own decide for**: `0x140049a8d` compares
    /// [`SCREEN_FRAMES`] with `0x14` and leaves the case for anything under it, so the edge check at
    /// `0x140049aa5` — the game's own `0x100` against the frame before's — is not reached for the
    /// screen's first twenty frames.
    ///
    /// **Which is what orb's question needs**, and what five launches without it showed: the hold is
    /// stored after the game's update, so a screen orb answers `menu_takes_a_press` true for on its
    /// first frame is a screen whose decide the game has already read. See
    /// [`Game::menu_takes_a_press`](super::Th06nc).
    pub const MENU_GRACE_FRAMES: i32 = 0x14;
}

/// **The supervisor**, the game's own object at `0x140c6db90` — the `+0x38` of the element
/// `0x14007c070` registers from `main` at `0x14004754d`, whose callback `0x14007b7f0` is the job that
/// drives everything: the front end built, the stage's chain registered, the screen a run ends at
/// allocated. The same object `th06.cfg`'s 44 bytes sit at `+0x434` of.
const SUPERVISOR: usize = 0x1_40c6_db90;

mod supervisor {
    /// **The state the job has acted on**, which its own twelve cases are jumped through on —
    /// `0x14007b8fa` reads it and `0x14007b915` refuses anything past `0xb`.
    pub const STATE: usize = 0x428;
    /// **And the state wanted**, which each of those cases then switches on: `0x14007b901` reads it, and
    /// `0x14007baf1` — the exit every case falls to — writes it over [`STATE`], which is what says which
    /// of the two is which.
    ///
    /// And `+0x430` is the state acted on before this one, written [`STATE`] at `0x14007b90f`, which
    /// nothing here reads.
    pub const WANTED: usize = 0x42c;
    /// **`th06.cfg` itself**, whose pad bindings — five `i16` `wButtons` bit numbers for the shot, the bomb,
    /// the focus, the conversation skip and the pause, `0x140429210`'s `12, 13, 9, 14, 4` by default — are
    /// its first ten bytes, and whose [`DECIDE_SWAPPED`] swaps the menus' decide and cancel: `0x140012f00`
    /// reads them at `0x140c6dfc4` to `0x140c6dfcc` and `0x140c6dfda`.
    pub const CONFIG_PAD: usize = 0x434;
    /// And that byte, `+0x16` of the file — th06c's own file keeps it at `+0x15`.
    pub const DECIDE_SWAPPED: usize = 0x16;
}

/// **The state a run is.** The state the front end is what is running is 1 — the job's own case for
/// state nought, the outer table's first entry at `0x14007b9b3`, writes it into
/// [`supervisor::WANTED`] and calls `0x140057190`, which builds the front end with `0x1400573bd`'s
/// registration inside it.
///
/// **And this is two**, the same number th06c's run is: the front end's own case indexes its inner
/// table by [`supervisor::WANTED`] less two (`0x14007b931`), and that table's first entry is
/// `0x14007b94c` — `0x14003c0a0`, the stage's chain. So wanting two from the front end is what starts a
/// stage.
const STATE_RUN: i32 = 2;

/// **And three is the next stage of a run**, which is what the stage's build reads at `0x14003c27c` to
/// take its stage-following path: the `0x140c6dfbc` everything else about this game was written down
/// against is [`supervisor::WANTED`] itself, `0x140c6db90 + 0x42c`. The case for wanting it out of
/// [`STATE_RUN`] is `0x14007baaf`, which takes the stage's two chain elements out through `0x140012e10`
/// and calls `0x14003c0a0` to register them again — so the stage that arrives is the next one, the build
/// reading this same field for which path to take.
const STATE_NEXT_STAGE: i32 = 3;

/// **The state the screen a run ends at is up in**, which is the state a run *finishes* into where
/// [`STATE_RUN_OVER`] is the state one is given up into: `0x14007b9e3` is the case for wanting it out of
/// [`STATE_RUN`], and `0x14007bd81`'s third branch the one for wanting it out of state 10. Both wait for
/// `[0x140c6e378]`'s `+0x34` to read 2 — the fade — take the stage's two chain elements out through
/// `0x140012e10`, take `0xbb48` bytes off `malloc`, build the screen with `0x14006f5d0`, and then share
/// the tail at `0x14007bdc3`: 1 into [`result_screen::A_RUNS_OWN`] and, by `[0x14053d404]`,
/// [`result_screen::TYPING_A_NAME`] or the way out at seventeen into [`result_screen::STATE`].
///
/// **Which is why a run given up reaches no screen and a run finished does**, and the whole of why
/// [`Game::run_finished`] can tell them apart: state 8's own case builds nothing.
///
/// **Read by the laid-out game and not by orb**, which is what the allow is for: `image` is behind
/// `feature = "sim"`, so a build of this crate without it is a build where this constant has no reader —
/// and `Game::run_finished` asks `+0x9e99` rather than the supervisor, that being what tells a run's own
/// screen from a ranking somebody asked for. Deleting it would take the reading with it.
#[allow(
    dead_code,
    reason = "read by `image`, which is behind `feature = \"sim\"`"
)]
const STATE_THE_RUNS_OWN_END: i32 = 7;

/// **The state a run is given up into**: the case for wanting it out of [`STATE_RUN`] is `0x14007bb13`,
/// which takes the stage's two chain elements out through `0x140012e10`, hands the replay being
/// recorded to `0x14006db10` where `[0x140abae58]` holds one, clears the two replay flags, and asks for
/// the front end itself.
///
/// **And that call writes nothing**, which is worth writing down where it would otherwise read as the
/// save: `0x14007bb40`'s `xor edx,edx` passes a null path, and `0x14006db4f` sends a null one to
/// `0x14006e058` — the teardown that frees the recorder's seven stage records and then the recorder —
/// instead of the write at `0x14006e038`. Nor does anything else of this case write `score.dat`. See
/// [`Th06nc::writes_its_own_records`].
///
/// **And wanting 1 out of 2 is the other way out of a run**, `0x14007bae1`: its `0x14007b770` takes the
/// stage's elements out the same way, and writes `score.dat` through `0x140076370` at `0x14007b782`
/// unless a replay is playing before it builds the front end.
///
/// **Which of the two the game gives a run up by turns on the replay**: the pause menu's own give-up
/// writes 8 only where [`A_REPLAY_IS_PLAYING`] is set (`0x14000b482`) and 1 otherwise (`0x14000b48e`), and
/// the continue question's give-up writes 8 for a replay too (`0x14000bccc`). So 8 is a replay given up,
/// and a run of somebody's own is given up into [`STATE_FRONT_END`].
const STATE_RUN_OVER: i32 = 8;

/// **The front end asked for out of a run**, `0x14007bae1`'s case, which writes `score.dat` on the way —
/// see [`STATE_RUN_OVER`] for the way out that does not.
const STATE_FRONT_END: i32 = 1;

/// **The game manager**, the static at `0x14053cab0` that a run is made of: it is the `+0x38` of both
/// chain elements the stage's build registers — the calc element at `0x14053ca70` and the draw element
/// at `0x14053ca30`, see [`THE_STAGES_CHAIN`] — and every number `th06nc/asm.md` had written down as a
/// static of its own is a field of it.
///
/// **A static and not an allocation**, which is what makes it readable with no chain walk: the front end
/// is the allocated one — see [`THE_FRONT_ENDS_CALLBACK`].
const GAME_MANAGER: usize = 0x1_4053_cab0;

mod game_manager {
    /// **The character and the shot**, one byte each: `0x14004ef15` and `0x14004ef29` put a replay
    /// header's `character * 2 + shot` back into them, and the stage's own build reads the pair at
    /// `0x14003c4a6` to pick the ranking record a run is held against.
    pub const CHARACTER: usize = 0x20;
    pub const SHOT: usize = 0x21;
    /// **The stage, counted from one** — `0x14003e6ac` multiplies it by a thousand into
    /// `Stage * 1000 =  %5d`.
    pub const STAGE: usize = 0x24;
    /// **The power**, a word the stage's build starts at nought — or at eight where `[0x14053d414]` is
    /// set, `0x14003c334`.
    pub const POWER: usize = 0x28;
    /// **The score the run carries**, a qword: what the panel draws, what the extend check at
    /// `0x14003c9c0` holds against the thresholds in the table at `0x14034e0c8`, what a chapter of the
    /// mode records and what the replay's write puts into its header's `+0x28` at `0x14006dcbf`. The
    /// ranking entry the screen a run ends at builds takes the other one, `+0x938`, at `0x1400722b5`.
    ///
    /// **Which of the manager's two score fields this is was measured and not read.** The qword at
    /// `+0x938` is the one seventeen rip-relative adds land on — `0x1f4` for a point item at
    /// `0x140011967`, `0x3e8` at `0x140044e46`, an enemy's own at `0x140039404` — and both
    /// `0x14003c794` and `0x14004143f` copy it into this one, so the disassembly reads as though
    /// `+0x938` were the run's own and this a snapshot of it. Two launches of
    /// `orb_real`'s `a_chapter_of_the_stage_after_the_first` say otherwise, `Th06nc::numbers` having said
    /// both fields from the frame a chapter is written on: at the second stage's own chapter `+0x938`
    /// read nought and this read 2,555,010 on one launch and 3,188,720 on the other, and at the first
    /// stage's — nine frames in, nothing scored — both read nought. **A field that is nought across a
    /// boundary is not the field a run's score is in**, whatever the adds say: `+0x938` is the score the
    /// run counts, which the stage's build copies over this one at `0x14003c794` and then zeroes in its
    /// shared tail at `0x14003ced2`, and which the stage's own job puts back out of this one at
    /// `0x14003beef` on its first frame — see `th06nc/asm.md`'s *The stage's two seams*.
    ///
    /// **What the wrong one cost is a chapter with no score in it**: recorded off `+0x938`, a chapter of
    /// a run at 2,555,010 was written down as nought and put back as nought, which is the one field of
    /// `resume: the landing is not the frame written down: score=2555010 against score=0`.
    pub const SCORE: usize = 0x930;
    /// **The two menus a run stops for**: the pause menu the menu button opens at `0x14003bb75`, and the
    /// menu a run out of lives is asked to continue from, which the player's death sets at
    /// `0x14006a581`. Bytes, and a frame with either set is a frame nothing of the run moved in — which
    /// is what the frame's own fast-forward refuses at `0x14003d919`.
    pub const IS_IN_GAME_MENU: usize = 0x950;
    pub const IS_IN_RETRY_MENU: usize = 0x951;
    /// **The point items collected**, a word the item walk's point case steps up at `0x140044a21`.
    pub const POINT_ITEMS: usize = 0x95c;
    /// **The difficulty**, which the stage's build writes 4 into at `0x14003c345` where the stage is six
    /// or past it — the Extra.
    pub const DIFFICULTY: usize = 0x960;
    /// **The card records**, 134 of [`super::CARD_BYTES`] — see [`super::CARD_HISTORY`].
    pub const CARD_HISTORY: usize = 0x968;
    /// **How far up the point item's own value table the run has got**, nought to thirty.
    ///
    /// The item walk's point case steps it at `0x14004462a` while the power is full, holds it at thirty
    /// where the step reaches `0x1f` (`0x140044636`), and `0x14004464f` indexes the dwords at
    /// `stage + 0x308580` by it for what the item adds to the score. So it is what a point item is
    /// *worth*, where [`POINT_ITEMS`] is how many have been taken.
    ///
    /// **And the stage-following build leaves it alone**, `0x14003c36d` writing nought on the other
    /// path only — so it carries across a stage boundary the way the others beside it do, and a stage
    /// picked up without it would pay the wrong score for every point item after it. A replay's own
    /// stage record keeps it too, at `+0x10` (`0x14006d019`) — see `th06nc/asm.md`.
    pub const POINT_ITEM_VALUE: usize = 0xd27d;
    /// **The rectangle the player is held to**, four floats the stage's build writes `8, 16, 368, 416`
    /// at `0x14003c2d9`.
    pub const PLAYER_AREA: usize = 0xd280;
    /// **The lives and the bombs**, one byte each — the item walk's 1up case stops the first at eight at
    /// `0x140044cc7` and the bomb item the second at `0x140044c87`.
    pub const LIVES: usize = 0xd290;
    pub const BOMBS: usize = 0xd291;
    /// **How many extra lives the score has paid for**, the byte the extend check at `0x14003c9d2`
    /// climbs.
    pub const EXTRA_LIVES: usize = 0xd292;
    /// **The seed the stage started from**, which `0x14003c9f1` copies out of the generator as the stage
    /// is built and `0x14003c9f8` zeroes the count of draws beside — see [`super::RNG_SEED`], which is
    /// the generator itself.
    pub const RANDOM_SEED: usize = 0xd408;
    /// **The frames the stage has run**, which the stage's own job steps up once a run at `0x14003c04d`
    /// and the chain build writes nought into at `0x14003c0f7`.
    pub const GAME_FRAMES: usize = 0xd418;
}

/// **The generator**, one word: every draw in the exe is inlined as
/// `xor cx,[0x140abae64]; sub cx,0x6553; movzx eax,cx; shl cx,2; shr ax,0xe; add ax,cx` with the sum
/// written back — 216 references, `0x14000db9a` and `0x14003c3a0` among them. The same shape th06c's
/// has.
const RNG_SEED: usize = 0x1_40ab_ae64;
/// **A card record**, `0x180` bytes, of which the stage's build lays out 134 at
/// [`game_manager::CARD_HISTORY`] — `0x14003c390`–`0x14003c457`, each preceded by `0xc0` words drawn
/// from the generator so that a record the game has not named holds noise and not zeroes, which is what
/// th06c's do too. `0x14006e310` at `0x14003c5e7` is what lays the file's own `CATK` records over them.
/// **The renderer**, a pointer to the `0x37070` bytes `main` takes off the heap at `0x140047521`,
/// zeroes and builds with `0x1400021c0` before writing it here — the last `0xff6c` of them the batch of
/// sprites, see [`renderer`]. The frame leaves `-1` in its `+0x20f80` after the draw, at `0x14003dafc`.
const THE_RENDERER: usize = 0x1_40ab_ac30;
pub(crate) const THE_RENDERERS_BLOCK: usize = 0x3_7070;
/// The field the frame leaves `-1` in.
const WHAT_THE_FRAME_LEAVES_IN_IT: usize = 0x2_0f80;

/// **The decoded `data/ascii.anm`**, a pointer to the block `0x14028fbf0` takes off the heap at
/// `0x1400f44d0` and writes here after handing the base64 of [`THE_SPRITE_SHEET_A_DRAW_DIVIDES_BY`] to a
/// decode.
///
/// **What is in it, read off `0x14028fcaa` onwards**: a header of `u16` counts at `[base]`, `[base+2]`
/// and so on, the data beginning at `base +` [`THE_SPRITE_SHEETS_HEADER`], and a cursor that each count
/// advances — its value written into the table of pointers at `0x140a80518`, eight bytes apart, by a loop
/// of `mov r9d,0x188` over eight counts an iteration after an unrolled first eight. So there are about
/// 3,144 counts and the block is the header plus their sum.
///
/// **And the pointer has only ever read nought**, which is what the `detail!` beside it exists to say:
/// every snapshot of every launch driven since reports
/// `memory: the sprite sheet's pointer reads Some(0); nothing is held back`, so `0x14028fbf0` has not run
/// by the time a chapter is taken — its one caller is `0x140291fba` and nothing else in the exe calls it.
/// The range stays for the launch where the decode does run, the block being the same kind of thing as
/// [`THE_RENDERERS_BLOCK`] — a heap block the game keeps a pointer to and a restore would write over —
/// and the line is what says which of the two happened rather than leaving a range that quietly holds
/// nothing back.
const THE_SPRITE_SHEETS_BLOCK: usize = 0x1_40a8_0510;
/// The header of counts the decoded sheet begins with — `lea rax,[rdx+0x1bec]` at `0x14028fcb5`, which is
/// where the data after it starts.
const THE_SPRITE_SHEETS_HEADER: usize = 0x1bec;
/// How many counts that header holds: the unrolled first eight and then `0x188` iterations of eight —
/// `mov r9d,0x188` at `0x14028fd69`. Read as the bound of the walk and not as a count of anything the
/// game says, so a sheet with fewer is a sheet whose later counts are nought.
const THE_SPRITE_SHEETS_COUNTS: usize = 8 + 0x188 * 8;

/// **The statics the renderer's COM objects are kept in**, which `0x140271c30` releases one after
/// another: `0x1409fcca8` and `0x1409fcca0` have their `+0x370` and `+0x378` called first — a device and
/// its context — and then `0x1409fcc48`, `0x1409fcc98`, `0x1409fcc50`, `0x1409fcc58`, `0x1409fcc60`,
/// `0x1409fcc68`, `0x1409fcc70`, `0x1409fcc78` and `0x1409fcc90`, with the sixteen bytes at
/// `0x1409fcc80` zeroed among them.
///
/// **A range and not the eleven, because the bound is what matters**: a restore that put any of them
/// back would name an object released since.
///
/// **And it is not every pointer a release names, which this range once claimed.** cdb caught the same
/// crash with this range and the counts both held back, on a static at `0x1409886c8` — see
/// [`THE_DRAWS_OWN_OBJECT`], which is `.data + 0x61a728` and nowhere near here. A bound over the
/// neighbours of the ones a release was read naming is a bound over the ones that were read, and no more.
const THE_RENDERERS_OBJECTS: Range<usize> = 0x1_409f_cc48..0x1_409f_ccb0;

/// **And the counts of how many of each of those objects are alive**, which begin where the pointers
/// above end and which a restore must leave alone for the same reason: a count is bookkeeping about
/// memory that is not the game's, and one put back describes a moment the process has left.
///
/// **Thirteen dwords, each `inc`ed where an interface is made and `dec`ed in its own `Release` thunk**:
/// `0x1409fccb0`, `d18`, `d20`, `d24`, `d28`, `d2c`, `d30`, `d34`, `d38`, `d3c`, `d40`, `d44` and `d48`
/// — `0x140271461` and `0x140272c20` are that pair for the first of them — and `0x140272a02` onwards
/// reads all thirteen in a row, which is what says they are one bank and not thirteen unrelated globals.
///
/// **The upper bound is `0x1409fccec` because `0x1409fccf0` is a different thing**: it is never `inc`ed
/// or `dec`ed and is taken by `lea` in a hundred places as the address of an object, where every dword
/// below it is only ever stepped and read.
///
/// **And it is why a run of th06nc died at its first chapter retry.** `0x1409fccdc` is one of the
/// thirteen, and the crash Windows recorded is `0xc0000005` at `0x140272c19` —
/// `dec DWORD PTR [0x1409fccdc]`, then `mov rax,[rcx]`, then `jmp [rax+0x10]`, which is
/// `IUnknown::Release` tail-called through a vtable that is not there. A count the game had raised since
/// the snapshot came back lowered, so a later release took an object still in use down to nought.
const THE_RENDERERS_OWN_COUNTS: Range<usize> = 0x1_409f_ccb0..0x1_409f_ccec;

/// **The object one of those releases is called on, which is not in the range above** — and the reason
/// that crash came back with the counts already held out of every restore.
///
/// **Caught under cdb, which is what this needed.** The fault prints nothing from orb's own filter, the
/// log being written on a thread of its own, so what a filter queues as the process dies goes with it;
/// three launches of the stage-3 recording recipe with `cdb -p <pid> -c 'g; .lastevent; .exr -1; r; kb'`
/// attached died on the third, and the debugger held it:
///
/// ```text
/// ExceptionCode: c0000005 (Access violation)
/// Attempt to read from address 0000000000000a88
/// rax=0000000000000a78 rcx=00000227d2891a08
/// th06nc+0x270e69: jmp qword ptr [rax+10h] ds:0000000000000a88=????????????????
/// ```
///
/// — `rax` is `[rcx]`, the object's own vtable pointer, and it reads `0xa78`: not a pointer at all. The
/// stack is the **draw frame's**, `0x14003da90` under `0x14002b1e7`, `0x14007cbc4`, `0x14003d42d`,
/// `0x140099560`, `0x140256071`, `0x140247c52` and `0x140247164`.
///
/// **And the caller says which static held it.** `0x14024713b` onwards reads `[0x1409886c8]` into `rcx`,
/// skips where it is null or where the dword at `0x1409886d0` is nought, calls the release thunk at
/// `0x140272c10` and then clears the static:
///
/// ```text
/// 0x14024713b  mov  rcx,[0x1409886c8]
/// 0x140247152  test rcx,rcx      ; je past the release
/// 0x140247157  cmp  [0x1409886d0],0
/// 0x14024715d  je   past the release
/// 0x14024715f  call 0x140272c10  ; dec [0x1409fccdc]; mov rax,[rcx]; jmp [rax+0x10]
/// 0x140247164  mov  [0x1409886c8],rsi
/// ```
///
/// **`0x1409886c8` is `.data + 0x61a728`, nowhere near [`THE_RENDERERS_OBJECTS`]'s
/// `0x1409fcc48..0x1409fccb0`** — so that range's own claim that every pointer a release names is inside
/// it is **wrong**, and this is the pointer it missed. A restore put the chapter's copy of it back, the
/// object it named had been freed and its memory reused, and the release followed it into whatever is
/// there now.
///
/// **The pointer and the count that gates it**, because holding one without the other leaves a release
/// that fires on a live object with a chapter's count or does not fire at all: the qword at `0x1409886c8`
/// and the dword at `0x1409886d0`, which is twelve bytes and costs a run nothing.
const THE_DRAWS_OWN_OBJECT: Range<usize> = 0x1_4098_86c8..0x1_4098_86d4;

/// **The sprite sheet a restore must leave as the run has it**, `data/ascii.anm` embedded in `.data` as
/// base64: 153,898 bytes from `0x1403f8140`, which is where the contiguous run of base64 bytes in the
/// exe's own image begins and ends.
///
/// **The base is the one address the exe names in all of it.** `0x14028fbf0` takes `0x1403f8140` by `lea`
/// four times — to `0x14025dfb0` and then twice to `0x140260040` — and names `data/ascii.anm` at
/// `0x14034da88` beside them, allocating through `0x1400f44d0` and reading three counts out of the block
/// at `[rdx]`, `[rdx+2]` and `[rdx+4]`. Nothing in the disassembly names an address *inside* the run, so
/// its end is measured off the bytes: `0x8b140..0xb0a6a` of `.data` is base64 the whole way and neither
/// byte either side of it is.
///
/// **And what says a restore must not put it back is a bisection, byte by byte.** `--stress=600` restores
/// every snapshot as it is taken; driven that way against `the-first-stage-at-full-speed.yaml` the game
/// died on its third forced restore every time, always on stage 1 frame 609. Twenty-three halvings with
/// `--hold-back-at` — `0+483330` surviving eight, `0+90666` dying on the control's own frame, and so down
/// through `90666+120c`, `90f6c+483`, `911ad+90`, `911f5+9` — end at `911f5+1`, one byte, which survived
/// eight restores and ran the recipe out. Held back in this list that byte took the same reproducer from
/// three forced restores to twenty-three, and what it then died of was another byte of this same run:
/// `0x90666`, which the bisection had already passed through. So the range and not the byte.
///
/// **What the bytes do is `0x140003986`'s divisor.** That instruction is `idiv ecx` with `ecx` the
/// truncation of the float at `+0x2c` of `[rbx+0x100]`, and `+0x28` is its pair divided by the same way
/// four instructions later — a draw dividing by a sprite's own two sizes, where the exception Windows
/// records is a division by nought. A drawable points into this sheet, so a sheet put back to what the
/// chapter had while the drawables are what they are now is a drawable pointed at a size that is not set
/// up.
const THE_SPRITE_SHEET_A_DRAW_DIVIDES_BY: Range<usize> = 0x1_403f_8140..0x1_4041_da6a;

/// **The block the sound layer keeps its own state in**, which a restore must leave alone: thirty-two
/// objects of `0x388` bytes at `0x1407e20c0`.
///
/// **Read off the table itself, not guessed from th06c's.** `0x1400f2f40` indexes it —
/// `lea r12,[0x1407e20c0]; imul rsi,rax,0x388` with the slot's first dword at `[rsi+r12]` — and
/// `0x1400f3610` walks it, `mov r14d,0x20` being how many and `add rbx,0x388` the step; `0x1400fbb94`
/// holds an index against `0x20` besides. **And the bounds are where the references stop**:
/// `0x1407e9190`, the last eight bytes this range would cover past the thirty-second slot, is referenced
/// nowhere, and `0x1407e91c0` — where the range ends — is referenced from seven places as something
/// else's first field.
///
/// **What makes it a range to leave alone is what each slot holds**: a lock at `+0x70`, which
/// `0x1400f2f85` takes the address of and `0x140264720` and `0x1402647d0` enter and leave around the
/// body of every walk. A lock put back from a snapshot says a thread is somewhere it is not — which is
/// what holding th06c's own block back was for, and what putting it back cost that game was the frame
/// rate after a chapter.
const THE_SOUND_LAYERS_BLOCK: Range<usize> = 0x1_407e_20c0..0x1_407e_91c0;

/// **The block the sound effects are queued through**, which a restore leaves alone: five queues, each a
/// `0x230`-byte lock — [`A_LOCKS_BYTES`] — with its own head, tail and count after it, at `0x1405fd520`,
/// `0x1405fd780`, `0x1405fd9f0`, `0x1405fdc60` and `0x1405fdf50`, under a header from `0x1405fd454` whose
/// `0x1405fd480` is the flag saying the block is built, referenced from 35 places.
///
/// **And the bounds are where the references stop**: nothing between `0x1405fd2a0` and `0x1405fd454` is
/// referenced at all, and the fifth queue's lock ends exactly at `0x1405fe180`, where the media's own
/// settings begin.
///
/// **What said a restore must leave it alone is a deadlock the sampler caught.** The frame's thread stood
/// at `th06nc.exe+0x262866` — the instruction after `EnterCriticalSection` inside the lock enter — under
/// `0x1400c9faf`, which is the `lea rcx,[0x1405fd9f0]` at `0x1400c9fa3` returned into, under
/// `0x140078257`, the flush of the frame's queued effects, under `0x14003d76d`, the calc walk's tail. So
/// the frame waits on a queue's lock whose owner is a thread of the library's, and a queue put back from a
/// snapshot leaves that owner where it is not: eight frames after a chapter went back the frame's thread
/// never came out of that wait, while orb's own threads went on logging. Holding the five locks alone was
/// not enough — the launch that did hang that way is what the block is measured from.
const THE_MEDIAS_QUEUES: Range<usize> = 0x1_405f_d454..0x1_405f_e180;

/// **The rest of the game's own locks, each a static of its own in `.data`**, which a restore leaves
/// alone for the reason [`THE_SOUND_LAYERS_BLOCK`] is left alone: a lock put back from a snapshot says a
/// thread is somewhere it is not, and a thread that is not the frame's holds these.
///
/// **Read off every call of the game's own lock enter**, `0x140264720`, whose object comes in `rcx` and
/// goes straight to `EnterCriticalSection` — `0x140264770` calls the pointer at `0x1405a7ef8`, which
/// `0x1400b5dd6`'s `GetProcAddress` of that name fills. Of its 93 call sites 19 take an object inside one
/// of the blocks above, 20 take one that cannot be read at the site — the sound layer's own `+0x70`, or a
/// field of a heap object the caller passes in — and the other 54 take one of twelve statics, of which
/// two are in `.rdata` and no snapshot copies them. Of the ten in `.data`, five are
/// [`THE_MEDIAS_QUEUES`]' and these are the rest. *The lock's own functions* in `asm.md` is the reading,
/// with the script that repeats it.
const THE_LOCKS_IN_THE_DATA: [usize; 5] = [
    0x1_405c_a1d0,
    0x1_405c_b520,
    0x1_407f_9a20,
    0x1_4080_5b48,
    0x1_409b_3468,
];

/// **The CRT's own locks**, which a restore leaves alone for the reason [`THE_LOCKS_IN_THE_DATA`] are left
/// alone. This build's CRT is linked into the exe, so its locks are statics of `.data` where the build
/// before kept them in `ucrtbase.dll`, whose memory no snapshot copies:
///
/// - fifteen of `0x28` at `0x140aba1a0`, the UCRT's own table, each initialised with a spin count of
///   `0xfa0` in the loop at `0x1402d51c8`..`0x1402d51f0`, the count of them at `0x140aba3f8` after;
/// - one at `0x140aba108`, the loop at `0x1402c5a14` stopping at one;
/// - and the three standard streams at `0x140426390`, `0x58` bytes apiece, each with its lock at `+0x30`
///   initialised at `0x1402d1a29` — held back whole, nothing of the game's own writing to them.
///
/// **What this does not reach is the CRT's objects off the heap** — an open file's own lock and the table
/// of its handles — which are the game's heap now and put back with it; see `th06nc/asm.md`'s *What is
/// not established*.
const THE_CRTS_LOCKS: [Range<usize>; 3] = [
    0x1_40ab_a1a0..0x1_40ab_a3f8,
    0x1_40ab_a108..0x1_40ab_a130,
    0x1_4042_6390..0x1_4042_6498,
];

/// **How long one of those locks is.** `0x1402646a0` calls `InitializeCriticalSection` on the object's
/// own address — `0x28` bytes on x86-64 — and then zeroes the `0x200` bytes at `+0x28` that
/// `0x140264798` copies the enter's name into; `0x1402647b9` and `0x1402647ae` write the enter's own
/// number at `+0x228` and the calling thread's id at `+0x22c`.
const A_LOCKS_BYTES: usize = 0x230;

/// **The allocator's own state, which a restore must not rewind** — and the one range here whose reason
/// is neither a handle nor a thread: it describes *the heap* rather than the run, and the heap is not
/// rewound with it.
///
/// **This build allocates out of a TLSF of its own** — Two-Level Segregated Fit, whose blocks carry a tag
/// and whose free ones hang off a two-level index — and it checks that structure on every allocation.
/// The check at `0x14026bc41` and `0x14026bc73`, inside the allocate at `0x14026b850`, works a block's
/// aligned end out of `[MemTag + 0x20]` and `[MemTag + 0x28]`, and where it does not reach the size asked
/// for it reports `TLSF カテゴリ構造エラー 1` or `2` through `0x1400c2ac0` and falls into
/// **`0x14026bc81`, `mov DWORD PTR ds:0x0,0xffffffff`** — a write to address nought, which is how this
/// game ends a process it has decided is unsound.
///
/// **The object is the static the allocate is handed in `rcx`**, `0x1407e91e8`: `0x1400f4255` is its
/// build-once — `[0x1407e91e0]` the flag, `lea rdi,[0x1407e91e8]` the object — with `+0x38` a mode,
/// `+0x40` the default alignment and `+0xd0` a lock of its own. **The upper bound is where the next
/// object begins**, `0x1407f1600`, nothing between the two being referenced from anywhere; the lower is
/// the flag, which says the allocator is built and is as much a fact about the process as the structure
/// itself.
///
/// **What said so is a debugger and then a launch.** `cdb` attached across a `--stress=60` launch caught
/// the seventeenth restore — the first of a `music=keep` chapter of stage 1 — as
/// `Access violation … th06nc+0x269d71 … Attempt to write to address 0`, with `rdi=0x300` the size asked
/// for and `r14=0x1c19028ce10` the MemTag; four launches of `an_attempt_at_a_card_counted_and_kept` and
/// as many probes had died there before, and holding this range back through `--hold-back-at` took the
/// same probe through **retries 17, 18, 19 and 20 of that chapter alive**. See `th06nc/asm.md`'s *The
/// allocator, which checks itself and kills the process*.
///
/// **And what holding it back costs is a block per retry.** Every allocation the run made after the
/// snapshot stays allocated, the tags that name it no longer being rewound — a leak inside the game's own
/// arena and not the process heap. The alternative is taking the region set from the arena's own segment
/// list at the moment of the restore rather than at the snapshot's, which is a reading this does not have.
const THE_ALLOCATORS_OWN_BLOCK: Range<usize> = 0x1_407e_91e0..0x1_407f_1600;

const CARD_HISTORY: usize = GAME_MANAGER + game_manager::CARD_HISTORY;
const CARDS: usize = 134;
const CARD_BYTES: usize = 0x180;
const CARD_HISTORY_BYTES: usize = CARDS * CARD_BYTES;

/// `'CATK'`, which the stage's own layout writes into every record at `0x14003c390` onwards.
const CATK_MAGIC: u32 = u32::from_le_bytes(*b"CATK");
/// **The card's name**, copied in byte by byte as the card starts — the loop at `0x140027950`, whose
/// destination is `lea rcx,[rsi+0x18]` at `0x14002794c`.
const CATK_NAME: usize = 0x18;
const CATK_NAME_BYTES: usize = CATK_ATTEMPTS - CATK_NAME;
/// **The low byte of the sum of the name's own bytes**, which `0x140027990`'s walk builds and
/// `0x1400279a0` compares: a name whose sum is another's is another card in this slot, and
/// `0x1400279aa` puts the count back to nought with the rest of the record.
const CATK_NAME_SUM: usize = 0x12;
/// **The attempts, a word capped at 9999**: `0x140027a2a` reads it, `0x140027a2e` holds it against
/// `0x270f` and `0x140027a3a` writes it back a step up.
const CATK_ATTEMPTS: usize = 0x3c;

/// **Which card the fight is on**, the dword the card's own start indexes the records by:
/// `movsx eax,WORD PTR [r14+0xe]` at `0x1400278ab` writes it, and `0x140027929` onwards reads it back,
/// refuses anything from `0x86` — the 134 of [`CARDS`] — and turns it into
/// `lea rcx,[rax+rax*2]; shl rcx,0x7`, which is `card * 0x180` off `0x14053d418`, the same
/// [`CARD_HISTORY`] this does.
const CURRENT_CARD: usize = ENEMY_MANAGER + enemy_manager::CARD_NOW;

/// **The copy of the score file the game reads once, at startup, and every parse of it works off**: the
/// pointer `0x14006e140` takes a copy of and deciphers, and the length it refuses under `0x18`.
const SCORE_FILE_COPY: usize = 0x1_40c6_e368;
const SCORE_FILE_COPY_LENGTH: usize = 0x1_40c6_e18c;

/// **The two functions the game takes that block through**, its own CRT's `malloc` and `free` —
/// `0x14003c471` and `0x14003c4db` are two of the call sites. What orb's own copy has to be taken off, so
/// that the game's `free` of it is a free of its own heap's — see
/// [`Game::read_what_the_modes_file_records`]. Called rather than stood in front of, the CRT being linked
/// into the exe with no import of them to stand at — see `Memory::ProcessHeapImport`.
const MALLOC: usize = 0x1_402c_db80;
const FREE: usize = 0x1_402c_6540;
/// **And how many draws have been made**, the dword four bytes below it, stepped up beside each draw —
/// by one at `0x14003c3a0` and by two where a draw takes two words, `0x14000dba1` — which the stage's
/// own build writes back to nought at `0x14003c9f8`.
const RNG_COUNT: usize = 0x1_40ab_ae60;

/// **The stage's own build**, `0x14003c250`, called once in the exe — as the calc element's own
/// `+0x10` — the one an element is called as it is registered — from `0x14003c173`, with
/// [`GAME_MANAGER`] in `rcx`, which is what makes it
/// [`Entered::InEcx`].
///
/// **What it puts in place is every number of a stage**: the field, the power, the card records, the
/// clear records, the score to beat, the rank, the seed's copy and the count of draws back to nought — so the
/// moment a run's own numbers can be written over is where this returns, which is what
/// [`Hooks::stage_begun`] is. Writing them at its prologue instead would be writing them under the
/// power, the rank and the extra lives it goes on to set.
const STAGE_BUILD: usize = 0x1_4003_c250;
/// `mov [rsp+0x10],rbx; mov [rsp+0x18],rbp; mov [rsp+0x20],rsi` — fifteen bytes exactly, none of them
/// rip-relative, which is what a 64-bit jump needs.
const STAGE_BUILD_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x10, 0x48, 0x89, 0x6c, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24, 0x20,
];

/// **The job registered right after the seed has been copied**, `0x14007a4a0`, called once in the exe —
/// at `0x14003ca09`, three instructions past `0x14003c9f1`'s copy and before anything of the stage has
/// drawn from the generator. Which is the one place a resumed run's seed can go in, and why it is a seam
/// of its own — see [`Hooks::stage_building`].
///
/// **Inside the build and not before it**, because the build draws `0xc0 × 134` words laying its card
/// records out: a seed written at [`STAGE_BUILD`]'s prologue would be twenty thousand draws old by the
/// time `0x14003c9f1` copied it.
const STAGE_SEEDED: usize = 0x1_4007_a4a0;
/// `mov [rsp+8],rbx; mov [rsp+0x10],rbp; mov [rsp+0x18],rsi` — fifteen bytes.
const STAGE_SEEDED_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18,
];

/// **The whole frame, which orb replaces to own this game's own.** `main`'s loop at `0x140047570` calls it
/// once round at `0x1400475fb` and nothing else calls it at all; it takes nothing, and what it answers is
/// what the loop reads — nought carries on to the message pump at `0x140047606`, anything else leaves the
/// loop at `0x140047613`, and every non-zero answer of the calc walk leaves one, `0x14003d7a1` and
/// `0x14003d78b` writing 1 and 2. What it holds is the camera, the calc chain, the sound's own catch-up,
/// the draw walked inline and the frame's own end — every one of which orb's own loop now calls, in that
/// order. `th06nc/asm.md` has the frame.
const THE_FRAME: usize = 0x1_4003_d8b0;
/// `mov [rsp+8],rbx; mov [rsp+0x10],rbp; mov [rsp+0x18],rsi` — fifteen bytes to the end of the third
/// instruction, none of them rip-relative, where a 64-bit jump needs fourteen.
const THE_FRAME_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x6c, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18,
];

/// **The walk of the calc chain, which is this game's tick** — `0x14003d700`, `rcx` a pointer the answer
/// the frame leaves with is written through, `al` non-zero to leave. It flushes the frame's queued sound
/// effects itself through `0x1400781d0` before it answers — see `th06nc/asm.md`.
const RUN_CALC_CHAIN: usize = 0x1_4003_d700;

/// **The camera the frame puts in place before anything of it runs, and again before its draw**:
/// `0x14007ccb0` on [`THE_GAME`] with nought and the float at `0x140352a90`, at `0x14003d8e5` and at
/// `0x14003da14`.
const MOVES_THE_CAMERA: usize = 0x1_4007_ccb0;
const THE_CAMERAS_FLOAT: usize = 0x1_4035_2a90;

/// **The draw, which the frame makes inline** at `0x14003d9fd`..`0x14003daf0` — the frame's own copy of
/// what was a function of its own in the build before — and orb makes call for call: see
/// [`walks_the_draw`]. First `0x1400019d0`, which puts the screenshot's two flags down; then the camera
/// again; then the viewport read into four ints with `0x1400855f0`, set to the four bytes at
/// `0x1405547f0` with `0x140097160`, the target cleared with `0x140081330(0)`, and the four ints put back
/// with `0x140097160`; then the draw chain; and last the screenshot's own path, `0x140001c70`.
const BEGINS_THE_DRAW: usize = 0x1_4000_19d0;
const READS_THE_VIEWPORT: usize = 0x1_4008_55f0;
const SETS_THE_VIEWPORT: usize = 0x1_4009_7160;
const CLEARS: usize = 0x1_4008_1330;
const TAKES_THE_SCREENSHOT: usize = 0x1_4000_1c70;
/// The four bytes the viewport is set to for the clear, in the order `0x14003da32`..`0x14003da49` hand
/// them over: `+2`, `+1`, `+0` and `+3`.
const WHAT_IS_CLEARED: usize = 0x1_4055_47f0;

/// **The draw chain's head**, which the frame walks inline at `0x14003da73` — `lea rdi,[0x14053c050]`,
/// the same walk as the calc chain's with nought taking an element out.
const THE_DRAW_CHAIN: usize = 0x1_4053_c050;
/// **What takes an element out of either chain**: `0x140012e10` with the element in `rdx`, called at
/// `0x14003db16` with the walk already moved on.
const TAKES_AN_ELEMENT_OUT: usize = 0x1_4001_2e10;

/// **What draws the sprites the renderer's batch holds** — see [`renderer`]: `0x140083eb0` with the batch,
/// its count, four, the word beside the count and one, which the draw walk calls after every job at
/// `0x14003da97`..`0x14003dace` and zeroes the count after.
const FLUSHES_THE_SPRITES: usize = 0x1_4008_3eb0;

/// **The batch every draw of this game queues its sprites into**, past the renderer's own fields — the
/// build before had none, and its renderer was `0x27108` bytes where this one's is [`THE_RENDERERS_BLOCK`].
/// Nothing is drawn until [`FLUSHES_THE_SPRITES`] is called on it.
mod renderer {
    /// The sprites themselves.
    pub const SPRITES: usize = 0x2_7104;
    /// How many vertices of them are queued, read at `0x14003daa3` and zeroed at `0x14003dace`.
    pub const QUEUED: usize = 0x3_7064;
    /// And the word the flush is handed beside the count, `0x14003daad`.
    pub const FLUSHED_AS: usize = 0x3_7068;
}

/// **What the game does after the draw frame**, `0x14003db50`: the bars either side of a viewport that
/// does not fill the window, the picture finished through `0x140097150`, and then the frame limiter and
/// the fps line. See [`Th06nc::end_drawing`] for how the limiter is got past.
const ENDS_THE_FRAME: usize = 0x1_4003_db50;
/// **The game's own object**, `0x140c6db90`, which the screen-mode functions and the config writers are
/// called on: `th06.cfg`'s 44 bytes are at its `+0x434` and `th06.env`'s twelve at its `+0x48c`.
const THE_GAME: usize = 0x1_40c6_db90;

/// **`th06.env`, twelve bytes read into `0x140c6e01c` at startup by `0x14007d240` and written back at
/// exit** — see `th06nc/asm.md`. The two of them that are the window.
mod env {
    /// **Which screen mode the game is in**: `main` reads it at `0x1400472e5` and calls one of three
    /// functions on it — nought [`super::MAKES_A_WINDOW`], one [`super::COVERS_THE_MONITOR`], two
    /// `0x14007dcb0`, the fullscreen — and `0x140046bc0` flips it between nought and one on a key.
    pub const SCREEN_MODE: usize = 0x1_40c6_e020;
    /// The mode that is a window of one of [`WINDOW_SIZES`].
    pub const A_WINDOW: u8 = 0;
    /// The mode that is a window covering the monitor, which is what `Screen::Fullscreen` means.
    pub const THE_MONITOR: u8 = 1;

    /// **Which window the window mode makes**, read at `0x14007e27b` as `[game + 0x491]`: nought is the
    /// largest of the four that fits the monitor, and one to four are the four, largest first. The
    /// option screen steps it at `0x14004fada`, and the mode function writes back the one it settled on
    /// at `0x14007e2fa`. The two bytes after it are the same choice for the other two modes, where it caps
    /// the picture and not the window.
    pub const WINDOW_SIZE: usize = 0x1_40c6_e021;

    /// **The four windows the window mode offers**, by the item that names each — `0x14007e2d0`,
    /// `0x14007e2c4`, `0x14007e2dc` and `0x14007e2ba`.
    pub const WINDOW_SIZES: [(u8, u32, u32); 4] = [
        (1, 3840, 2160),
        (2, 2560, 1440),
        (3, 1920, 1080),
        (4, 1280, 720),
    ];
}

/// **The function the window mode is entered by**, `0x14007e1b0`, called on [`THE_GAME`]: it reads
/// [`env::WINDOW_SIZE`], sizes the window and the picture to that item through `0x140098a40` at
/// `0x14007e36c`, and works with the window already made — the option screen calls it on the running
/// game at `0x14004feaa`, which is how orb calls it too.
const MAKES_A_WINDOW: usize = 0x1_4007_e1b0;

/// **And the function mode one is entered by**, `0x14007df70`, which sizes the window to the monitor it
/// is on — `0x140098a40` at `0x14007e0e7` takes the monitor's own size — whatever the item says: the
/// item caps the picture there. A launch with the env at this mode came up the whole of a 4K monitor,
/// `3862x2216` with its frame, which is the window that was too large.
const COVERS_THE_MONITOR: usize = 0x1_4007_df70;

/// **The function the picture is sized by**, `0x140098a40`, `(width, height, bits, hertz)`: it keeps the
/// four at `0x140555138` onwards and remakes the renderer at that size through `0x1400f2a00`, or
/// `0x140093d30` where one is up already. The mode functions call it with the item's size, or the
/// monitor's; called with a size of orb's own it takes that — a size of nought is `640x480` there, and
/// bits other than 8, 16 or 32 are refused at `0x140098aa7`.
const SIZES_THE_PICTURE: usize = 0x1_4009_8a40;
const THE_PICTURES_BITS: u32 = 32;
const THE_PICTURES_HERTZ: u32 = 60;

/// **Where in the window the picture is drawn**, four dwords — left, top, width, height — which the mode
/// functions write at `0x14007e168`, `0x14007e3ee` and `0x14007df22` out of the size they chose, letterboxed
/// to 16:9, and the draw reads back at `0x14003db94` and forty more places. Written by orb where it sizes the
/// picture itself, since the mode function that wrote it last was told another size.
const THE_PICTURE: usize = 0x1_40c6_e05c;
/// **And whether that picture is taller than 1080**, the byte `seta` writes beside it at `0x14007e3df`,
/// which the draw tests in fifty places to pick its larger assets.
const THE_PICTURE_IS_TALL: usize = 0x1_40c6_df34;

/// **The window a launch that plays itself gets**: as wide as `crate::window`'s own out-of-the-way client
/// for a 640x480 game, and 16:9 — a quarter of the smallest window the game offers. Exactly 16:9, so that
/// [`THE_PICTURE`] is the whole of it and no letterbox has to be worked out.
const OUT_OF_THE_WAY: (u32, u32) = (320, 180);

/// Whether the window has been sized to the settings, which is done on the first frame and never again.
static THE_WINDOW_IS_SIZED: AtomicBool = AtomicBool::new(false);

/// **The window sized to `Config::screen`, once**: `th06.env`'s mode and choice are written and the mode's
/// own function entered, the way the option screen does it — the window mode with the largest of
/// [`env::WINDOW_SIZES`] inside a `Screen::Window`, and the mode that covers the monitor for
/// `Screen::Fullscreen`, which is the borderless window orb means by that.
///
/// **On the first frame rather than before the window is made**, because there is nothing of orb's between
/// the two: `main` reads `th06.env` at `0x1400470d2`, chooses the size at `0x1400472d3` and makes the window
/// at `0x1400473d0` with no call orb answers in between, and a byte written before the read is read over.
/// So the window comes up the size the env says and is resized on the frame after — one frame of the
/// env's own window, against a patch on `main` for the other route.
///
/// **What it is not is the setting's exact size**: this game draws into a swapchain of its own at the
/// size it chose, so a window of another size would clip or stretch it — see `crate::window`'s note on
/// th06c. The four the game offers are what it can be, and the setting picks among them.
///
/// **And a launch that plays itself gets [`OUT_OF_THE_WAY`] whatever the settings say**, the way every
/// other game's does — see `crate::window::placed_out_of_the_way`. The game offers nothing that small, so
/// the window mode is entered with its smallest and then the picture is sized again through
/// [`SIZES_THE_PICTURE`] with orb's own size, [`THE_PICTURE`] written to match; and the frame the mode put
/// back is made a dialog's again, since the mode makes the window over and drops what
/// `create_window_ex_w` framed it with.
///
/// # Safety
/// Must run on the game's main thread, with the window made and the exe's base already told.
unsafe fn size_the_window_to_the_settings() {
    if THE_WINDOW_IS_SIZED.swap(true, Ordering::Relaxed) {
        return;
    }
    let was = (
        mem::read_committed::<u8>(at(env::SCREEN_MODE)),
        mem::read_committed::<u8>(at(env::WINDOW_SIZE)),
    );
    let out_of_the_way = unsafe { crate::window::plays_out_of_the_way() };
    let wanted = if out_of_the_way {
        let (width, height) = OUT_OF_THE_WAY;
        orb_config::Screen::Window { width, height }
    } else {
        unsafe { crate::window::screen() }
    };
    let (mode, entered, said) = match wanted {
        orb_config::Screen::Window { width, height } => {
            let (item, chosen_width, chosen_height) = env::WINDOW_SIZES
                .iter()
                .copied()
                .find(|(_, wide, high)| *wide <= width && *high <= height)
                .unwrap_or(env::WINDOW_SIZES[env::WINDOW_SIZES.len() - 1]);
            unsafe { mem::write::<u8>(at(env::WINDOW_SIZE), item) };
            (
                env::A_WINDOW,
                makes_a_window(),
                if out_of_the_way {
                    format!(
                        "the launch plays itself, so the window is {width}x{height} out of the \
                         game's own {chosen_width}x{chosen_height}, item {item}"
                    )
                } else {
                    format!(
                        "the settings ask for {width}x{height}, so the window is the game's own \
                         {chosen_width}x{chosen_height}, item {item}"
                    )
                },
            )
        }
        orb_config::Screen::Fullscreen => (
            env::THE_MONITOR,
            covers_the_monitor(),
            "the settings ask for fullscreen, so the window covers the monitor".to_owned(),
        ),
    };
    unsafe { mem::write::<u8>(at(env::SCREEN_MODE), mode) };
    let enters: unsafe extern "win64" fn(usize) = unsafe { std::mem::transmute(entered) };
    unsafe { enters(at(THE_GAME)) };
    if out_of_the_way {
        let (width, height) = OUT_OF_THE_WAY;
        let sizes: unsafe extern "win64" fn(u32, u32, u32, u32) =
            unsafe { std::mem::transmute(sizes_the_picture()) };
        unsafe {
            sizes(width, height, THE_PICTURES_BITS, THE_PICTURES_HERTZ);
            mem::write::<[i32; 4]>(at(THE_PICTURE), [0, 0, width as i32, height as i32]);
            mem::write::<u8>(at(THE_PICTURE_IS_TALL), 0);
        }
        // And the frame round that window is `runtime`'s to keep from here on, asked every frame rather
        // than on this one — see `crate::window::keeps_its_frame`, which has why one frame is not enough.
    }
    crate::log!(
        "screen: {said}, where th06.env held mode {} and item {}",
        was.0
            .map_or("nothing read".to_owned(), |was| was.to_string()),
        was.1
            .map_or("nothing read".to_owned(), |was| was.to_string()),
    );
}

/// **The limiter's own base and count**: the deadline it waits for is `base + period × (count + 1)`, and
/// `0x14003dccd` starts both over — base to now, count to nought — where now is already more than a
/// period past the deadline.
const THE_LIMITERS_BASE: usize = 0x1_40c6_e348;
const THE_LIMITERS_COUNT: usize = 0x1_40c6_e350;

/// **The sound's own catch-up, which the loop does between the calc walk and the draw** — `0x14003d966`
/// onwards. What it is for is a run that has gone by faster than the music: the loop counts the extra
/// walks of the calc chain it made into [`THE_EXTRA_WALKS`] and this is where that count is turned into a
/// seek and cleared.
const THE_EXTRA_WALKS: usize = 0x1_4055_42e0;
/// Which track is playing, and `-1` for none: the seek's second argument.
const THE_TRACK_PLAYING: usize = 0x1_4055_42bc;
/// Whether that track is playing at all, and whether it loops — the two bytes the catch-up reads before
/// it decides which bound to wrap the seek inside.
const THE_TRACK_IS_PLAYING: usize = 0x1_4055_42cc;
const THE_TRACK_LOOPS: usize = 0x1_4055_42cd;
/// Where that loop starts and ends, in the same samples the seek is in.
const THE_LOOP_STARTS: usize = 0x1_4055_42d0;
const THE_LOOP_ENDS: usize = 0x1_4055_42d8;
const WHERE_THE_MUSIC_IS: usize = 0x1_400c_66f0;
const HOW_LONG_THE_TRACK_IS: usize = 0x1_400c_6c70;
const SEEKS_THE_MUSIC: usize = 0x1_400c_b610;
/// **How many samples a walk is**, which the loop multiplies the count by outright — `imul rbx,rcx,0x320`
/// at `0x14003d99e` — 48,000 a second in sixtieths, where th06c asks the sound for its rate.
const SAMPLES_A_WALK: i64 = 0x320;

/// **The block of statics the music is played out of**, which a restore leaves alone so that the track
/// goes on where it is: every one the frame's own catch-up reads, from [`THE_TRACK_PLAYING`] to one past
/// [`THE_EXTRA_WALKS`].
///
/// **The bounds are the lowest and the highest of them**, and what says nothing of the block is missed
/// above is `main`: `0x1400474dd` takes the address of `0x1405542e8` — the next object along — so there
/// is nothing of the sound's between that and `0x1405542e4`. Inside it, besides the track's own five
/// statics, are two more:
///
/// - **the fade**, `0x1405542b0`..`0x1405542b8`, which the supervisor's own job counts down at
///   `0x14007b827` while `0x1405542b8` is set and, at nought, stops the track through `0x1400ceb30` and
///   writes [`THE_TRACK_PLAYING`] to `-1` — so a fade in progress is as much where the music is as the
///   track is;
/// - **the frame's queue of sound commands** at `0x1405542c0`, which `0x1400781d0` flushes: a restore
///   leaves a queued effect where the frame left it rather than playing it twice.
const THE_AUDIO_STATICS: Range<usize> = 0x1_4055_42b0..0x1_4055_42e8;

/// **The run's own numbers, every one of them a field of [`GAME_MANAGER`]** — `th06nc/asm.md` says what
/// each was read off.
///
/// **Fields and not statics of their own**, which is what they had been written down as: the addresses
/// are the same and what they are *in* is one object, the one both of the stage's chain elements are
/// called on.
///
/// The lives and the bombs are the pair the item walk stops at eight (`0x140044cc7`, `0x140044c87`) and
/// the run's own start `0x14004f320` writes out of the two bytes at `0x140c6dfd4` and `0x140c6dfd5` —
/// a starting stock this game lets somebody choose — with `0x302` over both at `0x14004f37c` where the
/// difficulty is 4 or the run is a practice, and `2` over the lives alone at `0x14004f373` for a
/// Challenge run. The supervisor's state 12 writes them the same way as it starts a run again, its
/// `0x302` at `0x14007bca6`. Measured: a launch's run started with `2 live and 3 bomb(s)`.
const LIVES: usize = GAME_MANAGER + game_manager::LIVES;
const BOMBS: usize = GAME_MANAGER + game_manager::BOMBS;
/// A word, the item walk's power cases writing it back at `0x1400446b1` and `0x140044a9b`, and `0x80`
/// being the most the full-power item writes it to at `0x140044e61`.
const POWER: usize = GAME_MANAGER + game_manager::POWER;
const POWER_FULL: u16 = 128;
/// **Counted from one**: `0x14003e6ac` multiplies it by a thousand into the `Stage * 1000 =  %5d` line,
/// a launch's first stage read `stage 1`, and the Extra read `stage 7` — `0x14007bc9c` writes six into it
/// as the Extra starts and the stage's own start then counts it on.
const STAGE: usize = GAME_MANAGER + game_manager::STAGE;
/// Compared against four — the Extra — at `0x14007bc97` beside the stage it then writes; a launch that
/// took the second item of the difficulty played on `1`, and the Extra on `4`.
const DIFFICULTY: usize = GAME_MANAGER + game_manager::DIFFICULTY;
/// **Whether the stage on the screen is a replay being played back**: the frame's own fast-forward at
/// `0x14003d910` runs only where it is set, `0x14004eeb3` sets it and `0x14004f395` clears it in the
/// same code that writes the lives and the bombs a run starts with.
const A_REPLAY_IS_PLAYING: usize = 0x1_4053_d3dc;

/// **The player**, a static of `0x7854` bytes `0x1400694e8` zeroes, with `0x14055184c`'s `0x2a5c` after
/// it, which its own hit test at `0x14006c090` is handed as `rcx` — `th06nc/asm.md` has the reading.
const PLAYER: usize = 0x1_4054_9ff0;

mod player {
    /// **Where the player is**, two floats the move at `0x14006aace` writes and the laser test reads as
    /// the statics `0x140551720` and `0x140551724`.
    pub const POSITION: usize = 0x7730;
    /// **The radius a hit is measured against**: `0x14006c145` adds it to a bullet's smaller half and
    /// squares the sum against the distance, and `0x14006c166` squares it alone against the nearest
    /// point of an enemy's box.
    pub const RADIUS: usize = 0x774c;
    /// **What one frame of holding a direction moves the player by**, four floats the walk at
    /// `0x14006a854` onwards picks between by the direction and the focus bit.
    pub const SPEED_STRAIGHT: usize = 0x7860;
    pub const SPEED_STRAIGHT_FOCUSED: usize = 0x7864;
    pub const SPEED_DIAGONAL: usize = 0x7868;
    pub const SPEED_DIAGONAL_FOCUSED: usize = 0x786c;
    /// And what that is scaled by on each axis before it reaches the position — `mulss` at
    /// `0x14006aaa6` and `0x14006aab6`.
    pub const SCALE_ACROSS: usize = 0x7710;
    pub const SCALE_DOWN: usize = 0x7714;
    /// **Which state the player is in**, one byte: nought is the only one a hit lands in
    /// (`0x14006c1b5`), 2 is what the hit writes (`0x14006c201`), 3 is what `0x14006a6b3` writes as the
    /// invulnerability begins.
    pub const STATE: usize = 0x7898;
    pub const NORMAL: u8 = 0;
    /// **What a stage's own register leaves and what the death's animation ends in**: `0x140069b3b`
    /// sets `cl` to 1 and `0x140069b42` writes it, with the count at `0x78` — already past this state's
    /// own thirty-frame gate at `0x14006a6a9`, so a stage's first update moves straight on to
    /// [`INVULNERABLE`] — and `0x14006a4ce` writes it again once state 2's thirty frames of the death's
    /// animation at `0x14006a4b7` have run, the player put back at (192, 384) and the count at nought,
    /// thirty frames before the invulnerability. A player in it is a player no hit lands on and a frame
    /// no chapter is taken on — see [`Game::read_state`]'s `unsettled`.
    pub const COMING_BACK: u8 = 1;
    pub const HIT: u8 = 2;
    pub const INVULNERABLE: u8 = 3;
    /// **The frames the invulnerability still has**, which `0x14006a37e` onwards counts down while
    /// [`STATE`] is [`INVULNERABLE`] — `dec eax` at `0x14006a391` — and at nought writes [`NORMAL`] back
    /// over the state and clears this beside it.
    pub const UNTOUCHABLE: usize = 0x7858;
    /// **What the game gives a respawn**, and what orb writes under [`INVULNERABLE`] for one update:
    /// `0x14006a6cf` writes it two instructions after `0x14006a6b3` writes the state — the same `0xf0`
    /// th06c's respawn gives.
    pub const INVULNERABLE_FRAMES: i32 = 0xf0;
    // **And what the register leaves on that count, and how long states 1 and 2 last, are not constants
    // here**: nothing of orb's reads either — `0x78` at `0x140069b3b` and the `0x1e` of `0x14006a4b7`
    // and `0x14006a6a9` — so they are `asm.md`'s *The frames nothing can reach the player in* and the
    // numbers `the_player_a_stage_of_th06nc_starts` holds the build to, and no dead constant of this
    // module's.
    /// **The frames a bomb still takes the hit back in**: the hit writes 8 into it at `0x14006c20e`,
    /// and the laser's own hit the same at `0x14006c45b`. Taken one off a frame of the hit's state at
    /// `0x14006a21b`, and the frame it reaches nought is the frame the death is made final — the count at
    /// `0x14053cab0` stepped at `0x14006a22a`; `0x14006a0e2` is the bomb refusing to fire where it is
    /// nought.
    pub const A_BOMB_STILL_TAKES_IT_BACK: usize = 0x773c;
    /// **Whether a bomb is going off**, one byte: `0x14006a1a4` writes 1 as the bomb's own start runs —
    /// beside the `0x3e7` at `+0x9f20` and the `-999` at `+0x7ac0` — `0x140069e9e` clears it as the
    /// stage's register leaves the player, and the four ends at `0x14000d8fc`, `0x14000e2a7`,
    /// `0x14000e9ca` and `0x14000ef47`, one per shot, clear it as each bomb finishes.
    ///
    /// **And it is read as well as written**: `0x14006a0b8` is the press's own gate — no second bomb
    /// while one is out — and `0x14006b2d2`, `0x14006b54d`, `0x14006b7e7`, `0x14006b856` and
    /// `0x14006ba40` are the hit test's, a bomb out being a frame nothing lands on.
    ///
    /// **Not th06c's `+0x97d0`**, which is what a guess would have written: this game's player keeps its
    /// state at `+0x7898` where th06c's is at `+0x73b8` and its deathbomb window at `+0x773c` where
    /// th06c's is `+0x7254`, so the two layouts are near each other and equal nowhere.
    pub const BOMB_IN_USE: usize = 0x9ec8;

    /// **The shots the player has out**, eighty slots of [`SHOT_BYTES`] inside the player's own block:
    /// the update fills them from `0x14006b2f1`, walking `rbx` from `+0x410` with `add rbx,0x170` and
    /// `cmp r14d,0x50` and taking the first whose [`shot::IN_USE`] is nought, and their hit test against
    /// the enemies walks the same eighty at `0x14006b4d0`, from `+0x550` — which is that field of the
    /// first — adding each hit's own `+0x15c` into the damage it answers with.
    ///
    /// **And the stage's own register clears every one of them**: `0x140069d74`–`0x140069e3f` write
    /// nought into the [`shot::IN_USE`] of each slot in turn, eighty `mov WORD PTR [rbx+N],si` at that
    /// stride. So a stage carries none of its shots in, which is why they are read as a fingerprint of
    /// the frame rather than put into [`resume::Numbers`].
    pub const SHOTS: usize = 0x410;
    pub const SHOT_SLOTS: usize = 0x50;
    pub const SHOT_BYTES: usize = 0x170;

    /// The fields of one of those slots that say it apart from another.
    pub mod shot {
        /// **Whether the slot holds a shot**, a word: `0x14006b300`'s `cmp WORD PTR [rbx+0x10],0x0` is
        /// what the fill skips a slot in use on, and `0x14006b35f` writes 1 into the one it takes.
        pub const IN_USE: usize = 0x10;
        /// **Where the shot is**, two floats: the hit test reads them at `0x14006b4f9` and `0x14006b4fd`
        /// and holds each against the enemy's box grown by the half-sizes at `+0x160` and `+0x164`.
        pub const POSITION: usize = 0x13c;
    }

    /// **The phase of the shot the player is firing, and the phase the update before it had** — the pair
    /// `0x14006b38a` steps every update a shot is out: `[+0x40c]` up one and `[+0x408]` keeping what it
    /// was, with the walk of the eighty slots above taking the new value as the shot it emits and
    /// skipping altogether where the two are equal.
    ///
    /// **What starts one is the composed word's own `0x1`**: `0x14006ae14`'s `test r12b,r11b` — `r12d`
    /// being 1 from `0x14006a769` and `r11d` the word from `0x14006aa94`, with no call between — fires
    /// where the pair is idle and no conversation is up, writing `0` into [`SHOT_PHASE`] and `-999` into
    /// [`SHOT_PHASE_BEFORE`]. `0x14006b39e` ends the cycle at thirty and `0x14006b3b4` puts `-999` and
    /// `-1` back, which is what idle is, and the stage's own register writes the same pair at
    /// `0x140069e46`.
    ///
    /// **And this is the site `asm.md` had as not existing**: the reading that no site in the exe tests
    /// that word's `0x1` was taken over a window of eight instructions, and the test is eight hundred
    /// and sixty bytes past the load.
    pub const SHOT_PHASE: usize = 0x40c;
    pub const SHOT_PHASE_BEFORE: usize = 0x408;
}

/// **The pair one update into a firing cycle**, which is what a stage's own first frame leaves where the
/// word it was played with held the shot — and the value
/// [`Game::starts_the_shot_a_stage_began_with`](crate::game::Game::starts_the_shot_a_stage_began_with)
/// puts back for this game.
///
/// **Not the pair the press writes.** `0x14006ae30` writes `0` into [`player::SHOT_PHASE`] and `-999`
/// into [`player::SHOT_PHASE_BEFORE`] beside it, and `0x14006b38a`'s advance runs in the *same* update —
/// putting what the phase held into the phase before and the phase up one. So nothing outside that update
/// ever reads `(0, -999)`, and what a frame ends on is this.
///
/// **Measured**: the recording of a stage-3 pair wrote `shot=1,0` for the stage's frame 1 and `shot=2,1`
/// for its frame 2, and the playback of it wrote `shot=1,0` for frame 2 — one update behind, from there to
/// the end of the chapter.
const ONE_UPDATE_INTO_A_CYCLE: (i32, i32) = (1, 0);

/// **The rectangle the player is held to**, left, top, width and height — `0x14006aade`–`0x14006ab49`
/// puts the position back inside it, and the stage's own build writes `8, 16, 368, 416` into it at
/// `0x14003c2d9`. Read the way th06c's is, with [`FIELD_SIZE`] behind it where it reads as nought.
const PLAYER_AREA: usize = GAME_MANAGER + game_manager::PLAYER_AREA;
/// **The field's own size**, which this exe holds as the literals `384` and `448` at `0x140352d08` and
/// `0x140352d14` — loaded by the bullets' update at `0x140011b13` and the enemies' at `0x140038d3f`.
const FIELD_SIZE: (f32, f32) = (384.0, 448.0);

/// **The bullets**, a static of `0xf5010` bytes `0x14003ca4f` zeroes, whose update is `0x140010fa0` and
/// whose array that update's own loop says the shape of — see `th06nc/asm.md`.
const BULLET_MANAGER: usize = 0x1_4043_6ef0;

mod bullet_manager {
    pub const BULLETS: usize = 0x8;
    /// `cmp r14d,0x280` at `0x140011cc9`.
    pub const SLOTS: usize = 0x280;
    /// `add rbx,0x620` at `0x140011ca8`.
    pub const BYTES: usize = 0x620;
    /// Counted up per live one at `0x1400110ab`.
    pub const LIVE: usize = 0xf5008;
    /// Three floats, `0x140011191` onwards adding them into the position.
    pub const STEP: usize = 0x08;
    pub const POSITION: usize = 0x30;
    /// **Which of five states it is in**, a word — `cmp WORD PTR [rbx+0x44],r13w` at `0x1400110a0`, and
    /// nought is a slot nothing is in.
    pub const STATE: usize = 0x44;
    /// **And the only one the player is tested against**: the test at `0x140011b48` is on state 1's path
    /// and on no other.
    pub const FLYING: u16 = 1;
    /// **The size the game's own hit test is given**, `lea r8,[rbx+0x5f4]` at `0x140011b32`, whose
    /// smaller float is the circle the test takes.
    pub const SIZE: usize = 0x5f4;

    /// **What a bullet's own step is scaled by before it reaches its position**: states 2, 3 and 4 divide
    /// one by `2.0`, `2.5` and `3.0` — `0x140011138`, `0x14001114e`, `0x14001115f`, the floats at
    /// `0x140352ad0`, `0x140352ae8` and `0x140352b08` — state 1 adds the step whole, and state 5 adds half of
    /// it: `0x1400110d9` onwards takes the bullet's own `+0x8` (`rbp`, `lea` at 0x140011024) times `xmm15`,
    /// `0x140352a44`'s `0.5` (0x140011028), into its `+0x30` (`rsi`, 0x140011031). The same five th06c has.
    pub const fn a_steps_scale(state: u16) -> f32 {
        match state {
            2 | 5 => 1.0 / 2.0,
            3 => 1.0 / 2.5,
            4 => 1.0 / 3.0,
            _ => 1.0,
        }
    }
}

/// **The lasers**, in that same static right after the bullets' live count: the update walks sixty-four
/// of them from `0x140011cdc`, `0x298` apart, holding its pointer `0x14` into each — so the array starts
/// `0x10` past the count, the way th06c's does.
const LASERS: usize = BULLET_MANAGER + bullet_manager::LIVE + 0x10;

mod laser {
    /// `mov edi,0x40` at `0x140011ce9`.
    pub const SLOTS: usize = 0x40;
    /// `add rbx,0x298` at `0x140011e5f`.
    pub const BYTES: usize = 0x298;
    /// **The first fields are th06c's**, read off the update with its pointer `0x14` in: the width is
    /// `[rbx-0x10]`, the frames `[rbx-0x8]`, the near end `[rbx-0x4]` and the far end `[rbx]`.
    pub const WIDTH: usize = 0x4;
    pub const FRAMES: usize = 0xc;
    pub const NEAR: usize = 0x10;
    pub const FAR: usize = 0x14;
    /// **And the rest sit `0x20` further on than th06c's**: the angle `[rbx+0x24c]`, the growth
    /// `[rbx+0x258]`, in use `[rbx+0x260]`, the origin `[rbx+0x264]`, the length `[rbx+0x274]` and the
    /// phase `[rbx+0x27c]`.
    pub const ANGLE: usize = 0x260;
    pub const GROWS: usize = 0x26c;
    pub const IN_USE: usize = 0x274;
    pub const ORIGIN: usize = 0x278;
    pub const PHASE: usize = 0x290;
    /// **How many frames its warning runs for**, which the warning phase compares the frames with at
    /// `0x140011f5f` before it moves the phase on.
    pub const WARNS_FOR: usize = 0x270;
    /// Nought is the warning, 1 fires — the one phase whose branch calls the test, at `0x140011fb3` —
    /// and 2 fades.
    pub const FIRING: u8 = 1;
    pub const FADING: u8 = 2;
    /// **What a hit on it is, to either side of its own line**: a quarter of the width, the update
    /// halving it at `0x140011daa` into the size the test is handed and the test halving that again
    /// with `0x140352a44` at `0x14006c36a` — the two halvings th06c's has.
    pub const HALF_OF_ITS_WIDTH: f32 = 4.0;
}

/// **The object a conversation runs in**, behind the pointer at this static: the one the stage's own
/// asset load fills — `data/msg1.dat` through `msg7.dat`, chosen per stage at `0x14003f163` onwards — and
/// the one both the dialogue's start and everything that has to leave a line alone read.
///
/// **Read as a pointer and not as the object**, the way th06c's panel is: `0x140038748` loads it and asks
/// `+0x36b0` of what it holds, and the same object is reached as `[loader + 0x38]` at `0x1400401e8`.
const THE_DIALOGUE: usize = 0x1_40ab_ae28;

/// **How long that object is**, which is the number its own allocation is made with: `0x1400435a7` loads
/// `ecx` with `0x4228`, calls the CRT's `malloc` at `0x1402c01c0`, zeroes the block with `r8d` at the
/// same `0x4228` through `0x1402f18d0`, runs its constructor at `0x1400438d0` and stores what comes back
/// in [`THE_DIALOGUE`]. So the object is 16,936 bytes and never another size — the site is the only write
/// of that static in the exe, and it is skipped altogether while `[0x140c6dfbc]` is 3.
///
/// **What it is for is [`Game::live_handles`]**: a restore that puts this object back is what th06nc's
/// stage 2 could not survive — see `Th06nc::live_handles`, and the launches in this change's own
/// description.
const THE_CONVERSATIONS_BYTES: usize = 0x4228;

/// **Where the stage's own script begins**, which is what its cursor is counted from — see
/// [`enemy_manager::TIMELINE_CURSOR`], whose reading this address comes out of: `0x140037d1a` fills the
/// cursor from here on the frame a stage's script has none yet.
const THE_STAGES_TIMELINE: usize = 0x1_40ab_ad90;

mod dialogue {
    /// **Which line of the conversation is being shown**, and `-1` for none: the dialogue's own start
    /// writes the line it was asked for at `0x1400406c4`, `0x1400401f3` and `0x140040510` write `-1` as
    /// it ends, and the game itself reads it as a question of whether a line is up — `0x14006a0d6`
    /// refuses the bomb press while it is nought or more, `0x14006ae20` gates the player on the same
    /// compare and `0x140038757` the script's own side of it.
    pub const LINE: usize = 0x36b0;
}

/// **The enemies**, the static `0x140038ae9` hands the element at `0x14053c190` as what
/// `0x140038cd0` runs on.
const ENEMY_MANAGER: usize = 0x1_40ae_e0b0;

mod enemy_manager {
    /// `lea rbx,[r12+0x8]` at `0x140038d3a`.
    pub const ENEMIES: usize = 0x8;
    /// `cmp r13d,0x100` at `0x140039aba`.
    pub const SLOTS: usize = 0x100;
    /// `add rbx,0x10b0` at `0x140039aae`.
    pub const BYTES: usize = 0x10b0;
    /// Counted up per live one at `0x140038dcd`, into the manager's first dword.
    pub const LIVE: usize = 0x0;
    /// **The clock the stage's script runs on**, and the one clock a midstage boundary can be written in:
    /// it moves the same way however the run goes.
    ///
    /// **Read off the two ends of the enemies' own job.** `0x140038cd0` takes the manager in `rcx` and
    /// keeps it in `r12`, and its last four instructions before the epilogue are
    /// `mov ecx,[r12+0x10c0bc]`, `mov [r12+0x10c0b8],ecx`, `inc ecx`, `mov [r12+0x10c0bc],ecx` — so this
    /// field is stepped once a frame and `+0x10c0b8` is what it was the frame before, which
    /// `0x140001328` starts at `-1`. And it is the *script's* clock and not a frame count, because
    /// `0x140037d9b` reads it and `0x140037daa` compares it with the word at the timeline's cursor —
    /// `+0x10c128`, which `0x140037cf7` sets and whose own entry's time a `jne` skips the spawn on —
    /// and `0x140037d01` and `0x140038762` write the clock from the script itself.
    pub const TIMELINE_TIME: usize = 0x10_c0bc;
    /// **The entry of the stage's script the spawn test is against**, as a pointer into the timeline:
    /// `0x140037d0e` reads it, `0x140037d1a` fills it from [`super::THE_STAGES_TIMELINE`] where it is
    /// nought, `0x140037d2f` takes the `u16` at the entry as that entry's own time, and `0x140037daa`
    /// compares [`TIMELINE_TIME`] with it and skips the spawn where the two differ.
    pub const TIMELINE_CURSOR: usize = 0x10_c128;
    /// **Which card the fight is on**, written as the card starts — see [`super::CURRENT_CARD`], which
    /// is where the reading is.
    pub const CARD_NOW: usize = 0x10_c0d8;
    /// **And whether one is up at all**, the dword of which kind of card it is: `0x1400278a4` writes
    /// the kind into it as a card starts, `0x140027a7f` — the card's end, where its bonus is worked out
    /// — skips everything where it is nought, and `0x140027c76` writes nought into it as the card ends.
    ///
    /// **Not the byte at `+0x10_c0dc` beside it**, which `0x14002789d` sets as a card starts as well and
    /// which is whether the card can still be taken: the player's death at `0x14006c1c8` and
    /// `0x14006c40d` and the bomb at `0x14006a1cd` clear it, `0x140027ac0` pays the bonus only where it
    /// is still up, and nothing clears it as the card ends. Read as the card being up, it held a card up
    /// through every attack after it until a death — `spell=21` on a stage 3 frame of a run with no boss
    /// on the field — and the chapters refused the nonspell after each card, the card being still up.
    ///
    /// **Driven on the real game**: a probe recipe that waited on `card: true` had `State::spellcard`
    /// answer at stage frame 6,245 of the first stage, the stage boss's own card — `spell=6` on the
    /// frame's line — and a death inside it read `retry: attempt 2 at this spell card`, which is
    /// [`Game::count_card_attempt`]'s first measurement on this build.
    pub const A_CARD_IS_UP: usize = 0x10_c0d0;
}

mod enemy {
    /// `lea rsi,[rbx+0xb0]` at `0x140038dd1`, handed to the player's test as the position.
    pub const POSITION: usize = 0xb0;
    /// **The size the player's own test is given**, three floats multiplied by two thirds at
    /// `0x140039363` — `0x140352a60` — and halved by the test, so a hit is a third of this on each side.
    pub const SIZE: usize = 0x2c0;
    pub const A_THIRD_OF_ITS_SIZE: f32 = 3.0;
    /// The flags byte the walk gates on: `cmp BYTE PTR [rbx+0xbc],0; jge` at `0x140038dc0` skips a slot
    /// whose top bit is clear.
    pub const FLAGS: usize = 0xbc;
    pub const IN_USE: u8 = 0x80;
    /// **And whether its body is run against the player at all**: `0x140039312` requires `0x4`,
    /// `0x14003931a` refuses where `+0xbe`'s `0x8` is set, and `0x140039338` runs the test only where
    /// the low two bits are both up.
    pub const SOLIDITY: usize = 0xbd;
    pub const TESTED: u8 = 0x4;
    pub const SOLID_BITS: u8 = 0x3;
    /// **And which one the fight is with**: `0x1400396b0` and `0x1400398f0` test this bit and divide the
    /// life below into the bar the panel draws for the enemy carrying it.
    pub const IS_THE_BOSS: u8 = 0x8;
    /// Its life, which the body taking the player off it drops by ten at `0x1400393a3`.
    pub const LIFE: usize = 0x234;
    /// **The frames its attack has run**, the dword the panel's own countdown is worked out from:
    /// `0x140038870`, gated on [`IS_THE_BOSS`], takes it off the limit at `+0xc54` and divides by sixty, and
    /// ends the attack where the two meet — th06c's `0x1400202df` instruction for instruction, the limit at
    /// another offset. Put back to nought as the script moves the boss on, which is what marks the next
    /// attack, and what a card's own frames are counted by.
    pub const TIMER: usize = 0x4;
    pub const MORE_FLAGS: usize = 0xbe;
    pub const PASSES_THROUGH: u8 = 0x8;
}

/// **The items**, a static walked by `0x140044240` with the array at `+8` — `lea r14,[0x140bfb2f8]` at
/// `0x140044264`.
const ITEM_MANAGER: usize = 0x1_40bf_b2f0;

mod item {
    pub const ITEMS: usize = 0x8;
    /// `cmp eax,0x200` at `0x140044ddd`.
    pub const SLOTS: usize = 0x200;
    /// `imul rdi,rcx,0x160` at `0x140011b72`, where the bullets' update reaches into this array.
    pub const BYTES: usize = 0x160;
    /// A byte, and nought is a slot nothing is in — `cmp BYTE PTR [r14],bpl` at `0x1400443a0`.
    pub const ALIVE: usize = 0x0;
    pub const POSITION: usize = 0x10;
    /// **Which of seven kinds it is**, a signed byte `0x140044600` reads and jumps through the table at
    /// `0x140045168` on. The cases are th06c's in th06c's order: the power written at `0x14004461a`,
    /// the bombs stopped at eight at `0x140044c87`, the lives at `0x140044cc7`, the power written to
    /// its cap at `0x140044e61`.
    pub const KIND: usize = 0x34;
    pub const POWER: i8 = 0;
    pub const POINT: i8 = 1;
    pub const BIG_POWER: i8 = 2;
    pub const BOMB: i8 = 3;
    pub const FULL_POWER: i8 = 4;
    pub const LIFE: i8 = 5;
    pub const CANCELLED: i8 = 6;
}

/// How much of the 256-byte snapshot is a key: every index, which is what `GetKeyboardState` fills.
const KEYS_IN_A_SNAPSHOT: usize = 256;

/// **The 256 bytes the game plays a run off**, which its own read fills through the table below.
const THE_KEYS_THE_GAME_READS: usize = 0x1_4058_e3dc;

/// **When the game last read the keyboard, in `timeGetTime`'s own milliseconds** — and the whole of why
/// a read of this game is not one per update: `0x1400b0057` asks `timeGetTime` through `0x1405a7c60`,
/// `0x1400b005d` holds this against the answer and `0x1400b0063` **leaves the read out altogether where
/// the two are equal**, `0x1400b0069` writing the new time where they are not. So the 256 bytes above are
/// refilled at most once a millisecond however often the word is composed out of them.
const WHEN_THE_KEYBOARD_WAS_READ: usize = 0x1_4058_e3d8;

/// **And when the pad was**, the same shape one device along: `0x1400af107` asks the same clock,
/// `0x1400af10d` holds this against it and `0x1400af10f` leaves the poll out where they are equal. The
/// address is the first device's record — `0x14058e3d0 + 0x2b78`, the base the whole table is walked from
/// with `0x530` bytes a device — and the pad's half of the word is ORed into the keyboard's by
/// `0x140012f00`.
const WHEN_THE_PAD_WAS_READ: usize = 0x1_4059_0f48;

/// What orb writes into those two to make the next read a real one: a time `timeGetTime` cannot answer,
/// since the game skips its read where the two are *equal*. Nought is the whole of what that takes — the
/// clock is milliseconds since the machine started and never nought while a game of this is running.
const NO_READ_YET: u32 = 0;
const DOWN_IN_ITS_OWN_KEYS: u8 = 0x80;
const UP_IN_ITS_OWN_KEYS: u8 = 0x00;
/// **The game's own table from a virtual key to the index it keeps that key at**: six bytes an entry,
/// walked at `0x1400b00c0` and ending at a key of `0xffff`. `A` is `0x41 -> 0x1e`, which is `DIK_A`,
/// so the buffer is indexed by DirectInput's own scan codes — the same table th06c has at `0x1402bc0b0`.
const THE_TABLE_OF_KEYS: usize = 0x1_402f_6060;
const A_KEY_IN_THE_TABLE: usize = 6;
const KEYS_IN_THE_TABLE: usize = 0x82;
const NO_MORE_KEYS: u16 = 0xffff;

/// What a key that is down reads as in a `GetKeyboardState` snapshot: the high bit of the byte.
const DOWN: u8 = 0x80;
/// And what one that is not reads as, which is what [`Game::press_into`] writes for every button the word
/// does not hold — see th06c's, which has why they are written rather than left as the host's read left
/// them.
const UP: u8 = 0x00;

/// The pad's own packet number, which moves on every state orb hands over.
static PACKETS: AtomicU32 = AtomicU32::new(1);

/// What the log's own `run:` line carries: the stage, the difficulty, the scene, the lives and the bombs.
type Said = (i32, i32, i32, i8, i8);

/// What that line last said, so that [`Game::read_state`] says it once each time any of it changes.
static SAID: std::sync::Mutex<Option<Said>> = std::sync::Mutex::new(None);

/// Whether [`A_REPLAY_IS_PLAYING`] was set the last time [`Game::read_state`] looked.
static PLAYING_BACK: AtomicBool = AtomicBool::new(false);

/// **The bytes beside the playback flag, said once each time any of them changes while a playback is
/// running** — the instrument for the one thing `Game::read_state` still declines about a playback:
/// whether anything tells the game's own **demo** from a replay somebody asked for.
///
/// **What makes the question answerable at all is that this game plays a demo by itself**, which a launch
/// found: a run given up walks `front end: screen 16, 17, 18` and then, about thirty seconds later,
/// `20` and `26`, the front end leaves the chain, and a playback runs stage 1 and stage 2 with orb
/// keeping nothing of it — `resume: stage N of a run orb is not keeping`. So a launch can have both kinds
/// in it: a replay chosen through the main menu's item 4, and the demo that comes up if the walk is left
/// alone.
///
/// **The four bytes are the ones the machines around a run read**: `0x14053d400`, which the pause's own
/// reading corrected this to; `0x14053d404`, the practice; `0x14053d405`, which `0x14000bc20` reads as
/// *a scene out of the table at `0x140c6e3e0`* rather than a run, and so the likeliest of the four to be
/// what a demo is; and `0x14053d406`, which that same machine writes 1 into for such a scene. The
/// supervisor's wanted state goes beside them, a scene of the table's being reached through it.
static A_PLAYBACKS_OWN_BYTES: std::sync::Mutex<Option<PlaybackSeen>> = std::sync::Mutex::new(None);

/// The four bytes that line says and the supervisor's own state beside them.
type PlaybackSeen = (u8, u8, u8, u8, i32);

/// The four bytes above and the supervisor's own state, read while a playback is running.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn say_what_a_playback_is_told_by_when_it_changes() {
    if !unsafe { a_replay_is_playing() } {
        *A_PLAYBACKS_OWN_BYTES.lock().unwrap() = None;
        return;
    }
    // Raw, as every static of the image this file reads is — see
    // [`Th06nc::the_worlds_fingerprint`] for what a committed read of one costs and buys.
    let byte = |at: usize| unsafe { mem::read::<u8>(at) };
    let seen = (
        byte(at(0x1_4053_d400)),
        byte(at(A_RUN_IS_PRACTICE)),
        byte(at(0x1_4053_d405)),
        byte(at(0x1_4053_d406)),
        unsafe { mem::read::<i32>(at(SUPERVISOR + supervisor::WANTED)) },
    );
    if A_PLAYBACKS_OWN_BYTES.lock().unwrap().replace(seen) != Some(seen) {
        crate::detail!(
            "playback: 0x14053d400={} practice={} scene={} 0x14053d406={}, the supervisor wants {}",
            seen.0,
            seen.1,
            seen.2,
            seen.3,
            seen.4,
        );
    }
}

/// **How many walks of the calc chain this frame has had**, which [`Game::prepare_frame`] clears at the
/// top of one: what it decides is which of them are extra, and so how far the music has to be seeked to
/// be where the run is — see [`walks_the_calc_chain`].
static WALKS_THIS_FRAME: AtomicU32 = AtomicU32::new(0);

/// **Where the game keeps a virtual key in the 256 it plays a run off**, out of its own table — see
/// [`THE_TABLE_OF_KEYS`]. Nothing for a key the table has no entry for, and nothing for an index past
/// the buffer; the end of the table ends the walk.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
/// **The key this game's own table holds for a button orb presses**, which is the key itself for eight of
/// the nine.
///
/// **The focus is the ninth.** The table has `VK_LSHIFT` and `VK_RSHIFT` and no `VK_SHIFT`, so a focus
/// looked up by [`key::FOCUS`] finds no entry and [`Th06nc::presses_into_its_own_keys`] skips the one
/// button it cannot place: every launch of this game before this one played every frame of every run
/// unfocused, and nothing said so — a key the table has no entry for is the one thing that walk answers
/// nothing for.
///
/// **And [`key::FOCUS`] still stands for what a person presses**: `GetKeyboardState` fills its 256 bytes
/// by virtual key and `VK_SHIFT` is the one either shift sets there, which is the read
/// [`Th06nc::word_in`] makes. So the two are different keys in different spaces and neither stands for
/// the other — a single constant for both is what hid this.
const fn the_key_the_games_table_holds(key: u8) -> u8 {
    match key {
        key::FOCUS => key::FOCUS_IN_THE_GAMES_TABLE,
        key => key,
    }
}

unsafe fn the_index_of_the_key(key: u8) -> Option<usize> {
    let table = at(THE_TABLE_OF_KEYS);
    for entry in 0..KEYS_IN_THE_TABLE {
        let entry = table + entry * A_KEY_IN_THE_TABLE;
        let wanted = mem::read_committed::<u16>(entry)?;
        if wanted == NO_MORE_KEYS {
            return None;
        }
        if wanted != u16::from(key) {
            continue;
        }
        let index = usize::from(mem::read_committed::<u16>(entry + size_of::<u16>())?);
        return (index < KEYS_IN_A_SNAPSHOT).then_some(index);
    }
    None
}

/// **The front end**, or nothing while it is not in the chain — see [`THE_FRONT_ENDS_CALLBACK`], which
/// has why it is found by its job and not by a pointer.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_front_end() -> Option<usize> {
    unsafe { what_a_job_runs_on(THE_FRONT_ENDS_CALLBACK) }
}

/// **The object a job of the calc chain is run on**, found by the callback it registered — every step
/// asked of the memory map rather than read outright, a chain being a list the game is free to be
/// halfway through rebuilding.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn what_a_job_runs_on(callback: usize) -> Option<usize> {
    // The head of it is a static of the image and read raw; every step after it is a block the game
    // allocated — see [`Th06nc::the_worlds_fingerprint`], and the doc above for why the steps are asked.
    let mut element = unsafe { mem::read::<usize>(at(THE_CALC_CHAIN) + chain::NEXT) };
    for _ in 0..CHAIN_LINKS {
        if element == 0 {
            return None;
        }
        if mem::read_committed::<usize>(element + chain::CALLBACK)? == at(callback) {
            return mem::read_committed::<usize>(element + chain::ON).filter(|on| *on != 0);
        }
        element = mem::read_committed::<usize>(element + chain::NEXT)?;
    }
    None
}

/// **Whether the calc chain holds a job of its own** — apart from [`what_a_job_runs_on`] because this one
/// is about the job being there at all.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_chain_holds(callback: usize) -> bool {
    // The head raw and the steps asked, as [`what_a_job_runs_on`] reads it.
    let mut element = Some(unsafe { mem::read::<usize>(at(THE_CALC_CHAIN) + chain::NEXT) });
    for _ in 0..CHAIN_LINKS {
        let Some(at_element) = element.filter(|element| *element != 0) else {
            return false;
        };
        if mem::read_committed::<usize>(at_element + chain::CALLBACK) == Some(at(callback)) {
            return true;
        }
        element = mem::read_committed::<usize>(at_element + chain::NEXT);
    }
    false
}

/// **The screen a name is being typed into, or nothing where none is** — the result screen while its own
/// state is [`result_screen::TYPING_A_NAME`].
///
/// **The state as well as the screen, so that the ranking the *Score* item opens answers nothing**: that is
/// the same object with no run behind it, and a name written there would be one nobody's run was ranked
/// under.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_entry_a_name_is_typed_into() -> Option<usize> {
    let screen = unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }?;
    (mem::read_committed::<i32>(screen + result_screen::STATE)
        == Some(result_screen::TYPING_A_NAME))
    .then_some(screen)
}

/// **How many characters of the line being shown are out**, `+0x3420` of the conversation's own object.
///
/// **What a press has to wait for**: `0x140040d17` wants this at eight or more before an edge of `0x301`
/// moves the line on, and `0x140040d34` steps it by one a frame up to the line's own length at
/// `[record + 4]` — see `th06nc/asm.md`'s *What advances a line*.
mod dialogue_typing {
    pub const CHARACTERS_OUT: usize = 0x3420;
}

/// **The line and how far it has been typed, said the frame either changes.**
///
/// **An instrument, for the one thing about a conversation that is not read**: a run the rules drive
/// spends thousands of frames in one, where the bot's own release gives it a press edge a second and a
/// dozen lines should therefore cost a dozen seconds. What this line says is how many lines there are and
/// how long each of them stood, which is the difference between the two.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn say_the_line_being_typed_when_it_changes() {
    static WAS: std::sync::Mutex<Option<(i32, i32)>> = std::sync::Mutex::new(None);
    // The pointer itself is a static of the image and read raw; what it points at is a block the game
    // allocated, which is asked about — see [`Th06nc::the_worlds_fingerprint`].
    let object = unsafe { mem::read::<usize>(at(THE_DIALOGUE)) };
    let seen = if object == 0 {
        None
    } else {
        Some((
            mem::read_committed::<i32>(object + dialogue::LINE).unwrap_or(-1),
            mem::read_committed::<i32>(object + dialogue_typing::CHARACTERS_OUT).unwrap_or(-1),
        ))
    };
    let mut was = WAS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // The characters climb once a frame while a line is being typed, which is a line a frame in the log
    // and not what this is for: only the line itself is said, with the count it stood at beside it.
    let moved = match (*was, seen) {
        (Some((line, _)), Some((now, _))) => line != now,
        (was, now) => was.is_some() != now.is_some(),
    };
    let before = std::mem::replace(&mut *was, seen);
    if !moved {
        return;
    }
    drop(was);
    match (before, seen) {
        (Some((line, out)), Some((now, _))) => {
            crate::log!("dialogue: line {line} stood with {out} character(s) out; now {now}")
        }
        (None, Some((now, _))) => crate::log!("dialogue: a conversation begins on line {now}"),
        (Some((line, out)), None) => {
            crate::log!("dialogue: the conversation is gone, last line {line} with {out} out")
        }
        (None, None) => {}
    }
}

/// **The job the menu a scene's end puts up runs in**, whose object holds the two fields below — the
/// machine that builds the screen a run ends at straight into its list of replay slots, which a card of
/// the Spell Practice reaches as it ends and an ordinary run never does.
const RANKINGS_MACHINE_UPDATE: usize = 0x1_4000_9800;

/// **Where that machine's own state sits inside the job's object**, `+0x6e10` — the sub-object
/// `0x14000996a` hands to `0x14000c960`, the machine's calc.
const THE_RANKINGS_MACHINE: usize = 0x6e10;

/// **The frames its state has been up**, which `0x14000b5c0`, `0x14000c3e9` and `0x14000d046` step.
const RANKINGS_FRAMES: usize = 0xb4c;

/// **And the byte that says which end the scene came to**, `0x140430e18`: `0x14003be3a`, in the stage's
/// own job, writes 1 where the boss's life is gone and nought where the card's time ran out with the boss
/// still alive, and `0x14000bc6e` writes nought where the scene's player has run out of lives. What it
/// then picks is the menu's pictures: `0x14000ca51`'s `cmovne` hands `0x14000a6f0` nine where it is set
/// and twelve where it is not, a sprite number past `0x76b`, and `0x14000c9c9` picks `0x776` or `0x775`.
///
/// **Its two writes are rip-relative**, the machine's object being the static `0x140429270 + 0x6e10` —
/// which is why a grep for `+0xd98` off a register finds the four reads, `0x14000c9ae`, `0x14000c9c9`,
/// `0x14000ca51` and `0x14000d199`, and neither write.
const RANKINGS_PICK: usize = 0xd98;

/// **Those two said the frame either changes**, so that a launch says what the byte above is ever set by.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn say_the_rankings_machine_when_it_changes() {
    static WAS: std::sync::Mutex<Option<(i32, u8)>> = std::sync::Mutex::new(None);
    let Some(job) = (unsafe { what_a_job_runs_on(RANKINGS_MACHINE_UPDATE) }) else {
        *WAS.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        return;
    };
    let machine = job + THE_RANKINGS_MACHINE;
    let seen = (
        mem::read_committed::<i32>(machine + RANKINGS_FRAMES).unwrap_or(-1),
        mem::read_committed::<u8>(machine + RANKINGS_PICK).unwrap_or(0),
    );
    let mut was = WAS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if was.replace(seen) != Some(seen) {
        let (frames, pick) = seen;
        crate::log!("ranking: {frames} frame(s) up, picking on {pick:#x}");
    }
}

/// **The supervisor's own pair said the frame either changes** — the state it has acted on and the state
/// wanted, [`supervisor::STATE`] and [`supervisor::WANTED`].
///
/// **An instrument and not a reading orb acts on**: what a launch is asked here is which of the
/// supervisor's twelve states are ever reached at all. Each is read off the exe — **4**, the game
/// quitting, through a register: `0x14004d91f` writes `ebx` into `[0x140c6dfbc]` in the front end's
/// state 5, which the main menu's item 9 opens — and a launch is what says which of them a run reaches.
/// Nothing said it before this line: `State::wanted` is the front end's screen for this game and these
/// two are another machine's numbers. See `th06nc/asm.md`'s *What is not established*.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn say_the_supervisors_state_when_it_changes() {
    static WAS: std::sync::Mutex<Option<(i32, i32)>> = std::sync::Mutex::new(None);
    // Raw, both being statics of the image — see [`Th06nc::the_worlds_fingerprint`].
    let read = |at_| unsafe { mem::read::<i32>(at_) };
    let seen = (
        read(at(SUPERVISOR + supervisor::STATE)),
        read(at(SUPERVISOR + supervisor::WANTED)),
    );
    let mut was = WAS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if was.replace(seen) != Some(seen) {
        let (state, wanted) = seen;
        crate::log!("supervisor: state {state}, wanting {wanted}");
    }
}

/// The front end's own state, where it is in the chain.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_scene() -> Option<i32> {
    let front_end = unsafe { the_front_end() }?;
    mem::read_committed::<i32>(front_end + front_end::STATE)
}

/// **Every job the calc chain holds, said the frame the set of them changes** — the same instrument
/// th06c's screens were found with, since a screen this game keeps on the heap is reached through the
/// job that registers it.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn say_the_jobs_when_they_change() {
    static WAS: AtomicUsize = AtomicUsize::new(0);
    // The head raw and the steps asked, as [`what_a_job_runs_on`] reads it.
    let mut element = unsafe { mem::read::<usize>(at(THE_CALC_CHAIN) + chain::NEXT) };
    let mut jobs = [0usize; CHAIN_LINKS];
    let mut held = 0;
    let mut fingerprint = 0usize;
    for (link, job) in jobs.iter_mut().enumerate() {
        if element == 0 {
            break;
        }
        let Some(callback) = mem::read_committed::<usize>(element + chain::CALLBACK) else {
            break;
        };
        *job = callback;
        held = link + 1;
        fingerprint = fingerprint.wrapping_add(callback.rotate_left(link as u32 % 64));
        match mem::read_committed::<usize>(element + chain::NEXT) {
            Some(next) => element = next,
            None => break,
        }
    }
    if WAS.swap(fingerprint, Ordering::Relaxed) == fingerprint {
        return;
    }
    let slide = SLIDE.load(Ordering::Relaxed);
    let said = jobs[..held]
        .iter()
        .map(|callback| format!("{:#x}", callback.wrapping_sub(slide)))
        .collect::<Vec<_>>()
        .join(" ");
    crate::detail!("chain: {held} job(s): {said}");
}

/// Whether a stage is running to be steered in: a run on, and nothing held.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn in_a_stage() -> bool {
    let held = 0 != unsafe { mem::read::<u8>(at(EVERYTHING_HELD)) };
    let running = unsafe { in_a_run() };
    running && !held
}

/// The low byte of the sum of a name's own bytes, which is what a record holds beside it at
/// [`CATK_NAME_SUM`] — the game's own walk at `0x140027990` `movsx`es each byte into a 32-bit
/// accumulator, and the two agree on the one byte either reads.
fn name_sum(name: &[u8]) -> u8 {
    name.iter().fold(0, |sum, byte| sum.wrapping_add(*byte))
}

/// The name inside a record's [`CATK_NAME`] span: the bytes up to the first `0`, or `None` where there is
/// no `0` in the span at all — which a name the game copied there always has and one the generator left
/// only has by chance.
fn name_in(span: &[u8]) -> Option<&[u8]> {
    let end = span.iter().position(|byte| *byte == 0)?;
    Some(&span[..end])
}

/// The record the game keeps for a card, and `None` for a number that is not one of the [`CARDS`] or a
/// slot with no record in it at all.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn card_record(card: u32) -> Option<usize> {
    let card = usize::try_from(card).ok()?;
    if card >= CARDS {
        return None;
    }
    let record = at(CARD_HISTORY) + card * CARD_BYTES;
    (mem::read_committed::<u32>(record) == Some(CATK_MAGIC)).then_some(record)
}

/// Whether the game has named the card a record is for, asked the way the game asks it at `0x1400279a0`:
/// the sum the record holds against the name it holds.
///
/// # Safety
/// `record` must be one of [`CARD_HISTORY`]'s.
unsafe fn named(record: usize) -> bool {
    let span = mem::read_committed_bytes(record + CATK_NAME, CATK_NAME_BYTES).unwrap_or_default();
    name_in(&span).is_some_and(|name| {
        !name.is_empty()
            && mem::read_committed::<u8>(record + CATK_NAME_SUM) == Some(name_sum(name))
    })
}

/// **Which line of a conversation is up**, and negative for none — the dialogue object's own field,
/// behind the pointer at [`THE_DIALOGUE`], so a null is no conversation either. See [`dialogue::LINE`].
///
/// **And it is seen set on the real game**, which three launches of stage 1 had not managed: a Normal run
/// the rules drove read `stage=0 frames=5125 script=5093 … dialogue` and twenty more samples like it
/// across its first two stages, with the script clock standing still each time the line was up. What the
/// three before it ran into is that a line takes a *press* to move on and the rules hold the shot: the
/// first waited out 3,000 frames of `play_until: { dialogue: true }`, the second stood still and had its
/// run die, and the third stood still over `--clear` with `input=0x0000` for 2,000 frames. The reading
/// that explains all four is in `th06nc/asm.md`'s *What advances a line*: the game wants an edge of
/// `0x301` with at least eight characters of the line out, which a shot held every frame never gives.
///
/// **And this is what the rules read to press past one**, `sight::what_a_bot_sees` putting it into
/// `Frame::dialogue`: `orb_bot::steer` lets the shot go every
/// `SHOT_LET_GO_EVERY_IN_A_CONVERSATION` frames while a line is up instead of every
/// `SHOT_LET_GO_EVERY`, [`Th06nc::BUTTONS`] naming no skip for `Keys::skip` to find — the game's own
/// being `0x8` of the word, left `Ctrl` or `;` on the keyboard and X on a pad by default.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_line_of_dialogue() -> i32 {
    Some(unsafe { mem::read::<usize>(at(THE_DIALOGUE)) })
        .filter(|object| *object != 0)
        .and_then(|object| mem::read_committed::<i32>(object + dialogue::LINE))
        .unwrap_or(-1)
}

/// **Whether a stage is being played, whoever is playing it** — see th06c's `sight::in_a_run`, whose
/// reasoning this shares and whose demo term this has nothing to put in.
///
/// **A playback is one of it, and that is what makes a judging pass over this game possible at all.**
/// `runtime::step` asks for `during_replay && state.replay && state.in_run` together and
/// `Chapters::tracking` for `in_game || (during_replay && playing && !demo)`, so a reading that excluded
/// every playback left a pass with no chapters to step between — which is exactly where th06c stood until
/// its two readings were split. The split here is this function against [`State::in_game`], which
/// `read_state` answers `in_run && !playback`: this one and [`in_a_stage`] are a stage being played, and
/// `in_game` is a run of somebody's own.
///
/// **And the term th06c has and this build has not is its attract demo.** That game plays one through the
/// same stage a run uses, so a step bounded by a run ending would wait out 34,200 frames of a run nobody
/// was playing; here both callers of the replay's own start `0x14004ede0` are the front end's replay list
/// and **nothing starts a playback that nobody asked for**: a launch of
/// `the-front-end-after-a-run.yaml` left a run, watched the front end walk `16, 17, 18` and then sat on
/// 18 — the replay list — for 21,600 frames with nothing pressed, and wrote not one `resume: frame N
/// played back` line, its `scene=18->-1` and the left run's own `frames=9307` standing still throughout.
/// So there is no demo for this to exclude, and `read_state` answers `demo: false` for every playback.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn in_a_run() -> bool {
    unsafe { a_stage_is_the_scene() }
}

/// **Whether a stage is what the game is showing**: the player's job is in the calc chain and the stage
/// the run has reached is one of the seven — see [`THE_PLAYERS_CALLBACK`], and `th06nc/asm.md` for the
/// launch that said a stage is not a state of the front end's here but the front end's own absence.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_stage_is_the_scene() -> bool {
    let stage = unsafe { mem::read::<i32>(at(STAGE)) };
    (1..=STAGES).contains(&stage) && unsafe { the_chain_holds(THE_PLAYERS_CALLBACK) }
}

/// The stages a run of this game has, the Extra counted: `0x14003e669` treats six as the last of the
/// numbered ones, and a launch of the Extra read `stage 7 on difficulty 4` for the whole of it — the same
/// seventh th06c's is.
const STAGES: i32 = 7;
/// **The stage this game counts its own from**, which is one: every reading of
/// [`game_manager::STAGE`] hands the number over less this, so that `State::stage` is nought-counted the
/// way orb counts a stage everywhere.
///
/// **And nothing writes it back up**, which is the one thing about this game's field worth a line:
/// [`Game::start_stage`] writes the nought-counted number as it stands, because the stage's own build
/// counts that field on itself — `0x14003c7c0`, on both of its paths. A write that added this back came
/// up a stage too far, which a launch of `orb-real` read as
/// `resume: not put back — stage 3 came up where stage 2 was asked for`.
const STAGE_COUNTS_FROM: i32 = 1;
/// Nought for Easy up to four for the Extra — the difficulty state's own four items are 0 to 3
/// (`0x14004c872` onwards) and a launch of the Extra read `difficulty 4`.
const DIFFICULTY_EXTRA: i32 = 4;
/// Two characters of two shots each, indexed `character * 2 + shot` everywhere the game indexes them.
const SHOT_COMBINATIONS: i32 = 4;

/// **The run's own start**, `0x14004f320`, called once in the exe — `0x14004dc06`, from the last screen
/// before a stage with the front end in `rcx` and `[0x140c6e378]`'s `+0x34` reading 2. What it writes:
/// the lives and the bombs out of `th06.cfg`'s `[0x140c6dfd4]` and `[0x140c6dfd5]`, with `0x302` over
/// both at `0x14004f37c` (two lives and three bombs, the same word th06c's start writes) only where the
/// difficulty is 4 or the run is a practice, and `2` over the lives alone at `0x14004f373` where the
/// mode is Challenge; the supervisor's state wanted to 2 at `0x14004f38b`, the playback flag at
/// `0x14053d3dc` cleared, the track put to `-1` with its playing byte cleared (`0x14004f3b3` on), the
/// extra walks at `0x1405542e0` cleared, and the byte at `0x14053d404` written from what it was handed —
/// which is the byte `0x14007bdde` reads to choose between the name entry and the way out. What orb
/// calls where it writes a run into being — [`Th06nc::start_a_run`].
const RUN_START: usize = 0x1_4004_f320;

/// **The byte a run is a stage practised on its own by**, which [`RUN_START`] writes from what the front
/// end handed it and `0x14007bdde` reads to open the way out instead of the name entry.
///
/// **It is the practice because a picture says so.** The main menu's item 2 is the only one that writes
/// `[front end + 0x168d1]` to 1, which is what a run's start copies here — and a launch that pressed that
/// item and held the screen after it for a minute came up on screen 9, the difficulty select, with
/// **`PRACTICE`** across the bottom of it. See `th06nc/asm.md`'s *The main menu's ten items*, where the
/// four banners that screen chooses between are read.
///
/// **And a run asked for no name is the same run**: a stage practised on its own is scored by nobody, so
/// the byte that skips the name entry and the byte that says practice are one byte and not two.
const A_RUN_IS_PRACTICE: usize = 0x1_4053_d404;

/// **And the byte a *scene* is played under**, which is this build's **Spell Practice**: the main menu's
/// item 3 walks screens 27 and 28, and a picture of 28 reads `Spell Practice` over a list of the cards
/// stage by stage. `0x14004ee53` sets this byte for a replay whose header's `+0x6` is 3 and reads that
/// scene's own stage and difficulty out of a `0x28`-byte entry of the table at `0x140c6e3e0`;
/// `0x14004f35c` clears it as an ordinary run starts; and the card's own start steps a count of the
/// scene's own — sixteen dwords at `+0xe0` of the card record, one per difficulty and shot — instead of
/// the attempts while it is set.
const A_SCENE_IS_BEING_PLAYED: usize = 0x1_4053_d405;

/// **Whether the run is in チャレンジモード rather than 標準モード**, which is the game's own mode and
/// what `0x140075137` draws `Standard` or `Challenge` from.
///
/// **A run's own copy, written by screen 25 out of the configuration**: `0x14004f348` is a `setne` off
/// `[`[`THE_MODE_LAST_CHOSEN`]`]`, so any non-zero there is Challenge — and what it moves is the numbers
/// a run begins with, `0x14004f373` writing two lives where it is set. See `th06nc/asm.md`'s
/// *`0x14053d414`, the byte that changes what a run starts with*.
const A_RUN_IS_IN_CHALLENGE_MODE: usize = 0x1_4053_d414;

/// **And where that choice lives until a run starts**: the configuration's own byte, `th06.cfg`'s
/// `0x17` — the object's `+0x44b`.
///
/// **Screen 14 writes it on both of its edges**, `0x14004d553` and `0x14004d55c` on the cancel's and
/// `0x14004d65f` and `0x14004d668` on the decide's, all four inside that screen's own case
/// `0x14004d44d`..`0x14004d6c2`. So the mode is settled from screen 14 onwards, which is what lets
/// [`Game::run_pointed_at`] answer it at screen 25 as well as [`Game::run_start`] answering it inside a
/// run.
const THE_MODE_LAST_CHOSEN: usize = 0x1_40c6_dfdb;
/// **The lives and the bombs `th06.cfg` gives a run**, the two bytes `0x14004f32b` and `0x14004f341` read.
#[allow(
    dead_code,
    reason = "read by `image`, which is behind `feature = \"sim\"`"
)]
const THE_CONFIGURED_LIVES: usize = 0x1_40c6_dfd4;
#[allow(
    dead_code,
    reason = "read by `image`, which is behind `feature = \"sim\"`"
)]
const THE_CONFIGURED_BOMBS: usize = 0x1_40c6_dfd5;

/// **What a Challenge run's slot is called by**, the word in front of the name a Standard run's slot is —
/// see [`Game::run_slot`], which is where the two are told apart. The game's own English for the mode,
/// which is what `0x140075137` draws over the difficulty.
const CHALLENGE: &str = "challenge";

/// **Which of the two modes a run is in, as [`RunStart::mode`] counts them**: nought for 標準モード and
/// one for チャレンジモード.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_mode_a_run_is_in() -> i32 {
    i32::from(mem::read_committed::<u8>(at(A_RUN_IS_IN_CHALLENGE_MODE)).unwrap_or(0) != 0)
}

/// **And the mode a run *about to start* will be in**, read out of the configuration instead — see
/// [`THE_MODE_LAST_CHOSEN`].
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_mode_the_next_run_is_in() -> i32 {
    i32::from(mem::read_committed::<u8>(at(THE_MODE_LAST_CHOSEN)).unwrap_or(0) != 0)
}

/// **Whether the run is one stage of the game or less of it**, which is either of the two bytes the main
/// menu's own items write: item 2's stage practice at [`A_RUN_IS_PRACTICE`] and item 3's spell practice at
/// [`A_SCENE_IS_BEING_PLAYED`].
///
/// **Both because both are declined the same way.** [`Game::run_slot`] names neither, so nothing of such a
/// run is written down: a card practised on its own is no run whose chapters a run of the whole game could
/// be put back into, and a slot shared with one would hand a full run the practice's own file.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn a_run_is_practice() -> bool {
    [A_RUN_IS_PRACTICE, A_SCENE_IS_BEING_PLAYED]
        .into_iter()
        .any(|byte| 0 != unsafe { mem::read::<u8>(at(byte)) })
}

/// # Safety
/// Must run on the game's main thread.
unsafe fn a_replay_is_playing() -> bool {
    0 != unsafe { mem::read::<u8>(at(A_REPLAY_IS_PLAYING)) }
}

/// **Whether the run is stopped at one of the two menus it stops for** — the pause menu and the menu a
/// run out of lives is asked to continue from, see [`game_manager::IS_IN_GAME_MENU`]. A frame with
/// either up is a frame nothing of the run moved in.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_run_is_stopped_at_a_menu() -> bool {
    [
        GAME_MANAGER + game_manager::IS_IN_GAME_MENU,
        GAME_MANAGER + game_manager::IS_IN_RETRY_MENU,
    ]
    .into_iter()
    .any(|menu| 0 != unsafe { mem::read::<u8>(at(menu)) })
}

/// **The calc chain walked, and its answer turned into one the frame loop reads.**
///
/// The walk answers a byte — nought to carry the frame on and anything else to leave it — the same as
/// th06c's, and writes what it leaves with through the pointer it is handed.
///
/// **And the frame's own word put into the game's own keys here**, this being the last moment before the
/// walk that reads them — see [`Game::presses_into_its_own_keys`] and `runtime::the_frames_word`, which is
/// this game's `get_input`.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn walks_the_calc_chain(_: *mut c_void) -> i32 {
    // **Every walk of a frame after the first counted where the game counts its own**, which is what
    // puts the music back in step with a run that has gone by faster than it — see
    // [`tops_up_the_sound`].
    if WALKS_THIS_FRAME.fetch_add(1, Ordering::Relaxed) > 0 {
        let walks = mem::read_committed::<i32>(at(THE_EXTRA_WALKS)).unwrap_or(0);
        unsafe { mem::write::<i32>(at(THE_EXTRA_WALKS), walks.saturating_add(1)) };
    }
    // **The frame's own word and not the bot's own**: a chapter being played back is fed its recorded
    // buttons here, a bot's word goes through the hold a question of orb's is raised on, and either is
    // noted against the frame.
    //
    // **`runtime::pressing` here is what six launches of the real game were spent on.** The bot's word
    // went from the recipe into the game's keys and through nothing else, so `held_back` was never asked
    // on a word with the decide down: every press reached the title menu's own item, `menu:` never once
    // in the log, and the mode question orb raises on that read never stood. What said so was the third
    // instrument — a log line written where `held_back` holds a decide down — staying silent through a
    // launch whose hold was on for 766 frames of the menu. Nothing was written down either, the same
    // read being where a frame's buttons are noted, so a chapter had nothing to feed back.
    if let Some(word) = unsafe { crate::runtime::the_frames_word(&Th06nc) } {
        unsafe { Th06nc.presses_into_its_own_keys(word) };
    }
    let walk: unsafe extern "win64" fn(*mut i32) -> u8 =
        unsafe { std::mem::transmute(run_calc_chain()) };
    let mut leaving_with = 0i32;
    match unsafe { walk(&mut leaving_with) } {
        0 => CARRY_THE_FRAME_ON,
        _ => crate::runtime::CHAIN_EXIT_SUCCESS,
    }
}

/// What this game's calc walk answers where the frame is to carry on, in the frame loop's own terms:
/// anything that is neither `CHAIN_EXIT_SUCCESS` nor `CHAIN_EXIT_ERROR`.
const CARRY_THE_FRAME_ON: i32 = 1;

/// **What a draw job answers, and what the walk does about it** — read off `0x14003da92`..`0x14003dae5`.
mod walked {
    /// Call it again where it stands.
    pub const AGAIN: i32 = 2;
    /// Take the element out of the chain and carry on with the next.
    pub const TAKE_IT_OUT: i32 = 0;
    /// The walk is over.
    pub const OVER: [i32; 3] = [3, 4, 5];
}

/// **The draw, as the frame makes it inline** — call for call what [`BEGINS_THE_DRAW`] names, the draw
/// chain walked between the viewport and the screenshot, and the `-1` into the renderer the frame writes
/// after it. Nothing it does leaves the frame, that being the calc chain's business, so this always
/// answers the loop's carry on.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn walks_the_draw(_: *mut c_void) -> i32 {
    let begins: unsafe extern "win64" fn() = unsafe { std::mem::transmute(begins_the_draw()) };
    unsafe { begins() };
    unsafe { moves_the_camera_as_the_frame_does() };
    let reads: unsafe extern "win64" fn(*mut i32, *mut i32, *mut i32, *mut i32) =
        unsafe { std::mem::transmute(reads_the_viewport()) };
    let sets: unsafe extern "win64" fn(i32, i32, i32, i32) =
        unsafe { std::mem::transmute(sets_the_viewport()) };
    let clears: unsafe extern "win64" fn(i32) = unsafe { std::mem::transmute(clears()) };
    let mut viewport = [0i32; 4];
    let [x, y, width, height] = &mut viewport;
    unsafe { reads(x, y, width, height) };
    let cleared = |at: usize| i32::from(mem::read_committed::<u8>(at).unwrap_or(0));
    let what = at(WHAT_IS_CLEARED);
    unsafe {
        sets(
            cleared(what + 2),
            cleared(what + 1),
            cleared(what),
            cleared(what + 3),
        )
    };
    unsafe { clears(0) };
    let [x, y, width, height] = viewport;
    unsafe { sets(x, y, width, height) };
    unsafe { walks_the_draw_chain() };
    let screenshot: unsafe extern "win64" fn() =
        unsafe { std::mem::transmute(takes_the_screenshot()) };
    unsafe { screenshot() };
    if let Some(renderer) =
        mem::read_committed::<usize>(at(THE_RENDERER)).filter(|renderer| *renderer != 0)
    {
        unsafe { mem::write::<i32>(renderer + WHAT_THE_FRAME_LEAVES_IN_IT, -1) };
    }
    CARRY_THE_FRAME_ON
}

/// **The camera, as the frame puts it in place** — see [`MOVES_THE_CAMERA`].
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn moves_the_camera_as_the_frame_does() {
    let camera: unsafe extern "win64" fn(usize, f32, f32) =
        unsafe { std::mem::transmute(moves_the_camera()) };
    let float = mem::read_committed::<f32>(at(THE_CAMERAS_FLOAT)).unwrap_or(0.0);
    unsafe { camera(at(THE_GAME), 0.0, float) };
}

/// **The draw chain walked**, as `0x14003da73` onwards walks it: every element's callback with the
/// element's own argument, **the sprites it queued drawn before the answer is acted on** — see
/// [`flushes_what_the_job_queued`] — and what it answered saying what the walk does next.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn walks_the_draw_chain() {
    let mut element = at(THE_DRAW_CHAIN);
    'walk: while element != 0 {
        let callback = mem::read_committed::<usize>(element + chain::CALLBACK).unwrap_or(0);
        if callback == 0 {
            element = mem::read_committed::<usize>(element + chain::NEXT).unwrap_or(0);
            continue;
        }
        let on = mem::read_committed::<usize>(element + chain::ON).unwrap_or(0);
        let run: unsafe extern "win64" fn(usize) -> i32 = unsafe { std::mem::transmute(callback) };
        loop {
            let answered = unsafe { run(on) };
            if answered == walked::AGAIN {
                continue;
            }
            unsafe { flushes_what_the_job_queued() };
            let next = mem::read_committed::<usize>(element + chain::NEXT).unwrap_or(0);
            if answered == walked::TAKE_IT_OUT {
                let out: unsafe extern "win64" fn(usize, usize) =
                    unsafe { std::mem::transmute(takes_an_element_out()) };
                unsafe { out(0, element) };
                element = next;
                continue 'walk;
            }
            if walked::OVER.contains(&answered) {
                break 'walk;
            }
            element = next;
            continue 'walk;
        }
    }
}

/// **The sprites a job of the draw chain queued, drawn as the job is done** — the renderer's batch
/// flushed where it holds anything, and its count put back to nought, as `0x14003da97`..`0x14003dace` do.
///
/// **After every job and not once after the walk**, because the next job sets state of its own before it
/// queues: a batch carried over is drawn under whichever job comes next, and one left after the last is
/// drawn by the next frame's camera, a frame late.
///
/// # Safety
/// Must run on the game's main thread, inside a walk of the draw chain.
unsafe fn flushes_what_the_job_queued() {
    let Some(renderer) =
        mem::read_committed::<usize>(at(THE_RENDERER)).filter(|renderer| *renderer != 0)
    else {
        return;
    };
    let queued = mem::read_committed::<i32>(renderer + renderer::QUEUED).unwrap_or(0);
    if queued <= 0 {
        return;
    }
    let flushed_as = mem::read_committed::<i32>(renderer + renderer::FLUSHED_AS).unwrap_or(0);
    let flush: unsafe extern "win64" fn(usize, i32, i32, i32, i32) =
        unsafe { std::mem::transmute(flushes_the_sprites()) };
    unsafe { flush(renderer + renderer::SPRITES, queued, 4, flushed_as, 1) };
    unsafe { mem::write::<i32>(renderer + renderer::QUEUED, 0) };
}

/// **The sound's own catch-up, as the loop between the calc walk and the draw has it** — th06c's shape
/// with this game's numbers: [`THE_EXTRA_WALKS`] turned into a seek of [`SAMPLES_A_WALK`] apiece and
/// cleared, held inside the track's own loop where it has one and under its length where it does not.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn tops_up_the_sound(_: usize) {
    let walks = i64::from(mem::read_committed::<i32>(at(THE_EXTRA_WALKS)).unwrap_or(0));
    if walks <= 0 {
        return;
    }
    unsafe { mem::write::<i32>(at(THE_EXTRA_WALKS), 0) };
    let track = mem::read_committed::<i32>(at(THE_TRACK_PLAYING)).unwrap_or(-1);
    if track == -1 || mem::read_committed::<u8>(at(THE_TRACK_IS_PLAYING)) == Some(0) {
        return;
    }
    let playing: unsafe extern "win64" fn() -> i64 =
        unsafe { std::mem::transmute(where_the_music_is()) };
    let playing = unsafe { playing() };
    if playing < 0 {
        return;
    }
    let mut seek = SAMPLES_A_WALK * walks + playing;
    match mem::read_committed::<u8>(at(THE_TRACK_LOOPS)) {
        Some(0) | None => {
            let length: unsafe extern "win64" fn(i32) -> i64 =
                unsafe { std::mem::transmute(how_long_the_track_is()) };
            let length = unsafe { length(track) };
            if length < 0 || seek >= length {
                return;
            }
        }
        Some(_) => {
            let ends = mem::read_committed::<i64>(at(THE_LOOP_ENDS)).unwrap_or(0);
            let starts = mem::read_committed::<i64>(at(THE_LOOP_STARTS)).unwrap_or(0);
            if ends > starts {
                while seek >= ends {
                    seek -= ends - starts;
                }
            }
        }
    }
    let seeks: unsafe extern "win64" fn(i64, i32) =
        unsafe { std::mem::transmute(seeks_the_music()) };
    unsafe { seeks(seek, track) };
}

/// **The present orb's own frame loop makes**, which is `dxgi`'s: the swapchain this game makes is taken
/// as it is made, through the factory orb answers `CreateDXGIFactory2` with.
///
/// # Safety
/// Must run on the game's main thread, with everything of the frame drawn.
unsafe extern "win64" fn presents(_: usize) {
    unsafe { crate::dxgi::present() };
}

pub struct Th06nc;

impl Th06nc {
    /// **The frame's own numbers read out of the game**, which is what [`Reproduction`] is a line of —
    /// see `Game::reproduction`, the one thing that asks.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the exe's base already told.
    unsafe fn reproduced() -> Reproduction {
        let manager = at(GAME_MANAGER);
        let player = at(PLAYER) + player::POSITION;
        let area = at(PLAYER_AREA);
        // Raw, for the reason [`Th06nc::the_worlds_fingerprint`] gives: 512 committed reads is 512
        // `VirtualQuery` calls about a static of the exe's own image, on every frame of a run.
        let items = (0..item::SLOTS)
            .filter(|slot| {
                0 != unsafe {
                    mem::read::<u8>(
                        at(ITEM_MANAGER) + item::ITEMS + slot * item::BYTES + item::ALIVE,
                    )
                }
            })
            .count() as u32;
        // And every field beside them raw for the same reason, each being a static of the image too: the
        // input the run acts on, the player, the play field, the generator's pair, the score and the extra
        // lives — which is what the line this builds is written once a frame from.
        Reproduction {
            input: unsafe { mem::read::<u32>(at(INPUT_THE_RUN_ACTS_ON)) } as u16,
            player: (unsafe { mem::read::<f32>(player) }, unsafe {
                mem::read::<f32>(player + size_of::<f32>())
            }),
            player_area: (
                unsafe { mem::read::<f32>(area + size_of::<f32>()) },
                unsafe { mem::read::<f32>(area + 3 * size_of::<f32>()) },
            ),
            randoms: unsafe { mem::read::<u32>(at(RNG_COUNT)) },
            seed: unsafe { mem::read::<u16>(at(RNG_SEED)) },
            items,
            score: unsafe { mem::read::<u64>(manager + game_manager::SCORE) },
            extra_lives: unsafe { mem::read::<i8>(manager + game_manager::EXTRA_LIVES) },
            enemies: unsafe { mem::read::<i32>(at(ENEMY_MANAGER + enemy_manager::LIVE)) },
            timeline_cursor: {
                let cursor = unsafe {
                    mem::read::<usize>(at(ENEMY_MANAGER + enemy_manager::TIMELINE_CURSOR))
                };
                let timeline = unsafe { mem::read::<usize>(at(THE_STAGES_TIMELINE)) };
                match (cursor, timeline) {
                    (0, _) | (_, 0) => -1,
                    (cursor, timeline) => cursor.wrapping_sub(timeline) as i64,
                }
            },
            shot_phase: (
                unsafe { mem::read::<i32>(at(PLAYER + player::SHOT_PHASE)) },
                unsafe { mem::read::<i32>(at(PLAYER + player::SHOT_PHASE_BEFORE)) },
            ),
        }
    }

    /// The buttons orb's word has and the key each is pressed with, in one place so that
    /// [`Game::button_bit`] and [`Game::press_into`] cannot disagree about one.
    const BUTTONS: &'static [(&'static str, u16, u8)] = &[
        ("shot", button::SHOT, key::SHOT),
        ("bomb", button::BOMB, key::BOMB),
        ("focus", button::FOCUS, key::FOCUS),
        ("menu", button::MENU, key::MENU),
        ("decide", button::DECIDE, key::DECIDE),
        ("up", button::UP, key::UP),
        ("down", button::DOWN, key::DOWN),
        ("left", button::LEFT, key::LEFT),
        ("right", button::RIGHT, key::RIGHT),
    ];

    /// **The numbers a stage starts with, read where the stage's own build leaves them** — the eight
    /// fields of [`GAME_MANAGER`] its stage-following path at `0x14003c78d` does not write, and the seed
    /// out of the copy that path's common tail makes at `0x14003c9f1`.
    ///
    /// **The seed out of the game manager's copy and not the generator**, for th06's reason: the copy is
    /// what the stage started from once the stage has drawn from the generator.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn numbers(&self) -> resume::Numbers {
        let manager = at(GAME_MANAGER);
        resume::Numbers {
            score: mem::read_committed::<u64>(manager + game_manager::SCORE).unwrap_or(0),
            seed: mem::read_committed::<u16>(manager + game_manager::RANDOM_SEED).unwrap_or(0),
            point_items: mem::read_committed::<u16>(manager + game_manager::POINT_ITEMS)
                .unwrap_or(0),
            power: mem::read_committed::<u16>(manager + game_manager::POWER).unwrap_or(0),
            point_item_value: mem::read_committed::<u8>(manager + game_manager::POINT_ITEM_VALUE)
                .unwrap_or(0),
            lives: mem::read_committed::<i8>(manager + game_manager::LIVES).unwrap_or(0),
            bombs: mem::read_committed::<i8>(manager + game_manager::BOMBS).unwrap_or(0),
            extra_lives: mem::read_committed::<i8>(manager + game_manager::EXTRA_LIVES)
                .unwrap_or(0),
            deaths: DEATHS.load(Ordering::Relaxed),
        }
    }

    /// **And back, field for field** — all but the seed, which is [`Th06nc::seed`]'s and has to be
    /// earlier.
    ///
    /// **The score into the one field it is in**, which is [`game_manager::SCORE`] and was measured
    /// rather than read: the qword at `+0x938` is nought across a stage boundary, so a run's score is not
    /// there and nothing here writes it. Written into both, this put a run's score back and then a
    /// nought beside it, which cost nothing while it stood but said the pair was orb's business.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage built and not yet updated.
    unsafe fn put_the_numbers_back(&self, numbers: &resume::Numbers) {
        let manager = at(GAME_MANAGER);
        unsafe {
            mem::write::<u64>(manager + game_manager::SCORE, numbers.score);
            mem::write::<u16>(manager + game_manager::POINT_ITEMS, numbers.point_items);
            mem::write::<u16>(manager + game_manager::POWER, numbers.power);
            mem::write::<u8>(
                manager + game_manager::POINT_ITEM_VALUE,
                numbers.point_item_value,
            );
            mem::write::<i8>(manager + game_manager::LIVES, numbers.lives);
            mem::write::<i8>(manager + game_manager::BOMBS, numbers.bombs);
            mem::write::<i8>(manager + game_manager::EXTRA_LIVES, numbers.extra_lives);
        }
        // **And orb's own count, which is not the game's to be written** — see [`DEATHS`].
        DEATHS.store(numbers.deaths, Ordering::Relaxed);
    }

    /// **The generator and the game manager's copy of it both, and the count of draws back to nought** —
    /// the three the build itself writes at `0x14003c9f1` and `0x14003c9f8`, made here from the seam
    /// inside it so that they land after that copy rather than twenty thousand draws before it. See
    /// [`STAGE_SEEDED`].
    ///
    /// # Safety
    /// Must run on the game's main thread, from the hook over [`Hooks::stage_building`], before the
    /// original.
    unsafe fn seed(&self, seed: u16) {
        unsafe {
            mem::write::<u16>(at(RNG_SEED), seed);
            mem::write::<u16>(at(GAME_MANAGER + game_manager::RANDOM_SEED), seed);
            mem::write::<u32>(at(RNG_COUNT), 0);
        }
    }
}

impl Game for Th06nc {
    /// Kept as the distance from [`IMAGE_BASE`] — see [`at`].
    fn set_image_base(&self, base: usize) {
        SLIDE.store(base.wrapping_sub(IMAGE_BASE), Ordering::Relaxed);
    }

    /// **The DirectInput keyboard taken away, so that the keys orb writes are the keys a run is played
    /// with** — see [`DEVICE`]. Asked every frame, the game remaking the device when its window is
    /// activated again.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn acquire_input(&self) -> bool {
        if mem::read_committed::<usize>(at(DEVICE)) == Some(0) {
            return true;
        }
        unsafe { mem::write::<usize>(at(DEVICE), 0) };
        true
    }

    /// **The frame, the two walks, the keyboard read and the four names a launch that plays itself
    /// answers.** The frame is stood in whole — see [`THE_FRAME`] — and the walks are handed over the way
    /// th06c's are, nothing of the game's calling either once the frame is orb's.
    fn hooks(&self) -> Hooks {
        Hooks {
            update: None,
            draw: None,
            walks: Some(Walks {
                update: walks_the_calc_chain as *const () as usize,
                draw: walks_the_draw as *const () as usize,
            }),
            // The one convention a 64-bit game has, and nothing patched for it to matter to: the walks are
            // handed over above.
            frame_entered: FrameEntered::InEcx,
            runs_its_frame: None,
            save_replay: None,
            stop_recording: None,
            // **The two moments a stage's numbers go in at**, which are th06c's two under this game's own
            // addresses: the stage's own build, entered with the game manager in `rcx` the way the chain
            // calls the element's `REGISTERED` callback, and the job registered inside it three
            // instructions past the seed's copy — see [`STAGE_BUILD`] and [`STAGE_SEEDED`], which have why
            // the seed's seam is inside the build and not in front of it.
            stage_begun: Some(Entered::InEcx(Patch {
                target: at(STAGE_BUILD),
                prologue: STAGE_BUILD_PROLOGUE,
            })),
            stage_building: Some(Building::Cdecl(Patch {
                target: at(STAGE_SEEDED),
                prologue: STAGE_SEEDED_PROLOGUE,
            })),
            unlocks_read: None,
            ranking_read: None,
            card_records_read: None,
            create_window: None,
            init_device: None,
            // **The whole frame, which is what makes it orb's**: `main`'s loop calls this once round and
            // orb's own loop stands in its place, calling every piece of it — see [`THE_FRAME`], and
            // `Game::prepare_frame`, `walks_the_draw`, `Game::end_drawing` and `Th06nc::frame_calls`.
            render: Some(Patch {
                target: at(THE_FRAME),
                prologue: THE_FRAME_PROLOGUE,
            }),
            input: Some(Read::LookedUp(Lookup {
                name: KEYBOARD_READ,
            })),
            joystick: None,
            tops_up_the_music: None,
            stops_the_music: None,
            pad_buttons: None,
            // **None: this exe puts no dialog up.** No `DialogBoxIndirectParamW` is in its import table
            // and no name of a dialog control is in its strings — what window it makes is `th06.cfg`'s.
            startup_dialog: None,
            makes_its_window: Some(Lookup { name: WINDOW_MADE }),
            moves_its_window: Some(Lookup { name: WINDOW_MOVED }),
            shows_its_window: Some(Lookup { name: WINDOW_SHOWN }),
            places_its_window: Some(Lookup {
                name: WINDOW_PLACED,
            }),
            restyles_its_window: Some(Lookup {
                name: WINDOW_RESTYLED,
            }),
            reads_a_pad: Some(Lookup { name: PAD_READ }),
            makes_its_renderer: Some(Lookup {
                name: RENDERER_MADE,
            }),
            // **What the CRT linked into this exe reaches the OS through, as th06c's does**: `objdump -p`
            // lists `steam_api64.dll`, `KERNEL32.dll`, `USER32.dll` and `ole32.dll` and nothing of the
            // universal CRT's; of `KERNEL32.dll` it takes `GetProcessHeap` — asked once, at `0x1402e72d4`,
            // and kept at `0x140abac18` — and no `HeapCreate` or `VirtualAlloc`, `CreateThread`, and
            // `CreateFileW`. All three are installed in front of now that this game rewinds: the heap's is
            // what a chapter's copy of the heap is the game's own through, the thread entry is what says
            // which thread a run is on, and the file open is what the mode's fork of the score file
            // stands in front of.
            takes_its_memory: Memory::ProcessHeapImport,
            makes_its_threads: Threads::CreateThread,
            opens_its_files: FileOpen::CreateFileW,
        }
    }

    /// **Yes**: the midstage table, the two seams a stage's numbers go in at, the snapshot a chapter
    /// is — the game's `.data` and the heap orb answers its `GetProcessHeap` with, see
    /// `Memory::ProcessHeapImport` —
    /// and the retry a death is offered are all here, and [`resume`] is the file a run is picked up
    /// from. And the questions those raise are drawn, through a canvas over this game's own swapchain.
    ///
    /// **What a launch still has to say** is what no reading settles: that a restore leaving
    /// [`THE_SOUND_LAYERS_BLOCK`] and the renderer's own alone is one the frame rate survives, and that
    /// the sprite tables a stage loads hold nothing a chapter of it releases — the one range
    /// [`Game::live_handles`] leaves out, on th06c's reasoning for the same engine. `orb-real`'s cases
    /// over this game are where that is asked.
    fn rewinds(&self) -> bool {
        true
    }

    /// **A word's buttons put into the snapshot the game is about to read** — see
    /// [`crate::runtime::keyboard_state`], and th06c's, which this is the same as.
    ///
    /// **Each button at its own virtual key and at the one this game's table holds for it**, which is
    /// what carries the focus: that key goes in at `VK_SHIFT` and the table the game fills its own keys
    /// through — `0x1402f6060`, see [`key::FOCUS_IN_THE_GAMES_TABLE`] — holds `0xa0` and `0xa1` at the
    /// shift scans and no `0x10` at all, so a snapshot holding `VK_SHIFT` alone composes a word with no
    /// focus in it whatever [`Th06nc::presses_into_its_own_keys`] wrote: the compose makes its own read
    /// over those keys every walk of the chain — `th06nc/asm.md`'s reading of `0x140013310` — and that
    /// read is this snapshot. Both keys is what a real `GetKeyboardState` answers, `VK_SHIFT` standing
    /// beside the side that is down, and for the other eight buttons the two are one key and the second
    /// write is the same byte again.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's read was given.
    unsafe fn press_into(&self, state: *mut u8, word: u16) {
        for (_, bit, key) in Self::BUTTONS {
            let held = match word & bit != 0 {
                true => DOWN,
                false => UP,
            };
            for key in [*key, the_key_the_games_table_holds(*key)] {
                if usize::from(key) < KEYS_IN_A_SNAPSHOT {
                    unsafe { state.add(usize::from(key)).write(held) };
                }
            }
        }
    }

    /// **The two times this game keeps put back to nought**, which makes its next read of the keyboard and
    /// of the pad real reads — see [`WHEN_THE_KEYBOARD_WAS_READ`] and [`WHEN_THE_PAD_WAS_READ`], whose own
    /// docs have the instructions that skip a read, and `Game::forgets_when_it_read` for what a playback
    /// wants it for.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the exe's base already told.
    unsafe fn forgets_when_it_read(&self) {
        unsafe { mem::write::<u32>(at(WHEN_THE_KEYBOARD_WAS_READ), NO_READ_YET) };
        unsafe { mem::write::<u32>(at(WHEN_THE_PAD_WAS_READ), NO_READ_YET) };
    }

    /// **The shot's own phase put where the stage's first frame left it** — see [`player::SHOT_PHASE`] for
    /// the pair and [`ONE_UPDATE_INTO_A_CYCLE`] for why the value is not the one the press writes.
    ///
    /// **Only where the pair is idle**, which is what a resumed stage's build leaves: the register writes
    /// the idle pair at `0x140069e46` and the build frame's own update presses nothing, orb having fed it
    /// nought. A pair already in a cycle is one something else started and not this method's to move.
    ///
    /// **And only where the run was firing across the stage's start**, which `word` answers — the first
    /// the playback will feed, the frame's own not being in the file. See this seam's own doc: a run that
    /// held no shot there is one a cycle would be invented for, which is what a laid-out pair measured
    /// when it was started unconditionally — `shot=2,1` against the recording's idle `-1,-999` from frame
    /// 2 to the end of the chapter.
    unsafe fn starts_the_shot_a_stage_began_with(&self, word: u16) {
        if word & button::SHOT == 0 {
            crate::detail!(
                "resume: the run was not firing as the stage began; its shot cycle stays idle"
            );
            return;
        }
        let phase = at(PLAYER + player::SHOT_PHASE);
        let was = unsafe { mem::read::<i32>(phase) };
        if was >= 0 {
            crate::detail!(
                "resume: the stage's shot is already on phase {was}; the cycle stays as it is"
            );
            return;
        }
        unsafe { mem::write::<i32>(phase, ONE_UPDATE_INTO_A_CYCLE.0) };
        unsafe {
            mem::write::<i32>(
                at(PLAYER + player::SHOT_PHASE_BEFORE),
                ONE_UPDATE_INTO_A_CYCLE.1,
            )
        };
        crate::detail!(
            "resume: the stage's shot cycle begins on phase {} where the run's did",
            ONE_UPDATE_INTO_A_CYCLE.0,
        );
    }

    /// **The word this game's own keys in that snapshot say** — see th06c's, which this is the same as, and
    /// `Game::word_in` for what asks. What it buys is both of the things it buys there now that
    /// [`Th06nc::rewinds`] answers true: a launch that plays itself is played by its own word, and a press
    /// held back for a question orb is about to draw is one the game does not act on. The second of those
    /// has not been seen work here — see [`Game::menu_pointed_at`], whose doc has the four launches.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's read was given.
    unsafe fn word_in(&self, state: *const u8) -> Option<u16> {
        let mut word = 0;
        for (_, bit, key) in Self::BUTTONS {
            if usize::from(*key) < KEYS_IN_A_SNAPSHOT
                && unsafe { state.add(usize::from(*key)).read() } & DOWN != 0
            {
                word |= bit;
            }
        }
        Some(word)
    }

    /// **A word's keys written straight into the buffer this game plays a run off**, through the
    /// game's own table — see [`THE_TABLE_OF_KEYS`] — and the keys the word does not hold written up,
    /// for the reason th06c's are.
    ///
    /// **Which is not the whole of handing a word over**: what the run reads is
    /// [`INPUT_THE_RUN_ACTS_ON`], which the game composes out of these keys in the supervisor's own job,
    /// and that job does not run on every walk of the calc chain — see this change's description, whose
    /// `TODO` about a playback that does not reproduce the run it is putting back is the whole of what
    /// that costs.
    unsafe fn presses_into_its_own_keys(&self, word: u16) -> bool {
        let mut pressed = false;
        for (_, bit, key) in Self::BUTTONS {
            let index = unsafe { the_index_of_the_key(the_key_the_games_table_holds(*key)) };
            let Some(index) = index else {
                continue;
            };
            let down = word & bit != 0;
            let held = match down {
                true => DOWN_IN_ITS_OWN_KEYS,
                false => UP_IN_ITS_OWN_KEYS,
            };
            unsafe { mem::write::<u8>(at(THE_KEYS_THE_GAME_READS) + index, held) };
            pressed |= down;
        }
        pressed
    }

    /// **A word's buttons put into the pad state orb answers this game's own read with** — the d-pad
    /// and the stick both, at full deflection, the way th06c's are, and the buttons the game's own
    /// `th06.cfg` binds: the same file as th06c's, kept here at [`supervisor::CONFIG_PAD`] and read by
    /// `0x140012f00` the same way (shot `0x140c6dfc4`, bomb `+0x2`, focus `+0x4`, skip `+0x6`, pause `+0x8`,
    /// the decide's swap `0x140c6dfda`). See `th06c::the_buttons_pressed_through`.
    ///
    /// # Safety
    /// `state` must be the sixteen bytes of an `XINPUT_STATE`.
    unsafe fn press_into_pad(&self, state: *mut u8, word: u16) {
        let buttons = crate::game::th06c::the_buttons_pressed_through(
            at(SUPERVISOR + supervisor::CONFIG_PAD),
            at(SUPERVISOR + supervisor::CONFIG_PAD + supervisor::DECIDE_SWAPPED),
            word,
        );
        let across = match (word & button::RIGHT != 0, word & button::LEFT != 0) {
            (true, false) => i16::MAX,
            (false, true) => i16::MIN,
            _ => 0,
        };
        let down = match (word & button::UP != 0, word & button::DOWN != 0) {
            (true, false) => i16::MAX,
            (false, true) => i16::MIN,
            _ => 0,
        };
        unsafe {
            state
                .cast::<u32>()
                .write(PACKETS.fetch_add(1, Ordering::Relaxed));
            state.add(4).cast::<u16>().write(buttons);
            state.add(6).write(0);
            state.add(7).write(0);
            state.add(8).cast::<i16>().write(across);
            state.add(10).cast::<i16>().write(down);
            state.add(12).cast::<i16>().write(0);
            state.add(14).cast::<i16>().write(0);
        }
    }

    fn button_bit(&self, name: &str) -> Option<u16> {
        Self::BUTTONS
            .iter()
            .find(|(button, _, _)| *button == name)
            .map(|(_, bit, _)| *bit)
    }

    /// **Which item of the front end the cursor is on**, and `None` while a stage runs — see
    /// [`front_end::CURSOR`].
    unsafe fn front_end_cursor(&self) -> Option<i32> {
        if unsafe { in_a_stage() } {
            return None;
        }
        let front_end = unsafe { the_front_end() }?;
        mem::read_committed::<i32>(front_end + front_end::CURSOR)
    }

    /// **And which screen that is**, by the front end's own number for it — see
    /// [`front_end::STATE`] — and nothing while a stage runs, the front end being out of the chain.
    unsafe fn front_end_screen(&self) -> Option<i32> {
        match unsafe { in_a_stage() } {
            true => None,
            false => unsafe { the_scene() },
        }
    }

    /// One frame of a stage as this game holds it — see [`sight`].
    unsafe fn what_a_bot_sees(&self, into: &mut crate::game::Frame) -> bool {
        unsafe { sight::what_a_bot_sees(into) }
    }

    unsafe fn say_what_was_near(&self) {
        unsafe { sight::say_what_was_near() };
    }

    fn named(&self) -> &'static str {
        "th06nc"
    }

    fn menu_decide(&self) -> u16 {
        button::DECIDE | button::SHOT
    }

    fn menu_cancel(&self) -> u16 {
        button::BOMB
    }

    /// The bits a run is written down with, which is every button orb has for this game.
    fn run_input(&self) -> u16 {
        let mut word = 0;
        for (_, bit, _) in Self::BUTTONS {
            word |= bit;
        }
        word
    }

    /// **The run's own numbers where they have been read, and nothing where they have not** — and the
    /// `run:` line said once each time any of them changes, which is how this game's screens were
    /// numbered at all. The scene is the front end's state while it is in the chain and `-1` while a
    /// stage runs, the front end being out of it then.
    unsafe fn read_state(&self) -> State {
        unsafe { say_the_jobs_when_they_change() };
        unsafe { say_the_screen_a_run_ends_at_when_it_changes() };
        unsafe { say_what_a_playback_is_told_by_when_it_changes() };
        unsafe { say_the_words_the_front_end_reads_when_they_change() };
        unsafe { say_the_supervisors_state_when_it_changes() };
        unsafe { say_the_rankings_machine_when_it_changes() };
        unsafe { say_the_line_being_typed_when_it_changes() };
        let scene = unsafe { the_scene() }.unwrap_or(-1);
        // **The run's own numbers read raw**, for the reason
        // [`Th06nc::the_worlds_fingerprint`] gives: each is a static of the exe's own image, committed
        // from the moment the module is mapped, and a committed read of one is a `VirtualQuery` about an
        // address that can never be anywhere else — see
        // `tests::a_state_read_asks_the_memory_map_only_about_what_the_game_allocated`, which is what
        // holds this down.
        let stage = unsafe { mem::read::<i32>(at(STAGE)) };
        let playing = unsafe { in_a_stage() };
        let in_run = unsafe { in_a_run() };
        let boss = unsafe { sight::the_boss_the_fights_own_slot() };
        let difficulty = unsafe { mem::read::<i32>(at(DIFFICULTY)) };
        let lives = unsafe { mem::read::<i8>(at(LIVES)) };
        let bombs = unsafe { mem::read::<i8>(at(BOMBS)) };
        // **Counted on the frame the player's state becomes the hit's and on no other**, a state being
        // held for frames: the frame before is what says this is the edge. Put back to nought where the
        // front end is what is running, so that a launch's second run starts its count where its first
        // did. The same shape th06c's is, and here for the same reason — see [`DEATHS`].
        let player_state = unsafe { mem::read::<u8>(at(PLAYER + player::STATE)) };
        let hit = player_state == player::HIT;
        if playing && hit && !WAS_HIT.swap(hit, Ordering::Relaxed) {
            DEATHS.fetch_add(1, Ordering::Relaxed);
        } else if !playing {
            WAS_HIT.store(false, Ordering::Relaxed);
            if scene == front_end::MENU || scene == front_end::TITLE {
                DEATHS.store(0, Ordering::Relaxed);
            }
        } else {
            WAS_HIT.store(hit, Ordering::Relaxed);
        }
        let playback = unsafe { a_replay_is_playing() };
        if PLAYING_BACK.swap(playback, Ordering::Relaxed) != playback {
            match playback {
                true => crate::log!("run: a replay is playing back"),
                false => crate::log!("run: the playback is over"),
            }
        }
        // **And how far a fight has got, ten times a minute while a boss is up.** The frame lines say a boss
        // is up and nothing of its life, and how far into the Extra's fight a run was lost is the one thing
        // a launch of it is read for — twelve minutes of `boss=true` with three deaths in them say nothing
        // about which attack took them. Every 360 frames, so that a fight is a dozen lines and not a page.
        // Raw, the bodies being slots of a static array of the image — `sight::the_boss_the_fights_own_slot`
        // reads their flags the same way, and it is where the slot this is the life of comes from.
        let boss_life = boss.map(|body| unsafe { mem::read::<i32>(body + enemy::LIFE) });
        match boss_life {
            Some(life) => {
                let frames = FRAMES_ON_A_BOSS.fetch_add(1, Ordering::Relaxed);
                if frames.is_multiple_of(360) {
                    crate::detail!(
                        "run: the boss has {life} life, {lives} live and {bombs} bomb(s) against it"
                    );
                }
                THE_BOSSES_LAST_LIFE.store(life, Ordering::Relaxed);
            }
            // **And how much it had as it went**, which is what tells the Extra's clear from its loss:
            // that stage has no ending, so the one thing that says Flandre was beaten rather than
            // survived is her life when she left — `5` on the launch that cleared, `1000` on the one
            // that died in her survival card. See `orb_real`'s check of the Extra.
            None => {
                if FRAMES_ON_A_BOSS.swap(0, Ordering::Relaxed) > 0 {
                    crate::log!(
                        "run: the boss is gone with {} life left, {lives} live and {bombs} bomb(s)",
                        THE_BOSSES_LAST_LIFE.load(Ordering::Relaxed),
                    );
                }
            }
        }
        // **The ending said once as it arrives**, which is the line a cleared run is read by: nothing else
        // of this game says a run was played out rather than lost — see [`ENDING_UPDATE`].
        let in_ending = unsafe { the_chain_holds(ENDING_UPDATE) };
        if ENDING.swap(in_ending, Ordering::Relaxed) != in_ending && in_ending {
            crate::log!(
                "run: the ending is running, with {lives} live and {bombs} bomb(s) in hand"
            );
        }
        let said = (stage, difficulty, scene, lives, bombs);
        let mut last = SAID
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if last.replace(said) != Some(said) {
            crate::log!(
                "run: stage {stage} on difficulty {difficulty}, scene {scene}, {lives} live and \
                 {bombs} bomb(s)",
            );
        }
        drop(last);
        State {
            scene,
            // **The same number, because this game's front end has no state in flight.** What
            // [`State::wanted`] is for is a game whose screen change takes a frame to arrive — th06 and
            // th06c each keep the state acted on beside the state asked for — and this front end keeps
            // one field, `[front end + 0x168b0]`, which each screen's own case writes outright. There
            // is nothing here to have arrived at later.
            //
            // **And answering `-1` made `runtime`'s own `scene == wanted` never true**, which is the
            // test the frame after a run was given up is found by: the records of a run left that way
            // were committed on no frame at all. The supervisor's pair is not the answer either, its
            // twelve states being another machine's numbers than the thirty-two [`the_scene`] reads —
            // see [`supervisor::STATE`], and `say_the_supervisors_state_when_it_changes`, which is
            // where those are read.
            wanted: scene,
            playing,
            in_run,
            // **The run of somebody's own, which is this game's playback taken out of the reading above**
            // — see [`in_a_run`], where the two are split and why: a stage being played is what a pass
            // steps between the chapters of, and a run of somebody's own is what a chapter is kept and a
            // ranking built out of.
            in_game: in_run && !playback,
            in_ending,
            // **And no script of it read**, which is what th06 1.02h's skip stops at the staff roll by:
            // this game's is unread, so the skip runs the roll out with the rest of it, as th06c's does.
            ending_script: None,
            // **Nothing says a playback is the attract demo here.** Both calls of the replay's own start
            // `0x14004ede0` are the front end's replay list — `0x14004a419` walks the header's stages for
            // the first one recorded and `0x14004a725` takes the one the cursor is on — and no flag beside
            // them has been found that a demo would be told by, so every playback reads as a replay. See
            // `th06nc/asm.md`, which had `0x14053d400` written down as that flag and has it corrected to
            // the pause menu.
            demo: false,
            replay: playback,
            // **A stage practised on its own or a card practised on its own**, off the two bytes the
            // menu's own items write — see [`a_run_is_practice`], [`A_RUN_IS_PRACTICE`] (item 2, with a
            // picture behind it) and [`A_SCENE_IS_BEING_PLAYED`] (item 3, the Spell Practice).
            practice: unsafe { a_run_is_practice() },
            // **Either of the two menus a run stops for**, which is what the frame's own fast-forward
            // refuses to run through — see [`the_run_is_stopped_at_a_menu`].
            paused: unsafe { the_run_is_stopped_at_a_menu() },
            // **A player the hit test cannot reach is a run standing somewhere no chapter is taken**: the
            // hit's own state and the one a stage's register leaves and the death's animation ends in —
            // see [`player::COMING_BACK`], and `asm.md`'s *The frames nothing can reach the player in*.
            // The invulnerability's own state is not among them, that being 240 frames of every stage and
            // a quarter of what a chapter is.
            // **The read above and not a second one of the same byte**, which is what this was: the state
            // the hit is counted off is this same field, and two reads of it a frame can also disagree —
            // nothing suspends the game between them, so a hit landing in the gap made `hit` false and
            // `unsettled` true off one frame's own player.
            unsettled: matches!(player_state, player::HIT | player::COMING_BACK),
            // **A bomb in the air**, which is another frame no chapter begins on — see
            // [`player::BOMB_IN_USE`], the byte the bomb's own start writes and each of the four ends
            // clears.
            bombing: 0 != unsafe { mem::read::<u8>(at(PLAYER + player::BOMB_IN_USE)) },
            // **Whether a line of a conversation is up**, off the line the dialogue object is on — see
            // [`the_line_of_dialogue`], and [`Game::a_line_of_dialogue_is_shown`], which answers the same
            // reading for the frame's own fast-forward.
            in_dialogue: unsafe { the_line_of_dialogue() } >= 0,
            // **The game's own number less one**, this game counting stages from one where `State::stage`
            // is counted from nought the way orb counts them everywhere — see `RunStart::stage`.
            stage: stage - 1,
            difficulty,
            // **The frames the stage has run**, out of the game manager — see
            // [`game_manager::GAME_FRAMES`].
            stage_frames: unsafe { mem::read::<u32>(at(GAME_MANAGER + game_manager::GAME_FRAMES)) },
            // **The clock the stage's own script runs on** — see
            // [`enemy_manager::TIMELINE_TIME`], which is what a midstage boundary would be counted in.
            script_frames: unsafe {
                mem::read::<i32>(at(ENEMY_MANAGER + enemy_manager::TIMELINE_TIME))
            },
            // **The generator itself**, which is the word every draw in the exe reads and writes back —
            // see [`RNG_SEED`].
            random_seed: unsafe { mem::read::<u16>(at(RNG_SEED)) },
            // **orb's own count and not the game's** — see [`DEATHS`], which says what the game's own
            // count at `0x14053cab0` does.
            deaths: DEATHS.load(Ordering::Relaxed),
            lives,
            bombs,
            power: unsafe { mem::read::<u16>(at(POWER)) },
            enemy_count: unsafe { mem::read::<i32>(at(ENEMY_MANAGER + enemy_manager::LIVE)) },
            bullet_count: unsafe { mem::read::<i32>(at(BULLET_MANAGER + bullet_manager::LIVE)) },
            // Raw, for the reason [`Th06nc::the_worlds_fingerprint`] gives: 256 committed reads is 256
            // `VirtualQuery` calls about a static of the exe's own image, on every frame orb reads.
            laser_count: (0..laser::SLOTS)
                .filter(|slot| {
                    0 != unsafe {
                        mem::read::<u8>(at(LASERS) + slot * laser::BYTES + laser::IN_USE)
                    }
                })
                .count() as i32,
            boss_present: boss.is_some(),
            boss_life,
            boss_attack_frames: boss.map(|body| unsafe { mem::read::<i32>(body + enemy::TIMER) }),
            // **Which card the fight is on, where one is up** — see [`enemy_manager::A_CARD_IS_UP`] and
            // [`CURRENT_CARD`], the two the card's own start writes.
            spellcard: (0
                != unsafe { mem::read::<u8>(at(ENEMY_MANAGER + enemy_manager::A_CARD_IS_UP)) })
            .then(|| unsafe { mem::read::<u32>(at(CURRENT_CARD)) }),
        }
    }

    /// **The window the swapchain presents into**, which is this game's own — see
    /// [`crate::dxgi::the_window`], and nought before a swapchain has been taken.
    ///
    /// **Asked of the swapchain and not of the game**, for th06c's reason: this game makes its window
    /// through `CreateWindowExW`, which it looked up, and where it keeps the handle that call answered is
    /// not in `th06nc/asm.md`. The swapchain was built for that window and says so.
    ///
    /// **What it is for is orb's own menus, and answering nought is what kept them unanswerable.**
    /// `runtime::on_update` polls the keyboard against this handle and `input::Keyboard::poll` reads
    /// nothing for a null one — so the mode question this game now raises could be answered by a recipe
    /// and by nobody at a keyboard. The same reading th06c's own doc carries, which this answered the
    /// opposite of.
    unsafe fn window(&self) -> orb_api::Hwnd {
        unsafe { crate::dxgi::the_window() }
    }

    unsafe fn d3d_device(&self) -> orb_api::Device {
        orb_api::Device(0)
    }

    /// **No, and a launch here is a launch with no overlay**: what this game draws through is a D3D11
    /// renderer of its own, the same as th06c.
    fn draws_through_its_d3d_device(&self) -> bool {
        false
    }

    fn music(&self) -> Option<crate::audio::Music> {
        None
    }

    /// **Nothing to wait for**, th06c's reason: the line above answers `None` on every frame.
    fn a_snapshot_waits_for_its_music(&self) -> bool {
        false
    }

    /// **Which track is playing**, out of the static the frame's own catch-up reads — see
    /// [`THE_TRACK_PLAYING`] — and `None` for the `-1` it holds between tracks. What it tells apart is a
    /// stage's own song from its boss's, which is what tells a midboss from the boss.
    fn music_identity(&self) -> Option<u32> {
        mem::read_committed::<i32>(at(THE_TRACK_PLAYING))
            .filter(|track| *track >= 0)
            .map(|track| track as u32)
    }

    fn audio_thread(&self) -> Option<u32> {
        None
    }

    unsafe fn stop_music(&self) {}

    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    /// **The block of statics the music is played out of**, which a restore leaves alone so that the
    /// track goes on where it is — see [`THE_AUDIO_STATICS`].
    fn audio_state(&self) -> Vec<Range<usize>> {
        // One block, collected rather than written as a `vec!` of one range: a `vec![a..b]` reads as a
        // range of numbers to a lint and to a reader alike.
        std::iter::once(at(THE_AUDIO_STATICS.start)..at(THE_AUDIO_STATICS.end)).collect()
    }

    /// **The renderer's COM objects and the pointer to the renderer itself**, which a restore must not
    /// rewind: what they name is D3D11's and DXGI's, made in a heap no snapshot copies, and a pointer put
    /// back names an object released since — see [`THE_RENDERERS_OBJECTS`] and [`THE_RENDERER`]. **And
    /// the sound layer's own block**, whose thirty-two slots each hold a lock — see
    /// [`THE_SOUND_LAYERS_BLOCK`] — **the five queues the frame's effects are flushed through**, see
    /// [`THE_MEDIAS_QUEUES`], **and the five locks that are statics of their own**, see
    /// [`THE_LOCKS_IN_THE_DATA`].
    ///
    /// **And not the textures the game's own sprites hold**, whose slots have not been read: a chapter is
    /// inside one stage and the stage's own sprites are loaded as it begins, so nothing between two
    /// chapters of it releases one. th06c's real cases have that much measured for the same engine; what
    /// says it of this game is a session of `cargo xtask real` over it, which is where a pointer put back
    /// to a texture released since would fault on the next draw.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        let mut live = vec![
            at(THE_RENDERERS_OBJECTS.start)..at(THE_RENDERERS_OBJECTS.end),
            at(THE_RENDERERS_OWN_COUNTS.start)..at(THE_RENDERERS_OWN_COUNTS.end),
            at(THE_RENDERER)..at(THE_RENDERER) + size_of::<usize>(),
            // **And the one object a release names from outside those ranges** — see
            // [`THE_DRAWS_OWN_OBJECT`], which cdb caught the crash of and which is why holding the counts
            // back was not enough.
            at(THE_DRAWS_OWN_OBJECT.start)..at(THE_DRAWS_OWN_OBJECT.end),
        ];
        // And the renderer itself, a block off the heap the snapshot otherwise copies whole. Nothing
        // where the game has not made it yet.
        if let Some(renderer) = mem::read_committed::<usize>(at(THE_RENDERER)).filter(|it| *it != 0)
        {
            live.push(renderer..renderer + THE_RENDERERS_BLOCK);
        }
        live.push(
            at(THE_SPRITE_SHEET_A_DRAW_DIVIDES_BY.start)
                ..at(THE_SPRITE_SHEET_A_DRAW_DIVIDES_BY.end),
        );
        // **And the sheet the base64 above decodes into**, whose span the block's own header says: the
        // counts at its front, summed, are the data after them. Nothing where the decode has not run yet,
        // and nothing for a count that reads as nought — see [`THE_SPRITE_SHEETS_COUNTS`], which is the
        // walk's bound rather than a number the game keeps.
        let sheet = mem::read_committed::<usize>(at(THE_SPRITE_SHEETS_BLOCK));
        if let Some(sheet) = sheet.filter(|it| *it != 0) {
            let after: usize = (0..THE_SPRITE_SHEETS_COUNTS)
                .filter_map(|count| {
                    mem::read_committed::<u16>(sheet + count * size_of::<u16>()).map(usize::from)
                })
                .sum();
            crate::detail!(
                "memory: the sprite sheet is {sheet:#x} and {:#x} bytes",
                THE_SPRITE_SHEETS_HEADER + after,
            );
            live.push(sheet..sheet + THE_SPRITE_SHEETS_HEADER + after);
        } else {
            // **Said rather than passed over**, because a sheet orb cannot find is a range a restore
            // writes back and the log is the only place that would say so.
            crate::detail!(
                "memory: the sprite sheet's pointer reads {sheet:?}; nothing is held back"
            );
        }
        // **And the object a conversation runs in**, whose restore is what stage 2's own chapter could not
        // survive — [`THE_DIALOGUE`] and [`THE_CONVERSATIONS_BYTES`], a block of `0x4228` off the game's
        // own heap.
        //
        // **The whole object and not the fields a bisect narrowed to**, because two of them are pointers
        // to blocks the stage's asset load *frees and refills* — the script's bytes at `+0x38f8` and the
        // buffer at `+0x3b58`, see `asm.md`'s *The conversation* — so a chapter of one stage holds
        // pointers into memory the next stage has handed back, and putting either back is a pointer to
        // something that has moved. Which is the same reason the renderer's block above is held back, and
        // it is why the whole allocation is the unit rather than the span that was measured.
        //
        // **How it was found**: `--hold-back-at=heap1.1:f40+2000` — the object's own first `0x2000` — took
        // the reproducer through stage 2's first chapter under 22 forced restores where every launch
        // without it died there, and `heap1.1` is the heap region this object is in. The description of
        // this change has the eleven statics that name that region and the cuts that named this one.
        let conversation = mem::read_committed::<usize>(at(THE_DIALOGUE));
        if let Some(conversation) = conversation.filter(|it| *it != 0) {
            crate::detail!(
                "memory: the conversation is {conversation:#x} and {THE_CONVERSATIONS_BYTES:#x} bytes"
            );
            live.push(conversation..conversation + THE_CONVERSATIONS_BYTES);
        } else {
            // Said rather than passed over, as the sheet's own miss is: a block orb cannot find is a
            // range a restore writes back, and the log is the only place that would say so.
            crate::detail!(
                "memory: the conversation's pointer reads {conversation:?}; nothing is held back"
            );
        }
        live.push(at(THE_SOUND_LAYERS_BLOCK.start)..at(THE_SOUND_LAYERS_BLOCK.end));
        live.push(at(THE_MEDIAS_QUEUES.start)..at(THE_MEDIAS_QUEUES.end));
        // **And the allocator's own state**, which is the one of these whose reason is the heap and not a
        // handle or a thread: put back, its tags describe an arena the heap has moved on from, and the
        // check on the next allocation ends the process on purpose — see [`THE_ALLOCATORS_OWN_BLOCK`].
        live.push(at(THE_ALLOCATORS_OWN_BLOCK.start)..at(THE_ALLOCATORS_OWN_BLOCK.end));
        for lock in THE_LOCKS_IN_THE_DATA {
            live.push(at(lock)..at(lock) + A_LOCKS_BYTES);
        }
        // And the CRT's own locks, which this build keeps in `.data` — see [`THE_CRTS_LOCKS`].
        live.extend(
            THE_CRTS_LOCKS
                .iter()
                .map(|locks| at(locks.start)..at(locks.end)),
        );
        live
    }

    fn play_area(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        None
    }

    unsafe fn repaint_lives_row(&self) {}

    unsafe fn draws_lives_row(&self) -> bool {
        false
    }

    fn lives_row(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn content_size(&self) -> (u32, u32) {
        (0, 0)
    }

    fn windowed(&self) -> bool {
        false
    }

    unsafe fn force_windowed(&self) {}

    unsafe fn set_play_viewport(&self, _device: orb_api::Device) {}

    fn chain(&self) -> *mut c_void {
        std::ptr::null_mut()
    }

    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    unsafe fn take_sent_keys(&self) -> bool {
        false
    }

    /// **The camera the frame begins with** — see [`MOVES_THE_CAMERA`] — the viewport being the draw's own
    /// business here, see [`walks_the_draw`]; and the count of walks cleared and the frame's own present
    /// told it is beginning. And the window sized to the settings on the first of them — see
    /// [`size_the_window_to_the_settings`].
    unsafe fn prepare_frame(&self, _device: orb_api::Device) {
        WALKS_THIS_FRAME.store(0, Ordering::Relaxed);
        unsafe { size_the_window_to_the_settings() };
        unsafe { crate::dxgi::starts_a_frame() };
        unsafe { moves_the_camera_as_the_frame_does() };
    }

    unsafe fn begin_drawing(&self, _device: orb_api::Device) {}

    /// **What the loop did after the draw frame**, called whole with its limiter got past — see
    /// [`ENDS_THE_FRAME`].
    ///
    /// **Got past by starting it over each frame rather than by cutting it out**: the bars, the picture's
    /// finish and the wait are one function with nothing between them to land in, so the wait is left
    /// in and given nothing to wait for. `0x14003dccd` starts the limiter over where the clock is more
    /// than a period past `base + period × (count + 1)`, and a base of nought puts every clock there —
    /// so the function measures nothing and waits for nothing, and the frame's timing is
    /// `frame::Pacing`'s alone, which is what a second pacing on top of it would spoil. The fps line it
    /// writes in the corner is what this costs.
    unsafe fn end_drawing(&self, _device: orb_api::Device) {
        unsafe {
            mem::write::<i64>(at(THE_LIMITERS_BASE), 0);
            mem::write::<i64>(at(THE_LIMITERS_COUNT), 0);
        }
        let ends: unsafe extern "win64" fn() = unsafe { std::mem::transmute(ends_the_frame()) };
        unsafe { ends() };
    }

    /// **Both of them orb's own**: the sound's catch-up is inline in the loop and the present is inside
    /// the frame's own end — see [`tops_up_the_sound`] and [`presents`].
    fn frame_calls(&self) -> FrameCalls {
        FrameCalls {
            play_sounds: Call {
                function: tops_up_the_sound as *const () as usize,
                this: 0,
            },
            present: Call {
                function: presents as *const () as usize,
                this: 0,
            },
            orbs_own: true,
        }
    }

    /// **This game counts the hit**, the way th06c does — orb's count moves as the state becomes
    /// [`player::HIT`], see [`DEATHS`] — so what became of it is read off the two fields the game decides it
    /// with: the state, and the frames a bomb is still taken in at [`player::A_BOMB_STILL_TAKES_IT_BACK`].
    /// A state no longer the hit's with the window still open is a death a bomb took back.
    ///
    /// **Not the default's `Taken`**, which is th07's answer for a game that counts a death only once the
    /// window has closed: read here, every deathbomb of a pointdevice run was a death, the retry menu going
    /// up on the frame of the hit and the bomb pressed inside the window never reaching the game.
    unsafe fn where_a_counted_death_stands(&self) -> CountedDeath {
        let state = mem::read_committed::<u8>(at(PLAYER + player::STATE)).unwrap_or(player::NORMAL);
        let window = mem::read_committed::<i32>(at(PLAYER + player::A_BOMB_STILL_TAKES_IT_BACK))
            .unwrap_or(0);
        match state == player::HIT {
            true if window > 0 => CountedDeath::Open,
            true => CountedDeath::Taken,
            false => CountedDeath::Bombed,
        }
    }

    /// **Whether a line of a conversation is up**, off the line the dialogue object is on — see
    /// [`the_line_of_dialogue`].
    ///
    /// **The same reading `State::in_dialogue` is**, as th06c's two are: the narrower answer this method
    /// is for elsewhere is the line no words are drawn for, and this build has no field telling that
    /// frame from the rest of a line — the index is up from the frame the conversation starts.
    ///
    /// **Not th06c's own handle on it**, which is a panel field its frame's fast-forward gates on: this
    /// build's fast-forward reads four things and none of them is one — `0x14003d910` onwards asks
    /// `[0x14053d3dc]` set, `[0x14053d400]` clear, `[0x140c6dfbc]` exactly 2 and `0x8000` of
    /// `[0x140abae80]`. What is read here instead is what the *game* asks when it has to leave a line
    /// alone: the bomb press is refused at `0x14006a0d6` while this is nought or more.
    unsafe fn a_line_of_dialogue_is_shown(&self) -> bool {
        (unsafe { the_line_of_dialogue() }) >= 0
    }

    /// **Declined, and what declines it is a reading and not a gap.** The call a jump would be is read:
    /// `0x14004ede0(front end, stage)`, the replay's own start, whose second argument is the stage counted
    /// from nought — `0x14004a400` walks the header's seven pointers at `+0x40` and hands over the first
    /// that is not null, and `0x14004a725` hands over `[front end + 0x28]`, the list's own cursor. So the
    /// pair is the replay's object and an index into those seven.
    ///
    /// **What cannot be had is either of them while a playback is running.** The replay a playback is of
    /// is freed as that playback starts: `0x14004a422` reads `[front end + 0x168b8]`, frees it through
    /// `0x1402c0378` and writes nought over the pointer, so the header the jump would index is gone by
    /// the frame the stage begins — and the front end's own teardown at `0x140057ad5` frees that same
    /// field again, which is what makes the null deliberate. And the front end leaves the chain from a
    /// run's start to the title, so `what_a_job_runs_on(FRONT_END_UPDATE)` answers nothing through the
    /// whole of one either.
    ///
    /// **So a jump over this game is not the call th06 and th07 make**: it would have to load the replay
    /// again the way the list's own decide does — format `replay/th6_%.2d.rpy`, read the file, and start
    /// it at the stage wanted — which is orb driving the front end's own machinery rather than one call.
    /// **And it is no gap against th06c, which declines the same method**:
    /// `moving_between_a_replays_stages` is a module th06 and th07 have and neither 紅魔郷 build does.
    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    /// **The state a respawn's own invulnerability is, written for one update** — see
    /// [`player::INVULNERABLE`], whose countdown the player's update takes a frame off and whose nought
    /// puts the state back itself.
    ///
    /// **Only out of the two states it is safe to leave**, which is th06c's own gate: written over the
    /// hit's state it would take a death back that the game has already begun, and written over a bomb's
    /// it would end the bomb.
    unsafe fn make_invulnerable(&self) {
        let state = at(PLAYER + player::STATE);
        let current = mem::read_committed::<u8>(state).unwrap_or(player::NORMAL);
        if current == player::NORMAL || current == player::INVULNERABLE {
            unsafe {
                mem::write::<u8>(state, player::INVULNERABLE);
                mem::write::<i32>(
                    at(PLAYER + player::UNTOUCHABLE),
                    player::INVULNERABLE_FRAMES,
                );
            }
        }
    }

    unsafe fn replaying(&self) -> bool {
        unsafe { a_replay_is_playing() }
    }

    /// **What the entry on the screen a run ends at holds** — see [`result_screen::NAME`], and only while
    /// that entry is the state the screen is in, so that the ranking the *Score* item opens answers nothing.
    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        let screen = unsafe { the_entry_a_name_is_typed_into() }?;
        crate::game::the_name_held_at(screen + result_screen::NAME, result_screen::NAME_BYTES)
    }

    /// Whether that entry is up at all — see [`the_entry_a_name_is_typed_into`].
    unsafe fn typing_a_name(&self) -> bool {
        unsafe { the_entry_a_name_is_typed_into() }.is_some()
    }

    /// **Into the field that entry's own keys write**, with its count of characters typed at the full eight
    /// and its cursor left on the item that ends it — the state the game itself is in once somebody has
    /// typed eight, so the press after it leaves the screen the way theirs would.
    ///
    /// **Worth having here for the same reason th06c has it**: the entry comes up while the rules are
    /// still being asked for a word, and the word they hold is the shot, which writes the character the
    /// cursor starts on eight times over — `AAAAAAAA`, on every clear a recipe registered before this.
    unsafe fn types_its_name(&self, name: &str) -> bool {
        let Some(screen) = (unsafe { the_entry_a_name_is_typed_into() }) else {
            return false;
        };
        if !crate::game::the_name_typed_into(
            screen + result_screen::NAME,
            result_screen::NAME_BYTES,
            name,
        ) {
            return false;
        }
        unsafe {
            mem::write::<i32>(
                screen + result_screen::TYPED,
                result_screen::NAME_BYTES as i32,
            );
            mem::write::<i32>(screen + result_screen::CURSOR, result_screen::THE_END_ITEM);
        }
        true
    }

    /// **Nothing, and the five methods below answer the emptiest true answer with it.** This game's
    /// ranking is the main menu's item 5, 夢の記録: its front end state 12 asks the supervisor for **6**
    /// at `0x14004d92c`, and the supervisor's case `0x14007b966` builds the screen a run ends at, which
    /// comes up in state 0 with [`result_screen::A_RUNS_OWN`] clear. See `th06nc/asm.md`'s *The main
    /// menu's ten items*.
    ///
    /// **And `0x14000c960` is not a ranking**: it is the menu a scene's end puts up, the machine the chain
    /// job at priority 2 runs behind `[0x14053d406]`, whose state 2 builds that screen with twelve — its
    /// list of replay slots — in its [`result_screen::STATE`] for item 1's replay save; the two writers of
    /// that byte are a function of the job's own and **the stage's build**, both behind the scene's byte,
    /// and no item of the main menu's.
    ///
    /// **And a ranking built from item 5 writes nothing**: `score.dat` is written as the screen a run
    /// ends at goes down, by its taken-out callback `0x140075b40`, only where its `+0x9e99` says it is a
    /// run's own, which the ranking's build leaves clear — the file's other write being `0x140076370`'s,
    /// as a run is left for the front end or started again. See `th06nc/asm.md`'s *The score file's two
    /// writes*. So a ranking orb built and took down would spend the whole of `commit_records`'s
    /// allowance and leave the file as it was.
    unsafe fn show_ranking(&self) {}

    unsafe fn showing_ranking(&self) -> bool {
        false
    }

    unsafe fn ranking_scene(&self) -> bool {
        false
    }

    unsafe fn restore_menu_cursor(&self) {}

    unsafe fn ranking_state(&self) -> String {
        String::new()
    }

    unsafe fn leave_ranking(&self) {}

    /// **One more attempt at the card the fight is on**, counted where the game cannot: the game counts
    /// one where a card *starts*, and a chapter beginning inside one never starts it.
    ///
    /// **Only where a card is up and the record is one the game has named** — see
    /// [`enemy_manager::A_CARD_IS_UP`], [`CURRENT_CARD`] and [`named`]: a record the layout has left is
    /// `0xc0` words of the generator's, so a slot whose name and sum do not agree is one no card has
    /// been declared into and a count written there would be a count of noise.
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        if mem::read_committed::<u8>(at(ENEMY_MANAGER + enemy_manager::A_CARD_IS_UP)).unwrap_or(0)
            == 0
        {
            crate::log!("score: no spell card is up; no attempt counted");
            return None;
        }
        let card = mem::read_committed::<u32>(at(CURRENT_CARD)).unwrap_or(u32::MAX);
        let record = unsafe { card_record(card) }?;
        if !unsafe { named(record) } {
            crate::log!("score: card {card} is not one the game has named; no attempt counted");
            return None;
        }
        let attempts = mem::read_committed::<u16>(record + CATK_ATTEMPTS)
            .unwrap_or(0)
            .saturating_add(1);
        unsafe { mem::write::<u16>(record + CATK_ATTEMPTS, attempts) };
        Some(attempts)
    }

    unsafe fn redraw_card_name(&self) {}

    /// **The 134 card records whole** — see [`CARD_HISTORY`].
    unsafe fn captures(&self) -> Vec<u8> {
        mem::read_committed_bytes(at(CARD_HISTORY), CARD_HISTORY_BYTES).unwrap_or_default()
    }

    unsafe fn set_captures(&self, saved: &[u8]) {
        if saved.len() != CARD_HISTORY_BYTES {
            return crate::log!(
                "score: {} bytes of captures is not this build's {CARD_HISTORY_BYTES}; left alone",
                saved.len()
            );
        }
        unsafe { mem::write_bytes(at(CARD_HISTORY), saved) };
    }

    /// **The name and the sum beside it together, never one without the other** — th06c's own answer, and
    /// th06's before it, whose records these are the shape of: a restore has to leave the name the game
    /// has written since, and a sum put back beside a name that was not is a record the game reads as
    /// another card's and clears the count of.
    ///
    /// Where the two sit is read now — [`CATK_NAME`] and [`CATK_NAME_SUM`], off the copy the card's own
    /// start makes at `0x140027950` and the walk `0x1400279a0` compares — so this keeps them where it
    /// once put the whole record back for want of the offset.
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]) {
        let mut going_back = saved.to_vec();
        for (card, record) in going_back
            .as_chunks_mut::<CARD_BYTES>()
            .0
            .iter_mut()
            .enumerate()
        {
            let standing = at(CARD_HISTORY) + card * CARD_BYTES;
            let name = mem::read_committed_bytes(standing + CATK_NAME, CATK_NAME_BYTES)
                .unwrap_or_default();
            if name.len() == CATK_NAME_BYTES {
                record[CATK_NAME..CATK_NAME + CATK_NAME_BYTES].copy_from_slice(&name);
            }
            if let Some(sum) = mem::read_committed::<u8>(standing + CATK_NAME_SUM) {
                record[CATK_NAME_SUM] = sum;
            }
        }
        unsafe { self.set_captures(&going_back) };
    }

    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// **True, and the read it means is `0x14003c5e7`**: the stage build's run-start branch calls
    /// `0x14006e310(the parsed file, manager + 0x968)` — [`CARD_HISTORY`], the array
    /// [`Game::count_card_attempt`] steps a count in — after the loop at `0x14003c390` has laid all
    /// [`CARDS`] records out. A stage a run walks into takes the other branch and reads nothing, so this
    /// is a once-a-run read and the loss is a run's counting going as the next run starts.
    fn its_run_build_lays_its_file_over_the_cards(&self) -> bool {
        true
    }

    /// **A count in any record's attempts**, the word at [`CATK_ATTEMPTS`] — the one count a record of
    /// this build carries that a run moves, `0x140027a3a` writing it a step up as a card starts.
    ///
    /// **Asked of the game rather than of the bytes**, for th07's reason: an emptied record is not a
    /// zeroed one. `0x14003c390`'s loop writes `CATK` and the card's number into all 134, so a scan for
    /// zero bytes would answer that an emptied array is full.
    fn these_records_hold_a_count(&self, records: &[u8]) -> bool {
        records.as_chunks::<CARD_BYTES>().0.iter().any(|record| {
            u16::from_le_bytes([record[CATK_ATTEMPTS], record[CATK_ATTEMPTS + 1]]) != 0
        })
    }

    /// **The counts back over what the build's own read laid down, with the names left as they are** —
    /// [`Th06nc::set_captures_keeping_names`] doing both, and its doc carrying why a name and the sum
    /// beside it are never written apart.
    ///
    /// **Back rather than added**, which is th07's reading and holds here for the same reason: the build
    /// lays the file's records over the array wholesale, so what a run counted is the file's own *with*
    /// that run's on top. Added to the file's again, a card attempted once would read as two.
    unsafe fn put_back_the_cards_a_run_left(&self, left: &[u8]) {
        unsafe { self.set_captures_keeping_names(left) };
        crate::log!(
            "score: what the run before counted put back over the {CARDS} this build read in"
        );
    }

    /// **True, and what it means here is that no ranking orb builds would write anything**, the game's
    /// own writes standing at every end of a run, [`Game::leave_run`]'s included.
    ///
    /// **This game writes its score file in two places, each serialising it whole from memory**:
    /// `0x140075b40`, the taken-out callback of the screen a run ends at, as that screen goes down and
    /// where its `+0x9e99` says it is a run's own; and `0x140076370`, called at `0x14007b782` as a run is
    /// left for the front end and at `0x14007bc2f` by the supervisor's state 12 as one is started again,
    /// both unless a replay is playing. A ranking put up from the main menu's item 5 leaves `+0x9e99`
    /// clear, so building one and taking it down, which is what this answering false asks
    /// `commit_records` to do, would spend that function's whole frame allowance on a screen and leave
    /// the file exactly as it was. **And a replay given up through the supervisor's 8 writes nothing**:
    /// `0x14007bb13`'s one call, `0x14006db10` at `0x14007bb42`, passes a null path, and `0x14006db4f`
    /// sends that to `0x14006e058`, which only drops the replay being recorded.
    ///
    /// **And the card counts reach the file at each of those writes**: both lay down the 134 `CATK`s out
    /// of `0x14053d418`, the very array a run counts its attempts in (`0x140075e7e`, `0x140076940`) — see
    /// `th06nc/asm.md`'s *The score file's two writes*. What [`Th06nc::put_back_the_cards_a_run_left`] is
    /// for is keeping them across the run build that would otherwise lay the file back over them, so that
    /// the next write carries them.
    fn writes_its_own_records(&self) -> bool {
        true
    }

    /// **The copy every parse works off, put together again out of both files**: the records of runs out
    /// of the mode's own, the `CLRD` records the front end lights its Extra from out of the game's —
    /// [`score_file`] has why it is a copy rather than a read. Installed the way the game's own read
    /// installs it: a block off the game's `malloc`, the block there before freed through its `free`, and
    /// the pointer and the length written where `0x14006e140` reads them.
    ///
    /// Both files are asked for by their own names, relative, so each resolves where the game's open
    /// would.
    ///
    /// A `malloc` that answers nought leaves the copy as it was, said in the log: a parse of what was
    /// there is a screen showing the other mode's records, where a parse of nothing would be a first
    /// launch.
    unsafe fn read_what_the_modes_file_records(&self) {
        let own = orb_api::fs::read(Path::new(crate::score::THEIRS_NAME)).ok();
        let modes_name = crate::score::the_file_now_opened();
        let modes = orb_api::fs::read(Path::new(modes_name)).ok();
        let copy = score_file::the_copy_the_mode_reads(own.as_deref(), modes.as_deref());
        let malloc: unsafe extern "win64" fn(usize) -> usize =
            unsafe { std::mem::transmute(mallocs()) };
        let free: unsafe extern "win64" fn(usize) = unsafe { std::mem::transmute(frees()) };
        let block = unsafe { malloc(copy.len()) };
        if block == 0 {
            crate::log!(
                "score: the game's malloc answered nothing for {} bytes; its copy of the file stands",
                copy.len()
            );
            return;
        }
        unsafe { mem::write_bytes(block, &copy) };
        let before = unsafe { mem::read::<usize>(at(SCORE_FILE_COPY)) };
        if before != 0 {
            unsafe { free(before) };
        }
        unsafe { mem::write::<usize>(at(SCORE_FILE_COPY), block) };
        unsafe {
            mem::write::<i32>(
                at(SCORE_FILE_COPY_LENGTH),
                i32::try_from(copy.len()).expect("a score file of a few kilobytes"),
            )
        };
        crate::log!(
            "score: the copy the game parses is {modes_name}'s runs under {}'s unlocks, {} bytes",
            crate::score::THEIRS_NAME,
            copy.len()
        );
    }

    /// **Yes**: the bytes at `0x140c6e368` are read once and every later parse deciphers a copy of them —
    /// see [`score_file`], and `Hooks::unlocks_read`, which is `None` here because there is no callback
    /// to write a jump over.
    fn reads_its_file_once_at_startup(&self) -> bool {
        true
    }

    /// **The item under the main menu's cursor, where that item is one of orb's two questions' own** —
    /// see [`front_end::MENU`]: its three items that start a run, and [`front_end::ITEM_REPLAYS`], the
    /// one the replays are behind. th06c asks over its *title*; this game's title is a state of its own
    /// that a decide opens the menu from, so the question goes over the menu.
    ///
    /// **And `None` for the other six items**: each of them opens a screen that is neither a run nor a
    /// list of replays, and those are the two things the mode is a question about — item 5 the ranking,
    /// 夢の記録 (see [`Game::show_ranking`]), item 6 the music room, item 7 the credits, item 8 the
    /// options and item 9 the game quitting. What each opens is in `th06nc/asm.md`.
    ///
    /// **And the question it is for has never gone up on the real game**, which two instrumented
    /// launches narrowed to one thing missing — see [`Game::menu_takes_a_press`], which is where it is
    /// written out. What those launches said, and what rules the other answers out: this method answered
    /// `Some(Menu::Run)` the frame the menu came up, `runtime`'s own line read
    /// `menu: something to ask true, the screen takes a press true, so the decide is held true` on the
    /// same frame, and the question still never stood — so neither this nor the hold is the fault.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        let front_end = unsafe { the_front_end() };
        let state = front_end.and_then(|at| mem::read_committed::<i32>(at + front_end::STATE));
        let cursor = front_end.and_then(|at| mem::read_committed::<i32>(at + front_end::CURSOR));
        let pointed = match (state, cursor) {
            (
                Some(front_end::MENU),
                Some(front_end::ITEM_RUN | front_end::ITEM_EXTRA | front_end::ITEM_PRACTICE),
            ) => Some(Menu::Run),
            (Some(front_end::MENU), Some(front_end::ITEM_REPLAYS)) => Some(Menu::Replays),
            _ => None,
        };
        // **What this saw, said once each time it changes** — the instrument the fault above needs: a
        // launch whose menu stands on item 0 and whose question never goes up is one where either the
        // front end is not found from here or its two fields read as something else, and nothing else
        // in the log tells those apart.
        let seen = (front_end.is_some(), state, cursor);
        if MENU_LAST_SEEN.lock().unwrap().replace(seen) != Some(seen) {
            crate::detail!(
                "menu: the front end is {}, its state {state:?} and its cursor {cursor:?}",
                if front_end.is_some() {
                    "in the chain"
                } else {
                    "not in the chain"
                },
            );
        }
        pointed
    }

    /// **Whether the menu would act on a decide on the read after this one**: the screen is the menu and
    /// it has been up for its grace — see [`front_end::MENU_GRACE_FRAMES`], the twenty frames
    /// `0x140049a8d` leaves its own case for.
    ///
    /// **A count and not just the screen, which is what five launches cost.** The hold is stored after
    /// the game's update — see `runtime::on_update` — so the frame orb first sees a screen is a frame the
    /// game has already read its input on. Answered true there, the hold arrives one frame late: the
    /// decide that opened the menu is still down and already spent, `held_back`'s `DECIDE_WAS_DOWN` is
    /// true so no press is recorded, and the screen acts on it. Those launches read
    /// `menu: something to ask true, the screen takes a press true, so the decide is held true` on the
    /// frame the menu came up, stood no question at all, and were on the next screen after. Held back
    /// over the grace and handed over afterwards, the press is one the screen had not read yet — which is
    /// th06c's answer for its own two screens.
    ///
    /// **One short of the grace**, for th06c's reason: the read after this one is a frame on, so a count
    /// of nineteen is a press the twentieth frame takes.
    unsafe fn menu_takes_a_press(&self) -> bool {
        let found = unsafe { the_front_end() };
        let Some(front_end) = found else {
            return false;
        };
        // **Both screens orb has a question over, each with its own grace** — the main menu, where the
        // mode is asked, and the last screen before a stage that takes a press, where picking a run up
        // is. th06c's answers for its two the same way and for the same reason: a screen orb holds a
        // press out of has to be one whose own case would act on it, and each of these ignores its
        // decide for its own count of frames.
        //
        // **Without the second of them a run cannot be picked up at all**, whatever
        // `Game::run_pointed_at` answers: `runtime`'s hold is
        // `something to ask && asks over the game && this`, so a screen this refuses is a screen whose
        // decide reaches the game — and for that screen the decide starts a run.
        let grace = match mem::read_committed::<i32>(front_end + front_end::STATE) {
            Some(front_end::MENU) => front_end::MENU_GRACE_FRAMES,
            Some(front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS) => {
                front_end::THE_LAST_SCREENS_GRACE_FRAMES
            }
            _ => return false,
        };
        let frames = mem::read_committed::<i32>(front_end + front_end::SCREEN_FRAMES).unwrap_or(0);
        // **The screen and its count, said as either turns over** — the instrument [`MENU_LAST_SEEN`]
        // is for `menu_pointed_at`, over the other half of the hold: a launch whose question never
        // stood had this answering false for 3,620 frames of a screen the other half said was the
        // right one, and no line said which of the two numbers it was refusing on.
        let seen = Some((frames, grace));
        let said = {
            let mut last = TAKES_A_PRESS_LAST_SEEN
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (*last != seen).then(|| *last = seen).is_some()
        };
        if said {
            crate::detail!(
                "menu: the screen has been up {frames} frame(s) and ignores a press for {grace}"
            );
        }
        // The read after this one is a frame on, so a count one short of the grace is a press the next
        // read takes.
        frames >= grace - 1
    }

    /// **th06c's, this game's word and pad being that one's bit for bit** — see `Th06c::pad`, `Th06c::pad_word`
    /// and `Th06c::pushed_for`.
    unsafe fn pad(&self, pad: Option<Reading>) -> Pad {
        super::th06c::the_pad_of(unsafe { self.pad_word(pad, true) })
    }

    unsafe fn pad_word(&self, pad: Option<Reading>, _hats: bool) -> u16 {
        pad.map_or(0, super::th06c::the_word_of_a_sampled_pad)
    }

    unsafe fn pushed_for(&self, word: u16) -> Option<crate::joystick::Pushed> {
        Some(super::th06c::pushed_as(word))
    }

    unsafe fn add_pad_buttons(&self, _into: *mut c_void, _pad: Option<Reading>) {}

    /// **The state the pause menu's own give-up asks of the supervisor**: [`STATE_FRONT_END`] for a run,
    /// which writes `score.dat` on its way to the title, and [`STATE_RUN_OVER`] for a replay, which writes
    /// nothing — see that constant. The two menus a run stops for are cleared first, so that the state the
    /// transition runs in is a run nothing is paused in.
    ///
    /// Nothing where a run is not what the supervisor is running.
    unsafe fn leave_run(&self) -> bool {
        if mem::read_committed::<i32>(at(SUPERVISOR + supervisor::STATE)) != Some(STATE_RUN) {
            return false;
        }
        let replay =
            mem::read_committed::<u8>(at(A_REPLAY_IS_PLAYING)).is_some_and(|byte| byte != 0);
        let wanted = match replay {
            true => STATE_RUN_OVER,
            false => STATE_FRONT_END,
        };
        unsafe {
            mem::write::<u8>(at(GAME_MANAGER + game_manager::IS_IN_GAME_MENU), 0);
            mem::write::<u8>(at(GAME_MANAGER + game_manager::IS_IN_RETRY_MENU), 0);
            mem::write::<i32>(at(SUPERVISOR + supervisor::WANTED), wanted);
        }
        true
    }

    /// **Every button into the word the run acts on, so that the edge every screen reads — this word
    /// against the frame before's — finds nothing on the frame the game carries on into.**
    ///
    /// **What makes it work is `0x14007b8ac`**: the supervisor's own job copies this word into the frame
    /// before's once a walk of the calc chain, after the music's fade and before the state it acts on,
    /// `mov eax,[0x140abae80]; mov [0x140abae84],eax`, and then reads the
    /// keyboard over it at `0x14007b8b7`. So a word of every bit left here is the frame before's next
    /// frame, and an edge — a bit set in this word and not in that one — is a bit that cannot be set in
    /// both: what is genuinely held still reads as held, and only the edge is gone.
    /// **Every bit but the fast-forward's**, which is the one bit of this word the game reads as a level
    /// and not as an edge: `0x14003d92b` tests `0x8000` of it and walks the calc chain up to seven more
    /// times while a replay is playing, the pause is clear and the state is a run's. So a word of every
    /// bit left here would be a frame of that run played eight times over — which is not what swallowing
    /// a press is for, and is the one thing this word carries that filling it cannot leave alone.
    ///
    /// **And that read is in the frame orb's own stands in for**, so under an installed patch it is code
    /// nothing executes: [`THE_FRAME`] is the function the fast-forward is inside. What multiplies a
    /// frame's updates in an orb launch is orb's own repeats instead. The bit stays out of this write all
    /// the same: it costs a swallowed press nothing, and a launch whose patch did not install is one whose
    /// frame is the game's own.
    unsafe fn swallow_input(&self) {
        unsafe { mem::write::<u32>(at(INPUT_THE_RUN_ACTS_ON), !THE_FAST_FORWARD) };
    }

    /// **[`result_screen::EXIT`] written over [`result_screen::SAVE_REPLAY_QUESTION`]**, which is
    /// th06c's own answer unchanged: one state is the whole of it, that screen's way out being a state
    /// and not a press, and nothing at all on a frame the screen is anywhere else.
    ///
    /// **Which state each of those is, and the picture that says so**, is written beside the two
    /// constants. What made the reading possible is an Extra run stood still until it was out of lives —
    /// the one run this game asks no continue about — and
    /// [`say_the_screen_a_run_ends_at_when_it_changes`]'s line beside a picture taken from outside, orb
    /// having no back buffer of this game to draw one from.
    ///
    /// **And the write is driven on the real game now, by a run played to its clear.** `runtime` asks this
    /// in Pointdevice mode alone, and a pointdevice run reaches that screen by a clear only: every death
    /// of one is answered by the retry, and a run given up at the retry walks the front end's own way back
    /// instead — a launch read `front end: screen 16, 17, 18` with the screen never built. What reaches it
    /// is `orb_real`'s `a_run_played_to_its_clear`, **`ok over 106264 frames`** over six stages and
    /// twenty-four retries, whose own check reads the name typed into that screen.
    ///
    /// **`--clear` was not what got there, and the reason it could not is withdrawn.** It was written up
    /// as standing still in stage 1's boss fight; two launches say the flag takes a run through that card
    /// and into stage 2, and that the standing still was a recipe's own `wait_until` step, which presses
    /// nothing. See `th06nc/asm.md`'s *The player*, whose paragraph carries both.
    ///
    /// **What is read of the rest of that screen**: the eighteen states of the table at `0x140073ef4` are
    /// eight handlers, five of them sharing `0x140072b05`, and that splits into ten at `0x1400739e8`,
    /// eleven at `0x140073842`, twelve the list of replay slots at `0x14007324c`, thirteen the entry of
    /// the name the replay is saved under at `0x140072d6f` and fourteen falling through at
    /// `0x140072b34`. Fourteen pulses two colour words by `[+0x9e7c]`, moves that cursor on `0x200` and
    /// `0x400` of `0x140abae80` against `0x140abae84`, and on `0x100` — the decide — takes the name
    /// buffer's address at `0x140072d3a` and writes **thirteen** into `[+0x9e94]` where the cursor is
    /// nought, or calls `0x14006e9e0` and stays where it is not. **And thirteen writes that one name into
    /// the replay and into `th06.cfg`**, `0x140072fed` taking `replay/th6_%.2d.rpy`, `0x14007301f` being
    /// the replay's write and `0x140073032` the copy of the name `th06.cfg` keeps.
    ///
    /// So that question is in front of the entry's own writing of the files and not behind it.
    ///
    /// **And what fourteen stands over is a slot that already holds a replay**: `0x1400734f3` indexes
    /// the screen's own slots at `+0x9eb0`, `0x78` apart, by that same cursor, and goes to fourteen
    /// where the slot's first dword is `T6RP` and its `+0x9eb4` is `0x10f` — a slot holding neither
    /// goes to `0x140072c0f`, the way into thirteen, without asking.
    ///
    /// **A clear walks the same screen**: the launch of `a_run_played_to_its_clear_without_a_bomb` that
    /// passed went `ending skipped` straight into the entry [`Game::typing_a_name`] reads, which is the
    /// state in front of the offer.
    unsafe fn skip_replay_prompt(&self) -> bool {
        let Some(screen) = (unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }) else {
            return false;
        };
        let state = screen + result_screen::STATE;
        if mem::read_committed::<i32>(state) != Some(result_screen::SAVE_REPLAY_QUESTION) {
            return false;
        }
        unsafe { mem::write::<i32>(state, result_screen::EXIT) };
        true
    }

    /// **[`result_screen::EXIT`] written over whatever state the screen is in**, that being the game's
    /// own way out of it — the name entry included, which is the state a played run's screen is built in.
    /// Nothing where the screen is already leaving.
    unsafe fn leave_the_screen_a_run_ends_at(&self) -> bool {
        let Some(screen) = (unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }) else {
            return false;
        };
        let state = screen + result_screen::STATE;
        if mem::read_committed::<i32>(state) == Some(result_screen::EXIT) {
            return false;
        }
        unsafe { mem::write::<i32>(state, result_screen::EXIT) };
        true
    }

    /// **The job being in the chain, which is the whole of that screen being up here**: it is built on
    /// the heap and nothing holds a pointer to it, so the chain holding [`RESULT_SCREEN_UPDATE`] is what
    /// says it exists — and what takes it down is the screen's own leaving, which takes the element out.
    ///
    /// **A run's own or the ranking somebody asked for, either way**, which is what this is asked for:
    /// [`Game::run_finished`] is the one that reads `+0x9e99` to tell the two apart.
    unsafe fn the_screen_a_run_ends_at_is_up(&self) -> bool {
        unsafe { the_chain_holds(RESULT_SCREEN_UPDATE) }
    }

    /// **The frames the stage has run**, and `None` where no stage is running — see
    /// [`game_manager::GAME_FRAMES`], which the stage's own job steps up once a frame. The game manager
    /// is a static that stays where it is, so the clock reads whatever it last held between stages,
    /// which is what the question of whether a stage is running is in front of it for.
    unsafe fn stage_frame(&self) -> Option<u32> {
        unsafe { in_a_stage() }
            .then(|| mem::read_committed::<u32>(at(GAME_MANAGER + game_manager::GAME_FRAMES)))
            .flatten()
    }

    /// The generator itself — see [`RNG_SEED`] — which is the same word [`State::random_seed`] reads.
    unsafe fn generator(&self) -> u16 {
        mem::read_committed::<u16>(at(RNG_SEED)).unwrap_or(0)
    }

    /// **Out of the game manager**, the static a run is made of — the difficulty, the character and the
    /// shot the front end's three screens wrote, and the stage counted from nought where the game counts
    /// from one.
    ///
    /// **And `practice` off both of the bytes the main menu's items write** — see [`a_run_is_practice`]:
    /// item 2's stage practice at [`A_RUN_IS_PRACTICE`], which a picture of the screen after that item
    /// settled (`PRACTICE` across the difficulty select), and item 3's **Spell Practice** at
    /// [`A_SCENE_IS_BEING_PLAYED`], which a picture of screen 28 settled. Neither is a run whose chapters
    /// are anybody's to put back, so [`Game::run_slot`] names neither.
    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: mem::read_committed::<i32>(at(DIFFICULTY)).unwrap_or(-1),
            character: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::CHARACTER))
                .map_or(-1, i32::from),
            shot_type: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::SHOT))
                .map_or(-1, i32::from),
            practice: unsafe { a_run_is_practice() },
            stage: mem::read_committed::<i32>(at(STAGE)).unwrap_or(0) - STAGE_COUNTS_FROM,
            // **The run's own byte and not the configuration's**, which is the difference between this
            // and [`Game::run_pointed_at`]: by the time a run is on, screen 25 has turned
            // `[0x140c6dfdb]` into [`A_RUN_IS_IN_CHALLENGE_MODE`] — see that constant, and
            // [`Game::run_slot`], which is what the number is for.
            mode: unsafe { the_mode_a_run_is_in() },
        }
    }

    /// **The run the front end is about to start, read on the last screen before the stage** — which is
    /// where the question about picking a run up stands, and where the three numbers a run is are all
    /// decided: the difficulty was written by screen 9, the character by 11 and the shot by 13, each into
    /// the game manager, and screen 25's own decide is what calls [`RUN_START`].
    ///
    /// **And not the shot select, which is where th06c reads it.** That screen presses itself through on
    /// its own fifteenth frame while the front end is skipping — `0x1400491af`, the mask `0x6a00`,
    /// reached only where `[front end + 0x168d0]` is set — so a question raised there could be gone
    /// before it was answered. Screen 25 presses itself through for nobody: a launch that pressed nothing
    /// at 24 sat there for 1,800 frames.
    ///
    /// **Read out of the manager and not out of the cursor**, the shot select's decide having already
    /// written the shot at `0x14004d21d`: a cursor read here would be screen 25's own and not the shot's.
    ///
    /// **And `practice` off the front end's own byte** — [`front_end::THE_RUN_IS_PRACTICE`] — rather
    /// than off [`A_RUN_IS_PRACTICE`], which is the static [`RUN_START`] copies it into a screen later:
    /// a run is pointed at before it is started, so the item's answer is only in the front end yet.
    /// That is what makes [`Game::run_slot`] decline here, and a practice run is therefore neither
    /// offered a chapter nor able to take the full run's.
    ///
    /// Nothing while the front end is anywhere else, and nothing for a run with no slot — see
    /// [`Game::run_slot`], which is what names the file.
    ///
    /// **A file left in `pointdevice_resume` costs a launch its walk**, which is why the launches driven
    /// of this game carry `--no-resume`: one whose directory held `normal-reimu-a` said
    /// `resume: 1 run(s) left unfinished: normal-reimu-a` and then walked past its title without ever
    /// asking about it, where the launch before it — the same recipe with nothing to pick up — walked
    /// into a run.
    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        let found = unsafe { the_front_end() };
        let front_end = found?;
        if mem::read_committed::<i32>(front_end + front_end::STATE)
            != Some(front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS)
        {
            return None;
        }
        Some(RunStart {
            difficulty: mem::read_committed::<i32>(at(DIFFICULTY))?,
            character: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::CHARACTER))?
                .into(),
            shot_type: mem::read_committed::<u8>(at(GAME_MANAGER + game_manager::SHOT))?.into(),
            practice: mem::read_committed::<u8>(front_end + front_end::THE_RUN_IS_PRACTICE)
                .unwrap_or(0)
                != 0,
            // A full run's chapter can be in any stage, and which one the mark says rather than the
            // run — the same shape th06c's has.
            stage: 0,
            // **The configuration's byte and not the run's**, screen 25 not having copied it yet — see
            // [`THE_MODE_LAST_CHOSEN`], which screen 14 wrote on the way here.
            mode: unsafe { the_mode_the_next_run_is_in() },
        })
    }

    /// **The difficulty and the shot, which is what a run of this game is one of** — the same four shots
    /// and five difficulties 東方紅魔郷 has always had, and the same names th06 and th06c write their
    /// slots as, so that a directory of runs reads the same whichever build played them.
    ///
    /// **And nothing at all for a stage practised on its own**, the same answer th06c gives: a practice
    /// run is one stage with nothing carried in and nothing after it, and the three numbers here do not
    /// say which stage it is — so a name built from them is the *full* run's name, and keeping a
    /// practice run under it would offer a whole run's chapter back as that stage. See
    /// `crate::resume`'s own paragraph on this, and [`front_end::THE_RUN_IS_PRACTICE`] for the byte
    /// [`Game::run_pointed_at`] answers it by.
    fn run_slot(&self, run: &RunStart) -> Option<String> {
        if run.practice {
            return None;
        }
        const DIFFICULTIES: [&str; 5] = ["easy", "normal", "hard", "lunatic", "extra"];
        const SHOTS: [&str; 4] = ["reimu-a", "reimu-b", "marisa-a", "marisa-b"];
        let difficulty = usize::try_from(run.difficulty)
            .ok()
            .and_then(|at| DIFFICULTIES.get(at));
        let shot = usize::try_from(run.character * 2 + run.shot_type)
            .ok()
            .and_then(|at| SHOTS.get(at));
        // **And nothing of the game's own mode in it yet, which is a slot this game owes.**
        // `0x14053d414` is **which of Standard and Challenge the run is in** — settled by two pictures:
        // the screen the shot select hands on to is 「開始するモードを選択してください」 over 標準モード
        // and チャレンジモード, and a launch that decided the second started a run whose HUD reads
        // `Challenge Mode` — and it changes what a run starts with, `0x14004f373` writing two lives where
        // the byte is set. So a slot that ignores it hands one kind of run the other's chapter.
        //
        // **And the mode is in the name, because it changes what a run starts with**: `0x14004f373`
        // writes two lives where the byte is set, so a チャレンジモード run and a 標準モード run of the
        // same difficulty and shot begin differently and are two runs. A slot that ignored it would
        // hand one kind of run the other's chapter — and offer it a replay of the other's, `replay::find`
        // matching by this name.
        //
        // **Standard keeps the bare name and Challenge is the one that carries a word**, rather than a
        // suffix on both: every chapter file written for this game so far is a Standard run's, and a
        // name of `normal-reimu-a` for those is a name none of them moves away from.
        let name = match (difficulty, shot) {
            (Some(difficulty), Some(shot)) => format!("{difficulty}-{shot}"),
            _ => format!(
                "difficulty{}-character{}-shot{}",
                run.difficulty, run.character, run.shot_type
            ),
        };
        Some(match run.mode {
            0 => name,
            _ => format!("{CHALLENGE}-{name}"),
        })
    }

    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8> {
        resume::encode(saved)
    }

    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept> {
        resume::decode(bytes)
    }

    unsafe fn run_state(&self) -> Vec<u8> {
        resume::encode_state(&unsafe { self.numbers() })
    }

    unsafe fn set_run_state(&self, state: &[u8]) {
        let Some(state) = resume::decode_state(state) else {
            return crate::log!("resume: the numbers put back are not a run of this game's");
        };
        unsafe { self.put_the_numbers_back(&state) };
    }

    unsafe fn seed_the_stage(&self, state: &[u8]) {
        let Some(state) = resume::decode_state(state) else {
            return crate::log!("resume: the seed put back is not a run of this game's");
        };
        unsafe { self.seed(state.seed) };
        // **And the instrument for the one defect a launch of `orb-real` found here**, which is why a
        // line of numbers sits in the middle of a seed: a run picked up in its second stage came up in
        // its third, `resume: not put back — stage 3 came up where stage 2 was asked for`. What decides
        // that is the build's own branch at `0x14003c27c` — `cmp [0x140c6dfbc],3`, the supervisor's state
        // *wanted* — whose taken side counts the stage on at `0x14003c7c0` and whose other side leaves it
        // as found. This seam is inside the build and past both, so what it says is which path ran and
        // what the stage field holds now, and a launch of that case answers it in one line. **Verbose,
        // and once a stage**: it is beside the seed because the seed is the one thing orb writes in here.
        crate::detail!(
            "resume: the stage was built with the supervisor on {:?} wanting {:?}, and the stage now \
             reads {:?}",
            mem::read_committed::<i32>(at(SUPERVISOR + supervisor::STATE)),
            mem::read_committed::<i32>(at(SUPERVISOR + supervisor::WANTED)),
            mem::read_committed::<i32>(at(STAGE)),
        );
    }

    /// Every field of them, out of the struct, with the seed said again in front as the `0x1a2b` every
    /// other line writes it as.
    fn say_run_state(&self, state: &[u8]) -> String {
        match resume::decode_state(state) {
            Some(state) => format!("seed={:#06x} {state:?}", state.seed),
            None => "numbers of another game's run".to_owned(),
        }
    }

    /// **A run asked of the supervisor and not yet acted on**, which is the one frame the game has
    /// settled what the run is and has built none of it: the run's own start asks for [`STATE_RUN`] at
    /// `0x14004f38b` and the supervisor's job acts on the ask on the walk *after* it — `0x14007b909`
    /// leaves for the exit while the two agree — so a frame whose wanted is the run's and whose acted-on
    /// is not is that frame and no other.
    ///
    /// **And the clock is what this read before, which the moment it is asked at does not survive.** The
    /// reading was `game_manager::GAME_FRAMES` at nought with a stage in the chain: the build writes the
    /// nought at `0x14003c0f7` and the stage's own job steps it up at the *end* of its first run,
    /// `0x14003c04d`. Both sites that ask sit **past** the chain walk, and that one walk holds the build
    /// and the step together, so nought was never once read there — a run picked up out of a file started
    /// at the stage the front end chose with none of its numbers back, the question answered `from where
    /// it stopped` and `resume: stage 1 begins with` saying a fresh set two lines later. The pair of
    /// states is read before the walk of the frame that acts on it and after the walk that asked, which
    /// is what makes it answerable from either side. `the_numbers_a_run_of_th06nc_was_made_with` is what
    /// measured it and is what holds it.
    ///
    /// A replay is refused, asking for a run the same way.
    unsafe fn run_chosen(&self) -> bool {
        let wanted = mem::read_committed::<i32>(at(SUPERVISOR + supervisor::WANTED));
        let acted_on = mem::read_committed::<i32>(at(SUPERVISOR + supervisor::STATE));
        let playing_back = unsafe { a_replay_is_playing() };
        wanted == Some(STATE_RUN) && acted_on != Some(STATE_RUN) && !playing_back
    }

    /// **The stage orb counts it as, which is from nought — because the build counts it on** — see
    /// [`STAGE`], the manager's own `+0x24`: `0x14003c7bc` reads that field and `0x14003c7c0` writes it
    /// back one higher, and **both of the build's paths reach those two**, `0x14003c78b` jumping the
    /// run-start path into the same `0x14003c7a2` the stage-following one falls to. So the number a stage
    /// comes up as is the number this wrote plus one, which is also why a run's own first stage comes up
    /// as one out of the nought the front end leaves in there.
    ///
    /// **Two launches of the real game measured it, and the first was measuring another bug.** Written
    /// `stage + STAGE_COUNTS_FROM`, a run picked up in its second stage came up in its third —
    /// `resume: not put back — stage 3 came up where stage 2 was asked for`, with
    /// `Th06nc::seed_the_stage` saying `the supervisor on Some(1) wanting Some(2)` from inside that same
    /// build, which is what ruled the branch at `0x14003c27c` out as the thing that raised it. The
    /// launch before that read `stage 1 came up where stage 2 was asked for` off a 0-based write, and
    /// that one was [`Game::run_chosen`] never being true: the resume pointed no build at all and the
    /// front end started a run of its own at stage one.
    ///
    /// Checked against the seven the game has, the number coming out of a file beside the game.
    unsafe fn start_stage(&self, stage: i32) -> bool {
        if !(0..STAGES).contains(&stage) {
            crate::log!("resume: there is no stage {}", stage + 1);
            return false;
        }
        unsafe { mem::write::<i32>(at(STAGE), stage) };
        true
    }

    /// **A run of these numbers written into being from the front end**, the way th06c's is: the
    /// difficulty, the character, the shot and the stage written where the screens that choose them write
    /// them, and then the game's own [`RUN_START`] called on the front end.
    ///
    /// **On the front end and not on nothing**, this game's start taking it in `rcx` — `0x14004dc06` is
    /// `mov rcx,rsi; call`, `rsi` being the object whose `+0x168b0` that screen had just written. So a
    /// start asked for while the front end is not in the chain is one there is nothing to call it on, and
    /// that is refused rather than called with nought.
    ///
    /// **The playback flag is not cleared here**: the start clears `0x14053d3dc` itself at `0x14004f395`,
    /// and so does the byte beside it.
    ///
    /// False where the game will not take it, which is a run of numbers no game of this one's has —
    /// nothing is written then, a half-written run being worse than none.
    unsafe fn start_a_run(&self, run: &RunStart) -> bool {
        if !(0..STAGES).contains(&run.stage) {
            crate::log!("replay: there is no stage {}", run.stage + 1);
            return false;
        }
        if !(0..=DIFFICULTY_EXTRA).contains(&run.difficulty) {
            crate::log!("replay: there is no difficulty {}", run.difficulty);
            return false;
        }
        if !(0..SHOT_COMBINATIONS).contains(&(run.character * 2 + run.shot_type)) {
            crate::log!(
                "replay: there is no character {} shot {}",
                run.character,
                run.shot_type
            );
            return false;
        }
        let found = unsafe { the_front_end() };
        let Some(front_end) = found else {
            crate::log!("replay: the front end is not up, so there is nothing to start a run on");
            return false;
        };
        unsafe {
            mem::write::<i32>(at(DIFFICULTY), run.difficulty);
            mem::write::<u8>(
                at(GAME_MANAGER + game_manager::CHARACTER),
                run.character as u8,
            );
            mem::write::<u8>(at(GAME_MANAGER + game_manager::SHOT), run.shot_type as u8);
            mem::write::<i32>(at(STAGE), run.stage);
        }
        let start: unsafe extern "win64" fn(usize) = unsafe { std::mem::transmute(run_start()) };
        unsafe { start(front_end) };
        true
    }

    /// **The run asked for through the last screen of the walk that takes a press**, which is
    /// [`front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS`]: the three numbers a run is written here, that
    /// screen put up past its own grace, and the caller's press then decided at it.
    ///
    /// **What that press then does is the whole of the walk from 24 on**, which is what going this way
    /// round buys over [`Game::start_a_run`]: `0x14004db6c` edge-checks the decide, `0x140078290` and
    /// `0x1400781d0` fade the track out over `0x1e` frames, `0x1400777f0` stops it, `0x140c6de2c` is
    /// cleared, and `0x14004dbd4` writes screen 25 — whose own case waits for `[0x140c6e378]`'s `+0x34`
    /// to read 2 (`0x14004dbf9`) and only then calls the run's own start `0x14004f320` with the front end
    /// in `rcx`. So the loader is waited out and the music is put down the way a person's own run has
    /// them, where a run written into being has the track still playing and the load not waited for.
    ///
    /// **The stage is not among what this writes**, there being no screen of the walk that asks for one:
    /// a run started this way is a run from its first stage, and a pick-up further in is
    /// [`Game::start_a_run`]'s with [`Game::jump_to_stage`] after it.
    unsafe fn front_end_starts_a_run(&self, run: &RunStart) -> bool {
        if !(0..=DIFFICULTY_EXTRA).contains(&run.difficulty) {
            crate::log!("replay: there is no difficulty {}", run.difficulty);
            return false;
        }
        if !(0..SHOT_COMBINATIONS).contains(&(run.character * 2 + run.shot_type)) {
            crate::log!(
                "replay: there is no character {} shot {}",
                run.character,
                run.shot_type
            );
            return false;
        }
        let found = unsafe { the_front_end() };
        let Some(front_end) = found else {
            crate::log!("replay: the front end is not up, so no run is asked of it");
            return false;
        };
        unsafe {
            mem::write::<i32>(at(DIFFICULTY), run.difficulty);
            mem::write::<u8>(
                at(GAME_MANAGER + game_manager::CHARACTER),
                run.character as u8,
            );
            mem::write::<u8>(at(GAME_MANAGER + game_manager::SHOT), run.shot_type as u8);
            mem::write::<i32>(
                front_end + front_end::STATE,
                front_end::THE_LAST_SCREEN_THAT_TAKES_A_PRESS,
            );
            mem::write::<i32>(
                front_end + front_end::SCREEN_FRAMES,
                front_end::THE_LAST_SCREENS_GRACE_FRAMES,
            );
        }
        true
    }

    /// **The screen a run ends at, and a run's own rather than a ranking somebody asked for**: the chain
    /// holds [`RESULT_SCREEN_UPDATE`] and the screen it runs on has [`result_screen::A_RUNS_OWN`] set,
    /// which `0x14007bdc3` writes as the job a stage ends into builds it and the *Score* item's build at
    /// `0x14000cd97` does not.
    unsafe fn run_finished(&self) -> bool {
        unsafe { what_a_job_runs_on(RESULT_SCREEN_UPDATE) }.is_some_and(|screen| {
            mem::read_committed::<u8>(screen + result_screen::A_RUNS_OWN)
                .is_some_and(|its_own| its_own != 0)
        })
    }

    /// **One frame of a run as the numbers a restore has to reproduce**, which is what `--self-check`
    /// holds two passes of the same chapter against: the word the update acted on, the player and the box
    /// they are held in, the generator and how far it has been drawn, the items in the air, the score as
    /// shown and the lives the score has paid for.
    ///
    /// **Which is th06c's field set less two**, each for a reading this game has not got: there is no
    /// replay manager's frame here — where this game keeps that count is `asm.md`'s *not established* —
    /// and no rank, the byte the manager keeps it in being unread. What a parting says is therefore which
    /// of eight numbers parted and not which of ten.
    /// **The bullet manager's own array, folded three ways** — which slots are in use and in which
    /// state, where each of them is, and what each moves by. th06c's own three, over this game's array:
    /// `0x280` slots of `0x620` bytes at `bullet_manager::BULLETS`.
    ///
    /// **Three folds and not one**, for th06c's reason: two passes whose slots agree and whose positions
    /// do not have moved the same bullets differently, and two whose slots disagree have fired or dropped
    /// a different bullet — and one number over all of it says only that something did.
    ///
    /// **The state of every slot, in use or not**, because a slot going out is a thing two passes can
    /// differ about on its own: the walk's own gate reads nought as a slot holding nothing.
    /// **And the enemies' array beside it, and the player's shots** — `0x100` slots of `0x10b0` at
    /// [`enemy_manager::ENEMIES`] and the eighty of [`player::SHOTS`], each folded the same way. Both are
    /// here because a pair of real launches parted in them and in nothing else the line carried: see
    /// [`Reproduction::enemies`] and [`Reproduction::shot_phase`].
    /// **Read raw rather than through `mem::read_committed`, which is what this walk cost.** A committed
    /// read is a `VirtualQuery` apiece — measured at about eleven microseconds on this machine — and this
    /// walks 640 slots with up to four more reads each, on every frame of every run a verbose launch
    /// plays. That was 18ms a frame of the 21 `profile::Phase::Update` reported, and half the frame rate
    /// of a 完全無欠モード run. The array is a static of the exe's own image, committed from the moment
    /// the module is mapped, and the game's own bullet update walks all 640 slots unguarded — so there is
    /// nothing for a query to find out. `Th06c`'s fingerprint reads it raw for the same reason.
    unsafe fn the_worlds_fingerprint(&self) -> Option<String> {
        // The fold every other one in this tree is: multiply by a prime and add, which two arrays
        // differing anywhere differ under.
        let eat = |fold: &mut u32, bits: u32| *fold = fold.wrapping_mul(31).wrapping_add(bits);
        let (mut states, mut places, mut steps) = (0u32, 0u32, 0u32);
        let mut live = 0usize;
        for slot in 0..bullet_manager::SLOTS {
            let bullet =
                at(BULLET_MANAGER) + bullet_manager::BULLETS + slot * bullet_manager::BYTES;
            let state = unsafe { mem::read::<u16>(bullet + bullet_manager::STATE) };
            eat(&mut states, u32::from(state));
            if state == 0 {
                continue;
            }
            live += 1;
            for (fold, of) in [
                (&mut places, bullet_manager::POSITION),
                (&mut steps, bullet_manager::STEP),
            ] {
                for axis in 0..2 {
                    let word = unsafe { mem::read::<f32>(bullet + of + axis * size_of::<f32>()) };
                    eat(fold, word.to_bits());
                }
            }
        }
        // **And the enemies, folded the same way**, which is what a playback that parts in a spawn's draws
        // wants: the count alone says there are as many and not that they are the same ones. Their flags,
        // their places and their life, the three a slot is told apart by — see [`enemy::FLAGS`],
        // [`enemy::POSITION`] and [`enemy::LIFE`]. This game's array is `0x100` slots of `0x10b0` bytes
        // where the bullets' is `0x200` of `0x160`, so it is a quarter of the walk above.
        let (mut kinds, mut where_they_are, mut lives) = (0u32, 0u32, 0u32);
        let mut enemies = 0usize;
        for slot in 0..enemy_manager::SLOTS {
            let enemy = at(ENEMY_MANAGER) + enemy_manager::ENEMIES + slot * enemy_manager::BYTES;
            let flags = unsafe { mem::read::<u8>(enemy + enemy::FLAGS) };
            eat(&mut kinds, u32::from(flags));
            if flags & enemy::IN_USE == 0 {
                continue;
            }
            enemies += 1;
            for axis in 0..2 {
                let word =
                    unsafe { mem::read::<f32>(enemy + enemy::POSITION + axis * size_of::<f32>()) };
                eat(&mut where_they_are, word.to_bits());
            }
            eat(&mut lives, unsafe { mem::read::<u32>(enemy + enemy::LIFE) });
        }
        // **And the player's own shots, which are in neither array above** — see [`player::SHOTS`]. Two
        // folds rather than three: a slot holds no step of its own that a place does not already say, the
        // hit test reading the pair at `+0x13c` and nothing else of where it has got to. Eighty slots, so
        // this is an eighth of the enemies' walk.
        let (mut fired, mut shot_at) = (0u32, 0u32);
        let mut shots = 0usize;
        for slot in 0..player::SHOT_SLOTS {
            let shot = at(PLAYER) + player::SHOTS + slot * player::SHOT_BYTES;
            let in_use = unsafe { mem::read::<u16>(shot + player::shot::IN_USE) };
            eat(&mut fired, u32::from(in_use));
            if in_use == 0 {
                continue;
            }
            shots += 1;
            for axis in 0..2 {
                let word = unsafe {
                    mem::read::<f32>(shot + player::shot::POSITION + axis * size_of::<f32>())
                };
                eat(&mut shot_at, word.to_bits());
            }
        }
        Some(format!(
            "bullets={live} states={states:08x} places={places:08x} steps={steps:08x} \
             alive={enemies} kinds={kinds:08x} at={where_they_are:08x} lives={lives:08x} \
             shots={shots} fired={fired:08x} shot_at={shot_at:08x}"
        ))
    }

    unsafe fn reproduction(&self) -> String {
        unsafe { Th06nc::reproduced() }.to_string()
    }

    /// **The supervisor asked for a run, or for the stage after one, and no stage running yet** — which
    /// is what a frame inside the stage's own build looks like from outside: `Game::stage_frame` answers
    /// nothing on it, the player is not registered and the play field reads as zeroes.
    ///
    /// **Not th06c's reading**, which is its replay manager's clock reading below nought: where this
    /// game keeps that count is one of the things `asm.md` still lists as not established. What is read
    /// here is the pair the supervisor moves between — `+0x42c` the state wanted — against
    /// [`in_a_stage`], whose own answer is the player's job being in the chain.
    unsafe fn mid_build(&self) -> bool {
        let wanted = mem::read_committed::<i32>(at(SUPERVISOR + supervisor::WANTED));
        let building = matches!(wanted, Some(STATE_RUN) | Some(STATE_NEXT_STAGE));
        building && !unsafe { in_a_stage() }
    }

    /// **Seven rows with nothing in them** — see [`chapters`], which has why they are empty and why
    /// another build's rows are not what goes in them.
    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &chapters::MIDSTAGE
    }

    fn stages_past_the_numbered(&self) -> &'static [&'static str] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        THE_FRAME_PROLOGUE, Th06nc, bullet_manager, button, key, the_key_the_games_table_holds,
    };
    use crate::game::th06nc::image::Image;
    use crate::game::{Game, Read};

    /// **What a state read may ask the memory map about: what the game allocated, and nothing it holds as
    /// a static of its own.** Every ask is a `VirtualQuery` in a launch — a syscall — so a walk of one of
    /// this game's arrays through `mem::read_committed` is one syscall per slot, on every frame of every
    /// run.
    ///
    /// **Which is the test th07 has and this game did not, and the defect it did not catch.**
    /// `Th06nc::the_worlds_fingerprint` walked all 640 bullet slots that way, `Th06nc::reproduced` all 512
    /// item slots and this read all 256 laser slots — 1,408 asks a frame about addresses that are in the
    /// exe's own image and can never be anywhere else. Read raw, the laser walk alone took a millisecond a
    /// frame off `profile::Phase::State`.
    #[test]
    fn a_state_read_asks_the_memory_map_only_about_what_the_game_allocated() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.registers_the_supervisor();
        image.registers_the_gameplay_scene();
        image.is_playing(0, 1);
        image.has_lives_and_bombs(2, 3);
        image.a_boss_is_fighting(1500, 0);
        image.bullets_alive(LIVE_BULLETS);
        for slot in 0..LIVE_BULLETS as usize {
            image.a_bullet_of_size_is_at(slot, 100.0, 100.0, (1.0, 1.0), (4.0, 4.0));
        }

        let before = image.space().asked_about_an_address();
        let read = unsafe { Th06nc.read_state() };
        let asked = image.space().asked_about_an_address() - before;
        assert_eq!(read.lives, 2, "the read found the run's own numbers");
        assert!(
            asked <= ASKS_A_STATE_READ_MAKES,
            "a state read asked the memory map {asked} time(s), which is more than the \
             {ASKS_A_STATE_READ_MAKES} the game's own allocations are behind",
        );
    }

    /// Enough bullets that a walk of the array would show up against the bound and no slot's worth of
    /// arithmetic is what the count is about.
    const LIVE_BULLETS: i32 = 64;

    /// **And the same for the line every frame of a run writes about how it reproduces**, which is the
    /// other half of what walked the arrays: `Game::reproduction` counted the 512 item slots through the
    /// map and `Game::the_worlds_fingerprint` read the state of all 640 bullet slots and four words of
    /// every slot in use. Together they were 1,152 asks a frame against this bound.
    ///
    /// **Its own test because `read_state` does not call either**, so the bound above says nothing about
    /// them — and they are asked on every frame of every verbose run, which is what a session that leaves
    /// a log is.
    #[test]
    fn the_line_a_frame_writes_about_its_reproduction_asks_the_memory_map_about_nothing() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.registers_the_supervisor();
        image.registers_the_gameplay_scene();
        image.is_playing(0, 1);
        image.a_boss_is_fighting(1500, 0);
        image.bullets_alive(LIVE_BULLETS);
        for slot in 0..LIVE_BULLETS as usize {
            image.a_bullet_of_size_is_at(slot, 100.0, 100.0, (1.0, 1.0), (4.0, 4.0));
        }

        let before = image.space().asked_about_an_address();
        let reproduction = unsafe { Th06nc.reproduction() };
        let fingerprint = unsafe { Th06nc.the_worlds_fingerprint() };
        let asked = image.space().asked_about_an_address() - before;
        assert!(
            !reproduction.is_empty(),
            "the reproduction the line is written from is empty",
        );
        // The count and not the three folds: what this test is about is how the walk reads, and a fold is
        // the arithmetic of whatever was laid out — `the_worlds_fingerprint`'s own e2e is where the folds
        // are held to two passes agreeing.
        let fingerprint = fingerprint.expect("this game answers a fingerprint");
        assert!(
            fingerprint.starts_with(&format!("bullets={LIVE_BULLETS} ")),
            "the fingerprint did not see the bullets that were laid out: {fingerprint}",
        );
        // `assert_eq!` and not the `<=` its neighbour holds its own bound with: nought is the least a
        // count can be, so the two read the same and only this one says which claim is being made.
        assert_eq!(
            asked, ASKS_THE_REPRODUCTION_LINE_MAKES,
            "the reproduction and the fingerprint asked the memory map {asked} time(s), where every \
             address either of them reads is a static of the exe's own image",
        );
    }

    /// **Nought, measured, against the 1,152 the two made between them before the arrays were read raw
    /// and the 9 they made while `Th06nc::reproduced`'s own fields still went through the map.** Every
    /// address either of them reads is a static of the exe's own image — the input the run acts on, the
    /// player, the play field, the generator's pair, the score, the extra lives, and the item slots the
    /// count walks — so the line a frame of a run writes about itself asks the memory map about nothing,
    /// which is what this test's name says.
    ///
    /// **So this bound is nought and not a small number**, and what it catches is any address either of
    /// them gains that is *not* a static: a field behind a pointer the game handed over belongs in the
    /// reproduction only with a reason beside it, and this is where that decision is noticed.
    const ASKS_THE_REPRODUCTION_LINE_MAKES: u64 = 0;

    /// **What that bound is: what this read asks now, measured** — 39, from 74 and then 69.
    ///
    /// **And what the 39 are is the calc chain**, which is the part that is not a static: `read_state`
    /// walks it five times a frame — `say_the_jobs_when_they_change` over every link, and
    /// `what_a_job_runs_on`, `the_front_end`, `the_scene` and `the_chain_holds` each until they find
    /// theirs — and every link is a block the game allocated and free to be halfway rebuilt, which is
    /// exactly what the map is asked about. The head of the chain is a static and read raw; two asks a
    /// link after it are not.
    ///
    /// **It came down from 74 to 69 when the run's own numbers were made raw** — `STAGE`, `DIFFICULTY`,
    /// `LIVES`, `BOMBS` and the player's state, five statics, five asks — **and from 69 to 39 when the
    /// instruments' were**: the front end's two words, the playback's five bytes, the supervisor's pair,
    /// the dialogue's pointer, the three chain heads, the two menus, the practice pair, the replay's byte,
    /// the hold, the stage, and the ten statics `State` is built out of. The player's state is read once
    /// now rather than twice, `unsettled` having read the same byte over again.
    ///
    /// **What is left to take off it is a walk and not a read**: the five walks are four more than a
    /// frame needs, the chain being the same list each time. That is a cache with a correctness question
    /// on it — the chain is what the game rebuilds as screens come and go — so it is not done here, and
    /// the number says what it would be worth.
    ///
    /// **And what it catches at this size is the thing that got in**: a walk of one of this game's arrays,
    /// the smallest of which is the 256 laser slots and the largest the 640 bullet ones.
    const ASKS_A_STATE_READ_MAKES: u64 = 39;

    /// **What orb stands in front of here is a name and not a place**, the same as th06c: the slot the
    /// game keeps its answer in is the game's own to write.
    #[test]
    fn the_keyboard_read_is_stood_in_front_of_by_the_name_the_game_asks_for() {
        let Some(Read::LookedUp(lookup)) = Th06nc.hooks().input else {
            panic!("th06nc's input is not a name the game asks for");
        };
        assert_eq!(lookup.name, "GetKeyboardState");
    }

    /// **And no dialog is answered**, this exe putting none up: a click posted at a control of a dialog
    /// that never comes would be a click at nothing.
    #[test]
    fn no_startup_dialog_is_answered() {
        assert!(Th06nc.hooks().startup_dialog.is_none());
    }

    /// **The frame is a function the loop calls, stood in whole**: its prologue's fifteen bytes are at
    /// least the fourteen a 64-bit jump takes.
    #[test]
    fn the_frame_is_stood_in_over_its_own_prologue() {
        let hooks = Th06nc.hooks();
        let frame = hooks.render.expect("th06nc's frame is a function");
        assert_eq!(frame.prologue, THE_FRAME_PROLOGUE);
        assert!(frame.prologue.len() >= 14);
        assert!(hooks.walks.is_some(), "the two walks are handed over");
    }

    /// **What each of the five bullet states multiplies its own step by** — th06c's five.
    #[test]
    fn each_bullet_state_scales_its_own_step_by_what_the_game_divides_it_by() {
        for (state, scale) in [(1, 1.0), (2, 0.5), (3, 0.4), (4, 1.0 / 3.0), (5, 0.5)] {
            assert_eq!(bullet_manager::a_steps_scale(state), scale);
        }
    }

    /// **The focus is looked up in the game's table as the left shift and every other button as itself.**
    #[test]
    fn the_focus_is_the_one_button_the_games_table_holds_under_another_key() {
        assert_eq!(
            the_key_the_games_table_holds(key::FOCUS),
            key::FOCUS_IN_THE_GAMES_TABLE,
        );
        for (name, _, pressed) in Th06nc::BUTTONS {
            if *pressed == key::FOCUS {
                continue;
            }
            assert_eq!(
                the_key_the_games_table_holds(*pressed),
                *pressed,
                "{name} is looked up under a key of its own",
            );
        }
    }

    /// **Every button orb's word has is a key it presses.**
    #[test]
    fn every_button_a_recipe_can_name_is_a_key_this_game_is_played_with() {
        for (name, bit, pressed) in Th06nc::BUTTONS {
            assert_eq!(
                Th06nc.button_bit(name),
                Some(*bit),
                "{name} is not a button a recipe can name",
            );
            assert_ne!(*pressed, 0, "{name} is pressed with no key at all");
        }
        assert_eq!(Th06nc.button_bit("nothing of the kind"), None);
    }

    /// **And a word's keys are the ones it names and no others.**
    #[test]
    fn a_word_presses_the_keys_its_own_buttons_are() {
        let mut state = [0u8; 256];
        unsafe { Th06nc.press_into(state.as_mut_ptr(), button::DECIDE | button::UP) };
        for (at, byte) in state.iter().enumerate() {
            let expected = u8::from(at == usize::from(key::DECIDE) || at == usize::from(key::UP));
            assert_eq!(
                u8::from(*byte != 0),
                expected,
                "the key at {at:#04x} reads {byte:#04x} where a decide and an up were pressed",
            );
        }
    }
}
