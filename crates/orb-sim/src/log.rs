//! The log, kept rather than written.
//!
//! A test asserting on what orb said is worth more than a temporary file would be: the log is the
//! instrument orb's own findings are read out of, so a scenario that claims a chapter began can
//! say so by pointing at the line that announced it.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use orb_api::LogFile;

/// The one handle a simulated log hands out. Non-zero, because orb keeps "no log file" as zero.
pub const HANDLE: LogFile = LogFile(1);

#[derive(Default)]
struct State {
    /// Where the log was opened, and whether it was started over rather than appended to — which
    /// is the whole of what `max_bytes` decides.
    opened: Option<(PathBuf, bool)>,
    closed: bool,
    lines: Vec<String>,
}

#[derive(Default)]
pub struct Log {
    state: Mutex<State>,
    /// What the file already held when the run began, for a test about the size at which the log is
    /// started over rather than appended to.
    existing_bytes: Mutex<u64>,
}

impl Log {
    pub fn new() -> Self {
        Self::default()
    }

    /// Says the log already has this many bytes in it, so that a test can reach the case where it
    /// is replaced instead of appended to.
    pub fn set_existing_bytes(&self, bytes: u64) {
        *self.existing_bytes.lock().unwrap() = bytes;
    }

    pub fn open(&self, path: &Path, max_bytes: u64) -> Option<LogFile> {
        let existing = *self.existing_bytes.lock().unwrap();
        let mut state = self.state.lock().unwrap();
        let restarted = existing > max_bytes;
        if restarted {
            state.lines.clear();
        }
        state.opened = Some((path.to_path_buf(), restarted));
        state.closed = false;
        Some(HANDLE)
    }

    pub fn write(&self, file: LogFile, bytes: &[u8]) {
        assert_eq!(file, HANDLE, "a log handle no simulated open handed out");
        let mut state = self.state.lock().unwrap();
        assert!(!state.closed, "written to after being closed");
        // Lossy rather than refused: orb formats its own lines and they are all UTF-8, so a
        // failure here would be a test's own bad bytes and not something to make it prove.
        state
            .lines
            .push(String::from_utf8_lossy(bytes).trim_end().to_string());
    }

    pub fn close(&self, file: LogFile) {
        assert_eq!(file, HANDLE, "a log handle no simulated open handed out");
        self.state.lock().unwrap().closed = true;
    }

    /// Every line orb has written, in the order it wrote them.
    pub fn lines(&self) -> Vec<String> {
        self.state.lock().unwrap().lines.clone()
    }

    /// Whether any line holds `needle`, which is how a scenario asserts on what orb reported.
    pub fn said(&self, needle: &str) -> bool {
        self.lines().iter().any(|line| line.contains(needle))
    }

    /// Whether the log has been closed, which orb does on the way out and from the crash handler.
    pub fn closed(&self) -> bool {
        self.state.lock().unwrap().closed
    }

    /// Where the log was opened, or `None` if it never was.
    pub fn path(&self) -> Option<PathBuf> {
        self.state
            .lock()
            .unwrap()
            .opened
            .as_ref()
            .map(|(path, _)| path.clone())
    }

    /// Whether the log was started over rather than appended to.
    pub fn restarted(&self) -> bool {
        self.state
            .lock()
            .unwrap()
            .opened
            .as_ref()
            .is_some_and(|(_, restarted)| *restarted)
    }
}
