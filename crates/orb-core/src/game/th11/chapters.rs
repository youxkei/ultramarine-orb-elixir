//! Midstage chapter boundaries, as script frame numbers — see `Th11::script_frames` for the clock they are
//! frames of: the frames of the stage none of its own scripts was held on, standing while they wait out a boss.
//!
//! **What the rows were chosen out of is every boundary the detector proposed** off one `--tune` pass, a
//! Pointdevice run of Normal, Reimu and Yukari at `--pace=16`, 2026-10-10 (`tune-n2`), which walked stages 1 to 6 —
//! twenty-five gaps in all, proposed at stage 1 `1005`; stage 2 `957, 4802, 5008, 5385`; stage 3 `823`; stage 4
//! `4930, 6993, 8068`; stage 5 `419, 767, 1052, 3841, 10017, 10327, 10607`; stage 6 `415, 6468`. **And by
//! `th08/chapters.rs`'s rules**, read against where each stage's fights come on by the same clock — stage 1's midboss
//! at 2111 and its boss at 5590; stage 2's at 2521 and 5440; stage 3's at 2200 and 3760; stage 4's two midbosses at
//! 381 and 3111 and its boss at 8320; stage 5's midboss at 5580 and its boss at 10660; stage 6's at 3160 and 6610 —
//! which refused `823, 419, 767, 415` for being under 880 into the stage, and `5008, 5385, 8068, 10327, 10607, 6468`
//! for being under 590 before a fight.
//!
//! **Stage 3 and stage 6 have none** for that. **The Extra's are a second pass's**, a whole Pointdevice Extra by the
//! same rules (`tune-x1`, 2026-10-10), which proposed `2273, 5750` against its midboss at 4160 and its boss at 7740,
//! and both are kept.

use crate::game::{Boundary, proposed};

/// Indexed by the stage counted from zero, which is what [`Th11::read_state`](super::Th11) answers: the six
/// stages of a run and the Extra.
pub const MIDSTAGE: [&[Boundary]; 7] = [
    /* stage 1 */ &[proposed(1005)],
    /* stage 2 */ &[proposed(957), proposed(4802)],
    /* stage 3 */ &[],
    /* stage 4 */ &[proposed(4930), proposed(6993)],
    /* stage 5 */ &[proposed(1052), proposed(3841), proposed(10017)],
    /* stage 6 */ &[],
    /* extra   */ &[proposed(2273), proposed(5750)],
];
