//! th06nc — 「東方紅魔郷: New Classic」, the other Steam release of 東方紅魔郷 — as a [`Game`].
//!
//! **The same engine th06c is built on, and not the same game.** Both exes look their host up by name
//! through one `GetProcAddress`, import the same nine names of `steam_api64.dll`, read the keyboard
//! through one `GetKeyboardState` translated through a table of the same shape, and walk the same two
//! `ChainElem` lists a frame — the host layer matches th06c's byte for byte in places, its keyboard read
//! being a function of the same `0x26f` bytes. What sits on top is another game: the frame is inline in
//! `main` rather than a function, the game manager has thirty-two states where th06c's has twenty-one
//! and leaves the chain for the whole of a run, the bullet is a different size, and a hit is a circle
//! about the player where th06c's is a box. `th06nc/asm.md` is where what has been read off this exe is
//! written down, and nothing of `game::th06c` is read here.
//!
//! **Two seams and no dialog.** The keyboard read is answered by name, and there is no startup dialog to
//! click through: this exe imports no `DialogBoxIndirectParamW`, and what window it makes is
//! `th06.env`'s business — which orb has a word in on the first frame, see
//! [`size_the_window_to_the_settings`].
//!
//! **And the frame is orb's**, through a stub in the game's own loop rather than over a function — see
//! [`FRAME_IN_THE_LOOP`], and [`crate::game::FrameInTheLoop`] for why a frame inline in a loop cannot be
//! stood in the way th06c's is. Off orb's frame a stage is read as well: [`sight`] answers
//! [`Game::what_a_bot_sees`] out of this game's own statics, so the rules play it. What is still not read
//! is what a run is made of past a stage: [`Th06nc::rewinds`] answers false, so there are no chapters, no
//! snapshot to rewind to and no resume.
//!
//! **Neither seam is an address**, for the reason th06c's are not: this exe is built with ASLR on. The
//! addresses below are the reading of a run, and every one of them goes through [`at`], which adds the
//! distance [`Game::set_image_base`] was told.
//!
//! **And every call into this game is `extern "win64"`**, the same four registers th06c's are.
#![doc = include_str!("asm.md")]

use std::ffi::c_void;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use orb_api::mem;

use crate::game::{
    Boundary, Call, FrameCalls, FrameInTheLoop, Game, Hooks, Lookup, Menu, Pad, PanelTile, Patch,
    Read, Reading, Rect, RunStart, State, Walks,
};

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

/// **The DirectInput keyboard the game reads a run with**, which orb takes away — the same shape th06c
/// has: the read at `0x1400ae180` takes the device where there is one, `GetDeviceState` straight into
/// [`THE_KEYS_THE_GAME_READS`] at `0x1400ae258`, and the translation from virtual keys orb's answer feeds
/// is on the other branch, reached where `[0x14055caa4]` or `[0x14055cae4]` is 1 or this is nought.
const DEVICE: usize = 0x1_4055_cad0;

/// **The read the game asks for by name**, resolved with `GetProcAddress` at startup into the slot at
/// `0x14055d408` and called at `0x1400ae2bc` and `0x1400ae315` — see [`Read::LookedUp`].
const KEYBOARD_READ: &str = "GetKeyboardState";

/// **And the name it asks for to make its window**: `CreateWindowExW` is in `.rdata` at `0x1402c28e0`
/// and in no import table.
const WINDOW_MADE: &str = "CreateWindowExW";

/// **And the name it reads a pad with**, which orb answers by being the pad — see
/// [`Game::press_into_pad`]. This exe walks `xinput1_4.dll`, `xinput1_3.dll` and `xinput9_1_0.dll`, the
/// three wide strings at `0x1402c1390` onwards, the same as th06c.
const PAD_READ: &str = "XInputGetState";

/// **And the name it makes its renderer through**, at `0x1402cd9a0` in `.rdata` with the two older
/// names beside it — see [`crate::dxgi`], which takes the swapchain the factory then makes.
const RENDERER_MADE: &str = "CreateDXGIFactory2";

/// **And the name it moves that window with afterwards**, `0x1402c29e0`.
const WINDOW_MOVED: &str = "MoveWindow";

/// **Which XInput button each of orb's own is** — the conventional mapping th06c uses too, see there.
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
}

/// **The bits orb's own word carries for this game**, the same shape th06 1.02h's word has so that a
/// recipe reads the same for every game — see [`Th06nc::press_into`].
///
/// **And this game's own word has the same bits**: the player's own walk at `0x140069072` onwards tests
/// `0x10`, `0x20`, `0x40` and the sign bit for the four directions and `0x4` for the focus, the
/// fast-forward at `0x140045db2` tests `0x8000` of [`INPUT_THE_RUN_ACTS_ON`], and the game manager's
/// own frame writes `0x100` into it at `0x140047a6c` where a screen is to be skipped.
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

/// **The word the game's own read leaves behind, which a run is played with**, and `0x140a6ec64` is the
/// frame before it: the game manager's state 0 clears both at `0x140047ad4` as a run starts, the player
/// reads it at `0x1400689ed` and five more places, and the frame's own fast-forward reads `0x8000` of it
/// at `0x140045db2`.
const INPUT_THE_RUN_ACTS_ON: usize = 0x1_40a6_ec60;

/// **The byte that says everything is held**, which the bullet manager's update tests first at
/// `0x14001087f` and jumps past the whole of itself on, and the player's at `0x14006882d`, the enemies'
/// at `0x14003815c` — seven readers in all, and one writer at `0x14003341d`.
const EVERYTHING_HELD: usize = 0x1_404f_27b9;

/// **The calc chain's head**, a static `ChainElem` the walk at [`RUN_CALC_CHAIN`] starts from and the
/// take-out at `0x1400126f6` searches first; the draw chain's is `0x1404f1400`, walked inline in the draw
/// frame at `0x14003c015` — which is [`DRAWS_THE_FRAME`], called whole, so nothing here walks it.
const THE_CALC_CHAIN: usize = 0x1_404f_1440;

/// The fields of a `ChainElem` a walk reads — the same `0x40` bytes th06c's is, read off the
/// registrations at `0x140067e1e` and `0x140055c5d`.
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

