//! Midstage chapter boundaries, as script frame numbers.
//!
//! The script clock is the one this game's stages run on, so a frame number here lands on the same point of the
//! wave pattern however fast the run is going and whatever difficulty it is — see
//! [docs/adr/0018](../../../../../docs/adr/0018-th07s-script-clock-is-the-first-of-four-main-contexts.md), which
//! is the reading that settled which of the game's four clocks that is. Boss attacks are not listed: those
//! boundaries are found as the fight is fought.
//!
//! Built with `orb --judge` over a Lunatic replay, a stage at a time; see *Building the midstage table* in
//! `SPEC.md`. A boundary inside a stage is read off such a run and off nothing else, so it cannot be walked out
//! of the exe the way every address in `th07/mod.rs` was. `chapters.rs` beside the game is what a pass writes for
//! pasting over this file, and `tuning.txt` beside it holds the same numbers for the next sitting.
//!
//! **Every one of them is a hand's**, which is this game's own answer and not a preference: the detector proposes
//! a boundary a second into a gap with nothing to shoot at, and th07 has no such gaps — a pass over its second
//! stage measured a floor of *one* enemy alive, with stretches of 450 and 929 frames at that floor and never
//! zero. So `--collect` proposes nothing here and every number below was put there by somebody watching the
//! stage. Which hand it was is read and not merely written down — the shortest a chapter may be lets one of these
//! through where it would refuse a proposal.
//!
//! **The stages with nothing in them are stages nobody has judged yet**, not stages without boundaries: a run
//! carries on dividing them at a stage's start and at the fight's own boundaries — every attack, every card, the
//! midboss beaten — which are found at runtime and are most of what a run is retried at.

use crate::game::{Boundary, hand};

/// Indexed by the stage counted from zero, which is what [`Th07::read_state`](super::Th07) answers: `[0x62f85c]`
/// less one while a stage is playing — see `G_STAGE`. Eight rows, which is what the game's own switch at
/// 0x407081 covers — six stages, the Extra and the Phantasm.
pub const MIDSTAGE: [&[Boundary]; 8] = [
    /* stage 1  */ &[hand(1388), hand(2098)],
    /* stage 2  */ &[hand(1549), hand(5566)],
    /* stage 3  */ &[],
    /* stage 4  */
    &[
        hand(1806),
        hand(3060),
        hand(4338),
        hand(10104),
        hand(11445),
        hand(13220),
    ],
    /* stage 5  */ &[hand(2577)],
    /* stage 6  */ &[],
    /* extra    */
    &[hand(1707), hand(2636), hand(3893), hand(5452), hand(7057)],
    /* phantasm */
    &[hand(1605), hand(2542), hand(3748), hand(5339), hand(6939)],
];
