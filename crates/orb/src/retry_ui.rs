//! The menu that appears where the chapter was lost.
//!
//! The game is frozen while this is up, which means its own input handling is
//! not running either — so the menu reads the keyboard itself, and takes the pad
//! from the sample orb's own thread keeps.

use crate::game::{Pad, Rect};
use crate::input::Keyboard;
use crate::log::log;
use crate::mode_ui::By;
use crate::overlay::{Label, Overlay};

const VK_RETURN: u8 = 0x0d;
const VK_ESCAPE: u8 = 0x1b;
const VK_UP: u8 = 0x26;
const VK_DOWN: u8 = 0x28;
const VK_X: u8 = 0x58;
const VK_Z: u8 = 0x5a;

const DIM: u32 = 0xb400_0000;
const NORMAL: u32 = 0xffff_ffff;
const SELECTED: u32 = 0xffff_e066;

const LINE_HEIGHT: f32 = 24.0;

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
    /// For the log, in the English the rest of it is in — the menu itself is in Japanese.
    pub fn label(self) -> &'static str {
        match self {
            Self::Chapter => "the chapter again",
            Self::Stage => "the stage again",
            Self::Quit => "the run given up",
        }
    }
}

/// The third says where it ends up rather than what it gives up, because that is the part
/// somebody reading the item does not know: the run ending is the obvious half, and that the game
/// itself carries on is not.
const CHOICES: [(Choice, &str); 3] = [
    (Choice::Chapter, "チャプターをやり直す"),
    (Choice::Stage, "ステージをやり直す"),
    (Choice::Quit, "タイトルに戻る"),
];

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
fn question(choice: Choice) -> Option<&'static str> {
    match choice {
        Choice::Chapter => None,
        Choice::Stage => Some("ステージの最初からやり直す？"),
        Choice::Quit => Some("やめてタイトルに戻る？"),
    }
}

/// Yes above no, with the cursor starting on no — which is where the game's own quit
/// question puts it, and it is what makes a press on the frame the grace ends cost nothing.
const ANSWERS: [(bool, &str); 2] = [(true, "はい"), (false, "いいえ")];
const NO: usize = 1;

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
    grace: u32,
    /// What the pad was doing last frame. The game is frozen here, so it is not reading the pad
    /// itself and this menu would take nothing from one at all — and dying with a pad in hand is
    /// exactly when this menu comes up.
    pad: Pad,
    chapter: Label,
    retry: Label,
    choices: [Label; CHOICES.len()],
    asked: Label,
    answers: [Label; ANSWERS.len()],
    cursor: Label,
}

/// A frame's presses, whichever hand made them.
struct Pressed {
    up: bool,
    down: bool,
    decide: bool,
    cancel: bool,
}

