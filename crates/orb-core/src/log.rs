//! Append-only log beside the game exe, which is where the launcher and `orb.yaml` are too.
//!
//! Every run appends rather than starting the file over: a run worth looking at
//! is often over by the time anyone looks, and the next launch would otherwise
//! have thrown it away. Only a log that has grown past `MAX_BYTES` is replaced.
//!
//! The writing itself is `orb_api::logfile`'s, which on Windows is a bare `WriteFile` rather than
//! anything of `std::fs`: the first lines are emitted from `DllMain` while the loader lock is
//! held, so the write must not be one that might take a lock of its own on the way.
//!
//! **A line the frame's own thread says is not written where it is said.** A `WriteFile` takes what it
//! takes, and one that lands in the millisecond a frame has to reach `DwmFlush` in costs that frame a
//! refresh — a run stuttering because it was being watched. So [`defer`] holds those until [`drain`],
//! which the frame loop calls where what is left of the turn is slack. [`line`] is the other way in,
//! for what a log that stops mid-run has to have: startup, the faults, and the panic hook and the
//! crash filter, none of which has a frame to wait for.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicIsize, AtomicU32, AtomicUsize, Ordering};

use orb_api::LogFile;
use orb_api::{clock, logfile, thread};
use orb_config::LogLevel;

/// Zero means "no log file", which is also how the process starts out — and which is why the
/// simulated log's own handle is not zero either.
static FILE: AtomicUsize = AtomicUsize::new(0);

/// Where the log is started over rather than appended to.
const MAX_BYTES: u64 = 8 << 20;

/// How much is being written. Set once the config has been read; until then
/// everything is written, because a fault during startup is the one worth having.
static LEVEL: AtomicIsize = AtomicIsize::new(VERBOSE);

const QUIET: isize = 0;
pub const NORMAL: isize = 1;
pub const VERBOSE: isize = 2;

/// Whether the frame loop writes what the pacing is doing. Off until the config says,
/// unlike the level, because there is no startup fault it would be the evidence for.
static PACING: AtomicBool = AtomicBool::new(false);

/// What writing the log has cost since it was last asked, in performance-counter ticks,
/// and how many writes that was. Index 0 is the thread the frame runs on and 1 is every
/// other.
///
/// Kept because the log is the instrument: a `WriteFile` takes what it takes, and one
/// that lands in the moments before a frame is handed over costs that frame a refresh.
/// A run whose stutters line up with what was written cannot say so unless what the
/// writing cost is written down too.
///
/// The two threads are kept apart because they are different findings. The appends
/// serialise on one handle, so either can hold a frame up, but only what the frame's own
/// thread writes is something the frame loop chose to do where it did.
static SPENT: [AtomicI64; 2] = [AtomicI64::new(0), AtomicI64::new(0)];
static WRITES: [AtomicI64; 2] = [AtomicI64::new(0), AtomicI64::new(0)];
static FREQUENCY: AtomicI64 = AtomicI64::new(0);

/// The thread the frame loop runs on, claimed by the first `drain`. Zero until then,
/// which no thread's id is, so everything before it is written where it is asked for.
///
/// What it decides is whether a line waits: one written from the frame's own thread can be held for
/// the slack after the flush, and one from anywhere else has no frame to wait for.
static FRAME_THREAD: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// Lines held back until the frame loop reaches a moment where writing one costs nothing.
    ///
    /// The moments between handing a frame over and the blank it is shown at are the ones a
    /// write must stay out of: the next frame has about a millisecond to reach `DwmFlush`,
    /// and one that arrives after the blank has gone waits out another refresh and is shown
    /// late. On the far side of that flush there are fourteen milliseconds of slack doing
    /// nothing, so that is where these go.
    ///
    /// **What is held is everything a level or a switch can turn off** — `summary!`, `detail!`
    /// and `pacing!` — which is what a run says while it is being watched, and what is read
    /// afterwards out of a file that got there. `log!` is written where it stands: it is what
    /// every level keeps, and a run that ends in a crash or a hang has to have its last lines
    /// in the file rather than in here.
    ///
    /// Per thread rather than in a `MainThread`, which is what it was. In the game there is one
    /// frame thread and the two are the same thing; in a test binary there are as many as there
    /// are e2e tests running at once, each of them the frame's thread for its own run, and a
    /// single buffer between them is two `&mut` to one `Vec`. The cost is a thread-local lookup
    /// twice a frame.
    static HELD: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}
