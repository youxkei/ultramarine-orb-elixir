//! 東方妖々夢, as a [`Game`] that answers what has been measured and declines the rest.
//!
//! `th07.exe`, 650752 bytes, `md5 0126afce1e805370d36c3482445e98da`. Every address below was read off
//! that exe with `i686-w64-mingw32-objdump -d -M intel` and nothing else: no table anybody published
//! and nothing carried across from 紅魔郷's addresses, which are where the same things happen to live
//! in another game. Each constant says what identifies it, because that is the whole of the evidence
//! there is — there is no decompilation of this build to cross-check against, as [`th06`](super::th06)
//! has.
//!
//! **What is measured is a frame, and nothing about a run.** The window, the device, the chain and its
//! two walks, the whole frame and everything it does around its own drawing, the play area and the size
//! it renders at. So orb *paces* 妖々夢 and does not *play* it: every method here that answers `None` or
//! nothing is a feature 完全無欠 does not have here — no chapters, no retry menu, no run picked up again,
//! no card counted, no mode question, and the game's own score file left as its own, which is
//! [`Th07::rewinds`]. The trait already says what each of those costs. See `docs/adr/0004` for the shape
//! and `docs/adr/0017` for the frame.
//!
//! **Nothing here panics.** An `unimplemented!()` in a method a frame reaches would take somebody's
//! game down at the moment orb ran out of measurements, which is the one thing an injected DLL must
//! not do. Where the trait offers no way to decline, the answer is the emptiest one that is *true* —
//! no regions, no music, no run in progress, no button worth writing down — and no method answers a
//! guess.

/// Reachable from this crate's own tests and, through the `sim` feature, from the tests of the
/// crates that drive it — which is where the e2e tests live.
#[cfg(any(test, feature = "sim"))]
pub mod image;

use std::ffi::c_void;
use std::ops::Range;
// For the slots behind `handed_over!`, which are the only atomics in here: a build with no laid-out game
// in it has the three addresses as constants, and nothing to import for them.
#[cfg(any(test, feature = "sim"))]
use std::sync::atomic::{AtomicUsize, Ordering};

use orb_api::Hwnd;
use orb_api::mem;

use crate::audio::Music;
use crate::game::{
    Boundary, Call, FrameCalls, Game, Hooks, Menu, Pad, PadRead, PanelTile, Patch, Reading, Rect,
    Reproduction, RunStart, RunState, State,
};
use orb_api::d3d8;
use orb_api::{Device, Viewport};

pub struct Th07;

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

/// The pad half of the input read, `__fastcall`: the word the keyboard was read into arrives in `ecx` and
/// comes back with the pad's bits added.
///
/// A tail call inside the keyboard read at 0x4309c0 — `mov ecx,[ebp-0x104]; call 0x4303f0` at 0x430f1a and
/// again at 0x4312b7 — so it cannot be told apart from that read from outside it, which is what 紅魔郷's
/// is too. **The convention is not 紅魔郷's**: that one takes the word on the stack (`push edx; call
/// 0x41cfc0; add esp,4` at 0x41dc77), and this one in `ecx`, which is why the seam says which — see
/// [`PadRead`].
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

/// The size the game renders at.
///
/// `0x280` and `0x1e0` written into the shared `D3DVIEWPORT8` at [`VIEWPORT`] over the whole output,
/// at 0x4347ad inside [`RENDER`] and again at 0x448d70, and set through +0xa0 each time.
const BACK_BUFFER: (u32, u32) = (640, 480);

/// The `D3DVIEWPORT8` the game builds its viewports in, which is one struct reused: the whole output
/// at 0x4347ad, and whatever else asks. Written here as well as handed to the device, so that a
/// viewport orb sets is one the game's own next read of it agrees with.
const VIEWPORT: usize = 0x00575a18;

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

