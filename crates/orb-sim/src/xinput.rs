//! The pads XInput has, as a test plugs them in.
//!
//! The other interface a pad can be on, and the reason orb reads two: a pad in XInput's second slot
//! leaves winmm's joystick 0 a phantom with no buttons and no axes, and is on none of winmm's other
//! indices at all. So a machine can have a pad here and nothing anywhere else, which is the machine orb
//! was measured on.
//!
//! What an e2e test says is what `XInputGetState` answers: which of its buttons are down and where its
//! left stick is, in XInput's own mask and its own ±32768. Turning that into the numbering the game's
//! mapping names buttons by, and into a direction, is orb's — see `orb_core::joystick`.

use std::sync::Mutex;

use orb_api::XinputPad;

/// XInput's four slots, each with a pad in it or nobody.
pub struct Xinput {
    slots: Mutex<[Option<XinputPad>; SLOTS as usize]>,
}

/// How many users XInput has, which is [`orb_api::xinput::SLOTS`] — named here too because what a
/// simulated host has to answer about is every slot orb asks about.
pub const SLOTS: u32 = orb_api::xinput::SLOTS;

impl Default for Xinput {
    fn default() -> Self {
        Self::new()
    }
}

impl Xinput {
    pub fn new() -> Self {
        Self {
            slots: Mutex::new([None; SLOTS as usize]),
        }
    }

    /// Puts a pad in that slot, with nothing pushed and its stick centred.
    ///
    /// # Panics
    /// Where `slot` is not one of XInput's [`SLOTS`].
    pub fn attach(&self, slot: u32) {
        *at(&mut self.slots.lock().unwrap()[..], slot) = Some(XinputPad::default());
    }

    /// And takes it back out, which is what a pad switched off looks like: the slot answers nobody.
    ///
    /// # Panics
    /// Where `slot` is not one of XInput's [`SLOTS`].
    pub fn unplug(&self, slot: u32) {
        *at(&mut self.slots.lock().unwrap()[..], slot) = None;
    }

    /// Pushes the buttons whose bits are set in `buttons`, which are XInput's own — see [`button`].
    ///
    /// # Panics
    /// Where that slot has no pad in it, there being nothing to push.
    pub fn pushes(&self, slot: u32, buttons: u16) {
        pad(&mut self.slots.lock().unwrap()[..], slot).buttons = buttons;
    }

    /// And the left stick, in XInput's own ±32768 measured rightwards and upwards.
    ///
    /// # Panics
    /// Where that slot has no pad in it.
    pub fn pushes_the_stick(&self, slot: u32, x: i16, y: i16) {
        let mut slots = self.slots.lock().unwrap();
        let pad = pad(&mut slots[..], slot);
        pad.left_x = x;
        pad.left_y = y;
    }

    /// `XInputGetState`.
    pub fn state(&self, slot: u32) -> Option<XinputPad> {
        *self.slots.lock().unwrap().get(slot as usize)?
    }
}

/// The slot at that index.
///
/// # Panics
/// Where `slot` is not one of XInput's [`SLOTS`], there being no slot to reach.
fn at(slots: &mut [Option<XinputPad>], slot: u32) -> &mut Option<XinputPad> {
    slots
        .get_mut(slot as usize)
        .unwrap_or_else(|| panic!("XInput has no slot {slot}: it has {SLOTS}"))
}

/// And the pad in it, which is what an e2e test pushes.
///
/// # Panics
/// Where the slot has nobody in it.
fn pad(slots: &mut [Option<XinputPad>], slot: u32) -> &mut XinputPad {
    at(slots, slot)
        .as_mut()
        .expect("a pad this e2e test put in that XInput slot")
}

/// XInput's own button mask, which is the order it reports its buttons in and **not** the order the
/// game's configuration names them by: that one is DirectInput's, and translating between the two is
/// what `orb_core::joystick` does so that *shoot decides* stays true whatever the player mapped.
///
/// Here because a simulated host answers with the host's own numbers, the way [`crate::keys`] holds the
/// virtual key codes a keyboard answers with.
pub mod button {
    pub const DPAD_UP: u16 = 0x0001;
    pub const DPAD_DOWN: u16 = 0x0002;
    pub const DPAD_LEFT: u16 = 0x0004;
    pub const DPAD_RIGHT: u16 = 0x0008;
    pub const START: u16 = 0x0010;
    pub const BACK: u16 = 0x0020;
    pub const LEFT_THUMB: u16 = 0x0040;
    pub const RIGHT_THUMB: u16 = 0x0080;
    pub const LEFT_SHOULDER: u16 = 0x0100;
    pub const RIGHT_SHOULDER: u16 = 0x0200;
    pub const A: u16 = 0x1000;
    pub const B: u16 = 0x2000;
    pub const X: u16 = 0x4000;
    pub const Y: u16 = 0x8000;
}