/// Where holding lines back stops being worth it, being far more than a frame makes.
const HELD_MAX: usize = 64;

pub fn set_level(level: LogLevel) {
    LEVEL.store(
        match level {
            LogLevel::Quiet => QUIET,
            LogLevel::Normal => NORMAL,
            LogLevel::Verbose => VERBOSE,
        },
        Ordering::Release,
    );
}

pub fn set_pacing(wanted: bool) {
    PACING.store(wanted, Ordering::Release);
}

pub fn pacing_wanted() -> bool {
    PACING.load(Ordering::Acquire)
}

pub fn wanted(level: isize) -> bool {
    LEVEL.load(Ordering::Acquire) >= level
}

/// Startup, and anything that went wrong. Written at every level, because a log with
/// none of this in it cannot say whether the run even happened.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => { $crate::log::line(&format!($($arg)*)) };
}

/// How a run is going: a line a second, or a line per scene. Off at `quiet`.
///
/// Held for the frame loop's slack, this being written from inside the frame's own work.
#[macro_export]
macro_rules! summary {
    ($($arg:tt)*) => {
        if $crate::log::wanted($crate::log::NORMAL) {
            $crate::log::defer(&format!($($arg)*))
        }
    };
}

/// Per-frame detail, for finding out why one frame in a hundred was late. Only at
/// `verbose`, because it is written faster than it can be read.
///
/// Held for the frame loop's slack, and the one that most has to be: a line every frame written
/// where it is worked out is a run whose frames are late because they are being watched.
#[macro_export]
macro_rules! detail {
    ($($arg:tt)*) => {
        if $crate::log::wanted($crate::log::VERBOSE) {
            $crate::log::defer(&format!($($arg)*))
        }
    };
}

/// What the frame loop says about the pacing, which every level writes and `--no-pacing` is
/// what stops.
///
/// Not a tier of the level, because it answers a different question from how the run is going:
/// what a run spends its frames on is worth writing whether or not that is. `quiet` with it on
/// is a file holding the startup lines and these and nothing else, which is what a sweep is read
/// off.
///
/// Held for the frame loop's slack like the two above it.
#[macro_export]
macro_rules! pacing {
    ($($arg:tt)*) => {
        if $crate::log::pacing_wanted() {
            $crate::log::defer(&format!($($arg)*))
        }
    };
}

pub fn open() {
    if held() {
        line("---- new run ----");
    }
}

/// One line for a launch that was refused, with the file opened and closed around it.
///
/// **The launcher's, and the only line here not written from inside a game.** Every other one comes
/// out of a run, where the log is open for as long as the run lasts; a refusal has no run to hold it
/// open.
///
/// It goes here because the file is the only one of the three places a refusal is said that is still
/// there afterwards: the printed line needs a console, which a launch from a shortcut has none of, and
/// the dialog is gone the moment it is answered. A launch that never started and left nothing behind
/// is one nobody can be asked what happened — measured, as `--pacing` for `--no-pacing` making orb
/// exit with an empty log and nothing on the screen.
pub fn refused(message: &str) {
    if held() {
        line(&format!("launch refused: {message}"));
        close();
    }
}

/// The log opened and its handle held for the writers above, or `false` where the host would not open
/// it at all.
fn held() -> bool {
    let Some(exe) = orb_api::module::host_exe() else {
        return false;
    };
    let Some(file) = logfile::open(&exe.with_file_name("orb.log"), MAX_BYTES) else {
        return false;
    };
    FILE.store(file.0, Ordering::Release);
    true
}

