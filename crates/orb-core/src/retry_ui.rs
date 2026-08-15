//! The menu that appears where the chapter was lost.
//!
//! Four ways on, and behind the second of them the chapters the stage has behind the one that was lost —
//! see [`crate::chapter`], which keeps a snapshot of each. A screen of its own rather than the ways on
//! becoming that list: the first item is answered every few seconds in a fight and is one press with
//! nothing to read, and the chapters are what somebody reads when that item is not the answer.
//!
//! How it reads its keys and draws its items is [`crate::menu_ui`], which the other two questions
//! orb asks share.

use orb_config::Language;

use crate::game::{Pad, Rect};
use crate::input::Keyboard;
use crate::log;
use crate::menu_ui::{self, ASIDE, By, DIM_FIELD, Keys, LINE_HEIGHT, NORMAL, Pressed};
use crate::overlay::{Label, Overlay};

/// Frames before the menu accepts anything. The player was holding keys when they
/// died — very likely a direction and the shoot key — and those presses belong to
/// the run, not to this menu.
const INPUT_GRACE_FRAMES: u32 = 24;

/// Frames before a confirmation, or the chapters behind the one that was lost, accepts anything.
///
/// Shorter than the menu's own grace, and for a different reason: what is being kept out
/// is not the run's keys but the press that opened the question. That press is an edge and
/// so is already spent, which is what makes this a few frames rather than a fifth of a
/// second — but a question answered on the frame after it appeared is a question nobody
/// read, and the answer here throws a stage or a run away.
///
/// The chapters hold their keys off for the same span and for the whole of that reason: one of them is
/// acted on the press that lands on it, so a press left over from opening the screen would send the run
/// back to whichever chapter the cursor started on.
const CONFIRM_GRACE_FRAMES: u32 = 12;

/// A chapter behind the one that was lost, as the menu is handed it.
pub struct Chapter {
    /// Which of the stage's snapshots puts it back, which is what choosing it names.
    pub at: usize,
    /// What that chapter is called, which is what the item says on screen.
    pub name: String,
    /// Whether it is the stage's own start.
    pub stage_start: bool,
}

/// The ways on, in the order they are offered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Way {
    /// The chapter that was lost, which is what this menu exists for.
    Chapter,
    /// The chapters behind it, which this lists rather than acts on — so it is the one way on that
    /// decides nothing.
    Further,
    /// The stage's own start, which is the oldest chapter there is.
    Stage,
    /// The run given up.
    Quit,
}

impl Way {
    /// What the item says on screen.
    ///
    /// The last says where it ends up rather than what it gives up, because that is the part somebody
    /// reading the item does not know: the run ending is the obvious half, and that the game itself
    /// carries on is not.
    fn text(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Chapter, Language::Japanese) => "チャプターをやり直す",
            (Self::Chapter, Language::English) => "Retry the chapter",
            (Self::Further, Language::Japanese) => "更に前からやり直す",
            (Self::Further, Language::English) => "Retry from further back",
            (Self::Stage, Language::Japanese) => "ステージをやり直す",
            (Self::Stage, Language::English) => "Retry the stage",
            (Self::Quit, Language::Japanese) => "タイトルに戻る",
            (Self::Quit, Language::English) => "Back to the title screen",
        }
    }
}

/// What the menu decides, once it decides anything.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    /// The chapter that was lost, which is the newest snapshot the stage has.
    Chapter,
    /// One further back, by which of the stage's snapshots puts it back.
    Further(usize),
    /// The stage's own start.
    Stage,
    /// The run given up: back to the title menu, the way the game's own pause menu leaves one.
    Quit,
}

impl Choice {
    /// For the log, in the English the rest of it is in — and it stays this wording whichever language
    /// the menu is in, a log being read beside a source tree rather than by whoever is playing.
    ///
    /// Which chapter one further back was is said by the restore's own line, so what this says is which
    /// of the menu's items was answered.
    pub fn label(self) -> &'static str {
        match self {
            Self::Chapter => "the chapter again",
            Self::Further(_) => "a chapter further back",
            Self::Stage => "the stage again",
            Self::Quit => "the run given up",
        }
    }
}

