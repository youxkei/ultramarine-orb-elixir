//! A th08 process image laid out in an address space, so that the real [`Th08`](super::Th08) has
//! something to read in a test with no game running.
//!
//! **What a frame of orb's own reads out of this space** is the device it draws through, the window it
//! asks about, the chain it calls the update and the draw at, the viewport it writes, the queue of quads
//! it empties and draws around them and the fog flag it forces on — and the update, the draw and the
//! three calls on that queue are functions an e2e test hands over rather than code that would have to be
//! here. See [`Image::hands_over_the_frames_own_calls`].
//!
//! **And a run, as far as [`Th08::read_state`](super::Th08) reads one**: the scene, the stage, the
//! difficulty, the clock, the generator, the run's own numbers behind the pointer its build allocates,
//! the kind of run, the player with its box, the dialogue, the boss, the enemies, the bullets, the lasers
//! and the items. Each of those is laid out where the game itself keeps it, so a test can say what orb
//! answers for a game in a state the game can really be in.
//!
//! The addresses are the game's own, written from the same constants the reads use, so what this cannot
//! catch is a constant that is wrong: a wrong one is wrong on both sides at once, and only the real game
//! says otherwise — every one of them was read against `th08.exe` of md5 `77b6785e`, see `th08/asm.md`.
//! What it does catch is everything built on top of them, which is where the code is.
//!
//! **The values are this file's own, though, and taking one from [`Th08`](super::Th08) is what is refused
//! here** — th07's image has why: a scene number, a screen number, a state or a mask shared between the
//! writer and the reader is silent when wrong, and what it costs is a piece of the mode never happening.
//! So each value below carries its own literal and where in the exe it was read.
//!
//! **And one block's base here is this file's own**: the queue of quads, which the real game takes off
//! the allocator so that it is at no fixed address at all. Its length and every offset inside it are the
//! game's, read off the three functions that move them — see [`QUAD_QUEUE`].
//!
//! **What no offset of that block can catch is itself.** orb only *calls* this game's three queue
//! functions — [`EMPTY_QUEUE`](super::EMPTY_QUEUE), [`DRAW_QUEUE`](super::DRAW_QUEUE) and the fog write
//! between them — and never reads a field of the object, so there is no read of one to be wrong. They are
//! the game's all the same, because what a laid-out queue is for is answering the way the real one does:
//! a frame that emptied it and drew it once leaves it where the real one leaves it.
//!
//! **Nothing here goes through a slide**: this game is a 32-bit image with relocations stripped
//! (`Characteristics 0x10f`), so the loader puts it at the `0x400000` its addresses are written against
//! and `Game::set_image_base` is never told otherwise.

use std::ffi::CStr;
use std::ops::Range;
use std::sync::Arc;

use orb_api::Kind;
use orb_api::{Device, Hwnd};
use orb_sim::{Sim, Space};

/// The game's static data, as the real one is: **one range**, which is what orb reads out of the PE and
/// writes in the log every launch — `.data 0x004c6000..0x018d9840 (21051456 bytes)`.
///
/// The section header's own virtual size and not its raw size, which is `0x6e00`: the rest is the globals
/// the loader zeroes, and all but five of the addresses below are in it. Twenty-one megabytes, which is
/// the range a snapshot of a chapter is a copy of, which a launch of the real game reads back as
/// `memory: saving data 0x004c6000+0x1413840` with five heap regions beside it.
const DATA: Range<usize> = 0x004c_6000..0x018d_9840;

/// **What a card being held reads as**, for [`Image::a_card_is_held`]: something rather than nothing,
/// which is the whole of what the game's own gate asks — 0x416b90 tests the byte and not its value.
const A_CARD_HELD: u8 = 1;

/// **How far apart the jobs a test lays on the chain's calc list stand**, and how far the first of them
/// is past the chain itself: a number of this file's choosing, the game's own allocator being where a real
/// element comes from — see [`Image::the_calc_list_holds`], and *The chain* in `asm.md` for the three
/// fields one carries. Wide enough that no element's own fields reach the next.
const A_JOBS_OWN_SPACE: usize = 0x100;

/// **The object the drawing queues its quads in**, `SprtCtrlInf`: a base of this file's choosing, the
/// real one being off the allocator and so in none of [`DATA`], and the length the game's own — the
/// 0x2a2570 bytes WinMain asks for at 0x4419d0.
const QUAD_QUEUE: Range<usize> = 0x0300_0000..0x0300_0000 + 0x2a_2570;
/// How many quads are in the queue, how many times it has been drawn, the cursor the next quad is written
/// at, the cursor the next draw starts from, and the buffer the two of those point into.
///
/// **The game's own offsets**, read off the three functions that move them:
///
/// - 0x462e00 empties the queue — the count at `+0x2520` to nothing, the write cursor at `+0x2a2524` back
///   to `this+0x2524`, and the drawn-from cursor at `+0x2a2528` after it;
/// - 0x462f10 queues one — six 0x1c-byte vertices copied out of the four it is handed (offsets 0, 0x1c,
///   0x38, 0x1c, 0x38, 0x54, which is the two triangles of a quad), the cursor [`QUAD_BYTES`] further on
///   and the count one higher;
/// - 0x462e40 draws what is queued — nothing at all where the count is nothing, and otherwise
///   `DrawPrimitiveUP` over `count * 2` triangles from the drawn-from cursor, then that cursor up to the
///   write cursor, the count back to nothing and `+0x18` one higher.
const QUEUED: usize = 0x2520;
const QUEUE_DRAWS: usize = 0x18;
const QUEUE_WRITE_AT: usize = 0x2a_2524;
const QUEUE_DRAWN_FROM: usize = 0x2a_2528;
const QUEUE_BUFFER: usize = 0x2524;
/// How many bytes a quad takes in that buffer: six vertices of 0x1c, which is what 0x462fb5 adds.
const QUAD_BYTES: usize = 0xa8;

/// **The run's own numbers**, which this game allocates per run and frees at the next build — see
/// `G_RUN_NUMBERS`, whose doc has the launch that read `lives=-128` through a freed one. A base of this
/// file's choosing and the length the game's own, `RUN_NUMBER_BYTES`.
const RUN_NUMBERS: Range<usize> = 0x0340_0000..0x0340_00e4;

/// **The rest of the run's seal** — see `Th08`'s `RUN_SEAL_KEY`: the salt the reseal draws beside the key, and
/// the step the sum adds to the seal for each of its bytes. Nothing of this file sets the step, and neither
/// does anything it lays out: what the check asks of it holds at nought as at anything else.
const RUN_SEAL_SALT: usize = 0xb4;
const RUN_SEAL_STEP: usize = 0xd8;
/// The keys this file's reseal writes, in the range the game's own draws are (`0x198f` on from a draw under
/// `100000`, 0x406e68 and 0x406e85), which the run manager's own update checks seven of the run's words
/// against (0x43a4f9).
const THE_SEALS_KEY: u32 = 0x1990;
const THE_SEALS_SALT: u32 = 0x1991;
/// Where the game manager keeps the pointer to the third of the sum's blocks, and the configuration the
/// fourth is — 0x439a7d and 0x439a93.
const GM_BLOCK_THE_SUM_TAKES: usize = 0x4;
const CONFIGURATION: usize = 0x017c_e870;

/// **What the fields of a freed front end read**, which is this file's own stand-in for whatever the
/// allocator left there: a number that is no screen of this game and no item of any of its lists, so a
/// read that answered it would be plainly wrong rather than accidentally right.
///
/// The one a launch really read was `1900876895` — see `Image::the_front_end_is_freed`.
const WHAT_A_FREED_FRONT_END_READS: i32 = 0x713a_3f5f;

/// **The front end**, which orb reaches through the callback of the element it registers: long enough to
/// hold the screen at `+0x14428` and the settled counter at `+0x1442c`, which are the last two fields of
/// it orb reads.
const FRONT_END: Range<usize> = 0x0350_0000..0x0351_5000;

/// **The screen a run ends at**, `0x477b0` bytes as the allocator call at 0x4582b0 asks for — long enough
/// for the name entry's field at `+0x462f3` — at a base of this file's choosing, the real one being off the
/// game's own heap. **Laid out only while that screen is up**, as the real one is allocated as it is
/// registered and freed as it is torn down: laid out for a whole launch, it was a region every chapter's
/// snapshot of a run held and the real game's never has.
const RESULT_SCREEN: Range<usize> = 0x0440_0000..0x0440_0000 + 0x477b0;

/// **The word a held key repeats through**, `[0x164d538]`: set once the input word has been held `0x1e`
/// frames and then every eight (0x445615 through 0x44565a), and put to nothing with the other two at the
/// supervisor's switch. orb reads none of it, so it is this file's alone.
const INPUT_REPEATING: usize = 0x0164_d538;

/// **The allocation the screen a stage is played on hangs off its `+0x8`**, `FRScreenImplInf`, `0x230b8`
/// bytes as the allocator call at 0x437b3d asks for. Long enough to hold the dialogue's own line at
/// `+0x2181c`.
const RUN_SCREEN_INNER: Range<usize> = 0x0360_0000..0x0362_30b8;

/// **The active team member's own numbers**, a `.sht` loaded out of `th08.dat`: long enough to hold the
/// four speeds at `+0x24` through `+0x30` and the full power at `+0x1c`.
const TEAM_SPEC: Range<usize> = 0x0370_0000..0x0370_1000;

/// **The sheets of one `.anm` slot**, which the real game takes off its own allocator: a base of this
/// file's choosing, and long enough for the `0xc`-byte entries this file lays out. See
/// [`Image::an_anm_slot_is_loaded`].
const ANM_SHEETS: Range<usize> = 0x0380_0000..0x0380_0000 + ANM_SLOTS_SHEETS;
/// Long enough for every one of the manager's slots to have its own — `0x100` slots
/// [`A_SLOTS_SHEETS_APART`] apart — so that a test naming any slot lays its sheets out inside the block.
const ANM_SLOTS_SHEETS: usize = 0x100 * A_SLOTS_SHEETS_APART;

/// **The stream a played track has and the wrapper the sound manager keeps DirectSound in**, both off
/// the allocator on the real game and so at bases of this file's choosing. The lengths are the game's —
/// `0x78` for the stream, four bytes for the wrapper — and a page apiece here, so that a range orb asks
/// for past the object's own end is still memory rather than a fault.
const SOUND_STREAM: Range<usize> = 0x0390_0000..0x0390_1000;
const SOUND_DEVICE: Range<usize> = 0x03a0_0000..0x03a0_1000;
/// **The source the stream reads through, and `thbgm.dat`'s table of tracks behind it** — a page with the
/// source at its start and the records from [`MUSIC_RECORDS_AT`] on, `0x34` bytes apart as the lookup at
/// 0x45c7c4 steps them.
const MUSIC_SOURCE: Range<usize> = 0x03d0_0000..0x03d0_1000;
const MUSIC_RECORDS_AT: usize = 0x100;
const MUSIC_RECORD_BYTES: usize = 0x34;
/// **The stage's own info block**, which the real game loads out of the stage's file — a page here, long
/// enough for the three names at the end of it.
const STAGE_INFO: Range<usize> = 0x03e0_0000..0x03e0_1000;
/// **A script context per enemy slot**, the game's own allocations hung off `ENEMY_SCRIPT`: one block of
/// [`ENEMY_SCRIPT_BYTES`] apiece, laid out here so that a test can give an enemy a behaviour to run.
const ENEMY_SCRIPTS: Range<usize> =
    0x03b0_0000..0x03b0_0000 + super::ENEMY_SLOTS * ENEMY_SCRIPT_BYTES;
const ENEMY_SCRIPT_BYTES: usize = 0x40;

/// **An animation instruction per bullet slot**, the one a bullet's own sprite is waiting on: one block of
/// [`BULLET_ANIMATION_BYTES`] apiece, laid out here so that a test can say a bullet's spawn has frames
/// left to run — see `BULLET_SPRITE`, which is what that decides.
const BULLET_ANIMATIONS: Range<usize> =
    0x03c0_0000..0x03c0_0000 + super::BULLET_SLOTS * BULLET_ANIMATION_BYTES;
const BULLET_ANIMATION_BYTES: usize = 0x10;

/// **The sound manager and how much of it there is**: `0x18b8a68`, which is inside [`DATA`] and so needs
/// no block of its own, and the `0x5224` bytes its own constructor zeroes — `0x1489` dwords at 0x45c1af.
///
/// This file's own literals, for the reason the module doc gives: an extent an e2e test asserts and orb
/// answers from is one number written twice, and a wrong one read off `Th08` would be a snapshot holding
/// back the wrong bytes and a test agreeing with it.
const SOUND_PLAYER: Range<usize> = 0x018b_8a68..0x018b_8a68 + 0x5224;

/// **How far apart the sheets of two `.anm` slots are laid out**, and the three values a laid-out slot
/// carries: a file pointer that is something rather than nothing, a texture and a block. Numbers of this
/// file's own — what they have to be is distinguishable, so that a range answered for one slot cannot
/// pass for another's.
const A_SLOTS_SHEETS_APART: usize = 0x100;
const A_LOADED_FILE: usize = 0x0410_0000;
const A_TEXTURE: u32 = 0x0420_0000;
const A_BLOCK: u32 = 0x0430_0000;
/// And where in the entry the second of those two is, `+0x4` — the block 0x466160 hands back to the
/// game's own allocator, beside the texture it releases.
const A_SHEETS_BLOCK: usize = 0x4;

/// **A kind a command in the manager's queue carries**, of this file's own choosing: one of the eight the
/// runner switches on, and anything but nothing would do — what the runner asks is whether the slot holds
/// a kind at all.
const A_SOUND_COMMAND: u32 = 3;

/// The four speeds that `.sht` holds, as one launch of the real game measured them: 4.00 straight, 2.83
/// diagonal, 2.00 focused and 1.41 focused diagonal — see `SPEED_STRAIGHT`, which has the recipe.
const SPEEDS: (f32, f32, f32, f32) = (4.0, 2.83, 2.0, 1.41);
/// And the power the shot stops adding to, which the panel's own compare at 0x437215 puts at 128.
const FULL_POWER: f32 = 128.0;
/// And the width of the player's own box, of which `+0xc` of the spec holds twice — 0x44d7e3 halves it
/// into the player's half. A launch read the half back as `0.82`, so the whole is `1.65`.
const PLAYER_BOX_ACROSS: f32 = 1.65;

/// **The record this game keeps per spell card**, as its own `CATK` chunk: 222 of them at `[gm+0x40]`,
/// `0x22c` bytes apiece, each stamped with the tag, its length twice, version 3 and its own index by the
/// init at 0x43bbe1 before the file is read over it.
///
/// This file's own literals, for the reason the module doc gives — and the tag is `CATK` spelled out
/// rather than the dword, so that a wrong byte order reads as a wrong tag.
const CARD_RECORDS: usize = 0x40;
const CARD_RECORD_BYTES: usize = 0x22c;
const CARDS: usize = 0xde;
const CARD_RECORD_TAG: &[u8; 4] = b"CATK";
const CARD_RECORD_VERSION: u8 = 3;
const CARD_RECORD_LENGTH: usize = 0x4;
const CARD_RECORD_LENGTH_AGAIN: usize = 0x6;
const CARD_RECORD_VERSION_AT: usize = 0x8;
const CARD_RECORD_INDEX: usize = 0xc;
const CARD_RECORD_SUM: usize = 0xe;
const CARD_RECORD_NAME: usize = 0x10;
/// And the six arrays of thirteen dwords, in the order the game's own checksum adds them.
const CARD_RECORD_ARRAYS: [usize; 6] = [0x158, 0x124, 0xf0, 0x1f4, 0x1c0, 0x18c];
const CARD_RECORD_CAPTURES: usize = 0x158;
const CARD_RECORD_ATTEMPTS: usize = 0x124;
const CARD_RECORD_SLOTS: usize = 13;

