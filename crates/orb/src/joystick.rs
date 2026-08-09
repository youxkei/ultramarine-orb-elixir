//! The write over the game's own `joyGetPosEx` entry.
//!
//! **The one thing in this module no scenario reaches**, which is the whole of what is left here: a game
//! laid out by hand has no import table, so it is handed the entry through
//! [`orb_core::joystick::install_over`] and calls [`orb_core::joystick::answer`] itself where its own read
//! would have gone through it. Everything past this line is covered that way — see
//! `orb-e2e/src/mode_on_a_winmm_pad.rs`.
//!
//! The thread that takes the samples, what a sample means, and the measurement of the 8.7ms call this
//! exists to move are all [`orb_core::joystick`]'s.

use orb_core::joystick::{JoyGetPosEx, answer, install_over};

use crate::hook;

/// Points the game's `joyGetPosEx` at orb, and takes the address of the caps it measures
/// axes against so that a device arriving mid-run can be described to it.
///
/// # Safety
/// `module` must be the game exe, and nothing may be executing its import table.
pub unsafe fn install(module: usize, calibration: Option<usize>) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "WINMM.dll",
            "joyGetPosEx",
            hook::address(answer as _),
        )
    }?;
    install_over(
        unsafe { std::mem::transmute::<usize, JoyGetPosEx>(previous) },
        calibration,
    );
    Ok(())
}
