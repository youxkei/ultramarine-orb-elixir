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
//!
//! **Which of the two a run is, is asked where the run is started**, over the game's own title menu, because
//! that is where it belongs: a run with chapters and a run without are two different things to start, and
//! th15 asks it in the same place. Not a key in `orb.yaml`, which is for what somebody sets once.
//!
//! **On the press, not after it.** The title menu's decide is held back the same way the shot type select's
//! is — `g_CurFrameInput & 0x1001`, tested at 0x437c1b against the frame before — and what the cursor is on
//! is what says whether orb has a question: `MainMenu+0x81a0` bounded to 0..=7 and jumped through the table
//! at 0x4381cc, of which items 0, 1 and 2 start a run and item 4 is the ranking. Answering picks the mode
//! and hands the press over, so the menu chooses its own item; cancelling hands it over to nobody, and the
//! menu is still on the item, with its cursor where it was and nothing of it redrawn.
//!
//! **Only where the screen would have acted on the press.** Each ignores its own decide for its first frames
//! — 20 for the title menu at 0x437c0e, 30 for the shot type select at 0x436c0a — and a press held back over
//! those and handed over afterwards is a keypress the game had thrown away, acted on late. Held from a frame
//! before either grace runs out, because what decides the holding is read after the game's update and
//! applies to the next read: tested for the frame it is on, the frame the grace expires on would go unheld
//! and a press there would start a run nobody was asked about. What the cursor is on is asked apart from
//! this, the mark on the shot type select being drawn from the same reading and having nothing to wait for.
//!
//! **The key a question was cancelled with is held back as well**, until it is let go or until the screen
//! it was cancelled on is gone, whichever comes first, on both screens. It is still down on the frame the
//! game carries on into, and going back is what each of them does with it — the title menu puts its cursor
//! on `Quit` (0x4381b0), the shot type select returns to the character select (0x436c49) — where what was
//! answered is about the screen the question was on. The press after it is the screen's own again:
//! cancelling a question and asking to go back are two different things to ask for. It ends with that
//! screen because a hold that outlived it would be taking the bomb and the pause button out of the run
//! started next, and out of what a resumed run writes down as its own buttons.
//!
//! | | |
//! | --- | --- |
//! | 完全無欠モード / `Pointdevice Mode` | chapters, snapshots, the retry menu, the wash a chapter gets, the lives painted over, the retry count on the status line, `pointdevice_score.dat`, and a replay of orb's own in `pointdevice_replay/` |
//! | レガシーモード / `Legacy Mode` | the game as it was: dying costs a life, the game's own replay can be saved, and the score goes in the game's own `score.dat` |
//!
//! On screen they are th15's own two names, in whichever of the two languages that game itself calls them,
//! since that is where the mode comes from and those are the names somebody who wants it knows. In the
//! code, the log and the file it writes they are pointdevice and normal — the English of the first, and what
//! the second actually is.
//!
//! **Neither is an answer too.** `x` — the game's own bomb key, which its menus read as back — escape, or the
//! pad's cancel, and the front end goes back to the title the way its own back button does: `gameState` to
//! `STATE_CHARACTER_LOAD` and its timer to zero, which 36 frames later reaches `STATE_STARTUP` and falls
//! through to the menu. Copied from the difficulty select's `RETURNMENU` branch rather than undoing what the
//! chosen item did, because the sprites are already running the fade that branch would set and the cursor
//! is already on the item that was chosen.
//!
//! The question goes over three of the title menu's items — a full run, the Extra stage, practice — because
//! each of them starts a run, and over `Score`, because the two modes have a ranking each. One answer for
//! both: orb is in one mode at a time, and the ranking of pointdevice runs *is* the file pointdevice runs
//! are written to. So looking at the other ranking puts orb in the other mode, and the next run asks again
//! — which is why the mode left behind cannot be what a screen reads from: every item that opens the file,
//! the ranking included, asks first.
//!
//! **And over `Replay`, where the two answers do different things rather than the same thing to different
//! files.** `Legacy Mode` there is the item being chosen: the press is handed over and the game shows the
//! replays it saved itself, out of its own files and on its own screen, which orb has nothing to put on.
//! `Pointdevice Mode` is a list of orb's own over an item the game never acted on — the game has no screen
//! for these, having no way to record one. So the question is worded for the screen rather than for the
//! file: `どちらのリプレイを見る` / `Which replays to watch`, with a line under each answer saying which of the
//! two screens it is. See [`crate::replay_ui`].
//!
//! **Which item the cursor is on is `MainMenu+0x81a0`**, bounded to 0..=7 at 0x437c5c and jumped through the
//! table at 0x4381cc: 0, 1 and 2 start a run, 3 is the replays and 4 is the ranking. The three left are the
//! music room, the options and quitting, and orb has nothing to ask about any of them. Nothing of the
//! replay item is read beyond the cursor being on it, and nothing of it is written: the press either
//! reaches the item or never does.
//!
//! **How the moment is caught.** `g_MainMenu.gameState` at 0x6dc8b0 — 0x81f0 into the 0x10f34-byte
//! `MainMenu` at 0x6d46c0, past its `AnmVm vm[122]` of 0x110 each — is read once a frame while the
//! supervisor says the front end is what is running. All three of the items that start a run set it to
//! `STATE_DIFFICULTY_LOAD`, and `Score` sets it to `STATE_SCORE`; each then waits 60 frames before the game
//! acts on it, so there is a second to spare. It is the *change* that is acted on, and coming back from the
//! difficulty select goes through `STATE_CHARACTER_LOAD` rather than through either, so a choice is caught
//! once. `MainMenu::RegisterChain` memsets the whole struct and writes the state afresh, so nothing there is
//! left over from the last time round.
//!
//! **Both of the supervisor's states have to say front end**, not just `curState`. `Supervisor::OnUpdate` is
//! chain priority 0 and assigns `wantedState = curState` as its last act, so a screen that leaves by setting
//! `curState` itself — which is how the ranking leaves — produces one frame ending with `curState` already
//! saying front end, `wantedState` still saying where the game was, and the front end not yet rebuilt.
//! `gameState` on that frame is whatever the screen being left was entered from, and for the ranking that is
//! `STATE_SCORE`: leaving the ranking read as choosing it, and the question was asked again for nothing —
//! `menu: Scores chosen, asking which mode` on the same millisecond as the score file being written on the
//! way out. Requiring `wantedState` too excludes exactly that frame, and the frame after it the memset has
//! taken the stale state away.
//!
//! Nothing has to be undone by the answer, which is why the question can be asked after the game has acted
//! on the keypress rather than instead of it: what the game has done by then is start a fade and set a
//! state, and both of them are wanted whichever mode is chosen. The game is frozen the frame after —
//! `RunCalcChain` returning `CHAIN_BREAK` while the drawing carries on, the mechanism the retry menu uses —
//! so one update of the front end runs after the keypress and none of the second that follows it. The play
//! field's viewport is *not* set for these frames, unlike a retry: what is underneath is a menu, and
//! `prepare_frame` has already given the frame the whole output.
//!
//! **The key that answers it does not also reach the game.** `g_CurFrameInput` is left as every button, so
//! that the `held & ~held-last-frame` every one of the game's own `WAS_PRESSED` is finds nothing on the
//! frame the game carries on into: `Supervisor::OnUpdate` runs first in the calc chain — priority 0 against
//! the main menu's 2 — and its first act is `g_LastFrameInput = g_CurFrameInput; g_CurFrameInput =
//! GetInput()`, so that is the one thing which reads it. What is genuinely held still reads as held, from
//! the fresh read; only the edge goes, and only for that frame.
//!
//! Not zero, which is what the game itself writes into those three at a scene change: zero leaves
//! `g_LastFrameInput` empty and turns every button still down into a fresh press, which is the opposite of
//! what is wanted. The game gets away with it because the screens it changes to guard their own first frames
//! with timers — which is also why doing nothing here would have worked, and is exactly the kind of reason
//! not to rely on: it is a property of the screen behind the question rather than something orb decided. A
//! retry needs none of this, since the snapshot restore puts the same state back with the rest of `.data`.
//!
//! **The fade goes on underneath it**, because `MainMenu::OnDraw` is what advances it —
//! `numFramesSinceActive` against the 60 the chosen item set `framesActive` to — and the drawing is the half
//! that is not frozen. So the title screen darkens to the black the item's fade was heading for and stays
//! there while the question is up, which is a background for it rather than a problem with it. The vms' own
//! scripts do stop, since those are stepped by `ExecuteScript` at the end of `MainMenu::OnUpdate`.
//!
//! **No replay of the game's own is offered for a pointdevice run.** What one recorded here would play back
//! as is a run nobody could tell from a flawless one: the game keeps its record of inputs in the heap a
//! snapshot covers — `ReplayManager` is `new`ed at 0x42a27f and each stage's record allocated per stage — so
//! a retry rewinds the record along with the run, and each attempt writes over the last from the chapter's
//! own frame. What would be saved is therefore the path that survived, with nothing in it of the attempts
//! that did not. **What is offered in its place is a replay of orb's own**, which holds the attempts as well
//! as the path — see [`crate::replay`] — and this is the frame that offer is made on. The screen that offers
//! to save the game's own is `ResultScreen`, reached through the job it registers — `ResultScreen::OnUpdate`
//! at 0x42d98e, the same walk the ending's object is found by — and its `resultScreenState` at +0x8 is
//! written from `SAVE_REPLAY_QUESTION` to `EXIT` on the frame it arrives there, before the frame timer
//! reaches the 60 that starts the question's own animation. So no part of it is ever drawn. `EXIT` is the
//! game's own way out and not something invented for this: it is the state a practice run's result screen is
//! registered in, and its `OnUpdate` case sets the supervisor back to the title menu and takes the job out
//! — which runs `DeletedCallback`, so the score file is still written on the way. The alternative was
//! answering the question for the player, which means writing the interrupt each of the screen's 38 sprites
//! is to run next and then waiting out the fade they play.
//!
//! **That question is the only state of that screen orb writes.** `RegisterChain` starts a finished run's
//! screen in `WRITING_HIGHSCORE_NAME` — `EXIT` for a practice run, which is why both are the states the game
//! skips its own parses in — and the name entry and the stats screen after it are screens whoever just
//! finished the run reads. Nothing of orb's is written into either.
//!
//! Refusing the write instead — which is what `--clear` still does, since a cleared run *does* reach that
//! screen — would leave somebody naming a replay file that never appears.
//!
//! **Nobody is asked** where there is nobody to ask: a pass over a replay (`--collect`, `--judge`,
//! `--replay`), a tuning session (`--tune`) and a clear (`--clear`) take the mode they are given, which is
//! pointdevice. A menu frozen on a question nobody answers is a pass that never ends. Nor is anything asked
//! with `--no-chapters`, where the mode is normal because there is nothing for it to be: orb is loaded with
//! none of its own work happening. And nothing is asked without the overlay, which is what would draw the
//! question — a frozen game with an invisible question over it looks broken, and what is lost by not asking
//! is the mode orb is in already.

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
