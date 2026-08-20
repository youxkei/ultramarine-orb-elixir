//! 東方妖々夢, as a [`Game`] that answers what has been measured and declines the rest.
//!
//! `th07.exe`, 650752 bytes, `md5 0126afce1e805370d36c3482445e98da`. Every address below was read off
//! that exe with `i686-w64-mingw32-objdump -d -M intel` and nothing else: no table anybody published
//! and nothing carried across from 紅魔郷's addresses, which are where the same things happen to live
//! in another game. Each constant says what identifies it, because that is the whole of the evidence
//! there is — there is no decompilation of this build to cross-check against, as [`th06`](super::th06)
//! has.
//!
//! **What is measured is a frame, and a run read, written and put back.** The window, the device, the chain
//! and its two walks, the whole frame and everything it does around its own drawing, the play area, the panel
//! and the size it renders at — every field of [`State`], which is what a run *is* as far as orb asks — and
//! the writes a chapter is made of: a stage started and left, a run's numbers and its seed put back, the
//! ranking asked for and left, the card record held across both, a card's attempt counted and its name baked
//! onto the plate again, the front end's own menus read and answered, and a resumed run's stage begun.
//!
//! **So 妖々夢 is a game orb plays**: [`Th07::rewinds`] answers `true`, and what that rests on is the e2e test
//! over a run of a laid-out one rather than the list above — `crates/orb-e2e/src/a_pointdevice_run_of_th07.rs`.
//! The music is in with the rest, the stream rewound through the file this game opens itself, which is
//! `audio::Track`'s second shape.
//!
//! **What is left is one measurement**: a chapter falls where [`Th07::midstage_table`] says, and that table is
//! read off a run with `--chapter-tuning` rather than off this exe — so it is a stage per entry with nothing in
//! any of them, and a chapter falls at a stage's start and nowhere inside one until such a run happens. One
//! thing is declined on purpose and says why where it is, [`Th07::run_slot`]'s names.
//!
//! See `docs/adr/0004` for the shape, `docs/adr/0017` for the frame, `docs/adr/0018` for which clock a stage's
//! script runs on, `docs/adr/0019` for the face orb's own screens are drawn in here, and
//! `docs/todo/th07-the-run.md` for what a run still wants. A snapshot is the piece this game answers a changed
//! number by sabotaging itself over: see [`DEATHS`] and [`CARD_CHECK`].
//!
//! **Nothing here panics.** An `unimplemented!()` in a method a frame reaches would take somebody's
//! game down at the moment orb ran out of measurements, which is the one thing an injected DLL must
//! not do. Where the trait offers no way to decline, the answer is the emptiest one that is *true* —
//! no regions, no music, no run in progress, no button worth writing down — and no method answers a
//! guess.

/// The midstage table, in a file of its own because a `--chapter-tuning` pass writes one to paste over it.
pub mod chapters;

/// Reachable from this crate's own tests and, through the `sim` feature, from the tests of the
/// crates that drive it — which is where the scenarios live.
#[cfg(any(test, feature = "sim"))]
pub mod image;

use std::ffi::c_void;
use std::ops::Range;
// The slots behind `handed_over!` want these only where a laid-out game can fill them — see the macro — and
// the cursor the ranking is asked for from wants them in every build.
use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::Hwnd;
use orb_api::mem;

use orb_api::{Device, SoundBuffer, Texture, Viewport};

use crate::audio::{Music, Track};
use crate::game::{
    Boundary, Call, Entered, Face, FrameCalls, Game, Hooks, Menu, Pad, PanelTile, Patch, Reading,
    Rect, Reproduction, RunStart, RunState, State,
};
use crate::{detail, log};

pub struct Th07;

/// Which item the title menu's cursor was on when the ranking was asked for, kept one along so that nothing
/// remembered and the first item are told apart.
///
/// A static because the front end that asked is gone by the time it can be put back: asking for the ranking
/// takes it down, and coming back builds another one — see [`Th07::restore_menu_cursor`].
static MENU_CURSOR_BEFORE: AtomicUsize = AtomicUsize::new(0);

/// `IDirect3DDevice8*`, and the `IDirect3D8*` it was made on.
///
/// Both fall out of one call: `IDirect3D8::CreateDevice` at 0x434d94, reached through the interface's
/// own vtable at +0x3c, with its seven arguments pushed from 0x434d70 down. The last pushed is
/// `ppReturnedDeviceInterface`, which is this address, and the `this` it is called on is the other.
///
/// A better witness than either would have been alone: the one call fixes the device, the interface it
/// came from and the window below, so none of the three rests on recognising a global by what is done
/// with it.
const G_D3D_DEVICE: usize = 0x00575958;

/// The game's window, and the object a frame is called on.
///
/// The `hFocusWindow` argument of that same `CreateDevice`, pushed at 0x434d7b — and also the object
/// [`RENDER`] is called on, at `mov ecx,0x575c20` in the message pump. So the handle is that object's
/// first field, as 紅魔郷's window object's is.
const G_GAME_WINDOW: usize = 0x00575c20;

/// `th07::Chain`, the object both walks below are called on.
///
/// The `this` of the two calls at 0x43474d and 0x4347da. What says it is a `Chain` of the shape orb
/// already knows is that the two walks start 0x20 apart — [`RUN_CALC_CHAIN`] from `this` and
/// [`RUN_DRAW_CHAIN`] from `this + 0x20` — which is 紅魔郷's layout arrived at from this exe rather
/// than assumed from that one.
const G_CHAIN: usize = 0x00626218;

/// `th07::Chain::RunCalcChain`, `__thiscall`. One call per logic frame, from [`RENDER`] at 0x4347da,
/// after the drawing.
///
/// Its answer is what the frame's own is made of, which is what identifies it: at 0x4347f1 a zero
/// makes the frame return 1 and at 0x434801 a `-1` makes it return 2, so nothing and a failed walk are
/// the game asking to stop. 紅魔郷's chain answers the same two.
const RUN_CALC_CHAIN: usize = 0x0042fd60;
/// `th07::Chain::RunDrawChain`, `__thiscall`. Called from [`RENDER`] at 0x43474d, between
/// `IDirect3DDevice8::BeginScene` (+0x88, at 0x434728) and `EndScene` (+0x8c, at 0x434788) — which is
/// what makes it the draw, and the only place an overlay may draw.
const RUN_DRAW_CHAIN: usize = 0x0042fe20;
/// `push ebp; mov ebp,esp; sub esp,0x14` — position-independent, 6 bytes, read out of the exe at both
/// addresses. The same prologue as 紅魔郷's two, which is MSVC6 and not a coincidence worth resting
/// anything on.
const RUN_CHAIN_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x14];

/// `th07::SoundPlayer::PlaySounds`, `__thiscall` on [`G_SOUND_PLAYER`]: the queued sound effects
/// handed to the sound system.
///
/// Called from the game's own frame at 0x4347e7, immediately after the calc chain, which is where
/// 紅魔郷 hands its own over. What it does with the object agrees: it answers nothing at once where
/// `[this + 0x610]` is zero and otherwise walks a list at `this + 0x73c`.
const PLAY_SOUNDS: usize = 0x0044c9c0;
const G_SOUND_PLAYER: usize = 0x004ba0d8;

/// The music thread's id, `+0x614` of that player — the same offset 紅魔郷 keeps its own at.
///
/// `CreateThread` at 0x44c113 is given `this + 0x614` as its `lpThreadId` and its answer goes into `+0x618`,
/// the handle; `StopBGM` at 0x44c6b0 posts message 0x12 to the id and waits on the handle. That thread is what
/// tops the streaming buffer up — 0x44d200 waits on an event and calls the refill at 0x45db60 — so it is the
/// one thread a restore of the game's own memory must not have running underneath it.
const MUSIC_THREAD_ID: usize = 0x614;

/// How much of the address space that player is, which is what a restore has to leave alone: 0x39cc bytes.
///
/// Its own constructor says so — 0x44b51a zeroes `0xe73` dwords of it before it fills its 0x80 slot handles
/// with -1 — and the two fields below being inside that is what says they are its own rather than statics
/// beside it.
const SOUND_PLAYER_BYTES: usize = 0xe73 * size_of::<u32>();

/// The queue orb's two music calls put a command in, at `+0x73c` of the player: whether the first slot holds a
/// kind is whether anything is still in flight.
///
/// **A command is walked over several runs of [`PLAY_SOUNDS`] rather than carried out by one.** 0x44c9c0 takes
/// the first slot, switches on its kind, and where that kind's own step is not the last one advances the phase
/// at `+0x8` and returns (0x44ca96) — the same slot again next run. A finished one is dropped by shifting the 31
/// slots down over it (0x44d092, 0x10c bytes apiece) and the run goes on to the next; an empty first slot is a
/// run with nothing to do, and PlaySounds answers with that slot's own kind (0x44d0ee).
///
/// Which is why orb reads it: what a restore does about the sound has to happen where the ask it made has been
/// carried out, and the frame's own sound is not that moment — the stage's build knows it too, looping
/// PlaySounds at 0x42f238 until it answers nothing rather than leaving its music for the next frame. A launch of
/// 妖々夢 acting one frame after the ask heard the stage's song start anyway and read `could not be put` for the
/// seek that was to follow it: both are the same command still sitting in this queue.
const SOUND_COMMANDS: usize = 0x73c;

/// How many runs of that queue one of orb's own asks is given before it is left to the frame's own sound: enough
/// that a command's phases are walked, few enough that a queue which never empties is a wait rather than a hang.
const RUNS_A_COMMAND_IS_GIVEN: u32 = 32;

/// The music's own stream, as a pointer at `+0x39bc` of the player, and how much of it there is.
///
/// `SoundPlayer::PlayBGM` (0x44c020) is where it comes from: it copies the track's name into `+0x38bc`, takes
/// the current one down, works a buffer size out of the file's own format, makes the event at `+0x39c0` and the
/// thread at `+0x614`, and hands all of it to 0x45c8e0 — which allocates 0x78 bytes at 0x45cb93 and writes the
/// object here.
const STREAM: usize = 0x39bc;
const STREAM_BYTES: usize = 0x78;

/// And what the stream is made of, all of it read off the refill at 0x45db60 and the draw of a chunk it makes:
/// the sound buffer it writes into at `+0x4`, that buffer's own size at `+0x8`, the source the bytes come from
/// at `+0xc`, where the next chunk goes at `+0x64`, how much goes in at a time at `+0x6c`, and a byte saying a
/// refill is in flight at `+0x74`.
///
/// `+0x4` is a wrapper and not the buffer itself: the refill reads `[[stream+0x4]]` and calls
/// `IDirectSoundBuffer::GetCurrentPosition` through it at 0x45dba3, which is what says the first field of that
/// wrapper is the buffer DirectSound made.
const STREAM_BUFFER: usize = 0x04;
const STREAM_BUFFER_BYTES: usize = 0x08;
const STREAM_SOURCE: usize = 0x0c;
const STREAM_WRITE_AT: usize = 0x64;
const STREAM_CHUNK: usize = 0x6c;

/// The source those bytes are read out of, and how much of it there is: 0x94 bytes, allocated at 0x45c933 and
/// built by 0x45e020.
///
/// **The one the music has is the streaming kind**, which is what makes this game's own position a file
/// pointer rather than memory: 0x45e020 leaves `+0x7c` at nothing, and the read at 0x45e360 takes that branch —
/// `ReadFile` on the handle at `+0x8c`, with the bytes left before the loop counted down in `+0x8`. The other
/// kind, `+0x7c` set by 0x45e1d0, is a track held whole in memory and is not what `thbgm.dat` is streamed as.
const SOURCE_BYTES: usize = 0x94;
const SOURCE_FILE: usize = 0x8c;
const SOURCE_RECORD: usize = 0x90;
/// And how much of the track's sound is left before it loops, `+0x8` of the source: the read at 0x45e46f
/// takes every byte it hands over off it, and the game's own seek puts it back to the whole length at
/// 0x45e34d or to the length past the loop point at 0x45e30f. Which makes it and the file's position the pair
/// `Music::bytes_left` is about.
const SOURCE_LEFT: usize = 0x08;

/// Which track is playing, out of the record the source is reading: where its sound starts in the file
/// (`+0x10`), where it loops back to (`+0x18`) and how long it is (`+0x1c`).
///
/// The three numbers the game's own seek is built out of, at 0x45e2cc — the file's base at `+0x39c8` of the
/// player plus the record's `+0x18` and `+0x10` — so they are the record's identity as far as the game itself
/// uses it. Which is the shape 紅魔郷's identity has too, and for the same reason: what a chapter needs to
/// know is whether the track has *changed*, and one file holds every track here.
const RECORD_AT: usize = 0x10;
const RECORD_LOOP_AT: usize = 0x18;
const RECORD_BYTES: usize = 0x1c;

/// And the track's own name, inline at the front of that record with the numbers above behind it: what the
/// player's own lookup compares a name against, at 0x44bc43, is the record's first bytes.
///
/// The name a record carries is the file's own, `th07_02.wav` and its like, the directory stripped — 0x44bb06
/// walks past the last `/` or `\` of the name it is given before it compares. Which is what
/// [`SLOT_NAMES`] has to be held against rather than matched whole.
const RECORD_NAME: usize = 0x00;
const RECORD_NAME_BYTES: usize = RECORD_AT;

/// `th07::GameWindow::Present`, taking nothing: the frame handed to the display.
///
/// `IDirect3DDevice8::Present(NULL, NULL, NULL, NULL)` through +0x3c at 0x4345df, the lost-device
/// `Reset` beside it at 0x434604, and the game's own screenshot key after it at 0x434621 — the three
/// things 紅魔郷's present does, in one function. The game's own frame calls it once per frame, at
/// 0x434a0e and 0x434a18, which are the two ways out of its frame-skip loop.
///
/// No `this`: it reads `ecx` nowhere, so the [`Call`] below carries nothing to call it on. `__thiscall`
/// with one argument is `fastcall` with nothing on the stack — see [`Call`] — and a `fastcall` whose
/// `ecx` the callee ignores is a `__cdecl` of no arguments called correctly.
const PRESENT: usize = 0x004345c0;

/// `th07::GameWindow::Render`, `__thiscall` on [`G_GAME_WINDOW`]: the game's own whole frame, which
/// orb's loop replaces.
///
/// Reached at `mov ecx,0x575c20; call 0x4346e0` from the message pump at 0x4341f2, on the frames
/// `IDirect3DDevice8::TestCooperativeLevel` answers `D3D_OK` for. What it is made of is one draw, one
/// update, the sounds and the present — and around them a loop that decides, from the game's own
/// frame-skip setting at 0x575a8b and its own timing at 0x575c34, whether a pass draws at all and whether
/// to take another before presenting. Read off 0x434708 to 0x434a33. orb's loop takes all of that out, as
/// it takes 紅魔郷's out: one update and one draw a frame, handed over on the display's own cadence.
const RENDER: usize = 0x004346e0;
/// `push ebp; mov ebp,esp; sub esp,0x4c` — position-independent, 6 bytes, read out of the exe at that
/// address.
const RENDER_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x4c];

/// The whole of what the game reads a frame's buttons with, `__cdecl` taking nothing and answering the word in
/// `ax`: 0x430b50.
///
/// **The keyboard both ways it has of asking, and the pad with it.** Where the game has no DirectInput device
/// (`[0x575960]` null) it goes through `GetKeyboardState` at 0x430b76 and reads the 0x100 bytes a key at a
/// time; where it has one it asks that device at 0x430eeb and re-acquires it on `DIERR_INPUTLOST` — and both
/// paths end in [`GET_CONTROLLER_INPUT`], which is what makes this one read rather than two.
///
/// Its callers are the supervisor's own input job, twice: 0x437d80 keeps last frame's word in 0x4b9e54 and
/// writes this one over [`G_INPUT_READ`], and 0x437dfc *adds* what it answers to the word already there — the
/// path taken where `[0x62627d]` is set, which is a run being played back rather than played. So a frame's
/// keys arrive here whichever of the two it is.
const INPUT_READ: usize = 0x00430b50;
/// `push ebp; mov ebp,esp; sub esp,0x108` — position-independent, 9 bytes, read out of the exe at that
/// address.
const INPUT_READ_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x81, 0xec, 0x08, 0x01, 0x00, 0x00];

/// The pad half of the input read, `__fastcall`: the word the keyboard was read into arrives in `ecx` and
/// comes back with the pad's bits added.
///
/// A tail call inside [`INPUT_READ`] and its only caller — `mov ecx,[ebp-0x104]; call 0x4303f0` at 0x430f1a
/// and again at 0x4312b7, once per way that read has of getting the keys — so it cannot be told apart from
/// that read from outside it, which is what 紅魔郷's is too. **The convention is not 紅魔郷's**: that one
/// takes the word on the stack (`push edx; call 0x41cfc0; add esp,4` at 0x41dc77), and this one in `ecx`,
/// which is why the seam says which — see [`Entered`].
///
/// What it does with the word is [`PAD_MAPPING`] and the stick, and orb answers the whole of it rather
/// than calling through: see [`crate::runtime::get_controller_input`] for why that is one thing in one
/// place. **Which is what makes a pad work here at all** — 妖々夢 reads a DirectInput device of its own
/// first (`[0x575964]`, polled at 0x430684) and falls back to `joyGetPosEx(0)` at 0x43042e, and a pad
/// neither of those two reaches is a pad the game has none of.
const GET_CONTROLLER_INPUT: usize = 0x004303f0;
/// `push ebp; mov ebp,esp; sub esp,0x160; push edi` — position-independent, 10 bytes.
const GET_CONTROLLER_INPUT_PROLOGUE: &[u8] =
    &[0x55, 0x8b, 0xec, 0x81, 0xec, 0x60, 0x01, 0x00, 0x00, 0x57];

/// The pad's buttons one byte apiece, as the game's own key config screen reads them: `__cdecl` taking
/// nothing and answering where [`PAD_BUTTONS`] is, with 0x80 in the byte of every button held.
///
/// **Its one caller is that screen**, at 0x4574d7: it walks the first 0x20 bytes for the first with 0x80
/// set and assigns that button number, which is what makes this the read a mapping is made through. The
/// function zeroes the whole array at 0x4309d2 and then fills it — from its DirectInput device where it has
/// one, at 0x430a5c, and otherwise from `joyGetPosEx(0)`'s button mask a bit at a time, at 0x430a25. So a
/// pad neither of those reaches is a pad nothing can be mapped from, which is what orb adds to it.
const GET_PAD_BUTTONS: usize = 0x004309c0;
/// `push ebp; mov ebp,esp; sub esp,0x154` — position-independent, 9 bytes.
const GET_PAD_BUTTONS_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x81, 0xec, 0x54, 0x01, 0x00, 0x00];
/// And the array itself, which that function answers with: `mov eax,0x135e218` at 0x430a0c, 0x430a52 and
/// 0x430b3d.
///
/// **Only a laid-out game needs the address**, which is why it is behind that gate: orb writes through the
/// pointer the game's own read answered with and never has to know where it is. What a laid-out one needs
/// it for is having an array there to answer with at all.
#[cfg(any(test, feature = "sim"))]
const PAD_BUTTONS: usize = 0x0135e218;
const PAD_BUTTONS_READ: usize = 0x20;
const PAD_BUTTON_HELD: u8 = 0x80;

/// Which pad button is which, as the game keeps it: nine `i16`, and a negative one names no button.
///
/// Read off the nine calls that read makes on the helper at 0x430370, one per bit of the word: 0x1 with
/// the `i16` here at 0x430457, then 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80 and 0x100 with the next apiece,
/// the last at 0x4305a7. The helper is `if (index >= 0 && buttons & (1 << index)) *word |= bit`, which is
/// what makes this a table of pad button numbers in the word's own order.
///
/// **The order is shoot, bomb, focus, menu, up, down, left, right, skip**, and what fills the table is the
/// first 18 bytes of `th07.cfg` — the same nine the launcher reads out of that file for its own dialog,
/// `pad::Mapping` there. The game's own defaults for them are the 18 bytes at 0x49ee40, copied here at
/// 0x4399d3: `0, 1, 2, 4, -1, -1, -1, -1, 3`, with the four directions unmapped because a stick is how a
/// pad is pushed.
const PAD_MAPPING: usize = 0x00575a68;

/// The four directions in that word, which the stick decides: 0x4305ac to 0x430671 sets right where the X
/// axis is past its centre by a quarter of its travel, left where it is short of it by the same, and the
/// two for Y — the dead zone [`crate::joystick::axis`] is the rule for.
///
/// The same four the mapping's fifth to eighth entries name, so a button configured as a direction and a
/// stick pushed that way set the same bit.
const PAD_UP: u16 = 0x10;
const PAD_DOWN: u16 = 0x20;
const PAD_LEFT: u16 = 0x40;
const PAD_RIGHT: u16 = 0x80;

/// And the four buttons of that word, which are the first, second, third and ninth of the same table —
/// shoot, bomb, focus and skip. **What a judging pass is driven with**, a pad being what somebody watching
/// a stage go by has in their hands.
///
/// The two menu masks above are made of these and the keyboard's own: `MENU_DECIDE` is shoot with return
/// and `MENU_CANCEL` is bomb with the menu button.
const BUTTON_SHOT: u16 = 0x001;
const BUTTON_BOMB: u16 = 0x002;
const BUTTON_FOCUS: u16 = 0x004;
const BUTTON_SKIP: u16 = 0x100;

/// The device set up: called once it exists and after every reset, before anything is presented.
///
/// **Three call sites, and together they are what identifies it.** 0x435187 is the tail of 0x434bd0, the
/// function that makes the device — `IDirect3D8::CreateDevice` is tried there at 0x434d94 and twice more
/// with the device type stepped, and the message pump reaches the whole of it once, at 0x4340eb. 0x434248
/// is the pump's `D3DERR_DEVICENOTRESET` branch, after `IDirect3DDevice8::Reset` (+0x38) answered a
/// success. 0x434607 is that same reset inside [`PRESENT`]. So every path that leaves a usable device
/// behind ends here, which is what orb needs of it: the first moment the device's vtable can be
/// redirected, and again after a reset has replaced it.
///
/// Its body is nothing but device state — `SetRenderState` through +0xc8 from 0x4356b9 on, the first of
/// them state 7 set from the configuration bit at `[0x575a9c] >> 6` — and it takes nothing: `ret` at
/// 0x435bc0 with no immediate, and `ecx` read nowhere in it.
const INIT_D3D_DEVICE: usize = 0x004356a0;
/// `push ebp; mov ebp,esp; sub esp,0x1c` — position-independent, 6 bytes.
const INIT_D3D_DEVICE_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x1c];

/// The object every quad the drawing queues goes into, **as a pointer**: 0x17e560 bytes off the
/// allocator at 0x434119, put here at 0x434147, and read back as `mov ecx,DWORD PTR ds:0x4b9e44` at
/// each of the three calls the game's frame makes on it.
///
/// So what orb calls those on is this address's contents and not this address. The object is on the
/// heap and there is nothing here until the game has been through its own startup, which is one thing
/// [`G_SOUND_PLAYER`] and [`G_CHAIN`] are not.
const G_QUAD_QUEUE: usize = 0x004b9e44;

/// `__thiscall` on what [`G_QUAD_QUEUE`] holds: the queue emptied, which is what makes the next quad
/// go to the start of the buffer.
///
/// Called from [`RENDER`] at 0x434734, inside the scene and before the drawing. Three writes: `0x2e530
/// = 0`, `0x17e534 = this + 0x2e534`, `0x17e538 = 0x17e534` — a count and the two ends of a buffer
/// which is 0x2e534..0x17e534 of the object.
///
/// **This is the call orb's own loop left out, and 0x17e534 is the null it faulted writing through**:
/// the append at 0x44f690 copies six vertices of 0x1c bytes to what that field points at and adds 0xa8
/// to it, so a frame whose queue was never emptied writes its first quad to address zero. See
/// [docs/adr/0004](../../../../../docs/adr/0004-th07-is-a-second-game-chosen-at-the-attach.md) for the
/// launch that faulted at 0x44f6aa, which is that `rep movsd`.
const EMPTY_QUEUE: usize = 0x0044f580;

/// And the queue drawn, `__thiscall` on the same object: `0x2e530` pairs of triangles from what
/// 0x17e538 points at through `IDirect3DDevice8::DrawPrimitiveUP` (+0x120) with a stride of 0x1c and
/// the vertex format at +0x130, and then the count back to zero and 0x17e538 up to 0x17e534.
///
/// Called from [`RENDER`] at 0x43475d, after the draw chain and inside the scene still — and again at
/// 0x43478e, outside it, where the count this one left at zero makes it return at 0x44f5d1 without
/// touching the device. So the second call is the game's own no-op on every frame it drew, and orb
/// makes the one that draws.
const DRAW_QUEUE: usize = 0x0044f5c0;

/// `__thiscall` on [`G_SUPERVISOR`]: the fog put out, with the queue drawn first so that what was
/// queued under fog is drawn under it.
///
/// Called from [`RENDER`] at 0x434748, inside the scene and before the drawing. It is the pair of
/// 0x43a1bd, which turns fog on: each compares [`FOG_ON`] against what it is about to set, sets it, and
/// calls `IDirect3DDevice8::SetRenderState(D3DRS_FOGENABLE, …)` — state 0x1c through +0xc8 on
/// `[this+8]` — one with 1 and one with 0, and **neither reaches the device where that field already
/// says so.** What turns the fog on is a stage's own background drawing, at 0x406d9f and 0x408128.
const FOG_OFF: usize = 0x0043a207;

/// The object those two are called on, whose `+8` is [`G_D3D_DEVICE`].
///
/// `mov ecx,0x575950` at 0x434743, and the `SetRenderState` inside [`FOG_OFF`] goes through
/// `[[this+8]]+0xc8` — which is 0x575958, the device the one `CreateDevice` fixed. So the device
/// pointer orb reads is a field of this object, and that 紅魔郷's own supervisor keeps its device at
/// +8 as well is agreement rather than the evidence for it.
const G_SUPERVISOR: usize = 0x00575950;

/// The scene the game is in and the one it has been asked for, which is where every scene change is
/// decided: `[this+0x154]` and `[this+0x158]`.
///
/// **Which of the two is which is the game's own log line.** `Supervisor::OnUpdate` at 0x437c70 compares
/// them at 0x437e14 and, where they differ, prints `scene %d -> %d` (0x497230) with `+0x154` pushed
/// first and `+0x158` second, then keeps what it is leaving in `+0x15c`. So the first is the scene now
/// and the second is the scene wanted, which is the pair orb's own state line is written from.
///
/// The transition is a switch on the scene being left — cases at 0x437eac, 0x437ee3, 0x437fb1, 0x437f7b,
/// 0x4381d1, 0x438216 and 0x438249 — and inside each, a switch on the scene wanted whose branches
/// register it: out of scene 1, wanted 2 goes through 0x42f3c5, 5 through 0x44a302, 8 through 0x43b4db
/// and 9 through 0x41e820.
const SUPERVISOR_SCENE: usize = 0x154;
const SUPERVISOR_WANTED: usize = 0x158;

/// The game manager, which is what a run is built into: the object 0x42f3c5 registers for the scene a run
/// is, as the argument of both the calc element at 0x62f8b4 and the draw element at 0x62f8d4.
///
/// Its update is 0x42d8d5, its added callback 0x42e83e — where a run's numbers are put in place — and its
/// draw 0x42e1d4.
const G_GAME_MANAGER: usize = 0x00626270;

/// The difficulty a run is being played on, `+0x10` of it.
///
/// **Two directions say so.** The added callback reads it at 0x42eab1 and starts the rank at 0xc8 where it
/// is 4 or more and 0x32 where it is less, which is the pair 紅魔郷 takes out of a table by difficulty. And
/// a replay's own header writes its difficulty straight into it: the byte at `+0x57` of what 0x455b45 read
/// goes to 0x626280, which is this field.
const DIFFICULTY: usize = 0x10;
/// Whose run it is and which of that character's shots, `+0x93d5` and `+0x93d6`, with `character * 2 +
/// shot` kept beside them at `+0x93d7` — which the added callback builds at 0x42e864, and which is the
/// number every table of either game is indexed by.
const CHARACTER: usize = 0x93d5;
const SHOT_TYPE: usize = 0x93d6;

/// The run's own numbers, **as a pointer**: 0xc8 bytes off the allocator at 0x42e93f, kept at the game
/// manager's `+0x8` and so at 0x626278 absolutely, which is how the rest of the game reaches them — 287
/// places read that address.
///
/// A pointer and not a struct, so every read of a field below goes through it and answers nothing where no
/// run has been built: the object does not exist until the added callback at 0x42e83e allocated it.
const G_RUN_NUMBERS: usize = 0x00626278;

/// The power the shot is at, which this game keeps as a **float** where 紅魔郷 keeps an integer:
/// `[run+0x7c]`.
///
/// Three places read it the same way — `fld DWORD PTR [0x626278+0x7c]`, the float-to-int at 0x48b8a0, then
/// `cmp eax,0x80` — at 0x415478, 0x42c3e3 and 0x432710, and 128 is this game's full power. The continue
/// path at 0x404389 puts it back to nothing.
///
/// **`State::power` is a `u16`, so what orb reads is the whole part of it**, taken the way the game takes
/// it: everything orb does with the number — the log's own line, and what a chapter would carry — works in
/// whole steps, and a power that is 62.5 is a shot at 62 either way. Whether the trait should carry what
/// each game keeps instead is a decision for the day a chapter of this game is taken, and it belongs in
/// `docs/adr/` then rather than in a cast here.
const POWER: usize = 0x7c;

/// The score, `[run+0x4]`, and the point items, `[run+0x28]`.
///
/// **The score is capped where a score is capped**: the run's teardown at 0x42f45d clamps this field to
/// 0x3b9ac9ff where it has reached 0x3b9aca00 — 999,999,999 — and copies it into `+0x0`, which is the number
/// the panel draws. An item collected adds a tenth of its value to it at 0x433258, the tenth being what makes
/// the panel's own last digit the continues rather than a unit.
///
/// **The point items are the count with thresholds behind it**: that same pickup adds one to `+0x28` at
/// 0x43327d, and 0x4333bd compares it against `+0x30` — where it has reached it, `+0x2c` goes up by one and
/// `+0x30` becomes `(+0x2c - 2) * 0x1f4 + 0x320`. So `+0x2c` is how far the point value has climbed and
/// `+0x30` is the next step's count, which is 妖々夢's own 点 system and not a rank: the build starting `+0x30`
/// at 0x32 or 0xc8 by difficulty is the first step, higher for the Extra and the Phantasm.
const SCORE: usize = 0x4;
const POINT_ITEMS: usize = 0x28;

/// The score the panel shows, `[run+0x0]`, which the run's own end catches up with [`SCORE`]: the teardown
/// at 0x42f483 copies one into the other, and so does every way out of the pause menu — 0x403b8b, 0x403bd0
/// and 0x403c0e. What [`Th07::leave_run`] does with it is what those three do.
const SCORE_SHOWN: usize = 0x00;

/// **妖々夢's cherry, which is three numbers and a baseline they are all counted from.** The baseline is
/// `[run+0x88]` and the three are statics: the cherry at 0x62f88c, what it is filling towards at 0x62f888,
/// and the cherry+ at 0x62f890. Each is kept as an absolute number and every reader takes the baseline off
/// it — 0x408621, 0x41b3f9, 0x42013b, 0x43316a and a dozen more — which is what makes the baseline part of
/// the set rather than a fourth number beside it.
///
/// **What each one is, by what moves it:**
///
/// - **The cherry** goes up as a run is played and down where it is spent: an item collected at 0x42016e
///   takes from it, a bomb at 0x440ae7 and a death at 0x440ee7 subtract what they cost, and the panel draws
///   it at 0x43316a against 0xc350 — 50000, the border's own figure.
/// - **What it is filling towards** is what the cherry is compared against: 0x43375f and three more sites
///   test `cherry >= [0x62f888]`, and 0x41b40f divides by it for the bar the panel draws. So it is the
///   target rather than a count, and a stage resumed with the wrong one fills at the wrong rate.
/// - **The cherry+** is the border's own: the respawn at 0x4414e5 clamps it at nothing, a border broken at
///   0x441da4 writes it, and 0x4428aa is where passing its figure puts a border up — 0x441960, the call the
///   game's own replay restore makes at 0x4438db for the same reason.
///
/// Which is why they are three fields of `RunState` and not a number orb works out: a stage played again has
/// to start with the cherry it had, filling towards what it was filling towards, with the border it had
/// earned. See [`Th07::run_state`].
const SCORE_BASELINE: usize = 0x88;
const G_CHERRY: usize = 0x0062f88c;
const G_CHERRY_TARGET: usize = 0x0062f888;
const G_CHERRY_PLUS: usize = 0x0062f890;
/// And what the game's own restore does with the cherry+ rather than writing it outright: 0x4438c1 holds it
/// to the baseline plus 0xc350 and puts a border up where it was over, which is that number being the border's
/// own figure — the same 0xc350 the panel draws the cherry against.
const CHERRY_FOR_A_BORDER: u32 = 0xc350;

