//! Which thread is running, what a thread of orb's own runs at, and stopping the ones the game made.
//!
//! The rule the game's threads are stopped by — every thread the game made stops while its memory is
//! copied, and the one playing the music does not — is checked with real threads rather than a model of
//! them, since what is being relied on is what `SuspendThread` does. A test registers threads of its own as
//! the game's and reads whether each is still counting. Counted turns rather than elapsed time: what says a
//! thread is running is that it got somewhere while another one was getting somewhere, and a wall-clock
//! window is the flaky way to ask. The exception is the case where nothing is left running to count
//! against, which has to wait. `SuspendThread` returns before the thread it names has stopped — a dozen more
//! counts after the call came back — so what a test reads is the count once that window has passed, not
//! the one from before the call.

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
/// instead of the one an e2e test laid out — a test that cannot fail. Carried at the spawn rather than
/// installed by the body, so that the body is the same code either way; `orb-sim`'s own
/// `log_off_thread` e2e test hands the same `Arc` to the thread it makes, which is the same move from
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

/// The calling thread put below the game's, which the sampling thread says of itself.
///
/// Called by the body rather than given to [`spawn`] because that is where the thread is — see
/// [`Win::below_normal`](crate::Win::below_normal).
pub fn below_normal() {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.below_normal();
    }
    host::below_normal();
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

/// Where a thread is, as a sample of it taken while it was held still.
pub struct Executing {
    /// The instruction it will run next.
    pub at: usize,
    /// The calls it is inside, innermost first: the words on its stack that point into a mapped image,
    /// which is what a return address looks like and what nothing else on a stack does.
    ///
    /// **Return addresses and not a walk of the frames**, which on x64 needs the unwind tables of every
    /// module on the stack and answers nothing more here: what is being asked is which of the game's own
    /// functions the thread is under, and a word pointing into the exe answers that whether or not the
    /// frame it belongs to can be named.
    pub under: Vec<usize>,
}

/// **Where the thread `id` is executing**, read with the thread held still for the read and let go
/// again. `None` where the thread cannot be opened or read — and always in a test, where a simulated
/// thread has no instruction pointer to read.
///
/// What it is for is a frame that ran long inside the game's own update, which no phase of orb's accounts
/// for: a sample of where the game was, taken from a thread of orb's while the frame's own thread is
/// still in there, is what says which of the game's functions the time went to.
pub fn where_it_is(id: u32) -> Option<Executing> {
    #[cfg(feature = "sim")]
    if crate::installed().is_some() {
        return None;
    }
    host::where_it_is(id)
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
    pub fn below_normal() {
        no_windows("thread::below_normal")
    }
    pub fn registered(_id: u32) -> bool {
        no_windows("thread::registered")
    }
    pub fn where_it_is(_id: u32) -> Option<super::Executing> {
        no_windows("thread::where_it_is")
    }
    pub fn suspend_game_threads(_audio: Option<u32>) -> Vec<u32> {
        no_windows("thread::suspend_game_threads")
    }
    pub fn resume_threads(_ids: &[u32]) {
        no_windows("thread::resume_threads")
    }
}
