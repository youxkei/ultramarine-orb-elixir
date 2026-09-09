//! **The recipe orb plays a run of th07 to**, and the words it presses to get through the screens on the
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
//! the title and a stage is not something a recipe can count, which item th07's title menu comes up on being
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

/// th07's own first two buttons, as `Th07` has them.
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
/// and how many there are is not something a recipe can count: which item th07's title menu comes up on is
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
/// **A step can wait for a boss to be up, which is what the midsection of a stage ends at.**
///
/// Practising the part of a stage before its boss is a thing a recipe could not say: what a step waits for is a
/// stage, a chapter, a screen, a cursor or a count of frames, so the midsection had to be bounded by a wall
/// clock — and a bound guessed at ended one run at chapter 3 with the midsection still going. The stage's own
/// boss being up is a thing orb already reads to decide whether a chapter's music is the stage's, so it is a
/// thing a step can wait for.
#[test]
fn a_step_ends_where_a_boss_is_up() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-waiting-for-a-boss",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { boss: true }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_player_is_playing();
        game.image().is_playing(4, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();

        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with no boss up at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.the_stages_own_boss_arrives();
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where a boss came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **A picture can be asked for a moment after its step begins, which is what a thing being drawn takes.**
///
/// A screen and a card's plate both have an animation of their own, so the frame a condition becomes true is
/// the frame before the thing that condition is about is on the screen: three pictures of three menus each came
/// out mid-fade showing the menu before, and a picture on the frame a card started came out with the boss's name
/// and the timer in place and no plate. Which was walked round by putting a `play_until: { frames: 30 }` in
/// front of every `screenshot` — a settle written out once per picture in every recipe, and left out is a
/// picture of the wrong frame. So the step that asks for the picture is the step that waits for it, and the
/// frames it waits are the game's own rules' to play: a run whose player stands still for half a second is a
/// run that gets shot.
#[test]
fn a_picture_can_be_asked_for_a_moment_after_its_step_begins() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-letting-a-plate-be-drawn",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- screenshot: the card's own plate\n  frames: 3\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_player_is_playing();
        game.image().is_playing(4, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();

        words(&game, 2);
        assert!(
            !game.log().said("asks for a picture"),
            "the picture was asked for before the frames it waits out had gone by:\n  {}",
            game.log().lines().join("\n  "),
        );

        // The three frames it waits out, then the frame the picture is asked on, then the step after it.
        let pressed = words(&game, 3);
        assert!(
            game.log()
                .said("autoplay: step 1 of 2 asks for a picture of the card's own plate"),
            "the picture was never asked for:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end with the asking:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **A step can wait for a dialogue to be up, which is the one thing a recipe cannot reach by counting
/// frames.**
///
/// A boss's conversation is what a stage's midsection ends at and the fight begins after, and the rules press
/// past it as fast as the game takes presses — so it is up for a handful of frames, and a step bounded by a
/// count of them walks either side of it. Six shots at hundred-frame intervals through stage 1's midsection caught
/// the stage every time and the conversation never. Whether one is up is a thing orb already reads to decide
/// whether the rules have a line to press past, so it is a thing a step can wait for.
#[test]
fn a_step_ends_where_a_dialogue_is_up() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-waiting-for-a-dialogue",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { dialogue: true }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_player_is_playing();
        game.image().is_playing(4, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();
        game.image().has_a_dialogue();

        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with no dialogue up at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And not on the line the screen is handed over on**, which is a frame with no words drawn on it: the
        // game's own predicate counts it, and a picture asked for there caught stage 5's and stage 6's boxes
        // still being drawn and nothing being said in them.
        game.image().the_dialogue_is_at_line(-2);
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended on the line the screen is handed over on:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_dialogue_is_at_line(0);
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where a dialogue came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **A step can wait for a spell card, which is the frame the plate its name is on is being drawn on.**
///
/// Which chapter of a stage is a card is the game's own numbering and differs stage by stage — stage 1's boss opens
/// its card at chapter 7 and stage 4's at chapter 12 — so a step that named one was a step written against a
/// reading of one afternoon's logs, and twelve pictures taken at chapters named that way caught no plate at
/// all. Whether a card is running is a thing orb already reads to name the chapter, so it is a thing a step can
/// wait for and no chapter has to be counted.
#[test]
fn a_step_ends_where_a_spell_card_is_running() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-waiting-for-a-card",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { card: true }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_player_is_playing();
        game.image().is_playing(4, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();
        game.image().no_spellcard_is_running();

        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with no card running at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_spellcard_is(0);
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where a card started:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **And a step can wait for the plate a card's name is on, which is not the frame the card started on.**
///
/// A card starting and its name being on the screen are two frames: pictures asked for as stage 1's and stage
/// 2's cards started each came out with the boss's name and the timer in place and nothing between them. So
/// the plate is its own condition — the bit the engine's own draw reads before it draws the thing — rather
/// than a count of frames after the card, which is a guess about an animation.
#[test]
fn a_step_ends_where_a_cards_name_is_shown() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-waiting-for-a-plate",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { card_name: true }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_player_is_playing();
        game.image().is_playing(4, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();
        // The card running, which is the frame the plate is *not* yet on the screen for.
        game.image().has_what_it_puts_over_a_run();
        game.image().the_spellcard_is(0);
        game.image().the_cards_name_is_drawn(false);

        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended on a card whose name was not being drawn:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().the_cards_name_is_drawn(true);
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where the card's name came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **A step gives the run up for the title**, which is how a session is interrupted rather than killed.
///
/// `quit` ends the process where it stands, and what that leaves is a run nobody left: no last frame, no line
/// saying the run ended, and a resume file holding whatever the last chapter wrote. What somebody playing does
/// instead is go back to the title from the pause menu — the run ends, and the file it was written down in is
/// the one the next launch offers. Which is the interruption a resume is *for*, so it is the one a recipe has to
/// be able to make.
///
/// **orb's own way out and not the menu's**, which is what `Th07::leave_run` already does for the retry menu's
/// third choice: the shown score caught up with the run's, the game's own menu flag cleared, and the front end
/// asked for. The pause menu asks for the screen a run ends on instead and that screen is drawn — a launch
/// showed a frame or two of its background — and a run given up is not a run anybody is being shown the score
/// of.
#[test]
fn a_step_gives_the_run_up_for_the_title() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-leaving-the-run",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- leave: true\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        // **The front end laid out before the run**, which is a launch's own order and what leaving needs: the
        // scene the run is given up to is that one, and a game asked to go back to a screen it was never on is a
        // game reading an object nothing made.
        game.image().the_front_end_is_up();
        game.image().the_player_is_playing();
        game.image().is_playing(1, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();

        words(&game, 4);
        assert_eq!(
            game.image().the_scene_wanted(),
            game.image().scene_of_the_front_end(),
            "the run was not put on its way to the title:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("autoplay: step 1 of 2 leaves the run"),
            "leaving was not said:\n  {}",
            game.log().lines().join("\n  "),
        );
        // And the step is over, the asking being the whole of it: what a recipe does next is its own business,
        // and the one this is written for closes the game.
        //
        // **Said rather than pressed**, because there is no run left to press into: the word a step hands over
        // after this one goes where the front end's own screens read it, and the word this test can see is the
        // run's.
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end with the asking:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A step asks for a picture of the frame being drawn**, which is what a screen full of words is checked
/// from: whether the words on a screen are the ones somebody reads is not a thing any line of the log can say,
/// so the frame itself is the answer.
///
/// Named by what it was taken of, since a directory of pictures is read by that and not by a number. The
/// asking is the whole of the step — what writes the file is the draw chain's own end, and a step that waited
/// for the file would be waiting on a disk.
#[test]
fn a_step_asks_for_a_picture_of_the_frame() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-taking-a-picture",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- screenshot: the title menu\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_front_end_is_up();

        words(&game, 4);
        assert!(
            game.log()
                .said("autoplay: step 1 of 2 asks for a picture of the title menu"),
            "the picture was not asked for by name:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end with the asking:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A step asks the run to start over, which is what the game's own pause menu item does** — and what orb
/// turns into its question of where to start again. The one way to reach a chapter again without dying for it,
/// so it is the one way a recipe can ask for a retry.
///
/// **The ask waits where there is nothing to ask about.** A recipe reaches its steps in order and a run
/// reaches its first chapter in its own time, so the ask stands until a stage can be retried from — which is
/// what the pause menu's own ask does to somebody who opens it before there is a chapter behind them.
///
/// What answers it is a press at `question: retry`, the same as a player pressing at it, so this asserts the
/// question was raised and not what was chosen.
#[test]
fn a_step_asks_the_run_to_start_over() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-starting-over",
            Work::wandering(WORK_US),
            |config| {
                // A step after the ask so that the log names this one as the first of two, and one that
                // presses nothing: a recipe's bomb is this game's own cancel, and a cancel reaching the
                // title while the run is being set up here puts its cursor on the item that ends the game —
                // see `the_title_menu_of_th07_walked`, where that is what the real menu does.
                config.autoplay = Some(
                    "- start_over: true\n\
                     - wait_until: { frames: 2 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.in_a_pointdevice_run();

        // The ask goes in on the recipe's first frame, before the stage has a chapter to go back to — and the
        // question comes up on the frame it has one.
        game.frames_until_a_thread("the question of where to start again", || {
            game.log().said("retry: the run was asked to start over")
        });
        assert!(
            game.log()
                .said("autoplay: step 1 of 2 asks the run to start over"),
            "the ask was not said:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A step tells the rules what to do about being hit, and the telling is the whole of it.**
///
/// What it changes is every frame after it rather than the frame it is on — the chapter the run is in is what
/// the chance is drawn for — so there is nothing here to wait for, and the frame it costs goes to the rules
/// rather than being a word of nothing: a step that handed the game one would be a player standing still.
///
/// What the chance *does* is `the_bot_playing_th07`'s, this being about the step being read and carried out.
#[test]
fn a_step_tells_the_rules_to_take_hits() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-taking-hits",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- take_hits: { chance: 0.2, times_each_retry: 0.5, gives_up: 0.1 }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        let pressed = words(&game, 3);
        assert!(
            game.log().said(
                "autoplay: step 1 of 2 takes hits, a chance of 0.200 per chapter, 0.500 of that \
                 each retry, and 0.100 of giving the run up at one"
            ),
            "the numbers the step was given were not said:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end with the telling:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

/// **And a chance that is not a chance is a recipe refused**, which is the same rule every other number in one
/// goes by: a step nothing can be read out of is a launch that says so and plays nothing.
#[test]
fn a_chance_that_is_not_a_chance_is_refused() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-with-no-chance",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- take_hits: { chance: 40 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        let pressed = words(&game, 5);
        assert!(
            game.log()
                .said("autoplay: the recipe cannot be read: step 1 says a chance of 40"),
            "a chance of forty was not refused:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed,
            vec![(0, 5)],
            "a recipe that could not be read was played anyway",
        );
    });
}

/// **And a midboss is not the boss it waits for**, the midsection carrying on after one.
///
/// A step waiting for a boss ended at stage 4's midboss, which is a third of the way into the midsection it was
/// written to practise. What tells the two apart is the song: th07 names two per stage and loads the second for
/// the fight the stage ends with, so a fight with the stage's own song still playing is a midboss — which is the
/// same test the chapter model already makes to decide whether to put a stage's song back on a resume.
#[test]
fn a_step_waiting_for_a_boss_does_not_end_at_a_midboss() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-past-a-midboss",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- play_until: { boss: true }\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.image().the_player_is_playing();
        game.image().is_playing(4, 1);
        game.image().has_a_runs_numbers();
        game.image().is_not_paused();

        // The midboss: a boss in the slot with the stage's own song still playing.
        game.the_boss_arrives();
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended at a midboss:\n  {}",
            game.log().lines().join("\n  "),
        );

        // Beaten, and the midsection carrying on — which is the part the step was written to play.
        game.the_boss_is_beaten();
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with the midboss beaten and no boss up at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        // And the stage's own boss, which brings its own song with it.
        game.the_stages_own_boss_arrives();
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where the stage's own boss came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame",
        );
    });
}

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

/// **Every question orb puts up is one a step can name**, which is what a launch playing itself needs to
/// reach the screens behind them at all: a name orb does not put up is a recipe refused, so a question added
/// to orb and not to this list is a screen no recipe can wait for.
///
/// The two about a replay are the ones this is here for — whether to keep the one just played, and the lists
/// one is chosen out of. Waited for with nothing to put them up, so what each step proves is that its name
/// was *read*: a step that gave up is a step whose `until` was understood, where an unreadable one takes the
/// whole recipe with it.
#[test]
fn every_question_orb_puts_up_is_one_a_step_can_name() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-naming-questions",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- wait_until: { question: mode }\n  give_up_after: 2\n\
                     - wait_until: { question: resume }\n  give_up_after: 2\n\
                     - wait_until: { question: retry }\n  give_up_after: 2\n\
                     - wait_until: { question: keep_replay }\n  give_up_after: 2\n\
                     - wait_until: { question: choose_replay }\n  give_up_after: 2\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        let pressed = words(&game, 12);
        assert!(
            !game.log().said("the recipe cannot be read"),
            "a question orb puts up is not one a step can name:\n  {}",
            game.log().lines().join("\n  "),
        );
        for step in 1..=5 {
            assert!(
                game.log()
                    .said(&format!("autoplay: step {step} of 6 gave up")),
                "step {step} never waited on the question it names:\n  {}",
                game.log().lines().join("\n  "),
            );
        }
        // And the step after them got its frames, which is what says the recipe was read to the end.
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after the questions never got a frame",
        );
    });
}

