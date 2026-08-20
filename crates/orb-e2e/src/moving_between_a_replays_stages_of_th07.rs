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

/// **Nothing of the stage left behind is left to go back into, and nothing of it walls the stage arrived
/// in.**
///
/// A stage's chapters are snapshots of memory that stage had: the game loads a stage's own on the way in
/// and releases what the last one had, so a snapshot of the stage before this one names objects that are
/// gone. `Chapters` drops them as a stage *begins* — and a stage begins a few frames after it is built,
/// once its own start has settled — which leaves the frames in between, where the pass is in the stage
/// arrived at and the chapters kept are the ones it left.
///
/// **A launch walked into it.** The across key took a pass out of a stage whose fight it had just won, and
/// on the frame after: the wall said `stage 2 has ended`, the fight of the stage *before* it still being
/// won as far as anything knew; and a step back put the first stage's own start back into a process that
/// had loaded the second — `step: held at the stage's start (script 9)`, the game in a stage it had torn
/// down, everything drawn over green, and 妖々夢 faulting inside itself an update later.
#[test]
fn a_stage_left_behind_of_th07_is_neither_gone_back_into_nor_a_wall_in_the_stage_arrived_at() {
    in_its_own_process(|| {
        let game = watching("th07-a-replay-leaving-a-stage");
        let log = game.log();
        game.frames_until("the stage played into", 600, || {
            game.state().stage_frames > INTO_THE_STAGE
        });

        // The fight this stage ends with, won: what the wall is for, and what must not still be true of the
        // stage after it.
        game.the_stages_own_boss_arrives();
        game.frames_until("the fight's own chapter", INTO_THE_STAGE, || {
            log.said("chapter 2")
        });
        game.the_boss_is_beaten();
        game.frames(2);

        // Out to the next stage, and the frames before its own start has settled: the pass is in the stage it
        // moved to and what it holds is still the stage it left.
        let from = log.written();
        game.forget();
        moves(&game, NEXT);
        game.moves_on_to_the_stage(SECOND);
        assert!(
            !log.said_since(from, "stage 2 has ended"),
            "the stage moved to was walled by the fight of the stage left behind:\n  {}",
            log.lines().split_off(from).join("\n  "),
        );
        // And no wash over the stage arrived in: a stage's own start is the one boundary that gets none,
        // and the cause a wash goes by was the fight of the stage left behind. Over the frames the new
        // stage's start settles across, those being the ones a wash would be drawn on.
        game.frames(30);
        let washed = game.drawn().clears.clone();
        assert!(
            washed.is_empty(),
            "the field was washed over the start of the stage moved to: {washed:#010x?}",
        );

        // And a step back, which is where the stage left behind would come back: what it finds is this
        // stage's own start and nothing behind that, the chapters of the stage left being none of this
        // one's.
        game.press(BACK);
        assert_eq!(
            game.state().stage,
            SECOND,
            "a step back went into the stage that was left behind:\n  {}",
            log.lines().split_off(from).join("\n  "),
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
