//! **A bomb of th11 is not a frame orb keeps away from either**, and the chapter that begins under one says
//! so — what `a_chapter_under_a_bomb_of_th10` holds that game to.
//!
//! `chapter::guarded` is the list of frames no chapter may begin on, and a bomb is deliberately not on it, for
//! the reason its comment records. What this game brings of its own is the word: th11 keeps a bomb running in
//! the bomb object's `+0x3c` (0x40652e), and orb reads a bomb off that word and off nothing else. So a boundary the
//! fight really has begins its chapter whatever the player has just spent, and the bomb turns up in the line that
//! carries the frame's state.

use crate::fake::th11::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// A stage with room for the midboss: her two attacks its second and third chapters.
const A_STAGE: u32 = 3_600;

/// The power the run is given, the laid-out shot type's most and four bombs of it: a fresh run's first stage has
/// none to let one go with, a bomb being power here (0x4311ac through 0x4311f9).
const POWER: i32 = 80;

#[test]
fn a_chapter_of_a_fight_of_th11_begins_while_a_bomb_is_going_off() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th11-a-bomb",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();
        game.its_title_menu_is_up();
        game.is_in_a_run();
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        let (score, _, lives) = game.image().the_runs_numbers();
        game.image().the_run_has(score, POWER, lives);

        // The midboss's first attack, which is the chapter the bomb is let go inside.
        game.frames_until("the midboss's first attack", A_STAGE, || {
            log.said("stage 1 chapter 2 ")
        });

        // The bomb: a bomb fewer on the power, and the word orb reads a bomb off up in the game's own memory.
        let bombs_before = game.state().bombs;
        game.a_bomb_goes_off();
        game.one_frame();
        let state = game.state();
        assert_eq!(state.bombs, bombs_before - 1, "the bomb was not spent");
        assert!(
            state.bombing,
            "the bomb orb reads is not going off: {state}"
        );

        // And the boundary the fight has while it is still going off begins its chapter all the same.
        game.frames_until("the midboss's second attack", A_STAGE, || {
            log.said("stage 1 chapter 3 ")
        });
        assert!(
            game.state().bombing,
            "the bomb was over by the frame the chapter began, so nothing was begun under one",
        );
        game.frames_until("a state line written under the bomb", 120, || {
            log.lines()
                .iter()
                .any(|line| line.contains(" bombing clears="))
        });

        // And the bomb ending is the run going on, with no chapter of its own on the frame it does.
        game.the_bomb_is_over();
        game.one_frame();
        assert!(!game.state().bombing, "the bomb did not end");
        let after = game.state().stage_frames;
        let ended = log.written();
        game.frames(10);
        assert_eq!(
            game.state().stage_frames,
            after + 10,
            "the game stopped being played when the bomb ended",
        );
        assert!(
            !log.said_since(ended, "chapter 4 at frame"),
            "a bomb ending began a chapter of its own:\n  {}",
            log.lines().join("\n  ")
        );
    });
}
