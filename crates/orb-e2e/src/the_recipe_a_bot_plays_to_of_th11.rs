//! **The recipe steps of th11 that end on something the game has up**: a conversation, and a spell card.
//!
//! What `the_recipe_a_bot_plays_to_of_th10.rs` asks of that game, asked of this one. A step is the same engine
//! over every game — `orb_bot`'s — and what ends one is a field of `State` or a question a `Game` answers, so what
//! is this game's is those reads: `Th11::a_line_of_dialogue_is_shown` over the conversation the panel at
//! `[0x4a8d84]` keeps at `+0x4438` while one runs (0x415af6, 0x431e90), and `State::spellcard` over the card
//! object's number at `+0x8dc` and its running bit, bit 0 of `+0x8e0` (0x40c6cc on).

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// th11's own bomb button, which the keyboard's read folds `X` into: what the step after the one waited on
/// presses, so that the frame it got is one the word says.
const BOMB: u32 = 0x002;

/// The card a step waits for, by its number: any card, what the test asks of it being that it is one.
const A_CARD: i32 = 20;

/// A launch played by `recipe` over a run placed in its first stage, with nothing fired at the player: what ends
/// the step is what the test puts up.
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

/// The words the run acted on over `frames` frames, one a frame: the run's copy the replay's calc job makes
/// (0x435fe0), which is what the player moves and bombs on.
fn words(game: &Fake, frames: u32) -> Vec<u32> {
    (0..frames)
        .map(|_| {
            game.frame();
            game.image().the_run_word()
        })
        .collect()
}

/// **A step can wait for a conversation**, which is up for only as long as it takes to press past.
#[test]
fn a_step_of_th11_ends_where_a_conversation_is_up() {
    in_its_own_process(|| {
        let game = launched(
            "th11-a-recipe-waiting-for-a-dialogue",
            "- play_until: { dialogue: true }\n- press: [bomb]\n  frames: 2\n",
        );
        game.image().no_conversation_is_up();
        words(&game, 4);
        assert!(
            !game.log().said("autoplay: step 1 of 2 done"),
            "the step ended with no conversation up at all:\n  {}",
            game.log().lines().join("\n  "),
        );

        game.image().a_conversation_is_up();
        let pressed = words(&game, 4);
        assert!(
            game.log().said("autoplay: step 1 of 2 done"),
            "the step did not end where a conversation came up:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            pressed.iter().any(|word| word & BOMB != 0),
            "the step after it never got a frame: {pressed:x?}",
        );
    });
}

/// **A step can wait for a spell card**, which chapter of a stage one is being this game's own fights' and
/// different stage by stage.
#[test]
fn a_step_of_th11_ends_where_a_spell_card_is_up() {
    in_its_own_process(|| {
        let game = launched(
            "th11-a-recipe-waiting-for-a-card",
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
