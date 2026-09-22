//! th08, as a [`Game`] that answers what has been measured and declines the rest.
//!
//! `th08.exe`, 840,704 bytes, `md5 77b6785e04a3406e50be68714a193650`. Every address below was read off
//! that exe with `i686-w64-mingw32-objdump -d -M intel` and nothing else: no table anybody published
//! and nothing carried across from th07's addresses, which are where the same things happen to live in
//! a different program. `asm.md` beside this file is that reading; the constants here are what orb does
//! with it.
//!
//! **What this game answers so far** is its frame, its scenes, its stage, its difficulty, its keys, the
//! screens and cursor of its front end, the generator, the stage's frame count, the numbers a run is
//! scored by — the score, the graze, the point items, the night's clock, the lives, the bombs, the misses
//! and the power — the player with its own box and its four speeds, the enemies and the bullets with
//! where each is and how big it is, the lasers with where each reaches and how wide it is, and the items
//! with what picking each one up does, which enemy the fight is with and how much life it has left,
//! whether a line of dialogue is up, the clock a midstage boundary would be a number on, the way out of
//! a run, and the numbers a stage of one is started again from. So `Game::what_a_bot_sees` fills a whole
//! frame and the rules steer a run of it, and a recipe can hand the game back to its own title.
//!
//! **And a chapter over that**, [`Th08::rewinds`] answering `true`: the two lists a snapshot has to leave
//! out are read — the sound's own state and the sheets of every `.anm` slot — and a launch of the real
//! game has taken a chapter, died in it and put it back. What is still declined is an enemy's hit boxes
//! past the first, the music a restore would name, and what a mode does *beside* a chapter — the ranking,
//! the jump to a stage, the panel's own tile. That method's doc says which is which. Every address is
//! this exe's, for the reason written at the top of `game/mod.rs`: one written down because it is where
//! another game keeps the same thing is the one thing that must not happen.
//!
//! **The frame is orb's**, which is what [`RENDER`] being patched means: one update and one draw a
//! frame on the display's own cadence, in place of the limiter and the frame-skip loop the game's own
//! frame is. What that took was not two more addresses but the frame's own *order* — the queue this game
//! empties before its draw chain and draws after it, and the fog flag it forces on in between — because
//! th07's first loop over its frame used addresses that were all correct and took the game down on its
//! first frame. See [`Th08::begin_drawing`], which is that order.
#![doc = include_str!("asm.md")]

/// The midstage table, in a file of its own because a `--chapter-tuning` pass writes one to paste over it.
pub mod chapters;
/// The numbers a stage of this game is started again from, and the file they go into — read out of this
/// game's own replay record, which is what `CLAUDE.md` asks a resume be shaped from.
pub mod resume;
/// What a bot playing this game is told of it, which is this game's own arrays read — see
/// `Game::what_a_bot_sees`.
mod sight;

/// This game's address space laid out, so that the reads above have something to land in with no game
/// running. Reachable from this crate's own tests and, through the `sim` feature, from the tests of the
/// crates that drive it — which is where the scenarios live.
#[cfg(any(test, feature = "sim"))]
pub mod image;

use std::ffi::c_void;
use std::ops::Range;
// The slots behind `handed_over!` want this only where a laid-out game can fill them — see the macro,
// whose other half is the constant itself.
#[cfg(any(test, feature = "sim"))]
use std::sync::atomic::AtomicUsize;
// And the two words `a_stage_has_just_been_built` carries across frames, which every build wants, and the
// phase the screen a run ends at was last said in.
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

/// **What the run's clock read when the stage now playing was built**, which `read_state` takes off it
/// to answer a stage's own frames — see [`STAGE_FRAMES`], which this game never restarts inside a run,
/// and [`Th08::a_stage_has_just_been_built`], which writes this on the frame the stage's numbers are in
/// place. Nought before any stage, which is also what a run's own first stage reads.
static STAGE_BEGAN_AT: AtomicU32 = AtomicU32::new(0);

use orb_api::Hwnd;
use orb_api::mem;
use orb_api::{Device, Viewport};

use crate::audio::Music;
use crate::game::song::{name_at, same_song};
use crate::game::{
    Boundary, Call, Entered, FileOpen, FrameCalls, Game, Hooks, Memory, Menu, Pad, PanelTile,
    Patch, Read, Reading, Rect, RunStart, State, Threads,
};
use crate::log;

/// th08 — 東方永夜抄, Imperishable Night — as the [`Game`] whose addresses are in this module.
pub struct Th08;

/// **The nearest hazard to the player, as [`Th08::hazards_near`] found it**: where it is, and how big
/// the game says it is — a whole width and height rather than a half, see [`ENEMY_SIZE`].
#[derive(Clone, Copy)]
struct Nearest {
    at: (f32, f32),
    size: (f32, f32),
}

/// **What one walk of this game's own arrays found**: how many enemies, bullets and items it reached,
/// and the nearest hazard of them to the player.
///
/// A type of its own rather than a tuple, because each count is held against the game's own and reading
/// which is which off the order they come back in is how that comparison gets made backwards.
struct Nearby {
    enemies: usize,
    bullets: usize,
    items: usize,
    nearest: Option<Nearest>,
}

/// The `IDirect3DDevice8` the one `CreateDevice` of this game made, which is the supervisor's `+0x8`.
///
/// Read off the vtable rather than off the create: WinMain compares `[[0x17ce760]+0xc]`'s answer against
/// `0x88760869` at 0x441b36, which is `D3DERR_DEVICENOTRESET`, so that slot is
/// `IDirect3DDevice8::TestCooperativeLevel` and this is a device. Five more slots agree — `+0x38` Reset,
/// `+0x3c` Present, `+0x88` BeginScene, `+0x8c` EndScene, `+0xa0` SetViewport — see `asm.md`.
const G_D3D_DEVICE: usize = 0x017ce760;

/// The game's own window, which is the object the frame runs on as well as its `+0x0`.
///
/// What `ShowWindow` (0x441c88), `MoveWindow` (0x441c9f) and `DestroyWindow` (0x441cab) are handed at
/// WinMain's teardown, and what 0x441e70 — the frame — takes in `ecx`.
const G_GAME_WINDOW: usize = 0x017ce700;

/// The chain, whose two lists are the run's update and the run's draw.
///
/// `mov ecx,0x164f548` at 0x441f48 and 0x441fe7, either side of this game's `BeginScene`, which is the
/// frame calling [`RUN_CALC_CHAIN`] and then [`RUN_DRAW_CHAIN`] on it. Its calc list's head is `+0x0`
/// and its draw list's is `+0x20` — 0x43cb73 adds the 0x20 before walking.
const G_CHAIN: usize = 0x0164f548;

/// The two the frame calls, and the two orb stands in front of: every element of the chain's calc list,
/// then every element of its draw list.
///
/// Both are `__thiscall` on [`G_CHAIN`] and share a prologue. An element is reached as
/// `mov ecx,[elem+0x1c]; call [elem+0x4]` — 0x43ca9b and 0x43ca9e — so what orb wraps here is every
/// callback of a frame at once, which is what the two seams are for.
const RUN_CALC_CHAIN: usize = 0x0043ca50;
const RUN_DRAW_CHAIN: usize = 0x0043cb60;
const RUN_CHAIN_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x18];

/// **One element of that list, whose three fields are the whole of what a walk of it needs**: the
/// callback at `+0x4` and the object it is called on at `+0x1c` — `mov ecx,[elem+0x1c]; call [elem+0x4]`
/// at 0x43ca9b and 0x43ca9e — with the next at `+0x14`.
///
/// **And the calc list's first element is the chain itself**, at its `+0x0`, not a pointer kept there:
/// 0x43ca6c takes `this` as the element it starts from, and its callback is null, which the walk skips
/// (0x43ca7f) — the draw list's the same at `this + 0x20` (0x43cb73). Read as a pointer, the first field
/// of that element led nowhere, and a Pointdevice run of the real game played to its end found no screen
/// on the chain to take the game's own offer of a replay off.
///
/// Read by [`Th08::the_jobs_a_frame_has`] and [`Th08::the_object_a_job_is_called_on`]: what a frame
/// *runs* goes through the seam orb already stands in, and these are for what is on the list.
const CHAIN_CALC_LIST: usize = 0x0;
const CHAIN_JOB_CALLBACK: usize = 0x4;
const CHAIN_JOB_NEXT: usize = 0x14;
const CHAIN_JOB_ARGUMENT: usize = 0x1c;

/// How far that walk goes before it gives up, a list longer than this being one orb has read wrong.
///
/// **Sixty-four, which is four times the longest list this game's own frame builds**: `asm.md`'s table of
/// everything the chain runs has sixteen entries at their own priorities, so a list of more than that is
/// either a pointer followed wrong or a game orb has no reading of.
const CHAIN_JOBS_AT_MOST: usize = 64;

/// **The game's own whole frame, which orb's loop replaces**: `__thiscall` on [`G_GAME_WINDOW`].
///
/// Reached at `mov ecx,0x17ce700; call 0x441e70` from the message pump at 0x441b15, on the frames
/// `IDirect3DDevice8::TestCooperativeLevel` answers `D3D_OK` for. What it is made of, read off 0x441e70
/// to 0x442051: the time out of 0x442190 into `[this+0x2c]`; a limiter over `[this+0x34]`, `[this+0x3c]`
/// and the double at 0x4b5bd8; the whole-output viewport; the calc chain; the sound queue; the
/// frame-skip counter `[this+0x10]` against the byte at 0x17ce893; `BeginScene`; the queue emptied, the
/// fog put out, the draw chain and the queue drawn; `SetTexture(0, null)`; `EndScene`; and the present.
/// orb's loop takes all of that out, as it takes th06's and th07's out: one update and one draw a frame,
/// handed over on the display's own cadence.
const RENDER: usize = 0x00441e70;
/// `push ebp; mov ebp,esp; sub esp,0x8` — position-independent, 6 bytes, read out of the exe at that
/// address.
const RENDER_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x08];

/// Where this game's `Present` is called from, which is the frame's last call.
///
/// 0x44207f is `[[0x17ce760]+0x3c]`, and the function it is in starts at 0x442060 — reached from the
/// frame at 0x44203d. So this is the whole of what orb's frame loop would have to call to put a frame on
/// the screen, the same as th07's 0x4345c0.
const PRESENT: usize = 0x00442060;

/// The sound manager and the call that empties its queue.
///
/// 0x45d790 reads `[0x18b8a68+0x610]`, answers 0 where that is nothing, and otherwise switches on
/// `[this+0x1f88] - 1` over eight cases. The frame calls it once (0x441f55) and WinMain's teardown loops
/// on it until it answers 0 (0x441ba4), which is what says it is a queue runner and not a mixer.
const PLAY_SOUNDS: usize = 0x0045d790;
const G_SOUND_PLAYER: usize = 0x018b8a68;

/// **How much of the address space that manager is**, which is what a restore has to leave alone:
/// `0x5224` bytes, off the `rep stos` of `0x1489` dwords its constructor at 0x45c190 opens with. The
/// same constructor writes `-1` into the eighty slot handles at `+0x408` and the twelve at `+0x620`,
/// which is what says those are its own fields and not statics beside it.
///
/// **And the address space agrees**: `0x18b8a68 + 0x5224` is `0x18bdc8c`, four bytes short of
/// [`G_QUAD_QUEUE`] at `0x18bdc90` — so the next static of the exe begins where this extent ends, which
/// an extent too long by a field would have run over.
const SOUND_PLAYER_BYTES: usize = 0x1489 * size_of::<u32>();

/// **The wrapper the manager keeps DirectSound in**, `+0x610`, and the four bytes of it: 0x45c214
/// allocates them, 0x4716e0 constructs them and 0x45c255 hangs them here, and every method of it tests
/// `[[this]]` — the interface — as its first act (0x471c70). The manager's own `+0x608` is that
/// interface again, which the teardown at 0x45c5ba releases through its vtable.
const SOUND_DEVICE: usize = 0x610;
const SOUND_DEVICE_BYTES: usize = size_of::<usize>();

/// **The music's own stream, as a pointer at `+0x5208` of the manager, and how much of it there is.**
///
/// `PlayBGM` (0x45cc70) is where it comes from: it copies the track's name into `+0x5108`, takes the
/// current one down through 0x45d2d0, works a buffer out of the loaded file's format at `[this+0x1f84]`,
/// makes the event at `+0x520c` and the thread at `+0x614`, and hands `this+0x5208` to 0x471c40 as an
/// out-pointer — which allocates `0x78` bytes at 0x471eb3, constructs them with 0x472aa0 and writes them
/// through it at 0x471f04.
const MUSIC_STREAM: usize = 0x5208;
const MUSIC_STREAM_BYTES: usize = 0x78;

/// **What that stream reads its bytes out of, and which track that is**: the source at `+0xc` of the
/// stream, the track's record at `+0x90` of the source, and in the record the track's name, where its
/// sound starts, where it loops back to and how long it is — `+0x0`, `+0x10`, `+0x18` and `+0x1c`.
///
/// Read off the play of a named track and the seek it makes. 0x45ce00 looks the name up with 0x45c6b0,
/// which cuts it past its last `/` or `\` (0x45c6c6, 0x45c6dd) and compares it against the first bytes of
/// each `0x34`-byte record at `[this+0x1f84]` until a record whose first byte is nothing (0x45c7c1 through
/// 0x45c853); it hands the record found to the source's 0x473300, which writes it into `+0x90` (0x473330)
/// and seeks with 0x4733e0 — the file at `+0x8c` moved to `[0x18bdc7c] + [record+0x18] + [record+0x10]`
/// and `+0x8` set to `[record+0x1c] - [record+0x18]` (0x47349c through 0x4734df). Which is th07's
/// `STREAM_SOURCE`, `SOURCE_RECORD` and `RECORD_*` at th07's own numbers, the two games sharing the engine.
const STREAM_SOURCE: usize = 0x0c;
const SOURCE_RECORD: usize = 0x90;
/// And how much of it there is: 0x471ca0 allocates `0x94` bytes for it and 0x4731f0 constructs them.
const MUSIC_SOURCE_BYTES: usize = 0x94;
/// **How much of the track is left before it loops, `+0x8` of the source**, which the seek puts back to the
/// whole length or to the length past the loop point (0x4734df, 0x47351d); and where in the buffer the stream
/// puts its next chunk and how big one is, `+0x64` and `+0x6c` of the stream, which 0x472aa0 zeroes and
/// 0x472ae0 writes — th07's `SOURCE_LEFT`, `STREAM_WRITE_AT` and `STREAM_CHUNK` at th07's own numbers.
const SOURCE_LEFT: usize = 0x08;
const STREAM_WRITE_AT: usize = 0x64;
const STREAM_CHUNK: usize = 0x6c;

/// **The stream's refill**, `__thiscall` on the stream with one argument on the stack: one chunk of the track
/// into the buffer. The music thread 0x45e0f0 waits on its event (0x45e124) and calls it on the stream at
/// `[0x18bdc70]` — the manager's [`MUSIC_STREAM`] — with `[stream+0x74]` set around the call (0x45e16b,
/// 0x45e18a), and its own prologue tests `[this+0x4]` and `[this+0xc]` as th07's does. Hooked so that a log
/// can say what is being fed, which is what says what a restore left the music as — see
/// `crate::game::Hooks::tops_up_the_music`.
const TOPS_UP_THE_MUSIC: usize = 0x00472d30;
/// `push ebp; mov ebp,esp; sub esp,0x34; push ebx` — position-independent, 7 bytes, read out of the exe at
/// that address, and th07's own refill's seven.
const TOPS_UP_THE_MUSIC_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x34, 0x53];
const RECORD_NAME: usize = 0x00;
const RECORD_NAME_BYTES: usize = RECORD_AT;
const RECORD_AT: usize = 0x10;
const RECORD_LOOP_AT: usize = 0x18;
const RECORD_BYTES: usize = 0x1c;

/// **The stage's own info block, and the three tracks it names**: `[0x4e4824]`, with a name at `+0x290`,
/// `+0x310` and `+0x390`, `0x80` bytes apart. The build puts each in the slot of its own number through
/// 0x447d56 (0x43b718 through 0x43b77a), leaving out a name whose first byte is a space, and the
/// conversation's script plays one of them by the slot its line names (0x434dbb). **The first is the
/// stage's own**: where run-kind bit 14 is set the build loads slot 0 out of the table at 0x4c7670 by the
/// card at `0x164d0b8` instead (0x43b6bc through 0x43b706), so what the first name is there is unread.
const G_STAGE_INFO: usize = 0x004e4824;
const STAGE_TRACK_NAMES: [usize; 3] = [0x290, 0x310, 0x390];
const TRACK_NAME_BYTES: usize = 0x80;
/// The run-kind bit that loads slot 0 by a card — see [`G_STAGE_INFO`].
const RUN_LOADS_A_CARDS_TRACK: u32 = 1 << 14;

/// **The queue orb's own asks would go into**, at `+0x1f88` of the manager: 31 slots of `0x10c` bytes,
/// whose first field is the kind — which is th07's own shape at th07's own stride.
///
/// 0x45e1e0 is what puts one in: it walks `i < 0x1f`, takes the first slot whose `+0x0` is nothing
/// (0x45e211) and writes the kind there (0x45e22c). [`PLAY_SOUNDS`] is what walks them, switching on
/// `[this+0x1f88] - 1` over eight cases, so a first slot holding a kind is a command not yet carried out.
const SOUND_COMMANDS: usize = 0x1f88;

/// The music thread's id and its handle, `+0x614` and `+0x618` — the offsets th07 keeps its own at.
///
/// `CreateThread` at 0x45cd63 is handed `this + 0x614` as its `lpThreadId` and 0x45e0f0 as the thread,
/// and its answer goes into `+0x618`. That thread is what tops the streaming buffer up, so it is the one
/// thread a restore of the game's own memory must not have running underneath it.
const MUSIC_THREAD_ID: usize = 0x614;

/// **The queue of quads a frame of this game's drawing fills**, as a pointer: the 0x2a2570-byte
/// allocation WinMain takes at 0x4419d0 and names `SprtCtrlInf` (0x4b5b7c).
///
/// `[queue+0x2520]` is how many quads are in it and `[queue+0x2a2524]` the cursor the next one is
/// written at, which is what the two calls below move — so the object is a header of a little over
/// 0x2520 bytes with 2.7MB of quads behind it.
const G_QUAD_QUEUE: usize = 0x018bdc90;

/// The queue emptied and the queue drawn, `__thiscall` on what [`G_QUAD_QUEUE`] points at — the two
/// calls orb's own frame has to make around the draw chain, because the game's frame makes them there.
///
/// 0x462e00 writes nothing into the count at `+0x2520` and puts the cursor at `+0x2a2524` back to
/// `this+0x2524`, which is a queue emptied. 0x462e40 answers at once where that count is nothing and
/// otherwise sets two texture stage states through the device's `+0xfc` and draws, which is a queue
/// flushed.
const EMPTY_QUEUE: usize = 0x00462e00;
const DRAW_QUEUE: usize = 0x00462e40;

/// **The `.anm` slots that same object keeps, which is where this game's textures are**: `0x100` of them
/// at `+0x24`, `0x1c` bytes apiece, and one `.anm` file loaded into each. th07's sprite manager is the
/// object its own quad queue is too, and this game's is the same object twice for the same reason.
///
/// **The count and the stride are the free's own** (0x466060, `__thiscall` on the manager with a slot):
/// it declines a slot below 0 or `0x100` or more (0x46606b, 0x466071), indexes `slot * 0x1c` and reads
/// three fields of it — `+0x28` the loaded file, which being nothing is a slot with nothing in it;
/// `+0x2c` a count; and `+0x38` an array of `0xc`-byte entries it walks that count of. The loader
/// (0x4658ca onwards) writes the same three at `this + slot * 0x1c + 0x24` plus `+0x0`, `+0xc` and
/// `+0x10`, so the slot is `+0x24` and the three offsets below are its own.
const ANM_SLOTS_AT: usize = 0x24;
const ANM_SLOT_BYTES: usize = 0x1c;
const ANM_SLOTS: usize = 0x100;
const ANM_SLOT_FILE: usize = 0x4;
const ANM_SLOT_SHEETS: usize = 0x8;
const ANM_SLOT_SHEETS_AT: usize = 0x14;

/// **One of those entries is a texture and an allocation**, `0xc` bytes: 0x466160 releases `[entry+0x0]`
/// through its vtable's `+0x8` — which is what says it is the COM object DirectX made — and hands
/// `[entry+0x4]` back to the game's own allocator. So the array is the one place a `.anm` slot's handles
/// live, and it is what a restore must leave alone.
const A_SHEET_BYTES: usize = 0xc;

/// And how many of them a hole may be made for, which is a bound and not a reading: the count is a dword
/// of the game's own that a restore may be halfway through putting back, and a length read off memory
/// and believed is a range that would hold megabytes of somebody else's back. The `.anm` files of this
/// game chain a handful of headers apiece, so anything past `0x40` is not a count.
const SHEETS_AT_MOST: u32 = 0x40;

/// **The fog put out**, `__thiscall` on [`G_SUPERVISOR`], which is the third call the frame makes
/// between `BeginScene` and the draw chain.
///
/// 0x448229 draws the queue first — a render state changed under quads already queued is a frame drawn
/// with the wrong state — and then, where `[this+0x350]` says the fog is on, clears that field and sets
/// `D3DRS_FOGENABLE` false through the device's `+0xc8`.
const FOG_OFF: usize = 0x00448229;

/// Whether the fog is on, as the supervisor keeps it: `[this+0x350]`, which is 0x17ceaa8 absolutely.
///
/// **Written absolutely as well, and that is what the frame's own `mov DWORD PTR ds:0x17ceaa8,0xff` at
/// 0x441fd3 is.** [`FOG_OFF`] reaches the device only where this field says the fog is on, so a frame
/// that wrote nothing here would tell the device once, on the frame after a stage turned fog on, and
/// never again. The frame writes it and then makes the call, so `D3DRS_FOGENABLE` is set false on every
/// frame drawn whatever the game last thought the state was.
const FOG_ON: usize = 0x0350;
/// And what the frame writes there, which is not what [`FOG_OFF`] writes: 0xff. Anything but zero would
/// do — that call compares this field against zero — and 0xff is what the frame writes.
const FOG_ON_FORCED: u32 = 0xff;

// All three together, because a frame that composed one of them and jumped into memory nothing has
// mapped for the next two is a frame no e2e test could reach at all. `handed_over!` is the parent
// module's — see it for why these are slots rather than the constants.
handed_over!(EMPTY_QUEUE_AT, empty_queue, set_empty_queue, EMPTY_QUEUE);
handed_over!(DRAW_QUEUE_AT, draw_queue, set_draw_queue, DRAW_QUEUE);
handed_over!(FOG_OFF_AT, fog_off, set_fog_off, FOG_OFF);

/// The game's own read of the keyboard, which orb answers with a word of its own.
///
/// 0x43d970 fills a 0x100-byte buffer with `GetKeyboardState` (0x43d996) and folds each key into one
/// word by testing its byte against 0x80 — so what stands in front of it is a [`Patch`] and what it owes
/// back is a `u16`. Every bit of that word is in `asm.md`; the five orb cares about are
/// [`BUTTON_SHOT`] and its four siblings.
///
/// **Two things want this one.** Keys are not read to the game while its window is behind — this read
/// being `GetKeyboardState`, it answers whatever somebody typed at whatever is in front — and a recipe
/// pressing anything at all presses it here.
const INPUT_READ: usize = 0x0043d970;
const INPUT_READ_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x81, 0xec, 0x08, 0x01, 0x00, 0x00];

/// **The one input word this game has**, and last frame's beside it.
///
/// Written at four sites and no more — the supervisor's own read (0x4455f5), that read OR'd into what is
/// already there while the run feeds itself (0x44567d), the scene transition's clear (0x445af9) and the
/// loading screen's read (0x47133d) — and read at 309. Last frame's is copied out of it before the read,
/// at 0x4455e4.
///
/// **The pause reads this one and not the run's own word**, which is what makes [`RUN_INPUT`] a mask here
/// where th07's is every bit: the run manager takes its pause off this word's edge (0x43a0e8 through
/// 0x43a10e) as every screen of the front end does, so a run's own word fed back through the read can open
/// the pause menu. The word the run acts on is [`G_INPUT_THE_RUN_ACTS_ON`], a copy of this one.
const G_INPUT_READ: usize = 0x0164d528;
const G_INPUT_LAST: usize = 0x0164d530;

/// **The word the run acts on**, and the one it acted on last: `0x164d52c` and `0x164d534`.
///
/// The player's own job reads the first for its moves (0x44aee8 through 0x44b01a) and holds the two
/// against each other for its bomb (0x44c8e1, 0x44c8f9), as a conversation does for its next line (0x4349d8,
/// 0x4349f0). **And they are the run's replay's to write**: its element, at chain priority 0x11 behind the
/// player's 9, copies the first into the second (0x452332) and [`G_INPUT_READ`] into the first (0x452340) —
/// so left to the game, the player moves on the word the element left a frame ago. See
/// [`Th08::hand_the_read_to_the_run`], which is orb closing that frame.
const G_INPUT_THE_RUN_ACTS_ON: usize = 0x0164d52c;
const G_INPUT_THE_RUN_ACTED_ON: usize = 0x0164d534;

/// The buttons the front end reads as *decide* and as *back*, which are the two masks every screen of it
/// tests against last frame's: `0x1001` at 105 sites — the shot button and return — and `0xa` at 74, the
/// bomb and the menu button. 0x4037bc is one of them, reading `(cur & 8) != (last & 8)`.
const MENU_DECIDE: u16 = 0x1001;
const MENU_CANCEL: u16 = 0x000a;

/// The five buttons orb names, out of the word [`INPUT_READ`] folds the keyboard into.
///
/// `Z` gives `0x1` at 0x43dbd7, `X` gives `0x2` at 0x43dbf9, `VK_SHIFT` `0x4` at 0x43dc1d, `VK_ESCAPE`
/// `0x8` at 0x43dc41 and `VK_CONTROL` `0x100` at 0x43dc64 — which is th07's assignment, arrived at
/// here by reading this exe's own folds and not by copying them.
const BUTTON_SHOT: u16 = 0x001;
const BUTTON_BOMB: u16 = 0x002;
const BUTTON_FOCUS: u16 = 0x004;
const BUTTON_MENU: u16 = 0x008;
const BUTTON_SKIP: u16 = 0x100;
/// And the four directions, each written by two keys — an arrow and a numpad digit — with the numpad's
/// corners writing two bits at once (0x50, 0x90, 0x60, 0xa0 at 0x43dacf through 0x43db3d).
const PAD_UP: u16 = 0x10;
const PAD_DOWN: u16 = 0x20;
const PAD_LEFT: u16 = 0x40;
const PAD_RIGHT: u16 = 0x80;

/// **What a run of this game is written down as**, which is every bit but the pause.
///
/// **Not the game's own mask, which is no mask at all**: its replay writes `[0x164d528]` down whole, two
/// bytes a frame at 0x4523af through 0x4523f9 — see *What a replay writes down per frame* in `asm.md`. th07
/// records the whole word too, and can, its run reading a word of its own; here the run and the front end
/// share [`G_INPUT_READ`], so a word handed back with `0x8` in it is a run that opens its own menu on the
/// frame it is fed — and a resume's playback feeds every word it keeps into that one read. Which is th06's
/// answer and for th06's reason.
const RUN_INPUT: u16 = !BUTTON_MENU;

/// The supervisor, whose `+0x8` is [`G_D3D_DEVICE`] and whose update decides every scene change.
///
/// WinMain writes `hInstance` into it at 0x4418d0, `Direct3DCreate8`'s answer into its `+0x4` at
/// 0x44220d and the device into its `+0x8`; its constructor at 0x445b7a zeroes `0xd9` dwords and
/// 0x441850 fills the same `0xd9` with `0xffffffff`, so the object is 0x364 bytes.
const G_SUPERVISOR: usize = 0x017ce758;

/// The scene the game is in and the one it has been asked for, which is where every scene change is
/// decided: `[this+0x158]` and `[this+0x15c]`.
///
/// **Which of the two is which is the game's own log line.** The supervisor's update at 0x445453
/// compares them at 0x44568a and, where they differ, keeps what it is leaving in `[this+0x160]` and
/// prints `scene %d -> %d` (0x4b6614) with `+0x15c` pushed first and `+0x158` second. So the first is
/// the scene now and the second the scene wanted — and the second is `0x17ce8b4` absolutely, which is
/// what the rest of the game writes a scene into at nineteen sites.
const SUPERVISOR_SCENE: usize = 0x158;
const SUPERVISOR_WANTED: usize = 0x15c;
/// **The loader the supervisor runs a scene's build on, and the two words that say it is running.**
/// 0x4486ba is `CreateThread` over a function it is handed, with the handle kept at `[this+0x284]` and
/// `[this+0x290]` set to 1 beside it; the supervisor's own update reads that word at 0x4455a4 and
/// answers 1 at 0x4455bc while it is set — which the walk takes to the next job, so what the thread has
/// put on the chain runs: see [`Th08::walks_a_frame_of_its_build`]. 0x44871b is the join:
/// `WaitForSingleObject` on the handle, `CloseHandle`, and both words back to nothing.
///
/// **Which is what a run's build is here** — see [`Th08::a_stage_has_just_been_built`]: the added
/// callback the chain runs is three calls and a `CreateThread`, and every number of a stage is put in
/// place by the thread. The frame the main thread first reads this word clear is therefore where the
/// numbers are in place, and — with orb walking the supervisor alone while it is set — where the stage has
/// not been updated, which is what [`Th08::a_stage_has_just_been_built`] answers off it.
const SUPERVISOR_LOADING: usize = 0x290;
/// **Whether a run's stage was still being built as orb read [`SUPERVISOR_LOADING`] this frame** — see
/// [`Th08::walks_a_frame_of_its_build`], which asks it, and [`Th08::a_stage_has_just_been_built`], which
/// takes the look.
static A_BUILD_WAS_GOING_ON: AtomicBool = AtomicBool::new(false);
/// **The supervisor's update**, the chain's first job — `__thiscall` on [`G_SUPERVISOR`], registered at
/// 0x445ed2 — which is what a frame of a build is walked as: see [`Th08::walks_a_frame_of_its_build`].
const SUPERVISOR_UPDATE: usize = 0x00445453;
/// **The frames a run's build has stood**, `[gm+0x3c]`: nought at the build's start (0x43abec) and a frame
/// more on every walk that reaches the run manager's update (0x439fdf) — which is what the build's last act
/// waits out: 0x43b86e through 0x43b87f sleeps `0x11` milliseconds at a time while it is under
/// [`A_BUILD_STANDS_FOR`]. A write breakpoint on it through a pick-up of stage 2 caught 0x439fe2 under the
/// walk's own call (0x43caa1) every time but the zeroing.
const BUILD_FRAMES: usize = 0x3c;
/// **How many of them the build waits for**: `0x1e` (0x43b871), and `0x50` for a recording played (0x43b85c,
/// bit 3 of [`G_RUN_KIND`] at 0x43b84f).
const A_BUILD_STANDS_FOR: i32 = 0x1e;
const A_RECORDINGS_BUILD_STANDS_FOR: i32 = 0x50;
handed_over!(
    SUPERVISOR_UPDATE_AT,
    supervisor_update,
    set_supervisor_update,
    SUPERVISOR_UPDATE
);

