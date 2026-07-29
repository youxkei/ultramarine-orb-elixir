//! Midstage chapter boundaries, as enemy-timeline frame numbers.
//!
//! The enemy timeline is the clock the stage's ECL script runs on, so these
//! frames land on the same points in the wave pattern regardless of how the
//! player is doing, and regardless of difficulty. Boss attacks are not listed:
//! those boundaries are detected at runtime.
//!
//! Built with `chapter_tuning: true`; see the README.

/// Indexed by `GameManager.currentStage` (0..=5 for stages 1-6, 6 for Extra).
pub const MIDSTAGE: [&[i32]; 7] = [
    /* stage 1 */ &[],
    /* stage 2 */ &[],
    /* stage 3 */ &[],
    /* stage 4 */ &[],
    /* stage 5 */ &[],
    /* stage 6 */ &[],
    /* extra   */ &[],
];
