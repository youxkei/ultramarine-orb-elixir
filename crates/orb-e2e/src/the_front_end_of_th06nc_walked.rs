//! **th06nc's front end walked the way the real one walks**: the logo it leaves on its own, the title's
//! own decide, the selects and the three screens between them and a stage, and the one field each of the
//! three items that start a run carries into the run it starts.
//!
//! Where the order comes from: `1, 2, 3, 8, 9, 11, 13, 14, 24, 25` is what a launch of the real game wrote
//! as its own `front end: screen N` lines walking into a run, which is what every recipe of
//! `cargo xtask real` is written against — see `th06nc/asm.md`'s table of the screens. A laid-out game that
//! put a screen somewhere else in that order would part from every trace that reaches it.
//!
//! **The menu underneath is `the_main_menu_of_th06nc_walked`'s subject** — its ten items, the Extra stepped
//! over while it is locked, and the twenty frames it ignores a press for. What this file is about is the
//! screens either side of it.
//!
//! **And the walk back from a run is its own attract cycle and not the way it came**: `16, 17, 18`, with
//! 18 the replay list, and what leaves that is the cancel. Timed on the real game by
//! `the-front-end-after-a-run.yaml`: 16 came 16ms after `leave: true`, 17 one frame after 16, 18 seven
//! frames after 17, and 21,600 frames of that launch then sat on 18 with nothing pressed.
//!
//! **The mode is fixed rather than asked**, which is what `fast_clear` does and the whole reason it is on:
//! orb's own question stands over the item that starts a run, so a walk that pressed the decide there
//! would have that press held back — what the question is drawn with and answered by is
//! `the_mode_question_over_th06nc`'s subject.

use orb_config::LogLevel;
use orb_core::game::th06nc::image::{
    MENU_ITEM_EXTRA, MENU_ITEM_PRACTICE, MENU_ITEM_RUN, SCREEN_CHARACTER, SCREEN_DIFFICULTY,
    SCREEN_LOGO, SCREEN_MENU, SCREEN_ON_ITS_OWN, SCREEN_SHOT_SELECT, SCREEN_TITLE,
};
use orb_sim::keys;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a walk here is given before it says the screens went nowhere: the logo lasts two
/// seconds, the screen after the menu's item one, and each of the selects and the two screens after them
/// a grace of its own.
const FRAMES: u32 = 1_200;

/// The screen a launch begins on, before its front end has been built at all.
const SCREEN_NOTHING: i32 = 0;

/// **The three screens between the shot select and the stage**, which `th06nc/asm.md`'s table names: 14 the
/// banner, 24 the one screen of the three that takes a press — `0x14004d981`, edge-checking the decide
/// after fifteen frames — and 25 the one that reads no word at all and calls the run's own start,
/// `0x14004dbe9`.
const SCREEN_AFTER_THE_SHOT: i32 = 14;
const SCREEN_OF_THE_LAST_PRESS: i32 = 24;
const SCREEN_THAT_STARTS_THE_RUN: i32 = 25;

/// **The three the front end walks when a run ends** — the attract cycle, 18 being the replay list it
/// comes to rest on — and the item a cancel there leaves the main menu's cursor on: the last of its ten,
/// which a launch of `an_attempt_at_a_card_counted_and_kept` read as the item that ends the game and whose
/// recipe re-aims the cursor at item 0 because of it.
const SCREEN_BACK_FROM_A_RUN: i32 = 16;
const SCREEN_BEFORE_THE_REPLAY_LIST: i32 = 17;
const THE_REPLAY_LIST: i32 = 18;
const THE_ITEM_THAT_ENDS_THE_GAME: i32 = 9;

/// **The difficulty the Extra's own item starts a run at**, which is how a run of it is told from a Normal
/// one: `0x14007bc97` compares the word at `0x14053d410` with four, and `0x14003c345` writes four where the
/// stage is six or past it.
const DIFFICULTY_OF_THE_EXTRA: i32 = 4;

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

/// **Every screen the front end came to, in the order it came to them**, with `key` tapped every other
/// frame and the walk stopped at `until`.
///
/// Tapped rather than held, because every screen of this game reads its decide off the edge of the word —
/// and read every frame rather than after each tap, because the screen the menu's item opens takes no press
/// at all and would be walked straight past by a poll that only looked when a key went down.
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

/// **Whether the game is in a stage and not merely on its way into one**: the last screen before a stage
/// calls the run's own start, and the stage is built on the frame after it — so a walk that stopped at the
/// call would stop with the stage word still nought, `State::stage` being that word less one and so -1.
fn in_a_stage(game: &Fake) -> bool {
    let state = game.state();
    state.playing && state.stage >= 0
}

