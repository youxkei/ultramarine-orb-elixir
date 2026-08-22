//! A th07 process image laid out in an address space, so that the real [`Th07`](super::Th07) has
//! something to read in a test with no game running.
//!
//! Less of one than [`th06::image`](super::super::th06::image), and for the reason the `Th07` beside it
//! declines what it declines. What a frame of orb's own reads out of this space is the device it draws
//! through, the window it asks about, the chain it calls the update and the draw at, the queue of quads it
//! empties and draws around them and the fog flag it writes — and the update, the draw and the calls on that
//! queue are functions an e2e test hands over rather than code that would have to be here.
//!
//! **And a run, as far as `Th07::read_state` reads one**: the scene, the stage, the two clocks, the run's own
//! numbers behind the pointer its build allocates, the kind of run, the player, the dialogue, the enemies and
//! their boss, the bullets, every laser slot, and an ending with the chain element that carries it. Each of
//! those is laid out where the game itself keeps it, so a test can say what orb answers for a game in a state
//! the game can really be in.
//!
//! The addresses are the game's own, written from the same constants the reads use, so what this cannot
//! catch is a constant that is wrong: a wrong one is wrong on both sides at once, and only the real
//! game says otherwise. What it does catch is everything built on top of them, which is where the code
//! is.
//!
//! **The values are this file's own, though, and taking one from [`Th07`](super::Th07) is what is refused
//! here.** An address shared between the writer and the reader is an address they fault or read nonsense
//! through together; a scene number, a screen number, an item, a phase, a mask or a cap shared between them
//! is silent, and what it costs is a whole piece of the mode never happening. That is not hypothetical: the
//! phase this file wrote the title's items in came from orb's own constant, which held 0 where the title
//! reads its items in phase 1, so orb held every press back on the real game and no e2e test over the mode
//! question could say so. So each value below carries its own literal and the exe reading it was taken from,
//! and the two colours in `bakes_a_string` are the same rule in assertion form — held against orb's own
//! constants they compared each with itself.

use std::ffi::CStr;
use std::ops::Range;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use orb_api::mem;
use orb_api::{Hwnd, Kind};
use orb_sim::{Sim, Space};

use orb_api::Device;

/// The game's static data, as the real one is: **one range**, `0x0049c000..0x01365258`, which is what
/// orb reads out of the PE and writes in the log every run. Six times 紅魔郷's, and the range a
/// snapshot of a chapter would be a copy of if 妖々夢 had chapters.
///
/// The section header's own virtual size, `0xec9258` — not its raw size, which is `0x3800`: the rest is
/// the globals the loader zeroes, and every address below is in it.
const DATA: Range<usize> = 0x0049_c000..0x0136_5258;

/// The object the drawing queues its quads in, which in the real game is 0x17e560 bytes off the
/// allocator and so is in none of [`DATA`]: a base of this file's choosing, the length the game's own.
///
/// The length is what makes it this block and not a smaller one — the two fields the frame's calls move
/// are at 0x17e534 and 0x17e538, near the end of it, and the buffer they point into is everything from
/// 0x2e534 up to them.
const QUAD_QUEUE: Range<usize> = 0x0300_0000..0x0317_E560;

/// Where the next quad goes, and where drawing the queue starts from: `0x17e534` and `0x17e538` of the
/// object.
///
/// Read off the three functions that move them — 0x44f580 points the first at the buffer and copies it
/// into the second, 0x44f690 adds [`QUAD_BYTES`] to the first, and 0x44f5c0 draws from the second and
/// then copies the first into it.
const QUEUE_WRITE_AT: usize = 0x0017_E534;
const QUEUE_DRAW_FROM: usize = 0x0017_E538;
/// The buffer those two point into, and the count of quads in it: `this + 0x2e534` as 0x44f580 writes
/// it, and the `0x2e530` it zeroes.
const QUEUE_BUFFER: usize = 0x0002_E534;
const QUEUED: usize = 0x0002_E530;
/// How many times the queue has been drawn: `[this+0x14]`, which 0x44f5c0 adds one to at 0x44f679 on the
/// calls that had something to draw.
///
/// The game's own count and not this file's, which is what makes it worth reading back: what an e2e
/// test asks of orb's frame is that the game's own idea of how many times it drew agrees with the
/// frames.
const QUEUE_DRAWS: usize = 0x14;
/// What one quad costs the buffer: six vertices of 0x1c bytes, which is what 0x44f690 copies in — the
/// four of a quad as two triangles, `v0 v1 v2 v1 v2 v3`, from its argument's 0x70 bytes.
const QUAD_BYTES: usize = 0xa8;

/// The run's own numbers, which in the real game are 0xc8 bytes off the allocator and so in none of
/// [`DATA`]: a base of this file's choosing, the length the game's own.
const RUN_NUMBERS: Range<usize> = 0x0320_0000..0x0320_00c8;

/// And what the game puts over a run — the dialogue and the plate a card's name is shown on — which is one
/// object allocated with a stage in the real game and so in none of [`DATA`] either: a base of this file's
/// choosing, long enough to reach the dialogue's line at its far end.
const GUI: Range<usize> = 0x0330_0000..0x0332_0000;

/// The speed struct the player points at, which in the real game is off the allocator and so is in none of
/// [`DATA`]: a base of this file's choosing, long enough to hold the two fields orb reads.
const PLAYER_SPEEDS: Range<usize> = 0x0340_0000..0x0340_0100;

/// The ending's own object, allocated with the scene in the real game: a base of this file's choosing, long
/// enough to reach its script.
const ENDING: Range<usize> = 0x0334_0000..0x0334_3000;

/// And the chain element that carries it, which the real one allocates too — laid out inside [`DATA`] because
/// what a walk of the chain follows is where the game keeps its own elements.
const ENDING_ELEMENT: usize = 0x0049_ee00;

/// The front end's own object, allocated with its scene in the real game, and the chain element that carries
/// it — the only place its address is kept, so a walk of the chain is the only way to it.
const FRONT_END: Range<usize> = 0x0335_0000..0x0336_0000;
const FRONT_END_ELEMENT: usize = 0x0049_ee40;

/// The screen a run ends on, allocated with its scene in the real game, and the element that carries it.
const RESULT: Range<usize> = 0x0337_0000..0x0337_1000;
const RESULT_ELEMENT: usize = 0x0049_ee80;

/// The replay's own manager and header, allocated in the real game: a base of this file's choosing.
const REPLAY: Range<usize> = 0x0336_0000..0x0336_1000;

/// The element the panel's draw job hangs off, which the real game allocates with the panel — laid out inside
/// [`DATA`] for the reason [`ENDING_ELEMENT`] is.
const PANEL_ELEMENT: usize = 0x0049_eec0;

/// A sprite of this file's choosing, and the texture the slot it names holds: the real ones are inside the
/// sprite manager, which this file lays out only as far as the frame needs it.
const A_SPRITE: usize = 0x0338_0000;
const A_TEXTURE: usize = 0x0338_1000;

/// The music's own two objects, which the real game allocates as a track starts: the stream and the source it
/// reads through. Bases of this file's choosing, the lengths the game's own.
const STREAM: usize = 0x033a_0000;
const SOURCE: usize = 0x033a_1000;
/// And where the pair the game makes for the *next* track lands, which is not where the last pair was: a track
/// change deletes both and allocates both again — see [`Image::streams_it_again_elsewhere`].
const ANOTHER_STREAM: usize = 0x033a_8000;
const ANOTHER_SOURCE: usize = 0x033a_9000;

/// The score-file object a save allocates, laid out where that allocation would be: a base of this file's
/// choosing, and the one field of it anything reads is the phase at `super::RESULT_PHASE` — which is what says
/// whether a write follows the read made on it.
const SCORE_SCREEN: Range<usize> = 0x033a_6000..0x033a_7000;

/// The image of the score file the game's own save reads, laid out where that save would have allocated it: a
/// base of this file's choosing, long enough for a chunk per card.
const SCORE_FILE_IMAGE: usize = 0x033a_5000;
/// The two fields the parse walks that image by: where its chunks begin and where the file ends — `[image+8]`
/// and `[image+0x10]`, read at 0x44508b and 0x44509d.
const FILE_CHUNKS_AT: usize = 0x8;
const FILE_BYTES: usize = 0x10;
/// And where the chunks are put behind those, which is any offset past them.
const FILE_HEADER_BYTES: usize = 0x18;
/// And which card a chunk is for, at `+0x28` of it — the word the parse looks a chunk's record up by (0x4450c6).
const CHUNK_CARD: usize = 0x28;

/// **The score files themselves, as bytes on a disk a laid-out game has none of**: one 0x78-byte `CATK` chunk per
/// card that has counts, which is what the game's own write leaves and its own parse walks.
///
/// Here rather than in the address space, because a file is not memory: what a save has to be able to say is that
/// the counts reached something that outlives the process, and a range of the game's own memory says nothing of
/// the kind.
///
/// **Keyed by name, because there are two of them and which one an open lands in is orb's decision**: a session
/// orb can rewind sends every open but one to `pointdevice_score.dat`, and the one it does not send is the read
/// the front end's unlocks come out of. A laid-out game with a single file cannot tell what that read costs — see
/// `Image::reads_its_card_records_out_of_the_file`.
static SCORE_FILES: Mutex<Vec<(String, Vec<u8>)>> = Mutex::new(Vec::new());

/// The bytes of the file of that name, and where to put them back.
fn score_file(name: &str) -> Vec<u8> {
    SCORE_FILES
        .lock()
        .unwrap()
        .iter()
        .find(|(held, _)| held == name)
        .map(|(_, bytes)| bytes.clone())
        .unwrap_or_default()
}

fn set_score_file(name: &str, bytes: Vec<u8>) {
    let mut held = SCORE_FILES.lock().unwrap();
    match held.iter_mut().find(|(held, _)| held == name) {
        Some((_, held)) => *held = bytes,
        None => held.push((name.to_owned(), bytes)),
    }
}

/// And the records of the tracks inside `thbgm.dat`, which are not a track's: the player reads the file's own
/// table once and keeps the array, and what a source is pointed at is one of its entries.
///
/// A base of this file's choosing again, laid out from the entry the player's own pointer field says: `+0x738`
/// of it, walked 0x34 bytes at a time by the lookup at 0x44bc01 and handed to the source at 0x44c1f1.
const MUSIC_RECORDS: usize = 0x033a_2000;
const SOUND_RECORDS: usize = 0x738;
const RECORD_STRIDE: usize = 0x34;

/// And the block the game's own music-table pointer points at, whose two names are the songs the stage that was
/// just built has: a base of this file's choosing again, the offsets the game's — see `Th07`'s `G_MUSIC_TABLE`.
const STAGE_SONGS: usize = 0x033a_4000;

/// The phase the title menu's own items are read in, which is what a screen of this game's front end is doing
/// rather than which screen it is: **1**, and this file's own number rather than `Th07`'s.
///
/// Read off the game's own update of that screen: 0x4555dd branches on `[this+8]` at 0x4555f8 — 0 to 0x455621,
/// which is the frame the screen arrives on (it starts the title's own song from slot 8 and sets 1), and 1 to
/// 0x45599d, which is where the cursor moves and the decide is read. So 1 is the phase a press is acted on in,
/// and a launch of 妖々夢 says the same thing: `front end: object 0x0a4a2bf8, screen Some(0), item Some(2),
/// phase Some(1), up for Some(84) frame(s)`, on a title menu nobody could get orb's question out of.
///
/// **Its own number and not the constant orb reads with**, which is the whole point of it being here: a laid-out
/// game that wrote the phase orb expects would agree with orb about a value neither of them had checked, and
/// that is exactly what let this through — see
/// [docs/adr/0008](../../../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md).
const TITLE_ITEMS_PHASE: i32 = 1;

/// The scenes this game's supervisor moves between, as the exe writes them, and **its own numbers for the same
/// reason [`TITLE_ITEMS_PHASE`] is**: a laid-out game that wrote the scene orb reads with would agree with orb
/// about a number neither of them had checked. An address has to be shared — both sides must mean the same word —
/// and a value must not, which is [docs/adr/0008](../../../../../docs/adr/0008-the-fake-game-copies-the-game-orb-is-injected-into.md)'s
/// own line.
///
/// Read off the switch at 0x437fbd and the writes that feed it: 1 is the front end (0x437ee3 is the case for
/// leaving it), 2 the run and 3 the run built again (0x42f3c5 registers the manager for 2, and its added callback
/// branches at 0x42e894 on 3), 5 the ranking (`[0x575aa8] = 5` at 0x455f95), 6 the screen a run ends on (0x403b8d,
/// 0x403bfe and 0x42a6ab write it) and 9 an ending (0x41e820 registers it).
const SCENE_FRONT_END: i32 = 1;
const SCENE_RUN: i32 = 2;
const SCENE_RUN_AGAIN: i32 = 3;
const SCENE_RANKING: i32 = 5;
const SCENE_RUN_OVER: i32 = 6;
const SCENE_ENDING: i32 = 9;

/// And the bits of [`G_RUN_KIND`](super::G_RUN_KIND) that say what kind of run it is: bit 0 a practice run, bit 1
/// the attract demo, bit 3 a replay — 0x44a372 and 0x44a37e write the first, 0x455b5e the second, 0x455b50 and
/// 0x45b395 the third. Its own numbers, as above.
const RUN_IS_A_PRACTICE: u32 = 1 << 0;
const RUN_IS_A_DEMO: u32 = 1 << 1;
const RUN_IS_A_REPLAY: u32 = 1 << 3;

/// And what the stage word holds while a stage is played: one more than orb's own 0-based number, the build at
/// 0x42ef50 adding it and the teardown at 0x43805f taking it back off. Its own number, as above.
const STAGE_COUNTS_FROM: i32 = 1;

/// The front end's own screens and items a laid-out game has to be able to be on, and the frames it ignores a
/// press for. Its own numbers, as above.
///
/// The title is screen 0 — the byte table at 0x4555ce sends it to the title's own handler — and a whole run's
/// shot select is 6, that item writing 4 at 0x455d20 and the three screens of its flow being 4, 5 and 6. Its
/// items are dispatched in phase 1 (0x455621's block, with 0x45599d the cursor's), the item that starts a run is
/// the first of the eight, and 0x455c6d compares the frames the screen has been up against 0xa and does nothing
/// under it.
const SCREEN_TITLE: i32 = 0;
const SCREEN_SHOT_SELECT: i32 = 6;
const SHOT_SELECT_PHASE_ITEMS: i32 = 1;
const TITLE_ITEM_START: i32 = 0;
const SETTLED_FRAMES: i32 = 0xa;

/// And the phases of the screen a run ends on: 0xa is the one a whole run's end brings it up in, and 2 is the one
/// its leaving runs in. Its own numbers, as above.
const RESULT_FROM_A_RUN: i32 = 0xa;
const RESULT_LEAVING: i32 = 2;

/// Where the game's own counting of a card stops: 0x4100eb and 0x41011c compare against 0x270f and skip the
/// increment, so a card attempted ten thousand times stays at 9999. Its own number, as above.
const CARD_COUNT_CAP: u16 = 0x270f;

/// And the two colours a card's name is baked in, which the baking here holds what it was given against —
/// `0xfff0f0` pushed at 0x428a61 and the zero in front of it at 0x428a5f. **Its own numbers is what makes that
/// assertion able to fail at all**: held against orb's own constants it compared each with itself.
const CARD_NAME_COLOUR: u32 = 0x00ff_f0f0;
const CARD_NAME_SHADOW: u32 = 0;

/// And a value in that word which is not zero, which is all "the game has asked for fog" is: the field is
/// compared against zero, so what a laid-out game writes there only has to be something. **Not orb's own
/// number here either**, so that a launch where orb wrote one number and read another would show up: this is
/// what the *game* would be holding, and 0xff is what orb's frame writes.
const FOG_ON_FORCED: u32 = 0xff;

/// The name loaded into each of the player's own slots, `+0x28bc` of it and 0x100 bytes apiece: the buffer the
/// load's command copies a name into at 0x44c242 and the play's reads back at 0x44cc95.
///
/// Here rather than beside the calls that take a slot, because orb names a slot and never reads one: which slot
/// is a stage's music comes out of the stage's own data — see `Th07`'s `G_MUSIC_TABLE`. What this is for is a
/// laid-out game answering a play the way the player's own thread answers one.
const SLOT_NAMES: usize = 0x28bc;
const SLOT_NAME_BYTES: usize = 0x100;
/// And the wrapper the stream keeps its sound buffer behind, whose first word is the buffer DirectSound
/// made — the one word orb reads rather than calls.
const BUFFER_WRAPPER: usize = 0x033a_3000;
/// その他の記録 as the file and the block hold them — see `Th07`'s `G_PLAY_RECORDS` for what is in there and
/// which of the game's own code reads and writes it. Here rather than in the parent because only a laid-out game
/// needs the layout: orb hands the block's address to the game's own parse and the game's own write takes it
/// from there.
const PLAY_RECORDS_BYTES: usize = 0x160;
/// `"PLST"` as a little-endian dword, which the block is headed with (0x438d51) and the game's own parse matches
/// (0x4454b7) — the same shape as `CARD_RECORD_TAG`.
const PLAY_RECORDS_TAG: u32 = u32::from_le_bytes(*b"PLST");

/// How many buffers that wrapper is the array of, at `+0x10` of the stream: what the game's own take-down of a
/// stream is bounded by — 0x45d860 walks `i < [stream+0x10]`, reads `[[stream+0x4] + i*4]` and calls
/// `IDirectSoundBuffer::Stop` through it (slot 18, `+0x48` of the vtable), and the play command's first phase
/// is that call, 0x44cc23.
///
/// Here rather than beside the stream's other offsets in the parent, because orb neither reads nor writes it:
/// what it is for is a laid-out game taking a stream down the way the real one does.
const STREAM_BUFFER_COUNT: usize = 0x10;
/// The slot that texture is in, which is nothing but a number orb has to carry from the sprite to the array.
const PANEL_TEXTURE_SLOT: u32 = 0x15;
/// `front.anm`'s own 32x32 tile as a piece of its 256x256 sheet: sprite 12, at (0, 224).
const PANEL_TILE_UV: [f32; 4] = [0.0, 224.0 / 256.0, 32.0 / 256.0, 1.0];

