//! **The recipe orb plays a run of 妖々夢 to**, and the words it presses to get through the screens on the
//! way.
//!
//! A run to be picked up out of a file has to be started, and starting one is menus: the mode question orb
//! puts up itself, the title, the difficulty, the character and the shot. None of that is a rule about
//! playing — it is a sequence somebody carries out once — so it is written down rather than worked out, and
//! what carries it out is the same read the run's own words go through.
//!
//! **A step is a word held for a number of frames.** Which is the whole vocabulary a menu needs and the one
//! thing every screen has in common: what a screen does with `shot` held for two frames is the screen's
//! business, and a recipe that says which words in which order can drive any of them without orb knowing
//! what any of them are.
//!
//! What comes after the last step is the run being played, which is [`the_bot_playing_th07`]'s rules.
//!
//! [`the_bot_playing_th07`]: super::the_bot_playing_th07

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// 妖々夢's own first two buttons, as `Th07` has them.
const SHOT: u16 = 0x001;
const BOMB: u16 = 0x002;

/// A recipe of two steps, each a word and how long to hold it — the shape every screen is driven with.
const RECIPE: &str = "\
- press: [shot]
  frames: 2
- press: [bomb]
  frames: 3
";

/// The words the run was handed over `frames` frames, in order, with the frames each was held for counted —
/// which is what a step *is*.
///
/// A frame at a time and read in between: the word a frame is played with is written where the read happens
/// and read back where it stands, so a run of frames driven at once would only ever show the last of them.
fn words(game: &Fake, frames: u32) -> Vec<(u16, usize)> {
    let mut runs: Vec<(u16, usize)> = Vec::new();
    for _ in 0..frames {
        game.frame();
        let word = game.image().the_word_the_run_acts_on();
        match runs.last_mut() {
            Some((last, held)) if *last == word => *held += 1,
            _ => runs.push((word, 1)),
        }
    }
    runs
}

/// Each step's word is held for the frames the step says, and then the next step's is.
#[test]
fn a_recipe_is_played_a_step_at_a_time() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(RECIPE.to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        // **Nothing of the menus driven here.** A launch playing to a recipe answers every read of the game's
        // own keyboard, so a test that pressed its way into a run would be pressing against the recipe — and
        // the word each frame was handed is written where a test can read it whatever screen is up.

        assert_eq!(
            words(&game, 5),
            vec![(SHOT, 2), (BOMB, 3)],
            "the recipe's steps were not the words the run was handed:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// And a recipe nothing can be read out of is a launch that says so and plays nothing, rather than one that
/// presses whatever a half-read file came to.
#[test]
fn a_recipe_that_cannot_be_read_is_refused() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-refused",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- press: [what]\n  frames: 2\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        let pressed = words(&game, 5);
        assert!(
            game.log().said("autoplay: the recipe cannot be read"),
            "a recipe naming a button nothing has was not refused:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed,
            vec![(0, 5)],
            "a recipe that could not be read was played anyway",
        );
    });
}
