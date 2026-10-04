//! A th06c process image laid out in an address space, so that the real [`Th06c`](super::Th06c) has
//! something to read in a test with no game running.
//!
//! **Everything a run of this game is made of is a static**, which is what makes this the smallest of the
//! three images to say: the game manager, the supervisor, the player, the enemies, the bullets, the items,
//! the generator, the front end and both chains are all inside the exe's own `.data`, and that one range
//! is what a chapter is a copy of. What the game keeps on its heap is what orb reaches through a pointer or
//! a chain element — the panel's block, the replay manager, the screen a run ends at, the elements the
//! chain is made of — and each of those is a block of this file's own choosing, outside [`DATA`], so that a
//! snapshot of a laid-out game has the same walk to do as one of the real one.
//!
//! The addresses are the game's own, written from the same constants the reads use, so what this cannot
//! catch is a constant that is wrong: a wrong one is wrong on both sides at once, and only the real game
//! says otherwise — every one of them was read against `th06c.exe` running, see `th06c/asm.md`. What it
//! does catch is everything built on top of them, which is where the code is.
//!
//! **The values are this file's own, though, and taking one from [`Th06c`](super::Th06c) is what is refused
//! here** — th07's image has why: a scene number, a screen number or a state shared between the writer and
//! the reader is silent when wrong, and what it costs is a piece of the mode never happening. So each value
//! below carries its own literal and where in the exe it was read.
//!
//! **And nothing here goes through [`super::at`]**: the slide is nought in a test, the exe being laid out at
//! the base its addresses are written against, and `Game::set_image_base` is never told otherwise.

use std::ops::Range;
use std::sync::Arc;

use orb_api::Kind;
use orb_sim::{Sim, Space};

/// The game's static data, as the real one is: **one range**, the `.data` section's own virtual size —
/// `0x140324000` and `0x8a0250` bytes, which is what orb reads out of the PE and writes in the log every
/// run: `.data 0x7ff61b2db000..0x7ff61bb7b250 (9044560 bytes)`. This is the range a chapter is a copy of,
/// and every static below is inside it.
const DATA: Range<usize> = 0x1_4032_4000..0x1_40bc_64d4;

/// Stands for the game's own code, so that a pointer into it reads as a live COM object's vtable and one
/// anywhere else reads as the stale pointer left in a block the allocator did not scrub.
const CODE: Range<usize> = 0x1_4000_1000..0x1_4000_2000;

/// **The page of `.rdata` the game's table from a virtual key to its own key index is on**, which the read
/// orb makes into the game's keys walks — see `super::THE_TABLE_OF_KEYS`. An image page rather than data, as
/// the section is: no snapshot copies it and no restore writes it.
const KEY_TABLE_PAGE: Range<usize> = 0x1_402b_c000..0x1_402b_d000;

/// **The keys this game is played with, as its table maps them**: the virtual key, and the DirectInput scan
/// code the game keeps that key at — read off the table at `0x1402bc0b0`, whose `A` is `0x41 -> 0x1e`.
/// The nine orb presses and no others, since what the table is for here is orb's writes reaching the
/// buffer the game plays off.
const KEYS: [(u16, u16); 9] = [
    (0x5a, 0x2c), // Z
    (0x58, 0x2d), // X
    (0x10, 0x2a), // Shift
    (0x1b, 0x01), // Escape
    (0x0d, 0x1c), // Enter
    (0x26, 0xc8), // Up
    (0x28, 0xd0), // Down
    (0x25, 0xcb), // Left
    (0x27, 0xcd), // Right
];
/// What ends that table, `cmp WORD PTR [rcx],0xffff` at the walk's head.
const NO_MORE_KEYS: u16 = 0xffff;

/// **The blocks the game takes off its heap that orb reaches**, each a base of this file's choosing and the
/// length the game's own where one is known.
///
/// The panel's block, `0x140955358` pointing at it: long enough to reach the line of dialogue at `+0x23a0`
/// and the field after it.
const GUI_IMPL: Range<usize> = 0x1_5000_0000..0x1_5000_3000;
/// The replay manager, `0x78` bytes at `0x14004d193`, `0x140955380` pointing at it.
const REPLAY_MANAGER: Range<usize> = 0x1_5000_3000..0x1_5000_4000;
/// The screen a run ends at, which is reached through the chain element it registers and through nothing
/// else — as long as the field furthest into it, the name entry's count at `+0x517c`.
const RESULT_SCREEN: Range<usize> = 0x1_5001_0000..0x1_5001_6000;
/// The chain elements: `0x40` bytes apiece off the heap, `0x14004fd6d` allocating the result screen's.
/// One block of them, a slot each — see [`Element`].
const ELEMENTS: Range<usize> = 0x1_5002_0000..0x1_5002_1000;
const ELEMENT_BYTES: usize = 0x40;
/// **Where the game's `malloc` and `free` hand blocks out of** — see `super::MALLOC`: one megabyte, given out
/// from the front and never taken back, the word at its start saying how far it has been given. A copy of the
/// score file is a few kilobytes and a launch installs one per settling of the mode, so nothing a test does
/// reaches the end.
const HEAP: Range<usize> = 0x1_5003_0000..0x1_5013_0000;
const HEAP_GIVEN: usize = HEAP.start;
const HEAP_FIRST_BLOCK: usize = HEAP.start + 0x10;

/// **The swapchain this game presents through**, a block of its own outside [`DATA`] as the real one is
/// DXGI's rather than the exe's.
const SWAPCHAIN: Range<usize> = 0x1_5014_0000..0x1_5014_1000;

/// **The renderer**, a block off the heap as the real one is, as long as `super::THE_RENDERERS_BLOCK` — the
/// laid-out game keeps nothing of it but the batch its draws queue their sprites into.
const RENDERER: Range<usize> = 0x1_5015_0000..0x1_5015_0000 + super::THE_RENDERERS_BLOCK;
/// **And its vtable, which has to be inside [`CODE`]**: `super::look_for_the_swapchain` will not call
/// through a pointer whose vtable is anywhere but a mapped image — see `orb_api::mem::vtable_in_image`,
/// which is what tells an object from a field that merely reads as one. Far enough into the page that the
/// twenty-three slots `IDXGISwapChain1` has reach no further than its end.
const SWAPCHAIN_VTABLE: usize = CODE.start + 0x800;

/// **Which slot of that vtable each of the four is**, counted in pointers, with their own literals here as
/// every number in this file has: `crate::dxgi` reads the same four, and one taken from there would be
/// wrong on both sides at once — what it costs is orb reading a swapchain it laid out itself. `IUnknown`'s
/// three come first, then `IDXGIObject`'s four and `IDXGIDeviceSubObject`'s one before `Present`; `Present1`
/// comes after `IDXGISwapChain`'s rest and `IDXGISwapChain1`'s own four.
const QUERY_INTERFACE: usize = 0;
const RELEASE: usize = 2;
const PRESENT: usize = 8;
/// `GetDesc`, after `IDXGISwapChain`'s own `GetBuffer`, `SetFullscreenState` and `GetFullscreenState` —
/// which is how orb reads the window this game plays in, see `crate::dxgi::the_window`.
const GET_DESC: usize = 12;
const PRESENT1: usize = 22;

/// Which slot of [`ELEMENTS`] each job the chain carries has.
#[derive(Clone, Copy)]
enum Element {
    Supervisor = 0,
    FrontEnd = 1,
    GameManager = 2,
    ResultScreen = 3,
    Ending = 4,
    Draw = 5,
}

impl Element {
    const fn at(self) -> usize {
        ELEMENTS.start + self as usize * ELEMENT_BYTES
    }
}

/// The element's own priority, its first word: `mov WORD PTR [r14],0xf` at `0x14004d2a5` writes the
/// recording's, and the walk keeps the list in its order.
const ELEMENT_PRIORITY: usize = 0x0;