/// The scenes orb names, out of the switch on the scene being left at 0x4456cd and the switches on the
/// scene wanted inside it.
///
/// 1 is the front end: leaving scene 0 sets the wanted scene to it and registers it through 0x47146d,
/// and every branch that goes back to a menu comes through the same body at 0x44571c. 2 is a run,
/// registered by 0x43aa5c out of scene 1. 3 is that run re-initialised: 0x445917, 0x445953 and 0x44598f
/// each leave 3 in the wanted scene for 0x43aa5c's own build to read and then write 2 back, which is
/// what a stage transition of this game is.
const SCENE_FRONT_END: i32 = 1;
const SCENE_RUN: i32 = 2;
const SCENE_RUN_AGAIN: i32 = 3;
/// **And 6 is the screen a run ends at**, registered by `0x4582a0` with `ecx` 1 out of scenes 1, 2 and 9
/// — and a scene whose transition takes only two answers: 0x445a28 leaves for `-1`, which is the launch
/// going down, and 0x445a2e for **1**, which tears the screen down through `0x4531f0` and then falls into
/// 0x44571c, the body every path back to a menu comes through. So nothing but the front end is reachable
/// from it, which is what makes [`Th08::leave_the_screen_a_run_ends_at`] one write.
///
/// Which of that screen's own phases a run reaches is `[screen+0x8]`, out of 0x458316: **9** where bit 0
/// of the run kind is clear, and where it is set, **0x16** for a spell practice and **0x11** otherwise —
/// the Extra's. `ecx` 2 gives 0x12 instead, which is the phase the score file's own save enters it in.
const SCENE_RUN_OVER: i32 = 6;

/// **That screen's own update**, a job on the chain's calc list: 0x4582a0 hangs it there at 0x45837c with
/// the object it allocated as its argument and keeps that object's address nowhere else, so the object is
/// reached the way th07's is — through the job. **Its phase is `+0x8` and its timer `+0x4`**: 0x4584bc
/// switches on the first through the table at 0x458658, and 0x458646 steps the second on every update that
/// falls through to the tail.
const RESULT_UPDATE: usize = 0x004584b0;
const RESULT_PHASE: usize = 0x08;
const RESULT_TIMER: usize = 0x04;
/// **Phase 9 is the name entry**, 0x45621e, which is where a run somebody played goes first. It types into
/// the eight bytes at `+0x462f3` — the name of the `HSCR` record it builds at `+0x462dc`, filled on its
/// first update with the last name typed (0x4563d0, out of the `LSNM` record at `+0x46450`). **Its cursor
/// is `+0x2c`** over the 0x60 characters `[0x4c7f48]` points at, sixteen to a row: a decide writes the
/// character under it at `+0x462f3 + [+0x1c]` (0x456782), a space where it is on 0x5e (0x456799), and
/// leaves the entry where it is past that (0x4567a2); **`[+0x1c]` counts what has been typed**, and at
/// eight 0x4567c9 puts the cursor on 0x5f — which is the state [`Th08::types_its_name`] leaves it in. The
/// entry left, 0x4568bf puts the screen in 0xf.
const RESULT_TYPING_A_NAME: i32 = 9;
const RESULT_NAME: usize = 0x462f3;
const RESULT_NAME_BYTES: usize = 8;
const RESULT_TYPED: usize = 0x1c;
const RESULT_CURSOR: usize = 0x2c;
const RESULT_THE_END_ITEM: i32 = 0x5f;
/// **0xa through 0xe are the game's own offer of a replay**, all five of them 0x45696f's: 0xf waits
/// ninety updates and a decide (0x45782c through 0x457880), 0x10 thirty more, and then 0x45789f enters
/// 0xa. **0xa is the question itself**, `[+0x1c]` its cursor put at nought, the first answer, on its tenth
/// update (0x456a78), and a decide there going on to 0xc (0x456bdf) or out of the group; 0xb is where 0xa
/// goes instead where 0x4481af or 0x453c40 answers anything or `[[0x160f510]+0x29]` is set (0x456a6b).
/// And the two places the group makes the file's name —
/// `"./replay/th8_%.2d.rpy"` at 0x4b7984, pushed at 0x456d90 and 0x45744e — are inside it. **2 is the way
/// out**: 0x458516 counts the timer to 0x14 and then writes the front end into the scene wanted
/// (0x458591), and the group's own ways out of it enter 2 with the timer at nothing — 0x456c4d, 0x456d17
/// and 0x457486 — which is what [`Th08::skip_replay_prompt`] writes too.
const RESULT_OFFERING_A_REPLAY: std::ops::RangeInclusive<i32> = 0xa..=0xe;
const RESULT_LEAVING: i32 = 2;

/// **The game's own pause, which is a byte and a menu over a run that stands still**: `[gm+0x3dbb2]`, `0x164d0ba`.
/// The run manager's update writes 1 there on an edge of the menu button (0x43a0e8 through 0x43a119) — not in
/// the demo, nor where it or `[gm+0x3dbb3]` is set already — and answers **3** while it is 1 or 2 (0x43a68e
/// through 0x43a6ba), which stops the chain's walk at priority 2, before every job of the stage: so the stage's
/// clock and its script stand while it is up. What clears it is the menu itself, 0x4037b0 on `0x4cce20 +
/// 0x82a0`, called from the job at priority 1 (0x402200, registered at 0x4027b6) that runs before the walk
/// stops: a menu press with the menu up puts it in its phase 4 (0x4037ff), and 0x14 updates later 0x4041e4
/// writes the byte back to nothing. A stage's build clears it too (0x43b1cf), and the front end (0x4714d0).
const G_PAUSED: usize = 0x0164d0ba;

/// **The stage words whose own boss fights in two halves**, 3 and 4 — the uncanny and the powerful fourth
/// stage — with a stretch of the stage's waves between them. Session 13's no-bomb clear read it off 4B: the
/// boss up through chapters 10 to 13 with the script clock standing at `4963`, then `boss=false` and the clock
/// running again, what was then the table's boundary at `5404` taken at script `5407`, and the boss back at
/// `6923` for chapters 15 on. **4A is here by a pass over its replay and not by a run**, no clear of orb's
/// having gone its way: the boundaries that pass proposed have the same shape, `4612` before the clock stood
/// at `4963` and `5412` inside the stretch after it.
const STAGES_WHOSE_BOSS_COMES_BACK: [i32; 2] = [3, 4];

/// **And 9 is the ending**, registered by `0x4296a0` out of scenes 1 and 2, which allocates `0x2ab8`
/// bytes, hangs a calc element with the callback `0x429860` on the chain at priority 5 and a draw element
/// with `0x4298f0`, and names itself `EndingInf` at `0x4b4a0c` — with `end01c.end`, `end02c.end` and
/// `end03c.end` in the bytes after that name. Its own `+0x8` callback, 0x429980, is what the screen above
/// is reached from, scene 6 being registered out of 9.
///
/// **What orb was doing without it is spending a cleared run's records.** A run that clears goes from the
/// run's scene straight into this one, and orb read `in_ending: false` there: the frame the run ended was
/// taken for a run given up on a settled screen, so `runtime`'s `commit_records` asked for the ranking over
/// an ending that cannot build one and spent its whole allowance —
/// `score: the ranking was not built after 240 update(s); nothing written` — on the deathless Normal clear
/// this was read off. Which is the launch th07's `the_ending.rs` was written for, on the game that had no
/// scene for it.
const SCENE_ENDING: i32 = 9;

/// The game manager, which is what a run is built into — and a static of this game rather than an
/// allocation.
///
/// `0x43aa5c` registers scene 2 with it as the argument of both elements: the calc element at
/// 0x164d344, priority 2, whose update is 0x439bc7 and whose added callback is 0x43aaf4, and the draw
/// element at 0x164d364, priority 5, whose draw is 0x43aa03. `[eax+0x3ddc0]` is written at 0x43aa9e, so
/// the object is at least 0x3ddc4 bytes — which is why it is out in the twenty megabytes of `.data` that
/// the file does not hold.
const G_GAME_MANAGER: usize = 0x0160f508;

/// The difficulty a run is being played on, `+0x30` of it.
///
/// **Two spellings say so.** The build at 0x43ac25 compares it against 4 and writes `1 << it` into
/// `[gm+0x34]` where it is less and `0xf` where it is 4 or more, which is a mask over five difficulties;
/// and the scene transition at 0x4458db spells the same word absolutely as `0x160f538`, comparing it
/// against 4 to decide whether to clear the stage. The five names at 0x4c7fd4 — `Easy`, `Normal`,
/// `Hard`, `Lunatic`, `Extra` — are what put the Extra at 4.
const DIFFICULTY: usize = 0x30;

/// How many difficulties there are, which is the table at 0x4c7fd4 read at 0x45489e and 0x4549df.
const DIFFICULTIES: i32 = 5;

/// The run's own numbers, **as a pointer**: 0xe4 bytes off the allocator at 0x43ad57, kept at the game
/// manager's `+0x8` and so at 0x160f510 absolutely, which is how the rest of the game reaches them.
///
/// A pointer and not a struct, so a read of any field through it answers nothing where no run has been
/// built — the object does not exist until the build at 0x43abd7 allocated it.
///
/// **What is in it is what the panel draws and what a replay's header says**, which is most of it: see
/// [`SCORE`] and the constants after it, each carrying the row or the line it was named by. The human
/// and youkai gauge is the one thing the panel draws that is not a number and so is not named here.
const G_RUN_NUMBERS: usize = 0x0160f510;

/// What that allocation is, off the `push 0xe4` at 0x43ad52: the whole of a run's numbers, dumped as
/// itself where two entries into a stage have to be held against each other — see
/// [`Th08::the_runs_own_bytes`].
const RUN_NUMBER_BYTES: usize = 0xe4;

/// **The score, and the copy the panel's number chases.**
///
/// `+0x8` is the one a run is torn down with: 0x43bf94 compares it against `0x3b9aca00` and 0x43bfa2
/// clamps it to `0x3b9ac9ff`, which is 999,999,999. `+0x0` is what 0x4048c9 copies it into, and what the
/// panel draws at (488, 56) with `%.9d` — nine digits and then the byte `+0x29` with `%1d` one glyph
/// along, which is this game's ten-digit score split in two.
const SCORE: usize = 0x8;
const SCORE_SHOWN: usize = 0x0;
const SCORE_LAST_DIGIT: usize = 0x29;

/// **The graze**, `+0xc`: the panel draws it at (488, 152) with `%d` — the row under the power — and
/// 0x44a9a4 caps it at `0xf423f`, which is 999,999 and a count rather than a frame.
const GRAZE: usize = 0xc;

/// **The point items collected toward the next step of their value, and how many that step wants**:
/// `+0x30` and `+0x38`, which the panel draws at (488, 168) as one and then the other with a `/`
/// between them — the `0/100` of a fresh run, the 100 being what the build puts in `+0x38` at 0x405121.
const POINT_ITEMS: usize = 0x30;
const POINT_ITEMS_WANTED: usize = 0x38;

/// **The time orbs picked up and how many the stage wants**, `+0x3c` and `+0x40`: the two the panel draws
/// at (488, 184) through the game's own getters 0x421b60 and 0x421b80, one `%d` (0x4b4ce0) and then the
/// other with a `/` (0x4b4cdc) between them — the same shape as [`POINT_ITEMS`]'s row above it.
///
/// **And what the pair decides is this game's Last Spell**, which is what a run has to meet the second
/// number for: 0x41fb0b sums the orbs picked up, `[0x4ea670+0x10]` of the card that is up and the orbs
/// still in the air — 0x4417e0 walks [`ITEM_LIST`] counting the items whose [`ITEM_CLASS`] is 7, the class
/// a cancelled bullet leaves — and answers **2** where that sum has reached `+0x40` and **0** where it has
/// not. Which is the strategy wiki's own rule from the other side:
/// `ボスを倒したとき、それまでに拾った刻符アイテムがノルマ以上あれば、敵がラストスペルを発動する`.
const TIME_ORBS: usize = 0x3c;
const TIME_ORBS_WANTED: usize = 0x40;

/// **The human–youkai gauge**, `+0x22`, a signed word — the one `resume.rs` carries by address as
/// `the_word_at_0x22`, which is now read: 0x43c0bb adds to it and clamps it between `[gm+0x3ddf8]` and
/// `[gm+0x3ddfa]`, the player's own update moves it every frame one way or the other by the byte at
/// `[player+0x3]` (0x44beb6, the sign flipped where that byte is nought), and an enemy killed moves it by
/// 200 the same way (0x42d65c).
///
/// **What it decides is whether an enemy is worth a time orb**: 0x42da25, on the enemy's death, spawns one
/// where 0x406d10 answers — the gauge at or below `[gm+0x3ddfc]` — or 0x406d70 does, at or above
/// `[gm+0x3ddfe]`, which is the strategy wiki's `逢魔が時に敵を倒す(1個のみ)`. **The one that pays under a
/// boss is on that same list and not read in the exe**: `人間逢魔が時にショットを敵に当てる`, a shot hitting an
/// enemy with the gauge at its human end, and `高速状態なら人へ、低速状態なら妖へ傾く` — unfocused leans it
/// human. So a run that means to meet a stage's norm is played unfocused, which is what
/// `Frame::shoots_a_boss_unfocused` has the rules do; which end of this word is the human one is what a
/// launch's `gauge=` says.
const GAUGE: usize = 0x22;
/// **The line the gauge is at its human end at or below**, a signed word of the game manager's that 0x406d10
/// holds `[run+0x22]` against.
const GAUGE_HUMAN_END: usize = 0x3ddfc;

/// **The misses and the bombs spent**, `+0x64` and `+0x84`, both floats.
///
/// Named by the game's own writing of them: 0x45363d reads `+0x64` through 0x453c80 and prints it with
/// `ミス回数\t%d` (0x4b70e4), and 0x45365c reads `+0x84` through 0x453ca0 and prints it with
/// `ボム回数\t%d` (0x4b70d4) — the two counts this game puts in the text header of a replay, beside the
/// last stage and the slowdown rate.
const DEATHS: usize = 0x64;
const BOMBS_SPENT: usize = 0x84;

/// **The lives and the bombs in hand**, `+0x74` and `+0x80`, both floats.
///
/// **Two readings agree about the lives.** The build writes the run header's starting lives into `+0x74`
/// at 0x43ae42, converting the byte with `fild`; and the panel draws a row of stars at (488, 88) — the
/// Player row — from what 0x42f2b0 answers, which is `[run+0x74]`. The bombs are the row under it, drawn
/// from 0x4398ce, which is `[run+0x80]`.
///
/// **`State::lives` and `State::bombs` are `i8`, so what orb reads is the whole part** — taken the way
/// the game takes it, since everything orb does with either works in whole stars.
const LIVES: usize = 0x74;
const BOMBS: usize = 0x80;

/// **The power the shot is at**, `+0x98`, which this game keeps as a float.
///
/// Read through the game's own getter: 0x422480 is `float_to_int([[gm+0x8]+0x98])`, and the panel at
/// (488, 136) draws what it answers with `%d` under 128 and the word `MAX` (0x4b4cd8) at 128 or more —
/// so 128 is this game's full power, off that same `cmp eax,0x80` at 0x437215.
const POWER: usize = 0x98;

/// **The five more of the run's numbers the replay's own record carries**, which is what a resume of
/// this game is shaped from — see `resume`, and `asm.md`'s *The replay* for the address each was read
/// at. Four of the five have no name of the game's and are written here by where they are.
///
/// `+0x24` is what one point item is worth, which the item's own collect at 0x440e50 reads; `+0x34` is
/// the step of the night's own table of point-item counts, which 0x440495 indexes `0x4c7a84` by; and
/// `+0x22`, `+0x1c` and `+0x28` are a word and two bytes the record keeps at `+0x18`, `+0x21` and
/// `+0x22` and the restore puts back at 0x452fd4, 0x452fa6 and 0x452fe5. The last of those is the one
/// 0x439df7 compares against `0xc` to choose which of the run's two endings it is.
const POINT_WORTH: usize = 0x24;
const POINTS_STEP: usize = 0x34;
const RUN_WORD_AT_0X22: usize = 0x22;
const RUN_BYTE_AT_0X1C: usize = 0x1c;
const RUN_BYTE_AT_0X28: usize = 0x28;

/// **And the two bytes of the record's set that are not in the run at all**, which the restore writes
/// straight into `.data`: 0x452ef1 puts the record's `+0x1f` into the first and 0x452f59 its `+0x20`
/// into the second. Nothing is read of what either means; the second sits immediately below
/// [`G_CHARACTER`].
const G_BYTE_AT_164D334: usize = 0x0164d334;
const G_BYTE_AT_164D0B0: usize = 0x0164d0b0;

/// **The seal over the run's numbers**, which this game checks every two hundred frames of its enemy manager
/// and stops the chain behind where it has been broken.
///
/// 0x40bb80 is the check, asked by the enemy manager's update where its own timer (`[enemies+0x9dced0]`) is a
/// multiple of 200 (0x42de49): `[run+0xc8]` against `[run+0xd8] * 0x274 + [run+0x50]`, and `[run+0xcc] +
/// [run+0xb4]` against the float at [`G_SUM_THIS_FRAME`]. Where either disagrees the update answers 4 at
/// 0x42de71 — which leaves the chain's walk on the spot — and jumps past the step of that timer (0x42de78), so
/// the next frame is the same multiple and the same answer, for good.
///
/// **What keeps them agreeing is the run manager and the reseal.** The manager's update, every frame
/// (0x43a4aa through 0x43a4d7), writes `+0x50` over [`RUN_SEAL`] and then puts [`RUN_CHECKSUM`]'s sum plus
/// `+0xb4` into [`G_SUM_THIS_FRAME`]; the sum adds `+0xd8` to [`RUN_SEAL`] for each of its `0x274` bytes, which
/// is the first half of the check. The reseal (0x406e50) draws two keys from the generator into `+0x50` and
/// `+0xb4`, copies the first over [`RUN_SEAL`] and keeps the sum in [`RUN_SEALED_SUM`] — so the second half asks
/// whether the bytes the sum is taken over are the ones they were when the run was last sealed. The power's and
/// the lives' setters (0x406fa0, 0x406e20) call it and the bombs' (0x406f30) does the same inline.
///
/// **Measured, because this is why a run picked up in stage 2 never moved.** Such a run's playback stood at
/// `192,384` through 7,318 frames, its bullets and its card's clock with it; a debugger read the enemy manager
/// answering 4 from the stage's first frames on and the check's own numbers as `xcc=16295 xb4=7534` against a
/// float of `24039`: `210` apart, which is the four bytes of `72.0` — `00 00 90 42`, the stage's own power,
/// written straight over the `0.0` its build had sealed.
const RUN_SEAL_KEY: usize = 0x50;
const RUN_SEAL: usize = 0xc8;
const RUN_SEALED_SUM: usize = 0xcc;
/// The sum itself, `__thiscall` on the game manager: every byte of the run's `0x80` from `+0x48`, its `0x14`
/// from `+0xd0`, `0x3c` of the block `[gm+0x4]` names, `0x3c` of the configuration at `0x17ce870` and `0x168`
/// of the game manager's own from `+0x3da2c` (0x439a48 through 0x439aa8, each through 0x439ac5).
const RUN_CHECKSUM: usize = 0x00439a2e;
/// Where the run manager puts the sum it took this frame, as a float — see [`RUN_SEAL_KEY`].
#[cfg(any(test, feature = "sim"))]
const G_SUM_THIS_FRAME: usize = G_GAME_MANAGER + 0x3ddf4;

handed_over!(
    RUN_CHECKSUM_AT,
    run_checksum,
    set_run_checksum,
    RUN_CHECKSUM
);

/// **The player**, which is a static of this game and not an allocation: 0x44c230 zeroes `0x38acc`
/// dwords of it — `0xe2b30` bytes — and registers its calc element at `[player+0xe2b10]` with 0x44c390
/// as the update, 0x44d650 as the added callback and 0x44dc60 as the deleted one, at priority 9.
///
/// The name comes off the game's own log line: 0x43adf1 calls 0x44c230 with `ecx` nothing and, where it
/// answers, prints `error : プレイヤーの初期化に失敗しました` (0x4b5930) — so that function is the
/// player's build and `0x17d5ef8` is what it builds.
const G_PLAYER: usize = 0x017d5ef8;

/// **Where the player is**, three floats at `[player+0x2b4]`: x, y and a z.
///
/// Read off the added callback, which is where a stage puts the player at its start: 0x44d745 loads the
/// play area's width out of `0x164d2e4` — `[gm+0x3dddc]`, which is [`FIELD_WIDTH`] — halves it by the 2
/// at `0x4b42ec` and writes it into the first (0x44d765); 0x44d767 takes the height out of `0x164d2e8`,
/// which is [`FIELD_HEIGHT`], subtracts the 64 at `0x4b42c8`
/// and writes that into the second (0x44d787); and 0x44d798 writes `0x3efae148` — 0.49 — into the
/// third. The accessor between them, 0x40b460, answers its own `this`.
///
/// **So a position of this game is measured from the play area's own corner and not the screen's**:
/// the start is (192, 384) where the area is 384 by 448, which is the middle across and 64 up from the
/// bottom.
const PLAYER_POSITION: usize = 0x2b4;

/// **The active team member's own numbers**, as a pointer into a `.sht` loaded out of `th08.dat`.
///
/// The player's teardown clears it at 0x44dcd0 and the one beside it at 0x44dcf4 — a team being two —
/// and the player's own code reads `+0x4`, `+0x8`, `+0xc` and `+0x10` of it. `0x4c7ce0` is the table of
/// file names it comes from, indexed by [`G_CHARACTER`]: `ply00a.sht` through `ply03as.sht`, the `s`
/// ones being the solo characters.
///
/// **So this game's speeds are in a data file and not in the exe**, and what named their offsets is a
/// launch: a recipe held each direction for twenty frames while orb printed the player's position every
/// frame, and differencing the frames gave 4.00 straight and 2.83 diagonal unfocused, 2.00 and 1.41
/// focused. The same launch printed the first sixteen floats of this object and four of them are exactly
/// those four numbers — see [`SPEED_STRAIGHT`] and its three neighbours.
const G_TEAM_SPEC: usize = 0x018b896c;
/// How much of it orb says, which is enough to hold everything named below: 16 floats.
const TEAM_SPEC_SAID: usize = 16;

/// **The four speeds the player moves at**, in the team member's own `.sht`.
///
/// Named by holding a launch's measurement against a launch's dump of this object, which read
/// `[0.00, 3.00, 0.00, 1.65, 2.80, 10.00, 24.00, 128.00, 0.00, 4.00, 2.00, 2.83, 1.41, 0.90, 0.00,
/// 0.00]` for the Barrier Team — so `+0x24` is the 4.00 a held direction moved the player by, `+0x28`
/// the 2.00 it moved by focused, `+0x2c` the 2.83 each axis moved by on a diagonal and `+0x30` the 1.41
/// of a focused diagonal. A diagonal is the straight speed over √2 and focused is half, so the four
/// are one number and its derivations — and the game keeps all four anyway.
const SPEED_STRAIGHT: usize = 0x24;
const SPEED_FOCUSED_STRAIGHT: usize = 0x28;
const SPEED_DIAGONAL: usize = 0x2c;
const SPEED_FOCUSED_DIAGONAL: usize = 0x30;

/// **And the power this team's shot stops adding at**, `+0x1c`, which that same dump read as 128.00.
///
/// A second spelling of the 128 [`POWER`]'s own doc gets off the panel's `cmp eax,0x80` — the two agree,
/// and this one is the game's own number rather than a constant in its code.
const SPEC_POWER_FULL: usize = 0x1c;

/// **How big the player's own hit box is**, as a half-extent: the Vector3 at `[player+0x3d4]`.
///
/// **Read off the two writes that make the box out of it, and out of the test that reads that box.**
/// Per frame, 0x44bc02 calls the vector subtract on `player+0x2b4` with `player+0x3d4` and copies the
/// answer into `+0x38c`, and 0x44bc37 calls the add with the same two and copies that into `+0x398`. The
/// hit test at 0x44a2c7 through 0x44a306 then compares a hazard's own box against those four floats —
/// `[+0x38c] <= right`, `[+0x390] <= bottom`, `[+0x398] >= left`, `[+0x39c] >= top`, an overlap and
/// nothing else. So `+0x38c` is the box's corner nearest the origin, `+0x398` the far one, and
/// `+0x3d4` is what the position is moved by to get each: the half.
///
/// **And the value itself is in a file and not in the exe.** 0x44d7e3 writes `[spec+0xc] / 2` into
/// `[player+0x3d8]` and 0x44d7f5 copies that into `+0x3d4`, `+0x3dc` taking a flat 5.0 — where `spec` is
/// `[0x18b896c]`, a `.sht` loaded out of `th08.dat` (`ply00a.sht` and its seven siblings, the table at
/// `0x4c7ce0`). So orb reads the half at runtime and there is no number to write down here.
const PLAYER_HALF: usize = 0x3d4;
/// And the box those two make of it, which is what the game's own tests read: the near corner and the
/// far one, three floats apiece.
const PLAYER_BOX_NEAR: usize = 0x38c;
const PLAYER_BOX_FAR: usize = 0x398;

/// **What the player is doing**, the byte at `[player+0x0]`, which its own update switches on: 0x44c47f
/// compares it against 2 and 0x44c49a against 1, and each of those runs a step of its own — 0x44cbf0
/// and 0x44d180.
///
/// **All four values are read now.** The hit handler at 0x44ab84 writes **2** and resets the count at
/// `player+0xe2af4`; at thirty 0x44cbf0 takes the life and writes **1** with the player back at the start
/// (0x44d012 through 0x44d100), which is also what every stage's build leaves (0x44d886, with 120 under it);
/// and 0x44d180, at thirty of state 1, writes **3** with `0xf0` under it (0x44d24a, 0x44d252, 0x44d29f), which
/// counts down to 0 again (0x44d35c, 0x44d39f). The three hit tests decline while the byte is not nothing —
/// 0x44a333, 0x44a445 and 0x44a8f2 each answer 1 without a death — and 0x44a51f declines for 2 and 1 by name.
/// So **0** is a player that can be hit, **2** a player that has been and has thirty frames before the life
/// goes, **1** a player coming in at the start, which moves nothing (0x44c4b5 through 0x44c4cb), and **3** a
/// player untouchable and blinking (0x44d3cf) — see `asm.md`'s *The player* for the whole timeline.
const PLAYER_STATE: usize = 0x0;
/// And the value orb takes for a player that can be hit.
///
/// **Named by a launch as well as by the switch**: over one run of stage 1 the byte read 0 on 713 of the
/// frames orb printed, 3 on 528, 1 on 152 and 2 on 62, and the 0 is the one it held while the player was
/// alive and moving.
const PLAYER_NORMAL: u8 = 0;
/// **The window a deathbomb is spent in**: the value 0x44ab84 writes as the hit lands, which 0x44d180
/// carries for thirty frames — `0x1e` of the countdown at `player+0xe2af4`, compared at 0x44d24a —
/// before the life goes.
const PLAYER_HIT: u8 = 2;

/// **The countdown that window is counted on**, `[player+0xe2af4]` — and the one thing about it that has
/// to be written along with the state, because a state of 2 whose countdown keeps climbing is a life
/// about to go rather than a player that cannot be hit.
///
/// Three dwords, read off the setter at 0x406610 that the hit handler reaches through 0x4065f0 with a 0:
/// `+0x8` takes the count (which 0x40d3b0 answers and 0x44d245 compares), `+0x4` takes nothing, and
/// `+0x0` takes `-0x3e7`.
const PLAYER_HIT_COUNTDOWN: usize = 0xe2af4;
const A_COUNTDOWN_PHASE: usize = 0x0;
const A_COUNTDOWN_PART: usize = 0x4;
const A_COUNTDOWN_COUNT: usize = 0x8;
const A_COUNTDOWN_RESTARTS_AT: i32 = -0x3e7;

/// **The byte a card's own update stops on**, `[gm+0x2c]`.
///
/// 0x418010 is the chain job a card is registered with and the call to 0x416b90 is the whole of what it
/// does, whose first act is two gates: bit 10 of the word at 0x164d0b4, and this byte. Where either
/// holds, the update answers **1** and the card does not advance at all. 0x425070 writes it, and **the
/// only thing that clears it is the hit handler** — 0x44ab57, two instructions into the 0x44ab40 that
/// takes the life.
///
/// **Which is why [`Th08::make_invulnerable`] clears it too.** A launch under `--clear` never lets that
/// handler run, so a card that began while this was set stayed there: the last stage's Last Spell held
/// `script=4264`, `boss_life=20000` and `bullets=0` from the run's frame 66,138 to 108,378 — eleven
/// minutes of game time with nothing changing at all.
const CARD_UPDATE_HELD: usize = 0x2c;

/// **The enemies**, as one static object of about ten megabytes: `0x577f20`.
///
/// 0x42c5e3 registers it — the calc element at `0xf54e30`, update 0x42c660, added 0x42ebf0, deleted
/// 0x42ee80, priority 0xb — and hangs the object off `[elem+0x1c]` at `0xf54e4c`.
///
/// **The array, off the walk at 0x42c854: slot `i` is at `enemies + 0x53d0 * (i + 1)` and there are
/// `0x1e0` of them.** The cursor starts at `this + 0x53d0` (0x42c83b), steps by `0x53d0` each slot
/// (0x42c869) and the bound is `0x1e0` (0x42c872) — so the array runs to `+0x9d79d0`, which is why the
/// count below sits at `+0x9dcdc4` and the sixteen-byte side table the walk also keeps at `+0x9dcdd0`
/// sits after it.
///
/// `[enemy+0x3324] & 1` is what the walk reads first: an enemy whose bit 0 is clear is a slot with
/// nothing in it (0x42c882), and `+0x2d88` and `+0x2d8c` are where it is — the two 0x424730 reads of the
/// same shape. Nothing else of an enemy is read, its life least of all.
const G_ENEMY_MANAGER: usize = 0x00577f20;

