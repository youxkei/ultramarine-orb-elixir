//! **A launch pressing the keyboard from the host** — `--press-through-the-host` — where every word is
//! pressed into the keyboard state orb keeps and read back out of it, through the game's own
//! `GetKeyboardState` read and its own key table.
//!
//! **A keyboard and not a word handed to the read.** A word written into what the game's read has already
//! produced skips everything of the game's between an input and a button; what a launch pressing from the
//! host fills is the keyboard state itself — orb's own stand-in for the virtual keyboard device a driver
//! would give — so the game turns the keys into its word with the table it would use for a person's.
//!
//! **And it does that with another window in front and without taking the focus.** A real launch plays
//! with its window behind while somebody uses the machine, so what the press must not do is type into
//! whatever is in front: `SendInput` was tried for this and is the wrong road, being delivered to the
//! window in front and to nothing else. Nothing here leaves the process at all.

use crate::fake::th06::{Fake, the_run};
use crate::fake::{Launched, WINDOW, in_its_own_process};
use orb_api::Hwnd;
use orb_config::LogLevel;
use orb_sim::keys;

/// The bits of th06's word the shot and the bomb are.
const SHOT: u16 = 0x1;
const BOMB: u16 = 0x2;

/// Another program's window in front, which is where a real launch's own window sits.
const ANOTHER: Hwnd = Hwnd(WINDOW.0 + 1);

/// **A run of a recipe holding the shot and the bomb**, its front end walked by the recipe's own steps —
/// the mode question answered outright, which is what `in_a_pointdevice_run` does with a hand at the
/// keyboard and what a launch pressing from the host cannot do that way: the bot's own decides are down
/// in the same keyboard the hand would press, so a press of the hand's is no edge at all.
const THE_RUN: &str = "\
- press: [decide]\n  frames: 2\n  until: { question: mode }\n  give_up_after: 1800\n\
- answer: { question: mode, item: 0 }\n  give_up_after: 300\n\
- press: [decide]\n  frames: 2\n  until: { stage: 1, chapter: 1 }\n  give_up_after: 1800\n\
- press: [shot, bomb]\n  frames: 100000\n";

/// The same run with the last step changed to the one a test names.
fn playing_with(name: &str, last: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.autoplay = Some(THE_RUN.replace("- press: [shot, bomb]\n  frames: 100000\n", last));
        config.press_through_the_host = true;
        config.log_level = LogLevel::Verbose;
    });
    // The last step's own pressing, which is what every test here reads: the front end walked by the
    // recipe's steps before it, and a few frames of the run under the last one.
    game.frames_until("the run's last step", 600, || {
        game.log().said("stage 1 chapter 1 (stage start)")
    });
    game.frames(6);
    game
}

fn playing(name: &str) -> Box<Fake> {
    playing_with(name, "- press: [shot, bomb]\n  frames: 100000\n")
}

/// Whether the keyboard orb keeps has `key` down, which is the state the game's own read is answered with.
fn pressed(key: u8) -> bool {
    orb_api::keyboard::pressed()[usize::from(key)] & 0x80 != 0
}

/// **The recipe's word reaches the run as the keys of its buttons**: the run acts on the shot and the
/// bomb, and the keyboard orb keeps is holding Z and X for it.
#[test]
fn a_recipes_word_is_pressed_into_the_keyboard_the_game_reads() {
    in_its_own_process(|| {
        let game = playing("th06-from-the-host");
        assert!(
            game.log().said(
                "input: the keyboard is pressed from the host; every word is a key the game's own read is answered with"
            ),
            "the launch did not say it presses from the host:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.image().input_now() & (SHOT | BOMB),
            SHOT | BOMB,
            "the recipe's word did not reach the run:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert!(
            pressed(keys::Z) && pressed(keys::X),
            "the keyboard the game's read is answered with does not hold the word's keys",
        );
    });
}

/// **And it reaches the run with another window in front, and takes no focus to do it** — which is every
/// real launch, played behind whatever the machine's owner is working at.
#[test]
fn the_press_reaches_the_run_behind_another_window_and_takes_no_focus() {
    in_its_own_process(|| {
        let game = playing("th06-from-the-host-behind");
        game.sim().display().set_foreground(ANOTHER);
        game.frames(10);
        assert_eq!(
            game.image().input_now() & (SHOT | BOMB),
            SHOT | BOMB,
            "the press did not reach the run behind another window:\n  {}",
            game.log().lines().join("\n  ")
        );
        assert_eq!(
            game.sim().display().foreground(),
            ANOTHER,
            "the launch took the focus from the window in front",
        );
        assert!(
            !game.sim().windows().brought_forward().contains(&WINDOW),
            "the launch asked for its window to come forward: {:?}",
            game.sim().windows().brought_forward(),
        );
    });
}

/// **And a key the word lets go of is let go of**, a step that presses nothing — an empty press, or a
/// wait, which is the one a recipe holds for a stretch of a fight — leaving Z up rather than the run
/// holding a key no frame pressed.
#[test]
fn a_key_the_word_lets_go_of_is_let_go_of() {
    in_its_own_process(|| {
        let game = playing_with(
            "th06-from-the-host-let-go",
            "- press: [shot, bomb]\n  frames: 30\n- wait_until: { frames: 90 }\n  give_up_after: 600\n",
        );
        game.frames(10);
        assert!(pressed(keys::Z) && pressed(keys::X));
        // The recipe's last step presses nothing, and the keys of the word before go up with it.
        game.frames_until("the keys let go of", 120, || {
            !pressed(keys::Z) && !pressed(keys::X)
        });
        assert_eq!(
            game.image().input_now() & (SHOT | BOMB),
            0,
            "the run still acts on keys the word let go of:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
