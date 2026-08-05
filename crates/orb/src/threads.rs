//! Holding the game still while its memory is copied.
//!
//! Only the threads the game created itself can touch the memory a snapshot
//! covers, so only those are stopped. Suspending everything — which is what
//! enumerating the process's threads amounts to — also stops DirectSound's mixer
//! and the graphics driver's workers, and stopping those for the length of a copy
//! is audible as the sound breaking up.
//!
//! The game's audio thread is left running as well, even though it is the game's:
//! it is the one thread whose pauses are heard, and the only game memory it
//! touches is the streaming bookkeeping, which the snapshot checks separately.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::System::Threading::{
    GetThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_CREATION_FLAGS,
    THREAD_SUSPEND_RESUME,
};

use crate::hook;
use crate::log;

/// The threads the game has created, with a handle each so that suspending one
/// needs no lookup. A thread that has exited fails to suspend and is dropped.
static GAME_THREADS: Mutex<Vec<Thread>> = Mutex::new(Vec::new());

static CREATE_THREAD: AtomicUsize = AtomicUsize::new(0);

struct Thread {
    id: u32,
    handle: isize,
}

#[allow(clippy::type_complexity)]
type CreateThread = unsafe extern "system" fn(
    *const c_void,
    usize,
    Option<unsafe extern "system" fn(*mut c_void) -> u32>,
    *const c_void,
    THREAD_CREATION_FLAGS,
    *mut u32,
) -> HANDLE;

/// # Safety
/// Must run before the game creates any thread, and `module` must be the exe.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "KERNEL32.dll",
            "CreateThread",
            hook::address(create_thread as _),
        )
    }?;
    CREATE_THREAD.store(previous, Ordering::Relaxed);
    Ok(())
}

unsafe extern "system" fn create_thread(
    attributes: *const c_void,
    stack_size: usize,
    start: Option<unsafe extern "system" fn(*mut c_void) -> u32>,
    parameter: *const c_void,
    flags: THREAD_CREATION_FLAGS,
    id_out: *mut u32,
) -> HANDLE {
    let original: CreateThread =
        unsafe { std::mem::transmute(CREATE_THREAD.load(Ordering::Relaxed)) };
    let thread = unsafe { original(attributes, stack_size, start, parameter, flags, id_out) };
    if !thread.is_null() {
        remember(unsafe { GetThreadId(thread) });
    }
    thread
}

fn remember(id: u32) {
    if id == 0 {
        return;
    }
    // A handle of our own, so suspending never depends on one the game may close.
    let handle = unsafe { OpenThread(THREAD_SUSPEND_RESUME, FALSE, id) };
    if handle.is_null() {
        return log!("threads: cannot open the game's thread {id}");
    }
    if let Ok(mut threads) = GAME_THREADS.lock() {
        threads.push(Thread {
            id,
            handle: handle as isize,
        });
    }
}

/// The game's threads, stopped for as long as this lives.
pub struct Suspended(Vec<HANDLE>);

impl Suspended {
    /// Stops every thread the game made except the caller's and `audio`.
    pub fn all_but(current: u32, audio: Option<u32>) -> Self {
        let Ok(mut threads) = GAME_THREADS.lock() else {
            return Self(Vec::new());
        };
        let mut suspended = Vec::with_capacity(threads.len());
        threads.retain(|thread| {
            if thread.id == current || Some(thread.id) == audio {
                return true;
            }
            let handle = thread.handle as HANDLE;
            if unsafe { SuspendThread(handle) } == u32::MAX {
                // Already gone: nothing to resume, and nothing to keep.
                unsafe { CloseHandle(handle) };
                return false;
            }
            suspended.push(handle);
            true
        });
        Self(suspended)
    }
}

