//! Which of the two things a run is, and the question orb asks to find out.
//!
//! Asked where the game is asked, because that is where the answer belongs: a run is started from
//! the title menu, and one with chapters and one without are two different things to start. Not a
//! setting in `orb.yaml`, which is for what somebody sets once — this is a choice made per run, the
//! way th15 asks it.
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

use orb_config::Language;

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

/// The two, in the order they are offered.
///
/// The modes and not their names, which are [`Mode::name`]'s: how many there are and which is first
/// is the same in either language, and the cursor is an index into this.
pub const CHOICES: [Mode; 2] = [Mode::Pointdevice, Mode::Normal];

impl Mode {
    /// What the mode is called on screen: th15's own names for the two, in whichever of the two
    /// languages that game itself calls them — since that is where the mode comes from and those are
    /// the names somebody who wants it knows.
    ///
    /// In the code, the log and the file it writes they stay pointdevice and normal, which is
    /// [`fmt::Display`] above and is what the second of them actually is.
    pub fn name(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Pointdevice, Language::Japanese) => "完全無欠モード",
            (Self::Pointdevice, Language::English) => "Pointdevice Mode",
            (Self::Normal, Language::Japanese) => "レガシーモード",
            (Self::Normal, Language::English) => "Legacy Mode",
        }
    }
}

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
                .position(|mode| *mode == current)
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
        Some((Answer::Chosen(CHOICES[self.selection]), by))
    }

    /// One of the two taken outright, for a launch playing itself: this cursor is orb's own and the word a
    /// recipe presses moves the game's, so a press here answers whichever mode the cursor came up on — see
    /// `runtime::answer_the_question`. `item` is its place in [`CHOICES`].
    pub fn answered_outright(&self, item: usize) -> Option<Answer> {
        Some(Answer::Chosen(*CHOICES.get(item)?))
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
        CHOICES[self.selection]
    }
}

/// What is being asked about. Two questions rather than one, because what a mode decides about a
/// run and what it decides about a ranking are not the same thing to somebody choosing.
pub fn title(asked: Menu, language: Language) -> &'static str {
    match (asked, language) {
        (Menu::Scores, Language::Japanese) => "どちらのスコアを見る",
        (Menu::Scores, Language::English) => "Which scores to see",
        (Menu::Replays, Language::Japanese) => "どちらのリプレイを見る",
        (Menu::Replays, Language::English) => "Which replays to watch",
        (Menu::Run, Language::Japanese) => "モードを選ぶ",
        (Menu::Run, Language::English) => "Choose a mode",
    }
}