impl Game for Th07 {
    /// The two a frame is made of, and the frame itself. Everything else is `None`: eleven seams
    /// 紅魔郷 has and 妖々夢 has not been read for, each costing what [`Hooks`] says it costs.
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
            // A replay is neither written nor watched here: nothing has been read of where 妖々夢 keeps
            // its record of inputs, so there is nothing orb could suppress or protect.
            save_replay: None,
            stop_recording: None,
            // No stage of a run is written over, which is what leaves a run unable to be picked up
            // again — and with `run_slot` answering `None` there is nothing to write one into either.
            stage_begun: None,
            stage_building: None,
            // One score file, the game's own, whichever mode a launch says it is in. Both of these
            // exist to keep two files apart, and 妖々夢 has no second one until orb has a mode here.
            unlocks_read: None,
            ranking_read: None,
            // The game makes whatever window its own configuration asks for. `force_windowed` has
            // nowhere measured to write, so there would be nothing for this hook to do.
            create_window: None,
            // The device's `Present` redirected, which is what puts the game's frames through a
            // letterbox: 妖々夢 renders 640x480 and its own present stretches that over whatever client
            // area it has, so a window that is not 4:3 — every borderless one on a 16:9 display — showed
            // the game stretched until this went in. [`INIT_D3D_DEVICE`] is where the slot can be
            // swapped, and where a reset that replaced it is caught.
            init_device: Some(Patch {
                target: INIT_D3D_DEVICE,
                prologue: INIT_D3D_DEVICE_PROLOGUE,
            }),
            // Every key the game reads reaches it. A run whose buttons are written down is a run that
            // can be picked up, and this game has none.
            input: None,
            // The pad half of that read, which is orb's: every pad the machine has, in place of the one
            // device the game looks for and the one winmm joystick it falls back to. Not called through —
            // see [`GET_CONTROLLER_INPUT`].
            joystick: Some(PadRead::InEcx(Patch {
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

    /// Nothing here can be rewound, which is the same answer every method below gives one piece of: no
    /// midstage table, no stage to write a run's numbers into, no run kept and no retry offered.
    ///
    /// **What it saves is a file.** A launch here left a `pointdevice_score.dat` in a 妖々夢 install,
    /// because the fork followed the mode and the mode was pointdevice wherever chapters were on — so the
    /// runs in it were runs anybody could have played, and they belong in the ranking the game keeps.
    /// Nothing was lost; a file appeared that should not have. This is the answer that stops it, and the
    /// day 妖々夢 has chapters it is what turns them on.
    fn rewinds(&self) -> bool {
        false
    }

    /// No run, which is what orb knows of 妖々夢 rather than a claim about the game.
    ///
    /// The trait has no way to decline a `State`, and this is the emptiest one that is true of what has
    /// been measured: nothing is playing, so `Chapters::observe` sees no run, no snapshot is taken, no
    /// boundary is judged and no mark is drawn. A field filled from a th06 offset would be a number out
    /// of another game's memory, which is worse than nothing by exactly the amount somebody would trust
    /// it.
    unsafe fn read_state(&self) -> State {
        State {
            scene: 0,
            wanted: 0,
            playing: false,
            in_run: false,
            in_game: false,
            in_ending: false,
            ending_script: None,
            demo: false,
            replay: false,
            practice: false,
            paused: false,
            unsettled: false,
            bombing: false,
            in_dialogue: false,
            stage: 0,
            difficulty: 0,
            stage_frames: 0,
            script_frames: 0,
            random_seed: 0,
            deaths: 0,
            lives: 0,
            bombs: 0,
            power: 0,
            enemy_count: 0,
            bullet_count: 0,
            laser_count: 0,
            boss_present: false,
            boss_life: None,
            boss_attack_frames: None,
            spellcard: None,
        }
    }

    unsafe fn window(&self) -> Hwnd {
        unsafe { mem::read(G_GAME_WINDOW) }
    }

    unsafe fn d3d_device(&self) -> Device {
        Device(unsafe { mem::read(G_D3D_DEVICE) })
    }

    /// No music, which costs a chapter its track and there being no chapters costs nothing.
    fn music(&self) -> Option<Music> {
        None
    }

    fn music_identity(&self) -> Option<u32> {
        None
    }

    fn audio_thread(&self) -> Option<u32> {
        None
    }

    unsafe fn stop_music(&self) {}

    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    fn audio_state(&self) -> Vec<Range<usize>> {
        Vec::new()
    }

    /// No range of the game's memory holds a handle orb knows about, which is only reached by a restore
    /// and there is nothing here to restore.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        Vec::new()
    }

    fn play_area(&self) -> Rect {
        PLAY_AREA
    }

    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        None
    }

    unsafe fn repaint_lives_row(&self) {}

    /// Nothing of orb goes over the lives, so nothing has to be repainted under it.
    unsafe fn draws_lives_row(&self) -> bool {
        false
    }

    /// Nowhere, which is true of a game whose panel orb has not read: a rectangle of no size is one no
    /// quad covers, and with `draws_lives_row` false nothing is drawn against it anyway.
    fn lives_row(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn content_size(&self) -> (u32, u32) {
        BACK_BUFFER
    }

    /// Not windowed, which is the answer that leaves the game's own configuration alone: with
    /// [`Hooks::create_window`] `None` there is no hook to overrule it from, so this is read nowhere.
    fn windowed(&self) -> bool {
        false
    }

    unsafe fn force_windowed(&self) {}

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
        d3d8::set_viewport(device, viewport);
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

    /// The keyboard needs nothing done to it, which is what `true` says. With [`Hooks::input`] `None`
    /// the game's read is never stood in front of, so it is never lost and never re-acquired.
    unsafe fn acquire_input(&self) -> bool {
        true
    }

    /// Nothing was let go of. `--take-sent-keys` is for driving the game's own menus from another
    /// program, and orb puts no question over 妖々夢's.
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
        d3d8::set_viewport(device, viewport);
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

    /// No stage to jump to: stepping is a replay's, and no replay is watched here.
    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn make_invulnerable(&self) {}

    /// Never a replay, which means orb offers a retry to every run it thinks it sees — and it sees
    /// none, `read_state` answering that nothing is playing.
    unsafe fn replaying(&self) -> bool {
        false
    }

    unsafe fn show_ranking(&self) {}

    unsafe fn showing_ranking(&self) -> bool {
        false
    }

    unsafe fn ranking_scene(&self) -> bool {
        false
    }

    unsafe fn restore_menu_cursor(&self) {}

    /// Said rather than numbered, there being no numbers: the line exists so that a ranking built and
    /// taken down can be read afterwards, and none is built here.
    unsafe fn ranking_state(&self) -> String {
        "nothing of 妖々夢's ranking screen has been measured".to_owned()
    }

    unsafe fn leave_ranking(&self) {}

    /// No card is counted. Where 妖々夢 keeps its record of spell cards has not been read, and a count
    /// written into an address that is not that would be a number in somebody's score file.
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        None
    }

    /// And no plate to put a name back on: what 妖々夢 shows a card's name with has not been read, and
    /// nothing here restores a chapter for it to be wrong in.
    unsafe fn redraw_card_name(&self) {}

    /// No record to hold across anything, which leaves the game's own alone through every snapshot and
    /// every ranking read — there being neither.
    unsafe fn captures(&self) -> Vec<u8> {
        Vec::new()
    }

    unsafe fn set_captures(&self, _saved: &[u8]) {}

    unsafe fn set_captures_keeping_names(&self, _saved: &[u8]) {}

    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// And no practice score read again: where 妖々夢 keeps one and which read of its score file fills it
    /// has not been read, so the mode leaves whatever that read found standing. Which is the answer the
    /// rest of this file gives — 妖々夢 has no fork over its score file at all.
    unsafe fn read_practice_scores(&self) {}

    /// Nothing orb has a question about is ever under the cursor, so no press is ever held back and
    /// every one of 妖々夢's own menus works as it does without orb.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        None
    }

