//! Which keys the real keyboard has down.

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyboardState};

/// The bit `GetAsyncKeyState` and a `GetKeyboardState` snapshot both carry for a key that is down, and
/// nothing else: the low bit is a key that has been toggled, and a caps lock left on is not a key down.
const DOWN: u16 = 0x8000;

/// The same bit as the byte of a `GetKeyboardState` snapshot, which is what [`device_state`] answers in.
const A_KEY: u8 = 0x80;

pub fn state() -> Option<[u8; 256]> {
    let mut keys = [0u8; 256];
    // Zero is the failure, and it happens: the call wants a message queue on the calling thread,
    // and orb reads the keyboard from the game's own thread where there is one. A caller that took
    // the array anyway would read whatever the last successful call left, which is a key stuck down.
    (unsafe { GetKeyboardState(keys.as_mut_ptr()) } != 0).then_some(keys)
}

/// **What the machine's own keyboard has down**, read one key at a time — `GetAsyncKeyState` answers
/// for a key the way a device does, and it is the one read here that does not go quiet with the focus.
pub fn device_state() -> [u8; 256] {
    let mut keys = [0u8; 256];
    for (key, down) in keys.iter_mut().enumerate() {
        let key = i32::try_from(key).expect("an index of a 256-byte array is a virtual key");
        if unsafe { GetAsyncKeyState(key) } as u16 & DOWN != 0 {
            *down = A_KEY;
        }
    }
    keys
}
