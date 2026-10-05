//! **th07's rank, and the run picked up that got a build's own.**
//!
//! `[0x62f8a4]` is `[gm+0x9634]`, the rank. A run's build fills it at 0x42e3a4, with its floor and its ceiling,
//! out of the table by difficulty at `[0x49f5d0 + difficulty*0xc]`, and the block that does so is the one
//! 0x42e89b jumps over — so a stage the run *walked into* carries whatever the stage before it left there, and
//! a stage built as a run's first gets the table's starting figure.
//!
//! **It is not a number that stays put while a run is played.** 0x42f4aa raises it for every 100 its
//! accumulator gathers and 0x42f526 lowers it the same way — items, grazes, extends and bombs — both storing
//! through `[ecx+0x9634]`, so no absolute store in the exe mentions it. A launch measured the difference that
//! leaves: the same stage 2 entered by a transition held 0x20 where the same stage 2 built by a resume held
//! 0x10.
//!
//! **And the enemies' fire is scaled by it.** The ECL runner reads it at 0x412ad4 and 0x412b3c for the bullet
//! counts an enemy holds, and at 0x412bab and 0x412c04, with `fimul`, for their speeds, each as
//! `low + (high - low) * rank / 32` added to what the instruction asked for. A higher rank is more bullets and
//! faster ones, and a few hundred frames of that is a stage the run did not play.
//!
//! **The game's own replay restore carries it**, 0x4432c9 writing it into a stage's record as `+0x25` and
//! 0x4437ea putting it back — one instruction after 0x4437e0 puts back the point items, which orb has always
//! carried. So the set orb writes was the record's set less this one number.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The line a stage the run walked into is written down at, which is where this run is interrupted.
const THE_STAGES_START: &str = "stage 2 chapter 1 (MIDSTAGE 1) at frame";

/// The frames a stage's settle and its music are given to leave the stage with a chapter running.
const SETTLES: u32 = 600;

/// The rank the run below is carrying, and not the one a build leaves: what the launch that found this
/// measured was 0x20 against a build's 0x10, so the two have to be told apart and neither can be nothing.
const CARRIED: u32 = 0x20;

#[test]
fn a_run_of_th07_is_put_back_with_the_rank_it_was_carrying() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-rank",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The rank as a run a stage into itself carries it, and **then** the stage after it: a stage's numbers
        // are read where that stage starts, so written into a stage already running they would be numbers no
        // chapter of it was written down with. The transition is what carries this one, a build filling it.
        game.image().the_rank_is(CARRIED);
        game.moves_on_to_the_stage(1);
        assert_eq!(
            game.image().the_rank(),
            CARRIED,
            "the transition filled the rank in, so this test is about nothing",
        );

        game.frames_until_a_thread("the stage's start written down", || {
            log.said(THE_STAGES_START)
        });
        game.frames_until("the stage's own first chapter", SETTLES, || {
            log.said("stage 2 chapter 1 (stage start)")
        });

        // And the run picked up again, whose own build fills the rank out of the table by difficulty: what
        // stands there afterwards is the recorded run's only if the resume put it back.
        game.gives_the_run_up();
        game.picks_the_run_up_at(1);
        assert_eq!(
            game.image().the_rank(),
            CARRIED,
            "the run came back with a build's own rank in place of the one it was carrying:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
