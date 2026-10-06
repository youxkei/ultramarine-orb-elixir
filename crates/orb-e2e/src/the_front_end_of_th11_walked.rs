//! **The title of th11 walked from its main menu into a run, by the screen numbers a recipe of this game presses
//! at.**
//!
//! `Game::front_end_screen` answers the title's `+0x1c` and `front_end_cursor` its `+0x24`, and the numbers
//! those come back with are what every recipe of this game is written against: 1 is the main menu, whose item 0
//! starts a run; 5 the difficulty select, which comes up on Normal; 6 the character select and 7 the partner
//! select, whose decide asks for the run (0x43d5fc). `orb-core`'s `game/th11/asm.md` has where each was read.
//!
//! **Which is what an e2e test over this game can say about the title and the whole of it**: what a press does
//! on each of those screens is the laid-out game's, so nothing here holds the real game to a screen it does not
//! have. What it does hold is orb — that a recipe's `until: { screen: 5 }` is answered off the field that screen
//! is really in, that a cursor pressed to is the cursor orb reads, and that a decide at the last of them is a
//! run orb sees start. The same claims `the_front_end_of_th10_walked.rs` makes of that game, but for one:
//!
//! - **No screen that waits for a key**: th11's first title comes up on its main menu, screen 0's set-up putting
//!   screen 1 up (`[0x4c3250]` nought), and the main menu reads no key until its frames are past `0x50`
//!   (0x43a22d).
//!
//! **And the two th10's file asks over its exit, asked of this one**: the switch takes the title down on its way
//! into a run (`0x439970`, at 0x42b220), whose destructor writes nought over `0x4a8ecc` (0x4398c6) — so what is
//! asserted here is that pointer gone, and nothing read; and **the frame the run is asked for reads the title's own
//! screen 2**: the partner select's decide puts the title on screen 2 before it asks for the run (0x43d5ae through
//! 0x43d5fc), and the title is a live object until the switch frees it on the next walk.

use crate::fake::th11::{DIFFICULTY_NORMAL, Fake, THE_ITEM_FOR_A_PRACTICE, THE_ITEM_FOR_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::Game;
use orb_core::game::th11::Th11;
use orb_core::game::th11::image::{Scene, Screen};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames a screen is given to answer a press, which is well past the forty the partner select's decide
/// takes to ask for the run (0x43d57a) and the fifty-five a stage's build is given here.
const FRAMES_FOR_A_SCREEN: u32 = 240;

/// The key this game reads as *decide* and the one it reads as down, as its own fold has them: `Z` gives bit
/// `0x1` and the down arrow bit `0x20` (0x4576ee through 0x457848).
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
    // title of a launch comes up on its main menu, which reads no key for its first eighty frames (0x43a22d).
    game.frames_until("the main menu", FRAMES_FOR_A_SCREEN, || {
        (unsafe { Th11.front_end_screen() }) == Some(Screen::MainMenu as i32)
    });
    game.frames_until(
        "the main menu taking a press",
        FRAMES_FOR_A_SCREEN,
        || unsafe { Th11.menu_takes_a_press() },
    );
    game
}

/// The screen that is up and the item under its cursor, as orb reads the pair.
fn where_it_is() -> (Option<i32>, Option<i32>) {
    unsafe { (Th11.front_end_screen(), Th11.front_end_cursor()) }
}

/// Decides until the screen is the one wanted: a select reads no key for its first six frames (0x43c8b6,
/// 0x43cd27, 0x43d3a3), so a decide as it comes up is no press, and a recipe's step presses until.
fn decides_into(game: &Fake, screen: Screen) {
    let wanted = screen as i32;
    game.press_until(DECIDE, &format!("screen {wanted} came up"), || {
        (unsafe { Th11.front_end_screen() }) == Some(wanted)
    });
}

