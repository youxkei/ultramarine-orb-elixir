//! **th08's title menu walked the way the real one walks**: onto the item that ends the game at a cancel,
//! round its nine, over the two items its records have not lit, and deaf to a press while it builds itself.
//!
//! The same reading as `the_title_menu_of_th06c_walked`'s and for the same reason: every recipe of
//! `cargo xtask real` walks this menu by pressing at it, so a laid-out menu that lands on another item
//! reaches a screen the case is not about. Where each number comes from:
//!
//! - **Nine items**: 0x467960 hands 0x47030b a 9, which takes up below nought to 8 (0x47037a through
//!   0x470386) and down at nine to nought (0x4703df through 0x470416).
//! - **The cancel onto the last of them**, which 0x4681f3 writes 8 for.
//! - **Two items stepped over where the records have not lit them**: the Extra, off 0x43c1e9, and the spell
//!   practice, off 0x43c2c5 (0x467977 through 0x4679bf).
//! - **And a build before any of it is read**: one item's sprite an update while the count at `+0x1442c` is
//!   below nine (0x4678ce through 0x46792f), the phase the items are read in written on the tenth with the
//!   count put back to nothing (0x467934 through 0x46794f), and the decide and the cancel read only once the
//!   count has reached ten again (0x467cac). So a direction moves the cursor from the title's tenth update on
//!   and a cancel from its twentieth.
//!
//! **What is pressed here is a direction and a cancel and never a decide**: a decide on the title's first
//! item is what raises orb's own question about the mode — `the_mode_question_over_th08`'s subject — so the
//! gate is said of the cancel, which the same compare holds off.

use orb_config::LogLevel;
use orb_sim::keys;

use crate::fake::th08::{
    Fake, TITLE_ITEM_EXTRA, TITLE_ITEM_PRACTICE, TITLE_ITEM_QUIT, TITLE_ITEM_START,
};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a launch is given to bring its title up.
const FRAMES: u32 = 600;

/// How many items this menu has, written out here rather than taken from the fake's own count: a test that
/// read its numbers off the laid-out game would agree with it whatever either said.
const ITEMS: i32 = 9;

/// **The updates the title reads nothing over, by what it reads**: a direction from the tenth, and a cancel
/// from the twentieth.
const A_DIRECTION_FROM: u32 = 10;
const A_CANCEL_FROM: u32 = 20;

/// A launch left to itself, stopped on the update its title came up on — which is the title's first.
fn as_the_title_comes_up(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.frames_until("the title", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == Some(0)
    });
    game
}

/// The same launch past the updates the title reads nothing over.
fn at_the_title(name: &str) -> Box<Fake> {
    let game = as_the_title_comes_up(name);
    game.frames(A_CANCEL_FROM);
    game
}

/// `key` down for the one update that is the title's `update`th and up for the next, counted from the update
/// it came up on as its first.
fn taps_on_its_update(game: &Fake, key: u8, update: u32, now: &mut u32) {
    game.frames(update - 1 - *now);
    game.keyboard().set(key, true);
    game.frame();
    game.keyboard().set(key, false);
    game.frame();
    *now = update + 1;
}

/// `key` tapped at a title past its gates, and the item the cursor is on after it.
fn after(game: &Fake, key: u8) -> Option<i32> {
    game.keyboard().set(key, true);
    game.frame();
    game.keyboard().set(key, false);
    game.frame();
    game.image().the_item_the_cursor_is_on()
}

/// A cancel puts the cursor on the item that ends the game, and a second one leaves it there.
#[test]
fn a_cancel_at_th08s_title_puts_the_cursor_on_the_item_that_ends_the_game() {
    in_its_own_process(|| {
        let game = at_the_title("th08-title-cancel");
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            Some(TITLE_ITEM_START),
            "the title did not come up on its first item",
        );
        assert_eq!(
            after(&game, keys::X),
            Some(TITLE_ITEM_QUIT),
            "the cancel did not put the cursor on the item that ends the game",
        );
        assert_eq!(
            after(&game, keys::X),
            Some(TITLE_ITEM_QUIT),
            "a second cancel moved the cursor off the item that ends the game",
        );
    });
}

/// And the cursor counts round the nine both ways, so up from the first item is the last.
#[test]
fn the_cursor_of_th08s_title_counts_round_its_nine_items() {
    in_its_own_process(|| {
        let game = at_the_title("th08-title-round");
        game.its_records_light_every_item();
        let mut seen = vec![game.image().the_item_the_cursor_is_on()];
        for _ in 0..ITEMS {
            seen.push(after(&game, keys::DOWN));
        }
        assert_eq!(
            seen,
            (0..ITEMS)
                .chain(std::iter::once(TITLE_ITEM_START))
                .map(Some)
                .collect::<Vec<_>>(),
            "the cursor did not count round the menu's own items",
        );
        assert_eq!(
            after(&game, keys::UP),
            Some(ITEMS - 1),
            "up from the first item is not the last",
        );
    });
}

/// And it steps over the Extra and the spell practice where the records light neither, both ways.
#[test]
fn the_cursor_of_th08s_title_steps_over_the_two_items_nothing_has_lit() {
    in_its_own_process(|| {
        let game = at_the_title("th08-title-nothing-lit");
        assert_eq!(
            after(&game, keys::DOWN),
            Some(TITLE_ITEM_PRACTICE),
            "the cursor stopped on an item the records have not lit",
        );
        assert_eq!(
            after(&game, keys::UP),
            Some(TITLE_ITEM_START),
            "the walk back up stopped on an item it had stepped over",
        );
    });
}

/// And where the records light the Extra, the cursor stops on it.
#[test]
fn the_cursor_of_th08s_title_stops_on_the_extra_the_records_lit() {
    in_its_own_process(|| {
        let game = at_the_title("th08-title-the-extra");
        game.its_records_light_every_item();
        assert_eq!(
            after(&game, keys::DOWN),
            Some(TITLE_ITEM_EXTRA),
            "the cursor stepped over an Extra the records had lit",
        );
    });
}

/// **A direction is read from the title's tenth update and not before**, the nine before it being its build.
#[test]
fn a_direction_at_th08s_title_is_read_from_its_tenth_update() {
    in_its_own_process(|| {
        let game = as_the_title_comes_up("th08-title-direction-gate");
        let mut now = 1;
        taps_on_its_update(&game, keys::DOWN, A_DIRECTION_FROM - 1, &mut now);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            Some(TITLE_ITEM_START),
            "a direction moved the cursor while the title was still building itself",
        );
        taps_on_its_update(&game, keys::DOWN, A_DIRECTION_FROM + 1, &mut now);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            Some(TITLE_ITEM_PRACTICE),
            "a direction past the build did not move the cursor",
        );
    });
}

/// **And a cancel from its twentieth**, which is the same count the decide is held off by.
#[test]
fn a_cancel_at_th08s_title_is_read_from_its_twentieth_update() {
    in_its_own_process(|| {
        let game = as_the_title_comes_up("th08-title-cancel-gate");
        let mut now = 1;
        taps_on_its_update(&game, keys::X, A_CANCEL_FROM - 1, &mut now);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            Some(TITLE_ITEM_START),
            "a cancel was read before the title's count reached ten",
        );
        taps_on_its_update(&game, keys::X, A_CANCEL_FROM + 1, &mut now);
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            Some(TITLE_ITEM_QUIT),
            "a cancel past the count did not put the cursor on the item that ends the game",
        );
    });
}