/// Where the enemy manager's own slots begin, which the parent names now that orb walks them itself.
use super::ENEMIES;

/// Which pad button is which as the game's own defaults have it, which is what a launch with nobody's
/// `th07.cfg` in it reads: the 18 bytes at 0x49ee40, copied to the mapping at 0x4399d3.
///
/// Laid out because zeros are not neutral here — a table of them says every bit of the word is button 0 —
/// and because this is the table a launch really has until somebody configures the game.
const PAD_MAPPING: [i16; 9] = [0, 1, 2, 4, -1, -1, -1, -1, 3];

/// A track as `thbgm.dat`'s own record describes it: its own name, where its sound starts in the file, where it
/// loops back to, and how long it is — the three the game's own seek is built out of and the three orb tells
/// one track from another by, with the name the player looks a loaded slot up by.
#[derive(Clone, Copy)]
pub struct Track {
    pub name: &'static CStr,
    pub at: u32,
    pub loops_at: u32,
    pub bytes: u32,
}

pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    /// In a host whose non-determinism is drawn from `seed` — the wake delays and the compositor's
    /// spikes — which an e2e test names in its assertions so that a failure can be replayed. The one
    /// constructor, for the reason `th06`'s says.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        sim.space().map(DATA.start, DATA.len(), Kind::Private);
        sim.space()
            .map(QUAD_QUEUE.start, QUAD_QUEUE.len(), Kind::Private);
        sim.space()
            .write::<usize>(super::G_QUAD_QUEUE, QUAD_QUEUE.start);
        sim.space()
            .write::<[i16; 9]>(super::PAD_MAPPING, PAD_MAPPING);
        Self { sim }
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is
    /// what makes `Th07`'s reads land in it.
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

    /// The object the game calls its own whole frame on, which orb's loop is handed and hands the frame
    /// back to on each of the three ways out of it that return.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// The front end's own object, which is what its callbacks are called on: an e2e test running one of them
    /// hands this over the way the chain hands it over.
    pub fn front_end_object(&self) -> usize {
        FRONT_END.start
    }

    /// And the game manager's, which is a static of the game's own rather than an allocation: the object its
    /// added callback is called on, and the one every field of a stage's own clock is read out of.
    pub fn game_manager_object(&self) -> usize {
        super::G_GAME_MANAGER
    }

    /// The chain the update and the draw are called on. Nothing is laid out at it — an e2e test's own
    /// two functions are what orb calls, and this is the argument they are called with — so what it has
    /// to be is an address inside the space, which is what a real one is.
    pub fn chain_object(&self) -> usize {
        super::G_CHAIN
    }

    /// The whole-output viewport the game keeps in one struct and orb writes as it prepares a frame,
    /// for an e2e test that reads back what orb put there.
    pub fn viewport(&self) -> [u32; 4] {
        let space = self.space();
        [0, 1, 2, 3].map(|word| space.read::<u32>(super::VIEWPORT + word * 4))
    }

    /// The three things the game's own frame does to the queue of quads, as the exe does them: the
    /// queue emptied at 0x44f580, a quad added at 0x44f690, and the queue drawn at 0x44f5c0.
    ///
    /// **Here rather than in whatever drives the game, because the block is here.** They are the game's
    /// own code and a laid-out game has to be able to answer them — code being the one thing an address
    /// space cannot hold — and what each is made of is the fields above, which nothing outside this file
    /// knows the offsets of.
    ///
    /// A quad added is the whole of why any of this is laid out: it writes [`QUAD_BYTES`] where
    /// [`QUEUE_WRITE_AT`] points, so a frame whose queue was never emptied writes to address zero, and
    /// one that emptied it and never drew walks up the buffer until it runs off the end. Both are what
    /// the real game does, which is the point — the first of them is the fault that took 妖々夢 down.
    pub fn empties_the_queue(&self) {
        let space = self.space();
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, 0);
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_WRITE_AT,
            QUAD_QUEUE.start + QUEUE_BUFFER,
        );
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_DRAW_FROM,
            space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT),
        );
    }

    pub fn queues_a_quad(&self) {
        let space = self.space();
        let at = space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT);
        // The vertices themselves, which nothing reads: what a quad *is* is the bytes it costs the
        // buffer and the address they went to.
        space.fill_bytes(at, 0xff, QUAD_BYTES);
        space.write::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT, at + QUAD_BYTES);
        let queued = space.read::<u32>(QUAD_QUEUE.start + QUEUED);
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, queued + 1);
    }

    pub fn draws_the_queue(&self) {
        let space = self.space();
        // Nothing at all where there is nothing queued, which is what makes the second of the frame's two
        // calls the game's own no-op: the count is what 0x44f5d1 returns on.
        if space.read::<u32>(QUAD_QUEUE.start + QUEUED) == 0 {
            return;
        }
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_DRAW_FROM,
            space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT),
        );
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, 0);
        let drawn = space.read::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS);
        space.write::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS, drawn + 1);
    }

    /// Where the next quad would go, for an e2e test that reads it back: the start of the buffer on a
    /// frame whose queue was emptied, and one quad further on for every quad drawn since it was not.
    pub fn queue_writes_at(&self) -> usize {
        self.space()
            .read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT)
    }

    /// The start of that buffer, and what one quad costs it: what the answer above is worth reading
    /// against.
    pub fn queue_buffer(&self) -> usize {
        QUAD_QUEUE.start + QUEUE_BUFFER
    }

    pub fn queue_bytes_per_quad(&self) -> usize {
        QUAD_BYTES
    }

    /// And how many quads are in it now, and how many times the game has drawn it.
    pub fn queued(&self) -> u32 {
        self.space().read::<u32>(QUAD_QUEUE.start + QUEUED)
    }

    pub fn queue_draws(&self) -> u32 {
        self.space().read::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS)
    }

    /// The scene the game is in and the one it has been asked for, as its supervisor keeps them: an e2e
    /// test writing the pair the game's own transition would have left there.
    pub fn is_in_scene(&self, scene: i32, wanted: i32) {
        let space = self.space();
        space.write::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_SCENE, scene);
        space.write::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_WANTED, wanted);
    }

    /// The pair read back, which is how a laid-out game does the half of a scene change that is the game's:
    /// orb asks for a screen by writing `wanted`, and it is the supervisor that then builds it.
    pub fn the_scene_now(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_SCENE)
    }

    pub fn the_scene_wanted(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_WANTED)
    }

    /// And two of them by what they are rather than by their numbers: the ranking orb asks for after a run has
    /// ended, and the front end every screen goes back to.
    pub fn scene_of_the_ranking(&self) -> i32 {
        SCENE_RANKING
    }

    pub fn scene_of_the_front_end(&self) -> i32 {
        SCENE_FRONT_END
    }

    /// And the one a run's own teardown asks for, which is where its objects go: what `Th07::leave_run` writes,
    /// and the scene a run given up passes through on its way back to the title.
    pub fn scene_of_a_run_over(&self) -> i32 {
        SCENE_RUN_OVER
    }

    /// The stage a transition is building, taken as orb counts stages and written without the scene: what a
    /// transition changes about the scene is the one *asked for* — see
    /// [`the_next_stage_is_being_asked_for`](Image::the_next_stage_is_being_asked_for).
    pub fn the_stage_being_rebuilt_is(&self, stage: i32) {
        self.space()
            .write::<i32>(super::G_STAGE, stage + STAGE_COUNTS_FROM);
    }

    /// The next stage asked for, which is the whole of what a transition does to the supervisor: the scene stays
    /// the one a run is played in and the rebuild's goes in the field beside it, which is what the game manager's
    /// added callback reads to tell a transition from a run's first stage.
    ///
    /// **And no frame is spent that way.** The supervisor does the whole transition inside one update — 0x4380b4
    /// writes the rebuild scene into `wanted`, 0x42f45d tears the stage down, 0x42f3c5 registers the manager for
    /// the next one, and 0x4380df puts `wanted` back to the run's own scene before the update returns — so the
    /// pair orb reads is never the rebuild's. That the two are read at all is [`Th07::stage_frame`]'s doing and
    /// deliberate: a field of a run in progress belongs to the run either way.
    pub fn the_next_stage_is_being_asked_for(&self) {
        self.is_in_scene(SCENE_RUN, SCENE_RUN_AGAIN);
    }

    /// And the scene a stage is rebuilt in, as a scene the game has arrived in: what nothing but a reader of
    /// `Th07::stage_frame` should be shown, no frame of a real transition being spent there.
    pub fn the_stage_is_being_rebuilt(&self) {
        let again = SCENE_RUN_AGAIN;
        self.is_in_scene(again, again);
    }

    /// And the scene a run is, so an e2e test says "a run is in progress" in the game's own terms rather
    /// than in a number of its own.
    pub fn scene_of_a_run(&self) -> i32 {
        SCENE_RUN
    }

    /// A run in progress as the game itself would have left it: the scene, the stage it is on counted from
    /// zero, and the difficulty in the game manager.
    ///
    /// The stage goes in the way the game's own build leaves it, which is one more than the number given
    /// here — so a test that asks for the first stage gets what orb would read on the first stage.
    pub fn is_playing(&self, stage: i32, difficulty: i32) {
        let run = SCENE_RUN;
        self.is_in_scene(run, run);
        let space = self.space();
        space.write::<i32>(super::G_STAGE, stage + STAGE_COUNTS_FROM);
        space.write::<i32>(super::G_GAME_MANAGER + super::DIFFICULTY, difficulty);
    }

    /// The game's own stage word, written as the game writes it and not as orb counts it: the number its
    /// build leaves there, which is one for the first stage.
    ///
    /// Beside [`is_playing`](Image::is_playing) rather than inside it because the two say different things:
    /// that one takes the stage orb counts and this one takes the game's, so a test can ask what orb answers
    /// for a word the game really holds.
    pub fn the_games_stage_word_is(&self, word: i32) {
        self.space().write::<i32>(super::G_STAGE, word);
    }

    /// The run's own numbers laid out, which is what a run being built leaves behind: a block of the size
    /// the game allocates, with the pointer to it where the game keeps it.
    ///
    /// Apart from [`is_playing`](Image::is_playing) because a run's *numbers* and the scene it is in are two
    /// things: orb reads every field of these through that pointer, and a null one is what it finds before
    /// any run has been built.
    pub fn has_a_runs_numbers(&self) {
        let space = self.space();
        // Laid out only where it is not already: a launch that builds a second stage — a transition, or the run
        // a resume starts again — allocates a block of its own in the real game and has one of these here, an
        // address space having one of each.
        if space.has_room(RUN_NUMBERS.start, RUN_NUMBERS.len()) {
            space.map(RUN_NUMBERS.start, RUN_NUMBERS.len(), Kind::Private);
        }
        space.write::<usize>(super::G_RUN_NUMBERS, RUN_NUMBERS.start);
    }

    /// And the power that run's shot is at, which the game keeps as a float.
    pub fn shot_is_at_power(&self, power: f32) {
        self.space()
            .write::<f32>(RUN_NUMBERS.start + super::POWER, power);
    }

    /// The lives and the bombs in hand, floats as well: what the game's own two guarded adds would have left
    /// behind after a bomb or an extend.
    pub fn has_lives_and_bombs(&self, lives: f32, bombs: f32) {
        let space = self.space();
        space.write::<f32>(RUN_NUMBERS.start + super::LIVES, lives);
        space.write::<f32>(RUN_NUMBERS.start + super::BOMBS, bombs);
    }

    /// And how many times the player has died in the run, which the game counts up as a float of its own
    /// rather than down from the lives.
    pub fn has_died(&self, deaths: f32) {
        self.space()
            .write::<f32>(RUN_NUMBERS.start + super::DEATHS, deaths);
    }

    /// The score and the point items collected, which the game keeps as whole numbers where the rest of a
    /// run's are floats.
    pub fn the_run_has_scored(&self, score: u32, point_items: u16) {
        let space = self.space();
        space.write::<u32>(RUN_NUMBERS.start + super::SCORE, score);
        space.write::<u32>(
            RUN_NUMBERS.start + super::POINT_ITEMS,
            u32::from(point_items),
        );
    }

    /// And the number the panel draws, `[run+0x0]`, which the game's own teardown and every way out of its
    /// pause menu catch up with the score above — see `Th07`'s own `SCORE_SHOWN`.
    pub fn the_panel_shows(&self, score: u32) {
        self.space()
            .write::<u32>(RUN_NUMBERS.start + super::SCORE_SHOWN, score);
    }

    pub fn the_score_the_panel_shows(&self) -> u32 {
        self.space()
            .read::<u32>(RUN_NUMBERS.start + super::SCORE_SHOWN)
    }

    /// How far the point value has climbed and how many items the next step of it wants, which is 妖々夢's own
    /// 点 system: 0x4333bd compares the count against the second of these and, where it has reached it, raises
    /// the first and works the second out again — see `Th07`'s own `SCORE`, where the arithmetic is written down.
    pub fn the_point_value_has_climbed(&self, step: u32, next: u32) {
        let space = self.space();
        space.write::<u32>(RUN_NUMBERS.start + super::POINT_VALUE_STEP, step);
        space.write::<u32>(RUN_NUMBERS.start + super::POINT_VALUE_NEXT, next);
    }

    pub fn the_point_value_it_has_climbed(&self) -> (u32, u32) {
        let space = self.space();
        (
            space.read::<u32>(RUN_NUMBERS.start + super::POINT_VALUE_STEP),
            space.read::<u32>(RUN_NUMBERS.start + super::POINT_VALUE_NEXT),
        )
    }

    /// Where the run's own numbers are, for a test that reads a field back rather than through `Th07`.
    pub fn run_numbers_at(&self) -> usize {
        RUN_NUMBERS.start
    }

    /// And the seed the stage began with, in the copy the game's own build leaves in its manager — beside the
    /// generator's own word, which is what [`generator_is_at`](Image::generator_is_at) writes.
    pub fn the_stage_began_with_the_seed(&self, seed: u16) {
        self.space()
            .write::<u16>(super::G_GAME_MANAGER + super::SEED_AT_STAGE_START, seed);
    }

    /// How long the stage has been running, as the game's own update leaves it: the count the register puts
    /// back to nothing at every stage and the update adds one to.
    pub fn has_been_in_the_stage_for(&self, frames: u32) {
        self.space()
            .write::<u32>(super::G_GAME_MANAGER + super::STAGE_FRAMES, frames);
    }

    /// How many enemies are alive, as their manager's own walk of its slots left the count.
    pub fn enemies_alive(&self, enemies: i32) {
        self.space()
            .write::<i32>(super::G_ENEMY_MANAGER + super::ENEMY_COUNT, enemies);
    }

    /// And how many bullets, which their own manager counts the same way.
    pub fn bullets_alive(&self, bullets: i32) {
        self.space()
            .write::<i32>(super::G_BULLET_MANAGER + super::BULLET_COUNT, bullets);
    }

    /// The lasers, which nothing counts: the first `lasers` of the game's own slots marked alive and the rest
    /// left dead, so that what orb answers is a walk of the same slots the game walks.
    pub fn lasers_alive(&self, lasers: usize) {
        let space = self.space();
        for slot in 0..super::LASER_SLOTS {
            let alive = u32::from(slot < lasers);
            space.write::<u32>(
                super::G_LASERS + slot * super::LASER_BYTES + super::LASER_ALIVE,
                alive,
            );
        }
    }

    /// A boss in the first of the manager's slots, with the life and the attack frames it would have: the
    /// enemy is one of the manager's own slots, so this hands out the address the game would have put there.
    ///
    /// **Alive as well as pointed at**, an enemy being what a boss is: the slot says which of them the fight is
    /// with and the alive bit says it is still there, and [`Image::a_boss_slot_leads_to`] is where those two
    /// come apart.
    pub fn a_boss_is_fighting(&self, life: i32, attack_frames: i32) {
        let boss = super::G_ENEMY_MANAGER + ENEMIES;
        let space = self.space();
        space.write::<usize>(super::G_ENEMY_MANAGER + super::BOSSES, boss);
        space.write::<u8>(boss + super::ENEMY_ALIVE, super::ENEMY_ALIVE_BIT);
        space.write::<i32>(boss + super::ENEMY_LIFE, life);
        space.write::<i32>(boss + super::ENEMY_ATTACK_FRAMES, attack_frames);
    }

    /// One of the manager's eight boss slots pointed at one of its enemy slots, and nothing else — no life, no
    /// attack frames and not a word about whether that enemy is alive.
    ///
    /// Which is the state 妖々夢's 4面 leaves behind: three bosses share the stage, the slots are written as
    /// each arrives, and nothing in the exe writes one back to zero — so a slot outlives the boss it was for.
    pub fn a_boss_slot_leads_to(&self, slot: usize, enemy: usize) {
        self.space().write::<usize>(
            super::G_ENEMY_MANAGER + super::BOSSES + slot * size_of::<usize>(),
            super::G_ENEMY_MANAGER + ENEMIES + enemy * super::ENEMY_BYTES,
        );
    }

    /// And an enemy that is no longer there, which is what its own walk reads off the same bit.
    pub fn the_enemy_is_gone(&self, slot: usize) {
        self.space().write::<u8>(
            super::G_ENEMY_MANAGER + ENEMIES + slot * super::ENEMY_BYTES + super::ENEMY_ALIVE,
            0,
        );
    }

    /// And no boss at all, which is what the manager's own walk of its slots leaves where the fight is over:
    /// the pointer it answers with is nothing.
    pub fn no_boss_is_fighting(&self) {
        self.space()
            .write::<usize>(super::G_ENEMY_MANAGER + super::BOSSES, 0);
    }

    /// And no card up, which is the flag the manager clears where an attack ends.
    pub fn no_spellcard_is_running(&self) {
        self.space()
            .write::<u32>(super::G_ENEMY_MANAGER + super::SPELLCARD_RUNNING, 0);
    }

    /// And the spellcard that boss is on, which the manager keeps as a flag and a number.
    pub fn the_spellcard_is(&self, card: u32) {
        let space = self.space();
        space.write::<u32>(super::G_ENEMY_MANAGER + super::SPELLCARD_RUNNING, 1);
        space.write::<u32>(super::G_ENEMY_MANAGER + super::SPELLCARD, card);
    }

    /// The player in the state a run is played in: 0, the one the respawn puts it back to at 0x4413fa — and so
    /// settled, which is where a chapter can begin.
    pub fn the_player_is_playing(&self) {
        self.space()
            .write::<i8>(super::G_PLAYER + super::PLAYER_STATE, 0);
    }

    /// And the player as their own register leaves them, which is what a stage's build runs: 0x4429d0, called
    /// from a run's build at 0x42e9d6 and from *every* stage's at 0x42ef03. It zeroes 0x2df9e dwords from the
    /// object's head — so the state is that same 0 — and then writes the timer at `+0x16a00` outright, `[+8]`
    /// and `[+4]` to nothing and `[+0]` to -999.
    ///
    /// **Only the frames of that timer are laid out here**, `[+8]` being the word orb reads and `--clear`
    /// writes. The other two, and the rest of what the register zeroes, are the shot, the position, the bomb's
    /// own object and the three chain elements the register makes again on the instructions after — a laid-out
    /// game that zeroed those would be taking its own chain apart to model writes nothing reads.
    pub fn its_player_is_registered_again(&self) {
        self.the_player_is_playing();
        self.space()
            .write::<i32>(super::G_PLAYER + super::PLAYER_COMING_BACK_FRAMES, 0);
    }

    /// **The player hit and not yet dead**, which is the state the hit test leaves at 0x43ee0e: the death
    /// handler is called out of the *update* that finds it, at 0x441fee, so this is what stands for the
    /// updates in between — and a bomb pressed in them is 妖々夢's 決死結界 rather than a bomb spent.
    pub fn the_player_has_been_hit(&self) {
        self.space()
            .write::<i8>(super::G_PLAYER + super::PLAYER_STATE, 2);
    }

    /// The player in the middle of dying, which is the state its own death handler sets: not the state a run
    /// is played in, and so unsettled.
    pub fn the_player_is_dying(&self) {
        self.space()
            .write::<i8>(super::G_PLAYER + super::PLAYER_STATE, 1);
    }

    /// The player's own state and the frames left under it, read back: which of the five the player is in, and
    /// what the timer the state's handler ticks holds.
    pub fn the_players_state(&self) -> i8 {
        self.space()
            .read::<i8>(super::G_PLAYER + super::PLAYER_STATE)
    }

    pub fn the_frames_the_state_has_left(&self) -> i32 {
        self.space()
            .read::<i32>(super::G_PLAYER + super::PLAYER_COMING_BACK_FRAMES)
    }

    /// And the player where the game's own respawn has put them, coming back with the frames it gave them.
    pub fn the_player_is_coming_back(&self, frames: i32) {
        let space = self.space();
        space.write::<i8>(super::G_PLAYER + super::PLAYER_STATE, 3);
        space.write::<i32>(super::G_PLAYER + super::PLAYER_COMING_BACK_FRAMES, frames);
    }

    /// And the player behind 妖々夢's border, which is the state after that one.
    pub fn the_players_border_is_up(&self) {
        self.space()
            .write::<i8>(super::G_PLAYER + super::PLAYER_STATE, 4);
    }

    /// A bomb going off, which is the flag the player's update raises as it takes the bomb off the count.
    pub fn a_bomb_is_going_off(&self) {
        self.space()
            .write::<u32>(super::G_PLAYER + super::BOMB_IN_USE, 1);
    }

    /// And the bomb over, which is the same flag lowered where its own frames run out.
    pub fn no_bomb_is_going_off(&self) {
        self.space()
            .write::<u32>(super::G_PLAYER + super::BOMB_IN_USE, 0);
    }

    /// What the game puts over a run in place, as a stage's load leaves it: the object and the pointer to it
    /// where the game keeps it.
    ///
    /// Laid out only where it is not already: the dialogue and the panel are two things inside this one object,
    /// and a test that asks for both would otherwise lay it out twice.
    /// The dialogue's own line goes in with it, at the -1 the game's own load writes (0x42998a): zero is a
    /// *line being played* to every reader of that field, orb's own among them, so an object laid out and left
    /// at nothing would be a stage that has a dialogue up on every frame of it.
    pub fn has_what_it_puts_over_a_run(&self) {
        let space = self.space();
        if space.has_room(GUI.start, GUI.len()) {
            space.map(GUI.start, GUI.len(), Kind::Private);
        }
        space.write::<usize>(super::G_GUI + super::GUI_IMPL, GUI.start);
        self.the_dialogue_is_at_line(-1);
    }

    /// The same object, named for the half of it a test about dialogues is about.
    pub fn has_a_dialogue(&self) {
        self.has_what_it_puts_over_a_run();
    }

    /// And which line of it is being played, the game's own numbering: -1 for none, -2 for the one it hands
    /// the screen over on, and a whole number for a line.
    pub fn the_dialogue_is_at_line(&self, line: i32) {
        self.space()
            .write::<i32>(GUI.start + super::DIALOGUE_LINE, line);
    }

    /// The vm a card's name is baked into, which is what the baking has to be given: the object above and the
    /// offset the game reaches the plate at.
    pub fn card_name_vm(&self) -> usize {
        GUI.start + super::CARD_NAME_VM
    }

    /// The panel as the game's own build leaves it: the object it is inside, the vm the background grid is
    /// drawn with, and the sprite that vm's script put on it — a texture slot and the piece of the sheet that
    /// tile is, which is what orb reads the tile out of rather than writing down.
    ///
    /// The numbers are `front.anm`'s own for its 32x32 tile: sprite 12, at (0, 224) of a 256x256 sheet, so the
    /// piece of it is 0.0 to 0.125 across and 0.875 to 1.0 down.
    pub fn has_a_panel(&self) {
        self.has_what_it_puts_over_a_run();
        let space = self.space();
        // Laid out only where it is not already, for the reason above: a test that takes the sprite off the vm
        // and puts the panel back asks for this one twice.
        let sprite = size_of::<usize>() * 16;
        if space.has_room(A_SPRITE, sprite) {
            space.map(A_SPRITE, sprite, Kind::Private);
        }
        space.write::<usize>(
            GUI.start + super::PANEL_TILE_VM + super::VM_SPRITE,
            A_SPRITE,
        );
        space.write::<u32>(A_SPRITE + super::SPRITE_TEXTURE, PANEL_TEXTURE_SLOT);
        for (at, value) in super::SPRITE_UV.into_iter().zip(PANEL_TILE_UV) {
            space.write::<f32>(A_SPRITE + at, value);
        }
        space.write::<usize>(
            QUAD_QUEUE.start
                + super::ANM_TEXTURES
                + PANEL_TEXTURE_SLOT as usize * size_of::<usize>(),
            A_TEXTURE,
        );
    }

    /// The tile that panel is laid with, as a test says what orb should have answered.
    pub fn the_panels_tile(&self) -> (usize, [f32; 4]) {
        (A_TEXTURE, PANEL_TILE_UV)
    }

    /// And the panel's own sprite left unread, which is a vm whose script has not run yet.
    pub fn the_panels_tile_has_no_sprite(&self) {
        self.space()
            .write::<usize>(GUI.start + super::PANEL_TILE_VM + super::VM_SPRITE, 0);
    }

    /// The panel's draw job on the draw list, which is the game painting its rows this frame — hung off the
    /// draw list's own head the way the game's own register hangs it.
    pub fn paints_its_panel(&self) {
        let space = self.space();
        space.write::<usize>(PANEL_ELEMENT + super::CHAIN_CALLBACK, super::PANEL_DRAW);
        space.write::<usize>(PANEL_ELEMENT + super::CHAIN_ARGUMENT, GUI.start);
        space.write::<usize>(
            super::G_CHAIN + super::CHAIN_DRAW_LIST + super::CHAIN_NEXT,
            PANEL_ELEMENT,
        );
    }

    /// And the two the panel keeps its own asking in: which rows are to be painted, and how long the ground
    /// under them has left to be laid again.
    pub fn the_rows_to_paint(&self) -> u32 {
        self.space().read::<u32>(super::G_GUI + super::GUI_ROWS)
    }

    pub fn the_panels_ground_is_laid_for(&self) -> u32 {
        self.space().read::<u32>(super::G_PANEL_REPAINT)
    }

    pub fn a_row_is_already_asked_for(&self, rows: u32) {
        self.space()
            .write::<u32>(super::G_GUI + super::GUI_ROWS, rows);
    }

    /// Where the sprite manager's textures are, which is the range a restore has to leave alone: the manager
    /// this image already lays out for the frame's own queue, and the array inside it.
    pub fn sprite_textures(&self) -> Range<usize> {
        let textures = QUAD_QUEUE.start + super::ANM_TEXTURES;
        textures..textures + super::ANM_TEXTURE_SLOTS * size_of::<usize>()
    }

    /// And no manager at all, which is what the pointer says before the game has made one.
    pub fn has_no_sprite_manager(&self) {
        self.space().write::<usize>(super::G_QUAD_QUEUE, 0);
    }

    /// The sheet in the slot the panel's tile is painted out of, as a load of the game's own leaves it: a
    /// texture in the manager's own array, which is the one thing a snapshot cannot hold a copy of.
    pub fn loads_the_panels_sheet(&self, texture: usize) {
        self.space().write::<usize>(self.panels_slot(), texture);
    }

    pub fn the_panels_sheet(&self) -> usize {
        self.space().read::<usize>(self.panels_slot())
    }

    fn panels_slot(&self) -> usize {
        QUAD_QUEUE.start + super::ANM_TEXTURES + PANEL_TEXTURE_SLOT as usize * size_of::<usize>()
    }

    /// A word of the manager's own outside that array, which orb reads nothing of and a restore therefore puts
    /// back: what makes the skipped range say something is that the block around it comes back like any other
    /// memory.
    pub fn set_sprite_manager_word(&self, word: u32) {
        self.space().write::<u32>(self.sprite_managers_word(), word);
    }

    pub fn sprite_manager_word(&self) -> u32 {
        self.space().read::<u32>(self.sprite_managers_word())
    }

    fn sprite_managers_word(&self) -> usize {
        self.sprite_textures().end
    }

    /// Whether the panel's own draw job is on the draw list, read back: the list, its elements and the
    /// callbacks in them are the game's own statics, so what a chapter holds is the list as it stood — see
    /// `the_handles_a_restore_leaves_alone_of_th07.rs`.
    pub fn paints_its_panel_now(&self) -> bool {
        self.space()
            .read::<usize>(super::G_CHAIN + super::CHAIN_DRAW_LIST + super::CHAIN_NEXT)
            == PANEL_ELEMENT
    }

    /// And that job cut, which is what the teardown of the object it belongs to leaves: the list's own head
    /// pointing at nothing.
    pub fn paints_no_panel(&self) {
        self.space().write::<usize>(
            super::G_CHAIN + super::CHAIN_DRAW_LIST + super::CHAIN_NEXT,
            0,
        );
    }

    /// How long the stage's own script has run, as its first main context's timer holds it.
    pub fn the_script_has_run_for(&self, frames: i32) {
        self.space()
            .write::<i32>(super::G_ENEMY_MANAGER + super::SCRIPT_FRAMES, frames);
    }

    /// An ending running, as the game's own register leaves it: the scene, the object with its script, and a
    /// chain element carrying the object — which is the only place the object's address is kept, so this is
    /// the whole of how orb can find it.
    pub fn an_ending_is_running(&self, script: usize) {
        let space = self.space();
        self.is_in_scene(SCENE_ENDING, SCENE_ENDING);
        space.map(ENDING.start, ENDING.len(), Kind::Private);
        space.write::<usize>(ENDING.start + super::ENDING_SCRIPT, script);
        space.write::<usize>(ENDING_ELEMENT + super::CHAIN_CALLBACK, super::ENDING_UPDATE);
        space.write::<usize>(ENDING_ELEMENT + super::CHAIN_ARGUMENT, ENDING.start);
        space.write::<usize>(super::G_CHAIN + super::CHAIN_NEXT, ENDING_ELEMENT);
    }

    /// And the script that ending is reading now, which is the one thing that says a part of one has ended: an
    /// ending moves on by reading the next file over the one it was running, and neither the flag nor the scene
    /// moves with it.
    pub fn the_ending_reads_the_script(&self, script: usize) {
        self.space()
            .write::<usize>(ENDING.start + super::ENDING_SCRIPT, script);
    }

    /// The generator's state, which is one word: what the game's own step would have left there, and what a
    /// replay's stage writes into it to play that stage again.
    pub fn generator_is_at(&self, seed: u16) {
        self.space().write::<u16>(super::G_RANDOM, seed);
    }

    /// A menu of the game's own up over the run, which is what stops its update: the pause menu's flag,
    /// which is the one the game raises itself on the menu button.
    pub fn is_paused(&self) {
        self.space()
            .write::<u8>(super::G_GAME_MANAGER + super::GAME_MENU_UP, 1);
    }

    /// And that menu gone, which is its own first item — 続ける — taken: the flag down and the run's update
    /// running again.
    pub fn is_not_paused(&self) {
        self.space()
            .write::<u8>(super::G_GAME_MANAGER + super::GAME_MENU_UP, 0);
    }

    /// Whether it is up, for a laid-out game whose own frame has to do what the manager's update does about
    /// it — see `Th07`'s `GAME_MENU_UP`, and [`Image::the_word_the_read_left`] for the word it goes up on.
    pub fn is_it_paused(&self) -> bool {
        self.space()
            .read::<u8>(super::G_GAME_MANAGER + super::GAME_MENU_UP)
            != 0
    }

    /// And a replay being watched rather than a run being played, which is the bit the game sets as it
    /// loads one.
    pub fn is_watching_a_replay(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(super::G_RUN_KIND, kind | RUN_IS_A_REPLAY);
    }

    /// And the attract demo playing, which is the bit the title screen sets as its idle timer runs out.
    pub fn is_the_attract_demo(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(super::G_RUN_KIND, kind | RUN_IS_A_DEMO);
    }

    /// And one stage of it practised rather than the whole game played, which is the bit its title menu sets
    /// for that item.
    pub fn is_practising(&self) {
        let space = self.space();
        let kind = space.read::<u32>(super::G_RUN_KIND);
        space.write::<u32>(super::G_RUN_KIND, kind | RUN_IS_A_PRACTICE);
    }

    /// The pad's buttons as the game's own key config read would have answered them, and where that array
    /// is: 0x80 in the byte of each button held.
    ///
    /// The read itself is the game's own function, which a laid-out game hands over — this is the array it
    /// answers with, and what an e2e test reads back to see the buttons orb added.
    pub fn pad_buttons(&self) -> usize {
        super::PAD_BUTTONS
    }

    pub fn pad_button_held(&self, button: usize) -> bool {
        self.space().read::<u8>(super::PAD_BUTTONS + button) & 0x80 != 0
    }

    /// And the array emptied, which is what the game's own read does before it fills it — every byte of
    /// what its own screen walks.
    pub fn empties_the_pad_buttons(&self) {
        self.space()
            .fill_bytes(super::PAD_BUTTONS, 0, super::PAD_BUTTONS_READ);
    }

    /// The fog turned on, as a stage's own background drawing turns it on at 0x43a1bd: the field both of
    /// the game's fog calls compare and write, set to the 1 that call writes.
    pub fn turns_the_fog_on(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, 1);
    }

    pub fn fog_is_on(&self) -> bool {
        self.space()
            .read::<u32>(super::G_SUPERVISOR + super::FOG_ON)
            != 0
    }

    /// The same field written the way the game's own frame writes it at 0x434739, which is what makes
    /// the call after it reach the device: 0xff, and the call does nothing at all where this says the
    /// fog is already out.
    pub fn forces_the_fog_call_through(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, FOG_ON_FORCED);
    }

    /// And the field 0x43a207 clears, which is the half of that call this block holds: the
    /// `SetRenderState` beside it goes through the device, which is a real object in an e2e test and not
    /// laid out at all.
    pub fn puts_the_fog_out(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, 0);
    }

    /// Hands over the three calls the game's own frame makes on that queue and on the fog, in place of
    /// the addresses `Th07` holds: [`empties_the_queue`](Image::empties_the_queue),
    /// [`draws_the_queue`](Image::draws_the_queue) and the fog put out.
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

    /// The front end up on its title menu: the object where the game's own register would have left it — hung
    /// off a chain element, which is the only place its address is kept — and the screen at the title.
    pub fn the_front_end_is_up(&self) {
        let space = self.space();
        self.is_in_scene(SCENE_FRONT_END, SCENE_FRONT_END);
        // Laid out only where it is not already, an address space having one of each: what a front end built a
        // second time writes is [`the_front_end_is_up_again`](Image::the_front_end_is_up_again)'s fields, and a
        // test that reaches this twice is one that came back to the title from somewhere.
        if space.has_room(FRONT_END.start, FRONT_END.len()) {
            space.map(FRONT_END.start, FRONT_END.len(), Kind::Private);
        }
        space.write::<i32>(FRONT_END.start + super::FRONT_END_SCREEN, SCREEN_TITLE);
        space.write::<usize>(
            FRONT_END_ELEMENT + super::CHAIN_CALLBACK,
            super::FRONT_END_UPDATE,
        );
        space.write::<usize>(FRONT_END_ELEMENT + super::CHAIN_ARGUMENT, FRONT_END.start);
        space.write::<usize>(super::G_CHAIN + super::CHAIN_NEXT, FRONT_END_ELEMENT);
        // The phase whose block the items are dispatched from, and long enough up for a press to be taken:
        // a title menu laid out in any other phase or younger than that is one the game itself ignores.
        space.write::<i32>(FRONT_END.start + super::FRONT_END_PHASE, TITLE_ITEMS_PHASE);
        self.the_screen_has_been_up_for(SETTLED_FRAMES);
    }

    /// How long the screen has been up, which is the game's own gate on a press: ten frames or more and it
    /// takes one.
    pub fn the_screen_has_been_up_for(&self, frames: i32) {
        self.space()
            .write::<i32>(FRONT_END.start + super::FRONT_END_SETTLED, frames);
    }

    /// The front end built again, as coming back from another scene builds it: the same fields written, without
    /// the object being laid out a second time — an address space has one of each.
    pub fn the_front_end_is_up_again(&self) {
        let space = self.space();
        self.is_in_scene(SCENE_FRONT_END, SCENE_FRONT_END);
        space.write::<i32>(FRONT_END.start + super::FRONT_END_SCREEN, SCREEN_TITLE);
        space.write::<usize>(super::G_CHAIN + super::CHAIN_NEXT, FRONT_END_ELEMENT);
    }

    /// The phase the screen that is up is in, written the way that screen's own update writes it: which phase
    /// means what belongs to the screen — [`TITLE_ITEMS_PHASE`] and [`SHOT_SELECT_PHASE_ITEMS`] are the two this
    /// file has — and what it decides is whether a press there is one the game acts on.
    pub fn the_screen_is_in_phase(&self, phase: i32) {
        self.space()
            .write::<i32>(FRONT_END.start + super::FRONT_END_PHASE, phase);
    }

    /// Which of the front end's screens is up and which item its title menu points at, read back: what orb must
    /// leave exactly as it found it where a question of its own was cancelled, and what it puts back after a
    /// ranking it asked for.
    pub fn the_screen_the_front_end_is_on(&self) -> i32 {
        self.space()
            .read::<i32>(FRONT_END.start + super::FRONT_END_SCREEN)
    }

    pub fn the_item_the_title_menu_points_at(&self) -> i32 {
        self.space()
            .read::<i32>(FRONT_END.start + super::FRONT_END_CURSOR)
    }

    /// And the screen a title menu is, so an e2e test says which one it is in the game's own terms rather than in
    /// a number of its own.
    pub fn screen_of_the_title_menu(&self) -> i32 {
        SCREEN_TITLE
    }

    /// And the cursor moved to one of the eight items, which is all moving it changes.
    pub fn the_title_menu_points_at(&self, item: i32) {
        self.space()
            .write::<i32>(FRONT_END.start + super::FRONT_END_CURSOR, item);
    }

    /// And the item that starts a whole run under the cursor, said by what it is rather than by its number:
    /// which item that is is this module's business, and an e2e test asking for it by index would be an e2e
    /// test carrying one of the game's own numbers.
    pub fn the_title_menu_points_at_a_run(&self) {
        self.the_title_menu_points_at(TITLE_ITEM_START);
    }

    /// A whole run's own shot select up, with a difficulty and a character already chosen and a shot under the
    /// cursor: the screen orb asks a run left unfinished over.
    ///
    /// Said by what it is rather than by its number, for the reason
    /// [`the_title_menu_points_at_a_run`](Image::the_title_menu_points_at_a_run) is: which of the three flows'
    /// shot selects that is is this module's business.
    pub fn the_shot_select_of_a_run_is_up(&self, difficulty: u8, character: u8, shot: u8) {
        self.the_shot_select_points_at(SCREEN_SHOT_SELECT, difficulty, character, shot);
        // And the phase the screen is in once it has been up long enough to read anything, which is the gate on
        // a press there — a screen left in the phase its build runs in is one the game itself acts on nothing
        // in.
        self.space().write::<i32>(
            FRONT_END.start + super::FRONT_END_PHASE,
            SHOT_SELECT_PHASE_ITEMS,
        );
    }

    /// And the front end having asked for the run scene, which is the one frame a run has been chosen and not
    /// built: what `Th07::run_chosen` reads, and so the frame a resume can point that build at a stage on.
    pub fn a_run_has_been_chosen(&self) {
        self.is_in_scene(SCENE_FRONT_END, SCENE_RUN);
    }

    /// **The numbers a run's own build draws for it, out of the generator as the real ones are.**
    ///
    /// 0x42e3da draws eight, each `(rand % 0x186a0) + 0x198f`: the baseline every reader of the cherry measures
    /// from — `[run+0x88]` — and the seven words at `[run+0x34]`, which are noise the seal is drawn over. It
    /// draws more of it than that; these are the ones a resume ever went near.
    ///
    /// **Drawn before a resumed run's seed goes in, which is why a laid-out game draws them at all.** The call
    /// is at 0x42e951, near the head of `GameManager::AddedCallback`, and the call to `STAGE_BUILD` — the one
    /// moment a resume can write the seed — is at 0x42f0b4, after it. So the draws move the generator where no
    /// resume can put it back, which is what makes the seed's own moment the one it is.
    ///
    /// **And the pair the check compares left agreeing**, `[run+0x3c]` and `[run+0xac]`: the build's own reseal
    /// at 0x42f08b makes the copy right after these draws, and a run that started out failing the check is one
    /// [`Image::the_seal_holds`] could say nothing about. The generator is not moved for it — the step
    /// `Fake::settles_a_seed_for_the_stage` takes stands for that reseal's two draws already.
    pub fn the_stage_draws_the_runs_numbers(&self) {
        let space = self.space();
        let drawn = || {
            let word = space.read::<u16>(super::G_RANDOM);
            let next = word.rotate_left(7) ^ 0x5bd1 | 1;
            space.write::<u16>(super::G_RANDOM, next);
            i32::from(next) % 0x186a0 + 0x198f
        };
        space.write::<i32>(RUN_NUMBERS.start + super::SCORE_BASELINE, drawn());
        for at in 0..super::RUN_RANDOMS {
            let word = drawn();
            space.write::<i32>(RUN_NUMBERS.start + super::RUN_RANDOMS_AT + at * 4, word);
        }
        // **And the two counts a run keeps of itself put back to nothing**, which the same callback does at
        // 0x42ea7c: a run picked up out of a file is a new run, so what it carries of the recorded run's is
        // whatever the resume writes and nothing else.
        space.write::<i32>(RUN_NUMBERS.start + super::COUNTED_ALL_RUN, 0);
        space.write::<i32>(RUN_NUMBERS.start + super::CARDS_CAPTURED, 0);
        // And the step the awards have climbed to back at the bottom, which is where a run that has collected
        // nothing yet has it.
        space.write::<u8>(super::AWARD_STEP, 0);
        seals_the_pair();
    }

    /// **The count a run's items have their cherry worked out from**, `[run+0x18]` — see
    /// [`super::COUNTED_ALL_RUN`], and the draw above for the build putting it back to nothing the way 0x42ea7c
    /// does.
    pub fn the_run_has_counted(&self, counted: i32) {
        self.space()
            .write::<i32>(RUN_NUMBERS.start + super::COUNTED_ALL_RUN, counted);
    }

    pub fn the_count_the_run_carries(&self) -> i32 {
        self.space()
            .read::<i32>(RUN_NUMBERS.start + super::COUNTED_ALL_RUN)
    }

    /// **The step the run's awards have climbed to**, `[0x62f644]` — see [`super::AWARD_STEP`], which is the
    /// table that step is an index into and the arithmetic that says a resume has to carry it.
    pub fn the_awards_have_climbed_to(&self, step: u8) {
        self.space().write::<u8>(super::AWARD_STEP, step);
    }

    pub fn the_step_the_awards_have_climbed(&self) -> u8 {
        self.space().read::<u8>(super::AWARD_STEP)
    }

    /// And the cards the run has captured, `[run+0x1c]` — see [`super::CARDS_CAPTURED`].
    pub fn the_run_has_captured(&self, captured: i32) {
        self.space()
            .write::<i32>(RUN_NUMBERS.start + super::CARDS_CAPTURED, captured);
    }

    pub fn the_cards_the_run_has_captured(&self) -> i32 {
        self.space()
            .read::<i32>(RUN_NUMBERS.start + super::CARDS_CAPTURED)
    }

    /// **Whether the seal the game checks still holds**, which is `[run+0xac]` against `[run+0x3c]` — see
    /// [`super::RESEAL`] for the two comparisons 0x404fe0 makes and why this is the one that catches a write of
    /// orb's. A run whose pair has come apart is a run the game ends on its next update, so this is what a test
    /// asks of a resume instead of laying out the tearing-down itself.
    pub fn the_seal_holds(&self) -> bool {
        let space = self.space();
        space.read::<i32>(RUN_NUMBERS.start + super::SEALED_COPY)
            == space.read::<i32>(RUN_NUMBERS.start + super::SEALED_WORD)
    }

    /// The shot select up, with a difficulty and a character already chosen and a shot under the cursor —
    /// which of the three flows' shot selects is the screen given here.
    pub fn the_shot_select_points_at(&self, screen: i32, difficulty: u8, character: u8, shot: u8) {
        let space = self.space();
        space.write::<i32>(FRONT_END.start + super::FRONT_END_SCREEN, screen);
        space.write::<u8>(super::G_DIFFICULTY_CHOSEN, difficulty);
        space.write::<u8>(super::G_CHARACTER_CHOSEN, character);
        // **The shot under the cursor is the cursor and not the byte.** 0x62f646 is written where the item is
        // *decided* (0x459e7d) and where the screen is left (0x459f72), out of `[front+0x0]` both times — so
        // while the cursor moves it still holds whichever shot was decided last. A laid-out game that wrote it
        // here would agree with an orb that read it, and a 妖々夢 session found what that costs: the mark for
        // 霊夢B's unfinished run, shown while the cursor was on 霊夢A.
        self.the_title_menu_points_at(i32::from(shot));
    }

    /// And the shot the game last *decided*, which is what that byte holds: a run started earlier, or the shot
    /// select left and come back to. Laid out so that an e2e test can say orb reads the cursor rather than this.
    pub fn the_shot_it_last_decided_is(&self, shot: u8) {
        self.space().write::<u8>(super::G_SHOT_CHOSEN, shot);
    }

    /// And one of its other screens up, which is a screen orb has no question for.
    pub fn the_front_end_is_on_another_screen(&self) {
        self.space()
            .write::<i32>(FRONT_END.start + super::FRONT_END_SCREEN, 3);
    }

    /// A replay whose record covers one stage, laid out the way its own header is: the manager's pointer, the
    /// header behind it, and a record in that stage's slot.
    pub fn a_replay_covers_the_stage(&self, stage: i32) {
        let space = self.space();
        // Laid out only where it is not already: one replay covers several stages, and an address space has one
        // of each.
        if space.has_room(REPLAY.start, REPLAY.len()) {
            space.map(REPLAY.start, REPLAY.len(), Kind::Private);
        }
        space.write::<usize>(super::G_REPLAY_MANAGER, REPLAY.start);
        let header = REPLAY.start + 0x100;
        space.write::<usize>(REPLAY.start + super::REPLAY_HEADER, header);
        space.write::<usize>(
            header + super::REPLAY_STAGE_RECORDS + stage as usize * size_of::<usize>(),
            header + 0x200,
        );
    }

    /// The generator as it stands, for a laid-out game whose own build draws from it: what the stage's frames run
    /// from is where those draws left it, so a build has to be able to read the number it is drawing from.
    pub fn the_generator_now(&self) -> u16 {
        self.space().read::<u16>(super::G_RANDOM)
    }

    /// And how far into that replay the playback has got, which is the counter at the manager's own head — the
    /// number a resumed run's frame is held against.
    pub fn the_replay_is_at(&self, frame: i32) {
        self.space().write::<i32>(REPLAY.start, frame);
    }

    /// How many numbers the generator has been asked for, which its own step keeps beside its state.
    pub fn the_generator_has_drawn(&self, randoms: u32) {
        self.space()
            .write::<u32>(super::G_RANDOM + super::RANDOMS_DRAWN, randoms);
    }

    /// Where the player is, as its own bomb would hand it to the effects.
    pub fn the_player_is_at(&self, x: f32, y: f32) {
        let space = self.space();
        space.write::<f32>(super::G_PLAYER + super::PLAYER_POSITION, x);
        space.write::<f32>(
            super::G_PLAYER + super::PLAYER_POSITION + size_of::<f32>(),
            y,
        );
    }

    /// And how many items are on the screen, as their own manager's walk left the count.
    pub fn items_alive(&self, items: i32) {
        self.space()
            .write::<i32>(super::G_ITEM_MANAGER + super::ITEM_COUNT, items);
    }

    /// An item of `kind` in one of the manager's own slots, where the spawn at 0x4326f0 would leave it: alive,
    /// with its kind and its position.
    ///
    /// The slot is the test's to name, because which slot an item is in is the whole question a walk answers —
    /// a rule that only ever looked at the first would pass a laid-out game and miss every real one.
    pub fn an_item_is_at(&self, slot: usize, kind: i8, x: f32, y: f32) {
        let item = super::G_ITEM_MANAGER + slot * super::ITEM_BYTES;
        let space = self.space();
        space.write::<i8>(item + super::ITEM_ALIVE, 1);
        space.write::<i8>(item + super::ITEM_KIND, kind);
        space.write::<f32>(item + super::ITEM_POSITION, x);
        space.write::<f32>(item + super::ITEM_POSITION + size_of::<f32>(), y);
    }

    /// And a slot with nothing in it, which is what the collect at 0x433b5a leaves: the alive byte cleared and
    /// what was in it left where it was, exactly as the game leaves it.
    pub fn the_item_is_gone(&self, slot: usize) {
        self.space().write::<i8>(
            super::G_ITEM_MANAGER + slot * super::ITEM_BYTES + super::ITEM_ALIVE,
            0,
        );
    }

    /// A bullet in one of its manager's own slots, alive and where its update would leave it: where it is, what
    /// that update adds to it every frame, and **the size the hit test is given** — no two kinds of bullet
    /// having the same one.
    pub fn a_bullet_is_at(&self, slot: usize, x: f32, y: f32, step: (f32, f32)) {
        self.a_bullet_of_size_is_at(slot, x, y, step, (0.0, 0.0));
    }

    /// And one with a size of its own, for what only a size can say: the bullet whose own box is what reaches
    /// the player rather than the player's reaching it.
    pub fn a_bullet_of_size_is_at(
        &self,
        slot: usize,
        x: f32,
        y: f32,
        step: (f32, f32),
        size: (f32, f32),
    ) {
        let bullet = super::G_BULLET_MANAGER + super::BULLETS + slot * super::BULLET_BYTES;
        let space = self.space();
        // The state the walk tests, whose 1 is the first of the five its dispatch knows.
        space.write::<u16>(bullet + super::BULLET_ALIVE, 1);
        space.write::<f32>(bullet + super::BULLET_POSITION, x);
        space.write::<f32>(bullet + super::BULLET_POSITION + size_of::<f32>(), y);
        space.write::<f32>(bullet + super::BULLET_STEP, step.0);
        space.write::<f32>(bullet + super::BULLET_STEP + size_of::<f32>(), step.1);
        space.write::<f32>(bullet + super::BULLET_SIZE, size.0);
        space.write::<f32>(bullet + super::BULLET_SIZE + size_of::<f32>(), size.1);
    }

    /// The player's own box, as the game's own hit test reads it: the smaller corner and the larger.
    ///
    /// Round nothing in particular, because what is read off it is how wide it is and not where: the real one
    /// follows the player and orb carries its half-extents to wherever a move would land.
    pub fn the_players_box_is(&self, half: (f32, f32)) {
        let space = self.space();
        space.write::<f32>(super::G_PLAYER + super::PLAYER_BOX_MIN, -half.0);
        space.write::<f32>(
            super::G_PLAYER + super::PLAYER_BOX_MIN + size_of::<f32>(),
            -half.1,
        );
        space.write::<f32>(super::G_PLAYER + super::PLAYER_BOX_MAX, half.0);
        space.write::<f32>(
            super::G_PLAYER + super::PLAYER_BOX_MAX + size_of::<f32>(),
            half.1,
        );
    }

    /// What one frame of holding a direction moves the player by, as the game's own build leaves it: the speed
    /// struct the player points at, and both of the scales at one so that the step is the speed.
    pub fn the_player_steps_by(&self, straight: f32, diagonal: f32) {
        let space = self.space();
        if space.has_room(PLAYER_SPEEDS.start, PLAYER_SPEEDS.len()) {
            space.map(PLAYER_SPEEDS.start, PLAYER_SPEEDS.len(), Kind::Private);
        }
        space.write::<usize>(super::G_PLAYER + super::PLAYER_SPEEDS, PLAYER_SPEEDS.start);
        space.write::<f32>(PLAYER_SPEEDS.start + super::SPEED_STRAIGHT, straight);
        space.write::<f32>(PLAYER_SPEEDS.start + super::SPEED_DIAGONAL, diagonal);
        space.write::<f32>(super::G_PLAYER + super::PLAYER_SPEED_SCALE, 1.0);
        space.write::<f32>(super::G_MOTION_SCALE, 1.0);
    }

    /// A laser in one of the slots, firing: where it comes from, which way it points, how far along it reaches
    /// and how wide it is.
    pub fn a_laser_is_at(
        &self,
        slot: usize,
        origin: (f32, f32),
        angle: f32,
        along: (f32, f32),
        width: f32,
    ) {
        let laser = super::G_LASERS + slot * super::LASER_BYTES;
        let space = self.space();
        space.write::<u32>(laser + super::LASER_ALIVE, 1);
        space.write::<f32>(laser + super::LASER_ORIGIN, origin.0);
        space.write::<f32>(laser + super::LASER_ORIGIN + size_of::<f32>(), origin.1);
        space.write::<f32>(laser + super::LASER_ANGLE, angle);
        space.write::<f32>(laser + super::LASER_NEAR, along.0);
        space.write::<f32>(laser + super::LASER_FAR, along.1);
        space.write::<f32>(laser + super::LASER_WIDTH, width);
        // Nought of its phases through, which is a beam that is still there to be hit by.
        space.write::<u8>(laser + super::LASER_PHASE, 0);
    }

    /// And one that has gone past the phase both of the game's own walks stop looking at.
    pub fn the_laser_is_fading(&self, slot: usize) {
        self.space().write::<u8>(
            super::G_LASERS + slot * super::LASER_BYTES + super::LASER_PHASE,
            super::LASER_FADING,
        );
    }

    /// An enemy in one of the manager's own slots, alive the way its own walk reads alive — bit 7 of
    /// `+0x2e28` — and where it is.
    pub fn an_enemy_is_at(&self, slot: usize, x: f32, y: f32) {
        self.an_enemy_of_size_is_at(slot, x, y, (0.0, 0.0));
    }

    /// And one with a body of its own, which is what a boss has: the size the player's own test is given.
    pub fn an_enemy_of_size_is_at(&self, slot: usize, x: f32, y: f32, size: (f32, f32)) {
        let enemy = super::G_ENEMY_MANAGER + ENEMIES + slot * super::ENEMY_BYTES;
        let space = self.space();
        space.write::<u8>(enemy + super::ENEMY_ALIVE, super::ENEMY_ALIVE_BIT);
        space.write::<f32>(enemy + super::ENEMY_POSITION, x);
        space.write::<f32>(enemy + super::ENEMY_POSITION + size_of::<f32>(), y);
        space.write::<f32>(enemy + super::ENEMY_SIZE, size.0);
        space.write::<f32>(enemy + super::ENEMY_SIZE + size_of::<f32>(), size.1);
    }

    /// Where the boss is, written into the slot [`Image::a_boss_is_fighting`] hands out — so the two go
    /// together, this one being the position of the enemy that one made a boss.
    pub fn the_boss_is_at(&self, x: f32, y: f32) {
        let boss = super::G_ENEMY_MANAGER + ENEMIES;
        let space = self.space();
        space.write::<f32>(boss + super::ENEMY_POSITION, x);
        space.write::<f32>(boss + super::ENEMY_POSITION + size_of::<f32>(), y);
    }

    /// The play field the player is held inside, as the stage's own build leaves it for the clamp at 0x43f416
    /// to read.
    pub fn the_play_field_is(&self, left: f32, top: f32, width: f32, height: f32) {
        let space = self.space();
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_LEFT, left);
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_TOP, top);
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_WIDTH, width);
        space.write::<f32>(super::G_GAME_MANAGER + super::FIELD_HEIGHT, height);
    }

    /// The word a run acts on, which is where the recorder puts what was read and where a replay puts what it
    /// recorded — not the raw read beside it.
    pub fn the_run_acts_on_the_word(&self, word: u16) {
        self.space()
            .write::<u16>(super::G_INPUT_THE_RUN_ACTS_ON, word);
    }

    /// The configuration asking for a window, or not, as the game's own option screen leaves it — and read
    /// back, which is how a test sees what orb asked the game for.
    pub fn is_configured_for_a_window(&self, windowed: bool) {
        self.space()
            .write::<u8>(super::CFG_WINDOWED, u8::from(windowed));
    }

    pub fn asks_for_a_window(&self) -> bool {
        self.space().read::<u8>(super::CFG_WINDOWED) != 0
    }

    /// And the word the read itself left, which is the one the front end's own screens compare against last
    /// frame's — put in by a test, and read back to see what orb left there.
    pub fn the_read_left_the_word(&self, word: u16) {
        self.space().write::<u16>(super::G_INPUT_READ, word);
    }

    pub fn the_word_the_read_left(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_READ)
    }

    /// And the word **the run acts on**, which is the one the player's own job reads: what a frame is played
    /// with, as against what its read produced — see `Th07`'s own `G_INPUT_THE_RUN_ACTS_ON`.
    pub fn the_word_the_run_acts_on(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_THE_RUN_ACTS_ON)
    }

    /// And last frame's of that word, which is the other half of every edge a run reads: a bomb is a press and
    /// not a hold, so what says one happened is the pair — see `Th07`'s own `G_INPUT_THE_RUN_ACTED_ON`.
    pub fn the_word_the_run_acted_on(&self) -> u16 {
        self.space().read::<u16>(super::G_INPUT_THE_RUN_ACTED_ON)
    }

    /// A card's own record touched, as the game's own counting would leave it: the record tagged the way every
    /// one of them is, with an attempt count in it.
    ///
    /// What orb does with these is hold them across a rewind and a file read, so what a test needs is a record
    /// that differs from its neighbours rather than the game's own field layout.
    pub fn a_card_has_been_attempted(&self, card: usize, attempts: u16) {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
        let space = self.space();
        space.write::<u32>(at, u32::from_le_bytes(*b"CATK"));
        space.write::<u16>(at + 0x4, super::CARD_RECORD_BYTES as u16);
        space.write::<u16>(at + super::CARD_ATTEMPTS, attempts);
    }

    /// A card starting, as far as its own record goes: the record marked as one the game has written, one more
    /// attempt under this run's own index and one more on the card's total, and the check byte put right —
    /// which is what 0x410110 does in the same breath as copying the name in.
    ///
    /// Here rather than in the e2e test that drives a card, because it is the *game's* count: orb adds the one
    /// a retry is on top of it, so a card whose start counted nothing would have orb reading a first attempt
    /// where a session has had two.
    pub fn a_card_is_being_attempted(&self, card: usize) {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
        let space = self.space();
        space.write::<u32>(at, u32::from_le_bytes(*b"CATK"));
        space.write::<u16>(at + 0x4, super::CARD_RECORD_BYTES as u16);
        let shot = usize::from(space.read::<u8>(super::G_SHOT_COMBINED));
        for word in [
            at + super::CARD_ATTEMPTS + shot * size_of::<u16>(),
            at + super::CARD_ATTEMPTS_IN_ALL,
        ] {
            space.write::<u16>(word, space.read::<u16>(word).wrapping_add(1));
        }
        space.write::<u8>(
            at + super::CARD_CHECK,
            self.card_check_the_game_would_compute(card),
        );
    }

    /// And a card captured, as the game's own count of one leaves the record: one more under this run's own shot.
    pub fn a_card_has_been_captured(&self, card: usize, shot: usize) {
        let at = super::G_CARD_RECORDS
            + card * super::CARD_RECORD_BYTES
            + super::CARD_CAPTURES
            + shot * size_of::<u16>();
        let space = self.space();
        space.write::<u16>(at, space.read::<u16>(at).wrapping_add(1));
        space.write::<u8>(
            super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES + super::CARD_CHECK,
            self.card_check_the_game_would_compute(card),
        );
    }

    /// And a card's capture count read back, per shot.
    pub fn card_captures(&self, card: usize, shot: usize) -> u16 {
        let at = super::G_CARD_RECORDS
            + card * super::CARD_RECORD_BYTES
            + super::CARD_CAPTURES
            + shot * size_of::<u16>();
        self.space().read::<u16>(at)
    }

    /// The game's own save handed over, and the image its file is read into laid out — see [`SCORE_FILE_IMAGE`].
    ///
    /// The parse is not handed over with it: what a launch installs over that one is a hook, so what a laid-out
    /// game does with it is *call* it, and the address it calls is orb's where orb installed one.
    /// The image its file is read into laid out — see [`SCORE_FILE_IMAGE`].
    ///
    /// Nothing is handed over with it: orb calls no save of this game's, and the parse inside one is a hook rather
    /// than a call of orb's, so what a laid-out game does with both is make them itself.
    pub fn lays_out_its_score_files_image(&self) {
        let bytes = FILE_HEADER_BYTES + super::CARDS * super::CARD_RECORD_BYTES;
        let space = self.space();
        for (base, bytes) in [
            (SCORE_FILE_IMAGE, bytes),
            (SCORE_SCREEN.start, SCORE_SCREEN.len()),
        ] {
            if space.has_room(base, bytes) {
                space.map(base, bytes, Kind::Private);
            }
        }
    }

    /// And the gather a save makes, for the launch that installs orb's hook over it — the same shape
    /// [`Image::its_card_record_parse`] is handed over in.
    pub fn its_score_file_gather(&self) -> extern "C" fn(*mut std::ffi::c_void) -> i32 {
        gathers_and_writes
    }

    /// Which object the read about to be made is made on, for the reason [`GATHERING_ON`] exists: the stand-in
    /// for that read takes no `this`, and what it reads through this is the phase that says whether a write
    /// follows.
    pub fn its_next_read_is_made_on(&self, screen: usize) {
        *GATHERING_ON.lock().unwrap() = screen;
    }

    /// And which file that read comes out of, where nothing is written afterwards: a screen's own read is a read,
    /// so the file it lands in is the mode's like every other — see [`Image::saves_its_score_file`] for the pair
    /// this is half of.
    pub fn saves_nothing_reading(&self, from: &str) {
        *SAVING_FROM.lock().unwrap() = from.to_owned();
        *SAVING_INTO.lock().unwrap() = None;
    }

    /// And the parse that save makes, for the launch that installs orb's hook over it: what a laid-out game hands
    /// over is what the hook calls through to, the way it hands over every other callback of its own.
    pub fn its_card_record_parse(&self) -> unsafe extern "fastcall" fn(usize, usize) -> i32 {
        parses_the_card_records
    }

    /// **The card records as the game's own read of its file leaves them**: the file's own chunks laid over the
    /// array, which is what both of its reads do — the front end's at 0x45bff3 and the one at a run's build at
    /// 0x42e735, each passing [`G_CARD_RECORDS`](super::G_CARD_RECORDS) to the same parse.
    ///
    /// Which is why a run's own counts are the file's plus that run's rather than that run's alone: the build
    /// empties the records (0x42e657) and then reads the file straight back into them.
    pub fn reads_its_card_records_out_of_the_file(&self, name: &str) {
        let held = score_file(name);
        let space = self.space();
        space.write::<u32>(SCORE_FILE_IMAGE + FILE_CHUNKS_AT, FILE_HEADER_BYTES as u32);
        space.write::<u32>(
            SCORE_FILE_IMAGE + FILE_BYTES,
            (FILE_HEADER_BYTES + held.len()) as u32,
        );
        space.write_bytes(SCORE_FILE_IMAGE + FILE_HEADER_BYTES, &held);
        unsafe { parses_the_card_records(SCORE_FILE_IMAGE, super::G_CARD_RECORDS) };
    }

    /// And every record emptied, which is what a run's build does before that read — a `rep stos` over the whole
    /// array at 0x42e657, then a header per card with no counts in it.
    pub fn empties_its_card_records(&self) {
        let space = self.space();
        for card in 0..super::CARDS {
            let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
            space.write_bytes(at, &[0u8; super::CARD_RECORD_BYTES]);
            space.write::<u32>(at, super::CARD_RECORD_TAG);
            space.write::<u16>(at + 0x4, super::CARD_RECORD_BYTES as u16);
            space.write::<u16>(at + 0x6, super::CARD_RECORD_BYTES as u16);
            space.write::<u8>(at + 0x8, 1);
            space.write::<u16>(at + CHUNK_CARD, card as u16);
        }
    }

    /// The counts the score file holds for a card, per shot: its attempts and its captures, as the file's own
    /// chunk for that card carries them, and zero where the file has no chunk for it.
    pub fn its_score_file_counts(&self, name: &str, card: usize, shot: usize) -> (u16, u16) {
        let held = score_file(name);
        held.as_chunks::<{ super::CARD_RECORD_BYTES }>()
            .0
            .iter()
            .find(|chunk| {
                u32::from_le_bytes(chunk[..4].try_into().unwrap()) == super::CARD_RECORD_TAG
                    && u16::from_le_bytes(chunk[CHUNK_CARD..CHUNK_CARD + 2].try_into().unwrap())
                        as usize
                        == card
            })
            .map_or((0, 0), |chunk| {
                let word = |at: usize| {
                    u16::from_le_bytes(
                        chunk[at + shot * 2..at + shot * 2 + 2]
                            .try_into()
                            .unwrap_or_default(),
                    )
                };
                (word(super::CARD_ATTEMPTS), word(super::CARD_CAPTURES))
            })
    }

    /// And the file with a card already counted in it, which is the history a session's own counts are added to:
    /// one chunk, tagged and numbered the way the game's own write leaves one.
    pub fn its_score_file_holds(
        &self,
        name: &str,
        card: usize,
        shot: usize,
        attempts: u16,
        captures: u16,
    ) {
        let mut chunk = vec![0u8; super::CARD_RECORD_BYTES];
        chunk[..4].copy_from_slice(&super::CARD_RECORD_TAG.to_le_bytes());
        chunk[0x4..0x6].copy_from_slice(&(super::CARD_RECORD_BYTES as u16).to_le_bytes());
        chunk[0x6..0x8].copy_from_slice(&(super::CARD_RECORD_BYTES as u16).to_le_bytes());
        chunk[0x8] = 1;
        chunk[CHUNK_CARD..CHUNK_CARD + 2].copy_from_slice(&(card as u16).to_le_bytes());
        let mut word = |at: usize, count: u16| {
            chunk[at + shot * 2..at + shot * 2 + 2].copy_from_slice(&count.to_le_bytes());
        };
        word(super::CARD_ATTEMPTS, attempts);
        word(super::CARD_CAPTURES, captures);
        let mut held = score_file(name);
        held.extend_from_slice(&chunk);
        set_score_file(name, held);
    }

    /// And the file with その他の記録 in it, which is the one chunk that carries both totals and every count
    /// under them — see `Th07`'s `G_PLAY_RECORDS`. The two times are all a test needs of it, so those are what
    /// go in: the hours, minutes and seconds the game keeps each as.
    pub fn its_score_file_records_times(
        &self,
        name: &str,
        running: (u32, u32, u32),
        play: (u32, u32, u32),
    ) {
        let mut chunk = vec![0u8; PLAY_RECORDS_BYTES];
        chunk[..4].copy_from_slice(&PLAY_RECORDS_TAG.to_le_bytes());
        chunk[0x4..0x6].copy_from_slice(&(PLAY_RECORDS_BYTES as u16).to_le_bytes());
        chunk[0x6..0x8].copy_from_slice(&(PLAY_RECORDS_BYTES as u16).to_le_bytes());
        chunk[0x8] = 1;
        for (total, time) in [(super::RUNNING_TIME, running), (super::PLAY_TIME, play)] {
            let (hours, minutes, seconds) = time;
            for (at, held) in [(0, hours), (4, minutes), (8, seconds)] {
                let at = total + at;
                chunk[at..at + 4].copy_from_slice(&held.to_le_bytes());
            }
        }
        let mut held = score_file(name);
        held.extend_from_slice(&chunk);
        set_score_file(name, held);
    }

    /// リトライ回数 as the block holds it: the difficulty's own and the total beside it — see `Th07`'s
    /// `G_PLAY_RECORDS` for which field that is and what fills it in each mode.
    pub fn the_retries_it_records(&self, difficulty: usize) -> (u32, u32) {
        let read = |at| mem::read_committed::<u32>(at).unwrap_or_default();
        (
            read(super::G_PLAY_RECORDS + super::RETRIES + difficulty * super::PLAY_RECORD_BYTES),
            read(super::G_PLAY_RECORDS + super::RETRIES_TOTAL),
        )
    }

    /// And what the block holds of each now, which is where a parse of one of those files puts them.
    pub fn the_times_it_records(&self) -> ((u32, u32, u32), (u32, u32, u32)) {
        let read = |total: usize| {
            let at = |offset| {
                mem::read_committed::<u32>(super::G_PLAY_RECORDS + total + offset)
                    .unwrap_or_default()
            };
            (at(0), at(4), at(8))
        };
        (read(super::RUNNING_TIME), read(super::PLAY_TIME))
    }

    /// A span of 総プレイ時間 counted the way the game counts one: milliseconds added to the last of the four
    /// and carried up through seconds and minutes into hours — 0x43a3f4, and 0x43a2ff for the other total.
    ///
    /// **Where the game puts a span and not where orb would**: the three points that call it are the front end's
    /// own update, a run beginning and a run's teardown, so a test says one by saying which of those the game is
    /// in and then that a span happened.
    pub fn counts_a_play_span(&self, milliseconds: u32) {
        self.counts_a_span(super::PLAY_TIME, milliseconds);
    }

    /// And a span of 総起動時間, which is the same arithmetic on the three before it: that one is every frame the
    /// game has been up, so it is counted wherever the game is.
    pub fn counts_a_running_span(&self, milliseconds: u32) {
        self.counts_a_span(super::RUNNING_TIME, milliseconds);
    }

    fn counts_a_span(&self, total: usize, milliseconds: u32) {
        let space = self.space();
        let at = |offset| super::G_PLAY_RECORDS + total + offset;
        // Read out as the one number the four are, added to, and put back carried — which is what the game's own
        // chain of divisions leaves, the block being normalised at every step of it.
        let held = [0, 4, 8, 0xc].map(|offset| space.read::<u32>(at(offset)));
        let [hours, minutes, seconds, ms] = held;
        let total = ((hours as u64 * 60 + minutes as u64) * 60 + seconds as u64) * 1000
            + ms as u64
            + milliseconds as u64;
        for (offset, held) in [
            (0, total / 3_600_000),
            (4, total / 60_000 % 60),
            (8, total / 1000 % 60),
            (0xc, total % 1000),
        ] {
            space.write::<u32>(at(offset), held as u32);
        }
    }

    /// And the save the game makes for itself, which its own run build makes before it empties the records
    /// (0x42e646): the same call orb asks for, made where the game makes it.
    pub fn saves_its_score_file(&self, from: &str, into: Option<&str>) {
        *SAVING_FROM.lock().unwrap() = from.to_owned();
        *SAVING_INTO.lock().unwrap() = into.map(str::to_owned);
        saves_its_records(super::SAVE_KIND_WRITES);
    }

    /// How many times the game has been asked for a save of its own, which is what says orb asked at all.
    pub fn times_it_was_asked_to_save(&self) -> u32 {
        SAVES_ASKED_FOR.load(Ordering::Relaxed)
    }

    /// And a card's attempt count read back, per shot, which is how a test says whose word orb counted into.
    pub fn card_attempts(&self, card: usize, shot: usize) -> u16 {
        let at = super::G_CARD_RECORDS
            + card * super::CARD_RECORD_BYTES
            + super::CARD_ATTEMPTS
            + shot * size_of::<u16>();
        self.space().read::<u16>(at)
    }

    /// And the card's own total, the word every shot's attempt goes into as well as its own.
    pub fn card_attempts_in_all(&self, card: usize) -> u16 {
        let at =
            super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES + super::CARD_ATTEMPTS_IN_ALL;
        self.space().read::<u16>(at)
    }

    /// A card whose counts are already where the game stops adding to them.
    pub fn a_cards_counts_are_at_the_cap(&self, card: usize) {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
        let space = self.space();
        space.write::<u16>(at + super::CARD_ATTEMPTS, CARD_COUNT_CAP);
        space.write::<u16>(at + super::CARD_ATTEMPTS_IN_ALL, CARD_COUNT_CAP);
    }

    /// A card named the way its own start names it, into the record's own name field.
    pub fn the_card_is_named(&self, card: usize, name: &[u8]) {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES + super::CARD_NAME;
        let space = self.space();
        for (byte, value) in name.iter().enumerate() {
            space.write::<u8>(at + byte, *value);
        }
        space.write::<u8>(at + name.len(), 0);
    }

    /// And the name read back, up to its terminator.
    pub fn the_cards_name(&self, card: usize) -> Vec<u8> {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES + super::CARD_NAME;
        let space = self.space();
        (0..super::CARD_ATTEMPTS - super::CARD_NAME)
            .map(|byte| space.read::<u8>(at + byte))
            .take_while(|byte| *byte != 0)
            .collect()
    }

    /// The result screen's own state and the timer its leaving waits on, read back.
    pub fn the_screens_state(&self) -> i32 {
        self.space().read::<i32>(RESULT.start + super::RESULT_PHASE)
    }

    pub fn the_screens_timer(&self) -> i32 {
        self.space().read::<i32>(RESULT.start + super::RESULT_TIMER)
    }

    /// A card record's own check byte, and the sum the game's own check at 0x41008b would compute for it —
    /// said separately so a test compares orb's answer against the game's arithmetic rather than against
    /// itself.
    pub fn card_check(&self, card: usize) -> u8 {
        let record = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
        self.space().read::<u8>(record + super::CARD_CHECK)
    }

    pub fn card_check_the_game_would_compute(&self, card: usize) -> u8 {
        let record = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
        let space = self.space();
        let mut sum = 0u32;
        // The game's own bound is the terminator and nothing else — 0x40ffe5 walks the name for its length
        // and 0x410027 sums that many bytes — so this walks the whole of the field the name is copied into.
        for byte in 0..super::CARD_ATTEMPTS - super::CARD_NAME {
            let read = space.read::<u8>(record + super::CARD_NAME + byte);
            if read == 0 {
                break;
            }
            sum = sum.wrapping_add(read as i8 as i32 as u32);
        }
        for shot in 0..super::CARD_SHOTS {
            sum = sum
                .wrapping_add(u32::from(
                    space.read::<u16>(record + super::CARD_CAPTURES + shot * size_of::<u16>()),
                ))
                .wrapping_add(u32::from(
                    space.read::<u16>(record + super::CARD_ATTEMPTS + shot * size_of::<u16>()),
                ))
                .wrapping_add(
                    space.read::<u32>(record + super::CARD_SCORES + shot * size_of::<u32>()),
                );
        }
        sum as u8
    }

    /// The screen a run ends on, brought up the way the given phase says: the object where its own register
    /// would have left it — hung off a chain element — and the phase inside it.
    pub fn the_run_ended_screen_is_up(&self, phase: i32) {
        let space = self.space();
        // Laid out only where it is not already: one `ResultScreen` reached two ways is the same object either
        // way, and an address space has one of each.
        if space.has_room(RESULT.start, RESULT.len()) {
            space.map(RESULT.start, RESULT.len(), Kind::Private);
        }
        space.write::<i32>(RESULT.start + super::RESULT_PHASE, phase);
        space.write::<usize>(RESULT_ELEMENT + super::CHAIN_CALLBACK, super::RESULT_UPDATE);
        space.write::<usize>(RESULT_ELEMENT + super::CHAIN_ARGUMENT, RESULT.start);
        space.write::<usize>(super::G_CHAIN + super::CHAIN_NEXT, RESULT_ELEMENT);
    }

    /// That screen's own object, for a test that has to hand it to a callback the way the chain hands it.
    pub fn the_screen_a_run_ends_at(&self) -> usize {
        RESULT.start
    }

    /// And that screen's phase moved, which is all the difference between the two ways in.
    pub fn the_screen_was_brought_up_by(&self, phase: i32) {
        self.space()
            .write::<i32>(RESULT.start + super::RESULT_PHASE, phase);
    }

    /// The same screen brought up the way a *whole run's* own end brings it, said by what it is rather than by
    /// the number of the phase: past the state its build runs in, which is what orb waits for while it asks
    /// the game to write a ranking.
    pub fn the_screen_a_run_ends_at_is_up(&self) {
        self.the_run_ended_screen_is_up(RESULT_FROM_A_RUN);
    }

    /// And that screen moved on to the question about saving a replay, which is the first of the phases it asks
    /// that in — and the one state of it that is orb's business.
    pub fn the_screen_a_run_ends_at_offers_a_replay(&self) {
        self.the_screen_was_brought_up_by(*super::RESULT_OFFERING_A_REPLAY.start());
    }

    /// The phase that screen is in, read back, and the two an e2e test holds it against: the one a whole run's
    /// own end brings it up in, and the one its leaving runs in.
    pub fn the_phase_the_screen_is_in(&self) -> i32 {
        self.space().read::<i32>(RESULT.start + super::RESULT_PHASE)
    }

    pub fn phase_of_a_screen_from_a_run(&self) -> i32 {
        RESULT_FROM_A_RUN
    }

    pub fn phase_of_a_screen_leaving(&self) -> i32 {
        RESULT_LEAVING
    }

    /// And whether that screen is the one on the chain and has been told to leave, which is the state its own
    /// menu puts it in — after which the game puts the scene back itself.
    ///
    /// On the chain as well as in that state, because what takes the screen down is the head of the chain
    /// moving on: the object is left where it was, so its phase alone would read as leaving for ever.
    pub fn the_screen_a_run_ends_at_is_leaving(&self) -> bool {
        let space = self.space();
        space.read::<usize>(super::G_CHAIN + super::CHAIN_NEXT) == RESULT_ELEMENT
            && space.read::<i32>(RESULT.start + super::RESULT_PHASE) == RESULT_LEAVING
    }

    /// The cherry where a run has it, as the game keeps it: the baseline the three are counted from, and each
    /// of the three as that baseline plus what it stands at.
    pub fn the_cherry_is(&self, baseline: i32, cherry: i32, target: i32, plus: i32) {
        let space = self.space();
        space.write::<i32>(RUN_NUMBERS.start + super::SCORE_BASELINE, baseline);
        space.write::<i32>(super::G_CHERRY, baseline + cherry);
        space.write::<i32>(super::G_CHERRY_TARGET, baseline + target);
        space.write::<i32>(super::G_CHERRY_PLUS, baseline + plus);
    }

    /// And the three read back the same way, which is what a resumed stage would be filling from.
    pub fn the_cherry(&self) -> (i32, i32, i32) {
        let space = self.space();
        let baseline = space.read::<i32>(RUN_NUMBERS.start + super::SCORE_BASELINE);
        (
            space.read::<i32>(super::G_CHERRY) - baseline,
            space.read::<i32>(super::G_CHERRY_TARGET) - baseline,
            space.read::<i32>(super::G_CHERRY_PLUS) - baseline,
        )
    }

    /// The tracks `thbgm.dat` holds, as the player's own read of that file's table leaves them: one record per
    /// track in the file's order, behind the pointer field the lookup walks from, and an empty record after the
    /// last — which is where that walk stops.
    ///
    /// Laid out by every test with a track playing, because a slot the game is asked to start is answered
    /// through here: a slot holds a name, and a name means whichever of these records carries it.
    pub fn holds_its_music_file(&self, tracks: &[Track]) {
        let space = self.space();
        let bytes = (tracks.len() + 1) * RECORD_STRIDE;
        if space.has_room(MUSIC_RECORDS, bytes) {
            space.map(MUSIC_RECORDS, bytes, Kind::Private);
        }
        space.write::<usize>(super::G_SOUND_PLAYER + SOUND_RECORDS, MUSIC_RECORDS);
        for (entry, track) in tracks.iter().enumerate() {
            let record = MUSIC_RECORDS + entry * RECORD_STRIDE;
            space.write_bytes(record + super::RECORD_NAME, track.name.to_bytes_with_nul());
            space.write::<u32>(record + super::RECORD_AT, track.at);
            space.write::<u32>(record + super::RECORD_LOOP_AT, track.loops_at);
            space.write::<u32>(record + super::RECORD_BYTES, track.bytes);
        }
    }

    /// The two songs the stage that was just built has, where its own data names them: the stage's own and the
    /// one the fight it ends with brings, behind the game's own music-table pointer.
    ///
    /// The `.mid` names the table carries, which is not what a slot or a record holds — the load writes `wav`
    /// over the extension and a record has no directory at all, so laying them out as the table has them is
    /// what makes an e2e test say the three are compared by the stem rather than whole.
    pub fn names_its_songs(&self, stage: &CStr, fight: &CStr) {
        let space = self.space();
        let bytes = super::FIGHT_SONG_NAME + super::SONG_NAME_BYTES;
        if space.has_room(STAGE_SONGS, bytes) {
            space.map(STAGE_SONGS, bytes, Kind::Private);
        }
        space.write::<usize>(super::G_MUSIC_TABLE, STAGE_SONGS);
        space.write_bytes(
            STAGE_SONGS + super::STAGE_SONG_NAME,
            stage.to_bytes_with_nul(),
        );
        space.write_bytes(
            STAGE_SONGS + super::FIGHT_SONG_NAME,
            fight.to_bytes_with_nul(),
        );
        // And the one a fight's second phase brings, which is the game's own string rather than this stage's —
        // see `Th07`'s `SECOND_PHASE_SONG`. Laid out here because a real game always has it: it is a constant of
        // the exe, so a laid-out one without it is a game orb reads a name out of and finds nothing.
        if space.has_room(super::SECOND_PHASE_SONG, super::SONG_NAME_BYTES) {
            space.map(
                super::SECOND_PHASE_SONG,
                super::SONG_NAME_BYTES,
                Kind::Private,
            );
        }
        space.write_bytes(
            super::SECOND_PHASE_SONG,
            SECOND_PHASE_SONG_NAME.to_bytes_with_nul(),
        );
    }

    /// And a track loaded into one of the player's own slots, which is what a stage's build does with two of
    /// them — its own song and its boss's — before it starts either: the name goes in that slot's buffer with
    /// the directory it was given still on it, as the load's own command leaves it.
    pub fn loads_a_slot(&self, slot: u32, path: &CStr) {
        self.space().write_bytes(
            super::G_SOUND_PLAYER + SLOT_NAMES + slot as usize * SLOT_NAME_BYTES,
            path.to_bytes_with_nul(),
        );
    }

    /// A track playing, as `SoundPlayer::PlayBGM` leaves the two objects it makes: the stream at the
    /// player's own field and the source it reads the file through, on that track's record.
    ///
    /// `sound` is the buffer and the file both — one object here, as it is in the sim — and `left` is the
    /// countdown the track's loop is taken on, which goes in with the file's position for the reason th06's
    /// own helper gives: the two are a pair, and moving one without the other is the fault this is about.
    pub fn streams_a_track(&self, sound: &orb_sim::Sound, track: Track, left: u32) {
        sound.install(&self.sim);
        self.streams_it_again(sound, track, left);
    }

    /// And another track through the sound already installed, which is what a stage handing over to a boss
    /// leaves: the same buffer and file, another of the file's records.
    ///
    /// Its own call because installing a sound tells the address space where the buffer is, and a second
    /// telling is a region laid out twice.
    pub fn streams_it_again(&self, sound: &orb_sim::Sound, track: Track, left: u32) {
        self.streams_it_at(STREAM, SOURCE, sound, track, left);
    }

    /// **And another track through objects the game made somewhere else**, which is what a track change really
    /// leaves: the stop deletes the stream and its source (0x44cf18) and the play makes new ones (0x45cb93), so
    /// where they land is the allocator's to say and not the same place every time.
    ///
    /// Its own call because nothing else needs it: what it is for is a restore that skips *the addresses it
    /// wrote down* rather than the ones the sound is at now — which is a restore writing over the stream the
    /// game is playing through.
    pub fn streams_it_again_elsewhere(&self, sound: &orb_sim::Sound, track: Track, left: u32) {
        self.streams_it_at(ANOTHER_STREAM, ANOTHER_SOURCE, sound, track, left);
    }

    fn streams_it_at(
        &self,
        stream: usize,
        source: usize,
        sound: &orb_sim::Sound,
        track: Track,
        left: u32,
    ) {
        let space = self.space();
        for (base, bytes) in [
            (stream, super::STREAM_BYTES),
            (source, super::SOURCE_BYTES),
            (BUFFER_WRAPPER, size_of::<usize>()),
        ] {
            if space.has_room(base, bytes) {
                space.map(base, bytes, Kind::Private);
            }
        }
        space.write::<usize>(super::G_SOUND_PLAYER + super::STREAM, stream);
        space.write::<usize>(stream + super::STREAM_BUFFER, BUFFER_WRAPPER);
        space.write::<usize>(BUFFER_WRAPPER, sound.buffer_object());
        // And how many buffers that array is, which is what the game's own take-down of this stream walks —
        // see `STREAM_BUFFER_COUNT`. One, this being the one buffer a laid-out sound is played through.
        space.write::<u32>(stream + STREAM_BUFFER_COUNT, 1);
        space.write::<u32>(stream + super::STREAM_BUFFER_BYTES, sound.buffer_size());
        space.write::<u32>(stream + super::STREAM_CHUNK, sound.notify_size());
        space.write::<u32>(stream + super::STREAM_WRITE_AT, 0);
        space.write::<usize>(stream + super::STREAM_SOURCE, source);
        space.write::<usize>(source + super::SOURCE_FILE, sound.mmio());
        space.write::<u32>(source + super::SOURCE_LEFT, left);
        space.write::<usize>(source + super::SOURCE_RECORD, record_named(track.name));
    }

    /// Which stream the player names now and how much of the track that one believes is left, which is the
    /// countdown the loop is taken on — read wherever that stream is.
    ///
    /// Both, because the pair is what a restore can get wrong: the field naming another object is one thing and
    /// the object's own countdown having been written over is another. Nothing where the chain is broken, which
    /// is a player naming no stream at all.
    pub fn the_stream_its_player_names(&self) -> Option<(usize, u32)> {
        let stream = mem::read_committed::<usize>(super::G_SOUND_PLAYER + super::STREAM)
            .filter(|it| *it != 0)?;
        let source =
            mem::read_committed::<usize>(stream + super::STREAM_SOURCE).filter(|it| *it != 0)?;
        let left = mem::read_committed::<u32>(source + super::SOURCE_LEFT)?;
        Some((stream, left))
    }

    /// The stream where the game leaves it when nothing is playing, which is what the player's own field says
    /// before a track is loaded and after one is taken down.
    pub fn plays_no_track(&self) {
        self.space()
            .write::<usize>(super::G_SOUND_PLAYER + super::STREAM, 0);
    }

    /// And that same field written back naming the stream the take-down deleted, which is what a restore's
    /// memory copy does with the value its snapshot holds — and after that copy is where orb reads which track
    /// the chapter was under, the read going through this very pointer.
    pub fn names_the_stream_it_had(&self) {
        self.space()
            .write::<usize>(super::G_SOUND_PLAYER + super::STREAM, STREAM);
    }

    /// And the same stream with a **buffer that is no longer a live object**, which is what a released COM
    /// object leaves behind: the word at its head pointing at the game's own data rather than into an image.
    pub fn frees_the_stream_buffer(&self) {
        self.space().write::<usize>(BUFFER_WRAPPER, SOURCE);
    }

    /// And the file forgotten, which is a stream orb cannot move the position of: every seek of a handle of
    /// nothing is refused, so a capture holds no position and a restore puts back everything else.
    pub fn forgets_the_file_handle(&self) {
        self.space().write::<usize>(SOURCE + super::SOURCE_FILE, 0);
    }

    /// The countdown and the next write, read back and written: the two the game's own memory carries and a
    /// restore has to put back with the buffer.
    pub fn bytes_left(&self) -> u32 {
        self.space().read::<u32>(SOURCE + super::SOURCE_LEFT)
    }

    pub fn set_bytes_left(&self, left: u32) {
        self.space().write::<u32>(SOURCE + super::SOURCE_LEFT, left);
    }

    /// The stream the sound player names, which is the pointer its own stop deletes what it finds behind and
    /// clears: zero is a player with no track of any kind, and any other value a live object it may delete.
    pub fn the_stream_it_names(&self) -> usize {
        self.space()
            .read::<usize>(super::G_SOUND_PLAYER + super::STREAM)
    }

    pub fn next_write_offset(&self) -> u32 {
        self.space().read::<u32>(STREAM + super::STREAM_WRITE_AT)
    }

    pub fn set_next_write_offset(&self, at: u32) {
        self.space()
            .write::<u32>(STREAM + super::STREAM_WRITE_AT, at);
    }

    /// The three the music's own ranges are, for a test that says which memory a restore has to leave alone.
    pub fn the_sound_systems_own_memory(&self) -> Vec<Range<usize>> {
        vec![
            super::G_SOUND_PLAYER..super::G_SOUND_PLAYER + super::SOUND_PLAYER_BYTES,
            STREAM..STREAM + super::STREAM_BYTES,
            SOURCE..SOURCE + super::SOURCE_BYTES,
        ]
    }

    /// The music thread's id where the game's own `CreateThread` would have left it.
    pub fn the_music_thread_is(&self, id: u32) {
        self.space()
            .write::<u32>(super::G_SOUND_PLAYER + super::MUSIC_THREAD_ID, id);
    }

    /// The game's own two music calls and the one that carries them out, handed over — and a way to ask what
    /// orb asked of them.
    ///
    /// What the real ones do is branch on the sound backend and either command the music thread or drive the
    /// slot object; what these do is queue and count, because what a test can ask is that orb took the music
    /// down before the copy, asked for the right track after, and ran the command rather than leaving it for
    /// the next frame — the three a restore depends on.
    pub fn hands_over_the_music(&self) -> impl Fn() -> (u32, u32) {
        MUSIC_STOPPED.store(0, Ordering::Relaxed);
        MUSIC_STARTED.store(0, Ordering::Relaxed);
        QUEUED_PLAYS.lock().unwrap().clear();
        TRACKS_STARTED.lock().unwrap().clear();
        SOUND_COMMANDS_RUN.store(0, Ordering::Relaxed);
        *MADE_A_STREAM_OVER.lock().unwrap() = None;
        // The name the game's own two calls hand `PlayBGM` — see `Th07`'s `MUSIC_FILE` — laid where the game
        // keeps it, so that what orb passes is a name and not an address nothing reads.
        // As much room as a name is read out of, which is what makes the read answer at all: a name is read as
        // the whole of that buffer and a shorter mapping is a read of memory that is not there.
        let space = self.space();
        if space.has_room(super::MUSIC_FILE, MUSIC_FILE_NAME_BYTES) {
            space.map(super::MUSIC_FILE, MUSIC_FILE_NAME_BYTES, Kind::Private);
        }
        space.write_bytes(super::MUSIC_FILE, MUSIC_FILE_NAME.to_bytes_with_nul());
        super::set_stop_music(stops_the_music as *const () as usize);
        super::set_start_music(starts_the_music as *const () as usize);
        super::set_play_bgm(plays_the_bgm as *const () as usize);
        super::set_play_sounds(runs_the_sound_commands as *const () as usize);
        || {
            (
                MUSIC_STOPPED.load(Ordering::Relaxed),
                MUSIC_STARTED.load(Ordering::Relaxed),
            )
        }
    }

    /// The queued plays carried out, which is what the game's own frame does once whether orb asked for it or
    /// not — see [`runs_the_sound_commands`].
    pub fn runs_the_sound_commands(&self) {
        runs_the_sound_commands(super::G_SOUND_PLAYER);
    }

    /// The pause menu's own 「最初からやり直す」 chosen, which is the run asked to be given up and begun again:
    /// the scene the supervisor is asked for, set the way 0x403908 sets it — see `Th07`'s `SCENE_START_OVER`.
    ///
    /// The item itself is a sprite rather than a string and its menu is the game's own, so what a test says is
    /// the ask that item leaves behind it.
    pub fn the_pause_menu_asks_to_start_over(&self) {
        self.space()
            .write::<i32>(super::G_SUPERVISOR + super::SUPERVISOR_WANTED, 0xa);
    }

    /// And the game's own script asking for one of its slots, which is what fills that queue: a stage's build
    /// asks for the stage's song and the ECL asks for the fight's — see [`asks_for_a_song`]. Nothing carries it
    /// out until a frame's sound runs.
    pub fn the_script_asks_for_a_song(&self, slot: u32) {
        asks_for_a_song(slot);
    }

    /// And run until the play queued in it has been carried out, which is what the game's own stage build does
    /// at 0x42f238 — see [`RUNS_A_PLAY_TAKES`] for why one run is not enough.
    pub fn carries_the_play_out(&self) {
        for _ in 0..RUNS_A_PLAY_TAKES {
            self.runs_the_sound_commands();
        }
    }

    /// And how many times that has happened, which is the game's own sound work counted: a frame's is one, and
    /// a second one inside a frame is orb asking the game to carry a queued command out itself.
    pub fn times_the_sound_commands_ran(&self) -> u32 {
        SOUND_COMMANDS_RUN.load(Ordering::Relaxed)
    }

    /// And which tracks came out of the plays that have been carried out, in the order they were: the record's
    /// own name each time, so what a test reads back is the file's word for the track and not orb's for the
    /// slot.
    pub fn music_started(&self) -> Vec<String> {
        TRACKS_STARTED.lock().unwrap().clone()
    }

    /// And the file the last stream was made over, which is the name `PlayBGM` was handed — see
    /// [`plays_the_bgm`]. Nothing where no stream has been made.
    pub fn made_a_stream_over(&self) -> Option<String> {
        MADE_A_STREAM_OVER.lock().unwrap().clone()
    }

    /// The three of the game's own that a practice-score read is made of, handed over — and a way to ask what
    /// orb did with them.
    ///
    /// What the real ones do is read the file, copy its `PSCR` records into a table and give the buffer back;
    /// what these do is record the order and the arguments, because what a test can ask is that orb read the
    /// mode's own file, parsed into the game's own table, and gave the buffer back.
    pub fn hands_over_the_score_file(&self) -> impl Fn() -> (u32, usize, u32) {
        FILE_READ.store(0, Ordering::Relaxed);
        PARSED_INTO.store(0, Ordering::Relaxed);
        FILE_RELEASED.store(0, Ordering::Relaxed);
        // The image that read fills, which is the same one a save's own read fills — see
        // [`Image::lays_out_its_score_files_image`], laid out here as well because a launch that hands these
        // three over is a launch orb reads the file in.
        self.lays_out_its_score_files_image();
        super::set_read_score_file(reads_the_score_file as *const () as usize);
        super::set_parse_practice_scores(parses_the_practice_scores as *const () as usize);
        // And the card records' own parse, which orb calls here as well as hooks: what the front end read is
        // the game's own file's, and this is the call that lays the mode's own over it.
        super::set_parse_card_records(parses_the_card_records as *const () as usize);
        // And その他の記録's, which orb calls for the same reason: the two totals and the counts under them are
        // a record of runs, so the mode's own file is where they belong.
        super::set_parse_play_records(parses_the_play_records as *const () as usize);
        super::set_release_score_file(releases_the_score_file as *const () as usize);
        || {
            (
                FILE_READ.load(Ordering::Relaxed),
                PARSED_INTO.load(Ordering::Relaxed),
                FILE_RELEASED.load(Ordering::Relaxed),
            )
        }
    }

    /// The game's own seal, handed over — and a way to ask how many times orb called it.
    ///
    /// What the real one does is draw twice from the generator and salt the answers into the run's numbers;
    /// what this one does is count, because what a test can ask of orb is that it asked at all. A laid-out
    /// game has no code to run, which is `handed_over!`'s own reason.
    /// And its own text baking, which is the call a chapter put back inside a card puts that card's name on the
    /// plate through: what the name on the screen *is* is a sprite, so a game laid out by hand has to be able to
    /// answer the call that bakes one.
    ///
    /// The vm and both colours are held against the plate's own inside the call, so a baking of the right string
    /// onto the wrong thing fails there rather than reading here as the plate.
    pub fn hands_over_the_text_baking(&self) {
        BAKINGS.store(0, Ordering::Relaxed);
        BAKED_FROM.store(0, Ordering::Relaxed);
        super::set_draw_string(bakes_a_string as *const () as usize);
    }

    /// How many strings have been baked, and what the last of them said — the string as the call was given it,
    /// read out of the game's own memory up to its terminator, which is how the call itself would read one.
    pub fn bakings(&self) -> u32 {
        BAKINGS.load(Ordering::Relaxed)
    }

    pub fn the_plate_says(&self) -> Vec<u8> {
        let from = BAKED_FROM.load(Ordering::Relaxed);
        if from == 0 {
            return Vec::new();
        }
        let space = self.space();
        (0..)
            .map(|byte| space.read::<u8>(from + byte))
            .take_while(|byte| *byte != 0)
            .collect()
    }

    /// A card named the way **the game itself** names one, which is what makes the plate worth reading back:
    /// 0x428887 bakes the name the declaration carries where a card starts, and orb bakes it again out of the
    /// record where a chapter is put back inside one. Both end at the same plate, so what a test asks is what
    /// the plate says and not who last said it.
    ///
    /// The record's own copy is what goes on, which is the copy the game makes at 0x40ffb4 — so a card named
    /// here says the same thing whichever of the two baked it.
    pub fn the_game_names_the_card(&self, card: usize) {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES + super::CARD_NAME;
        BAKED_FROM.store(at, Ordering::Relaxed);
        BAKINGS.fetch_add(1, Ordering::Relaxed);
    }

    /// How many times orb has asked for each of the two music calls, for an e2e test that keeps no closure of
    /// its own: the same two counts [`Image::hands_over_the_music`] answers with, asked of the image instead.
    pub fn music_calls(&self) -> (u32, u32) {
        (
            MUSIC_STOPPED.load(Ordering::Relaxed),
            MUSIC_STARTED.load(Ordering::Relaxed),
        )
    }

    pub fn hands_over_the_seal(&self) -> impl Fn() -> u32 {
        SEALED.store(0, Ordering::Relaxed);
        SEED_WHEN_SEALED.store(0, Ordering::Relaxed);
        super::set_reseal(seals_the_numbers as *const () as usize);
        || SEALED.load(Ordering::Relaxed)
    }

    /// The seed the generator held when that seal ran — see [`SEED_WHEN_SEALED`].
    pub fn the_seed_it_sealed_over(&self) -> u16 {
        SEED_WHEN_SEALED.load(Ordering::Relaxed) as u16
    }

    /// And the border going up, counted the same way: what the real one does is put the player behind it for
    /// 540 frames, and what a test can ask is that orb asked for it where the cherry+ had earned one.
    pub fn hands_over_the_border(&self) -> impl Fn() -> u32 {
        BORDERED.store(0, Ordering::Relaxed);
        super::set_border_up(puts_a_border_up as *const () as usize);
        || BORDERED.load(Ordering::Relaxed)
    }

    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// The host this game is laid out in, for an e2e test that needs more of it than the memory — the
    /// display it declares, or which window is in front.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }
}