impl Drop for Suspended {
    fn drop(&mut self) {
        for &handle in &self.0 {
            unsafe { ResumeThread(handle) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GAME_THREADS, Suspended, remember};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::{Mutex, MutexGuard};
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;

    /// Real threads rather than a model of them, because what is being checked is what
    /// `SuspendThread` does — and the list they go into is the process's own, so one test at a time.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    struct Alone(#[expect(dead_code)] MutexGuard<'static, ()>);

    impl Alone {
        fn new() -> Self {
            let held = ONE_AT_A_TIME
                .lock()
                .unwrap_or_else(|held| held.into_inner());
            GAME_THREADS.lock().unwrap().clear();
            Self(held)
        }
    }

    impl Drop for Alone {
        fn drop(&mut self) {
            GAME_THREADS.lock().unwrap().clear();
        }
    }

    /// A thread of the game's that counts for as long as it is let run, so that whether it is
    /// running is something to read rather than something to wait out.
    struct Counter {
        id: u32,
        count: Arc<AtomicU32>,
        stop: Arc<AtomicBool>,
        join: Option<std::thread::JoinHandle<()>>,
    }

    impl Counter {
        fn spawn() -> Self {
            let count = Arc::new(AtomicU32::new(0));
            let stop = Arc::new(AtomicBool::new(false));
            let told = Arc::new(AtomicU32::new(0));
            let join = std::thread::spawn({
                let (count, stop, told) =
                    (Arc::clone(&count), Arc::clone(&stop), Arc::clone(&told));
                move || {
                    told.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
                    while !stop.load(Ordering::Relaxed) {
                        count.fetch_add(1, Ordering::Relaxed);
                        std::thread::yield_now();
                    }
                }
            });
            let id = loop {
                let id = told.load(Ordering::SeqCst);
                if id != 0 {
                    break id;
                }
                std::thread::yield_now();
            };
            remember(id);
            Self {
                id,
                count,
                stop,
                join: Some(join),
            }
        }

        fn count(&self) -> u32 {
            self.count.load(Ordering::Relaxed)
        }

        /// Waits until this one has counted `by` more, and answers whether it did.
        ///
        /// Bounded by turns of the scheduler rather than by a clock: what says a thread is running is
        /// that it got somewhere while another one was getting somewhere, and a wall-clock window is
        /// the flaky way to ask that.
        fn advanced_by(&self, by: u32) -> bool {
            let from = self.count();
            for _ in 0..2_000_000 {
                if self.count() >= from + by {
                    return true;
                }
                std::thread::yield_now();
            }
            false
        }
    }

    impl Drop for Counter {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    /// The whole of what this module is for: the game's threads stop while its memory is copied, and
    /// the one playing the music does not.
    ///
    /// Read off the counters rather than off a clock — the audio thread getting a thousand turns is
    /// what makes "the other one got none" mean something.
    #[test]
    fn the_audio_thread_runs_on_while_the_rest_are_stopped() {
        let _alone = Alone::new();
        let audio = Counter::spawn();
        let other = Counter::spawn();
        assert!(other.advanced_by(1), "it counts before anything stops it");

        {
            let _suspended = Suspended::all_but(unsafe { GetCurrentThreadId() }, Some(audio.id));
            // `SuspendThread` returns before the thread it names has actually stopped — measured
            // here, as a dozen more counts after the call came back — so what is read is the count
            // once that window has passed, not the one from before the call.
            assert!(
                audio.advanced_by(1_000),
                "the music's thread was left running",
            );
            let settled = other.count();
            assert!(audio.advanced_by(1_000), "and goes on running");
            assert_eq!(other.count(), settled, "and the other one was stopped");
        }
        assert!(other.advanced_by(1), "and let go again afterwards");
    }

    /// With no thread named as the audio one, every thread the game made stops. A snapshot taken with
    /// the sound already down is the case: there is nothing left whose pauses would be heard.
    #[test]
    fn naming_no_audio_thread_stops_them_all() {
        let _alone = Alone::new();
        let one = Counter::spawn();
        let two = Counter::spawn();
        assert!(one.advanced_by(1) && two.advanced_by(1));

        {
            let _suspended = Suspended::all_but(unsafe { GetCurrentThreadId() }, None);
            // Nothing left running to count turns against, so this is the one place a wait is the
            // only way to ask. Twice, because `SuspendThread` returns before the thread has stopped:
            // the first wait is that window, and what the second one has to leave alone is the count
            // after it.
            let settle = std::time::Duration::from_millis(50);
            std::thread::sleep(settle);
            let (first, second) = (one.count(), two.count());
            std::thread::sleep(settle);
            assert_eq!((one.count(), two.count()), (first, second));
        }
        assert!(one.advanced_by(1) && two.advanced_by(1));
    }

    /// The caller's own thread is never suspended, whatever else is: it is the one doing the copying,
    /// and stopping it is a process that never comes back.
    #[test]
    fn the_thread_doing_the_copying_is_left_alone() {
        let _alone = Alone::new();
        let current = unsafe { GetCurrentThreadId() };
        remember(current);

        let _suspended = Suspended::all_but(current, None);
        // Reaching this line is the assertion.
        assert!(GAME_THREADS.lock().unwrap().iter().any(|t| t.id == current));
    }

    /// A thread that has exited is dropped from the list rather than kept and retried: its handle
    /// names nothing, and a list that grew a dead entry per stage would be a list of them.
    #[test]
    fn a_thread_that_has_gone_is_forgotten() {
        let _alone = Alone::new();
        let id = {
            let counter = Counter::spawn();
            counter.id
            // Dropped here, which stops and joins it.
        };
        assert!(GAME_THREADS.lock().unwrap().iter().any(|t| t.id == id));

        let _suspended = Suspended::all_but(unsafe { GetCurrentThreadId() }, None);
        assert!(
            !GAME_THREADS.lock().unwrap().iter().any(|t| t.id == id),
            "the dead thread was dropped from the list",
        );
    }
}
