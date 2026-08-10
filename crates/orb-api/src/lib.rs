//! The seam between orb and the host it runs on.
//!
//! What orb gets from the host goes through one of the modules here. Each of them is a
//! facade of free functions with two answers behind it: the real one, under
//! `#[cfg(windows)]`, and whatever [`Win`] implementation a test has installed. Nothing in
//! any signature is a `windows-sys` type, which is what lets the crates over this seam —
//! `orb-core`, `orb-sim` — be built and tested on a host that is not Windows.
//!
//! **The host and not Windows**, which is a distinction with one member: [`Win::spin_once`] is the
//! `pause` instruction and no call at all — see
//! [docs/adr/0007](../../../docs/adr/0007-the-spins-pause-is-behind-the-seam.md). So what decides whether
//! something belongs here is not whether there is a Win32 function on the other side of it.
//!
//! **What it is instead is simply: does orb reach outside itself for it.** Every such reach goes through a
//! module here, and the two that used to be argued about are settled — the `pause`, which is no call at all,
//! is in; and the files orb reads and writes of its own are in, through [`fs`].
//!
//! That rule replaced a narrower one, and the narrower one is worth reading because of what it let out:
//! *whether it is the host doing something to orb that no test could otherwise decide.* Files failed that
//! test, since a test can put a file in a directory it owns — and doing so is how `resume` and `orb.yaml`
//! were driven for as long as they were `std::fs`. What that cost is the whole of why the rule changed: an
//! e2e test with a real directory behind it has one to empty before it runs and one left behind when it
//! fails, and the question it cannot ask at all is *what happens when the write fails*, there being no way
//! to make `std::fs` refuse except by deforming the machine into a shape where it does. See
//! [docs/adr/0012](../../../docs/adr/0012-orb-reads-and-writes-its-own-files-through-the-seam.md).
//!
//! The call sites keep their shape. `mem::read(address)` is what it was before the seam went
//! in, and the alternative — making every caller carry a `&dyn Win` — would have rewritten
//! two thousand lines of structure walking to say nothing new. The install point is a
//! thread-local instead, because the test harness runs tests side by side in one process and
//! a simulated Windows in a static would be two tests writing each other's game.

use std::ops::Range;
use std::path::{Path, PathBuf};

pub mod clock;
pub mod codepage;
pub mod d3d8;
pub mod display;
pub mod dsound;
pub mod fs;
pub mod joystick;
pub mod keyboard;
pub mod logfile;
pub mod mem;
pub mod module;
pub mod mouse;
pub mod process;
pub mod text;
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
/// The same four fields at the same offsets as Windows' `RECT`, and exclusive at the right and bottom
/// as that one is, so the arithmetic either side of the seam is the arithmetic that was already
/// written. Plain data, so the types the seam traffics in build on any host.
///
/// **`#[repr(C)]` because that sentence is a claim about a layout**, and one something relies on: the
/// `Present` slot's signature takes two `*const RECT`, and the replacement `orb::window` puts in the slot
/// has to have that signature exactly. Four `i32`s give a compiler no reason to reorder them and it does
/// not, which is why nothing had gone wrong before this attribute — and is also why nothing would have
/// said if it did. `orb::window` holds the four offsets against Windows' own at compile time, being the
/// last place that names `RECT` at all; a size assert would pass whatever order the fields came out in,
/// which is the one thing being asked about.
#[repr(C)]
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

/// Where a stack of lines goes in the black beside the game: the rows to repaint, and where in them
/// the text starts.
///
/// **Here rather than in `orb-core`**, which is the one thing about this that cannot be got wrong
/// quietly: it is [`Win::write_lines`]'s argument, that call is answered below the seam, and the far
/// side may not name `orb-core`. So it joins [`Viewport`] and [`Locked`] at this crate's root, laid out
/// above the seam and read below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bar {
    /// Everything to clear and put on the screen, in the window's client coordinates.
    pub area: Rect,
    /// The x the text is aligned from, in client coordinates.
    pub x: i32,
    /// The bottom of the lowest line.
    pub bottom: i32,
    /// One line's height, which is also the em the font is made at.
    pub height: i32,
    /// [`Bar::LEFT`] or [`Bar::RIGHT`].
    pub align: u32,
}

