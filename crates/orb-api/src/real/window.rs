//! Which real window is in front, what the real monitor measures, the frame a real host puts round a
//! client area, and the GDI a stack of lines is measured and written with.

use windows_sys::Win32::Foundation::{BOOL, LPARAM, POINT, RECT, SIZE, TRUE};
use windows_sys::Win32::Graphics::Gdi::{
    ANTIALIASED_QUALITY, BLACKNESS, BitBlt, CLIP_DEFAULT_PRECIS, CreateCompatibleBitmap,
    CreateCompatibleDC, DEFAULT_CHARSET, DEFAULT_PITCH, DeleteDC, DeleteObject,
    EnumDisplayMonitors, FW_NORMAL, GetDC, GetMonitorInfoW, HDC, HFONT, HGDIOBJ, HMONITOR,
    MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint, OUT_DEFAULT_PRECIS, PatBlt, ReleaseDC,
    SRCCOPY, SelectObject, SetBkMode, SetTextAlign, SetTextColor, TA_LEFT, TA_RIGHT, TRANSPARENT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRect, DispatchMessageW, GWL_EXSTYLE, GetClientRect, GetForegroundWindow,
    GetWindowLongPtrW, GetWindowRect, IsWindowVisible, MB_ICONERROR, MB_OK, MB_SETFOREGROUND,
    MB_SYSTEMMODAL, MSG, MessageBoxW, PM_REMOVE, PeekMessageW, SW_SHOWNA, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetForegroundWindow, SetProcessDPIAware,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, WM_QUIT,
};

use crate::real::gdi;
use crate::{Bar, Hwnd, Rect};

// The two alignments, written out above the seam and held against Windows' own numbers here. Which way
// a stack of lines is aligned is half of deciding which bar it goes in, so it is decided where the rest
// of that layout is — and a number that drifted from `windows-sys` is a build that stops rather than
// text written over the game.
const _: () = {
    assert!(Bar::LEFT == TA_LEFT);
    assert!(Bar::RIGHT == TA_RIGHT);
};

/// The white the lines are written in, over the black the window class paints.
const TEXT: u32 = 0x00ff_ffff;

pub fn foreground() -> Hwnd {
    Hwnd(unsafe { GetForegroundWindow() } as usize)
}

pub fn bring_forward(window: Hwnd) -> bool {
    unsafe { SetForegroundWindow(window.0 as _) != 0 }
}

pub fn set_process_dpi_aware() -> bool {
    unsafe { SetProcessDPIAware() != 0 }
}

pub fn primary_monitor() -> Option<Rect> {
    let monitor = unsafe { MonitorFromPoint(POINT { x: 0, y: 0 }, MONITOR_DEFAULTTOPRIMARY) };
    if monitor.is_null() {
        return None;
    }
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    (unsafe { GetMonitorInfoW(monitor, &mut info) } != 0).then(|| neutral(info.rcMonitor))
}

/// Every monitor, as the desktop has them arranged.
///
/// `EnumDisplayMonitors` and not a walk of `MonitorFromPoint`: which monitors there are and where each
/// one sits is one question, and this is the call that answers it.
pub fn monitors() -> Vec<Rect> {
    let mut found: Vec<Rect> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(collect),
            (&raw mut found) as isize,
        )
    };
    found
}

/// One monitor, onto the end of the list the enumeration was given.
unsafe extern "system" fn collect(
    _monitor: HMONITOR,
    _dc: HDC,
    rect: *mut RECT,
    into: LPARAM,
) -> BOOL {
    let found = unsafe { &mut *(into as *mut Vec<Rect>) };
    found.push(neutral(unsafe { *rect }));
    TRUE
}

pub fn adjust_window_rect(area: Rect, style: u32, menu: bool) -> Option<Rect> {
    let mut rect = windows(area);
    (unsafe { AdjustWindowRect(&mut rect, style, i32::from(menu)) } != 0).then(|| neutral(rect))
}

pub fn window_rect(window: Hwnd) -> Option<Rect> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    (unsafe { GetWindowRect(window.0 as _, &mut rect) } != 0).then(|| neutral(rect))
}

pub fn client_rect(window: Hwnd) -> Option<Rect> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    (unsafe { GetClientRect(window.0 as _, &mut rect) } != 0).then(|| neutral(rect))
}

pub fn add_ex_style(window: Hwnd, ex_style: u32) -> bool {
    // What `SetWindowLongPtrW` answers is the style that was there, and a window with none answers
    // nought for success and for failure alike — so the frame's redraw below is what says it took.
    // Through `u32`, because the style is one on both targets and the call's own integer is not: a
    // `LONG` on the 32-bit build and a `LONG_PTR` on the 64-bit one.
    let had = unsafe { GetWindowLongPtrW(window.0 as _, GWL_EXSTYLE) } as u32;
    unsafe { SetWindowLongPtrW(window.0 as _, GWL_EXSTYLE, (had | ex_style) as _) };
    // The frame is redrawn where the style changed, and nothing else about the window is touched.
    unsafe {
        SetWindowPos(
            window.0 as _,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        ) != 0
    }
}

