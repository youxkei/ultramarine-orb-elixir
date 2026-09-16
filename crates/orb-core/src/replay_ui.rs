//! The screens orb puts up about the replay of a pointdevice run: whether to keep the one just played,
//! and — for one that was kept — which stage, chapter and attempt of it to watch.
//!
//! **The question is orb's own and not the game's.** th06 has a screen that offers to save a replay and orb
//! writes past it, because what the game would save is the path that survived with nothing in it of the
//! attempts that did not — see [`crate::replay`]. So the offer somebody answers is this one, over the same
//! frame the game's own would have been on, and what it keeps is the file orb has been writing all along.
//!
//! Asked only where the setting says to ask. On, the replay is kept and nothing appears; off, this is what
//! decides. Which way round that is is written where the key is.
//!
//! How these read their keys and draw their items is [`crate::menu_ui`], which every question orb asks
//! shares.
//!
//! **Watched by stage, then chapter, then attempt**, which is what a replay holds: a run is stages, a stage
//! is chapters, and a chapter is the attempts made at it. Four lists, one line per item — the replays with
//! whose run each was and when it was played, then the stages, then the chapters by the names the boundary
//! detector gave them, then the attempts by number with what each was under the cursor: the one the run
//! went on from, or one it was rewound away from, and how long it lasted.
//!
//! Twelve items on the screen at once with the cursor held inside the window, a stage having more chapters
//! than that. A cancel goes back a list, and a cancel on the first closes them.
//!
//! **And what was chosen out of every list already walked past is above the one that is up**, a line apiece
//! and in the words that list's own item had: the replay, then its stage, then that stage's chapter. Because
//! the item chosen is what the next list is *about* — three lists deep the screen was `トライを選ぶ` over a
//! column of numbered attempts, with nothing on it to say whose run or which chapter they were attempts at.
//!
//! **The replays newest first**, which is the run somebody has just played; the three lists below it are in
//! the run's own order, a stage list that did not start at stage 1 being no list of a run. Ordered by when
//! each run began, which is what its line says — and not as the directory reads, since a name says which
//! run it was before it says when: `hard-` sorts before `normal-` whatever evening either was played on,
//! which put tonight's runs above last night's and read oldest-first inside each of them.
//!
//! **And all four stand over the screen the game keeps its own replays on**, which orb asks the front end
//! for rather than pressing for: the press that would have chosen that item was held back for the mode
//! question, so what takes the game there is orb writing the state that item writes. A list of replays over
//! the title menu reads as having left the replays rather than as being in them.
//!
//! **Asked for with the frames its coming-up state waits already spent**, so it arrives on the next update
//! rather than a second later. Those frames are the item's own fade, and what they draw is the screen being
//! left — which for a list coming back after an attempt is the title menu a left run goes to, a second of it
//! on the way to a list that exists so that screen need not be walked again.
//!
//! Then the frames that screen is left to *draw* itself in, everything under one of these lists being
//! frozen for as long as it is up: a couple where the list is going up for the first time, the screen behind
//! it having been drawn every frame since somebody walked a cursor to that item, and a second where it is
//! coming back — the front end a left run goes to is one built on the frame the run went, and a list two
//! frames later stood over the flat grey an undrawn screen is.
//!
//! What a cancel out of the first list leaves standing is that screen, whose own cancel is the way back to
//! the title menu — which is where whoever chose the item would have been.
//!
//! A launch with nothing kept says so on the screen rather than nothing appearing, a press held back for a
//! screen that never came being the one thing that would look broken. And a screen asked for that never
//! comes up is the same: after five seconds the list goes over whatever the front end has instead.

use orb_config::Language;

use crate::game::Pad;
use crate::input::Keyboard;
use crate::log;
use crate::menu_ui::{self, ASIDE, By, DIM_SCREEN, Keys, LINE_HEIGHT, NORMAL, Pressed};
use crate::overlay::{Label, Overlay, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Frames before the question accepts anything.
///
/// The same span the retry menu's own confirmations hold: the press that put this up is already spent, so
/// what this is for is a question answered on the frame it appeared — and one of the two answers here
/// throws a run's replay away.
const INPUT_GRACE_FRAMES: u32 = 12;

/// The two answers, in the order they are offered, and what each is worth.
///
/// **Keeping is first, and that is what the cursor starts on.** Not because it is the likelier answer but
/// because of what the two mistakes cost: a replay kept by accident is a file somebody deletes, and one
/// declined by accident is a run nobody can watch again.
const ANSWERS: [bool; 2] = [true, false];

/// What came of putting it up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    /// Kept, and put where the list offers it.
    Keep,
    /// Not kept: the file the run was being written into goes.
    Decline,
}

