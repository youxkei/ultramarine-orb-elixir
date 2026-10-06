//! How much of the screen the game gets, worked out.
//!
//! The window is the size it is going to be from the moment it exists, because the arguments of the
//! game's own `CreateWindowExA` call are rewritten on the way through — there is no frame to remove
//! afterwards and nothing to flash on screen first. Fullscreen is that window borderless and covering the
//! monitor; a size is that window centred on it, with a caption and nothing to drag, since the size is a
//! setting rather than something to pull about.
//!
//! In windowed mode the game asks Direct3D for a `D3DSWAPEFFECT_COPY` swap chain, and that is the swap
//! effect that honours a destination rectangle on `Present`. So the 640x480 back buffer is presented into
//! a centred rectangle with the game's aspect ratio, and the rest of the window keeps the black background
//! its class was given.
//!
//! **What is left in `orb::window` is the write over two of the exe's imports, and the black brush the
//! rewrite of `RegisterClassA` swaps in.** Everything those rewrites decide is here: where the window
//! goes, where the game goes inside it, the `Present` slot redirected into that rectangle, and where
//! orb's own lines go in the black that is left. The GDI those lines are measured and written with is
//! behind the seam, in `orb_api::window`; what is here is which of the two bars they go in and where in it
//! they start.
//!
//! The arguments of the game's own `CreateWindowExA` are rewritten on the way through, so the window is the
//! size it is going to be from the moment it exists — no frame to remove afterwards and nothing to flash
//! first. Its window class gets a black background brush, and that is the letterbox.
//!
//! `screen: fullscreen` is that window borderless — `WS_POPUP | WS_VISIBLE`, and nothing else, since a
//! caption or a frame is exactly what puts a border on one — covering the monitor. `screen: 1280x720` is
//! that window with a caption to move it by and a system menu to close it with, centred on the monitor, and
//! nothing to resize it with: the size is one of the settings, so dragging the edge would be a second place
//! to say it and the one that is not written down. The size is of what is *inside* the window,
//! `AdjustWindowRect` being what turns it into the window to ask for, so `1280x720` is 1280x720 of game
//! however thick the host's frames are. A window too big for the monitor goes against its top-left corner
//! rather than half off the top, since a caption above the screen cannot be dragged back onto it.
//!
//! **Always a window, either way.** The game's own fullscreen setting is overruled before it creates
//! anything: a game that has taken the display exclusively has no window to size, and orb needs one to write
//! its numbers in the black beside the game.
//!
//! **Display scaling is off**, said with `SetProcessDPIAware` at the same moment — before the window exists,
//! which is the only moment it can be said. So a size in `orb.yaml` is that many pixels of screen, and the
//! monitor a fullscreen window is measured against is the whole of it. The size the window came out with is
//! logged beside the size that was asked for, and again when the device is created, because those are two
//! different numbers whenever anything in between has an opinion.
//!
//! The back buffer stays 640x480. Windowed, the game asks for `D3DSWAPEFFECT_COPY`, the swap effect that
//! honours a destination rectangle on `Present`, so the back buffer goes into a centred rectangle of the
//! game's aspect ratio. Which is why a 16:9 window is worth offering at all: the black it leaves down the
//! sides is where the status line goes.
//!
//! **Anything outside the game that squares windows up wins**, and there is nothing orb can do about it:
//! such a tool acts on the window after it exists, which is after every moment orb has. It takes the black
//! beside the game with it, and with that the numbers written there — a window created with exactly the
//! client asked for, still that at the device, and squared up to 4:3 filling the monitor's height some
//! seconds later. The client the window came out with is logged next to the size asked for, and again at the
//! device, so a run where this happens says so rather than looking like orb getting the size wrong. vpatch's
//! `[Window]` section is the same hazard from inside.
//!
//! A monitor-sized back buffer would not scale the game: its 2D drawing uses `D3DFVF_XYZRHW`, whose
//! coordinates are already screen space, so a viewport does not transform them. Scaling that way would need
//! a render target and a full-screen quad.
//!
//! **The status line.** The chapter, `RETRY`, `INPUT LAG` and the frame rate are drawn with GDI onto the
//! window, in the black beside the game, stacked in whichever letterbox bar is wider and lined up with the
//! game's edge. `SCRIPT` and `TABLE` join them while a table is being built, and `HOLD` while the game's
//! update is held. On a 16:9 monitor a 4:3 game leaves that black down the sides. Shown whatever the game is
//! doing, including demos and menus, and redrawn only when the text changes.
//!
//! **In a run the chapter is named for the part of the stage it is in, and numbered inside it**: `MIDSTAGE
//! 3`, `MIDBOSS SPELL 1`, `BOSS NONSPELL 2`. A count of the chapters gone by says nothing about where the
//! game is standing, while the second spellcard of a fight is how a fight is talked about and how a chapter
//! worth grinding is named to somebody else. *Midstage* rather than "stage" because "STAGE 3" beside a stage
//! number reads as the third stage. The waves are one run of chapters through the whole stage — its start,
//! each midstage boundary, and the waves handed back when a midboss goes down — since a midboss interrupts
//! them rather than starting a new set: the chapter after the midboss of stage 4 is `MIDSTAGE 3`, not
//! `MIDSTAGE 1` again. Where there is no chapter, in a menu, that line is not there at all.
//!
//! **Neither it nor `RETRY` is there in a normal run**, where there are no chapters: an empty name and a
//! retry count that cannot move are two lines saying the run is not the one they describe. What is left is
//! the lag, the compositor's share and the frame rate, which are about the machine and are the same in both
//! modes.
//!
//! The count comes out of the frames this stage's chapters began at rather than a counter per kind, because
//! a retry puts the mark back and the run then reaches the same chapters again. A counter would name the
//! same chapter one further along every time it was played.
//!
//! **A pass building the table shows `CH 05` and why the chapter changed instead** — see [`crate::tuning`].
//! That is a different question from what the chapter is, and the one worth having while what is being
//! decided is whether the boundary belongs in the table: the name would say the same thing about a boundary
//! to keep and a boundary to throw away.
//!
//! **The GDI it is written with is gdi32's own and not this DLL's import of it.** A translation patch
//! injected into the same game rewrites the import table of every module in the process, orb's included,
//! and puts its own replacement in the text calls — one written for the game's `-A` calls, which takes a
//! `-W` call's count of characters for a count of bytes. So the calls that carry a string or hand back a
//! font are read out of gdi32's export directory instead, which is the loader's own and is not rewritten;
//! the overlay's glyphs go the same way.
//!
//! **Each draw clears what it is going to write in, and everything the last draw wrote in, to black
//! first.** A stack with fewer lines than the one before it, or a smaller font, does not reach every row that
//! already had text in it. Clearing and writing are two operations, so both go into a bitmap of their own
//! and reach the window as one `BitBlt`: done straight on the window, a refresh landing between them shows a
//! bar with nothing in it, and at 120Hz that is a flicker. A stack taller than the black there is runs off
//! the edge of that bitmap and is clipped there, rather than being drawn over the game.
//!
//! **The size fits the widest line into the black there is.** What goes here runs from three characters to
//! twenty — a chapter named for the part of a stage it belongs to is the long one — and a line clipped at
//! the bar's edge cannot be read at all, which is worse than small. The line is measured at the full size
//! and the height scaled by what it overran by, once per change of text, which lands inside a pixel or two
//! because character widths go with the em height. Nothing shrinks below the point where the text stops
//! being readable anyway; past that, clipping the odd long line is the better trade.
//!
//! The game's back buffer is the wrong place for it twice over: all 640x480 of it is shown inside the
//! letterbox, so anything there is over the game; and the game does not clear it between frames — it draws
//! the static background once and redraws only the moving parts — so anything in a corner it does not
//! repaint stays there and accumulates. The black is the window's own background and Direct3D never touches
//! it. The retry menu is drawn with the Direct3D overlay instead, because it belongs over the game.
//!
//! `INPUT LAG` is measured, from the keyboard read to that frame being on screen, across the two frames
//! those ends fall in.

use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

use orb_api::{Bar, Hresult, Hwnd, Rect};
use orb_config::Screen;

use crate::sync::MainThread;
use crate::{detail, log, profile};

/// All a borderless window needs. Anything else — caption, frame, system menu — is what puts a border on
/// it.
///
/// `WS_POPUP | WS_VISIBLE`, written out rather than taken from `windows-sys`: what a style is is the
/// game's own `CreateWindowExA` argument, and this is the half that decides it. `orb::window` holds the
/// two against Windows' own numbers at compile time, so a wrong one here is a build that stops.
pub const BORDERLESS_STYLE: u32 = 0x8000_0000 | WS_VISIBLE;
/// A window of a chosen size: a caption to move it by and a system menu to close it with, and nothing to
/// resize it with. The size is one of the settings, so dragging the edge of the window would be a second
/// place to say it and the one that is not written down.
///
/// `WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE`.
pub const WINDOWED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x0002_0000 | WS_VISIBLE;

/// **The one bit that decides whether a window is on the screen the moment it is made** — `WS_VISIBLE`,
/// written out for [`BORDERLESS_STYLE`]'s reason and held against Windows' own number by `orb::window`.
///
/// In the two styles above, and taken out of the one a window orb plays in is made with — see
/// [`create_window_ex_w`], which strips it, and [`keeps_its_corner`], which shows the window once the
/// corner has held.
pub const WS_VISIBLE: u32 = 0x1000_0000;

/// How tall the lines written beside the game are, in pixels of the monitor rather than
/// of the game — this text is not scaled with the game's output. Reduced where the widest
/// line does not fit the black it is written in.
const BAR_TEXT_HEIGHT: i32 = 30;
/// How small that may get. A line clipped at the bar's edge cannot be read at all, so
/// fitting it wins over size — down to here, below which nothing is readable either and
/// clipping the odd long line is the better trade.
const BAR_TEXT_MIN: i32 = 11;
/// How far the text keeps from the edges of the window.
const BAR_MARGIN: i32 = 8;

/// The aspect ratio to keep, as the size the game renders at.
static CONTENT: MainThread<(u32, u32)> = MainThread::new((640, 480));
/// How much of the screen the game is to get, out of `orb.yaml`.
static SCREEN: MainThread<Screen> = MainThread::new(Screen::Fullscreen);
/// And whether the window is one orb plays in rather than one somebody does — `Config::autoplay`, which
/// takes that decision off the settings entirely. See [`placed_out_of_the_way`].
static OUT_OF_THE_WAY: MainThread<bool> = MainThread::new(false);
/// The window created for the game, which is the device's window.
///
/// **Here rather than with the rewrite that stores it.** `orb::window`'s replacement of
/// `CreateWindowExA` is the writer and stays there, being a patch; which window the game got is a fact
/// about the run rather than about the patch, so it lives with the two that read it — and an e2e test
/// reading the letterbox back then needs no launch to have happened for the address to be there.
static GAME_WINDOW: AtomicUsize = AtomicUsize::new(0);
/// Worked out from the client area on the first present and kept until it changes.
static DESTINATION: MainThread<Option<(Rect, Rect)>> = MainThread::new(None);

