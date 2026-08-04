//! The question orb puts over the game's own menu: whether a run keeps chapters, and which of
//! the two rankings is being looked at.
//!
//! Asked where the game is asked, because that is where the answer belongs: a run is started
//! from the title menu, and one with chapters and one without are two different things to start.
//! Not a setting in `orb.yaml`, which is for what somebody sets once — this is a choice made per
//! run, the way 紺珠伝 asks it.
//!
//! How it reads its keys and draws its items is [`crate::menu_ui`], which the other two questions
//! share.

use std::fmt;

use crate::game::{Menu, Pad};
use crate::input::Keyboard;
use crate::menu_ui::{self, ASIDE, By, DIM_SCREEN, Keys, LINE_HEIGHT, NORMAL};
use crate::overlay::{Label, Overlay, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Frames before the menu accepts anything. The key that chose the item this is asked over is
/// very likely still down, and while a press is only acted on as it goes down, somebody who
/// pressed it twice meant both presses for the game's menu and not for this.
const INPUT_GRACE_FRAMES: u32 = 10;

/// Which of the two things a run is, and which of the two files a ranking is.
///
/// One choice for both, rather than a mode per screen: the ranking of pointdevice runs and a
/// pointdevice run are the same file, and orb being in one mode is what makes the unlocks the
/// game reads out of that file the ones the run will be recorded in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Chapters, snapshots and the retry menu: dying sends the player back to the start of the
    /// chapter they were in.
    Pointdevice,
    /// The game as it was. Dying costs a life, replays can be saved, and the score goes in the
    /// game's own file.
    Normal,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pointdevice => "pointdevice",
            Self::Normal => "normal",
        })
    }
}

/// What the two are called on screen: 紺珠伝's own names for them, since that is where the mode
/// comes from and those are the names somebody who wants it knows. In the code, the log and the
/// file it writes they stay pointdevice and normal, which is the English of the first and what the
/// second actually is.
const CHOICES: [(Mode, &str); 2] = [
    (Mode::Pointdevice, "完全無欠モード"),
    (Mode::Normal, "レガシーモード"),
];

/// What came of putting the question up.
pub enum Answer {
    /// One of the two was chosen.
    Chosen(Mode),
    /// Neither: what was asked for is the menu this was asked over, which the game is put back on
    /// its way to.
    Cancelled,
}

pub struct ModeMenu {
    asked: Menu,
    selection: usize,
    keys: Keys,
    title: Label,
    choices: [Label; CHOICES.len()],
    aside: Label,
    cursor: Label,
}

impl ModeMenu {
    /// `current` is what orb is in now, which is where the cursor starts: the answer most likely
    /// to be wanted is the one that was wanted last time.
    pub fn new(asked: Menu, current: Mode) -> Self {
        Self {
            asked,
            selection: CHOICES
                .iter()
                .position(|(mode, _)| *mode == current)
                .unwrap_or(0),
            keys: Keys::new(INPUT_GRACE_FRAMES),
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
        self.selection = menu_ui::moved(self.selection, CHOICES.len(), &pressed);
        if let Some(by) = pressed.cancel {
            return Some((Answer::Cancelled, by));
        }
        let by = pressed.decide?;
        Some((Answer::Chosen(CHOICES[self.selection].0), by))
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let selected = CHOICES[self.selection].0;
        unsafe {
            self.title.set(overlay, title(self.asked));
            self.aside.set(overlay, aside(self.asked, selected));
            self.cursor.set(overlay, "▶");
            for (label, (_, text)) in self.choices.iter_mut().zip(CHOICES) {
                label.set(overlay, text);
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        // The whole screen, not the play field: this is over the game's own menu.
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

/// What is being asked about. Two questions rather than one, because what a mode decides about a
/// run and what it decides about a ranking are not the same thing to somebody choosing.
fn title(asked: Menu) -> &'static str {
    match asked {
        Menu::Scores => "どちらのスコアを見る",
        _ => "モードを選ぶ",
    }
}

/// What the choice under the cursor means. For a ranking that is the file it is kept in, which
/// is the whole of what there is to say and is also what a directory listing then shows.
fn aside(asked: Menu, mode: Mode) -> &'static str {
    match (asked, mode) {
        (Menu::Scores, Mode::Pointdevice) => "pointdevice_score.dat",
        (Menu::Scores, Mode::Normal) => "score.dat  ゲームのもの",
        (_, Mode::Pointdevice) => "死んだらチャプターの頭からやり直す",
        (_, Mode::Normal) => "ゲームそのまま  死んだら残機が減る",
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, aside, title};
    use crate::game::Menu;

    /// A run and a ranking are asked about differently, and each choice says what it means.
    #[test]
    fn each_question_says_what_it_is_about() {
        assert_ne!(title(Menu::Run), title(Menu::Scores));
        for asked in [Menu::Run, Menu::Scores] {
            assert_ne!(aside(asked, Mode::Pointdevice), aside(asked, Mode::Normal));
        }
    }

    /// The ranking's two choices are the two files, named as they are on disk: that is what
    /// somebody looking for one afterwards has to go on.
    #[test]
    fn a_ranking_names_the_file_it_is_kept_in() {
        assert!(aside(Menu::Scores, Mode::Pointdevice).contains("pointdevice_score.dat"));
        assert!(aside(Menu::Scores, Mode::Normal).contains("score.dat"));
    }
}