/// The border going up, `__thiscall` on [`G_PLAYER`] taking nothing: 0x441960.
///
/// Two callers and they are the two moments a border is earned — 0x4428c3, where the cherry+ passes its own
/// figure, and 0x4438db, where the game's own replay restore finds a record whose cherry+ had passed it. The
/// second is the one [`Th07::set_run_state`] stands in for, which is why orb calls this rather than writing
/// the state a border is: what it does is 540 frames on the player's own timer, the state at 4, and the
/// effects and the sound that go with it — see [`PLAYER_COMING_BACK`] for the states either side of it.
///
/// Handed over for `handed_over!`'s own reason: it is code, and a laid-out game has none.
const BORDER_UP: usize = 0x00441960;

/// The two scenes a run leaves for: 7 where a replay was being watched and 6 otherwise.
///
/// The pause menu's own way out is where both come from — 0x403bb8 writes 7 under bit 3 of [`G_RUN_KIND`] and
/// 0x403b8d and 0x403bfe write 6 — and the run ending of its own accord at 0x42a6ab writes 6 as well. Each
/// path clears [`OTHER_MENU_UP`] and equalises the two scores first, which is the whole of what leaving is.
const SCENE_RUN_OVER: i32 = 6;
const SCENE_REPLAY_OVER: i32 = 7;

/// The seed the stage began with, which the game keeps beside the generator: `[gm+0x95e4]`.
///
/// Written at 0x42f0a4, in the build, from the word at [`G_RANDOM`] — and read at 0x4432e3 into a replay's
/// stage record, which is the same use orb has for it. So the generator says where the numbers have got to
/// and this says where they started, which is what a stage played again has to be given.
const SEED_AT_STAGE_START: usize = 0x95e4;

/// The lives, the bombs and the deaths, floats too and beside the power: `[run+0x5c]`, `[run+0x68]` and
/// `[run+0x50]`.
///
/// **Each is named by the event that changes it**, and the events are what makes these three certain where
/// the panel could not: the game reaches all five of its guarded numbers through one shape, a `__thiscall`
/// on the game manager taking a whole number, adding it to one field as a float and re-sealing — 0x42d5cd
/// for `+0x5c`, 0x42d612 for `+0x68`, 0x43b750 for `+0x50`, 0x43b7a0 for `+0x6c`, [`POWER`]'s own 0x4325e0.
/// A bomb going off calls two of them at 0x440bc1 and 0x440bcd, `+0x6c` up by one and `+0x68` down by one:
/// the bombs, and the count of bombs used beside them. A death calls three, `+0x50` up by one at 0x440d8c,
/// the power down by 16 at 0x440dd9 — which is what a death costs in this game — and `+0x5c` down by one at
/// 0x441171: the deaths and the lives. A run's build fills `+0x5c` from the byte at `[[0x626274]+0x1c]`,
/// which the same build sets to 2 for the Extra and the Phantasm and to 8 for a practice run.
///
/// **Writing any of them takes more than the write.** 0x4012b0 re-seals the numbers and every one of those
/// five calls it after its add; 0x404fe0 is the check, and where it fails the caller fills 0xb4 dwords from
/// 0x575950 with 0xffffffff — the device pointer, the viewport, [`PAD_MAPPING`] and the scene at
/// [`SUPERVISOR_SCENE`] all inside that range. So this game answers a changed number by destroying itself,
/// and a snapshot of orb's that put lives back one field at a time would trip it. See
/// `docs/todo/th07-the-run.md`.
const DEATHS: usize = 0x50;
const LIVES: usize = 0x5c;
const BOMBS: usize = 0x68;

/// How long the stage has been running, `+0x95e8` of the game manager.
///
/// **The reset is what makes it the stage's and not the run's**: 0x42f3c5 puts it back to nothing, and that
/// is the function that hangs the manager on the chain — called for the scene wanted being 2 *and* for its
/// being 3, which is a stage transition, so the count starts again at every stage. The update at 0x42d8d5
/// adds one to it as the last thing it does, at 0x42e1bf, on the paths that reach its tail; three paths jump
/// straight to the return at 0x42db3d, 0x42dc86 and 0x42de9b and count nothing. Nothing else in the exe
/// touches the field.
///
/// **A frame under either menu is not one of them**, which is what says a chapter's frame numbers survive a
/// pause: 0x42de6f tests [`GAME_MENU_UP`] against 1 and against 2 and the byte after it against nothing, and
/// any of the three jumps to 0x42de98, which answers **3** and returns — 0x334 bytes before the add. The same
/// 3 is what the game's own slowdown answers with (0x42e19a), on the frames it decides not to advance the run
/// on: with the byte at `[[G_GAME_MANAGER+4]+0x25]` set, one frame in five, four or three counts nothing where
/// the live bullet count at 0x9a9a80 has reached 0x80, 0xe0 or 0x140.
///
/// And [`SCRIPT_FRAMES`] stops with it, because of what 3 means to the chain: the walk's own table at 0x42fdf9
/// sends it to 0x42fdd0, which **stops the walk** and answers 1. So a frame the manager answers 3 on is a frame
/// no job below its own priority runs in at all — the enemy manager's, which is where the four main contexts
/// are stepped, the player's at 8 and the run's own word at 0x10 included.
const STAGE_FRAMES: usize = 0x95e8;

/// `th07::GameManager::AddedCallback`, `__thiscall` on the manager: the added callback of the element the
/// scene-2 register hangs on the chain, so `Chain::AddToCalcChain` runs it at the moment a stage is
/// registered — before that stage's own first update, which comes later in the same walk.
///
/// **This is where the game itself puts a stage's numbers in place**, which is why a resumed run's go back
/// after it returns rather than before it: 0x42e83e to 0x42f2e0 allocates the run's numbers on the first
/// stage of a run, fills the lives out of the stock byte and the bombs out of the configuration, puts the
/// power to nothing, takes the high score out of the score file, adds one to [`G_STAGE`], copies
/// [`G_RANDOM`] into [`SEED_AT_STAGE_START`], and loads the stage's own script and sprites. Anything written
/// before it is written over.
const GAME_MANAGER_ADDED: usize = 0x0042e83e;
/// `push ebp; mov ebp,esp; sub esp,0x40` — position-independent, 6 bytes, read out of the exe at that
/// address.
const GAME_MANAGER_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x40];

/// The stage's own build, `__thiscall` on the stage's data: 0x4074c0, which clears the 0x14ad dwords at
/// 0x1347b00 and lays the stage out of them. **The one moment a resumed run's seed can go in.**
///
/// **Called from one place in the whole exe**, 0x42f0b4, and that place is what makes it the seam: the twelve
/// instructions before the call are the game settling the stage's seed — 0x42f07e saves [`G_RANDOM`], 0x42f08b
/// seals the run's numbers over the top of it (two draws, see [`RESEAL`]), 0x42f094 puts it back and 0x42f0a4
/// writes it down as [`SEED_AT_STAGE_START`]. So at this entry the seed is settled and **nothing of the stage
/// has been drawn from the generator yet**, which is what [`crate::resume::stage_building`] needs.
///
/// **Both of those are inside [`GAME_MANAGER_ADDED`]**, which is why the seed cannot go in with the numbers:
/// orb's hook over that callback runs when it returns, by which time the build has drawn. A 妖々夢 run picked
/// up out of its file parted from the run at the stage's first frame with its generator still holding the seed
/// — `the generator is 0x6167 where the run had 0xe33d` — and the stage had been laid out from a seed of the
/// new build's own.
const STAGE_BUILD: usize = 0x004074c0;
/// `push ebp; mov ebp,esp; sub esp,0x8; push edi` — position-independent, 7 bytes, read out of the exe at that
/// address. The same seven 紅魔郷's own stage register begins with, both being what this compiler writes.
const STAGE_BUILD_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x08, 0x57];

/// The player, which is a static object at 0x4bdad8 and not a pointer: its register at 0x4429d0 is called
/// with nothing in `ecx` and starts by filling 0x2df9e dwords from that address with zeroes, then makes the
/// three chain elements whose callbacks are 0x441fb0, 0x4420b0 and 0x442350 and hangs the object off each.
/// 0x441fb0 is the update, which is the function every field below is read or written by.
const G_PLAYER: usize = 0x004bdad8;

/// What the player is doing, `+0x2408`, a signed byte and a state machine of five values.
///
/// **The transitions are what name them**: the hit test sets 2 at 0x43ee0e, the update calls the death
/// handler at 0x441fee where it finds 2, that handler sets 1 at 0x4410a0 beside the deaths and the lives it
/// moves, the respawn walks 3 and 4 (0x441785 out of 1, 0x441a42 beside the byte at `+0x240d`), and 0x4413fa
/// puts it back to 0. So 0 is the state a run is played in and every other value is the player in the
/// middle of dying or coming back — which is the whole of what `State::unsettled` asks.
///
/// Which of 3 and 4 is the respawn is not separated here because nothing of orb's needs it: a chapter cannot
/// be taken or begun in any of the four.
const PLAYER_STATE: usize = 0x2408;
const PLAYER_NORMAL: i8 = 0;
/// And the one of the five [`Th07::make_invulnerable`] writes: 3, which the respawn at 0x441785 enters and
/// which nothing can hit — the two hit tests, 0x43e375 and 0x43e360, answer *survived* for any state but 0.
///
/// **Not 4, which is the state after it.** 4 is 妖々夢's border, and a hit there is deflected by 0x441bd0 —
/// which takes the border down, plays its own effect and is part of what a run scores. Writing that from orb
/// would be playing the game rather than keeping the player alive for an update.
const PLAYER_COMING_BACK: i8 = 3;
/// The frames left under that state, which is `+0x8` of the timer at `+0x16a00`: 0x4394c7 takes one off it
/// every update, and the state's own handler puts the player back to [`PLAYER_NORMAL`] at 0x4413fa where it has
/// run out.
///
/// **It has to be written and not only the state**, for 紅魔郷's reason: frames left where the last respawn
/// left them is a state put back to normal in the same update it was written for, and the update it runs out in
/// is one the hit test sees a player it can kill. 0xf0 is what the respawn itself writes first (0x441756),
/// before it settles on 0x28 for the walk in — and it is far enough from nothing that the tick cannot reach it
/// inside one update, which is the whole of what orb needs of the number.
const PLAYER_COMING_BACK_FRAMES: usize = 0x16a08;
const PLAYER_INVULNERABLE_FRAMES: i32 = 0xf0;

/// Where the player is, `+0x930` of it: the position its own bomb hands the effect manager at 0x440de5, which
/// is a bomb going off *at the player*.
const PLAYER_POSITION: usize = 0x930;

/// Whether a bomb is going off, `+0x16a20` of the player — the flag its own update branches on.
///
/// 0x4409f0 is that update: where the flag is set it runs the bomb's own object at `+0x16a30`, and where it
/// is clear it asks whether a bomb can be started — the bomb button in the word at 0x4b9e50, [`BOMBS`] above
/// zero, no dialogue running, `+0x23fc` at nothing — and starting one is 0x440bfa, which writes this flag as
/// it takes the bomb off the count. So the flag is up for exactly as long as the bomb is.
const BOMB_IN_USE: usize = 0x16a20;

/// The textures the sprite manager holds, which a restore must not put back: `+0x282ac` of it, 0x108 of
/// them.
///
/// **0x44e6f0 is where both numbers come from**, the manager's own release of one slot: it refuses a slot
/// below zero or 0x108 and over, calls `IUnknown::Release` through `[[this+slot*4+0x282ac]]+0x8`, and puts
/// the slot back to nothing — then frees `[this+slot*4+0x286cc]`, the file the texture was made from. And
/// those two offsets check each other: 0x282ac plus 0x108 pointers is 0x286cc exactly, so the array ends
/// where the next one starts.
///
/// The manager is the object the frame already reaches for its queue of quads, [`G_QUAD_QUEUE`] — a pointer,
/// null until the game made one, which is why the answer is a walk through it rather than an address.
///
/// **What is deliberately left out** is everything else it holds: 妖々夢 does not lose its device inside a
/// stage any more than 紅魔郷 does, and a range left out of a restore is a range the snapshot no longer
/// describes.
const ANM_TEXTURES: usize = 0x282ac;
const ANM_TEXTURE_SLOTS: usize = 0x108;

/// The enemies, as their manager counts them itself: the static object at 0x9a9b00 and the count at
/// `+0x9545bc`.
///
/// 0x422f40 is the manager's register — it makes three chain elements, the calc one at priority 0xa with
/// 0x420620 as its callback — and 0x420620 is the update that keeps this field: it puts the count back to
/// nothing at 0x4207bd and adds one for every slot it finds alive as it walks all 0x1e0 of them, each 0x4f48
/// bytes from `+0x4f50`, alive being bit 7 of the slot's `+0x2e28`. So the field is the live count as of this
/// frame, which is what orb would otherwise walk 480 slots to work out.
const G_ENEMY_MANAGER: usize = 0x009a9b00;
const ENEMY_COUNT: usize = 0x9545bc;

/// The bosses, as **eight pointers** at `+0x954598` — and whether any is there is the game's own answer:
/// 0x4232a0 walks those eight and returns 1 at the first that is not null, which is what `boss_present` is.
const BOSSES: usize = 0x954598;
const BOSS_SLOTS: usize = 8;

/// The clock the stage's own script runs on: `+0x9545fc`, which is the **first** of the manager's four main
/// contexts.
///
/// Each context is 0x10 bytes from `+0x9545f4` and is a timer with the instruction pointer after it —
/// `previous`, `current_f`, `current`, `instruction` — and 0x41f6f0 runs it: while `current` has reached the
/// time word at the head of the instruction it runs that instruction and steps the pointer on by the size at
/// the instruction's `+0x6`, then copies `current` into `previous` and steps the timer through the
/// supervisor's helper at 0x43958d, which is the same step every other timer in the game takes. The manager
/// is re-registered for every stage, so the clock starts again with the stage.
///
/// **Which of the four is a decision and not a reading** — 紅魔郷 has one timeline on one clock and orb's
/// midstage boundaries are written in its frames. See `docs/adr/0018`.
///
/// And it stops where [`STAGE_FRAMES`] stops, for a reason that is not its own: the contexts are stepped from
/// inside the enemy manager's update (0x4207a1), which is a job below the game manager's on the chain — so a
/// frame the manager answers 3 on is a frame the walk never reaches them in.
const SCRIPT_FRAMES: usize = 0x9545fc;

/// A boss's life and how long its current attack has run, `+0x2bb8` and `+0x2bcc` of the enemy.
///
/// The life is what a spawn is given and what a hit takes away: 0x41f374 writes the argument of the spawn
/// into `+0x2bb8`, 0x42143b subtracts the damage from it and writes it back, and 0x4205fa takes ten off it.
///
/// The frames are the enemy's own timer, whose `current` is at `+0x2bcc` — the same shape as the spellcard's
/// below, advanced by one at 0x42085a — and **what makes it the *attack's* frames is the comparison the ECL
/// makes**: 0x41ffe1 reads it against `[enemy+0x2edc]`, the limit of the attack being run, which is what times
/// an attack out. A clock compared against this attack's limit is this attack's clock.
const ENEMY_LIFE: usize = 0x2bb8;
const ENEMY_ATTACK_FRAMES: usize = 0x2bcc;

/// The spellcard being fought, as the manager keeps it: `+0x9545c8` says one is running and `+0x9545d8` is
/// which, counted the way the score file counts them.
///
/// 0x40fd89 is a card starting — it sets both, the number out of the enemy's own `+0xe`, and reaches the
/// record for it at `0x626288 + card * 0x78` where the run's 141 `CATK` records are. 0x41048d clears
/// `+0x9545c8` as the card ends, beside the one it adds to `[run+0x1c]`.
///
/// **The neighbouring `+0x9545c4` is not this**, and the difference is what a death does: 0x440d7b clears
/// that one and leaves this one alone, as a bomb does at 0x440c77 — so that is the bonus still being
/// collectable and this is the card still being fought. orb wants the card, not the bonus.
///
/// `+0x9545e8` is the card's own timer, zeroed as the card starts and stepped every frame at 0x417773
/// through the supervisor's timer helper. `boss_attack_frames` is [`ENEMY_ATTACK_FRAMES`] rather than this,
/// because an attack that is not a card has to reset it too.
const SPELLCARD_RUNNING: usize = 0x9545c8;
const SPELLCARD: usize = 0x9545d8;

/// The bullets, and the count their manager keeps: the static at 0x62f958 and `+0x37a128`.
///
/// 0x425a50 is the update its register hangs on the chain at priority 0xc: it puts the count back to nothing
/// at 0x425a99 and adds one for every live slot as it walks all 0x400 of them, each 0xd68 bytes from
/// `+0xb8c0`, alive being the word at the slot's `+0xbfc`. The game manager's own update reads the same field
/// absolutely at 0x42e1a6 — `cmp DWORD PTR ds:0x9a9a80,0x80` — which is 0x62f958 + 0x37a128.
const G_BULLET_MANAGER: usize = 0x0062f958;
const BULLET_COUNT: usize = 0x37a128;

/// The lasers, which **no field counts**: 0x40 slots of 0x4ec bytes at 0x995f80, alive being `+0x4d4`.
///
/// Both walks of them do it this way — 0x4188aa from the absolute address and 0x424827 as the bullet
/// manager's `+0x366628` — and there is no count kept anywhere, so orb counts them itself. 64 reads a frame
/// is what the game's own walk costs.
const G_LASERS: usize = 0x00995f80;
const LASER_BYTES: usize = 0x4ec;
const LASER_SLOTS: usize = 0x40;
const LASER_ALIVE: usize = 0x4d4;

/// What the game puts over the run — the dialogue and the plate a spell card's name is shown on — as the one
/// object both are inside: the small static at 0x49fbf0, whose `+0x8` is a pointer to it.
///
/// The object is allocated with the stage and null before it, and every call the game makes about either is
/// made on the static with `ecx` — 0x42ad66's question about a dialogue and 0x428887's naming of a card among
/// them — reaching the object through that field.
const G_GUI: usize = 0x0049fbf0;
const GUI_IMPL: usize = 0x08;

/// The line of a dialogue being played, `+0x1fbac` of it, put to -1 by the load at 0x42998a — the shape
/// 紅魔郷's own message index has.
///
/// **The predicate is the game's own**, at 0x42ad66: null pointer is no dialogue, a line of -1 or less is no
/// dialogue *except* -2, which counts. -2 is written at 0x42a624 beside `+0x209bc`, and what makes it a
/// dialogue for orb's purposes is what makes it one for the game's: 0x440b48 is the bomb asking this same
/// function whether a bomb may be started, and a -2 refuses it.
const DIALOGUE_LINE: usize = 0x1fbac;
const DIALOGUE_HANDING_OVER: i32 = -2;

/// And the plate a spell card's name is shown on, `+0x6760` of the same object: the vm the name is baked into.
///
/// 0x428887 is the game's own naming of a card, `__thiscall` on [`G_GUI`] taking which plate to show and the
/// name — called from one place in the whole exe, the card declaration at 0x40fcf1, with the name the ECL
/// carries decrypted a byte at a time by 0xaa. It puts four vms in place, this one last, and bakes the name
/// into this one at 0x428a78.
const CARD_NAME_VM: usize = 0x6760;
/// The colour that baking is asked for, `0xfff0f0` at 0x428a61, and the zero pushed in front of it at
/// 0x428a5f.
const CARD_NAME_COLOUR: u32 = 0x00ff_f0f0;
const CARD_NAME_SHADOW: u32 = 0;

/// **The two input words, which are not the same word.** 0x4b9e4c is what the read produced and 0x4b9e50 is
/// what a run acts on.
///
/// The supervisor's own input step writes the first — 0x437d85, 0x437e0d, 0x4382ae and 0x45c210, each right
/// after the read at 0x430b50 — and keeps last frame's beside it at 0x4b9e54. The recorder then copies it into
/// 0x4b9e50 at 0x442d07, with last frame's of *that* at 0x4b9e58, and writes the same value into the replay's
/// own entry. So a replay being watched has its recorded word in 0x4b9e50 while 0x4b9e4c still holds whatever
/// the keyboard did, which is what makes the second the run's and the first the machine's.
///
/// **Which is also why the pause button is no hazard here**: the manager reads it out of 0x4b9e4c at 0x42d927,
/// the raw word, so a run's own word fed back can never open that menu — see [`RUN_INPUT`].
///
/// Both are read here. The run's is what a frame's own buttons are written down against; the read's is the word
/// **the front end's every screen compares against last frame's** — 0x455c7b takes the title menu's decide out
/// of it and 0x455c9b out of 0x4b9e54 beside it — which is what makes it the one
/// [`Th07::swallow_input`] fills.
const G_INPUT_THE_RUN_ACTS_ON: usize = 0x004b9e50;
const G_INPUT_READ: usize = 0x004b9e4c;
/// And last frame's of the word the run acts on, which the recorder keeps beside it at 0x442cf9: what a run
/// reads an *edge* off is the pair, so a word moved on without this one moved with it is a bomb whose press is
/// never new — see [`Th07::hand_the_read_to_the_run`].
const G_INPUT_THE_RUN_ACTED_ON: usize = 0x004b9e58;

/// Every bit of it, which is what this game's own replay records: 0x442da8 writes the whole word into the
/// entry and masks nothing, four bytes a frame with the flags at `+0x2` beside it.
///
/// A mask would be 紅魔郷's answer, whose record leaves the pause button out because a resume feeding it back
/// would open the menu. Here that cannot happen, [`G_INPUT_READ`] being where the pause is read from — so
/// leaving every bit in is both what the game does and what is safe.
const RUN_INPUT: u16 = 0xffff;

/// The buttons the front end reads as *decide* and as *back*, which are the two masks every screen of it tests
/// against last frame's: 0x1001 in 59 places — the shot button and return — and 0xa in 40, the bomb and the
/// menu button. The title menu's own pair is at 0x455c7b and 0x455c96.
const MENU_DECIDE: u16 = 0x1001;
const MENU_CANCEL: u16 = 0x000a;

/// The game's own record of its spell cards, which orb holds across the two things that would replace it:
/// 0x626288, one 0x78-byte record per card and 0x8d of them.
///
/// **Three readings agree on the extent.** The build clears it at 0x42e657 with a `rep stos` of 0x1086 dwords,
/// which is 0x4218 bytes — exactly 0x8d records of 0x78. The walk that fills it, at 0x42e659, writes `"CATK"`
/// into each record's first dword and 0x78 into the two words at `+0x4` and `+0x6`, which is a record saying
/// its own size. And the file's own reader walks the same 0x8d at 0x445713 and 0x44a145, matching each record
/// by that tag.
///
/// Taken as bytes rather than field by field, which is what the trait asks for: what orb needs is to hold the
/// record across a rewind and across the ranking read, not to know what is in it.
const G_CARD_RECORDS: usize = 0x00626288;
const CARD_RECORD_BYTES: usize = 0x78;
const CARDS: usize = 0x8d;
/// What every record begins with, which is how the game itself tells one it has written from a slot it has
/// not: `"CATK"` as a little-endian dword, written by the build at 0x42e67b and matched by the file's own
/// reader at 0x44573c and 0x44a179.
const CARD_RECORD_TAG: u32 = u32::from_le_bytes(*b"CATK");
/// The attempts and the captures, **a word per shot apiece**: `+0x5c` and `+0x6a`, indexed by the combined
/// `character * 2 + shot` the game keeps at 0x62f647.
///
/// A card starting is what counts an attempt — 0x410110 adds one to `+0x5c` under that index, in the same
/// function that copies the card's name into `+0x2b` — and a card captured adds one to `+0x6a` at 0x4103fb.
/// Both are put back to nothing together where the name in the record is not the card's, at 0x4100b8 and
/// 0x410375, which is a slot being taken over by a card the file knew under another name.
const CARD_ATTEMPTS: usize = 0x5c;
const CARD_CAPTURES: usize = 0x6a;
/// The same attempt counted a second time, into the card's own total: `+0x68`, which 0x410132 adds to right
/// after the shot's own word and which no shot has one of. **It is outside the sum the check byte is over** —
/// the loops at 0x410053 walk `+0x6a`, `+0x5c` and `+0xc` and reach neither this word nor `+0x68` — so it is
/// the one count in the record that can be written on its own.
const CARD_ATTEMPTS_IN_ALL: usize = 0x68;
/// And where the game stops adding to both of them: 0x4100eb and 0x41011c compare against 0x270f and skip
/// the increment, so a card attempted ten thousand times stays at 9999 for the game and has to here.
const CARD_COUNT_CAP: u16 = 0x270f;

/// **And the record checks itself, which is why nothing may write one of those words alone.**
///
/// 0x41008b is the check: the game sums the bytes of the name at `+0x2b`, then the seven captures at `+0x6a`,
/// the seven attempts at `+0x5c` and the seven scores at `+0xc`, and compares the low byte of that sum against
/// the byte at `+0x2a`. **Where they differ it zeroes all fourteen counts**, at 0x4100b8 and 0x4100c5 — so a
/// count raised without the byte put right takes that card's whole history with it the next time the game
/// looks. The captures are `+0x6a`, which is why that offset is a constant after all: the sum is over both. [`Th07::count_card_attempt`] recomputes it for that reason, and this is the same shape the run's own
/// numbers have: see [`DEATHS`] for the other seal and what failing it costs.
///
/// Seven and not six, which is the loop's own bound at 0x41004d and 0x4100ac — 妖々夢 has six shots and the
/// arrays have room for one more.
const CARD_CHECK: usize = 0x2a;
const CARD_NAME: usize = 0x2b;
const CARD_SCORES: usize = 0x0c;
const CARD_SHOTS: usize = 7;
/// And which of those words is this run's: the combined index, `character * 2 + shot`, as the front end and a
/// replay's header both leave it — see [`G_SHOT_CHOSEN`], whose two halves this is the sum of.
const G_SHOT_COMBINED: usize = 0x0062f647;

/// The items on the screen: the third big static at 0x575c70 with its own count at `+0xae2ec`, kept the way
/// the enemies' and the bullets' are — 0x4329e0 puts it back to nothing and the walk over 0x44c slots of 0x288
/// bytes adds one for each that is alive.
const G_ITEM_MANAGER: usize = 0x00575c70;
const ITEM_COUNT: usize = 0xae2ec;

/// How many numbers the generator has been asked for, `+0x4` of it: the step at 0x431870 adds one every time
/// it is called. Which is what makes it worth writing down beside a chapter — two runs of the same stage that
/// have drawn a different number of times have diverged, whatever else agrees.
const RANDOMS_DRAWN: usize = 0x04;

/// The generator, whose whole state is the word at 0x49fe20 — which is what `State::random_seed` is.
///
/// 0x431870 is the step, called on that address from some twenty places: it takes the word, xors it with
/// 0x9630, subtracts 0x6553, rotates the result left by two — `(x >> 14) + x * 4` masked to a word — writes
/// it back and adds one to the dword at `+0x4`, then answers the word. So the word is the whole state and
/// `+0x4` is a count of how many numbers have been drawn; 0x4318d0 above it makes a dword out of two steps.
///
/// **A replay says the same thing from the other side**: a stage of one being played writes the word at
/// `+0x20` of that stage's own record straight into this address at 0x443910, and a stage beginning copies
/// this address into the object at 0x442c76 and puts the draw count back to nothing. A game that reproduces
/// a stage by restoring this word and nothing else is a game whose only randomness is this word.
const G_RANDOM: usize = 0x0049fe20;

/// The two flags a menu of the game's own raises, which is what stops a run: the update at 0x42d8d5
/// refuses to do anything at all while either is set — `[+0x93dd]` at 0x42d8e1 and `[+0x93dc]` at 0x42d8f3,
/// each `jne` straight to its own tail.
///
/// The second is the pause menu's: the same update raises it at 0x42d958 on the frame the menu button goes
/// down, which it works out by masking the word at 0x4b9e4c against 0x8 and comparing that against the
/// word at 0x4b9e54 — this frame's input against last frame's. 0x8 is the menu button in the same
/// numbering [`PAD_MAPPING`]'s fourth entry names, which is how the two readings meet.
///
/// What raises the first is not read yet. Both are here because `State::paused` is the pair: a run that is
/// not being updated is one nothing of orb's should act on, whichever menu is up. **And what stops the run
/// under them is the answer that update gives** — see [`STAGE_FRAMES`], where the same 3 that stops the chain
/// walk skips both clocks.
///
/// **Two bytes of the manager's own head are not this pair and do not belong in it.** `[+0xd]` is the frame
/// the game did not advance the run on — its own slowdown sets it at 0x42e19a and clears it at 0x42e108 —
/// which is a frame where neither clock moved and no job below the manager ran, so what orb reads there is the
/// frame before it read again, and `resume::Record::note` is written for exactly that. And `[+0xc]`, which the
/// player's update returns straight away on at 0x441fb7, is read at five addresses and written at none: a
/// switch nothing in this build turns on.
const GAME_MENU_UP: usize = 0x93dc;
const OTHER_MENU_UP: usize = 0x93dd;

/// The stage a run is on, **counted from one while that stage is being played**, where orb counts stages
/// from zero everywhere — so what orb reads is this less one, the way 紅魔郷's own 1-based field is read.
///
/// **The switch at 0x407081 is what says which it is**: it takes the word, subtracts one at 0x40708d, bounds
/// that at 7 and jumps through a table of eight at 0x4073e9 — six stages, the Extra and the Phantasm. The
/// subtraction is the whole of the evidence, and the menu paths agree with it from the other side: a normal
/// start writes nothing here (0x459ebb), the Extra and the Phantasm write 6 and 7 (0x459ec4), a practice run
/// writes the 0-based stage its menu chose (0x45b460), and then the build at 0x42e83e adds one at 0x42ef50
/// before loading `data/ecldataN.ecl` by the result out of the table at 0x49f560 — index 1 for `ecldata1`,
/// index 7 for the Extra's `ecldata7`. So the word is 0-based only until the stage is built, and 1-based for
/// every frame orb reads it on. The teardown at 0x43805f takes the one back off.
const G_STAGE: usize = 0x0062f85c;
/// And what orb counts from: the stage the word says, less the one the build added.
const STAGE_COUNTS_FROM: i32 = 1;
/// How many stages this game has, which the switch at 0x407081 fixes: it bounds the word less one at 7 and
/// jumps through a table of eight — six stages, the Extra and the Phantasm. A number the game fixes rather
/// than one this tree keeps in step.
const STAGES: i32 = 8;

/// What kind of run this is, as a bitfield of its own: 0x62f648, read in some ninety places.
///
/// **Bit 3 is a replay being watched.** 0x455b45 is where a replay is loaded — the file named at 0x62f654
/// read through 0x431330 and taken through 0x4433b0 — and it sets this bit from `(1 & 1) << 3` before
/// filling the run out of that replay's header. The pause menu at 0x403ba1 reads the same bit to send a run
/// being watched back to scene 7 rather than scene 6.
///
/// **Bit 0 is a practice run**, and the title menu is what says so: its eight items dispatch through the
/// table at 0x45622d, and of the four that start something, only the third sets this bit — 0x455d93, against
/// 0x455cec, 0x455e4c and 0x455eea, which each clear it. Third of that menu is Practice Start, under Start
/// and Extra Start and over Replay. The start paths read the bit to leave the stage and the difficulty
/// alone: 0x459e9a and 0x43805f both skip choosing them where it is set, a practice run having chosen both
/// already, and the build gives such a run eight lives rather than the configured stock — see [`LIVES`].
///
/// **Bit 1 is the attract demo**, and what the demo opens is what says so. The title screen's idle count at
/// `[menu+0xd100]` passes 0x384 at 0x455a94, picks the next of three names out of the table at 0x49ea7c by
/// the byte at 0x62f64e, and goes on into the same load a replay uses — so a demo has bit 3 set as well and
/// this bit is what tells the two apart. 0x443571 turns the bit into the load's own argument, 1 where it is
/// clear and 0 where it is set: the demo's replays are `data/demo/demorpy0.rpy` and its two neighbours,
/// inside `th07.dat`, and a player's are files under `replay/`. The update at 0x42d905 reads the bit and
/// refuses to open the pause menu on such a frame, 0x4018e0 runs a block only on it, and the title screen
/// clears it at 0x45c4b0 as it comes back.
///
/// **The word is the game manager's `+0x93d8`** as well, which is how those three read it: 0x626270 +
/// 0x93d8 is 0x62f648, and both spellings are the same word.
///
/// Bits 2 and 4 are not written down here because they are not read yet — see
/// `docs/todo/th07-the-run.md` for where each is set.
const G_RUN_KIND: usize = 0x0062f648;
const RUN_IS_A_PRACTICE: u32 = 1 << 0;
const RUN_IS_A_DEMO: u32 = 1 << 1;
const RUN_IS_A_REPLAY: u32 = 1 << 3;