/// Whether orb is deciding this game's window at all, which [`install_over`] having run is what says so.
///
/// Read by [`hook_device`]: a launch whose window hooks did not go in is one where the game's window is
/// whatever the game asked for, and presenting into a rectangle of a window nobody laid out would put the
/// game somewhere inside its own. Which is why `orb::window::install` hands over *last* — a launch that
/// could not find one of the two import entries settles nothing.
static ENABLED: AtomicBool = AtomicBool::new(false);
/// What was in the device's `Present` slot before orb's went in, and zero before that has happened.
static PRESENT: AtomicUsize = AtomicUsize::new(0);

/// The window class the game registers and creates. Matching it means orb leaves
/// alone any other window the game makes.
///
/// A `CStr` comparison against the game's own class name and no Windows anywhere in it, which is why
/// this and [`is_game_class`] are here even though the one rewrite that stays in `orb` —
/// `register_class_a`, for its brush — asks them too.
const GAME_WINDOW_CLASS: &CStr = c"BASE";

/// Whether a class name is the game's own. `pub` because `orb::window`'s rewrite of `RegisterClassA` asks
/// it, that one staying there for the black brush it swaps in.
///
/// # Safety
/// `name` must be null, an atom, or a readable NUL-terminated string.
pub unsafe fn is_game_class(name: *const u8) -> bool {
    // A class can also be given as an atom, which has nothing to dereference.
    if name.is_null() || (name as usize) <= u16::MAX as usize {
        return false;
    }
    unsafe { CStr::from_ptr(name.cast()) == GAME_WINDOW_CLASS }
}

/// The game's own `CreateWindowExA`, which [`create_window_ex_a`] calls through with the arguments it
/// decided.
///
/// The window handles are `*mut c_void`, which is what `HWND` and `HMENU` both are: nothing here looks
/// inside one, and what orb does with the window it made goes through [`orb_api::Hwnd`].
#[allow(clippy::type_complexity)]
pub type CreateWindowExA = unsafe extern "system" fn(
    u32,
    *const u8,
    *const u8,
    u32,
    i32,
    i32,
    i32,
    i32,
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *const c_void,
) -> *mut c_void;

/// The game's own `CreateWindowExA`, which the game's import table pointed at — or the function a game
/// laid out by hand handed over in place of it.
static CREATE_WINDOW_EX_A: AtomicUsize = AtomicUsize::new(0);

/// And the wide one, for a game that asks the host for it by name rather than importing either — see
/// [`create_window_ex_w`] and `game::Hooks::makes_its_window`.
static CREATE_WINDOW_EX_W: AtomicUsize = AtomicUsize::new(0);

/// The same twelve arguments with the two strings wide, which is the whole difference between the two
/// calls.
#[allow(clippy::type_complexity)]
pub type CreateWindowExW = unsafe extern "system" fn(
    u32,
    *const u16,
    *const u16,
    u32,
    i32,
    i32,
    i32,
    i32,
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *const c_void,
) -> *mut c_void;

/// **Makes the window a launch that plays itself gets in [`the_leftmost_of`]'s corner, framed as a
/// dialog and kept off the taskbar** — the corner and the frame of [`placed_out_of_the_way`], which is
/// what the game whose `CreateWindowExA` is written over gets the whole of.
///
/// **[`OUT_OF_THE_WAY_EX_STYLE`] for the reason it is there at all**: a window a bot plays in that the window
/// manager treats as an ordinary one is a window in the way — in the taskbar, in alt-tab, and taking a
/// turn at being focused.
///
/// **But not the size here**, which is [`keeps_its_corner`]'s to hold: the game moves and sizes its own
/// window again after it is made, applying the resolution its startup dialog was told, so a size asked for
/// here is a size undone. What makes a small window a small picture rather than a cut one is
/// `crate::dxgi`'s — see its `resizes_its_buffers`.
///
/// **And nothing at all for a launch somebody plays**, which is the other thing this does differently: a
/// game that looks this call up has had no [`settle`] run for it — the installation that one is part of
/// needs an import table — so there is no ratio, no screen and no letterbox settled, and a window laid
/// out against them would be laid out against nothing.
///
/// **Top-level windows only.** A child is placed in its parent's own coordinates, where the desktop's
/// corner means nothing at all.
///
/// # Safety
/// The arguments are `CreateWindowExW`'s own, and [`answers_the_window_made`] must have run first.
#[allow(clippy::too_many_arguments)]
pub unsafe extern "system" fn create_window_ex_w(
    ex_style: u32,
    class_name: *const u16,
    window_name: *const u16,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: *mut c_void,
    menu: *mut c_void,
    instance: *mut c_void,
    param: *const c_void,
) -> *mut c_void {
    let original: CreateWindowExW =
        unsafe { std::mem::transmute(CREATE_WINDOW_EX_W.load(Ordering::Relaxed)) };
    let corner = (unsafe { *OUT_OF_THE_WAY.get() } && parent.is_null())
        .then(|| the_leftmost_of(&orb_api::window::monitors()))
        .flatten();
    let (into_x, into_y) = match corner {
        Some(monitor) => (monitor.left, monitor.top),
        None => (x, y),
    };
    // **And made hidden, for a window orb chose the corner of.** Something moves this window after it is
    // made — [`keeps_its_corner`] is what that costs — so a window that is on the screen from the moment
    // it is created is one somebody sees in the middle of the primary monitor before it is put back. What
    // is on the screen while that is settled is nothing at all, and [`keeps_its_corner`] shows it once the
    // corner has held.
    let style = match corner {
        Some(_) => style & !WS_VISIBLE,
        None => style,
    };
    let window = unsafe {
        original(
            // Added to the game's own rather than written over it, the same as the other rewrite.
            ex_style | corner.map_or(0, |_| OUT_OF_THE_WAY_EX_STYLE),
            class_name,
            window_name,
            style,
            into_x,
            into_y,
            width,
            height,
            parent,
            menu,
            instance,
            param,
        )
    };
    if !window.is_null() && corner.is_some() {
        // Remembered so that [`keeps_its_corner`] has a window to put back, whatever moves it — and the
        // frames this one has held its corner for counted from nought, this game making its window over
        // once the settings are applied and the window it made first not the one that is played in.
        PLAYED_IN.store(window as usize, Ordering::Relaxed);
        HELD_FOR.store(0, Ordering::Relaxed);
        log!(
            "screen: the window orb plays in is at {into_x},{into_y} sized {width}x{height}, \
             which is the size the game asked for, and is hidden until its corner holds",
        );
    }
    window
}

/// The game's own `MoveWindow`, which it calls through — or the whole of it in a bot's launch.
static MOVE_WINDOW: AtomicUsize = AtomicUsize::new(0);

/// `BOOL MoveWindow(HWND, int x, int y, int cx, int cy, BOOL repaint)`.
pub type MoveWindow = unsafe extern "system" fn(*mut c_void, i32, i32, i32, i32, i32) -> i32;

/// **Holds the window orb plays in in its corner and at its size**, for a game that moves its own
/// afterwards — the size being [`OUT_OF_THE_WAY_CLIENT`] inside the window's own frame, for a game that
/// draws at the size it asked for, and the game's own for any other: see [`the_size_it_plays_at`].
///
/// **The size as well as the corner, because the game's own moves are what undid it.** th06c's move asks
/// for that corner and the resolution its dialog was told, and a launch that held the size in
/// [`keeps_its_corner`] alone was put back every frame — `sized 662x536; putting it back … sized 342x296`
/// on line after line — and never held long enough to be shown.
///
/// **A launch nobody asked to play itself is moved exactly as it asked.**
///
/// # Safety
/// The arguments are `MoveWindow`'s own, and [`answers_the_window_moved`] must have run first.
pub unsafe extern "system" fn move_window(
    window: *mut c_void,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    repaint: i32,
) -> i32 {
    let original: MoveWindow = unsafe { std::mem::transmute(MOVE_WINDOW.load(Ordering::Relaxed)) };
    let corner = unsafe { *OUT_OF_THE_WAY.get() }
        .then(|| the_leftmost_of(&orb_api::window::monitors()))
        .flatten();
    let (into_x, into_y) = match corner {
        Some(monitor) => (monitor.left, monitor.top),
        None => (x, y),
    };
    let ours = corner.is_some() && window as usize == PLAYED_IN.load(Ordering::Relaxed);
    let (into_width, into_height) = match ours {
        true => the_size_asked_of(window as usize, width, height),
        false => (width, height),
    };
    if corner.is_some() && (into_x, into_y, into_width, into_height) != (x, y, width, height) {
        detail!(
            "screen: the game moved its window to {x},{y} sized {width}x{height}; it goes to \
             {into_x},{into_y} sized {into_width}x{into_height} instead"
        );
    }
    unsafe { original(window, into_x, into_y, into_width, into_height, repaint) }
}

/// **The size a move of the window orb plays in is made at in place of the game's own `width` by
/// `height`**: the whole window whose client is [`OUT_OF_THE_WAY_CLIENT`] — see [`the_size_it_plays_at`] —
/// and the size asked where the host will not say where the window is.
fn the_size_asked_of(window: usize, width: i32, height: i32) -> (i32, i32) {
    if !unsafe { crate::runtime::the_game_draws_at_the_size_it_asked_for() } {
        return (width, height);
    }
    match orb_api::window::window_rect(Hwnd(window)) {
        Some(is_at) => the_size_it_plays_at(window, is_at),
        None => (width, height),
    }
}

/// Says which function [`move_window`] calls through.
///
/// # Safety
/// Must run before the game has moved its window.
pub unsafe fn answers_the_window_moved(original: usize) {
    MOVE_WINDOW.store(original, Ordering::Relaxed);
}

/// The game's own `ShowWindow`, which [`show_window`] calls through — or nothing at all while the window
/// orb plays in is being kept off the screen.
static SHOW_WINDOW: AtomicUsize = AtomicUsize::new(0);

/// `BOOL ShowWindow(HWND, int how)`.
pub type ShowWindow = unsafe extern "system" fn(*mut c_void, i32) -> i32;

