//! **The rules playing a laid-out th10, over what five of its cards are dodged by**: `土着神「宝永四年の赤蛙」`,
//! card 107, whose every grain of rice stands still and then bursts at the one age; `蛙狩「蛙は口ゆえ蛇に呑まるる」`,
//! card 104, whose every knot does the same at an age of its own; `神具「洩矢の鉄の輪」`, card 102, whose rings are
//! each a band of bullets of one age; and `源符「厭い川の翡翠」`, card 103, whose banks' bullets stand still and set off
//! at the run at the one age — which a bullet's age read out of the game's own field is what says; and
//! `天流「お天水の奇跡」`, card 90, cut back across against fire aimed at the run.

use crate::fake::th10::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// th10's own four directions and its focus key, as `Th10::button_bit` has them.
const UP: u16 = 0x010;
const DOWN: u16 = 0x020;
const LEFT: u16 = 0x040;
const RIGHT: u16 = 0x080;
const FOCUS: u16 = 0x004;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// **The place the player is put**: the middle across, the field being counted from its centre line, at the low
/// line the rules wait on — in the hundredths the player's place is kept in.
const MIDDLE: i32 = 0;
const LOW: i32 = 36_800;

/// **The card the way is named for**, and where the boss whose card it is stands.
const THE_RED_FROG: i32 = 107;
const THE_BOSS_AT: (f32, f32) = (0.0, 100.0);

/// Two slots the fake's own walk steps, one grain a side of the run.
const LEFT_OF_IT: usize = 10;
const RIGHT_OF_IT: usize = 11;

/// **How far a grain stands either side of the run**: an unfocused tap of four frames at `4.5` lands `6` off it,
/// clear of a grain's `2` and the run's `1`; and inside the `10` a grain about to burst is kept further off by.
/// A focused tap at `2` lands `16` off it, clear of both.
const A_GRAIN_OFF_THE_RUN: f32 = 24.0;
const A_GRAINS_SIZE: (f32, f32) = (4.0, 4.0);

/// **A grain's age a frame of the card's own after the frames it is placed over**: put at this, the grains are
/// read at `410` on the frame the move is judged on — inside the four before `412`, the age the card bursts them at.
const ABOUT_TO_BURST: i32 = 407;

/// **The way of card 107 keeps a burst's reach from rice about to burst**: a grain still on each side of the run
/// `24` off it, two frames from the age every grain of the card bursts at. Either unfocused tap across lands
/// within `10` more of one of them, so the run's own way to the side it is going is not taken — where a grain
/// that is not about to burst leaves that tap clear, and the place the way wants, at a side, is reached by it.
#[test]
fn the_rules_of_th10_keep_off_rice_about_to_burst_where_the_red_frog_of_hoei_four_is_dodged() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-off-rice-about-to-burst",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: the_red_frog_of_hoei_four, spell: 107 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        let low = LOW as f32 / 100.0;
        for (slot, x) in [
            (LEFT_OF_IT, -A_GRAIN_OFF_THE_RUN),
            (RIGHT_OF_IT, A_GRAIN_OFF_THE_RUN),
        ] {
            image.a_bullet_is_at(slot, (x, low), (0.0, 0.0), A_GRAINS_SIZE);
            image.the_bullet_has_been_out_for(slot, ABOUT_TO_BURST);
        }
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (LEFT | RIGHT) == 0 || word & FOCUS != 0,
            "the rules tapped across unfocused into a grain about to burst, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A bullet coming out over the run, three frames from turning solid**, laid at an age of `3` so that the frame
/// the move is judged on reads it at `6` — every bullet of th10's turns active at `9`.
const COMING_OUT_FOR: i32 = 3;
/// And a ring of eight active bullets round it at `24`, still, so that every move the run has is hit within some
/// five frames of going: further off than waiting is from the one coming out, nearer than anywhere clear.
const A_RINGS_SLOTS: std::ops::Range<usize> = 20..28;
const THE_RINGS_REACH: f32 = 24.0;
const THE_ONE_COMING_OUT: usize = 30;

/// **The rules do not wait inside a bullet coming out, its turn to solid read as a hit**: the run on the low line with
/// a bullet coming out on top of it, three frames from active, and every way out a few frames longer — so the move
/// that is hit latest is a step off it, where a bullet coming out read as a warning for good left waiting there the
/// one move nothing hits. The Extra's card 99 was hit that way at `158.3,373.7` 943 frames in, the rules pressing
/// nothing with `the move 0x000 was hit first on none … warned on 40`, under `a bullet not hitting yet at
/// 157.6,372.0`.
#[test]
fn the_rules_of_th10_step_off_a_bullet_coming_out_over_the_run() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-step-off-a-bullet-coming-out",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- play_until: { frames: 400 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        // Two frames in before anything is laid: the stage's first frame empties the bullet list.
        game.frames(2);
        let image = game.image();
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        let (x, y) = (MIDDLE as f32 / 100.0, LOW as f32 / 100.0);
        for (slot, at) in A_RINGS_SLOTS.zip(0..8) {
            let turn = at as f32 * std::f32::consts::FRAC_PI_4;
            let place = (
                x + THE_RINGS_REACH * turn.cos(),
                y + THE_RINGS_REACH * turn.sin(),
            );
            image.a_bullet_is_at(slot, place, (0.0, 0.0), (4.0, 4.0));
        }
        image.a_bullet_is_at(THE_ONE_COMING_OUT, (x, y), (0.0, 0.0), (4.0, 4.0));
        image.the_bullet_is_coming_out(THE_ONE_COMING_OUT);
        image.the_bullet_has_been_out_for(THE_ONE_COMING_OUT, COMING_OUT_FOR);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (UP | DOWN | LEFT | RIGHT) != 0,
            "the rules waited inside a bullet three frames from turning solid, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The snake that eats the croaking frog, the card**, whose every knot of bullets stands still and then bursts at
