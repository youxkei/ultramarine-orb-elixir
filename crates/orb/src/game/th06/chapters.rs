//! Midstage chapter boundaries, as enemy-timeline frame numbers.
//!
//! The enemy timeline is the clock the stage's ECL script runs on, so these
//! frames land on the same points in the wave pattern regardless of how the
//! player is doing, and regardless of difficulty. Boss attacks are not listed:
//! those boundaries are detected at runtime.
//!
//! Built with `orb-launcher --collect` and `--judge` over a Lunatic replay of a 1→6 run and
//! an Extra replay, a stage at a time; see the README. Stages keep whatever is here until one
//! is looked at again, and `tuning.txt` beside the launcher holds the thirty boundaries judged
//! out of this as well, so a stage picked up again starts from what was decided rather than
//! from nothing.
//!
//! `by hand` marks the ones somebody put there, which are the numbers nothing would propose
//! again if they were lost. Extra is all of them: the detector's proposals for it were all
//! refused, its waves leaving gaps a second and a half long where a chapter wants somewhere
//! to stand.

/// Indexed by the stage counted from zero — 0..=5 for stages 1-6, 6 for Extra —
/// which is `GameManager.currentStage` less one; see its comment for why.
pub const MIDSTAGE: [&[i32]; 7] = [
    /* stage 1 */ &[4472 /* by hand */],
    /* stage 2 */ &[880, 4597 /* by hand */],
    /* stage 3 */ &[1009, 2653],
    /* stage 4 */ &[2341 /* by hand */, 3395, 7467 /* by hand */, 8328, 9739],
    /* stage 5 */ &[2363 /* by hand */, 6827],
    /* stage 6 */ &[1535 /* by hand */],
    /* extra   */ &[2649 /* by hand */, 3728 /* by hand */, 5448 /* by hand */, 7356 /* by hand */],
];
