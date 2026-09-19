//! The panel a test declares, and the window manager that puts a frame round a client area.
//!
//! Apart from [`display`](crate::Display), which is what the pacing reads — the blanks, the rate and
//! which window is in front. This is what the *layout* reads: how many pixels the monitor has, whether
//! it will admit to them, the frame it costs to get a client area of a given size, and the windows it
//! has been asked to make.
//!
//! Three things here are as unhelpful as the real host, because that is what the measurements say it
//! does and a kinder simulation would let a test rest on something false:
//!
//! - **The monitor lies about its size until it is asked not to.** A process that has not called
//!   `SetProcessDPIAware` is told a scaled size, and every layout worked out from it is laid out
//!   against a monitor that is not there. So the panel here has two sizes and answers the wrong one
//!   first — see [`Monitor`].
//! - **A window is not the size it was asked for.** `CreateWindowExA` is given the whole window, frame
//!   and all, and what comes back to the game is the client inside it. So the frame is subtracted on
//!   the way in and added on the way out, which is what makes "the size in `orb.yaml` is the size of
//!   the game" a claim with something to fail.
//! - **A window is where it was last put, and a test is what puts it anywhere but where it was made.**
//!   What the real host does to one afterwards — the swapchain a game builds for the window it has just
//!   made, which moves that window to the middle of the primary monitor — is nothing this can predict,
//!   so [`Windows::move_window`] is how an e2e test says it happened and orb's answer to it is what such
//!   a test asks about: `orb_core::window::keeps_its_corner` puts the window back once a frame, and the
//!   window is made hidden until that corner has held.

use std::sync::Mutex;

use orb_api::{Bar, Hwnd, Rect};

/// What a monitor reports, and the two answers it has.
///
/// A scaled desktop reports the scaled size to a process that has not said it is DPI aware and the
/// panel's own once it has, so the same monitor has two sizes and a layout is right or wrong depending
/// on which it used. Which is the whole reason orb calls `SetProcessDPIAware` before it reads a monitor
/// at all: without it a client asked for on a scaled panel is laid out against a fraction of it, and
/// Windows scales the result behind the game's back. The pair below is an e2e test's to declare.
#[derive(Clone, Copy)]
pub struct Monitor {
    /// The pixels the panel really has, which is what it reports once the process is DPI aware.
    pub real: (i32, i32),
    /// And what it reports before that: the panel divided by the scaling in force.
    pub scaled: (i32, i32),
}

impl Monitor {
    /// A panel with scaling on it: 3840x2160 reading as 2560x1440, which is 150%. One shape of host
    /// among the ones worth covering, and the numbers are this declaration's rather than any machine's.
    pub fn scaled() -> Self {
        Self {
            real: (3840, 2160),
            scaled: (2560, 1440),
        }
    }

    /// One with no scaling on it, which is every monitor an e2e test is not asking about scaling.
    pub fn plain(width: i32, height: i32) -> Self {
        Self {
            real: (width, height),
            scaled: (width, height),
        }
    }
}

/// How much bigger than its client a window is, in pixels.
///
/// A property of the host and its theme, which is why it is declared here and not worked out: the frame
/// round the caption-and-system-menu style orb asks for is whatever the theme draws, and orb's layout
/// has to hold for any of them. What an e2e test charges is the number below.
#[derive(Clone, Copy)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
}

impl Frame {
    /// A host that charges one, round `WINDOWED_STYLE`.
    pub const SOME: Self = Self {
        width: 6,
        height: 40,
    };

    /// None at all, which is what a borderless window has and what `AdjustWindowRect` adds to one.
    pub const NONE: Self = Self {
        width: 0,
        height: 0,
    };
}

/// `WS_POPUP`, which is the one style bit that decides whether there is a frame to add.
///
/// A number rather than a name out of `windows-sys`: what crosses the seam is the style the game's own
/// `CreateWindowExA` was given, and this side has to read the same bit out of it that
/// `AdjustWindowRect` does. Nothing links Windows for it.
const WS_POPUP: u32 = 0x8000_0000;

/// Whether a window of this style has a frame round its client area, decided the way the real
/// `AdjustWindowRect` decides it: a popup has none and everything else does.
pub fn has_a_frame(style: u32) -> bool {
    style & WS_POPUP == 0
}

