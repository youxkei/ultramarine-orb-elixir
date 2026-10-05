//! **What a stage transition of th06c carries, and what only the start of a run puts in place.**
//!
//! The game manager's own added callback is `0x1400242d0`, one function reached two ways, and the whole of
//! the difference is its first condition: `cmp DWORD PTR [0x140bc28ac],0x3` at `0x1400242f9` — the
//! supervisor's state against the one a stage is rebuilt in — jumps a transition to `0x140024cf5` and lets a
//! run's first stage fall through.
//!
//! **What falls through is a run being made**: the movement area at `+0xc`..`+0x18` and the box at
//! `+0x17e4`..`+0x17f0`, the rank at `+0x1970` set to eight (`0x1400243cf`) and then, with its bounds, to the
//! table by difficulty at `0x1403e0010` (`0x140024cac` on) — sixteen — the score's own fields cleared,
//! the extra life the score has paid for, and the `0x186a0` the next one costs.
//!
//! **What the transition takes is eight instructions and nothing of the run's numbers**: `0x140024cf5` copies
//! the score into the panel's own copy of it (`+0x7b0` into `+0x7ac`), zeroes the next increment at `+0x7b4`,
//! the sub-rank at `+0x197c` (`0x140024d10`), `+0x7c4`, `+0x17d0` and `+0x7bc`, and steps the stage word at
//! `+0x20` up by one. In a full run that is not a playback, the lives, the bombs, the power and the rank are
//! written **nowhere in it** — which is what a run *is*: six stages with one set of numbers walked through
//! them. A practice run has its power written on stage 2 and after (`0x140024db1`–`0x140024dce`), and a
//! playback its rank's bounds and the extra lives' index (`0x140024e1d`–`0x140024e86`).
//!
//! **Read out of the game's own code rather than off this tree**, because the laid-out game is both the
//! writer and the reader of the memory orb is driven through: a transition that put a stage's numbers back
//! would be a transition every e2e test here agreed with.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below starts in and the one it walks into, orb's own 0-based numbers.
const THE_FIRST_STAGE: i32 = 0;
const THE_NEXT_STAGE: i32 = 1;

/// How long each stage lasts here, in the game's own frames: past the 270 a stage begins with — see
/// `the_player_a_stage_of_th06c_starts` — so that a run's numbers can be spent inside the first one, and no
/// further than it has to be.
const STAGE_FRAMES: u32 = 330;

/// The frames the walk into the next stage is given past the first stage's last.
const A_TRANSITION: u32 = 60;

/// And the frames a stage's own start is given to settle into a chapter, which is `chapter`'s own wait and
/// a second on top of it as this test's wall.
const A_SETTLE: u32 = 120;

/// **The numbers this run is carrying when it crosses the boundary**, none of them a stage's own: lives and
/// bombs spent down from what the configuration gives, a shot part-way up its power, and a score with the
/// panel's copy of it left behind.
const LIVES: i8 = 1;
const BOMBS: i8 = 1;
const POWER: u16 = 96;
const SCORE: u32 = 1_234_560;
const THE_PANEL_WAS_SHOWING: u32 = 7;
const RANK: i32 = 23;
const SUB_RANK: i32 = 7;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// A run in its first stage, with nothing fired at the player: a death in this mode is a retry offered, and
/// a question nobody answers is a game standing still — see `the_frame_a_scene_of_th06c_is_built_on`.
fn in_its_first_stage(name: &str) -> Box<Fake> {
    let game = launched(name);
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(THE_FIRST_STAGE);
    game
}

/// The run walked into the stage after the one it was in, stopping on the frame that stage was built on.
fn walks_into_the_next_stage(game: &Fake) {
    game.frames_until(
        "the stage after the first",
        STAGE_FRAMES + A_TRANSITION,
        || game.state().stage == THE_NEXT_STAGE,
    );
}

/// A run carries its lives, its bombs and its power across a stage boundary.
#[test]
fn a_run_of_th06c_carries_its_lives_bombs_and_power_across_a_stage_boundary() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-the-numbers");
        game.image().has_lives_and_bombs(LIVES, BOMBS);
        game.image().shot_is_at_power(POWER);
        walks_into_the_next_stage(&game);

        assert_eq!(
            (
                game.image().lives_and_bombs(),
                game.image().the_shots_power()
            ),
            ((LIVES, BOMBS), POWER),
            "the stage the run walked into was handed numbers of its own",
        );
    });
}