/// What the chapters behind the one that was lost are being chosen for, written above them.
///
/// A question, because the items under it are places rather than sentences: each is named the way the
/// status line names it, and the verb they all share is here.
fn title(language: Language) -> &'static str {
    match language {
        Language::Japanese => "どこからやり直す",
        Language::English => "Where to start again",
    }
}

/// The second question a choice asks, where it asks one. `chapter` is what the chapter one further
/// back is called, which that question names.
///
/// Nothing for the chapter that was lost, which is the choice this menu exists for: it is answered
/// every time a chapter is lost, which in a fight worth grinding is every few seconds, and a
/// question in front of it would be a question answered without reading it — which is worse than no
/// question at all, since it also trains the hand that then answers what is below.
///
/// Every other one asks, because none of them can be taken back: the chapters after the one put back
/// go with the restore, the stage's start throws away everything the stage has gained since it, and
/// giving up throws away the run.
///
/// The one about a chapter **names it** rather than saying *this chapter*: the item it was chosen from
/// is not on the screen the question replaces it with, and the line under the question is about another
/// chapter — the one the run is in. See [`left_behind`].
fn question(choice: Choice, chapter: &str, language: Language) -> Option<String> {
    match (choice, language) {
        (Choice::Chapter, _) => None,
        (Choice::Further(_), Language::Japanese) => Some(format!("{chapter} からやり直す？")),
        (Choice::Further(_), Language::English) => Some(format!("Start again from {chapter}?")),
        (Choice::Stage, Language::Japanese) => Some("ステージの最初からやり直す？".to_owned()),
        (Choice::Stage, Language::English) => {
            Some("Start the stage over from the beginning?".to_owned())
        }
        (Choice::Quit, Language::Japanese) => Some("やめてタイトルに戻る？".to_owned()),
        (Choice::Quit, Language::English) => {
            Some("Give up and go back to the title screen?".to_owned())
        }
    }
}

/// What answering はい leaves behind, said under the question.
///
/// Going back drops the chapters after the one it puts back — the run has not played them from there —
/// so the chapter it is being answered in is not one it can come back to. Which is the half of what is
/// about to happen that the question does not say, and the half worth being sure of: what looked like
/// stepping back one attack is the fight from that chapter onwards again.
///
/// **The line the questions used to have was a different line and was taken out**: it spelled out what
/// a stage's worth of progress was, which the question naming the stage's start already says. This one
/// says what no item and no question on either screen does.
///
/// Nothing under giving up: what that costs is the run, which its own item names, and the chapter it
/// was in is written down for a later launch — see *Picking a run up again*.
fn left_behind(choice: Choice, language: Language) -> Option<&'static str> {
    match (choice, language) {
        (Choice::Chapter | Choice::Quit, _) => None,
        (Choice::Further(_) | Choice::Stage, Language::Japanese) => {
            Some("今のチャプターには戻れません")
        }
        (Choice::Further(_) | Choice::Stage, Language::English) => {
            Some("There is no going back to the chapter you are in")
        }
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

/// Which of the menu's two lists the cursor is in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Listing {
    /// The ways on.
    Ways,
    /// The chapters behind the one that was lost.
    Chapters,
}

/// What the menu is showing.
#[derive(Clone, Copy)]
enum Showing {
    /// One of the two lists.
    Listing(Listing),
    /// The question one of its items asked: which item, and the list it was asked from — cancelling
    /// goes back to the items the hand was among rather than to the ways on.
    Confirming(Choice, Listing),
}

