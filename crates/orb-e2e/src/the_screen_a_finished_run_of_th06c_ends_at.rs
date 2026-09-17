//! **The screen a run of th06c ends at is the game's to walk, and orb writes one state into it.**
//!
//! That screen is several one after another, off the jump table at `0x140053538`: the name entry a played
//! run opens in (state 9, `0x14004fd59`), the stats the name's decide opens (15, which takes a decide of its
//! own once up ninety frames), the thirty frames those fade for (16), the question about saving a replay
//! (10) and that question with its cursor live (14), and the way out (17).
//!
//! **Only the question is orb's business.** A Pointdevice run has nothing the *game* could put in a replay —
//! its record is orb's own file — so `Th06c::skip_replay_prompt` writes the way out over that one state and
//! over no other, which is the frame the game would have asked on. The name entry and the stats before it
//! belong to whoever just finished the run.
//!
//! **Which is why the write is at the question and not where the screen comes up.** A write on the frame the
//! screen is built would cover the name entry, and a name entry sent away is a run's own score with nobody's
//! name on it: th06 1.02h lost both that screen and the stats screen inside two frames to a ranking asked
//! for on the wrong one, which is `the_screen_a_finished_run_ends_at.rs`'s own subject.

use orb_config::LogLevel;
use orb_core::game::th06c::image::{
    RESULT_LEAVING, RESULT_SAVE_REPLAY_QUESTION, RESULT_STATS, RESULT_STATS_FADING,
    RESULT_TYPING_A_NAME,
};
use orb_sim::keys;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, in the game's own frames, and how many frames a whole run of six
/// needs: `--clear` is how a result screen is reached at all in this mode, so the shape is
/// `a_clear_on_demand_of_th06c`'s.
const STAGE_FRAMES: u32 = 330;
const STAGES: u32 = 6;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES * 2;

/// How many frames the screens of a finished run are given to be walked: the stats alone ignore a press for
/// ninety of them and the fade is thirty more.
const THE_SCREENS: u32 = 600;

/// The line orb writes where it has written the question away.
const NO_REPLAY: &str = "result: no replay of the game's own is offered for a run with chapters";

/// A run in Pointdevice mode played to its ending under `--clear`, and the screen it ends at after it.
fn a_cleared_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = true;
            config.block_replay_save = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game.frames_until("the screen the run ends at", A_WHOLE_RUN, || {
        game.image().the_state_the_screen_a_run_ends_at_is_in() != 0
    });
    game
}

/// **Every state that screen came to, in the order it came to them**, with the decide tapped every other
/// frame until the screen is on its way out.
fn walked(game: &Fake) -> Vec<i32> {
    let mut seen = vec![game.image().the_state_the_screen_a_run_ends_at_is_in()];
    let mut down = false;
    for _ in 0..THE_SCREENS {
        if *seen.last().expect("a state") == RESULT_LEAVING {
            break;
        }
        down = !down;
        game.keyboard().set(keys::Z, down);
        game.frame();
        let state = game.image().the_state_the_screen_a_run_ends_at_is_in();
        if state != *seen.last().expect("a state") {
            seen.push(state);
        }
    }
    game.keyboard().set(keys::Z, false);
    seen
}

/// A finished run's screen opens in the name entry, which is the state the game builds it in.
#[test]
fn the_screen_a_finished_run_of_th06c_ends_at_opens_in_its_name_entry() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-screen-a-run-ends-at");
        assert_eq!(
            game.image().the_state_the_screen_a_run_ends_at_is_in(),
            RESULT_TYPING_A_NAME,
            "the screen the run ended at did not open in the name entry",
        );
        // And nothing of orb's has been written into it yet, which is what the write's own frame is about.
        assert!(
            !game.log().said(NO_REPLAY),
            "the question was written away on the frame the screen came up, which covers the name entry",
        );
    });
}

/// And the screens after it are the game's own, walked by presses, with orb's one write at the question.
#[test]
fn the_screens_of_a_finished_run_of_th06c_are_walked_to_the_question_orb_writes_away() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-screens-a-run-ends-at");
        let seen = walked(&game);

        assert_eq!(
            seen,
            vec![
                RESULT_TYPING_A_NAME,
                RESULT_STATS,
                RESULT_STATS_FADING,
                RESULT_LEAVING
            ],
            "the screens of a finished run did not walk the name entry, the stats and their fade into the \
             way out",
        );
        // **And the question never stood for a frame of its own**, which is where orb's one write lands: it
        // is made on the frame the game put that state there, so no frame of the screen is ever the
        // question's — the read is `Th06c::skip_replay_prompt`'s own gate.
        assert!(
            !seen.contains(&RESULT_SAVE_REPLAY_QUESTION),
            "the question about saving a replay stood for a frame of its own: {seen:?}",
        );
        assert!(
            game.log().said(NO_REPLAY),
            "the question about saving a replay was not written away:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And that state is said in the log whenever it changes, which is how a walk of those screens is read back
/// off a launch of the real game.
#[test]
fn the_state_the_screen_a_finished_run_of_th06c_ends_at_is_in_is_said_as_it_changes() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06c-the-screen-said");
        walked(&game);
        let said: Vec<String> = game
            .log()
            .lines()
            .into_iter()
            .filter(|line| line.contains("result: the screen a run ends at"))
            .collect();
        assert!(
            said.len() >= 2,
            "the screen's own states were not said as they changed: {said:?}",
        );
    });
}
