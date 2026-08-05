//! The question asked where a run about to be started has a chapter of its own left unfinished: from
//! where it stopped, or from the beginning.
//!
//! Asked on the shot type select, on the press that would have started the run — held back in the
//! input read, so the screen never sees it. Which run this is includes the character and the shot:
//! 紺珠伝 keeps a pointdevice save per difficulty and character, and a run of another shot would take
//! the buttons written down and play somebody else's with them. That screen knows all three, the
//! first two being settled and the third under its cursor.
//!
//! On the press rather than on the frame the run is chosen, which is where this used to be asked. The
//! shot type select does not fade or wait: its decide writes the run's shot and `curState` in one go,
//! and by the frame the run is chosen the front end has taken its own job out — so a question asked
//! there is one with nothing behind it, and its cancel had nothing to do. Held back a screen earlier
//! there is nothing to put back, the screen having never moved.
//!
//! How it reads its keys and draws its items is [`crate::menu_ui`], which the other two questions
//! share.

use crate::game::Pad;
use crate::input::Keyboard;
use crate::menu_ui::{self, ASIDE, By, DIM_SCREEN, Keys, LINE_HEIGHT, NORMAL, Pressed, SELECTED};
use crate::overlay::{Label, Overlay, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Frames before the menu accepts anything. The key this went up on is still down — it is the press
/// that was held back — and while a press is only acted on as it goes down, somebody who pressed it
/// twice meant both presses for the game's own select.
const INPUT_GRACE_FRAMES: u32 = 10;

/// What came of putting the question up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    /// The chapter the run was left in, played back into place.
    Continue,
    /// The run as the game would have started it.
    Beginning,
    /// Neither: the run is not started at all. The screen underneath is the one the question was
    /// asked over, on the shot it was asked about, and the press that would have left it was never
    /// handed to it — so this is the screen carrying on rather than a screen put back.
    Cancelled,
}

/// Whether cancelling is something this question can be answered with, which is a property of the
/// frame it went up on rather than of the question.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cancels {
    /// The run: it was asked on the press that would have started one, and that press is being held
    /// back. Cancelling is the press never being handed over.
    TheRun,
    /// Nothing. Asked on the frame a run was chosen, where the front end has already taken its own job
    /// out: there is no press left to withhold and no screen to carry on, so the question stays up
    /// until one of its two items is chosen.
    Nothing,
}

const CHOICES: [(Answer, &str); 2] = [
    (Answer::Continue, "つづきから"),
    (Answer::Beginning, "はじめから"),
];

/// What starting again costs, said under that item: the run left unfinished is written over as soon
/// as the new one reaches a chapter, and this is the last moment anybody is told so.
const OVERWRITES: &str = "中断データは上書きされます";

pub struct ResumeMenu {
    selection: usize,
    keys: Keys,
    cancels: Cancels,
    /// Which chapter the run was left in, for the line under the choices.
    left: String,
    title: Label,
    choices: [Label; CHOICES.len()],
    aside: Label,
    cursor: Label,
}

impl ResumeMenu {
    /// `left` describes the chapter that was left, which is what the item offering it says.
    ///
    /// The cursor starts on that item. Not because it is the likelier answer but because of what the
    /// two mistakes cost: a run picked up by accident is a run put back where it was, while a fresh
    /// run started by accident writes its own first chapter over the file and the one left
    /// unfinished is gone.
    pub fn new(left: String, cancels: Cancels) -> Self {
        Self {
            selection: 0,
            keys: Keys::new(INPUT_GRACE_FRAMES),
            cancels,
            left,
            title: Label::new(),
            choices: [Label::new(), Label::new()],
            aside: Label::new(),
            cursor: Label::new(),
        }
    }