impl Bar {
    /// `TA_LEFT` and `TA_RIGHT`, written out here and held against Windows' own numbers by a `const`
    /// assert below the seam — the same way `orb_core::window`'s two window styles are.
    ///
    /// Above the seam because which way a stack of lines is aligned is part of deciding *which* bar it
    /// goes in: down the side the text runs away from the game and under it the text runs back towards
    /// the game's own edge, and that is one decision with the alignment as half of its answer.
    pub const LEFT: u32 = 0;
    pub const RIGHT: u32 = 2;
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

    /// Swaps the word at `address` for `value`, unprotecting the page for as long as the write takes,
    /// and answers what was there. `None` where the page could not be made writable at all.
    ///
    /// Here beside [`commit`](Win::commit) rather than left to the caller because it is a page operation
    /// and not a decision: what the caller wants is a vtable slot swapped, and the page that slot is in
    /// being read-only is this side's business. The read and the write on their own are
    /// [`read_bytes`](Win::read_bytes) and [`write_bytes`](Win::write_bytes) already.
    fn replace_word(&self, address: usize, value: usize) -> Option<usize>;

    /// Whether `address` holds a pointer into a mapped image, which is what a live COM
    /// object's vtable pointer looks like.
    fn vtable_in_image(&self, address: usize) -> bool;

    /// The committed regions the game owns, as `(base, len)` — what a snapshot walks. The
    /// data range comes first, as a real game's does; a region of the game's code is left out,
    /// since a chapter is a copy of the game's state and not of itself.
    ///
    /// No two entries may cover the same pages. That is this side's rule to keep because it is a rule
    /// about *these* pages: a real walk finds a heap region and a reservation naming the same ones and
    /// merges them, and laid-out memory answers with the objects an e2e test put there, two of which
    /// merged because they abut is one range nothing in that space can read.
    fn game_regions(&self, data: &Range<usize>) -> Vec<(usize, usize)>;

    /// Remembers a heap the game has just allocated from, so [`game_regions`](Win::game_regions) walks
    /// it.
    ///
    /// The same shape [`register_thread`](Win::register_thread) has and for the same reason: the
    /// noticing is orb's — import hooks on `HeapCreate`, `HeapAlloc`, `HeapReAlloc` and `HeapFree` —
    /// and what is done with what was noticed is the host's, `HeapLock` and `HeapWalk` belonging on this
    /// side. Every allocation the game makes says it again, so a host must take a heap it already has
    /// once.
    fn note_heap(&self, heap: usize);

    /// And a range the game reserved straight from the OS — an import hook on `VirtualAlloc`. A commit
    /// inside a range already noted is not a new one.
    fn note_reservation(&self, base: usize, len: usize);

    /// And that one has been released, which is `VirtualFree` with `MEM_RELEASE`. A range freed and not
    /// forgotten is a range a snapshot would read after the OS took it back.
    fn forget_reservation(&self, base: usize);

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

    /// `Sleep` — a coarse wait, in whole milliseconds, on a thread nothing is waiting for.
    ///
    /// Apart from [`wait`](Win::wait), which is aimed at a frame's own deadline on a high-resolution
    /// timer and whose whole reason for existing is the input lag left to win. This one is the gap
    /// between two reads of a device, and a millisecond is as fine as it needs to be.
    ///
    /// **A simulated host must not advance the clock here.** The caller is the sampling thread, and a
    /// background thread moving the frame loop's own counter would break every pacing e2e test — each
    /// asserts about that counter to the microsecond. So a simulated host waits and writes down what it
    /// was asked for, which is what makes the cadence something an e2e test can read back.
    fn sleep(&self, ms: u32);

