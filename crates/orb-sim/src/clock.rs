//! A clock a test moves itself.
//!
//! Nothing here waits. A test that wants a frame to have taken eleven milliseconds says so, which
//! is the only way the numbers a run is judged by can be written down in the test that asserts on
//! them — a real clock would make every such assertion a measurement of the machine it ran on.

use std::sync::atomic::{AtomicI64, Ordering};

/// Ticks a second, matching what Windows reports on every machine since Windows 8: the counter is
/// in 100ns units there, so a test that works out a duration in microseconds is working it out the
/// way the real one does.
pub const FREQUENCY: i64 = 10_000_000;

/// What one read of the counter costs, in ticks.
///
/// Not a convenience — without it the pacing does not terminate. `sleep_until` sleeps most of the
/// way to its deadline and *spins* the last 1500µs, and a spin is a loop whose only host call is the
/// counter read: a clock that moved only when something asked it to wait would never reach the
/// deadline and the frame loop would hang. A real one does not, because reading the counter takes
/// time.
///
/// One tick, which is the smallest step the counter has. A real `QueryPerformanceCounter` takes
/// twenty or thirty nanoseconds against this counter's hundred, so consecutive real reads often
/// answer the same value and time passes a few times slower in a real spin than in this one. What
/// matters is that it passes.
const READ_TICKS: i64 = 1;

pub struct Clock {
    counter: AtomicI64,
    /// What a read costs. [`READ_TICKS`] unless a test has said otherwise — nothing that spins may
    /// set it to zero.
    read_cost: AtomicI64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    pub fn new() -> Self {
        // Not zero. A run beginning at tick zero is one where "has this been set yet" and "was
        // this at the very start" are the same reading, and orb has statics that mean the first.
        Self {
            counter: AtomicI64::new(FREQUENCY),
            read_cost: AtomicI64::new(READ_TICKS),
        }
    }

    /// Reads the counter, and moves it on by what the read cost.
    ///
    /// The value answered is the one before the cost, so two reads in a row differ by exactly it.
    pub fn counter(&self) -> i64 {
        let cost = self.read_cost.load(Ordering::Relaxed);
        self.counter.fetch_add(cost, Ordering::Relaxed)
    }

    /// The counter without moving it, for a test reading back what a run came out as.
    pub fn peek(&self) -> i64 {
        self.counter.load(Ordering::Relaxed)
    }

    /// What a read of the counter costs, in ticks. Zero makes the clock stand still under reads,
    /// which anything that spins will hang on.
    pub fn set_read_cost(&self, ticks: i64) {
        self.read_cost.store(ticks, Ordering::Relaxed);
    }

    pub fn frequency(&self) -> i64 {
        FREQUENCY
    }

    /// Milliseconds since the host started, as `GetTickCount` reports them: derived from the same
    /// counter so that the two cannot disagree about how long something took.
    pub fn ticks(&self) -> u32 {
        (self.counter() / (FREQUENCY / 1_000)) as u32
    }

    /// Moves the clock on by `micros`, which is what a test does instead of sleeping.
    pub fn advance_micros(&self, micros: i64) {
        self.advance(micros * (FREQUENCY / 1_000_000));
    }

    /// Moves the clock on by `ticks`.
    pub fn advance(&self, ticks: i64) {
        self.counter.fetch_add(ticks, Ordering::Relaxed);
    }

    /// Moves the clock to `counter`, or leaves it where it is if that is not ahead.
    ///
    /// What a wait does: a `Sleep` or a `DwmFlush` puts the clock where the waiting ended, and
    /// neither can end in the past.
    pub fn advance_to(&self, counter: i64) {
        self.counter.fetch_max(counter, Ordering::Relaxed);
    }

    /// Ticks for a span in microseconds, and back, so a test can say a duration in the units it
    /// thinks in and compare against what orb measured.
    pub fn ticks_for_micros(micros: i64) -> i64 {
        micros * (FREQUENCY / 1_000_000)
    }

    pub fn micros_for_ticks(ticks: i64) -> i64 {
        ticks / (FREQUENCY / 1_000_000)
    }
}
