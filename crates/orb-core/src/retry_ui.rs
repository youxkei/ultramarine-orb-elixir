//! The menu that appears where the chapter was lost.
//!
//! How it reads its keys and draws its items is [`crate::menu_ui`], which the other two questions
//! orb asks share.

use orb_config::Language;

use crate::game::{Pad, Rect};
use crate::input::Keyboard;
use crate::log;
use crate::menu_ui::{self, By, DIM_FIELD, Keys, LINE_HEIGHT, NORMAL, Pressed};
use crate::overlay::{Label, Overlay};

/// Frames before the menu accepts anything. The player was holding keys when they
/// died — very likely a direction and the shoot key — and those presses belong to
/// the run, not to this menu.
const INPUT_GRACE_FRAMES: u32 = 24;

/// Frames before a confirmation accepts anything.
///
/// Shorter than the menu's own grace, and for a different reason: what is being kept out
/// is not the run's keys but the press that opened the question. That press is an edge and
/// so is already spent, which is what makes this a few frames rather than a fifth of a
/// second — but a question answered on the frame after it appeared is a question nobody
/// read, and the answer here throws a stage or a run away.
const CONFIRM_GRACE_FRAMES: u32 = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    Chapter,
    Stage,
    /// The run given up: back to the title menu, the way the game's own pause menu leaves one.
    Quit,
}

impl Choice {
    /// For the log, in the English the rest of it is in — and it stays this wording whichever language
    /// the menu is in, a log being read beside a source tree rather than by whoever is playing.
    pub fn label(self) -> &'static str {
        match self {
            Self::Chapter => "the chapter again",
            Self::Stage => "the stage again",
            Self::Quit => "the run given up",
        }
    }

    /// What the item says on screen.
    ///
    /// The third says where it ends up rather than what it gives up, because that is the part
    /// somebody reading the item does not know: the run ending is the obvious half, and that the game
    /// itself carries on is not.
    fn text(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Chapter, Language::Japanese) => "チャプターをやり直す",
            (Self::Chapter, Language::English) => "Retry the chapter",
            (Self::Stage, Language::Japanese) => "ステージをやり直す",
            (Self::Stage, Language::English) => "Retry the stage",
            (Self::Quit, Language::Japanese) => "タイトルに戻る",
            (Self::Quit, Language::English) => "Back to the title screen",
        }
    }
}

/// The three ways on, in the order they are offered. The words are [`Choice::text`]'s.
const CHOICES: [Choice; 3] = [Choice::Chapter, Choice::Stage, Choice::Quit];

/// The second question a choice asks, where it asks one.
///
/// Nothing for the chapter, which is the choice this menu exists for: it is answered every
/// time a chapter is lost, which in a fight worth grinding is every few seconds, and a
/// question in front of it would be a question answered without reading it — which is worse
/// than no question at all, since it also trains the hand that then answers the other two.
///
/// The other two are one press away from that hand and neither can be taken back: the
/// stage's start throws away everything the stage has gained since it, and giving up throws
/// away the run.
///
/// The question is the whole of what is said. A line under it spelling out what would be lost
/// was tried and taken out: the question already names what is about to happen, and a screen
/// somebody meets by dying is not the place to be read at.
fn question(choice: Choice, language: Language) -> Option<&'static str> {
    match (choice, language) {
        (Choice::Chapter, _) => None,
        (Choice::Stage, Language::Japanese) => Some("ステージの最初からやり直す？"),
        (Choice::Stage, Language::English) => Some("Start the stage over from the beginning?"),
        (Choice::Quit, Language::Japanese) => Some("やめてタイトルに戻る？"),
        (Choice::Quit, Language::English) => Some("Give up and go back to the title screen?"),
    }
}

/// Yes above no, with the cursor starting on no — which is where the game's own quit
/// question puts it, and it is what makes a press on the frame the grace ends cost nothing.
const ANSWERS: [bool; 2] = [true, false];
const NO: usize = 1;

/// What an answer says on screen.
fn answer(yes: bool, language: Language) -> &'static str {
    match (yes, language) {
        (true, Language::Japanese) => "はい",
        (true, Language::English) => "Yes",
        (false, Language::Japanese) => "いいえ",
        (false, Language::English) => "No",
    }
}

/// What the menu is showing.
#[derive(Clone, Copy)]
enum Showing {
    /// The three ways on.
    Choices,
    /// The question one of them asked, and which one asked it.
    Confirming(Choice),
}

pub struct RetryMenu {
    showing: Showing,
    selection: usize,
    /// Which answer the cursor is on, while a confirmation is up.
    answer: usize,
    /// Which language its words are in, kept for the same reason [`crate::mode_ui::ModeMenu`] keeps
    /// it: what the menu decides is the same either way, and every word of it is a function of this.
    language: Language,
    keys: Keys,
    chapter: Label,
    retry: Label,
    choices: [Label; CHOICES.len()],
    asked: Label,
    answers: [Label; ANSWERS.len()],
    cursor: Label,
}

