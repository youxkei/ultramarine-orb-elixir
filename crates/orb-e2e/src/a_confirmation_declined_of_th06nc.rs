//! **The items of the retry menu that ask first over th06nc, and the two ways of saying no to them.**
//!
//! ステージをやり直す throws away everything the stage has gained since its start and タイトルに戻る throws
//! away the run, and both are two presses from the hand that has just answered チャプターをやり直す for the
//! fortieth time in a fight. So each asks, and what has to be true of the answer is that a session which
//! lost a stage anyway can see whether the stop happened — which is a line in the log.
//!
//! **Three ways on here where th06c is asked over four**, which is the one thing about this game the file
//! is for: `retry_ui` leaves 更に前からやり直す out where the chapter that was lost has nothing behind it,
//! and the death here falls in the stage's first chapter, before the midboss's arrival or any row of this
//! game's `chapters.rs`. ステージをやり直す is therefore **one** down from チャプターをやり直す and not two,
//! and タイトルに戻る two and not three. A recipe of `orb-real`'s that answered 3 got
//! `retry: item 3 is past the 3 way(s) on`.
//!
//! **And the question is answered on the keyboard**, which is what says orb sees a press of this game at
//! all: the word the run acts on is what `runtime::the_frames_word` reads, the frame's own keys go into it
//! through `reads_the_keys`, and a launch of the real game answered `retry: the chapter again on the pad`
//! off exactly that. `a_pointdevice_run_of_th06nc` drives its answers from the recipe instead, which is
//! the other half of the same claim.
//!
//! Two ways of declining, because they are different presses and not one: **いいえ** is the cursor left
//! where the question puts it and decide pressed again, and **cancel** is the key that closes anything.

use orb_config::LogLevel;
use orb_core::game::th06nc::image::PLAYER_PLAYING;
use orb_core::menu_ui::{NORMAL, SELECTED};
use orb_sim::keys;

use crate::fake::th06nc::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long a stage lasts here, and how many frames the walk into one is given: the death is asked for a
/// moment in, so neither has to be a stage's own length.
const STAGE_FRAMES: u32 = 1_200;
const FRAMES: u32 = 600;

/// The three ways on, in the order they are offered — 更に前からやり直す left out, see the module's doc.
const AGAIN: &str = "チャプターをやり直す";
const STAGE: &str = "ステージをやり直す";
const QUIT: &str = "タイトルに戻る";

/// **How far down ステージをやり直す is from the item the menu opens on**, which is the number this file is
/// about: one where th06c's is two.
const DOWN_TO_THE_STAGE: u32 = 1;

/// A run played a moment into its first stage and then lost there: the retry menu up over a game standing
/// on the frame the death was noticed on, and the state it is standing in.
fn lost_the_stages_chapter(name: &str) -> (Box<Fake>, orb_core::game::State) {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.stages_last(STAGE_FRAMES);
    // **Nothing fires at the player while the run walks in**, so that the death this asks for is the one
    // it asked for: a stage of this laid-out game that fires killed the run before a test got to it.
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game.frames_until("the run's own first chapter", FRAMES, || {
        log.said("stage 1 chapter 1")
    });
    // **And past the frames nothing of the stage can reach its player in**, which is this game's own doing
    // and not a wait for tidiness: the stage's build leaves the player coming back for thirty frames and
    // invulnerable for two hundred and forty after that, and a hit asked for inside either is a hit that
    // does not land — see `the_player_a_stage_of_th06nc_starts`. **The state a hit lands in is the playing
    // one**, which `State::unsettled` does not answer: that field is the hit and the walk-in, and the
    // invulnerability between them is neither.
    game.frames_until("the player playing", FRAMES, || {
        game.image().the_players_state() == PLAYER_PLAYING
    });
    game.hit();
    // **And then the deathbomb window out**, which the hit stands through: the fake counts it down and
    // the death is the frame it runs out on — `A_BOMB_IS_TAKEN_FOR` in `fake::th06nc`.
    game.frames_until("the death seen", 60, || log.said("died in chapter 1"));
    let at_the_death = game.state();
    (game, at_the_death)
}

/// Walks the cursor to the item `down` presses below チャプターをやり直す and presses decide on it, then
/// waits for the question that item asks.
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

/// The three ways on back on the screen with the cursor still on the one that was asked about, which is
/// what going back to them looks like from outside the log — **and 更に前からやり直す on none of them**,
/// which is the item this game is not offered.
fn the_choices_are_back(game: &Fake, under_the_cursor: &str) {
    game.forget();
    game.one_frame();
    for item in [AGAIN, STAGE, QUIT] {
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
    assert!(
        game.says("更に前からやり直す").is_empty(),
        "the way back to a chapter further on is offered, where this chapter has none behind it",
    );
}

/// いいえ, which is where the question's own cursor starts: the press that lands on the frame the
/// confirmation begins reading keys on costs nothing but the question closing.
#[test]
fn answering_no_to_the_stage_of_th06nc_goes_back_to_the_choices_with_the_stage_kept() {
    in_its_own_process(|| {
        let (game, at_the_death) = lost_the_stages_chapter("th06nc-declined-no");
        let log = game.log();
        asks_about(&game, DOWN_TO_THE_STAGE, "the stage again");

        // The question, and its two answers with the cursor on the one that costs nothing.
        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("ステージの最初からやり直す？").len(),
            1,
            "the item asked nothing",
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
fn cancelling_the_question_about_the_stage_of_th06nc_goes_back_to_the_choices() {
    in_its_own_process(|| {
        let (game, at_the_death) = lost_the_stages_chapter("th06nc-declined-cancel");
        let log = game.log();
        asks_about(&game, DOWN_TO_THE_STAGE, "the stage again");

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

/// **And the way out asks as well**, which is the item this game numbers two where th06c numbers it
/// three: タイトルに戻る throws the run away, so declining it has to leave the run standing too.
#[test]
fn answering_no_to_the_title_of_th06nc_leaves_the_run_standing() {
    in_its_own_process(|| {
        let (game, at_the_death) = lost_the_stages_chapter("th06nc-declined-quit");
        let log = game.log();
        asks_about(&game, DOWN_TO_THE_STAGE + 1, "the run given up");

        game.forget();
        game.one_frame();
        assert_eq!(
            game.says("やめてタイトルに戻る？").len(),
            1,
            "the way out asked nothing",
        );

        let from = log.written();
        game.frames(READS_KEYS_AFTER);
        game.press_until(keys::Z, "the question declined", || {
            log.said_since(
                from,
                "retry: the run given up — answered no, back to the choices",
            )
        });
        the_choices_are_back(&game, QUIT);
        assert_eq!(
            game.state().to_string(),
            at_the_death.to_string(),
            "the run did not stay standing where the death was noticed",
        );
    });
}
