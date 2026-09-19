//! The log, kept rather than written.
//!
//! A test asserting on what orb said is worth more than a temporary file would be: the log is the
//! instrument orb's own findings are read out of, so an e2e test that claims a chapter began can
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
    /// What to do before answering what orb said — see [`Log::waits_first`].
    waits: Mutex<Option<Box<dyn Fn() + Send + Sync>>>,
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
        // **A write after the close is dropped rather than made a panic**, which is what the host does
        // with one: a thread of orb's own — the replay's writer, the joystick's poll, the sampler — can be
        // inside `log!` while the game closes its file at the end of a launch, and a `WriteFile` on a
        // closed handle fails there and costs nothing. Held as a panic it failed a test whose subject was
        // somewhere else entirely: `th07::the_frames_reach_the_device_inside_a_letterbox_of_the_games_own_shape`
        // came back `written to after being closed` under a suite heavy enough for that thread to be late,
        // and passed alone. What the assert was for is the handle, which is checked above.
        if state.closed {
            return;
        }
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

    /// **What to do before answering any of the questions below**, which is how a reader waits for what has
    /// been *said* to have been written.
    ///
    /// orb hands its lines to a thread of its own rather than writing them where they are said — see
    /// `orb_core::log`, and the three quarters of a second one write took on the real game — so a test that
    /// pressed a key and asked what the log said would otherwise be asking before the line it waits for had
    /// arrived. Installed by whoever drives orb, this crate being the one that must not know about it: a
    /// simulated Windows has no business calling into the thing it is simulating for.
    pub fn waits_first(&self, wait: impl Fn() + Send + Sync + 'static) {
        *self.waits.lock().unwrap() = Some(Box::new(wait));
    }

    /// Everything said so far written, where a reader asked for that — see [`Log::waits_first`].
    fn waited(&self) {
        // Not while the lock is held: what the wait is for is a thread that writes, and writing takes the
        // state's own lock.
        let waits = self.waits.lock().unwrap();
        if let Some(wait) = waits.as_ref() {
            wait();
        }
    }

    /// Every line orb has written, in the order it wrote them.
    pub fn lines(&self) -> Vec<String> {
        self.waited();
        self.state.lock().unwrap().lines.clone()
    }

    /// Whether any line holds `needle`, which is how an e2e test asserts on what orb reported.
    pub fn said(&self, needle: &str) -> bool {
        self.lines().iter().any(|line| line.contains(needle))
    }

    /// How many lines orb has written, which is where an e2e test marks its place before walking something
    /// it has walked once already.
    pub fn written(&self) -> usize {
        self.waited();
        self.state.lock().unwrap().lines.len()
    }

    /// And whether any line since that mark holds `needle`. [`said`](Self::said) asks whether a line was
    /// *ever* written, so an e2e test waiting on one for the second time is answered yes by the first — and
    /// waits out nothing at all.
    pub fn said_since(&self, from: usize, needle: &str) -> bool {
        self.lines()
            .iter()
            .skip(from)
            .any(|line| line.contains(needle))
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
