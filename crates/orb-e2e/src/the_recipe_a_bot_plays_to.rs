//! **The recipe orb plays a run of th06 to**: the steps whose meaning is this game's to answer.
//!
//! What a step *is* — a word held, a tap until something is reached, a bound, a picture, a question named — is
//! `orb-bot`'s and is not written twice; `the_recipe_a_bot_plays_to_of_th07.rs` holds it over the game the
//! recipes were written for. What is here is every step that ends on something read out of the game's own
//! memory, asked of this game's reading: the stage's own boss, a line of a conversation, a spell card and the
//! plate its name is on, the run asked to start over, the rules told what to do about being hit, and the button
//! that opens the game's own pause menu.
//!
//! **Walked in by the recipe's own first step**, which is what a launch playing to one does: the bot answers
//! every read of the game's own keyboard, so the presses the walk-in wants have to be the bot's — a step that
//! waits presses nothing, and the walk-in then waits for a decide nobody makes. So every recipe here opens with
//! the tap that takes the title, the shot select and orb's own question, and the step under test is its second.
//!
//! What a frame was played with is read off `g_CurFrameInput`, which is where this game's own read leaves the
//! word and where its stage reads it.

use crate::fake::th06::{BOSS_ARRIVES, CARD_NAME, CARD_STARTS, Fake, STAGE_BOSS_ARRIVES, the_run};
use crate::fake::{Launched, in_its_own_process};
use orb_config::LogLevel;

/// th06's own bomb and its pause button, as `Th06` has them: the bits `sight::bit` answers `bomb` and `menu`
/// with.
const BOMB: u16 = 0x002;
const MENU: u16 = 0x008;

/// The step every recipe here opens with: decide tapped until the stage's first chapter, which is the title,
/// the shot select and orb's own question — the same walk `the_bot_playing_th06` makes.
const WALK_IN: &str = "\
- press: [decide]
  frames: 2
  until: { stage: 1, chapter: 1 }
  give_up_after: 1800
";

/// And the step every recipe here ends with, whose word is what says the step before it ended: a press hands
/// the game its own word and nothing of the rules', so a frame with the bomb in it is a frame of this step.
const THEN_BOMB: &str = "\
- press: [bomb]
  frames: 2
";

/// A recipe of the walk-in, `step`, and the bomb after it.
fn walking_in_to(step: &str) -> String {
    format!("{WALK_IN}{step}{THEN_BOMB}")
}

/// What the second step of a three-step recipe says when it is over.
const STEP_2_DONE: &str = "autoplay: step 2 of 3 done";

/// The words the run was handed over `frames` frames, in order, with the frames each was held for counted —
/// which is what a step *is*.
///
/// A frame at a time and read in between: the word a frame is played with is written where the read happens
/// and read back where it stands, so a run of frames driven at once would only ever show the last of them.
fn words(game: &Fake, frames: u32) -> Vec<(u16, usize)> {
    let mut runs: Vec<(u16, usize)> = Vec::new();
    for _ in 0..frames {
        game.frame();
        let word = game.image().input_now();
        match runs.last_mut() {
            Some((last, held)) if *last == word => *held += 1,
            _ => runs.push((word, 1)),
        }
    }
    runs
}

/// A launch playing to `recipe`, with the run walked into by that recipe's own first step.
///
/// # Panics
/// Naming whichever step of the walk the run did not get past.
fn playing_to(name: &str, recipe: &str, songs: bool) -> Box<Fake> {
    let game = Fake::attach(name, the_run(), |config| {
        config.autoplay = Some(recipe.to_owned());
        config.log_level = LogLevel::Verbose;
    });
    if songs {
        game.plays_its_songs();
    }
    game.in_a_pointdevice_run();
    game
}