/// A launch with the front end at its title and the cursor of the menu behind it walked to `item`.
fn at_the_menu(name: &str, item: i32) -> Box<Fake> {
    let game = launched(name);
    game.frames_until("the title", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
    });
    game.press(keys::RETURN);
    game.frames_until("the main menu", FRAMES, || {
        game.image().the_screen_the_front_end_is_on() == SCREEN_MENU
    });
    if game.image().the_item_the_cursor_is_on() != item {
        game.press_until(keys::DOWN, "the cursor on the item walked to", || {
            game.image().the_item_the_cursor_is_on() == item
        });
    }
    game
}

/// The screens a walk out of the main menu goes through on its way into a stage, whichever of the three
/// items that start a run it left the menu on.
const SCREENS_INTO_A_RUN: [i32; 7] = [
    SCREEN_ON_ITS_OWN,
    SCREEN_DIFFICULTY,
    SCREEN_CHARACTER,
    SCREEN_SHOT_SELECT,
    SCREEN_AFTER_THE_SHOT,
    SCREEN_OF_THE_LAST_PRESS,
    SCREEN_THAT_STARTS_THE_RUN,
];

/// **And every screen of that walk has arrived as soon as it is read**, which is what `State::wanted`
/// says: this front end keeps one state field and each screen's own case writes it outright, so there is
/// no screen in flight for a frame the way th06's and th06c's have — those two keep the state acted on
/// beside the state asked for.
///
/// **What rests on it is the frame after a run is given up**: `runtime` commits that run's records on the
/// first frame where `scene` and `wanted` agree, and `Th06nc::read_state` answering `-1` there made them
/// never agree at all.
#[test]
fn every_screen_of_th06ncs_front_end_reads_as_one_it_has_arrived_at() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-front-end-arrived");
        let mut seen = Vec::new();
        let mut down = false;
        for _ in 0..FRAMES {
            let state = game.state();
            assert_eq!(
                state.wanted, state.scene,
                "a screen of the front end read as one still in flight: {state}",
            );
            if seen.last() != Some(&state.scene) {
                seen.push(state.scene);
            }
            if seen.contains(&SCREEN_MENU) {
                break;
            }
            down = !down;
            game.keyboard().set(keys::Z, down);
            game.frame();
        }
        game.keyboard().set(keys::Z, false);
        assert!(
            seen.contains(&SCREEN_MENU),
            "the walk never reached the main menu: {seen:?}",
        );
    });
}

/// A launch walks its own logo to its title with nothing pressed.
#[test]
fn a_launch_of_th06nc_walks_its_logo_to_its_title_on_its_own() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-front-end-the-logo");
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

/// **And the title takes a decide of its own**, which is the press this game's front end has that th06c's
/// has not: there the title *is* the menu, here it is the screen in front of one.
#[test]
fn the_title_of_th06nc_opens_its_main_menu_on_a_decide() {
    in_its_own_process(|| {
        let game = launched("th06nc-the-front-end-the-title");
        game.frames_until("the title", FRAMES, || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_TITLE
        });
        let seen = walked(&game, Some(keys::Z), || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_MENU
        });
        assert_eq!(
            seen,
            vec![SCREEN_TITLE, SCREEN_MENU],
            "the title's decide put another screen up than its main menu",
        );
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            MENU_ITEM_RUN,
            "the menu did not come up on the item that starts a run",
        );
    });
}

/// And the menu's own first item walks the three selects and the three screens after them into a run.
#[test]
fn the_run_item_of_th06nc_walks_its_screens_in_order_into_a_stage() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-the-front-end-a-run", MENU_ITEM_RUN);
        let seen = walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert_eq!(
            seen,
            [&[SCREEN_MENU][..], &SCREENS_INTO_A_RUN[..]].concat(),
            "the walk into a run did not go through the game's own screens in that order",
        );
        assert_eq!(
            game.state().stage,
            0,
            "the last screen before a stage did not start a run in the first stage: {}",
            game.state(),
        );
    });
}

/// **And the Extra's own item walks the same screens**, its difficulty written as the screen the item opens
/// goes by rather than chosen at the select.
#[test]
fn the_extra_item_of_th06nc_walks_the_same_screens_into_a_run_of_its_own() {
    in_its_own_process(|| {
        // The Extra is behind a clear: the records this game's copy holds have one, which is what puts that
        // item under a cursor at all — see `the_main_menu_of_th06nc_walked`.
        let game = at_the_menu("th06nc-the-front-end-the-extra", MENU_ITEM_EXTRA);
        let seen = walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert_eq!(
            seen,
            [&[SCREEN_MENU][..], &SCREENS_INTO_A_RUN[..]].concat(),
            "the Extra's walk did not go through the same screens in the same order",
        );
        assert_eq!(
            game.image().the_difficulty_now(),
            DIFFICULTY_OF_THE_EXTRA,
            "the Extra's item started a run at another difficulty",
        );
    });
}

