//! Which window the host has in front.

use crate::Hwnd;

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

#[cfg(windows)]
use crate::real::window as host;

#[cfg(not(windows))]
mod host {
    use crate::{Hwnd, no_windows};

    pub fn foreground() -> Hwnd {
        no_windows("window::foreground")
    }
}
