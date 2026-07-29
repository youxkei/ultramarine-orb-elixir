//! Borderless fullscreen, through the game's own window and device.
//!
//! The window is borderless and covering the monitor from the moment it exists,
//! because the arguments of the game's own `CreateWindowExA` call are rewritten on
//! the way through — there is no frame to remove afterwards and nothing to flash
//! on screen first. Its window class gets a black background, and that is what
//! fills the letterbox.
//!
//! In windowed mode the game asks Direct3D for a `D3DSWAPEFFECT_COPY` swap chain,
//! and that is the swap effect that honours a destination rectangle on `Present`.
//! So the 640x480 back buffer is presented into a centred rectangle with the
//! game's aspect ratio, and the rest of the window keeps its black background.

use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{COLORREF, HWND, POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    ANTIALIASED_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET,
    DEFAULT_PITCH, DeleteObject, ETO_OPAQUE, ExtTextOutW, FW_NORMAL, GetDC, GetMonitorInfoW,
    HBRUSH, HGDIOBJ, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint, OPAQUE,
    OUT_DEFAULT_PRECIS, ReleaseDC, SelectObject, SetBkColor, SetBkMode, SetTextAlign,
    SetTextColor, TA_LEFT, TA_RIGHT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetClientRect, HMENU, WNDCLASSA, WS_POPUP, WS_VISIBLE,
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

/// The window class the game registers and creates. Matching it means orb leaves
/// alone any other window the game or vpatch makes.
const GAME_WINDOW_CLASS: &CStr = c"BASE";
/// How tall the lines written beside the game are, in pixels of the monitor rather than
/// of the game — this text is not scaled with the game's output.
const BAR_TEXT_HEIGHT: i32 = 30;
/// How far the text keeps from the edges of the window.
const BAR_MARGIN: i32 = 8;

/// The `Present` slot in `IDirect3DDevice8`'s vtable.
const PRESENT_SLOT: usize = 15;

static ENABLED: AtomicBool = AtomicBool::new(false);
/// The aspect ratio to keep, as the size the game renders at.
static CONTENT: MainThread<(u32, u32)> = MainThread::new((640, 480));
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
type Present =
    unsafe extern "system" fn(*mut Device, *const RECT, *const RECT, HWND, *const c_void) -> Hresult;

/// # Safety
/// Must run before the game creates its window, and `module` must be the exe.
pub unsafe fn install(module: usize, content: (u32, u32)) -> Result<(), hook::Error> {
    unsafe {
        *CONTENT.get() = content;
        for (function, replacement, original) in [
            ("RegisterClassA", register_class_a as usize, &REGISTER_CLASS_A),
            ("CreateWindowExA", create_window_ex_a as usize, &CREATE_WINDOW_EX_A),
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
    match unsafe { hook::replace_pointer(slot, present as usize) } {
        Ok(original) => {
            PRESENT.store(original, Ordering::Relaxed);
            log!("borderless: presenting through a letterbox");
        }
        Err(error) => log!("borderless: cannot hook Present: {error}"),
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

/// Creates the game's window borderless and covering the monitor.
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
                ex_style, class_name, window_name, style, x, y, width, height, parent, menu,
                instance, param,
            )
        };
    };

    let window = unsafe {
        original(
            ex_style,
            class_name,
            window_name,
            BORDERLESS_STYLE,
            monitor.left,
            monitor.top,
            monitor.right - monitor.left,
            monitor.bottom - monitor.top,
            parent,
            menu,
            instance,
            param,
        )
    };
    if !window.is_null() {
        GAME_WINDOW.store(window as isize, Ordering::Relaxed);
        log!(
            "borderless: window at {},{} sized {}x{}",
            monitor.left,
            monitor.top,
            monitor.right - monitor.left,
            monitor.bottom - monitor.top,
        );
    }
    window
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
        log!("borderless: Present into a letterbox failed ({result:#x}); stretching instead");
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
/// Only when the text changes, and with an opaque background, so each line covers what
/// was there before.
///
/// # Safety
/// Must run on the thread that owns the window, and outside a scene.
pub unsafe fn write_beside(lines: &[String]) {
    static SHOWN: MainThread<Vec<String>> = MainThread::new(Vec::new());
    let window = GAME_WINDOW.load(Ordering::Relaxed) as HWND;
    let letterbox = unsafe { letterbox() };
    let Some(letterbox) = letterbox else { return };
    let mut client = RECT { left: 0, top: 0, right: 0, bottom: 0 };
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
            RECT { left: letterbox.right, top: 0, right: client.right, bottom: 0 },
            letterbox.right + BAR_MARGIN,
            TA_LEFT,
        )
    } else {
        (
            RECT { left: client.left, top: 0, right: client.right, bottom: 0 },
            letterbox.right,
            TA_RIGHT,
        )
    };
    if beside.max(below) < BAR_TEXT_HEIGHT {
        report_no_bar(client, letterbox);
        return;
    }

    let shown = unsafe { SHOWN.get() };
    if shown == lines {
        return;
    }
    shown.clear();
    shown.extend_from_slice(lines);

    let dc = unsafe { GetDC(window) };
    if dc.is_null() {
        return;
    }
    unsafe {
        let font = CreateFontW(
            BAR_TEXT_HEIGHT,
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
        );
        let previous = SelectObject(dc, font as HGDIOBJ);
        SetTextColor(dc, 0x00ff_ffff);
        SetBkColor(dc, 0x0000_0000);
        SetBkMode(dc, OPAQUE as i32);
        SetTextAlign(dc, align);
        // Bottom line last, so the stack grows upwards from the corner.
        for (index, line) in lines.iter().rev().enumerate() {
            let top = bottom - BAR_TEXT_HEIGHT * (index as i32 + 1);
            // Cleared across the whole bar, so a line shorter than the one it replaces
            // leaves nothing of it behind.
            let strip = RECT { top, bottom: top + BAR_TEXT_HEIGHT, ..strip };
            let wide: Vec<u16> = line.encode_utf16().collect();
            ExtTextOutW(
                dc,
                x,
                top,
                ETO_OPAQUE,
                &strip,
                wide.as_ptr(),
                wide.len() as u32,
                std::ptr::null(),
            );
        }
        SelectObject(dc, previous);
        DeleteObject(font as HGDIOBJ);
        ReleaseDC(window, dc);
    }
}

