//! **The numbers a run of th07 is made with, and what a resume is allowed to do to them.**
//!
//! th07 keeps a seal over the run's own numbers and checks it: 0x4012b0 draws a word into `[run+0x3c]`, moves
//! a copy of it on in `[run+0xac]` by `[run+0xbc]` for each byte it sums and keeps the sum in `[run+0xb0]`;
//! 0x404fe0 holds the copy against the word and the sum against the one the manager's update takes again; and
//! every guarded write the game makes asks that check first and reseals after. Where it fails the caller fills 0xb4 dwords from 0x575950 with 0xffffffff — the
//! device pointer and the scene the supervisor reads are both in that range — so a run whose seal has come
//! apart is a run that ends on its next update.
//!
//! **Which is what two launches paid for.** 0x42e3da draws noise into the run's object as it is allocated,
//! `[run+0x34]`'s seven words among it, and a resume wrote a recorded run's seven back over the top of them
//! after resealing. The third of the seven is `[run+0x3c]`. Both launches were over one update into the
//! playback with the ranking read as if the run had ended; the same run with the seven left alone played 2842
//! of its frames.
//!
//! So what a resume owes the numbers is the two things here: the ones a stage of a run in progress actually
//! has, and the seal still holding over them afterwards.

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

/// **And the numbers a stage of a run in progress has that a first stage does not**: the score the panel is
/// drawing, and how many extends the point items have given with how many items the next one wants.
///
/// The score no more decides a frame of th07 than the number on the panel does, so a run resumed without the
/// panel's is a run that plays out right and is *recorded* wrong: the panel counts up from whatever the front
/// end left, and what a run is for is the number at the end of it. The extends are what the buttons do play
/// against — the point item that reaches the threshold gives a life, or a bomb where the lives are full — so a
/// run resumed with a first stage's thresholds gives its extend on another item than the run did.
#[test]
fn a_run_of_th07_is_put_back_with_the_score_it_had_and_the_extends_its_point_items_gave() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-panel-and-the-ladder",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // A run a stage into itself: the panel drawing what it has scored, and the point items five extends in
        // rather than at a first stage's thresholds. **Then the stage after it**, because the numbers a
        // stage starts with are read where that stage starts — written into the stage already running, they
        // would be numbers no chapter of it was ever written down with.
        game.image().the_panel_shows(7_654_320);
        game.image().the_point_items_have_given(5, 1_820);
        game.moves_on_to_the_stage(1);

        game.frames_until_a_thread("the stage's start written down", || {
            log.said(THE_STAGES_START)
        });
        // And its first chapter begun, because the run below is given up by dying and a death is announced in a
        // chapter: the write above lands in the settle before the stage has one.
        game.frames_until("the stage's own first chapter", SETTLES, || {
            log.said("stage 2 chapter 1 (stage start)")
        });

        game.gives_the_run_up();
        game.picks_the_run_up_at(1);
        assert_eq!(
            game.image().the_score_the_panel_shows(),
            7_654_320,
            "the panel came back showing another run's score:\n  {}",
            log.lines().join("\n  "),
        );
        assert_eq!(
            game.image().the_extends_the_point_items_have_given(),
            (5, 1_820),
            "the point items' extends came back at another stage's thresholds",
        );
    });
}

/// A run picked up comes back with the seal over its numbers holding, which is what the game checks before
/// every guarded write it makes.
#[test]
fn a_run_of_th07_is_put_back_with_the_seal_over_its_numbers_holding() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-seal-over-the-numbers",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // A stage into the run, so that every number a resume writes has something to write: a first stage's
        // are the ones a build leaves, and a resume that wrote none of them would pass this test by writing
        // nothing at all.
        game.image().the_panel_shows(7_654_320);
        game.image().the_point_items_have_given(5, 1_820);
        game.moves_on_to_the_stage(1);
        assert!(
            game.image().the_seal_holds(),
            "the run's own build left the seal apart, so this test is about nothing",
        );

        game.frames_until_a_thread("the stage's start written down", || {
            log.said(THE_STAGES_START)
        });
        // And its first chapter begun, because the run below is given up by dying and a death is announced in a
        // chapter: the write above lands in the settle before the stage has one.
        game.frames_until("the stage's own first chapter", SETTLES, || {
            log.said("stage 2 chapter 1 (stage start)")
        });

        // And the run picked up again, which is where a resume writes its numbers and reseals over them. What
        // this asks is the question the game's own check asks on the playback's first update.
        game.gives_the_run_up();
        game.picks_the_run_up_at(1);
        assert!(
            game.image().the_seal_holds(),
            "the resume wrote a number the seal was already taken over:\n  {}",
            log.lines().join("\n  "),
        );
    });
}

/// **And a number of the run's written with no reseal after it is a seal that no longer holds**, which is
/// what makes the test above about anything: the word and its copy the reseal draws are untouched by a write
/// of the lives, and what catches one is the sum 0x42d7be takes — the manager's update sums the run again
/// every update it runs through (0x42dc8b–0x42dcb8) and leaves `[run+0xb0]` as the last reseal had it, so
/// 0x404fe0's second comparison fails the next update.
#[test]
fn a_life_of_th07_written_with_no_reseal_after_it_breaks_the_seal() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-the-seal-a-write-breaks",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        assert!(
            game.image().the_seal_holds(),
            "the run's own build left the seal apart, so this test is about nothing",
        );

        game.image().the_lives_are_written_over(7.0);
        assert!(
            !game.image().the_seal_holds(),
            "a life written over after the seal still passed the game's check",
        );
    });
}
