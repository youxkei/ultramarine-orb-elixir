//! Midstage chapter boundaries, as enemy-timeline frame numbers.
//!
//! The enemy timeline is the clock the stage's ECL script runs on, so these
//! frames land on the same points in the wave pattern regardless of how the
//! player is doing, and regardless of difficulty. Boss attacks are not listed:
//! those boundaries are detected at runtime.
//!
//! Built with `orb --collect` and `--judge` over a Lunatic replay of a 1→6 run and an Extra
//! replay, a stage at a time; see *Building the midstage table* in `SPEC.md`. Stages keep whatever
//! is here until one is looked at again, and `tuning.txt` beside the launcher holds the thirty
//! boundaries judged out of this as well, so a stage picked up again starts from what was decided
//! rather than from nothing.
//!
//! `hand` marks the ones somebody put there, which are the numbers nothing would propose again
//! if they were lost. Extra is all of them: the detector's proposals for it were all refused,
//! its waves leaving gaps a second and a half long where a chapter wants somewhere to stand.
//! Which hand it was is read and not merely written down — the shortest a chapter may be lets
//! one of these through where it would refuse a proposal, so a table that only said it in a
//! comment divided a stage differently in play than in the pass that chose it.

use crate::game::{Boundary, hand, proposed};

/// Indexed by the stage counted from zero — 0..=5 for stages 1-6, 6 for Extra —
/// which is `GameManager.currentStage` less one; see its comment for why.
pub const MIDSTAGE: [&[Boundary]; 7] = [
    /* stage 1 */ &[hand(4472)],
    /* stage 2 */ &[proposed(880), hand(4597)],
    /* stage 3 */ &[proposed(1009), proposed(2653)],
    /* stage 4 */
    &[
        hand(2341),
        proposed(3395),
        hand(7467),
        proposed(8328),
        proposed(9739),
    ],
    /* stage 5 */ &[hand(2363), proposed(6827)],
    /* stage 6 */ &[hand(1535)],
    /* extra   */ &[hand(2649), hand(3728), hand(5448), hand(7356)],
];