/// **The game manager's frame callback**, which is the way to the manager: the manager is allocated
/// rather than static and **is taken out of the chain as a run starts** — a launch's `chain:` lines went
/// from four jobs with this one among them to three without it, and then to the stage's eleven — so the
/// element `0x140055cb1` keeps at `0x140c07248` points at freed memory for the whole of a run. What
/// holds is the chain itself: the element whose [`chain::CALLBACK`] is this has the manager at its
/// [`chain::ON`]. The wrapper calls `0x1400479e0` once — or up to six hundred times where
/// `[manager + 0x168d0]` says to skip.
const THE_GAME_MANAGERS_CALLBACK: usize = 0x1_4004_7940;

/// **The player's frame callback**, registered at `0x140067e1e`, which is what says a stage is running:
/// the stage's own jobs — this, the enemies' `0x1400373b0`, the bullets' `0x140010870` and eight more —
/// arrive together as the game manager leaves, and go together as it comes back.
const THE_PLAYERS_CALLBACK: usize = 0x1_4006_8820;

/// **The job an ending runs as**, which is what says one is running: the frame the sixth stage's jobs
/// left, one launch's `chain:` line went from the stage's eleven to five — the three that are always
/// there, `0x14006b970` of the stage's, and this one, registered at `0x14002b936`. It reads the input
/// word at `0x14002bb11` the way a screen that is pressed through does, and the game manager is not back
/// while it runs, so [`Game::read_state`] answers `State::in_ending` by asking whether the chain holds it.
const ENDING_UPDATE: usize = 0x1_4002_bac0;

/// **The job the screen a run ends at registers**, which is the one way to that screen: it is built on
/// the heap — `0x14007a218` allocates its `0xbb48` bytes, `0x14006df00` builds it — and `0x14006dd20`
/// puts it at the [`chain::ON`] of the element it registers with this callback, the same way th06c's is.
/// It arrived on the frame the ending's job left, and the game manager came back after it.
const RESULT_SCREEN_UPDATE: usize = 0x1_4006_e3a0;

mod result_screen {
    /// **The screen's own state**, which its update reads at `0x14006e3d8` and jumps through a table of
    /// eighteen at `0x140072824` on.
    pub const STATE: usize = 0x9e94;

    /// **The one of those eighteen the name is typed in.** `0x14007a25e` writes it as a run's own screen
    /// is built — nine where `[0x1404f27b4]` is clear and seventeen where it is not, which is the run that
    /// is asked for no name — and the screen the *Score* item opens is built at `0x14000c699` in twelve.
    pub const TYPING_A_NAME: i32 = 9;

    /// **The field the entry types into**, eight bytes: `0x140071021` writes the character at
    /// `[this + 0x9e70 + index]`, the index being [`TYPED`] capped at seven by the `cmovge` at
    /// `0x140070ffd`. The same shape th06c has at `+0x5173`.
    pub const NAME: usize = 0x9e70;
    pub const NAME_BYTES: usize = 8;

    /// **How many characters have been typed**, which `0x140071033` steps up and `0x14007102e` stops at
    /// eight — and reaching eight is what `0x140071040` puts the cursor on [`THE_END_ITEM`] for.
    pub const TYPED: usize = 0x9e7c;

    /// **Which character of the grid the cursor is on**, the first dword of the screen's own object:
    /// `0x140071000` reads it, `0x140071008` takes the character at `0x14030c250 + it` — the grid of 96,
    /// sixteen to a row, reached off the image base in `r13` — and `0x140071019` writes a space where it
    /// is `0x5e`.
    pub const CURSOR: usize = 0x0;

    /// **The item that ends the entry**, the one past the space at `0x5e`: `0x140071013` leaves the typing
    /// for it, and it is what `0x140071040` puts the cursor on once eight characters have been typed.
    pub const THE_END_ITEM: i32 = 0x5f;
}

/// Whether the ending was running the last time [`Game::read_state`] looked, so that its arrival is one
/// line.
static ENDING: AtomicBool = AtomicBool::new(false);

/// How many frames running a boss has been up for, which paces the line that says how its fight is going.
static FRAMES_ON_A_BOSS: AtomicU32 = AtomicU32::new(0);
/// And the life it last had, for the line that says how it left.
static THE_BOSSES_LAST_LIFE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

mod game_manager {
    /// **Which of thirty-two states the manager is in**: `0x140047a18` reads it and `0x140047ab0` jumps
    /// through the table of thirty-two at `0x14004d0e8`, of which fifteen to twenty and twenty-six share
    /// one case. Which is which screen is `th06nc/asm.md`'s table, read off launches.
    pub const STATE: usize = 0x168b0;
    /// **Which item of the front end the cursor is on**: the difficulty's own state compares it with 0,
    /// 1, 2 and 3 at `0x14004b112`–`0x14004b1d4`, one per difficulty, and lights the item it names. The
    /// same offset th06c keeps its cursor at.
    ///
    /// **And not `+0x30`**, which every state writes nought into as it comes up and which a launch read
    /// climbing once a frame — a counter of the state's own frames, and what the difficulty state hands
    /// `0x1400478e0` to blink the lit item with.
    pub const CURSOR: usize = 0x28;
}

/// **The frame, which is a stretch of `main`'s loop and not a function** — see `th06nc/asm.md`, which
/// has the loop `0x140045ce0`–`0x140045ea5` piece by piece. What orb stands in is the fourteen bytes at
/// `0x140045d7d`: `lea rcx,[rsp+0x70]; call 0x14003be80; test al,al; jne 0x140045de9` — the calc walk's
/// call and the test of what it answered — and the stub at [`th06nc_frame_in_the_loop`] stands there.
///
/// **Where the loop is rejoined is one of two places.** A frame that carries on goes to `0x140045e9e`,
/// which is the message pump the game's own loop ends with, the draw frame and the timing before it
/// having been orb's own calls; a frame that leaves goes to `0x140045eab`, which is where the game's
/// own `jne` sends a walk that answered non-zero and left a non-zero answer through its pointer — and
/// every non-zero answer of that walk leaves one, `0x14003bf21` and `0x14003bf0b` writing 1 and 2.
const FRAME_IN_THE_LOOP: usize = 0x1_4004_5d7d;
const FRAME_IN_THE_LOOPS_BYTES: &[u8] = &[
    0x48, 0x8d, 0x4c, 0x24, 0x70, 0xe8, 0xf9, 0x60, 0xff, 0xff, 0x84, 0xc0, 0x75, 0x5e,
];
const WHERE_THE_LOOP_CARRIES_ON: usize = 0x1_4004_5e9e;
const WHERE_THE_LOOP_LEAVES: usize = 0x1_4004_5eab;

