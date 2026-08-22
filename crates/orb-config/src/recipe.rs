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
//! **And a `press` with an `until` of its own is a tap**: down for its frames, up for as many, over and over
//! until what it waits for is reached. Which is how a run is *started* without counting the screens on the way
//! — a menu answers the frame a button goes down, so a word held across four screens takes one of them, and
//! how many there are is not something a recipe can know: which item a title menu comes up on is whatever the
//! game's own score file last remembered.
//!
//! **What a tap waits for has to be something the tap cannot take away.** A tap of `decide` waiting on a
//! `question` answers that question the frame it comes up, so the frame the walker next looks at has no
//! question up and the step never ends — which is a launch that walked its whole front end and then tapped
//! through a stage without the rules ever getting a frame. A `stage` and a `chapter` are *reached*, so no press
//! can undo one.
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
    /// A word held for a number of frames, which is what a screen takes — or **tapped** for as long as it
    /// takes, where an `until` says what it is waiting for: held for `frames`, released for as many, over and
    /// over.
    ///
    /// Tapped rather than held because a menu answers the frame a button goes down: a word held across four
    /// screens takes one of them. And for as long as it takes rather than a count of taps, because how many
    /// screens there are between the title and a stage is not something a recipe can know — which item a title
    /// menu comes up on is whatever the game's own score file last remembered.
    Press {
        buttons: Vec<String>,
        frames: u32,
        until: Option<Until>,
        give_up_after: Option<u32>,
    },
    /// The game played by its own rules until something is reached.
    Play {
        until: Until,
        give_up_after: Option<u32>,
    },
    /// And nothing held until something is reached, which is what waiting for a screen to come up is.
    Wait {
        until: Until,
        give_up_after: Option<u32>,
    },
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
    /// **The game's own cursor being on the item at this place in the list**, counted from zero the way the
    /// game counts it.
    ///
    /// Which is how a screen this grammar cannot see the items of is walked: 面別プラクティス is the third of
    /// the title menu's items and every screen after it is another list, so a recipe that counted taps instead
    /// would be a recipe written against one machine's unlocks.
    Cursor(i32),
    /// **That item of that screen**, which is what a list has to be named by: the cursor is one field whatever
    /// screen is up, so waiting for a number alone waits for whichever screen reaches it first. A launch
    /// walking into 面別プラクティス put the difficulty on Lunatic that way — the 3 it was waiting for was the
    /// difficulty's fourth item and not the stage's.
    Where { screen: i32, cursor: i32 },
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
    /// A `press`'s own, which is what turns it into a tap — beside `frames` rather than in place of it, those
    /// being how long each tap holds.
    until: Option<WrittenUntil>,
    /// How many frames a step is worth waiting through, for one whose `until` may never happen.
    ///
    /// **Which is what stops a recipe holding a launch for ever.** A `play_until` a stage the run died before
    /// reaching waits on a screen the rules do not press, and a launch nobody is watching sits there until
    /// somebody closes it by hand — so a step that can go wrong that way says how long it is worth waiting
    /// through, and the walker says in the log which of the two ended it.
    give_up_after: Option<u32>,
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
    screen: Option<i32>,
    cursor: Option<i32>,
    frames: Option<u32>,
}

impl WrittenUntil {
    fn read(&self, at: usize) -> Result<Until, String> {
        match (
            self.stage,
            self.chapter,
            &self.question,
            self.screen,
            self.cursor,
            self.frames,
        ) {
            (Some(stage), Some(chapter), None, None, None, None) => {
                Ok(Until::Chapter { stage, chapter })
            }
            (None, None, Some(question), None, None, None) => Ok(Until::Question(question.clone())),
            (None, None, None, Some(screen), Some(cursor), None) => {
                Ok(Until::Where { screen, cursor })
            }
            (None, None, None, None, Some(cursor), None) => Ok(Until::Cursor(cursor)),
            (None, None, None, None, None, Some(0)) => {
                Err(format!("step {at} is over before it starts"))
            }
            (None, None, None, None, None, Some(frames)) => Ok(Until::Frames(frames)),
            _ => Err(format!(
                "step {at} waits for a stage and a chapter, a question, an item the cursor is on \
                 with or without the screen it is a list of, or a count of frames, and for one of \
                 those"
            )),
        }
    }
}