pub fn show_without_activating(window: Hwnd) -> bool {
    // `ShowWindow` answers whether the window *was* visible, which is nought for the one this is for —
    // it was made hidden. So the answer is thrown away and the call is what there is; a window that
    // refuses to show has no other sign to give.
    unsafe { ShowWindow(window.0 as _, SW_SHOWNA) };
    true
}

pub fn is_visible(window: Hwnd) -> bool {
    unsafe { IsWindowVisible(window.0 as _) != 0 }
}

/// `PeekMessageW` with `PM_REMOVE` until the queue is empty, each message translated and dispatched the
/// way a window's own loop does it, and `true` where a `WM_QUIT` was among them.
pub fn pump() -> bool {
    let mut quit = false;
    let mut message = MSG {
        hwnd: std::ptr::null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 },
    };
    while unsafe { PeekMessageW(&raw mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
        if message.message == WM_QUIT {
            quit = true;
            continue;
        }
        unsafe { TranslateMessage(&raw const message) };
        unsafe { DispatchMessageW(&raw const message) };
    }
    quit
}

pub fn ex_style(window: Hwnd) -> u32 {
    // Through `u32` for [`add_ex_style`]'s reason: the style is one on both targets and the call's own
    // integer is not.
    let has = unsafe { GetWindowLongPtrW(window.0 as _, GWL_EXSTYLE) };
    has as u32
}

/// A font at `em` pixels of em, in whatever face the host substitutes for no name at all: the lines
/// beside the game are orb's own text and not the game's, so nothing here asks for the game's font.
///
/// Through [`gdi::text`] rather than this DLL's import of it, which is where every call below that carries
/// a string or a font goes — see that module for what a translation patch injected into the same game does
/// to them.
fn font(em: i32) -> HFONT {
    unsafe {
        (gdi::text().create_font)(
            em,
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
    }
}

/// The screen's own device context, which is what a measurement is taken against: text extents are the
/// font's and the device's, and every window of this process is on the same one.
pub fn measure_lines(lines: &[String], em: i32) -> (i32, i32) {
    let dc = unsafe { GetDC(std::ptr::null_mut()) };
    if dc.is_null() {
        return (0, 0);
    }
    let font = font(em);
    let previous = unsafe { SelectObject(dc, font as HGDIOBJ) };
    let mut widest = 0;
    let mut line = 0;
    for text in lines {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut size = SIZE { cx: 0, cy: 0 };
        let measured =
            unsafe { (gdi::text().text_extent)(dc, wide.as_ptr(), wide.len() as i32, &mut size) };
        if measured != 0 {
            widest = widest.max(size.cx);
            line = line.max(size.cy);
        }
    }
    unsafe {
        SelectObject(dc, previous);
        DeleteObject(font as HGDIOBJ);
        ReleaseDC(std::ptr::null_mut(), dc);
    }
    (widest, line)
}

/// Clears the bar to black, writes the lines into it, and puts the result on the screen in one go.
///
/// Through a bitmap of its own rather than straight onto the window, because the clear and the text are
/// two operations: done on the window, a refresh landing between them shows a bar with nothing in it,
/// and at 120Hz that is a flicker somebody sees. One `BitBlt` cannot be caught half done.
pub fn write_lines(window: Hwnd, bar: Bar, lines: &[String]) -> bool {
    let (width, height) = (bar.area.width(), bar.area.height());
    if width <= 0 || height <= 0 {
        return false;
    }
    let window = window.0 as _;
    let dc = unsafe { GetDC(window) };
    if dc.is_null() {
        return false;
    }
    let font = font(bar.height);
    let blitted = unsafe { paint(dc, &bar, font, lines) };
    unsafe {
        DeleteObject(font as HGDIOBJ);
        ReleaseDC(window, dc);
    }
    blitted
}

/// # Safety
/// `dc` must be the window's device context and `font` a live font, both the caller's to release.
unsafe fn paint(dc: HDC, bar: &Bar, font: HFONT, lines: &[String]) -> bool {
    let (width, height) = (bar.area.width(), bar.area.height());
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
        SetTextColor(memory, TEXT);
        SetBkMode(memory, TRANSPARENT as i32);
        SetTextAlign(memory, bar.align);
        // Bottom line last, so the stack grows upwards from the corner. A line that would
        // start above the area — a stack taller than the black there is — runs off the top of
        // the bitmap and is clipped there, rather than being drawn over the game.
        for (index, line) in lines.iter().rev().enumerate() {
            let top = bar.bottom - bar.height * (index as i32 + 1) - bar.area.top;
            let wide: Vec<u16> = line.encode_utf16().collect();
            (gdi::text().ext_text_out)(
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

pub fn message_box(title: &str, text: &str) {
    let title: Vec<u16> = title.encode_utf16().chain([0]).collect();
    let text: Vec<u16> = text.encode_utf16().chain([0]).collect();
    // No owner window, and system-modal and foreground on top of that: the game this is raised from
    // is drawing full-screen through Direct3D, and a plain application-modal box behind it is one
    // nobody can read — which for the one message that ends a launch is the same as saying nothing.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR | MB_SYSTEMMODAL | MB_SETFOREGROUND,
        )
    };
}

fn neutral(rect: RECT) -> Rect {
    Rect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

fn windows(rect: Rect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
