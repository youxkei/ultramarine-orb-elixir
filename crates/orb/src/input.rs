//! Keyboard reading for orb's own keys.
//!
//! Read directly rather than through the game's input state, because the keys
//! that matter most are the ones pressed while the game's update — and with it
//! its input handling — is deliberately not running.

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

const DOWN: u8 = 0x80;

pub struct Keyboard {
    previous: [u8; 256],
    current: [u8; 256],
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            previous: [0; 256],
            current: [0; 256],
        }
    }

    /// Reads the keyboard, treating everything as released unless `window` is in
    /// front: alt-tabbing away must not leave a key stuck down, and must not let
    /// typing elsewhere drive the game.
    pub fn poll(&mut self, window: HWND) {
        self.previous = self.current;
        let focused = !window.is_null() && unsafe { GetForegroundWindow() } == window;
        if !focused || unsafe { GetKeyboardState(self.current.as_mut_ptr()) } == 0 {
            self.current = [0; 256];
        }
    }

    pub fn held(&self, key: u8) -> bool {
        self.current[key as usize] & DOWN != 0
    }

    pub fn pressed(&self, key: u8) -> bool {
        self.held(key) && self.previous[key as usize] & DOWN == 0
    }
}