/// **Keeps the window orb plays in off the screen until its corner has held**, for a game that shows its
/// own window — which is what [`create_window_ex_w`]'s stripped `WS_VISIBLE` would otherwise be undone
/// by.
///
/// **Because the style is not the only way a window reaches the screen.** th06nc looks `ShowWindow` up by
/// name at its startup — `hook: the game looked ShowWindow up` — and a launch whose window was made
/// hidden was measured on the screen one frame later with orb having shown nothing, which put every
/// put-back [`keeps_its_corner`] makes in front of somebody: the window appeared at `1269,656`, the
/// middle of the primary monitor, and jumped to the corner a frame afterwards.
///
/// **Swallowed rather than deferred**: what the game asks for here is the window it has just made put on
/// the screen, and orb shows that same window itself once the corner is settled — so the ask is answered
/// by the show that is coming, and nothing is queued. Nought is the answer, which is `ShowWindow`'s for a
/// window that was not visible, and true at the moment it is given.
///
/// Everything else through: another window of the game's, a launch nobody asked to play itself, and every
/// call after orb has shown the window — a game hiding or minimising its own window after that is a game
/// doing what it was told to.
///
/// # Safety
/// The arguments are `ShowWindow`'s own, and [`answers_the_window_shown`] must have run first.
pub unsafe extern "system" fn show_window(window: *mut c_void, how: i32) -> i32 {
    let original: ShowWindow = unsafe { std::mem::transmute(SHOW_WINDOW.load(Ordering::Relaxed)) };
    if unsafe { held_off_the_screen(window) } {
        detail!("screen: the game asked to show the window orb plays in ({how}); it stays hidden");
        return 0;
    }
    unsafe { original(window, how) }
}

/// Says which function [`show_window`] calls through.
///
/// # Safety
/// Must run before the game has shown its window.
pub unsafe fn answers_the_window_shown(original: usize) {
    SHOW_WINDOW.store(original, Ordering::Relaxed);
}

/// The game's own `SetForegroundWindow`, which [`set_foreground_window`] calls through.
static SET_FOREGROUND_WINDOW: AtomicUsize = AtomicUsize::new(0);

/// `BOOL SetForegroundWindow(HWND)`.
pub type SetForegroundWindow = unsafe extern "system" fn(*mut c_void) -> i32;

/// **The window orb plays in left where it is when the game asks for it to be brought to the front**, and every
/// other window brought forward as asked.
///
/// **Because [`OUT_OF_THE_WAY_EX_STYLE`]'s `WS_EX_NOACTIVATE` does not reach this call**: it keeps a window from
/// being activated by a click and by the system, and a program asking for its own window to be in front is asked
/// for outright — th10's WinMain does that once its window is made (0x438cb1), and a launch with the style on still
/// took the focus off whatever somebody was working in. True is the answer, which is the host's for a window brought
/// forward: the game asked to be in front and goes on as though it were.
///
/// # Safety
/// The argument is `SetForegroundWindow`'s own, and [`answers_the_window_brought_forward`] must have run first.
pub unsafe extern "system" fn set_foreground_window(window: *mut c_void) -> i32 {
    let original: SetForegroundWindow =
        unsafe { std::mem::transmute(SET_FOREGROUND_WINDOW.load(Ordering::Relaxed)) };
    let ours = window as usize;
    if unsafe { *OUT_OF_THE_WAY.get() }
        && ours != 0
        && (ours == GAME_WINDOW.load(Ordering::Relaxed)
            || ours == PLAYED_IN.load(Ordering::Relaxed))
    {
        detail!(
            "screen: the game asked for the window orb plays in to be brought forward; it stays where it is"
        );
        return 1;
    }
    unsafe { original(window) }
}

/// The game's own `SetActiveWindow`, which [`set_active_window`] calls through.
static SET_ACTIVE_WINDOW: AtomicUsize = AtomicUsize::new(0);

/// `HWND SetActiveWindow(HWND)`.
pub type SetActiveWindow = unsafe extern "system" fn(*mut c_void) -> *mut c_void;

/// **The window orb plays in left as it is when the game asks for it to be activated**, and every other window
/// activated as asked — [`set_foreground_window`]'s answer to the other call a game brings itself forward with.
///
/// # Safety
/// The argument is `SetActiveWindow`'s own, and [`answers_the_window_activated`] must have run first.
pub unsafe extern "system" fn set_active_window(window: *mut c_void) -> *mut c_void {
    let original: SetActiveWindow =
        unsafe { std::mem::transmute(SET_ACTIVE_WINDOW.load(Ordering::Relaxed)) };
    let ours = window as usize;
    if unsafe { *OUT_OF_THE_WAY.get() }
        && ours != 0
        && (ours == GAME_WINDOW.load(Ordering::Relaxed)
            || ours == PLAYED_IN.load(Ordering::Relaxed))
    {
        detail!(
            "screen: the game asked for the window orb plays in to be activated; it is left as it is"
        );
        // Nothing, which is `SetActiveWindow`'s answer where no window was active before: the game goes on as though
        // its window were, th08 reading nothing of what comes back (0x443708).
        return std::ptr::null_mut();
    }
    unsafe { original(window) }
}

/// Says which function [`set_active_window`] calls through.
///
/// # Safety
/// Must run before the game asks for its window to be activated.
pub unsafe fn answers_the_window_activated(original: usize) {
    SET_ACTIVE_WINDOW.store(original, Ordering::Relaxed);
}

/// The game's own `SendMessageA`, which [`send_message_a`] calls through.
static SEND_MESSAGE_A: AtomicUsize = AtomicUsize::new(0);

/// `LRESULT SendMessageA(HWND, UINT, WPARAM, LPARAM)`.
pub type SendMessageA = unsafe extern "system" fn(*mut c_void, u32, usize, isize) -> isize;

/// `WM_SYSCOMMAND`, and the two of its commands that minimise a window and put it back.
const WM_SYSCOMMAND: u32 = 0x0112;
const SC_MINIMIZE: usize = 0xf020;
const SC_RESTORE: usize = 0xf120;

/// **The window orb plays in neither minimised nor put back when the game asks**, and every other message sent as
/// asked — [`set_foreground_window`]'s answer to a third way a game brings itself forward: th11 sends its new
/// window `SC_MINIMIZE`, sleeps 16 milliseconds and sends it `SC_RESTORE` (0x446cfa, 0x446d16), and a window put
/// back from the taskbar is a window Windows activates, whatever its style. Nought is the answer, which is a
/// window procedure's for a system command it carried out.
///
/// # Safety
/// The arguments are `SendMessageA`'s own, and [`answers_the_message_sent`] must have run first.
pub unsafe extern "system" fn send_message_a(
    window: *mut c_void,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    let original: SendMessageA =
        unsafe { std::mem::transmute(SEND_MESSAGE_A.load(Ordering::Relaxed)) };
    let ours = window as usize;
    if unsafe { *OUT_OF_THE_WAY.get() }
        && ours != 0
        && (ours == GAME_WINDOW.load(Ordering::Relaxed)
            || ours == PLAYED_IN.load(Ordering::Relaxed))
        && message == WM_SYSCOMMAND
        && matches!(wparam & 0xfff0, SC_MINIMIZE | SC_RESTORE)
    {
        detail!(
            "screen: the game asked for the window orb plays in to be minimised or put back; it stays as it is"
        );
        return 0;
    }
    unsafe { original(window, message, wparam, lparam) }
}

/// Says which function [`send_message_a`] calls through.
///
/// # Safety
/// Must run before the game sends its window anything.
pub unsafe fn answers_the_message_sent(original: usize) {
    SEND_MESSAGE_A.store(original, Ordering::Relaxed);
}

/// The game's own `Direct3DCreate9`, which [`direct3d_create9`] calls through.
static DIRECT3D_CREATE9: AtomicUsize = AtomicUsize::new(0);

/// And the `IDirect3D9::CreateDevice` its object came with, which [`create_device9`] calls through.
static CREATE_DEVICE9: AtomicUsize = AtomicUsize::new(0);

/// `IDirect3D9 *Direct3DCreate9(UINT SDKVersion)`.
pub type Direct3DCreate9 = unsafe extern "system" fn(u32) -> *mut c_void;

/// `HRESULT IDirect3D9::CreateDevice(this, adapter, device type, focus window, behaviour flags, presentation
/// parameters, the device made)`.
pub type CreateDevice9 = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *mut c_void,
    u32,
    *mut c_void,
    *mut *mut c_void,
) -> i32;

/// **Whether the device is made to present without waiting for the display**, for a launch paced faster than it —
/// see [`presents_without_waiting`].
static PRESENTS_WITHOUT_WAITING: AtomicBool = AtomicBool::new(false);

/// **Says whether a Direct3D 9 device is to be made presenting without waiting for the display's blank**, which is
/// the whole of what a launch paced faster than the display needs of one: its interval is the display pacing it,
/// so a frame asked for faster than the blanks and a present that waits on one are two answers to one question, and
/// the blank wins. **Measured**: th10 at `--pace=8` ran at 2.00x on a 120Hz display, `601 frames, 8309us apart` — one
/// frame a refresh. The DXGI games' answer is the same, made per present — see `crate::dxgi::hands_over_without_waiting`
/// — where a D3D9 device's interval is its present parameters', set as it is made.
///
/// Set only from [`crate::frame::Pacing::paces_faster`], so an ordinary launch's device is made as the game asks.
pub fn presents_without_waiting(without: bool) {
    PRESENTS_WITHOUT_WAITING.store(without, Ordering::Relaxed);
}

/// **The Direct3D 9 object the game makes its device with**, with its `CreateDevice` written over where orb plays
/// the game in a window of its own or paces it faster than the display — see [`create_device9`].
///
/// # Safety
/// The argument is `Direct3DCreate9`'s own, and [`answers_the_direct3d_made`] must have run first.
pub unsafe extern "system" fn direct3d_create9(sdk: u32) -> *mut c_void {
    let original: Direct3DCreate9 =
        unsafe { std::mem::transmute(DIRECT3D_CREATE9.load(Ordering::Relaxed)) };
    let made = unsafe { original(sdk) };
    let wanted =
        unsafe { *OUT_OF_THE_WAY.get() } || PRESENTS_WITHOUT_WAITING.load(Ordering::Relaxed);
    if made.is_null() || !wanted {
        return made;
    }
    // The vtable read out of the object, for the reason [`hook_device`] reads the device's that way.
    let vtable = unsafe { orb_api::mem::read::<usize>(made as usize) };
    let slot = vtable + orb_api::d3d9::CREATE_DEVICE_SLOT * size_of::<usize>();
    let ours = create_device9 as *const () as usize;
    match unsafe { orb_api::mem::replace_word(slot, ours) } {
        // **Kept only where it is not orb's own**: every object `Direct3DCreate9` makes shares the one vtable, so a
        // second object finds orb's in the slot already, and the game's is the one kept from the first.
        Some(previous) if previous != ours => CREATE_DEVICE9.store(previous, Ordering::Relaxed),
        Some(_) => {}
        None => log!("screen: the Direct3D object's CreateDevice could not be written over"),
    }
    made
}

