//! The game's frame loop, as `orb/lib.rs` composes it, for the pacing scenarios to drive.
//!
//! The order here is that function's order and the marks are its marks. A harness that waited and
//! handed over in some order of its own would be measuring itself.

// Compiled into each test binary that uses it, so every one sees the helpers the others needed.
#![allow(dead_code)]

use orb_api::Hwnd;
use orb_core::frame::{self, Pacing};
use orb_sim::{Clock, Compose, Sim};
use std::sync::Arc;

/// A display to pace against, as a scenario declares it.
pub struct Display {
    /// What the monitor the window is on reports, in whole Hz. `None` is one that will not say.
    pub monitor_hz: Option<u32>,
    /// What the compositor is timing, which need not be the same monitor. `None` is no compositor.
    pub compositor_hz: Option<u32>,
    /// What the compositor takes over a frame.
    pub compose: Compose,
    /// Which stream of wake delays the host has. Named in a failure so the run can be replayed.
    pub seed: u64,
    /// Whether to take the host's non-determinism away, leaving a metronome. Only for a scenario
    /// making a claim about arithmetic — no machine is one.
    pub metronome: bool,
}

impl Display {
    /// One display, with the compositor timing it — what almost every machine has.
    pub fn agreed(hz: u32) -> Self {
        Self {
            monitor_hz: Some(hz),
            compositor_hz: Some(hz),
            compose: Compose::measured(),
            seed: 0,
            metronome: false,
        }
    }

    /// The game's window on one monitor while the compositor times another.
    pub fn split(monitor_hz: u32, compositor_hz: u32) -> Self {
        Self {
            monitor_hz: Some(monitor_hz),
            compositor_hz: Some(compositor_hz),
            compose: Compose::measured(),
            seed: 0,
            metronome: false,
        }
    }

    /// Nothing will say the rate, which is the one case paced by the clock.
    pub fn unknown() -> Self {
        Self {
            monitor_hz: None,
            compositor_hz: None,
            compose: Compose::measured(),
            seed: 0,
            metronome: false,
        }
    }
}

pub struct Run {
    // Declared first so it is dropped first: the sim has to still be in front while the run is
    // being taken down.
    _installed: orb_api::Installed,
    sim: Arc<Sim>,
    window: Hwnd,
    pacing: Pacing,
}

impl Run {
    /// Lays the display out, opens the log so that what the pacing says about it can be read, and
    /// starts the loop as the DLL does — `configure` once, before any frame.
    pub fn started(display: Display) -> Self {
        let sim = Arc::new(Sim::seeded(display.seed));
        let installed = sim.enter();
        let window = Hwnd(1);
        sim.display().set_monitor_hz(display.monitor_hz);
        sim.display().set_desktop_hz(display.monitor_hz);
        sim.display().set_foreground(window);
        if let Some(hz) = display.compositor_hz {
            sim.display().attach_compositor(
                sim.clock().peek(),
                sim.clock().frequency() / i64::from(hz),
                display.compose,
            );
        }

        if display.metronome {
            sim.display().as_a_metronome();
        }
        orb_core::log::open();
        let mut pacing = Pacing::new();
        pacing.configure();
        Self {
            _installed: installed,
            sim,
            window,
            pacing,
        }
    }

    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }

    pub fn pacing(&mut self) -> &mut Pacing {
        &mut self.pacing
    }

    /// Runs `count` frames whose own work takes `work_us`, and answers the tick each frame's wait
    /// ended at — which is the frame's turn beginning, and what the cadence is read off.
    pub fn frames(&mut self, count: usize, work_us: i64) -> Vec<i64> {
        (0..count).map(|_| self.frame(work_us)).collect()
    }

    /// One turn: the work before the wait, the wait, the game's update and draw, the hand-over.
    pub fn frame(&mut self, work_us: i64) -> i64 {
        let started = frame::now();
        // `prepare_frame` — the viewport and the clear. Cheap, and it is before the wait.
        self.sim.clock().advance_micros(200);
        let cleared = frame::now();

        self.pacing.wait_for_slot(self.window);
        let waited = frame::now();

        // The game's update, its sounds and the draw. One span, because what matters to the pacing
        // is how long the frame took between its turn and being handed over.
        self.sim.clock().advance_micros(work_us);
        let ran = frame::now();
        let sounded = frame::now();
        let drawn = frame::now();

        // `Present`, which queues the frame and returns: from here it is the compositor's, and the
        // next flush waits for it to be composed.
        self.sim.presented();
        self.pacing.finished(frame::Marks {
            started,
            cleared,
            waited,
            updated: ran,
            sounded,
            drawn,
            presented: frame::now(),
        });
        waited
    }

    /// The turns in microseconds.
    pub fn turns(&self, waits: &[i64]) -> Vec<i64> {
        waits
            .iter()
            .zip(waits.iter().skip(1))
            .map(|(before, after)| Clock::micros_for_ticks(after - before))
            .collect()
    }

    /// The turns as counts of the compositor's refreshes, rounded.
    pub fn refreshes(&self, waits: &[i64]) -> Vec<i64> {
        let period = self.sim.display().compositor_period().unwrap();
        waits
            .iter()
            .zip(waits.iter().skip(1))
            .map(|(before, after)| (after - before + period / 2) / period)
            .collect()
    }

    /// The rate the run came out at, over the turns after `warm_up`.
    pub fn fps(&self, waits: &[i64], warm_up: usize) -> f64 {
        let turns = self.turns(waits);
        let settled = &turns[warm_up..];
        1_000_000.0 / (settled.iter().sum::<i64>() as f64 / settled.len() as f64)
    }

    pub fn said(&self, needle: &str) -> bool {
        self.sim.log().said(needle)
    }

    pub fn lines(&self) -> Vec<String> {
        self.sim.log().lines()
    }
}

