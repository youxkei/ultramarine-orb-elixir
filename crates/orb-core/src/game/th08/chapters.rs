//! Midstage chapter boundaries, as script frame numbers.
//!
//! **The script clock stands still while a boss or a midboss is up** — see *The script clock* in `asm.md` —
//! so each fight is one frame of it, and a stage is divided at its midboss and its boss before any row here
//! is read. What a row divides is the stretch between two of those: the stage's start to its midboss, and
//! the midboss beaten to the boss. Boss attacks are not listed: those boundaries are found as the fight is
//! fought.
//!
//! **What the rows were chosen out of is every boundary the detector proposed**, off one `--collect` pass
//! a replay: `replay/th8_01.rpy`, the Normal run that cleared Kaguya's stage and walked stages 1, 2, 3, 4b, 5
//! and 6b; a Normal run that cleared Eirin's, for stage 6a; a Normal run of the second team, for stage 4a,
//! which only the second and third teams' runs go through; and an Extra cleared with bombs. See
//! `orb_core::tuning` for the pass, and `collect.yaml` beside this file for the walk that reaches a replay
//! — driven as `--autoplay=collect.yaml --collect --keep-in=<an empty directory>`, the last so that no
//! `tuning.txt` of an earlier pass is read back into the next. The passes proposed forty-five rows in all:
//! a gap of sixty frames with nothing to shoot at and no boss up, at least 120 frames into the chapter
//! before it.
//!
//! **And the choice among them was made by the rules th06's and th07's judged tables keep to**, which is
//! the one sitting of `--judge` either game had, rather than a sitting over this exe. th06's `tuning.txt`
//! holds forty-five judged rows — seventeen kept, twenty-eight refused — and th07's twenty-one are hands,
//! its waves leaving no gap the detector could propose in. Held against the frames each of those stages
//! brings its midboss and boss on and the frames its waves come on — `orb-real spawns` over a trace of each
//! game's Normal run and its Extra, and the log's once-a-second state lines for th07 — they keep to these:
//!
//! - **how many rows a stretch gets** is its length over 1,450, rounded, less one: 20 of th06's and th07's
//!   27 stretches have exactly that many, and the other seven one more or one fewer. So a chapter is some
//!   1,450 script frames on average, and a stretch shorter than 2,175 is left whole — th06's stage 3 after
//!   its midboss is 2,109 and has no row, and th07's stages 3 and 6 have none at all, though th06's stages 1
//!   and 2 have one each in stretches of 1,942 and 1,829.
//! - **no row within 590 frames before a midboss or a boss arrives**: every one of th06's proposals closer
//!   than that was refused — the gap before a fight was proposed in eleven of its fourteen stretches and
//!   kept in none — and the closest kept is th07's hand at 590 before stage 1's midboss.
//! - **none sooner than 880 frames after the stage starts**, th06's first kept row being at 880 and its
//!   proposals at 626 and 855 refused; **and none sooner than 440 after a midboss**, th06's stage 2 keeping
//!   one at 441.
//! - **rows at least 700 apart**, the nearest two kept being th07's 1388 and 2098: th06's stage 4 refused
//!   all three of the proposals 280 frames apart after its midboss.
//! - **a row stands in a lull**, a stretch of 150 frames or more with no enemy coming on: fourteen of th06's
//!   seventeen do. **Where a row is a hand it stands twenty frames before the next wave comes on** — the
//!   seven of th06's hands that stand in a lull are 10 to 48 frames before it, five of them under 30, and
//!   two are proposals moved there from earlier in the same lull: 3447 to 3728 and 7313 to 7356. So a row
//!   placed here by hand is the wave's first frame less twenty.
//! - **and of the divisions those allow, a proposal over a hand**, then the one standing in the longer lulls,
//!   then the one nearest to even. Those weights were the ones that brought back the most of th06's own
//!   table: six of its seventeen rows on the frame, and three more in the lull the kept one stood in.
//!
//! **Stage 2 is th07's shape, its enemies coming on with one lull of 150 frames between the start and the
//! midboss where the rules want two rows**, so its two are hands in the two lulls of a hundred that divide
//! it nearest to even. Of the detector's four there, three are in the last 570 frames before the midboss,
//! where the rules keep nothing, and the fourth, 4142, stands in no lull of a hundred.
//!
//! **The lulls are off the traces of the four launches the rows were proposed from**, and the Normal clear
//! of Kaguya's stage, `card-144-in-a-normal-clear`, for the stages it walked. What a trace says of when an
//! enemy comes on is the stage's script and not the run's play, so the three launches that went through a
//! stage agree on it; what they disagree on is how soon each wave was shot down, which is why a proposal
//! stands where the run the pass read left a gap.
//!
//! **And rows chosen by rules are not rows judged by a person watching the stage.** Six of seventeen, and
//! three more a lull off, is what the rules make of a table somebody did judge, so a sitting of `--tune --replay --judge` over this
//! exe is what would make this table this game's own: a row it keeps stays, a row it refuses is refused.
//!
//! **Not th07's rows copied across.** A boundary is a measurement of a run and not a reading of the exe, so
//! a number written here that nobody measured against `th08.exe` is a number a tuning pass would have to
//! find wrong before it could find its own. th08's stages are not th07's stages either: the waves differ,
//! and so does what the player is doing between them.
//!
//! **Nine rows, which is what this game's own tables hold** — see the stage table in `asm.md`: the two
//! `.std` tables at `0x4c731c` and `0x4c7340` and the `.ecl` one at `0x4c73cc` each have nine entries,
//! indexed by `0x164d2cc` counted from zero, and the name table beside them names the same nine. The
//! Last Words are the tenth name and none of those nine, loading `ecldata_yk.ecl` and its five siblings
//! out of the spell-practice table by card — the seventeen entries at `0x4c7418` from card `0xcd`, eleven
//! spell-practice files and then the six Last Words for cards `0xd8` to `0xdd` (0x42ed71) — so they are
//! not a row here.

