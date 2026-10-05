//! **How the rules play th06c's cards they are told a way of dodging**, over the laid-out game — what each way
//! wants on the frames of its card, read off the rules' own `wanted` line.

use std::f32::consts::FRAC_PI_2;

use crate::fake::th06c::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;

/// What the game's own frame is declared to take.
const WORK_US: i64 = 700;

/// `禁忌「レーヴァテイン」`, by the index the card's own start shifts into the records by — `spell=55` in the
/// state lines of a launch that played it.
const LAEVATEINN: u32 = 55;

/// **The sword's head, pointing straight down**, as a launch read it: `6.0` of half across is `24` of the
/// width the game keeps, and it runs `32` to `400` along.
const THE_HEADS_WIDTH: f32 = 24.0;
const ALONG: (f32, f32) = (32.0, 400.0);

/// The phases of a laser, as `game::th06c`'s own reading numbers them: drawn and warning, firing, fading.
const WARNING: u8 = 0;
const FIRING: u8 = 1;
const FADING: u8 = 2;

/// **Where the card's head warns from**, the two edges of the field at the top of it.
const THE_LEFT_EDGE: (f32, f32) = (0.0, 40.0);
const THE_RIGHT_EDGE: (f32, f32) = (384.0, 40.0);

/// A launch whose recipe tells the rules to dodge the card that way, with the card up under a boss and the
/// run at the middle of the low line.
fn under_laevateinn(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(format!(
                "- dodge: {{ with: laevateinn, spell: {LAEVATEINN} }}\n\
                 - play_until: {{ frames: 4000 }}\n"
            ));
            config.log_level = LogLevel::Verbose;
        },
    );
    game.in_a_run_in_the_stage(1);
    game.fires_at_the_player(false);
    game.image().the_player_is_at(192.0, 368.0);
    game.image().a_boss_is_fighting(1000, 370);
    game.image().the_spellcard_is(LAEVATEINN);
    game
}

/// The x the rules last wanted the card's own dodge at, off their own line.
fn wanted_at(game: &Fake) -> Option<f32> {
    wanted(game).map(|(x, _)| x)
}

/// And the whole place, x and y.
fn wanted(game: &Fake) -> Option<(f32, f32)> {
    let lines = game.log().lines();
    lines.iter().rev().find_map(|line| {
        let (_, after) = line.split_once("wanted the place the card's own dodge puts it at ")?;
        let (x, after) = after.split_once(',')?;
        let y = after.split(' ').next()?;
        Some((x.parse().ok()?, y.parse().ok()?))
    })
}

/// **The lasers trailing the head**, `3.0` of half across as a launch read them, running `32` to `420` along.
const THE_TRAILS_WIDTH: f32 = 12.0;
const THE_TRAIL_ALONG: (f32, f32) = (32.0, 420.0);

/// **How many frames the head takes to narrow away once it fades, and how long it has faded for** — `20`, which
/// is what the stubs a launch read the head down to fit: `211..221` along two frames into its fade, `212..220`
/// at eight, `214..218` at fourteen and `216..216` at twenty, about the middle of its `32..400`.
const THE_HEAD_NARROWS_OVER: i32 = 20;
const THE_HEAD_HAS_FADED_FOR: i32 = 2;

/// **A swing to the right, three frames of it, and then its head fading at `334`** — with two of the lasers
/// trailing it still firing behind, as a launch read the frame the head faded on: `3.0` across at `320` and
/// `310`.
fn a_swing_to_the_right_fades(game: &Fake) {
    for (frame, x) in (0..).zip([280.0, 286.0, 292.0]) {
        game.image().a_laser_is(
            0,
            (x, 80.0),
            FRAC_PI_2,
            THE_HEADS_WIDTH,
            ALONG,
            FIRING,
            frame,
            0,
        );
        game.frames(1);
    }
    game.image().a_laser_is(
        0,
        (334.0, 80.0),
        FRAC_PI_2,
        THE_HEADS_WIDTH,
        ALONG,
        FADING,
        THE_HEAD_HAS_FADED_FOR,
        0,
    );
    game.image()
        .the_laser_narrows_over(0, THE_HEAD_NARROWS_OVER);
    for (slot, x) in [(1, 320.0), (2, 310.0)] {
        game.image().a_laser_is(
            slot,
            (x, 90.0),
            FRAC_PI_2,
            THE_TRAILS_WIDTH,
            THE_TRAIL_ALONG,
            FIRING,
            2,
            0,
        );
    }
}

