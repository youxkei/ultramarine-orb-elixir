//! **A conversation of th08, which is the one thing that holds the run's clock while the run plays on.**
//!
//! Instruction 148 of a stage's script hands its line to the conversation and adds `0x708` — 1,800 — to the
//! run clock on the spot (0x41d6cd), and the step that clock is counted by is gated on no line being up
//! (0x42c6a1 against 0x4358bb). So the clock takes a number, skips 1,800 of them, and then stands still for
//! as long as the line is shown.
//!
//! **What does not stand still is the run.** A launch of the real game spent 103 updates on frame 7,179 and
//! over them the player walked from `293,368` to `253,352`, four items were collected, the score climbed and
//! the midboss arrived from off the field. Those are frames of a run, and the number they are written down
//! against is one.
//!
//! Which is what `resume::Again` is for, and this file is the half of it the game answers: the clock jumping,
//! the clock standing, the run advancing under it, and orb reading all three off the fields it was given.
//! That the record then keeps every read of such a frame is `resume`'s own unit tests.

use crate::fake::th08::{FIRST_STAGE, Fake};
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the laid-out stage lasts, which is longer than this plays: what is watched is one conversation
/// inside one stage, so the stage must not end under it.
const A_STAGE: u32 = 100_000;

/// Where in the stage the conversation begins and how many updates it lasts. Twenty against the real 103,
/// which is the same shape in a test that runs in a moment.
const A_CONVERSATION_AT: u32 = 120;
const A_CONVERSATION_LASTS: u32 = 20;

/// And what it costs the clock, which is the game's own number: `0x708`.
const THE_CLOCK_A_LINE_COSTS: u32 = 1800;

fn a_stage_with_a_conversation_in_it(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    );
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.stages_last(A_STAGE);
    game.has_a_conversation(A_CONVERSATION_AT, A_CONVERSATION_LASTS);
    game.plays_its_own_stage();
    game
}

#[test]
fn the_clock_a_run_of_th08_is_written_down_against_jumps_1800_and_then_stands() {
    in_its_own_process(|| {
        let game = a_stage_with_a_conversation_in_it("th08-a-conversation");

        // Up to the frame before the line: the clock counts every update, one apiece.
        game.frames_until("the frame before the line", A_STAGE, || {
            game.state().stage_frames + 1 == A_CONVERSATION_AT
        });
        let before = game.state();
        assert_eq!(before.stage, FIRST_STAGE);
        assert!(
            !before.in_dialogue,
            "a line is up before the script starts one"
        );

        // And the frame the script starts one on: the clock is 1,800 further on and the line is up.
        game.one_frame();
        let started = game.state();
        assert!(started.in_dialogue, "the script started no line: {started}");
        assert_eq!(
            started.stage_frames,
            before.stage_frames + THE_CLOCK_A_LINE_COSTS,
            "the clock did not take the 1,800 the instruction adds",
        );

        // **Then it stands, and the run does not.** Every update the line is up reads the same number off
        // the clock, and the generator moves under it — which is a frame of a run and not a frame of
        // nothing.
        let standing = started.stage_frames;
        let mut generators = vec![game.image().the_generators_word()];
        for _ in 1..A_CONVERSATION_LASTS {
            game.one_frame();
            let state = game.state();
            assert!(
                state.in_dialogue,
                "the line went early at {}",
                state.stage_frames
            );
            assert_eq!(
                state.stage_frames, standing,
                "the clock moved while a line was up",
            );
            generators.push(game.image().the_generators_word());
        }
        assert_eq!(
            generators
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            generators.len(),
            "the run drew nothing from its generator while the line was up: {generators:?}",
        );

        // **And the line over, which is still not a frame the clock counts.** The gate is read where the
        // clock is stepped — the enemy manager's own update, priority 0xb — and the line is taken down by
        // the screen's element above it at priority 0xf, so the step on the frame a line ends has already
        // seen it up. The frame after that one is the first the clock counts again.
        game.one_frame();
        let ended = game.state();
        assert!(!ended.in_dialogue, "the line outlasted the conversation");
        assert_eq!(ended.stage_frames, standing);
        game.one_frame();
        assert_eq!(game.state().stage_frames, standing + 1);
    });
}
