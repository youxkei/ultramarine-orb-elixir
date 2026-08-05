//! The game's window on one monitor while the compositor times another.
//!
//! *What the fixed stutter costs* in `TODO.md` named this case and said of it that "that refusal has
//! not been seen happen", and that the hazard beside it is one "nothing has re-checked since". It
//! wanted two monitors of different rates and the window on the wrong one. Here it is a line.
//!
//! What this pins is what orb *does*, not what it should do — see the notes on the assertions.
//!
//! The numbers a real machine gives for this are in `DONE.md`; the direction is what is asserted here.

mod pacing;

use pacing::{Display, Run, for_each_seed};

/// What the monitor the window is on reports, and a whole multiple of sixty.
const MONITOR_HZ: u32 = 120;
/// What the compositor is timing, which is another monitor of the desktop.
const COMPOSITOR_HZ: u32 = 144;
const WORK_US: i64 = 1_500;

const FRAMES: usize = 400;
const WARM_UP: usize = 100;

#[test]
fn a_monitor_the_compositor_is_not_timing_is_not_sixty_and_says_the_wrong_reason() {
    for_each_seed(|seed| {
        let mut display = Display::split(MONITOR_HZ, COMPOSITOR_HZ);
        display.seed = seed;
        let mut run = Run::started(display);
        let waits = run.frames(FRAMES, WORK_US);

        // The refusal itself. `same_rate` is asked of the compositor's spacing against the monitor's
        // reported rate, and 144Hz spacing reads back as 143 whole Hz — which is not 120 by any
        // tolerance, so the two are known not to be describing the same display.
        assert!(
            run.lines()
                .iter()
                .any(|line| line.contains("120Hz monitor but the compositor is timing")),
            "seed {seed}: {:?}",
            run.lines()
        );

        // And what it actually does, which is the hazard rather than the refusal.
        //
        // A rate that *is* a whole multiple stays paced by the blanks: `adopt` sets that from the multiple
        // alone, and the `agrees` check only guards the branch for a rate that is not one. So the frames go
        // on the compositor's blanks while the cadence is counted in the monitor's refreshes, and the rate
        // is not sixty.
        //
        // **Which way it is wrong is measured on the machine, not here**: `DONE.md` records 13894-14090µs a
        // frame, 71 to 72 frames a second — the game runs *fast*. This simulator gives 69 to 70 over the
        // same declared display, and an earlier version of it that modelled the host as a metronome gave 48,
        // which was an artifact of that and nothing about orb. So what is asserted below is the direction
        // and not the number: the rate is not sixty, and it is faster rather than slower.
        let rate = run.fps(&waits, WARM_UP);
        assert!(
            (rate - 60.0).abs() > pacing::NEAR_SIXTY,
            "seed {seed}: {rate} frames a second, which is sixty — {}",
            run.pacing().report()
        );
        assert!(
            rate > 65.0,
            "seed {seed}: {rate} frames a second, where the machine gives 71 to 72 — {}",
            run.pacing().report()
        );

        // And the log names the wrong mechanism while it does. Measured on the machine too: the same run
        // says `pacing by the clock` and then `0 frame(s) paced by the clock`.
        let shown = run.pacing().shown();
        assert!(
            shown.contains("0 frame(s) paced by the clock"),
            "seed {seed}: the line above says the clock paced it — {shown}"
        );
    });
}
