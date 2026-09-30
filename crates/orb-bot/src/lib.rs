//! **The harness that plays the game, so that a run to be picked up out of a file costs nobody an
//! afternoon.**
//!
//! Every measurement of a resume has cost somebody a stage of th07 and another every time the file grew a
//! field. A run the machine plays is a run to be had again on demand, and the same run every launch — which is
//! what makes two launches comparable at all.
//!
//! **It plays the way somebody does.** No invulnerability and no keys sent from outside: the word goes in
//! where a resumed run's own recorded words go in, and what it is made of is the game's own numbers read where
//! the game keeps them. Every record a played run leaves is left, the score file and the ranking included —
//! a run written down differently is not the thing a resume is measured against.
//!
//! **So a screen of the game's own is pressed at and never written past**, which is where the imitation ends
//! and what [`Step::Answer`] is the one exception to: a question orb puts up keeps a cursor of its own that
//! nothing of it reaches, so an item of one of those is named rather than walked to — and the game's own
//! screens, whose cursors the game keeps, are walked. The high score's name entry, the stats screen after it,
//! the difficulty and shot selects and the game's own pause menu are all pressed through by a `press` and a
//! count of frames, however long that takes; orb has a state to write past two of them and a recipe does not
//! reach for it, because a launch that skipped a screen would be a launch that never showed the screen could
//! be walked.
//!
//! **And what plays the game for the player is not this.** `--clear` makes nothing able to hit the player and
//! `--sent-keys` has another program press the keys; either is a launch somebody is looking at rather than a
//! run being played, so a measurement of what a run leaves behind cannot lean on one — see
//! `orb_real`'s own doc, where the cases are.
//!
//! **Its own crate so that none of it is in the logic that ships.** `orb_core::runtime::plays_the_run` is a
//! slot, `orb` fills it where `--autoplay` names a recipe, and a launch nobody asked to play itself reaches
//! none of this. The same shape every other seam in this tree has — `window::install_over`,
//! `score::install_over`, `joystick::install_over` — where orb-core keeps the slot and something else decides
//! what goes in it.
//!
//! **A run has to be started before it can be played, and starting one is menus.** That is not a rule about
//! playing — it is a sequence somebody carries out once — so it is written down rather than worked out:
//! `Config::autoplay` is the text of a recipe and [`orb_config::recipe`] is its grammar.
//!
//! ```yaml
//! - press: [decide]                        # the title, the difficulty, the character, the shot and orb's own
//!   frames: 10                             # question — tapped, a menu answering the frame a button goes down
//!   until: { stage: 1, chapter: 1 }         # and stopping at something no press of its own can take away
//!   give_up_after: 3600
//! - take_hits: { chance: 0.2,               # and a chapter in five played without dodging, so that the run
//!     times_each_retry: 0.5 }               # dies and retries the way one somebody plays does
//! - play_until: { stage: 2, chapter: 3 }    # play by the rules as far as stage 2's midboss
//!   give_up_after: 36000                    # or close, a run that died leaving a menu the rules do not press
//! - leave: true                            # the run given up for the title, which is how a session is
//! - quit: true                             # interrupted — and then the game closed
//! ```
//!
//! **What each name is, is the game's** — `Game::button_bit`, `decide` being a shot and a return key in th07
//! and something else in the next game. A shared vocabulary could only be the union of every game's, and Touhou is
//! not one kind of game to take a union of.
//!
//! **The rules a run is played by are [`steer`]'s**, off what `Game::what_a_bot_sees` hands over: reading a
//! game's own arrays needs that game's addresses and deciding what to do about them needs none, so the reading
//! is the game seam's and the deciding is here, where the rest of the harness is.

use std::ops::Range;
use std::sync::Mutex;

use orb_config::recipe::{self, Until};
use orb_core::game::{Game, Question, Reached};
use orb_core::log;
use orb_core::runtime;

/// **Ways of dodging one card of one game**, each named for its card.
mod cards;
/// **Being hit on purpose**, which is what a `take_hits` step asks for.
mod hits;
/// The rules a stage is played by, which is what a recipe falls through to.
mod steer;
/// **The autoplay engine's own ways**, for each game and difficulty.
mod ways;

/// The recipe this launch is playing to and where it has got to, or `None` for a recipe that could not be
/// read.
///
/// A `Mutex` because what reads it is the game's own input read, on the game's main thread, and what writes it
/// is [`install`] at the attach — which is not that thread on every launch. Uncontended either way, one lock a
/// frame being nothing beside a frame.
static PLAYING: Mutex<Option<Playing>> = Mutex::new(None);

/// How long a tap holds and then lets go for, where the walker taps something a recipe did not ask for: long
/// enough that a screen with a wait of its own takes the press, short enough to be through a menu of a few
/// items in a moment.
const A_TAP: u32 = 10;

/// How often a step that has not ended says how long it has been going.
///
/// **Which is what a launch nobody is watching needs of its log.** Three launches in a row sat on a screen the
/// rules could not answer, and every one of them was found by somebody looking at the screen rather than at the
/// log — so a step says where it is at, and a stall is a thing to `grep` for. Ten seconds of frames, which is
/// often enough to catch one and rare enough that a `play_until` a whole stage away says it a few dozen times.
const STILL_WAITING: u32 = 600;

struct Playing {
    steps: Vec<Step>,
    /// Which step is being carried out, and how many frames of it have gone by. A step is finished by moving
    /// this on rather than by taking it out of the list, so that a log line can say which of how many.
    at: usize,
    frames: u32,
    /// **What the rules are to do about being hit**, which a [`Step::TakeHits`] sets and which every frame
    /// after it asks — the chapter it is in being what the answer is about rather than the step.
    hits: hits::TakingHits,
    /// **Whether the rules may press the bomb**, which a [`Step::Bombs`] sets and every frame after it is
    /// steered under. True until a step says otherwise, which is the rules bombing as they see fit.
    bombs: bool,
    /// Whether the step now being carried out has asked for its answer, for a [`Step::Answer`]: what ends one
    /// is the question it named being *gone*, and a question that was never up is one to wait for. Without
    /// this the two are the same frame, and a step whose question never came up read as done.
    asked: bool,
    /// Whether the step now being carried out has seen the run under way, for a [`Step::Play`]: what ends one
    /// early is the run being *over*, and a run that has not started yet is not that. The frame a resume's
    /// answer lands on has the run wanted and no run built — th07 spends one such frame between the answer and
    /// the build — and a `play` step starting there read the run as over and gave up before it began.
    in_run_seen: bool,
    /// **The place a [`Step::StandAt`] written without a card told the rules**, which stands until another
    /// one says otherwise.
    standing: Option<steer::Standing>,
    /// **The ways the recipe named for where the run is**, each taken up there in place of the engine's.
    written: Ways,
    /// **And the autoplay engine's own for the run's difficulty**, read the first frame that difficulty is seen —
    /// see [`ways`].
    engine: Option<(i32, Ways)>,
    /// **The chapter the run is in and the stage frame it was first seen on**, which a way written for frames of a
    /// chapter counts them from.
    chapter_began: Option<(i32, u32, u32)>,
    /// What the rules were last handed of the places, so that a card's place taking over or lapsing is said
    /// the once.
    told: Option<steer::Standing>,
    /// **The last card seen up, and the stage it was up in**, which is what says whose fire a boss's between two
    /// cards is.
    card_before: Option<(i32, u32)>,
    /// And the one the rules were last handed, for the same reason as [`Playing::told`].
    dodging: Option<recipe::Dodge>,
    /// And the card whose path the rules were last handed, for the same reason as [`Playing::told`].
    following: Option<u32>,
    /// And the attempt they were last handed — see [`hands_the_rules_their_attempt`].
    attempt: u32,
}

