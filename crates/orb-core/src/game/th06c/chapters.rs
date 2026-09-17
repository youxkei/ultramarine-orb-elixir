//! Midstage chapter boundaries, as script frame numbers.
//!
//! The script clock is the one this game's stages run on — `enemy_manager::TIMELINE_TIME` in `th06c/mod.rs` —
//! so a frame number here lands on the same point of the wave pattern however fast the run is going and
//! whatever difficulty it is. Boss attacks are not listed: those boundaries are found as the fight is fought.
//!
//! **Every row is empty, and that is where this game's tuning stands rather than an answer for it.** A boundary
//! inside a stage is read off a `--judge` pass over a run of the stage — see `orb_core::tuning` — and no such
//! pass has been made over this exe. So a stage of it is one chapter from its start to its midboss and another
//! from the midboss beaten to the boss, and what divides the rest is the fight's own boundaries, which are most
//! of what a run is retried at.
//!
//! **And not th06 1.02h's rows copied across.** The two are one game to a player, and the stages' scripts
//! may well be byte for byte the same — but a boundary is a measurement of a run and not a reading of the exe,
//! and a number written here that nobody measured against this exe is a number the tuning pass would have to
//! find wrong before it could find its own.

use crate::game::Boundary;

/// Indexed by the stage counted from zero, which is what [`Th06c::read_state`](super::Th06c) answers: the game
/// manager's stage less one while a stage is playing — see `STAGE` in `th06c/mod.rs`. Seven rows, which is
/// what the game has: six stages and the Extra.
pub const MIDSTAGE: [&[Boundary]; 7] = [
    /* stage 1 */ &[],
    /* stage 2 */ &[],
    /* stage 3 */ &[],
    /* stage 4 */ &[],
    /* stage 5 */ &[],
    /* stage 6 */ &[],
    /* extra   */ &[],
];
