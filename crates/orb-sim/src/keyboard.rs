//! A keyboard a test presses.
//!
//! Every key at once, the way `GetKeyboardState` answers, so what an e2e test sets is the state orb
//! would have read — not a press it has interpreted. Whether two keys down on one frame is a press
//! of both is then orb's answer to give and not the simulator's.

use std::sync::Mutex;

const DOWN: u8 = 0x80;

/// The keys, and whether the host will answer about them at all.
pub struct Keyboard {
    keys: Mutex<[u8; 256]>,
    /// The keys another program sent rather than a hand pressing — see [`Keyboard::sends`].
    sent: Mutex<[u8; 256]>,
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
            sent: Mutex::new([0; 256]),
            refusing: Mutex::new(false),
        }
    }

    /// Holds `key` down, or lets it up. A virtual-key code, which is what orb's menus are written
    /// in — [`crate::keys`] names the ones they read.
    pub fn set(&self, key: u8, down: bool) {
        self.keys.lock().unwrap()[key as usize] = if down { DOWN } else { 0 };
    }

    /// Sends `key` the way another program does with `SendInput`, or stops sending it.
    ///
    /// **The system takes it and an exclusive foreground DirectInput device does not see it.** Measured:
    /// keys injected that way — carrying the virtual key with its scancode, and as the scancode alone with
    /// `KEYEVENTF_SCANCODE` — are accepted (`SendInput` returns 1) and the game, whose
    /// `Controller::GetInput` takes the keyboard `DISCL_EXCLUSIVE | DISCL_FOREGROUND`, sat idle into its
    /// attract demo twice. So a sent key is down as far as `GetKeyboardState` is concerned — see
    /// [`state`](Self::state) — and [`held`](Self::held) is what a device that refuses them reads instead.
    ///
    /// Which is the whole of why `--sent-keys` exists: every measurement of a front end nobody was there
    /// to press keys at was taken through it.
    pub fn sends(&self, key: u8, sending: bool) {
        self.sent.lock().unwrap()[key as usize] = if sending { DOWN } else { 0 };
    }

    /// Lets everything up, which is what an e2e test does between two presses of the same key: a
    /// press is an edge, so a key held across frames is one press however many frames it is held.
    pub fn release_all(&self) {
        *self.keys.lock().unwrap() = [0; 256];
        *self.sent.lock().unwrap() = [0; 256];
    }

    pub fn refuse(&self, refusing: bool) {
        *self.refusing.lock().unwrap() = refusing;
    }

    /// `GetKeyboardState`: every key a hand is holding *and* every key another program sent, which is what
    /// makes that read the one a sent key is seen through.
    pub fn state(&self) -> Option<[u8; 256]> {
        if *self.refusing.lock().unwrap() {
            return None;
        }
        let mut keys = *self.keys.lock().unwrap();
        for (key, sent) in keys.iter_mut().zip(self.sent.lock().unwrap().iter()) {
            *key |= *sent;
        }
        Some(keys)
    }

    /// And the keys a hand is really holding, with nothing another program sent among them — what a device
    /// held `DISCL_EXCLUSIVE | DISCL_FOREGROUND` answers with, and the read `--sent-keys` exists to get the
    /// game off.
    pub fn held(&self, key: u8) -> bool {
        self.keys.lock().unwrap()[key as usize] & DOWN != 0
    }
}

/// The virtual-key codes orb's own menus read, so an e2e test presses the key somebody would rather
/// than a number.
///
/// Here rather than in `orb-core`, where the same six are `const` and private: what those are is
/// orb's business, and an e2e test naming them itself is what makes it fail if orb changes which key
/// answers.
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
