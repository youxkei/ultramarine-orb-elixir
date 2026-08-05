//! Which real window is in front.

use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use crate::Hwnd;

pub fn foreground() -> Hwnd {
    Hwnd(unsafe { GetForegroundWindow() } as usize)
}
