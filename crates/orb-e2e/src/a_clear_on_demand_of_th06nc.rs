//! **`--clear` over th06nc: a run to the ending on nothing that can kill it.**
//!
//! What `--clear` is for is reaching a result screen without half an hour of playing well, so it is also how
//! the ending, the records written on the way out and the screens after a run are reached at all. What makes
//! it work on this game is `Th06nc::make_invulnerable`, called before every one of the game's own updates:
//! the player's state byte is written to 3 with `0xf0` under it, and the hit at `0x14006c1b5` is gated on
//! that byte reading nought — see `the_player_a_stage_of_th06nc_starts`, which reads both off the exe.
//!
//! **And what makes these tests say anything is a hit that would really land.** `Fake::hit` leaves one
//! waiting and the laid-out game's own hit test takes it where the game takes one — inside the stage's job,
//! in the arm the player's state chooses — so a run that came out with no death came out that way because
//! something stopped it. The last test here is the control: the same waiting hit in a run without `--clear`
//! is a death and a retry offered.
//!
//! What these are about is how the run *went* and not how long it took: the wall clock is one machine's and
//! is not written down. The claims are no death anywhere, the six stages in order, and the ending after them.
//! `a_clear_on_demand_of_th06c` is the same claim over the other 64-bit build.

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, in the game's own frames: past the 241 a stage begins with, so that
/// a hit waiting inside one is a hit the player's own state would otherwise take — see
/// `the_player_a_stage_of_th06nc_starts`.
const STAGE_FRAMES: u32 = 330;

/// How many stages a run of this game plays through before its ending.
const STAGES: i32 = 6;

/// How long a whole run of six of those needs, in frames, with room for the transitions and the ending.
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES as u32 * 2;

/// The frames a hit stands before it is a death, which is the wait the control below needs — this game's own
/// deathbomb window, and a frame after it for the death to be made final.
const A_HIT_BECOMES_A_DEATH: u32 = 8 + 2;

/// The line orb writes where it has run an ending out, which is what says a run reached one: a launch skips
/// an ending by default, and the frames it takes are all inside the one the ending came up on.
const THE_ENDING: &str = "ending skipped";

/// The state a hit lands in, and how many frames of a stage of the laid-out game go by before the player is
/// in it: the one update of state 1 and the `0xf0` of state 3 behind it — see
/// `the_player_a_stage_of_th06nc_starts`, the register's count being already past state 1's thirty.
const PLAYING: u8 = 0;
const A_STAGES_FIRST_FRAMES: u32 = 1 + 0xf0;

/// A launch in a run, with `--clear`'s two halves that these are about and not its other two.
///
/// `speed` stays at one where `--clear` names sixty-four: what these count is frames of the run, and a
/// launch running sixty-four updates inside each of them counts them in sixty-fourths. `resume` stays on
/// because nothing here writes a chapter down to be offered one.
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
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game
}

/// **The whole run played, with a hit waiting in every stage of it**, and the stages it went through in the
/// order it went through them.
fn played_to_the_ending(game: &Fake) -> Vec<i32> {
    let mut walked = vec![game.state().stage];
    let mut hit_in = game.state().stage;
    for _ in 0..A_WHOLE_RUN {
        game.frame();
        let state = game.state();
        // **The frames between two stages are not one of them**: this game's stage word reads nought while
        // the next stage is being built, so `State::stage` is -1 there — see `a_stage_transition_of_th06nc`.
        if state.playing
            && state.stage >= 0
            && state.stage != *walked.last().expect("a stage to have been in")
        {
            walked.push(state.stage);
        }
        // One hit per stage, left waiting past the frames a stage begins with so that the player's own state
        // is not what turns it away.
        if state.playing && state.stage_frames == STAGE_FRAMES / 2 && hit_in != state.stage {
            hit_in = state.stage;
            game.hit();
        }
        // **The ending is read off the log and not off `State::in_ending`**, which no frame of a test can
        // see: the ending is skipped inside the frame it comes up on — `runtime` runs the chain on until the
        // flag goes out — so what says a run reached one is the line that skip writes.
        if game.log().said(THE_ENDING) {
            break;
        }
    }
    walked
}

/// A run under `--clear` reaches its ending, through every stage in order, with nothing having killed it.
#[test]
fn a_clear_on_demand_of_th06nc_reaches_its_ending_with_no_death_in_it() {
    in_its_own_process(|| {
        let game = clearing("th06nc-a-clear-on-demand", true);
        let walked = played_to_the_ending(&game);

        assert_eq!(
            walked,
            (0..STAGES).collect::<Vec<_>>(),
            "the run did not walk through the six stages in order",
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
///
/// **The state and not the count**, because there are two states nothing can be hit in and `--clear` reaches
/// only one of them: `make_invulnerable` writes 3 over a player who is playing or already untouchable and
/// leaves state 1 alone, so the frames after a stage boundary are state 1's own. What the hit at
/// `0x14006c1b5` asks is whether the byte is nought, and the claim is that it never is.
#[test]
fn a_clear_on_demand_of_th06nc_leaves_its_player_in_no_state_a_hit_can_land_in() {
    in_its_own_process(|| {
        let game = clearing("th06nc-a-clear-untouchable", true);
        // Past the frames a stage begins with, in which the player is untouchable anyway: what is under test
        // is the frames after them, where a run nobody was holding this for would be killable.
        game.frames_until("the frames a stage begins with", STAGE_FRAMES, || {
            game.state().stage_frames > A_STAGES_FIRST_FRAMES
        });
        for _ in 0..A_WHOLE_RUN / 2 {
            game.frame();
            let state = game.image().the_players_state();
            assert_ne!(
                state, PLAYING,
                "the player is in the state a hit lands in on a frame of a run under --clear",
            );
        }
    });
}

/// **The control**: the same hit waiting in a run without `--clear` is a death, and the mode offers the
/// chapter again for it.
#[test]
fn a_run_of_th06nc_without_the_clear_is_killed_by_the_hit_the_clear_turns_away() {
    in_its_own_process(|| {
        let game = clearing("th06nc-a-clear-the-control", false);
        let log = game.log();
        game.frames_until("the frames a stage begins with", STAGE_FRAMES, || {
            game.state().stage_frames > A_STAGES_FIRST_FRAMES
        });
        game.hit();
        game.frames_until("the death", A_HIT_BECOMES_A_DEATH + 2, || {
            game.state().deaths > 0
        });
        assert_eq!(
            game.state().deaths,
            1,
            "the hit that a clear turns away did not kill a run without one",
        );
        game.frames_until("the death read as one", A_HIT_BECOMES_A_DEATH + 2, || {
            log.said("died in chapter")
        });
    });
}
