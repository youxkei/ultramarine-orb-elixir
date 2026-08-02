//! Append-only log beside the game exe, which is where the launcher and `orb.yaml` are too.
//!
//! Every run appends rather than starting the file over: a run worth looking at
//! is often over by the time anyone looks, and the next launch would otherwise
//! have thrown it away. Only a log that has grown past `MAX_BYTES` is replaced.
//!
//! Writes go straight to `WriteFile` rather than through `std::fs`, because the
//! first lines are emitted from `DllMain` while the loader lock is held.

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicIsize, AtomicU32, Ordering};

use orb_config::LogLevel;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CreateFileW, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ,
    FILE_SHARE_WRITE, GetFileAttributesExW, GetFileExInfoStandard, OPEN_ALWAYS,
    WIN32_FILE_ATTRIBUTE_DATA, WriteFile,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

use crate::sync::MainThread;

/// Zero means "no log file", which is also how the process starts out.
static FILE: AtomicIsize = AtomicIsize::new(0);

/// Where the log is started over rather than appended to.
const MAX_BYTES: u64 = 8 << 20;

/// How much is being written. Set once the config has been read; until then
/// everything is written, because a fault during startup is the one worth having.
static LEVEL: AtomicIsize = AtomicIsize::new(VERBOSE);

const QUIET: isize = 0;
pub(crate) const NORMAL: isize = 1;
pub(crate) const VERBOSE: isize = 2;

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
/// This is also what makes `defer` and `drain` safe from any thread without a lock:
/// only the one thread whose id this is ever touches `HELD`.
static FRAME_THREAD: AtomicU32 = AtomicU32::new(0);

/// Lines held back until the frame loop reaches a moment where writing one costs
/// nothing.
///
/// The moments between handing a frame over and the blank it is shown at are the ones a
/// write must stay out of: the next frame has about a millisecond to reach `DwmFlush`,
/// and one that arrives after the blank has gone waits out another refresh and is shown
/// late. On the far side of that flush there are fourteen milliseconds of slack doing
/// nothing, so that is where these go.
///
/// Only what the frame loop says about itself is held back. Startup and faults are
/// written as they happen, because a run that ends in a crash must not lose them.
static HELD: MainThread<Vec<String>> = MainThread::new(Vec::new());
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
macro_rules! log {
    ($($arg:tt)*) => { crate::log::line(&format!($($arg)*)) };
}
pub(crate) use log;

/// How a run is going: a line a second, or a line per scene. Off at `quiet`.
macro_rules! summary {
    ($($arg:tt)*) => {
        if crate::log::wanted(crate::log::NORMAL) {
            crate::log::line(&format!($($arg)*))
        }
    };
}
pub(crate) use summary;

/// Per-frame detail, for finding out why one frame in a hundred was late. Only at
/// `verbose`, because it is written faster than it can be read.
macro_rules! detail {
    ($($arg:tt)*) => {
        if crate::log::wanted(crate::log::VERBOSE) {
            crate::log::line(&format!($($arg)*))
        }
    };
}
pub(crate) use detail;

/// What the frame loop says about the pacing, which `--pacing` turns on by itself and
/// every level writes.
///
/// Not a tier of the level, because `verbose` also turns on the writers that are among
/// the suspects: at `--log=quiet --pacing` the file holds the startup lines and these,
/// and every write in the run is one this made.
///
/// Held back until the frame loop has slack for it, since writing it where it is worked
/// out would cost the next frame a refresh — which is the very thing being counted.
macro_rules! pacing {
    ($($arg:tt)*) => {
        if crate::log::pacing_wanted() {
            crate::log::defer(&format!($($arg)*))
        }
    };
}
pub(crate) use pacing;

pub fn open() {
    let Some(path) = host_exe() else { return };
    let path: Vec<u16> = path
        .with_file_name("orb.log")
        .as_os_str()
        .encode_wide()
        .chain([0])
        .collect();
    let disposition = if size_of(&path) > MAX_BYTES {
        CREATE_ALWAYS
    } else {
        OPEN_ALWAYS
    };
    let file = unsafe {
        CreateFileW(
            path.as_ptr(),
            FILE_APPEND_DATA,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            disposition,
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        )
    };
    if file != INVALID_HANDLE_VALUE {
        FILE.store(file as isize, Ordering::Release);
        line("---- new run ----");
    }
}

