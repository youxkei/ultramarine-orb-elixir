//! **What a stage transition of th11 carries, and what only the start of a run puts in place.**
//!
//! The conversation after a stage's boss wants scene 12 (0x41eda3), and the switch answers it by taking the game
//! object down with the run's flag `0x2` set, the stage's managers kept for the next build (0x41fe73), and building
//! the game object again on its loader thread (0x42b269 through 0x42b295) — with `+0x564` at nought (0x42b279), so
//! that **what only a fresh run's build writes, the run's numbers put in place** (0x41f932 through 0x41fb01), is not
//! written, and the stage after one won keeps the score, the power, the lives and the point items' value it reached:
//! a run is six stages with one set of numbers walked through them.
//!
//! **And the stage before fades out under the new one's first thirty frames** (`0x800` of the game object's flags,
//! 0x420424 and 0x42043a), which are none of the new stage's: its frames count again from nought once the fade is
//! over (0x4205b1).

use crate::fake::th11::{FIRST_STAGE, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run walks into, orb's own 0-based number.
const THE_NEXT_STAGE: i32 = FIRST_STAGE + 1;

/// The frames a stage's build and the walk into it are given.
const A_TRANSITION: u32 = 600;

/// **The numbers this run is carrying when it crosses the boundary**, none of them a fresh run's: a life spent,
/// a power part way up, a score, and the point items' value past what a run starts with.
const SCORE: u32 = 4_653_489;
const POWER: i32 = 72;
const LIVES: i32 = 1;
const POINT_VALUE: i32 = 31_250;

/// **A run in its first stage, placed rather than walked to**, with nothing fired at the player: what this is
/// about is the build between two stages.
fn in_its_first_stage(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.frames_until("the first stage", 120, || {
        game.state().in_run && game.state().stage_frames > 0
    });
    game
}

/// The run walked into the stage after the one it was in, and stopping once that stage's own frames have begun.
fn walks_into_the_next_stage(game: &Fake) {
    game.moves_on_to_the_next_stage();
    game.frames_until("the stage after the first, begun", A_TRANSITION, || {
        let state = game.state();
        state.stage == THE_NEXT_STAGE && state.stage_frames > 0
    });
}

/// A run carries its score, its power, its lives and its point items' value across a stage boundary.
#[test]
fn a_run_of_th11_carries_its_numbers_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th11-a-transition-the-numbers");
        game.image().the_run_has(SCORE, POWER, LIVES);
        game.image().the_run_also_has(POINT_VALUE, 0, 0, 0, 0, 0);
        walks_into_the_next_stage(&game);

        let (score, power, lives) = game.image().the_runs_numbers();
        let (point_value, _) = game.image().the_point_value_and_the_number_at_4a5744();
        assert_eq!(
            (score, power, lives, point_value),
            (SCORE, POWER, LIVES, POINT_VALUE),
            "the stage the run walked into was handed numbers of its own",
        );
    });
}

/// A transition is not the run ending: every frame of it reads as a run in progress.
#[test]
fn a_stage_transition_of_th11_is_not_the_run_ending() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th11-a-transition-not-an-ending");
        game.moves_on_to_the_next_stage();
        for _ in 0..A_TRANSITION {
            game.frame();
            let state = game.state();
            assert!(
                state.in_run,
                "a frame of the transition reads as no run at all: {state}",
            );
            if state.stage == THE_NEXT_STAGE && state.stage_frames > 0 {
                return;
            }
        }
        panic!("the run never walked into the stage after the first");
    });
}

/// And the stage the run walked into gets a chapter of its own once the stage before has faded out, which is
/// what makes a transition a place the mode can go back to.
#[test]
fn the_stage_of_th11_a_run_walks_into_is_given_a_chapter_of_its_own() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th11-a-transition-the-chapter");
        let log = game.log();
        walks_into_the_next_stage(&game);
        game.frames_until("the stage's own first chapter", A_TRANSITION, || {
            log.said(&format!(
                "stage {} chapter 1 (stage start)",
                THE_NEXT_STAGE + 1
            ))
        });
        assert_eq!(
            game.state().stage,
            THE_NEXT_STAGE,
            "the chapter was taken somewhere other than the stage the run walked into",
        );
    });
}
