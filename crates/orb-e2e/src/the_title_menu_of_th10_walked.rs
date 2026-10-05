//! **th10's main menu walked the way the real one walks**: onto the item that ends the game at a cancel, round
//! its eight, over the Extra where its records have not lit it, and deaf to a press while it builds itself.
//!
//! The same reading as `the_title_menu_of_th08_walked`'s and for the same reason: every recipe of `cargo xtask
//! real` walks this menu by pressing at it, so a laid-out menu that lands on another item reaches a screen the
//! case is not about. Where each number comes from:
//!
//! - **Eight items**: step 0 writes 8 for the count at `+0x2c` (0x42d43b), which 0x44bea0 takes round at either
//!   end.
//! - **The cancel onto the last of them**, which 0x42d5c0 through 0x42d5dc writes as the count less one — and a
//!   cancel *on* it is that item's decide, the step the title leaves by written (0x42d5a5, 0x42d700 through
//!   0x42d718), where th08's second cancel leaves the cursor where it is.
//! - **One item stepped over where the records have not lit it**: the Extra, off 0x42c850 (0x42d442 through
//!   0x42d45c). th10 has no spell practice for a second.
//! - **And a build before any of it is read**: step 0 builds the menu and puts it in step 1 (0x42d4be), which waits
//!   until the screen's frames are past ten (0x42d4ca) before step 2 (0x42d4d7) — the step the arrows, the cancel
//!   and the decide are all read in (0x42d516 through 0x42d609). So a press is read from the menu's thirteenth
//!   update on, and the cancel from the same one: th08's later gate on the cancel has no counterpart, this menu
//!   reading every key in the one step.
//!
//! **What is pressed here is a direction and a cancel and never a decide**: a decide on the first item is what
//! raises orb's own question about the mode — `the_mode_question_over_th10`'s subject.

use orb_config::LogLevel;
use orb_core::game::th10::image::{STEP_LEFT, STEP_READING, Screen};
use orb_core::game::th10::score_file::{Records, settings};
use orb_sim::keys;

use crate::fake::th10::{Fake, THE_ITEM_FOR_A_PRACTICE, THE_ITEM_FOR_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a launch is given to bring a screen of its title up.
const FRAMES: u32 = 600;

/// How many items this menu has, and the two of them this file names past the fake's own: written out here
/// rather than taken from the fake's own count, a test that read its numbers off the laid-out game agreeing with
/// it whatever either said.
const ITEMS: i32 = 8;
const THE_EXTRA: i32 = 1;
const THE_WAY_OUT: i32 = ITEMS - 1;

/// **The first update the menu reads a key on**, counted from the update it is built on as its first.
const A_KEY_FROM: u32 = 13;

/// A score object whose clears light the Extra, which is any of its bytes for it — the settings' fill being one
/// of them.
const A_CLEAR: u8 = 1;

/// The main menu up and past its build, placed rather than walked to.
fn at_the_main_menu(name: &str, the_extra_lit: bool) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    if the_extra_lit {
        let own = Records {
            settings: Some(settings(A_CLEAR)),
            ..Records::default()
        };
        game.its_score_files_hold(&own, &Records::default());
    }
    game.its_title_menu_is_up();
    game
}

/// **A launch left to itself, stopped on the update that put the main menu up**: the screen that waits for a key
/// pressed past as it reads one, which puts the menu up thirty frames later (0x42d3ef). The update after this is
/// the menu's first, the one it is built on.
fn as_the_main_menu_comes_up(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.frames_until(
        "the screen that waits for a key reading one",
        FRAMES,
        || {
            let (screen, step, _, _) = game.image().where_the_title_is();
            game.image().the_title_is_up()
                && screen == Screen::PressAKey as i32
                && step == STEP_READING
        },
    );
    game.press(keys::Z);
    game.frames_until("the main menu", FRAMES, || {
        game.image().where_the_title_is().0 == Screen::MainMenu as i32
    });
    game
}

/// The item the menu's cursor is on, as the title keeps it at `+0x24`.
fn the_cursor(game: &Fake) -> i32 {
    game.image().the_titles_cursor().0
}

/// `key` tapped at a menu past its build, and the item the cursor is on after it.
fn after(game: &Fake, key: u8) -> i32 {
    game.press(key);
    the_cursor(game)
}

/// `key` down for the one update that is the menu's `update`th and up for the next, `now` the updates of it
/// already gone.
fn taps_on_its_update(game: &Fake, key: u8, update: u32, now: &mut u32) {
    game.frames(update - 1 - *now);
    game.keyboard().set(key, true);
    game.frame();
    game.keyboard().set(key, false);
    game.frame();
    *now = update + 1;
}

/// A cancel puts the cursor on the item that ends the game, and a second one is that item's decide.
#[test]
fn a_cancel_at_th10s_main_menu_puts_the_cursor_on_the_item_that_ends_the_game() {
    in_its_own_process(|| {
        let game = at_the_main_menu("th10-title-cancel", false);
        assert_eq!(
            the_cursor(&game),
            THE_ITEM_FOR_A_RUN,
            "the menu did not come up on its first item",
        );
        assert_eq!(
            after(&game, keys::X),
            THE_WAY_OUT,
            "the cancel did not put the cursor on the item that ends the game",
        );
        game.keyboard().set(keys::X, true);
        game.frame();
        game.keyboard().set(keys::X, false);
        let (screen, step, cursor, _) = game.image().where_the_title_is();
        assert_eq!(
            (screen, step, cursor),
            (Screen::MainMenu as i32, STEP_LEFT, THE_WAY_OUT),
            "a second cancel did not take the item that ends the game",
        );
    });
}

