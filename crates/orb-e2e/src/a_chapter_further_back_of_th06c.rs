//! **更に前からやり直す over a run of th06c**: the chapters the stage has behind the one that was lost, and
//! the run put back at any of them.
//!
//! The same question `a_chapter_further_back.rs` puts to th06 and `a_chapter_further_back_of_th07.rs` to th07
//! — why the way back has to reach further than one chapter is beside `chapter.rs`'s `guarded` — asked of
//! this game's own fields. What th06c brings is what a chapter of it is *made* of: the fight read off the
//! enemy manager's first slot and the timer of an attack, the frame read off the game manager's own clock,
//! and the nine megabytes of `.data` and the heap blocks beside them that a restore puts back. So the
//! assertion that a run came back to a chapter is made field for field against the state that chapter began
//! with, which is the whole of what a snapshot of this game is worth.
//!
//! **Field for field but the deaths**, which this game keeps no count of: orb counts them itself — see
//! `Th06c::read_state` — so a chapter put back leaves the count where the hit that lost it put it, the same
//! as a death in any game does to the run's own tally. Every other field is the game's own memory, and a
//! restore is answerable for all of it.
//!
//! **The two chapters behind are this laid-out game's fight**, its midboss arriving at [`MIDBOSS_ARRIVES`]
//! and putting its card up at [`CARD_STARTS`]: the table's first boundary of stage one is thousands of frames
//! past the fight, so the fight is what a stage of it has two chapters of first.
//!
//! In the order a run meets them: the four ways on, the chapters listed behind the second of them with the
//! one that was lost not among them, the run put back at one of those, the file this run is written down in
//! following it there, and then the same menu from where it landed — where the list holds what the stage
//! still has, the chapters after the one restored having gone with the restore.
//!
//! **th07's third test is not here**: the pause menu's own 「最初からやり直す」 asking where to start over from
//! rests on `Game::asked_to_start_the_run_over`, which only th07 answers. Of this exe's item the count it
//! steps is read — `game_manager::NUM_RETRIES`, which orb's own retries are counted into — and the ask it
//! makes of the supervisor is not, so th06c leaves that question at its default `false` and the item does
//! what the game's own does.

