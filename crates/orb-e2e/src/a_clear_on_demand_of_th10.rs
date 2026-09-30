//! **`--clear` over th10: a run to the ending on nothing that can kill it.**
//!
//! What `--clear` is for is reaching a result screen without half an hour of playing well. What makes it work
//! on this game is `Th10::make_invulnerable`, called before every one of the game's own updates: the player's
//! own countdown at `+0x4310` put back up to 2, which every hit and both of the player's tests ask is nought
//! before they take anything (0x426756, 0x426950) — and which the player's job counts one down a frame
//! (0x425e46), so it never reaches nought inside an update.
//!
//! **And what makes these tests say anything is fire that would really land**: the laid-out stage's own
//! script aims a bullet at the player every so often, and the player stands where it was put. The last test is
//! the control: the same fire over a run without `--clear` is a death and a retry offered.
//!
//! What these are about is how the run *went* and not how long it took. The claims are no death anywhere, the
//! six stages in order, and the ending after them.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, in the game's own frames: past the `0x78` frames the player is built
/// with nothing to hit it for (0x424d14) and the first of the stage's own bullets at 480, so that its fire
/// reaches a player the countdown would let it.
const STAGE_FRAMES: u32 = 900;

/// The stages a run goes through, as orb counts them: the six of a run, the last of which goes to the ending
/// (0x416eef).
const THE_RUNS_STAGES: [i32; 6] = [0, 1, 2, 3, 4, 5];

/// How long a whole run of those needs, in frames, with room for the builds between them and the ending.
const A_WHOLE_RUN: u32 = STAGE_FRAMES * THE_RUNS_STAGES.len() as u32 * 2;

/// The line orb writes where it has run an ending out, which is what says a run reached one: a launch skips
/// an ending by default, and the frames it takes are all inside the one the ending came up on.
const THE_ENDING: &str = "ending skipped";

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
    for _ in 0..A_WHOLE_RUN {
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
fn a_clear_on_demand_of_th10_reaches_its_ending_with_no_death_in_it() {
    in_its_own_process(|| {
        let game = clearing("th10-a-clear-on-demand", true);
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

/// And no frame of it left the player with nothing of its countdown, which is the whole of how.
#[test]
fn a_clear_on_demand_of_th10_leaves_its_player_unhittable_on_every_frame() {
    in_its_own_process(|| {
        let game = clearing("th10-a-clear-untouchable", true);
        for _ in 0..A_WHOLE_RUN / 2 {
            game.frame();
            assert!(
                game.image().the_frames_nothing_hits_the_player_for() > 0,
                "the player's countdown is at nought on a frame of a run under --clear",
            );
            if game.log().said(THE_ENDING) {
                return;
            }
        }
    });
}

/// **The control**: the same fire over a run without `--clear` is a death, and the mode offers the chapter
/// again for it.
#[test]
fn a_run_of_th10_without_the_clear_is_killed_by_the_fire_the_clear_turns_away() {
    in_its_own_process(|| {
        let game = clearing("th10-a-clear-the-control", false);
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
