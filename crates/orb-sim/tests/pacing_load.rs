//! A stage load, and what the allowance must not do about it.
//!
//! A load is a frame that takes a quarter of a second, and the frames around it miss their blanks
//! because of it. The compositor had nothing to do with that, so none of it may reach the allowance —
//! `measure_compose`'s own comment records what happens when it does: a run "ratchets to seven
//! milliseconds of lag by the third stage", which is a quarter of a frame of input lag bought for
//! nothing and never given back.
//!
//! So three loads here rather than one, since the third stage is where it was noticed.

mod pacing;

use orb_sim::Compose;
use pacing::{Display, Run, for_each_seed};

const HZ: u32 = 120;
const WORK_US: i64 = 700;
/// What `budget_ceiling`'s comment calls the frame that must not be allowed to set anything: a stage
/// load, a quarter of a second of it.
const LOAD_US: i64 = 250_000;
/// A compositor comfortably inside the 2500µs orb starts by allowing, so that any climb at all is the
/// load's doing and not the compositor's.
const QUICK_COMPOSE_US: i64 = 1_000;

#[test]
fn three_stage_loads_buy_the_compositor_nothing() {
    for_each_seed(|seed| {
        let mut display = Display::agreed(HZ);
        display.compose = Compose::flat(QUICK_COMPOSE_US);
        display.seed = seed;
        let mut run = Run::started(display);

        run.frames(2 * pacing::A_SECOND, WORK_US);
        let (_, _, before) = run.pacing().status();

        let mut waits = Vec::new();
        for _ in 0..3 {
            waits.push(run.frame(LOAD_US));
            waits.extend(run.frames(3 * pacing::A_SECOND, WORK_US));
        }

        let (_, _, allowed) = run.pacing().status();
        assert_eq!(
            allowed,
            before,
            "seed {seed}: the allowance climbed from {before}us to {allowed}us over three loads, and \
             the compositor was taking {QUICK_COMPOSE_US}us throughout — {}",
            run.pacing().shown()
        );

        // And the damage stops at the second the load is in. Measured: a load's own second reads 47.8
        // to 48.8 — sixty frames in the 1.25 seconds a quarter-second load makes of it, which is
        // arithmetic and not pacing — and every other second of the run is exactly sixty. So what is
        // asserted is that no more seconds than there were loads are off at all, and that the ones that
        // are lost the load and nothing besides.
        let off: Vec<f64> = run
            .rate_each_second(&waits, pacing::A_SECOND)
            .into_iter()
            .filter(|rate| (rate - 60.0).abs() >= pacing::AT_SIXTY)
            .collect();
        assert!(
            off.len() <= 3,
            "seed {seed}: {} seconds off sixty for three loads — {off:?} — {}",
            off.len(),
            run.pacing().report()
        );
        for rate in &off {
            assert!(
                *rate >= 47.0,
                "seed {seed}: a second at {rate} frames, where a quarter of a second lost is 48 — \
                 {off:?} — {}",
                run.pacing().report()
            );
        }
    });
}
