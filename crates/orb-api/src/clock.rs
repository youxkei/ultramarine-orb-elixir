//! The host's clock: reading it, the stamp divided down from it, the wait to a frame's own deadline,
//! and the coarse one a thread nobody is waiting for takes between two reads of a device.
//!
//! No decision of the pacing's is behind the seam. Every one of them is already a function of the
//! numbers that come out — `grid_aim`, `next_budget`, `whole_multiple`, `on_cadence` — so those are
//! tested by handing them numbers, and what a seam adds is the order the waiting calls come in. What
//! matters about the waiting is whether frames land on blanks, which is a measurement of real
//! hardware rather than anything a simulated clock can answer; what it decided is beside
//! `frame::Pacing::grid`.
//!
//! The wait's own timer is made behind here and never crosses, and how accurate it is was measured
//! too — beside `frame::SPIN_US`, which is the spin that covers what it overshoots by.

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
///
/// Divided down from [`counter`] rather than read off `GetTickCount`, which follows the system timer
/// tick and is therefore fifteen milliseconds coarse unless somebody has asked the whole system for
/// better. Nothing asks any more — the frame loop's wait does not need it — and a stamp that could
/// not say when anything happened would take `--pacing`'s two-stamp readings with it. Measured: with
/// no resolution in force `GetTickCount` advances by the system timer's tick and nothing finer, and
/// the counter is exact to the millisecond either way.
///
/// Zero where the host has no counter, which is a host that cannot say when anything happened at all.
pub fn ticks() -> u32 {
    let frequency = frequency();
    if frequency == 0 {
        return 0;
    }
    (counter() / (frequency / 1_000)) as u32
}

/// Waits `ticks` of the counter out, and answers whether the host could.
///
/// `false` is the high-resolution timer refusing to be created, which is a host orb does not run on:
/// see [`Win::wait`](crate::Win::wait).
pub fn wait(ticks: i64) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.wait(ticks);
    }
    host::wait(ticks)
}

/// Waits `ms` whole milliseconds out, on a thread nothing is waiting for.
///
/// Apart from [`wait`], which is the frame's own deadline: see [`Win::sleep`](crate::Win::sleep) for
/// why the two are different calls rather than one with a unit.
pub fn sleep(ms: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.sleep(ms);
    }
    host::sleep(ms);
}

/// One turn of the spin that finishes a wait: the `pause` instruction, and nothing else in a build
/// with no simulated Windows in it.
pub fn spin_once() {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.spin_once();
    }
    host::spin_once();
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
    pub fn wait(_ticks: i64) -> bool {
        no_windows("clock::wait")
    }
    pub fn sleep(_ms: u32) {
        no_windows("clock::sleep")
    }
    pub fn spin_once() {
        no_windows("clock::spin_once")
    }
}
