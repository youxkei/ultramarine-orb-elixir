//! A practice run of th10: one stage played on its own, which is a run orb rewinds and a run it writes
//! nothing down for — what `a_practice_run_of_th07` is for that game.
//!
//! **Both halves are the same fact from two sides.** A practice run is a run — the chapters, the snapshots and
//! the retry menu are a stage's whether the stage was reached by playing to it or chosen from a menu. What it
//! has not got is a name: `Game::run_slot` answers `None` for it, so there is no file for the chapter it is in,
//! and nothing to offer the next time the same stage is practised.
//!
//! Which for this game rests on one bit of the run's flags, the one the main menu's practice item sets (see
//! `Th10`'s `RUN_IS_A_PRACTICE`): a run whose bit is not read is a practice run written down as a whole one — a
//! file offering to pick up a stage that was never part of a run.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_sim::keys;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long the laid-out stage lasts here, which is longer than this test plays.
const A_STAGE: u32 = 100_000;

#[test]
fn a_practice_run_of_th10_is_rewound_and_nothing_is_written_down_for_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-practice",
            Work::wandering(WORK_US),
            |config| config.log_level = LogLevel::Verbose,
        );
        let log = game.log();
        game.fires_at_the_player(false);
        game.stages_last(A_STAGE);
        game.plays_its_own_stage();

        // ── 1. The run, with the flag the main menu's practice item sets.
        game.its_title_menu_is_up();
        game.image().the_run_becomes_a_practice();
        game.is_in_a_run();
        assert!(game.state().practice, "the run reads as a whole run");

        // ── 2. Its chapter is a stage's like any other's: the stage's own start.
        game.frames_until("the stage's first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });
        let at_the_chapter = game.state();

        // ── 3. The hit, and the chapter put back: what the mode promises is the memory as it was, which for a
        // practice run is the same mechanism and the same snapshot as for a whole one.
        game.the_player_is_hit();
        game.frames_until("the death seen", 16, || log.said("died in chapter 1"));
        game.press_until(keys::Z, "the retry menu answered", || {
            log.said("retry: the chapter again on the keyboard")
        });
        let state = game.state();
        assert_eq!(
            (state.stage, state.lives, state.practice),
            (at_the_chapter.stage, at_the_chapter.lives, true),
            "the practice run is not where the chapter began",
        );
        assert!(state.playing, "the run was not left playing");

        // ── 4. And nothing of it on the disk. A run with no slot is not looked for and not written down:
        // nothing was asked about where to start it — which is what says the file was never even looked for —
        // and the chapter it has reached has left nothing behind.
        game.frames(60);
        assert!(
            !log.said("resume: nothing was left of"),
            "a practice run was looked up as a run that could have been left:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            !log.said("resume: stage 1 chapter"),
            "a chapter of a practice run was written down:\n  {}",
            log.lines().join("\n  ")
        );
        assert!(
            orb_core::resume::left(game.dir()).is_empty(),
            "a practice run left a chapter behind: {:?}",
            orb_core::resume::left(game.dir()),
        );
    });
}
