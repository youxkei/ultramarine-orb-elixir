//! **The two items of the retry menu that ask first, and the two ways of saying no to them.**
//!
//! ステージをやり直す throws away everything the stage has gained since its start and タイトルに戻る throws
//! away the run, and both are two presses from the hand that has just answered チャプターをやり直す for
//! the fortieth time in a fight. So each asks, and `retry_ui`'s own comment says what has to be true of
//! the answer: a confirmation exists to stop something, and a session that lost a stage anyway has to be
//! able to see whether the stop happened. That is a line in the log, and this is what holds orb to it.
//!
//! Two ways of declining, because they are different presses and not one: **いいえ** is the cursor left
//! where the question puts it and decide pressed again, and **cancel** is the key that closes anything.
//! Either goes back to the ways on with nothing thrown away, and the menu is still the menu — which
//! is what the チャプターをやり直す at the end of this is evidence of.

use crate::fake::th06::{CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::th06::image::Screen;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_sim::keys;

/// A run played to the chapter its midboss's card begins, and then lost there: the retry menu up over a
/// game frozen on the frame the death was noticed on.
fn lost_the_card(name: &str) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
    });
    let log = game.log();
    game.at_the_title_menu();
    game.press(keys::Z);
    game.press_until(keys::Z, "the mode chosen", || {
        log.said("mode: answered on the keyboard")
    });
    game.frames_until("the shot type select", 90, || {
        let front = game.image().front_end_now();
        front.screen == Screen::ShotType && front.acts_on_a_press()
    });
    game.press(keys::Z);
    game.frames_until("the stage built", 8, || game.state().playing);
    game.frames_until("the card's chapter", CARD_STARTS + 400, || {
        log.said(&format!(
            "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
        ))
    });
    game.hit();
    game.frame();
    assert!(
        log.said("died in chapter 3"),
        "the death was not noticed:\n  {}",
        log.lines().join("\n  ")
    );
    game
}

/// Walks the cursor to the item `down` presses below チャプターをやり直す and presses decide on it, and
/// waits for the question that item asks.
///
/// The direction is pressed only after the frames the menu reads nothing over: one inside those is a press
/// nothing moved on — see [`READS_KEYS_AFTER`].
fn asks_about(game: &Fake, down: u32, asked: &str) {
    game.frames(READS_KEYS_AFTER);
    for _ in 0..down {
        game.press(keys::DOWN);
    }
    let log = game.log();
    game.press_until(keys::Z, asked, || {
        log.said(&format!("retry: asking about {asked}"))
    });
}

/// The four ways on back on the screen with the cursor still on the one that was asked about, which is
/// what going back to them looks like from outside the log.
fn the_choices_are_back(game: &Fake, under_the_cursor: &str) {
    game.forget();
    game.frame();
    let items = [
        "チャプターをやり直す",
        "更に前からやり直す",
        "ステージをやり直す",
        "タイトルに戻る",
    ];
    for item in items {
        let drawn = game.says(item);
        assert_eq!(
            drawn.len(),
            1,
            "{item} is not on the screen after the question was declined",
        );
        assert_eq!(
            drawn[0].color,
            if item == under_the_cursor {
                SELECTED
            } else {
                NORMAL
            },
            "the cursor is not on {under_the_cursor}",
        );
    }
}

/// いいえ, which is where the question's own cursor starts: the press that lands on the frame the
/// confirmation begins reading keys on costs nothing but the question closing.
#[test]
fn answering_no_to_the_stage_goes_back_to_the_choices_with_the_stage_kept() {
    in_its_own_process(|| {
        let game = lost_the_card("declined-no");
        let log = game.log();
        let at_the_death = game.state();

        asks_about(&game, 2, "the stage again");
        // The question, and its two answers with the cursor on the one that costs nothing.
        game.forget();
        game.frame();
        assert_eq!(
            game.says("ステージの最初からやり直す？").len(),
            1,
            "the third item asked nothing",
        );
        // And what it says under the question: what going back to the stage's start leaves behind is
        // the chapter the run is in, which nothing else on the screen says.
        assert_eq!(
            game.says("今のチャプターには戻れません").len(),
            1,
            "the question does not say what answering はい leaves behind",
        );
        let no = game.says("いいえ");
        assert_eq!(no.len(), 1);
        assert_eq!(no[0].color, SELECTED, "the cursor does not start on いいえ");
        assert_eq!(game.says("はい")[0].color, NORMAL);

        // Decide again, with the cursor where it was left: the answer is no.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::Z);
        assert!(
            log.said("retry: the stage again — answered no, back to the choices"),
            "the answer was not said out loud:\n  {}",
            log.lines().join("\n  ")
        );
        the_choices_are_back(&game, "ステージをやり直す");
        assert_eq!(
            game.state(),
            at_the_death,
            "something was put back by a question that was answered no",
        );

        // And the menu is still the menu: チャプターをやり直す from here is the item that was always two
        // presses away, and it puts the chapter back. Down twice from ステージをやり直す, four items
        // wrapping.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapter again", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state().stage_frames,
            CARD_STARTS,
            "the chapter the menu went on to put back is not the one that was lost",
        );
    });
}

/// And cancel, which is the way out of a question asked by mistake — the ways on underneath have no
/// cancel of their own, the player being dead and those items the only ways on.
#[test]
fn cancelling_the_question_about_giving_up_leaves_the_run_running() {
    in_its_own_process(|| {
        let game = lost_the_card("declined-cancel");
        let log = game.log();

        asks_about(&game, 3, "the run given up");
        game.frames(READS_KEYS_AFTER);
        game.press(keys::X);
        assert!(
            log.said("retry: the run given up — cancelled, back to the choices"),
            "the cancel was not said out loud:\n  {}",
            log.lines().join("\n  ")
        );
        the_choices_are_back(&game, "タイトルに戻る");
        assert!(
            !log.said("retry: the run is given up"),
            "the run was given up by a question that was cancelled:\n  {}",
            log.lines().join("\n  ")
        );

        // The run is still there to be retried, which is the whole of what the cancel bought: down once
        // from タイトルに戻る wraps onto the chapter that was lost.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapter again", || {
            log.said("retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state().stage_frames,
            CARD_STARTS,
            "the chapter the menu went on to put back is not the one that was lost",
        );
    });
}