/// How many times orb has asked the game to seal its numbers, and the `__thiscall` that counts.
///
/// A static because the address handed over has to be a function's, and the seam takes an address rather
/// than a closure — the same shape the e2e tests' own three handed-over calls have.
static SEALED: AtomicU32 = AtomicU32::new(0);

/// And the seed standing in the generator when it ran, which is what makes the *order* of a restore readable:
/// the real seal draws twice from the generator, so a seed put back before it is not the seed the stage then
/// runs on. Recorded rather than reproduced — what those two draws leave is not measured, and a step invented
/// here would be a number out of orb rather than out of the game.
static SEED_WHEN_SEALED: AtomicU32 = AtomicU32::new(0);

extern "thiscall" fn seals_the_numbers(_game_manager: usize) {
    SEALED.fetch_add(1, Ordering::Relaxed);
    let drawing_from = unsafe { mem::read::<u16>(super::G_RANDOM) };
    SEED_WHEN_SEALED.store(u32::from(drawing_from), Ordering::Relaxed);
    // **And the generator moved, because the real one draws from it twice.** What the two draws come to is not
    // known and not pretended at — a step of this file's own stands for them — but that they *happen* is the
    // whole of why the game saves the generator around its own call to this (0x42f07e, 0x42f094) and why orb
    // does the same. A laid-out seal that left the generator alone is one no test could ask that of.
    unsafe {
        mem::write::<u16>(super::G_RANDOM, drawing_from.rotate_left(7) ^ 0x51ed | 1);
    }
    seals_the_pair();
}

