//! Which thread is running.

/// `GetCurrentThreadId`. Never zero, which is what lets the log claim the frame's own thread with
/// nothing but a compare-and-swap.
pub fn current_id() -> u32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.current_thread_id();
    }
    host::current_id()
}

#[cfg(windows)]
use crate::real::thread as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn current_id() -> u32 {
        no_windows("thread::current_id")
    }
}
