//! **th06c's title menu walked the way the real one walks**: onto the item that ends the game at a cancel,
//! over the item it is not offering, round its nine, up before down, on again under a held key, and deaf to
//! a decide for its own first frames.
//!
//! The same reading as `the_title_menu_of_th07_walked`'s and for the same reason: every recipe of
//! `cargo xtask real` walks this menu by pressing at it, so a laid-out menu that lands on another item
//! reaches a screen the case is not about. Where each number comes from:
//!
//! - **Nine items**, `0x1400364b0(menu, 9)` stepping the cursor and the jump at `0x14002d657` taking 0 to 8,
//!   the last of them the one a cancel puts the cursor on (`movl $0x8` at `0x14002f9a5`); `0x14002d592` steps
//!   the cursor over the second where no clear has unlocked the Extra — `inc eax; mov [rdi+0x28],eax`.
//! - **Up before down**: `0x1400364b0` tests the up bit first and takes the down bit only where up is not
//!   pressed.
//! - **A held key pressed again every eight updates** once the word has stood thirty-two (`0x140058330`'s
//!   repeat, `[0x140955398]`), which the cursor's step takes as a press.
//! - **And the decide and the cancel ignored for twenty updates**, `cmp DWORD PTR [rdi+0x9450],0x14` inside
//!   the title's own case at `0x14002d614`, the count raised at the update's end — so the twenty-first is the
//!   first to take one, which is what every recipe's six-frame tap is repeated past.
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

/// How many items this menu has, and where the four that matter here sit — `Th06c`'s own `front_end::ITEM_*`,
/// which that module keeps to itself.
const ITEMS: i32 = 9;
const START: i32 = 0;
const EXTRA: i32 = 1;
const PRACTICE: i32 = 2;
const QUIT: i32 = 8;

/// The updates a key is held for here, and the presses the cursor takes in them: the one it went down on and
/// the repeats at the thirty-third, the forty-first and the forty-ninth.
const HELD_FOR: u32 = 50;
const PRESSES_IN_IT: i32 = 4;

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

/// And the cursor counts round the nine, so a walk that presses down enough times comes back where it
/// started.
#[test]
fn the_cursor_of_th06cs_title_counts_round_its_nine_items() {
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

/// Up and down held together are up.
#[test]
fn up_and_down_together_at_th06cs_title_step_the_cursor_up() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-title-up-and-down", true);
        game.keyboard().set(keys::UP, true);
        game.keyboard().set(keys::DOWN, true);
        game.frame();
        game.keyboard().set(keys::UP, false);
        game.keyboard().set(keys::DOWN, false);
        game.frame();
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            QUIT,
            "up and down together did not step the cursor up, round to the last item",
        );
    });
}

/// A key held down steps the cursor again and again, as the game's own repeat presses it.
#[test]
fn a_key_held_at_th06cs_title_steps_the_cursor_on_as_it_repeats() {
    in_its_own_process(|| {
        let game = at_the_title("th06c-title-held", true);
        game.keyboard().set(keys::DOWN, true);
        game.frames(HELD_FOR);
        game.keyboard().set(keys::DOWN, false);
        game.frame();
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            START + PRESSES_IN_IT,
            "the held key did not step the cursor once a press and once a repeat",
        );
    });
}

/// A launch just come to its title, with nothing of the title's own frames waited out.
fn just_at_the_title(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.frames_until("the title", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
    });
    game
}

/// The update the title first takes a decide on.
const THE_FIRST_TO_TAKE_ONE: u32 = 0x14 + 1;

/// A decide on the title's twentieth update goes unheard.
#[test]
fn a_decide_on_th06cs_twentieth_update_of_its_title_goes_unheard() {
    in_its_own_process(|| {
        let game = just_at_the_title("th06c-title-a-decide-too-early");
        game.frames(THE_FIRST_TO_TAKE_ONE - 2);
        game.keyboard().set(keys::Z, true);
        game.frame();
        game.keyboard().set(keys::Z, false);
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_TITLE,
            "a decide on the title's twentieth update was taken",
        );
    });
}

/// And one on its twenty-first is taken.
#[test]
fn a_decide_on_th06cs_twenty_first_update_of_its_title_is_taken() {
    in_its_own_process(|| {
        let game = just_at_the_title("th06c-title-a-decide-in-time");
        game.frames(THE_FIRST_TO_TAKE_ONE - 1);
        game.keyboard().set(keys::Z, true);
        game.frame();
        game.keyboard().set(keys::Z, false);
        assert_ne!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_TITLE,
            "a decide on the title's twenty-first update went unheard",
        );
    });
}

/// And a cancel goes unheard for the same updates the decide does.
#[test]
fn a_cancel_on_th06cs_title_goes_unheard_for_its_first_twenty_updates() {
    in_its_own_process(|| {
        let game = just_at_the_title("th06c-title-a-cancel-too-early");
        game.frames(THE_FIRST_TO_TAKE_ONE - 2);
        game.keyboard().set(keys::X, true);
        game.frame();
        game.keyboard().set(keys::X, false);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            START,
            "a cancel on the title's twentieth update moved the cursor",
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