/// **Those two places where the exe actually is**, which the stub jumps through: written by
/// [`Game::set_image_base`], the stub having no [`at`] of its own.
static LOOP_CARRIES_ON_AT: AtomicUsize = AtomicUsize::new(0);
static LOOP_LEAVES_AT: AtomicUsize = AtomicUsize::new(0);

// **The stub the jump at [`FRAME_IN_THE_LOOP`] lands in.** It is entered by a jump from the middle of
// `main` with `main`'s own frame live and `rsp` a multiple of sixteen — `main` pushes three registers
// and takes `0x120`, and `movaps [rsp+0x100]` at `0x140045b90` says so — so the shadow space is all a
// call wants, and `runtime::render` being `win64` keeps every register `main` holds in the callee-saved
// ones. What `render` answers decides which of the two places above the loop is rejoined at: nought is
// a frame that carries on, anything else is one that leaves. Hand-written because a Rust function has a
// prologue and a `ret`, and neither is wanted where nothing pushed a return address.
core::arch::global_asm!(
    ".globl th06nc_frame_in_the_loop",
    "th06nc_frame_in_the_loop:",
    "sub rsp, 0x20",
    "xor ecx, ecx",
    "call {render}",
    "add rsp, 0x20",
    "test eax, eax",
    "jne 2f",
    "jmp qword ptr [rip + {carries_on}]",
    "2:",
    "jmp qword ptr [rip + {leaves}]",
    render = sym crate::runtime::render,
    carries_on = sym LOOP_CARRIES_ON_AT,
    leaves = sym LOOP_LEAVES_AT,
);

unsafe extern "C" {
    fn th06nc_frame_in_the_loop();
}

/// **The walk of the calc chain, which is this game's tick** — `0x14003be80`, `rcx` a pointer the answer
/// the frame leaves with is written through, `al` non-zero to leave. It flushes the frame's queued sound
/// effects itself through `0x140076a30` before it answers — see `th06nc/asm.md`.
const RUN_CALC_CHAIN: usize = 0x1_4003_be80;

/// **The draw frame**, `0x14003bf70`: the viewport, the draw chain walked inline, and the screenshot
/// path behind it — called whole where th06c's loop calls the walk it wrote itself, this one taking no
/// argument and answering nothing the loop reads.
const DRAWS_THE_FRAME: usize = 0x1_4003_bf70;

/// **What the game does after the draw frame**, `0x14003c330`: the bars either side of a viewport that
/// does not fill the window, the picture finished through `0x1400953e0`, and then the frame limiter and
/// the fps line. See [`Th06nc::end_drawing`] for how the limiter is got past.
const ENDS_THE_FRAME: usize = 0x1_4003_c330;
/// **The game's own object**, `0x140c21970`, which the screen-mode functions and the config writers are
/// called on: `th06.cfg`'s 44 bytes are at its `+0x434` and `th06.env`'s twelve at its `+0x48c`.
const THE_GAME: usize = 0x1_40c2_1970;

/// **`th06.env`, twelve bytes read into `0x140c21dfc` at startup by `0x14007b5d0` and written back at
/// exit** — see `th06nc/asm.md`. The two of them that are the window.
mod env {
    /// **Which screen mode the game is in**: `main` reads it at `0x140045a88` and calls one of three
    /// functions on it — nought [`super::MAKES_A_WINDOW`], one [`super::COVERS_THE_MONITOR`], two
    /// `0x14007c060`, the fullscreen — and `0x140045310` flips it between nought and one on a key.
    pub const SCREEN_MODE: usize = 0x1_40c2_1e00;
    /// The mode that is a window of one of [`WINDOW_SIZES`].
    pub const A_WINDOW: u8 = 0;
    /// The mode that is a window covering the monitor, which is what `Screen::Fullscreen` means.
    pub const THE_MONITOR: u8 = 1;

    /// **Which window the window mode makes**, read at `0x14007c62b` as `[game + 0x491]`: nought is the
    /// largest of the four that fits the monitor, and one to four are the four, largest first. The
    /// option screen steps it at `0x14004e37a`, and the mode function writes back the one it settled on
    /// at `0x14007c6aa`. The two bytes after it are the same choice for the other two modes, where it caps
    /// the picture and not the window.
    pub const WINDOW_SIZE: usize = 0x1_40c2_1e01;

    /// **The four windows the window mode offers**, by the item that names each — `0x14007c680`,
    /// `0x14007c674`, `0x14007c68c` and `0x14007c66a`.
    pub const WINDOW_SIZES: [(u8, u32, u32); 4] = [
        (1, 3840, 2160),
        (2, 2560, 1440),
        (3, 1920, 1080),
        (4, 1280, 720),
    ];
}

/// **The function the window mode is entered by**, `0x14007c560`, called on [`THE_GAME`]: it reads
/// [`env::WINDOW_SIZE`], sizes the window and the picture to that item through `0x140096cd0` at
/// `0x14007c71c`, and works with the window already made — the option screen calls it on the running
/// game at `0x14004e74a`, which is how orb calls it too.
const MAKES_A_WINDOW: usize = 0x1_4007_c560;

/// **And the function mode one is entered by**, `0x14007c320`, which sizes the window to the monitor it
/// is on — `0x140096cd0` at `0x14007c497` takes the monitor's own size — whatever the item says: the
/// item caps the picture there. A launch with the env at this mode came up the whole of a 4K monitor,
/// `3862x2216` with its frame, which is the window that was too large.
const COVERS_THE_MONITOR: usize = 0x1_4007_c320;

