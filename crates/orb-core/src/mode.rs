//! Which of the two things a run is, and the question orb asks to find out.
//!
//! Asked where the game is asked, because that is where the answer belongs: a run is started from
//! the title menu, and one with chapters and one without are two different things to start. Not a
//! setting in `orb.yaml`, which is for what somebody sets once — this is a choice made per run, the
//! way 紺珠伝 asks it.
//!
//! On the press that would have chosen the item, held back in the input read, so the menu has not
//! moved: answering picks the mode and hands that press over, and cancelling is the press never
//! being handed over at all. Asked after the menu had acted, cancelling meant putting the front end
//! back the way its own back button does — which reloads the title and plays its animation through
//! for a question somebody has just said no to.
//!
//! The drawing is `mode_ui` in the `orb` crate. Here is what it decides, which is a function of a
//! keyboard and a pad — see [`crate::menu`].

use std::fmt;

use crate::game::{Menu, Pad};
use crate::input::Keyboard;
use crate::menu::{self, By, Keys};

/// Frames before the menu accepts anything. The key this went up on is still down — it is the press
/// that was held back — and while a press is only acted on as it goes down, somebody who pressed it
/// twice meant both presses for the game's menu and not for this.
pub const INPUT_GRACE_FRAMES: u32 = 10;

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
pub const CHOICES: [(Mode, &str); 2] = [
    (Mode::Pointdevice, "完全無欠モード"),
    (Mode::Normal, "レガシーモード"),
];

/// What came of putting the question up.
pub enum Answer {
    /// One of the two was chosen.
    Chosen(Mode),
    /// Neither: the item is not chosen at all. The menu underneath never acted on the press, so it is
    /// still on that item and nothing has to be put back.
    Cancelled,
}

/// The question itself: which of the two is under the cursor, and the keys it reads to move and
/// answer.
pub struct Question {
    asked: Menu,
    selection: usize,
    keys: Keys,
}

impl Question {
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
        }
    }

    /// Returns what was answered, once something is. `pad` is what the pad is doing now, which the
    /// caller asks the game to read: the game is frozen here, so its own reading of the pad is not
    /// running and a pad would otherwise do nothing on this menu at all.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Answer, By)> {
        let pressed = self.keys.read(keyboard, pad)?;
        self.selection = menu::moved(self.selection, CHOICES.len(), &pressed);
        if let Some(by) = pressed.cancel {
            return Some((Answer::Cancelled, by));
        }
        let by = pressed.decide?;
        Some((Answer::Chosen(CHOICES[self.selection].0), by))
    }

    /// Which is being asked about, and which of the two the cursor is on. Both for the drawing,
    /// which is the other half of this and lives over the seam.
    pub fn asked(&self) -> Menu {
        self.asked
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn selected(&self) -> Mode {
        CHOICES[self.selection].0
    }
}

/// What is being asked about. Two questions rather than one, because what a mode decides about a
/// run and what it decides about a ranking are not the same thing to somebody choosing.
pub fn title(asked: Menu) -> &'static str {
    match asked {
        Menu::Scores => "どちらのスコアを見る",
        Menu::Run => "モードを選ぶ",
    }
}

/// What the choice under the cursor means, a line at a time.
///
/// A line rather than a sentence wrapped by the drawing: a label is one `TextOutW` and so one line,
/// and where each break falls is a decision about what belongs together rather than about how wide
/// the screen is.
///
/// **How much room a line has, for whoever adds a longer one.** The longest here is
/// `進行状況は自動的にセーブされ、いつでも続きから遊べます`, 26 characters at an em of 15 against a
/// 640-wide output — some 400 pixels, and inside the screen by that arithmetic rather than by anyone
/// having held a ruler to it. Nothing clips a line that outgrows the width; it is drawn off the edge
/// and the part past it cannot be read at all.
pub fn aside(asked: Menu, mode: Mode) -> &'static [&'static str] {
    match (asked, mode) {
        // Nothing under a ranking's two choices: the two names are the whole of that choice, and a
        // line naming the file each is kept in answers a question about the disk that nobody
        // standing in front of a ranking is asking.
        (Menu::Scores, _) => &[],
        // What a pointdevice run gives and what it costs, both: the progress kept is the reason to
        // choose it, and the replay is the one thing the game would have offered afterwards that it
        // no longer does — see `Game::skip_replay_prompt`.
        (Menu::Run, Mode::Pointdevice) => &[
            "被弾したらチャプターの頭からやり直します",
            "進行状況は自動的にセーブされ、いつでも続きから遊べます",
            "リプレイは保存できません",
        ],
        (Menu::Run, Mode::Normal) => &["いつものゲームモードです", "被弾したら残機が減ります"],
    }
}

#[cfg(test)]
mod tests {
    use super::{CHOICES, Mode, aside, title};
    use crate::game::Menu;

    /// A run and a ranking are asked about differently, and the run's two choices each say what
    /// they mean.
    #[test]
    fn each_question_says_what_it_is_about() {
        assert_ne!(title(Menu::Run), title(Menu::Scores));
        assert_ne!(
            aside(Menu::Run, Mode::Pointdevice),
            aside(Menu::Run, Mode::Normal)
        );
    }

    /// A ranking is asked with nothing under either choice: what a mode does to a run is not what
    /// somebody looking at a ranking of one came to read.
    #[test]
    fn a_ranking_is_asked_with_no_description_at_all() {
        for mode in [Mode::Pointdevice, Mode::Normal] {
            assert!(aside(Menu::Scores, mode).is_empty());
        }
    }

    /// The two things a pointdevice run does that the game does not: it keeps where the run got to,
    /// and it has no replay to offer at the end of one. Said because the question is the last place
    /// the choice between the modes is still open.
    #[test]
    fn a_pointdevice_run_says_its_progress_is_kept_and_its_replay_is_not() {
        let said = aside(Menu::Run, Mode::Pointdevice).join("\n");
        assert!(said.contains("セーブ"));
        assert!(said.contains("リプレイは保存できません"));
    }

    /// 完全無欠モード is the first of the two, which is what a cursor starting on neither would land
    /// on — and it is the mode orb exists for.
    #[test]
    fn the_mode_with_chapters_is_the_first_choice_and_is_named_for_紺珠伝() {
        assert_eq!(CHOICES[0], (Mode::Pointdevice, "完全無欠モード"));
        assert_eq!(CHOICES[1].0, Mode::Normal);
    }
}
