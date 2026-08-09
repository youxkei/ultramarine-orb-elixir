//! The display, and the compositor that times it.
//!
//! Behind the seam because of what the pacing decides from it and cannot otherwise be made to
//! decide. Which path a frame takes turns on two numbers this module answers — the monitor's
//! reported rate and the compositor's own — and the case that matters is the one where they
//! disagree, which needs two monitors of different rates and the game's window on the one the
//! compositor is not timing. A simulated display says so in a line.
//!
//! What is *not* behind the seam is whether a frame landed on a real blank. That is a measurement of
//! real hardware, and what it decided is beside `frame::Pacing::grid`.

use crate::{Composition, Hwnd};

/// The refresh rate of the monitor the window is on, in whole Hz.
pub fn monitor_refresh(window: Hwnd) -> Option<u32> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.monitor_refresh(window);
    }
    host::monitor_refresh(window)
}

/// The desktop's refresh rate, in whole Hz — what the pacing starts from, before there is a window
/// to ask about.
pub fn desktop_refresh() -> Option<u32> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.desktop_refresh();
    }
    host::desktop_refresh()
}

/// What the compositor says about the blanks, or `None` when it will not say.
pub fn composition() -> Option<Composition> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.composition();
    }
    host::composition()
}

/// Waits for the compositor to compose the next frame — so it returns at the blank the frame just
/// handed over reached. `false` where the compositor has gone.
pub fn flush() -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.flush();
    }
    host::flush()
}

#[cfg(windows)]
use crate::real::display as host;

#[cfg(not(windows))]
mod host {
    use crate::{Composition, Hwnd, no_windows};

    pub fn monitor_refresh(_window: Hwnd) -> Option<u32> {
        no_windows("display::monitor_refresh")
    }
    pub fn desktop_refresh() -> Option<u32> {
        no_windows("display::desktop_refresh")
    }
    pub fn composition() -> Option<Composition> {
        no_windows("display::composition")
    }
    pub fn flush() -> bool {
        no_windows("display::flush")
    }
}