/// How near sixty every second of a run has to come, in frames a second.
///
/// Three, not a tenth, and judged a second at a time rather than over the run — which is what somebody
/// playing has. A turn a refresh long either way is nothing; a second at half rate is the complaint. An
/// average over three thousand frames is no use for saying so: a run that spends a second at 30 and a
/// second at 90 averages sixty and is unplayable.
///
/// Six because of what a good display measures at, and because of what one lost refresh costs. The host
/// is not a metronome — it wakes the waiting thread when it gets round to it, and its compositor is slow
/// now and then — so a second here and there loses a refresh. At 60Hz a refresh is a sixtieth, so *one*
/// lost in a second of sixty frames takes that second from 60 to 59.0, and a handful takes it to 55.3,
/// which is the worst measured over four hosts and the rates a display reports. With the compositor's
/// cost held flat instead, every second comes out within 0.08.
///
/// So the band is what the modelled host does, and it is nowhere near the rates a defect produces: 30
/// on a latched allowance, 48, 52, 71. What it does not distinguish is a second that lost a refresh
/// from one that did not — `shown()`'s own count is what says that, and the scenarios read it where it
/// matters.
pub const NEAR_SIXTY: f64 = 6.0;

/// How near sixty a second has to be to *count* as sixty, in frames a second.
///
/// Tight, unlike [`NEAR_SIXTY`], because this is not a tolerance — it is the question "was this second
/// sixty frames a second or was it not". Half a frame either way is the game running at its own speed;
/// anything more is a second that lost a refresh or gained one. What the scenarios then assert is the
/// *proportion* of seconds that were, which is the shape the answer wants: a run is not "60fps ± 6", it
/// is 60fps for so much of its length.
pub const AT_SIXTY: f64 = 0.5;

/// How many hosts a scenario is held against.
///
/// One is not enough: the wake delays are drawn, so a run is one of many the machine could have given.
/// A scenario that holds for one seed and not another has found something a real machine can do, which
/// is a defect and not a flake — so every assertion names the seed it failed on.
pub const SEEDS: u64 = 4;

/// Runs `body` against each host in turn.
pub fn for_each_seed(mut body: impl FnMut(u64)) {
    for seed in 0..SEEDS {
        body(seed);
    }
}

/// A second of play, in frames. The unit the rate is judged in, because it is the unit somebody playing
/// notices: one turn a refresh long either way is nothing, and a second at half rate is the complaint.
pub const A_SECOND: usize = 60;

