//! Midstage chapter boundaries, as script frame numbers.
//!
//! The script clock is the one this game's stages run on, so a frame number here lands on the same point of the
//! wave pattern however fast the run is going and whatever difficulty it is — see
//! [docs/adr/0018](../../../../../docs/adr/0018-th07s-script-clock-is-the-first-of-four-main-contexts.md), which
//! is the reading that settled which of the game's four clocks that is. Boss attacks are not listed: those
//! boundaries are found as the fight is fought.
//!
//! **Nothing is in it yet, and that is the one piece of work this game is still waiting for.** A boundary inside a
//! stage is read off a run with `--chapter-tuning` and off nothing else — `--collect` proposes them over a replay
//! and `--judge` steps between them so each can be looked at — so it cannot be walked out of the exe the way
//! every address in `th07/mod.rs` was. The recipe is *Building the midstage table* in `SPEC.md`, the difficulty to
//! tune on is Lunatic (a gap there is a gap on every difficulty under it), and `chapters.rs` is what a pass writes
//! beside the game for pasting over this file.
//!
//! Until then a chapter of 妖々夢 falls at a stage's start and nowhere inside one, which is a poorer 完全無欠モード
//! and not a broken one: the fight's own boundaries — every attack, every card, the midboss beaten — are found at
//! runtime and are most of what a run is retried at.
//!
//! **The rows are here empty rather than absent** because a pass writes one line per row and reads one back: the
//! table's shape is what makes a stage judgeable at all, so it is settled now and the numbers arrive later. Eight
//! of them, which is what the game's own switch at 0x407081 covers — six stages, the Extra and the Phantasm.

use crate::game::Boundary;

/// Indexed by the stage counted from zero, which is what [`Th07::read_state`](super::Th07) answers: `[0x62f85c]`
/// less one while a stage is playing — see `G_STAGE`.
pub const MIDSTAGE: [&[Boundary]; 8] = [
    /* stage 1  */ &[],
    /* stage 2  */ &[],
    /* stage 3  */ &[],
    /* stage 4  */ &[],
    /* stage 5  */ &[],
    /* stage 6  */ &[],
    /* extra    */ &[],
    /* phantasm */ &[],
];