fn size_of(path: &[u16]) -> u64 {
    let mut attributes: WIN32_FILE_ATTRIBUTE_DATA = unsafe { std::mem::zeroed() };
    let read = unsafe {
        GetFileAttributesExW(
            path.as_ptr(),
            GetFileExInfoStandard,
            (&raw mut attributes).cast(),
        )
    };
    if read == 0 {
        return 0;
    }
    u64::from(attributes.nFileSizeHigh) << 32 | u64::from(attributes.nFileSizeLow)
}

pub fn close() {
    let file = FILE.swap(0, Ordering::AcqRel);
    if file != 0 {
        unsafe { CloseHandle(file as HANDLE) };
    }
}

pub fn line(message: &str) {
    let file = FILE.load(Ordering::Acquire);
    if file == 0 {
        return;
    }
    let file = file as HANDLE;
    // From here rather than from around the `WriteFile` alone: the frame pays for the
    // formatting as much as for the write, and what is wanted is what having written the
    // line cost it.
    let started = counter();
    let line = format!("[{:>8}ms] {message}\r\n", unsafe { GetTickCount() });
    let mut written = 0u32;
    unsafe {
        WriteFile(
            file,
            line.as_ptr(),
            line.len() as u32,
            &mut written,
            std::ptr::null_mut(),
        );
    }
    let side = usize::from(unsafe { GetCurrentThreadId() } != FRAME_THREAD.load(Ordering::Relaxed));
    SPENT[side].fetch_add(counter() - started, Ordering::Relaxed);
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
    if unsafe { GetCurrentThreadId() } != FRAME_THREAD.load(Ordering::Relaxed) {
        return line(message);
    }
    let held = unsafe { HELD.get() };
    if held.len() >= HELD_MAX {
        // A drain happens twice per frame's turn, so this many held means the frame loop
        // has stopped running and no better moment is coming. Written where they stand
        // rather than lost, which for a run that ended in a fault is the difference
        // between having the last frames and not.
        line("log: the frame loop has stopped draining; writing what is held where it stands");
        for held in held.drain(..) {
            line(&held);
        }
        return line(message);
    }
    held.push(message.to_string());
}

/// Writes what has been held back, and claims the calling thread as the frame's.
///
/// To be called from the frame loop at a point where a write costs nothing: after the
/// wait for the blank, not before it.
pub fn drain() {
    let thread = unsafe { GetCurrentThreadId() };
    if FRAME_THREAD.swap(thread, Ordering::Relaxed) != thread {
        // The first drain, or the frame moved threads. Nothing is held for this one yet.
        return;
    }
    let held = unsafe { HELD.get() };
    for message in held.drain(..) {
        line(&message);
    }
}

fn counter() -> i64 {
    let mut counter = 0;
    unsafe { QueryPerformanceCounter(&mut counter) };
    counter
}

fn frequency() -> i64 {
    let cached = FREQUENCY.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let mut frequency = 0;
    unsafe { QueryPerformanceFrequency(&mut frequency) };
    FREQUENCY.store(frequency, Ordering::Relaxed);
    frequency
}

/// The path of the exe this is running inside — the game's, since that is the process orb
/// is injected into. `orb.yaml` and the log go in its directory, which is where the
/// launcher is installed too.
///
/// Not this DLL's own directory: the launcher carries the DLL inside itself and unpacks it
/// to the temp directory, so asking where the DLL is would find neither the config nor a
/// sensible place for the log.
pub fn host_exe() -> Option<PathBuf> {
    module_path(std::ptr::null_mut())
}

/// Where a loaded module's file is. Null for the exe of this process.
///
/// Used by the crash handler to say which module an address is in, which is worth having:
/// most of what has gone wrong here was found by reading `module+offset` out of the log.
pub fn module_path(module: HANDLE) -> Option<PathBuf> {
    let mut buffer = [0u16; 1024];
    let length = unsafe { GetModuleFileNameW(module, buffer.as_mut_ptr(), buffer.len() as u32) };
    if length == 0 || length as usize >= buffer.len() {
        return None;
    }
    Some(PathBuf::from(OsString::from_wide(
        &buffer[..length as usize],
    )))
}
