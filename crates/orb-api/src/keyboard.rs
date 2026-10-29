//! Which keys the host has down, and which a launch of orb's own is pressing.
//!
//! Behind the seam because orb's own menus are up on frames the game is frozen on, so they read the
//! keyboard themselves rather than through the game's input state — and a test that cannot press a
//! key cannot reach any of that. What the mode question does with a press is the mechanism, and it
//! was decided by hand until this was here.

use std::sync::Mutex;

/// **The keys a launch playing itself is pressing**, held where a keyboard's state is read.
///
/// **orb's own stand-in for a virtual keyboard device.** A device would need a driver; what a driver
/// would buy is a state every reader of the keyboard sees — the game's `GetKeyboardState` and orb's
/// own menus — that belongs to nobody's hands and keeps working while another window is in front. That
/// is what this is: the launch presses it, [`state`] and [`device_state`] answer it with whatever the
/// machine's keyboard holds, and nothing is sent to the system, so a launch pressing this way can play
/// with its window behind and leave the machine to whoever is using it.
///
/// **`SendInput` was the first answer and it was the wrong one**: it is delivered to whichever window
/// is in front, so a launch pressing it with the game behind types the arrows into whatever somebody
/// else has up — and the only way to stop that is to take the focus, which makes the machine unusable
/// for as long as the run lasts.
static PRESSED: Mutex<[u8; 256]> = Mutex::new([0; 256]);

/// **What the launch is pressing, as the keyboard a game reads** — `held` and nothing else, the keys
/// given `0x80` the way `GetKeyboardState` holds one down.
///
/// Called once a frame with the word the frame is being played with, so a key the word lets go of is
/// let go of here too.
pub fn press(held: &[u8]) {
    if let Ok(mut pressed) = PRESSED.lock() {
        pressed.fill(0);
        for key in held {
            pressed[usize::from(*key)] = 0x80;
        }
    }
}

/// What the launch is pressing now, for a test to read back — a production reader gets it through
/// [`state`] or [`device_state`], which is where a game's own read goes.
pub fn pressed() -> [u8; 256] {
    *PRESSED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Every key, `0x80` set on the ones that are down, or `None` where the host would not say.
///
/// **The launch's own presses and the machine's keys together**: a person at the keyboard and a launch
/// pressing from the host are the same read, which is what makes one indistinguishable from the other.
pub fn state() -> Option<[u8; 256]> {
    #[cfg(feature = "sim")]
    let mut answer = if let Some(win) = crate::installed() {
        win.keyboard_state()
    } else {
        host::state()
    };
    #[cfg(not(feature = "sim"))]
    let mut answer = host::state();
    if let Some(keys) = answer.as_mut() {
        keys.iter_mut()
            .zip(pressed())
            .for_each(|(key, pressed)| *key |= pressed);
    }
    answer
}

/// **What the keyboard the machine's own device reports has down, whether or not this process is in
/// front** — `GetAsyncKeyState` read once for each of the 256 keys, and the launch's own presses on
/// top.
///
/// Behind the seam as the read a game's `GetKeyboardState` is answered from, which is the state a
/// launch pressing from the host fills.
pub fn device_state() -> [u8; 256] {
    #[cfg(feature = "sim")]
    let mut answer = if let Some(win) = crate::installed() {
        win.keyboard_device()
    } else {
        host::device_state()
    };
    #[cfg(not(feature = "sim"))]
    let mut answer = host::device_state();
    answer
        .iter_mut()
        .zip(pressed())
        .for_each(|(key, pressed)| *key |= pressed);
    answer
}

#[cfg(windows)]
use crate::real::keyboard as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn state() -> Option<[u8; 256]> {
        no_windows("keyboard::state")
    }

    pub fn device_state() -> [u8; 256] {
        no_windows("keyboard::device_state")
    }
}
