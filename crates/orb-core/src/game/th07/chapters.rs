//! Midstage chapter boundaries, as script frame numbers.
//!
//! The script clock is the one this game's stages run on, so a frame number here lands on the same point of the
//! wave pattern however fast the run is going and whatever difficulty it is; which of the game's four clocks
//! that is, is settled beside `SCRIPT_FRAMES` in `th07/mod.rs`. Boss attacks are not listed: those
//! boundaries are found as the fight is fought.
//!
//! Built with `orb --judge` over a Lunatic replay, a stage at a time; see `orb_core::tuning`. A boundary inside a stage is read off such a run and off nothing else, so it cannot be walked out
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
//! **Stage 3 and stage 6 carry none, and that is the answer for them rather than a row left blank.** Their
//! midsections want no boundary of their own, so each is one chapter from the stage's start to its midboss and
//! another from the midboss beaten to the stage's own boss. What divides the rest of either is the fight's own
//! boundaries — every attack, every card, the midboss beaten — which are found as the fight is fought and are
//! most of what a run is retried at.

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