/// **The spell card that is up**, a static of its own: the flags word whose bit 0 says one is, and the
/// number beside it that indexes the records above.
const SPELL_CARD_IS_UP: u32 = 0x1;
const SPELL_CARD_NUMBER: usize = 0x8;
/// And what it pays in time orbs, with the count of each of its two timers beside it — see
/// [`Image::the_card_pays`], and `Th08`'s `CARD_TIME_ORBS` for the ramp.
const SPELL_CARD_TIME_ORBS: usize = 0x10;
const SPELL_CARD_TIME_LEFT: usize = 0x108 + 0x8;
const SPELL_CARD_TIME: usize = 0x114 + 0x8;

/// **The trail a long enemy is tested along**: the array of places it has been, the count of them, and
/// the flags byte whose being anything at all is what says it has one — and whose bit 1 tapers each box.
///
/// This file's own literals, for the reason the module doc gives, and the walk's own first index and step
/// with them: 1 and 6.
const ENEMY_BOXES_AT: usize = 0x3394;
const ENEMY_BOX_BYTES: usize = 0x1c;
const ENEMY_FIRST_BOX: usize = 1;
const ENEMY_BOX_EVERY: usize = 6;
const ENEMY_BOXES: usize = 0x5350;
const ENEMY_BOX_FLAGS: usize = 0x534c;
const ENEMY_BOXES_TAPER: u8 = 0x2;
/// And a bit of that byte which is not the taper, so that an enemy with a trail and no taper still has a
/// byte the game's own gate reads as something.
const ENEMY_HAS_A_TRAIL: u8 = 0x1;

/// **The contexts the stage's script is run in**, `enemies+0x9dcdd0`: sixteen objects of `0x10` bytes,
/// each a timer whose count is at [`A_COUNTDOWN_COUNT`] and an instruction pointer at `+0xc` — the shape
/// 0x42a243 array-constructs and 0x42c7d5 walks.
///
/// This file's own literals, for the reason the module doc gives.
const SCRIPT_CONTEXTS: usize = 0x9dcdd0;
const SCRIPT_CONTEXT_BYTES: usize = 0x10;
const SCRIPT_CONTEXTS_KEPT: usize = 0x10;
const SCRIPT_CONTEXT_AT: usize = 0xc;

/// **How far apart the script contexts behind the first are laid out**, and where each of them stands in
/// the script: numbers of this file's own, so that a read of the first context cannot be answered by any
/// of the fifteen the enemy manager keeps behind it.
const A_CONTEXT_APART: u32 = 1_000;
const A_SCRIPT_STANDS_AT: u32 = 0x2000;

/// **The countdown a hit is taken over**, `[player+0xe2af4]`: three dwords, of which `+0x8` is the count
/// the life goes at `0x1e` of, `+0x4` a part of nothing and `+0x0` a phase the reset puts at `-0x3e7`.
///
/// This file's own literals, for the reason the module doc gives.
const PLAYER_HIT_COUNTDOWN: usize = 0xe2af4;
const A_COUNTDOWN_PHASE: usize = 0x0;
const A_COUNTDOWN_PART: usize = 0x4;
const A_COUNTDOWN_COUNT: usize = 0x8;
const A_COUNTDOWN_RESTARTS_AT: i32 = -0x3e7;

/// **The play area, as the game's own four floats have it**: 32 in, 16 down, 384 across and 448 down,
/// which the pause writes at 0x43a120 out of the constants at `0x4b42cc`, `0x4b42d4`, `0x4b4528` and
/// `0x4b4940`.
const PLAY_AREA: (f32, f32, f32, f32) = (32.0, 16.0, 384.0, 448.0);
/// And the four the player is held inside, in the area's own coordinates — the constants 0x43c6d2
/// through 0x43c6ff write, see `PLAYER_BOUNDS_LEFT`.
const PLAYER_BOUNDS: (f32, f32, f32, f32) = (8.0, 16.0, 368.0, 416.0);

/// **Where a stage puts the player**, which is the middle of the area across and 64 up from its bottom:
/// 0x44d745 halves the width into the first and 0x44d767 takes 64 off the height into the second.
const PLAYER_START: (f32, f32) = (192.0, 384.0);

/// **The player's two states orb names none of**: 1, coming in at the start a stage puts it at, which the
/// added callback writes (0x44d886) and a hit run out writes (0x44d012); and 3, untouchable and blinking
/// (0x44d3cf), which 0x44d252 writes once state 1's count has reached thirty. See `asm.md`'s *The player*.
const PLAYER_COMING_IN: u8 = 1;
const PLAYER_UNTOUCHABLE: u8 = 3;

/// **The scenes the supervisor's own update switches on**, off the cases at 0x44571c, 0x445753, 0x445821,
/// 0x4457eb, 0x445a1c, 0x445a61 and 0x445a94 — and the three more that are only ever *wanted*, which the
/// run manager writes to carry a run into its next stage (0x439ddc, 0x439e25).
///
/// This file's own literals, for the reason the module doc gives.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    /// What the game comes up in, and what asks for the front end at 0x44571c.
    Starting = 0,
    FrontEnd = 1,
    Run = 2,
    /// The run built again, which is how this game moves to its next stage.
    RunAgain = 3,
    /// The screen a run ends at, which a game over reaches: a launch read `scene=6->6` on the frame the
    /// third life went.
    RunOver = 6,
    /// The ending, whose own register is `0x4296a0` — the `EndingInf` allocation.
    Ending = 9,
}

/// **The screens of the front end**, as `Th08::front_end_screen` reads them out of `+0x14428`: the title
/// menu, the difficulty and the team select, which are the three a recipe of this game walks to a run,
/// and the list of replays the title's fifth item opens.
///
/// This file's own literals, read off launches of the real game: `front end: screen 0, cursor on item 0`
/// at the title, screen 4 after the first decide and screen 5 after the second, and `front end: screen
/// 7, cursor on item 4` the frame the title's item 4 was decided on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title = 0,
    Difficulty = 4,
    Team = 5,
    Replays = 7,
    /// **A stage practice's three**: the difficulty select's and the team select's handlers on the numbers
    /// the title's fourth item leads to, and the stage select the second of them puts up (0x46abac) — the
    /// handler at 0x46adb0.
    DifficultyOfAPractice = 8,
    TeamOfAPractice = 9,
    StageOfAPractice = 0xb,
    /// The Extra's own difficulty select, the difficulty select's handler on the screen number whose
    /// decide settles `cursor + 4` (0x46a48d) — which the title's second item puts up.
    DifficultyOfTheExtra = 0xc,
    /// The Extra's own team select, the team select's handler on the screen number whose decide makes the
    /// stage the Extra's — see `Th08`'s `SCREEN_TEAM_OF_THE_EXTRA`.
    TeamOfTheExtra = 0xd,
}

pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    /// In a host whose non-determinism is drawn from `seed` — the wake delays and the compositor's spikes
    /// — which an e2e test names in its assertions so that a failure can be replayed. The one
    /// constructor, for the reason th06's image says.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        let space = sim.space();
        for block in [
            &DATA,
            &QUAD_QUEUE,
            &RUN_NUMBERS,
            &FRONT_END,
            &RUN_SCREEN_INNER,
            &TEAM_SPEC,
            &ANM_SHEETS,
            &SOUND_STREAM,
            &SOUND_DEVICE,
            &MUSIC_SOURCE,
            &STAGE_INFO,
            &ENEMY_SCRIPTS,
            &BULLET_ANIMATIONS,
        ] {
            space.map(block.start, block.len(), Kind::Private);
        }
        space.write::<usize>(super::G_QUAD_QUEUE, QUAD_QUEUE.start);
        space.write::<usize>(
            super::G_RUN_SCREEN + super::RUN_SCREEN_INNER,
            RUN_SCREEN_INNER.start,
        );
        space.write::<usize>(super::G_TEAM_SPEC, TEAM_SPEC.start);
        // **The frame's own step, which is one where nothing is slowing the world down** — 0x425369
        // writes the 1.0, and a laid-out game with nothing there would say every laser grows by nothing.
        space.write::<f32>(super::G_FRAME_STEP, 1.0);
        let image = Self { sim };
        image.the_teams_spec_says_what_the_sht_says();
        image.the_play_area_is_the_games_own();
        image
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is
    /// what makes `Th08`'s reads land in it.
    pub fn enter(&self) -> orb_api::Installed {
        self.sim.enter()
    }

    pub fn space(&self) -> &Space {
        self.sim.space()
    }

    /// The device orb draws through and the window it asks about, where the game's own
    /// `IDirect3D8::CreateDevice` left them.
    ///
    /// A real pointer for the device, code being the one thing an address space cannot hold: orb reads
    /// the pointer out of here and then goes through its vtable, which is a vtable of Rust functions.
    pub fn shows_through(&self, device: Device, window: Hwnd) {
        let space = self.space();
        space.write::<usize>(super::G_D3D_DEVICE, device.0);
        space.write::<Hwnd>(super::G_GAME_WINDOW, window);
    }

    /// The object the game calls its own whole frame on, which orb's loop is handed.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// The chain the update and the draw are called on. Nothing is laid out at it — an e2e test's own two
    /// functions are what orb calls, and this is the argument they are called with — so what it has to be
    /// is an address inside the space, which is what a real one is.
    pub fn chain_object(&self) -> usize {
        super::G_CHAIN
    }

    /// **Jobs on that chain's calc list**, one element apiece and linked in the order given, each holding
    /// the callback and the object it would be called on — `looping` points the last one back at the first
    /// instead of ending the list, which is the shape a walk has to give up on rather than follow forever.
    ///
    /// **Hung off the chain's own element**, which is the list's first — its callback null and its next the
    /// first job, as the walk reads it: see `Th08`'s `CHAIN_CALC_LIST`.
    ///
    /// Laid out past the chain itself and not where the game's own heap puts them: what an element has to
    /// be for a walk to read it is an address inside the space, and nothing read of this game says where
    /// its allocator lays one.
    pub fn the_calc_list_holds(&self, jobs: &[(usize, usize)], looping: bool) {
        let space = self.space();
        let at = |which: usize| super::G_CHAIN + (which + 1) * A_JOBS_OWN_SPACE;
        let head = super::G_CHAIN + super::CHAIN_CALC_LIST;
        space.write::<usize>(head + super::CHAIN_JOB_CALLBACK, 0);
        space.write::<usize>(
            head + super::CHAIN_JOB_NEXT,
            match jobs.is_empty() {
                true => 0,
                false => at(0),
            },
        );
        for (which, (callback, on)) in jobs.iter().enumerate() {
            space.write::<usize>(at(which) + super::CHAIN_JOB_CALLBACK, *callback);
            space.write::<usize>(at(which) + super::CHAIN_JOB_ARGUMENT, *on);
            let next = match (which + 1 < jobs.len(), looping) {
                (true, _) => at(which + 1),
                (false, true) => at(0),
                (false, false) => 0,
            };
            space.write::<usize>(at(which) + super::CHAIN_JOB_NEXT, next);
        }
    }

    /// The game manager, which is a static of this game's own rather than an allocation: the object the
    /// run's own build is called on, and the one every field of a stage's clock is read out of.
    pub fn game_manager_object(&self) -> usize {
        super::G_GAME_MANAGER
    }

    /// And the front end's own object, which is what its callbacks are called on.
    pub fn front_end_object(&self) -> usize {
        FRONT_END.start
    }

    /// The whole-output viewport the game keeps in one struct and orb writes as it prepares a frame, for
    /// an e2e test that reads back what orb put there.
    pub fn viewport(&self) -> [u32; 6] {
        let space = self.space();
        [0, 1, 2, 3, 4, 5].map(|word| space.read::<u32>(super::VIEWPORT + word * 4))
    }

    /// **Whether the game is in a window**, which is the byte orb writes to overrule the display setting.
    pub fn is_windowed(&self) -> bool {
        self.space().read::<u8>(super::CFG_WINDOWED) != 0
    }

    /// And the setting put back to full screen, which is what a launch starts from before orb has written
    /// it: `Th08::force_windowed` is what an e2e test then reads the change of.
    pub fn asks_for_full_screen(&self) {
        self.space().write::<u8>(super::CFG_WINDOWED, 0);
    }

    /// **The four floats the rest of the game clips against**, which `sight` reads the field out of: see
    /// [`PLAY_AREA`].
    fn the_play_area_is_the_games_own(&self) {
        let space = self.space();
        let (left, top, width, height) = PLAY_AREA;
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_WIDTH - 8, left);
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_WIDTH - 4, top);
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_WIDTH, width);
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_HEIGHT, height);
        // And the four the player is held inside, which is what `sight` steers a bot within — see
        // `PLAYER_BOUNDS_LEFT`, and [`PLAYER_BOUNDS`] for the numbers.
        let (left, top, width, height) = PLAYER_BOUNDS;
        space.write::<f32>(super::G_GAME_MANAGER + super::PLAYER_BOUNDS_LEFT, left);
        space.write::<f32>(super::G_GAME_MANAGER + super::PLAYER_BOUNDS_TOP, top);
        space.write::<f32>(super::G_GAME_MANAGER + super::PLAYER_BOUNDS_WIDTH, width);
        space.write::<f32>(super::G_GAME_MANAGER + super::PLAYER_BOUNDS_HEIGHT, height);
    }

    /// **The active team member's numbers as its own `.sht` holds them**, which is where this game keeps
    /// the player's speeds and the power its shot stops at — see [`SPEEDS`] and [`FULL_POWER`].
    fn the_teams_spec_says_what_the_sht_says(&self) {
        let space = self.space();
        let (straight, diagonal, focused, focused_diagonal) = SPEEDS;
        space.write::<f32>(TEAM_SPEC.start + super::SPEED_STRAIGHT, straight);
        space.write::<f32>(TEAM_SPEC.start + super::SPEED_DIAGONAL, diagonal);
        space.write::<f32>(TEAM_SPEC.start + super::SPEED_FOCUSED_STRAIGHT, focused);
        space.write::<f32>(
            TEAM_SPEC.start + super::SPEED_FOCUSED_DIAGONAL,
            focused_diagonal,
        );
        space.write::<f32>(TEAM_SPEC.start + super::SPEC_POWER_FULL, FULL_POWER);
    }

    /// **The three things the game's own frame does to the queue of quads**, which a laid-out game has to
    /// be able to answer because code is the one thing an address space cannot hold.
    ///
    /// Each writes what the function it stands for writes — see [`QUEUED`] for the three addresses and
    /// what each of them does — less the `DrawPrimitiveUP` and the render states, which are a device's
    /// and not an address space's.
    pub fn empties_the_queue(&self) {
        let space = self.space();
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, 0);
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_WRITE_AT,
            QUAD_QUEUE.start + QUEUE_BUFFER,
        );
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_DRAWN_FROM,
            QUAD_QUEUE.start + QUEUE_BUFFER,
        );
    }

    /// A quad put in it, which is what the game's own drawing does between the two.
    pub fn queues_a_quad(&self) {
        let space = self.space();
        let at = space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT);
        space.write::<[u8; QUAD_BYTES]>(at, [0xcd; QUAD_BYTES]);
        space.write::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT, at + QUAD_BYTES);
        let queued = space.read::<u32>(QUAD_QUEUE.start + QUEUED);
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, queued + 1);
    }

    /// And the queue drawn, which leaves nothing in it and the drawn-from cursor where the write cursor
    /// is — **and does nothing at all where nothing is queued**, which is the gate at 0x462e4a.
    ///
    /// The write cursor is not moved back, which is the one thing only the empty does: a frame that drew
    /// twice draws the second batch from where the first ended.
    pub fn draws_the_queue(&self) {
        let space = self.space();
        if space.read::<u32>(QUAD_QUEUE.start + QUEUED) == 0 {
            return;
        }
        let at = space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT);
        space.write::<usize>(QUAD_QUEUE.start + QUEUE_DRAWN_FROM, at);
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, 0);
        let draws = space.read::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS);
        space.write::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS, draws + 1);
    }

    /// Hands over the three calls the game's own frame makes on that queue and on the fog, in place of
    /// the addresses [`Th08`](super::Th08) holds: the two above and the fog put out.
    ///
    /// All three together, because orb's frame makes all three: a game that handed over fewer would be
    /// jumping into memory nothing has mapped part way through one frame.
    pub fn hands_over_the_frames_own_calls(
        &self,
        empties_the_queue: usize,
        draws_the_queue: usize,
        puts_the_fog_out: usize,
    ) {
        super::set_empty_queue(empties_the_queue);
        super::set_draw_queue(draws_the_queue);
        super::set_fog_off(puts_the_fog_out);
    }

    /// And the run's sum, which orb takes again over the numbers a resume puts back — see
    /// [`Image::the_runs_checksum`].
    pub fn hands_over_the_runs_checksum(&self, sums_the_run: usize) {
        super::set_run_checksum(sums_the_run);
    }

    /// **The supervisor's update this laid-out game answers with**, which is what orb calls in place of the
    /// walk on a frame a build is going on — see `Th08::walks_a_frame_of_its_build`.
    pub fn hands_over_the_supervisors_update(&self, updates: usize) {
        super::set_supervisor_update(updates);
    }

    /// **The frames a run's build has stood**, which its last act waits for — see `BUILD_FRAMES`.
    pub fn the_build_has_stood_for(&self, frames: i32) {
        self.space()
            .write::<i32>(super::G_GAME_MANAGER + super::BUILD_FRAMES, frames);
    }

    pub fn how_long_the_build_has_stood(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_GAME_MANAGER + super::BUILD_FRAMES)
    }

    /// And how many of them it waits for, a recording's more — see `A_BUILD_STANDS_FOR`.
    pub fn what_the_build_stands_for(&self) -> i32 {
        match self.the_run_plays_a_recording() {
            true => super::A_RECORDINGS_BUILD_STANDS_FOR,
            false => super::A_BUILD_STANDS_FOR,
        }
    }

    /// How many quads are in the queue, and how many times it has been drawn — the two an e2e test over
    /// orb's own frame reads back.
    pub fn queued(&self) -> u32 {
        self.space().read::<u32>(QUAD_QUEUE.start + QUEUED)
    }

    pub fn queue_draws(&self) -> u32 {
        self.space().read::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS)
    }

    /// Where the next quad would go, for a test that wants to see the write pointer move.
    pub fn queue_writes_at(&self) -> usize {
        self.space()
            .read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT)
    }

    /// And where an emptied queue puts it, with how far one quad moves it — the two a test holds that
    /// pointer against. Both this file's own, for the reason the module doc gives.
    pub fn queue_buffer(&self) -> usize {
        QUAD_QUEUE.start + QUEUE_BUFFER
    }

    pub fn queue_bytes_per_quad(&self) -> usize {
        QUAD_BYTES
    }

    /// **The fog flag the game's own frame forces on between the two**, which orb writes as it begins the
    /// drawing — see `Th08::begin_drawing`, and `FOG_ON_FORCED` for the value.
    pub fn the_fog_flag(&self) -> u32 {
        self.space()
            .read::<u32>(super::G_SUPERVISOR + super::FOG_ON)
    }

    /// And that flag forced on, which the game's own frame does before it makes the call — see
    /// `FOG_ON_FORCED`, whose value this writes: a laid-out game's own frame has to write it where its
    /// frame writes it, the call doing nothing where the field already says the fog is out.
    pub fn forces_the_fog_call_through(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, super::FOG_ON_FORCED);
    }

    /// And the flag cleared, which is what that call does where it reaches the device.
    pub fn puts_the_fog_out(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, 0);
    }

    /// The scene the game is in and the one it wants, which is the pair `Th08::read_state` reads.
    pub fn is_in_scene(&self, scene: Scene, wanted: Scene) {
        let space = self.space();
        space.write::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_SCENE, scene as i32);
        space.write::<i32>(
            super::G_SUPERVISOR + super::SUPERVISOR_WANTED,
            wanted as i32,
        );
    }

    /// **Whether the supervisor's loader is running**, which is the word `Th08::a_stage_has_just_been_built`
    /// reads a build's end off — see `SUPERVISOR_LOADING`, and `Fake::follows_its_supervisor` for the
    /// frame it is set on and the one it is cleared on.
    pub fn is_loading(&self, loading: bool) {
        self.space().write::<i32>(
            super::G_SUPERVISOR + super::SUPERVISOR_LOADING,
            i32::from(loading),
        );
    }

    pub fn the_loader_is_running(&self) -> bool {
        self.space()
            .read::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_LOADING)
            != 0
    }

    pub fn the_scene_now(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_SCENE)
    }

    pub fn the_scene_wanted(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_WANTED)
    }

    /// **The scene the run itself asks for next**, which is `-1` while a run goes on: the field
    /// `Th08::leave_run` writes and the run manager's own hand-over reads — see `SCENE_THE_RUN_WANTS`.
    pub fn the_run_wants_no_scene(&self) {
        self.space()
            .write::<i32>(super::G_GAME_MANAGER + super::SCENE_THE_RUN_WANTS, -1);
    }

    pub fn the_scene_the_run_wants(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_GAME_MANAGER + super::SCENE_THE_RUN_WANTS)
    }

    /// **And the two bits of the run's kind that say it is ending**, which are the other half of what
    /// leaving a run writes — see `RUN_IS_ENDING`.
    pub fn the_run_is_ending(&self) -> u32 {
        (self.space().read::<u32>(super::G_RUN_KIND) & super::RUN_IS_ENDING) >> 5
    }

    /// And the bit of it a recording played carries, a replay's and the demo's — see `G_RUN_KIND`.
    pub fn the_run_plays_a_recording(&self) -> bool {
        self.space().read::<u32>(super::G_RUN_KIND) & super::RUN_READS_ITS_OWN_INPUT != 0
    }

    /// And the spell practice's, which is what the player's build and its coming in are shortened for
    /// (0x406c70, 0x44d28f).
    pub fn the_run_practises_a_card(&self) -> bool {
        self.space().read::<u32>(super::G_RUN_KIND) & super::RUN_IS_A_SPELL_PRACTICE != 0
    }

    /// And the one only the attract demo's own start writes.
    pub fn the_run_is_the_demos(&self) -> bool {
        self.space().read::<u32>(super::G_RUN_KIND) & super::RUN_IS_THE_DEMO != 0
    }

    /// **Whether a run is on**, which is the run's own scene or the run built again — the two
    /// `Th08::read_state` answers `in_run` for, read here without the rest of what that reads.
    pub fn a_run_is_on(&self) -> bool {
        let scene = self.the_scene_now();
        scene == super::SCENE_RUN || scene == super::SCENE_RUN_AGAIN
    }

    /// And the bit a stage's build loads its first track by a card for — see `G_STAGE_INFO`.
    pub fn the_run_loads_a_cards_track(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(super::G_RUN_KIND, kind | super::RUN_LOADS_A_CARDS_TRACK);
    }

    /// Which screen of the front end is up, and what its cursor is on — the two a recipe of this game
    /// presses at. The phase and the settled counter go in with them, those being what
    /// `Th08::menu_takes_a_press` reads.
    pub fn the_front_end_shows(&self, screen: Screen, cursor: i32) {
        let space = self.space();
        space.write::<usize>(super::G_FRONT_END, FRONT_END.start);
        space.write::<i32>(FRONT_END.start + super::FRONT_END_SCREEN, screen as i32);
        space.write::<i32>(FRONT_END.start + super::FRONT_END_CURSOR, cursor);
        space.write::<i32>(
            FRONT_END.start + super::FRONT_END_PHASE,
            super::TITLE_PHASE_ITEMS,
        );
        space.write::<i32>(
            FRONT_END.start + super::FRONT_END_SETTLED,
            super::SETTLED_FRAMES,
        );
    }

    /// **A screen of the front end on the update it comes up on**, as its handler's own entry finds it:
    /// the phase it builds itself in and a settled count of nothing — which the team select counts up
    /// through before it reads a press, see `Th08`'s `TEAM_PHASE_CHOOSING`.
    pub fn the_front_end_comes_up_on(&self, screen: Screen, cursor: i32) {
        self.the_front_end_shows(screen, cursor);
        self.the_front_end_is_at(0, 0);
    }

    /// **The cursor moved and nothing else**, which is what a direction does: the phase and the count go on as
    /// the screen had them.
    pub fn the_front_end_cursor_is(&self, cursor: i32) {
        if let Some(front) = self.the_front_end() {
            self.space()
                .write::<i32>(front + super::FRONT_END_CURSOR, cursor);
        }
    }

    /// The phase of the screen up and how long it has been settled, written — and read back below.
    pub fn the_front_end_is_at(&self, phase: i32, settled: i32) {
        let space = self.space();
        space.write::<i32>(FRONT_END.start + super::FRONT_END_PHASE, phase);
        space.write::<i32>(FRONT_END.start + super::FRONT_END_SETTLED, settled);
    }

    pub fn where_the_front_end_is(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(FRONT_END.start + super::FRONT_END_PHASE),
            space.read::<i32>(FRONT_END.start + super::FRONT_END_SETTLED),
        )
    }

    /// Which screen of it is up and which item its cursor is on, read back: what orb must leave exactly
    /// as it found it where a question of its own was cancelled, and what a laid-out game's own walk of
    /// those screens reads to know where it is.
    ///
    /// Nothing for a front end that is not there, its object being freed at a run's build.
    pub fn the_screen_the_front_end_is_on(&self) -> Option<i32> {
        self.the_front_end()
            .map(|front| self.space().read::<i32>(front + super::FRONT_END_SCREEN))
    }

    pub fn the_item_the_cursor_is_on(&self) -> Option<i32> {
        self.the_front_end()
            .map(|front| self.space().read::<i32>(front + super::FRONT_END_CURSOR))
    }

    fn the_front_end(&self) -> Option<usize> {
        Some(self.space().read::<usize>(super::G_FRONT_END)).filter(|front| *front != 0)
    }

    /// **The difficulty the screen before the team select settled**, which is a byte of the
    /// configuration rather than a field of the front end — see `CFG_DIFFICULTY`, and
    /// `Th08::run_pointed_at`, which reads it a screen before the run starts.
    pub fn the_difficulty_chosen_is(&self, difficulty: u8) {
        self.space().write::<u8>(super::CFG_DIFFICULTY, difficulty);
    }

    /// And read back, which is what the team select's decide copies into the run's own.
    pub fn the_difficulty_chosen(&self) -> i32 {
        i32::from(self.space().read::<u8>(super::CFG_DIFFICULTY))
    }

    /// **And the front end freed, which is what a run is — with its pointer left naming the object**,
    /// because that is what the real game leaves: `G_FRONT_END` is written once in the whole exe and
    /// never cleared, so a run's build frees the allocation and the static goes on pointing at it.
    ///
    /// **Which is the one thing a zero here would hide.** A laid-out game that cleared the pointer is one
    /// where `Th08::front_end`'s old filter on nothing answered correctly, and the real game read
    /// `front end: screen 1900876895` off the freed object on every frame of a stage. So the fields are
    /// filled with something that is not a screen this game has, and what has to decline is the scene.
    pub fn the_front_end_is_freed(&self) {
        let space = self.space();
        space.write::<usize>(super::G_FRONT_END, FRONT_END.start);
        space.write::<i32>(
            FRONT_END.start + super::FRONT_END_SCREEN,
            WHAT_A_FREED_FRONT_END_READS,
        );
        space.write::<i32>(
            FRONT_END.start + super::FRONT_END_CURSOR,
            WHAT_A_FREED_FRONT_END_READS,
        );
    }

    /// Where that pointer still points, for a test that says orb declined the screen without the pointer
    /// having gone.
    pub fn the_front_end_is_still_pointed_at(&self) -> bool {
        self.space().read::<usize>(super::G_FRONT_END) == FRONT_END.start
    }

    /// **A run's own numbers in place**, which is the pointer the build allocates — see `G_RUN_NUMBERS`.
    pub fn has_a_runs_numbers(&self) {
        self.space()
            .write::<usize>(super::G_RUN_NUMBERS, RUN_NUMBERS.start);
    }

    /// **The stage's time orbs picked up and the norm it wants**, the pair the panel draws and the Last
    /// Spell is decided against — see `Th08`'s `TIME_ORBS`.
    pub fn the_stage_has_its_time_orbs(&self, orbs: u32, norm: u32) {
        let space = self.space();
        space.write::<u32>(RUN_NUMBERS.start + super::TIME_ORBS, orbs);
        space.write::<u32>(RUN_NUMBERS.start + super::TIME_ORBS_WANTED, norm);
    }

    /// **Where the human–youkai gauge is, and the line its human end is at** — see `Th08`'s `GAUGE` and
    /// `GAUGE_HUMAN_END`, the gauge a word of the run's numbers and the line one of the game manager's.
    /// Resealed, the gauge being among the numbers the seal is over.
    pub fn the_gauge_is_at(&self, gauge: i16, human_end: i16) {
        let space = self.space();
        space.write::<i16>(RUN_NUMBERS.start + super::GAUGE, gauge);
        space.write::<i16>(super::G_GAME_MANAGER + super::GAUGE_HUMAN_END, human_end);
        self.reseals_the_run();
    }

    /// And the pointer left naming what the next build freed, which is the state one launch read
    /// `lives=-128 bombs=-128` at the title through.
    pub fn its_runs_numbers_are_freed(&self) {
        self.space().write::<usize>(super::G_RUN_NUMBERS, 0);
    }

    pub fn run_numbers_at(&self) -> usize {
        RUN_NUMBERS.start
    }

    /// The stage and the difficulty, as a run in progress has them: the stage counted from zero the way
    /// this game counts it.
    pub fn is_playing(&self, stage: i32, difficulty: i32) {
        let space = self.space();
        space.write::<i32>(super::G_STAGE, stage);
        space.write::<i32>(super::G_GAME_MANAGER + super::DIFFICULTY, difficulty);
    }

    pub fn the_games_stage_word(&self) -> i32 {
        self.space().read::<i32>(super::G_STAGE)
    }

    /// How long the run has been played, which is the clock this game steps as a stage carries on — see
    /// `Th08::read_state`.
    pub fn has_been_in_the_run_for(&self, frames: u32) {
        self.space()
            .write::<u32>(super::G_GAME_MANAGER + super::STAGE_FRAMES, frames);
    }

    /// The run's own clock as the game keeps it, which `Th08::read_state` answers a stage's frames as a
    /// subtraction from: a stage that steps it reads it here and not off the state.
    pub fn the_frames_the_run_has_been_going(&self) -> u32 {
        self.space()
            .read::<u32>(super::G_GAME_MANAGER + super::STAGE_FRAMES)
    }

    /// **How far the stage's script has run**, written into the first of the contexts the enemy manager
    /// keeps — and the rest of them left standing somewhere of their own, so that a read of the first
    /// cannot pass by reading any other.
    pub fn the_stage_script_has_run_for(&self, frames: u32) {
        let space = self.space();
        for context in 0..SCRIPT_CONTEXTS_KEPT {
            let at = super::G_ENEMY_MANAGER + SCRIPT_CONTEXTS + context * SCRIPT_CONTEXT_BYTES;
            let count = if context == 0 {
                frames
            } else {
                frames + A_CONTEXT_APART * context as u32
            };
            space.write::<u32>(at + A_COUNTDOWN_COUNT, count);
            space.write::<u32>(at + SCRIPT_CONTEXT_AT, A_SCRIPT_STANDS_AT + context as u32);
        }
    }

    /// The generator's own word, the copy of it the game manager keeps, and the count of draws — the four
    /// numbers a seed of this game is, less one: see `Th08::seed`.
    pub fn the_stage_began_with_the_seed(&self, seed: u16) {
        let space = self.space();
        space.write::<u16>(super::G_RANDOM, seed);
        space.write::<u16>(super::G_RANDOM + super::GENERATOR_KEPT, seed);
        space.write::<u32>(super::G_RANDOM + super::GENERATOR_DRAWS, 0);
        space.write::<u16>(super::G_GAME_MANAGER + super::SEED_AT_STAGE_START, seed);
    }

    pub fn the_generators_word(&self) -> u16 {
        self.space().read::<u16>(super::G_RANDOM)
    }

    /// **The generator drawn from**, which is what every frame of a run does to it: the word alone, where
    /// [`the_stage_began_with_the_seed`](Image::the_stage_began_with_the_seed) writes the kept copy and
    /// the count of draws with it. A stage whose word never moved is one orb reads as a frame nothing
    /// happened on — see `Th08::generator`.
    pub fn the_generator_has_drawn(&self, word: u16) {
        self.space().write::<u16>(super::G_RANDOM, word);
    }

    pub fn the_seed_the_stage_began_with(&self) -> u16 {
        self.space()
            .read::<u16>(super::G_GAME_MANAGER + super::SEED_AT_STAGE_START)
    }

    /// The numbers the panel shows, which this game keeps as floats behind the run's own pointer.
    /// **Resealed, the way the game's own setters for the lives, the bombs and the power are** — see
    /// [`Image::reseals_the_run`].
    pub fn has_lives_and_bombs(&self, lives: f32, bombs: f32) {
        let space = self.space();
        space.write::<f32>(RUN_NUMBERS.start + super::LIVES, lives);
        space.write::<f32>(RUN_NUMBERS.start + super::BOMBS, bombs);
        self.reseals_the_run();
    }

    pub fn shot_is_at_power(&self, power: f32) {
        self.space()
            .write::<f32>(RUN_NUMBERS.start + super::POWER, power);
        self.reseals_the_run();
    }

    /// And the deaths with them, which the game counts where a hit becomes a death (0x44e260) and follows with
    /// the life taken through the lives' own setter on the next update (0x43c641).
    pub fn has_died(&self, deaths: f32) {
        self.space()
            .write::<f32>(RUN_NUMBERS.start + super::DEATHS, deaths);
        self.reseals_the_run();
    }

    /// **The run's numbers sealed**, which is 0x406e50 — see `Th08`'s `RUN_SEAL_KEY`: two keys, the second's
    /// copy over the seal, the sum over the numbers as they stand and the float the run manager holds it
    /// against.
    ///
    /// **Keys of this file's own and not two draws of the generator**, which is where the game takes them:
    /// nothing orb does reads either, and a laid-out run whose generator moved with every setter would draw
    /// differently in every test that places a number.
    fn reseals_the_run(&self) {
        let space = self.space();
        let run = RUN_NUMBERS.start;
        space.write::<u32>(run + super::RUN_SEAL_KEY, THE_SEALS_KEY);
        space.write::<u32>(run + RUN_SEAL_SALT, THE_SEALS_SALT);
        space.write::<u32>(run + super::RUN_SEAL, THE_SEALS_KEY);
        let sum = self.the_runs_checksum();
        space.write::<u32>(run + super::RUN_SEALED_SUM, sum);
        space.write::<f32>(
            super::G_SUM_THIS_FRAME,
            sum.wrapping_add(THE_SEALS_SALT) as i32 as f32,
        );
    }

    /// **The run's sum**, which is 0x439a2e — see `Th08`'s `RUN_CHECKSUM`: every byte of its five blocks
    /// added up, and the run's own step added to its seal for each. A block `[gm+0x4]` does not name is
    /// counted as that many bytes of nothing, the seal stepped for them all the same.
    pub fn the_runs_checksum(&self) -> u32 {
        let space = self.space();
        let run = RUN_NUMBERS.start;
        let named = space.read::<usize>(super::G_GAME_MANAGER + GM_BLOCK_THE_SUM_TAKES);
        let blocks = [
            (run + 0x48, 0x80),
            (run + 0xd0, 0x14),
            (named, 0x3c),
            (CONFIGURATION, 0x3c),
            (super::G_GAME_MANAGER + 0x3da2c, 0x168),
        ];
        let step = space.read::<u32>(run + RUN_SEAL_STEP);
        let mut sum = 0u32;
        let mut seal = space.read::<u32>(run + super::RUN_SEAL);
        for (at, bytes) in blocks {
            for i in 0..bytes {
                if at != 0 {
                    sum = sum.wrapping_add(u32::from(space.read::<u8>(at + i)));
                }
                seal = seal.wrapping_add(step);
            }
        }
        space.write::<u32>(run + super::RUN_SEAL, seal);
        sum
    }

    /// **The sum the run manager takes every frame** (0x43a4aa through 0x43a4d7): the key over the seal, and
    /// the sum plus the salt into the float the check holds the kept one against.
    pub fn the_run_manager_takes_its_sum(&self) {
        let space = self.space();
        let run = RUN_NUMBERS.start;
        space.write::<u32>(
            run + super::RUN_SEAL,
            space.read::<u32>(run + super::RUN_SEAL_KEY),
        );
        let sum = self.the_runs_checksum();
        space.write::<f32>(
            super::G_SUM_THIS_FRAME,
            sum.wrapping_add(space.read::<u32>(run + RUN_SEAL_SALT)) as i32 as f32,
        );
    }

    /// **And the check the enemy manager makes of it** (0x40bb80): the seal against the key and the step, and
    /// the kept sum and the salt against the float.
    pub fn the_run_is_as_it_was_sealed(&self) -> bool {
        let space = self.space();
        let run = RUN_NUMBERS.start;
        let seal = space.read::<u32>(run + super::RUN_SEAL);
        let wanted = space
            .read::<u32>(run + RUN_SEAL_STEP)
            .wrapping_mul(0x274)
            .wrapping_add(space.read::<u32>(run + super::RUN_SEAL_KEY));
        let kept = space
            .read::<u32>(run + super::RUN_SEALED_SUM)
            .wrapping_add(space.read::<u32>(run + RUN_SEAL_SALT)) as i32;
        seal == wanted && kept == space.read::<f32>(super::G_SUM_THIS_FRAME) as i32
    }

    /// The score and the point items, which the resume carries and the panel draws — **the score in both of its
    /// words**, the panel's copy having caught up with it, which is what a stage's build leaves the copy as
    /// (0x43b14a) and what orb reads a stage's score out of.
    pub fn the_run_has_scored(&self, score: u32, point_items: u32) {
        let space = self.space();
        space.write::<u32>(RUN_NUMBERS.start + super::SCORE, score);
        space.write::<u32>(RUN_NUMBERS.start + super::SCORE_SHOWN, score);
        space.write::<u32>(RUN_NUMBERS.start + super::POINT_ITEMS, point_items);
    }

    /// **The score and the copy the panel draws, apart** — see `Th08`'s `SCORE` — and read back as
    /// `(score, shown)`.
    pub fn the_score_and_the_panels_copy_are(&self, score: u32, shown: u32) {
        let space = self.space();
        space.write::<u32>(RUN_NUMBERS.start + super::SCORE, score);
        space.write::<u32>(RUN_NUMBERS.start + super::SCORE_SHOWN, shown);
    }

    /// The point items collected, as the run's own numbers hold them.
    pub fn the_point_items(&self) -> u32 {
        self.space()
            .read::<u32>(RUN_NUMBERS.start + super::POINT_ITEMS)
    }

    pub fn the_score_and_the_panels_copy(&self) -> (u32, u32) {
        let space = self.space();
        (
            space.read::<u32>(RUN_NUMBERS.start + super::SCORE),
            space.read::<u32>(RUN_NUMBERS.start + super::SCORE_SHOWN),
        )
    }

    /// **What a build of a stage carrying a run on does to the score**: the panel's copy takes the score
    /// (0x43b14a) and the score goes to nought (0x43b808) — the first only on that path, 0x43ac89 through
    /// 0x43acb8 sending a run's own start, a practice and the Extra past it, and the second on every build.
    pub fn a_carried_builds_hands_the_score_to_the_panel(&self) {
        let (score, _) = self.the_score_and_the_panels_copy();
        self.the_score_and_the_panels_copy_are(0, score);
    }

    /// **And the run manager's floor under the score**, on every update it runs past its pause: a score below
    /// the panel's copy is put up to it (0x43a6e9 through 0x43a713). Which is what gives a stage carried on
    /// its score back on its first update, the build having zeroed it.
    pub fn the_run_manager_holds_the_score_at_the_panels_copy(&self) {
        let (score, shown) = self.the_score_and_the_panels_copy();
        if score < shown {
            self.the_score_and_the_panels_copy_are(shown, shown);
        }
    }

    /// **Which kind of run it is**, as the word of flags has it: nothing for a run somebody is playing,
    /// and `RUN_READS_ITS_OWN_INPUT` alone for a replay played back off the game's own list, which is the
    /// one bit that list's decide writes (0x46ed24).
    pub fn the_run_is_its_own(&self) {
        self.space().write::<u32>(super::G_RUN_KIND, 0);
    }

    pub fn the_run_reads_its_own_input(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(super::G_RUN_KIND, kind | super::RUN_READS_ITS_OWN_INPUT);
    }

    /// **And the bit a stage practice is played under**, which the title's fourth item and the stage select
    /// both set (0x467dde, 0x46af8e) — with the spell practice's cleared, which that item clears.
    pub fn the_run_is_a_practice(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(
            super::G_RUN_KIND,
            (kind | super::RUN_STARTS_WITH_EIGHT) & !super::RUN_IS_A_SPELL_PRACTICE,
        );
    }

    /// And the one bit every start screen clears as it asks for a run, a recording played — 0x46b0be for the
    /// stage select's.
    pub fn the_run_plays_no_recording(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(super::G_RUN_KIND, kind & !super::RUN_READS_ITS_OWN_INPUT);
    }

    /// And the two bits the attract demo's own start writes, a recording played and the demo — 0x467b65
    /// through 0x467b8e.
    pub fn the_run_is_the_demo(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(
            super::G_RUN_KIND,
            kind | super::RUN_READS_ITS_OWN_INPUT | super::RUN_IS_THE_DEMO,
        );
    }

    /// The scene asked for, with the scene now left as it stands — what a screen's own start writes.
    pub fn asks_for_the_scene(&self, wanted: Scene) {
        self.space().write::<i32>(
            super::G_SUPERVISOR + super::SUPERVISOR_WANTED,
            wanted as i32,
        );
    }

    /// **The game's own pause**, the byte its run manager and its menu write — see `Th08`'s `G_PAUSED`.
    pub fn the_run_is_paused(&self, paused: bool) {
        self.space().write::<u8>(super::G_PAUSED, u8::from(paused));
    }

    pub fn is_paused(&self) -> bool {
        self.space().read::<u8>(super::G_PAUSED) != 0
    }

    /// **The screen a run ends at, up in `phase` with its timer at nothing**, and its update on the chain's
    /// calc list with that screen as the object it is called on — which is the one way orb reaches it, the
    /// real screen's address being kept nowhere but the job: see `Th08`'s `RESULT_UPDATE`.
    pub fn the_screen_a_run_ends_at_is_up_in(&self, phase: i32) {
        let space = self.space();
        if space.has_room(RESULT_SCREEN.start, RESULT_SCREEN.len()) {
            space.map(RESULT_SCREEN.start, RESULT_SCREEN.len(), Kind::Private);
        }
        self.the_screen_a_run_ends_at_is_at(phase, 0);
        self.the_calc_list_holds(&[(super::RESULT_UPDATE, RESULT_SCREEN.start)], false);
    }

    /// And its update off the chain, which is what that update answering nought does, and the screen
    /// freed where it is still laid out — its teardown's.
    pub fn the_screen_a_run_ends_at_is_off_the_chain(&self) {
        self.the_calc_list_holds(&[], false);
        let space = self.space();
        if !space.has_room(RESULT_SCREEN.start, RESULT_SCREEN.len()) {
            space.unmap(RESULT_SCREEN.start);
        }
    }

    /// Whether that update is the chain's, which is whether the screen is still running.
    pub fn the_screen_a_run_ends_at_is_on_the_chain(&self) -> bool {
        let space = self.space();
        let first =
            space.read::<usize>(super::G_CHAIN + super::CHAIN_CALC_LIST + super::CHAIN_JOB_NEXT);
        first != 0 && space.read::<usize>(first + super::CHAIN_JOB_CALLBACK) == super::RESULT_UPDATE
    }

    /// Its phase and its timer, written — what the screen's own handlers write as they move it on.
    pub fn the_screen_a_run_ends_at_is_at(&self, phase: i32, timer: i32) {
        let space = self.space();
        space.write::<i32>(RESULT_SCREEN.start + super::RESULT_PHASE, phase);
        space.write::<i32>(RESULT_SCREEN.start + super::RESULT_TIMER, timer);
    }

    /// And read, as `(phase, timer)`.
    pub fn where_the_screen_a_run_ends_at_is(&self) -> (i32, i32) {
        let space = self.space();
        (
            space.read::<i32>(RESULT_SCREEN.start + super::RESULT_PHASE),
            space.read::<i32>(RESULT_SCREEN.start + super::RESULT_TIMER),
        )
    }

    /// **The name entry's field, how many of it have been typed and the item its cursor is on** — the three
    /// the entry's own keys move, see `Th08`'s `RESULT_TYPING_A_NAME`. The count's word is the offer's
    /// cursor too, the one field serving both.
    pub fn the_name_entry_holds(
        &self,
        name: [u8; super::RESULT_NAME_BYTES],
        typed: i32,
        cursor: i32,
    ) {
        let space = self.space();
        space.write_bytes(RESULT_SCREEN.start + super::RESULT_NAME, &name);
        space.write::<i32>(RESULT_SCREEN.start + super::RESULT_TYPED, typed);
        space.write::<i32>(RESULT_SCREEN.start + super::RESULT_CURSOR, cursor);
    }

    /// And read back, as `(name, typed, cursor)`.
    pub fn what_the_name_entry_holds(&self) -> ([u8; super::RESULT_NAME_BYTES], i32, i32) {
        let space = self.space();
        let mut name = [0; super::RESULT_NAME_BYTES];
        name.copy_from_slice(&space.read_bytes(
            RESULT_SCREEN.start + super::RESULT_NAME,
            super::RESULT_NAME_BYTES,
        ));
        (
            name,
            space.read::<i32>(RESULT_SCREEN.start + super::RESULT_TYPED),
            space.read::<i32>(RESULT_SCREEN.start + super::RESULT_CURSOR),
        )
    }

    /// **The player playing**, which is its state byte at nothing and its position at the start a stage
    /// puts it at — with the box the game's own tests read made out of its half, which is what
    /// `sight::player_box` reads back.
    pub fn the_player_is_playing(&self) {
        let space = self.space();
        space.write::<u8>(super::G_PLAYER + super::PLAYER_STATE, super::PLAYER_NORMAL);
        self.the_player_is_at(PLAYER_START);
    }

    /// And the player hit, which is the thirty-frame window a deathbomb is spent in — with its count from
    /// nothing, which is the hit handler's own 0x4065f0 with a 0 beside the state it writes at 0x44ab84.
    pub fn the_player_has_been_hit(&self) {
        self.space()
            .write::<u8>(super::G_PLAYER + super::PLAYER_STATE, super::PLAYER_HIT);
        self.the_hit_has_counted(0);
    }

    /// **The player coming in at the start a stage puts it at**, state 1, with `count` under it: what a
    /// stage's build leaves (0x44d886, with 120) and what a hit run out leaves (0x44d012, with nothing).
    pub fn the_player_comes_in(&self, count: i32) {
        self.space()
            .write::<u8>(super::G_PLAYER + super::PLAYER_STATE, PLAYER_COMING_IN);
        self.the_player_is_at(PLAYER_START);
        self.the_hit_has_counted(count);
    }

    /// **And the player back to playing where it stands**, state 0 with its count at nothing — what the last
    /// update of state 3 writes (0x44d39f, 0x44d3ad), which moves it nowhere.
    pub fn the_player_is_playing_where_it_stands(&self) {
        self.space()
            .write::<u8>(super::G_PLAYER + super::PLAYER_STATE, super::PLAYER_NORMAL);
        self.the_hit_has_counted(0);
    }

    /// **And the player untouchable, blinking**, state 3, with `count` updates of it left — see
    /// `PLAYER_UNTOUCHABLE`.
    pub fn the_player_is_untouchable_for(&self, count: i32) {
        self.space()
            .write::<u8>(super::G_PLAYER + super::PLAYER_STATE, PLAYER_UNTOUCHABLE);
        self.the_hit_has_counted(count);
    }

    /// **And that hit some frames into its countdown**, which is the number the life goes at `0x1e` of:
    /// `[player+0xe2af4+0x8]`, the count `0x40d3b0` answers.
    ///
    /// The three dwords of it written the way the game's own reset writes them, less the count — the
    /// phase at `-0x3e7` and the part at nothing.
    pub fn the_hit_has_counted(&self, frames: i32) {
        let space = self.space();
        let countdown = super::G_PLAYER + PLAYER_HIT_COUNTDOWN;
        space.write::<i32>(countdown + A_COUNTDOWN_COUNT, frames);
        space.write::<i32>(countdown + A_COUNTDOWN_PART, 0);
        space.write::<i32>(countdown + A_COUNTDOWN_PHASE, A_COUNTDOWN_RESTARTS_AT);
    }

    /// What the player is doing and how far its countdown has got — the two an e2e test over
    /// `Th08::make_invulnerable` reads back.
    pub fn what_the_player_is_doing(&self) -> u8 {
        self.space()
            .read::<u8>(super::G_PLAYER + super::PLAYER_STATE)
    }

    /// **A card of this game held where it stands**, which is the byte its own update stops on — see
    /// `Th08::CARD_UPDATE_HELD`, whose only writer that clears it is the handler a hit goes through.
    pub fn a_card_is_held(&self) {
        self.space()
            .write::<u8>(super::G_GAME_MANAGER + super::CARD_UPDATE_HELD, A_CARD_HELD);
    }

    /// And that byte read back, which is what says an unhittable player was handed the hit handler's own
    /// first act as well as its last.
    pub fn the_card_is_held(&self) -> u8 {
        self.space()
            .read::<u8>(super::G_GAME_MANAGER + super::CARD_UPDATE_HELD)
    }

    pub fn the_frames_the_hit_has_counted(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_PLAYER + PLAYER_HIT_COUNTDOWN + A_COUNTDOWN_COUNT)
    }

    /// Where it is, and the box the game builds out of its own half around it — `position ± half`, which
    /// is what 0x44bc02 and 0x44bc37 write every frame.
    pub fn the_player_is_at(&self, (x, y): (f32, f32)) {
        let space = self.space();
        let half = PLAYER_BOX_ACROSS / 2.0;
        space.write::<f32>(super::G_PLAYER + super::PLAYER_POSITION, x);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_POSITION + 4, y);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_HALF, half);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_HALF + 4, half);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_BOX_NEAR, x - half);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_BOX_NEAR + 4, y - half);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_BOX_FAR, x + half);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_BOX_FAR + 4, y + half);
    }

    pub fn where_the_player_is(&self) -> (f32, f32) {
        let space = self.space();
        (
            space.read::<f32>(super::G_PLAYER + super::PLAYER_POSITION),
            space.read::<f32>(super::G_PLAYER + super::PLAYER_POSITION + 4),
        )
    }

    /// **A line of dialogue up**, which is what stops this game's stage clock: the numbered values are a
    /// line with words on it and `-2` the frame the screen is handed back on.
    pub fn a_line_of_dialogue_is_up(&self, line: i32) {
        self.space()
            .write::<i32>(RUN_SCREEN_INNER.start + super::DIALOGUE_LINE, line);
    }

    /// And no conversation at all, which is `-1`: 0x4358bb answers no for anything below nothing that is
    /// not `-2`.
    pub fn no_dialogue_is_up(&self) {
        self.space()
            .write::<i32>(RUN_SCREEN_INNER.start + super::DIALOGUE_LINE, -1);
    }

    /// **A boss up, and the enemy the fight is with**: the run screen's own byte and the bit on that
    /// enemy's flags, which are the two `Th08::the_boss` asks to agree — and the enemy in the first of
    /// `BOSSES`, where the instruction that sets the bit puts it (0x41c4b9).
    pub fn a_boss_is_fighting(&self, slot: usize, at: (f32, f32), life: i32) {
        let space = self.space();
        space.write::<u8>(super::G_RUN_SCREEN + super::BOSS_IS_UP, 1);
        let enemy = self.enemy_at(slot);
        space.write::<usize>(super::G_ENEMY_MANAGER + super::BOSSES, enemy);
        space.write::<u32>(
            enemy + super::ENEMY_ALIVE,
            1 | super::ENEMY_IS_THE_BOSS | super::ENEMY_SOLID | super::ENEMY_TAKES_THE_SHOT,
        );
        space.write::<f32>(enemy + super::ENEMY_POSITION, at.0);
        space.write::<f32>(enemy + super::ENEMY_POSITION + 4, at.1);
        space.write::<i32>(enemy + super::ENEMY_LIFE, life);
    }

    /// **And how long that boss's current attack has run**, the count at `ENEMY_ATTACK_FRAMES` — which the
    /// laid-out game's own update then steps with every enemy's.
    pub fn the_boss_has_attacked_for(&self, slot: usize, frames: i32) {
        self.space()
            .write::<i32>(self.enemy_at(slot) + super::ENEMY_ATTACK_FRAMES, frames);
    }

    /// And none, with her entry of `BOSSES` back to nought as the instruction taking her down leaves it
    /// (0x41c5b6).
    pub fn no_boss_is_fighting(&self) {
        let space = self.space();
        space.write::<u8>(super::G_RUN_SCREEN + super::BOSS_IS_UP, 0);
        space.write::<usize>(super::G_ENEMY_MANAGER + super::BOSSES, 0);
    }

    /// Whether a boss is up, as the run screen's byte says.
    pub fn a_boss_is_up(&self) -> bool {
        self.space()
            .read::<u8>(super::G_RUN_SCREEN + super::BOSS_IS_UP)
            != 0
    }

    /// **That many enemies alive**, each solid, each taking the player's shots and each somewhere of its own
    /// inside the field, with the game's own count of them beside — which is the number `Th08::hazards_near`
    /// holds its own walk against.
    pub fn enemies_alive(&self, enemies: usize) {
        let space = self.space();
        for slot in 0..super::ENEMY_SLOTS {
            let enemy = self.enemy_at(slot);
            if slot >= enemies {
                space.write::<u32>(enemy + super::ENEMY_ALIVE, 0);
                continue;
            }
            space.write::<u32>(
                enemy + super::ENEMY_ALIVE,
                1 | super::ENEMY_SOLID | super::ENEMY_TAKES_THE_SHOT,
            );
            space.write::<f32>(enemy + super::ENEMY_POSITION, 32.0 + slot as f32);
            space.write::<f32>(enemy + super::ENEMY_POSITION + 4, 48.0 + slot as f32);
            space.write::<f32>(enemy + super::ENEMY_SIZE, 24.0);
            space.write::<f32>(enemy + super::ENEMY_SIZE + 4, 24.0);
        }
        space.write::<i32>(super::G_ENEMY_MANAGER + super::ENEMY_COUNT, enemies as i32);
    }

    /// And where one of them is.
    pub fn an_enemy_is_at(&self, slot: usize, at: (f32, f32)) {
        let space = self.space();
        let enemy = self.enemy_at(slot);
        space.write::<f32>(enemy + super::ENEMY_POSITION, at.0);
        space.write::<f32>(enemy + super::ENEMY_POSITION + 4, at.1);
    }

    /// And one of them a familiar — see `Th08`'s `ENEMY_FAMILIAR`, which asks only that the word is not nothing.
    pub fn an_enemy_is_a_familiar(&self, slot: usize) {
        self.space()
            .write::<u32>(self.enemy_at(slot) + super::ENEMY_FAMILIAR, 1);
    }

    /// And one of them with no body to run into, which is `ENEMY_SOLID` clear on a slot still alive.
    pub fn an_enemy_has_no_body(&self, slot: usize) {
        self.space()
            .write::<u32>(self.enemy_at(slot) + super::ENEMY_ALIVE, 1);
    }

    /// **And one of them the game's own walk tests nothing of**, with the very flags a debugger read off the
    /// one stage 1 of a replay kept alive through its midsection: `0x1000005d`, solid and shootable by
    /// their own bits and out of both tests by bit 4.
    pub fn an_enemy_is_passed_over_by_every_test(&self, slot: usize) {
        self.space()
            .write::<u32>(self.enemy_at(slot) + super::ENEMY_ALIVE, 0x1000_005d);
    }

    /// **And one of them new in a slot another enemy had**, which is that slot somewhere else with the count
    /// at `ENEMY_FRAMES` starting again from nothing — see `Hazard::frames`. `蘇活「生命遊戯」` is a card
    /// whose cells are born and killed while it runs, so its slots hold a different enemy every few frames.
    pub fn an_enemy_is_new_in_its_slot(&self, slot: usize, at: (f32, f32)) {
        self.an_enemy_is_at(slot, at);
        self.space()
            .write::<i32>(self.enemy_at(slot) + super::ENEMY_FRAMES, 0);
    }

    /// **Every live enemy one frame older**, which is what this game's own update does to the timer each of
    /// them carries at `ENEMY_FRAMES`: the count is what says the enemy in a slot next frame is the one that
    /// was there, and a laid-out game whose enemies never aged would have every slot read as given away.
    pub fn its_enemies_are_a_frame_older(&self) {
        let len = super::ENEMY_SLOTS * super::ENEMY_BYTES;
        self.space().edit_bytes(self.enemy_at(0), len, |enemies| {
            for enemy in enemies.as_chunks_mut::<{ super::ENEMY_BYTES }>().0 {
                if word_in(enemy, super::ENEMY_ALIVE) & 1 == 0 {
                    continue;
                }
                let frames = word_in(enemy, super::ENEMY_FRAMES) as i32;
                put_word_in(enemy, super::ENEMY_FRAMES, (frames + 1) as u32);
            }
        });
    }

    /// **And one of them firing a beam**, which is its script context holding one of the three behaviours
    /// — see `ENEMY_SCRIPT`: the context laid out in [`ENEMY_SCRIPTS`], the behaviour at its `+0x10`, the
    /// anchor the beam is laid from at its `+0x38` as nothing, and the angle on the enemy itself.
    pub fn an_enemy_fires_a_beam(&self, slot: usize, at: (f32, f32), angle: f32, which: usize) {
        self.an_enemy_aims_a_beam(slot, at, angle);
        let context = ENEMY_SCRIPTS.start + slot * ENEMY_SCRIPT_BYTES;
        self.space().write::<usize>(
            context + super::SCRIPT_BEHAVIOUR,
            super::BEAM_BEHAVIOURS[which].0,
        );
    }

    /// **And one firing a beam from an anchor of the script's own choosing**, which is a vector nothing makes
    /// the one [`Image::an_enemy_aims_a_beam`] writes: the behaviour lays its box from `position - anchor`
    /// along the angle whatever the anchor is (0x42474a through 0x424770).
    pub fn an_enemy_fires_a_beam_anchored_at(
        &self,
        slot: usize,
        at: (f32, f32),
        angle: f32,
        anchor: (f32, f32),
        which: usize,
    ) {
        self.an_enemy_fires_a_beam(slot, at, angle, which);
        let context = ENEMY_SCRIPTS.start + slot * ENEMY_SCRIPT_BYTES;
        let space = self.space();
        space.write::<f32>(context + super::SCRIPT_BEAM_ANCHOR, anchor.0);
        space.write::<f32>(context + super::SCRIPT_BEAM_ANCHOR + 4, anchor.1);
    }

    /// **And a beam of one aimed and not yet firing**, which is every frame of one but the frames it is
    /// testing the player: the anchor and the angle in place with the behaviour still nought — see
    /// `BEAM_ANCHOR_ALONG`, which is the launch that measured the real game doing exactly this for
    /// seventeen seconds before it took a life.
    ///
    /// **The anchor is the real game's own**, `BEAM_LENGTH / 2` along the angle, so the enemy sits at the
    /// middle of the box: written as nought by the laid-out game, an enemy here was a beam whose box began
    /// where the real one's middle is, and orb had nothing to tell a beam from any other enemy by.
    pub fn an_enemy_aims_a_beam(&self, slot: usize, at: (f32, f32), angle: f32) {
        let space = self.space();
        let enemy = self.enemy_at(slot);
        let context = ENEMY_SCRIPTS.start + slot * ENEMY_SCRIPT_BYTES;
        space.write::<f32>(enemy + super::ENEMY_POSITION, at.0);
        space.write::<f32>(enemy + super::ENEMY_POSITION + 4, at.1);
        space.write::<f32>(enemy + super::ENEMY_ANGLE, angle);
        space.write::<usize>(enemy + super::ENEMY_SCRIPT, context);
        space.write::<usize>(context + super::SCRIPT_BEHAVIOUR, 0);
        space.write::<f32>(
            context + super::SCRIPT_BEAM_ANCHOR,
            angle.cos() * super::BEAM_ANCHOR_ALONG,
        );
        space.write::<f32>(
            context + super::SCRIPT_BEAM_ANCHOR + 4,
            angle.sin() * super::BEAM_ANCHOR_ALONG,
        );
    }

    /// **A trail on one of those enemies**, which is what a long one of this game has: the places it has
    /// been, one entry apiece, with the count and the flags byte the game's own walk is gated on.
    ///
    /// `places` fills the entries the walk visits and nothing else — the walk starts at 1 and steps by 6,
    /// so `places[0]` goes into entry 1, `places[1]` into entry 7, and the count is written to just past
    /// the last of them. `taper` sets the bit that narrows each box towards the far end.
    pub fn an_enemy_has_a_trail_of(&self, slot: usize, places: &[(f32, f32)], taper: bool) {
        let space = self.space();
        let enemy = self.enemy_at(slot);
        let mut entries = 0;
        for (along, (x, y)) in places.iter().enumerate() {
            entries = ENEMY_FIRST_BOX + along * ENEMY_BOX_EVERY;
            let at = enemy + ENEMY_BOXES_AT + entries * ENEMY_BOX_BYTES;
            space.write::<f32>(at, *x);
            space.write::<f32>(at + size_of::<f32>(), *y);
        }
        space.write::<i16>(enemy + ENEMY_BOXES, entries as i16 + 1);
        let flags = match taper {
            true => ENEMY_HAS_A_TRAIL | ENEMY_BOXES_TAPER,
            false => ENEMY_HAS_A_TRAIL,
        };
        space.write::<u8>(enemy + ENEMY_BOX_FLAGS, flags);
    }

    /// And an enemy with no trail at all, which is what the flags byte at nothing is — and what every
    /// enemy but the long ones has.
    pub fn an_enemy_has_no_trail(&self, slot: usize) {
        let enemy = self.enemy_at(slot);
        self.space().write::<u8>(enemy + ENEMY_BOX_FLAGS, 0);
        self.space().write::<i16>(enemy + ENEMY_BOXES, 0);
    }

    /// **One `.anm` slot of the sprite manager with a file in it**, with that many sheets behind it: the
    /// file pointer whose being anything at all is what says the slot is loaded, the count, and the
    /// array of `0xc`-byte entries — one texture and one block apiece — a restore has to leave standing.
    ///
    /// The sheets of every slot are laid out one after another inside [`ANM_SHEETS`], so two loaded
    /// slots answer two ranges that do not overlap.
    pub fn an_anm_slot_is_loaded(&self, slot: usize, sheets: usize) -> Range<usize> {
        let space = self.space();
        let at = super::ANM_SLOTS_AT + slot * super::ANM_SLOT_BYTES;
        let sheets_at = ANM_SHEETS.start + slot * A_SLOTS_SHEETS_APART;
        space.write::<usize>(QUAD_QUEUE.start + at + super::ANM_SLOT_FILE, A_LOADED_FILE);
        space.write::<u32>(
            QUAD_QUEUE.start + at + super::ANM_SLOT_SHEETS,
            sheets as u32,
        );
        space.write::<usize>(QUAD_QUEUE.start + at + super::ANM_SLOT_SHEETS_AT, sheets_at);
        // **The two fields of an entry as the dwords they are on the game**, and not as a `usize`: this
        // image is a 32-bit game's, and a laid-out entry whose texture took eight bytes on a 64-bit test
        // host would write over the next entry's.
        for sheet in 0..sheets {
            let entry = sheets_at + sheet * super::A_SHEET_BYTES;
            space.write::<u32>(entry, A_TEXTURE + sheet as u32);
            space.write::<u32>(entry + A_SHEETS_BLOCK, A_BLOCK + sheet as u32);
        }
        sheets_at..sheets_at + sheets * super::A_SHEET_BYTES
    }

    /// And a slot with nothing in it, which is what the file pointer at nothing is — and what all but a
    /// handful of the manager's `0x100` slots are.
    pub fn an_anm_slot_is_empty(&self, slot: usize) {
        let at = super::ANM_SLOTS_AT + slot * super::ANM_SLOT_BYTES;
        self.space()
            .write::<usize>(QUAD_QUEUE.start + at + super::ANM_SLOT_FILE, 0);
    }

    /// **A track playing, as the sound manager holds one**: the wrapper it keeps DirectSound in, the stream
    /// the play made and the source that stream reads through, which are the three objects beside the
    /// manager's own range that a restore has to leave alone. Answers the three ranges it laid out.
    pub fn a_track_is_playing(&self) -> [Range<usize>; 3] {
        let space = self.space();
        space.write::<usize>(
            super::G_SOUND_PLAYER + super::SOUND_DEVICE,
            SOUND_DEVICE.start,
        );
        space.write::<usize>(
            super::G_SOUND_PLAYER + super::MUSIC_STREAM,
            SOUND_STREAM.start,
        );
        space.write::<usize>(
            SOUND_STREAM.start + super::STREAM_SOURCE,
            MUSIC_SOURCE.start,
        );
        [
            SOUND_DEVICE.start..SOUND_DEVICE.start + super::SOUND_DEVICE_BYTES,
            SOUND_STREAM.start..SOUND_STREAM.start + super::MUSIC_STREAM_BYTES,
            MUSIC_SOURCE.start..MUSIC_SOURCE.start + super::MUSIC_SOURCE_BYTES,
        ]
    }

    /// **The tracks `thbgm.dat` holds**, as the manager's read of that file's table leaves them: a record per
    /// name, the numbers it is sought by this file's own and different for each, and an empty record after
    /// the last — which is where the game's own lookup stops.
    pub fn holds_its_music_file(&self, names: &[&CStr]) {
        let space = self.space();
        for (entry, name) in names.iter().enumerate() {
            let record = self.the_record_of(entry);
            space.write_bytes(record, &[0; MUSIC_RECORD_BYTES]);
            space.write_bytes(record + super::RECORD_NAME, name.to_bytes_with_nul());
            let at = 0x0010_0000 * (entry as u32 + 1);
            space.write::<u32>(record + super::RECORD_AT, at);
            space.write::<u32>(record + super::RECORD_LOOP_AT, at / 4);
            space.write::<u32>(record + super::RECORD_BYTES, at / 2 + entry as u32);
        }
        space.write_bytes(self.the_record_of(names.len()), &[0; MUSIC_RECORD_BYTES]);
    }

    /// **And the stream reading the track at that entry of the table**, which is what the game's own play of a
    /// named track leaves: the source's record pointed at it — see `Th08`'s `STREAM_SOURCE`.
    pub fn the_stream_reads_the_track(&self, entry: usize) {
        self.space().write::<usize>(
            MUSIC_SOURCE.start + super::SOURCE_RECORD,
            self.the_record_of(entry),
        );
    }

    /// **The stream's own numbers**: where it puts its next chunk, how big a chunk is and how much of the track
    /// is left before the loop — `Th08`'s `STREAM_WRITE_AT`, `STREAM_CHUNK` and `SOURCE_LEFT`.
    pub fn the_stream_is_at(&self, write: u32, chunk: u32, left: u32) {
        let space = self.space();
        space.write::<u32>(SOUND_STREAM.start + super::STREAM_WRITE_AT, write);
        space.write::<u32>(SOUND_STREAM.start + super::STREAM_CHUNK, chunk);
        space.write::<u32>(MUSIC_SOURCE.start + super::SOURCE_LEFT, left);
    }

    /// And read back, in the same order.
    pub fn where_the_stream_is(&self) -> (u32, u32, u32) {
        let space = self.space();
        (
            space.read::<u32>(SOUND_STREAM.start + super::STREAM_WRITE_AT),
            space.read::<u32>(SOUND_STREAM.start + super::STREAM_CHUNK),
            space.read::<u32>(MUSIC_SOURCE.start + super::SOURCE_LEFT),
        )
    }

    /// The music's stream, which the refill is called on.
    pub fn the_music_stream(&self) -> usize {
        SOUND_STREAM.start
    }

    fn the_record_of(&self, entry: usize) -> usize {
        MUSIC_SOURCE.start + MUSIC_RECORDS_AT + entry * MUSIC_RECORD_BYTES
    }

    /// **The three tracks the stage's info block names**, the first the stage's own — as the stage's file has
    /// them, with the directory and the `.mid` a record has neither of, and a space for a slot the stage has
    /// no track in.
    pub fn names_its_tracks(&self, tracks: [&CStr; 3]) {
        let space = self.space();
        space.write::<usize>(super::G_STAGE_INFO, STAGE_INFO.start);
        for (at, track) in super::STAGE_TRACK_NAMES.into_iter().zip(tracks) {
            space.write_bytes(STAGE_INFO.start + at, &[0; super::TRACK_NAME_BYTES]);
            space.write_bytes(STAGE_INFO.start + at, track.to_bytes_with_nul());
        }
    }

    /// And no track and no device, which is what a launch before its first play has: both fields at
    /// nothing, the manager's own constructor having zeroed them.
    pub fn no_track_is_playing(&self) {
        let space = self.space();
        space.write::<usize>(super::G_SOUND_PLAYER + super::SOUND_DEVICE, 0);
        space.write::<usize>(super::G_SOUND_PLAYER + super::MUSIC_STREAM, 0);
    }

    /// **The manager's own range**, which is the one range of the sound a launch with no track answers —
    /// this file's own literals, see [`SOUND_PLAYER`].
    pub fn the_sound_manager_is(&self) -> Range<usize> {
        SOUND_PLAYER
    }

    /// **A command sitting in the manager's own queue, or none**, which is the first slot's kind: the
    /// runner switches on it, so anything but nothing is a command it has not carried out yet.
    pub fn a_sound_command_is_waiting(&self, waiting: bool) {
        let kind = if waiting { A_SOUND_COMMAND } else { 0 };
        self.space()
            .write::<u32>(super::G_SOUND_PLAYER + super::SOUND_COMMANDS, kind);
    }

    /// The thread id `PlayBGM`'s own `CreateThread` would have written down.
    pub fn the_music_is_streamed_on_thread(&self, thread: u32) {
        self.space()
            .write::<u32>(super::G_SOUND_PLAYER + super::MUSIC_THREAD_ID, thread);
    }

    /// And that many bullets, each of the kind whose step is its whole velocity.
    pub fn bullets_alive(&self, bullets: usize) {
        let space = self.space();
        for slot in 0..super::BULLET_SLOTS {
            let bullet = self.bullet_at(slot);
            if slot >= bullets {
                space.write::<u16>(bullet + super::BULLET_KIND, 0);
                continue;
            }
            space.write::<u16>(bullet + super::BULLET_KIND, 1);
            space.write::<f32>(bullet + super::BULLET_POSITION, 64.0 + slot as f32);
            space.write::<f32>(bullet + super::BULLET_POSITION + 4, 96.0 + slot as f32);
            space.write::<f32>(bullet + super::BULLET_SIZE, 4.0);
            space.write::<f32>(bullet + super::BULLET_SIZE + 4, 4.0);
            space.write::<f32>(bullet + super::BULLET_STEP, 0.0);
            space.write::<f32>(bullet + super::BULLET_STEP + 4, 2.0);
        }
        space.write::<i32>(
            super::G_BULLET_MANAGER + super::BULLET_COUNT,
            bullets as i32,
        );
    }

    /// **One bullet in one of its manager's own slots**, where its update would leave it: the kind whose
    /// step is its whole velocity, where it is, what that step adds every frame, and the size the hit
    /// test is given.
    ///
    /// Apart from [`bullets_alive`](Image::bullets_alive) because a game playing a stage of its own moves
    /// one bullet at a time, where that one lays a field out in a shape of its own choosing.
    pub fn a_bullet_is_at(&self, slot: usize, at: (f32, f32), step: (f32, f32), size: (f32, f32)) {
        let space = self.space();
        let bullet = self.bullet_at(slot);
        space.write::<u16>(bullet + super::BULLET_KIND, 1);
        space.write::<f32>(bullet + super::BULLET_POSITION, at.0);
        space.write::<f32>(bullet + super::BULLET_POSITION + 4, at.1);
        space.write::<f32>(bullet + super::BULLET_SIZE, size.0);
        space.write::<f32>(bullet + super::BULLET_SIZE + 4, size.1);
        space.write::<f32>(bullet + super::BULLET_STEP, step.0);
        space.write::<f32>(bullet + super::BULLET_STEP + 4, step.1);
        // No behaviour on it, which is what a fresh bullet has: one written over a slot a test had a
        // changing step in would keep that step changing.
        space.write::<u32>(bullet + super::BULLET_BEHAVIOURS, 0);
        // And shown, which the spawn writes too (0x42f906).
        space.write::<u8>(bullet + super::BULLET_UNTESTED, 0);
    }

    /// **And a bullet hidden by the script**, the byte at `BULLET_UNTESTED` set the way 0x424a20 and 0x424c40
    /// set it: the kind stays what it was and neither of the player's tests is made of it.
    pub fn the_bullet_is_hidden(&self, slot: usize) {
        self.space()
            .write::<u8>(self.bullet_at(slot) + super::BULLET_UNTESTED, 1);
    }

    /// **And a bullet new in a slot another one had**, which is that slot somewhere else with the count at
    /// `BULLET_FRAMES` starting again from nothing, the way 0x432198 starts it as a bullet is made — see
    /// `Hazard::frames`. `狂符「幻視調律」` lays a lattice of them and cancels it again and again, so its
    /// slots hold a different bullet every few frames.
    pub fn a_bullet_is_new_in_its_slot(
        &self,
        slot: usize,
        at: (f32, f32),
        step: (f32, f32),
        size: (f32, f32),
    ) {
        self.a_bullet_is_at(slot, at, step, size);
        self.space()
            .write::<i32>(self.bullet_at(slot) + super::BULLET_FRAMES, 0);
    }

    /// **Every live bullet one frame older**, which is what this game's own bullet update does to the timer
    /// each of them carries at `BULLET_FRAMES`: the count is what says the bullet in a slot next frame is the
    /// one that was there, and a laid-out game whose bullets never aged would have every slot read as given
    /// away — so nothing of what a bullet did against what its step claimed is read at all.
    pub fn its_bullets_are_a_frame_older(&self) {
        let len = super::BULLET_SLOTS * super::BULLET_BYTES;
        self.space().edit_bytes(self.bullet_at(0), len, |bullets| {
            for bullet in bullets.as_chunks_mut::<{ super::BULLET_BYTES }>().0 {
                let kind = u16::from_le_bytes([
                    bullet[super::BULLET_KIND],
                    bullet[super::BULLET_KIND + 1],
                ]);
                if kind == 0 {
                    continue;
                }
                let frames = word_in(bullet, super::BULLET_FRAMES) as i32;
                put_word_in(bullet, super::BULLET_FRAMES, (frames + 1) as u32);
            }
        });
    }

    /// **And the kind a bullet is in**, for a fresh one: a bullet is written in kind 1, the kind it spends its
    /// life in, and kinds 2 to 4 are the frame it is born on — see `Th08::how_much_of_its_step_a_bullet_takes`.
    pub fn the_bullet_is_of_kind(&self, slot: usize, kind: u16) {
        self.space()
            .write::<u16>(self.bullet_at(slot) + super::BULLET_KIND, kind);
    }

    /// **And how far through its own animation a spawning bullet's sprite is**, which is what says whether
    /// the spawn can end on this frame — see `BULLET_SPRITE`: the instruction it is waiting on, the frames
    /// that instruction waits, and the frames it has waited. A bullet laid out with `waited` short of
    /// `waits` is one whose spawn cannot end yet, so it moves its divided share and nothing more.
    ///
    /// A bullet none of this is written for has no instruction at all, which the game answers ended for —
    /// so the frame a spawn ends on is what a test gets without asking.
    pub fn the_bullets_spawn_has_waited(&self, slot: usize, waited: i32, waits: i16) {
        let space = self.space();
        let sprite = self.bullet_at(slot) + super::BULLET_SPRITE;
        let script = BULLET_ANIMATIONS.start + slot * BULLET_ANIMATION_BYTES;
        space.write::<usize>(sprite + super::SPRITE_SCRIPT, script);
        space.write::<u32>(sprite + super::SPRITE_FLAGS, 0);
        space.write::<u16>(sprite + super::SPRITE_CASE, 0);
        space.write::<i32>(sprite + super::SPRITE_ELAPSED, waited);
        space.write::<i16>(script + super::SPRITE_WAIT, waits);
    }

    /// **And a bullet whose step is changing**, the behaviour at `BULLET_BEHAVIOURS` set and the three
    /// numbers 0x4322b0 reads beside it: what is added to the step each frame, how many frames that has
    /// run for, and how many it runs for in all.
    pub fn a_bullet_accelerates_by(&self, slot: usize, by: (f32, f32), run: i32, of: i32) {
        let space = self.space();
        let bullet = self.bullet_at(slot);
        space.write::<u32>(bullet + super::BULLET_BEHAVIOURS, super::BULLET_ACCELERATES);
        space.write::<f32>(bullet + super::BULLET_ACCELERATION, by.0);
        space.write::<f32>(bullet + super::BULLET_ACCELERATION + 4, by.1);
        space.write::<i32>(bullet + super::BULLET_ACCELERATION_FRAMES_RUN, run);
        space.write::<i32>(bullet + super::BULLET_ACCELERATION_FRAMES, of);
    }

    /// And a slot with no bullet in it, which is the kind at nothing.
    pub fn the_bullet_is_gone(&self, slot: usize) {
        self.space()
            .write::<u16>(self.bullet_at(slot) + super::BULLET_KIND, 0);
    }

    /// And the manager's own count of them, which is the number `Th08::read_state` answers with rather
    /// than the walk's.
    pub fn the_bullet_manager_counts(&self, bullets: i32) {
        self.space()
            .write::<i32>(super::G_BULLET_MANAGER + super::BULLET_COUNT, bullets);
    }

    /// And that many lasers, each firing rather than warning: the phase past the warning is what makes
    /// `fires_in` nothing.
    pub fn lasers_alive(&self, lasers: usize) {
        let space = self.space();
        for slot in 0..super::LASER_SLOTS {
            let laser = self.laser_at(slot);
            if slot >= lasers {
                space.write::<u32>(laser + super::LASER_ALIVE, 0);
                continue;
            }
            space.write::<u32>(laser + super::LASER_ALIVE, 1);
            space.write::<u8>(laser + super::LASER_PHASE, 1);
            space.write::<f32>(laser + super::LASER_ORIGIN, 16.0 + slot as f32);
            space.write::<f32>(laser + super::LASER_ORIGIN + 4, 24.0 + slot as f32);
            space.write::<f32>(laser + super::LASER_ANGLE, 0.5);
            space.write::<f32>(laser + super::LASER_NEAR, 0.0);
            space.write::<f32>(laser + super::LASER_FAR, 320.0);
            space.write::<f32>(laser + super::LASER_WIDTH, 16.0);
            space.write::<f32>(laser + super::LASER_GROWS, 2.0);
            space.write::<i32>(laser + super::LASER_TIMER, 40);
            space.write::<i32>(laser + super::LASER_LIVE_AT, 30);
        }
    }

    /// **One laser where a test puts it**: fired from `origin` along `angle`, reaching from `along.0` to
    /// `along.1`, `width` wide, settled in length, and firing — the phase past the warning.
    pub fn a_laser_is_at(
        &self,
        slot: usize,
        origin: (f32, f32),
        angle: f32,
        along: (f32, f32),
        width: f32,
    ) {
        let space = self.space();
        let laser = self.laser_at(slot);
        space.write::<u32>(laser + super::LASER_ALIVE, 1);
        space.write::<u8>(laser + super::LASER_PHASE, 1);
        space.write::<f32>(laser + super::LASER_ORIGIN, origin.0);
        space.write::<f32>(laser + super::LASER_ORIGIN + 4, origin.1);
        space.write::<f32>(laser + super::LASER_ANGLE, angle);
        space.write::<f32>(laser + super::LASER_NEAR, along.0);
        space.write::<f32>(laser + super::LASER_FAR, along.1);
        space.write::<f32>(laser + super::LASER_WIDTH, width);
        space.write::<f32>(laser + super::LASER_GROWS, 0.0);
        space.write::<i32>(laser + super::LASER_TIMER, 0);
        space.write::<i32>(laser + super::LASER_LIVE_AT, 0);
    }

    /// **And that laser still drawing its warning**, this many frames short of the one it starts hitting
    /// on: the first phase, its count at nought and the frame it goes live at `frames`.
    pub fn the_laser_fires_in(&self, slot: usize, frames: i32) {
        let space = self.space();
        let laser = self.laser_at(slot);
        space.write::<u8>(laser + super::LASER_PHASE, 0);
        space.write::<i32>(laser + super::LASER_TIMER, 0);
        space.write::<i32>(laser + super::LASER_LIVE_AT, frames);
    }

    /// **And that laser past its warning**: the second phase with its count started again from nought,
    /// which is how card 198's lasers read on the real game the frame after their first phase's sixtieth —
    /// phase 1 at a count of 1 once [`the_laser_was_turned_to`](Image::the_laser_was_turned_to) has moved it
    /// on.
    pub fn the_laser_goes_live(&self, slot: usize) {
        let space = self.space();
        let laser = self.laser_at(slot);
        space.write::<u8>(laser + super::LASER_PHASE, 1);
        space.write::<i32>(laser + super::LASER_TIMER, 0);
    }

    /// **And that laser one frame on, pointing somewhere else** — which is a laser its boss's script is
    /// turning: the angle it points along now, and one more frame on the count of its phase, which is what
    /// says the laser in this slot is still the same one (see `game::Beam::frames`).
    pub fn the_laser_was_turned_to(&self, slot: usize, angle: f32) {
        let space = self.space();
        let laser = self.laser_at(slot);
        space.write::<f32>(laser + super::LASER_ANGLE, angle);
        let count = space.read::<i32>(laser + super::LASER_TIMER);
        space.write::<i32>(laser + super::LASER_TIMER, count + 1);
    }

    /// And that many items on the list, each of the class that is a power item and each reachable: the
    /// list is how this game keeps the live ones, so what is laid out is the links.
    pub fn items_alive(&self, items: usize) {
        let space = self.space();
        let head = super::G_ITEM_MANAGER + super::ITEM_LIST;
        space.write::<usize>(head, if items == 0 { 0 } else { self.item_at(0) });
        for slot in 0..items {
            let item = self.item_at(slot);
            space.write::<f32>(item + super::ITEM_POSITION, 128.0 + slot as f32);
            space.write::<f32>(item + super::ITEM_POSITION + 4, 160.0 + slot as f32);
            space.write::<u8>(item + super::ITEM_CLASS, 0);
            space.write::<u8>(item + super::ITEM_REACH, 0);
            let next = match slot + 1 < items {
                true => self.item_at(slot + 1),
                false => 0,
            };
            space.write::<usize>(item + super::ITEM_NEXT, next);
        }
        space.write::<i32>(super::G_ITEM_MANAGER + super::ITEM_COUNT, items as i32);
    }

    /// **And one item where a test puts it**, the list holding that one alone: a power item, reachable.
    pub fn an_item_is_at(&self, at: (f32, f32)) {
        self.items_alive(1);
        let item = self.item_at(0);
        let space = self.space();
        space.write::<f32>(item + super::ITEM_POSITION, at.0);
        space.write::<f32>(item + super::ITEM_POSITION + 4, at.1);
    }

    /// And of a class of a test's choosing — see `sight::worth` for which of this game's nine each is.
    pub fn the_item_is_of_class(&self, class: u8) {
        self.space()
            .write::<u8>(self.item_at(0) + super::ITEM_CLASS, class);
    }

    /// Where one slot of each array is, which is the walk the game's own updates do — see
    /// `ENEMIES_AT`, `BULLETS_AT` and `LASERS_AT`.
    fn enemy_at(&self, slot: usize) -> usize {
        super::G_ENEMY_MANAGER + super::ENEMIES_AT + slot * super::ENEMY_BYTES
    }

    fn bullet_at(&self, slot: usize) -> usize {
        super::G_BULLET_MANAGER + super::BULLETS_AT + slot * super::BULLET_BYTES
    }

    fn laser_at(&self, slot: usize) -> usize {
        super::G_BULLET_MANAGER + super::LASERS_AT + slot * super::LASER_BYTES
    }

    /// And where one item is, which is a slot of the pool the manager hands its list out of: `0x831` of
    /// `0x2e4` bytes from the object's own start.
    fn item_at(&self, slot: usize) -> usize {
        super::G_ITEM_MANAGER + slot * ITEM_BYTES
    }

    /// **The 222 card records stamped**, which is what the init at 0x43bbe1 leaves before the score file
    /// is read over them: the tag, the length twice, the version and the record's own index.
    ///
    /// Which is the state `Game::these_records_hold_a_count` has to answer `false` for — a stamped record
    /// is full of bytes and holds no count.
    pub fn its_card_records_are_stamped(&self) {
        let space = self.space();
        for card in 0..CARDS {
            let record = self.card_record(card);
            space.write_bytes(record, CARD_RECORD_TAG);
            space.write::<u16>(record + CARD_RECORD_LENGTH, CARD_RECORD_BYTES as u16);
            space.write::<u16>(record + CARD_RECORD_LENGTH_AGAIN, CARD_RECORD_BYTES as u16);
            space.write::<u8>(record + CARD_RECORD_VERSION_AT, CARD_RECORD_VERSION);
            space.write::<u16>(record + CARD_RECORD_INDEX, card as u16);
            for array in CARD_RECORD_ARRAYS {
                for slot in 0..CARD_RECORD_SLOTS {
                    space.write::<u32>(record + array + slot * size_of::<u32>(), 0);
                }
            }
            space.write::<u8>(record + CARD_RECORD_SUM, 0);
        }
    }

    /// The name one of them carries, which is what the card's own start copies in — and what
    /// `Game::set_captures_keeping_names` has to leave standing.
    pub fn a_cards_record_is_named(&self, card: usize, name: &str) {
        let space = self.space();
        let record = self.card_record(card);
        space.write_bytes(record + CARD_RECORD_NAME, name.as_bytes());
        space.write::<u8>(record + CARD_RECORD_NAME + name.len(), 0);
    }

    pub fn the_name_a_cards_record_carries(&self, card: usize) -> String {
        let record = self.card_record(card) + CARD_RECORD_NAME;
        // Up to where the first of the six arrays starts, which is the whole of the room a name has.
        let bytes: Vec<u8> = (0..CARD_RECORD_ARRAYS[2] - CARD_RECORD_NAME)
            .map(|at| self.space().read::<u8>(record + at))
            .take_while(|byte| *byte != 0)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// The attempts and the captures one of them holds for one character, which are the two counts of the
    /// six arrays that are read.
    pub fn a_card_has_been_attempted(&self, card: usize, character: usize, attempts: u32) {
        self.space().write::<u32>(
            self.card_record(card) + CARD_RECORD_ATTEMPTS + character * size_of::<u32>(),
            attempts,
        );
    }

    pub fn the_attempts_at_a_card(&self, card: usize, character: usize) -> u32 {
        self.space().read::<u32>(
            self.card_record(card) + CARD_RECORD_ATTEMPTS + character * size_of::<u32>(),
        )
    }

    pub fn a_card_has_been_captured(&self, card: usize, character: usize, captures: u32) {
        self.space().write::<u32>(
            self.card_record(card) + CARD_RECORD_CAPTURES + character * size_of::<u32>(),
            captures,
        );
    }

    pub fn the_captures_of_a_card(&self, card: usize, character: usize) -> u32 {
        self.space().read::<u32>(
            self.card_record(card) + CARD_RECORD_CAPTURES + character * size_of::<u32>(),
        )
    }

    /// **The byte one of them carries, and the byte the game's own rule says it should**: the name's own
    /// bytes as signed, then every dword of all six arrays, truncated.
    ///
    /// Written out here as well as in `Th08::stamp_a_cards_record` on purpose — a record whose count orb
    /// changed and whose byte orb left is one the game zeroes, and the only way to say the byte is right
    /// is to work it out from this file's own literals rather than from the code under test.
    pub fn the_sum_a_cards_record_carries(&self, card: usize) -> u8 {
        self.space()
            .read::<u8>(self.card_record(card) + CARD_RECORD_SUM)
    }

    pub fn the_sum_the_games_rule_gives_a_cards_record(&self, card: usize) -> u8 {
        let space = self.space();
        let record = self.card_record(card);
        let mut sum = 0u32;
        for at in 0..CARD_RECORD_ARRAYS[2] - CARD_RECORD_NAME {
            let letter = space.read::<i8>(record + CARD_RECORD_NAME + at);
            if letter == 0 {
                break;
            }
            sum = sum.wrapping_add(letter as u32);
        }
        for array in CARD_RECORD_ARRAYS {
            for slot in 0..CARD_RECORD_SLOTS {
                sum = sum.wrapping_add(space.read::<u32>(record + array + slot * size_of::<u32>()));
            }
        }
        sum as u8
    }

    /// **A card up, by its own number**, which is the pair `Th08::the_card_that_is_up` reads: the flags
    /// word's bit 0 and the number beside it.
    pub fn a_card_is_up(&self, card: usize) {
        let space = self.space();
        space.write::<u32>(super::G_SPELL_CARD, SPELL_CARD_IS_UP);
        space.write::<i32>(super::G_SPELL_CARD + SPELL_CARD_NUMBER, card as i32);
    }

    pub fn no_card_is_up(&self) {
        self.space().write::<u32>(super::G_SPELL_CARD, 0);
    }

    /// **What the card would pay in time orbs, and the two times its own ramp is on** — see
    /// `Th08`'s `CARD_TIME_ORBS`, whose ramp from 100 to 1,000 is what this game's Last Spell is decided
    /// by. The counts go in where the timer keeps them, which is `+0x8` of each.
    pub fn the_card_pays(&self, orbs: i32, left: i32, of: i32) {
        let space = self.space();
        space.write::<i32>(super::G_SPELL_CARD + SPELL_CARD_TIME_ORBS, orbs);
        space.write::<i32>(super::G_SPELL_CARD + SPELL_CARD_TIME_LEFT, left);
        space.write::<i32>(super::G_SPELL_CARD + SPELL_CARD_TIME, of);
    }

    /// Whose run it is, which is the index the counts above are kept per.
    pub fn the_character_playing_is(&self, character: u8) {
        self.space().write::<u8>(super::G_CHARACTER, character);
    }

    /// And a run that is a practice of one card, which is the one kind of run the game counts no attempt
    /// for.
    pub fn the_run_is_a_spell_practice(&self) {
        self.space()
            .write::<u32>(super::G_RUN_KIND, super::RUN_IS_A_SPELL_PRACTICE);
    }

    fn card_record(&self, card: usize) -> usize {
        super::G_GAME_MANAGER + CARD_RECORDS + card * CARD_RECORD_BYTES
    }

    /// The word the input read last left, and the one before it — the pair every screen of this game's
    /// front end compares to find an edge.
    pub fn the_keys_read(&self, word: u16, last: u16) {
        let space = self.space();
        space.write::<u16>(super::G_INPUT_READ, word);
        space.write::<u16>(super::G_INPUT_LAST, last);
    }

    pub fn the_word_the_keys_read(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_READ)
    }

    pub fn the_word_the_keys_read_last(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_LAST)
    }

    /// **What the supervisor's switch leaves the keys' words as**: the repeat word, last frame's and the read
    /// all put to nothing at the switch's tail (0x445adf through 0x445af9) — so a key held across a scene is an
    /// edge again on the new scene's first read.
    pub fn the_switch_puts_the_keys_to_nothing(&self) {
        let space = self.space();
        space.write::<u16>(INPUT_REPEATING, 0);
        space.write::<u16>(super::G_INPUT_LAST, 0);
        space.write::<u16>(super::G_INPUT_READ, 0);
    }

    /// **The word the run acts on**, which is what the player's own job moves on — see
    /// `G_INPUT_THE_RUN_ACTS_ON` — and the one it acted on last.
    pub fn the_word_the_run_acts_on(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_THE_RUN_ACTS_ON)
    }

    pub fn the_word_the_run_acted_on(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_THE_RUN_ACTED_ON)
    }

    /// **The run's replay's own copy, at its element's priority 0x11**: the word acted on into the one before
    /// it (0x452332), and the read into the word acted on (0x452340).
    pub fn the_runs_replay_copies_the_read(&self) {
        let space = self.space();
        let acts_on = space.read::<u16>(super::G_INPUT_THE_RUN_ACTS_ON);
        space.write::<u16>(super::G_INPUT_THE_RUN_ACTED_ON, acts_on);
        let read = space.read::<u16>(super::G_INPUT_READ);
        space.write::<u16>(super::G_INPUT_THE_RUN_ACTS_ON, read);
    }

    /// And both put to nothing, which is what registering the run's replay does (0x451fb2, 0x451fbb).
    pub fn the_runs_replay_is_registered(&self) {
        let space = self.space();
        space.write::<u16>(super::G_INPUT_THE_RUN_ACTS_ON, 0);
        space.write::<u16>(super::G_INPUT_THE_RUN_ACTED_ON, 0);
    }

    /// The one range this game's statics are in, which is what a laid-out launch hands
    /// `orb_core::runtime::attach_to` in place of the section read out of the PE — see [`DATA`].
    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// The host this game is laid out in, for an e2e test that needs more of it than the memory — the
    /// display it declares, or which window is in front.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }
}