/// How long the rate is given to settle, in seconds.
///
/// The allowance starts at 2500µs and climbs a hundred microseconds a miss, so a display whose
/// compositor wants more than that spends its first seconds below rate — that is the climb working, not
/// a fault. What would be a fault is it still being below after a few seconds of play, which is when
/// somebody has reached the title screen and would notice.
pub const GRACE_SECONDS: usize = 3;

impl Run {
    /// The rate over each second of the run from `warm_up` on, so that a stretch at the wrong rate cannot
    /// be averaged out by a stretch at the right one.
    ///
    /// An average over the whole run is not what a player has: a run that spends a second at 30 and a
    /// second at 90 averages sixty and is unplayable.
    pub fn rate_each_second(&self, waits: &[i64], warm_up: usize) -> Vec<f64> {
        self.turns(waits)[warm_up..]
            .chunks(A_SECOND)
            .filter(|second| second.len() == A_SECOND)
            .map(|second| 1_000_000.0 / (second.iter().sum::<i64>() as f64 / second.len() as f64))
            .collect()
    }

    /// What fraction of the seconds from `warm_up` on were sixty frames a second, by [`AT_SIXTY`].
    pub fn share_at_sixty(&self, waits: &[i64], warm_up: usize) -> f64 {
        let seconds = self.rate_each_second(waits, warm_up);
        let at = seconds
            .iter()
            .filter(|rate| (**rate - 60.0).abs() < AT_SIXTY)
            .count();
        at as f64 / seconds.len() as f64
    }

    /// The second that came out furthest from sixty, and how far.
    pub fn worst_second(&self, waits: &[i64], warm_up: usize) -> (usize, f64) {
        self.rate_each_second(waits, warm_up)
            .into_iter()
            .enumerate()
            .fold((0, 60.0), |worst, (at, rate)| {
                if (rate - 60.0).abs() > (worst.1 - 60.0).abs() {
                    (at, rate)
                } else {
                    worst
                }
            })
    }