use crate::game::{Boundary, hand, proposed};

/// Indexed by the stage counted from zero, which is what [`Th08::read_state`](super::Th08) answers:
/// `0x164d2cc` as it stands — see `G_STAGE` in `th08/mod.rs`, which this game already counts from
/// nothing.
///
/// Nine rows: the six stages a Normal run plays, the two halves the fourth and the last are each
/// split into, and the Extra.
pub const MIDSTAGE: [&[Boundary]; 9] = [
    /* stage 1          */ &[hand(1352)],
    /* stage 2          */ &[hand(1273), hand(2873)],
    /* stage 3          */ &[proposed(1897)],
    /* stage 4 uncanny  */ &[hand(1520), hand(3248)],
    /* stage 4 powerful */ &[hand(1484), proposed(3160)],
    /* stage 5          */ &[hand(1084), proposed(2850), proposed(5917)],
    /* stage 6 Eirin    */ &[proposed(1424)],
    /* stage 6 Kaguya   */ &[proposed(1613)],
    /* extra            */
    &[
        proposed(1011),
        proposed(2393),
        proposed(3498),
        proposed(6776),
        hand(8418),
    ],
];

#[cfg(test)]
mod tests {
    use super::MIDSTAGE;

    /// The script frame each stage's midboss and then its boss arrives on, which the clock stands on while
    /// she is up — read off the traces the rows were chosen with, where every launch through a stage read
    /// the same two.
    const FIGHTS: [(i32, i32); 9] = [
        (2935, 4176),
        (4870, 4871),
        (3080, 4663),
        (4963, 6923),
        (4963, 6923),
        (4531, 7482),
        (3407, 4023),
        (3492, 4113),
        (5170, 9498),
    ];

    fn stretches(stage: usize) -> [(i32, i32); 2] {
        let (midboss, boss) = FIGHTS[stage];
        [(0, midboss), (midboss, boss)]
    }

    fn rows_in(stage: usize, (from, to): (i32, i32)) -> Vec<i32> {
        MIDSTAGE[stage]
            .iter()
            .map(|row| row.frame)
            .filter(|&frame| from < frame && frame < to)
            .collect()
    }

    #[test]
    fn every_row_stands_between_the_stages_start_and_its_boss() {
        for (stage, rows) in MIDSTAGE.iter().enumerate() {
            let (_, boss) = FIGHTS[stage];
            for row in rows.iter() {
                assert!(
                    0 < row.frame && row.frame < boss,
                    "stage {stage} row {}",
                    row.frame
                );
            }
        }
    }

    #[test]
    fn a_stretch_has_a_row_for_every_1450_frames_of_it_but_one() {
        for stage in 0..MIDSTAGE.len() {
            for stretch in stretches(stage) {
                let wanted =
                    (((stretch.1 - stretch.0) as f64 / 1450.0).round() as usize).max(1) - 1;
                assert_eq!(
                    rows_in(stage, stretch).len(),
                    wanted,
                    "stage {stage} {stretch:?}"
                );
            }
        }
    }

    #[test]
    fn no_row_stands_within_590_frames_before_a_fight() {
        for stage in 0..MIDSTAGE.len() {
            for stretch in stretches(stage) {
                for row in rows_in(stage, stretch) {
                    assert!(stretch.1 - row >= 590, "stage {stage} row {row}");
                }
            }
        }
    }

    #[test]
    fn no_row_stands_within_880_frames_of_the_start_or_440_of_the_midboss() {
        for stage in 0..MIDSTAGE.len() {
            let [start, after_the_midboss] = stretches(stage);
            for row in rows_in(stage, start) {
                assert!(row >= 880, "stage {stage} row {row}");
            }
            for row in rows_in(stage, after_the_midboss) {
                assert!(row - after_the_midboss.0 >= 440, "stage {stage} row {row}");
            }
        }
    }

    #[test]
    fn the_rows_of_a_stage_stand_700_frames_apart_in_order() {
        for (stage, rows) in MIDSTAGE.iter().enumerate() {
            for pair in rows.windows(2) {
                assert!(
                    pair[1].frame - pair[0].frame >= 700,
                    "stage {stage} {}",
                    pair[1].frame
                );
            }
        }
    }
}
