//! **The screen a run of th11 ends at is the game's to walk, and orb writes one state into it.**
//!
//! That screen is the pause object's end of a run, a won run's states 20 to 26 handled by 0x42e6b0 (its table
//! `0x42f31c`). A run won — which a practice's stage won is (0x41eba3 into 0x42d6b0) — opens on the won run's tally (21), which ten
//! frames on raises the high score and, for a practice, puts up the won run's menu of three — the title, the
//! replay's save and the start-over — with its cursor on the title (22, 0x42e774 through 0x42e7f7); the menu's
//! decide carries the item out by the state that leaves (26), which twelve frames on puts the pause at 27 and asks
//! for the title (0x42f2b2 through 0x42f301).
//!
//! **Only the menu is orb's business in Pointdevice mode.** A run with chapters has nothing the *game* could put
//! in a replay — its record is orb's own file — so `Th11::skip_replay_prompt` writes the way out to the title over
//! the menu and over no other state, which is the frame the game would have asked on. The tally belongs to
//! whoever just finished the run.
//!
//! **th10's run over, the state before its tally, has no counterpart here**: 0x42d6b0 puts the pause straight on
//! the tally (`the_run_is_over_cleared`), so the screen opens on it.
//!
//! **The run here is a practice**, whose stage won ends it there: a run of the six stages ends at the ending
//! instead.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_core::game::th11::image::Scene;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the practised stage lasts: its midboss from frame 1200 and its boss for the last 900 frames, the
/// conversation after her ending the stage.
const STAGE_FRAMES: u32 = 3_600;

/// How many frames the screens of a finished run are given to be walked.
const THE_SCREENS: u32 = 600;

/// **The states, by the numbers the pause object's switch takes** — see `Th11`'s `PAUSE_THE_RUN_IS_OVER`. Written
/// out rather than borrowed, what an e2e test says the game went through being the game's numbers and not orb's
/// reading of them.
const TALLYING: i32 = 21;
const THE_MENU: i32 = 22;
const LEAVING: i32 = 26;
const DONE: i32 = 27;

/// The line orb writes where it has written the menu away.
const NO_REPLAY: &str = "result: no replay of the game's own is offered for a run with chapters";

/// **A practice of the first stage played by the laid-out stage to its end**, nothing fired at the player, with
/// the chapters on or off.
fn a_practice_won(name: &str, chapters: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            config.chapters = chapters;
        },
    );
    game.fires_at_the_player(false);
    game.stages_last(STAGE_FRAMES);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.image().the_run_becomes_a_practice();
    game.is_in_a_run();
    game.frames_until("the screen the run ends at", STAGE_FRAMES * 2, || {
        game.image().where_the_pause_is().0 == TALLYING
    });
    game
}

/// **Every state that screen came to, in the order it came to them**, read off it frame by frame with a decide
/// tapped every other frame until the title is up.
fn walked(game: &Fake) -> Vec<i32> {
    let mut seen = vec![game.image().where_the_pause_is().0];
    for frame in 0..THE_SCREENS {
        game.keyboard().set(keys::Z, frame % 2 == 0);
        game.frame();
        if game.image().the_scene_now() == Scene::Title as i32 {
            break;
        }
        let state = game.image().where_the_pause_is().0;
        if state != *seen.last().expect("a state") {
            seen.push(state);
        }
    }
    game.keyboard().set(keys::Z, false);
    seen
}

/// A finished run's screen opens on the tally, and nothing of orb's is written into it yet.
#[test]
fn the_screen_a_finished_run_of_th11_ends_at_opens_on_the_tally() {
    in_its_own_process(|| {
        let game = a_practice_won("th11-the-screen-a-run-ends-at", true);
        assert!(
            !game.log().said(NO_REPLAY),
            "the menu was written away on the frame the screen came up, which covers the tally",
        );
    });
}

/// **And the stage stands still under it**: the run's end puts up the game object's flag `0x10` (0x42d73f), the
/// pause menu's own, which stops the chain's walk at the game object — so neither the stage's frames nor anything
/// of the run below it goes on, and orb reads the game as paused.
#[test]
fn the_stage_of_a_finished_run_of_th11_stands_still_under_its_screen() {
    in_its_own_process(|| {
        let game = a_practice_won("th11-the-screen-stops-the-stage", true);
        assert!(
            game.state().paused,
            "the game object's flag was not up under the screen the run ends at",
        );
        let frames = game.state().stage_frames;
        game.frames(30);
        assert_eq!(
            game.state().stage_frames,
            frames,
            "the stage went on under the screen the run ends at",
        );
    });
}

/// And the states after it are the game's own, walked by presses, with orb's one write at the menu.
#[test]
fn the_screens_of_a_finished_run_of_th11_are_walked_to_the_menu_orb_writes_away() {
    in_its_own_process(|| {
        let game = a_practice_won("th11-the-screens-a-run-ends-at", true);
        let seen = walked(&game);
        assert_eq!(
            seen,
            vec![TALLYING, LEAVING, DONE],
            "the screens of a finished run did not walk the tally into the way out",
        );
        // **And the menu never stood for a frame of its own**, which is where orb's one write lands: it is made
        // on the frame the game put that state there, so no frame of the screen is ever the menu's.
        assert!(
            !seen.contains(&THE_MENU),
            "the end's menu stood for a frame of its own: {seen:?}",
        );
        assert!(
            game.log().said(NO_REPLAY),
            "the end's menu was not written away:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.image().the_scene_now(),
            Scene::Title as i32,
            "the way out orb wrote did not reach the title",
        );
    });
}

/// **And with the chapters off the menu is the game's**, coming up on its first item — the title — as the tally
/// puts it there for a practice (0x42e7f7), so the decide that answers it without a move is the way to the title.
#[test]
fn the_menu_a_finished_run_of_th11_ends_on_is_the_games_with_the_chapters_off() {
    in_its_own_process(|| {
        let game = a_practice_won("th11-the-ends-menu-the-games", false);
        let seen = walked(&game);
        assert_eq!(
            seen,
            vec![TALLYING, THE_MENU, LEAVING, DONE],
            "the screens of a finished run without chapters did not walk to the game's own menu",
        );
        assert!(
            !game.log().said(NO_REPLAY),
            "orb wrote the menu away over a launch with no chapters",
        );
        assert_eq!(
            game.image().the_scene_now(),
            Scene::Title as i32,
            "the menu's decide, on the item it comes up on, did not reach the title",
        );
    });
}
