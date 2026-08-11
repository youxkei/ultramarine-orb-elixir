//! The pads XInput has, which are not the ones winmm has.
//!
//! Behind the seam beside [`crate::joystick`] and for the same reason: a machine can have a pad on one
//! of the two interfaces and not on the other, so orb reads both and an e2e test has to be able to plug
//! a pad into either. What decides which is which is not orb's to know — a pad in XInput's second slot
//! leaves winmm's joystick 0 a phantom with no buttons and no axes, and is on none of winmm's other
//! indices at all.
//!
//! Which of XInput's three libraries a machine has is a property of the machine, so the real side loads
//! one by name rather than importing it — see [`crate::real::xinput`].

use crate::XinputPad;

/// How many users XInput has, which is how many slots there are to ask about.
pub const SLOTS: u32 = 4;

/// `XInputGetState(slot)` — the pad in that slot, and `None` for an empty slot or a host with no
/// XInput.
pub fn state(slot: u32) -> Option<XinputPad> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.xinput_state(slot);
    }
    host::state(slot)
}

#[cfg(windows)]
use crate::real::xinput as host;

#[cfg(not(windows))]
mod host {
    use crate::{XinputPad, no_windows};

    pub fn state(_slot: u32) -> Option<XinputPad> {
        no_windows("xinput::state")
    }
}