    unsafe fn menu_takes_a_press(&self) -> bool {
        false
    }

    /// A pad that is doing nothing, which drives the menus orb has here: none.
    unsafe fn pad(&self, _pad: Option<Reading>) -> Pad {
        Pad::default()
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

    /// No run to give up, the retry menu that would ask being one orb does not put up here.
    unsafe fn leave_run(&self) -> bool {
        false
    }

    unsafe fn swallow_input(&self) {}

    /// No bit is the decide, so nothing is taken out of the word the game reads and nothing is handed
    /// back for one read. Which is what a game orb asks no question of wants — see
    /// [`Game::menu_decide`] for what the bits are for.
    fn menu_decide(&self) -> u16 {
        0
    }

    fn menu_cancel(&self) -> u16 {
        0
    }

    unsafe fn skip_replay_prompt(&self) -> bool {
        false
    }

    /// No stage is running as far as orb knows, so no frame's buttons are written down against one.
    unsafe fn stage_frame(&self) -> Option<u32> {
        None
    }

    /// No bit of a frame's input is worth writing down, there being nothing to write it into: the word
    /// a run is recorded with is masked with this, so nothing is recorded.
    fn run_input(&self) -> u16 {
        0
    }

    /// Nothing, which is only asked with a run in progress and orb sees none.
    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: 0,
            character: 0,
            shot_type: 0,
            practice: false,
            stage: 0,
        }
    }

    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        None
    }

    /// No slot, which is the one answer that stops a run being kept at all: nothing to name is nothing
    /// to write, nothing to offer and nothing to mark. 妖々夢's characters and shots have not been read,
    /// and a file named from another game's numbering would be a chapter offered to the wrong run.
    fn run_slot(&self, _run: &RunStart) -> Option<String> {
        None
    }

    unsafe fn run_state(&self) -> RunState {
        RunState {
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
        }
    }

    unsafe fn set_run_state(&self, _state: &RunState) {}

    unsafe fn set_run_seed(&self, _seed: u16) {}

    /// No run is ever chosen, so the frame a chapter would be put back on never arrives.
    unsafe fn run_chosen(&self) -> bool {
        false
    }

    unsafe fn start_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn run_finished(&self) -> bool {
        false
    }

    unsafe fn reproduction(&self) -> Reproduction {
        Reproduction {
            replay_frame: 0,
            input: 0,
            player: (0.0, 0.0),
            player_area: (0.0, 0.0),
            randoms: 0,
            seed: 0,
            items: 0,
            score: 0,
            extra_lives: 0,
            rank: 0,
            sub_rank: 0,
        }
    }

    /// No table, so no stage has a midstage boundary and the shape of one — script frame numbers on one
    /// clock — is a question that does not come due until 妖々夢 is a game orb plays. With
    /// `stage_begun` and `stage_building` `None` beside it, nothing reaches a chapter here at all.
    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &[]
    }
}

/// The four directions as the bits 妖々夢's own input word names them by.
fn directions((up, down, left, right): (bool, bool, bool, bool)) -> u16 {
    [
        (up, PAD_UP),
        (down, PAD_DOWN),
        (left, PAD_LEFT),
        (right, PAD_RIGHT),
    ]
    .into_iter()
    .filter(|(pushed, _)| *pushed)
    .fold(0, |word, (_, button)| word | button)
}