    // --- the one failure that ends a launch -------------------------------------
    //
    // Behind the seam because there is behaviour here no e2e test could otherwise reach: one that
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

    /// `SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL)` — the calling thread put
    /// below the game's.
    ///
    /// No thread named and no priority chosen, because there is one caller and it is the sampling
    /// thread on itself: a sample a millisecond old costs nobody anything and a late frame costs the
    /// thing orb was written for. Not a parameter of [`thread::spawn`](crate::thread::spawn) either —
    /// the priority is set from inside the body because that is where the thread is, and
    /// `std::thread::Builder` has nowhere to say it.
    fn below_normal(&self);

    // --- the log --------------------------------------------------------------

    /// Opens the log for appending, starting it over rather than appending where it has already
    /// grown past `max_bytes`. `None` where it could not be opened at all, which orb carries on
    /// without.
    fn open_log(&self, path: &Path, max_bytes: u64) -> Option<LogFile>;
    /// Appends bytes to a log opened by [`open_log`](Win::open_log). Bytes rather than a `&str`
    /// because the line endings orb writes are the file's, not the platform's.
    fn write_log(&self, file: LogFile, bytes: &[u8]);
    fn close_log(&self, file: LogFile);

    // --- the files orb reads and writes of its own ----------------------------
    //
    // Not the game's own: the score file is the game's `CreateFileA` under orb's hook, and the log is
    // the three above. See [`crate::fs`], and `docs/adr/0012` for why these are here at all.
    //
    // `std::io::Result`, which is neither a `windows-sys` type nor a new one: what every call site
    // does with an error is put it in a line of the log.

