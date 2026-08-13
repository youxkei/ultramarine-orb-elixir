//! **The same launch on a machine whose own windows are English.**
//!
//! Every other e2e test here reads orb's screens in Japanese, that being the language the game is in and
//! the one a simulated machine is set to — see [`LANGUAGE`](crate::fake::LANGUAGE). This one declares the
//! machine instead, with nothing in `orb.yaml` about the language, so the answer comes from the only
//! place left: the language Windows is showing its own windows in.
//!
//! **What is asserted is the words on the screen and not that a function was called.** A screen is
//! localised when what it drew is the English wording and none of the Japanese is anywhere on it, so
//! both are read back — the Japanese by name, because a table with one cell in the wrong language is
//! the mistake two languages make and it draws a screen that looks finished.
//!
//! The two screens here are the two a session meets whether or not anything goes wrong: the question
//! over the title menu, and the menu that appears where a chapter was lost. The question about a run
//! left unfinished is the third, and it is here too — its own words plus the mark the shot type select
//! carries before it is asked.

use crate::fake::th06::{CARD_STARTS, Fake};
use crate::fake::{Launched, READS_KEYS_AFTER, in_its_own_process};
use orb_config::{Language, LogLevel};
use orb_core::game::th06::image::{Scene, Screen};
use orb_core::game::{Menu, RunStart};
use orb_core::menu_ui::SELECTED;
use orb_core::mode::{self, Mode};
use orb_sim::{keys, langid};

/// The run every game here plays: Normal, Reimu A, from stage one — the run the rest of the e2e tests
/// play, so that nothing about this one is unusual except the machine it is on.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// A launch on a machine whose windows are English, with `orb.yaml` saying nothing about the language.
///
/// # Panics
/// If orb did not say which language it settled on and where the answer came from. That line is what a
/// screenshot of a screen somebody could not read is held against, so a launch without it is one this
/// e2e test cannot claim anything about either.
fn on_an_english_machine(name: &str) -> Box<Fake> {
    let game = Fake::attach_on_a_machine_in(langid::ENGLISH, name, the_run(), |config| {
        config.log_level = LogLevel::Verbose;
        // Said rather than left as it comes: what this e2e test is about is the answer coming from the
        // machine, and a scenario that had set the language would be asking a different question.
        assert_eq!(config.language, None, "the file says which language");
    });
    assert!(
        game.log()
            .said("language: english, which is what this machine's own windows are in"),
        "nothing said which language orb settled on:\n  {}",
        game.log().lines().join("\n  ")
    );
    game
}

/// Every line of a screen, held against both languages at once: the English on it once, and none of the
/// Japanese anywhere.
///
/// `lit` names the one line that has to be under the cursor, which is what makes the screen answerable —
/// a menu with nothing lit is one nobody could choose from.
///
/// # Panics
/// Naming whichever line is missing, doubled, or in the language this machine is not in.
fn reads(game: &Fake, said: &[(&str, &str)], lit: &str) {
    game.forget();
    game.frame();
    for (english, japanese) in said {
        let drawn = game.says(english);
        assert_eq!(drawn.len(), 1, "{english:?} is not on the screen once");
        if *english == lit {
            assert_eq!(
                drawn[0].color, SELECTED,
                "{english:?} is not under the cursor"
            );
        }
        assert!(
            game.says(japanese).is_empty(),
            "{japanese:?} is on the screen of a machine whose windows are English",
        );
    }
}

/// The question over the game's own title menu: its title, the two modes, and what the mode under the
/// cursor means.
#[test]
fn the_question_over_the_title_menu_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("english-mode-question");
        game.at_the_title_menu();
        game.press(keys::Z);
        game.one_frame();

        let mut said = vec![
            (
                mode::title(Menu::Run, Language::English),
                mode::title(Menu::Run, Language::Japanese),
            ),
            (
                Mode::Pointdevice.name(Language::English),
                Mode::Pointdevice.name(Language::Japanese),
            ),
            (
                Mode::Normal.name(Language::English),
                Mode::Normal.name(Language::Japanese),
            ),
        ];
        // The cursor starts on the mode orb is already in, which for a launch nobody has answered yet is
        // the one with chapters — so what is described under the two is that one's.
        let lines = mode::aside(Menu::Run, Mode::Pointdevice, Language::English)
            .iter()
            .zip(mode::aside(
                Menu::Run,
                Mode::Pointdevice,
                Language::Japanese,
            ))
            .map(|(english, japanese)| (*english, *japanese));
        said.extend(lines);
        reads(&game, &said, Mode::Pointdevice.name(Language::English));
    });
}