/// **How many enemies are live**, `[enemies+0x9dcdc4]`, counted fresh every frame: 0x42c84a puts nothing
/// there before the walk.
const ENEMY_COUNT: usize = 0x9dcdc4;

/// The array itself: slot `i` at `enemies + 0x53d0 * (i + 1)`, `0x1e0` of them, alive on
/// `[enemy+0x3324] & 1`. See [`G_ENEMY_MANAGER`], where the walk those three come off is written out.
const ENEMIES_AT: usize = 0x53d0;
const ENEMY_BYTES: usize = 0x53d0;
const ENEMY_SLOTS: usize = 0x1e0;
const ENEMY_ALIVE: usize = 0x3324;

/// **Where an enemy is and how big it is**: `+0x2d88` and `+0x2d70`.
///
/// Read off the hand-over rather than off a draw: 0x42cf6d pushes `this + 0x2d70` and 0x42cf71 pushes
/// `this + 0x2d88` into 0x42c290, whose own body is the pair `[ebp+0x8]` and `[ebp+0xc]` — and 0x42c309
/// hands the same two to the player's test at 0x44a470. 0x424730 reads `+0x2d88` and `+0x2d8c` of the
/// same shape, so the position is at least two floats wide.
///
/// **The size is a whole width and height and not a half**: the test at 0x44a274 through 0x44a2bb builds
/// its box as `position ± size / 2`, dividing by the 2 at `0x4b42ec` four times.
const ENEMY_POSITION: usize = 0x2d88;
const ENEMY_SIZE: usize = 0x2d70;

/// **What that size is divided by before the body becomes a hit**, which is what makes an enemy's own
/// half a third of it rather than a half.
///
/// 0x42c290 tests an enemy against the player twice and the deadly one is the second: 0x42c333 divides
/// the size by the `1.5` at `0x4b4333` through the vector divide at 0x40c7d0, and 0x42c362 hands the
/// answer to 0x44a360, which builds `position ± size / 2` and — on an overlap with a player whose own
/// state is nothing — calls the game manager's 0x44e160 and the player's 0x44ab40, the two that take the
/// life. So the half is `size / 1.5 / 2`.
///
/// **The first test is not it.** 0x42c2a1 divides the size by the `0.7` at `0x4b4c84` and 0x42c309 hands
/// that to 0x44a470, which is the same box test grown by the `20.0` at `0x4b6e98` and gated on the
/// enemy's own `[+0x3324] & 0x80` and on its timer being a multiple of six (0x42c2e8). A box that wide
/// tested one frame in six is not what kills a run.
const ENEMY_BOX_OF_ITS_SIZE: f32 = 1.5;

/// **Which bit of `[enemy+0x3324]` says the enemy has a body to run into at all**: bit 2, read at
/// 0x42cf56, and the whole of 0x42c290 is skipped where it is clear.
///
/// **Which matters because an enemy with it clear is not a hazard.** A stage's own script keeps enemies
/// about that nothing may touch — the one a shot is aimed off, the one a card is fired from — and a bot
/// told every live enemy is solid is one that will not fly where it has to.
const ENEMY_SOLID: u32 = 0x4;

/// **And the bit in front of that one, which passes an enemy's own boxes over entirely**: bit 31 of the
/// same word. 0x42cf30 through 0x42cf47 reads it and, where it is set *and* [`G_BOXES_SKIPPED_WHILE`] is
/// not nothing, jumps past the body test and the trail's walk both — so an enemy with that bit is one
/// the game itself never tests the player against while that static stands.
///
/// **Which is what left a run with nowhere open on stage 6's `蘇活「生命遊戯」`.** That card stands a
/// hundred and more enemies up, and one of them reached the rules as a solid body `2408.0,2502.2` across
/// — 4,816 by 5,004 on a field 384 by 448 — so all nine moves read `hit first on 0` with `-1448.7 of
/// room` and the chapter took nine deaths at the floor whatever place the recipe told. The 128 of that
/// card's own grid a debugger caught mid-card were `size=24.0` with [`ENEMY_SOLID`] clear, so the wide
/// one is not the grid — and a picture of that card out of the trace shows why it never looked like
/// anything: the grid is drawn as small squares across the field and the wide one is drawn as nothing at
/// all, which is what an enemy the game's own walk passes over looks like.
const ENEMY_BOX_SKIPPED: u32 = 0x8000_0000;

/// **And the three bits in front of all of it, which take an enemy out of every test the walk makes**: 4, 5
/// and 11, read at 0x42cef1, 0x42cf08 and 0x42cf1f, each jumping to 0x42d4df — past the body, the boxes and
/// the shots alike, with nothing of [`G_BOXES_SKIPPED_WHILE`] asked.
///
/// **Which a stage's own script keeps alive through whole stretches of a stage.** Stages 1 to 4b of a
/// replay spent hundreds of frames before their midboss with exactly one enemy alive — 1,540 of them in
/// 4b — and a debugger reading the array twice a thousand frames apart in stage 1 found the same one both
/// times: slot 0, `0x1000005d`, life 20. Its [`ENEMY_SOLID`] is set, so a bot handed it by that bit alone
/// was handed a body the game never tests.
const ENEMY_UNTESTED: u32 = (1 << 4) | (1 << 5) | (1 << 11);

/// **Whether an enemy is a familiar**, which is `[enemy+0x2da4]` being anything at all: 0x41fd20 answers that and
/// nothing else, and the body test at 0x42c290 asks it.
///
/// **And the runs a familiar's body never takes a life of**: 0x42c30e reads [`G_CHARACTER`] and, where it is
/// `0` or `4`, asks 0x41fd20 of the enemy and jumps past the deadly box test (0x42c331) for a familiar — so for
/// those two indices a familiar is touched with nothing taken, whichever of the pair is out and whether the
/// player is focused or not. Index 0 is the first team, the one whose `.sht` is `ply00a`, which is the team
/// this game's cases play.
pub(crate) const ENEMY_FAMILIAR: usize = 0x2da4;
pub(crate) const FAMILIARS_PASSED_THROUGH_BY: [u8; 2] = [0, 4];

/// **And the bit an enemy is shot by**: 0x42d07d hands it to the shot manager's own test at 0x451670 only
/// where bit 6 is set — so an enemy with it clear, or with any of [`ENEMY_UNTESTED`], is not a thing the
/// player can shoot.
const ENEMY_TAKES_THE_SHOT: u32 = 1 << 6;

/// **The static that gate stands on**, which is `[player+0xfdc]` by where it is: the game skips an
/// enemy's boxes only while this is not nothing. Read absolutely, the way the game's own test reads it.
///
/// **And what it is, which is a bomb going off**: the player's own update writes `1` there at 0x44cad7, straight
/// after 0x44e2e0 has counted a bomb spent, and `0` at 0x44c70b once the timer at `+0xff4` has run the length at
/// `+0xfe4` out — so it says nothing of whether the player is focused.
const G_BOXES_SKIPPED_WHILE: usize = 0x017d_6ed4;

/// **How many frames the enemy has been alive**, `[enemy+0x2e1c]`, which is the count inside the timer the
/// game keeps on it at `+0x2e14`.
///
/// 0x42c2d6 hands `enemy + 0x2e14` to 0x40d3d0, whose body is `[this+0x8] != [this+0x0]`, and 0x42c2ed hands
/// the same to 0x40d410, whose body is `[this+0x8] % n` — the `% 6` the wide first box test is gated on.
/// 0x40d3b0 is the same field's getter, `return [this+0x8]`, so the timer's own count is at `+0x8` of it and
/// the enemy's at `+0x2e1c`. It is an integer where the game reads it: 0x41f635 reads it as one and 0x42035a
/// `fild`s it.
const ENEMY_FRAMES: usize = 0x2e1c;

/// **An enemy's own script context, `[enemy+0x2ca0]`, and the behaviour it runs every frame,
/// `[context+0x10]`** — which is the third way this game kills a player, beside a bullet's box and an
/// enemy's body.
///
/// 0x44ab40 takes the life, and three places call it: 0x44a34e inside the bullet box test 0x44a230,
/// 0x44a460 inside the enemy box test 0x44a360, and **0x44a91b inside 0x44a6a0**, the rotated-box test
/// the laser walk hands its lasers to (0x431ea1, 0x431f3b, 0x43208a) — and which three functions that
/// are no laser's hand an **enemy** to, six times between them: 0x4247ec and 0x42480b in 0x424730, the
/// same pair in 0x424820 and 0x424910. Each is `__fastcall` on the enemy, and each tests a box `590`
/// long (`0x44138000`) laid from `enemy position - [context+0x38..0x3c]` along `[enemy+0x14]`, every
/// frame at one width — `160`, `240`, `288` (`0x43200000`, `0x43700000`, `0x43900000`) — and every
/// twelfth (0x40ebc0 on the timer at `+0x2e14` with `0xc`) at a narrower one, `128`, `192`, `224`.
///
/// **How an enemy comes to run one**: the script interpreter at 0x4184b0 has an instruction whose
/// handler at 0x41d569 reads the table at `0x4c6cb0` by the instruction's own argument and writes the
/// entry into `[[enemy+0x2ca0]+0x10]` (0x41d57f) — the function the enemy then runs each frame; 0x424730
/// is entry 9 (`0x4c6cd4`). So an enemy whose context holds one of the three is firing that beam now,
/// and one whose context holds anything else is not.
///
/// **Which is what killed a run at stage 4's boss, twenty-eight times at one place.** `103.3,368.9` on a
/// move judged `236.7 of room` — nothing near: `22/22` bullets agreeing and all of them 280 units off, no
/// laser, and the nearest enemy a bodiless one at `130.8,371.5`, 27.7 away, sitting directly under the
/// boss at `130.8,76.5`. A beam 590 long from the boss downward, 160 wide, is what the player 27 units
/// from that column is inside.
const ENEMY_SCRIPT: usize = 0x2ca0;
const SCRIPT_BEHAVIOUR: usize = 0x10;
const SCRIPT_BEAM_ANCHOR: usize = 0x38;
/// `[enemy+0x14]`, the angle the beam is laid along — the one `[ecx+0x14]` 0x4247d7 hands the test.
const ENEMY_ANGLE: usize = 0x14;
/// The three behaviours that are a beam, each with the width it tests every frame.
const BEAM_BEHAVIOURS: [(usize, f32); 3] = [
    (0x0042_4730, 160.0),
    (0x0042_4820, 240.0),
    (0x0042_4910, 288.0),
];
const BEAM_LENGTH: f32 = 590.0;

/// **What says a beam is there before it is firing**, which is the whole of what the rules get a warning
/// out of: [`SCRIPT_BEAM_ANCHOR`] holds `BEAM_LENGTH / 2` along the enemy's own angle, so the enemy sits
/// at the middle of the box its beam will be, and the game writes that vector hundreds of frames before
/// the behaviour does anything.
///
/// **Measured over one attempt at stage 4's boss.** The enemy stood at `109.5,376.0` with `angle=2.011`
/// and `anchor=-125.7,266.9` — which is `295 * (cos, sin)` of that angle to a tenth — for 17.5 seconds
/// of a launch at `--pace=8` with `[context+0x10]` **nought**, and the behaviour became 0x424730 on the
/// very frame the player was hit at `113.3,369.0`, 8 units from that box's middle. So the behaviour field
/// says a beam is testing the player *this frame* and never says one is coming: a bot that waits for it
/// has already been hit, which is twenty-eight deaths at one place and then fifteen more.
const BEAM_ANCHOR_ALONG: f32 = BEAM_LENGTH / 2.0;

/// How far off that vector a beam's own anchor may read and still be one, in the game's own units. A
/// whole unit, the angle and the length being what they are compared through.
const BEAM_ANCHOR_SLACK: f32 = 1.0;

/// **And how wide one is taken to be before its behaviour says**, which is the narrowest of the three and
/// the one measured: which of them will run is in no field read, and stage 4's boss runs 0x424730, whose
/// width this is.
///
/// **It is the width of a warning and no longer of a lane already firing** — see
/// [`crate::game::FIRES_WHEN_NOTHING_SAYS`], which is what an armed enemy's beam is handed over as.
///
/// **The widest was tried first and cornered the run.** Four beams reach the rules at once in that
/// fight — the four bodiless enemies stacked at the box's middle, one beam apiece — and at 288 across they
/// left the rules `nowhere open to it` in the bottom right corner, dead seven times running at
/// `300.6,431.9` with the nearest of those enemies still 115 units off. So a lane read wider than it is,
/// is a field the rules cannot use, and the cost of reading one narrower is a beam of 240 or 288 whose
/// outer 40 or 64 units the rules will stand in until its behaviour says otherwise.
const BEAM_NARROWEST: f32 = 160.0;

/// **The boxes an enemy has past the one its own body is**, which 0x42c290 is called for once apiece —
/// the same turned-box test the body gets.
///
/// The walk is 0x42cfaa through 0x42d068: an index that **starts at 1 and steps by 6** (0x42cfb6),
/// bounded by the signed word at [`ENEMY_BOXES`] (0x42cfbf), with each box's position at
/// `enemy + 0x3394 + i * 0x1c` (0x42d058). Which makes the array at `+0x3394` a trail of where the enemy
/// has been — one entry a frame, `0x1c` bytes apiece, two floats of it a position — and every sixth
/// entry of it a box.
///
/// **And the size each box is tested with is the enemy's own, tapered where bit 1 of
/// [`ENEMY_BOX_FLAGS`] is set**: 0x42cfe0 through 0x42d03a divides `[enemy+0x2d70]` by the count and
/// multiplies by the index, so the far end of a long enemy is narrower than its head. Where that bit is
/// clear every box is the enemy's whole size.
///
/// **Both gates are the same ones the body's own box is behind, and one more.** 0x42cf5e skips the whole
/// of it — body and trail alike — where [`ENEMY_SOLID`] is clear, and **0x42cf8b skips the trail where
/// [`ENEMY_BOX_FLAGS`] is nothing at all**, which is every enemy that is not one of the long ones. So a
/// trail is a thing few enemies have and the byte is what says which.
///
/// Through the same 0x42c290 as the body, so the same [`ENEMY_BOX_OF_ITS_SIZE`] applies to each.
const ENEMY_BOXES_AT: usize = 0x3394;
const ENEMY_BOX_BYTES: usize = 0x1c;
/// The first of them and the step between them, which are the loop's own 1 and 6.
const ENEMY_FIRST_BOX: usize = 1;
const ENEMY_BOX_EVERY: usize = 6;
/// How many entries of that trail are filled, as a signed word, and the flags byte beside it — nothing
/// for an enemy with no trail, and bit 1 for one whose boxes taper.
const ENEMY_BOXES: usize = 0x5350;
const ENEMY_BOX_FLAGS: usize = 0x534c;
const ENEMY_BOXES_TAPER: u8 = 0x2;

/// **Which bit of `[enemy+0x3324]` says this enemy is the boss**: bit 1, which the ECL instruction at
/// 0x41c4f0 sets (0x41c51d) beside telling the run screen a boss is up, and the one at 0x41c5a0 clears
/// (0x41c5ca) beside telling it none is.
///
/// **Two other things read it and agree.** 0x42c375 stops the player's own body taking life off an enemy
/// that has it — a boss is not hurt by being flown into — and 0x41c9b5 gates the life bar's own update
/// on it. So an enemy with this bit is the one the fight is with; see [`Th08::the_boss`].
const ENEMY_IS_THE_BOSS: u32 = 0x2;

/// **The bosses, by the number the ECL gave each**: eight pointers at `[enemies+0x9dcda0]`, `0xf54cc0`.
/// The instruction that puts a boss up writes the enemy into its entry (0x41c4b9) before it sets
/// [`ENEMY_IS_THE_BOSS`], and the entry goes back to nought wherever the bit is cleared — the instruction
/// taking the boss down (0x41c5b6) and the enemy's own death (0x42bd66, 0x42bdf7). The game's own walks
/// over it stop at eight (0x40d2cd, 0x44c760).
///
/// **Read rather than every slot of the array asked for the bit**, which was 0x1e0 reads a call and two
/// calls to a state read: measured at 0.42ms a state read in the laid-out game, most of an e2e frame, and a
/// `VirtualQuery` a slot on the real one.
const BOSSES: usize = 0x9dcda0;
const BOSS_ENTRIES: usize = 8;

/// **How much life an enemy has left**, `[enemy+0x2dfc]`, as a whole number, with the maximum beside it
/// at `+0x2e00`.
///
/// Read off two places that agree: the life-bar instruction writes both at 0x41c989 and 0x41c97a out of
/// its own argument, and 0x42c391 takes `0xa` off the first where the player's own body touched the
/// enemy. `State::boss_life` is what reads it — see [`Th08::the_boss_enemy`].
const ENEMY_LIFE: usize = 0x2dfc;

/// **How long the enemy's own clock has run, which is what says a boss has moved to its next attack** —
/// `[enemy+0x2e1c]`, the count of the timer at `+0x2e14`.
///
/// `+0x2e14` is a timer of the shape every clock in this game has, `0x40d3b0` answering its `+0x8`: the
/// enemy update steps it (0x42c2d9 hands it to 0x40d3d0 and 0x42c2ed to 0x40d410) and the beam behaviours
/// ask 0x40ebc0 whether its count is a multiple of twelve, which is their narrower test's own frame. And
/// **the ECL reads that count as a variable of its own**, 0x41f635 answering `[enemy+0x2e1c]` for one of
/// the ids its variable read switches over — the same shape th07's `ENEMY_ATTACK_FRAMES` has.
///
/// **Which is what a chapter of a fight needs.** `chapter::Chapters::due` cuts a chapter where a boss's
/// attack clock goes *back*, and with `State::boss_attack_frames` answering `None` the only cuts this game
/// got were a card starting, a boss arriving, a midboss beaten and the midstage table — so **a nonspell
/// that follows a spellcard got no chapter of its own**, and a run that died in one was sent back to the
/// card. A launch spent its retries that way on stage 5's second card, 「ビジョナリチューニング」, dying
/// in the waves after it and fighting the card again each time.
const ENEMY_ATTACK_FRAMES: usize = 0x2e1c;

/// **The bullets**, as one static object of about seven megabytes: `0xf54e90`.
///
/// 0x4311a0 registers it — the calc element at `0x160f408`, update 0x431240, added 0x433070, deleted
/// 0x433820, priority 0xe — and hangs the object off `[elem+0x1c]`.
const G_BULLET_MANAGER: usize = 0x00f54e90;

/// **How many bullets are live**, `[bullets+0x6ba538]`.
///
/// Counted fresh every frame by the update rather than kept: 0x431283 puts nothing there before the walk
/// and 0x4312d0 adds one for each slot whose kind is not nothing. So it is the count as of the last
/// update and not a running total, which is what `State::bullet_count` wants.
const BULLET_COUNT: usize = 0x6ba538;

/// The array itself and how it is walked, off the loop at 0x43129e: **slot `i` is at
/// `bullets + 0x1a880 + i * 0x10b8` and there are `0x600` of them**.
///
/// The walk runs backwards, which is what makes the stride readable: 0x431254 starts the cursor at
/// `this + 0x1a880`, 0x431b64 takes `0x10b8` off it every slot, and 0x431b58 adds `0x645000` back when
/// the index wraps — and `0x600 * 0x10b8` is exactly `0x645000`. The count above sits past the array's
/// end, `0x1a880 + 0x645000` being `0x65f880`.
///
/// `[bullet+0xdb8]` is a word the walk reads first: nothing means the slot is free, and 0x4312ef takes
/// one off it and compares against 4, which is a kind of five. Nothing else of a bullet is read — see
/// the list of what is not established in `asm.md`.
const BULLETS_AT: usize = 0x1a880;
const BULLET_BYTES: usize = 0x10b8;
const BULLET_SLOTS: usize = 0x600;
const BULLET_KIND: usize = 0xdb8;

/// **The one kind of the five whose test takes a life**, which is the kind a bullet spends its life in.
///
/// `0x44a230` is the box test that reaches the life-taker at 0x44a34e, and **it has one caller**: 0x4316b1,
/// inside kind 1's own branch. **The spawn kinds call `0x449ff0` instead — 0x4317cf, 0x4318e1 and 0x4319f2,
/// one apiece — and that is not a test of the player at all**: it walks the 192 circles of the player's own
/// at `player+0xbb834` that *erase* a bullet, which is read in *The bullets* of `asm.md`. So those branches
/// ask whether the bullet is inside one of those circles and nothing else, and a 2 marks the bullet —
/// 0x4317dc writes 1 into `[bullet+0xdbe]`, and on the frame its spawn ends 0x431812 writes **5** over the
/// kind in place of 1. Kind 5's branch asks nothing at all.
///
/// So a bullet coming out cannot take a life whatever it is over, one erased while it comes out is harmless
/// for the rest of its life, and kind 1's box is the only one that reaches the life-taker. Which is what a
/// run standing in a mesh of them as it comes out lives through, and what the strategy wikis for
/// 狂符「幻視調律」 — stage 5's second card — tell a person to do.
const BULLET_HITS: u16 = 1;

/// **A byte that takes a bullet out of both of the player's tests while it is set**, `[bullet+0x10b4]` — and
/// the bullet's kind is left as it was, so kind 1 alone does not say a bullet can take a life.
///
/// 0x4315a1 reads it at the head of kind 1's own tests and 0x4315aa jumps past the graze test (0x44a470) and
/// the box test ([`BULLET_HITS`]'s 0x44a230) both where it is not nothing. The spawn at 0x42f906 writes
/// nothing into it, and the only writes of anything else are the two walks at 0x424a20 and 0x424c40: each goes
/// down all [`BULLET_SLOTS`] and, for a live bullet whose word at `+0xdb0` meets the mask the script hands it,
/// either hides the bullet — the word at `+0x1fc` its sprite is drawn by put to nothing, this byte to 1 and a
/// new step written from the script's own angle and speed (0x424ade, 0x424d0c) — or shows it again, the word
/// back to 1, this byte back to nothing and the bullet's own step put back (0x424b59, 0x424dc0). So a bullet
/// with this set is one the player sees nothing of and cannot be hit by, which is what the strategy wiki says
/// of stage 5's `散符「真実の月（インビジブルフルムーン）」`: `弾が完全に見えなくなるが当たり判定もやっぱり消えている`.
///
/// **Read by the laid-out game and by the line a hit is said with**, `sight` handing a hidden bullet over as
/// hitting all the same — see the reason beside its read of the kind.
const BULLET_UNTESTED: usize = 0x10b4;

/// **Where a bullet is and how big it is**: `+0xd44` and `+0xd34`, the pair the walk hands the player's
/// own tests.
///
/// 0x4315d9 and 0x4315e3 push `this + 0xd34` and `this + 0xd44` into 0x44a470, and 0x431699 and
/// 0x4316a3 push the same two into 0x44a230 — the position first in the call and the size second, which
/// is the order those two read them in. A whole width and height rather than a half, for the same reason
/// [`ENEMY_SIZE`] is.
const BULLET_POSITION: usize = 0xd44;
const BULLET_SIZE: usize = 0xd34;

/// **How fast a bullet is going**, `+0xd50`, two floats — and **the kind is how much of it the frame
/// takes**, which is the reading that makes the field usable.
///
/// Every one of the five branches of the walk's own switch adds this vector to the position, and each
/// divides it by a number of its own first: 0x4314ae adds it whole for kind 1, and 0x43178a, 0x43189c,
/// 0x4319ad and 0x431aae hand `+0xd50` to the vector divide at 0x40c7d0 with `2.0`, `2.5`, `3.0` and
/// `2.0` for kinds 2 to 5 — the constants pushed at 0x43177e, 0x431890, 0x4319a1 and 0x431aa2. Kind 1's
/// add is the one behind a gate: `[0x160f534]` has to be nothing, which the player's own death writes
/// (0x44ab57).
///
/// **And kinds 2 to 4 are a bullet's spawn, which ends in kind 1's path.** The switch's table at 0x432156
/// sends kind 1 to 0x431322 and kinds 2 to 4 to branches that add the divided step and then ask
/// 0x45ea00 whether the sprite at `+0x2a4` has finished its animation (0x4317f3): while it has not, the
/// branch goes to the walk's tail at 0x431aeb and the bullet stays in its kind; the frame it has, the
/// branch ends `jmp 0x431306` (0x43187b, 0x43198c, 0x431a9d) — three instructions *before* kind 1's own
/// entry, at the `mov word [ecx+0xdb8],1` that writes 1 over the kind — so that one frame adds half the
/// step in its own branch and the whole step again at 0x4314ae, and every frame after is kind 1's.
/// Which is the "half again" a launch measured: `1.10,-2.07` moved against `0.73,-1.38` claimed, and
/// fifty deaths at one hit each naming three bullets that had moved `2.99` saying `2.00` — the boss's
/// fresh bullets, on the frame their spawn ended. (`+0xdbe` set on that frame sends the bullet to kind 5
/// instead, 0x431812.)
/// Kind 5's branch at 0x431aa2 is the one that does not: after its half it tests the bullet against the
/// field through 0x45ea00 and goes to 0x432170 or to the walk's tail at 0x431aeb, so a kind-5 bullet
/// moves half a step a frame and stays 5.
///
/// So a bullet's step on its next frame is this field times [`Th08::how_much_of_its_step_a_bullet_takes`],
/// which is one for the kind a bullet spends its life in and more for the frame it is born on.
const BULLET_STEP: usize = 0xd50;

/// **And the step changing under a bullet.** `[bullet+0xdac]` is the word of behaviours the walk switches
/// on before the kind — 0x43132d skips them all where it is nothing — and its bit `0x10` is 0x4322b0:
/// while the timer at `+0xfac` has
/// run for fewer frames than `+0xfcc` asks (0x40b8e0 compares its `+0x8`, so `+0xfb4`, against that
/// count), the three floats at `+0xfc0` scaled by [`G_FRAME_STEP`] (0x409120) are **added to the step**
/// through 0x410a70 at 0x432312, and the kind's own add of the step to the position comes after. The bit
/// is cleared where the timer has run out (0x4322de).
///
/// So the step orb reads after a frame is the one the *next* frame adds, plus this once more while the
/// timer runs — which is what `sight.rs` hands over. Read looking for the half again a kind-1 bullet
/// moved, which turned out to be the kind's own frame (see [`BULLET_STEP`]); what this is, is the one
/// behaviour of the nine that writes the step, the others re-aiming it or wrapping the position.
/// **The sprite a bullet draws itself with, whose animation says when its spawn ends** —
/// `bullet + 0x2a4`, which 0x4317e6 pushes into the sprite manager's own 0x45ea00.
///
/// The frames a spawning bullet moves its divided share and the one frame it moves that share and the
/// whole step both are told apart by that animation and by nothing of the bullet: see
/// [`Th08::how_much_of_its_step_a_bullet_takes`], whose doc has what each frame moves.
///
/// **What can be asked of it without walking the animation's own script**: 0x45ea10 answers ended
/// outright where [`SPRITE_SCRIPT`] is nothing, 0x45ea2c answers not ended on bit 19 of
/// [`SPRITE_FLAGS`], and where the word at [`SPRITE_CASE`] is nothing 0x45ea69 reads the count at
/// [`SPRITE_ELAPSED`] — 0x40d3b0 being `return this->+0x8` on `sprite+0x38` — and 0x45ea70 compares it
/// against the wait at [`SPRITE_WAIT`] of the instruction, an animation whose wait is still ahead of its
/// count being one that carries on. So **a wait still ahead is a frame the spawn cannot end on**, which
/// is the frames the divided share is all a bullet takes; the rest are frames it may, and orb hands over
/// the whole multiple there.
const BULLET_SPRITE: usize = 0x2a4;

/// **How many frames the bullet has been alive**, `[bullet+0xd94]`, which is the count inside the timer at
/// `+0xd8c` — the same shape [`ENEMY_FRAMES`] is read off.
///
/// 0x4315c5 hands `bullet + 0xd8c` to 0x40d3b0, the getter whose body is `return [this+0x8]`, and compares
/// the answer against `0x10`: a bullet younger than sixteen frames is one the graze test at 0x4315d9 is
/// jumped past. 0x432198 pushes nothing and hands the same object to 0x4065f0 as a bullet is made, so the
/// count starts again from nought with every bullet the slot is given.
///
/// **Which is the only thing that tells the bullet in a slot from the next bullet the slot is given** — see
/// `Hazard::frames`. A run's own hits name what it costs: stage 5's `狂符「幻視調律」` lays a lattice of
/// bullets and cancels it over and over, so its slots hold a different bullet every few frames, and a dump
/// of one hit read `a bullet at 383.4,296.9 stepping 0.0,0.0, half of it 2.0,2.0, feared out to 385.4,2.0` —
/// a bullet that claims no step at all, feared over the whole width of a 384-wide field because the slot's
/// last bullet had stood 9.6 away.
const BULLET_FRAMES: usize = 0xd94;
/// `[sprite+0x220]`, the animation's own instruction, and nothing where it has none.
const SPRITE_SCRIPT: usize = 0x220;
/// `[instruction+0x4]`, the frames it waits on, as a signed word.
const SPRITE_WAIT: usize = 0x4;
/// `[sprite+0x40]`, the frames the instruction has been waiting.
const SPRITE_ELAPSED: usize = 0x40;
/// `[sprite+0x1f8]`, whose bit 19 is an animation that never ends of itself.
const SPRITE_FLAGS: usize = 0x1f8;
const SPRITE_NEVER_ENDS: u32 = 1 << 19;
/// `[sprite+0x1fe]`, a word whose being anything takes 0x45ea00 down a path this does not read.
const SPRITE_CASE: usize = 0x1fe;

const BULLET_BEHAVIOURS: usize = 0xdac;
const BULLET_ACCELERATES: u32 = 0x10;
const BULLET_ACCELERATION: usize = 0xfc0;
const BULLET_ACCELERATION_FRAMES_RUN: usize = 0xfb4;
const BULLET_ACCELERATION_FRAMES: usize = 0xfcc;

