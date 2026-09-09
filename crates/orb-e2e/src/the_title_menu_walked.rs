//! **th06's title menu walked the way the real one walks**: round its items, onto the last one at a cancel,
//! and over the item it is not offering.
//!
//! Every claim here was read off a trace of the real game — `cargo xtask real`'s own, kept per case under the
//! copy it was played in and read back with `orb-real show` — and each of them is a way a laid-out game was
//! not this game until it was written down here. What makes them worth an e2e test of their own rather than
//! a line in the case list is that a recipe walks this menu by pressing at it: a walk that lands somewhere
//! else reaches a screen the case is not about, and the case then passes or fails for the wrong reason.
//!
//! The screens between the title and a run are walked in `the_walk_into_a_run`, which is the other half of
//! the same reading.

use orb_config::LogLevel;
use orb_core::game::th06::image::{Screen, item};
use orb_sim::keys;

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, in_its_own_process};

/// How many items this menu has, which is th06's own eight — `Start`, `Extra Start`, `Practice Start`, the
/// replays, the score, the music, the options and the one that ends the game.
const ITEMS: i32 = 8;

/// The state the menu writes when that last item is pressed, which the real game's own `front end:` line
/// carried on the frame the decide went in: 4, where the title is 2.
const ENDING_THE_GAME: i32 = 4;

/// The game at its title menu with its records holding a clear, which is what a laid-out th06 has unless an
/// e2e test says otherwise.
fn game(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.at_the_title_menu();
    game
}

#[test]
fn the_cursor_goes_round_rather_than_stopping_at_either_end() {
    in_its_own_process(|| {
        let game = game("th06-title-round");
        assert_eq!(
            game.image().front_end_now().cursor,
            item::GAME_START,
            "the title came up somewhere other than its first item",
        );
        // Up at the first item, which the real menu answers with its last: the case that walks to
        // `Extra Start` gets there by this wrap, and a menu that stopped at the end would leave that walk
        // pressing at an item it was already on.
        game.press(keys::UP);
        assert_eq!(
            game.image().front_end_now().cursor,
            ITEMS - 1,
            "the cursor did not come round to the last item",
        );
        game.press(keys::DOWN);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::GAME_START,
            "the cursor did not come round to the first item",
        );
    });
}

#[test]
fn a_cancel_puts_the_cursor_on_the_last_item() {
    in_its_own_process(|| {
        let game = game("th06-title-cancel");
        // The six-frame cancel a recipe presses to take orb's own question away, which the real menu answers
        // by putting the cursor on the item that ends the game: every walk those recipes make afterwards
        // starts from there, so a laid-out menu that left the cursor alone walked from somewhere else.
        game.press(keys::X);
        assert_eq!(
            game.image().front_end_now().cursor,
            ITEMS - 1,
            "a cancel at the title left the cursor where it was",
        );
    });
}

#[test]
fn a_decide_on_the_last_item_ends_the_game() {
    in_its_own_process(|| {
        let game = game("th06-title-quit");
        // The item a cancel puts the cursor on, which is the one the case that clears the game walks to: the
        // real game's trace stops ten frames past the step before the decide made there, the process gone
        // with it. A menu that took that decide for one of the items that start a run walked four screens
        // into a run instead.
        game.press(keys::X);
        assert_eq!(
            game.image().front_end_now().cursor,
            ITEMS - 1,
            "the cursor is not on the item that ends the game",
        );
        game.press(keys::Z);
        assert!(
            game.sim().exited().is_some(),
            "a decide on the item that ends the game left the game running",
        );
        // And the state the real menu wrote on that frame, which its own `front end:` line carried: the
        // process going is the frame after, so what the read of that frame finds is a state of its own.
        assert_eq!(
            game.image().front_end_now().screen,
            Screen::Other(ENDING_THE_GAME),
            "the menu did not write the state the real one writes on its way out",
        );
    });
}

#[test]
fn the_item_that_starts_the_extra_is_stepped_over_where_the_records_hold_no_clear() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-title-locked-extra", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        game.its_records_hold_no_clear();
        game.at_the_title_menu();
        // Down from the first item reaches the third, the second being the Extra's and not offered: measured
        // on the real game, whose menu walked 0 to 2 pressing down and 0 to 7 pressing up while its score
        // file held no clear. A menu that rested there would let a walk aiming at the Extra start one the
        // records have not unlocked.
        game.press(keys::DOWN);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::PRACTICE,
            "the cursor rested on the Extra, which this game's records do not offer",
        );
        game.press(keys::UP);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::GAME_START,
            "the cursor came back through the Extra",
        );
    });
}

#[test]
fn the_extra_is_reached_where_they_hold_one() {
    in_its_own_process(|| {
        let game = game("th06-title-unlocked-extra");
        // And the item is there where the records hold a clear, which is what the case after the one that
        // clears the game finds: the same press, the other answer.
        game.press(keys::DOWN);
        assert_eq!(
            game.image().front_end_now().cursor,
            item::EXTRA,
            "the cursor stepped over the Extra of a game that has been cleared",
        );
    });
}