    fn read_file(&self, path: &Path) -> std::io::Result<Vec<u8>>;
    fn read_file_to_string(&self, path: &Path) -> std::io::Result<String>;
    fn write_file(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn remove_file(&self, path: &Path) -> std::io::Result<()>;
    /// The files directly inside a directory, as their whole paths. See [`crate::fs::files_in`] for why
    /// files rather than entries.
    fn files_in(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;

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
    /// monitor: a scaled desktop reports the scaled size until this is called and the panel's own
    /// afterwards, so the same monitor has two sizes and which one a layout used is the whole of
    /// whether it was right. `orb_sim::Windows::monitor_reads` is a host that answers both.
    fn set_process_dpi_aware(&self) -> bool;

    /// `MonitorFromPoint(0,0)` and `GetMonitorInfoW` — the primary monitor, in the desktop's own
    /// coordinates. `None` where the host will not say, which leaves the window as the game made it.
    fn primary_monitor(&self) -> Option<Rect>;

    /// `AdjustWindowRect` — the whole window a client area of that size needs, frame included.
    ///
    /// Behind the seam because how thick a frame is belongs to the host and to its theme: the size in
    /// `orb.yaml` is the size of what is *inside* the window, so the frame is what stands between the
    /// number asked for and the window to ask for, and it is not a number orb can know. The frame an
    /// e2e test charges is `orb_sim::Windows`'s.
    ///
    /// `None` where the host refused, which leaves the rectangle as the client area — a window a
    /// frame too small rather than no window.
    fn adjust_window_rect(&self, area: Rect, style: u32, menu: bool) -> Option<Rect>;

    /// `GetClientRect` — what the game draws into, which is not the size that was asked for whenever
    /// anything between here and the screen has an opinion. `None` for a window that is not there.
    fn client_rect(&self, window: Hwnd) -> Option<Rect>;

    // --- the status line, in the black beside the game ---------------------------
    //
    // Two, and **coarser than the drawing seam's eighteen on purpose**. That one is a mirror of the
    // device's slots because the failure a state-block bracket exists to prevent is the *game's own
    // scene* drawing wrong, which no test below the seam could see. Here there is no such bracket: the
    // black beside the game is orb's alone, Direct3D never touches it, and the one failure below this
    // line is a flicker, which the single blit prevents and which no test could have seen either way.
    // What an e2e test can have an opinion about — which bar the lines went in, which height they got
    // and where the block landed — is all above. See
    // [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

    /// The widest of `lines` and one line's height, at an em height of `em` pixels —
    /// `GetTextExtentPoint32W` through a font made at that height, over each line.
    ///
    /// Both, because that is what the call answers. The widest is what a size is chosen from: what is
    /// written beside the game runs from three characters to twenty, and a line clipped at the bar's
    /// edge cannot be read at all.
    fn measure_lines(&self, lines: &[String], em: i32) -> (i32, i32);

    /// `lines` written into `bar` of `window`'s client area — the clear, the text and the blit — and
    /// whether they reached the screen.
    ///
    /// **One call because the three have to be one operation.** Done on the window in turn, a refresh
    /// landing between the clear and the text shows a bar with nothing in it, and at 120Hz that is a
    /// flicker somebody sees.
    ///
    /// The font is made at [`Bar::height`], that being the em a line's height was measured at.
    fn write_lines(&self, window: Hwnd, bar: Bar, lines: &[String]) -> bool;

    // --- the keyboard ----------------------------------------------------------

    /// `GetKeyboardState` — every key at once, `0x80` set on the ones that are down. `None` where
    /// the call failed, which orb reads as nothing being down.
    ///
    /// The whole array rather than a key at a time, because that is what the call answers: asking
    /// per key would be several reads of a state that may have moved between them, and orb's menus
    /// read six keys a frame.
    fn keyboard_state(&self) -> Option<[u8; 256]>;

    // --- the mouse pointer ------------------------------------------------------

    /// `GetCursorPos` — where the pointer is, in the desktop's own coordinates. `None` where the call
    /// failed, which orb reads as a pointer it cannot follow.
    fn mouse_position(&self) -> Option<(i32, i32)>;

    /// `ShowCursor` — the display counter moved one step either way, and what it reads afterwards.
    ///
    /// The counter rather than a flag because that is what Windows keeps: the pointer is drawn while it
    /// is not negative, and every caller in the process moves the same number. Which is why the answer
    /// comes back — see [`mouse::show`].
    fn show_mouse(&self, showing: bool) -> i32;

    // --- the joystick ----------------------------------------------------------

    /// `joyGetPosEx` — where the joystick is, and what the call answered.
    ///
    /// The result as well as the position, because which of the failures it is is what orb reports
    /// and how often it asks again: no such joystick is one to look for once a second, and one
    /// answering is one to sample four times a frame.
    fn joystick_position(&self, device: u32, flags: u32) -> (u32, JoyInfo);

    /// `joyGetDevCapsA` — what the device is and what its axes' bounds are. `None` where the call
    /// failed, which orb reads as a device it cannot describe.
    fn joystick_caps(&self, device: u32) -> Option<JoyCaps>;

    // --- the machine's code page -------------------------------------------------

    /// `MultiByteToWideChar` with `CP_ACP` — bytes a Win32 `-A` call answered, in whatever code page
    /// the machine is set to, as a string the log can hold.
    ///
    /// A simulated host reads them as UTF-8, lossily, and that is not a gap: what an e2e test asks of
    /// the line this ends up in is which device it names, and the name it names is one the e2e test
    /// declared.
    fn codepage_text(&self, bytes: &[u8]) -> String;

    // --- loaded modules --------------------------------------------------------

    /// The address of a function exported by an already-loaded module — `mmioSeek` out of the
    /// winmm the game has loaded — or `None` where either is not there.
    ///
    /// Named rather than handed a handle because a handle is the thing that does not survive the
    /// seam: what the caller knows is `"winmm.dll"` and `"mmioSeek"`, and the lookup of both
    /// belongs on the far side.
    fn proc_address(&self, module: &str, name: &str) -> Option<usize>;

    // --- glyphs -----------------------------------------------------------------
    //
    // Behind the seam because rasterisation is the third object orb reaches that is somebody else's
    // code — the GDI, after Direct3D and DirectSound — and the one a grep for `windows-sys` would
    // find least, `Font::load` having been a struct of orb's own. **Coarser than the other two on
    // purpose**: a mirror of the calls would put a device context across the seam, and the whole of
    // what orb asks a rasteriser is *bake this string at this height*. See
    // [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).

    /// `AddFontResourceExW` for the file, then `CreateFontIndirectW` for a face of it at `height`
    /// pixels of em. `None` where the file is not a font that can be added, or no face could be made
    /// of it.
    ///
    /// Process-private, which is the whole reason the add is here rather than left to the caller: a
    /// font installed system-wide would outlive the run. A host that has no such file must refuse,
    /// because a launch beside an exe with no `font.ttf` next to it is a launch with no overlay and
    /// orb says so in the log.
    fn load_face(&self, path: &Path, height: i32) -> Option<Face>;

    /// What the host actually selected — `GetTextFaceW`. Asked so that a substituted face shows up in
    /// the log rather than silently: `Font::load` survives one, and the glyphs are then not the
    /// game's.
    fn face_name(&self, face: Face) -> Option<String>;

    /// Bakes `text` through `face` into a coverage mask, and `None` for a string that measures to
    /// nothing — an empty one, or one the host would not measure at all.
    ///
    /// The mask is what the quad round the string is sized from, so its own width and height come
    /// back with it.
    fn bake(&self, face: Face, text: &str) -> Option<Mask>;

    /// The face deleted and the add taken back out. The host counts the adds, and orb makes one per
    /// size every time an overlay is built.
    fn drop_face(&self, face: Face);

    // --- the device the game shows its frames through ---------------------------
    //
    // Eighteen slots, which is every one `crates/orb-api/src/d3d8.rs` types — fifteen of
    // `IDirect3DDevice8` and three of `IDirect3DTexture8`. **A mirror of them and not an abstraction
    // over them**, which is the whole of the design and the one way it can be got wrong: what must not
    // cross is a decision. See that module, and
    // [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
    //
    // Behind the seam because Direct3D is somebody else's code reached through a pointer the game handed
    // over rather than through a crate a grep would find — so a rule kept by looking for `windows-sys`
    // passed a file that was calling Windows fifteen times.

    // Eight of them, which is `CreateTexture`'s own list and the reason this is a mirror: a seam that
    // decided the levels, the usage, the format or the pool for its caller would be one deciding what a
    // texture is, and what orb makes them for — a managed A8R8G8B8 with one level — is a decision above
    // this line with the drawing's reasons beside it.
    #[allow(clippy::too_many_arguments)]
    fn create_texture(
        &self,
        device: Device,
        width: u32,
        height: u32,
        levels: u32,
        usage: u32,
        format: u32,
        pool: u32,
    ) -> (Hresult, Option<Texture>);
    fn create_state_block(&self, device: Device, kind: u32) -> (Hresult, u32);
    fn capture_state_block(&self, device: Device, token: u32) -> Hresult;
    fn apply_state_block(&self, device: Device, token: u32);
    fn delete_state_block(&self, device: Device, token: u32);
    fn set_render_state(&self, device: Device, state: u32, value: u32);
    fn set_texture_stage_state(&self, device: Device, stage: u32, kind: u32, value: u32);
    fn set_texture(&self, device: Device, stage: u32, texture: Option<Texture>);
    fn set_vertex_shader(&self, device: Device, shader: u32);
    fn set_viewport(&self, device: Device, viewport: Viewport);
    fn get_viewport(&self, device: Device) -> Viewport;
    fn draw_primitive_up(
        &self,
        device: Device,
        kind: u32,
        count: u32,
        vertices: &[u8],
        stride: u32,
    );
    fn begin_scene(&self, device: Device);
    fn end_scene(&self, device: Device);
    fn clear(&self, device: Device, flags: u32, color: u32, z: f32, stencil: u32);

    fn lock_rect(&self, texture: Texture, level: u32, flags: u32) -> Option<Locked>;
    fn unlock_rect(&self, texture: Texture, level: u32);
    fn release_texture(&self, texture: Texture);

    // --- the buffer the game's music is played out of ----------------------------
    //
    // Eight slots, which is every one `crates/orb-api/src/dsound.rs` types. Behind the seam for the same
    // reason the device is: DirectSound is somebody else's code reached through a pointer the game handed
    // over, and a grep for `windows-sys` finds none of it.
    //
    // Named apart from the device's — `buffer_position` and not `get_current_position` — because a trait
    // has one namespace and two COM interfaces have a `GetStatus` each. The facades keep the slots' own
    // names, which is where a reader is looking for them.

    fn buffer_position(&self, buffer: SoundBuffer) -> (Hresult, u32, u32);
    fn buffer_status(&self, buffer: SoundBuffer) -> (Hresult, u32);
    fn lock_buffer(
        &self,
        buffer: SoundBuffer,
        offset: u32,
        bytes: u32,
        flags: u32,
    ) -> (Hresult, LockedBuffer);
    fn unlock_buffer(&self, buffer: SoundBuffer, locked: LockedBuffer);
    fn play_buffer(&self, buffer: SoundBuffer, reserved: u32, priority: u32, flags: u32);
    fn stop_buffer(&self, buffer: SoundBuffer);
    fn set_buffer_position(&self, buffer: SoundBuffer, position: u32);
    fn restore_buffer(&self, buffer: SoundBuffer);
}

/// A font face at one size, as far as anything baking a string through one can tell.
///
/// On Windows it is what `real::text` boxed to hold the `HFONT` and the path the add was made with;
/// under a simulated Windows it is which of the faces an e2e test declared. Opaque either way — orb
/// only ever hands one back to the seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face(pub usize);

/// A baked string: `0x00ffffff` with the coverage in the alpha channel, row by row from the top.
///
/// The colour is applied by the vertex colour at draw time, so a label costs one bake however many
/// colours it is drawn in — which is why what crosses here is coverage and not pixels.
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

/// The game's `IDirect3DDevice8`, as far as anything drawing through one can tell.
///
/// The address the game keeps it at, which is what orb reads out of the game's memory and hands back —
/// so the only thing it has to be is the same number coming out as went in, the way [`Hwnd`] is. What
/// is at that address is a pointer to a vtable, and the one piece of code that follows it is
/// [`real::d3d8`](crate::real::d3d8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Device(pub usize);

impl Device {
    pub const NULL: Self = Self(0);

    pub fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// An `IDirect3DTexture8` the device made, the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Texture(pub usize);

/// What Direct3D calls an `HRESULT`: negative for a failure, and the number itself is what orb writes
/// into the log.
pub type Hresult = i32;

/// A viewport, field for field and in the same order as `D3DVIEWPORT8`, so the arithmetic either side of
/// the seam is the arithmetic that was already written.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub min_z: f32,
    pub max_z: f32,
}

/// The buffer the game's music is played out of, as far as anything asking about one can tell.
///
/// The address the game's streaming sound keeps it at, the way [`Device`] is. The one word of it orb
/// reads rather than calls is the vtable pointer at its head — through [`mem`], because whether it lands
/// in a mapped image is how a live buffer is told from the stale one left in a freed block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundBuffer(pub usize);

/// What a lock of a sound buffer hands back: two runs, because a lock that reaches the end of a looping
/// buffer wraps and comes back in two halves.
///
/// Addresses rather than slices, for the reason [`Locked`] carries: what is behind them is the host's
/// memory, and which of the two runs a caller writes is the caller's decision. The same value goes back
/// to [`dsound::unlock`], which is what the slot takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LockedBuffer {
    pub first: usize,
    pub first_bytes: u32,
    pub second: usize,
    pub second_bytes: u32,
}

/// A texture's rows, as a lock hands them over: how far apart they are, and where the first of them is.
///
/// **An address and not a slice, and not a closure taking one.** What is behind it is memory the host
/// handed back — Direct3D's own on a real device, and the simulator's own storage under one — so the
/// caller builds the slice it wants over the rows it means. A closure would put the walk of those rows
/// on this side of the seam, and that walk is what the drawing decides: which rows the mask fills and
/// what the padding a power-of-two texture added is left as. `mem` would be the wrong door for it too —
/// that one is the *game's* address space, and a locked texture is not in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Locked {
    pub pitch: i32,
    pub bits: usize,
}