    /// Every second after [`GRACE_SECONDS`] ran at `wanted` frames a second, by [`AT_SIXTY`].
    ///
    /// `wanted` is sixty for every display that has a blank on a sixtieth of a second, which is every
    /// whole multiple and every fractional rate the grid can chase. It is *not* sixty for the
    /// NTSC-derived ones: a display reporting 59 gets one blank a frame and runs at its own rate, which
    /// `DONE.md` records for the real 119.88Hz case — "59.94fps, the display's own rate halved". That is
    /// the pacing working; a tenth of a percent slow is a clock nobody can see.
    ///
    /// All of the seconds, not most: measured over four hosts and thirty seconds apiece, every display
    /// the pacing accepts holds every second — which is what `DONE.md`'s real runs show too, `gaps in
    /// refreshes 2x600` with none lost. So a shortfall here is a finding rather than a flake, and the
    /// seed in the message is how to go back to it.
    pub fn assert_every_second_at(&mut self, waits: &[i64], wanted: f64, seed: u64) {
        let warm_up = GRACE_SECONDS * A_SECOND;
        let seconds = self.rate_each_second(waits, warm_up);
        assert!(
            !seconds.is_empty(),
            "a run this short says nothing after {GRACE_SECONDS}s of grace"
        );
        let astray: Vec<(usize, f64)> = seconds
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, rate)| (rate - wanted).abs() >= AT_SIXTY)
            .collect();
        assert!(
            astray.is_empty(),
            "seed {seed}: {} of {} seconds were not {wanted} frames a second — {astray:?}\n  {}\n  {}",
            astray.len(),
            seconds.len(),
            self.pacing().report(),
            self.pacing().shown()
        );
    }

    /// Most of the seconds were sixty, and the run as a whole was.
    ///
    /// For a scenario where something is *meant* to cost a frame its blank now and then: a spike does
    /// that by definition, and the second it lands in comes out a frame short. What must not happen is
    /// the run losing the rate — a dip is a dip, and a stretch of them is a slow game.
    ///
    /// Four fifths, measured: a compositor spiking about one frame in three hundred put one second of
    /// eleven off sixty. The share and the rate together are what say the dips are dips.
    pub fn assert_mostly_sixty(&mut self, waits: &[i64], seed: u64) {
        let warm_up = GRACE_SECONDS * A_SECOND;
        let share = self.share_at_sixty(waits, warm_up);
        let over_the_run = self.fps(waits, warm_up);
        assert!(
            share >= 0.8 && (over_the_run - 60.0).abs() < AT_SIXTY,
            "seed {seed}: {:.0}% of the seconds were sixty and the run came out at {over_the_run} — \
             {}\n  {}",
            share * 100.0,
            self.pacing().report(),
            self.pacing().shown()
        );
    }

    /// And the other way: not one second of it was.
    pub fn assert_never_sixty(&mut self, waits: &[i64], seed: u64) {
        let warm_up = GRACE_SECONDS * A_SECOND;
        let share = self.share_at_sixty(waits, warm_up);
        assert_eq!(
            share,
            0.0,
            "seed {seed}: {:.0}% of the seconds came out at sixty — {}",
            share * 100.0,
            self.pacing().report()
        );
    }

    /// Sixty frames a second, reached inside [`GRACE_SECONDS`] and held for the rest of the run.
    ///
    /// Two halves, because a run has two failures: never getting there, and getting there and losing it.
    /// The grace is what the allowance's climb is allowed — it starts at 2500µs and rises a hundred
    /// microseconds a miss, so a display whose compositor wants more than that spends its first seconds
    /// below rate — and after it there is nothing left to settle.
    ///
    /// # Panics
    /// Naming the second that was wrong, the seed that produced it, and what the pacing reported, which
    /// together are enough to replay it.
    pub fn assert_settles_at_sixty(&mut self, waits: &[i64], seed: u64) {
        let seconds = self.rate_each_second(waits, 0);
        assert!(
            seconds.len() > GRACE_SECONDS,
            "a run of {} second(s) says nothing about settling inside {GRACE_SECONDS}",
            seconds.len()
        );
        let astray: Vec<(usize, f64)> = seconds
            .into_iter()
            .enumerate()
            .skip(GRACE_SECONDS)
            .filter(|(_, rate)| (rate - 60.0).abs() >= NEAR_SIXTY)
            .collect();
        assert!(
            astray.is_empty(),
            "seed {seed}: after {GRACE_SECONDS}s the rate was still astray at {astray:?} — {}\n  {}",
            self.pacing().report(),
            self.pacing().shown()
        );
    }

    /// Sixty frames a second held from `warm_up` on, for a scenario that has its own reason to wait.
    pub fn assert_holds_sixty(&mut self, waits: &[i64], warm_up: usize, seed: u64) {
        let (at, rate) = self.worst_second(waits, warm_up);
        assert!(
            (rate - 60.0).abs() < NEAR_SIXTY,
            "seed {seed}: second {at} of the run came out at {rate} frames a second — {}\n  {}",
            self.pacing().report(),
            self.pacing().shown()
        );
    }

    /// And the other way: the rate is not sixty and does not settle there, which is what a display the
    /// pacing has got wrong looks like.
    ///
    /// Judged over the run rather than second by second, unlike [`assert_settles_at_sixty`]. The band
    /// there is wide because one lost refresh costs a second several frames, and a run that is *wrong*
    /// can have the odd second wander into that width — 65.7 in one second of six was measured — without
    /// the run being anything like sixty. What says it is wrong is where it sits, not whether every
    /// second of it does.
    pub fn assert_never_settles_at_sixty(&mut self, waits: &[i64], warm_up: usize, seed: u64) {
        let seconds = self.rate_each_second(waits, warm_up);
        let over_the_run = self.fps(waits, warm_up);
        let near = seconds
            .iter()
            .filter(|rate| (**rate - 60.0).abs() < NEAR_SIXTY)
            .count();
        assert!(
            (over_the_run - 60.0).abs() >= NEAR_SIXTY && near * 2 < seconds.len(),
            "seed {seed}: {over_the_run} frames a second over the run, with {near} of {} seconds \
             inside the band — {}",
            seconds.len(),
            self.pacing().report()
        );
    }
}