/// Said once: with the game filling the window there is nowhere beside it to write, and
/// that is a fact about the monitor rather than a fault.
fn report_no_bar(client: RECT, letterbox: RECT) {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        log!(
            "borderless: client {}x{}, game {}x{} at {},{} — no black to write in",
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
    let mut client = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    if unsafe { GetClientRect(window, &mut client) } == 0 {
        return None;
    }

    let cached = unsafe { DESTINATION.get() };
    if let Some((known_client, destination)) = cached {
        if same(known_client, &client) {
            return Some(*destination);
        }
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
        (available_width, available_width * wanted_height / wanted_width)
    } else {
        (available_height * wanted_width / wanted_height, available_height)
    };

    let left = client.left + ((available_width - width) / 2) as i32;
    let top = client.top + ((available_height - height) / 2) as i32;
    RECT { left, top, right: left + width as i32, bottom: top + height as i32 }
}

fn primary_monitor() -> Option<RECT> {
    let monitor =
        unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
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
    use super::fit;
    use windows_sys::Win32::Foundation::RECT;

    fn client(width: i32, height: i32) -> RECT {
        RECT { left: 0, top: 0, right: width, bottom: height }
    }

    #[test]
    fn a_four_three_game_is_pillarboxed_on_a_sixteen_nine_screen() {
        let rect = fit(client(2560, 1440), (640, 480));
        assert_eq!((rect.left, rect.top, rect.right, rect.bottom), (320, 0, 2240, 1440));
    }

    #[test]
    fn a_game_wider_than_the_screen_is_letterboxed() {
        let rect = fit(client(1000, 1000), (640, 480));
        assert_eq!((rect.left, rect.top, rect.right, rect.bottom), (0, 125, 1000, 875));
    }

    #[test]
    fn a_matching_ratio_fills_the_screen() {
        let rect = fit(client(1600, 1200), (640, 480));
        assert_eq!((rect.left, rect.top, rect.right, rect.bottom), (0, 0, 1600, 1200));
    }
}