/// **The word and its copy the check holds against each other**, which is the part of [`super::RESEAL`] a test
/// can be written over: 0x4012b0 draws `[run+0x3c]` afresh and writes it into `[run+0xac]`, and 0x404fe0
/// compares the two. The word is drawn out of the generator rather than left as it was, since a copy of an
/// unchanged word would pass the check whatever a resume wrote into it in between.
///
/// The generator's own step is the caller's: [`seals_the_numbers`] has already taken one for the real call's two
/// draws, and a build's draws stand for its reseal's.
fn seals_the_pair() {
    let word = unsafe { mem::read::<u16>(super::G_RANDOM) };
    let sealed = i32::from(word) % 0x186a0 + 0x198f;
    unsafe {
        mem::write::<i32>(RUN_NUMBERS.start + super::SEALED_WORD, sealed);
        mem::write::<i32>(RUN_NUMBERS.start + super::SEALED_COPY, sealed);
    }
}

/// And how many times orb has asked for the border the cherry+ earns, counted for the same reason.
static BORDERED: AtomicU32 = AtomicU32::new(0);

extern "thiscall" fn puts_a_border_up(_player: usize) {
    BORDERED.fetch_add(1, Ordering::Relaxed);
}

/// The baking as the call itself sees it: how many strings orb has baked, and where the last one was read from.
///
/// Which vm it went into is not kept — it is held against the plate's own here instead, so a call that named
/// another vm fails inside the call rather than being read back as the plate.
static BAKINGS: AtomicU32 = AtomicU32::new(0);
static BAKED_FROM: AtomicUsize = AtomicUsize::new(0);

