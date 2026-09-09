//! **th06's front end walked the way the real one walks**: the state a launch comes up on, which waits for a
//! decide; the item the menu comes back on when a run is left; and the row its replay screen comes up on.
//!
//! Every claim here was read off a trace of the real game — `cargo xtask real`'s own, kept per case under the
//! copy it was played in and read back with `orb-real show` — and each of them is a way a laid-out game was
//! not this game until it was written down here. What makes them worth an e2e test of their own rather than a
//! line in the case list is that the `front end: screen N, cursor on item M` line is what a case's walk is
//! held to: a laid-out game that reads a screen or an item the real one does not parts from every trace that
//! reaches it, and the case then passes or fails for a reason that is not the case's.
//!
//! The title menu's own items are walked in `the_title_menu_walked`, which is the other half of the same
//! reading.

use orb_config::LogLevel;
use orb_core::game::th06::image::{Scene, Screen, item};
use orb_sim::keys;

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, in_its_own_process};

/// The number `main_menu::GAME_STATE` holds on the screen a launch comes up on, which is the state the front
/// end is built into: 1, where the title is 2.
const COMES_UP_ON: i32 = 1;

/// How long the real game was watched on that screen with nothing pressed at it — the wait a measurement of
/// the day put after the `leave` step of `a_run_left_in_its_second_chapter`'s own recipe.
const WATCHED_FOR: u32 = 3_600;

/// A launch on the screen the real one comes up on, with the screens between the title and a run asked for:
/// that state is where they start.
fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.walks_the_screens_between_its_title_and_a_run();
    game
}

/// A launch at the title menu, whose screens between it and a run are the ones a laid-out game has by
/// default: the shot type select and nothing else.
fn at_the_title(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    game
}

/// A run started from that item of the title menu and given up at the game's own pause, which is the shape
/// of every case that leaves one: the item pressed, orb's question about the mode answered on the press
/// after, the shot type taken, and the stage select behind it where the item was `Practice Start`.
///
/// # Panics
/// Naming whichever screen the walk did not get past.
fn a_run_started_from(game: &Fake, from: i32) {
    let log = game.log();
    while game.image().front_end_now().cursor < from {
        game.press(keys::DOWN);
    }
    assert_eq!(
        game.image().front_end_now().cursor,
        from,
        "the cursor is not on the item the run is to be started from",
    );
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered", || {
        log.said("mode: answered on the keyboard")
    });
    game.frames_until("the shot type select ready to act on a press", 120, || {
        let front = game.image().front_end_now();
        front.screen == Screen::ShotType && front.acts_on_a_press()
    });
    game.press(keys::Z);
    if from == item::PRACTICE {
        game.frames_until(
            "the practice stage select ready to act on a press",
            120,
            || {
                let front = game.image().front_end_now();
                front.screen == Screen::PracticeStage && front.acts_on_a_press()
            },
        );
        game.press(keys::Z);
    }
    game.frames_until("the stage built", 120, || game.state().playing);
}

/// And that run left: やめる at the game's own pause, and the menu it comes back to built.
///
/// Waited for by the screen and not by the scene alone, the two being a frame apart: the pause writes the
/// scene the front end runs in and the supervisor builds the menu on the frame after, so the front end a
/// frame too early is still the one the run was started from.
fn and_left(game: &Fake) {
    game.gives_the_run_up_at_its_own_pause();
    game.frames_until("the title menu built again", 60, || {
        game.image().scene() == Scene::FrontEnd
            && game.image().front_end_now().screen == Screen::Title
    });
}

#[test]
fn the_state_a_launch_comes_up_on_stays_where_nothing_is_pressed() {
    in_its_own_process(|| {
        let game = game("th06-comes-up-on-waits");
        game.frames(WATCHED_FOR);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Other(COMES_UP_ON),
            "the state a launch comes up on went somewhere with nothing pressed at it",
        );
    });
}

#[test]
fn and_hands_over_to_the_title_on_the_frame_after_a_decide() {
    in_its_own_process(|| {
        let game = game("th06-comes-up-on-decided");
        // Held from the launch's first frame rather than tapped, which is what the recipes do — their first
        // step is a decide held for thirty frames — and what the trace this is read off shows: the state is
        // still the one the launch came up on while that first frame is read.
        game.keyboard().set(keys::Z, true);
        game.frames(1);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Other(COMES_UP_ON),
            "the launch's own first frame was read somewhere other than the state it comes up on",
        );
        game.frames(1);
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Title,
            "a decide at the state a launch comes up on did not reach the title",
        );
    });
}

#[test]
fn a_left_run_comes_back_on_the_item_it_was_started_from() {
    in_its_own_process(|| {
        let game = at_the_title("th06-left-run-comes-back-on-the-extra");
        // The Extra's item, which the real game came back on: `front end: screen 1, cursor on item 1` on the
        // frame the front end was built again, where the run before it had walked the cursor to that item and
        // every screen after it had the cursor of its own list. A menu coming back on its first item would
        // leave the case after this one walking to the Extra from somewhere else.
        a_run_started_from(&game, item::EXTRA);
        and_left(&game);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::EXTRA,
            "the menu came back on an item the run was not started from",
        );
    });
}

#[test]
fn and_on_the_practice_item_where_that_is_where_it_was_started() {
    in_its_own_process(|| {
        let game = at_the_title("th06-left-run-comes-back-on-practice");
        // `Practice Start`, which the real game came back on the same way: `screen 1, cursor on item 2` after
        // a practice run was left, where the stage select the run was started from had had the cursor on its
        // own first row.
        a_run_started_from(&game, item::PRACTICE);
        and_left(&game);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::PRACTICE,
            "the menu came back on an item the practice run was not started from",
        );
    });
}

#[test]
fn the_replay_screen_comes_up_on_its_first_row() {
    in_its_own_process(|| {
        let game = at_the_title("th06-replay-screen-first-row");
        // The item the replays are behind, whose own screen the real game brought up on its first row:
        // `front end: screen 12, cursor on item 3` while it comes up — the item pressed, still — and
        // `screen 13, cursor on item 0` once it is up. A screen still reading the title's item is one orb's
        // own list stands over saying a row nobody chose.
        let log = game.log();
        while game.image().front_end_now().cursor < item::REPLAY {
            game.press(keys::DOWN);
        }
        // The decide at it and orb's own question about the mode answered on the press after, which is what
        // the case does: 完全無欠モード is the answer whose list stands over the game's own screen, so the
        // screen below comes up either way.
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        game.frames_until("the screen the game keeps its own replays on", 300, || {
            game.image().front_end_now().screen == Screen::Replays
        });
        assert_eq!(
            game.image().front_end_now().cursor,
            0,
            "the replay screen came up on a row other than its first",
        );
    });
}
