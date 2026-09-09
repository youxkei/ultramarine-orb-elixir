//! **th07's front end walked the way the real one walks**: the item the menu comes back on when a run is
//! left, the screens a left practice run comes back through, and the item the Extra's own first screen comes
//! up on.
//!
//! The same reading as `the_front_end_walked`'s, off traces of the real th07 — `cargo xtask real` keeps one
//! per case and `orb-real show` reads it back — and here for the same reason: the `front end: screen N,
//! cursor on item M` line is what a case's walk is held to, so a laid-out game that reads a screen or an item
//! the real one does not parts from every trace that reaches it.
//!
//! The title menu's own items are walked in `the_title_menu_of_th07_walked`, which is the other half of the
//! same reading.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many taps a walk here may make before it says the screens went nowhere: the three screens between an
/// item and a run each ignore a press for their own first frames, and a tap is two of them.
const TAPS: u32 = 300;

/// The items of the title menu these e2e tests start runs from — `Th07`'s own `TITLE_ITEM_START`,
/// `TITLE_ITEM_EXTRA` and `TITLE_ITEM_PRACTICE`, which that module keeps to itself.
const EXTRA: i32 = 1;
const PRACTICE: i32 = 2;

/// And the screens the front end walks, by the numbers the table at 0x4555ce sends to Start's own handlers:
/// the title, the Extra's difficulty, and the practice flow's four.
const SCREEN_TITLE: i32 = 0;
const SCREEN_EXTRA_DIFFICULTY: i32 = 12;
const SCREEN_PRACTICE_DIFFICULTY: i32 = 8;
const SCREEN_PRACTICE_CHARACTER: i32 = 9;
const SCREEN_PRACTICE_SHOT_SELECT: i32 = 10;
const SCREEN_PRACTICE_STAGE: i32 = 11;

/// A launch with the game at its title menu, past the frames its own screen ignores a press for.
fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.frames_until("an overlay", 8, || log.said("overlay: ready"));
    game.frame();
    game.image().the_front_end_is_up();
    game.image().the_title_menu_points_at_a_run();
    game.image().the_screen_has_been_up_for(60);
    game.one_frame();
    game.one_frame_to_drain_the_log();
    game
}

/// The cursor walked to that item of the title menu and the decide made at it, with orb's own question about
/// the mode answered 完全無欠モード on the press after — which is the way into a run, that question standing
/// over every item that starts one.
///
/// # Panics
/// Naming whichever press did not land.
fn the_mode_answered_at(game: &Fake, item: i32) {
    let log = game.log();
    while game.image().the_item_the_title_menu_points_at() < item {
        game.press(keys::DOWN);
    }
    assert_eq!(
        game.image().the_item_the_title_menu_points_at(),
        item,
        "the cursor is not on the item the run is to be started from",
    );
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode question answered", || {
        log.said("mode: answered on the keyboard")
    });
}

/// And the screens behind that item walked to the run they end in: a decide apiece, tapped until the run is
/// asked for.
///
/// Tapped over a bound of its own rather than through `press_until`, whose thirty presses are one screen's
/// worth: each of these screens ignores a press for the first thirty frames it is up, so three of them cost
/// more taps than that.
///
/// # Panics
/// If no run is asked for.
fn the_screens_behind_it_walked(game: &Fake) {
    let run = game.image().scene_of_a_run();
    for _ in 0..TAPS {
        if game.image().the_scene_wanted() == run {
            break;
        }
        game.press(keys::Z);
    }
    assert_eq!(
        game.image().the_scene_wanted(),
        run,
        "the screens behind that item asked for no run in {TAPS} tap(s)",
    );
    game.frames_until("the stage that run is played in", 8, || {
        game.image().the_scene_now() == game.image().scene_of_a_run()
    });
}

/// And that run left, which is the run's own teardown asking for the front end — `leave`'s way out, and the
/// one the cases take.
fn and_left(game: &Fake) {
    let front_end = game.image().scene_of_the_front_end();
    let now = game.image().the_scene_now();
    game.image().is_in_scene(now, front_end);
    game.frames_until("the front end the run went back to", 60, || {
        game.image().the_scene_now() == front_end && game.image().the_scene_wanted() == front_end
    });
}

/// Where the front end reads over the next `frames`, one pair per change — which is what the line in a trace
/// is: `front end: screen N, cursor on item M` is written where either of the two moves.
fn read_over(game: &Fake, frames: u32) -> Vec<(i32, i32)> {
    let mut read = Vec::new();
    for _ in 0..frames {
        let now = (
            game.image().the_screen_the_front_end_is_on(),
            game.image().the_item_the_title_menu_points_at(),
        );
        if read.last() != Some(&now) {
            read.push(now);
        }
        game.frame();
    }
    read
}

#[test]
fn a_left_run_comes_back_on_the_item_it_was_started_from() {
    in_its_own_process(|| {
        let game = game("th07-left-run-comes-back-on-the-extra");
        // The Extra's item, which the real game came back on: `front end: screen 0, cursor on item 1` on the
        // frame the front end was up again, where the screens between it and the run had each had the cursor
        // of their own list. A menu coming back on its first item would leave the case after this one walking
        // to the Extra from somewhere else.
        the_mode_answered_at(&game, EXTRA);
        the_screens_behind_it_walked(&game);
        and_left(&game);
        assert_eq!(
            (
                game.image().the_screen_the_front_end_is_on(),
                game.image().the_item_the_title_menu_points_at(),
            ),
            (SCREEN_TITLE, EXTRA),
            "the menu came back somewhere other than the item the run was started from",
        );
    });
}

#[test]
fn and_a_left_practice_run_back_through_the_screens_it_was_started_at() {
    in_its_own_process(|| {
        let game = game("th07-left-practice-run-comes-back");
        // And where the item was `Practice Start`, the real game walked back into that flow rather than
        // resting at its title: `screen 8, cursor on item 2` on the frame the front end was up again — the
        // item the title was left on, still in the one cursor this front end has — and then screens 9, 10 and
        // 11 on the three frames after it, each on its own first item. Which is the stage select the run was
        // started at, so whoever left it is offered another stage.
        the_mode_answered_at(&game, PRACTICE);
        the_screens_behind_it_walked(&game);
        and_left(&game);
        assert_eq!(
            read_over(&game, 8),
            vec![
                (SCREEN_PRACTICE_DIFFICULTY, PRACTICE),
                (SCREEN_PRACTICE_CHARACTER, 0),
                (SCREEN_PRACTICE_SHOT_SELECT, 0),
                (SCREEN_PRACTICE_STAGE, 0),
            ],
            "the practice run's own screens are not what the front end came back through",
        );
    });
}

#[test]
fn the_extras_own_first_screen_comes_up_on_its_first_item() {
    in_its_own_process(|| {
        let game = game("th07-the-extras-first-screen");
        // The screen the Extra's item opens, which the real game brought up on its first item: `front end:
        // screen 12, cursor on item 0`, and no reading of it on any other. The difficulty a run of it is
        // played on is the item's own and no item of that select is another, so a cursor put on the Extra's
        // difficulty there is a number that screen never shows.
        the_mode_answered_at(&game, EXTRA);
        game.frames_until("the screen the Extra's item opens", 120, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_EXTRA_DIFFICULTY
        });
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            0,
            "the Extra's own first screen came up on an item other than its first",
        );
    });
}
