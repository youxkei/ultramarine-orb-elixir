//! **The grammar of a recipe orb plays a run to**: a list of steps carried out in order.
//!
//! ```yaml
//! - press: { buttons: [decide], frames: 2 }
//! - play: { until: { stage: 2, chapter: 3 } }
//! - press: { buttons: [cancel], frames: 2 }
//! - wait: { until: { question: resume } }
//! - quit
//! ```
//!
//! **Four steps, and the difference between two of them is the whole reason for a recipe.** A `press` holds a
//! word for a number of frames, which is what a screen takes: a screen answers the same way however long the
//! run before it took. A `play` hands the frames to the game's own rules until something is *reached*, which
//! is what a stage takes: how many frames stage 1 costs is not a number anybody can write down, so a recipe
//! that counted them would be a recipe for one run of one afternoon.
//!
//! `wait` is a `press` of nothing with an `until` — the frames a screen takes to come up are as unwritable as
//! a stage's. `quit` closes the game, which is the last step of a recipe nobody is watching.
//!
//! **The grammar is here and the names are the game's.** A list of button names, a stage and a chapter and a
//! question is the same list whatever game reads it, so it is read once — and what `decide` *is*, and which
//! chapter a stage's midboss falls in, differ from game to game, so nothing here resolves either.
//! `game::th07::bot` is where 妖々夢's are, and a name that game has nothing for is a recipe it refuses.
//!
//! Here rather than in `orb-core` because this is where the YAML reader is: `orb.yaml` goes through the same
//! one, and a second copy of it in the DLL for a diagnostic would be weight in what ships.

use serde::Deserialize;

/// One step of a recipe.
/// One step of a recipe, as [`read`] hands it over: one of four things and never two.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// A word held for a number of frames, which is what a screen takes.
    Press { buttons: Vec<String>, frames: u32 },
    /// The game played by its own rules until something is reached.
    Play(Until),
    /// And nothing held until something is reached, which is what waiting for a screen to come up is.
    Wait(Until),
    /// The game closed, which is a recipe nobody is watching having finished.
    Quit,
}

/// What a `play` or a `wait` is waiting for: one of these and never two.
///
/// One at a time because a step waits for one thing and the next step waits for the next, which is what puts
/// them in an order. What each of them *means* is the game's — see `Game::settle_the_recipe`.
#[derive(Debug, PartialEq, Eq)]
pub enum Until {
    /// A chapter of a stage being the one the run is in, both counted the way the log counts them.
    Chapter { stage: i32, chapter: u32 },
    /// One of orb's own questions being up, by the name the log gives it.
    Question(String),
    /// Or that many frames having gone by.
    Frames(u32),
}

/// A step as it is *written*, which is a map whose keys say which of the four it is.
///
/// **Loose here and strict out of [`read`]**, rather than an enum the reader could take straight: an
/// externally tagged enum is a YAML *tag* to `serde_yaml_ng` — `- !press {…}` — and a recipe is written by
/// hand, so the file gets the plain keys and this pays for them by being checked once.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    press: Option<Vec<String>>,
    frames: Option<u32>,
    play_until: Option<WrittenUntil>,
    wait_until: Option<WrittenUntil>,
    quit: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenUntil {
    stage: Option<i32>,
    chapter: Option<u32>,
    question: Option<String>,
    frames: Option<u32>,
}

impl WrittenUntil {
    fn read(&self, at: usize) -> Result<Until, String> {
        match (self.stage, self.chapter, &self.question, self.frames) {
            (Some(stage), Some(chapter), None, None) => Ok(Until::Chapter { stage, chapter }),
            (None, None, Some(question), None) => Ok(Until::Question(question.clone())),
            (None, None, None, Some(0)) => Err(format!("step {at} is over before it starts")),
            (None, None, None, Some(frames)) => Ok(Until::Frames(frames)),
            _ => Err(format!(
                "step {at} waits for a stage and a chapter, a question, or a count of frames, and \
                 for one of the three"
            )),
        }
    }
}