/// A window this host has been asked to make: the whole of what was asked for, and the client it
/// came out with.
#[derive(Clone, Copy)]
pub struct Made {
    pub handle: Hwnd,
    /// The rectangle `CreateWindowExA` was given, frame included.
    pub asked: Rect,
    /// And the client inside it, which is what the game draws in.
    pub client: Rect,
    /// Whether the style it was made with is one that has a frame.
    pub framed: bool,
    /// And that style whole and unread, the same as [`ex_style`](Self::ex_style) and for its reason: what
    /// a window is asked for is the question, and `WS_VISIBLE` is a bit of it a test asks about.
    pub style: u32,
    /// And the extended style it has, whole and unread: what a window is *asked* for is the question an
    /// e2e test has, and which bits of it mean what is Windows' business and the test's.
    ///
    /// The style it was made with, plus what [`Windows::add_ex_style`] has added to it and instead of it
    /// whatever [`Windows::set_ex_style`] has written over it — which is how a game that makes its own
    /// window over is said, that being a call orb does not stand in front of.
    pub ex_style: u32,
}

/// A stack of lines this host has been asked to write in the black beside the game: where it went, and
/// what was in it.
///
/// Which is the whole of what an e2e test can ask about the status line — the height the lines got, which
/// of the two bars they went in, where the block landed, and that a shorter stack afterwards clears the
/// rows the longer one wrote in. Nothing is rasterised: what a line comes out as at an em height is the
/// declared metric, the same one a baked string is measured by.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Written {
    pub window: Hwnd,
    pub bar: Bar,
    pub lines: Vec<String>,
}

pub struct Windows {
    monitor: Mutex<Option<Monitor>>,
    frame: Mutex<Frame>,
    dpi_aware: Mutex<bool>,
    /// Whether `SetProcessDPIAware` is refused, for the launch where every size is whatever Windows
    /// scales it to and orb says so in the log.
    refuse_dpi: Mutex<bool>,
    made: Mutex<Vec<Made>>,
    /// Every window this host has been asked to show, in the order it was asked — see
    /// [`show`](Windows::show).
    shown: Mutex<Vec<Hwnd>>,
    /// The handle the next window gets. Any number that is not zero; what it is for is that orb hands
    /// back what it was given.
    next: Mutex<usize>,
    /// Every read of the monitor, and whether the process had said it was DPI aware at the time — see
    /// [`monitor_reads`](Windows::monitor_reads).
    reads: Mutex<Vec<(bool, Rect)>>,
    /// Every stack of lines written in the black beside the game, in the order it was written.
    written: Mutex<Vec<Written>>,
    /// What painting one stack of them costs this host, in microseconds.
    ///
    /// Nothing by default, a laid-out host doing no drawing — and an e2e test about *where* orb paints
    /// says the number, the way one about where it calls `PlaySounds` says `Work::sound_us`. Painting
    /// text is GDI on the window itself and a real host takes milliseconds over it, which is a cost
    /// that decides nothing while it is nowhere and decides a refresh once it is inside the span a
    /// frame has to reach its blank in.
    text_cost_us: Mutex<i64>,
}

impl Default for Windows {
    fn default() -> Self {
        Self::new()
    }
}

impl Windows {
    /// No monitor and no window, which is a host that will not say — the launch orb leaves the
    /// window as the game made it.
    pub fn new() -> Self {
        Self {
            monitor: Mutex::new(None),
            frame: Mutex::new(Frame::NONE),
            dpi_aware: Mutex::new(false),
            refuse_dpi: Mutex::new(false),
            made: Mutex::new(Vec::new()),
            shown: Mutex::new(Vec::new()),
            next: Mutex::new(0x1_0000),
            reads: Mutex::new(Vec::new()),
            written: Mutex::new(Vec::new()),
            text_cost_us: Mutex::new(0),
        }
    }

    /// What painting one stack of lines beside the game costs this host, which an e2e test about where
    /// orb paints declares the way one about the compositor declares [`Compose`](crate::Compose).
    pub fn set_text_cost(&self, us: i64) {
        *self.text_cost_us.lock().unwrap() = us;
    }

    pub(crate) fn text_cost_us(&self) -> i64 {
        *self.text_cost_us.lock().unwrap()
    }

    /// Puts a monitor there, with the frame this host puts round a window of a chosen size.
    pub fn set_monitor(&self, monitor: Monitor, frame: Frame) {
        *self.monitor.lock().unwrap() = Some(monitor);
        *self.frame.lock().unwrap() = frame;
    }

