//! Which displays the game runs at sixty frames a second on, and which it does not.
//!
//! One case a test, because the question it answers is a table: somebody with a stutter wants to know
//! whether their desktop is one of the ones that breaks, and reading the arithmetic will not tell them.
//! They can share a binary because the pacing is a value now — it was a page of statics, and one process
//! could only ever have paced one display.
//!
//! Roughly the rate and not the exact turn: the host is not a metronome. Long enough runs that the
//! allowance has finished climbing, since the climb itself costs frames.
//!
//! **What the split rows do and do not establish.** That a compositor can be timing a rate the game's
//! monitor is not, and that a flush follows the compositor whatever monitor the window is on, are measured
//! on real hardware — `scripts/compositor-probe.c`, numbers in `DONE.md`, and the game itself measured
//! there too. What these rows add is orb's own arithmetic over that.

mod pacing;

use pacing::{Display, Run, for_each_seed};

const WORK_US: i64 = 700;
/// Fifteen seconds of play: a few for the allowance to settle in and a dozen to hold it.
const FRAMES: usize = 15 * pacing::A_SECOND;

/// Sixty frames a second in every second of the run past the grace — which is what "the game runs at the
/// right speed" means to somebody playing it. The music and every timer in it are counted in the game's
/// own frames, so a second at the wrong rate is a second of the game at the wrong speed however the
/// average over the run reads.
fn assert_rate(display: Display, wanted: f64, seed: u64) {
    let mut run = Run::started(display);
    let waits = run.frames(FRAMES, WORK_US);
    run.assert_every_second_at(&waits, wanted, seed);
}

fn assert_sixty(display: Display, seed: u64) {
    assert_rate(display, 60.0, seed);
}

fn agreed(hz: u32, seed: u64) -> Display {
    let mut display = Display::agreed(hz);
    display.seed = seed;
    display
}

fn split(monitor_hz: u32, compositor_hz: u32, seed: u64) -> Display {
    let mut display = Display::split(monitor_hz, compositor_hz);
    display.seed = seed;
    display
}

// ── The displays that work: one monitor, or several the compositor agrees with ──────────────────

#[test]
fn every_rate_a_display_reports_gets_sixty_frames_a_second() {
    for hz in [60u32, 120, 144, 165, 240] {
        for_each_seed(|seed| assert_sixty(agreed(hz, seed), seed));
    }
}

/// And the NTSC-derived rates get the display's own, which is a tenth of a percent short of sixty.
///
/// `dmDisplayFrequency` is whole Hz, so a 59.94Hz display reports 59 and a 119.88Hz one reports 119.
/// `whole_multiple` reads those as one blank a frame and two, which is right — and the rate that comes
/// out is then the display's, not sixty. `DONE.md` has the real 119.88Hz case at 59.94fps, "the
/// display's own rate halved", which is the same thing.
///
/// Here the compositor really is 59 and 119 rather than 59.94 and 119.88, since those are the numbers
/// the scenario declares, so the rates to expect are 59 and 59.5.
#[test]
fn an_ntsc_rate_gets_the_displays_own_rate() {
    for (hz, wanted) in [(59u32, 59.0), (119, 59.5)] {
        for_each_seed(|seed| assert_rate(agreed(hz, seed), wanted, seed));
    }
}

#[test]
fn a_display_that_will_not_say_its_rate_is_paced_by_the_clock_at_sixty() {
    for_each_seed(|seed| {
        let mut display = Display::unknown();
        display.seed = seed;
        assert_sixty(display, seed);
    });
}

// ── A mixed-rate desktop: the compositor times one monitor and the game is on another ───────────
//
// What decides the outcome is whether the rate the *compositor* is timing has a blank where a sixtieth of
// a second falls, because the frames go on its blanks whatever the monitor says. Where it has one, the
// disagreement costs nothing however large it is.

#[test]
fn a_disagreement_the_compositor_can_still_land_a_sixtieth_on_is_harmless() {
    for_each_seed(|seed| {
        assert_sixty(split(120, 240, seed), seed);
        assert_sixty(split(240, 60, seed), seed);
    });
}

/// And the game's own rate not being a whole multiple is harmless too, because then `adopt` leaves the
/// blanks alone until the compositor agrees — the check does guard that branch.
#[test]
fn a_fractional_monitor_the_compositor_disagrees_with_falls_back_to_the_clock() {
    for_each_seed(|seed| {
        let mut run = Run::started(split(144, 120, seed));
        let waits = run.frames(FRAMES, WORK_US);
        run.assert_every_second_at(&waits, 60.0, seed);
        assert!(
            run.pacing().shown().contains("frame(s) paced by the clock"),
            "seed {seed}: {}",
            run.pacing().shown()
        );
    });
}

/// **The broken ones.** A monitor that *is* a whole multiple stays paced by the blanks whatever the
/// compositor says — `adopt` sets that from the multiple alone and the `agrees` check only guards the
/// other branch — so the frames go on the compositor's blanks while the cadence is counted in the
/// monitor's refreshes, and the rate is not sixty.
///
/// Which way it is wrong is measured on the machine: `DONE.md` has the 144 row at 71 to 72 frames a
/// second, the game running fast. What is asserted here is that none of them is sixty, which is the part
/// the arithmetic decides; the numbers themselves belong to `DONE.md`.
#[test]
fn a_whole_multiple_monitor_is_never_sixty_where_the_compositor_is_fractional() {
    for compositor_hz in [70u32, 75, 90, 100, 110, 144, 150, 165, 200] {
        for_each_seed(|seed| {
            let mut run = Run::started(split(120, compositor_hz, seed));
            let waits = run.frames(FRAMES, WORK_US);
            // Not one second of it, rather than not on average: a display the pacing has got wrong is
            // wrong the whole way through, and there is nothing for the grace to settle.
            run.assert_never_sixty(&waits, seed);
        });
    }
}
