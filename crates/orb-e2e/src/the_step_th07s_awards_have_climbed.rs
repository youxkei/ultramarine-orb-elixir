//! **The step 妖々夢's awards have climbed to, and the run picked up that started from the bottom of it.**
//!
//! `[0x62f644]` is an index into the 31 dwords at 0x49ecf8 — 10, 20, 30 … 100, 200 … 1000, 2000, 3000 … 12000,
//! 12800. 0x432efe adds one to it, 0x432f19 holds it at 30, and 0x432f42 adds a tenth of what the table holds
//! there to the score. A death puts it back to nothing at 0x440d6a, and so does 0x432fb2.
//!
//! **The arithmetic is what says a resume has to carry it.** A run of 妖々夢 picked up in stage 2 reproduced
//! 978 frames of itself word for word — the same buttons, the same place, the same count of items, the same
//! word out of the generator — and then took 3 where the run it was reproducing had taken 203. That is 30
//! against 2030: a difference of exactly 2000, which is entry 19 of that table. The recording carried the step
//! its first stage had climbed to and the run put back started from the bottom.
//!
//! Nothing of that shows in a reproduction line and climbing the step draws no words out of the generator, so
//! two runs stay in step for as long as it takes an award to differ — which is why a stage 1 resume, whose step
//! is at the bottom either way, reproduced all 3479 of its frames.
//!
//! **The game's own replay restore carries it**, 0x4432f1 writing it into a stage's record and 0x443865 putting
//! it back.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The line a stage's own start is written down with, which is the first point of it a run reaches: see
/// `a_stage_of_th07_is_written_down_as_it_begins`.
const THE_STAGES_START: &str = "stage 2 chapter 1 (MIDSTAGE 1) at frame";

/// The frames a stage's settle and its music are given to leave the stage with a chapter running.
const SETTLES: u32 = 600;

/// The step the run below has climbed to, which is the one whose award is 2000 — the entry the launch that
/// found this was 2000 points short of.
const STEP: u8 = 19;

#[test]
fn a_run_of_th07_is_put_back_at_the_step_its_awards_had_climbed_to() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-step-awards-climbed",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The step as a run a stage into itself has climbed it, and **then** the stage after it: what a stage
        // starts with is read where that stage starts.
        game.image().the_awards_have_climbed_to(STEP);
        game.moves_on_to_the_stage(1);

        game.frames_until_a_thread("the stage's start written down", || {
            log.said(THE_STAGES_START)
        });
        // And its first chapter begun, because the run below is given up by dying and a death is announced in a
        // chapter: the write above lands in the settle before the stage has one.
        game.frames_until("the stage's own first chapter", SETTLES, || {
            log.said("stage 2 chapter 1 (stage start)")
        });

        // And the run picked up again, whose own build leaves the step at the bottom: what stands there
        // afterwards is the recorded run's only if the resume put it back.
        game.gives_the_run_up();
        game.picks_the_run_up_at(1);
        assert_eq!(
            game.image().the_step_the_awards_have_climbed(),
            STEP,
            "the run came back with its awards at another step:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
