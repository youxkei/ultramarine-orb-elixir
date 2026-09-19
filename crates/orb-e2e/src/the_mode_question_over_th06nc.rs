//! **Orb's own mode question over th06nc's main menu**, which is the one screen of this game orb has
//! something to ask about — and the one thing eight launches of the real game were spent on.
//!
//! What is under test is the whole path a question stands on for a game with no Direct3D 8 device: the
//! swapchain found among the renderer's own objects, the canvas made over its back buffer, the question
//! drawn on it, and the decide taken out of the word so that the game's own menu never sees the press that
//! raised it.
//!
//! **The last of those is what the real game failed at**, and it failed silently. This game's calc walk
//! put the bot's own word straight into the game's keys, so `held_back` — the one call that raises a
//! question on a read — was never asked on a word at all: every press reached the menu's own item, the
//! run started, and `menu:` was never once in a log. What says so now is
//! [`the_decide_that_raises_th06ncs_question_does_not_reach_its_menu`].
//!
//! Where the numbers come from:
//!
//! - **The menu is state 3 and the title state 2**, so one decide opens the menu before the question's own
//!   decide is pressed at it — this game's title being a screen of its own, where th06c's title *is* the
//!   menu.
//! - **And the menu ignores a decide for twenty frames**, `0x14004832d` against `0x14`, which is what the
//!   recipe's tap is repeated past.

use orb_config::LogLevel;
use orb_core::game::th06nc::image::{MENU_ITEM_RUN, SCREEN_MENU};
use orb_core::game::{Game, Menu};

use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_core::mode::Mode;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, LANGUAGE, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames the walk to the question is given. The recipe's own bounds are longer than this between
/// them; what this is is the wall the test puts up rather than sitting in a loop.
const FRAMES: u32 = 4000;

/// **The walk to the menu's first item and orb's question over it, and no answer after it.** It ends at
/// the question on purpose: a recipe with nothing left to do presses nothing, so the question stands while
/// the game is read.
const TO_THE_MODE_QUESTION: &str = "\
- wait_until: { screen: 2 }
  give_up_after: 36000
- wait_until: { frames: 60 }
- press: [decide]
  frames: 6
- wait_until: { screen: 3 }
  give_up_after: 1800
- press: [up]
  frames: 6
  until: { screen: 3, cursor: 0 }
  give_up_after: 900
- wait_until: { frames: 30 }
- press: [decide]
  frames: 6
  until: { question: mode }
  give_up_after: 1800
";

/// A launch walked to the question.
///
/// # Panics
/// If the question never goes up, since every assertion below is about the game with it standing.
fn asking(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(TO_THE_MODE_QUESTION.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    let log = game.log();
    game.frames_until("the mode question", FRAMES, || {
        log.said("menu: Run is under the cursor, asking which mode")
    });
    game
}

/// **The question stands over the item that starts a run**, which is what `Game::menu_pointed_at` answers
/// and what a recipe's `answer: { question: mode }` is asked of.
#[test]
fn th06ncs_menu_raises_orbs_question_over_the_item_that_starts_a_run() {
    in_its_own_process(|| {
        let game = asking("th06nc-question-raised");
        assert_eq!(
            unsafe { orb_core::game::th06nc::Th06nc.menu_pointed_at() },
            Some(Menu::Run),
            "the question is up over something other than the item that starts a run",
        );
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_MENU,
            "the question is up over a screen that is not the main menu",
        );
        assert_eq!(
            game.image().the_item_the_cursor_is_on(),
            MENU_ITEM_RUN,
            "the cursor is not on the item the question is about",
        );
    });
}

/// **And the decide that raised it never reaches the game's own menu**, which is the whole of what the
/// hold is for: with the question standing the menu is still the menu, and no run has been started.
///
/// **This is the assertion eight launches of the real game were spent on.** The walk was put into the
/// game's keys through nothing that holds a decide back, so the press reached the item and orb never saw
/// one; a run started and the question never stood. See `game::th06nc`'s own calc walk, which hands the
/// frame's word over through `runtime::the_frames_word` now.
#[test]
fn the_decide_that_raises_th06ncs_question_does_not_reach_its_menu() {
    in_its_own_process(|| {
        let game = asking("th06nc-question-holds-the-decide");
        let state = game.state();
        assert_eq!(
            game.image().the_screen_the_front_end_is_on(),
            SCREEN_MENU,
            "the decide the question was raised on left the menu as well",
        );
        assert!(
            !state.in_game,
            "a run is in a stage with the question still standing: {state:?}",
        );
        assert!(
            !game
                .log()
                .lines()
                .iter()
                .any(|line| line.contains("chapter 1")),
            "a chapter was taken with the question still standing:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}

/// **The question is drawn**, over a game whose renderer is a Direct3D 11 one: the canvas over its own
/// swapchain and the overlay on it, which is what a question of orb's reaches this game's screen through.
#[test]
fn the_mode_question_of_th06nc_is_drawn_over_the_games_own_swapchain() {
    in_its_own_process(|| {
        let game = asking("th06nc-question-drawn");
        let log = game.log();
        assert!(
            log.said("overlay: ready"),
            "the question stood and the overlay never came up:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            game.pictures_finished() > 0,
            "the frame never ended, so nothing was presented for the question to be drawn on:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("canvas: drawing through this game's own Direct3D 11, over 640x480"),
            "no canvas was made over this game's swapchain:\n  {}",
            log.lines().join("\n  ")
        );
        // **And the two modes on the screen with the one orb is in lit**, which is the whole of what a
        // question is: a person who cannot read which is which cannot answer it.
        let mut lit = None;
        for mode in orb_core::mode::CHOICES {
            let text = mode.name(LANGUAGE);
            let drawn = game.says(text);
            assert_eq!(drawn.len(), 1, "{text} is not on the screen once");
            if drawn[0].color == SELECTED {
                assert!(lit.replace(mode).is_none(), "both modes are lit");
            } else {
                assert_eq!(drawn[0].color, NORMAL, "{text} is neither lit nor not");
            }
        }
        assert_eq!(
            lit,
            Some(Mode::Pointdevice),
            "the mode orb is in is not the one the question is on",
        );
    });
}

/// **And the ask about what that object is was never made**, which is how this game's swapchain differs
/// from th06c's: orb takes it as the factory answers it, so nothing has to ask an object whether it is a
/// swapchain — and nothing gives a reference back.
#[test]
fn th06ncs_swapchain_is_taken_as_it_is_made_and_never_asked_what_it_is() {
    in_its_own_process(|| {
        let game = asking("th06nc-swapchain-not-asked");
        assert_eq!(
            game.swapchain_references_given_back(),
            0,
            "orb asked this game's swapchain what it is, which is th06c's way and not this one's",
        );
        assert!(
            game.log()
                .said("frame: the swapchain this game makes will be orb's to present through"),
            "the factory's swapchains were not taken:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
