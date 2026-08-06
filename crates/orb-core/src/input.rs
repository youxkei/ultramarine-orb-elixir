//! Keyboard reading for orb's own keys.
//!
//! Read directly rather than through the game's input state, because the keys
//! that matter most are the ones pressed while the game's update — and with it
//! its input handling — is deliberately not running.

use orb_api::{Hwnd, keyboard, window};

const DOWN: u8 = 0x80;

pub struct Keyboard {
    previous: [u8; 256],
    current: [u8; 256],
    /// Whether the last read was one orb was allowed to make. What this is for is the frame after it
    /// was not — see `poll`.
    reading: bool,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            previous: [0; 256],
            current: [0; 256],
            reading: false,
        }
    }

    /// Reads the keyboard, treating everything as released unless `window` is in
    /// front: alt-tabbing away must not leave a key stuck down, and must not let
    /// typing elsewhere drive the game.
    pub fn poll(&mut self, ours: Hwnd) {
        self.previous = self.current;
        let reading = !ours.is_null() && window::foreground() == ours;
        let was_reading = std::mem::replace(&mut self.reading, reading);
        self.current = reading.then(keyboard::state).flatten().unwrap_or([0; 256]);
        // Whatever is down on the first frame orb is reading again counts as already held, not as
        // just pressed. Zeroing while away is what keeps a key from sticking down, and it is also
        // what would make the way back an edge: everything read as up, then read as down, which is
        // a press by the rule below. Measured in
        // `orb-sim/tests/scenario_mode_question.rs`'s
        // `keys_are_not_read_with_another_window_in_front_or_a_host_that_will_not_say` — a key held
        // through an alt-tab chose a mode on the frame the window came forward, which is typing
        // elsewhere driving the game by the door this was closing.
        //
        // The first read of all goes the same way, orb starting while a key is down being the same
        // thing as coming back to one.
        if reading && !was_reading {
            self.previous = self.current;
        }
    }

    pub fn held(&self, key: u8) -> bool {
        self.current[key as usize] & DOWN != 0
    }

    pub fn pressed(&self, key: u8) -> bool {
        self.held(key) && self.previous[key as usize] & DOWN == 0
    }
}