/// **The function the picture is sized by**, `0x140096cd0`, `(width, height, bits, hertz)`: it keeps the
/// four at `0x14050a198` onwards and remakes the renderer at that size through `0x1400f0d20`, or
/// `0x140091fc0` where one is up already. The mode functions call it with the item's size, or the
/// monitor's; called with a size of orb's own it takes that — a size of nought is `640x480` there, and
/// bits other than 8, 16 or 32 are refused at `0x140096d37`.
const SIZES_THE_PICTURE: usize = 0x1_4009_6cd0;
const THE_PICTURES_BITS: u32 = 32;
const THE_PICTURES_HERTZ: u32 = 60;

/// **Where in the window the picture is drawn**, four dwords — left, top, width, height — which the mode
/// functions write at `0x14007c518`, `0x14007c79e` and `0x14007c2d2` out of the size they chose, letterboxed
/// to 16:9, and the draw reads back at `0x14003c374` and forty more places. Written by orb where it sizes the
/// picture itself, since the mode function that wrote it last was told another size.
const THE_PICTURE: usize = 0x1_40c2_1e3c;
/// **And whether that picture is taller than 1080**, the byte `seta` writes beside it at `0x14007c78f`,
/// which the draw tests in fifty places to pick its larger assets.
const THE_PICTURE_IS_TALL: usize = 0x1_40c2_1d14;

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
/// the two: `main` reads `th06.env` at `0x14004586f`, chooses the size at `0x140045a76` and makes the window
/// at `0x140045b74` with no call orb answers in between, and a byte written before the read is read over.
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
                MAKES_A_WINDOW,
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
            COVERS_THE_MONITOR,
            "the settings ask for fullscreen, so the window covers the monitor".to_owned(),
        ),
    };
    unsafe { mem::write::<u8>(at(env::SCREEN_MODE), mode) };
    let enters: unsafe extern "win64" fn(usize) = unsafe { std::mem::transmute(at(entered)) };
    unsafe { enters(at(THE_GAME)) };
    if out_of_the_way {
        let (width, height) = OUT_OF_THE_WAY;
        let sizes: unsafe extern "win64" fn(u32, u32, u32, u32) =
            unsafe { std::mem::transmute(at(SIZES_THE_PICTURE)) };
        unsafe {
            sizes(width, height, THE_PICTURES_BITS, THE_PICTURES_HERTZ);
            mem::write::<[i32; 4]>(at(THE_PICTURE), [0, 0, width as i32, height as i32]);
            mem::write::<u8>(at(THE_PICTURE_IS_TALL), 0);
            crate::window::keeps_its_frame();
        }
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
/// `0x14003c4ad` starts both over — base to now, count to nought — where now is already more than a
/// period past the deadline.
const THE_LIMITERS_BASE: usize = 0x1_40c2_2128;
const THE_LIMITERS_COUNT: usize = 0x1_40c2_2130;

/// **The sound's own catch-up, which the loop does between the calc walk and the draw** — `0x140045dfa`
/// onwards. What it is for is a run that has gone by faster than the music: the loop counts the extra
/// walks of the calc chain it made into [`THE_EXTRA_WALKS`] and this is where that count is turned into a
/// seek and cleared.
const THE_EXTRA_WALKS: usize = 0x1_4050_9690;
/// Which track is playing, and `-1` for none: the seek's second argument.
const THE_TRACK_PLAYING: usize = 0x1_4050_966c;
/// Whether that track is playing at all, and whether it loops — the two bytes the catch-up reads before
/// it decides which bound to wrap the seek inside.
const THE_TRACK_IS_PLAYING: usize = 0x1_4050_967c;
const THE_TRACK_LOOPS: usize = 0x1_4050_967d;
/// Where that loop starts and ends, in the same samples the seek is in.
const THE_LOOP_STARTS: usize = 0x1_4050_9680;
const THE_LOOP_ENDS: usize = 0x1_4050_9688;
const WHERE_THE_MUSIC_IS: usize = 0x1_400c_4a10;
const HOW_LONG_THE_TRACK_IS: usize = 0x1_400c_4f90;
const SEEKS_THE_MUSIC: usize = 0x1_400c_9930;
/// **How many samples a walk is**, which the loop multiplies the count by outright — `imul rbx,rcx,0x320`
/// at `0x140045e33` — 48,000 a second in sixtieths, where th06c asks the sound for its rate.
const SAMPLES_A_WALK: i64 = 0x320;

/// **The run's own numbers, each a static of its own** — `th06nc/asm.md` says what each was read off.
///
/// The lives and the bombs are the pair `0x14007a126` writes `0x302` over as an Extra run starts and
/// that the item walk stops at eight (`0x140043407`, `0x1400433c7`); a Normal run's start writes the two
/// bytes at `0x140c21db4` and `0x140c21db5` over them instead, which is a starting stock this game lets
/// somebody choose. Measured: a launch's run started with `2 live and 3 bomb(s)`.
const LIVES: usize = 0x1_404f_f0f0;
const BOMBS: usize = 0x1_404f_f0f1;
/// A word, the item walk's power cases writing it back at `0x140042df1` and `0x1400431db`, and `0x80`
/// being the most the full-power item writes it to at `0x1400435a1`.
const POWER: usize = 0x1_404f_1e88;
const POWER_FULL: u16 = 128;
/// **Counted from one**: `0x14003cdcc` multiplies it by a thousand into the `Stage * 1000 =  %5d` line,
/// a launch's first stage read `stage 1`, and the Extra read `stage 7` — `0x14007a11c` writes six into it
/// as the Extra starts and the stage's own start then counts it on.
const STAGE: usize = 0x1_404f_1e84;
/// Compared against four — the Extra — at `0x14007a117` beside the stage it then writes; a launch that
/// took the second item of the difficulty played on `1`, and the Extra on `4`.
const DIFFICULTY: usize = 0x1_404f_27c0;
/// **Whether the stage on the screen is a replay being played back**: the frame's own fast-forward at
/// `0x140045d97` runs only where it is set, `0x14004d753` sets it and `0x14004dc35` clears it in the
/// same code that writes the lives and the bombs a run starts with.
const A_REPLAY_IS_PLAYING: usize = 0x1_404f_278c;
/// **And whether that playback is the attract demo**, which the same fast-forward refuses at
/// `0x140045da0`.
const THE_PLAYBACK_IS_THE_DEMO: usize = 0x1_404f_27b0;

/// **The player**, a static of `0x7854` bytes `0x140067dd8` zeroes, with `0x140506bfc`'s `0x2a5c` after
/// it, which its own hit test at `0x14006a980` is handed as `rcx` — `th06nc/asm.md` has the reading.
const PLAYER: usize = 0x1_404f_f3a0;

mod player {
    /// **Where the player is**, two floats the move at `0x1400693b0` writes and the laser test reads as
    /// the statics `0x140506ad0` and `0x140506ad4`.
    pub const POSITION: usize = 0x7730;
    /// **The radius a hit is measured against**: `0x14006aa35` adds it to a bullet's smaller half and
    /// squares the sum against the distance, and `0x14006aa56` squares it alone against the nearest
    /// point of an enemy's box.
    pub const RADIUS: usize = 0x774c;
    /// **What one frame of holding a direction moves the player by**, four floats the walk at
    /// `0x140069136` onwards picks between by the direction and the focus bit.
    pub const SPEED_STRAIGHT: usize = 0x7860;
    pub const SPEED_STRAIGHT_FOCUSED: usize = 0x7864;
    pub const SPEED_DIAGONAL: usize = 0x7868;
    pub const SPEED_DIAGONAL_FOCUSED: usize = 0x786c;
    /// And what that is scaled by on each axis before it reaches the position — `mulss` at
    /// `0x140069388` and `0x140069398`.
    pub const SCALE_ACROSS: usize = 0x7710;
    pub const SCALE_DOWN: usize = 0x7714;
    /// **Which state the player is in**, one byte: nought is the only one a hit lands in
    /// (`0x14006aaa5`), 2 is what the hit writes (`0x14006aaf1`), 3 is what `0x140068f95` writes as the
    /// invulnerability begins.
    pub const STATE: usize = 0x7898;
    pub const NORMAL: u8 = 0;
    pub const HIT: u8 = 2;
    /// **The frames a bomb still takes the hit back in**: the hit writes 8 into it at `0x14006aafe`,
    /// and the laser's own hit the same at `0x14006ad4b`.
    pub const A_BOMB_STILL_TAKES_IT_BACK: usize = 0x773c;
}

/// **The rectangle the player is held to**, left, top, width and height — `0x1400693c0`–`0x14006942b`
/// puts the position back inside it. Read the way th06c's is, with [`FIELD_SIZE`] behind it where it
/// reads as nought.
const PLAYER_AREA: usize = 0x1_404f_f0e0;
/// **The field's own size**, which this exe holds as the literals `384` and `448` at `0x14030d008` and
/// `0x14030d014` — loaded by the bullets' update at `0x1400113e3` and the enemies' at `0x14003741f`.
const FIELD_SIZE: (f32, f32) = (384.0, 448.0);

/// **The bullets**, a static of `0xf5010` bytes `0x14003b1cf` zeroes, whose update is `0x140010870` and
/// whose array that update's own loop says the shape of — see `th06nc/asm.md`.
const BULLET_MANAGER: usize = 0x1_403e_c2a0;

mod bullet_manager {
    pub const BULLETS: usize = 0x8;
    /// `cmp r14d,0x280` at `0x140011599`.
    pub const SLOTS: usize = 0x280;
    /// `add rbx,0x620` at `0x140011578`.
    pub const BYTES: usize = 0x620;
    /// Counted up per live one at `0x14001097b`.
    pub const LIVE: usize = 0xf5008;
    /// Three floats, `0x140010a61` onwards adding them into the position.
    pub const STEP: usize = 0x08;
    pub const POSITION: usize = 0x30;
    /// **Which of five states it is in**, a word — `cmp WORD PTR [rbx+0x44],r13w` at `0x140010970`, and
    /// nought is a slot nothing is in.
    pub const STATE: usize = 0x44;
    /// **And the only one the player is tested against**: the test at `0x140011418` is on state 1's path
    /// and on no other.
    pub const FLYING: u16 = 1;
    /// **The size the game's own hit test is given**, `lea r8,[rbx+0x5f4]` at `0x140011402`, whose
    /// smaller float is the circle the test takes.
    pub const SIZE: usize = 0x5f4;

    /// **What a bullet's own step is scaled by before it reaches its position**: states 2, 3 and 4 divide
    /// one by `2.0`, `2.5` and `3.0` — `0x140010a08`, `0x140010a1e`, `0x140010a2f`, the floats at
    /// `0x14030cdd8`, `0x14030cdf0` and `0x14030ce10` — and states 1 and 5 add the step whole. The same
    /// five th06c has.
    pub const fn a_steps_scale(state: u16) -> f32 {
        match state {
            2 => 1.0 / 2.0,
            3 => 1.0 / 2.5,
            4 => 1.0 / 3.0,
            _ => 1.0,
        }
    }
}

/// **The lasers**, in that same static right after the bullets' live count: the update walks sixty-four
/// of them from `0x1400115ac`, `0x298` apart, holding its pointer `0x14` into each — so the array starts
/// `0x10` past the count, the way th06c's does.
const LASERS: usize = BULLET_MANAGER + bullet_manager::LIVE + 0x10;

mod laser {
    /// `mov edi,0x40` at `0x1400115b9`.
    pub const SLOTS: usize = 0x40;
    /// `add rbx,0x298` at `0x14001172f`.
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
    /// `0x14001182f` before it moves the phase on.
    pub const WARNS_FOR: usize = 0x270;
    /// Nought is the warning, 1 fires — the one phase whose branch calls the test, at `0x140011883` —
    /// and 2 fades.
    pub const FIRING: u8 = 1;
    pub const FADING: u8 = 2;
    /// **What a hit on it is, to either side of its own line**: a quarter of the width, the update
    /// halving it at `0x14001167a` into the size the test is handed and the test halving that again
    /// with `0x14030cd58` at `0x14006ac5a` — the two halvings th06c's has.
    pub const HALF_OF_ITS_WIDTH: f32 = 4.0;
}

/// **The enemies**, the static `0x1400371c9` hands the element at `0x1404f1540` as what
/// `0x1400373b0` runs on.
const ENEMY_MANAGER: usize = 0x1_40aa_1e90;

mod enemy_manager {
    /// `lea rbx,[r12+0x8]` at `0x14003741a`.
    pub const ENEMIES: usize = 0x8;
    /// `cmp r13d,0x100` at `0x14003819a`.
    pub const SLOTS: usize = 0x100;
    /// `add rbx,0x10b0` at `0x14003818e`.
    pub const BYTES: usize = 0x10b0;
    /// Counted up per live one at `0x1400374ad`, into the manager's first dword.
    pub const LIVE: usize = 0x0;
}

mod enemy {
    /// `lea rsi,[rbx+0xb0]` at `0x1400374b1`, handed to the player's test as the position.
    pub const POSITION: usize = 0xb0;
    /// **The size the player's own test is given**, three floats multiplied by two thirds at
    /// `0x140037a43` — `0x14030cd74` — and halved by the test, so a hit is a third of this on each side.
    pub const SIZE: usize = 0x2c0;
    pub const A_THIRD_OF_ITS_SIZE: f32 = 3.0;
    /// The flags byte the walk gates on: `cmp BYTE PTR [rbx+0xbc],0; jge` at `0x1400374a0` skips a slot
    /// whose top bit is clear.
    pub const FLAGS: usize = 0xbc;
    pub const IN_USE: u8 = 0x80;
    /// **And whether its body is run against the player at all**: `0x1400379f2` requires `0x4`,
    /// `0x1400379fa` refuses where `+0xbe`'s `0x8` is set, and `0x140037a18` runs the test only where
    /// the low two bits are both up.
    pub const SOLIDITY: usize = 0xbd;
    pub const TESTED: u8 = 0x4;
    pub const SOLID_BITS: u8 = 0x3;
    /// **And which one the fight is with**: `0x140037d90` and `0x140037fd0` test this bit and divide the
    /// life below into the bar the panel draws for the enemy carrying it.
    pub const IS_THE_BOSS: u8 = 0x8;
    /// Its life, which the body taking the player off it drops by ten at `0x140037a83`.
    pub const LIFE: usize = 0x234;
    pub const MORE_FLAGS: usize = 0xbe;
    pub const PASSES_THROUGH: u8 = 0x8;
}

/// **The items**, a static walked by `0x140042980` with the array at `+8` — `lea r14,[0x140baf0d8]` at
/// `0x1400429a4`.
const ITEM_MANAGER: usize = 0x1_40ba_f0d0;

mod item {
    pub const ITEMS: usize = 0x8;
    /// `cmp eax,0x200` at `0x14004351d`.
    pub const SLOTS: usize = 0x200;
    /// `imul rdi,rcx,0x160` at `0x140011442`, where the bullets' update reaches into this array.
    pub const BYTES: usize = 0x160;
    /// A byte, and nought is a slot nothing is in — `cmp BYTE PTR [r14],bpl` at `0x140042ae0`.
    pub const ALIVE: usize = 0x0;
    pub const POSITION: usize = 0x10;
    /// **Which of seven kinds it is**, a signed byte `0x140042d40` reads and jumps through the table at
    /// `0x1400438a8` on. The cases are th06c's in th06c's order: the power written at `0x140042d5a`,
    /// the bombs stopped at eight at `0x1400433c7`, the lives at `0x140043407`, the power written to
    /// its cap at `0x1400435a1`.
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
const THE_KEYS_THE_GAME_READS: usize = 0x1_4054_343c;
const DOWN_IN_ITS_OWN_KEYS: u8 = 0x80;
const UP_IN_ITS_OWN_KEYS: u8 = 0x00;
/// **The game's own table from a virtual key to the index it keeps that key at**: six bytes an entry,
/// walked at `0x1400ae350` and ending at a key of `0xffff`. `A` is `0x41 -> 0x1e`, which is `DIK_A`,
/// so the buffer is indexed by DirectInput's own scan codes — the same table th06c has at `0x1402872e0`.
const THE_TABLE_OF_KEYS: usize = 0x1_402c_1200;
const A_KEY_IN_THE_TABLE: usize = 6;
const KEYS_IN_THE_TABLE: usize = 0x82;
const NO_MORE_KEYS: u16 = 0xffff;

/// What a key that is down reads as in a `GetKeyboardState` snapshot: the high bit of the byte.
const DOWN: u8 = 0x80;

/// The pad's own packet number, which moves on every state orb hands over.
static PACKETS: AtomicU32 = AtomicU32::new(1);

/// What the log's own `run:` line carries: the stage, the difficulty, the scene, the lives and the bombs.
type Said = (i32, i32, i32, i8, i8);

/// What that line last said, so that [`Game::read_state`] says it once each time any of it changes.
static SAID: std::sync::Mutex<Option<Said>> = std::sync::Mutex::new(None);

/// Whether [`A_REPLAY_IS_PLAYING`] was set the last time [`Game::read_state`] looked.
static PLAYING_BACK: AtomicBool = AtomicBool::new(false);

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

/// **The game manager**, or nothing while it is not in the chain — see
/// [`THE_GAME_MANAGERS_CALLBACK`], which has why it is found by its job and not by a pointer.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe fn the_game_manager() -> Option<usize> {
    unsafe { what_a_job_runs_on(THE_GAME_MANAGERS_CALLBACK) }
}

/// **The object a job of the calc chain is run on**, found by the callback it registered — every step
/// asked of the memory map rather than read outright, a chain being a list the game is free to be
/// halfway through rebuilding.
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

/// **Whether the calc chain holds a job of its own** — apart from [`what_a_job_runs_on`] because this one
/// is about the job being there at all.
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

/// The game manager's own state, where there is a manager.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_scene() -> Option<i32> {
    let manager = unsafe { the_game_manager() }?;
    mem::read_committed::<i32>(manager + game_manager::STATE)
}