extern "cdecl" fn bakes_a_string(manager: usize, vm: usize, colour: u32, shadow: u32, text: usize) {
    assert_eq!(
        manager, QUAD_QUEUE.start,
        "a string was baked through something that is not the sprite manager"
    );
    assert_eq!(
        vm,
        GUI.start + super::CARD_NAME_VM,
        "a string was baked into a vm that is not the one a card's name is shown on"
    );
    assert_eq!(colour, CARD_NAME_COLOUR, "another colour");
    assert_eq!(shadow, CARD_NAME_SHADOW, "another second colour");
    BAKED_FROM.store(text, Ordering::Relaxed);
    BAKINGS.fetch_add(1, Ordering::Relaxed);
}

/// And the two music calls, counted the same way and for the same reason.
static MUSIC_STOPPED: AtomicU32 = AtomicU32::new(0);
static MUSIC_STARTED: AtomicU32 = AtomicU32::new(0);

/// The plays orb has asked for and nothing has carried out yet, and the tracks the ones carried out came out
/// as.
///
/// **A queue because the real call leaves one.** On the backend a launch of this game makes, 0x439ec1 writes a
/// command into the player's own queue and returns, and what carries a command out is `PLAY_SOUNDS` — so what
/// track a slot means is decided later, and a play orb never runs is one nothing has happened for.
///
/// The stop is counted rather than queued: what it is *for* is a streaming thread being got rid of before a
/// restore writes over its object, and a laid-out game has no such thread for anything here to say that about.
static QUEUED_PLAYS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static TRACKS_STARTED: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// And the file a stream was last made over, which is what [`plays_the_bgm`] is given.
static MADE_A_STREAM_OVER: Mutex<Option<String>> = Mutex::new(None);
/// The name the game's own calls hand `PlayBGM`, which is the file every track of this game is in — see
/// `Th07`'s `MUSIC_FILE` for where the game keeps it and which calls pass it.
const MUSIC_FILE_NAME: &CStr = c"thbgm.dat";
/// And the name the game's own sixth stage loads its second phase's song by, which is a string of the exe's —
/// see `Th07`'s `SECOND_PHASE_SONG`.
const SECOND_PHASE_SONG_NAME: &CStr = c"bgm/th07_13b.mid";
/// How much of a name that call reads, which is the room the player keeps the one it is given in — `+0x38bc`,
/// the buffer 0x44c041 copies into.
const MUSIC_FILE_NAME_BYTES: usize = 0x100;
/// And how many times the queue has been run, which is the game's own sound work: once a frame, where the
/// frame's own call is the only thing running it.
static SOUND_COMMANDS_RUN: AtomicU32 = AtomicU32::new(0);
/// And how many of those runs the play now queued has been walked over — see [`RUNS_A_PLAY_TAKES`].
static PLAYS_WALKED: AtomicU32 = AtomicU32::new(0);