/// The two scenes a run is, and the one of them a run is being *played* in.
///
/// 0x42f3c5 registers the game manager for scene 2, and its added callback at 0x42e83e branches at
/// 0x42e894 on the scene wanted being 3 and skips the whole build there — which is one manager built for
/// 2 and re-initialised for 3, the shape a stage transition has. So both are a run in progress and the
/// first is the one whose frames are played.
const SCENE_RUN: i32 = 2;
const SCENE_RUN_AGAIN: i32 = 3;

/// And the scene an ending is, which is 9: the transition into it runs the run's own teardown at 0x42f45d and
/// then 0x41e820, and that function is the ending's register — it allocates the object, hangs a calc element
/// with [`ENDING_UPDATE`] as its callback on the chain at priority 4 and a draw element beside it, and its
/// deleted callback at 0x41e790 asks for scene 6 as it frees the script.
const SCENE_ENDING: i32 = 9;

/// The ending's update, which is how its object is reached: the register puts the object in the element's
/// argument and nowhere else, so orb walks the chain for this callback the way it does in 紅魔郷.
const ENDING_UPDATE: usize = 0x0041d2f0;

/// And the script that object is running, `+0x24d4`: what the load at 0x41e4d2 wrote there, which is the file
/// the ending was read from.
///
/// **An ending is one file per part of itself**, and this changing is the only mark of a part ending: the
/// choice of file is made at 0x41e73f out of two tables — `[0x49f628 + character*4]`, whose names are
/// `end00b.end` and its two neighbours, where `[run+0x20]` says a continue was used, and
/// `[0x49f634 + shot*4]` otherwise. Which is a second reading of `[run+0x20]` as the continues used: a run
/// that continued gets 妖々夢's other ending.
const ENDING_SCRIPT: usize = 0x24d4;

/// The front end, as its own update and the two fields orb asks it about.
///
/// 0x45c632 is the register: it makes a calc element with 0x4554d6 as its callback at priority 3, a draw
/// element with 0x45bd6c at priority 0, and hands the object to each — so the object is reached the way the
/// ending's is, by walking the chain for that callback.
///
/// **`+0xd0f8` is which of its screens is up.** The update dispatches on it, bounding it at 0xe and taking a
/// byte out of the table at 0x4555ce as the index into the jump table at 0x4555aa — and the entry for 0 is
/// the only one that reaches 0x4555dd, the screen whose own dispatch is the title menu's eight items. So 0 is
/// the title and every other value is a screen orb has no question for.
///
/// **`+0x0` is the cursor**, which that dispatch reads at 0x455cc8 on the frame a decide is pressed and jumps
/// through the table at 0x45622d by: 0 Start, 1 Extra Start, 2 Practice Start, 3 Replay, 4 Score, 5 Music
/// Room, 6 Option, 7 Quit. Which of them are which is [`RUN_IS_A_PRACTICE`]'s evidence as well — the third is
/// the one that sets that bit.
/// **And `+0xd0fc` is how long the screen has been up, which is what decides whether a press is taken at
/// all**: 0x455c6d compares it against 0xa and leaves the whole update where it is under — so the ten frames
/// after the title menu appears take nothing, and orb's own question must not be answered on them either. The
/// title's phase at `+0x8` is the other half, and **the items are read in phase 1**: 0x4555f8 branches on it —
/// 0 to 0x455621, which is the frame the screen arrives on, where it starts the title's own song from slot 8 and
/// then sets 1; 1 to 0x45599d, where the cursor moves and the decide is read; and 2 and 3 to 0x45612e and
/// 0x456184.
///
/// **Phase 0 was written here first, and it cost every question orb has over that menu.** A press is only held
/// back where the phase is the items', so with 0 the hold never happened: no launch of 妖々夢 ever put the mode
/// question up, while every e2e test over it passed — the laid-out game wrote the phase orb expected. What said
/// so in the end was a launch: `front end: object 0x0a4a2bf8, screen Some(0), item Some(2), phase Some(1), up
/// for Some(84) frame(s)`, from `Th07::says_what_it_reads_of_the_front_end`. The laid-out game writes its own
/// number now — see `image`'s `TITLE_ITEMS_PHASE`.
const FRONT_END_UPDATE: usize = 0x004554d6;
const FRONT_END_SCREEN: usize = 0xd0f8;
const FRONT_END_SETTLED: usize = 0xd0fc;
const FRONT_END_PHASE: usize = 0x08;
const FRONT_END_CURSOR: usize = 0x00;
const SCREEN_TITLE: i32 = 0;
const TITLE_PHASE_ITEMS: i32 = 1;

/// **The front end's fifteen screens are three flows of three**, and these are the two shot selects orb
/// answers on.
///
/// The byte table at 0x4555ce is `[0, 8, 1, 2, 3, 4, 5, 6, 3, 4, 5, 7, 3, 4, 5]`, so screens 4, 8 and 12
/// reach one handler, 5, 9 and 13 the next, and 6, 10 and 14 the third — a difficulty select, a character
/// select and a shot select apiece. Which flow is which is what each title item writes: Start writes 4 at
/// 0x455d20, Practice Start writes 8 at 0x455dc7 and Extra Start writes 0xc at 0x455e77. So the shot select
/// is 6 for a whole run, 10 for a practice run and 14 for the Extra — and a practice run is the one the trait
/// says to answer nothing for, its stage being part of which run it is and chosen on the screen after.
const SCREEN_SHOT_SELECT: i32 = 6;
const SCREEN_SHOT_SELECT_EXTRA: i32 = 14;
/// The phase that screen reads its cursor and its presses in, which is what says a press there is one the game
/// will act on.
///
/// Its handler at 0x459518 dispatches on the phase and has two: phase 0 builds the screen and 0x459b80 leaves
/// it there, and every frame after that reaches 0x459bbb, which compares the count of frames the screen has
/// been up against 0x1e and writes this phase at 0x459bc7. The block that phase dispatches to — 0x459bd3
/// through 0x45a1a5 — is the whole of the screen's own reading: 0x459e7d writes the shot under the cursor as it
/// moves, and 0x459ead is where a decide copies the difficulty into 0x626280 and starts the run.
///
/// So the thirty frames are the gate and this is what they leave, which is why the count is not compared
/// against here as it is for the title menu: the phase does not go back, the screen being torn down where the
/// run is built.
const SHOT_SELECT_PHASE_ITEMS: i32 = 1;

/// The difficulty the select before it left, which is **not** [`DIFFICULTY`] yet: 0x459ead is where the shot
/// select's own decide copies this byte into 0x626280, so until the press it is the only place the choice is.
const G_DIFFICULTY_CHOSEN: usize = 0x00575a89;

/// Where the shot select's own writing about the difficulty begins, in the game's 640x480 output: 驚異 at the
/// top of it, `Lunatic Level` under that and 奇特な方向け under that again, ending around y=474.
///
/// **Measured off a screen and not off the exe**, which is the only place it can be read: what lays those
/// strings out is the vms the screen builds, and no constant in the exe says where they land. The way to read it
/// is a screen with orb's own mark on it, whose two lines are at heights this code chose: the gap between them
/// is [`crate::menu_ui::LINE_HEIGHT`] and gives the scale the picture was taken at, and the difficulty's first
/// line is measured against that.
///
/// Read twice. The first reading, off a mark at y=24 and y=48, put it at 416 and was **16 pixels high** — the
/// mark went a line above 416 and still touched 驚異. The second, off that mark at y=368 and y=392, puts it here.
/// So the number is good to a few pixels rather than exact, and [`Th07::mark_corner`] keeps two lines clear of it
/// rather than one.
const DIFFICULTY_LABEL_TOP: f32 = 400.0;

/// And the character and the shot **as a press settled them**, which is not the same as what the cursor is on:
/// 0x4593b7 and 0x459475 write the first, 0x459e7d and 0x459f72 the second, and every one of the four is inside
/// a press handler — the first of each pair under the decide mask `0x1001` and the second under the cancel mask
/// `0xa` — copying `[front+0x0]`, the cursor itself, into the byte.
///
/// **So which of them orb may read depends on which screen it is standing on.** The character's screen has been
/// left by the time the shot select is up, so its byte is that choice; the shot's has not, so its byte still
/// holds whichever shot the run before settled on — and a 妖々夢 session was offered 霊夢B's unfinished run with
/// the cursor sitting on 霊夢A. What [`Th07::run_pointed_at`] reads for the shot is the cursor.
const G_CHARACTER_CHOSEN: usize = 0x0062f645;
/// **Only a laid-out game needs the shot's own byte**, which is why it is behind that gate: orb reads the cursor
/// on that screen and [`G_SHOT_COMBINED`] once a run is running, so the one thing left to do with this address
/// is to hold a decided shot for a test to prove orb is not reading it.
#[cfg(any(test, feature = "sim"))]
const G_SHOT_CHOSEN: usize = 0x0062f646;
/// The frames the screen has to have been up for, which is the number that comparison is against.
const SETTLED_FRAMES: i32 = 0xa;
const TITLE_ITEM_START: i32 = 0;
const TITLE_ITEM_EXTRA: i32 = 1;
const TITLE_ITEM_PRACTICE: i32 = 2;
const TITLE_ITEM_SCORE: i32 = 4;

/// And the scene the front end is, which is the one a run is entered from: 0x437ee3 is the case for leaving
/// scene 1, and its own switch on the scene wanted is what registers scene 2 for a run, 5, 8 and 9.
const SCENE_FRONT_END: i32 = 1;

/// The screen a run ends on, and how orb tells a run that *ended* from one that was left.
///
/// Scene 6 is that screen: leaving the run at 0x43803c tears the manager down and calls its register,
/// 0x44a302, with 1 in `ecx` — and 0x42e646 calls the same register with 2. **The register writes that number's
/// meaning into the object's `+0x8`**: 0xa where a run ended and 0x12 where the run that ended was a practice
/// one (bit 0 of [`G_RUN_KIND`]), both written by kind 1 at 0x44a372 and 0x44a37e. The ranking asked for from
/// the front end is kind 0 (0x437f3a, `xor ecx,ecx`), which writes no phase at all and leaves whatever the
/// constructor put there. So the phase says which way the screen was brought up, and a run given up through
/// [`Th07::leave_run`] never reaches it at all.
///
/// **0x13 is none of those three**, and it is worth saying because it reads like a fourth way up: it is the save
/// (kind 2), whose object is never on the chain — see [`SAVE_KIND_WRITES`], and
/// [`Th07::its_records_are_about_to_be_written`], which is the one thing that number is read for.
///
/// The object is reached the way the ending's and the front end's are: its register hangs it on the chain at
/// 0x44a3a6 with 0x445db3 as the callback and keeps its address nowhere else.
const RESULT_UPDATE: usize = 0x00445db3;
const RESULT_PHASE: usize = 0x08;
const RESULT_FROM_A_RUN: i32 = 0xa;
const RESULT_FROM_A_PRACTICE_RUN: i32 = 0x12;
/// And the phase the other way in leaves: the ranking asked for from the front end, which 0x44a38d writes.
/// And the two states of that screen orb writes or waits on: **0 is the one its own build runs in** — the
/// state a screen asked for by the Score item starts in, whose handler at 0x445dfa lays the rows out — and
/// **2 is the one it leaves in**, whose handler at 0x44633f waits for the timer at `+0x4` to reach 0x3c and
/// then puts the scene back to [`SCENE_FRONT_END`]. The screen's own menu enters 2 the same way, at 0x44631e
/// and 0x447463, zeroing that timer as it goes.
///
/// Which is why leaving is two questions: [`Th07::showing_ranking`] is the screen being up and past its build,
/// and [`Th07::ranking_scene`] is the scene still being it — a second of drawn frames after the state changed.
const RESULT_BUILDING: i32 = 0;
const RESULT_LEAVING: i32 = 2;
const RESULT_TIMER: usize = 0x04;

/// And the phases that screen offers to save a replay in: 0xb to 0xf, which is what its own jump table at
/// 0x445dd4 sends to one handler, 0x447198.
///
/// **That handler is the replay's**, which is what says so: 0x4475a2 and 0x447c35 are where it makes the name
/// of the file — `"./replay/th7_%.2d.rpy"` at 0x4967bc — and the phases either side of the group are not it.
/// The walk into them is the score's own name entry at 0xa, then 0x10 and 0x11, then 0xb at 0x44808d.
///
/// What [`Th07::skip_replay_prompt`] writes instead is [`RESULT_LEAVING`] with the timer at nothing, which is
/// what the screen's own two ways out of the group write: 0x447463 and 0x44752b.
const RESULT_OFFERING_A_REPLAY: std::ops::RangeInclusive<i32> = 0xb..=0xf;

/// The scene the ranking is shown on, which is what the title menu's Score item asks for: `[0x575aa8] = 5` at
/// 0x455f95. The same register builds it as builds the screen a run ends on — 0x44a302, with nothing in `ecx`
/// rather than 1 — so what tells the two apart is the scene, and the state that register leaves.
const SCENE_RANKING: i32 = 5;

/// The three of the game's own that a practice-score read is made of, and where those scores live.
///
/// **0x444c20 reads the file** — `__fastcall` with the name in `ecx`, answering the buffer it read. **0x4452f4
/// parses the `PSCR` records out of it** into a table given in `edx`: it lays every entry out empty first and
/// then copies each record the file holds over its own, indexed `shot * 24 + stage * 4 + difficulty` at 0x18
/// bytes apiece — six shots, six stages and four difficulties, which is 妖々夢's practice without the Extra.
/// **0x4454fc gives the buffer back.**
///
/// The table is **0x62e760**, which all three of the game's own callers pass: the run's build at 0x42e757, the
/// ranking screen's read at 0x44a081, and the front end's at 0x45bfee. The two tables beside it in those same
/// calls are the cards' at 0x626288 and the high scores' at 0x62e6b8.
///
/// Handed over for `handed_over!`'s own reason. The parse is the piece orb could not do for itself without
/// writing this game's file format down, and it is the only entry point that touches `PSCR` and nothing else —
/// which is what keeps the unlocks the front end lights its items from out of the way.
const READ_SCORE_FILE: usize = 0x00444c20;
const PARSE_PRACTICE_SCORES: usize = 0x004452f4;
const RELEASE_SCORE_FILE: usize = 0x004454fc;
const G_PRACTICE_SCORES: usize = 0x0062e760;

/// The front end's own added callback, which is the read that takes what its menus *offer* out of the score
/// file: 0x45c4c8, a `__thiscall` on the front end that calls 0x45bf15.
///
/// **That read is the one open of the file that is not the mode's**, which is what this seam is for: it fills
/// the practice menu's own stages, the shots that are unlocked and the Extra, and none of those is a record of
/// a run — a stage that has been reached has been reached. 0x45bfd1 is the read inside it, with the three
/// parses after it at 0x45bfe1, 0x45bfee and 0x45bffb, and it is the same three [`READ_SCORE_FILE`] names.
///
/// Its register hangs it off the element at 0x45c662, beside the update at 0x4554d6 — which is what says it is
/// the front end's added callback and not another read of the file.
const FRONT_END_ADDED: usize = 0x0045c4c8;
/// `push ebp; mov ebp,esp; push ecx; mov [ebp-0x4],ecx` — position-independent, 7 bytes, read out of the exe
/// at that address.
const FRONT_END_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x51, 0x89, 0x4d, 0xfc];

/// The game's own save of its score file: `0x44a302(kind)`, the kind in `ecx`, which allocates the score-file
/// object and — for **kind 2** — sets its phase to 0x13 and gathers, reads and writes the file inline.
///
/// **The one write in the exe, and the only kind that reaches it.** 0x44a1f9 is the write's one caller and is
/// gated on that phase being 0x13 (0x44a1e0); the two kinds the ranking screen is built with are 0xa and 0x12,
/// which write nothing at any point. Kind 2 has two callers of its own — 0x434277, after the message pump ends,
/// and 0x42e646 inside [`GAME_MANAGER_ADDED`], which saves and then empties every record for the run about to
/// begin — so a run's counts are in memory alone until one of them runs.
///
/// What it writes is memory: 0x44552c serialises out of [`G_CARD_RECORDS`] itself (0x445713). And what it lays
/// over memory first is the file — see [`PARSE_CARD_RECORDS`] — which is why the counts a session made are put
/// on at that seam rather than before the call.
///
/// **And no address of orb's, which is why this is a doc rather than a constant**: orb calls that save nowhere,
/// so what it does with it is wait for one of the game's own. The call runs the screen's added callback and its
/// deleted one behind it, and the deleted one frees the anm files in the manager's slots 0x2a to 0x2d (0x44a25a)
/// before it cuts the screen's chain element. The game makes it where nothing is standing on those files; orb
/// making it with the front end up left that screen's vms on a file that had gone —
/// `crash: 0xc0000005 at 0x00450db1`, inside the walk of the 41 vms an update makes.
///
/// **So only a laid-out game needs the kind**, which is why it is behind that gate: what orb does with this save
/// is recognise the one the game made, by the phase below rather than by the kind, and a laid-out game is the
/// only thing here that is ever *passed* one.
#[cfg(any(test, feature = "sim"))]
const SAVE_KIND_WRITES: u32 = 2;
/// And the phase that kind leaves in the object, which is what the write is gated on (0x44a390, 0x44a1e0): the
/// two the ranking screen is built with are 0xa and 0x12 and write at no point.
const SAVE_PHASE_WRITES: i32 = 0x13;

/// The parse that lays the file's own card records over the ones in memory: 0x445069, taking the file's image in
/// `ecx` and the array in `edx`.
///
/// It walks the image's chunks from `[image+8]` for `[image+0x10] - [image+8]` bytes and, for every `CATK` whose
/// version byte at `+0x8` is 1 and whose card number at `+0x28` is under [`CARDS`], copies the whole 0x78-byte
/// chunk into `edx + card * 0x78` — `rep movs` of 0x1e dwords at 0x4450e8, the file into the array and never the
/// other way. **Three call sites pass [`G_CARD_RECORDS`] as that array**, which is what makes each of them a
/// place a session's records have to go back: 0x42e735 inside a run's build, after it has emptied them;
/// 0x44a059 inside a save, whose write follows; and 0x45bff3 inside the read the front end makes for its own
/// menus, which is the game's own file whatever the mode.
///
/// Called by orb as well as hooked, that third one being why: what the front end read is the game's own file's,
/// and the mode's own has to be laid over it again — see [`Th07::read_what_the_modes_file_records`].
const PARSE_CARD_RECORDS: usize = 0x00445069;
/// `push ebp; mov ebp,esp; sub esp,0x14` — position-independent, 6 bytes, read out of the exe at that address.
const PARSE_CARD_RECORDS_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x14];
handed_over!(
    PARSE_CARD_RECORDS_SLOT,
    parse_card_records,
    set_parse_card_records,
    PARSE_CARD_RECORDS
);

/// The read of the score file that fills the ranking, which is that screen's own added callback: 0x449b05,
/// reached the moment the chain adds the screen. It walks six characters by six difficulties by ten entries out
/// of the file — and the part of the file it parses *copies* the records it holds and leaves the rest as they
/// were, which is what [`Th07::forget_captures`] is about.
const RANKING_ADDED: usize = 0x00449b05;
/// `push ebp; mov ebp,esp; sub esp,0x5c` — position-independent, 6 bytes, read out of the exe at that address.
const RANKING_ADDED_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x5c];

/// The replay being recorded or watched, as a pointer at 0x4b9e48, and its header at `+0x4`.
///
/// **One object holds both halves of a replay**, which is what says it is the manager: the per-frame flags
/// at `+0xd6` are set from the run — bit 0 where a bomb goes off at 0x440bad, bit 2 where the player dies at
/// 0x440d63 — and `+0x4` is the header, read at 0x443a34, 0x44443a and 0x444482. 0x443040 is where that header
/// is built: 0xe8 bytes, with **seven stage records at `+0x1c`** and seven input buffers at `+0x38`, indexed
/// by [`G_STAGE`].
///
/// A record of nothing is a stage the replay does not cover, which is what [`Th07::jump_to_stage`] has to
/// answer false for rather than ask for.
const G_REPLAY_MANAGER: usize = 0x004b9e48;
const REPLAY_HEADER: usize = 0x04;
const REPLAY_STAGE_RECORDS: usize = 0x1c;
const REPLAY_STAGES: i32 = 7;

/// The chain's own element: the callback at `+0x4`, what it is called on at `+0x1c`, and the next element at
/// `+0x14`.
///
/// Read off the walk itself, [`RUN_CALC_CHAIN`]: it starts at the object it is called on, calls `[elem+0x4]`
/// with `[elem+0x1c]` in `ecx` where that callback is not null, and follows `[elem+0x14]`. So the head of the
/// calc list is [`G_CHAIN`] itself and the draw list's is 0x20 past it, which is where the two walks start.
const CHAIN_CALLBACK: usize = 0x04;
const CHAIN_NEXT: usize = 0x14;
const CHAIN_ARGUMENT: usize = 0x1c;

/// Longer than any chain this game builds: its jobs are one per priority from 0 to 0x10, and a chain whose
/// links do not end is a walk that answers nothing rather than one that does not end.
const CHAIN_LINKS: usize = 64;

/// Whether the fog is on, as that object keeps it: `[this+0x2bc]`, compared and written by both of the
/// two calls above.
///
/// **Written absolutely as well, and that is what the frame's own `mov DWORD PTR ds:0x575c0c,0xff` at
/// 0x434739 is**: 0x575950 + 0x2bc is 0x575c0c. Which is the whole of why that write is in the frame —
/// [`FOG_OFF`] reaches the device only where this field says the fog is on, so a frame that wrote
/// nothing here would tell the device once, on the frame after a stage turned fog on, and never again.
/// The frame writes it and then makes the call, so `D3DRS_FOGENABLE` is set false on every frame drawn
/// whatever the game last thought the state was.
const FOG_ON: usize = 0x02bc;
/// And what the frame writes there, which is neither of the two the calls write: 0xff. Anything but
/// zero would do — [`FOG_OFF`] compares this field against zero — and 0xff is what the frame writes.
const FOG_ON_FORCED: u32 = 0xff;

// All three together, because a frame that composed one of them and jumped into memory nothing has
// mapped for the next two is a frame no e2e test could reach at all. `handed_over!` is the parent
// module's — see it for why these are slots rather than the constants.
handed_over!(EMPTY_QUEUE_AT, empty_queue, set_empty_queue, EMPTY_QUEUE);
handed_over!(DRAW_QUEUE_AT, draw_queue, set_draw_queue, DRAW_QUEUE);
handed_over!(FOG_OFF_AT, fog_off, set_fog_off, FOG_OFF);

/// The game's own two music calls, `__thiscall` on [`G_SUPERVISOR`]: 0x43a05f takes the music down and
/// 0x439ec1 starts a slot of it.
///
/// **Both of them answer for two sound backends, which is why orb calls them rather than the sound itself.**
/// The byte at 0x575a87 says which this launch is making, and each of these branches on it: where it is 2 they
/// go through the object at `[0x575acc]` — 0x436b30 to stop, 0x436790 and 0x436ad0 to start — and where it is
/// 1 they command the music thread through 0x44d2f0, with 4 and 3 to stop and 2 to start. A launch's own
/// setting decides, so orb asking the game is the only way to be right in both.
///
/// **Neither of them has done anything by the time it returns on backend 1.** 0x44d2f0 writes the command
/// into the queue at `+0x73c` of the player and nothing more, and what runs a queued command is
/// [`PLAY_SOUNDS`] — which the game's own frame calls once, and which the stage's build calls in a loop of its
/// own at 0x42f238 rather than leave its music for the next frame. So each of orb's two calls below runs it
/// too: a restore has one frame to take the music down, put the memory back and put the song where the
/// chapter had it, and a command still sitting in that queue is none of those three having happened yet.
const STOP_MUSIC: usize = 0x0043a05f;
const START_MUSIC: usize = 0x00439ec1;
/// `SoundPlayer::PlayBGM`, `__thiscall` on the player, taking the name of the track to play — which it copies
/// into `+0x38bc` (0x44c041) before it takes the current stream down, works a buffer size out of the file's own
/// format, makes the event and the thread, and hands all of it to 0x45c8e0.
///
/// **The only call that makes a stream**, which is why a restore needs it. The game's own play at 0x439ec1
/// queues command 2 instead, and that command reuses the stream the player's own field names: it stops the
/// buffers that stream owns (0x44cc23), points its source at the record the slot names (0x44cce1) and fills its
/// buffer (0x44cd1d). None of that makes anything, so a play asked for after the music has been taken down —
/// which frees the stream and releases its buffer, 0x44cebd — is a play walking whatever the memory copy left
/// in that field.
const PLAY_BGM: usize = 0x0044c020;

/// **And the name [`PLAY_BGM`] takes is the file every track is in**, not a track's: the game's own two calls
/// hand it the string at 0x496fac, `thbgm.dat` (0x438cdf and 0x43aa10 — the other backend gets `th07.dat` at
/// 0x497150). 0x45c8e0 makes one streaming sound over that file, and which track is playing is the record its
/// source is pointed at — which is what the game's own play at 0x439ec1 is for.
///
/// The game's own string rather than one of orb's, so that what it is handed is what the game hands it.
const MUSIC_FILE: usize = 0x00496fac;
/// And the thread the music is streamed on, its handle, and the event it waits on — all three made by
/// [`PLAY_BGM`] and closed by the take-down at 0x44cebd, and all three carried in the player's own memory. So
/// all three come back with a chapter, naming handles the take-down has closed.
const MUSIC_THREAD_HANDLE: usize = 0x618;
const MUSIC_EVENT: usize = 0x39c0;

/// **The two songs a stage's own data names, and the slots its build loads them into**: its own at
/// `[0x1347f9c]+0x290` into slot 0 (0x42f1d5, played at 0x42f216), and the fight it ends with at `+0x310` into
/// slot 1 (0x42f1ec).
///
/// These two are what a track is held against rather than the slots' own contents, and a 妖々夢 session is why:
/// a chapter at a stage's start is taken frames into the stage — `stage 6 chapter 1 (stage start) at frame 9` —
/// and what was playing then was `th07_01.wav`, the title screen's, which the front end had left **in slot 8**.
/// A slot did hold it, so asking for the slot the track was in asked for slot 8 and the title screen's song is
/// what came back. What the stage's own data names is the stage's music whatever is playing.
///
/// The names there are the `.mid` ones, `bgm/th07_12.mid` and its like, where a slot holds the same name with
/// `wav` written over it (0x439e92) and a record holds the file's own name with no directory. So what the three
/// are compared by is the stem — see [`file_stem`].
/// **And a slot is what the game's own play at 0x439ec1 takes, which is why the two above are the whole of the answer.** The
/// load at 0x439dd0 copies the name it is given, writes `wav` over its extension (0x439e92) and sends command 1
/// with the slot and that name, which 0x44c242 copies into a 0x100-byte buffer of that slot's own at `+0x28bc`
/// of the player. The play, command 2, reads that buffer back (0x44cc95), looks the name up in the record array
/// at `[player+0x738]` (0x34 apiece — see [`RECORD_NAME`]) and points the stream's one source at the record it
/// finds (0x44c1b0, then 0x45e130). A name in no record is record 0 (0x44bceb), so a slot nothing was ever
/// loaded into plays the first track in `thbgm.dat` rather than nothing at all — which is what asking for slot
/// 0x1f did: 0x1f is where the *other* backend's object keeps a named file (0x4369c0), the music thread has no
/// such slot, and the first track of that file is the title screen's song.
const G_MUSIC_TABLE: usize = 0x01347f9c;
const STAGE_SONG_NAME: usize = 0x290;
const FIGHT_SONG_NAME: usize = 0x310;
/// How much room a name there has, which is the distance between the two: entries of that table are 0x80 apart.
const SONG_NAME_BYTES: usize = FIGHT_SONG_NAME - STAGE_SONG_NAME;
const STAGE_SONG_SLOT: u32 = 0;
const FIGHT_SONG_SLOT: u32 = 1;

/// `th07::GameManager::Reseal`, `__thiscall` on [`G_GAME_MANAGER`] taking nothing: what makes a changed
/// number pass the check at 0x404fe0 instead of wrecking the game — see [`DEATHS`] for what failing it does.
///
/// **It costs two draws of the generator**, which is why it is written down here rather than treated as a
/// bookkeeping call: 0x4012b0 asks 0x4318d0 on [`G_RANDOM`] twice, takes each modulo 0x186a0 and salts it by
/// 0x198f into `[run+0x3c]` and the field beside it, and 0x404fe0 recomputes `[run+0xac]` and `[run+0xb0]`
/// from those two and four more. So anything of orb's that reseals moves the generator on, and a stage
/// resumed from a recording has to have its seed put back *after* the last reseal — which is exactly the
/// order the game's own replay restore uses at 0x443790: the numbers with their reseals first, the seed into
/// 0x49fe20 last, at 0x443910.
///
/// Handed over for `handed_over!`'s own reason: it is code, and a laid-out game has none.
const RESEAL: usize = 0x004012b0;

/// `th07::AnmManager::DrawStringFormat`, `__cdecl` taking the manager, the vm the string is baked into, a
/// colour, a second colour, and the text as a format string.
///
/// The one place a spell card's name reaches the screen: 0x428887 calls it at 0x428a78 with the five arguments
/// [`Th07::redraw_card_name`] passes — what [`G_QUAD_QUEUE`] holds, the vm at [`CARD_NAME_VM`],
/// [`CARD_NAME_COLOUR`], zero, and the name. Nothing keeps the string afterwards: the vm holds a baked sprite,
/// and the caller only walks the name to size the plate from its length (0x428a80). Which is why a restore has
/// to bake it again.
///
/// **The whole of it is a format string**, `vsprintf` at 0x4543f2 into 0x50 bytes of its own stack: what the
/// game hands it is a name out of its own ECL, and what orb hands it is the copy of that name the game itself
/// made in the card's record.
const DRAW_STRING: usize = 0x004543b0;

handed_over!(RESEAL_AT, reseal, set_reseal, RESEAL);
handed_over!(DRAW_STRING_AT, draw_string, set_draw_string, DRAW_STRING);
handed_over!(BORDER_UP_AT, border_up, set_border_up, BORDER_UP);
/// What [`Th07::says_what_it_reads_of_the_front_end`] said last, so that it says it once per change.
static SAID_OF_THE_FRONT_END: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);

handed_over!(STOP_MUSIC_AT, stop_music_call, set_stop_music, STOP_MUSIC);
handed_over!(
    START_MUSIC_AT,
    start_music_call,
    set_start_music,
    START_MUSIC
);
handed_over!(PLAY_BGM_AT, play_bgm_call, set_play_bgm, PLAY_BGM);
handed_over!(
    PLAY_SOUNDS_AT,
    play_sounds_call,
    set_play_sounds,
    PLAY_SOUNDS
);
handed_over!(
    READ_SCORE_FILE_AT,
    read_score_file,
    set_read_score_file,
    READ_SCORE_FILE
);
handed_over!(
    PARSE_PRACTICE_SCORES_AT,
    parse_practice_scores,
    set_parse_practice_scores,
    PARSE_PRACTICE_SCORES
);
handed_over!(
    RELEASE_SCORE_FILE_AT,
    release_score_file,
    set_release_score_file,
    RELEASE_SCORE_FILE
);

/// The size the game renders at.
///
/// `0x280` and `0x1e0` written into the shared `D3DVIEWPORT8` at [`VIEWPORT`] over the whole output,
/// at 0x4347ad inside [`RENDER`] and again at 0x448d70, and set through +0xa0 each time.
const BACK_BUFFER: (u32, u32) = (640, 480);

/// The `D3DVIEWPORT8` the game builds its viewports in, which is one struct reused: the whole output
/// at 0x4347ad, and whatever else asks. Written here as well as handed to the device, so that a
/// viewport orb sets is one the game's own next read of it agrees with.
const VIEWPORT: usize = 0x00575a18;