/// The word at `at` of one slot of an array handed over by `Space::edit_bytes`.
fn word_in(slot: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(slot[at..at + 4].try_into().unwrap())
}

fn put_word_in(slot: &mut [u8], at: usize, word: u32) {
    slot[at..at + 4].copy_from_slice(&word.to_le_bytes());
}

/// How many bytes one item of the manager's pool takes, off 0x4400b2's `imul` — `0x2e4`, and `0x831` of
/// them from the object's own start, which is exactly the `0x17adac` its bookkeeping begins at.
const ITEM_BYTES: usize = 0x2e4;

#[cfg(test)]
mod tests {
    use super::{DATA, Image, Scene, Screen};
    use crate::game::Game;

    /// Every block the image lays out is somewhere orb can read, and the statics are in the one range a
    /// chapter would be a copy of: a block laid out outside it is one a snapshot would not carry.
    #[test]
    fn every_static_this_game_keeps_is_in_the_range_a_chapter_copies() {
        for address in [
            super::super::G_GAME_MANAGER,
            super::super::G_SUPERVISOR,
            super::super::G_PLAYER,
            super::super::G_ENEMY_MANAGER,
            super::super::G_BULLET_MANAGER,
            super::super::G_ITEM_MANAGER,
            super::super::G_RANDOM,
            super::super::G_CHAIN,
            super::super::G_STAGE,
            super::super::G_RUN_SCREEN,
            super::super::G_RUN_KIND,
            super::super::G_INPUT_READ,
        ] {
            assert!(
                DATA.contains(&address),
                "{address:#x} is outside the range a chapter copies"
            );
        }
    }