pub fn close() {
    // What is held goes in first: this is the last moment there is, and the crash filter reaches it
    // on the thread that faulted — which for a fault inside a frame is the one whose held lines say
    // what the run was doing when it happened.
    flush();
    let file = FILE.swap(0, Ordering::AcqRel);
    if file != 0 {
        logfile::close(LogFile(file));
    }
}

/// Writes a line where the caller stands, whatever the frame loop is doing.
pub fn line(message: &str) {
    let file = FILE.load(Ordering::Acquire);
    if file == 0 {
        return;
    }
    // From here rather than from around the write alone: the frame pays for the formatting as
    // much as for the write, and what is wanted is what having written the line cost it.
    let started = clock::counter();
    let line = format!("[{:>8}ms] {message}\r\n", clock::ticks());
    logfile::write(LogFile(file), line.as_bytes());
    let side = usize::from(thread::current_id() != FRAME_THREAD.load(Ordering::Relaxed));
    SPENT[side].fetch_add(clock::counter() - started, Ordering::Relaxed);
    WRITES[side].fetch_add(1, Ordering::Relaxed);
}

/// What writing the log has cost, microseconds and writes, from the frame's own thread
/// and from every other one.
pub struct Cost {
    pub us: i64,
    pub writes: i64,
    pub other_us: i64,
    pub other_writes: i64,
}

/// What the log has cost since this was last asked. Resets, so whoever asks owns the
/// interval — which is the frame loop, once a frame.
pub fn spent() -> Cost {
    let frequency = frequency();
    let micros = |ticks: i64| {
        if frequency == 0 {
            0
        } else {
            ticks * 1_000_000 / frequency
        }
    };
    Cost {
        us: micros(SPENT[0].swap(0, Ordering::Relaxed)),
        writes: WRITES[0].swap(0, Ordering::Relaxed),
        other_us: micros(SPENT[1].swap(0, Ordering::Relaxed)),
        other_writes: WRITES[1].swap(0, Ordering::Relaxed),
    }
}

/// Holds a line back until the frame loop has slack to write it in.
///
/// Safe from any thread: only the frame's own thread has anything held for it, and every
/// other writes where it stands. A line from elsewhere would be no better off waiting
/// for a frame it is not part of.
pub fn defer(message: &str) {
    if thread::current_id() != FRAME_THREAD.load(Ordering::Relaxed) {
        return line(message);
    }
    // Taken out of the buffer before anything is written, because `line` is reached below and a
    // borrow still held across it would be a second borrow of the same cell.
    let overflowed = HELD.with(|held| {
        let mut held = held.borrow_mut();
        if held.len() < HELD_MAX {
            held.push(message.to_string());
            return None;
        }
        Some(std::mem::take(&mut *held))
    });
    let Some(overflowed) = overflowed else { return };
    // A drain happens twice per frame's turn, so this many held means the frame loop has stopped
    // running and no better moment is coming. Written where they stand rather than lost, which for a
    // run that ended in a fault is the difference between having the last frames and not.
    line("log: the frame loop has stopped draining; writing what is held where it stands");
    for held in overflowed {
        line(&held);
    }
    line(message);
}

/// Writes what has been held back, and claims the calling thread as the frame's.
///
/// To be called from the frame loop at a point where a write costs nothing: after the
/// wait for the blank, not before it.
pub fn drain() {
    let thread = thread::current_id();
    if FRAME_THREAD.swap(thread, Ordering::Relaxed) != thread {
        // The first drain, or the frame moved threads. Nothing is held for this one yet.
        return;
    }
    flush();
}

/// Writes what the calling thread is holding, wherever it stands.
fn flush() {
    for message in HELD.with(|held| std::mem::take(&mut *held.borrow_mut())) {
        line(&message);
    }
}

fn frequency() -> i64 {
    let cached = FREQUENCY.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let frequency = clock::frequency();
    FREQUENCY.store(frequency, Ordering::Relaxed);
    frequency
}