/// **Under a warning from the left edge the rules want the right one**, the edge the swing ends at; and **under
/// one from the right, the left** — see `orb_bot::cards::laevateinn`.
#[test]
fn the_rules_of_th06c_wait_a_swing_of_laevateinn_out_at_the_edge_it_ends_at() {
    in_its_own_process(|| {
        let game = under_laevateinn("th06c-laevateinn-edges");
        game.image().a_laser_is(
            0,
            THE_LEFT_EDGE,
            FRAC_PI_2,
            THE_HEADS_WIDTH,
            ALONG,
            WARNING,
            10,
            60,
        );
        game.frames(2);
        let right = wanted_at(&game);
        assert!(
            right.is_some_and(|x| x > 300.0),
            "under a warning from the left edge the rules wanted {right:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.image().a_laser_is(
            0,
            THE_RIGHT_EDGE,
            FRAC_PI_2,
            THE_HEADS_WIDTH,
            ALONG,
            WARNING,
            10,
            60,
        );
        game.frames(2);
        let left = wanted_at(&game);
        assert!(
            left.is_some_and(|x| x < 80.0),
            "under a warning from the right edge the rules wanted {left:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And once a swing that ended at the right is fading, the left at once**, before the next is drawn: that
/// one starts where this one ended, and the frames between are the width of the field at the full step. A
/// fading laser of this game is a stub about the middle of its length, well above the low line.
#[test]
fn the_rules_of_th06c_cross_to_the_left_once_a_swing_of_laevateinn_to_the_right_is_fading() {
    in_its_own_process(|| {
        let game = under_laevateinn("th06c-laevateinn-gone");
        game.image().the_player_is_at(360.0, 368.0);
        for (frame, x) in (0..).zip([280.0, 286.0, 292.0]) {
            game.image().a_laser_is(
                0,
                (x, 80.0),
                FRAC_PI_2,
                THE_HEADS_WIDTH,
                ALONG,
                FIRING,
                frame,
                0,
            );
            game.frames(1);
        }
        let swinging = wanted_at(&game);
        assert!(
            swinging.is_some_and(|x| x > 300.0),
            "while the swing moved right the rules wanted {swinging:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
        game.image().a_laser_is(
            0,
            (334.0, 80.0),
            FRAC_PI_2,
            THE_HEADS_WIDTH,
            ALONG,
            FADING,
            1,
            0,
        );
        game.frames(2);
        let fading = wanted_at(&game);
        assert!(
            fading.is_some_and(|x| x < 80.0),
            "once the swing to the right was fading the rules wanted {fading:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And the left all the same with the lasers trailing the head still firing**, which outlive it and are
/// no sword of their own.
#[test]
fn the_rules_of_th06c_cross_to_the_left_with_the_lasers_trailing_laevateinn_still_firing() {
    in_its_own_process(|| {
        let game = under_laevateinn("th06c-laevateinn-trails");
        game.image().the_player_is_at(360.0, 368.0);
        a_swing_to_the_right_fades(&game);
        for (slot, x) in [(1, 322.0), (2, 305.0)] {
            game.frames(1);
            game.image().a_laser_is(
                slot,
                (x, 90.0),
                FRAC_PI_2,
                THE_TRAILS_WIDTH,
                THE_TRAIL_ALONG,
                FIRING,
                3,
                0,
            );
        }
        game.frames(1);
        let fading = wanted_at(&game);
        assert!(
            fading.is_some_and(|x| x < 80.0),
            "with the lasers trailing a faded swing still firing the rules wanted {fading:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **The bullets a swing leaves, as a launch read them after the first swing faded**: five columns ahead of a
/// run at the right edge, each of two bullets `48` apart and each `2` to `3` higher than the one to its right —
/// so two lines rising to the left, with a gap `37` high between them.
const TWO_LINES: [(f32, f32); 5] = [
    (339.0, 361.8),
    (330.0, 359.5),
    (320.0, 357.0),
    (308.0, 354.0),
    (294.0, 350.7),
];
const THE_LINES_APART: f32 = 48.0;
const A_BULLET: (f32, f32) = (4.0, 4.0);

/// **Whole lines of them, across the field from the run at the right edge to its left side**, as they stand
/// `frames` after the swing faded: a column every `14` across from `60` to `340` when it did, each `0.18`
/// higher for every unit to the left — the rise of the five above — and five lines `48` apart, from two
/// above the one at `362` to two below it, so that nothing below or above them is a way round; all of it
/// going right at [`THE_LINES_GO`] a frame.
fn lines_across(game: &Fake, frames: u32) {
    for (slot, at) in (16..).zip(the_lines_at(frames)) {
        game.image()
            .a_bullet_of_size_is_at(slot, at.0, at.1, (THE_LINES_GO, 0.0), A_BULLET);
    }
}

/// Where each bullet of [`lines_across`] is, `frames` after the swing faded.
fn the_lines_at(frames: u32) -> Vec<(f32, f32)> {
    const FROM: f32 = 60.0;
    const TO: f32 = 340.0;
    const APART: f32 = 14.0;
    const RISE: f32 = 0.18;
    const THE_LINE_AT_THE_RIGHT: f32 = 362.0;
    const LINES: [f32; 5] = [-2.0, -1.0, 0.0, 1.0, 2.0];
    let columns = ((TO - FROM) / APART) as usize + 1;
    let gone = THE_LINES_GO * frames as f32;
    (0..columns)
        .flat_map(|column| {
            let x = FROM + column as f32 * APART;
            let at_the_line = THE_LINE_AT_THE_RIGHT - RISE * (TO - x);
            LINES.map(|line| (x + gone, at_the_line + line * THE_LINES_APART))
        })
        .collect()
}

/// Whether a run at `at` with [`THE_RUNS_HALF`] overlaps one of those bullets, box on box as the game tests it.
fn inside_a_line(at: (f32, f32), frames: u32) -> bool {
    the_lines_at(frames).iter().any(|bullet| {
        (bullet.0 - at.0).abs() < THE_RUNS_HALF.0 + A_BULLET.0 / 2.0
            && (bullet.1 - at.1).abs() < THE_RUNS_HALF.1 + A_BULLET.1 / 2.0
    })
}

/// **How fast those lines go right** as the swing fades, a launch reading `0.42` to `0.72` a frame across the
/// columns nearest the run on that frame — and faster later.
const THE_LINES_GO: f32 = 0.5;

/// **The run's own box on th06c**, half of it `1.2` either way, as a launch's hit dump read it.
const THE_RUNS_HALF: (f32, f32) = (1.2, 1.2);

/// **And the rules make their way along the gap to the left**, which no one of the nine moves held for the
/// whole of the frames a move is ordinarily judged over does: the gap rises as it goes, so straight left
/// meets the lower line and up and to the left the upper one.
#[test]
fn the_rules_of_th06c_make_their_way_left_between_the_lines_laevateinn_leaves() {
    in_its_own_process(|| {
        let game = under_laevateinn("th06c-laevateinn-along-the-lane");
        // After the laid-out stage's own start, whose build puts the run's box back as a stage starts it.
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        a_swing_to_the_right_fades(&game);
        // And the lasers trailing it gone, as they were six frames after the head faded.
        game.frames(1);
        for slot in [1, 2] {
            game.image().no_laser_is_in(slot);
        }
        // The run at the right edge in the gap, out of the walk-in a stage starts it in and with its own box.
        game.image().the_player_is_playing();
        game.image().the_player_is_at(352.0, 386.0);
        game.image().the_players_box_is(THE_RUNS_HALF);
        let from = game.image().where_the_player_is();
        for frame in 0..40 {
            lines_across(&game, frame);
            game.frames(1);
            let at = game.image().where_the_player_is();
            assert!(
                !inside_a_line(at, frame + 1),
                "the rules ran into a line at {at:?}, {} frame(s) in:\n  {}",
                frame + 1,
                game.log().lines().join("\n  "),
            );
        }
        let to = game.image().where_the_player_is();
        assert!(
            to.0 < from.0 - 80.0,
            "the rules went from {from:?} to {to:?} in forty frames between the lines:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Crossing to the left, the rules want the middle of the gap between two of those lines**, and not the low
/// line six under the upper one.
#[test]
fn the_rules_of_th06c_cross_between_the_lines_of_bullets_laevateinn_leaves() {
    in_its_own_process(|| {
        let game = under_laevateinn("th06c-laevateinn-lane");
        // After the laid-out stage's own start, whose build puts the run back where a stage starts it — the
        // lines are ahead of a run at the right edge and of none at the start.
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().the_player_is_playing();
        game.image().the_player_is_at(352.0, 368.0);
        a_swing_to_the_right_fades(&game);
        // Slots past the sixteen the laid-out stage's own fire rewrites every frame.
        for (slot, (x, y)) in (16..).step_by(2).zip(TWO_LINES) {
            game.image()
                .a_bullet_of_size_is_at(slot, x, y, (1.0, 0.0), A_BULLET);
            game.image().a_bullet_of_size_is_at(
                slot + 1,
                x,
                y + THE_LINES_APART,
                (1.0, 0.0),
                A_BULLET,
            );
        }
        game.frames(2);
        let upper = TWO_LINES[0].1;
        let lower = TWO_LINES[TWO_LINES.len() - 1].1 + THE_LINES_APART;
        let middle = (upper + lower) / 2.0;
        let place = wanted(&game);
        assert!(
            place.is_some_and(|(x, y)| x < 80.0 && (y - middle).abs() < 6.0),
            "between lines at {upper} and {lower} the rules wanted {place:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// `禁弾「過去を刻む時計」`, by the index the card's own start shifts into the records by — `spell=61` in the
/// state lines of a launch that played it.
const THE_CLOCK: u32 = 61;

/// **A clock's four hands, as a launch read them**: a cross, `6.0` of half across and `0..216` along out of
/// one centre.
const A_HANDS_WIDTH: f32 = 24.0;
const A_HANDS_ALONG: (f32, f32) = (0.0, 216.0);

/// **The two heights the two clocks' centres travel at**, the low one's and the high one's.
const A_LOW_CLOCK: f32 = 288.0;
const A_HIGH_CLOCK: f32 = 160.0;

/// A launch whose recipe tells the rules to dodge the card that way, with the card up under a boss.
fn under_the_clock(name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(format!(
                "- dodge: {{ with: clock, spell: {THE_CLOCK} }}\n\
                 - play_until: {{ frames: 4000 }}\n"
            ));
            config.log_level = LogLevel::Verbose;
        },
    );
    game.in_a_run_in_the_stage(1);
    game.fires_at_the_player(false);
    game.image().a_boss_is_fighting(1000, 370);
    game.image().the_spellcard_is(THE_CLOCK);
    game
}

/// **The two clocks up at once**, as a launch read them: the low one's four hands in the first four slots and
/// the high one's in the next four, each cross a quarter turn apart.
fn the_clocks_at(game: &Fake, low: f32, high: f32) {
    for (first, centre, turned) in [
        (0, (low, A_LOW_CLOCK), 0.2),
        (4, (high, A_HIGH_CLOCK), -0.2),
    ] {
        for hand in 0..4 {
            game.image().a_laser_is(
                first + hand,
                centre,
                turned + FRAC_PI_2 * hand as f32,
                A_HANDS_WIDTH,
                A_HANDS_ALONG,
                FIRING,
                30,
                0,
            );
        }
    }
}

/// **The two clocks travelling a frame at a time**, the low one by `step` and the high one the other way, for
/// `frames` frames from where they are.
fn the_clocks_travel(game: &Fake, low: f32, high: f32, step: f32, frames: u32) {
    for frame in 0..frames {
        let gone = step * frame as f32;
        the_clocks_at(game, low + gone, high - gone);
        game.frames(1);
    }
}

/// How far a clock's centre travels in a frame, as a launch read it.
const A_CLOCKS_STEP: f32 = 1.17;

/// **Under a low clock travelling right from far off to the left, the rules want the right edge at the
/// bottom**, which is further across than the hands reach there.
#[test]
fn the_rules_of_th06c_wait_a_low_clock_out_at_the_right_edge_while_it_is_far_off() {
    in_its_own_process(|| {
        let game = under_the_clock("th06c-clock-far-off");
        game.image().the_player_is_at(300.0, 400.0);
        the_clocks_travel(&game, 100.0, 284.0, A_CLOCKS_STEP, 3);
        let place = wanted(&game);
        assert!(
            place.is_some_and(|(x, y)| x > 330.0 && y > 410.0),
            "under a low clock at 100 the rules wanted {place:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And once its centre has come near, past it to its left**, behind the way it travels.
#[test]
fn the_rules_of_th06c_cross_behind_a_low_clock_once_it_is_near() {
    in_its_own_process(|| {
        let game = under_the_clock("th06c-clock-near");
        game.image().the_player_is_at(342.0, 424.0);
        the_clocks_travel(&game, 237.0, 144.0, A_CLOCKS_STEP, 3);
        let place = wanted(&game);
        assert!(
            place.is_some_and(|(x, y)| x < 240.0 - 150.0 && y > 410.0),
            "under a low clock at 240 the rules at the right edge wanted {place:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And under a high clock, the bottom wherever they are**, which the hands do not reach.
#[test]
fn the_rules_of_th06c_wait_under_a_high_clock_at_the_bottom() {
    in_its_own_process(|| {
        let game = under_the_clock("th06c-clock-high");
        game.image().the_player_is_at(192.0, 300.0);
        for hand in 0..4 {
            game.image().a_laser_is(
                hand,
                (192.0, A_HIGH_CLOCK),
                0.2 + FRAC_PI_2 * hand as f32,
                A_HANDS_WIDTH,
                A_HANDS_ALONG,
                FIRING,
                30,
                0,
            );
        }
        game.frames(2);
        let place = wanted(&game);
        assert!(
            place.is_some_and(|(x, y)| (x - 192.0).abs() < 10.0 && y > 410.0),
            "under a high clock the rules wanted {place:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And under a low clock travelling left, the left edge while it is far off**, the second pair of clocks
/// coming the other way.
#[test]
fn the_rules_of_th06c_wait_a_low_clock_travelling_left_out_at_the_left_edge() {
    in_its_own_process(|| {
        let game = under_the_clock("th06c-clock-travelling-left");
        game.image().the_player_is_at(60.0, 424.0);
        the_clocks_travel(&game, 290.0, 100.0, -A_CLOCKS_STEP, 3);
        let place = wanted(&game);
        assert!(
            place.is_some_and(|(x, y)| x < 60.0 && y > 410.0),
            "under a low clock travelling left from 290 the rules wanted {place:?}:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **And with no clock up, out of reach of where the next low one comes up**: the right edge before a pair whose
/// low clock comes up on the left, and the left edge before the pair after it, whose low clock comes up on the
/// right — the first coming up at card frame `160` and again at `1250`, the second at `510`.
#[test]
fn the_rules_of_th06c_wait_for_the_next_pair_of_clocks_out_of_its_low_ones_reach() {
    in_its_own_process(|| {
        const BEFORE_THE_FIRST_PAIR: i32 = 1200;
        const BEFORE_THE_SECOND_PAIR: i32 = 460;
        for (name, into, far_side) in [
            (
                "th06c-clock-before-the-first-pair",
                BEFORE_THE_FIRST_PAIR,
                330.0_f32,
            ),
            (
                "th06c-clock-before-the-second-pair",
                BEFORE_THE_SECOND_PAIR,
                54.0_f32,
            ),
        ] {
            let game = under_the_clock(name);
            game.image().a_boss_is_fighting(1000, into);
            game.image().the_player_is_at(192.0, 400.0);
            game.frames(2);
            let place = wanted(&game);
            assert!(
                place.is_some_and(|(x, y)| (x - 192.0).signum() == (far_side - 192.0).signum()
                    && (x - 192.0).abs() >= (far_side - 192.0).abs()
                    && y > 410.0),
                "{into} frames into the card with no clock up the rules wanted {place:?}",
            );
        }
    });
}

/// `秘弾「そして誰もいなくなるか？」`, by the index the card's own start shifts into the records by — `spell=62` in
/// the state lines of a launch that played it.
const AND_THEN_THERE_WERE_NONE: u32 = 62;

/// **The corners the card's chasers are led into, `48` in from each edge of the box the player is held to** —
/// the box the stage callback writes at `0x1403ee7f4`, `368` by `416` from `8,16` (0x140024345 through
/// 0x140024363).
const THE_WHOLE_FIELDS_CORNERS: (f32, f32, f32, f32) = (56.0, 64.0, 328.0, 384.0);

/// **The card's frames its second half is played at**, past the four chasers coming in out of the field's
/// corners at `1020` — see `cards::and_then_there_were_none` — and the first half's, before any chaser is up.
const IN_THE_SECOND_HALF: i32 = 2000;
const IN_THE_FIRST_HALF: i32 = 100;

/// **A chaser of the card's**, as a launch read them: a body `37` across that nothing collides with, in the
/// slot after the boss's.
const A_CHASERS_SIZE: f32 = 56.0;
const A_CHASERS_SLOT: usize = 1;

/// A launch whose recipe tells the rules to dodge the card that way, with the card up under a boss and `into`
/// frames into it.
fn under_and_then_there_were_none(name: &str, into: i32) -> Box<Fake> {
    let game = Fake::attach_configured(
        Display::agreed(120),
        name,
        Work::wandering(WORK_US),
        |config| {
            config.autoplay = Some(format!(
                "- dodge: {{ with: and_then_there_were_none, spell: {AND_THEN_THERE_WERE_NONE} }}\n\
                 - play_until: {{ frames: 4000 }}\n"
            ));
            config.log_level = LogLevel::Verbose;
        },
    );
    game.in_a_run_in_the_stage(1);
    game.fires_at_the_player(false);
    game.image().a_boss_is_fighting(1000, into);
    game.image().the_spellcard_is(AND_THEN_THERE_WERE_NONE);
    game
}

/// **The trace carries where the player was on every frame the rules read**, which is what two runs of one
/// stretch are held against each other by.
#[test]
fn the_trace_of_a_run_of_th06c_says_where_its_player_was() {
    in_its_own_process(|| {
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-player-traced",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
            },
        );
        game.in_a_run_in_the_stage(1);
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().the_player_is_at(100.0, 300.0);
        game.frames(3);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let last = records.iter().rev().find_map(|record| record.at);
        assert!(
            last.is_some_and(|(x, y)| (x - 100.0).abs() < 16.0 && (y - 300.0).abs() < 16.0),
            "the trace's last player was at {last:?} where the run's was put at 100,300",
        );
    });
}

/// **The field is traced on the frames no card is up on where a launch named none, and on no card's**, which
/// is how a stage's stretch between its cards is measured: every bullet where it is, and the player.
#[test]
fn the_field_of_th06c_is_traced_where_no_card_is_up_for_a_launch_that_named_none() {
    in_its_own_process(|| {
        const A_CARD: u32 = 7;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-field-traced-where-no-card-is-up",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
                config.trace_the_field_in = Some(orb_core::trace::NO_CARD);
            },
        );
        game.in_a_run_in_the_stage(1);
        game.fires_at_the_player(false);
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().the_player_is_at(100.0, 300.0);
        game.image()
            .a_bullet_of_size_is_at(0, 150.0, 200.0, (0.0, 0.0), (8.0, 8.0));
        game.image().bullets_alive(1);
        game.frames(3);
        game.image().a_boss_is_fighting(1000, 10);
        game.image().the_spellcard_is(A_CARD);
        game.frames(3);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let traced: Vec<_> = records
            .iter()
            .filter_map(|record| record.the_field.clone())
            .collect();
        assert!(
            traced.iter().any(|field| field
                .bullets
                .iter()
                .any(|bullet| bullet.at == (150.0, 200.0))),
            "no frame with no card up had its field traced with the bullet on it: {traced:?}",
        );
        assert!(
            traced
                .iter()
                .all(|field| field.card == orb_core::trace::NO_CARD),
            "a frame of a card had its field traced: {:?}",
            traced.iter().map(|field| field.card).collect::<Vec<_>>(),
        );
    });
}

/// **And on every frame, a card's and no card's alike, where a launch named all** — which is how a replay is read
/// in one playing, its every card and the stretches between them.
#[test]
fn the_field_of_th06c_is_traced_on_a_card_and_where_no_card_is_up_for_a_launch_that_named_all() {
    in_its_own_process(|| {
        const A_CARD: u32 = 7;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-field-traced-on-every-frame",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
                config.trace_the_field_in = Some(orb_core::trace::EVERY_FRAME);
            },
        );
        game.in_a_run_in_the_stage(1);
        game.fires_at_the_player(false);
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().the_player_is_at(100.0, 300.0);
        game.image()
            .a_bullet_of_size_is_at(0, 150.0, 200.0, (0.0, 0.0), (8.0, 8.0));
        game.image().bullets_alive(1);
        game.frames(3);
        game.image().a_boss_is_fighting(1000, 10);
        game.image().the_spellcard_is(A_CARD);
        game.frames(3);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let cards: Vec<_> = records
            .iter()
            .filter_map(|record| record.the_field.as_ref().map(|field| field.card))
            .collect();
        assert!(
            cards.contains(&orb_core::trace::NO_CARD) && cards.contains(&A_CARD),
            "the frames with no card up and the card's were not both traced: {cards:?}",
        );
    });
}

/// **A place a recipe tells the rules goes as deep as the floor it names**, below the low line the rules wait
/// on of their own — which is where th06c's Extra after its midboss is got through, its fans of three aimed at
/// the run furthest apart there.
#[test]
fn the_rules_of_th06c_stand_on_the_floor_where_a_recipe_tells_them_to() {
    in_its_own_process(|| {
        const THE_FLOOR: (f32, f32) = (192.0, 432.0);
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-told-to-stand-on-the-floor",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some(
                    "- stand_at: { x: 192, y: 432 }\n\
                     - play_until: { frames: 4000 }\n"
                        .to_owned(),
                );
                config.log_level = LogLevel::Verbose;
            },
        );
        game.in_a_run_in_the_stage(1);
        game.fires_at_the_player(false);
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().the_player_is_at(100.0, 368.0);
        game.frames(2);
        let wanted = game.log().lines().iter().rev().find_map(|line| {
            let (_, after) = line.split_once("wanted the place it was told to stand at ")?;
            let (x, after) = after.split_once(',')?;
            let y = after.split(' ').next()?;
            Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))
        });
        assert_eq!(
            wanted,
            Some(THE_FLOOR),
            "told to stand on the floor, the rules wanted {wanted:?}",
        );
    });
}

/// **The shot is let go on the same frames of every stage**, one in every sixty of the stage's own clock
/// however long the rules have been playing: stage 1 lasting `100` frames, a let-go counted on from it would
/// fall `40` further on in stage 2. Which is what lets a run picked up at a chapter play it the way the run it
/// was written by did — see `steer::the_shot_is_let_go`.
#[test]
fn the_shot_of_th06c_is_let_go_on_the_same_frames_of_every_stage() {
    in_its_own_process(|| {
        const THE_SHOTS_BIT: u16 = 0x0001;
        const A_LET_GO_IN_EVERY: u32 = 60;
        let game = Fake::attach_configured(
            Display::agreed(120),
            "th06c-the-shot-let-go-by-the-stage",
            Work::wandering(WORK_US),
            |config| {
                config.autoplay = Some("[]".to_owned());
                config.trace = Some(std::path::PathBuf::from(orb_real::TRACE_FILE));
            },
        );
        game.stages_last(100);
        game.in_a_run_in_the_stage(1);
        game.frames(400);
        orb_core::trace::close();
        let bytes = game
            .sim()
            .files()
            .get(game.dir().join(orb_real::TRACE_FILE))
            .expect("the launch wrote a trace");
        let records = orb_core::trace::read(&bytes).expect("the trace reads back");
        let let_go: Vec<(i32, u32)> = records
            .iter()
            .filter(|record| record.word & THE_SHOTS_BIT == 0)
            .filter_map(|record| record.state.as_ref())
            .filter(|state| state.in_run && state.stage_frames > 1)
            .map(|state| (state.stage, state.stage_frames))
            .collect();
        let stages: std::collections::BTreeSet<i32> =
            let_go.iter().map(|(stage, _)| *stage).collect();
        assert!(
            stages.len() >= 2,
            "the shot was let go in {stages:?} and not in two stages: {let_go:?}",
        );
        let when: std::collections::BTreeSet<u32> = let_go
            .iter()
            .map(|(_, frames)| frames % A_LET_GO_IN_EVERY)
            .collect();
        assert_eq!(
            when.len(),
            1,
            "the shot was let go at different frames of each sixty of the stage: {let_go:?}",
        );
    });
}

/// The rules put at `at`, and the last place they wanted two frames later with where they wanted it from — the
/// laid-out run having moved between the two.
fn wanted_from(game: &Fake, at: (f32, f32)) -> Option<((f32, f32), (f32, f32))> {
    game.image().the_player_is_at(at.0, at.1);
    game.frames(2);
    game.log().lines().iter().rev().find_map(|line| {
        let (_, after) = line.split_once("wanted the place the card's own dodge puts it at ")?;
        let (place, after) = after.split_once(" from ")?;
        let from = after.split(' ').next()?;
        let pair = |text: &str| {
            let (x, y) = text.split_once(',')?;
            Some((x.parse().ok()?, y.parse().ok()?))
        };
        Some((pair(place)?, pair(from)?))
    })
}

/// **For the card's first half the rules lead its chaser into the corners**: to the bottom left first — the
/// strategy wiki's `下→左下` — and waiting there while the chaser is still across the field.
#[test]
fn the_rules_of_th06c_wait_in_a_corner_for_the_chaser_under_and_then_there_were_none() {
    in_its_own_process(|| {
        let (left, _, _, bottom) = THE_WHOLE_FIELDS_CORNERS;
        let game = under_and_then_there_were_none(
            "th06c-and-then-there-were-none-waiting",
            IN_THE_FIRST_HALF,
        );
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().an_enemy_that_passes_through_is_at(
            A_CHASERS_SLOT,
            192.0,
            224.0,
            A_CHASERS_SIZE,
        );
        for at in [(192.0, 424.0), (left, bottom)] {
            let wanted = wanted_from(&game, at);
            assert!(
                wanted.is_some_and(
                    |(place, _)| (place.0 - left).abs() < 1.0 && (place.1 - bottom).abs() < 1.0
                ),
                "put at {at:?} with the chaser across the field the rules wanted {wanted:?} and not the bottom \
                 left {:?}:\n  {}",
                (left, bottom),
                game.log().lines().join("\n  "),
            );
        }
    });
}

/// **And on to the next corner clockwise once the chaser has come to them there**: from the bottom left, the
/// top left — so that the chaser has come into the corner, and what it drops is dropped there.
#[test]
fn the_rules_of_th06c_lead_the_chaser_on_to_the_next_corner_under_and_then_there_were_none() {
    in_its_own_process(|| {
        let (left, top, _, bottom) = THE_WHOLE_FIELDS_CORNERS;
        let game = under_and_then_there_were_none(
            "th06c-and-then-there-were-none-leading",
            IN_THE_FIRST_HALF,
        );
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().an_enemy_that_passes_through_is_at(
            A_CHASERS_SLOT,
            left + 40.0,
            bottom - 30.0,
            A_CHASERS_SIZE,
        );
        let wanted = wanted_from(&game, (left, bottom));
        assert!(
            wanted.is_some_and(
                |(place, _)| (place.0 - left).abs() < 1.0 && (place.1 - top).abs() < 1.0
            ),
            "at the bottom left with the chaser come up to it the rules wanted {wanted:?} and not the top left \
             {:?}:\n  {}",
            (left, top),
            game.log().lines().join("\n  "),
        );
    });
}

/// **And on their way to a corner they take the step that way where it is clear for the next six frames,
/// though every one of the nine is hit within forty** — which is where the card's field of slow fire had the
/// rules standing still, each of its hits being of a run that stood. Here the way down and to the left, to
/// the bottom left, meets a bullet `44` off, ten frames on; the other seven meet one `80` off, and standing
/// still one coming down at `0.5` a frame, thirty frames on. Judged over forty frames standing still is hit
/// latest; over six, nothing is hit and the step towards the corner is the nearest.
#[test]
fn the_rules_of_th06c_step_through_slow_fire_on_their_way_under_and_then_there_were_none() {
    in_its_own_process(|| {
        const DOWN_AND_LEFT: u32 = 0x0020 | 0x0040;
        const THE_MIDDLE: (f32, f32) = (192.0, 224.0);
        const THE_SLOT_AFTER_THE_STAGE_S_OWN: usize = 16;
        let game = under_and_then_there_were_none(
            "th06c-and-then-there-were-none-slow-fire",
            IN_THE_FIRST_HALF,
        );
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().the_player_is_playing();
        game.image().the_player_is_at(THE_MIDDLE.0, THE_MIDDLE.1);
        game.image().the_players_box_is(THE_RUNS_HALF);
        let diagonal = std::f32::consts::FRAC_1_SQRT_2;
        let near = 44.0 * diagonal;
        let far = 80.0;
        let far_diagonal = far * diagonal;
        let (x, y) = THE_MIDDLE;
        for (slot, (at, step)) in (THE_SLOT_AFTER_THE_STAGE_S_OWN..).zip([
            ((x - near, y + near), (0.0, 0.0)),
            ((x - far, y), (0.0, 0.0)),
            ((x + far, y), (0.0, 0.0)),
            ((x, y - far), (0.0, 0.0)),
            ((x, y + far), (0.0, 0.0)),
            ((x - far_diagonal, y - far_diagonal), (0.0, 0.0)),
            ((x + far_diagonal, y - far_diagonal), (0.0, 0.0)),
            ((x + far_diagonal, y + far_diagonal), (0.0, 0.0)),
            ((x, y - 20.0), (0.0, 0.5)),
        ]) {
            game.image()
                .a_bullet_of_size_is_at(slot, at.0, at.1, step, A_BULLET);
        }
        game.frames(2);
        let word = game.image().the_word_the_run_acts_on();
        assert_eq!(
            word & DOWN_AND_LEFT,
            DOWN_AND_LEFT,
            "the rules pressed {word:#06x} and not the step down and to the left towards the bottom left:\n  {}",
            game.log().lines().join("\n  "),
        );
    });
}

/// **Once the chasers are all gone the rules wait the card out at the bottom middle**, a quarter of the box the
/// player is held to up from its floor — that box being `416` high from `16`, `328` — from wherever they were.
#[test]
fn the_rules_of_th06c_wait_the_card_out_low_in_the_middle_under_and_then_there_were_none() {
    in_its_own_process(|| {
        const THE_BOTTOM_MIDDLE_A_QUARTER_UP: (f32, f32) = (192.0, 328.0);
        let (_, top, right, _) = THE_WHOLE_FIELDS_CORNERS;
        let game = under_and_then_there_were_none(
            "th06c-and-then-there-were-none-waited-out",
            IN_THE_SECOND_HALF,
        );
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        for at in [(right, top), (50.0, 400.0)] {
            let wanted = wanted_from(&game, at);
            assert!(
                wanted.is_some_and(
                    |(place, _)| (place.0 - THE_BOTTOM_MIDDLE_A_QUARTER_UP.0).abs() < 1.0
                        && (place.1 - THE_BOTTOM_MIDDLE_A_QUARTER_UP.1).abs() < 1.0
                ),
                "put at {at:?} with no chaser left the rules wanted {wanted:?} and not the bottom middle \
                 {THE_BOTTOM_MIDDLE_A_QUARTER_UP:?}:\n  {}",
                game.log().lines().join("\n  "),
            );
        }
    });
}

/// **And the lead goes on through the second half while a chaser is up**, the four more coming in out of the
/// field's corners then: the bottom left waited at, as it is from the card's start.
#[test]
fn the_rules_of_th06c_lead_the_chasers_on_through_the_second_half_under_and_then_there_were_none() {
    in_its_own_process(|| {
        let (left, _, _, bottom) = THE_WHOLE_FIELDS_CORNERS;
        let game = under_and_then_there_were_none(
            "th06c-and-then-there-were-none-leading-on",
            IN_THE_SECOND_HALF,
        );
        let log = game.log();
        game.frames_until("the stage started", 600, || log.said("(stage start)"));
        game.image().an_enemy_that_passes_through_is_at(
            A_CHASERS_SLOT,
            192.0,
            224.0,
            A_CHASERS_SIZE,
        );
        let wanted = wanted_from(&game, (192.0, 424.0));
        assert!(
            wanted.is_some_and(
                |(place, _)| (place.0 - left).abs() < 1.0 && (place.1 - bottom).abs() < 1.0
            ),
            "with a chaser up in the second half the rules wanted {wanted:?} and not the bottom left {:?}:\n  {}",
            (left, bottom),
            game.log().lines().join("\n  "),
        );
    });
}