/// **The screens walked, and the run the last of them starts.**
#[test]
fn a_decide_at_each_screen_of_th11s_title_walks_into_a_run() {
    in_its_own_process(|| {
        let game = at_its_title("th11-the-front-end-walked");

        // The main menu, on the item a run is behind.
        assert_eq!(
            where_it_is(),
            (Some(Screen::MainMenu as i32), Some(THE_ITEM_FOR_A_RUN)),
            "the walk did not come up at the main menu with the cursor on item 0",
        );
        assert!(
            unsafe { Th11.menu_takes_a_press() },
            "the main menu is up and orb reads it as taking no press",
        );

        // **And orb's own question answered before the walk goes anywhere**, which is what `Th11::rewinds`
        // answering true puts over this item: the first decide at a run is held back and asked about, and the
        // answer hands it on to the menu — which is what walks it into the difficulty select. The same steps
        // every recipe of this game carries — see `recipes/th11/a_run_started_and_given_up.yaml`.
        let log = game.log();
        game.press_until(DECIDE, "the mode question up", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });
        game.press_until(DECIDE, "the mode question answered", || {
            log.said("mode: answered on the keyboard")
        });
        game.frames_until("the difficulty select", FRAMES_FOR_A_SCREEN, || {
            (unsafe { Th11.front_end_screen() }) == Some(Screen::Difficulty as i32)
        });
        assert_eq!(
            where_it_is(),
            (Some(Screen::Difficulty as i32), Some(DIFFICULTY_NORMAL)),
            "the difficulty select did not come up on Normal",
        );
        // **And no press held back over it**, which is `Th11::menu_takes_a_press` answering false for every
        // screen but the main menu: what a press does on this one is the game's, so orb holds none.
        assert!(
            !unsafe { Th11.menu_takes_a_press() },
            "orb holds a press back over a screen of this game it asks nothing on",
        );

        decides_into(&game, Screen::Character);
        assert_eq!(
            where_it_is(),
            (Some(Screen::Character as i32), Some(0)),
            "the character select did not come up with its cursor on the first character",
        );
        decides_into(&game, Screen::Partner);
        assert_eq!(
            where_it_is(),
            (Some(Screen::Partner as i32), Some(0)),
            "the partner select did not come up with its cursor on the first partner",
        );

        // The decide that asks for the run, which is what `Th11::run_chosen` reads — the title's scene with the
        // run's wanted — and on that frame the title still built, on the screen it leaves on.
        game.press_until(DECIDE, "the run asked for", || unsafe { Th11.run_chosen() });
        assert_eq!(
            where_it_is(),
            (Some(Screen::Leaving as i32), Some(0)),
            "the frame the run is asked for does not read the title's own screen it leaves on",
        );
        game.frames_until("the run started", FRAMES_FOR_A_SCREEN, || {
            game.state().in_run
        });
        // And the stage built, which runs on a loader thread of its own — see `Th11::a_stage_has_just_been_built`.
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
fn a_direction_at_th11s_main_menu_moves_the_cursor_orb_reads() {
    in_its_own_process(|| {
        let game = at_its_title("th11-the-cursor-moved");

        // **To the practice and not the Extra**, a score object with no ending seen leaving the Extra's item
        // out of the menu (0x43a0c3 through 0x43a0de) and the cursor stepping over it.
        game.press(DOWN);
        assert_eq!(
            unsafe { Th11.front_end_cursor() },
            Some(THE_ITEM_FOR_A_PRACTICE),
            "a press of down did not move the cursor orb reads past the item nothing has lit",
        );

        // **And the screen it is on with it**, which is the other half of the pair: a cursor read off a screen
        // the walk had left would be a recipe's `until: { cursor: 0 }` answered about the wrong list.
        assert_eq!(
            unsafe { Th11.front_end_screen() },
            Some(Screen::MainMenu as i32),
            "a press of down left the main menu",
        );
    });
}

/// The difficulty, the character and the partner the partner select is left on: Hard, Marisa and her third —
/// none of them the first, so a run read at nought cannot pass for it.
const A_DIFFICULTY: i32 = 2;
const A_CHARACTER: i32 = 1;
const A_PARTNER: i32 = 2;
/// And the Extra's difficulty and the stage it is played on, counted from nought: the partner select writes the
/// stage 7 where the difficulty is 4 (0x43d5b3 through 0x43d5ec).
const DIFFICULTY_EXTRA: i32 = 4;
const THE_EXTRAS_STAGE: i32 = 6;
/// The partner select's count of items, three.
const PARTNERS: i32 = 3;

/// **The run the partner select points at, which is the mark that says a run of it was left unfinished** — and
/// the one screen of this game that knows: the cursor there *is* the partner, which is the shot type, and the
/// difficulty and the character are where the two screens before it wrote them (0x43cb11, 0x43cfb6).
#[test]
fn the_run_th11s_partner_select_points_at_is_the_cursor_and_what_the_selects_before_it_chose() {
    in_its_own_process(|| {
        let game = at_its_title("th11-the-run-pointed-at");

        // Nothing at the main menu, which is a screen that knows none of it.
        assert!(
            unsafe { Th11.run_pointed_at() }.is_none(),
            "orb reads a run being pointed at on the main menu",
        );

        let image = game.image();
        image.the_difficulty_is(A_DIFFICULTY);
        image.the_character_is(A_CHARACTER);
        image.the_title_is_on(Screen::Partner);
        image.the_titles_cursor_is(A_PARTNER, PARTNERS);

        let pointed = unsafe { Th11.run_pointed_at() }.expect("the partner select knows which run");
        assert_eq!(
            (
                pointed.difficulty,
                pointed.character,
                pointed.shot_type,
                pointed.stage
            ),
            (A_DIFFICULTY, A_CHARACTER, A_PARTNER, 0),
            "the run the partner select points at is not the cursor and the selects before it",
        );
        assert!(
            !pointed.practice,
            "the run a whole-run flow points at reads as a practice",
        );

        // **And the Extra's stage where the difficulty is the Extra's**, which is the stage the decide there
        // would start.
        image.the_difficulty_is(DIFFICULTY_EXTRA);
        let extra = unsafe { Th11.run_pointed_at() }.expect("the partner select knows which run");
        assert_eq!(
            (extra.difficulty, extra.stage),
            (DIFFICULTY_EXTRA, THE_EXTRAS_STAGE),
            "the run pointed at over the Extra is not the Extra's stage",
        );

        // And nothing again on the character select, where the partner has not been chosen.
        image.the_title_is_on(Screen::Character);
        assert!(
            unsafe { Th11.run_pointed_at() }.is_none(),
            "orb reads a run being pointed at a screen before the partner is chosen",
        );
    });
}