/// A bound checked the way every other count of frames in here is: nothing to wait through is a step that
/// never happens.
fn gives_up(frames: Option<u32>, at: usize) -> Result<Option<u32>, String> {
    match frames {
        Some(0) => Err(format!("step {at} gives up before it starts")),
        held => Ok(held),
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
                until,
                give_up_after,
                play_until: None,
                wait_until: None,
                quit: None,
            } => match frames {
                None => return Err(format!("step {at} presses without saying for how long")),
                Some(0) => return Err(format!("step {at} is over before it starts")),
                Some(frames) => Step::Press {
                    buttons: buttons.clone(),
                    frames: *frames,
                    until: until.as_ref().map(|until| until.read(at)).transpose()?,
                    give_up_after: gives_up(*give_up_after, at)?,
                },
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after,
                play_until: Some(until),
                wait_until: None,
                quit: None,
            } => Step::Play {
                until: until.read(at)?,
                give_up_after: gives_up(*give_up_after, at)?,
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after,
                play_until: None,
                wait_until: Some(until),
                quit: None,
            } => Step::Wait {
                until: until.read(at)?,
                give_up_after: gives_up(*give_up_after, at)?,
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
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
                    until: None,
                    give_up_after: None,
                },
                Step::Play {
                    until: Until::Chapter {
                        stage: 2,
                        chapter: 3,
                    },
                    give_up_after: None,
                },
                Step::Wait {
                    until: Until::Question("resume".to_owned()),
                    give_up_after: None,
                },
                Step::Quit,
            ]),
        );
    }

    /// A press with an `until` is one step and not two, and it keeps its frames: those are how long each tap
    /// holds, not how long the step lasts.
    #[test]
    fn a_press_can_be_tapped_until_something_is_reached() {
        assert_eq!(
            read("- press: [decide]\n  frames: 10\n  until: { question: mode }\n"),
            Ok(vec![Step::Press {
                buttons: vec!["decide".to_owned()],
                frames: 10,
                until: Some(Until::Question("mode".to_owned())),
                give_up_after: None,
            }]),
        );
    }

    /// **A press can be tapped until the game's own cursor is on an item**, which is what makes a screen this
    /// grammar cannot see the items of walkable: the item is named by where it sits, and the game keeps that
    /// number itself.
    ///
    /// Which is how 面別プラクティス is reached at all — Practice Start is the third of the title menu's items
    /// and each screen after it is another list, so a recipe that could only count taps would be a recipe
    /// written against somebody's unlocks.
    #[test]
    fn a_press_can_be_tapped_until_the_cursor_is_on_an_item() {
        assert_eq!(
            read("- press: [down]\n  frames: 10\n  until: { cursor: 2 }\n"),
            Ok(vec![Step::Press {
                buttons: vec!["down".to_owned()],
                frames: 10,
                until: Some(Until::Cursor(2)),
                give_up_after: None,
            }]),
        );
        // And not with anything else beside it, an `until` being one of the four and not a set of conditions.
        assert!(read("- press: [down]\n  until: { cursor: 2, stage: 1 }\n").is_err());
    }

    /// **And the screen the list belongs to beside the item**, since the cursor is one field whatever screen is
    /// up: a launch waiting for item 3 of the stage list got item 3 of the difficulty list and put the run on
    /// Lunatic.
    #[test]
    fn a_press_can_be_tapped_until_the_cursor_is_on_an_item_of_a_screen() {
        assert_eq!(
            read("- press: [down]\n  frames: 10\n  until: { screen: 11, cursor: 3 }\n"),
            Ok(vec![Step::Press {
                buttons: vec!["down".to_owned()],
                frames: 10,
                until: Some(Until::Where {
                    screen: 11,
                    cursor: 3,
                }),
                give_up_after: None,
            }]),
        );
        // The screen alone names no item, so it is not an `until` of its own.
        assert!(read("- press: [down]\n  until: { screen: 11 }\n").is_err());
    }

    /// **A step can say how long it is worth waiting through**, which is what stops a recipe holding a launch
    /// for ever: a `play_until` a stage the run died before reaching waits on a screen the rules do not press.
    #[test]
    fn a_step_can_say_how_long_it_is_worth_waiting_through() {
        assert_eq!(
            read("- play_until: { stage: 2, chapter: 3 }\n  give_up_after: 30000\n"),
            Ok(vec![Step::Play {
                until: Until::Chapter {
                    stage: 2,
                    chapter: 3,
                },
                give_up_after: Some(30000),
            }]),
        );
        assert_eq!(
            read("- wait_until: { question: resume }\n  give_up_after: 0\n"),
            Err("step 1 gives up before it starts".to_owned()),
        );
    }

    /// And the two recipes the resume's own parting is measured with, word for word as
    /// `docs/todo/th07-the-run.md` carries them — a grammar this stopped reading is a measurement nobody can
    /// take again, and the file that holds them says nothing when it goes stale.
    ///
    /// **Two launches and no pause menu.** A run wants closing in the middle for a resume to have anything to
    /// pick up, and `quit` closes the game where it stands — where leaving through the pause menu would be a
    /// cursor to walk down a screen this grammar cannot see the items of.
    ///
    /// **And the tap waits for the first chapter and not for the mode question**, which is what a launch had to
    /// say: a tap of `decide` answers that question the frame it comes up, so what it was waiting for was gone
    /// before the frame the walker next looked at — the step never ended, and the rules never got a frame. A
    /// chapter is reached rather than offered, so nothing the tap presses can take it away again.
    /// **And every step that could wait for something that will not happen says how long it is worth waiting
    /// through**, so that a launch nobody is watching closes itself: a run that died before stage 2 leaves orb's
    /// own retry menu up, which the rules do not press, and one that was left there ran for an hour before
    /// anybody noticed.
    #[test]
    fn the_recipes_the_partings_measurement_is_taken_with_read() {
        assert_eq!(
            read(
                "- press: [decide]\n  frames: 10\n  until: { stage: 1, chapter: 1 }\n\
                 \x20 give_up_after: 3600\n\
                 - play_until: { stage: 2, chapter: 3 }\n  give_up_after: 36000\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 10,
                    until: Some(Until::Chapter {
                        stage: 1,
                        chapter: 1,
                    }),
                    give_up_after: Some(3600),
                },
                Step::Play {
                    until: Until::Chapter {
                        stage: 2,
                        chapter: 3,
                    },
                    give_up_after: Some(36000),
                },
                Step::Quit,
            ]),
        );
        assert_eq!(
            read(
                "- wait_until: { question: resume }\n  give_up_after: 1800\n\
                 - press: [decide]\n  frames: 2\n\
                 - play_until: { frames: 600 }\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Wait {
                    until: Until::Question("resume".to_owned()),
                    give_up_after: Some(1800),
                },
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                    until: None,
                    give_up_after: None,
                },
                Step::Play {
                    until: Until::Frames(600),
                    give_up_after: None,
                },
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