/// **A step can wait for the stage's own boss to be up, and a midboss is not it.**
///
/// Practising the part of a stage before its boss is a thing a recipe could not say: what a step waits for is a
/// stage, a chapter, a screen, a cursor or a count of frames, so the midsection had to be bounded by a wall
/// clock. What tells the stage's own boss from a midboss in this game is the song — an STD names the stage's
/// and the boss's, and the game plays the second for the fight the stage ends with — which is the same test the
/// chapter model makes to decide whether a chapter's music is the stage's to rewind. So the stage streams its
/// songs here, and the fight at [`BOSS_ARRIVES`] under the stage's own song is a midboss the step plays through.
#[test]
fn a_step_ends_where_th06s_stages_own_boss_is_up_and_not_at_its_midboss() {
    in_its_own_process(|| {
        let game = playing_to(
            "th06-a-recipe-waiting-for-a-boss",
            &walking_in_to("- play_until: { boss: true }\n"),
            true,
        );
        let log = game.log();

        // The midboss: a boss in the slot with the stage's own song still playing.
        game.frames_until("the midboss fought", BOSS_ARRIVES + 400, || {
            game.state().stage_frames > BOSS_ARRIVES + 30
        });
        assert!(
            game.state().boss_present,
            "the midboss never arrived, so nothing here says a step plays through one",
        );
        assert!(
            !log.said(STEP_2_DONE),
            "the step ended at a midboss:\n  {}",
            log.lines().join("\n  ")
        );

        // And the stage's own boss, which brings the second of the two songs with it.
        game.frames_until("the stage's own boss", STAGE_BOSS_ARRIVES + 400, || {
            game.state().stage_frames > STAGE_BOSS_ARRIVES
        });
        game.frames_until("the step ended", 30, || log.said(STEP_2_DONE));
        assert_eq!(
            words(&game, 1),
            vec![(BOMB, 1)],
            "the step after it never got a frame:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **A step can wait for a line of a conversation to be up**, which is the one thing a recipe cannot reach by
/// counting frames: the rules press past a line as fast as the game takes presses.
///
/// What says one is up in this game is the panel's own `currentMsgIdx` — `Th06::a_line_of_dialogue_is_shown`
/// reads the same index `State::in_dialogue` does, this game having no line of its own for handing the screen
/// over — so a line written there is a conversation on the screen.
#[test]
fn a_step_ends_where_a_line_of_th06s_dialogue_is_up() {
    in_its_own_process(|| {
        let game = playing_to(
            "th06-a-recipe-waiting-for-a-dialogue",
            &walking_in_to("- play_until: { dialogue: true }\n"),
            false,
        );
        let log = game.log();

        words(&game, 4);
        assert!(
            !log.said(STEP_2_DONE),
            "the step ended with no dialogue up at all:\n  {}",
            log.lines().join("\n  ")
        );

        game.image().the_dialogue_is_at_line(0);
        game.frames_until("the step ended", 30, || log.said(STEP_2_DONE));
        assert_eq!(
            words(&game, 1),
            vec![(BOMB, 1)],
            "the step after it never got a frame:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **A step can wait for a spell card**, which is what orb reads to name a chapter and so a thing no chapter
/// has to be counted for: the midboss at [`BOSS_ARRIVES`] is a boss and not a card, and the card it puts up
/// at [`CARD_STARTS`] is.
#[test]
fn a_step_ends_where_a_spell_card_of_th06_is_running() {
    in_its_own_process(|| {
        let game = playing_to(
            "th06-a-recipe-waiting-for-a-card",
            &walking_in_to("- play_until: { card: true }\n"),
            false,
        );
        let log = game.log();

        game.frames_until("the midboss fought", BOSS_ARRIVES + 400, || {
            game.state().stage_frames > BOSS_ARRIVES + 30
        });
        assert!(
            game.state().boss_present && game.state().spellcard.is_none(),
            "the midboss is not a boss with no card up, so nothing here tells a card from a boss",
        );
        assert!(
            !log.said(STEP_2_DONE),
            "the step ended with a boss up and no card running:\n  {}",
            log.lines().join("\n  ")
        );

        game.frames_until("the card", CARD_STARTS + 400, || {
            game.state().stage_frames > CARD_STARTS
        });
        game.frames_until("the step ended", 30, || log.said(STEP_2_DONE));
        assert_eq!(
            words(&game, 1),
            vec![(BOMB, 1)],
            "the step after it never got a frame:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And a step can wait for the plate a card's name is on**, which in this game is baked as the card is
/// declared: `EnemyManager::RunEclInstruction` names the record and bakes the plate out of the same instruction,
/// so the frame the card starts is a frame the name is on the screen for.
///
/// What says the plate is up is the visible bit of the vm the name is baked into — `Image::card_name_vm`, the
/// one a restore bakes a name back through — which this game's own draw reads before it draws either; see
/// `docs/th06/asm.md`. The laid-out game sets it as a card starts and clears it as one ends, the way the card's
/// ANM script does.
#[test]
fn a_step_ends_where_th06s_cards_name_is_shown() {
    in_its_own_process(|| {
        let game = playing_to(
            "th06-a-recipe-waiting-for-a-plate",
            &walking_in_to("- play_until: { card_name: true }\n"),
            false,
        );
        let log = game.log();

        game.frames_until("the card", CARD_STARTS + 400, || {
            game.state().stage_frames > CARD_STARTS
        });
        assert_eq!(
            game.card_name_on_the_plate(),
            CARD_NAME,
            "the card's name was not baked onto the plate as the card started",
        );
        game.frames_until("the step ended", 30, || log.said(STEP_2_DONE));
        assert_eq!(
            words(&game, 1),
            vec![(BOMB, 1)],
            "the step after it never got a frame:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **A step asks the run to start over**, which is what orb turns into its question of where to start again —
/// the one way to reach a chapter again without dying for it.
///
/// **The ask waits where there is nothing to ask about**: the step lands on the frame the walk-in ended, which
/// is the stage's first chapter, and the question comes up on the first frame a stage can be retried from. What
/// answers it is a press at `question: retry`, the same as a player pressing at it, so this asserts the
/// question was raised and not what was chosen — `the_bot_playing_th06` is where an item is chosen by name.
#[test]
fn a_step_asks_the_run_of_th06_to_start_over() {
    in_its_own_process(|| {
        let game = playing_to(
            "th06-a-recipe-starting-over",
            &walking_in_to("- start_over: true\n"),
            false,
        );
        let log = game.log();

        game.frames_until("the question of where to start again", 60, || {
            log.said("retry: the run was asked to start over")
        });
        assert!(
            log.said("autoplay: step 2 of 3 asks the run to start over"),
            "the ask was not said:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("where from is the question"),
            "the ask was not turned into the question of where to start over from:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **A step tells the rules what to do about being hit, and the telling is the whole of it.**
///
/// What it changes is every frame after it rather than the frame it is on — the chapter the run is in is what
/// the chance is drawn for — so there is nothing here to wait for, and the frame it costs goes to the rules
/// rather than being a word of nothing. What the chance *does* to a run of this game is not asked here: a hit
/// in the laid-out th06 is an e2e test saying so rather than a bullet the player stood under, so a chapter
/// drawn to be hit in has nothing here to be hit by. See `Fake::puts_a_bullet_on_the_player`.
#[test]
fn a_step_tells_the_rules_over_th06_to_take_hits() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-a-recipe-taking-hits", the_run(), |config| {
            config.autoplay = Some(format!(
                "- take_hits: {{ chance: 0.2, times_each_retry: 0.5, gives_up: 0.1 }}\n{THEN_BOMB}"
            ));
            config.log_level = LogLevel::Verbose;
        });
        let log = game.log();

        let pressed = words(&game, 3);
        assert!(
            log.said(
                "autoplay: step 1 of 2 takes hits, a chance of 0.200 per chapter, 0.500 of that \
                 each retry, and 0.100 of giving the run up at one"
            ),
            "the numbers the step was given were not said:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            log.said("autoplay: step 1 of 2 done"),
            "the step did not end with the telling:\n  {}",
            log.lines().join("\n  ")
        );
        assert_eq!(
            pressed.last().map(|(word, _)| *word),
            Some(BOMB),
            "the step after it never got a frame:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// **And `menu` is a word a recipe can press over this game**, which is the one road it could not take: what
/// is behind that button is the game's own pause menu and everything its items lead to.
///
/// **What this asks is which word it lands in.** The bit is outside every button this game is played with —
/// `Th06::run_input` keeps it out of a run *written down*, so that a resume feeding its frames back does not
/// open a menu instead of playing — and the word a recipe presses is handed to the game inside its own input
/// read. So the press has to reach `g_CurFrameInput`, which is what the game's own pause reads its edge off,
/// and a press that did not is one no menu ever sees.
#[test]
fn the_menu_button_a_recipe_presses_reaches_the_word_th06s_read_leaves() {
    in_its_own_process(|| {
        let game = Fake::attach("th06-a-recipe-opening-the-menu", the_run(), |config| {
            // Held for a moment and let go: the game takes the button's edge, so a word held for ever opens
            // the menu once and a recipe that never released it could not close one either.
            config.autoplay = Some("- press: [menu]\n  frames: 4\n".to_owned());
            config.log_level = LogLevel::Verbose;
        });
        // Two frames and not one: the first is the frame the supervisor builds the front end on, and the
        // last act of that build is the word zeroed — a press already spent, as the game has it — so the
        // word a read leaves standing is the second frame's.
        game.frames(2);

        assert_eq!(
            game.image().input_now() & MENU,
            MENU,
            "the pause button a recipe pressed is in no word the game read:\n  {}",
            game.log().lines().join("\n  ")
        );
    });
}
