//! **th06c's Extra**, cleared with no hit and no bomb by these (2026-09-27), each measured off a pick-up of its own
//! chapter.

use orb_config::recipe::Dodge;

use super::ON_THE_FLOOR;
use crate::Ways;

/// **The Extra's stage**, counted from 1 as a run's chapters are.
const EXTRA: i32 = 7;

/// **How long chapter 7 is**, script 4743 to 5448 — `game/th06c/chapters.rs`'s Extra rows.
const CHAPTER_7: u32 = 705;

pub(super) fn extra() -> Ways {
    Ways {
        // **The three cards their dodges** — 禁忌「レーヴァテイン」 (55), 禁弾「過去を刻む時計」 (61) and
        // 秘弾「そして誰もいなくなるか？」 (62), see `crate::cards`.
        dodges: vec![
            (55, Dodge::Laevateinn),
            (61, Dodge::Clock),
            (62, Dodge::AndThenThereWereNone),
        ],
        // **And the floor held from 300 frames into chapter 7 through chapter 8, the x left to the rules**: the first
        // 300 are where the midboss's drop and its 1up are gathered, and from there the fairies fire fans of three at
        // the run — the strategy wiki's 最下部から素直に弾に集中してちょん避けし続ける — and walk it into a corner
        // anywhere higher.
        //
        // **The x left to the rules**, which is where every other hold of chapters 7 and 8 lost recordings: picked up
        // at chapter 7 of the runs kept (`kept-no-hit-1` to `-8`), x 192 on the low line lost two of five, x 280 three
        // into the bottom-right corner, a walk from side to side along it one or two, the same about the middle two of
        // three and focused at x 192 the first two played; on the floor x 192 lost two of the three recordings made
        // after those five, and x 96, 144, 240 and 288 each lost one of the two x 192 lost. What those runs lost to is
        // one moment of the stage, script frames 5384 to 5441, where two columns of fairies fly in from outside both
        // walls, taken on whichever wall the fire had pushed the run to; with the x the rules' own, all eight reached
        // chapter 9 with no death (2026-09-27).
        //
        // **And not th06nc's middle of the floor from chapter 7's frame 450**, though the same stage 100 script frames
        // on: those eight were pick-ups of kept chapters, each one attempt over again, so launched from nothing it was
        // measured, and it moved nothing — chapter 7 lost 4 of 24 and chapter 8 5 of 26 over 20 launches of
        // `recipes/th06c/measured/the_extra_played_through_its_second_midstage` at pace 4 (orb-real 1791746835), where
        // the x the rules' own lost 3 of 13 and 2 of 13 over ten whole Extras (1791744628), its hits in chapter 7 on
        // the left wall, 21.7,432.0 and 8.0,276.9. th06nc's chapter 8 went from 11 of 36 lost to 3 of 24 by it.
        for_chapters: vec![
            (EXTRA, 7, Some(300..CHAPTER_7), ON_THE_FLOOR),
            (EXTRA, 8, None, ON_THE_FLOOR),
        ],
        ..Ways::default()
    }
}