    /// **The jobs a frame has are the calc list's own, in its own order and with the object each is called
    /// on** — which is what says a frame that ran none of them can be told from one whose list was empty.
    #[test]
    fn the_jobs_a_frame_has_are_the_calc_lists_own() {
        let image = Image::laid_out_seeded(1);
        let _held = image.enter();
        image.the_calc_list_holds(&[], false);
        assert_eq!(
            unsafe { super::super::Th08.the_jobs_a_frame_has() }.as_deref(),
            Some("0 on the calc list:"),
        );
        image.the_calc_list_holds(&A_FRAMES_JOBS, false);
        let said = unsafe { super::super::Th08.the_jobs_a_frame_has() }
            .expect("this game says what is on its chain");
        assert_eq!(
            said,
            format!(
                "{} on the calc list: {:#010x}({:#010x}) {:#010x}({:#010x})",
                A_FRAMES_JOBS.len(),
                A_FRAMES_JOBS[0].0,
                A_FRAMES_JOBS[0].1,
                A_FRAMES_JOBS[1].0,
                A_FRAMES_JOBS[1].1,
            ),
        );
    }

    /// **And a list that points back at itself is one the walk gives up on**, which is what keeps a
    /// pointer read wrong from being a frame that never ends: the count it stops at is the bound and not
    /// the list's own length, there being none.
    #[test]
    fn a_calc_list_that_loops_is_walked_no_further_than_its_bound() {
        let image = Image::laid_out_seeded(1);
        let _held = image.enter();
        image.the_calc_list_holds(&A_FRAMES_JOBS, true);
        let said = unsafe { super::super::Th08.the_jobs_a_frame_has() }
            .expect("this game says what is on its chain");
        assert!(
            said.starts_with(&format!(
                "{} on the calc list:",
                super::super::CHAIN_JOBS_AT_MOST
            )),
            "a looping list was walked to {said}"
        );
    }

