//! The question orb puts over the game's own menu: whether a run keeps chapters, and which of
//! the two rankings is being looked at.
//!
//! Asked where the game is asked, because that is where the answer belongs: a run is started
//! from the title menu, and one with chapters and one without are two different things to start.
//! Not a setting in `orb.yaml`, which is for what somebody sets once — this is a choice made per
//! run, the way 紺珠伝 asks it.
//!
//! The game is frozen while this is up, which means its own input handling is not running
//! either — so this menu reads the keyboard itself, and takes the pad from the sample orb's own
//! thread keeps. Without that second half a pad does nothing at all here while working perfectly
//! on the game's own menu one keypress earlier, which is the whole of how it looks broken.

use std::fmt;

use crate::game::{Menu, Pad};
use crate::input::Keyboard;
use crate::overlay::{Label, Overlay, SCREEN_HEIGHT, SCREEN_WIDTH};

const VK_RETURN: u8 = 0x0d;
const VK_ESCAPE: u8 = 0x1b;
const VK_UP: u8 = 0x26;
const VK_DOWN: u8 = 0x28;
const VK_X: u8 = 0x58;
const VK_Z: u8 = 0x5a;

const DIM: u32 = 0xc800_0000;
const NORMAL: u32 = 0xffff_ffff;
const SELECTED: u32 = 0xffff_e066;
/// The line under the choices, which says what the one under the cursor means.
const ASIDE: u32 = 0xffb0_b0b0;

const LINE_HEIGHT: f32 = 24.0;

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

/// What answered, which goes in the log.
///
/// Because a menu of orb's reads the pad itself — see the module comment — whether a pad works on
/// one is a question about orb rather than about the pad, and a log that does not say which hand
/// answered cannot settle it. It could not, and that cost a session.
#[derive(Clone, Copy, Debug)]
pub enum By {
    Keyboard,
    Pad,
}

impl fmt::Display for By {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Keyboard => "keyboard",
            Self::Pad => "pad",
        })
    }
}

pub struct ModeMenu {
    asked: Menu,
    selection: usize,
    grace: u32,
    /// What the pad was doing last frame, since what this acts on is a press and not the holding —
    /// and holding a button is how somebody arrives here from the game's own menu.
    pad: Pad,
    title: Label,
    choices: [Label; CHOICES.len()],
    aside: Label,
    hint: Label,
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
            grace: INPUT_GRACE_FRAMES,
            pad: Pad::default(),
            title: Label::new(),
            choices: [Label::new(), Label::new()],
            aside: Label::new(),
            hint: Label::new(),
            cursor: Label::new(),
        }
    }

    /// Returns what was answered, once something is. `pad` is what the pad is doing now, which the
    /// caller asks the game to read: the game is frozen here, so its own reading of the pad is not
    /// running and a pad would otherwise do nothing on this menu at all.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Answer, By)> {
        // Every frame, grace or not, so that a button held from before the menu opened is not a
        // press the moment the grace ends.
        let was = std::mem::replace(&mut self.pad, pad);
        let pushed = |now: bool, before: bool| now && !before;
        if self.grace > 0 {
            self.grace -= 1;
            return None;
        }
        if keyboard.pressed(VK_UP) || pushed(pad.up, was.up) {
            self.selection = self.selection.checked_sub(1).unwrap_or(CHOICES.len() - 1);
        }
        if keyboard.pressed(VK_DOWN) || pushed(pad.down, was.down) {
            self.selection = (self.selection + 1) % CHOICES.len();
        }
        // The game's own cancel — `x` is its bomb key and its menus read that as back — escape,
        // which is what anything with a window on it is expected to close on, and whichever pad
        // button the game maps to that.
        if keyboard.pressed(VK_X) || keyboard.pressed(VK_ESCAPE) {
            return Some((Answer::Cancelled, By::Keyboard));
        }
        if pushed(pad.cancel, was.cancel) {
            return Some((Answer::Cancelled, By::Pad));
        }
        let chosen = Answer::Chosen(CHOICES[self.selection].0);
        if keyboard.pressed(VK_Z) || keyboard.pressed(VK_RETURN) {
            return Some((chosen, By::Keyboard));
        }
        pushed(pad.decide, was.decide).then_some((chosen, By::Pad))
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let selected = CHOICES[self.selection].0;
        unsafe {
            self.title.set(overlay, title(self.asked));
            self.aside.set(overlay, aside(self.asked, selected));
            self.hint.set(overlay, HINT);
            self.cursor.set(overlay, "▶");
            for (label, (_, text)) in self.choices.iter_mut().zip(CHOICES) {
                label.set(overlay, text);
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        // The whole screen, not the play field: this is over the game's own menu, and what is
        // underneath is not something to leave half readable.
        frame.fill(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, DIM);

        let center = SCREEN_WIDTH / 2.0;
        let mut y = SCREEN_HEIGHT / 2.0 - LINE_HEIGHT * 3.0;
        frame.label(&self.title, center - self.title.width() / 2.0, y, NORMAL);

        y += LINE_HEIGHT * 2.0;
        for (index, label) in self.choices.iter().enumerate() {
            let chosen = index == self.selection;
            let x = center - label.width() / 2.0;
            frame.label(label, x, y, if chosen { SELECTED } else { NORMAL });
            if chosen {
                frame.label(&self.cursor, x - self.cursor.width() - 6.0, y, SELECTED);
            }
            y += LINE_HEIGHT;
        }

        y += LINE_HEIGHT;
        frame.label(&self.aside, center - self.aside.width() / 2.0, y, ASIDE);

        // Said, because cancelling is the one thing here nothing else on the screen suggests: the
        // game has already taken the item and started its fade, so that going back is possible at
        // all is not obvious.
        y += LINE_HEIGHT * 2.0;
        frame.label(&self.hint, center - self.hint.width() / 2.0, y, ASIDE);
    }
}

/// The keys, in the game's own terms: `z` is its shoot key and `x` its bomb key, which are what
/// its menus take as decide and cancel.
const HINT: &str = "Z 決定    X 戻る";

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
