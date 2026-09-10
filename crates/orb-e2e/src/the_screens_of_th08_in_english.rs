//! **The same launch of th08 on a machine whose own windows are English.**
//!
//! Every other e2e test over this game reads orb's screens in Japanese, that being the language the game is
//! in and the one a simulated machine is set to — see [`LANGUAGE`](crate::fake::LANGUAGE). This one declares
//! the machine instead, with nothing in `orb.yaml` about the language, so the answer comes from the only
//! place left: the language Windows is showing its own windows in.
//!
//! **What is asserted is the words on the screen and not that a function was called.** A screen is localised
//! when what it drew is the English wording and none of the Japanese is anywhere on it, so both are read
//! back — the Japanese by name, because a table with one cell in the wrong language is the mistake two
//! languages make and it draws a screen that looks finished.
//!
//! **One screen, the question over the title**, for `the_screens_of_th06c_in_english`'s reason: the wordings
//! and the picking between them are `orb_core::mode`'s and `menu_ui`'s, which other games' files hold to both
//! languages already. What is this game's is the face it is drawn in, which the machine lends — th08 ships no
//! font of its own — and a face lent for one language is not evidence about the other.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{Language, LogLevel};
use orb_core::game::Menu;
use orb_core::menu_ui::SELECTED;
use orb_core::mode::{self, Mode};
use orb_sim::langid;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames the walk to the question is given.
const FRAMES: u32 = 4000;

/// **The walk to the question over the title's first item, and a wait under it** — the first steps of
/// `fake::th08::TO_A_RUN` and no answer, the question being what is read here.
const TO_THE_QUESTION: &str = "\
- wait_until: { screen: 0 }
  give_up_after: 1800
- press: [up]
  frames: 30
  until: { cursor: 0 }
  give_up_after: 900
- press: [decide]
  frames: 30
  until: { question: mode }
  give_up_after: 1800
- wait_until: { frames: 600 }
- quit: true
";

/// A launch on a machine whose windows are English, walked to the question over the title's own Start item.
fn asking_in_english(name: &str) -> Box<Fake> {
    let game = Fake::attach_on_a_machine_in(
        langid::ENGLISH,
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(TO_THE_QUESTION.to_owned());
            config.log_level = LogLevel::Verbose;
            // Said rather than left as it comes: what this is about is the answer coming from the machine,
            // and a launch that had set the language would be asking a different question.
            assert_eq!(config.language, None, "the file says which language");
        },
    );
    let log = game.log();
    assert!(
        log.said("language: english, which is what this machine's own windows are in"),
        "nothing said which language orb settled on:\n  {}",
        log.lines().join("\n  ")
    );
    game.frames_until("an overlay", FRAMES, || log.said("overlay: ready"));
    game.frames_until("the mode question", FRAMES, || {
        log.said("menu: Run is under the cursor, asking which mode")
    });
    game
}

/// Every line of the screen, held against both languages at once: the English on it once, and none of the
/// Japanese anywhere — with `lit` the one line that has to be under the cursor.
fn reads(game: &Fake, said: &[(&str, &str)], lit: &str) {
    game.forget();
    game.one_frame();
    for (english, japanese) in said {
        let drawn = game.says(english);
        assert_eq!(drawn.len(), 1, "{english} is not on the screen once");
        assert!(
            game.says(japanese).is_empty(),
            "{japanese} is on a screen this machine reads in English",
        );
        if lit == *english {
            assert_eq!(drawn[0].color, SELECTED, "the cursor is not on {english}");
        }
    }
}

/// The question over the game's own title menu: its title, the two modes, and what the mode under the cursor
/// means — every line of it in English.
#[test]
fn the_mode_question_over_th08_is_in_english_on_an_english_machine() {
    in_its_own_process(|| {
        let game = asking_in_english("th08-english-mode-question");
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
