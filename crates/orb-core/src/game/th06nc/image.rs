//! A th06nc process image laid out in an address space, so that the real [`Th06nc`](super::Th06nc) has
//! something to read in a test with no game running.
//!
//! **Everything a run of this game is made of is a static**, the same shape th06c's image has and for the
//! same reason: the game manager, the supervisor, the player, the enemies, the bullets, the items, the
//! generator and both chains are all inside the exe's own `.data`, and that one range is what a chapter is
//! a copy of. What the game keeps on its heap is what orb reaches through a pointer or a chain element —
//! the front end, the screen a run ends at, the elements the chain is made of, the renderer's block — and
//! each of those is a block of this file's own choosing, outside [`DATA`], so that a snapshot of a
//! laid-out game has the same walk to do as one of the real one.
//!
//! The addresses are the game's own, written from the same constants the reads use, so what this cannot
//! catch is a constant that is wrong: a wrong one is wrong on both sides at once, and only the real game
//! says otherwise — every one of them was read against `th06nc.exe`, see `th06nc/asm.md`. What it does
//! catch is everything built on top of them, which is where the code is.
//!
//! **The values are this file's own, though, and taking one from [`Th06nc`](super::Th06nc) is what is
//! refused here** — th07's image has why: a screen number, a state or a priority shared between the writer
//! and the reader is silent when wrong, and what it costs is a piece of the mode never happening. So each
//! value below carries its own literal and where in the exe it was read. **And a number read off th06c is
//! no number of this game's**: the two exes are the same engine and differ where it matters — this one's
//! table of keys has no `VK_SHIFT` in it at all, which cost every launch before it its focus.
//!
//! **And nothing here goes through [`super::at`]**: the slide is nought in a test, the exe being laid out
//! at the base its addresses are written against, and `Game::set_image_base` is never told otherwise.

use std::ops::Range;
use std::sync::Arc;

use orb_api::Kind;
use orb_sim::{Sim, Space};

/// The game's static data, as the real one is: **one range**, the `.data` section's own virtual size —
/// `0x14036d000` and `0x908884` bytes, which is what orb reads out of the PE and writes in the log every
/// run — `.data <base>..<base + 0x908884> (9472132 bytes)`. This is the range a chapter is a copy of, and
/// every static below is inside it.
const DATA: Range<usize> = 0x1_4036_d000..0x1_40c7_5884;

/// Stands for the game's own code, so that a pointer into it reads as a live COM object's vtable and one
/// anywhere else reads as the stale pointer left in a block the allocator did not scrub. The first page of
/// `.text`, which the section table puts at `0x140001000`.
const CODE: Range<usize> = 0x1_4000_1000..0x1_4000_2000;

/// **The page of `.rdata` the game's table from a virtual key to its own key index is on**, which the
/// write orb makes into the game's keys walks — see `super::THE_TABLE_OF_KEYS`. An image page rather than
/// data, as the section is: no snapshot copies it and no restore writes it.
const KEY_TABLE_PAGE: Range<usize> = 0x1_402f_6000..0x1_402f_7000;

/// **The keys this game is played with, as its own table maps them**: the virtual key, and the
/// DirectInput scan code the game keeps that key at — read out of the table at `0x1402f6060`, whose 110
/// entries are six bytes apiece and whose `A` is `0x41 -> 0x1e`. The nine orb presses and no others, since
/// what the table is for here is orb's writes reaching the buffer the game plays off.
///
/// **The focus is `VK_LSHIFT` and not `VK_SHIFT`**, which this table has no entry for at all — see
/// `super::the_key_the_games_table_holds`, which is the one reason a laid-out game can press it.
const KEYS: [(u16, u16); 9] = [
    (0x5a, 0x2c), // Z
    (0x58, 0x2d), // X
    (0xa0, 0x2a), // Left shift
    (0x1b, 0x01), // Escape
    (0x0d, 0x1c), // Enter
    (0x26, 0xc8), // Up
    (0x28, 0xd0), // Down
    (0x25, 0xcb), // Left
    (0x27, 0xcd), // Right
];
/// What ends that table, `0xffff` in the key of the entry after the last — entry 110 of the `0x82` the
/// walk will read.
const NO_MORE_KEYS: u16 = 0xffff;

/// **The blocks the game takes off its heap that orb reaches**, each a base of this file's choosing and
/// the length the game's own where one is known.
///
/// The front end, which orb reaches through the callback of the element it registers: long enough to hold
/// the field furthest into it, the state at `+0x168b0`.
const FRONT_END: Range<usize> = 0x1_5000_0000..0x1_5001_7000;
/// The screen a run ends at, `0xbb48` bytes at `0x14007bd98`, reached through the chain element it
/// registers and through nothing else.
const RESULT_SCREEN: Range<usize> = 0x1_5002_0000..0x1_5002_c000;
/// The chain elements: `0x40` bytes apiece, `0x1400573ae` allocating the front end's and `0x14006f408`
/// the screen a run ends at's. One block of them, a slot each — see [`Element`].
const ELEMENTS: Range<usize> = 0x1_5003_0000..0x1_5003_1000;
const ELEMENT_BYTES: usize = 0x40;
/// **Where the game's own `malloc` and `free` hand blocks out of**: one megabyte, given out from the front
/// and never taken back, the word at its start saying how far it has been given. A copy of the score file
/// is a few kilobytes and a launch installs one per settling of the mode, so nothing a test does reaches
/// the end.
const HEAP: Range<usize> = 0x1_5004_0000..0x1_5014_0000;
const HEAP_GIVEN: usize = HEAP.start;
const HEAP_FIRST_BLOCK: usize = HEAP.start + 0x10;
/// **The renderer's own block**, `super::THE_RENDERERS_BLOCK` bytes off the game's `malloc`, which
/// `super::THE_RENDERER` points at and which a restore leaves alone — see `Game::live_handles`. What the
/// laid-out game keeps in it is the batch its draws queue their sprites into.
const RENDERERS_BLOCK: Range<usize> = 0x1_501b_0000..0x1_501b_0000 + super::THE_RENDERERS_BLOCK;

/// **The object a conversation runs in**, which the stage's own asset load fills on the real game and
/// which `super::THE_DIALOGUE` points at.
///
/// **`super::THE_CONVERSATIONS_BYTES` long, which is the real allocation's own size** — `0x1400435a7`
/// asks `malloc` for `0x4228` — where this was `0x4000`, enough for the field furthest into it that is
/// read. What made the size matter is `Game::live_handles`: it holds the whole allocation back off the
/// pointer, so a block laid out shorter than the real one is a hold-back running past its own region,
/// and a test over the ranges a restore skips would be reading a number no launch of the real game has.
const DIALOGUE: Range<usize> = 0x1_501a_0000..0x1_501a_0000 + super::THE_CONVERSATIONS_BYTES;

/// **The swapchain this game presents through**, a block of its own outside [`DATA`] as the real one is
/// DXGI's rather than the exe's.
const SWAPCHAIN: Range<usize> = 0x1_5018_0000..0x1_5018_1000;
/// **And the factory that makes it**, which is how this game's swapchain comes to be orb's: the exe asks
/// for `CreateDXGIFactory2` by name — see `super::RENDERER_MADE` — and orb answers a factory whose
/// `CreateSwapChainForHwnd` it has written over, so the swapchain the game then makes is one orb took. A
/// launch of the real one says as much: `frame: the swapchain this game makes will be orb's to present
/// through`, and then `canvas: drawing through this game's own Direct3D 11, over 640x480`.
///
/// **Which is not th06c's way at all** — that game asks for no factory and orb searches its statics for an
/// object that answers as a swapchain. A laid-out th06nc that laid its swapchain out where those statics
/// point and never made it through a factory is a game orb draws nothing over, because nothing of orb's
/// ever looks there.
const FACTORY: Range<usize> = 0x1_5019_0000..0x1_5019_1000;
/// **And its vtable, which has to be inside [`CODE`]**: `crate::dxgi`'s search will not call through a
/// pointer whose vtable is anywhere but a mapped image — see `orb_api::mem::vtable_in_image`, which is
/// what tells an object from a field that merely reads as one. Far enough into the page that the
/// twenty-three slots `IDXGISwapChain1` has reach no further than its end.
const SWAPCHAIN_VTABLE: usize = CODE.start + 0x800;
/// And the factory's own, far enough from the swapchain's that neither's slots reach the other's.
const FACTORY_VTABLE: usize = CODE.start + 0x400;

/// **Which slot of that vtable each of the five is**, counted in pointers, with their own literals here as
/// every number in this file has: `crate::dxgi` reads the same five, and one taken from there would be
/// wrong on both sides at once — what it costs is orb reading a swapchain it laid out itself. `IUnknown`'s
/// three come first, then `IDXGIObject`'s four and `IDXGIDeviceSubObject`'s one before `Present`;
/// `Present1` comes after `IDXGISwapChain`'s rest and `IDXGISwapChain1`'s own four.
const QUERY_INTERFACE: usize = 0;
const RELEASE: usize = 2;
const PRESENT: usize = 8;
/// `GetDesc`, after `IDXGISwapChain`'s own `GetBuffer`, `SetFullscreenState` and `GetFullscreenState` —
/// which is how orb reads the window this game plays in.
const GET_DESC: usize = 12;
/// `ResizeBuffers`, after `GetDesc` — what the game makes its buffers the window's size again through.
const RESIZE_BUFFERS: usize = 13;
const PRESENT1: usize = 22;
/// And `IDXGIFactory2::CreateSwapChainForHwnd`, after `IUnknown`'s three, `IDXGIObject`'s four,
/// `IDXGIFactory`'s five and `IDXGIFactory1`'s two — the slot orb writes over.
const CREATE_SWAP_CHAIN_FOR_HWND: usize = 15;

