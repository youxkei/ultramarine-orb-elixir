//! Which keys the real keyboard has down.

use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardState;

pub fn state() -> Option<[u8; 256]> {
    let mut keys = [0u8; 256];
    // Zero is the failure, and it happens: the call wants a message queue on the calling thread,
    // and orb reads the keyboard from the game's own thread where there is one. A caller that took
    // the array anyway would read whatever the last successful call left, which is a key stuck down.
    (unsafe { GetKeyboardState(keys.as_mut_ptr()) } != 0).then_some(keys)
}