/// **A step answers one of orb's own questions by item, which is the one thing a press cannot do to them.**
///
/// A question of orb's keeps its own cursor and nothing of it reaches a bot, so every press takes the item the
/// cursor came up on: `つづきから` and never `はじめから`, and the first stage, chapter and attempt of a replay
/// and never any other. Three launches on the real game were written to press `down` and then `decide` at
/// `どこから始める`, and all three came back `resume: from where it stopped` — the item the cursor was on.
///
/// So the step says which item it means, and the number is its place in the list on the screen. What this
/// asserts is the grammar and the waiting: the step is read, it waits for the question it names, and its bound
/// ends it where nothing puts one up. The answer landing on the item named is
/// `the_replay_offered.rs`'s, over the question that has one.
#[test]
fn a_step_answers_one_of_orbs_questions_by_the_item_it_names() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-answering",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- answer: { question: resume, item: 1 }\n  give_up_after: 3\n\
                     - answer: { question: keep_replay, item: 1 }\n  give_up_after: 3\n\
                     - press: [bomb]\n  frames: 2\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);

        let pressed = words(&game, 8);
        assert!(
            !game.log().said("the recipe cannot be read"),
            "a step that answers a question by item is not a step:\n  {}",
            game.log().lines().join("\n  "),
        );
        // Nothing pressed while it waits, which is the point of it: a press at one of these takes the item
        // the cursor is on, and that is the item the step exists to say is not the one meant.
        assert_eq!(
            pressed,
            vec![(0, 6), (BOMB, 2)],
            "the steps answering a question pressed something:\n  {}",
            game.log().lines().join("\n  "),
        );
        for step in 1..=2 {
            assert!(
                game.log()
                    .said(&format!("autoplay: step {step} of 3 gave up")),
                "step {step} never waited on the question it answers:\n  {}",
                game.log().lines().join("\n  "),
            );
        }
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

