//! **th11's ways**: the attacks the rules lost a run on, each with the way that took it with no hit.

use orb_config::recipe::Dodge;

use crate::Ways;

/// **The stage a way is written for**, counted from 1 as a run's chapters are.
const STAGE_5: i32 = 5;
const EXTRA: i32 = 7;

/// **The Normal's**, on the whole Normal runs and stage practices of 2026-10-07.
pub(super) fn normal() -> Ways {
    Ways {
        dodges: vec![
            // **Card 11, 妬符「グリーンアイドモンスター」, after the monster**: the rules' own took it with no hit on two
            // whole runs and were hit three times on a third, standing still each time with no room read; from the chapter
            // kept as `hits2/s2-c3-f3492`, in the middle took nine, side to side nine, under the boss eight, wherever it is
            // emptiest four and a third of the way up none (2026-10-07). Then over the four chapters kept by then
            // (`hits2`, `hits3`, `hits7` and `hits9`), a third of the way up took 0, 6, 1 and 1, under the boss 8, 0, 6
            // and 0, wherever it is emptiest ten and in the middle thirty-two, round the field sixteen and low in the
            // middle thirty-nine — a place held answering some seeds and not others, the monster caging a run that
            // stands still. **Led up under the boss as the monster reaches the run, then**, the strategy wiki's way —
            // see `orb_bot::cards` — none on all four, the boss beaten each time before her second monster
            // (2026-10-07).
            (11, Dodge::GreenEyedMonster),
            // **Card 15, stage 2's second card of the boss's, wherever it is emptiest**: the rules' own were hit once at
            // the right wall, `184,258`, on a whole Pointdevice run; from the chapter it kept as `hits4/s2-c6-f7119`,
            // wherever it is emptiest and in the middle none, side to side and a third of the way up one each and low in
            // the middle three; and in the middle was then hit three times on the next whole run, whose chapter
            // `hits6/s2-c6-f7025` took wherever it is emptiest and a third of the way up none, side to side one, in the
            // middle three and low in the middle five — wherever it is emptiest, then, the one with none on both
            // (2026-10-07).
            // **And round the boss, then**, the strategy wiki's way for Normal — see `orb_bot::cards` — wherever it is
            // emptiest having been hit three times on `hits4/s2-c6-f7119` standing under the boss in the middle
            // (2026-10-08): none on all four chapters kept for the card off four runs, where wherever it is emptiest
            // took 3, 0, 0 and 0, the pictures having the run round her and over her (2026-10-09). It was then hit once
            // on six whole runs, left behind by her at the left wall (`hits-n15/s2-c6-f8110`); held to the round, on past
            // a wall near her and at two degrees a frame, none on the five chapters kept for the card (2026-10-09).
            (15, Dodge::JealousyOfTheKind),
            // **Card 127, 恨霊「スプリーンイーター」, low in the middle**: the rules' own were hit in a corner once a stage 5
            // practice, and in the middle once on one practice and not at all on two whole Pointdevice runs, then six
            // times on a third; from the chapter that run kept as `hits4/s5-c7-f14437`, in the middle took three, a third
            // of the way up and no lower three, wherever it is emptiest four, under the boss five, side to side two, and
            // low in the middle none; and from `hits5/s5-c7-f14293`, another seed's, after low in the middle had been hit
            // six times there on a whole run, low in the middle three, a third of the way up and no lower two, wherever it
            // is emptiest three, and side to side, in the middle and under the boss none; and side to side then hit once on
            // the next whole run, whose `hits6/s5-c7-f14233` took low in the middle none, side to side one, wherever it is
            // emptiest two, under the boss and a third of the way up and no lower three and in the middle four — low in
            // the middle, then, with none on two seeds of the three where side to side had none on one, both three in
            // all (2026-10-07). And once the sight kept each spirit in its slot, over the six chapters kept by then
            // (`hits4` to `hits9`), low in the middle took six, a third of the way up and no lower nine, in the middle
            // twelve, side to side sixteen, away from the fire thirteen, and where there will be room and round the
            // field fifteen each — every hit at
            // `y = 428` to `432` with nowhere open, between two lines of the burst spirits' slow bullets crossing at the
            // floor (2026-10-07). **Not dodged no lower than the height it waits at, anywhere across**, which was tried
            // as a way of its own: it took twenty-two over the same six, at the floor still and in its corners, the floor
            // held off only while some move was clear and the run no longer held by the middle. **Its own fire played on,
            // then** — the card's rice, spirits and their burst, see `orb_bot::cards` — 0, 0, 2, 1, 0, 1 and 0 over the
            // first chapter kept off each of seven runs (`hits4` to `hits10`), where low in the middle took 0, 1, 0, 3,
            // 1, 1 and 1 (2026-10-09). **And then the partner's way, against a side wall and through Yukari's gap as a
            // ring closes in** — the wiki's 「紫の場合は開始直後に端で待機し、怨霊が迫ってきたらスキマを使えば時間切れまで
            // 粘れる」, see `orb_bot::cards::against_the_nearer_side_wall`: 0 0 0 0 0 0 2 0 0 0 0 0 0 0 0 0 over the
            // sixteen chapters kept for the card at `--pace=16`, where its fire played on with its home among the rice at
            // rest took 3 0 1 1 1 0 0 1 0 0 0 1 0 2 1 0 (2026-10-10); gone through with a spirit 112 off rather than 64,
            // and bodies kept 16 further from the gap's way, none of the sixteen hit (2026-10-10).
            (127, Dodge::ThroughTheGap),
            // **Card 131, 贖罪「旧地獄の針山」**, its name on the pictures (`hits/s5-c8-f17448`, 2026-10-09) and 131 in the
            // card table (`game/th11/asm.md`), and not `呪精「ゾンビフェアリー」`, which is 123 and the card the way
            // tried below up as each wave comes was written after: its "zombie fairies" are this card's spirits going
            // round. What follows was read so.
            // Its spirits, thirty-six to fifty-one of them,
            // close round the run and follow it, and the rules' own backed down away from them to the floor and the walls
            // and were hit there three times a stage 5 practice and thirteen attempts running on a whole Pointdevice run,
            // at `y = 410` to `432`; from the chapter kept as `hits2/s5-c8-f17311`, in the middle took two, a third of the
            // way up four, round the field two, and wherever it is emptiest, side to side and under the boss none
            // (2026-10-07). Wherever it is emptiest was then hit ten times running in the bottom-right corner with nothing
            // open on the real `a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb`, the place wanted flipping along
            // the floor between `82` and `-123`; and from `hits5/s5-c8-f17902`, another seed's, side to side took six,
            // wherever it is emptiest three, under the boss two, low in the middle one, and in the middle and a third of
            // the way up and no lower none — **in the middle, then**, the fewest over both, two (2026-10-07). A wave of
            // 38 fairies comes down onto the run from the top every 240 frames, and 22 of the 23 hits read were at the
            // floor. **And not drawn up and then down under each wave**, the strategy wiki's way
            // (`ゾンビ上へ誘導→下へ一気に`, wikiwiki.jp/thk/地/5), which was tried as a way of its own: up to 212 from each
            // wave's coming and down from 100 frames into it took none at hits5 and four at hits2, and up to 252 and
            // down from 170 took two and four, every hit at the floor still — the run, once down, not let up again by
            // the waves' bullets above it. **Nor in the middle with each fairy's step spent over twelve frames rather
            // than carried on straight**, though read with `--log=verbose` the run stood 175 frames at the floor with
            // none of its nine moves clear and the nearest fairy 90 to 110 off: that took two at hits2 and seven at
            // hits5, every one at the floor with nothing open still (2026-10-07). **Under the boss, then**: in the middle
            // was hit twice on the next whole run, whose `hits7/s5-c8-f19502` took under the boss and low in the middle
            // none, wherever it is emptiest and a third of the way up and no lower one, in the middle two and side to
            // side three — over the three seeds under the boss two in all and none on two of them, the fewest
            // (2026-10-07). **Low in the middle, then**, once the sight kept each fairy in its slot, which changed how
            // every way read them: over the four kept chapters `hits2`, `hits5`, `hits7` and `hits9`, under the boss
            // took eight, in the middle eight, wherever it is emptiest three and low in the middle three, none on two
            // of them each — and wherever it is emptiest the one of those two hit ten times running in a corner on a
            // whole run (2026-10-07). **Up as each wave of zombies comes and then in front of the boss, then**, the
            // strategy wiki's way at full power — see `orb_bot::cards` — and held to that place as card 127's circle is:
            // none, none, one and none over the four, where round the boss clockwise between took seven, led up and
            // then in front of her at the low line five, and 128 off the floor three (2026-10-07). It is this card's own
            // way, written on this card's own field, and only named after the other. It was hit once a whole Pointdevice
            // run on each of six (hits-n1 to hits-n6), four times on the sixth, and 1, 3, 2, 0, 0, 1, 0 and 1 over eight
            // chapters kept for the card (2026-10-09). **Not under the boss held within 40**, the wiki's
            // `お燐の前で自機に当たりそうな弾だけを小さく避ける` for the nonspells whose winder this card lays: its needles go
            // out from her, and under her the run stood in their stream and was carried down to the floor with nothing
            // open — 48 off the floor, 15 hits over nine chapters kept for the card, 120 off it, 13 (2026-10-09). **Nor its
            // fire played on as card 127's is**, its needles at rest let go together as the oldest is 160 frames old out
            // from her and speeding up by a sixtieth to 1.0 (scratchpad setoff127.py, from `hits-n1/s5-c8-f16964`): 12, 9
            // and 7 on the first three of the nine, the run never got out from among them (2026-10-09). **Between the
            // spinning spirits' ways, then**, the wiki's `回転霊` stepped aside of early — see `orb_bot::cards`: none on
            // any of the nine, where up as each wave comes took 1, 3, 2, 0, 0, 1, 0, 1 and 4 (2026-10-09). Hit twice on
            // whole run 19 let out of a narrow gap, then held within 16 of it: none on that chapter and the nine
            // (2026-10-09).
            (131, Dodge::BetweenTheSpinningSpirits),
            // **Card 135, 「死灰復燃」, wherever it is emptiest**: the boss stands at `0,224` in the field's middle for it, and
            // a third of the way up and no lower — which took a stage 5 practice with no hit — took nine hits in nine attempts at
            // the chapter kept as `hits/s5-c9-f20915`, the run hit by her body or chased up to the top by the rings of
            // bullets her zombie fairies burst into; on the floor by the middle five, and low in the middle, wherever it
            // is emptiest and side to side none; and from `hits4/s5-c9-f21243`, another seed's, low in the middle two and
            // the other two none; and side to side then hit once on the next whole run, whose `hits7/s5-c9-f21122` took
            // low in the middle and wherever it is emptiest none and side to side one — wherever it is emptiest none on
            // all three seeds (2026-10-07). **Up and down from the bodies, then**, the strategy wiki's way for Normal —
            // see `orb_bot::cards` — 0, 0, 3 and 0 over the four chapters kept for the card where wherever it is emptiest
            // took 0, 2, 0 and 0, the pictures having the run under her and at the top by turns (2026-10-09).
            (135, Dodge::Rekindled),
            // **Card 143, 核熱「ニュークリアフュージョン」, between the suns coming**: its rings of eight suns of radius 88 are
            // flung out across the whole field from above the boss, and the rules' own were hit at the floor under one
            // six times on each of two whole Normals and three stage 6 practices; wherever it is emptiest, held at
            // `y = 200`, in the ring's middle and at one fixed gap were each hit six to eight times a practice, and
            // between the two suns of each ring nearest straight down once (2026-10-07). Then, once the sight kept each
            // sun in its slot, seven, one and one at the chapters kept as `hits/s6-c5-f10070`, `hits8/s6-c5-f9287` and
            // `hits9/s6-c5-f8984`, every hit in a bottom corner; and held within forty across of its gap — see
            // `orb_bot::steer` — none on all three (2026-10-07).
            (143, Dodge::BetweenTheSuns),
            // **Card 147, 爆符「メガフレア」, on the floor where the flares come down last**: its flares fall straight down
            // from a reach of 80 to 92 and cross the floor some 30 to 50 across, and the rules' own were hit at the floor
            // three to seven times a stage 6 practice, away from the fire seven; on the floor away from where each would
            // land three, and over fifty attempts at the chapter kept as `hits/s6-c6-f14624` round the same four hits;
            // on the floor where they come down last, the way there crossed, the same chapter through with none
            // (2026-10-07).
            (147, Dodge::BetweenTheFlares),
            // **Card 151, 焔星「フィクストスター」, a third of the way up in the middle and no lower**: its stars of radius 82
            // go round the boss low on the field, and the rules' own hid in a bottom corner on their way and were hit
            // there seven times on one stage 6 practice; inside their ring a third of the way up, the next practice
            // that reached it went through it with no hit (2026-10-07). **On the right between the rings and up and
            // down, then**, off a summary of the strategy wiki — and from its section read whole, **in front of the boss
            // at the height between the rings**, the wiki's own (2026-10-09) — see `orb_bot::cards`.
            (151, Dodge::FixedStar),
            // **Card 159, 「地獄の人工太陽」, on the floor by the middle**: its sun of radius 124 stands at `0,224`, and the
            // rules' own were hit at its edge twice a stage 6 practice with `2.5` and `3.1 of room` read, high above it and
            // just under it — it seems to pull the player in, which is not read; on the floor, 84 under its edge, the
            // next practice went through it with no hit (2026-10-07). **Under it on the floor, away from its pull, then**,
            // the strategy wiki's `吸引に逆らって下がりつつ` — see `orb_bot::cards`, which has why not up and to its left.
            (159, Dodge::ArtificialSun),
        ],
        stretches: vec![
            // **Stage 5 after its midboss, a little below the middle**, the wiki's `中央やや下` as after the yin-yangs: the
            // rules' own went after the items to the left wall, were carried up it from `370` by the bullets rising at 3 a
            // frame from below, the wiki's `上に上ってくる弾幕を避けないこと`, and hit in the top left corner at
            // `-176.3,32.0` at its frame 8071 on the real case a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb and
            // again picking the chapter up (`s5-c4-f8071`, session `1791593378`, 2026-10-10); waited there, none on it and
            // on `hits-n12/s5-c4-f9077` at `--pace=16` (2026-10-10). From the midboss's end at 7,588.
            (STAGE_5, 7588..8000, Dodge::AfterTheYinYangs),
            // **Stage 5 before its boss, in the middle**: the rules' own were hit at the bottom-right corner once a stage 5
            // practice between its frames 8,000 and 8,700, and in the middle not at all on that practice and two whole
            // Pointdevice runs (2026-10-07). **Across the floor and never back, then**, the strategy wiki's way through its
            // yin-yangs — see `orb_bot::cards`.
            (STAGE_5, 8000..8700, Dodge::AcrossTheYinYangs),
            // **Stage 5 after its yin-yangs, a little below the middle**, the wiki's `中央やや下`: the rules' own were
            // carried to the floor's left and hit there at `-76.8,432.0` at its frame 9077 on one of six whole runs, and
            // again picking the chapter up (`hits-n12/s5-c4-f9077`); waited there, none (2026-10-09). Till the boss comes
            // at 10,993.
            (STAGE_5, 8700..10993, Dodge::AfterTheYinYangs),
        ],
        ..Ways::default()
    }
}

