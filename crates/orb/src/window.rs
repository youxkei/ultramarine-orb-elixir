//! How much of the screen the game gets, through the game's own window and device.
//!
//! The window is the size it is going to be from the moment it exists, because the arguments of
//! the game's own `CreateWindowExA` call are rewritten on the way through — there is no frame to
//! remove afterwards and nothing to flash on screen first. Fullscreen is that window borderless
//! and covering the monitor; a size is that window centred on it, with a caption and nothing to
//! drag, since the size is a setting rather than something to pull about. Its window class gets a
//! black background either way, and that is what fills the letterbox.
//!
//! The game is always made to create a window, whichever of the two it is: a game that has taken
//! the display exclusively has no window to size, and orb needs one to draw its own numbers
//! beside the game in.
//!
//! In windowed mode the game asks Direct3D for a `D3DSWAPEFFECT_COPY` swap chain,
//! and that is the swap effect that honours a destination rectangle on `Present`.
//! So the 640x480 back buffer is presented into a centred rectangle with the
//! game's aspect ratio, and the rest of the window keeps its black background.

use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

use orb_config::Screen;
use windows_sys::Win32::Foundation::{COLORREF, HWND, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    ANTIALIASED_QUALITY, BLACKNESS, BitBlt, CLIP_DEFAULT_PRECIS, CreateCompatibleBitmap,
    CreateCompatibleDC, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH, DeleteDC,
    DeleteObject, ExtTextOutW, FW_NORMAL, GetDC, GetMonitorInfoW, GetTextExtentPoint32W, HBRUSH,
    HDC, HFONT, HGDIOBJ, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
    OUT_DEFAULT_PRECIS, PatBlt, ReleaseDC, SRCCOPY, SelectObject, SetBkMode, SetTextAlign,
    SetTextColor, TA_LEFT, TA_RIGHT, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRect, GetClientRect, HMENU, SetProcessDPIAware, WNDCLASSA, WS_CAPTION,
    WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_VISIBLE,
};

use crate::d3d8::{Device, Hresult};
use crate::hook;
use crate::log::log;
use crate::profile;
use crate::sync::MainThread;

const BLACK: COLORREF = 0x0000_0000;
/// All a borderless window needs. Anything else — caption, frame, system menu —
/// is what puts a border on it.
const BORDERLESS_STYLE: u32 = WS_POPUP | WS_VISIBLE;
/// A window of a chosen size: a caption to move it by and a system menu to close it with, and
/// nothing to resize it with. The size is one of the settings, so dragging the edge of the
/// window would be a second place to say it and the one that is not written down.
const WINDOWED_STYLE: u32 = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE;

/// The window class the game registers and creates. Matching it means orb leaves
/// alone any other window the game makes.
const GAME_WINDOW_CLASS: &CStr = c"BASE";
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

/// The `Present` slot in `IDirect3DDevice8`'s vtable.
const PRESENT_SLOT: usize = 15;

static ENABLED: AtomicBool = AtomicBool::new(false);
/// The aspect ratio to keep, as the size the game renders at.
static CONTENT: MainThread<(u32, u32)> = MainThread::new((640, 480));
/// How much of the screen the game is to get, out of `orb.yaml`.
static SCREEN: MainThread<Screen> = MainThread::new(Screen::Fullscreen);
/// The window created for the game, which is the device's window.
static GAME_WINDOW: AtomicIsize = AtomicIsize::new(0);
/// Worked out from the client area on the first present and kept until it changes.
static DESTINATION: MainThread<Option<(RECT, RECT)>> = MainThread::new(None);

static REGISTER_CLASS_A: AtomicUsize = AtomicUsize::new(0);
static CREATE_WINDOW_EX_A: AtomicUsize = AtomicUsize::new(0);
static PRESENT: AtomicUsize = AtomicUsize::new(0);

type RegisterClassA = unsafe extern "system" fn(*const WNDCLASSA) -> u16;
#[allow(clippy::type_complexity)]
type CreateWindowExA = unsafe extern "system" fn(
    u32,
    *const u8,
    *const u8,
    u32,
    i32,
    i32,
    i32,
    i32,
    HWND,
    HMENU,
    *mut c_void,
    *const c_void,
) -> HWND;
type Present = unsafe extern "system" fn(
    *mut Device,
    *const RECT,
    *const RECT,
    HWND,
    *const c_void,
) -> Hresult;