    /// Returns what was answered, once something is. `pad` is what the pad is doing now, which the
    /// caller asks the game to read: the game is frozen here, so its own reading of the pad is not
    /// running and a pad would otherwise do nothing on this menu at all.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Answer, By)> {
        let pressed = self.keys.read(keyboard, pad)?;
        let answer = self.step(&pressed)?;
        // The hand that made the press this answer *is*, and not whichever of the two is there: a frame
        // carrying a decide on one and a cancel on the other is answered by the cancel — see `step` —
        // and naming the other hand for it is the one thing `By` exists to stop being guessed at.
        let by = match answer {
            Answer::Cancelled => pressed.cancel,
            Answer::Continue | Answer::Beginning => pressed.decide,
        };
        Some((answer, by?))
    }

    fn step(&mut self, pressed: &Pressed) -> Option<Answer> {
        self.selection = menu_ui::moved(self.selection, CHOICES.len(), pressed);
        if pressed.cancel.is_some() && self.cancels == Cancels::TheRun {
            return Some(Answer::Cancelled);
        }
        pressed.decide.map(|_| CHOICES[self.selection].0)
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let said = match CHOICES[self.selection].0 {
            Answer::Continue => self.left.as_str(),
            // `Cancelled` is not one of the two items and so never under the cursor: it is what a
            // cancel answers, and the items are what a decide does.
            Answer::Beginning | Answer::Cancelled => OVERWRITES,
        };
        unsafe {
            self.title.set(overlay, "どこから始める");
            self.aside.set(overlay, said);
            self.cursor.set(overlay, "▶");
            for (label, (_, text)) in self.choices.iter_mut().zip(CHOICES) {
                label.set(overlay, text);
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        // The whole screen, the way the mode question covers it: what is underneath is the shot type
        // select, still standing and still being drawn — its update is what has stopped.
        frame.fill(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, DIM_SCREEN);

        let center = SCREEN_WIDTH / 2.0;
        let y = SCREEN_HEIGHT / 2.0 - LINE_HEIGHT * 3.0;
        menu_ui::centred(&frame, &self.title, center, y, NORMAL);

        let y = menu_ui::list(
            &frame,
            &self.choices,
            &self.cursor,
            center,
            y + LINE_HEIGHT * 2.0,
            self.selection,
        );
        menu_ui::centred(&frame, &self.aside, center, y + LINE_HEIGHT, ASIDE);
    }
}

/// One line on the shot type select, saying that the run under the cursor has a chapter written down
/// and where it was left.
///
/// Said without stopping anything, unlike the question the same screen asks on the press: nothing is
/// frozen, the cursor moves between the two shots, and this follows it. What it saves somebody is
/// pressing at all on the shot they always choose and finding the question there —
/// `MainMenu::RegisterChain` memsets the cursor, so nothing on the game's own screens remembers which
/// run was left.
pub struct Mark {
    /// Which slot the line is about, so that the file is read when the cursor moves onto another run
    /// and not on every frame the screen is up.
    about: Option<String>,
    said: Option<String>,
    title: Label,
    aside: Label,
}

/// Bottom left, under everything the screen itself draws: the two shots and the description of the
/// one under the cursor are above the middle, and the corner is the one part of it nothing uses.
const MARK_LEFT: f32 = 24.0;
const MARK_BOTTOM: f32 = SCREEN_HEIGHT - 20.0;

impl Mark {
    pub const fn new() -> Self {
        Self {
            about: None,
            said: None,
            title: Label::new(),
            aside: Label::new(),
        }
    }

    /// The run the front end is pointing at, and what to say about it. `look` is only asked where
    /// that has changed, this being called every frame.
    pub fn pointing(&mut self, slot: Option<&str>, look: impl FnOnce(&str) -> Option<String>) {
        if self.about.as_deref() == slot {
            return;
        }
        self.said = slot.and_then(look);
        self.about = slot.map(str::to_owned);
    }

    /// # Safety
    /// Must run on the device's thread, inside the scene the game draws into.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let Self {
            said, title, aside, ..
        } = self;
        let Some(said) = said else {
            return;
        };
        unsafe {
            title.set(overlay, "中断データあり");
            aside.set(overlay, said);
        }
        let Some(frame) = (unsafe { overlay.frame() }) else {
            return;
        };
        frame.label(&self.title, MARK_LEFT, MARK_BOTTOM - LINE_HEIGHT, SELECTED);
        frame.label(&self.aside, MARK_LEFT, MARK_BOTTOM, ASIDE);
    }
}