extern "thiscall" fn stops_the_music(_supervisor: usize) {
    MUSIC_STOPPED.fetch_add(1, Ordering::Relaxed);
    // The stream deleted and the player's pointer to it cleared, which is what the stop command's own teardown
    // does at 0x44cebd — and the half of the state anything above the game can see, every read of the music
    // going through that pointer.
    unsafe { mem::write::<usize>(super::G_SOUND_PLAYER + super::STREAM, 0) };
    // And the buffer released with the stream it belonged to, which is the half of a stop that is not memory:
    // `StopBGM` frees both, and a restore writing the pointers back cannot make the object behind them live
    // again. So a seek before the game has made a new one is a seek of a released buffer, and fails as one.
    orb_sim::Sound::taken_down();
}

/// **Asking for a song is asking for one of the player's slots**, which is what the stage's build, the script
/// and a restore all do — and what it leaves is a command in the player's queue rather than a track playing.
/// The stream that command needs is `PlayBGM`'s to make; see [`plays_the_bgm`].
///
/// **The slot is carried and not checked here**, and that is the point of the queue below: what this stood in
/// for once was `assert_eq!(slot, Th07::MUSIC_SLOT)`, an assertion held against the constant orb passes, which
/// no wrong slot can fail. A stand-in for a call that *resolves* something has to resolve it the way the game
/// does, or the test over it is a test of nothing — the launch that found this heard the title screen's song
/// where a chapter's own should have come back, with the suite green.
extern "thiscall" fn starts_the_music(_supervisor: usize, slot: u32) {
    asks_for_a_song(slot);
}

