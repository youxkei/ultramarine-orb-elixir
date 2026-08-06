//! Lines the frame loop holds back until writing one costs nothing.
//!
//! The moments between handing a frame over and the blank it is shown at are the ones a write must stay
//! out of: the next frame has about a millisecond to reach `DwmFlush`, and one that arrives after the
//! blank has gone waits out another refresh and is shown late. So what the pacing says about itself is
//! held, and written on the far side of that flush, where what is left of the turn is slack.
//!
//! **Which is a claim about where the real loop drains**, and that is why this is driven by a game whose
//! own loop calls `render`: a harness that deferred and drained in an order of its own would be saying
//! where *it* drains. The two lines below are read off the log — the moment orb stamped the line with,
//! against the ticks the game was handed its frames over at.
//!
//! One test to a file — see `log_writes.rs` in `orb-sim`, which is what covers the writing itself.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use orb_core::profile;
use pacing::launched;

/// A display with room for a frame that takes its time: at 60Hz a turn is 16.6ms, and the work below is
/// eight of them.
const HZ: u32 = 60;

/// What the game's own work takes. Far larger than any other pacing scenario asks for, and that is the
/// point: the stamps here are milliseconds, so the moment the loop drains and the moment it does the
/// frame's work have to be milliseconds apart for the log to say which of them the line was written in.
const WORK_US: i64 = 8_000;
const WORK_MS: i64 = WORK_US / 1_000;

#[test]
fn what_the_frame_loop_defers_is_written_on_the_far_side_of_the_next_flush() {
    in_its_own_process(|| {
        let game = launched(Display::agreed(HZ), "log-deferral", WORK_US);
        // Up to and including the frame whose own update works the pacing's account of the period out.
        game.frames(profile::INTERVAL);
        let over = pacing::millis_at(*game.handovers().last().expect("a frame handed over"));

        // Held back, not written: writing it where it was worked out is what would cost the next frame
        // its blank, and that frame is the one the account is about.
        assert!(
            pacing::reports(&game).is_empty(),
            "written where it was worked out:\n  {}",
            game.log().lines().join("\n  ")
        );

        // One more frame, which is the one with the slack in it.
        game.frame();
        let next = pacing::millis_at(*game.handovers().last().expect("a frame handed over"));
        let lines = game.log().lines();
        let written = lines
            .iter()
            .find(|line| line.contains(" frames, ") && line.contains("us apart"))
            .unwrap_or_else(|| panic!("nothing was drained:\n  {}", lines.join("\n  ")));
        let held = pacing::stamped_at(written);

        // After the frame it is about was handed over, which is the holding back.
        assert!(
            held > over,
            "stamped {held}ms, and the frame it is about went out at {over}ms — {written}"
        );
        // And before that next frame did its own work, which is where the slack is: the flush returned,
        // the line was written, and the turn's remaining fourteen milliseconds were still ahead. A line
        // written beside the work instead would be inside the millisecond the frame after it has to reach
        // the flush in.
        assert!(
            next - held >= WORK_MS,
            "stamped {held}ms against a frame handed over at {next}ms, which is inside its own \
             {WORK_MS}ms of work — {written}"
        );

        // And the three lines of a period are written in the order they were deferred, so that a run
        // reads as a sequence of frames rather than as whatever came out of the buffer first.
        let at = |needle: &str| {
            lines
                .iter()
                .position(|line| line.contains(needle))
                .unwrap_or_else(|| {
                    panic!("no line holds {needle:?} among\n  {}", lines.join("\n  "))
                })
        };
        assert!(at(" frames, ") < at("on screen from"));
        assert!(at("on screen from") < at("refreshes past the blank aimed at"));
    });
}
