//! Where the real pointer is, and the display counter the real Windows draws it by.

use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, ShowCursor};

pub fn position() -> Option<(i32, i32)> {
    let mut point = POINT { x: 0, y: 0 };
    (unsafe { GetCursorPos(&mut point) } != 0).then_some((point.x, point.y))
}

pub fn show(showing: bool) -> i32 {
    unsafe { ShowCursor(i32::from(showing)) }
}
