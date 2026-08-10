//! Storage for state that only the game's main thread ever touches.
//!
//! A `Mutex` would be misleading here: the frame hook, `DllMain` and the retry
//! menu all run on that one thread, so there is nothing to lock against, and
//! blocking inside a frame is exactly what must not happen.

use std::cell::UnsafeCell;

pub struct MainThread<T>(UnsafeCell<T>);

unsafe impl<T> Sync for MainThread<T> {}

impl<T> MainThread<T> {
    /// **`const fn` because every one of these is a static**, which is also why no e2e test can enter
    /// it: what happens here is const evaluation and not execution, so a coverage run reports it never
    /// run however many launches there were. See `game::proposed`, which is the same zero.
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// # Safety
    /// The caller must be on the game's main thread, and must not already hold
    /// a reference handed out by this method.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}
