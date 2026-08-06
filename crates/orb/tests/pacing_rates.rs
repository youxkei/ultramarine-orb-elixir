//! Which displays the game runs at sixty frames a second on, and which it does not.
//!
//! One case a test, because the question it answers is a table: somebody with a stutter wants to know
//! whether their desktop is one of the ones that breaks, and reading the arithmetic will not tell them.
//!
//! Roughly the rate and not the exact turn: the host is not a metronome. Long enough runs that the
//! allowance has finished climbing, since the climb itself costs frames.
//!
//! **What the split rows do and do not establish.** That a compositor can be timing a rate the game's
//! monitor is not, and that a flush follows the compositor whatever monitor the window is on, are measured
//! on real hardware — `scripts/compositor-probe.c`, numbers in `DONE.md`, and the game itself measured
//! there too. What these rows add is orb's own arithmetic over that.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use pacing::{for_each_seed, launched};

const WORK_US: i64 = 700;
/// Fifteen seconds of play: a few for the allowance to settle in and a dozen to hold it.
const FRAMES: u32 = 15 * pacing::A_SECOND as u32;

/// Sixty frames a second in every second of the run past the grace — which is what "the game runs at the
/// right speed" means to somebody playing it. The music and every timer in it are counted in the game's
/// own frames, so a second at the wrong rate is a second of the game at the wrong speed however the
/// average over the run reads.
fn assert_rate(display: Display, name: &str, wanted: f64, seed: u64) {
    let game = launched(display, name, WORK_US);
    game.frames(FRAMES);
    pacing::assert_every_second_at(&game, &game.handovers(), wanted, seed);
}

fn assert_sixty(display: Display, name: &str, seed: u64) {
    assert_rate(display, name, 60.0, seed);
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
    in_its_own_process(|| {
        for hz in [60u32, 120, 144, 165, 240] {
            for_each_seed(|seed| {
                assert_sixty(agreed(hz, seed), &format!("rate-{hz}-{seed}"), seed);
            });
        }
    });
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
    in_its_own_process(|| {
        for (hz, wanted) in [(59u32, 59.0), (119, 59.5)] {
            for_each_seed(|seed| {
                assert_rate(agreed(hz, seed), &format!("ntsc-{hz}-{seed}"), wanted, seed);
            });
        }
    });
}

#[test]
fn a_display_that_will_not_say_its_rate_is_paced_by_the_clock_at_sixty() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let mut display = Display::unknown();
            display.seed = seed;
            assert_sixty(display, &format!("unknown-{seed}"), seed);
        });
    });
}

// ── A mixed-rate desktop: the compositor times one monitor and the game is on another ───────────
//
// What decides the outcome is whether the rate the *compositor* is timing has a blank where a sixtieth of
// a second falls, because the frames go on its blanks whatever the monitor says. Where it has one, the
// disagreement costs nothing however large it is.

#[test]
fn a_disagreement_the_compositor_can_still_land_a_sixtieth_on_is_harmless() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            assert_sixty(
                split(120, 240, seed),
                &format!("split-120-240-{seed}"),
                seed,
            );
            assert_sixty(split(240, 60, seed), &format!("split-240-60-{seed}"), seed);
        });
    });
}

/// And the game's own monitor not being a whole multiple is harmless too, because the rate the cadence
/// is counted in is the compositor's either way — here a 120Hz one, which is a whole multiple.
#[test]
fn a_fractional_monitor_a_whole_multiple_compositor_is_timing_gets_sixty() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let game = launched(
                split(144, 120, seed),
                &format!("split-144-120-{seed}"),
                WORK_US,
            );
            game.frames(FRAMES);
            pacing::assert_every_second_at(&game, &game.handovers(), 60.0, seed);
            // On the blanks, and on the compositor's count of them rather than the monitor's.
            assert!(
                game.log()
                    .said("120Hz compositor, one frame every 2 blank(s)"),
                "seed {seed}: {:?}",
                game.log().lines()
            );
            assert!(
                !game.log().said("pacing by the clock"),
                "seed {seed}: {}",
                pacing::last_said(&game)
            );
        });
    });
}

/// **The ones that used to be broken.** A monitor that *is* a whole multiple used to keep the cadence in
/// its own refreshes while the frames went on the compositor's blanks, and the rate came out wrong at
/// every one of these — measured on the machine at 71 to 72 frames a second for the 144 row (`DONE.md`),
/// the game running *fast*, with music and every timer in it counted in frames that were coming too
/// quickly.
///
/// The cadence is counted in the compositor's spacing now, so each of these is the same fractional grid a
/// display of that rate would get: one frame on whichever blank is nearest each sixtieth. Every second of
/// every one of them is sixty.
#[test]
fn a_fractional_compositor_gets_sixty_whatever_the_monitor_says() {
    in_its_own_process(|| {
        for compositor_hz in [70u32, 75, 90, 100, 110, 144, 150, 165, 200] {
            for_each_seed(|seed| {
                let game = launched(
                    split(120, compositor_hz, seed),
                    &format!("was-broken-{compositor_hz}-{seed}"),
                    WORK_US,
                );
                game.frames(FRAMES);
                pacing::assert_every_second_at(&game, &game.handovers(), 60.0, seed);
            });
        }
    });
}

/// A monitor that will not say its rate, on a desktop whose compositor will.
///
/// Which is the other case the old cadence lost: with nothing to take a whole multiple of, the blanks
/// were refused and the frames were paced by the clock — while a compositor sat there timing them the
/// whole time. The rate is taken from the compositor now, so this is an ordinary 120Hz run.
#[test]
fn a_monitor_that_will_not_say_is_paced_on_the_compositors_blanks() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let mut display = Display::unknown();
            display.compositor_hz = Some(120);
            display.seed = seed;
            let game = launched(display, &format!("silent-monitor-{seed}"), WORK_US);
            game.frames(FRAMES);
            pacing::assert_every_second_at(&game, &game.handovers(), 60.0, seed);
            assert!(
                game.log()
                    .said("120Hz compositor, one frame every 2 blank(s)"),
                "seed {seed}: {:?}",
                game.log().lines()
            );
            assert!(
                game.log().said("0 frame(s) paced by the clock"),
                "seed {seed}: {}",
                pacing::last_said(&game)
            );
        });
    });
}
