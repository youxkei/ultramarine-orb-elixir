//! **th06nc's Extra, by the ways th06c's was cleared with no hit and no bomb by** — see `super::th06c`.

use orb_config::recipe::Dodge;

use super::ON_THE_FLOOR;
use crate::Ways;
use crate::steer::{Across, Standing};

/// **The Extra's stage**, counted from 1 as a run's chapters are.
const EXTRA: i32 = 7;

/// **How long chapter 7 is**, script 4643 to 5348: this game's script runs 100 behind th06c's — 4643 where th06c's
/// resumes at 4743, and `game/th06nc/chapters.rs`'s Extra rows each 100 below th06c's — so it is th06c's 705.
const CHAPTER_7: u32 = 705;

/// **Where in chapter 7 the run is held in the middle instead**, script 5093: ahead of every attempt lost being
/// walked to the left wall — from 5175 in two of them and from 5140 in the third, which was on the wall by 5180
/// and held from 5200 could not leave it. Held from 5200 first, that third one was the attempt it lost.
const WALKED_TO_A_WALL: u32 = 450;

/// **And how far into chapter 8 that lasts**, past the columns' last at script 5341 and the eight frames into the
/// chapter a run left on the floor by the wall was lost at.
const THE_COLUMNS_GONE: u32 = 120;

/// **The middle of the field across, the height the rules' own.**
const IN_THE_MIDDLE: Standing = Standing {
    across: Some(Across::At(192.0)),
    y: None,
};

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
        // **And the floor held after the midboss, the x the rules' own**, over th06c's chapters — **but the middle
        // of the field held across the two columns of fairies**, from chapter 7's frame 450 (script 5093) to 120
        // frames into chapter 8: the columns fly in from outside both walls at script 5284 to 5341, this game's
        // 100 behind th06c's 5384 to 5441, and the floor left the run to be carried onto a wall by then. Read off
        // th06nc_dev's traces of the no-hit Extra through the pad (2026-10-05): the two attempts lost were walked
        // to the left wall from script 5175 — hit at 8.0,304.4 in chapter 7 in one session, and boxed in at
        // 16.5,432.0 on the floor in the other, where chapter 8 began eight frames before the hit and every retry
        // of it was lost on the same frame — and the three that got through stood at x 114 to 326 as the columns
        // came, the run before the ways moved into the engine among them.
        for_chapters: vec![
            (EXTRA, 7, Some(300..WALKED_TO_A_WALL), ON_THE_FLOOR),
            (EXTRA, 7, Some(WALKED_TO_A_WALL..CHAPTER_7), IN_THE_MIDDLE),
            (EXTRA, 8, Some(0..THE_COLUMNS_GONE), IN_THE_MIDDLE),
            (EXTRA, 8, None, ON_THE_FLOOR),
        ],
        ..Ways::default()
    }
}
