//! The joystick winmm has, which is not the one DirectInput has.
//!
//! Behind the seam because it is the branch the game takes where its own enumeration found no game
//! controller, and orb has to read the same device or its menus answer to a pad the game has not got.
//! Two calls and nothing else: where the stick is, and what it is — see
//! [`crate::JoyInfo`] and [`crate::JoyCaps`], both of which are winmm's own layouts because the first
//! is handed to the game and the second is written into its memory.

use crate::{JoyCaps, JoyInfo};

/// `joyGetPosEx(device, flags)`, as the result it answered and the position it filled in.
pub fn position(device: u32, flags: u32) -> (u32, JoyInfo) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.joystick_position(device, flags);
    }
    host::position(device, flags)
}

/// `joyGetDevCapsA(device)`. `None` where the call failed, which is a device orb cannot describe and
/// must not write the game's axis calibration from.
pub fn caps(device: u32) -> Option<JoyCaps> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.joystick_caps(device);
    }
    host::caps(device)
}

/// `joyGetNumDevs()` — how many indices there are to ask about, which is not how many pads are
/// plugged in. The game only ever asks about joystick 0; orb asks about all of them, a second pad
/// being that same player's second pad.
pub fn count() -> u32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.joystick_count();
    }
    host::count()
}

#[cfg(windows)]
use crate::real::joystick as host;

#[cfg(not(windows))]
mod host {
    use crate::{JoyCaps, JoyInfo, no_windows};

    pub fn position(_device: u32, _flags: u32) -> (u32, JoyInfo) {
        no_windows("joystick::position")
    }
    pub fn caps(_device: u32) -> Option<JoyCaps> {
        no_windows("joystick::caps")
    }
    pub fn count() -> u32 {
        no_windows("joystick::count")
    }
}