    /// Makes `SetProcessDPIAware` refuse, which leaves the monitor reporting its scaled size for the
    /// whole launch.
    pub fn refuse_dpi_awareness(&self) {
        *self.refuse_dpi.lock().unwrap() = true;
    }

    /// What the monitor reports *now*, which is the scaled size until the process has said otherwise.
    ///
    /// Written down as it is answered, so that an e2e test can say the read happened on the far side of
    /// `SetProcessDPIAware` rather than only that its answer was the real pixels — see
    /// [`monitor_reads`](Self::monitor_reads).
    ///
    /// `None` where no monitor was declared.
    pub fn monitor_now(&self) -> Option<Rect> {
        let monitor = (*self.monitor.lock().unwrap())?;
        let aware = *self.dpi_aware.lock().unwrap();
        let (width, height) = if aware { monitor.real } else { monitor.scaled };
        let answered = Rect::sized(width, height);
        self.reads.lock().unwrap().push((aware, answered));
        Some(answered)
    }

    /// Every read of the monitor, in the order they came, each with whether the process had said it
    /// was DPI aware when it was answered.
    ///
    /// Which is the whole of how an e2e test says orb read the monitor's real pixels rather than its
    /// scaled ones: an `false` here is a read taken before the process asked to be told the truth, and
    /// every size laid out from it is laid out against a monitor that is not there.
    pub fn monitor_reads(&self) -> Vec<(bool, Rect)> {
        self.reads.lock().unwrap().clone()
    }

    /// Whether the process has said it reads sizes as real pixels.
    pub fn is_dpi_aware(&self) -> bool {
        *self.dpi_aware.lock().unwrap()
    }

    pub fn set_process_dpi_aware(&self) -> bool {
        if *self.refuse_dpi.lock().unwrap() {
            return false;
        }
        *self.dpi_aware.lock().unwrap() = true;
        true
    }

    /// The whole window a client of that size needs, for a window of this style.
    pub fn adjust(&self, area: Rect, style: u32) -> Rect {
        if !has_a_frame(style) {
            return area;
        }
        let frame = *self.frame.lock().unwrap();
        Rect {
            left: area.left,
            top: area.top,
            right: area.right + frame.width,
            bottom: area.bottom + frame.height,
        }
    }

    /// Makes a window of the whole size asked for, and hands back its handle.
    ///
    /// The client is that size less the frame, which is the half of `AdjustWindowRect` that runs the
    /// other way: a caller that asked for the client's own size and did not adjust it gets a client a
    /// frame too small, and that is the defect this can catch.
    pub fn create_window(&self, asked: Rect, style: u32, ex_style: u32) -> Hwnd {
        let framed = has_a_frame(style);
        let frame = if framed {
            *self.frame.lock().unwrap()
        } else {
            Frame::NONE
        };
        let handle = {
            let mut next = self.next.lock().unwrap();
            *next += 1;
            Hwnd(*next)
        };
        self.made.lock().unwrap().push(Made {
            handle,
            asked,
            client: Rect::sized(asked.width() - frame.width, asked.height() - frame.height),
            framed,
            style,
            ex_style,
        });
        handle
    }

    /// **Moves a window that was made, whole frame and all**, and whether there was one to move.
    ///
    /// The client goes with it, a frame's worth smaller as it was when the window was made: what a real
    /// `MoveWindow` is given is the whole window, the same as `CreateWindowExA`, so a mover that asked
    /// for a client's size moves a client a frame too small — which is the defect this can catch on this
    /// side of the seam as well as at the creation.
    pub fn move_window(&self, window: Hwnd, to: Rect) -> bool {
        // The frame first and the window after it, which is the order [`create_window`](Self::create_window)
        // takes the two locks in: the other order is a second order, and two of them is what a deadlock is
        // made of.
        let charged = *self.frame.lock().unwrap();
        let mut made = self.made.lock().unwrap();
        let Some(made) = made.iter_mut().find(|made| made.handle == window) else {
            return false;
        };
        let frame = if made.framed { charged } else { Frame::NONE };
        made.asked = to;
        made.client = Rect::sized(to.width() - frame.width, to.height() - frame.height);
        true
    }

