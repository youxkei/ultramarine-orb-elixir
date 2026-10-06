//! **What a death of th11 costs the run's numbers past the life**: 0x4327d0 takes 10,000,000 off the point items'
//! value, held no lower than its start `[0x4a5768]` (0x433d10 through 0x433d29), and `0x400` off the word at
//! `0x4a5744`, held to ±`0x400` (0x432a52); and the dying state takes the shot type's most, `[0x4a5748]`, off the
//! power on its third frame, held at nought (0x4312c6 through 0x4312ee).
//!
//! Read out of the game's own code rather than off this tree, a resume carrying all three: a laid-out death that
//! left them where they were is a run whose numbers after one no e2e test could hold anything against.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The numbers the run has as it is hit: the point items' value well over its start, the word at `0x4a5744` half
/// way up its range, and power past what the death takes.
const POINT_VALUE: i32 = 50_000_000;
const THE_NUMBER_AT_4A5744: i32 = 0x200;
const POWER: i32 = 200;
const LIVES: i32 = 2;

/// And what the death leaves of them: 10,000,000 off the value, `0x400` off the word, and the fake's shot type's
/// most, 80, off the power.
const POINT_VALUE_AFTER: i32 = 40_000_000;
const THE_NUMBER_AFTER: i32 = -0x200;
const POWER_AFTER: i32 = 120;

/// The frames the hit and the dying state's first three take, and a few more: the eight a bomb takes the hit
/// back in, and the dying state's third frame inside the thirty before the player comes back.
const PAST_THE_DYING_STATES_THIRD_FRAME: u32 = 16;

/// **And an enemy's body hits the player as a bullet does**: an enemy's update tests its body against the player
/// through the circle test (0x411e48 into 0x431f70), wherever neither of its flags `0x22` is up and the frames its
/// body is off for have run out (0x411df9) — so walking into one is a hit.
#[test]
fn an_enemys_body_on_th11s_player_hits_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-a-body",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
                config.chapters = false;
            },
        );
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        // Playing, and past the frames nothing hits it for that a run's start gives it.
        game.frames_until("the player playing and hittable", 600, || {
            game.state().in_run
                && !game.state().unsettled
                && game.image().the_frames_nothing_hits_the_player_for() <= 0
        });
        game.image().a_wave_is_up(1);
        game.image()
            .the_waves_enemy_is_at(0, game.image().where_the_player_is());
        game.frame();
        assert!(
            game.state().unsettled,
            "an enemy's body on the player did not hit it",
        );
    });
}

#[test]
fn a_death_of_th11_costs_the_point_value_the_number_at_4a5744_and_the_power() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-a-death",
            Work::wandering(WORK_US),
            // With no chapters, so that nothing of orb's holds the game at the death: what is asked is the game's
            // own dying, and a retry menu would stand the frames still before its third.
            |config| {
                config.log_level = LogLevel::Verbose;
                config.chapters = false;
            },
        );
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the stage's first frames", 120, || {
            game.state().in_run && game.state().stage_frames > 0
        });
        let (score, _, _) = game.image().the_runs_numbers();
        game.image().the_run_has(score, POWER, LIVES);
        game.image()
            .the_run_also_has(POINT_VALUE, 0, THE_NUMBER_AT_4A5744, 0, 0, 0);

        game.the_player_is_hit();
        game.frames(PAST_THE_DYING_STATES_THIRD_FRAME);
        let (_, power, lives) = game.image().the_runs_numbers();
        assert_eq!(lives, LIVES - 1, "the hit did not reach a death");
        assert_eq!(
            game.image().the_point_value_and_the_number_at_4a5744(),
            (POINT_VALUE_AFTER, THE_NUMBER_AFTER),
            "the death left the point items' value or the word at 0x4a5744 where the game moves them",
        );
        assert_eq!(
            power, POWER_AFTER,
            "the dying state left the power where it takes it"
        );
    });
}
