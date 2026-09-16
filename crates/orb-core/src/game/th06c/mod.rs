//! th06c — the Steam release of 東方紅魔郷 — as a [`Game`].
//!
//! **A game of its own and not a build of th06.** It is a 64-bit process built two decades later, whose
//! every address, struct and calling convention is its own, and `th06c/asm.md` is where what has
//! been read off its exe is written down. Nothing of `game::th06` means anything here.
//!
//! **Two seams, and a stage read through them.** The dialog is got past by clicking what a person would,
//! and the keyboard read is this game's one moment per tick, so a recipe's steps are walked there and the
//! word they press is put into the snapshot the game is about to read — see
//! [`crate::runtime::keyboard_state`]. Off that one call a stage is read as well: [`sight`] answers
//! [`Game::what_a_bot_sees`] out of this game's own statics, so the rules play it.
//!
//! **And the frame is orb's.** [`Th06c::hooks`] hands over this game's two chain walks and the whole
//! frame around them, so the pacing, the present and the moment the rules are asked are orb's own — see
//! [`THE_FRAME`]. What is still not read is what a run is made of past a stage: [`Th06c::rewinds`]
//! answers false, so there are no chapters, no snapshot to rewind to and no resume, and the methods
//! below that answer nothing say so one by one.
//!
//! **Neither seam is an address**, which is what makes them hold at all: this exe is built with ASLR on
//! and every launch measured came up at a different base, and the pointers it calls through are ones it
//! looked up itself. So orb answers the name the game asks for and clicks the controls of the dialog it
//! puts up, and neither needs an address of the game's.
//!
//! **The addresses below are the reading of a stage, and every one of them goes through [`at`]**, which
//! adds the distance [`Game::set_image_base`] was told.
//!
//! **And every call into this game is `extern "win64"`**, which is the same ABI as the `fastcall` the
//! frame loop's own statics are declared with and the name this target has for it: `fastcall` is an x86
//! convention, a 64-bit build falls back to the C one for it, and it warns each time. The loop's own
//! signatures stay `fastcall` because that code is compiled for both architectures, where `win64` is not
//! a name at all — and on this one the two are the same four registers.
#![doc = include_str!("asm.md")]

use std::ffi::c_void;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use orb_api::mem;

use crate::game::{
    Boundary, Call, Dialog, FrameCalls, Game, Hooks, Lookup, Menu, Pad, PanelTile, Patch, Read,
    Reading, Rect, RunStart, State, Walks,
};

mod sight;

/// Where the exe asks to be loaded, which is what the addresses below are written against.
const IMAGE_BASE: usize = 0x1_4000_0000;

/// How far it turned out to be from that ask — see [`Game::set_image_base`], which writes it, and
/// [`at`], which is the only place it is read. Nought until told, which is a game laid out at the
/// addresses it is written down with.
static SLIDE: AtomicUsize = AtomicUsize::new(0);

/// An address of this game's own where its exe actually is.
///
/// **Wrapping, because the slide is a difference and can be either way about**: an exe below its own
/// ask is a slide that borrows, and the sum is the address all the same.
fn at(address: usize) -> usize {
    address.wrapping_add(SLIDE.load(Ordering::Relaxed))
}

