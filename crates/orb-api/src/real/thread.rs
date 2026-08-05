//! Which real thread is running, and stopping the real threads the game made.
//!
//! The table lives here rather than over the seam because what a thread handle is belongs on this
//! side: orb notices a thread being created — an import hook on `CreateThread` — and hands over the
//! id, and everything done with it afterwards is Windows'.

use std::sync::Mutex;

use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::System::Threading::{
    GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread, THREAD_SUSPEND_RESUME,
};

/// The threads the game has created, with a handle each so that suspending one needs no lookup. A
/// thread that has exited fails to suspend and is dropped.
static GAME_THREADS: Mutex<Vec<Thread>> = Mutex::new(Vec::new());

struct Thread {
    id: u32,
    handle: isize,
}

pub fn current_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}

/// Opens a handle of our own for `id`, so that suspending never depends on one the game may close.
pub fn registered(id: u32) -> bool {
    if id == 0 {
        return false;
    }
    let handle = unsafe { OpenThread(THREAD_SUSPEND_RESUME, FALSE, id) };
    if handle.is_null() {
        return false;
    }
    if let Ok(mut threads) = GAME_THREADS.lock() {
        threads.push(Thread {
            id,
            handle: handle as isize,
        });
    }
    true
}

pub fn suspend_game_threads(audio: Option<u32>) -> Vec<u32> {
    let current = current_id();
    let Ok(mut threads) = GAME_THREADS.lock() else {
        return Vec::new();
    };
    let mut suspended = Vec::with_capacity(threads.len());
    threads.retain(|thread| {
        if thread.id == current || Some(thread.id) == audio {
            return true;
        }
        if unsafe { SuspendThread(thread.handle as HANDLE) } == u32::MAX {
            // Already gone: nothing to resume, and nothing to keep.
            unsafe { CloseHandle(thread.handle as HANDLE) };
            return false;
        }
        suspended.push(thread.id);
        true
    });
    suspended
}

pub fn resume_threads(ids: &[u32]) {
    let Ok(threads) = GAME_THREADS.lock() else {
        return;
    };
    for id in ids {
        if let Some(thread) = threads.iter().find(|thread| thread.id == *id) {
            unsafe { ResumeThread(thread.handle as HANDLE) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GAME_THREADS, registered, resume_threads, suspend_game_threads};
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

    /// Stops them and starts them again at the end of the block, the way the guard over the seam
    /// does — the guard itself lives there because that is where the callers are.
    struct Held(Vec<u32>);

    impl Held {
        fn all_but_audio(audio: Option<u32>) -> Self {
            Self(suspend_game_threads(audio))
        }
    }

    impl Drop for Held {
        fn drop(&mut self) {
            resume_threads(&self.0);
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
            registered(id);
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

    /// The whole of what this is for: the game's threads stop while its memory is copied, and the one
    /// playing the music does not.
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
            let _suspended = Held::all_but_audio(Some(audio.id));
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
            let _suspended = Held::all_but_audio(None);
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
        registered(current);

        let _suspended = Held::all_but_audio(None);
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

        let _suspended = Held::all_but_audio(None);
        assert!(
            !GAME_THREADS.lock().unwrap().iter().any(|t| t.id == id),
            "the dead thread was dropped from the list",
        );
    }
}
