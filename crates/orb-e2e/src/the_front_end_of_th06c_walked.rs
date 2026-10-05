//! **th06c's front end walked the way the real one walks**: the logo it leaves on its own, the three selects
//! between the title's own Start item and a run, the two an Extra run has instead, the item the title comes
//! back on when a run is left, and the screen the Score item puts up.
//!
//! The same reading as `the_front_end_of_th07_walked`'s, and off the same instrument: `0, 1, 2, 6, 7, 9, 11`
//! is what a launch of the real th06c wrote as its own `front end: screen N, cursor on item M` lines walking
//! into a run, which is what every recipe of `cargo xtask real` is written against. A laid-out game that put
//! a screen somewhere else in that order would part from every trace that reaches it.
//!
//! **And the Extra's two selects are shown under the numbers the other path's first two use** — the
//! character under 7 and the shot under 9 — which is this game's own doing and the reason a recipe aiming at
//! them names the screen: `a_normal_mode_run_of_th06c_played_to_its_clear_and_the_extra_it_unlocked` walks 7
//! and 9 for an Extra run where `a_run_of_th06c_left_in_its_second_chapter` walks 7, 9 and 11 for a Normal
//! one.
//!
//! **The mode is fixed rather than asked here**, which is what `fast_clear` does and the whole reason it is
//! on: orb's own question stands over the title's Start item, and what it is drawn with and answered by is
//! `the_mode_question_over_th06c`'s subject rather than this file's.

use orb_config::LogLevel;
use orb_core::game::th06c::image::{
    RESULT_RANKING, SCREEN_CHARACTER, SCREEN_DIFFICULTY, SCREEN_LOGO, SCREEN_ON_ITS_OWN,
    SCREEN_RANKING, SCREEN_SHOT_SELECT, SCREEN_TITLE,
};
use orb_sim::keys;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a walk here is given before it says the screens went nowhere: each of the screens
/// between an item and a run ignores a press for its own first frames, and the one between the title and
/// them all takes none at all.
const FRAMES: u32 = 1_200;

/// The screen a launch begins on, before its front end has been built at all.
const SCREEN_NOTHING: i32 = 0;

/// The title's own items, as the cursor counts them — `Th06c`'s own `front_end::ITEM_*`, which that module
/// keeps to itself.
const ITEM_START: i32 = 0;
const ITEM_EXTRA: i32 = 1;
const ITEM_SCORE: i32 = 4;

/// The stage word an Extra run is played under, which is what says the walk reached one.
const THE_EXTRAS_STAGE_WORD: i32 = 6;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = true;
            config.log_level = LogLevel::Verbose;
        },
    )
}

/// **Every screen the front end came to, in the order it came to them**, with `key` tapped every other frame
/// and the walk stopped at `until`.
///
/// Tapped rather than held, because every screen of this game reads its decide off the edge of the word —
/// and read every frame rather than after each tap, because the screen between the title and the selects
/// takes no press at all and would be walked straight past by a poll that only looked when a key went down.
fn walked(game: &Fake, key: Option<u8>, until: impl Fn() -> bool) -> Vec<i32> {
    let mut seen = vec![game.image().the_screen_the_front_end_is_on()];
    let mut down = false;
    for _ in 0..FRAMES {
        if until() {
            break;
        }
        if let Some(key) = key {
            down = !down;
            game.keyboard().set(key, down);
        }
        game.frame();
        let screen = game.image().the_screen_the_front_end_is_on();
        if screen != *seen.last().expect("a screen to have been on") {
            seen.push(screen);
        }
    }
    if let Some(key) = key {
        game.keyboard().set(key, false);
    }
    seen
}

/// **Whether the game is in a stage and not merely on its way into one**: the shot select's own decide asks
/// the supervisor for the run's scene and the stage is built on the frame after it, so a walk that stopped at
/// the ask would stop with the stage word still nought — `State::stage` being that word less one, and so -1.
fn in_a_stage(game: &Fake) -> bool {
    let state = game.state();
    state.playing && state.stage >= 0
}