/// **The device made with Direct3D told to leave the focus window alone**, where orb plays the game in a window of
/// its own: `D3DCREATE_NOWINDOWCHANGES` beside the game's own flags. **Because the device's making is the one call of
/// the game's that can have brought the window to the front**: made hidden, shown with `SW_SHOWNA`,
/// `WS_EX_NOACTIVATE` on it and the game's own `SetForegroundWindow` swallowed, th10's window still came to the front
/// some 110 milliseconds after it was made — tick 3501540094 to 3501540109 on the log's clock, between the window
/// made at 3501539993 and the pad read at 3501540233, WinMain's device (0x439890) being the one call of the game's
/// between those. Whether it is the device or Windows bringing a new process's first window forward is not
/// established — see th10's `asm.md`, where the window still comes to the front with this flag on.
///
/// **And made presenting without waiting for the display**, where the launch is paced faster than it — see
/// [`presents_without_waiting`]. **Written into the game's own parameters** rather than a copy of them, because
/// th10 hands the same ones to its `Reset` (0x439232), and a device put back by a reset that waited again would be a
/// pace lost from then on.
///
/// # Safety
/// The arguments are `IDirect3D9::CreateDevice`'s own, and [`direct3d_create9`] must have written this over the slot.
pub unsafe extern "system" fn create_device9(
    this: *mut c_void,
    adapter: u32,
    kind: u32,
    focus: *mut c_void,
    behaviour: u32,
    parameters: *mut c_void,
    device: *mut *mut c_void,
) -> i32 {
    let original: CreateDevice9 =
        unsafe { std::mem::transmute(CREATE_DEVICE9.load(Ordering::Relaxed)) };
    let behaviour = match unsafe { *OUT_OF_THE_WAY.get() } {
        true => behaviour | orb_api::d3d9::D3DCREATE_NOWINDOWCHANGES,
        false => behaviour,
    };
    // **And not `D3DCREATE_MULTITHREADED`**, though th11's title loads `title.anm` and `title_v.anm` on a thread of its
    // own (0x4394f0, their textures among it) while the main thread draws, through a device made with `0x40` or
    // `0x20` alone (0x446f5d): with and without it, one launch of each in turns over 30 pairs of
    // `recipes/th11/measured/the_main_menu_reached_and_left` at pace 16, 2 and 3 stood spinning at the title on a
    // `title.anm` script written over (`scratchpad` sessions from 1791766810, 2026-10-12) — the 6 of 30 without it
    // and 1 of 30 with it of separate sessions before that (1791750231, 1791754411) were the sessions' own swing.
    if PRESENTS_WITHOUT_WAITING.load(Ordering::Relaxed) && !parameters.is_null() {
        let interval = parameters as usize + orb_api::d3d9::PRESENTATION_INTERVAL;
        unsafe {
            orb_api::mem::write::<u32>(interval, orb_api::d3d9::D3DPRESENT_INTERVAL_IMMEDIATE)
        };
        log!("frame: the device is made presenting without waiting for the display's blank");
    }
    unsafe { original(this, adapter, kind, focus, behaviour, parameters, device) }
}

/// Says which function [`direct3d_create9`] calls through.
///
/// # Safety
/// Must run before the game makes its Direct3D object.
pub unsafe fn answers_the_direct3d_made(original: usize) {
    DIRECT3D_CREATE9.store(original, Ordering::Relaxed);
}

/// Says which function [`set_foreground_window`] calls through.
///
/// # Safety
/// Must run before the game asks for its window to be brought forward.
pub unsafe fn answers_the_window_brought_forward(original: usize) {
    SET_FOREGROUND_WINDOW.store(original, Ordering::Relaxed);
}

/// The game's own `SetWindowPos`, which [`set_window_pos`] calls through.
static SET_WINDOW_POS: AtomicUsize = AtomicUsize::new(0);

/// `BOOL SetWindowPos(HWND, HWND after, int x, int y, int cx, int cy, UINT flags)`.
pub type SetWindowPos =
    unsafe extern "system" fn(*mut c_void, *mut c_void, i32, i32, i32, i32, u32) -> i32;

/// `SWP_NOSIZE`, `SWP_NOMOVE` and `SWP_SHOWWINDOW`, written out for [`BORDERLESS_STYLE`]'s reason and held
/// against Windows' own numbers by `orb::window`: what crosses into here is the game's own argument, and
/// these are the three bits of it orb reads.
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_SHOWWINDOW: u32 = 0x0040;

/// **The corner kept and the show swallowed where the game places its own window** — the other half of
/// [`show_window`], `SetWindowPos` being the one call that moves a window and puts it on the screen at
/// once.
///
/// th06nc looks this up beside `ShowWindow`, and the two moves a launch of it was measured making —
/// `-2160,-1676` to `1269,656` and then to `1749,926` — are moves [`keeps_its_corner`] put back a frame
/// later, which is a frame of the window somewhere else. Answered here, they never land.
///
/// **The size as well as the position**, for [`move_window`]'s reason. A call carrying `SWP_NOMOVE` asks
/// for no move and gets none, and one carrying `SWP_NOSIZE` no size.
///
/// # Safety
/// The arguments are `SetWindowPos`'s own, and [`answers_the_window_placed`] must have run first.
#[allow(clippy::too_many_arguments)]
pub unsafe extern "system" fn set_window_pos(
    window: *mut c_void,
    after: *mut c_void,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    flags: u32,
) -> i32 {
    let original: SetWindowPos =
        unsafe { std::mem::transmute(SET_WINDOW_POS.load(Ordering::Relaxed)) };
    let ours =
        unsafe { *OUT_OF_THE_WAY.get() } && window as usize == PLAYED_IN.load(Ordering::Relaxed);
    let corner = ours
        .then(|| the_leftmost_of(&orb_api::window::monitors()))
        .flatten();
    let (into_x, into_y) = match corner {
        Some(monitor) if flags & SWP_NOMOVE == 0 => (monitor.left, monitor.top),
        _ => (x, y),
    };
    let (into_width, into_height) = match corner {
        Some(_) if flags & SWP_NOSIZE == 0 => the_size_asked_of(window as usize, width, height),
        _ => (width, height),
    };
    if (into_x, into_y, into_width, into_height) != (x, y, width, height) {
        detail!(
            "screen: the game placed its window at {x},{y} sized {width}x{height}; it goes to \
             {into_x},{into_y} sized {into_width}x{into_height} instead"
        );
    }
    // And the show, for the reason [`show_window`] swallows one: a window put on the screen here is one
    // put there before its corner is settled.
    let flags = if unsafe { held_off_the_screen(window) } && flags & SWP_SHOWWINDOW != 0 {
        detail!(
            "screen: the game asked to show the window orb plays in as it placed it; it stays hidden"
        );
        flags & !SWP_SHOWWINDOW
    } else {
        flags
    };
    unsafe {
        original(
            window,
            after,
            into_x,
            into_y,
            into_width,
            into_height,
            flags,
        )
    }
}

/// Says which function [`set_window_pos`] calls through.
///
/// # Safety
/// Must run before the game has placed its window.
pub unsafe fn answers_the_window_placed(original: usize) {
    SET_WINDOW_POS.store(original, Ordering::Relaxed);
}

/// The game's own `SetWindowLongW`, which [`set_window_long_w`] calls through.
static SET_WINDOW_LONG_W: AtomicUsize = AtomicUsize::new(0);

/// `LONG SetWindowLongW(HWND, int index, LONG value)`.
pub type SetWindowLongW = unsafe extern "system" fn(*mut c_void, i32, i32) -> i32;

/// `GWL_STYLE` and `GWL_EXSTYLE`, written out for [`BORDERLESS_STYLE`]'s reason and held against Windows'
/// own numbers by `orb::window`: what crosses into here is the game's own argument, and these are the two
/// indices of it orb reads.
pub const GWL_STYLE: i32 = -16;
pub const GWL_EXSTYLE: i32 = -20;

/// **The dialog frame kept and the show stripped where the game writes its window's styles itself** —
/// the write that put the window orb plays in on the screen as an ordinary window, which a window manager
/// then placed as it places any other.
///
/// th06c enters its window mode through the pair at `0x14009d12e`/`0x14009d151` (and `0x14009d8a4`/
/// `0x14009d8c4`), both through `SetWindowLongW` looked up by name: the extended style first, out of a
/// table holding `0x100` — no dialog frame and back in the taskbar — and then the style, with
/// `WS_VISIBLE` added at `0x14009d0ee`, a style write being a show of its own. th06nc makes the same pair
/// at `0x1400befee`/`0x1400bf011` and `0x1400bf764`/`0x1400bf784`. A launch was measured with the window
/// on the screen and its extended style `0x100` before orb had shown it, and moved to `-2146,853` by the
/// window manager, which is not a place orb or the game chose. [`keeps_its_frame`] puts the frame back a
/// frame later, which is a frame too late: what decides a window manager is what the window is when it
/// first reaches the screen.
///
/// **`SetWindowLongW` and not its `A` twin beside it**: the game's wrapper at `0x140096f40` takes `W`
/// where `0x140442194` is set, which `0x140095b28` sets on any system with Unicode — every NT.
///
/// Everything else through: another window of the game's, a launch nobody asked to play itself, another
/// index, and the style written once orb has shown the window.
///
/// # Safety
/// The arguments are `SetWindowLongW`'s own, and [`answers_the_window_restyled`] must have run first.
pub unsafe extern "system" fn set_window_long_w(
    window: *mut c_void,
    index: i32,
    value: i32,
) -> i32 {
    let original: SetWindowLongW =
        unsafe { std::mem::transmute(SET_WINDOW_LONG_W.load(Ordering::Relaxed)) };
    let ours =
        unsafe { *OUT_OF_THE_WAY.get() } && window as usize == PLAYED_IN.load(Ordering::Relaxed);
    let value = match index {
        GWL_EXSTYLE
            if ours && value as u32 & OUT_OF_THE_WAY_EX_STYLE != OUT_OF_THE_WAY_EX_STYLE =>
        {
            detail!(
                "screen: the game wrote the extended style {value:#010x} over the window orb plays in; \
                 it keeps the dialog frame"
            );
            (value as u32 | OUT_OF_THE_WAY_EX_STYLE) as i32
        }
        GWL_STYLE if unsafe { held_off_the_screen(window) } && value as u32 & WS_VISIBLE != 0 => {
            detail!(
                "screen: the game wrote the style {value:#010x} over the window orb plays in; \
                 it stays hidden"
            );
            (value as u32 & !WS_VISIBLE) as i32
        }
        _ => value,
    };
    unsafe { original(window, index, value) }
}

/// Says which function [`set_window_long_w`] calls through.
///
/// # Safety
/// Must run before the game has written its window's styles.
pub unsafe fn answers_the_window_restyled(original: usize) {
    SET_WINDOW_LONG_W.store(original, Ordering::Relaxed);
}