/// Where a joystick is, as winmm reports it: `JOYINFOEX`, field for field and in the same order.
///
/// **The same layout and not a subset**, which is the one type across this seam that has to be: orb
/// stands in front of the game's own `joyGetPosEx` and hands the game whatever the last sample held,
/// so what crosses here is the struct the game asked to be filled. Plain data all the same — there is
/// no `windows-sys` type in any signature — and the real side asserts the two are the same size.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JoyInfo {
    pub size: u32,
    pub flags: u32,
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub r: u32,
    pub u: u32,
    pub v: u32,
    pub buttons: u32,
    pub button_number: u32,
    pub pov: u32,
    pub reserved1: u32,
    pub reserved2: u32,
}

/// And what a joystick *is* — `JOYCAPSA`, the same way and for a harder reason.
///
/// 紅魔郷 keeps one of these at 0x69d760 and measures every axis against it, and it only ever reads
/// one where a joystick answered at startup — so a pad that turns up later is measured against zeros.
/// orb writes the answering device's caps in there, byte for byte, which is why this has to be the
/// same 0x194 bytes rather than the four fields orb reads for itself.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoyCaps {
    pub manufacturer: u16,
    pub product: u16,
    /// The device's name in the machine's own code page, which is what winmm answers with.
    pub name: [u8; 32],
    pub x_min: u32,
    pub x_max: u32,
    pub y_min: u32,
    pub y_max: u32,
    pub z_min: u32,
    pub z_max: u32,
    pub buttons: u32,
    pub period_min: u32,
    pub period_max: u32,
    pub r_min: u32,
    pub r_max: u32,
    pub u_min: u32,
    pub u_max: u32,
    pub v_min: u32,
    pub v_max: u32,
    pub caps: u32,
    pub max_axes: u32,
    pub axes: u32,
    pub max_buttons: u32,
    pub registry_key: [u8; 32],
    pub oem_driver: [u8; 260],
}

impl Default for JoyCaps {
    fn default() -> Self {
        Self {
            manufacturer: 0,
            product: 0,
            name: [0; 32],
            x_min: 0,
            x_max: 0,
            y_min: 0,
            y_max: 0,
            z_min: 0,
            z_max: 0,
            buttons: 0,
            period_min: 0,
            period_max: 0,
            r_min: 0,
            r_max: 0,
            u_min: 0,
            u_max: 0,
            v_min: 0,
            v_max: 0,
            caps: 0,
            max_axes: 0,
            axes: 0,
            max_buttons: 0,
            registry_key: [0; 32],
            oem_driver: [0; 260],
        }
    }
}

/// What the two calls above answer where they worked, and the two failures orb tells apart: no such
/// joystick, and one that is not plugged in. `JOYERR_NOERROR`, `JOYERR_PARMS` and `JOYERR_UNPLUGGED`,
/// which are winmm's own numbers.
pub mod joyerr {
    pub const NOERROR: u32 = 0;
    pub const PARMS: u32 = 165;
    pub const UNPLUGGED: u32 = 167;
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
