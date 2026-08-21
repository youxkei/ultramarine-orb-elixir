//! **The harness that plays the game, so that a run to be picked up out of a file costs nobody an
//! afternoon.**
//!
//! Every measurement of a resume has cost somebody a stage of 妖々夢 and another every time the file grew a
//! field. A run the machine plays is a run to be had again on demand, and the same run every launch — which is
//! what makes two launches comparable at all.
//!
//! **It plays the way somebody does.** No invulnerability and no keys sent from outside: the word goes in
//! where a resumed run's own recorded words go in, and what it is made of is the game's own numbers read where
//! the game keeps them. Every record a played run leaves is left, the score file and the ranking included —
//! a run written down differently is not the thing a resume is measured against.
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
//! - play_until: { stage: 2, chapter: 3 }    # play by the rules as far as stage 2's midboss
//! - quit: true                             # closed in the middle, which is what leaves a run to pick up
//! ```
//!
//! **What each name and each rule is, is the game's** — `Game::button_bit` and `Game::word_to_press`. A shared
//! bot could only be the union of every game's vocabulary, and 東方 is not one kind of game to take a union
//! of.

use std::sync::Mutex;

use orb_config::recipe::{self, Until};
use orb_core::game::{Game, Question, Reached};
use orb_core::log;
use orb_core::runtime;

/// The recipe this launch is playing to and where it has got to, or `None` for a recipe that could not be
/// read.
///
/// A `Mutex` because what reads it is the game's own input read, on the game's main thread, and what writes it
/// is [`install`] at the attach — which is not that thread on every launch. Uncontended either way, one lock a
/// frame being nothing beside a frame.
static PLAYING: Mutex<Option<Playing>> = Mutex::new(None);

struct Playing {
    steps: Vec<Step>,
    /// Which step is being carried out, and how many frames of it have gone by. A step is finished by moving
    /// this on rather than by taking it out of the list, so that a log line can say which of how many.
    at: usize,
    frames: u32,
}

/// One step, with its buttons already turned into the word they are and its question already resolved.
enum Step {
    Press {
        word: u16,
        frames: u32,
        /// Tapped until this is reached, rather than held the once: `frames` down, `frames` up, over and over.
        until: Option<Wait>,
    },
    Play(Wait),
    Wait(Wait),
    Quit,
}

enum Wait {
    Chapter { stage: i32, chapter: u32 },
    Question(Question),
    Frames(u32),
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
    let written = match recipe::read(recipe) {
        Ok(written) => written,
        Err(why) => return log!("autoplay: the recipe cannot be read: {why}"),
    };
    let mut steps = Vec::with_capacity(written.len());
    for (index, step) in written.into_iter().enumerate() {
        let at = index + 1;
        let step = match step {
            recipe::Step::Press {
                buttons,
                frames,
                until,
            } => {
                let mut word = 0;
                for name in &buttons {
                    let Some(bit) = game.button_bit(name) else {
                        return log!(
                            "autoplay: the recipe cannot be read: step {at} presses a button this \
                             game has not, {name}"
                        );
                    };
                    word |= bit;
                }
                let until = match until.map(|until| waiting(until, at)) {
                    Some(None) => return,
                    held => held.flatten(),
                };
                Step::Press {
                    word,
                    frames,
                    until,
                }
            }
            recipe::Step::Play(until) => match waiting(until, at) {
                Some(wait) => Step::Play(wait),
                None => return,
            },
            recipe::Step::Wait(until) => match waiting(until, at) {
                Some(wait) => Step::Wait(wait),
                None => return,
            },
            recipe::Step::Quit => Step::Quit,
        };
        steps.push(step);
    }
    log!(
        "autoplay: orb is playing this run itself, to {} step(s)",
        steps.len()
    );
    *playing = Some(Playing {
        steps,
        at: 0,
        frames: 0,
    });
    unsafe { runtime::plays_the_run(Some(word_to_press)) };
}

/// One `until` with its question resolved, or `None` — said in the log — for a name orb does not put up.
fn waiting(until: Until, at: usize) -> Option<Wait> {
    Some(match until {
        Until::Chapter { stage, chapter } => Wait::Chapter { stage, chapter },
        Until::Frames(frames) => Wait::Frames(frames),
        Until::Question(name) => Wait::Question(match name.as_str() {
            "mode" => Question::Mode,
            "resume" => Question::Resume,
            _ => {
                log!(
                    "autoplay: the recipe cannot be read: step {at} waits for a question orb does \
                     not put up, {name}"
                );
                return None;
            }
        }),
    })
}

/// The word the bot presses this frame, which is what the slot in `orb_core::runtime` is filled with.
fn word_to_press(game: &'static dyn Game, reached: Reached) -> Option<u16> {
    let mut held = PLAYING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let playing = held.as_mut()?;
    let Some(step) = playing.steps.get(playing.at) else {
        // Past the last step, which is the run played by the game's own rules for as long as it lasts.
        return Some(unsafe { game.word_to_press() });
    };
    playing.frames += 1;
    let (word, over) = match step {
        Step::Press {
            word,
            frames,
            until: None,
        } => (*word, playing.frames >= *frames),
        // **Tapped**: down for `frames`, up for as many, over and over. The cycle is counted rather than the
        // taps, so a step that ends mid-cycle ends there — what the next step presses is its own business, and
        // a screen already gone is not waiting on this one to let go.
        Step::Press {
            word,
            frames,
            until: Some(wait),
        } => (
            if (playing.frames - 1) % (frames * 2) < *frames {
                *word
            } else {
                0
            },
            waited(wait, reached, playing.frames),
        ),
        Step::Wait(wait) => (0, waited(wait, reached, playing.frames)),
        Step::Play(wait) => (
            unsafe { game.word_to_press() },
            waited(wait, reached, playing.frames),
        ),
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
    if over {
        playing.at += 1;
        playing.frames = 0;
        log!(
            "autoplay: step {} of {} done",
            playing.at,
            playing.steps.len()
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
        Wait::Frames(wanted) => frames >= *wanted,
    }
}
