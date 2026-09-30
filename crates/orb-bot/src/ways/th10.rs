//! **th10's ways**: the attacks that carried the rules into a wall or a corner, each with the way that took it with no
//! hit.

use std::sync::Arc;

use orb_config::recipe::Dodge;

use crate::{Ways, steer};

/// **The stage a way is written for**, counted from 1 as a run's chapters are.
const STAGE_4: i32 = 4;
const EXTRA: i32 = 7;

/// **The Normal's**, on the whole Normal runs and stage practices of 2026-10-01 to 04: stage 1's boss's nonspell after
/// card 3, stage 4's midboss and the waterfall after her, stage 5's cards 62 and 70, and stage 6's card 78, the
/// nonspells after cards 78 and 82, and cards 90 and 94.
pub(super) fn normal() -> Ways {
    Ways {
        dodges: vec![
            (62, Dodge::UnderTheBoss),
            // **Card 70 in the middle**: the rules' own carried the run to the floor and were hit there on two whole
            // Pointdevice Normals, u19's in the bottom-left corner at `-184.0,432.0` 1,265 frames in and u22's at
            // `-2.2,432.0` 903 in; the latter's kept chapter picked up again was hit at the same place and frame, and in
            // the middle, under the boss, a third of the way up, right of the middle and where there will be room each
            // took it with no hit (c70k, 2026-10-04, at sixteen times), as they did four stage 5 practices apiece (c70p).
            (70, Dodge::InTheMiddle),
            // **Card 78 in the middle and held up off the floor**, between its lasers: four straight down about the
            // field's middle, at `±26` from the top and `±42.6` from `y = 326`, `7.5` either side, and under the boss —
            // left of the middle — the run stood between the columns at `-42.6` and `-26`, `1.6` of room, and was
            // carried down into the one at `-26`, hit at the floor 778 frames in on a whole legacy Normal (u22) and two
            // stage 6 practices of eighteen. **Not in the middle with the height left to the rules**, which took six
            // practices of six (c78p): u38's Pointdevice Normal was pushed down from `0,368` to the floor there some
            // 950 frames into the card and hit at `14.4,432.0` 1,204 in on every pickup, where a third of the way up
            // and no lower took that chapter, keep\hits-pd2's and four stage 6 practices with no hit (c78-a3.sh,
            // 2026-10-04, at sixteen times).
            (78, Dodge::AThirdOfTheWayUpAndNoLower),
            // **Card 90 cut back across short of a wall**, its fire being aimed at the run: held at any one place — the
            // middle, under the boss, `0,410`, `96,410`, `150,410` — the run was led along to a wall and hit there 870
            // to 1,430 frames in, where cut back it went through all four of the card's kept Pointdevice chapters and
            // all six stage 6 practices.
            (90, Dodge::TheMiracleOfOtensui),
            // **Card 94, 「マウンテン・オブ・フェイス」, on the floor beside the middle**, where the user's own th10_16.rpy
            // took it, between `y = 370` and the floor and `0` to `90` across: the rules' own climbed through it a step
            // at a time, from `y = 410` to `207` and `258`, and were taken inside a ring of its bullets up there, hit
            // on two stage 6 practices of the thirteen that reached it (s6h and s94, 2026-10-04, at sixteen times);
            // on the floor by the middle, the four of s94 that reached it went through.
            (94, Dodge::OnTheFloorByTheMiddle),
        ],
        after_cards: vec![
            (3, Dodge::UnderTheBoss),
            // **Stage 5's nonspell after card 66 in the middle**, which three stage 5 practices of three took (q1.sh,
            // 2026-10-03), and not the rules' own: under them u44's Pointdevice Normal was hit at `-184.0,432.0`, the
            // bottom-left corner, 892 frames in on some forty retries running until it was closed — the emptiest box
            // the rules go to taking no part of a retry's aside, so every retry was the one before it — and u41's
            // at `-184.0,428.8` 896 in.
            (66, Dodge::InTheMiddle),
            // **The nonspell after card 78 on the floor beside the middle**, the floor being where the user's own
            // th10_16.rpy took it, streaming from the bottom-right corner to the middle: its fire is aimed at the run
            // and leads one along into a wall, and under the boss it was led into a corner on five stage 6 practices
            // of some thirty-five (s6h, s6m and s94, 2026-10-04, at sixteen times). Five practices apiece up to card
            // 82 (s78n): on the floor by the middle and under the boss five of five, right of the middle three, side
            // to side two — the floor's hold against the lead being what under the boss lacks.
            (78, Dodge::OnTheFloorByTheMiddle),
            // **The nonspell after card 82, 筒粥「神の粥」, a third of the way up in the middle and held there**, at the
            // user's word and the user's own place in th10_16.rpy, `0,285`: its fire is aimed at the run and leads one
            // that holds anywhere into a corner. Held to the height and the middle, all eight stage 6 practices that
            // reached it went through it (s6m, 2026-10-04, at sixteen times). Side to side along the low line — the
            // way of six practices of six (c78s) — was hit in it on a whole legacy Normal at `-126.2,432.0` (u23) and
            // a practice on all ten of its retries at `-116.8,432.0` (c78p); a third of the way up with the x left to
            // the rules, on three whole runs of four, carried down to the floor; and held to the height alone, a
            // practice was led along it into the left wall and down it to the floor (s6h).
            (82, Dodge::AThirdOfTheWayUpAndNoLower),
        ],
        // **By the stage's own frames and not its chapters'**: the midboss and the waterfall are one chapter to 5136
        // (`game/th10/chapters.rs`) and want five ways inside it, and the frame the stage's script brings each on at
        // is the same every run.
        stretches: vec![
            // The midboss on the floor beside the middle until 4,500, where the user's own th10_16.rpy stood, and under
            // her after it — six practices of six with no hit (p4m6), against four of six under her throughout and
            // three of six a third of the way up.
            (STAGE_4, 3600..4500, Dodge::OnTheFloorByTheMiddle),
            (STAGE_4, 4500..4660, Dodge::UnderTheBoss),
            // The waterfall's last fall on the floor beside the middle, from 4,880 to 5,140, where the user's own
            // th10_16.rpy was between `-85` and `45`: away from where the fall lands through it, the run crossed right
            // behind the band and was carried along the floor into the bottom-right corner, three stage 4 practices of
            // six hit there at 5,076 to 5,083 and a legacy Normal of 2026-10-03 at 5,085 — on the floor, six of six
            // went through (p4f). **From 4,880 and not 4,940**: away from where the fall lands to 4,940 carried two
            // whole Pointdevice Normals (u17b, u20) from `-69.5,373.4` at 4,880 into the bottom-left corner by 4,940,
            // and the floor's way began with the run in it and was hit within fifteen frames, three times; from 4,880
            // six stage 4 practices of six went through (2026-10-04, at sixteen times), as did six from 4,660, and five
            // of six kept chapters at the waterfall (w4k). **From 4,660 and not 4,880**: away from where the fall lands
            // to 4,880 left u38's Pointdevice Normal at `-69,374` as the floor's way took over, and it went down to the
            // floor and along it into the bottom-left corner, hit at `-179.5,432.0` on 4,948 on every pickup — and
            // u18-p8's chapter at 4,951 the same way: five of the seven kept chapters that played through (w4-u38.sh),
            // where on the floor from 4,660 all eight went through (w4-f4660.sh, 2026-10-04).
            (STAGE_4, 4660..5140, Dodge::OnTheFloorByTheMiddle),
            (STAGE_4, 5140..5400, Dodge::AwayFromWhereTheFallLands),
        ],
        ..Ways::default()
    }
}

