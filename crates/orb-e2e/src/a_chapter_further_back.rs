//! **更に前からやり直す, which lists the chapters the stage has behind the one that was lost** — so that
//! where a run goes back to is any of them rather than that chapter or the stage's own start.
//!
//! Why the way back has to reach further than one chapter is beside `chapter.rs`'s `guarded`: a boundary
//! the fight really has is worth more than one that is always comfortable, so a chapter can begin under a
//! bomb, on the frame a death is certain, or anywhere else its own start cannot be cleared from — and the
//! way out of one of those is the chapter before it.
//!
//! **A screen of its own behind an item, rather than the four ways on becoming a list of chapters.** The
//! first item is answered every few seconds in a fight and is one press with nothing to read; the
//! chapters are what somebody reads when that item is not the answer.
//!
//! In the order a run meets them: the four ways on, the chapters listed behind the second of them with the
//! one that was lost not among them, the run put back at the second of those — read out of the game's own
//! memory against the frame that chapter began on — the file this run is written down in following it
//! there, and then the same menu from where it landed, where the list holds what the stage still has: the
//! chapters after the one restored went with the restore.

use crate::fake::th06::{ATTACK_CHANGES, BOSS_ARRIVES, CARD_STARTS, Fake, the_run};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::LogLevel;
use orb_core::menu_ui::{LINE_HEIGHT, NORMAL, SELECTED};
use orb_sim::keys;

/// What the four chapters this run reaches are called, in the order it reaches them: the stage's own
/// start, the midboss arriving, its spell card, and the attack after that card.
///
/// The names the boundary detector gives them — which part of the stage each belongs to and which one of
/// those it is — since that is what the list shows and what somebody choosing one reads.
const STAGE_START: &str = "MIDSTAGE 1";
const THE_MIDBOSS: &str = "MIDBOSS NONSPELL 1";
const THE_CARD: &str = "MIDBOSS SPELL 1";
const AFTER_THE_CARD: &str = "MIDBOSS NONSPELL 2";

/// The four ways on, in the order they are offered.
const AGAIN: &str = "チャプターをやり直す";
const FURTHER: &str = "更に前からやり直す";
const STAGE: &str = "ステージをやり直す";
const QUIT: &str = "タイトルに戻る";

/// What the screen behind 更に前からやり直す asks above the chapters it lists.
const ASKED: &str = "どこからやり直す";

/// The lines on the screen now: what is written above them, the items in the order they were listed,
/// and which one is under the cursor.
///
/// Each is one quad because each is one line drawn once, so an item on the screen twice — or a chapter
/// the stage no longer has a snapshot of — fails here rather than in whichever assertion below happens
/// to read it.
fn reads(game: &Fake, header: &str, items: &[&str], under_the_cursor: &str) {
    game.forget();
    game.frame();
    let mut lines = Vec::new();
    for item in items {
        let drawn = game.says(item);
        assert_eq!(drawn.len(), 1, "{item} is not one line of the menu");
        assert_eq!(
            drawn[0].color,
            if item == &under_the_cursor {
                SELECTED
            } else {
                NORMAL
            },
            "the cursor is not on {under_the_cursor}",
        );
        lines.push(drawn[0].y);
    }
    for (item, pair) in items.iter().zip(lines.windows(2)) {
        assert_eq!(
            pair[1] - pair[0],
            LINE_HEIGHT,
            "what follows {item} is not the next line of the menu",
        );
    }
    let above = game.says(header);
    assert_eq!(above.len(), 1, "{header} is not written once");
    assert!(
        above[0].y < lines[0],
        "{header} is not above what is listed"
    );
}

/// Walks the cursor `down` presses from the item it starts on and presses decide there, then waits for
/// what that press was answered with.
///
/// The direction is pressed only after the frames the menu reads nothing over — a press inside those is
/// one nothing moved on, see [`READS_KEYS_AFTER`].
fn chooses(game: &Fake, down: u32, said: &str) {
    let log = game.log();
    let from = log.written();
    game.frames(READS_KEYS_AFTER);
    for _ in 0..down {
        game.press(keys::DOWN);
    }
    game.press_until(keys::Z, said, || log.said_since(from, said));
}

/// Answers a confirmation with はい, which is one press from the いいえ its cursor starts on.
fn answers_yes(game: &Fake, said: &str) {
    let log = game.log();
    let from = log.written();
    game.frames(READS_KEYS_AFTER);
    game.press(keys::UP);
    game.press_until(keys::Z, said, || log.said_since(from, said));
}