/// **The ways a run is played by where it is**: the places told and the ways named for a card, the fire after one,
/// a chapter and a stretch of a stage, and the paths written for a card — a recipe's own, or the autoplay engine's
/// for one difficulty (see [`ways`]).
#[derive(Default)]
struct Ways {
    /// **The places written for a card**, each standing while its own card runs — see [`recipe::Step::StandAt`].
    for_cards: Vec<(u32, steer::Standing)>,
    /// **And for a stretch of a stage**, each standing while that stage's frame is inside it — see
    /// [`recipe::Step::StandAtThroughAStretch`].
    for_stretches: Vec<(i32, Range<u32>, steer::Standing)>,
    /// **And for a chapter of a stage**, or a stretch of its frames — see [`recipe::Step::StandAtThroughAChapter`].
    for_chapters: Vec<(i32, u32, Option<Range<u32>>, steer::Standing)>,
    /// **The ways named for a card**, each taken up while its own card runs — see [`recipe::Step::Dodge`].
    dodges: Vec<(u32, recipe::Dodge)>,
    /// **And for a stretch of a stage**, each taken up while that stage's frame is inside it — see
    /// [`recipe::Step::DodgeAStretch`].
    stretches: Vec<(i32, Range<u32>, recipe::Dodge)>,
    /// **And for the fire after a card**, each taken up between that card and the next — see
    /// [`recipe::Step::DodgeAfterACard`].
    after_cards: Vec<(u32, recipe::Dodge)>,
    /// **And for a chapter of a stage**, or a stretch of its frames — see [`recipe::Step::DodgeAChapter`].
    chapters: Vec<(i32, u32, Option<Range<u32>>, recipe::Dodge)>,
    /// **The paths written for a card**, each gone along while its own card runs — see [`recipe::Step::Follow`].
    follows: Vec<(u32, steer::Following)>,
}

/// **Where in its run a frame is, for a way to be named for**: the stage, the chapter and how many frames into it, the
/// stage's own frame, the card that is up and the card whose fire is up after it.
struct Where {
    stage: i32,
    chapter: u32,
    into_the_chapter: Option<u32>,
    stage_frame: Option<u32>,
    card: Option<u32>,
    after_the_card: Option<u32>,
}

/// **Which of a [`Ways`]' names a way was taken up by**, for the line that says so.
#[derive(Clone, Debug, PartialEq)]
enum NamedBy {
    Card(u32),
    AfterTheCard(u32),
    Chapter(i32, u32),
    Stretch(i32, Range<u32>),
}

impl Ways {
    /// **The way a step names kept**, in place of one named for the same card, fire, chapter or stretch before it —
    /// and whether the step named one at all.
    fn take(&mut self, step: &Step) -> bool {
        match step {
            Step::StandAt {
                standing: Some(place),
                spell: Some(card),
            } => {
                self.for_cards.retain(|(it, _)| it != card);
                self.for_cards.push((*card, *place));
            }
            Step::StandAtThroughAStretch {
                standing,
                stage,
                frames,
            } => {
                self.for_stretches
                    .retain(|(at, over, _)| (at, over) != (stage, frames));
                self.for_stretches.push((*stage, frames.clone(), *standing));
            }
            Step::StandAtThroughAChapter {
                standing,
                stage,
                chapter,
                frames,
            } => {
                self.for_chapters
                    .retain(|(at, of, over, _)| (at, of, over) != (stage, chapter, frames));
                self.for_chapters
                    .push((*stage, *chapter, frames.clone(), *standing));
            }
            Step::Dodge { with, spell } => {
                self.dodges.retain(|(card, _)| card != spell);
                self.dodges.push((*spell, *with));
            }
            Step::DodgeAStretch {
                with,
                stage,
                frames,
            } => {
                self.stretches
                    .retain(|(at, over, _)| (at, over) != (stage, frames));
                self.stretches.push((*stage, frames.clone(), *with));
            }
            Step::DodgeAfterACard { with, after } => {
                self.after_cards.retain(|(card, _)| card != after);
                self.after_cards.push((*after, *with));
            }
            Step::DodgeAChapter {
                with,
                stage,
                chapter,
                frames,
            } => {
                self.chapters
                    .retain(|(at, of, over, _)| (at, of, over) != (stage, chapter, frames));
                self.chapters
                    .push((*stage, *chapter, frames.clone(), *with));
            }
            Step::Follow { spell, following } => {
                self.follows.retain(|(card, _)| card != spell);
                self.follows.push((*spell, following.clone()));
            }
            _ => return false,
        }
        true
    }

    /// **The place told for where the run is**: a card's where one is up, a chapter's, and a stretch's — the
    /// narrowest first.
    fn the_place(&self, at: &Where) -> Option<steer::Standing> {
        at.card
            .and_then(|up| {
                self.for_cards
                    .iter()
                    .find(|(card, _)| *card == up)
                    .map(|(_, place)| *place)
            })
            .or_else(|| {
                self.for_chapters
                    .iter()
                    .find(|(stage, chapter, frames, _)| {
                        (*stage, *chapter) == (at.stage, at.chapter) && Self::inside(frames, at)
                    })
                    .map(|(.., place)| *place)
            })
            .or_else(|| {
                let frame = at.stage_frame?;
                self.for_stretches
                    .iter()
                    .find(|(stage, frames, _)| *stage == at.stage && frames.contains(&frame))
                    .map(|(.., place)| *place)
            })
    }

    /// **The way named for where the run is**, and what named it: the card's own where one is up, the fire after a
    /// card's between that card and the next, a chapter's, and a stretch's — the narrowest first.
    fn the_dodge(&self, at: &Where) -> Option<(recipe::Dodge, NamedBy)> {
        let a_cards = at.card.and_then(|up| {
            self.dodges
                .iter()
                .find(|(card, _)| *card == up)
                .map(|(card, with)| (*with, NamedBy::Card(*card)))
        });
        let after_a_card = || {
            at.after_the_card.and_then(|before| {
                self.after_cards
                    .iter()
                    .find(|(card, _)| *card == before)
                    .map(|(card, with)| (*with, NamedBy::AfterTheCard(*card)))
            })
        };
        let a_chapters = || {
            self.chapters
                .iter()
                .find(|(stage, chapter, frames, _)| {
                    (*stage, *chapter) == (at.stage, at.chapter) && Self::inside(frames, at)
                })
                .map(|(stage, chapter, _, with)| (*with, NamedBy::Chapter(*stage, *chapter)))
        };
        let a_stretchs = || {
            let frame = at.stage_frame?;
            self.stretches
                .iter()
                .find(|(stage, frames, _)| *stage == at.stage && frames.contains(&frame))
                .map(|(stage, frames, with)| (*with, NamedBy::Stretch(*stage, frames.clone())))
        };
        a_cards
            .or_else(after_a_card)
            .or_else(a_chapters)
            .or_else(a_stretchs)
    }

    /// **The path written for the card that is up**.
    fn the_path(&self, at: &Where) -> Option<(u32, &steer::Following)> {
        let up = at.card?;
        self.follows
            .iter()
            .find(|(card, _)| *card == up)
            .map(|(card, path)| (*card, path))
    }

    /// Whether a way written for some of a chapter's frames is for this one of them, or for all of it.
    fn inside(frames: &Option<Range<u32>>, at: &Where) -> bool {
        match (frames, at.into_the_chapter) {
            (None, _) => true,
            (Some(frames), Some(into)) => frames.contains(&into),
            (Some(_), None) => false,
        }
    }
}

impl Playing {
    /// **Where in its run this frame is**, the chapter's own frame counted from the one it was first seen on.
    fn where_the_run_is(&self, reached: &Reached) -> Where {
        Where {
            stage: reached.stage,
            chapter: reached.chapter,
            into_the_chapter: self
                .chapter_began
                .filter(|(stage, chapter, _)| {
                    (*stage, *chapter) == (reached.stage, reached.chapter)
                })
                .zip(reached.stage_frames)
                .map(|((.., began), frame)| frame.saturating_sub(began)),
            stage_frame: reached.stage_frames,
            card: reached.spell,
            after_the_card: the_card_whose_fire_is_up(self, reached),
        }
    }