/// **The Extra's**, on the whole Pointdevice Extras and the chapters kept off them of 2026-10-07.
pub(super) fn extra() -> Ways {
    Ways {
        dodges: vec![
            // **Card 164, 神徳「五穀豊穣ライスシャワー」, under the boss a third of the way up**: rice falling over the field
            // and pellets scattered from her; the rules' own were carried along the low line to the bottom left corner and
            // hit there 2,018 frames in on `hits-extra15/s7-c4-f10039`, and under her a third of the way up, the wiki's
            // `ある程度ボスに近付くと粒弾が薄くなる`, none (2026-10-08).
            (164, Dodge::RiceShower),
            // **Card 163, 奇跡「ミラクルフルーツ」, a little above the floor and down to it as rice comes**, the wiki's
            // `画面最下段よりやや上で待機→弾が来たら最下段まで下がり残りの弾を横に避ける`: the rules' own were hit at
            // `-52.1,390.8` 673 frames in on the real case the_extra_played_to_its_clear_with_no_hit_and_no_bomb,
            // kept as `hits-real6/s7-c3-f7109`; with this way it took none there three times, nor through it from seven of
            // the chapters kept for chapter 1, the run on the floor on 1,050 of its 1,428 frames (2026-10-08).
            (163, Dodge::MiracleFruit),
            // **Card 167, 本能「イドの解放」, in front of the boss a little below the middle**, the wiki's
            // `右→左→右と交互に抜けていく`: the rules' own were pinned against the left wall and hit at `-184.0,416.1` 874
            // frames in on a real case's Extra, kept as `hits-real8/s7-c9-f20431`; with this way none there three times
            // left unheld, the pictures having it at the left wall half the card, and none three times held within 40,
            // in front of her below the middle on every picture (2026-10-08). Lowered from 268 to 326 down the field,
            // the wiki's 「「東方地霊殿」の地～霊あたり」 from its section read whole (2026-10-09).
            (167, Dodge::ReleaseOfTheId),
            // **Card 169, 反応「妖怪ポリグラフ」, between the rays round the boss, at one of two distances**: its eight rays
            // turn round her and lay rice as far off her as the run is, and a whole Pointdevice Extra was hit on it
            // fifteen times and more with the rules' own; a dozen ways under her or round her at one distance took three
            // to five on each of the three chapters kept as `hits-extra/s7-c11-f26723`, `f26899` and `f27227`, and
            // between the rays at 130 or 190 off her, gone to the other as the rice ahead comes to the one it is at, none
            // on all three (2026-10-07).
            (169, Dodge::YoukaiPolygraph),
            // **Card 171, 復燃「恋の埋火」, where its hearts come last**: hearts aimed at the run and thrown below the
            // boss to either side, coming back off every wall with a trail of slow bullets behind each; the rules' own
            // were hit twice on `hits-extra7/s7-c13-f35551`, low in the middle fourteen times on a whole Extra (extra8),
            // and the low line's place in the middle the hearts come onto last, carried on off the walls, none on
            // `hits-extra8/s7-c13-f35817` and `hits-extra6/s7-c13-f35909` and once on `f35551` 1,203 frames in
            // (2026-10-08).
            (171, Dodge::EmbersOfLove),
            // **Card 172, 深層「無意識の遺伝子」, across the field from the boss**: she goes corner to corner laying bullets
            // that set off from where she was; the rules' own were hit once on `hits-extra7/s7-c14-f44490` at the bottom
            // left where she came along the bottom, and across from her, low while she is high and up while she is
            // along the bottom, none on it and on `hits-extra8/s7-c14-f39468` (2026-10-08).
            (172, Dodge::GeneticsOfTheSubconscious),
            // **Card 173, 「嫌われ者のフィロソフィ」, at the top right of the boss's round and then just off its middle**:
            // the rules' own were hit by her roses some sixty times running on the floor at the bottom of her round,
            // and the middle, the place across from her and the places weighed against her two to five times on each of
            // the three chapters kept as `hits-extra4/s7-c15-f43323`, `f43580` and `f44225`; at the top right through
            // the blue stage and then 60 off the middle away from her, never moved to the left from left of it, none on
            // all three (2026-10-07). Then hit on three of seven whole Extras in its red stage by a row bullet a rose
            // took and grew; with the row bullets a step of the grid from a rose feared growing — see `crate::steer`'s
            // `A_ROSE_TAKES_WITHIN` — none on the seven chapters kept for the card (2026-10-09).
            (173, Dodge::PhilosophyOfTheHated),
            // **Card 174, 「サブタレイニアンローズ」, through each ring where its rose keeps clear**: rings of bullets going
            // out from the boss with a rose going round on each, each ring's the other way from the one before; the
            // rules' own were hit at the floor some eighteen times running, and meeting each ring at the place below
            // her nearest the run where its rose keeps clear all the while it goes over — its band read off its own
            // bullets, the way there weighed too — went through its 11,460 frames with no hit on the three chapters
            // kept as `hits-extra5/s7-c16-f45966`, `f46033` and `f46037` (r26, 2026-10-08).
            (174, Dodge::SubterraneanRose),
        ],
        stretches: vec![
            // **The Extra's first 1,500 frames, where there will be room**: the sunflower fairies at the top lay streams
            // of big bullets that curve down across onto the run, and the wiki's `どちらかのサイドで避け続けると安全` is a
            // side dodged along and not one stood against — which side the stream leaves changes with the seed, and the
            // rules' own and right of the middle were carried to the right wall and hit there, twice on
            // `hits-extra10/s7-c1-f593`, against the left wall three times up it on `hits-extra11/s7-c1-f633`; over the
            // four chapters kept, those two and `hits-extra6/s7-c1-f614` and `hits-extra8/s7-c1-f475`, against the left
            // wall, right of the middle and wherever it is emptiest took three hits each, and where there will be room,
            // each bullet carried on along its step, none (2026-10-08).
            //
            // **Where the red fairies come out and not where there will be room**, which it was: that was a way written
            // for no attack in particular, where the wiki's own for this stage is a place on the floor on one side kept
            // to — see `crate::cards::under_the_red_fairies`.
            (EXTRA, 0..1500, Dodge::WhereTheRedFairiesComeOut),
        ],
        chapters: vec![
            // **The stage after the Extra's midboss, against the left wall**, the strategy wiki's
            // `画面左端で密着してショットを撃つと1セット目をほとんど撃たれない` (wikiwiki.jp/thk/地/EX): fairies lined up at the top
            // left lay streams of rice that sweep down across to the bottom right, and the rules' own were carried along
            // the low line to the right wall and pinned in the bottom right corner, hit there two to four times on each
            // of `hits-extra9/s7-c5-f9708`, `f9852` and `hits-extra10/s7-c5-f9875`; against the left wall from stage frame
            // 9,400 to 10,600 they were hit once just after it, and to 11,400 or to 13,200 not at all on any of the three
            // (2026-10-08).
            //
            // **The whole chapter and not a stretch of the stage's frames from 9,400**, which it was: the midboss is
            // beaten anywhere from 9,198 to 9,612, and on a whole Extra (extra21) she was at 9,279, the rules carried the
            // run to the right in the 121 frames before the stretch, and it was pinned in the bottom right corner and hit
            // at `184,432` on 9,500; the whole chapter against the left wall took none on it, `hits-extra21/s7-c5-f9500`,
            // and on the three before (2026-10-08).
            //
            // **And not against the left wall all through it**, which it was: the wiki has the wall only for the first
            // set, and on the real case the_extra_played_to_its_clear_with_no_hit_and_no_bomb the later sets' streams
            // filled the bottom left and the run was hit there at `-182.1,432.0` 530 frames after the midboss, as it
            // was on each pick-up of that chapter, `hits-real1/s7-c5-f9780` (2026-10-08). The wiki's own way, the wall
            // for the first set, along to the right wall and up it into the room the sets leave there, took none on
            // that chapter and the four kept before; on `f9780` the run went up the right wall to `184,317` 470 frames
            // after the midboss, the last set's streams came down onto it there, and it went back to the bottom left by
            // 650 (2026-10-08).
            //
            // **Chapters 8 and 9 since the attack after each of the midboss's cards is a chapter of its own**, which were
            // 6 and 7 on the midstage rows of 2026-10-10 and chapter 5 before them: the Extra's row at 2,273 comes before
            // the midboss, her fight is chapters 3 to 7 — two nonspells, a card, a nonspell and a card — and the row at
            // 5,750 cuts the stage after her in two. Left at 6 and 7, this way was the midboss's last nonspell's and
            // card's, and the stage after her was the rules' own: the real case
            // `the_extra_played_to_its_clear_with_no_hit_and_no_bomb` was hit in chapter 8, 568 frames after she was
            // beaten (session `1791664944`, kept as `s7-c8-f10014`, 2026-10-11).
            (EXTRA, 8, None, Dodge::FromTheLeftWallToTheTopRight),
            (EXTRA, 9, None, Dodge::FromTheLeftWallToTheTopRight),
        ],
        after_cards: vec![
            // **The boss's fire after card 169, under the gap between its long rows**: the strategy wiki's
            // `ボス通常攻撃その6`, long rows of big bullets wound out from her, `通常3の弾列が5発から10発になったもの`, of
            // whose 通常3 it says
            // `左へ誘導した方が楽` (wikiwiki.jp/thk/地/EX); the rules' own were pinned in the bottom left corner and hit
            // there at `-184,432` on a whole Extra (extra20) and on each of three pick-ups of its chapter
            // `hits-extra20/s7-c11-f29560`, and side to side the same, where left of the middle took none on it and on
            // the ten chapters kept for card 169 (2026-10-08).
            //
            // **Under the gap between the long rows and not left of the middle alone**, which it was: that was a way
            // written for another card's fire, where this fire's own is the wiki's `縦の隙間` between rows `横に長い`.
            (169, Dodge::BetweenTheLongRows),
        ],
        ..Ways::default()
    }
}