#[cfg(test)]
mod tests {
    use super::{Answer, Cancels, MARK_LEFT, Mark, Pressed, ResumeMenu};
    use crate::d3d8::recording::Screen;
    use crate::menu_ui::{By, LINE_HEIGHT};
    use crate::overlay::{SCREEN_HEIGHT, SCREEN_WIDTH};

    /// The mark is asked what a slot holds when the cursor arrives on it, and not again while it sits
    /// there — the screen is one somebody sits on, and the answer is a file read.
    #[test]
    fn a_slot_is_looked_up_when_the_cursor_reaches_it_and_not_every_frame() {
        let mut looked = Vec::new();
        let mut mark = Mark::new();
        for slot in ["normal-reimu-a", "normal-reimu-a", "normal-reimu-b"] {
            for _ in 0..3 {
                mark.pointing(Some(slot), |slot| {
                    looked.push(slot.to_owned());
                    Some("STAGE 1".to_owned())
                });
            }
        }
        assert_eq!(looked, ["normal-reimu-a", "normal-reimu-b"]);
        // And nothing is said about a screen the cursor has left.
        mark.pointing(None, |_| panic!("nothing to look up"));
        assert!(mark.said.is_none());
    }

    fn open() -> ResumeMenu {
        opened(Cancels::TheRun)
    }

    fn opened(cancels: Cancels) -> ResumeMenu {
        let mut menu = ResumeMenu::new("STAGE 4  BOSS SPELL 2  RETRY 42".to_owned(), cancels);
        menu.keys.hold(0);
        menu
    }

    fn cancelled() -> Pressed {
        Pressed {
            cancel: Some(By::Keyboard),
            ..nothing()
        }
    }

    fn nothing() -> Pressed {
        Pressed {
            up: false,
            down: false,
            decide: None,
            cancel: None,
        }
    }

    /// The chapter that was left is what the cursor starts on: the mistake worth avoiding is the
    /// fresh run that writes over it.
    #[test]
    fn the_chapter_left_behind_is_the_one_offered() {
        let mut menu = open();
        assert_eq!(menu.step(&decide()), Some(Answer::Continue));
    }

    fn decide() -> Pressed {
        Pressed {
            decide: Some(By::Keyboard),
            ..nothing()
        }
    }

    /// A direction is the other answer, both ways: with two of them there is nowhere else for one to
    /// go, and a cursor that only moved one way would leave `up` doing nothing.
    #[test]
    fn either_direction_moves_between_the_two() {
        for direction in [
            Pressed {
                down: true,
                ..nothing()
            },
            Pressed {
                up: true,
                ..nothing()
            },
        ] {
            let mut menu = open();
            assert_eq!(menu.step(&direction), None);
            assert_eq!(menu.step(&decide()), Some(Answer::Beginning));
        }
    }

    /// Cancelling answers neither, which is the run not being started: the press that would have
    /// started it is the one this question was asked on, and it was held back.
    #[test]
    fn cancelling_starts_no_run_at_all() {
        let mut menu = open();
        assert_eq!(menu.step(&cancelled()), Some(Answer::Cancelled));
    }

    /// Whichever item the cursor is on: neither of them is what a cancel is a slower way of asking
    /// for, and starting a run that writes over the chapter is what it is asking not to do.
    #[test]
    fn cancelling_is_not_the_item_under_the_cursor() {
        let mut menu = open();
        assert_eq!(
            menu.step(&Pressed {
                down: true,
                ..nothing()
            }),
            None
        );
        assert_eq!(menu.step(&cancelled()), Some(Answer::Cancelled));
    }

