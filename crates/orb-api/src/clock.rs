//! The host's clock.
//!
//! Only the reading of it is behind the seam. Every decision orb's pacing makes is already a
//! function of the numbers that come out — `grid_aim`, `next_budget`, `whole_multiple`,
//! `on_cadence` — so those are tested by handing them numbers, and what a seam adds is the order
//! the waiting calls come in. What matters about the waiting is whether frames land on blanks,
//! which is a measurement of real hardware; `DONE.md` keeps those.

/// `QueryPerformanceCounter`, for measuring how long something took.
pub fn counter() -> i64 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.counter();
    }
    host::counter()
}

/// Ticks a second, or zero where the host has no counter — which every caller divides by and so
/// has to check.
pub fn frequency() -> i64 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.frequency();
    }
    host::frequency()
}

/// Milliseconds since the host started, which is what every log line is stamped with.
pub fn ticks() -> u32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.ticks();
    }
    host::ticks()
}

/// Waits, to the timer resolution [`begin_period`] asked for.
pub fn sleep_millis(millis: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.sleep_millis(millis);
    }
    host::sleep_millis(millis);
}

/// Asks for a timer resolution. Zero is granted; anything else is the error the caller logs.
///
/// Without it `Sleep` is only accurate to the system's tick, which is some fifteen milliseconds —
/// nearly two refreshes at 120Hz, and exactly the size of the stutter measured whenever the pacing
/// fell back to the clock.
pub fn begin_period(millis: u32) -> u32 {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.begin_period(millis);
    }
    host::begin_period(millis)
}

/// Gives back what [`begin_period`] was asked for.
pub fn end_period(millis: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.end_period(millis);
    }
    host::end_period(millis);
}

#[cfg(windows)]
use crate::real::clock as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn counter() -> i64 {
        no_windows("clock::counter")
    }
    pub fn frequency() -> i64 {
        no_windows("clock::frequency")
    }
    pub fn ticks() -> u32 {
        no_windows("clock::ticks")
    }
    pub fn sleep_millis(_millis: u32) {
        no_windows("clock::sleep_millis")
    }
    pub fn begin_period(_millis: u32) -> u32 {
        no_windows("clock::begin_period")
    }
    pub fn end_period(_millis: u32) {
        no_windows("clock::end_period")
    }
}