    /// A window shown, written down as asked and always taken.
    ///
    /// Nothing here draws, so showing a window is the record of it and no more — which is the whole of
    /// what an e2e test asks: that the window orb made hidden was shown, once, and not before its corner
    /// held.
    ///
    /// `pub` rather than this crate's own, because the laid-out game's host calls it too: a game that
    /// shows its own window reaches `ShowWindow` through orb, whose answer calls the host's — see
    /// `orb-e2e`'s `fake::th06nc`.
    pub fn show(&self, window: Hwnd) -> bool {
        self.shown.lock().unwrap().push(window);
        true
    }

    /// Every window this host was asked to show, in the order it was asked.
    pub fn shown(&self) -> Vec<Hwnd> {
        self.shown.lock().unwrap().clone()
    }

    /// **Bits added to a window's extended style**, and whether there was a window to add them to —
    /// `SetWindowLongPtrW` with what was there, which is what orb's own facade does.
    pub fn add_ex_style(&self, window: Hwnd, ex_style: u32) -> bool {
        let mut made = self.made.lock().unwrap();
        let Some(made) = made.iter_mut().find(|made| made.handle == window) else {
            return false;
        };
        made.ex_style |= ex_style;
        true
    }

    /// **And a whole extended style written over whatever was there**, which is the game doing to its own
    /// window what orb cannot answer: th06c's window was measured with `0x100` where orb had made it with
    /// `WS_EX_DLGMODALFRAME | WS_EX_TOOLWINDOW`, so the game had written its own over orb's. An e2e test
    /// says that happened with this.
    pub fn set_ex_style(&self, window: Hwnd, ex_style: u32) -> bool {
        let mut made = self.made.lock().unwrap();
        let Some(made) = made.iter_mut().find(|made| made.handle == window) else {
            return false;
        };
        made.ex_style = ex_style;
        true
    }

    /// The extended style a window has now, and nought for a window this host never made.
    pub fn ex_style(&self, window: Hwnd) -> u32 {
        self.made
            .lock()
            .unwrap()
            .iter()
            .find(|made| made.handle == window)
            .map_or(0, |made| made.ex_style)
    }

    /// **Whether a window is on the screen**: made with `WS_VISIBLE`, or shown since — and false for a
    /// window this host never made.
    ///
    /// `WS_VISIBLE` written out here as [`WS_POPUP`] is, and for its reason: what crosses the seam is the
    /// style the game's own call was given, and this side reads the same bit out of it that Windows does.
    pub fn is_visible(&self, window: Hwnd) -> bool {
        const WS_VISIBLE: u32 = 0x1000_0000;
        let made = self.made.lock().unwrap();
        let Some(made) = made.iter().find(|made| made.handle == window) else {
            return false;
        };
        made.style & WS_VISIBLE != 0 || self.shown.lock().unwrap().contains(&window)
    }

    pub fn client(&self, window: Hwnd) -> Option<Rect> {
        self.made
            .lock()
            .unwrap()
            .iter()
            .find(|made| made.handle == window)
            .map(|made| made.client)
    }

    /// And the whole rectangle it was asked for, frame included — which is where it is, nothing here
    /// moving a window after it was made.
    pub fn asked_for(&self, window: Hwnd) -> Option<Rect> {
        self.made
            .lock()
            .unwrap()
            .iter()
            .find(|made| made.handle == window)
            .map(|made| made.asked)
    }

    /// Every window this host has been asked to make, in the order it was asked — which is how an
    /// e2e test says that one was made and that nothing came before it to flash on the screen.
    pub fn made(&self) -> Vec<Made> {
        self.made.lock().unwrap().clone()
    }

    /// Writes a stack of lines down as written, and says it reached the screen.
    ///
    /// Always, where there is a rectangle to write in at all. What a real blit fails for is a device
    /// context that is not a window's, and orb's answer to that is to leave the bar holding whatever it
    /// held and put it right on the next call — which is behaviour an e2e test reaches by asking for a
    /// window this host has never made.
    pub(crate) fn write_lines(&self, window: Hwnd, bar: Bar, lines: &[String]) -> bool {
        if bar.area.width() <= 0 || bar.area.height() <= 0 || self.client(window).is_none() {
            return false;
        }
        self.written.lock().unwrap().push(Written {
            window,
            bar,
            lines: lines.to_vec(),
        });
        true
    }

    /// Every stack of lines written in the black beside the game, in order — and a stack of none is one
    /// of them, that being the bar cleared.
    pub fn written(&self) -> Vec<Written> {
        self.written.lock().unwrap().clone()
    }
}
