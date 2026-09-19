//! Midstage chapter boundaries, as script frame numbers.
//!
//! The script clock is the one this game's stages run on — `enemy_manager::TIMELINE_TIME` in `th06c/mod.rs` —
//! so a frame number here lands on the same point of the wave pattern however fast the run is going and
//! whatever difficulty it is. Boss attacks are not listed: those boundaries are found as the fight is fought.
//!
//! **Every row is th06 1.02h's, carried across.** The two are one game to a player — this is its port — and
//! the stages play on the same clock: a boundary of th06's table is a wave gap of this game's stage on the
//! same script frame. The marks are carried with the numbers, since the shortest a chapter may be reads them.
//!
//! **Carried rather than found again by a `--collect` pass**, which is what `orb_core::tuning` would make of
//! this exe: the pass proposes a second into every gap and a person judges each one, and those rows are what
//! that sitting came to on th06 — the Extra's four and stage 1's being hands nothing would propose again.
//! What is owed of this exe is only that each lands where it did there, which a run of each game stopped on
//! the boundary answers.
//!
//! **And each one does**, held against th06 1.02h on 2026-09-26 by `midstage-on-th06c.yaml` and
//! `midstage-on-th06.yaml` beside this file: a Normal run of each game to stage 6, stopped on every chapter
//! a row begins and the screen taken. All thirteen of stages 1 to 6 began the chapter th06 numbers them as, on the same
//! script frame — stage 5's second on frame 7067 of the stage in both, the midboss having held the clock
//! 240 frames — and each pair of pictures is one scene: the same background at the same scroll, the
//! enemies the table's gap is between standing where they stand in the other, and a field as full or as
//! empty. The enemies and bullets alive a frame either side agree to within what the two runs' own play
//! makes of them — two enemies and four bullets at stage 5's first in both, four and seven against four
//! and eight at stage 6's. The Extra's four, by `extra-on-th06c.yaml` and `extra-on-th06.yaml`, the same:
//! chapters 2, 3, 8 and 9 in both on the row's frames, the midboss between them arriving on script frame
//! 4741 at frame 5114 of the stage in each, and the fairies at the third coming in across the top of the
//! field in one formation.
//!
//! **Made on the build of md5 acadbc0c and made again on the one Steam installed after it**, md5 3d01e7d9:
//! the update changed the exe and none of the data files, and `midstage-on-th06c.yaml` driven over the new
//! build wrote the thirteen `tl` lines of stages 1 to 6 identical to the old build's, chapter, stage frame
//! and script frame alike.

use crate::game::{Boundary, hand, proposed};

/// Indexed by the stage counted from zero, which is what [`Th06c::read_state`](super::Th06c) answers: the game
/// manager's stage less one while a stage is playing — see `STAGE` in `th06c/mod.rs`. Seven rows, which is
/// what the game has: six stages and the Extra.
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