/// What the choice under the cursor means, a line at a time.
///
/// A line rather than a sentence wrapped by the drawing: a label is one `TextOutW` and so one line,
/// and where each break falls is a decision about what belongs together rather than about how wide
/// the screen is.
///
/// **How much room a line has, for whoever adds a longer one.** The longest Japanese line here is
/// `進行状況は自動的にセーブされ、いつでも続きから遊べます`, 26 characters at an em of 15 against a
/// 640-wide output — some 400 pixels, and inside the screen by that arithmetic rather than by anyone
/// having held a ruler to it. The longest English one is that line's own translation at 54 characters,
/// which at about half an em apiece comes to the same 400. Nothing clips a line that outgrows the
/// width; it is drawn off the edge and the part past it cannot be read at all.
pub fn aside(asked: Menu, mode: Mode, language: Language) -> &'static [&'static str] {
    match (asked, mode, language) {
        // Nothing under a ranking's two choices: the two names are the whole of that choice, and a
        // line naming the file each is kept in answers a question about the disk that nobody
        // standing in front of a ranking is asking.
        (Menu::Scores, _, _) => &[],
        // The replays have a line each, unlike the ranking, because the two answers do different things
        // rather than the same thing to different files: one is the game's own screen and the other is a
        // list of orb's, and what can be chosen out of the second is not what the first offers.
        (Menu::Replays, Mode::Pointdevice, Language::Japanese) => {
            &["ステージ・チャプター・トライを選んで見られます"]
        }
        (Menu::Replays, Mode::Pointdevice, Language::English) => {
            &["Watchable by stage, chapter and attempt"]
        }
        (Menu::Replays, Mode::Normal, Language::Japanese) => &["ゲーム本来のリプレイです"],
        (Menu::Replays, Mode::Normal, Language::English) => &["The game's own replays"],
        // What a pointdevice run gives, in the order somebody choosing cares about it: dying costs the
        // chapter, the progress is kept, and the replay is orb's own rather than the game's — which is
        // what makes it hold the attempts that were rewound away as well as the one that survived. See
        // `Game::skip_replay_prompt`, which is the game's own offer being written past, and
        // [`crate::replay`] for what is offered instead.
        (Menu::Run, Mode::Pointdevice, Language::Japanese) => &[
            "被弾したらチャプターの頭からやり直します",
            "進行状況は自動的にセーブされ、いつでも続きから遊べます",
            "リプレイはやり直したトライも残ります",
        ],
        (Menu::Run, Mode::Pointdevice, Language::English) => &[
            "Getting hit starts the chapter over",
            "Your progress is saved and can be picked up at any time",
            "Its replay keeps the attempts you were rewound from too",
        ],
        (Menu::Run, Mode::Normal, Language::Japanese) => {
            &["いつものゲームモードです", "被弾したら残機が減ります"]
        }
        (Menu::Run, Mode::Normal, Language::English) => {
            &["The game as it always was", "Getting hit costs a life"]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CHOICES, Language, Mode, aside, title};
    use crate::game::Menu;

    /// Both languages, everywhere below: a question with a line missing in one of them is a screen
    /// somebody reads with a blank in it.
    const LANGUAGES: [Language; 2] = [Language::Japanese, Language::English];

    /// And every screen the question is asked over, for the same reason: what each of them says is a table
    /// this crate keeps by hand.
    const ASKED: [Menu; 3] = [Menu::Run, Menu::Scores, Menu::Replays];

    /// A run, a ranking and the replays are asked about differently, and each answer that has a
    /// description says what it means.
    #[test]
    fn each_question_says_what_it_is_about() {
        for language in LANGUAGES {
            let asked: std::collections::BTreeSet<&str> =
                ASKED.iter().map(|asked| title(*asked, language)).collect();
            assert_eq!(
                asked.len(),
                ASKED.len(),
                "two of the screens are asked about in the same words",
            );
            for asked in [Menu::Run, Menu::Replays] {
                assert_ne!(
                    aside(asked, Mode::Pointdevice, language),
                    aside(asked, Mode::Normal, language)
                );
            }
        }
    }

    /// **The replays are the one item where the two answers do different things**, so each says what it is:
    /// one is the game's own screen and the other a list of orb's, out of which a stage, a chapter and an
    /// attempt can be chosen.
    #[test]
    fn the_replays_say_which_of_the_two_screens_each_answer_is() {
        let pointdevice = aside(Menu::Replays, Mode::Pointdevice, Language::Japanese).join("\n");
        assert!(pointdevice.contains("トライ"));
        let normal = aside(Menu::Replays, Mode::Normal, Language::Japanese).join("\n");
        assert!(normal.contains("ゲーム本来"));

        let pointdevice = aside(Menu::Replays, Mode::Pointdevice, Language::English).join("\n");
        assert!(pointdevice.contains("attempt"));
        let normal = aside(Menu::Replays, Mode::Normal, Language::English).join("\n");
        assert!(normal.contains("The game's own"));
    }

    /// A ranking is asked with nothing under either choice: what a mode does to a run is not what
    /// somebody looking at a ranking of one came to read.
    #[test]
    fn a_ranking_is_asked_with_no_description_at_all() {
        for language in LANGUAGES {
            for mode in CHOICES {
                assert!(aside(Menu::Scores, mode, language).is_empty());
            }
        }
    }

    /// The two things a pointdevice run does that the game does not: it keeps where the run got to, and its
    /// replay keeps the attempts that were rewound away — which the game's own cannot, its record of inputs
    /// being rewound with the run. Said because the question is the last place the choice between the modes
    /// is still open.
    #[test]
    fn a_pointdevice_run_says_its_progress_is_kept_and_what_its_replay_holds() {
        let japanese = aside(Menu::Run, Mode::Pointdevice, Language::Japanese).join("\n");
        assert!(japanese.contains("セーブ"));
        assert!(japanese.contains("トライ"));

        let english = aside(Menu::Run, Mode::Pointdevice, Language::English).join("\n");
        assert!(english.contains("saved"));
        assert!(english.contains("attempts"));
    }

    /// Pointdevice mode is the first of the two, which is what a cursor starting on neither would land
    /// on — and it is the mode orb exists for.
    #[test]
    fn the_mode_with_chapters_is_the_first_choice_and_is_named_for_th15() {
        assert_eq!(CHOICES[0], Mode::Pointdevice);
        assert_eq!(CHOICES[0].name(Language::Japanese), "完全無欠モード");
        assert_eq!(CHOICES[0].name(Language::English), "Pointdevice Mode");
        assert_eq!(CHOICES[1], Mode::Normal);
    }

    /// Every screen of a question is in one language: a name, a title or a line that came out in the
    /// other is what a table of two languages gets wrong, and it is not something the drawing can
    /// notice.
    #[test]
    fn nothing_english_is_said_in_japanese_and_nothing_japanese_in_english() {
        let japanese = |text: &str| text.chars().any(|c| c > '\u{2000}');
        for asked in ASKED {
            for mode in CHOICES {
                for language in LANGUAGES {
                    let said: Vec<&str> = [mode.name(language), title(asked, language)]
                        .into_iter()
                        .chain(aside(asked, mode, language).iter().copied())
                        .collect();
                    for text in said {
                        assert_eq!(
                            japanese(text),
                            language == Language::Japanese,
                            "{text:?} is not {language}",
                        );
                    }
                }
            }
        }
    }
}