/// **Where the game keeps a virtual key in the 256 it plays a run off**, out of its own table — see
/// [`THE_TABLE_OF_KEYS`], and `Th06c::presses_into_its_own_keys`, which is the one thing that asks.
///
/// Nothing for a key the table has no entry for, and nothing for an index past the buffer: both are a
/// table read wrongly rather than a key nobody pressed, and writing outside those 256 bytes is writing
/// over whatever the game keeps after them.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told — see [`at`].
unsafe fn the_index_of_the_key(key: u8) -> Option<usize> {
    let table = at(THE_TABLE_OF_KEYS);
    for entry in 0..KEYS_IN_THE_TABLE {
        let entry = table + entry * A_KEY_IN_THE_TABLE;
        let wanted = mem::read_committed::<u16>(entry)?;
        // **The end of the table ends the walk**, and does not merely skip an entry: a walk that carried
        // on read whatever follows the table as entries, matched a key against one of them and wrote a
        // `0x80` at the index that garbage held. What that looked like was a launch whose title cursor
        // walked itself from item 0 to item 5 and opened whatever was there.
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

/// **The DirectInput keyboard the game reads a run with**, which orb takes away.
///
/// **Because a run's own input does not come through the keyboard at all while this is here.** The read
/// at `0x14008f930` takes the device where there is one — `GetDeviceState` straight into the game's own
/// key buffer at `0x14008fa08` — and the translation from virtual keys that orb's answer feeds is on the
/// *other* branch, reached only where `[0x1403f8024]` or `[0x1403f8064]` is 1 or this is nought. So a
/// menu, which is up while one of those flags is, took orb's presses, and a run did not: measured with
/// the shot and the right held for six hundred frames, the player still in the middle of the field and
/// the score still `000000000` with the fairies of stage 1 in formation in front of it.
///
/// Nought here is what `Config::sent_keys` is for th06 and th07 — the game reading its keyboard the way
/// it does when DirectInput has no device.
const DEVICE: usize = 0x1_403f_8050;

/// **The read the game asks for by name**, which is the one thing orb stands in front of here: it is
/// resolved with `GetProcAddress` at startup into a slot of the game's own, so orb answers the ask and
/// the game keeps orb's pointer itself — see [`Read::LookedUp`], which has why the slot is the wrong
/// place to write.
const KEYBOARD_READ: &str = "GetKeyboardState";

/// **And the name it asks for to make its window**, which orb answers for the same reason: there is no
/// `CreateWindowEx` of either width in this exe's import table, only the string in its `.rdata` — see
/// `th06c/asm.md`. Its answer lands in the slot at `0x1403f8608` and is called at `0x140095c59`.
const WINDOW_MADE: &str = "CreateWindowExW";

/// **And the name it reads a pad with**, which orb answers by being the pad — see
/// [`Game::press_into_pad`], and `Hooks::reads_a_pad` for why a pad rather than the keys.
///
/// This exe walks `xinput1_4.dll`, `xinput1_3.dll` and `xinput9_1_0.dll` for whichever is there and
/// keeps the handle at `0x1403f8008`, then asks that module for this by name — see `th06c/asm.md`.
const PAD_READ: &str = "XInputGetState";

/// **And the name it makes its renderer through**, which orb answers to take the present — see
/// [`crate::dxgi`], and `Hooks::makes_its_renderer`.
///
/// `0x140250839` asks for it by this string at `0x140294970`, of the module kept at `0x14084d210`, and
/// `0x1402508a8` puts what the call answered at `0x14084d248`. The two older names beside it —
/// `CreateDXGIFactory1` at `0x1402508db` and `CreateDXGIFactory` at `0x14025097b` — are the fallback
/// this exe walks where the newest is not there, and `dxgi` has why orb answers only the newest.
const RENDERER_MADE: &str = "CreateDXGIFactory2";

/// **Which XInput button each of orb's own is**, which is what orb's answer to that read is filled from.
///
/// **The conventional mapping and not a reading of this build's own**: the directions are the d-pad's
/// four bits, the shot is `A` and the bomb is `B`, which is how a pad has been held to play this game
/// since it was sold with one. What this game's own pad screen maps them to is `th06.cfg`'s business and
/// has not been read, so a machine whose pad was configured otherwise is one where a bot presses the
/// wrong buttons — which is a thing the first launch that plays itself says out loud.
mod pad {
    pub const UP: u16 = 0x0001;
    pub const DOWN: u16 = 0x0002;
    pub const LEFT: u16 = 0x0004;
    pub const RIGHT: u16 = 0x0008;
    pub const START: u16 = 0x0010;
    pub const A: u16 = 0x1000;
    pub const B: u16 = 0x2000;
    pub const X: u16 = 0x4000;
}

/// **And the name it moves that window with afterwards**, which is what put a window orb had made in
/// the corner back in the middle of a screen: this exe looks up `MonitorFromWindow`, `GetMonitorInfoW`,
/// `GetSystemMetrics`, `ClientToScreen` and this together, which between them are a window centred on
/// the monitor it is on.
///
/// **And not `SetWindowPos`, which it also calls.** Both of its calls of that one pass `0x27` —
/// `SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_FRAMECHANGED`, at `0x14009e498` and `0x14009f34b` — so
/// neither moves anything.
const WINDOW_MOVED: &str = "MoveWindow";

/// **The startup dialog's own controls**, read off the dialog while it was up: `1` is 「ゲーム起動」,
/// `200` the ボーダーレスウィンドウ it comes up on, `201`–`205` the フルスクリーン sizes, `206`–`210`
/// the ウィンドウ ones — largest first, so `210` is the 640x480 at the bottom of the list — and `400`
/// the VSync it comes up with ticked.
///
/// The dialog is `DialogBoxIndirectParamW`'s, built in memory rather than from a resource, and it comes
/// up **every launch**: measured, a launch with the `th06.cfg` and `th06.env` of an earlier one beside
/// the exe put the same dialog up again.
mod dialog {
    /// The smallest window it offers, which is what a launch nobody is watching asks for.
    pub const A_640_BY_480_WINDOW: i32 = 210;
    pub const START: i32 = 1;
}

/// **Which virtual key each of this game's buttons is pressed with**, which is what orb puts into the
/// snapshot for a word — see [`Th06c::press_into`].
///
/// **The keys 東方紅魔郷 has always been played with** and not a reading of this build's own
/// configuration: `th06.cfg` beside the exe is where that would be, and nothing has been read out of it.
/// A machine whose keys were changed there is one where a bot presses the wrong ones, which is a thing
/// the first launch that plays itself says out loud.
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
}

/// **The bits orb's own word carries for this game**, which are orb's and not the game's: what this
/// game's own read answers is a snapshot of keys, so there is no word of its own to match — see
/// [`Th06c::press_into`], which is the only place these are turned back into anything.
///
/// The same shape as th06 1.02h's own word all the same, so that a recipe written for one game reads the
/// same for the other: a `press: [decide]` is a decide in both.
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

/// **The word the game's own read leaves behind, which a run is played with** — `0x14090a114` is the
/// frame before it and the pair is what an edge is taken from, at `0x14004a5c3`.
///
/// The bits are [`button`]'s, which is what makes that module's numbers the game's here as well as
/// orb's: `0x14004ac69` onwards tests `0x10`, `0x20`, `0x40` and the sign bit for the four directions
/// and `0x14004ac9d` tests `0x04` for the focus.
const INPUT_THE_RUN_ACTS_ON: usize = 0x1_4090_a110;

/// **The byte that says everything is held**, which `0x14000d8cd` and six other managers answer 1 and
/// return on. A frame with it set is a frame nothing of the game moved in.
const EVERYTHING_HELD: usize = 0x1_403a_3b62;

/// **The walk of the calc chain, which is this game's frame** — every job of a tick, in priority
/// order, and the frame's own queued sound effects at the end of it.
///
/// **The one function of the frame there is to stand in.** `0x1400256f0` calls this once and then up to
/// seven times more, as many as the sound's own position says the run is behind by, and walks the *draw*
/// chain inlined in its own code — so the draw side has no function of its own and this side has one.
/// `th06c/asm.md` has the frame, the return codes the walk acts on, and how both were read.
/// **Called rather than patched**, nothing of the game's calling it once the frame around it is orb's:
/// see [`walks_the_calc_chain`], which is what orb's own loop calls and where the answer this gives is
/// turned into one that loop reads.
const RUN_CALC_CHAIN: usize = 0x1_4002_5500;

/// **The whole frame, which orb replaces to own this game's own.** The loop at `0x14002c4b7` calls it
/// once round and nothing else calls it at all, and what it holds is the viewport, the calc chain, the
/// sound's own catch-up, the draw chain walked inline and the frame's own timing — every one of which
/// orb's own loop now calls, in that order. `th06c/asm.md` has the frame.
const THE_FRAME: usize = 0x1_4002_56f0;
/// `mov [rsp+8],rbx; mov [rsp+0x10],rsi; push rdi; sub rsp,0xd0` — eighteen bytes to the end of the
/// fourth instruction, none of them rip-relative, where a 64-bit jump needs fourteen.
const THE_FRAME_PROLOGUE: &[u8] = &[
    0x48, 0x89, 0x5c, 0x24, 0x08, 0x48, 0x89, 0x74, 0x24, 0x10, 0x57, 0x48, 0x81, 0xec, 0xd0, 0x00,
    0x00, 0x00,
];

/// **The draw chain's head**, which the frame walks inline at `0x1400258fd` — so there is no function of
/// the game's to patch and the walk is orb's own, see [`walks_the_draw_chain`].
const THE_DRAW_CHAIN: usize = 0x1_403a_2940;

/// **What takes an element out of either chain**, for the walk whose callback answered nought:
/// `0x14000f350` searches the calc chain's head and then the draw chain's for it, unlinks it, calls its
/// own taken-out callback and frees it where it was heap-allocated. Only `rdx` is its argument — the
/// first thing it does is test it — and the frame passes the element there.
const TAKES_AN_ELEMENT_OUT: usize = 0x1_4000_f350;

/// The fields of a `ChainElem` a walk reads. `th06c/asm.md` has the whole `0x40` bytes of one.
mod chain {
    /// **The callback a frame runs**, called with [`ON`] in `rcx`.
    pub const CALLBACK: usize = 0x8;
    /// The link a walk follows.
    pub const NEXT: usize = 0x28;
    /// What that callback is called on.
    pub const ON: usize = 0x38;
}

/// **What a chain's callback answers, and what the walk does about it** — read off `0x14002591a`
/// onwards, which is the draw chain's own walk.
mod walked {
    /// Call it again where it stands.
    pub const AGAIN: i32 = 2;
    /// Take the element out of the chain and carry on with the next.
    pub const TAKE_IT_OUT: i32 = 0;
    /// The walk is over. Three answers with one meaning for the draw chain, whose walk ends on each of
    /// them; the calc chain's own frame reads which one it was.
    pub const OVER: [i32; 3] = [3, 4, 5];
}

/// **The viewport the frame sets before anything of it runs**: `0x140073ed0` with the whole 640 by 480
/// output and the two floats the game keeps for the near and far planes, and the three fields it writes
/// beside them. Read off `0x14002572e`.
const SETS_THE_VIEWPORT: usize = 0x1_4007_3ed0;
const THE_VIEWPORTS_NEAR: usize = 0x1_40b7_768c;
const THE_VIEWPORTS_FAR: usize = 0x1_40b7_7690;
const WHAT_THE_VIEWPORT_WAS_ASKED_FOR: usize = 0x1_40b7_767c;
const THE_VIEWPORTS_WIDTH: usize = 0x1_40b7_7684;
const THE_VIEWPORTS_HEIGHT: usize = 0x1_40b7_7688;
/// The output's own size, which the frame hands over as a pair of integers.
const THE_OUTPUT: (u32, u32) = (0x280, 0x1e0);

/// **The matrix the frame puts in place after the viewport**: `0x14008ad00` builds one into the buffer it
/// is handed and answers a pointer to `0x40` bytes, which `0x140075e30` is then given a copy of. Read off
/// `0x140025769`–`0x1400257a5`, where the copy is the frame's own stack.
const MAKES_THE_MATRIX: usize = 0x1_4008_ad00;
const TAKES_THE_MATRIX: usize = 0x1_4007_5e30;
/// How many bytes of it are copied, which is what both calls are about: four `movaps` of sixteen.
const A_MATRIX: usize = 0x40;

/// **The sound's own catch-up, which the frame does between the two chains** — `0x14002585d` onwards.
/// What it is for is a run that has gone by faster than the music: the frame counts the extra walks of
/// the calc chain it made into [`THE_EXTRA_WALKS`], and this is where that count is turned into a seek
/// and cleared.
const THE_SOUNDS_RATE: usize = 0x1_400a_5fd0;
const WHERE_THE_MUSIC_IS: usize = 0x1_400a_5ed0;
const HOW_LONG_THE_TRACK_IS: usize = 0x1_400a_65b0;
const SEEKS_THE_MUSIC: usize = 0x1_400a_af30;
/// What the rate answers where it answers nothing at all, which is 44,100 — the frame's own fallback.
const SAMPLES_A_SECOND: i64 = 0xac44;
/// How many frames of them a second is, which is what the frame divides the rate by.
const FRAMES_A_SECOND: i64 = 60;
/// **How many walks of the calc chain have gone by that the music has not.** The frame increments it per
/// extra walk and this is what clears it.
const THE_EXTRA_WALKS: usize = 0x1_403a_4df0;
/// Which track is playing, and `-1` for none: the seek's second argument.
const THE_TRACK_PLAYING: usize = 0x1_403a_4dcc;
/// Whether that track is playing at all, and whether it loops — the two bytes the catch-up reads before
/// it decides which bound to wrap the seek inside.
const THE_TRACK_IS_PLAYING: usize = 0x1_403a_4ddc;
const THE_TRACK_LOOPS: usize = 0x1_403a_4ddd;
/// Where that loop starts and ends, in the same samples the seek is in.
const THE_LOOP_STARTS: usize = 0x1_403a_4de0;
const THE_LOOP_ENDS: usize = 0x1_403a_4de8;

/// **What the frame does after the draw chain and before it leaves**: a `-1` into the renderer's own
/// field at `0x14090a050`'s `+0x1cb50`, read off `0x14002593b`.
const THE_RENDERER: usize = 0x1_4090_a050;
const WHAT_THE_FRAME_LEAVES_IN_IT: usize = 0x1_cb50;

/// **And the two calls at the head of the frame's own timing, which are what finish its picture.**
/// `0x140025990` begins with `0x140058e40` on `0x140b77300` — the thing that puts the frame's own
/// `60.00fps` together, off the renderer's `+0x1cb57` — and then `0x140073b50`, which tails into the
/// renderer at `0x140071000`. Only after those two does it read a clock and wait.
///
/// **So the two are called and the wait is not**, which took a launch to tell apart: with the whole
/// function left out, everything of the game ran and nothing new ever reached the window — the picture
/// stood at the startup splash while the log walked the front end and played a stage, and a launch of
/// the same build with `--no-frame-loop` drew every screen. The draw chain was reaching six elements
/// and five jobs in both.
const PUTS_THE_RATE_TOGETHER: usize = 0x1_4005_8e40;
const WHAT_IT_PUTS_IT_ON: usize = 0x1_40b7_7300;
const FINISHES_THE_PICTURE: usize = 0x1_4007_3b50;
/// **How far into that object [`the_swapchain`] looks for one**, which is as far as the frame itself
/// reaches into it and a little over: what is being searched is the renderer's own fields, and the
/// furthest of them anything here names is the one above.
const THE_RENDERERS_OWN_FIELDS: usize = WHAT_THE_FRAME_LEAVES_IN_IT + 0x100;

/// **Where the renderer keeps its COM objects**, which `0x14024ff74` onwards releases one after another:
/// `0x14084d1e0` and `0x14084d218` are two of them, `0x14084d228` to `0x14084d260` are the rest, and
/// `0x14084d210` between them is a module handle rather than an object at all.
///
/// **A range walked and not a list of the six the release names**, because which one is the swapchain is
/// a question orb asks the objects themselves — see [`the_swapchain`] — and asking one more costs a
/// `QueryInterface`. What makes walking past a module handle safe is the vtable check there.
const THE_RENDERERS_OBJECTS: Range<usize> = 0x1_4084_d1e0..0x1_4084_d268;

/// Whether the search below has been made, so that it is made once a launch rather than once a frame.
static LOOKED_FOR_A_SWAPCHAIN: AtomicBool = AtomicBool::new(false);

/// **The present orb's own frame loop makes**, which is `dxgi`'s — and, where the swapchain was not
/// taken as it was made, a look for it among the pointers this game keeps first.
///
/// **That look is the second way in and not the first.** `Hooks::makes_its_renderer` is the first: the
/// swapchain a factory of orb's makes is taken there and nothing has to be searched for at all. What the
/// search is for is a launch whose factory came some other way — the two older names orb does not
/// answer — and it is made once rather than once a frame.
///
/// # Safety
/// Must run on the game's main thread, with everything of the frame drawn.
unsafe extern "win64" fn presents(_: usize) {
    if !LOOKED_FOR_A_SWAPCHAIN.swap(true, Ordering::Relaxed) {
        unsafe { look_for_the_swapchain() };
    }
    unsafe { crate::dxgi::present() };
}

/// **Which of the pointers this game keeps is a swapchain, asked of each of them in turn.**
///
/// `QueryInterface` and nothing read off the exe, which is what makes it hold across a build: what an
/// object *is* is a thing the object answers — see `dxgi::is_a_swapchain`.
///
/// **Where it looks is the renderer's own object as well as the block of statics beside it**, every
/// eight bytes of both: what `0x14024ff74` releases are the objects kept in `.data`, and the one that
/// presents may be a field of the object at [`THE_RENDERER`] instead, which is `0x1cb54` bytes at least
/// — `0x14002593b` writes that far into it.
///
/// **And what makes searching safe is the vtable check**: a field that is not a pointer to a mapped
/// image is never called on, which is what tells an object from the module handle the same block holds.
/// That handle reads as a pointer and what it points at is a `MZ` header, so a `QueryInterface` off it
/// would be a call into the letters of that header.
///
/// # Safety
/// Must run on the game's main thread, with the game's renderer already made.
unsafe fn look_for_the_swapchain() {
    let renderer = mem::read_committed::<usize>(at(THE_RENDERER)).unwrap_or(0);
    let fields = match renderer {
        0 => 0..0,
        renderer => renderer..renderer + THE_RENDERERS_OWN_FIELDS,
    };
    let mut asked = 0;
    for slot in THE_RENDERERS_OBJECTS
        .step_by(size_of::<usize>())
        .map(at)
        .chain(fields.step_by(size_of::<usize>()))
    {
        let object = mem::read_committed::<usize>(slot).unwrap_or(0);
        if object == 0 || !mem::vtable_in_image(object) {
            continue;
        }
        asked += 1;
        if unsafe { crate::dxgi::is_a_swapchain(object) } {
            crate::log!("frame: the swapchain is the object kept at {slot:#x}");
            unsafe { crate::dxgi::take(object) };
            return;
        }
    }
    // **Said with how many objects there were to ask**, which is the half of it worth having: nought of
    // them is a search that happened before the renderer existed, and a hundred is one that happened and
    // found no swapchain.
    crate::log!(
        "frame: none of the {asked} object(s) this game keeps answers to a swapchain, so the present \
         is whatever took it"
    );
}

/// **The calc chain walked, and its answer turned into one the frame loop reads.**
///
/// The walk answers a byte — nought to carry the frame on and anything else to leave it — and writes the
/// answer the frame leaves *with* through the pointer it is handed, which the game's own frame kept on
/// its stack. So the loop's two ways out are the other way about from this game's: nought here means
/// carry on, and what the loop reads as carry on is anything that is neither of its own two.
///
/// **And the word put into the game's own keys here.** This is the last moment before the walk that reads
/// them, and the rules were asked for it on the frame around this one — see
/// [`Game::presses_into_its_own_keys`], which has what a word left for the keyboard's own read costs.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn walks_the_calc_chain(_: *mut c_void) -> i32 {
    // **Every walk of a frame after the first counted where the game counts its own**, which is what
    // puts the music back in step with a run that has gone by faster than it: `--speed` runs this more
    // than once per frame and the sound is played by something else at its own rate, so the frames
    // between them are exactly what [`tops_up_the_sound`] seeks past. The game's own fast-forward
    // counts them in this same field.
    if WALKS_THIS_FRAME.fetch_add(1, Ordering::Relaxed) > 0 {
        let walks = mem::read_committed::<i32>(at(THE_EXTRA_WALKS)).unwrap_or(0);
        unsafe { mem::write::<i32>(at(THE_EXTRA_WALKS), walks.saturating_add(1)) };
    }
    if let Some(word) = crate::runtime::pressing() {
        unsafe { Th06c.presses_into_its_own_keys(word) };
    }
    let walk: unsafe extern "win64" fn(*mut i32) -> u8 =
        unsafe { std::mem::transmute(at(RUN_CALC_CHAIN)) };
    let mut leaving_with = 0i32;
    match unsafe { walk(&mut leaving_with) } {
        0 => CARRY_THE_FRAME_ON,
        _ => crate::runtime::CHAIN_EXIT_SUCCESS,
    }
}

