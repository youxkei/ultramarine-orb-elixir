//! **The same launch of th06nc on a machine whose own windows are English.**
//!
//! Every other e2e test over this game reads orb's screens in Japanese, that being the language the game
//! is in and the one a simulated machine is set to — see [`LANGUAGE`](crate::fake::LANGUAGE). This one
//! declares the machine instead, with nothing in `orb.yaml` about the language, so the answer comes from
//! the only place left: the language Windows is showing its own windows in.
//!
//! **What is asserted is the words on the screen and not that a function was called.** A screen is
//! localised when what it drew is the English wording and none of the Japanese is anywhere on it, so both
//! are read back — the Japanese by name, because a table with one cell in the wrong language is the
//! mistake two languages make and it draws a screen that looks finished.
//!
//! **One screen here, the same one th06c's module takes**, and for the same reason: the wordings and the
//! picking between them are `orb_core::mode`'s and `menu_ui`'s, which the th06 and th07 modules already
//! hold to both languages. What is this game's own is **English drawn through a canvas over a
//! swapchain** — no Direct3D 8 device, glyphs uploaded as textures of the canvas's own, and the face they
//! are baked in one the machine lent — so the screen taken is the one that reaches that drawing, which is
//! the question over the menu's first item.

use crate::fake::th06nc::{Fake, TO_A_RUN};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::{Language, LogLevel};
use orb_core::game::Menu;
use orb_core::menu_ui::SELECTED;
use orb_core::mode::{self, Mode};
use orb_sim::langid;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How many frames the walk to the question is given. The recipe's own bounds are longer than this
/// between them; what this is is the wall the test puts up rather than sitting in a loop.
const FRAMES: u32 = 4000;

/// **The decide at the menu's first item that raises the question**, which is [`TO_A_RUN`]'s own next
/// step: this game's title is a screen of its own, so the walk reaches the menu and the question is one
/// press further.
const AND_THE_QUESTION: &str = "\
- press: [decide]
  frames: 6
  until: { question: mode }
  give_up_after: 1800
";

/// A launch on a machine whose windows are English, walked to the question over the menu's own first
/// item.
///
/// # Panics
/// If orb did not say which language it settled on and where the answer came from, or if the canvas did
/// not come up: the line about the language is what a screenshot of a screen somebody could not read is
/// held against, and the one about the drawing is what every word below is drawn through.
fn asking_in_english(name: &str) -> Box<Fake> {
    let game = Fake::attach_on_a_machine_in(
        langid::ENGLISH,
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(format!("{TO_A_RUN}{AND_THE_QUESTION}"));
            config.log_level = LogLevel::Verbose;
            // Said rather than left as it comes: what this is about is the answer coming from the
            // machine, and a launch that had set the language would be asking a different question.
            assert_eq!(config.language, None, "the file says which language");
        },
    );
    let log = game.log();
    assert!(
        log.said("language: english, which is what this machine's own windows are in"),
        "nothing said which language orb settled on:\n  {}",
        log.lines().join("\n  ")
    );
    game.frames_until("a canvas over this game's swapchain", FRAMES, || {
        log.said("canvas: drawing through this game's own Direct3D 11")
    });
    game.frames_until("the mode question", FRAMES, || {
        log.said("menu: Run is under the cursor, asking which mode")
    });
    game
}

/// Every line of the screen, held against both languages at once: the English on it once, and none of
/// the Japanese anywhere.
///
/// `lit` names the one line that has to be under the cursor, which is what makes the screen answerable —
/// a menu with nothing lit is one nobody could choose from.
///
/// # Panics
/// Naming whichever line is missing, doubled, or in the language this machine is not in.
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

/// The question over the game's own main menu: its title, the two modes, and what the mode under the
/// cursor means — every line of it in English, through this game's own Direct3D 11.
#[test]
fn the_mode_question_over_th06nc_is_in_english_on_an_english_machine() {
    in_its_own_process(|| {
        let game = asking_in_english("th06nc-english-mode-question");
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