/// **A death with nothing behind the chapter it happened in leaves that item out**, since what is behind
/// it would be a screen with nothing on it: the stage's own start is the chapter being played, and the
/// two items that put it back are the first and the third.
///
/// Read off the screen rather than out of the menu, because what an item costs when it does nothing is
/// paid there: a line somebody walks the cursor onto to be shown an empty list.
#[test]
fn the_chapters_behind_it_are_not_offered_where_the_run_has_reached_no_other() {
    in_its_own_process(|| {
        let game = Fake::attach("going-back-from-the-first", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        // Which stops on the stage's own first chapter, well before the fight the stage's first boundary
        // is: nothing has been reached for a list to hold.
        game.in_a_pointdevice_run();
        assert!(
            !log.said(&format!("chapter 2 at frame {BOSS_ARRIVES}")),
            "the run reached a second chapter, so there is something behind the first:\n  {}",
            log.lines().join("\n  ")
        );

        game.hit();
        game.frame();
        assert!(
            log.said("died in chapter 1"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        reads(&game, STAGE_START, &[AGAIN, STAGE, QUIT], AGAIN);
        assert!(
            game.says(FURTHER).is_empty(),
            "an item was offered for chapters the run has not reached",
        );
    });
}

#[test]
fn the_chapters_behind_the_one_that_was_lost_are_listed_and_are_where_choosing_one_puts_the_run() {
    in_its_own_process(|| {
        // `verbose`, because the file each chapter is written down in says so at that level, and a run
        // that goes back through its own chapters writes over one.
        let game = Fake::attach("going-back-further", the_run(), |config| {
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();
        game.in_a_pointdevice_run();

        // The fight, chapter by chapter, with the frame each began on read out of the game's memory: that
        // is what a restore has to put back, field for field.
        game.frames_until("the midboss's chapter", BOSS_ARRIVES + 400, || {
            log.said(&format!("chapter 2 at frame {BOSS_ARRIVES}"))
        });
        let at_the_midboss = game.state();
        assert_eq!(at_the_midboss.stage_frames, BOSS_ARRIVES);

        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!("chapter 3 at frame {CARD_STARTS}"))
        });
        let at_the_card = game.state();
        assert_eq!(at_the_card.stage_frames, CARD_STARTS);

        game.frames_until("the chapter after the card", ATTACK_CHANGES + 400, || {
            log.said(&format!("chapter 4 at frame {ATTACK_CHANGES}"))
        });

        // ── 被弾, and the four ways on over the frozen game, with the chapter that was lost named above
        // them and the cursor on the item that puts it back.
        game.hit();
        game.frame();
        assert!(
            log.said("died in chapter 4"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        // The game as the menu froze it, for the question below: what a question nobody has answered
        // yet has to leave alone.
        let at_the_death = game.state();
        reads(&game, AFTER_THE_CARD, &[AGAIN, FURTHER, STAGE, QUIT], AGAIN);

        // ── 更に前からやり直す, which is a screen rather than a place: the chapters the stage has behind
        // the one that was lost, newest first, with the stage's own start the last of them. The chapter
        // that was lost is not among them — it is the item above this one.
        chooses(&game, 1, "retry: asking which chapter further back");
        reads(
            &game,
            ASKED,
            &[THE_CARD, THE_MIDBOSS, STAGE_START],
            THE_CARD,
        );
        assert!(
            game.says(AFTER_THE_CARD).is_empty(),
            "the chapter that was lost is listed among the ones behind it",
        );

        // ── The midboss arriving, which is two chapters back, and the question it asks before it acts:
        // named, since the item it was chosen from is not on the screen the question replaces it with,
        // and under it what going back costs — the chapter the run is in is not one it can come back to.
        chooses(&game, 1, "retry: asking about a chapter further back");
        game.forget();
        game.frame();
        assert_eq!(
            game.says(&format!("{THE_MIDBOSS} からやり直す？")).len(),
            1,
            "the question does not name the chapter it goes to",
        );
        assert_eq!(
            game.says("今のチャプターには戻れません").len(),
            1,
            "the question does not say the chapter the run is in cannot be come back to",
        );
        assert_eq!(
            game.state(),
            at_the_death,
            "something was put back by a question nobody had answered yet",
        );

        // Answered はい, and the run comes back to the frame that chapter began on.
        answers_yes(&game, "retry chapter 2 (retry 1)");
        assert_eq!(
            game.state(),
            at_the_midboss,
            "the run is not where the chapter two back began, field for field",
        );
        assert!(
            log.said("retry: a chapter further back on the keyboard"),
            "the chapter was not chosen on the keyboard:\n  {}",
            log.lines().join("\n  ")
        );
        // And the file this run is written down in follows it back, so a session closed here picks the
        // run up at the chapter it went back to. Nothing of the record has to be dropped for that: it is
        // one entry per stage frame, and the frames under a chapter are the ones that reached it.
        let landed = log.written();
        game.one_frame_to_drain_the_log();
        assert!(
            log.said_since(
                landed,
                &format!(
                    "chapter 2 ({THE_MIDBOSS}) at frame {BOSS_ARRIVES}, \
                     {BOSS_ARRIVES} frame(s) of buttons"
                ),
            ),
            "the chapter written down is not the one the run went back to:\n  {}",
            log.lines().join("\n  ")
        );

        // ── And the stage played on from there. What the list holds now is what the stage still has: the
        // chapters that came after the one restored went with it, so the card's chapter — reached again —
        // is the one that was lost, and the list behind it is the midboss and the stage's start.
        let from = log.written();
        game.frames_until("the card's chapter again", CARD_STARTS + 400, || {
            log.said_since(from, &format!("chapter 3 at frame {CARD_STARTS}"))
        });
        game.hit();
        game.frame();
        assert!(
            log.said_since(from, "died in chapter 3"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        reads(&game, THE_CARD, &[AGAIN, FURTHER, STAGE, QUIT], AGAIN);
        chooses(&game, 1, "retry: asking which chapter further back");
        reads(&game, ASKED, &[THE_MIDBOSS, STAGE_START], THE_MIDBOSS);
        assert!(
            game.says(AFTER_THE_CARD).is_empty(),
            "a chapter the run went back past is still offered",
        );
        assert_eq!(
            game.says("RETRY 1").len(),
            1,
            "the list does not say what the run has already spent",
        );

        // ── And the way out of that screen, the four ways on having none of their own: cancel goes back
        // to them with nothing put back, and チャプターをやり直す from there is the item it always was.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::X);
        assert!(
            log.said("retry: which chapter further back — cancelled, back to the choices"),
            "the cancel was not said out loud:\n  {}",
            log.lines().join("\n  ")
        );
        reads(&game, THE_CARD, &[AGAIN, FURTHER, STAGE, QUIT], FURTHER);
        chooses(&game, 3, "retry chapter 3 (retry 2)");
        assert_eq!(
            game.state(),
            at_the_card,
            "the run is not where the card's chapter began, field for field",
        );
    });
}
