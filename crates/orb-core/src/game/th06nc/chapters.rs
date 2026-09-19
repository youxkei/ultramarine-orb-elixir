//! Midstage chapter boundaries, as script frame numbers.
//!
//! **The clock these are counted on is the one th06c's are**: script frames, the clock the stage's script
//! runs on, so that a number lands on the same point of the wave pattern whatever the difficulty. This game
//! keeps it at `enemy_manager::TIMELINE_TIME` — the enemies' own job steps it in its last four
//! instructions and the spawner holds the timeline's cursor against it — and `State::script_frames` is
//! what reads it.
//!
//! **Every row is th06c's, moved onto this game's clock.** 「東方紅魔郷: New Classic」 is the same game to a
//! player — the same waves in the same order, the same midboss and boss attacks — and so the same places
//! to divide a stage at. But its stages do not run on th06c's numbers: its waves come on earlier, by a
//! different number of frames in each stage and from one wave to the next inside one — the first wave of
//! stage 1 28 frames before th06c's, the midboss 126, the boss 186; stage 3's midboss 330 early, stage 4's
//! 74, the Extra's 100. So th06c's numbers copied across would each fall somewhere else in the wave
//! pattern, and each row here is th06c's moved by the one distance that keeps it where it was: its
//! distance from the next wave to come on. The marks are carried with the numbers.
//!
//! **Moved by the waves either side, and not by a `--judge` sitting over this exe**, which is what
//! `orb_core::tuning` would make of it: a person's judgement of each gap, which th06c's rows already are
//! — they are th06 1.02h's, whose sitting they came out of, held against th06 on the real game. What is
//! owed of this game is only that each lands in the same place of the stage.
//!
//! **Where each goes was read off a Normal run of each game** played by the rules under `--tune`, on
//! 2026-09-27: `orb-real spawns` over each launch's trace prints the script frames the enemy count rose
//! on, which are the stage's script's alone whatever the run's play, and the two lists laid side by side
//! say how far each wave moved. stage 1's row sits between a wave that ends on 4299 and one that begins on
//! 4499 in th06c, 4173 and 4373 here; stage 3's second between 2523 and 2823, here 2323 and 2623; the
//! Extra's between waves each 100 earlier here, as its midboss and boss are.
//!
//! **And each one lands where th06c's does**, held against th06c on 2026-09-27 by `midstage-on-th06nc.yaml`
//! and `extra-on-th06nc.yaml` beside this file against th06c's own two: a Normal run of each game stopped
//! on every chapter a row begins and the screen taken. All thirteen of stages 1 to 6 began the chapter th06c
//! numbers them as, on this table's frame — this game's midbosses taking as many chapters as th06c's — and
//! each pair of pictures is one scene. The enemies and bullets alive on the boundary agree to within what
//! the two runs' play makes of them: 1 and 7 against 1 and 8 at stage 1's, 0 and 63 against 0 and 64 at
//! stage 4's second, 2 and 0 in both at its third; the widest, 3 enemies against 6 at stage 2's second, is
//! the last of a wave the rules had not yet shot down. The Extra's four the same: chapters 2, 3, 8 and 9 in
//! both on the row's frames, 1 and 1 in both at the first, 0 and 77 against 0 and 78 at the second, and the
//! fairies at the third coming down both sides of the field over one wall of bullets, 14 and 488 against
//! 18 and 462.

use crate::game::{Boundary, hand, proposed};

/// Indexed by the stage counted from zero, which is what [`Th06nc::read_state`](super::Th06nc)
/// answers: the game manager's stage less one while a stage is playing — see `STAGE` in `th06nc/mod.rs`.
/// Seven rows, which is what the game has: six stages and the Extra.
pub const MIDSTAGE: [&[Boundary]; 7] = [
    /* stage 1 */ &[hand(4346)],
    /* stage 2 */ &[proposed(790), hand(4507)],
    /* stage 3 */ &[proposed(809), proposed(2453)],
    /* stage 4 */
    &[
        hand(2257),
        proposed(3321),
        hand(7353),
        proposed(8214),
        proposed(9625),
    ],
    /* stage 5 */ &[hand(2263), proposed(6727)],
    /* stage 6 */ &[hand(1450)],
    /* extra   */ &[hand(2549), hand(3628), hand(5348), hand(7256)],
];
