//! **The number 妖々夢 scales its shot by, and the run picked up that got a build's own.**
//!
//! `[0x62f8a4]` is `[gm+0x9634]`. A run's build fills it at 0x42e3a4 out of the per-shot table at
//! `[0x49f5d0 + shot*0xc]`, and the block that does so is the one 0x42e89b jumps over — so a stage the run
//! *walked into* carries whatever the stage before it left there, and a stage built as a run's first gets the
//! table's own figure.
//!
//! **It is not a number that stays put while a run is played.** 0x40ecf1 hands its *address* out of the
//! dispatcher that gives a script a variable to write — the entry beside `[0x626280]`, the difficulty — so what
//! writes it is the stage's own script and no absolute store in the exe mentions it. A launch measured the
//! difference that leaves: the same 2面 entered by a transition held 0x20 where the same 2面 built by a resume
//! held 0x10.
//!
//! **And it is read every frame the player is updated.** Nine sites in 0x40e and 0x412 read it, four with
//! `fimul` — 0x412bab is `current + (target - current) * this / 0x498a84` written back into the field it came
//! from, which is a rate. Twice the rate is a world where the player's own numbers walk at twice the speed, and
//! a few hundred frames of that is a world that collects items the run did not.
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

/// The number the run below is carrying, and not the one a build leaves: what the launch that found this
/// measured was 0x20 against a build's 0x10, so the two have to be told apart and neither can be nothing.
const SCALED_BY: u32 = 0x20;

#[test]
fn a_run_of_th07_is_put_back_with_the_number_its_shot_is_scaled_by() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-number-the-shot-is-scaled-by",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The number as a run a stage into itself carries it, and **then** the stage after it: a stage's numbers
        // are read where that stage starts, so written into a stage already running they would be numbers no
        // chapter of it was written down with. The transition is what carries this one, a build filling it.
        game.image().the_shots_own_number_is(SCALED_BY);
        game.moves_on_to_the_stage(1);
        assert_eq!(
            game.image().the_shots_own_number(),
            SCALED_BY,
            "the transition filled the number in, so this test is about nothing",
        );

        game.frames_until_a_thread("the stage's start written down", || {
            log.said(THE_STAGES_START)
        });
        game.frames_until("the stage's own first chapter", SETTLES, || {
            log.said("stage 2 chapter 1 (stage start)")
        });

        // And the run picked up again, whose own build fills the number out of the per-shot table: what stands
        // there afterwards is the recorded run's only if the resume put it back.
        game.gives_the_run_up();
        game.picks_the_run_up_at(1);
        assert_eq!(
            game.image().the_shots_own_number(),
            SCALED_BY,
            "the run came back with a build's own number in place of the one it was carrying:\n  {}",
            log.lines().join("\n  "),
        );
    });
}
