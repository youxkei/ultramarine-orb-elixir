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
//! - **Nothing moves a window after it is made.** What the real host does to one afterwards — a
//!   borderless tool resizing a client three and a half seconds later, which happened on this machine —
//!   is not here, and `TODO.md` keeps it under *What else on the desktop does to the window*.

use std::sync::Mutex;

use orb_api::{Bar, Hwnd, Rect};

/// What a monitor reports, and the two answers it has.
///
/// Measured on this machine, a 3840x2160 panel at 150%: `EnumDisplayMonitors` and everything else
/// report **2560x1440** to a process that has not said it is DPI aware, and **3840x2160** once it has.
/// Which is the whole reason orb calls `SetProcessDPIAware` before it reads a monitor at all: without
/// it a 1280x720 client asked for on that panel is laid out against two thirds of it, and Windows
/// scales the result behind the game's back.
#[derive(Clone, Copy)]
pub struct Monitor {
    /// The pixels the panel really has, which is what it reports once the process is DPI aware.
    pub real: (i32, i32),
    /// And what it reports before that: the panel divided by the scaling in force.
    pub scaled: (i32, i32),
}

impl Monitor {
    /// This machine's: 3840x2160 reading as 2560x1440, which is 150%.
    pub fn measured() -> Self {
        Self {
            real: (3840, 2160),
            scaled: (2560, 1440),
        }
    }

    /// One with no scaling on it, which is every monitor a scenario is not asking about scaling.
    pub fn plain(width: i32, height: i32) -> Self {
        Self {
            real: (width, height),
            scaled: (width, height),
        }
    }
}

/// How much bigger than its client a window is, in pixels.
///
/// Measured on this machine, round the caption-and-system-menu style orb asks for: a `1280x720`
/// client came out of a **1286x760** window, so **6x40**. A property of the host and its theme, which
/// is why it is declared here and not worked out.
#[derive(Clone, Copy)]
pub struct Frame {
    pub width: i32,
    pub height: i32,
}

impl Frame {
    /// This machine's, round `WINDOWED_STYLE`.
    pub const MEASURED: Self = Self {
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
}

/// A stack of lines this host has been asked to write in the black beside the game: where it went, and
/// what was in it.
///
/// Which is the whole of what a scenario can ask about the status line — the height the lines got, which
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
    /// The handle the next window gets. Any number that is not zero; what it is for is that orb hands
    /// back what it was given.
    next: Mutex<usize>,
    /// Every read of the monitor, and whether the process had said it was DPI aware at the time — see
    /// [`monitor_reads`](Windows::monitor_reads).
    reads: Mutex<Vec<(bool, Rect)>>,
    /// Every stack of lines written in the black beside the game, in the order it was written.
    written: Mutex<Vec<Written>>,
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
            next: Mutex::new(0x1_0000),
            reads: Mutex::new(Vec::new()),
            written: Mutex::new(Vec::new()),
        }
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
    /// Written down as it is answered, so that a scenario can say the read happened on the far side of
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
    /// Which is the whole of how a scenario says orb read the monitor's real pixels rather than its
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
    pub fn create_window(&self, asked: Rect, style: u32) -> Hwnd {
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
        });
        handle
    }

    pub fn client(&self, window: Hwnd) -> Option<Rect> {
        self.made
            .lock()
            .unwrap()
            .iter()
            .find(|made| made.handle == window)
            .map(|made| made.client)
    }

    /// Every window this host has been asked to make, in the order it was asked — which is how a
    /// scenario says that one was made and that nothing came before it to flash on the screen.
    pub fn made(&self) -> Vec<Made> {
        self.made.lock().unwrap().clone()
    }

    /// Writes a stack of lines down as written, and says it reached the screen.
    ///
    /// Always, where there is a rectangle to write in at all. What a real blit fails for is a device
    /// context that is not a window's, and orb's answer to that is to leave the bar holding whatever it
    /// held and put it right on the next call — which is behaviour a scenario reaches by asking for a
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