/// **Whether this is the window orb is keeping off the screen**: the one a launch that plays itself made
/// in a corner, up to the moment [`shown_now_the_corner_is_held`] shows it.
///
/// # Safety
/// Must run on the game's main thread, after [`answers_the_window_made`].
unsafe fn held_off_the_screen(window: *mut c_void) -> bool {
    let window = window as usize;
    let ours = unsafe { *OUT_OF_THE_WAY.get() };
    ours && window != 0
        && window == PLAYED_IN.load(Ordering::Relaxed)
        && window != SHOWN.load(Ordering::Relaxed)
}

/// The window a game that looks its own creation up was given, so that [`keeps_its_corner`] has one to
/// put back.
static PLAYED_IN: AtomicUsize = AtomicUsize::new(0);

/// How many frames running that window has been found already in its corner, counted from the frame it
/// was made and from every frame it had to be put back on.
static HELD_FOR: AtomicUsize = AtomicUsize::new(0);

/// And the window [`shown_now_the_corner_is_held`] has shown, so that a game making its window over gets
/// the new one shown too — the handle and not a flag, that being what says which window it was.
static SHOWN: AtomicUsize = AtomicUsize::new(0);

/// Whether a window orb made hidden has been found on the screen with orb not having shown it, which is
/// said once a launch — see [`keeps_its_corner`].
static WAS_SHOWN_BY_SOMETHING_ELSE: AtomicBool = AtomicBool::new(false);

/// **How long the corner has to hold before the window is shown**, in frames.
///
/// A second of them, and not the one frame that would say the corner was held at all: what moves the
/// window does it while the game is starting — at the swapchain it builds for the window it has just made
/// — and orb has no hold on which frame that is. A window shown before then is one that flashes in the
/// middle of the monitor, which is the whole thing being avoided, and a second of nothing on the screen
/// is a second of a game that is loading anyway. Whatever moves it later is [`keeps_its_corner`]'s to
/// answer, as it was before the window was made hidden at all.
///
/// **And not the swapchain answered instead**, which was the other way out: orb already stands in front
/// of `CreateDXGIFactory2` for these games, and putting the corner back inside `CreateSwapChainForHwnd`
/// before it returns would be a put-back no frame comes between. What that costs is a reading of whether
/// the move even happens there — the two moves a launch of th06nc was measured making, from
/// `-2160,-1676` to `1269,656` and then to `1749,926`, are moves nothing in the log says the maker of —
/// and a window nobody can see needs no such reading, whoever moves it and however many times.
const FRAMES_THE_CORNER_HOLDS_FIRST: usize = 60;

/// **Puts the window orb plays in back in its corner where something has moved it**, asked once a frame.
///
/// **Because answering the calls is not enough.** A window made in the corner at the game's own size,
/// with th06c's own `MoveWindow` answered and both of its `SetWindowPos` calls carrying `SWP_NOMOVE`,
/// was still measured in the middle of the primary monitor — the mover is something else again, and
/// `SetWindowPos` is in the names that exe looks up as well as in its import table. So this stops asking
/// who moved it: what orb decided is where the window is put back to, whoever moved it and however.
///
/// Nothing where no launch is playing the game itself, where no such window was made, or where the host
/// gave orb no `MoveWindow` to put it back with.
///
/// # Safety
/// Must run on the thread the game's window belongs to.
pub unsafe fn keeps_its_corner() {
    if !unsafe { *OUT_OF_THE_WAY.get() } {
        return;
    }
    let window = PLAYED_IN.load(Ordering::Relaxed);
    let putting_back = MOVE_WINDOW.load(Ordering::Relaxed);
    if window == 0 || putting_back == 0 {
        return;
    }
    let Some(corner) = the_leftmost_of(&orb_api::window::monitors()) else {
        return;
    };
    let Some(is_at) = orb_api::window::window_rect(Hwnd(window)) else {
        return;
    };
    // **And whether something else has shown it**, which is what says the wait above is worth anything:
    // orb made this window hidden, so a window that is up before orb showed it is a window somebody else
    // put on the screen — and every put-back below it is a move somebody can see.
    if SHOWN.load(Ordering::Relaxed) != window
        && orb_api::window::is_visible(Hwnd(window))
        && !WAS_SHOWN_BY_SOMETHING_ELSE.swap(true, Ordering::Relaxed)
    {
        log!(
            "screen: the window orb plays in is on the screen and orb did not show it; \
             whatever moves it now is seen",
        );
    }
    // **The corner and the size both**, the size being [`OUT_OF_THE_WAY_CLIENT`] inside whatever frame the
    // window has. Where both are already right there is nothing to do, which is every frame but the few
    // something moved it on — except the first of them, which is where a window made hidden is shown.
    let (width, height) = the_size_it_plays_at(window, is_at);
    if (is_at.left, is_at.top, is_at.width(), is_at.height())
        == (corner.left, corner.top, width, height)
    {
        shown_now_the_corner_is_held(window);
        return;
    }
    detail!(
        "screen: the window orb plays in is at {},{} sized {}x{}; putting it back to {},{} sized {}x{}",
        is_at.left,
        is_at.top,
        is_at.width(),
        is_at.height(),
        corner.left,
        corner.top,
        width,
        height,
    );
    HELD_FOR.store(0, Ordering::Relaxed);
    let original: MoveWindow = unsafe { std::mem::transmute(putting_back) };
    unsafe {
        original(
            window as *mut c_void,
            corner.left,
            corner.top,
            width,
            height,
            1,
        )
    };
}

/// **The whole window whose client is [`OUT_OF_THE_WAY_CLIENT`]**, measured off the window as it is: the
/// frame it has around the client it has now, around the client wanted. The size it is at where the host
/// will not say what its client is.
///
/// **And the size it is at for a game that draws at the size of its buffers**, which is th06nc — see
/// `Game::draws_at_the_size_it_asked_for`. That one is made small by its own window mode, which lays its
/// frame out for the size it chose; held small from here as well, its picture was a quarter of the window
/// with its buffers left large and a null texture read at `0x140258665` with them made small.
fn the_size_it_plays_at(window: usize, is_at: Rect) -> (i32, i32) {
    if !unsafe { crate::runtime::the_game_draws_at_the_size_it_asked_for() } {
        return (is_at.width(), is_at.height());
    }
    match orb_api::window::client_rect(Hwnd(window)) {
        Some(client) if client.width() > 0 && client.height() > 0 => (
            is_at.width() - client.width() + OUT_OF_THE_WAY_CLIENT.0,
            is_at.height() - client.height() + OUT_OF_THE_WAY_CLIENT.1,
        ),
        _ => (is_at.width(), is_at.height()),
    }
}

/// **Shows the window orb plays in once it has been in its corner for
/// [`FRAMES_THE_CORNER_HOLDS_FIRST`] frames** — the window [`create_window_ex_w`] made hidden, and once
/// per window it made.
///
/// Nothing on every frame after that one: the count climbs and the handle shown is written down, so a
/// game making its window over shows the new one and a window already up is left alone.
fn shown_now_the_corner_is_held(window: usize) {
    let held = HELD_FOR.load(Ordering::Relaxed);
    if held >= FRAMES_THE_CORNER_HOLDS_FIRST {
        return;
    }
    HELD_FOR.store(held + 1, Ordering::Relaxed);
    if held + 1 < FRAMES_THE_CORNER_HOLDS_FIRST || SHOWN.swap(window, Ordering::Relaxed) == window {
        return;
    }
    log!(
        "screen: the window orb plays in has held its corner for \
         {FRAMES_THE_CORNER_HOLDS_FIRST} frames; showing it",
    );
    // **Without activating it**, for [`OUT_OF_THE_WAY_EX_STYLE`]'s reason: `SW_SHOW` activates the window it
    // shows, the style or no style, and the focus went with it.
    if !orb_api::window::show_without_activating(Hwnd(window)) {
        log!("screen: the window orb plays in could not be shown");
    }
}

/// **Whether the window is one orb plays in**, which is what [`answers_the_window_made`] was told — for a
/// game that sizes its own window afterwards and has to be told to keep it small.
///
/// # Safety
/// Must run on the game's main thread, after [`answers_the_window_made`].
pub unsafe fn plays_out_of_the_way() -> bool {
    unsafe { *OUT_OF_THE_WAY.get() }
}

/// **Frames the window orb plays in as a dialog again where something has taken that frame off**, asked
/// once a frame beside [`keeps_its_corner`] — for a game that makes its own window over after
/// [`create_window_ex_w`] framed it, whose frame is an ordinary one, in the taskbar and in alt-tab.
///
/// **Read rather than written every frame, and read for the same reason the corner is.** Both games do
/// this and neither does it through a call orb answers: th06c's window was measured with an extended
/// style of `0x100` where orb had made it with [`OUT_OF_THE_WAY_EX_STYLE`] — `dialog-frame False
/// off-the-taskbar False`, read from outside the process by
/// `.claude/skills/driving-orb-with-autoplay/scripts/window-style.ps1` — so the game had written its own
/// style over orb's some time after the window was made. A put-back on one chosen frame is a put-back
/// that can happen before the game's own write: this asks every frame and writes only where the bits are
/// gone, which costs one `GetWindowLongPtrW` a frame.
///
/// Nothing where the launch is somebody's own, where no such window was made, or where the host will not
/// say what the style is.
///
/// # Safety
/// Must run on the game's main thread, with the window made.
pub unsafe fn keeps_its_frame() {
    if !unsafe { *OUT_OF_THE_WAY.get() } {
        return;
    }
    let window = PLAYED_IN.load(Ordering::Relaxed);
    if window == 0 {
        return;
    }
    let has = orb_api::window::ex_style(Hwnd(window));
    if has & OUT_OF_THE_WAY_EX_STYLE == OUT_OF_THE_WAY_EX_STYLE {
        return;
    }
    detail!(
        "screen: the window orb plays in has the extended style {has:#010x}; \
         framing it as a dialog again",
    );
    if !orb_api::window::add_ex_style(Hwnd(window), OUT_OF_THE_WAY_EX_STYLE) {
        log!("screen: the window orb plays in could not be framed as a dialog again");
    }
}

/// Says which function [`create_window_ex_w`] calls through, and whether this launch is one orb plays
/// the game in.
///
/// **That second thing is said here and not by [`settle`]**, which is the other place it is said: a game
/// that looks its window creation up is a game whose `RegisterClassA` is not in an import table either,
/// so the installation `settle` is part of never runs. What a launch like that settles is the corner and
/// nothing else — there is no size, no ratio and no letterbox for it to be laying out.
///
/// # Safety
/// Must run before the game has made its window.
pub unsafe fn answers_the_window_made(original: usize, out_of_the_way: bool) {
    CREATE_WINDOW_EX_W.store(original, Ordering::Relaxed);
    unsafe { *OUT_OF_THE_WAY.get() = out_of_the_way };
    the_focus_is_where_it_was(out_of_the_way);
}