    /// **The engine's ways where the recipe named none**, for the run's difficulty.
    fn the_engines(&self, reached: &Reached) -> Option<&Ways> {
        self.engine
            .as_ref()
            .filter(|(difficulty, _)| Some(*difficulty) == reached.difficulty)
            .map(|(_, ways)| ways)
    }

    /// **Where the rules are to stand this frame**: the place the recipe wrote for where the run is, the engine's
    /// where it wrote none, and the place written without a card otherwise.
    fn the_place_told(&self, reached: &Reached) -> Option<steer::Standing> {
        let at = self.where_the_run_is(reached);
        self.written
            .the_place(&at)
            .or_else(|| self.the_engines(reached)?.the_place(&at))
            .or(self.standing)
    }
}

/// One step, with its buttons already turned into the word they are and its question already resolved.
enum Step {
    Press {
        word: u16,
        frames: u32,
        /// Tapped until this is reached, rather than held the once: `frames` down, `frames` up, over and over.
        until: Option<Wait>,
        give_up_after: Option<u32>,
    },
    Play {
        until: Wait,
        give_up_after: Option<u32>,
    },
    Wait {
        until: Wait,
        give_up_after: Option<u32>,
    },
    Leave,
    Quit,
    Screenshot {
        named: String,
        frames: u32,
    },
    StartOver,
    TakeHits(hits::Told),
    /// Whether the rules may bomb from here on — see [`recipe::Step::Bombs`].
    Bombs(bool),
    /// Where the rules are told to stand from here on, or freed of it — or, with a card's number, where they
    /// stand while that card runs. See [`recipe::Step::StandAt`].
    StandAt {
        standing: Option<steer::Standing>,
        spell: Option<u32>,
    },
    /// Where the rules stand while a stretch of one stage runs — see [`recipe::Step::StandAtThroughAStretch`].
    StandAtThroughAStretch {
        standing: steer::Standing,
        stage: i32,
        frames: Range<u32>,
    },
    /// The way the rules dodge one card while it runs — see [`recipe::Step::Dodge`].
    Dodge {
        with: recipe::Dodge,
        spell: u32,
    },
    /// The way the rules dodge a stretch of one stage — see [`recipe::Step::DodgeAStretch`].
    DodgeAStretch {
        with: recipe::Dodge,
        stage: i32,
        frames: Range<u32>,
    },
    /// The way the rules dodge the fire after one card — see [`recipe::Step::DodgeAfterACard`].
    DodgeAfterACard {
        with: recipe::Dodge,
        after: u32,
    },
    /// The way the rules dodge a chapter of one stage, or some of its frames — see [`recipe::Step::DodgeAChapter`].
    DodgeAChapter {
        with: recipe::Dodge,
        stage: i32,
        chapter: u32,
        frames: Option<Range<u32>>,
    },
    /// Where the rules stand through a chapter of one stage, or some of its frames — see
    /// [`recipe::Step::StandAtThroughAChapter`].
    StandAtThroughAChapter {
        standing: steer::Standing,
        stage: i32,
        chapter: u32,
        frames: Option<Range<u32>>,
    },
    /// A player's own path the rules go along through one card — see [`recipe::Step::Follow`].
    Follow {
        spell: u32,
        following: steer::Following,
    },
    Answer {
        question: Question,
        item: usize,
        give_up_after: Option<u32>,
    },
    /// The name a run's ranking is registered under — see `recipe::Step::Name`.
    Name {
        typed: String,
        give_up_after: Option<u32>,
    },
}

enum Wait {
    Chapter { stage: i32, chapter: u32 },
    Question(Question),
    Cursor(i32),
    Where { screen: i32, cursor: i32 },
    Screen(i32),
    Boss,
    Dialogue,
    Card,
    CardName,
    Name,
    Frames(u32),
    Spell(u32),
}

/// **Reads the recipe and installs the bot**, or leaves the run to whoever is at the keyboard.
///
/// **A recipe that cannot be read plays nothing**, rather than pressing whatever a half-read file came to: a
/// step naming a button the game has not, or a question orb does not put up, is a recipe written for something
/// else, and carrying out the rest of it would be driving somebody's screens by guesswork.
///
/// Said out loud either way. A launch that plays itself is one nobody is watching, so the log is the only
/// place it can be seen at all.
///
/// # Safety
/// Must run at the attach, before the game has been asked for a frame.
pub unsafe fn install(recipe: &str, game: &dyn Game) {
    let mut playing = PLAYING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *playing = None;
    unsafe { runtime::plays_the_run(None) };
    let Some(read) = read_the_recipe(recipe, game) else {
        return;
    };
    *playing = Some(read);
    unsafe { runtime::plays_the_run(Some(word_to_press)) };
}

/// **The recipe read into what a bot plays to**, or `None` — said in the log — for one it cannot read.
///
/// **Apart from the static it goes in**, which is what lets a test hold one of its own: the bot's own is one
/// per process because the read hook reaches it with nothing to hand it, and two tests in one process sharing
/// it are two tests whose order decides what either reads. Which is what happened —
/// `a_tap_ends_when_the_cursor_reaches_the_item_it_names` failed in a suite that passes it alone, the other
/// test having installed its own recipe over this one's in between.
fn read_the_recipe(recipe: &str, game: &dyn Game) -> Option<Playing> {
    let written = match recipe::read(recipe) {
        Ok(written) => written,
        Err(why) => {
            log!("autoplay: the recipe cannot be read: {why}");
            return None;
        }
    };
    let steps = steps_of(written, game)?;
    log!(
        "autoplay: orb is playing this run itself, to {} step(s)",
        steps.len()
    );
    Some(Playing {
        steps,
        at: 0,
        frames: 0,
        hits: hits::TakingHits::default(),
        bombs: true,
        asked: false,
        in_run_seen: false,
        standing: None,
        written: Ways::default(),
        engine: None,
        chapter_began: None,
        told: None,
        card_before: None,
        dodging: None,
        following: None,
        attempt: 0,
    })
}

/// **A recipe's steps with their buttons turned into the game's word and their questions resolved**, or `None` —
/// said in the log — for a step that names something the game or orb has not.
fn steps_of(written: Vec<recipe::Step>, game: &dyn Game) -> Option<Vec<Step>> {
    let mut steps = Vec::with_capacity(written.len());
    for (index, step) in written.into_iter().enumerate() {
        let at = index + 1;
        let step = match step {
            recipe::Step::Press {
                buttons,
                frames,
                until,
                give_up_after,
            } => {
                let mut word = 0;
                for name in &buttons {
                    let Some(bit) = game.button_bit(name) else {
                        log!(
                            "autoplay: the recipe cannot be read: step {at} presses a button this \
                             game has not, {name}"
                        );
                        return None;
                    };
                    word |= bit;
                }
                let until = match until.map(|until| waiting(until, at)) {
                    Some(None) => return None,
                    held => held.flatten(),
                };
                Step::Press {
                    word,
                    frames,
                    until,
                    give_up_after,
                }
            }
            recipe::Step::Play {
                until,
                give_up_after,
            } => Step::Play {
                until: waiting(until, at)?,
                give_up_after,
            },
            recipe::Step::Wait {
                until,
                give_up_after,
            } => Step::Wait {
                until: waiting(until, at)?,
                give_up_after,
            },
            recipe::Step::Leave => Step::Leave,
            recipe::Step::Quit => Step::Quit,
            recipe::Step::Screenshot { named, frames } => Step::Screenshot { named, frames },
            recipe::Step::StartOver => Step::StartOver,
            recipe::Step::TakeHits {
                chance,
                times_each_retry,
                gives_up,
            } => Step::TakeHits(hits::Told {
                chance,
                times_each_retry,
                gives_up,
            }),
            recipe::Step::Bombs { allowed } => Step::Bombs(allowed),
            recipe::Step::StandAtThroughAStretch {
                x,
                y,
                away_from_the_boss,
                stage,
                frames,
            } => Step::StandAtThroughAStretch {
                standing: steer::Standing {
                    across: the_place_across(x, away_from_the_boss),
                    y,
                },
                stage,
                frames,
            },
            recipe::Step::StandAt {
                x,
                y,
                away_from_the_boss,
                spell,
            } => {
                let across = the_place_across(x, away_from_the_boss);
                Step::StandAt {
                    standing: match (across, y) {
                        (None, None) => None,
                        (across, y) => Some(steer::Standing { across, y }),
                    },
                    spell,
                }
            }
            recipe::Step::Dodge { with, spell } => Step::Dodge { with, spell },
            recipe::Step::DodgeAStretch {
                with,
                stage,
                frames,
            } => Step::DodgeAStretch {
                with,
                stage,
                frames,
            },
            recipe::Step::DodgeAfterACard { with, after } => Step::DodgeAfterACard { with, after },
            recipe::Step::DodgeAChapter {
                with,
                stage,
                chapter,
                frames,
            } => Step::DodgeAChapter {
                with,
                stage,
                chapter,
                frames,
            },
            recipe::Step::StandAtThroughAChapter {
                x,
                y,
                away_from_the_boss,
                stage,
                chapter,
                frames,
            } => Step::StandAtThroughAChapter {
                standing: steer::Standing {
                    across: the_place_across(x, away_from_the_boss),
                    y,
                },
                stage,
                chapter,
                frames,
            },
            recipe::Step::Follow { spell, every, path } => Step::Follow {
                spell,
                following: steer::Following {
                    every,
                    path: path.into(),
                },
            },
            recipe::Step::Answer {
                question,
                item,
                give_up_after,
            } => Step::Answer {
                question: named_question(&question, at)?,
                item,
                give_up_after,
            },
            recipe::Step::Name {
                typed,
                give_up_after,
            } => Step::Name {
                typed,
                give_up_after,
            },
        };
        steps.push(step);
    }
    Some(steps)
}