/// **The calc chain's jobs, by the callback each element carries** — the addresses of the game's own
/// callbacks, because that is what orb matches on: `what_a_job_runs_on` finds the result screen by
/// `0x14004ff40` and `Jobs::hold` the ending by `0x140018110`. A game laid out by hand dispatches on
/// these to functions of its own — see `fake::th06c::Fake::job`.
///
/// Read off a launch's own `chain:` lines: `2 job(s): 0x140058330 0x1400077f0` before anything is up,
/// `0x14002ccf0` arriving with the front end and the ten of a stage after it.
pub mod chain_job {
    pub const SUPERVISOR: usize = 0x1_4005_8330;
    pub const FRONT_END: usize = 0x1_4002_ccf0;
    /// The game manager's own update, the first of a stage's ten.
    pub const GAME_MANAGER: usize = 0x1_4002_3950;
    pub const RESULT_SCREEN: usize = super::super::RESULT_SCREEN_UPDATE;
    pub const ENDING: usize = super::super::ENDING_UPDATE;
}

/// And their priorities, in the order a walk reaches them: the supervisor first at nought, the front end
/// at two, a stage's manager at four and the result screen at thirteen — th06 1.02h's own order, which the
/// `chain:` lines above show kept.
const CALC_SUPERVISOR: u16 = 0;
const CALC_FRONT_END: u16 = 2;
const CALC_ENDING: u16 = 3;
const CALC_GAME_MANAGER: u16 = 4;
const CALC_RESULT_SCREEN: u16 = 13;

/// What a callback of either chain answers, read off the draw walk at `0x140025a4c`: carry on, run it
/// again, take the element out, or end the walk.
pub mod chain_result {
    pub const CONTINUES: i32 = 1;
    pub const AGAIN: i32 = 2;
    pub const REMOVED: i32 = 0;
    pub const OVER: i32 = 3;
}

/// **The supervisor's states**, th06 1.02h's numbering: `0x14002cc40` writes the run's start as two,
/// `0x140050d79` puts one back as the result screen leaves, `0x140018f05` writes seven at an ending's end,
/// `0x1400362d7` writes six for the ranking, and `0x1400242f9` reads three as the stage callback begins.
const SCENE_FRONT_END: i32 = 1;
const SCENE_RUN: i32 = 2;
const SCENE_REBUILDING: i32 = 3;
const SCENE_RANKING: i32 = 6;
const SCENE_RUN_OVER: i32 = 7;
const SCENE_ENDING: i32 = 10;

/// **The front end's screens**, by the number its state holds: read off a launch's own `front end:` lines
/// — `0, 1, 2, 6, 7, 9, 11` walking a fresh launch into a run — and the Score item's write of ten at
/// `0x14002e9bd`.
pub const SCREEN_LOGO: i32 = 1;
pub const SCREEN_TITLE: i32 = 2;
pub const SCREEN_ON_ITS_OWN: i32 = 6;
pub const SCREEN_DIFFICULTY: i32 = 7;
pub const SCREEN_CHARACTER: i32 = 9;
pub const SCREEN_SHOT_SELECT: i32 = 11;
pub const SCREEN_RANKING: i32 = 10;

/// **The result screen's states**, off the jump table at `0x1400535c8`: nine is the name entry a played run
/// opens in (`0x14004fde9`); fifteen the stats the name's decide opens, which takes a decide of its own once
/// up ninety frames (`0x14005352f`); sixteen the thirty frames it fades for (`0x1400534fa`); ten the question
/// about saving a replay as it comes up, which is where orb's own stands in front and which takes a press
/// once up eighty frames (`0x1400532ed`); fourteen that question with its cursor live; and seventeen the way
/// out. The question comes up on いいえ — `0x140053374` takes a cursor of nought for the save and the cursor
/// is not nought — so its decide leaves, through state 2 (`0x1400533b7`) and the exit: a launch of the real
/// one tapping decide from the name entry had the title back 200 frames on.
pub const RESULT_TYPING_A_NAME: i32 = 9;
/// And one, the state the ranking the title's Score item asks for stands in — a launch's own
/// `score: the ranking is up — ... state=1` line — whose one decide is the way out.
pub const RESULT_RANKING: i32 = 1;
pub const RESULT_STATS: i32 = 15;
pub const RESULT_STATS_FADING: i32 = 16;
pub const RESULT_SAVE_REPLAY_QUESTION: i32 = 10;
pub const RESULT_SAVE_REPLAY_ASKED: i32 = 14;
pub const RESULT_LEAVING: i32 = 17;
/// What each of those waits: the stats before their decide, the fade, and the question before its cursor is
/// live — the last this laid-out game's own number, the real one's fades being uncounted.
pub const RESULT_STATS_TAKE_A_PRESS_AFTER: i32 = 0x5a;
pub const RESULT_STATS_FADE_FOR: i32 = 0x1e;
pub const RESULT_QUESTION_COMES_UP_AFTER: i32 = 30;

/// The stage the game counts from: one, `0x140027255` multiplying it by a thousand into `Stage * 1000`.
const STAGE_COUNTS_FROM: i32 = 1;

/// The player's four states — nought playing, one a respawn, two the hit, three invulnerable — off
/// `0x14004c78e`, which gates the hit on nought, and `0x14004c7f4`, which writes two.
const PLAYER_PLAYING: u8 = 0;
const PLAYER_COMING_BACK: u8 = 1;
const PLAYER_HIT: u8 = 2;
const PLAYER_INVULNERABLE: u8 = 3;

/// An enemy that is there and is the boss: the top bit of its flags byte (`0x140020850`) and the boss bit
/// of the byte after it (`0x140021222`), with the two solidity bits the hit test wants up.
const ENEMY_IN_USE: u8 = 0x80;
const ENEMY_SOLID_AND_TESTED: u8 = 0x07;
const ENEMY_IS_THE_BOSS: u8 = 0x08;

/// A bullet's one hitting state, `0x14000dcd2`'s first case.
const BULLET_FLYING: u16 = 1;

/// `'CATK'`, which every record carries from the stage callback's fill at `0x1400249cb`.
const CATK_MAGIC: u32 = u32::from_le_bytes(*b"CATK");
/// The record's own size words and version, `0x1400249d1` and `0x1400249d8`.
const CATK_SIZES: u32 = 0x0040_0040;
const CATK_VERSION: u8 = 0x10;

