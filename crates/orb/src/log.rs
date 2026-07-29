//! Append-only log beside `orb.dll`.
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
use std::sync::atomic::{AtomicIsize, Ordering};

use orb_config::LogLevel;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_ALWAYS, CreateFileW, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ,
    FILE_SHARE_WRITE, GetFileAttributesExW, GetFileExInfoStandard, OPEN_ALWAYS,
    WIN32_FILE_ATTRIBUTE_DATA, WriteFile,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::SystemInformation::GetTickCount;

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

pub fn open() {
    let Some(path) = host_exe() else { return };
    let path: Vec<u16> = path.with_file_name("orb.log").as_os_str().encode_wide().chain([0]).collect();
    let disposition = if size_of(&path) > MAX_BYTES { CREATE_ALWAYS } else { OPEN_ALWAYS };
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
    let line = format!("[{:>8}ms] {message}\r\n", unsafe { GetTickCount() });
    let mut written = 0u32;
    unsafe {
        WriteFile(file, line.as_ptr(), line.len() as u32, &mut written, std::ptr::null_mut());
    }
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
    Some(PathBuf::from(OsString::from_wide(&buffer[..length as usize])))
}