fn asks_for_a_song(slot: u32) {
    MUSIC_STARTED.fetch_add(1, Ordering::Relaxed);
    QUEUED_PLAYS.lock().unwrap().push(slot);
    // And the player's own queue saying so, which is what orb reads to know the ask has not been carried out
    // yet — see `Th07`'s `SOUND_COMMANDS`. The kind is a kind rather than this file's own number: what is asked
    // of it is only that an empty slot and a full one are told apart.
    unsafe { mem::write::<u32>(super::G_SOUND_PLAYER + super::SOUND_COMMANDS, A_PLAY) };
}

/// The kind a queued play is, as the player's own first slot holds it — see `Th07`'s `SOUND_COMMANDS` for how a
/// run of the sound tells an empty slot from a full one, which is all this stands for.
const A_PLAY: u32 = 2;

/// How many runs of the game's own sound a play takes before anything is audible: **two**, which is a laid-out
/// game walking the phases the real player walks rather than finishing a command in the run it was queued in.
///
/// Without it every e2e test over a restore's sound passes on a game that cannot be wrong about the timing, and
/// two launches of 妖々夢 were what said so: one read `restore: the song could not be put` for a seek made the
/// frame after the ask, and one heard the stage's own song start after orb had stopped it in that same frame.
/// What the real player does is advance the command's phase at `+0x8` and return (0x44ca96), so the run the ask
/// was made in is not the run that carries it out.
const RUNS_A_PLAY_TAKES: u32 = 2;