/// Whether the game is to make a window rather than take the display, as its own configuration keeps it: the
/// byte at 0x575a8a, which the option screen at 0x4569cf turns over between nothing and one and the load at
/// 0x4399c5 leaves at nothing.
///
/// **Two readers make it the windowed flag rather than any other option.** The window creation branches on it
/// at 0x434ae4 for the style and the size it asks `CreateWindowExA` for, and the device creation at 0x434c06
/// takes the `D3DPRESENT_PARAMETERS` it builds down the path that writes `Windowed` — `+0x1c` of the struct at
/// `[ebp-0x4c]`, 1 at 0x434d12 — and a swap effect of copy rather than flip beside it. The struct itself is
/// then kept at 0x575a30, which is what the 13-dword copy at 0x434f5b is.
const CFG_WINDOWED: usize = 0x00575a8a;

/// `th07::GameWindow::CreateGameWindow`, `__thiscall` on the window it makes: the `WNDCLASSA` at `[ebp-0x30]`
/// filled and registered, and then `CreateWindowExA` with the style [`CFG_WINDOWED`] chose.
///
/// The startup at 0x434020 calls it at 0x4340dd, **after the configuration is read** — 0x4340ae is that read,
/// on the supervisor — and before the device is made at 0x4340eb. Which is what `Hooks::create_window` asks
/// for: the last moment a windowed flag can be changed and still be obeyed.
const CREATE_GAME_WINDOW: usize = 0x00434a80;
/// `push ebp; mov ebp,esp; sub esp,0x34` — position-independent, 6 bytes, read out of the exe at that address.
const CREATE_GAME_WINDOW_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xec, 0x34];

/// The play field, in the game's own 640x480 output.
///
/// The `D3DVIEWPORT8` built on the stack at 0x406ace — `0x20`, `0x10`, `0x180`, `0x1c0` — set through
/// +0xa0 at 0x406afc and cleared through +0x90 straight after it. The same rectangle 紅魔郷 has, read
/// off this exe rather than taken from that one.
const PLAY_AREA: Rect = Rect {
    left: 32.0,
    top: 16.0,
    width: 384.0,
    height: 448.0,
};

/// Where the game shows how many lives are left, in that output: one 16x16 mark per life from (496, 96)
/// rightwards, 16 apart.
///
/// **The loop at 0x42bc5f is where all four numbers come from.** It draws the vm at `GuiImpl+0x14ac` once per
/// life — the count is `[run+0x5c]`, a float, through the game's own float-to-int at 0x48b8a0 — from x at
/// 0x498c50 (496.0) by the step at 0x498a80 (16.0), with y at 0x498c54 (96.0). The bombs are the same loop
/// under it, from the same x with y at 0x498ca8 (112.0), which is what makes the row 16 tall: a mark of the
/// height the rows are apart, and nothing of orb's may reach the row below.
///
/// The mark's own 16x16 is `front.anm`'s sprite 10, at (64, 80) of a 256x256 sheet, and the position is its
/// top-left corner: the script that draws it (`front.anm`'s script 9, which the vm is given as global script
/// 0x609) is `sprite 10` and then the same corner-anchoring instruction the panel's own background tile runs.
/// **Which is what says the anchor is the corner**: that tile is laid from (416, 16) in 32s, and centred it
/// would start at 400 and cover the play field's own right-hand column.
///
/// **That much is not in the exe**, and it is the only line here that is not: `front.anm` is a member of
/// `th07.dat`, a `PBG4` archive whose file table and members are each LZSS with a 13-bit offset and a 4-bit
/// length, and inside it the sprites are `(id, x, y, w, h)` behind the entry header's own offset table. The
/// load at 0x428b91 is what ties the two together — `data/front.anm` into slot 0x15 with its scripts numbered
/// from 0x600 — and [`Th07::panel_tile`] needs none of it, reading the tile it draws with out of the game.
///
/// 144 wide, 496 to the right edge of the output, the way 紅魔郷's own row is: the count can reach 624 and the
/// band from there is the panel's, repainted with it — see [`Th07::repaint_lives_row`] for what repaints it.
///
/// **Which is also why `Game::room_around_the_lives_row` is left at nothing for this game**: the bombs are 16
/// under a row 16 tall, so a mark of orb's has this row and no panel either side of it. What is *above* the row
/// might be free — the score's own band, which the game may be repainting every frame anyway — and that is
/// unmeasured, so the answer here is the one that cannot paint over something the game will not put back.
const LIVES_ROW: Rect = Rect {
    left: 496.0,
    top: 96.0,
    width: 144.0,
    height: 16.0,
};

/// The face this game draws its own text in, which it has the machine make rather than shipping — the whole of
/// the ask it hands `CreateFontA` at 0x4322e7.
///
/// **It ships no font at all**, which is what makes this the answer to *what is 妖々夢's own face*: the only
/// text machinery in the exe is GDI's — `CreateFontA`, `CreateCompatibleDC`, `CreateDIBSection`, `TextOutA`,
/// one import each — and that one call site asks for `ＭＳ ゴシック` (the Shift-JIS name at 0x497cbc), weight
/// 0x2bc, charset 0x80, quality 4 and pitch 0x11, then bakes the string into a DIB it uploads. `ascii.anm`
/// inside `th07.dat` is the panel's own digits and not a face anything else is drawn in.
///
/// **And the height is carried, because the same family at another height is not the same text.** The pushes
/// before that call read, in the order the arguments are taken: `2 * n - 2` for the height, and it is
/// *positive*, so what is asked for is the whole cell rather than the em. The bitmap it draws into is twice as
/// wide and `2 * n + 6` tall — 0x432343's `shl $1` and 0x43233b's `lea 0x6(%eax,%eax,1)` — so the glyphs are
/// baked at twice the size the string is shown at and scaled back down, and the edges and stroke weights that
/// come of that are what 妖々夢's text looks like. A 妖々夢 session saw orb's screens beside it and said they
/// were not the same face; they were the same family baked at the size it is drawn at.
///
/// The two fields of the ask that are *not* carried are the pitch and the out precision: `FIXED_PITCH |
/// FF_ROMAN` and the default precision decide which face GDI substitutes where the family is missing, and
/// which one came back is the line the log already writes — see `overlay::loaded`.
///
/// **This is the family the exe names, and a thcrap launch is not measured.** thcrap patches the text a game
/// draws and can put its own face behind it, in which case the game's text on screen is in one face and orb's
/// screens in this one. What would settle it is asking through the game's own `CreateFontA` import, which a
/// patch has already hooked — see [docs/adr/0019](../../../../../docs/adr/0019-a-game-that-ships-no-font-is-drawn-in-the-machines-own-face.md).
const OWN_FACE: Face = Face {
    family: "ＭＳ ゴシック",
    bold: true,
    height: |size| size * 2 - 2,
    baked_at: 2,
};

/// The panel's own background, which is a grid of one 32x32 tile: laid from (416, 16), the loop at 0x42b6e8
/// stepping x by 32 from 0x498a74 (416.0) while it is under 0x498cb0 (624.0), and y by the same step from
/// 0x498a80 (16.0) while it is under 0x498b10 (464.0).
const PANEL_ORIGIN: (f32, f32) = (416.0, 16.0);
const PANEL_PITCH: f32 = 32.0;
/// The vm that grid is drawn with, `GuiImpl+0x1b90` — which is what orb takes the tile's texture and its
/// piece of the sheet out of, rather than writing either down: see [`Th07::panel_tile`].
const PANEL_TILE_VM: usize = 0x1b90;

/// A vm's own sprite, `+0x1e4` of it: a pointer to the sprite the script it is running last set.
///
/// The draw is where both this and the two below come from. 0x44f7c4 reads the sprite through this field for
/// the size it draws the quad at, 0x44f3bc takes `[sprite]` as the slot of the texture to set — the same
/// [`ANM_TEXTURES`] array `live_handles` walks — and 0x44f0c7 through 0x44f131 build the quad's own corners
/// out of four floats of it, the two the u of a corner comes from and the two the v does.
const VM_SPRITE: usize = 0x01e4;
const SPRITE_TEXTURE: usize = 0x00;
/// `[u0, v0, u1, v1]`, which is the order [`PanelTile::uv`] wants them in.
const SPRITE_UV: [usize; 4] = [0x1c, 0x20, 0x24, 0x28];

/// The word the panel keeps a row's own "paint me" in, `+0x4` of [`G_GUI`]: two bits per row, and the lives'
/// are the lowest two.
///
/// The draw tests each pair and paints that row where it is not zero — 0x42bc34 for the lives, and the bombs,
/// the graze and the point items at 2, 6 and 8 bits up — then takes one off every pair that is not zero, at
/// 0x42c489 through 0x42c570. So a pair set to 2 is a row painted on this frame and the next, which is what
/// the game's own writers set and what [`Th07::repaint_lives_row`] sets.
const GUI_ROWS: usize = 0x04;
const LIVES_ROW_PAINT: u32 = 0b11;
const PAINT_FOR_TWO_FRAMES: u32 = 0b10;

/// And how long the panel's own background has left to be painted for: 0x575ab4, counted down by one a frame
/// at 0x4346c9 and read by the draw at 0x42b685, which lays the whole grid again while it is not zero.
///
/// **Two writes and not one, because the row and the ground under it are two things here.** 紅魔郷 erases a
/// row with a bar of its own before it draws the count again, so asking for the row is the whole of it there;
/// this panel draws the marks and nothing else, and what takes the last frame's away is the grid being laid
/// again. A mark of orb's over a row whose ground nobody repainted is ink over ink until it is a shape of its
/// own — see `Game::repaint_lives_row` — so orb asks for both. Two is what every one of the game's own writers
/// of this field asks for, 0x427c71 and 0x428b0b among them.
const G_PANEL_REPAINT: usize = 0x00575ab4;

/// The panel's own draw job, which is the callback of the element the register at 0x42d20d hangs off the draw
/// list: 0x427f22, whose `[gui+0x8]` is what 0x42b603 lays the panel out of.
///
/// What it answers for orb is whether the game is going to paint that row *this* frame — a run being left is
/// not a run whose panel has gone, and the row the game paints last is the one left standing there.
const PANEL_DRAW: usize = 0x00427f22;
/// And the head of the draw list itself, 0x20 past the chain — see [`CHAIN_CALLBACK`] for where that is read.
const CHAIN_DRAW_LIST: usize = 0x20;

impl Th07 {
    /// The run's own numbers, or nothing where there is no run: the pointer at [`G_RUN_NUMBERS`] is null
    /// until the added callback allocated the object, and every read of a field of it goes through here.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_numbers(&self) -> Option<usize> {
        // The pointer is the game manager's own field, read outright for the reason [`Th07::lasers`] gives.
        let run = unsafe { mem::read::<usize>(G_RUN_NUMBERS) };
        (run != 0).then_some(run)
    }

    /// The boss being fought, or nothing where none is: the first of the manager's [`BOSS_SLOTS`] pointers
    /// that is not null, which is how 0x4232a0 answers the same question.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn boss() -> Option<usize> {
        // The slots themselves are the manager's own fields and are read outright, for the reason
        // [`Th07::lasers`] gives; what the pointer in one leads to is the allocator's and is asked about.
        (0..BOSS_SLOTS)
            .map(|slot| unsafe {
                mem::read::<usize>(G_ENEMY_MANAGER + BOSSES + slot * size_of::<usize>())
            })
            .find(|boss| *boss != 0)
    }

    /// The script the ending being played is running, found by walking the chain for [`ENDING_UPDATE`]: the
    /// ending's register hands its object to the element and keeps it nowhere else, so the chain is the only
    /// way to it.
    ///
    /// Every step is asked of the memory map rather than read outright, which is what makes it answerable
    /// with no game around it — and the walk is bounded by [`CHAIN_LINKS`], so a chain whose links do not end
    /// answers nothing rather than not ending either.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn ending_script() -> Option<usize> {
        let ending = unsafe { Self::chain_argument(ENDING_UPDATE) }?;
        mem::read_committed::<usize>(ending + ENDING_SCRIPT).filter(|script| *script != 0)
    }

    /// What a chain job's callbacks are called on, found by the callback it was registered for — which for
    /// the two objects orb wants that way, the ending's and the front end's, is the only place their address
    /// is kept: each register allocates the object, hands it to its element and puts it nowhere else.
    ///
    /// Every step is asked of the memory map rather than read outright, which is what makes it answerable
    /// with no game around it, and the walk is bounded by [`CHAIN_LINKS`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn chain_argument(callback: usize) -> Option<usize> {
        let elem = unsafe { Self::chain_element(G_CHAIN, callback) }?;
        mem::read_committed::<usize>(elem + CHAIN_ARGUMENT).filter(|it| *it != 0)
    }

    /// And the element itself, in the list at `head`: [`G_CHAIN`] for the one the update is walked from and
    /// [`CHAIN_DRAW_LIST`] past it for the drawing's.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn chain_element(head: usize, callback: usize) -> Option<usize> {
        let mut elem = mem::read_committed::<usize>(head + CHAIN_NEXT)?;
        for _ in 0..CHAIN_LINKS {
            if elem == 0 {
                return None;
            }
            if mem::read_committed::<usize>(elem + CHAIN_CALLBACK)? == callback {
                return Some(elem);
            }
            elem = mem::read_committed::<usize>(elem + CHAIN_NEXT)?;
        }
        None
    }

    /// The check byte of one card's record put right, which is the game's own sum recomputed: the name's bytes
    /// up to its terminator, then the seven captures, the seven attempts and the seven scores, low byte kept.
    ///
    /// # Safety
    /// Must run on the game's main thread, with `record` one of the game's own.
    unsafe fn seal_card_record(record: usize) {
        let mut sum = 0u32;
        // The terminator is the game's only bound — 0x40ffe5 walks the name for its length and 0x410027 sums
        // that many bytes — so every byte of the field a name is copied into is walked to find it.
        for byte in 0..CARD_RECORD_BYTES - CARD_NAME {
            let at = record + CARD_NAME + byte;
            match mem::read_committed::<u8>(at) {
                None | Some(0) => break,
                // Signed, the way the game's own `movsx` reads each one.
                Some(byte) => sum = sum.wrapping_add(byte as i8 as i32 as u32),
            }
        }
        for shot in 0..CARD_SHOTS {
            let captured =
                mem::read_committed::<u16>(record + CARD_CAPTURES + shot * size_of::<u16>());
            let attempted =
                mem::read_committed::<u16>(record + CARD_ATTEMPTS + shot * size_of::<u16>());
            let scored = mem::read_committed::<u32>(record + CARD_SCORES + shot * size_of::<u32>());
            sum = sum
                .wrapping_add(u32::from(captured.unwrap_or(0)))
                .wrapping_add(u32::from(attempted.unwrap_or(0)))
                .wrapping_add(scored.unwrap_or(0));
        }
        unsafe { mem::write::<u8>(record + CARD_CHECK, sum as u8) };
    }

    /// How many lasers are alive, which nothing in the game counts: the same walk of the same slots the
    /// game's own two make.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    /// **Read outright and not through the memory map**, which is what makes 0x40 reads worth what 0x40
    /// reads cost: every slot is inside the exe's own `.data`, which the loader commits whole, so there is
    /// nothing for the map to answer — and asking it is a `VirtualQuery` apiece. A launch of 妖々夢 spent
    /// **1206us a frame** in orb's own work against 紅魔郷's 18us with this walk going through the map, and
    /// a run picked up out of its file took 3.2 seconds to play 2667 frames of buttons in.
    unsafe fn lasers() -> i32 {
        (0..LASER_SLOTS)
            .filter(|slot| unsafe {
                mem::read::<u32>(G_LASERS + slot * LASER_BYTES + LASER_ALIVE) != 0
            })
            .count() as i32
    }

    /// The line of the dialogue being played, or nothing where the game has not built what it puts over a run
    /// at all — which is the first thing the game's own answer at 0x42ad66 asks, the pointer being null until a
    /// stage loaded one.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn dialogue_line() -> Option<i32> {
        unsafe { Self::over_the_run() }
            .and_then(|gui| mem::read_committed::<i32>(gui + DIALOGUE_LINE))
    }

    /// What the game puts over a run, or nothing where it has built none — see [`G_GUI`].
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn over_the_run() -> Option<usize> {
        // Another of the game's own statics read outright, and what it points at is asked about — see
        // [`Th07::lasers`].
        let gui = unsafe { mem::read::<usize>(G_GUI + GUI_IMPL) };
        (gui != 0).then_some(gui)
    }

    /// The card being fought and the record the game keeps of it: nothing where no card is up, and nothing
    /// where the slot is not one the game has written — which is what its `"CATK"` tag says.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn card_being_fought() -> Option<(usize, usize)> {
        let running: u32 = unsafe { mem::read(G_ENEMY_MANAGER + SPELLCARD_RUNNING) };
        if running == 0 {
            return None;
        }
        let card =
            usize::try_from(unsafe { mem::read::<u32>(G_ENEMY_MANAGER + SPELLCARD) }).ok()?;
        if card >= CARDS {
            return None;
        }
        let record = G_CARD_RECORDS + card * CARD_RECORD_BYTES;
        (mem::read_committed::<u32>(record)? == CARD_RECORD_TAG).then_some((card, record))
    }

    /// The stream the sound player is playing through, or nothing where it has none.
    ///
    /// The player's own field, read outright for the reason [`Th07::lasers`] gives — and this one is read on
    /// every frame of a fight, `the_fights_own_song_is_playing` being how a chapter learns which song it is
    /// under. What the field points at is the allocator's and is asked about.
    fn the_stream() -> Option<usize> {
        let stream = unsafe { mem::read::<usize>(G_SOUND_PLAYER + STREAM) };
        (stream != 0).then_some(stream)
    }

    /// The track the source is reading, as the name its own record carries — see [`RECORD_NAME`]. Nothing
    /// where no track is playing.
    fn the_track_playing() -> Option<Vec<u8>> {
        let stream = Self::the_stream()?;
        let source = mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)?;
        let record = mem::read_committed::<usize>(source + SOURCE_RECORD).filter(|it| *it != 0)?;
        name_at(record + RECORD_NAME, RECORD_NAME_BYTES)
    }

    /// What orb read of the front end this frame, said once per change of it.
    ///
    /// **Because a launch of 妖々夢 has never once put the mode question up**, and every e2e test over that
    /// question passes: the question is asked where [`Th07::menu_pointed_at`] answers `Menu::Run`, so what a
    /// log has to be able to say is which of the reads on the way there did not answer — the element the
    /// front end's own update is registered in, the screen it is on, the item under its cursor, its phase,
    /// and how long it has been up. A laid-out game is written from these same constants, so only a launch
    /// can say which of them is wrong.
    ///
    /// On change rather than per frame, the count of frames the screen has been up excepted: that one climbs
    /// every frame, so a line per change of it would be a line per frame.
    fn says_what_it_reads_of_the_front_end(front: Option<usize>) {
        let read = |at: usize| front.and_then(|front| mem::read_committed::<i32>(front + at));
        let (screen, cursor, phase) = (
            read(FRONT_END_SCREEN),
            read(FRONT_END_CURSOR),
            read(FRONT_END_PHASE),
        );
        let now = i64::from_le_bytes([
            front.is_some() as u8,
            screen.unwrap_or(-1) as u8,
            cursor.unwrap_or(-1) as u8,
            phase.unwrap_or(-1) as u8,
            0,
            0,
            0,
            0,
        ]);
        if SAID_OF_THE_FRONT_END.swap(now, Ordering::Relaxed) == now {
            return;
        }
        detail!(
            "front end: object {}, screen {screen:?}, item {cursor:?}, phase {phase:?}, up for {:?} frame(s)",
            front.map_or_else(
                || "not in the chain".to_owned(),
                |front| format!("{front:#010x}")
            ),
            read(FRONT_END_SETTLED),
        );
    }

    /// And the two songs the stage that was built has, out of its own data: its own and the one the fight it
    /// ends with brings — see [`G_MUSIC_TABLE`]. Nothing before a stage has been built.
    fn the_stages_own_songs() -> Option<(Vec<u8>, Vec<u8>)> {
        // The pointer is a static of the game's own and is read outright — [`Th07::lasers`] again, and this is
        // the other half of what a fight's own song costs per frame.
        let table = unsafe { mem::read::<usize>(G_MUSIC_TABLE) };
        let table = (table != 0).then_some(table)?;
        Some((
            name_at(table + STAGE_SONG_NAME, SONG_NAME_BYTES)?,
            name_at(table + FIGHT_SONG_NAME, SONG_NAME_BYTES)?,
        ))
    }

    /// The commands the game's own two music calls leave standing, carried out — which is what makes either of
    /// them the music having changed rather than a command in a queue.
    ///
    /// On the backend a launch of this game makes, [`STOP_MUSIC`] and the game's own play at 0x439ec1 write into the queue at
    /// [`SOUND_COMMANDS`] and return; [`PLAY_SOUNDS`] is what walks it.
    ///
    /// **Run until that queue is empty rather than once**, which is what the game's own stage build does at
    /// 0x42f238: one call finishes only the commands whose own step is their last — 0x44d0d5 goes back to the
    /// head for as long as it *handled* something, and a command that only advanced its phase returns instead
    /// (0x44ca96). So a take-down left to one call is a take-down that may not have happened when the memory
    /// copy after it writes over the stream's own object.
    ///
    /// Bounded, a queue that never empties being a hang rather than a wait: what says the work is done is the
    /// queue itself, and the bound only decides how long a game whose sound is stuck is given.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames: it is the game's own per-frame sound work.
    unsafe fn runs_the_sound_commands() {
        let run: unsafe extern "thiscall" fn(usize) =
            unsafe { std::mem::transmute(play_sounds_call()) };
        for _ in 0..RUNS_A_COMMAND_IS_GIVEN {
            unsafe { run(G_SOUND_PLAYER) };
            if !Th07.a_sound_command_is_in_flight() {
                return;
            }
        }
        log!(
            "music: the player's own queue is still not empty; leaving it to the frame's own sound"
        );
    }
}

impl Game for Th07 {
    /// Every seam 紅魔郷 has, less the two this game does not need — and each of those two is a reading rather
    /// than a gap: the replay write and the recording's own teardown are declined beside `save_replay` below.
    fn hooks(&self) -> Hooks {
        Hooks {
            update: Patch {
                target: RUN_CALC_CHAIN,
                prologue: RUN_CHAIN_PROLOGUE,
            },
            draw: Patch {
                target: RUN_DRAW_CHAIN,
                prologue: RUN_CHAIN_PROLOGUE,
            },
            // orb's own cadence, and a frame of input lag gone with it. What this replaces is
            // [`RENDER`], and what the two seams either side of the draw chain are for is the rest of
            // what that frame does — see [`Th07::begin_drawing`] and
            // [docs/adr/0017](../../../../../docs/adr/0017-the-frame-loop-has-a-seam-either-side-of-the-draw-chain.md).
            //
            // **This was `None` and a run is why**, which is the one thing worth carrying here: orb's
            // first loop over that frame took the game down on its first frame — `crash: code
            // 0xc0000005 at 0x0044f6aa in th07.exe+0x4f6aa`, `writing 0x00000000`, which is the
            // `rep movsd` at the top of the append at 0x44f690 copying a quad to where
            // [`EMPTY_QUEUE`] had never pointed anything. Every address that loop used was right. So
            // what a frame is is not a list of addresses, and the two seams are the frame's own
            // *order* rather than two more numbers.
            render: Some(Patch {
                target: RENDER,
                prologue: RENDER_PROLOGUE,
            }),
            // **Neither of these two is a gap, and both are readings.** The write of a replay file is
            // reached from one place in this game, the offer at the end of a run — which
            // [`Th07::skip_replay_prompt`] takes the game off before a frame of it is drawn, so there is
            // no write left to suppress. And nothing here finishes a record off at a stage's teardown:
            // 紅魔郷 needs that seam because its own `StopRecording` runs from the teardown and, during
            // playback, terminates the *replay's* record at wherever it had reached — 妖々夢's teardown
            // (0x42f347) cuts its three chain elements at 0x442b10 and writes no entry at all, and its
            // recorder (0x442cd0) refuses a frame outright where the run is a playback.
            save_replay: None,
            stop_recording: None,
            // The moment a stage's numbers have just been put in place, which is the one
            // [`Th07::set_run_state`] is written to be called after — and entered in `ecx`, this game's
            // chain calling every callback as `mov ecx,[elem+0x1c]; call [elem+0x4]`.
            stage_begun: Some(Entered::InEcx(Patch {
                target: GAME_MANAGER_ADDED,
                prologue: GAME_MANAGER_ADDED_PROLOGUE,
            })),
            // **And the other half after all, which a launch settled.** This was declined for a stage's worth
            // of reasoning — 妖々夢 takes its stage's seed *during* the build, so the seed could go in where
            // the numbers go in — and a run picked up out of its file parted from that run at the stage's
            // first frame all the same: the numbers go in at the *end* of the build, which is after the
            // build's own draws. See [`STAGE_BUILD`], and [`crate::resume::stage_building`] for what is
            // written there.
            stage_building: Some(Patch {
                target: STAGE_BUILD,
                prologue: STAGE_BUILD_PROLOGUE,
            }),
            // The one read of the score file that is not the mode's: what the front end offers is not a
            // record of a run, so it comes out of the game's own file whichever mode a launch is in — see
            // [`FRONT_END_ADDED`]. Entered in `ecx`, this game's chain calling every callback that way.
            unlocks_read: Some(Entered::InEcx(Patch {
                target: FRONT_END_ADDED,
                prologue: FRONT_END_ADDED_PROLOGUE,
            })),
            // The screen's own read of the file, entered in `ecx` the way this game's chain enters every
            // callback — see [`RANKING_ADDED`] and [`Entered`].
            ranking_read: Some(Entered::InEcx(Patch {
                target: RANKING_ADDED,
                prologue: RANKING_ADDED_PROLOGUE,
            })),
            // The seam inside the game's own save, where the file's records have just been laid over the
            // ones in memory and the write out of memory has not happened yet — see [`PARSE_CARD_RECORDS`].
            // Two registers rather than one, the parse taking the image in `ecx` and the array in `edx`.
            card_records_read: Some(Patch {
                target: PARSE_CARD_RECORDS,
                prologue: PARSE_CARD_RECORDS_PROLOGUE,
            }),
            // The last moment `force_windowed` can be obeyed: after the configuration is read and before
            // either the window or the device is made out of it — see [`CREATE_GAME_WINDOW`].
            create_window: Some(Patch {
                target: CREATE_GAME_WINDOW,
                prologue: CREATE_GAME_WINDOW_PROLOGUE,
            }),
            // The device's `Present` redirected, which is what puts the game's frames through a
            // letterbox: 妖々夢 renders 640x480 and its own present stretches that over whatever client
            // area it has, so a window that is not 4:3 — every borderless one on a 16:9 display — showed
            // the game stretched until this went in. [`INIT_D3D_DEVICE`] is where the slot can be
            // swapped, and where a reset that replaced it is caught.
            init_device: Some(Patch {
                target: INIT_D3D_DEVICE,
                prologue: INIT_D3D_DEVICE_PROLOGUE,
            }),
            // Two things want this one, and both are 妖々夢's now: keys are not read to the game while its
            // window is behind — which this game needs *more* than a game with a device of its own, its read
            // being `GetKeyboardState` and so answering what somebody typed at whatever is in front, see
            // [`Th07::acquire_input`] — and a run whose buttons are written down here is a run that can be
            // picked up again, which is what `stage_frame` and `run_input` above answer for. See [`INPUT_READ`].
            input: Some(Patch {
                target: INPUT_READ,
                prologue: INPUT_READ_PROLOGUE,
            }),
            // The pad half of that read, which is orb's: every pad the machine has, in place of the one
            // device the game looks for and the one winmm joystick it falls back to. Not called through —
            // see [`GET_CONTROLLER_INPUT`].
            joystick: Some(Entered::InEcx(Patch {
                target: GET_CONTROLLER_INPUT,
                prologue: GET_CONTROLLER_INPUT_PROLOGUE,
            })),
            // And the read its key config screen makes, which is the one hook here orb calls through:
            // without it a pad orb supplies is a pad that plays the game and cannot be configured in it.
            pad_buttons: Some(Patch {
                target: GET_PAD_BUTTONS,
                prologue: GET_PAD_BUTTONS_PROLOGUE,
            }),
        }
    }

    /// A run of this game can be rewound: every piece a chapter is taken and put back with is here — the
    /// snapshot and the ranges it must leave alone, the stage started and left, the numbers and the seed with
    /// the game's own seal, the cherry, the card record with its own seal and the name back on the plate, the
    /// music, the retry menu's own reads, and the front end asked which mode a run is.
    ///
    /// **What it rests on is one e2e test and not this list**, which is why the answer moved on the day that
    /// test did: `crates/orb-e2e/src/a_pointdevice_run_of_th07.rs` plays a run of a laid-out 妖々夢 through
    /// the whole of the mode — the question, a chapter, a death, the retry, the numbers back. The unit tests
    /// beside it each say one method answers one address; that one says the mode holds together.
    ///
    /// **[`Th07::midstage_table`] being empty is not a reason to answer `false`, and this is the tie worth
    /// knowing**: a chapter then falls at a stage's start and nowhere inside it, which is a coarse mode
    /// rather than no mode — and `Runtime::chaptering` is what a tuning pass observes through, so a `false`
    /// here is a `--chapter-tuning` launch that writes nothing and a table that can never be measured. The
    /// table comes *after* this answer, out of a run somebody plays.
    ///
    /// **What the answer decides is also a file.** A launch here once left a `pointdevice_score.dat` in a
    /// 妖々夢 install while nothing of that game could be rewound, so the runs in it were runs anybody could
    /// have played. That is what a `false` was for; with the mode really there, a rewound run's score belongs
    /// in orb's file and a normal one in the game's, which is what the fork does either way.
    fn rewinds(&self) -> bool {
        true
    }