/// What the question asks, which is what an e2e test reads off the screen to say it is up.
pub fn asked(language: Language) -> &'static str {
    title(language)
}

/// What the question asks.
fn title(language: Language) -> &'static str {
    match language {
        Language::Japanese => "リプレイを保存する",
        Language::English => "Keep this replay",
    }
}

/// What each answer says on screen, in the game's own words for the two.
fn answer(keep: bool, language: Language) -> &'static str {
    match (keep, language) {
        (true, Language::Japanese) => "はい",
        (true, Language::English) => "Yes",
        (false, Language::Japanese) => "いいえ",
        (false, Language::English) => "No",
    }
}

/// What the answer under the cursor means, said under it.
///
/// The line under いいえ is the one worth having: this is the last moment the run is watchable, and nothing
/// after it will say so.
fn aside(keep: bool, language: Language) -> &'static str {
    match (keep, language) {
        (true, Language::Japanese) => "ステージ・チャプター・トライを選んで見られます",
        (true, Language::English) => "Watchable by stage, chapter and attempt",
        (false, Language::Japanese) => "このリプレイは失われます",
        (false, Language::English) => "This replay is lost",
    }
}

/// The question itself, over the screen the run ended at or the one a run was about to be started over on.
pub struct SaveMenu {
    selection: usize,
    keys: Keys,
    language: Language,
    title: Label,
    answers: [Label; ANSWERS.len()],
    aside: Label,
    cursor: Label,
}

impl SaveMenu {
    pub fn new(language: Language) -> Self {
        Self {
            selection: 0,
            keys: Keys::new(INPUT_GRACE_FRAMES),
            language,
            title: Label::new(),
            answers: [const { Label::new() }; ANSWERS.len()],
            aside: Label::new(),
            cursor: Label::new(),
        }
    }

    /// Returns what was answered, once something is. `pad` is what the pad is doing now, which the caller
    /// asks the game to read: the game is frozen here, so its own reading of the pad is not running and a
    /// pad would otherwise do nothing on this question at all.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Answer, By)> {
        let pressed = self.keys.read(keyboard, pad)?;
        let answer = self.step(&pressed)?;
        let by = match answer {
            Answer::Decline => pressed.cancel.or(pressed.decide),
            Answer::Keep => pressed.decide,
        };
        Some((answer, by?))
    }

    /// One of the two answers taken outright, for a launch playing itself: `いいえ` is an item no press can
    /// walk to, the cursor being orb's own — see `runtime::answer_the_question`.
    pub fn answered_outright(&self, item: usize) -> Option<Answer> {
        Some(match ANSWERS.get(item)? {
            true => Answer::Keep,
            false => Answer::Decline,
        })
    }

    fn step(&mut self, pressed: &Pressed) -> Option<Answer> {
        self.selection = menu_ui::moved(self.selection, ANSWERS.len(), pressed);
        // A cancel is the answer that leaves nothing behind, which is what a cancel means everywhere else
        // orb asks something: there is no third way out of this one — the run is over either way — so
        // saying nothing has to mean one of the two, and it means the one that changes nothing on the disk.
        if pressed.cancel.is_some() {
            return Some(Answer::Decline);
        }
        pressed.decide.map(|_| match ANSWERS[self.selection] {
            true => Answer::Keep,
            false => Answer::Decline,
        })
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        unsafe {
            self.title.set(overlay, title(self.language));
            self.cursor.set(overlay, "▶");
            self.aside
                .set(overlay, aside(ANSWERS[self.selection], self.language));
            for (label, keep) in self.answers.iter_mut().zip(ANSWERS) {
                label.set(overlay, answer(keep, self.language));
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        // The whole screen, the way every question orb asks over one of the game's own screens covers it.
        frame.fill(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, DIM_SCREEN);

        let center = SCREEN_WIDTH / 2.0;
        let y = SCREEN_HEIGHT / 2.0 - LINE_HEIGHT * 3.0;
        menu_ui::centred(&frame, &self.title, center, y, NORMAL);
        let y = menu_ui::list(
            &frame,
            &self.answers,
            &self.cursor,
            center,
            y + LINE_HEIGHT * 2.0,
            self.selection,
        );
        menu_ui::centred(&frame, &self.aside, center, y + LINE_HEIGHT, ASIDE);
    }
}

/// Frames the list holds its keys off for, which is the press that opened it: the same span the question
/// above holds, and for the same reason — the first item of a list acted on the press that put the list up
/// is a choice nobody made.
const LIST_GRACE_FRAMES: u32 = 12;

/// How many items of a list are on the screen at once.
///
/// **Twelve, which is what fits.** A line is `LINE_HEIGHT` and the screen is `SCREEN_HEIGHT`, with the
/// title, a blank line and the line under the items to leave room for: what is left is thirteen lines and
/// this is one short of it. A stage of a run has more chapters than that, so the list is a window over them
/// rather than the whole — see [`shown`].
const SHOWN: usize = 12;

/// Which of the four lists is up, which is what a decide moves on from and a cancel goes back to.
///
/// **Four and not one screen per level of a file**, because that is what a replay of a pointdevice run is: a
/// run is stages, a stage is chapters, and a chapter is the attempts that were made at it. Each of them is a
/// list of one line per item, and what tells them apart is what the line says.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Choosing {
    /// The replays there are, which is what the item on the title menu leads to.
    Replay,
    /// The stages of the one chosen.
    Stage,
    /// The chapters of that stage.
    Chapter,
    /// And the attempts at that chapter, which is the last of the four.
    Attempt,
}