    /// Two jobs for those two tests, each a callback and the object it would be called on: addresses of
    /// this file's choosing, since what the walk does with them is carry them into a line.
    const A_FRAMES_JOBS: [(usize, usize); 2] =
        [(0x0043_c880, 0x0160_f508), (0x0041_8010, 0x004e_a670)];

    /// And the image answers what was written into it, which is what says the layout and the reads agree
    /// on where each thing is.
    #[test]
    fn what_is_laid_out_is_what_the_game_reads_back() {
        let image = Image::laid_out_seeded(1);
        let _held = image.enter();
        image.is_in_scene(Scene::Run, Scene::Run);
        image.has_a_runs_numbers();
        image.is_playing(2, 1);
        image.has_been_in_the_run_for(1_234);
        image.the_stage_script_has_run_for(567);
        image.the_stage_began_with_the_seed(0x1a2b);
        image.has_lives_and_bombs(2.0, 3.0);
        image.shot_is_at_power(64.0);
        image.has_died(1.0);
        image.the_run_is_its_own();
        image.the_player_is_playing();
        image.no_dialogue_is_up();
        image.no_boss_is_fighting();
        image.enemies_alive(3);
        image.bullets_alive(7);
        image.lasers_alive(2);
        image.items_alive(4);

        let state = unsafe { super::super::Th08.read_state() };
        assert_eq!(state.scene, Scene::Run as i32);
        assert_eq!(state.stage, 2);
        assert_eq!(state.difficulty, 1);
        assert_eq!(state.stage_frames, 1_234);
        // The stage script's own clock, which is the first context's and none of the fifteen behind it.
        assert_eq!(state.script_frames, 567);
        assert_eq!(state.random_seed, 0x1a2b);
        assert_eq!(state.lives, 2);
        assert_eq!(state.bombs, 3);
        assert_eq!(state.power, 64);
        assert_eq!(state.deaths, 1);
        assert!(!state.replay);
        assert!(!state.unsettled);
        assert!(!state.in_dialogue);
        assert!(!state.boss_present);
        assert_eq!(state.enemy_count, 3);
        assert_eq!(state.bullet_count, 7);
        assert_eq!(state.laser_count, 2);
    }