/// Says which function [`create_window_ex_a`] calls through, and settles the window it is to make.
///
/// What `orb::window::install` does above this and a laid-out game cannot is patch an import table — so
/// the original is handed over the way every other call into such a game is, and the rewrite is reached by
/// that game calling [`create_window_ex_a`] itself — the same argument as the frame loop's own two calls.
///
/// # Safety
/// `original` must be the game's own `CreateWindowExA` and outlive the last window it makes, and this must
/// run on the thread the game's frames will run on and before it creates that window.
pub unsafe fn install_over(
    original: CreateWindowExA,
    content: (u32, u32),
    screen: Screen,
    out_of_the_way: bool,
) {
    unsafe { settle(content, screen, out_of_the_way) };
    CREATE_WINDOW_EX_A.store(original as usize, Ordering::Relaxed);
    ENABLED.store(true, Ordering::Relaxed);
}

/// Creates the game's window the size the settings say, in place of the size the game asked
/// for.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where the
/// real game's own `CreateWindowExA` call lands, and there is no import table to reach it through.
///
/// # Safety
/// The arguments are `CreateWindowExA`'s own, and an [`install_over`] must have run first — without one
/// the original this calls through is null.
#[allow(clippy::too_many_arguments)]
pub unsafe extern "system" fn create_window_ex_a(
    ex_style: u32,
    class_name: *const u8,
    window_name: *const u8,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: *mut c_void,
    menu: *mut c_void,
    instance: *mut c_void,
    param: *const c_void,
) -> *mut c_void {
    let original: CreateWindowExA =
        unsafe { std::mem::transmute(CREATE_WINDOW_EX_A.load(Ordering::Relaxed)) };
    // The class first, so a window that is not the game's is not the reason the monitor was read: the
    // game makes others, and orb leaves those the size they asked for.
    //
    // **And which monitor depends on which window this is to be.** A window somebody plays in goes on the
    // primary monitor, which is the one the settings are about; a window orb plays in goes into the corner
    // of the leftmost, which is where a run nobody is watching belongs — see [`placed_out_of_the_way`].
    let monitor = unsafe { is_game_class(class_name) }.then(|| {
        if unsafe { *OUT_OF_THE_WAY.get() } {
            the_leftmost_of(&orb_api::window::monitors())
        } else {
            orb_api::window::primary_monitor()
        }
    });
    let monitor = monitor.flatten();
    let Some(monitor) = monitor else {
        return unsafe {
            original(
                ex_style,
                class_name,
                window_name,
                style,
                x,
                y,
                width,
                height,
                parent,
                menu,
                instance,
                param,
            )
        };
    };
    // A window orb plays in is not the settings': see `placed_out_of_the_way`, which is where that is
    // argued, and `Config::autoplay`, which is what asks for one.
    let wanted = if unsafe { *OUT_OF_THE_WAY.get() } {
        placed_out_of_the_way(monitor)
    } else {
        placed(monitor, unsafe { screen() })
    };

    // **And made hidden where orb plays in it, to be shown without being activated**: made visible, a launch's window
    // took the focus off whatever somebody was working in the moment it was made, with [`OUT_OF_THE_WAY_EX_STYLE`]'s
    // `WS_EX_NOACTIVATE` on it and the game's own `SetForegroundWindow` swallowed — a new process's first visible
    // window being one Windows brings to the front however it is styled.
    let out_of_the_way = unsafe { *OUT_OF_THE_WAY.get() };
    let style = match out_of_the_way {
        true => wanted.style & !WS_VISIBLE,
        false => wanted.style,
    };
    let window = unsafe {
        original(
            // Added to the game's own rather than written over it: what the game asked for is the game's
            // business and the only placement that asks for any of this is the one orb plays in.
            ex_style | wanted.ex_style,
            class_name,
            window_name,
            style,
            wanted.area.left,
            wanted.area.top,
            wanted.area.width(),
            wanted.area.height(),
            parent,
            menu,
            instance,
            param,
        )
    };
    if !window.is_null() {
        window_created(Hwnd(window as usize));
        if out_of_the_way && !orb_api::window::show_without_activating(Hwnd(window as usize)) {
            log!("screen: the window orb plays in could not be shown");
        }
        // The client it came out with as well as the window that was asked for, because those are
        // two different numbers whenever anything between here and the screen has an opinion —
        // display scaling, a shell that remembers window sizes — and the client is the one the
        // letterbox and the bar beside it are worked out from.
        log!(
            "screen: {} — window at {},{} sized {}x{}, client {}",
            unsafe { screen() },
            wanted.area.left,
            wanted.area.top,
            wanted.area.width(),
            wanted.area.height(),
            match game_client() {
                Some((width, height)) => format!("{width}x{height}"),
                None => "unknown".to_owned(),
            },
        );
    }
    window
}

/// What both of `orb::window`'s installations say before any window exists: the ratio to keep, how much
/// of the screen the game gets, and that this process reads sizes as the monitor's own pixels.
///
/// # Safety
/// Must run on the thread the game's frames will run on, and before the game creates its window.
pub unsafe fn settle(content: (u32, u32), screen: Screen, out_of_the_way: bool) {
    // Before the window exists, which is the only moment this can be said. Without it every
    // size below is a size Windows scales behind the game's back: a 1280x720 window asked for on
    // a monitor at 150% is 1920x1080 of screen, and the monitor a fullscreen window is measured
    // against reads as two thirds of itself. The game never asked to be told about scaling, so
    // orb asks for it.
    //
    // And before the monitor is read rather than merely before the window is made, which is the part an
    // e2e test holds: `orb_sim::Windows::monitor_reads` writes down whether the process had said this
    // when each read was answered, and a read on the wrong side of it lays the window out against
    // two thirds of the panel.
    let told = orb_api::window::set_process_dpi_aware();
    log!(
        "screen: display scaling {}",
        if told {
            "is being ignored, sizes are real pixels"
        } else {
            "could not be turned off; sizes are whatever Windows scales them to"
        }
    );
    unsafe {
        *CONTENT.get() = content;
        *SCREEN.get() = screen;
        *OUT_OF_THE_WAY.get() = out_of_the_way;
    }
    the_focus_is_where_it_was(out_of_the_way);
}

/// **The window that was in front when orb settled the window it is to play in**, which [`keeps_the_focus_where_it_was`]
/// hands the focus back to — nought where somebody plays, or where nothing was.
static FOCUS_WAS: AtomicUsize = AtomicUsize::new(0);

/// Writes down the window in front, for a launch orb plays — before the game has a window of its own to take it.
fn the_focus_is_where_it_was(out_of_the_way: bool) {
    let was = match out_of_the_way {
        true => orb_api::window::foreground().0,
        false => 0,
    };
    FOCUS_WAS.store(was, Ordering::Relaxed);
}

/// **The focus handed back to the window that had it**, the frame the window orb plays in is seen in front — whatever
/// brought it there.
///
/// **Beside what keeps it from coming forward at all**, and not instead of it: [`OUT_OF_THE_WAY_EX_STYLE`] keeps the
/// window from a click and from the system, [`set_foreground_window`] from the game's own call, and
/// [`create_device9`] from a Direct3D 9 device made on it — and that is th10's three. Direct3D 8 has no
/// `D3DCREATE_NOWINDOWCHANGES`, and th06c's and th06nc's renderer is another again, so what is left for a game whose
/// window comes forward by a way orb has no answer for is to give the focus back as soon as it is seen gone. A frame
/// of it is the cost, which is a keystroke or two somebody typed going to the game.
///
/// Nothing where somebody plays, before the game has a window, or where the window in front is not orb's.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn keeps_the_focus_where_it_was() {
    let was = FOCUS_WAS.load(Ordering::Relaxed);
    if !unsafe { *OUT_OF_THE_WAY.get() } || was == 0 {
        return;
    }
    let front = orb_api::window::foreground().0;
    let ours = [
        GAME_WINDOW.load(Ordering::Relaxed),
        PLAYED_IN.load(Ordering::Relaxed),
    ];
    if front == 0 || front == was || !ours.contains(&front) {
        return;
    }
    log!(
        "screen: the window orb plays in came to the front; the focus handed back to the window that had it"
    );
    if !orb_api::window::bring_forward(Hwnd(was)) {
        log!("screen: the focus could not be handed back");
    }
}

/// How much of the screen the game is to get, which is what the rewrite of `CreateWindowExA` lays its
/// window out from.
///
/// # Safety
/// Must run on the game's main thread, after [`settle`].
pub unsafe fn screen() -> Screen {
    unsafe { *SCREEN.get() }
}

/// **The setting alone, for a game whose window orb does not lay out** — one that looks `CreateWindowExW`
/// up by name and so has no import for [`settle`]'s installation to patch. Nothing of the ratio, the
/// letterbox or the DPI is settled by this, since none of it would be orb's: what it leaves is the answer
/// to [`screen`], for a `Game` that sizes its own window off its own settings to read.
///
/// # Safety
/// Must run on the game's main thread, before its first frame.
pub unsafe fn set_screen(screen: Screen) {
    unsafe { *SCREEN.get() = screen };
}

/// Says which window the game got, which the rewrite of `CreateWindowExA` calls with what it made.
pub fn window_created(window: Hwnd) {
    GAME_WINDOW.store(window.0, Ordering::Relaxed);
}

/// The client area of the window the game got, for the line that says what is being presented into a
/// letterbox — `None` before there is a window, or for one the host will not measure.
pub fn game_client() -> Option<(i32, i32)> {
    client_size(game_window())
}

fn game_window() -> Hwnd {
    Hwnd(GAME_WINDOW.load(Ordering::Relaxed))
}

/// The client area of `window`, which is what the game draws into and what orb works its
/// letterbox out from.
fn client_size(window: Hwnd) -> Option<(i32, i32)> {
    client_area(window).map(|client| (client.width(), client.height()))
}

fn client_area(window: Hwnd) -> Option<Rect> {
    orb_api::window::client_rect(window)
}

/// The rectangle inside the client area that the game's output belongs in,
/// recomputed only when the client area changes.
///
/// `pub` for an e2e test that says how much black there is beside the game: this is where the 320
/// pixels either side of a 4:3 game on a 2560x1440 client are decided, and nothing else in orb says
/// so — `orb::window`'s replacement of `Present` hands the rectangle straight to Direct3D and the bar
/// beside it is written through the GDI.
///
/// # Safety
/// Must run on the thread the game's frames run on.
pub unsafe fn letterbox() -> Option<Rect> {
    let window = game_window();
    if window.is_null() {
        return None;
    }
    let client = client_area(window)?;

    let cached = unsafe { DESTINATION.get() };
    if let Some((known_client, destination)) = cached
        && *known_client == client
    {
        return Some(*destination);
    }
    let destination = fit(client, unsafe { *CONTENT.get() });
    *cached = Some((client, destination));
    Some(destination)
}