    /// No run, which is what orb knows of 妖々夢 rather than a claim about the game.
    ///
    /// The trait has no way to decline a `State`, and this is the emptiest one that is true of what has
    /// been measured: nothing is playing, so `Chapters::observe` sees no run, no snapshot is taken, no
    /// boundary is judged and no mark is drawn. A field filled from a th06 offset would be a number out
    /// of another game's memory, which is worse than nothing by exactly the amount somebody would trust
    /// it.
    unsafe fn read_state(&self) -> State {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        let wanted: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_WANTED) };
        let run = unsafe { self.run_numbers() };
        let kind: u32 = unsafe { mem::read(G_RUN_KIND) };
        let boss = unsafe { Self::boss() };
        let in_ending = scene == SCENE_ENDING;
        // **The scene the supervisor is on lags a run's build by the rest of one update**, so a run being built
        // is read off the scene it is on its way to as well: 0x437f1d calls that build from the branch for the
        // scene being *left* — the front end's — and `scene = wanted` is the update's own tail at 0x4382c0,
        // after the call returns. Everything the build reaches therefore reads the old scene, orb's own hook
        // over the added callback included, which is the one moment a resumed run's numbers can go in.
        //
        // **The numbers are what tell that from the frame the front end merely asked for a run**, which has the
        // same pair of scenes and no run object at all — the last one's teardown freed it. A 妖々夢 session is
        // why this is here: a run picked up out of its file came up with the game's own lives, bombs and power,
        // the log saying `resume: not put back`, because `in_run` was false inside the very build it describes.
        let building = (wanted == SCENE_RUN || wanted == SCENE_RUN_AGAIN) && run.is_some();
        State {
            scene,
            wanted,
            playing: scene == SCENE_RUN,
            in_run: scene == SCENE_RUN || scene == SCENE_RUN_AGAIN || building,
            // Somebody at the keyboard: the gameplay scene, and neither of the two runs that look like play
            // and have nobody playing them.
            in_game: scene == SCENE_RUN && kind & (RUN_IS_A_DEMO | RUN_IS_A_REPLAY) == 0,
            in_ending,
            // Asked for only on the frames of an ending, which is what th06's own does and for the same
            // reason: it is a walk of the game's own chain, and there is no ending on any other frame.
            ending_script: in_ending
                .then(|| unsafe { Self::ending_script() })
                .flatten(),
            demo: kind & RUN_IS_A_DEMO != 0,
            replay: kind & RUN_IS_A_REPLAY != 0,
            practice: kind & RUN_IS_A_PRACTICE != 0,
            paused: unsafe { mem::read::<u8>(G_GAME_MANAGER + GAME_MENU_UP) } != 0
                || unsafe { mem::read::<u8>(G_GAME_MANAGER + OTHER_MENU_UP) } != 0,
            unsettled: unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) } != PLAYER_NORMAL,
            bombing: unsafe { mem::read::<u32>(G_PLAYER + BOMB_IN_USE) } != 0,
            in_dialogue: unsafe { Self::dialogue_line() }
                .is_some_and(|line| line >= 0 || line == DIALOGUE_HANDING_OVER),
            stage: unsafe { mem::read::<i32>(G_STAGE) } - STAGE_COUNTS_FROM,
            difficulty: unsafe { mem::read(G_GAME_MANAGER + DIFFICULTY) },
            stage_frames: unsafe { mem::read(G_GAME_MANAGER + STAGE_FRAMES) },
            script_frames: unsafe { mem::read(G_ENEMY_MANAGER + SCRIPT_FRAMES) },
            random_seed: unsafe { mem::read(G_RANDOM) },
            deaths: run.map_or(0, |run| unsafe { mem::read::<f32>(run + DEATHS) } as i32),
            lives: run.map_or(0, |run| unsafe { mem::read::<f32>(run + LIVES) } as i8),
            bombs: run.map_or(0, |run| unsafe { mem::read::<f32>(run + BOMBS) } as i8),
            power: run.map_or(0, |run| unsafe { mem::read::<f32>(run + POWER) } as u16),
            enemy_count: unsafe { mem::read(G_ENEMY_MANAGER + ENEMY_COUNT) },
            bullet_count: unsafe { mem::read(G_BULLET_MANAGER + BULLET_COUNT) },
            laser_count: unsafe { Self::lasers() },
            boss_present: boss.is_some(),
            boss_life: boss.map(|boss| unsafe { mem::read(boss + ENEMY_LIFE) }),
            boss_attack_frames: boss.map(|boss| unsafe { mem::read(boss + ENEMY_ATTACK_FRAMES) }),
            spellcard: (unsafe { mem::read::<u32>(G_ENEMY_MANAGER + SPELLCARD_RUNNING) } != 0)
                .then(|| unsafe { mem::read(G_ENEMY_MANAGER + SPELLCARD) }),
        }
    }

    unsafe fn window(&self) -> Hwnd {
        unsafe { mem::read(G_GAME_WINDOW) }
    }

    unsafe fn d3d_device(&self) -> Device {
        Device(unsafe { mem::read(G_D3D_DEVICE) })
    }

    /// The stream the player holds, as much of it as a rewind is: the buffer DirectSound made, how much of it
    /// there is, how much goes in at a time, the countdown the loop is taken on and the offset the next chunk
    /// goes at — see [`STREAM`] for where each comes from.
    ///
    /// **The file is this game's own rather than winmm's**, which is the one thing 妖々夢 does not share with
    /// 紅魔郷: it opens `thbgm.dat` itself at 0x44bff0 and reads every track out of that handle, so what
    /// carries the position is [`Track::OpenFile`] and a position in it is an offset in the archive. See
    /// `audio::Track`, which is where that difference is written down.
    ///
    /// Nothing where the buffer is not a live object — the word at the wrapper's head is what says so, and a
    /// released one leaves a pointer into a freed block — and nothing where the stream, the source or the size
    /// is not what a playing track has.
    fn music(&self) -> Option<Music> {
        /// A streaming buffer holds a fraction of a second of audio; anything near this is not a buffer size.
        const MAX_BUFFER_SIZE: u32 = 8 << 20;

        let stream = Self::the_stream()?;
        let wrapper = mem::read_committed::<usize>(stream + STREAM_BUFFER).filter(|it| *it != 0)?;
        let buffer = mem::read_committed::<usize>(wrapper)?;
        if !mem::vtable_in_image(buffer) {
            return None;
        }
        let buffer_size = mem::read_committed::<u32>(stream + STREAM_BUFFER_BYTES)?;
        if buffer_size == 0 || buffer_size > MAX_BUFFER_SIZE {
            return None;
        }
        let source = mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)?;
        Some(Music {
            stream,
            buffer: SoundBuffer(buffer),
            buffer_size,
            notify_size: mem::read_committed(stream + STREAM_CHUNK)?,
            track: Track::OpenFile(mem::read_committed::<usize>(source + SOURCE_FILE).unwrap_or(0)),
            bytes_left: source + SOURCE_LEFT,
            write_offset: stream + STREAM_WRITE_AT,
        })
    }

    /// Which track that is, out of the record the source is reading — see [`RECORD_AT`] for why those three
    /// numbers, and 紅魔郷's own answer for why they are mixed this way: what a chapter asks is whether the
    /// track has changed, so what this has to be is different for different tracks rather than meaningful.
    fn music_identity(&self) -> Option<u32> {
        let stream = Self::the_stream()?;
        let source = mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)?;
        let record = mem::read_committed::<usize>(source + SOURCE_RECORD).filter(|it| *it != 0)?;
        let at = mem::read_committed::<u32>(record + RECORD_AT)?;
        let loops_at = mem::read_committed::<u32>(record + RECORD_LOOP_AT)?;
        let bytes = mem::read_committed::<u32>(record + RECORD_BYTES)?;
        Some(bytes ^ at.rotate_left(11) ^ loops_at.rotate_left(22))
    }

    /// And whether that track is the second of the two songs the stage's own data names, which is the one the
    /// game plays for the fight the stage ends with — see [`G_MUSIC_TABLE`].
    ///
    /// By name rather than by the track having changed, because when a chapter was taken decides nothing here:
    /// the table's names are the `.mid` ones and a record's is the file's own, so what they are held against
    /// each other by is the stem — see [`file_stem`].
    fn the_fights_own_song_is_playing(&self) -> Option<bool> {
        let track = Self::the_track_playing()?;
        let (_, fight) = Self::the_stages_own_songs()?;
        Some(same_song(&fight, &track))
    }

    /// The family this game draws its own text in, which orb's own screens are drawn in too — see
    /// [`OWN_FACE`].
    fn own_face(&self) -> Option<Face> {
        Some(OWN_FACE)
    }

    /// The thread that tops the music's buffer up, so a restore of the game's own memory can hold it still
    /// while it copies — see [`MUSIC_THREAD_ID`]. Nothing before the game made one, which is what its id being
    /// zero says.
    fn audio_thread(&self) -> Option<u32> {
        mem::read_committed::<u32>(G_SOUND_PLAYER + MUSIC_THREAD_ID).filter(|id| *id != 0)
    }

    /// Whether the player's own queue still holds a command — see [`SOUND_COMMANDS`].
    fn a_sound_command_is_in_flight(&self) -> bool {
        mem::read_committed::<u32>(G_SOUND_PLAYER + SOUND_COMMANDS).is_some_and(|kind| kind != 0)
    }

    /// The music down through the game's own call, which answers for both of this game's sound backends —
    /// see [`STOP_MUSIC`], and [`Th07::runs_the_sound_commands`] for why the call alone is not the music down.
    unsafe fn stop_music(&self) {
        let stop: unsafe extern "thiscall" fn(usize) =
            unsafe { std::mem::transmute(stop_music_call()) };
        unsafe { stop(G_SUPERVISOR) };
        unsafe { Self::runs_the_sound_commands() };
    }

    /// And the track the chapter was taken under started again, which is a slot of the player's own started
    /// again: the one holding the name the record the source is reading carries — see [`SLOT_NAMES`].
    ///
    /// **Read rather than fixed, because no one slot is a stage's music.** A stage's build loads its own song
    /// into 0 and its boss's into 1 (0x42f1d5, 0x42f1ec), stage 6's loads a third into 2 (0x42f233), and the
    /// ECL plays whichever of them the fight asks for — so which slot a chapter was taken under is only
    /// answerable from the track that was playing. Which is the one thing a restore has already put back by the
    /// time this runs: the stream, the source and the record are the game's own memory, so the track they name
    /// is the chapter's own even though the objects themselves are dead.
    ///
    /// **And [`STAGE_SONG_SLOT`] where no slot holds it**, which is a chapter taken under a track this stage
    /// has since loaded over: a chapter at a stage's start is taken frames into that stage, and what was
    /// playing then can be the scene before it — the front end's own song, whose name the build has already
    /// replaced in the slot it was in. Then the stage's own song is the answer, which is the one
    /// `th06::restart_stage_music` always gives, and the seek that follows is refused by its own identity
    /// check rather than putting a position from another track into this one.
    ///
    /// Nothing at all where there is no stream to read a track off, a restore having no reason to give a chapter
    /// music it had none of.
    ///
    /// **Asked for even where the chapter had nothing audible**, which is not orb wanting the song: this player
    /// keeps its stream in one field — [`STREAM`] — and a track is started by making a new object there, so the
    /// take-down before the copy leaves that field null and a play is the only thing that fills it again. A field
    /// left null is a launch of 妖々夢 with no music at all and then a crash: the play command's own block is
    /// skipped whole where it is null (0x44cbf3), so the stage's script asking for its song does nothing, and
    /// 0x44d01a reads the stream through it with no test at all — measured as a fault at 0x44d023 reading `0x74`
    /// with `ecx` zero, three instructions before the same code tests the same field. What such a chapter has
    /// instead is the sound taken back down once that play has been carried out — see `Snapshot::restore`.
    unsafe fn restart_stage_music(&self) -> bool {
        let Some(track) = Self::the_track_playing() else {
            return false;
        };
        let name = String::from_utf8_lossy(&track).into_owned();
        // **Which is read out of the stage's data and not out of the slots**: the track a chapter's stream is on
        // can be a third thing entirely — a stage's first chapter falls frames into the stage, where that stream
        // is still on the song of the scene before it — and that song is in a slot of its own all the same, so
        // the slot the track is in is no answer: a 妖々夢 session asked for it and got the title screen's music
        // back. What the stage's own data names is the stage's music whatever the stream is on.
        let slot = if self.the_fights_own_song_is_playing() == Some(true) {
            FIGHT_SONG_SLOT
        } else {
            STAGE_SONG_SLOT
        };
        log!("music: restarting slot {slot}, the chapter's own track being {name}");
        // **The stream the copy resurrected let go of before the game is asked for anything.** Those bytes are
        // a chapter's: the object they describe was freed and its sound buffer released when the track changed
        // — the take-down at 0x44cebd closes the thread and the event and calls the stream's own destructor —
        // and no memory copy brings a released COM object back. So every handle the player's memory now
        // carries names something the game has already given up.
        //
        // **A play walks that.** The command the game's own play queues reuses the stream this field names
        // rather than making one: it stops the buffers that stream owns, points its source at a record and
        // fills its buffer. A launch of 妖々夢 stepping about in stage 3 faulted twice over doing it —
        // `0x0045d8d5` with `E_INVALIDARG` left by an `IDirectSoundBuffer::Stop` that had written over the
        // game's own frame, and then a call through address zero from the fill at 0x45dfaf once the released
        // buffer's block had been reused.
        //
        // So the stream is made again before the track is asked for, [`PLAY_BGM`] being the one call that makes
        // one — and these are cleared first so that its own take-down of the current stream walks nothing,
        // exactly as `th06::restart_stage_music` clears its own before asking.
        unsafe {
            mem::write::<usize>(G_SOUND_PLAYER + STREAM, 0);
            mem::write::<u32>(G_SOUND_PLAYER + MUSIC_THREAD_ID, 0);
            mem::write::<usize>(G_SOUND_PLAYER + MUSIC_THREAD_HANDLE, 0);
            mem::write::<usize>(G_SOUND_PLAYER + MUSIC_EVENT, 0);
        }
        let make: unsafe extern "thiscall" fn(usize, usize) -> i32 =
            unsafe { std::mem::transmute(play_bgm_call()) };
        let made = unsafe { make(G_SOUND_PLAYER, MUSIC_FILE) };
        if made < 0 {
            log!("music: the game would not make a stream over its own music file ({made})");
            return false;
        }
        // And the track on that stream, which is the slot asked for: a stream is the file, and which of the
        // file's records is playing is what a play decides.
        let start: unsafe extern "thiscall" fn(usize, u32) =
            unsafe { std::mem::transmute(start_music_call()) };
        unsafe { start(G_SUPERVISOR, slot) };
        // **And the queue left to the frame's own sound**, which is the one thing this must not do itself: the
        // ask is a command the player walks a phase at a time, and carrying it out here is carrying it out over
        // the stream the memory copy has just written back — a launch of 妖々夢 faulted inside the game's own
        // sound doing exactly that, at `0x0045d8d5` reading `4` with `E_INVALIDARG` in two registers. What waits
        // for the queue to empty is `Snapshot::finish_the_restores_sound`.
        true
    }

    /// The three objects the sound system is, which a restore has to leave alone: the player itself, the
    /// stream it holds and the source that stream reads the file through — see [`SOUND_PLAYER_BYTES`],
    /// [`STREAM_BYTES`] and [`SOURCE_BYTES`] for where each extent comes from.
    ///
    /// The player is always there, being a static; the other two are objects a playing track has and nothing
    /// else does, so a launch with no music answers the one range.
    fn audio_state(&self) -> Vec<Range<usize>> {
        // `once` rather than `vec!` of the one range, which is the lint `live_handles` answers the same way.
        let mut ranges: Vec<Range<usize>> =
            std::iter::once(G_SOUND_PLAYER..G_SOUND_PLAYER + SOUND_PLAYER_BYTES).collect();
        let stream = mem::read_committed::<usize>(G_SOUND_PLAYER + STREAM).filter(|it| *it != 0);
        if let Some(stream) = stream {
            ranges.push(stream..stream + STREAM_BYTES);
            if let Some(source) =
                mem::read_committed::<usize>(stream + STREAM_SOURCE).filter(|it| *it != 0)
            {
                ranges.push(source..source + SOURCE_BYTES);
            }
        }
        ranges
    }

    /// The sprite manager's textures, said as the one range they are — see [`ANM_TEXTURES`] for why that is
    /// all of them and why nothing else the manager holds is here.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        mem::read_committed::<usize>(G_QUAD_QUEUE)
            .filter(|manager| *manager != 0)
            .map(|manager| {
                let textures = manager + ANM_TEXTURES;
                // One range, said as one: the slots are contiguous, and `vec!` of a single range is a
                // lint 紅魔郷's own answer avoids the same way.
                std::iter::once(textures..textures + ANM_TEXTURE_SLOTS * size_of::<usize>())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn play_area(&self) -> Rect {
        PLAY_AREA
    }

    /// **Read out of the vm the game lays its own background with rather than written down**, which is what
    /// [`VM_SPRITE`] is for: the sprite that vm is on carries the slot of the texture the sheet is in and the
    /// piece of it the tile is, so orb has neither the sheet's own layout nor which slot `front.anm` was loaded
    /// into as a constant of its own. See [`PANEL_TILE_VM`].
    ///
    /// Nothing before the panel is built — no object, no sprite on the vm, no manager — which is every frame
    /// outside a stage.
    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        let over = unsafe { Self::over_the_run() }?;
        let sprite = mem::read_committed::<usize>(over + PANEL_TILE_VM + VM_SPRITE)
            .filter(|sprite| *sprite != 0)?;
        let slot = usize::try_from(mem::read_committed::<u32>(sprite + SPRITE_TEXTURE)?).ok()?;
        if slot >= ANM_TEXTURE_SLOTS {
            return None;
        }
        let manager = mem::read_committed::<usize>(G_QUAD_QUEUE).filter(|it| *it != 0)?;
        let texture =
            mem::read_committed::<usize>(manager + ANM_TEXTURES + slot * size_of::<usize>())
                .filter(|texture| *texture != 0)?;
        let mut uv = [0.0; 4];
        for (corner, at) in uv.iter_mut().zip(SPRITE_UV) {
            *corner = mem::read_committed::<f32>(sprite + at)?;
        }
        Some(PanelTile {
            texture: Texture(texture),
            uv,
            origin: PANEL_ORIGIN,
            pitch: PANEL_PITCH,
        })
    }

    /// The lives' own pair of bits set to two, and the panel's ground asked for alongside it — see
    /// [`GUI_ROWS`] for why both, and [`G_PANEL_REPAINT`] for what the second one is.
    ///
    /// The rest of the word is left as it is: the other pairs are the panel's other rows, and a row the game
    /// asked for is one it is owed.
    unsafe fn repaint_lives_row(&self) {
        let rows = unsafe { mem::read::<u32>(G_GUI + GUI_ROWS) };
        unsafe {
            mem::write::<u32>(
                G_GUI + GUI_ROWS,
                rows & !LIVES_ROW_PAINT | PAINT_FOR_TWO_FRAMES,
            )
        };
        unsafe { mem::write::<u32>(G_PANEL_REPAINT, PAINT_FOR_TWO_FRAMES) };
    }

    /// Whether the game will paint that row this frame, which is its panel's own draw job being on the draw
    /// list — see [`PANEL_DRAW`] for why that and not whether a run is being played.
    unsafe fn draws_lives_row(&self) -> bool {
        unsafe { Self::chain_element(G_CHAIN + CHAIN_DRAW_LIST, PANEL_DRAW) }.is_some()
    }

    /// Above the difficulty's own name, which is what this game writes in the bottom-left: 驚異, then
    /// `Lunatic Level`, then 奇特な方向け under it — where 紅魔郷 has nothing, and a launch of 妖々夢 showed
    /// orb's two lines over the top of them. Above it and not in another corner: the top-left was tried, and
    /// read badly for being nowhere near what it is about.
    ///
    /// **Two lines clear of it and not one**, because [`DIFFICULTY_LABEL_TOP`] is a reading off a screen and the
    /// first such reading was 16 pixels out — one line above it still touched 驚異 — and because the lower of
    /// orb's own two lines is as tall as the face it is drawn in, which is a size that has moved once already.
    fn mark_corner(&self) -> (f32, f32) {
        (
            24.0,
            DIFFICULTY_LABEL_TOP - crate::menu_ui::LINE_HEIGHT * 2.0,
        )
    }

    fn lives_row(&self) -> Rect {
        LIVES_ROW
    }

    fn content_size(&self) -> (u32, u32) {
        BACK_BUFFER
    }

    /// The game's own configuration, which is the byte its window and its device are both built from — see
    /// [`CFG_WINDOWED`].
    fn windowed(&self) -> bool {
        unsafe { mem::read::<u8>(CFG_WINDOWED) != 0 }
    }

    unsafe fn force_windowed(&self) {
        unsafe { mem::write::<u8>(CFG_WINDOWED, 1) };
    }

    /// The play field's own viewport, and no clear.
    ///
    /// What the game does at 0x406afc is set this viewport and then clear it to opaque black through
    /// +0x90 — `D3DCLEAR_TARGET`, Z 1.0, no stencil, read off the pushes at 0x406b02. That clear is the
    /// stage wiping its own field rather than device setup being put back, and no depth clear beside it
    /// has been read, so putting one here would be orb inventing one. Reached only on the frames orb
    /// freezes the game for, of which 妖々夢 has none: it asks no question and offers no menu here.
    unsafe fn set_play_viewport(&self, device: Device) {
        let viewport = Viewport {
            x: PLAY_AREA.left as u32,
            y: PLAY_AREA.top as u32,
            width: PLAY_AREA.width as u32,
            height: PLAY_AREA.height as u32,
            min_z: 0.5,
            max_z: 1.0,
        };
        unsafe { mem::write(VIEWPORT, viewport) };
        orb_api::d3d8::set_viewport(device, viewport);
    }

    fn chain(&self) -> *mut c_void {
        G_CHAIN as *mut c_void
    }

    /// Taken as not wiping it, so that orb cleans up after whatever it draws rather than leaving it to
    /// a wipe that may not come. Which of 妖々夢's options wipes the back buffer has not been read, and
    /// the wrong way round here is orb's own drawing hardening into the corners of a frame.
    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    /// The keyboard needs nothing done to it, which is what `true` says: **妖々夢 reads its keys with
    /// `GetKeyboardState`**, and the system never takes that away.
    ///
    /// Read off the read itself — 0x430b76 calls `[0x48d1b0]`, the second entry of the USER32 import table at
    /// 0x48d1ac, which is `GetKeyboardState`, with a 0x100-byte buffer on its own stack, and every key it wants
    /// is a byte of that buffer tested against 0x80. So there is no device to lose and none to ask for again,
    /// which is 紅魔郷's `--sent-keys` launch and not its ordinary one.
    unsafe fn acquire_input(&self) -> bool {
        true
    }

    /// Nothing was let go of, because there is nothing to let go of: 妖々夢 reads its keys with
    /// `GetKeyboardState` — see [`Th07::acquire_input`] — which is the read a key another program sent is seen
    /// through already.
    ///
    /// So `--sent-keys` costs this game nothing and buys it nothing: what that flag does to 紅魔郷 is take away a
    /// device held `DISCL_EXCLUSIVE | DISCL_FOREGROUND` so that the game falls back to this same read.
    unsafe fn take_sent_keys(&self) -> bool {
        false
    }

    /// The whole-output viewport the game sets at this point in its own frame.
    ///
    /// 0,0,640,480 written into the game's own `D3DVIEWPORT8` at 0x575a18 at 0x4347ad and set through
    /// +0xa0 at 0x4347d4, which is where its own frame does it: after the scene it drew into ends and
    /// before the update. No background
    /// clear, that being an option of the game's that has not been read, and no framerate multiplier,
    /// 妖々夢 not having been read for one.
    unsafe fn prepare_frame(&self, device: Device) {
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: BACK_BUFFER.0,
            height: BACK_BUFFER.1,
            min_z: 0.0,
            max_z: 1.0,
        };
        unsafe { mem::write(VIEWPORT, viewport) };
        orb_api::d3d8::set_viewport(device, viewport);
    }

    /// What 妖々夢's own frame does inside its scene before its draw chain, in that frame's own order:
    /// the queue of quads emptied, the fog flag written, and the fog put out.
    ///
    /// Three things and not one address. The queue is the one that must be here — the append the drawing
    /// reaches writes where [`EMPTY_QUEUE`] points a field, and a frame that skipped it writes a quad
    /// through a null pointer, which is how the game went down the first time orb's loop replaced this
    /// frame. The other two are one call and the write that makes it reach the device: see [`FOG_ON`].
    ///
    /// # Safety
    /// As the trait says, and both calls go into the game's own code: the game must be past the startup
    /// that allocated what [`G_QUAD_QUEUE`] points at.
    unsafe fn begin_drawing(&self, _device: Device) {
        let queue = unsafe { mem::read::<usize>(G_QUAD_QUEUE) };
        let empty: unsafe extern "thiscall" fn(usize) =
            unsafe { std::mem::transmute(empty_queue()) };
        unsafe { empty(queue) };
        unsafe { mem::write::<u32>(G_SUPERVISOR + FOG_ON, FOG_ON_FORCED) };
        let fog: unsafe extern "thiscall" fn(usize) = unsafe { std::mem::transmute(fog_off()) };
        unsafe { fog(G_SUPERVISOR) };
    }

    /// And what it does after the chain: the queue drawn, which is every sprite of that frame reaching
    /// the device.
    ///
    /// The game's own frame makes this call twice — again at 0x43478e, outside the scene — and the second
    /// one does nothing, the count this one leaves at zero being what it returns on. So orb makes the one
    /// that draws and not the one that does not.
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
        }
    }

    /// The replay being watched started at another of its stages, the game's own way: the stage put where a
    /// build reads it and the scene asked for the one that rebuilds the manager — 0x42e894's own branch, the
    /// scene a stage transition uses — after which the replay's record for that stage puts the run back.
    ///
    /// **The stage goes in the way a menu leaves it and not the way orb reads it**: the build adds one at
    /// 0x42ef50, so what is written here is orb's own 0-based number. See [`G_STAGE`].
    ///
    /// False for a stage the replay does not cover, which is [`G_REPLAY_MANAGER`]'s record of nothing: asking
    /// for one drops the game to its menu rather than saying so.
    unsafe fn jump_to_stage(&self, stage: i32) -> bool {
        if !(0..REPLAY_STAGES).contains(&stage) {
            return false;
        }
        let record = mem::read_committed::<usize>(G_REPLAY_MANAGER)
            .filter(|manager| *manager != 0)
            .and_then(|manager| mem::read_committed::<usize>(manager + REPLAY_HEADER))
            .filter(|header| *header != 0)
            .and_then(|header| {
                mem::read_committed::<usize>(
                    header + REPLAY_STAGE_RECORDS + stage as usize * size_of::<usize>(),
                )
            })
            .filter(|record| *record != 0);
        if record.is_none() {
            return false;
        }
        unsafe {
            mem::write::<i32>(G_STAGE, stage);
            mem::write::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED, SCENE_RUN_AGAIN);
        }
        true
    }

    /// The state and the frames left under it, both, and only out of the two states the player can be hit in —
    /// see [`PLAYER_COMING_BACK`] for why that state and not the border after it.
    ///
    /// Dying, being hit and the border are left as they are: each is the game in the middle of something orb has
    /// no business finishing, and none of the three can be hit in anyway. Coming back is written over rather
    /// than left, because leaving it means the frames run out and the update they run out in is one the hit test
    /// sees a player it can kill.
    unsafe fn make_invulnerable(&self) {
        let state = G_PLAYER + PLAYER_STATE;
        let current: i8 = unsafe { mem::read(state) };
        if current == PLAYER_NORMAL || current == PLAYER_COMING_BACK {
            unsafe {
                mem::write::<i8>(state, PLAYER_COMING_BACK);
                mem::write::<i32>(
                    G_PLAYER + PLAYER_COMING_BACK_FRAMES,
                    PLAYER_INVULNERABLE_FRAMES,
                );
            }
        }
    }

    /// Whether the replay in memory is one being watched, whose record of inputs nothing may write into:
    /// the bit its own load sets — see [`RUN_IS_A_REPLAY`].
    unsafe fn replaying(&self) -> bool {
        let kind: u32 = unsafe { mem::read(G_RUN_KIND) };
        kind & RUN_IS_A_REPLAY != 0
    }

    /// The ranking asked for by writing what the game's own Score item writes, and the cursor remembered for
    /// afterwards: asking takes the front end down, so the cursor has to be read while it is still up.
    unsafe fn show_ranking(&self) {
        let cursor = unsafe { Self::chain_argument(FRONT_END_UPDATE) }
            .and_then(|front| mem::read_committed::<i32>(front + FRONT_END_CURSOR));
        if let Some(cursor) = cursor {
            // Kept one along, so that nothing remembered and item 0 are told apart.
            MENU_CURSOR_BEFORE.store(cursor as usize + 1, Ordering::Relaxed);
        }
        unsafe { mem::write::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED, SCENE_RANKING) };
    }

    /// The ranking built: the scene being it, the screen on the chain, and that screen past the state its own
    /// build runs in — see [`RESULT_BUILDING`].
    unsafe fn showing_ranking(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        if scene != SCENE_RANKING {
            return false;
        }
        let screen = unsafe { Self::chain_argument(RESULT_UPDATE) };
        let Some(screen) = screen else { return false };
        mem::read_committed::<i32>(screen + RESULT_PHASE)
            .is_some_and(|state| state != RESULT_BUILDING)
    }

    /// Whether the ranking is still the scene at all, which is the coarser question: the screen answers a
    /// request to leave by changing its own state and putting the scene back a second later, and those frames
    /// are drawn ones.
    unsafe fn ranking_scene(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        scene == SCENE_RANKING
    }

    /// The front end's cursor put back where it was before the ranking was asked for — into the front end that
    /// is up *now*, the one that asked having been taken down and built again.
    unsafe fn restore_menu_cursor(&self) {
        let Some(cursor) = MENU_CURSOR_BEFORE.swap(0, Ordering::Relaxed).checked_sub(1) else {
            return;
        };
        let front = unsafe { Self::chain_argument(FRONT_END_UPDATE) };
        if let Some(front) = front {
            unsafe { mem::write::<i32>(front + FRONT_END_CURSOR, cursor as i32) };
        }
    }

    /// Said rather than numbered, there being no numbers: the line exists so that a ranking built and
    /// taken down can be read afterwards, and none is built here.
    /// The four numbers and no verdict, which is what the trait asks for.
    unsafe fn ranking_state(&self) -> String {
        let screen = unsafe { Self::chain_argument(RESULT_UPDATE) };
        format!(
            "cur={} wanted={} screen={:#x} state={}",
            unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_SCENE) },
            unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED) },
            screen.unwrap_or(0),
            screen
                .and_then(|screen| mem::read_committed::<i32>(screen + RESULT_PHASE))
                .unwrap_or(-1),
        )
    }

    /// The screen told to leave the way its own menu tells it: the state its leaving runs in, and the timer
    /// that state waits on put back to nothing — see [`RESULT_LEAVING`]. The screen then puts the scene back
    /// itself, which is what writes the file.
    unsafe fn leave_ranking(&self) {
        let screen = unsafe { Self::chain_argument(RESULT_UPDATE) };
        let Some(screen) = screen else { return };
        unsafe {
            mem::write::<i32>(screen + RESULT_TIMER, 0);
            mem::write::<i32>(screen + RESULT_PHASE, RESULT_LEAVING);
        }
    }

    /// No card is counted. Where 妖々夢 keeps its record of spell cards has not been read, and a count
    /// written into an address that is not that would be a number in somebody's score file.
    /// One more attempt at the card being fought, counted where the game cannot count it itself: a chapter
    /// begun inside a card never starts that card, so the game's own count at 0x410110 never runs for it.
    ///
    /// Nothing where no card is up, and nothing where the record is not one the game has written — which is
    /// what its `"CATK"` tag says. See [`CARD_ATTEMPTS`].
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        let running: u32 = unsafe { mem::read(G_ENEMY_MANAGER + SPELLCARD_RUNNING) };
        if running == 0 {
            return None;
        }
        let card =
            usize::try_from(unsafe { mem::read::<u32>(G_ENEMY_MANAGER + SPELLCARD) }).ok()?;
        if card >= CARDS {
            return None;
        }
        let record = G_CARD_RECORDS + card * CARD_RECORD_BYTES;
        if mem::read_committed::<u32>(record)? != CARD_RECORD_TAG {
            return None;
        }
        let shot = usize::from(unsafe { mem::read::<u8>(G_SHOT_COMBINED) });
        let at = record + CARD_ATTEMPTS + shot * size_of::<u16>();
        let counted = counted_once_more(mem::read_committed::<u16>(at)?);
        unsafe { mem::write::<u16>(at, counted) };
        let total = record + CARD_ATTEMPTS_IN_ALL;
        let in_all = counted_once_more(mem::read_committed::<u16>(total)?);
        unsafe { mem::write::<u16>(total, in_all) };
        // Without this the game wipes the card's whole history at its next look — see [`CARD_CHECK`].
        unsafe { Self::seal_card_record(record) };
        Some(counted)
    }

    /// **Not the game's own naming of a card (0x428887)**, which is the obvious way to do this. It takes a
    /// second argument orb cannot recover: which plate to show, which the declaration carries as a word of the
    /// ECL instruction at its `+0xc`, and that instruction is one the interpreter moved past frames ago. What
    /// that argument picks is a sprite in the vms the same object holds — memory, which a snapshot puts back —
    /// and the call does three more things a restore has no business repeating: it plays the sound a card starts
    /// with, it sets the plate's own animation going again, and it writes the supervisor's field at 0x575ab4.
    ///
    /// What is *not* memory is the sprite a string was baked into, so what a restore has to do again is the
    /// baking, which is the call underneath that function and takes only the name. See [`DRAW_STRING`].
    unsafe fn redraw_card_name(&self) {
        let fought = unsafe { Self::card_being_fought() };
        let Some((card, record)) = fought else {
            return;
        };
        // The name out of the card's own record, which is the copy the game made of the declaration's when the
        // card started: 0x40ffb4 is that copy, and the declaration itself is an instruction nothing keeps a
        // pointer to. A record the game has written and never named is a card no session has started — the
        // bytes there are nothing, and a plate blanked is a worse answer than one left as it was.
        let at = record + CARD_NAME;
        let name = unsafe { mem::read_bytes(at, CARD_RECORD_BYTES - CARD_NAME) };
        let Some(end) = name.iter().position(|byte| *byte == 0) else {
            return log!(
                "card: card {card}'s name runs past the record it is in; the plate is left as it is"
            );
        };
        if end == 0 {
            return;
        }
        // And the two objects the call is made on and into, neither of which exists outside a stage.
        let over_the_run = unsafe { Self::over_the_run() };
        let Some(over) = over_the_run else {
            return;
        };
        let Some(manager) = mem::read_committed::<usize>(G_QUAD_QUEUE).filter(|it| *it != 0) else {
            return;
        };
        let bake: unsafe extern "cdecl" fn(usize, usize, u32, u32, usize) =
            unsafe { std::mem::transmute(draw_string()) };
        unsafe {
            bake(
                manager,
                over + CARD_NAME_VM,
                CARD_NAME_COLOUR,
                CARD_NAME_SHADOW,
                at,
            )
        };
    }

    /// The card record as it stands, taken whole — see [`G_CARD_RECORDS`].
    unsafe fn captures(&self) -> Vec<u8> {
        unsafe { mem::read_bytes(G_CARD_RECORDS, CARDS * CARD_RECORD_BYTES) }
    }

    /// And put back, all of it or none: a record half this build's size is one from another build, and
    /// writing part of it would leave the game reading a card that is half two others.
    unsafe fn set_captures(&self, saved: &[u8]) {
        let bytes = CARDS * CARD_RECORD_BYTES;
        if saved.len() != bytes {
            return log!(
                "score: {} bytes of captures is not this build's {bytes}; left alone",
                saved.len()
            );
        }
        unsafe { mem::write_bytes(G_CARD_RECORDS, saved) };
    }

    /// The counts back and every name left as the playback named it — and the check byte recomputed for each
    /// record, since a name held over counts that were put back is a sum that matches neither.
    ///
    /// See [`CARD_CHECK`] for what an unmatched sum costs, and the trait for why a name is not a count.
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]) {
        let bytes = CARDS * CARD_RECORD_BYTES;
        if saved.len() != bytes {
            return log!(
                "score: {} bytes of captures is not this build's {bytes}; left alone",
                saved.len()
            );
        }
        let mut going_back = saved.to_vec();
        for (card, record) in going_back
            .as_chunks_mut::<CARD_RECORD_BYTES>()
            .0
            .iter_mut()
            .enumerate()
        {
            let at = G_CARD_RECORDS + card * CARD_RECORD_BYTES;
            let named = unsafe { mem::read_bytes(at + CARD_NAME, CARD_ATTEMPTS - CARD_NAME) };
            record[CARD_NAME..CARD_NAME + named.len()].copy_from_slice(&named);
        }
        unsafe { mem::write_bytes(G_CARD_RECORDS, &going_back) };
        // Every record and not only the ones a name came back into: the counts that went in are the saved
        // ones, so each record's sum has to be taken again against the name it now holds.
        for card in 0..CARDS {
            unsafe { Self::seal_card_record(G_CARD_RECORDS + card * CARD_RECORD_BYTES) };
        }
    }

    /// The records a run left, written back over the ones the file's own parse has just laid down — and the
    /// counts of every card that moved said out loud, which is what a launch is read for.
    ///
    /// **Back and not added**, for the reason the trait gives: the build a run begins with empties these records
    /// and reads the file into them (0x42e657 and 0x42e735), so a run's own are the file's with its counts on
    /// top already. Added to the file's again, a run's history would be counted twice.
    unsafe fn put_back_the_cards_a_run_left(&self, left: &[u8]) {
        let bytes = CARDS * CARD_RECORD_BYTES;
        if left.len() != bytes {
            return log!(
                "score: {} bytes of captures is not this build's {bytes}; nothing put back",
                left.len()
            );
        }
        let shot = usize::from(unsafe { mem::read::<u8>(G_SHOT_COMBINED) });
        let mut moved = 0;
        for (card, record) in left.as_chunks::<CARD_RECORD_BYTES>().0.iter().enumerate() {
            let at = G_CARD_RECORDS + card * CARD_RECORD_BYTES;
            let word = |from: &[u8], word: usize| {
                let at = word + shot * size_of::<u16>();
                u16::from_le_bytes([from[at], from[at + 1]])
            };
            let standing = unsafe { mem::read_bytes(at, CARD_RECORD_BYTES) };
            if standing == record {
                continue;
            }
            moved += 1;
            detail!(
                "score: card {card} now {} attempt(s) and {} capture(s) for shot {shot}, where the file had \
                 {} and {}",
                word(record, CARD_ATTEMPTS),
                word(record, CARD_CAPTURES),
                word(&standing, CARD_ATTEMPTS),
                word(&standing, CARD_CAPTURES),
            );
        }
        unsafe { mem::write_bytes(G_CARD_RECORDS, left) };
        log!("score: the records this run left put back over {moved} the file's own differ in");
    }

    /// True, and [`SAVE_KIND_WRITES`] is the save it means: the one write in this exe, made by the game at every
    /// run's build and once on the way out.
    fn writes_its_own_records(&self) -> bool {
        true
    }

    /// The phase the score-file object carries, which is [`SAVE_PHASE_WRITES`] for the save that writes and 0xa
    /// or 0x12 for the two the ranking screen is built with — see [`SAVE_KIND_WRITES`], where the write is gated
    /// on it at 0x44a1e0.
    unsafe fn its_records_are_about_to_be_written(&self, screen: *mut c_void) -> bool {
        mem::read_committed::<i32>(screen as usize + RESULT_PHASE) == Some(SAVE_PHASE_WRITES)
    }

    /// One attempt and one capture on a card's record, and the total with them — the words the game's own
    /// counting moves, held at [`CARD_COUNT_CAP`] the way it holds them.
    unsafe fn count_a_card_for_a_check(&self, card: usize) -> bool {
        if card >= CARDS {
            log!("score: {card} is not a card of this game's {CARDS}; nothing counted");
            return false;
        }
        let at = G_CARD_RECORDS + card * CARD_RECORD_BYTES;
        let shot = usize::from(unsafe { mem::read::<u8>(G_SHOT_COMBINED) });
        for word in [
            CARD_ATTEMPTS + shot * size_of::<u16>(),
            CARD_CAPTURES + shot * size_of::<u16>(),
            CARD_ATTEMPTS_IN_ALL,
        ] {
            let Some(standing) = mem::read_committed::<u16>(at + word) else {
                return false;
            };
            unsafe { mem::write::<u16>(at + word, counted_once_more(standing)) };
        }
        unsafe { Self::seal_card_record(at) };
        log!(
            "score: card {card} counted once more for shot {shot}, for a check of the file's write"
        );
        true
    }

    /// The captures in memory emptied where a ranking read is about to define them, and kept where it is not.
    ///
    /// **The scene is what tells those apart in this game**, rather than the screen's own state: 0x44a302 builds
    /// one screen for both, and [`SCENE_RANKING`] is the one the front end asked for while [`SCENE_RUN_OVER`] is
    /// the one a run ends into. On the way out of a run the record in memory is that run's own, and the read
    /// that follows must add to it rather than replace it — so nothing is cleared there.
    ///
    /// The `screen` is unread for the same reason. See [`G_CARD_RECORDS`] for what is being emptied.
    unsafe fn forget_captures(&self, _screen: *mut c_void) {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        if scene == SCENE_RUN_OVER {
            return;
        }
        unsafe { mem::fill_bytes(G_CARD_RECORDS, 0, CARDS * CARD_RECORD_BYTES) };
        log!("score: the captures in memory cleared for the ranking about to be read");
    }

    /// The two records this game keeps read again out of the mode's own file, over the half of the front end's
    /// read that is a record of runs: the practice scores, and the card records — see the trait, and
    /// [`READ_SCORE_FILE`] for the three calls this is.
    ///
    /// One read for both, which is what the game's own does with them: [`PARSE_PRACTICE_SCORES`] takes the
    /// scores out of the file's image and [`PARSE_CARD_RECORDS`] takes the cards, each over the array it is
    /// handed.
    ///
    /// Asked for by the name orb asks for, which is what puts the answer in the mode's file: the fork is over
    /// `CreateFileA` and decides by the name, so this open lands where every other open of `score.dat` lands.
    unsafe fn read_what_the_modes_file_records(&self) {
        let read: unsafe extern "fastcall" fn(*const u8) -> usize =
            unsafe { std::mem::transmute(read_score_file()) };
        let scores: unsafe extern "fastcall" fn(usize, usize) -> i32 =
            unsafe { std::mem::transmute(parse_practice_scores()) };
        let cards: unsafe extern "fastcall" fn(usize, usize) -> i32 =
            unsafe { std::mem::transmute(parse_card_records()) };
        let release: unsafe extern "fastcall" fn(usize) =
            unsafe { std::mem::transmute(release_score_file()) };
        let file = unsafe { read(crate::score::asked_for()) };
        if file == 0 {
            return log!("score: no file of the mode's own to read what it records out of");
        }
        unsafe { scores(file, G_PRACTICE_SCORES) };
        unsafe { cards(file, G_CARD_RECORDS) };
        unsafe { release(file) };
        log!("score: what the mode's own file records read out of it again");
    }

    /// What the title menu's cursor is on, where orb has a question about it — and nothing on any other
    /// screen of the front end, on any other scene, or on a scene still in flight.
    ///
    /// The three items that start a run are one question and the ranking is the other, which is
    /// [`Menu`]'s own shape. See [`FRONT_END_UPDATE`] for where each of the three reads comes from.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        let settled = unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_SCENE) }
            == SCENE_FRONT_END
            && unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED) } == SCENE_FRONT_END;
        if !settled {
            return None;
        }
        let front = unsafe { Self::chain_argument(FRONT_END_UPDATE) };
        Self::says_what_it_reads_of_the_front_end(front);
        let front = front?;
        if mem::read_committed::<i32>(front + FRONT_END_SCREEN)? != SCREEN_TITLE {
            return None;
        }
        match mem::read_committed::<i32>(front + FRONT_END_CURSOR)? {
            TITLE_ITEM_START | TITLE_ITEM_EXTRA | TITLE_ITEM_PRACTICE => Some(Menu::Run),
            TITLE_ITEM_SCORE => Some(Menu::Scores),
            _ => None,
        }
    }

    /// Whether a press at the title menu would be acted on: its own phase and the count of frames the screen
    /// has been up, which is the game's own gate — see [`FRONT_END_UPDATE`].
    unsafe fn menu_takes_a_press(&self) -> bool {
        let front = unsafe { Self::chain_argument(FRONT_END_UPDATE) };
        let Some(front) = front else { return false };
        let phase = mem::read_committed::<i32>(front + FRONT_END_PHASE);
        match mem::read_committed::<i32>(front + FRONT_END_SCREEN) {
            Some(SCREEN_TITLE) => {
                phase == Some(TITLE_PHASE_ITEMS)
                    && mem::read_committed::<i32>(front + FRONT_END_SETTLED)
                        .is_some_and(|frames| frames >= SETTLED_FRAMES)
            }
            // And the shot select, which is the other screen orb has a question over: a run left unfinished
            // is offered there, and a press orb does not hold back there is a run started before the question
            // could be asked. Its own gate is the phase and not the count of frames — see
            // [`SHOT_SELECT_PHASE_ITEMS`].
            Some(SCREEN_SHOT_SELECT | SCREEN_SHOT_SELECT_EXTRA) => {
                phase == Some(SHOT_SELECT_PHASE_ITEMS)
            }
            _ => false,
        }
    }

    /// What the pad is doing, for the menus orb puts over this game: the reading orb sampled, made into the
    /// game's own word and read with the game's own masks.
    ///
    /// **Which is the only pad there is here.** 妖々夢 looks for a DirectInput device of its own and falls back to
    /// one winmm joystick, and neither found the pad on the machine [`GET_CONTROLLER_INPUT`] was measured on — so
    /// the pad that plays the game is the one orb samples, and a menu of orb's reading anything else is a menu
    /// ignoring a pad that plainly works. What that costs is the mode question and, dozens of times a fight, the
    /// retry menu: a hand holding a controller cannot reach for the keyboard for each of those.
    ///
    /// The word rather than the reading, because which button is *decide* is the game's business and not orb's:
    /// [`PAD_MAPPING`] is what `th07.cfg` holds, and [`MENU_DECIDE`] and [`MENU_CANCEL`] are the two masks every
    /// screen of the game's own front end tests against. So a pad configured in the game answers orb's questions
    /// with the same buttons it answers the game's with.
    ///
    /// Hats always, whatever `dpad_moves` says: that setting is about the word the *game* is handed — see
    /// [`Th07::pad_word`] — and a d-pad on a menu of orb's is a direction like any other.
    unsafe fn pad(&self, pad: Option<Reading>) -> Pad {
        let word = unsafe { self.pad_word(pad, true) };
        Pad {
            up: word & PAD_UP != 0,
            down: word & PAD_DOWN != 0,
            decide: word & MENU_DECIDE != 0,
            cancel: word & MENU_CANCEL != 0,
            left: word & PAD_LEFT != 0,
            right: word & PAD_RIGHT != 0,
            shot: word & BUTTON_SHOT != 0,
            bomb: word & BUTTON_BOMB != 0,
            focus: word & BUTTON_FOCUS != 0,
            skip: word & BUTTON_SKIP != 0,
        }
    }

    /// Every pad orb sampled, in the array the game's own read just filled: 0x80 in the byte of each
    /// button held, which is what its key config screen walks for the button to assign.
    ///
    /// Bounded by what that screen reads rather than by the array's own length, so a pad with more buttons
    /// than the game looks at cannot write past it — the game's own fill has the same bound.
    unsafe fn add_pad_buttons(&self, into: *mut c_void, pad: Option<Reading>) {
        let Some(reading) = pad else {
            return;
        };
        for button in 0..PAD_BUTTONS_READ {
            let held = 1u32
                .checked_shl(u32::try_from(button).unwrap_or(u32::MAX))
                .is_some_and(|bit| reading.buttons & bit != 0);
            if held {
                unsafe { mem::write::<u8>(into as usize + button, PAD_BUTTON_HELD) };
            }
        }
    }

    /// Every pad orb sampled, as the whole of what the game's own pad read would have made of one — that
    /// read being hooked and not called through.
    ///
    /// **The buttons go through the game's own mapping**, which is what makes this the word the game
    /// would have built: [`PAD_MAPPING`] says which pad button each bit of it is, so a pad whose button 3
    /// is bomb here is one whose button 3 bombs. The stick is the four directions, each axis measured
    /// against the travel that pad's own caps report rather than joystick 0's — see
    /// [`crate::joystick::axis`] for the rule, which is the game's own.
    ///
    /// And the hat, where `dpad_moves` says so: the game's own read has no look at that field, so
    /// everything a d-pad does here is orb's, and an XInput pad's d-pad arrives as a hat.
    ///
    /// Nothing where there is no pad, rather than nothing done at all: a pad let go of, lost or unplugged
    /// is one that is not being held any more, and the word the keyboard was read into is the whole of the
    /// read then.
    unsafe fn pad_word(&self, pad: Option<Reading>, hats: bool) -> u16 {
        let Some(reading) = pad else {
            return 0;
        };
        // The table in one read, nine `i16` being what the game keeps and what `th07.cfg` holds.
        let mapping = unsafe { mem::read::<[i16; 9]>(PAD_MAPPING) };
        let buttons = mapping
            .iter()
            .enumerate()
            .filter(|(_, button)| match u32::try_from(**button) {
                // `checked_shl` and not `1 << button`, because nothing here may panic: the table is
                // whatever `th07.cfg` holds, and a number past the width of a mask is a file somebody
                // edited rather than a button.
                Ok(button) => 1u32
                    .checked_shl(button)
                    .is_some_and(|bit| reading.buttons & bit != 0),
                // Negative, which names no button — the game's own check at 0x43037d.
                Err(_) => false,
            })
            .fold(0, |word, (bit, _)| word | 1 << bit);
        let stick = {
            let (up, down) = crate::joystick::axis(reading.y);
            let (left, right) = crate::joystick::axis(reading.x);
            directions((up, down, left, right))
        };
        let hat = if hats {
            directions(crate::joystick::hat(reading.pov))
        } else {
            0
        };
        buttons | stick | hat
    }

    /// The run given up: the shown score caught up with the run's, the menu flag the game's own pause menu
    /// clears cleared, and the scene asked for — 7 where a replay was being watched, and **the front end's
    /// otherwise**.
    ///
    /// **Not [`SCENE_RUN_OVER`], which is what that menu asks for**, and the difference is one call in the
    /// game's own transition out of a run: both of them tear the run down through 0x42f45d, and 6 then
    /// registers the screen a run ends on with `0x44a302(1)` (0x43803c) while the front end's registers
    /// nothing (0x43801f). That screen is drawn — a launch of 妖々夢 showed a frame or two of its background
    /// on the way back to the title — and a run given up through orb's own menu is not a run anybody is being
    /// shown the score of. The replay's scene is left as it is: what a replay ends at is not that screen.
    ///
    /// False where there is no run to give up, which is any scene but the two a run is.
    unsafe fn leave_run(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        if scene != SCENE_RUN && scene != SCENE_RUN_AGAIN {
            return false;
        }
        if let Some(run) = unsafe { self.run_numbers() } {
            let score: u32 = unsafe { mem::read(run + SCORE) };
            unsafe { mem::write::<u32>(run + SCORE_SHOWN, score) };
        }
        let replay = unsafe { mem::read::<u32>(G_RUN_KIND) } & RUN_IS_A_REPLAY != 0;
        unsafe {
            mem::write::<u8>(G_GAME_MANAGER + OTHER_MENU_UP, 0);
            mem::write::<i32>(
                G_SUPERVISOR + SUPERVISOR_WANTED,
                if replay {
                    SCENE_REPLAY_OVER
                } else {
                    SCENE_FRONT_END
                },
            );
        }
        true
    }

    /// The word the read produced put where the run reads it, with last frame's kept beside it — which is what
    /// the recorder does at 0x442cf2 and 0x442d07, made to happen at the read instead of at chain priority 0x10.
    ///
    /// **Nothing while a replay is being watched**: what the run acts on there is the recording, which that same
    /// recorder writes into this word out of the replay's own entry. A live keyboard put there would be the
    /// player playing over the playback for the eight priorities between the read and the copy.
    ///
    /// The recorder still runs and still copies, and copies the same value: what it keeps as last frame's is
    /// then this frame's, which is why the pair is written here in the same order — the next frame's read puts
    /// the word that has just been acted on into last frame's before it moves the other on.
    unsafe fn hand_the_read_to_the_run(&self, word: u16) {
        if unsafe { mem::read::<u32>(G_RUN_KIND) } & RUN_IS_A_REPLAY != 0 {
            return;
        }
        unsafe {
            let acted_on = mem::read::<u16>(G_INPUT_THE_RUN_ACTS_ON);
            mem::write::<u16>(G_INPUT_THE_RUN_ACTED_ON, acted_on);
            mem::write::<u16>(G_INPUT_THE_RUN_ACTS_ON, word);
        }
    }

    /// Every button held, in the word the front end's screens compare against last frame's: the read writes
    /// this frame's over it and keeps this value as last frame's, so nothing the key is still down for reads as
    /// a new press. See [`G_INPUT_READ`], and `Game::menu_cancel` for what a mask of two buttons costs against
    /// a word of all ones.
    unsafe fn swallow_input(&self) {
        unsafe { mem::write::<u16>(G_INPUT_READ, u16::MAX) };
    }

    /// The two masks the game's own front end reads a press and a back out of — see [`MENU_DECIDE`].
    fn menu_decide(&self) -> u16 {
        MENU_DECIDE
    }

    fn menu_cancel(&self) -> u16 {
        MENU_CANCEL
    }

    /// The screen taken to its own way out — [`RESULT_LEAVING`] with the timer at nothing, which is what its
    /// menu writes for the same two answers — and only from the phases the offer is in.
    ///
    /// The scene is the one read on every other frame, which is what keeps this off the cost of a run's frame:
    /// there is no such screen outside it, so nothing else is asked. See [`RESULT_OFFERING_A_REPLAY`].
    unsafe fn skip_replay_prompt(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        if scene != SCENE_RUN_OVER {
            return false;
        }
        let on_the_chain = unsafe { Self::chain_argument(RESULT_UPDATE) };
        let Some(screen) = on_the_chain else {
            return false;
        };
        let phase = mem::read_committed::<i32>(screen + RESULT_PHASE);
        if !phase.is_some_and(|phase| RESULT_OFFERING_A_REPLAY.contains(&phase)) {
            return false;
        }
        unsafe {
            mem::write::<i32>(screen + RESULT_TIMER, 0);
            mem::write::<i32>(screen + RESULT_PHASE, RESULT_LEAVING);
        }
        log!("replay: the offer to save one was taken off the screen a run ended on");
        true
    }

    /// The frames the stage has run for, which is the manager's own count — and only while a stage is one of
    /// the two scenes it runs in, so that a frame of the front end is never written down as a stage's.
    unsafe fn stage_frame(&self) -> Option<u32> {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        (scene == SCENE_RUN || scene == SCENE_RUN_AGAIN)
            .then(|| unsafe { mem::read(G_GAME_MANAGER + STAGE_FRAMES) })
    }

    /// The generator itself — see [`G_RANDOM`], which is the word every draw of this game moves.
    unsafe fn generator(&self) -> u16 {
        unsafe { mem::read(G_RANDOM) }
    }

    /// Every bit of the word a run acts on — see [`RUN_INPUT`].
    fn run_input(&self) -> u16 {
        RUN_INPUT
    }

    /// Which run this is, as far as the exe has been read: the character and the shot the added callback
    /// builds its own index out of, the difficulty and stage `read_state` answers with, and the bit that
    /// says one stage is being practised.
    ///
    /// **`practice` is the field here that decides something**: a practice run is one stage on its own, so
    /// [`Th07::run_slot`] gives it no name and nothing of it is written down. It is read out of
    /// [`G_RUN_KIND`], which is where every other reader of it — `read_state`'s own flag among them — reads
    /// it.
    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: unsafe { mem::read(G_GAME_MANAGER + DIFFICULTY) },
            character: i32::from(unsafe { mem::read::<u8>(G_GAME_MANAGER + CHARACTER) }),
            shot_type: i32::from(unsafe { mem::read::<u8>(G_GAME_MANAGER + SHOT_TYPE) }),
            practice: unsafe { mem::read::<u32>(G_RUN_KIND) } & RUN_IS_A_PRACTICE != 0,
            stage: unsafe { mem::read::<i32>(G_STAGE) } - STAGE_COUNTS_FROM,
        }
    }

    /// Which run the shot select is pointing at, for the mark that says a run of it was left unfinished — and
    /// nothing on a practice run's own shot select, whose stage is chosen on the screen after.
    ///
    /// See [`SCREEN_SHOT_SELECT`] for which screens those are, and [`G_DIFFICULTY_CHOSEN`] for why the
    /// difficulty is not read where a run in progress keeps it.
    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        let settled = unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_SCENE) }
            == SCENE_FRONT_END
            && unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_WANTED) } == SCENE_FRONT_END;
        if !settled {
            return None;
        }
        let front = unsafe { Self::chain_argument(FRONT_END_UPDATE) }?;
        let screen = mem::read_committed::<i32>(front + FRONT_END_SCREEN)?;
        if screen != SCREEN_SHOT_SELECT && screen != SCREEN_SHOT_SELECT_EXTRA {
            return None;
        }
        Some(RunStart {
            difficulty: i32::from(mem::read_committed::<u8>(G_DIFFICULTY_CHOSEN)?),
            character: i32::from(mem::read_committed::<u8>(G_CHARACTER_CHOSEN)?),
            // **The cursor, and not [`G_SHOT_CHOSEN`]**: this is the screen the shot is chosen *on*, and that
            // byte is written where the choice is decided (0x459e7d) or the screen left (0x459f72) — out of this
            // very field both times. So while the cursor moves it holds the shot of the run before, and a 妖々夢
            // session was offered 霊夢B's unfinished run with the cursor sitting on 霊夢A.
            //
            // The difficulty and the character above are the screens *before* this one, so their bytes are
            // decided by the time anybody is standing here — see [`G_DIFFICULTY_CHOSEN`].
            shot_type: mem::read_committed::<i32>(front + FRONT_END_CURSOR)?,
            practice: false,
            // A whole run's chapter can be in any of its stages, and which one is what the mark says rather
            // than part of naming the run — the same answer 紅魔郷 gives here.
            stage: 0,
        })
    }

    /// One slot per run there can be one of, named by the three numbers that make it that run — and nothing
    /// for a practice run, which is one stage rather than a run to come back to.
    ///
    /// **Named by the numbers and not by the characters, on purpose.** 妖々夢 has the six names in the exe —
    /// `ReimuA` through `SakuyaB`, eight bytes apart from 0x495b28 downwards, which is `character * 2 + shot`'s
    /// own order — and its five difficulties twelve apart from 0x495628. Neither table is reached from any
    /// code: no address in either range is pushed or indexed anywhere in the exe, so the tie between an index
    /// and a name is the layout's suggestion rather than something read. A directory of runs reads better with
    /// names, and a name against the wrong number would offer somebody 魔理沙's unfinished run as 霊夢's, so
    /// the numbers go in until the tables are read from the other end. 紅魔郷's own answer falls back to this
    /// same shape where its tables do not cover an index.
    fn run_slot(&self, run: &RunStart) -> Option<String> {
        if run.practice {
            return None;
        }
        /// The six difficulties in the order [`DIFFICULTY`] numbers them, which is the order the game's own
        /// table of their names is laid out in — twelve bytes apart from 0x495628 downwards, `Easy` through
        /// `Phantasm`.
        const DIFFICULTIES: [&str; 6] = ["easy", "normal", "hard", "lunatic", "extra", "phantasm"];
        /// And the six shots in `character * 2 + shot`'s order, which is the order of every table the game
        /// indexes by that byte — see [`G_SHOT_COMBINED`], and the player's own load at 0x442478, which
        /// switches on the character and takes `data/player00.anm`, `player01.anm` or `player02.anm`.
        const SHOTS: [&str; 6] = [
            "reimu-a", "reimu-b", "marisa-a", "marisa-b", "sakuya-a", "sakuya-b",
        ];

        let difficulty = usize::try_from(run.difficulty)
            .ok()
            .and_then(|at| DIFFICULTIES.get(at));
        let shot = usize::try_from(run.character * 2 + run.shot_type)
            .ok()
            .and_then(|at| SHOTS.get(at));
        // Numbered where either is not one of this game's, a name being the one thing this cannot make up — and
        // the file has to be some run's either way. Which is 紅魔郷's own answer here, and was this game's whole
        // answer until the names went in.
        Some(match (difficulty, shot) {
            (Some(difficulty), Some(shot)) => format!("{difficulty}-{shot}"),
            _ => format!(
                "difficulty{}-character{}-shot{}",
                run.difficulty, run.character, run.shot_type
            ),
        })
    }

    /// The numbers of the run this stage started with, all but three of them.
    ///
    /// **`power_items`, `extra_lives` and `rank` are nothing, and each for its own reason.** 妖々夢 gives its
    /// power in whole steps rather than counting items towards one, so there is no count to read. Its extra
    /// lives are paid for out of the cherry rather than the score, so there is no count of ones already paid
    /// either — what pays for them is the three below, [`G_CHERRY`] and the two beside it. And it has no rank
    /// at all: the field that looked like one is the next step of the point value, which [`SCORE`] says.
    ///
    /// **The cherry is answered as a count from the run's own baseline**, which is how the game reads it
    /// everywhere: what a stage played again has to start with is the count, and the baseline is whatever that
    /// stage's own build left.
    unsafe fn run_state(&self) -> RunState {
        let run = unsafe { self.run_numbers() };
        let cherry = |at: usize| {
            run.map_or(0, |run| {
                let baseline = unsafe { mem::read::<i32>(run + SCORE_BASELINE) };
                unsafe { mem::read::<i32>(at) }.wrapping_sub(baseline)
            })
        };
        RunState {
            score: run.map_or(0, |run| unsafe { mem::read(run + SCORE) }),
            seed: unsafe { mem::read(G_GAME_MANAGER + SEED_AT_STAGE_START) },
            point_items: run.map_or(0, |run| unsafe { mem::read::<u32>(run + POINT_ITEMS) }
                as u16),
            power: run.map_or(0, |run| unsafe { mem::read::<f32>(run + POWER) } as u16),
            lives: run.map_or(0, |run| unsafe { mem::read::<f32>(run + LIVES) } as i8),
            bombs: run.map_or(0, |run| unsafe { mem::read::<f32>(run + BOMBS) } as i8),
            rank: 0,
            power_items: 0,
            extra_lives: 0,
            deaths: run.map_or(0, |run| unsafe { mem::read::<f32>(run + DEATHS) } as i32),
            cherry: cherry(G_CHERRY),
            cherry_target: cherry(G_CHERRY_TARGET),
            cherry_plus: cherry(G_CHERRY_PLUS),
        }
    }

    /// The numbers back and the game asked to seal them, with the generator held still over that call — see
    /// [`RESEAL`], which draws twice.
    ///
    /// **And the seed is not written here**, which is where it used to be: it goes in at [`STAGE_BUILD`], before
    /// the build draws the stage out of it. Written here it would be written *after* those draws, and a 妖々夢 run
    /// picked up out of its file parted from the run at the stage's first frame because of it.
    ///
    /// **Every field this writes is one the game seals**, and one write of any of them without the call that
    /// follows is a launch that tears its own graphics down a moment later — [`DEATHS`] says what that looks
    /// like. So the seal is not an afterthought here; it is what makes the writes allowed.
    ///
    /// The three of `RunState` this game has nothing for are not written, and the cherry is — each of its
    /// three against the baseline *this* stage has, which is what the game's own restore does at 0x44387d.
    /// The cherry+ is the one of them that is not written outright: 0x4438c1 holds it to
    /// [`CHERRY_FOR_A_BORDER`] and puts the border up where the record had earned one, which is
    /// [`BORDER_UP`].
    unsafe fn set_run_state(&self, state: &RunState) {
        let run = unsafe { self.run_numbers() };
        let Some(run) = run else { return };
        unsafe {
            mem::write::<u32>(run + SCORE, state.score);
            mem::write::<u32>(run + POINT_ITEMS, u32::from(state.point_items));
            mem::write::<f32>(run + LIVES, f32::from(state.lives));
            mem::write::<f32>(run + BOMBS, f32::from(state.bombs));
            mem::write::<f32>(run + POWER, f32::from(state.power));
            mem::write::<f32>(run + DEATHS, state.deaths as f32);
        }
        // **The seal draws twice, and the generator is put back over those draws** — which is what the game
        // itself does around this very call: 0x42f07e saves [`G_RANDOM`], 0x42f08b seals, 0x42f094 writes the
        // saved word back. A seal whose draws were left standing would be two numbers of a stage's own spent on
        // orb's bookkeeping, and a run picked up out of its file would play a stage nobody played.
        let seal: unsafe extern "thiscall" fn(usize) = unsafe { std::mem::transmute(reseal()) };
        let drawing_from = unsafe { mem::read::<u16>(G_RANDOM) };
        unsafe { seal(G_GAME_MANAGER) };
        unsafe { mem::write::<u16>(G_RANDOM, drawing_from) };
        // The cherry after the seal and outside it: none of these three is a field the seal covers — they are
        // statics rather than the run's own numbers — and the game's own restore writes them here too.
        let baseline = unsafe { mem::read::<i32>(run + SCORE_BASELINE) };
        let earned = state.cherry_plus >= CHERRY_FOR_A_BORDER as i32;
        let plus = if earned {
            CHERRY_FOR_A_BORDER as i32
        } else {
            state.cherry_plus
        };
        unsafe {
            mem::write::<i32>(G_CHERRY, baseline.wrapping_add(state.cherry));
            mem::write::<i32>(G_CHERRY_TARGET, baseline.wrapping_add(state.cherry_target));
            mem::write::<i32>(G_CHERRY_PLUS, baseline.wrapping_add(plus));
        }
        if earned {
            let border: unsafe extern "thiscall" fn(usize) =
                unsafe { std::mem::transmute(border_up()) };
            unsafe { border(G_PLAYER) };
        }
    }

    /// The generator, and the copy the game keeps beside it: the two writes the build makes in the other
    /// order, so that a stage built after this draws what the stage being played again drew.
    unsafe fn set_run_seed(&self, seed: u16) {
        unsafe {
            mem::write::<u16>(G_RANDOM, seed);
            mem::write::<u16>(G_GAME_MANAGER + SEED_AT_STAGE_START, seed);
        }
    }

    /// The front end still up and a run already asked for, which is the one frame where what the run *is* is
    /// settled and none of it has been acted on.
    ///
    /// **The two scenes are what say so, and the order they say it in is the chain's.** The supervisor's own
    /// job is priority 0 and the front end's is priority 3, so an item that asks for a run writes the wanted
    /// scene after the supervisor has already looked this frame: the frame ends with the pair disagreeing, and
    /// the next frame's supervisor tears the front end down and builds the run. Every path agrees on the
    /// write — 0x459ed0, 0x45a65d and 0x45b47d each put 2 there — and [`GAME_MANAGER_ADDED`] is where the
    /// building then happens.
    ///
    /// **And never of the attract demo or a replay**, which ask for a run exactly the same way and have
    /// nobody to ask a question of: both have set their bit in [`G_RUN_KIND`] by then, the demo's at 0x455b5e
    /// and a replay's at 0x455b50 or 0x45b395, before any of those three writes.
    unsafe fn run_chosen(&self) -> bool {
        let scene: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_SCENE) };
        let wanted: i32 = unsafe { mem::read(G_SUPERVISOR + SUPERVISOR_WANTED) };
        let kind: u32 = unsafe { mem::read(G_RUN_KIND) };
        scene == SCENE_FRONT_END
            && wanted == SCENE_RUN
            && kind & (RUN_IS_A_DEMO | RUN_IS_A_REPLAY) == 0
    }

    /// The run about to be built pointed at another of its stages: the word a build reads, written the way a
    /// menu leaves it — orb's own 0-based number, which the build adds one to at 0x42ef50.
    ///
    /// False for a stage this game does not have, which is anything outside the eight the switch at 0x407081
    /// covers: six stages, the Extra and the Phantasm. Nothing else of the run is touched, the difficulty and
    /// the character being the ones its front end just chose.
    unsafe fn start_stage(&self, stage: i32) -> bool {
        if !(0..STAGES).contains(&stage) {
            return false;
        }
        unsafe { mem::write::<i32>(G_STAGE, stage) };
        true
    }

    /// Whether the run finished into the game's own screen for it, which is the phase that screen was brought
    /// up with — see [`RESULT_UPDATE`]. A run given up does not go through it, which is what makes this the
    /// chapter worth coming back to told from the run that has none.
    unsafe fn run_finished(&self) -> bool {
        let result = unsafe { Self::chain_argument(RESULT_UPDATE) };
        let Some(result) = result else { return false };
        mem::read_committed::<i32>(result + RESULT_PHASE)
            .is_some_and(|phase| phase == RESULT_FROM_A_RUN || phase == RESULT_FROM_A_PRACTICE_RUN)
    }

    /// What a run's own frame looks like, for telling a resumed run from the recording it came from — every
    /// field of it read where this game keeps it.
    ///
    /// **Three are nothing and each says why.** `extra_lives` and `rank` and `sub_rank` are 紅魔郷's: this game
    /// pays its extra lives out of the three cherry counters rather than the score and has no rank at all —
    /// [`Th07::run_state`] and [`SCORE`] say where those readings come from. And `player_area` is unread: the
    /// play area is a constant here ([`PLAY_AREA`]) and whether the game narrows the player's own is not
    /// measured, so a zero says *unknown* rather than *unchanged*.
    unsafe fn reproduction(&self) -> Reproduction {
        let run = unsafe { self.run_numbers() };
        Reproduction {
            replay_frame: mem::read_committed::<usize>(G_REPLAY_MANAGER)
                .filter(|manager| *manager != 0)
                .and_then(mem::read_committed::<i32>)
                .unwrap_or(-1),
            input: unsafe { mem::read(G_INPUT_THE_RUN_ACTS_ON) },
            player: (unsafe { mem::read(G_PLAYER + PLAYER_POSITION) }, unsafe {
                mem::read(G_PLAYER + PLAYER_POSITION + size_of::<f32>())
            }),
            player_area: (0.0, 0.0),
            randoms: unsafe { mem::read(G_RANDOM + RANDOMS_DRAWN) },
            seed: unsafe { mem::read(G_RANDOM) },
            items: unsafe { mem::read(G_ITEM_MANAGER + ITEM_COUNT) },
            score: run.map_or(0, |run| unsafe { mem::read(run + SCORE) }),
            extra_lives: 0,
            rank: 0,
            sub_rank: 0,
        }
    }

    /// The table [`chapters`] holds, which is a stage per entry and nothing in any of them yet — so a chapter
    /// falls at a stage's start and nowhere inside it. Its own file for the reason 紅魔郷's is: what a
    /// `--chapter-tuning` pass writes is a `chapters.rs` to paste over, and a table inline here would be a paste
    /// into the middle of this file. Everything about how it is filled is in that module.
    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &chapters::MIDSTAGE
    }
}