use crate::fake::th06c::{ATTACK_CHANGES, CARD_STARTS, Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::State;
use orb_core::menu_ui::{LINE_HEIGHT, NORMAL, SELECTED};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **One line of the menu as it reaches this game's back buffer**, which is not the overlay's own 640x480:
/// the canvas lays the overlay over the whole of the buffer the swapchain says it has — see
/// `the_mode_question_over_th06c`'s canvas test — so a quad of it lands in the buffer's pixels, and a line
/// of [`LINE_HEIGHT`] is that many times the buffer's height over the overlay's.
const A_LINE: f32 =
    LINE_HEIGHT * crate::fake::BACK_BUFFER.1 as f32 / orb_core::overlay::SCREEN_HEIGHT;

/// How many frames the fight's own boundaries are waited for over: past the last of them.
const THE_FIGHTS_FRAMES: u32 = MIDBOSS_BEATEN;

/// What the four chapters this run reaches are called, in the order it reaches them: the stage's own start,
/// the midboss arriving, its card, and the attack after that card.
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

/// A Pointdevice mode run of this game in its first stage, stopped at the stage's own first chapter, with
/// nothing fired at the player — a death is a thing each test here takes where it means to.
fn a_run(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    let log = game.log();
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
    game.frames_until("the stage's own first chapter", MIDBOSS_ARRIVES, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The run as it stands now, with the deaths carried over from `began` — see this file's own doc for why
/// that one field is the one a restore is not answerable for.
fn now_but_the_deaths(game: &Fake, began: &State) -> State {
    State {
        deaths: began.deaths,
        ..game.state()
    }
}

/// The player hit where the run stands, and the death that hit is noticed as, in chapter `number`.
fn dies_in(game: &Fake, number: u32, from: usize) {
    let log = game.log();
    game.hit();
    game.frames_until("the death seen", 16, || {
        log.said_since(from, &format!("died in chapter {number}"))
    });
}

/// The lines on the screen now: what is written above them, the items in the order they were listed, and
/// which one is under the cursor.
///
/// Each is one quad because each is one line drawn once, so an item on the screen twice — or a chapter the
/// stage no longer has a snapshot of — fails here rather than in whichever assertion below happens to read it.
///
/// Read off [`Launched::a_frame_that_says`] rather than the next frame, which a frame orb was late for draws
/// nothing in — see that method's own doc.
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
        // Within a hundredth of a pixel, the stretch onto the buffer being arithmetic in floats.
        assert!(
            (pair[1] - pair[0] - A_LINE).abs() < 0.01,
            "what follows {item} is not the next line of the menu: {} apart where a line is {A_LINE}",
            pair[1] - pair[0],
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
/// **The hit lands once the stage's opening invulnerability is over** — the walk back in and the `0xf0`
/// behind it, see `the_player_a_stage_of_th06c_starts` — which is still inside the stage's first chapter,
/// the midboss arriving after it.
#[test]
fn the_chapters_behind_it_are_not_offered_where_a_run_of_th06c_has_reached_no_other() {
    in_its_own_process(|| {
        let game = a_run("th06c-going-back-from-the-first");
        let log = game.log();

        game.hit();
        game.frames_until("the death seen", MIDBOSS_ARRIVES, || {
            log.said("died in chapter 1")
        });
        // Which is the stage's own first chapter: nothing of the fight has come, so there is nothing for a
        // list to hold.
        assert!(
            !log.said("chapter 2 at frame"),
            "the run reached a second chapter, so there is something behind the first:\n  {}",
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
fn the_chapters_behind_the_one_lost_in_th06c_are_listed_and_are_where_choosing_one_puts_the_run() {
    in_its_own_process(|| {
        let game = a_run("th06c-going-back-further");
        let log = game.log();

        // The fight, chapter by chapter, with the state the midboss's arrival began with read out of the
        // game's memory: that is what a restore has to put back, field for field.
        game.frames_until("the midboss's chapter", THE_FIGHTS_FRAMES, || {
            log.said("chapter 2 at frame")
        });
        let at_the_midboss = game.state();
        game.frames_until("the card's chapter", THE_FIGHTS_FRAMES, || {
            log.said("chapter 3 at frame")
        });
        game.frames_until("the chapter after the card", THE_FIGHTS_FRAMES, || {
            log.said("chapter 4 at frame")
        });
        assert!(
            game.state().stage_frames >= ATTACK_CHANGES,
            "the chapter after the card began before the script moved the fight on",
        );

        // ── the hit, and the four ways on over the frozen game, with the chapter that was lost named above
        // them and the cursor on the item that puts it back.
        dies_in(&game, 4, 0);
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
        assert_eq!(
            game.a_frame_that_says(&format!("{THE_MIDBOSS} からやり直す？"))
                .len(),
            1,
            "the question does not name the chapter it goes to",
        );
        assert_eq!(
            game.says("今のチャプターには戻れません").len(),
            1,
            "the question does not say the chapter the run is in cannot be come back to",
        );
        assert_eq!(
            game.state().to_string(),
            at_the_death.to_string(),
            "something was put back by a question nobody had answered yet",
        );

        // Answered はい, and the run comes back to the frame that chapter began on — every field of this
        // game's own state as the chapter had it.
        answers_yes(&game, "retry chapter 2 (retry 1)");
        assert_eq!(
            now_but_the_deaths(&game, &at_the_midboss),
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
        // `the_run_of_th06c_written_down_off_the_frame` — so the line lands whenever that thread is given a
        // turn. Nothing of the record has to be dropped for that: it is one entry per stage frame, and the
        // frames under a chapter are the ones that reached it.
        let at = at_the_midboss.stage_frames;
        let written = format!("chapter 2 ({THE_MIDBOSS}) at frame {at}, {at} frame(s) of buttons");
        game.frames_until_a_thread("the chapter written down", || {
            log.said_since(going_back, &written)
        });

        // ── And the stage played on from there, its script putting the card up again. What the list holds
        // now is what the stage still has: the chapters that came after the one restored went with it, so the
        // card's chapter is the one that was lost and the list behind it is the midboss and the stage's start.
        let from = log.written();
        game.frames_until("the card's chapter again", THE_FIGHTS_FRAMES, || {
            log.said_since(from, "chapter 3 at frame")
        });
        let at_the_card = game.state();
        dies_in(&game, 3, from);
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
            now_but_the_deaths(&game, &at_the_card),
            at_the_card,
            "the run is not where the card's chapter began, field for field",
        );
        assert!(
            at_the_card.stage_frames >= CARD_STARTS,
            "the card's chapter began before the script put the card up",
        );
    });
}