/// The x a place is written with: an x outright, a distance from the boss's, or the rules' own.
fn the_place_across(x: Option<f32>, away_from_the_boss: Option<f32>) -> Option<steer::Across> {
    match (x, away_from_the_boss) {
        (Some(x), _) => Some(steer::Across::At(x)),
        (None, Some(distance)) => Some(steer::Across::AwayFromTheBoss(distance)),
        (None, None) => None,
    }
}

/// One `until` with its question resolved, or `None` — said in the log — for a name orb does not put up.
fn waiting(until: Until, at: usize) -> Option<Wait> {
    Some(match until {
        Until::Chapter { stage, chapter } => Wait::Chapter { stage, chapter },
        Until::Cursor(item) => Wait::Cursor(item),
        Until::Where { screen, cursor } => Wait::Where { screen, cursor },
        Until::Screen(screen) => Wait::Screen(screen),
        Until::Boss => Wait::Boss,
        Until::Dialogue => Wait::Dialogue,
        Until::Card => Wait::Card,
        Until::CardName => Wait::CardName,
        Until::Name => Wait::Name,
        Until::Frames(frames) => Wait::Frames(frames),
        Until::Spell(spell) => Wait::Spell(spell),
        Until::Question(name) => Wait::Question(named_question(&name, at)?),
    })
}

/// One of orb's own questions by the name the log gives it, or `None` — said in the log — for a name orb does
/// not put up.
///
/// One table for the two places a recipe names one: a step can wait for a question and a step can answer it,
/// and a name that meant one in the first and nothing in the second would be a grammar with two vocabularies.
fn named_question(name: &str, at: usize) -> Option<Question> {
    Some(match name {
        "mode" => Question::Mode,
        "resume" => Question::Resume,
        "retry" => Question::Retry,
        "keep_replay" => Question::KeepReplay,
        "choose_replay" => Question::ChooseReplay,
        _ => {
            log!(
                "autoplay: the recipe cannot be read: step {at} names a question orb does not put \
                 up, {name}"
            );
            return None;
        }
    })
}

/// **Whether the step now being carried out is one that answers orb's retry question itself**, which is what
/// makes the block below stand aside.
///
/// The retry is taken wherever it comes up because a death lands in the middle of a step, and what takes it is
/// a tap of decide — which is the item the cursor is on, the first of them. So a recipe naming any other item
/// was answered `retry: the chapter again on the pad` before its own step saw the question, and the step then
/// sat out its bound: a run asked to start over could be sent back to the chapter it was in and nowhere else,
/// which is what put a check of a stage's own first chapter half an hour of autoplay away.
fn a_step_answers_the_retry(playing: &Playing) -> bool {
    matches!(
        playing.steps.get(playing.at),
        Some(Step::Answer {
            question: Question::Retry,
            ..
        })
    )
}

/// The word the bot presses this frame, which is what the slot in `orb_core::runtime` is filled with.
///
/// **The lock and nothing else**, the run being played standing in a static because the read hook reaches this
/// with nothing to hand it. What it is played *to* is [`the_word`], which takes that run as an argument — so a
/// test holds one of its own rather than the one the last test installed.
fn word_to_press(game: &'static dyn Game, reached: Reached) -> Option<u16> {
    let mut held = PLAYING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    the_word(held.as_mut()?, game, reached)
}

/// **The place the rules are to stand this frame, handed to them where it is not the one they have** — and
/// said as a card's own place takes over or lapses, which is a thing a recipe wrote once and the log would
/// otherwise never show happening.
fn hands_the_rules_their_place(playing: &mut Playing, reached: &Reached) {
    let spell = reached.spell;
    let told = playing.the_place_told(reached);
    if told == playing.told {
        return;
    }
    steer::stand_at(told);
    let a_cards = spell.filter(|up| {
        let named = |ways: &Ways| ways.for_cards.iter().any(|(card, _)| card == up);
        named(&playing.written) || playing.the_engines(reached).is_some_and(named)
    });
    match (a_cards, told) {
        (Some(card), Some(place)) => log!(
            "autoplay: card {card} is up, and the rules stand {} through it",
            where_they_stand(place)
        ),
        (_, Some(place)) => log!("autoplay: the rules stand {}", where_they_stand(place)),
        (_, None) => log!("autoplay: the rules stand where their own errands take them"),
    }
    playing.told = told;
}

/// **The path the rules are to go along through the card that is up, handed to them where it is not the one they
/// have** — and said as it is taken up or put down, the way a card's own way is.
fn hands_the_rules_their_path(playing: &mut Playing, reached: &Reached) {
    let at = playing.where_the_run_is(reached);
    let written = playing
        .written
        .the_path(&at)
        .or_else(|| playing.the_engines(reached)?.the_path(&at));
    let following = written.map(|(card, _)| card);
    if following == playing.following {
        return;
    }
    let path = written.map(|(_, path)| path.clone());
    match (following, &path) {
        (Some(card), Some(path)) => log!(
            "autoplay: card {card} is up, and the rules go along the player's path through it, {} places {} frames \
             apart",
            path.path.len(),
            path.every
        ),
        _ => log!("autoplay: the rules go along no player's path"),
    }
    steer::follow(path);
    playing.following = following;
}

/// **The way the rules are to dodge the card that is up, handed to them where it is not the one they have** —
/// and said as it is taken up or put down, the way a card's own place is.
fn hands_the_rules_their_dodge(playing: &mut Playing, reached: &Reached) {
    let spell = reached.spell;
    if let Some(up) = spell {
        playing.card_before = Some((reached.stage, up));
    }
    let told = the_dodge_told(playing, reached);
    let dodging = told.as_ref().map(|(with, ..)| *with);
    if dodging == playing.dodging {
        return;
    }
    steer::dodge_with(dodging);
    match told {
        Some((with, named, whose)) => log!(
            "autoplay: {}, and the rules dodge it {} as {whose} names",
            match named {
                NamedBy::Card(card) => format!("card {card} is up"),
                NamedBy::AfterTheCard(card) => format!("the fire after card {card} is up"),
                NamedBy::Chapter(stage, chapter) =>
                    format!("stage {stage}'s chapter {chapter} is up"),
                NamedBy::Stretch(stage, frames) => format!(
                    "stage {stage} is between its frames {} and {}",
                    frames.start, frames.end
                ),
            },
            with.name(),
        ),
        None => log!("autoplay: the rules dodge the way their own rules do"),
    }
    playing.dodging = dodging;
}

