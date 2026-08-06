//! A display nothing will say the rate of, which is the one case paced by the clock.
//!
//! No compositor and no reported rate. The frames then have no blank to be put on, so the cadence is
//! kept by sleeping to a grid — which is the path `sleep_until` and `wait_by_clock` exist for and
//! which the blank path never reaches.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use orb_core::profile;
use pacing::launched;

const WORK_US: i64 = 1_500;
/// Past a reporting period, so that orb has written its own account of the run and a frame after it has
/// drained the line: what the pacing says about itself waits for the slack on the far side of a flush,
/// which is `log_deferral.rs`'s subject and the reason this is not exactly a period.
const FRAMES: u32 = profile::INTERVAL + 1;

#[test]
fn a_display_that_will_not_say_its_rate_is_paced_by_the_clock() {
    in_its_own_process(|| {
        let game = launched(Display::unknown(), "by-the-clock", WORK_US);
        game.frames(FRAMES);
        let handovers = game.handovers();

        assert!(
            game.log()
                .said("an unknown monitor and the compositor will not say; pacing by the clock"),
            "{:?}",
            game.log().lines()
        );

        // Sixty frames a second, held by the clock rather than by the blanks. Not to the tick: the wait
        // ends by spinning, so it overshoots by however long one look at the counter takes.
        let rate = pacing::fps(&handovers, 0);
        assert!((rate - 60.0).abs() < 0.5, "{rate} frames a second");

        // And said to be the clock's, which is what stops somebody reading the gaps above as blanks:
        // every frame of the period, since there was never a blank for one of them to have missed.
        //
        // One more than the period's frames, and both counts are right. The line is written from inside
        // a frame's own update, so that frame has taken its turn by the clock and has not been handed
        // over yet: it is counted here and will be counted among the next period's frames.
        let reported = pacing::reported(&game);
        assert!(
            game.log().said(&format!(
                "{} frame(s) paced by the clock",
                reported.frames + 1
            )),
            "{} frames reported — {}",
            reported.frames,
            pacing::last_said(&game)
        );
    });
}