impl RetryMenu {
    pub fn new() -> Self {
        Self {
            showing: Showing::Choices,
            selection: 0,
            answer: NO,
            grace: INPUT_GRACE_FRAMES,
            pad: Pad::default(),
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
        // Every frame, grace or not: the player was holding the shot key when they died, and that
        // must not become a press the moment the grace ends.
        let was = std::mem::replace(&mut self.pad, pad);
        let pushed = |now: bool, before: bool| now && !before;
        let decided = keyboard.pressed(VK_Z) || keyboard.pressed(VK_RETURN);
        let pressed = Pressed {
            up: keyboard.pressed(VK_UP) || pushed(pad.up, was.up),
            down: keyboard.pressed(VK_DOWN) || pushed(pad.down, was.down),
            decide: decided || pushed(pad.decide, was.decide),
            // The game's own cancel — `x` is its bomb key and its menus read that as back —
            // escape, which is what anything with a window on it is expected to close on, and
            // whichever pad button the game maps to that.
            cancel: keyboard.pressed(VK_X)
                || keyboard.pressed(VK_ESCAPE)
                || pushed(pad.cancel, was.cancel),
        };
        if self.grace > 0 {
            self.grace -= 1;
            return None;
        }
        // Which hand answered, which is only ever asked about the press that decided.
        let by = if decided { By::Keyboard } else { By::Pad };
        self.step(pressed).map(|choice| (choice, by))
    }

    fn step(&mut self, pressed: Pressed) -> Option<Choice> {
        match self.showing {
            Showing::Choices => {
                if pressed.up {
                    self.selection = self.selection.checked_sub(1).unwrap_or(CHOICES.len() - 1);
                }
                if pressed.down {
                    self.selection = (self.selection + 1) % CHOICES.len();
                }
                if !pressed.decide {
                    return None;
                }
                let chosen = CHOICES[self.selection].0;
                if question(chosen).is_none() {
                    return Some(chosen);
                }
                self.showing = Showing::Confirming(chosen);
                self.answer = NO;
                self.grace = CONFIRM_GRACE_FRAMES;
                log!("retry: asking about {}", chosen.label());
                None
            }
            Showing::Confirming(choice) => {
                // Either direction, there being two answers: what a direction means here is the
                // other one.
                if pressed.up || pressed.down {
                    self.answer = (self.answer + 1) % ANSWERS.len();
                }
                if pressed.cancel {
                    self.back_to_choices(choice, "cancelled");
                    return None;
                }
                if !pressed.decide {
                    return None;
                }
                if ANSWERS[self.answer].0 {
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
                    for (label, (_, text)) in self.choices.iter_mut().zip(CHOICES) {
                        label.set(overlay, text);
                    }
                }
                Showing::Confirming(choice) => {
                    if let Some(asked) = question(choice) {
                        self.asked.set(overlay, asked);
                    }
                    for (label, (_, text)) in self.answers.iter_mut().zip(ANSWERS) {
                        label.set(overlay, text);
                    }
                }
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        frame.fill(area.left, area.top, area.width, area.height, DIM);

        let center = area.center_x();
        // The header and the gap under it are three lines, which puts what follows one line above
        // the middle of the field: the three items straddle it, and a confirmation — a question
        // and its two answers, with a line between — sits inside the same room.
        let mut y = area.center_y() - LINE_HEIGHT * 4.0;
        for label in [&self.chapter, &self.retry] {
            frame.label(label, center - label.width() / 2.0, y, NORMAL);
            y += LINE_HEIGHT;
        }
        y += LINE_HEIGHT;

        match self.showing {
            Showing::Choices => {
                for (index, label) in self.choices.iter().enumerate() {
                    let selected = index == self.selection;
                    let x = center - label.width() / 2.0;
                    frame.label(label, x, y, if selected { SELECTED } else { NORMAL });
                    if selected {
                        frame.label(&self.cursor, x - self.cursor.width() - 6.0, y, SELECTED);
                    }
                    y += LINE_HEIGHT;
                }
            }
            Showing::Confirming(_) => {
                frame.label(&self.asked, center - self.asked.width() / 2.0, y, NORMAL);
                y += LINE_HEIGHT * 2.0;
                for (index, label) in self.answers.iter().enumerate() {
                    let selected = index == self.answer;
                    let x = center - label.width() / 2.0;
                    frame.label(label, x, y, if selected { SELECTED } else { NORMAL });
                    if selected {
                        frame.label(&self.cursor, x - self.cursor.width() - 6.0, y, SELECTED);
                    }
                    y += LINE_HEIGHT;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ANSWERS, CHOICES, CONFIRM_GRACE_FRAMES, Choice, NO, Pressed, RetryMenu, Showing, question,
    };

    /// A frame nothing was pressed on.
    fn nothing() -> Pressed {
        Pressed {
            up: false,
            down: false,
            decide: false,
            cancel: false,
        }
    }

    fn decide() -> Pressed {
        Pressed {
            decide: true,
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
            cancel: true,
            ..nothing()
        }
    }

    /// A menu past the grace the run's own keys are kept out by, which is where every test
    /// here starts: what is being watched is what the presses mean, not that they are held
    /// off first.
    fn open() -> RetryMenu {
        let mut menu = RetryMenu::new();
        menu.grace = 0;
        menu
    }

    /// Walks the cursor down to a choice and presses decide on it.
    fn choose(menu: &mut RetryMenu, choice: Choice) -> Option<Choice> {
        let at = CHOICES
            .iter()
            .position(|(item, _)| *item == choice)
            .expect("the menu offers it");
        while menu.selection != at {
            assert_eq!(menu.step(down()), None);
        }
        menu.step(decide())
    }

    /// Lets the confirmation's grace run out, which `update` is what spends — so a test that
    /// drives `step` has to spend it itself.
    fn read_it(menu: &mut RetryMenu) {
        assert!(menu.grace > 0, "a confirmation holds its keys off first");
        menu.grace = 0;
    }

    /// The chapter is the one this menu exists for, and it acts on the press that chose it:
    /// it is answered every few seconds in a fight, and a question there would be answered
    /// unread.
    #[test]
    fn the_chapter_is_not_asked_about() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Chapter), Some(Choice::Chapter));
        assert!(question(Choice::Chapter).is_none());
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
            assert_eq!(menu.step(down()), None);
            assert!(ANSWERS[menu.answer].0, "on yes");
            assert_eq!(menu.step(decide()), Some(choice));
        }
    }

    /// The cursor starts on no, so the press that lands on the frame the grace ends — which is
    /// what a held key does — costs nothing but the question closing.
    #[test]
    fn a_confirmation_starts_on_no_and_no_goes_back() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Quit), None);
        assert_eq!(menu.answer, NO);
        assert!(!ANSWERS[menu.answer].0);

        read_it(&mut menu);
        assert_eq!(menu.step(decide()), None);
        assert!(matches!(menu.showing, Showing::Choices));
        // And the cursor is still on the item that was asked about, not moved by the answering.
        assert_eq!(CHOICES[menu.selection].0, Choice::Quit);
    }

    /// The way out of a question asked by mistake, which is the whole point of `x` here: the
    /// menu underneath has no cancel of its own.
    #[test]
    fn cancelling_a_confirmation_goes_back_to_the_choices() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Stage), None);
        read_it(&mut menu);
        assert_eq!(menu.step(cancel()), None);
        assert!(matches!(menu.showing, Showing::Choices));
    }

    /// Nothing is answered on the frames a confirmation has just gone up, whatever is pressed.
    #[test]
    fn a_confirmation_holds_its_keys_off_before_it_answers() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Quit), None);
        assert_eq!(menu.grace, CONFIRM_GRACE_FRAMES);
    }

    /// A direction is the other answer, both ways: with two of them there is nowhere else for
    /// one to go, and a cursor that only moved one way would leave `up` doing nothing.
    #[test]
    fn either_direction_moves_between_the_two_answers() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Choice::Stage), None);
        read_it(&mut menu);

        assert_eq!(menu.step(down()), None);
        assert!(ANSWERS[menu.answer].0, "on yes");
        assert_eq!(
            menu.step(Pressed {
                up: true,
                ..nothing()
            }),
            None
        );
        assert!(!ANSWERS[menu.answer].0, "back on no");
    }
}
