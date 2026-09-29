//! **th06nc's main menu walked the way the real one walks**: over the item it is not offering, round its
//! ten, and deaf to a decide for its own first twenty frames.
//!
//! The same reading as `the_title_menu_of_th06c_walked`'s and for the same reason: every recipe of
//! `cargo xtask real` walks this menu by pressing at it, so a laid-out menu that lands on another item
//! reaches a screen the case is not about. Where each number comes from, off `th06nc.exe`:
//!
//! - **Ten items**, which `0x14004e950` steps the cursor through and whose second — the Extra — is
//!   stepped over where none of the clear records' bytes at `+0xd2a1` onwards is `0x63`. A fresh launch's
//!   taps of down went `0, 2, 3 … 9`.
//! - **And the decide ignored for twenty frames**, `0x140049a8d` comparing `[front end + 0x9638]` with
//!   `0x14` and leaving the menu's own case for anything under it — which is what every recipe's six-frame
//!   tap is repeated past, and what `Th06nc::menu_takes_a_press` answers a count for.
//!
//! **This game's title is a screen of its own**, which is the difference from th06c: state 2 is the title
//! and a decide there opens the menu at state 3, where th06c's title *is* the menu. So the walk below
//! presses once more than th06c's does.
//!
//! The item orb's own question stands over is `the_mode_question_over_th06nc`'s subject; what this file is
//! about is the menu underneath it, so the mode is fixed rather than asked.

use orb_config::LogLevel;
use orb_core::game::th06nc::image::{SCREEN_MENU, SCREEN_TITLE};
use orb_sim::keys;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a walk here is given before it says the menu went nowhere.
const FRAMES: u32 = 1_200;

/// How many items this menu has, and where the two that matter here sit — `Th06nc`'s own
/// `front_end::ITEM_*`, which that module keeps to itself.
const ITEMS: i32 = 10;
const RUN: i32 = 0;
const EXTRA: i32 = 1;

/// The frames the menu ignores a press for, and a few more: what a walk waits out before its first press
/// lands at all.
const TAKES_A_PRESS_AFTER: u32 = 0x14 + 6;

/// A launch with the game at its main menu, past the frames its own screen ignores a press for.
///
/// **Walked to rather than said**, which is what makes it a walk of the menu: the title is reached on its
/// own and one decide opens the menu, both of which are the front end's own doing.
fn at_the_menu(name: &str, cleared: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
        },
    );
    if !cleared {
        game.its_records_hold_no_clear();
    }
    game.frames_until("the title", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
    });
    game.press(keys::RETURN);
    game.frames_until("the main menu", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_MENU
    });
    game.frames(TAKES_A_PRESS_AFTER);
    game
}

/// **A decide inside the menu's own grace does nothing**, which is the number every recipe's tap is
/// repeated past: the menu comes up on its first item and stays there.
#[test]
fn a_decide_in_the_first_twenty_frames_of_th06ncs_menu_is_not_acted_on() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06nc-menu-grace",
            Work::wandering(WORK_US),
            |config| {
                config.log_level = LogLevel::Verbose;
            },
        );
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        game.press(keys::RETURN);
        game.frames_until("the main menu", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_MENU
        });
        // The menu is one frame old here, so a decide is inside its grace.
        game.press(keys::RETURN);
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_MENU,
            "a decide inside the menu's grace left the menu",
        );
    });
}

/// **The cursor goes round the ten items**, which is what a recipe pressing down at this menu is held to.
#[test]
fn the_cursor_of_th06ncs_menu_goes_round_its_ten_items() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-menu-round", true);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            RUN,
            "the menu did not come up on its first item",
        );
        for item in 1..ITEMS {
            game.press(keys::DOWN);
            assert_eq!(
                game.image().the_item_the_cursor_is_on(),
                item,
                "a tap of down did not reach item {item}",
            );
        }
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            RUN,
            "the cursor did not come back round to the first item",
        );
    });
}

/// **And it steps over the Extra while no clear has unlocked it**, which is why a recipe that walks to the
/// third item taps down once from the first and not twice.
#[test]
fn the_cursor_of_th06ncs_menu_steps_over_the_extra_it_is_not_offering() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-menu-locked-extra", false);
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            EXTRA + 1,
            "the cursor stopped on the Extra with no clear to unlock it",
        );
    });
}

/// **A cancel at the menu puts the cursor on its last item and leaves the menu standing**, which is what a
/// launch of the real game found and what `an_attempt_at_a_card_counted_and_kept`'s recipe walks on: it taps
/// the cancel until the menu is up and then aims the cursor at the first item again, and it would have
/// reached no menu at all had a cancel gone back to the title.
///
/// **Tapped more than once**, because the recipe does: a cancel that brought the menu up again would start
/// its twenty frames of grace over each time, and the decide the recipe presses after it would land on
/// nothing.
#[test]
fn a_cancel_at_th06ncs_menu_puts_the_cursor_on_its_last_item_and_leaves_the_menu_up() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-menu-cancel", true);
        for _ in 0..3 {
            game.press(keys::X);
            assert_eq!(
                game.image().the_screen_the_front_end_is_on(),
                SCREEN_MENU,
                "a cancel at the menu left it",
            );
            assert_eq!(
                game.image().the_item_the_cursor_is_on(),
                ITEMS - 1,
                "a cancel did not put the cursor on the menu's last item",
            );
        }
        // And the menu takes a press straight after, the cursor going round from its last item to its
        // first — which is the walk the recipe makes once the cancel has stood the menu up — and a decide
        // there is acted on: orb's own question goes up over the item a run is behind, which is the press
        // reaching the menu's own case and not a grace the cancels started over.
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            RUN,
            "the menu did not take a press after the cancels",
        );
        let log = game.log();
        game.press(keys::RETURN);
        game.frames(2);
        assert!(
            log.said("menu: Run is under the cursor, asking which mode"),
            "a decide after the cancels was held back by a grace they had started over:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **With a clear among the records it stops there**, the same menu offering one more item.
#[test]
fn the_cursor_of_th06ncs_menu_stops_on_the_extra_a_clear_unlocked() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-menu-unlocked-extra", true);
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            EXTRA,
            "the cursor stepped over the Extra a clear had unlocked",
        );
    });
}
