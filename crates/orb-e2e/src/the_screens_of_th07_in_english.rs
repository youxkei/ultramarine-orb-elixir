//! **The same launch of th07 on a machine whose own windows are English.**
//!
//! Every other e2e test over this game reads orb's screens in Japanese, that being the language the game is in
//! and the one a simulated machine is set to — see [`LANGUAGE`](crate::fake::LANGUAGE). This one declares the
//! machine instead, with nothing in `orb.yaml` about the language, so the answer comes from the only place left:
//! the language Windows is showing its own windows in.
//!
//! **What is asserted is the words on the screen and not that a function was called.** A screen is localised when
//! what it drew is the English wording and none of the Japanese is anywhere on it, so both are read back — the
//! Japanese by name, because a table with one cell in the wrong language is the mistake two languages make and it
//! draws a screen that looks finished.
//!
//! And it is asked of this game because the *face* is this game's own: th07 ships no `font.ttf`, so every screen
//! here is drawn in a face the machine lent — see
//! [docs/adr/0019](../../../docs/adr/0019-a-game-that-ships-no-font-is-drawn-in-the-machines-own-face.md) — and a
//! face lent for one language is not evidence about the other.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, READS_KEYS_AFTER, Work, in_its_own_process};
use orb_config::{Language, LogLevel};
use orb_core::game::Menu;
use orb_core::menu_ui::SELECTED;
use orb_core::mode::{self, Mode};
use orb_sim::{keys, langid};

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The card the midboss puts up, by its own record, named the way th07 names cards.
const CARD: usize = 96;
const CARD_NAME: &str = "幽夢「猫の見た夢」";

/// The frames between one part of the fight and the next: past the shortest a chapter may be.
const APART: u32 = 120;

/// A launch on a machine whose windows are English, with `orb.yaml` saying nothing about the language.
///
/// # Panics
/// If orb did not say which language it settled on and where the answer came from. That line is what a screenshot
/// of a screen somebody could not read is held against, so a launch without it is one this e2e test cannot claim
/// anything about either.
fn on_an_english_machine(name: &str) -> Box<Fake> {
    let game = Fake::attach_on_a_machine_in(
        langid::ENGLISH,
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.log_level = LogLevel::Verbose;
            // Said rather than left as it comes: what this e2e test is about is the answer coming from the
            // machine, and a launch that had set the language would be asking a different question.
            assert_eq!(config.language, None, "the file says which language");
        },
    );
    assert!(
        game.log()
            .said("language: english, which is what this machine's own windows are in"),
        "nothing said which language orb settled on:\n  {}",
        game.log().lines().join("\n  ")
    );
    game
}

/// Every line of a screen, held against both languages at once: the English on it once, and none of the Japanese
/// anywhere.
///
/// `lit` names the one line that has to be under the cursor, which is what makes the screen answerable — a menu
/// with nothing lit is one nobody could choose from. `None` where the line under the cursor is not one of these:
/// the retry menu's is a chapter's name, which is the same in both languages.
///
/// # Panics
/// Naming whichever line is missing, doubled, or in the language this machine is not in.
fn reads(game: &Fake, said: &[(&str, &str)], lit: Option<&str>) {
    game.forget();
    game.one_frame();
    for (english, japanese) in said {
        let drawn = game.says(english);
        assert_eq!(drawn.len(), 1, "{english} is not on the screen once");
        assert!(
            game.says(japanese).is_empty(),
            "{japanese} is on a screen this machine reads in English",
        );
        if lit == Some(english) {
            assert_eq!(drawn[0].color, SELECTED, "the cursor is not on {english}");
        }
    }
}

/// The question over the game's own title menu: its title, the two modes, and what the mode under the cursor
/// means.
#[test]
fn the_question_over_the_title_menu_of_th07_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("th07-english-mode-question");
        let log = game.log();
        game.frames_until("an overlay", 8, || log.said("overlay: ready"));
        game.image().the_front_end_is_up();
        game.image().the_title_menu_points_at_a_run();
        game.image().the_screen_has_been_up_for(60);
        game.press_until(keys::Z, "the mode question", || {
            log.said("menu: Run is under the cursor, asking which mode")
        });

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
        // The cursor starts on the mode orb is already in, which for a launch nobody has answered yet is the one
        // with chapters — so what is described under the two is that one's.
        let lines = mode::aside(Menu::Run, Mode::Pointdevice, Language::English)
            .iter()
            .zip(mode::aside(
                Menu::Run,
                Mode::Pointdevice,
                Language::Japanese,
            ))
            .map(|(english, japanese)| (*english, *japanese));
        said.extend(lines);
        reads(
            &game,
            &said,
            Some(Mode::Pointdevice.name(Language::English)),
        );
    });
}

/// And the menu that appears where a chapter was lost, with the question one of its items asks.
#[test]
fn the_menu_where_a_chapter_of_th07_was_lost_is_in_english() {
    in_its_own_process(|| {
        let game = on_an_english_machine("th07-english-retry-menu");
        let log = game.log();
        game.streams_its_song(0);
        game.in_a_pointdevice_run();
        game.frames(APART);
        game.the_boss_arrives();
        game.frames_until("the midboss's chapter", APART, || log.said("chapter 2"));
        game.frames(APART);
        game.a_card_starts(CARD, CARD_NAME);
        game.frames_until("the card's chapter", APART, || log.said("chapter 3"));
        game.dies();
        game.frames_until("the death seen", 8, || log.said("died in chapter 3"));

        reads(
            &game,
            &[
                ("Retry the chapter", "チャプターをやり直す"),
                ("Retry from further back", "更に前からやり直す"),
                ("Retry the stage", "ステージをやり直す"),
                ("Back to the title screen", "タイトルに戻る"),
            ],
            Some("Retry the chapter"),
        );

        // The screen behind the second item, which is a screen of its own with a question of its own above what it
        // lists. The chapters it lists are named by the boundary detector — the names the status line and the log
        // call them by — and those stay as they are in either language.
        game.frames(READS_KEYS_AFTER);
        game.press(keys::DOWN);
        game.press_until(keys::Z, "the chapters behind it", || {
            log.said("retry: asking which chapter further back")
        });
        reads(&game, &[("Where to start again", "どこからやり直す")], None);
        assert_eq!(
            game.says("MIDSTAGE 1").len(),
            1,
            "the stage's own start is not listed",
        );

        // And the question one of them asks, which names the chapter in whichever language the rest of the line is
        // in, and says under it what going back leaves behind.
        game.frames(READS_KEYS_AFTER);
        game.press_until(keys::Z, "the chapter asked about", || {
            log.said("retry: asking about a chapter further back")
        });
        reads(
            &game,
            &[
                (
                    "Start again from MIDBOSS NONSPELL 1?",
                    "MIDBOSS NONSPELL 1 からやり直す？",
                ),
                (
                    "There is no going back to the chapter you are in",
                    "今のチャプターには戻れません",
                ),
                ("Yes", "はい"),
                ("No", "いいえ"),
            ],
            // The cursor starts on no, which is what makes a press on the frame the question begins reading keys
            // on cost nothing.
            Some("No"),
        );
    });
}
