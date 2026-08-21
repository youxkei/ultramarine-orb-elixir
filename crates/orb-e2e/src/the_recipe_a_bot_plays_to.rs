//! **The recipe orb plays a run of 妖々夢 to**, and the words it presses to get through the screens on the
//! way.
//!
//! A run to be picked up out of a file has to be started, and starting one is menus: the mode question orb
//! puts up itself, the title, the difficulty, the character and the shot. None of that is a rule about
//! playing — it is a sequence somebody carries out once — so it is written down rather than worked out, and
//! what carries it out is the same read the run's own words go through.
//!
//! **A step is a word held for a number of frames** — or tapped for as long as it takes, where the step says
//! what it is waiting for. Which is the whole vocabulary a menu needs and the one thing every screen has in
//! common: what a screen does with `shot` held for two frames is the screen's business, and a recipe that says
//! which words in which order can drive any of them without orb knowing what any of them are.
//!
//! The tap is what a run being *started* takes, and it is not a convenience: a menu answers the frame a button
//! goes down, so one word held across four screens takes one of them — and how many screens there are between
//! the title and a stage is not something a recipe can count, which item 妖々夢's title menu comes up on being
//! whatever its score file last remembered.
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

/// How often a step that has not ended says how long it has been, as `orb_bot::STILL_WAITING` has it.
const STILL_WAITING: u32 = 600;

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

/// **A press with an `until` is tapped rather than held**: held for its frames, released for as many, over
/// and over until what it is waiting for is reached.
///
/// Which is what the screens between the title and a stage take. A menu of this game's answers the frame a
/// button goes down and not the frames it stays down, so a word held across four screens takes one of them —
/// and how many there are is not something a recipe can count: which item 妖々夢's title menu comes up on is
/// whatever its score file last remembered.
#[test]
fn a_press_with_an_until_is_tapped_until_it_is_reached() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-tapped",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- press: [shot]\n  frames: 2\n  until: { frames: 9 }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        assert_eq!(
            words(&game, 11),
            vec![(SHOT, 2), (0, 2), (SHOT, 2), (0, 2), (SHOT, 1), (BOMB, 2),],
            "the press was not tapped until its ninth frame:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A step can give up, which is what stops a recipe holding a launch for ever.**
///
/// What a step waits for may never happen: a `play_until` a stage the run died before reaching waits on a
/// screen the rules do not press, and a launch nobody is watching sits there until somebody comes back and
/// closes it by hand. A bound says how long that step is worth waiting through, and the log says which of the
/// two ended it.
#[test]
fn a_step_that_gives_up_ends_when_it_says() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-giving-up",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- wait_until: { question: resume }\n  give_up_after: 3\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        let pressed = words(&game, 5);
        assert_eq!(
            pressed,
            vec![(0, 3), (BOMB, 2)],
            "the step waiting for a question nothing put up did not give up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("autoplay: step 1 of 2 gave up"),
            "giving up was not said:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A `play` step ends where the run is over**, there being nothing left to play.
///
/// Which is what a launch nobody is watching needs of it: a run that lost its last life leaves the game on its
/// own screen, and the rules press nothing there — so the step waited out its whole bound while the log said
/// `still waiting` sixty times over. An attempt that failed should end and let the next one start.
#[test]
fn a_play_step_ends_where_the_run_is_over() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-with-the-run-over",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { stage: 6, chapter: 30 }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        // The scene a run that lost its last life leaves the game in, which is not a run's.
        let over = game.image().scene_of_a_run_over();
        game.image().is_in_scene(over, over);

        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 gave up"),
            "the step went on playing a run that was over:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **A step that is getting nowhere says so, over and over, for as long as it is getting nowhere.**
///
/// Which is what a launch nobody is watching needs of its log: three launches in a row sat on a screen the
/// rules could not answer — orb's own offer to retry a death, whose menu reads the keyboard rather than the word
/// the bot presses — and every one of them was found by somebody looking at the screen. A line at a time makes
/// it `grep`, and the count in it says how long.
#[test]
fn a_step_that_is_getting_nowhere_says_so() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-getting-nowhere",
            Work::wandering(WORK_US),
            |config| {
                // Waiting for something nothing will put up, and with nothing to end it: the shape every stall
                // has had.
                config.autoplay = Some("- wait_until: { question: retry }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        words(&game, STILL_WAITING * 2 + 10);
        let said = game
            .log()
            .lines()
            .iter()
            .filter(|line| line.contains("autoplay: step 1 of 1 is still waiting"))
            .count();
        assert_eq!(
            said,
            2,
            "a step that got nowhere for {} frames did not say so twice:\n  {}",
            STILL_WAITING * 2,
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
