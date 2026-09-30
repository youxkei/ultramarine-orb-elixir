//! **th06nc's Extra, by the ways th06c's was cleared with no hit and no bomb by** — see `super::th06c`.

use orb_config::recipe::Dodge;

use super::ON_THE_FLOOR;
use crate::Ways;

/// **The Extra's stage**, counted from 1 as a run's chapters are.
const EXTRA: i32 = 7;

/// **How long chapter 7 is**, script 4643 to 5348: this game's script runs 100 behind th06c's — 4643 where th06c's
/// resumes at 4743, and `game/th06nc/chapters.rs`'s Extra rows each 100 below th06c's — so it is th06c's 705.
const CHAPTER_7: u32 = 705;

pub(super) fn extra() -> Ways {
    Ways {
        // **th06c's three cards under this game's numbers for them**: 禁忌「レーヴァテイン」 is 122 here where it is 55
        // there, 禁弾「過去を刻む時計」 128 for 61 and 秘弾「そして誰もいなくなるか？」 129 for 62, each read off this
        // game's traced Extra by the stage frame the card came up on — 14818 against th06c's 15184, 29177 against
        // 29541, 30655 against 31551.
        dodges: vec![
            (122, Dodge::Laevateinn),
            (128, Dodge::Clock),
            (129, Dodge::AndThenThereWereNone),
        ],
        // **And the floor held after the midboss, the x the rules' own**, over th06c's chapters.
        for_chapters: vec![
            (EXTRA, 7, Some(300..CHAPTER_7), ON_THE_FLOOR),
            (EXTRA, 8, None, ON_THE_FLOOR),
        ],
        ..Ways::default()
    }
}