/// The slot's own signature, which is what a replacement has to have and what the original is called
/// through.
///
/// The device is a `*mut c_void` and the window an `isize`: the object is the game's, and nothing here
/// looks inside one — what is read out of it, the vtable pointer, goes through [`orb_api::mem`]. The two
/// rectangles are [`Rect`], which is `#[repr(C)]` and held against Windows' own `RECT` field by field in
/// `orb::window`, that being the last place in the tree which names one.
type Present = unsafe extern "system" fn(
    *mut c_void,
    *const Rect,
    *const Rect,
    isize,
    *const c_void,
) -> Hresult;

/// Redirects the device's `Present` so the back buffer lands in a rectangle of
/// the game's aspect ratio rather than being stretched over the whole window.
///
/// A hook body like the eleven in [`crate::runtime`], and the swap it makes is
/// [`orb_api::mem::replace_word`] — the page a vtable is in being read-only is the host's business and
/// not this decision's. What is decided here is whether a launch wants a letterbox at all, that a second
/// call must not patch twice, and that a client of that size is being presented into one.
///
/// # Safety
/// `device` must be the game's live device, and this must run before it presents.
pub unsafe fn hook_device(device: orb_api::Device) {
    if !ENABLED.load(Ordering::Relaxed) || PRESENT.load(Ordering::Relaxed) != 0 {
        return;
    }
    // The vtable read out of the object rather than dereferenced: a COM object is a pointer to its
    // vtable at offset zero, and the read goes through the seam the way every other read of the game's
    // memory does — which is also what the swap below writes the slot through.
    let vtable = unsafe { orb_api::mem::read::<usize>(device.0) };
    let slot = vtable + orb_api::d3d8::PRESENT_SLOT * size_of::<usize>();
    // Through a pointer rather than straight to an integer: a function *item* is a type of its own with
    // no address until it is coerced to a pointer, so `present as usize` would ask for the address of a
    // thing that does not have one yet and get it by an implied coercion.
    let replacement = present as *const () as usize;
    // **The game's own kept before the slot is written over**, and not out of what the write answers: a present
    // through the slot between the two would reach orb's with nothing to hand on to.
    PRESENT.store(
        unsafe { orb_api::mem::read::<usize>(slot) },
        Ordering::Relaxed,
    );
    match unsafe { orb_api::mem::replace_word(slot, replacement) } {
        Some(_) => {
            // With the client as it is now: the device has just been created, and creating one is
            // the other thing that can resize a window out from under whoever asked for it.
            log!(
                "screen: presenting through a letterbox, client {}",
                match game_client() {
                    Some((width, height)) => format!("{width}x{height}"),
                    None => "unknown".to_owned(),
                }
            );
        }
        None => {
            // And nothing kept where nothing was swapped, which is what says the letterbox is not in.
            PRESENT.store(0, Ordering::Relaxed);
            log!("screen: cannot hook Present: the slot's page cannot be made writable");
        }
    }
}

/// What goes in that slot: the game's own present with the destination rectangle it did not ask for.
///
/// # Safety
/// The arguments are `IDirect3DDevice8::Present`'s own, and [`hook_device`] must have put the original
/// this calls through in place first.
unsafe extern "system" fn present(
    device: *mut c_void,
    source: *const Rect,
    destination: *const Rect,
    window_override: isize,
    dirty: *const c_void,
) -> Hresult {
    let original: Present = unsafe { std::mem::transmute(PRESENT.load(Ordering::Relaxed)) };
    let letterbox = unsafe { letterbox() };
    let Some(letterbox) = letterbox else {
        return unsafe { original(device, source, destination, window_override, dirty) };
    };

    let started = profile::now();
    let result = unsafe { original(device, std::ptr::null(), &letterbox, window_override, dirty) };
    unsafe { profile::record(profile::Phase::Present, started) };
    if result < 0 {
        // Some drivers may refuse to stretch on a present. Falling back to the
        // game's own call keeps it playable, just stretched.
        report_letterbox_failure(result);
        return unsafe { original(device, source, destination, window_override, dirty) };
    }
    result
}

fn report_letterbox_failure(result: Hresult) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        log!("screen: Present into a letterbox failed ({result:#x}); stretching instead");
    }
}

/// The lines waiting for the frame loop's slack, which is the whole of why there are two calls below
/// rather than the one.
///
/// Painting is GDI on the window itself and a real host takes milliseconds over it — measured at 4275µs
/// at the median and 15237µs at the worst on a handheld — while the numbers change every
/// `HUD_NUMBER_INTERVAL` frames. So a paint where the lines are worked out is a frame in thirty that
/// cannot reach its blank, and the budget the frame after it is started against rises to what that frame
/// took: `next_budget` climbs at once and falls a sixty-fourth at a time, so a paint every thirtieth frame
/// holds it near the paint for the whole run, and that budget is lag on every frame rather than on the
/// ones that painted. Measured on th06: two per cent of frames a refresh late, and a budget of 4.1 to
/// 7.5ms against a game whose own work is 2.9.
///
/// Which is what the log did with its lines until they went to a thread of their own — see [`log`], whose
/// note has the write that made a thread of it. A paint cannot follow them there: GDI on this window is the
/// frame's own thread's to call, so the slack is the whole of what there is for it, and a paint every
/// thirtieth frame fits in one where a write of unknown length does not.
static HELD: MainThread<Option<Vec<String>>> = MainThread::new(None);

/// Holds a stack of lines for the slack, replacing whatever was waiting.
///
/// Replacing rather than queueing: what is wanted on the screen is the newest reading, and a stack that
/// went stale before it was ever painted is one nobody needed to see.
///
/// # Safety
/// Must run on the thread that owns the window.
pub unsafe fn hold_beside(lines: Vec<String>) {
    *unsafe { HELD.get() } = Some(lines);
}

/// Paints whatever is held, which is where the milliseconds go.
///
/// # Safety
/// Must run on the thread that owns the window, and outside a scene.
pub unsafe fn paint_held() {
    let Some(lines) = unsafe { HELD.get() }.take() else {
        return;
    };
    unsafe { write_beside(&lines) };
}

/// Writes `lines` in the black beside the game, where the window class paints it.
///
/// Drawn straight onto the window rather than into the game's back buffer, because the
/// back buffer is 640x480 and all of it is shown inside the letterbox — anything put
/// there appears over the game, and the game does not clear it between frames so it also
/// stays there. The black around the game is the window's own background and Direct3D
/// never touches it, so this is the one part of the window that is orb's to draw in.
///
/// Whether that black is to the sides or above and below depends on the monitor: a 4:3
/// game on a 16:9 screen is bars down the sides, which is why the lines are stacked
/// rather than run together.
///
/// Only when the text changes. What it then paints goes back to black first, all of it,
/// which is what keeps a shorter stack or a smaller font from leaving part of the line before
/// it on the screen.
///
/// # Safety
/// Must run on the thread that owns the window, and outside a scene.
pub unsafe fn write_beside(lines: &[String]) {
    static SHOWN: MainThread<Vec<String>> = MainThread::new(Vec::new());
    /// The rows the last call painted, so this one can clear them whether or not it writes
    /// anything there itself.
    static PAINTED: MainThread<Option<Rect>> = MainThread::new(None);
    // **Only where the picture is presented into the letterbox**, the swap of `Present` having gone in: a game
    // whose device orb does not hook — th10's Direct3D 9 — has its back buffer stretched over the whole client,
    // so the rectangle [`letterbox`] works out has no black beside it, and lines written there were over th10's
    // picture, covered by its next present and back on the one after.
    if PRESENT.load(Ordering::Relaxed) == 0 {
        return;
    }
    let window = game_window();
    let letterbox = unsafe { letterbox() };
    let Some(letterbox) = letterbox else { return };
    if window.is_null() {
        return;
    }
    let Some(client) = client_area(window) else {
        return;
    };

    // Whichever bar is wider, and in it the corner nearest the game: lined up with the
    // game's own edge rather than the monitor's, so the numbers read as belonging to what
    // they are about.
    let beside = client.right - letterbox.right;
    let below = client.bottom - letterbox.bottom;
    let bottom = client.bottom - BAR_MARGIN;
    // Down the side: the text runs away from the game, starting at its edge. Right-
    // aligning on that edge instead would put it over the game, which is the one thing
    // this is here to avoid.
    let (strip, x, align) = if beside >= below {
        (
            Rect {
                left: letterbox.right,
                top: 0,
                right: client.right,
                bottom: 0,
            },
            letterbox.right + BAR_MARGIN,
            Bar::LEFT,
        )
    } else {
        (
            Rect {
                left: client.left,
                top: 0,
                right: client.right,
                bottom: 0,
            },
            letterbox.right,
            Bar::RIGHT,
        )
    };
    if beside.max(below) < BAR_TEXT_MIN {
        report_no_bar(client, letterbox);
        return;
    }
    // What there is to write in, from where the text starts to the far edge of the bar. The
    // side bar runs away from the game and the strip under it runs back towards the game's
    // own edge, so the room is on opposite sides of the same x.
    let room = if align == Bar::LEFT {
        strip.right - x
    } else {
        x - strip.left
    };

    // How far up the black goes. The strip under the game runs the full width of the window,
    // so a stack taller than that strip would be painted over the game's own output; beside
    // the game the whole height is orb's.
    let limit = if beside >= below {
        client.top
    } else {
        letterbox.bottom
    };

    let shown = unsafe { SHOWN.get() };
    if shown == lines {
        return;
    }

    let height = fitting_font(lines, room);
    let block = Rect {
        left: strip.left,
        top: (bottom - height * lines.len() as i32).max(limit),
        right: strip.right,
        bottom,
    };
    let painted = unsafe { PAINTED.get() };
    let bar = Bar {
        // What the last call painted goes with it, since a stack of fewer lines or a smaller
        // font does not reach the rows the one before it wrote in.
        area: painted.map_or(block, |last| union(last, block)),
        x,
        bottom,
        height,
        align,
    };
    let drawn = orb_api::window::write_lines(window, bar, lines);
    // Only once it is on the screen: a draw that failed has left the bar holding whatever it
    // held, and the next call should be the one that puts it right rather than deciding
    // there is nothing to do.
    if drawn {
        *painted = Some(block);
        shown.clear();
        shown.extend_from_slice(lines);
    }
}

fn union(a: Rect, b: Rect) -> Rect {
    Rect {
        left: a.left.min(b.left),
        top: a.top.min(b.top),
        right: a.right.max(b.right),
        bottom: a.bottom.max(b.bottom),
    }
}