/// What this game's calc walk answers where the frame is to carry on, in the frame loop's own terms:
/// anything that is neither `CHAIN_EXIT_SUCCESS` nor `CHAIN_EXIT_ERROR`.
const CARRY_THE_FRAME_ON: i32 = 1;

/// **How many walks of the calc chain this frame has had**, which [`Game::prepare_frame`] clears at the
/// top of one: what it decides is which of them are extra, and so how far the music has to be seeked to
/// be where the run is — see [`walks_the_calc_chain`].
static WALKS_THIS_FRAME: AtomicU32 = AtomicU32::new(0);

/// **Sixteen bytes' worth of alignment for a buffer the game's own code touches with `movaps`**, which
/// faults on anything less — see [`Game::prepare_frame`], whose matrix is one.
#[repr(align(16))]
struct Aligned<const N: usize>([u8; N]);

/// **The draw chain walked, which this game's own frame does inline and orb's loop calls here.**
///
/// The same walk as `0x1400258fd`: every element's callback with the element's own argument, and what it
/// answers says what the walk does next — see [`walked`]. Nothing it answers leaves the frame, that being
/// the calc chain's own business, so this always answers the loop's carry on.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn walks_the_draw_chain(_: *mut c_void) -> i32 {
    let mut element = at(THE_DRAW_CHAIN);
    // **How many elements this walk reaches and how many of them it runs, said once.** A walk that
    // reaches nothing is a frame with nothing drawn in it, which is a window standing at whatever it
    // last showed while everything else about the run goes on — and that is a thing no other line says.
    let mut reached = 0;
    let mut ran = 0;
    'walk: while element != 0 {
        reached += 1;
        let callback = mem::read_committed::<usize>(element + chain::CALLBACK).unwrap_or(0);
        if callback == 0 {
            element = mem::read_committed::<usize>(element + chain::NEXT).unwrap_or(0);
            continue;
        }
        let on = mem::read_committed::<usize>(element + chain::ON).unwrap_or(0);
        let run: unsafe extern "win64" fn(usize) -> i32 = unsafe { std::mem::transmute(callback) };
        ran += 1;
        loop {
            let answered = unsafe { run(on) };
            if answered == walked::AGAIN {
                continue;
            }
            let next = mem::read_committed::<usize>(element + chain::NEXT).unwrap_or(0);
            if answered == walked::TAKE_IT_OUT {
                let out: unsafe extern "win64" fn(usize, usize) =
                    unsafe { std::mem::transmute(at(TAKES_AN_ELEMENT_OUT)) };
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
    if !SAID_WHAT_THE_DRAW_WALKS.swap(true, Ordering::Relaxed) {
        crate::log!("frame: the draw chain reaches {reached} element(s), {ran} of them with a job");
    }
    CARRY_THE_FRAME_ON
}

/// So the line above is said once rather than once a frame.
static SAID_WHAT_THE_DRAW_WALKS: AtomicBool = AtomicBool::new(false);

/// **The sound's own catch-up, as the frame between the two chains has it.**
///
/// What it is about is a run that has gone by faster than the music: [`THE_EXTRA_WALKS`] counts the walks
/// of the calc chain the music has not heard, and this turns that count into a seek of the track by as
/// many frames' worth of samples and clears it. Nothing at all where the count is nought, which is every
/// frame of a run going by at its own speed.
///
/// **Held inside the track's own loop where it has one**, which is what the two bounds are for:
/// `0x1400258ab` onwards takes the seek back by the loop's length until it is inside it, and a track
/// with no loop is held against its own length instead.
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
    let rate: unsafe extern "win64" fn() -> i32 =
        unsafe { std::mem::transmute(at(THE_SOUNDS_RATE)) };
    let rate = match i64::from(unsafe { rate() }) {
        rate if rate > 0 => rate,
        _ => SAMPLES_A_SECOND,
    };
    let playing: unsafe extern "win64" fn() -> i64 =
        unsafe { std::mem::transmute(at(WHERE_THE_MUSIC_IS)) };
    let playing = unsafe { playing() };
    if playing < 0 {
        return;
    }
    let mut seek = rate / FRAMES_A_SECOND * walks + playing;
    match mem::read_committed::<u8>(at(THE_TRACK_LOOPS)) {
        Some(0) | None => {
            let length: unsafe extern "win64" fn(i32) -> i64 =
                unsafe { std::mem::transmute(at(HOW_LONG_THE_TRACK_IS)) };
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
        unsafe { std::mem::transmute(at(SEEKS_THE_MUSIC)) };
    unsafe { seeks(seek, track) };
}

/// **The player**, a static object of `0x9b88` bytes: `0x140049af0` clears it in two pieces and hands
/// it to the three `ChainElem`s it builds. `th06c/asm.md` has how it was found.
const PLAYER: usize = 0x1_40b6_d770;

mod player {
    /// Three floats, and the third is a depth — `0x140049e93` writes `0.49` into it.
    pub const POSITION: usize = 0x7248;
    /// **The two corners the game's own hit test is given**, at `0x14004c692`–`0x14004c6c8`, rather than
    /// the half they are worked out from: those are the numbers being compared.
    ///
    /// **And not the four floats at `+0x7274`**, which is what a reading of `0x14004c5d0`'s own prologue
    /// took them for: that address is the first of *sixteen* boxes the test walks *before* it reaches
    /// the player's own, each `0x10` bytes of width, height and a centre, and `0x14004a4db` onwards
    /// clears all sixteen every frame — so the `ucomiss` against nought at `0x14004c63a` is that walk
    /// finding an empty slot and not a player who cannot be hit. `th06c/asm.md` has both.
    pub const BOX_MIN: usize = 0x410;
    pub const BOX_MAX: usize = 0x41c;
    /// **What one frame of holding a direction moves the player by**, which is four numbers: the switch
    /// at `0x14004acbf` picks the pair by direction and the focus bit picks which of the pair.
    pub const SPEED_STRAIGHT: usize = 0x7380;
    pub const SPEED_STRAIGHT_FOCUSED: usize = 0x7384;
    pub const SPEED_DIAGONAL: usize = 0x7388;
    pub const SPEED_DIAGONAL_FOCUSED: usize = 0x738c;
    /// And what that is scaled by on each axis before it reaches the position — `mulss` at
    /// `0x14004af14` and `0x14004af24`. A pair, because nothing says the two axes are scaled alike.
    pub const SCALE_ACROSS: usize = 0x7228;
    pub const SCALE_DOWN: usize = 0x722c;
    /// **Which of four states the player is in**, one byte.
    pub const STATE: usize = 0x73b8;
    /// The state a hit can land in, and the only one: `0x14004c6ce` gates the hit on this byte being
    /// nought and `0x14004c734` is what writes [`HIT`] over it.
    pub const NORMAL: u8 = 0;
    pub const HIT: u8 = 2;
    /// **The frames a bomb still takes the hit back in**, one off a frame at `0x14004a6ee`, and the frame
    /// it reaches nought is the frame the death is made final — `0x14004a6fd` onwards reads the lives.
    ///
    /// **Which is what says a hit was taken at all**, where the state above says what the player is in
    /// the middle of: read beside it in the line a hit writes, because a run kept reporting hits with
    /// nothing within sixteen of the player and the two readings tell those apart.
    pub const A_BOMB_STILL_TAKES_IT_BACK: usize = 0x7254;
    /// And the frames of invulnerability the state above counts down — `0x14004ab8e`, whose pair is a
    /// `Timer`.
    pub const UNTOUCHABLE: usize = 0x7378;
}

/// **The field the player may move in**, four floats — left, top, width, height — which
/// `0x14004af46`–`0x14004af9f` puts the position back inside.
///
/// **And nothing in the exe writes them**, which is why [`sight::field`] does not trust them alone: they
/// initialise to nought in `.data`, and a search of every function `.pdata` names and every byte between
/// those functions found no instruction naming one as a destination. A rectangle of nought would pin the
/// player at its corner, so the game fills them some way this reading has not found.
const PLAYER_AREA: usize = 0x1_403a_4b84;

/// **The field's own size**, two floats initialised in `.data` to `384` and `448`, and what every
/// comparison a coordinate is held against reads — `0x140049e49` onwards, where a life starts at half
/// the first and 64 up from the second.
const FIELD_SIZE: usize = 0x1_403a_33b4;

/// The run's own numbers, each a static of its own — `th06c/asm.md` says what each was read off.
const LIVES: usize = 0x1_403a_4b94;
const BOMBS: usize = 0x1_403a_4b95;
/// A word, and `0x14002aa8e` holds the cap it is written back at.
const POWER: usize = 0x1_403a_33c4;
const POWER_FULL: u16 = 128;
/// Counted from one: `0x140027065` multiplies it by a thousand into `Stage * 1000 = %5d`, and
/// `0x14002712e` gives the lives and bombs their bonus only where it has reached six.
const STAGE: usize = 0x1_403a_33c0;
/// Nought for Easy up to four for the Extra, which is the order the five `… Rank * …` lines at
/// `0x1402c7c70` onwards are picked in at `0x1400271a3`.
const DIFFICULTY: usize = 0x1_403a_3b68;

/// **Whether the stage on the screen is a replay being played back**, which is the byte both of this
/// game's playbacks set and a run of somebody's own clears: `0x1400306ab` sets it as the replay list's
/// own decide copies the file it names into [`THE_REPLAY_BEING_PLAYED`], `0x14002f90d` sets it where the
/// logo screen starts the attract demo, and `0x14002cb87` clears it inside the same function that writes
/// `0x302` over the lives and the bombs — a run starting. What reads it is `0x140024c45`: set, the path
/// at [`THE_REPLAY_BEING_PLAYED`] goes to `0x14004d0d0` to be *read*, where clear sends
/// `replay/th6_00.rpy` to that same call to be written.
const A_REPLAY_IS_PLAYING: usize = 0x1_403a_3b48;

/// **And whether that playback is the attract demo**, the one of the two nobody asked for.
/// `0x14002f90d` is the only writer that sets it and `0x140037842` the one that clears it — the run's
/// own hand-over back to the title, which starts `bgm/th06_01.mid` for a run that ended and does
/// nothing but clear these two bytes for a demo that did. What reads it is the panel: `0x1400299b7`
/// draws ` DEMO PLAY` at `0x1402c8080` in place of the stage's own name, which is what a picture of one
/// said while every line of the log still read it as a stage.
const THE_PLAYBACK_IS_THE_DEMO: usize = 0x1_403a_3b61;

/// **The path of the replay being played**, `0x104` bytes written two ways: `0x14002f921` copies
/// `data/demo/demo00.rpy` over it where the demo starts, and `0x1400306d0` copies the name out of the
/// record the replay list's cursor is on. Read for the log alone — which of the two a playback is, is
/// [`THE_PLAYBACK_IS_THE_DEMO`]'s answer and not this one's.
const THE_REPLAY_BEING_PLAYED: usize = 0x1_403a_4bfc;
/// **The game manager**, a static of `0x11628` bytes whose frame callback is `0x14002cc00`.
const GAME_MANAGER: usize = 0x1_40b5_c140;

mod game_manager {
    /// The dword `0x14002cc3b` reads and `0x14002cc58` jumps through a table of twenty-one at
    /// `0x140036324` on. Twelve to seventeen are a stage's own six phases, which `0x14002f95d` takes
    /// twelve off and jumps through a second table of six.
    pub const STATE: usize = 0x11608;
    /// **Which item of the front end the cursor is on**, counted from nought: `0x140030650` steps it up
    /// and wraps it at seven, and the decide at `0x140030695` reads the record of the item it names.
    ///
    /// One field whatever screen is up, which is what the front end here is — a menu whose items carry
    /// their own records, and the decide follows the one at `[+0x11610]` into the next list. Which is
    /// why a recipe aiming with it names the screen too.
    pub const CURSOR: usize = 0x28;
}

/// **The calc chain's own head**, the other of the two — see [`THE_DRAW_CHAIN`], and
/// `th06c/asm.md` for why both are sentinels rather than pointers.
const THE_CALC_CHAIN: usize = 0x1_403a_2980;

/// Longer than any chain this game builds, so that a chain caught in a state whose links do not end is a
/// walk that answers nothing rather than one that does not return.
const CHAIN_LINKS: usize = 64;

/// **The job the screen a run ends at registers**, which is the one way to that screen: it is built on
/// the heap, nothing holds a pointer to it, and `0x14004fd3d` is where its own constructor puts it into
/// the element it registers — see [`what_a_job_runs_on`].
const RESULT_SCREEN_UPDATE: usize = 0x1_4004_feb0;

/// **The job an ending runs as**, which is what says one is running: nothing else does.
///
/// **Because an ending of this game is a job and not a screen.** It runs inside the scene the stage it
/// followed had — one launch's frame lines read `scene=11 stage=5 enemies=0 bullets=0 boss=false` for the
/// whole of it — so the game manager's own state says a stage is still up, and `State::in_ending` had
/// nowhere to come from. The chain is where it is: this callback arrives on the frame the stage's own ten
/// jobs go and stands alone until the screen a run ends at registers.
///
/// **Found by walking the chain rather than by the file it reads.** `th06ED.dat`'s string sits at
/// `0x1402c60f8` and no reference to it is in `.text` at all — not rip-relative, not indexed, not a
/// `movabs` operand — which is the negative the name entry's own grid gave before that was found the same
/// way. So [`say_the_jobs_when_they_change`] wrote the chain's callbacks down whenever the set of them
/// changed, and one launch of a Normal clear named this one outright: the stage's jobs at `f98220`, this
/// one alone from `f98220` to `f106680` — `8,460` frames, which is the ending's own length — and
/// [`RESULT_SCREEN_UPDATE`] registered on the frame after it went.
///
/// **And the staff roll goes with it.** `State::ending_script` is what th06 1.02h's skip watches to stop
/// where the roll begins, and this game's own script is unread, so the skip runs to where this job leaves
/// — which is the roll's end and not its start.
const ENDING_UPDATE: usize = 0x1_4001_7e50;

mod result_screen {
    /// **The screen's own state**, which its update reads at `0x14004fee1` and jumps through a table of
    /// eighteen at `0x140053538` on.
    pub const STATE: usize = 0x5194;

    /// **The one of those eighteen the name is typed in.** `0x14004fd59` writes it as the screen is
    /// built — nine where `[0x1403a3b60]` is clear and seventeen where it is not, which is the run that
    /// is asked for no name at all.
    pub const TYPING_A_NAME: i32 = 9;

    /// **The field the entry types into**, eight bytes: `0x140051fb4` writes the character at
    /// `[this + 0x5173 + index]`, the index being [`TYPED`] capped at seven by the `cmovge` at
    /// `0x140051f90`.
    ///
    /// **The same shape th06 1.02h has**, whose own is `[this+0x5193]` with the cursor at `+0x20` — this
    /// build has the cursor at the front of the object instead and the field `0x20` earlier.
    pub const NAME: usize = 0x5173;
    pub const NAME_BYTES: usize = 8;

    /// **How many characters have been typed**, which `0x140051fc8` steps up and `0x140051fc1` stops at
    /// eight — and reaching eight is what `0x140051fd3` puts the cursor on [`THE_END_ITEM`] for.
    pub const TYPED: usize = 0x517c;

    /// **Which character of the grid the cursor is on**, the first dword of the screen's own object:
    /// `0x140051f93` reads it, `0x140051f9b` takes the character at `0x1402c8af0 + it`, and
    /// `0x140051fac` writes a space where it is `0x5e`.
    ///
    /// **The grid is reached off the image base in a register** and not rip-relative —
    /// `[rax + r11 + 0x2c8af0]`, r11 being the base — which is why no search of the exe for a reference
    /// to it as an operand finds one. `th06c/asm.md` has the sweep that said so.
    pub const CURSOR: usize = 0x0;

    /// **The item that ends the entry**, which is the one past the space at `0x5e`: `0x140051fa6` leaves
    /// the typing for anything past that space, and this is what `0x140051fd3` puts the cursor on once
    /// eight characters have been typed.
    pub const THE_END_ITEM: i32 = 0x5f;
}

/// **The object a job of the calc chain is run on**, found by the callback it registered.
///
/// **Which is the only way to a screen this game keeps on the heap**: nothing holds a pointer to one, and
/// the element the screen registers carries both — the callback at [`chain::CALLBACK`] and the object at
/// [`chain::ON`].
///
/// Every step asked of the memory map rather than read outright, a chain being a list the game is free to
/// be halfway through rebuilding.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn what_a_job_runs_on(callback: usize) -> Option<usize> {
    let mut element = mem::read_committed::<usize>(at(THE_CALC_CHAIN) + chain::NEXT)?;
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

/// **Every job the calc chain holds, said the frame the set of them changes and on no other frame** —
/// which is how the job that draws something nothing points at is found.
///
/// **Because a screen this game keeps on the heap is reached through the job that registers it**, and
/// which job that is, for a thing no string of the exe is referenced from, is what a launch says rather
/// than what a listing does — see [`what_a_job_runs_on`], and `th06c/asm.md`'s note on the name
/// entry's own grid, whose address appears in `.text` as no operand at all. So the chain is walked and
/// the callbacks are written down whenever one arrives or leaves: the job added on the frame the last
/// boss dies is the ending's own.
///
/// **On the change and not every frame**, the walk being `CHAIN_LINKS` reads of the game's memory: what
/// is interesting is the arrival, and a line a frame would be a page a second.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn say_the_jobs_when_they_change() {
    static WAS: AtomicUsize = AtomicUsize::new(0);
    let mut element = match mem::read_committed::<usize>(at(THE_CALC_CHAIN) + chain::NEXT) {
        Some(element) => element,
        None => return,
    };
    let mut jobs = [0usize; CHAIN_LINKS];
    let mut held = 0;
    // A number the set of them makes, so that the walk is compared without keeping the last one: the
    // callbacks added up, each turned by where it sits so that two jobs swapping is a change as well.
    let mut fingerprint = 0usize;
    for link in 0..CHAIN_LINKS {
        if element == 0 {
            break;
        }
        let Some(callback) = mem::read_committed::<usize>(element + chain::CALLBACK) else {
            break;
        };
        jobs[link] = callback;
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

/// **Whether the calc chain holds a job of its own**, which is what a thing that neither moves nor is
/// pointed at is read by — see [`ENDING_UPDATE`].
///
/// Apart from [`what_a_job_runs_on`] because the two ask different things: that one wants the object a
/// job runs on and answers nothing for a job whose object is null, where this one is about the job being
/// there at all.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_chain_holds(callback: usize) -> bool {
    let mut element = mem::read_committed::<usize>(at(THE_CALC_CHAIN) + chain::NEXT);
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

/// **The bullets**, a static of `0xf7c20` bytes — see `th06c/asm.md`, where the array was read off
/// the update's own loop at `0x14000d8c0`.
const BULLET_MANAGER: usize = 0x1_4090_a190;

mod bullet_manager {
    pub const BULLETS: usize = 0x8;
    pub const SLOTS: usize = 0x280;
    pub const BYTES: usize = 0x5d0;
    /// Counted up per live one at `0x14000d9fb`.
    pub const LIVE: usize = 0xe8808;
    pub const STEP: usize = 0x08;
    pub const POSITION: usize = 0x30;
    /// **Which of five states it is in**, a word the update jumps on at `0x14000da02`, and nought is a
    /// slot nothing is in.
    pub const STATE: usize = 0x44;
    /// **And the only one the player is tested against.** State 1's branch is the one whose path reaches
    /// `0x14000e12b`, and the four calls of the player's own test at `0x14004c5d0` are all on it; the
    /// other four end at the manager's own loop tail with no test made.
    pub const FLYING: u16 = 1;
    /// **The size the game's own hit test is given**, which `0x14000e1ae` hands over beside the position
    /// at `0x14000e1c0`.
    pub const SIZE: usize = 0x5a4;

    /// **What a bullet's own step is scaled by before it reaches its position**, which is its state's and
    /// not one number — five integrations, one per branch of `0x14000da02`'s jump:
    ///
    /// | state | where | what it multiplies by |
    /// | --- | --- | --- |
    /// | 1 | `0x14000e0a2` | nothing at all: `[+0x30] += [+0x8]` |
    /// | 2 | `0x14000da87` | a half — `1.0 / 2.0` |
    /// | 3 | `0x14000da9d` | `1.0 / 2.5` |
    /// | 4 | `0x14000daae` | `1.0 / 3.0` |
    /// | 5 | `0x14000da29` | nothing at all either |
    ///
    /// **The jump is a chain of subtractions and not a table**: `0x14000da06` onwards takes `r15d` — one —
    /// off the state four times over, sending 1 to `0x14000db34`, 2, 3 and 4 to the divisor chain at
    /// `0x14000da82`, and letting 5 fall through to `0x14000da29`.
    ///
    /// **And state 5 was read as a half, which a run then contradicted.** `0x14000da45`'s `mulss xmm1,xmm13`
    /// is in the code that branch falls into and `xmm13` is `0x1402c9490`'s `0.5`, so the half looked like
    /// the answer — but what that block scales is `[rsi]`..`[rsi+8]` into `[rdi]`..`[rdi+8]`, off the
    /// `rbx+0x490` the `lea` before it takes, and **not** the bullet's own `+0x8` into its `+0x30`. A launch
    /// said so outright: `the widest of them moved 3.30,5.38 saying 1.65,2.69, in the game's own state 5` —
    /// twice what a half predicts, on both axes, which is the step applied whole.
    ///
    /// **And getting this wrong is what a flat half cost**: a launch that scaled every state by `0.5`
    /// had the rules judge ten of ten bullets as moved by something other than their own step, so every
    /// one of them was feared over the whole box it had just swept and none was projected — the model
    /// could only dodge what was already on the player, and that run was down to its last life by
    /// stage 3's boss.
    pub const fn a_steps_scale(state: u16) -> f32 {
        match state {
            2 => 1.0 / 2.0,
            3 => 1.0 / 2.5,
            4 => 1.0 / 3.0,
            _ => 1.0,
        }
    }
}

/// **The lasers**, which are in that same static: `0x14000e23d` puts the array at `+0xe8818` of the
/// bullet manager and walks sixty-four of them, right after the bullets and in the same update.
const LASERS: usize = 0x1_409f_29a8;

mod laser {
    /// `sub rdi,r15` from `0x40` at `0x14000e3c7`.
    pub const SLOTS: usize = 0x40;
    /// `add rbx,0x278` at `0x14000e3c0`.
    pub const BYTES: usize = 0x278;
    /// A byte, and nought is a slot nothing is in — `0x14000e2a0`. `0x14000e39e` clears it once the
    /// near end has run off the far edge.
    pub const IN_USE: usize = 0x254;
    /// **How wide it is**, and the test is given half of it: `0x14000e2f2` reads it and `0x14000e30c`
    /// halves it into the size handed to `0x14004c7d0`.
    pub const WIDTH: usize = 0x4;
    /// **What a hit on it is, to either side of its own line**: a quarter of that width, which is two
    /// halvings read off the exe.
    ///
    /// `0x14000e30c` multiplies the width by `xmm13` into the size the test is handed — `[rsp+0x34]`, the
    /// second of the pair at `0x14000e317`, which `0x14004c7d0` compares on the axis *across* the beam —
    /// and that test multiplies it by `xmm6` again at `0x14004c8ac`. Both registers hold
    /// `[0x1402c9490]`, which is `0.5`, and the same `xmm6` is what the bullet test halves the bullet's
    /// own size with — where the model reads the half as the size over two and is right.
    ///
    /// **A half was here, set against a run, and the run had another explanation.** A laser `11.7` off a
    /// player, modelled `6.0` wide by this arithmetic, hit it — and a laser's origin is carried by the
    /// script that fired it, which nothing read at the time: th06c's Extra boss fires one twice this
    /// width from where she stands and walks it `6.4` a frame, so the lane that hit was `6.4` from where
    /// the judgement had it and not twice as wide. What a half cost meanwhile is in `th06c/asm.md`:
    /// the gaps of a fan of lasers all read as shut, and two hits of an Extra run with `nowhere open to
    /// it` on every one of the nine moves.
    pub const HALF_OF_ITS_WIDTH: f32 = 4.0;
    /// How far along it runs, from the near end to the far one. The far end is what grows and the near
    /// end follows it once they are [`LENGTH`] apart — `0x14000e2ad`–`0x14000e2df`.
    pub const NEAR: usize = 0x10;
    pub const FAR: usize = 0x14;
    /// And how much further the far end reaches each frame, added straight on with no scale.
    pub const GROWS: usize = 0x24c;
    /// **Where it is fired from and which way it points**, which is the pivot `0x14004c7d0` turns the
    /// player about: `xmm8`/`xmm9` from `[r9]`, and the angle in `xmm0`.
    pub const ORIGIN: usize = 0x258;
    pub const ANGLE: usize = 0x240;
    /// The frames it has been in its phase, and the phase — `0x14000e305` reads the phase and
    /// `0x14000e3b3` is the pair `+0x8` and `+0xc` are.
    pub const FRAMES: usize = 0xc;
    pub const PHASE: usize = 0x270;
    /// **The frame of its first phase it starts hitting on**: `0x14000e4d1` runs the player's own test
    /// only once the frames have reached it, so a laser before that is drawn and hits nothing.
    pub const HITS_FROM: usize = 0x26c;
    /// From this phase on the test is run every frame — `0x14000e577` and `0x14000e66d`.
    pub const FIRING: u8 = 1;
    /// And from this one it is going out, `0x14000e5ed` onwards taking `width × (1 − frames ÷`
    /// [`NARROWS_OVER`]`)` and writing it where the length goes.
    pub const FADING: u8 = 2;
    /// **How many frames it goes out over**, which is what that fraction is of.
    ///
    /// **And what it takes down is the length**, not the width: `0x14000e61a` writes that number into
    /// `[rsp+0x30]`, the slot `0x14004c89a` compares *along* the beam, where `0x14000e317` put the width
    /// in `[rsp+0x34]` for the axis across it. So a fading laser is a stub about the middle of its own
    /// line at the width it always had, and `0x14000e5eb` skips the whole of it where this is nought.
    pub const NARROWS_OVER: usize = 0x18;
}

/// **The enemies**, a static of `0xff020` bytes whose frame callback is `0x1400204c0` and whose array
/// that function's own loop says the shape of.
const ENEMY_MANAGER: usize = 0x1_40a3_2f60;

mod enemy_manager {
    pub const ENEMIES: usize = 0x8;
    /// `cmp r15d,0x100; jl` at `0x1400210a5`.
    pub const SLOTS: usize = 0x100;
    /// `add rbx,0xfd0` at `0x140021097`.
    pub const BYTES: usize = 0xfd0;
    /// Counted up per live one at `0x14002059d`, into the manager's first dword.
    pub const LIVE: usize = 0x0;
}

mod enemy {
    /// `lea rbp,[rbx+0xa8]` at `0x1400205a0`, and `+0xfa4` onwards is what the walk adds to it.
    pub const POSITION: usize = 0xa8;
    /// **The size the player's own hit test is given**, which `0x140020af5` onwards multiplies by two
    /// thirds before handing over — and that test halves what it is given, so what a hit is is a third
    /// of this on each side.
    pub const SIZE: usize = 0x2a4;
    /// **What a hit is, on each side of it**: a third of the size above, which is that size times two
    /// thirds at `0x140020af5` and then halved by the test at `0x14004c5d0`.
    ///
    /// **A third and not a sixth.** The half was worked out as this over two — the size over three and
    /// then halved again — which is every enemy's body read at half the reach the game gives it.
    pub const A_THIRD_OF_ITS_SIZE: f32 = 3.0;
    /// The flags byte the walk itself gates on: `cmp BYTE PTR [rbx+0xb4],0; jge` at `0x140020590`
    /// skips a slot whose top bit is clear.
    pub const FLAGS: usize = 0xb4;
    pub const IN_USE: u8 = 0x80;
    /// **And whether the game runs its body against the player at all.** `0x140020ac2` requires `0x4`,
    /// `0x140020adc` refuses where `+0xb6`'s `0x8` is set, and `0x140020aef` runs the test only where
    /// the low two bits are both up.
    pub const SOLIDITY: usize = 0xb5;
    pub const TESTED: u8 = 0x4;
    pub const SOLID_BITS: u8 = 0x3;
    /// **And which one the fight is with.** `0x140020f62` tests this bit of the same byte and, for the
    /// one that has it, works `+0x21c / +0x220` out into `0x14090a0cc` — which is the life bar the panel
    /// draws, so the enemy carrying it is the boss.
    pub const IS_THE_BOSS: u8 = 0x8;
    /// Its life, which `+0x220` beside it is the most of.
    pub const LIFE: usize = 0x21c;
    pub const MORE_FLAGS: usize = 0xb6;
    pub const PASSES_THROUGH: u8 = 0x8;
}

/// **The items**, a static of `0x2a160` bytes whose walk is `0x14002a970`.
const ITEM_MANAGER: usize = 0x1_40b3_1fe0;

mod item {
    pub const ITEMS: usize = 0x8;
    /// `cmp r9d,0x200; jl` at `0x14002b956`.
    pub const SLOTS: usize = 0x200;
    /// `add r14,0x150` at `0x14002b932`.
    pub const BYTES: usize = 0x150;
    /// A byte, and nought is a slot nothing is in — `cmp BYTE PTR [r14],bpl; je` at `0x14002aae0`.
    pub const ALIVE: usize = 0x0;
    pub const POSITION: usize = 0x10;
    /// **Which of seven kinds it is**, a signed byte the walk jumps through a table of seven at
    /// `0x14002ba1c` on — `0x14002ad97`. What each number is is what its own case does:
    pub const KIND: usize = 0x34;
    /// `inc r15w` into the power at `0x14002ae2e`.
    pub const POWER: i8 = 0;
    /// The point items counted up at `0x14002b1d8`.
    pub const POINT: i8 = 1;
    /// `add r15w,0x8` into the power at `0x14002b29e`.
    pub const BIG_POWER: i8 = 2;
    /// The bombs, stopped at eight and stepped up at `0x14002b509`.
    pub const BOMB: i8 = 3;
    /// The power written to its cap at `0x14002b5eb` on.
    pub const FULL_POWER: i8 = 4;
    /// The lives, the same shape at `0x14002b538`.
    pub const LIFE: i8 = 5;
    /// The one a bomb leaves behind, worth a score off the graze — `0x14002b7db`, and the bullet
    /// manager spawns it as kind 6 at `0x14000e19d`.
    pub const CANCELLED: i8 = 6;
}

/// How much of the 256-byte snapshot is a key: every index, which is what `GetKeyboardState` fills.
const KEYS_IN_A_SNAPSHOT: usize = 256;

/// **The 256 bytes the game plays a run off**, which its own read fills through the table below — see
/// `th06c/asm.md`, and [`Th06c::presses_into_its_own_keys`], which writes into it for a frame that
/// is past the read.
const THE_KEYS_THE_GAME_READS: usize = 0x1_403d_e9bc;
/// **What a key that is down reads as there**, the same `0x80` a `GetKeyboardState` snapshot carries.
const DOWN_IN_ITS_OWN_KEYS: u8 = 0x80;
/// And what one that is not reads as, which is what the game's own read leaves for every key nobody is
/// holding — see `Th06c::presses_into_its_own_keys`, which writes it for the keys a word does not hold.
const UP_IN_ITS_OWN_KEYS: u8 = 0x00;
/// **The game's own table from a virtual key to the index it keeps that key at**: six bytes an entry —
/// the virtual key, the index, and two bytes of nought — walked at `0x14008fb00` and ending at a key of
/// `0xffff`. `A` is `0x41 -> 0x1e`, which is `DIK_A`, so the buffer is indexed by DirectInput's own scan
/// codes.
///
/// **Read rather than written down**, because a scan code per button is nine numbers to get right and
/// this is the mapping the game itself uses.
const THE_TABLE_OF_KEYS: usize = 0x1_4028_72e0;
/// How far apart its entries are, and how many of them there can be before the walk gives up: the table
/// ends at a key of `0xffff` and `0x41` entries is what the walk was measured over, so twice that is a
/// bound that cannot loop on a table orb has misread.
const A_KEY_IN_THE_TABLE: usize = 6;
const KEYS_IN_THE_TABLE: usize = 0x82;
/// What the table's own end is.
const NO_MORE_KEYS: u16 = 0xffff;

/// What a key that is down reads as in that snapshot, which is what `GetKeyboardState` writes: the high
/// bit of the byte.
const DOWN: u8 = 0x80;

/// The pad's own packet number, which moves on every state orb hands over — see
/// [`Game::press_into_pad`].
static PACKETS: AtomicU32 = AtomicU32::new(1);

/// What the log's own `run:` line last said, so that [`Game::read_state`] says it once each time any of
/// it changes rather than once a frame: the stage, the difficulty, the scene, the lives and the bombs.
static SAID: std::sync::Mutex<Option<(i32, i32, i32, i8, i8)>> = std::sync::Mutex::new(None);

/// Whether [`A_REPLAY_IS_PLAYING`] was set the last time [`Game::read_state`] looked, so that a
/// playback starting and a playback ending are each one line.
static PLAYING_BACK: AtomicBool = AtomicBool::new(false);

/// **The file the playback is reading**, off [`THE_REPLAY_BEING_PLAYED`] and cut at the NUL the copy
/// that wrote it left.
///
/// `0x40` bytes is a bound and not a reading: nothing in the exe says how long that buffer is, and both
/// of the paths written into it are this game's own — `data/demo/demo00.rpy` and `replay/th6_NN.rpy`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_replays_path() -> String {
    let bytes = mem::read_committed_bytes(at(THE_REPLAY_BEING_PLAYED), 0x40).unwrap_or_default();
    let path = &bytes[..bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len())];
    String::from_utf8_lossy(path).into_owned()
}

pub struct Th06c;

impl Th06c {
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
}

impl Game for Th06c {
    /// Kept as the distance from [`IMAGE_BASE`] — see [`at`].
    fn set_image_base(&self, base: usize) {
        SLIDE.store(base.wrapping_sub(IMAGE_BASE), Ordering::Relaxed);
    }

    /// **The DirectInput keyboard taken away, so that the keys orb writes are the keys a run is played
    /// with** — see [`DEVICE`], which has the measurement.
    ///
    /// Asked every frame rather than once, because the game makes that device during its own startup
    /// and remakes it whenever the window is activated again: a launch that nulled it at the attach
    /// would be a launch whose run took the device back.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn acquire_input(&self) -> bool {
        if orb_api::mem::read_committed::<usize>(at(DEVICE)) == Some(0) {
            return true;
        }
        unsafe { orb_api::mem::write::<usize>(at(DEVICE), 0) };
        true
    }

    /// **The keyboard read and the startup dialog**, which is as far as this game has been read: the read
    /// stood in front of by answering the name the game asks for rather than by patching code, and the
    /// dialog got past by clicking the controls a person would — see [`crate::game::Read::LookedUp`] and
    /// [`Dialog`].
    ///
    /// Every other hook is `None` because the address it would need is unread, and the two the seam
    /// takes outright are `None` for the same reason — a launch here has no frame of orb's own, which is
    /// what [`Hooks::update`] says.
    fn hooks(&self) -> Hooks {
        Hooks {
            // **Neither, and not for want of a frame loop: this game's two walks are handed over.**
            // Nothing of the game's calls either of them once the frame is orb's — that frame was the
            // only caller — so there is no prologue to be in the path of, and a patch could not reach
            // the draw side at all, its walk being inline in that frame. See [`walks`](Hooks::walks).
            update: None,
            draw: None,
            walks: Some(Walks {
                update: walks_the_calc_chain as *const () as usize,
                draw: walks_the_draw_chain as *const () as usize,
            }),
            // **Nothing, because orb is inside this game's frame now rather than outside it.** What that
            // hook was is one call per walk of the calc chain for a game whose frame orb had not been
            // read far enough into to replace; the walk is orb's own to call now, and the rules are
            // asked where every other game's are.
            runs_its_frame: None,
            save_replay: None,
            stop_recording: None,
            stage_begun: None,
            stage_building: None,
            unlocks_read: None,
            ranking_read: None,
            card_records_read: None,
            create_window: None,
            init_device: None,
            // **The whole frame, which is what makes it orb's**: the loop at `0x14002c4b7` calls this
            // once round and orb's own loop stands in its place, calling every piece of it — see
            // [`THE_FRAME`], and `Game::prepare_frame`, `Game::end_drawing` and `Th06c::frame_calls`,
            // which are where the pieces that are not the two walks went.
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
            startup_dialog: Some(Dialog {
                clicks: &[dialog::A_640_BY_480_WINDOW, dialog::START],
            }),
            makes_its_window: Some(Lookup { name: WINDOW_MADE }),
            moves_its_window: Some(Lookup { name: WINDOW_MOVED }),
            reads_a_pad: Some(Lookup { name: PAD_READ }),
            makes_its_renderer: Some(Lookup {
                name: RENDERER_MADE,
            }),
        }
    }

    /// **No**, and not because a run of it could not be: what a chapter needs is a snapshot of the
    /// address space a run is in, and which of this game's memory that is has not been read. A launch
    /// here is a launch of the other mode in everything but name.
    fn rewinds(&self) -> bool {
        false
    }

    /// **A word's buttons put into the snapshot the game is about to read**, which is how a bot presses
    /// anything here: the game reads its keys once a tick and turns them into its own word itself, so
    /// what orb writes is the keys and not the word — see [`crate::runtime::keyboard_state`].
    ///
    /// Written into the snapshot the read was given rather than into the game's own key buffer at
    /// `0x1403de9bc`, because the translation between the two is the game's: a key written here reaches
    /// that buffer through the table at `0x1402872e0` the way the keyboard's own would. See
    /// `th06c/asm.md`.
    ///
    /// # Safety
    /// `state` must be the 256 bytes the game's read was given.
    unsafe fn press_into(&self, state: *mut u8, word: u16) {
        for (_, bit, key) in Self::BUTTONS {
            if word & bit != 0 && usize::from(*key) < KEYS_IN_A_SNAPSHOT {
                unsafe { state.add(usize::from(*key)).write(DOWN) };
            }
        }
    }

    /// **A word's keys written straight into the buffer this game plays a run off**, which is what a word
    /// decided on the game's own frame needs: that frame is past the read `press_into` writes into, so a
    /// word left there is acted on a frame later — see `Game::presses_into_its_own_keys`, which has what
    /// a frame of that cost.
    ///
    /// **Through the game's own table from a virtual key to its index**, rather than nine scan codes
    /// written down here: what the buffer is indexed by is DirectInput's own, and the mapping is the one
    /// the game's read uses — see [`THE_TABLE_OF_KEYS`].
    ///
    /// **And the keys the word does not hold are written up, not left alone.** That buffer is filled by
    /// the game's own read as well — `0x14008fad7` memsets the 256 bytes and the walk at `0x14008fb00`
    /// puts orb's snapshot into them — and that read is made at its own rate rather than once a frame,
    /// so a word written on top of one already carrying the *previous* word leaves the run acting on the
    /// union of the two. Which is not a word the rules ever chose, and the direction decode at
    /// `0x14004ac69` makes the union worse than a coin toss: it tests `0x10` before `0x20` and the sign
    /// bit last, so up beats down and right beats left whenever both are down. Measured — a stage 1 hit
    /// whose dump says `the word the run acts on is 0x81f1`, all four directions at once, where the
    /// rules had chosen `0x050`: the player went up and right out of a move chosen up and left, and six
    /// hits of one run were that same reversal.
    unsafe fn presses_into_its_own_keys(&self, word: u16) -> bool {
        let mut pressed = false;
        for (_, bit, key) in Self::BUTTONS {
            let index = unsafe { the_index_of_the_key(*key) };
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

    /// **A word's buttons put into the pad state orb answers this game's own read with**, which is how a
    /// bot presses anything that a run acts on — see `Hooks::reads_a_pad` and [`pad`].
    ///
    /// **The stick as well as the d-pad**, both at once: which of the two a game reads its directions
    /// from is the game's business, and a pad held to the corner says the same thing either way. Full
    /// deflection, since a stick half over is a game's own dead zone away from meaning nothing.
    ///
    /// # Safety
    /// `state` must be the sixteen bytes of an `XINPUT_STATE`.
    unsafe fn press_into_pad(&self, state: *mut u8, word: u16) {
        let mut buttons = 0u16;
        for (bit, button) in [
            (button::SHOT, pad::A),
            (button::DECIDE, pad::A),
            (button::BOMB, pad::B),
            (button::FOCUS, pad::X),
            (button::MENU, pad::START),
            (button::UP, pad::UP),
            (button::DOWN, pad::DOWN),
            (button::LEFT, pad::LEFT),
            (button::RIGHT, pad::RIGHT),
        ] {
            if word & bit != 0 {
                buttons |= button;
            }
        }
        let across = match (word & button::RIGHT != 0, word & button::LEFT != 0) {
            (true, false) => i16::MAX,
            (false, true) => i16::MIN,
            _ => 0,
        };
        // **Up is positive on a stick and down is negative**, which is the other way about from a
        // screen's own rows.
        let down = match (word & button::UP != 0, word & button::DOWN != 0) {
            (true, false) => i16::MAX,
            (false, true) => i16::MIN,
            _ => 0,
        };
        unsafe {
            // `XINPUT_STATE`: the packet number, then the gamepad's buttons, its two triggers and its
            // four thumb axes. The packet number moves every frame, a game that watched it for a change
            // being one that ignores a state it has already seen.
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

    /// The bits a recipe's button names are, which is what makes `press: [decide]` a decide here — see
    /// [`Th06c::BUTTONS`].
    fn button_bit(&self, name: &str) -> Option<u16> {
        Self::BUTTONS
            .iter()
            .find(|(button, _, _)| *button == name)
            .map(|(_, bit, _)| *bit)
    }

    /// **Which item of the front end the cursor is on**, and `None` while a stage runs — see
    /// [`game_manager::CURSOR`], which is one field for whatever list is up.
    unsafe fn front_end_cursor(&self) -> Option<i32> {
        match unsafe { sight::in_a_stage() } {
            true => None,
            false => mem::read_committed::<i32>(at(GAME_MANAGER + game_manager::CURSOR)),
        }
    }

    /// **And which screen that is**, by the game manager's own number for it — see
    /// [`game_manager::STATE`]. `None` while a stage runs, which is what makes a
    /// `until: { screen: N, cursor: M }` a list of the front end's and not a stage.
    unsafe fn front_end_screen(&self) -> Option<i32> {
        match unsafe { sight::in_a_stage() } {
            true => None,
            false => mem::read_committed::<i32>(at(GAME_MANAGER + game_manager::STATE)),
        }
    }

    /// One frame of a stage as this game holds it — see [`sight`], which is where every address it
    /// reads is written down.
    unsafe fn what_a_bot_sees(&self, into: &mut crate::game::Frame) -> bool {
        unsafe { sight::what_a_bot_sees(into) }
    }

    unsafe fn say_what_was_near(&self) {
        unsafe { sight::say_what_was_near() };
    }

    fn named(&self) -> &'static str {
        "th06c"
    }

    /// Which of orb's own bits the front end's two answers are — see [`button`], and
    /// `th06c/asm.md` for what of this game's own front end has been read, which is nothing.
    fn menu_decide(&self) -> u16 {
        button::DECIDE | button::SHOT
    }

    fn menu_cancel(&self) -> u16 {
        button::BOMB
    }

    /// The bits a run is written down with, which is every button orb has for this game: nothing of its
    /// own record has been read, so nothing here is left out of one.
    fn run_input(&self) -> u16 {
        let mut word = 0;
        for (_, bit, _) in Self::BUTTONS {
            word |= bit;
        }
        word
    }

    // **Almost everything below is unread.** Each of these is an address of this game's own that
    // `th06c/asm.md` does not carry yet, so what they answer is the answer that costs a launch
    // nothing: nothing read, nothing written, nothing claimed. A launch here reaches none of the
    // machinery they are for — [`Th06c::rewinds`] answers false and `Config`'s own defaults turn the
    // rest off — and the day one of them is read is the day the method beside it stops being a stub.
    // [`Game::read_state`] and [`Game::replaying`] are the two that have stopped.

    /// **The run's own numbers where they have been read, and nothing where they have not.**
    ///
    /// `scene` is the game manager's own state — the dword its frame callback jumps through a table of
    /// twenty-one on — and which of those numbers is which screen is unread, so `playing` is not taken
    /// from it: what says a stage is running is the stage the run has reached and the byte that holds
    /// everything, which is what [`sight::what_a_bot_sees`] gates on too and has the same reasoning
    /// beside it.
    ///
    /// Everything still answering nothing is a reading `th06c/asm.md` lists as not established —
    /// the script clock, the seed, the deaths, the boss and the card among them.
    unsafe fn read_state(&self) -> State {
        unsafe { say_the_jobs_when_they_change() };
        let scene =
            mem::read_committed::<i32>(at(GAME_MANAGER + game_manager::STATE)).unwrap_or(-1);
        // **The game's own number less one**, this game counting stages from one where `State::stage`
        // is counted from nought the way orb counts them everywhere — see `RunStart::stage`.
        let stage = mem::read_committed::<i32>(at(STAGE)).unwrap_or(0) - 1;
        let playing = unsafe { sight::in_a_stage() };
        // **Wider than `playing`, the way th06's is**: a frame the game holds everything on is a frame
        // of a run, and what reads this is whether a `play_until` is over — see [`sight::in_a_run`].
        let in_run = unsafe { sight::in_a_run() };
        let boss = unsafe { sight::the_boss_the_fights_own_slot() };
        let difficulty = mem::read_committed::<i32>(at(DIFFICULTY)).unwrap_or(-1);
        // **What run this is, said once each time any of it changes.** Which difficulty a launch played
        // is otherwise in no line of the log and on no part of the screen the panel draws, and a recipe
        // that walks this game's front end by taps cannot say which item it landed on — so the run orb
        // played is a thing only this says.
        //
        // **The lives and the bombs are in what makes a line and not only in what a line carries**,
        // because a death is the thing a reader is looking for and it happens inside a stage: keyed on
        // the stage alone, a launch that lost three lives on stage 5 said `5 live` at the stage's start
        // and nothing after it, and how many deaths the run had could only be read off the pictures.
        let lives = mem::read_committed::<i8>(at(LIVES)).unwrap_or(0);
        let bombs = mem::read_committed::<i8>(at(BOMBS)).unwrap_or(0);
        // **Which of the two playbacks the game is holding**, and the file it is playing said the frame
        // one starts. Said on its own line rather than keyed into the one below, because what the line
        // below reads of a playback is a run's own numbers and nothing else: 34,200 frames of the demo
        // went by with every line of one launch's log saying `stage 4 on difficulty 1, scene 11`.
        //
        // **The byte itself and not the byte with a stage's scene beside it**, which was tried and says
        // nothing about the demo a launch actually meets: a demo started off the logo screen plays
        // through *that* screen's own state — `scene 1` for the whole of it, with the run's own lives
        // and bombs moving underneath — so a reading that waited for `scene 11` said nothing at all.
        let playback = unsafe { sight::a_replay_is_playing() };
        let demo = playback && unsafe { sight::the_playback_is_the_demo() };
        if PLAYING_BACK.swap(playback, Ordering::Relaxed) != playback {
            match playback {
                true => crate::log!(
                    "run: {} is playing back {}",
                    if demo { "the demo" } else { "a replay" },
                    unsafe { the_replays_path() },
                ),
                false => crate::log!("run: the playback is over"),
            }
        }
        let said = (
            playing.then_some(stage + 1).unwrap_or(0),
            difficulty,
            scene,
            lives,
            bombs,
        );
        let mut last = SAID
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if std::mem::replace(&mut *last, Some(said)) != Some(said) {
            crate::log!(
                "run: stage {} on difficulty {difficulty}, scene {scene}, {lives} live and \
                 {bombs} bomb(s)",
                said.0,
            );
        }
        drop(last);
        State {
            scene,
            wanted: -1,
            playing,
            in_run,
            // **Which is already neither playback**, [`sight::in_a_run`] asking whether one is on: the
            // attract demo runs in the same scene with the same stage as a run, so a reading that only
            // asked the scene had no player to offer a retry to and thought it had one.
            in_game: in_run,
            // **The job an ending runs as being in the chain**, there being nothing else to ask: see
            // [`ENDING_UPDATE`], and `runtime`'s own skip, which runs the chain on until this goes out.
            in_ending: unsafe { the_chain_holds(ENDING_UPDATE) },
            // **And no script of it read**, which is what th06 1.02h's skip stops at the staff roll by:
            // this game's is unread, so the skip runs the roll out with the rest of it.
            ending_script: None,
            demo,
            replay: playback && !demo,
            practice: false,
            paused: false,
            unsettled: false,
            bombing: false,
            in_dialogue: false,
            stage,
            difficulty,
            stage_frames: 0,
            script_frames: 0,
            random_seed: 0,
            deaths: 0,
            lives: mem::read_committed::<i8>(at(LIVES)).unwrap_or(0),
            bombs: mem::read_committed::<i8>(at(BOMBS)).unwrap_or(0),
            power: mem::read_committed::<u16>(at(POWER)).unwrap_or(0),
            enemy_count: mem::read_committed::<i32>(at(ENEMY_MANAGER + enemy_manager::LIVE))
                .unwrap_or(0),
            bullet_count: mem::read_committed::<i32>(at(BULLET_MANAGER + bullet_manager::LIVE))
                .unwrap_or(0),
            laser_count: (0..laser::SLOTS)
                .filter(|slot| {
                    mem::read_committed::<u8>(at(LASERS + slot * laser::BYTES + laser::IN_USE))
                        .is_some_and(|in_use| in_use != 0)
                })
                .count() as i32,
            boss_present: boss.is_some(),
            boss_life: boss.and_then(|body| mem::read_committed::<i32>(body + enemy::LIFE)),
            boss_attack_frames: None,
            spellcard: None,
        }
    }

    /// Unread: the game makes its window through a pointer it looked up, and where it keeps the handle
    /// that call answers is not in `th06c/asm.md`.
    unsafe fn window(&self) -> orb_api::Hwnd {
        orb_api::Hwnd::NULL
    }

    /// Unread: what this game draws through is a swapchain it built, and nothing of orb's is in front of
    /// it — see `th06c/asm.md`, where the device's own creation is the next reading.
    unsafe fn d3d_device(&self) -> orb_api::Device {
        orb_api::Device(0)
    }

    /// **No, and a launch here is a launch with no overlay.** What this game draws through is a D3D11
    /// renderer of its own, which orb has no device of and draws nothing in — so the frame loop opens no
    /// scene of orb's around the draw chain, and the questions orb would put over the game, the lives
    /// row and the picture a `screenshot` step takes have nowhere to be drawn. The frame is still orb's:
    /// see [`Th06c::hooks`], whose `render` is the whole of this game's own.
    fn draws_through_its_d3d_device(&self) -> bool {
        false
    }

    fn music(&self) -> Option<crate::audio::Music> {
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

    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        Vec::new()
    }

    /// Unread, and what asks is the overlay and the panel: neither is drawn here.
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

    /// Unread: this game sizes its own window and orb does not rewrite the ask.
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

    /// **The viewport and the matrix the frame put in place before anything else of it ran**, which is
    /// where orb's own loop has this call too — `0x14002572e`–`0x1400257a5`, in that order.
    ///
    /// **Before the calc chain and not after it**, which is the frame's own order and the reason the loop
    /// calls this where it does: the viewport this sets covers the whole output and what narrows it to the
    /// playfield is the update, so a frame that set it afterwards would draw the field's contents outside
    /// the field.
    ///
    /// The device is nothing here: what this game draws through is a D3D11 renderer of its own and orb
    /// has no device of it — see [`Game::draws_through_its_d3d_device`].
    unsafe fn prepare_frame(&self, _device: orb_api::Device) {
        WALKS_THIS_FRAME.store(0, Ordering::Relaxed);
        unsafe { crate::dxgi::starts_a_frame() };
        let near = mem::read_committed::<f32>(at(THE_VIEWPORTS_NEAR)).unwrap_or(0.0);
        let far = mem::read_committed::<f32>(at(THE_VIEWPORTS_FAR)).unwrap_or(1.0);
        unsafe {
            mem::write::<u64>(at(WHAT_THE_VIEWPORT_WAS_ASKED_FOR), 0);
            mem::write::<u32>(at(THE_VIEWPORTS_WIDTH), THE_OUTPUT.0);
            mem::write::<u32>(at(THE_VIEWPORTS_HEIGHT), THE_OUTPUT.1);
        }
        // The two floats on the stack, which is where the fifth and sixth arguments of a `win64` call
        // go: the frame puts them at `[rsp+0x20]` and `[rsp+0x28]` itself.
        let sets: unsafe extern "win64" fn(u32, u32, u32, u32, f32, f32) =
            unsafe { std::mem::transmute(at(SETS_THE_VIEWPORT)) };
        unsafe { sets(0, 0, THE_OUTPUT.0, THE_OUTPUT.1, near, far) };
        // **Aligned to sixteen, both of them**: the frame copies the matrix with `movaps`, which faults
        // on a buffer that is not — so a matrix of orb's own goes where one of the game's would.
        let mut into = Aligned([0; A_MATRIX * 2]);
        let makes: unsafe extern "win64" fn(*mut u8) -> *const u8 =
            unsafe { std::mem::transmute(at(MAKES_THE_MATRIX)) };
        let made = unsafe { makes(into.0.as_mut_ptr()) };
        if made.is_null() {
            return;
        }
        let mut matrix = Aligned([0; A_MATRIX]);
        unsafe { std::ptr::copy_nonoverlapping(made, matrix.0.as_mut_ptr(), A_MATRIX) };
        let takes: unsafe extern "win64" fn(*const u8) =
            unsafe { std::mem::transmute(at(TAKES_THE_MATRIX)) };
        unsafe { takes(matrix.0.as_ptr()) };
    }

    /// Nothing: this game's frame does nothing of its own between the sound and the draw chain.
    unsafe fn begin_drawing(&self, _device: orb_api::Device) {}

    /// **What the frame did after its draw chain**: the `-1` into the renderer's own field at
    /// `0x14002593b`, and then the two calls its own timing begins with — see
    /// [`FINISHES_THE_PICTURE`], which is the one that puts the frame on the screen.
    ///
    /// **And not the wait that timing ends with, which is the one piece of the frame orb does not
    /// call.** `0x140025aa0` onwards works out how far off its own deadline the frame is, sleeps through
    /// the pointer at `0x140b77810` for as much of it as it can, and then spins on the clock at
    /// `0x140b77800` until the deadline itself — and the frame's timing is `frame::Pacing`'s now. Two
    /// pacings is one too many and this is the one that goes: measured, it was `14.7ms` of a `16.6ms`
    /// frame, so a launch at `--pace=2` ran at sixty a second like every other.
    unsafe fn end_drawing(&self, _device: orb_api::Device) {
        if let Some(renderer) =
            mem::read_committed::<usize>(at(THE_RENDERER)).filter(|renderer| *renderer != 0)
        {
            unsafe { mem::write::<i32>(renderer + WHAT_THE_FRAME_LEAVES_IN_IT, -1) };
        }
        let rate: unsafe extern "win64" fn(usize) =
            unsafe { std::mem::transmute(at(PUTS_THE_RATE_TOGETHER)) };
        unsafe { rate(at(WHAT_IT_PUTS_IT_ON)) };
        let finish: unsafe extern "win64" fn() =
            unsafe { std::mem::transmute(at(FINISHES_THE_PICTURE)) };
        unsafe { finish() };
    }

    /// **Both of them orb's own, for a game whose frame has neither as a function to call.** The sound's
    /// catch-up is inline in the frame orb replaced and the present is inside one of the draw chain's
    /// jobs — see [`tops_up_the_sound`] and [`presents`], which is also where the game's own present is
    /// taken over.
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
        }
    }

    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn make_invulnerable(&self) {}

    /// Whether the stage on the screen is a playback — see [`sight::a_replay_is_playing`], and
    /// [`Self::read_state`] for which of the two kinds it is.
    unsafe fn replaying(&self) -> bool {
        unsafe { sight::a_replay_is_playing() }
    }

    /// **What the entry on the screen a run ends at holds** — see [`result_screen::NAME`].
    ///
    /// **Only while that entry is the state the screen is in**, so that the ranking the *Score* item opens
    /// answers nothing: it is the same object with no run behind it.
    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        let screen = unsafe { the_entry_a_name_is_typed_into() }?;
        crate::game::the_name_held_at(screen + result_screen::NAME, result_screen::NAME_BYTES)
    }

    /// Whether that entry is up at all — see [`the_entry_a_name_is_typed_into`].
    unsafe fn typing_a_name(&self) -> bool {
        unsafe { the_entry_a_name_is_typed_into() }.is_some()
    }

    /// **Into the field that entry's own keys write**, with its count of characters typed at the full eight
    /// and its cursor left on the item that ends it — which is the state the game itself is in once somebody
    /// has typed eight, so the press after it leaves the screen the way theirs would.
    ///
    /// **And it is the rules' own held shot that makes this worth having here.** The entry comes up while the
    /// game manager's state is still a stage's, so the frame it opens on is a frame the rules are being asked
    /// for a word on — and the word they hold is the shot, which writes whichever character the cursor starts
    /// on, eight times over. Every ranking a clear of this game registered was `AAAAAAAA`, read back out of
    /// the replay the same run wrote.
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

    unsafe fn count_card_attempt(&self) -> Option<u16> {
        None
    }

    unsafe fn redraw_card_name(&self) {}

    unsafe fn captures(&self) -> Vec<u8> {
        Vec::new()
    }

    unsafe fn set_captures(&self, _saved: &[u8]) {}

    unsafe fn set_captures_keeping_names(&self, _saved: &[u8]) {}

    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    unsafe fn read_what_the_modes_file_records(&self) {}

    /// Unread: which item this game's title menu is on is a field nothing here has found, so orb asks
    /// nothing over it — its questions are the one thing a launch here could not answer.
    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        None
    }

    unsafe fn menu_takes_a_press(&self) -> bool {
        false
    }

    unsafe fn pad(&self, _pad: Option<Reading>) -> Pad {
        Pad::default()
    }

    unsafe fn pad_word(&self, _pad: Option<Reading>, _hats: bool) -> u16 {
        0
    }

    unsafe fn add_pad_buttons(&self, _into: *mut c_void, _pad: Option<Reading>) {}

    unsafe fn leave_run(&self) -> bool {
        false
    }

    unsafe fn swallow_input(&self) {}

    unsafe fn skip_replay_prompt(&self) -> bool {
        false
    }

    unsafe fn leave_the_screen_a_run_ends_at(&self) -> bool {
        false
    }

    unsafe fn the_screen_a_run_ends_at_is_up(&self) -> bool {
        false
    }

    /// Unread, and what asks is the resume: no frame of a stage is counted here, so a step that waits
    /// for one waits for ever and ends at its own bound.
    unsafe fn stage_frame(&self) -> Option<u32> {
        None
    }

    unsafe fn generator(&self) -> u16 {
        0
    }

    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: -1,
            character: -1,
            shot_type: -1,
            practice: false,
            stage: -1,
        }
    }

    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        None
    }

    fn run_slot(&self, _run: &RunStart) -> Option<String> {
        None
    }

    fn encode_resume(&self, _saved: &crate::resume::Kept) -> Vec<u8> {
        Vec::new()
    }

    fn decode_resume(&self, _bytes: &[u8]) -> Option<crate::resume::Kept> {
        None
    }

    unsafe fn run_state(&self) -> Vec<u8> {
        Vec::new()
    }

    unsafe fn set_run_state(&self, _state: &[u8]) {}

    unsafe fn seed_the_stage(&self, _state: &[u8]) {}

    fn say_run_state(&self, _state: &[u8]) -> String {
        String::new()
    }

    unsafe fn run_chosen(&self) -> bool {
        false
    }

    unsafe fn start_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn start_a_run(&self, _run: &RunStart) -> bool {
        false
    }

    unsafe fn run_finished(&self) -> bool {
        false
    }

    unsafe fn reproduction(&self) -> String {
        String::new()
    }

    unsafe fn mid_build(&self) -> bool {
        false
    }

    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &[]
    }

    fn stages_past_the_numbered(&self) -> &'static [&'static str] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::{Th06c, bullet_manager, button, dialog, key};
    use crate::game::{Game, Read};

    /// **What each of the five bullet states multiplies its own step by**, which is the whole of what the
    /// model does with the state it reads: a bullet moves `[+0x30] += [+0x8] × this`, and a scale that is
    /// wrong makes the movement disagree with the step, which is the test the rules use to tell a bullet a
    /// script is moving from one that is moving itself.
    ///
    /// State 5 is the one a launch corrected — read as a half off `0x1402c9490` and measured as whole,
    /// `moved 3.30,5.38 saying 1.65,2.69` — so it is here beside the four that were right.
    #[test]
    fn each_bullet_state_scales_its_own_step_by_what_the_game_divides_it_by() {
        for (state, scale) in [(1, 1.0), (2, 0.5), (3, 0.4), (4, 1.0 / 3.0), (5, 1.0)] {
            assert_eq!(
                bullet_manager::a_steps_scale(state),
                scale,
                "the step of a bullet in state {state} is scaled by something else",
            );
        }
    }

    /// **What orb stands in front of here is a name and not a place**, which is what makes the read
    /// reachable at all: the slot the game keeps its answer in holds nought until the game's own startup
    /// writes it, so there is nothing at the attach to write over.
    #[test]
    fn the_keyboard_read_is_stood_in_front_of_by_the_name_the_game_asks_for() {
        let Some(Read::LookedUp(lookup)) = Th06c.hooks().input else {
            panic!("th06c's input is not a name the game asks for");
        };
        assert_eq!(lookup.name, "GetKeyboardState");
    }

    /// **And the startup dialog is got past by clicking a window and then the start button**, in that
    /// order: the start button is what makes the dialog procedure read the radio buttons, so a launch
    /// that clicked it first would come up in whatever the dialog opened on.
    #[test]
    fn the_startup_dialog_is_answered_with_a_window_and_then_the_start_button() {
        let Some(answer) = Th06c.hooks().startup_dialog else {
            panic!("th06c puts a startup dialog up and orb answers none");
        };
        assert_eq!(answer.clicks, &[dialog::A_640_BY_480_WINDOW, dialog::START],);
        assert_eq!(
            answer.clicks.last(),
            Some(&dialog::START),
            "the start button is clicked before the window is chosen",
        );
    }

    /// **Every button orb's word has is a key it presses**, which is what stops a recipe naming one that
    /// reaches nothing: the two tables are one table, and this is what says so.
    #[test]
    fn every_button_a_recipe_can_name_is_a_key_this_game_is_played_with() {
        for (name, bit, pressed) in Th06c::BUTTONS {
            assert_eq!(
                Th06c.button_bit(name),
                Some(*bit),
                "{name} is not a button a recipe can name",
            );
            assert_ne!(*pressed, 0, "{name} is pressed with no key at all");
        }
        assert_eq!(
            Th06c.button_bit("nothing of the kind"),
            None,
            "a name this game has no button for answered a bit",
        );
    }

    /// **And a word's keys are the ones it names and no others**, which is what a snapshot orb writes
    /// into has to be: a key nobody pressed is a key the game reads as pressed.
    #[test]
    fn a_word_presses_the_keys_its_own_buttons_are() {
        let mut state = [0u8; 256];
        unsafe { Th06c.press_into(state.as_mut_ptr(), button::DECIDE | button::UP) };
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