    /// **An enemy the game's own walk tests nothing of is no body to the rules and nothing to shoot at** —
    /// the flags stage 1's midsection keeps one enemy alive under from its start to the midboss, which a
    /// count of the live ones never let reach nothing and a `--collect` pass proposed no boundary through.
    #[test]
    fn an_enemy_passed_over_by_every_test_is_no_body_and_nothing_to_shoot_at() {
        let image = Image::laid_out_seeded(3);
        let _held = image.enter();
        image.is_in_scene(Scene::Run, Scene::Run);
        image.has_a_runs_numbers();
        image.the_player_is_playing();
        image.no_dialogue_is_up();
        image.no_boss_is_fighting();
        image.enemies_alive(2);
        image.an_enemy_is_passed_over_by_every_test(0);

        let state = unsafe { super::super::Th08.read_state() };
        assert_eq!(
            state.enemy_count, 1,
            "the enemy nothing can shoot was counted"
        );

        let mut frame = crate::game::Frame::default();
        assert!(unsafe { super::super::Th08.what_a_bot_sees(&mut frame) });
        let solid: Vec<(usize, bool)> = frame
            .hazards
            .iter()
            .filter_map(|hazard| match hazard.kind {
                crate::game::Kind::Body { solid } => Some((hazard.slot, solid)),
                _ => None,
            })
            .collect();
        assert_eq!(
            solid,
            vec![(0, false), (1, true)],
            "the enemy the game tests nothing of was handed over as a body to keep out of",
        );
    }

