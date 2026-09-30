//! **The title of th10 walked from the screen that waits for a key into a run, by the screen numbers a recipe
//! of this game presses at.**
//!
//! `Game::front_end_screen` answers the title's `+0x1c` and `front_end_cursor` its `+0x24`, and the numbers
//! those come back with are what every recipe of this game is written against: 1 is the screen that waits for
//! a key, 2 the main menu, whose item 0 starts a run; 6 the difficulty select, which comes up on Normal; 7 the
//! character select and 8 the shot select, whose decide asks for the run (0x430f5a). `orb-core`'s
//! `game/th10/asm.md` has where each was read.
//!
//! **Which is what an e2e test over this game can say about the title and the whole of it**: what a press does
//! on each of those screens is the laid-out game's, so nothing here holds the real game to a screen it does not
//! have. What it does hold is orb — that a recipe's `until: { screen: 6 }` is answered off the field that screen
//! is really in, that a cursor pressed to is the cursor orb reads, and that a decide at the last of them is a
//! run orb sees start. The same claims `the_front_end_of_th08_walked.rs` makes of that game, but for two:
//!
//! - **No front end pointed at after the run is built**: th08 never clears its pointer and frees the object
//!   under it, so orb has to decline a screen read off it by the scene. th10's switch takes the title down on
//!   its way into a run (0x421a03), and the title's destructor writes nought over `0x47784c` (0x42cd25) — so
//!   what is asserted here is that pointer gone, and nothing read.
//! - **The frame the run is asked for reads the title's own screen 3**, where th08's reads nothing: the shot
//!   select's decide puts the title on the screen it leaves on before it asks for the run (0x430f0a through
//!   0x430f5a), and the title is a live object until the switch frees it on the next walk — before anything of
//!   orb's reads it again. th08's gate exists for a freed object whose head page was still committed, which
//!   this game never leaves standing.

