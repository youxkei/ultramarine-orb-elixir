//! Which window the host has in front, and the sizes the host decides: what the monitor measures,
//! the frame it puts round a client area, and the client a created window came out with.
//!
//! The three sizes are behind the seam for one reason, and it is not that they are Win32 calls: what
//! `orb::window` lays out is decided by numbers only the host knows, and a test that cannot move them
//! cannot reach the layout at all. The frame is the host's and its theme's — 6x40 on this machine —
//! and the monitor answers two different sizes for one panel depending on whether the process has said
//! it is DPI aware. Both of those are measurements, and each is kept beside the call that reports it.

use crate::{Hwnd, Rect};

/// The window the host has in front, or [`Hwnd::NULL`] if that is none of them.
///
/// Asked of the system rather than read from the game's own `WM_ACTIVATEAPP` flag, which only says
/// what the game was last told.
pub fn foreground() -> Hwnd {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.foreground_window();
    }
    host::foreground()
}

/// Says this process reads sizes as the monitor's real pixels, and whether the host took it.
pub fn set_process_dpi_aware() -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_process_dpi_aware();
    }
    host::set_process_dpi_aware()
}

/// The primary monitor, in the desktop's own coordinates.
pub fn primary_monitor() -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.primary_monitor();
    }
    host::primary_monitor()
}

/// The whole window — frame included — that a client area of `area`'s size needs.
pub fn adjust_window_rect(area: Rect, style: u32, menu: bool) -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.adjust_window_rect(area, style, menu);
    }
    host::adjust_window_rect(area, style, menu)
}

/// What the game draws into, which is what the letterbox and the bar beside it are worked out from.
pub fn client_rect(window: Hwnd) -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.client_rect(window);
    }
    host::client_rect(window)
}

#[cfg(windows)]
use crate::real::window as host;

#[cfg(not(windows))]
mod host {
    use crate::{Hwnd, Rect, no_windows};

    pub fn foreground() -> Hwnd {
        no_windows("window::foreground")
    }
    pub fn set_process_dpi_aware() -> bool {
        no_windows("window::set_process_dpi_aware")
    }
    pub fn primary_monitor() -> Option<Rect> {
        no_windows("window::primary_monitor")
    }
    pub fn adjust_window_rect(_area: Rect, _style: u32, _menu: bool) -> Option<Rect> {
        no_windows("window::adjust_window_rect")
    }
    pub fn client_rect(_window: Hwnd) -> Option<Rect> {
        no_windows("window::client_rect")
    }
}
