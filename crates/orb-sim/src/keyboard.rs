//! A keyboard a test presses.
//!
//! Every key at once, the way `GetKeyboardState` answers, so what a scenario sets is the state orb
//! would have read — not a press it has interpreted. Whether two keys down on one frame is a press
//! of both is then orb's answer to give and not the simulator's.

use std::sync::Mutex;

const DOWN: u8 = 0x80;

/// The keys, and whether the host will answer about them at all.
pub struct Keyboard {
    keys: Mutex<[u8; 256]>,
    /// `GetKeyboardState` returning zero, which happens on a thread with no message queue. Off by
    /// default: the ordinary case is a host that answers, and a test that wants the failure asks for
    /// it — orb reads it as nothing being down, which is a rule worth being able to check.
    refusing: Mutex<bool>,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            keys: Mutex::new([0; 256]),
            refusing: Mutex::new(false),
        }
    }

    /// Holds `key` down, or lets it up. A virtual-key code, which is what orb's menus are written
    /// in — [`crate::keys`] names the ones they read.
    pub fn set(&self, key: u8, down: bool) {
        self.keys.lock().unwrap()[key as usize] = if down { DOWN } else { 0 };
    }

    /// Lets everything up, which is what a scenario does between two presses of the same key: a
    /// press is an edge, so a key held across frames is one press however many frames it is held.
    pub fn release_all(&self) {
        *self.keys.lock().unwrap() = [0; 256];
    }

    pub fn refuse(&self, refusing: bool) {
        *self.refusing.lock().unwrap() = refusing;
    }

    pub fn state(&self) -> Option<[u8; 256]> {
        (!*self.refusing.lock().unwrap()).then(|| *self.keys.lock().unwrap())
    }
}

/// The virtual-key codes orb's own menus read, so a scenario presses the key somebody would rather
/// than a number.
///
/// Here rather than in `orb-core`, where the same six are `const` and private: what those are is
/// orb's business, and a test naming them itself is what makes a scenario fail if orb changes which
/// key answers.
pub mod keys {
    pub const RETURN: u8 = 0x0d;
    pub const ESCAPE: u8 = 0x1b;
    pub const UP: u8 = 0x26;
    pub const DOWN: u8 = 0x28;
    /// The game's bomb key, which its own menus read as back.
    pub const X: u8 = 0x58;
    /// And its shot key, which they read as decide.
    pub const Z: u8 = 0x5a;
}
