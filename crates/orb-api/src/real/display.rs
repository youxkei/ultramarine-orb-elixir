//! The real display, and the real compositor.

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::{DWM_TIMING_INFO, DwmFlush, DwmGetCompositionTimingInfo};
use windows_sys::Win32::Graphics::Gdi::{
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW, GetDC, GetDeviceCaps, GetMonitorInfoW,
    MONITOR_DEFAULTTONEAREST, MONITORINFOEXW, MonitorFromWindow, ReleaseDC, VREFRESH,
};

use crate::{Composition, Hwnd};

pub fn monitor_refresh(window: Hwnd) -> Option<u32> {
    if window.is_null() {
        return None;
    }
    let monitor = unsafe { MonitorFromWindow(window.0 as HWND, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    if unsafe { GetMonitorInfoW(monitor, (&raw mut info).cast()) } == 0 {
        return None;
    }
    let mut mode: DEVMODEW = unsafe { std::mem::zeroed() };
    mode.dmSize = size_of::<DEVMODEW>() as u16;
    let read =
        unsafe { EnumDisplaySettingsW(info.szDevice.as_ptr(), ENUM_CURRENT_SETTINGS, &mut mode) };
    (read != 0 && mode.dmDisplayFrequency > 1).then_some(mode.dmDisplayFrequency)
}

pub fn desktop_refresh() -> Option<u32> {
    let screen = unsafe { GetDC(std::ptr::null_mut()) };
    if screen.is_null() {
        return None;
    }
    let refresh = unsafe { GetDeviceCaps(screen, VREFRESH as i32) };
    unsafe { ReleaseDC(std::ptr::null_mut(), screen) };
    // One and zero are what the call answers for "the default rate", which is not a rate.
    u32::try_from(refresh).ok().filter(|refresh| *refresh > 1)
}

pub fn composition() -> Option<Composition> {
    let mut info: DWM_TIMING_INFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<DWM_TIMING_INFO>() as u32;
    let asked = unsafe { DwmGetCompositionTimingInfo(std::ptr::null_mut(), &mut info) };
    if asked < 0 {
        return None;
    }
    Some(Composition {
        refresh_period: info.qpcRefreshPeriod as i64,
        refresh: info.cRefresh as i64,
        vblank: info.qpcVBlank as i64,
        frames_late: info.cFramesLate as i64,
    })
}

pub fn flush() -> bool {
    let flushed = unsafe { DwmFlush() };
    flushed >= 0
}
