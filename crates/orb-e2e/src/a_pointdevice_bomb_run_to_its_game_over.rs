//! A run of 完全無欠ボムモード on Hard played out of lives: each hit without the press half a life, and the one
//! that lands at nought lives the game's own game over at once — no retry offered, and the chapter kept for the
//! run discarded with it. On Hard and not Lunatic, because a whole life a hit is where the two rules for nought
//! lives agree: what is shown is that a part of a life is not waited for there.
//!
//! In its own file and process beside `a_pointdevice_bomb_run.rs`, the run being a game's whole run.

use crate::fake::th06::{A_BOMB_IS_TAKEN_FOR, Fake};
use crate::fake::{Launched, in_its_own_process};
use orb_core::game::RunStart;
use orb_sim::keys;

/// The run this e2e test plays: Normal, Reimu A, from stage one, kept under `normal-reimu-a`.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
        mode: 0,
    }
}

/// Frames played after a retry before the next hit: the frame a chapter is put back on is one orb holds
/// nothing of the frame before against, so a hit on it is a count moving that nothing compares.
const AFTER_A_RETRY: u32 = 4;

#[test]
fn a_bomb_mode_run_out_of_lives_is_the_games_own_game_over() {
    in_its_own_process(|| {
        let game = Fake::attach("pointdevice-bomb-hard", the_run(), |_| {});
        let log = game.log();
        game.in_a_pointdevice_bomb_run(1);
        assert!(log.said("bomb: the difficulty is hard"));
        assert_eq!(game.state().lives, 2);

        // The run's chapters are kept apart from Pointdevice mode's.
        let kept = game
            .dir()
            .join("pointdevice_bomb_resume")
            .join("normal-reimu-a.msgpack");
        let files = game.sim().files();
        game.frames_until("the chapter kept", 60, || files.holds(&kept));
        assert!(
            !files.holds(
                game.dir()
                    .join("pointdevice_resume")
                    .join("normal-reimu-a.msgpack")
            ),
            "the bomb mode's chapter went where Pointdevice mode's go",
        );

        // ── Four hits without the press, half a life each, and the chapter retried after every one: the lives
        // go down a whole one at every second.
        for lives in [2, 1, 1, 0] {
            let mark = log.written();
            game.hit();
            game.frames(A_BOMB_IS_TAKEN_FOR as u32 + 2);
            assert!(
                log.said_since(mark, "died in chapter 1"),
                "the death was not settled: {:?}",
                &log.lines()[mark..],
            );
            game.press_until(keys::Z, "the retry menu answered", || {
                log.said_since(mark, "retry: the chapter again on the keyboard")
            });
            assert_eq!(game.state().lives, lives, "the life was not taken");
            game.frames(AFTER_A_RETRY);
        }

        // ── And the fifth, at nought lives: the game over then and there, with no half a life waited for — nothing
        // is offered, and the game's own death plays out into its game over.
        let mark = log.written();
        game.hit();
        game.frames(A_BOMB_IS_TAKEN_FOR as u32 + 2);
        assert!(
            log.said_since(mark, "and the run has no life to give"),
            "the hit was not judged a game over: {:?}",
            &log.lines()[mark..],
        );
        assert!(
            !log.said_since(mark, "died in chapter"),
            "a retry was offered to a run with no lives",
        );
        game.frames_until("the run over", 900, || !game.state().in_run);
        game.frames_until("the kept chapter discarded", 120, || !files.holds(&kept));
    });
}
