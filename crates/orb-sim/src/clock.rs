//! A clock a test moves itself.
//!
//! Nothing here waits. A test that wants a frame to have taken eleven milliseconds says so, which
//! is the only way the numbers a run is judged by can be written down in the test that asserts on
//! them — a real clock would make every such assertion a measurement of the machine it ran on.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

/// Ticks a second, matching what Windows reports on every machine since Windows 8: the counter is
/// in 100ns units there, so a test that works out a duration in microseconds is working it out the
/// way the real one does.
pub const FREQUENCY: i64 = 10_000_000;

/// What one read of the counter costs, in ticks.
///
/// One tick, which is the smallest step the counter has. A real `QueryPerformanceCounter` takes
/// twenty or thirty nanoseconds against this counter's hundred, so consecutive real reads often
/// answer the same value and time passes a few times slower in a real spin than in this one. What
/// matters is that it passes.
///
/// It used to be the whole of what carried the spin to its deadline, and it is not any more:
/// [`PAUSE_TICKS`] is, at a hundred times the step. So this is here because a real read costs time and
/// no longer because the frame loop hangs without it — a claim nothing now proves, since the spin would
/// still arrive on the `pause` alone.
const READ_TICKS: i64 = 1;

/// What one turn of the spin costs, in ticks.
///
/// A real `pause` is twenty-odd cycles — a hundredth of a microsecond — and this is a hundred times
/// that, deliberately. What it buys is the suite's time: `frame::SPIN_US` is 1500µs, so at a faithful
/// cost the spin is fifteen thousand turns of a real loop per simulated frame, and `orb-e2e`'s `pacing`
/// was 76.6 seconds of almost nothing else.
///
/// What it sells is how precisely a simulated frame lands on its deadline, and only that: the loop reads
/// the counter each turn and stops once past the deadline, so a frame lands somewhere in
/// `[0, PAUSE_TICKS)` past it rather than within a tick. One microsecond against the 100µs wake delay a
/// flush's return already carries, and against a real landing that `scripts/spin-probe.c` measured at a
/// median of 0.0µs and a worst of 80.3µs over 600 frames.
///
/// **It is the coarsest step that leaves every scenario's answer unchanged, and the next one up does
/// not.** Measured: at 10 ticks `pacing` is 17.5 seconds and the suite 30.3, all 297
/// passing, where they were 76.6 and 95.0. At 100 the file is 7.1 seconds and
/// `holds::the_whole_multiple_with_no_room_on_a_restless_desktop` fails — seed 3, a second at 46.83
/// frames against a bound of 47 — which is the scenario with a 250ms stage load on a display whose
/// compositor has no room, so the one with the least of it to give. That failure is the reason this is
/// not larger; whoever wants the suite faster should read it before raising this, and the answer is not
/// to loosen the bound it crossed.
///
/// Confined to the spin because `pause` appears nowhere else in orb, which is what makes it a different
/// knob from [`set_read_cost`](Clock::set_read_cost): that one charges *every* counter read, including
/// the ones inside a span the game is measuring, and `frame::budget`'s allowance for those is derived
/// from a read costing one tick. See
/// [docs/adr/0007](../../../docs/adr/0007-the-spins-pause-is-behind-the-seam.md).
const PAUSE_TICKS: i64 = 10;

pub struct Clock {
    counter: AtomicI64,
    /// What a read costs. [`READ_TICKS`] unless a test has said otherwise — nothing that spins may
    /// set it to zero.
    read_cost: AtomicI64,
    /// Whether the high-resolution timer every wait is made on can be created at all. A host that
    /// cannot is one orb does not run on, and the whole of what happens then is a scenario's to
    /// drive — see [`refuse_the_timer`](Self::refuse_the_timer).
    timer: AtomicBool,
    /// What one turn of the spin costs. [`PAUSE_TICKS`] unless a scenario has said otherwise —
    /// nothing that spins may set it to zero.
    pause_cost: AtomicI64,
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
            timer: AtomicBool::new(true),
            pause_cost: AtomicI64::new(PAUSE_TICKS),
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

    /// Milliseconds since the host started, which is what every log line is stamped with: derived
    /// from the same counter so that the two cannot disagree about how long something took.
    ///
    /// The same arithmetic `orb_api::clock::ticks` does above the seam, and here for a scenario that
    /// wants to know what a stamp should read. It used to be a divergence — the real host's stamp was
    /// `GetTickCount`, which this machine advances by 15 or 16ms — and
    /// [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md) is
    /// where that was measured and closed.
    pub fn ticks(&self) -> u32 {
        (self.counter() / (FREQUENCY / 1_000)) as u32
    }

    /// One turn of the spin that finishes a wait, which moves the clock on by what a `pause` costs.
    ///
    /// The spin has no other way to reach its deadline — a read costs a tick and a `pause` is what the
    /// loop otherwise spends its time on — so this is what decides how many real iterations a simulated
    /// frame's spin takes. See [`PAUSE_TICKS`].
    pub fn spin_once(&self) {
        self.advance(self.pause_cost.load(Ordering::Relaxed));
    }

    /// What one turn of the spin costs, for a scenario that wants a finer landing than [`PAUSE_TICKS`]
    /// gives — at fifteen thousand turns a frame if it asks for a tick.
    pub fn set_pause_cost(&self, ticks: i64) {
        self.pause_cost.store(ticks, Ordering::Relaxed);
    }

    /// Makes the high-resolution timer refuse to be created, which is a host below Windows 10 1803
    /// and a host orb does not run on: there is no second wait to fall back to, so what orb does is
    /// say so and stop.
    pub fn refuse_the_timer(&self) {
        self.timer.store(false, Ordering::Relaxed);
    }

    /// Waits `ticks` out, and whether the host could.
    ///
    /// Exactly as long as it was asked for. A real wait overshoots by whatever the host's wake delay
    /// is — measured in `scripts/wait-probe.c` as up to 1.2ms, which is what `SPIN_US` covers — and
    /// modelling that here would make every assertion about a wait a statement about the overshoot
    /// instead of about the pacing's arithmetic.
    pub fn wait(&self, ticks: i64) -> bool {
        if !self.timer.load(Ordering::Relaxed) {
            return false;
        }
        self.advance(ticks);
        true
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