/// **The lasers, which this game keeps in the bullet manager and not in a manager of their own.**
///
/// The walk is the second one in the same update: 0x431b72 puts the cursor at `this + 0x660938`,
/// 0x431ba2 steps it by `0x59c` and 0x431bab bounds it at `0x100`. `0x100 * 0x59c` is `0x59c00`, and
/// `0x660938 + 0x59c00` is `0x6ba538` — which is [`BULLET_COUNT`], so the lasers fill the manager from
/// the end of the bullet array to the count and there is nothing else in there.
///
/// **That is why no laser manager was found**: the reading before this one looked for an object of its
/// own on the chain and there is none, the eleven elements the chain runs being the ones `asm.md` lists.
const LASERS_AT: usize = 0x660938;
const LASER_BYTES: usize = 0x59c;
const LASER_SLOTS: usize = 0x100;
/// `[laser+0x584]`, which the walk reads first: a slot with nothing there is skipped at 0x431bbb.
const LASER_ALIVE: usize = 0x584;

/// **Where a laser is fired from and which way it points**: `+0x548` and `+0x554`.
///
/// Read off the hit test the walk hands the player. 0x431e8d pushes `this + 0x548` and 0x431e83 pushes
/// `[this+0x554]` into 0x44a6a0, whose own body takes the player's position (`player+0x2b4`), subtracts
/// that first argument (0x44a6e3), turns the difference by the *negated* angle (0x44a708) and adds the
/// origin back (0x44a71f) — so the test is an upright box in a frame the laser's own angle has been
/// turned out of, and the laser runs along `x` from its origin.
const LASER_ORIGIN: usize = 0x548;
const LASER_ANGLE: usize = 0x554;

/// **How far along it the laser reaches**, from `+0x558` to `+0x55c`.
///
/// The box handed to the test is centred at `origin.x + near + (far - near) / 2` (0x431ca1 through
/// 0x431ccb) and `origin.y` (0x431cd1), and its own width is `far - near` (0x431c81) — which is what
/// makes these two the ends. `+0x560` is what the length is capped at: 0x431bfc compares `far - near`
/// against it and 0x431c1b pulls `near` up to `far - [+0x560]`.
const LASER_NEAR: usize = 0x558;
const LASER_FAR: usize = 0x55c;

/// **How much further the far end reaches each frame**, `+0x56c`: 0x431bcf multiplies it by
/// [`G_FRAME_STEP`] and 0x431bde adds the answer to `far`.
const LASER_GROWS: usize = 0x56c;

/// **How much of a frame this frame is worth**, which this game scales the motion it works out per frame
/// by: `0x17ce8e0`, one where nothing is slowing the world down (0x425369 writes the 1.0).
///
/// Read because a laser's growth is a rate this multiplies rather than a distance — 0x431bcf — so a
/// beam's own reach next frame is the product and not the field.
const G_FRAME_STEP: usize = 0x017ce8e0;

/// **How wide the laser is**, `+0x564`, of which the hit test uses a quarter on each side: 0x431c47
/// halves it into the box's own height and 0x44a7b6 halves that again, the test building
/// `centre ± size / 2` the way [`ENEMY_SIZE`]'s does.
///
/// `+0x568` is the width it is drawn at this frame, which the warning line grows from nothing to this
/// one over `[+0x570]` frames (0x431e18 through 0x431e41).
const LASER_WIDTH: usize = 0x564;

/// **How many frames the laser has been in the phase it is in**: `[laser+0x590]`, the whole part of the
/// timer at `+0x588` — 0x40d3b0 answers `[timer+0x8]` and 0x4065f0 puts nothing back into it at each
/// phase's start.
const LASER_TIMER: usize = 0x590;

/// **Which frame of the first phase the laser starts hitting on**, `+0x574`.
///
/// The only phase whose hit test is gated: 0x431e75 asks the timer whether it has reached this, and the
/// call at 0x431ea1 is skipped where it has not. The other two phases test the player every frame.
const LASER_LIVE_AT: usize = 0x574;

/// **Which of its three phases the laser is in**, the byte at `+0x598`, which 0x431d48 switches on: 0
/// is the warning line growing towards its width, 1 the laser firing and 2 the laser going out.
const LASER_PHASE: usize = 0x598;
/// And the phase from which it is on its way out.
const LASER_FADING: u8 = 2;

/// **The items**, whose manager is `0x1653648` and is not the one the chain runs at priority 0xd.
///
/// **Two managers of this shape is what made this hard to find.** `0x4ece60` is on the chain and holds
/// 653 slots of `0x360` bytes whose position is at `+0x2a4` as well — and it is the effects: the
/// sparkles, the score plates, the death scatter. What a player picks up is here, and the two are told
/// apart by the collect: only this one's update calls the player's own pickup test at 0x44a5a0
/// (0x440a17) and switches on a class to add to the run's numbers.
///
/// 0x4b3878 builds it and 0x440010 lays its pool out: `0x831` slots of `0x2e4` bytes from the object's
/// own start, `0x2e4 * 0x831` being exactly `0x17adac`, which is where the pool's bookkeeping begins.
const G_ITEM_MANAGER: usize = 0x01653648;

/// **The live items, as a list and not an array**: `[items+0x17b088]` is the first and `[item+0x2dc]`
/// the next, nothing being the end.
///
/// Read off the walk in the update: 0x440516 takes the head into the cursor, 0x440584 stops on nothing
/// and 0x44099c follows `+0x2dc`. The array is there to be walked too — `items + i * 0x2e4` for `0x831`
/// of them, which is what 0x4400a0 spawns into by a round-robin index at `+0x17ada4` — but a free slot
/// of a pool is not marked, so the list is the only honest walk.
const ITEM_LIST: usize = 0x17b088;
const ITEM_NEXT: usize = 0x2dc;
/// And how many are live, `[items+0x17ada8]`, counted fresh by the walk (0x440543, 0x44059d).
const ITEM_COUNT: usize = 0x17ada8;
/// How many slots the pool has, which is what bounds orb's own walk of the list: a list longer than
/// this is a list orb has read wrong, and a bounded walk is what keeps that from being a frame that
/// never ends.
const ITEM_SLOTS: usize = 0x831;

/// **Where an item is**, three floats at `+0x2a4` — `+0x2a8` being the one the fall is read off
/// (0x440978, against the field's own height) and the one the auto-collect line is compared with
/// (0x440e5e).
const ITEM_POSITION: usize = 0x2a4;

/// **What picking the item up does**, the byte at `+0x2d4`: the update switches over 0 to 8 through the
/// jump table at `0x440cc5` once the player's own test at 0x44a5a0 answers (0x440a63).
///
/// | class | handler | what it does |
/// | --- | --- | --- |
/// | 0 | 0x440cf0 | power, by the thresholds at `0x4c7a6c` |
/// | 1 | 0x440e40 | a point item, worth `[run+0x24]` above the line at `[[0x18b896c]+0x1c]` |
/// | 2 | 0x441170 | power, eight of it (0x4411bd) |
/// | 3 | 0x440a9e | a bomb, one more under a cap of eight |
/// | 4 | 0x440aeb | power to the 128 it stops at |
/// | 5 | 0x440adc → 0x439b29 | a life, one more under a cap of eight |
/// | 6 | 0x440b96 | score off the graze, `graze / 0x28 * 0xa + 0x12c` |
/// | 7 | 0x440bf9 → 0x4412b0 | what a cancelled bullet leaves, three at a time (0x43164b) |
/// | 8 | 0x441020 | a point item, which is what 0 and 2 become at full power (0x44010e) |
const ITEM_CLASS: usize = 0x2d4;

/// **Whether the item can be picked up by flying into it at all**: `[item+0x2d7]`, which 0x4409f5 reads
/// and skips the player's own test on when it is 3.
const ITEM_UNREACHABLE: u8 = 3;
const ITEM_REACH: usize = 0x2d7;

/// **The generator**, whose whole state is the word at this address.
///
/// Read off the draw itself: 0x43ecc0 takes `[this+0x0]` as a word, exclusive-ors it with `0x9630`,
/// subtracts `0x6553`, rotates the result left by two — `(w & 0xc000) >> 14 | w << 2`, written as a
/// `lea` — writes it back and answers it. 0x43ed20 calls that twice to make a dword and is what
/// every other draw goes through, `0x406ef0` taking a bounded one out of it with a `div`.
///
/// **And the object has two more fields.** `+0x2` is a kept copy the game restores the word from at
/// 0x43a1b1, and `+0x4` is a count of draws that 0x43ed0b steps on every one. Both go back with the word
/// for a chapter to draw what the run drew, which is what [`Th08::seed`] writes all four of.
const G_RANDOM: usize = 0x0164d520;

/// The word a stage started from, kept in the game manager: `[gm+0x3ddbc]`, which 0x43b3d2 copies the
/// generator into.
const SEED_AT_STAGE_START: usize = 0x3ddbc;

/// **The generator's other two fields**, which a stage put back has to have as well as its word: `+0x2`
/// is a kept copy the game restores the word from at 0x43a1b1, and `+0x4` is the count of draws 0x43ed0b
/// steps on every one. See [`Th08::seed`], which writes all four numbers a seed is.
const GENERATOR_KEPT: usize = 0x2;
const GENERATOR_DRAWS: usize = 0x4;

/// **The job the build calls right after the seed has been copied**, `0x409b20`, called from one place in
/// the exe — 0x43b3e2, three instructions past 0x43b3cb's copy. Which is the one place a resumed run's
/// seed can go in, and why it is a seam of its own: see `Hooks::stage_building`.
///
/// **Inside the build and not at its prologue**, for th06nc's reason: what a build draws from the
/// generator before the copy is a seed written in front of it made stale. Writing it here and writing
/// [`SEED_AT_STAGE_START`] with it — which [`Th08::seed`] does — is what leaves the two agreeing.
const STAGE_SEEDED: usize = 0x00409b20;
/// `push ebp; mov ebp,esp; sub esp,0xc; push edi` — seven bytes, position-independent.
const STAGE_SEEDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x0c, 0x57];

/// **The screen a stage is played on**, which is what everything about a stage's own presentation hangs
/// off: the dialogue, the boss's life bar and the panel.
///
/// **Two spellings that agree.** 0x4358bb, 0x41c4fb and 0x4230c0 are all `__thiscall` on the static
/// `0x160f428`; and 0x437b6e writes the address of the object the element at `0x160f4e4` is hung off
/// into `0x160f500`, having set that object's `+0x8` to the `0x230b8`-byte allocation the allocator call
/// at 0x437b3d names `FRScreenImplInf`. For both to be true this static is what that pointer holds, and
/// the chain runs its element at priority 0xf with update 0x4338ca.
const G_RUN_SCREEN: usize = 0x0160f428;
/// And the allocation hung off it, where the dialogue lives.
const RUN_SCREEN_INNER: usize = 0x8;

/// **Whether a boss is up**, the byte at `+0x2c`: 0x422c20 is the one writer and the two ECL
/// instructions that make an enemy a boss and take it back are what call it — 0x41c500 with 1 and
/// 0x41c5a7 with 0.
const BOSS_IS_UP: usize = 0x2c;

/// **Which line of dialogue is up**, `[inner+0x2181c]`, and what stops this game's stage clock.
///
/// Note: the constant below is an offset into [`RUN_SCREEN_INNER`]'s allocation and not into the screen.
///
/// 0x4358bb is the game's own question about it: it answers *yes* where this is `0` or more, or exactly
/// `-2`, and 0x42c6a1 is what asks — the enemy manager's update, which steps [`STAGE_FRAMES`] only
/// where the answer is no. The dialogue's own region is the `0x55c` dwords 0x4339a2 zeroes from
/// `+0x21814`, and `msg start %d` at `0x4b4cac` is what 0x43397f logs as one begins.
const DIALOGUE_LINE: usize = 0x2181c;
/// The one value below nothing that still means a line is up, off 0x4358e5.
const DIALOGUE_LINE_ENDING: i32 = -2;
/// And the value that is no line at all, which is what a playback writes to get past one — see
/// `Game::give_up_on_a_line_of_dialogue`.
const DIALOGUE_NO_LINE: i32 = -1;

/// **How many frames the run has been played**, `0x164d30c` — `[gm+0x3de04]`.
///
/// Stepped at 0x42c6ad, in the update of the manager the chain runs at priority 0xb, behind one gate:
/// 0x4358bb answering nothing on `0x160f428`, which is this game's "something has the run stopped".
///
/// **And put on 1,800 by the instruction that starts a conversation**, which is the only other write of
/// this word in the exe: instruction 148 of the stage `.ecl` adds `0x708` at 0x41d6cd, having handed the
/// line to the conversation the gate above asks about — see `asm.md`, whose table has the arm. So this
/// clock takes a number, skips 1,800 of them, and then stands still for as long as the line is up, and a
/// recording keyed by it has 1,800 keys nothing was ever noted against per conversation: a stage 1 run
/// wrote 15,743 frames of buttons with the generator on 10,344 of them. Which is what
/// `resume::Record::feed`'s "not the next frame of the run" says on this game and means nothing by —
/// the jump is the script's own.
///
/// **And it is the run's clock and not the stage's, which a launch settled and this doc had wrong.** The
/// eighth launch of the Normal run cleared stage 1 and the two lines either side of the transition read
/// `stage=0 frames=18218` and `stage=1 frames=18218` — the same number — with the run's own first frame
/// at `frames=0`. So nothing resets it at a stage change.
///
/// **What that costs is [`Th08::midstage_table`]**: a boundary inside a stage is a number on this clock,
/// and on this game the number for anything past the first stage is *the whole run so far plus the way
/// in*, which is a different number every run. So the rows of `chapters::MIDSTAGE` cannot be numbers on
/// this clock, and are on the stage script's instead — see `State::script_frames`, which starts again with
/// each stage.
///
/// **Two other fields look like this one and are not it.** `[gm+0x3daa8]` is written at 0x43a1a1, in the
/// block that also puts the play area back and plays the `Pause` sound — once a pause. `[gm+0x3de08]`,
/// the word right after this one, is stepped at 0x43a269 behind a second gate on `[0x164d0bd]` being 1,
/// and orb read `frames=0` off a whole run of stage 1 with it. Both were tried before this one, which is
/// why they are written down.
const STAGE_FRAMES: usize = 0x3de04;

/// **The clock the stage's own script is run on**, `[enemies+0x9dcdd8]`: the count of the timer of the
/// first of the sixteen contexts the enemy manager keeps for the `.ecl` it loads for the stage.
///
/// **The array, off the constructor and the walk.** 0x42a243 array-constructs `0x10` objects of `0x10`
/// bytes at `enemies+0x9dcdd0` with 0x449f50, which is the one thing between the `0x1e1` enemies below
/// it and the timer at `+0x9dced0` above it. The walk at 0x42c7d5 runs as many of them as the `.ecl`
/// header says — 0x42dfb0 answers `[[eclmgr]+0x6]` as a word — and hands each to 0x42a8a0 as
/// `enemies + i * 0x10 + 0x9dcdd0`, filling `+0xc` from 0x42dfd0 where it is still nothing.
///
/// **A context is a timer and an instruction pointer**: `+0x0` through `+0xb` are the timer
/// [`A_COUNTDOWN_COUNT`] is the count of, and `+0xc` is where in the script it stands. 0x42a8cb asks
/// 0x40d3f0 whether the count *equals* the dword the instruction carries at `[ip]`, runs the
/// instruction where it does and steps `+0xc` on by the byte at `[ip+0x6]`; 0x42ad57 steps the timer
/// once through 0x406640 as the frame's run of that context ends. So this counts the frames of the
/// script and not of the run, and it is the clock the `.ecl`'s own instruction times are written in.
///
/// **And it starts again with the stage, which a launch measured**: the two state lines either side of a
/// stage 1 cleared read `stage=0 frames=17132 script=4283` and `stage=1 frames=17132 script=0`, and
/// stage 2 counted 0, 50, 110, 170 from there while the run's clock carried on from 17,132. Which is why
/// `State::script_frames` comes off this and `State::stage_frames` off [`STAGE_FRAMES`]: one is the
/// stage's and the other the run's, and [`Th08::midstage_table`] is written in this one.
///
/// **It is reproducible where the run's clock is not**, which is what a table of numbers needs: stage
/// 1's midboss arrived at `script=2935` on three launches and its boss at `script=4176` on four, where
/// the run's clock read 18,218 frames for that same stage on one launch and 16,346 on another.
///
/// **And a fight holds it whole rather than attack by attack** — 4176 stood for ten thousand frames of
/// the boss on one launch — so a boundary inside a fight is not a number this can carry. The dialogue
/// does not stop it: it went 4223 to 4283 across the boss's own conversation while the run's clock stood
/// still, the gate at 0x42c6a1 being the run clock's and not the walk's.
const SCRIPT_CONTEXTS: usize = 0x9dcdd0;

/// The stage, counted from **zero**, which is the index two loaders take into this game's own tables:
/// 0x409925 into the `.std` table at 0x4c731c and 0x42ed3b into the `.ecl` one at 0x4c73cc.
///
/// It is also `[gm+0x3ddc4]`, which the build reads at 0x43ac01 to make the word `1 << stage` — so the
/// absolute address and the offset are the same number twice.
const G_STAGE: usize = 0x0164d2cc;

/// How many stages there are, which is what both of those tables hold: nine — the six a Normal run
/// plays, the two halves the fourth and the last are each split into, and the Extra. The tenth name at
/// 0x4c7fd0 is `Last Word`, which loads out of a table of its own and is none of the nine.
///
/// **And the six a run plays are not the first six of the nine.** 0x43c4b3 chooses the next index as a
/// stage ends: 2 goes to 3 or 4 by the character, both of those to 5, and 5 to 6 or 7 — so a run walks
/// 0, 1, 2, one of 3 and 4, 5, and one of 6 and 7. The run manager ends the run instead of choosing
/// where the stage is already 6, 7 or 8 (0x439d4d), which is why a recipe of this game waits for the
/// log's `stage=7` and not its `stage=6`. `asm.md` has both tables.
const STAGES: i32 = 9;

/// **The front end**, as a pointer: the 0x14490-byte object 0x47146d allocates for scene 1 and keeps
/// here, whose calc element (priority 4) runs 0x467399 and whose draw element (priority 3) runs
/// 0x47087f.
///
/// A static rather than the chain element's argument, which is how th07 has to find its own: this game
/// keeps the pointer where anything can read it, so orb reads it there.
///
/// **And it is written exactly once in the whole exe and never cleared** — 0x4714b9, that register's own
/// store, is the one write `objdump` finds — so once a run's build has freed the object the pointer still
/// names it. Which is why [`Th08::front_end`] is gated on the scene and not on the pointer: a launch read
/// `front end: screen 1900876895, cursor on item 0` off a freed front end on every frame of a stage, and
/// a recipe waiting on `screen: 0` could have been answered by whatever the allocator left there.
///
/// The same shape as [`G_RUN_NUMBERS`], whose freed pointer read `lives=-128` — and the one difference is
/// that this one is never zeroed, so there is nothing about the pointer itself to test.
const G_FRONT_END: usize = 0x018bde08;

/// Which screen of the front end is up, `[front+0x14428]`, and the cursor on it, `[front+0x0]`.
///
/// The screen is the switch 0x4673d6 makes: an index into the byte table at 0x4674ce, eighteen wide,
/// and through it into the jump table at 0x4674a6. Screen 0 is the title menu, whose handler is
/// 0x4674e0.
///
/// The cursor is `[front+0x0]`, with last frame's kept beside it at `+0x8`: 0x467ca4 copies the one
/// into the other at the top of the title's own input, 0x4681f3 writes 8 into it where the back button
/// was pressed — the Quit item — and the decide at 0x467cf8 dispatches on it.
const FRONT_END_SCREEN: usize = 0x14428;
const FRONT_END_CURSOR: usize = 0x0;

/// What the title menu is doing, `[front+0xc]`: 0 while it builds its sprites, 1 once they are up and
/// its items take a press, 2 and 3 on the way out.
///
/// 0x4674f2 switches on it and 0x46794f writes the 1, which is the frame the build finished on.
const FRONT_END_PHASE: usize = 0xc;
/// And how long the screen has been settled, `[front+0x1442c]`: the title's own input is gated on this
/// having reached 0xa (0x467cac), which is the fade-in it counts up through.
const FRONT_END_SETTLED: usize = 0x1442c;
/// What that counter has to reach before a press is read at all — 0xa, off that compare.
const SETTLED_FRAMES: i32 = 0xa;
/// And the phase its items are read in, off 0x46794f's write and 0x4674f2's switch.
const TITLE_PHASE_ITEMS: i32 = 1;

/// The screen the title menu is, which is the one orb asks its own question over.
const SCREEN_TITLE: i32 = 0;

/// The items of that menu, off the dispatch the decide at 0x467cf8 falls into — each writes the screen
/// it leads to into [`FRONT_END_SCREEN`], or asks for a scene:
///
/// 0 to screen 4, 1 to screen 0xc, 2 to screen 0x10 with `0x164d0b4`'s bits 0 and 0x4000 set, 3 to screen
/// 8 with bit 0 set, 4 to screen 7, 5 asks for scene 8, 6 asks for scene 5, 7 to screen 2, and 8 frees the
/// front end — the jump table at 0x468357 sending 1, 2 and 3 to 0x467e8f, 0x467f40 and 0x467dd9. So 1 is
/// the Extra, whose flow settles `cursor + 4`; 2 the spell practice, bit 14 being [`RUN_IS_A_SPELL_PRACTICE`];
/// and 3 the stage practice. **And 1 and 2 are the two the title can leave out**: 0x467977 steps the cursor
/// past 2 where 0x43c2c5 answers no and 0x46799c past 1 where 0x43c1e9 does — see `asm.md`'s *What a clear
/// writes into a `CLRD` record*. What orb needs of them is that the first four start a run and the fifth
/// is the replays.
const TITLE_ITEM_START: i32 = 0;
const TITLE_ITEM_EXTRA: i32 = 1;
const TITLE_ITEM_SPELL_PRACTICE: i32 = 2;
const TITLE_ITEM_PRACTICE: i32 = 3;
const TITLE_ITEM_REPLAY: i32 = 4;
const TITLE_ITEM_SCORE: i32 = 5;

/// **The screen the team is chosen on, which is the last of the three a whole run is decided by** — and
/// the Extra's own, which is the same handler a flow along.
///
/// The front end's screens dispatch through a byte table at `0x4674ce`, eighteen wide, into the jump
/// table at `0x4674a6`: `00 09 01 02 03 04 09 05 03 04 09 06 03 04 09 04 07 08`, so screens 4, 8 and
/// 0xc all reach the difficulty select at `0x46a071` and screens 5, 9, 0xd and 0xf all reach the team
/// select at `0x46a625`. Which is th07's shape — Start, Practice Start and Extra Start walking one set
/// of handlers — at this game's numbers.
///
/// **And 5 and 0xd are the two whose decide asks for a run.** 0x46ab98 and 0x46abc1 send 9 and 0xf to a
/// screen of their own instead (0x4694c3 with 0xb and 0x10), which is the stage a practice picks; the
/// other two fall to 0x46abe7 and write the run's own scene.
const SCREEN_TEAM: i32 = 5;
const SCREEN_TEAM_OF_THE_EXTRA: i32 = 0xd;

/// **The team select's two phases**, `[front+0xc]`: 0 while it comes up, and the phase its decide is read
/// in. 0x46a637 switches on it; phase 0 builds the screen on its first update ([`FRONT_END_SETTLED`] at
/// nought, 0x46a659) and writes the 1 on the update that counter reads 8 (0x46a92d through 0x46a939), so
/// a press is read from the update after that — **which is why a recipe presses this screen twice**: a
/// press held from its first update is a word with no edge by the ninth.
const TEAM_PHASE_CHOOSING: i32 = 1;
const TEAM_SETTLES_AT: i32 = 8;

/// The difficulty the Extra is, the last of [`DIFFICULTIES`] — `Extra` at `0x4c7fd4`'s fifth name.
const DIFFICULTY_EXTRA: i32 = 4;

/// **The difficulty the front end settled**, `0x17ce891` — a byte of the configuration rather than a
/// field of the front end.
///
/// The difficulty select writes it out of its own cursor (0x46a47f, 0x46a52e) and out of `cursor + 4` on
/// screen 0xc, which is the Extra's flow (0x46a48d, 0x46a53c); the team select's decide copies it into
/// `[gm+0x30]` at 0x46abee, which is where every read of the difficulty during a run goes.
///
/// So on the team select the difficulty is already decided and this is where it is, which is what makes
/// [`Th08::run_pointed_at`] answerable a screen before the run starts.
const CFG_DIFFICULTY: usize = 0x017ce891;

/// **Whose run it is**, which for this game is a team and which of its two is out rather than a
/// character and a shot: `0x164d0b1` and `0x164d0b2`.
///
/// Read off the demo's own start at 0x467be8 and 0x467bd6, which is the one place both are written from
/// one number: the byte at `+0x6a` of a replay's header goes into the first as it stands, and its
/// remainder by 2 into the second. So the first is the whole index the game's tables are taken by —
/// `0x164d0b1` is read at 26 sites, every one of them an index — and the second is which of the pair.
///
/// The difficulty comes from the same header's `+0x6b` into [`G_GAME_MANAGER`]'s `+0x30` at 0x467ba7,
/// which is the third thing that agrees about where the difficulty is.
const G_CHARACTER: usize = 0x0164d0b1;
const G_SHOT_TYPE: usize = 0x0164d0b2;

/// **The spell card that is up**, a static object of `0x2644` bytes that `0x414590` builds.
///
/// `SpellCard::Start` is `0x4152a0`, a `__thiscall` on it reached from 0x4212ca with the card's own
/// number in the first of six pushes: it sets bit 0 and bit 2 of the flags word at `+0x0`, clears bits
/// 3, 4, 6, 7, 9 and 10, and writes that number into `+0x8` (0x41535b). `+0x14` is the card's name and
/// `+0xfc` the bonus its capture is worth.
///
/// **And the chain agrees it is this object**: `0x43c880` registers it at priority 0xc with 0x418010 as
/// its update (0x417fee), which is `asm.md`'s own table of everything a frame runs.
///
/// **Measured on the real game**: a launch of a Normal run read `boss=true spell=3` through stage 1's
/// fight, out of this object's bit 0 and its `+0x8` — which is what says the address, the bit and the
/// offset are this exe's and not a laid-out game agreeing with itself.
const G_SPELL_CARD: usize = 0x004ea670;
/// **Whether a card is up at all**, which is bit 0 of that word: `0x4178a0` answers it and the enemy
/// walk asks it at 0x42b05d and 0x42d091, where a card being up changes what a hit does.
const CARD_IS_UP: u32 = 0x1;
/// And the card's own number, which is the index into [`CARD_RECORDS`].
const CARD_NUMBER: usize = 0x8;

/// **What the card pays in time orbs**, `+0x10`, which is the number this game's Last Spell is decided by
/// and not a number the panel's count ever holds: 0x41fb0b sums the orbs picked up, this, and the orbs
/// still in the air, and answers *Last Spell* where that sum has reached the stage's norm — see
/// [`TIME_ORBS`].
///
/// **Written at the capture, on a ramp from 100 to 1,000**, which 0x416262 through 0x416347 is:
///
/// - 700 flat where bit 27 of the enemy's flags at `[[card+0x4]+0x3324]` is set (0x4162a2);
/// - otherwise `total = [card+0x114]`'s count and `most = total - total / 7`, and **1,000** where the time
///   left — `[card+0x108]`'s count — is `most` or more (0x4162ef), which is a card taken inside the first
///   seventh of its own time;
/// - `(left - 180) * 900 / (most - 180) + 100` where the left is 180 frames or more (0x416338);
/// - and **100** below that (0x416340).
///
/// **Which is why every payout a launch has seen was the floor and none of them showed up in the count.**
/// The count climbed in steps of 21 to 25 over stage 1 and reached 2,094 of a norm of 2,500, and the
/// inference drawn from that — that the payout is all-or-nothing and the rules are seven times too slow —
/// is wrong twice: the payout is never added to the count, and it is a ramp. What the norm wants of stage
/// 1 is a payout of about **406**, which the ramp reaches with a little over a third of the card's own
/// time left.
const CARD_TIME_ORBS: usize = 0x10;
/// **How much of the card's own time is left**, and **how long it had**: two timers of the shape every
/// clock in this game has, `+0x8` of each being the count that 0x40d3b0 answers — see
/// [`ENEMY_ATTACK_FRAMES`], which is read the same way. The pair is what [`CARD_TIME_ORBS`]'s ramp is
/// worked out from, and orb reads them to say how far a fight was from the payout it needed.
const CARD_TIME_LEFT: usize = 0x108 + 0x8;
const CARD_TIME: usize = 0x114 + 0x8;
/// **The time left under which a card pays the floor of [`CARD_TIME_ORBS`]'s ramp**, 180 frames (0x416338).
const CARD_PAYS_ITS_LEAST_UNDER: i32 = 180;

/// **The record this game keeps per spell card**, 222 of them at `[gm+0x40]` and `0x22c` bytes apiece —
/// see `asm.md`'s *The score file*, where the reader that fills them and the `CATK` chunk each one is
/// are written out.
///
/// 0x43bbe1 zeroes the lot and stamps each with its own tag, length, version 3 and index before the file
/// is read over them, and 0x43bdc1 copies all 222 aside afterwards.
const CARD_RECORDS: usize = G_GAME_MANAGER + 0x40;
const CARD_RECORD_BYTES: usize = 0x22c;
const CARDS: usize = 0xde;
/// The card's name inside one, copied out of the card object's `+0x14` at 0x415800.
const CARD_RECORD_NAME: usize = 0x10;
/// **And the byte the game checks the record against**, which is why a count orb writes has to be
/// stamped: 0x4159a0 through 0x415a3b sums the name's own bytes as signed and then every dword of the
/// six arrays below, and where the low byte of that sum differs from `+0xe` it **zeroes all six**
/// (0x415a74). 0x416772 is where the game writes it back after a capture.
const CARD_RECORD_SUM: usize = 0xe;
/// **The six arrays of thirteen dwords a record holds**, twelve characters and a thirteenth that is the
/// total across them — `[gm+0x3dba9]`, which is [`G_CHARACTER`], is the index.
///
/// **They are the same three things twice**: the best bonus taken, the attempts and the captures, once
/// for an ordinary run and once for a **spell practice** — and which pair the game writes is bit 14 of
/// the run kind, which [`RUN_IS_A_SPELL_PRACTICE`] is. 0x415adb branches the attempt on it and 0x416500
/// the capture. `asm.md`'s *What a `CATK` record holds* has the address each of the twelve is written at.
///
/// orb writes the ordinary run's attempts and carries the other five by address — see
/// [`Th08::count_card_attempt`], which declines a practice for the reason written there.
const CARD_RECORD_BEST: usize = 0xf0;
const CARD_RECORD_ATTEMPTS: usize = 0x124;
const CARD_RECORD_CAPTURES: usize = 0x158;
/// And the three a spell practice is counted in, in the same order.
const CARD_RECORD_OF_A_PRACTICE: [usize; 3] = [0x18c, 0x1c0, 0x1f4];
const CARD_RECORD_SLOTS: usize = 13;
/// Which of those thirteen is the total, which both counters bump beside the character's own.
const CARD_RECORD_EVERY_CHARACTER: usize = 12;
/// And the number the game's own two counters stop climbing at, tested at 0x415aef and 0x41656c.
const A_CARDS_COUNT_STOPS_AT: u32 = 0x270f;