/// And its rank, where the sub-rank that was climbing towards the next step of it is put back to nothing.
#[test]
fn a_run_of_th06c_carries_its_rank_across_a_stage_boundary_and_not_the_step_towards_it() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-the-rank");
        game.image().the_rank_is(RANK, SUB_RANK);
        walks_into_the_next_stage(&game);

        assert_eq!(
            (game.image().the_rank(), game.image().the_sub_rank()),
            (RANK, 0),
            "the rank did not cross the boundary, or the step towards it did",
        );
    });
}

/// And the panel's own copy of the score is put back to the run's, which is the one thing the transition
/// does to the numbers.
#[test]
fn a_stage_transition_of_th06c_shows_the_runs_own_score_on_the_panel() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-the-panel");
        game.image().the_run_has_scored(SCORE, 0);
        game.image().the_panel_shows(THE_PANEL_WAS_SHOWING);
        walks_into_the_next_stage(&game);

        assert_eq!(
            game.image().the_score_the_panel_shows(),
            SCORE,
            "the panel kept drawing what it was drawing before the boundary",
        );
        assert_eq!(
            game.image().the_score(),
            SCORE,
            "the run's own score did not cross the boundary",
        );
    });
}

/// **And the stage the run walked into is written down with the run's score**, the score a chapter of it is
/// picked up with: the stage callback leaves the run's own field at nought, so the score that stands when it
/// returns is the panel's.
#[test]
fn the_stage_of_th06c_a_run_walks_into_is_written_down_with_the_runs_score() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-the-score-written-down");
        let log = game.log();
        game.image().the_run_has_scored(SCORE, 0);
        walks_into_the_next_stage(&game);
        game.frames_until("the stage's own first chapter", A_SETTLE, || {
            log.said(&format!(
                "stage {} chapter 1 (stage start)",
                THE_NEXT_STAGE + 1
            ))
        });
        assert!(
            log.said(&format!("Numbers {{ score: {SCORE},")),
            "the stage the run walked into was written down with a score of its own:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// A transition is not the run ending: every frame of it reads as a run in progress.
#[test]
fn a_stage_transition_of_th06c_is_not_the_run_ending() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-not-an-ending");
        // Every frame from the last of the first stage to the first of the next, which is where a reading
        // that asked only for the gameplay scene would answer that the run had gone.
        game.frames_until(
            "the first stage's last frames",
            STAGE_FRAMES + A_TRANSITION,
            || game.state().stage_frames + 2 >= STAGE_FRAMES,
        );
        for _ in 0..A_TRANSITION {
            game.frame();
            let state = game.state();
            assert!(
                state.in_run,
                "a frame of the transition reads as no run at all: {state}",
            );
        }
        assert_eq!(
            game.state().stage,
            THE_NEXT_STAGE,
            "the run never walked into the stage after the first",
        );
    });
}

/// And the stage the run walked into gets a chapter of its own, which is what makes a transition a place the
/// mode can go back to.
#[test]
fn the_stage_of_th06c_a_run_walks_into_is_given_a_chapter_of_its_own() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-the-chapter");
        let log = game.log();
        walks_into_the_next_stage(&game);
        game.frames_until("the stage's own first chapter", A_SETTLE, || {
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

/// And the score file is not read again for it: this game reads its own once, at the startup orb brackets.
#[test]
fn a_stage_transition_of_th06c_makes_no_read_of_the_score_file() {
    in_its_own_process(|| {
        let game = in_its_first_stage("th06c-a-transition-the-score-file");
        let reads = || -> usize {
            game.score_file_opens()
                .iter()
                .filter(|(_, write)| !write)
                .count()
        };
        let before = reads();
        walks_into_the_next_stage(&game);
        assert_eq!(
            reads(),
            before,
            "the stage the run walked into read the score file: {:?}",
            game.score_file_opens()
        );
    });
}
