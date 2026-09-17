//! **What a stage of th06c leaves its player as, which is 270 frames nothing can kill them in.**
//!
//! th06 1.02h's own answer is `the_player_a_stage_starts.rs` and this game has the same one, read off its
//! exe rather than carried over:
//!
//! - **`0x140049de0` is the player's own register.** It reads the character at `0x1403a33bc` and the shot at
//!   `0x1403a33bd`, fills the four speeds at `+0x7380` from the table at `0x140396420`, and writes the state
//!   byte at `+0x73b8` to **1** (`0x14004a072`), the count at `+0x7378` to **120** (`0x14004a08c`) and the
//!   timer at `+0x7374` to **-999** (`0x14004a096`). Its own `cmp DWORD PTR [0x140b7762c],0x3` at
//!   `0x140049e01` is it telling a stage's build from a run's start, so it runs at both.
//! - **The walk-in lasts thirty frames.** `0x14004aaba`'s `cmp r8d,0x1e` is the gate, and past it
//!   `0x14004aac0` onwards writes the state **3**, the count **0xf0**, the timer -999 and six frames of
//!   deathbomb window at `+0x7254`.
//! - **And state 3 counts down to nothing.** `0x14004ab8e` tests the state against 3 and
//!   `0x14004ab9d`–`0x14004aba9` takes one off the count and puts the state back to nought where it runs out.
//!
//! **So the hit test cannot reach the player for 30 + 240 = 270 frames of every stage**, `0x14004c6ce`
//! gating a hit on that byte being nought. Which is what the second test here is about, and what the first
//! one is is the two numbers the register leaves — the state a chapter cannot be taken in, and a count
//! nobody but state 3 reads.
//!
//! **Read off the exe and not off this tree**, because the laid-out game is both the writer and the reader
//! of the memory orb is driven through: a build that left the player playing would be one every e2e test
//! here agreed with, and every one of them would have a player a bullet can reach on a stage's first frame.

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// The stage the run below is played in, orb's own 0-based number for it.
const THE_FIRST_STAGE: i32 = 0;

/// The player's four states, written out here rather than taken from `Th06c`'s own `player` module, which
/// keeps the same numbers: a test that read them off the code under test would agree with it whatever either
/// said.
const PLAYING: u8 = 0;
const COMING_BACK: u8 = 1;
const UNTOUCHABLE: u8 = 3;

/// What the register leaves on the count, what the walk-in leaves there, and how long the walk-in is.
const THE_REGISTER_LEAVES: i32 = 120;
const THE_WALK_IN_LEAVES: i32 = 0xf0;
const THE_WALK_IN_TAKES: u32 = 30;

fn launched(name: &str) -> Box<Fake> {
    Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| config.log_level = LogLevel::Verbose,
    )
}

/// A stage's own build leaves the player coming back, with the register's count under it — and orb reads
/// that as a run in the middle of something.
#[test]
fn a_stage_of_th06c_starts_its_player_coming_back() {
    in_its_own_process(|| {
        let game = launched("th06c-the-players-stage-start");
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);

        // Read before a frame is run, which is what "what the build leaves" means: the register is the
        // stage callback's own and the walk-in is the frames after it.
        assert_eq!(
            (
                game.image().the_players_state(),
                game.image().the_frames_the_player_is_untouchable_for(),
            ),
            (COMING_BACK, THE_REGISTER_LEAVES),
            "the stage's build did not leave the player as its own register leaves them",
        );
        assert!(
            game.state().unsettled,
            "a player coming back reads as a run standing somewhere a chapter can be taken",
        );
    });
}

/// And nothing reaches the player until the walk-in and the invulnerability after it have both run out,
/// which is 270 frames of every stage.
#[test]
fn nothing_of_a_stage_of_th06c_reaches_its_player_for_the_first_270_frames() {
    in_its_own_process(|| {
        let game = launched("th06c-the-players-first-frames");
        game.in_a_run_in_the_stage(THE_FIRST_STAGE);

        // The walk-in, which is the state the build left and the frames it lasts.
        for frame in 0..THE_WALK_IN_TAKES {
            game.frame();
            let state = game.image().the_players_state();
            assert!(
                state == COMING_BACK || (state == UNTOUCHABLE && frame + 1 == THE_WALK_IN_TAKES),
                "the player is in state {state} on frame {} of a walk-in of {THE_WALK_IN_TAKES}",
                frame + 1,
            );
        }
        assert_eq!(
            (
                game.image().the_players_state(),
                game.image().the_frames_the_player_is_untouchable_for(),
            ),
            (UNTOUCHABLE, THE_WALK_IN_LEAVES),
            "the walk-in did not hand the player over to the count the game's own gives it",
        );

        // And the invulnerability after it, run out frame by frame: the player is playing on the frame the
        // count reaches nothing and in no frame before it.
        game.frames_until(
            "the player playing",
            THE_WALK_IN_LEAVES as u32 + THE_WALK_IN_TAKES,
            || game.image().the_players_state() == PLAYING,
        );
        assert_eq!(
            game.state().stage_frames,
            THE_WALK_IN_TAKES + THE_WALK_IN_LEAVES as u32,
            "the player began playing on another frame of the stage than the count says",
        );

        // **And the stage's own fire passed through them**, which is what those frames are for: a bullet
        // aimed at a player who has not moved reaches them inside this window, and the hit test is gated on
        // a state none of it was in.
        assert_eq!(
            game.state().deaths,
            0,
            "something reached the player inside the frames nothing can",
        );
    });
}
