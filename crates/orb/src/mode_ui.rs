//! The question orb puts over the game's own title menu, drawn: whether a run keeps chapters, and
//! which of the two rankings is being looked at.
//!
//! What it decides — the two modes, the cursor, the keys it answers to and what each choice says —
//! is [`orb_core::mode`], because all of that is a function of a keyboard and a pad and so is
//! something a test can drive. What is here is the labels it draws them with, which needs the GDI
//! and a Direct3D device.

use crate::game::{Menu, Pad};
use crate::input::Keyboard;
use crate::menu_ui::{self, ASIDE, By, DIM_SCREEN, LINE_HEIGHT, NORMAL};
use crate::overlay::{Label, Overlay, SCREEN_HEIGHT, SCREEN_WIDTH};

pub use orb_core::mode::{Answer, Mode};
use orb_core::mode::{CHOICES, Question, aside, title};

/// How many lines the longest of `mode::aside`'s descriptions is, and so how many labels are kept
/// for whichever one is up. A number rather than something read off the descriptions, which a
/// `match` cannot be asked for in a const; a test holds the two together.
const ASIDE_LINES: usize = 3;

pub struct ModeMenu {
    question: Question,
    title: Label,
    choices: [Label; CHOICES.len()],
    aside: [Label; ASIDE_LINES],
    cursor: Label,
}

impl ModeMenu {
    /// `current` is what orb is in now, which is where the cursor starts: the answer most likely
    /// to be wanted is the one that was wanted last time.
    pub fn new(asked: Menu, current: Mode) -> Self {
        Self {
            question: Question::new(asked, current),
            title: Label::new(),
            choices: [Label::new(), Label::new()],
            aside: [const { Label::new() }; ASIDE_LINES],
            cursor: Label::new(),
        }
    }

    /// Returns what was answered, once something is. `pad` is what the pad is doing now, which the
    /// caller asks the game to read: the game is frozen here, so its own reading of the pad is not
    /// running and a pad would otherwise do nothing on this menu at all.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Answer, By)> {
        self.question.update(keyboard, pad)
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let asked = self.question.asked();
        let said = aside(asked, self.question.selected());
        unsafe {
            self.title.set(overlay, title(asked));
            self.cursor.set(overlay, "▶");
            for (label, (_, text)) in self.choices.iter_mut().zip(CHOICES) {
                label.set(overlay, text);
            }
            for (label, line) in self.aside.iter_mut().zip(said.iter().copied()) {
                label.set(overlay, line);
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        // The whole screen, not the play field: this is over the game's own menu.
        frame.fill(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, DIM_SCREEN);

        let center = SCREEN_WIDTH / 2.0;
        let y = SCREEN_HEIGHT / 2.0 - LINE_HEIGHT * 3.0;
        menu_ui::centred(&frame, &self.title, center, y, NORMAL);

        let mut y = menu_ui::list(
            &frame,
            &self.choices,
            &self.cursor,
            center,
            y + LINE_HEIGHT * 2.0,
            self.question.selection(),
        );
        // Only the lines this choice has: the labels past them still hold whatever the choice
        // before said, the ones that are set being set by what is up now.
        for label in self.aside.iter().take(said.len()) {
            y += LINE_HEIGHT;
            menu_ui::centred(&frame, label, center, y, ASIDE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ASIDE_LINES, Mode, ModeMenu};
    use crate::game::Menu;
    use crate::menu_ui::{ASIDE, DIM_SCREEN};
    use crate::overlay::{SCREEN_HEIGHT, SCREEN_WIDTH};
    use crate::recording::{Quad, Screen};
    use orb_core::mode::aside;

    /// Every description fits the labels the menu keeps for one: a line past those is a line
    /// nothing draws. Here rather than beside `aside` itself, `ASIDE_LINES` being how many labels
    /// this menu holds and so a fact about the drawing.
    #[test]
    fn no_description_is_longer_than_the_labels_kept_for_it() {
        for asked in [Menu::Run, Menu::Scores] {
            for mode in [Mode::Pointdevice, Mode::Normal] {
                assert!(aside(asked, mode).len() <= ASIDE_LINES);
            }
        }
    }

    fn frame(screen: &Screen, menu: &mut ModeMenu) -> Vec<Quad> {
        screen.frame(|overlay| unsafe { menu.draw(overlay) })
    }

    /// The whole screen is washed, not the play field: this question goes over the game's own title
    /// menu, and a wash the size of the play field would leave the menu readable around it.
    #[test]
    fn the_whole_screen_is_dimmed_under_the_question() {
        let screen = Screen::new();
        let quads = frame(&screen, &mut ModeMenu::new(Menu::Run, Mode::Normal));

        let wash = *quads.first().expect("something was drawn first");
        assert_eq!((wash.x, wash.y), (0.0, 0.0));
        assert_eq!((wash.width, wash.height), (SCREEN_WIDTH, SCREEN_HEIGHT));
        assert_eq!(wash.color, DIM_SCREEN);
    }

    /// The description under the choices is drawn in the aside's own colour, and there is as much of
    /// it as the choice has lines to say. A ranking has none, and the labels holding the run's lines
    /// must not be drawn there — they still hold whatever was said last.
    #[test]
    fn only_the_lines_this_choice_has_are_drawn() {
        let screen = Screen::new();

        // The cursor starts on the mode the run is already in, so what is described is that one's.
        // One quad a line: a label's drop shadow is drawn in the shadow's colour, not the aside's.
        let run = frame(&screen, &mut ModeMenu::new(Menu::Run, Mode::Normal));
        let described = run.iter().filter(|quad| quad.color == ASIDE).count();
        assert_eq!(described, aside(Menu::Run, Mode::Normal).len());

        let scores = frame(&screen, &mut ModeMenu::new(Menu::Scores, Mode::Normal));
        assert!(
            !scores.iter().any(|quad| quad.color == ASIDE),
            "a ranking is asked with nothing under either choice",
        );
        // And it is a shorter screen for it.
        let lowest = |quads: &[Quad]| {
            quads
                .iter()
                .map(|quad| quad.y)
                .max_by(f32::total_cmp)
                .expect("something was drawn")
        };
        assert!(lowest(&scores) < lowest(&run));
    }
}
