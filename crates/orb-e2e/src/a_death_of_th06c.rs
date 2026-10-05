//! **A death of th06c as 0x14004a4b0 plays it.**
//!
//! The hit leaves the player in state 2 with the count at `+0x7378` put back to nothing (0x14004c7f4). For the six
//! updates of the window at `+0x7254` a bomb takes the hit back — its own callback leaving state 3 (0x1400095c0) —
//! and once that window is out the state stands with nothing taken until the count is past 0x1d: then state 1,
//! the player put back at the field's middle and 64 up from its foot, the life taken and the bombs set to the
//! run's starting count (0x14004a7bd on). Nothing of it holds the rest of the game — the byte that does
//! (`0x1403ed7d2`) is written by an enemy's script alone — and the player does not move in states 2 and 1.
//!
//! **And where there is no life to take**, the continue question: state 1 and the game manager's `+0x7bd` up,
//! which orb reads as the game paused.
//!
//! Without chapters throughout, so that nothing of orb's holds the game at the death.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The game's own numbers: the hit's state, the walk-in's, a bomb's, and where the walk-in starts — the field's
/// middle across and 64 up from its foot, the field being 384 by 448.
const HIT: u8 = 2;
const COMING_BACK: u8 = 1;
const UNTOUCHABLE: u8 = 3;
const PUT_BACK_AT: (f32, f32) = (192.0, 384.0);

/// The updates after the hit the state stands for with the life still there: the count past 0x1d is the 31st.
const STANDS_FOR: u32 = 29;
/// And the updates by which it has gone on to the walk-in.
const GONE_BY: u32 = 33;

/// The bombs the run started with, as the fake's own configuration has them, and more than that to hold
/// before the hit.
const STARTING_BOMBS: i8 = 3;
const MORE_BOMBS: i8 = 5;

fn in_a_run(name: &str) -> Box<Fake> {
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
    game.in_a_run_in_the_stage(0);
    game.frames(300);
    game
}

#[test]
fn a_death_of_th06c_takes_its_life_once_its_count_is_past_0x1d_and_holds_nothing() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-a-death");
        let (lives, _) = game.image().lives_and_bombs();
        game.hit();
        game.frame();
        assert_eq!(
            game.image().the_players_state(),
            HIT,
            "the hit did not land"
        );
        for update in 0..STANDS_FOR {
            game.frame();
            assert_eq!(
                (
                    game.image().the_players_state(),
                    game.image().lives_and_bombs().0
                ),
                (HIT, lives),
                "the death went on before its count was past 0x1d, {update} update(s) after the hit",
            );
            assert!(
                !game.state().paused && !game.image().everything_is_held_now(),
                "the death held the game",
            );
        }
        game.frames_until("the walk-in", GONE_BY - STANDS_FOR, || {
            game.image().the_players_state() == COMING_BACK
        });
        assert_eq!(
            game.image().lives_and_bombs().0,
            lives - 1,
            "the life was not taken"
        );
        assert_eq!(
            game.image().where_the_player_is(),
            PUT_BACK_AT,
            "the walk-in did not start the player where the death puts it",
        );
        assert!(
            !game.image().everything_is_held_now(),
            "the walk-in held the game"
        );
    });
}

#[test]
fn the_player_of_th06c_does_not_move_while_it_dies() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-a-death-stands");
        game.hit();
        game.frame();
        let at = game.image().where_the_player_is();
        game.keyboard().set(keys::LEFT, true);
        game.frames(10);
        game.keyboard().set(keys::LEFT, false);
        assert_eq!(
            game.image().where_the_player_is(),
            at,
            "the player moved while it was dying"
        );
    });
}

#[test]
fn a_death_of_th06c_sets_the_bombs_to_the_runs_starting_count() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-a-death-bombs");
        let (lives, _) = game.image().lives_and_bombs();
        game.image().has_lives_and_bombs(lives, MORE_BOMBS);
        game.hit();
        game.frames_until("the walk-in", GONE_BY + 1, || {
            game.image().the_players_state() == COMING_BACK
        });
        assert_eq!(
            game.image().lives_and_bombs().1,
            STARTING_BOMBS,
            "the death left the bombs other than at the run's starting count",
        );
    });
}

#[test]
fn a_bomb_in_th06cs_window_takes_the_hit_back_into_the_untouchable_state() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-a-deathbomb");
        game.hit();
        game.frame();
        game.keyboard().set(keys::X, true);
        game.frame();
        game.keyboard().set(keys::X, false);
        assert_eq!(
            game.image().the_players_state(),
            UNTOUCHABLE,
            "the bomb did not leave the player in the bomb's own state",
        );
    });
}

#[test]
fn a_death_of_th06c_with_no_life_left_puts_the_continue_question_up() {
    in_its_own_process(|| {
        let game = in_a_run("th06c-a-death-continue");
        let (_, bombs) = game.image().lives_and_bombs();
        game.image().has_lives_and_bombs(0, bombs);
        game.hit();
        game.frames_until("the continue question", GONE_BY + 1, || game.state().paused);
        assert_eq!(
            game.image().the_players_state(),
            COMING_BACK,
            "the continue question came up over another state of the player",
        );
    });
}