/// Which slot of [`ELEMENTS`] each job the chain carries has.
#[derive(Clone, Copy)]
enum Element {
    Supervisor = 0,
    FrontEnd = 1,
    GameManager = 2,
    Player = 3,
    ResultScreen = 4,
    Ending = 5,
    Draw = 6,
}

impl Element {
    const fn at(self) -> usize {
        ELEMENTS.start + self as usize * ELEMENT_BYTES
    }
}

/// The element's own priority, its first word: `0x1400696b7` writes the player's as seven and the insert
/// right after it keeps the list in that order — `cmp WORD PTR [rax],si; jg` at `0x1400696c3`, which is
/// what says the list is sorted ascending and an element goes in front of the first one above it.
const ELEMENT_PRIORITY: usize = 0x0;
/// And the link back, which the same insert writes: `0x1400696db` reads `[rax+0x20]` as the element before
/// and `0x1400696e8` writes this one into its `+0x28`. Nothing of orb's follows it — a walk of the chain
/// reads `super::chain::NEXT` alone — and it is written here because the game writes it.
const ELEMENT_PREVIOUS: usize = 0x20;

/// **The calc chain's jobs, by the callback each element carries** — the addresses of the game's own
/// callbacks, because that is what orb matches on: `what_a_job_runs_on` finds the screen a run ends at by
/// `0x14006fa70` and `the_chain_holds` the ending by `0x14002c450`. A game laid out by hand dispatches on
/// these to functions of its own — see `fake::th06nc::Fake::job`.
///
/// Read off a launch's own `chain:` lines: `3 job(s): 0x14007b7f0 0x140009800 0x140077250` before anything
/// is up, `0x1400490a0` arriving with the front end and a stage's eight after it.
pub mod chain_job {
    pub const SUPERVISOR: usize = 0x1_4007_b7f0;
    pub const FRONT_END: usize = 0x1_4004_90a0;
    /// The game manager's own update, the first of a stage's eight.
    pub const GAME_MANAGER: usize = 0x1_4003_baa0;
    /// **And the player's own, the third of them** — which is what says a stage is running at all: this
    /// game's `in_a_stage` asks whether the chain holds *this* callback and not the manager's, the
    /// manager's element being registered a frame before the rest of a stage's arrive.
    pub const PLAYER: usize = 0x1_4006_9f30;
    pub const RESULT_SCREEN: usize = 0x1_4006_fa70;
    pub const ENDING: usize = 0x1_4002_c450;
}

/// **What a callback of the calc chain answers**, read off the walk at `0x14003d73a` onwards: two calls it
/// again, nought takes the element out through `0x140012e10`, three ends the walk with the frame carrying
/// on, and one — like anything else the walk has no case for — moves on to the next element and counts as
/// an element that ran. Four and five also end the walk, and six starts it over from the head; neither is
/// answered here.
pub mod chain_result {
    pub const CONTINUES: i32 = 1;
    pub const AGAIN: i32 = 2;
    pub const REMOVED: i32 = 0;
    pub const OVER: i32 = 3;
}

/// And their priorities, in the order a walk reaches them: the supervisor first at nought, the front end
/// at two, an ending at three, a stage's manager at four and the screen a run ends at at thirteen. Each is
/// the number its own registration writes into [`ELEMENT_PRIORITY`] — `0x14007c101` writing nought from a
/// cleared register, `0x14005744e` two, `0x14002c30e` three, `0x14003c0dd` loading four, and
/// `0x14007bde4` passing thirteen to the register at `0x14006f3f0`.
const CALC_SUPERVISOR: u16 = 0;
const CALC_FRONT_END: u16 = 2;
const CALC_ENDING: u16 = 3;
const CALC_GAME_MANAGER: u16 = 4;
/// And the player's own seven, which `0x1400696b7` writes into the element its registration allocated.
const CALC_PLAYER: u16 = 7;
const CALC_RESULT_SCREEN: u16 = 13;

/// **The supervisor's states**, by the number `super::supervisor::STATE` holds: one the front end, two a
/// run, three the stage after one, seven the screen a run that finished ends at and eight a run given up
/// — `0x14007bdfc` writing one into the state wanted, and `super::STATE_RUN`,
/// `super::STATE_THE_RUNS_OWN_END` and `super::STATE_RUN_OVER` where the other three are read. The
/// remaining seven of the twelve its job dispatches on are not read.
const SCENE_FRONT_END: i32 = 1;
const SCENE_RUN: i32 = 2;
const SCENE_REBUILDING: i32 = 3;
const SCENE_THE_RUNS_OWN_END: i32 = super::STATE_THE_RUNS_OWN_END;
const SCENE_RUN_OVER: i32 = 8;

/// **The front end's screens**, by the number its state holds: read off a launch's own
/// `front end: screen N` lines, which walked a fresh launch into a run as `1, 2, 3, 8, 9, 11, 13, 14, 24,
/// 25` and back out of one as `16, 17, 18`.
///
/// **A practice run's walk has one screen more**, `14, 22, 24, 25`, which is where the stage to be
/// practised is chosen — see `th06nc/asm.md`'s table of the screens. Nothing here lays 22 out: the
/// laid-out game is walked into a run by the screens above, and a recipe that aims its last press at a
/// chapter rather than at a screen walks either game.
pub const SCREEN_LOGO: i32 = 1;
pub const SCREEN_TITLE: i32 = 2;
pub const SCREEN_MENU: i32 = 3;
pub const SCREEN_ON_ITS_OWN: i32 = 8;
pub const SCREEN_DIFFICULTY: i32 = 9;
pub const SCREEN_CHARACTER: i32 = 11;
pub const SCREEN_SHOT_SELECT: i32 = 13;

/// **The three items of the main menu orb has a question over**: nought a run, one the Extra — which is
/// the order a fresh launch's taps of down walked, `0, 2, 3 … 9`, the Extra skipped while locked — and
/// two a stage practised on its own, whose banner a picture of the screen after it settled.
pub const MENU_ITEM_RUN: i32 = 0;
pub const MENU_ITEM_EXTRA: i32 = 1;
pub const MENU_ITEM_PRACTICE: i32 = 2;

/// **And the item the replays are behind**, which is orb's other question over this menu: a pointdevice
/// run's replays are orb's own list and not the screen item 4 opens — see
/// `super::front_end::ITEM_REPLAYS`, whose doc has the walk from that item to the game's own list.
pub const MENU_ITEM_REPLAYS: i32 = super::front_end::ITEM_REPLAYS;

/// **The result screen's states**, by the number `super::result_screen::STATE` holds: nine the name entry
/// a played run opens in and seventeen the way out a run asked for no name takes, which `0x14007bdde`
/// chooses between on `[0x14053d404]`; twelve the ranking, written at `0x14000cdc9` by the machine a run
/// over puts up; and fifteen the offer to save a replay, which the name entry's own end item hands on to
/// and which `Game::skip_replay_prompt` writes the way out over — a launch walked
/// `9, 15, 16, 10, 12, 14` with a decide each and photographed the question at fifteen.
pub const RESULT_TYPING_A_NAME: i32 = 9;
/// **And the item of the entry's own grid that ends it**, which reaching eight characters puts the cursor
/// on — see `super::result_screen::THE_END_ITEM`, and `Game::types_its_name`, which writes both.
pub const RESULT_THE_END_ITEM: i32 = super::result_screen::THE_END_ITEM;
/// **How many characters the entry takes**, which is where it puts the cursor on the item above.
pub const RESULT_NAME_BYTES: i32 = super::result_screen::NAME_BYTES as i32;
pub const RESULT_RANKING: i32 = 12;
pub const RESULT_SAVE_REPLAY_QUESTION: i32 = super::result_screen::SAVE_REPLAY_QUESTION;
pub const RESULT_LEAVING: i32 = super::result_screen::EXIT;

/// The stage the game counts from: one, `super::Game::run_start` reading the manager's own `+0x24`.
const STAGE_COUNTS_FROM: i32 = 1;

/// The player's four states — nought playing, one coming back, two the hit and three the
/// invulnerability — `super::player`'s own being where each is read.
///
/// **Playing is public because it is the one a hit lands in**, which a test that asks for a death has to
/// wait for: `State::unsettled` is the hit and the walk-in, so it answers false through the two hundred
/// and forty frames of invulnerability a stage's build hands the player — see
/// `a_confirmation_declined_of_th06nc`.
pub const PLAYER_PLAYING: u8 = 0;
const PLAYER_COMING_BACK: u8 = 1;
const PLAYER_HIT: u8 = 2;
const PLAYER_INVULNERABLE: u8 = 3;

/// An enemy that is there and is the boss: the top bit of its flags byte and the boss bit of the byte
/// after it, with the two solidity bits the hit test wants up — `super::enemy`'s own three.
const ENEMY_IN_USE: u8 = 0x80;
const ENEMY_SOLID_AND_TESTED: u8 = 0x07;
const ENEMY_IS_THE_BOSS: u8 = 0x08;