/// **And `menu` is a word a recipe can press**, which is the one road it could not take.
///
/// What is behind that button is the game's own pause menu — 「最初からやり直す」 and everything that item
/// leads to: orb's question about where to start over, and what the pause did to the music on the way. A
/// recipe that cannot press it cannot reach any of that, so a session at the keyboard was the only way to try
/// one.
///
/// **What this asks is which word it lands in.** The manager reads that button off the word the game's own
/// *read* left, as an edge against the frame before — so a press that did not reach *that* word is a press no
/// menu ever sees, and it is the half a recipe's own machinery could get wrong: the word a recipe presses is
/// handed to the game inside its own input hook, and everything about a run is written down off it.
///
/// The run's own word carries the button too, the game's own copy of the read being the whole word — what
/// `RUN_INPUT` keeps it out of is a run *written down*, so that a resume feeding its frames back does not open
/// a menu instead of playing. What the menu then does with the press is the game's, and
/// `the_word_a_run_of_th07_is_written_down_as.rs` is where both of those are held to.
#[test]
fn the_menu_button_a_recipe_presses_reaches_the_word_the_read_leaves() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th07-a-recipe-opening-the-menu",
            Work::wandering(WORK_US),
            |config| {
                // Held for a moment and let go: the manager takes the button's edge, so a word held for ever
                // opens the menu once and a recipe that never released it could not close one either.
                config.autoplay = Some("- press: [menu]\n  frames: 4\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.streams_its_song(0);
        game.frame();

        assert_eq!(
            game.image().the_word_the_read_left() & MENU,
            MENU,
            "the pause button a recipe pressed is in no word the game read:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// th07's pause button, as `Th07`'s own `BUTTON_MENU` has it — the mapping's fourth entry.
const MENU: u16 = 0x008;
