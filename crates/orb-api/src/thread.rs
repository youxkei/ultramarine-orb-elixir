//! Which thread is running, and stopping the ones the game made.

/// `GetCurrentThreadId`. Never zero, which is what lets the log claim the frame's own thread with
/// nothing but a compare-and-swap.
pub fn current_id() -> u32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.current_thread_id();
    }
    host::current_id()
}

/// A thread of orb's own, with whatever simulated Windows the caller reads through carried onto it.
///
/// **The installation is per thread**, which is why this is here rather than a plain
/// `std::thread::spawn` at the call site: a thread spawned without it sees no simulated host and falls
/// through to the real one, and what that costs is the joystick poller reading this machine's own winmm
/// instead of the one a scenario laid out — a test that cannot fail. Carried at the spawn rather than
/// installed by the body, so that the body is the same code either way; `orb-sim`'s own
/// `log_off_thread` scenario hands the same `Arc` to the thread it makes, which is the same move from
/// the other side.
///
/// Nothing to carry in a shipped launch, where `installed()` is `None` and this is
/// `std::thread::Builder::spawn` and the handle dropped.
pub fn spawn(body: impl FnOnce() + Send + 'static) -> std::io::Result<()> {
    #[cfg(feature = "sim")]
    let carried = crate::installed();
    std::thread::Builder::new()
        .spawn(move || {
            #[cfg(feature = "sim")]
            let _installed = carried.as_ref().map(crate::install);
            body();
        })
        .map(|_| ())
}

/// Remembers a thread the game has just created, and answers whether it can be stopped later.
///
/// The id and not a handle: the noticing is orb's — an import hook on `CreateThread` — and opening a
/// handle of our own, so that suspending never depends on one the game may close, is the host's.
/// `false` is a thread that cannot be opened, which the caller logs.
pub fn registered(id: u32) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.register_thread(id);
    }
    host::registered(id)
}

/// The game's own threads, stopped for as long as this lives.
///
/// A guard rather than a pair of calls because the one thing that must not happen is a copy that
/// returns without starting them again: the game would be frozen for the rest of the run.
pub struct Suspended(Vec<u32>);

impl Suspended {
    /// Stops every thread the game made except the caller's and `audio`.
    ///
    /// Only the game's own, which are the only ones that can touch the memory a snapshot covers.
    /// Suspending everything — which is what enumerating the process's threads amounts to — also
    /// stops DirectSound's mixer and the graphics driver's workers, and stopping those for the length
    /// of a copy is audible as the sound breaking up. The audio thread is left running for the same
    /// reason even though it is the game's: the only game memory it touches is the streaming
    /// bookkeeping, which the snapshot checks separately.
    pub fn all_but_audio(audio: Option<u32>) -> Self {
        #[cfg(feature = "sim")]
        if let Some(win) = crate::installed() {
            return Self(win.suspend_game_threads(audio));
        }
        Self(host::suspend_game_threads(audio))
    }

    /// How many were stopped, for the log line that says what holding the game still cost.
    pub fn count(&self) -> usize {
        self.0.len()
    }
}

impl Drop for Suspended {
    fn drop(&mut self) {
        if self.0.is_empty() {
            return;
        }
        #[cfg(feature = "sim")]
        if let Some(win) = crate::installed() {
            win.resume_threads(&self.0);
            return;
        }
        host::resume_threads(&self.0);
    }
}

#[cfg(windows)]
use crate::real::thread as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn current_id() -> u32 {
        no_windows("thread::current_id")
    }
    pub fn registered(_id: u32) -> bool {
        no_windows("thread::registered")
    }
    pub fn suspend_game_threads(_audio: Option<u32>) -> Vec<u32> {
        no_windows("thread::suspend_game_threads")
    }
    pub fn resume_threads(_ids: &[u32]) {
        no_windows("thread::resume_threads")
    }
}