/// # Safety
/// Must run before the game creates its window, and `module` must be the exe.
pub unsafe fn install(
    module: usize,
    content: (u32, u32),
    screen: Screen,
) -> Result<(), hook::Error> {
    // Before the window exists, which is the only moment this can be said. Without it every
    // size below is a size Windows scales behind the game's back: a 1280x720 window asked for on
    // a monitor at 150% is 1920x1080 of screen, and the monitor a fullscreen window is measured
    // against reads as two thirds of itself. The game never asked to be told about scaling, so
    // orb asks for it.
    let told = unsafe { SetProcessDPIAware() };
    log!(
        "screen: display scaling {}",
        if told != 0 {
            "is being ignored, sizes are real pixels"
        } else {
            "could not be turned off; sizes are whatever Windows scales them to"
        }
    );
    unsafe {
        *CONTENT.get() = content;
        *SCREEN.get() = screen;
        for (function, replacement, original) in [
            (
                "RegisterClassA",
                hook::address(register_class_a as _),
                &REGISTER_CLASS_A,
            ),
            (
                "CreateWindowExA",
                hook::address(create_window_ex_a as _),
                &CREATE_WINDOW_EX_A,
            ),
        ] {
            let previous = hook::install_import(module, "USER32.dll", function, replacement)?;
            original.store(previous, Ordering::Relaxed);
        }
    }
    ENABLED.store(true, Ordering::Relaxed);
    Ok(())
}

/// Redirects the device's `Present` so the back buffer lands in a rectangle of
/// the game's aspect ratio rather than being stretched over the whole window.
///
/// # Safety
/// `device` must be the game's live device, and this must run before it presents.
pub unsafe fn hook_device(device: *mut Device) {
    if !ENABLED.load(Ordering::Relaxed) || PRESENT.load(Ordering::Relaxed) != 0 {
        return;
    }
    let slot = unsafe { (*device).vtable as usize + PRESENT_SLOT * size_of::<usize>() };
    match unsafe { hook::replace_pointer(slot, hook::address(present as _)) } {
        Ok(original) => {
            PRESENT.store(original, Ordering::Relaxed);
            // With the client as it is now: the device has just been created, and creating one is
            // the other thing that can resize a window out from under whoever asked for it.
            log!(
                "screen: presenting through a letterbox, client {}",
                match unsafe { client_size(GAME_WINDOW.load(Ordering::Relaxed) as HWND) } {
                    Some((width, height)) => format!("{width}x{height}"),
                    None => "unknown".to_owned(),
                }
            );
        }
        Err(error) => log!("screen: cannot hook Present: {error}"),
    }
}

/// Gives the game's window class a black background, so the letterbox is painted
/// by Windows and repainted whenever the window is uncovered.
unsafe extern "system" fn register_class_a(class: *const WNDCLASSA) -> u16 {
    let original: RegisterClassA =
        unsafe { std::mem::transmute(REGISTER_CLASS_A.load(Ordering::Relaxed)) };
    if class.is_null() || !unsafe { is_game_class((*class).lpszClassName) } {
        return unsafe { original(class) };
    }
    let mut patched = unsafe { *class };
    patched.hbrBackground = unsafe { CreateSolidBrush(BLACK) } as HBRUSH;
    unsafe { original(&patched) }
}