/// A launch walks its own logo to its title with nothing pressed.
#[test]
fn a_launch_of_th06c_walks_its_logo_to_its_title_on_its_own() {
    in_its_own_process(|| {
        let game = launched("th06c-the-front-end-the-logo");
        let seen = walked(&game, None, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        assert_eq!(
            seen,
            vec![SCREEN_NOTHING, SCREEN_LOGO, SCREEN_TITLE],
            "the launch did not walk its logo to its title on its own",
        );
    });
}

/// **And a press on the logo is taken only from its thirty-first update**, `0x1e` against the count
/// 0x14002ccf0's logo case raises at the update's end: one landing before it goes unheard, one after it goes
/// to the title.
#[test]
fn a_press_on_th06cs_logo_is_taken_only_once_it_has_been_up_thirty_updates() {
    in_its_own_process(|| {
        let game = launched("th06c-the-front-end-a-press-on-the-logo");
        // Up for one update short of the thirtieth, so that the press lands on that one.
        game.frames_until("the logo", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_LOGO
                && game.image().frames_the_screen_has_been_up() == LOGO_HEARS_NOTHING_FOR as i32 - 1
        });
        game.press(keys::Z);
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_LOGO,
            "a press on the logo's first updates was taken",
        );
        game.frames(LOGO_HEARS_NOTHING_FOR);
        game.press(keys::Z);
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_TITLE,
            "a press on the logo once it had been up thirty updates went unheard",
        );
    });
}

/// The updates the logo takes no press for.
const LOGO_HEARS_NOTHING_FOR: u32 = 0x1e;

/// And the title's own Start item walks the difficulty, the character and the shot into a run.
#[test]
fn the_start_item_of_th06c_walks_three_selects_into_a_run() {
    in_its_own_process(|| {
        let game = launched("th06c-the-front-end-a-run");
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            ITEM_START,
            "the title did not come up on the item that starts a run",
        );

        let seen = walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert_eq!(
            seen,
            vec![
                SCREEN_TITLE,
                SCREEN_ON_ITS_OWN,
                SCREEN_DIFFICULTY,
                SCREEN_CHARACTER,
                SCREEN_SHOT_SELECT
            ],
            "the walk into a run did not go through the three selects in that order",
        );
        assert_eq!(
            game.state().stage,
            0,
            "the shot select's decide did not start a run in the first stage: {}",
            game.state(),
        );
    });
}

/// And the Extra's own item walks two selects and no difficulty, under the numbers the other walk's first
/// two use.
#[test]
fn the_extra_item_of_th06c_walks_two_selects_into_a_run_of_its_own() {
    in_its_own_process(|| {
        let game = launched("th06c-the-front-end-the-extra");
        // The Extra is behind a clear: the records the title reads hold one here, which is what puts that
        // item under a cursor at all — see `the_extra_of_th06c_is_lit_from_the_games_own_records_*`.
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        game.press_until(keys::DOWN, "the cursor on the Extra", || {
            game.image().the_item_the_cursor_is_on() == ITEM_EXTRA
        });

        let seen = walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert_eq!(
            seen,
            vec![
                SCREEN_TITLE,
                SCREEN_ON_ITS_OWN,
                SCREEN_DIFFICULTY,
                SCREEN_CHARACTER
            ],
            "the Extra's walk did not go through two selects under the first two numbers",
        );
        assert_eq!(
            game.image().the_games_stage_word(),
            THE_EXTRAS_STAGE_WORD,
            "the walk did not start a run of the Extra's own stage",
        );
    });
}

