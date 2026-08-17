//! **更に前からやり直す over a run of 妖々夢**: the chapters the stage has behind the one that was lost, and
//! the run put back at any of them.
//!
//! The same question `a_chapter_further_back.rs` puts to 紅魔郷 — why the way back has to reach further than
//! one chapter is beside `chapter.rs`'s `guarded` — asked of this game's own fields. What 妖々夢 brings is what
//! a chapter of it is *made* of: the fight read off the manager's slot and the timer of an attack, the frame
//! read off the game manager's own clock, and the memory a restore puts back. So the assertion that a run came
//! back to a chapter is made field for field against the state that chapter began with, which is the whole of
//! what a snapshot of this game is worth.
//!
//! In the order a run meets them: the four ways on, the chapters listed behind the second of them with the one
//! that was lost not among them, the run put back at one of those, the file this run is written down in
//! following it there, and then the same menu from where it landed — where the list holds what the stage still
//! has, the chapters after the one restored having gone with the restore.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::menu_ui::{LINE_HEIGHT, NORMAL, SELECTED};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way 妖々夢 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be, so that each is
/// a chapter of its own.
const APART: u32 = 120;

/// What the four chapters this run reaches are called, in the order it reaches them: the stage's own start, the
/// midboss arriving, its card, and the attack after that card.
///
/// The names the boundary detector gives them — which part of the stage each belongs to and which one of those
/// it is — since that is what the list shows and what somebody choosing one reads.
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

/// A 完全無欠モード run of this game with its stage's own track streaming, stopped at the stage's own first
/// chapter.
fn a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game
}

/// The lines on the screen now: what is written above them, the items in the order they were listed, and which
/// one is under the cursor.
///
/// Each is one quad because each is one line drawn once, so an item on the screen twice — or a chapter the
/// stage no longer has a snapshot of — fails here rather than in whichever assertion below happens to read it.
///
/// # The frame it reads, and why it is not simply the next one
///
/// `MIDBOSS SPELL 1 is not written once`, `left: 0` — nothing read off the screen at all — came out of two runs
/// of the whole suite, and the same test on its own passed every time after each of them: a frame orb was late
/// for is a frame it drew nothing in. `Launched::a_frame_that_says` is what that cost, and what this reads.
fn reads(game: &Fake, header: &str, items: &[&str], under_the_cursor: &str) {
    let above = game.a_frame_that_says(header);
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
    assert_eq!(above.len(), 1, "{header} is not written once");
    assert!(
        above[0].y < lines[0],
        "{header} is not above what is listed"
    );
}

/// Walks the cursor `down` presses from the item it starts on and presses decide there, then waits for what
/// that press was answered with.
///
/// The direction is pressed only after the frames the menu reads nothing over — a press inside those is one
/// nothing moved on, see [`READS_KEYS_AFTER`].
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

/// **A death with nothing behind the chapter it happened in leaves that item out**, since what is behind it
/// would be a screen with nothing on it: the stage's own start is the chapter being played, and the two items
/// that put it back are the first and the third.
///
/// Read off the screen rather than out of the menu, because what an item costs when it does nothing is paid
/// there: a line somebody walks the cursor onto to be shown an empty list.
#[test]
fn the_chapters_behind_it_are_not_offered_where_a_run_of_th07_has_reached_no_other() {
    in_its_own_process(|| {
        let game = a_run("th07-going-back-from-the-first");
        let log = game.log();
        // Which stops on the stage's own first chapter: nothing of the fight has been driven, so there is
        // nothing for a list to hold.
        assert!(
            !log.said("chapter 2 at frame"),
            "the run reached a second chapter, so there is something behind the first:\n  {}",
            log.lines().join("\n  ")
        );

        game.dies();
        game.one_frame();
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
fn the_chapters_behind_the_one_lost_in_th07_are_listed_and_are_where_choosing_one_puts_the_run() {
    in_its_own_process(|| {
        let game = a_run("th07-going-back-further");
        let log = game.log();

        // The fight, chapter by chapter, with the state each began with read out of the game's memory: that
        // is what a restore has to put back, field for field.
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        let at_the_midboss = game.state();

        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 3"));

        game.frames(APART);
        game.the_card_is_over();
        game.the_attack_changes();
        game.frames_until("the chapter after the card", APART, || {
            log.said("chapter 4")
        });

        // ── 被弾, and the four ways on over the frozen game, with the chapter that was lost named above them
        // and the cursor on the item that puts it back.
        game.dies();
        game.one_frame();
        assert!(
            log.said("died in chapter 4"),
            "the death was not noticed:\n  {}",
            log.lines().join("\n  ")
        );
        // The game as the menu froze it, for the question below: what a question nobody has answered yet has
        // to leave alone.
        let at_the_death = game.state();
        reads(&game, AFTER_THE_CARD, &[AGAIN, FURTHER, STAGE, QUIT], AGAIN);

        // ── 更に前からやり直す, which is a screen rather than a place: the chapters the stage has behind the
        // one that was lost, newest first, with the stage's own start the last of them. The chapter that was
        // lost is not among them — it is the item above this one.
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
        // named, since the item it was chosen from is not on the screen the question replaces it with, and
        // under it what going back costs — the chapter the run is in is not one it can come back to.
        //
        // Where the log stands first, this run having been through that chapter once already: what is read
        // back below has to be what going back to it wrote, not what reaching it did.
        let going_back = log.written();
        chooses(&game, 1, "retry: asking about a chapter further back");
        game.forget();
        game.one_frame();
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

        // Answered はい, and the run comes back to the frame that chapter began on — every field of this
        // game's own state as the chapter had it.
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
        // And the file this run is written down in follows it back, so a session closed here picks the run up
        // at the chapter it went back to. Waited for rather than read at once, and held against the mark
        // taken before the restore: the write is a thread of orb's own — see
        // `the_run_written_down_off_the_frame` — so the line lands whenever that thread is given a turn.
        // Nothing of the record has to be dropped for that: it is one entry per stage frame, and the frames
        // under a chapter are the ones that reached it.
        let at = at_the_midboss.stage_frames;
        let written = format!("chapter 2 ({THE_MIDBOSS}) at frame {at}, {at} frame(s) of buttons");
        game.frames_until_a_thread("the chapter written down", || {
            log.said_since(going_back, &written)
        });

        // ── And the stage played on from there, the card put up again. What the list holds now is what the
        // stage still has: the chapters that came after the one restored went with it, so the card's chapter
        // is the one that was lost and the list behind it is the midboss and the stage's start.
        let from = log.written();
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter again", APART, || {
            log.said_since(from, "chapter 3")
        });
        let at_the_card = game.state();
        game.dies();
        game.one_frame();
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

        // ── And the way out of that screen, the four ways on having none of their own: cancel goes back to
        // them with nothing put back, and チャプターをやり直す from there is the item it always was.
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
