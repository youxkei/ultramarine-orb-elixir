//! Midstage chapter boundaries, as script frame numbers — see `Th10::script_frames` for the clock they are
//! frames of: the latest time the stage's own scripts have reached, which stands while the stage's last boss
//! is fought and climbs everywhere else, a midboss's fight included on stages 4, 5 and 6.
//!
//! **What the rows were chosen out of is every boundary the detector proposed** off one `--tune` pass, a
//! Pointdevice run of Normal, Reimu A, 2026-10-01, which walked stages 2 to 6 — a gap of sixty frames with
//! nothing to shoot at and no boss up, at least 120 frames into the chapter before it — nineteen in all:
//! stage 2 `367, 5534`; stage 3 `562, 1926, 2395, 2810, 5239, 5745`; stage 4 `299, 5136, 8443`; stage 5
//! `805, 1428, 2806, 6301, 8415`; stage 6 `595, 1994, 3229`. **And by `th08/chapters.rs`'s rules**, read
//! against where each stage's fights come on — stage 2's midboss at 2553 and its boss at 5864; stage 3's at
//! 3280 and 5885; stage 4's at 3602 and 8601; stage 5's at 3802 and 8607; stage 6's first fight at 3891 —
//! which refused `367, 562, 299, 805, 595` for being under 880 into the stage, `5534, 2810, 5745, 8443, 8415`
//! for being under 590 before a fight, and `2395` for a stretch that wants one row taking the longer lull,
//! `1926`'s 194 frames to its 145.
//!
//! **Stage 1 has none**: a second pass from its start proposed `4142` alone, which is 228 before its boss at
//! 4370, and its stretch to the midboss at 2201 had no gap the detector could propose in. **The Extra has none
//! yet**, no pass having played it.

use crate::game::{Boundary, proposed};

/// Indexed by the stage counted from zero, which is what [`Th10::read_state`](super::Th10) answers: the six
/// stages of a run and the Extra.
pub const MIDSTAGE: [&[Boundary]; 7] = [
    /* stage 1 */ &[],
    /* stage 2 */ &[],
    /* stage 3 */ &[proposed(1926), proposed(5239)],
    /* stage 4 */ &[proposed(5136)],
    /* stage 5 */ &[proposed(1428), proposed(2806), proposed(6301)],
    /* stage 6 */ &[proposed(1994), proposed(3229)],
    /* extra   */ &[],
];