/// And the menu that appears where a chapter was lost, with the question one of its items asks.
#[test]
fn the_menu_where_a_chapter_was_lost_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("english-retry-menu");
        let log = game.log();
        game.in_a_pointdevice_run();
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

        reads(
            &game,
            &[
                ("Retry the chapter", "チャプターをやり直す"),
                ("Retry the stage", "ステージをやり直す"),
                ("Back to the title screen", "タイトルに戻る"),
            ],
            "Retry the chapter",
        );

        // And the question the second item asks, which is a screen of its own inside the same field.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the stage asked about", || {
            log.said("retry: asking about the stage again")
        });
        reads(
            &game,
            &[
                (
                    "Start the stage over from the beginning?",
                    "ステージの最初からやり直す？",
                ),
                ("Yes", "はい"),
                ("No", "いいえ"),
            ],
            // The cursor starts on no, which is what makes a press on the frame the question begins
            // reading keys on cost nothing.
            "No",
        );
    });
}

/// And the question about a run left unfinished, with the mark that goes up before it is asked.
#[test]
fn the_question_about_a_run_left_unfinished_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("english-resume-question");
        let log = game.log();
        game.in_a_pointdevice_run();
        game.frames_until("the card's chapter", CARD_STARTS + 400, || {
            log.said(&format!(
                "chapter 3 at frame {CARD_STARTS} (script {CARD_STARTS}): a midboss spellcard"
            ))
        });
        // やめる at the game's own pause, which is the one way out of a run that leaves its chapter
        // written down: the run did not finish, so nothing takes the file away.
        game.gives_the_run_up_at_its_own_pause();
        game.frames_until("the front end", 60, || {
            game.image().scene() == Scene::FrontEnd
        });

        // The same run chosen again, as far as the shot type select — where the mark says that the run
        // under the cursor has a chapter written down, and the press asks about it.
        game.frames_until("the title menu ready to act on a press", 300, || {
            let front = game.image().front_end_now();
            game.image().scene() == Scene::FrontEnd
                && front.screen == Screen::Title
                && front.acts_on_a_press()
        });
        game.press(keys::Z);
        game.press_until(keys::Z, "the mode question answered again", || {
            game.image().front_end_now().screen == Screen::ShotType
        });
        game.frames_until("the shot type select ready to act on a press", 90, || {
            game.image().front_end_now().acts_on_a_press()
        });
        game.forget();
        game.frame();
        assert_eq!(
            game.says("A run left unfinished").len(),
            1,
            "the mark is not on the shot type select of a machine whose windows are English",
        );
        assert!(
            game.says("中断データあり").is_empty(),
            "the mark is in Japanese on a machine whose windows are English",
        );

        game.press(keys::Z);
        game.frames_until("the question", 8, || {
            log.said("resume: normal-reimu-a was left; asking where to start")
        });
        reads(
            &game,
            &[
                ("Where to start", "どこから始める"),
                ("Continue", "つづきから"),
                ("From the beginning", "はじめから"),
            ],
            // The cursor starts on the chapter that was left: a run picked up by accident is a run put
            // back where it was, where a fresh one writes over what was left.
            "Continue",
        );
        // And what the other item costs, which is said only while the cursor is on it.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.forget();
        game.frame();
        assert_eq!(
            game.says("The run left unfinished will be written over")
                .len(),
            1,
            "what starting again costs is not said in English",
        );
        assert!(
            game.says("中断データは上書きされます").is_empty(),
            "what starting again costs is said in Japanese",
        );
    });
}
