//! **th07's title menu walked the way the real one walks**: onto the last item at a cancel, and over the
//! item it is not offering.
//!
//! The same reading as `the_title_menu_walked`'s, off traces of the real th07 — `cargo xtask real` keeps one
//! per case and `orb-real show` reads it back — and here for the same reason: a recipe walks this menu by
//! pressing at it, so a walk that lands elsewhere reaches a screen the case is not about.
//!
//! What is not here is the wrap this game's menu has and th06's shares, which `Th07`'s own `sight::in_a_stage`
//! already carries a reading of: the laid-out menu has counted its items round from the start.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many items this menu has, which is th07's own eight, and where the three that matter here sit:
/// `Start`, `Extra Start` and `Practice Start` — `Th07`'s own `TITLE_ITEM_START`, `TITLE_ITEM_EXTRA` and
/// `TITLE_ITEM_PRACTICE`, which that module keeps to itself.
const ITEMS: i32 = 8;
const START: i32 = 0;
const EXTRA: i32 = 1;
const PRACTICE: i32 = 2;

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

#[test]
fn a_cancel_puts_the_cursor_on_the_last_item() {
    in_its_own_process(|| {
        let game = game("th07-title-cancel");
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            START,
            "the title did not come up on its first item",
        );
        // The six-frame cancel a recipe presses to take orb's own question away, which the real menu answers
        // by putting the cursor on the item that ends the game — as th06's does. Every walk those recipes
        // make afterwards starts from there.
        game.press(keys::X);
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            ITEMS - 1,
            "a cancel at the title left the cursor where it was",
        );
    });
}

#[test]
fn a_decide_on_the_last_item_ends_the_game() {
    in_its_own_process(|| {
        let game = game("th07-title-quit");
        // The item a cancel puts the cursor on, which is the one the case that clears the game walks to: the
        // real game's trace stops a few frames past `step 13 of 16 done`, the decide of the step after it
        // having ended the process. th06 does the same.
        game.press(keys::X);
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            ITEMS - 1,
            "the cursor is not on the item that ends the game",
        );
        // **And not at once**: the item sends the front end to its phase 2 (0x456000), which asks for the scene
        // that ends the game only once its timer has passed 0x3b (0x4554d6's own case), and the loop that then
        // stops saves the score file on its way out (0x434277) — the play records being tagged since the
        // supervisor's own start (0x438986).
        let saves = game.image().times_it_was_asked_to_save();
        game.press(keys::Z);
        assert!(
            game.sim().exited().is_none(),
            "the game ended on the decide, before the front end's own way down",
        );
        game.frames_until("the game ended", THE_WAY_DOWN_TAKES, || {
            game.sim().exited().is_some()
        });
        assert!(
            game.image().times_it_was_asked_to_save() > saves,
            "the game ended without the save its way out makes",
        );
    });
}

/// The updates the front end's way down to the end of the game takes, its timer passing 0x3b, and a few more for
/// the loop's own way out.
const THE_WAY_DOWN_TAKES: u32 = 0x3c + 4;

#[test]
fn the_item_that_starts_the_extra_is_stepped_over_where_the_records_hold_no_clear() {
    in_its_own_process(|| {
        let game = game("th07-title-locked-extra");
        game.its_records_hold_no_clear();
        // Down from the first item reaches the third, the second being the Extra's and not offered: measured
        // on a copy whose score file held no clear, where the cursor went 0 to 2 pressing down and 0 to 7
        // pressing up. What that costs a walk aiming at the Extra is the decide after it, which starts
        // whatever the cursor did land on.
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            PRACTICE,
            "the cursor rested on the Extra, which this game's records do not offer",
        );
        game.press(keys::UP);
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            START,
            "the cursor came back through the Extra",
        );
    });
}

#[test]
fn the_extra_is_reached_where_they_hold_one() {
    in_its_own_process(|| {
        let game = game("th07-title-unlocked-extra");
        // And the item is there where the records hold a clear, which is what the case after the one that
        // clears the game finds: the same press, the other answer.
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_title_menu_points_at(),
            EXTRA,
            "the cursor stepped over the Extra of a game that has been cleared",
        );
    });
}
