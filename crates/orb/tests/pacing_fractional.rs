//! A display whose rate is not a whole multiple of sixty: two refreshes, three, two, three.
//!
//! At 144Hz a frame is 2.4 refreshes, so there is no one count to keep. Each frame goes on whichever
//! blank is nearest where a sixtieth-of-a-second grid has got to.
//!
//! The pattern is asserted on a metronome host, because it is a claim about the grid's arithmetic; that
//! the rate comes out at sixty over a real one is `pacing_converges.rs`.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use orb_sim::Compose;
use pacing::{NEAR_SIXTY, for_each_seed, launched};

const HZ: u32 = 144;
const WORK_US: i64 = 700;
const FRAMES: u32 = 400;
/// The grid takes a few frames to come up, and the budget a few more.
const WARM_UP: usize = 100;

#[test]
fn the_grid_alternates_two_refreshes_and_three_in_the_pattern_that_averages_two_point_four() {
    in_its_own_process(|| {
        let mut display = Display::agreed(HZ);
        display.metronome = true;
        display.compose = Compose::flat(1_000);
        let game = launched(display, "fractional-grid", WORK_US);
        game.frames(FRAMES);

        assert!(
            game.log().said(
                "144Hz compositor is not a multiple of 60Hz; one frame on whichever blank is nearest each sixtieth"
            ),
            "{:?}",
            game.log().lines()
        );

        let counts = pacing::refreshes(&game, &game.handovers());
        let settled = &counts[WARM_UP..];
        assert!(
            settled.iter().all(|count| *count == 2 || *count == 3),
            "{settled:?}"
        );

        // Twelve refreshes to every five frames, wherever the five are taken from. Being true of *every*
        // window rather than of the average is the self-correcting part: the grid is a moment and not an
        // accumulated count, so a frame put on the nearer blank does not push the ones after it.
        for (at, window) in settled.windows(5).enumerate() {
            assert_eq!(
                window.iter().sum::<i64>(),
                12,
                "five frames from {at} took {window:?} refreshes"
            );
        }
    });
}

/// And over a real host it is still sixty frames a second, and still only twos and threes — a turn of
/// one or four would be a frame the grid never asked for, whatever a late wake did to it.
#[test]
fn a_late_wake_leaves_the_rate_at_sixty_and_the_counts_where_the_grid_put_them() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let mut display = Display::agreed(HZ);
            display.seed = seed;
            let game = launched(display, &format!("fractional-host-{seed}"), WORK_US);
            game.frames(3_000);
            let handovers = game.handovers();
            let rate = pacing::fps(&handovers, 2_000);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "seed {seed}: {rate} frames a second — {}",
                pacing::last_said(&game)
            );
            let counts = pacing::refreshes(&game, &handovers);
            let settled = &counts[2_000..];
            assert!(
                settled.iter().all(|count| (2..=3).contains(count)),
                "seed {seed}: {settled:?} — {}",
                pacing::last_said(&game)
            );
        });
    });
}