impl Choosing {
    /// What the list asks, which is the line above its items.
    fn title(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Replay, Language::Japanese) => "リプレイを選ぶ",
            (Self::Replay, Language::English) => "Choose a replay",
            (Self::Stage, Language::Japanese) => "ステージを選ぶ",
            (Self::Stage, Language::English) => "Choose a stage",
            (Self::Chapter, Language::Japanese) => "チャプターを選ぶ",
            (Self::Chapter, Language::English) => "Choose a chapter",
            (Self::Attempt, Language::Japanese) => "トライを選ぶ",
            (Self::Attempt, Language::English) => "Choose an attempt",
        }
    }

    /// The list a decide moves on to, and `None` for the last of them — which is where a decide is the
    /// answer rather than another list.
    fn next(self) -> Option<Self> {
        match self {
            Self::Replay => Some(Self::Stage),
            Self::Stage => Some(Self::Chapter),
            Self::Chapter => Some(Self::Attempt),
            Self::Attempt => None,
        }
    }

    /// And the one a cancel goes back to, `None` being the list closing.
    fn back(self) -> Option<Self> {
        match self {
            Self::Replay => None,
            Self::Stage => Some(Self::Replay),
            Self::Chapter => Some(Self::Stage),
            Self::Attempt => Some(Self::Chapter),
        }
    }
}

/// What the list says where there is nothing to choose from.
///
/// Said rather than the list never appearing, which is what a press held back for a screen that never came
/// would be: the answer to *which replays* was pointdevice, and this is that answer with nothing behind it.
fn nothing_kept(language: Language) -> &'static str {
    match language {
        Language::Japanese => "保存されたリプレイがありません",
        Language::English => "No replays have been kept",
    }
}

/// And what the line under the attempts says of the one under the cursor: the last of them is the one the
/// run went on from, and every other is one it was rewound away from.
fn how_it_ended(survived: bool, language: Language) -> &'static str {
    match (survived, language) {
        (true, Language::Japanese) => "このトライで先に進みました",
        (true, Language::English) => "The attempt the run went on from",
        (false, Language::Japanese) => "このトライはやり直しになりました",
        (false, Language::English) => "An attempt that was rewound away",
    }
}

/// One attempt at a chapter, as the list needs it: nothing of what was pressed, which is the file's.
pub struct ListedAttempt {
    /// How many frames it lasted, for the line the item shows.
    pub frames: u32,
    /// Whether it is the one the run went on from, which is the last of a chapter's.
    pub survived: bool,
}

/// One chapter of a stage, as the list needs it.
pub struct ListedChapter {
    /// Which chapter of the stage it is, as the run counted them.
    pub number: u32,
    /// What it is called, which is what the item says.
    pub name: String,
    pub attempts: Vec<ListedAttempt>,
}