/// The em height these lines fit `room` pixels at.
///
/// Measured rather than guessed, because how wide a line comes out is the font's business:
/// what is written here runs from three characters to twenty, and the widest — a chapter
/// named for the part of a stage it belongs to — is the one that decides. Clipped at the
/// bar's edge it cannot be read at all, which is worse than small.
///
/// One measurement: character widths go with the em height closely enough that scaling by the ratio
/// lands inside a pixel or two, and the caller only gets here when the lines have changed.
fn fitting_font(lines: &[String], room: i32) -> i32 {
    let (width, _) = orb_api::window::measure_lines(lines, BAR_TEXT_HEIGHT);
    if width <= room || width <= 0 || room <= 0 {
        return BAR_TEXT_HEIGHT;
    }

    let height = (BAR_TEXT_HEIGHT * room / width).max(BAR_TEXT_MIN);
    static SAID: AtomicIsize = AtomicIsize::new(0);
    if SAID.swap(height as isize, Ordering::Relaxed) != height as isize {
        log!("screen: {width}px of text in {room}px of black, writing at {height}px");
    }
    height
}

/// Said once: with the game filling the window there is nowhere beside it to write, and
/// that is a fact about the monitor rather than a fault.
fn report_no_bar(client: Rect, letterbox: Rect) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        log!(
            "screen: client {}x{}, game {}x{} at {},{} — no black to write in",
            client.width(),
            client.height(),
            letterbox.width(),
            letterbox.height(),
            letterbox.left,
            letterbox.top,
        );
    }
}

/// A window to create: what kind it is, and the whole of it — frame included — in the monitor's
/// own coordinates.
pub struct Placed {
    pub style: u32,
    /// The extended style, which is nothing for a window somebody plays in — see
    /// [`placed_out_of_the_way`], the one placement that asks for any.
    pub ex_style: u32,
    pub area: Rect,
}

/// `WS_EX_DLGMODALFRAME | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, written out for [`BORDERLESS_STYLE`]'s reason:
/// what an extended style is is the game's own `CreateWindowExA` argument.
///
/// The dialog frame is what a window orb plays in is asked to look like; the tool window beside it is what
/// keeps it off the taskbar and out of alt-tab, which is what *not being in the way* takes and the half to
/// drop if a window nobody can alt-tab to turns out to be worse. **And the window never activated**, by its
/// own showing or by a click on it: a launch that plays itself is started on the machine somebody is working
/// on, and every launch of a bench that took the focus took it off what they were typing into.
pub const OUT_OF_THE_WAY_EX_STYLE: u32 = 0x0000_0001 | 0x0000_0080 | 0x0800_0000;

/// The client a window orb plays in is given: half of a 640x480 game, which is small enough to sit in a
/// corner of somebody's screen and big enough to see what the bot is doing to the run.
const OUT_OF_THE_WAY_CLIENT: (i32, i32) = (320, 240);

/// **Where a window orb plays the game in goes**: that client, in the monitor's top-left corner, framed as
/// a dialog and kept off the taskbar.
///
/// Whatever the settings say, and that is the point: `Config::autoplay` is a run nobody is watching, played
/// on the machine somebody is working on. Fullscreen there would be orb taking the screen for a run that is
/// not theirs.
///
/// **Against the corner rather than inset from it**, which the top-left corner allows and the middle of the
/// screen does not: a window placed by its client area's own corner has its caption above that corner, and
/// a caption off the top of the screen cannot be dragged back on — see [`centred`]. The rectangle asked for
/// here is the whole window, frame and caption included, so the caption is on the screen.
///
/// Which monitor's corner is [`the_leftmost_of`]'s.
pub fn placed_out_of_the_way(monitor: Rect) -> Placed {
    let client = Rect::sized(OUT_OF_THE_WAY_CLIENT.0, OUT_OF_THE_WAY_CLIENT.1);
    // A failure leaves the rectangle as the client area, which is a window a frame too small rather than no
    // window at all — `placed`'s own answer to the same call failing.
    let area = orb_api::window::adjust_window_rect(client, WINDOWED_STYLE, false).unwrap_or(client);
    Placed {
        style: WINDOWED_STYLE,
        ex_style: OUT_OF_THE_WAY_EX_STYLE,
        area: Rect {
            left: monitor.left,
            top: monitor.top,
            right: monitor.left + area.width(),
            bottom: monitor.top + area.height(),
        },
    }
}

/// **The monitor furthest to the left of the desktop**, which is the one a window orb plays in goes into the
/// corner of — the leftmost by how the monitors are arranged and not by which the game came up on, since
/// that is what somebody looking at their own screens means by *the left*.
///
/// **The topmost of the leftmost where two share an edge.** Two monitors stacked one above the other have
/// the same `left`, and a corner of the desktop is one place: the upper of them is the one that holds it.
///
/// `None` where the host names no monitor, which leaves the window as the game made it — the same answer
/// `primary_monitor` answering nothing gets.
pub fn the_leftmost_of(monitors: &[Rect]) -> Option<Rect> {
    monitors
        .iter()
        .min_by_key(|monitor| (monitor.left, monitor.top))
        .copied()
}

/// Where the game's window goes.
///
/// The size in the settings is the size of what is *inside* the window, so that `1280x720` is
/// 1280x720 of game however thick this machine's window frames are; `AdjustWindowRect` is what
/// turns that into the window to ask for.
pub fn placed(monitor: Rect, screen: Screen) -> Placed {
    let Screen::Window { width, height } = screen else {
        return Placed {
            style: BORDERLESS_STYLE,
            ex_style: 0,
            area: monitor,
        };
    };
    let client = Rect::sized(width as i32, height as i32);
    // A failure leaves the rectangle as the client area, which is a window a frame too small
    // rather than no window at all.
    let area = orb_api::window::adjust_window_rect(client, WINDOWED_STYLE, false).unwrap_or(client);
    Placed {
        style: WINDOWED_STYLE,
        ex_style: 0,
        area: centred(monitor, area),
    }
}

/// A rectangle of that size in the middle of the monitor, and against its top-left corner if it
/// is too big to be in the middle of it — a window whose caption is off the top of the screen
/// cannot be moved back on.
pub fn centred(monitor: Rect, area: Rect) -> Rect {
    let (width, height) = (area.width(), area.height());
    let left = monitor.left + ((monitor.width() - width) / 2).max(0);
    let top = monitor.top + ((monitor.height() - height) / 2).max(0);
    Rect {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

/// The largest rectangle with `content`'s aspect ratio that fits `client`, centred.
pub fn fit(client: Rect, content: (u32, u32)) -> Rect {
    let available_width = i64::from(client.width());
    let available_height = i64::from(client.height());
    let wanted_width = i64::from(content.0.max(1));
    let wanted_height = i64::from(content.1.max(1));

    // Whichever axis runs out first sets the scale. All integer, so the result is
    // exactly the game's ratio rather than nearly it.
    let (width, height) = if available_width * wanted_height <= available_height * wanted_width {
        (
            available_width,
            available_width * wanted_height / wanted_width,
        )
    } else {
        (
            available_height * wanted_width / wanted_height,
            available_height,
        )
    };

    let left = client.left + ((available_width - width) / 2) as i32;
    let top = client.top + ((available_height - height) / 2) as i32;
    Rect {
        left,
        top,
        right: left + width as i32,
        bottom: top + height as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::{centred, fit, the_leftmost_of};
    use orb_api::Rect;

    fn client(width: i32, height: i32) -> Rect {
        Rect::sized(width, height)
    }

    fn monitor(left: i32, top: i32, width: i32, height: i32) -> Rect {
        Rect {
            left,
            top,
            right: left + width,
            bottom: top + height,
        }
    }

    /// **The leftmost monitor is the leftmost of the arrangement**, which for a monitor to the left of the
    /// primary one is a negative `left` and not the first the host happened to name.
    #[test]
    fn the_window_orb_plays_in_goes_on_the_leftmost_monitor() {
        let primary = monitor(0, 0, 1920, 1080);
        let to_the_right = monitor(1920, 0, 2560, 1440);
        let to_the_left = monitor(-2560, -200, 2560, 1440);
        assert_eq!(
            the_leftmost_of(&[primary, to_the_right, to_the_left]),
            Some(to_the_left),
        );
        assert_eq!(the_leftmost_of(&[primary, to_the_right]), Some(primary));
        assert_eq!(the_leftmost_of(&[primary]), Some(primary));
    }

    /// And the upper of two that share that edge, a corner of the desktop being one place.
    #[test]
    fn two_monitors_stacked_on_the_left_edge_are_answered_by_the_upper_one() {
        let lower = monitor(-1920, 1080, 1920, 1080);
        let upper = monitor(-1920, 0, 1920, 1080);
        assert_eq!(the_leftmost_of(&[lower, upper]), Some(upper));
        assert_eq!(the_leftmost_of(&[upper, lower]), Some(upper));
    }

    /// And a host that names no monitor at all is answered with none, which leaves the window as the game
    /// made it.
    #[test]
    fn a_desktop_with_no_monitors_is_answered_with_none() {
        assert_eq!(the_leftmost_of(&[]), None);
    }

    /// A window goes in the middle of the monitor it is on, which for a second monitor is not
    /// the middle of anything measured from zero.
    #[test]
    fn a_window_is_centred_on_its_monitor() {
        let placed = centred(client(1920, 1080), client(1280, 760));
        assert_eq!(
            (placed.left, placed.top, placed.right, placed.bottom),
            (320, 160, 1600, 920)
        );
        let second = Rect {
            left: 1920,
            top: 0,
            right: 3840,
            bottom: 1080,
        };
        let placed = centred(second, client(1280, 760));
        assert_eq!((placed.left, placed.top), (2240, 160));
    }

    /// Against the corner rather than half off the top, since a caption above the screen cannot
    /// be dragged back onto it.
    #[test]
    fn a_window_too_big_for_the_monitor_starts_at_its_corner() {
        let placed = centred(client(800, 600), client(1280, 760));
        assert_eq!((placed.left, placed.top), (0, 0));
        assert_eq!((placed.right, placed.bottom), (1280, 760));
    }

    #[test]
    fn a_four_three_game_is_pillarboxed_on_a_sixteen_nine_screen() {
        let rect = fit(client(2560, 1440), (640, 480));
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (320, 0, 2240, 1440)
        );
    }

    #[test]
    fn a_game_wider_than_the_screen_is_letterboxed() {
        let rect = fit(client(1000, 1000), (640, 480));
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (0, 125, 1000, 875)
        );
    }

    #[test]
    fn a_matching_ratio_fills_the_screen() {
        let rect = fit(client(1600, 1200), (640, 480));
        assert_eq!(
            (rect.left, rect.top, rect.right, rect.bottom),
            (0, 0, 1600, 1200)
        );
    }
}
