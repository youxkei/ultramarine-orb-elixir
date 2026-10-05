//! **The continue th06nc offers a run with no life left**, as 0x14000bc20 walks it: practice, a replay, the
//! Extra and a run that has continued three times already offered none; otherwise the question up on its
//! second item, the one that gives the run up, deaf for its first thirty updates; up or down onto the first,
//! deaf for four; a decide on the second giving the run up, and on the first the run going on thirty updates
//! later with the lives and the bombs it started with and the menu down.
//!
//! Without chapters throughout, so that nothing of orb's stands in the way.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

const NORMAL: i32 = 1;
const EXTRA: i32 = 4;

/// The updates the question's second item and its first are deaf for, and the ones a continue takes.
const THE_QUESTION_HEARS_NOTHING_FOR: u32 = 0x1e;
const THE_FIRST_ITEM_HEARS_NOTHING_FOR: u32 = 4;
const A_CONTINUE_TAKES: u32 = 0x1e;

/// A run with no life left, hit and waited out to the question: the window, the death's own thirty and a
/// few frames over.
fn out_of_lives(name: &str, difficulty: i32, practice: bool) -> Box<Fake> {
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
    game.image().the_difficulty_is(difficulty);
    game.image().the_item_asks_for_a_practice(practice);
    game.in_a_run_in_the_stage(0);
    game.frames(300);
    let (_, bombs) = game.image().lives_and_bombs();
    game.image().has_lives_and_bombs(0, bombs);
    game.hit();
    game
}

fn gave_the_run_up(game: &Fake) -> bool {
    game.image().the_scene_now() != game.image().scene_of_a_run()
        || game.image().the_scene_wanted() == game.image().scene_of_the_runs_own_end()
}

#[test]
fn th06ncs_continue_question_takes_no_decide_for_its_first_thirty_updates_and_then_gives_the_run_up()
 {
    in_its_own_process(|| {
        let game = out_of_lives("th06nc-a-continue-declined", NORMAL, false);
        game.frames_until("the continue question", 60, || {
            game.image().is_the_retry_menu_up()
        });
        game.frames(THE_QUESTION_HEARS_NOTHING_FOR - 4);
        game.press(keys::Z);
        assert!(
            game.image().is_the_retry_menu_up() && !gave_the_run_up(&game),
            "a decide on the question's first updates was taken",
        );
        game.frames(4);
        game.press(keys::Z);
        game.frames_until("the run given up", 60, || gave_the_run_up(&game));
    });
}

#[test]
fn th06ncs_continue_question_goes_on_with_the_runs_own_lives_on_its_first_item() {
    in_its_own_process(|| {
        let game = out_of_lives("th06nc-a-continue-taken", NORMAL, false);
        game.frames_until("the continue question", 60, || {
            game.image().is_the_retry_menu_up()
        });
        // Past the update the question is built on and the thirty after it.
        game.frames(THE_QUESTION_HEARS_NOTHING_FOR + 1);
        game.press(keys::UP);
        game.frames(THE_FIRST_ITEM_HEARS_NOTHING_FOR);
        game.press(keys::Z);
        game.frames_until("the run going on", A_CONTINUE_TAKES + 4, || {
            !game.image().is_the_retry_menu_up()
        });
        assert!(!gave_the_run_up(&game), "the continue gave the run up");
        assert_eq!(
            game.image().lives_and_bombs(),
            game.image().the_configuration_now(),
            "the run did not go on with the lives and the bombs it started with",
        );
    });
}

#[test]
fn a_practice_run_of_th06nc_with_no_life_left_is_offered_no_continue() {
    in_its_own_process(|| {
        let game = out_of_lives("th06nc-a-practice-no-continue", NORMAL, true);
        game.frames_until("the run given up", 60, || gave_the_run_up(&game));
    });
}

#[test]
fn the_extra_of_th06nc_with_no_life_left_is_offered_no_continue() {
    in_its_own_process(|| {
        let game = out_of_lives("th06nc-the-extra-no-continue", EXTRA, false);
        game.frames_until("the run given up", 60, || gave_the_run_up(&game));
    });
}