/// **The Extra's**: the cards the rules cannot get through with no hit, each with a way of its own — see each in
/// `crate::cards` — the fire after cards 101 and 104, the midstage before Suwako, and card 107 along a player's own
/// path.
pub(super) fn extra() -> Ways {
    Ways {
        dodges: vec![
            // 神符「杉で結ぶ古き縁」 tapped off each aim.
            (98, Dodge::OldTiesBoundByCedars),
            // 神符「神が歩かれた御神渡り」 aside of each pillar as it is laid down.
            (99, Dodge::OmiwatariThatGodWalked),
            // 開宴「二拝二拍一拝」 by its spokes' schedule.
            (100, Dodge::TwoBowsTwoClapsOneBow),
            // 土着神「手長足長さま」 up and down the middle, off the foot as each slow ring comes: the ring, bullets `13`
            // apart coming down at `0.52` a frame, comes down through the middle's foot each 360 frames, and met the
            // run there on card frames 601 to 603, 962 three times and 1,319 to 1,322 of the whole Extras of
            // 2026-10-04 — side to side along the low line crossing the middle then, or in the middle standing at it.
            // Over the six kept chapters at its start (c101x, at sixteen times): up and down the middle six of six on
            // the card, side_to_side five, hit on keep\b2-1 at `0.0,356.6` 1,322 frames in; before that (c101w)
            // side_to_side four of five, wherever_it_is_emptiest four of five, in_the_middle three of four and
            // where_there_will_be_room three of five.
            (101, Dodge::TenagaAshinaga),
            // 神具「洩矢の鉄の輪」 at the ends of the boss's column as each ring arrives, under the boss and above her
            // changing ends: no hit on the first attempt from all three of the card's kept Pointdevice chapters tried,
            // where in_the_middle was hit twice on the second and twice on the third.
            (102, Dodge::MoriyasIronRings),
            // 源符「厭い川の翡翠」 played on ahead.
            (103, Dodge::JadeOfTheLoathsomeRiver),
            // 蛙狩「蛙は口ゆえ蛇に呑まるる」 with each knot about to burst feared over its burst.
            (104, Dodge::TheSnakeEatsTheCroakingFrog),
            // **Card 108, 「諏訪大戦　～ 土着神話 vs 中央神話」, by the left wall half way up**, where the user's own Extra
            // took it: its slow bullets, `0.6` a frame down, pile at the floor with a stream of `2.5` a frame through
            // them, and the card holds its boss's life at 10,000 to its end. The rules' own were carried down to the
            // floor in its last 700 frames and hit among the pile three times (4,991, 4,918 and 4,309 frames in, u19 and
            // u20); a third of the way up with the x left to the rules, hit there on two whole legacy Extras of four
            // (u26 at 4,969 frames in, c108x at 4,874); by the left wall with the height the rules', on three of seven
            // (c108x, c108y), the run standing at the wall just above the low line and pushed through it in the card's
            // last frames. By the wall half way up, the two whole legacy Extras of c108z that reached it went through
            // (2026-10-04, at sixteen times). Over the two kept chapters every way tried went through.
            (108, Dodge::GreatSuwaWar),
        ],
        after_cards: vec![
            // **The fire after card 101 low in the middle and held there**, where the user's own Extra took it:
            // streams of bullets coming down to the left at some `-1.0,2.0` a frame, over a thousand up, carried the
            // rules' own along the floor into the bottom-left corner and hit them at `-172.6,432.0` on every retry of
            // one kept chapter (c99fix, 2026-10-04). Over the six kept chapters at card 101's start (a101-ways, at
            // sixteen times), counting the pickups that reached it: low_in_the_middle six of six (a101z),
            // right_of_the_middle five of six twice over, hit on keep\b2-3 at `-180.4,432.0` 760 frames in both times
            // and on a whole legacy Extra (u25) carried from `98,369` down to the floor and along it into that corner;
            // on_the_floor_by_the_middle four of five, in_the_middle, side_to_side and under_the_boss four of six
            // apiece — their every hit in that corner, at `-172` to `-184` some 720 to 980 frames in — and
            // where_there_will_be_room two of three.
            //
            // **And a third of the way up and held by the middle and to that height, rather than low in the middle**:
            // low in it, five whole Extras were pushed from the way's `±16,405` to the floor and hit there 459 to 508
            // frames in (u32, u34, u35, u36), keep\hits-u34\s7-c14-f18970 on both pickups of it; held a third of the
            // way up, both of them went through, as did in_the_middle's two — whose every earlier hit was in the
            // bottom-left corner the hold keeps out of reach (a101-u34.sh, 2026-10-04).
            (101, Dodge::AThirdOfTheWayUpAndNoLower),
            // **The fire after card 104 a third of the way up, and held by the middle and to that height**: a third of
            // the way up with the x left to the rules, two whole Extras were carried into the bottom-left corner and hit
            // at `-184.0,432.0` some 605 frames in (u23's Pointdevice one at 606, u34's second legacy one at 604). Over
            // the one kept chapter of it both went through every time it was played (a104-holds.sh, 2026-10-04), a
            // pickup not replaying a whole run's attempt; held so, the corner is out of reach.
            (104, Dodge::AThirdOfTheWayUpAndNoLower),
        ],
        // And the midstage before Suwako, from 9200 to her coming on at 13039, without the items: the rules alone went
        // after the point items through the sunflower fairies and were carried into the bottom-left corner, hit at
        // `-184,432` on every attempt from a kept chapter of it; without_the_items and in_the_middle came through it
        // with no hit, a_third_of_the_way_up was hit twice. **By the stage's frames and from 9200**, the chapter it is
        // having begun at 9291, 9318 and 9399 on three launches.
        //
        // **And nowhere to stand through her conversation after it**: where the run is as it ends sets the side card
        // 100's lasers come from (the strategy wiki, wikiwiki.jp/thk, 風/EX), and a run put there for it is a run
        // playing the game's workings rather than its cards — two_bows_two_claps_one_bow plans against the lasers it
        // sees, from either side.
        stretches: vec![(EXTRA, 9200..13039, Dodge::WithoutTheItems)],
        follows: vec![(
            107,
            steer::Following {
                every: 4,
                path: THE_RED_FROGS_PATH
                    .iter()
                    .map(|&[x, y]| (x, y))
                    .collect::<Arc<[_]>>(),
            },
        )],
        ..Ways::default()
    }
}

