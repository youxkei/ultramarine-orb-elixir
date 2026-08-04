//! The question asked where a run has been chosen and a chapter of that same run was left: from
//! where it stopped, or from the beginning.
//!
//! Asked after the character select rather than beside the mode question, because which run this is
//! includes the character: 紺珠伝 keeps a pointdevice save per difficulty and character, and a run
//! of another character would take the buttons written down and play somebody else's shot with them.
//! By the frame this goes up the game has settled all three — difficulty, character and shot — and
//! has not built anything yet, so it is also the one frame on which the answer costs nothing either
//! way.
//!
//! How it reads its keys and draws its items is [`crate::menu_ui`], which the other two questions
//! share.
//!
//! **Nothing cancels it.** The front end has already taken itself down by the time the run is
//! asked for — its own update is what removes its job — so there is nothing behind this question to
//! go back to, and its two items are the two ways into the run that was chosen. Which is why the
//! cancel every one of these menus reads is the one thing here that goes unread.

use crate::game::Pad;
use crate::input::Keyboard;
use crate::menu_ui::{self, ASIDE, By, DIM_SCREEN, Keys, LINE_HEIGHT, NORMAL, Pressed, SELECTED};
use crate::overlay::{Label, Overlay, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Frames before the menu accepts anything. The key that chose the shot type is very likely still
/// down, and while a press is only acted on as it goes down, somebody who pressed it twice meant
/// both presses for the game's own select.
const INPUT_GRACE_FRAMES: u32 = 10;

/// What came of putting the question up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    /// The chapter the run was left in, played back into place.
    Continue,
    /// The run as the game would have started it.
    Beginning,
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
    pub fn new(left: String) -> Self {
        Self {
            selection: 0,
            keys: Keys::new(INPUT_GRACE_FRAMES),
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
        let by = pressed.decide;
        let answer = self.step(&pressed)?;
        // `step` answers only on a press that decided, so there is a hand to name.
        Some((answer, by?))
    }

    fn step(&mut self, pressed: &Pressed) -> Option<Answer> {
        self.selection = menu_ui::moved(self.selection, CHOICES.len(), pressed);
        pressed.decide.map(|_| CHOICES[self.selection].0)
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let said = match CHOICES[self.selection].0 {
            Answer::Continue => self.left.as_str(),
            Answer::Beginning => OVERWRITES,
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
        // The whole screen, the way the mode question covers it: what is underneath is the title the
        // front end left behind when it took itself down.
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
/// Drawn over the game's own screen rather than asked there: the screen carries on running, the
/// cursor moves between the two shots, and this follows it. What it saves somebody is choosing the
/// shot they always choose and finding out a screen later that the run they left was the other one —
/// `MainMenu::RegisterChain` memsets the cursor, so nothing on the game's screens remembers.
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
    pub fn pointing(&mut self, slot: Option<String>, look: impl FnOnce(&str) -> Option<String>) {
        if self.about == slot {
            return;
        }
        self.said = slot.as_deref().and_then(look);
        self.about = slot;
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
    use super::{Answer, Mark, Pressed, ResumeMenu};
    use crate::menu_ui::By;

    /// The mark is asked what a slot holds when the cursor arrives on it, and not again while it sits
    /// there — the screen is one somebody sits on, and the answer is a file read.
    #[test]
    fn a_slot_is_looked_up_when_the_cursor_reaches_it_and_not_every_frame() {
        let mut looked = Vec::new();
        let mut mark = Mark::new();
        for slot in ["normal-reimu-a", "normal-reimu-a", "normal-reimu-b"] {
            for _ in 0..3 {
                mark.pointing(Some(slot.to_owned()), |slot| {
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
        let mut menu = ResumeMenu::new("STAGE 4  BOSS SPELL 2  RETRY 42".to_owned());
        menu.keys.hold(0);
        menu
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

    /// Nothing cancels this one: the front end it was asked over has already taken itself down, so
    /// there is nothing to go back to — and the cancel the other two act on is read here and left
    /// alone.
    #[test]
    fn nothing_cancels_it() {
        let mut menu = open();
        let cancelled = Pressed {
            cancel: Some(By::Keyboard),
            ..nothing()
        };
        assert_eq!(menu.step(&cancelled), None);
        // And it changed nothing: the question is still up, on the item it started on.
        assert_eq!(menu.step(&decide()), Some(Answer::Continue));
    }

    /// Nothing is answered on the frames it has just gone up, whatever is pressed: the key that
    /// chose the shot type is still down.
    #[test]
    fn it_holds_its_keys_off_first() {
        assert!(ResumeMenu::new(String::new()).keys.held() > 0);
    }
}