/// Creates the game's window the size the settings say, in place of the size the game asked
/// for.
#[allow(clippy::too_many_arguments)]
unsafe extern "system" fn create_window_ex_a(
    ex_style: u32,
    class_name: *const u8,
    window_name: *const u8,
    style: u32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: HWND,
    menu: HMENU,
    instance: *mut c_void,
    param: *const c_void,
) -> HWND {
    let original: CreateWindowExA =
        unsafe { std::mem::transmute(CREATE_WINDOW_EX_A.load(Ordering::Relaxed)) };
    let monitor = primary_monitor();
    let Some(monitor) = monitor.filter(|_| unsafe { is_game_class(class_name) }) else {
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
    let wanted = unsafe { placed(monitor, *SCREEN.get()) };

    let window = unsafe {
        original(
            ex_style,
            class_name,
            window_name,
            wanted.style,
            wanted.area.left,
            wanted.area.top,
            wanted.area.right - wanted.area.left,
            wanted.area.bottom - wanted.area.top,
            parent,
            menu,
            instance,
            param,
        )
    };
    if !window.is_null() {
        GAME_WINDOW.store(window as isize, Ordering::Relaxed);
        // The client it came out with as well as the window that was asked for, because those are
        // two different numbers whenever anything between here and the screen has an opinion —
        // display scaling, a shell that remembers window sizes — and the client is the one the
        // letterbox and the bar beside it are worked out from.
        let client = unsafe { client_size(window) };
        log!(
            "screen: {} — window at {},{} sized {}x{}, client {}",
            unsafe { *SCREEN.get() },
            wanted.area.left,
            wanted.area.top,
            wanted.area.right - wanted.area.left,
            wanted.area.bottom - wanted.area.top,
            match client {
                Some((width, height)) => format!("{width}x{height}"),
                None => "unknown".to_owned(),
            },
        );
    }
    window
}

/// The client area of `window`, which is what the game draws into and what orb works its
/// letterbox out from.
unsafe fn client_size(window: HWND) -> Option<(i32, i32)> {
    let mut client = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    (unsafe { GetClientRect(window, &mut client) } != 0)
        .then(|| (client.right - client.left, client.bottom - client.top))
}

/// A window to create: what kind it is, and the whole of it — frame included — in the monitor's
/// own coordinates.
struct Placed {
    style: u32,
    area: RECT,
}

/// Where the game's window goes.
///
/// The size in the settings is the size of what is *inside* the window, so that `1280x720` is
/// 1280x720 of game however thick this machine's window frames are; `AdjustWindowRect` is what
/// turns that into the window to ask for.
fn placed(monitor: RECT, screen: Screen) -> Placed {
    let Screen::Window { width, height } = screen else {
        return Placed {
            style: BORDERLESS_STYLE,
            area: monitor,
        };
    };
    let mut area = RECT {
        left: 0,
        top: 0,
        right: width as i32,
        bottom: height as i32,
    };
    // A failure leaves the rectangle as the client area, which is a window a frame too small
    // rather than no window at all.
    unsafe { AdjustWindowRect(&mut area, WINDOWED_STYLE, 0) };
    Placed {
        style: WINDOWED_STYLE,
        area: centred(monitor, area),
    }
}

/// A rectangle of that size in the middle of the monitor, and against its top-left corner if it
/// is too big to be in the middle of it — a window whose caption is off the top of the screen
/// cannot be moved back on.
fn centred(monitor: RECT, area: RECT) -> RECT {
    let (width, height) = (area.right - area.left, area.bottom - area.top);
    let left = monitor.left + ((monitor.right - monitor.left - width) / 2).max(0);
    let top = monitor.top + ((monitor.bottom - monitor.top - height) / 2).max(0);
    RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

unsafe extern "system" fn present(
    device: *mut Device,
    source: *const RECT,
    destination: *const RECT,
    window_override: HWND,
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
    static PAINTED: MainThread<Option<RECT>> = MainThread::new(None);
    let window = GAME_WINDOW.load(Ordering::Relaxed) as HWND;
    let letterbox = unsafe { letterbox() };
    let Some(letterbox) = letterbox else { return };
    let mut client = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if window.is_null() || unsafe { GetClientRect(window, &mut client) } == 0 {
        return;
    }

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
            RECT {
                left: letterbox.right,
                top: 0,
                right: client.right,
                bottom: 0,
            },
            letterbox.right + BAR_MARGIN,
            TA_LEFT,
        )
    } else {
        (
            RECT {
                left: client.left,
                top: 0,
                right: client.right,
                bottom: 0,
            },
            letterbox.right,
            TA_RIGHT,
        )
    };
    if beside.max(below) < BAR_TEXT_MIN {
        report_no_bar(client, letterbox);
        return;
    }
    // What there is to write in, from where the text starts to the far edge of the bar. The
    // side bar runs away from the game and the strip under it runs back towards the game's
    // own edge, so the room is on opposite sides of the same x.
    let room = if align == TA_LEFT {
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

    let dc = unsafe { GetDC(window) };
    if dc.is_null() {
        return;
    }
    let (font, height) = unsafe { fitting_font(dc, lines, room) };
    let block = RECT {
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
    let drawn = unsafe { paint(dc, &bar, font, lines) };
    unsafe {
        DeleteObject(font as HGDIOBJ);
        ReleaseDC(window, dc);
    }
    // Only once it is on the screen: a draw that failed has left the bar holding whatever it
    // held, and the next call should be the one that puts it right rather than deciding
    // there is nothing to do.
    if drawn {
        *painted = Some(block);
        shown.clear();
        shown.extend_from_slice(lines);
    }
}

/// Where a stack of lines goes: the rows to repaint, and where in them the text starts.
struct Bar {
    /// Everything to clear and put on the screen, in client coordinates.
    area: RECT,
    /// The x the text is aligned from, in client coordinates.
    x: i32,
    /// The bottom of the lowest line.
    bottom: i32,
    /// One line's height.
    height: i32,
    align: u32,
}

/// Clears the bar to black, writes the lines into it, and puts the result on the screen in
/// one go.
///
/// Through a bitmap of its own rather than straight onto the window, because the clear and
/// the text are two operations: done on the window, a refresh landing between them shows a
/// bar with nothing in it, and at 120Hz that is a flicker somebody sees. One `BitBlt` cannot
/// be caught half done.
///
/// # Safety
/// `dc` must be the window's device context and `font` a live font, both the caller's to
/// release.
unsafe fn paint(dc: HDC, bar: &Bar, font: HFONT, lines: &[String]) -> bool {
    let width = bar.area.right - bar.area.left;
    let height = bar.area.bottom - bar.area.top;
    if width <= 0 || height <= 0 {
        return false;
    }
    unsafe {
        let memory = CreateCompatibleDC(dc);
        if memory.is_null() {
            return false;
        }
        let bitmap = CreateCompatibleBitmap(dc, width, height);
        if bitmap.is_null() {
            DeleteDC(memory);
            return false;
        }
        let previous_bitmap = SelectObject(memory, bitmap as HGDIOBJ);
        // Asked for by name rather than through a brush of ours: it is the same black the
        // window class paints the letterbox with, and there is nothing to keep or delete.
        PatBlt(memory, 0, 0, width, height, BLACKNESS);

        let previous_font = SelectObject(memory, font as HGDIOBJ);
        SetTextColor(memory, 0x00ff_ffff);
        SetBkMode(memory, TRANSPARENT as i32);
        SetTextAlign(memory, bar.align);
        // Bottom line last, so the stack grows upwards from the corner. A line that would
        // start above the area — a stack taller than the black there is — runs off the top of
        // the bitmap and is clipped there, rather than being drawn over the game.
        for (index, line) in lines.iter().rev().enumerate() {
            let top = bar.bottom - bar.height * (index as i32 + 1) - bar.area.top;
            let wide: Vec<u16> = line.encode_utf16().collect();
            ExtTextOutW(
                memory,
                bar.x - bar.area.left,
                top,
                0,
                std::ptr::null(),
                wide.as_ptr(),
                wide.len() as u32,
                std::ptr::null(),
            );
        }
        let blitted = BitBlt(
            dc,
            bar.area.left,
            bar.area.top,
            width,
            height,
            memory,
            0,
            0,
            SRCCOPY,
        ) != 0;
        SelectObject(memory, previous_font);
        SelectObject(memory, previous_bitmap);
        DeleteObject(bitmap as HGDIOBJ);
        DeleteDC(memory);
        blitted
    }
}

fn union(a: RECT, b: RECT) -> RECT {
    RECT {
        left: a.left.min(b.left),
        top: a.top.min(b.top),
        right: a.right.max(b.right),
        bottom: a.bottom.max(b.bottom),
    }
}

/// A font for these lines that fits them in `room` pixels, and the height it came out at.
///
/// Measured rather than guessed, because how wide a line comes out is the font's business:
/// what is written here runs from three characters to twenty, and the widest — a chapter
/// named for the part of a stage it belongs to — is the one that decides. Clipped at the
/// bar's edge it cannot be read at all, which is worse than small.
///
/// One measurement and at most one more font: character widths go with the em height
/// closely enough that scaling by the ratio lands inside a pixel or two, and the caller only
/// gets here when the lines have changed.
///
/// # Safety
/// `dc` must be a live device context, and the font returned is the caller's to delete.
unsafe fn fitting_font(dc: HDC, lines: &[String], room: i32) -> (HFONT, i32) {
    let font = |height| unsafe {
        CreateFontW(
            height,
            0,
            0,
            0,
            FW_NORMAL as i32,
            0,
            0,
            0,
            u32::from(DEFAULT_CHARSET),
            u32::from(OUT_DEFAULT_PRECIS),
            u32::from(CLIP_DEFAULT_PRECIS),
            u32::from(ANTIALIASED_QUALITY),
            u32::from(DEFAULT_PITCH),
            std::ptr::null(),
        )
    };
    let widest = |dc| {
        lines
            .iter()
            .map(|line| {
                let wide: Vec<u16> = line.encode_utf16().collect();
                let mut size = windows_sys::Win32::Foundation::SIZE { cx: 0, cy: 0 };
                let measured = unsafe {
                    GetTextExtentPoint32W(dc, wide.as_ptr(), wide.len() as i32, &mut size)
                };
                if measured == 0 { 0 } else { size.cx }
            })
            .max()
            .unwrap_or(0)
    };

    let full = font(BAR_TEXT_HEIGHT);
    let previous = unsafe { SelectObject(dc, full as HGDIOBJ) };
    let width = widest(dc);
    unsafe { SelectObject(dc, previous) };
    if width <= room || width <= 0 || room <= 0 {
        return (full, BAR_TEXT_HEIGHT);
    }

    let height = (BAR_TEXT_HEIGHT * room / width).max(BAR_TEXT_MIN);
    unsafe { DeleteObject(full as HGDIOBJ) };
    static SAID: AtomicIsize = AtomicIsize::new(0);
    if SAID.swap(height as isize, Ordering::Relaxed) != height as isize {
        log!("screen: {width}px of text in {room}px of black, writing at {height}px");
    }
    (font(height), height)
}

/// Said once: with the game filling the window there is nowhere beside it to write, and
/// that is a fact about the monitor rather than a fault.
fn report_no_bar(client: RECT, letterbox: RECT) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        log!(
            "screen: client {}x{}, game {}x{} at {},{} — no black to write in",
            client.right - client.left,
            client.bottom - client.top,
            letterbox.right - letterbox.left,
            letterbox.bottom - letterbox.top,
            letterbox.left,
            letterbox.top,
        );
    }
}

