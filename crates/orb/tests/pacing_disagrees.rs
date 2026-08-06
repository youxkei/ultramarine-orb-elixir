//! The game's window on one monitor while the compositor times another.
//!
//! *What the fixed stutter costs* in `TODO.md` named this case and said of it that "that refusal has
//! not been seen happen", and that the hazard beside it is one "nothing has re-checked since". It was
//! both: the refusal never fired, and what happened instead ran the game **fast** — 71 to 72 frames a
//! second on the machine (`DONE.md`), because the frames went on the compositor's blanks while the
//! cadence was counted in the monitor's. Music and every timer in the game are counted in its own
//! frames, so a run like that is a run at the wrong speed.
//!
//! It is not a limit and never was. `DwmFlush` returns at the compositor's blanks and at nobody
//! else's, whatever monitor the window is on — measured on a real mixed-rate desktop, see
//! `scripts/compositor-probe.c` — so that grid is the one a frame can be put on, and counting the
//! cadence in anything else is the fault. The grid is the compositor's now, and a 144Hz compositor is
//! paced the way any 144Hz display is: one frame on whichever of its blanks is nearest each sixtieth.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use pacing::{for_each_seed, launched};

/// What the monitor the window is on reports, and a whole multiple of sixty — which is what used to
/// decide the cadence and now decides nothing.
const MONITOR_HZ: u32 = 120;
/// What the compositor is timing, which is another monitor of the desktop, and not a whole multiple.
const COMPOSITOR_HZ: u32 = 144;
const WORK_US: i64 = 1_500;

/// Fifteen seconds: a few for the allowance to settle and a dozen to hold the rate through.
const FRAMES: u32 = 15 * pacing::A_SECOND as u32;

#[test]
fn a_monitor_the_compositor_is_not_timing_is_still_paced_at_sixty() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let mut display = Display::split(MONITOR_HZ, COMPOSITOR_HZ);
            display.seed = seed;
            let game = launched(display, &format!("disagrees-{seed}"), WORK_US);
            game.frames(FRAMES);

            // The grid taken is the compositor's, and named as the compositor's: 144Hz is not a
            // multiple of sixty, so it is the same fractional path a 144Hz monitor takes.
            assert!(
                game.log().said(
                    "144Hz compositor is not a multiple of 60Hz; one frame on whichever blank is nearest each sixtieth"
                ),
                "seed {seed}: {:?}",
                game.log().lines()
            );

            // And the desktop it is happening on is said once, because a frame shown on the
            // compositor's blank still has the window's own panel to reach — which is the part of this
            // that no simulator can speak for and `DONE.md` keeps.
            assert!(
                game.log().said(
                    "the window's own monitor is 120Hz, which is not what the compositor is timing"
                ),
                "seed {seed}: {:?}",
                game.log().lines()
            );

            // Sixty frames a second in every second past the grace, which is the whole of the fix: it
            // used to be 71 to 72 on the machine and 69 to 70 here.
            pacing::assert_every_second_at(&game, &game.handovers(), 60.0, seed);

            // On the blanks throughout. The old code wrote `pacing by the clock` and then paced by the
            // blanks anyway — two lines of one run contradicting each other — and neither is written
            // now: the frames are on the blanks and the log says so.
            assert!(
                !game.log().said("pacing by the clock"),
                "seed {seed}: {}",
                pacing::last_said(&game)
            );
            assert!(
                game.log().said("0 frame(s) paced by the clock"),
                "seed {seed}: {}",
                pacing::last_said(&game)
            );
        });
    });
}