/// **The way named for where the run is**, what named it and whose naming it is: the recipe's where it named one,
/// and the autoplay engine's for the run's difficulty where it did not.
fn the_dodge_told(
    playing: &Playing,
    reached: &Reached,
) -> Option<(recipe::Dodge, NamedBy, &'static str)> {
    let at = playing.where_the_run_is(reached);
    let written = || {
        playing
            .written
            .the_dodge(&at)
            .map(|(with, named)| (with, named, "the recipe"))
    };
    let engines = || {
        playing
            .the_engines(reached)?
            .the_dodge(&at)
            .map(|(with, named)| (with, named, "the autoplay engine"))
    };
    written().or_else(engines)
}

/// **The card whose fire a boss between two cards is firing**: the last one up, while a boss is up in the same
/// stage and no card is.
fn the_card_whose_fire_is_up(playing: &Playing, reached: &Reached) -> Option<u32> {
    playing
        .card_before
        .filter(|(stage, _)| reached.spell.is_none() && reached.boss && *stage == reached.stage)
        .map(|(_, card)| card)
}

/// **The attempt at its chapter the run is on, handed to the rules where it is not the one they have** — and
/// said where it is a retry, the wait moving being a thing the log would otherwise never show.
fn hands_the_rules_their_attempt(playing: &mut Playing) {
    let attempt = playing.hits.attempt();
    if attempt == playing.attempt {
        return;
    }
    steer::this_attempt(attempt);
    if attempt > 0 {
        log!(
            "autoplay: retry {attempt} of this chapter, and the rules wait {:+.1} to the side of the \
             boss, the middle and the place the card's own dodge puts it",
            steer::aside_on(attempt),
        );
    }
    playing.attempt = attempt;
}

/// Some of a chapter's frames in the words the log says them in, and nothing for the whole of it.
fn the_frames_of_it(frames: &Option<Range<u32>>) -> String {
    match frames {
        Some(frames) => format!(" from its frame {} to {}", frames.start, frames.end),
        None => String::new(),
    }
}

/// A told place in the words the log says it in.
fn where_they_stand(place: steer::Standing) -> String {
    let steer::Standing { across, y } = place;
    format!(
        "{}{}",
        match across {
            Some(steer::Across::At(x)) => format!("at x = {x}"),
            Some(steer::Across::AwayFromTheBoss(it)) => format!("{it} away from the boss"),
            None => "where they were going".to_owned(),
        },
        match y {
            Some(y) => format!(", at y = {y}"),
            None => String::new(),
        },
    )
}

