//! Which simulated Windows the calling thread reads through.
//!
//! Installed per thread. The test harness runs tests side by side in one process, so a
//! simulated Windows in a static would be two tests writing each other's game; and it hands
//! its threads out again, so the installation has to come back off at the end of a test rather
//! than be left for whatever runs there next — which is what [`Installed`] is for.

use std::cell::RefCell;
use std::sync::Arc;

use crate::Win;

thread_local! {
    static INSTALLED: RefCell<Option<Arc<dyn Win>>> = const { RefCell::new(None) };
}

/// Puts whatever was installed before back, when it goes.
///
/// Nested rather than refused, so that a test can have two games laid out at once and each
/// read the one it is asking about — the innermost is the one in front.
#[must_use = "the simulated Windows comes off the thread the moment this is dropped"]
pub struct Installed(Option<Arc<dyn Win>>);

impl Drop for Installed {
    fn drop(&mut self) {
        let previous = self.0.take();
        INSTALLED.with(|installed| *installed.borrow_mut() = previous);
    }
}

/// Puts `win` in front of the real host, for this thread, until the return value is dropped.
///
/// Takes an `Arc` so that the threads a simulated game runs on can be given the same one.
pub fn install(win: &Arc<dyn Win>) -> Installed {
    INSTALLED.with(|installed| {
        let mut installed = installed.borrow_mut();
        Installed(installed.replace(Arc::clone(win)))
    })
}

/// The simulated Windows this thread reads through, if any.
pub fn installed() -> Option<Arc<dyn Win>> {
    INSTALLED.with(|installed| installed.borrow().clone())
}
