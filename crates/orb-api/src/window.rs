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

/// One frame of a window: the window's size in its own pixels, and the pixels as rows from the top.
///
/// `rows` is `width * height * 4` bytes: four bytes for every pixel, the rows one after another with no
/// padding between them. The four bytes are blue, green, red and an unused fourth — the order a BMP
/// keeps them in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rows: Vec<u8>,
    /// **How many bytes the frame's own texture had between rows of it**, which is the picture's width four
    /// times over unless the host's texture pads its rows — carried so that a picture that came out wrong can
    /// say whether its rows were the reason.
    pub row_pitch: u32,
}

/// **One frame of `window`, captured by the host in the window's own pixels.**
///
/// **Which is the window and not the buffer it shows**: the desktop composes it, so a window whose frame
/// holds one thing and whose buffer another is photographed as it is — and a window that runs off every
/// monitor answers **black** for the part over nothing (measured 2026-11-02, a `5120x2880` window on a
/// `3840x2160` desktop). A caller that would rather have the frame holds the window where it fits.
///
/// `None` where the window could not be captured: a host with no capture to give, a window that is not
/// there, or a frame that did not arrive before the wait ran out. A laid-out host answers the frame its
/// canvas drew, through `Win::capture_window`.
pub fn capture(window: Hwnd) -> Option<Picture> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.capture_window(window);
    }
    #[cfg(not(feature = "sim"))]
    {
        host::capture(window)
    }
    #[cfg(feature = "sim")]
    {
        let _ = window;
        None
    }
}

/// **Why the last [`capture`] answered nothing**, in the words of the step that gave up — the seam keeps it
/// because a capture that fails is one line in a launch's record rather than a picture nobody can account for.
pub fn why_the_capture_failed() -> String {
    #[cfg(all(windows, not(feature = "sim")))]
    {
        crate::real::capture::why_the_last_capture_failed()
    }
    // A laid-out host answers through its own `capture_window`, whose `None` is the test's own to read: there
    // is no capture of Windows' to name a step of.
    #[cfg(any(not(windows), feature = "sim"))]
    {
        "the host's capture answered nothing".to_owned()
    }
}

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

/// **Whether the window in front belongs to the process orb is inside**, which is the question a game's
/// window handle cannot always answer — see `real::window::foreground_is_in_this_process`.
pub fn foreground_is_in_this_process() -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.window_is_this_processes(win.foreground_window());
    }
    host::foreground_is_in_this_process()
}

/// **Brings a window to the front and gives it the keyboard** — `SetForegroundWindow` — and whether the host did.
pub fn bring_forward(window: Hwnd) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.bring_forward(window);
    }
    host::bring_forward(window)
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

/// **Shows a window that was made hidden, and does not activate it** — `ShowWindow` with `SW_SHOWNA` — which
/// is how a launch orb lays out keeps its window off the screen until the corner is settled: see
/// `orb_core::window::create_window_ex_w`, which strips `WS_VISIBLE`, and `keeps_its_corner`, which shows it.
/// False where the host refused.
pub fn show_without_activating(window: Hwnd) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.show_without_activating(window);
    }
    host::show_without_activating(window)
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

/// **Takes what is waiting in this thread's message queue and gives it to the window it is for**, and
/// answers whether a quit was among them.
///
/// **What it is for is a stretch orb spends inside one frame**, which is a stretch the game's own loop is
/// not pumping in: a playback of a chapter deep in a run is a minute of updates inside one frame, and a
/// window that has not answered a message in a minute is one Windows draws a ghost of and offers to
/// close. A launch put back at stage 4's chapter ended on the game's own `exit()` with no exception at
/// all — `Exit process, code 0` under `mainCRTStartup` — which is what a close reaching a window looks
/// like from the inside.
///
/// Everything waiting rather than a chosen few, the game's own loop making no such choice either.
pub fn pump() -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.pump();
    }
    host::pump()
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
    pub fn foreground_is_in_this_process() -> bool {
        no_windows("window::foreground_is_in_this_process")
    }
    pub fn bring_forward(_window: Hwnd) -> bool {
        no_windows("window::bring_forward")
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
    #[cfg(not(feature = "sim"))]
    pub fn capture(_window: Hwnd) -> Option<super::Picture> {
        no_windows("window::capture")
    }
    pub fn add_ex_style(_window: Hwnd, _ex_style: u32) -> bool {
        no_windows("window::add_ex_style")
    }
    pub fn show_without_activating(_window: Hwnd) -> bool {
        no_windows("window::show_without_activating")
    }
    pub fn is_visible(_window: Hwnd) -> bool {
        no_windows("window::is_visible")
    }
    pub fn pump() -> bool {
        no_windows("window::pump")
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
