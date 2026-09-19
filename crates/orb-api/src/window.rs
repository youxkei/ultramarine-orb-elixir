//! Which window the host has in front, the sizes the host decides — what the monitor measures, the
//! frame it puts round a client area, and the client a created window came out with — the GDI a stack
//! of lines is measured and written to the window with, and the one window orb puts up itself.
//!
//! The three sizes are behind the seam for one reason, and it is not that they are Win32 calls: what
//! `orb::window` lays out is decided by numbers only the host knows, and a test that cannot move them
//! cannot reach the layout at all. The frame is the host's and its theme's, and the monitor answers two
//! different sizes for one panel depending on whether the process has said it is DPI aware. Neither is
//! a number orb may assume, which is why each is a call here and why an e2e test declares its own.

use crate::{Bar, Hwnd, Rect};

/// The window the host has in front, or [`Hwnd::NULL`] if that is none of them.
///
/// Asked of the system rather than read from the game's own `WM_ACTIVATEAPP` flag, which only says
/// what the game was last told.
pub fn foreground() -> Hwnd {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.foreground_window();
    }
    host::foreground()
}

/// Says this process reads sizes as the monitor's real pixels, and whether the host took it.
pub fn set_process_dpi_aware() -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_process_dpi_aware();
    }
    host::set_process_dpi_aware()
}

/// The primary monitor, in the desktop's own coordinates.
pub fn primary_monitor() -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.primary_monitor();
    }
    host::primary_monitor()
}

/// **Every monitor, in the desktop's own coordinates**, which is what says how they are arranged: a
/// monitor to the left of the primary one has a negative `left`, and one above it a negative `top`.
///
/// A list and not a choice, the choosing being `orb_core::window`'s — see
/// `orb_core::window::the_leftmost_of`, which is where a window orb plays in is put in a corner of.
pub fn monitors() -> Vec<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.monitors();
    }
    host::monitors()
}

/// The whole window — frame included — that a client area of `area`'s size needs.
pub fn adjust_window_rect(area: Rect, style: u32, menu: bool) -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.adjust_window_rect(area, style, menu);
    }
    host::adjust_window_rect(area, style, menu)
}

/// **Where a window is on the desktop**, frame and caption included, which is what says whether it is
/// still where orb put it — see `orb_core::window::keeps_its_corner`.
///
/// Apart from [`client_rect`] because the two are different rectangles in different coordinates: that
/// one is what the game draws into, measured from its own corner.
pub fn window_rect(window: Hwnd) -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.window_rect(window);
    }
    host::window_rect(window)
}

/// **Adds `ex_style` to a window's extended style and has the frame redrawn**, for a game that makes its
/// window over after orb framed it — see `orb_core::window::keeps_its_frame`. False where the host refused.
pub fn add_ex_style(window: Hwnd, ex_style: u32) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.add_ex_style(window, ex_style);
    }
    host::add_ex_style(window, ex_style)
}

/// **A window's extended style**, which is what says whether the frame orb put round it is still there —
/// see `orb_core::window::keeps_its_frame`, which puts it back where a game has written its own over it.
/// Nought where the host will not say, that being a style with none of the bits orb looks for.
pub fn ex_style(window: Hwnd) -> u32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.ex_style(window);
    }
    host::ex_style(window)
}

/// **Shows a window that was made hidden**, which is how a launch orb lays out keeps its window off the
/// screen until the corner is settled — see `orb_core::window::create_window_ex_w`, which strips
/// `WS_VISIBLE`, and `keeps_its_corner`, which shows it. False where the host refused.
pub fn show(window: Hwnd) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.show(window);
    }
    host::show(window)
}

/// **Whether a window is on the screen** — `IsWindowVisible`, which is the style bit and not whether
/// anything of it can be seen: a window behind another is visible, and one of a hidden parent is not.
///
/// Asked so that a launch can say when something *else* showed the window orb made hidden — see
/// `orb_core::window::keeps_its_corner`, which says so in the log.
pub fn is_visible(window: Hwnd) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.is_visible(window);
    }
    host::is_visible(window)
}

/// What the game draws into, which is what the letterbox and the bar beside it are worked out from.
pub fn client_rect(window: Hwnd) -> Option<Rect> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.client_rect(window);
    }
    host::client_rect(window)
}

/// The widest of `lines` and one line's height, at an em height of `em` pixels.
pub fn measure_lines(lines: &[String], em: i32) -> (i32, i32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.measure_lines(lines, em);
    }
    host::measure_lines(lines, em)
}

/// `lines` written into `bar` of `window`'s client area, and whether they reached the screen.
pub fn write_lines(window: Hwnd, bar: Bar, lines: &[String]) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.write_lines(window, bar, lines);
    }
    host::write_lines(window, bar, lines)
}

/// Puts up a modal with nothing to answer, for the one thing orb has to say to somebody rather than
/// to its log: that this host cannot do what orb needs of it.
///
/// Not to be called from `DllMain`. A `MessageBoxW` under the loader lock loads `comctl32` and
/// `uxtheme` and pumps messages into a process that is not finished starting, which is the textbook
/// deadlock — so this is the frame hook's to call, that being the game's main thread with a window
/// and a pump.
pub fn message_box(title: &str, text: &str) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.message_box(title, text);
    }
    host::message_box(title, text);
}

#[cfg(windows)]
use crate::real::window as host;

#[cfg(not(windows))]
mod host {
    use crate::{Bar, Hwnd, Rect, no_windows};

    pub fn foreground() -> Hwnd {
        no_windows("window::foreground")
    }
    pub fn set_process_dpi_aware() -> bool {
        no_windows("window::set_process_dpi_aware")
    }
    pub fn primary_monitor() -> Option<Rect> {
        no_windows("window::primary_monitor")
    }
    pub fn monitors() -> Vec<Rect> {
        no_windows("window::monitors")
    }
    pub fn adjust_window_rect(_area: Rect, _style: u32, _menu: bool) -> Option<Rect> {
        no_windows("window::adjust_window_rect")
    }
    pub fn window_rect(_window: Hwnd) -> Option<Rect> {
        no_windows("window::window_rect")
    }
    pub fn client_rect(_window: Hwnd) -> Option<Rect> {
        no_windows("window::client_rect")
    }
    pub fn add_ex_style(_window: Hwnd, _ex_style: u32) -> bool {
        no_windows("window::add_ex_style")
    }
    pub fn show(_window: Hwnd) -> bool {
        no_windows("window::show")
    }
    pub fn is_visible(_window: Hwnd) -> bool {
        no_windows("window::is_visible")
    }
    pub fn ex_style(_window: Hwnd) -> u32 {
        no_windows("window::ex_style")
    }
    pub fn measure_lines(_lines: &[String], _em: i32) -> (i32, i32) {
        no_windows("window::measure_lines")
    }
    pub fn write_lines(_window: Hwnd, _bar: Bar, _lines: &[String]) -> bool {
        no_windows("window::write_lines")
    }
    pub fn message_box(_title: &str, _text: &str) {
        no_windows("window::message_box")
    }
}