/// `SoundPlayer::PlaySounds` as far as the music goes: every queued play carried out the way command 2 carries
/// one out — the name the slot holds, the record of the track that name is, and the stream's own source put on
/// it.
///
/// A slot nothing was loaded into is a name no record carries, and that is the file's first track rather than
/// nothing at all: the same fallback the player's own lookup takes.
/// `CSound::Stop` over the buffers the player's stream owns, as the play command's first phase does it — see
/// the call in [`runs_the_sound_commands`].
///
/// Nothing where the count is zero: a stream that owns no buffers has none to stop, which is what a stream
/// whose buffers DirectSound has already taken back is.
fn takes_the_stream_down(stream: usize) {
    let count = mem::read_committed::<u32>(stream + STREAM_BUFFER_COUNT).unwrap_or(0);
    let Some(array) = mem::read_committed::<usize>(stream + super::STREAM_BUFFER) else {
        return;
    };
    for at in 0..count as usize {
        let Some(buffer) = mem::read_committed::<usize>(array + at * size_of::<usize>()) else {
            continue;
        };
        orb_api::dsound::stop(orb_api::SoundBuffer(buffer));
    }
}

/// And the buffer filled from the source, which is the play command's last phase: 0x44cd1d into 0x45df50,
/// which refuses outright where the stream's own buffer array holds nothing (0x45df5f) and otherwise locks the
/// buffer that array's first entry names.
///
/// Whether that entry is a buffer anybody still owns is what a restore decides, so this is where a play walking
/// a stream the memory copy resurrected shows: a lock is a call on the buffer, and a released one refuses it.
fn fills_the_buffer(stream: usize) -> bool {
    let Some(array) = mem::read_committed::<usize>(stream + super::STREAM_BUFFER) else {
        return false;
    };
    let Some(buffer) = mem::read_committed::<usize>(array).filter(|it| *it != 0) else {
        return false;
    };
    let Some(bytes) = mem::read_committed::<u32>(stream + super::STREAM_BUFFER_BYTES) else {
        return false;
    };
    let buffer = orb_api::SoundBuffer(buffer);
    let (asked, locked) = orb_api::dsound::lock(buffer, 0, bytes, 0);
    if asked < 0 {
        return false;
    }
    orb_api::dsound::unlock(buffer, locked);
    true
}

/// `SoundPlayer::PlayBGM`, which is the only call that *makes* a stream — see `Th07`'s `PLAY_BGM`.
///
/// **What it makes is one stream over the whole music file, and no track**: the name it is given is that file's,
/// and which of the file's records is playing is a play's to decide. So this puts the stream and its buffer
/// where a laid-out game keeps them and leaves the source's record alone.
///
/// **The name is written down rather than assumed**, for the reason [`asks_for_a_song`] carries its slot rather
/// than checking it: a stand-in that ignores what it is given cannot fail on being given the wrong thing, and a
/// launch of 妖々夢 handed this call a *track's* name — the game answered -1, left the player holding no stream
/// at all, and its own sound then read through that null field and faulted at 0x44d023.
extern "thiscall" fn plays_the_bgm(_player: usize, name: usize) -> i32 {
    let name = super::name_at(name, MUSIC_FILE_NAME_BYTES).unwrap_or_default();
    *MADE_A_STREAM_OVER.lock().unwrap() = Some(String::from_utf8_lossy(&name).into_owned());
    unsafe {
        mem::write::<usize>(super::G_SOUND_PLAYER + super::STREAM, STREAM);
        // The buffer this stream owns, which a made stream owns one of — see [`STREAM_BUFFER_COUNT`].
        mem::write::<u32>(STREAM + STREAM_BUFFER_COUNT, 1);
    }
    // The buffer live again, this being where the game makes the one it plays a track through.
    orb_sim::Sound::started_again();
    0
}

extern "thiscall" fn runs_the_sound_commands(_player: usize) {
    SOUND_COMMANDS_RUN.fetch_add(1, Ordering::Relaxed);
    // A phase advanced and the command left where it is, which is what a run that is not a command's last does
    // — see [`RUNS_A_PLAY_TAKES`]. The queue is what orb reads, so it is emptied on the run that carries the
    // play out and not before.
    let walked = PLAYS_WALKED.fetch_add(1, Ordering::Relaxed) + 1;
    if QUEUED_PLAYS.lock().unwrap().is_empty() {
        PLAYS_WALKED.store(0, Ordering::Relaxed);
        return;
    }
    if walked < RUNS_A_PLAY_TAKES {
        return;
    }
    PLAYS_WALKED.store(0, Ordering::Relaxed);
    let queued: Vec<u32> = QUEUED_PLAYS.lock().unwrap().drain(..).collect();
    unsafe { mem::write::<u32>(super::G_SOUND_PLAYER + super::SOUND_COMMANDS, 0) };
    // **A play makes no stream, and that is the whole of why this walks the phases the game's does.** The
    // command reuses the one the player's own field names: 0x44cc23 stops the buffers it owns, 0x44cce1 puts
    // the record the slot names on its source, and 0x44cd1d fills its buffer from that source. What *makes* a
    // stream is `PlayBGM` and nothing else — see [`plays_the_bgm`] — so a play asked for while that field
    // names nothing is dropped where the game drops it, at 0x44cbf3.
    let stream =
        mem::read_committed::<usize>(super::G_SOUND_PLAYER + super::STREAM).filter(|it| *it != 0);
    let Some(stream) = stream else {
        return;
    };
    takes_the_stream_down(stream);
    for slot in queued {
        let record = slot_holds(slot)
            .and_then(|name| record_of(&name))
            .unwrap_or(MUSIC_RECORDS);
        unsafe { mem::write::<usize>(SOURCE + super::SOURCE_RECORD, record) };
        // And nothing audible where the buffer would not take the sound, which is a stream whose buffer is not
        // one anybody owns any more.
        if !fills_the_buffer(stream) {
            continue;
        }
        orb_sim::Sound::started_again();
        let track = super::name_at(record + super::RECORD_NAME, super::RECORD_NAME_BYTES)
            .unwrap_or_default();
        TRACKS_STARTED
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(&track).into_owned());
    }
}

/// `0x44a302(kind)`, as far as the kind that writes goes: the file read into an image, its own records laid over
/// the ones in memory through the parse below, and then memory written back out to the file.
///
/// **Those three in that order, which is the whole of what makes this worth laying out**: the parse is what a
/// session's counts have to survive, and a laid-out save that only wrote memory out could not tell a fix that
/// adds them at that seam from one that does nothing at all.
///
/// The other kinds build the ranking screen and write nothing — see `Th07`'s `SAVE_SCORE_FILE` — so a laid-out
/// game answers them by doing nothing, which is what they do to the file.
extern "fastcall" fn saves_its_records(kind: u32) -> i32 {
    SAVES_ASKED_FOR.fetch_add(1, Ordering::Relaxed);
    if kind != super::SAVE_KIND_WRITES {
        return 0;
    }
    // The phase that kind leaves in the object, and the gather made *on* that object: the read is what a
    // put-back is decided by — see `Th07::its_records_are_about_to_be_written` — so a laid-out save that called
    // the parse straight would be one no e2e test could ask that of. Through orb's own hook where a launch
    // installed it, the way every other callback of this game reaches it.
    unsafe {
        mem::write::<i32>(
            SCORE_SCREEN.start + super::RESULT_PHASE,
            super::SAVE_PHASE_WRITES,
        )
    };
    *GATHERING_ON.lock().unwrap() = SCORE_SCREEN.start;
    let screen = SCORE_SCREEN.start as *mut std::ffi::c_void;
    if crate::runtime::RANKING_READ.load(Ordering::Relaxed) == 0 {
        gathers_and_writes(screen);
    } else {
        unsafe { crate::runtime::ranking_read_in_ecx(screen) };
    }
    0
}

/// `0x449b05` as far as the card records go: the file read into an image, its own records laid over the ones in
/// memory through the parse, and then memory written back out to the file — the three halves of a save, in the
/// order the game walks them.
///
/// Its own function because orb's hook goes *over* it: what the hook brackets is this call, and the parse inside
/// it is a second hook of its own.
///
/// `__cdecl` where the game's own is `__thiscall`, and the screen is why that is not a mismatch: nothing here
/// reads it — which object the read is made on is left in [`GATHERING_ON`] by whoever drives it — so which
/// register or stack slot it arrived in is a question this function never asks. A caller that passes it in `ecx`
/// pushes nothing, and a callee that takes nothing off the stack leaves none behind.
extern "C" fn gathers_and_writes(_screen: *mut std::ffi::c_void) -> i32 {
    let on = *GATHERING_ON.lock().unwrap();
    let held = score_file(&SAVING_FROM.lock().unwrap().clone());
    unsafe {
        mem::write::<u32>(SCORE_FILE_IMAGE + FILE_CHUNKS_AT, FILE_HEADER_BYTES as u32);
        mem::write::<u32>(
            SCORE_FILE_IMAGE + FILE_BYTES,
            (FILE_HEADER_BYTES + held.len()) as u32,
        );
        mem::write_bytes(SCORE_FILE_IMAGE + FILE_HEADER_BYTES, &held);
    }
    // Through orb's own hook where a launch installed one, the way every other callback of this game reaches
    // it: what that hook is for is this moment, and a laid-out save that called past it would be a save no e2e
    // test could say anything about.
    let parse: unsafe extern "fastcall" fn(usize, usize) -> i32 =
        if crate::runtime::CARD_RECORDS_READ.load(Ordering::Relaxed) == 0 {
            parses_the_card_records
        } else {
            crate::runtime::card_records_read
        };
    unsafe { parse(SCORE_FILE_IMAGE, super::G_CARD_RECORDS) };
    // **And written only where the phase says so**, which the game gates the write on at 0x44a1e0: the two the
    // ranking screen is built with reach this read and leave the file exactly as it was. A laid-out gather that
    // wrote at every read would be one that cannot tell what a look at the ランキング costs.
    if mem::read_committed::<i32>(on + super::RESULT_PHASE) != Some(super::SAVE_PHASE_WRITES) {
        return 0;
    }
    // And the file written out of memory, which is where 0x44552c takes it from: every record that carries the
    // tag, in the order the cards are in.
    let mut written = Vec::new();
    for card in 0..super::CARDS {
        let at = super::G_CARD_RECORDS + card * super::CARD_RECORD_BYTES;
        if mem::read_committed::<u32>(at) != Some(super::CARD_RECORD_TAG) {
            continue;
        }
        written.extend_from_slice(&unsafe { mem::read_bytes(at, super::CARD_RECORD_BYTES) });
    }
    // Where that open was refused there is no file to write, which is what a run nobody could lose leaves: the
    // records stand in memory and nothing of them reaches a disk.
    if let Some(name) = SAVING_INTO.lock().unwrap().as_deref() {
        set_score_file(name, written);
    }
    0
}

/// And 0x445069, the parse that lays the file's own records over the ones in memory: every `CATK` chunk of the
/// image copied whole into the record of the card its own `+0x28` names.
///
/// The version byte and the card number are read the way the game reads them, since what a chunk is for is to be
/// found by them — a chunk naming a card past the array's end is one the game walks past.
unsafe extern "fastcall" fn parses_the_card_records(image: usize, records: usize) -> i32 {
    let Some(chunks) = mem::read_committed::<u32>(image + FILE_CHUNKS_AT) else {
        return -1;
    };
    let Some(bytes) = mem::read_committed::<u32>(image + FILE_BYTES) else {
        return -1;
    };
    let mut at = image + chunks as usize;
    let end = image + bytes as usize;
    while at + super::CARD_RECORD_BYTES <= end {
        let card = mem::read_committed::<u16>(at + CHUNK_CARD).unwrap_or_default() as usize;
        if mem::read_committed::<u32>(at) == Some(super::CARD_RECORD_TAG) && card < super::CARDS {
            let chunk = unsafe { mem::read_bytes(at, super::CARD_RECORD_BYTES) };
            unsafe {
                mem::write_bytes(records + card * super::CARD_RECORD_BYTES, &chunk);
            }
        }
        at += super::CARD_RECORD_BYTES;
    }
    0
}

/// How many times the game has been asked to save, which is what says orb asked at all.
static SAVES_ASKED_FOR: AtomicU32 = AtomicU32::new(0);

/// Which file the save now running opens, since a laid-out game opens none itself: what decides it is orb's own
/// fork, so the name is asked of that fork by whoever drives the save and left here for it — see
/// [`Image::saves_its_score_file`].
static SAVING_FROM: Mutex<String> = Mutex::new(String::new());

/// And which object the read now running is made on, since the stand-in for that read is `__cdecl` and cannot
/// take a `this` out of `ecx` — see [`gathers_and_writes`], which reads the phase through it to decide whether a
/// write follows.
static GATHERING_ON: Mutex<usize> = Mutex::new(0);

/// And which file it writes, or nothing where that open was refused — which is a session whose run is not a
/// score: the write fails and the file is left as it was, the read before it having been allowed.
static SAVING_INTO: Mutex<Option<String>> = Mutex::new(None);

/// The name a slot was loaded with, with the directory taken off it — the whole of what a play has to go on,
/// and what the player's own lookup strips before it compares.
fn slot_holds(slot: u32) -> Option<Vec<u8>> {
    let at = super::G_SOUND_PLAYER + SLOT_NAMES + slot as usize * SLOT_NAME_BYTES;
    let name = super::name_at(at, SLOT_NAME_BYTES)?;
    Some(super::file_name(&name).to_vec())
}

/// Which of the file's records a name is: the array walked from the player's own pointer field until an empty
/// name, which is where the lookup at 0x44bc01 stops.
fn record_of(name: &[u8]) -> Option<usize> {
    let array = mem::read_committed::<usize>(super::G_SOUND_PLAYER + SOUND_RECORDS)?;
    (0..)
        .map(|entry| array + entry * RECORD_STRIDE)
        .map_while(|record| {
            super::name_at(record + super::RECORD_NAME, super::RECORD_NAME_BYTES)
                .map(|held| (record, held))
        })
        .find(|(_, held)| held == name)
        .map(|(record, _)| record)
}

/// And the record a track this file is laying out belongs in, where a name no record carries is a test asking
/// for a track its own music file does not hold.
fn record_named(name: &CStr) -> usize {
    record_of(name.to_bytes())
        .unwrap_or_else(|| panic!("{name:?} is not a track this file's own music holds"))
}

/// The score file as the handed-over three see it: how many times it was read, what the parse was pointed at,
/// and how many times the buffer was given back.
static FILE_READ: AtomicU32 = AtomicU32::new(0);
static PARSED_INTO: AtomicUsize = AtomicUsize::new(0);
static FILE_RELEASED: AtomicU32 = AtomicU32::new(0);

/// **The file read into an image the parses can walk**, which is [`SCORE_FILE_IMAGE`] and the same one a save's
/// own read fills: what orb makes this call for is the two records inside, so a buffer with nothing in it would
/// be a read no parse could be asked anything about.
///
/// Which of the two files it is is asked of the fork, since a read here goes nowhere near `CreateFileA` —
/// `crate::score::the_file_now_opened`.
extern "fastcall" fn reads_the_score_file(_name: *const u8) -> usize {
    FILE_READ.fetch_add(1, Ordering::Relaxed);
    let held = score_file(crate::score::the_file_now_opened());
    unsafe {
        mem::write::<u32>(SCORE_FILE_IMAGE + FILE_CHUNKS_AT, FILE_HEADER_BYTES as u32);
        mem::write::<u32>(
            SCORE_FILE_IMAGE + FILE_BYTES,
            (FILE_HEADER_BYTES + held.len()) as u32,
        );
        mem::write_bytes(SCORE_FILE_IMAGE + FILE_HEADER_BYTES, &held);
    }
    SCORE_FILE_IMAGE
}

/// その他の記録 out of a file's own image, as 0x44547f takes them: the chunk tagged `PLST` copied whole over
/// whatever the caller gave.
unsafe extern "fastcall" fn parses_the_play_records(image: usize, records: usize) -> i32 {
    let Some(chunks) = mem::read_committed::<u32>(image + FILE_CHUNKS_AT) else {
        return -1;
    };
    let Some(bytes) = mem::read_committed::<u32>(image + FILE_BYTES) else {
        return -1;
    };
    let mut at = image + chunks as usize;
    let end = image + bytes as usize;
    while at + PLAY_RECORDS_BYTES <= end {
        if mem::read_committed::<u32>(at) == Some(PLAY_RECORDS_TAG)
            && mem::read_committed::<u8>(at + 8) == Some(1)
        {
            let chunk = unsafe { mem::read_bytes(at, PLAY_RECORDS_BYTES) };
            unsafe { mem::write_bytes(records, &chunk) };
            return 0;
        }
        // Chunk by chunk, by the size each carries — which is what the game's own walk steps by (0x4454ef).
        let step = mem::read_committed::<u16>(at + 4).unwrap_or_default() as usize;
        if step == 0 {
            return -1;
        }
        at += step;
    }
    0
}

extern "fastcall" fn parses_the_practice_scores(file: usize, into: usize) -> i32 {
    assert_eq!(
        file, SCORE_FILE_IMAGE,
        "the parse was handed something else"
    );
    PARSED_INTO.store(into, Ordering::Relaxed);
    0
}

extern "fastcall" fn releases_the_score_file(file: usize) {
    assert_eq!(file, SCORE_FILE_IMAGE, "another buffer was given back");
    FILE_RELEASED.fetch_add(1, Ordering::Relaxed);
}