/// **Every job the calc chain holds, said the frame the set of them changes** — the same instrument
/// th06c's screens were found with, since a screen this game keeps on the heap is reached through the
/// job that registers it.
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
    let held = mem::read_committed::<u8>(at(EVERYTHING_HELD)).unwrap_or(0) != 0;
    let running = unsafe { in_a_run() };
    running && !held
}

/// **Whether there is a run at all**, and a playback is none of it — see th06c's `sight::in_a_run`,
/// whose reasoning this shares.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn in_a_run() -> bool {
    unsafe { a_stage_is_the_scene() && !a_replay_is_playing() }
}

/// **Whether a stage is what the game is showing**: the player's job is in the calc chain and the stage
/// the run has reached is one of the seven — see [`THE_PLAYERS_CALLBACK`], and `th06nc/asm.md` for the
/// launch that said a stage is not a state of the game manager's here but the manager's own absence.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_stage_is_the_scene() -> bool {
    let stage = mem::read_committed::<i32>(at(STAGE)).unwrap_or(0);
    (1..=STAGES).contains(&stage) && unsafe { the_chain_holds(THE_PLAYERS_CALLBACK) }
}

/// The stages a run of this game has, the Extra counted: `0x14003cd89` treats six as the last of the
/// numbered ones, and a launch of the Extra read `stage 7 on difficulty 4` for the whole of it — the same
/// seventh th06c's is.
const STAGES: i32 = 7;