/// the one age, `131`.
const THE_SNAKE: i32 = 104;
/// **A knot of the card's**: six bullets in one place, in slots past the ones the fake's own walk steps, `12` over
/// the run on its own x — standing `9` clear of its box, more than the rules keep spare.
const A_KNOTS_SLOTS: std::ops::Range<usize> = 100..106;
const A_KNOT_OVER_THE_RUN: f32 = 12.0;
/// **A knot's age a frame of the card's own after the frames it is placed over**: put at this, the knot is read at
/// `128` on the frame the move is judged on — three before `131`, the age the card bursts its knots at.
const ABOUT_TO_GO: i32 = 125;
/// **Knots just laid in each of the four other boxes across the field**, a dozen bullets at rest at the low line in
/// the middle of each — so the box the run stands in is the emptiest, as it was where the rules were hit.
const THE_OTHER_BOXES: [f32; 4] = [-153.6, -76.8, 76.8, 153.6];
const A_CROWDS_BULLETS: usize = 12;
const THE_CROWDS_FIRST_SLOT: usize = 110;

/// **The rules step off a knot about to burst where the snake that eats the croaking frog is dodged**: the run
/// on the low line's middle with a knot of six `12` over it, three frames from the age every knot of the card goes
/// off at `4` a frame, and the box it stands in the emptiest — so a move is pressed, where projected standing the
/// knot leaves standing room to spare.
#[test]
fn the_rules_of_th10_step_off_a_knot_about_to_burst_where_the_snake_eats_the_croaking_frog_is_dodged()
 {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-step-off-a-knot-about-to-burst",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: the_snake_eats_the_croaking_frog, spell: 104 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        image.a_fight_is_up(SUWAKO_AT, 1000);
        image.a_card_is_up(THE_SNAKE);
        let low = LOW as f32 / 100.0;
        for slot in A_KNOTS_SLOTS {
            image.a_bullet_is_at(
                slot,
                (MIDDLE as f32, low - A_KNOT_OVER_THE_RUN),
                (0.0, 0.0),
                A_GRAINS_SIZE,
            );
            image.the_bullet_has_been_out_for(slot, ABOUT_TO_GO);
        }
        for (box_, x) in THE_OTHER_BOXES.into_iter().enumerate() {
            for bullet in 0..A_CROWDS_BULLETS {
                let slot = THE_CROWDS_FIRST_SLOT + box_ * A_CROWDS_BULLETS + bullet;
                image.a_bullet_is_at(slot, (x, low), (0.0, 0.0), A_GRAINS_SIZE);
                image.the_bullet_has_been_out_for(slot, 0);
            }
        }
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (LEFT | RIGHT | UP | DOWN) != 0,
            "the rules stood under a knot about to burst, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Moriya's iron rings, the card**, and the boss whose card it is, standing where she does through it.
const MORIYAS_IRON_RINGS: i32 = 102;
const SUWAKO_AT: (f32, f32) = (0.0, 224.0);
/// **Where the run waits under her**: on her x at the floor of the field, `432` — the player's own bounds.
const AT_THE_FLOOR: i32 = 43_200;
/// **A ring**: 48 bullets of one age on a circle `62` across its middle, the middle circle of the card's three,
/// in slots past the ones the fake's own walk steps so that it stands where it is put; coming down at the run
/// at `1.61` a frame, its middle `72` above the run, which puts its near edge `10` off it.
const A_RINGS_BULLETS: usize = 48;
const A_RINGS_FIRST_SLOT: usize = 100;
const A_RINGS_RADIUS: f32 = 62.0;
const A_RINGS_STEP: (f32, f32) = (0.0, 1.61);
const A_RINGS_MIDDLE_ABOVE_THE_RUN: f32 = 72.0;
const A_RINGS_AGE: i32 = 200;

/// **The way of card 102 goes from under the boss to above her as a ring is about to reach the run**: the run on
/// her x at the floor, and a ring of one age coming down at it with its near edge `10` off — so the place it wants
/// is the side of her, `96` to her left at her own height, on its way to the top.
#[test]
fn the_rules_of_th10_go_round_the_boss_to_above_her_as_a_ring_reaches_the_run_where_moriyas_iron_rings_is_dodged()
 {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-go-above-suwako-as-a-ring-comes",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: moriyas_iron_rings, spell: 102 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        image.the_player_is_at_hundredths(MIDDLE, AT_THE_FLOOR);
        image.a_fight_is_up(SUWAKO_AT, 1000);
        image.a_card_is_up(MORIYAS_IRON_RINGS);
        game.frames(2);
        let floor = AT_THE_FLOOR as f32 / 100.0;
        let middle = (MIDDLE as f32, floor - A_RINGS_MIDDLE_ABOVE_THE_RUN);
        for bullet in 0..A_RINGS_BULLETS {
            let angle = std::f32::consts::TAU * bullet as f32 / A_RINGS_BULLETS as f32;
            let at = (
                middle.0 + A_RINGS_RADIUS * angle.cos(),
                middle.1 + A_RINGS_RADIUS * angle.sin(),
            );
            let slot = A_RINGS_FIRST_SLOT + bullet;
            image.a_bullet_is_at(slot, at, A_RINGS_STEP, (4.0, 4.0));
            image.the_bullet_has_been_out_for(slot, A_RINGS_AGE);
        }
        game.frames(1);
        let wanted = game
            .log()
            .lines()
            .iter()
            .rev()
            .find_map(|line| {
                let (_, at) =
                    line.split_once("wanted the place the card's own dodge puts it at ")?;
                let (x, rest) = at.split_once(',')?;
                let y = rest.split(' ').next()?;
                Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))
            })
            .unwrap_or_else(|| {
                panic!(
                    "the rules wanted no place of the way's:\n  {}",
                    game.log().lines().join("\n  ")
                )
            });
        assert_eq!(
            wanted,
            (SUWAKO_AT.0 - 96.0, SUWAKO_AT.1),
            "the rules did not make for the boss's side as the ring came:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The jade of the loathsome river, the card**, and where its boss stands through it.
const THE_JADE_OF_THE_LOATHSOME_RIVER: i32 = 103;
const SUWAKO_OVER_THE_RIVER_AT: (f32, f32) = (0.0, 100.0);
/// **Where the card's three rivers cross**: the middle across, `280` down a field whose floor is `432`.
const WHERE_THE_RIVERS_CROSS: i32 = 28_000;
/// **Two bullets of a river's banks, at rest level with the run either side of it** — in slots past the ones the
/// fake's own walk steps, so that they stand where they are put.
const THE_BANKS: [(usize, f32); 2] = [(100, -40.0), (101, 40.0)];
/// **The age the card sets a bank's bullet off at**, at `1.40` a frame straight at the run.
const SETS_OFF_AT: i32 = 226;
/// How many frames the run is watched for.
const WATCHED_FOR: usize = 8;

/// **Every word the keys read over [`WATCHED_FOR`] frames of card 103 dodged its own way**, with the run where the
/// rivers cross and a bank's bullet at rest either side of it, `40` off and `age` frames old — handed to `judge`
/// with the log, in the test's own process.
fn the_words_beside_the_banks(name: &str, age: i32, judge: impl FnOnce(&[u16], &str)) {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            name,
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: jade_of_the_loathsome_river, spell: 103 }\n\
                 - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        // The stage's start, which empties the enemies' list and leaves no fight laid before it.
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, WHERE_THE_RIVERS_CROSS);
        image.a_fight_is_up(SUWAKO_OVER_THE_RIVER_AT, 1000);
        image.a_card_is_up(THE_JADE_OF_THE_LOATHSOME_RIVER);
        // And the frame the card is first read on, which is still the rules' own.
        game.frames(1);
        image.the_player_is_at_hundredths(MIDDLE, WHERE_THE_RIVERS_CROSS);
        let level = WHERE_THE_RIVERS_CROSS as f32 / 100.0;
        for (slot, x) in THE_BANKS {
            image.a_bullet_is_at(slot, (x, level), (0.0, 0.0), (4.0, 4.0));
            image.the_bullet_has_been_out_for(slot, age);
        }
        let mut words = Vec::new();
        for _ in 0..WATCHED_FOR {
            // The card's own frames, which the game's enemy update steps and a fight laid out by hand does not.
            image.the_fight_goes_on();
            game.frames(1);
            words.push(image.the_words_the_keys_read().0);
        }
        judge(&words, &game.log().lines().join("\n  "));
    });
}

