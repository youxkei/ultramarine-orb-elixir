//! Which keys the host has down.
//!
//! Behind the seam because orb's own menus are up on frames the game is frozen on, so they read the
//! keyboard themselves rather than through the game's input state — and a test that cannot press a
//! key cannot reach any of that. What the mode question does with a press is the mechanism, and it
//! was decided by hand until this was here.

/// Every key, `0x80` set on the ones that are down, or `None` where the host would not say.
pub fn state() -> Option<[u8; 256]> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.keyboard_state();
    }
    host::state()
}

#[cfg(windows)]
use crate::real::keyboard as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn state() -> Option<[u8; 256]> {
        no_windows("keyboard::state")
    }
}
