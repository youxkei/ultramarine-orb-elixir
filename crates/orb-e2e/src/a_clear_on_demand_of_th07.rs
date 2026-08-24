//! **`--clear` over th07: a run to the ending with nothing able to kill the player.**
//!
//! What `--clear` is for is reaching a result screen without half an hour of playing well, so it is also how the
//! ending, the screen a run ends at and the score file's write on the way out are reached. It fixes the mode
//! rather than asking — `mode: pointdevice to start with; nobody is asked`.
//!
//! **What keeps the player alive is this game's own two fields**, and that is what makes this th07's e2e test
//! rather than a second copy of th06's: `Th07::make_invulnerable` writes the state the respawn walks in and the
//! frames left under it — 3 and 0xf0, out of the five-value machine at `+0x2408` and the timer at `+0x16a08` —
//! and it writes them *every update*, because frames left where the last respawn left them is a state put back to
//! normal in the same update it was written for. Both are read back here, on frame after frame of a run.
//!
//! And what it must not touch is the three states that are the game in the middle of something: dying, being hit,
//! and the border th07 has where th06 has nothing. A launch that wrote over the border would be playing the
//! game rather than keeping the player alive for an update.
//!
//! What these are about is how the run *went*, not how long it took: the wall clock is one machine's and is not
//! written down.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stages a whole run of this game is, counted the way orb counts them: six, and then the ending. The Extra
/// and the Phantasm are runs of their own and are not what a cleared run walks.
const STAGES: i32 = 6;

/// How long each stage of the run is given, in frames: past the frames a stage spends settling before its first
/// chapter, so every stage of the run is a stage that took one.
const STAGE_FRAMES: u32 = 120;

/// How many updates the ending's own script is given before it hands over to the roll's.
const ENDING_UPDATES: i32 = 600;

/// The state the respawn walks in and the frames orb leaves under it, which is what `--clear` writes every
/// update. Written out here rather than read off `Th07`, which keeps both: what an e2e test holds is the pair
/// somebody playing gets, and a test that took its numbers from the code under test would agree with it whatever
/// either said.
const COMING_BACK: i8 = 3;
const FRAMES_LEFT: i32 = 0xf0;

/// A `--clear` launch of this game, with a run in it.
///
/// Two of the things that word sets — the mode fixed and the replay write refused — and not the speed: what this
/// counts is frames of the run, and a launch running 64 updates in each of them counts them in sixty-fourths.
fn clearing(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.fast_clear = true;
            config.block_replay_save = true;
            config.log_level = LogLevel::Verbose;
        },
    );
    game.streams_its_song(0);
    game.builds_a_stage();
    game
}

/// Six stages with nothing able to hit the player, the ending after them, and no death anywhere.
#[test]
fn a_cleared_run_of_th07_reaches_the_ending_with_no_death_in_it() {
    in_its_own_process(|| {
        let game = clearing("th07-a-clear-through-six");
        let log = game.log();
        // Nobody was asked which mode, which is what `--clear` is: it takes the mode it is given.
        assert!(
            log.said("mode: pointdevice to start with; nobody is asked"),
            "the launch asked which mode:\n  {}",
            log.lines().join("\n  ")
        );

        // Every stage of the run, in order, each one a stage that took a chapter of its own.
        for stage in 1..=STAGES {
            game.frames_until(&format!("stage {stage}"), 600, || {
                log.said(&format!("stage {stage} chapter 1 (stage start)"))
            });
            assert_eq!(
                game.state().deaths,
                0,
                "a death was counted before stage {stage} of a run nothing could hit",
            );
            if stage < STAGES {
                game.frames(STAGE_FRAMES);
                game.moves_on_to_the_stage(stage);
            }
        }

        // And then the ending rather than a seventh stage, which is what clearing the last one is: run out inside
        // the frame it began on rather than played, which is what `--clear` does with one.
        game.an_ending_is_running(ENDING_UPDATES);
        game.frames_until("the ending run out", 60, || log.said("ending run out in"));
        assert_eq!(
            game.state().deaths,
            0,
            "a death was counted in a run nothing could hit",
        );
        // Not one, which is the other half: `deaths` is the game's count and this is orb's, and a run that lost
        // the frames of invulnerability would have both.
        assert!(
            !log.said("died in chapter"),
            "orb noticed a death in a run nothing could hit:\n  {}",
            log.lines().join("\n  ")
        );
    });
}

/// The two fields that keep the player alive are written every update, and put back where the game's own tick
/// takes them down.
///
/// **Every update and not once**, which is the whole of the arrangement: the frames left run down by one an
/// update, so a launch that wrote them once would hand the hit test a player it can kill on the update they
/// reach nothing.
#[test]
fn a_cleared_run_of_th07_writes_the_players_own_two_fields_every_update() {
    in_its_own_process(|| {
        let game = clearing("th07-a-clear-the-player");
        let log = game.log();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        // Frame after frame, both fields as orb leaves them — and the game's own tick taking one off the frames
        // between them, which is what makes writing them again per update the thing being asserted.
        for _ in 0..30 {
            assert_eq!(
                game.image().the_players_state(),
                COMING_BACK,
                "the player is not in the state orb writes for a cleared run",
            );
            assert_eq!(
                game.image().the_frames_the_state_has_left(),
                FRAMES_LEFT,
                "the frames left under that state are not the ones orb writes",
            );
            // One off, the way the game's own update takes it: what orb writes next frame is what this asserts.
            game.image().the_player_is_coming_back(FRAMES_LEFT - 1);
            game.one_frame();
        }
    });
}

/// And the border is left where the game put it, which is the state orb has no business finishing.
///
/// th07's own, and the one state of the five that th06 has not got: a hit there is deflected by the game
/// itself, which takes the border down, plays its effect and is part of what a run scores. So a `--clear` that
/// wrote over it would be playing the game.
#[test]
fn a_cleared_run_of_th07_leaves_the_border_where_the_game_put_it() {
    in_its_own_process(|| {
        let game = clearing("th07-a-clear-the-border");
        let log = game.log();
        game.frames_until("the stage's own first chapter", 600, || {
            log.said("stage 1 chapter 1 (stage start)")
        });

        game.image().the_players_border_is_up();
        for _ in 0..30 {
            game.one_frame();
            assert_eq!(
                game.image().the_players_state(),
                4,
                "orb wrote over the border, which is the game's own to take down",
            );
        }
    });
}