/// What a bot playing `playing` presses this frame.
fn the_word(playing: &mut Playing, game: &'static dyn Game, reached: Reached) -> Option<u16> {
    // **The frame the chapter the run is in began on**, the first one it is seen on, which a way written for some of
    // a chapter's frames counts them from.
    if playing
        .chapter_began
        .map(|(stage, chapter, _)| (stage, chapter))
        != Some((reached.stage, reached.chapter))
    {
        playing.chapter_began = reached
            .stage_frames
            .map(|frame| (reached.stage, reached.chapter, frame));
    }
    // **And the engine's ways for the run's difficulty**, the first frame a run of it is seen on.
    if let Some(difficulty) = reached.difficulty
        && playing.engine.as_ref().map(|(of, _)| *of) != Some(difficulty)
    {
        playing.engine = Some((difficulty, ways::for_a_run_of(game.named(), difficulty)));
    }
    // **A frame of a run being played back is the rules' to see and nothing else's** — see `orb_core`'s
    // `runtime::the_rules_watch`, which asks for it and feeds the run's own word whatever this answers. Not
    // the recipe's: its steps are the pick-up's own, and a step counting these frames would be counting
    // frames of the run that wrote the file. Nor the hits': an attempt counted here is one the run already
    // made, so the rules see its frames as attempt 0's whichever attempt at a chapter the file kept.
    if unsafe { orb_core::resume::landing() }.is_some() {
        hands_the_rules_their_place(playing, &reached);
        hands_the_rules_their_dodge(playing, &reached);
        hands_the_rules_their_path(playing, &reached);
        steer::the_card_has_run(reached.card_frames);
        steer::the_stage_has_run(reached.stage_frames);
        return Some(unsafe { steer::word_to_press(game, hits::Hazards::Dodged, playing.bombs) });
    }
    // **What the rules are to do about being hit, asked before anything else this frame.** It is the chapter
    // the run is in that decides, and every frame of one has to be counted for that: the blocks below hand a
    // frame back without reaching the rules — orb's own offer to retry is one, and that offer going down is
    // the very thing that says a chapter is being played again.
    let hazards = playing.hits.this_frame(reached);
    // **And which attempt at it this is**, counted by the same offer going down.
    hands_the_rules_their_attempt(playing);
    // **And where to stand, which is the card that is up's to say where a recipe gave it a place** — asked
    // every frame for the same reason, a card beginning and ending in the middle of whatever step is on.
    hands_the_rules_their_place(playing, &reached);
    // **And how to dodge it, which is the card's to say where a recipe named it a way** — for the same reason,
    // or the stretch of the stage's where one named that.
    hands_the_rules_their_dodge(playing, &reached);
    // **And the path to go along through it, which is the card's to say where a recipe wrote one.**
    hands_the_rules_their_path(playing, &reached);
    // **And how far into the card they are**, which a way of dodging one card by its own schedule reads.
    steer::the_card_has_run(reached.card_frames);
    // **And the stage's own frame**, which the shot's let-go is timed by.
    steer::the_stage_has_run(reached.stage_frames);
    // **orb's own offer to retry is taken wherever it comes up**, whatever step the recipe is on: Pointdevice mode
    // is a mode whose whole content is that a death is retried, and a death lands in the middle of a step — so
    // no recipe can say when, and a bot that waited for one to say so would stop at its first death. Tapped
    // rather than held, a menu answering the frame a button goes down.
    if reached.question == Some(Question::Retry) && !a_step_answers_the_retry(playing) {
        playing.frames += 1;
        // **And nothing pressed at an offer already answered.** Where the chapter was drawn to be given up
        // at, the answer is asked for outright rather than pressed for — and a press that lands after the
        // menu has gone reaches the front end, which starts a run with it.
        if runtime::giving_the_run_up_at_the_retry() {
            return Some(0);
        }
        return Some(if ((playing.frames - 1) / A_TAP).is_multiple_of(2) {
            game.button_bit("decide").unwrap_or(0)
        } else {
            0
        });
    }
    let Some(step) = playing.steps.get(playing.at) else {
        // Past the last step, which is the run played by the game's own rules for as long as it lasts — and
        // **nothing at all where those have nothing to say**, which is not the same as a word of no buttons:
        // this is the read the game makes on every screen, and a rule with no opinion about the screen that is
        // up should leave the read where it found it. A `wait` step's own silence is a word and stays one.
        let word = unsafe { steer::word_to_press(game, hazards, playing.bombs) };
        return (word != 0).then_some(word);
    };
    playing.frames += 1;
    let (word, over) = match step {
        Step::Press {
            word,
            frames,
            until: None,
            ..
        } => (*word, playing.frames >= *frames),
        // **Tapped**: down for `frames`, up for as many, over and over. The cycle is counted rather than the
        // taps, so a step that ends mid-cycle ends there — what the next step presses is its own business, and
        // a screen already gone is not waiting on this one to let go.
        //
        // **And nothing pressed on the frame a step waiting for something of the game's is over**, which is
        // what stops a tap overshooting the item it was aiming at. A menu moves on the *edge* of a press, so a
        // hold already under way costs nothing more — but a step whose `until` is satisfied on its very first
        // frame presses a fresh edge and walks one item past the target. That is the whole of what a recipe
        // otherwise has to work around: a step aiming at a cursor the list already sat on ended one item
        // further on, and what saved the recipes that did it was only that the item beside the target was
        // harmless.
        //
        // **A count of the step's own frames is not one of those**, and its last frame is pressed: `frames` is
        // how long the step is rather than something that has happened, so a tap of two held to nine frames
        // holds the ninth — see `a_press_with_an_until_is_tapped_until_it_is_reached`.
        Step::Press {
            word,
            frames,
            until: Some(wait),
            ..
        } => {
            let over = waited(wait, reached, playing.frames);
            let letting_go = over && !matches!(wait, Wait::Frames(_));
            (
                if !letting_go && (playing.frames - 1) % (frames * 2) < *frames {
                    *word
                } else {
                    0
                },
                over,
            )
        }
        Step::Wait { until, .. } => (0, waited(until, reached, playing.frames)),
        Step::Play { until, .. } => (
            unsafe { steer::word_to_press(game, hazards, playing.bombs) },
            waited(until, reached, playing.frames),
        ),
        // **The run left for the title, and the step over with the asking.** What a recipe does after it is its
        // own business — the one this was written for closes the game — and there is nothing to wait for here:
        // the game's own half of leaving is a scene it changes on its own time, and a step that waited for it
        // would be a step waiting on a screen this cannot see.
        Step::Leave => {
            let left = unsafe { orb_core::runtime::give_the_run_up(game) };
            log!(
                "autoplay: step {} of {} leaves the run{}",
                playing.at + 1,
                playing.steps.len(),
                if left {
                    ""
                } else {
                    ", and there was none to leave"
                },
            );
            (0, true)
        }
        // **A picture of the frame being drawn**, which is one frame's work and over on the frame it is asked
        // on: what writes it is the draw chain's own end, so nothing here waits for the file.
        //
        // **The frames in front of it are the rules' to play**, and not a word of nothing: what a picture waits
        // out is a thing being drawn, which happens while a run carries on — and a player standing still for
        // half a second inside a card is a player that gets shot. Off a run the rules say nothing anyway.
        Step::Screenshot { named, frames } => {
            let now = playing.frames > *frames;
            if now {
                orb_core::shot::wanted(named);
                log!(
                    "autoplay: step {} of {} asks for a picture of {named}",
                    playing.at + 1,
                    playing.steps.len(),
                );
            }
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                now,
            )
        }
        // **The rules told what to do about being hit, and the step over with the telling.** What it changes is
        // every frame after it rather than this one, so there is nothing to wait for — and the frame it costs
        // goes to the rules, a step that handed the game a word of nothing being a player standing still.
        Step::TakeHits(told) => {
            playing.hits.told(*told);
            log!(
                "autoplay: step {} of {} takes hits, a chance of {:.3} per chapter, {:.3} of that \
                 each retry, and {:.3} of giving the run up at one",
                playing.at + 1,
                playing.steps.len(),
                told.chance,
                told.times_each_retry,
                told.gives_up,
            );
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **The rules told whether they may bomb, and the step over with the telling** — the same terms a
        // `take_hits` is on.
        Step::Bombs(allowed) => {
            playing.bombs = *allowed;
            log!(
                "autoplay: step {} of {} {} the rules the bomb",
                playing.at + 1,
                playing.steps.len(),
                if *allowed { "allows" } else { "forbids" },
            );
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **The rules told where to stand, or freed of it, and the step over with the telling** — on the same
        // terms as a `bombs`: it stands until another says otherwise. A card's place is kept beside the others
        // for the frames that card runs, which the hand-over at the top of this function looks after.
        Step::StandAt { standing, spell } => {
            let (standing, spell) = (*standing, *spell);
            if !playing.written.take(step) {
                playing.standing = standing;
            }
            log!(
                "autoplay: step {} of {} {}{}",
                playing.at + 1,
                playing.steps.len(),
                match standing {
                    Some(place) => format!("tells the rules to stand {}", where_they_stand(place)),
                    None => "frees the rules of the place they were told to stand".to_owned(),
                },
                match spell {
                    Some(card) => format!(" while card {card} runs"),
                    None => String::new(),
                },
            );
            // Handed over on this frame and not the next, the step's own line having said what it is.
            let told = playing.the_place_told(&reached);
            steer::stand_at(told);
            playing.told = told;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **And a place told for a stretch of a stage**, kept for the frames of it the way a card's is.
        Step::StandAtThroughAStretch {
            standing,
            stage,
            frames,
        } => {
            let (standing, stage, frames) = (*standing, *stage, frames.clone());
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} tells the rules to stand {} from stage {stage}'s frame {} to {}",
                playing.at + 1,
                playing.steps.len(),
                where_they_stand(standing),
                frames.start,
                frames.end,
            );
            // Handed over on this frame, for the reason a card's place is.
            let told = playing.the_place_told(&reached);
            steer::stand_at(told);
            playing.told = told;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **A way named for a card, and the step over with the naming** — kept for the frames that card runs,
        // which the hand-over at the top of this function looks after the way it does a card's own place.
        Step::Dodge { with, spell } => {
            let (with, spell) = (*with, *spell);
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} has the rules dodge card {spell} {}",
                playing.at + 1,
                playing.steps.len(),
                with.name(),
            );
            // Handed over on this frame and not the next, the step's own line having said what it is.
            let dodging = reached
                .spell
                .filter(|up| *up == spell)
                .map(|_| with)
                .or(playing.dodging);
            steer::dodge_with(dodging);
            playing.dodging = dodging;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **And a way named for a stretch of a stage**, kept the same way for the frames of it.
        Step::DodgeAStretch {
            with,
            stage,
            frames,
        } => {
            let (with, stage, frames) = (*with, *stage, frames.clone());
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} has the rules dodge stage {stage} from its frame {} to {} {}",
                playing.at + 1,
                playing.steps.len(),
                frames.start,
                frames.end,
                with.name(),
            );
            // Handed over on this frame, for the reason the card's is.
            let dodging = reached
                .stage_frames
                .filter(|frame| reached.stage == stage && frames.contains(frame))
                .map(|_| with)
                .or(playing.dodging);
            steer::dodge_with(dodging);
            playing.dodging = dodging;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **And a way named for the fire after a card**, kept the same way for the frames between that card and
        // the next.
        Step::DodgeAfterACard { with, after } => {
            let (with, after) = (*with, *after);
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} has the rules dodge the fire after card {after} {}",
                playing.at + 1,
                playing.steps.len(),
                with.name(),
            );
            // Handed over on this frame, for the reason the card's is.
            let dodging = the_dodge_told(playing, &reached)
                .map(|(with, ..)| with)
                .or(playing.dodging);
            steer::dodge_with(dodging);
            playing.dodging = dodging;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **And a way named for a chapter of a stage**, kept the same way for that chapter's frames.
        Step::DodgeAChapter {
            with,
            stage,
            chapter,
            frames,
        } => {
            let (with, stage, chapter, frames) = (*with, *stage, *chapter, frames.clone());
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} has the rules dodge stage {stage}'s chapter {chapter}{} {}",
                playing.at + 1,
                playing.steps.len(),
                the_frames_of_it(&frames),
                with.name(),
            );
            // Handed over on this frame, for the reason the card's is.
            let dodging = the_dodge_told(playing, &reached)
                .map(|(with, ..)| with)
                .or(playing.dodging);
            steer::dodge_with(dodging);
            playing.dodging = dodging;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **And a place told for a chapter of a stage**, kept the same way.
        Step::StandAtThroughAChapter {
            standing,
            stage,
            chapter,
            frames,
        } => {
            let (standing, stage, chapter, frames) = (*standing, *stage, *chapter, frames.clone());
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} tells the rules to stand {} through stage {stage}'s chapter {chapter}{}",
                playing.at + 1,
                playing.steps.len(),
                where_they_stand(standing),
                the_frames_of_it(&frames),
            );
            // Handed over on this frame, for the reason a card's place is.
            let told = playing.the_place_told(&reached);
            steer::stand_at(told);
            playing.told = told;
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **And a player's path written for a card**, kept the same way for the frames that card runs.
        Step::Follow { spell, following } => {
            let spell = *spell;
            playing.written.take(step);
            log!(
                "autoplay: step {} of {} has the rules go along a player's path through card {spell}",
                playing.at + 1,
                playing.steps.len(),
            );
            // Handed over on this frame, for the reason a card's way is.
            if reached.spell == Some(spell) {
                steer::follow(Some(following.clone()));
                playing.following = Some(spell);
            }
            (
                unsafe { steer::word_to_press(game, hazards, playing.bombs) },
                true,
            )
        }
        // **The run asked to start over, and the step over with the asking.** What raises the question is the
        // frame hook, on this frame or the first one a stage can be retried from, so there is nothing here to
        // wait for — the step after it presses at the question.
        Step::StartOver => {
            orb_core::runtime::start_the_run_over();
            log!(
                "autoplay: step {} of {} asks the run to start over",
                playing.at + 1,
                playing.steps.len(),
            );
            (0, true)
        }
        // **An item of one of orb's own questions, asked for outright.** The step waits for that question to
        // be up — nothing else raises one — and ends once the answer has been taken, which is the frame the
        // block that answers it runs. Nothing is pressed while it waits: a press at one of these takes the
        // item the cursor is on, which is the item this step exists to say is not the one meant.
        Step::Answer { question, item, .. } => {
            let up = reached.question == Some(*question);
            if up && !playing.asked && !runtime::answering_a_question() {
                runtime::answer_the_question(*item);
                playing.asked = true;
                log!(
                    "autoplay: step {} of {} answers with item {item}",
                    playing.at + 1,
                    playing.steps.len(),
                );
            }
            // **Over once the answer it asked for has been taken**, which is the item no longer standing
            // where the blocks that answer these read it from — and never before it has asked at all: a
            // question that has not come up is one to wait for, and the two read as the same frame without
            // the flag. Asking twice would be answered by whatever screen came after, so the flag holds that
            // too.
            //
            // **And not the question having gone**, which is the same thing for every question but these
            // lists: the four a replay is chosen out of are one question under one name, so a step that
            // waited for `choose_replay` to go would sit out its whole bound with the next list already up —
            // ten seconds a list, three times over, on the launch that measured it.
            (0, playing.asked && !runtime::answering_a_question())
        }
        // **The name a run's ranking is registered under, written into the game's own entry.** Nothing is
        // pressed while it waits for that entry: a press at one writes whatever character its cursor is on,
        // which is the `AAAAAAAA` this step exists instead of — see `Game::types_its_name`.
        Step::Name { typed, .. } => {
            let written = unsafe { game.types_its_name(typed) };
            if written {
                // **And what the entry holds after it, read back through the game's own field.** A write
                // that answered true and a ranking that came out `AAAAAAAA` are the same line without
                // this, and they are different repairs: one is the field not being where it was read to
                // be, the other the ranking taking its name from somewhere the entry is not.
                log!(
                    "autoplay: step {} of {} types {typed} into the name the game asks for, which now \
                     holds {}",
                    playing.at + 1,
                    playing.steps.len(),
                    match unsafe { game.the_name_its_own_entry_holds() } {
                        Some(held) => format!("{held:?}"),
                        None => "nothing this game reads back".to_owned(),
                    },
                );
            }
            (0, written)
        }
        // Nothing pressed for it and nothing after it: the step is left in front so that the log says which
        // one the launch stopped on, and the game is asked to close.
        Step::Quit => {
            log!(
                "autoplay: step {} of {} closes the game",
                playing.at + 1,
                playing.steps.len()
            );
            orb_core::runtime::close_the_game();
            // Nothing after that on a real host, the process being gone. A laid-out one answers the call and
            // returns, so the step stays in front and nothing is pressed for it — which is what a test can
            // hold the log line against.
            (0, false)
        }
    };
    // **Or the step's bound reached, which is what stops a recipe holding a launch for ever**: a step waiting
    // for something that will not happen ends here rather than never, and the log says which of the two ended
    // it — `done` and `gave up` being a recipe that worked and one that did not.
    //
    // **And a `play` gives up the moment the run is over**, whatever its bound: there is nothing left to play,
    // the game is on its own screen for it, and the rules press nothing there — so a step waiting for a stage
    // would sit out its whole bound. An attempt that failed ends and lets the next one start.
    playing.in_run_seen |= !reached.over;
    let gave_up = !over
        && match step {
            // An `answer` waits for its question the way a `wait_until` for one does, so it takes a bound the
            // same way: a question that never comes up is a step that would never end.
            // And a `name` waits for the game's own entry, which a run that ended some other way never
            // reaches.
            Step::Press { give_up_after, .. }
            | Step::Wait { give_up_after, .. }
            | Step::Answer { give_up_after, .. }
            | Step::Name { give_up_after, .. } => {
                give_up_after.is_some_and(|bound| playing.frames >= bound)
            }
            Step::Play { give_up_after, .. } => {
                reached.over && playing.in_run_seen
                    || give_up_after.is_some_and(|bound| playing.frames >= bound)
            }
            // None of these can wait for something that will not happen, so none has a bound to reach: each is
            // an ask — the run ended, the process closed, a frame written out, the run started over, the rules
            // told what to do about being hit — and a picture's own frames are a count it reaches by counting.
            Step::Leave
            | Step::Quit
            | Step::Screenshot { .. }
            | Step::StartOver
            | Step::TakeHits(..)
            | Step::Bombs(..)
            | Step::StandAt { .. }
            | Step::StandAtThroughAStretch { .. }
            | Step::Dodge { .. }
            | Step::DodgeAStretch { .. }
            | Step::DodgeAfterACard { .. }
            | Step::DodgeAChapter { .. }
            | Step::StandAtThroughAChapter { .. }
            | Step::Follow { .. } => false,
        };
    if over || gave_up {
        playing.at += 1;
        playing.frames = 0;
        playing.asked = false;
        playing.in_run_seen = false;
        log!(
            "autoplay: step {} of {} {}",
            playing.at,
            playing.steps.len(),
            if gave_up { "gave up" } else { "done" }
        );
    } else if playing.frames.is_multiple_of(STILL_WAITING) {
        // **Where it has got to, for a launch nobody is watching.** A step that never ends is the shape every
        // stall so far has had, and until this said so each of them was found by somebody looking at the screen.
        log!(
            "autoplay: step {} of {} is still waiting, {} frame(s) in",
            playing.at + 1,
            playing.steps.len(),
            playing.frames,
        );
    }
    Some(word)
}

