//! **The screen a run of th08 ends at is the game's to walk, and orb writes one phase into it.**
//!
//! That screen is several one after another, off the table at 0x458658: the name entry a played run opens
//! in (phase 9, 0x45621e), the score the entry's end opens (0xf, which takes a decide of its own once up
//! ninety updates), the thirty updates before the offer (0x10), the offer of a replay and the steps of saving
//! one (0xa through 0xe, 0x45696f), and the way out (2).
//!
//! **Only the offer is orb's business.** A Pointdevice run has nothing the *game* could put in a replay — its
//! record is orb's own file — so `Th08::skip_replay_prompt` writes the way out over the offer and over no
//! other phase, which is the frame the game would have asked on. The name entry and the score before it
//! belong to whoever just finished the run.

use orb_config::LogLevel;

use crate::fake::th08::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_core::game::th08::image::Scene;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each stage of these runs lasts and how many frames a whole run of six of them is given — see
/// `the_replay_offered_of_th08`, whose run this is.
const STAGE_FRAMES: u32 = 1_500;
const A_WHOLE_RUN: u32 = STAGE_FRAMES * 6 * 3;

/// How many frames the screens of a finished run are given to be walked: the score alone ignores a decide
/// for ninety of them and the pause after it is thirty more.
const THE_SCREENS: u32 = 1_200;

/// **The phases, by the numbers the screen's own table switches on** — see `Th08`'s `RESULT_PHASE`. Written
/// out rather than borrowed, what an e2e test says the game went through being the game's numbers and not
/// orb's reading of them.
const TYPING_A_NAME: i32 = 9;
const SHOWING_THE_SCORE: i32 = 0xf;
const BEFORE_THE_OFFER: i32 = 0x10;
const OFFERING_A_REPLAY: i32 = 0xa;
const LEAVING: i32 = 2;

/// The line orb writes where it has written the offer away.
const NO_REPLAY: &str = "result: no replay of the game's own is offered for a run with chapters";

/// **A run in Pointdevice mode played to its clear by the rules**, and the screen it ends at after the
/// ending, walked by the decide tapped every other frame until the title is up.
fn a_cleared_run(name: &str) -> Box<Fake> {
    let recipe = format!(
        "{TO_A_RUN}\
- play_until: {{ frames: {A_WHOLE_RUN} }}
- press: [decide]
  frames: 1
  until: {{ screen: 0 }}
  give_up_after: {THE_SCREENS}
- wait_until: {{ frames: 300 }}
- quit: true
"
    );
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe);
            config.log_level = LogLevel::Verbose;
            config.save_replays = true;
        },
    );
    game.plays_its_own_stage();
    game.stages_last(STAGE_FRAMES);
    game.frames_until("the screen the run ends at", A_WHOLE_RUN * 2, || {
        game.image().the_scene_now() == Scene::RunOver as i32
    });
    game
}

/// **Every phase that screen came to, in the order it came to them**, read off it frame by frame until its
/// update leaves the chain — which is the screen gone, its object freed with it.
fn walked(game: &Fake) -> Vec<i32> {
    let mut seen = vec![game.image().where_the_screen_a_run_ends_at_is().0];
    for _ in 0..THE_SCREENS {
        game.frame();
        if !game.image().the_screen_a_run_ends_at_is_on_the_chain() {
            break;
        }
        let (phase, _) = game.image().where_the_screen_a_run_ends_at_is();
        if phase != *seen.last().expect("a phase") {
            seen.push(phase);
        }
    }
    seen
}

/// A finished run's screen opens in the name entry, which is the phase the game registers it in.
#[test]
fn the_screen_a_finished_run_of_th08_ends_at_opens_in_its_name_entry() {
    in_its_own_process(|| {
        let game = a_cleared_run("th08-the-screen-a-run-ends-at");
        assert_eq!(
            game.image().where_the_screen_a_run_ends_at_is().0,
            TYPING_A_NAME,
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
fn the_screens_of_a_finished_run_of_th08_are_walked_to_the_offer_orb_writes_away() {
    in_its_own_process(|| {
        let game = a_cleared_run("th08-the-screens-a-run-ends-at");
        let seen = walked(&game);
        assert_eq!(
            seen,
            vec![TYPING_A_NAME, SHOWING_THE_SCORE, BEFORE_THE_OFFER, LEAVING],
            "the screens of a finished run did not walk the name entry, the score and the pause into the \
             way out",
        );
        // **And the offer never stood for a frame of its own**, which is where orb's one write lands: it is
        // made on the frame the game put that phase there, so no frame of the screen is ever the offer's.
        assert!(
            !seen.contains(&OFFERING_A_REPLAY),
            "the offer of a replay stood for a frame of its own: {seen:?}",
        );
        assert!(
            game.log().said(NO_REPLAY),
            "the offer of a replay was not written away:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And that phase is said in the log whenever it changes, which is how a walk of those screens is read back
/// off a launch of the real game.
#[test]
fn the_phase_the_screen_a_finished_run_of_th08_ends_at_is_in_is_said_as_it_changes() {
    in_its_own_process(|| {
        let game = a_cleared_run("th08-the-screen-said");
        walked(&game);
        let said: Vec<String> = game
            .log()
            .lines()
            .into_iter()
            .filter(|line| line.contains("result: the screen a run ends at is in phase"))
            .collect();
        assert!(
            said.len() >= 4,
            "the screen's own phases were not said as they changed: {said:?}",
        );
    });
}
