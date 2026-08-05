//! A display nothing will say the rate of, which is the one case paced by the clock.
//!
//! No compositor and no reported rate. The frames then have no blank to be put on, so the cadence is
//! kept by sleeping to a grid — which is the path `sleep_until` and `wait_by_clock` exist for and
//! which the blank path never reaches.

mod pacing;

use pacing::{Display, Run};

const WORK_US: i64 = 1_500;

#[test]
fn a_display_that_will_not_say_its_rate_is_paced_by_the_clock() {
    let mut run = Run::started(Display::unknown());
    let waits = run.frames(30, WORK_US);

    assert!(
        run.said("an unknown monitor and the compositor will not say; pacing by the clock"),
        "{:?}",
        run.lines()
    );

    // Sixty frames a second, held by the clock rather than by the blanks. Not to the tick: the wait
    // ends by spinning, so it overshoots by however long one look at the counter takes.
    let fps = run.fps(&waits, 0);
    assert!((fps - 60.0).abs() < 0.5, "{fps} frames a second");

    // And said to be the clock's, which is what stops somebody reading the gaps above as blanks.
    let shown = run.pacing().shown();
    assert!(
        shown.contains(&format!("{} frame(s) paced by the clock", waits.len())),
        "{shown}"
    );
}
