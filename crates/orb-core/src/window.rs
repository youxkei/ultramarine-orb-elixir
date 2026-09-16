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
pub const BORDERLESS_STYLE: u32 = 0x8000_0000 | 0x1000_0000;
/// A window of a chosen size: a caption to move it by and a system menu to close it with, and nothing to
/// resize it with. The size is one of the settings, so dragging the edge of the window would be a second
/// place to say it and the one that is not written down.
///
/// `WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE`.
pub const WINDOWED_STYLE: u32 = 0x00c0_0000 | 0x0008_0000 | 0x0002_0000 | 0x1000_0000;

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
/// **[`DIALOG_EX_STYLE`] for the reason it is there at all**: a window a bot plays in that the window
/// manager treats as an ordinary one is a window in the way — in the taskbar, in alt-tab, and taking a
/// turn at being focused.
///
/// **But not the size, because this game's drawing is not orb's to lay out.** It presents through a
/// swapchain of its own, built for the window it asked for, and orb has no hold on that present: a
/// client of [`OUT_OF_THE_WAY_CLIENT`] clips the game rather than making it small, and the launch that
/// asked for one lost the corner as well — its window measured back in the middle of the primary
/// monitor at the swapchain's own size. So the size and the style are what the game's own startup
/// dialog was told, and `game::Dialog` is where the smallest window it offers is chosen.
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
    let window = unsafe {
        original(
            // Added to the game's own rather than written over it, the same as the other rewrite.
            ex_style | corner.map_or(0, |_| DIALOG_EX_STYLE),
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
        // Remembered so that [`keeps_its_corner`] has a window to put back, whatever moves it.
        PLAYED_IN.store(window as usize, Ordering::Relaxed);
        log!(
            "screen: the window orb plays in is at {into_x},{into_y} sized {width}x{height}, \
             which is the size the game asked for",
        );
    }
    window
}

/// The game's own `MoveWindow`, which it calls through — or the whole of it in a bot's launch.
static MOVE_WINDOW: AtomicUsize = AtomicUsize::new(0);

/// `BOOL MoveWindow(HWND, int x, int y, int cx, int cy, BOOL repaint)`.
pub type MoveWindow = unsafe extern "system" fn(*mut c_void, i32, i32, i32, i32, i32) -> i32;

/// **Holds the window orb plays in in its corner**, for a game that moves its own afterwards — and
/// leaves the size it is asking for alone.
///
/// **The corner and not the size, which is measured rather than chosen.** th06c's own move asks for
/// that same corner and a bigger window: it is applying the resolution its dialog was told, and its
/// swapchain is that size. A launch that answered the size too was measured with its window back in the
/// middle of the primary monitor at the swapchain's own size — the size is DXGI's to enforce, and
/// fighting it costs the corner as well.
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
    if corner.is_some() && (into_x, into_y) != (x, y) {
        detail!(
            "screen: the game moved its window to {x},{y}; it goes to {into_x},{into_y} instead"
        );
    }
    unsafe { original(window, into_x, into_y, width, height, repaint) }
}

/// Says which function [`move_window`] calls through.
///
/// # Safety
/// Must run before the game has moved its window.
pub unsafe fn answers_the_window_moved(original: usize) {
    MOVE_WINDOW.store(original, Ordering::Relaxed);
}

/// The window a game that looks its own creation up was given, so that [`keeps_its_corner`] has one to
/// put back.
static PLAYED_IN: AtomicUsize = AtomicUsize::new(0);

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
    // **The corner and not the size**, for [`create_window_ex_w`]'s reason: the size is the game's own
    // and a smaller one clips what it draws. Where the corner is already right there is nothing to do,
    // which is every frame but the few something moved it on.
    if (is_at.left, is_at.top) == (corner.left, corner.top) {
        return;
    }
    detail!(
        "screen: the window orb plays in is at {},{}; putting it back to {},{}",
        is_at.left,
        is_at.top,
        corner.left,
        corner.top,
    );
    let original: MoveWindow = unsafe { std::mem::transmute(putting_back) };
    unsafe {
        original(
            window as *mut c_void,
            corner.left,
            corner.top,
            is_at.width(),
            is_at.height(),
            1,
        )
    };
}

/// **Whether the window is one orb plays in**, which is what [`answers_the_window_made`] was told — for a
/// game that sizes its own window afterwards and has to be told to keep it small.
///
/// # Safety
/// Must run on the game's main thread, after [`answers_the_window_made`].
pub unsafe fn plays_out_of_the_way() -> bool {
    unsafe { *OUT_OF_THE_WAY.get() }
}

/// **Frames the window orb plays in as a dialog again**, for a game that made its window over after
/// [`create_window_ex_w`] framed it — th06nc's window mode does, and the frame it puts back is an
/// ordinary one, in the taskbar and in alt-tab. [`DIALOG_EX_STYLE`] is added to what the window has;
/// nothing where the launch is somebody's or the window is not made yet.
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
    if !orb_api::window::add_ex_style(Hwnd(window), DIALOG_EX_STYLE) {
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

    let window = unsafe {
        original(
            // Added to the game's own rather than written over it: what the game asked for is the game's
            // business and the only placement that asks for any of this is the one orb plays in.
            ex_style | wanted.ex_style,
            class_name,
            window_name,
            wanted.style,
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
    match unsafe { orb_api::mem::replace_word(slot, replacement) } {
        Some(original) => {
            PRESENT.store(original, Ordering::Relaxed);
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
        None => log!("screen: cannot hook Present: the slot's page cannot be made writable"),
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

/// `WS_EX_DLGMODALFRAME | WS_EX_TOOLWINDOW`, written out for [`BORDERLESS_STYLE`]'s reason: what an
/// extended style is is the game's own `CreateWindowExA` argument.
///
/// The dialog frame is what a window orb plays in is asked to look like; the tool window beside it is what
/// keeps it off the taskbar and out of alt-tab, which is what *not being in the way* takes and the half to
/// drop if a window nobody can alt-tab to turns out to be worse.
pub const DIALOG_EX_STYLE: u32 = 0x0000_0001 | 0x0000_0080;

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
        ex_style: DIALOG_EX_STYLE,
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
