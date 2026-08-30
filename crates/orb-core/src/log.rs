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
//! **A line the frame's own thread says is written by another thread.** A `WriteFile` takes what a disk,
//! an antivirus reading every write or a folder being synced decides, and a frame that waits for one waits
//! that long: measured on th07, one write took **751621µs** with that frame's own work under a
//! millisecond, and two more took 230ms and 178ms — three of the stutters somebody watching the run
//! noticed. So the frame's thread formats the line and hands it over, and [`writing`] puts it on the disk,
//! the way [`crate::writer`] already does for the files orb keeps of its own.
//!
//! **Holding the line for the frame's own slack was the answer before this**, and it is not one: the slack
//! is what is left of one frame's turn, so a write of three quarters of a second put there misses the next
//! frame's blank instead of this one's — the same stutter, one frame later. `writer`'s own module note
//! makes that argument about a chapter's file; this is the same argument about a line.
//!
//! **The writing waits for a thread there is**, which is why [`off_the_frame`] starts it rather than
//! [`line`]: the first lines are said from `DllMain` with the loader lock held, and a thread made there is
//! a thread whose own `DLL_THREAD_ATTACH` waits for that lock. So until the frame loop has been round
//! once, every line is written where it stands — which is also what a launch whose thread cannot be made
//! falls back to, a log written in the frame being better than no log.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicIsize, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};

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

/// The thread the frame loop runs on, claimed by [`off_the_frame`]. Zero until then, which no thread's id
/// is, so every write before it is counted as some other thread's.
///
/// What it decides is which side of the accounting above a write falls on, and nothing else: where a line
/// is written is [`handed_over`]'s answer, which is the same for every thread.
static FRAME_THREAD: AtomicU32 = AtomicU32::new(0);

/// How long [`stop`] waits for the thread to finish what it holds, in milliseconds.
///
/// The same wait [`crate::writer::stop`] takes and for the same reason: long enough for a write that is
/// slow — which is what this whole arrangement is about — and not long enough to hold a game that is
/// closing behind a disk that has stopped answering. What is still waiting after it is written here.
const STOP_WAIT_MS: u64 = 2_000;

/// The lines waiting for the thread that writes them, and whether that thread is still to stop.
///
/// A queue and not one waiting line, which is where this differs from [`crate::writer`]'s single slot per
/// file: what a file holds is where the run is *now*, so a write the next one overtakes is a write nothing
/// wants — where a log is the sequence itself, and a line dropped is a line nobody can ask about again.
struct Waiting {
    lines: Vec<String>,
    /// How many lines have been handed over, and how many of those are in the file.
    ///
    /// Two counts and not the queue's own length, because the queue is empty for as long as a write is in
    /// flight: the thread takes what it holds out before it writes it, so a reader waiting for an empty
    /// queue would be waiting for nothing — see [`written_out`].
    said: u64,
    written: u64,
    stopping: bool,
    stopped: bool,
}

static WAITING: Mutex<Waiting> = Mutex::new(Waiting {
    lines: Vec::new(),
    said: 0,
    written: 0,
    stopping: false,
    stopped: false,
});
static CHANGED: Condvar = Condvar::new();
/// Whether there is a thread writing. Cleared by [`stop`], so that a launch after one that was taken out
/// writes its own lines rather than handing them to a thread that has gone.
static WRITING: AtomicBool = AtomicBool::new(false);

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
#[macro_export]
macro_rules! summary {
    ($($arg:tt)*) => {
        if $crate::log::wanted($crate::log::NORMAL) {
            $crate::log::line(&format!($($arg)*))
        }
    };
}