/// And the front end rebuilt after a run comes back through this game's own logo.
///
/// **Asked for the way the game asks for it**, which is the supervisor's own state: `0x140050d79` is the
/// screen a run ends at writing one into it, and that is the whole of how a run gets back to the title.
#[test]
fn the_front_end_of_th06c_rebuilt_after_a_run_comes_back_through_its_logo() {
    in_its_own_process(|| {
        let game = launched("th06c-the-front-end-come-back");
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        game.press_until(keys::DOWN, "the cursor on the Extra", || {
            game.image().the_item_the_cursor_is_on() == ITEM_EXTRA
        });
        walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert!(in_a_stage(&game), "the walk never reached a run");

        game.image()
            .is_in_scene(game.image().scene_of_the_front_end(), {
                game.image().the_scene_last_acted_on()
            });
        let seen = walked(&game, None, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        // The first of them is not a screen this walk went to: the field still held the select the run was
        // started from, nothing having written it since.
        assert_eq!(
            &seen[1..],
            [SCREEN_LOGO, SCREEN_TITLE],
            "the front end came back somewhere other than through this game's own logo",
        );
        // **And the cursor is where orb's own walk left it rather than where the run was started from.** A
        // run that ended anywhere but the screen a run ends at is taken through the game's own ranking so
        // that what it counted is written, and reaching that ranking is the cursor moved to the item it sits
        // behind: measured, the walk above left the title on its Extra item and the cursor read nought here,
        // with five updates of orb's between them.
        assert!(
            game.log().said("score: the ranking built and taken down"),
            "the run that was left was not taken through the game's own ranking:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// And the Score item puts this game's own ranking up, which is the screen orb has it build a file from.
#[test]
fn the_score_item_of_th06c_puts_its_own_ranking_up() {
    in_its_own_process(|| {
        let game = launched("th06c-the-front-end-the-ranking");
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        game.press_until(keys::DOWN, "the cursor on the Score item", || {
            game.image().the_item_the_cursor_is_on() == ITEM_SCORE
        });
        game.press_until(keys::Z, "the ranking up", || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_RANKING
        });
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_RANKING,
            "the Score item put another screen up",
        );
    });
}

/// **And the ranking is a menu of seven, left by its last item**, as 0x14004ff40 walks it: up 0x14 updates
/// before its menu takes a key, a decide on its first item opening a page of it rather than leaving, a cancel
/// putting the cursor on the seventh, and a decide there going 0x3c updates later to the front end.
#[test]
fn the_ranking_of_th06c_is_left_by_its_seventh_item() {
    in_its_own_process(|| {
        let game = launched("th06c-the-ranking-left");
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        game.press_until(keys::DOWN, "the cursor on the Score item", || {
            game.image().the_item_the_cursor_is_on() == ITEM_SCORE
        });
        game.press_until(keys::Z, "the Score item's screen", || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_RANKING
        });
        game.frames_until("the ranking's menu", FRAMES, || {
            game.image().the_scene_now() == game.image().scene_of_the_ranking()
                && game.image().the_state_the_screen_a_run_ends_at_is_in() == RESULT_RANKING
        });
        game.frames(A_MENU_TAKES_A_KEY_AFTER);
        game.press(keys::Z);
        assert_ne!(
            game.image().the_scene_now(),
            game.image().scene_of_the_front_end(),
            "a decide on the ranking's first item left it",
        );
        game.frames_until("the ranking's menu again", FRAMES, || {
            game.image().the_state_the_screen_a_run_ends_at_is_in() == RESULT_RANKING
        });
        game.press(keys::X);
        assert_eq!(
            game.image().the_characters_typed(),
            THE_WAY_OUT,
            "the cancel did not move the cursor"
        );
        game.press(keys::Z);
        game.frames_until("the front end", THE_WAY_OUT_TAKES + 2, || {
            game.image().the_scene_now() == game.image().scene_of_the_front_end()
        });
    });
}

/// The updates the ranking's menu takes no key for, its item that leaves, and the updates leaving takes.
const A_MENU_TAKES_A_KEY_AFTER: u32 = 0x14;
const THE_WAY_OUT: i32 = 6;
const THE_WAY_OUT_TAKES: u32 = 0x3c;