/// What kind of run this is, as a word of flags.
///
/// Four of its bits have been read. `& 1` at 0x43acaa and 0x43add9 forces the run header's starting
/// lives to 8, and the title's Extra and Spell Practice items are what set it (0x467dde, 0x467f57);
/// `>> 0xe & 1` at 0x42ed2c sends the ECL loader to the spell-practice table at 0x4c73f0 instead of the
/// ordinary one, and that same Spell Practice item sets it (0x467f64); `| 2` and bit 3 are both written
/// by the demo's own start at 0x467b80 and 0x467b8e.
///
/// **Bit 3 is a recording played and bit 1 is the demo's own**, which the five places that ask the
/// supervisor for a run settle between them: the replay list's decide writes bit 3 as 1 and nothing else
/// of the word (0x46ed24) before it asks (0x46ee2e), the demo writes it with bit 1 beside it, and every
/// start screen writes bit 3 as nothing first (0x46ac1f, 0x46b0be, 0x46ca8d). The front end clears bit 1
/// whenever it is built (0x470da4) and bit 3 with it after a demo (0x470b31). Bit 3 is also what the
/// run's own build branches on at 0x43ae1e, where it skips putting the starting lives in — which is what
/// a run whose numbers come out of a record rather than out of the configuration does.
const G_RUN_KIND: usize = 0x0164d0b4;

/// **And two more of its bits, which are how a run of this game ends**: 5 and 6, which the run
/// manager's own ending section at 0x439bc7 reads first — 0x439be4 shifts them down and 0x439bec jumps
/// past the whole of the section while they are nothing.
///
/// A **2** there starts the ending, whose body works out the scene to hand over and which of two paths
/// returns 3 from the update — and 3 is what stops the chain's walk. A **1** or a **3** goes straight to
/// the hand-over at 0x439f3c instead, which takes the scene out of [`SCENE_THE_RUN_WANTS`] and writes it
/// into the supervisor's own wanted scene (0x439fb6) once a decide press has come, clearing these two
/// bits as it goes (0x439f9c). So the one orb writes is the hand-over's.
const RUN_IS_ENDING: u32 = 0x60;
/// The value of those two bits that hands a run over without running the ending's own body.
const RUN_HANDS_ITSELF_OVER: u32 = 0x20;

/// **The scene the run wants next**, `[gm+0x3db9c]` — `0x164d0a4` absolutely — which is `-1` while a run
/// goes on (0x439c29 ors `-1` in) and is what 0x439fb6 gives the supervisor.
///
/// The four its own endings put there are read in `asm.md`: 6 and 9 for the two results, 7 where a
/// replay has no stage left, and 3 — `SCENE_RUN_AGAIN` — where there is a next stage.
const SCENE_THE_RUN_WANTS: usize = 0x3db9c;
/// The bit the build reads a run's numbers out of a record for, which a replay and the demo both set, and
/// the one the demo sets beside it — see [`G_RUN_KIND`], where what is settled about each is written out.
const RUN_READS_ITS_OWN_INPUT: u32 = 0x8;
const RUN_IS_THE_DEMO: u32 = 0x2;
/// And the bit the spell-practice ECL set hangs off, which the title's third item sets (0x467f64).
const RUN_IS_A_SPELL_PRACTICE: u32 = 0x4000;
/// And the bit that makes a run start with eight lives, which the title's third and fourth items set — the
/// spell practice and the stage practice (0x467f57, 0x467dde) — and its first and second clear.
const RUN_STARTS_WITH_EIGHT: u32 = 0x1;

/// The configuration this game reads and writes, `0x3c` bytes — and the byte of it that says windowed.
///
/// WinMain writes the whole struct to `th08.cfg` at 0x441d43 (`push 0x3c`), and the file beside this game
/// is 60 bytes. Once read it is spread over the supervisor, and 0x447bc0 tests the byte at `+0x13a`
/// before printing `ウィンドウモードで起動します` — so that byte is the answer, and `0x17ce892`
/// absolutely.
const CFG_WINDOWED: usize = 0x017ce892;

/// The game's own `D3DVIEWPORT8`, which is the one it has: six dwords written twice a frame.
///
/// 0x40925e through 0x40927c write `{0x20, 0x10, 0x180, 0x1c0}` into it — the play area — and 0x441f1b
/// writes `{0, 0, 0x280, 0x1e0}`, the whole output, before the calc chain. Both are set through the
/// device's `+0xa0`.
const VIEWPORT: usize = 0x017ce820;

/// The play area, off the four dwords 0x40925e writes into [`VIEWPORT`]: 32 and 16 in, 384 by 448.
const PLAY_AREA: Rect = Rect {
    left: 32.0,
    top: 16.0,
    width: 384.0,
    height: 448.0,
};

/// **The same four as the game's own floats**, in the game manager: `[gm+0x3ddd4]` through
/// `[gm+0x3dde0]` — `0x164d2dc` through `0x164d2e8` — which the pause writes at 0x43a120 out of the
/// constants 32, 16, 384 and 448 at `0x4b42cc`, `0x4b42d4`, `0x4b4528` and `0x4b4940`, in that order.
/// So the width is the third of the four and the height the fourth, which is what the two below are.
///
/// A second spelling of [`PLAY_AREA`], and what the rest of the game clips against — so it is these
/// that `sight` reads for the field a bot is steered inside, and the constant above that orb's own
/// drawing uses. The left and the top are here for the reading's sake; nothing reads them, because a
/// position of this game is measured from the area's own corner rather than the screen's — see
/// `sight`'s own doc.
///
/// **Read by the laid-out game alone now**, which lays the area out for what the rest of the game clips
/// against; what `sight` steers a bot inside is the four below.
#[cfg(any(test, feature = "sim"))]
const FIELD_WIDTH: usize = 0x3dddc;
#[cfg(any(test, feature = "sim"))]
const FIELD_HEIGHT: usize = 0x3dde0;

/// **And the four the player is held inside**, right after them: `[gm+0x3dde4]` through `[gm+0x3ddf0]`,
/// written at 0x43c6d2 through 0x43c6ff out of the constants 8, 16, 368 and 416 at `0x4b4300`,
/// `0x4b42d4`, `0x4b4cf4` and `0x4b42fc` — a left, a top, a width and a height in the area's own
/// coordinates, so the player's x runs 8 to 376 and its y 16 to 432. The player's own update reads them
/// at 0x44baf8 through 0x44bbc9: a position past `left` or `left + width` is put back to it, and the
/// same for the top and `top + height`.
///
/// **Which is the rectangle a bot has to be steered inside, and not the play area.** The rules judge a
/// move by where it lands, and a landing is clamped to the field they are handed; handed the area, they
/// read a step down from `y = 432` as landing at 436 and clear of a bullet, where the game leaves the
/// player at 432 under it. A whole Normal run lost stage 1's opening eight times that way, the last six
/// at `302.6,432.0` — the floor — on a move judged `6.1 of room`.
const PLAYER_BOUNDS_LEFT: usize = 0x3dde4;
const PLAYER_BOUNDS_TOP: usize = 0x3dde8;
const PLAYER_BOUNDS_WIDTH: usize = 0x3ddec;
const PLAYER_BOUNDS_HEIGHT: usize = 0x3ddf0;

/// What this game renders into, off the viewport its own frame sets at 0x441f1b: `0x280` by `0x1e0`.
const BACK_BUFFER: (u32, u32) = (640, 480);

/// Where the game makes its window, which is the last moment [`Th08::force_windowed`] can be obeyed.
///
/// `__thiscall` on the supervisor, called from WinMain at 0x441999 — after the configuration has been
/// read out of `th08.cfg` at 0x44196a and before the device is made at 0x4419a7. It is the function that
/// holds this game's one `RegisterClassA` (0x44229e) and both of its `CreateWindowExA`s (0x4422e7 and
/// 0x44234f).
const CREATE_GAME_WINDOW: usize = 0x00442240;
const CREATE_GAME_WINDOW_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x34, 0x56, 0x57];

/// Where the device is made, which is where the slot orb's letterbox lives in can be swapped and where a
/// reset that replaced it is caught.
///
/// Called from WinMain at 0x4419a7, straight after the window, and the first thing it does with
/// `[0x17ce75c]` — the `IDirect3D8` — is at 0x4424e5.
const INIT_D3D_DEVICE: usize = 0x004424c0;
const INIT_D3D_DEVICE_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0xb8, 0x88, 0x20, 0x00, 0x00];

/// **The front end's own added callback**, `0x471651`: the front end's register hangs it off the element at
/// 0x471507, and it is a `__thiscall` of four instructions that calls the title's own load, 0x470a6c — which
/// asks the three questions the title's items are lit from (0x43c1e9, 0x43c2c5 and 0x447d04), puts up the
/// message for an item newly lit, and then reads `CLRD`, `PSCR`, `CATK` and `FLSP` out of the score file into
/// memory. See `asm.md`'s *What a clear writes into a `CLRD` record*.
///
/// **That read is the one that has to be the game's own file whichever mode a launch is in**, and what it cost
/// to leave it following the mode is every clear this game recorded. orb starts a launch in Pointdevice mode,
/// so the title loaded the records of `pointdevice_score.dat`; the run then started in the game's own mode, and
/// this game saves its score file at the start of every run — the result screen's register called with 2,
/// phase 0x12, `read, write, read` in the log straight after the mode was answered — so it wrote the other
/// file's records over `score.dat`'s. Three Normal clears each left `0x8077` in team 0's Normal word, stage 6
/// passed and Final A cleared, a debugger reading it seconds before the save; `score.dat` decoded after the
/// next launch's start read `0x0007`. And the Extra's own item is lit from those same records.
const FRONT_END_ADDED: usize = 0x00471651;
/// `push ebp; mov ebp,esp; push ecx; mov [ebp-0x4],ecx` — position-independent, 7 bytes, read out of the exe
/// at that address; the same bytes th07's front end opens with.
const FRONT_END_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x51, 0x89, 0x4d, 0xfc];

/// The row the lives are drawn on, which orb draws its own over.
///
/// **Nothing of this game's panel has been read**, so this is the play area's own top strip: it is where
/// orb would put a row of its own, and it is not a reading of where this game puts one. Nothing uses it
/// while [`Th08::draws_lives_row`] answers `false`.
const LIVES_ROW: Rect = Rect {
    left: PLAY_AREA.left,
    top: PLAY_AREA.top,
    width: PLAY_AREA.width,
    height: 0.0,
};

impl Th08 {
    /// The run's own numbers, or nothing where there is no run.
    ///
    /// **Gated on the scene, and a launch is what says why.** [`G_RUN_NUMBERS`] is a pointer the build
    /// fills and the *next* build frees (0x43acf5) — so between one run and the next it names memory the
    /// allocator has taken back, and a read of it answers whatever is there now. Driven at the title
    /// menu without this gate, orb's own state line read `lives=-128 bombs=-128`, which is what a float
    /// of rubbish comes out as.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_numbers(&self) -> Option<usize> {
        let (scene, _) = unsafe { self.scenes() };
        if scene != SCENE_RUN && scene != SCENE_RUN_AGAIN {
            return None;
        }
        mem::read_committed::<usize>(G_RUN_NUMBERS).filter(|run| *run != 0)
    }

    /// Whether the supervisor's loader is running — see [`SUPERVISOR_LOADING`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn is_loading(&self) -> bool {
        let loading: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_LOADING) };
        loading != 0
    }

    /// **How many of its own steps a bullet of this kind moves on the frame it is born into kind 1 on** —
    /// see [`BULLET_STEP`], where the walk's switch is read: kind 1 adds the step whole; kinds 2, 3 and 4
    /// are a bullet's spawn, adding it divided by 2.0, 2.5 and 3.0 a frame while the sprite's animation
    /// runs and, on the frame it ends, **falling into kind 1's path and adding it whole as well**, with 1
    /// written over the kind; and kind 5 adds half of it and goes to the walk's tail, staying 5.
    ///
    /// **This is the ending frame's multiple, and `sight.rs` takes the whole step back off it on a frame
    /// the spawn cannot end** — see [`BULLET_SPRITE`], which is what says whether it can. Both halves of
    /// that were measured the hard way: handed over as the divided share alone, a run read three bullets a
    /// frame `moved 2.99 saying 2.00` on the frame their spawn ended and lost one fight fifty times over
    /// at one hit; handed over as one and a half on every frame, every dump of stage 4's boss read
    /// `22 bullet(s) no step of their own explained ... moved -1.00,0.01 saying -3.02,0.03, in the game's
    /// own state 2` — a wall of 152 bullets read three times too fast, which is a run left
    /// `nowhere open to it` in a corner it need not have been in.
    ///
    /// One for a kind nothing is read of, which is the step whole: a bullet whose slot holds a word the
    /// switch has no branch for is not moved by that switch at all, so saying it moves by its own field
    /// is the nearest true thing and the way round that fears it rather than flying through it.
    fn how_much_of_its_step_a_bullet_takes(&self, kind: u16) -> f32 {
        match kind {
            2 => 1.0 + 1.0 / 2.0,
            3 => 1.0 + 1.0 / 2.5,
            4 => 1.0 + 1.0 / 3.0,
            5 => 1.0 / 2.0,
            _ => 1.0,
        }
    }

    /// **How many lasers are live**, counted the way the game's own walk counts an enemy: by asking every
    /// slot. The manager keeps no count of them — the count at [`BULLET_COUNT`] is the bullets' — so this
    /// is the whole of the answer. **A plain read a slot**, the manager being a static of the exe's own and
    /// committed from its load: asked with `read_committed`, every state read made 0x100 `VirtualQuery`
    /// calls of the real game.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn lasers_alive(&self) -> i32 {
        (0..LASER_SLOTS)
            .filter(|slot| {
                let laser = G_BULLET_MANAGER + LASERS_AT + slot * LASER_BYTES;
                unsafe { mem::read::<u32>(laser + LASER_ALIVE) != 0 }
            })
            .count() as i32
    }

    /// **How many enemies the player can shoot**, which is the live ones with [`ENEMY_TAKES_THE_SHOT`] and
    /// none of [`ENEMY_UNTESTED`] — asked of every slot, the way [`Th08::lasers_alive`] is.
    ///
    /// **Not the manager's own count at [`ENEMY_COUNT`]**, which is every live enemy: what reads this is
    /// the tuning pass's *nothing to shoot at*, and a stage whose script keeps an untested enemy alive
    /// through its midsection is one that count never lets reach nothing — see [`ENEMY_UNTESTED`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn enemies_to_shoot_at(&self) -> i32 {
        (0..ENEMY_SLOTS)
            .filter(|slot| {
                let enemy = G_ENEMY_MANAGER + ENEMIES_AT + slot * ENEMY_BYTES;
                let flags = unsafe { mem::read::<u32>(enemy + ENEMY_ALIVE) };
                flags & 1 != 0 && flags & ENEMY_TAKES_THE_SHOT != 0 && flags & ENEMY_UNTESTED == 0
            })
            .count() as i32
    }

    /// The record of the track the music's stream is reading — see [`STREAM_SOURCE`] — and nothing while
    /// no track has been played.
    fn the_record_playing() -> Option<usize> {
        let stream =
            mem::read_committed::<usize>(G_SOUND_PLAYER + MUSIC_STREAM).filter(|it| *it != 0)?;
        let source = mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)?;
        mem::read_committed::<usize>(source + SOURCE_RECORD).filter(|it| *it != 0)
    }

    /// **Every live enemy and bullet, as a position and a size**, walked the way the game's own updates
    /// walk them: the enemies at [`ENEMIES_AT`] and the bullets at [`BULLETS_AT`].
    ///
    /// **What this is for is the reading `sight.rs` still needs**, and not for a bot: a launch that says
    /// how many each walk found and where the nearest one to the player is, is a launch that says whether
    /// the strides and the offsets above are right. A position outside the play area, or a count that
    /// disagrees with the game's own, is a number in this module that is wrong.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn hazards_near(&self) -> Nearby {
        // Plain reads, the three objects walked being statics of the exe: see `sight.rs`, where the
        // same walk says what a committed read a slot cost a launch.
        let at = |object: usize, position: usize| unsafe {
            (
                mem::read::<f32>(object + position),
                mem::read::<f32>(object + position + size_of::<f32>()),
            )
        };
        let player = at(G_PLAYER, PLAYER_POSITION);
        let away = |(x, y): (f32, f32)| (x - player.0).hypot(y - player.1);
        let mut found = Nearby {
            enemies: 0,
            bullets: 0,
            items: 0,
            nearest: None,
        };
        let mut how_far = f32::INFINITY;
        let mut note = |object: usize, position: usize, size: usize| {
            let it = at(object, position);
            let size = at(object, size);
            if away(it) < how_far {
                how_far = away(it);
                found.nearest = Some(Nearest { at: it, size });
            }
        };
        for slot in 0..ENEMY_SLOTS {
            let enemy = G_ENEMY_MANAGER + ENEMIES_AT + slot * ENEMY_BYTES;
            if unsafe { mem::read::<u32>(enemy + ENEMY_ALIVE) } & 1 != 0 {
                found.enemies += 1;
                note(enemy, ENEMY_POSITION, ENEMY_SIZE);
            }
        }
        for slot in 0..BULLET_SLOTS {
            let bullet = G_BULLET_MANAGER + BULLETS_AT + slot * BULLET_BYTES;
            if unsafe { mem::read::<u16>(bullet + BULLET_KIND) } != 0 {
                found.bullets += 1;
                note(bullet, BULLET_POSITION, BULLET_SIZE);
            }
        }
        // **And the items, which are a list rather than an array** — see [`ITEM_LIST`]. Counted and not
        // noted: an item is a thing to fly *to* and never the nearest hazard.
        let mut item = mem::read_committed::<usize>(G_ITEM_MANAGER + ITEM_LIST).unwrap_or(0);
        for _ in 0..ITEM_SLOTS {
            if item == 0 {
                break;
            }
            found.items += 1;
            item = mem::read_committed::<usize>(item + ITEM_NEXT).unwrap_or(0);
        }
        found
    }

    /// **The player, the two counts each way, and the nearest hazard**, as one line of a reproduction.
    ///
    /// **Each count twice, and that is the point**: the game keeps its own — `[enemies+0x9dcdc4]` and
    /// `[bullets+0x6ba538]` — and [`Th08::hazards_near`] walks the arrays with the strides this module
    /// claims. Where the two agree the strides are right; where they differ, one of `0x53d0`, `0x1e0`,
    /// `0x10b8`, `0x600`, `0x53d0` and `0x1a880` is wrong and the line says so on the frame it happens.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_world_said(&self) -> String {
        let found = unsafe { self.hazards_near() };
        format!(
            " at=({:.2},{:.2}) half=({:.2},{:.2}) box=({:.2},{:.2})..({:.2},{:.2}) player={} \
             enemies={}/{} bullets={}/{} items={}/{} boss={} nearest={}",
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_POSITION) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_POSITION + size_of::<f32>()) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_HALF) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_HALF + size_of::<f32>()) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_BOX_NEAR) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_BOX_NEAR + size_of::<f32>()) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_BOX_FAR) },
            unsafe { mem::read::<f32>(G_PLAYER + PLAYER_BOX_FAR + size_of::<f32>()) },
            unsafe { mem::read::<u8>(G_PLAYER + PLAYER_STATE) },
            unsafe { mem::read::<i32>(G_ENEMY_MANAGER + ENEMY_COUNT) },
            found.enemies,
            unsafe { mem::read::<i32>(G_BULLET_MANAGER + BULLET_COUNT) },
            found.bullets,
            unsafe { mem::read::<i32>(G_ITEM_MANAGER + ITEM_COUNT) },
            found.items,
            // **Where the fight's own enemy is and how much life it has left**, which is what says
            // whether the rules are standing under it: `Settled::ABoss` puts the player at the boss's
            // own x, so a launch whose boss reads one place and whose player reads another is a launch
            // the bit named the wrong enemy in.
            match unsafe { self.the_boss_enemy() } {
                Some(enemy) => format!(
                    "({:.2},{:.2})/{}",
                    unsafe { mem::read::<f32>(enemy + ENEMY_POSITION) },
                    unsafe { mem::read::<f32>(enemy + ENEMY_POSITION + size_of::<f32>()) },
                    unsafe { mem::read::<i32>(enemy + ENEMY_LIFE) },
                ),
                None => "none".to_owned(),
            },
            match found.nearest {
                Some(Nearest {
                    at: (x, y),
                    size: (width, height),
                }) => format!("({x:.2},{y:.2})+({width:.2}x{height:.2})"),
                None => "none".to_owned(),
            },
        ) + &unsafe { self.the_teams_spec_said() }
    }

    /// **The first sixteen floats of the active team member's own numbers** — see [`G_TEAM_SPEC`], and
    /// `asm.md` for why they are said as numbers rather than named.
    ///
    /// The speeds this game moves the player by are in a `.sht` inside `th08.dat`, so the disassembly
    /// gives the offsets of a file's fields and no values. A launch gives the values: a recipe that held
    /// each direction for twenty frames measured 4.00 straight and 2.83 diagonal unfocused, 2.00 and
    /// 1.41 focused. Which field of this is which is read off by matching those four against this line.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_teams_spec_said(&self) -> String {
        let Some(spec) = mem::read_committed::<usize>(G_TEAM_SPEC).filter(|it| *it != 0) else {
            return " spec=none".to_owned();
        };
        let said: Vec<String> = (0..TEAM_SPEC_SAID)
            .map(
                |at| match mem::read_committed::<f32>(spec + at * size_of::<f32>()) {
                    Some(value) => format!("{value:.2}"),
                    None => "-".to_owned(),
                },
            )
            .collect();
        format!(" spec=[{}]", said.join(","))
    }

    /// **The front end, or nothing where it is not up and settled** — which is the gate, because
    /// [`G_FRONT_END`] is never cleared: the pointer names a freed object for the whole of every run, and
    /// a filter on the pointer catches only the zero it holds before the first register.
    ///
    /// **Two launches, and the second is why the gate is the settled pair and not the scene alone.** The
    /// first read `front end: screen 1900876895, cursor on item 0` on every frame of stage 1, out of the
    /// freed object; gated on the scene now, the second read `front end: screen -1, cursor on item
    /// 26280128` — one line, on the frame the run was asked for. The supervisor's own transition writes
    /// the scene it is entering *after* it registers what it is entering (0x445adf), so on that frame the
    /// scene still reads 1 while the front end has already gone. Its wanted does not, which is what
    /// [`Th08::the_front_end_is_settled`] asks about.
    unsafe fn front_end(&self) -> Option<usize> {
        if !unsafe { self.the_front_end_is_settled() } {
            return None;
        }
        mem::read_committed::<usize>(G_FRONT_END).filter(|front| *front != 0)
    }

    /// Whether the front end is up and settled, which is what says a screen of it is worth reading:
    /// the scene is 1 and nothing else is being asked for.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_front_end_is_settled(&self) -> bool {
        let (scene, wanted) = unsafe { self.scenes() };
        scene == SCENE_FRONT_END && wanted == SCENE_FRONT_END
    }

    /// The scene now and the scene wanted, read together because every answer about where the game is
    /// wants both.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn scenes(&self) -> (i32, i32) {
        (
            unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) },
            unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_WANTED) },
        )
    }

    /// **The object the job `callback` is called on**, off the chain's calc list — see [`G_CHAIN`] — and
    /// nothing where no such job is on it.
    ///
    /// Bounded at [`CHAIN_JOBS_AT_MOST`] and read committed, for the reason `the_jobs_a_frame_has` is: the
    /// chain and its elements are the game's own allocations, and a next pointer read wrong is a walk that
    /// never ends.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_object_a_job_is_called_on(callback: usize) -> Option<usize> {
        let mut element = G_CHAIN + CHAIN_CALC_LIST;
        for _ in 0..CHAIN_JOBS_AT_MOST {
            if element == 0 {
                return None;
            }
            if mem::read_committed::<usize>(element + CHAIN_JOB_CALLBACK)? == callback {
                return mem::read_committed::<usize>(element + CHAIN_JOB_ARGUMENT)
                    .filter(|on| *on != 0);
            }
            element = mem::read_committed::<usize>(element + CHAIN_JOB_NEXT)?;
        }
        None
    }

    /// **The screen a run ends at and the phase it is in**, where the scene is that screen's and its update
    /// is on the chain — see [`RESULT_UPDATE`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_screen_a_run_ends_at(&self) -> Option<(usize, i32)> {
        if unsafe { self.scenes() }.0 != SCENE_RUN_OVER {
            return None;
        }
        let screen = unsafe { Self::the_object_a_job_is_called_on(RESULT_UPDATE) }?;
        Some((screen, mem::read_committed::<i32>(screen + RESULT_PHASE)?))
    }

    /// **The phase of the screen a run ends at, said whenever it changes** — `-1` where no such screen is
    /// up, which is how the line marks it going down. A recipe walking that screen has nothing else to read
    /// which phase a press landed on, and a launch that sat in one for the rest of its bound would otherwise
    /// say nothing about which.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn says_the_screen_a_run_ends_at_as_it_changes(&self) {
        static WAS: AtomicI32 = AtomicI32::new(-1);
        let phase = unsafe { self.the_screen_a_run_ends_at() }.map_or(-1, |(_, phase)| phase);
        if WAS.swap(phase, Ordering::Relaxed) != phase {
            crate::detail!("result: the screen a run ends at is in phase {phase}");
        }
    }

    /// **That screen, while its name entry is up** — the phase as well as the scene, so that a name is
    /// written only where one of the entry's own presses would have written it.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_entry_a_name_is_typed_into(&self) -> Option<usize> {
        let (screen, phase) = unsafe { self.the_screen_a_run_ends_at() }?;
        (phase == RESULT_TYPING_A_NAME).then_some(screen)
    }

    /// **The numbers a stage of this game would be started again from**, which is the set its own replay
    /// record carries — see [`resume::Numbers`], and `asm.md`'s *The replay* for the address each field
    /// was read at.
    ///
    /// **Nothing where there is no run**, which is a read off a pointer the next build frees — see
    /// [`Th08::run_numbers`], whose gate this is behind.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn numbers(&self) -> resume::Numbers {
        let run = unsafe { self.run_numbers() };
        let word = |at: usize| run.and_then(|run| mem::read_committed::<u32>(run + at));
        let whole = |at: usize| {
            run.and_then(|run| mem::read_committed::<f32>(run + at))
                .map(f32::trunc)
        };
        resume::Numbers {
            // **The shown score and not the score**, which on a stage after the first is the total and the other
            // is nought: the build copies the score into the shown one (0x43b14a) and then zeroes the score
            // (0x43b808), so the moment the numbers are read is a moment the score word reads nothing. Read off
            // that word, every stage after a run's first was written down as `score: 0`, and session 11's pick-up
            // of stage 2 landed on `score=24900 against score=46559790` — stage 1's whole 46,534,890 gone.
            score: word(SCORE_SHOWN).unwrap_or(0),
            seed: unsafe { mem::read::<u16>(G_GAME_MANAGER + SEED_AT_STAGE_START) },
            point_items: word(POINT_ITEMS).unwrap_or(0),
            graze: word(GRAZE).unwrap_or(0),
            points_step: word(POINTS_STEP).unwrap_or(0),
            points_wanted: word(POINT_ITEMS_WANTED).unwrap_or(0),
            point_worth: word(POINT_WORTH).unwrap_or(0),
            power: whole(POWER).unwrap_or(0.0) as u16,
            lives: whole(LIVES).unwrap_or(0.0) as i8,
            bombs: whole(BOMBS).unwrap_or(0.0) as i8,
            the_word_at_0x22: run
                .and_then(|run| mem::read_committed::<u16>(run + RUN_WORD_AT_0X22))
                .unwrap_or(0),
            the_byte_at_0x1c: run
                .and_then(|run| mem::read_committed::<u8>(run + RUN_BYTE_AT_0X1C))
                .unwrap_or(0),
            the_byte_at_0x28: run
                .and_then(|run| mem::read_committed::<i8>(run + RUN_BYTE_AT_0X28))
                .unwrap_or(0),
            the_byte_at_164d334: unsafe { mem::read::<u8>(G_BYTE_AT_164D334) },
            the_byte_at_164d0b0: unsafe { mem::read::<u8>(G_BYTE_AT_164D0B0) },
            deaths: whole(DEATHS).unwrap_or(0.0) as i32,
        }
    }

    /// **The whole of what seeding a stage of this game is**, which is four numbers and not one: the
    /// generator's word, the copy of it the game manager keeps, the generator's own kept copy and its
    /// count of draws back to nothing — see [`G_RANDOM`], [`SEED_AT_STAGE_START`] and
    /// [`GENERATOR_KEPT`].
    ///
    /// **All four because a chapter has to draw what the run drew.** The word alone leaves the count
    /// where the run left it and the kept copy holding the word the game would restore from, and a stage
    /// built from that draws the same sequence out of a different place in it.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn seed(&self, seed: u16) {
        unsafe {
            mem::write::<u16>(G_RANDOM, seed);
            mem::write::<u16>(G_RANDOM + GENERATOR_KEPT, seed);
            mem::write::<u32>(G_RANDOM + GENERATOR_DRAWS, 0);
            mem::write::<u16>(G_GAME_MANAGER + SEED_AT_STAGE_START, seed);
        }
    }

    /// **And them put back, the way this game's own replay puts a stage's record back** — all but the
    /// seed, which [`Th08::seed_the_stage`] writes at a moment of its own.
    ///
    /// **Three of them go through the game manager's own setters and not into the run**, which is what
    /// the restore does: 0x452f27 hands the power to 0x406fa0, 0x452f03 the lives to 0x406e20 and
    /// 0x452f15 the bombs to 0x406f30, and each of those turns the whole number into the float the run
    /// keeps — **and reseals the run after it**, which is the other thing in those three functions. orb
    /// writes the same floats and then takes the run's sum again, see [`Th08::seal_the_run`].
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn put_the_numbers_back(&self, numbers: &resume::Numbers) {
        let Some(run) = (unsafe { self.run_numbers() }) else {
            return log!("resume: there is no run of th08 to put numbers back into");
        };
        let word = |at: usize, value: u32| unsafe { mem::write::<u32>(run + at, value) };
        // **Both halves of the score**, which is how a run carried into its next stage has them by its first
        // update: the recording's line of that update read `score=46534990 shown=46534890` — the total and the
        // ten that update adds (0x44c560 through 0x4181f0). Written into the score alone, the panel's chase
        // would climb from nothing.
        word(SCORE, numbers.score);
        word(SCORE_SHOWN, numbers.score);
        word(POINT_ITEMS, numbers.point_items);
        word(GRAZE, numbers.graze);
        word(POINTS_STEP, numbers.points_step);
        word(POINT_ITEMS_WANTED, numbers.points_wanted);
        word(POINT_WORTH, numbers.point_worth);
        unsafe { mem::write::<f32>(run + POWER, f32::from(numbers.power)) };
        unsafe { mem::write::<f32>(run + LIVES, f32::from(numbers.lives)) };
        unsafe { mem::write::<f32>(run + BOMBS, f32::from(numbers.bombs)) };
        unsafe { mem::write::<u16>(run + RUN_WORD_AT_0X22, numbers.the_word_at_0x22) };
        unsafe { mem::write::<u8>(run + RUN_BYTE_AT_0X1C, numbers.the_byte_at_0x1c) };
        unsafe { mem::write::<i8>(run + RUN_BYTE_AT_0X28, numbers.the_byte_at_0x28) };
        unsafe { mem::write::<u8>(G_BYTE_AT_164D334, numbers.the_byte_at_164d334) };
        unsafe { mem::write::<u8>(G_BYTE_AT_164D0B0, numbers.the_byte_at_164d0b0) };
        unsafe { mem::write::<f32>(run + DEATHS, numbers.deaths as f32) };
        unsafe { self.seal_the_run(run) };
    }

    /// **The run sealed again over the numbers just written**, which the game's own setters do and orb's
    /// writes do not — see [`RUN_SEAL_KEY`]: the sum taken the way the run manager takes it every frame, and
    /// kept where the reseal keeps it.
    ///
    /// **Not the reseal (0x406e50) itself**, which is what the restore reaches through the setters: it draws
    /// twice from the generator for its two keys, and the seed is in by the time the numbers are — so the stage
    /// would be built from the seed the run was and then draw from a different place in it.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn seal_the_run(&self, run: usize) {
        unsafe { mem::write::<u32>(run + RUN_SEAL, mem::read::<u32>(run + RUN_SEAL_KEY)) };
        let sum: unsafe extern "thiscall" fn(usize) -> u32 =
            unsafe { std::mem::transmute(run_checksum()) };
        unsafe { mem::write::<u32>(run + RUN_SEALED_SUM, sum(G_GAME_MANAGER)) };
    }

    /// **Whether a line of dialogue is up**, asked exactly the way the game asks it at 0x4358bb — see
    /// [`DIALOGUE_LINE`], and [`STAGE_FRAMES`] for what turns on the answer.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn a_line_is_up(&self) -> bool {
        unsafe { self.dialogue_line() }
            .is_some_and(|line| line >= 0 || line == DIALOGUE_LINE_ENDING)
    }

    /// **Which line of the conversation is up**, or nothing where the screen has no inner yet — see
    /// [`DIALOGUE_LINE`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn dialogue_line(&self) -> Option<i32> {
        mem::read_committed::<usize>(G_RUN_SCREEN + RUN_SCREEN_INNER)
            .filter(|it| *it != 0)
            .and_then(|inner| mem::read_committed::<i32>(inner + DIALOGUE_LINE))
    }

    /// **Tells the game the line up is over**, by writing the value its own question reads as no line at
    /// all — see [`DIALOGUE_LINE`], whose `-1` is what 0x4358bb answers *no* for.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn end_the_line(&self) {
        if let Some(inner) =
            mem::read_committed::<usize>(G_RUN_SCREEN + RUN_SCREEN_INNER).filter(|it| *it != 0)
        {
            unsafe { mem::write(inner + DIALOGUE_LINE, DIALOGUE_NO_LINE) };
        }
    }

    /// **Where the boss is**, or nothing where no fight has one up.
    ///
    /// **Two things have to agree**, which is what makes this a reading rather than a guess: the run
    /// screen's own [`BOSS_IS_UP`] byte, and a live enemy with [`ENEMY_IS_THE_BOSS`] set. The bit is
    /// what the same two ECL instructions write beside that byte — 0x41c51d sets it and 0x41c5ca clears
    /// it — and it is also what keeps the player's own body from damaging that enemy (0x42c375) and what
    /// gates the life bar (0x41c9b5). Found through [`BOSSES`], where the instruction that sets the bit
    /// puts the enemy; the lowest of the live ones there, which is the slot a walk of the array comes to
    /// first.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_boss_enemy(&self) -> Option<usize> {
        if mem::read_committed::<u8>(G_RUN_SCREEN + BOSS_IS_UP) != Some(1) {
            return None;
        }
        (0..BOSS_ENTRIES)
            .map(|entry| unsafe {
                mem::read::<usize>(G_ENEMY_MANAGER + BOSSES + entry * size_of::<usize>())
            })
            .filter(|enemy| {
                *enemy != 0
                    && mem::read_committed::<u32>(enemy + ENEMY_ALIVE)
                        .is_some_and(|flags| flags & 1 != 0 && flags & ENEMY_IS_THE_BOSS != 0)
            })
            .min()
    }

    /// **And where it is**, which is the pair `sight` hands the rules — see [`ENEMY_POSITION`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub(super) unsafe fn the_boss(&self) -> Option<(f32, f32)> {
        let enemy = unsafe { self.the_boss_enemy() }?;
        let x = mem::read_committed::<f32>(enemy + ENEMY_POSITION)?;
        let y = mem::read_committed::<f32>(enemy + ENEMY_POSITION + size_of::<f32>())?;
        Some((x, y))
    }

    /// **Which spell card is up**, `None` where none is — see [`CARD_IS_UP`], the bit `0x4178a0`
    /// answers, and [`CARD_NUMBER`], the number `SpellCard::Start` writes beside it.
    ///
    /// The number is held against [`CARDS`] because it is what indexes [`CARD_RECORDS`]: a card outside
    /// that range is a record this build has none of, and the game's own reader refuses the same way
    /// (0x45ab21).
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_card_that_is_up(&self) -> Option<usize> {
        let flags = mem::read_committed::<u32>(G_SPELL_CARD)?;
        if flags & CARD_IS_UP == 0 {
            return None;
        }
        let card = usize::try_from(mem::read_committed::<i32>(G_SPELL_CARD + CARD_NUMBER)?).ok()?;
        (card < CARDS).then_some(card)
    }

    /// **The byte the game checks a card's record against, worked out the way the game works it out**:
    /// the name's own bytes as signed, then every dword of all six arrays, truncated — see
    /// [`CARD_RECORD_SUM`], whose doc has what a record that fails it loses.
    ///
    /// Written by orb wherever orb changes a count, because the game's own writer (0x416772) only runs
    /// where the game itself counted something.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `record` a record of [`CARD_RECORDS`].
    unsafe fn stamp_a_cards_record(&self, record: usize) {
        let mut sum = 0u32;
        for byte in 0..CARD_RECORD_BEST - CARD_RECORD_NAME {
            let Some(letter) = mem::read_committed::<i8>(record + CARD_RECORD_NAME + byte) else {
                break;
            };
            if letter == 0 {
                break;
            }
            sum = sum.wrapping_add(letter as u32);
        }
        for array in [
            CARD_RECORD_CAPTURES,
            CARD_RECORD_ATTEMPTS,
            CARD_RECORD_BEST,
            CARD_RECORD_OF_A_PRACTICE[2],
            CARD_RECORD_OF_A_PRACTICE[1],
            CARD_RECORD_OF_A_PRACTICE[0],
        ] {
            for slot in 0..CARD_RECORD_SLOTS {
                let at = record + array + slot * size_of::<u32>();
                sum = sum.wrapping_add(mem::read_committed::<u32>(at).unwrap_or(0));
            }
        }
        unsafe { mem::write::<u8>(record + CARD_RECORD_SUM, sum as u8) };
    }

    /// The whole of the viewport this game has, written where it keeps it and set on the device — which
    /// is what both [`Th08::prepare_frame`] and [`Th08::set_play_viewport`] do, differing only in the
    /// rectangle.
    ///
    /// Written as well as set because the game reads its own copy back: [`VIEWPORT`] is the one
    /// `D3DVIEWPORT8` in this exe, and leaving it saying something the device does not is a frame whose
    /// next writer starts from the wrong numbers.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn viewport(&self, device: Device, area: Rect, depth: (f32, f32)) {
        let viewport = Viewport {
            x: area.left as u32,
            y: area.top as u32,
            width: area.width as u32,
            height: area.height as u32,
            min_z: depth.0,
            max_z: depth.1,
        };
        unsafe { mem::write(VIEWPORT, viewport) };
        orb_api::d3d8::set_viewport(device, viewport);
    }
}