/// The two statics a laid-out game hands orb's hooks as their objects: the supervisor, which orb's frame
/// is called on, and the game manager, which the stage callback is entered with.
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
        space.map(GUI_IMPL.start, GUI_IMPL.len(), Kind::Private);
        space.map(REPLAY_MANAGER.start, REPLAY_MANAGER.len(), Kind::Private);
        space.map(RESULT_SCREEN.start, RESULT_SCREEN.len(), Kind::Private);
        space.map(ELEMENTS.start, ELEMENTS.len(), Kind::Private);
        space.map(HEAP.start, HEAP.len(), Kind::Private);
        space.map(SWAPCHAIN.start, SWAPCHAIN.len(), Kind::Private);
        space.map(RENDERER.start, RENDERER.len(), Kind::Private);
        let image = Self { sim };
        image.space().write::<usize>(HEAP_GIVEN, HEAP_FIRST_BLOCK);
        // The renderer, made at startup as the real one's is, before anything is drawn.
        image
            .space()
            .write::<usize>(super::THE_RENDERER, RENDERER.start);
        // The table the game's own read maps a virtual key through, which is what orb's writes into the
        // game's keys go through too.
        let mut entry = super::THE_TABLE_OF_KEYS;
        for (key, index) in KEYS {
            image.space().write::<u16>(entry, key);
            image.space().write::<u16>(entry + size_of::<u16>(), index);
            entry += super::A_KEY_IN_THE_TABLE;
        }
        image.space().write::<u16>(entry, NO_MORE_KEYS);
        // The panel's block, reached through the pointer, with no line of dialogue up in it: a zeroed
        // block reads as line nought, which is a dialogue, and a dialogue is a frame no chapter may begin
        // on.
        image
            .space()
            .write::<usize>(super::GUI_IMPL, GUI_IMPL.start);
        image
            .space()
            .write::<i32>(GUI_IMPL.start + super::gui_impl::MSG_IDX, -1);
        // The field's own size and the box the player is held to, as `.data` initialises the first and
        // the game fills the second: 384 by 448 at the origin.
        image.space().write::<f32>(super::FIELD_SIZE, 384.0);
        image
            .space()
            .write::<f32>(super::FIELD_SIZE + size_of::<f32>(), 448.0);
        image.the_play_field_is(0.0, 0.0, 384.0, 448.0);
        // No track playing, which is what the run's own start writes: `-1` at `0x14002ccbb`.
        image.space().write::<i32>(super::THE_TRACK_PLAYING, -1);
        image
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is
    /// what makes `Th06c`'s reads land in it.
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

    /// **Hands over every call orb makes into this game's code**, in place of the addresses `Th06c` holds:
    /// the calc walk, the take-out of an element, the frame's view, the batch of sprites flushed, the four
    /// of the sound's catch-up, and the two that finish a picture. All ten together, because orb's frame
    /// makes them all: a game that handed over fewer would be jumping into memory nothing has mapped part way
    /// through one frame.
    #[allow(clippy::too_many_arguments)]
    pub fn hands_over_the_frames_own_calls(
        &self,
        run_calc_chain: usize,
        takes_an_element_out: usize,
        sets_the_frames_view: usize,
        flushes_the_sprites: usize,
        the_sounds_rate: usize,
        where_the_music_is: usize,
        how_long_the_track_is: usize,
        seeks_the_music: usize,
        puts_the_rate_together: usize,
        finishes_the_picture: usize,
    ) {
        super::set_run_calc_chain(run_calc_chain);
        super::set_takes_an_element_out(takes_an_element_out);
        super::set_sets_the_frames_view(sets_the_frames_view);
        super::set_flushes_the_sprites(flushes_the_sprites);
        super::set_the_sounds_rate(the_sounds_rate);
        super::set_where_the_music_is(where_the_music_is);
        super::set_how_long_the_track_is(how_long_the_track_is);
        super::set_seeks_the_music(seeks_the_music);
        super::set_puts_the_rate_together(puts_the_rate_together);
        super::set_finishes_the_picture(finishes_the_picture);
    }

    /// And the game's own run start, `0x14002cc00`, which orb calls where it writes a run into being — see
    /// `Th06c::start_a_run`.
    pub fn hands_over_the_run_start(&self, run_start: usize) {
        super::set_run_start(run_start);
    }

    // ---- the supervisor ----

    /// The supervisor's two states written: the one it is in and the one its update last acted on.
    pub fn is_in_scene(&self, scene: i32, last: i32) {
        let space = self.space();
        space.write::<i32>(super::SUPERVISOR + super::supervisor::CUR_STATE, scene);
        space.write::<i32>(super::SUPERVISOR + super::supervisor::LAST_STATE, last);
    }

    pub fn the_scene_now(&self) -> i32 {
        self.space()
            .read::<i32>(super::SUPERVISOR + super::supervisor::CUR_STATE)
    }

    pub fn the_scene_last_acted_on(&self) -> i32 {
        self.space()
            .read::<i32>(super::SUPERVISOR + super::supervisor::LAST_STATE)
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

    pub fn scene_of_the_ranking(&self) -> i32 {
        SCENE_RANKING
    }

    pub fn scene_of_a_run_over(&self) -> i32 {
        SCENE_RUN_OVER
    }

    pub fn scene_of_an_ending(&self) -> i32 {
        SCENE_ENDING
    }

    /// The lives and the bombs a run starts with, out of the configuration the supervisor keeps: the
    /// game's own defaults, `02 03` at `0x140059935`.
    pub fn the_configuration_gives(&self, lives: u8, bombs: u8) {
        let space = self.space();
        space.write::<u8>(super::SUPERVISOR + super::supervisor::CONFIG_LIVES, lives);
        space.write::<u8>(super::SUPERVISOR + super::supervisor::CONFIG_BOMBS, bombs);
    }

    // ---- the chains ----

    /// `Chain::AddToCalc`: the element linked into the calc list by its priority, behind every element of
    /// a lower or equal one, with the callback and the object it is called on.
    pub fn registers_in_the_calc_chain(
        &self,
        elem: usize,
        priority: u16,
        callback: usize,
        on: usize,
    ) {
        self.registers_in(super::THE_CALC_CHAIN, elem, priority, callback, on);
    }

    /// And the draw list, whose one job here is whatever a laid-out game draws with.
    pub fn registers_in_the_draw_chain(&self, callback: usize, on: usize) {
        self.registers_in(super::THE_DRAW_CHAIN, Element::Draw.at(), 0, callback, on);
    }

    /// **A swapchain kept where this game keeps its objects**, so that
    /// [`look_for_the_swapchain`](super::look_for_the_swapchain) finds one and orb has something to draw
    /// over — a laid-out game without this is a launch whose overlay never comes up, which is what th06c's
    /// e2e tests were before.
    ///
    /// The five functions are the ones orb calls on a swapchain, and a laid-out game has to answer all
    /// five: the `QueryInterface` that says the object *is* one (`IID_IDXGISwapChain`, a reference added),
    /// the `Release` that gives that reference straight back, the `GetDesc` orb reads the window out of,
    /// and the two presents, whose slots orb writes over and whose originals it then calls from its own
    /// loop. Their signatures are DXGI's own — `(object, iid, out) -> i32`, `(object) -> u32`,
    /// `(swapchain, desc) -> i32`, `(swapchain, interval, flags) -> i32` and that with a parameters
    /// pointer after it.
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
    ) -> usize {
        let space = self.space();
        for (slot, function) in [
            (QUERY_INTERFACE, query_interface),
            (RELEASE, release),
            (GET_DESC, describes),
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

    /// Where that swapchain is, for a laid-out game whose own frame presents through it — see
    /// `fake::th06c::finishes_the_picture`, which is the call the real one's composite makes.
    pub fn the_swapchain(&self) -> usize {
        SWAPCHAIN.start
    }

    fn registers_in(&self, head: usize, elem: usize, priority: u16, callback: usize, on: usize) {
        let space = self.space();
        space.write::<u16>(elem + ELEMENT_PRIORITY, priority);
        space.write::<usize>(elem + super::chain::CALLBACK, callback);
        space.write::<usize>(elem + super::chain::ON, on);
        let mut link = head + super::chain::NEXT;
        loop {
            let next: usize = space.read(link);
            if next == 0 || space.read::<u16>(next + ELEMENT_PRIORITY) > priority {
                space.write::<usize>(elem + super::chain::NEXT, next);
                space.write::<usize>(link, elem);
                return;
            }
            link = next + super::chain::NEXT;
        }
    }

    /// `0x14000f620`: the element unlinked from whichever chain holds it, and its links cleared.
    pub fn cuts_from_the_chain(&self, elem: usize) {
        for head in [super::THE_CALC_CHAIN, super::THE_DRAW_CHAIN] {
            let space = self.space();
            let mut link = head + super::chain::NEXT;
            loop {
                let next: usize = space.read(link);
                if next == 0 {
                    break;
                }
                if next == elem {
                    space.write::<usize>(link, space.read::<usize>(elem + super::chain::NEXT));
                    space.write::<usize>(elem + super::chain::NEXT, 0);
                    space.write::<usize>(elem + super::chain::CALLBACK, 0);
                    return;
                }
                link = next + super::chain::NEXT;
            }
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

    /// The front end's own job at its priority, and the cut on its way down.
    pub fn registers_the_front_end(&self) {
        self.registers_in_the_calc_chain(
            Element::FrontEnd.at(),
            CALC_FRONT_END,
            chain_job::FRONT_END,
            super::FRONT_END,
        );
    }

    pub fn cuts_the_front_end(&self) {
        self.cuts_from_the_chain(Element::FrontEnd.at());
    }

    /// The game manager's own job, which is the first of a stage's ten and stands for all of them here.
    pub fn registers_the_gameplay_scene(&self) {
        self.registers_in_the_calc_chain(
            Element::GameManager.at(),
            CALC_GAME_MANAGER,
            chain_job::GAME_MANAGER,
            super::GAME_MANAGER,
        );
    }

    pub fn cuts_the_gameplay_scene(&self) {
        self.cuts_from_the_chain(Element::GameManager.at());
    }

    /// **The screen a run ends at registered**, in the state it is built in, which is the one way to that
    /// screen: the element carries its update at `+0x8` and the screen itself at `+0x38` (`0x14004fdcd`).
    pub fn the_screen_a_run_ends_at_is_up(&self, state: i32) {
        self.registers_in_the_calc_chain(
            Element::ResultScreen.at(),
            CALC_RESULT_SCREEN,
            chain_job::RESULT_SCREEN,
            RESULT_SCREEN.start,
        );
        self.the_screen_a_run_ends_at_is_in(state);
    }

    pub fn the_screen_a_run_ends_at_is_in(&self, state: i32) {
        self.space()
            .write::<i32>(RESULT_SCREEN.start + super::result_screen::STATE, state);
    }

    pub fn the_state_the_screen_a_run_ends_at_is_in(&self) -> i32 {
        self.space()
            .read::<i32>(RESULT_SCREEN.start + super::result_screen::STATE)
    }

    pub fn the_screen_a_run_ends_at_is_down(&self) {
        self.cuts_from_the_chain(Element::ResultScreen.at());
    }

    pub fn the_screen_a_run_ends_at(&self) -> usize {
        RESULT_SCREEN.start
    }

    /// **The name the entry on that screen holds**, eight bytes with spaces to the end, and the count typed
    /// and the grid's cursor as the screen's own keys leave them — see `super::result_screen`.
    pub fn the_entry_holds(&self, name: &str) {
        let mut bytes = [b' '; super::result_screen::NAME_BYTES];
        for (into, byte) in bytes.iter_mut().zip(name.bytes()) {
            *into = byte;
        }
        self.space()
            .write_bytes(RESULT_SCREEN.start + super::result_screen::NAME, &bytes);
    }

    pub fn the_name_the_entry_holds(&self) -> String {
        let bytes = self.space().read_bytes(
            RESULT_SCREEN.start + super::result_screen::NAME,
            super::result_screen::NAME_BYTES,
        );
        String::from_utf8_lossy(&bytes).trim_end().to_owned()
    }

    pub fn the_characters_typed(&self) -> i32 {
        self.space()
            .read::<i32>(RESULT_SCREEN.start + super::result_screen::TYPED)
    }

    pub fn the_entrys_cursor(&self) -> i32 {
        self.space()
            .read::<i32>(RESULT_SCREEN.start + super::result_screen::CURSOR)
    }

    /// **An ending running**, which is its job being in the chain — see `super::ENDING_UPDATE` — and
    /// nothing else: the supervisor's state stays a stage's through one.
    pub fn an_ending_is_running(&self) {
        self.registers_in_the_calc_chain(
            Element::Ending.at(),
            CALC_ENDING,
            chain_job::ENDING,
            Element::Ending.at(),
        );
    }

    pub fn the_ending_is_over(&self) {
        self.cuts_from_the_chain(Element::Ending.at());
    }

    // ---- the front end ----

    /// The front end on one of its screens, with its cursor on `cursor` and up for no frames yet.
    pub fn the_front_end_is_on(&self, screen: i32, cursor: i32) {
        let space = self.space();
        space.write::<i32>(super::FRONT_END + super::front_end::STATE, screen);
        space.write::<i32>(super::FRONT_END + super::front_end::CURSOR, cursor);
        space.write::<i32>(super::FRONT_END + super::front_end::SCREEN_FRAMES, 0);
    }

    pub fn the_screen_the_front_end_is_on(&self) -> i32 {
        self.space()
            .read::<i32>(super::FRONT_END + super::front_end::STATE)
    }

    pub fn the_item_the_cursor_is_on(&self) -> i32 {
        self.space()
            .read::<i32>(super::FRONT_END + super::front_end::CURSOR)
    }

    pub fn the_cursor_is_on(&self, item: i32) {
        self.space()
            .write::<i32>(super::FRONT_END + super::front_end::CURSOR, item);
    }

    pub fn frames_the_screen_has_been_up(&self) -> i32 {
        self.space()
            .read::<i32>(super::FRONT_END + super::front_end::SCREEN_FRAMES)
    }

    pub fn the_screen_has_been_up_for(&self, frames: i32) {
        self.space()
            .write::<i32>(super::FRONT_END + super::front_end::SCREEN_FRAMES, frames);
    }

    // ---- the run's own numbers ----

    /// A stage of a run playing: the scene, the stage as the game counts it, and the difficulty.
    pub fn is_playing(&self, stage: i32, difficulty: i32) {
        self.is_in_scene(SCENE_RUN, SCENE_RUN);
        let space = self.space();
        space.write::<i32>(super::STAGE, stage + STAGE_COUNTS_FROM);
        space.write::<i32>(super::DIFFICULTY, difficulty);
    }

    /// The game's own stage word, written as the game writes it and not as orb counts it.
    pub fn the_games_stage_word_is(&self, word: i32) {
        self.space().write::<i32>(super::STAGE, word);
    }

    pub fn the_games_stage_word(&self) -> i32 {
        self.space().read::<i32>(super::STAGE)
    }

    /// What the two selects and the shot select write: the difficulty, the character and the shot, with
    /// the run no playback. The practice flag is the title item's and stays as it left it.
    pub fn the_shot_select_decided(&self, difficulty: i32, character: u8, shot: u8) {
        let space = self.space();
        space.write::<i32>(super::DIFFICULTY, difficulty);
        space.write::<u8>(
            super::GAME_MANAGER + super::game_manager::CHARACTER,
            character,
        );
        space.write::<u8>(super::GAME_MANAGER + super::game_manager::SHOT, shot);
        space.write::<u8>(super::A_REPLAY_IS_PLAYING, 0);
        space.write::<u8>(super::THE_PLAYBACK_IS_THE_DEMO, 0);
    }

    pub fn is_not_practising(&self) {
        self.space()
            .write::<u8>(super::GAME_MANAGER + super::game_manager::IS_IN_PRACTICE, 0);
    }

    pub fn the_difficulty_is(&self, difficulty: i32) {
        self.space().write::<i32>(super::DIFFICULTY, difficulty);
    }

    pub fn the_difficulty_now(&self) -> i32 {
        self.space().read::<i32>(super::DIFFICULTY)
    }

    pub fn the_character_chosen(&self) -> u8 {
        self.space()
            .read::<u8>(super::GAME_MANAGER + super::game_manager::CHARACTER)
    }

    pub fn the_shot_chosen(&self) -> u8 {
        self.space()
            .read::<u8>(super::GAME_MANAGER + super::game_manager::SHOT)
    }

    /// The one frame a run has been chosen and nothing of it built: the supervisor already in the run's
    /// state while the state it last acted on is still the front end's.
    pub fn a_run_has_been_chosen(&self) {
        self.is_in_scene(SCENE_RUN, SCENE_FRONT_END);
    }

    pub fn is_practising(&self) {
        self.space()
            .write::<u8>(super::GAME_MANAGER + super::game_manager::IS_IN_PRACTICE, 1);
    }

    pub fn has_lives_and_bombs(&self, lives: i8, bombs: i8) {
        let space = self.space();
        space.write::<i8>(super::LIVES, lives);
        space.write::<i8>(super::BOMBS, bombs);
    }

    pub fn lives_and_bombs(&self) -> (i8, i8) {
        (
            self.space().read::<i8>(super::LIVES),
            self.space().read::<i8>(super::BOMBS),
        )
    }

    pub fn shot_is_at_power(&self, power: u16) {
        self.space().write::<u16>(super::POWER, power);
    }

    /// The score into both of its fields, the panel's and the run's, and the point items beside them.
    pub fn the_run_has_scored(&self, score: u32, point_items: u16) {
        let space = self.space();
        space.write::<u32>(super::GAME_MANAGER + super::game_manager::GUI_SCORE, score);
        space.write::<u32>(super::GAME_MANAGER + super::game_manager::SCORE, score);
        space.write::<u16>(
            super::GAME_MANAGER + super::game_manager::POINT_ITEMS,
            point_items,
        );
    }

    pub fn the_score(&self) -> u32 {
        self.space()
            .read::<u32>(super::GAME_MANAGER + super::game_manager::SCORE)
    }

    /// The panel's own copy of the score alone, which a stage transition puts back to the run's —
    /// `0x140024cf5`'s first instruction, and the reason this is a setter of its own.
    pub fn the_panel_shows(&self, score: u32) {
        self.space()
            .write::<u32>(super::GAME_MANAGER + super::game_manager::GUI_SCORE, score);
    }

    /// The run's own field alone, which the stage callback leaves at nought and the stage's first update raises
    /// back to the panel's — `0x14002548d` and `0x140023f71`.
    pub fn the_runs_own_score_is(&self, score: u32) {
        self.space()
            .write::<u32>(super::GAME_MANAGER + super::game_manager::SCORE, score);
    }

    pub fn the_score_the_panel_shows(&self) -> u32 {
        self.space()
            .read::<u32>(super::GAME_MANAGER + super::game_manager::GUI_SCORE)
    }

    pub fn the_point_items(&self) -> u16 {
        self.space()
            .read::<u16>(super::GAME_MANAGER + super::game_manager::POINT_ITEMS)
    }

    pub fn the_shots_power(&self) -> u16 {
        self.space().read::<u16>(super::POWER)
    }

    pub fn the_power_items(&self) -> i8 {
        self.space()
            .read::<i8>(super::GAME_MANAGER + super::game_manager::POWER_ITEMS)
    }

    pub fn the_extra_lives_the_score_paid_for(&self) -> i8 {
        self.space()
            .read::<i8>(super::GAME_MANAGER + super::game_manager::EXTRA_LIVES)
    }

    pub fn the_rank_is(&self, rank: i32, sub_rank: i32) {
        let space = self.space();
        space.write::<i32>(super::GAME_MANAGER + super::game_manager::RANK, rank);
        space.write::<i32>(
            super::GAME_MANAGER + super::game_manager::SUB_RANK,
            sub_rank,
        );
    }

    pub fn the_rank(&self) -> i32 {
        self.space()
            .read::<i32>(super::GAME_MANAGER + super::game_manager::RANK)
    }

    /// The counter that climbs towards the next step of the rank, which a stage transition puts back to
    /// nothing — `0x140024d10`.
    pub fn the_sub_rank(&self) -> i32 {
        self.space()
            .read::<i32>(super::GAME_MANAGER + super::game_manager::SUB_RANK)
    }

    pub fn the_score_has_paid_for(&self, extra_lives: i8) {
        self.space().write::<i8>(
            super::GAME_MANAGER + super::game_manager::EXTRA_LIVES,
            extra_lives,
        );
    }

    pub fn the_power_items_towards_the_score(&self, power_items: i8) {
        self.space().write::<i8>(
            super::GAME_MANAGER + super::game_manager::POWER_ITEMS,
            power_items,
        );
    }

    pub fn the_retries_counted(&self) -> u8 {
        self.space()
            .read::<u8>(super::GAME_MANAGER + super::game_manager::NUM_RETRIES)
    }

    /// The seed the stage started from, as the stage callback copies it out of the generator.
    pub fn the_stage_began_with_the_seed(&self, seed: u16) {
        self.space()
            .write::<u16>(super::GAME_MANAGER + super::game_manager::RANDOM_SEED, seed);
    }

    pub fn the_seed_the_stage_began_with(&self) -> u16 {
        self.space()
            .read::<u16>(super::GAME_MANAGER + super::game_manager::RANDOM_SEED)
    }

    /// The generator itself, and how many draws have been made of it.
    pub fn the_generator_is(&self, seed: u16, randoms: u32) {
        let space = self.space();
        space.write::<u16>(super::RNG_SEED, seed);
        space.write::<u32>(super::RNG_COUNT, randoms);
    }

    pub fn the_generator(&self) -> (u16, u32) {
        (
            self.space().read::<u16>(super::RNG_SEED),
            self.space().read::<u32>(super::RNG_COUNT),
        )
    }

    /// One draw of the generator, as every one in the exe makes it — `0x1400243f6` onwards — and the
    /// count stepped beside it.
    pub fn draws_from_the_generator(&self) -> u16 {
        let (seed, randoms) = self.the_generator();
        let mut word = seed ^ 0x9630;
        word = word.wrapping_sub(0x6553);
        let drawn = (word << 2).wrapping_add(word >> 14);
        self.the_generator_is(drawn, randoms + 1);
        drawn
    }

    pub fn has_been_in_the_stage_for(&self, frames: u32) {
        self.space().write::<u32>(
            super::GAME_MANAGER + super::game_manager::GAME_FRAMES,
            frames,
        );
    }

    pub fn frames_in_the_stage(&self) -> u32 {
        self.space()
            .read::<u32>(super::GAME_MANAGER + super::game_manager::GAME_FRAMES)
    }

    pub fn the_script_has_run_for(&self, frames: i32) {
        self.space().write::<i32>(
            super::ENEMY_MANAGER + super::enemy_manager::TIMELINE_TIME,
            frames,
        );
    }

    /// The pause menu and the continue menu, which are the two flags a run stops on.
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

    pub fn is_it_paused(&self) -> bool {
        self.space()
            .read::<u8>(super::GAME_MANAGER + super::game_manager::IS_IN_GAME_MENU)
            != 0
    }

    /// The byte that holds everything, which a death's own animation sets.
    pub fn everything_is_held(&self, held: bool) {
        self.space()
            .write::<u8>(super::EVERYTHING_HELD, u8::from(held));
    }

    // ---- the playback ----

    pub fn a_replay_is_playing(&self, demo: bool) {
        let space = self.space();
        space.write::<u8>(super::A_REPLAY_IS_PLAYING, 1);
        space.write::<u8>(super::THE_PLAYBACK_IS_THE_DEMO, u8::from(demo));
    }

    pub fn no_replay_is_playing(&self) {
        let space = self.space();
        space.write::<u8>(super::A_REPLAY_IS_PLAYING, 0);
        space.write::<u8>(super::THE_PLAYBACK_IS_THE_DEMO, 0);
    }

    /// The replay manager a run makes as it starts, and its count of the stage's frames — the field a
    /// reproduction line reads a stage's position out of.
    pub fn the_replay_manager_is_up(&self) {
        self.space()
            .write::<usize>(super::REPLAY_MANAGER, REPLAY_MANAGER.start);
        self.set_replay_clock(0);
    }

    pub fn the_replay_manager_is_gone(&self) {
        self.space().write::<usize>(super::REPLAY_MANAGER, 0);
    }

    pub fn set_replay_clock(&self, frame: i32) {
        self.space()
            .write::<i32>(REPLAY_MANAGER.start + super::replay_manager::FRAME, frame);
    }

    // ---- the player ----

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
            space.read::<f32>(super::PLAYER + super::player::POSITION),
            space.read::<f32>(super::PLAYER + super::player::POSITION + size_of::<f32>()),
        )
    }

    /// The field the player is held to, four floats: left, top, width, height.
    pub fn the_play_field_is(&self, left: f32, top: f32, width: f32, height: f32) {
        let space = self.space();
        for (word, value) in [left, top, width, height].into_iter().enumerate() {
            space.write::<f32>(super::PLAYER_AREA + word * size_of::<f32>(), value);
        }
    }

    /// The player's own box as the hit test is handed it: the two corners, `half` either side of where
    /// the player is.
    pub fn the_players_box_is(&self, half: (f32, f32)) {
        let (x, y) = self.where_the_player_is();
        let space = self.space();
        space.write::<f32>(super::PLAYER + super::player::BOX_MIN, x - half.0);
        space.write::<f32>(
            super::PLAYER + super::player::BOX_MIN + size_of::<f32>(),
            y - half.1,
        );
        space.write::<f32>(super::PLAYER + super::player::BOX_MAX, x + half.0);
        space.write::<f32>(
            super::PLAYER + super::player::BOX_MAX + size_of::<f32>(),
            y + half.1,
        );
    }

    /// What a frame of a direction moves the player by, straight and diagonally, and the scale the two axes
    /// take before it reaches the position: one, the numbers being this file's own.
    pub fn the_player_steps_by(&self, straight: f32, diagonal: f32) {
        let space = self.space();
        space.write::<f32>(super::PLAYER + super::player::SPEED_STRAIGHT, straight);
        space.write::<f32>(
            super::PLAYER + super::player::SPEED_STRAIGHT_FOCUSED,
            straight / 2.0,
        );
        space.write::<f32>(super::PLAYER + super::player::SPEED_DIAGONAL, diagonal);
        space.write::<f32>(
            super::PLAYER + super::player::SPEED_DIAGONAL_FOCUSED,
            diagonal / 2.0,
        );
        space.write::<f32>(super::PLAYER + super::player::SCALE_ACROSS, 1.0);
        space.write::<f32>(super::PLAYER + super::player::SCALE_DOWN, 1.0);
    }

    pub fn the_player_is_playing(&self) {
        self.space()
            .write::<u8>(super::PLAYER + super::player::STATE, PLAYER_PLAYING);
    }

    /// The hit landed, with the frames a bomb is still taken in under it.
    pub fn the_player_has_been_hit(&self, a_bomb_is_taken_for: i32) {
        let space = self.space();
        space.write::<u8>(super::PLAYER + super::player::STATE, PLAYER_HIT);
        space.write::<i32>(
            super::PLAYER + super::player::A_BOMB_STILL_TAKES_IT_BACK,
            a_bomb_is_taken_for,
        );
    }

    pub fn the_frames_a_bomb_is_still_taken_in(&self, frames: i32) {
        self.space().write::<i32>(
            super::PLAYER + super::player::A_BOMB_STILL_TAKES_IT_BACK,
            frames,
        );
    }

    /// The player walking back in, with `untouchable` on the count under them: the game's own register
    /// leaves 120 there at a stage's build (`0x14004a14c`) and its death leaves nothing, and no state but
    /// [`super::player::INVULNERABLE`] reads that field — see `the_player_a_stage_of_th06c_starts`.
    pub fn the_player_is_coming_back(&self, untouchable: i32) {
        let space = self.space();
        space.write::<u8>(super::PLAYER + super::player::STATE, PLAYER_COMING_BACK);
        space.write::<i32>(super::PLAYER + super::player::UNTOUCHABLE, untouchable);
    }

    pub fn the_player_is_invulnerable_for(&self, frames: i32) {
        let space = self.space();
        space.write::<u8>(super::PLAYER + super::player::STATE, PLAYER_INVULNERABLE);
        space.write::<i32>(super::PLAYER + super::player::UNTOUCHABLE, frames);
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

    pub fn the_players_state(&self) -> u8 {
        self.space()
            .read::<u8>(super::PLAYER + super::player::STATE)
    }

    pub fn the_frames_the_player_is_untouchable_for(&self) -> i32 {
        self.space()
            .read::<i32>(super::PLAYER + super::player::UNTOUCHABLE)
    }

    pub fn a_bomb_is_going_off(&self, going_off: bool) {
        self.space().write::<u32>(
            super::PLAYER + super::player::BOMB_IN_USE,
            u32::from(going_off),
        );
    }

    // ---- the enemies, the bullets, the items ----

    pub fn enemies_alive(&self, enemies: i32) {
        self.space()
            .write::<i32>(super::ENEMY_MANAGER + super::enemy_manager::LIVE, enemies);
    }

    pub fn bullets_alive(&self, bullets: i32) {
        self.space()
            .write::<i32>(super::BULLET_MANAGER + super::bullet_manager::LIVE, bullets);
    }

    /// One bullet in its slot, flying: where it is, its step, and the size the hit test is handed.
    pub fn a_bullet_of_size_is_at(
        &self,
        slot: usize,
        x: f32,
        y: f32,
        step: (f32, f32),
        size: (f32, f32),
    ) {
        let bullet = super::BULLET_MANAGER
            + super::bullet_manager::BULLETS
            + slot * super::bullet_manager::BYTES;
        let space = self.space();
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
        self.space().write::<u16>(
            super::BULLET_MANAGER
                + super::bullet_manager::BULLETS
                + slot * super::bullet_manager::BYTES
                + super::bullet_manager::STATE,
            0,
        );
    }

    /// **A boss fighting**, in the first enemy slot: in use, solid, carrying the boss bit, with the life and
    /// the frames its attack has run — and the panel's own byte set, as the script's naming sets it.
    pub fn a_boss_is_fighting(&self, life: i32, attack_frames: i32) {
        let boss = super::ENEMY_MANAGER + super::enemy_manager::ENEMIES;
        let space = self.space();
        space.write::<u8>(boss + super::enemy::FLAGS, ENEMY_IN_USE);
        space.write::<u8>(
            boss + super::enemy::SOLIDITY,
            ENEMY_SOLID_AND_TESTED | ENEMY_IS_THE_BOSS,
        );
        space.write::<i32>(boss + super::enemy::LIFE, life);
        space.write::<i32>(boss + super::enemy::LIFE + size_of::<i32>(), life.max(1));
        space.write::<i32>(boss + super::enemy::TIMER, attack_frames);
        space.write::<u8>(super::BOSS_PRESENT, 1);
    }

    /// **An enemy nothing collides with**, in its slot: in use, its body untested, at `x,y` and `size` across as
    /// the game keeps it — three times the half its test would reach.
    pub fn an_enemy_that_passes_through_is_at(&self, slot: usize, x: f32, y: f32, size: f32) {
        let body = super::ENEMY_MANAGER
            + super::enemy_manager::ENEMIES
            + slot * super::enemy_manager::BYTES;
        let space = self.space();
        space.write::<u8>(body + super::enemy::FLAGS, ENEMY_IN_USE);
        space.write::<u8>(body + super::enemy::SOLIDITY, 0);
        space.write::<f32>(body + super::enemy::POSITION, x);
        space.write::<f32>(body + super::enemy::POSITION + size_of::<f32>(), y);
        space.write::<f32>(body + super::enemy::SIZE, size);
        space.write::<f32>(body + super::enemy::SIZE + size_of::<f32>(), size);
    }

    pub fn no_boss_is_fighting(&self) {
        let boss = super::ENEMY_MANAGER + super::enemy_manager::ENEMIES;
        let space = self.space();
        space.write::<u8>(boss + super::enemy::FLAGS, 0);
        space.write::<u8>(boss + super::enemy::SOLIDITY, 0);
        space.write::<u8>(super::BOSS_PRESENT, 0);
    }

    pub fn the_bosses_attack_has_run_for(&self) -> i32 {
        self.space()
            .read::<i32>(super::ENEMY_MANAGER + super::enemy_manager::ENEMIES + super::enemy::TIMER)
    }

    /// **A laser in `slot`**, pointing `angle` from `origin` and `width` across as the game keeps it —
    /// four times the half its test reaches — from `near` to `far` along, `frames` into `phase`, and hitting
    /// from its `hits_from`th frame.
    #[allow(clippy::too_many_arguments)]
    pub fn a_laser_is(
        &self,
        slot: usize,
        origin: (f32, f32),
        angle: f32,
        width: f32,
        (near, far): (f32, f32),
        phase: u8,
        frames: i32,
        hits_from: i32,
    ) {
        let beam = super::LASERS + slot * super::laser::BYTES;
        let space = self.space();
        space.write::<u8>(beam + super::laser::IN_USE, 1);
        space.write::<f32>(beam + super::laser::ORIGIN, origin.0);
        space.write::<f32>(beam + super::laser::ORIGIN + size_of::<f32>(), origin.1);
        space.write::<f32>(beam + super::laser::ANGLE, angle);
        space.write::<f32>(beam + super::laser::WIDTH, width);
        space.write::<f32>(beam + super::laser::NEAR, near);
        space.write::<f32>(beam + super::laser::FAR, far);
        space.write::<f32>(beam + super::laser::GROWS, 0.0);
        space.write::<i32>(beam + super::laser::FRAMES, frames);
        space.write::<u8>(beam + super::laser::PHASE, phase);
        space.write::<i32>(beam + super::laser::HITS_FROM, hits_from);
    }

    /// **And how many frames the laser in `slot` takes to narrow away once it fades**, which is what makes a
    /// fading one a stub of its own line — see `super::laser::NARROWS_OVER`. Nought, which a slot never given
    /// one holds and [`Self::a_laser_is`] does not write, is a laser that keeps its whole length while it fades.
    pub fn the_laser_narrows_over(&self, slot: usize, frames: i32) {
        let beam = super::LASERS + slot * super::laser::BYTES;
        self.space()
            .write::<i32>(beam + super::laser::NARROWS_OVER, frames);
    }

    /// And that slot empty again.
    pub fn no_laser_is_in(&self, slot: usize) {
        self.space().write::<u8>(
            super::LASERS + slot * super::laser::BYTES + super::laser::IN_USE,
            0,
        );
    }

    /// The spell card up, by the index the card's own start shifts into the records by.
    pub fn the_spellcard_is(&self, card: u32) {
        let space = self.space();
        space.write::<i32>(
            super::ENEMY_MANAGER + super::enemy_manager::SPELLCARD_IS_ACTIVE,
            1,
        );
        space.write::<u32>(super::CURRENT_CARD, card);
    }

    pub fn no_spellcard_is_running(&self) {
        self.space().write::<i32>(
            super::ENEMY_MANAGER + super::enemy_manager::SPELLCARD_IS_ACTIVE,
            0,
        );
    }

    /// The stage callback's own fill of the sixty-four records: thirty-two draws of the generator apiece
    /// into the record, then the magic, the sizes, the version and the index over the front of it, and the
    /// two counts cleared — `0x1400243f0` to `0x1400249f5`.
    pub fn fills_the_card_records(&self) {
        let space = self.space();
        for card in 0..super::CARDS {
            let record = super::CARD_HISTORY + card * super::CARD_BYTES;
            for word in 0..super::CARD_BYTES / size_of::<u16>() {
                let drawn = self.draws_from_the_generator();
                space.write::<u16>(record + word * size_of::<u16>(), drawn);
            }
            space.write::<u32>(record, CATK_MAGIC);
            space.write::<u32>(record + 4, CATK_SIZES);
            space.write::<u8>(record + 8, CATK_VERSION);
            space.write::<u16>(record + 0x10, card as u16);
            space.write::<u32>(record + super::CATK_ATTEMPTS, 0);
        }
    }

    /// A card started, as `0x140014c99` onwards does it: the name copied in, its sum written beside it, and
    /// the attempts stepped where the game counts one.
    pub fn starts_the_card(&self, card: u32, name: &str) {
        let record = super::CARD_HISTORY + card as usize * super::CARD_BYTES;
        let space = self.space();
        // **A copy into a fixed field, which always leaves a terminator**: the span is `CATK_NAME_BYTES`
        // long and the last of them stays nought, so a name as long as the field is one the copy cuts. What
        // reads it back looks for that nought — a span with none of them is the generator's noise and not a
        // name — so a fake that filled the field would write a record the game could not have written.
        let mut bytes = vec![0u8; super::CATK_NAME_BYTES];
        let fits = super::CATK_NAME_BYTES - 1;
        for (into, byte) in bytes.iter_mut().take(fits).zip(name.bytes()) {
            *into = byte;
        }
        space.write_bytes(record + super::CATK_NAME, &bytes);
        // And the sum over the bytes that went in rather than over the name that was handed over, which is
        // the same thing wherever the name fits: the game's own accumulation at `0x140014d00` runs over the
        // field it filled.
        space.write::<u8>(
            record + super::CATK_NAME_SUM,
            super::name_sum(&bytes[..name.len().min(fits)]),
        );
        let attempts = space.read::<u16>(record + super::CATK_ATTEMPTS);
        space.write::<u16>(record + super::CATK_ATTEMPTS, attempts.saturating_add(1));
        self.the_spellcard_is(card);
    }

    pub fn card_attempts(&self, card: u32) -> u16 {
        self.space().read::<u16>(
            super::CARD_HISTORY + card as usize * super::CARD_BYTES + super::CATK_ATTEMPTS,
        )
    }

    pub fn card_name(&self, card: u32) -> Option<String> {
        let bytes = self.space().read_bytes(
            super::CARD_HISTORY + card as usize * super::CARD_BYTES + super::CATK_NAME,
            super::CATK_NAME_BYTES,
        );
        let end = bytes.iter().position(|byte| *byte == 0)?;
        Some(String::from_utf8_lossy(&bytes[..end]).into_owned())
    }

    /// A line of a conversation up, by its index in the panel's block, and none.
    pub fn a_line_of_dialogue_is_up(&self, line: i32) {
        self.space()
            .write::<i32>(GUI_IMPL.start + super::gui_impl::MSG_IDX, line);
    }

    pub fn no_dialogue_is_up(&self) {
        self.a_line_of_dialogue_is_up(-1);
    }

    // ---- the score file ----

    /// The game's `malloc` and `free`, functions of its own exe: what orb calls for a block of the heap the
    /// game's copy of its score file comes off.
    pub fn hands_over_its_heap(&self, malloc: usize, free: usize) {
        super::set_mallocs(malloc);
        super::set_frees(free);
    }

    // ---- the renderer ----

    /// **Sprites a draw has queued**, `count` vertices more in the renderer's batch — which is how every one
    /// of this game's draws hands its sprites over.
    pub fn queues_sprites(&self, count: i32) {
        let queued = RENDERER.start + super::renderer::QUEUED;
        let now = self.space().read::<i32>(queued);
        self.space().write::<i32>(queued, now + count);
    }

    /// How many vertices the renderer's batch holds, not yet drawn.
    pub fn the_sprites_queued(&self) -> i32 {
        self.space()
            .read::<i32>(RENDERER.start + super::renderer::QUEUED)
    }

    /// **Where the batch is and what the flush is told about it**, which are what the game's own flush is
    /// called with: the sprites' own block and the word beside the count.
    pub fn the_batch(&self) -> (usize, i32) {
        (
            RENDERER.start + super::renderer::SPRITES,
            self.space()
                .read::<i32>(RENDERER.start + super::renderer::FLUSHED_AS),
        )
    }

    /// **The view the frame's own call sets, out of the block it reads**: the corner, and the corner plus
    /// the size — which is how the call hands them on.
    pub fn the_frames_view(&self) -> [u32; 4] {
        let space = self.space();
        let x = space.read::<u32>(super::WHAT_THE_VIEWPORT_WAS_ASKED_FOR);
        let y = space.read::<u32>(super::WHAT_THE_VIEWPORT_WAS_ASKED_FOR + size_of::<u32>());
        let width = space.read::<u32>(super::THE_VIEWPORTS_WIDTH);
        let height = space.read::<u32>(super::THE_VIEWPORTS_HEIGHT);
        [x, y, x + width, y + height]
    }

    /// A block off the game's heap, sixteen-aligned, which is what its `malloc` answers.
    ///
    /// # Panics
    /// Where the heap is spent, which a test asking for megabytes of score file is.
    pub fn takes_a_block_off_its_heap(&self, len: usize) -> usize {
        let block = self.space().read::<usize>(HEAP_GIVEN);
        let next = (block + len).next_multiple_of(0x10);
        assert!(
            next <= HEAP.end,
            "the laid-out game's heap is spent: {len} bytes asked with {} left",
            HEAP.end - block
        );
        self.space().write::<usize>(HEAP_GIVEN, next);
        block
    }

    /// **The copy of the score file the game read at its startup**: a block off its heap holding `bytes`,
    /// pointed at and measured where the parse reads them — `0x14002c2b2` and `0x14002c2b9`.
    pub fn the_score_file_copy_is(&self, bytes: &[u8]) {
        let block = self.takes_a_block_off_its_heap(bytes.len().max(1));
        self.space().write_bytes(block, bytes);
        self.space().write::<usize>(super::SCORE_FILE_COPY, block);
        self.space().write::<i32>(
            super::SCORE_FILE_COPY_LENGTH,
            i32::try_from(bytes.len()).expect("a score file of a few kilobytes"),
        );
    }

    /// The copy as it stands — whoever installed it — and `None` where the game read no file.
    pub fn the_score_file_copy(&self) -> Option<Vec<u8>> {
        let block = self.space().read::<usize>(super::SCORE_FILE_COPY);
        if block == 0 {
            return None;
        }
        let len = usize::try_from(self.space().read::<i32>(super::SCORE_FILE_COPY_LENGTH)).ok()?;
        Some(self.space().read_bytes(block, len))
    }

    // ---- the input ----

    /// The word the run acts on and the frame before's, which is what an edge is taken from.
    pub fn the_run_acts_on_the_word(&self, word: u32) {
        let space = self.space();
        let before = space.read::<u32>(super::INPUT_THE_RUN_ACTS_ON);
        space.write::<u32>(super::INPUT_THE_RUN_ACTS_ON + size_of::<u32>(), before);
        space.write::<u32>(super::INPUT_THE_RUN_ACTS_ON, word);
    }

    pub fn the_word_the_run_acts_on(&self) -> u32 {
        self.space().read::<u32>(super::INPUT_THE_RUN_ACTS_ON)
    }

    pub fn the_word_before(&self) -> u32 {
        self.space()
            .read::<u32>(super::INPUT_THE_RUN_ACTS_ON + size_of::<u32>())
    }

    /// The 256 bytes the game plays a run off, indexed by DirectInput's own codes — see
    /// `super::THE_KEYS_THE_GAME_READS`.
    pub fn the_keys_the_game_reads(&self) -> Vec<u8> {
        self.space()
            .read_bytes(super::THE_KEYS_THE_GAME_READS, super::KEYS_IN_A_SNAPSHOT)
    }

    pub fn the_game_reads_the_keys(&self, keys: &[u8]) {
        self.space()
            .write_bytes(super::THE_KEYS_THE_GAME_READS, keys);
    }

    /// The table from a virtual key to the game's own index, as this image lays it out.
    pub fn the_index_of_the_key(&self, key: u8) -> Option<usize> {
        KEYS.iter()
            .find(|(wanted, _)| *wanted == u16::from(key))
            .map(|(_, index)| usize::from(*index))
    }

    // ---- the music ----

    /// Which track is playing, and whether: the statics the frame's catch-up reads.
    pub fn plays_the_track(&self, track: i32) {
        let space = self.space();
        space.write::<i32>(super::THE_TRACK_PLAYING, track);
        space.write::<u8>(super::THE_TRACK_IS_PLAYING, u8::from(track >= 0));
    }

    /// **How many walks of the calc chain the music has not heard**, which is the count the frame's own
    /// sound catch-up turns into a seek and clears — see `super::tops_up_the_sound`, which does nothing at
    /// all while it is nought. Written here because orb's own walk is what raises it, and a frame that
    /// walked once raises it never: a test that wants the catch-up to reach the game says so.
    pub fn the_music_is_behind_by(&self, walks: i32) {
        self.space().write::<i32>(super::THE_EXTRA_WALKS, walks);
    }

    pub fn the_music_is_behind_by_now(&self) -> i32 {
        self.space().read::<i32>(super::THE_EXTRA_WALKS)
    }

    pub fn the_track_playing(&self) -> i32 {
        self.space().read::<i32>(super::THE_TRACK_PLAYING)
    }

    /// How many walks of the calc chain the music has not heard, which the frame's catch-up seeks past.
    pub fn the_extra_walks(&self) -> i32 {
        self.space().read::<i32>(super::THE_EXTRA_WALKS)
    }

    /// **A word of the sound library's own block**, which a restore leaves alone — the first of the five
    /// locks in it, `0x140497a80`, whose `LockCount` is the word an e2e test can tell two values of apart.
    ///
    /// Written and read rather than played through: what orb does with this block is hold it back from a
    /// restore, and it reads no word of it.
    pub fn the_sound_librarys_word_is(&self, word: u32) {
        self.space().write::<u32>(THE_SOUND_LIBRARYS_WORD, word);
    }

    pub fn the_sound_librarys_word(&self) -> u32 {
        self.space().read::<u32>(THE_SOUND_LIBRARYS_WORD)
    }

    /// **And a word just below that block**, which a restore therefore puts back: `0x1404979a0`, under the
    /// `0x1404979b4` the library's own code reaches no lower than.
    pub fn the_word_below_the_sound_library_is(&self, word: u32) {
        self.space().write::<u32>(BELOW_THE_SOUND_LIBRARY, word);
    }

    pub fn the_word_below_the_sound_library(&self) -> u32 {
        self.space().read::<u32>(BELOW_THE_SOUND_LIBRARY)
    }

    /// **The render target view the first window of the renderer's table holds**, which a restore leaves
    /// alone — see `super::THE_WINDOWS`. Written and read rather than made: what orb does with the table is
    /// hold it back from a restore, and it reads no word of it.
    pub fn the_first_windows_view_is(&self, view: usize) {
        self.space().write::<usize>(THE_FIRST_WINDOWS_VIEW, view);
    }

    pub fn the_first_windows_view(&self) -> usize {
        self.space().read::<usize>(THE_FIRST_WINDOWS_VIEW)
    }

    /// **And the count of windows open, just below that table**, which a restore therefore puts back.
    pub fn the_count_of_windows_is(&self, count: u32) {
        self.space().write::<u32>(THE_COUNT_OF_WINDOWS, count);
    }

    pub fn the_count_of_windows(&self) -> u32 {
        self.space().read::<u32>(THE_COUNT_OF_WINDOWS)
    }
}

/// The first window's render target view, `+0x108` into the entry at `0x1407e3898` — where `0x140235567`
/// has `0x1402516a0` make it. Inside [`super::THE_WINDOWS`].
const THE_FIRST_WINDOWS_VIEW: usize = 0x1_407e_39a0;
/// And the count of the windows open, `0x1407e3890`, which `0x140234971` counts up.
const THE_COUNT_OF_WINDOWS: usize = 0x1_407e_3890;

/// The first of the sound library's five locks — `0x140497a80`, initialised at `0x1400a5126` — whose own
/// first word is what an e2e test writes. Inside [`super::THE_SOUND_LIBRARYS_BLOCK`].
const THE_SOUND_LIBRARYS_WORD: usize = 0x1_4049_7a80;
/// And a word below the block, `0x1404979a0`: the lowest address anything reaches in it is `0x1404979b4`,
/// so this one is the game's own memory and comes back like any other.
const BELOW_THE_SOUND_LIBRARY: usize = 0x1_4049_79a0;
