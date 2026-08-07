//! The seam between orb and the host it runs on.
//!
//! What orb gets from the host goes through one of the modules here. Each of them is a
//! facade of free functions with two answers behind it: the real one, under
//! `#[cfg(windows)]`, and whatever [`Win`] implementation a test has installed. Nothing in
//! any signature is a `windows-sys` type, which is what lets the crates over this seam —
//! `orb-core`, `orb-sim` — be built and tested on a host that is not Windows.
//!
//! **The host and not Windows**, which is a distinction with one member: [`Win::spin_once`] is the
//! `pause` instruction and no call at all. What decides whether something belongs here is whether it
//! is the host doing something to orb that no test could otherwise decide — see
//! [docs/adr/0007](../../../docs/adr/0007-the-spins-pause-is-behind-the-seam.md) — and not whether
//! there is a Win32 function on the other side of it.
//!
//! The call sites keep their shape. `mem::read(address)` is what it was before the seam went
//! in, and the alternative — making every caller carry a `&dyn Win` — would have rewritten
//! two thousand lines of structure walking to say nothing new. The install point is a
//! thread-local instead, because the test harness runs tests side by side in one process and
//! a simulated Windows in a static would be two tests writing each other's game.

use std::ops::Range;
use std::path::{Path, PathBuf};

pub mod clock;
pub mod display;
pub mod keyboard;
pub mod logfile;
pub mod mem;
pub mod module;
pub mod process;
pub mod thread;
pub mod window;

#[cfg(windows)]
mod real;

#[cfg(feature = "sim")]
mod install;
#[cfg(feature = "sim")]
pub use install::{Installed, install, installed};

/// An opaque OS window handle. On Windows it is the `HWND` reinterpreted as a `usize`; plain
/// data so that the types the seam traffics in build on any host.
///
/// orb never asks what is inside one. It reads the game's window out of the game's memory and
/// hands it back to Windows, so the only thing it has to be is the same number coming out as
/// went in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Hwnd(pub usize);

impl Hwnd {
    pub const NULL: Self = Self(0);

    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// A rectangle in whatever coordinates the call that answered it works in: a monitor's own for the
/// monitor, and the window's own for a client area.
///
/// The same four fields in the same order as Windows' `RECT`, and exclusive at the right and bottom
/// as that one is, so the arithmetic either side of the seam is the arithmetic that was already
/// written. Plain data, so the types the seam traffics in build on any host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    /// One of that size at the origin, which is what a client area is measured as.
    pub fn sized(width: i32, height: i32) -> Self {
        Self {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        }
    }

    pub fn width(self) -> i32 {
        self.right - self.left
    }

    pub fn height(self) -> i32 {
        self.bottom - self.top
    }
}

/// What a region of the address space is, as far as anything reading it can tell.
///
/// [`Win::vtable_in_image`] is the one question orb asks that tells them apart: a live COM
/// object's vtable pointer lands in a mapped image, and the stale pointer left in a block the
/// game's allocator did not scrub does not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A mapped executable — the game's own image, or a DLL it loaded.
    Image,
    /// Anything the game allocated.
    Private,
}