/// The four directions as bits of the word this game's own read builds — see [`PAD_UP`] and the three
/// beside it, which is where each bit comes from.
fn directions((up, down, left, right): (bool, bool, bool, bool)) -> u16 {
    [
        (up, PAD_UP),
        (down, PAD_DOWN),
        (left, PAD_LEFT),
        (right, PAD_RIGHT),
    ]
    .into_iter()
    .filter(|(pushed, _)| *pushed)
    .fold(0, |word, (_, bit)| word | bit)
}

/// A name out of the game's own memory, up to the first zero byte and no further than the room it is given —
/// which is what every name this game keeps is: a fixed buffer with a terminated string in it.
///
/// `None` where the memory cannot be read, so a pointer chase that has gone stale answers nothing rather than
/// a name made of whatever was there.
fn name_at(address: usize, room: usize) -> Option<Vec<u8>> {
    let held = mem::read_committed_bytes(address, room)?;
    let end = held
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(held.len());
    Some(held[..end].to_vec())
}

/// The file's own name out of a path, which is everything past the last separator: the player's own lookup
/// walks past both kinds at 0x44bb06, so both are cut here.
fn file_name(path: &[u8]) -> &[u8] {
    let cut = path
        .iter()
        .rposition(|byte| *byte == b'/' || *byte == b'\\')
        .map_or(0, |at| at + 1);
    &path[cut..]
}