/// **The way of card 103 stands where the rivers cross while no bank's bullet is about to set off at the run**:
/// the two either side of it a whole card's letting go from setting off, nothing is pressed in any direction.
#[test]
fn the_rules_of_th10_stand_where_the_rivers_cross_while_the_banks_are_at_rest_where_jade_of_the_loathsome_river_is_dodged()
 {
    the_words_beside_the_banks(
        "th10-the-rules-stand-where-the-rivers-cross",
        0,
        |words, log| {
            assert!(
                words
                    .iter()
                    .all(|word| word & (LEFT | RIGHT | UP | DOWN) == 0),
                "the rules moved off where the rivers cross with the banks at rest, {words:#06x?}:\n  {log}",
            );
        },
    );
}

/// **Where the run is put below the rivers' crossing**: `80` under it, on the middle across.
const UNDER_WHERE_THE_RIVERS_CROSS: i32 = 36_000;
/// **How many updates each drawn frame runs**, the speed the real cases play a whole game at.
const UPDATES_A_FRAME: u32 = 4;

/// **The way of card 103 takes the run towards where the rivers cross at four updates a frame as well**: put `80`
/// under the crossing with nothing of the card's fire up, and the fight's timer stepped every update as the game's
/// own enemy update steps it, the run is within `40` of the crossing after `40` drawn frames — `160` updates, in
/// which the slowest of the way's paces goes `64`, and the way stands once that pace would end further from the
/// crossing than standing leaves it, `32` under it.
#[test]
fn the_rules_of_th10_go_to_where_the_rivers_cross_at_four_updates_a_frame_where_jade_of_the_loathsome_river_is_dodged()
 {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-go-to-where-the-rivers-cross-at-four-updates-a-frame",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: jade_of_the_loathsome_river, spell: 103 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.speed = UPDATES_A_FRAME;
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, UNDER_WHERE_THE_RIVERS_CROSS);
        image.a_fight_is_up(SUWAKO_OVER_THE_RIVER_AT, 1000);
        image.a_card_is_up(THE_JADE_OF_THE_LOATHSOME_RIVER);
        game.steps_the_fight_laid_out();
        game.frames(40);
        let (x, y) = image.where_the_player_is();
        let crossing = WHERE_THE_RIVERS_CROSS as f32 / 100.0;
        assert!(
            x.hypot(y - crossing) < 40.0,
            "the run was at {x:.1},{y:.1} and not near where the rivers cross, {crossing:.0} down:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And goes on from there as a bank's bullet is about to set off at it**: the two either side of it a frame from
/// the age they set off at, the run is moved before they are let go at it.
#[test]
fn the_rules_of_th10_go_on_from_where_the_rivers_cross_as_a_banks_bullet_is_about_to_set_off_where_jade_of_the_loathsome_river_is_dodged()
 {
    the_words_beside_the_banks(
        "th10-the-rules-go-on-as-a-bank-sets-off",
        SETS_OFF_AT - 1,
        |words, log| {
            assert!(
                words
                    .iter()
                    .any(|word| word & (LEFT | RIGHT | UP | DOWN) != 0),
                "the rules stood where the rivers cross with the banks about to set off at the run, {words:#06x?}:\n  {log}",
            );
        },
    );
}

/// **The miracle of Otensui, the card**, and where its boss stands through it.
const THE_MIRACLE_OF_OTENSUI: i32 = 90;
const KANAKO_AT: (f32, f32) = (0.0, 120.0);
/// **Where the dodging has carried the run**: `100` right of the middle, within the `96` of the right wall a cut back
/// starts at, on the way's own low line, `410`.
const CARRIED_TO: (i32, i32) = (10_000, 41_000);
/// **Where the swing goes, and where the way back from it**: `40` on towards the wall and `32` back past where it set
/// off, both up off the low line at `350`.
const SWUNG_TO: (f32, f32) = (140.0, 350.0);
const BACK_TO: (f32, f32) = (68.0, 350.0);
/// **The card's fire coming at the run from where it has been**: a column of bullets `60` back towards the middle,
/// from `310` to `390` down, going right at the card's `1.2` a frame — so that standing the run is reached in 50
/// frames, past the forty the rules look ahead, while every move left goes into it within them.
const THE_FIRES_SLOTS: std::ops::Range<usize> = 100..111;
const THE_FIRE_BEHIND_BY: f32 = 60.0;
const THE_FIRES_TOP: f32 = 310.0;
const THE_FIRES_SPACING: f32 = 8.0;
const THE_FIRES_STEP: (f32, f32) = (1.2, 0.0);

/// **How far off a place the run is let be and still be said to want it**: one of its whole steps, `4.5` — the run
/// having moved that much on the frame before the card is up.
const WITHIN_A_STEP: f32 = 4.5;

/// **Whether the last place the rules said the way wanted is within a step of `place`**, out of the log.
fn the_way_last_wanted(log: &[String], place: (f32, f32)) -> bool {
    log.iter()
        .rev()
        .find_map(|line| {
            let (_, at) = line.split_once("wanted the place the card's own dodge puts it at ")?;
            let (x, rest) = at.split_once(',')?;
            let y = rest.split(' ').next()?;
            Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))
        })
        .is_some_and(|(x, y)| {
            (x - place.0).abs() <= WITHIN_A_STEP && (y - place.1).abs() <= WITHIN_A_STEP
        })
}