/// A simulated Windows, for the tests that run with no Windows under them.
///
/// Taken `&self` and `Send + Sync` so that the threads a simulated game runs on can be handed
/// the same one: an audio thread reading its own copy of the game would agree with nothing.
///
/// Byte-level throughout, with no method generic over what is being read. The facades over it
/// are generic — `mem::read::<T>` — and do the marshalling on this side of the trait, because
/// a trait with a generic method is not one that can be installed as a `dyn Win`.
pub trait Win: Send + Sync + 'static {
    // --- reads and writes of the game's memory -------------------------------
    //
    // # Panics
    // Where no region holds the whole of the range asked for, or the one that does is not
    // readable. A real read there takes the process down, so a test reaching one has a wrong
    // address in it and saying which is the whole of the help there is to give. Falling
    // through to the real address space instead is the failure the seam exists to remove: it
    // would read the test binary's own image and call it the game's.

    fn read_bytes(&self, address: usize, len: usize) -> Vec<u8>;
    fn write_bytes(&self, address: usize, source: &[u8]);
    fn fill_bytes(&self, address: usize, byte: u8, len: usize);

    /// Reads only where the whole range can be read at all, for chasing pointers out of the
    /// game's structures that may not be valid yet or any more. `None` where the range is
    /// unmapped, reserved without being committed, or guarded.
    ///
    /// Alignment is the facade's to refuse, not this: it is a property of the type being read
    /// and not of the address space.
    fn read_committed_bytes(&self, address: usize, len: usize) -> Option<Vec<u8>>;

    /// Puts back any page of `address..address + len` that has gone since a snapshot, so a
    /// restore has somewhere to write, and says whether the whole of it is now there.
    fn commit(&self, address: usize, len: usize) -> bool;

    /// Whether `address` holds a pointer into a mapped image, which is what a live COM
    /// object's vtable pointer looks like.
    fn vtable_in_image(&self, address: usize) -> bool;

    /// The committed regions the game owns, as `(base, len)` — what a snapshot walks. The
    /// data range comes first, as a real game's does; a region of the game's code is left out,
    /// since a chapter is a copy of the game's state and not of itself.
    fn game_regions(&self, data: &Range<usize>) -> Vec<(usize, usize)>;

    /// Says a range is orb's own, so that [`private_regions`](Win::private_regions) leaves it out.
    ///
    /// What orb keeps there is copies of the game's memory, and `self_check` finds memory the game
    /// changed outside a snapshot by walking the process's private pages — so a walk that counted
    /// orb's copies would report them as the game having changed them.
    ///
    /// Told rather than asked for: the memory itself is an ordinary allocation of orb's, because a
    /// buffer handed out by one side of the seam and given back after the other one has been
    /// installed is a `VirtualFree` of a heap pointer. Measured, before it was this way round — the
    /// suite crashed taking a chapter's snapshots down after the simulated Windows had gone.
    fn keep_out_of_private_regions(&self, base: usize, len: usize);

    /// And that it is not any more.
    fn count_private_region_again(&self, base: usize);

    /// Every committed, private, readable region in the process, as `(base, len)`, less the ones
    /// [`keep_out_of_private_regions`](Win::keep_out_of_private_regions) named.
    ///
    /// What `self_check` fingerprints, to find memory a snapshot did not cover that something changed
    /// anyway. A diagnostic, so a host with nothing to say answers with nothing and the check reports
    /// on what it did cover.
    fn private_regions(&self) -> Vec<(usize, usize)>;

    /// The regions of the process heap, as `(base, len)`.
    ///
    /// Apart from [`private_regions`](Win::private_regions) because `self_check` counts changes here
    /// rather than listing them: Rust's allocator and the DirectX runtimes share this heap, so a
    /// change in it is nobody's in particular.
    fn process_heap_regions(&self) -> Vec<(usize, usize)>;

    // --- the clock ------------------------------------------------------------

    /// `QueryPerformanceCounter`. Monotonic; the unit is [`frequency`](Win::frequency) ticks a
    /// second.
    fn counter(&self) -> i64;
    /// `QueryPerformanceFrequency` — ticks a second, and zero where there is no counter.
    fn frequency(&self) -> i64;

    /// Waits `ticks` of the counter out on a `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` waitable
    /// timer, made on first use and kept for the rest of the run — and `false` where it could not
    /// be made at all.
    ///
    /// The counter's own ticks and not milliseconds, because a millisecond is the granularity being
    /// left behind: this wait is aimed at a frame's own deadline, which is where the input lag left
    /// to win is. See
    /// [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md).
    ///
    /// The handle stays behind the seam, which is the rule the thread ids and the log's token
    /// already follow. A simulated host waits exactly as long as it was asked to: a real one
    /// overshoots by whatever the host's own wake delay is, and modelling that would make every
    /// assertion about a wait a statement about the overshoot instead of about the pacing's
    /// arithmetic.
    ///
    /// `false` is a host orb does not run on rather than a wait to make some other way — the flag
    /// is Windows 10 1803's and nothing older is a target — so the caller says so and stops.
    fn wait(&self, ticks: i64) -> bool;

    /// One turn of the spin that finishes the wait — the `pause` instruction on a real host.
    ///
    /// Behind the seam for the same reason [`counter`](Win::counter) is: it costs time, and a simulated
    /// host that made it free would have the spin reach its deadline only through the counter's own read
    /// cost, one tick at a time — fifteen thousand turns of a real loop per simulated frame. That it is
    /// an instruction rather than a call is nothing to the seam, which is the host's side of orb and not
    /// Win32's.
    fn spin_once(&self);

    // --- the one failure that ends a launch -------------------------------------
    //
    // Behind the seam because there is behaviour here no scenario could otherwise reach: one that
    // raised a real `MessageBoxW` would wait for a click that is never coming, and one that really
    // exited would take the harness's child with it.

    /// `MessageBoxW`, modal and with nothing to answer — what a host that cannot do what orb needs
    /// is told to say, where somebody will read it.
    fn message_box(&self, title: &str, text: &str);

    /// `ExitProcess`, which does not return on a real host. A simulated one writes down that it was
    /// asked and does return, so the caller must not carry on as though it had not been called.
    fn exit_process(&self, code: u32);

    // --- the display and the compositor ---------------------------------------

    /// The refresh rate of the monitor the window is on, in whole Hz — `MonitorFromWindow`, then
    /// `EnumDisplaySettingsW`. `None` for a window that is not there or a rate that will not be
    /// given.
    ///
    /// Whole Hz, so the NTSC-derived rates come back short: 119.88 reports as 119.
    fn monitor_refresh(&self, window: Hwnd) -> Option<u32>;
    /// The desktop's refresh rate, in whole Hz — `GetDC` and `GetDeviceCaps(VREFRESH)`. What the
    /// pacing starts from, before there is a window to ask about.
    fn desktop_refresh(&self) -> Option<u32>;
    /// `DwmGetCompositionTimingInfo`, for the desktop. `None` when the compositor will not say.
    ///
    /// The rate this describes is the compositor's own, which follows one monitor of the desktop
    /// and need not be the one being drawn to. The pacing checks the two against each other
    /// rather than assuming, so a simulated one must be able to disagree with
    /// [`monitor_refresh`](Win::monitor_refresh).
    fn composition(&self) -> Option<Composition>;
    /// `DwmFlush`, and `false` where it failed — the compositor turned off, or a session change.
    ///
    /// Waits for the compositor to compose the next frame rather than for the next blank as such,
    /// so it returns at the blank *the frame just handed over* reached. That is the whole of how
    /// the pacing knows whether a frame made its blank, so a simulated compositor has to model it
    /// that way and not as "wait for the next blank".
    fn flush(&self) -> bool;

    // --- threads --------------------------------------------------------------

    /// `GetCurrentThreadId`. Never zero, which is what lets the log tell the frame's own thread
    /// from every other one without a lock.
    fn current_thread_id(&self) -> u32;

    /// Stops every thread the game made except the caller's and `audio`, and answers with the ones
    /// stopped so they can be started again.
    ///
    /// Only the threads the game created itself, which the host is told about as they are made — see
    /// [`Win::register_thread`]. Suspending everything, which is what enumerating the process's
    /// threads amounts to, also stops DirectSound's mixer and the graphics driver's workers, and
    /// stopping those for the length of a copy is audible as the sound breaking up. The game's audio
    /// thread is left running for the same reason, even though it is the game's.
    ///
    /// Ids and not handles, so that nothing about a handle crosses the seam. A host that knows of no
    /// such threads answers with none, which is a copy taken with nothing to hold still.
    fn suspend_game_threads(&self, audio: Option<u32>) -> Vec<u32>;

    /// Starts them again.
    fn resume_threads(&self, ids: &[u32]);

    /// Remembers a thread the game has just created, and answers whether it can be stopped later.
    ///
    /// The id and not a handle: the noticing is orb's — an import hook on `CreateThread` — and
    /// opening a handle of our own is the host's, because what a handle is belongs on this side.
    fn register_thread(&self, id: u32) -> bool;

    // --- the log --------------------------------------------------------------

    /// Opens the log for appending, starting it over rather than appending where it has already
    /// grown past `max_bytes`. `None` where it could not be opened at all, which orb carries on
    /// without.
    fn open_log(&self, path: &Path, max_bytes: u64) -> Option<LogFile>;
    /// Appends bytes to a log opened by [`open_log`](Win::open_log). Bytes rather than a `&str`
    /// because the line endings orb writes are the file's, not the platform's.
    fn write_log(&self, file: LogFile, bytes: &[u8]);
    fn close_log(&self, file: LogFile);

    // --- loaded modules -------------------------------------------------------

    /// Where a loaded module's file is; `None` for the exe of this process, which is the game's.
    fn module_path(&self, module: Option<usize>) -> Option<PathBuf>;
    /// Whether a module is loaded into this process at all — asked apart from
    /// [`proc_address`](Win::proc_address) because the log tells the two failures apart.
    fn module_loaded(&self, module: &str) -> bool;
    // --- windows ---------------------------------------------------------------

    /// `GetForegroundWindow`. The pacing asks it because counting refreshes against a window that
    /// is not in front comes out wrong.
    fn foreground_window(&self) -> Hwnd;

    /// `SetProcessDPIAware`, and whether the host took it. Said once, before there is a window.
    ///
    /// Behind the seam because every size orb lays out is measured against what the host reports
    /// *after* this, and no test can otherwise make a host report two different sizes for one
    /// monitor. Measured on this machine: a 3840x2160 panel reads as 2560x1440 until this is called.
    fn set_process_dpi_aware(&self) -> bool;

    /// `MonitorFromPoint(0,0)` and `GetMonitorInfoW` — the primary monitor, in the desktop's own
    /// coordinates. `None` where the host will not say, which leaves the window as the game made it.
    fn primary_monitor(&self) -> Option<Rect>;

    /// `AdjustWindowRect` — the whole window a client area of that size needs, frame included.
    ///
    /// Behind the seam because how thick a frame is belongs to the host and to its theme: the size in
    /// `orb.yaml` is the size of what is *inside* the window, so the frame is what stands between the
    /// number asked for and the window to ask for. Measured on this machine: 6x40 round the
    /// caption-and-system-menu style `orb::window` asks for.
    ///
    /// `None` where the host refused, which leaves the rectangle as the client area — a window a
    /// frame too small rather than no window.
    fn adjust_window_rect(&self, area: Rect, style: u32, menu: bool) -> Option<Rect>;

    /// `GetClientRect` — what the game draws into, which is not the size that was asked for whenever
    /// anything between here and the screen has an opinion. `None` for a window that is not there.
    fn client_rect(&self, window: Hwnd) -> Option<Rect>;

    // --- the keyboard ----------------------------------------------------------

    /// `GetKeyboardState` — every key at once, `0x80` set on the ones that are down. `None` where
    /// the call failed, which orb reads as nothing being down.
    ///
    /// The whole array rather than a key at a time, because that is what the call answers: asking
    /// per key would be several reads of a state that may have moved between them, and orb's menus
    /// read six keys a frame.
    fn keyboard_state(&self) -> Option<[u8; 256]>;

    // --- loaded modules --------------------------------------------------------

    /// The address of a function exported by an already-loaded module — `mmioSeek` out of the
    /// winmm the game has loaded — or `None` where either is not there.
    ///
    /// Named rather than handed a handle because a handle is the thing that does not survive the
    /// seam: what the caller knows is `"winmm.dll"` and `"mmioSeek"`, and the lookup of both
    /// belongs on the far side.
    fn proc_address(&self, module: &str, name: &str) -> Option<usize>;
}

