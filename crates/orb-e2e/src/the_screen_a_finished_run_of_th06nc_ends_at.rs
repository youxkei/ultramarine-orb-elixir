//! **The screen a run of th06nc ends at is the game's to walk, and orb writes one state into it.**
//!
//! That screen is several one after another, off the jump table at `0x140073ef4`, and a launch of the real
//! game walked them: **9** the name entry over the ranking, which the job a run ends into builds it in
//! (`0x14007bdde`); **15** the offer to save a replay, which the entry's own end hands on to
//! (`0x1400727d5`); then `16, 10, 12, 14`, a decide each, the question standing drawn throughout and its
//! answer taken at fourteen; and **17** the way out, `0x14006fb0e`, which fades the screen and writes the
//! front end into the supervisor's wanted state.
//!
//! **Only the offer is orb's business.** A Pointdevice run has nothing the *game* could put in a replay —
//! its record is orb's own file, and the buttons without the rewinds between them play back as a run dying
//! where this one carried on — so `Th06nc::skip_replay_prompt` writes the way out over that one state and
//! over no other. The name entry before it belongs to whoever just finished the run.
//!
//! **Which is why the write is at the offer and not where the screen comes up.** A write on the frame the
//! screen is built would cover the name entry, and a name entry sent away is a run's own score with
//! nobody's name on it: th06 1.02h lost both that screen and the stats screen inside two frames to a
//! ranking asked for on the wrong one, which is `the_screen_a_finished_run_ends_at.rs`'s own subject.
//!
//! **And the four states between the offer and the way out are what the laid-out game owes.** The real
//! screen animates the question through `16, 10, 12` and takes the answer at `14`, where
//! `0x1400734f3` raises it over a replay slot that already holds a `T6RP`; this game has the offer as one
//! state, because that one is what orb reaches and every path to the others goes through it. A test that
//! walked the four would be asserting `fake::th06nc` rather than the game.

use orb_config::LogLevel;
use orb_core::game::th06nc::image::{
    RESULT_LEAVING, RESULT_SAVE_REPLAY_QUESTION, RESULT_TYPING_A_NAME, SCREEN_TITLE,
};
use orb_sim::keys;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs is, in the game's own frames, and how many frames a whole run of six
/// needs: `--clear` is how a result screen is reached at all in this mode, so the shape is
/// `a_clear_on_demand_of_th06nc`'s.
const STAGE_FRAMES: u32 = 330;
const STAGES: u32 = 6;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * STAGES * 2;

/// How many frames the screens of a finished run are given to be walked.
const THE_SCREENS: u32 = 600;

/// The line orb writes where it has written the offer away.
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
fn the_screen_a_finished_run_of_th06nc_ends_at_opens_in_its_name_entry() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06nc-the-screen-a-run-ends-at");
        assert_eq!(
            game.image().the_state_the_screen_a_run_ends_at_is_in(),
            RESULT_TYPING_A_NAME,
            "the screen the run ended at did not open in the name entry",
        );
        // And nothing of orb's has been written into it yet, which is what the write's own frame is about.
        assert!(
            !game.log().said(NO_REPLAY),
            "the offer was written away on the frame the screen came up, which covers the name entry",
        );
    });
}

/// And the screens after it are the game's own, walked by presses, with orb's one write at the offer.
#[test]
fn the_screens_of_a_finished_run_of_th06nc_are_walked_to_the_offer_orb_writes_away() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06nc-the-screens-a-run-ends-at");
        let seen = walked(&game);

        assert_eq!(
            *seen.first().expect("a state"),
            RESULT_TYPING_A_NAME,
            "the walk did not start at the name entry: {seen:?}",
        );
        assert_eq!(
            *seen.last().expect("a state"),
            RESULT_LEAVING,
            "the screen never reached its way out: {seen:?}",
        );
        // **And the offer never stood for a frame the walk could read**, which is where orb's one write
        // lands: `Th06nc::skip_replay_prompt` is asked on every frame of a pointdevice run, so the state
        // the game put there is written past inside the frame it appeared in.
        assert!(
            !seen.contains(&RESULT_SAVE_REPLAY_QUESTION),
            "the offer to save a replay stood for a frame of its own: {seen:?}",
        );
        assert!(
            game.log().said(NO_REPLAY),
            "the offer to save a replay was not written away:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And that state is said in the log whenever it changes, which is how a walk of those screens is read back
/// off a launch of the real game — the line the states above were read with.
#[test]
fn the_state_the_screen_a_finished_run_of_th06nc_ends_at_is_in_is_said_as_it_changes() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06nc-the-screen-said");
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

/// **And the front end a finished run comes back to is the title**, where a run *given up* comes back to
/// the attract cycle instead.
///
/// **Measured on the real game, and two launches say it between them**: the launch of
/// `the-offer-written-away.yaml` read `result: the screen a run ends at is down` and then
/// `front end: screen 2` a quarter of a second later with no 16 between them, where
/// `the-front-end-after-a-run.yaml`'s `leave: true` read `front end: screen 16` sixteen milliseconds
/// after the give-up and then 17 and 18. So what the front end comes up on is which kind of end the run
/// had — see `the_front_end_of_th06nc_walked`, which holds the other one.
#[test]
fn the_front_end_a_finished_run_of_th06nc_comes_back_to_is_its_title() {
    in_its_own_process(|| {
        let game = a_cleared_run("th06nc-the-screen-back-to-the-title");
        walked(&game);
        // The way out taken: the screen asks the supervisor for the front end, which is the build that
        // writes the screen it comes up on — nought until then, this run having been said rather than
        // walked into.
        game.frames_until("the front end back", THE_SCREENS, || {
            game.image().the_screen_the_front_end_is_on() != 0
        });
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_TITLE,
            "the front end a finished run came back to was not the title",
        );
    });
}
