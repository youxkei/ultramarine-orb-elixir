//! Where the mouse pointer is, and whether the host is drawing it.
//!
//! Behind the seam because both halves are the host's: the pointer moves when somebody moves the
//! mouse, and whether it is drawn is a counter Windows keeps for this process. A test that cannot move
//! a mouse and cannot read back whether the pointer is on the screen cannot reach what orb does about
//! either.
//!
//! **The pointer, and not a cursor.** A cursor in this tree is the mark beside the item a question of
//! orb's is on — `menu_ui`'s and `retry_ui`'s — and the two would read as one word for two things. The
//! Win32 calls are `GetCursorPos` and `ShowCursor`, and they are named beside the place each is made.

/// `GetCursorPos` — where the pointer is, in the desktop's own coordinates.
///
/// `None` where the host would not say, which is a pointer orb cannot follow: the call fails on a
/// desktop that is not the input one, and a locked session is that.
pub fn position() -> Option<(i32, i32)> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.mouse_position();
    }
    host::position()
}

/// `ShowCursor` — the display counter moved one step, and what it reads afterwards.
///
/// **The counter is what the call keeps, and the answer is why this one comes back.** Windows draws
/// the pointer while the counter is not negative and every call moves it by one, so what orb asks for
/// is a step over that edge rather than a state — and which side of it the counter is now on is not
/// something orb could otherwise know.
pub fn show(showing: bool) -> i32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.show_mouse(showing);
    }
    host::show(showing)
}

#[cfg(windows)]
use crate::real::mouse as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn position() -> Option<(i32, i32)> {
        no_windows("mouse::position")
    }
    pub fn show(_showing: bool) -> i32 {
        no_windows("mouse::show")
    }
}