    /// The front end's own screen and cursor read back, which is what a recipe of this game presses at —
    /// **and nothing of either once a run's build has freed it**, though the pointer still names it.
    ///
    /// The scene is what the second half turns on: `G_FRONT_END` is written once in the whole exe and
    /// never cleared, so what says the front end is gone is the supervisor and not the pointer.
    #[test]
    fn the_front_end_answers_the_screen_it_was_put_on() {
        let image = Image::laid_out_seeded(2);
        let _held = image.enter();
        image.is_in_scene(Scene::FrontEnd, Scene::FrontEnd);
        image.the_front_end_shows(Screen::Difficulty, 1);
        assert_eq!(
            unsafe { super::super::Th08.front_end_screen() },
            Some(Screen::Difficulty as i32)
        );
        assert_eq!(unsafe { super::super::Th08.front_end_cursor() }, Some(1));

        // The run built, which is what frees it: the pointer still points and the answer is nothing.
        image.is_in_scene(Scene::Run, Scene::Run);
        image.the_front_end_is_freed();
        assert!(image.the_front_end_is_still_pointed_at());
        assert_eq!(unsafe { super::super::Th08.front_end_screen() }, None);
        assert_eq!(unsafe { super::super::Th08.front_end_cursor() }, None);
    }

    /// **What the card would pay in time orbs, with the two times its ramp is on**, read back off the
    /// reproduction line: this game's Last Spell is decided by that number and by nothing the panel's own
    /// count ever holds — see `Th08`'s `CARD_TIME_ORBS` — so a run held against its norm is held against
    /// this line.
    #[test]
    fn a_cards_own_payout_is_in_the_line_a_run_is_reproduced_by() {
        let image = Image::laid_out_seeded(5);
        let _held = image.enter();
        image.is_in_scene(Scene::Run, Scene::Run);
        image.has_a_runs_numbers();
        image.a_card_is_up(3);
        // A card taken with a third of its own time left, which the ramp pays about four hundred for.
        image.the_card_pays(406, 1200, 3600);
        let said = unsafe { super::super::Th08.reproduction() };
        assert!(
            said.contains("card_orbs=406 card_left=1200/3600"),
            "the card's payout is not in the line: {said}",
        );
    }

    /// And what a bot is told of a laid-out run, which is every array `sight.rs` walks: a count read back
    /// as another is an offset this file and the reads disagree on.
    #[test]
    fn a_bot_is_told_of_every_array_the_image_lays_out() {
        use crate::game::Frame;

        let image = Image::laid_out_seeded(3);
        let _held = image.enter();
        image.is_in_scene(Scene::Run, Scene::Run);
        image.has_a_runs_numbers();
        image.the_player_is_playing();
        image.no_dialogue_is_up();
        image.no_boss_is_fighting();
        image.enemies_alive(2);
        image.bullets_alive(5);
        image.lasers_alive(1);
        image.items_alive(3);
        // One of the two enemies long, which is the shape a trail is handed over for, and the other
        // not — so what comes back is a count neither an enemy per slot nor a box per entry.
        image.an_enemy_has_a_trail_of(0, &A_TRAIL, false);
        image.an_enemy_has_no_trail(1);

        let mut frame = Frame::default();
        assert!(unsafe { super::super::Th08.what_a_bot_sees(&mut frame) });
        // The four the player is held inside, and not the area: see `PLAYER_BOUNDS`.
        assert_eq!(frame.field, super::PLAYER_BOUNDS);
        assert_eq!(frame.at, (192.0, 384.0));
        assert_eq!(frame.straight, (4.0, 4.0));
        assert_eq!(frame.focused_straight, (2.0, 2.0));
        assert_eq!(frame.hazards.len(), 7 + A_TRAIL.len());
        assert_eq!(frame.beams.len(), 1);
        assert_eq!(frame.items.len(), 3);
        assert!(frame.normal);
        assert!(!frame.hit);
        assert!(frame.boss.is_none());

        // **And the trail's own boxes are where they were laid out**, which is what says the walk's own
        // first index and step of 1 and 6 are the ones orb reads: a box at any other stride lands in an
        // entry this file never wrote and comes back as nothing.
        let trail: Vec<(f32, f32)> = frame
            .hazards
            .iter()
            .filter(|hazard| matches!(hazard.kind, crate::game::Kind::Trail))
            .map(|hazard| hazard.at)
            .collect();
        assert_eq!(trail, A_TRAIL);

        // **And a beam an enemy's script lays reaches the rules as a beam, body or no body** — see
        // `ENEMY_SCRIPT`: from the middle the enemy sits at back along its angle, half the width the
        // behaviour tests. The origin is the enemy less the anchor, which is the beam's own half length
        // along that angle, so the box runs from behind the enemy to as far again in front.
        const ANGLE: f32 = 1.0;
        let middle = (100.0, 50.0);
        let behind = (
            middle.0 - ANGLE.cos() * super::super::BEAM_ANCHOR_ALONG,
            middle.1 - ANGLE.sin() * super::super::BEAM_ANCHOR_ALONG,
        );
        image.an_enemy_has_no_body(1);
        image.an_enemy_fires_a_beam(1, middle, ANGLE, 1);
        let mut frame = Frame::default();
        assert!(unsafe { super::super::Th08.what_a_bot_sees(&mut frame) });
        let beam = frame
            .beams
            .iter()
            .find(|beam| beam.slot == 1)
            .expect("the enemy's beam was not handed over");
        assert_eq!(
            (
                beam.origin,
                beam.angle,
                beam.far,
                beam.half_across,
                beam.fires_in
            ),
            (
                behind,
                ANGLE,
                super::super::BEAM_LENGTH,
                super::super::BEAM_BEHAVIOURS[1].1 / 2.0,
                0,
            ),
        );
        assert_eq!(
            frame.beams.len(),
            2,
            "the laser laid out and the enemy's beam"
        );

        // **And one aimed and not yet firing is the same beam at the narrowest of the three, handed over
        // as one nothing says the frame of**, which is what the rules are given a warning out of: the
        // behaviour says which width only once it is testing, and it is also the whole of what says the
        // lane is live — four enemies of stage 4's fight carry an anchor apiece and fire in turn.
        image.an_enemy_aims_a_beam(1, middle, ANGLE);
        let mut frame = Frame::default();
        assert!(unsafe { super::super::Th08.what_a_bot_sees(&mut frame) });
        let beam = frame
            .beams
            .iter()
            .find(|beam| beam.slot == 1)
            .expect("the beam aimed but not firing was not handed over");
        assert_eq!(
            (beam.origin, beam.angle, beam.half_across, beam.fires_in),
            (
                behind,
                ANGLE,
                super::super::BEAM_NARROWEST / 2.0,
                crate::game::FIRES_WHEN_NOTHING_SAYS,
            ),
        );
    }

    /// The places a long enemy has been, as this test lays them out: three of them, each somewhere of its
    /// own, so a box read out of the wrong entry cannot pass for one of these.
    const A_TRAIL: [(f32, f32); 3] = [(100.0, 110.0), (120.0, 130.0), (140.0, 150.0)];

    /// **The track playing is told apart by its record, and a fight's song by the stage's own names** —
    /// the second and the third of them, held against the record's by the stem, and the first the stage's
    /// own. Nothing where no track has been played, and nothing where the first slot holds a card's track.
    #[test]
    fn the_track_playing_is_told_apart_and_a_fights_song_is_one_the_stage_names_after_its_own() {
        let image = Image::laid_out_seeded(6);
        let _held = image.enter();
        let th08 = super::super::Th08;
        image.no_track_is_playing();
        assert_eq!(
            th08.music_identity(),
            None,
            "a launch with no track read an identity"
        );
        assert_eq!(th08.the_fights_own_song_is_playing(), None);

        image.a_track_is_playing();
        image.holds_its_music_file(&[c"th08_03.wav", c"th08_04.wav", c"th08_05.wav"]);
        image.names_its_tracks([c"bgm/th08_03.mid", c"bgm/th08_04.mid", c"bgm/th08_05.mid"]);
        let heard: Vec<(Option<u32>, Option<bool>)> = (0..3)
            .map(|entry| {
                image.the_stream_reads_the_track(entry);
                (th08.music_identity(), th08.the_fights_own_song_is_playing())
            })
            .collect();
        assert!(
            heard[0].0 != heard[1].0 && heard[1].0 != heard[2].0 && heard[0].0 != heard[2].0,
            "two tracks read as one identity: {heard:?}",
        );
        assert_eq!(
            heard.iter().map(|(_, fights)| *fights).collect::<Vec<_>>(),
            [Some(false), Some(true), Some(true)],
        );

        // A slot whose name is a space is one the stage has no track in, and no song of a fight's.
        image.names_its_tracks([c"bgm/th08_03.mid", c"bgm/th08_04.mid", c" "]);
        assert_eq!(th08.the_fights_own_song_is_playing(), Some(false));

        image.the_run_loads_a_cards_track();
        assert_eq!(th08.the_fights_own_song_is_playing(), None);
    }
}
