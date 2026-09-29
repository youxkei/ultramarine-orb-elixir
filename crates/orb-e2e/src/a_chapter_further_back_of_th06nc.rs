//! **更に前からやり直す over a run of th06nc**: the chapters the stage has behind the one that was lost, and
//! the run put back at any of them.
//!
//! The same question `a_chapter_further_back_of_th06c.rs` puts to that game, asked of this one's own fields.
//! **What made it a question worth asking only now is the table**: a stage of th06nc was one chapter until its
//! fight started while `game/th06nc/chapters.rs` held seven empty rows, so what a list behind a lost chapter
//! could hold was the fight's chapters and nothing of the waves. The fight is still what a stage of this
//! laid-out game has two chapters of first — its midboss arriving at [`MIDBOSS_ARRIVES`] and putting its card
//! up at [`CARD_STARTS`], the table's first boundary of stage one being thousands of frames past it — and what
//! is asked of it is the whole of what a snapshot of this game is worth: the run back at a chapter field for
//! field, and the file it is written down in following it there.
//!
//! **Field for field but the deaths**, which orb counts itself — see `Th06nc::read_state` — so a chapter put
//! back leaves the count where the hit that lost it put it.
//!
//! **The item left out where nothing is behind the chapter is not here**:
//! `a_confirmation_declined_of_th06nc.rs` already reads the three ways on over a death in the stage's own
//! first chapter.

use crate::fake::th06nc::{ATTACK_CHANGES, CARD_STARTS, Fake, MIDBOSS_ARRIVES, MIDBOSS_BEATEN};
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::game::State;
use orb_core::menu_ui::{LINE_HEIGHT, NORMAL, SELECTED};
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **One line of the menu as it reaches this game's back buffer**, which is not the overlay's own 640x480:
/// the canvas lays the overlay over the whole of the buffer the swapchain says it has.
const A_LINE: f32 =
    LINE_HEIGHT * crate::fake::BACK_BUFFER.1 as f32 / orb_core::overlay::SCREEN_HEIGHT;

/// How many frames the fight's own boundaries are waited for over: past the last of them.
const THE_FIGHTS_FRAMES: u32 = MIDBOSS_BEATEN;

/// What the four chapters this run reaches are called, in the order it reaches them.
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
/// nothing fired at the player.
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

/// The run as it stands now, with the deaths carried over from `began`.
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
/// which one is under the cursor — each one quad, so an item drawn twice fails here.
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

#[test]
fn the_chapters_behind_the_one_lost_in_th06nc_are_listed_and_are_where_choosing_one_puts_the_run() {
    in_its_own_process(|| {
        let game = a_run("th06nc-going-back-further");
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

        // ── the hit, and the four ways on over the frozen game.
        dies_in(&game, 4, 0);
        let at_the_death = game.state();
        reads(&game, AFTER_THE_CARD, &[AGAIN, FURTHER, STAGE, QUIT], AGAIN);

        // ── 更に前からやり直す: the chapters behind the one lost, newest first, the lost one not among them.
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

        // ── The midboss arriving, two chapters back, and the question it asks before it acts — which puts
        // nothing back while nobody has answered it.
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

        // Answered はい, and the run comes back to the frame that chapter began on.
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
        // And the file this run is written down in follows it back, written off the frame on orb's own
        // thread — so waited for, and held against the mark taken before the restore.
        let at = at_the_midboss.stage_frames;
        let written = format!("chapter 2 ({THE_MIDBOSS}) at frame {at}, {at} frame(s) of buttons");
        game.frames_until_a_thread("the chapter written down", || {
            log.said_since(going_back, &written)
        });

        // ── And the stage played on from there: what the list holds now is what the stage still has.
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

        // ── And the way out of that screen: cancel goes back to the four ways on with nothing put back.
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