    /// Where the run has already been chosen there is nothing a cancel could do — no press being held
    /// back, and no screen behind the question to carry on — so it is read and left alone, and the
    /// question stays up on the item it was on.
    #[test]
    fn a_question_asked_after_the_run_was_chosen_cancels_nothing() {
        let mut menu = opened(Cancels::Nothing);
        assert_eq!(menu.step(&cancelled()), None);
        assert_eq!(menu.step(&decide()), Some(Answer::Continue));
    }

    /// A line is only said about a run there is a chapter of: a run with nothing written down for it is
    /// a screen with nothing on it.
    #[test]
    fn nothing_is_said_about_a_run_with_no_chapter() {
        let mut mark = Mark::new();
        mark.pointing(Some("normal-reimu-a"), |_| Some("STAGE 1".to_owned()));
        assert!(mark.said.is_some());
        mark.pointing(Some("normal-reimu-b"), |_| None);
        assert!(mark.said.is_none());
    }

    /// Nothing is answered on the frames it has just gone up, whatever is pressed: the key that
    /// chose the shot type is still down.
    #[test]
    fn it_holds_its_keys_off_first() {
        assert!(ResumeMenu::new(String::new(), Cancels::TheRun).keys.held() > 0);
    }

    /// The mark on the shot type select is a line in the bottom-left corner, which is the one part of
    /// that screen nothing else uses: the two shots and the description of the one under the cursor
    /// are all above the middle. Drawn over the game's own screen rather than on a washed one, so a
    /// line that wandered would be a line over the game's writing.
    #[test]
    fn the_mark_sits_in_the_corner_the_screen_leaves_free() {
        let screen = Screen::new();
        let mut mark = Mark::new();
        mark.pointing(Some("normal-reimu-a"), |_| Some("STAGE 3".to_owned()));
        let quads = screen.frame(|overlay| unsafe { mark.draw(overlay) });

        assert!(!quads.is_empty(), "a run with a chapter says so");
        // Nothing is washed: this goes over the front end, which is still being read.
        assert!(
            !quads.iter().any(|quad| quad.width >= SCREEN_WIDTH),
            "the screen is not dimmed under it",
        );
        for quad in &quads {
            assert!(quad.x >= MARK_LEFT - 1.0, "{quad:?}");
            assert!(quad.y > SCREEN_HEIGHT / 2.0, "below the middle: {quad:?}");
            assert!(quad.bottom() <= SCREEN_HEIGHT, "on the screen: {quad:?}");
        }
    }

    /// And a run with nothing written down draws nothing at all, rather than an empty line where a
    /// line would be: the corner is the game's own screen when there is nothing to say about it.
    #[test]
    fn a_run_with_no_chapter_draws_nothing() {
        let screen = Screen::new();
        let mut mark = Mark::new();
        mark.pointing(Some("normal-reimu-b"), |_| None);
        assert!(
            screen
                .frame(|overlay| unsafe { mark.draw(overlay) })
                .is_empty()
        );
    }

    /// The two lines are the title above and what it says below, one line apart — read as one thing
    /// in the corner rather than as two.
    #[test]
    fn the_marks_two_lines_are_a_line_apart() {
        let screen = Screen::new();
        let mut mark = Mark::new();
        mark.pointing(Some("normal-reimu-a"), |_| Some("STAGE 3".to_owned()));
        let quads = screen.frame(|overlay| unsafe { mark.draw(overlay) });

        let mut lines: Vec<f32> = quads.iter().map(|quad| quad.y).collect();
        lines.sort_by(f32::total_cmp);
        lines.dedup();
        // Two lines, each drawn twice for its drop shadow, and the shadow a pixel down — so four
        // distinct ys a pixel and a line apart.
        assert_eq!(lines.len(), 4, "{lines:?}");
        assert_eq!(lines[2] - lines[0], LINE_HEIGHT);
    }
}
