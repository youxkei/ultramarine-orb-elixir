//! The real host's clock, and the timer the frame loop waits on.

use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{FALSE, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::Threading::{
    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, INFINITE, SetWaitableTimer,
    Sleep, TIMER_ALL_ACCESS, WaitForSingleObject,
};

pub fn counter() -> i64 {
    let mut counter = 0;
    unsafe { QueryPerformanceCounter(&mut counter) };
    counter
}

pub fn frequency() -> i64 {
    let mut frequency = 0;
    unsafe { QueryPerformanceFrequency(&mut frequency) };
    frequency
}

/// The `pause` instruction, which is all one turn of a spin is on a real host: no call, no kernel,
/// and about twenty cycles. Here so that the simulated host can charge time for it.
pub fn spin_once() {
    std::hint::spin_loop();
}

/// `Sleep`, and not the timer [`wait`] is made on: nothing is waiting for the thread that calls this,
/// so the system timer tick it rounds to — fifteen milliseconds on this host — costs it nothing.
pub fn sleep(ms: u32) {
    unsafe { Sleep(ms) };
}

/// The timer, made on first use and kept for the rest of the run: the handle, or [`REFUSED`].
///
/// A static rather than anything a caller holds, because nothing about a handle crosses the seam —
/// and made once rather than per wait, which would be a kernel object created and closed sixty
/// times a second.
///
/// **A `OnceLock` and not an atomic loaded and then stored**, which is what this was. Two threads both
/// find it unasked, both call `CreateWaitableTimerExW`, and both store: one handle is leaked, and the two
/// callers are left holding different timers. Measured rather than argued —
/// `this_host_can_create_the_timer_the_waits_are_made_on` and
/// `a_wait_takes_at_least_as_long_as_it_asked_for_and_nothing_like_ten_times_it` run side by side in this
/// crate's test binary and race exactly there, and the assertion that a timer is made once failed with
/// **324 against 284**, two handles forty apart. In a shipped launch only the frame's own thread waits, so
/// nothing there reaches it — but a kernel handle is not a thing to leave resting on a rule about who
/// calls, and the suite is a caller too.
static TIMER: OnceLock<isize> = OnceLock::new();
const REFUSED: isize = -1;

/// A wait aimed at a deadline, in the counter's own ticks.
///
/// `SetWaitableTimer` takes a relative due time in hundreds of nanoseconds, negative, so the ticks
/// are converted here: the seam's unit is the counter's because that is what the caller measured its
/// deadline in.
pub fn wait(ticks: i64) -> bool {
    let timer = timer();
    if timer == REFUSED {
        return false;
    }
    let timer = timer as HANDLE;
    // A host with no counter would divide by zero here. It cannot reach this: every deadline the
    // frame loop works out is then zero ticks away and nothing is waited for at all.
    let due = -(ticks * 10_000_000 / frequency().max(1));
    // A failure of either call is answered the same way as a timer that could not be made, which is
    // the caller saying so and stopping. Not `true`: that says the wait happened, and the caller's
    // loop would ask again for the same deadline and fail again, at a hundred per cent of a core.
    // Not silence either — there is no second way to wait, which is the whole of the decision.
    unsafe {
        if SetWaitableTimer(timer, &due, 0, None, std::ptr::null(), FALSE) == 0 {
            return false;
        }
        WaitForSingleObject(timer, INFINITE) == WAIT_OBJECT_0
    }
}

pub(crate) fn timer() -> isize {
    *TIMER.get_or_init(|| {
        let made = unsafe {
            CreateWaitableTimerExW(
                std::ptr::null(),
                std::ptr::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS,
            )
        };
        if made.is_null() {
            REFUSED
        } else {
            made as isize
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{REFUSED, counter, frequency, timer, wait};

    /// The real calls against the real host, because that is the one thing no scenario reaches: a
    /// simulated `wait` advances a counter, and whether *these* two calls work at all is a fact about
    /// the machine and the `windows-sys` bindings.
    ///
    /// What is deliberately not asserted here is how *accurate* the wait is. That is a property of the
    /// host and what it decided is beside `frame::SPIN_US` — a suite that asserted it would be
    /// asserting that the machine it happens to be running on is a good one.
    #[test]
    fn this_host_can_create_the_timer_the_waits_are_made_on() {
        assert_ne!(
            timer(),
            REFUSED,
            "orb does not run on a host that cannot make it, so a host running this suite had better"
        );
        assert_eq!(timer(), timer(), "made once and kept, not made per wait");
    }

    /// The conversion from the counter's ticks to the timer's hundreds of nanoseconds, which is the
    /// one piece of arithmetic in the wait and the one a wrong factor of ten would hide in.
    #[test]
    fn a_wait_takes_at_least_as_long_as_it_asked_for_and_nothing_like_ten_times_it() {
        let frequency = frequency();
        let asked_us = 5_000;
        let asked = asked_us * frequency / 1_000_000;

        let before = counter();
        assert!(wait(asked), "the host could not wait");
        let took_us = (counter() - before) * 1_000_000 / frequency;

        // Never early: a waitable timer does not signal before its due time, which is what lets the
        // spin after the wait be the only thing between it and the deadline.
        assert!(took_us >= asked_us, "{took_us}us for a {asked_us}us wait");
        // And generously bounded, because the number this is here to catch is a units error — the
        // conversion off by ten makes this 50,000µs. The slack is orders of magnitude above any
        // overshoot measured, so this is a suite that does not fail for being run on a busy machine.
        assert!(
            took_us < asked_us + 20_000,
            "{took_us}us for a {asked_us}us wait"
        );
    }
}
