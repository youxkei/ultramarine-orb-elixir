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
//!   give_up_after: 3600
//! - play_until: { stage: 2, chapter: 3 }    # play by the rules as far as stage 2's midboss
//!   give_up_after: 36000                    # or close, a run that died leaving a menu the rules do not press
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
    Quit,
}

enum Wait {
    Chapter { stage: i32, chapter: u32 },
    Question(Question),
    Cursor(i32),
    Where { screen: i32, cursor: i32 },
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
                give_up_after,
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
                    give_up_after,
                }
            }
            recipe::Step::Play {
                until,
                give_up_after,
            } => match waiting(until, at) {
                Some(until) => Step::Play {
                    until,
                    give_up_after,
                },
                None => return,
            },
            recipe::Step::Wait {
                until,
                give_up_after,
            } => match waiting(until, at) {
                Some(until) => Step::Wait {
                    until,
                    give_up_after,
                },
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
        Until::Cursor(item) => Wait::Cursor(item),
        Until::Where { screen, cursor } => Wait::Where { screen, cursor },
        Until::Frames(frames) => Wait::Frames(frames),
        Until::Question(name) => Wait::Question(match name.as_str() {
            "mode" => Question::Mode,
            "resume" => Question::Resume,
            "retry" => Question::Retry,
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
    // **orb's own offer to retry is taken wherever it comes up**, whatever step the recipe is on: 完全無欠モード
    // is a mode whose whole content is that a death is retried, and a death lands in the middle of a step — so
    // no recipe can say when, and a bot that waited for one to say so would stop at its first death. Tapped
    // rather than held, a menu answering the frame a button goes down.
    if reached.question == Some(Question::Retry) {
        playing.frames += 1;
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
        let word = unsafe { game.word_to_press() };
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
        Step::Press {
            word,
            frames,
            until: Some(wait),
            ..
        } => (
            if (playing.frames - 1) % (frames * 2) < *frames {
                *word
            } else {
                0
            },
            waited(wait, reached, playing.frames),
        ),
        Step::Wait { until, .. } => (0, waited(until, reached, playing.frames)),
        Step::Play { until, .. } => (
            unsafe { game.word_to_press() },
            waited(until, reached, playing.frames),
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
    // **Or the step's bound reached, which is what stops a recipe holding a launch for ever**: a step waiting
    // for something that will not happen ends here rather than never, and the log says which of the two ended
    // it — `done` and `gave up` being a recipe that worked and one that did not.
    //
    // **And a `play` gives up the moment the run is over**, whatever its bound: there is nothing left to play,
    // the game is on its own screen for it, and the rules press nothing there — so a step waiting for a stage
    // would sit out its whole bound. An attempt that failed ends and lets the next one start.
    let gave_up = !over
        && match step {
            Step::Press { give_up_after, .. } | Step::Wait { give_up_after, .. } => {
                give_up_after.is_some_and(|bound| playing.frames >= bound)
            }
            Step::Play { give_up_after, .. } => {
                reached.over || give_up_after.is_some_and(|bound| playing.frames >= bound)
            }
            Step::Quit => false,
        };
    if over || gave_up {
        playing.at += 1;
        playing.frames = 0;
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
        Wait::Frames(wanted) => frames >= *wanted,
    }
}

#[cfg(test)]
mod tests {
    use super::{A_TAP, install, word_to_press};
    use orb_core::game::{Game, Question, Reached, th07::Th07};

    /// The game these are read against, which is 妖々夢 for the reason every recipe test uses it: what a button
    /// name *is* is the game's, and this is the game whose bot the recipes were written for.
    const GAME: &Th07 = &Th07;

    /// Where the run has got to, with a question of orb's own up or none.
    fn reached(question: Option<Question>) -> Reached {
        Reached {
            stage: 1,
            chapter: 1,
            question,
            cursor: None,
            screen: None,
            over: false,
        }
    }

    /// **A tap goes on until the game's own cursor is on the item the step names**, and stops the frame it is.
    ///
    /// Which is the whole of how 面別プラクティス is entered: the title menu's third item is Practice Start and
    /// each screen after it is another list, and a step that tapped a fixed number of times would be walking
    /// somebody else's unlocks.
    #[test]
    fn a_tap_ends_when_the_cursor_reaches_the_item_it_names() {
        unsafe {
            install(
                "- press: [down]\n  frames: 1\n  until: { cursor: 2 }\n",
                GAME,
            )
        };
        let down = GAME.button_bit("down").expect("妖々夢 moves a cursor down");

        let mut on = Reached {
            cursor: Some(0),
            ..reached(None)
        };
        assert_eq!(
            word_to_press(GAME, on),
            Some(down),
            "the step did not tap towards the item it names",
        );

        // **The tap gone and not the word gone.** Past its last step a recipe hands the frame to the rules, and
        // what those answer is a question about a game that is not running here — so what says the step ended is
        // that nothing is being pressed towards the item any more.
        on.cursor = Some(2);
        assert_eq!(
            word_to_press(GAME, on).unwrap_or(0) & down,
            0,
            "the step went on tapping past the item it names",
        );
    }

    /// **orb's own offer to retry is taken wherever it comes up**, whatever step the recipe is on.
    ///
    /// 完全無欠モード is a mode whose whole content is that a death is retried, and a launch that sat on that
    /// menu stopped at its first death — waiting out its step's own bound on the screen it died on. It cannot be
    /// a step, because a death lands in the middle of one and no recipe can say when.
    ///
    /// **Tapped and not held**, a menu answering the frame a button goes down: held, it walks one screen of the
    /// menu and then holds it.
    #[test]
    fn orbs_own_offer_to_retry_is_taken_wherever_it_comes_up() {
        // A recipe of no steps, so that nothing but the offer decides what is pressed.
        unsafe { install("[]", GAME) };
        let decide = GAME
            .button_bit("decide")
            .expect("妖々夢 decides with something");

        let mut pressed = Vec::new();
        for _ in 0..A_TAP * 2 {
            pressed.push(word_to_press(GAME, reached(Some(Question::Retry))));
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
