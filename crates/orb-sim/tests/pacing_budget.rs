//! A frame whose work does not fit the budget it was started against, and the budget finding its
//! size.
//!
//! `prepare_us` is how long before its blank a frame starts, and every microsecond of it is input
//! lag. It is measured rather than chosen, so a frame that overran has to raise it and the cadence
//! has to come right and stay right. `next_budget` is tested on its own arithmetic; this is the loop
//! doing it.

mod pacing;

use pacing::{Display, NEAR_SIXTY, Run, for_each_seed};

const HZ: u32 = 120;
/// More than the 4000µs the budget starts at once the compositor's share is added, so the first
/// frame cannot make the blank it was aimed at.
const WORK_US: i64 = 4_000;

#[test]
fn a_frame_that_overran_raises_the_budget_and_the_cadence_comes_right() {
    let mut display = Display::agreed(HZ);
    // Flat, so that what the budget is answering to is the frame's own work and not a spike.
    display.compose = orb_sim::Compose::flat(1_000);
    display.metronome = true;
    let mut run = Run::started(display);
    let waits = run.frames(600, WORK_US);
    let counts = run.refreshes(&waits);

    // The first turn is three refreshes: the frame was started 4000µs before its blank, spent 4000µs of
    // that on itself, and the compositor still wanted its share — so it was shown at the refresh after
    // the one it asked for.
    assert_eq!(counts[0], 3, "{counts:?}");

    // And every turn after it is two, because the budget was raised to cover what the frame really
    // takes. One miss and no more, which is what makes a stutter a stutter rather than a rate.
    let report = run.pacing().report();
    assert!(
        counts[1..].iter().all(|count| *count == 2),
        "{counts:?} — {report}"
    );

    // What it settled at: the frame's own 4000µs plus what the compositor is being given. Read off the
    // line the run reports rather than the field, because that line is what somebody debugging a
    // stutter actually has.
    assert!(
        report.contains("4000us to draw + 2500us for the compositor"),
        "{report}"
    );

    // One frame late in six hundred, and named as one whose drawing overran rather than one the
    // compositor was short-changed on — different faults, and giving the compositor longer would not
    // have touched this one.
    let shown = run.pacing().shown();
    assert!(
        shown.contains("1x1") && shown.contains("whose drawing overran"),
        "{shown}"
    );
}

/// And over a real host the rate still comes right, which is the part that matters to a run.
#[test]
fn the_rate_comes_right_over_a_real_host_too() {
    for_each_seed(|seed| {
        let mut display = Display::agreed(HZ);
        display.seed = seed;
        let mut run = Run::started(display);
        let waits = run.frames(3_000, WORK_US);
        let rate = run.fps(&waits, 2_000);
        assert!(
            (rate - 60.0).abs() < NEAR_SIXTY,
            "seed {seed}: {rate} frames a second — {}",
            run.pacing().report()
        );
    });
}