impl Game for Th08 {
    /// **The seams this game's own frame is reached through, and no more.**
    ///
    /// The two chain runners put orb inside every frame; the read of the keyboard is what a recipe
    /// presses through; the window and the device are where a launch's screen setting is obeyed. Every
    /// other field is `None`, and each of those is a reading not yet made rather than a decision — see
    /// the list in `asm.md`.
    ///
    /// **`render` is `None` on purpose.** Taking the frame over means calling this game's own drawing in
    /// this game's own order, and the queue it empties and draws between `BeginScene` and `EndScene` has
    /// not been read. th07's first loop over its frame used addresses that were all correct and took the
    /// game down on its first frame; what a frame is is its order and not a list of numbers.
    fn hooks(&self) -> Hooks {
        Hooks {
            update: Some(Patch {
                target: RUN_CALC_CHAIN,
                prologue: RUN_CHAIN_PROLOGUE,
            }),
            draw: Some(Patch {
                target: RUN_DRAW_CHAIN,
                prologue: RUN_CHAIN_PROLOGUE,
            }),
            // Nothing: this game has a function for each of its two walks and the patches above are how
            // orb's loop reaches them.
            walks: None,
            // Nothing: this game runs its frame from its own message pump, and [`RENDER`] is that frame.
            runs_its_frame: None,
            // orb's own cadence, and a frame of input lag gone with it. What this replaces is
            // [`RENDER`], and what the two seams either side of the draw chain are for is the rest of
            // what that frame does — see [`Th08::begin_drawing`].
            render: Some(Patch {
                target: RENDER,
                prologue: RENDER_PROLOGUE,
            }),
            // Nothing of this game's replay write or its recorder has been read; a run here is not
            // rewound, so neither is suppressed.
            save_replay: None,
            stop_recording: None,
            // **Nothing, and that is a reading**: this game builds a stage on a thread of the supervisor's
            // — see [`Th08::a_stage_has_just_been_built`], which answers the moment off the loader's own
            // word instead, and [`SUPERVISOR_LOADING`] for the thread. The run manager's added callback,
            // which this stood over first, returns before the thread has written a byte of the stage.
            stage_begun: None,
            // The job the build calls three instructions past the seed's copy — see [`STAGE_SEEDED`],
            // which has why the seed's seam is inside the build and not in front of it. On the loader's
            // thread, which is where the seed has to go in and where nothing else of orb's runs.
            stage_building: Some(Patch {
                target: STAGE_SEEDED,
                prologue: STAGE_SEEDED_PROLOGUE,
            }),
            // **The one read of the score file that is not the mode's**: what the title lights its items from
            // and what the save at every run's start writes back, so it comes out of the game's own file
            // whichever mode a launch is in — see [`FRONT_END_ADDED`], whose doc has the clears that were lost
            // without it. Entered in `ecx`, this game's chain calling every callback that way.
            unlocks_read: Some(Entered::InEcx(Patch {
                target: FRONT_END_ADDED,
                prologue: FRONT_END_ADDED_PROLOGUE,
            })),
            // The ranking's own read and the card records' seam are not read, so neither is bracketed.
            ranking_read: None,
            card_records_read: None,
            create_window: Some(Patch {
                target: CREATE_GAME_WINDOW,
                prologue: CREATE_GAME_WINDOW_PROLOGUE,
            }),
            init_device: Some(Patch {
                target: INIT_D3D_DEVICE,
                prologue: INIT_D3D_DEVICE_PROLOGUE,
            }),
            input: Some(Read::Code(Patch {
                target: INPUT_READ,
                prologue: INPUT_READ_PROLOGUE,
            })),
            // **Nothing, though this game's pad read is one orb could answer** — which is a decision and
            // not a gap in the reading now. 0x43d0c0 asks for a pad with `joyGetPosEx(0, …)` over a
            // `0x34`-byte `JOYINFOEX` with `dwFlags = 0xff` and keeps the caps at `0x164d388`, and
            // 0x43d120 reads it again every frame and folds `dwButtons` into the same word the keyboard
            // folds into, through the same helper the configuration's own buttons go through (0x43d760).
            // What DirectInput8 is here is the *keyboard*, the branch at 0x43dd18 — see `asm.md`.
            //
            // So a real pad works under orb natively, and orb's own d-pad does not reach a run of this
            // game: what would make it is this pair of hooks and a `reads_a_pad` beside them, which is
            // the same shape th06 and th07 have.
            joystick: None,
            pad_buttons: None,
            // **The refill, and not the stop**: the refill is read — see [`TOPS_UP_THE_MUSIC`] — and the
            // stream's own stop has not been.
            tops_up_the_music: Some(Patch {
                target: TOPS_UP_THE_MUSIC,
                prologue: TOPS_UP_THE_MUSIC_PROLOGUE,
            }),
            stops_the_music: None,
            // And no dialog of its own to get past: what th08 has instead is `custom.exe`, a program
            // beside the game, and a screen inside the game reached from its own title menu.
            startup_dialog: None,
            // Nor a window it looks the making of up: `CreateWindowExA` is imported, and the window stays
            // where it was made.
            makes_its_window: None,
            moves_its_window: None,
            shows_its_window: None,
            places_its_window: None,
            restyles_its_window: None,
            reads_a_pad: None,
            makes_its_renderer: None,
            // The four KERNEL32 imports, off the import table at 0x4b404c: `HeapAlloc` and `HeapFree` are
            // there and no ucrt of its own is.
            takes_its_memory: Memory::Kernel32Imports,
            // `CreateThread` at 0x4b40b8 of the same table, which is the one this game makes threads with.
            makes_its_threads: Threads::CreateThread,
            // `CreateFileA` of the same table, which is how it opens what it reads.
            opens_its_files: FileOpen::CreateFileA,
        }
    }

    /// **Nothing of a run of this game has been put back on the real game yet, so there is nothing to
    /// rewind to.**
    ///
    /// A chapter is a snapshot of a run plus the numbers to put back at it. All four things that wants
    /// are read now: the numbers themselves — see [`LIVES`] and its neighbours — the clock a midstage
    /// boundary is a number on, which is [`SCRIPT_CONTEXTS`] and which a launch has measured restarting
    /// with the stage, the way out of a run, which [`Th08::leave_run`] takes, and the two lists a
    /// snapshot has to leave out.
    ///
    /// **What is left is the dozen methods a mode needs besides a chapter.** The chapter itself is in:
    /// `resume` has the set a stage is started again from, read out of this game's own replay record
    /// both ways round; [`Th08::put_the_numbers_back`] writes it, [`Th08::seed`] seeds the stage, and
    /// [`Th08::a_stage_has_just_been_built`] and [`STAGE_SEEDED`] are the two moments the two of those
    /// go in at.
    ///
    /// **And most of what a mode is made of around that is in now**: [`Th08::run_finished`] and
    /// [`Th08::the_screen_a_run_ends_at_is_up`] off scene 6, [`Th08::leave_the_screen_a_run_ends_at`],
    /// [`Th08::captures`] and its three neighbours over the 222 `CATK` records,
    /// [`Th08::count_card_attempt`] with the byte those records are checked against,
    /// [`Th08::make_invulnerable`] over the hit's own window, [`Th08::run_slot`] and
    /// [`Th08::run_pointed_at`].
    ///
    /// **Both lists a snapshot has to *leave out* are in now.** [`Th08::audio_state`] is the manager,
    /// the wrapper it keeps DirectSound in and the stream a played track has, each bounded by the code
    /// that makes it; [`Th08::live_handles`] is the sheets of every `.anm` slot the sprite manager
    /// holds, which is where this game's textures and their blocks live.
    ///
    /// **What is left of the sound is the two calls a restore makes, and a restore of this game makes
    /// neither.** `Th08::stop_music` and `Th08::restart_stage_music` answer nothing — `PlayBGM`
    /// (0x45cc70) and its take-down (0x45d2d0) are read and neither is handed over, and which track a
    /// stage plays is not read at all. Which is the *same* path th07's own boss-fight chapters take:
    /// `Snapshot::music_still_playing` answers true for a snapshot that holds no stream, so a restore
    /// calls neither and leaves the music playing over memory [`Th08::audio_state`] held back.
    ///
    /// **And what is left is a chapter tried on the real game**, which is why this still answers false:
    /// nothing here has put a snapshot of this game back, and a `true` that has never been restored
    /// against is a mode offered on a reading nobody has held to the exe. That launch is this change's
    /// own TODO.
    ///
    /// `show_ranking` and its three, `jump_to_stage`, `panel_tile` and the music's four decline here as
    /// they decline for th06nc, which rewinds: they are what a mode does *beside* a chapter and not what
    /// one is made of.
    ///
    /// Until that launch a launch here is a normal-mode launch, which is what `attached` says in the log.
    fn rewinds(&self) -> bool {
        true
    }

    /// Where the game is, and the emptiest true answer for everything about a run.
    ///
    /// The scenes, the stage, the difficulty and the frames are readings; the rest is not, and a zero
    /// here is orb declining rather than orb measuring nothing. `asm.md` says which is which.
    /// **A line with words on it**, which is the narrower predicate: [`DIALOGUE_LINE`] holds the line's
    /// own number, and the `-2` the game also counts as a conversation is the frame it hands the screen
    /// back on. So `State::in_dialogue` takes both and this takes the numbered ones.
    unsafe fn a_line_of_dialogue_is_shown(&self) -> bool {
        unsafe { self.dialogue_line() }.is_some_and(|line| line >= 0)
    }