/// Reads one, or says what is wrong with it.
///
/// A whole-list parse and `deny_unknown_fields` throughout, for the resume file's reason: a key nothing knows
/// is a recipe written for something else, and carrying out half of one is worse than carrying out none. And
/// every step checked before any is handed back, for the same reason.
pub fn read(text: &str) -> Result<Vec<Step>, String> {
    let written: Vec<Written> = serde_yaml_ng::from_str(text).map_err(|why| why.to_string())?;
    let mut steps = Vec::with_capacity(written.len());
    for (index, step) in written.iter().enumerate() {
        let at = index + 1;
        let step = match step {
            Written {
                press: Some(buttons),
                frames,
                play_until: None,
                wait_until: None,
                quit: None,
            } => match frames {
                None => return Err(format!("step {at} presses without saying for how long")),
                Some(0) => return Err(format!("step {at} is over before it starts")),
                Some(frames) => Step::Press {
                    buttons: buttons.clone(),
                    frames: *frames,
                },
            },
            Written {
                press: None,
                frames: None,
                play_until: Some(until),
                wait_until: None,
                quit: None,
            } => Step::Play(until.read(at)?),
            Written {
                press: None,
                frames: None,
                play_until: None,
                wait_until: Some(until),
                quit: None,
            } => Step::Wait(until.read(at)?),
            Written {
                press: None,
                frames: None,
                play_until: None,
                wait_until: None,
                quit: Some(true),
            } => Step::Quit,
            _ => {
                return Err(format!(
                    "step {at} is a press, a play_until, a wait_until or a quit, and one of the four"
                ));
            }
        };
        steps.push(step);
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::{Step, Until, read};

    /// A recipe of every step there is, which is what one looks like — and what the scenario this was
    /// written for is: 妖々夢 played into stage 2 as far as its midboss, left, picked up again, and closed.
    #[test]
    fn a_recipe_is_a_list_of_steps_carried_out_in_order() {
        assert_eq!(
            read(
                "- press: [decide]\n  frames: 2\n\
                 - play_until: { stage: 2, chapter: 3 }\n\
                 - wait_until: { question: resume }\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                },
                Step::Play(Until::Chapter {
                    stage: 2,
                    chapter: 3,
                }),
                Step::Wait(Until::Question("resume".to_owned())),
                Step::Quit,
            ]),
        );
    }

    /// And the recipe the resume's own parting is measured with, word for word as `docs/todo/th07-the-run.md`
    /// carries it — a grammar this stopped reading is a measurement nobody can take again, and the file that
    /// holds it says nothing when it goes stale.
    ///
    /// The released stretch between the two presses is the whole reason it is not the recipe above: a menu
    /// takes a press where the bit goes up, so two presses with nothing between them are one press.
    #[test]
    fn the_recipe_the_partings_measurement_is_taken_with_reads() {
        assert_eq!(
            read(
                "- press: [decide]\n  frames: 2\n\
                 - wait_until: { frames: 10 }\n\
                 - play_until: { stage: 2, chapter: 3 }\n\
                 - press: [cancel]\n  frames: 2\n\
                 - wait_until: { question: resume }\n\
                 - press: [decide]\n  frames: 2\n\
                 - play_until: { frames: 600 }\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                },
                Step::Wait(Until::Frames(10)),
                Step::Play(Until::Chapter {
                    stage: 2,
                    chapter: 3,
                }),
                Step::Press {
                    buttons: vec!["cancel".to_owned()],
                    frames: 2,
                },
                Step::Wait(Until::Question("resume".to_owned())),
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                },
                Step::Play(Until::Frames(600)),
                Step::Quit,
            ]),
        );
    }

    /// **A step that is over before it starts is refused**, rather than skipped: a recipe is carried out a
    /// step at a time and counted in the log, so one whose steps do not all happen is one nobody can hold
    /// against the file it came from.
    #[test]
    fn a_step_that_is_over_before_it_starts_is_refused() {
        assert_eq!(
            read("- press: [shot]\n  frames: 0\n"),
            Err("step 1 is over before it starts".to_owned()),
        );
        assert_eq!(
            read("- press: [shot]\n  frames: 1\n- play_until: { frames: 0 }\n"),
            Err("step 2 is over before it starts".to_owned()),
        );
    }

    /// A press without its frames, which is a step nothing says the length of.
    #[test]
    fn a_press_without_its_frames_is_refused() {
        assert_eq!(
            read("- press: [shot]\n"),
            Err("step 1 presses without saying for how long".to_owned()),
        );
    }

    /// Two steps in one, and a step that is none: both are a recipe nobody can carry out in an order.
    #[test]
    fn a_step_that_is_two_things_or_nothing_is_refused() {
        assert!(read("- press: [shot]\n  frames: 2\n  quit: true\n").is_err());
        assert!(read("- {}\n").is_err());
    }

    /// And a wait for two things at once, or for nothing.
    #[test]
    fn a_wait_for_more_than_one_thing_is_refused() {
        assert!(read("- wait_until: { stage: 2, question: resume }\n").is_err());
        assert!(read("- wait_until: { stage: 2 }\n").is_err());
    }

    /// And a key nothing knows, which is a recipe written for something else.
    #[test]
    fn a_step_with_a_key_nothing_knows_is_refused() {
        assert!(read("- press: [shot]\n  frames: 2\n  hold: yes\n").is_err());
        assert!(read("- dance: [shot]\n").is_err());
    }
}