/// The rectangle inside the client area that the game's output belongs in,
/// recomputed only when the client area changes.
unsafe fn letterbox() -> Option<RECT> {
    let window = GAME_WINDOW.load(Ordering::Relaxed) as HWND;
    if window.is_null() {
        return None;
    }
    let mut client = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { GetClientRect(window, &mut client) } == 0 {
        return None;
    }

    let cached = unsafe { DESTINATION.get() };
    if let Some((known_client, destination)) = cached
        && same(known_client, &client)
    {
        return Some(*destination);
    }
    let destination = fit(client, unsafe { *CONTENT.get() });
    *cached = Some((client, destination));
    Some(destination)
}

/// The largest rectangle with `content`'s aspect ratio that fits `client`, centred.
fn fit(client: RECT, content: (u32, u32)) -> RECT {
    let available_width = i64::from(client.right - client.left);
    let available_height = i64::from(client.bottom - client.top);
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
    RECT {
        left,
        top,
        right: left + width as i32,
        bottom: top + height as i32,
    }
}

fn primary_monitor() -> Option<RECT> {
    let monitor = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
    if monitor.is_null() {
        return None;
    }
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    (unsafe { GetMonitorInfoW(monitor, &mut info) } != 0).then_some(info.rcMonitor)
}

unsafe fn is_game_class(name: *const u8) -> bool {
    // A class can also be given as an atom, which has nothing to dereference.
    if name.is_null() || (name as usize) <= u16::MAX as usize {
        return false;
    }
    unsafe { CStr::from_ptr(name.cast()) == GAME_WINDOW_CLASS }
}

fn same(a: &RECT, b: &RECT) -> bool {
    (a.left, a.top, a.right, a.bottom) == (b.left, b.top, b.right, b.bottom)
}

#[cfg(test)]
mod tests {
    use super::{centred, fit};
    use windows_sys::Win32::Foundation::RECT;

    fn client(width: i32, height: i32) -> RECT {
        RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        }
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
        let second = RECT {
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
