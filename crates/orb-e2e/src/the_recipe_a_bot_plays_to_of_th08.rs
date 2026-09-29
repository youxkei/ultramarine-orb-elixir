//! **The recipe steps of th08 that end on something the game has up**: a line of a conversation, and a spell
//! card.
//!
//! What `the_recipe_a_bot_plays_to_of_th07.rs` asks of that game, for the two conditions this game reads its
//! own way. A step is the same engine over every game — `orb_bot`'s — and what ends one is a field of `State`
//! or a question a `Game` answers, so what is this game's is those reads: `Th08::a_line_of_dialogue_is_shown`
//! over the run screen's line and `State::spellcard` over the card's own static. The stage's own boss against a
//! midboss is `th08.rs`'s `a_recipes_boss_of_th08_is_the_one_fought_to_a_song_of_her_own`.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// th08's own bomb button, as `Th08` has it: what the step after the one waited on presses, so that the frame
/// it got is one the word says.
const BOMB: u16 = 0x002;

/// The card a step waits for, by the record it has of its own: any card, what the test asks of it being that
/// it is one.
const A_CARD: usize = 20;

/// A launch played by `recipe` over a run placed in its first stage, with the stage's own script standing and
/// nothing fired at the player: what ends the step is what the test puts up.
fn launched(name: &str, recipe: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.fires_at_the_player(false);
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game
}

/// The words the run acted on over `frames` frames, one a frame.
fn words(game: &Fake, frames: u32) -> Vec<u16> {
    (0..frames)
        .map(|_| {
            game.frame();
            game.image().the_word_the_run_acts_on()
        })
        .collect()
}

/// **A step can wait for a conversation, which is up for only as long as it takes to press past.** A
/// conversation here holds the run's clock and not the stage, so a step bounded by a count of frames walks
/// either side of it; whether a line is up is a thing orb already reads, so it is a thing a step can wait for.
#[test]
fn a_step_of_th08_ends_where_a_line_of_dialogue_is_up() {
    in_its_own_process(|| {
        let game = launched(
            "th08-a-recipe-waiting-for-a-dialogue",
            "- play_until: { dialogue: true }\n- press: [bomb]\n  frames: 2\n",
        );
        game.image().no_dialogue_is_up();
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with no conversation up at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        // **And not on the line the screen is handed over on**, `-2`, which the game's own predicate counts
        // (0x4358bb) and which is a frame with nothing said on it.
        game.image().a_line_of_dialogue_is_up(-2);
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended on the line the screen is handed over on:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().a_line_of_dialogue_is_up(0);
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where a line came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            pressed.iter().any(|word| word & BOMB != 0),
            "the step after it never got a frame: {pressed:x?}",
        );
    });
}

/// **A step can wait for a spell card**, which chapter of a stage one is being this game's own numbering and
/// different stage by stage.
#[test]
fn a_step_of_th08_ends_where_a_spell_card_is_up() {
    in_its_own_process(|| {
        let game = launched(
            "th08-a-recipe-waiting-for-a-card",
            "- play_until: { card: true }\n- press: [bomb]\n  frames: 2\n",
        );
        game.image().no_card_is_up();
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with no card up at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().a_card_is_up(A_CARD);
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where a card came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            pressed.iter().any(|word| word & BOMB != 0),
            "the step after it never got a frame: {pressed:x?}",
        );
    });
}