/// **Card 107 土着神「宝永四年の赤蛙」 gone through along a player's own path**, a place every four frames of the card,
/// written out of their replay (th10\replay\th10_23.rpy, played back with `--trace` and `orb-real paths` over the
/// trace): the card's doubles go at where the player is and lay their rice on the way, so a run where the player was,
/// frame for frame, is fired at as they were — the run up the right wall a focused step at a time, along the top to the
/// left at the whole step as the second double comes out, down the left wall and round the field. All eight of the
/// card's kept chapters went through so, where the way of the rules' own took four of seven.
#[rustfmt::skip]
const THE_RED_FROGS_PATH: &[[f32; 2]] = &[
      [68.2, 432.0], [68.2, 432.0], [68.2, 432.0], [68.2, 432.0], [68.2, 432.0], [68.2, 432.0], [72.2, 432.0], [76.2, 432.0],
      [76.2, 432.0], [76.2, 432.0], [82.2, 432.0], [82.2, 432.0], [82.2, 432.0], [82.2, 432.0], [82.2, 432.0], [82.2, 432.0],
      [82.2, 432.0], [64.2, 432.0], [46.2, 432.0], [28.2, 432.0], [10.2, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0],
      [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0],
      [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0],
      [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [-0.8, 432.0], [12.7, 432.0], [30.7, 432.0], [48.7, 432.0], [66.7, 432.0],
      [84.7, 432.0], [102.7, 432.0], [120.7, 432.0], [138.7, 432.0], [156.7, 432.0], [174.7, 432.0], [179.2, 432.0], [179.2, 432.0],
      [179.2, 432.0], [179.2, 432.0], [179.2, 432.0], [175.2, 432.0], [167.2, 432.0], [159.2, 432.0], [157.2, 432.0], [157.2, 432.0],
      [157.2, 432.0], [157.2, 432.0], [157.2, 432.0], [157.2, 432.0], [157.2, 432.0], [157.2, 432.0], [157.2, 430.0], [157.2, 422.0],
      [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0],
      [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0],
      [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 414.0], [157.2, 412.0], [157.2, 404.0], [157.2, 396.0], [157.2, 394.0],
      [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0],
      [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 394.0], [157.2, 392.0], [157.2, 384.0],
      [157.2, 376.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0],
      [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 370.0],
      [157.2, 370.0], [157.2, 370.0], [157.2, 370.0], [157.2, 368.0], [157.2, 360.0], [157.2, 352.0], [157.2, 350.0], [157.2, 350.0],
      [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0],
      [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 350.0], [157.2, 346.0], [157.2, 338.0],
      [157.2, 330.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0],
      [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 328.0],
      [157.2, 328.0], [157.2, 328.0], [157.2, 328.0], [157.2, 324.0], [157.2, 316.0], [157.2, 308.0], [157.2, 306.0], [157.2, 306.0],
      [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0],
      [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 306.0], [157.2, 300.0], [157.2, 292.0],
      [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0],
      [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0],
      [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 284.0], [157.2, 278.0], [157.2, 270.0], [157.2, 262.0], [157.2, 256.0],
      [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0],
      [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 256.0], [157.2, 252.0], [157.2, 244.0],
      [157.2, 236.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0],
      [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0],
      [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 230.0], [157.2, 224.0], [157.2, 216.0], [157.2, 208.0], [157.2, 204.0],
      [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0],
      [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 204.0], [157.2, 200.0],
      [157.2, 192.0], [157.2, 184.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0],
      [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 180.0],
      [157.2, 180.0], [157.2, 180.0], [157.2, 180.0], [157.2, 178.0], [157.2, 170.0], [157.2, 162.0], [157.2, 154.0], [157.2, 152.0],
      [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0],
      [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 152.0], [157.2, 148.0],
      [157.2, 140.0], [157.2, 132.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0],
      [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0],
      [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 128.0], [157.2, 122.0], [157.2, 114.0], [157.2, 106.0],
      [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0],
      [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 102.0], [157.2, 100.0],
      [157.2, 92.0], [157.2, 84.0], [157.2, 76.0], [157.2, 68.0], [157.2, 60.0], [157.2, 52.0], [151.2, 44.0], [135.8, 37.6],
      [117.8, 37.6], [99.8, 37.6], [81.8, 37.6], [63.8, 37.6], [45.8, 37.6], [27.8, 37.6], [9.8, 37.6], [-8.2, 37.6],
      [-26.2, 37.6], [-44.2, 37.6], [-62.2, 37.6], [-80.2, 37.6], [-98.2, 37.6], [-116.2, 37.6], [-134.2, 37.6], [-152.2, 37.6],
      [-170.2, 37.6], [-184.0, 37.6], [-184.0, 37.6], [-184.0, 37.6], [-184.0, 37.6], [-184.0, 37.6], [-184.0, 37.6], [-184.0, 37.6],
      [-184.0, 41.6], [-184.0, 49.6], [-184.0, 49.6], [-184.0, 49.6], [-184.0, 49.6], [-184.0, 55.6], [-184.0, 57.6], [-184.0, 57.6],
      [-184.0, 57.6], [-184.0, 65.6], [-184.0, 71.6], [-184.0, 71.6], [-184.0, 71.6], [-184.0, 71.6], [-184.0, 71.6], [-184.0, 71.6],
      [-184.0, 71.6], [-184.0, 73.6], [-184.0, 81.6], [-184.0, 83.6], [-184.0, 83.6], [-184.0, 83.6], [-184.0, 91.6], [-184.0, 93.6],
      [-184.0, 93.6], [-184.0, 93.6], [-184.0, 93.6], [-184.0, 101.6], [-184.0, 105.6], [-184.0, 105.6], [-184.0, 105.6], [-184.0, 107.6],
      [-184.0, 115.6], [-184.0, 117.6], [-184.0, 117.6], [-184.0, 117.6], [-184.0, 117.6], [-184.0, 117.6], [-184.0, 121.6], [-184.0, 125.6],
      [-184.0, 125.6], [-184.0, 125.6], [-184.0, 125.6], [-184.0, 131.6], [-184.0, 137.6], [-184.0, 137.6], [-184.0, 137.6], [-184.0, 137.6],
      [-184.0, 145.6], [-184.0, 149.6], [-184.0, 149.6], [-184.0, 149.6], [-184.0, 149.6], [-184.0, 157.6], [-184.0, 161.6], [-184.0, 161.6],
      [-184.0, 161.6], [-184.0, 161.6], [-184.0, 161.6], [-184.0, 169.6], [-184.0, 169.6], [-184.0, 169.6], [-184.0, 169.6], [-184.0, 173.6],
      [-184.0, 181.6], [-184.0, 181.6], [-184.0, 181.6], [-184.0, 181.6], [-184.0, 189.6], [-184.0, 195.6], [-184.0, 195.6], [-184.0, 195.6],
      [-184.0, 197.6], [-184.0, 205.6], [-184.0, 205.6], [-184.0, 205.6], [-184.0, 205.6], [-184.0, 205.6], [-184.0, 207.6], [-184.0, 215.6],
      [-184.0, 215.6], [-184.0, 215.6], [-184.0, 215.6], [-184.0, 219.6], [-184.0, 225.6], [-184.0, 225.6], [-184.0, 225.6], [-184.0, 225.6],
      [-184.0, 229.6], [-184.0, 233.6], [-184.0, 233.6], [-184.0, 233.6], [-184.0, 237.6], [-184.0, 245.6], [-184.0, 245.6], [-184.0, 245.6],
      [-184.0, 245.6], [-184.0, 245.6], [-184.0, 245.6], [-184.0, 253.6], [-184.0, 255.6], [-184.0, 255.6], [-184.0, 255.6], [-184.0, 257.6],
      [-184.0, 265.6], [-184.0, 265.6], [-184.0, 265.6], [-184.0, 265.6], [-184.0, 269.6], [-184.0, 277.6], [-184.0, 277.6], [-184.0, 277.6],
      [-184.0, 281.6], [-184.0, 289.6], [-184.0, 289.6], [-184.0, 289.6], [-184.0, 289.6], [-184.0, 289.6], [-184.0, 289.6], [-184.0, 295.6],
      [-184.0, 297.6], [-184.0, 297.6], [-184.0, 297.6], [-184.0, 299.6], [-184.0, 307.6], [-184.0, 307.6], [-184.0, 307.6], [-184.0, 307.6],
      [-184.0, 315.6], [-184.0, 319.6], [-184.0, 319.6], [-184.0, 321.6], [-184.0, 329.6], [-184.0, 331.6], [-184.0, 331.6], [-184.0, 331.6],
      [-184.0, 331.6], [-184.0, 331.6], [-184.0, 337.6], [-184.0, 341.6], [-184.0, 341.6], [-184.0, 341.6], [-184.0, 343.6], [-184.0, 349.6],
      [-184.0, 349.6], [-184.0, 349.6], [-184.0, 355.6], [-184.0, 359.6], [-184.0, 359.6], [-184.0, 359.6], [-184.0, 359.6], [-184.0, 359.6],
      [-184.0, 365.6], [-184.0, 373.6], [-184.0, 381.6], [-184.0, 392.1], [-174.5, 406.2], [-161.7, 418.9], [-147.7, 428.4], [-129.7, 428.4],
      [-111.7, 428.4], [-96.3, 432.0], [-83.6, 432.0], [-70.9, 432.0], [-58.2, 432.0], [-42.8, 432.0], [-24.8, 432.0], [-6.8, 432.0],
      [9.9, 432.0], [27.9, 432.0], [45.9, 432.0], [63.9, 432.0], [81.9, 432.0], [99.9, 432.0], [112.6, 419.3], [127.9, 412.9],
      [145.9, 412.9], [163.9, 412.9], [181.9, 412.9], [184.0, 403.4], [184.0, 390.7], [184.0, 377.9], [184.0, 359.9], [184.0, 341.9],
      [184.0, 323.9], [184.0, 305.9], [184.0, 287.9], [184.0, 269.9], [184.0, 251.9], [184.0, 233.9], [184.0, 215.9], [184.0, 197.9],
      [184.0, 179.9], [184.0, 166.9], [184.0, 158.9], [184.0, 150.9], [178.4, 145.3], [165.6, 132.6], [152.9, 119.9], [140.2, 107.1],
      [127.5, 94.4], [114.8, 81.7], [102.0, 69.0], [88.0, 59.4], [70.0, 59.4], [52.0, 59.4], [34.0, 59.4], [16.0, 59.4],
      [-2.0, 59.4], [-20.0, 59.4], [-38.0, 59.4], [-56.0, 59.4], [-74.0, 59.4], [-92.0, 59.4], [-110.0, 59.4], [-128.0, 59.4],
];