/// A bullet's one hitting state, `super::bullet::FLYING`'s number.
const BULLET_FLYING: u16 = 1;

/// `'CATK'`, which every card record carries from the layout at `0x14003c390`.
const CATK_MAGIC: u32 = u32::from_le_bytes(*b"CATK");
/// The record's own two size words, `0x180` and `0x180`, and its version — `0x15` here, where th06c's is
/// `0x10` over a record a quarter of the size.
const CATK_SIZES: u32 = 0x0180_0180;
const CATK_VERSION: u8 = 0x15;
/// Where a record keeps its own card index and how many times that card has been attempted, `+0x10` as a
/// word and `+0x3c` as one.
const CATK_INDEX: usize = 0x10;
const CATK_ATTEMPTS: usize = 0x3c;
/// And the name the card's own start copies in at `+0x18`, with the low byte of its own bytes' sum at
/// `+0x12` — the pair `0x1400279a0` compares before it counts an attempt.
const CATK_NAME: usize = 0x18;
const CATK_NAME_SUM: usize = 0x12;
const CATK_NAME_BYTES: usize = CATK_ATTEMPTS - CATK_NAME;
/// **The cap the file's own read holds a count down to**, `0x270f` — `0x14006e3f3` and `0x14006e3fd`
/// hold the words at `+0x3c` and `+0x3e` against it, and `0x140027a2e` holds the attempts against the
/// same as the game counts one. What the word after the attempts *is* has not been read, th06's record
/// keeping a capture count beside its attempts and nothing of this build having been read as one, so it
/// stands here by its offset alone.
const MOST_ATTEMPTS: u16 = 0x270f;
const CATK_WORD_AFTER_THE_ATTEMPTS: usize = 0x3e;

/// The two statics a laid-out game hands orb's hooks as their objects: the supervisor, which the frame's
/// own stretch of the loop runs beside, and the game manager, which the stage's build is entered with.
pub const SUPERVISOR_OBJECT: usize = super::SUPERVISOR;
pub const GAME_MANAGER_OBJECT: usize = super::GAME_MANAGER;

pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    /// In a host whose non-determinism is drawn from `seed`, which an e2e test names in its assertions so
    /// that a failure can be replayed. The one constructor, for the reason th06's says.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        let space = sim.space();
        space.map(DATA.start, DATA.len(), Kind::Private);
        space.map(CODE.start, CODE.len(), Kind::Image);
        space.map(KEY_TABLE_PAGE.start, KEY_TABLE_PAGE.len(), Kind::Image);
        space.map(FRONT_END.start, FRONT_END.len(), Kind::Private);
        space.map(RESULT_SCREEN.start, RESULT_SCREEN.len(), Kind::Private);
        space.map(ELEMENTS.start, ELEMENTS.len(), Kind::Private);
        space.map(HEAP.start, HEAP.len(), Kind::Private);
        space.map(RENDERERS_BLOCK.start, RENDERERS_BLOCK.len(), Kind::Private);
        space.map(SWAPCHAIN.start, SWAPCHAIN.len(), Kind::Private);
        space.map(FACTORY.start, FACTORY.len(), Kind::Private);
        space.map(DIALOGUE.start, DIALOGUE.len(), Kind::Private);
        let image = Self { sim };
        image.space().write::<usize>(HEAP_GIVEN, HEAP_FIRST_BLOCK);
        // The table the game's own read maps a virtual key through, which is what orb's writes into the
        // game's keys go through too.
        let mut entry = super::THE_TABLE_OF_KEYS;
        for (key, index) in KEYS {
            image.space().write::<u16>(entry, key);
            image.space().write::<u16>(entry + size_of::<u16>(), index);
            entry += super::A_KEY_IN_THE_TABLE;
        }
        image.space().write::<u16>(entry, NO_MORE_KEYS);
        // The field's own size and the box the player is held to, as `.data` initialises the first and
        // the game fills the second: 384 by 448 at the origin.
        image.the_play_field_is(0.0, 0.0, 384.0, 448.0);
        // No track playing, which is what a run's own start writes.
        image.space().write::<i32>(super::THE_TRACK_PLAYING, -1);
        // The renderer's block where orb reads its pointer, so that `Game::live_handles` names a range
        // rather than nothing.
        image
            .space()
            .write::<usize>(super::THE_RENDERER, RENDERERS_BLOCK.start);
        // The object a conversation runs in, with no line up: the pointer is there from the start, as it
        // is on the real game from the frame a stage's assets are loaded, and `-1` is what says no line
        // stands — see `no_line_of_dialogue_is_up`.
        image
            .space()
            .write::<usize>(super::THE_DIALOGUE, DIALOGUE.start);
        image.no_line_of_dialogue_is_up();
        image
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is
    /// what makes `Th06nc`'s reads land in it.
    pub fn enter(&self) -> orb_api::Installed {
        self.sim.enter()
    }

    pub fn space(&self) -> &Space {
        self.sim.space()
    }

    /// The host this game is laid out in, for an e2e test that needs more of it than the memory.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }

    /// The range a chapter is a copy of — what `orb::attach` reads out of the PE and hands the runtime.
    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// **Hands over every call orb makes into this game's code**, in place of the addresses `Th06nc`
    /// holds: the calc walk, the camera, the frame's own end, the three of the music's catch-up, and the
    /// three the window's own sizing makes — and the draw's own, see [`Image::hands_over_the_draws_own_calls`].
    /// All together, because orb's frame makes them all: a game that handed over fewer would be jumping
    /// into memory nothing has mapped part way through one frame — which is what the window's three did
    /// before they were here, on the first frame of every e2e test.
    #[allow(clippy::too_many_arguments)]
    pub fn hands_over_the_frames_own_calls(
        &self,
        run_calc_chain: usize,
        moves_the_camera: usize,
        ends_the_frame: usize,
        where_the_music_is: usize,
        how_long_the_track_is: usize,
        seeks_the_music: usize,
        makes_a_window: usize,
        covers_the_monitor: usize,
        sizes_the_picture: usize,
    ) {
        super::set_run_calc_chain(run_calc_chain);
        super::set_moves_the_camera(moves_the_camera);
        super::set_ends_the_frame(ends_the_frame);
        super::set_where_the_music_is(where_the_music_is);
        super::set_how_long_the_track_is(how_long_the_track_is);
        super::set_seeks_the_music(seeks_the_music);
        super::set_makes_a_window(makes_a_window);
        super::set_covers_the_monitor(covers_the_monitor);
        super::set_sizes_the_picture(sizes_the_picture);
    }

    /// **And the calls the draw makes**, the frame's own inline copy of what was a function of its own —
    /// see `super::walks_the_draw`: its start, the viewport read and set, the clear, the take-out of an
    /// element, the sprites flushed and the screenshot's path.
    #[allow(clippy::too_many_arguments)]
    pub fn hands_over_the_draws_own_calls(
        &self,
        begins_the_draw: usize,
        reads_the_viewport: usize,
        sets_the_viewport: usize,
        clears: usize,
        takes_an_element_out: usize,
        flushes_the_sprites: usize,
        takes_the_screenshot: usize,
    ) {
        super::set_begins_the_draw(begins_the_draw);
        super::set_reads_the_viewport(reads_the_viewport);
        super::set_sets_the_viewport(sets_the_viewport);
        super::set_clears(clears);
        super::set_takes_an_element_out(takes_an_element_out);
        super::set_flushes_the_sprites(flushes_the_sprites);
        super::set_takes_the_screenshot(takes_the_screenshot);
    }

    // ---- the renderer and the draw chain ----

    /// **Sprites a draw has queued**, `count` vertices more in the renderer's batch — which is how every one
    /// of this game's draws hands its sprites over.
    pub fn queues_sprites(&self, count: i32) {
        let queued = RENDERERS_BLOCK.start + super::renderer::QUEUED;
        let now = self.space().read::<i32>(queued);
        self.space().write::<i32>(queued, now + count);
    }

    /// How many vertices the renderer's batch holds, not yet drawn.
    pub fn the_sprites_queued(&self) -> i32 {
        self.space()
            .read::<i32>(RENDERERS_BLOCK.start + super::renderer::QUEUED)
    }

    /// **The draw chain's one job**, registered at the head as the real game's own jobs are — whatever a
    /// laid-out game draws with.
    pub fn registers_in_the_draw_chain(&self, callback: usize, on: usize) {
        let space = self.space();
        let elem = Element::Draw.at();
        space.write::<usize>(elem + super::chain::CALLBACK, callback);
        space.write::<usize>(elem + super::chain::ON, on);
        space.write::<usize>(elem + super::chain::NEXT, 0);
        space.write::<usize>(super::THE_DRAW_CHAIN + super::chain::NEXT, elem);
    }

    /// And the game's own run start, `0x14004f320`, which orb calls where it writes a run into being —
    /// see `Th06nc::start_a_run`. Apart from the nine above because nothing of the frame calls it.
    pub fn hands_over_the_run_start(&self, run_start: usize) {
        super::set_run_start(run_start);
    }

    // ---- the supervisor ----

    /// The state its job has acted on and the state wanted, which is the pair `Game::run_state` and
    /// `Game::say_run_state` read.
    pub fn is_in_scene(&self, scene: i32, wanted: i32) {
        let space = self.space();
        space.write::<i32>(super::SUPERVISOR + super::supervisor::STATE, scene);
        space.write::<i32>(super::SUPERVISOR + super::supervisor::WANTED, wanted);
    }

    pub fn the_scene_now(&self) -> i32 {
        self.space()
            .read(super::SUPERVISOR + super::supervisor::STATE)
    }

    pub fn the_scene_wanted(&self) -> i32 {
        self.space()
            .read(super::SUPERVISOR + super::supervisor::WANTED)
    }

    pub fn scene_of_the_front_end(&self) -> i32 {
        SCENE_FRONT_END
    }

    pub fn scene_of_a_run(&self) -> i32 {
        SCENE_RUN
    }

    pub fn scene_of_a_stage_being_rebuilt(&self) -> i32 {
        SCENE_REBUILDING
    }

    /// The state a run that *finished* ends in, which is the one the screen a run ends at is built by —
    /// see `super::STATE_THE_RUNS_OWN_END`, and [`Self::scene_of_a_run_over`] for the one a run given up
    /// reaches instead, which builds no screen.
    pub fn scene_of_the_runs_own_end(&self) -> i32 {
        SCENE_THE_RUNS_OWN_END
    }

    pub fn scene_of_a_run_over(&self) -> i32 {
        SCENE_RUN_OVER
    }

    // ---- the chain ----

    /// The element linked into the calc list by its priority, behind every element of a lower or equal
    /// one, with the callback and the object it is called on.
    pub fn registers_in_the_calc_chain(
        &self,
        elem: usize,
        priority: u16,
        callback: usize,
        on: usize,
    ) {
        let space = self.space();
        space.write::<u16>(elem + ELEMENT_PRIORITY, priority);
        space.write::<usize>(elem + super::chain::CALLBACK, callback);
        space.write::<usize>(elem + super::chain::ON, on);
        let mut link = super::THE_CALC_CHAIN + super::chain::NEXT;
        let mut before = super::THE_CALC_CHAIN;
        loop {
            let next: usize = space.read(link);
            if next == 0 || space.read::<u16>(next + ELEMENT_PRIORITY) > priority {
                space.write::<usize>(elem + super::chain::NEXT, next);
                space.write::<usize>(elem + ELEMENT_PREVIOUS, before);
                if next != 0 {
                    space.write::<usize>(next + ELEMENT_PREVIOUS, elem);
                }
                space.write::<usize>(link, elem);
                return;
            }
            before = next;
            link = next + super::chain::NEXT;
        }
    }

    /// The element unlinked from the calc list, and its links and its callback cleared — which is what a
    /// job that answers *take me out* leaves behind.
    pub fn cuts_from_the_chain(&self, elem: usize) {
        let space = self.space();
        let mut link = super::THE_CALC_CHAIN + super::chain::NEXT;
        loop {
            let next: usize = space.read(link);
            if next == 0 {
                return;
            }
            if next == elem {
                let after: usize = space.read(elem + super::chain::NEXT);
                space.write::<usize>(link, after);
                if after != 0 {
                    space.write::<usize>(
                        after + ELEMENT_PREVIOUS,
                        space.read(elem + ELEMENT_PREVIOUS),
                    );
                }
                space.write::<usize>(elem + super::chain::NEXT, 0);
                space.write::<usize>(elem + ELEMENT_PREVIOUS, 0);
                space.write::<usize>(elem + super::chain::CALLBACK, 0);
                return;
            }
            link = next + super::chain::NEXT;
        }
    }

    /// The head of the calc list, whose own element is the sentinel a walk starts from.
    pub fn calc_chain_head(&self) -> usize {
        super::THE_CALC_CHAIN
    }

    pub fn chain_callback(&self, elem: usize) -> usize {
        self.space().read(elem + super::chain::CALLBACK)
    }

    pub fn chain_next(&self, elem: usize) -> usize {
        self.space().read(elem + super::chain::NEXT)
    }

    pub fn chain_on(&self, elem: usize) -> usize {
        self.space().read(elem + super::chain::ON)
    }

    /// Every job the calc list holds, as the callback each element carries, in the order a walk reaches
    /// them.
    pub fn calc_chain_jobs(&self) -> Vec<usize> {
        let mut jobs = Vec::new();
        let mut at: usize = self.chain_next(super::THE_CALC_CHAIN);
        while at != 0 {
            jobs.push(self.chain_callback(at));
            at = self.chain_next(at);
        }
        jobs
    }

    /// The supervisor's own job, which is the chain's first and the one every other goes in above.
    pub fn registers_the_supervisor(&self) {
        self.registers_in_the_calc_chain(
            Element::Supervisor.at(),
            CALC_SUPERVISOR,
            chain_job::SUPERVISOR,
            super::SUPERVISOR,
        );
    }

    /// The front end's job, on the block it is built in — which is the one way orb reaches that block.
    pub fn registers_the_front_end(&self) {
        self.registers_in_the_calc_chain(
            Element::FrontEnd.at(),
            CALC_FRONT_END,
            chain_job::FRONT_END,
            FRONT_END.start,
        );
    }

    /// And out again, which is what a run starting does to it.
    pub fn cuts_the_front_end(&self) {
        self.cuts_from_the_chain(Element::FrontEnd.at());
    }

    /// Where that block is, for a laid-out game whose own front end is a screen number in it.
    pub fn the_front_end(&self) -> usize {
        FRONT_END.start
    }

    /// **A stage's own jobs, of which orb reads two**: the game manager's update on the static a run is
    /// made of, and the player's own on the player — which is the one that says a stage is running, see
    /// [`chain_job::PLAYER`]. Both, because a laid-out game with only the manager's is one `in_a_stage`
    /// answers false for on every frame of a run: the recipe walks the whole front end, the stage is
    /// built, and no run was ever started as far as anything above `Game` can tell.
    pub fn registers_the_gameplay_scene(&self) {
        self.registers_in_the_calc_chain(
            Element::GameManager.at(),
            CALC_GAME_MANAGER,
            chain_job::GAME_MANAGER,
            super::GAME_MANAGER,
        );
        self.registers_in_the_calc_chain(
            Element::Player.at(),
            CALC_PLAYER,
            chain_job::PLAYER,
            super::PLAYER,
        );
    }

    pub fn cuts_the_gameplay_scene(&self) {
        self.cuts_from_the_chain(Element::GameManager.at());
        self.cuts_from_the_chain(Element::Player.at());
    }

    /// An ending running, which orb reads as the chain holding its job.
    pub fn an_ending_is_running(&self) {
        self.registers_in_the_calc_chain(
            Element::Ending.at(),
            CALC_ENDING,
            chain_job::ENDING,
            super::SUPERVISOR,
        );
    }

    pub fn the_ending_is_over(&self) {
        self.cuts_from_the_chain(Element::Ending.at());
    }

    // ---- the screen a run ends at ----

    /// The screen a run ends at, up in `state` — its job registered on its own block, which is the one way
    /// orb reaches it, and the byte that says the screen is a run's own rather than the ranking the title
    /// opens.
    pub fn the_screen_a_run_ends_at_is_up(&self, state: i32) {
        self.space()
            .write::<u8>(RESULT_SCREEN.start + super::result_screen::A_RUNS_OWN, 1);
        self.the_screen_a_run_ends_at_is_in(state);
        self.registers_in_the_calc_chain(
            Element::ResultScreen.at(),
            CALC_RESULT_SCREEN,
            chain_job::RESULT_SCREEN,
            RESULT_SCREEN.start,
        );
    }

    pub fn the_screen_a_run_ends_at_is_in(&self, state: i32) {
        self.space()
            .write::<i32>(RESULT_SCREEN.start + super::result_screen::STATE, state);
    }

    pub fn the_state_the_screen_a_run_ends_at_is_in(&self) -> i32 {
        self.space()
            .read(RESULT_SCREEN.start + super::result_screen::STATE)
    }

    pub fn the_screen_a_run_ends_at_is_down(&self) {
        self.cuts_from_the_chain(Element::ResultScreen.at());
    }

    pub fn the_screen_a_run_ends_at(&self) -> usize {
        RESULT_SCREEN.start
    }

    /// The name the entry holds and how many characters of it have been typed, which is what orb reads to
    /// say a run has been named.
    pub fn the_entry_holds(&self, name: &str) {
        let space = self.space();
        let mut bytes = [0u8; super::result_screen::NAME_BYTES];
        for (slot, byte) in bytes.iter_mut().zip(name.bytes()) {
            *slot = byte;
        }
        space.write_bytes(RESULT_SCREEN.start + super::result_screen::NAME, &bytes);
        space.write::<i32>(
            RESULT_SCREEN.start + super::result_screen::TYPED,
            i32::try_from(name.len().min(super::result_screen::NAME_BYTES))
                .expect("a name of eight bytes at most"),
        );
    }

    pub fn the_name_the_entry_holds(&self) -> String {
        let bytes = self.space().read_bytes(
            RESULT_SCREEN.start + super::result_screen::NAME,
            super::result_screen::NAME_BYTES,
        );
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    }

    pub fn the_characters_typed(&self) -> i32 {
        self.space()
            .read(RESULT_SCREEN.start + super::result_screen::TYPED)
    }

    pub fn the_entrys_cursor(&self) -> i32 {
        self.space()
            .read(RESULT_SCREEN.start + super::result_screen::CURSOR)
    }

    pub fn the_entrys_cursor_is_on(&self, item: i32) {
        self.space()
            .write::<i32>(RESULT_SCREEN.start + super::result_screen::CURSOR, item);
    }

    // ---- the front end ----

    /// Which screen the front end is on and which item its cursor is on, which is the pair
    /// `Game::menu_pointed_at` reads.
    pub fn the_front_end_is_on(&self, screen: i32, cursor: i32) {
        let space = self.space();
        space.write::<i32>(FRONT_END.start + super::front_end::STATE, screen);
        space.write::<i32>(FRONT_END.start + super::front_end::CURSOR, cursor);
    }

    pub fn the_screen_the_front_end_is_on(&self) -> i32 {
        self.space().read(FRONT_END.start + super::front_end::STATE)
    }

    pub fn the_item_the_cursor_is_on(&self) -> i32 {
        self.space()
            .read(FRONT_END.start + super::front_end::CURSOR)
    }

    pub fn the_cursor_is_on(&self, item: i32) {
        self.space()
            .write::<i32>(FRONT_END.start + super::front_end::CURSOR, item);
    }

    /// How long the screen the front end is on has been up, which is what the main menu holds its own
    /// twenty-frame grace against — see `Game::menu_takes_a_press`.
    pub fn frames_the_screen_has_been_up(&self) -> i32 {
        self.space()
            .read(FRONT_END.start + super::front_end::SCREEN_FRAMES)
    }

    pub fn the_screen_has_been_up_for(&self, frames: i32) {
        self.space()
            .write::<i32>(FRONT_END.start + super::front_end::SCREEN_FRAMES, frames);
    }

    /// **The item the menu was decided on asking for a stage practised on its own**, which is the byte
    /// the real one's item 2 writes and its item 0 clears — see
    /// [`super::front_end::THE_RUN_IS_PRACTICE`]. What `Game::run_pointed_at` answers `practice` by, a
    /// screen before the run's own start copies it into [`super::A_RUN_IS_PRACTICE`].
    pub fn the_item_asks_for_a_practice(&self, practice: bool) {
        self.space().write::<u8>(
            FRONT_END.start + super::front_end::THE_RUN_IS_PRACTICE,
            u8::from(practice),
        );
    }

    pub fn does_the_item_ask_for_a_practice(&self) -> bool {
        self.space()
            .read::<u8>(FRONT_END.start + super::front_end::THE_RUN_IS_PRACTICE)
            != 0
    }

    // ---- the game manager ----

    /// A run in the stage and at the difficulty, which is what `Game::run_start` and `Game::run_slot`
    /// read.
    pub fn is_playing(&self, stage: i32, difficulty: i32) {
        let space = self.space();
        space.write::<i32>(super::STAGE, stage);
        space.write::<i32>(super::DIFFICULTY, difficulty);
    }

    /// And what the shot select decided, which is the rest of a run's own slot.
    pub fn the_shot_select_decided(&self, difficulty: i32, character: u8, shot: u8) {
        let space = self.space();
        space.write::<i32>(super::DIFFICULTY, difficulty);
        space.write::<u8>(
            super::GAME_MANAGER + super::game_manager::CHARACTER,
            character,
        );
        space.write::<u8>(super::GAME_MANAGER + super::game_manager::SHOT, shot);
    }

    pub fn the_difficulty_is(&self, difficulty: i32) {
        self.space().write::<i32>(super::DIFFICULTY, difficulty);
    }

    pub fn the_difficulty_now(&self) -> i32 {
        self.space().read(super::DIFFICULTY)
    }

    pub fn the_character_chosen(&self) -> u8 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::CHARACTER)
    }

    pub fn the_shot_chosen(&self) -> u8 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::SHOT)
    }

    pub fn the_stage_the_run_is_in(&self) -> i32 {
        self.space().read(super::STAGE)
    }

    pub fn the_stage_a_run_counts_from(&self) -> i32 {
        STAGE_COUNTS_FROM
    }

    pub fn has_lives_and_bombs(&self, lives: i8, bombs: i8) {
        let space = self.space();
        space.write::<i8>(super::LIVES, lives);
        space.write::<i8>(super::BOMBS, bombs);
    }

    pub fn lives_and_bombs(&self) -> (i8, i8) {
        let space = self.space();
        (space.read(super::LIVES), space.read(super::BOMBS))
    }

    pub fn shot_is_at_power(&self, power: u16) {
        self.space().write::<u16>(super::POWER, power);
    }

    pub fn the_shots_power(&self) -> u16 {
        self.space().read(super::POWER)
    }

    /// What the run has scored and how many point items it has taken, which is two of the eight numbers a
    /// chapter is written down with.
    ///
    /// **One score field and not two**, which is what two launches of the real game say: the qword at
    /// the manager's `+0x938` reads nought across a stage boundary while `super::game_manager::SCORE`
    /// holds the run's — so what the panel draws and what a chapter records are the same field here, and
    /// a laid-out game with a second one would be laying out a claim nothing measured.
    pub fn the_run_has_scored(&self, score: u64, point_items: u16) {
        let space = self.space();
        space.write::<u64>(super::GAME_MANAGER + super::game_manager::SCORE, score);
        space.write::<u16>(
            super::GAME_MANAGER + super::game_manager::POINT_ITEMS,
            point_items,
        );
    }

    pub fn the_score(&self) -> u64 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::SCORE)
    }

    pub fn the_point_items(&self) -> u16 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::POINT_ITEMS)
    }

    /// **What a point item is worth**, which is not how many have been taken — see
    /// [`game_manager::POINT_ITEM_VALUE`](super::game_manager::POINT_ITEM_VALUE).
    pub fn a_point_item_is_worth(&self, step: u8) {
        self.space().write::<u8>(
            super::GAME_MANAGER + super::game_manager::POINT_ITEM_VALUE,
            step,
        );
    }

    pub fn what_a_point_item_is_worth(&self) -> u8 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::POINT_ITEM_VALUE)
    }

    pub fn the_score_has_paid_for(&self, extra_lives: i8) {
        self.space().write::<i8>(
            super::GAME_MANAGER + super::game_manager::EXTRA_LIVES,
            extra_lives,
        );
    }

    pub fn the_extra_lives_the_score_paid_for(&self) -> i8 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::EXTRA_LIVES)
    }

    /// The seed the stage began with, which the manager keeps its own copy of and which a chapter is
    /// written down with.
    pub fn the_stage_began_with_the_seed(&self, seed: u16) {
        self.space()
            .write::<u16>(super::GAME_MANAGER + super::game_manager::RANDOM_SEED, seed);
    }

    pub fn the_seed_the_stage_began_with(&self) -> u16 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::RANDOM_SEED)
    }

    /// The generator itself: the seed it is at and how many numbers have been drawn from it.
    pub fn the_generator_is(&self, seed: u16, randoms: u32) {
        let space = self.space();
        space.write::<u16>(super::RNG_SEED, seed);
        space.write::<u32>(super::RNG_COUNT, randoms);
    }

    pub fn the_generator(&self) -> (u16, u32) {
        let space = self.space();
        (space.read(super::RNG_SEED), space.read(super::RNG_COUNT))
    }

    /// One number drawn from it, which is what every frame of a stage and every step of a build does: the
    /// seed moved on and the count up one. The multiplier and the addend are this laid-out game's own —
    /// what matters above `Game` is that a seed put back draws the same numbers again and another seed
    /// draws others.
    pub fn draws_from_the_generator(&self) -> u16 {
        let (seed, randoms) = self.the_generator();
        let drawn = seed.wrapping_mul(0x5d59).wrapping_add(0x3f);
        self.the_generator_is(drawn, randoms.wrapping_add(1));
        drawn
    }

    /// How long the run has been in the stage, which is the clock a chapter is placed on.
    pub fn has_been_in_the_stage_for(&self, frames: u32) {
        self.space().write::<u32>(
            super::GAME_MANAGER + super::game_manager::GAME_FRAMES,
            frames,
        );
    }

    pub fn frames_in_the_stage(&self) -> u32 {
        self.space()
            .read(super::GAME_MANAGER + super::game_manager::GAME_FRAMES)
    }

    /// Paused, which is the pause menu's own byte and the retry menu's beside it.
    pub fn is_paused(&self) {
        self.space().write::<u8>(
            super::GAME_MANAGER + super::game_manager::IS_IN_GAME_MENU,
            1,
        );
    }

    pub fn is_not_paused(&self) {
        let space = self.space();
        space.write::<u8>(
            super::GAME_MANAGER + super::game_manager::IS_IN_GAME_MENU,
            0,
        );
        space.write::<u8>(
            super::GAME_MANAGER + super::game_manager::IS_IN_RETRY_MENU,
            0,
        );
    }

    pub fn is_in_the_retry_menu(&self) {
        self.space().write::<u8>(
            super::GAME_MANAGER + super::game_manager::IS_IN_RETRY_MENU,
            1,
        );
    }

    pub fn is_it_paused(&self) -> bool {
        let space = self.space();
        space.read::<u8>(super::GAME_MANAGER + super::game_manager::IS_IN_GAME_MENU) != 0
            || space.read::<u8>(super::GAME_MANAGER + super::game_manager::IS_IN_RETRY_MENU) != 0
    }

    pub fn everything_is_held(&self, held: bool) {
        self.space()
            .write::<u8>(super::EVERYTHING_HELD, u8::from(held));
    }

    pub fn a_replay_is_playing(&self, playing: bool) {
        self.space()
            .write::<u8>(super::A_REPLAY_IS_PLAYING, u8::from(playing));
    }

    /// **The run under way being a stage practised on its own**, the static the real one's run start
    /// writes from the front end's own byte and `Game::read_state` answers `practice` off.
    pub fn a_run_is_practice(&self, practice: bool) {
        self.space()
            .write::<u8>(super::A_RUN_IS_PRACTICE, u8::from(practice));
    }

    pub fn is_the_run_a_practice(&self) -> bool {
        self.space().read::<u8>(super::A_RUN_IS_PRACTICE) != 0
    }

    /// **And the other byte one stage or less of a run is played under**: the scene the main menu's item 3
    /// starts — this build's Spell Practice — see `super::A_SCENE_IS_BEING_PLAYED`.
    pub fn a_scene_is_being_played(&self, scene: bool) {
        self.space()
            .write::<u8>(super::A_SCENE_IS_BEING_PLAYED, u8::from(scene));
    }

    /// **Which of the game's two modes the run is in**, the byte screen 25 writes out of the
    /// configuration and `Game::run_slot` names the slot by — see
    /// `super::A_RUN_IS_IN_CHALLENGE_MODE`.
    pub fn the_run_is_in_challenge_mode(&self, challenge: bool) {
        self.space()
            .write::<u8>(super::A_RUN_IS_IN_CHALLENGE_MODE, u8::from(challenge));
    }

    /// **And the mode the next run will be in**, the configuration's own byte that screen 14 writes —
    /// see `super::THE_MODE_LAST_CHOSEN`, which is what `Game::run_pointed_at` reads.
    pub fn the_mode_last_chosen_is_challenge(&self, challenge: bool) {
        self.space()
            .write::<u8>(super::THE_MODE_LAST_CHOSEN, u8::from(challenge));
    }

    // ---- the player and what is on the field ----

    pub fn the_player_is_at(&self, x: f32, y: f32) {
        let space = self.space();
        space.write::<f32>(super::PLAYER + super::player::POSITION, x);
        space.write::<f32>(
            super::PLAYER + super::player::POSITION + size_of::<f32>(),
            y,
        );
    }

    pub fn where_the_player_is(&self) -> (f32, f32) {
        let space = self.space();
        (
            space.read(super::PLAYER + super::player::POSITION),
            space.read(super::PLAYER + super::player::POSITION + size_of::<f32>()),
        )
    }

    /// **The phase of the shot the player is firing, and the phase the update before it had** — see
    /// `super::player::SHOT_PHASE`, whose pair this is.
    pub fn the_shot_phase_is(&self, phase: i32, before: i32) {
        let space = self.space();
        space.write::<i32>(super::PLAYER + super::player::SHOT_PHASE, phase);
        space.write::<i32>(super::PLAYER + super::player::SHOT_PHASE_BEFORE, before);
    }

    pub fn the_shot_phase(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read(super::PLAYER + super::player::SHOT_PHASE),
            space.read(super::PLAYER + super::player::SHOT_PHASE_BEFORE),
        )
    }

    /// The field the run is played in, as the game fills the manager's own rectangle: left, top, width
    /// and height.
    pub fn the_play_field_is(&self, left: f32, top: f32, width: f32, height: f32) {
        let space = self.space();
        let area = super::PLAYER_AREA;
        for (slot, value) in [left, top, width, height].into_iter().enumerate() {
            space.write::<f32>(area + slot * size_of::<f32>(), value);
        }
    }

    /// **What one frame of holding a direction moves the player by**, as the game keeps it: a speed per
    /// kind of move and the player's own scale for each axis, which `sight`'s `a_step` multiplies
    /// together. The focused pair is half the straight one, the way this game's own is.
    ///
    /// **Without this the rules cannot move at all.** `a_step` falls back to a probe for a speed that
    /// reads as nought, so a laid-out player with no speeds written is one the bot sees as unable to
    /// step — and a laid-out run then dies at the same frame of the stage on every retry, having dodged
    /// nothing.
    pub fn the_player_steps_by(&self, straight: f32, diagonal: f32) {
        let space = self.space();
        space.write::<f32>(super::PLAYER + super::player::SCALE_ACROSS, 1.0);
        space.write::<f32>(super::PLAYER + super::player::SCALE_DOWN, 1.0);
        space.write::<f32>(super::PLAYER + super::player::SPEED_STRAIGHT, straight);
        space.write::<f32>(super::PLAYER + super::player::SPEED_DIAGONAL, diagonal);
        space.write::<f32>(
            super::PLAYER + super::player::SPEED_STRAIGHT_FOCUSED,
            straight / 2.0,
        );
        space.write::<f32>(
            super::PLAYER + super::player::SPEED_DIAGONAL_FOCUSED,
            diagonal / 2.0,
        );
    }

    /// **And the box a hit is tested against**, which this game keeps as one radius — `sight`'s
    /// `player_half` reads it and squares it up.
    pub fn the_players_box_is(&self, half: f32) {
        self.space()
            .write::<f32>(super::PLAYER + super::player::RADIUS, half);
    }

    pub fn the_player_is_playing(&self) {
        self.space()
            .write::<u8>(super::PLAYER + super::player::STATE, PLAYER_PLAYING);
    }

    pub fn the_player_has_been_hit(&self) {
        self.space()
            .write::<u8>(super::PLAYER + super::player::STATE, PLAYER_HIT);
    }

    pub fn the_players_state(&self) -> u8 {
        self.space().read(super::PLAYER + super::player::STATE)
    }

    /// **The player coming back, with the frames the state has left beside it** — the pair a stage's own
    /// register writes and the pair a death leaves, `player::COMING_BACK` and `player::UNTOUCHABLE`.
    pub fn the_player_is_coming_back(&self, frames: i32) {
        let space = self.space();
        space.write::<u8>(super::PLAYER + super::player::STATE, PLAYER_COMING_BACK);
        space.write::<i32>(super::PLAYER + super::player::UNTOUCHABLE, frames);
    }

    /// And the invulnerability the walk-in hands over to, which counts itself down to nothing.
    pub fn the_player_is_invulnerable_for(&self, frames: i32) {
        let space = self.space();
        space.write::<u8>(super::PLAYER + super::player::STATE, PLAYER_INVULNERABLE);
        space.write::<i32>(super::PLAYER + super::player::UNTOUCHABLE, frames);
    }

    pub fn the_frames_the_player_is_untouchable_for(&self) -> i32 {
        self.space()
            .read(super::PLAYER + super::player::UNTOUCHABLE)
    }

    /// **A bomb in the air, or not** — `player::BOMB_IN_USE`, the byte the bomb's own start writes and
    /// each of the game's four bomb ends clears.
    pub fn a_bomb_is_going_off(&self, going: bool) {
        self.space()
            .write::<u8>(super::PLAYER + super::player::BOMB_IN_USE, u8::from(going));
    }

    /// **How many enemies are alive, in the manager's own first dword** — `enemy_manager::LIVE`, which is
    /// the count where `enemy_manager::ENEMIES` is the array.
    pub fn enemies_alive(&self, enemies: i32) {
        self.space()
            .write::<i32>(super::ENEMY_MANAGER + super::enemy_manager::LIVE, enemies);
    }

    /// **The card the fight is on**, which is the two the card's own start writes: the byte that says one
    /// is up and the index the records are looked up by.
    pub fn the_spellcard_is(&self, card: u32) {
        let space = self.space();
        space.write::<u8>(super::ENEMY_MANAGER + super::enemy_manager::A_CARD_IS_UP, 1);
        space.write::<u32>(super::ENEMY_MANAGER + super::enemy_manager::CARD_NOW, card);
    }

    pub fn no_spellcard_is_running(&self) {
        self.space()
            .write::<u8>(super::ENEMY_MANAGER + super::enemy_manager::A_CARD_IS_UP, 0);
    }

    /// **Where the object a conversation runs in is laid out**, for a test that has to name the range a
    /// restore holds back: the base a real game's `[THE_DIALOGUE]` holds, whose length is
    /// `super::THE_CONVERSATIONS_BYTES`.
    pub fn the_conversation(&self) -> usize {
        DIALOGUE.start
    }

    /// **The line of a conversation that is up**, written where the dialogue's own start writes it: the
    /// object behind `super::THE_DIALOGUE`, at `super::dialogue::LINE`.
    pub fn a_line_of_dialogue_is_up(&self, line: i32) {
        self.space()
            .write::<i32>(DIALOGUE.start + super::dialogue::LINE, line);
    }

    /// **And none up**, which is the `-1` the two ends of a conversation write.
    pub fn no_line_of_dialogue_is_up(&self) {
        self.a_line_of_dialogue_is_up(-1);
    }

    /// Which line the dialogue object says is up, as `Th06nc` reads it.
    pub fn the_line_of_dialogue_up(&self) -> i32 {
        self.space().read(DIALOGUE.start + super::dialogue::LINE)
    }

    /// **How long the stage's own script has run**, in the manager's own tail —
    /// `enemy_manager::TIMELINE_TIME`, which the enemies' job steps once a frame.
    pub fn the_script_has_run_for(&self, frames: i32) {
        self.space().write::<i32>(
            super::ENEMY_MANAGER + super::enemy_manager::TIMELINE_TIME,
            frames,
        );
    }

    /// And the bullets', which is a long way into that static — `bullet_manager::LIVE`, past all `0x280`
    /// slots of it.
    pub fn bullets_alive(&self, bullets: i32) {
        self.space()
            .write::<i32>(super::BULLET_MANAGER + super::bullet_manager::LIVE, bullets);
    }

    /// A boss fighting, in the first enemy slot: alive, solid, tested, the boss, and with the life it has
    /// left.
    ///
    /// **The boss bit belongs to `enemy::SOLIDITY` and not to `enemy::MORE_FLAGS`**, both being `0x8` of
    /// their own byte: `0x1400396b0` and `0x1400398f0` test it at `+0xbd`, where `+0xbe`'s `0x8` is what
    /// makes a body pass through the player. Written into the wrong one of the two, a boss laid out here
    /// was never the slot `sight::the_boss_the_fights_own_slot` finds — so `State::boss_present` read
    /// clear through a fight nothing else could tell from one, which is what
    /// `th06nc::every_number_a_stage_of_th06nc_is_played_with_reads_back_as_the_game_put_it` caught.
    pub fn a_boss_is_fighting(&self, life: i32, attack_frames: i32) {
        let space = self.space();
        let slot = super::ENEMY_MANAGER + super::enemy_manager::ENEMIES;
        space.write::<u8>(slot + super::enemy::FLAGS, ENEMY_IN_USE);
        space.write::<u8>(
            slot + super::enemy::SOLIDITY,
            ENEMY_SOLID_AND_TESTED | ENEMY_IS_THE_BOSS,
        );
        space.write::<i32>(slot + super::enemy::LIFE, life);
        space.write::<i32>(slot + super::enemy::TIMER, attack_frames);
        self.enemies_alive(1);
    }

    pub fn the_bosses_attack_has_run_for(&self) -> i32 {
        self.space()
            .read::<i32>(super::ENEMY_MANAGER + super::enemy_manager::ENEMIES + super::enemy::TIMER)
    }

    /// **The boss's attack a frame further on**, which the game's own countdown at `0x140038870` steps once an
    /// update — its life and everything else of it left as they are.
    pub fn the_bosses_attack_runs_on(&self) {
        let timer = super::ENEMY_MANAGER + super::enemy_manager::ENEMIES + super::enemy::TIMER;
        self.space()
            .write::<i32>(timer, self.the_bosses_attack_has_run_for() + 1);
    }

    pub fn no_boss_is_fighting(&self) {
        let space = self.space();
        let slot = super::ENEMY_MANAGER + super::enemy_manager::ENEMIES;
        space.write::<u8>(slot + super::enemy::FLAGS, 0);
        space.write::<u8>(slot + super::enemy::SOLIDITY, 0);
        self.enemies_alive(0);
    }

    /// **A bullet flying at a place, with the step it is on and the size it is tested at** — all three,
    /// because all three are what the rules dodge it by: the step is where it will be next frame and the
    /// size is how near is near enough. A bullet written down with neither is one the rules read as
    /// standing still and a point wide, and a laid-out run of this game then died at the very same frame
    /// of the stage on every one of its retries — which
    /// `the_cases_of_th06nc_the_real_game_is_tested_with::a_death_of_th06nc_answered_by_the_chapter_again`
    /// is what caught.
    pub fn a_bullet_of_size_is_at(
        &self,
        slot: usize,
        x: f32,
        y: f32,
        step: (f32, f32),
        size: (f32, f32),
    ) {
        let space = self.space();
        let bullet = super::BULLET_MANAGER
            + super::bullet_manager::BULLETS
            + slot * super::bullet_manager::BYTES;
        space.write::<u16>(bullet + super::bullet_manager::STATE, BULLET_FLYING);
        space.write::<f32>(bullet + super::bullet_manager::POSITION, x);
        space.write::<f32>(
            bullet + super::bullet_manager::POSITION + size_of::<f32>(),
            y,
        );
        space.write::<f32>(bullet + super::bullet_manager::STEP, step.0);
        space.write::<f32>(
            bullet + super::bullet_manager::STEP + size_of::<f32>(),
            step.1,
        );
        space.write::<f32>(bullet + super::bullet_manager::SIZE, size.0);
        space.write::<f32>(
            bullet + super::bullet_manager::SIZE + size_of::<f32>(),
            size.1,
        );
    }

    pub fn the_bullet_is_gone(&self, slot: usize) {
        let bullet = super::BULLET_MANAGER
            + super::bullet_manager::BULLETS
            + slot * super::bullet_manager::BYTES;
        self.space()
            .write::<u16>(bullet + super::bullet_manager::STATE, 0);
    }

    // ---- the card records ----

    /// Every card record laid out as the stage's own build lays them: the magic, the two sizes, the
    /// version and the card's own index, with nought attempts.
    pub fn fills_the_card_records(&self) {
        let space = self.space();
        for card in 0..super::CARDS {
            let record = super::CARD_HISTORY + card * super::CARD_BYTES;
            space.write::<u32>(record, CATK_MAGIC);
            space.write::<u32>(record + size_of::<u32>(), CATK_SIZES);
            space.write::<u8>(record + 2 * size_of::<u32>(), CATK_VERSION);
            space.write::<u16>(
                record + CATK_INDEX,
                u16::try_from(card).expect("one of a hundred and thirty-four cards"),
            );
            space.write::<u16>(record + CATK_ATTEMPTS, 0);
        }
    }

    /// **The file's own card records laid over the ones in memory, as a run's first stage lays them** —
    /// `0x14006e310(the parsed file, manager + 0x968)`, called from `0x14003c5e7` on that build's
    /// run-start branch and nowhere else, which is the read a run that counted an attempt loses it to.
    ///
    /// Of each record of the file that is a `CATK` carrying this build's version, the whole
    /// [`super::CARD_BYTES`] go over the record the word at `+0x10` names — the copy at `0x14006e390` is
    /// three turns of `0x80` bytes off the magic — and the two words at `+0x3c` and `+0x3e` are then held
    /// down to [`MOST_ATTEMPTS`] by `0x14006e3f3` and `0x14006e3fd`.
    ///
    /// **And a record naming a card this build has not got ends the walk rather than being skipped**:
    /// `0x14006e370`'s `jae` on `0x86` jumps to the function's own way out, so the records after it are
    /// not laid down either.
    ///
    /// Two things that read does are not done here, neither being read by orb and neither established:
    /// the dwords across `+0xe0`..`+0x130` it holds down to `0x3b9ac9ff`, and the `0x180` bytes it takes
    /// from a record whose own size says fewer — a record no write of this build's makes.
    pub fn the_files_card_records_are_laid_over(&self, records: &[super::score_file::Record]) {
        let space = self.space();
        for record in records {
            let bytes = &record.bytes;
            if record.magic() != b"CATK"
                || bytes.len() < super::CARD_BYTES
                || bytes[2 * size_of::<u32>()] != CATK_VERSION
            {
                continue;
            }
            let card = usize::from(u16::from_le_bytes([
                bytes[CATK_INDEX],
                bytes[CATK_INDEX + 1],
            ]));
            if card >= super::CARDS {
                return;
            }
            let into = super::CARD_HISTORY + card * super::CARD_BYTES;
            space.write_bytes(into, &bytes[..super::CARD_BYTES]);
            for held in [CATK_ATTEMPTS, CATK_WORD_AFTER_THE_ATTEMPTS] {
                let standing: u16 = space.read(into + held);
                if standing > MOST_ATTEMPTS {
                    space.write::<u16>(into + held, MOST_ATTEMPTS);
                }
            }
        }
    }

    /// **A card's own name in its record, and the sum the game holds it against** — both, because the
    /// count is only stepped where the two agree: a record the layout has left holds `0xc0` words of the
    /// generator's, which is a name nobody declared.
    pub fn the_card_is_named(&self, card: usize, name: &str) {
        let space = self.space();
        let record = super::CARD_HISTORY + card * super::CARD_BYTES;
        let bytes = name.as_bytes();
        assert!(
            bytes.len() < CATK_NAME_BYTES,
            "a name of {} bytes does not fit a record's {CATK_NAME_BYTES} with its terminator",
            bytes.len()
        );
        for (step, byte) in bytes.iter().enumerate() {
            space.write::<u8>(record + CATK_NAME + step, *byte);
        }
        space.write::<u8>(record + CATK_NAME + bytes.len(), 0);
        space.write::<u8>(
            record + CATK_NAME_SUM,
            bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)),
        );
    }

    /// How many times a card has been attempted, in its own record.
    pub fn the_card_has_been_attempted(&self, card: usize, attempts: u16) {
        self.space().write::<u16>(
            super::CARD_HISTORY + card * super::CARD_BYTES + CATK_ATTEMPTS,
            attempts,
        );
    }

    pub fn card_attempts(&self, card: usize) -> u16 {
        self.space()
            .read(super::CARD_HISTORY + card * super::CARD_BYTES + CATK_ATTEMPTS)
    }

    // ---- the heap and the score file ----

    /// The game's `malloc` and `free`, functions of its own exe: what orb calls for a block of the heap the
    /// copy of the score file is put in.
    pub fn hands_over_its_heap(&self, malloc: usize, free: usize) {
        super::set_mallocs(malloc);
        super::set_frees(free);
    }

    /// A block off that heap, given out from the front and never taken back — what a laid-out game's own
    /// `malloc` answers.
    pub fn takes_a_block_off_its_heap(&self, len: usize) -> usize {
        let space = self.space();
        let given: usize = space.read(HEAP_GIVEN);
        let block = given.max(HEAP_FIRST_BLOCK);
        let next = block + len.next_multiple_of(size_of::<usize>());
        if next >= HEAP.end {
            return 0;
        }
        space.write::<usize>(HEAP_GIVEN, next);
        block
    }

    /// The copy of the score file the game's own parse reads, as orb installs it: a block and its length.
    pub fn the_score_file_copy_is(&self, bytes: &[u8]) {
        let space = self.space();
        let block = self.takes_a_block_off_its_heap(bytes.len());
        space.write_bytes(block, bytes);
        space.write::<usize>(super::SCORE_FILE_COPY, block);
        space.write::<i32>(
            super::SCORE_FILE_COPY_LENGTH,
            i32::try_from(bytes.len()).expect("a score file of a few kilobytes"),
        );
    }

    pub fn the_score_file_copy(&self) -> Option<Vec<u8>> {
        let space = self.space();
        let block: usize = space.read(super::SCORE_FILE_COPY);
        if block == 0 {
            return None;
        }
        let len: i32 = space.read(super::SCORE_FILE_COPY_LENGTH);
        let len = usize::try_from(len).ok()?;
        Some(space.read_bytes(block, len))
    }

    // ---- the keys and the word ----

    /// The word the game's own read leaves behind, which a run is played with.
    pub fn the_run_acts_on_the_word(&self, word: u32) {
        self.space()
            .write::<u32>(super::INPUT_THE_RUN_ACTS_ON, word);
    }

    pub fn the_word_the_run_acts_on(&self) -> u32 {
        self.space().read(super::INPUT_THE_RUN_ACTS_ON)
    }

    /// **And the frame before's word**, which is the dword after it: the supervisor's own job copies this
    /// one into it at `0x14007b8ac` before the read, so an edge is a bit set here and not there.
    pub fn the_word_before_is(&self, word: u32) {
        self.space()
            .write::<u32>(super::INPUT_THE_RUN_ACTS_ON + size_of::<u32>(), word);
    }

    pub fn the_word_before(&self) -> u32 {
        self.space()
            .read(super::INPUT_THE_RUN_ACTS_ON + size_of::<u32>())
    }

    pub fn the_keys_the_game_reads(&self) -> Vec<u8> {
        self.space()
            .read_bytes(super::THE_KEYS_THE_GAME_READS, super::KEYS_IN_A_SNAPSHOT)
    }

    pub fn the_game_reads_the_keys(&self, keys: &[u8]) {
        self.space()
            .write_bytes(super::THE_KEYS_THE_GAME_READS, keys);
    }

    /// Where the game's own table keeps a virtual key in the 256 it plays a run off, which is what a test
    /// reads a press back out of.
    pub fn the_index_of_the_key(&self, key: u8) -> Option<usize> {
        let space = self.space();
        for entry in 0..super::KEYS_IN_THE_TABLE {
            let entry = super::THE_TABLE_OF_KEYS + entry * super::A_KEY_IN_THE_TABLE;
            let wanted: u16 = space.read(entry);
            if wanted == NO_MORE_KEYS {
                return None;
            }
            if wanted == u16::from(key) {
                return Some(usize::from(space.read::<u16>(entry + size_of::<u16>())));
            }
        }
        None
    }

    // ---- the music ----

    /// The track playing and where in it, which is what the frame's own catch-up seeks.
    pub fn plays_the_track(&self, track: i32) {
        let space = self.space();
        space.write::<i32>(super::THE_TRACK_PLAYING, track);
        space.write::<u8>(super::THE_TRACK_IS_PLAYING, 1);
    }

    pub fn the_track_playing(&self) -> i32 {
        self.space().read(super::THE_TRACK_PLAYING)
    }

    pub fn the_track_loops(&self, loops: bool) {
        self.space()
            .write::<u8>(super::THE_TRACK_LOOPS, u8::from(loops));
    }

    pub fn the_loop_runs_from_to(&self, from: i64, to: i64) {
        let space = self.space();
        space.write::<i64>(super::THE_LOOP_STARTS, from);
        space.write::<i64>(super::THE_LOOP_ENDS, to);
    }

    /// **How many walks the music has not heard**, which is what the frame's own catch-up turns into a
    /// seek and clears — the count the game's own loop keeps at `THE_EXTRA_WALKS`.
    pub fn the_extra_walks_are(&self, walks: i32) {
        self.space().write::<i32>(super::THE_EXTRA_WALKS, walks);
    }

    pub fn the_extra_walks(&self) -> i32 {
        self.space().read(super::THE_EXTRA_WALKS)
    }

    // ---- the swapchain ----

    /// **A swapchain kept where this game keeps its renderer's objects**, so that the search finds one and
    /// orb has something to draw over — a laid-out game without this is a launch whose overlay never comes
    /// up.
    ///
    /// The five functions are the ones orb calls on a swapchain, and a laid-out game has to answer all
    /// five: the `QueryInterface` that says the object *is* one, the `Release` that gives that reference
    /// straight back, the `GetDesc` orb reads the window out of, and the two presents, whose slots orb
    /// writes over and whose originals it then calls from its own loop.
    ///
    /// Answers where the object is, which is what orb's log names and what an e2e test can hold that
    /// against.
    pub fn keeps_a_swapchain(
        &self,
        query_interface: usize,
        release: usize,
        describes: usize,
        present: usize,
        present1: usize,
        resize_buffers: usize,
    ) -> usize {
        let space = self.space();
        for (slot, function) in [
            (QUERY_INTERFACE, query_interface),
            (RELEASE, release),
            (GET_DESC, describes),
            (RESIZE_BUFFERS, resize_buffers),
            (PRESENT, present),
            (PRESENT1, present1),
        ] {
            space.write::<usize>(SWAPCHAIN_VTABLE + slot * size_of::<usize>(), function);
        }
        space.write::<usize>(SWAPCHAIN.start, SWAPCHAIN_VTABLE);
        // The first of the statics the renderer's objects are kept in, which is where the search begins.
        space.write::<usize>(super::THE_RENDERERS_OBJECTS.start, SWAPCHAIN.start);
        SWAPCHAIN.start
    }

    /// Where that swapchain is, for a laid-out game whose own frame presents through it.
    pub fn the_swapchain(&self) -> usize {
        SWAPCHAIN.start
    }

    /// **The factory the host answers this game's ask for `CreateDXGIFactory2` with**, whose
    /// `CreateSwapChainForHwnd` is `create_swap_chain` — and which orb writes over on the way past, so
    /// that the swapchain made through it is one orb took. See [`FACTORY`], which has why this and not
    /// th06c's search of the statics.
    ///
    /// Answers where the factory is, which is what a laid-out game hands its own `CreateDXGIFactory2`
    /// back.
    pub fn keeps_a_factory(&self, create_swap_chain: usize) -> usize {
        let space = self.space();
        space.write::<usize>(
            FACTORY_VTABLE + CREATE_SWAP_CHAIN_FOR_HWND * size_of::<usize>(),
            create_swap_chain,
        );
        space.write::<usize>(FACTORY.start, FACTORY_VTABLE);
        FACTORY.start
    }

    /// Which slot of a factory's vtable its swapchains are made through, for a laid-out game that has to
    /// call the one orb left there.
    pub fn where_a_factory_makes_a_swapchain(&self) -> usize {
        CREATE_SWAP_CHAIN_FOR_HWND
    }

    /// Where that factory is, for the host call that answers this game's ask with it.
    pub fn the_factory(&self) -> usize {
        FACTORY.start
    }

    // ---- the queues a restore leaves alone ----

    /// **A word inside the block the frame's sound effects are queued through**, which a restore has to
    /// leave where it finds it — see `super::THE_MEDIAS_QUEUES`.
    pub fn the_medias_word_is(&self, word: u32) {
        self.space().write::<u32>(THE_MEDIAS_WORD, word);
    }

    pub fn the_medias_word(&self) -> u32 {
        self.space().read::<u32>(THE_MEDIAS_WORD)
    }

    /// **And a word just below that block**, which a restore therefore puts back: `0x1405fd440`, under the
    /// `0x1405fd454` nothing of this game's reaches below.
    pub fn the_word_below_the_medias_queues_is(&self, word: u32) {
        self.space().write::<u32>(BELOW_THE_MEDIAS_QUEUES, word);
    }

    pub fn the_word_below_the_medias_queues(&self) -> u32 {
        self.space().read::<u32>(BELOW_THE_MEDIAS_QUEUES)
    }
}

/// The first of the media's five queue locks — `0x1405fd520`, initialised at `0x1400c70bd` — whose own
/// first word is what an e2e test writes. Inside [`super::THE_MEDIAS_QUEUES`].
const THE_MEDIAS_WORD: usize = 0x1_405f_d520;
/// And a word below that block, `0x1405fd440`: nothing in the exe references anything between
/// `0x1405fd2a0` and `0x1405fd454`, so this one is the game's own memory and comes back like any other.
const BELOW_THE_MEDIAS_QUEUES: usize = 0x1_405f_d440;
