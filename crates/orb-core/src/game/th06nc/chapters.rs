//! Midstage chapter boundaries, as script frame numbers.
//!
//! **Every row is empty, and that is where this game's tuning stands rather than an answer for it.** A
//! boundary inside a stage is read off a `--judge` pass over a run of the stage — see
//! `orb_core::tuning` — and no such pass has been made over this exe. So a stage of it is one chapter
//! from its start to its midboss and another from the midboss beaten to the boss, and what divides the
//! rest is the fight's own boundaries, which are most of what a run is retried at.
//!
//! **And not th06 1.02h's rows copied across, nor th06c's.** 「東方紅魔郷: New Classic」 is the same game
//! to a player and a rewritten one in the exe — its game manager is the static `0x1404f1e60` where
//! th06c's is `0x1403a33a0`, its bullet is `0x620` bytes, and a hit is a circle about the player where
//! th06c's is a box. A boundary is a measurement of a run and not a reading of the exe, so a number
//! written here that nobody measured against this exe is a number the tuning pass would have to find
//! wrong before it could find its own.
//!
//! **And the clock these are counted on is the one th06c's are**: script frames, the clock the stage's
//! script runs on, so that a number lands on the same point of the wave pattern whatever the difficulty.
//! This game keeps it at `enemy_manager::TIMELINE_TIME` — the enemies' own job steps it in its last four
//! instructions and the spawner holds the timeline's cursor against it — and `State::script_frames` is
//! what reads it, which is the field a `--judge` pass writes a row out of.

use crate::game::Boundary;

/// Indexed by the stage counted from zero, which is what [`Th06nc::read_state`](super::Th06nc)
/// answers: the game manager's stage less one while a stage is playing — see `STAGE` in `th06nc/mod.rs`.
/// Seven rows, which is what the game has: six stages and the Extra.
pub const MIDSTAGE: [&[Boundary]; 7] = [
    /* stage 1 */ &[],
    /* stage 2 */ &[],
    /* stage 3 */ &[],
    /* stage 4 */ &[],
    /* stage 5 */ &[],
    /* stage 6 */ &[],
    /* extra   */ &[],
];