/// One stage of a replay.
pub struct ListedStage {
    /// The stage as somebody counts them, which is one more than the run does.
    pub stage: i32,
    pub chapters: Vec<ListedChapter>,
}

/// One replay, as the list of them needs it.
pub struct Listed {
    /// The file's own name, which is what reading it again asks for and what the answer names.
    pub name: String,
    /// The line the item shows: whose run it was, which run, and when it was played.
    pub said: String,
    /// When the run began, as the file itself says — which is what the list is ordered by. Here rather than
    /// read out of the name: the name carries the same evening, and the file is where it is a fact.
    pub started: String,
    pub stages: Vec<ListedStage>,
}

/// What came of the lists: an attempt to watch, or nothing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Watched {
    /// One attempt of one chapter of one stage of one replay, each named as the file names it.
    Chosen {
        /// The file, by the name [`Listed::name`] gave it.
        replay: String,
        /// Which of the replay's stages, chapters and attempts, each counted from nought the way the file
        /// holds them: what turns these back into a stretch of a stage is the file, and it is read again
        /// rather than carried here.
        stage: usize,
        chapter: usize,
        attempt: usize,
    },
    /// Nothing: the list is closed and the game's own menu is where it was — the press that would have
    /// chosen its item was held back for the question this came from and is handed over unanswered.
    Cancelled,
}

/// The lists themselves, over the frozen title menu.
pub struct WatchMenu {
    replays: Vec<Listed>,
    choosing: Choosing,
    /// Which item of each list the cursor is on, kept per list so that going back lands where it was.
    at: [usize; 4],
    keys: Keys,
    language: Language,
    title: Label,
    items: [Label; SHOWN],
    /// The lines under the items, which are two where the attempts are being chosen between: what the one
    /// under the cursor *was*, and how long it lasted. Two lines rather than one because they are two facts
    /// — and a line that ran them together would be a screen where neither can be read on its own.
    aside: [Label; ASIDE_LINES],
    /// And the lines above the title: what was chosen out of each list already walked past — see
    /// [`WatchMenu::chosen_so_far`].
    chosen: [Label; CHOSEN_LINES],
    cursor: Label,
}

/// How many lines the list keeps under its items. Two, which is what the attempts have; every other list
/// says one thing or nothing.
const ASIDE_LINES: usize = 2;

/// And how many it keeps above its title: one per list already chosen out of, which is three at the deepest.
/// The number is the file's own shape — a run is stages, a stage is chapters, a chapter is the attempts made
/// at it — and not a count of anything that grows.
const CHOSEN_LINES: usize = 3;

impl WatchMenu {
    /// **Newest first**, which is the order this list is read in: the run somebody has just played is the one
    /// they are most likely to want to watch, and the four lists below it are in the run's own order already.
    ///
    /// Sorted here rather than taken as the directory reads, because a directory reads by name and a name
    /// says which run it was before it says when — `hard-` before `normal-`, whatever evening either was
    /// played on. What that put on the screen was tonight's runs above last night's *and* oldest-first
    /// inside each of them, which is the order looking wrong in two directions at once. Ties go to the name,
    /// there being nothing else left to tell two runs of one second apart by.
    pub fn new(mut replays: Vec<Listed>, language: Language) -> Self {
        replays.sort_by(|one, other| {
            other
                .started
                .cmp(&one.started)
                .then_with(|| one.name.cmp(&other.name))
        });
        Self {
            replays,
            choosing: Choosing::Replay,
            at: [0; 4],
            keys: Keys::new(LIST_GRACE_FRAMES),
            language,
            title: Label::new(),
            items: [const { Label::new() }; SHOWN],
            aside: [const { Label::new() }; ASIDE_LINES],
            chosen: [const { Label::new() }; CHOSEN_LINES],
            cursor: Label::new(),
        }
    }

    /// **Put back up after the attempt it was left on was watched**, which is the same list at the same item
    /// and only its keys to hold off again: the press that ended the attempt, or the one somebody stopped it
    /// with, is not a press at the list that comes back.
    pub fn opened_again(&mut self) {
        self.keys.hold(LIST_GRACE_FRAMES);
    }

    /// Returns what was chosen, once something is. `pad` is what the pad is doing now, which the caller asks
    /// the game to read: the game is frozen here, so its own reading of the pad is not running.
    pub fn update(&mut self, keyboard: &Keyboard, pad: Pad) -> Option<(Watched, By)> {
        let pressed = self.keys.read(keyboard, pad)?;
        let by = pressed.decide.or(pressed.cancel);
        let chosen = self.step(&pressed)?;
        Some((chosen, by?))
    }

