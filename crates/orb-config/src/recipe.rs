//! **The grammar of a recipe orb plays a run to**: a list of steps, each a set of buttons to hold and how
//! many frames to hold them for.
//!
//! ```yaml
//! - press: [decide]
//!   frames: 2
//! - press: []
//!   frames: 30
//! - press: [down, decide]
//!   frames: 2
//! ```
//!
//! **The grammar is here and the names are the game's.** A list of names and a count of frames is the same
//! list whatever game reads it, so it is read once — and what `decide` or `focus` *is* differs from game to
//! game, so nothing here resolves one. `game::th07::bot` is where 妖々夢's are, and a name that game has
//! nothing for is a recipe it refuses.
//!
//! Here rather than in `orb-core` because this is where the YAML reader is: `orb.yaml` goes through the same
//! one, and a second copy of it in the DLL for a diagnostic would be weight in what ships.

use serde::Deserialize;

/// One step of a recipe.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    /// The buttons held together for this step, by the names the game knows them by. Empty is a step that
    /// holds nothing, which is how a recipe waits.
    pub press: Vec<String>,
    /// And how many frames to hold them for. A step of no frames presses nothing and is a step that could
    /// have been left out, so it is refused rather than skipped — a recipe whose steps do not all happen is
    /// one nobody can read off the file.
    pub frames: u32,
}

/// Reads one, or says what is wrong with it.
///
/// `deny_unknown_fields` on the step and a whole-list parse rather than a step at a time, for the resume
/// file's reason: a key nothing knows is a recipe written for something else, and carrying out half of one is
/// worse than carrying out none.
pub fn read(text: &str) -> Result<Vec<Step>, String> {
    let steps: Vec<Step> = serde_yaml_ng::from_str(text).map_err(|why| why.to_string())?;
    if let Some(at) = steps.iter().position(|step| step.frames == 0) {
        return Err(format!("step {} is held for no frames", at + 1));
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::{Step, read};

    /// A recipe of two steps, which is what one looks like.
    #[test]
    fn a_recipe_is_a_list_of_words_and_the_frames_they_are_held_for() {
        assert_eq!(
            read("- press: [decide]\n  frames: 2\n- press: []\n  frames: 30\n"),
            Ok(vec![
                Step {
                    press: vec!["decide".to_owned()],
                    frames: 2,
                },
                Step {
                    press: Vec::new(),
                    frames: 30,
                },
            ]),
        );
    }

    /// **A step that presses nothing for no frames is refused**, rather than skipped: a recipe is carried out
    /// a step at a time and counted in the log, so one whose steps do not all happen is one nobody can hold
    /// against the file it came from.
    #[test]
    fn a_step_held_for_no_frames_is_refused() {
        assert_eq!(
            read("- press: [shot]\n  frames: 0\n"),
            Err("step 1 is held for no frames".to_owned()),
        );
    }

    /// And a key nothing knows, which is a recipe written for something else.
    #[test]
    fn a_step_with_a_key_nothing_knows_is_refused() {
        assert!(read("- press: [shot]\n  frames: 2\n  hold: yes\n").is_err());
    }
}