/// **The way of card 90 cuts back short of a wall, up and across against the fire**: carried within reach of the right
/// wall, the run is sent on towards it, up off the low line; and there, with the card's fire coming at it from where
/// it has been, it goes straight back the other way — every move left being hit within the forty frames the rules
/// look ahead, and none within the six a crossing is judged over.
#[test]
fn the_rules_of_th10_swing_up_towards_a_wall_and_cut_back_against_the_fire_where_the_miracle_of_otensui_is_dodged()
 {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-cut-back-from-a-wall",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: the_miracle_of_otensui, spell: 90 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        image.the_player_is_at_hundredths(CARRIED_TO.0, CARRIED_TO.1);
        image.a_fight_is_up(KANAKO_AT, 1000);
        image.a_card_is_up(THE_MIRACLE_OF_OTENSUI);
        game.steps_the_fight_laid_out();
        game.frames(2);
        assert!(
            the_way_last_wanted(&game.log().lines(), SWUNG_TO),
            "the rules did not swing on towards the wall:\n  {}",
            game.log().lines().join("\n  "),
        );

        image.the_player_is_at_hundredths((SWUNG_TO.0 * 100.0) as i32, (SWUNG_TO.1 * 100.0) as i32);
        for (row, slot) in THE_FIRES_SLOTS.enumerate() {
            let at = (
                SWUNG_TO.0 - THE_FIRE_BEHIND_BY,
                THE_FIRES_TOP + THE_FIRES_SPACING * row as f32,
            );
            image.a_bullet_is_at(slot, at, THE_FIRES_STEP, (4.0, 4.0));
            image.the_bullet_has_been_out_for(slot, 100);
        }
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            the_way_last_wanted(&game.log().lines(), BACK_TO),
            "the rules did not go back from the swing:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            word & LEFT != 0,
            "the rules did not cut back against the fire, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A player's path through a card, as a recipe writes it**: four frames apart, from the low line's middle along it
/// to the right at the whole step's `4.5` a frame, `18` a place, for 40 frames.
const A_PLAYERS_PATH: &str = "[[0.0, 368.0], [18.0, 368.0], [36.0, 368.0], [54.0, 368.0], [72.0, 368.0], [90.0, 368.0], \
                              [108.0, 368.0], [126.0, 368.0], [144.0, 368.0], [162.0, 368.0]]";

/// **The rules go along a player's path through the card it is written for**, frame for frame: put on the path's
/// first place as the card starts, the run is where the path has it 30 frames on, at the whole step the path goes
/// at — where its own rules would have stood under the boss.
#[test]
fn the_rules_of_th10_go_along_a_players_path_through_the_card_a_recipe_follows_it_in() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-go-along-a-players-path",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- follow: {{ spell: {THE_RED_FROG}, every: 4, path: {A_PLAYERS_PATH} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        game.steps_the_fight_laid_out();
        game.frames(30);
        // Where the path is on the card's frame 30, between its places on 28 and 32 — the card having come up a frame
        // or two into those 30, within a few steps of it.
        let (x, y) = image.where_the_player_is();
        assert!(
            (x - 117.0).abs() < 4.0 * 4.5 && (y - LOW as f32 / 100.0).abs() < 4.0,
            "the run was at {x:.1},{y:.1} and not along the path, some 117,368:\n  {}",
            game.log().lines().join("\n  "),
        );
        assert!(
            game.log()
                .said("wanted the place on the player's path it follows at"),
            "the rules did not say they went along the path:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where th10's stage 4 midboss is waited for on the floor**: `25` right of the middle, and no move taken further
/// across than `40` from there — the run `39` right of the place with bullets resting `9` to its left and up and to
/// its left and one falling on it from above, the steps right that are clear of all three landing more than `40` off
/// the place and not taken.
#[test]
fn the_rules_of_th10_keep_on_the_floor_by_the_middle_where_a_dodge_would_take_them_off_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-on-the-floor-by-the-middle",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: on_the_floor_by_the_middle, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let floor = 432.0;
        image.the_player_is_at_hundredths(6400, 43_200);
        image.a_bullet_is_at(0, (55.0, floor), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (64.0, floor - 32.0), (0.0, 2.3), (4.0, 4.0));
        image.a_bullet_is_at(2, (57.0, floor - 12.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the floor's place by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A third of the way up th10's field, and no move taken lower than `32` below it**: the run `20` below that height
/// with bullets resting either side of it and up either side, and one falling on it from `40` above at `3` a frame —
/// so standing is hit some eleven frames on, and the steps down, clear of all of them, land lower than that and are
/// not taken.
#[test]
fn the_rules_of_th10_keep_a_third_of_the_way_up_where_a_dodge_would_take_them_lower() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-a-third-of-the-way-up",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: a_third_of_the_way_up_and_no_lower, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let y = 432.0 - 400.0 / 3.0 + 20.0;
        image.the_player_is_at_hundredths(0, (y * 100.0) as i32);
        image.a_bullet_is_at(0, (-9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(2, (-9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(3, (9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(4, (0.0, y - 40.0), (0.0, 3.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & DOWN == 0,
            "the rules stepped down, below a third of the way up by more than 32, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And no move taken further across than `40` from the middle**: the run `39` right of the middle with bullets
/// resting `9` to its left and up and to its left and one falling on it from above, the steps right that are clear of
/// all three landing more than `40` off the middle and not taken.
#[test]
fn the_rules_of_th10_keep_a_third_of_the_way_up_by_the_middle_where_a_dodge_would_take_them_off_it()
{
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-a-third-of-the-way-up-by-the-middle",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: a_third_of_the_way_up_and_no_lower, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let floor = 432.0;
        image.the_player_is_at_hundredths(3900, 43_200);
        image.a_bullet_is_at(0, (30.0, floor), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (39.0, floor - 32.0), (0.0, 2.3), (4.0, 4.0));
        image.a_bullet_is_at(2, (32.0, floor - 12.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the middle by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Low in th10's field's middle, and no move taken further across than `40` from it**: the run `39` right of the
/// middle with bullets resting `9` to its left and up and to its left and one falling on it from above, the steps
/// right that are clear of all three landing more than `40` off the middle and not taken.
#[test]
fn the_rules_of_th10_keep_low_in_the_middle_where_a_dodge_would_take_them_off_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-low-in-the-middle",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: low_in_the_middle, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let floor = 432.0;
        image.the_player_is_at_hundredths(3900, 43_200);
        image.a_bullet_is_at(0, (30.0, floor), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (39.0, floor - 32.0), (0.0, 2.3), (4.0, 4.0));
        image.a_bullet_is_at(2, (32.0, floor - 12.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the middle by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where th10's `蛙狩「蛙は口ゆえ蛇に呑まるる」` is dodged, no move taken further across than `40` from the middle**:
/// the run `39` right of the middle with bullets resting `9` to its left and up and to its left and one falling on it
/// from above, the steps right that are clear of all three landing more than `40` off the middle and not taken.
#[test]
fn the_rules_of_th10_keep_by_the_middle_where_the_snake_eats_the_croaking_frog_is_dodged() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-by-the-middle-against-the-snake",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: the_snake_eats_the_croaking_frog, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let floor = 432.0;
        image.the_player_is_at_hundredths(3900, 43_200);
        image.a_bullet_is_at(0, (30.0, floor), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (39.0, floor - 32.0), (0.0, 2.3), (4.0, 4.0));
        image.a_bullet_is_at(2, (32.0, floor - 12.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, off the middle by more than 40, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where th10's `土着神「手長足長さま」` is dodged, the run goes under the gap its slow ring leaves nearest it as the
/// ring comes down on it**: the run `40` right of the middle at its foot with a ring's bullets `30` above it, `12`
/// apart and stepping `0.5` a frame, one of them `0.5` left of the middle — the way wants the gap at `41.5`, and not the
/// middle or the gap nearest it.
#[test]
fn the_rules_of_th10_go_under_the_gap_in_the_slow_ring_coming_down_where_tenaga_ashinaga_is_dodged()
{
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-go-under-the-gap-in-the-slow-ring",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: tenaga_ashinaga, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let foot = 32.0 + 325.0;
        image.the_player_is_at_hundredths(4000, (foot * 100.0) as i32);
        for slot in 0..9 {
            let x = -0.5 + 12.0 * (slot as f32 - 4.0);
            image.a_bullet_is_at(slot, (x, foot - 30.0), (0.0, 0.5), (4.0, 4.0));
            image.the_bullet_has_been_out_for(slot, 425);
        }
        game.frames(1);
        assert!(
            the_way_last_wanted(&game.log().lines(), (41.5, foot)),
            "the rules did not go under the ring's gap nearest the run:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And no move taken lower once up through it**: the run `8` above the ring's bullets, `12` apart either side of
/// it, with bullets resting either side of it and up either side and one falling on it from `40` above at `3` a frame —
/// so standing is hit some eleven frames on, and the step down through the gap below it, clear of all of them and
/// towards the way's place at the foot, is not taken.
#[test]
fn the_rules_of_th10_keep_above_the_slow_ring_once_up_through_it_where_tenaga_ashinaga_is_dodged() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-above-the-slow-ring",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: tenaga_ashinaga, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let y = 340.0;
        image.the_player_is_at_hundredths(MIDDLE, (y * 100.0) as i32);
        image.a_bullet_is_at(0, (-9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(2, (-9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(3, (9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(4, (0.0, y - 40.0), (0.0, 3.0), (4.0, 4.0));
        for (slot, x) in [(5, -18.0), (6, -6.0), (7, 6.0), (8, 18.0)] {
            image.a_bullet_is_at(slot, (x, y + 8.0), (0.0, 0.5), (4.0, 4.0));
            image.the_bullet_has_been_out_for(slot, 425);
        }
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & DOWN == 0,
            "the rules stepped down, back through the slow ring, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Where th10's `「諏訪大戦　～ 土着神話 vs 中央神話」` is dodged by the left wall, no move taken further out from it
/// than `72`**: the run `71` out from the wall with bullets resting `9` to its left and up and to its left and one
/// falling on it from above, the steps right that are clear of all three landing more than `72` out and not taken.
#[test]
fn the_rules_of_th10_keep_by_the_left_wall_where_a_dodge_would_take_them_off_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-by-the-left-wall",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: great_suwa_war, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let (floor, x) = (432.0, -184.0 + 71.0);
        image.the_player_is_at_hundredths((x * 100.0) as i32, 43_200);
        image.a_bullet_is_at(0, (x - 9.0, floor), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (x, floor - 32.0), (0.0, 2.3), (4.0, 4.0));
        image.a_bullet_is_at(2, (x - 7.0, floor - 12.0), (0.0, 0.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & RIGHT == 0,
            "the rules stepped right, out from the left wall by more than 72, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Up the middle, once card 108 is past its wall, no move taken more than a step below the height**: the run `8`
/// above that step's foot, `0,136`, with bullets resting either side of it and up either side and one falling on it
/// from `40` above at `3` a frame — so standing is hit some eleven frames on, and the steps down, clear of all of them,
/// land below the height's step and are not taken.
#[test]
fn the_rules_of_th10_keep_up_the_middle_where_the_great_suwa_war_has_left_the_wall() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-up-the-middle-in-the-great-suwa-war",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: great_suwa_war, spell: 108 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.a_fight_is_up(SUWAKO_AT, 1000);
        image.a_card_is_up(108);
        game.steps_the_fight_laid_out();
        image.the_fight_has_attacked_for(4_100);
        game.frames(1);
        let (x, y) = (0.0, 32.0 + 80.0 + 32.0 - 8.0);
        image.the_player_is_at_hundredths((x * 100.0) as i32, (y * 100.0) as i32);
        image.a_bullet_is_at(0, (x - 9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (x + 9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(2, (x - 9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(3, (x + 9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(4, (x, y - 40.0), (0.0, 3.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & DOWN == 0,
            "the rules stepped down, below the height up the middle, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And by the wall no move taken below the low line**: the run `30` out from the wall and `8` above the low line
/// with bullets resting either side of it and up either side, and one falling on it from `40` above at `3` a frame —
/// so standing is hit some eleven frames on, and the steps down, clear of all of them, land below the low line and
/// are not taken.
#[test]
fn the_rules_of_th10_keep_by_the_left_wall_where_a_dodge_would_take_them_below_the_low_line() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-by-the-left-wall-above-the-low-line",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- dodge: { with: great_suwa_war, stage: 1, from: 0, to: 9000 }\n\
                     - play_until: { frames: 400 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        let (x, y) = (-184.0 + 30.0, 432.0 - 64.0 - 8.0);
        image.the_player_is_at_hundredths((x * 100.0) as i32, (y * 100.0) as i32);
        image.a_bullet_is_at(0, (x - 9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(1, (x + 9.0, y), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(2, (x - 9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(3, (x + 9.0, y - 12.0), (0.0, 0.0), (4.0, 4.0));
        image.a_bullet_is_at(4, (x, y - 40.0), (0.0, 3.0), (4.0, 4.0));
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & DOWN == 0,
            "the rules stepped down, below the low line by the left wall, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A player's path that stands still**, on the low line's middle, for 40 frames.
const A_PLAYERS_PATH_THAT_STANDS: &str = "[[0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], \
                                          [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0], [0.0, 368.0]]";

/// **And still dodge what was fired before the card**, which the path was never fired at: a bullet out for 500
/// frames falling on the run where the path has it stand, and the rules step aside of it — the bullet of the attack
/// before `神具「洩矢の鉄の輪」` the run was hit by 26 frames into the card on the path's own place.
#[test]
fn the_rules_of_th10_on_a_players_path_step_aside_of_fire_from_before_the_card() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-on-a-path-step-aside-of-fire-from-before",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- follow: {{ spell: {THE_RED_FROG}, every: 4, path: {A_PLAYERS_PATH_THAT_STANDS} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        game.steps_the_fight_laid_out();
        game.frames(2);
        image.a_bullet_is_at(0, (0.0, LOW as f32 / 100.0 - 20.0), (0.0, 2.3), (4.0, 4.0));
        image.the_bullet_has_been_out_for(0, 500);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (LEFT | RIGHT) != 0,
            "the rules stood on the path under fire from before the card, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **But not what was fired before and is going away from the run**: a bullet out for 500 frames `60` behind it
/// and going on away at `2.3` a frame, and the run is where the path has it 30 frames on at the whole step, as it
/// is with nothing behind it.
#[test]
fn the_rules_of_th10_on_a_players_path_keep_to_it_past_fire_from_before_going_away() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-on-a-path-keep-to-it-past-fire-going-away",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- follow: {{ spell: {THE_RED_FROG}, every: 4, path: {A_PLAYERS_PATH} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        image.a_bullet_is_at(0, (-60.0, LOW as f32 / 100.0), (-2.3, 0.0), (4.0, 4.0));
        image.the_bullet_has_been_out_for(0, 500);
        game.steps_the_fight_laid_out();
        game.frames(30);
        let (x, y) = image.where_the_player_is();
        assert!(
            (x - 117.0).abs() < 4.0 * 4.5 && (y - LOW as f32 / 100.0).abs() < 4.0,
            "the run was at {x:.1},{y:.1} and not along the path, some 117,368:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the card's own way once the path is over**, and not the path's last place: a path of three places, eight
/// frames, and 30 frames on the rules want what the recipe's dodge for the card has them want.
#[test]
fn the_rules_of_th10_go_the_cards_own_way_once_a_players_path_through_it_is_over() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-go-the-cards-way-once-the-path-is-over",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- dodge: {{ with: in_the_middle, spell: {THE_RED_FROG} }}\n\
                     - follow: {{ spell: {THE_RED_FROG}, every: 4, path: [[0.0, 368.0], [18.0, 368.0], [36.0, 368.0]] }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        game.steps_the_fight_laid_out();
        game.frames(30);
        let last = game
            .log()
            .lines()
            .into_iter()
            .rev()
            .find(|line| line.contains("autoplay: wanted"))
            .unwrap_or_default();
        assert!(
            !last.contains("the place on the player's path"),
            "the rules still went along the path once it was over: {last}\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And what was fired at the run on its way to the path**, which the path was never fired at either: put `30`
/// right of where the path stands as the card starts, the run is on it some five frames later, and a bullet out
/// for 27 of the card's 30 frames falling on it there is stepped aside of — the rings thrown at the run on its way
/// that hit it 506 frames into `神具「洩矢の鉄の輪」` on the path's own place.
#[test]
fn the_rules_of_th10_on_a_players_path_step_aside_of_fire_from_before_the_run_was_on_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-on-a-path-step-aside-of-fire-from-before-they-were-on-it",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- follow: {{ spell: {THE_RED_FROG}, every: 4, path: {A_PLAYERS_PATH_THAT_STANDS} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE + 3000, LOW);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        game.steps_the_fight_laid_out();
        game.frames(30);
        let (x, y) = image.where_the_player_is();
        assert!(
            x.abs() < 8.0,
            "the run was at {x:.1},{y:.1} and not on the path at 0,368:\n  {}",
            game.log().lines().join("\n  "),
        );
        image.a_bullet_is_at(0, (x, y - 20.0), (0.0, 2.3), (4.0, 4.0));
        image.the_bullet_has_been_out_for(0, 27);
        game.frames(1);
        let (word, _) = image.the_words_the_keys_read();
        assert!(
            word & (LEFT | RIGHT) != 0,
            "the rules stood on the path under fire from before they were on it, {word:#06x}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **A player's path that turns at the whole step**: down the field from the low line's middle, `18` a place four
/// frames apart, and then down and to the right at `12.7` a place each way — the turn the user's own path through
/// `神具「洩矢の鉄の輪」` makes off the left wall.
const A_PLAYERS_PATH_THAT_TURNS: &str = "[[0.0, 300.0], [0.0, 318.0], [0.0, 336.0], [0.0, 354.0], [0.0, 372.0], \
                                         [12.7, 384.7], [25.4, 397.4], [38.1, 410.1], [50.8, 422.8]]";

/// **And keep to it through its turn**, at the whole step the turn goes at and not the focused one: 30 frames
/// in, the path is `25` down the diagonal, which the focused step's `1.41` each way a frame is twice as far short of
/// as a path is gone along within — **with a bullet resting on the floor ahead of the diagonal**, which the whole
/// step carried on along it for the frames looked at reaches and the focused one does not, the bullet the user's
/// run off the wall had ahead of it.
#[test]
fn the_rules_of_th10_keep_to_a_players_path_at_the_whole_step_through_a_turn() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-rules-keep-to-a-players-path-through-a-turn",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(format!(
                    "- follow: {{ spell: {THE_RED_FROG}, every: 4, path: {A_PLAYERS_PATH_THAT_TURNS} }}\n\
                     - play_until: {{ frames: 400 }}\n"
                ));
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        game.frames(2);
        image.the_player_is_at_hundredths(MIDDLE, 30000);
        image.a_fight_is_up(THE_BOSS_AT, 1000);
        image.a_card_is_up(THE_RED_FROG);
        image.a_bullet_is_at(0, (100.0, 432.0), (0.0, 0.0), (4.0, 4.0));
        game.steps_the_fight_laid_out();
        game.frames(30);
        // Where the path is on the card's frame 30, between its places on 28 and 32 — the card having come up a frame
        // or two into those 30, within a couple of steps of it.
        let (x, y) = image.where_the_player_is();
        let (there_x, there_y) = (25.4 + 12.7 / 2.0, 397.4 + 12.7 / 2.0);
        assert!(
            (x - there_x).hypot(y - there_y) < 2.0 * 4.5,
            "the run was at {x:.1},{y:.1} and not along the path's turn, some {there_x:.0},{there_y:.0}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **An enemy of a wave shot down is no motion of the one behind it**, though the two came on the same frame and
/// their attacks' timers agree: two enemies laid far apart, the first taken off the list, so that the second moves
/// into its place on it — and the run, with nothing near it, keeps every one of its nine moves open. Where the step
/// between the two places was read as the second enemy's motion, the fear spread over it covered the field: the
/// Extra's stretch before Suwako read `an enemy at -143.0,224.0 stepping 3.0,0.0, feared out to 306.0,200.0` over a
/// hit with every one of the nine moves at some `-70 of room`, at stage frame 11,643 at 2x and at 16x alike.
#[test]
fn an_enemy_of_th10_shot_down_is_no_motion_of_the_one_behind_it() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-an-enemy-shot-down-is-no-motion",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- play_until: { frames: 400 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        // After the stage's first frame, whose start empties the list down to `main`.
        game.frames(2);
        image.a_wave_is_up(2);
        // Laid so that the step between the two places carried on from the second lands on the run: `75,216` and on
        // by `-75,152` is `0,368`.
        image.the_waves_enemy_is_at(0, (150.0, 64.0));
        image.the_waves_enemy_is_at(1, (75.0, 216.0));
        for _ in 0..4 {
            image.the_waves_attacks_go_on();
            game.frames(1);
        }
        let log = game.log();
        let from = log.written();
        image.the_waves_enemy_is_shot_down(0);
        for _ in 0..2 {
            image.the_waves_attacks_go_on();
            image.the_player_is_at_hundredths(MIDDLE, LOW);
            game.frames(1);
        }
        assert!(
            log.said_since(from, "of the nine clear"),
            "the rules said nothing of their moves after the enemy was shot down:\n  {}",
            log.lines()[from..].join("\n  "),
        );
        assert!(
            !log.said_since(from, " 0 of the nine clear"),
            "the enemy behind the one shot down was feared over the step between their places:\n  {}",
            log.lines()[from..].join("\n  "),
        );
    });
}

/// **A card the autoplay engine has a way for in th10's Normal**, stage 5's 62, which it dodges under the boss.
const A_CARD_OF_THE_NORMAL: i32 = 62;
/// The difficulties a run is played on, as th10 numbers them.
const NORMAL: i32 = 1;
const HARD: i32 = 2;

/// A run of `difficulty` played to `recipe` with that card up, two frames in, and the log it left.
fn the_rules_dodging_a_card_of_the_normal(
    case: &str,
    recipe: &str,
    difficulty: i32,
) -> Vec<String> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        case,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(recipe.to_owned());
            config.log_level = LogLevel::Verbose;
        },
    );
    game.is_in_a_run();
    let image = game.image();
    image.the_difficulty_is(difficulty);
    image.the_player_is_at_hundredths(MIDDLE, LOW);
    image.a_fight_is_up(THE_BOSS_AT, 1000);
    image.a_card_is_up(A_CARD_OF_THE_NORMAL);
    game.frames(2);
    game.log().lines()
}

/// **A card of a Normal run is dodged the way the autoplay engine names for it**, with no step of the recipe naming
/// one.
#[test]
fn the_rules_of_th10_dodge_a_card_of_a_normal_run_the_way_the_autoplay_engine_names() {
    in_its_own_process(|| {
        let lines = the_rules_dodging_a_card_of_the_normal(
            "th10-the-rules-dodge-the-way-the-engine-names",
            "- play_until: { frames: 400 }\n",
            NORMAL,
        );
        let said =
            "card 62 is up, and the rules dodge it under_the_boss as the autoplay engine names";
        assert!(
            lines.iter().any(|line| line.contains(said)),
            "not said, {said}:\n  {}",
            lines.join("\n  "),
        );
    });
}

/// **And the way a recipe names for the same card is taken in place of the engine's.**
#[test]
fn the_rules_of_th10_dodge_a_card_the_way_the_recipe_names_over_the_autoplay_engines() {
    in_its_own_process(|| {
        let lines = the_rules_dodging_a_card_of_the_normal(
            "th10-the-rules-dodge-the-way-the-recipe-names-over-the-engine",
            "- dodge: { with: in_the_middle, spell: 62 }\n- play_until: { frames: 400 }\n",
            NORMAL,
        );
        let said = "card 62 is up, and the rules dodge it in_the_middle as the recipe names";
        assert!(
            lines.iter().any(|line| line.contains(said)),
            "not said, {said}:\n  {}",
            lines.join("\n  "),
        );
    });
}

/// **A conversation is skipped**, which is th10's own Ctrl — the bit `0x100` of the word, which runs a conversation on
/// where it lets one be (0x416a3a) — as th07's bot skips that game's: a bot that sat through every one of them
/// would spend the afternoon it was written to save.
#[test]
fn the_bot_skips_a_conversation_of_th10() {
    const SKIP: u16 = 0x100;
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th10-the-bot-skips-a-conversation",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("- play_until: { frames: 400 }\n".to_owned());
                config.log_level = LogLevel::Verbose;
            },
        );
        game.is_in_a_run();
        let image = game.image();
        image.the_player_is_at_hundredths(MIDDLE, LOW);
        game.frames(2);
        let (word, _) = image.the_words_the_keys_read();
        assert_eq!(word & SKIP, 0, "the bot skipped with no conversation up");

        image.a_conversation_is_up();
        game.frames(2);
        let (word, _) = image.the_words_the_keys_read();
        assert_eq!(
            word & SKIP,
            SKIP,
            "the bot did not skip the conversation:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And a run of a difficulty the engine has no way written for takes none**, the same card up in a Hard run.
#[test]
fn the_rules_of_th10_dodge_a_card_of_a_hard_run_by_no_way_of_the_autoplay_engines() {
    in_its_own_process(|| {
        let lines = the_rules_dodging_a_card_of_the_normal(
            "th10-the-rules-of-a-hard-run-dodge-no-way-the-engine-names",
            "- play_until: { frames: 400 }\n",
            HARD,
        );
        assert!(
            !lines
                .iter()
                .any(|line| line.contains("card 62 is up, and the rules dodge it")),
            "a way was taken:\n  {}",
            lines.join("\n  "),
        );
    });
}
