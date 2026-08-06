//! A frame whose work does not fit the budget it was started against, and the budget finding its
//! size.
//!
//! `prepare_us` is how long before its blank a frame starts, and every microsecond of it is input
//! lag. It is measured rather than chosen, so a frame that overran has to raise it and the cadence
//! has to come right and stay right. `next_budget` is tested on its own arithmetic; this is the loop
//! doing it.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use pacing::{NEAR_SIXTY, for_each_seed, launched};

const HZ: u32 = 120;
/// More than the 4000µs the budget starts at once the compositor's share is added, so the first
/// frame cannot make the blank it was aimed at.
const WORK_US: i64 = 4_000;
/// What orb starts by allowing the compositor, before any frame has missed a blank.
const ALLOWED_AT_FIRST: i64 = 2_500;
/// How much longer a frame's own work reads back as than the game put into it.
///
/// The frame loop reads the counter seven times between its marks and the simulated one moves a tick per
/// read, so a span the game spent 4000µs in is measured at 4002. What must not drift is the span being
/// the game's work and nothing else, which is what the bound is for rather than the exact microsecond.
const READS_US: i64 = 10;

#[test]
fn a_frame_that_overran_raises_the_budget_and_the_cadence_comes_right() {
    in_its_own_process(|| {
        let mut display = Display::agreed(HZ);
        // Flat, so that what the budget is answering to is the frame's own work and not a spike.
        display.compose = orb_sim::Compose::flat(1_000);
        display.metronome = true;
        let game = launched(display, "budget", WORK_US);
        game.frames(600);
        let handovers = game.handovers();
        let counts = pacing::refreshes(&game, &handovers);

        // The first turn is three refreshes: the frame was started 4000µs before its blank, spent 4000µs of
        // that on itself, and the compositor still wanted its share — so it was shown at the refresh after
        // the one it asked for.
        assert_eq!(counts[0], 3, "{counts:?}");

        // And every turn after it is two, because the budget was raised to cover what the frame really
        // takes. One miss and no more, which is what makes a stutter a stutter rather than a rate.
        assert!(
            counts[1..].iter().all(|count| *count == 2),
            "{counts:?} — {}",
            pacing::last_said(&game)
        );

        // What it settled at: the frame's own 4000µs plus what the compositor is being given. Read off the
        // line the run reports rather than the field, because that line is what somebody debugging a
        // stutter actually has.
        pacing::until_reported(&game);
        let reported = pacing::reported(&game);
        assert!(
            (WORK_US..=WORK_US + READS_US).contains(&reported.draw_us),
            "the frame's own work reads back as {}us against the {WORK_US}us it takes — {}",
            reported.draw_us,
            pacing::last_said(&game)
        );
        assert_eq!(
            reported.compose_us,
            ALLOWED_AT_FIRST,
            "the compositor is being given {}us, and nothing here asked it for more — {}",
            reported.compose_us,
            pacing::last_said(&game)
        );
        assert_eq!(
            reported.prepare_us,
            reported.draw_us + reported.compose_us,
            "the lag is not the two times it is said to be made of — {}",
            pacing::last_said(&game)
        );

        // One frame late in six hundred, and named as one whose drawing overran rather than one the
        // compositor was short-changed on — different faults, and giving the compositor longer would not
        // have touched this one.
        assert!(
            game.log().said("1x1") && game.log().said("whose drawing overran"),
            "{}",
            pacing::last_said(&game)
        );
    });
}

/// And over a real host the rate still comes right, which is the part that matters to a run.
#[test]
fn the_rate_comes_right_over_a_real_host_too() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let mut display = Display::agreed(HZ);
            display.seed = seed;
            let game = launched(display, &format!("budget-host-{seed}"), WORK_US);
            game.frames(3_000);
            let rate = pacing::fps(&game.handovers(), 2_000);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "seed {seed}: {rate} frames a second — {}",
                pacing::last_said(&game)
            );
        });
    });
}