/// # Safety
/// Must run on the game's main thread.
unsafe fn a_replay_is_playing() -> bool {
    mem::read_committed::<u8>(at(A_REPLAY_IS_PLAYING)).unwrap_or(0) != 0
}

/// # Safety
/// Must run on the game's main thread.
unsafe fn the_playback_is_the_demo() -> bool {
    mem::read_committed::<u8>(at(THE_PLAYBACK_IS_THE_DEMO)).unwrap_or(0) != 0
}

/// **The calc chain walked, and its answer turned into one the frame loop reads.**
///
/// The walk answers a byte — nought to carry the frame on and anything else to leave it — the same as
/// th06c's, and writes what it leaves with through the pointer it is handed.
///
/// **And the word put into the game's own keys here**, this being the last moment before the walk that
/// reads them — see [`Game::presses_into_its_own_keys`].
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
    if let Some(word) = crate::runtime::pressing() {
        unsafe { Th06nc.presses_into_its_own_keys(word) };
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

/// **The draw frame called whole** — see [`DRAWS_THE_FRAME`]. Nothing it does leaves the frame, that
/// being the calc chain's business, so this always answers the loop's carry on.
///
/// # Safety
/// Must run on the game's main thread, with the exe's base already told.
unsafe extern "win64" fn draws_the_frame(_: *mut c_void) -> i32 {
    let draw: unsafe extern "win64" fn() = unsafe { std::mem::transmute(at(DRAWS_THE_FRAME)) };
    unsafe { draw() };
    CARRY_THE_FRAME_ON
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
        unsafe { std::mem::transmute(at(WHERE_THE_MUSIC_IS)) };
    let playing = unsafe { playing() };
    if playing < 0 {
        return;
    }
    let mut seek = SAMPLES_A_WALK * walks + playing;
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

impl Game for Th06nc {
    /// Kept as the distance from [`IMAGE_BASE`] — see [`at`] — and the two places the frame's stub
    /// rejoins the loop at written out where the exe is, the stub having no [`at`] of its own.
    fn set_image_base(&self, base: usize) {
        SLIDE.store(base.wrapping_sub(IMAGE_BASE), Ordering::Relaxed);
        LOOP_CARRIES_ON_AT.store(at(WHERE_THE_LOOP_CARRIES_ON), Ordering::Relaxed);
        LOOP_LEAVES_AT.store(at(WHERE_THE_LOOP_LEAVES), Ordering::Relaxed);
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
    /// answers.** The frame is a stretch of the loop and the stub stands in it — see
    /// [`FRAME_IN_THE_LOOP`] — and the walks are handed over the way th06c's are, nothing of the game's
    /// calling either once the frame is orb's.
    fn hooks(&self) -> Hooks {
        Hooks {
            update: None,
            draw: None,
            walks: Some(Walks {
                update: walks_the_calc_chain as *const () as usize,
                draw: draws_the_frame as *const () as usize,
            }),
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
            // **None, because this game's frame is no function**: what stands in it is the stub below.
            render: None,
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
            reads_a_pad: Some(Lookup { name: PAD_READ }),
            makes_its_renderer: Some(Lookup {
                name: RENDERER_MADE,
            }),
            frame_in_the_loop: Some(FrameInTheLoop {
                patch: Patch {
                    target: at(FRAME_IN_THE_LOOP),
                    prologue: FRAME_IN_THE_LOOPS_BYTES,
                },
                stub: th06nc_frame_in_the_loop as *const () as usize,
            }),
        }
    }

    /// **No**: what a chapter needs is a snapshot of the address space a run is in, and which of this
    /// game's memory that is has not been read.
    fn rewinds(&self) -> bool {
        false
    }

    /// **A word's buttons put into the snapshot the game is about to read** — see
    /// [`crate::runtime::keyboard_state`], and th06c's, which this is the same as.
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

    /// **A word's keys written straight into the buffer this game plays a run off**, through the
    /// game's own table — see [`THE_TABLE_OF_KEYS`] — and the keys the word does not hold written up,
    /// for the reason th06c's are.
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

    /// **A word's buttons put into the pad state orb answers this game's own read with** — the d-pad
    /// and the stick both, at full deflection, the way th06c's are.
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
    /// [`game_manager::CURSOR`].
    unsafe fn front_end_cursor(&self) -> Option<i32> {
        if unsafe { in_a_stage() } {
            return None;
        }
        let manager = unsafe { the_game_manager() }?;
        mem::read_committed::<i32>(manager + game_manager::CURSOR)
    }

    /// **And which screen that is**, by the game manager's own number for it — see
    /// [`game_manager::STATE`] — and nothing while a stage runs, the manager being out of the chain.
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
    /// numbered at all. The scene is the game manager's state while it is in the chain and `-1` while a
    /// stage runs, the manager being out of it then.
    unsafe fn read_state(&self) -> State {
        unsafe { say_the_jobs_when_they_change() };
        let scene = unsafe { the_scene() }.unwrap_or(-1);
        let stage = mem::read_committed::<i32>(at(STAGE)).unwrap_or(0);
        let playing = unsafe { in_a_stage() };
        let in_run = unsafe { in_a_run() };
        let boss = unsafe { sight::the_boss_the_fights_own_slot() };
        let difficulty = mem::read_committed::<i32>(at(DIFFICULTY)).unwrap_or(-1);
        let lives = mem::read_committed::<i8>(at(LIVES)).unwrap_or(0);
        let bombs = mem::read_committed::<i8>(at(BOMBS)).unwrap_or(0);
        let playback = unsafe { a_replay_is_playing() };
        let demo = playback && unsafe { the_playback_is_the_demo() };
        if PLAYING_BACK.swap(playback, Ordering::Relaxed) != playback {
            match playback {
                true => crate::log!(
                    "run: {} is playing back",
                    if demo { "the demo" } else { "a replay" },
                ),
                false => crate::log!("run: the playback is over"),
            }
        }
        // **And how far a fight has got, ten times a minute while a boss is up.** The frame lines say a boss
        // is up and nothing of its life, and how far into the Extra's fight a run was lost is the one thing
        // a launch of it is read for — twelve minutes of `boss=true` with three deaths in them say nothing
        // about which attack took them. Every 360 frames, so that a fight is a dozen lines and not a page.
        let boss_life = boss.and_then(|body| mem::read_committed::<i32>(body + enemy::LIFE));
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
            wanted: -1,
            playing,
            in_run,
            in_game: in_run,
            in_ending,
            // **And no script of it read**, which is what th06 1.02h's skip stops at the staff roll by:
            // this game's is unread, so the skip runs the roll out with the rest of it, as th06c's does.
            ending_script: None,
            demo,
            replay: playback && !demo,
            practice: false,
            paused: false,
            unsettled: false,
            bombing: false,
            in_dialogue: false,
            // **The game's own number less one**, this game counting stages from one where `State::stage`
            // is counted from nought the way orb counts them everywhere — see `RunStart::stage`.
            stage: stage - 1,
            difficulty,
            stage_frames: 0,
            script_frames: 0,
            random_seed: 0,
            deaths: 0,
            lives,
            bombs,
            power: mem::read_committed::<u16>(at(POWER)).unwrap_or(0),
            enemy_count: mem::read_committed::<i32>(at(ENEMY_MANAGER + enemy_manager::LIVE))
                .unwrap_or(0),
            bullet_count: mem::read_committed::<i32>(at(BULLET_MANAGER + bullet_manager::LIVE))
                .unwrap_or(0),
            laser_count: (0..laser::SLOTS)
                .filter(|slot| {
                    mem::read_committed::<u8>(at(LASERS) + slot * laser::BYTES + laser::IN_USE)
                        .is_some_and(|in_use| in_use != 0)
                })
                .count() as i32,
            boss_present: boss.is_some(),
            boss_life: boss.and_then(|body| mem::read_committed::<i32>(body + enemy::LIFE)),
            boss_attack_frames: None,
            spellcard: None,
        }
    }

    unsafe fn window(&self) -> orb_api::Hwnd {
        orb_api::Hwnd::NULL
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

    /// **Nothing of the game's**, the viewport being the draw frame's own business here — see
    /// [`DRAWS_THE_FRAME`]: only the count of walks cleared and the frame's own present told it is
    /// beginning.
    /// And the window sized to the settings on the first of them — see
    /// [`size_the_window_to_the_settings`].
    unsafe fn prepare_frame(&self, _device: orb_api::Device) {
        WALKS_THIS_FRAME.store(0, Ordering::Relaxed);
        unsafe { size_the_window_to_the_settings() };
        unsafe { crate::dxgi::starts_a_frame() };
    }

    unsafe fn begin_drawing(&self, _device: orb_api::Device) {}

    /// **What the loop did after the draw frame**, called whole with its limiter got past — see
    /// [`ENDS_THE_FRAME`].
    ///
    /// **Got past by starting it over each frame rather than by cutting it out**: the bars, the picture's
    /// finish and the wait are one function with nothing between them to land in, so the wait is left
    /// in and given nothing to wait for. `0x14003c4ad` starts the limiter over where the clock is more
    /// than a period past `base + period × (count + 1)`, and a base of nought puts every clock there —
    /// so the function measures nothing and waits for nothing, and the frame's timing is
    /// `frame::Pacing`'s alone, which is what a second pacing on top of it would spoil. The fps line it
    /// writes in the corner is what this costs.
    unsafe fn end_drawing(&self, _device: orb_api::Device) {
        unsafe {
            mem::write::<i64>(at(THE_LIMITERS_BASE), 0);
            mem::write::<i64>(at(THE_LIMITERS_COUNT), 0);
        }
        let ends: unsafe extern "win64" fn() = unsafe { std::mem::transmute(at(ENDS_THE_FRAME)) };
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
        }
    }

    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    unsafe fn make_invulnerable(&self) {}

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
    use super::{FRAME_IN_THE_LOOPS_BYTES, Th06nc, bullet_manager, button, key};
    use crate::game::{Game, Read};

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

    /// **The frame is a stretch of the loop and not a function**: the stub stands in fourteen bytes,
    /// which is what a 64-bit jump takes, and no function of the game's is patched for the frame.
    #[test]
    fn the_frame_is_stood_in_by_the_stub_over_fourteen_bytes() {
        let hooks = Th06nc.hooks();
        assert!(hooks.render.is_none());
        let frame = hooks
            .frame_in_the_loop
            .expect("th06nc's frame is in its loop");
        assert_eq!(frame.patch.prologue, FRAME_IN_THE_LOOPS_BYTES);
        assert_eq!(frame.patch.prologue.len(), 14);
        assert_ne!(frame.stub, 0);
        assert!(hooks.walks.is_some(), "the two walks are handed over");
    }

    /// **What each of the five bullet states multiplies its own step by** — th06c's five.
    #[test]
    fn each_bullet_state_scales_its_own_step_by_what_the_game_divides_it_by() {
        for (state, scale) in [(1, 1.0), (2, 0.5), (3, 0.4), (4, 1.0 / 3.0), (5, 1.0)] {
            assert_eq!(bullet_manager::a_steps_scale(state), scale);
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