/// And the name without its extension, which is what two names for one song have in common: a stage's data
/// carries `bgm/th07_12.mid` where a slot holds `th07_12.wav` and a record holds `th07_12.wav` — so the stem
/// is the whole of what they agree on.
fn file_stem(path: &[u8]) -> &[u8] {
    let name = file_name(path);
    match name.iter().rposition(|byte| *byte == b'.') {
        Some(at) => &name[..at],
        None => name,
    }
}

/// Whether two names are the same song, which is their stems held against each other — see [`file_stem`].
fn same_song(one: &[u8], other: &[u8]) -> bool {
    file_stem(one).eq_ignore_ascii_case(file_stem(other))
}

/// One more attempt at a card, counted the way the game counts one: left where it is at the cap rather than
/// wrapping or stopping one short of it — see [`CARD_COUNT_CAP`].
fn counted_once_more(count: u16) -> u16 {
    if count >= CARD_COUNT_CAP {
        CARD_COUNT_CAP
    } else {
        count + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::th07::image::Image;

    /// The records of `thbgm.dat` the tests over the music are played out of: the title screen's, which is the
    /// file's first record and so what a play of a slot nothing was loaded into comes out as; the stage's own; and
    /// the one the fight it ends with brings. Small, and no two of them sharing a place in the file, so a restore
    /// putting one where another goes is readable.
    const TITLE_TRACK: image::Track = image::Track {
        name: c"th07_01.wav",
        at: 0x0,
        loops_at: 0x80,
        bytes: 0x200,
    };
    const STAGE_TRACK: image::Track = image::Track {
        name: c"th07_02.wav",
        at: 0x1000,
        loops_at: 0x40,
        bytes: 0x400,
    };
    const FIGHT_TRACK: image::Track = image::Track {
        name: c"th07_03.wav",
        at: 0x1400,
        loops_at: 0x40,
        bytes: 0x400,
    };

    /// **A state read asks the memory map about the addresses the game allocated and about no others**,
    /// which is what keeps it costing what reads cost: every ask is a `VirtualQuery` in a launch, and a
    /// syscall is not what reading a word out of the exe's own `.data` is worth.
    ///
    /// Measured, and it is why this is here: orb's own per-frame work was **1206us a frame** on 妖々夢
    /// against **18us** on 紅魔郷 — `perf: update=1206us/frame` — and a run picked up out of its file took
    /// 3.2 seconds to play 2667 frames of buttons in, at 1.2ms an update, where the same thing on 紅魔郷
    /// runs 13675 of them in 0.8s. The state read is made twice a frame and the laser count walked all
    /// 0x40 slots through the map.
    ///
    /// The bound is the slots the fields behind a pointer need and nothing like the count a walk of any
    /// of the game's own arrays would run up: what is asked about here is `G_RUN_NUMBERS`'s own object
    /// and the dialogue the stage built, and both are the allocator's.
    #[test]
    fn a_state_read_asks_the_memory_map_only_about_what_the_game_allocated() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        image.has_a_runs_numbers();
        image.a_boss_is_fighting(1500, 240);
        image.lasers_alive(LASER_SLOTS);

        let before = image.space().asked_about_an_address();
        let read = unsafe { Th07.read_state() };
        let asked = image.space().asked_about_an_address() - before;
        assert_eq!(read.laser_count, LASER_SLOTS as i32);
        assert!(
            asked <= ASKS_A_STATE_READ_MAKES,
            "a state read asked the memory map {asked} time(s), which is more than the \
             {ASKS_A_STATE_READ_MAKES} the game's own allocations are behind",
        );
    }

    /// What that bound is: the run's own numbers, and the dialogue over them.
    const ASKS_A_STATE_READ_MAKES: u64 = 4;

    /// The boss, its life, how long its attack has run and the card it is on — the four a chapter's
    /// boundaries are drawn from, and the one place orb follows a pointer the game hands it.
    #[test]
    fn a_boss_is_the_pointer_its_manager_keeps_and_the_life_behind_it() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        let none = unsafe { Th07.read_state() };
        assert!(!none.boss_present);
        assert_eq!((none.boss_life, none.boss_attack_frames), (None, None));
        assert_eq!(none.spellcard, None);

        image.a_boss_is_fighting(1500, 240);
        let fighting = unsafe { Th07.read_state() };
        assert!(fighting.boss_present);
        assert_eq!(
            (fighting.boss_life, fighting.boss_attack_frames),
            (Some(1500), Some(240))
        );
        // A boss without a card is a boss: the two are separate reads, and 妖々夢's non-card attacks are
        // where a chapter is taken as much as its cards are.
        assert_eq!(fighting.spellcard, None);

        image.the_spellcard_is(96);
        assert_eq!(unsafe { Th07.read_state() }.spellcard, Some(96));
    }

    /// And neither count goes past where the game's own two stop.
    #[test]
    fn a_card_counted_up_to_the_games_cap_is_not_counted_past_it() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        image.the_spellcard_is(96);
        image.a_card_has_been_attempted(96, 0);
        image.a_cards_counts_are_at_the_cap(96);

        assert_eq!(unsafe { Th07.count_card_attempt() }, Some(CARD_COUNT_CAP));
        assert_eq!(image.card_attempts(96, 0), CARD_COUNT_CAP);
        assert_eq!(image.card_attempts_in_all(96), CARD_COUNT_CAP);
    }

    /// A cherry+ at or past the figure that earns a border, which the game's own restore does two things
    /// about: holds the count to that figure, and puts the border up.
    #[test]
    fn a_cherry_plus_that_has_earned_a_border_is_put_back_as_one() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        image.has_a_runs_numbers();
        let _sealed = image.hands_over_the_seal();
        image.the_cherry_is(1_000_000, 0, 0, 0);
        let bordered = image.hands_over_the_border();

        let start = RunState {
            cherry: 0x1000,
            cherry_target: 0xc350,
            cherry_plus: CHERRY_FOR_A_BORDER as i32 + 0x500,
            ..RunState {
                score: 0,
                seed: 0,
                point_items: 0,
                power: 0,
                lives: 0,
                bombs: 0,
                rank: 0,
                power_items: 0,
                extra_lives: 0,
                deaths: 0,
                cherry: 0,
                cherry_target: 0,
                cherry_plus: 0,
            }
        };
        unsafe { Th07.set_run_state(&start) };

        assert_eq!(
            image.the_cherry(),
            (0x1000, 0xc350, CHERRY_FOR_A_BORDER as i32),
            "the cherry+ was left past the figure a border is earned at"
        );
        assert_eq!(bordered(), 1, "the border the recording had was not put up");
    }

    /// And the dialogue, whose numbering is the game's own: no object at all before a stage is loaded, then
    /// -1 for no dialogue, a whole number for a line, and -2 for the line it hands the screen over on — which
    /// counts as a dialogue because the game's own answer at 0x42ad66 counts it, and that answer is what
    /// refuses a bomb.
    #[test]
    fn a_dialogue_is_the_line_the_games_own_answer_counts() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(
            !unsafe { Th07.read_state() }.in_dialogue,
            "a stage with no dialogue object read as a dialogue through a null pointer"
        );

        image.has_a_dialogue();
        assert!(!unsafe { Th07.read_state() }.in_dialogue);

        for line in [0, 3] {
            image.the_dialogue_is_at_line(line);
            assert!(unsafe { Th07.read_state() }.in_dialogue, "line {line}");
        }

        image.the_dialogue_is_at_line(DIALOGUE_HANDING_OVER);
        assert!(unsafe { Th07.read_state() }.in_dialogue);

        image.the_dialogue_is_at_line(-3);
        assert!(!unsafe { Th07.read_state() }.in_dialogue);
    }

    /// Every field of a frame's reproduction read where this game keeps that field, which is what a resumed run
    /// is held against: the word the run acts on, the player's position, the generator and how many numbers it
    /// has drawn, the items on the screen, the run's score, and the frame a playback is on.
    ///
    /// **The replay's frame is -1 where nothing is playing back**, rather than 0: frame 0 is a replay on its
    /// first frame, and a resume that compared the two would call a landing right on a run that has no replay at
    /// all.
    ///
    /// Four of the fields are nothing here and the test says so: this game pays its extra lives out of the cherry
    /// rather than the score and has no rank of either kind, and whether it narrows the player's own area is
    /// unmeasured — see [`Th07::reproduction`].
    #[test]
    fn a_frames_reproduction_is_read_where_this_game_keeps_each_of_them() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(3, 1);
        image.has_a_runs_numbers();
        image.the_run_has_scored(7_654_321, 0);
        image.the_run_acts_on_the_word(PAD_LEFT | PAD_UP);
        image.the_player_is_at(-64.0, 128.0);
        image.generator_is_at(0x4321);
        image.the_generator_has_drawn(909);
        image.items_alive(12);

        let landed = unsafe { Th07.reproduction() };
        assert_eq!(landed.input, PAD_LEFT | PAD_UP);
        assert_eq!(landed.player, (-64.0, 128.0));
        assert_eq!((landed.seed, landed.randoms), (0x4321, 909));
        assert_eq!(landed.items, 12);
        assert_eq!(landed.score, 7_654_321);
        assert_eq!(landed.replay_frame, -1, "no replay is playing back");
        assert_eq!(
            (
                landed.extra_lives,
                landed.rank,
                landed.sub_rank,
                landed.player_area
            ),
            (0, 0, 0, (0.0, 0.0))
        );

        // And a replay being watched: the frame is that manager's own counter.
        image.a_replay_covers_the_stage(3);
        image.the_replay_is_at(4242);
        assert_eq!(unsafe { Th07.reproduction() }.replay_frame, 4242);
    }

    /// A menu of the game's own stops a run, which is what `paused` says: the update refuses to run the
    /// frame's logic under either flag, so nothing of orb's should act on such a frame either.
    #[test]
    fn a_menu_of_the_games_own_is_the_run_paused() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(!unsafe { Th07.read_state() }.paused);
        image.is_paused();
        assert!(unsafe { Th07.read_state() }.paused);
    }

    /// The three frames a chapter cannot be taken on, which are the player's rather than the run's: dying,
    /// bombing, and a dialogue being played.
    #[test]
    fn a_player_that_is_not_playing_is_no_place_for_a_chapter() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        let settled = unsafe { Th07.read_state() };
        assert!(!settled.unsettled && !settled.bombing && !settled.in_dialogue);

        image.the_player_is_dying();
        assert!(unsafe { Th07.read_state() }.unsettled);

        image.a_bomb_is_going_off();
        assert!(unsafe { Th07.read_state() }.bombing);
    }

    /// And one stage practised rather than the whole game played, which is the other bit of that word orb
    /// reads. Here for the same reason: a practice run has no ranking to write and no next stage to go to,
    /// so what it reaches is orb's behaviour and not the log's line.
    #[test]
    fn a_practice_run_is_the_bit_the_games_own_menu_sets_for_that_item() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        assert!(!unsafe { Th07.read_state() }.practice);
        image.is_practising();
        let state = unsafe { Th07.read_state() };
        assert!(state.practice);
        // And not the replay bit with it: the two are separate items of the same menu, and a run being
        // watched is not a run being practised.
        assert!(!state.replay);
    }

    /// A replay being watched, which is the same bit `State` carries and is asked here for the other reason:
    /// its record of inputs is one nothing may write into.
    #[test]
    fn a_replay_being_watched_is_a_record_nothing_may_write_into() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(!unsafe { Th07.replaying() });
        image.is_watching_a_replay();
        assert!(unsafe { Th07.replaying() });
    }

    /// A replay taken to another of its stages, which is the game's own way between them — and refused for a
    /// stage it does not cover, since asking for one drops the game to its menu.
    #[test]
    fn a_replay_is_taken_to_a_stage_it_covers_and_no_further() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(
            !unsafe { Th07.jump_to_stage(3) },
            "there is no replay to take anywhere"
        );

        image.a_replay_covers_the_stage(3);
        assert!(!unsafe { Th07.jump_to_stage(2) }, "a stage with no record");
        assert!(
            !unsafe { Th07.jump_to_stage(REPLAY_STAGES) },
            "past the end"
        );
        assert!(!unsafe { Th07.jump_to_stage(-1) }, "before the start");

        assert!(unsafe { Th07.jump_to_stage(3) });
        // The stage as a menu leaves it and the scene that rebuilds the manager, which is what the game's own
        // transition between stages does.
        let state = unsafe { Th07.read_state() };
        assert_eq!(state.wanted, SCENE_RUN_AGAIN);
        assert_eq!(image.space().read::<i32>(G_STAGE), 3);
    }

    /// What kind of run this is, read out of the game's own word for it: the bit a replay's load sets, and
    /// nothing else in that word touched.
    ///
    /// Here rather than in an e2e test because the log has nowhere to say it: `State`'s own line prints the
    /// scene, the stage and the numbers, and what a run *is* reaches orb's behaviour rather than its
    /// output — a replay is a run nobody is playing, so no retry is offered to one.
    #[test]
    fn a_replay_is_the_bit_the_game_sets_as_it_loads_one() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        assert!(!unsafe { Th07.read_state() }.replay);
        image.is_watching_a_replay();
        assert!(unsafe { Th07.read_state() }.replay);
    }

    /// The run about to be built pointed at one of its stages, which is all picking a run up asks of the front
    /// end — and refused for a stage this game does not have.
    #[test]
    fn a_run_being_built_is_pointed_at_a_stage_the_game_has() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        for stage in 0..STAGES {
            assert!(unsafe { Th07.start_stage(stage) }, "stage {stage}");
            // The word as a menu leaves it, which is one less than what orb reads once the build has added
            // its own — the trap [`G_STAGE`] carries.
            assert_eq!(image.space().read::<i32>(G_STAGE), stage);
        }
        assert!(!unsafe { Th07.start_stage(STAGES) }, "past the Phantasm");
        assert!(!unsafe { Th07.start_stage(-1) }, "before the first");
    }

    /// A slot per run there can be one of, which is what a chapter left behind is written into — and nothing
    /// for a practice run.
    #[test]
    fn a_run_has_a_slot_of_its_own_and_a_practice_run_has_none() {
        let run = RunStart {
            difficulty: 3,
            character: 2,
            shot_type: 1,
            practice: false,
            stage: 0,
        };
        // **Named and not numbered**, which is what the directory of runs left unfinished is read as: 咲夜's
        // second shot on Lunatic. See [`Th07::run_slot`] for where each half of that name comes from.
        assert_eq!(Th07.run_slot(&run).as_deref(), Some("lunatic-sakuya-b"));
        assert_eq!(
            Th07.run_slot(&RunStart {
                difficulty: 5,
                character: 0,
                shot_type: 0,
                ..run
            })
            .as_deref(),
            Some("phantasm-reimu-a"),
        );
        // And a run whose numbers are not any of this game's is numbered rather than named, a name being what
        // this cannot make up: a file has to be one run's either way.
        assert_eq!(
            Th07.run_slot(&RunStart {
                difficulty: 9,
                character: 4,
                shot_type: 1,
                ..run
            })
            .as_deref(),
            Some("difficulty9-character4-shot1"),
        );
        // Every run that is a different run gets a different slot, which is the whole of what this is for.
        let other = RunStart {
            shot_type: 0,
            ..run
        };
        assert_ne!(Th07.run_slot(&run), Th07.run_slot(&other));
        assert_eq!(
            Th07.run_slot(&RunStart {
                practice: true,
                ..run
            }),
            None
        );
    }

    /// And the scene, the stage and the difficulty, which are the three the state line does say.
    #[test]
    fn a_run_in_progress_is_its_scene_its_stage_and_its_difficulty() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        let state = unsafe { Th07.read_state() };
        assert!(!state.in_run && !state.playing);

        image.is_playing(4, 3);
        let state = unsafe { Th07.read_state() };
        assert!(state.in_run && state.playing);
        assert_eq!(
            (state.scene, state.stage, state.difficulty),
            (SCENE_RUN, 4, 3)
        );
    }

    /// The frame a run has been asked for and nothing of it built, which is where a chapter offered from an
    /// earlier launch has to be put: the front end still the scene, a run the scene wanted, and neither of
    /// the two runs nobody is playing.
    #[test]
    fn a_run_is_chosen_on_the_frame_the_front_end_asked_for_one() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.the_front_end_is_up();
        assert!(!unsafe { Th07.run_chosen() }, "nothing has been asked for");

        image.is_in_scene(SCENE_FRONT_END, SCENE_RUN);
        assert!(unsafe { Th07.run_chosen() });

        // And not once the run is built, which is the scene having caught up with what was wanted.
        image.is_in_scene(SCENE_RUN, SCENE_RUN);
        assert!(!unsafe { Th07.run_chosen() });

        // Nor for the attract demo or a replay, which ask for a run the same way and have nobody to ask.
        image.is_in_scene(SCENE_FRONT_END, SCENE_RUN);
        image.is_the_attract_demo();
        assert!(!unsafe { Th07.run_chosen() });
    }

    /// The run given up back to the front end, and nothing to give up anywhere else.
    ///
    /// **The front end and not the scene the game's own pause menu asks for**: both tear the run down through
    /// 0x42f45d and only that one registers the screen a run ends on (0x43803c against 0x43801f), whose
    /// background is drawn on the way past — see [`Th07::leave_run`].
    #[test]
    fn a_run_is_given_up_to_the_front_end_rather_than_the_screen_a_run_ends_on() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.the_front_end_is_up();
        assert!(!unsafe { Th07.leave_run() }, "there is no run to give up");

        image.is_playing(2, 1);
        image.has_a_runs_numbers();
        image.the_run_has_scored(4_242_424, 0);
        image.is_paused();
        assert!(unsafe { Th07.leave_run() });
        let state = unsafe { Th07.read_state() };
        assert_eq!(state.wanted, SCENE_FRONT_END);
        assert_eq!(
            image
                .space()
                .read::<u32>(image.run_numbers_at() + SCORE_SHOWN),
            4_242_424,
            "the shown score did not catch up with the run's"
        );

        // A replay leaves for its own screen instead, which is where it was picked.
        image.is_in_scene(SCENE_RUN, SCENE_RUN);
        image.is_watching_a_replay();
        assert!(unsafe { Th07.leave_run() });
        assert_eq!(unsafe { Th07.read_state() }.wanted, SCENE_REPLAY_OVER);
    }

    /// A run that ended told from one that was given up, which is the phase the screen it ends on was brought
    /// up with — a run given up never reaches that screen at all.
    #[test]
    fn a_run_that_ended_is_the_screen_it_ended_on_and_why_it_came_up() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(!unsafe { Th07.run_finished() }, "the run is still going");

        image.the_run_ended_screen_is_up(RESULT_FROM_A_RUN);
        assert!(unsafe { Th07.run_finished() });

        image.the_screen_was_brought_up_by(RESULT_FROM_A_PRACTICE_RUN);
        assert!(
            unsafe { Th07.run_finished() },
            "a practice run ends into the same screen"
        );

        // And the other way in is not a run ending: the ranking asked for from the front end.
        image.the_screen_was_brought_up_by(0x13);
        assert!(!unsafe { Th07.run_finished() });
    }

    /// The numbers a stage started with, which is what a run picked up in a later launch is given back, and
    /// the seed put in the two places the game keeps it.
    ///
    /// Three of `RunState`'s ten are nothing here and the test says so: this game keeps no count of power
    /// items and no count of extra lives paid for, and it has no rank — see [`SCORE`] for what `+0x30`, the
    /// field that looked like one, turned out to be.
    #[test]
    fn a_stages_numbers_are_the_runs_own_and_the_seed_is_kept_twice() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        image.has_a_runs_numbers();
        image.has_lives_and_bombs(2.0, 3.0);
        image.has_died(1.0);
        image.shot_is_at_power(96.0);
        image.the_run_has_scored(12_345_678, 640);
        image.the_stage_began_with_the_seed(0xbeef);

        let start = unsafe { Th07.run_state() };
        assert_eq!(start.score, 12_345_678);
        assert_eq!(start.point_items, 640);
        assert_eq!(start.seed, 0xbeef);
        assert_eq!((start.lives, start.bombs, start.deaths), (2, 3, 1));
        assert_eq!(start.power, 96);
        assert_eq!(
            (start.rank, start.power_items, start.extra_lives),
            (0, 0, 0)
        );

        // And the seed put back into both places: the generator, which the stage draws from, and the copy the
        // game itself keeps for a replay — a stage given one and not the other plays out differently the next
        // time it is written down.
        unsafe { Th07.set_run_seed(0x1234) };
        assert_eq!(unsafe { Th07.read_state() }.random_seed, 0x1234);
        assert_eq!(unsafe { Th07.run_state() }.seed, 0x1234);
    }

    /// And those numbers put back: every field this game has written, the game asked to seal them, and **the
    /// generator left exactly where it was**.
    ///
    /// The seal draws twice, which is why the game saves the generator around its own call to it (0x42f07e,
    /// 0x42f094) and why orb does the same: two numbers spent here are two a stage did not draw, and a run picked
    /// up out of its file would then play a stage nobody played.
    ///
    /// **And the seed is not one of the fields**: it goes in at [`STAGE_BUILD`], before the build draws the stage
    /// out of it, so what stands here afterwards is the seed the stage was already going to have.
    #[test]
    fn a_stages_numbers_go_back_sealed_with_the_generator_where_it_was() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        let sealed = image.hands_over_the_seal();
        let _bordered = image.hands_over_the_border();
        image.is_playing(0, 0);

        let going_back = RunState {
            score: 98_765_432,
            seed: 0x5555,
            point_items: 128,
            power: 128,
            lives: 4,
            bombs: 2,
            deaths: 3,
            rank: 0,
            power_items: 0,
            extra_lives: 0,
            cherry: 0,
            cherry_target: 0,
            cherry_plus: 0,
        };
        // Nothing at all before a run is built, there being no numbers to write into — and nothing asked of the
        // game either, a seal over numbers that never went back being a seal over somebody else's.
        unsafe { Th07.set_run_state(&going_back) };
        assert_eq!(sealed(), 0);

        image.has_a_runs_numbers();
        image.the_stage_began_with_the_seed(0xaaaa);
        image.generator_is_at(0x1234);
        unsafe { Th07.set_run_state(&going_back) };

        assert_eq!(sealed(), 1, "the numbers went back unsealed");
        assert_eq!(
            image.the_seed_it_sealed_over(),
            0x1234,
            "the seal ran over some other number than the one standing"
        );
        // **And the generator is where it was**, the seal's own two draws put back over: what a stage draws is
        // the stage's, and two of them spent here would be a stage nobody played — see `STAGE_BUILD`.
        assert_eq!(unsafe { Th07.read_state() }.random_seed, 0x1234);
        // The seed itself is untouched here, going in at the build instead.
        assert_eq!(unsafe { Th07.run_state() }.seed, 0xaaaa);
        assert_eq!(
            unsafe { Th07.run_state() },
            RunState {
                seed: 0xaaaa,
                ..going_back
            },
            "every number but the seed is the one that went in"
        );
    }

    /// The window asked for out of the game's own configuration, and asked for on its behalf where a launch
    /// wants one — which is the byte both its window and its device are built from.
    #[test]
    fn a_window_is_asked_for_where_the_game_would_have_taken_the_display() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_configured_for_a_window(false);
        assert!(!Th07.windowed());

        unsafe { Th07.force_windowed() };
        assert!(image.asks_for_a_window());
        assert!(Th07.windowed());

        // And a launch that was going to make one anyway is left as it is.
        unsafe { Th07.force_windowed() };
        assert!(Th07.windowed());
    }

    /// An attempt counted where the game itself cannot count one — a chapter begun inside a card never starts
    /// that card — and nothing at all where there is no card up or no record the game has written.
    #[test]
    fn an_attempt_is_counted_only_into_a_record_the_game_itself_wrote() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert_eq!(
            unsafe { Th07.count_card_attempt() },
            None,
            "no card is being fought"
        );

        // A card up, but its slot never written by the game: nothing counted into it.
        image.the_spellcard_is(96);
        assert_eq!(unsafe { Th07.count_card_attempt() }, None);

        // The game's own record for it, and the count going up from where the game left it.
        image.a_card_has_been_attempted(96, 7);
        assert_eq!(unsafe { Th07.count_card_attempt() }, Some(8));
        assert_eq!(unsafe { Th07.count_card_attempt() }, Some(9));

        // Under this run's own shot and not another's, which is the word the ranking draws for it.
        assert_eq!(image.card_attempts(96, 0), 9);
        assert_eq!(image.card_attempts(96, 1), 0, "another shot's word");
    }

    /// An ending, and the script it is running: the scene says one is running and the script is behind a
    /// pointer only the chain holds, so this is the one field orb walks the game's own chain for.
    #[test]
    fn an_ending_is_its_scene_and_the_script_only_the_chain_can_find() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        let run = unsafe { Th07.read_state() };
        assert!(!run.in_ending);
        assert_eq!(run.ending_script, None);

        image.an_ending_is_running(0x0334_2000);
        let ending = unsafe { Th07.read_state() };
        assert!(ending.in_ending);
        assert_eq!(ending.ending_script, Some(0x0334_2000));
        // And it is no longer a run: the ending is its own scene, so nothing of a run's is offered there.
        assert!(!ending.playing && !ending.in_run && !ending.in_game);
    }

    /// The row the lives are shown in asked for, which is two things here: the row itself, and the ground under
    /// it that nothing else lays again.
    #[test]
    fn asking_for_the_lives_row_asks_for_the_ground_under_it_too() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.has_a_panel();
        // Another row already asked for, which orb may not answer for: the bombs' pair is the two bits above
        // the lives'.
        image.a_row_is_already_asked_for(0b1100);

        unsafe { Th07.repaint_lives_row() };
        assert_eq!(image.the_rows_to_paint(), 0b1110);
        assert_eq!(image.the_panels_ground_is_laid_for(), 2);

        // And asked for again on the next frame, which is what every frame of a mark does.
        unsafe { Th07.repaint_lives_row() };
        assert_eq!(image.the_rows_to_paint(), 0b1110);
    }

    /// And nothing at all where the game has not built what it puts over a run, which is every frame outside a
    /// stage: the plate is inside that object and there is none.
    #[test]
    fn no_name_is_baked_where_the_game_has_nothing_to_bake_it_into() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();
        image.hands_over_the_text_baking();

        image.is_playing(0, 0);
        image.the_spellcard_is(96);
        image.a_card_has_been_attempted(96, 1);
        image.the_card_is_named(96, b"Card of a card");

        unsafe { Th07.redraw_card_name() };
        assert_eq!(image.bakings(), 0);
    }

    /// And the attract demo, the third bit orb reads, with `in_game` the whole point of it: a run nobody is
    /// playing is one no retry is offered to, and the demo and a replay are the two of those.
    #[test]
    fn the_attract_demo_is_a_run_orb_is_not_in_the_game_of() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        let played = unsafe { Th07.read_state() };
        assert!(!played.demo && played.in_game);

        image.is_the_attract_demo();
        let demo = unsafe { Th07.read_state() };
        assert!(demo.demo);
        assert!(!demo.in_game, "orb would offer a retry to the attract demo");
        // Still the gameplay scene, and still a run: the demo is played in the same scene by the same
        // manager, so everything but `in_game` reads as it did.
        assert!(demo.playing && demo.in_run);
    }

    /// The bullets and the lasers, which the game counts in two different ways: one field it keeps and one
    /// walk it does, so orb does the walk too.
    #[test]
    fn the_bullets_are_counted_for_orb_and_the_lasers_are_counted_by_orb() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        let empty = unsafe { Th07.read_state() };
        assert_eq!((empty.bullet_count, empty.laser_count), (0, 0));

        image.bullets_alive(342);
        image.lasers_alive(5);
        let full = unsafe { Th07.read_state() };
        assert_eq!((full.bullet_count, full.laser_count), (342, 5));

        // Every slot of the game's own array, which is what says orb walks the whole of it: a walk that
        // stopped at the first dead slot would answer nothing here.
        image.lasers_alive(LASER_SLOTS);
        assert_eq!(unsafe { Th07.read_state() }.laser_count, LASER_SLOTS as i32);
    }

    /// The buttons swallowed, so that the key which answered a question of orb's is not a press the screen
    /// behind it acts on when the game runs again.
    #[test]
    fn the_buttons_are_swallowed_out_of_the_word_the_front_end_compares() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.the_read_left_the_word(MENU_DECIDE);
        unsafe { Th07.swallow_input() };
        assert_eq!(
            image.the_word_the_read_left(),
            u16::MAX,
            "the frame the game carries on into has a new press on it"
        );
    }

    /// The captures emptied where a ranking read would otherwise add to them, and kept on the way out of a
    /// run — where the record in memory is that run's own.
    #[test]
    fn the_captures_are_emptied_for_a_ranking_read_and_kept_for_a_runs_own() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.a_card_has_been_attempted(96, 5);
        let counted = unsafe { Th07.captures() };

        // A run ending into its screen: the record is the run's, so the read that follows adds to it.
        image.is_in_scene(SCENE_RUN_OVER, SCENE_RUN_OVER);
        unsafe { Th07.forget_captures(std::ptr::null_mut()) };
        assert_eq!(unsafe { Th07.captures() }, counted);

        // The ranking asked for from the front end: the read defines the history, so what is in memory goes.
        image.is_in_scene(SCENE_RANKING, SCENE_RANKING);
        unsafe { Th07.forget_captures(std::ptr::null_mut()) };
        assert!(unsafe { Th07.captures() }.iter().all(|byte| *byte == 0));
    }

    /// The name of the card being fought put back on the plate, which is the one part of it a snapshot cannot
    /// carry: the sprite it is baked into is a texture. Only while a card is up, only out of a record the game
    /// has named, and out of that record rather than out of the declaration the interpreter has moved past.
    #[test]
    fn the_card_being_fought_has_its_name_baked_onto_the_plate_again() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();
        image.hands_over_the_text_baking();

        image.is_playing(0, 0);
        image.has_what_it_puts_over_a_run();

        // No card up: nothing baked, the plate being the card's and there being none.
        unsafe { Th07.redraw_card_name() };
        assert_eq!(image.bakings(), 0);

        // A card up whose slot the game has never written: still nothing, the bytes there being no name.
        image.the_spellcard_is(96);
        unsafe { Th07.redraw_card_name() };
        assert_eq!(image.bakings(), 0);

        // The game's own record for it, and no name in it yet: a plate blanked is a worse answer than one left
        // as it is.
        image.a_card_has_been_attempted(96, 1);
        unsafe { Th07.redraw_card_name() };
        assert_eq!(image.bakings(), 0);

        // And named, which is what the card's own start leaves there.
        image.the_card_is_named(96, "幽夢「猫の見た夢」".as_bytes());
        unsafe { Th07.redraw_card_name() };
        assert_eq!(image.bakings(), 1);
        assert_eq!(image.the_plate_says(), "幽夢「猫の見た夢」".as_bytes());
    }

    /// The card record held across what would replace it, taken and put back whole — and refused where the
    /// bytes are not this build's, since a record half another build's is a card that is half two others.
    #[test]
    fn the_card_record_is_taken_and_put_back_whole_or_not_at_all() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.a_card_has_been_captured(96, 7);
        let saved = unsafe { Th07.captures() };
        assert_eq!(saved.len(), CARDS * CARD_RECORD_BYTES);

        // The ranking read is what would replace it: another value in, the saved bytes back, the value gone.
        image.a_card_has_been_captured(96, 1);
        assert_ne!(unsafe { Th07.captures() }, saved);
        unsafe { Th07.set_captures(&saved) };
        assert_eq!(unsafe { Th07.captures() }, saved);

        // And a record of the wrong size left alone rather than half written.
        unsafe { Th07.set_captures(&saved[..saved.len() - 1]) };
        assert_eq!(unsafe { Th07.captures() }, saved);
    }

    /// The check byte over a name as long as the field holds, which is where a bound of orb's own rather than
    /// the terminator the game stops at would answer a different byte from the game's.
    #[test]
    fn the_check_byte_covers_the_whole_of_a_name_the_game_would_sum() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        image.the_spellcard_is(96);
        image.a_card_has_been_attempted(96, 7);
        // As long as the game's own decrypt of a declaration can make it, every byte of it counted.
        image.the_card_is_named(96, &[b'z'; 0x2f]);

        assert_eq!(unsafe { Th07.count_card_attempt() }, Some(8));
        assert_eq!(
            image.card_check(96),
            image.card_check_the_game_would_compute(96),
            "the record was left in a state the game reads as tampered with"
        );
    }

    /// The counts put back and the names left where they stand, which is what a replay's own records need: the
    /// counts are the run's and the name on the plate is a sprite baked out of whatever the record says now.
    ///
    /// **A name as long as the field holds**, because a bound short of the field is where the tail of a name comes
    /// back out of the saved bytes instead of staying — half the card the playback named and half the card the
    /// snapshot held.
    ///
    /// And every record's check byte taken again, a name standing over counts that were put back summing to
    /// neither of them — see [`CARD_CHECK`] for what the game does with a record whose sum does not match.
    #[test]
    fn the_counts_go_back_and_the_names_stay() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        image.a_card_has_been_attempted(96, 3);
        let saved = unsafe { Th07.captures() };

        // The playback counts the card further and names it, which is what a card's own start does.
        image.a_card_has_been_attempted(96, 9);
        let named = vec![b'z'; CARD_ATTEMPTS - CARD_NAME - 1];
        image.the_card_is_named(96, &named);

        unsafe { Th07.set_captures_keeping_names(&saved) };
        assert_eq!(image.card_attempts(96, 0), 3, "the count did not go back");
        assert_eq!(image.the_cards_name(96), named, "the name did not stay");
        assert_eq!(
            image.card_check(96),
            image.card_check_the_game_would_compute(96),
            "the record was left in a state the game reads as tampered with"
        );

        // And bytes that are not this build's records left alone whole, for the reason a whole record is put back
        // whole: a record half another build's is a card that is half two others.
        unsafe { Th07.set_captures_keeping_names(&saved[..saved.len() - 1]) };
        assert_eq!(image.card_attempts(96, 0), 3);
        assert_eq!(image.the_cards_name(96), named);
    }

    /// The frame a stage is on, answered only while one is running — which is what the input read asks on every
    /// frame there is, the menus' included.
    #[test]
    fn the_frame_a_stage_is_on_is_answered_only_while_a_stage_is_running() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.has_been_in_the_stage_for(1234);
        image.the_front_end_is_up();
        assert_eq!(
            unsafe { Th07.stage_frame() },
            None,
            "a frame of the front end was answered as a stage's"
        );

        image.is_playing(0, 0);
        assert_eq!(unsafe { Th07.stage_frame() }, Some(1234));

        // And the frames a stage is rebuilt on, which is a stage running as much as the frames either side of it.
        image.the_stage_is_being_rebuilt();
        assert_eq!(unsafe { Th07.stage_frame() }, Some(1234));
    }

    /// The three words orb answers about a game's input, each of which is a different word of the game's:
    /// which bits a run is made of, and the two the front end reads a press and a back out of.
    #[test]
    fn the_input_masks_are_the_games_own_and_the_run_takes_every_bit() {
        // Every bit, because this game's own record keeps every bit — and the pause button being read from
        // the other word is what makes that safe.
        assert_eq!(Th07.run_input(), 0xffff);
        // The two the front end tests: the shot button and return, the bomb and the menu button.
        assert_eq!(Th07.menu_decide(), 0x1001);
        assert_eq!(Th07.menu_cancel(), 0x000a);
        assert_eq!(
            Th07.menu_decide() & Th07.menu_cancel(),
            0,
            "a button that both decided and went back would do neither"
        );
    }

    /// And whether the game is going to paint that row at all this frame, which is its own draw job being on
    /// the draw list — not whether a run is being played.
    #[test]
    fn the_lives_row_is_painted_while_the_panels_own_job_is_on_the_draw_list() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(
            !unsafe { Th07.draws_lives_row() },
            "a run with no panel drawn was said to be painting one"
        );

        image.paints_its_panel();
        assert!(unsafe { Th07.draws_lives_row() });
    }

    /// The lives, the bombs and the deaths, floats as well, and nothing at all before a run is built — the
    /// three read through the same pointer the power does, so a null one answers all four together.
    #[test]
    fn the_lives_the_bombs_and_the_deaths_are_the_whole_parts_of_three_more_floats() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        let none = unsafe { Th07.read_state() };
        assert_eq!((none.lives, none.bombs, none.deaths), (0, 0, 0));

        image.has_a_runs_numbers();
        image.has_lives_and_bombs(3.5, 2.0);
        image.has_died(4.0);
        let state = unsafe { Th07.read_state() };
        assert_eq!((state.lives, state.bombs, state.deaths), (3, 2, 4));
    }

    /// A stage per entry, and every boundary in one a hand's and ahead of the one before it.
    ///
    /// **What this is against is the paste**, the table being written beside the game by a judging pass and copied
    /// into `chapters.rs` by somebody: a number dropped in the wrong row, out of order or twice over is a chapter
    /// beginning where nobody put one, and none of it shows in a build. Every one of them is a hand's because this
    /// game's waves never empty — see `chapters.rs` — and the shortest a chapter may be reads that flag, so a
    /// number pasted as the detector's would divide a stage differently than the pass that chose it.
    #[test]
    fn the_midstage_table_is_a_stage_per_entry_of_hands_in_the_order_they_come() {
        let table = Th07.midstage_table();
        assert_eq!(table.len(), STAGES as usize);
        for (stage, boundaries) in table.iter().enumerate() {
            let frames: Vec<i32> = boundaries.iter().map(|boundary| boundary.frame).collect();
            assert!(
                boundaries.iter().all(|boundary| boundary.by_hand),
                "stage {} has a boundary the detector cannot have proposed here: {frames:?}",
                stage + 1,
            );
            assert!(
                frames.windows(2).all(|pair| pair[0] < pair[1]),
                "stage {}'s boundaries are not in the order the stage reaches them: {frames:?}",
                stage + 1,
            );
            assert!(
                frames.iter().all(|frame| *frame > 0),
                "stage {} has a boundary before its own start: {frames:?}",
                stage + 1,
            );
        }
    }

    /// The music a chapter is taken with, read off the two objects a play leaves: the buffer DirectSound made, how
    /// much of it there is and how much goes in at a time, and the file — which in this game is the archive the
    /// game opened itself, so what carries the position is a handle rather than a wav of the track's own.
    ///
    /// The two positions are addresses in the game's own memory, and the test reads the game's own fields back
    /// through them: the countdown the track's loop is taken on, and the offset the next chunk goes at.
    ///
    /// Nothing at all where no track is playing or where the buffer is no longer a live object — a released one
    /// leaves a pointer into a freed block, and bytes put back into that are audible as the music breaking up.
    #[test]
    fn the_music_is_rewound_through_the_file_the_game_opened_itself() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.plays_no_track();
        assert!(Th07.music().is_none(), "no track is playing");

        let sound = orb_sim::Sound::of(
            (0..STAGE_TRACK.bytes).map(|byte| byte as u8).collect(),
            0x80,
            0x20,
        );
        image.holds_its_music_file(&[STAGE_TRACK]);
        image.streams_a_track(&sound, STAGE_TRACK, STAGE_TRACK.bytes - 0x100);

        let music = Th07.music().expect("the track this test laid out");
        assert_eq!(music.buffer, SoundBuffer(sound.buffer_object()));
        assert_eq!(music.buffer_size, sound.buffer_size());
        assert_eq!(music.notify_size, sound.notify_size());
        let Track::OpenFile(file) = music.track else {
            panic!("妖々夢 reads every track out of the one file it opened itself")
        };
        assert_eq!(file, sound.mmio());

        image.set_bytes_left(0x123);
        image.set_next_write_offset(0x40);
        assert_eq!(image.space().read::<u32>(music.bytes_left), 0x123);
        assert_eq!(image.space().read::<u32>(music.write_offset), 0x40);

        image.frees_the_stream_buffer();
        assert!(
            Th07.music().is_none(),
            "a buffer that is not a live object was read as one"
        );
    }

    /// The music taken down and started again through the game's own two calls, with the queue those calls write
    /// into walked — which is what makes either of them the music having changed rather than a command waiting.
    ///
    /// **The take-down walks that queue and the start leaves it**, which is the difference a launch faulted over:
    /// the play would be carried out over the stream the memory copy is about to write back, so what runs it is the
    /// frame's own sound afterwards.
    ///
    /// **And which slot the start asks for is read off the track that was playing**: the fight's own song where
    /// that is what the stage's data names, and the stage's own for anything else — including a track this stage
    /// has since loaded over, whose slot holds another name entirely. A 妖々夢 session that asked for a fixed slot
    /// heard the title screen's song where the chapter's own should have come back.
    #[test]
    fn the_music_is_taken_down_and_started_again_by_the_games_own_calls() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();
        let music = image.hands_over_the_music();

        image.plays_no_track();
        assert!(
            !unsafe { Th07.restart_stage_music() },
            "a chapter taken with no track has no music to start again"
        );
        assert_eq!(music(), (0, 0));

        let sound = orb_sim::Sound::of(
            (0..STAGE_TRACK.bytes).map(|byte| byte as u8).collect(),
            0x80,
            0x20,
        );
        image.holds_its_music_file(&[TITLE_TRACK, STAGE_TRACK, FIGHT_TRACK]);
        image.names_its_songs(c"bgm/th07_02.mid", c"bgm/th07_03.mid");
        image.loads_a_slot(STAGE_SONG_SLOT, c"bgm/th07_02.wav");
        image.loads_a_slot(FIGHT_SONG_SLOT, c"bgm/th07_03.wav");
        image.streams_a_track(&sound, STAGE_TRACK, STAGE_TRACK.bytes);

        unsafe { Th07.stop_music() };
        assert_eq!(music(), (1, 0));
        assert!(
            Th07.music().is_none(),
            "the stop left the player naming a stream it had deleted"
        );
        let ran = image.times_the_sound_commands_ran();
        assert!(ran > 0, "the take-down was left in the queue");

        // The memory copy back, which is what a restore does before it asks for the track again — and where orb
        // reads which track the chapter was under, that read going through this very pointer.
        image.names_the_stream_it_had();
        assert!(unsafe { Th07.restart_stage_music() });
        assert_eq!(music(), (1, 1));
        assert_eq!(
            image.times_the_sound_commands_ran(),
            ran,
            "the play was carried out over the stream the memory copy has just written back"
        );

        image.carries_the_play_out();
        assert_eq!(image.music_started(), ["th07_02.wav"]);

        // The fight's own song playing, which is the second of the two names the stage's data carries.
        image.streams_it_again(&sound, FIGHT_TRACK, FIGHT_TRACK.bytes);
        assert!(unsafe { Th07.restart_stage_music() });
        image.carries_the_play_out();
        assert_eq!(image.music_started(), ["th07_02.wav", "th07_03.wav"]);

        // And the song of the scene before this stage, whose slot the stage's build has loaded over: the stage's
        // own song is the answer there, and the seek that follows is refused by its own identity check rather than
        // putting a position from another track into this one.
        image.streams_it_again(&sound, TITLE_TRACK, TITLE_TRACK.bytes);
        assert!(unsafe { Th07.restart_stage_music() });
        image.carries_the_play_out();
        assert_eq!(
            image.music_started(),
            ["th07_02.wav", "th07_03.wav", "th07_02.wav"]
        );
    }

    /// The offer to save a replay taken off the screen, and every other frame left alone — including the score's
    /// own name entry on the same screen, which is the ranking a run belongs in.
    #[test]
    fn the_offer_to_save_a_replay_is_the_only_phase_taken_off_that_screen() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        assert!(
            !unsafe { Th07.skip_replay_prompt() },
            "a frame of a run was taken for the screen a run ends on"
        );

        image.is_in_scene(SCENE_RUN_OVER, SCENE_RUN_OVER);
        image.the_run_ended_screen_is_up(RESULT_FROM_A_RUN);
        assert!(!unsafe { Th07.skip_replay_prompt() });

        image.the_screen_was_brought_up_by(*RESULT_OFFERING_A_REPLAY.start());
        assert!(unsafe { Th07.skip_replay_prompt() });
        assert_eq!(image.the_screens_state(), RESULT_LEAVING);
        assert_eq!(
            image.the_screens_timer(),
            0,
            "the screen would leave a second early or late"
        );

        // Every phase of the offer, and none of the phases either side of it.
        for phase in RESULT_OFFERING_A_REPLAY {
            image.the_screen_was_brought_up_by(phase);
            assert!(unsafe { Th07.skip_replay_prompt() }, "phase {phase:#x}");
        }
        for phase in [
            RESULT_BUILDING,
            RESULT_LEAVING,
            RESULT_FROM_A_RUN,
            RESULT_FROM_A_PRACTICE_RUN,
        ] {
            image.the_screen_was_brought_up_by(phase);
            assert!(!unsafe { Th07.skip_replay_prompt() }, "phase {phase:#x}");
        }
    }

    /// The tile the panel's own background is laid with, taken out of the vm the game lays it with rather than
    /// written down: the texture the slot in that vm's sprite names, and the piece of the sheet that sprite is.
    #[test]
    fn the_panels_tile_is_the_one_the_game_lays_its_own_background_with() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        // Outside a stage there is nothing to take it out of.
        assert!(
            unsafe { Th07.panel_tile() }.is_none(),
            "a tile was answered where the game has built nothing to lay one from"
        );

        image.has_a_panel();
        let (texture, uv) = image.the_panels_tile();
        let tile = unsafe { Th07.panel_tile() }.expect("the panel is up");
        assert_eq!(tile.texture.0, texture);
        assert_eq!(tile.uv, uv);
        assert_eq!(tile.origin, PANEL_ORIGIN);
        assert_eq!(tile.pitch, PANEL_PITCH);

        // And a vm whose script has not put a sprite on it yet, which is every frame before the build.
        image.the_panels_tile_has_no_sprite();
        assert!(unsafe { Th07.panel_tile() }.is_none());

        // Nor without the manager the texture is one of.
        image.has_a_panel();
        image.has_no_sprite_manager();
        assert!(unsafe { Th07.panel_tile() }.is_none());
    }

    /// The player put where nothing can hit them for an update, out of the two states that can be hit in — and
    /// left alone in the three that cannot, since each of those is the game in the middle of something.
    #[test]
    fn the_player_is_put_where_nothing_can_hit_them_without_taking_the_border() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.is_playing(0, 0);
        unsafe { Th07.make_invulnerable() };
        assert_eq!(image.the_players_state(), PLAYER_COMING_BACK);
        assert_eq!(
            image.the_frames_the_state_has_left(),
            PLAYER_INVULNERABLE_FRAMES,
            "the state would run out inside the update it was written for"
        );

        // Already coming back: the frames topped up rather than left to run out.
        image.the_player_is_coming_back(1);
        unsafe { Th07.make_invulnerable() };
        assert_eq!(
            image.the_frames_the_state_has_left(),
            PLAYER_INVULNERABLE_FRAMES
        );

        // Dying is left alone — a death the game is in the middle of is not orb's to take back.
        image.the_player_is_dying();
        unsafe { Th07.make_invulnerable() };
        assert_eq!(image.the_players_state(), 1);

        // And so is the border, whose hit is deflected at the cost of the border itself.
        image.the_players_border_is_up();
        unsafe { Th07.make_invulnerable() };
        assert_eq!(image.the_players_state(), 4);
    }

    /// The power, which this game keeps as a float and `State` as a whole number.
    ///
    /// And nothing at all where no run has been built, which is what the null pointer at the game
    /// manager's `+0x8` means: a field read through it then would be a read of address 0x7c.
    #[test]
    fn the_power_is_the_whole_part_of_the_float_the_game_keeps() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        assert_eq!(unsafe { Th07.read_state() }.power, 0);
        image.has_a_runs_numbers();
        image.shot_is_at_power(62.5);
        assert_eq!(unsafe { Th07.read_state() }.power, 62);
        image.shot_is_at_power(128.0);
        assert_eq!(unsafe { Th07.read_state() }.power, 128);
    }

    /// The two records the mode's own file keeps read again out of it, over the half of the front end's own read
    /// that is a record of runs: the practice scores into the game's own table, and the card records with them.
    ///
    /// **One read for both**, which is what the game's own does with them, and the buffer given back afterwards —
    /// a read whose buffer is never released is one every visit to the title leaks.
    ///
    /// The stage-select screen showing a legacy run's scores is what this is about: the front end reads the game's
    /// own file at its own build, and 完全無欠モード's scores are in a file of its own.
    ///
    /// **Which of the two files that is, is not this test's**: the open orb makes carries the game's own name and
    /// the fork over `CreateFileA` decides from it, so what is laid out here is whichever file an open lands in
    /// now — see [`crate::score::the_file_now_opened`].
    #[test]
    fn the_practice_scores_are_read_out_of_the_modes_own_file() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();
        let read = image.hands_over_the_score_file();

        // A card already counted in the file, so that what the parse laid down is readable rather than a table of
        // zeroes agreeing with an empty one.
        image.empties_its_card_records();
        image.its_score_file_holds(crate::score::the_file_now_opened(), 96, 0, 15, 7);

        unsafe { Th07.read_what_the_modes_file_records() };

        let (times_read, parsed_into, released) = read();
        assert_eq!(times_read, 1, "the file was read once for both records");
        assert_eq!(
            parsed_into, G_PRACTICE_SCORES,
            "the scores were parsed into another table than the game's own"
        );
        assert_eq!(released, 1, "the buffer the read made was not given back");
        assert_eq!(
            (image.card_attempts(96, 0), image.card_captures(96, 0)),
            (15, 7),
            "the card records were not read out of the file with the scores"
        );
    }

    /// What the title menu's cursor is on, which is the one screen orb has a question of its own to put
    /// over — and nothing anywhere else, since a question drawn over the wrong screen is worse than none.
    #[test]
    fn the_question_orb_asks_is_only_asked_of_the_title_menus_own_items() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        // A run in progress is not the front end, and the front end has not been built.
        image.is_playing(0, 0);
        assert_eq!(unsafe { Th07.menu_pointed_at() }, None);

        image.the_front_end_is_up();
        for (item, asked) in [
            (TITLE_ITEM_START, Some(Menu::Run)),
            (TITLE_ITEM_EXTRA, Some(Menu::Run)),
            (TITLE_ITEM_PRACTICE, Some(Menu::Run)),
            (TITLE_ITEM_SCORE, Some(Menu::Scores)),
            (3, None),
            (5, None),
            (7, None),
        ] {
            image.the_title_menu_points_at(item);
            assert_eq!(
                unsafe { Th07.menu_pointed_at() },
                asked,
                "the cursor on item {item}"
            );
        }

        // A press is taken there, which is what makes the question worth putting on the press — and not for
        // the ten frames after the screen came up, which the game itself ignores.
        assert!(unsafe { Th07.menu_takes_a_press() });
        image.the_screen_has_been_up_for(SETTLED_FRAMES - 1);
        assert!(!unsafe { Th07.menu_takes_a_press() });
        image.the_screen_has_been_up_for(SETTLED_FRAMES);

        // And nothing on the front end's other screens: the replay list and the practice select are its
        // screens too, and the cursor there is a cursor over something else.
        image.the_title_menu_points_at(TITLE_ITEM_START);
        image.the_front_end_is_on_another_screen();
        assert_eq!(unsafe { Th07.menu_pointed_at() }, None);
        assert!(!unsafe { Th07.menu_takes_a_press() });
    }

    /// The ranking asked for the way the game's own Score item asks for it, read as built once its screen is past
    /// its own build, told to leave the way that screen's menu tells it, and the cursor put back into the front end
    /// that comes up afterwards.
    ///
    /// **The cursor is read while the front end is still up and written into the one that is up later**, which are
    /// two different objects' worth of the same field: asking for the ranking takes the front end down, and coming
    /// back from it builds another.
    #[test]
    fn the_ranking_is_asked_for_built_left_and_the_cursor_put_back() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.the_front_end_is_up();
        image.the_title_menu_points_at(4);
        unsafe { Th07.show_ranking() };
        assert_eq!(unsafe { Th07.read_state() }.wanted, SCENE_RANKING);

        // The screen the game builds for it, still inside its own build: not up yet, which is what orb waits for
        // before it asks the screen to leave.
        image.is_in_scene(SCENE_RANKING, SCENE_RANKING);
        image.the_run_ended_screen_is_up(RESULT_BUILDING);
        assert!(!unsafe { Th07.showing_ranking() });

        image.the_screen_was_brought_up_by(image.phase_of_a_screen_from_a_run());
        assert!(unsafe { Th07.showing_ranking() });

        // Told to leave, which is the state its own menu puts it in with the timer that state waits on emptied.
        unsafe { Th07.leave_ranking() };
        assert!(image.the_screen_a_run_ends_at_is_leaving());
        assert_eq!(image.the_screens_timer(), 0);

        // And the cursor back where it was, into the front end built after the ranking rather than the one that
        // asked for it.
        image.the_front_end_is_up_again();
        image.the_title_menu_points_at(0);
        unsafe { Th07.restore_menu_cursor() };
        assert_eq!(image.the_item_the_title_menu_points_at(), 4);

        // And nothing remembered is nothing put back, rather than item 0 written over whatever the menu is on.
        image.the_title_menu_points_at(2);
        unsafe { Th07.restore_menu_cursor() };
        assert_eq!(image.the_item_the_title_menu_points_at(), 2);
    }

    /// And which scene that is: the one the front end's own Score item asks for, and not the one a run ends into —
    /// 0x44a302 builds one screen for both, so the screen alone cannot tell them apart.
    ///
    /// The coarser question of the two, and the reason there are two: a screen answers a request to leave by
    /// changing its own state and putting the scene back a second later, and those frames are drawn ones.
    #[test]
    fn the_ranking_is_the_screen_the_front_end_asked_for_and_not_a_run_ending() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.the_front_end_is_up();
        assert!(!unsafe { Th07.ranking_scene() });

        image.is_in_scene(SCENE_RANKING, SCENE_RANKING);
        assert!(unsafe { Th07.ranking_scene() });

        // A run ending into the same screen on the scene of its own: the screen is up and past its build, and it is
        // not the ranking orb asked for.
        image.is_in_scene(SCENE_RUN_OVER, SCENE_RUN_OVER);
        image.the_screen_a_run_ends_at_is_up();
        assert!(!unsafe { Th07.ranking_scene() });
        assert!(!unsafe { Th07.showing_ranking() });
    }

    /// Which run the shot select is pointing at, which is where the mark saying one was left unfinished goes —
    /// and nothing on a practice run's, whose stage is chosen on the screen after.
    #[test]
    fn the_run_pointed_at_is_the_shot_select_of_a_whole_run_or_the_extra() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.the_front_end_is_up();
        assert_eq!(unsafe { Th07.run_pointed_at() }, None, "the title menu");

        // **The shot the cursor is on, and not the one the game last decided**, which is the byte at
        // [`G_SHOT_CHOSEN`]: it is written where an item is decided and where the screen is left, so on the
        // screen itself it holds the shot of the run before. A 妖々夢 session was offered 霊夢B's unfinished run
        // with the cursor on 霊夢A, which is that byte read as the cursor.
        image.the_shot_it_last_decided_is(1);
        image.the_shot_select_points_at(SCREEN_SHOT_SELECT, 3, 2, 0);
        assert_eq!(
            unsafe { Th07.run_pointed_at() },
            Some(RunStart {
                difficulty: 3,
                character: 2,
                shot_type: 0,
                practice: false,
                stage: 0,
            })
        );

        image.the_shot_select_points_at(SCREEN_SHOT_SELECT, 3, 2, 1);
        assert_eq!(
            unsafe { Th07.run_pointed_at() }.map(|run| run.shot_type),
            Some(1)
        );

        // The Extra's own shot select is a run as much as the first is.
        image.the_shot_select_points_at(SCREEN_SHOT_SELECT_EXTRA, 4, 0, 0);
        assert_eq!(
            unsafe { Th07.run_pointed_at() }.map(|run| run.difficulty),
            Some(4)
        );

        // And a practice run's is not: its stage is part of which run it is, and it has not been chosen.
        image.the_shot_select_points_at(10, 3, 2, 1);
        assert_eq!(unsafe { Th07.run_pointed_at() }, None);
    }

    /// And the stage counted the way orb counts stages, which is not the way this game holds it: the word its
    /// build leaves for the first stage is one, and every stage of the eight is one along.
    ///
    /// Written against the game's own word rather than through
    /// [`is_playing`](Image::is_playing) on purpose. Both sides of that helper are this module's, so a stage
    /// put in and read back would agree with itself whichever way the one went — this is the test that fails
    /// if orb starts answering the number the game holds.
    #[test]
    fn the_stage_orb_answers_is_the_word_the_build_left_less_one() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        for (word, stage) in [(1, 0), (6, 5), (7, 6), (8, 7)] {
            image.the_games_stage_word_is(word);
            assert_eq!(
                unsafe { Th07.read_state() }.stage,
                stage,
                "the game's own stage word {word} is orb's stage {stage}"
            );
        }
    }

    /// The three objects the sound system is, which a restore has to hold still while it copies: the player, the
    /// stream it holds and the source that stream reads the file through.
    ///
    /// The player is always there, being a static; the other two are objects a playing track has and nothing else
    /// does, so a launch with no music answers the one range — and a stream naming no source answers two.
    #[test]
    fn the_sound_systems_own_memory_is_what_a_restore_leaves_alone() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.plays_no_track();
        let quiet = Th07.audio_state();
        assert_eq!(quiet.len(), 1);
        assert_eq!(quiet[0].len(), SOUND_PLAYER_BYTES);

        let sound = orb_sim::Sound::of(
            (0..STAGE_TRACK.bytes).map(|byte| byte as u8).collect(),
            0x80,
            0x20,
        );
        image.holds_its_music_file(&[STAGE_TRACK]);
        image.streams_a_track(&sound, STAGE_TRACK, STAGE_TRACK.bytes);
        assert_eq!(Th07.audio_state(), image.the_sound_systems_own_memory());
    }

    /// Which track is playing, out of the record the source is reading rather than out of the slot it was asked
    /// for: the two are not the same, a slot holding a path and a record the file's own name.
    ///
    /// **Different tracks answer different identities**, which is all a chapter asks of the number — whether the
    /// track has changed since the snapshot, not which track it is. And whether the fight's own song is the one
    /// playing is the record's name against the second of the two the stage's data carries, held by the stem: the
    /// table's names are the `.mid` ones and a record's is the file's own.
    ///
    /// Nothing at all where no track is playing, a restore having no reason to give a chapter music it had none of.
    #[test]
    fn the_track_playing_is_the_record_the_source_is_reading() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        image.plays_no_track();
        assert_eq!(Th07.music_identity(), None);
        assert_eq!(Th07.the_fights_own_song_is_playing(), None);

        let sound = orb_sim::Sound::of(
            (0..STAGE_TRACK.bytes).map(|byte| byte as u8).collect(),
            0x80,
            0x20,
        );
        image.holds_its_music_file(&[TITLE_TRACK, STAGE_TRACK, FIGHT_TRACK]);
        image.streams_a_track(&sound, STAGE_TRACK, STAGE_TRACK.bytes);
        let stage = Th07.music_identity().expect("the track this test laid out");

        // The stage's data naming its two songs, which is what the record playing is held against — and no table at
        // all is no answer, rather than the stage's song by default.
        assert_eq!(Th07.the_fights_own_song_is_playing(), None);
        image.names_its_songs(c"bgm/th07_02.mid", c"bgm/th07_03.mid");
        assert_eq!(Th07.the_fights_own_song_is_playing(), Some(false));

        // The fight's, which is another record on the same stream: the identity moves with it.
        image.streams_it_again(&sound, FIGHT_TRACK, FIGHT_TRACK.bytes);
        assert_eq!(Th07.the_fights_own_song_is_playing(), Some(true));
        assert_ne!(
            Th07.music_identity(),
            Some(stage),
            "two tracks came out as the same track, so a chapter cannot tell them apart"
        );
    }

    /// The one range of the game's memory a restore must leave alone, and nothing at all before the sprite
    /// manager exists — the pointer being null then, which is a run's very first frames.
    #[test]
    fn the_textures_a_restore_must_not_put_back_are_the_sprite_managers_own() {
        let image = Image::laid_out_seeded(0);
        let _installed = image.enter();

        let textures = image.sprite_textures();
        assert_eq!(unsafe { Th07.live_handles() }, vec![textures.clone()]);
        assert_eq!(textures.len(), ANM_TEXTURE_SLOTS * size_of::<usize>());

        image.has_no_sprite_manager();
        assert!(
            unsafe { Th07.live_handles() }.is_empty(),
            "a null manager answered a range, which a restore would write over"
        );
    }
}
