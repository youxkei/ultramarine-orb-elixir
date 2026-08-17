//! **Moving between the stages of a replay of 妖々夢, which is what a `--collect` pass spends its time doing.**
//!
//! The across key held with next or back is the other half of the hand `stepping_inside_a_stage_of_th07.rs` is
//! about: it leaves the stage altogether and asks the game to start the replay at the one either side. What
//! 妖々夢 answers that with is its own — `Th07::jump_to_stage` reads the replay's header for a record of that
//! stage and asks the supervisor for the scene a stage is rebuilt in — so a stage the replay does not cover is a
//! move that must not happen: the load would fail inside the game and drop it to its menu rather than saying so.
//!
//! **What a laid-out 妖々夢 cannot say is that the replay plays out the same way afterwards.** 紅魔郷's own e2e
//! test compares two passes over one stage update for update, and what makes that possible is a record of inputs
//! its laid-out game feeds; nothing of 妖々夢's replay records is laid out here, and inventing one would be this
//! game claiming to know a format the exe has not been read for. What is left, and what this is, is the move
//! itself: asked for where the replay has that stage, refused where it has not, and the pass carrying on in the
//! stage it arrives in.
//!
//! And the recording's own terminator is not one of these either, for a reason that is written where it is
//! declined: `Th07::hooks` says 妖々夢's teardown writes no entry and its recorder refuses a frame during
//! playback, so there is nothing here to hold back.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The two keys a stage move is made of, as orb fixes them: the across key held, and next or back pressed.
/// `orb_config::keys` is where their numbers are, orb's own names for them being its own.
const ACROSS: u8 = orb_config::keys::SHIFT.0;
const NEXT: u8 = orb_config::keys::RIGHT.0;
const BACK: u8 = orb_config::keys::LEFT.0;

/// The two stages this replay covers, counted from zero the way everything above `Game` counts.
const FIRST: i32 = 0;
const SECOND: i32 = 1;

/// How far into a stage these go before moving out of it: past the frames its own start is a chapter at, so the
/// stage being left is one that was being played.
const INTO_THE_STAGE: u32 = 120;

/// A launch watching a replay of its own stages with the stepping keys on, which is the only configuration a
/// stage move happens in — `--collect` being what it is for.
fn watching(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.chapter_stepping = true;
            config.during_replay = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    let log = game.log();
    game.streams_its_song(0);
    // A replay covering the two stages below and no others, which is what the move is asked against.
    game.image().a_replay_covers_the_stage(FIRST);
    game.image().a_replay_covers_the_stage(SECOND);
    game.watches_a_replay();
    game.frames_until("the stage's own first chapter", 600, || {
        log.said("stage 1 chapter 1 (stage start)")
    });
    game
}

/// The across key held while `key` is pressed, which is one stage move.
fn moves(game: &Fake, key: u8) {
    game.keyboard().set(ACROSS, true);
    game.keyboard().set(key, true);
    game.one_frame();
    game.keyboard().set(key, false);
    game.keyboard().set(ACROSS, false);
    game.one_frame();
}

#[test]
fn a_move_between_the_stages_of_a_replay_of_th07_is_asked_of_the_game_and_the_pass_carries_on() {
    in_its_own_process(|| {
        let game = watching("th07-a-replay-between-stages");
        let log = game.log();
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });

        // Out to the next stage: asked of the game, which is the scene a stage is rebuilt in — and then built,
        // which is the game's own half of the move.
        let from = log.written();
        moves(&game, NEXT);
        assert!(
            !log.said_since(from, "step: the replay has no stage"),
            "the move to a stage the replay covers was refused:\n  {}",
            log.lines().join("\n  ")
        );
        game.moves_on_to_the_stage(SECOND);
        game.frames_until("the second stage's own first chapter", 600, || {
            log.said_since(from, "stage 2 chapter 1 (stage start)")
        });
        assert_eq!(
            game.state().stage,
            SECOND,
            "the pass is not in the stage it moved to",
        );

        // And back again, which is the move the same hand makes the other way.
        let from = log.written();
        game.frames_until("the second stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });
        moves(&game, BACK);
        assert!(
            !log.said_since(from, "step: the replay has no stage"),
            "the move back to a stage the replay covers was refused:\n  {}",
            log.lines().join("\n  ")
        );
        game.moves_on_to_the_stage(FIRST);
        game.frames_until("the first stage's own first chapter again", 600, || {
            log.said_since(from, "stage 1 chapter 1 (stage start)")
        });
        assert_eq!(
            game.state().stage,
            FIRST,
            "the pass is not in the stage it moved back to",
        );
    });
}

/// And a stage the replay does not cover: the move is refused, said out loud, and nothing of the pass moves.
///
/// Which is the half that cannot be left to the game: the load of a stage a replay has no record of fails inside
/// 妖々夢 and drops it to its menu, so what orb must not do is ask.
#[test]
fn a_move_to_a_stage_a_replay_of_th07_does_not_cover_is_refused() {
    in_its_own_process(|| {
        let game = watching("th07-a-replay-past-its-stages");
        let log = game.log();
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });

        // Out to the second stage, which the replay covers, and then out again to the third, which it does not.
        let from = log.written();
        moves(&game, NEXT);
        game.moves_on_to_the_stage(SECOND);
        game.frames_until("the second stage's own first chapter", 600, || {
            log.said_since(from, "stage 2 chapter 1 (stage start)")
        });

        let from = log.written();
        let before = game.state();
        moves(&game, NEXT);
        assert!(
            log.said_since(from, "step: the replay has no stage 3"),
            "the move past the replay's own stages was not refused out loud:\n  {}",
            log.lines().join("\n  ")
        );
        // And the pass is where it was: the stage it is in, and the frames of it that have run.
        let after = game.state();
        assert_eq!(after.stage, before.stage, "the pass left the stage anyway");
        assert!(
            after.stage_frames >= before.stage_frames,
            "the stage's own clock went back over a move that never happened",
        );
    });
}
