//! **更に前からやり直す over a run of th08 that has reached no chapter but its first: the item left out.**
//!
//! The same question `a_chapter_further_back_of_th07.rs` puts to that game, asked of this one at the one place
//! the answer is a menu with an item fewer. What is behind the chapter lost would be a screen with nothing on
//! it: the stage's own start is the chapter being played, and the two items that put it back are the first
//! and the third. The list behind a later chapter, and the run put back at one of its chapters, are the
//! real-game cases `a_chapter_further_back_taken_again` and `a_death_answered_by_a_chapter_further_back`,
//! which `the_cases_of_th08_the_real_game_is_tested_with` plays over the laid-out game.
//!
//! **And what this game has no answer to is the pause menu's own 「最初からやり直す」**, which th07's file asks
//! about second: `Th08` does not answer `Game::asked_to_start_the_run_over`, whose default is no, so the ask
//! is the game's own to carry out — nothing of this game's pause menu's items has been read.

use orb_config::LogLevel;
use orb_core::menu_ui::{LINE_HEIGHT, NORMAL, SELECTED};

use crate::fake::th08::{BOMBS, Fake, LIVES};
use crate::fake::{Display, Launched, Work, in_its_own_process};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The frames a stage's build and its first chapter are given.
const FRAMES_FOR_A_BUILD: u32 = 1_200;

/// The chapter a death in the stage's first frames is lost in, by the name the retry menu gives it.
const STAGE_START: &str = "MIDSTAGE 1";

/// The four ways on, in the order they are offered.
const AGAIN: &str = "チャプターをやり直す";
const FURTHER: &str = "更に前からやり直す";
const STAGE: &str = "ステージをやり直す";
const QUIT: &str = "タイトルに戻る";

/// One frame of a stage placed by hand, the run's clock stepped the way the stage's own jobs step it and the
/// script's left standing, so that no row of the stage's table comes up under what this is about.
fn a_frame_of_the_stage(game: &Fake) {
    let image = game.image();
    image.has_been_in_the_run_for(image.the_frames_the_run_has_been_going() + 1);
    game.frame();
}

/// The lines on the screen now: the chapter written above them, the items in the order they were listed, and
/// which one is under the cursor. Each is one quad because each is one line drawn once.
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

/// **A death with nothing behind the chapter it happened in leaves that item out**, read off the screen
/// rather than out of the menu, because what an item costs when it does nothing is paid there: a line somebody
/// walks the cursor onto to be shown an empty list.
#[test]
fn the_chapters_behind_it_are_not_offered_where_a_run_of_th08_has_reached_no_other() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th08-going-back-from-the-first",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.its_title_menu_is_up();
        game.is_in_a_run();
        for _ in 0..FRAMES_FOR_A_BUILD {
            if log.said("stage 1 chapter 1 (stage start)") {
                break;
            }
            a_frame_of_the_stage(&game);
        }
        assert!(
            log.said("stage 1 chapter 1 (stage start)") && !log.said("chapter 2 at frame"),
            "the run is not standing in the stage's own first chapter and no other:\n  {}",
            log.lines().join("\n  ")
        );

        game.image().has_died(1.0);
        game.image().has_lives_and_bombs(LIVES - 1.0, BOMBS);
        game.image().the_player_has_been_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 1"));
        reads(&game, STAGE_START, &[AGAIN, STAGE, QUIT], AGAIN);
        assert!(
            game.says(FURTHER).is_empty(),
            "an item was offered for chapters the run has not reached",
        );
    });
}
