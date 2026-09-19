//! **th06c's title menu walked the way the real one walks**: onto the item that ends the game at a cancel,
//! over the item it is not offering, round its eight, and deaf to a decide for its own first frames.
//!
//! The same reading as `the_title_menu_of_th07_walked`'s and for the same reason: every recipe of
//! `cargo xtask real` walks this menu by pressing at it, so a laid-out menu that lands on another item
//! reaches a screen the case is not about. Where each number comes from:
//!
//! - **Eight items in th06 1.02h's own order**, which `0x14002d592` steps the cursor over the second of
//!   where no clear has unlocked the Extra — `inc eax; mov [rdi+0x28],eax`.
//! - **And the decide ignored for twenty frames**, `cmp DWORD PTR [rdi+0x9450],0x14` inside the title's own
//!   case at `0x14002d614`, which is what every recipe's six-frame tap is repeated past.
//!
//! The item orb's own question stands over is `the_mode_question_over_th06c`'s subject; what this file is
//! about is the menu underneath it, so the mode is fixed rather than asked — see
//! `the_front_end_of_th06c_walked`, whose launch is shaped the same way and for the same reason.

use orb_config::LogLevel;
use orb_core::game::th06c::image::SCREEN_TITLE;
use orb_sim::keys;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a walk here is given before it says the menu went nowhere.
const FRAMES: u32 = 1_200;

/// How many items this menu has, which is th06 1.02h's own eight, and where the four that matter here sit —
/// `Th06c`'s own `front_end::ITEM_*`, which that module keeps to itself.
const ITEMS: i32 = 8;
const START: i32 = 0;
const EXTRA: i32 = 1;
const PRACTICE: i32 = 2;
const QUIT: i32 = 7;

/// A launch with the game at its title menu, past the frames its own screen ignores a press for.
fn at_the_title(name: &str, cleared: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    if !cleared {
        game.its_records_hold_no_clear();
    }
    game.frames_until("the title", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
    });
    game.frames(TAKES_A_PRESS_AFTER);
    game
}

/// The frames the title ignores a press for, and a few more: what a walk waits out before its first press
/// lands at all.
const TAKES_A_PRESS_AFTER: u32 = 0x14 + 6;

/// A cancel puts the cursor on the item that ends the game, which is where every recipe's walk starts from
/// after orb's own question has been taken away.
#[test]
fn a_cancel_at_th06cs_title_puts_the_cursor_on_the_item_that_ends_the_game() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-title-cancel", true);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            START,
            "the title did not come up on its first item",
        );
        game.press(keys::X);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            QUIT,
            "the cancel did not put the cursor on the item that ends the game",
        );
    });
}

/// And the cursor counts round the eight, so a walk that presses down enough times comes back where it
/// started.
#[test]
fn the_cursor_of_th06cs_title_counts_round_its_eight_items() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-title-round", true);
        let mut seen = vec![game.image().the_item_the_cursor_is_on()];
        for _ in 0..ITEMS {
            game.press(keys::DOWN);
            seen.push(game.image().the_item_the_cursor_is_on());
        }
        assert_eq!(
            seen,
            (0..ITEMS).chain(std::iter::once(START)).collect::<Vec<_>>(),
            "the cursor did not count round the menu's own items",
        );
    });
}

/// And it steps over the Extra where the records hold no clear, which is the one item this menu takes off.
#[test]
fn the_cursor_of_th06cs_title_steps_over_the_extra_where_no_clear_unlocked_it() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-title-no-extra", false);
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            PRACTICE,
            "the cursor stopped on an Extra no clear unlocked",
        );

        // And up from there is the item the walk came from, the step over being both ways: a menu that took
        // the item off on the way down alone would leave a walk back up on an item it does not draw.
        game.press(keys::UP);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            START,
            "the walk back up stopped on the Extra it had stepped over",
        );
    });
}

/// And where a clear did unlock it, the cursor stops on it.
#[test]
fn the_cursor_of_th06cs_title_stops_on_the_extra_a_clear_unlocked() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-title-the-extra", true);
        game.press(keys::DOWN);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            EXTRA,
            "the cursor stepped over an Extra a clear had unlocked",
        );
    });
}
