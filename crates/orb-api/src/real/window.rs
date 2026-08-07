//! Which real window is in front, what the real monitor measures, and the frame a real host puts
//! round a client area.

use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRect, GetClientRect, GetForegroundWindow, MB_ICONERROR, MB_OK, MB_SETFOREGROUND,
    MB_SYSTEMMODAL, MessageBoxW, SetProcessDPIAware,
};

use crate::{Hwnd, Rect};

pub fn foreground() -> Hwnd {
    Hwnd(unsafe { GetForegroundWindow() } as usize)
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

pub fn adjust_window_rect(area: Rect, style: u32, menu: bool) -> Option<Rect> {
    let mut rect = windows(area);
    (unsafe { AdjustWindowRect(&mut rect, style, i32::from(menu)) } != 0).then(|| neutral(rect))
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