    /// **An item of the list up now taken outright**, which is what a launch playing itself walks these with:
    /// the cursor is orb's own and nothing of it reaches a bot, so a press only ever takes the first item of
    /// each — and a replay is chosen out of four lists, none of whose lengths a recipe can see.
    ///
    /// Answers with the attempt where the list up is the last of the four, and `None` for every list before
    /// it: those are moves to the next list and not answers, the same as a decide on them. `None` too for an
    /// item this list has not got, which is a recipe naming one that is not there — said by the caller,
    /// which is the one place that knows both the number and the list.
    pub fn answered_outright(&mut self, item: usize) -> Option<Watched> {
        let count = self.items().len();
        if item >= count {
            return None;
        }
        self.at[self.choosing as usize] = item;
        match self.choosing.next() {
            Some(next) => {
                self.said_what_was_chosen();
                self.choosing = next;
                self.at[next as usize] = 0;
                self.keys.hold(LIST_GRACE_FRAMES);
                None
            }
            None => Some(Watched::Chosen {
                replay: self.replays[self.at[Choosing::Replay as usize]]
                    .name
                    .clone(),
                stage: self.at[Choosing::Stage as usize],
                chapter: self.at[Choosing::Chapter as usize],
                attempt: self.at[Choosing::Attempt as usize],
            }),
        }
    }

    /// How many items the list up now has, for whoever is saying which of them it means: a recipe naming one
    /// that is not there is worth a line saying how many there were.
    pub fn items_now(&self) -> usize {
        self.items().len()
    }

