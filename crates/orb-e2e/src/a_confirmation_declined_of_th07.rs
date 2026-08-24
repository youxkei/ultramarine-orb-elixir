//! **The two items of th07's retry menu that ask first, and the two ways of saying no to them.**
//!
//! ステージをやり直す throws away everything the stage has gained since its start and タイトルに戻る throws away
//! the run, and both are two presses from the hand that has just answered チャプターをやり直す for the fortieth
//! time in a fight. So each asks, and `retry_ui`'s own comment says what has to be true of the answer: a
//! confirmation exists to stop something, and a session that lost a stage anyway has to be able to see whether
//! the stop happened. That is a line in the log, and this is what holds orb to it over this game.
//!
//! Two ways of declining, because they are different presses and not one: **いいえ** is the cursor left where
//! the question puts it and decide pressed again, and **cancel** is the key that closes anything. Either goes
//! back to the ways on with nothing thrown away — which for th07 means the memory of the stage untouched, so
//! what is read back after each is the state the death was noticed on, field for field — and the menu is still
//! the menu, which the チャプターをやり直す at the end of each is evidence of.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// The four ways on, in the order they are offered.
const AGAIN: &str = "チャプターをやり直す";
const FURTHER: &str = "更に前からやり直す";
const STAGE: &str = "ステージをやり直す";
const QUIT: &str = "タイトルに戻る";

/// A run played to the chapter its midboss's card begins and then lost there: the retry menu up over a game
/// frozen on the frame the death was noticed on.
///
/// Two chapters before the one that is lost, so that all four ways on are offered — 更に前からやり直す is left
/// out where there is nothing behind, and the presses below count items.
/// Answers the game and the state that chapter began with, which is what a retry has to put back — read on the
/// frame the chapter was taken on, since one more frame of the run is another frame.
fn lost_the_card(name: &str) -> (Box<Fake>, orb_core::game::State) {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game.frames(APART);
    game.the_boss_arrives();
    game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
    game.frames(APART);
    game.a_card_starts(CARD, CARD_NAME);
    game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
    let at_the_card = game.state();
    game.dies();
    game.frames_until("the death seen", 8, || log.said("died in chapter 3"));
    (game, at_the_card)
}

/// Walks the cursor to the item `down` presses below チャプターをやり直す and presses decide on it, then waits
/// for the question that item asks.
///
/// The direction is pressed only after the frames the menu reads nothing over: one inside those is a press
/// nothing moved on — see [`READS_KEYS_AFTER`].
fn asks_about(game: &Fake, down: u32, asked: &str) {
    let log = game.log();
    let from = log.written();
    game.frames(READS_KEYS_AFTER);
    for _ in 0..down {
        game.press(keys::DOWN);
    }
    game.press_until(keys::Z, asked, || {
        log.said_since(from, &format!("retry: asking about {asked}"))
    });
}

/// The four ways on back on the screen with the cursor still on the one that was asked about, which is what
/// going back to them looks like from outside the log.
fn the_choices_are_back(game: &Fake, under_the_cursor: &str) {
    game.forget();
    game.one_frame();
    for item in [AGAIN, FURTHER, STAGE, QUIT] {
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

/// いいえ, which is where the question's own cursor starts: the press that lands on the frame the confirmation
/// begins reading keys on costs nothing but the question closing.
#[test]
fn answering_no_to_the_stage_of_th07_goes_back_to_the_choices_with_the_stage_kept() {
    in_its_own_process(|| {
        let (game, at_the_card) = lost_the_card("th07-declined-no");
        let log = game.log();
        let at_the_death = game.state();

        asks_about(&game, 2, "the stage again");
        // The question, and its two answers with the cursor on the one that costs nothing.
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("ステージの最初からやり直す？").len(),
            1,
            "the third item asked nothing",
        );
        // And what it says under the question: what going back to the stage's start leaves behind is the
        // chapter the run is in, which nothing else on the screen says.
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
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::Z);
        assert!(
            log.said_since(
                from,
                "retry: the stage again — answered no, back to the choices"
            ),
            "the answer was not said out loud:\n  {}",
            log.lines().join("\n  ")
        );
        the_choices_are_back(&game, STAGE);
        assert_eq!(
            game.state(),
            at_the_death,
            "something was put back by a question that was answered no",
        );

        // And the menu is still the menu: チャプターをやり直す from here is the item that was always two presses
        // away, and it puts the chapter back. Down twice from ステージをやり直す, four items wrapping.
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapter again", || {
            log.said_since(from, "retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the chapter the menu went on to put back is not the one that was lost",
        );
    });
}

/// And cancel, which is the way out of a question asked by mistake — the ways on underneath have no cancel of
/// their own, the player being dead and those items the only ways on.
#[test]
fn cancelling_the_question_of_th07_about_giving_up_leaves_the_run_running() {
    in_its_own_process(|| {
        let (game, at_the_card) = lost_the_card("th07-declined-cancel");
        let log = game.log();

        asks_about(&game, 3, "the run given up");
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::X);
        assert!(
            log.said_since(
                from,
                "retry: the run given up — cancelled, back to the choices"
            ),
            "the cancel was not said out loud:\n  {}",
            log.lines().join("\n  ")
        );
        the_choices_are_back(&game, QUIT);
        assert!(
            !log.said_since(from, "retry: the run is given up"),
            "the run was given up by a question that was cancelled:\n  {}",
            log.lines().join("\n  ")
        );

        // The run is still there to be retried, which is the whole of what the cancel bought: down once from
        // タイトルに戻る wraps onto the chapter that was lost.
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapter again", || {
            log.said_since(from, "retry: the chapter again on the keyboard")
        });
        assert_eq!(
            game.state(),
            at_the_card,
            "the chapter the menu went on to put back is not the one that was lost",
        );
    });
}