/// What the compositor says about the blanks, all of it out of one `DwmGetCompositionTimingInfo`
/// as the real one answers it.
///
/// One value rather than a call per field because that is what the call is: asking it three times
/// for three fields is three trips through the compositor, and the pacing asks it before a flush,
/// out of the millisecond the frame has to reach that flush in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Composition {
    /// How far apart the blanks are, in performance-counter ticks — `qpcRefreshPeriod`.
    pub refresh_period: i64,
    /// Which composition was the last, counted from when the compositor started — `cRefresh`.
    /// Compositions of this window rather than refreshes of the display, which is why the pacing
    /// does not count cadence with it.
    pub refresh: i64,
    /// When the last blank was, on the performance counter — `qpcVBlank`. Zero when it will not
    /// say.
    pub vblank: i64,
    /// How many frames the compositor could not show at the refresh they were aimed at —
    /// `cFramesLate`. Reported and not acted on: it read zero through every run whose cadence was
    /// broken.
    pub frames_late: i64,
}

/// An open log, as far as anything writing to one can tell.
///
/// On Windows it is the `HANDLE` reinterpreted; under a simulated Windows it is an index into the
/// lines a test can read back. Opaque either way — orb only ever hands one back to the seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogFile(pub usize);

/// What a seam function does on a host with no Windows and no simulated Windows installed.
///
/// Reached only by a test that forgot to install one: there is no third answer to give, and
/// answering zero would let the test pass on a read that never happened.
#[cfg(not(windows))]
#[cold]
#[track_caller]
fn no_windows(what: &str) -> ! {
    panic!(
        "{what}: no simulated Windows is installed, and this host has no real one. \
         A test reaching here has not called orb_api::install."
    )
}