/// The frames a direction is held for below, and the moves of the cursor that holding makes: the press, and the
/// auto-repeat 0x44a5f0 keeps at `+0x4` — a bit's own count past 0x19 sets it and takes eight off (0x44a8d2
/// through 0x44a90a), so the 26th frame held repeats and every eighth after it.
const HELD_FOR: u32 = 40;
const MOVES_IT_MAKES: i32 = 3;

/// **A direction held at the main menu moves the cursor on its own**, which the menu reads as `+0x6 | +0x4`
/// (0x42d52b) — so a recipe holding one for more than a press walks further than the one press.
#[test]
fn a_direction_held_at_th10s_main_menu_moves_the_cursor_on_by_itself() {
    in_its_own_process(|| {
        let game = at_the_main_menu("th10-title-held", true);
        assert_eq!(the_cursor(&game), THE_ITEM_FOR_A_RUN);
        game.keyboard().set(keys::DOWN, true);
        game.frames(HELD_FOR);
        game.keyboard().set(keys::DOWN, false);
        assert_eq!(
            the_cursor(&game),
            THE_ITEM_FOR_A_RUN + MOVES_IT_MAKES,
            "the cursor did not repeat where the down was held",
        );
    });
}

/// And the cursor counts round the eight both ways, so up from the first item is the last.
#[test]
fn the_cursor_of_th10s_main_menu_counts_round_its_eight_items() {
    in_its_own_process(|| {
        let game = at_the_main_menu("th10-title-round", true);
        let mut seen = vec![the_cursor(&game)];
        for _ in 0..ITEMS {
            seen.push(after(&game, keys::DOWN));
        }
        assert_eq!(
            seen,
            (0..ITEMS)
                .chain(std::iter::once(THE_ITEM_FOR_A_RUN))
                .collect::<Vec<_>>(),
            "the cursor did not count round the menu's own items",
        );
        assert_eq!(
            after(&game, keys::UP),
            THE_WAY_OUT,
            "up from the first item is not the last",
        );
    });
}

/// And it steps over the Extra where the records have not lit it, both ways.
#[test]
fn the_cursor_of_th10s_main_menu_steps_over_the_extra_nothing_has_lit() {
    in_its_own_process(|| {
        let game = at_the_main_menu("th10-title-nothing-lit", false);
        assert_eq!(
            after(&game, keys::DOWN),
            THE_ITEM_FOR_A_PRACTICE,
            "the cursor stopped on an Extra the records have not lit",
        );
        assert_eq!(
            after(&game, keys::UP),
            THE_ITEM_FOR_A_RUN,
            "the walk back up stopped on the Extra it had stepped over",
        );
    });
}

/// And where the records light the Extra, the cursor stops on it.
#[test]
fn the_cursor_of_th10s_main_menu_stops_on_the_extra_the_records_lit() {
    in_its_own_process(|| {
        let game = at_the_main_menu("th10-title-the-extra", true);
        assert_eq!(
            after(&game, keys::DOWN),
            THE_EXTRA,
            "the cursor stepped over an Extra the records had lit",
        );
    });
}

/// **A direction is read from the menu's thirteenth update and not before**, the twelve before it being its build
/// and the ten frames it waits out.
#[test]
fn a_direction_at_th10s_main_menu_is_read_from_its_thirteenth_update() {
    in_its_own_process(|| {
        let game = as_the_main_menu_comes_up("th10-title-direction-gate");
        let mut now = 0;
        taps_on_its_update(&game, keys::DOWN, A_KEY_FROM - 1, &mut now);
        assert_eq!(
            the_cursor(&game),
            THE_ITEM_FOR_A_RUN,
            "a direction moved the cursor while the menu was still building itself",
        );
        taps_on_its_update(&game, keys::DOWN, A_KEY_FROM + 1, &mut now);
        assert_eq!(
            the_cursor(&game),
            THE_ITEM_FOR_A_PRACTICE,
            "a direction past the build did not move the cursor",
        );
    });
}

/// **And a cancel from the same update**, which th08 holds off for longer: this menu reads every key in the one
/// step.
#[test]
fn a_cancel_at_th10s_main_menu_is_read_from_its_thirteenth_update() {
    in_its_own_process(|| {
        let game = as_the_main_menu_comes_up("th10-title-cancel-gate");
        let mut now = 0;
        taps_on_its_update(&game, keys::X, A_KEY_FROM - 1, &mut now);
        assert_eq!(
            the_cursor(&game),
            THE_ITEM_FOR_A_RUN,
            "a cancel was read while the menu was still building itself",
        );
        taps_on_its_update(&game, keys::X, A_KEY_FROM + 1, &mut now);
        assert_eq!(
            the_cursor(&game),
            THE_WAY_OUT,
            "a cancel past the build did not put the cursor on the item that ends the game",
        );
    });
}
