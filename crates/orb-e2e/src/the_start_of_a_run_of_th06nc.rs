//! **What a run of th06nc starts with**, as 0x14004f320 writes it: the lives and the bombs out of the
//! configuration — two and three for the Extra and for practice alike (`0x14004f37c`), and two lives with the
//! configuration's bombs in a Challenge run (`0x14004f373`) — the run's mode out of the configuration's (the
//! `setne` at `0x14004f348`), and the scene's byte put down.
//!
//! Without chapters throughout, so that the numbers are the game's and none of orb's.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A configuration other than the run's two and three, so that which a run took is plain.
const CONFIGURED: (i8, i8) = (5, 1);
const TWO_AND_THREE: (i8, i8) = (2, 3);
const A_CHALLENGE_RUNS_LIVES: i8 = 2;

const NORMAL: i32 = 1;

fn a_launch(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.chapters = false;
        },
    );
    game.fires_at_the_player(false);
    game.image()
        .the_configuration_gives(CONFIGURED.0, CONFIGURED.1);
    game.image().the_difficulty_is(NORMAL);
    game
}

#[test]
fn a_run_of_th06nc_starts_with_the_lives_and_the_bombs_the_configuration_gives() {
    in_its_own_process(|| {
        let game = a_launch("th06nc-a-run-starts-configured");
        game.in_a_run_in_the_stage(0);
        assert_eq!(game.image().lives_and_bombs(), CONFIGURED);
    });
}

#[test]
fn a_practice_run_of_th06nc_starts_with_two_lives_and_three_bombs_whatever_the_configuration() {
    in_its_own_process(|| {
        let game = a_launch("th06nc-a-practice-run-starts");
        game.image().the_item_asks_for_a_practice(true);
        game.in_a_run_in_the_stage(1);
        assert_eq!(game.image().lives_and_bombs(), TWO_AND_THREE);
    });
}

#[test]
fn a_challenge_run_of_th06nc_starts_in_its_mode_with_two_lives_and_the_configured_bombs() {
    in_its_own_process(|| {
        let game = a_launch("th06nc-a-challenge-run-starts");
        game.image().the_mode_last_chosen_is_challenge(true);
        game.in_a_run_in_the_stage(0);
        assert!(
            game.image().is_the_run_in_challenge_mode(),
            "the run did not take its mode from the configuration",
        );
        assert_eq!(
            game.image().lives_and_bombs(),
            (A_CHALLENGE_RUNS_LIVES, CONFIGURED.1)
        );
    });
}

#[test]
fn a_run_of_th06nc_starts_with_the_scenes_byte_down() {
    in_its_own_process(|| {
        let game = a_launch("th06nc-a-run-puts-the-scene-down");
        game.image().a_scene_is_being_played(true);
        game.in_a_run_in_the_stage(0);
        assert!(!game.image().is_a_scene_being_played());
    });
}
