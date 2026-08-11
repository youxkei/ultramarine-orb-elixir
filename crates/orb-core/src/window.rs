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
//! they start. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

use orb_api::{Bar, Hresult, Hwnd, Rect};
use orb_config::Screen;

use crate::sync::MainThread;
use crate::{log, profile};

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

/// Says which function [`create_window_ex_a`] calls through, and settles the window it is to make.
///
/// What `orb::window::install` does above this and a laid-out game cannot is patch an import table — so
/// the original is handed over the way every other call into such a game is, and the rewrite is reached by
/// that game calling [`create_window_ex_a`] itself. See
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md),
/// which is the same argument about the frame loop's own two calls.
///
/// # Safety
/// `original` must be the game's own `CreateWindowExA` and outlive the last window it makes, and this must
/// run on the thread the game's frames will run on and before it creates that window.
pub unsafe fn install_over(original: CreateWindowExA, content: (u32, u32), screen: Screen) {
    unsafe { settle(content, screen) };
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
    let monitor = unsafe { is_game_class(class_name) }
        .then(orb_api::window::primary_monitor)
        .flatten();
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
    let wanted = placed(monitor, unsafe { screen() });

    let window = unsafe {
        original(
            ex_style,
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
pub unsafe fn settle(content: (u32, u32), screen: Screen) {
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
/// ones that painted. Measured on 紅魔郷: two per cent of frames a refresh late, and a budget of 4.1 to
/// 7.5ms against a game whose own work is 2.9.
///
/// Which is [`log::defer`] and [`log::drain`] again, for the same reason and in the same shape — see the
/// note beside the drain in `frame::Pacing::wait_for_slot` for what the slack is and why a millisecond
/// spent there is spent out of nothing.
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
    pub area: Rect,
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
            area: monitor,
        };
    };
    let client = Rect::sized(width as i32, height as i32);
    // A failure leaves the rectangle as the client area, which is a window a frame too small
    // rather than no window at all.
    let area = orb_api::window::adjust_window_rect(client, WINDOWED_STYLE, false).unwrap_or(client);
    Placed {
        style: WINDOWED_STYLE,
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
    use super::{centred, fit};
    use orb_api::Rect;

    fn client(width: i32, height: i32) -> Rect {
        Rect::sized(width, height)
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