pub struct RetryMenu {
    showing: Showing,
    /// Which of the ways on the cursor is on, and which of the chapters. One apiece, because coming
    /// back from the chapters has to find the ways where they were left, and a screen opened again
    /// starts at the chapter nearest the one that was lost.
    way: usize,
    chapter: usize,
    /// Which answer the cursor is on, while a confirmation is up.
    answer: usize,
    /// Which language its words are in, kept for the same reason [`crate::mode_ui::ModeMenu`] keeps
    /// it: what the menu decides is the same either way, and every word of it is a function of this.
    language: Language,
    keys: Keys,
    ways: Vec<Way>,
    /// The chapters behind the one that was lost, newest first.
    ///
    /// Settled where the menu goes up rather than read per frame, because the game is frozen under it:
    /// these are the chapters the stage had on the frame the death was noticed, and nothing can add to
    /// them or take one away while it is up.
    chapters: Vec<Chapter>,
    header: Label,
    retry: Label,
    /// A line for each item of whichever list is showing, so there are as many as the longer of the
    /// two.
    items: Vec<Label>,
    asked: Label,
    /// What the question's はい leaves behind, under it.
    aside: Label,
    answers: [Label; ANSWERS.len()],
    cursor: Label,
}

impl RetryMenu {
    /// `chapters` are the ones behind the chapter that was lost, in the order they are listed. Empty
    /// leaves 更に前からやり直す out of the ways on: an item with nothing behind it is a screen with
    /// nothing on it.
    pub fn new(language: Language, chapters: Vec<Chapter>) -> Self {
        let mut ways = vec![Way::Chapter];
        if !chapters.is_empty() {
            ways.push(Way::Further);
        }
        ways.extend([Way::Stage, Way::Quit]);
        let lines = ways.len().max(chapters.len());
        Self {
            showing: Showing::Listing(Listing::Ways),
            way: 0,
            chapter: 0,
            answer: NO,
            language,
            keys: Keys::new(INPUT_GRACE_FRAMES),
            ways,
            chapters,
            header: Label::new(),
            retry: Label::new(),
            items: (0..lines).map(|_| Label::new()).collect(),
            asked: Label::new(),
            aside: Label::new(),
            answers: [Label::new(), Label::new()],
            cursor: Label::new(),
        }
    }

    /// The question a choice asks before it acts, and `None` where it acts on the press.
    ///
    /// The chapter a question about one further back names is the one under the cursor: a cursor does
    /// not move while a question is up, so it is still on the item the question is about.
    fn asking(&self, choice: Choice) -> Option<String> {
        let named = self
            .chapters
            .get(self.chapter)
            .map_or("", |chapter| chapter.name.as_str());
        question(choice, named, self.language)
    }