    /// And which of the four lists is up, said as the word a recipe names it by.
    pub fn list_now(&self) -> &'static str {
        match self.choosing {
            Choosing::Replay => "replay",
            Choosing::Stage => "stage",
            Choosing::Chapter => "chapter",
            Choosing::Attempt => "attempt",
        }
    }

    fn step(&mut self, pressed: &Pressed) -> Option<Watched> {
        let count = self.items().len();
        if count > 0 {
            let at = &mut self.at[self.choosing as usize];
            *at = menu_ui::moved(*at, count, pressed);
        }
        if pressed.cancel.is_some() {
            match self.choosing.back() {
                Some(back) => {
                    self.choosing = back;
                    // The keys held off again, this press having been spent on going back: a list arrived
                    // at with a decide still down would choose the item the cursor lands on.
                    self.keys.hold(LIST_GRACE_FRAMES);
                    return None;
                }
                None => return Some(Watched::Cancelled),
            }
        }
        pressed.decide?;
        // Nothing to choose is nothing a decide does, on any of the four: what is on the screen then is the
        // line saying there is nothing, and a cancel is the way out.
        if count == 0 {
            return None;
        }
        match self.choosing.next() {
            Some(next) => {
                self.said_what_was_chosen();
                self.choosing = next;
                self.at[next as usize] = 0;
                self.keys.hold(LIST_GRACE_FRAMES);
                None
            }
            None => Some(Watched::Chosen {
                replay: self.replays[self.at[Choosing::Replay as usize]]
                    .name
                    .clone(),
                stage: self.at[Choosing::Stage as usize],
                chapter: self.at[Choosing::Chapter as usize],
                attempt: self.at[Choosing::Attempt as usize],
            }),
        }
    }

    /// Says what the item just chosen was and how much is behind it, which is the one thing about walking
    /// these lists that a log can be read for: a replay of a run played across two sessions has stages a
    /// launch may or may not have written down, and how many of them there are is the answer.
    ///
    /// Said by the menu rather than by whoever put it up, because what was chosen is only here: the answer
    /// this hands back is the *last* of the four, and the three before it are moves nothing else sees.
    fn said_what_was_chosen(&self) {
        match self.choosing {
            Choosing::Replay => {
                if let Some(replay) = self.chosen_replay() {
                    log!(
                        "replay: {} chosen; {} stage(s) in it",
                        replay.name,
                        replay.stages.len(),
                    );
                }
            }
            Choosing::Stage => {
                if let Some(stage) = self.chosen_stage() {
                    log!(
                        "replay: stage {} chosen; {} chapter(s) in it",
                        stage.stage,
                        stage.chapters.len(),
                    );
                }
            }
            Choosing::Chapter => {
                if let Some(chapter) = self.chosen_chapter() {
                    log!(
                        "replay: chapter {} ({}) chosen; {} attempt(s) at it",
                        chapter.number,
                        chapter.name,
                        chapter.attempts.len(),
                    );
                }
            }
            // The last of the four is not a move to another list: it is the answer, and what says which
            // attempt it was is the line the run it is watched in is started with.
            Choosing::Attempt => {}
        }
    }

    /// Which replay, stage and chapter the cursor has settled on, as far as the list it is on has been
    /// answered: each of these is `None` where its own list has not been reached.
    fn chosen_replay(&self) -> Option<&Listed> {
        self.replays.get(self.at[Choosing::Replay as usize])
    }

    fn chosen_stage(&self) -> Option<&ListedStage> {
        self.chosen_replay()?
            .stages
            .get(self.at[Choosing::Stage as usize])
    }

    fn chosen_chapter(&self) -> Option<&ListedChapter> {
        self.chosen_stage()?
            .chapters
            .get(self.at[Choosing::Chapter as usize])
    }

    /// The lines the list up now is made of, in the order it shows them.
    fn items(&self) -> Vec<String> {
        match self.choosing {
            Choosing::Replay => self
                .replays
                .iter()
                .map(|replay| replay.said.clone())
                .collect(),
            Choosing::Stage => self
                .chosen_replay()
                .map(|replay| {
                    replay
                        .stages
                        .iter()
                        .map(|it| stage_said(it.stage))
                        .collect()
                })
                .unwrap_or_default(),
            Choosing::Chapter => self
                .chosen_stage()
                .map(|stage| {
                    stage
                        .chapters
                        .iter()
                        .map(|chapter| chapter.name.clone())
                        .collect()
                })
                .unwrap_or_default(),
            Choosing::Attempt => self
                .chosen_chapter()
                .map(|chapter| {
                    (1..=chapter.attempts.len())
                        .map(|which| attempt_said(which, self.language))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// What the lines under the items say of the one the cursor is on, in the order they are drawn — and
    /// nothing at all where it has nothing to say.
    fn said_of_the_item(&self) -> Vec<String> {
        match self.choosing {
            Choosing::Replay | Choosing::Stage => Vec::new(),
            // How many attempts there were at it, which is what somebody choosing a chapter wants: a
            // chapter played once and one played eleven times are what a run's history is.
            Choosing::Chapter => self
                .chosen_chapter()
                .map(|chapter| vec![attempts_said(chapter.attempts.len(), self.language)])
                .unwrap_or_default(),
            Choosing::Attempt => self
                .chosen_chapter()
                .and_then(|chapter| chapter.attempts.get(self.at[Choosing::Attempt as usize]))
                .map(|attempt| {
                    vec![
                        how_it_ended(attempt.survived, self.language).to_owned(),
                        seconds_said(attempt.frames, self.language),
                    ]
                })
                .unwrap_or_default(),
        }
    }

    /// **What was chosen out of each list already walked past**, a line apiece and in the order they were
    /// walked: the replay, then its stage, then that stage's chapter.
    ///
    /// Because the item chosen is what the next list is *about*, and a list says nothing about it: three
    /// lists deep the screen was `トライを選ぶ` over a column of numbered attempts, with nothing on it to
    /// say whose run, which stage or which chapter they were attempts at. Each line is the same text that
    /// list's own item had, so what is read here is what was chosen there.
    fn chosen_so_far(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.choosing == Choosing::Replay {
            return lines;
        }
        lines.extend(self.chosen_replay().map(|replay| replay.said.clone()));
        if self.choosing == Choosing::Stage {
            return lines;
        }
        lines.extend(self.chosen_stage().map(|stage| stage_said(stage.stage)));
        if self.choosing == Choosing::Chapter {
            return lines;
        }
        lines.extend(self.chosen_chapter().map(|chapter| chapter.name.clone()));
        lines
    }

    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay) {
        let items = self.items();
        let at = self.at[self.choosing as usize];
        let (from, shown) = shown(items.len(), at);
        // The line saying there is nothing where the list is empty, which is the one thing a list with no
        // items has to say — see [`nothing_kept`].
        let mut said = self.said_of_the_item();
        if items.is_empty() {
            said = vec![nothing_kept(self.language).to_owned()];
        }
        let chosen = self.chosen_so_far();
        unsafe {
            self.title.set(overlay, self.choosing.title(self.language));
            self.cursor.set(overlay, "▶");
            for (label, line) in self.aside.iter_mut().zip(&said) {
                label.set(overlay, line);
            }
            for (label, line) in self.chosen.iter_mut().zip(&chosen) {
                label.set(overlay, line);
            }
            for (label, line) in self.items.iter_mut().zip(&items[from..from + shown]) {
                label.set(overlay, line);
            }
        }

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        frame.fill(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, DIM_SCREEN);

        let center = SCREEN_WIDTH / 2.0;
        // Centred on the screen with as many lines as this list has, the way the retry menu centres its
        // own: a fixed top would put a list of two items against the ceiling. The blank line between what
        // was chosen and the title is counted with the lines it separates.
        let above = if chosen.is_empty() {
            0.0
        } else {
            chosen.len() as f32 + 1.0
        };
        let lines = 2.0 + shown as f32 + said.len() as f32 + above;
        let mut y = (SCREEN_HEIGHT - lines * LINE_HEIGHT) / 2.0;
        // Read as what a title has under it rather than as items, which is what they are: dimmer than the
        // list, and above the line saying what this list is a list of.
        for label in self.chosen.iter().take(chosen.len()) {
            menu_ui::centred(&frame, label, center, y, ASIDE);
            y += LINE_HEIGHT;
        }
        y += above.min(1.0) * LINE_HEIGHT;
        menu_ui::centred(&frame, &self.title, center, y, NORMAL);
        let mut y = menu_ui::list(
            &frame,
            &self.items[..shown],
            &self.cursor,
            center,
            y + LINE_HEIGHT * 2.0,
            at - from,
        );
        // Only the lines this item has: the labels past them still hold whatever the item before said, and
        // the ones that are set are set by what is up now.
        for label in self.aside.iter().take(said.len()) {
            menu_ui::centred(&frame, label, center, y, ASIDE);
            y += LINE_HEIGHT;
        }
    }
}

/// What the list says where a name would be, for the replay of a run nobody finished — see
/// [`crate::replay::Ended`].
///
/// The game's own word for a run left where it stood, which is the one the offer to pick one up uses:
/// `中断データあり` on the screen a run is started from is the same 中断 as this.
pub fn interrupted(language: Language) -> &'static str {
    match language {
        Language::Japanese => "中断",
        Language::English => "Interrupted",
    }
}

/// What one stage's item says, and what the line above the lists below it says once it is chosen — one
/// function so that the two read the same. The game's own word for a stage, which is the same in both
/// languages: the panel a run is played under says `STAGE` too.
fn stage_said(stage: i32) -> String {
    format!("STAGE {stage}")
}

/// What one attempt's item says, which is its number and nothing else: what it *was* is the line under the
/// items, there being one cursor and one line to say it on.
fn attempt_said(which: usize, language: Language) -> String {
    match language {
        Language::Japanese => format!("トライ {which}"),
        Language::English => format!("Attempt {which}"),
    }
}

/// How many attempts a chapter has, said under it.
fn attempts_said(attempts: usize, language: Language) -> String {
    match language {
        Language::Japanese => format!("{attempts} トライ"),
        Language::English => format!("{attempts} attempt(s)"),
    }
}

/// How long an attempt lasted, in seconds of the game's own frames — sixty to one, which is what th06 and
/// th07 both run at.
///
/// Seconds rather than frames, because what somebody choosing between attempts is judging is how far each
/// one got: a number of frames is that number divided by sixty in their head.
fn seconds_said(frames: u32, language: Language) -> String {
    let seconds = frames / 60;
    match language {
        Language::Japanese => format!("{seconds}秒"),
        Language::English => format!("{seconds}s"),
    }
}

/// Which slice of a list is on the screen, as the first item shown and how many: the window is [`SHOWN`]
/// long and holds the cursor inside it, and a list that fits is shown whole.
fn shown(items: usize, at: usize) -> (usize, usize) {
    if items <= SHOWN {
        return (0, items);
    }
    // Half a window either side of the cursor where there is room for it, and pinned at either end where
    // there is not: a window that ran past the end would leave blank lines under the last item.
    let from = at.saturating_sub(SHOWN / 2).min(items - SHOWN);
    (from, SHOWN)
}

#[cfg(test)]
mod tests {
    use super::{
        ANSWERS, Answer, Language, Listed, ListedAttempt, ListedChapter, ListedStage, Pressed,
        SaveMenu, WatchMenu, Watched, aside, title,
    };
    use crate::menu_ui::By;

    /// The language these screens are read in. Which one changes nothing they decide, so it is the one the
    /// game itself is in.
    const LANGUAGE: Language = Language::Japanese;

    fn open() -> SaveMenu {
        let mut menu = SaveMenu::new(LANGUAGE);
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

    fn decide() -> Pressed {
        Pressed {
            decide: Some(By::Keyboard),
            ..nothing()
        }
    }

    /// Keeping it is what the cursor starts on: the mistake worth avoiding is the run nobody can watch
    /// again.
    #[test]
    fn keeping_the_replay_is_what_the_cursor_starts_on() {
        assert!(ANSWERS[0], "the first answer is not the one that keeps it");
        assert_eq!(open().step(&decide()), Some(Answer::Keep));
    }

    /// A direction is the other answer, both ways: with two of them either direction is the other one.
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
            assert_eq!(menu.step(&decide()), Some(Answer::Decline));
        }
    }

    /// Cancelling is declining: there is no third way out of this question, the run being over either way,
    /// so saying nothing means the answer that changes nothing on the disk.
    #[test]
    fn cancelling_declines_it() {
        let mut menu = open();
        assert_eq!(
            menu.step(&Pressed {
                cancel: Some(By::Keyboard),
                ..nothing()
            }),
            Some(Answer::Decline),
        );
    }

    /// Nothing is answered on the frames it has just gone up, whatever is pressed: the press that put it
    /// there is still down.
    #[test]
    fn it_holds_its_keys_off_first() {
        assert!(SaveMenu::new(LANGUAGE).keys.held() > 0);
    }

    /// Each answer says what it means, and the two say different things: what いいえ costs is the one thing
    /// this screen is the last chance to read.
    #[test]
    fn each_answer_says_what_it_means_in_both_languages() {
        for language in [Language::Japanese, Language::English] {
            assert_ne!(aside(true, language), aside(false, language));
            assert!(!title(language).is_empty());
        }
    }

    /// Every word of a screen is in one language: a line that came out in the other is what a table of two
    /// languages gets wrong, and the drawing cannot notice it.
    #[test]
    fn nothing_english_is_said_in_japanese_and_nothing_japanese_in_english() {
        let japanese = |text: &str| text.chars().any(|c| c > '\u{2000}');
        for language in [Language::Japanese, Language::English] {
            for keep in ANSWERS {
                for text in [
                    title(language),
                    aside(keep, language),
                    super::answer(keep, language),
                ] {
                    assert_eq!(
                        japanese(text),
                        language == Language::Japanese,
                        "{text:?} is not {language}",
                    );
                }
            }
        }
    }

    /// **The newest replay first**, which is the order somebody reads this list in: the run they have just
    /// played is the one they are most likely to want to watch.
    ///
    /// The names sort by which run it was before they sort by when — `hard-` before `normal-`, whatever
    /// evening either was played on — so a list in the order the directory reads put tonight's lunatic run
    /// above last night's normal ones and then read oldest-first inside each of them.
    #[test]
    fn the_replays_are_listed_newest_first() {
        let listed = |started: &str| Listed {
            name: format!("normal-reimu-a-{started}"),
            said: started.to_owned(),
            started: started.to_owned(),
            stages: vec![ListedStage {
                stage: 1,
                chapters: vec![ListedChapter {
                    number: 1,
                    name: "MIDSTAGE 1".to_owned(),
                    attempts: vec![ListedAttempt {
                        frames: 60,
                        survived: true,
                    }],
                }],
            }],
        };
        let mut menu = WatchMenu::new(
            vec![
                listed("20040815-201500"),
                listed("20040816-013000"),
                listed("20040815-214233"),
            ],
            LANGUAGE,
        );
        // The first item of each of the four lists, which is the newest replay and the one stage, chapter
        // and attempt in it.
        let chosen = (0..4)
            .find_map(|_| menu.answered_outright(0))
            .expect("an attempt chosen out of the lists");
        assert_eq!(
            chosen,
            Watched::Chosen {
                replay: "normal-reimu-a-20040816-013000".to_owned(),
                stage: 0,
                chapter: 0,
                attempt: 0,
            },
        );
    }
}
