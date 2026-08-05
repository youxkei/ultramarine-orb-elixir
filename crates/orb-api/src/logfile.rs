//! The file the log is appended to.
//!
//! Behind the seam rather than left to `std::fs` because of where the first lines come from: they
//! are written out of `DllMain` while the loader lock is held, so the write has to be the bare
//! `WriteFile` and nothing that might take a lock of its own on the way.
//!
//! A simulated Windows keeps the lines instead of a file, which is worth more than a temporary
//! directory would be: what orb said about a frame is then something a test can assert on.

use std::path::Path;

use crate::LogFile;

/// Opens the log for appending, starting it over rather than appending where it has already grown
/// past `max_bytes`.
pub fn open(path: &Path, max_bytes: u64) -> Option<LogFile> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.open_log(path, max_bytes);
    }
    host::open(path, max_bytes)
}

pub fn write(file: LogFile, bytes: &[u8]) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.write_log(file, bytes);
    }
    host::write(file, bytes);
}

pub fn close(file: LogFile) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.close_log(file);
    }
    host::close(file);
}

#[cfg(windows)]
use crate::real::logfile as host;

#[cfg(not(windows))]
mod host {
    use crate::{LogFile, no_windows};
    use std::path::Path;

    pub fn open(_path: &Path, _max_bytes: u64) -> Option<LogFile> {
        no_windows("logfile::open")
    }
    pub fn write(_file: LogFile, _bytes: &[u8]) {
        no_windows("logfile::write")
    }
    pub fn close(_file: LogFile) {
        no_windows("logfile::close")
    }
}