/// **And the practice item's own run reads as a practice**, which is the one field an item of this menu
/// carries into the run it starts: `0x14004f320` copies the front end's `+0x168d1` into `0x14053d404` as a
/// run starts, and that is what `Th06nc::read_state` answers `practice` off.
#[test]
fn the_practice_item_of_th06nc_starts_a_run_the_game_reads_as_a_practice() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-the-front-end-a-practice", MENU_ITEM_PRACTICE);
        walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert!(
            in_a_stage(&game),
            "the practice item's walk reached no stage"
        );
        assert!(
            game.state().practice,
            "the run the practice item started does not read as a practice: {}",
            game.state(),
        );
    });
}

/// **And the front end rebuilt after a run comes back through this game's own attract cycle**, not through
/// the screens the run was started from: 16, 17 and 18 in that order, and 18 is where it rests.
///
/// **Asked for the way the game asks for it**, which is the supervisor's own state: `0x14007bb13` takes the
/// stage's elements out and asks for state 1 — the front end — and that is the whole of how a run gets back
/// to one.
#[test]
fn the_front_end_of_th06nc_rebuilt_after_a_run_walks_its_attract_cycle() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-the-front-end-come-back", MENU_ITEM_RUN);
        walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert!(in_a_stage(&game), "the walk never reached a run");

        game.image().is_in_scene(game.image().the_scene_now(), {
            game.image().scene_of_the_front_end()
        });
        let seen = walked(&game, None, || {
            game.image().the_screen_the_front_end_is_on() == THE_REPLAY_LIST
        });
        // **The first of them is not a screen this walk went to**: the field still held the last screen
        // before the stage, nothing of a run writing it — which is what the real game does too, a launch
        // reading `front end: screen 25` and then nothing at all until 16 a whole run later.
        assert_eq!(
            seen.first().copied(),
            Some(SCREEN_THAT_STARTS_THE_RUN),
            "a run wrote the front end's own state while it was playing",
        );
        assert_eq!(
            &seen[1..],
            [
                SCREEN_BACK_FROM_A_RUN,
                SCREEN_BEFORE_THE_REPLAY_LIST,
                THE_REPLAY_LIST
            ],
            "the front end came back somewhere other than through this game's attract cycle",
        );

        // And it stays on the replay list with nothing pressed, which is what the real one does for
        // 21,600 frames.
        game.frames(600);
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            THE_REPLAY_LIST,
            "the replay list handed on to another screen with nothing pressed",
        );
    });
}

/// **And the cancel is what leaves it**, which puts the main menu up with its cursor on the item that ends
/// the game — the reason `an_attempt_at_a_card_counted_and_kept` re-aims the cursor after tapping its way
/// back rather than pressing on.
#[test]
fn the_cancel_over_th06ncs_attract_cycle_puts_its_main_menu_up() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-the-front-end-cancelled-back", MENU_ITEM_RUN);
        walked(&game, Some(keys::Z), || in_a_stage(&game));
        game.image().is_in_scene(game.image().the_scene_now(), {
            game.image().scene_of_the_front_end()
        });
        walked(&game, None, || {
            game.image().the_screen_the_front_end_is_on() == THE_REPLAY_LIST
        });

        game.press_until(keys::X, "the main menu back up", || {
            game.image().the_screen_the_front_end_is_on() == SCREEN_MENU
        });
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            THE_ITEM_THAT_ENDS_THE_GAME,
            "the menu the cancel put up came up on another item",
        );
    });
}

/// **Where the item that starts a plain run clears the same byte**, so that a practice walked to once does
/// not make every run after it read as one.
#[test]
fn the_run_item_of_th06nc_starts_a_run_the_game_reads_as_no_practice() {
    in_its_own_process(|| {
        let game = at_the_menu("th06nc-the-front-end-no-practice", MENU_ITEM_RUN);
        // The byte a practice walked to before this one would have left set.
        game.image().the_item_asks_for_a_practice(true);
        walked(&game, Some(keys::Z), || in_a_stage(&game));
        assert!(in_a_stage(&game), "the walk reached no stage");
        assert!(
            !game.state().practice,
            "the run the first item started reads as a practice: {}",
            game.state(),
        );
    });
}