    /// And what that question's はい leaves behind, where it leaves anything: nothing where the stage's
    /// own start is the chapter the run is in, which is a death before the stage's first boundary —
    /// going back to where the run stands leaves nothing behind at all.
    fn warning(&self, choice: Choice) -> Option<&'static str> {
        (!self.chapters.is_empty())
            .then(|| left_behind(choice, self.language))
            .flatten()
    }

    /// Returns the choice once it is confirmed. `pad` is what the pad is doing now, read for this
    /// menu by the caller.
    ///
    /// Nothing cancels the menu itself: the player is dead, and its items are the only ways on.
    /// Cancelling a confirmation, or the chapters, goes back to them.
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
            Showing::Listing(Listing::Ways) => {
                self.way = menu_ui::moved(self.way, self.ways.len(), pressed);
                pressed.decide?;
                match self.ways[self.way] {
                    Way::Chapter => self.decided(Choice::Chapter),
                    Way::Further => {
                        self.showing = Showing::Listing(Listing::Chapters);
                        self.chapter = 0;
                        self.keys.hold(CONFIRM_GRACE_FRAMES);
                        log!("retry: asking which chapter further back");
                        None
                    }
                    Way::Stage => self.decided(Choice::Stage),
                    Way::Quit => self.decided(Choice::Quit),
                }
            }
            Showing::Listing(Listing::Chapters) => {
                self.chapter = menu_ui::moved(self.chapter, self.chapters.len(), pressed);
                if pressed.cancel.is_some() {
                    self.showing = Showing::Listing(Listing::Ways);
                    log!("retry: which chapter further back — cancelled, back to the choices");
                    return None;
                }
                pressed.decide?;
                let chapter = &self.chapters[self.chapter];
                // The stage's own start is the last of them and is the same place the way on below
                // them puts back, so it asks the same question here: one act cannot be guarded on one
                // screen and not on the other.
                self.decided(if chapter.stage_start {
                    Choice::Stage
                } else {
                    Choice::Further(chapter.at)
                })
            }
            Showing::Confirming(choice, from) => {
                self.answer = menu_ui::moved(self.answer, ANSWERS.len(), pressed);
                if pressed.cancel.is_some() {
                    self.back_to_items(choice, from, "cancelled");
                    return None;
                }
                pressed.decide?;
                if ANSWERS[self.answer] {
                    return Some(choice);
                }
                self.back_to_items(choice, from, "answered no");
                None
            }
        }
    }

    /// Acts on the choice, or puts up the question it asks first.
    fn decided(&mut self, chosen: Choice) -> Option<Choice> {
        if self.asking(chosen).is_none() {
            return Some(chosen);
        }
        // Which list the hand was in, which is where cancelling the question goes back to.
        let from = match self.showing {
            Showing::Listing(from) | Showing::Confirming(_, from) => from,
        };
        self.showing = Showing::Confirming(chosen, from);
        self.answer = NO;
        self.keys.hold(CONFIRM_GRACE_FRAMES);
        log!("retry: asking about {}", chosen.label());
        None
    }

    /// Said rather than passed over: what a confirmation is for is stopping something, and a
    /// session that lost a stage anyway has to be able to see whether the stop happened.
    fn back_to_items(&mut self, choice: Choice, from: Listing, how: &str) {
        self.showing = Showing::Listing(from);
        log!("retry: {} — {how}, back to the choices", choice.label());
    }

    /// Drawn into the game's own back buffer through the D3D overlay, which is right for this menu —
    /// it belongs over the game — and carries the game's repainting with it: anything drawn this way
    /// where the game does not repaint accumulates instead of being replaced.
    ///
    /// `chapter` is what the chapter that was lost is called, which is written above the ways on.
    ///
    /// # Safety
    /// Must run between the game's `BeginScene` and `EndScene`.
    pub unsafe fn draw(&mut self, overlay: &Overlay, area: Rect, chapter: &str, retries: u32) {
        // Which list the items on the screen belong to, a confirmation being read against the one it
        // was asked from: that is where the cancel goes back to, so the line above it stays put.
        let listing = match self.showing {
            Showing::Listing(listing) => listing,
            Showing::Confirming(_, from) => from,
        };
        // Every label this frame will draw, baked before the overlay's frame is opened: baking
        // one creates a texture and locks it, and the frame is a window with the device's state
        // captured and replaced. Nothing about the two is known to collide, which is the reason
        // not to find out.
        let shown = unsafe {
            self.header.set(
                overlay,
                match listing {
                    Listing::Ways => chapter,
                    Listing::Chapters => title(self.language),
                },
            );
            self.retry.set(overlay, &format!("RETRY {retries}"));
            self.cursor.set(overlay, "▶");
            match self.showing {
                Showing::Listing(Listing::Ways) => {
                    for (label, way) in self.items.iter_mut().zip(&self.ways) {
                        label.set(overlay, way.text(self.language));
                    }
                    self.ways.len()
                }
                Showing::Listing(Listing::Chapters) => {
                    for (label, chapter) in self.items.iter_mut().zip(&self.chapters) {
                        label.set(overlay, &chapter.name);
                    }
                    self.chapters.len()
                }
                Showing::Confirming(choice, _) => {
                    if let Some(asked) = self.asking(choice) {
                        self.asked.set(overlay, &asked);
                    }
                    let left_behind = self.warning(choice);
                    if let Some(left_behind) = left_behind {
                        self.aside.set(overlay, left_behind);
                    }
                    for (label, yes) in self.answers.iter_mut().zip(ANSWERS) {
                        label.set(overlay, answer(yes, self.language));
                    }
                    // The question, what its はい leaves behind where it says so, the blank line, and
                    // the answers.
                    2 + ANSWERS.len() + usize::from(left_behind.is_some())
                }
            }
        };

        let frame = unsafe { overlay.frame() };
        let Some(frame) = frame else { return };
        frame.fill(area.left, area.top, area.width, area.height, DIM_FIELD);

        let center = area.center_x();
        // The lines this screen has — what is written above the items, the retries, the blank line
        // under them, and the items — so that the block is centred in the field whichever screen it
        // is and however many chapters the stage has. Held at a fixed height instead, a stage deep
        // enough to have kept eight of them would list the last few past the bottom of the field.
        let mut y = area.center_y() - LINE_HEIGHT * (3 + shown) as f32 / 2.0;
        for label in [&self.header, &self.retry] {
            menu_ui::centred(&frame, label, center, y, NORMAL);
            y += LINE_HEIGHT;
        }
        y += LINE_HEIGHT;

        match self.showing {
            Showing::Listing(listing) => {
                let selection = match listing {
                    Listing::Ways => self.way,
                    Listing::Chapters => self.chapter,
                };
                menu_ui::list(
                    &frame,
                    &self.items[..shown],
                    &self.cursor,
                    center,
                    y,
                    selection,
                );
            }
            Showing::Confirming(choice, _) => {
                menu_ui::centred(&frame, &self.asked, center, y, NORMAL);
                // The answers a blank line under whichever of the two the question ended with, so that
                // they read as its answers rather than as more items.
                let mut answers = y + LINE_HEIGHT * 2.0;
                if self.warning(choice).is_some() {
                    menu_ui::centred(&frame, &self.aside, center, y + LINE_HEIGHT, ASIDE);
                    answers += LINE_HEIGHT;
                }
                menu_ui::list(
                    &frame,
                    &self.answers,
                    &self.cursor,
                    center,
                    answers,
                    self.answer,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ANSWERS, CONFIRM_GRACE_FRAMES, Chapter, Choice, Listing, NO, Pressed, RetryMenu, Showing,
        Way, question,
    };
    use crate::game::Rect;
    use crate::menu_ui::{ASIDE, By, DIM_FIELD, LINE_HEIGHT, NORMAL, SELECTED};
    use crate::overlay::Drawing;
    use orb_config::Language;
    use orb_sim::Quad;

    /// The language every menu below is read in. Which one changes nothing these tests assert — what
    /// a press means, and where a line lands — so it is the one the game itself is in.
    const LANGUAGE: Language = Language::Japanese;

    /// The chapters behind the one that was lost, a fight into a stage: nearest first, with the
    /// stage's own start the last of them, each by the name the boundary detector gave it.
    fn behind_it() -> Vec<Chapter> {
        vec![
            Chapter {
                at: 2,
                name: "MIDBOSS NONSPELL 1".to_owned(),
                stage_start: false,
            },
            Chapter {
                at: 1,
                name: "MIDSTAGE 2".to_owned(),
                stage_start: false,
            },
            Chapter {
                at: 0,
                name: "MIDSTAGE 1".to_owned(),
                stage_start: true,
            },
        ]
    }

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
        opened_on(behind_it())
    }

    fn opened_on(chapters: Vec<Chapter>) -> RetryMenu {
        let mut menu = RetryMenu::new(LANGUAGE, chapters);
        menu.keys.hold(0);
        menu
    }

    /// Walks the cursor down to a way on and presses decide on it.
    fn choose(menu: &mut RetryMenu, way: Way) -> Option<Choice> {
        let at = menu
            .ways
            .iter()
            .position(|item| *item == way)
            .expect("the menu offers it");
        while menu.way != at {
            assert_eq!(menu.step(&down()), None);
        }
        menu.step(&decide())
    }

    /// Lets the grace a confirmation or the chapters hold their keys off for run out, which `update`
    /// is what spends — so a test that drives `step` has to spend it itself.
    fn read_it(menu: &mut RetryMenu) {
        assert!(menu.keys.held() > 0, "the screen holds its keys off first");
        menu.keys.hold(0);
    }

    /// The four ways on, in order, and the cursor on the chapter that was lost.
    #[test]
    fn the_ways_on_are_the_chapter_the_chapters_behind_it_the_stage_and_giving_up() {
        let menu = open();
        assert_eq!(
            menu.ways,
            [Way::Chapter, Way::Further, Way::Stage, Way::Quit],
        );
        assert_eq!(menu.way, 0);
    }

    /// With nothing behind the chapter that was lost — a death before the stage's first boundary —
    /// there is no item for the chapters: what is behind it is an empty screen.
    #[test]
    fn the_chapters_behind_it_are_not_offered_where_there_are_none() {
        let menu = opened_on(Vec::new());
        assert_eq!(menu.ways, [Way::Chapter, Way::Stage, Way::Quit]);
    }

    /// The chapter that was lost is acted on the press that chose it: it is answered every few
    /// seconds in a fight, and a question there would be answered unread.
    #[test]
    fn the_chapter_that_was_lost_is_not_asked_about() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Chapter), Some(Choice::Chapter));
        for language in [Language::Japanese, Language::English] {
            assert!(question(Choice::Chapter, "MIDSTAGE 2", language).is_none());
        }
    }

    /// 更に前からやり直す decides nothing: it lists the chapters behind the one that was lost, nearest
    /// first, and each press down is one chapter further back.
    ///
    /// One of them asks before it acts, and the question **names it**: the item it was chosen from is
    /// not on the screen the question replaces it with.
    #[test]
    fn the_chapters_behind_it_are_listed_and_one_of_them_asks_naming_it() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Further), None);
        assert!(matches!(menu.showing, Showing::Listing(Listing::Chapters)));
        assert_eq!(
            menu.chapter, 0,
            "on the chapter nearest the one that was lost"
        );

        read_it(&mut menu);
        assert_eq!(menu.step(&decide()), None, "it acted without asking");
        assert_eq!(
            menu.asking(Choice::Further(2)).as_deref(),
            Some("MIDBOSS NONSPELL 1 からやり直す？"),
        );
        read_it(&mut menu);
        assert_eq!(menu.step(&down()), None);
        assert!(ANSWERS[menu.answer], "on yes");
        assert_eq!(menu.step(&decide()), Some(Choice::Further(2)));

        // And one press down the list is the chapter behind that one, whose question names it in turn.
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Further), None);
        read_it(&mut menu);
        assert_eq!(menu.step(&down()), None);
        assert_eq!(menu.step(&decide()), None);
        assert_eq!(
            menu.asking(Choice::Further(1)).as_deref(),
            Some("MIDSTAGE 2 からやり直す？"),
        );
        read_it(&mut menu);
        assert_eq!(menu.step(&down()), None);
        assert_eq!(menu.step(&decide()), Some(Choice::Further(1)));
    }

    /// What every question that goes back says under itself: the chapters after the one put back go
    /// with the restore, so the chapter it is answered in is not one the run can come back to.
    ///
    /// Nothing under the two that leave nothing behind — the chapter that was lost, and giving up,
    /// whose chapter is written down for a later launch.
    #[test]
    fn the_questions_that_go_back_say_what_they_leave_behind() {
        let menu = open();
        for choice in [Choice::Further(2), Choice::Stage] {
            assert_eq!(
                menu.warning(choice),
                Some("今のチャプターには戻れません"),
                "{choice:?} says nothing about what it leaves behind",
            );
        }
        for choice in [Choice::Chapter, Choice::Quit] {
            assert_eq!(menu.warning(choice), None);
        }
        // And nothing at all where the stage's own start is the chapter the run is in: there is no
        // chapter behind it to leave.
        assert_eq!(opened_on(Vec::new()).warning(Choice::Stage), None);
    }

    /// Nothing is chosen on the frames that screen has just gone up, whatever is pressed: the press
    /// that opened it is spent, and one left over would send the run back to whichever chapter the
    /// cursor started on.
    #[test]
    fn the_chapters_behind_it_hold_their_keys_off_before_one_is_chosen() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Further), None);
        assert_eq!(menu.keys.held(), CONFIRM_GRACE_FRAMES);
    }

    /// The stage's own start is the last of them and asks the same question there as the way on below
    /// them does: one act cannot be guarded on one screen and not on the other.
    #[test]
    fn the_stages_start_asks_wherever_it_is_chosen() {
        for reached in [None, Some(Way::Further)] {
            let mut menu = open();
            if let Some(way) = reached {
                assert_eq!(choose(&mut menu, way), None);
                read_it(&mut menu);
                // Down to the last of the chapters, which is the stage's own start.
                for _ in 0..menu.chapters.len() - 1 {
                    assert_eq!(menu.step(&down()), None);
                }
                assert_eq!(menu.step(&decide()), None);
            } else {
                assert_eq!(choose(&mut menu, Way::Stage), None);
            }
            assert!(
                matches!(menu.showing, Showing::Confirming(Choice::Stage, _)),
                "the stage's own start acted without asking",
            );
            read_it(&mut menu);
            assert_eq!(menu.step(&down()), None);
            assert!(ANSWERS[menu.answer], "on yes");
            assert_eq!(menu.step(&decide()), Some(Choice::Stage));
        }
    }

    /// And giving up asks, and nothing has happened when the question goes up: it takes the cursor
    /// moved onto yes and a second press.
    #[test]
    fn giving_up_asks_first() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Quit), None);
        assert!(matches!(
            menu.showing,
            Showing::Confirming(Choice::Quit, Listing::Ways)
        ));

        read_it(&mut menu);
        assert_eq!(menu.step(&down()), None);
        assert!(ANSWERS[menu.answer], "on yes");
        assert_eq!(menu.step(&decide()), Some(Choice::Quit));
    }

    /// The cursor starts on no, so the press that lands on the frame the grace ends — which is
    /// what a held key does — costs nothing but the question closing.
    #[test]
    fn a_confirmation_starts_on_no_and_no_goes_back() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Quit), None);
        assert_eq!(menu.answer, NO);
        assert!(!ANSWERS[menu.answer]);

        read_it(&mut menu);
        assert_eq!(menu.step(&decide()), None);
        assert!(matches!(menu.showing, Showing::Listing(Listing::Ways)));
        // And the cursor is still on the item that was asked about, not moved by the answering.
        assert_eq!(menu.ways[menu.way], Way::Quit);
    }

    /// A question asked from the chapters goes back to the chapters, which is where the hand was: the
    /// way out of one asked by mistake leaves the cursor where the mistake was made.
    #[test]
    fn a_confirmation_asked_from_the_chapters_goes_back_to_them() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Further), None);
        read_it(&mut menu);
        for _ in 0..menu.chapters.len() - 1 {
            assert_eq!(menu.step(&down()), None);
        }
        assert_eq!(menu.step(&decide()), None);
        read_it(&mut menu);
        assert_eq!(menu.step(&cancel()), None);
        assert!(matches!(menu.showing, Showing::Listing(Listing::Chapters)));
    }

    /// The way out of the chapters, which the ways on have none of: the player is dead and its items
    /// are the only ways on, so what a cancel closes is a screen above them.
    #[test]
    fn cancelling_the_chapters_goes_back_to_the_ways_on() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Further), None);
        read_it(&mut menu);
        assert_eq!(menu.step(&cancel()), None);
        assert!(matches!(menu.showing, Showing::Listing(Listing::Ways)));
        // On the item that opened it, so the chapter that was lost is one press away again.
        assert_eq!(menu.ways[menu.way], Way::Further);
    }

    /// Nothing is answered on the frames a confirmation has just gone up, whatever is pressed.
    #[test]
    fn a_confirmation_holds_its_keys_off_before_it_answers() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Quit), None);
        assert_eq!(menu.keys.held(), CONFIRM_GRACE_FRAMES);
    }

    /// A direction is the other answer, both ways: with two of them there is nowhere else for
    /// one to go, and a cursor that only moved one way would leave `up` doing nothing.
    #[test]
    fn either_direction_moves_between_the_two_answers() {
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Stage), None);
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

    /// What the chapter that was lost is called, written above the ways on.
    const LOST: &str = "MIDBOSS SPELL 1";

    /// One frame of the menu, on a screen of its own.
    fn frame(drawing: &Drawing, menu: &mut RetryMenu) -> Vec<Quad> {
        drawing.frame(|overlay| unsafe { menu.draw(overlay, FIELD, LOST, 3) })
    }

    /// Every line something was written on, top to bottom. Lines rather than quads, because there are
    /// more quads than lines: the drop shadow under a label is the same line a pixel down, the cursor
    /// is a label of its own beside the item it marks, and the wash is a quad that is no line at all.
    fn lines(quads: &[Quad]) -> Vec<f32> {
        let mut lines: Vec<f32> = quads
            .iter()
            .filter(|quad| quad.color == NORMAL || quad.color == SELECTED)
            .map(|quad| quad.y)
            .collect();
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

    /// However many chapters the stage has, what the menu draws is inside the field and centred in
    /// it: a stage deep enough to have kept eight of them would otherwise list the last few below
    /// the bottom of the play area, where nothing is drawn over and nothing can be read.
    #[test]
    fn the_lines_are_centred_in_the_field_whatever_the_stage_has_kept() {
        let drawing = Drawing::new();
        let deep: Vec<Chapter> = (0..7)
            .map(|back| Chapter {
                at: 6 - back,
                name: format!("MIDSTAGE {}", 7 - back),
                stage_start: back == 6,
            })
            .collect();
        for chapters in [Vec::new(), behind_it(), deep] {
            let mut menu = opened_on(chapters);
            // Both screens: the ways on, and the chapters behind the one that was lost where there
            // are any.
            let offered = !menu.chapters.is_empty();
            for screen in 0..if offered { 2 } else { 1 } {
                if screen == 1 {
                    assert_eq!(choose(&mut menu, Way::Further), None);
                    read_it(&mut menu);
                }
                let quads = frame(&drawing, &mut menu);
                let written = lines(&quads);
                let (top, bottom) = (written[0], *written.last().expect("something was written"));
                assert!(top > FIELD.top, "the first line is above the field");
                assert!(
                    bottom + LINE_HEIGHT < FIELD.top + FIELD.height,
                    "the last line is below the field",
                );
                assert!(
                    ((top + bottom) / 2.0 - FIELD.center_y()).abs() < LINE_HEIGHT,
                    "the lines are not centred in the field: {top} to {bottom}",
                );
            }
        }
    }

    /// A confirmation is a different screen in the same room: a question, what answering it leaves
    /// behind on the line under that, and the two answers rather than the items — with the field
    /// washed under all of it just the same.
    ///
    /// The blank line between the question and its answers is what makes them read as answers rather
    /// than as two more items, and it is the thing to hold this to: both screens draw a list under a
    /// header, so a count of labels or of lines would say nothing. The line about what is left behind
    /// is inside that gap and in the colour that says it is not an item either.
    #[test]
    fn a_confirmation_puts_its_answers_a_line_below_what_it_asks() {
        let drawing = Drawing::new();
        let mut menu = open();
        assert_eq!(choose(&mut menu, Way::Stage), None);
        assert!(matches!(menu.showing, Showing::Confirming(..)));
        let quads = frame(&drawing, &mut menu);

        assert_eq!(quads[0].color, DIM_FIELD, "still washed");
        // What is written above the items, the retries, the question, and its two answers: the
        // answers are the last two lines written and the question is the one above them.
        let written = lines(&quads);
        let asked = written[written.len() - 3];
        let answers = &written[written.len() - 2..];
        let aside: Vec<f32> = quads
            .iter()
            .filter(|quad| quad.color == ASIDE)
            .map(|quad| quad.y)
            .collect();
        assert_eq!(
            aside,
            [asked + LINE_HEIGHT],
            "what はい leaves behind is not the line under the question",
        );
        assert_eq!(answers[0] - asked, LINE_HEIGHT * 3.0);
        assert_eq!(answers[1] - answers[0], LINE_HEIGHT);

        let on = lit(&quads);
        assert_eq!(on.len(), 2, "the answer and its cursor: {on:?}");
        assert_eq!(on[0], on[1]);
    }
}
