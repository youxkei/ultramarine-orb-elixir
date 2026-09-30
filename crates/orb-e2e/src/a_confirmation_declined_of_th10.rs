//! **The two items of the retry menu that ask first over th10, and the two ways of saying no to them.**
//!
//! ステージをやり直す throws away everything the stage has gained since its start and タイトルに戻る throws
//! away the run, and both are two presses from the hand that has just answered チャプターをやり直す for the
//! fortieth time in a fight. So each asks, and what has to be true of the answer is that a session which lost a
//! stage anyway can see whether the stop happened — which is a line in the log.
//!
//! **What is this game's here is what the answer is read back off**: the menu and its wordings are
//! `retry_ui`'s, which `a_confirmation_declined_of_th06c.rs` holds to that game; what this file asks is that the
//! menu is on the screen over this game and that declining leaves the run standing where the death was noticed
//! — the state field for field.
//!
//! Two ways of declining, because they are different presses and not one: **いいえ** is the cursor left where
//! the question puts it and decide pressed again, and **cancel** is the key that closes anything.

use orb_config::LogLevel;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_sim::keys;

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **How long a stage lasts here**: past its midboss, whose two attacks from frame 1200 are its second and third
/// chapters, so that the chapter the run is lost in has two behind it and all four ways on are offered.
const STAGE_FRAMES: u32 = 4_500;

/// How many frames the walk to that chapter is given.
const FRAMES: u32 = STAGE_FRAMES * 2;

/// The four ways on, in the order they are offered.
const AGAIN: &str = "チャプターをやり直す";
const FURTHER: &str = "更に前からやり直す";
const STAGE: &str = "ステージをやり直す";
const QUIT: &str = "タイトルに戻る";

/// **A run played to its midboss's card and then lost there**: the retry menu up over a game standing on the
/// frame the death was noticed on, and the state it is standing in. Placed, with the stage's clock its own and
/// nothing fired at the player, so that the death is the one the test puts there.
fn lost_the_third_chapter(name: &str) -> (Box<Fake>, orb_core::game::State) {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.stages_last(STAGE_FRAMES);
    game.fires_at_the_player(false);
    game.plays_its_own_stage();
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.frames_until("the midboss's card", FRAMES, || {
        log.said("stage 1 chapter 3 ")
    });
    game.frames(60);
    game.the_player_is_hit();
    game.frames_until("the death seen", 16, || log.said("died in chapter 3"));
    let at_the_death = game.state();
    (game, at_the_death)
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
fn answering_no_to_the_stage_of_th10_goes_back_to_the_choices_with_the_stage_kept() {
    in_its_own_process(|| {
        let (game, at_the_death) = lost_the_third_chapter("th10-declined-no");
        let log = game.log();
        asks_about(&game, 2, "the stage again");

        // The question, and its two answers with the cursor on the one that costs nothing.
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("ステージの最初からやり直す？").len(),
            1,
            "the third item asked nothing",
        );
        let no = game.says("いいえ");
        assert_eq!(no.len(), 1, "the question has no いいえ on it");
        assert_eq!(no[0].color, SELECTED, "the cursor does not start on いいえ");
        assert_eq!(game.says("はい")[0].color, NORMAL, "はい is lit as well");

        // Decide again, with the cursor where it was left: the answer is no.
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press_until(keys::Z, "the question declined", || {
            log.said_since(
                from,
                "retry: the stage again — answered no, back to the choices",
            )
        });
        the_choices_are_back(&game, STAGE);
        assert_eq!(
            game.state().to_string(),
            at_the_death.to_string(),
            "the run did not stay standing where the death was noticed",
        );
    });
}

/// And cancel declines it too, which is the key that closes anything of orb's.
#[test]
fn cancelling_the_question_about_the_stage_of_th10_goes_back_to_the_choices() {
    in_its_own_process(|| {
        let (game, at_the_death) = lost_the_third_chapter("th10-declined-cancel");
        let log = game.log();
        asks_about(&game, 2, "the stage again");
        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press_until(keys::X, "the question cancelled", || {
            log.said_since(
                from,
                "retry: the stage again — cancelled, back to the choices",
            )
        });
        the_choices_are_back(&game, STAGE);
        assert_eq!(
            game.state().to_string(),
            at_the_death.to_string(),
            "the run did not stay standing where the death was noticed",
        );
    });
}