impl RetryMenu {
    pub fn new(language: Language) -> Self {
        Self {
            showing: Showing::Choices,
            selection: 0,
            answer: NO,
            language,
            keys: Keys::new(INPUT_GRACE_FRAMES),
            chapter: Label::new(),
            retry: Label::new(),
            choices: [Label::new(), Label::new(), Label::new()],
            asked: Label::new(),
            answers: [Label::new(), Label::new()],
            cursor: Label::new(),
        }
    }

    /// Returns the choice once it is confirmed. `pad` is what the pad is doing now, read for this
    /// menu by the caller.
    ///
    /// Nothing cancels the menu itself: the player is dead, and its items are the only ways on.
    /// Cancelling a confirmation goes back to them.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Choice, By)> {
        let pressed = self.keys.read(keyboard, pad)?;
        // Which hand answered, which is only ever asked about the press that decided.
        let by = pressed.decide;
        let choice = self.step(&pressed)?;
        // `step` returns a choice only on a press that decided it, so there is a hand to name.
        Some((choice, by?))
    }

    fn step(&mut self, pressed: &Pressed) -> Option<Choice> {
        match self.showing {
            Showing::Choices => {
                self.selection = menu_ui::moved(self.selection, CHOICES.len(), pressed);
                pressed.decide?;
                let chosen = CHOICES[self.selection];
                if question(chosen, self.language).is_none() {
                    return Some(chosen);
                }
                self.showing = Showing::Confirming(chosen);
                self.answer = NO;
                self.keys.hold(CONFIRM_GRACE_FRAMES);
                log!("retry: asking about {}", chosen.label());
                None
            }
            Showing::Confirming(choice) => {
                self.answer = menu_ui::moved(self.answer, ANSWERS.len(), pressed);
                if pressed.cancel.is_some() {
                    self.back_to_choices(choice, "cancelled");
                    return None;
                }
                pressed.decide?;
                if ANSWERS[self.answer] {
                    return Some(choice);
                }
                self.back_to_choices(choice, "answered no");
                None
            }
        }
    }

    /// Said rather than passed over: what a confirmation is for is stopping something, and a
    /// session that lost a stage anyway has to be able to see whether the stop happened.
    fn back_to_choices(&mut self, choice: Choice, how: &str) {
        self.showing = Showing::Choices;
        log!("retry: {} — {how}, back to the choices", choice.label());
    }

    /// Drawn into the game's own back buffer through the D3D overlay, which is right for this menu —
    /// it belongs over the game — and carries the game's repainting with it: anything drawn this way
    /// where the game does not repaint accumulates instead of being replaced.
    ///
    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay, area: Rect, chapter: &str, retries: u32) {
        // Every label this frame will draw, baked before the overlay's frame is opened: baking
        // one creates a texture and locks it, and the frame is a window with the device's state
        // captured and replaced. Nothing about the two is known to collide, which is the reason
        // not to find out.
        unsafe {
            self.chapter.set(overlay, chapter);
            self.retry.set(overlay, &format!("RETRY {retries}"));
            self.cursor.set(overlay, "▶");
            match self.showing {
                Showing::Choices => {
                    for (label, choice) in self.choices.iter_mut().zip(CHOICES) {
                        label.set(overlay, choice.text(self.language));
                    }
                }
                Showing::Confirming(choice) => {
                    if let Some(asked) = question(choice, self.language) {
                        self.asked.set(overlay, asked);
                    }
                    for (label, yes) in self.answers.iter_mut().zip(ANSWERS) {
                        label.set(overlay, answer(yes, self.language));
                    }
                }
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        frame.fill(area.left, area.top, area.width, area.height, DIM_FIELD);

        let center = area.center_x();
        // The header and the gap under it are three lines, which puts what follows one line above
        // the middle of the field: the three items straddle it, and a confirmation — a question
        // and its two answers, with a line between — sits inside the same room.
        let mut y = area.center_y() - LINE_HEIGHT * 4.0;
        for label in [&self.chapter, &self.retry] {
            menu_ui::centred(&frame, label, center, y, NORMAL);
            y += LINE_HEIGHT;
        }
        y += LINE_HEIGHT;

        match self.showing {
            Showing::Choices => {
                menu_ui::list(
                    &frame,
                    &self.choices,
                    &self.cursor,
                    center,
                    y,
                    self.selection,
                );
            }
            Showing::Confirming(_) => {
                menu_ui::centred(&frame, &self.asked, center, y, NORMAL);
                menu_ui::list(
                    &frame,
                    &self.answers,
                    &self.cursor,
                    center,
                    y + LINE_HEIGHT * 2.0,
                    self.answer,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ANSWERS, CHOICES, CONFIRM_GRACE_FRAMES, Choice, NO, Pressed, RetryMenu, Showing, question,
    };
    use crate::game::Rect;
    use crate::menu_ui::{By, DIM_FIELD, LINE_HEIGHT, SELECTED};
    use crate::overlay::Drawing;
    use orb_config::Language;
    use orb_sim::Quad;

    /// The language every menu below is read in. Which one changes nothing these tests assert — what
    /// a press means, and where a line lands — so it is the one the game itself is in.
    const LANGUAGE: Language = Language::Japanese;

    /// A frame nothing was pressed on.
    fn nothing() -> Pressed {
        Pressed {
            up: false,
            down: false,
            decide: None,
            cancel: None,
        }
    }

    fn decide() -> Pressed {
        Pressed {
            decide: Some(By::Keyboard),
            ..nothing()
        }
    }

    fn down() -> Pressed {
        Pressed {
            down: true,
            ..nothing()
        }
    }

    fn cancel() -> Pressed {
        Pressed {
            cancel: Some(By::Keyboard),
            ..nothing()
        }
    }

    /// A menu past the grace the run's own keys are kept out by, which is where every test
    /// here starts: what is being watched is what the presses mean, not that they are held
    /// off first.
    fn open() -> RetryMenu {
        let mut menu = RetryMenu::new(LANGUAGE);
        menu.keys.hold(0);
        menu
    }

    /// Walks the cursor down to a choice and presses decide on it.
    fn choose(menu: &mut RetryMenu, choice: Choice) -> Option<Choice> {
        let at = CHOICES
            .iter()
            .position(|item| *item == choice)
            .expect("the menu offers it");
        while menu.selection != at {
            assert_eq!(menu.step(&down()), None);
        }
        menu.step(&decide())
    }

    /// Lets the confirmation's grace run out, which `update` is what spends — so a test that
    /// drives `step` has to spend it itself.
    fn read_it(menu: &mut RetryMenu) {
        assert!(
            menu.keys.held() > 0,
            "a confirmation holds its keys off first"
        );
        menu.keys.hold(0);
    }

    /// The chapter is the one this menu exists for, and it acts on the press that chose it:
    /// it is answered every few seconds in a fight, and a question there would be answered
    /// unread.
    #[test]
    fn the_chapter_is_not_asked_about() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Chapter), Some(Choice::Chapter));
        for language in [Language::Japanese, Language::English] {
            assert!(question(Choice::Chapter, language).is_none());
        }
    }

    /// The other two ask first, and neither has happened when the question goes up: it takes
    /// the cursor moved onto yes and a second press.
    #[test]
    fn the_stage_and_giving_up_ask_first() {
        for choice in [Choice::Stage, Choice::Quit] {
            let mut menu = open();
            assert_eq!(choose(&mut menu, choice), None);
            assert!(matches!(menu.showing, Showing::Confirming(asked) if asked == choice));

            read_it(&mut menu);
            assert_eq!(menu.step(&down()), None);
            assert!(ANSWERS[menu.answer], "on yes");
            assert_eq!(menu.step(&decide()), Some(choice));
        }
    }

    /// The cursor starts on no, so the press that lands on the frame the grace ends — which is
    /// what a held key does — costs nothing but the question closing.
    #[test]
    fn a_confirmation_starts_on_no_and_no_goes_back() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Quit), None);
        assert_eq!(menu.answer, NO);
        assert!(!ANSWERS[menu.answer]);

        read_it(&mut menu);
        assert_eq!(menu.step(&decide()), None);
        assert!(matches!(menu.showing, Showing::Choices));
        // And the cursor is still on the item that was asked about, not moved by the answering.
        assert_eq!(CHOICES[menu.selection], Choice::Quit);
    }

    /// The way out of a question asked by mistake, which is the whole point of `x` here: the
    /// menu underneath has no cancel of its own.
    #[test]
    fn cancelling_a_confirmation_goes_back_to_the_choices() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Stage), None);
        read_it(&mut menu);
        assert_eq!(menu.step(&cancel()), None);
        assert!(matches!(menu.showing, Showing::Choices));
    }

    /// Nothing is answered on the frames a confirmation has just gone up, whatever is pressed.
    #[test]
    fn a_confirmation_holds_its_keys_off_before_it_answers() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Quit), None);
        assert_eq!(menu.keys.held(), CONFIRM_GRACE_FRAMES);
    }

    /// A direction is the other answer, both ways: with two of them there is nowhere else for
    /// one to go, and a cursor that only moved one way would leave `up` doing nothing.
    #[test]
    fn either_direction_moves_between_the_two_answers() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Stage), None);
        read_it(&mut menu);

        assert_eq!(menu.step(&down()), None);
        assert!(ANSWERS[menu.answer], "on yes");
        assert_eq!(
            menu.step(&Pressed {
                up: true,
                ..nothing()
            }),
            None
        );
        assert!(!ANSWERS[menu.answer], "back on no");
    }

    /// The play field, as the game's own output measures it.
    const FIELD: Rect = Rect {
        left: 32.0,
        top: 16.0,
        width: 384.0,
        height: 448.0,
    };

    /// One frame of the menu, on a screen of its own.
    fn frame(drawing: &Drawing, menu: &mut RetryMenu) -> Vec<Quad> {
        drawing.frame(|overlay| unsafe { menu.draw(overlay, FIELD, "MIDSTAGE 2", 3) })
    }

    /// Every line something was written on, whichever order the writing came in. Lines rather than
    /// quads, because there are more quads than lines: a label is drawn twice for its drop shadow,
    /// and the cursor is a label of its own beside the item it marks.
    fn lines(quads: &[Quad]) -> Vec<f32> {
        let mut lines: Vec<f32> = quads.iter().map(|quad| quad.y).collect();
        lines.sort_by(f32::total_cmp);
        lines.dedup();
        lines
    }

    /// The lines drawn in the lit colour: the item under the cursor, and the cursor beside it.
    fn lit(quads: &[Quad]) -> Vec<f32> {
        quads
            .iter()
            .filter(|quad| quad.color == SELECTED)
            .map(|quad| quad.y)
            .collect()
    }

    /// The field is washed before anything is written on it, over the whole of it and no further:
    /// the menu is a screen of its own inside the play area, and a wash that missed a corner would
    /// leave the run's own bullets showing through it.
    #[test]
    fn the_field_is_dimmed_under_the_menu() {
        let drawing = Drawing::new();
        let quads = frame(&drawing, &mut open());

        let wash = *quads.first().expect("something was drawn first");
        assert_eq!((wash.x, wash.y), (FIELD.left, FIELD.top));
        assert_eq!((wash.width, wash.height), (FIELD.width, FIELD.height));
        assert_eq!(wash.color, DIM_FIELD);
        // And it is under the writing rather than over it: everything else comes after.
        assert!(quads.len() > 1);
    }

    /// One line lit at a time. Two would be two things to read as chosen.
    #[test]
    fn one_choice_is_lit_and_the_others_are_not() {
        let drawing = Drawing::new();
        let quads = frame(&drawing, &mut open());

        let on = lit(&quads);
        assert_eq!(on.len(), 2, "the item and its cursor: {on:?}");
        assert_eq!(on[0], on[1], "on the same line");
    }

    /// The items stay where they are and the highlight moves down one line.
    ///
    /// The selection tests above would read the same if the drawing moved the items under a fixed
    /// cursor instead, and that would be wrong on the screen: what somebody reads is which line is
    /// lit.
    #[test]
    fn moving_the_cursor_moves_the_highlight_down_one_line() {
        let drawing = Drawing::new();
        let mut menu = open();
        let first = frame(&drawing, &mut menu);
        assert_eq!(menu.step(&down()), None);
        let second = frame(&drawing, &mut menu);

        assert_eq!(first.len(), second.len(), "the same things are drawn");
        assert_eq!(
            lines(&first),
            lines(&second),
            "nothing moved to another line"
        );
        // The topmost lit quad is the item's own; the cursor beside it shares the line.
        let top = |quads: &[Quad]| lit(quads).into_iter().min_by(f32::total_cmp).unwrap();
        assert_eq!(top(&second) - top(&first), LINE_HEIGHT);
    }

    /// A confirmation is a different screen in the same room: a question with its two answers under
    /// it rather than the three choices, and the field washed under that just the same.
    ///
    /// The answers sit a line lower than the choices did, the blank line between them and the
    /// question being what makes it read as a question rather than as a fourth item. Which is the
    /// thing to hold it to, since both screens draw the same number of labels and a count would say
    /// nothing.
    #[test]
    fn a_confirmation_puts_its_answers_below_where_the_choices_were() {
        let drawing = Drawing::new();
        let mut menu = open();
        let choices = frame(&drawing, &mut menu);

        assert_eq!(choose(&mut menu, Choice::Stage), None);
        assert!(matches!(menu.showing, Showing::Confirming(_)));
        let confirming = frame(&drawing, &mut menu);

        assert_eq!(confirming[0].color, DIM_FIELD, "still washed");

        let lowest = |quads: &[Quad]| {
            quads
                .iter()
                .map(|quad| quad.y)
                .max_by(f32::total_cmp)
                .expect("something was drawn")
        };
        assert_eq!(lowest(&confirming) - lowest(&choices), LINE_HEIGHT);

        let on = lit(&confirming);
        assert_eq!(on.len(), 2, "the answer and its cursor: {on:?}");
        assert_eq!(on[0], on[1]);
    }
}
