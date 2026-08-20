//! Keyboard and pad reading for orb's own keys.
//!
//! Read directly rather than through the game's input state, because the keys
//! that matter most are the ones pressed while the game's update — and with it
//! its input handling — is deliberately not running.

use orb_api::{Hwnd, keyboard, window};

use crate::game::Pad;

const DOWN: u8 = 0x80;

pub struct Keyboard {
    previous: [u8; 256],
    current: [u8; 256],
    /// Whether the last read was one orb was allowed to make. What this is for is the frame after it
    /// was not — see `poll`.
    reading: bool,
}

/// Beside `new` because the module is public, and one with nothing read yet is exactly what `new` makes.
/// Nothing calls it, which is why every coverage run reports it as never entered — the six of these are all
/// the same one.
impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

/// What the pads are pushing now and what they were pushing last frame, so that a push can be told from a
/// hold: the same pair the keyboard keeps, and for the same reason — a step is a press and the key held with
/// one is a hold.
///
/// **For the judging hand**, which is a hand on a controller: somebody watching a stage go by has one in
/// their hands and the keyboard across the room. Every key that pass uses is a button of the game's own —
/// see `runtime`'s `keys` and [`crate::game::Pad`].
#[derive(Default)]
pub struct Pads {
    previous: Pad,
    current: Pad,
    /// Whether the pair describes two frames of reading. What it is for is the first frame after a stretch
    /// of not reading at all — see [`Pads::forget`].
    reading: bool,
}

impl Pads {
    pub const fn new() -> Self {
        Self {
            previous: Pad::none(),
            current: Pad::none(),
            reading: false,
        }
    }

    /// Takes this frame's reading, keeping the one before it.
    ///
    /// **Whatever is pushed on the first frame of reading counts as already held**, which is the rule the
    /// keyboard's own read follows and for the same reason: a button held from before the pass began would
    /// otherwise be a press on the frame it starts, and the press it would be read as is a boundary put
    /// down or a step taken that nobody asked for.
    pub fn poll(&mut self, pushing: Pad) {
        if std::mem::replace(&mut self.reading, true) {
            self.previous = std::mem::replace(&mut self.current, pushing);
        } else {
            self.previous = pushing;
            self.current = pushing;
        }
    }

    /// Says nothing is being read, for the frames a pass is not watching: the pads are not asked on those —
    /// a read of the device the game holds costs the game's own poll — and what is pushed when it starts
    /// watching again is held rather than pressed.
    pub fn forget(&mut self) {
        self.reading = false;
    }

    /// Whether `of` is pushed now and was not last frame, which is a press.
    pub fn pressed(&self, of: impl Fn(&Pad) -> bool) -> bool {
        of(&self.current) && !of(&self.previous)
    }

    /// And whether it is pushed at all, which is what a key held with another one is read as.
    pub fn held(&self, of: impl Fn(&Pad) -> bool) -> bool {
        of(&self.current)
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
        // `orb-e2e/src/mode_question.rs`'s
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