use crate::fake::th10::{DIFFICULTY_NORMAL, Fake, THE_ITEM_FOR_A_PRACTICE, THE_ITEM_FOR_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th10::Th10;
use orb_core::game::th10::image::{Scene, Screen};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a screen is given to answer a press, which is well past the forty the shot select's decide
/// takes to ask for the run (0x430ee3) and the fifty-five a stage's build runs for.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// The key this game reads as *decide* and the one it reads as down, as its own fold has them: `Z` gives bit
/// `0x1` and the down arrow bit `0x20` (0x44a63c through 0x44a74f).
const DECIDE: u8 = keys::Z;
const DOWN: u8 = keys::DOWN;

fn at_its_title(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    // Left to itself rather than placed, which is the launch a recipe's first step presses through: the first
    // title of a launch comes up on the screen that waits for a key (0x42cff2), a key there puts the main menu
    // up thirty frames later (0x42d3ef), and the main menu reads none for its first ten (0x42d4ca).
    game.frames_until(
        "the screen that waits for a key",
        FRAMES_FOR_A_SCREEN,
        || (unsafe { Th10.front_end_screen() }) == Some(Screen::PressAKey as i32),
    );
    game.press_until(DECIDE, "the main menu came up", || {
        (unsafe { Th10.front_end_screen() }) == Some(Screen::MainMenu as i32)
    });
    game.frames_until(
        "the main menu taking a press",
        FRAMES_FOR_A_SCREEN,
        || unsafe { Th10.menu_takes_a_press() },
    );
    game
}

/// The screen that is up and the item under its cursor, as orb reads the pair.
fn where_it_is() -> (Option<i32>, Option<i32>) {
    unsafe { (Th10.front_end_screen(), Th10.front_end_cursor()) }
}

/// Decides until the screen is the one wanted: a select reads no key for its first six frames (0x4303fd,
/// 0x4307d4, 0x430d3e), so a decide as it comes up is no press, and a recipe's step presses until.
fn decides_into(game: &Fake, screen: Screen) {
    let wanted = screen as i32;
    game.press_until(DECIDE, &format!("screen {wanted} came up"), || {
        (unsafe { Th10.front_end_screen() }) == Some(wanted)
    });
}

/// **The screens walked, and the run the last of them starts.**
#[test]
fn a_decide_at_each_screen_of_th10s_title_walks_into_a_run() {
    in_its_own_process(|| {
        let game = at_its_title("th10-the-front-end-walked");

        // The main menu, on the item a run is behind.
        assert_eq!(
            where_it_is(),
            (Some(Screen::MainMenu as i32), Some(THE_ITEM_FOR_A_RUN)),
            "the walk did not come up at the main menu with the cursor on item 0",
        );
        assert!(
            unsafe { Th10.menu_takes_a_press() },
            "the main menu is up and orb reads it as taking no press",
        );

        // **And orb's own question answered before the walk goes anywhere**, which is what `Th10::rewinds`
        // answering true puts over this item: the first decide at a run is held back and asked about, and the
        // answer hands it on to the menu — which is what walks it into the difficulty select. The same steps
        // every recipe of this game carries — see `recipes/th10/a_run_started_and_given_up.yaml`.
        let log = game.log();
        game.press_until(DECIDE, "the mode question up", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        game.press_until(DECIDE, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        game.frames_until("the difficulty select", FRAMES_FOR_A_SCREEN, || {
            (unsafe { Th10.front_end_screen() }) == Some(Screen::Difficulty as i32)
        });
        assert_eq!(
            where_it_is(),
            (Some(Screen::Difficulty as i32), Some(DIFFICULTY_NORMAL)),
            "the difficulty select did not come up on Normal",
        );
        // **And no press held back over it**, which is `Th10::menu_takes_a_press` answering false for every
        // screen but the main menu: what a press does on this one is the game's, so orb holds none.
        assert!(
            !unsafe { Th10.menu_takes_a_press() },
            "orb holds a press back over a screen of this game it asks nothing on",
        );

        decides_into(&game, Screen::Character);
        assert_eq!(
            where_it_is(),
            (Some(Screen::Character as i32), Some(0)),
            "the character select did not come up with its cursor on the first character",
        );
        decides_into(&game, Screen::Shot);
        assert_eq!(
            where_it_is(),
            (Some(Screen::Shot as i32), Some(0)),
            "the shot select did not come up with its cursor on the first shot",
        );

        // The decide that asks for the run, which is what `Th10::run_chosen` reads — the title's scene with the
        // run's wanted — and on that frame the title still built, on the screen it leaves on.
        game.press_until(DECIDE, "the run asked for", || unsafe { Th10.run_chosen() });
        assert_eq!(
            where_it_is(),
            (Some(Screen::Leaving as i32), Some(0)),
            "the frame the run is asked for does not read the title's own screen it leaves on",
        );
        game.frames_until("the run started", FRAMES_FOR_A_SCREEN, || {
            game.state().in_run
        });
        // And the stage built, which runs on a loader thread of its own — see `Th10::a_stage_has_just_been_built`.
        game.frames_until("the stage built", FRAMES_FOR_A_SCREEN, || {
            !game.image().the_stage_is_being_built()
        });

        let run = game.state();
        assert_eq!(
            (run.scene, run.wanted),
            (Scene::Run as i32, Scene::Run as i32),
            "the scene the walk ended in is not the one a run is played in",
        );
        assert_eq!(
            run.difficulty, DIFFICULTY_NORMAL,
            "the run started is not on the difficulty the select was left on",
        );
        // **And nothing of a screen read in the run**, the title gone with its pointer.
        assert!(
            !game.image().the_title_is_up(),
            "the laid-out run left the title pointed at, which the real game's destructor clears",
        );
        assert_eq!(
            where_it_is(),
            (None, None),
            "orb read a screen and a cursor in a run, with no title up",
        );
    });
}

/// **And a direction moves the cursor**, which is what a recipe's `until: { cursor: 0 }` is answered off.
#[test]
fn a_direction_at_th10s_main_menu_moves_the_cursor_orb_reads() {
    in_its_own_process(|| {
        let game = at_its_title("th10-the-cursor-moved");

        // **To the practice and not the Extra**, a score object with nothing cleared leaving the Extra's item
        // out of the menu (0x42d442 through 0x42d45c) and the cursor stepping over it.
        game.press(DOWN);
        assert_eq!(
            unsafe { Th10.front_end_cursor() },
            Some(THE_ITEM_FOR_A_PRACTICE),
            "a press of down did not move the cursor orb reads past the item nothing has lit",
        );

        // **And the screen it is on with it**, which is the other half of the pair: a cursor read off a screen
        // the walk had left would be a recipe's `until: { cursor: 0 }` answered about the wrong list.
        assert_eq!(
            unsafe { Th10.front_end_screen() },
            Some(Screen::MainMenu as i32),
            "a press of down left the main menu",
        );
    });
}

/// The difficulty, the character and the shot the shot select is left on: Hard, Marisa and her third — none
/// of them the first, so a run read at nought cannot pass for it.
const A_DIFFICULTY: i32 = 2;
const A_CHARACTER: i32 = 1;
const A_SHOT: i32 = 2;
/// And the Extra's difficulty and the stage it is played on, counted from nought: the shot select writes the
/// stage word 7 where the difficulty is 4 or more (0x430f16, 0x430f3f through 0x430f4f).
const DIFFICULTY_EXTRA: i32 = 4;
const THE_EXTRAS_STAGE: i32 = 6;
/// The shot select's count of items, three (0x430a7b).
const SHOTS: i32 = 3;

/// **The run the shot select points at, which is the mark that says a run of it was left unfinished** — and
/// the one screen of this game that knows: the cursor there *is* the shot, and the difficulty and the character
/// are where the two screens before it wrote them (0x4305cb, 0x430997).
#[test]
fn the_run_th10s_shot_select_points_at_is_the_cursor_and_what_the_selects_before_it_chose() {
    in_its_own_process(|| {
        let game = at_its_title("th10-the-run-pointed-at");

        // Nothing at the main menu, which is a screen that knows none of it.
        assert!(
            unsafe { Th10.run_pointed_at() }.is_none(),
            "orb reads a run being pointed at on the main menu",
        );

        let image = game.image();
        image.the_difficulty_is(A_DIFFICULTY);
        image.the_character_is(A_CHARACTER);
        image.the_title_is_on(Screen::Shot);
        image.the_titles_cursor_is(A_SHOT, SHOTS);

        let pointed = unsafe { Th10.run_pointed_at() }.expect("the shot select knows which run");
        assert_eq!(
            (
                pointed.difficulty,
                pointed.character,
                pointed.shot_type,
                pointed.stage
            ),
            (A_DIFFICULTY, A_CHARACTER, A_SHOT, 0),
            "the run the shot select points at is not the cursor and the selects before it",
        );
        assert!(
            !pointed.practice,
            "the run a whole-run flow points at reads as a practice",
        );

        // **And the Extra's stage where the difficulty is the Extra's**, which is the stage the decide there
        // would start.
        image.the_difficulty_is(DIFFICULTY_EXTRA);
        let extra = unsafe { Th10.run_pointed_at() }.expect("the shot select knows which run");
        assert_eq!(
            (extra.difficulty, extra.stage),
            (DIFFICULTY_EXTRA, THE_EXTRAS_STAGE),
            "the run pointed at over the Extra is not the Extra's stage",
        );

        // And nothing again on the character select, where the shot has not been chosen.
        image.the_title_is_on(Screen::Character);
        assert!(
            unsafe { Th10.run_pointed_at() }.is_none(),
            "orb reads a run being pointed at a screen before the shot is chosen",
        );
    });
}