    unsafe fn read_state(&self) -> State {
        unsafe { self.says_the_screen_a_run_ends_at_as_it_changes() };
        let (scene, wanted) = unsafe { self.scenes() };
        let kind: u32 = unsafe { mem::read(G_RUN_KIND) };
        let run = unsafe { self.run_numbers() };
        let boss = unsafe { self.the_boss_enemy() };
        let whole = |at: usize| {
            run.map_or(0.0, |run| unsafe { mem::read::<f32>(run + at) })
                .trunc()
        };
        State {
            scene,
            wanted,
            playing: scene == SCENE_RUN,
            in_run: scene == SCENE_RUN || scene == SCENE_RUN_AGAIN,
            // Somebody at the keyboard: the gameplay scene, and no recording played in it.
            in_game: scene == SCENE_RUN && kind & RUN_READS_ITS_OWN_INPUT == 0,
            in_ending: scene == SCENE_ENDING,
            // **Not read, where the scene above is.** th07 answers this by walking the game's own chain for
            // the pointer to the ending's script, and what th08's ending object holds is unread — the
            // register's `0x2ab8` bytes have not been gone through. Nothing of orb's needs it: what asks is
            // the line that says how far an ending has got, where this game's own scene is what the ranking
            // waits on.
            ending_script: None,
            // The two bits of [`G_RUN_KIND`] a recording is played under — see it for which writes which.
            demo: kind & RUN_IS_THE_DEMO != 0,
            replay: kind & RUN_READS_ITS_OWN_INPUT != 0 && kind & RUN_IS_THE_DEMO == 0,
            // **A stage practised on its own**, one stage rather than a whole run: bit 0 of the kind
            // without the spell practice's bit 14. The title's fourth item sets the first and clears the
            // second (0x467dde, 0x467deb) and the stage select sets the first again as it comes up
            // (0x46af8e), where Start and Extra Start clear both (0x467d28, 0x467ea6) — see
            // [`TITLE_ITEM_START`], whose doc has the jump table the items dispatch through.
            practice: kind & RUN_STARTS_WITH_EIGHT != 0 && kind & RUN_IS_A_SPELL_PRACTICE == 0,
            // The game's own pause menu up — see [`G_PAUSED`].
            paused: unsafe { mem::read::<u8>(G_PAUSED) } != 0,
            // The player being in any of its own three hit states — see [`PLAYER_STATE`].
            unsettled: unsafe { mem::read::<u8>(G_PLAYER + PLAYER_STATE) } != 0,
            bombing: false,
            in_dialogue: unsafe { self.a_line_is_up() },
            stage: unsafe { mem::read(G_STAGE) },
            difficulty: unsafe { mem::read(G_GAME_MANAGER + DIFFICULTY) },
            // **The run's clock less what it read when this stage was built**, which is the count from the
            // stage's own first frame that [`STAGE_FRAMES`] is not — see its doc, whose launch settled
            // that, and [`Th08::a_stage_has_just_been_built`], which writes the subtrahend. **A count that
            // carries across a stage is one the chapters never see begin**: `Chapters` takes a stage as
            // replaced where this reads nought or one, and a whole Normal run in Pointdevice mode read
            // `resume: stage 2 begins` and then took no chapter of stage 2 at all — its three deaths there
            // went unretried and the run ended at `scene=6`, where stage 1's had been answered by the
            // chapter every time.
            stage_frames: unsafe { mem::read::<u32>(G_GAME_MANAGER + STAGE_FRAMES) }
                .wrapping_sub(STAGE_BEGAN_AT.load(Ordering::Relaxed)),
            // **Two clocks and not one field twice.** A midstage boundary wants a clock that advances the
            // same way however the player is doing, and both of these do: 0x42c6a1 steps [`STAGE_FRAMES`]
            // only where 0x4358bb answers no, and that question is *is a line of dialogue up* and nothing
            // else — see [`DIALOGUE_LINE`] — while the context below is stepped inside the same update, so
            // a frame the walk never reaches is a frame neither counts. A death does not stop either, and
            // a bomb does not. What stands here is the stage script's own clock — see [`SCRIPT_CONTEXTS`],
            // which is the first context of the `.ecl` the stage loads — and it waits out a whole fight on
            // one number where the count above does not.
            script_frames: unsafe {
                mem::read::<u32>(G_ENEMY_MANAGER + SCRIPT_CONTEXTS + A_COUNTDOWN_COUNT)
            } as i32,
            random_seed: unsafe { mem::read(G_RANDOM) },
            deaths: whole(DEATHS) as i32,
            lives: whole(LIVES) as i8,
            bombs: whole(BOMBS) as i8,
            power: whole(POWER) as u16,
            enemy_count: unsafe { self.enemies_to_shoot_at() },
            bullet_count: unsafe { mem::read(G_BULLET_MANAGER + BULLET_COUNT) },
            laser_count: unsafe { self.lasers_alive() },
            // The run screen's own byte, which is what the ECL writes as a fight begins and ends — see
            // [`BOSS_IS_UP`].
            boss_present: mem::read_committed::<u8>(G_RUN_SCREEN + BOSS_IS_UP) == Some(1),
            // **And the boss's own life, off the enemy the bit names**: `[+0x2dfc]`, which the damage the
            // player's own body does takes 10 off at 0x42c391 and the life-bar instruction writes at
            // 0x41c989 beside the maximum at `+0x2e00`.
            boss_life: boss.and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_LIFE)),
            // **And how long its current attack has run** — see [`ENEMY_ATTACK_FRAMES`], which is what
            // makes a nonspell after a card a chapter of its own.
            boss_attack_frames: boss
                .and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_ATTACK_FRAMES)),
            // **The card that is up, by its own number** — see [`G_SPELL_CARD`], and
            // [`Th08::the_card_that_is_up`] for the bit that says one is.
            spellcard: unsafe { self.the_card_that_is_up() }
                .and_then(|card| u32::try_from(card).ok()),
        }
    }

    unsafe fn window(&self) -> Hwnd {
        unsafe { mem::read(G_GAME_WINDOW) }
    }

    unsafe fn d3d_device(&self) -> Device {
        Device(unsafe { mem::read(G_D3D_DEVICE) })
    }

    /// Nothing: the stream this game feeds its music through is read as far as which track it is on — see
    /// [`STREAM_SOURCE`] — and not as far as where in the track it is, which is what a `Music` puts back,
    /// so orb holds no music of it across anything.
    ///
    /// **And a `Music` would take away what a restore of this game does right.** `Th08::audio_state` holds
    /// the manager, its DirectSound wrapper, the stream and its source out of every restore, so the song plays
    /// on across one from where it had got to: a chapter of stage 1 taken again read `19,865,432` bytes of the
    /// track left on the refill before the restore and `19,821,332` on the first after it, the countdown
    /// falling a `44,100`-byte chunk a refill from there on — no stretch of it gone round again, which is what
    /// th07's own rewind came back with before its fixes. A position put back would be a seek into a stream
    /// that has moved on.
    fn music(&self) -> Option<Music> {
        None
    }

    /// **Which track the stream is reading**, out of the three numbers its record is sought by — see
    /// [`STREAM_SOURCE`] — and nothing while no track has been played. The shape th07's identity has,
    /// for th07's reason: one file holds every track, so what tells two apart is where each is in it.
    fn music_identity(&self) -> Option<u32> {
        let record = Self::the_record_playing()?;
        let at = mem::read_committed::<u32>(record + RECORD_AT)?;
        let loops_at = mem::read_committed::<u32>(record + RECORD_LOOP_AT)?;
        let bytes = mem::read_committed::<u32>(record + RECORD_BYTES)?;
        Some(bytes ^ at.rotate_left(11) ^ loops_at.rotate_left(22))
    }

    /// **The stream's own numbers and nothing else**, this being the game's streaming thread — see
    /// [`TOPS_UP_THE_MUSIC`]: how much goes in, where in the buffer it goes, and how much of the track is left
    /// before the loop, which is what says a section loop — the countdown climbing down once a pass over the
    /// song where a stream going round a few seconds of itself turns it over a narrow range.
    unsafe fn the_refill_is_about_to(&self, stream: usize) -> Option<String> {
        let source = mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)?;
        let left = mem::read_committed::<u32>(source + SOURCE_LEFT)?;
        let write = mem::read_committed::<u32>(stream + STREAM_WRITE_AT)?;
        let chunk = mem::read_committed::<u32>(stream + STREAM_CHUNK)?;
        Some(format!(
            "puts {chunk} byte(s) in at {write}, {left} byte(s) of the track left before it loops",
        ))
    }

    /// The name on the record the stream is reading, which is the file's own name for that track — see
    /// [`STREAM_SOURCE`].
    fn the_track_heard(&self) -> Option<String> {
        let record = Self::the_record_playing()?;
        let name = name_at(record + RECORD_NAME, RECORD_NAME_BYTES)?;
        Some(String::from_utf8_lossy(&name).into_owned())
    }

    /// **The two fourth stages**, whose boss fights, leaves for a stretch of the stage and comes back — see
    /// [`STAGES_WHOSE_BOSS_COMES_BACK`].
    fn the_stages_boss_comes_back(&self, stage: i32) -> bool {
        STAGES_WHOSE_BOSS_COMES_BACK.contains(&stage)
    }

    /// **And whether that track is one of the fight's** — a track the stage's info block names after its
    /// first, see [`G_STAGE_INFO`] — which is what tells a boss this game gives its own song from a
    /// midboss fought to the stage's.
    ///
    /// By name rather than by the track having changed since the stage began, because this game's stage
    /// starts its song after the frame its first chapter is taken on — a launch read `stage 2 chapter 1
    /// (stage start) at frame 249, music: false` — so a change counted from there would make the stage's
    /// own song the change and every midboss a boss. Nothing where the first slot holds a card's track,
    /// which is a stage whose names have not been read.
    fn the_fights_own_song_is_playing(&self) -> Option<bool> {
        let kind: u32 = unsafe { mem::read(G_RUN_KIND) };
        if kind & RUN_LOADS_A_CARDS_TRACK != 0 {
            return None;
        }
        let record = Self::the_record_playing()?;
        let track = name_at(record + RECORD_NAME, RECORD_NAME_BYTES)?;
        let info = mem::read_committed::<usize>(G_STAGE_INFO).filter(|it| *it != 0)?;
        Some(
            STAGE_TRACK_NAMES[1..]
                .iter()
                .filter_map(|name| name_at(info + name, TRACK_NAME_BYTES))
                .filter(|name| name.first() != Some(&b' '))
                .any(|fights| same_song(&fights, &track)),
        )
    }

    /// Whether the manager's own queue still holds a command — see [`SOUND_COMMANDS`].
    fn a_sound_command_is_in_flight(&self) -> bool {
        mem::read_committed::<u32>(G_SOUND_PLAYER + SOUND_COMMANDS).is_some_and(|kind| kind != 0)
    }

    /// The id `PlayBGM`'s own `CreateThread` wrote down — see [`MUSIC_THREAD_ID`] — and nothing while no
    /// track has been played, which is the zero the manager's constructor left there.
    fn audio_thread(&self) -> Option<u32> {
        mem::read_committed::<u32>(G_SOUND_PLAYER + MUSIC_THREAD_ID).filter(|id| *id != 0)
    }

    /// Nothing stopped, though the call that would do it is read: 0x45d2d0 is the take-down `PlayBGM`
    /// itself reaches for at 0x45cced, `__thiscall` on [`G_SOUND_PLAYER`].
    ///
    /// **Because a restore of this game has no sound to take down**: it leaves the sound objects alone and the
    /// song plays on — see `Th08::music` for what was heard. What this is for is a restore that cannot put
    /// the sound back, and `Th08::audio_state` is why this one never has to.
    unsafe fn stop_music(&self) {}

    /// Nothing put back. Which track a stage plays is read — the first of [`G_STAGE_INFO`]'s names — and
    /// so is the play that would start it, 0x447f21.
    ///
    /// **For the reason `Th08::stop_music` does nothing**: the stream a restore would have to rebuild is
    /// never taken down, so the track playing after one is the track that was playing before it.
    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    /// **The objects the sound system is, which a restore has to leave alone**: the manager itself, the
    /// wrapper it keeps DirectSound in, and the stream a played track has — see [`SOUND_PLAYER_BYTES`],
    /// [`SOUND_DEVICE`] and [`MUSIC_STREAM`], where each extent is read off the code that makes it.
    ///
    /// The manager is always there, being a static; the other two are an object a playing track has and
    /// nothing else does, so a launch with no music answers the one range.
    ///
    /// **And the event and the thread are inside the manager's own range** rather than ranges of their
    /// own: `+0x520c`, `+0x614` and `+0x618` are all below `0x5224`, so a restore that leaves the
    /// manager alone leaves the handle the streaming thread is waiting on alone with it.
    ///
    /// **What is not here is the notification array**, and it is not an omission: 0x471dbb allocates one
    /// 8-byte entry per notification, hands them to `IDirectSoundNotify::SetNotificationPositions` at
    /// 0x471e2d and frees them again at 0x471ea4, so no field of the game's names that memory afterwards
    /// — which is the difference from th07, where the array of buffers is what DirectSound made and
    /// lives nowhere else.
    fn audio_state(&self) -> Vec<Range<usize>> {
        // `once` rather than `vec!` of the one range, which is the lint th07's two answer the same way.
        let mut ranges: Vec<Range<usize>> =
            std::iter::once(G_SOUND_PLAYER..G_SOUND_PLAYER + SOUND_PLAYER_BYTES).collect();
        let device =
            mem::read_committed::<usize>(G_SOUND_PLAYER + SOUND_DEVICE).filter(|it| *it != 0);
        if let Some(device) = device {
            ranges.push(device..device + SOUND_DEVICE_BYTES);
        }
        let stream =
            mem::read_committed::<usize>(G_SOUND_PLAYER + MUSIC_STREAM).filter(|it| *it != 0);
        if let Some(stream) = stream {
            ranges.push(stream..stream + MUSIC_STREAM_BYTES);
            // **And the source the stream reads through**, whose record and count of bytes left are the
            // file's own position said twice: put back from a snapshot while the file plays on, they name
            // the chapter's track and the file another — see [`STREAM_SOURCE`].
            if let Some(source) =
                mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)
            {
                ranges.push(source..source + MUSIC_SOURCE_BYTES);
            }
        }
        ranges
    }

    /// **The sheets of every `.anm` slot the sprite manager holds**, one range per slot with a file in
    /// it — see [`ANM_SLOTS_AT`], where the slot's own shape is read off the free, and [`A_SHEET_BYTES`],
    /// where the two fields of a sheet are.
    ///
    /// A range per slot and not per sheet, because a slot's sheets are one allocation: the whole of it is
    /// what a restore has to leave standing, the texture DirectX made and the block the game's own
    /// allocator gave alike.
    ///
    /// Nothing at all before the manager is made, which is every frame of a launch that has not drawn
    /// yet, and nothing for the slots with no file in them — `0x100` slots being what the manager keeps
    /// and a handful what a run has loaded.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        let Some(manager) =
            mem::read_committed::<usize>(G_QUAD_QUEUE).filter(|manager| *manager != 0)
        else {
            return Vec::new();
        };
        (0..ANM_SLOTS)
            .filter_map(|slot| {
                let slot = manager + ANM_SLOTS_AT + slot * ANM_SLOT_BYTES;
                mem::read_committed::<usize>(slot + ANM_SLOT_FILE).filter(|file| *file != 0)?;
                // **Bounded rather than believed**, which is what th07's own two lists do with a length
                // read off memory: a count a restore is halfway through putting back, handed to a hole,
                // is a range that would hold megabytes of somebody else's back.
                let sheets = usize::try_from(
                    mem::read_committed::<u32>(slot + ANM_SLOT_SHEETS)?.clamp(1, SHEETS_AT_MOST),
                )
                .ok()?;
                let at = mem::read_committed::<usize>(slot + ANM_SLOT_SHEETS_AT)
                    .filter(|at| *at != 0)?;
                Some(at..at + sheets * A_SHEET_BYTES)
            })
            .collect()
    }

    fn play_area(&self) -> Rect {
        PLAY_AREA
    }

    /// Nothing: the sprite this game's panel is drawn from has not been read, so orb has no tile of its
    /// own to draw with.
    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        None
    }

    /// Nothing: there is no row of this game's orb draws over, so there is none to ask for again.
    unsafe fn repaint_lives_row(&self) {}

    /// False: orb draws no row of this game's lives, so nothing of its own stands where one would be.
    unsafe fn draws_lives_row(&self) -> bool {
        false
    }

    fn lives_row(&self) -> Rect {
        LIVES_ROW
    }

    fn content_size(&self) -> (u32, u32) {
        BACK_BUFFER
    }

    fn windowed(&self) -> bool {
        unsafe { mem::read::<u8>(CFG_WINDOWED) != 0 }
    }

    unsafe fn force_windowed(&self) {
        unsafe { mem::write::<u8>(CFG_WINDOWED, 1) };
    }

    unsafe fn set_play_viewport(&self, device: Device) {
        unsafe { self.viewport(device, PLAY_AREA, (0.5, 1.0)) };
    }

    fn chain(&self) -> *mut c_void {
        G_CHAIN as *mut c_void
    }

    /// False: which of this game's options wipes the back buffer has not been read, and the wrong way
    /// round here is orb's own drawing hardening into the corners of a frame — see th07, whose answer is
    /// the same and for the same reason.
    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    /// The keyboard needs nothing done to it, which is what `true` says: **th08 reads its keys with
    /// `GetKeyboardState`**, and the system never takes that away.
    ///
    /// Read off the read itself — 0x43d996 calls `[0x4b41cc]`, the second entry of the USER32 import
    /// table at 0x4b41c8, with a 0x100-byte buffer on its own stack, and every key it wants is a byte of
    /// that buffer tested against 0x80. There is a DirectInput path behind `0x17ce768` and orb stands in
    /// front of the whole function, so which one the game would have taken does not reach it.
    unsafe fn acquire_input(&self) -> bool {
        true
    }

    /// Nothing was let go of, because there is nothing to let go of — see [`Th08::acquire_input`].
    unsafe fn take_sent_keys(&self) -> bool {
        false
    }

    unsafe fn prepare_frame(&self, device: Device) {
        unsafe {
            self.viewport(
                device,
                Rect {
                    left: 0.0,
                    top: 0.0,
                    width: BACK_BUFFER.0 as f32,
                    height: BACK_BUFFER.1 as f32,
                },
                (0.0, 1.0),
            )
        };
    }

    /// **What this game's own frame does between `BeginScene` and its draw chain**, which is the order
    /// orb's loop has to keep: the queue emptied, the fog flag forced on, and the fog put out.
    ///
    /// Read off 0x441fc8 to 0x441fe7 and made in that order. The fog flag is written before the call
    /// because the call is what reaches the device and it does nothing where the flag says the fog is
    /// already out — see [`FOG_ON`].
    unsafe fn begin_drawing(&self, _device: Device) {
        let queue = unsafe { mem::read::<usize>(G_QUAD_QUEUE) };
        let empty: unsafe extern "thiscall" fn(usize) =
            unsafe { std::mem::transmute(empty_queue()) };
        unsafe { empty(queue) };
        unsafe { mem::write::<u32>(G_SUPERVISOR + FOG_ON, FOG_ON_FORCED) };
        let fog: unsafe extern "thiscall" fn(usize) = unsafe { std::mem::transmute(fog_off()) };
        unsafe { fog(G_SUPERVISOR) };
    }

    /// And what it does after that chain: the queue drawn — 0x441ff1, the last thing before the frame
    /// unbinds its texture and ends the scene.
    unsafe fn end_drawing(&self, _device: Device) {
        let queue = unsafe { mem::read::<usize>(G_QUAD_QUEUE) };
        let draw: unsafe extern "thiscall" fn(usize) = unsafe { std::mem::transmute(draw_queue()) };
        unsafe { draw(queue) };
    }

    fn frame_calls(&self) -> FrameCalls {
        FrameCalls {
            play_sounds: Call {
                function: PLAY_SOUNDS,
                this: G_SOUND_PLAYER,
            },
            present: Call {
                function: PRESENT,
                this: 0,
            },
            // Both are this exe's own functions at this exe's own addresses.
            orbs_own: false,
        }
    }

    /// False: this game's replay manager has not been read, so there is no record of a stage to check a
    /// jump against and none to make.
    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    /// **The window a hit is taken over, entered and its countdown put back** — which for this game is
    /// the whole of being unhittable: all four of its hit tests decline while [`PLAYER_STATE`] is not
    /// nothing, and there is no window beside that one.
    ///
    /// **Which is why the countdown goes in with it.** That state ends in a death and not in the game
    /// taking it back: 0x44cbf0 takes the life and puts the player back at the start the frame
    /// [`PLAYER_HIT_COUNTDOWN`] reaches `0x1e`. So what makes the player unhittable for the update about to
    /// run is the state *and* the count put back to nothing, the way the hit handler itself does it
    /// (0x44ab84 through 0x44ab92), and an update that does both never reaches the frame the life would go
    /// on. **And a player held in it moves nothing**, the state skipping the player's input (0x44c4b5
    /// through 0x44c4cb): a run under `--clear` is played standing where the build put it.
    ///
    /// **And [`CARD_UPDATE_HELD`] with them, which is the same handler's own first act** (0x44ab57):
    /// a card's update does nothing at all while that byte stands, and the handler is the only thing
    /// that clears it — so a player made unhittable without it is a fight that can stop for good. It did:
    /// a `--clear` launch held the last stage's Last Spell for eleven minutes of game time with the
    /// stage's clock frozen and not a bullet on the screen.
    unsafe fn make_invulnerable(&self) {
        let countdown = G_PLAYER + PLAYER_HIT_COUNTDOWN;
        unsafe {
            mem::write::<u8>(G_PLAYER + PLAYER_STATE, PLAYER_HIT);
            mem::write::<i32>(countdown + A_COUNTDOWN_COUNT, 0);
            mem::write::<i32>(countdown + A_COUNTDOWN_PART, 0);
            mem::write::<i32>(countdown + A_COUNTDOWN_PHASE, A_COUNTDOWN_RESTARTS_AT);
            mem::write::<u8>(G_GAME_MANAGER + CARD_UPDATE_HELD, 0);
        }
    }

    /// Whether the run being played is one being fed its own recorded input — the bit the build reads a
    /// run's numbers out of a record for, which a replay and the demo both set. See [`G_RUN_KIND`].
    unsafe fn replaying(&self) -> bool {
        let kind = unsafe { mem::read::<u32>(G_RUN_KIND) };
        kind & RUN_READS_ITS_OWN_INPUT != 0
    }

    /// Nothing: this game's ranking screen has not been read.
    unsafe fn show_ranking(&self) {}

    unsafe fn showing_ranking(&self) -> bool {
        false
    }

    unsafe fn ranking_scene(&self) -> bool {
        false
    }

    unsafe fn restore_menu_cursor(&self) {}

    unsafe fn ranking_state(&self) -> String {
        let (scene, wanted) = unsafe { self.scenes() };
        format!("cur={scene} wanted={wanted} screen=unread")
    }

    unsafe fn leave_ranking(&self) {}

    /// **One more attempt at the card that is up**, counted where the game cannot count it: the
    /// character's own slot and the total beside it, both stopping at [`A_CARDS_COUNT_STOPS_AT`] the way
    /// the game's own increment does (0x415aef).
    ///
    /// **And the record stamped afterwards**, because a count written without it is a record the game
    /// zeroes the next time it looks — see [`CARD_RECORD_SUM`].
    ///
    /// **Not counted for a spell practice**, which is the gate the game's own count has at 0x415adb:
    /// bit 14 of the run kind sends it to the pair at [`CARD_RECORD_OF_A_PRACTICE`]`[1]` instead, and a
    /// practice of one card is a run orb rewinds nothing of — so the count orb would be adding to is
    /// one nothing of orb's ever takes back.
    ///
    /// The count answered is the character's own and not the total, that being the one a screen shows
    /// per shot.
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        let card = unsafe { self.the_card_that_is_up() }?;
        if unsafe { mem::read::<u32>(G_RUN_KIND) } & RUN_IS_A_SPELL_PRACTICE != 0 {
            return None;
        }
        let record = CARD_RECORDS + card * CARD_RECORD_BYTES;
        let character = usize::from(mem::read_committed::<u8>(G_CHARACTER)?);
        if character >= CARD_RECORD_EVERY_CHARACTER {
            return None;
        }
        let mut counted = 0;
        for slot in [character, CARD_RECORD_EVERY_CHARACTER] {
            let at = record + CARD_RECORD_ATTEMPTS + slot * size_of::<u32>();
            let count = mem::read_committed::<u32>(at)?;
            let count = match count < A_CARDS_COUNT_STOPS_AT {
                true => count + 1,
                false => count,
            };
            unsafe { mem::write::<u32>(at, count) };
            if slot == character {
                counted = count;
            }
        }
        unsafe { self.stamp_a_cards_record(record) };
        u16::try_from(counted).ok()
    }

    /// Nothing: this game bakes a card's name into a sprite from its own ECL and orb has read neither the
    /// bake nor the plate, so there is nothing of it to draw again.
    unsafe fn redraw_card_name(&self) {}

    /// **The 222 records this game keeps one per spell card**, as the bytes they stand in — see
    /// [`CARD_RECORDS`].
    ///
    /// The live array and not the copy at `[gm+0x1e268]`: that copy is what the file was last written
    /// from, and what a chapter has to hold across a rewind is what the session has counted.
    unsafe fn captures(&self) -> Vec<u8> {
        let mut records = vec![0u8; CARDS * CARD_RECORD_BYTES];
        for (at, byte) in records.iter_mut().enumerate() {
            *byte = mem::read_committed::<u8>(CARD_RECORDS + at).unwrap_or(0);
        }
        records
    }

    /// **Whether any of those records holds a count**, which is the three arrays that are read and not
    /// the bytes: a record the game has just stamped holds its tag, its length, its version and its
    /// index, so counting bytes answers full for an array that has never been played.
    fn these_records_hold_a_count(&self, records: &[u8]) -> bool {
        (0..CARDS).any(|card| {
            [CARD_RECORD_BEST, CARD_RECORD_ATTEMPTS, CARD_RECORD_CAPTURES]
                .into_iter()
                .flat_map(|array| {
                    (0..CARD_RECORD_SLOTS).map(move |slot| array + slot * size_of::<u32>())
                })
                .any(|inside| {
                    let at = card * CARD_RECORD_BYTES + inside;
                    records
                        .get(at..at + size_of::<u32>())
                        .is_some_and(|word| word.iter().any(|byte| *byte != 0))
                })
        })
    }

    unsafe fn set_captures(&self, saved: &[u8]) {
        for (at, byte) in saved.iter().take(CARDS * CARD_RECORD_BYTES).enumerate() {
            unsafe { mem::write::<u8>(CARD_RECORDS + at, *byte) };
        }
    }

    /// **And the same put back with every name left as it stands**, which is the one thing about a card a
    /// playback tells the session: the counts come out of `saved` and the bytes from
    /// [`CARD_RECORD_NAME`] to [`CARD_RECORD_BEST`] are left alone.
    ///
    /// **Then each record stamped**, because a name from now with counts from then is a pair the byte at
    /// `+0xe` was never written for.
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]) {
        for card in 0..CARDS {
            let record = CARD_RECORDS + card * CARD_RECORD_BYTES;
            for at in 0..CARD_RECORD_BYTES {
                if (CARD_RECORD_NAME..CARD_RECORD_BEST).contains(&at) {
                    continue;
                }
                let Some(byte) = saved.get(card * CARD_RECORD_BYTES + at) else {
                    return;
                };
                unsafe { mem::write::<u8>(record + at, *byte) };
            }
            unsafe { self.stamp_a_cards_record(record) };
        }
    }

    /// Nothing: this game's ranking read is a seam orb does not stand in — `Hooks::ranking_read` is
    /// `None` — so nothing of orb's runs in front of the read this would be emptying the records for.
    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// Nothing: orb keeps no file of its own for this game, so there is nothing of a mode's own to read
    /// back out of one.
    unsafe fn read_what_the_modes_file_records(&self) {}

    /// What the title menu's cursor is pointing at, as one of the three things orb's own question is put
    /// over: a run, the replays or the scores.
    ///
    /// The four items that start a run are one answer, the way th07's three are — Start, Extra Start,
    /// Stage Practice and Spell Practice all build a run out of the same manager, so the question over
    /// any of them is the one question.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        if !unsafe { self.the_front_end_is_settled() } {
            return None;
        }
        let front = unsafe { self.front_end() }?;
        if mem::read_committed::<i32>(front + FRONT_END_SCREEN)? != SCREEN_TITLE {
            return None;
        }
        match mem::read_committed::<i32>(front + FRONT_END_CURSOR)? {
            TITLE_ITEM_START
            | TITLE_ITEM_EXTRA
            | TITLE_ITEM_PRACTICE
            | TITLE_ITEM_SPELL_PRACTICE => Some(Menu::Run),
            TITLE_ITEM_REPLAY => Some(Menu::Replays),
            TITLE_ITEM_SCORE => Some(Menu::Scores),
            _ => None,
        }
    }

    /// Whether the screen that is up would act on a press: the title menu's items are read only in
    /// phase 1 and only once its own fade-in has counted to [`SETTLED_FRAMES`] — the two the gate at
    /// 0x467cac and the switch at 0x4674f2 make.
    ///
    /// **False for every other screen of this game**, which is a reading not yet made rather than an
    /// answer: what a press does on the difficulty select and the character select has not been read,
    /// so orb holds no press back over them.
    unsafe fn menu_takes_a_press(&self) -> bool {
        let Some(front) = (unsafe { self.front_end() }) else {
            return false;
        };
        if mem::read_committed::<i32>(front + FRONT_END_SCREEN) != Some(SCREEN_TITLE) {
            return false;
        }
        mem::read_committed::<i32>(front + FRONT_END_PHASE) == Some(TITLE_PHASE_ITEMS)
            && mem::read_committed::<i32>(front + FRONT_END_SETTLED)
                .is_some_and(|settled| settled >= SETTLED_FRAMES)
    }

    /// Nothing held: orb stands in front of none of this game's pad read, so a pad reaches it the way the
    /// system gives it — see [`Th08::hooks`], where the read is written down and the two hooks that would
    /// stand over it are declined.
    unsafe fn pad(&self, _pad: Option<Reading>) -> Pad {
        Pad::none()
    }

    unsafe fn pad_word(&self, _pad: Option<Reading>, _hats: bool) -> u16 {
        0
    }

    unsafe fn add_pad_buttons(&self, _into: *mut c_void, _pad: Option<Reading>) {}

    /// **The run handed back to the front end, and a crash is why it is done this way.** Asking for the
    /// front end by writing the scene wanted is not how this game leaves a run, and the launch that
    /// tried it took the game down on the next frame.
    ///
    /// What was written was `[0x17ce8b4] = 1`, which is what the game's own give-up writes (0x43a366 and
    /// 0x43a498, both inside the run manager's update). **But both of those return 3 as well**, and 3 is
    /// the answer that makes the chain runner stop the walk where it stands — the jump table at
    /// 0x43cb38 sends it to 0x43caf7, which breaks out — so nothing after the manager runs on the frame
    /// a run is given up. orb writes the scene from a recipe step, outside the walk, and cannot answer
    /// for the manager.
    ///
    /// What it cost: `crash: code 0xc0000005 at 0x00406a1a in th08.exe+0x6a1a`, `reading 0x00000004`,
    /// the stack `0x406a1a <- 0x404a67 <- 0x40234e <- 0x43caa1` — the chain runner walking the element
    /// the game adds at priority 1, whose update is 0x402200 on the text object `0x4cce20`. That element
    /// is registered once at startup and never taken off, so on the frame after the teardown it read
    /// something the teardown had freed.
    ///
    /// **And the reading since made says what the way out is instead**: bits 5 and 6 of [`G_RUN_KIND`],
    /// which is the word the run manager's own ending section at 0x439bc7 reads first (0x439be4) —
    /// nothing at all happens while they are 0, a 2 starts the ending and a 1 or a 3 goes straight to
    /// the hand-over. The scene that hand-over gives the supervisor comes out of `[gm+0x3db9c]`, which
    /// is `-1` while a run goes on, and 0x439fb6 is the one place that copies it across. `asm.md`'s
    /// *How a run of th08 ends* has the four scenes the game's own endings put there and which press the
    /// hand-over waits for.
    ///
    /// **So what orb writes is the hand-over and not the ending**: [`SCENE_THE_RUN_WANTS`] takes the
    /// front end's own scene, and the two bits take [`RUN_HANDS_ITSELF_OVER`]. Nothing of the teardown is
    /// orb's: the manager clears the bits itself, gives the supervisor the scene and lets the chain's
    /// walk finish, which is the whole of what the first attempt did wrong.
    ///
    /// **And the press it waits for is the one the retry menu's own answer is.** 0x439f3c wants
    /// `[0x164d528] & 0x1001` to have changed since `[0x164d530]` — the two bits
    /// [`Th08::menu_decide`] is — so the frames after this are frames a press has to arrive on, which is
    /// what the answer to the retry menu already is.
    ///
    /// **Measured on the real game**: a launch of `the-way-out-of-a-run.yaml` read
    /// `retry: the run is given up; the game is on its way to the title` on orb frame 900 with
    /// `scene=2->2`, and `scene=1->1` with `front end: screen 0, cursor on item 0` on frame **902** —
    /// two frames, the title menu, and **no `crash:` line anywhere in the launch**.
    unsafe fn leave_run(&self) -> bool {
        unsafe { mem::write::<i32>(G_GAME_MANAGER + SCENE_THE_RUN_WANTS, SCENE_FRONT_END) };
        let kind = unsafe { mem::read::<u32>(G_RUN_KIND) };
        unsafe { mem::write::<u32>(G_RUN_KIND, (kind & !RUN_IS_ENDING) | RUN_HANDS_ITSELF_OVER) };
        true
    }

    /// **True: the hand-over 0x439f3c makes waits for a press**, the two bits of [`MENU_DECIDE`] down in
    /// `[0x164d528]` and not the same as last frame's `[0x164d530]` — and hands over with nothing pressed only
    /// where the run plays a recording (bit 3 of [`G_RUN_KIND`], 0x439f69) or its stage word is 6, 7 or 8
    /// (0x439f78 through 0x439f91). So a `leave` a recipe answered outright, after the rules had been holding
    /// the shot, left a run that played on: every case of this game that gives a run up read `retry: the run
    /// is given up; the game is on its way to the title` on the laid-out game and never left the stage, and
    /// a watched attempt's own end gave the run up two hundred frames running.
    fn a_run_given_up_waits_for_a_press(&self) -> bool {
        true
    }

    /// Every button held, in the one word this game's screens compare against last frame's — and last
    /// frame's filled with it, so that nothing a key is still down for reads as a new press.
    ///
    /// **Both, where th07 writes one.** th07 can fill the read alone because its screens compare against
    /// a copy the read itself keeps; here [`G_INPUT_LAST`] is written by the supervisor before the read,
    /// so a word of all ones left in [`G_INPUT_READ`] alone would be a press on the very next frame.
    unsafe fn swallow_input(&self) {
        unsafe {
            mem::write::<u16>(G_INPUT_READ, u16::MAX);
            mem::write::<u16>(G_INPUT_LAST, u16::MAX);
        }
    }

    /// Nothing: this game's own read is a word and orb answers it with one — see [`INPUT_READ`].
    unsafe fn press_into(&self, _state: *mut u8, _word: u16) {}

    fn menu_decide(&self) -> u16 {
        MENU_DECIDE
    }

    fn menu_cancel(&self) -> u16 {
        MENU_CANCEL
    }

    /// **What the name entry of the screen a run ends at holds** — see [`RESULT_TYPING_A_NAME`] — read in
    /// whichever phase that screen is in, the field being the same field in all of them: what a replay kept
    /// at the offer after it is named by is what the entry was left holding.
    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        let (screen, _) = unsafe { self.the_screen_a_run_ends_at() }?;
        crate::game::the_name_held_at(screen + RESULT_NAME, RESULT_NAME_BYTES)
    }

    /// **Into the field the entry's own keys write**, with the count typed at the full eight and the cursor
    /// on the item that ends the entry — the state 0x4567c9 leaves it in once somebody has typed eight, so
    /// the press after it leaves the entry the way theirs would. Only while the entry is up.
    unsafe fn types_its_name(&self, name: &str) -> bool {
        let Some(screen) = (unsafe { self.the_entry_a_name_is_typed_into() }) else {
            return false;
        };
        if !crate::game::the_name_typed_into(screen + RESULT_NAME, RESULT_NAME_BYTES, name) {
            return false;
        }
        unsafe {
            mem::write::<i32>(screen + RESULT_TYPED, RESULT_NAME_BYTES as i32);
            mem::write::<i32>(screen + RESULT_CURSOR, RESULT_THE_END_ITEM);
        }
        true
    }

    /// Whether that entry is up at all — see [`Th08::the_entry_a_name_is_typed_into`].
    unsafe fn typing_a_name(&self) -> bool {
        unsafe { self.the_entry_a_name_is_typed_into() }.is_some()
    }

    /// **The screen taken to its own way out** — [`RESULT_LEAVING`] with the timer at nothing, which is
    /// what the offer's own ways out write — and only from the phases the offer is in: a Pointdevice run's
    /// replay is orb's, and the game's would be the one path that survived with none of the attempts
    /// that did not.
    unsafe fn skip_replay_prompt(&self) -> bool {
        let Some((screen, phase)) = (unsafe { self.the_screen_a_run_ends_at() }) else {
            return false;
        };
        if !RESULT_OFFERING_A_REPLAY.contains(&phase) {
            return false;
        }
        unsafe {
            mem::write::<i32>(screen + RESULT_TIMER, 0);
            mem::write::<i32>(screen + RESULT_PHASE, RESULT_LEAVING);
        }
        log!("replay: the offer to save one was taken off the screen a run ended on");
        true
    }

    /// **The front end asked for, which is the one thing that screen's own transition takes**: the
    /// supervisor's wanted scene at [`SCENE_FRONT_END`], out of the case at 0x445a1c — see
    /// [`SCENE_RUN_OVER`], whose doc has the two answers that case has and what each does.
    ///
    /// The screen's own teardown runs through 0x4531f0 on the way, which is what writes the file, so this
    /// asks the screen to leave rather than stepping round it.
    unsafe fn leave_the_screen_a_run_ends_at(&self) -> bool {
        unsafe { mem::write::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED, SCENE_FRONT_END) };
        true
    }

    unsafe fn the_screen_a_run_ends_at_is_up(&self) -> bool {
        unsafe { self.scenes() }.0 == SCENE_RUN_OVER
    }

    /// The frames the stage has run, counted from its build as `read_state`'s are — see
    /// [`STAGE_BEGAN_AT`] — and nothing off a stage.
    ///
    /// **Not [`STAGE_FRAMES`] as it stands**, which is the run's own clock and carries across a build: this is
    /// the frame a read's buttons are written down against, and a chapter is written at the stage's own count,
    /// so a record counted off the run's held nothing of a stage after the first under the chapter's frames.
    /// A whole Normal run wrote `20585 frame(s) of buttons and no generator for any of them` for a chapter of
    /// stage 2, and nothing at all from stage 6 on, where the run's clock was past the `108,000` a record is
    /// bounded by: `resume: not written — 0 frame(s) of buttons for a chapter at 612`.
    unsafe fn stage_frame(&self) -> Option<u32> {
        let (scene, _) = unsafe { self.scenes() };
        (scene == SCENE_RUN || scene == SCENE_RUN_AGAIN).then(|| {
            unsafe { mem::read::<u32>(G_GAME_MANAGER + STAGE_FRAMES) }
                .wrapping_sub(STAGE_BEGAN_AT.load(Ordering::Relaxed))
        })
    }

    /// The generator's whole state, which is one word — see [`G_RANDOM`].
    unsafe fn generator(&self) -> u16 {
        unsafe { mem::read(G_RANDOM) }
    }

    fn run_input(&self) -> u16 {
        RUN_INPUT
    }

    /// **The word the read produced put where the run reads it, with the word it acted on moved along beside
    /// it** — which is what the run's replay does at 0x452332 and 0x452340, made to happen at the read instead
    /// of at chain priority 0x11. See [`G_INPUT_THE_RUN_ACTS_ON`].
    ///
    /// **Nothing while a recording is played**, a replay's or the demo's: what the run acts on there is the
    /// recording, which the playback's own element writes into this word out of its entry (0x4525aa through
    /// 0x4525c1). A live keyboard put there would be the player playing over the playback for the priorities
    /// between the read and that copy.
    ///
    /// The element still copies once the player's job has run, and copies the same values, so what it leaves
    /// as the word acted on is this frame's — which is what the next read moves along before it writes the
    /// other.
    unsafe fn hand_the_read_to_the_run(&self, word: u16, moved_on: bool) {
        if unsafe { mem::read::<u32>(G_RUN_KIND) } & RUN_READS_ITS_OWN_INPUT != 0 {
            return;
        }
        unsafe {
            if moved_on {
                let acts_on = mem::read::<u16>(G_INPUT_THE_RUN_ACTS_ON);
                mem::write::<u16>(G_INPUT_THE_RUN_ACTED_ON, acts_on);
            }
            mem::write::<u16>(G_INPUT_THE_RUN_ACTS_ON, word);
        }
    }

    /// The difficulty, whose run it is and the stage, all three read where this game keeps them.
    ///
    /// **`character` is the whole index and `shot_type` which of the pair is out**, which is what
    /// [`G_CHARACTER`] and [`G_SHOT_TYPE`] hold: a run of this game is a team of two rather than a
    /// character with a shot, so the pair the trait carries is not `character * 2 + shot` here and
    /// nothing multiplies them together.
    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: unsafe { mem::read(G_GAME_MANAGER + DIFFICULTY) },
            character: i32::from(unsafe { mem::read::<u8>(G_CHARACTER) }),
            shot_type: i32::from(unsafe { mem::read::<u8>(G_SHOT_TYPE) }),
            // The same two bits `read_state`'s own practice is read off.
            practice: {
                let kind = unsafe { mem::read::<u32>(G_RUN_KIND) };
                kind & RUN_STARTS_WITH_EIGHT != 0 && kind & RUN_IS_A_SPELL_PRACTICE == 0
            },
            stage: unsafe { mem::read(G_STAGE) },
            mode: 0,
        }
    }

    /// **The run the team select is pointing at**, which is the last screen a whole run of this game is
    /// decided by — see [`SCREEN_TEAM`], and [`SCREEN_TEAM_OF_THE_EXTRA`], the same handler a flow
    /// along.
    ///
    /// The character is the cursor itself: the decide there copies `[front+0x0]` straight into
    /// [`G_CHARACTER`] (0x46ab6f) and clears [`G_SHOT_TYPE`] beside it, so what the cursor sits on *is*
    /// whose run it would be. The difficulty is [`CFG_DIFFICULTY`], which the screen before wrote.
    ///
    /// **Nothing on the two screens that share the handler**, 9 and 0xf: their own decide goes to a
    /// stage select instead of asking for a run, so the run they point at is one stage of it and which
    /// stage is not chosen yet — which is the `practice` the contract declines.
    ///
    /// And nothing where the front end is not up or settled, the cursor then being the last screen's.
    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        if !unsafe { self.the_front_end_is_settled() } {
            return None;
        }
        let front = unsafe { self.front_end() }?;
        let screen = mem::read_committed::<i32>(front + FRONT_END_SCREEN)?;
        if screen != SCREEN_TEAM && screen != SCREEN_TEAM_OF_THE_EXTRA {
            return None;
        }
        Some(RunStart {
            difficulty: i32::from(mem::read_committed::<u8>(CFG_DIFFICULTY)?),
            character: mem::read_committed::<i32>(front + FRONT_END_CURSOR)?,
            // Cleared beside the character at 0x46ab76, a team's own two being the character's index.
            shot_type: 0,
            practice: false,
            // A whole run's chapter can be in any of its stages, and which one is what the mark says
            // rather than part of naming the run — the answer th06 and th07 both give here.
            stage: 0,
            mode: 0,
        })
    }

    /// **One slot per run there can be one of, named by the two numbers that make it that run**: the
    /// difficulty and whose run it is.
    ///
    /// **Both names are read off this exe rather than suggested by a layout**, which is what th07's own
    /// `run_slot` refuses to do and what this game lets it do: the difficulty's table at `0x4c7fd4` holds
    /// five pointers, `Easy` at `0x4b704c` down to `Extra` at `0x4b702c`, and it is read at 0x45489e and
    /// 0x4549df with the difficulty as the index; the shot's table at `0x4c7ce0` holds sixteen pointers to
    /// the `.sht` a team member is loaded out of, `ply00a.sht` through `ply03as.sht`, and it is indexed by
    /// [`G_CHARACTER`] — so the tie between the number and the file is the game's own.
    ///
    /// **Named by the `.sht`'s own stem and not by a person's name.** Which team member each index is
    /// would be the same leap th07 declines: `ply00as` is the second of team 0 and nothing in this exe
    /// says which of the two that is. The stem is what the game itself calls it.
    ///
    /// **And the index goes in the name with it, because that table has duplicates**: 0 and 4 both point
    /// at `ply00a.sht`, and 5 and 12 both at `ply00as.sht`. A name of the stem alone would therefore be a
    /// file two different runs share, which is the one thing a slot must not be — so the number the game
    /// indexes by is what makes it unique and the stem is what makes it readable.
    ///
    /// The shot within the pair is not in it, [`G_CHARACTER`] being the whole index already — see its
    /// doc, where the demo's own start splits one byte into both.
    fn run_slot(&self, run: &RunStart) -> Option<String> {
        // **And none for a stage practised on its own**, which is a run and not a run of the game: there is
        // nothing of it to pick up, the stage being the whole of it.
        if run.practice {
            return None;
        }
        /// The five names the table at `0x4c7fd4` points at, in the order [`DIFFICULTY`] numbers them.
        const DIFFICULTIES: [&str; 5] = ["easy", "normal", "hard", "lunatic", "extra"];
        /// And the sixteen the table at `0x4c7ce0` points at, with `.sht` taken off: `0x4b6fc8`,
        /// `0x4b6fbc`, `0x4b6fb0`, `0x4b6fa4` for the four whole teams, and then the eight whose second
        /// entry is the `s` one, twice over.
        const SHOTS: [&str; 16] = [
            "ply00a", "ply01a", "ply02a", "ply03a", "ply00a", "ply00as", "ply01a", "ply01as",
            "ply02a", "ply02as", "ply03a", "ply03as", "ply00as", "ply01as", "ply02as", "ply03as",
        ];

        let difficulty = usize::try_from(run.difficulty)
            .ok()
            .and_then(|at| DIFFICULTIES.get(at));
        let shot = usize::try_from(run.character)
            .ok()
            .and_then(|at| SHOTS.get(at));
        // Numbered where either is not one of this game's, a name being the one thing this cannot make
        // up — and the file has to be some run's either way. Which is th06's and th07's own answer here.
        Some(match (difficulty, shot) {
            (Some(difficulty), Some(shot)) => {
                format!("{difficulty}-{}-{shot}", run.character)
            }
            _ => format!("difficulty{}-character{}", run.difficulty, run.character),
        })
    }

    fn named(&self) -> &'static str {
        "th08"
    }

    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8> {
        resume::encode(saved)
    }

    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept> {
        resume::decode(bytes)
    }

    /// The word the generator is put on before the stage is built, which is the one field of the set that
    /// cannot wait for the rest: building a stage draws from it.
    ///
    /// The game's own restore does the same thing through 0x453be0 — a write of the word and nothing
    /// else — at 0x452f94.
    unsafe fn seed_the_stage(&self, state: &[u8]) {
        let Some(state) = resume::decode_state(state) else {
            return log!("resume: the seed put back is not a run of th08's");
        };
        unsafe { self.seed(state.seed) };
    }

    fn say_run_state(&self, state: &[u8]) -> String {
        match resume::decode_state(state) {
            Some(numbers) => format!("{numbers:?}"),
            None => "numbers that are not a run of th08's".to_owned(),
        }
    }

    unsafe fn run_state(&self) -> Vec<u8> {
        resume::encode_state(&unsafe { self.numbers() })
    }

    unsafe fn set_run_state(&self, state: &[u8]) {
        let Some(state) = resume::decode_state(state) else {
            return log!("resume: the numbers put back are not a run of th08's");
        };
        unsafe { self.put_the_numbers_back(&state) };
    }

    /// Whether the front end has just asked for a run, which the pair of scenes says: 1 going to 2 is
    /// this game starting one.
    ///
    /// **And never of a recording played**, the demo or a replay off the game's own list, which ask for a
    /// run exactly the same way and have nobody to ask a question of: both have set bit 3 of
    /// [`G_RUN_KIND`] by then, and every screen that starts a run somebody plays has cleared it.
    unsafe fn run_chosen(&self) -> bool {
        let (scene, wanted) = unsafe { self.scenes() };
        let kind: u32 = unsafe { mem::read(G_RUN_KIND) };
        scene == SCENE_FRONT_END && wanted == SCENE_RUN && kind & RUN_READS_ITS_OWN_INPUT == 0
    }

    /// The stage a run is to start at, written where both of this game's loaders read it — see
    /// [`G_STAGE`], which this game already counts from nothing.
    unsafe fn start_stage(&self, stage: i32) -> bool {
        if !(0..STAGES).contains(&stage) {
            return false;
        }
        unsafe { mem::write::<i32>(G_STAGE, stage) };
        true
    }

    /// **A run asked for the way this game's own demo asks for one**: the difficulty, the team and which
    /// of its two, the stage, the kind of run cleared, and then the scene.
    ///
    /// Read off 0x467ba7 through 0x467c34, which is the one place in the exe that sets a run up from
    /// numbers rather than from a screen — the demo taking them out of a replay's header. What is
    /// cleared rather than written is the kind: a run started here is not a playback and not a spell
    /// practice, and the two bits the title's own items set are the two this has to put out.
    unsafe fn start_a_run(&self, run: &RunStart) -> bool {
        if !(0..STAGES).contains(&run.stage) {
            log!("replay: there is no stage {} of th08", run.stage + 1);
            return false;
        }
        if !(0..DIFFICULTIES).contains(&run.difficulty) {
            log!("replay: there is no difficulty {} of th08", run.difficulty);
            return false;
        }
        let Ok(character) = u8::try_from(run.character) else {
            log!("replay: there is no character {} of th08", run.character);
            return false;
        };
        let Ok(shot) = u8::try_from(run.shot_type) else {
            log!("replay: there is no shot {} of th08", run.shot_type);
            return false;
        };
        let kind = unsafe { mem::read::<u32>(G_RUN_KIND) };
        unsafe {
            mem::write::<i32>(G_GAME_MANAGER + DIFFICULTY, run.difficulty);
            mem::write::<u8>(G_CHARACTER, character);
            mem::write::<u8>(G_SHOT_TYPE, shot);
            mem::write::<u32>(
                G_RUN_KIND,
                kind & !(RUN_READS_ITS_OWN_INPUT | RUN_IS_A_SPELL_PRACTICE | RUN_STARTS_WITH_EIGHT),
            );
            mem::write::<i32>(G_STAGE, run.stage);
            mem::write::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED, SCENE_RUN);
        }
        true
    }

    /// **The team select put up on the run's own team, in the phase its decide is read in** — screen 0xd's
    /// for the Extra, whose decide makes the Extra's stage out of the difficulty (0x46ac08) — so that the
    /// press handed over next is the one that starts the run. See [`Game::front_end_starts_a_run`] for why
    /// a run is asked for this way.
    ///
    /// **Because a run [`Th08::start_a_run`] writes into being leaves the front end running under it.** What
    /// takes the front end off the chain is its own update answering nought, which the team select does
    /// straight after it asks for the scene (0x46ac15 through 0x46ac47) — and a scene written from outside
    /// answers nothing. So an attempt watched out of orb's list, which is asked for over the title, played
    /// with the title still counting its own idle updates (0x467a9c) and starting the attract demo over the
    /// run at `0x5dc` of them (0x467ac4): a launch read `diff=0` 133 frames into the watch, the demo's own
    /// difficulty out of its replay's header (0x467ba7), and the game gone with the calc list empty 150
    /// frames after that — twice, the same way.
    ///
    /// Written the way the title's Start item and the difficulty select leave what they write: bits 0 and
    /// 14 of the run kind out (0x467d23 through 0x467d3a) and the difficulty in [`CFG_DIFFICULTY`], which
    /// the team select copies into the run's own. The stage its decide writes is the first, and a watch puts
    /// its own in on the frame the run is registered — see `resume::the_stage_of_a_watch`.
    unsafe fn front_end_starts_a_run(&self, run: &RunStart) -> bool {
        if run.practice || !(0..DIFFICULTIES).contains(&run.difficulty) {
            return false;
        }
        let Ok(character) = u8::try_from(run.character) else {
            return false;
        };
        let Some(front) = (unsafe { self.front_end() }) else {
            return false;
        };
        let screen = if run.difficulty == DIFFICULTY_EXTRA {
            SCREEN_TEAM_OF_THE_EXTRA
        } else {
            SCREEN_TEAM
        };
        let kind = unsafe { mem::read::<u32>(G_RUN_KIND) };
        unsafe {
            mem::write::<u32>(
                G_RUN_KIND,
                kind & !(RUN_IS_A_SPELL_PRACTICE | RUN_STARTS_WITH_EIGHT),
            );
            mem::write::<u8>(CFG_DIFFICULTY, run.difficulty as u8);
            mem::write::<u8>(G_CHARACTER, character);
            mem::write::<i32>(front + FRONT_END_SCREEN, screen);
            mem::write::<i32>(front + FRONT_END_CURSOR, i32::from(character));
            mem::write::<i32>(front + FRONT_END_PHASE, TEAM_PHASE_CHOOSING);
            mem::write::<i32>(front + FRONT_END_SETTLED, TEAM_SETTLES_AT + 1);
        }
        true
    }

    /// **The game on the screen a run ends at**, which is [`SCENE_RUN_OVER`] — and the same read as
    /// [`Th08::the_screen_a_run_ends_at_is_up`], this game's finished run and the screen it is finished
    /// on being one scene.
    ///
    /// **Which is what tells a run that ended from a run given up**: [`Th08::leave_run`] hands the run
    /// straight to the front end and never reaches 6, so a scene of 6 is a run that played itself out.
    /// Measured: a launch read `scene=6->6` on the frame the third life went.
    unsafe fn run_finished(&self) -> bool {
        unsafe { self.scenes() }.0 == SCENE_RUN_OVER
    }

    /// **What one pass of a stage of this game is held against another's**, which is the run's own
    /// numbers said by name.
    ///
    /// Every field here is one the game itself draws or writes down — see the constants, each of which
    /// carries the panel row or the replay header line it was named by — with the generator and the
    /// stage's frame count beside them. What is *not* here is the player and the enemies, which th07's
    /// own reproduction carries and which nothing of this game has been read of.
    unsafe fn reproduction(&self) -> String {
        let Some(run) = (unsafe { self.run_numbers() }) else {
            return "no run of th08 is built".to_owned();
        };
        let whole = |at: usize| unsafe { mem::read::<f32>(run + at) }.trunc();
        let word = |at: usize| unsafe { mem::read::<u32>(run + at) };
        let digit = unsafe { mem::read::<u8>(run + SCORE_LAST_DIGIT) };
        // **And what the card that is up would pay in time orbs, with the two times its own ramp is on** —
        // see [`CARD_TIME_ORBS`], which is the number this game's Last Spell is decided by and which the
        // count above never holds. Read whether or not a card is up: the object is a static one the game
        // reuses, so what stands here between cards is the last capture's, which is the number to hold a
        // fight against.
        let card = |at: usize| mem::read_committed::<i32>(G_SPELL_CARD + at).unwrap_or(-1);
        format!(
            "frames={} seed={:#06x} stage_seed={:#06x} score={}{} shown={}{} graze={} \
             points={}/{} time_orbs={}/{} card_orbs={} card_left={}/{} gauge={} lives={} bombs={} \
             power={} misses={} bombed={}",
            // **The stage's own count, as [`Th08::stage_frame`] counts it**, and not the run's clock: a run
            // picked up in a stage after its first is built as a run's first, with the clock from nothing, so
            // the run's clock is the one field no such landing could match: a pick-up in stage 7 of the laid-out
            // game read `frames=26281 against frames=3781` and agreed on every other field.
            unsafe { mem::read::<u32>(G_GAME_MANAGER + STAGE_FRAMES) }
                .wrapping_sub(STAGE_BEGAN_AT.load(Ordering::Relaxed)),
            unsafe { mem::read::<u16>(G_RANDOM) },
            unsafe { mem::read::<u16>(G_GAME_MANAGER + SEED_AT_STAGE_START) },
            word(SCORE),
            digit,
            word(SCORE_SHOWN),
            digit,
            word(GRAZE),
            word(POINT_ITEMS),
            word(POINT_ITEMS_WANTED),
            word(TIME_ORBS),
            word(TIME_ORBS_WANTED),
            card(CARD_TIME_ORBS),
            card(CARD_TIME_LEFT),
            card(CARD_TIME),
            unsafe { mem::read::<i16>(run + GAUGE) },
            whole(LIVES),
            whole(BOMBS),
            whole(POWER),
            whole(DEATHS),
            whole(BOMBS_SPENT),
        ) + &unsafe { self.the_world_said() }
    }

    /// The frames the supervisor's loader is running on — see [`SUPERVISOR_LOADING`] — which are the
    /// frames a run's block is allocated and not yet written, and the reproduction line reads bytes
    /// nothing has set.
    unsafe fn mid_build(&self) -> bool {
        unsafe { self.is_loading() }
    }

    /// **The first frame the loader's word reads clear after it read set, in a run's scene.**
    ///
    /// **Where a run of this game is built is a thread and not the run manager's own added callback.**
    /// 0x43aa5c is the registration — it writes 0x439bc7 as the element's update, `0x43aaf4` as its added
    /// callback and 0x43be2c as its deleted one into `0x164d344`, hangs the game manager off it and adds
    /// it to the chain at priority 2 — and the added callback, run by the chain on the spot, is forty
    /// instructions: it branches on the wanted scene at 0x43aafd — 3, 0xb and 0xc being a stage that
    /// carries a run on and anything else a run starting — sets the loading screen up through 0x44886f,
    /// and hands **`0x43abd7`** to 0x4486ba, which is `CreateThread`. It returns at 0x43abd6 with nothing
    /// of the stage in place.
    ///
    /// **`0x43abd7`–0x43b935 is the build**, on that thread: it frees the last run's two blocks and
    /// allocates this one's (0x43ad33, 0x43ad40, 0x43ad57), copies the configuration in, puts the
    /// starting lives in at 0x43ae42 on the run-start branch, builds the player through 0x44c230, copies
    /// the seed at 0x43b3cb and calls [`STAGE_SEEDED`] three instructions later. So `Hooks::stage_begun`
    /// has no function to stand over here — the thread's return is not the main thread's, and the added
    /// callback's return is before any of it — and the moment is read off [`SUPERVISOR_LOADING`]: orb walks
    /// the supervisor alone on a frame the word is set — see [`Th08::walks_a_frame_of_its_build`] — so the
    /// first frame it reads clear is one no job of the run has run on.
    ///
    /// **A launch is what read this apart from the callback.** A patch stood over 0x43aaf4 and the record
    /// taken at its return held `score: 107981937, lives: -128, deaths: -2147483648`, the same bytes on two
    /// launches: an allocation nothing had written yet. The reproduction line of the run's own frame 0
    /// read the same rubbish and frame 1 read the run — the frame the thread had finished by.
    ///
    /// **In a run's scene**, because the front end and the endings load through the same thread, and a
    /// build of theirs is no stage of anybody's.
    /// **The line up written away**, which is the only way a playback of this game gets past a
    /// conversation inside a stage: nothing of the run advances while one is up — a playback held at one
    /// ran 76,000 updates with `script=4962 seed=0x12c5`, both standing — so no press it feeds reaches
    /// anything. See [`Th08::end_the_line`].
    unsafe fn give_up_on_a_line_of_dialogue(&self) {
        unsafe { self.end_the_line() };
    }

    unsafe fn a_stage_has_just_been_built(&self) -> bool {
        static WAS_LOADING: AtomicBool = AtomicBool::new(false);
        let loading = unsafe { self.is_loading() };
        let was = WAS_LOADING.swap(loading, Ordering::Relaxed);
        let (scene, _) = unsafe { self.scenes() };
        let a_run = scene == SCENE_RUN || scene == SCENE_RUN_AGAIN;
        A_BUILD_WAS_GOING_ON.store(loading && a_run, Ordering::Relaxed);
        let built = was && !loading && a_run;
        // And where the stage's own frames start counting from — see [`STAGE_BEGAN_AT`]: the run's clock
        // as the stage's numbers came into place, which a run's first stage has at nought and every stage
        // after at whatever the stages before it counted to.
        if built {
            STAGE_BEGAN_AT.store(
                unsafe { mem::read::<u32>(G_GAME_MANAGER + STAGE_FRAMES) },
                Ordering::Relaxed,
            );
        }
        built
    }

    /// **The supervisor's update alone, on every frame of a run's build that the build has stood out** — the
    /// [`BUILD_FRAMES`] it waits for counted — up to the one orb reads it over on. The build as the look
    /// [`Th08::a_stage_has_just_been_built`] took it, **and on every update after one that started a build**,
    /// the word being set now where the look saw it clear: the main thread sets it (0x4486fb, inside the walk)
    /// and only the thread clears it, so the two together hold every update from the one after the build was
    /// asked for to the frame orb reads it over on.
    ///
    /// **Because those are frames the stage runs on, and before anything of orb's is back in.** The
    /// supervisor's answer while [`SUPERVISOR_LOADING`] is set is 1 at 0x4455bc, and the walk (0x43ca50) takes
    /// 1 to the next job (0x43cb0e); the thread has the stage's jobs on the chain from straight after the seed —
    /// the bullets' update 0x431240 at priority 0xe through 0x4311a0 (0x43b414), the enemies' 0x42c660 at 0xb
    /// through 0x42c590 (0x43b441) — and its last act is to wait out [`BUILD_FRAMES`] seventeen milliseconds a
    /// sleep. So the frames between that count coming up and the thread waking to clear its word are frames of
    /// the stage, as many as the main thread fits into a sleep: three pick-ups of one file at `--speed=4` began
    /// their stage at `frames=5` with `script=5`, `7` and `8`, and the two ahead read `the generator is 0xeede
    /// where the run had 0xe08d` on the playback's first frame.
    ///
    /// **And the walk as ever before the count is up, and the supervisor rather than nothing after it, because
    /// the build waits on both.** Walking nothing stood the build still for good — `resume: given up — the
    /// stage never came up` — and the thread's stack said where: the player's added callback, run on the thread
    /// as 0x44c230 puts the player's job up, asks the object at `[0x18bdc90]` for a file through 0x465a40 and
    /// sleeps a millisecond at a time (0x465a82) until the entry's `+0x18` goes back to nought, and what puts it
    /// back is 0x466000 — every pending entry of `0x100` handed to 0x465ba0, which clears it at 0x465c3a —
    /// called at 0x445570 of the supervisor's update, before the loader's word is read. And walking the
    /// supervisor alone from the start stood it still at its last wait, the run manager's step of
    /// [`BUILD_FRAMES`] being a job behind it. What the supervisor answers is handed on the way the walk would:
    /// 4 is a walk stopped answering nought, 5 answering -1, and every other is the frame carried on.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn walks_a_frame_of_its_build(&self) -> Option<i32> {
        let (scene, _) = unsafe { self.scenes() };
        let building = A_BUILD_WAS_GOING_ON.load(Ordering::Relaxed)
            || (unsafe { self.is_loading() } && (scene == SCENE_RUN || scene == SCENE_RUN_AGAIN));
        if !building {
            return None;
        }
        let stood_out = match unsafe { mem::read::<u32>(G_RUN_KIND) } & RUN_READS_ITS_OWN_INPUT {
            0 => A_BUILD_STANDS_FOR,
            _ => A_RECORDINGS_BUILD_STANDS_FOR,
        };
        if unsafe { mem::read::<i32>(G_GAME_MANAGER + BUILD_FRAMES) } < stood_out {
            return None;
        }
        let update: unsafe extern "thiscall" fn(usize) -> i32 =
            unsafe { std::mem::transmute(supervisor_update()) };
        Some(match unsafe { update(G_SUPERVISOR) } {
            4 => crate::runtime::CHAIN_EXIT_SUCCESS,
            5 => crate::runtime::CHAIN_EXIT_ERROR,
            _ => crate::runtime::CHAIN_BREAK,
        })
    }

    /// Where the title menu's cursor is, for a recipe that waits on an item before pressing at it.
    unsafe fn front_end_cursor(&self) -> Option<i32> {
        let front = unsafe { self.front_end() }?;
        mem::read_committed::<i32>(front + FRONT_END_CURSOR)
    }

    /// And which screen of the front end is up — see [`FRONT_END_SCREEN`] for the eighteen the game's
    /// own switch covers.
    unsafe fn front_end_screen(&self) -> Option<i32> {
        let front = unsafe { self.front_end() }?;
        mem::read_committed::<i32>(front + FRONT_END_SCREEN)
    }

    /// **The buttons a recipe names, which is the whole of how a launch of this game is driven.**
    ///
    /// Every bit here was read off the fold at [`INPUT_READ`] — see `asm.md` for the key each one comes
    /// from — and the two masks are the ones every screen of the front end tests, see
    /// [`Th08::menu_decide`].
    ///
    /// `menu` is in the list because the game's own pause menu is behind it and 「最初からやり直す」 is
    /// behind that, which a recipe can reach no other way. Its items are the game's own sprites, so a
    /// recipe presses its way down them blind.
    fn button_bit(&self, name: &str) -> Option<u16> {
        Some(match name {
            "shot" => BUTTON_SHOT,
            "bomb" => BUTTON_BOMB,
            "focus" => BUTTON_FOCUS,
            "skip" => BUTTON_SKIP,
            "up" => PAD_UP,
            "down" => PAD_DOWN,
            "left" => PAD_LEFT,
            "right" => PAD_RIGHT,
            "decide" => MENU_DECIDE,
            "cancel" => MENU_CANCEL,
            "menu" => BUTTON_MENU,
            _ => return None,
        })
    }

    /// **Every job on the chain's calc list**, as the callback each is and the object it is called on —
    /// see [`G_CHAIN`], whose calc head is `+0x0`, and *The chain* in `asm.md` for the element's own
    /// three fields.
    ///
    /// **Bounded at [`CHAIN_JOBS_AT_MOST`]**, which is what every walk orb makes of a game's own list is:
    /// a next pointer read wrong is a walk that never ends, and a frame that never ends is a game that
    /// hangs. Read committed for the same reason, the chain and its elements being the game's own
    /// allocations.
    unsafe fn the_jobs_a_frame_has(&self) -> Option<String> {
        let mut jobs = String::new();
        // From the element after the chain's own, which is the list's first and has no callback — see
        // [`CHAIN_CALC_LIST`].
        let mut element =
            mem::read_committed::<usize>(G_CHAIN + CHAIN_CALC_LIST + CHAIN_JOB_NEXT).unwrap_or(0);
        let mut counted = 0;
        while element != 0 && counted < CHAIN_JOBS_AT_MOST {
            let callback = mem::read_committed::<usize>(element + CHAIN_JOB_CALLBACK).unwrap_or(0);
            let on = mem::read_committed::<usize>(element + CHAIN_JOB_ARGUMENT).unwrap_or(0);
            jobs.push_str(&format!(" {callback:#010x}({on:#010x})"));
            element = mem::read_committed::<usize>(element + CHAIN_JOB_NEXT).unwrap_or(0);
            counted += 1;
        }
        Some(format!("{counted} on the calc list:{jobs}"))
    }

    /// **The run's own numbers as bytes, and the word that says what kind of run it is** — which is what
    /// the next reading of this game is made from rather than anything orb acts on.
    ///
    /// The 0xe4 bytes at [`G_RUN_NUMBERS`] are barely placed, and a launch that prints them whole is one
    /// two entries into a stage can be held against each other out of: a field that moved between two
    /// passes over the same wave is a field worth naming. See th07's answer to this, which is the same
    /// dump for the same purpose.
    unsafe fn the_runs_own_bytes(&self) -> Option<String> {
        let said = match unsafe { self.run_numbers() } {
            Some(run) => match mem::read_committed_bytes(run, RUN_NUMBER_BYTES) {
                Some(bytes) => bytes.iter().fold(String::new(), |mut said, byte| {
                    said.push_str(&format!("{byte:02x}"));
                    said
                }),
                None => "unreadable".to_owned(),
            },
            None => "none".to_owned(),
        };
        Some(format!("run={said} kind={:#010x}", unsafe {
            mem::read::<u32>(G_RUN_KIND)
        },))
    }

    /// **One frame of a run as the rules see it** — see [`sight`], whose own doc says what is in it and
    /// what is not.
    unsafe fn what_a_bot_sees(&self, into: &mut crate::game::Frame) -> bool {
        unsafe { sight::what_a_bot_sees(into) }
    }

    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &chapters::MIDSTAGE
    }

    /// The one stage past the numbered ones, which is the ninth row of the table: this game's fourth and
    /// last are each two stages of their own rather than two games' worth of extra.
    fn stages_past_the_numbered(&self) -> &'static [&'static str] {
        &["extra"]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BUTTON_BOMB, BUTTON_FOCUS, BUTTON_MENU, BUTTON_SHOT, BUTTON_SKIP, DIFFICULTIES,
        MENU_CANCEL, MENU_DECIDE, PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, RUN_INPUT, STAGES, Th08,
        chapters,
    };
    use crate::game::{Game, RunStart};

    #[test]
    fn the_midstage_table_has_a_row_for_every_stage_the_game_has() {
        assert_eq!(chapters::MIDSTAGE.len(), STAGES as usize);
        assert_eq!(Th08.midstage_table().len(), STAGES as usize);
    }

    #[test]
    fn the_stages_past_the_numbered_are_named_for_the_rows_past_the_six() {
        let numbered = STAGES as usize - Th08.stages_past_the_numbered().len();
        assert_eq!(numbered, 8);
    }

    #[test]
    fn a_run_of_this_game_is_written_down_without_the_button_that_opens_its_menu() {
        assert_eq!(RUN_INPUT & BUTTON_MENU, 0);
        for button in [BUTTON_SHOT, BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SKIP] {
            assert_eq!(RUN_INPUT & button, button);
        }
        for direction in [PAD_UP, PAD_DOWN, PAD_LEFT, PAD_RIGHT] {
            assert_eq!(RUN_INPUT & direction, direction);
        }
    }

    #[test]
    fn the_two_masks_the_front_end_reads_share_no_button() {
        assert_eq!(MENU_DECIDE & MENU_CANCEL, 0);
        assert_eq!(MENU_DECIDE & BUTTON_SHOT, BUTTON_SHOT);
        assert_eq!(MENU_CANCEL & BUTTON_BOMB, BUTTON_BOMB);
        assert_eq!(MENU_CANCEL & BUTTON_MENU, BUTTON_MENU);
    }

    #[test]
    fn this_game_has_the_five_difficulties_its_own_table_names() {
        assert_eq!(DIFFICULTIES, 5);
    }

    /// **A run of this game is rewound**, which is what the mode is: the two lists a snapshot has to
    /// leave out are read, and a launch of the real game has taken a chapter of stage 1, died in it and
    /// put it back — see [`Th08::rewinds`], whose doc has what that launch said.
    #[test]
    fn a_run_of_this_game_is_rewound() {
        assert!(Th08.rewinds());
    }

    /// **Every run this game can be a run of gets a slot of its own**, which is what keeps two runs out
    /// of one file: the five difficulties against the sixteen indices the shot table has, all eighty
    /// distinct.
    ///
    /// **Which the `.sht` stems alone do not give**, that table pointing twice at four of its eight
    /// files: what this holds is the *slot*, and it is what says the index has to be in the name.
    #[test]
    fn a_run_of_this_game_is_named_by_its_difficulty_and_the_sht_its_shot_is_loaded_from() {
        let named = |difficulty, character| {
            Th08.run_slot(&RunStart {
                difficulty,
                character,
                shot_type: 0,
                practice: false,
                stage: 0,
                mode: 0,
            })
            .expect("every run of this game has a slot")
        };
        assert_eq!(named(1, 0), "normal-0-ply00a");
        assert_eq!(named(4, 11), "extra-11-ply03as");

        // The eighty, all of them different — which is the property a file per run stands on, and the
        // one the table's own duplicates would break.
        let slots: std::collections::BTreeSet<String> = (0..DIFFICULTIES)
            .flat_map(|difficulty| (0..16).map(move |character| named(difficulty, character)))
            .collect();
        assert_eq!(slots.len(), DIFFICULTIES as usize * 16);

        // And a run whose numbers are not this game's is numbered rather than named.
        assert_eq!(named(9, 99), "difficulty9-character99");
    }
}
