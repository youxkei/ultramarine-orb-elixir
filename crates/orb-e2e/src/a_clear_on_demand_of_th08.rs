//! **`--clear` over th08: a run to the ending on nothing that can kill it.**
//!
//! What `--clear` is for is reaching a result screen without half an hour of playing well. What makes it work
//! on this game is `Th08::make_invulnerable`, called before every one of the game's own updates: the state
//! byte at `[player+0x0]` written to 2 with its count put back to nothing, so the three box tests — each of
//! them asking whether that byte is nothing first, 0x44a333 and 0x44a445 answering 1 and 0x44a900 answering 0
//! — take no life off any hazard, and the count never reaches the thirty 0x44cbf0 takes the life at.
//!
//! **And what makes these tests say anything is fire that would really land**: the laid-out stage's own
//! script aims a bullet at the player every so often, and the player stands where it was put. The last test is
//! the control: the same fire over a run without `--clear` is a death and a retry offered.
//!
//! What these are about is how the run *went* and not how long it took. The claims are no death anywhere, the
//! stages in the order a run of the first team goes through them, and the ending after them.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, in the game's own frames: past the 240 a stage begins with and the
/// first of the stage's own bullets, so that its fire reaches a player the state byte would let it.
const STAGE_FRAMES: u32 = 900;

/// The stage words a run of the first team goes through, which the stage chooser's own table gives — see the
/// fake's `A_RUNS_STAGES`, where 3 and 4 both go to 5 and the first team's is 4.
const THE_RUNS_STAGES: [i32; 6] = [0, 1, 2, 4, 5, 6];

/// How long a whole run of those needs, in frames, with room for the builds between them and the ending.
const A_WHOLE_RUN: u32 = STAGE_FRAMES * THE_RUNS_STAGES.len() as u32 * 2;

/// The line orb writes where it has run an ending out, which is what says a run reached one: a launch skips
/// an ending by default, and the frames it takes are all inside the one the ending came up on.
const THE_ENDING: &str = "ending skipped";

/// The state a hit lands in.
const PLAYING: u8 = 0;

/// A launch in a run, with `--clear`'s two halves that these are about and not its other two — `speed` at one
/// for the reason `a_clear_on_demand_of_th06c` gives.
fn clearing(name: &str, clear: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = clear;
            config.block_replay_save = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.stages_last(STAGE_FRAMES);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game
}

/// **The whole run played**, and the stages it went through in the order it went through them.
fn played_to_the_ending(game: &Fake) -> Vec<i32> {
    let mut walked = vec![game.state().stage];
    // The shot tapped all the way, which is what takes each stage's tally down.
    let mut down = false;
    for _ in 0..A_WHOLE_RUN {
        down = !down;
        game.keyboard().set(orb_sim::keys::Z, down);
        game.frame();
        let state = game.state();
        if state.playing && state.stage != *walked.last().expect("a stage to have been in") {
            walked.push(state.stage);
        }
        // **The ending is read off the log**, it being skipped inside the frame it comes up on.
        if game.log().said(THE_ENDING) {
            break;
        }
    }
    walked
}

/// A run under `--clear` reaches its ending, through every stage in order, with nothing having killed it.
#[test]
fn a_clear_on_demand_of_th08_reaches_its_ending_with_no_death_in_it() {
    in_its_own_process(|| {
        let game = clearing("th08-a-clear-on-demand", true);
        let walked = played_to_the_ending(&game);

        assert_eq!(
            walked,
            THE_RUNS_STAGES.to_vec(),
            "the run did not walk through its stages in order",
        );
        assert!(
            game.log().said(THE_ENDING),
            "the run never reached its ending: {}",
            game.state(),
        );
        assert_eq!(
            game.state().deaths,
            0,
            "something killed the player in a run nothing could hit",
        );
    });
}

/// And no frame of it was a frame the player could be hit in, which is the whole of how.
#[test]
fn a_clear_on_demand_of_th08_leaves_its_player_in_no_state_a_hit_can_land_in() {
    in_its_own_process(|| {
        let game = clearing("th08-a-clear-untouchable", true);
        for _ in 0..A_WHOLE_RUN / 2 {
            game.frame();
            assert_ne!(
                game.image().what_the_player_is_doing(),
                PLAYING,
                "the player is in the state a hit lands in on a frame of a run under --clear",
            );
            if game.log().said(THE_ENDING) {
                return;
            }
        }
    });
}

/// **And the hit it is held in costs the run nothing**: 0x44cbf0 runs a hit's own count at `[player+0xe2a68]`
/// down first, 15 time orbs an update while it does (0x44cc10) and the miss's losses on the update it runs out
/// (0x44cd29 onward) — so a player held in the hit with that count left where states 0 and 3 hold it is a run
/// bled of its time orbs and its power.
#[test]
fn a_clear_on_demand_of_th08_costs_its_run_no_time_orbs_and_no_power() {
    in_its_own_process(|| {
        let game = clearing("th08-a-clear-costs-nothing", true);
        game.frames_until("the stage being played", A_WHOLE_RUN, || {
            game.state().playing
        });
        game.image()
            .the_stage_has_its_time_orbs(TIME_ORBS, TIME_ORBS_WANTED);
        // The count where a stage's own build leaves it (0x44d9a9), this run having been placed rather than
        // built.
        game.image()
            .the_hits_own_count_is(game.image().what_the_hits_own_count_is_held_at());
        let power = game.state().power;
        game.frames(A_HELD_SPELL);
        assert!(
            game.image().the_time_orbs_picked_up() >= TIME_ORBS,
            "the run under --clear lost time orbs: {} of {TIME_ORBS} left",
            game.image().the_time_orbs_picked_up(),
        );
        assert!(
            game.state().power >= power,
            "the run under --clear lost power: {} from {power}",
            game.state().power,
        );
    });
}

/// The time orbs that run holds and wants, and the updates it is watched over — more than any hit's own count.
const TIME_ORBS: u32 = 3000;
const TIME_ORBS_WANTED: u32 = 6000;
const A_HELD_SPELL: u32 = 120;

/// **The control**: the same fire over a run without `--clear` is a death, and the mode offers the chapter
/// again for it.
#[test]
fn a_run_of_th08_without_the_clear_is_killed_by_the_fire_the_clear_turns_away() {
    in_its_own_process(|| {
        let game = clearing("th08-a-clear-the-control", false);
        let log = game.log();
        game.frames_until("the death", STAGE_FRAMES, || game.state().deaths > 0);
        assert_eq!(
            game.state().deaths,
            1,
            "the fire a clear turns away did not kill a run without one",
        );
        game.frames_until("the death read as one", 60, || log.said("died in chapter"));
    });
}