/// Whether what a step was waiting for has been reached.
fn waited(wait: &Wait, reached: Reached, frames: u32) -> bool {
    match wait {
        // Past as well as at, because a chapter is only the run's for as long as it is in it: a boundary whose
        // frame fell while the step was being carried out came and went.
        Wait::Chapter { stage, chapter } => (reached.stage, reached.chapter) >= (*stage, *chapter),
        Wait::Question(question) => reached.question == Some(*question),
        // **At and not past**, a cursor being a thing that goes round: a list walked one item at a time reaches
        // every item on the way, and a step that ended at "past" would end on the frame the screen came up.
        Wait::Cursor(item) => reached.cursor == Some(*item),
        // **The screen as well as the item**, the cursor being one field whatever list is up: waiting for a
        // number alone waits for whichever screen reaches it first, which put a run on Lunatic.
        Wait::Where { screen, cursor } => {
            reached.screen == Some(*screen) && reached.cursor == Some(*cursor)
        }
        // **And the screen on its own, for a step waiting to press at it** — see `recipe::Until::Screen`.
        Wait::Screen(screen) => reached.screen == Some(*screen),
        Wait::Boss => reached.boss,
        Wait::Dialogue => reached.dialogue,
        Wait::Card => reached.card,
        Wait::CardName => reached.card_name,
        Wait::Name => reached.typing_a_name,
        Wait::Frames(wanted) => frames >= *wanted,
        Wait::Spell(spell) => reached.spell == Some(*spell),
    }
}