/// Per-frame detail, for finding out why one frame in a hundred was late. Only at
/// `verbose`, because it is written faster than it can be read.
#[macro_export]
macro_rules! detail {
    ($($arg:tt)*) => {
        if $crate::log::wanted($crate::log::VERBOSE) {
            $crate::log::line(&format!($($arg)*))
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
#[macro_export]
macro_rules! pacing {
    ($($arg:tt)*) => {
        if $crate::log::pacing_wanted() {
            $crate::log::line(&format!($($arg)*))
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
    // What is waiting goes in first: this is the last moment there is, and the crash filter reaches it on
    // the thread that faulted — which for a fault inside a frame is the one whose lines say what the run
    // was doing when it happened. Every one of those lines was handed over rather than written.
    stop();
    let file = FILE.swap(0, Ordering::AcqRel);
    if file != 0 {
        logfile::close(LogFile(file));
    }
}

/// Says a line: stamped where it is said, and handed to the thread that writes it — or written here where
/// there is no such thread yet.
///
/// **The stamp is taken here and not where the line reaches the disk**, because what a reader takes a
/// line's time for is when the thing it says happened. A run read after the fact is read by those stamps.
pub fn line(message: &str) {
    let file = FILE.load(Ordering::Acquire);
    if file == 0 {
        return;
    }
    let line = format!("[{:>8}ms] {message}\r\n", clock::ticks());
    if !handed_over(&line) {
        write_where_it_stands(file, &line);
    }
}

/// The write itself, counted against the thread that made it.
///
/// **Writes and not lines said**, which is what makes the count the instrument it is: the frame's thread
/// hands its lines over, so what it reports is nothing at all — and a launch whose lines it *is* writing,
/// before the loop's first turn or where no thread could be made, reports them. See [`spent`], and the
/// `log Nus in M writes here` of the frame loop's own line.
fn write_where_it_stands(file: usize, line: &str) {
    let started = clock::counter();
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

/// **Claims the calling thread as the frame's and starts the thread that writes the log**, which is what
/// takes every line after this off the frame's own.
///
/// Called from the frame loop, which is the one place that knows both things: which thread the frame runs
/// on — the split the accounting above is read by — and that `DllMain` is behind us, a thread made in
/// there being one whose own `DLL_THREAD_ATTACH` waits for the loader lock the caller holds.
///
/// Idempotent, and cheap enough to call every frame: a load, a compare, and nothing else once the thread
/// is up.
pub fn off_the_frame() {
    FRAME_THREAD.store(thread::current_id(), Ordering::Relaxed);
    if WRITING
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
        && orb_api::thread::spawn(writing).is_err()
    {
        WRITING.store(false, Ordering::Relaxed);
        line("log: cannot start a thread; the log is written in the frame");
    }
}

/// Hands a line to that thread, and says whether it took it.
///
/// `false` where there is no thread to take it — before the frame loop's first turn, after [`stop`], and
/// in a launch whose thread could not be made — and then the caller writes it where it stands.
fn handed_over(line: &str) -> bool {
    if !WRITING.load(Ordering::Relaxed) {
        return false;
    }
    let Ok(mut waiting) = WAITING.lock() else {
        // The thread left the queue locked, which is a panic inside it: nothing more can be handed over,
        // and the caller writing where it stands is what keeps the log going.
        WRITING.store(false, Ordering::Relaxed);
        return false;
    };
    waiting.lines.push(line.to_owned());
    waiting.said += 1;
    CHANGED.notify_all();
    true
}

/// The thread's own body: whatever is waiting, in the order it was said, until [`stop`] says otherwise.
fn writing() {
    // Below the game's, which is what every thread of orb's own says of itself: what it does here is wait
    // and write, and neither is worth a core the frame wants. It is also what keeps this thread out of the
    // clock a pacing e2e test reads — a simulated counter moves by what a read costs, and reads from here
    // would move the frames' own moments by however this machine happened to schedule them.
    orb_api::thread::below_normal();
    loop {
        let Ok(mut waiting) = WAITING.lock() else {
            return;
        };
        while waiting.lines.is_empty() && !waiting.stopping {
            let Ok(next) = CHANGED.wait(waiting) else {
                return;
            };
            waiting = next;
        }
        let lines = std::mem::take(&mut waiting.lines);
        let stopping = waiting.stopping;
        // Not while the queue is held: a write takes what it takes, and the frame's thread hands its next
        // line over in the middle of it.
        drop(waiting);
        let file = FILE.load(Ordering::Acquire);
        let written = lines.len() as u64;
        for line in lines {
            if file != 0 {
                write_where_it_stands(file, &line);
            }
        }
        // Said to be written, for whoever is waiting to read them — see [`written_out`].
        if let Ok(mut waiting) = WAITING.lock() {
            waiting.written += written;
            CHANGED.notify_all();
        }
        if stopping {
            if let Ok(mut waiting) = WAITING.lock() {
                waiting.stopped = true;
                CHANGED.notify_all();
            }
            return;
        }
    }
}

/// Waits until everything said so far is in the file.
///
/// **For whoever reads the log while the run is still going**, which is a test: orb hands its lines to a
/// thread, so a test that pressed a key and asked what the log said would be asking before the line it is
/// waiting for had been written. The run itself never waits — that is what the thread is for — and nothing
/// in a shipped launch calls this.
///
/// Bounded by [`STOP_WAIT_MS`] for the reason [`stop`] is: a disk that has stopped answering is not
/// something to hold a caller behind for ever.
pub fn written_out() {
    if !WRITING.load(Ordering::Relaxed) {
        return;
    }
    let Ok(waiting) = WAITING.lock() else { return };
    let _ = CHANGED.wait_timeout_while(
        waiting,
        std::time::Duration::from_millis(STOP_WAIT_MS),
        |waiting| waiting.written < waiting.said,
    );
}

/// Stops that thread and writes what it still holds, on the calling thread.
///
/// **What is waiting goes in first, wherever this is called from**: that is the last moment there is, and
/// the crash filter reaches it on the thread that faulted — whose own lines are the ones saying what the
/// run was doing when it happened.
pub fn stop() {
    if !WRITING.swap(false, Ordering::Relaxed) {
        return;
    }
    let Ok(mut waiting) = WAITING.lock() else {
        return;
    };
    waiting.stopping = true;
    CHANGED.notify_all();
    let waited = CHANGED.wait_timeout_while(
        waiting,
        std::time::Duration::from_millis(STOP_WAIT_MS),
        |waiting| !waiting.stopped,
    );
    // And whatever it did not get to, written here: a thread stuck in a slow write is what the wait above
    // is bounded for, and the lines behind that write are still worth having.
    if let Ok((mut waiting, _)) = waited {
        let lines = std::mem::take(&mut waiting.lines);
        waiting.stopping = false;
        waiting.stopped = false;
        drop(waiting);
        let file = FILE.load(Ordering::Acquire);
        for line in lines {
            if file != 0 {
                write_where_it_stands(file, &line);
            }
        }
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
