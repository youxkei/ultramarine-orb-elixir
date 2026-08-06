//! A display whose rate is a whole multiple of sixty: the same number of blanks to every frame.
//!
//! orb's own frame loop, driven by a game whose loop calls it — the real `configure`, `settle`,
//! `wait_for_slot` and `finished`, in the order `render` composes them, over a compositor a scenario
//! declares. What drove this before was a copy of that loop written in a harness, and nothing held the
//! copy to the original: the arithmetic was covered and the order it is asked for in was not.
//!
//! Roughly the right rate rather than exact ticks. The host wakes the waiting thread when it gets round
//! to it and its compositor is slow now and then — see `orb-sim/src/display.rs` — so a scenario that
//! asserted a turn to the microsecond would be asserting that the host is a metronome, which none is.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use pacing::{NEAR_SIXTY, for_each_seed, launched};

const HZ: u32 = 120;
/// What the game's own frame takes, read off a real run's report line: "(694us to draw + …)".
const WORK_US: i64 = 700;
/// Long enough for the allowance to have finished climbing; the climb itself costs refreshes.
const FRAMES: u32 = 3_000;
const SETTLED: usize = 2_000;

#[test]
fn a_120hz_display_gets_two_blanks_a_frame_and_sixty_frames_a_second() {
    in_its_own_process(|| {
        for_each_seed(|seed| {
            let mut display = Display::agreed(HZ);
            display.seed = seed;
            let game = launched(display, &format!("pacing-blanks-{seed}"), WORK_US);
            game.frames(FRAMES);
            let handovers = game.handovers();

            // What `settle` made of the display, which is the decision everything below rests on. The
            // compositor's own spacing, not the monitor's reported rate: that is the grid the frames go
            // on, and the two agree here.
            assert!(
                game.log()
                    .said("120Hz compositor, one frame every 2 blank(s)"),
                "seed {seed}: {:?}",
                game.log().lines()
            );

            // Every frame reached the game's own `Present`, which is what makes the ticks below a
            // reading of the whole run rather than of the part of it orb paced.
            assert_eq!(
                handovers.len(),
                FRAMES as usize,
                "seed {seed}: frames handed over — {}",
                pacing::last_said(&game)
            );

            let rate = pacing::fps(&handovers, SETTLED);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "seed {seed}: {rate} frames a second — {}",
                pacing::last_said(&game)
            );

            // Two blanks to a frame, near enough always. A real host loses one now and then to a late
            // wake or a slow composition; what must not happen is a turn the grid never asked for.
            let counts = pacing::refreshes(&game, &handovers);
            let settled = &counts[SETTLED..];
            assert!(
                settled.iter().all(|count| (2..=3).contains(count)),
                "seed {seed}: turns of {settled:?} refreshes — {}",
                pacing::last_said(&game)
            );

            // And orb's own line about the run agreeing with the ticks. Nobody debugging a stutter has
            // the ticks; this line is what they have, so a run whose rate was right and whose account of
            // it was wrong is a run orb cannot be read about.
            let reported = pacing::reported(&game);
            let said = reported.fps();
            assert!(
                (said - 60.0).abs() < NEAR_SIXTY,
                "seed {seed}: orb reported {said} frames a second over {} frames — {}",
                reported.frames,
                pacing::last_said(&game)
            );
            assert_eq!(
                reported.in_buckets(),
                reported.frames,
                "seed {seed}: gaps in refreshes account for {:?} of {} frames — {}",
                reported.gaps,
                reported.frames,
                pacing::last_said(&game)
            );
            // The buckets are the same claim the counts above are, made in refreshes: two to a frame,
            // with whatever the host lost put in the bucket above.
            assert!(
                reported
                    .gaps
                    .iter()
                    .all(|(refreshes, _)| (2..=3).contains(refreshes)),
                "seed {seed}: {:?} — {}",
                reported.gaps,
                pacing::last_said(&game)
            );
            assert_eq!(
                reported.shown_late,
                0,
                "seed {seed}: the compositor could not show {} of them when it meant to — {}",
                reported.shown_late,
                pacing::last_said(&game)
            );
        });
    });
}