#[cfg(test)]
mod tests {
    use super::{A_TAP, Playing, read_the_recipe, steer, the_word};
    use orb_core::game::{Game, Question, Reached, th07::Th07};

    /// The game these are read against, which is th07 for the reason every recipe test uses it: what a button
    /// name *is* is the game's, and this is the game whose bot the recipes were written for.
    const GAME: &Th07 = &Th07;

    /// **A run of this recipe's own, apart from the static a launch keeps its bot in.** Two tests in one
    /// process installing into that static are two tests whose order decides what either reads, and a suite
    /// that passed each of these alone failed one of them beside the other.
    ///
    /// # Panics
    /// Where the recipe cannot be read, which is a test whose own recipe is wrong.
    fn playing_to(recipe: &str) -> Playing {
        read_the_recipe(recipe, GAME).expect("a recipe this test wrote")
    }

    /// Where the run has got to, with a question of orb's own up or none.
    fn reached(question: Option<Question>) -> Reached {
        Reached {
            stage: 1,
            chapter: 1,
            question,
            cursor: None,
            screen: None,
            over: false,
            boss: false,
            dialogue: false,
            card: false,
            card_name: false,
            typing_a_name: false,
            spell: None,
            card_frames: None,
            stage_frames: None,
            difficulty: None,
        }
    }

    /// **A card's own place stands while that card runs and the place written without one stands otherwise** —
    /// through another card too, which is what makes a place told for one card no place of any other's.
    #[test]
    fn a_cards_place_stands_through_that_card_alone() {
        let mut playing = playing_to("- quit: true\n");
        let everywhere = steer::Standing {
            across: Some(steer::Across::At(40.0)),
            y: None,
        };
        let through_the_card = steer::Standing {
            across: Some(steer::Across::At(192.0)),
            y: Some(260.0),
        };
        playing.standing = Some(everywhere);
        playing.written.for_cards.push((105, through_the_card));
        let up = |spell| Reached {
            spell,
            ..reached(None)
        };

        assert_eq!(playing.the_place_told(&up(None)), Some(everywhere));
        assert_eq!(
            playing.the_place_told(&up(Some(105))),
            Some(through_the_card)
        );
        assert_eq!(playing.the_place_told(&up(Some(94))), Some(everywhere));
        playing.standing = None;
        assert_eq!(playing.the_place_told(&up(Some(94))), None);
    }

    /// **And a stretch's own place stands while the stage's frame is inside it**, in that stage alone, a card's
    /// own place standing over it and the place written without either standing otherwise.
    #[test]
    fn a_stretchs_place_stands_through_that_stretch_alone() {
        let mut playing = playing_to("- quit: true\n");
        let everywhere = steer::Standing {
            across: Some(steer::Across::At(40.0)),
            y: None,
        };
        let through_the_stretch = steer::Standing {
            across: Some(steer::Across::AwayFromTheBoss(96.0)),
            y: Some(300.0),
        };
        let through_the_card = steer::Standing {
            across: Some(steer::Across::At(192.0)),
            y: Some(260.0),
        };
        playing.standing = Some(everywhere);
        playing
            .written
            .for_stretches
            .push((4, 3600..4660, through_the_stretch));
        playing.written.for_cards.push((105, through_the_card));
        let at = |stage, frame, spell| Reached {
            stage,
            stage_frames: Some(frame),
            spell,
            ..reached(None)
        };

        assert_eq!(playing.the_place_told(&at(4, 3599, None)), Some(everywhere));
        assert_eq!(
            playing.the_place_told(&at(4, 3600, None)),
            Some(through_the_stretch)
        );
        assert_eq!(
            playing.the_place_told(&at(4, 4659, None)),
            Some(through_the_stretch)
        );
        assert_eq!(playing.the_place_told(&at(4, 4660, None)), Some(everywhere));
        assert_eq!(playing.the_place_told(&at(3, 4000, None)), Some(everywhere));
        assert_eq!(
            playing.the_place_told(&at(4, 4000, Some(105))),
            Some(through_the_card)
        );
    }

    /// **A step waiting for a card by its number waits through every other card**, and ends the frame that one
    /// is up — which is what makes a told place bracketed by it the card's and nobody else's. A `wait_until`
    /// and not a `play_until`, the two reading the condition alike and only the second asking the rules for
    /// a word about a game this test has not laid out.
    #[test]
    fn a_step_for_a_card_by_its_number_waits_through_the_others() {
        let mut playing = playing_to("- wait_until: { spell: 94 }\n- quit: true\n");
        let another = Reached {
            card: true,
            spell: Some(90),
            ..reached(None)
        };
        the_word(&mut playing, GAME, another);
        assert_eq!(playing.at, 0, "the step ended on a card it does not name");
        let named = Reached {
            card: true,
            spell: Some(94),
            ..reached(None)
        };
        the_word(&mut playing, GAME, named);
        assert_eq!(playing.at, 1, "the step did not end on the card it names");
    }

    /// **A tap goes on until the game's own cursor is on the item the step names**, and stops the frame it is.
    ///
    /// Which is the whole of how stage practice is entered: the title menu's third item is Practice Start and
    /// each screen after it is another list, and a step that tapped a fixed number of times would be walking
    /// somebody else's unlocks.
    #[test]
    fn a_tap_ends_when_the_cursor_reaches_the_item_it_names() {
        let mut playing = playing_to("- press: [down]\n  frames: 1\n  until: { cursor: 2 }\n");
        let down = GAME.button_bit("down").expect("th07 moves a cursor down");

        let mut on = Reached {
            cursor: Some(0),
            ..reached(None)
        };
        assert_eq!(
            the_word(&mut playing, GAME, on),
            Some(down),
            "the step did not tap towards the item it names",
        );

        // **The tap gone and not the word gone.** Past its last step a recipe hands the frame to the rules, and
        // what those answer is a question about a game that is not running here — so what says the step ended is
        // that nothing is being pressed towards the item any more.
        on.cursor = Some(2);
        assert_eq!(
            the_word(&mut playing, GAME, on).unwrap_or(0) & down,
            0,
            "the step went on tapping past the item it names",
        );
    }

    /// **orb's own offer to retry is taken wherever it comes up**, whatever step the recipe is on.
    ///
    /// Pointdevice mode is a mode whose whole content is that a death is retried, and a launch that sat on that
    /// menu stopped at its first death — waiting out its step's own bound on the screen it died on. It cannot be
    /// a step, because a death lands in the middle of one and no recipe can say when.
    ///
    /// **Tapped and not held**, a menu answering the frame a button goes down: held, it walks one screen of the
    /// menu and then holds it.
    #[test]
    fn orbs_own_offer_to_retry_is_taken_wherever_it_comes_up() {
        // A recipe of no steps, so that nothing but the offer decides what is pressed.
        let mut playing = playing_to("[]");
        let decide = GAME
            .button_bit("decide")
            .expect("th07 decides with something");

        let mut pressed = Vec::new();
        for _ in 0..A_TAP * 2 {
            pressed.push(the_word(&mut playing, GAME, reached(Some(Question::Retry))));
        }
        assert_eq!(
            pressed,
            [
                vec![Some(decide); A_TAP as usize],
                vec![Some(0); A_TAP as usize]
            ]
            .concat(),
            "the offer was not tapped for {A_TAP} frames and let go for as many",
        );
    }
    // What is pressed with no offer up is the run played by the game's own rules, which reads where the game
    // keeps its numbers — so it is read against a laid-out game rather than here: `orb-e2e`'s
    // `the_bot_playing_th07`.
}
