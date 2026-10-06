//! **Ways of dodging one card of one game, each named for its card** — a card whose pattern is the same every
//! launch, so that what gets through it is its own schedule and nothing any other card shares — **and one stretch
//! of a stage or one boss's fire between two cards, the same way**. The ways that are a mechanism a card may want,
//! whichever card it is, are `steer`'s.
//!
//! # From the strategy wiki's words to a way
//!
//! A way here is the strategy wiki's for its card (wikiwiki.jp/thk), and what the wiki says a player does is turned
//! into code by these, each learned on th11's cards of 2026-10-07:
//!
//! - **A place is told and the rules dodge round it.** A way answers where the run is to be this frame; `steer`'s
//!   rules still dodge every bullet, and a move landing too far from the place and further than the run is now is
//!   judged hit (`ROUND_THE_SPLEEN_EATERS_CIRCLE_HELD_WITHIN` in `steer`). With no hold the rules' own dodging carries
//!   the run off to a wall or the floor, the place being only a wish.
//! - **What the wiki forbids is a move judged hit.** `左への移動は無意味で危険` for card 173 is a move to the left from
//!   left of its place; `上下左右の移動に限定` is a diagonal move. Each is judged in `steer` beside the hold, so that
//!   the rules choose among what is left.
//! - **The wiki's place is read off the card's own things, measured.** `右上が長時間安全` is the corner of the round
//!   the boss is read to go along that she comes to last; `中間位置` is half way between the floor and where she
//!   stands; `薔薇が通り過ぎてすぐに抜ける` is the distance a ring will be at as its rose has gone past the run's
//!   angle. Each number comes off the field traced and the pictures, and the reading is beside its constant.
//! - **A way is timed on what is on the field, or on the stage's frame, and not the card's timer**: a card's stages are
//!   read by their frames from the card's start as the stage's frame counts them (`STAGE_FRAMES` in `steer`),
//!   forgotten after a gap of calls — which is what the ways were written against, and is the same count as th11's
//!   card timer, `+0x88c`, going up by 1 from the card's start (1 to 1,350 through card 169 and 1 to 1,500 through
//!   card 127, 2026-10-08), so there is nothing to move them for.
//! - **What comes is weighed along the way it goes**: the boss along her round, turning at its corners, and a rose
//!   round its ring the way that ring's rose goes — one ring's clockwise and the next the other way — and a place is
//!   only one the run gets to before what it waits for does.
//! - **A place is kept until another is better by a margin**, so that two much the same do not send the run back and
//!   forth across the field through the bullets between.
//!
//! And these, learned on th11's Extra of 2026-10-08:
//!
//! - **A way is its own attack's, and never one written for another.** A way that got a run through some other
//!   attack, tried on this one and taking no hit on the chapters kept, has only been lucky with how those chapters
//!   came: against the left wall all through the stage after the Extra's midboss took none on four kept chapters
//!   and was hit on the real case, where the streams went the other way, and the wiki has the wall for its first
//!   set alone (`from_the_left_wall_to_the_top_right`).
//! - **The wiki's words are found on the field before they are written down.** The sets, rows and gaps it names
//!   are read off the bodies and bullets traced — `--trace-the-field-in=<card>`, or `all` for a stretch with no card
//!   up — and each step of the way is timed on when they come, so that `1セット目` is the row along 160 that comes
//!   in from the left and `縦の隙間` is the 50 to 80 between rows by the floor (`between_the_long_rows`).
//! - **The wiki's speed is the step.** `大きく速く動く` and `低速移動だけでは危険` are the whole step and never the
//!   focused one (`the_whole_step_through` in `steer`): card 169 swung at the focused step laid its rice as a fence
//!   across the way ahead and stalled against it.
//! - **A way is checked on the trace's own pictures, and not only on its count of hits** (`orb-real picture`): no hit
//!   says the run got through, and a run that got through without doing what the wiki says got through by chance,
//!   which the next chapter that comes another way takes back. A hit is read on its pictures and its field first,
//!   before anything is changed.
//! - **A way is judged on every chapter kept for its attack and then on a whole run**, the same attack coming
//!   differently with where the run comes into it and when it began; one chapter's hit read again with a way made
//!   for it is not the way judged.

use std::f32::consts::FRAC_PI_2;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use orb_core::detail;
use orb_core::game::{Frame, Hazard, Kind};

/// A bullet's place, its step a frame and the half of its box, each on both axes.
pub(crate) type Moving = ((f32, f32), (f32, f32), (f32, f32));
/// A bullet's age, its place and its step a frame.
type Aged = (i32, (f32, f32), (f32, f32));

/// **Where th06c's `禁忌「レーヴァテイン」` is waited out, by the sword it is swinging** — at the far end of a
/// swing across the field, and nowhere of this card's own while it swings round the boss, where the rules'
/// own errand gets through.
///
/// **Its swings across come from either edge by turns**, read off every beam of eighteen attempts at the
/// card (`--trace-the-field-in=55`, 2026-09-26). The sword is a head of `6.0` across with trailing lasers of
/// `3.0` behind it. It first swings round the boss at `192,80` from `-2.59` rad to `1.73`, frames `130` to
/// `250` of the card, which the rules' errand survived on every attempt. Then it points straight down from
/// the left edge, a warning at `0,40` from frame `370`, and sweeps right at about six a frame from `430`,
/// fading at `341` by `490`; then from the right edge, a warning at `384,40` from `520`, and sweeps left
/// from `580`. Straight down it reaches the whole height of the field, so the one way through a swing
/// across is to be at the edge it ends at before it gets there. **The rules instead waited at the edge the
/// first swing ended at**, which is the edge the second starts from: fifteen of the eighteen attempts died
/// at frames `586` to `597`, caught at `x = 258` to `332` running from a sword that moves seven a frame to
/// their four. The other three died at `413` to `458`, overtaken by the first.
///
/// **So the edge wanted is the one the swing moves towards**: the far edge from a warning's own, the edge
/// the head is moving to while it sweeps, and — once a swing that ended at the right is gone — the left,
/// the next one starting where it ended. Eighty frames lie between that one going and the next sweeping,
/// which is `320` at the full step — the width of the field and not much more, so this is taken the moment
/// the swing goes and not when the next is drawn.
///
/// **And the crossing is made at the height [`the_lane`] finds**, the swing having left its bullets behind it
/// in lines the run has to go between.
pub(crate) fn laevateinn(frame: &Frame) -> Option<(f32, f32)> {
    let field = frame.field;
    let middle = field.0 + field.2 / 2.0;
    // **Not a fading one**, which is a swing over: what a fading laser of this game keeps is a stub about
    // the middle of its length — see `game::th06c::sight` — and this one's is `211..221` along, `300` above
    // a run on the low line. The head fades at `334` on its way to `376`, sixteen frames before it goes, so
    // taking the fade as the end is sixteen frames more of the crossing.
    let head = frame.beams.iter().find(|beam| {
        !beam.fading
            && beam.half_across > A_TRAILS_HALF_ACROSS
            && (beam.angle - FRAC_PI_2).abs() < STRAIGHT_DOWN
    });
    let mut swing = LAEVATEINN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let edge = match head {
        Some(beam) if beam.fires_in > 0 => Some(match beam.origin.0 < middle {
            true => Edge::Right,
            false => Edge::Left,
        }),
        Some(beam) => match swing.head_was {
            Some(was) if beam.origin.0 > was + A_SWEEPS_STEP => Some(Edge::Right),
            Some(was) if beam.origin.0 < was - A_SWEEPS_STEP => Some(Edge::Left),
            _ => swing.towards,
        },
        // **Gone**: the next swing starts where this one ended, which is the far side for a swing that
        // ended at the right — and nothing of this card's own after one that ended at the left, what
        // follows that being the swing round the boss.
        None => match (swing.head_was, swing.towards) {
            (Some(_), Some(Edge::Right)) => Some(Edge::Left),
            (Some(_), Some(Edge::Left)) => None,
            (None, towards) => towards,
            (Some(_), None) => None,
        },
    };
    swing.head_was = head.map(|beam| beam.origin.0);
    swing.towards = edge;
    let x = match edge? {
        Edge::Left => field.0 + IN_FROM_THE_EDGE,
        Edge::Right => field.0 + field.2 - IN_FROM_THE_EDGE,
    };
    Some((x, the_lane(frame, x)))
}

/// **The height `「レーヴァテイン」` is crossed at**: the middle of the gap between two lines of the bullets
/// its swing left, the gap the run is in or the nearest to it — measured off the bullets up to
/// [`LANE_AHEAD`] ahead of the run on its way to `x`, or towards the middle once it is there.
///
/// **The bullets a swing leaves are lines across the field**, read off one attempt's bullets on the frames
/// after the first swing faded: the columns it dropped, `12` to `30` apart, each of bullets `48` above one
/// another, and each column's `5` or so higher than the one to the right of it — `(66, 312)`, `(96, 318)`
/// on to `(317, 362)` — so that the bullets make lines rising to the left, `48` apart, going right at one
/// a frame and more. The gap between two is `40` high, and nowhere is a height clear the whole way across.
/// **The rules at the low line were `6` under one of those lines**, and with `3` of the nine moves clear —
/// none of them left — stood at `352,368` from the fade to the next swing, which reached them.
///
/// **The low line where nothing is ahead**, which is every frame before the swing has left anything.
fn the_lane(frame: &Frame, x: f32) -> f32 {
    let field = frame.field;
    let floor = field.1 + field.3;
    let (px, py) = frame.at;
    let towards = match (x - px).abs() < ARRIVED {
        true => field.0 + field.2 / 2.0,
        false => x,
    };
    let ahead = match towards < px {
        true => px - LANE_AHEAD..=px,
        false => px..=px + LANE_AHEAD,
    };
    let lines: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter(|hazard| ahead.contains(&hazard.at.0))
        .map(|hazard| (hazard.at.1, hazard.half.1))
        .collect();
    if lines.is_empty() {
        return floor - ABOVE_THE_FLOOR;
    }
    let clearance = |y: f32| {
        lines
            .iter()
            .map(|(at, half)| (y - at).abs() - half)
            .fold(f32::INFINITY, f32::min)
    };
    // **Scored less half a unit for every unit off the run's own height**, so that the middle of the gap the
    // run is in beats every height of it — the room growing a unit for every unit nearer — and a gap one line
    // over, `48` away, costs more than the whole of a gap's room.
    (0..=LANE_BAND as i32)
        .map(|below| floor - below as f32)
        .max_by(|one, other| {
            let score = |y: f32| clearance(y) - (y - py).abs() / 2.0;
            score(*one).total_cmp(&score(*other))
        })
        .unwrap_or(floor - ABOVE_THE_FLOOR)
}

/// **How far ahead of the run [`the_lane`] reads the lines**: a line rises `11` over `64` across, so the gap
/// read is the one the next sixteen frames of the step go through and not one bent out of it.
const LANE_AHEAD: f32 = 64.0;

/// **How high above the floor a crossing may go**: two gaps of `48`, and the low line at `64` inside the
/// band.
const LANE_BAND: f32 = 128.0;

/// **How near its edge the run is where [`the_lane`] reads towards the middle instead**: a step of the rules'.
const ARRIVED: f32 = 4.0;

/// Which edge of the field `「レーヴァテイン」` is waited out at — see [`laevateinn`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Right,
}

/// What [`laevateinn`] keeps between frames: where the sword's head was, and the edge it was waited out at.
struct Swing {
    head_was: Option<f32>,
    towards: Option<Edge>,
}

static LAEVATEINN: Mutex<Swing> = Mutex::new(Swing {
    head_was: None,
    towards: None,
});

/// **How near straight down a laser has to point to be a swing across**, in radians: the swing round the
/// boss passes through straight down too, but moving `0.039` a frame, and a swing across holds it exactly.
const STRAIGHT_DOWN: f32 = 0.01;

/// **How wide the lasers trailing the sword's head are**, by the half across the game gives them: the head's is
/// `6.0` and theirs `3.0`, so a laser wider than this is the head. **Not the widest of those straight down**,
/// which once the head fades is one of these: they are drawn afresh along the swing, a different one widest
/// from frame to frame, and one attempt read their jumps as the sword moving on and waited at the right
/// until the next swing reached it.
const A_TRAILS_HALF_ACROSS: f32 = 3.0;

/// **How far the sword's head has to move in a frame to be sweeping**, the head moving six a frame while it
/// sweeps and not at all while it warns.
const A_SWEEPS_STEP: f32 = 0.5;

/// **How far in from the edge a swing across is waited out**, the swing fading `50` short of the far edge —
/// `334` of a field `384` wide — and its head reaching `7` either side of its line: `34` is the far side of
/// that, and every unit nearer the middle is a unit less of the crossing to the other edge.
///
/// **Which the crossing had none of to spare at `16`.** One attempt waited at `360` and was told the left at
/// the next warning, `80` frames before that swing began: `336` across at the full step's four is `84`
/// frames, and it stood where it was with `4` to `6` of the nine clear — none of them the way it wanted —
/// until the swing reached it.
const IN_FROM_THE_EDGE: f32 = 34.0;

/// **And how far above the floor**, which is the rules' own low line.
const ABOVE_THE_FLOOR: f32 = 64.0;

/// **Where th06c's `禁弾「過去を刻む時計」` is waited out, by the clock that is up** — at the bottom of the
/// field, out of a clock's reach: across from a low one and under a high one.
///
/// **Two clocks are up at once, each of four hands out of one centre**, read off seven attempts' beams
/// (`--trace-the-field-in=61`, 2026-09-27): a cross of lasers `0..216` along and `6.0` across, turning at
/// about `0.02` rad a frame, round a centre that travels across the field at about `1.17` a frame. From card
/// frame `160` one is centred at `64,288` and travels right and the other at `320,160` and travels left, the
/// two turning opposite ways; their hands warn for sixty frames and the centres move from about `210` to
/// `430`, and the pair comes again from `500`. **The rules died climbing into them**: the attempts ended at
/// card frames `357` to `359` and `707` to `708`, every hit near `y = 200`, where the bottom is out of the
/// high clock's reach altogether — `160 + 216` being well above it.
///
/// **So the bottom is where the card is waited out**, and a clock whose reach covers it — the low one — is
/// waited out across from its centre: at the edge it is travelling towards while it is far enough off, and
/// behind it once it comes near, which is the wiki's `右から左` under the first.
///
/// **And with no clock up, out of reach of where the next low one comes up** — see [`the_next_pair`]. With no
/// clock up the card was the rules' own, and one run's rules were under the boss at `154,432` when a pair came up
/// at card frame `1250` with its low clock at `64,288`: inside that clock's reach, they kept clear of its hand
/// sweeping down while it warned by going the way it swept, under its centre to the left wall, and the turning
/// hands walked them up the wall and under the high clock, hit at `178.6,202.0`. Six other runs' chapter 25 went
/// through with the gaps left to the rules.
pub(crate) fn the_clock(frame: &Frame, card_frames: Option<i32>) -> Option<(f32, f32)> {
    let field = frame.field;
    let floor = field.1 + field.3 - A_CLOCK_IS_WAITED_ABOVE_THE_FLOOR;
    let reach = A_HANDS_REACH + CLEAR_OF_A_HAND;
    let clocks = the_clocks_centres(frame);
    if clocks.is_empty() {
        return match the_next_pair(card_frames?) {
            Pair::LowFromTheLeft => Some((field.0 + field.2 - IN_FROM_THE_EDGE, floor)),
            Pair::LowFromTheRight => Some((field.0 + IN_FROM_THE_EDGE, floor)),
            Pair::FromTheTopCorners => None,
        };
    }
    let low = clocks.into_iter().find(|centre| floor - centre.1 < reach);
    let mut travelling = CLOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(centre) = low else {
        *travelling = None;
        return Some((frame.at.0.clamp(field.0, field.0 + field.2), floor));
    };
    // **Which way it travels**: the way it last moved, and before it has moved, away from the side of the
    // field it came up on. A centre that jumps is another clock.
    let middle = field.0 + field.2 / 2.0;
    let towards = match *travelling {
        Some((was, _)) if (centre.0 - was).abs() > A_CLOCK_JUMPS => None,
        Some((was, _)) if centre.0 > was + A_CLOCK_STEPS => Some(1.0),
        Some((was, _)) if centre.0 < was - A_CLOCK_STEPS => Some(-1.0),
        Some((_, towards)) => Some(towards),
        None => None,
    }
    .unwrap_or(match centre.0 < middle {
        true => 1.0,
        false => -1.0,
    });
    *travelling = Some((centre.0, towards));
    // How far across from the centre the bottom is out of reach.
    let below = floor - centre.1;
    let across = (reach * reach - below * below).sqrt();
    let (ahead, behind) = match towards > 0.0 {
        true => (
            field.0 + field.2 - IN_FROM_THE_EDGE,
            field.0 + IN_FROM_THE_EDGE,
        ),
        false => (
            field.0 + IN_FROM_THE_EDGE,
            field.0 + field.2 - IN_FROM_THE_EDGE,
        ),
    };
    let still_ahead = (frame.at.0 - centre.0) * towards > 0.0;
    let x = match still_ahead && (ahead - centre.0).abs() > across {
        true => ahead,
        false => match towards > 0.0 {
            true => (centre.0 - across).max(behind),
            false => (centre.0 + across).min(behind),
        },
    };
    Some((x, floor))
}

/// **The three kinds of pair the card puts up, by the one after `card_frames`**, read off one run's traced card
/// (`--trace-the-field-in=61`, 2026-09-27): from card frame `160` the low clock at `64,288` travelling right and
/// the high one at `320,160` travelling left; from `510` the two the other way round, the low one at `320,288`
/// travelling left; from `860` both out of the top corners at `64,128` and `320,128`, crossing on their way down,
/// neither reaching the floor to begin with; and from `1250` the first again — a round of `1090`.
fn the_next_pair(card_frames: i32) -> Pair {
    match (card_frames - THE_FIRST_PAIR_COMES_UP_AT).rem_euclid(A_ROUND_OF_PAIRS) {
        into if into < THE_SECOND_PAIR_COMES_UP_AFTER => Pair::LowFromTheRight,
        into if into < THE_THIRD_PAIR_COMES_UP_AFTER => Pair::FromTheTopCorners,
        _ => Pair::LowFromTheLeft,
    }
}

/// The pairs of [`the_next_pair`].
enum Pair {
    LowFromTheLeft,
    LowFromTheRight,
    FromTheTopCorners,
}

/// The card frames [`the_next_pair`] counts its round by.
const THE_FIRST_PAIR_COMES_UP_AT: i32 = 160;
const THE_SECOND_PAIR_COMES_UP_AFTER: i32 = 350;
const THE_THIRD_PAIR_COMES_UP_AFTER: i32 = 700;
const A_ROUND_OF_PAIRS: i32 = 1090;

/// What [`the_clock`] keeps between frames: where the low clock's centre was, and which way it travels.
static CLOCK: std::sync::Mutex<Option<(f32, f32)>> = std::sync::Mutex::new(None);

/// **How far a centre moves in a frame to be travelling**, the clocks going `1.17` a frame, and how far it
/// jumps to be another clock.
const A_CLOCK_STEPS: f32 = 0.5;
const A_CLOCK_JUMPS: f32 = 24.0;

/// **The centres the hands of a clock turn round**: every origin at least [`A_CLOCKS_HANDS`] of the beams
/// share, and nothing where no clock is up.
fn the_clocks_centres(frame: &Frame) -> Vec<(f32, f32)> {
    let mut centres: Vec<(f32, f32)> = Vec::new();
    for beam in &frame.beams {
        let hands = frame
            .beams
            .iter()
            .filter(|other| other.origin == beam.origin)
            .count();
        if hands >= A_CLOCKS_HANDS && !centres.contains(&beam.origin) {
            centres.push(beam.origin);
        }
    }
    centres
}

/// How many hands a clock has, which is what tells one from a laser of the card's own: a cross.
const A_CLOCKS_HANDS: usize = 4;

/// **How far a clock's hands reach from its centre**, `0..216` along, and how far past that a place is clear
/// of one: a hand's own `6.0` across, the run's `1.2`, and eight to spare.
const A_HANDS_REACH: f32 = 216.0;
const CLEAR_OF_A_HAND: f32 = 16.0;

/// **How far above the floor a clock is waited out**: the lower the further from a low clock's centre, and
/// a step of the rules' to dodge with under it.
const A_CLOCK_IS_WAITED_ABOVE_THE_FLOOR: f32 = 8.0;

/// **Where th06c's `秘弾「そして誰もいなくなるか？」` goes next, by how far into the card it is**: while its chasers
/// are up, the corner they are being led into ([`the_lead_into_the_corners`]); once they are all gone, the
/// bottom middle of the field, a quarter of its height up ([`the_card_s_end_is_waited_out_at`]).
///
/// **Waited out there and not walked round**, which is what the card's second half wants (asked for 2026-09-27:
/// `後半は回る必要ないよ。中央のしたから1/4くらいで避ければ良い`): the chasers led into the corners have left their
/// fire there and along the walls, and once they are gone that fire streams outwards to the walls — `0.5` a frame
/// outwards to `0.8` from card frame `2100` and `1.1` from `2400`, read off the traced bullets' steps — so the
/// middle is where it leaves, and a walk round the walls is a walk into it.
///
/// **The whole of the chasers' half and not only the one chaser's**, which is where this first handed over: at
/// card frame `1020`, the four more coming in out of the corners, a walk round the field took the rules from the
/// bottom right up through the middle of it and the five closed on them there, hit at `376,185` on `1551` — the
/// lead left off just as there was most to lead.
pub(crate) fn and_then_there_were_none(frame: &Frame, card_frames: Option<i32>) -> (f32, f32) {
    let frames = card_frames.unwrap_or(0);
    match frames >= THE_CORNERS_CHASERS_COME_AT && the_chasers(frame).next().is_none() {
        true => the_card_s_end_is_waited_out_at(frame),
        false => the_lead_into_the_corners(frame, frames),
    }
}

/// **The bottom middle of the field, [`A_QUARTER_UP`] of its height above its floor** — where
/// `「そして誰もいなくなるか？」` is waited out once its chasers are gone.
fn the_card_s_end_is_waited_out_at(frame: &Frame) -> (f32, f32) {
    let field = frame.field;
    (
        field.0 + field.2 / 2.0,
        field.1 + field.3 * (1.0 - A_QUARTER_UP),
    )
}

/// How far up the field `「そして誰もいなくなるか？」`'s end is waited out, as a share of its height.
const A_QUARTER_UP: f32 = 0.25;

/// **The card frame the four chasers come in out of the field's corners from** — the first at `0,0` on
/// `1020`, read off the card's bodies (`--trace-the-field-in=62`, 2026-09-27), the three others by `1200` —
/// before which the field with no chaser on it is the card before its first has come out, and after which it
/// is the card with its last gone, by `1860`.
const THE_CORNERS_CHASERS_COME_AT: i32 = 1020;

/// **The card's chasers**: every body nothing collides with but the boss's own, which stands still at `192,80`
/// the whole card and chases nothing.
fn the_chasers(frame: &Frame) -> impl Iterator<Item = (f32, f32)> + '_ {
    frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Body { solid: false }))
        .map(|hazard| hazard.at)
        .filter(|at| {
            frame
                .boss
                .is_none_or(|boss| distance(*at, boss) > THE_BOSSES_OWN_BODY_WITHIN)
        })
}

/// **The corner the card's chasers are led into**: waited at until a chaser has come within
/// [`A_CHASER_IS_LED_ON_WITHIN`] of the run, and then the next corner clockwise, from the bottom left — the
/// strategy wiki's `下→左下→左上→右上`, [`A_LEAD_S_CORNERS_IN_FROM_THE_WALLS`] in from the walls.
///
/// **So that what the chasers drop is dropped in the corners.** What the card sends at the run is its chasers,
/// read off its bodies (`--trace-the-field-in=62`, 2026-09-27): bodies `37` across and not solid, the boss's own
/// standing still at `192,80` the whole card. One follows the run from the card's start; from frame `1020` to
/// `1200` four more come in, one out of each corner of the field — `0,0`, `384,0`, `0,448` and `384,448` in that
/// order — and all of them follow it, dropping the slow bullets that fill the field to its `640` from `1320` to
/// `1680`, and are gone by `1860`. Led into a corner and away along the edge, their fire piles up there and along
/// the walls, and the middle of the field is left open. Led so from a pick-up of the card, the one chaser came
/// into each corner behind the run — `31,407` at the bottom left on card frame `360`, `31,90` at the top left on
/// `540`, `342,40` at the top right on `900` — and the card was taken on the first attempt on three runs'
/// recordings of it, one the walk round below had lost.
///
/// **And not walked round the field**, which was tried four ways and each lost a recording — and was not the
/// card's way at all (`自機がつねに公転するように回るのは微妙`): the lower half of the field, as th06's
/// `round_the_field` walks it, was hit at the side walls; the whole field's edge one way round every time lost
/// one recording either way, clockwise up the left wall at `60.1,305.9` on `1393` and counter-clockwise along the
/// top at `185,16` on `1447`, the chasers from `384,0` and `0,0` closing on it from either end; turned away from
/// the chasers at each corner it went back and forth along the bottom and was met in the corner by all five, on
/// `1392` and `1607`; and round on arcs without stopping, it left the chaser `50` to `150` behind the run the
/// whole way, its fire in a ring a little inside the walk's own.
fn the_lead_into_the_corners(frame: &Frame, card_frames: i32) -> (f32, f32) {
    let field = frame.field;
    let inset = A_LEAD_S_CORNERS_IN_FROM_THE_WALLS;
    let (left, top) = (field.0 + inset, field.1 + inset);
    let (right, bottom) = (field.0 + field.2 - inset, field.1 + field.3 - inset);
    // Clockwise, so that the corner after each one is the next one round.
    let corners = [(left, top), (right, top), (right, bottom), (left, bottom)];
    let the_bottom_left = 3;
    let mut lead = THE_LEAD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A card begun again — a retry — is led from its first corner again.
    if card_frames < lead.card_frames {
        lead.to = the_bottom_left;
    }
    lead.card_frames = card_frames;
    let there = distance(corners[lead.to], frame.at) <= A_CORNER_IS_REACHED_WITHIN;
    let led_on =
        the_chasers(frame).any(|chaser| distance(chaser, frame.at) <= A_CHASER_IS_LED_ON_WITHIN);
    if there && led_on {
        lead.to = (lead.to + 1) % corners.len();
    }
    corners[lead.to]
}

fn distance(one: (f32, f32), other: (f32, f32)) -> f32 {
    (one.0 - other.0).hypot(one.1 - other.1)
}

/// What [`the_lead_into_the_corners`] keeps between frames: the corner it is leading the chaser into, and the
/// card frame it last read, which going back is a card begun again.
struct Lead {
    to: usize,
    card_frames: i32,
}

static THE_LEAD: Mutex<Lead> = Mutex::new(Lead {
    to: 3,
    card_frames: 0,
});

/// **How near a corner the run is to be at it**: the lead is the rules' errand and not a place held, so the
/// run stands a few units off it while it dodges.
const A_CORNER_IS_REACHED_WITHIN: f32 = 16.0;

/// **How near the run the chaser comes before the run goes on to the next corner**: its own `37` across and a
/// little room, so that it has come into the corner behind the run and not stopped short of it.
const A_CHASER_IS_LED_ON_WITHIN: f32 = 64.0;

/// **How near the boss a body is to be hers**, which stands still at `192,80` the whole card and chases nothing.
const THE_BOSSES_OWN_BODY_WITHIN: f32 = 4.0;

/// **How far in from the walls `「そして誰もいなくなるか？」`'s chasers are led to**: room to dodge either way at
/// a corner, which the walls themselves leave none of.
///
/// **Forty-eight and not twenty-four**, which was tried first: led `24` in, the five chasers along the top to the
/// top right and on down the right side, the rules were pushed onto the right wall itself — `376`, eight in —
/// from card frame `1680`, and held there by the fire streaming out to the walls until the card's end, on all
/// three recordings it was played on; one of them was hit there at `1934` and every retry at `1949`.
const A_LEAD_S_CORNERS_IN_FROM_THE_WALLS: f32 = 48.0;

/// **Where th08's `「インペリシャブルシューティング」` is waited at, by how far into the card it is** — and nothing
/// before its last wave, where the rules' own errand is the way.
///
/// **Its waves come at the same frames of the card every launch**, read off the frames its bullet count
/// jumped on in three Extras, each within a frame of the others: `124`, `575`, three at `1067`, `1095` and
/// `1127`, five `132` apart from `1559` to `2087`, five `8` to `12` apart from `2539` and from `2959`, one
/// from about `3363`, and the last from `4003`, which fills the bullet pool to its `1536` by `4123` and runs out at
/// about `4700`. The strategy wiki's line for that last one is `画面の右下端or左下端で避けると楽`.
///
/// **The rules' own through the seven before it**, which is under Mokou in the middle at the low line: they
/// took those waves with one hit in the three Extras, in the fifth, and the one thing a told place would add
/// there is the crossing to it. Through the last, one of those Extras found the bottom-right corner on its
/// own — at `331` to `370` along the floor for its last `300` frames, and was not hit.
pub(crate) fn imperishable_shooting(frame: &Frame, card_frames: i32) -> Option<(f32, f32)> {
    let field = frame.field;
    (card_frames >= IMPERISHABLE_SHOOTINGS_LAST_WAVE).then_some((
        field.0 + field.2 - IN_FROM_THE_RIGHT_WALL,
        field.1 + field.3,
    ))
}

/// The frame of `「インペリシャブルシューティング」` its last wave starts on — see [`imperishable_shooting`].
const IMPERISHABLE_SHOOTINGS_LAST_WAVE: i32 = 4000;

/// **How far in from the right wall that wave is waited at** — `352` on a field whose right is `376`, the middle
/// of the `331` to `370` the run that found the corner stood at.
const IN_FROM_THE_RIGHT_WALL: f32 = 24.0;

/// **Whether th08's `狂符「幻視調律（ビジョナリチューニング）」` has its lattice where none of it can hit the run**, by how
/// far into the card it is: hidden, and not shown again for more than [`BEFORE_IT_IS_SHOWN`] frames.
///
/// **Its lattice is hidden on the same frames of the card every launch**, read off every bullet of a Normal
/// practice of it frame by frame (`--trace-the-field-in=105`): shown from its first bullets at `91` to `340`, then
/// hidden for a hundred frames and shown for a hundred to the card's end, the first hidden frame `341` and the
/// first shown `441`. Shown, a bullet goes straight up or straight down at `1.3` a frame — born in pairs in a band
/// from about `y = 190` to `250`, one of each pair going each way — so that below the band the lattice is one body
/// going down: at the card's `1060`th frame, `306` of the `307` bullets between `250` and `450` were. Hidden, every
/// one of them goes sideways at `0.6` a frame, either way, and none can hit — see `th08::BULLET_UNTESTED` — where
/// `sight` hands them over as hitting all the same.
///
/// **So while it is hidden the run climbs, which the rules alone never do** — the strategy wiki's
/// `どんどん上を意識していこう`. A body of bullets coming down is not climbed against: every move up meets it faster
/// than standing does, and the moves that give way to it give way downwards, so the rules waiting at `260` were
/// carried to the floor — the practice reached it on the card's `681`st frame and was at the right wall's foot by
/// its `941`st, and two clears of one session spent a bomb twice apiece in it on stretches with nothing open,
/// three of the four at `y = 413` and below. And a lattice that cannot hit is no more climbed through, the rules
/// dodging it as though it could.
///
/// **And not for the last [`BEFORE_IT_IS_SHOWN`] frames**, where the rules' own dodging is what puts the run clear
/// of every bullet by the frame they can hit again: a hidden bullet is shown where it has got to.
pub(crate) fn visionary_tunings_lattice_cannot_hit(card_frames: i32) -> bool {
    let into = card_frames - VISIONARY_TUNINGS_FIRST_HIDDEN;
    into >= 0 && into % VISIONARY_TUNINGS_CYCLE < VISIONARY_TUNINGS_HIDDEN_FOR - BEFORE_IT_IS_SHOWN
}

/// The frame of `「幻視調律」` its lattice is first hidden on, and the frames of a hide and a show together and
/// of the hide alone — see [`visionary_tunings_lattice_cannot_hit`].
const VISIONARY_TUNINGS_FIRST_HIDDEN: i32 = 341;
const VISIONARY_TUNINGS_CYCLE: i32 = 200;
const VISIONARY_TUNINGS_HIDDEN_FOR: i32 = 100;

/// **How many frames before `「幻視調律」`'s lattice is shown again the rules dodge it again** — twenty, which is
/// room for the climb and room after it.
///
/// **The climb to [`VISIONARY_TUNING_ABOVE_THE_FLOOR`] is `36` frames from the floor** at the rules' whole step of
/// `4`, so the eighty frames of a hide this leaves are twice that; and a hidden bullet drifts `0.6` a frame,
/// so twenty frames of the rules' own dodging is a run that has five steps of its own to be out of any box it
/// is standing in by the frame the box can hit.
const BEFORE_IT_IS_SHOWN: i32 = 20;

/// **How far above the field's floor `「幻視調律」` is waited at** — `290` on a field whose floor is `432` — and in
/// the middle.
///
/// **Because the lattice carries the run down with it for a hundred frames at a time**, `130` of the game's
/// units at its `1.3` a frame: from here that ends at `420` and not against the floor, where the rules have
/// no move left to give way with. **And no higher**, the band from `190` to `250` being where every bullet of
/// the card is born: forty below it is thirty frames of a bullet born at its foot. **Nor at the low line**,
/// under which twenty-two attempts told nothing were shut in, `nowhere open to it` at `376.0,405.4` and
/// `200.4,401.7`.
///
/// **And in the middle**, which is the x the launches before the climb measured: two passed the card on one
/// death with a height told and the x left to the errand, and a third died fourteen times at `8.0,432.0`, the
/// field's own bottom-left corner.
///
/// **And for this card and not for rings that close on the run, which a wait this high was tried for.** A card
/// that sends rings out of a boss at the top of the middle and hides and shows them again was waited for at
/// `260` on one no-bomb clear, the strategy wiki's way through it being down with each ring and back up once it
/// is gone — and the run was hit at `14.0,423.5`, the bottom-left corner, on six attempts running, where waiting
/// at the low line it had taken the card with no death at all. `Dodge::Focused` is that card's.
pub(crate) const VISIONARY_TUNING_ABOVE_THE_FLOOR: f32 = 142.0;

/// **Where th10's `開宴「二拝二拍一拝」` is waited out: on a first attempt where its spokes leave room as its frames
/// go ([`where_the_spokes_leave_room`]); on a retry its fire led, [`TWO_BOWS_ABOVE_THE_FLOOR`] up, to the middle or to
/// a place off where the spokes close as it takes aim, the run down and to one side as it comes, and back to the
/// place once it has gone through.**
///
/// **Every bullet of the card's fire is aimed at where the run is at [`TWO_BOWS_AIMED_AT`] frames old**, read
/// off one attempt's bullets (`--trace-the-field-in=100` from keep\at-99, 2026-10-02, `big100aim.py`): two lots of
/// sixty — four of `28` across, eight of `10`, sixteen of `6`, thirty-two of `4` — born 30 frames apart every 240
/// (card frames 150 and 180, 390 and 420, …), thrown up and slowing, and from that age on speeding up straight at
/// the place the run stood then: 28 of the 32 big ones the run's direction at age 49 to 51 within `0.06°`, and the
/// four others born while the run stood still, every age the same direction. **So it is the run that says where the
/// fire goes**, which is the strategy wiki's and the user's way through it (大玉の移動開始のタイミングで誘導): stood
/// at one place while the lots take aim, both come down on that place — **as a band and not through a point**, the
/// small ones aimed from wherever they were thrown to: led to `0,320`, the lots crossed `y = 320` between `-78` and
/// `57` across, `352` between `-107` and `82`, and the floor between `-164` and `138` (`band100.py`, from the chapter
/// a whole Extra kept, 2026-10-02) — the wiki's `画面下に居すぎると弾がばらけて危険`.
///
/// **And not planned against the fire where the run is, as the way before this was**: whatever it judged, the
/// places it judged were wherever the rules' own dodging had carried the run, so the fire went there too — it went
/// along the floor into the bottom-right corner from keep\at-99, hit at `115.0,432.0` 1,143 frames in, and with a
/// place led to judged near the run, the place was `158,432` in that corner, hit 1,140 frames in.
///
/// **Its lasers close on one direction out of the boss**: a laser let go every four frames `2.25°` on from the last,
/// each going straight out along its own angle and speeding up as it goes — `166` out 49 frames after it was let go,
/// `353` out 81 after (`far100.py`) — so that a spoke at any one distance is a wall, lasers `7` across a few apart,
/// open only far out, at the floor. Spokes
/// set off in pairs a half turn apart and sweep towards each other (`sweep100.py`): from keep\at-99 from `-28.8°`
/// and `151.2°`, closing on `61°`; from the chapter a whole Extra kept from `6.1°` and `186.1°`, closing on `96°`
/// — the wiki's `会話終了時の自機位置を基準とする`. Led to the middle, the second pair's close was the place led to
/// itself: the rules' dodging of the spokes carried the run to the floor on every throw, and the lots' band came
/// down on it there, hit at `71.5,410.8` 407 frames in on every one of a hundred attempts from that chapter, and at
/// `78.4,432.0` with the run taken to the far side; led off the close by `30°`, the place to the side was against
/// the right wall and the run was hit on the sixth throw, `126.6,432.0` 1,361 frames in; by `20°`, the card was
/// taken with no hit. **And only there**: keep\at-99's and keep\at-100b's close is `29°` off the middle, and led
/// `20°` off it — `81°`, the place the spoke from the left sweeps across before it reaches the close — the run was
/// carried into the bottom-right corner, `184.0,425.6` 864 frames in from both; led to the middle and down `56`
/// aside on the side the spokes sweep away from, both took the card with no hit — and with the `112` aside and `16`
/// down the place led off the close wants, or on a side kept for the card, hit at `-82.0,432.0` 1,602 frames in and
/// at `885`. **A chapter picked up partway through the card** sees no first laser, and is played as the middle's.
pub(crate) fn two_bows_two_claps_one_bow(frame: &Frame, card_frames: Option<i32>) -> (f32, f32) {
    let mut planned = TWO_BOWS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A retry of its chapter starts the card over, with nothing led anywhere — and counted, so that it is played
    // another way: a retry rewound into the same chapter plays it out exactly as the attempt before did, and a way
    // that loses it once from there loses it every time, card 100 from a whole Extra's chapter at `50.0,432.0`
    // 1,838 frames in on ten attempts running (2026-10-02 15:00), where a pick-up of that chapter in a fresh launch
    // took it with no hit.
    if planned
        .on
        .is_some_and(|was| card_frames.is_some_and(|now| now < was))
    {
        *planned = TwoBows {
            retries: planned.retries + 1,
            ..TwoBows::NONE
        };
    }
    if planned.on.is_some() && planned.on == card_frames {
        return planned.there;
    }
    let field = frame.field;
    let middle = field.0 + field.2 / 2.0;
    let floor = field.1 + field.3;
    let height = floor - TWO_BOWS_ABOVE_THE_FLOOR;
    let inside = field.0 + TWO_BOWS_OFF_THE_WALLS..=field.0 + field.2 - TWO_BOWS_OFF_THE_WALLS;
    // The card's first laser is let go a quarter turn short of where its spokes close — read only where it is the
    // card's first, which a chapter picked up partway through the card has long gone past.
    if planned.spokes_close_on.is_none()
        && let Some(frames) = card_frames.filter(|frames| *frames <= TWO_BOWS_FIRST_LASER_BY)
        && let Some(first) = frame.beams.iter().max_by_key(|beam| beam.frames)
    {
        planned.spokes_close_on = Some(first.angle + FRAC_PI_2);
        planned.first_laser_on = Some(frames - first.frames + 1);
    }
    // Where a line out of the boss at an angle crosses the height led to, across.
    let across_at_the_height = |angle: f32| {
        frame
            .boss
            .filter(|_| angle.sin() > TWO_BOWS_LOW_ENOUGH)
            .map(|boss| boss.0 + (height - boss.1) / angle.sin() * angle.cos())
    };
    let closing = planned.spokes_close_on.and_then(across_at_the_height);
    let way = TWO_BOWS_WAYS[planned.retries % TWO_BOWS_WAYS.len()];
    if way.led == Led::ByTheSpokes
        && let (Some(close), Some(first_on), Some(boss), Some(frames)) = (
            planned.spokes_close_on,
            planned.first_laser_on,
            frame.boss,
            card_frames,
        )
    {
        let (off, out) = where_the_spokes_leave_room(frames - first_on);
        let (sin, cos) = (close + off.to_radians()).sin_cos();
        // Shortened along its line to stay inside the walls and off the floor, so that the place keeps its angle
        // about the boss, which is what the spokes are dodged by.
        let lured_inside = field.0 + TWO_BOWS_LURED_OFF_THE_WALLS
            ..=field.0 + field.2 - TWO_BOWS_LURED_OFF_THE_WALLS;
        let to_the_wall = match cos {
            cos if cos < 0.0 => (boss.0 - lured_inside.start()) / -cos,
            cos if cos > 0.0 => (lured_inside.end() - boss.0) / cos,
            _ => f32::INFINITY,
        };
        let to_the_floor = match sin {
            sin if sin > 0.0 => (floor - TWO_BOWS_OFF_THE_WALLS - boss.1) / sin,
            _ => f32::INFINITY,
        };
        let out = out.min(to_the_wall).min(to_the_floor).max(0.0);
        // **Shortened and not lowered as well**, though shortened the place is near the boss's own height — `120,179`
        // to `217` and `-120,164` to `220` — and a run the last lots had pushed down to `363` there had no way up to
        // it and was carried into the wall, 1,779 and 1,785 frames into the card (u37) and 2,040 (u38): lowered to
        // `320` where the wall shortened it, five of the seven c100-wall.sh took six of went through, keep\at-100b
        // hit at `180.5,432.0` 1,123 frames in and keep\chain100c sooner than before (c100-low.sh, 2026-10-04).
        let there = (boss.0 + cos * out, boss.1 + sin * out);
        planned.on = card_frames;
        planned.there = there;
        return there;
    }
    let led_to_x = match way.led {
        Led::ByTheSpokes => None,
        Led::ToTheMiddle => None,
        Led::OffTheClose {
            by,
            nearer_the_middle,
        } => planned.spokes_close_on.and_then(|close| {
            let crossings = [close - by.to_radians(), close + by.to_radians()]
                .into_iter()
                .filter_map(across_at_the_height);
            let off_the_middle = |x: &f32| (x - middle).abs();
            match nearer_the_middle {
                true => crossings.min_by(|a, b| off_the_middle(a).total_cmp(&off_the_middle(b))),
                false => crossings.max_by(|a, b| off_the_middle(a).total_cmp(&off_the_middle(b))),
            }
        }),
    }
    .unwrap_or(middle)
    .clamp(*inside.start(), *inside.end());
    let led_to = (led_to_x, height);
    let bullets: Vec<(i32, (f32, f32))> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter_map(|hazard| hazard.frames.map(|age| (age, hazard.at)))
        .collect();
    let still_to_aim = bullets.iter().any(|(age, _)| *age < TWO_BOWS_AIMED_AT);
    // **Coming, and the throw's whole**: aimed at the place led to and not yet through its height — and aimed
    // longer ago than a throw's two lots are apart, so that a throw's first lot coming while its second is still
    // to aim keeps the run stood where the second is led to. **And born inside the card**: the fire before it is
    // still on its way out as it begins, and taken for a throw it sent the run aside of a place nothing had aimed at
    // — `112,16`, the top of the field, the card's first frame from keep\hits-u22\s7-c11-f14712.
    let coming = bullets.iter().any(|(age, at)| {
        card_frames.is_some_and(|frames| *age <= frames)
            && *age >= TWO_BOWS_AIMED_AT + TWO_BOWS_LOTS_APART
            && at.1 < led_to.1 + TWO_BOWS_GONE_THROUGH_BELOW
            && (field.0..field.0 + field.2).contains(&at.0)
            && at.1 > field.1
    });
    let there = if coming {
        // **Down and to the side of the place the throw was aimed at while it comes**, the user's way through it
        // (たまがこちらにむかってきただんかいでみぎしたとかひだりしたにいく): the band the lots come down in is
        // narrowest at that height and spreads below it. Chosen once a throw, the side away from where the spokes
        // close — the other is where they were seen to close on the run — held off the walls. **And not the side
        // the spokes sweep away from where the fire is led to the middle**, which was that side from keep\at-99
        // (closing on `61°`) and took its card, and the side of the close from the chapter keep\chain100c holds
        // (closing on `121°`): the run went down to the bottom-left and stayed there throw after throw, hit at
        // `-59.8,432.0` to `-164.0,432.0` some 1,360 frames in on all ~270 attempts of a whole Extra. The sweep is
        // what is left where no close was read, the chapter picked up past the card's first laser.
        //
        // **And not aside of where it was aimed and towards the middle, for a way gone aside of the middle, where the
        // throw was aimed off it**, which was tried: from keep\hits-x16-pd\s7-c11-f15707 — the run still `70` left of
        // the middle on its way back as the throw took aim, sent to `56,352` across the band and hit at
        // `-133.4,432.0` 1,851 frames in — going to `-32,352` and `-68,352` instead, it was carried to the left wall
        // throw after throw all the same, and hit at `-54.2,432.0` 1,831 frames in.
        if !planned.side_chosen {
            planned.side = match closing {
                Some(closing) if closing > planned.aimed_at.0 => -1.0,
                Some(_) => 1.0,
                None => the_side_the_spokes_sweep_away_from(frame),
            };
            planned.side_chosen = true;
        }
        match way.aside {
            Aside::OfWhereItAimed => (
                (planned.aimed_at.0 + planned.side * TWO_BOWS_GOES_ACROSS)
                    .clamp(*inside.start(), *inside.end()),
                planned.aimed_at.1 + TWO_BOWS_GOES_DOWN_BY,
            ),
            Aside::OfTheMiddle => (
                middle + planned.side * TWO_BOWS_GOES_ACROSS_FROM_THE_MIDDLE,
                height + TWO_BOWS_GOES_DOWN_FROM_THE_MIDDLE,
            ),
        }
    } else {
        // **And stood at the place led to otherwise**, which is where the next throw takes aim — a place of the way's
        // own and not one judged against the fire where the run happens to be: judged, the place went wherever
        // the rules' dodging had carried the run, led to `158,432`, the bottom-right corner, from keep\at-99 and
        // hit there 1,140 frames in on every attempt. **Back as soon as the lot is through and not a count of
        // frames after it aimed**: waited out by frames, the run was still on its way back as the next throw
        // took aim, led it to `100,350` and was carried with it into the bottom-right corner, hit at `184.0,432.0`
        // 1,099 frames in on every attempt.
        if still_to_aim {
            planned.side_chosen = false;
            planned.aimed_at = frame.at;
        }
        // **Up first where the run is below the place while a lot through it is still in the field below it, and
        // across at its height after**: that lot is spreading on below the place, and the straight way back from
        // low crosses it — the rules' dodging of a
        // spoke carried the run to the floor and over to `97,432`, outside the lot's spread there, and the way back
        // to the middle went along the floor into it, hit at `71.5,410.8` 407 frames in on every attempt from the
        // chapter a whole Extra kept (2026-10-02).
        let spreading_below = bullets.iter().any(|(age, at)| {
            *age >= TWO_BOWS_AIMED_AT
                && at.1 > led_to.1
                && (field.0..field.0 + field.2).contains(&at.0)
                && at.1 < floor
        });
        if way.aside == Aside::OfWhereItAimed
            && spreading_below
            && frame.at.1 > led_to.1 + TWO_BOWS_GOES_DOWN_BY
        {
            (frame.at.0, led_to.1)
        } else {
            led_to
        }
    };
    planned.on = card_frames;
    planned.there = there;
    there
}

/// **What [`two_bows_two_claps_one_bow`] keeps**: where it planned the run to be and on which of the card's frames,
/// where the run stood as the throw took aim, and which side of that the run goes to while the throw comes, chosen
/// once a throw.
struct TwoBows {
    on: Option<i32>,
    there: (f32, f32),
    aimed_at: (f32, f32),
    side: f32,
    side_chosen: bool,
    spokes_close_on: Option<f32>,
    first_laser_on: Option<i32>,
    retries: usize,
}
impl TwoBows {
    const NONE: TwoBows = TwoBows {
        on: None,
        there: (0.0, 0.0),
        aimed_at: (0.0, 0.0),
        side: 1.0,
        side_chosen: false,
        spokes_close_on: None,
        first_laser_on: None,
        retries: 0,
    };
}
/// **One way of playing a throw: where its fire is led, and what the run goes aside of as it comes.**
#[derive(Clone, Copy)]
struct Way {
    led: Led,
    aside: Aside,
}

#[derive(Clone, Copy, PartialEq)]
enum Led {
    /// Wherever the spokes leave room as the card's own frames go, out of [`where_the_spokes_leave_room`] — the
    /// fire led there with the rest, and nothing judged against it.
    ByTheSpokes,
    ToTheMiddle,
    /// Where a line `by` degrees round from the spokes' close crosses the height, the crossing of the two nearer
    /// the middle or the one further from it; the middle where no close was read.
    OffTheClose {
        by: f32,
        nearer_the_middle: bool,
    },
}

#[derive(Clone, Copy, PartialEq)]
enum Aside {
    /// [`TWO_BOWS_GOES_ACROSS`] aside and [`TWO_BOWS_GOES_DOWN_BY`] down of where the run stood as the throw aimed.
    OfWhereItAimed,
    /// [`TWO_BOWS_GOES_ACROSS_FROM_THE_MIDDLE`] aside and [`TWO_BOWS_GOES_DOWN_FROM_THE_MIDDLE`] down of the middle.
    OfTheMiddle,
}

/// **The ways the card is played, a retry taking the one after the way before it**: a retry rewound into the same
/// chapter plays the card out as the attempt before did, so a chapter one way loses is lost on every attempt but for
/// another way.
///
/// **The first attempt by the spokes' schedule** ([`Led::ByTheSpokes`]): played from eight keeps of the card's start,
/// one seed apiece (`bench100.sh`, 2026-10-03), it took six — the first attempt played led `20°` off the close where
/// the spokes close within `20°` of straight down and to the middle where they close further off took two, both
/// readings out of four keeps that the eight went against: of those four, led `20°` off took two and lost two, led to
/// the middle took the other two, and keep\chain100c (closing on `121°`) was lost by both. The ways after it are
/// those, kept for the retries.
const TWO_BOWS_WAYS: [Way; 8] = [
    Way {
        led: Led::ByTheSpokes,
        aside: Aside::OfWhereItAimed,
    },
    Way {
        led: Led::OffTheClose {
            by: 20.0,
            nearer_the_middle: true,
        },
        aside: Aside::OfWhereItAimed,
    },
    Way {
        led: Led::ToTheMiddle,
        aside: Aside::OfWhereItAimed,
    },
    Way {
        led: Led::OffTheClose {
            by: 30.0,
            nearer_the_middle: true,
        },
        aside: Aside::OfWhereItAimed,
    },
    Way {
        led: Led::OffTheClose {
            by: 20.0,
            nearer_the_middle: false,
        },
        aside: Aside::OfWhereItAimed,
    },
    Way {
        led: Led::ToTheMiddle,
        aside: Aside::OfTheMiddle,
    },
    Way {
        led: Led::OffTheClose {
            by: 30.0,
            nearer_the_middle: false,
        },
        aside: Aside::OfWhereItAimed,
    },
    Way {
        led: Led::OffTheClose {
            by: 10.0,
            nearer_the_middle: true,
        },
        aside: Aside::OfWhereItAimed,
    },
];
/// **The card frame its first laser is let go by**: `92` from keep\at-99 and from keep\chain100 (`sweep100.py`), the
/// next spoke not before `122`.
const TWO_BOWS_FIRST_LASER_BY: i32 = 100;
/// How far below level out of the boss a line has to point to be one whose crossing of the height led to is
/// counted — the sine of some `12°`, past which the crossing is far outside the field.
const TWO_BOWS_LOW_ENOUGH: f32 = 0.2;
/// **Where the run goes to as a throw led off the close comes**: `112` to the side of the place it was aimed at and
/// `16` below it — outside the band the lots come down in at that height, which led to `0,320` was `-78` to `57`
/// across at `320` and `-107` to `82` at `352`, by `20` and more.
///
/// **And not the middle's `56` aside and `32` below, where the spokes close on the middle**, out of a reading that
/// the lots go through the place as through a point: that place is inside the band, and the rules' dodging of it
/// carried the run down to the floor and along it into the band, hit at `71.5,410.8` 407 frames in on every attempt
/// from the chapter a whole Extra kept. **Nor low in a corner**: the band at the floor is `-164` to `138`; from `80` aside
/// and `52` off the floor, and from `120` aside and `24` off it, the run was taken by the next throw on its way
/// back and carried into the bottom-right corner (`171.4,432.0` 871 frames in, `184.0,432.0` 1,099 in).
const TWO_BOWS_GOES_ACROSS: f32 = 112.0;
const TWO_BOWS_GOES_DOWN_BY: f32 = 16.0;
/// **Where the run goes to as a throw led to the middle comes**: `56` to the side of the middle and `32` below the
/// place — the place that took keep\at-99's and keep\at-100b's card with no hit, where `112` aside and `16` down was
/// hit on both (see [`two_bows_two_claps_one_bow`]).
const TWO_BOWS_GOES_ACROSS_FROM_THE_MIDDLE: f32 = 56.0;
const TWO_BOWS_GOES_DOWN_FROM_THE_MIDDLE: f32 = 32.0;
/// **How far in from a wall the places led to and gone to are held**: some three of the rules' whole steps of `4.5`
/// — a place against the wall is one the rules' dodging has no room to give way from.
const TWO_BOWS_OFF_THE_WALLS: f32 = 16.0;
/// **How far in from a wall [`Led::ByTheSpokes`] holds its places**, further than [`TWO_BOWS_OFF_THE_WALLS`]: where
/// the spokes close some `28°` off straight down, the place lured to `30°` round on the wall's side is shortened to
/// the wall, and the fire led there came down along it — the run was carried up and down the left wall through the
/// whole card and hit at `-184.0,355.9` 2,016 frames in from keep\at-100 and at `-153.3,432.0` 1,591 in from
/// keep\hits-u31\s7-c11-f14514, both closing on `118°` (`c100-ways.sh`, 2026-10-04); held so, both took the card.
///
/// **And not led to the middle on a first attempt where the spokes close so far off**, which was tried for closes more
/// than `25°` off straight down, the run going aside of the middle away from the close: keep\at-99 was hit at
/// `-162.3,432.0` 629 frames in and keep\at-100 at `-184.0,432.0` 1,376 in, both of which the spokes' way held off
/// the wall takes (c100-far.sh, 2026-10-04).
const TWO_BOWS_LURED_OFF_THE_WALLS: f32 = 64.0;

/// **Which side the run goes down to as a throw comes where no close was read, `-1` for the left: the one the
/// spokes are sweeping away from**, out of a spoke's youngest laser and the one let go before it.
fn the_side_the_spokes_sweep_away_from(frame: &Frame) -> f32 {
    let sweeping: f32 = frame
        .beams
        .iter()
        .filter_map(|beam| {
            let before = frame.beams.iter().find(|other| {
                other.frames > beam.frames
                    && other.frames <= beam.frames + TWO_BOWS_A_SPOKE_LETS_GO_EVERY
                    && (other.angle - beam.angle).abs() < TWO_BOWS_ONE_SPOKE_WITHIN
            })?;
            Some(-beam.angle.sin() * (beam.angle - before.angle))
        })
        .sum();
    if sweeping > 0.0 { -1.0 } else { 1.0 }
}
/// **How often a spoke lets a laser go**, in frames: its lasers' ages go `1, 5, 9, …`.
const TWO_BOWS_A_SPOKE_LETS_GO_EVERY: i32 = 4;
/// **How near in angle two lasers are to be one spoke's**: a spoke turns `2.25°` a laser.
const TWO_BOWS_ONE_SPOKE_WITHIN: f32 = 0.2;
static TWO_BOWS: Mutex<TwoBows> = Mutex::new(TwoBows::NONE);
/// **The age at which every bullet of the card's fire is aimed at where the run is** — see
/// [`two_bows_two_claps_one_bow`], whose measurement it is.
const TWO_BOWS_AIMED_AT: i32 = 50;
/// **How many frames apart a throw's two lots take aim**, as they are born (card frames 150 and 180, …).
const TWO_BOWS_LOTS_APART: i32 = 30;
/// **How far below the place led to a lot is through it**: a big one's `14` half across and some — a lot reaches
/// that height 165 to 185 frames after it takes aim, some 30 before the next throw does (`through100.py`).
const TWO_BOWS_GONE_THROUGH_BELOW: f32 = 24.0;
/// **How far above the floor the places the fire is led to are**: `320` on a field whose floor is `432` — the
/// strategy wiki's `画面下に居すぎると弾がばらけて危険` (wikiwiki.jp/thk, 風/EX): the fire led through a place goes
/// on past it and spreads, and every place at the floor this card was tried at lost it, `16` and `64` up alike.
const TWO_BOWS_ABOVE_THE_FLOOR: f32 = 112.0;

/// **Where [`Led::ByTheSpokes`] has the run, as the angle off the spokes' close about the boss in degrees and the
/// distance out from it, for the card frames since its first laser** — the spokes keep a schedule of their own,
/// the same from every chapter (`spokes100.py`, keep\hits-u1-extra-clear-pd-1\s7-c11-f14047 and keep\chain100,
/// 2026-10-03): every [`TWO_BOWS_A_PERIOD`] frames from the first laser a pair sweeping `2.25°` a laser in to the
/// close from either side, the second of them 30 frames after the first and each stopping `2.25°` short of the
/// close, then a pair `2.0°` a laser from 120 frames, both at once and stopping `12°` short; and the side that
/// sweeps first changes every period. The fire is thrown on the period's frames 58 and 88 and takes aim 50 after.
///
/// **So the fire is led on the side whose spoke comes second**, [`TWO_BOWS_LURED_OFF_THE_CLOSE`] round and
/// [`TWO_BOWS_LURED_OUT`] out, where neither spoke reaches before both lots have taken aim — the strategy wiki's
/// `次の青レーザーの方向に誘導`; **and the run waits [`TWO_BOWS_WAITS_OFF_THE_CLOSE`] round on the other side while
/// the lots come**, past the close and inside the slower pair's `12°`, which is where the next period's fire is led
/// from: the fire comes down on the side it was led to, and the run is on the other. **And not in the close
/// itself**, which no laser crosses either: stood there it has from nothing to `4` of room through a whole card on
/// either side of the lasers stopping short of it (`hole100.py`), which the rules' dodging reads as nowhere open.
fn where_the_spokes_leave_room(since_the_first_laser: i32) -> (f32, f32) {
    let lured = |period: i32| {
        let side = if period % 2 == 0 { 1.0 } else { -1.0 };
        (side * TWO_BOWS_LURED_OFF_THE_CLOSE, TWO_BOWS_LURED_OUT)
    };
    if since_the_first_laser < TWO_BOWS_LURED_UNTIL {
        return lured(0);
    }
    let since = since_the_first_laser - TWO_BOWS_LURED_UNTIL;
    let period = since / TWO_BOWS_A_PERIOD;
    match since % TWO_BOWS_A_PERIOD
        < TWO_BOWS_A_PERIOD - (TWO_BOWS_LURED_UNTIL - TWO_BOWS_LURED_FROM)
    {
        true => {
            let (off, _) = lured(period);
            (
                -off.signum() * TWO_BOWS_WAITS_OFF_THE_CLOSE,
                TWO_BOWS_WAITS_OUT,
            )
        }
        false => lured(period + 1),
    }
}
/// **How many frames apart the spokes' periods are**, the same as the fire's throws.
const TWO_BOWS_A_PERIOD: i32 = 240;
/// **From which of a period's frames the fire is led** — past the slower pair's last laser on that side, which
/// lies across the way there until the period's frame 84 at the distance led to (`chart100.py` at `212`).
const TWO_BOWS_LURED_FROM: i32 = 84;
/// **Until which of a period's frames the fire is led**: the spoke that comes second reaches the place
/// [`TWO_BOWS_LURED_OFF_THE_CLOSE`] round at the distance led to some 178 frames into the period.
const TWO_BOWS_LURED_UNTIL: i32 = 170;
const TWO_BOWS_LURED_OFF_THE_CLOSE: f32 = 30.0;
const TWO_BOWS_LURED_OUT: f32 = 212.0;
/// **How far round past the close and how far out the run waits**: `7°` and `160`, inside the slower pair's `12°`
/// and clear of the faster pair's last lasers by some `2` to `5` stood still through a whole wait (`hole100.py`),
/// which took seven of the eight keeps of the card's start (`bench100.sh`, 2026-10-03).
///
/// **And not `250` out**, below the height the fire was led to, which took six: stood still there it has less than
/// nothing of room on every period, and where the close is `30°` off straight down — `120°` from
/// keep\hits-u3-extra-clear-pd-1\s7-c11-f14536, `62°` from keep\hits-xpd3\s7-c11-f15741 — the place is against the
/// wall, where the lots aimed at the place led to come down along it: hit at `-102.4,432.0` 1,602 frames in and
/// `184.0,389.8` 1,305 in. The boss moving some `18` towards the run on each period's frame 180
/// (`bossmoves100.py`) takes it to `±75` on such a card, and the one keep `160` lost, keep\chain100c (`121°`), was
/// lost that way, at `-100.0,432.0` 2,080 frames in.
const TWO_BOWS_WAITS_OFF_THE_CLOSE: f32 = 7.0;
const TWO_BOWS_WAITS_OUT: f32 = 160.0;

/// **Where th10's `神符「杉で結ぶ古き縁」` is waited out**: standing in the middle of the field across, a tap of
/// [`CEDARS_TAP`] up or down from where the run is each time the card's fire is aimed at it, and turning back
/// between [`CEDARS_HIGHEST`] and [`CEDARS_LOWEST`] above the floor.
///
/// **A tap and not a run**, the way aimed fire is dodged (`ちょん避け`): the run stands still while the fire is
/// on its way to being aimed, so that it is aimed at one place, and steps off that place once it has been — the
/// wiki's own with a bomb in hand is `正面一番下から開始して上にちょん避けを繰り返し`. Kept going instead, the
/// place wanted two dozen ahead of the run every frame, the run was hit at `-180.8,432` 1,953 frames into the
/// card from keep\hits-xpd3\s7-c6-f7513, having been carried wall to wall.
///
/// **The card is fire aimed at the run in two goes, green and then white**, and the strategy wiki's way through it
/// (wikiwiki.jp/thk, 風/EX) is `X軸中央で上下に大きく避ける`, turning back only once all of it has turned white and
/// never far: each go is aimed at where the run stood, so a run that keeps going has it come down behind it. The
/// rules alone were carried along the floor into a corner on three attempts of four from keep\at-98
/// (`184.0,432` at 846 frames into the card, `-180.8,425.6` at 1,127, `-177.6,432` at 1,892) and once in a whole
/// Extra (`184.0,425.6` at 757).
///
/// **So the end of the band is waited at and not turned back from**, until no bullet of the card is still on its
/// way to being aimed the second time — which is what the bullet's own list of changes says, see
/// [`CEDARS_AIMED_AGAIN_AT`]. Turned back at each end whatever was coming, the run was hit three times from
/// keep\hits-xpd3\s7-c6-f7513 (`-173.1,432` 544 frames into the card, `-164.1,432` at 1,227, `-179.5,415.8` at
/// 1,806), every one of them having been carried to a wall and down it into a corner.
///
/// **And the place the run stands at is taken from where the run is, at each tap, and not kept apart from it.**
/// Kept as a place of its own going up and down the band, it went on to the far end while the run was held off:
/// the run at `-10,365` wanted `0,150` with four of the nine clear, every one of them sideways, and took them
/// along the floor into the corner.
pub(crate) fn old_ties_bound_by_cedars(frame: &Frame, card_frames: Option<i32>) -> (f32, f32) {
    let field = frame.field;
    let (top, bottom) = cedars_band(field);
    let mut cedars = CEDARS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let begun_again = match &*cedars {
        None => true,
        // A retry of its chapter starts the card over, going up from where the run is.
        Some(kept) => matches!((kept.on, card_frames), (Some(was), Some(now)) if now < was),
    };
    if begun_again {
        *cedars = Some(Cedars {
            on: card_frames,
            downwards: false,
            stand: frame.at.1.clamp(top, bottom),
            tapped_on: None,
            aimed: Vec::new(),
        });
    }
    let Some(cedars) = cedars.as_mut() else {
        unreachable!("set just above")
    };
    if cedars.on != card_frames {
        cedars.on = card_frames;
        let aimed: Vec<usize> = frame
            .hazards
            .iter()
            .filter(|hazard| {
                matches!(
                    hazard.kind,
                    Kind::Bullet { hitting: true, next_change: Some(change), .. } if change >= CEDARS_AIMED_AT
                )
            })
            .map(|hazard| hazard.slot)
            .collect();
        let newly_aimed = aimed.iter().any(|slot| !cedars.aimed.contains(slot));
        let tapped_lately = matches!(
            (cedars.tapped_on, card_frames),
            (Some(then), Some(now)) if now - then < CEDARS_TAP_EVERY
        );
        if newly_aimed && !tapped_lately {
            let all_aimed_again = !frame.hazards.iter().any(|hazard| {
                matches!(
                    hazard.kind,
                    Kind::Bullet { hitting: true, next_change: Some(change), .. }
                        if (CEDARS_THROWN_AT..CEDARS_AIMED_AGAIN_AT).contains(&change)
                )
            });
            let y = frame.at.1;
            if all_aimed_again {
                if y + CEDARS_TAP > bottom {
                    cedars.downwards = false;
                } else if y - CEDARS_TAP < top {
                    cedars.downwards = true;
                }
            }
            let tap = if cedars.downwards {
                CEDARS_TAP
            } else {
                -CEDARS_TAP
            };
            cedars.stand = (y + tap).clamp(top, bottom);
            cedars.tapped_on = card_frames;
        }
        cedars.aimed = aimed;
    }
    (field.0 + field.2 / 2.0, cedars.stand)
}

/// **The band [`old_ties_bound_by_cedars`] taps up and down in**, its top and its bottom, which the rules keep the
/// run inside — see `crate::steer::keeps_to_the_middle`.
pub(crate) fn cedars_band(field: (f32, f32, f32, f32)) -> (f32, f32) {
    let floor = field.1 + field.3;
    (floor - CEDARS_HIGHEST, floor - CEDARS_LOWEST)
}

/// **Where [`old_ties_bound_by_cedars`] has the run standing**, which way its taps go, the card's frame it was last
/// asked on and last tapped on, and the slots of the bullets aimed at the run on that frame.
struct Cedars {
    on: Option<i32>,
    downwards: bool,
    stand: f32,
    tapped_on: Option<i32>,
    aimed: Vec<usize>,
}
static CEDARS: Mutex<Option<Cedars>> = Mutex::new(None);
/// **How far one tap goes**: past the bullet's own box, `4` across, and the run's, `2`, several times over, so
/// that fire aimed at where the run stood goes by it. Not yet measured against the gaps of the card's rings.
const CEDARS_TAP: f32 = 16.0;
/// **How many of the card's frames one tap stands for**: the bullets of a ring are aimed over a few frames and not
/// on one, and each of those frames is the same go.
const CEDARS_TAP_EVERY: i32 = 10;
/// **The change at which the card's fire is aimed at the run**, the first time.
const CEDARS_AIMED_AT: u32 = 4;
/// **How far above the floor the band goes up to and down to**: `320` to `408` on th10's field, whose floor is
/// `432` — taps in the lower middle, off the floor and well under the boss. Down to `12` above it, the run tapped
/// its way onto the floor in the middle and was held there, hit at `-1.9,432` 907 frames into the card from
/// keep\at-98 on all eight attempts, with nothing open to it below; up to `166`, it was taken up past it towards
/// the boss and hit at `1.6,147.9` 1,273 frames in on all five; and up to `234`, the fire shut every move of the
/// middle at the top of the band and the run went up out of it, hit at `-1.4,148.1` 701 frames in on fourteen
/// attempts running (keep\hits-xpd5\s7-c6-f7913).
const CEDARS_HIGHEST: f32 = 112.0;
const CEDARS_LOWEST: f32 = 24.0;
/// **The changes the card's aimed fire goes through**: thrown out from the boss at `2`, aimed at the run at `4`,
/// and aimed at it again from where it is at `5` — the green and the white of the wiki — read off the bullets
/// nearest the run with `--trace-the-world` on 2026-10-02. The card's other fire, sprite `0x1d`, stays at `1`
/// and is no part of the two goes.
const CEDARS_THROWN_AT: u32 = 2;
const CEDARS_AIMED_AGAIN_AT: u32 = 5;

/// **Where th10's `源符「厭い川の翡翠」` has the run go**: on at whichever steady pace — standing, or one of
/// [`RIVER_PACES`] along any of sixteen headings — the card's own fire played on leaves unhit longest over
/// [`RIVER_LOOKS_AHEAD`] frames, and of those the one it leaves most room to and nearest the place the rivers
/// cross; chosen again from where the run is every [`A_PACE_IS_CHOSEN_EVERY`] frames of the card.
///
/// **The card's fire is the same every time but for where it is aimed**, read off the field of one launch
/// (`--trace-the-field-in=103` from keep\at-103, 2026-10-02): three rivers in turn — the first `57..162` across at
/// the field's top and `-114..-9` at its floor, the second `-68..68` straight down, the third the first's mirror —
/// each laid from the top down two bullets every four frames, rows `8` apart, from frame 80 of the card and every
/// 300 after. Each bullet comes in level from a wall at a pace of its own for [`RIVER_LAID_AT_FULL_PACE_FOR`]
/// frames, slows to rest over [`RIVER_LAID_SLOWING_OVER`], and sets off at [`RIVER_SETS_OFF_AT`] frames old — so a
/// river is let go from the top down at the pace it was laid — at [`RIVER_LETS_GO_AT`] a frame straight at where
/// the run is on that frame, never turning. A model of that, played with the launch's own path for the run,
/// matched the field to `0.00` on every frame.
///
/// **So where the run is as each bullet sets off is all the card's fire depends on**, and what looking ahead in
/// the card's own rules is up against is that every way of the run's own shape was hit in the model. A run going
/// steadily on is never hit by fire aimed at it, which crosses its line only where it was; what hits is standing,
/// turning back, and being near a bank as it sets off. A run that went back to one place between lettings go
/// was hit there; one that went down a river while it was let go and up the next was hit as it turned, by bullets
/// set off below it and chasing it up; one going straight down the middle was hit within `14` of a bank as its row
/// set off; and a path searched for over the whole card was hit least on 4 frames, and on 28 as the run's own steps
/// would play it. The way before this one held a height `60` under the field's top and went across the river
/// there, which is inside no two of the rivers: the next one's laying pushed the run down onto the boss with
/// `0 of the nine clear`, hit at `0.6,91.9` and `-46.9,73.5` 705 and 722 frames into the card from
/// keep\hits-xpd2\s7-c17-f24245. Looking ahead the way this does, the model's run went through all 1,885 frames of
/// the card unhit, the nearest bullet `3.3` off as its steps would play it.
///
/// **And with the crossing moved aside by a quarter of `aside`**, the attempt's own draw — see
/// `crate::steer::aside_on`: a way that looks ahead plays the card the same way on every retry of it, and a quarter
/// of the draw's `96` at most keeps the run inside the rivers' middles.
pub(crate) fn jade_of_the_loathsome_river(
    frame: &Frame,
    card_frames: Option<i32>,
    aside: f32,
) -> (f32, f32) {
    looked_ahead(frame, card_frames, aside, &RIVER, &THE_RIVER)
}

/// **Where a card whose whole fire can be played on has the run go**: see [`jade_of_the_loathsome_river`] and
/// [`the_red_frog_of_hoei_four`], each of which this is with the card's own [`LookedAhead`].
fn looked_ahead(
    frame: &Frame,
    card_frames: Option<i32>,
    aside: f32,
    kept: &Mutex<River>,
    card: &LookedAhead,
) -> (f32, f32) {
    let Some(now) = card_frames else {
        return frame.at;
    };
    let field = frame.field;
    let mut river = kept
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A retry of its chapter starts the card over.
    if river.chosen_on.is_some_and(|was| now < was) {
        *river = River::NONE;
    }
    // **Moved on once a card's frame and not once a call**, the way being asked more than once a frame.
    if river.on != Some(now) {
        river.on = Some(now);
        river.bodies_before = std::mem::take(&mut river.bodies);
        river.bodies = frame
            .hazards
            .iter()
            .filter(|hazard| matches!(hazard.kind, Kind::Body { .. }))
            .map(|hazard| (hazard.slot, hazard.at))
            .collect();
        river.ran.push((now, frame.at));
        if river.ran.len() > THE_RUN_IS_KEPT_FOR {
            river.ran.remove(0);
        }
        if river
            .chosen_on
            .is_none_or(|was| now - was >= A_PACE_IS_CHOSEN_EVERY)
        {
            river.chosen_on = Some(now);
            river.place = frame.at;
            let pace = {
                let ran = &river.ran;
                let run_was = |age: i32| {
                    let then = now - age - SPLEEN_EATERS_RING_CAME_BEFORE_ITS_RICE_BY;
                    ran.iter()
                        .find(|(on, _)| *on >= then)
                        .map_or(frame.at, |(_, at)| *at)
                };
                the_pace_through(frame, aside, card, &river.bodies_before, &run_was, now)
            };
            river.pace = pace;
            let pace = river.pace.0.hypot(river.pace.1);
            let focused = frame.focused_straight.0.max(frame.focused_straight.1);
            THE_WHOLE_STEP.store(pace > focused + 0.01, Ordering::Relaxed);
        }
        river.place = (
            (river.place.0 + river.pace.0).clamp(field.0, field.0 + field.2),
            (river.place.1 + river.pace.1).clamp(field.1, field.1 + field.3),
        );
    }
    river.place
}

/// **Whether the pace [`looked_ahead`] last chose is past the focused step**, which the rules then go at with the
/// focus key let go: a place moved on by `4.5` a frame is one the focused `2.0` falls behind.
pub(crate) fn a_pace_looked_ahead_wants_the_whole_step() -> bool {
    THE_WHOLE_STEP.load(Ordering::Relaxed)
}

static THE_WHOLE_STEP: AtomicBool = AtomicBool::new(false);

/// **What [`looked_ahead`] plays a card on by**: how its bullets at rest set off, the paces tried and how far
/// ahead, and where the run would rather be — how far above the floor at the field's middle across, how far from
/// there a pace may end for each unit of room it is left more, and how much room the field's edge is for each unit
/// the run is from it.
#[derive(Clone, Copy)]
struct LookedAhead {
    sets_off: SetsOff,
    paces: &'static [f32],
    looks_ahead: i32,
    /// How many frames unhit a pace is no better for more of, the room it is left then choosing between them.
    unhit_enough: i32,
    /// After how many frames a pace is turned to every other once, and of how many of the best paces, where no one
    /// pace is unhit throughout — or never.
    turns_once: Option<(i32, usize)>,
    home: Home,
    home_is_worth: f32,
    an_edge_is_worth: f32,
    keeps_a_box_wider_by: f32,
    aside_share: f32,
}

/// **Where a card looked ahead through has the run rather be.**
#[derive(Clone, Copy)]
enum Home {
    /// So far above the floor, at the field's middle across.
    AboveTheFloor(f32),
    /// Wherever on the field the rice at rest is fewest and youngest — see [`where_the_rice_is_youngest`].
    WhereTheRiceIsYoungest,
    /// The middle of the rice at rest, younger than it is when it sets off — see [`among_the_rice_at_rest`]; and so far
    /// above the floor at the field's middle across with none at rest.
    AmongTheRiceAtRest { sets_off_at: i32, above: f32 },
}

/// **How a card's bullets at rest set off**: straight at the run, each its own way, or speeding up straight at where
/// the run was as the ring that laid them came.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SetsOff {
    AtTheRun,
    AnyWay,
    AtWhereTheRunWas,
}

const THE_RIVER: LookedAhead = LookedAhead {
    sets_off: SetsOff::AtTheRun,
    paces: &RIVER_PACES,
    looks_ahead: RIVER_LOOKS_AHEAD,
    unhit_enough: RIVER_LOOKS_AHEAD + 1,
    turns_once: None,
    home: Home::AboveTheFloor(RIVER_HOME_ABOVE_THE_FLOOR),
    home_is_worth: RIVER_HOME_IS_WORTH,
    an_edge_is_worth: RIVER_AN_EDGE_IS_WORTH,
    keeps_a_box_wider_by: RIVER_KEEPS_A_BOX_WIDER_BY,
    aside_share: 0.25,
};

/// **The pace [`looked_ahead`] goes on at from where the run is**, every pace tried against the card's fire
/// played on — a body's step taken from where it was the frame before, in `bodies_before` — standing still among
/// them.
fn the_pace_through(
    frame: &Frame,
    aside: f32,
    card: &LookedAhead,
    bodies_before: &[(usize, (f32, f32))],
    run_was: &dyn Fn(i32) -> (f32, f32),
    now: i32,
) -> (f32, f32) {
    let field = frame.field;
    let home = match card.home {
        Home::AboveTheFloor(above) => (field.0 + field.2 / 2.0, field.1 + field.3 - above),
        Home::WhereTheRiceIsYoungest => where_the_rice_is_youngest(frame),
        Home::AmongTheRiceAtRest { sets_off_at, above } => {
            among_the_rice_at_rest(frame, sets_off_at)
                .unwrap_or((field.0 + field.2 / 2.0, field.1 + field.3 - above))
        }
    };
    let home = (
        (home.0 + aside * card.aside_share).clamp(field.0, field.0 + field.2),
        home.1,
    );
    // The run's own box, kept wider by the card's margin.
    let mine = (
        frame.half.0 + card.keeps_a_box_wider_by,
        frame.half.1 + card.keeps_a_box_wider_by,
    );
    let fire: Vec<Fire> = frame
        .hazards
        .iter()
        .filter_map(|hazard| {
            let moved = bodies_before
                .iter()
                .find(|(slot, _)| *slot == hazard.slot)
                .map(|(_, was)| (hazard.at.0 - was.0, hazard.at.1 - was.1));
            // **A double of `「宝永四年の赤蛙」` standing is played on as standing, and not as setting off for the run**,
            // though that is what one standing off the field does next: it goes at `15` a frame along the line through
            // where the run is as it sets off, laying its rice the whole way (keep\hits-u6-extra-clear-pd-1\
            // s7-c25-f32992, `doubles-near.py`). Played on as setting off that frame, every way of telling which double
            // stands for that — off the field, anywhere, more than `100` from the run, and that with the edge worth
            // `1.0` — kept the run off their lines and walked it to the field's edge late in the card, where it was shut
            // in: 2 of 6, 1 of 4 and 2 of 5 of the card's kept chapters through, against 3 of the same five or six
            // played on as standing (c107e to c107g, 2026-10-03).
            Fire::of(hazard, mine, card.sets_off, moved, run_was)
        })
        .chain(the_spleen_eaters_burst_to_come(frame, card, mine, now))
        .collect();
    let paces = std::iter::once((0.0, 0.0)).chain((0..16).flat_map(|heading| {
        let angle = heading as f32 * std::f32::consts::PI / 8.0;
        card.paces
            .iter()
            .map(move |pace| (pace * angle.cos(), pace * angle.sin()))
    }));
    let paces: Vec<(f32, f32)> = paces.collect();
    let mut played = Vec::with_capacity(fire.len());
    let mut best = ((0.0, 0.0), -1, f32::MIN);
    let mut each = Vec::with_capacity(paces.len());
    for &pace in &paces {
        played.clear();
        played.extend_from_slice(&fire);
        let (unhit, room) = played_on(&mut played, frame.at, (pace, None), field, home, mine, card);
        let unhit = unhit.min(card.unhit_enough);
        each.push((pace, unhit, room));
        if unhit > best.1 || (unhit == best.1 && room > best.2) {
            best = (pace, unhit, room);
        }
    }
    // **And each of the best few turned once part way to every other pace**, where no one pace is unhit throughout:
    // the pace chosen is the first of the best of those, chosen again a few frames on.
    if let Some((on, few)) = card.turns_once
        && best.1 <= card.looks_ahead
    {
        each.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.total_cmp(&a.2)));
        for &(pace, unhit, _) in each.iter().take(few) {
            if unhit <= on {
                continue;
            }
            for &turned in &paces {
                played.clear();
                played.extend_from_slice(&fire);
                let (unhit, room) = played_on(
                    &mut played,
                    frame.at,
                    (pace, Some((on, turned))),
                    field,
                    home,
                    mine,
                    card,
                );
                let unhit = unhit.min(card.unhit_enough);
                if unhit > best.1 || (unhit == best.1 && room > best.2) {
                    best = (pace, unhit, room);
                }
            }
        }
    }
    detail!(
        "autoplay: looked ahead through {} of the card's fire, the pace {:.2},{:.2} unhit for {} of {} frames, {:.1} of room",
        fire.len(),
        best.0.0,
        best.0.1,
        best.1,
        card.looks_ahead,
        best.2,
    );
    best.0
}

/// **How many frames a run going on at `pace` from `from` goes unhit by the card's fire**, the card's frames
/// looked ahead and one for all of them, and the room it is left: the least of it on the way, at most
/// [`ROOM_ENOUGH`], less how far from `home` it ends — or, for a run that is hit, how far into the fire.
/// A pace across and down, and the frame it gives way to another and that one.
type PaceAndThen = ((f32, f32), Option<(i32, (f32, f32))>);

fn played_on(
    fire: &mut Vec<Fire>,
    from: (f32, f32),
    (pace, then): PaceAndThen,
    field: (f32, f32, f32, f32),
    home: (f32, f32),
    mine: (f32, f32),
    card: &LookedAhead,
) -> (i32, f32) {
    let mut at = from;
    let mut nearest = ROOM_ENOUGH;
    let mut laid = Vec::new();
    let mut chance = 0.0;
    for ahead in 1..=card.looks_ahead {
        // The pace turned to, from the frame it is turned to on.
        let pace = match then {
            Some((on, turned)) if ahead > on => turned,
            _ => pace,
        };
        at = (
            (at.0 + pace.0).clamp(field.0, field.0 + field.2),
            (at.1 + pace.1).clamp(field.1, field.1 + field.3),
        );
        let mut room = f32::MAX;
        for bullet in fire.iter_mut() {
            bullet.goes_on(at);
            room = room.min(bullet.room_from(at));
            chance += bullet.chance_on(at);
            // **Laid where the double is, though a grain is born off it** — `2.17` at the median and `3.71` nine
            // times in ten (`lay-spread2.py` over keep\hits-u6-extra-clear-pd-1\s7-c25-f32992's field). Kept `3.7`
            // wider for it, the run went through 1 of the card's first four kept chapters against 2 laid here, the
            // chapter it was measured on hit 2,200 frames in and not 1,666 (c107h, 2026-10-03).
            if matches!(bullet.moves, Moves::Laying(_)) {
                laid.push(Fire {
                    at: bullet.at,
                    half: (A_GRAINS_HALF + mine.0, A_GRAINS_HALF + mine.1),
                    moves: Moves::AtRest(RED_FROG_BURSTS_AT, card.sets_off),
                });
            }
        }
        fire.append(&mut laid);
        if room < 0.0 {
            return (ahead, room - chance * A_HIT_IS_WORTH);
        }
        // **The edge as its least on every frame of the pace, and not only where the pace ends**, which was tried
        // for card 107 so that of two paces from beside a wall the one going away from it would be the better: over
        // seven keeps of the card (`benchcard.sh`, 2026-10-03) it took four as this does, through from
        // keep\hits-u4-extra-clear-pd-1\s7-c25-f33668 and hit from keep\hits-xpd6\s7-c25-f33026 at `178.9,234.3`
        // 1,981 frames in, which this takes.
        let edge = (at.0 - field.0)
            .min(field.0 + field.2 - at.0)
            .min(at.1 - field.1)
            .min(field.1 + field.3 - at.1);
        nearest = nearest.min(room).min(edge * card.an_edge_is_worth);
    }
    (
        card.looks_ahead + 1,
        nearest - distance(at, home) / card.home_is_worth - chance * A_HIT_IS_WORTH,
    )
}

/// **How much room a pace gives up for each hit it may take**, counted in chances: a sure one is worth eight times the
/// most room a pace is credited with. At four times, the run went into the rice bursting about the boss rather than
/// stay over a double's trail, and was hit there at `83.1,240.2` 621 frames into the card (keep\hits-xpd11).
const A_HIT_IS_WORTH: f32 = 8.0 * ROOM_ENOUGH;

/// **One thing of the card's that hits, as [`played_on`] plays it on**: where it is, its box grown by the run's
/// own kept wider by the card's margin, and how it moves.
#[derive(Clone, Copy)]
struct Fire {
    at: (f32, f32),
    half: (f32, f32),
    moves: Moves,
}

#[derive(Clone, Copy)]
enum Moves {
    /// On by the same step every frame: a bullet let go, a body, and anything else.
    Straight((f32, f32)),
    /// A bullet at rest, the frames until it sets off and how.
    AtRest(i32, SetsOff),
    /// A bullet being laid: its heading, the pace it came in at, and its age.
    Laid((f32, f32), f32, i32),
    /// A grain of rice gone off its own way, which way being unknown: how far it has gone from where it stood, and
    /// the frames since it went.
    Bursting(f32, i32),
    /// A grain of rice gone off a way it is seen going: its heading, and the frames since it went.
    Slowing((f32, f32), i32),
    /// A grain at rest that sets off at a place: the frames until it does, that place, and how much faster it goes each
    /// frame once it has.
    AtRestFor(i32, (f32, f32), f32),
    /// A grain set off and speeding up: its heading, its pace, and how much faster each frame.
    Speeding((f32, f32), f32, f32),
    /// A body on by the same step every frame and its reach growing by so much: a spirit and its burst.
    Growing((f32, f32), f32),
    /// A burst to come where it is: the frames until it does, and the frames it lasts.
    BurstsIn(i32, i32),
    /// A double that cannot touch the run and lays a grain of rice where it is every frame, on by its step.
    Laying((f32, f32)),
    /// Off the field.
    Gone,
}

impl Fire {
    /// The fire a hazard is, with `moved` how far it went since the frame before, for a hazard handed over with no
    /// step of its own.
    fn of(
        hazard: &Hazard,
        mine: (f32, f32),
        sets_off: SetsOff,
        moved: Option<(f32, f32)>,
        run_was: &dyn Fn(i32) -> (f32, f32),
    ) -> Option<Fire> {
        let step = hazard.step.or(moved).unwrap_or((0.0, 0.0));
        let moves = match hazard.kind {
            Kind::Bullet { .. } => {
                let pace = step.0.hypot(step.1);
                let at_rest_until = match sets_off {
                    SetsOff::AtTheRun => RIVER_SETS_OFF_AT,
                    SetsOff::AnyWay => RED_FROG_BURSTS_AT,
                    SetsOff::AtWhereTheRunWas => SPLEEN_EATERS_RICE_SETS_OFF_AT,
                };
                let age = hazard.frames.unwrap_or(at_rest_until);
                let heading = (
                    step.0 / pace.max(f32::EPSILON),
                    step.1 / pace.max(f32::EPSILON),
                );
                if sets_off == SetsOff::AtWhereTheRunWas
                    && pace < AT_REST_UNDER
                    && age < at_rest_until
                {
                    Moves::AtRestFor(
                        at_rest_until - age,
                        run_was(age),
                        SPLEEN_EATERS_RICE_SPEEDS_UP_BY,
                    )
                } else if sets_off == SetsOff::AtWhereTheRunWas
                    && (AT_REST_UNDER..SPLEEN_EATERS_RICE_GOES_AT_MOST).contains(&pace)
                    && age < at_rest_until + SPLEEN_EATERS_RICE_SPEEDS_UP_OVER
                {
                    // **And no faster than the rice goes**, a bullet past that being one of the card's others: played
                    // on as rice speeding up, the `5.0` a frame one that came down at the run was slowed to `1.0` and
                    // the run hit at `-62.8,401.7` five times on `hits5/s5-c7-f14293` (2026-10-09).
                    Moves::Speeding(heading, pace, SPLEEN_EATERS_RICE_SPEEDS_UP_BY)
                } else if pace < AT_REST_UNDER && age < at_rest_until {
                    Moves::AtRest(at_rest_until - age, sets_off)
                } else if sets_off == SetsOff::AnyWay
                    && pace >= AT_REST_UNDER
                    && (RED_FROG_BURSTS_AT..RED_FROG_BURSTS_AT + A_BURST_LASTS).contains(&age)
                {
                    Moves::Slowing((step.0 / pace, step.1 / pace), age - RED_FROG_BURSTS_AT)
                } else if sets_off == SetsOff::AtTheRun
                    && pace >= AT_REST_UNDER
                    && step.1.abs() < RIVER_LAID_LEVEL
                    && age < RIVER_LAID_AT_FULL_PACE_FOR + RIVER_LAID_SLOWING_OVER
                {
                    let slowed = (age - RIVER_LAID_AT_FULL_PACE_FOR).max(0) as f32;
                    let came_in_at = pace / (1.0 - slowed / RIVER_LAID_SLOWING_OVER as f32);
                    Moves::Laid((step.0 / pace, step.1 / pace), came_in_at, age)
                } else {
                    Moves::Straight(step)
                }
            }
            // **A double that cannot touch the run, for the card whose doubles lay their rice where they are** —
            // see [`the_red_frog_of_hoei_four`].
            Kind::Body { solid: false } if sets_off == SetsOff::AnyWay => Moves::Laying(step),
            // **The spirits of `恨霊「スプリーンイーター」` going round and in, and their burst growing**, which the wiki has
            // the run keep off — `急いで抜けようとして怨霊に当たらないように注意`. Their burst is played on ahead of its
            // growing too — see [`the_spleen_eaters_burst_to_come`].
            Kind::Body { solid: true } if sets_off == SetsOff::AtWhereTheRunWas => {
                Moves::Growing(step, hazard.grows.max(0.0))
            }
            Kind::Body { solid: false } => return None,
            Kind::Body { solid: true } | Kind::Trail => Moves::Straight(step),
        };
        Some(Fire {
            at: hazard.at,
            half: (hazard.half.0 + mine.0, hazard.half.1 + mine.1),
            moves,
        })
    }

    /// One frame on, with the run at `run`.
    fn goes_on(&mut self, run: (f32, f32)) {
        match &mut self.moves {
            Moves::Straight(step) => {
                self.at = (self.at.0 + step.0, self.at.1 + step.1);
            }
            // **On straight at its pace, though a double slows by `0.965` a frame** (`15.5`, `15.0`, `14.4`, `13.9`,
            // `13.4` flying off in the traced field): played on slowing so, the run was hit 0, 1 and 2 times from
            // keep\at-107, keep\hits-xpd11\s7-c25-f31504 and keep\hits-xpd10\s7-c25-f31841, against 1, 0 and 0
            // played on straight — a double's whole line kept off and not only the stretch of it it slows over.
            Moves::Laying(step) => {
                self.at = (self.at.0 + step.0, self.at.1 + step.1);
            }
            Moves::AtRest(frames, sets_off) => {
                *frames -= 1;
                if *frames > 0 {
                    return;
                }
                self.moves = match sets_off {
                    SetsOff::AtTheRun => {
                        let (dx, dy) = (run.0 - self.at.0, run.1 - self.at.1);
                        let away = dx.hypot(dy).max(f32::EPSILON);
                        let step = (dx / away * RIVER_LETS_GO_AT, dy / away * RIVER_LETS_GO_AT);
                        self.at = (self.at.0 + step.0, self.at.1 + step.1);
                        Moves::Straight(step)
                    }
                    SetsOff::AnyWay => Moves::Bursting(a_bursts_pace(0), 0),
                    SetsOff::AtWhereTheRunWas => Moves::Gone,
                };
            }
            Moves::AtRestFor(frames, towards, by) => {
                *frames -= 1;
                if *frames > 0 {
                    return;
                }
                let (dx, dy) = (towards.0 - self.at.0, towards.1 - self.at.1);
                let away = dx.hypot(dy).max(f32::EPSILON);
                let heading = (dx / away, dy / away);
                let pace = A_GRAIN_SETS_OFF_AT + *by;
                self.at = (self.at.0 + heading.0 * pace, self.at.1 + heading.1 * pace);
                self.moves = Moves::Speeding(heading, pace, *by);
            }
            Moves::Growing(step, grows) => {
                self.at = (self.at.0 + step.0, self.at.1 + step.1);
                self.half = (
                    (self.half.0 + *grows).min(SPLEEN_EATERS_BURST_REACHES),
                    (self.half.1 + *grows).min(SPLEEN_EATERS_BURST_REACHES),
                );
            }
            Moves::BurstsIn(frames, lasts) => {
                if *frames > 0 {
                    *frames -= 1;
                } else if *lasts > 0 {
                    *lasts -= 1;
                } else {
                    self.moves = Moves::Gone;
                }
            }
            Moves::Speeding(heading, pace, by) => {
                *pace = (*pace + *by).min(SPLEEN_EATERS_RICE_GOES_AT_MOST);
                self.at = (self.at.0 + heading.0 * *pace, self.at.1 + heading.1 * *pace);
            }
            Moves::Laid(heading, came_in_at, age) => {
                *age += 1;
                let slowed = (*age - RIVER_LAID_AT_FULL_PACE_FOR).max(0) as f32;
                let pace = (*came_in_at * (1.0 - slowed / RIVER_LAID_SLOWING_OVER as f32)).max(0.0);
                self.at = (self.at.0 + heading.0 * pace, self.at.1 + heading.1 * pace);
            }
            Moves::Bursting(gone, frames) => {
                *frames += 1;
                if *frames >= A_BURST_LASTS {
                    self.moves = Moves::Gone;
                } else {
                    *gone += a_bursts_pace(*frames);
                }
            }
            Moves::Slowing(heading, frames) => {
                *frames += 1;
                if *frames >= A_BURST_LASTS {
                    self.moves = Moves::Gone;
                } else {
                    let pace = a_bursts_pace(*frames);
                    self.at = (self.at.0 + heading.0 * pace, self.at.1 + heading.1 * pace);
                }
            }
            Moves::Gone => {}
        }
    }

    /// How far outside its box the run at `run` is, on whichever axis is the further: below nought inside it.
    fn room_from(&self, run: (f32, f32)) -> f32 {
        match self.moves {
            Moves::Bursting(..) | Moves::Gone | Moves::Laying(_) => f32::MAX,
            Moves::BurstsIn(frames, _) if frames > 0 => f32::MAX,
            _ => ((run.0 - self.at.0).abs() - self.half.0)
                .max((run.1 - self.at.1).abs() - self.half.1),
        }
    }

    /// **The chance a grain gone off its own way is on the run at `run`**: nought off the circle it may be
    /// anywhere on, and on it the share of the circle its box covers.
    ///
    /// **A chance and not a hit**: taken to be anywhere on its circle and so hitting wherever the circle crosses
    /// the run, the rice shut every pace in, the run was walked into fire with `nowhere open to it` and hit at
    /// `90.1,285.7` 637 frames into the card on nine attempts of ten from keep\at-107.
    fn chance_on(&self, run: (f32, f32)) -> f32 {
        let Moves::Bursting(gone, _) = self.moves else {
            return 0.0;
        };
        let across = self.half.0.max(self.half.1);
        if (distance(run, self.at) - gone).abs() >= across {
            return 0.0;
        }
        (2.0 * across / (std::f32::consts::TAU * gone.max(across))).min(1.0)
    }
}

/// **How far a grain of th10's `土着神「宝永四年の赤蛙」` goes on the frame of its burst counted from nought**:
/// [`A_BURST_GOES_AT_FIRST`] on the first two, then `1/35` of that less each frame — `5.00`, `4.57`, `4.00` and
/// `3.43` every four frames in the traced field, `90` gone by the burst's last frame and every grain then gone.
fn a_bursts_pace(frames: i32) -> f32 {
    A_BURST_GOES_AT_FIRST * (1.0 - (frames - 1).max(0) as f32 / A_BURST_SLOWS_OVER).max(0.0)
}

/// What [`looked_ahead`] keeps between frames, for one card: the card's frame it was last asked on and last
/// chose on, the pace chosen, and the place it has the run at — moved on by the pace each frame, so that a pace
/// slower than a step is a step every few frames.
///
/// **And where each body was on the frame before**, by its slot: a body has no step of its own handed over, its
/// script being what moves it, and a double of `「宝永四年の赤蛙」` read as standing still lays its rice on a run
/// it is coming for — hit at `-16.3,396.2` and `-19.9,395.8` 399 and 403 frames into the card from
/// keep\hits-xpd10\s7-c25-f31841, before any of the rice had burst.
struct River {
    on: Option<i32>,
    chosen_on: Option<i32>,
    pace: (f32, f32),
    place: (f32, f32),
    bodies: Vec<(usize, (f32, f32))>,
    bodies_before: Vec<(usize, (f32, f32))>,
    /// Where the run was on each of the card's last [`THE_RUN_IS_KEPT_FOR`] frames, by the card's frame.
    ran: Vec<(i32, (f32, f32))>,
}

impl River {
    const NONE: River = River {
        on: None,
        chosen_on: None,
        pace: (0.0, 0.0),
        place: (0.0, 0.0),
        bodies: Vec::new(),
        bodies_before: Vec::new(),
        ran: Vec::new(),
    };
}
/// **How many of the card's frames where the run was is kept for**: as far back as where the run was as the ring
/// that laid a grain of `恨霊「スプリーンイーター」`'s rice came, which it sets off at — see
/// [`SPLEEN_EATERS_RING_CAME_BEFORE_ITS_RICE_BY`].
const THE_RUN_IS_KEPT_FOR: usize = 320;

static RIVER: Mutex<River> = Mutex::new(River::NONE);
/// **The paces tried, a frame**: the focused step's `2.0` and two slower, which the run goes at by a step every few
/// frames. In the model, the three of them found the card's every frame unhit.
const RIVER_PACES: [f32; 3] = [0.4, 1.0, 2.0];
/// **How far ahead each pace is played on**: long enough for a bank's bullet set off at the run from well down the
/// river to have got there at [`RIVER_LETS_GO_AT`], and for a way down to the floor to be seen ending there. Played
/// in the model with the run's own focused steps from five places at the card's start, `120` with no
/// [`RIVER_AN_EDGE_IS_WORTH`] was shut in at the floor from two of them — `2,432` 866 frames in from `-36,390`,
/// which is where the real game was, `1.8,432.0` some 877 frames in from keep\hits-xpd2\s7-c17-f24245 — and `160`
/// went through from all five with the edges worth nothing, `0.25` or `0.5`.
const RIVER_LOOKS_AHEAD: i32 = 160;
/// **How much room the field's edge is, for each unit the run is from it**: a run at the floor has nowhere to go
/// from fire coming down at it. At `0.25`, `120` frames ahead went through from all five places as well; at `0.5`,
/// it was shut in at the floor from `0,280`, `64,432` 1,145 frames in.
const RIVER_AN_EDGE_IS_WORTH: f32 = 0.25;
/// **How often a pace is chosen again**, in frames of the card: a pace of `0.4` is one step of `2.0` in five, and
/// a choice made more often than a step would drop the pace before it was taken.
const A_PACE_IS_CHOSEN_EVERY: i32 = 4;
/// **The age a bank's bullet sets off at**: 610 of 610 in the traced field, every one at rest at `225` frames old and
/// moving at `226`.
const RIVER_SETS_OFF_AT: i32 = 226;
/// **How fast a bank's bullet goes once set off**, a frame: `1.40` for every one of them in the traced field.
const RIVER_LETS_GO_AT: f32 = 1.4;
/// **How long a bullet being laid comes in at the pace it came out at, and how long it then slows to rest over**,
/// in frames: `8.00` a frame to its eighth frame and `0.27` a frame less every two frames after, at rest at its
/// sixty-eighth, for the first bullet of the traced field — and each of them likewise at a pace of its own.
const RIVER_LAID_AT_FULL_PACE_FOR: i32 = 8;
const RIVER_LAID_SLOWING_OVER: i32 = 60;
/// **What a bullet at rest steps at most**, a frame: the banks' of `「厭い川の翡翠」` are at `0.01`, the rice of
/// `「宝永四年の赤蛙」` at `0.00`.
const AT_REST_UNDER: f32 = 0.05;
/// **How little a bullet being laid steps down at most**, a frame: they come in level from the walls.
const RIVER_LAID_LEVEL: f32 = 0.01;
/// **How much wider than the game's own test a box is kept**, on each side: the run's steps leave it up to a step
/// off the place it was going to.
const RIVER_KEEPS_A_BOX_WIDER_BY: f32 = 1.0;
/// **The room past which a pace is no better for more of it.**
const ROOM_ENOUGH: f32 = 26.0;
/// **Where the rivers cross**: `280` on a field whose floor is `432`, where the middles of all three meet at the
/// field's own middle — so a run near it is never near a bank as its row sets off.
const RIVER_HOME_ABOVE_THE_FLOOR: f32 = 152.0;
/// **How far from the crossing a pace may end for each unit of room it is left more**: at `20` the run was let
/// drift to the floor and shut in there — see [`RIVER_LOOKS_AHEAD`].
const RIVER_HOME_IS_WORTH: f32 = 8.0;

/// **Where th10's `土着神「宝永四年の赤蛙」` has the run go**: as [`jade_of_the_loathsome_river`] does, on at
/// whichever steady pace the card's own fire played on leaves unhit longest — a grain of rice gone off its own way
/// taken to be anywhere on the circle it has reached, which way being unknown until it has gone.
///
/// **The card lays rice where Suwako's doubles are and bursts it at one age**, read off its field
/// (`--trace-the-field-in=107` from keep\xpd10-fresh, 2026-10-02, `doubles107.py`, `rice-life107.py`): a double goes
/// to where the run is, slowing as it comes, stands there a while, flies off to some far point and comes back for
/// the run again — one double from frame 81 of the card and a second from 1,341 — and a grain of rice is born where
/// a double is every frame or so, `2.8` from it at the median. Every grain stands still at `0.00` until
/// [`RED_FROG_BURSTS_AT`] frames old and then goes off its own way, `90` in [`A_BURST_LASTS`] frames, and is gone.
/// The doubles cannot touch the run themselves, but the rice they lay is where they are, so they are kept off as a
/// grain would be.
///
/// **So the rice bursts where the run was some 412 frames before**, the doubles having come for it there; and the
/// way before this one, which walked the low line end to end turning back after 120 frames standing, came back to
/// where it had been: from keep\xpd10-fresh it was shut in near the middle and hit at `10.5,301.5` 1,650 frames into
/// the card and at `6.2,377.9` 1,594 in, by turns over ten attempts, where the run had been at `-25,363` 1,201 frames
/// in. Walked so with the moves judged held and not tapped, it had been hit at `-29.8,353.9` 1,513 frames in on seven
/// attempts running (keep\hits-xpd6\s7-c25-f33026); turning back after 20 frames, the end it went to was turned in
/// the middle and it was hit at `-83.6,348.1` 1,272 frames in on five attempts running (keep\hits-xpd7\s7-c25-f34177);
/// with each grain about to burst kept `10` wider over the 4 frames before, it was hit the frame after a clump went,
/// at `-131.0,365.7` 1,822 frames in, `15` wider over the whole of the frames looked at shut the field, hit at
/// `-6.1,374.6` 659 frames in on ten attempts running, and every grain at rest `10` wider shut it too, hit at
/// `30.9,384.9` 650 frames in on ten attempts running; and gone round the field from frame 1,800 on, it was hit at
/// the right wall going round, `171.8,292.4` and `164.3,391.7`.
///
/// **And with the place it would rather be moved aside by `aside`**, the attempt's own draw — see
/// `crate::steer::aside_on`: played the same way on every retry, the run was hit at the same place, 660 frames into
/// the card, on twelve attempts running in a whole Pointdevice Extra (keep\hits-x12-pd).
pub(crate) fn the_red_frog_of_hoei_four(
    frame: &Frame,
    card_frames: Option<i32>,
    aside: f32,
) -> (f32, f32) {
    looked_ahead(frame, card_frames, aside, &RED_FROG, &THE_RED_FROG)
}

const THE_RED_FROG: LookedAhead = LookedAhead {
    sets_off: SetsOff::AnyWay,
    paces: &RED_FROG_PACES,
    looks_ahead: RED_FROG_LOOKS_AHEAD,
    unhit_enough: RED_FROG_LOOKS_AHEAD + 1,
    turns_once: None,
    home: Home::WhereTheRiceIsYoungest,
    home_is_worth: RIVER_HOME_IS_WORTH,
    an_edge_is_worth: RED_FROG_AN_EDGE_IS_WORTH,
    keeps_a_box_wider_by: RED_FROG_KEEPS_A_BOX_WIDER_BY,
    aside_share: 1.0,
};

/// **Where on the field [`the_red_frog_of_hoei_four`] would rather the run be**: of a lattice of places
/// [`A_PLACE_OF_THE_LATTICE_IS`] apart and [`RED_FROG_IN_FROM_THE_EDGE`] in from the field's edge, the one with
/// the least rice at rest about it, each grain counted by how near it is to bursting and how near the place.
///
/// **Not a place held**, which is what the low line's middle was: a run that stands is a run the doubles come for
/// where it stands, and their rice there bursts some 412 frames on where it stood — from keep\at-107 the run stood
/// at `15,377` and `11,373` for some 160 frames apiece, was pressed to the floor and hit at `-32.3,428.4` 1,184
/// frames into the card and at `-117.5,401.3` 1,987 in. The wiki's `画面の角で囲まれないように大きく動いた方が安全`
/// is the run going where the rice is not.
///
/// **Nor a place going round a ring once in 824 frames**, which was weighed in the model for being on the far side
/// of it from where the run stood 412 frames before (`0,330`, `120` across and `70` down, each unit from it
/// costing half one of room): 6 plays of 22 hit (`card_107_played_in_a_model`), against 2 of 24 here.
///
/// **Nor the strategy wiki's bottom corner the doubles are led to and the other one after the second comes out**
/// (wikiwiki.jp/thk, 風/EX), weighed as a place by the card's own frames: the run was not held in either, walked up
/// the right wall instead of across, and went through 4 of the card's 8 kept chapters, every hit at that wall (c107i,
/// 2026-10-03). What gets a player through it is going where they went — a recipe's `follow`.
/// **The middle of the bullets at rest younger than `sets_off_at`**, which is inside the rows they were laid in; with
/// none at rest, nowhere.
fn among_the_rice_at_rest(frame: &Frame, sets_off_at: i32) -> Option<(f32, f32)> {
    let resting: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter(|hazard| hazard.frames.is_some_and(|age| age < sets_off_at))
        .filter(|hazard| {
            hazard
                .step
                .is_none_or(|step| step.0.hypot(step.1) < AT_REST_UNDER)
        })
        .map(|hazard| hazard.at)
        .collect();
    let count = resting.len() as f32;
    (!resting.is_empty()).then(|| {
        (
            resting.iter().map(|at| at.0).sum::<f32>() / count,
            resting.iter().map(|at| at.1).sum::<f32>() / count,
        )
    })
}

fn where_the_rice_is_youngest(frame: &Frame) -> (f32, f32) {
    let field = frame.field;
    let rice: Vec<((f32, f32), f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter_map(|hazard| {
            let age = hazard.frames?;
            let step = hazard.step.unwrap_or((0.0, 0.0));
            (step.0.hypot(step.1) < AT_REST_UNDER && age < RED_FROG_BURSTS_AT)
                .then(|| (hazard.at, (age as f32 / RED_FROG_BURSTS_AT as f32).powi(2)))
        })
        .collect();
    let (left, right) = (
        field.0 + RED_FROG_IN_FROM_THE_EDGE,
        field.0 + field.2 - RED_FROG_IN_FROM_THE_EDGE,
    );
    // **And a burst's reach below the boss**, whom the doubles fly out from and back to: a place beside her was gone
    // up to into the rice bursting about her and hit at `83.1,240.2` 621 frames into the card, from an empty suspend
    // directory (keep\hits-xpd11).
    let below_her = frame.boss.map_or(field.1, |boss| boss.1 + A_BURSTS_REACH);
    let (top, bottom) = (
        (field.1 + RED_FROG_IN_FROM_THE_EDGE).max(below_her),
        field.1 + field.3 - RED_FROG_IN_FROM_THE_EDGE,
    );
    let low_middle = (
        field.0 + field.2 / 2.0,
        field.1 + field.3 - RED_FROG_HOME_ABOVE_THE_FLOOR,
    );
    // The lattice laid from the low middle, so that it is one of its places.
    let first = |from: f32, to: f32| {
        from - ((from - to) / A_PLACE_OF_THE_LATTICE_IS).floor() * A_PLACE_OF_THE_LATTICE_IS
    };
    let mut best = (low_middle.0, low_middle.1, f32::MAX);
    let mut y = first(low_middle.1, top);
    while y <= bottom {
        let mut x = first(low_middle.0, left);
        while x <= right {
            let crowd: f32 = rice
                .iter()
                .map(|(at, soon)| {
                    let near = distance((x, y), *at) / A_BURSTS_REACH;
                    soon / (1.0 + near * near)
                })
                .sum::<f32>()
                + distance((x, y), low_middle) * A_UNIT_FROM_THE_LOW_MIDDLE_COSTS;
            if crowd < best.2 {
                best = (x, y, crowd);
            }
            x += A_PLACE_OF_THE_LATTICE_IS;
        }
        y += A_PLACE_OF_THE_LATTICE_IS;
    }
    (best.0, best.1)
}

/// **How far apart the places [`where_the_rice_is_youngest`] weighs are**, and **how far in from the field's edge**
/// the nearest of them is: a burst's reach in, so that a place chosen is never one a wall shuts a side of.
const A_PLACE_OF_THE_LATTICE_IS: f32 = 32.0;
const RED_FROG_IN_FROM_THE_EDGE: f32 = 64.0;
/// **How far a burst reaches**, every grain of it `90` from where it stood when it is gone.
const A_BURSTS_REACH: f32 = 90.0;

static RED_FROG: Mutex<River> = Mutex::new(River::NONE);
/// **The age every grain of the card's rice bursts at**: 1,205 of 1,205 in the traced field, at rest at `411`
/// frames old and gone off at `412` — and 1,447 of the card's first 1,830 frames in another
/// (keep\hits-xpd7\s7-c25-f33066, `bursts107.py`). Its own changes give two frames' warning and no more (`+0x43c`
/// going from `0x8000` to `0` and then `+0x45c` from `4` to `5`).
const RED_FROG_BURSTS_AT: i32 = 412;
/// **How long a burst lasts and how fast it starts**: `5.00` a frame on its first frame and `1/35` of that less
/// each frame from its second, every grain gone `36` frames after it went and `90` from where it stood then.
const A_BURST_LASTS: i32 = 36;
const A_BURST_GOES_AT_FIRST: f32 = 5.0;
const A_BURST_SLOWS_OVER: f32 = 35.0;
/// **Half the box of a grain of the card's rice**: every bullet of the traced field is `4.0` across.
const A_GRAINS_HALF: f32 = 2.0;
/// **How much wider than the game's own test a box is kept, on each side**: the river's.
///
/// **Not `3.0`, for a whole step's turn off the place it was going to**, which was tried after the run went by a
/// grain slowing to rest at `1.4` off and was hit on it, `-96.9,348.7` 1,039 frames into the card from
/// keep\hits-xpd10\s7-c25-f31841 (`--trace-the-field-in=107`, `near107.py`: slot 1092, `2.0` a frame off its burst at
/// 1,024 and `0.25` at 1,036): kept so much wider, every box shut more of the field, and from the same chapter the
/// run was walked to the right wall and hit there on five attempts running — `180.6,391.4`, `178.4,366.4`,
/// `173.0,432.0` — never through.
const RED_FROG_KEEPS_A_BOX_WIDER_BY: f32 = RIVER_KEEPS_A_BOX_WIDER_BY;
/// **The paces tried, a frame**: those of the river, the focused step's `2.0` being as far as one of the run's own
/// steps follows a place without overshooting it.
///
/// **Not the whole step's `4.5` among them**, which the run would go at with the focus key let go — see
/// [`a_pace_looked_ahead_wants_the_whole_step`]. Tried twice: with `120` frames looked ahead and a grain gone off
/// its own way taken to hit wherever its circle crosses the run, the run was hit at `90.1,285.7` 637 frames into the
/// card on nine attempts of ten from keep\at-107; with these, it was hit 2 times in 24 plays of the model
/// (`card_107_played_in_a_model`) against 4, and on the real game 0, 1 and 2 times from keep\at-107,
/// keep\hits-xpd11\s7-c25-f31504 and keep\hits-xpd10\s7-c25-f31841 against 0, 1 and 0 without it.
const RED_FROG_PACES: [f32; 3] = RIVER_PACES;
/// **How far ahead each pace is played on**: two bursts' length and more, so that a pace that ends among rice
/// about to go is seen ending there.
const RED_FROG_LOOKS_AHEAD: i32 = 90;
/// **How much room the field's edge is, for each unit the run is from it**: twice the river's. At the river's
/// `0.25` the run was pressed down onto the floor by the doubles' rice and walked along it into the bottom-right
/// corner, where it stood from 1,056 frames into the card and was hit at `184.0,432.0` 1,272 in, from
/// keep\hits-xpd10\s7-c25-f31841 — the wiki's `画面の角で囲まれないように`.
const RED_FROG_AN_EDGE_IS_WORTH: f32 = 0.5;
/// **Where the run would rather be with the rice alike everywhere**, as with none: the low line the way before this
/// one walked, `368` on a field whose floor is `432`, at the field's middle across — a unit off it costing as much
/// as a thousandth of a grain about to burst at the place.
const RED_FROG_HOME_ABOVE_THE_FLOOR: f32 = 64.0;
const A_UNIT_FROM_THE_LOW_MIDDLE_COSTS: f32 = 0.001;

/// **The age every bullet of th10's `蛙狩「蛙は口ゆえ蛇に呑まるる」` bursts at, and how fast it goes**, which
/// `crate::steer` fears a knot about to burst over — see [`Dodge::TheSnakeEatsTheCroakingFrog`]. Read off the card's
/// field (`--trace-the-field-in=104` from keep\at-104, 2026-10-02, `rest104.py`, `knots104.py`): the card lays a knot
/// of six bullets at rest within `30` of its snake every frame — a body that cannot touch the run, going round the
/// field from under the boss — and every one of the knots' 7,721 bullets stood at `0.00` from age `1` to `130` and went
/// off at `4.00` a frame at `131`, the six of a knot on three headings `120°` apart.
///
/// **Not a way of the card's own that looks ahead, as [`the_red_frog_of_hoei_four`] does**, the two cards' fire being
/// one kind: played on so, the knots at rest made every place near the snake's way a crowd, and the run fled them into
/// the bottom-right corner and was shut in there and at the walls, hit at `175.8,393.8`, `174.9,432.0` and
/// `-181.2,430.6` 493 to 532 frames into the card on three attempts running from keep\at-104 — which the rules alone go
/// through.
///
/// [`Dodge::TheSnakeEatsTheCroakingFrog`]: orb_config::recipe::Dodge::TheSnakeEatsTheCroakingFrog
pub(crate) const SNAKE_KNOTS_BURST_AT: i32 = 131;
pub(crate) const SNAKE_KNOTS_BURST_PACE: f32 = 4.0;

/// **Where th10's `神符「神が歩かれた御神渡り」` has the run while a pillar is being laid down to it**:
/// aside of the pillar, on the side of it with more of the field, at the run's own height — and wherever the rules
/// would have it otherwise.
///
/// **A pillar is laid from the top of the field down at where the run stood**, read off the card's field
/// (`--trace-the-field-in=99` from keep\hits-x15-pd\s7-c8-f9088, 2026-10-03, `births99.py`, `life99.py`): every
/// 220 frames or so two bullets a frame are born at rest `4` below the two before, one strand straight down at the
/// pillar's middle and the other winding `25` either side of it once in 60 frames, so that the pillar reaches the
/// floor 108 frames after it starts; each stands at `0.00` some 110 frames and then rises off the top. **What hits
/// is the pillar's foot as it is laid**, bullets the rules cannot see before they are born: from that chapter the run
/// stood at the middle of a pillar, stepped down along it as its foot came, and was hit at `73.6,414.3` by bullets
/// two to eight frames old, 953 frames into the card.
///
/// **Card 99 by the game's own name for it**, read out of `scoreth10.dat`'s records of the cards and not from
/// memory: this was named for `神祭「エクスパンデッド・オンバシラ」`, which is cards 77 and 78 of stage 6.
pub(crate) fn omiwatari_that_god_walked(frame: &Frame) -> Option<(f32, f32)> {
    let laid: Vec<(i32, (f32, f32))> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter(|hazard| {
            let step = hazard.step.unwrap_or((0.0, 0.0));
            step.0.hypot(step.1) < AT_REST_UNDER
        })
        .filter_map(|hazard| {
            let age = hazard.frames?;
            (age <= A_PILLAR_WINDS_ONCE_IN).then_some((age, hazard.at))
        })
        .collect();
    // Its foot, the place the youngest of it is being laid at.
    let (_, foot) = laid.iter().min_by_key(|(age, _)| *age)?;
    if foot.1 > frame.at.1 + A_PILLARS_FOOT_IS_THROUGH_BELOW {
        return None;
    }
    let (left, right) = laid
        .iter()
        .fold((f32::MAX, f32::MIN), |(left, right), (_, at)| {
            (left.min(at.0), right.max(at.0))
        });
    let (left, right) = (
        left - A_PILLAR_IS_KEPT_OFF_BY,
        right + A_PILLAR_IS_KEPT_OFF_BY,
    );
    if !(left..=right).contains(&frame.at.0) {
        return None;
    }
    let field = frame.field;
    let (wall_left, wall_right) = (
        field.0 + TWO_BOWS_OFF_THE_WALLS,
        field.0 + field.2 - TWO_BOWS_OFF_THE_WALLS,
    );
    let x = match left - wall_left > wall_right - right {
        true => left,
        false => right,
    };
    Some((x.clamp(wall_left, wall_right), frame.at.1))
}
/// **How long a pillar takes to wind once about its middle**, in frames — and so how old the youngest of its bullets
/// laid across its whole width are.
const A_PILLAR_WINDS_ONCE_IN: i32 = 60;
/// **How far below the run a pillar's foot is once it is through**: some seven of the rules' whole steps of `4.5`.
const A_PILLARS_FOOT_IS_THROUGH_BELOW: f32 = 32.0;
/// **How far off a pillar's own width the run is kept while it is laid**: some five of the rules' whole steps.
const A_PILLAR_IS_KEPT_OFF_BY: f32 = 24.0;

/// **Where th10's `神具「洩矢の鉄の輪」` has the run**: under the boss or above her, and over to the other as soon
/// as fire is about to reach it — the strategy wiki's `諏訪子の真下で撃ち込みつつ待機して…輪を迂回して諏訪子真上へ`
/// (wikiwiki.jp/thk, 風/EX), with the change made when the rings come and not by their count: what a player of
/// the card does is wait under her until the rings are about to arrive, go above her at full pace, and come back
/// under as they arrive there.
///
/// **The rings are aimed at where the run stood as each was made**, read off the card's field (`ringlife.py`,
/// 2026-10-02): a band of three circles `46` to `77` across its middle, made at the boss, `0,224`, every 40 frames
/// in rounds of ten 550 apart; it grows there for 70 frames, goes at `1.61` a frame to the place, and comes back
/// past the boss. So rings sent at a run under her come down the middle and back up it, and rings sent at one above
/// her go up and back down — a run that changes ends as they arrive leaves each ring a place it has gone from.
///
/// **Ways of the card's that kept the run in one part of the field each lost it there**, from keep\hits-xpd8\
/// s7-c15-f19277: in_the_middle and a climb over `120` from the top judged hit were walked into a wall and up it
/// (`-57.9,32.0`, `184.0,141.6`, `173.7,141.6`, `-184.0,32.0`); round_the_field at `-138.1,432.0` 1,714 frames in,
/// a slow lap of the low half at `184.0,432.0` 672 frames in, round_the_field with the edges warned of at
/// `145.0,432.0`, and the bottom-right corner stood in at `184.0,432.0` 351 frames in.
pub(crate) fn moriyas_iron_rings(
    frame: &Frame,
    card_frames: Option<i32>,
    aside: f32,
) -> (f32, f32) {
    let field = frame.field;
    let floor = field.1 + field.3;
    let boss = frame
        .boss
        .unwrap_or((field.0 + field.2 / 2.0, MORIYAS_BOSS_AT));
    // **As low and as high as the run goes**, the field's own floor and top: the furthest a ring sent straight down
    // or up has to come, and so the most time to go to the other end before it gets there. `16` and `32` in from them
    // left the run at `416` and `64` with the field at `32` to `432`.
    //
    // **And a share of a retry's aside off her**: held under her on every attempt, a retry rewound into the chapter
    // played the card out as the attempt before did — u34's Pointdevice Extra was hit at `91.9,432.0` 1,859 frames in
    // on twenty-five attempts running (traces\20261004-104142-engine-extra-pd.msgpack), until it was closed.
    let off_her = (boss.0 + aside * MORIYAS_ASIDE_SHARE).clamp(field.0, field.0 + field.2);
    let under = (off_her, floor);
    let above = (off_her, field.1);
    let mut kept = MORIYAS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A retry of its chapter starts the card over, under her.
    if kept
        .on
        .is_none_or(|was| card_frames.is_some_and(|now| now < was))
    {
        *kept = Moriyas::NONE;
    }
    kept.on = card_frames;
    // **The rings up, each by the age its bullets share**: a ring's 144 are made together, so one age is one ring —
    // its middle where they are on average, its reach the furthest of them from it, and where it is going their
    // steps on average, the circles' spreading cancelling out across them.
    let mut bullets: Vec<Aged> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter_map(|hazard| Some((hazard.frames?, hazard.at, hazard.step.unwrap_or((0.0, 0.0)))))
        .collect();
    bullets.sort_unstable_by_key(|(age, _, _)| *age);
    let at = frame.at;
    let about_to_reach_it = bullets
        .chunk_by(|a, b| a.0 == b.0)
        .filter(|ring| ring.len() >= MORIYAS_A_RINGS_BULLETS)
        .any(|ring| {
            let n = ring.len() as f32;
            let sum = |of: fn(&Aged) -> (f32, f32)| {
                ring.iter()
                    .map(of)
                    .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1))
            };
            let (x, y) = sum(|b| b.1);
            let middle = (x / n, y / n);
            let (dx, dy) = sum(|b| b.2);
            let going = (dx / n, dy / n);
            let reach = ring
                .iter()
                .map(|b| distance(b.1, middle))
                .fold(0.0, f32::max);
            let across = ring.iter().map(|b| distance(b.1, middle)).sum::<f32>() / n;
            // **Coming at the run by its middle's step, and not by its nearest bullet's as well**: a ring going by on
            // average still reaches the run with its edge, and counted so it changed the ends 120 frames sooner on
            // keep\hits-u34\s7-c15-f21020 — but the run, on the floor under her, had no way up through that ring's
            // edge and was hit at `91.9,432.0` 1,859 frames in as before, the other three kept chapters clean either
            // way (c102-near.sh, 2026-10-04).
            let towards = going.0 * (at.0 - middle.0) + going.1 * (at.1 - middle.1) > 0.0;
            MORIYAS_A_RINGS_RADIUS.contains(&across)
                && towards
                && distance(at, middle) - reach < MORIYAS_A_RING_ABOUT_TO_REACH_WITHIN
        });
    // **At its end, by which side of the boss it is on and not by how near the end's own place**: a run the rings
    // pushed `84` along the top from it went on waiting there for a ring to come, and was shut in at the top-right
    // corner, `184.0,56.4`, 684 frames into the card on every attempt.
    //
    // **And past her by more than a ring's reach**, not merely past her height: a run just over it on the way up,
    // `-89,214`, was turned back down by a ring coming, and with the way down on the far side of her was shut in at
    // the top, `-105.3,76.5`, 1,346 frames into the card on every attempt from keep\at-102.
    let at_its_end = match kept.above {
        true => at.1 < boss.1 - MORIYAS_AT_AN_END_PAST_HER_BY,
        false => at.1 > boss.1 + MORIYAS_AT_AN_END_PAST_HER_BY,
    };
    // **And not back under her as soon as the run is above her**, which was tried: going straight back down, the run
    // changed ends twice as often, came down the side it had gone up into the rings it sent there, and was held at the
    // left wall and hit at `-175.8,77.3` 1,162 frames into the card from keep\at-102 — and turned back at the field's
    // top instead, at `-180.8,94.0` 1,652 in (c102n, c102p, 2026-10-03).
    if about_to_reach_it && at_its_end {
        kept.above = !kept.above;
    }
    let wanted = match kept.above {
        true => above,
        false => under,
    };
    // **The way round the rings coming down the middle**, by where the run is: straight at the boss's side at her
    // own height, and from there straight at the far end — `96` off her clear of a ring as it grows about her.
    // **Up the left and down the right**, the rings sent at the run on its way up being the ones left lying on the
    // side it went up.
    //
    // **Not across at the end and then up or down the side**: across the top first, the rings sent at the run
    // as it went were sent up to the right of the boss, and it was shut in under them at `184.0,79.2` 697 frames
    // into the card on every attempt.
    //
    // **Or the side the run is already on**, where it is off her x by half the way round or more: a way down the
    // right from the left of her is a way over her, through the rings growing there.
    let side = match (at.0 - boss.0, wanted == above) {
        (off, _) if off.abs() >= MORIYAS_ROUND_THE_RINGS_BY / 2.0 => {
            boss.0 + MORIYAS_ROUND_THE_RINGS_BY.copysign(off)
        }
        (_, true) => boss.0 - MORIYAS_ROUND_THE_RINGS_BY,
        (_, false) => boss.0 + MORIYAS_ROUND_THE_RINGS_BY,
    };
    let past_her = match wanted == above {
        true => at.1 < boss.1 - MORIYAS_AT_HER_SIDE_WITHIN,
        false => at.1 > boss.1 + MORIYAS_AT_HER_SIDE_WITHIN,
    };
    let at_her_side = (at.1 - boss.1).abs() <= MORIYAS_AT_HER_SIDE_WITHIN
        && (at.0 - side).abs() <= MORIYAS_AT_HER_SIDE_WITHIN;
    match past_her || at_her_side {
        true => wanted,
        false => (side, boss.1),
    }
}

/// What [`moriyas_iron_rings`] keeps between frames: the card's frame it was last asked on, and which end the run
/// is at.
struct Moriyas {
    on: Option<i32>,
    above: bool,
}

impl Moriyas {
    const NONE: Moriyas = Moriyas {
        on: None,
        above: false,
    };
}

static MORIYAS: Mutex<Moriyas> = Mutex::new(Moriyas::NONE);
/// **How many bullets of one age are read as a ring**: a ring's 144, of which some may have gone.
const MORIYAS_A_RINGS_BULLETS: usize = 40;
/// **How near a ring coming at the run has to be before the run goes to the other end**: at the ring's `1.61` a frame
/// against the run's `4.5`, out of its way before it gets there — and nearer than the `24` a run went down at, which
/// left the top too soon.
const MORIYAS_A_RING_ABOUT_TO_REACH_WITHIN: f32 = 12.0;
/// **How far a ring's bullets stand from its middle on average**: `62` once grown, its three circles `46`, `62` and
/// `77` — and a group of one age outside it no ring, which is what had the run go up 40 frames into the card with
/// no ring of it yet sent.
const MORIYAS_A_RINGS_RADIUS: std::ops::RangeInclusive<f32> = 40.0..=80.0;
/// **Where the boss stands**, `0,224` throughout the card, for a frame that hands none over.
const MORIYAS_BOSS_AT: f32 = 224.0;
/// **How much of a retry's aside the ends are moved off her by**: a quarter, `24` at the most — under a ring's `96`
/// clear of her, so that an end is still where a ring sent straight at it has furthest to come.
const MORIYAS_ASIDE_SHARE: f32 = 0.25;
/// **How far to the side the run goes round the rings coming down the middle**: past a ring's outer circle, `77`.
const MORIYAS_ROUND_THE_RINGS_BY: f32 = 96.0;
/// **How far past the boss's own height the run has to be to be at an end**: a ring's reach as it grows about her,
/// `77`, and more.
const MORIYAS_AT_AN_END_PAST_HER_BY: f32 = 96.0;
/// **How near the place at the boss's side the run has to be before it goes on to the far end**: a few steps, its
/// own dodging moving it about.
const MORIYAS_AT_HER_SIDE_WITHIN: f32 = 16.0;

/// **Where th10's `天流「お天水の奇跡」` has the run**: along the low line wherever the rules' own dodging has carried
/// it, and — once that has brought it within [`OTENSUI_SWUNG_IN_FROM_THE_WALL`] of a wall — cut back (`切り返し`), the
/// strategy wiki's way through the card (wikiwiki.jp/thk, 風/6): a quick swing up and [`OTENSUI_SWUNG_BY`] on towards
/// the wall, and from there at once back the other way to [`OTENSUI_CUT_BACK_BY`] past where the swing set off, before
/// going on along the low line from there.
///
/// **The dodging already leads the fire along**, which is the first of a cut back's three parts: aimed at the run, it
/// lands where the run has just been as long as the run goes on the same way at a steady pace, and the rules' taps off
/// it are that pace. **What it cannot do is turn**, the fire let go at a slow run coming down as a close row behind it:
/// so the swing, quick where the lead was slow, has the fire let go during it aimed at places far apart, and the way
/// back goes through the gap that leaves.
///
/// **Its fire is aimed at the run from a line across the field**, read off the card's field (`--trace-the-field-in=90`
/// from four kept Pointdevice chapters at its start, 2026-10-03, `ages90.py`, `fan90.py`): bullets come down at `9.3` a
/// frame slowing to rest at `y` 295 by age 65, and at age 70 set off at `1.2` a frame, many of them straight at the
/// run, among a spray whose heading turns. So the bullets near a run off the middle are going the way it is, `-0.7` a
/// frame by it at `x = -140` and `+0.8` at `150` (`path-card.py`), and the rules dodging them stream along the floor
/// to a wall — where, the fire still aimed at the run, there is no further to go.
///
/// **Not a place to hold**, which every way tried was: in_the_middle and under_the_boss, and stands at `0,410`,
/// `96,410` and `150,410`, were each walked to a wall and hit there 870 to 1,430 frames in, 3, 1 and 3 of the same four
/// chapters through; the one attempt through by a wall had cut back from it, `-184` to `118` in 150 frames. Cut back
/// so, with the moves judged over `crate::steer`'s crossing look-ahead while it is, all four went through (c90r,
/// 2026-10-03), the chapter every other way lost among them.
pub(crate) fn the_miracle_of_otensui(
    frame: &Frame,
    card_frames: Option<i32>,
    aside: f32,
) -> (f32, f32) {
    let field = frame.field;
    let (left, right) = (field.0, field.0 + field.2);
    let swung_in = OTENSUI_SWUNG_IN_FROM_THE_WALL + aside * OTENSUI_ASIDE_SHARE;
    let low = field.1 + field.3 - OTENSUI_ABOVE_THE_FLOOR;
    let at = frame.at;
    let mut kept = OTENSUI
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // A retry of its chapter starts the card over, with no cut back under way: the card's frames counted down again.
    // **Not a frame that hands no count over**, which leaves the cut back under way as it was — reset on one, a swing
    // was set off again from wherever it had got to and never came back.
    if card_frames.is_some_and(|now| kept.on.is_some_and(|was| now < was)) {
        *kept = Otensui::NONE;
    }
    if card_frames.is_some() {
        kept.on = card_frames;
    }
    let to = |cut_back: CutBack| {
        let x = match cut_back {
            CutBack::Swinging { from, towards } => from + OTENSUI_SWUNG_BY * towards,
            CutBack::Back { from, towards } => from - OTENSUI_CUT_BACK_BY * towards,
        };
        x.clamp(left, right)
    };
    let up = field.1 + field.3 - OTENSUI_CUT_BACK_ABOVE_THE_FLOOR;
    let reached = |cut_back: CutBack| (at.0 - to(cut_back)).abs() <= OTENSUI_REACHED_WITHIN;
    // **The swing done only up at its height as well as across**: done on the x alone, a swing begun on the low line
    // reached its x at `123,368` on the way up, the way back was set off from there, and the fire let go at the run
    // at that height stood between it and the middle — two moves of nine clear at most and none of them back, the
    // run carried down to the floor (card frames 630 to 740 of a stage 6 practice, c90f, 2026-10-03), where one done
    // up at `119,348` had that fire behind it and came back through.
    let swung = |swing: CutBack| reached(swing) && (at.1 - up).abs() <= OTENSUI_REACHED_WITHIN;
    kept.cut_back = match kept.cut_back {
        Some(swing @ CutBack::Swinging { from, towards }) if swung(swing) => {
            Some(CutBack::Back { from, towards })
        }
        // **Wanted back until it is got to, and not given up after a time**, which was tried: a way back the fire has
        // shut is wanted on while the rules dodge — one such was wanted 400 frames and hit at `184.0,254.6` (c90t) —
        // but given up after 160 frames the run, by then near the wall it swung at, swung at it again at once and was
        // shut into that wall's bottom corner, hit at `184.0,428.8` and `184.0,432.0` some 1,030 frames into the card
        // on two practices of three (c90u) where the swing done at its height alone went through five of six.
        Some(back @ CutBack::Back { .. })
            if (at.0 - to(back)).abs() <= OTENSUI_BACK_REACHED_WITHIN =>
        {
            None
        }
        // Up off the low line on the way, and only back to it once across.
        None if at.0 < left + swung_in => Some(CutBack::Swinging {
            from: at.0,
            towards: -1.0,
        }),
        None if at.0 > right - swung_in => Some(CutBack::Swinging {
            from: at.0,
            towards: 1.0,
        }),
        under_way => under_way,
    };
    match kept.cut_back {
        Some(cut_back) => (to(cut_back), up),
        None => (at.0, low),
    }
}

/// What [`the_miracle_of_otensui`] keeps between frames: the card's frame it was last asked on, and the cut back
/// under way.
struct Otensui {
    on: Option<i32>,
    cut_back: Option<CutBack>,
}

impl Otensui {
    const NONE: Otensui = Otensui {
        on: None,
        cut_back: None,
    };
}

/// **A cut back under way**, by the x it set off from and the way to the wall it is short of, `-1` left and `1` right:
/// the swing on towards the wall, or the way back the other side of where it set off.
#[derive(Clone, Copy, Debug, PartialEq)]
enum CutBack {
    Swinging { from: f32, towards: f32 },
    Back { from: f32, towards: f32 },
}

static OTENSUI: Mutex<Otensui> = Mutex::new(Otensui::NONE);
/// **How near a wall the dodging is let carry the run before it cuts back**: room for the swing and some steps more —
/// every attempt hit by a wall had been carried there and stood at it 150 frames and more.
const OTENSUI_SWUNG_IN_FROM_THE_WALL: f32 = 96.0;
/// **How much of a retry's aside moves that reach**, a quarter, `24` either way at the most — see
/// `crate::steer::aside_on`: with the reach the same on every retry a retry is the attempt before it over again, and a
/// Pointdevice Normal (u27, 2026-10-04, traces\20261004-071836-engine-normal-pd.msgpack) was hit at `184.0,432.0`,
/// 2,050 frames into the card with its boss's life at 92, on every one of some forty retries running before it was
/// closed.
const OTENSUI_ASIDE_SHARE: f32 = 0.25;
/// **How far the swing goes on towards the wall**: some nine of the rules' whole steps of `4.5`, quick where the lead
/// along the low line is a tap at a time, so that what is let go while it is under way is aimed at places that far
/// apart.
const OTENSUI_SWUNG_BY: f32 = 40.0;
/// **How far back past where the swing set off the way back goes**: past the row the fire let go at the slow run came
/// down in behind it. **Not the swing's own width besides**, `64`, which was tried: the way back from a swing at `89`
/// to `129` wanted `25,350` across the fire for 250 frames and more with `0 of the nine clear`, and the run was pushed
/// down to the floor and hit — `136.4,391.7` 1,055 frames in from keep\hits-u34\s6-c11-f13588 on every pickup, and
/// `135.5,432.0` 1,932 in on u40's legacy Normal; at `32` that chapter and the five others kept at the card went
/// through (c90-cb32.sh, 2026-10-04).
const OTENSUI_CUT_BACK_BY: f32 = 32.0;
/// **How near the x it is going to a part of the cut back is over**: a step, the run's own dodging moving it about.
const OTENSUI_REACHED_WITHIN: f32 = 6.0;
/// **How near the x it is going to the way back is over**, wider than [`OTENSUI_REACHED_WITHIN`]: the way back is
/// across the fire's lines, and the run's dodging of them holds it some steps off the place — u38's Pointdevice
/// Normal came back to `34,363` with the place at `26,350` 1,070 frames into the card, was held there, and with the
/// place still wanted was carried along to the right wall and hit at `184.0,306.6` 1,569 frames in on every pickup
/// (keep\hits-u38\s6-c11-f14264, c90-u38.sh, 2026-10-04).
const OTENSUI_BACK_REACHED_WITHIN: f32 = 16.0;
/// **How high the run goes along the low line**: `410` on th10's floor of `432`, the height the stands held and the
/// attempts through went along at.
const OTENSUI_ABOVE_THE_FLOOR: f32 = 22.0;
/// **How high a cut back goes**: `350`, some `55` under the line the bullets come to rest along, where the fire set off
/// at the run has come least far and the lines of it aimed at one place are furthest apart — the strategy wiki's cut
/// back goes up as it swings. **Not along the low line**, which was tried: the fire let go at the run some `100` frames
/// before it reaches it, the way back along the low line went into a close row of it still coming down where the run
/// had been, and the dodging took the run on into the wall — 1 of four kept chapters through, hit at `175.5,432.0`,
/// `-184.0,305.7` and `184.0,373.8` (c90p, 2026-10-03).
const OTENSUI_CUT_BACK_ABOVE_THE_FLOOR: f32 = 82.0;

/// **Where across th10's `土着神「手長足長さま」` goes up through its slow ring, where the ring is coming down on the
/// run: under the gap between two of its bullets nearest the run**, and `None` where no such ring is above.
///
/// **The card's slow ring is a line the rules' dodging does not cross**: its bullets are `11` or `12` apart round it,
/// stepping `0.5` a frame, and the run below it was carried down by it instead of up through it — from
/// keep\hits-u32\s7-c13-f17489 under the ring from 1,100 frames into the card, at `258` where the way wanted `236`
/// and at `321` as it wanted `341` and up, and hit at `-64.2,338.1` 1,295 frames in with the ring's bullets at
/// `-67,334` and `-56,338` (`slow101.py`, 2026-10-04) — the place and the frame of four hits of the whole Extras of
/// 2026-10-04 (u32's three and keep\b2-2's). **The user's own goes through it** (watch-replay-02): below it at
/// `y = 353` by 1,230 frames in and above it at `275` by 1,320, at `x = 4` to `8` — between the ring's bullets at
/// `5` and `17` at its foot, where it is flattest.
///
/// **The gap nearest the run, and not the one nearest the middle**, which was tried: from that chapter the run came
/// under the ring `100` left of the middle, and wanting the middle's gap it was judged hit on every move along under
/// the ring towards it — `0 of the nine clear` for some 120 frames — and hit at the same place and frame
/// (c101-gap.sh, 2026-10-04). The gap above the run is at most half a gap away.
pub(crate) fn through_tenaga_ashinagas_ring(frame: &Frame) -> Option<f32> {
    let slow_above = |hazard: &&Hazard| {
        matches!(hazard.kind, Kind::Bullet { .. })
            && hazard
                .step
                .is_some_and(|step| step.0.hypot(step.1) < TENAGA_ASHINAGAS_RING_STEPS_UNDER)
            && hazard.at.1 < frame.at.1
            && hazard.at.1 > frame.at.1 - TENAGA_ASHINAGAS_RING_ABOVE_WITHIN
            && (hazard.at.0 - frame.at.0).abs() < TENAGA_ASHINAGAS_RING_LOOKED_AT_WITHIN
    };
    // The ring's own bullets, all born on one frame: the lowest of the slow ones above the run is the ring that
    // reaches it first.
    let ring = frame
        .hazards
        .iter()
        .filter(slow_above)
        .max_by(|a, b| a.at.1.total_cmp(&b.at.1))?
        .frames;
    let mut across: Vec<f32> = frame
        .hazards
        .iter()
        .filter(slow_above)
        .filter(|hazard| hazard.frames == ring)
        .map(|hazard| hazard.at.0)
        .collect();
    across.sort_by(f32::total_cmp);
    across
        .windows(2)
        .map(|pair| (pair[0] + pair[1]) / 2.0)
        .min_by(|a, b| (a - frame.at.0).abs().total_cmp(&(b - frame.at.0).abs()))
}
/// **Whether card 101's slow ring is just below the run**, across from it — the run just up through it, where the way
/// keeps it from going back down.
///
/// **Because the way after the gap is no longer under it**: from keep\hits-u32\s7-c13-f17465 the run went up through
/// the ring between its bullets at `-2` and `11` 1,320 frames into the card, to `4.2,352.3` with the ring at `357`, and
/// back down through it five frames later onto the bullet at `-1.7,361.2`, hit at `4.2,356.8` 1,322 in — wanting the
/// middle above with nothing between it and there that was not judged hit (c101-gap2.sh, 2026-10-04).
pub(crate) fn tenaga_ashinagas_ring_just_below(frame: &Frame) -> bool {
    frame.hazards.iter().any(|hazard| {
        matches!(hazard.kind, Kind::Bullet { .. })
            && hazard
                .step
                .is_some_and(|step| step.0.hypot(step.1) < TENAGA_ASHINAGAS_RING_STEPS_UNDER)
            && hazard.at.1 > frame.at.1
            && hazard.at.1 < frame.at.1 + TENAGA_ASHINAGAS_RING_BELOW_WITHIN
            && (hazard.at.0 - frame.at.0).abs() < TENAGA_ASHINAGAS_RING_JUST_BELOW_WITHIN
    })
}
/// **How near across from the run the ring is just below it**: some three of the whole step's `4.5`.
const TENAGA_ASHINAGAS_RING_JUST_BELOW_WITHIN: f32 = 16.0;
/// **How near below the run the ring is just below it**, further than across: held above a ring only `16` below it,
/// the run up at `5,306` 827 frames into the card came down with the way's height once the ring was further below,
/// was under it again at `30,349` by 880, and with no way up through it a second time was hit at `-0.3,361.5` 959 in
/// (u41, traces\20261004-163659-engine-extra.msgpack). At `48` the seven card 101 chapters `16` took went through as
/// well (c101-below48.sh, 2026-10-04).
const TENAGA_ASHINAGAS_RING_BELOW_WITHIN: f32 = 48.0;
/// **How slow a bullet of the slow ring steps**, a frame: its own `0.5`, the card's other fire twice that and more.
const TENAGA_ASHINAGAS_RING_STEPS_UNDER: f32 = 0.8;

/// **Where th11's `妬符「グリーンアイドモンスター」` is waited out on the stage's frame `stage_frames`**: under the boss
/// at the low line as each monster comes, up under her from when it comes within
/// [`THE_GREEN_EYED_MONSTER_REACHES_THE_RUN_WITHIN`] of the run at [`THE_GREEN_EYED_MONSTER_IS_LED_UP_BY`] a frame to
/// [`THE_GREEN_EYED_MONSTER_IS_LED_UP_TO`] below her, and
/// from there down again [`BESIDE_THE_GREEN_EYED_MONSTERS_TRAIL_BY`] to the side the field is wider on — and
/// nowhere before the first monster and once one is long gone.
///
/// **Led up under the boss and back down beside its trail**, the strategy wiki's way for Normal
/// (`一回目をパルスィの真下で打ち込み`, `弾が来たら低速で上に移動しながら限界まで粘って打ち込み`,
/// `打ち込めない高さまで行ったら切り返して適当に避ける`, `一回目の弾が消えたらすぐにパルスィの真下で再び打ち込み`,
/// wikiwiki.jp/thk/地/2). Read off the chapters of card 11 kept on 2026-10-07
/// (traces\20261007-152745-tune-s2-c3-round_the_field.msgpack): every 490 frames the boss goes over to `x = 128`
/// or `-128` by turns and a monster some 11 across comes in from a top corner and follows the run, laying a still
/// bullet behind it every other frame, 275 by the time the whole trail goes at once; the rules' own stood still and
/// the monster went round them, and they were hit 80 frames on with bullets of the trail 1 to 25 off on every side.
/// Led straight up, the trail is laid as a line under the run.
pub(crate) fn after_the_green_eyed_monster(frame: &Frame, stage_frames: i64) -> Option<(f32, f32)> {
    let monster = frame
        .hazards
        .iter()
        .find(|hazard| {
            matches!(hazard.kind, Kind::Body { .. })
                && (A_GREEN_EYED_MONSTER_REACHES..A_GREEN_EYED_MONSTER_REACHES_UNDER)
                    .contains(&hazard.half.0.max(hazard.half.1))
                && hazard.at.1 > frame.field.1 + A_GREEN_EYED_MONSTER_IS_IN_BY
        })
        .map(|hazard| hazard.at);
    let mut seen = THE_GREEN_EYED_MONSTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if monster.is_some() && !seen.there {
        seen.came = Some(stage_frames);
        seen.reached = None;
    }
    seen.there = monster.is_some();
    if seen.reached.is_none()
        && monster.is_some_and(|at| {
            distance(at, frame.at) < THE_GREEN_EYED_MONSTER_REACHES_THE_RUN_WITHIN
        })
    {
        seen.reached = Some(stage_frames);
    }
    let (came, boss) = (seen.came?, frame.boss?);
    let since = stage_frames - came;
    if !(0..A_GREEN_EYED_MONSTER_IS_GONE_AFTER).contains(&since) {
        return None;
    }
    let (left, top, width, height) = frame.field;
    let low = top + height - THE_GREEN_EYED_MONSTER_IS_WAITED_FOR_ABOVE_THE_FLOOR;
    let highest = boss.1 + THE_GREEN_EYED_MONSTER_IS_LED_UP_TO;
    let led = seen.reached.map_or(0.0, |reached| {
        (stage_frames - reached).max(0) as f32 * THE_GREEN_EYED_MONSTER_IS_LED_UP_BY
    });
    let climb = (low - highest).max(0.0);
    if led <= climb {
        return Some((boss.0, low - led));
    }
    let middle = left + width / 2.0;
    let aside = if boss.0 < middle { 1.0 } else { -1.0 };
    Some((
        boss.0 + aside * BESIDE_THE_GREEN_EYED_MONSTERS_TRAIL_BY,
        (highest + (led - climb)).min(low),
    ))
}
/// **How far a body reaches to be card 11's monster**: its 11, between its bullets' and the boss's 19.
const A_GREEN_EYED_MONSTER_REACHES: f32 = 9.0;
const A_GREEN_EYED_MONSTER_REACHES_UNDER: f32 = 15.0;
/// **How far below the field's top a body has to be to be the monster**: the two bodies of that reach that stay at
/// `0,-16` and `0,0` through the card are not.
const A_GREEN_EYED_MONSTER_IS_IN_BY: f32 = 8.0;
/// **How many frames after a monster came it is long gone**: the next comes 490 after it.
const A_GREEN_EYED_MONSTER_IS_GONE_AFTER: i64 = 600;
/// **How far above the floor the run waits under the boss**: the low line.
const THE_GREEN_EYED_MONSTER_IS_WAITED_FOR_ABOVE_THE_FLOOR: f32 = 32.0;
/// **When the run is led up, how fast and how high**: from as the monster comes within 80 of it, at 1.0 a frame, to 80
/// below the boss.
///
/// **From the monster reaching the run and not from a frame after it came**, which was tried: it reaches the run only
/// once it has gone down the field from wall to wall, some 170 frames after it came at `hits2/s2-c3-f3492`, and then
/// goes back and forth through where the run is some 40 either side; led up from 40 frames after it came at 2.0 a
/// frame, the run was back down beside the trail by then and stood there as the monster went back and forth through it,
/// hit ten times at `90` to `117` across, `375` to `393` down (2026-10-07). And at 1.0 a frame, half the 2.0 tried, so
/// that the climb lasts while the monster is at the run.
const THE_GREEN_EYED_MONSTER_REACHES_THE_RUN_WITHIN: f32 = 80.0;
const THE_GREEN_EYED_MONSTER_IS_LED_UP_BY: f32 = 1.0;
const THE_GREEN_EYED_MONSTER_IS_LED_UP_TO: f32 = 80.0;
/// **How far aside of the line it was led up the run comes back down**: clear of the trail laid under it.
const BESIDE_THE_GREEN_EYED_MONSTERS_TRAIL_BY: f32 = 40.0;
/// Whether [`after_the_green_eyed_monster`] saw a monster the frame before, and when the last one came and reached the
/// run.
struct GreenEyedMonster {
    came: Option<i64>,
    reached: Option<i64>,
    there: bool,
}
static THE_GREEN_EYED_MONSTER: Mutex<GreenEyedMonster> = Mutex::new(GreenEyedMonster {
    came: None,
    reached: None,
    there: false,
});

/// **Where th11's `恨霊「スプリーンイーター」` has the run go**: as [`jade_of_the_loathsome_river`] does, on at whichever
/// steady pace the card's own fire played on leaves unhit longest — its rice setting off at where the run was as the
/// ring that laid it came, and its spirits going on as they go — and of those the one nearest the floor's middle.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/5): `開幕画面最下段に陣取り`, and for Easy and Normal
/// `早めに抜けないと爆発に巻き込まれる。だからと言って急いで抜けようとして怨霊に当たらないように注意`. Read off its
/// field (`--trace-the-field-in=127` from `hits4/s5-c7-f14437`, scratchpad sets5.py and setoff127.py, 2026-10-08):
/// eight spirits come out of one place and spread some 165 round where the run stands, go round it and close on that
/// place, meeting there 270 frames after they spread (at 14,065 round `-38,335`, met at `-30,332` by 14,335), a ring
/// every 400 frames and another 152 after each; the rice they lay stands until [`SPLEEN_EATERS_RICE_SETS_OFF_AT`]
/// frames old — 1,858 of 1,858 at 187 or 188 — and then goes straight at where the run was as its ring came, 4
/// degrees off it at the median and 14 off where the run is then, speeding up by [`SPLEEN_EATERS_RICE_SPEEDS_UP_BY`]
/// a frame from [`A_GRAIN_SETS_OFF_AT`] to [`SPLEEN_EATERS_RICE_GOES_AT_MOST`].
pub(crate) fn the_spleen_eaters_rice(
    frame: &Frame,
    card_frames: Option<i32>,
    aside: f32,
) -> (f32, f32) {
    looked_ahead(frame, card_frames, aside, &SPLEEN_EATER, &THE_SPLEEN_EATER)
}
// **The place told is not kept 160 below the field's top either**, for the wiki's `N以上は抜けた後あまり遠くまで行かない`
// after whole Normals 23 and 25 were hit carried up to beside the boss at `184.0,121.1` and `164.1,36.6`
// (`hits-n23/s5-c7-f15757`, `hits-n25/s5-c7-f15592`, 2026-10-09), which was tried: the run was shut in along that line
// instead and hit at `171.1,198.3`, `-55.8,192.5`, `-114.9,191.4` and `-97.2,191.1` on those two chapters, and the
// real case's took 2 where let up it took 1 — 2, 2, 3, 3, 3, 2 and 0 over the first seven of the sixteen kept.

const THE_SPLEEN_EATER: LookedAhead = LookedAhead {
    sets_off: SetsOff::AtWhereTheRunWas,
    paces: &SPLEEN_EATER_PACES,
    looks_ahead: RIVER_LOOKS_AHEAD,
    unhit_enough: SPLEEN_EATER_IS_UNHIT_ENOUGH_FOR,
    turns_once: Some(SPLEEN_EATER_TURNS_ONCE),
    home: Home::AmongTheRiceAtRest {
        sets_off_at: SPLEEN_EATERS_RICE_SETS_OFF_AT,
        above: SPLEEN_EATER_HOME_ABOVE_THE_FLOOR,
    },
    home_is_worth: RIVER_HOME_IS_WORTH,
    an_edge_is_worth: RIVER_AN_EDGE_IS_WORTH,
    keeps_a_box_wider_by: RIVER_KEEPS_A_BOX_WIDER_BY,
    aside_share: 0.0,
};
// **Its home among the rice at rest**, the wiki's `「設置された弾列の外側には行かない」みたいな心構えで`: with its home at
// the floor's middle, the run went out past the rows laid to the top right and was hit there by the arcs coming in at
// `164.1,36.6` on whole Normal 25 (`hits-n25/s5-c7-f15592`, its pictures, 2026-10-09). The sixteen chapters kept for
// the card took 3 0 1 1 1 0 0 1 0 0 0 1 0 2 1 0 so at `--pace=16`, 11 hits in 8 of them, where at the floor's middle 1
// 1 3 3 1 2 0 1 0 0 0 1 0 0 0 0, 13 in 8 — about as many, and whole run 22's three chapters 1, 1 and 1 where they took
// 3, 3 and 1.
//
// **Its home is not on the round 16 past the burst's reach nearest the run**, for the wiki's
// `爆発の縁をなぞるように回転するといい` and `N以上は抜けた後あまり遠くまで行かない`, which was tried after whole run 22 was
// hit three times in the bottom right corner (`hits-n22/s5-c7`, 2026-10-09): the run, drawn to the burst, waited where
// the rice of the ring before came in and was hit at `15.5,432.0`, `-2.1,432.0` and `8.3,432.0` on all three of run 22's
// chapters, and the eight kept before took 0, 0, 1, 2, 3, 1, 1 and 8 where its home at the floor's middle took 1, 0, 0,
// 0, 1, 0, 0 and 0.
// **Its paces let take the run as high as the field goes, and not kept 192 above the floor** for the wiki's
// `開幕画面最下段に陣取り`, which was tried: late in the card with no pace unhit throughout, the one unhit longest was
// chosen again and again upward and the run was carried up the right wall to the top and hit beside the boss at
// `-28.4,41.7` (the real case a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb, orb-real 1791509042, and whole
// run 19 at `158.5,32.0`) — but kept down, it was shut in on the floor instead, at its corners: 1, 0, 1, 0, 1, 0, 1, 0
// over eight chapters kept for the card where it was let up took 1, 0, 0, 0, 1, 0, 0, 0, and on the real case's own
// chapter hit at `177.0,432.0` rather than at the top (2026-10-09).
static SPLEEN_EATER: Mutex<River> = Mutex::new(River::NONE);
/// **The paces tried, a frame**: the river's, and the whole step to get out from under a ring's middle before its
/// rice comes in — the wiki's `早めに抜けないと`.
const SPLEEN_EATER_PACES: [f32; 4] = [0.4, 1.0, 2.0, 4.5];
/// **How many frames unhit a pace is no better for more of**: every one of the river's `160` it is played on.
///
/// **And not fewer**, though with the rice coming in from every side no one pace was unhit for 160 frames from 157
/// frames before a hit on, and the one chosen each time was the one hit last — carried on into the ring as its unhit
/// frames ran down by four each choice, `161` to `2`, and hit at `133.2,271.2` on `hits-n3/s5-c7-f14090` (2026-10-09):
/// with only `60` played on, the nine chapters kept for the card took 3, 1, 11, 0, 1, 1, 0, 1 and 3, and with `60`
/// counted of the `160`, 3, 2, 4 and 5 on the first four, where all `160` took 7, —, 0, 0, 2, 1, 0, 1 and 0.
const SPLEEN_EATER_IS_UNHIT_ENOUGH_FOR: i32 = RIVER_LOOKS_AHEAD + 1;
/// **After how many frames the best paces are each turned to every other once, and how many of them**: a turn after
/// 40 frames, from the best 4 — the way out of a ring coming in from every side being a turn, which no one pace makes.
const SPLEEN_EATER_TURNS_ONCE: (i32, usize) = (40, 4);
/// **Where the run would rather be with no rice at rest**: low in the middle, `27` above the floor.
const SPLEEN_EATER_HOME_ABOVE_THE_FLOOR: f32 = 27.0;
/// **The age a grain of the card's rice sets off at**: 1,641 of 1,858 set off at `187` frames old and the rest at
/// `188`, in the traced field.
const SPLEEN_EATERS_RICE_SETS_OFF_AT: i32 = 187;
/// **How fast a grain goes on the frame it sets off, and how much faster each frame after**: `0.058` on its first
/// frame, `0.217` twenty on, `0.550` sixty on and `1.000` from 120 on at the median, so [`A_GRAIN_SETS_OFF_AT`] and a
/// 120th more a frame.
const SPLEEN_EATERS_RICE_SPEEDS_UP_BY: f32 = 1.0 / 120.0;
/// **How fast a grain set off speeding up goes before its first frame's more**: `0.05` for the rice of
/// `恨霊「スプリーンイーター」`.
const A_GRAIN_SETS_OFF_AT: f32 = 0.05;
const SPLEEN_EATERS_RICE_GOES_AT_MOST: f32 = 1.0;
/// **How many frames a grain speeds up over**, its age past setting off up to which it is played on speeding up.
const SPLEEN_EATERS_RICE_SPEEDS_UP_OVER: i32 = 120;
/// **How long before a grain is laid its ring came**, which is where the run was then: the grain goes at where the run
/// was 270 frames before it sets off, at 187 frames old — so 83 before it was laid.
const SPLEEN_EATERS_RING_CAME_BEFORE_ITS_RICE_BY: i32 = 83;

/// **The burst of `恨霊「スプリーンイーター」`'s ring to come**, for [`looked_ahead`] to play on: where its spirits are
/// gathered round, on the card's next frame a burst comes on, for as long as one lasts — and none for any other card,
/// or with no spirit up. **One is enough**: those of a ring are one body at the place they meet by the time they do
/// — at the card's frame 440 of `hits4/s5-c7-f14437`, one at `72,381`, 17 frames before the burst there.
///
/// **Where its spirits are gathered round, and not where the run was as its ring came**, which is where the rice goes:
/// the bursts of a card traced came 20 to 45 off where the run was 270 of the card's frames before each
/// (`hits4/s5-c7-f14437`, scratchpad burstat127.py, 2026-10-08). And it is played on at all because the bodies' own
/// growing comes too late to get out of: played on with them alone, the run stood `13` from where they met and was
/// hit at `-7.3,377.6` as they burst, the burst out to `41` in 8 frames.
fn the_spleen_eaters_burst_to_come(
    frame: &Frame,
    card: &LookedAhead,
    mine: (f32, f32),
    now: i32,
) -> Option<Fire> {
    if card.sets_off != SetsOff::AtWhereTheRunWas {
        return None;
    }
    let middle = where_the_spleen_eaters_spirits_gather(frame)?;
    // The first burst not yet over.
    let past = now - SPLEEN_EATERS_BURST_FIRST_AT - SPLEEN_EATERS_BURST_LASTS;
    let over = match past > 0 {
        true => (past + SPLEEN_EATERS_BURST_EVERY - 1) / SPLEEN_EATERS_BURST_EVERY,
        false => 0,
    };
    let until = SPLEEN_EATERS_BURST_FIRST_AT + over * SPLEEN_EATERS_BURST_EVERY - now;
    if until > card.looks_ahead {
        return None;
    }
    let lasts = SPLEEN_EATERS_BURST_LASTS + until.min(0);
    Some(Fire {
        at: middle,
        half: (
            SPLEEN_EATERS_BURST_REACHES + mine.0,
            SPLEEN_EATERS_BURST_REACHES + mine.1,
        ),
        moves: Moves::BurstsIn(until.max(0), lasts.max(0)),
    })
}
/// **Where the spirits of `恨霊「スプリーンイーター」` gather to burst**: the middle of the ring they go round on; with no
/// spirit out, nowhere.
fn where_the_spleen_eaters_spirits_gather(frame: &Frame) -> Option<(f32, f32)> {
    let spirits = the_spleen_eaters_spirits(frame);
    (!spirits.is_empty()).then(|| the_middle_of_the_ring_through(&spirits))
}
/// **Where the spirits of `恨霊「スプリーンイーター」` are**: every solid body, the boss's own taken out — the bodies a
/// th11 field carries at its top middle whatever is up being none of them.
fn the_spleen_eaters_spirits(frame: &Frame) -> Vec<(f32, f32)> {
    let boss = frame.boss;
    frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Body { solid: true }))
        .map(|hazard| hazard.at)
        .filter(|at| boss.is_none_or(|boss| distance(*at, boss) > THE_SPLEEN_EATERS_BOSS_IS_WITHIN))
        .collect()
}

/// **Where `恨霊「スプリーンイーター」` is waited out the partner's way**, Reimu with Yukari: against the side wall nearer the
/// run, [`THROUGH_THE_GAP_ABOVE_THE_FLOOR`] up — the wiki's 「紫の場合は開始直後に端で待機し、怨霊が迫ってきたらスキマを
/// 使えば時間切れまで粘れる」 (wikiwiki.jp/thk/地/5, read whole 2026-10-10), the gap gone through as the spirits close
/// in (see [`the_spleen_eaters_spirits_close_in`]) leaving the run against the other wall, where it waits for the next
/// ring.
pub(crate) fn against_the_nearer_side_wall(frame: &Frame) -> (f32, f32) {
    let (left, top, width, height) = frame.field;
    let x = match frame.at.0 < left + width / 2.0 {
        true => left,
        false => left + width,
    };
    (x, top + height - THROUGH_THE_GAP_ABOVE_THE_FLOOR)
}
/// How far above the floor the run waits against a wall for the gap: a ring's reach above it, so that a ring round it is
/// not half under the floor.
const THROUGH_THE_GAP_ABOVE_THE_FLOOR: f32 = 120.0;

/// **Whether a ring of `恨霊「スプリーンイーター」`'s spirits is closing in on the run**: one within
/// [`THE_SPIRITS_CLOSE_IN_WITHIN`] of it — a ring spreading some 165 round where the run stands and closing on that place
/// over 270 frames (see [`the_spleen_eaters_rice`]), so that what it closes on, and the rice that follows it, is where
/// the run was before the gap.
pub(crate) fn the_spleen_eaters_spirits_close_in(frame: &Frame) -> bool {
    the_spleen_eaters_spirits(frame)
        .into_iter()
        .any(|spirit| distance(spirit, frame.at) < THE_SPIRITS_CLOSE_IN_WITHIN)
}
///
/// **And not as near as `64`**, which was tried: a ring round the run against a wall goes half out past it, and closed
/// to 64 its spirits out there stood on the gap's own way, the run carried into two of them at `-203.7,332.8` with
/// nowhere open on `hits-n19/s5-c7-f15914` (2026-10-10).
const THE_SPIRITS_CLOSE_IN_WITHIN: f32 = 112.0;
/// **How near the boss a body is taken for hers**.
const THE_SPLEEN_EATERS_BOSS_IS_WITHIN: f32 = 16.0;

/// **The middle of the circle the places `on` lie round**, fitted through them where three or more are not in a line,
/// and the middle of them otherwise.
///
/// **Fitted and not their middle**, which it was: with some of a ring's spirits shot down, the middle of those left
/// is off to their side, and the burst played on there was `40` off the one at `24,391` that hit the run at
/// `29.9,432.0` on `hits10/s5-c7-f15130`, the way having it unhit for 160 frames standing there 7 frames before
/// (2026-10-09).
fn the_middle_of_the_ring_through(on: &[(f32, f32)]) -> (f32, f32) {
    let count = on.len() as f32;
    let (mx, my) = (
        on.iter().map(|at| at.0).sum::<f32>() / count,
        on.iter().map(|at| at.1).sum::<f32>() / count,
    );
    if on.len() < 3 {
        return (mx, my);
    }
    // The circle through them least off, `(x - a)² + (y - b)² = r²` taken about their middle.
    let (mut suu, mut svv, mut suv, mut suuu, mut svvv, mut suvv, mut svuu) =
        (0.0f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(x, y) in on {
        let (u, v) = (x - mx, y - my);
        suu += u * u;
        svv += v * v;
        suv += u * v;
        suuu += u * u * u;
        svvv += v * v * v;
        suvv += u * v * v;
        svuu += v * u * u;
    }
    let determinant = suu * svv - suv * suv;
    if determinant <= THE_RING_IS_IN_A_LINE_UNDER * suu * svv {
        return (mx, my);
    }
    let (ru, rv) = ((suuu + suvv) / 2.0, (svvv + svuu) / 2.0);
    (
        mx + (ru * svv - rv * suv) / determinant,
        my + (rv * suu - ru * suv) / determinant,
    )
}
/// **How little of the spread of the places across and down is left once what they share is taken out, for them to be
/// in a line**, which no circle goes through: a hundredth.
const THE_RING_IS_IN_A_LINE_UNDER: f32 = 0.01;
/// **The card's frame its first burst comes on, how many apart they come, how far out a burst reaches and how long it
/// lasts**: on the card's frames 457, 857, 1,257, 1,657 and 2,457 of one launch and 457 on each of six attempts of
/// another, out from `13` to `41` over 8 frames and gone some 12 after it came (scratchpad bursts127.py, 2026-10-08).
/// **Played on out to `48`**, the length of its four boxes' arms — `45.7`, `47.0` along them at the hits read — and not
/// the `42` it reaches across: kept off by `42`, the run was hit by bursts at `-41.7,409.8`, `-22.6,406.6` and
/// `8.1,432.0`, 44 to 46 off their middles, eleven times on `hits-n4/s5-c7-f13922` alone; out to `48` with its middle
/// fitted round its spirits — see [`the_middle_of_the_ring_through`] — the nine chapters kept for the card took 1, 0, 0,
/// 0, 1, 0, 0, 0 and 0 where out to `42` they took 0, 3, 4, 1, 0, 0, 0, 1 and 1 (2026-10-09). Out to `48` with the
/// burst at its spirits' plain middle it had taken more, 5, 3 and 1 on the first three.
const SPLEEN_EATERS_BURST_FIRST_AT: i32 = 457;
const SPLEEN_EATERS_BURST_EVERY: i32 = 400;
const SPLEEN_EATERS_BURST_REACHES: f32 = 48.0;
const SPLEEN_EATERS_BURST_LASTS: i32 = 14;
// **Its fire played on, and not round a circle on the floor, nor out of each ring, nor low in the middle and out of
// its burst's way across the floor** — which last took 1, 3, 3 and 4 on the first four of the seven below, the burst
// got out of and the rice coming in on the run all the same (2026-10-08).
// Round a circle 60 across with its foot on the floor in the middle, half way round each 400 frames so that the rings
// came by turns at two places across it — the wiki's `爆発の縁をなぞるように回転` — took 0, 3, 0, 3, 1, 4 and 2 on the
// first chapter kept for card 127 off each of seven runs (`hits4` to `hits10`), hit where the spirits closing on it
// pushed the run up off it, as high as `165.6,285.0` and `-19.8,69.8`; 30 off the floor it took more; with a move
// landing over its top judged hit, 0, 1, 4, 3, 2, 3 and 0, as many; and once round each 400 frames, the rings coming at
// one place — the wiki's Lunatic `同じ箇所で霊を発生させ続けるように回転` — 0, 2, 4, 3 and 2 on the first five. Just
// outside each ring through the gap between its spirits nearest the run took six and three on the first two, the
// ring's middle on the floor and its arms closing on the run with nothing open up through them (2026-10-08). Low in
// the middle alone took 0, 1, 0, 3, 1, 1 and 1 on the same seven. **Nor round each ring's own middle**, tried
// three ways over the six chapters kept as `hits4` to `hits9` on 2026-10-07 — waited for in that ring's middle until
// 280 and gone round from the floor on its left at 4 degrees a frame, 3, 7, 8, 1, 9 and 5 hits; at 1 degree a frame,
// 6, 2, 0, 1, 8 and 0; and following the oldest ring whose arms were not yet through, leaving its middle at 190
// straight up, 3, 7 and 3 on the first three — the run in the ring as its arms came in every time.

/// **Where across th11's `核熱「ニュークリアフュージョン」` is waited out at the height `y`**: on the line out of the ring's
/// middle half way between two of its suns still coming, the line nearest the run of those pointing down into the
/// field — and where no sun is coming, in front of the boss.
///
/// **Because the suns come out of one point and go straight out from it.** Read off `--trace-the-field-in=143` over three
/// stage 6 practices of 2026-10-07: each ring is eight bodies of radius 88 born together at `0,128` — thirty-two above
/// the boss at `0,160` — 45 degrees apart, going out some 1.2 a frame, and turned its own way each time (`0`, `17`, `19`,
/// `22`, `23` and `42` degrees for the first of them). Down by the floor, 280 out, two suns' middles are 220 apart, which
/// leaves some 40 between their edges on the line half way between them, and that line is the same all the way out. The
/// rules' own dodging stood on the floor under one sun and was hit as it came through, six times a practice, and
/// waiting in the ring's middle was hit by the next ring being born there.
pub(crate) fn between_the_suns(frame: &Frame, y: f32) -> Option<f32> {
    let (boss_x, boss_y) = frame.boss?;
    let middle = (boss_x, boss_y - SUNS_BORN_ABOVE_THE_BOSS);
    let out_to = (y - middle.1).max(0.0);
    let mut suns: Vec<(f32, f32)> = Vec::new();
    for hazard in &frame.hazards {
        let solid = matches!(hazard.kind, Kind::Body { solid: true });
        if !solid || hazard.half.0.max(hazard.half.1) < A_SUN_REACHES_PAST {
            continue;
        }
        if suns.contains(&hazard.at) {
            continue;
        }
        suns.push(hazard.at);
    }
    // **Every ring still coming and not the furthest out alone**, which was tried: a gap between a sun of one ring and one
    // of the ring born after it reads as no gap, but held to the ring furthest out the run was hit fourteen times in the
    // chapter kept as `hits/s6-c5-f10070`, most of them at the left wall, where with every ring it was hit once
    // (2026-10-07).
    let coming: Vec<(f32, f32)> = suns
        .into_iter()
        .filter(|at| (at.0 - middle.0).hypot(at.1 - middle.1) < out_to)
        .collect();
    let mut angles: Vec<f32> = coming
        .iter()
        .map(|at| (at.1 - middle.1).atan2(at.0 - middle.0))
        .collect();
    let mut held = BETWEEN_THE_SUNS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // **And in front of her where no ring is coming**, the wiki's way for Normal: `恒星は自機狙いなので恒星が放たれる度
    // に正面→恒星を横に避ける→正面に戻るの繰り返し` — the next ring aimed at the run from where it stands, so that the gap
    // it goes into is the one beside her front and not one nearer a wall each ring.
    // The six chapters kept for the card, `hits/s6-c5-f10070` to `hits9/s6-c5-f8984`, took none so at `--pace=16`, as
    // they did held at the last place (2026-10-10).
    if angles.len() < 2 {
        *held = Some(boss_x);
        return *held;
    }
    angles.sort_by(f32::total_cmp);
    let first = angles[0] + std::f32::consts::TAU;
    angles.push(first);
    let (left, _, width, _) = frame.field;
    // **The gap between the two suns whose lines the run already stands between**, and not the gap nearest it: a gap on
    // the far side of a sun's line is a way across the line that sun is coming down, and from `hits/s6-c5-f10070` the
    // run went from `104` to `63` along the floor for the gap at `-37` across the line of a sun coming down at `84` and
    // was hit by it, twice in the same place (2026-10-07). **And the nearest gap the run gets to across each sun's line
    // before that sun gets there, where that one's line leaves the field before the run's height, and not the wall on
    // its side**: held at the wall the same chapter took five hits there, at `-140` to `-182`, the suns' edges reaching
    // the wall before the gap did, and taken to the nearest gap whatever lay between, three — the first of them the
    // crossing above again. Where no gap is got to in time, the run stays where it is.
    //
    // **And gaps and not the place across shut last**, which was tried as card 147's flares are waited out: the corners
    // read as shut last, and the run went to the left wall and was hit there, at `-184,425`, ten times running.
    // **Where the run is and not where it is waited for**: held on the floor at `y = 432` under the place at `409`, the
    // run at `123,432` in `hits/s6-c5-f10070` stood in the gap pointing down at `2` and read at `409` as across the line
    // of the sun coming down at `119`, its gap there out of the field — so it was told to stay, and the rules' dodging
    // carried it along the floor to `184` and the sun came through it there (2026-10-07, `--log=verbose`).
    let the_runs = (frame.at.1 - middle.1).atan2(frame.at.0 - middle.0);
    let the_runs = if the_runs < angles[0] {
        the_runs + std::f32::consts::TAU
    } else {
        the_runs
    };
    let on_the_runs_height = |angle: f32| {
        (angle.sin() > A_LINE_POINTS_DOWN_PAST)
            .then_some(middle.0 + out_to * angle.cos() / angle.sin())
            .filter(|x| (left..=left + width).contains(x))
    };
    let gaps = || {
        angles
            .windows(2)
            .map(|pair| (pair, (pair[0] + pair[1]) / 2.0))
    };
    let from = frame.at.0;
    // Whether the run gets across every sun's line between it and `x` before that sun's edge is on the line there.
    let got_to_in_time = |x: f32| {
        coming.iter().all(|sun| {
            let angle = (sun.1 - middle.1).atan2(sun.0 - middle.0);
            let Some(line) = on_the_runs_height(angle) else {
                return true;
            };
            if !(from.min(x)..=from.max(x)).contains(&line) {
                return true;
            }
            let the_runs = (line - from).abs() / A_RUN_STEPS_BY;
            let the_suns = ((line - sun.0).hypot(y - sun.1) - A_SUNS_REACH - A_SUN_IS_KEPT_FROM_BY)
                / A_SUN_GOES_OUT_BY;
            the_runs < the_suns
        })
    };
    let across = gaps()
        .find(|(pair, _)| (pair[0]..pair[1]).contains(&the_runs))
        .and_then(|(_, angle)| on_the_runs_height(angle))
        .or_else(|| {
            gaps()
                .filter_map(|(_, angle)| on_the_runs_height(angle))
                .filter(|x| got_to_in_time(*x))
                .min_by(|a, b| (a - from).abs().total_cmp(&(b - from).abs()))
        })
        .or(Some(from));
    if across.is_some() {
        *held = across;
    }
    *held
}
/// **How far above the boss a ring of suns is born**: `0,128` against her `0,160`, every ring of three practices.
const SUNS_BORN_ABOVE_THE_BOSS: f32 = 32.0;
/// **How far a body has to reach to be a sun**: the suns reach 87 from their middles, the boss 23.
const A_SUN_REACHES_PAST: f32 = 40.0;
/// **How far a sun reaches, how fast it goes out and how far past its edge the run keeps**: 87, some 1.2 a frame, and
/// the run's own half with the rules' slack.
const A_SUNS_REACH: f32 = 87.0;
const A_SUN_GOES_OUT_BY: f32 = 1.2;
const A_SUN_IS_KEPT_FROM_BY: f32 = 6.0;
/// **How steeply a line has to point down to be one the run can stand on**, its sine: a line nearer across than this
/// meets the run's height outside the field.
const A_LINE_POINTS_DOWN_PAST: f32 = 0.3;
/// The place across [`between_the_suns`] last found, for the frames no sun is coming on.
static BETWEEN_THE_SUNS: Mutex<Option<f32>> = Mutex::new(None);

/// **Where across th11's `爆符「メガフレア」` is waited out on the floor at `y`**: the place across the field the flares
/// come down on last, of those the run can reach before one comes down on its way — each flare carried down at the
/// pace it falls and shrinking as it does, and a place it covers while it falls past the run's height being shut
/// from the frame its edge first reaches the run to the frame it has gone.
///
/// **Because the flares are the size they are now nowhere near the floor.** Read off `--trace-the-field-in=147` over a
/// stage 6 practice of 2026-10-07: a flare is born every twenty frames somewhere across the top of the field, swells to
/// a reach of 80 to 92 and falls straight down at 2.5 a frame, giving up some 0.25 of its reach for each unit it falls,
/// to 16 — so it crosses the floor some 30 to 50 across. The rules' own fear carried each flare down at the reach it
/// had higher up, read the floor as shut, and stood there: seven hits on the floor in one practice, each by a flare 24
/// to 48 across with the run inside it.
///
/// **And by when a place is shut and not by how far it is from where flares land**, which was the first way of it: a
/// flare's reach where it lands is not what hits — it covers the run while it passes the run's height, its middle some
/// way above and its reach the larger for it — and a place a flare lands on far off is no place to stand if a flare
/// coming down nearer shuts it, or the way to it, first. Picked up from `hits/s6-c6-f14624` (2026-10-07), that way went
/// round the same four hits over fifty attempts: the run at the left wall under a flare of reach 83 coming down at
/// `-161`, out from under it to `-130` and hit at the edge of that same flare with 44 of reach left at `y = 397`.
pub(crate) fn between_the_flares(frame: &Frame, y: f32) -> Option<f32> {
    let mut flares: Vec<((f32, f32), f32, f32)> = Vec::new();
    for hazard in &frame.hazards {
        let solid = matches!(hazard.kind, Kind::Body { solid: true });
        let reach = hazard.half.0.max(hazard.half.1);
        if !solid || reach < A_FLARE_REACHES_PAST || hazard.at.1 > y + reach {
            continue;
        }
        if flares.iter().any(|(at, ..)| *at == hazard.at) {
            continue;
        }
        flares.push((hazard.at, reach, hazard.grows));
    }
    if flares.is_empty() {
        return None;
    }
    // The first frame a flare covers `x` at the run's height, each flare followed until it has gone past, and no
    // later than the frames weighed: a place shut that far off is as good as open.
    let shut_at = |x: f32| -> f32 {
        let mut soonest = A_FLARE_IS_WATCHED_FOR;
        for (at, reach, grows) in &flares {
            // **Swelled to its whole reach first where it is still swelling**, and only then given up as it falls.
            let swelled_by = match *grows > 0.0 {
                true => ((A_FLARE_SWELLS_TO - reach) / grows).max(0.0),
                false => 0.0,
            };
            let mut frame_ahead = 0.0;
            loop {
                let fallen = frame_ahead * A_FLARE_FALLS_BY;
                let middle = at.1 + fallen;
                let reach = match frame_ahead < swelled_by {
                    true => reach + grows * frame_ahead,
                    false => {
                        let from = reach.max(A_FLARE_SWELLS_TO * f32::from(swelled_by > 0.0));
                        let shrinking = (frame_ahead - swelled_by) * A_FLARE_FALLS_BY;
                        (from - A_FLARE_SHRINKS_BY * shrinking).max(A_FLARE_LANDS_AT_LEAST)
                    }
                };
                if middle - reach > y + A_FLARE_IS_KEPT_FROM_BY || frame_ahead >= soonest {
                    break;
                }
                if (x - at.0).hypot(y - middle) < reach + A_FLARE_IS_KEPT_FROM_BY {
                    soonest = frame_ahead;
                    break;
                }
                frame_ahead += 1.0;
            }
        }
        soonest
    };
    let (left, _, width, _) = frame.field;
    let places: Vec<(f32, f32)> = (0..=(width / A_FLARE_IS_LOOKED_AT_EVERY) as i32)
        .map(|step| left + step as f32 * A_FLARE_IS_LOOKED_AT_EVERY)
        .map(|x| (x, shut_at(x)))
        .collect();
    let from = frame.at.0;
    // A place is reached only where nothing shuts any place on the way to it before the run is past it.
    // With no frames to spare — see [`A_RUN_STEPS_BY`].
    let reached = |x: f32| -> bool {
        places
            .iter()
            .filter(|(on, _)| {
                (*on - from) * (x - from) >= 0.0 && (*on - from).abs() <= (x - from).abs()
            })
            .all(|(on, shut)| (*on - from).abs() / A_RUN_STEPS_BY < *shut)
    };
    // **A place weighed by the room about it as well as by when it is shut**: the mean of when every place within
    // [`A_FLARES_ROOM_IS_WEIGHED_ACROSS`] of it is shut, a place past either wall counted as shut already. Because a
    // flare not yet born comes down where nothing said one would, and the run is then only as safe as the room it
    // has to step aside in: weighed by its own place alone, the run on the chapter the real case
    // a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb kept as `s6-c7-f15980` (session `1791795267`) stood
    // some 300 frames against the right wall at `184,432` with flares at `102` and `138` crossing the floor beside
    // it, and a flare born at `154,233` 62 frames before shut the corner with no way out, hit there six times in six;
    // and once between two flares at `-138` and `-72` the same way.
    let reach_of_the_room = (A_FLARES_ROOM_IS_WEIGHED_ACROSS / A_FLARE_IS_LOOKED_AT_EVERY) as i32;
    let room = |at: usize| -> f32 {
        let shut_at: f32 = (-reach_of_the_room..=reach_of_the_room)
            .map(|step| {
                usize::try_from(at as i32 + step)
                    .ok()
                    .and_then(|on| places.get(on))
                    .map_or(0.0, |(_, shut)| *shut)
            })
            .sum();
        shut_at / (2 * reach_of_the_room + 1) as f32
    };
    let best = |reachable_only: bool| {
        places
            .iter()
            .enumerate()
            .filter(|(_, (x, _))| !reachable_only || reached(*x))
            .map(|(at, (x, shut))| {
                (
                    *x,
                    (shut + room(at)) / 2.0 - (x - from).abs() * A_FLARE_STEP_COSTS,
                )
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(x, _)| x)
    };
    // **And where the run's own place is shut already, the place shut last whatever lies between**: nothing is
    // reached then, and standing is the one thing sure to be hit.
    best(true).or_else(|| best(false))
}
/// **How far a body has to reach to be a flare**, the boss reaching 23 and a flare 16 even on the floor.
const A_FLARE_REACHES_PAST: f32 = 15.0;
/// **How far a flare falls a frame**, 50 every twenty frames.
const A_FLARE_FALLS_BY: f32 = 2.5;
/// **How much of its reach a flare gives up for each unit it falls**, and the least it lands at: `-161` from a reach
/// of 83 at `y = 247` to 44 at `397`, `-96` from 53 at `287` to 16 at `432`.
const A_FLARE_SHRINKS_BY: f32 = 0.25;
/// The least reach a flare lands at, the smallest one read at the floor.
const A_FLARE_LANDS_AT_LEAST: f32 = 16.0;
/// **The reach a flare swells to before it gives any up**: from 21 to 87 in 12 frames and on to 89.8 at `4.4,205.9`,
/// read off the field traced on the chapter the real case kept as `s6-c6-f15797` (session `1791577212`, 2026-10-10).
/// **Because one born low swells near the floor**: carried down from the reach it had and given up from there, the
/// flare at `72.8,436.3` was read as 16 across at the floor where it reached 35.5, and the run stood under it and was
/// hit at `35.5,432.0` on the chapter the real case `a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb` kept
/// as `s6-c6-f15597` (session `1791589071`, 2026-10-10). Of the six chapters kept for the card, given up from the reach
/// it had, that one was hit twice on its real case, `s6-c6-f15797` twice and `hits/s6-c6-f14624`, `f15043` and
/// `f15108` not at all at `--pace=16` (`s6-c6-f15248` measured only with frames to spare, see [`A_RUN_STEPS_BY`]); swelled
/// first, none of the six was hit, all of them through to chapter 7 (2026-10-10).
const A_FLARE_SWELLS_TO: f32 = 90.0;

/// How many frames on a place is first shut, asked of each place.
type ShutAt<'a> = Box<dyn Fn((f32, f32)) -> f32 + 'a>;

/// **How many frames on a place is first shut by th11's `復燃「恋の埋火」`**, at most [`A_HEART_IS_WATCHED_FOR`]: a heart
/// followed along its step and off the walls it comes back from coming within [`A_HEART_IS_KEPT_FROM_BY`] of it, or a
/// trail's bullet going on straight within [`A_TRAIL_IS_KEPT_FROM_BY`]; with no heart out, nothing to reckon.
///
/// Read off `--trace-the-field-in=171` over a pickup of `hits-extra7/s7-c13-f35551` (2026-10-08): the boss throws two
/// hearts at a time, a pair aimed at the run and a pair below her to either side by turns, each `10` across going 2 a
/// frame and coming back off all four walls of the field, and each leaves some twenty-five bullets `6` across behind it
/// going 0.35 a frame, neither changing size (2026-10-09); the pairs pile up from two hearts to eight in some 960
/// frames, when every one of them goes and the card begins again. The wiki has a heart go off the field after ten
/// reflections (「ハート弾は10回反射したら画面外にいく」), which this does not count: one is played on back off every
/// wall it meets in the frames weighed.
fn the_embers_shut_at(frame: &Frame) -> Option<ShutAt<'_>> {
    let bullets = || {
        frame
            .hazards
            .iter()
            .filter(|hazard| matches!(hazard.kind, Kind::Bullet { hitting: true, .. }))
    };
    let hearts: Vec<((f32, f32), (f32, f32))> = bullets()
        .filter(|hazard| hazard.half.0.max(hazard.half.1) >= A_HEART_IS_HALF_AT_LEAST)
        .filter_map(|hazard| hazard.step.map(|step| (hazard.at, step)))
        .collect();
    if hearts.is_empty() {
        return None;
    }
    let trails: Vec<((f32, f32), (f32, f32))> = bullets()
        .filter(|hazard| hazard.half.0.max(hazard.half.1) < A_HEART_IS_HALF_AT_LEAST)
        .map(|hazard| (hazard.at, hazard.step.unwrap_or((0.0, 0.0))))
        .collect();
    // **The first frame a trail's bullet going on straight comes within reach of `place`**, and none past it.
    let a_trail_reaches =
        |(at, step): ((f32, f32), (f32, f32)), place: (f32, f32)| -> Option<f32> {
            let (dx, dy) = (at.0 - place.0, at.1 - place.1);
            let reach = A_TRAIL_IS_KEPT_FROM_BY;
            let c = dx * dx + dy * dy - reach * reach;
            if c < 0.0 {
                return Some(0.0);
            }
            let a = step.0 * step.0 + step.1 * step.1;
            // Too far off to come within reach in the frames weighed, which is most of them.
            let goes = a.sqrt() * A_HEART_IS_WATCHED_FOR + reach;
            if dx * dx + dy * dy > goes * goes {
                return None;
            }
            let b = 2.0 * (dx * step.0 + dy * step.1);
            let roots = b * b - 4.0 * a * c;
            if a < f32::EPSILON || roots < 0.0 {
                return None;
            }
            let first = (-b - roots.sqrt()) / (2.0 * a);
            (first >= 0.0).then_some(first)
        };
    let (left, top, width, height) = frame.field;
    let (right, floor) = (left + width, top + height);
    // Where a heart is `frames` on, come back off every wall it has met on the way.
    let carried = |at: f32, step: f32, low: f32, high: f32, frames: f32| -> f32 {
        let span = high - low;
        let gone = (at - low + step * frames).rem_euclid(2.0 * span);
        low + if gone > span { 2.0 * span - gone } else { gone }
    };
    // **Each heart's way played on once, and not again for every place weighed**: played on per place, the rules took
    // 12.5 ms a call through the card, 18 a frame, where every other card of the Extra took 2 at most — the frame the
    // run waited on its own steering, at some 33 a second under `--pace=16` (2026-10-09).
    let ways: Vec<Vec<(f32, f32)>> = hearts
        .iter()
        .map(|(at, step)| {
            (0..A_HEART_IS_WATCHED_FOR as usize)
                .map(|frame_ahead| {
                    let frame_ahead = frame_ahead as f32;
                    (
                        carried(at.0, step.0, left, right, frame_ahead),
                        carried(at.1, step.1, top, floor, frame_ahead),
                    )
                })
                .collect()
        })
        .collect();
    Some(Box::new(move |(x, y): (f32, f32)| -> f32 {
        // **A trail's bullet going on onto the place shuts it from the frame it comes**, and not only one on it now:
        // the trails go 0.35 a frame, and the place the hearts came onto last was shut in by those going on into it,
        // the run hit there late in the card with nowhere open — at `-6.0,370.6`, `111.0,371.3` and `22.1,379.9` on
        // whole Extra 30 and the real case the_extra_played_to_its_clear_with_no_hit_and_no_bomb (2026-10-09).
        let mut soonest = trails
            .iter()
            .filter_map(|trail| a_trail_reaches(*trail, (x, y)))
            .fold(A_HEART_IS_WATCHED_FOR, f32::min);
        let kept = A_HEART_IS_KEPT_FROM_BY * A_HEART_IS_KEPT_FROM_BY;
        for way in &ways {
            let first = way
                .iter()
                .take(soonest.ceil() as usize)
                .position(|heart| (heart.0 - x).powi(2) + (heart.1 - y).powi(2) < kept);
            if let Some(first) = first {
                soonest = soonest.min(first as f32);
            }
        }
        soonest
    }))
}

/// **The ground th11's `復燃「恋の埋火」`'s hearts will lay their trails on in the next frames**, handed to the rules as
/// bullets standing where each heart will be [`A_HEART_IS_FOLLOWED_FOR`] frames on, [`A_TRAIL_IS_LAID_WITHIN`] across:
/// the rules see only what is laid, and a trail's bullet is laid within 13.4 of its heart's middle — so the run that
/// steered past a heart 13 to 15 off was hit by the bullet laid beside it the frame after, with room read, three
/// times on the seventeen chapters kept for the card on the midstage rows of 2026-10-10.
pub(crate) fn the_embers_trails_to_come(frame: &Frame) -> Vec<Hazard> {
    let first_slot = frame
        .hazards
        .iter()
        .map(|hazard| hazard.slot + 1)
        .max()
        .unwrap_or(0);
    frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { hitting: true, .. }))
        .filter(|hazard| hazard.half.0.max(hazard.half.1) >= A_HEART_IS_HALF_AT_LEAST)
        .filter_map(|hazard| hazard.step.map(|step| (hazard.at, step)))
        .flat_map(|(at, step)| {
            (1..=A_HEART_IS_FOLLOWED_FOR).map(move |ahead| {
                let ahead = ahead as f32;
                (at.0 + step.0 * ahead, at.1 + step.1 * ahead)
            })
        })
        .enumerate()
        .map(|(laid, at)| Hazard {
            kind: Kind::Bullet {
                hitting: true,
                state: 1,
                next_change: None,
            },
            slot: first_slot + laid,
            frames: None,
            at,
            step: Some((0.0, 0.0)),
            half: (A_TRAIL_IS_LAID_WITHIN, A_TRAIL_IS_LAID_WITHIN),
            grows: 0.0,
        })
        .collect()
}
/// How many frames on each heart's trail is reckoned, and the half of the ground a trail's bullet may be laid on.
const A_HEART_IS_FOLLOWED_FOR: usize = 8;
const A_TRAIL_IS_LAID_WITHIN: f32 = 8.0;

/// **Where th11's `復燃「恋の埋火」` is waited out in front of the boss**, the strategy wiki's own way (wikiwiki.jp/thk/
/// 地/EX, its section read whole on 2026-10-09): 「基本正面維持で、横向きのハートの縦軸だけ気にしながらやれば結構簡単」 —
/// across, under her, kept [`THE_EMBERS_MIDDLE_IS_KEPT_WITHIN`] of the middle, 「隅っこ（特に右下左下）に追い詰められると
/// 跳ね返りで超危険なので、できるだけ中の方を維持」; and down, at the height between `high` and `low` the hearts and trails
/// come onto last, of those the run can reach before one comes onto its way — the hearts aimed at it led down
/// (「極力真下最下段に向けて撃たせる」), and the ones going across stepped over or under. With no heart out, nothing; and
/// with every place shut too soon, no place — see [`THE_EMBERS_PLACE_IS_TOLD_WITH_AT_LEAST`].
pub(crate) fn in_front_of_the_embers(
    frame: &Frame,
    high: f32,
    low: f32,
) -> Option<Option<(f32, f32)>> {
    let mut kept = THE_EMBERS_HEIGHT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(shut_at) = the_embers_shut_at(frame) else {
        *kept = None;
        return None;
    };
    let (left, _, width, _) = frame.field;
    let middle = left + width / 2.0;
    let under_her = frame.boss.map_or(middle, |(x, _)| x).clamp(
        middle - THE_EMBERS_MIDDLE_IS_KEPT_WITHIN,
        middle + THE_EMBERS_MIDDLE_IS_KEPT_WITHIN,
    );
    let across = (-IN_FRONT_OF_THE_EMBERS_ASIDE_BY..=IN_FRONT_OF_THE_EMBERS_ASIDE_BY)
        .step_by(IN_FRONT_OF_THE_EMBERS_LOOKED_AT_EVERY as usize)
        .map(|aside| under_her + aside as f32);
    let places: Vec<((f32, f32), f32)> = across
        .flat_map(|x| {
            (0..=((low - high) / IN_FRONT_OF_THE_EMBERS_LOOKED_AT_EVERY as f32) as i32).map(
                move |step| {
                    (
                        x,
                        low - step as f32 * IN_FRONT_OF_THE_EMBERS_LOOKED_AT_EVERY as f32,
                    )
                },
            )
        })
        .map(|place| (place, shut_at(place)))
        .collect();
    let from = frame.at;
    let alongs: Vec<f32> = places.iter().map(|(on, _)| distance(from, *on)).collect();
    // A place is reached only where nothing shuts a place on the straight way to it before the run is past it.
    //
    // **The way off a place last**, it being the one distance of each pair not already had: weighed first, the check
    // was 2.3 of the 2.65 ms a call took with sixteen hearts and 800 trails' bullets out (2026-10-09).
    let reached = |to: (f32, f32)| -> bool {
        let way = distance(from, to);
        places.iter().zip(&alongs).all(|((on, shut), along)| {
            *along > way
                || along / A_RUN_STEPS_BY < *shut
                || distance(*on, to) + along - way > IN_FRONT_OF_THE_EMBERS_LOOKED_AT_EVERY as f32
        })
    };
    // Of two places shut alike, the nearer the run, under her and the lower, the aimed hearts then led down.
    let worth = |(x, y): (f32, f32), shut: f32| {
        shut - distance(from, (x, y)) * A_FLARE_STEP_COSTS
            - (x - under_her).abs() * A_FLARE_STEP_COSTS
            - (low - y) * A_FLARE_STEP_COSTS
    };
    let best = |reachable_only: bool| {
        places
            .iter()
            .filter(|(place, _)| !reachable_only || reached(*place))
            .map(|(place, shut)| (*place, worth(*place, *shut)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    };
    let (place, better) = best(true).or_else(|| best(false))?;
    let place = match *kept {
        Some(last) => {
            let last = (under_her + last.0, last.1);
            match places.iter().find(|(on, _)| distance(*on, last) < 0.5) {
                Some((_, shut)) if better - worth(last, *shut) <= THE_EMBERS_PLACE_IS_KEPT_BY => {
                    last
                }
                _ => place,
            }
        }
        None => place,
    };
    *kept = Some((place.0 - under_her, place.1));
    let shut = places
        .iter()
        .find(|(on, _)| distance(*on, place) < 0.5)
        .map_or(0.0, |(_, shut)| *shut);
    Some((shut >= THE_EMBERS_PLACE_IS_TOLD_WITH_AT_LEAST).then_some(place))
}
/// **How many frames a place has to stay open to be told at all**: with every place shutting sooner, the run is left to
/// the rules' own dodge and not held near one.
///
/// **Because held, it waited under a heart coming down onto it**: on the real case
/// the_extra_played_to_its_clear_with_no_hit_and_no_bomb's chapter, every place shut within some 30 frames from the
/// card's frame 1,905, the one told was 6 aside of a heart going straight down at 2 a frame, and the run, held within 40
/// of it, was hit by it at `33.7,387.8` 14 from its middle (2026-10-09). Left to the rules then, Extra 41's two chapters
/// and the seventeen kept before went through 16 of nineteen with 3 hits, where always told a place they went through 14
/// with 7.
const THE_EMBERS_PLACE_IS_TOLD_WITH_AT_LEAST: f32 = 30.0;
/// **How far aside of under the boss a place is**, and how far apart the places weighed are: some steps of the run's,
/// so that the aimed hearts' trails laid down through where it stood are stepped out of and not come back into.
///
/// **And not right under her**, which was tried: held there, the run stepped aside of each aimed heart and was drawn
/// back into the column of trails the ones before had laid, and was hit in it at `-48.9,272.6` and `-26.2,317.9` some
/// 15 seconds into the card on whole Extra 41 (2026-10-09).
const IN_FRONT_OF_THE_EMBERS_ASIDE_BY: i32 = 32;
const IN_FRONT_OF_THE_EMBERS_LOOKED_AT_EVERY: i32 = 8;
/// Where [`in_front_of_the_embers`] last told the run to be, across from under the boss and down, while hearts are out.
static THE_EMBERS_HEIGHT: Mutex<Option<(f32, f32)>> = Mutex::new(None);

// **In front of the boss up and down, and not across the low line at the place the hearts come onto last**, the way it
// was (2026-10-08) off the wiki's lines read through a summary: through 11 of the seventeen chapters kept for the card by
// 2026-10-09, where in front of her through 12 — it went across after the hearts' ways and away from her, where the
// wiki keeps the run in front of her and steps it over and under the hearts going across. Waited so, it was kept by
// [`THE_EMBERS_PLACE_IS_KEPT_BY`] rather than the best of each frame for its place going from `56` to `-88`, back to
// `56`, to `-128` and to `-12` over 150 frames on the real case the_extra_played_to_its_clear_with_no_hit_and_no_bomb,
// the run following none of them and carried up into the trails, hit at `21.1,295.5` 2,255 frames in (2026-10-08).
//
// **Nor going across the low line and back to leave the aimed hearts' trails behind**, which was tried — at 0.5 a frame
// 96 either side of the middle: going, it went into the hearts' way rather than letting them by, and took 3, 14, 14 and
// 4 on the first four chapters it was tried on, the real case's and Extra 30's two and Extra 26's (2026-10-09). **Nor its
// fire played on as card 127's is**, each heart back off every wall and each trail bullet on straight, the pace unhit
// longest gone on at, which was tried: 0, 3, 3, 4, 5, 3, 1, 0 and 1 over nine kept chapters where the low line's place
// took 1, 2, 2, 1, 1, 0, 0, 0 and 0, the pace chosen walking the run into the trails laid ahead of where it went
// (2026-10-09).

/// **By how much another height has to be better than the one told last to be gone to**, in the frames before a heart
/// comes onto it: some ten steps of the run's.
const THE_EMBERS_PLACE_IS_KEPT_BY: f32 = 20.0;
/// **How big a bullet's half has to be to be a heart**: a heart's is 5 and its trail's 3.
const A_HEART_IS_HALF_AT_LEAST: f32 = 4.0;
/// **How many frames ahead a heart is weighed**: long enough to cross the middle part of the field twice at 2 a frame,
/// the run crossing it in under sixty.
///
/// **And not `180`**, which was tried for the run shut in with nowhere open late in the card: 6 of the seventeen chapters
/// kept for it by 2026-10-09 hit, 11 hits in all, where this hit about as many with fewer hits (2026-10-09).
const A_HEART_IS_WATCHED_FOR: f32 = 120.0;
/// **How near a heart's middle the run keeps**: its half and the run's, the rules' slack and its trail's first bullets
/// beside it, laid as far as 13.4 off its middle.
///
/// **And not `16`**, which it was: a heart going past 15.1 from the run on `hits-extra15/s7-c13-f41081` laid a trail's
/// bullet 3.1 from it the frame after, and the run was hit there at `75.0,367.4` with `6.6 of room` read, 1,963 frames
/// into the card (2026-10-08). **Nor `32`**, which was tried for the run shut in on the low line late in the card:
/// 0, 2, 2 and 3 on the first four of the fourteen chapters kept for the card, where this took 1, 2, 2 and 1
/// (2026-10-09).
const A_HEART_IS_KEPT_FROM_BY: f32 = 24.0;
/// **How near a trail's bullet the place is not**: its half and the run's and the rules' slack.
const A_TRAIL_IS_KEPT_FROM_BY: f32 = 8.0;
/// **How far either side of the middle a place is**, the strategy wiki's `できるだけ中の方を維持`: two thirds of the way to
/// each wall, off the corners it warns of, 「隅っこ（特に右下左下）に追い詰められると跳ね返りで超危険」.
///
/// **And not `96`, half the way**, which was tried for the run on the real case
/// the_extra_played_to_its_clear_with_no_hit_and_no_bomb, stood at `128,368` at the end of the places till a heart
/// came back off the right wall onto it and hit at `129.7,368.3` 811 frames into the card
/// (`hits-real6/s7-c13-f35802`): with the places closer in, the first four chapters tried took two, none, three and
/// none, the run hit about the middle with the hearts and trails crowded in on it, where at this the eight before
/// took none (2026-10-08). **Nor `160`**, which was tried and went through one more of the sixteen chapters kept for the
/// card by 2026-10-09 than this, ten to nine — inside what one attempt each tells apart, and up against the corners the
/// wiki has the run keep off, 24 from each wall (2026-10-09).
const THE_EMBERS_MIDDLE_IS_KEPT_WITHIN: f32 = 128.0;
/// **Where th11's `深層「無意識の遺伝子」` is waited out**: the strategy wiki's way (wikiwiki.jp/thk/地/EX) — `極力画面下に
/// 行き`, and up out of her way when the boss comes along the bottom — read as across the field from her, on the low line
/// at `low` while she is in the upper part of the field and up at `high` from the frame she comes down into its lower
/// part until she is back up out of it.
///
/// **Because what she lays is thickest where she goes.** Read off `--trace-the-field-in=172` over a pickup of
/// `hits-extra7/s7-c14-f44490` (2026-10-08): she goes from corner to corner in the wiki's order — the bottom left, the
/// top left, the bottom right, the top right and at the run, then the bottom left, the top right, the top left, the
/// bottom right and at the run — laying bullets along her way that set off from it at 2.4 a frame, some 376 out at a
/// time. The rules' own, on the low line, were pinned under her in the bottom left corner as she came along the
/// bottom after coming at the run, and hit there at `-140.7,428.8`.
pub(crate) fn across_from_the_genes(
    frame: &Frame,
    low: f32,
    high: f32,
    card_frames: Option<i32>,
) -> Option<(f32, f32)> {
    let (boss_x, boss_y) = frame.boss?;
    let (left, top, width, height) = frame.field;
    let middle = left + width / 2.0;
    let mut place = THE_GENES_PLACE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // **And up in the corner across from her as she comes at the run**, the wiki's 「自機狙いの時に右上or左上に誘導するといい」:
    // her fifth and tenth ways are at the run, after the four corners before each, and waited for on the low line the
    // run was come at from `174,81` straight down to `78,400` beside it, laying her bullets along that line, and hit
    // there with nowhere open on `s7-c14-f41927` (session `1791464525`) every time (2026-10-10).
    let now = card_frames.unwrap_or(0);
    if now < place.card_frames {
        place.corners = 0;
        place.last_corner = None;
    }
    place.card_frames = now;
    let off = (boss_x - middle).abs() > width * THE_GENES_CORNER_IS_OUT_PAST;
    let corner = (off
        && (boss_y < top + height * THE_GENES_CORNER_IS_OUT_PAST
            || boss_y > top + height * (1.0 - THE_GENES_CORNER_IS_OUT_PAST)))
        .then_some((boss_x > middle, boss_y > top + height / 2.0));
    if corner.is_some() && corner != place.last_corner {
        place.corners += 1;
        place.last_corner = corner;
        // Across from the corner she has come to, and kept so as she crosses the middle coming at the run.
        place.lured_to = if boss_x > middle { -1.0 } else { 1.0 };
    }
    if place.corners % THE_GENES_WAYS_TO_THE_RUN == THE_GENES_WAYS_TO_THE_RUN - 1 {
        return Some((
            middle + place.lured_to * THE_GENES_PLACE_ACROSS,
            top + THE_GENES_LURED_DOWN,
        ));
    }
    let was_up = place.up;
    if boss_y > top + height * THE_GENES_BOSS_IS_LOW_PAST {
        place.up = true;
    } else if boss_y < top + height * THE_GENES_BOSS_IS_HIGH_ABOVE {
        place.up = false;
    }
    let her_side = if boss_x > middle { 1.0 } else { -1.0 };
    // **On the run's own side as the place goes up or down, and across from her only when she crosses the middle**: she
    // goes from corner to corner laying her bullets along her way, and a place gone up and across at once had the run
    // cross the line she had just come down from the top right to the bottom left — the place at `96,232` and the run
    // held at `-43,360` with one of its nine moves clear and seventeen of her bullets within 30, hit there 5,930 and
    // 6,068 frames into the card on the chapters the real cases kept as `s7-c14-f41927` (session `1791464525`) and
    // `s7-c14-f42507` (session `1791589071`); gone down and across at once, the run went from `95,234` to the left under
    // her and was hit at `69.5,234.2` 1,670 in on `s7-c14-f36609` (session `1791582013`) (2026-10-10). Over the twelve
    // chapters kept for the card, from the real cases' six back to `hits-extra7/s7-c14-f44490`, gone across at once they
    // took 1 3 1 0 0 0 3 3 1 0 0 0 at `--pace=16`, and so 2 0 0 0 0 0 0 1 0 0 0 3 (2026-10-10).
    place.side = match (
        place.up != was_up,
        place.side,
        place.her_side == Some(her_side),
    ) {
        (true, ..) | (false, None, _) => Some(if frame.at.0 < middle { -1.0 } else { 1.0 }),
        (false, Some(side), true) => Some(side),
        (false, Some(_), false) => Some(-her_side),
    };
    place.her_side = Some(her_side);
    let across = place.side.unwrap_or(1.0) * THE_GENES_PLACE_ACROSS;
    Some((middle + across, if place.up { high } else { low }))
}
/// Where [`across_from_the_genes`] has its place: up or down, which side of the middle, and which side the boss was on;
/// and how many corners she has come to on this attempt at the card, the last of them, the side across from it and the
/// card's frame asked on.
struct GenesPlace {
    up: bool,
    side: Option<f32>,
    her_side: Option<f32>,
    corners: u32,
    last_corner: Option<(bool, bool)>,
    lured_to: f32,
    card_frames: i32,
}
/// **How far out a corner she comes to is**, of the field's width off its middle and of its height off its top or bottom:
/// the corners are some 150 across and 35 to 60 or 370 to 424 down, and the run's own place up there inside it.
const THE_GENES_CORNER_IS_OUT_PAST: f32 = 0.3;
/// **How many of her ways go round to the one at the run**: four corners and then the run, twice over.
const THE_GENES_WAYS_TO_THE_RUN: u32 = 5;
/// **How far down the run waits for her up in the corner**.
///
/// **Across from the middle by [`THE_GENES_PLACE_ACROSS`] and not by 48**, which was tried for the hit the real case
/// `the_extra_played_to_its_clear_with_no_hit_and_no_bomb` took at `-49.8,112.6` 6,053 frames into the card (session
/// `1791658192`, kept as `s7-c25-f42999`): that chapter went through, but of the eleven kept as `hits-extra*/s7-c14-*`
/// five were hit in the card, 2 1 2 1 1, where this way takes one of twelve (2026-10-11).
const THE_GENES_LURED_DOWN: f32 = 64.0;
/// **How far down the field the boss has come to be along its bottom**, of its height: the bottom corners she comes to
/// are some 370 to 424 down of 448, and the top ones 35 to 60.
const THE_GENES_BOSS_IS_LOW_PAST: f32 = 0.6;
/// **And how far up she has to be back for the run to go down again**, so that one swing of hers past the line does
/// not send the run up and down on every frame.
const THE_GENES_BOSS_IS_HIGH_ABOVE: f32 = 0.5;
/// **How far across from the middle the place is**, on the side away from her: half way to the wall.
const THE_GENES_PLACE_ACROSS: f32 = 96.0;
/// The place, its height held between the two heights above.
static THE_GENES_PLACE: Mutex<GenesPlace> = Mutex::new(GenesPlace {
    up: false,
    side: None,
    her_side: None,
    corners: 0,
    last_corner: None,
    lured_to: 1.0,
    card_frames: 0,
});

// **Card 171's place across weighed off under the boss**, which the way across the low line did at `0.05` a unit, a frame
// of being shut later for every twenty across: at `0.1` the run came back under her after every pair aimed at it, between
// the trails of the pairs before going straight down either side of it, and was hit there boxed in at `-8.5,367.8` and
// `-19.4,368.2` some 1,570 frames in on `hits-extra10/s7-c13-f34656`; at `0.02` it kept off her side of the field, her
// life went from 3,600 to 2,298 in 1,600 frames on `hits-extra16/s7-c13-f37057` with none taken for 200 of them, and it
// was hit at `86.0,350.0` with nowhere open 1,759 frames in (2026-10-08) — her shot's reach being what the wiki's
// 「長くなると辛いので、極力正面を維持して速攻撃破推奨」 asks of the place, and what in front of her keeps.
/// **How many frames ahead a flare is weighed**: long enough for one swelled to its whole reach under the boss to come
/// down past the floor, 300 at 2.5 a frame — the run's whole step crossing the field in fewer than ninety.
const A_FLARE_IS_WATCHED_FOR: f32 = 120.0;
/// **How far past a flare's reach the run keeps**, its own half and the rules' slack.
const A_FLARE_IS_KEPT_FROM_BY: f32 = 6.0;
/// How far apart the places across weighed are.
const A_FLARE_IS_LOOKED_AT_EVERY: f32 = 4.0;
/// **How far the run moves across a frame**, unfocused, which is how long the way to a place takes.
///
/// **With no frames to spare past each place on the way before it is shut**, though the run goes across slower than
/// its whole step while the rules dodge on the way: from `91` to `-12` in 25 frames, 4.1 a frame, on the chapter the
/// real case `a_normal_run_played_to_its_clear_with_no_hit_and_no_bomb` kept as `s6-c6-f15797` (session `1791577212`,
/// 2026-10-10), its way judged reached with under a frame to spare at `-12` and hit there on the edge of the flare
/// coming down at `4.4`, 42.6 from its middle at a reach of 42.65. Which was tried: with 8 frames to spare the five
/// chapters kept for the card took 2 0 2 2 2 at `--pace=16` where none to spare took 2 0 0 0 on the four it was run
/// on; with 4, and with a place reached with none to spare gone to where none was reached with 4, 2 0 2 — the run left
/// standing in a corner at `-184.0,432.0` and `184.0,432.0` as the flares came down on it, on chapters that had gone
/// through with none (2026-10-10).
const A_RUN_STEPS_BY: f32 = 4.5;
/// **What each unit across from the run costs a place**, a frame of being shut later for every twenty across, so that of
/// two places shut equally late the nearer is taken.
const A_FLARE_STEP_COSTS: f32 = 0.05;
/// **How far either side of a place the room about it is weighed**, a flare's width at the floor and the step out of it:
/// a flare crosses the floor some 30 to 50 across.
const A_FLARES_ROOM_IS_WEIGHED_ACROSS: f32 = 48.0;
/// **How far above the run the ring is gone under**: some fifty frames of the ring coming down and the run climbing.
///
/// **And not `32`**, which was tried: from keep\hits-u32\s7-c13-f17465 the ring came within it of the run at the
/// middle's foot some 1,262 frames into the card, and by then every move along under it to the gap `5` across was
/// judged hit — the run stood at `-0.3,361.3` to 1,316 frames in, went up through the ring at 1,320, and was hit back
/// under it at `4.2,356.8` 1,322 in by the lasers sweeping across the foot (c101-gap3.sh, 2026-10-04), where the
/// user's own was up through it by 1,290 and at `275` by 1,320.
const TENAGA_ASHINAGAS_RING_ABOVE_WITHIN: f32 = 64.0;
/// **How far either side of the run its bullets are looked at**: some five of its gaps.
const TENAGA_ASHINAGAS_RING_LOOKED_AT_WITHIN: f32 = 64.0;

/// **Where th11's `反応「妖怪ポリグラフ」` is waited out on the stage's frame `stage_frames`**: straight under the boss
/// [`UNDER_THE_POLYGRAPH_FIRST`] off her till [`THE_POLYGRAPHS_RAYS_GO_OUT_AT`] frames into the card, and from then
/// half way between the two rays the run stands between — turned round her with the rays — at the distance off her it
/// was told last while the rice ahead keeps [`A_DISTANCE_IS_CLEAR_BY`] from it, and otherwise at the other of
/// [`UNDER_THE_POLYGRAPH_IN`] and [`UNDER_THE_POLYGRAPH_OUT`] — the latter no further than the walls — where that rice
/// keeps further off; nowhere with no boss.
///
/// **Two distances and not whichever from 130 to 190 is furthest from that rice**, which was tried: at
/// `hits-extra/s7-c11-f26899` (e12v, 2026-10-07) the run stood 60 frames at 169 off her with one of its nine moves
/// clear, told in to 129, an arc of the ray ahead's rice laid at 158 just inside it from when the run had last been
/// there, till the ray behind came on it at `-144.8,153.2` — a distance between two leaves the ray ahead a dense arc
/// there, where a run gone at its whole step from one of two to the other leaves only a bullet each 8 frames, some 36
/// apart, the wiki's `大きく速く動くことで粒弾に隙間ができ`.
///
/// **In first, then out and between the rays, round her with them, zigzagging in and out**, the strategy wiki's way
/// (`開始直後はまず内側に入ってから外側に大きく振る`, `ジグザグに移動`, `振った後にある程度待つ`, two turns round her over
/// the card, wikiwiki.jp/thk/地/EX): the rays are lasers and are not to be crossed, and lay their rice as far off her as
/// the run is, so the rice the ray ahead leaves between itself and the run is the run's own distance as it was, and a
/// run going to a distance that rice keeps off is never where it is laid. **Turned round her with the rays and not
/// waited for in one place under her**, which was tried five ways — swung in and out as a ray came, across it, with
/// its rice spaced first, and timed on its bullets being laid — every one hit at much the same place, 26 to 32 across
/// and 373 to 376 down, the run's own place being on the ray pointing straight down from her each time one came: read
/// off the trace's own pictures at `--pace=1` (`hits-extra/s7-c11-f26723`, frames 760 to 888), the ray lays its rice in
/// a column up the run's own line as it passes over it, and a swing in or out under her is a swing along that column
/// (2026-10-07). A dozen ways of going round her between the
/// rays were tried before, from the chapters kept as `hits-extra/s7-c11-*` (2026-10-07), each at one distance kept or
/// one closing in or going out, and took three to five hits each, pinned at a wall or caught on the rice laid at its
/// own distance. **And not up, down, left and right only**, the wiki's `上下左右の移動に限定`, which every way waited
/// out under her was held to and still took three hits on each of the three chapters: a run turned round her with the
/// rays moves across and down at once. Held to them on this way as well, the run was hit twice on each of the three
/// `hits-extra` chapters it went through with none without it, at `136.7,72.2` near the top wall (2026-10-08).
///
/// **And not sent straight in or out to the other distance while a ray lays its rice at the run's own distance coming
/// round on it**, which was tried for the hit four of the twelve chapters kept for the card took — at `55.8,346.3` on the
/// chapter the real case `the_extra_played_to_its_clear` kept as `s7-c11-f24993` (session `1791464525`), the run held
/// at `64.8` degrees round her 139 off her from 440 frames in, 20 behind its place, the grains laid at 139 coming round
/// on it at `73.7` … `67.0` from 448 to 496 at 0.137 a frame, and the one laid on it 506 in. With a grain within 12
/// degrees ahead at its distance sending it in or out from where it stood, the twelve took 0 1 0 2 0 0 0 1 1 0 0 2 where
/// this way takes 1 1 1 3 0 0 0 0 0 0 0 0; kept at that distance 20 frames once sent, so as not to be sent out on one
/// frame and back in on the next, and only while held 15 behind its place, 0 1 3 0 0 0 1 1 0 0 0 2, some hit with 7 of
/// room going in or out (2026-10-10).
pub(crate) fn between_the_polygraphs_rays(frame: &Frame, stage_frames: i64) -> Option<(f32, f32)> {
    let (boss_x, boss_y) = frame.boss?;
    let mut swing = UNDER_THE_POLYGRAPH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // **On the stage's frame and not the card's**: the card's timer th11's sight reads, `+0x88c`, stood at 51 to 59
    // over and over through card 169 at `hits-extra/s7-c11-f26723`, its script setting it back, so a swing timed on it
    // was taken back every few frames (2026-10-07).
    if stage_frames < swing.asked || stage_frames - swing.asked > A_SWING_IS_FORGOTTEN_AFTER {
        swing.began = stage_frames;
    }
    swing.asked = stage_frames;
    let into_the_card = stage_frames - swing.began;
    if into_the_card < THE_POLYGRAPHS_RAYS_GO_OUT_AT {
        swing.swung = stage_frames;
        swing.out = None;
        return Some((boss_x, boss_y + UNDER_THE_POLYGRAPH_FIRST));
    }
    // The game's own and not [`the_polygraphs_grains_to_come`], which carry no age.
    let rice: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }) && hazard.frames.is_some())
        .map(|hazard| (hazard, (hazard.at.0 - boss_x).hypot(hazard.at.1 - boss_y)))
        .filter(|(_, off)| *off > A_RAY_STARTS_PAST)
        .map(|(hazard, off)| ((hazard.at.1 - boss_y).atan2(hazard.at.0 - boss_x), off))
        .collect();
    let mut rays: Vec<f32> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter(|hazard| hazard.frames.is_some_and(|age| age <= A_RAY_IS_NEW_FOR))
        .filter(|hazard| (hazard.at.0 - boss_x).hypot(hazard.at.1 - boss_y) > A_RAY_STARTS_PAST)
        .map(|hazard| (hazard.at.1 - boss_y).atan2(hazard.at.0 - boss_x))
        .collect();
    let run = (frame.at.1 - boss_y).atan2(frame.at.0 - boss_x);
    rays.sort_by(f32::total_cmp);
    // **One ray's bullets as the one ray**: laid as the run's distance changed, one ray's newest are at more than one
    // distance and some 3 degrees round her apart, and the gap between two of them is not one between rays.
    let mut rays = rays
        .into_iter()
        .fold(Vec::<Vec<f32>>::new(), |mut rays, angle| {
            match rays.last_mut() {
                Some(ray) if angle - ray[ray.len() - 1] < A_RAY_IS_ONE_WITHIN.to_radians() => {
                    ray.push(angle)
                }
                _ => rays.push(vec![angle]),
            }
            rays
        });
    if rays.len() > 1
        && rays[0][0] + std::f32::consts::TAU - rays[rays.len() - 1][rays[rays.len() - 1].len() - 1]
            < A_RAY_IS_ONE_WITHIN.to_radians()
    {
        let first = rays.remove(0);
        let last = rays.len() - 1;
        rays[last].extend(first.into_iter().map(|angle| angle + std::f32::consts::TAU));
    }
    let mut rays: Vec<f32> = rays
        .iter()
        .map(|ray| ray.iter().sum::<f32>() / ray.len() as f32)
        .collect();
    rays.sort_by(f32::total_cmp);
    let between = match rays.len() < 2 {
        true => run,
        false => {
            let first = rays[0];
            rays.push(first + std::f32::consts::TAU);
            let run = first + (run - first).rem_euclid(std::f32::consts::TAU);
            rays.windows(2)
                .find(|pair| pair[0] <= run && run < pair[1])
                .map_or(run, |pair| (pair[0] + pair[1]) / 2.0)
        }
    };
    let (along_x, along_y) = (between.cos(), between.sin());
    let (left, top, width, height) = frame.field;
    let to = |wall: f32, along: f32| match along.abs() < f32::EPSILON {
        true => f32::INFINITY,
        false => wall / along,
    };
    let furthest = [
        to(left + width - A_WALL_IS_KEPT_OFF_BY - boss_x, along_x),
        to(left + A_WALL_IS_KEPT_OFF_BY - boss_x, along_x),
        to(top + height - A_WALL_IS_KEPT_OFF_BY - boss_y, along_y),
        to(top + A_WALL_IS_KEPT_OFF_BY - boss_y, along_y),
    ]
    .into_iter()
    .filter(|off| *off > 0.0)
    .fold(UNDER_THE_POLYGRAPH_OUT, f32::min)
    .max(UNDER_THE_POLYGRAPH_IN);
    // **The rice ahead of the run round her**: what the ray ahead laid as the run's own distance was, from where the
    // ray is back to the run, the rays turning to the lesser angle.
    let clear_by = |off: f32| {
        rice.iter()
            .filter(|(angle, _)| {
                let ahead = (between - angle + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                ahead > -A_RICE_IS_LOOKED_AT_BEHIND.to_radians()
                    && ahead < A_RICE_IS_LOOKED_AT_AHEAD.to_radians()
            })
            .map(|(_, laid)| (laid - off).abs())
            .fold(f32::INFINITY, f32::min)
    };
    let out = swing.out.unwrap_or(true);
    let at_out = |out: bool| match out {
        true => furthest,
        false => UNDER_THE_POLYGRAPH_IN,
    };
    let held_long = stage_frames - swing.swung >= A_DISTANCE_IS_HELD_FOR_AT_MOST;
    // **And the whole way from the distance held to the other**, the rice the ray ahead laid between them being what
    // the run goes through on its way.
    let clear_all_the_way = |from: f32, to: f32| {
        let steps = ((to - from).abs() / A_RICE_IS_LOOKED_AT_EVERY)
            .ceil()
            .max(1.0) as i32;
        (1..=steps)
            .map(|step| from + (to - from) * step as f32 / steps as f32)
            .map(clear_by)
            .fold(f32::INFINITY, f32::min)
    };
    let stays = (clear_by(at_out(out)) >= A_DISTANCE_IS_CLEAR_BY
        || clear_by(at_out(!out)) <= clear_by(at_out(out)))
        && !(held_long && clear_all_the_way(at_out(out), at_out(!out)) >= A_DISTANCE_IS_CLEAR_BY);
    let out = match stays {
        true => out,
        false => {
            swing.swung = stage_frames;
            !out
        }
    };
    swing.out = Some(out);
    let off = at_out(out);
    Some((boss_x + off * along_x, boss_y + off * along_y))
}
/// **The grains th11's `反応「妖怪ポリグラフ」` has still to lay on the run**, as bullets for the rules to dodge: each
/// ray's newest grain laid at the run's own distance off the boss within [`A_RAY_IS_FOLLOWED_ON_TO_THE_RUN_WITHIN`]
/// ahead of it, carried round her at that distance at the rays' turn, the way the ray goes on laying them. None with no
/// boss. They carry no age, which is what keeps them out of the rays and the rice [`between_the_polygraphs_rays`]
/// reads.
///
/// **Because a ray lays its rice where the run is and nothing of it is there till it is laid**: read off the field
/// traced on the chapter the real case `the_extra_played_to_its_clear` kept as `s7-c11-f24993` (session `1791464525`,
/// 2026-10-10), the run stood at `64.8` degrees round her 139 off her from 440 frames into the card with `2.3 of room`,
/// the grains laid at 139 coming round on it at `73.7`, `72.6` … `67.0` from 448 to 496, and was hit 506 in by the one
/// laid on it at `55.8,346.3` — the hit four of the twelve chapters kept for the card took, and the real cases'
/// Extras hit at `55.7,347.5` and `51.7,91.0` standing with 2.0 of room. Over those twelve, from the real cases'
/// three back to `hits-extra8/s7-c11-f28675`, the rules without them took 1 1 1 3 0 0 0 0 0 0 0 0 at `--pace=16`, and
/// with them 0 0 0 0 0 0 0 0 0 0 0 2 (2026-10-10).
pub(crate) fn the_polygraphs_grains_to_come(frame: &Frame) -> Vec<Hazard> {
    let Some((boss_x, boss_y)) = frame.boss else {
        return Vec::new();
    };
    let (run_x, run_y) = (frame.at.0 - boss_x, frame.at.1 - boss_y);
    let run_off = run_x.hypot(run_y);
    if run_off < A_RAY_STARTS_PAST {
        return Vec::new();
    }
    let run = run_y.atan2(run_x);
    let turn = A_RAY_TURNS_BY.to_radians();
    // **Slots past the game's own and not from the top of the range**: the rules keep a row per slot, grown to reach
    // it, and a grain handed on at `usize::MAX` grew one past the end — the launch was taken down 398 frames into the
    // card, `index out of bounds: the len is 0 but the index is 4294967295` (2026-10-10).
    let first_slot = frame
        .hazards
        .iter()
        .map(|hazard| hazard.slot + 1)
        .max()
        .unwrap_or(0);
    frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .filter(|hazard| hazard.frames.is_some_and(|age| age <= A_RAY_IS_NEW_FOR))
        .map(|hazard| (hazard.at.0 - boss_x, hazard.at.1 - boss_y))
        .filter(|(x, y)| {
            (x.hypot(*y) - run_off).abs() < A_GRAIN_IS_LAID_AT_THE_RUNS_DISTANCE_WITHIN
        })
        .map(|(x, y)| y.atan2(x))
        .filter(|angle| {
            (angle - run).rem_euclid(std::f32::consts::TAU)
                < A_RAY_IS_FOLLOWED_ON_TO_THE_RUN_WITHIN.to_radians()
        })
        .enumerate()
        .map(|(grain, angle)| Hazard {
            kind: Kind::Bullet {
                hitting: true,
                state: 1,
                next_change: None,
            },
            slot: first_slot + grain,
            frames: None,
            at: (
                boss_x + run_off * angle.cos(),
                boss_y + run_off * angle.sin(),
            ),
            // Round her to the lesser angle.
            step: Some((run_off * turn * angle.sin(), -run_off * turn * angle.cos())),
            half: (A_GRAIN_IS_HALF, A_GRAIN_IS_HALF),
            grows: 0.0,
        })
        .collect()
}
/// **How far round her a ray turns a frame, in degrees**, to the lesser angle: `73.7` to `67.0` over the 48 frames from
/// 448 to 496 on the chapter [`the_polygraphs_grains_to_come`] was measured on.
const A_RAY_TURNS_BY: f32 = 0.137;
/// **How far round her ahead of the run, in degrees, a ray's grains are followed on to it**: some ninety frames of its
/// turn, the first of the grains coming on the run on that chapter laid 9 ahead of it.
const A_RAY_IS_FOLLOWED_ON_TO_THE_RUN_WITHIN: f32 = 12.0;
/// **How near the run's own distance off her a ray's newest grain is laid at it**: some three of the grains' 4 across.
const A_GRAIN_IS_LAID_AT_THE_RUNS_DISTANCE_WITHIN: f32 = 12.0;
/// Half a grain, its `size` being 4.
const A_GRAIN_IS_HALF: f32 = 2.0;
/// **How far under the boss the run is till the rays go out, and the nearest and furthest it goes from her after**: 60,
/// the wiki's `まず内側に入ってから`, and then 130 to 190 — out where the rays are furthest apart, 150 apart at 190
/// against 78 at 100.
///
/// **Out and not in under her**, which was tried: waited for at 100 off her and swung out to 170 and back, the run sat
/// on the ring of rice the rays laid at its own distance, round her where they and her blue bullets are thickest —
/// read off the trace's own pictures (`--shot-every=1` at `--pace=1` from `hits-extra/s7-c11-f26723`) — and was hit
/// there three times at each of the chapters kept as `f26723`, `f26899` and `f27227` (2026-10-07).
const UNDER_THE_POLYGRAPH_FIRST: f32 = 60.0;
const UNDER_THE_POLYGRAPH_IN: f32 = 130.0;
const UNDER_THE_POLYGRAPH_OUT: f32 = 190.0;
/// **The card's frame its rays first go out on**, which the run waits in for.
const THE_POLYGRAPHS_RAYS_GO_OUT_AT: i64 = 123;
/// **How far the rice ahead keeps from the distance the run is at, for the run to stay there**, the wiki's
/// `振った後にある程度待つ`: some four of the rice's 4 across and the run's own.
///
/// **Not gone from one distance to the other by turns each 120 frames**, which was tried: at the chapters kept as
/// `hits-extra/s7-c11-f26899` and `f27227` the run was taken three times, the same three each (2026-10-07) — once
/// pinned at the right wall at `184,334`, 190 off her being out of the field there, and twice at `118.7,151.2`, where
/// it had stood 60 frames half way between its two distances with every move judged into the rice the ray ahead had
/// laid as it went from one to the other 195 frames before, till the ray behind it came on it.
///
/// **And not gone to the distance between the two the rice ahead keeps furthest from where it lies at both**, which
/// was tried for `hits-extra21/s7-c11-f26408`, where the ray ahead had laid arcs at 128 and 188 off her ahead of the
/// run and it stood 124 frames at 130 with one of its nine moves clear till it was hit at `55.6,338.9`: gone to
/// whichever between was clearest, that chapter took none and `hits-extra19/s7-c11-f26313` one, the run standing at
/// a 145 with rice 3 ahead till the ray behind came; gone only to one clear by 16, none was (175 by 12.7 the
/// clearest) and `f26408` was hit as before; by 10, two of the eleven chapters were hit, crossing the arcs laid at the
/// distances between, where this way took one (2026-10-08).
const A_DISTANCE_IS_CLEAR_BY: f32 = 16.0;
/// **How many frames one distance is held at most before the other is gone to where it is clear**, the strategy
/// wiki's `大きく速く動くと粒弾に隙間ができて抜けれるようになる。レーザーに気をつけてジグザグに動く` (wikiwiki.jp/thk/地/EX):
/// the rice is laid as far off her as the run is, so a run held at one distance lays a ring of it there round itself.
///
/// **And not only when the rice ahead comes to the distance held**, which it was: on `hits-extra14/s7-c11-f26019` the
/// run stood some 150 off her for 100 frames with one of its nine moves clear, its place going round with the rays
/// away from it, and was hit there at `150.8,283.9` 776 frames into the card (2026-10-08). Over the seven chapters kept
/// for card 169 that took some nine hits without this, five at 60 frames and six at 120; at 30 the first three of nine
/// took seven.
const A_DISTANCE_IS_HELD_FOR_AT_MOST: i64 = 60;
/// **How far apart the distances between the two are that the rice is looked at on**, for a swing made because one
/// was held long: the rice is 4 across and the run's own half 1.
///
/// **And the way there looked at and not the other distance alone**, which it was: the run going from one to the
/// other stood some 150 off her half way between them with one of its nine moves clear among the rice laid there, and
/// was hit at `72.0,339.8` 549 frames into the card on each of the three `hits-extra` chapters and a whole Extra
/// (extra16), all of which it went through with none before (2026-10-08). **And round the place's angle, and not the
/// run's own 10 degrees behind to 30 ahead**, which was tried: nine hits over the nine chapters kept for card 169,
/// where this way took seven. **Nor swung for rice lying across the run's own way straight in or out to the distance
/// told**, which was tried for the run held 150 off her at `73,349` by an arc of rice between it and its place 130 off
/// her on `hits-extra19/s7-c11-f26313`: the first three chapters took four, three and three where this way took one,
/// one and none (2026-10-08).
const A_RICE_IS_LOOKED_AT_EVERY: f32 = 8.0;
/// **How far round her ahead of the run its rice is looked at, in degrees**: some ninety frames of the rays' 0.115 a
/// frame, the rice the ray ahead laid 90 to 195 frames before.
const A_RICE_IS_LOOKED_AT_AHEAD: f32 = 10.0;
/// **And how far behind**, the run's own half at 130 off her.
const A_RICE_IS_LOOKED_AT_BEHIND: f32 = 2.0;
/// **How far inside the field's walls the run is told to keep**, its own half and the rice's.
///
/// **And not `48`**, room for a step out of the way of the rice on the side away from the wall, which was tried for the
/// hits near the top wall at `-104.0,82.6` and `81.8,97.7` on `hits-extra9/s7-c11-f26324` and
/// `hits-extra10/s7-c11-f27544`: those two were still hit twice each, and the three `hits-extra` chapters this way went
/// through with none were hit twice each at `110.2,160.7` (2026-10-08).
const A_WALL_IS_KEPT_OFF_BY: f32 = 16.0;
/// **How old a bullet still marks where its ray is**: one bullet of each, the rays laying one every 8 frames.
const A_RAY_IS_NEW_FOR: i32 = 8;
/// **How near round her, in degrees, two of the newest bullets are one ray's**: the 45 between rays against the 3 of
/// one ray's at two distances, read at `hits-extra/s7-c11-f26723` (2026-10-07).
const A_RAY_IS_ONE_WITHIN: f32 = 8.0;
/// **How far out from the boss a bullet is a ray's**, past the ones she fires round herself.
const A_RAY_STARTS_PAST: f32 = 20.0;
/// **How many frames with no call make the card's start again**: a chapter taken again, or another card.
const A_SWING_IS_FORGOTTEN_AFTER: i64 = 60;
/// The stage's frame [`between_the_polygraphs_rays`] was last asked on, the frame the card began on as it saw it, and
/// whether the distance off the boss it last told was the further.
struct PolygraphSwing {
    asked: i64,
    began: i64,
    out: Option<bool>,
    /// The stage's frame the distance was last gone to.
    swung: i64,
}
static UNDER_THE_POLYGRAPH: Mutex<PolygraphSwing> = Mutex::new(PolygraphSwing {
    asked: i64::MIN / 2,
    began: 0,
    out: None,
    swung: 0,
});

/// **Where th11's `奇跡「ミラクルフルーツ」` is waited out**: where the run is across, [`THE_FRUIT_IS_WAITED_FOR_ABOVE`]
/// above the floor while nothing is coming down on it, and on the floor while a bullet is within
/// [`THE_FRUIT_COMES_WITHIN`] above it and [`THE_FRUIT_IS_LOOKED_AT_ACROSS`] either side of it, the rules stepping
/// aside along the floor from there.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/EX): `早苗の真下で撃ち込むことは意識しないで避けに徹する`
/// and `画面最下段よりやや上で待機→弾が来たら最下段まで下がり残りの弾を横に避ける。Lの字を描くように動く`. Read off its
/// pictures (`hits-real6/s7-c3-f7109`, 2026-10-08): big fruits burst into rings of rice that go out in rows, and the
/// rules' own were hit at `-52.1,390.8` 673 frames in with nowhere open, on the real case
/// the_extra_played_to_its_clear_with_no_hit_and_no_bomb.
pub(crate) fn under_the_miracle_fruit(frame: &Frame) -> (f32, f32) {
    let (_, top, _, height) = frame.field;
    let floor = top + height;
    let (x, y) = frame.at;
    let coming = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .any(|hazard| {
            (hazard.at.0 - x).abs() < THE_FRUIT_IS_LOOKED_AT_ACROSS
                && hazard.at.1 < y
                && y - hazard.at.1 < THE_FRUIT_COMES_WITHIN
        });
    match coming {
        true => (x, floor),
        false => (x, floor - THE_FRUIT_IS_WAITED_FOR_ABOVE),
    }
}
/// **How far above the floor the run waits**, the wiki's `最下段よりやや上`: the room of some ten steps down to the floor.
const THE_FRUIT_IS_WAITED_FOR_ABOVE: f32 = 48.0;
/// **How near above the run a bullet sends it down to the floor**: some twenty frames of the rice's going out.
const THE_FRUIT_COMES_WITHIN: f32 = 80.0;
/// **And how far either side of it**, a row's few bullets and the run's own.
const THE_FRUIT_IS_LOOKED_AT_ACROSS: f32 = 40.0;

/// **Where th11's `本能「イドの解放」` is waited out**: in front of the boss, [`THE_ID_IS_WAITED_OUT_DOWN`] down the
/// field, the rules stepping aside from there; where the run is across with no boss read.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/EX): `ピンクと赤のハート弾がこいしから曲線軌道で発射`,
/// `弾の隙間が一定でないため一定のリズムで避けるということはできない`, and waited a little below the middle of the
/// field in front of her, `右→左→右と交互に抜けていく` — through each heart going by the way it goes, mostly across.
/// The rules' own, waiting in the emptiest box under her, were pinned against the left wall and hit at
/// `-184.0,416.1` 874 frames in on a real case's Extra (orb-real 1791460614, 2026-10-08).
pub(crate) fn in_front_of_the_id(frame: &Frame) -> (f32, f32) {
    let (_, top, _, _) = frame.field;
    let x = frame.boss.map_or(frame.at.0, |(x, _)| x);
    (x, top + THE_ID_IS_WAITED_OUT_DOWN)
}
/// **How far down the field the run waits**, the wiki's 「こいしの正面、画面真ん中よりやや下辺り（「東方地霊殿」の地～霊
/// あたり）」 (its section read whole, 2026-10-09): the panel's 地 and 霊 stand on the field's rows 288 to 321 and 337 to
/// 364 (a picture of a whole run's, scratchpad hudrows), and this is between them.
///
/// **Not `268`**, which it was off a summary's `画面中央よりやや下` alone, some forty under the field's middle and above
/// the 地 the wiki names (2026-10-09).
pub(crate) const THE_ID_IS_WAITED_OUT_DOWN: f32 = 326.0;

/// **Where th11's `焔星「フィクストスター」` is waited out**: in front of the boss, [`AMONG_THE_FIXED_STARS_DOWN`] down the
/// field, kept to it across and dodged from there; in the field's middle across with no boss read.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/6, its section read whole, 2026-10-09):
/// 「実は内側の恒星にも外側の恒星にも当たらない高さがあり、そこで小弾を左右避けすれば楽勝。目安としてはボス正面、画面右の
/// 霊の文字の下半分あたりの高さが安地」, and 「ただし調子に乗って中央付近から横に出すぎると当たるので注意」. Read off its
/// pictures (a whole Pointdevice run, 2026-10-09): rings of stars about suns turning round the boss.
///
/// **In front of her and not on the field's right**, which it was, 72 in from the right wall 300 down: a summary of the
/// wiki read the panel's 霊, beside the field's right, as the field's right (2026-10-09).
pub(crate) fn among_the_fixed_stars(frame: &Frame) -> (f32, f32) {
    let (left, top, width, _) = frame.field;
    let x = frame.boss.map_or(left + width / 2.0, |(x, _)| x);
    (x, top + AMONG_THE_FIXED_STARS_DOWN)
}
/// **How far down the run waits**, the lower half of the panel's 霊: the character stands on the field's rows 337 to 364
/// (a picture of a whole run's, scratchpad hudrows, 2026-10-09).
pub(crate) const AMONG_THE_FIXED_STARS_DOWN: f32 = 356.0;

/// **Where th11's `「地獄の人工太陽」` is waited out**: on the floor under the sun at the field's middle, as far from it
/// as the field goes.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/6, Normal): `戻り弾(赤小弾)の吸引速度は自機の吸引速度と全く
/// 同じなので、移動の操作をしなければ戻り弾にはあたらない。点弾が来たら吸引に逆らって下がりつつ…`, the run pulled in to
/// the sun and the way out of its fire down away from it. Read off its pictures (a whole Pointdevice run, 2026-10-09):
/// the sun at the middle, rings of bullets going out from it and back in.
///
/// **Not up and to the left of it**, the wiki's `上方や斜め上は隙間が広い`, which was tried: waited for 160 off its middle
/// up there, the run was pulled in to `9.7,177.5`, 46 above its edge, and hit there on every attempt — 78 of them on
/// one whole run before it was stopped (2026-10-09).
pub(crate) fn under_the_artificial_sun(frame: &Frame) -> (f32, f32) {
    let (left, top, width, height) = frame.field;
    (left + width / 2.0, top + height)
}

/// **Where across stage 5 of th11 after its midboss is waited out on the stage's frame `stage_frames`**: on the floor,
/// from [`ACROSS_THE_YIN_YANGS_OFF_THE_WALL_BY`] off the wall on the side the run is on as it is first asked, on across
/// to the same off the other at [`ACROSS_THE_YIN_YANGS_AT`] a frame, and never back.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/5, 道中2): `自機狙い陰陽玉ラッシュ地帯はグレイズを意識して
/// チョン避けすれば楽` and `切り返しは危険のためボム推奨` — streamed across with nothing turned back, there being no bomb
/// here.
pub(crate) fn across_the_yin_yangs(frame: &Frame, stage_frames: i64) -> f32 {
    let (left, _, width, _) = frame.field;
    let middle = left + width / 2.0;
    let mut across = ACROSS_THE_YIN_YANGS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if stage_frames < across.0 || stage_frames - across.0 > A_SWING_IS_FORGOTTEN_AFTER {
        let from_the_right = frame.at.0 > middle;
        let x = match from_the_right {
            true => left + width - ACROSS_THE_YIN_YANGS_OFF_THE_WALL_BY,
            false => left + ACROSS_THE_YIN_YANGS_OFF_THE_WALL_BY,
        };
        *across = (stage_frames, from_the_right, x);
    }
    let towards = match across.1 {
        true => -1.0,
        false => 1.0,
    };
    across.2 = (across.2 + towards * ACROSS_THE_YIN_YANGS_AT * (stage_frames - across.0) as f32)
        .clamp(
            left + ACROSS_THE_YIN_YANGS_OFF_THE_WALL_BY,
            left + width - ACROSS_THE_YIN_YANGS_OFF_THE_WALL_BY,
        );
    across.0 = stage_frames;
    across.2
}
/// **How far off the walls the way across starts and ends, and how fast**: some two of the run's steps, and the field's
/// width less those in the stretch's 700 frames.
const ACROSS_THE_YIN_YANGS_OFF_THE_WALL_BY: f32 = 24.0;
const ACROSS_THE_YIN_YANGS_AT: f32 = 0.48;
/// **Where th11's stage 5 after its yin-yangs is waited out**: at the field's middle across, [`AFTER_THE_YIN_YANGS_DOWN`]
/// down it.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/5, 道中2, read 2026-10-09): `陰陽玉と自機狙い怨霊の組み合わせ`
/// and then `撃ち返し怨霊が3波`, waited `中央やや下` and stepped aside of what comes, `逆に上からの針弾に当たらない` —
/// the rules' own, left to themselves, were carried to the floor's left and hit there at `-76.8,432.0` at the stage's
/// frame 9077 with nowhere open, a whirl of big white bullets and chains of small blue ones on the picture (whole run
/// 12, `hits-n12/s5-c4-f9077`, taken again with the same hit).
pub(crate) fn after_the_yin_yangs(frame: &Frame) -> (f32, f32) {
    let (left, top, width, _) = frame.field;
    (left + width / 2.0, top + AFTER_THE_YIN_YANGS_DOWN)
}
/// **How far down the field the run waits**, the wiki's `中央やや下`: some 76 under the middle of its 448.
pub(crate) const AFTER_THE_YIN_YANGS_DOWN: f32 = 300.0;

/// The stage's frame [`across_the_yin_yangs`] was last asked on, whether it set off from the right, and where across.
static ACROSS_THE_YIN_YANGS: Mutex<(i64, bool, f32)> = Mutex::new((i64::MIN / 2, false, 0.0));

/// **Where th11's `「死灰復燃」` is waited out on the stage's frame `stage_frames`**: under the boss
/// [`UP_AND_DOWN_FROM_THE_REKINDLED_LOW`] above the floor to start with, and up to
/// [`UP_AND_DOWN_FROM_THE_REKINDLED_HIGH`] under the top a little left of her, and a little right the next time, once
/// a body has come within
/// [`UP_AND_DOWN_FROM_THE_REKINDLED_WHEN_WITHIN`] of the run, and down again the next time, up and down so all the card.
///
/// **The strategy wiki's way through it for Normal** (wikiwiki.jp/thk/地/5): `妖精は周期的に固定炎弾を撃ちながら自機に
/// 向かってくる…その場で青炎弾を撃つ→自機を捕捉→青炎弾を撃ちながらその位置まで直線で移動`, and the run waits under her
/// shooting, goes up to the top as a zombie comes, waits a little left of the middle there, and down at once as the next
/// comes after it — and up again a little right of the middle, `最上部の少し右で待機`, by turns: so, the fourteen chapters
/// kept for the card, `hits-n29/s5-c9-f20021` to `hits7/s5-c9-f21122` and the real case's `s5-c9-f21767`, took none at
/// `--pace=16` (2026-10-10).
pub(crate) fn up_and_down_from_the_rekindled(frame: &Frame, stage_frames: i64) -> (f32, f32) {
    let (_, top, _, height) = frame.field;
    let boss = frame.boss.map_or(frame.at.0, |(x, _)| x);
    let near = frame.hazards.iter().any(|hazard| {
        matches!(hazard.kind, Kind::Body { solid: true })
            && frame
                .boss
                .is_none_or(|boss| distance(hazard.at, boss) > THE_SPLEEN_EATERS_BOSS_IS_WITHIN)
            && distance(hazard.at, frame.at) < UP_AND_DOWN_FROM_THE_REKINDLED_WHEN_WITHIN
    });
    let mut kept = UP_AND_DOWN_FROM_THE_REKINDLED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if stage_frames < kept.0 || stage_frames - kept.0 > A_SWING_IS_FORGOTTEN_AFTER {
        *kept = (stage_frames, false, false, 0);
    }
    // **A little left of her the first time up, a little right the next, and so by turns**, the wiki's pattern for
    // Normal: `最上部の少し左で待機` and then `最上部の少し右で待機`.
    let place = |up: bool, trips: u32| {
        let aside = match trips % 2 {
            1 => -UP_AND_DOWN_FROM_THE_REKINDLED_ASIDE_BY,
            _ => UP_AND_DOWN_FROM_THE_REKINDLED_ASIDE_BY,
        };
        match up {
            true => (boss + aside, top + UP_AND_DOWN_FROM_THE_REKINDLED_HIGH),
            false => (boss, top + height - UP_AND_DOWN_FROM_THE_REKINDLED_LOW),
        }
    };
    // **Turned once a body comes near the run where it waits, and not again until it has gone off and another comes**
    // — and not on the way there, the wiki's `死体をある程度引き付け`: turned again by the body that came after it while
    // the run was going up, the run went down from `36,161`, up and down again between the zombies, was carried to the
    // right wall and hit in the top corner at `184.0,32.0` with nowhere open, six times on the `--tune` Normal `tune-n2`
    // (2026-10-10).
    let there =
        distance(frame.at, place(kept.1, kept.3)) <= UP_AND_DOWN_FROM_THE_REKINDLED_THERE_WITHIN;
    let near = near && there;
    if near && !kept.2 {
        kept.1 = !kept.1;
        if kept.1 {
            kept.3 += 1;
        }
    }
    kept.2 = near;
    kept.0 = stage_frames;
    place(kept.1, kept.3)
}
/// **How near its place the run has to be to be waiting there**: two of its steps.
const UP_AND_DOWN_FROM_THE_REKINDLED_THERE_WITHIN: f32 = 24.0;
/// **How far above the floor the run waits under her, how far under the top up there and how far aside of her**: room
/// to step aside of a body at the floor and at the top, and the wiki's `少し左` and `少し右`.
const UP_AND_DOWN_FROM_THE_REKINDLED_LOW: f32 = 48.0;
const UP_AND_DOWN_FROM_THE_REKINDLED_HIGH: f32 = 48.0;
const UP_AND_DOWN_FROM_THE_REKINDLED_ASIDE_BY: f32 = 32.0;
/// **How near the run a body comes for it to turn**: some twenty frames of a body going straight at it.
const UP_AND_DOWN_FROM_THE_REKINDLED_WHEN_WITHIN: f32 = 100.0;
/// The stage's frame [`up_and_down_from_the_rekindled`] was last asked on, whether it has the run up, whether a body
/// was near then, and how many times it has gone up.
static UP_AND_DOWN_FROM_THE_REKINDLED: Mutex<(i64, bool, bool, u32)> =
    Mutex::new((i64::MIN / 2, false, false, 0));

/// **Where th11's `贖罪「旧地獄の針山」` is waited out on the stage's frame `stage_frames`**: on the floor, in the gap
/// nearest the run between the ways the spirits' circles will cross it on, kept [`BETWEEN_THE_SPINNING_SPIRITS_IN_BY`]
/// in from the gap's edges and [`BETWEEN_THE_SPINNING_SPIRITS_OFF_THE_WALL_BY`] off the walls; where the run is across
/// with no circle coming down; with no boss read, nowhere.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/5, read 2026-10-09): `設置型ワインダーと回転霊`, and for
/// Normal and above `回転霊は若干反時計回り方向にずれた奇数弾なので左に避けるときは回転霊の直径の1/3ほど、右のときは2/3
/// ほど移動すると良い`, `移動はなるべく早めを心がけたほうが良い` — what kills is a circle coming down over the run, and the
/// way out is aside of it early, before the needles shut the way. Read off its field (`20261007-172443`'s trace, card
/// 131, 2026-10-09): each wave is seven circles of five spirits each, [`A_SPIRIT_GOES_ROUND_IN`] frames a round some
/// 47 across from its middle, sent out 30° apart from [`THE_SPINNING_SPIRITS_SET_OFF_FROM`] off the boss and faster
/// as they go, from 1.1 a frame to 2.2 over 128 frames, a wave every 235 frames, the middle one aimed within 12° of the
/// run; and every hit read was a spirit's, `nowhere open to it`.
pub(crate) fn between_the_spinning_spirits(frame: &Frame, stage_frames: i64) -> Option<(f32, f32)> {
    let boss = frame.boss?;
    let (left, _, width, _) = frame.field;
    let floor = frame.field.1 + frame.field.3;
    let source = (
        boss.0 + THE_SPINNING_SPIRITS_SET_OFF_FROM.0,
        boss.1 + THE_SPINNING_SPIRITS_SET_OFF_FROM.1,
    );
    let mut seen = SPINNING_SPIRITS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    seen.retain(|&(_, at, _)| at <= stage_frames && stage_frames - at < A_SPIRIT_GOES_ROUND_IN);
    if !seen.iter().any(|&(_, at, _)| at == stage_frames) {
        seen.extend(
            frame
                .hazards
                .iter()
                .filter(|hazard| {
                    matches!(hazard.kind, Kind::Body { solid: true })
                        && distance(hazard.at, boss) > THE_SPLEEN_EATERS_BOSS_IS_WITHIN
                })
                .map(|hazard| (hazard.slot, stage_frames, hazard.at)),
        );
    }
    let mut slots: Vec<usize> = seen.iter().map(|&(slot, _, _)| slot).collect();
    slots.sort_unstable();
    slots.dedup();
    let mut ways: Vec<(f32, f32)> = slots
        .into_iter()
        .filter_map(|slot| {
            let round: Vec<(f32, f32)> = seen.iter().filter(|s| s.0 == slot).map(|s| s.2).collect();
            if (round.len() as i64) < A_SPIRIT_GOES_ROUND_IN {
                return None;
            }
            let n = round.len() as f32;
            let middle = (
                round.iter().map(|p| p.0).sum::<f32>() / n,
                round.iter().map(|p| p.1).sum::<f32>() / n,
            );
            // A slot given to a spirit at the source as the last one left the field reads a middle half way between.
            if round
                .iter()
                .any(|&p| distance(p, middle) > A_SPIRITS_ROUND_IS_AT_MOST)
            {
                return None;
            }
            let (dx, dy) = (middle.0 - source.0, middle.1 - source.1);
            let off = dx.hypot(dy);
            let down = dy / off;
            if off < THE_SPINNING_SPIRITS_ARE_SENT_OUT_PAST
                || down < A_SPIRITS_WAY_COMES_DOWN_AT_LEAST
                || middle.1 > floor
            {
                return None;
            }
            let across = source.0 + (floor - source.1) * dx / dy;
            let wide = A_SPIRITS_WAY_IS_WIDE / down;
            Some((across - wide, across + wide))
        })
        .collect();
    drop(seen);
    ways.sort_by(|one, other| one.0.total_cmp(&other.0));
    let (low, high) = (
        left + BETWEEN_THE_SPINNING_SPIRITS_OFF_THE_WALL_BY,
        left + width - BETWEEN_THE_SPINNING_SPIRITS_OFF_THE_WALL_BY,
    );
    let mut gaps = Vec::new();
    let mut from = low;
    for (start, end) in ways {
        if start > from {
            gaps.push((from, start.min(high)));
        }
        from = from.max(end);
    }
    if from < high {
        gaps.push((from, high));
    }
    let x = frame.at.0;
    let into = |&(start, end): &(f32, f32)| {
        let by = BETWEEN_THE_SPINNING_SPIRITS_IN_BY.min((end - start) / 2.0);
        x.clamp(start + by, end - by)
    };
    let wide_enough = gaps
        .iter()
        .any(|(start, end)| end - start >= A_GAP_BETWEEN_THE_SPIRITS_IS_WIDE_ENOUGH);
    let x = gaps
        .iter()
        .filter(|(start, end)| end > start)
        .filter(|(start, end)| {
            !wide_enough || end - start >= A_GAP_BETWEEN_THE_SPIRITS_IS_WIDE_ENOUGH
        })
        .map(into)
        .min_by(|one, other| (one - x).abs().total_cmp(&(other - x).abs()))
        .unwrap_or(x);
    Some((x, floor))
}
/// **How many frames a spirit takes to go once round its circle**, read off slot 924's place coming back to `-25.5`
/// across at frames 652, 684 and 716 of `20261007-172443`'s trace; a round's places averaged are its middle.
const A_SPIRIT_GOES_ROUND_IN: i64 = 32;
/// **How far a spirit is ever from its circle's middle**: the circle's 47 and the half of the round its middle moves.
const A_SPIRITS_ROUND_IS_AT_MOST: f32 = 90.0;
/// **Where off the boss the circles are sent out from**: every circle's middle on the same line out of `22,104` with
/// the boss at `0,128` — the one going down at `22.0` across, the one going left at `104.0` down, over four waves.
const THE_SPINNING_SPIRITS_SET_OFF_FROM: (f32, f32) = (22.0, -24.0);
/// **How far out a circle's middle is before its way is read**: the next wave's spirits gather about the source
/// before they are sent.
const THE_SPINNING_SPIRITS_ARE_SENT_OUT_PAST: f32 = 60.0;
/// **How far a way has to point down to come to the floor**: the circles sent out level or up never cross it.
const A_SPIRITS_WAY_COMES_DOWN_AT_LEAST: f32 = 0.2;
/// **Half how wide a circle's way is across**: its 47, a spirit's half of `7.2`, the run's own and some steps' room.
const A_SPIRITS_WAY_IS_WIDE: f32 = 64.0;
/// **How far in from a gap's edges and off the walls the run waits**: as far as the run is let wander from where it
/// is told — see `crate::steer`'s `ROUND_THE_SPLEEN_EATERS_CIRCLE_HELD_WITHIN` — and the middle of a gap narrower.
///
/// **Not some steps of the run's, `16`**, which was tried: the place told following the run across as the needles
/// moved it, at `hits5/s5-c8-f17902` it was carried from `-118` in the middle of a gap 66 wide to `-179` in the way
/// coming down at `-242` across, and hit there by a spirit at `-181.8,424.3` (2026-10-09).
const BETWEEN_THE_SPINNING_SPIRITS_IN_BY: f32 = 40.0;
const BETWEEN_THE_SPINNING_SPIRITS_OFF_THE_WALL_BY: f32 = 24.0;
/// **How wide a gap has to be to be waited in while a wider one is open**: the needles move the run some way across
/// its gap, and a narrow one has a way either side of it. Waited in the nearest whatever its width, at
/// `hits/s5-c8-f17448` the run went to the middle of a gap 44 wide between the ways at `-59` and `118` across, where
/// one 82 wide was open at the left, was moved out of it by the needles and hit at `-26.1,432.0` (2026-10-09).
const A_GAP_BETWEEN_THE_SPIRITS_IS_WIDE_ENOUGH: f32 = 64.0;
/// A spirit's slot, the stage's frame it was seen on and where.
type SpiritSeen = (usize, i64, (f32, f32));
/// Each spirit seen, for the last round's frames.
static SPINNING_SPIRITS: Mutex<Vec<SpiritSeen>> = Mutex::new(Vec::new());

/// **Where th11's `花咲爺「華やかなる仁者への嫉妬」` is waited out on the stage's frame `stage_frames`**: round the boss
/// [`ROUND_THE_JEALOUS_BOSS_AT`] off her, clockwise at [`ROUND_THE_JEALOUS_BOSS_BY`] a frame from where the run is
/// round her as the way is first asked, over her and down again and on past a side of her a wall is near, kept on the
/// field; with no boss read, nowhere.
///
/// **The strategy wiki's way through it for Normal** (wikiwiki.jp/thk/地/2): `ノーマル以下の場合上を通ってパルスィの
/// 周りを回転するように避けると楽`. Read off its field (`hits4/s2-c6-f7119`, 2026-10-08 and 2026-10-09): the boss goes
/// about the top half throwing big bullets at where the run is — at `1.0` a frame for some 13 frames, then faster by
/// `0.1` a frame to `4.0` — and `先頭の大弾に沿って、桜の形の弾は配置される`, lines of petals laid along each.
pub(crate) fn round_the_jealous_boss(frame: &Frame, stage_frames: i64) -> Option<(f32, f32)> {
    let boss = frame.boss?;
    let (left, top, width, height) = frame.field;
    let mut round = ROUND_THE_JEALOUS_BOSS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if stage_frames < round.0 || stage_frames - round.0 > A_SWING_IS_FORGOTTEN_AFTER {
        round.1 = (frame.at.1 - boss.1).atan2(frame.at.0 - boss.0);
    } else {
        round.1 += (stage_frames - round.0) as f32 * ROUND_THE_JEALOUS_BOSS_BY.to_radians();
    }
    round.0 = stage_frames;
    // **On past the side a wall is near her on**, and not round between her and it: held round her, the run was
    // squeezed between her and the right wall to `162.2,261.0`, 70 off her, and hit there by a big bullet with rows of
    // petals either side of it (`hits-n1/s2-c6-f6622`, 2026-10-09). Not past the top, the wiki's way being over her.
    let across_room = |angle: f32| {
        let dx = angle.cos();
        match dx {
            dx if dx > f32::EPSILON => {
                (left + width - ROUND_THE_JEALOUS_BOSS_KEPT_OFF_THE_TOP_BY - boss.0) / dx
            }
            dx if dx < -f32::EPSILON => {
                (left + ROUND_THE_JEALOUS_BOSS_KEPT_OFF_THE_TOP_BY - boss.0) / dx
            }
            _ => f32::INFINITY,
        }
    };
    for _ in 0..180 {
        if across_room(round.1) >= ROUND_THE_JEALOUS_BOSS_BESIDE_A_WALL_AT_LEAST {
            break;
        }
        round.1 += ROUND_THE_JEALOUS_BOSS_BY.to_radians();
    }
    let angle = round.1;
    // **Nearer her with her near the top**, so that the round over her stays on the field: clamped to the top instead,
    // the run was pinned against it at `-69.7,32.0` and `-184.0,32.0` with nowhere open (2026-10-09).
    //
    // **And nearer her on the side a wall is**, so that the round keeps off it rather than running along it: clamped to
    // the left wall, the run was pinned in the top left corner at `-184.0,35.2` with nowhere open (2026-10-09).
    let (dx, dy) = (angle.cos(), angle.sin());
    let kept = ROUND_THE_JEALOUS_BOSS_KEPT_OFF_THE_TOP_BY;
    let room = |towards: f32, from: f32, low: f32, high: f32| match towards {
        towards if towards > f32::EPSILON => (high - kept - from) / towards,
        towards if towards < -f32::EPSILON => (low + kept - from) / towards,
        _ => f32::INFINITY,
    };
    let off = ROUND_THE_JEALOUS_BOSS_AT
        .min(room(dx, boss.0, left, left + width))
        .min(room(dy, boss.1, top, top + height))
        .max(ROUND_THE_JEALOUS_BOSS_AT_LEAST);
    Some((
        (boss.0 + off * dx).clamp(left, left + width),
        (boss.1 + off * dy).clamp(top, top + height),
    ))
}
/// **How far inside the field's walls and top the round about the boss is kept, and how near her it may come for it**:
/// a step's room past the run, and her body's and a big bullet's off her.
const ROUND_THE_JEALOUS_BOSS_KEPT_OFF_THE_TOP_BY: f32 = 24.0;
const ROUND_THE_JEALOUS_BOSS_AT_LEAST: f32 = 48.0;
/// **How much room across beside her the round goes through at least**: three quarters of the round's own, a big
/// bullet's length and some steps.
const ROUND_THE_JEALOUS_BOSS_BESIDE_A_WALL_AT_LEAST: f32 = 72.0;
/// **How far off the boss the run goes round her, and how fast**: a big bullet's length of room to step aside of it
/// in as it comes, and some `3.4` a frame along the round, over the `1.3` a frame she sweeps the field's width at.
///
/// **Not a degree a frame**, `1.7` along the round and under her own sweep, which was tried: she swept away from the
/// run, which unheld stayed at the left wall at `-184.0,168.8` some 330 off her (whole run 15, `hits-n15/s2-c6-f8110`);
/// held within 40 and on past a wall, 0, 0, 1, 0 and 0 over the five chapters kept for the card, the one some 270 off
/// her at `-119.9,275.4` with rows of petals between, where at two degrees a frame none (2026-10-09).
///
/// **Not behind her as she goes across and over her as she turns**, which was tried: the run was hit at `-61.3,72.9`
/// and `-43.4,150.6` some 200 off her on her left as she went right, in the rows of petals she lays along the big
/// bullets thrown back at it (`hits-n1/s2-c6-f6622`), and 0, 0, 2, 0 and 1 over the five chapters kept for the card
/// where clockwise at a degree a frame took 0, 0, 1, 0 and 0 (2026-10-09).
const ROUND_THE_JEALOUS_BOSS_AT: f32 = 96.0;
const ROUND_THE_JEALOUS_BOSS_BY: f32 = 2.0;
/// The stage's frame [`round_the_jealous_boss`] was last asked on and the angle round the boss it had the run at.
static ROUND_THE_JEALOUS_BOSS: Mutex<(i64, f32)> = Mutex::new((i64::MIN / 2, 0.0));

// **Round the boss, and not from a wall in to the middle and across**, the wiki's other way, `画面端からチョン避け→中央
// まで来たら一気に逆サイドへ`: 16 off the walls and in at `0.75` a frame, it took seven hits over two whole Pointdevice
// runs, three in the bottom left corner with nowhere open (hits-n1, 2026-10-09); 48 off them, 0, 3, 2 and 4 over four
// chapters kept for the card where wherever it is emptiest took 3, 0, 0 and 0. The wiki's `霊夢Ａなら切り返さず画面端
// から端へのワープを使えば回避は余裕` asks for the warp Yukari's equipment has — at a wall, the shot and the focus let go
// and the way to it pressed twice — which the rules do not press.

/// **Where across the stage before th11's Extra midboss is waited out on the stage's frame `stage_frames`**: till
/// [`THE_RED_FAIRIES_ARE_UP_TILL`], under the side's middle of the bodies gathered above
/// [`THE_RED_FAIRIES_GATHER_ABOVE`] that has more of them, or with none gathered under the lowest one going up — kept
/// [`UNDER_THE_RED_FAIRIES_OFF_THE_WALL_BY`] off either wall — and after it where it last was; nowhere with no body
/// up yet.
///
/// **The strategy wiki's way through it** (wikiwiki.jp/thk/地/EX): `最初は最下段、赤妖精が出て来る場所のどちらかで
/// 待機。片方を尻撃ちで倒した後、もう片方の赤妖精の弾をチョン避けしつつ倒す`, then of the sunflower fairies `どちらかの
/// サイドで避け続けると安全` and of the blue fairies' rice `大きく動かずにその場でチョコチョコ避ける`. Read off the stage's
/// whole field traced and its pictures (`--trace-the-field-in=all`, scratchpad bodies-up.py, 2026-10-08): from 194
/// frames red fairies come out at the floor at both walls and go up, and gather at the top, at the top right on one
/// launch and at the top left on another, aiming streams of big bullets at the run from 300 frames to some 600; a
/// sunflower fairy at the top follows to 700, and fairies coming down with rings of rice from 700 on. Every hit the
/// ways before took in it, seven on the chapters kept as `hits-extra*/s7-c1-*`, was from 337 to 737 frames in.
///
/// **Checked on the trace's own field** (`hits-extra10/s7-c1-f593`, 2026-10-08): the run went under the ones
/// gathered at the top left at `-101` by 400 frames, they were gone by 440, and it went on under the ones at the top
/// right, of which two were left by 520; none of the eight chapters kept was hit.
pub(crate) fn under_the_red_fairies(frame: &Frame, stage_frames: i64) -> Option<f32> {
    let mut last = UNDER_THE_RED_FAIRIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if stage_frames >= THE_RED_FAIRIES_ARE_UP_TILL {
        return *last;
    }
    let (left, top, width, height) = frame.field;
    let middle = left + width / 2.0;
    let bodies: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        // **Solid or not**: th11's fairies are read as bodies with nothing solid on them, and only being inside the
        // field tells them from the boss's and the player's own laid at `0,0` and `0,-16`.
        .filter(|hazard| matches!(hazard.kind, Kind::Body { .. }))
        .map(|hazard| hazard.at)
        .filter(|(x, y)| *y > top && *y < top + height && *x > left && *x < left + width)
        .collect();
    let up = |on_the_left: bool| -> Vec<f32> {
        bodies
            .iter()
            .filter(|(x, y)| {
                *y < top + THE_RED_FAIRIES_GATHER_ABOVE && (*x < middle) == on_the_left
            })
            .map(|(x, _)| *x)
            .collect()
    };
    let (on_the_left, on_the_right) = (up(true), up(false));
    let gathered = match on_the_left.len() >= on_the_right.len() {
        true => on_the_left,
        false => on_the_right,
    };
    let x = match gathered.is_empty() {
        false => gathered.iter().sum::<f32>() / gathered.len() as f32,
        true => bodies.iter().max_by(|a, b| a.1.total_cmp(&b.1))?.0,
    };
    let x = x.clamp(
        left + UNDER_THE_RED_FAIRIES_OFF_THE_WALL_BY,
        left + width - UNDER_THE_RED_FAIRIES_OFF_THE_WALL_BY,
    );
    *last = Some(x);
    Some(x)
}
/// **How many frames into the Extra the red fairies are up**, the last of their streams laid by some 600.
const THE_RED_FAIRIES_ARE_UP_TILL: i64 = 600;
/// **How far down the red fairies gathered at the top are**: they stand from 30 to 70 down there, and go up past 100
/// on their way.
const THE_RED_FAIRIES_GATHER_ABOVE: f32 = 120.0;
/// **How far off a wall the place under them is kept**: room to step aside either way from the streams they aim at
/// the run, the wiki's `チョン避け`.
///
/// **And not `34`**, which was tried as `-150`, under the column they come out at on the left: the run went into the
/// wall stepping aside from the streams and was hit there at `-184.0,396.8` some 630 frames in on
/// `hits-extra10/s7-c1-f593` and `f595` twice each and at `-184.0,377.2` on `hits-extra11/s7-c1-f633`
/// (2026-10-08, read off the trace's pictures). **Nor a side chosen beforehand**, which was tried both ways: on the
/// left all through, the run was pushed into the wall at `-179.5,429.4` on `hits-extra6/s7-c1-f614` by the streams
/// of the ones gathered at the top right; gone right at 380, it could not get across them on `hits-extra11/s7-c1-f633`
/// and was hit at the wall at `-179.5,374.8`; and on the right first, the ones gathered at the top left on
/// `hits-extra10/s7-c1-f593` were left up till 600 — which side they gather at changes from launch to launch.
const UNDER_THE_RED_FAIRIES_OFF_THE_WALL_BY: f32 = 82.0;
/// Where across [`under_the_red_fairies`] last told the run to be.
static UNDER_THE_RED_FAIRIES: Mutex<Option<f32>> = Mutex::new(None);

/// **Where across th11's Extra boss's fire after card 169 is waited out at the height `y`**: of the places from
/// [`THE_LONG_ROWS_ARE_KEPT_FROM_THE_WALL_BY`] off the left wall to the middle, every
/// [`THE_LONG_ROWS_ARE_LOOKED_AT_EVERY`] across, the one with its next row furthest above it, the nearest of those
/// level with it; nowhere with nothing in the field.
///
/// **The strategy wiki's way through it**, its `ボス通常攻撃その6`: `通常3の弾列が5発から10発になったもの。通常3よりは
/// 縦の隙間が大きいので事故は少ないかも。弾列が横に長いため、低速移動だけでは危険`, and of 通常3 `左へ誘導した方が楽`
/// (wikiwiki.jp/thk/地/EX). Read off its whole field traced (`--trace-the-field-in=all` from
/// `hits-extra20/s7-c11-f29560`, scratchpad map.py, 2026-10-08): arms of bullets wound out from the boss, which goes
/// after the run across, each lying near level by the time it is down at the floor as a row some 50 across, the rows
/// 50 to 80 above one another; left to the rules the run was carried to the bottom left corner and hit there at
/// `-184,432` on a whole Extra and on each of three pick-ups of that chapter.
///
/// **Checked on the trace's own pictures** (`hits-extra20/s7-c11-f29560`, 2026-10-08): the run stood on the floor in
/// the gaps between the rows, the boss went after it left from `0` to `-81` across, and the focus key was down on 44
/// of the fire's 1,340 frames, the fine step's where nothing at the whole one was open; but the rules carried the run
/// away from its place to `115.7` across some 1,040 frames in, right of the middle the wiki keeps it left of. Held
/// within 40 of its place as well (see `steer`), the run kept from `-113.6` to `4.8` across, the boss went after it to
/// `-78`, the focus key was down on 244 of 1,237 frames, and none of the eleven chapters kept that go through the fire
/// was hit, as none had been unheld.
pub(crate) fn between_the_long_rows(frame: &Frame, y: f32) -> Option<f32> {
    let (left, _, width, _) = frame.field;
    let middle = left + width / 2.0;
    let rows: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
        .map(|hazard| hazard.at)
        .collect();
    if rows.is_empty() {
        return None;
    }
    let next_above = |x: f32| {
        rows.iter()
            .filter(|(bx, by)| (bx - x).abs() < THE_LONG_ROWS_ARE_LOOKED_AT_WITHIN && *by < y)
            .map(|(_, by)| y - by)
            .fold(f32::INFINITY, f32::min)
    };
    let first = left + THE_LONG_ROWS_ARE_KEPT_FROM_THE_WALL_BY;
    let steps = ((middle - first) / THE_LONG_ROWS_ARE_LOOKED_AT_EVERY).floor() as i32;
    (0..=steps)
        .map(|step| first + step as f32 * THE_LONG_ROWS_ARE_LOOKED_AT_EVERY)
        .map(|x| (x, next_above(x).min(THE_LONG_ROWS_ARE_CLEAR_ABOVE)))
        .max_by(|a, b| {
            a.1.total_cmp(&b.1).then_with(|| {
                (b.0 - frame.at.0)
                    .abs()
                    .total_cmp(&(a.0 - frame.at.0).abs())
            })
        })
        .map(|(x, _)| x)
}
/// **How far off the left wall the places start**: some two of its rows' 50 across, out of the corner it was pinned
/// in.
const THE_LONG_ROWS_ARE_KEPT_FROM_THE_WALL_BY: f32 = 96.0;
/// **How far apart across the places are**, half a row's 50 across.
const THE_LONG_ROWS_ARE_LOOKED_AT_EVERY: f32 = 24.0;
/// **How far either side of a place its rows are looked for**: half a row's 50 across and the run's own.
const THE_LONG_ROWS_ARE_LOOKED_AT_WITHIN: f32 = 28.0;
/// **How far above a place a row is as good as none**: the most two rows lie apart, so that places clear that far are
/// chosen between by how near they are.
const THE_LONG_ROWS_ARE_CLEAR_ABOVE: f32 = 80.0;

/// **Where the stage after th11's Extra midboss is waited out on the stage's frame `stage_frames`**, counted from the
/// first frame it is asked on, which is the frame the midboss is beaten on: against the left wall
/// [`AFTER_THE_MIDBOSS_LOW`] down till [`AFTER_THE_MIDBOSS_THE_SECOND_SET_COMES`], along the low line to the right wall
/// by [`AFTER_THE_MIDBOSS_AT_THE_RIGHT_WALL_BY`], straight up it to [`AFTER_THE_MIDBOSS_UP`] till
/// [`AFTER_THE_MIDBOSS_BACK_DOWN_AT`], and back to the bottom left after — all of it again from
/// [`AFTER_THE_MIDBOSS_THE_SECOND_WAVE_COMES`]; from [`AFTER_THE_MIDBOSS_THE_THREE_WAYS_COME`], nowhere.
///
/// **The strategy wiki's way through it**: `早苗撃破後、画面左端で密着してショットを撃つと1セット目をほとんど撃たれない。
/// 2セット目がきたらやや間隔を多めにとる感じでチョン避けしていくと、画面右端についたころに真上にスペースができるのでそこ
/// に逃げこみ、最後のセットをそこに撃たせて左下に戻ると楽` (wikiwiki.jp/thk/地/EX). Read off the bodies and bullets of
/// the whole chapter traced (`--trace-the-field-in=all` from `hits-real1/s7-c5-f9780`, scratchpad sets5.py,
/// 2026-10-08): the first set is a row of fairies along 160 down coming in from the left, each killed as it goes over
/// a run shooting up the left wall; from 105 frames on rows come in along it from the right, along 64 from the left
/// and round down the left side, and their streams fill the bottom left — some 50 to 80 bullets in its sixteenth of
/// the field by 250 frames, where the run left against the left wall was hit at `-182.1,432.0` 530 frames in; by 420
/// the sixteenth at the top right and the one under it hold none to four; by 620 the streams lie at the bottom right,
/// some 100 in its sixteenth, and the left half is empty. A second wave of the same sets comes 1,500 frames in, and
/// from 2,150 the fairies come down in columns from the top firing the wiki's `3way大弾地帯`.
///
/// **Up the right wall to where its column is emptiest, and not to one height**, which it was: the room the wiki has
/// come above the run does not come every launch — on `hits-extra24/s7-c5-f9621` the fairies coming round down the
/// middle's right laid their streams down the right wall over the run, it did not get up from `184,413`, and was hit
/// at `184.0,432.0` 438 frames after the midboss; on `hits-real1/s7-c5-f9780` it went up to `184,317` with the room
/// there (2026-10-08).
pub(crate) fn from_the_left_wall_to_the_top_right(
    frame: &Frame,
    stage_frames: i64,
) -> Option<(f32, f32)> {
    let field = frame.field;
    let mut since = AFTER_THE_MIDBOSS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if stage_frames < since.asked || stage_frames - since.asked > A_SWING_IS_FORGOTTEN_AFTER {
        since.began = stage_frames;
    }
    since.asked = stage_frames;
    let into = stage_frames - since.began;
    if into >= AFTER_THE_MIDBOSS_THE_THREE_WAYS_COME {
        return None;
    }
    let into = match into >= AFTER_THE_MIDBOSS_THE_SECOND_WAVE_COMES {
        true => into - AFTER_THE_MIDBOSS_THE_SECOND_WAVE_COMES,
        false => into,
    };
    let (left, top, width, _) = field;
    let low = top + AFTER_THE_MIDBOSS_LOW;
    let right = left + width;
    // How many bullets lie about a height up the right wall, which is how much room there is at it.
    let crowd_at = |y: f32| {
        frame
            .hazards
            .iter()
            .filter(|hazard| matches!(hazard.kind, Kind::Bullet { .. }))
            .filter(|hazard| {
                right - hazard.at.0 < AFTER_THE_MIDBOSS_THE_WALLS_ROOM_IS_LOOKED_AT_WITHIN
                    && (hazard.at.1 - y).abs()
                        < AFTER_THE_MIDBOSS_THE_WALLS_ROOM_IS_LOOKED_AT_WITHIN
            })
            .count()
    };
    // **Up from the low line and never under it**, which was tried: weighed down to the floor, the bottom right corner
    // read emptiest before the streams came into it, and on `hits-extra25/s7-c5-f9830` the run went down into it and
    // was hit there seven times running at `180.8,432.0` (2026-10-08), the wiki's way being up the wall.
    let up_the_wall = || {
        let highest = top + AFTER_THE_MIDBOSS_UP;
        let steps = ((low - highest) / AFTER_THE_MIDBOSS_THE_WALL_IS_LOOKED_UP_EVERY) as i32;
        (0..=steps)
            .map(|step| highest + step as f32 * AFTER_THE_MIDBOSS_THE_WALL_IS_LOOKED_UP_EVERY)
            .min_by(|a, b| {
                crowd_at(*a)
                    .cmp(&crowd_at(*b))
                    .then_with(|| (a - frame.at.1).abs().total_cmp(&(b - frame.at.1).abs()))
            })
            .unwrap_or(highest)
    };
    Some(match into {
        into if into < AFTER_THE_MIDBOSS_THE_SECOND_SET_COMES => (left, low),
        // **Slid along at an even pace, and not in hops**, which was tried for the wiki's `やや間隔を多めにとる感じでチョン
        // 避け`: eight hops of 48 some 37 frames apart, each stream aimed at the run left where it stood, took six hits on
        // five of the six chapters kept for this one it was tried on, five of them at the bottom right corner, where
        // sliding took one on one of eight (2026-10-08).
        into if into < AFTER_THE_MIDBOSS_AT_THE_RIGHT_WALL_BY => {
            let along = (into - AFTER_THE_MIDBOSS_THE_SECOND_SET_COMES) as f32
                / (AFTER_THE_MIDBOSS_AT_THE_RIGHT_WALL_BY - AFTER_THE_MIDBOSS_THE_SECOND_SET_COMES)
                    as f32;
            let risen = ((along - AFTER_THE_MIDBOSS_RISES_FROM)
                / (1.0 - AFTER_THE_MIDBOSS_RISES_FROM))
                .max(0.0);
            (
                left + width * along,
                top + AFTER_THE_MIDBOSS_ALONG
                    - (AFTER_THE_MIDBOSS_ALONG - AFTER_THE_MIDBOSS_UP) * risen,
            )
        }
        into if into < AFTER_THE_MIDBOSS_BACK_DOWN_AT => (right, up_the_wall()),
        _ => (left, low),
    })
}
/// **How far along to the right wall the run starts up towards [`AFTER_THE_MIDBOSS_UP`]**, so that it comes to the
/// wall in the room above the streams and not under them.
///
/// **And not up the wall from where it comes to it**, which it was: on `hits-real8/s7-c5-f9774` the run came to the
/// right wall at `184,347` 391 frames in with the sixteenth of the field it was in holding some 100 bullets and the one
/// above it 3 to 16, could not get up through them, was carried down the wall and was hit at `150.3,432.0` — on the
/// real case it was kept off and on both chapters taken again from it (2026-10-08). Rising from six tenths of the way
/// along, it came to the wall in the room above them and took none on all nine chapters kept for this one, the pictures
/// having the last set fired at it there and the run back at the bottom left after (2026-10-08).
const AFTER_THE_MIDBOSS_RISES_FROM: f32 = 0.6;
/// **How far off the right wall, and either side of a height, its room is counted within**: a stream's width.
const AFTER_THE_MIDBOSS_THE_WALLS_ROOM_IS_LOOKED_AT_WITHIN: f32 = 40.0;
/// **How far apart the heights up the right wall weighed are.**
const AFTER_THE_MIDBOSS_THE_WALL_IS_LOOKED_UP_EVERY: f32 = 16.0;
/// **How far down the run waits against the left wall**: 368, the rules' own height there, below the rows of fairies
/// along 160 and shooting up into them.
const AFTER_THE_MIDBOSS_LOW: f32 = 368.0;
/// **And how far down it goes along to the right wall**: 300, above where the streams aimed at it pile along the
/// floor and below the rows along 160.
///
/// **And not at `368`**, which it was: on `hits-extra25/s7-c5-f9830` the run was pushed down to the floor going along
/// at that, the streams it drew following it along the floor into the bottom right corner, and it came to the right
/// wall at `184,432` with none of its nine moves clear and the right half holding 120 to 170 bullets to the left's 13
/// to 52, and was hit there seven times running (2026-10-08). At `300` the same chapter went along at 335 to 346,
/// came to the right wall at 342 and was not hit, and of the eight chapters kept for it seven were played through
/// with none and one with one: `hits-extra24/s7-c5-f9621`, whose run is kept at the floor near the right wall and
/// was hit 22 frames in crossing to the left wall through the first set, before this way's along begins.
const AFTER_THE_MIDBOSS_ALONG: f32 = 300.0;
/// **How many frames after the midboss the second set's row comes in from the right**, read off its bodies.
const AFTER_THE_MIDBOSS_THE_SECOND_SET_COMES: i64 = 105;
/// **By how many frames after the midboss the run is at the right wall**: where the top right sixteenth of the field
/// stood empty, the wiki's `画面右端についたころに真上にスペースができる`.
const AFTER_THE_MIDBOSS_AT_THE_RIGHT_WALL_BY: i64 = 400;
/// **How far down it goes up the right wall to at the highest**: 208, inside the sixteenth under the top right one and
/// below the row of fairies along 160 coming in from the right there.
const AFTER_THE_MIDBOSS_UP: f32 = 208.0;
/// **How many frames after the midboss it comes back to the bottom left**: where the streams of the last set lie along
/// the bottom right and the left half of the field is empty, the wiki's `最後のセットをそこに撃たせて左下に戻る`.
const AFTER_THE_MIDBOSS_BACK_DOWN_AT: i64 = 600;
/// **How many frames after the midboss the second wave of the same sets comes**, read off its first row's bodies.
const AFTER_THE_MIDBOSS_THE_SECOND_WAVE_COMES: i64 = 1500;
/// **How many frames after the midboss the columns of fairies firing three ways come down**, from which the rules dodge
/// it: the run picked up after its hit and left to them went through to the boss.
///
/// **Left to the rules and not the wiki's way through them**, `3way大弾地帯は、画面上部端から外周に沿ってUの字を描くように
/// チョン避け` (wikiwiki.jp/thk/地/EX, 道中2): one hit there over eleven whole Extras, at `97.2,32.0` on Extra 31, and
/// none on the six chapters kept for the stage after the midboss played through to the boss on the midstage rows
/// (2026-10-10) — where the U's frames, from the top edge round to the other side, are a reading nobody has made.
const AFTER_THE_MIDBOSS_THE_THREE_WAYS_COME: i64 = 2150;
/// **Whether the run is held to the place [`from_the_left_wall_to_the_top_right`] last told**: along the low line from
/// the left wall to the right one, and not up the right wall or back.
///
/// **Held along the low line**, which it was not: left to the rules, the run stayed against the left wall 240 frames
/// after the midboss where its place had set off along the low line at 105, and on the real case it was still there
/// as the streams filled the bottom left and was hit at `-181.4,432.0` 341 frames in (`hits-real2/s7-c5-f9635`).
/// **And not held up the right wall and back**, which was tried, at 600 and at 700: the run was pushed up the wall
/// into the last set's streams coming down it and through them on its way back, and the first four of the six
/// chapters kept were hit five, one, none and four times, at `184.0,258.5` and `-158.6,432.0` among others, where
/// unheld those five chapters took none (2026-10-08). The wiki's room above is one that `ができる`, there when it is.
/// Held along the low line alone, the six chapters kept took none, and on `f9635` the run left the wall at 105 frames,
/// was at the right one by 388, stood there from 393 to 412 down with the room above not come, and was back at the
/// bottom left by 670.
pub(crate) fn the_stage_after_the_midboss_is_held_along_the_low_line() -> bool {
    let since = AFTER_THE_MIDBOSS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let into = since.asked - since.began;
    let into = match into >= AFTER_THE_MIDBOSS_THE_SECOND_WAVE_COMES {
        true => into - AFTER_THE_MIDBOSS_THE_SECOND_WAVE_COMES,
        false => into,
    };
    (AFTER_THE_MIDBOSS_THE_SECOND_SET_COMES..AFTER_THE_MIDBOSS_AT_THE_RIGHT_WALL_BY).contains(&into)
}
/// The stage's frame [`from_the_left_wall_to_the_top_right`] was last asked on, and the frame the stage after the
/// midboss began on as it saw it.
struct AfterTheMidboss {
    asked: i64,
    began: i64,
}
static AFTER_THE_MIDBOSS: Mutex<AfterTheMidboss> = Mutex::new(AfterTheMidboss {
    asked: i64::MIN / 2,
    began: 0,
});

/// **Where th11's `「嫌われ者のフィロソフィ」` is waited out on the stage's frame `stage_frames`**: at the top right
/// inside the boss's round through its first [`THE_BLUE_STAGE_LASTS`] frames, and after them
/// [`THE_HATED_IS_LEFT_OFF_THE_MIDDLE_BY`] off its middle the other way from her roses, the middle of every one of
/// them, or from her where none is out; nowhere with no boss.
///
/// **Away from the middle of her roses and not from her**, through the yellow stage and the red: the roses are where the
/// lights going over the field cross her rows, which is not where she is, and kept away from her the run was left among
/// them — at `-39.2,149.1` and `-84.2,175.0` in the red stage, the roses come up the left beside it and the run carried up
/// into the top left with none of its nine moves clear (the real case's `the_extra_played_to_its_clear_with_no_hit_and_no_bomb`,
/// sessions `1791573961` and `1791572844`, 2026-10-10), and at `-9.9,253.2` in the yellow, the roses gathered under the
/// middle with the run held on it (`hits-extra34/s7-c15-f43953`). Over the twelve chapters kept off the card, from those
/// three back to `hits-extra4/s7-c15-f43323`, kept away from her took 1 1 0 0 2 0 0 0 0 0 0 0 at `--pace=16`, away from
/// her roses in the red stage alone 0 0 0 0 2 0 0 0 0 0 0 0, and in both 0 throughout (2026-10-10).
///
/// **Not the corner furthest from her through the blue stage**, which was tried: through it she goes up and down the left
/// of her round, the corner furthest from her going from the top right to the bottom right and back as she does, and
/// the run was sent 200 down the right and back across the columns each time, held within 40 of its place with no move
/// clear, stood at `19,338` half way, and was hit at `-28.6,308.3` 795 frames in by her roses as she came to the bottom
/// left — twice on each of the three chapters kept as `hits-extra4/s7-c15-f43323`, `f43580` and `f44225` (2026-10-07).
///
/// **A corner kept and not the place across the middle from her**, which was tried: 0.8 as far across it as she is,
/// the run went through the blue stage with no hit on the three chapters kept as `hits-extra4/s7-c15-f43323`, `f43580`
/// and `f44225`, and was hit twice on each at `-184.0,196.3`, 2,303 frames in: the yellow stage's boss crosses her
/// round through its middle from one corner to the other, the place across from her going over to the far side as
/// she does, and the run was left at the left wall as her roses came up there (2026-10-07).
///
/// **Away from her and not the middle of her round**, which was tried: held within 40 of `0,244`, the middle of the
/// round she goes along through the card (`-128` to `128` across and `128` to `360` down, read off the bodies traced at
/// `hits-extra4/s7-c15-f43323`, 2026-10-07), the run was hit five times on each of the three chapters kept as
/// `f43323`, `f43580` and `f44225`, at `-0.3,266.8` and `-40.9,258.3` and the like: the trace's pictures show the blue
/// stage's roses as a band some 180 across that follows her up the side she is on and reaches to within 12 of the
/// middle — the wiki's `青段階：右上が長時間安全` (wikiwiki.jp/thk/地/EX), the corner she comes to last. The rules'
/// own were hit some sixty times running on the floor, on the bottom of her round.
///
/// **And not gone through the blue stage's rows by crossing them**, as card 174's rings are, which was tried for the
/// wiki's `右上でちょこちょこ`: on `hits-extra8/s7-c15-f46365`, where the card began with the run in the bottom right
/// corner, it was still hit twice at `112.4,432.0`, and on the three `hits-extra4` chapters this way went through with
/// none it was hit twice on each at `111.6,300.7` (2026-10-08).
///
/// **And its red stage waited out away from her and not in the middle of a square of its grid**, the wiki's
/// `赤弾の作る四角形の中央は薔薇が届かない`, which was tried: the grid's lines are 100 apart and go across the field up
/// and to the left at 1 a frame, the squares with them, and a finer grid 30 apart comes over it late in the stage;
/// read off where its bullets stand, the square the run was in had it go after it up and to the left with one of its
/// nine moves clear, and the square near it towards the bottom right, the wiki's `とにかくひたすら右下を目指す`, had it
/// go from one to the next as the grid went across — either way it was hit twice over `hits-extra13/s7-c15-f46804`,
/// `hits-extra9/s7-c15-f43581`, `hits-extra8/s7-c15-f46365` and `hits-extra4/s7-c15-f43323`, the place away from her
/// once (2026-10-08).
///
/// **And its row bullets beside a rose not handed to the rules as roses**, which was tried: its roses spread through
/// the rows by taking the bullets they touch — on `hits-extra9/s7-c15-f43581` the bullet `11.7` from the run, 6 across,
/// was a round one of reach 16.8 the frame after, 15.2 from the next rose's middle, with the run inside it and `4.7 of
/// room` read the frame before — but with each bullet within 24 of a rose read as a rose, the run had none of its nine
/// moves clear beside them and was hit two and three times on that chapter, `hits-extra8/s7-c15-f46365` and
/// `hits-extra4/s7-c15-f43323`, the last two having gone through with none without it (2026-10-08).
pub(crate) fn away_from_the_hated(frame: &Frame, stage_frames: i64) -> Option<(f32, f32)> {
    let (boss_x, boss_y) = frame.boss?;
    let (left, top, width, _) = frame.field;
    let middle = (left + width / 2.0, top + THE_HATEDS_ROUND_MIDDLE_DOWN);
    let into_the_card = {
        let mut card = THE_HATEDS_CARD
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (asked, began) = *card;
        let began = match stage_frames < asked || stage_frames - asked > A_SWING_IS_FORGOTTEN_AFTER
        {
            true => stage_frames,
            false => began,
        };
        *card = (stage_frames, began);
        stage_frames - began
    };
    if into_the_card < THE_BLUE_STAGE_LASTS {
        return Some((
            middle.0 + THE_HATEDS_CORNERS_ACROSS,
            middle.1 - THE_HATEDS_CORNERS_DOWN,
        ));
    }
    let roses: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| {
            matches!(hazard.kind, Kind::Bullet { .. })
                && hazard.half.0.max(hazard.half.1) >= crate::steer::A_ROSE_IS_HALF_AT_LEAST
        })
        .map(|hazard| hazard.at)
        .collect();
    let (roses_x, roses_y) = match roses.len() {
        0 => (boss_x, boss_y),
        n => {
            let (x, y) = roses
                .iter()
                .fold((0.0, 0.0), |sum, at| (sum.0 + at.0, sum.1 + at.1));
            (x / n as f32, y / n as f32)
        }
    };
    if into_the_card < THE_YELLOW_STAGE_ENDS {
        let (off_x, off_y) = (middle.0 - roses_x, middle.1 - roses_y);
        let off = off_x.hypot(off_y);
        if off >= THE_HATEDS_ROSES_REACH_IN_YELLOW || off < f32::EPSILON {
            return Some(middle);
        }
        let out = THE_HATEDS_ROSES_REACH_IN_YELLOW / off;
        return Some((roses_x + off_x * out, roses_y + off_y * out));
    }
    let (from_x, from_y) = (middle.0 - roses_x, middle.1 - roses_y);
    let mut away = THE_HATEDS_WAY_AWAY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let far = from_x.hypot(from_y);
    if far > THE_HATED_IS_OFF_THE_MIDDLE_PAST {
        *away = (from_x / far, from_y / far);
    }
    Some((
        middle.0 + away.0 * THE_HATED_IS_LEFT_OFF_THE_MIDDLE_BY,
        middle.1 + away.1 * THE_HATED_IS_LEFT_OFF_THE_MIDDLE_BY,
    ))
}
/// **Where th11's `「サブタレイニアンローズ」` is waited out**: below the boss, at the place nearest the run, inside the
/// ring gone out past it, where the rose of the ring coming out at it keeps [`A_ROSE_IS_KEPT_OFF_BY`] round from it
/// all the while it goes out over it — and with none, where the run is from her at the angle that rose will be
/// furthest from; nowhere with no boss or no ring coming.
///
/// **And a place inside the ring coming as well as outside it**, gone in through the ring to meet it: with places
/// outside it alone, the run was told out after them from 192 to 227 from her by the floor to wait for its rose to go
/// round, stood there with none or one of its nine moves clear, and was hit at `4.4,358.8` 414 frames in on every
/// attempt (r17v, 2026-10-07). **And in past where its rose's band will be**, which was not: told just inside the ring,
/// the run went in through it and stopped between it and its rose's band some 20 in from it, and was hit there by the
/// band as the ring went on out, 1,134 frames in (r18, 2026-10-08).
///
/// **Through each ring with its rose gone past and not coming back round before it is out past the run**, the strategy
/// wiki's `バラが通り過ぎてすぐに抜ける×3→1回抜けずに待つ`. **And not at a distance kept, at the angle furthest from where the rose will be as the ring comes**, which
/// was tried: 130 and 152 from her, the run was dodged out ahead of each ring to 190 and 216 rather than through it,
/// and hit out there by the rose of the ring after, three to five times on each of the three chapters kept; and held
/// from going out further, it went through between the ring's bullets wherever it was as the ring came, and was hit
/// eleven times on each as the rose came round onto it there, its bullets turning from 6 to 22 across beside it
/// (2026-10-07).
///
/// **Out of the rose's way as the ring comes and not in one place half way down**, the strategy wiki's
/// `薔薇が通り過ぎたら、瞬時に抜けないと回ってきた薔薇に被弾する` (wikiwiki.jp/thk/地/EX): read off the field traced at
/// `hits-extra5/s7-c16-f45966` (2026-10-07), each ring is 56 bullets going out from her at 1.0 a frame, some 17 apart
/// where the run is, and its rose is a stretch of it some 50 degrees round that goes round it clockwise at some 2.8
/// degrees a frame; waited for half way between the floor and her, `0,280`, the run was hit on the place itself by
/// the first ring's rose 231 frames in, five times on each of the three chapters kept as `f45966`, `f46033` and
/// `f46037`, and the rules' own on the floor 350 to 415 frames in.
///
/// **The same through its fourth stage and not the wiki's own way there**, `最初は赤が回ってくるので、青の列で待機。これを
/// 赤青交互に`: one hit there over fifteen whole Extras, at `36.2,242.6` on Extra 43, where which row is red and which
/// blue is a reading nobody has made. **Nor its other answer, `得意な段階で粘る`**, holding fire to stay in an earlier
/// stage, which keeps the card up longer.
pub(crate) fn out_of_the_subterranean_roses_way(frame: &Frame) -> Option<(f32, f32)> {
    let (boss_x, boss_y) = frame.boss?;
    let small: Vec<(f32, f32)> = frame
        .hazards
        .iter()
        .filter(|hazard| {
            matches!(hazard.kind, Kind::Bullet { .. }) && hazard.half.0 < A_ROSE_IS_BIGGER_THAN
        })
        .map(|hazard| {
            (
                (hazard.at.0 - boss_x).hypot(hazard.at.1 - boss_y),
                (hazard.at.1 - boss_y).atan2(hazard.at.0 - boss_x),
            )
        })
        .collect();
    // **The rings**: the bullets at distances from her each within [`A_RING_IS_READ_BY`] of the next, a ring's worth of
    // them, at their middle distance — and the ring coming out at the run the furthest out of those inside it.
    let mut small = small;
    small.sort_by(|one, other| one.0.total_cmp(&other.0));
    let mut rings: Vec<Vec<(f32, f32)>> = Vec::new();
    for bullet in small {
        match rings.last_mut() {
            Some(ring) if bullet.0 - ring[ring.len() - 1].0 <= A_RING_IS_READ_BY => {
                ring.push(bullet)
            }
            _ => rings.push(vec![bullet]),
        }
    }
    let rings: Vec<(f32, Vec<f32>)> = rings
        .into_iter()
        .filter(|ring| ring.len() >= A_RING_HAS_AT_LEAST)
        .map(|ring| {
            let off = ring.iter().map(|(off, _)| off).sum::<f32>() / ring.len() as f32;
            (off, ring.into_iter().map(|(_, angle)| angle).collect())
        })
        .collect();
    let run_off = (frame.at.0 - boss_x).hypot(frame.at.1 - boss_y);
    let (ring, mut angles) = rings
        .iter()
        .rev()
        .find(|(off, _)| *off < run_off)
        .cloned()?;
    let gone_past = rings
        .iter()
        .map(|(off, _)| *off)
        .find(|off| *off > run_off)
        .unwrap_or(f32::INFINITY);
    angles.sort_by(f32::total_cmp);
    let first = *angles.first()?;
    angles.push(first + std::f32::consts::TAU);
    let rose = angles
        .windows(2)
        .max_by(|one, other| (one[1] - one[0]).total_cmp(&(other[1] - other[0])))
        .map(|pair| (pair[0] + pair[1]) / 2.0)?;
    // **Which way the ring's rose goes round**, read off where it was the frame before as the ring was a frame further
    // in: one ring's rose clockwise and the next's the other way.
    let turns = {
        let mut seen = THE_ROSE_SEEN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let turns = match *seen {
            Some((last_ring, last_rose, last_turns))
                if ring >= last_ring && ring - last_ring <= A_RING_IS_READ_BY =>
            {
                let went = (rose - last_rose + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                match went.abs() < f32::EPSILON {
                    true => last_turns,
                    false => went.signum(),
                }
            }
            _ => 1.0,
        };
        *seen = Some((ring, rose, turns));
        turns
    };
    // **Where to meet the ring at each angle below her**: as far out as it will be as its rose's tail has just gone
    // past that angle, going round the way it goes — if that is inside the ring gone out past the run and on the field.
    // **How far ahead of its middle and behind it the rose reaches round its ring, in degrees, and how far in from its
    // ring and out**: its own bullets, 22 across, about the ring's distance, read off the field — the ring's gap is
    // only where they stand thickest, and they stand some 20 in from the ring and not on it.
    let (lead, tail, inside, outside) = frame
        .hazards
        .iter()
        .filter(|hazard| {
            matches!(hazard.kind, Kind::Bullet { .. }) && hazard.half.0 >= A_ROSE_IS_BIGGER_THAN
        })
        .map(|hazard| {
            let angle = (hazard.at.1 - boss_y).atan2(hazard.at.0 - boss_x);
            let ahead = (((angle - rose) * turns + std::f32::consts::PI)
                .rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI)
                .to_degrees();
            (
                ahead,
                (hazard.at.0 - boss_x).hypot(hazard.at.1 - boss_y) - ring,
            )
        })
        .filter(|(_, out)| out.abs() <= A_ROSE_IS_DEEP_BY)
        .fold(
            (
                A_ROSE_REACHES_AT_LEAST,
                A_ROSE_REACHES_AT_LEAST,
                A_ROSE_IS_THICK_BY,
                A_ROSE_IS_THICK_BY,
            ),
            |(lead, tail, inside, outside), (ahead, out)| {
                (
                    lead.max(ahead),
                    tail.max(-ahead),
                    inside.max(-out),
                    outside.max(out),
                )
            },
        );
    // **How far in from the ring gone past the run its rose reaches**, off its own bullets: further out by the floor its
    // band is the thicker.
    let gone_past_reaches_in = frame
        .hazards
        .iter()
        .filter(|hazard| {
            matches!(hazard.kind, Kind::Bullet { .. }) && hazard.half.0 >= A_ROSE_IS_BIGGER_THAN
        })
        .map(|hazard| gone_past - (hazard.at.0 - boss_x).hypot(hazard.at.1 - boss_y))
        .filter(|inside| (-A_ROSE_IS_DEEP_BY..=A_ROSE_IS_DEEP_BY * 2.0).contains(inside))
        .fold(0.0_f32, f32::max);
    let furthest = (gone_past
        - A_RING_GONE_PAST_IS_KEPT_FROM_BY.max(gone_past_reaches_in + A_ROSE_IS_KEPT_INSIDE_BY))
    .min(THE_ROSES_FURTHEST_FROM_HER);
    let (lowest, highest) = THE_ROSES_LOWEST_ANGLE_BELOW_HER;
    let nearest = THE_ROSES_NEAREST_TO_HER;
    let meet = (0..=((highest - lowest) / 5.0) as usize)
        .map(|step| (lowest + 5.0 * step as f32).to_radians())
        .flat_map(|angle| {
            (0..)
                .map(move |step| nearest + A_RING_IS_READ_BY * step as f32)
                .take_while(move |off| *off <= furthest)
                .map(move |off| (angle, off))
        })
        .filter(|(angle, off)| {
            // Clear of the rose round its ring all the while it goes out over the place, from its outside to its
            // inside: where it is round the ring then, against the angle, is never within its reach and some. **And
            // for a place inside the ring, all the while the run goes in through it on its way there**, the two
            // coming at each other at the ring's going out and the run's going in.
            let (from, to, kept_off) = match *off >= ring {
                true => (
                    (off - ring - outside).max(0.0),
                    (off - ring + inside).max(0.0),
                    A_ROSE_IS_KEPT_OFF_BY,
                ),
                false => {
                    let closing = A_RING_GOES_OUT_BY + A_RUN_GETS_TO_A_PLACE_BY;
                    (
                        (run_off - ring - outside).max(0.0) / closing,
                        (run_off - ring + inside).max(0.0) / closing,
                        A_ROSE_IS_KEPT_OFF_BY + A_ROSE_IS_KEPT_OFF_GOING_IN_BY,
                    )
                }
            };
            (0..)
                .map(|step| from + A_ROSE_IS_WEIGHED_EVERY * step as f32)
                .take_while(|frames| *frames <= to + A_ROSE_IS_WEIGHED_EVERY)
                .all(|frames| {
                    let rose_then = rose
                        + turns * (A_ROSE_GOES_ROUND_BY * frames / A_RING_GOES_OUT_BY).to_radians();
                    let ahead = (((angle - rose_then) * turns + std::f32::consts::PI)
                        .rem_euclid(std::f32::consts::TAU)
                        - std::f32::consts::PI)
                        .to_degrees();
                    ahead > lead + kept_off || ahead < -tail - kept_off
                })
        })
        .map(|(angle, off)| {
            (
                (boss_x + off * angle.cos(), boss_y + off * angle.sin()),
                off,
            )
        })
        // **And on the field**, off its walls.
        .filter(|(place, _)| {
            let (left, top, width, height) = frame.field;
            (left + A_WALL_IS_KEPT_OFF_BY..=left + width - A_WALL_IS_KEPT_OFF_BY).contains(&place.0)
                && (top + A_WALL_IS_KEPT_OFF_BY..=top + height - A_WALL_IS_KEPT_OFF_BY)
                    .contains(&place.1)
        })
        // **And whose way from the run the rose's band does not come over the run on**, the band some 20 in from its
        // ring and as deep as read, the run going at its 3 a frame.
        .filter(|(place, _)| {
            let way = (place.0 - frame.at.0).hypot(place.1 - frame.at.1);
            let frames = way / A_RUN_GETS_TO_A_PLACE_BY;
            (0..)
                .map(|step| A_ROSE_IS_WEIGHED_EVERY * step as f32)
                .take_while(|now| *now <= frames)
                .all(|now| {
                    let k = match frames > 0.0 {
                        true => now / frames,
                        false => 1.0,
                    };
                    let at = (
                        frame.at.0 + (place.0 - frame.at.0) * k,
                        frame.at.1 + (place.1 - frame.at.1) * k,
                    );
                    let off = (at.0 - boss_x).hypot(at.1 - boss_y);
                    let ring_then = ring + A_RING_GOES_OUT_BY * now;
                    let (deep, past) = (
                        inside + A_ROSE_IS_DEEP_TO_THE_RUN_BY,
                        outside + A_ROSE_IS_DEEP_TO_THE_RUN_BY,
                    );
                    !(ring_then - deep..=ring_then + past).contains(&off) || {
                        let rose_then = rose
                            + turns
                                * (A_ROSE_GOES_ROUND_BY * now / A_RING_GOES_OUT_BY).to_radians();
                        let angle = (at.1 - boss_y).atan2(at.0 - boss_x);
                        let ahead = (((angle - rose_then) * turns + std::f32::consts::PI)
                            .rem_euclid(std::f32::consts::TAU)
                            - std::f32::consts::PI)
                            .to_degrees();
                        ahead > lead + A_ROSE_IS_KEPT_OFF_BY
                            || ahead < -tail - A_ROSE_IS_KEPT_OFF_BY
                    }
                })
        })
        // **And, outside the ring, that the run gets to before the ring does — and inside it, that is in past where
        // the rose's band will be as the run gets there**, the band standing in from its ring.
        .filter(|(place, off)| {
            let way = (place.0 - frame.at.0).hypot(place.1 - frame.at.1);
            match *off < ring {
                true => {
                    let ring_then = ring + A_RING_GOES_OUT_BY * way / A_RUN_GETS_TO_A_PLACE_BY;
                    *off <= ring_then - inside - A_ROSE_IS_KEPT_INSIDE_BY
                }
                false => way <= A_RUN_GETS_TO_A_PLACE_BY * (off - ring) / A_RING_GOES_OUT_BY,
            }
        })
        .min_by(|(one, one_off), (other, other_off)| {
            let last = *THE_ROSES_PLACE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let weigh = |place: &(f32, f32), off: f32| {
                let way = (place.0 - frame.at.0).hypot(place.1 - frame.at.1);
                way + (way - A_ROSES_PLACE_IS_NEAR_WITHIN).max(0.0) * A_ROSES_PLACE_COSTS_PAST_NEAR
                    + (off - THE_ROSES_BEST_FROM_HER).abs() * A_ROSES_PLACE_COSTS_OFF_THE_MIDDLE
                    + last.map_or(0.0, |last| (place.0 - last.0).hypot(place.1 - last.1))
                        * A_ROSES_PLACE_COSTS_OFF_THE_LAST
            };
            weigh(one, *one_off).total_cmp(&weigh(other, *other_off))
        })
        .map(|(place, _)| place);
    *THE_ROSES_PLACE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = meet;
    if let Some(meet) = meet {
        return Some(meet);
    }
    // **And with none in time, where the run is from her at the angle the ring's rose will be furthest from as the
    // ring comes to it**, the wiki's `1回抜けずに待つ`.
    let off = run_off.clamp(
        THE_ROSES_NEAREST_TO_HER,
        furthest.max(THE_ROSES_NEAREST_TO_HER),
    );
    let rose_then =
        rose + turns * (A_ROSE_GOES_ROUND_BY * (off - ring) / A_RING_GOES_OUT_BY).to_radians();
    let angle = (0..=((highest - lowest) / 5.0) as usize)
        .map(|step| (lowest + 5.0 * step as f32).to_radians())
        .max_by(|one, other| {
            let from_the_rose = |angle: f32| {
                let off = (angle - rose_then).rem_euclid(std::f32::consts::TAU);
                off.min(std::f32::consts::TAU - off)
            };
            from_the_rose(*one).total_cmp(&from_the_rose(*other))
        })?;
    Some((boss_x + off * angle.cos(), boss_y + off * angle.sin()))
}
/// **How far inside the ring gone out past it the run meets the next**: the depth of that ring's rose across it, some
/// 25, and the run's own half.
///
/// The ring just gone past and its rose are what hit the run when it was waited out 152 from her: dodged 60 out off
/// it to `-37.4,341.7`, 216 from her, 477 frames in, on the first attempt at each of the three chapters kept as
/// `hits-extra5/s7-c16-f45966`, `f46033` and `f46037` (2026-10-07).
///
/// **And not `32`**, which was tried: waiting 212 from her, the run was hit at `-1.5,340.4` 2,544 frames in by the
/// rose of the ring gone out past it as it came round, the rose's bullets 22 across reaching in past it (r10,
/// 2026-10-07).
const A_RING_GONE_PAST_IS_KEPT_FROM_BY: f32 = 44.0;
/// **How far inside the furthest in the rose of the ring gone past reaches the run is kept**: its bullets' half and
/// some.
///
/// **Its reach read off its bullets and not the same for every ring**, which was tried: the pictures of
/// `hits-extra5/s7-c16-f45966` at `--pace=1` show the rose of a ring gone out by the floor as a band some 50 deep, and
/// the run held 44 inside its ring at 228 from her was boxed between it and the ring coming, whose rose had gone a
/// whole round in the 114 frames that ring took to get out there, and hit at `2.5,356.3` 414 frames in on every
/// attempt (r15, 2026-10-07).
const A_ROSE_IS_KEPT_INSIDE_BY: f32 = 16.0;
/// **How near to and far from the boss the run meets a ring**: about the half way between the floor and her, the
/// wiki's `最下段とボスとの中間位置が、薔薇を気にする必要がなく一番避けやすい`.
///
/// **And not as far out as the floor**, which was tried: with no ring gone past it yet to hold it in, the run went out
/// to meet the first ring a whole round of its rose on, 195 and then 281 from her, waited at the floor and was hit
/// there at `-1.2,404.6` 353 frames in — three times on each of the three chapters kept (r5, 2026-10-07).
///
/// **And not only to `190`**, which was tried with places the run could not get to by then left in: 184 straight under
/// her with a ring 12 frames off and its rose 100 round from coming onto it, the one place left was 150 across, and
/// the run went through where it was and was hit 1,073 frames in — three times on each of the three chapters kept,
/// its attempts otherwise going to 2,308 frames (r9, 2026-10-07).
///
/// **And not as near as `100`**, which was tried: 115 from her as the card's next stage began, its first ring's rose
/// came round onto the run where a rose's degrees are fewest across, and the run was hit at `16.0,241.4` 4,922 frames
/// in with -17.9 of room (r24, 2026-10-08).
const THE_ROSES_NEAREST_TO_HER: f32 = 120.0;
const THE_ROSES_FURTHEST_FROM_HER: f32 = 230.0;
/// **How far a frame the run is taken to get towards a place**: under its whole step, the rules dodging on the way.
const A_RUN_GETS_TO_A_PLACE_BY: f32 = 3.0;
/// **Where between those the run is best met by a ring**: the wiki's half way between the floor and her.
const THE_ROSES_BEST_FROM_HER: f32 = 150.0;
/// **What each unit off that distance and off the place told the frame before costs a place, against each unit of the
/// run's way to it**: so that of places as near, the one nearer half way is taken, and one place is kept from frame to
/// frame.
///
/// **And not the place nearest the run alone**, which was tried: the run was told to wait as far out as it could, 229
/// from her by the floor, frame after frame, the place going from `-0,358` to `175,275` to `-0,318` and back as the
/// frames went, with none or one of its nine moves clear — and was hit at `8.7,352.4` 414 frames in (r12, 2026-10-07).
const A_ROSES_PLACE_COSTS_OFF_THE_MIDDLE: f32 = 0.5;
/// **How far off the run a place is near, and what each unit past that costs it besides**: so that while a place near
/// the run keeps clear, one across the field is not taken for being a little better.
///
/// **And not a place weighed by its way alone**, which was tried: played over the field traced alongside the game's
/// own run (scratchpad sim174.py), the place told went 160 across in 8 frames as the one by the run went out of the
/// rose's clear, from `78,193` to `-83,197` and back, the run hit on its way across 3,364 frames in (r21, 2026-10-08).
const A_ROSES_PLACE_IS_NEAR_WITHIN: f32 = 60.0;
const A_ROSES_PLACE_COSTS_PAST_NEAR: f32 = 2.0;
const A_ROSES_PLACE_COSTS_OFF_THE_LAST: f32 = 0.5;
/// The place [`out_of_the_subterranean_roses_way`] last told where a ring was met.
static THE_ROSES_PLACE: Mutex<Option<(f32, f32)>> = Mutex::new(None);
/// **How far round, in degrees, the run keeps off the reach of a ring's rose all the while the rose goes out over it.**
///
/// **And not only just after the rose has gone past**, which was tried: with no angle below her where the ring came to
/// the run just after its rose, the run was sent round her to the angle furthest from the rose, went through the ring
/// half way there 7 frames on with the rose 108 degrees off, and was hit by it as it came round after, 20 frames on,
/// the ring only 15 out past the run — seven times on each of the three chapters kept (r8, 2026-10-07).
///
/// **And not as the ring comes to the place alone**, which was tried: the pictures of `hits-extra5/s7-c16-f45966` at
/// 414 and 2,821 frames in show each rose as a band some 20 in from its ring and not on it, going out over the run
/// some 20 frames after the ring had, and the run hit by it there with the ring gone past (r14, 2026-10-07).
///
/// **And not `20`**, which was tried for the hit at `16.0,241.4` 4,922 frames in, near her where 12 degrees is some 24
/// across, as the card's next stage began (r24, 2026-10-08): played over the field traced without the game it took
/// none in 2,400 frames where 12 took one, but in the game it left the run fewer places clear and its attempts went to
/// 1,372 to 2,746 frames where 12's went to 1,861 to 4,922 (r25, 2026-10-08).
const A_ROSE_IS_KEPT_OFF_BY: f32 = 12.0;
/// **How much further round the rose is kept off for a place gone in through its ring**: the run is not always let in
/// through the ring's bullets as it comes, and the rose comes round on it as it waits.
///
/// **And not the same as for a place waited at**, which was tried: told in through the ring under her to `0,252`, the
/// run stood at `1,278` with none of its nine moves clear as the ring came, and was hit there 832 frames in by the
/// rose's head coming round onto it — on every other attempt at each of the chapters kept (r20, 2026-10-08).
const A_ROSE_IS_KEPT_OFF_GOING_IN_BY: f32 = 30.0;
/// **How far in and out past the rose's band the run on its way to a place is kept**: the run's own half and its
/// bullets'.
///
/// **The way to a place weighed and not the place alone**, which was tried: told to a place the rose kept clear of,
/// `56,282`, the run on its way there at `46,263` was hit by the band going over it, 3,364 frames in on the first
/// attempt at the chapter kept as `hits-extra5/s7-c16-f45966` (r21 and r22, 2026-10-08).
const A_ROSE_IS_DEEP_TO_THE_RUN_BY: f32 = 6.0;
/// How many frames apart the rose is weighed as it goes out over the place.
const A_ROSE_IS_WEIGHED_EVERY: f32 = 2.0;
/// **How far in and out of its ring a rose reaches at least**, with none of its bullets read: their half.
const A_ROSE_IS_THICK_BY: f32 = 12.0;
/// **How far round its middle a rose reaches at least, in degrees**, with none of its bullets read: half its ring's
/// gap.
///
/// **Its reach read off its own bullets and not taken as the ring's gap**, which was tried: the pictures of
/// `hits-extra5/s7-c16-f45966` at 414 and 711 frames in show a rose on a ring out by the floor as a band some 80 to 90
/// degrees round, its bullets 22 across reaching far past the 50 degrees its ring's gap is, and the run hit at its
/// head and on its band where it had been taken to have gone past (r12, 2026-10-07).
const A_ROSE_REACHES_AT_LEAST: f32 = 25.0;
/// **How far in and out of its ring a rose's bullets are read as its own**: the some 20 a rose stands in from its ring
/// and its bullets' half, under the 96 the rings are apart.
const A_ROSE_IS_DEEP_BY: f32 = 40.0;
/// **How far apart in distance from her two bullets of one ring may be**: under the rings' 96 apart, over a frame's
/// going out.
///
/// **One ring read as its bullets within that of each other and not as a band of 8 its bullets stand in**, which was
/// tried: a ring astride two bands was read as two, each with a gap of its own where the other's bullets were, and its
/// rose read 20 degrees off — the attempts going to 414 and 767 frames on two of every five (r11, 2026-10-07).
const A_RING_IS_READ_BY: f32 = 4.0;
/// **How many bullets at one distance are a ring**: some seventh of its 56, a ring gone out by the floor having most of
/// it off the field and some of the rest a rose.
///
/// **And not `12`**, which was tried: a ring gone out past the run by the floor showed 13 to 20 of its bullets on the
/// field and was not read at all, the run let out to 228 from her beside its rose and hit there at `2.5,356.3` 414
/// frames in on every attempt (r16, 2026-10-07).
const A_RING_HAS_AT_LEAST: usize = 8;
/// **How big a bullet is a rose's**: the ring's are 6 across and its rose's 22.
const A_ROSE_IS_BIGGER_THAN: f32 = 5.0;
/// **How far a ring goes out a frame**, read off the field traced (2026-10-07).
const A_RING_GOES_OUT_BY: f32 = 1.0;
/// **How many degrees a frame a ring's rose goes round it**, one ring's clockwise and the next's the other way: 5.6
/// on one frame and none on the next, 3.0 a frame over the whole card wherever it is within 200 of her, its middle
/// followed from frame to frame through the field traced at `hits-extra13/s7-c16-f49325` (scratchpad rosespeed.py,
/// 2026-10-08).
///
/// **And not `2.8`**, which it was, off a few frames of the field traced on 2026-10-07: over the 30 to 50 frames a ring
/// takes to go out over the run that is 6 to 10 degrees of the 12 a place is kept off its rose by, and on
/// `hits-extra12/s7-c16-f53066` and `hits-extra13/s7-c16-f49325` the run at its place was hit by the rose coming round
/// onto it, at `139.7,243.1` 550 frames in and `-115.0,267.0` 6,846 in.
///
/// **The way each goes read and not clockwise for all**, which was tried: every other ring's rose came round onto the
/// run at the place where it was to have gone past, the run hit by its bullets 22 across at `23.1,298.1` and
/// `-54.8,303.4` and the like five times on each of the three chapters kept as `hits-extra5/s7-c16-f45966`, `f46033`
/// and `f46037` (r7, 2026-10-07).
const A_ROSE_GOES_ROUND_BY: f32 = 3.0;
/// The ring [`out_of_the_subterranean_roses_way`] last met, its rose, and the way that rose went round, `1` for
/// clockwise.
static THE_ROSE_SEEN: Mutex<Option<(f32, f32, f32)>> = Mutex::new(None);
/// **The angles below the boss the run waits at, in degrees**: the field's top too near above her for the run to go
/// round over her, and the walls too near either side of her for it to dodge a ring there.
///
/// **And not `20` to `160`**, which was tried: sent to 160, near the left wall, the run was hit there at
/// `-157.5,209.2` 254 frames in as the first ring came with none open, on every attempt after the first at each of
/// the three chapters kept (2026-10-07).
///
/// **And not `40` to `140` alone**, which was tried after it: the card's later stage makes each rose a band some half
/// way round her, which can lie over the whole of that, and the run was hit at its end at `-73.1,265.5` 4,279 frames
/// in and beside her 4,229 in with no angle there clear (r23, 2026-10-08); the walls being kept off by the place's
/// own distance from her, at `10` the run is no nearer the wall than 130 from her is.
const THE_ROSES_LOWEST_ANGLE_BELOW_HER: (f32, f32) = (10.0, 170.0);
/// The stage's frame [`away_from_the_hated`] was last asked on, and the frame the card began on as it saw it.
static THE_HATEDS_CARD: Mutex<(i64, i64)> = Mutex::new((i64::MIN / 2, 0));
/// **How far from the boss of th11's `「嫌われ者のフィロソフィ」` its yellow stage is waited out**, the middle of her round
/// being kept only while she is as far from it: her roses lie some 75 round her.
///
/// **And not the middle wherever she is**: she comes in over the top of her round to `13,162` some 1,440 frames into
/// the card, 84 from it, and the run there was hit at `0.9,266.6` under her roses on `hits-extra4/s7-c15-f43323`
/// (2026-10-08).
const THE_HATEDS_ROSES_REACH_IN_YELLOW: f32 = 140.0;

/// **Whether th11's `「嫌われ者のフィロソフィ」` is in its yellow stage at `stage_frames`**, by the card's start
/// [`away_from_the_hated`] last read.
pub(crate) fn the_hateds_yellow_stage_is_up(stage_frames: i64) -> bool {
    let (asked, began) = *THE_HATEDS_CARD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let into_the_card = stage_frames - began;
    stage_frames >= asked
        && stage_frames - asked <= A_SWING_IS_FORGOTTEN_AFTER
        && (THE_BLUE_STAGE_LASTS..THE_YELLOW_STAGE_ENDS).contains(&into_the_card)
}
/// **How many frames into card 173 its blue stage lasts, waited out at the top right of her round**: the wiki's
/// `青段階：右上が長時間安全` (wikiwiki.jp/thk/地/EX) — through it she goes along the top of her round to its left, up
/// and down its left and along its bottom to the right, and never near its top right, read off the bodies traced at
/// `hits-extra4/s7-c15-f43323`; its pictures there show the card's timer at 42 with the columns still blue and at 40
/// with them yellow, from its 58 at the start (2026-10-07).
///
/// **And not a place weighed against her through it as well**, which was tried: as she set off along the bottom from
/// its left the run left the bottom right for the top right, was carried back down to the middle dodging the blue
/// columns, stood at `18.3,257.4` with no move clear, and was hit there 870 frames in by her roses 115 from her — twice
/// on each of the three chapters kept as `f43323`, `f43580` and `f44225` (2026-10-07).
const THE_BLUE_STAGE_LASTS: i64 = 1080;
/// **How many frames into the card its yellow stage ends**, the wiki's `画面中央を維持するだけでクリアできる`
/// (wikiwiki.jp/thk/地/EX): its bullets are all taken off and the red grid laid some 2,540 frames in, read off the field
/// traced at `hits-extra4/s7-c15-f43580` (2026-10-08), the card's own clock showing 20 at 2,334 and 16 at 2,567.
///
/// **Waited out in the middle of her round through it, and not away from her**, which was tried: she goes round the
/// edge of her round, `-128` to `128` across and `128` to `360` down, her roses some 75 round her, and the place 60 off
/// its middle away from her came onto her way as she came down its left side — the run there was hit at `17.8,330.1`
/// on `hits-extra9/s7-c15-f43581` and nine times running on a whole Extra, and at `-39.2,240.7` on
/// `hits-extra8/s7-c15-f46365`; the middle is 116 from everywhere she goes.
const THE_YELLOW_STAGE_ENDS: i64 = 2520;
/// The way off the middle of her round [`away_from_the_hated`] last told, as the length of one.
static THE_HATEDS_WAY_AWAY: Mutex<(f32, f32)> = Mutex::new((-1.0, 0.0));
/// **How far off the middle of her round the run waits through the yellow stage, away from her**: 176 from her with her
/// on a side of it, past her roses some 90 round her, and a move of the run's no longer than 120 as she goes round.
///
/// **And not a place weighed against her and against where she will be as she goes**, which was tried — a corner of
/// her round or the point of one of its sides nearest the run, whichever was furthest from her for the way to it, no
/// nearer her than 140 and then 180, where she would be 90 frames on straight and then 120 along her round weighed too
/// — and was hit twice on each of the three chapters kept as `hits-extra4/s7-c15-f43323`, `f43580` and `f44225` each
/// time (2026-10-07), at `16.0,402.8`, `117.6,322.6`, `-100.7,391.3` and `-124.4,383.2`: every way from one side of her
/// round to another is 200 or more through the yellow stage's rows, and the run stood within some 20 of where it was
/// for 200 frames with no move clear as she came — the wiki's `少ない動きで画面中央維持` (wikiwiki.jp/thk/地/EX).
const THE_HATED_IS_LEFT_OFF_THE_MIDDLE_BY: f32 = 60.0;
/// **How far off the middle she has to be for the way away from her to be read again**: crossing it through the yellow
/// stage from one corner of her round to the other, the way is kept as she passes.
const THE_HATED_IS_OFF_THE_MIDDLE_PAST: f32 = 40.0;
/// **How far down the middle of the boss's round through card 173 is**: `244` on th11's field, half way between the
/// `128` and `360` she goes along.
pub(crate) const THE_HATEDS_ROUND_MIDDLE_DOWN: f32 = 212.0;
/// **How far across and up from the middle of her round its top right is**, inside the `128` and `116` she goes along.
const THE_HATEDS_CORNERS_ACROSS: f32 = 112.0;
const THE_HATEDS_CORNERS_DOWN: f32 = 100.0;

#[cfg(test)]
mod tests {
    use orb_core::game::{Beam, Frame, Hazard, Kind};

    use super::{
        A_PILLAR_IS_KEPT_OFF_BY, A_PILLAR_WINDS_ONCE_IN, A_PILLARS_FOOT_IS_THROUGH_BELOW, CEDARS,
        CEDARS_LOWEST, CEDARS_TAP, CEDARS_TAP_EVERY, Cedars, MORIYAS, MORIYAS_ASIDE_SHARE,
        MORIYAS_AT_AN_END_PAST_HER_BY, MORIYAS_BOSS_AT, Moriyas, OTENSUI, OTENSUI_ABOVE_THE_FLOOR,
        OTENSUI_BACK_REACHED_WITHIN, OTENSUI_CUT_BACK_ABOVE_THE_FLOOR, OTENSUI_CUT_BACK_BY,
        OTENSUI_REACHED_WITHIN, OTENSUI_SWUNG_BY, OTENSUI_SWUNG_IN_FROM_THE_WALL, Otensui,
        RED_FROG, RED_FROG_BURSTS_AT, RIVER, RIVER_SETS_OFF_AT, River, TWO_BOWS, TWO_BOWS_A_PERIOD,
        TWO_BOWS_ABOVE_THE_FLOOR, TWO_BOWS_AIMED_AT, TWO_BOWS_GOES_ACROSS, TWO_BOWS_GOES_DOWN_BY,
        TWO_BOWS_LURED_FROM, TWO_BOWS_LURED_OFF_THE_CLOSE, TWO_BOWS_LURED_OFF_THE_WALLS,
        TWO_BOWS_LURED_OUT, TWO_BOWS_LURED_UNTIL, TWO_BOWS_OFF_THE_WALLS,
        TWO_BOWS_WAITS_OFF_THE_CLOSE, TWO_BOWS_WAITS_OUT, TwoBows, jade_of_the_loathsome_river,
        moriyas_iron_rings, old_ties_bound_by_cedars, omiwatari_that_god_walked,
        the_miracle_of_otensui, the_red_frog_of_hoei_four, through_tenaga_ashinagas_ring,
        two_bows_two_claps_one_bow,
    };

    const FIELD: (f32, f32, f32, f32) = (-192.0, 0.0, 384.0, 448.0);

    fn a_bullet_at_change(slot: usize, change: u32) -> Hazard {
        Hazard {
            kind: Kind::Bullet {
                hitting: true,
                state: 1,
                next_change: Some(change),
            },
            slot,
            frames: Some(0),
            at: (0.0, 100.0),
            step: Some((0.0, 0.0)),
            half: (4.0, 4.0),
            grows: 0.0,
        }
    }

    // One test and not two: the way's state is a static, which tests run side by side would share.
    #[test]
    fn card_98_is_dodged_by_a_tap_off_each_aim_turning_back_once_every_bullet_is_aimed_again() {
        // Stood still in the middle across, wherever a bullet has put the run, while nothing is aimed at it.
        *CEDARS.lock().unwrap() = None;
        let mut frame = Frame {
            field: FIELD,
            at: (-100.0, 400.0),
            ..Frame::default()
        };
        assert_eq!(old_ties_bound_by_cedars(&frame, Some(10)), (0.0, 400.0));
        frame.hazards = vec![a_bullet_at_change(1, 2)];
        assert_eq!(old_ties_bound_by_cedars(&frame, Some(11)), (0.0, 400.0));

        // A tap up off the place the fire is aimed at, as it is aimed, and stood still after it.
        frame.hazards = vec![a_bullet_at_change(1, 4)];
        assert_eq!(
            old_ties_bound_by_cedars(&frame, Some(12)),
            (0.0, 400.0 - CEDARS_TAP)
        );
        frame.at = (0.0, 390.0);
        assert_eq!(
            old_ties_bound_by_cedars(&frame, Some(13)),
            (0.0, 400.0 - CEDARS_TAP)
        );

        // One tap for a go aimed over several frames.
        frame.hazards = vec![a_bullet_at_change(1, 4), a_bullet_at_change(2, 4)];
        assert_eq!(
            old_ties_bound_by_cedars(&frame, Some(14)),
            (0.0, 400.0 - CEDARS_TAP)
        );
        // And the next go is the next tap, from where the run is.
        frame.hazards.push(a_bullet_at_change(3, 5));
        assert_eq!(
            old_ties_bound_by_cedars(&frame, Some(12 + CEDARS_TAP_EVERY)),
            (0.0, 390.0 - CEDARS_TAP)
        );

        // Waited at the bottom of the band going down, while any bullet is still to be aimed the second time.
        let bottom = FIELD.1 + FIELD.3 - CEDARS_LOWEST;
        *CEDARS.lock().unwrap() = Some(Cedars {
            on: Some(99),
            downwards: true,
            stand: bottom,
            tapped_on: None,
            aimed: Vec::new(),
        });
        frame.at = (0.0, bottom);
        frame.hazards = vec![a_bullet_at_change(1, 5), a_bullet_at_change(2, 4)];
        assert_eq!(old_ties_bound_by_cedars(&frame, Some(100)), (0.0, bottom));
        // The card's other fire stays at its first change and is no part of the two goes.
        frame.hazards = vec![
            a_bullet_at_change(1, 5),
            a_bullet_at_change(3, 5),
            a_bullet_at_change(4, 1),
        ];
        assert_eq!(
            old_ties_bound_by_cedars(&frame, Some(100 + CEDARS_TAP_EVERY)),
            (0.0, bottom - CEDARS_TAP)
        );

        // And a retry of the chapter starts over, stood where the run is.
        frame.at = (0.0, 400.0);
        assert_eq!(old_ties_bound_by_cedars(&frame, Some(5)), (0.0, 400.0));
    }

    /// th10's field, as its player's own bounds are.
    const TH10S_FIELD: (f32, f32, f32, f32) = (-184.0, 32.0, 368.0, 400.0);

    /// A bank's bullet at rest, `age` frames old.
    fn a_banks_bullet(slot: usize, at: (f32, f32), age: i32) -> Hazard {
        Hazard {
            kind: Kind::Bullet {
                hitting: true,
                state: 1,
                next_change: None,
            },
            slot,
            frames: Some(age),
            at,
            step: Some((0.0, 0.0)),
            half: (2.0, 2.0),
            grows: 0.0,
        }
    }

    #[test]
    fn card_101s_run_goes_up_under_the_gap_its_slow_ring_leaves_nearest_it() {
        let a_ring_bullet = |slot: usize, at: (f32, f32), age: i32| Hazard {
            step: Some((0.0, 0.5)),
            ..a_banks_bullet(slot, at, age)
        };
        // The foot of a ring born 425 frames ago, its bullets 12 apart about `x = -1`, `4` and less above the run;
        // a younger ring's further up, and fire stepping faster beside them.
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: (-58.0, 335.0),
            ..Frame::default()
        };
        frame.hazards = (0..11)
            .map(|n| {
                let x = -61.0 + 12.0 * n as f32;
                a_ring_bullet(n, (x, 331.0 - (x + 1.0).abs() / 10.0), 425)
            })
            .collect();
        frame.hazards.push(a_ring_bullet(20, (-3.0, 310.0), 65));
        frame.hazards.push(a_ring_bullet(21, (9.0, 310.0), 65));
        frame.hazards.push(Hazard {
            step: Some((0.0, 2.0)),
            ..a_banks_bullet(22, (-5.0, 333.0), 300)
        });
        assert_eq!(through_tenaga_ashinagas_ring(&frame), Some(-55.0));

        // And nothing to go under once the run is up through it.
        frame.at = (5.0, 320.0);
        frame.hazards.truncate(11);
        assert_eq!(through_tenaga_ashinagas_ring(&frame), None);
    }

    // One test and not two, for the reason card 98's is one.
    #[test]
    fn card_100s_first_attempt_keeps_to_the_spokes_and_a_retry_leads_its_fire_off_where_they_close()
    {
        card_100s_fire_is_led_on_the_side_whose_spoke_comes_second_and_the_run_waits_past_the_close_after();

        // A retry: led off where the spokes close, and the run away from where the fire was aimed as it comes.
        *TWO_BOWS.lock().unwrap() = TwoBows {
            retries: 1,
            ..TwoBows::NONE
        };
        let height = 432.0 - TWO_BOWS_ABOVE_THE_FLOOR;
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: (-100.0, 400.0),
            boss: Some((0.0, 128.0)),
            ..Frame::default()
        };
        // No spoke yet: stood in the middle, wherever the run was — and a bullet older than the card, the fire before
        // it still on its way out, is no throw of the card's coming.
        frame.hazards = vec![a_banks_bullet(1, (-120.0, 120.0), TWO_BOWS_AIMED_AT + 200)];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(30)), (0.0, height));
        frame.hazards = vec![a_banks_bullet(1, (-150.0, 30.0), 10)];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(60)), (0.0, height));
        // The card's first laser at 6°, so its spokes close on 96°, 6° from straight down: led to where 76° out of
        // the boss crosses the height, 20° off the close on the middle's side — and the lasers after it change
        // nothing.
        frame.beams = vec![a_laser(6.0, 1)];
        let led_to = two_bows_two_claps_one_bow(&frame, Some(92));
        assert_eq!(led_to.1, height);
        assert!(
            (led_to.0 - (height - 128.0) / 76.0_f32.to_radians().tan()).abs() < 0.5,
            "{led_to:?}"
        );
        frame.beams = vec![a_laser(186.0, 1), a_laser(10.0, 30)];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(122)), led_to);

        // A lot thrown up and still to take aim: stood there.
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(160)), led_to);
        frame.at = led_to;
        frame.hazards = vec![a_banks_bullet(1, (-150.0, 30.0), TWO_BOWS_AIMED_AT - 1)];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(199)), led_to);
        // The throw's first lot aimed while its second is still to: stood there for the second.
        frame.hazards = vec![
            a_banks_bullet(1, (-140.0, 60.0), TWO_BOWS_AIMED_AT + 10),
            a_banks_bullet(2, (120.0, 30.0), TWO_BOWS_AIMED_AT - 20),
        ];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(210)), led_to);

        // Both aimed and coming: down and to the side of where they were aimed away from where the spokes close —
        // the right, the close crossing the height left of it — and the same side for the rest of the throw,
        // wherever the run has got to.
        frame.hazards = vec![a_banks_bullet(1, (-120.0, 120.0), TWO_BOWS_AIMED_AT + 40)];
        let aside = (
            led_to.0 + TWO_BOWS_GOES_ACROSS,
            led_to.1 + TWO_BOWS_GOES_DOWN_BY,
        );
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(240)), aside);
        frame.at = (100.0, 330.0);
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(241)), aside);

        // And once it is through the height it was aimed at, back up to the place for the next throw — straight
        // up first where the rules' dodging carried the run lower while the lot is still spreading below that
        // height, so as not to cross it, and across at the height after.
        frame.hazards = vec![a_banks_bullet(1, (-30.0, 360.0), TWO_BOWS_AIMED_AT + 180)];
        frame.at = aside;
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(410)), led_to);
        frame.at = (aside.0, 400.0);
        assert_eq!(
            two_bows_two_claps_one_bow(&frame, Some(411)),
            (aside.0, led_to.1)
        );
        frame.hazards = vec![a_banks_bullet(1, (-30.0, 440.0), TWO_BOWS_AIMED_AT + 200)];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(412)), led_to);

        // A throw aimed where the place to the side would be past a wall goes to the wall's foot short of it.
        frame.at = (120.0, led_to.1);
        frame.hazards = vec![a_banks_bullet(1, (-150.0, 30.0), TWO_BOWS_AIMED_AT - 1)];
        two_bows_two_claps_one_bow(&frame, Some(439));
        frame.hazards = vec![a_banks_bullet(1, (-120.0, 120.0), TWO_BOWS_AIMED_AT + 40)];
        assert_eq!(
            two_bows_two_claps_one_bow(&frame, Some(480)),
            (
                TH10S_FIELD.0 + TH10S_FIELD.2 - TWO_BOWS_OFF_THE_WALLS,
                led_to.1 + TWO_BOWS_GOES_DOWN_BY
            )
        );

        // A retry of the chapter is played the next way: led to the middle, the same close notwithstanding — and
        // the one after that, led 30° off it, to where 66° crosses.
        frame.beams = Vec::new();
        frame.hazards = vec![a_banks_bullet(1, (-150.0, 30.0), 10)];
        two_bows_two_claps_one_bow(&frame, Some(1));
        frame.beams = vec![a_laser(6.0, 1)];
        assert_eq!(two_bows_two_claps_one_bow(&frame, Some(92)), (0.0, height));
        frame.beams = Vec::new();
        two_bows_two_claps_one_bow(&frame, Some(1));
        frame.beams = vec![a_laser(6.0, 1)];
        let led_to_again = two_bows_two_claps_one_bow(&frame, Some(92));
        assert!(
            (led_to_again.0 - (height - 128.0) / 66.0_f32.to_radians().tan()).abs() < 0.5,
            "{led_to_again:?}"
        );
    }

    /// The first attempt's half of card 100's one test.
    fn card_100s_fire_is_led_on_the_side_whose_spoke_comes_second_and_the_run_waits_past_the_close_after()
     {
        *TWO_BOWS.lock().unwrap() = TwoBows::NONE;
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: (-100.0, 400.0),
            boss: Some((0.0, 128.0)),
            ..Frame::default()
        };
        let out_of_the_boss = |degrees: f32, out: f32| {
            let (sin, cos) = degrees.to_radians().sin_cos();
            (cos * out, 128.0 + sin * out)
        };
        let near =
            |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01;
        // The card's first laser at 0°, on frame 92, so its spokes close on straight down: led `30°` round on
        // the side whose spoke comes second, the left, until the period's frame 170.
        frame.beams = vec![a_laser(0.0, 1)];
        let lured = out_of_the_boss(90.0 + TWO_BOWS_LURED_OFF_THE_CLOSE, TWO_BOWS_LURED_OUT);
        assert!(near(two_bows_two_claps_one_bow(&frame, Some(92)), lured));
        frame.beams = Vec::new();
        assert!(near(
            two_bows_two_claps_one_bow(&frame, Some(92 + TWO_BOWS_LURED_UNTIL - 1)),
            lured
        ));
        // Then waiting past the close on the right until the next period's fire is led there.
        let waits = out_of_the_boss(90.0 - TWO_BOWS_WAITS_OFF_THE_CLOSE, TWO_BOWS_WAITS_OUT);
        assert!(near(
            two_bows_two_claps_one_bow(&frame, Some(92 + TWO_BOWS_LURED_UNTIL)),
            waits
        ));
        let next = 92 + TWO_BOWS_A_PERIOD;
        assert!(near(
            two_bows_two_claps_one_bow(&frame, Some(next + TWO_BOWS_LURED_FROM - 1)),
            waits
        ));
        let lured_next = out_of_the_boss(90.0 - TWO_BOWS_LURED_OFF_THE_CLOSE, TWO_BOWS_LURED_OUT);
        assert!(near(
            two_bows_two_claps_one_bow(&frame, Some(next + TWO_BOWS_LURED_FROM)),
            lured_next
        ));
        // And past the close on the left once that is through, the first spoke coming from the left this time.
        assert!(near(
            two_bows_two_claps_one_bow(&frame, Some(next + TWO_BOWS_LURED_UNTIL)),
            out_of_the_boss(90.0 + TWO_BOWS_WAITS_OFF_THE_CLOSE, TWO_BOWS_WAITS_OUT)
        ));

        // A place past a wall is brought in along its own line out of the boss, held well off the wall.
        *TWO_BOWS.lock().unwrap() = TwoBows::NONE;
        frame.boss = Some((-100.0, 128.0));
        frame.beams = vec![a_laser(20.0, 1)];
        let wall = TH10S_FIELD.0 + TWO_BOWS_LURED_OFF_THE_WALLS;
        let (sin, cos) = (110.0_f32 + TWO_BOWS_LURED_OFF_THE_CLOSE)
            .to_radians()
            .sin_cos();
        let out = (-100.0 - wall) / -cos;
        assert!(near(
            two_bows_two_claps_one_bow(&frame, Some(92)),
            (wall, 128.0 + sin * out)
        ));
    }

    #[test]
    fn card_99s_run_steps_aside_of_a_pillar_being_laid_down_to_it_and_is_left_to_the_rules_otherwise()
     {
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: (60.0, 370.0),
            ..Frame::default()
        };
        // A pillar about `x = 70`, winding `25` either side, laid down to `y = 300` so far — above the run.
        let pillar = |foot: f32| -> Vec<Hazard> {
            (0..A_PILLAR_WINDS_ONCE_IN)
                .map(|age| {
                    let wound = 25.0
                        * (age as f32 / A_PILLAR_WINDS_ONCE_IN as f32 * std::f32::consts::TAU)
                            .sin();
                    a_banks_bullet(age as usize, (70.0 + wound, foot - 4.0 * age as f32), age)
                })
                .collect()
        };
        frame.hazards = pillar(300.0);
        // Aside of its whole width, on the left where the field is wider, at the run's own height.
        let (x, y) = omiwatari_that_god_walked(&frame).unwrap();
        assert!(x < 45.0 - A_PILLAR_IS_KEPT_OFF_BY + 0.5, "{x}");
        assert_eq!(y, 370.0);
        // Laid past the run: left to the rules.
        frame.hazards = pillar(370.0 + A_PILLARS_FOOT_IS_THROUGH_BELOW + 4.0);
        assert_eq!(omiwatari_that_god_walked(&frame), None);
        // Nor anything to do with a run already clear of its width.
        frame.hazards = pillar(300.0);
        frame.at = (-60.0, 370.0);
        assert_eq!(omiwatari_that_god_walked(&frame), None);
    }

    #[test]
    fn card_102s_run_waits_above_the_boss_until_a_ring_comes_at_it() {
        let boss = (0.0, MORIYAS_BOSS_AT);
        // Above her by more than a ring's reach, short of the field's top.
        let frame = Frame {
            field: TH10S_FIELD,
            at: (0.0, MORIYAS_BOSS_AT - MORIYAS_AT_AN_END_PAST_HER_BY - 8.0),
            boss: Some(boss),
            ..Frame::default()
        };
        *MORIYAS.lock().unwrap() = Moriyas {
            on: Some(600),
            above: true,
        };
        // No ring coming: on up to the top, above her.
        assert_eq!(
            moriyas_iron_rings(&frame, Some(601), 0.0),
            (boss.0, TH10S_FIELD.1)
        );
        assert!(MORIYAS.lock().unwrap().above);
    }

    #[test]
    fn card_102s_retry_waits_a_share_of_its_aside_off_the_boss() {
        let boss = (0.0, MORIYAS_BOSS_AT);
        let frame = Frame {
            field: TH10S_FIELD,
            at: (0.0, MORIYAS_BOSS_AT - MORIYAS_AT_AN_END_PAST_HER_BY - 8.0),
            boss: Some(boss),
            ..Frame::default()
        };
        *MORIYAS.lock().unwrap() = Moriyas {
            on: Some(600),
            above: true,
        };
        assert_eq!(
            moriyas_iron_rings(&frame, Some(601), -80.0),
            (boss.0 - 80.0 * MORIYAS_ASIDE_SHARE, TH10S_FIELD.1)
        );
    }

    // One test and not two, for the reason card 98's is one.
    #[test]
    fn card_90s_run_goes_where_the_dodging_carries_it_and_short_of_a_wall_swings_up_on_to_it_and_cuts_back()
     {
        *OTENSUI.lock().unwrap() = Otensui::NONE;
        let low = TH10S_FIELD.1 + TH10S_FIELD.3 - OTENSUI_ABOVE_THE_FLOOR;
        let left_wall = TH10S_FIELD.0;
        let right_wall = TH10S_FIELD.0 + TH10S_FIELD.2;
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: (-60.0, 380.0),
            ..Frame::default()
        };
        // Along the low line wherever the run is across.
        assert_eq!(the_miracle_of_otensui(&frame, Some(500), 0.0), (-60.0, low));
        let short_of_it = left_wall + OTENSUI_SWUNG_IN_FROM_THE_WALL + 1.0;
        frame.at = (short_of_it, low);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(501), 0.0),
            (short_of_it, low)
        );

        // Carried within reach of the left wall: swung up and on towards it, held there on the way.
        let up = TH10S_FIELD.1 + TH10S_FIELD.3 - OTENSUI_CUT_BACK_ABOVE_THE_FLOOR;
        let from = left_wall + OTENSUI_SWUNG_IN_FROM_THE_WALL - 1.0;
        frame.at = (from, low);
        let swung_to = from - OTENSUI_SWUNG_BY;
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(502), 0.0),
            (swung_to, up)
        );
        frame.at = (from - OTENSUI_SWUNG_BY / 2.0, 380.0);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(505), 0.0),
            (swung_to, up)
        );
        // Across as far as the swing goes and still short of its height: swinging yet.
        frame.at = (swung_to, up + 2.0 * OTENSUI_REACHED_WITHIN);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(507), 0.0),
            (swung_to, up)
        );

        // Swung: at once back past where it set off, up off the low line, held there on the way.
        let back_to = from + OTENSUI_CUT_BACK_BY;
        frame.at = (swung_to, up);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(510), 0.0),
            (back_to, up)
        );
        frame.at = (from, up);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(520), 0.0),
            (back_to, up)
        );

        // Back, though held some steps short of the place by the fire it came across: on along the low line wherever
        // the dodging carries it.
        let held_short = back_to - OTENSUI_BACK_REACHED_WITHIN + 1.0;
        frame.at = (held_short, up);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(530), 0.0),
            (held_short, low)
        );
        frame.at = (back_to + 30.0, low);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(560), 0.0),
            (back_to + 30.0, low)
        );

        // Carried right up to the right wall: swung only as far as the wall, and back from there.
        frame.at = (right_wall - 4.0, low);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(900), 0.0),
            (right_wall, up)
        );
        frame.at = (right_wall, up);
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(901), 0.0),
            (right_wall - 4.0 - OTENSUI_CUT_BACK_BY, up)
        );

        // And a retry of the chapter starts over, with no cut back under way.
        frame.at = (0.0, low);
        assert_eq!(the_miracle_of_otensui(&frame, Some(10), 0.0), (0.0, low));
    }

    /// **A retry's aside moves where card 90's cut back starts**, a quarter of it: carried to `24` inside the reach
    /// of the left wall, the run swings at once with nothing aside, and with `96` aside the other way it is still
    /// short of the reach and goes on along the low line.
    #[test]
    fn card_90s_cut_back_starts_further_from_or_nearer_the_wall_on_a_retry() {
        *OTENSUI.lock().unwrap() = Otensui::NONE;
        let low = TH10S_FIELD.1 + TH10S_FIELD.3 - OTENSUI_ABOVE_THE_FLOOR;
        let up = TH10S_FIELD.1 + TH10S_FIELD.3 - OTENSUI_CUT_BACK_ABOVE_THE_FLOOR;
        let from = TH10S_FIELD.0 + OTENSUI_SWUNG_IN_FROM_THE_WALL - 23.0;
        let frame = Frame {
            field: TH10S_FIELD,
            at: (from, low),
            ..Frame::default()
        };
        assert_eq!(the_miracle_of_otensui(&frame, Some(10), -96.0), (from, low));
        *OTENSUI.lock().unwrap() = Otensui::NONE;
        assert_eq!(
            the_miracle_of_otensui(&frame, Some(10), 0.0),
            (from - OTENSUI_SWUNG_BY, up)
        );
    }

    /// One of the boss's lasers, `frames` old, let go at `degrees`.
    fn a_laser(degrees: f32, frames: i32) -> Beam {
        Beam {
            slot: frames as usize,
            origin: (0.0, 128.0),
            angle: degrees.to_radians(),
            frames,
            near: 0.0,
            far: 128.0,
            grows: 0.0,
            half_across: 4.0,
            fires_in: 0,
            fading: false,
        }
    }

    // One test and not two, for the reason card 98's is one.
    #[test]
    fn card_103_is_stood_where_the_rivers_cross_until_a_bank_is_about_to_set_off_at_the_run() {
        *RIVER.lock().unwrap() = River::NONE;
        let crossing = (0.0, 280.0);
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: crossing,
            half: (1.0, 1.0),
            ..Frame::default()
        };
        // A bank either side, `40` off, a whole letting go from setting off: stood where the rivers cross.
        frame.hazards = vec![
            a_banks_bullet(1, (-40.0, 280.0), 0),
            a_banks_bullet(2, (40.0, 280.0), 0),
        ];
        assert_eq!(jade_of_the_loathsome_river(&frame, Some(10), 0.0), crossing);
        // The same two a frame from setting off at it: gone on from there, and not across into either.
        *RIVER.lock().unwrap() = River::NONE;
        frame.hazards = vec![
            a_banks_bullet(1, (-40.0, 280.0), RIVER_SETS_OFF_AT - 1),
            a_banks_bullet(2, (40.0, 280.0), RIVER_SETS_OFF_AT - 1),
        ];
        let (x, y) = jade_of_the_loathsome_river(&frame, Some(10), 0.0);
        assert!(
            (y - crossing.1).abs() > (x - crossing.0).abs(),
            "went to {x},{y} with a bank's bullet either side about to set off"
        );
    }

    // One test and not two, for the reason card 98's is one.
    #[test]
    fn card_107_is_stood_through_on_the_low_line_until_rice_beside_the_run_is_about_to_burst() {
        *RED_FROG.lock().unwrap() = River::NONE;
        let low = (0.0, 368.0);
        let mut frame = Frame {
            field: TH10S_FIELD,
            at: low,
            half: (1.0, 1.0),
            ..Frame::default()
        };
        // A clump of a double's rice `40` to the side, a whole age from bursting: stood on the low line.
        let a_clump = |age| {
            (1..=12)
                .map(|slot| a_banks_bullet(slot, (40.0, 368.0), age))
                .collect()
        };
        frame.hazards = a_clump(0);
        assert_eq!(the_red_frog_of_hoei_four(&frame, Some(10), 0.0), low);
        // The same clump a frame from bursting, each grain going `90` its own way: gone on from it.
        *RED_FROG.lock().unwrap() = River::NONE;
        frame.hazards = a_clump(RED_FROG_BURSTS_AT - 1);
        let there = the_red_frog_of_hoei_four(&frame, Some(10), 0.0);
        assert!(
            super::distance(there, (40.0, 368.0)) > 40.0,
            "went to {there:?} with rice beside the run about to burst"
        );
    }

    /// **Card 107 played in a model of it**, `RUNS` times (12 where unset), for weighing a change to its way against
    /// the last: the frame and the place each play was first hit at, if it was, and how many were.
    ///
    /// **The model is the card's field as traced** (keep\xpd10-fresh, `--trace-the-field-in=107`, 2026-10-02): the
    /// boss at `0,224`; a double from frame 81 and a second from 1,341, each going to where the run is, standing,
    /// flying off and resting — 90, 34, 56 and 24 frames — at a pace falling by `0.965` a frame, `15.5` to start
    /// flying off, and laying a grain where it is three frames in two; and every grain bursting at 412 frames old
    /// its own way at a pace of [`a_bursts_pace`], gone 36 frames later. The run takes the focused step nearest the
    /// place the way gives it, and a play ends at its first hit, the game's own box test.
    #[test]
    #[ignore = "a measurement, minutes long: cargo test --release -- --ignored card_107_played_in_a_model"]
    fn card_107_played_in_a_model() {
        let runs = std::env::var("RUNS")
            .ok()
            .and_then(|it| it.parse().ok())
            .unwrap_or(12u64);
        let mut hit = 0;
        for seed in 1..=runs {
            let first = a_play_of_card_107(seed);
            println!("play {seed}: {first:?}");
            hit += usize::from(first.is_some());
        }
        println!("{hit} of {runs} plays hit");
    }

    /// One play of [`card_107_played_in_a_model`]'s model: the frame of its first hit and the run's place then.
    fn a_play_of_card_107(seed: u64) -> Option<(i32, (f32, f32))> {
        use super::{A_BURST_LASTS, a_bursts_pace};

        const BOSS: (f32, f32) = (0.0, 224.0);
        const SLOWS: f32 = 0.965;
        const COMING: i32 = 90;
        const STANDING: i32 = 34;
        const GOING: i32 = 56;
        const RESTING: i32 = 24;
        const FLIES_OFF_AT: f32 = 15.5;
        const THE_CARD_LASTS: i32 = 2400;
        const DOUBLES_COME_OUT_AT: [i32; 2] = [81, 1341];

        let mut state = seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
        let mut draw = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f32 / (1u64 << 53) as f32
        };
        *RED_FROG.lock().unwrap() = River::NONE;
        let field = TH10S_FIELD;
        let mut run = (draw() * 200.0 - 100.0, 368.0);
        /// A double: where it is, its step, the frame its stretch began on, and which of the four it is in.
        struct Double {
            at: (f32, f32),
            step: (f32, f32),
            since: i32,
            stretch: u8,
        }
        /// A grain: where it is, its age, and its heading once it has gone.
        #[derive(Clone)]
        struct Grain {
            at: (f32, f32),
            age: i32,
            heading: (f32, f32),
        }
        let mut doubles: Vec<Double> = Vec::new();
        let mut rice: Vec<Grain> = Vec::new();
        // The nine moves at the focused step and at the whole one.
        let moves = |straight: f32, diagonal: f32| -> Vec<(f32, f32)> {
            (-1..=1)
                .flat_map(|dx| (-1..=1).map(move |dy| (dx as f32, dy as f32)))
                .map(|(dx, dy)| match dx != 0.0 && dy != 0.0 {
                    true => (dx * diagonal, dy * diagonal),
                    false => (dx * straight, dy * straight),
                })
                .collect()
        };
        let (focused, whole) = (moves(2.0, 1.41), moves(4.5, 3.18));
        for frame in 1..=THE_CARD_LASTS {
            if DOUBLES_COME_OUT_AT.contains(&frame) {
                doubles.push(Double {
                    at: BOSS,
                    step: (0.0, 0.0),
                    since: frame,
                    stretch: 3,
                });
            }
            for double in doubles.iter_mut() {
                let lasted = frame - double.since;
                let over = match double.stretch {
                    0 => lasted >= COMING,
                    1 => lasted >= STANDING,
                    2 => lasted >= GOING,
                    _ => lasted >= RESTING,
                };
                if over {
                    double.stretch = (double.stretch + 1) % 4;
                    double.since = frame;
                    double.step = match double.stretch {
                        0 => {
                            let (dx, dy) = (run.0 - double.at.0, run.1 - double.at.1);
                            (dx * (1.0 - SLOWS), dy * (1.0 - SLOWS))
                        }
                        2 => {
                            let angle = draw() * std::f32::consts::TAU;
                            (FLIES_OFF_AT * angle.cos(), FLIES_OFF_AT * angle.sin())
                        }
                        _ => (0.0, 0.0),
                    };
                }
                double.at = (double.at.0 + double.step.0, double.at.1 + double.step.1);
                double.step = (double.step.0 * SLOWS, double.step.1 * SLOWS);
                let grains = 1 + usize::from(frame % 2 == 0);
                let laid = Grain {
                    at: double.at,
                    age: 0,
                    heading: (0.0, 0.0),
                };
                rice.extend(std::iter::repeat_n(laid, grains));
            }
            for grain in rice.iter_mut() {
                grain.age += 1;
                if grain.age == RED_FROG_BURSTS_AT {
                    let angle = draw() * std::f32::consts::TAU;
                    grain.heading = (angle.cos(), angle.sin());
                }
                if grain.age >= RED_FROG_BURSTS_AT {
                    let pace = a_bursts_pace(grain.age - RED_FROG_BURSTS_AT);
                    grain.at = (
                        grain.at.0 + grain.heading.0 * pace,
                        grain.at.1 + grain.heading.1 * pace,
                    );
                }
            }
            rice.retain(|grain| grain.age < RED_FROG_BURSTS_AT + A_BURST_LASTS);
            let mut hazards: Vec<Hazard> = rice
                .iter()
                .enumerate()
                .map(|(slot, grain)| {
                    let pace = match grain.age >= RED_FROG_BURSTS_AT {
                        true => a_bursts_pace(grain.age - RED_FROG_BURSTS_AT),
                        false => 0.0,
                    };
                    let mut hazard = a_banks_bullet(slot, grain.at, grain.age);
                    hazard.step = Some((grain.heading.0 * pace, grain.heading.1 * pace));
                    hazard
                })
                .collect();
            let body = |slot, at, half| Hazard {
                kind: Kind::Body { solid: false },
                slot,
                frames: None,
                at,
                step: None,
                half,
                grows: 0.0,
            };
            hazards.push(body(10_000, BOSS, (16.0, 16.0)));
            for (n, double) in doubles.iter().enumerate() {
                hazards.push(body(10_001 + n, double.at, (12.0, 12.0)));
            }
            let seen = Frame {
                field,
                at: run,
                half: (1.0, 1.0),
                focused_straight: (2.0, 2.0),
                focused_diagonal: (1.41, 1.41),
                hazards,
                ..Frame::default()
            };
            let wanted = the_red_frog_of_hoei_four(&seen, Some(frame), 0.0);
            let moves = match super::a_pace_looked_ahead_wants_the_whole_step() {
                true => &whole,
                false => &focused,
            };
            let step = moves
                .iter()
                .copied()
                .min_by(|a, b| {
                    let off = |m: &(f32, f32)| super::distance((run.0 + m.0, run.1 + m.1), wanted);
                    off(a).total_cmp(&off(b))
                })
                .unwrap_or((0.0, 0.0));
            run = (
                (run.0 + step.0).clamp(field.0, field.0 + field.2),
                (run.1 + step.1).clamp(field.1, field.1 + field.3),
            );
            if rice
                .iter()
                .any(|grain| (grain.at.0 - run.0).abs() < 3.0 && (grain.at.1 - run.1).abs() < 3.0)
            {
                return Some((frame, run));
            }
        }
        None
    }

    /// A flare of th11's card 147: a solid body of reach `reach` with its middle at `at`.
    fn a_flare(slot: usize, at: (f32, f32), reach: f32) -> Hazard {
        Hazard {
            kind: Kind::Body { solid: true },
            slot,
            frames: Some(0),
            at,
            step: None,
            half: (reach, reach),
            grows: 0.0,
        }
    }

    /// Whether a flare of `reach` at `at` covers `place` at any frame of its fall, as card 147's flares fall.
    fn the_flare_comes_down_on(at: (f32, f32), reach: f32, place: (f32, f32)) -> bool {
        (0..200).any(|ahead| {
            let fallen = ahead as f32 * super::A_FLARE_FALLS_BY;
            let reach =
                (reach - super::A_FLARE_SHRINKS_BY * fallen).max(super::A_FLARE_LANDS_AT_LEAST);
            (place.0 - at.0).hypot(place.1 - (at.1 + fallen)) < reach
        })
    }

    #[test]
    fn card_143_is_waited_out_in_the_gap_the_run_stands_in_and_not_across_a_suns_line() {
        // A ring born at `0,128` over the boss at `0,160`: suns going out at 40, 85 and 130 degrees, the run on the floor
        // right of the line the 85-degree sun comes down along, which meets the floor at `27` — the gap nearest the run
        // being the one across that line, at `-96`, and the one it stands in at `158`.
        *super::BETWEEN_THE_SUNS.lock().unwrap() = None;
        let middle: (f32, f32) = (0.0, 128.0);
        let sun = |degrees: f32, out: f32| {
            let angle = degrees.to_radians();
            (middle.0 + out * angle.cos(), middle.1 + out * angle.sin())
        };
        let floor = 432.0;
        let frame = Frame {
            field: FIELD,
            at: (30.0, floor),
            boss: Some((0.0, 160.0)),
            hazards: vec![
                a_flare(1, sun(40.0, 180.0), 87.0),
                a_flare(2, sun(85.0, 180.0), 87.0),
                a_flare(3, sun(130.0, 180.0), 87.0),
            ],
            ..Frame::default()
        };
        let x = super::between_the_suns(&frame, floor).expect("a place across");
        let the_line_coming_down = middle.0 + (floor - middle.1) / 85_f32.to_radians().tan();
        assert!(
            x > the_line_coming_down,
            "{x} is across the line at {the_line_coming_down}"
        );
    }

    #[test]
    fn card_143_is_waited_out_in_the_gap_the_run_stands_in_where_it_is_held_below_the_height_waited_at()
     {
        // The ring of hits/s6-c5-f10070's first hit: suns going out at 22, 67, 112 and 157 degrees from `0,128`, the
        // place waited at `409` and the run held on the floor under it at `123` — left of the line the 67-degree sun
        // comes down along where the run is, at `129`, and right of it at the height waited at, at `119`, the sun 240
        // out and too near that line to be got across before it.
        *super::BETWEEN_THE_SUNS.lock().unwrap() = None;
        let middle: (f32, f32) = (0.0, 128.0);
        let sun = |degrees: f32, out: f32| {
            let angle = degrees.to_radians();
            (middle.0 + out * angle.cos(), middle.1 + out * angle.sin())
        };
        let (waited_at, floor) = (409.0, 432.0);
        let frame = Frame {
            field: FIELD,
            at: (123.0, floor),
            boss: Some((0.0, 160.0)),
            hazards: vec![
                a_flare(1, sun(22.0, 240.0), 87.0),
                a_flare(2, sun(67.0, 240.0), 87.0),
                a_flare(3, sun(112.0, 240.0), 87.0),
                a_flare(4, sun(157.0, 240.0), 87.0),
            ],
            ..Frame::default()
        };
        let x = super::between_the_suns(&frame, waited_at).expect("a place across");
        let the_gap = middle.0 + (waited_at - middle.1) / 89.5_f32.to_radians().tan();
        assert!((x - the_gap).abs() < 4.0, "{x} is not the gap at {the_gap}");
    }

    #[test]
    fn card_143_is_waited_out_in_front_of_the_boss_where_no_ring_is_coming() {
        *super::BETWEEN_THE_SUNS.lock().unwrap() = Some(150.0);
        let frame = Frame {
            field: FIELD,
            at: (150.0, 432.0),
            boss: Some((-20.0, 160.0)),
            ..Frame::default()
        };
        assert_eq!(
            super::between_the_suns(&frame, 432.0),
            Some(-20.0),
            "the last gap held and not in front of her"
        );
    }

    #[test]
    fn card_11_is_waited_out_under_the_boss_then_led_up_under_her_and_brought_down_beside_the_trail()
     {
        let boss = (128.0, 224.0);
        let run = (128.0, 400.0);
        let on = |monster: Option<(f32, f32)>, stage_frames: i64| {
            let frame = Frame {
                field: FIELD,
                at: run,
                boss: Some(boss),
                hazards: match monster {
                    Some(at) => vec![a_flare(1, at, 11.0)],
                    None => vec![a_flare(2, (0.0, 0.0), 11.0)],
                },
                ..Frame::default()
            };
            super::after_the_green_eyed_monster(&frame, stage_frames)
        };
        let (far, near) = (Some((-184.0, 61.0)), Some((run.0 - 40.0, run.1 - 40.0)));
        assert_eq!(on(None, 1_000), None, "before the first monster");
        let came = 1_001;
        let low = FIELD.1 + FIELD.3 - super::THE_GREEN_EYED_MONSTER_IS_WAITED_FOR_ABOVE_THE_FLOOR;
        let highest = boss.1 + super::THE_GREEN_EYED_MONSTER_IS_LED_UP_TO;
        let by = super::THE_GREEN_EYED_MONSTER_IS_LED_UP_BY;
        assert_eq!(on(far, came), Some((boss.0, low)), "under her as it comes");
        assert_eq!(
            on(far, came + 150),
            Some((boss.0, low)),
            "and while it is far off"
        );
        let reached = came + 170;
        assert_eq!(
            on(near, reached),
            Some((boss.0, low)),
            "as it reaches the run"
        );
        assert_eq!(
            on(near, reached + 10),
            Some((boss.0, low - 10.0 * by)),
            "led up under her"
        );
        let up = ((low - highest) / by) as i64;
        let (x, y) = on(near, reached + up + 10).expect("a place");
        assert!(x < boss.0, "{x} is not beside the trail on the wider side");
        assert!(y > highest, "{y} is not on its way back down");
        assert_eq!(
            on(near, came - 1),
            None,
            "on a chapter taken again from before it"
        );
    }

    #[test]
    fn card_127s_rice_is_played_on_setting_off_at_187_at_where_the_run_was_and_speeding_up() {
        let grain = |age: i32, step: (f32, f32)| Hazard {
            frames: Some(age),
            step: Some(step),
            ..a_trails_bullet(1, (100.0, 300.0))
        };
        let was = (0.0, 300.0);
        let run_was = |_: i32| was;
        let played = |hazard: &Hazard| {
            super::Fire::of(
                hazard,
                (1.0, 1.0),
                super::SetsOff::AtWhereTheRunWas,
                None,
                &run_was,
            )
            .expect("a grain of rice")
        };
        let mut at_rest = played(&grain(185, (0.0, 0.0)));
        let run = (50.0, 432.0);
        at_rest.goes_on(run);
        assert_eq!(at_rest.at, (100.0, 300.0), "at rest at 186 frames old");
        at_rest.goes_on(run);
        assert!(
            (at_rest.at.0 - (100.0 - 0.058)).abs() < 0.001 && at_rest.at.1 == 300.0,
            "set off at 187 at where the run was and not where it is, at {:?}",
            at_rest.at
        );
        for _ in 0..200 {
            at_rest.goes_on(run);
        }
        let gone = 100.0 - at_rest.at.0;
        assert!(
            gone > 100.0 && gone < 170.0,
            "sped up to no more than a frame a frame, {gone} gone"
        );
        let mut speeding = played(&grain(200, (-0.2, 0.0)));
        speeding.goes_on(run);
        assert!(
            (speeding.at.0 - (100.0 - 0.2 - 1.0 / 120.0)).abs() < 0.001,
            "speeding up on its heading, at {:?}",
            speeding.at
        );
    }

    #[test]
    fn card_127s_home_is_the_middle_of_its_rice_at_rest_and_not_of_the_rice_gone_or_going() {
        let resting = |slot: usize, at: (f32, f32), frames: i32, step: (f32, f32)| Hazard {
            frames: Some(frames),
            step: Some(step),
            ..a_trails_bullet(slot, at)
        };
        let frame = Frame {
            field: FIELD,
            hazards: vec![
                resting(1, (-40.0, 200.0), 50, (0.0, 0.0)),
                resting(2, (40.0, 280.0), 100, (0.0, 0.0)),
                resting(3, (180.0, 20.0), 200, (0.0, 0.0)),
                resting(4, (180.0, 40.0), 60, (0.0, 1.0)),
            ],
            ..Frame::default()
        };
        assert_eq!(
            super::among_the_rice_at_rest(&frame, super::SPLEEN_EATERS_RICE_SETS_OFF_AT),
            Some((0.0, 240.0))
        );
        let none = Frame {
            field: FIELD,
            ..Frame::default()
        };
        assert_eq!(
            super::among_the_rice_at_rest(&none, super::SPLEEN_EATERS_RICE_SETS_OFF_AT),
            None
        );
    }

    #[test]
    fn card_127_the_partners_way_waits_against_the_nearer_side_wall_and_goes_as_a_spirit_closes_in()
    {
        let spirit = |slot: usize, at: (f32, f32)| Hazard {
            kind: Kind::Body { solid: true },
            ..a_trails_bullet(slot, at)
        };
        let on = |run: (f32, f32), hazards: Vec<Hazard>| Frame {
            field: FIELD,
            at: run,
            boss: Some((0.0, 100.0)),
            hazards,
            ..Frame::default()
        };
        let (left, right) = (FIELD.0, FIELD.0 + FIELD.2);
        assert_eq!(
            super::against_the_nearer_side_wall(&on((-20.0, 300.0), vec![])).0,
            left
        );
        assert_eq!(
            super::against_the_nearer_side_wall(&on((20.0, 300.0), vec![])).0,
            right
        );
        let at_the_wall = (left, 330.0);
        assert!(!super::the_spleen_eaters_spirits_close_in(&on(
            at_the_wall,
            vec![spirit(1, (left + 150.0, 330.0))]
        )));
        assert!(super::the_spleen_eaters_spirits_close_in(&on(
            at_the_wall,
            vec![spirit(1, (left + 40.0, 330.0))]
        )));
        assert!(
            !super::the_spleen_eaters_spirits_close_in(&on(
                (0.0, 110.0),
                vec![spirit(1, (0.0, 100.0))]
            )),
            "the boss's own body read as a spirit"
        );
    }

    #[test]
    fn card_127s_ring_is_found_round_the_spirits_left_of_it_and_not_at_their_middle() {
        let middle = (20.0, 390.0);
        let on = |degrees: f32| {
            let angle = degrees.to_radians();
            (
                middle.0 + 120.0 * angle.cos(),
                middle.1 + 120.0 * angle.sin(),
            )
        };
        let three_of_eight = [on(0.0), on(45.0), on(90.0)];
        let found = super::the_middle_of_the_ring_through(&three_of_eight);
        assert!(
            super::distance(found, middle) < 0.5,
            "round the three left, at {found:?}"
        );
        let in_a_line = [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)];
        assert_eq!(
            super::the_middle_of_the_ring_through(&in_a_line),
            (10.0, 0.0),
            "their middle, in a line"
        );
        assert_eq!(
            super::the_middle_of_the_ring_through(&[(5.0, 7.0)]),
            (5.0, 7.0),
            "the one, alone"
        );
    }

    #[test]
    fn card_127s_burst_is_played_on_where_its_spirits_gather_on_the_cards_frames_457_and_every_400_after()
     {
        let middle = (-20.0, 380.0);
        let spirit = |n: usize| {
            let angle = (45.0 * n as f32).to_radians();
            Hazard {
                kind: Kind::Body { solid: true },
                slot: 100 + n,
                frames: None,
                at: (middle.0 + 60.0 * angle.cos(), middle.1 + 60.0 * angle.sin()),
                step: None,
                half: (8.0, 8.0),
                grows: 0.0,
            }
        };
        let frame = Frame {
            field: FIELD,
            at: (0.0, 405.0),
            boss: Some((0.0, 120.0)),
            hazards: (0..8).map(spirit).collect(),
            ..Frame::default()
        };
        let burst = |now: i32| {
            super::the_spleen_eaters_burst_to_come(
                &frame,
                &super::THE_SPLEEN_EATER,
                (1.0, 1.0),
                now,
            )
        };
        let mut coming = burst(400).expect("the burst at 457 to come");
        assert!(
            super::distance(coming.at, middle) < 0.01,
            "where the spirits gather, at {:?}",
            coming.at
        );
        let run = (middle.0, middle.1);
        for _ in 0..56 {
            coming.goes_on(run);
        }
        assert_eq!(
            coming.room_from(run),
            f32::MAX,
            "nothing before the card's frame 457"
        );
        coming.goes_on(run);
        assert!(
            coming.room_from(run) < 0.0,
            "on the run at the card's frame 457"
        );
        let next = burst(700).expect("the burst at 857 to come, the one at 457 over");
        assert!(matches!(next.moves, super::Moves::BurstsIn(157, _)));
        assert!(
            burst(200).is_none(),
            "none further off than the card's frames looked ahead"
        );
        assert!(
            burst(472).is_none(),
            "none between the one at 457 over and the next out of sight"
        );
        let one = Frame {
            field: FIELD,
            at: (0.0, 405.0),
            boss: Some((0.0, 120.0)),
            hazards: vec![spirit(0)],
            ..Frame::default()
        };
        let gathered =
            super::the_spleen_eaters_burst_to_come(&one, &super::THE_SPLEEN_EATER, (1.0, 1.0), 440)
                .expect("the burst where the one spirit left is");
        assert!(
            super::distance(gathered.at, spirit(0).at) < 0.01,
            "at {:?}",
            gathered.at
        );
        let none = Frame {
            field: FIELD,
            at: (0.0, 405.0),
            boss: Some((0.0, 120.0)),
            ..Frame::default()
        };
        assert!(
            super::the_spleen_eaters_burst_to_come(
                &none,
                &super::THE_SPLEEN_EATER,
                (1.0, 1.0),
                400
            )
            .is_none(),
            "none with no spirit up"
        );
    }

    #[test]
    fn card_147_is_waited_out_on_the_floor_out_from_under_the_flare_coming_down_on_the_run() {
        let floor = 432.0;
        let flare = ((-161.0, 247.0), 83.0);
        let frame = Frame {
            field: FIELD,
            at: (-170.0, floor),
            hazards: vec![a_flare(1, flare.0, flare.1)],
            ..Frame::default()
        };
        let x = super::between_the_flares(&frame, floor).expect("a place across");
        assert!(!the_flare_comes_down_on(flare.0, flare.1, (x, floor)));
    }

    #[test]
    fn card_147_is_not_waited_out_under_a_flare_still_swelling_low_down() {
        let floor = 432.0;
        // Born low, 21 across and swelling by 5 a frame, right over the run: given up from 21 as it falls it reads as
        // 16 across at the floor, which the run stands clear of; swelled to its whole reach first, it covers the run.
        let swelling = Hazard {
            grows: 5.0,
            ..a_flare(1, (0.0, 360.0), 21.0)
        };
        let frame = Frame {
            field: FIELD,
            at: (30.0, floor),
            hazards: vec![swelling],
            ..Frame::default()
        };
        let x = super::between_the_flares(&frame, floor).expect("a place across");
        assert!(
            x.abs() > super::A_FLARE_SWELLS_TO,
            "{x} is under the flare swelling at 0,360"
        );
    }

    #[test]
    fn card_147_is_not_waited_out_beyond_a_flare_that_crosses_the_floor_on_the_way_there() {
        let floor = 432.0;
        // One flare about to cross the floor just right of the run, and the floor beyond it clear.
        let near = ((-100.0, 400.0), 30.0);
        let frame = Frame {
            field: FIELD,
            at: (-150.0, floor),
            hazards: vec![a_flare(1, near.0, near.1)],
            ..Frame::default()
        };
        let x = super::between_the_flares(&frame, floor).expect("a place across");
        assert!(x < near.0.0, "{x} is through the flare at {:?}", near.0);
        assert!(!the_flare_comes_down_on(near.0, near.1, (x, floor)));
    }

    fn a_heart(slot: usize, at: (f32, f32), step: (f32, f32)) -> Hazard {
        Hazard {
            kind: Kind::Bullet {
                hitting: true,
                state: 1,
                next_change: None,
            },
            slot,
            frames: Some(0),
            at,
            step: Some(step),
            half: (5.0, 5.0),
            grows: 0.0,
        }
    }

    fn a_trails_bullet(slot: usize, at: (f32, f32)) -> Hazard {
        Hazard {
            half: (3.0, 3.0),
            step: Some((0.0, 0.35)),
            ..a_heart(slot, at, (0.0, 0.0))
        }
    }

    #[test]
    fn card_171_is_waited_out_in_front_of_the_boss_over_or_under_a_heart_going_across() {
        let (high, low) = (288.0, 408.0);
        let boss = (40.0, 128.0);
        let on = |run: (f32, f32), hazards: Vec<Hazard>| {
            let frame = Frame {
                field: FIELD,
                at: run,
                boss: Some(boss),
                hazards,
                ..Frame::default()
            };
            super::in_front_of_the_embers(&frame, high, low)
                .map(|place| place.expect("a place open long enough"))
        };
        assert_eq!(on((0.0, low), vec![]), None, "with no heart out");
        let across = a_heart(1, (-150.0, 392.0), (2.0, 0.0));
        let (x, y) = on((40.0, 392.0), vec![across]).expect("a place in front of her");
        assert!((x - boss.0).abs() <= 32.0, "{x} is not in front of her");
        assert!(
            (y - 392.0).abs() >= super::A_HEART_IS_KEPT_FROM_BY,
            "{y} is in the way of the heart going across"
        );
        // Each case after a frame with no heart out, the height told before forgotten.
        on((0.0, low), vec![]);
        // A trail's bullet coming down onto where the run stands, the heart that laid it gone far off.
        let far = a_heart(3, (180.0, 20.0), (2.0, -2.0));
        let (_, y) = on((40.0, 360.0), vec![far, a_trails_bullet(4, (40.0, 330.0))])
            .expect("a place in front of her");
        assert!(
            y > 360.0 + super::A_TRAIL_IS_KEPT_FROM_BY,
            "{y} is where the trail comes down"
        );
        on((0.0, low), vec![]);
        let frame = Frame {
            field: FIELD,
            at: (0.0, low),
            boss: Some((180.0, 128.0)),
            hazards: vec![a_heart(2, (0.0, 40.0), (2.0, 0.0))],
            ..Frame::default()
        };
        let (x, _) = super::in_front_of_the_embers(&frame, high, low)
            .flatten()
            .expect("a place in front of her");
        assert!(
            x <= super::THE_EMBERS_MIDDLE_IS_KEPT_WITHIN + 32.0,
            "{x} is not kept off the corner with her by the wall"
        );
    }

    #[test]
    fn card_171_tells_no_place_with_every_place_in_front_of_the_boss_shut_soon() {
        let (high, low) = (288.0, 408.0);
        // Hearts all over the places in front of her, each a step from the next and barely moving.
        let hearts = (0..7)
            .flat_map(|column| (0..10).map(move |row| (column, row)))
            .map(|(column, row)| {
                a_heart(
                    column * 10 + row,
                    (-48.0 + column as f32 * 16.0, high - 8.0 + row as f32 * 16.0),
                    (0.0, 0.1),
                )
            })
            .collect();
        let frame = Frame {
            field: FIELD,
            at: (0.0, low),
            boss: Some((0.0, 128.0)),
            hazards: hearts,
            ..Frame::default()
        };
        super::in_front_of_the_embers(
            &Frame {
                field: FIELD,
                ..Frame::default()
            },
            high,
            low,
        );
        assert_eq!(super::in_front_of_the_embers(&frame, high, low), Some(None));
    }

    #[test]
    fn card_172_is_waited_out_low_while_she_is_high_and_up_while_she_is_along_the_bottom_across_only_as_she_crosses()
     {
        let (low, high) = (384.0, 200.0);
        let on = |boss: Option<(f32, f32)>, run_x: f32| {
            let frame = Frame {
                field: FIELD,
                at: (run_x, low),
                boss,
                ..Frame::default()
            };
            super::across_from_the_genes(&frame, low, high, None)
        };
        let across = super::THE_GENES_PLACE_ACROSS;
        assert_eq!(
            on(Some((-160.0, 48.0)), 100.0),
            Some((across, low)),
            "low on the run's side at first"
        );
        assert_eq!(
            on(Some((160.0, 48.0)), 100.0),
            Some((-across, low)),
            "and across from her as she crosses"
        );
        assert_eq!(
            on(Some((-160.0, 400.0)), -100.0),
            Some((-across, high)),
            "up on the run's own side as she comes along the bottom, and not across from her"
        );
        assert_eq!(
            on(Some((-120.0, 240.0)), -100.0),
            Some((-across, high)),
            "kept there on her way back up"
        );
        assert_eq!(
            on(Some((120.0, 240.0)), -100.0),
            Some((-across, high)),
            "across from her as she crosses"
        );
        assert_eq!(
            on(Some((120.0, 200.0)), 100.0),
            Some((across, low)),
            "and low on the run's side back up top"
        );
        assert_eq!(on(None, 0.0), None, "with no boss");

        // **And up in the corner across from her once she has come to four corners**, her next way being at the run —
        // in the same test as the above since the way keeps its place in one static: a card begun again.
        let at = |boss: (f32, f32), card_frames: i32| {
            let frame = Frame {
                field: FIELD,
                at: (0.0, low),
                boss: Some(boss),
                ..Frame::default()
            };
            super::across_from_the_genes(&frame, low, high, Some(card_frames))
        };
        at((0.0, 224.0), 100);
        let lured = (-across, FIELD.1 + super::THE_GENES_LURED_DOWN);
        for (frames, corner) in [
            (1, (-160.0, 400.0)),
            (2, (-160.0, 48.0)),
            (3, (160.0, 400.0)),
        ] {
            assert_ne!(
                at(corner, frames),
                Some(lured),
                "not up there before the fourth corner"
            );
        }
        assert_eq!(
            at((160.0, 48.0), 4),
            Some(lured),
            "up across from her at the fourth"
        );
        assert_eq!(
            at((-40.0, 120.0), 5),
            Some(lured),
            "and while she comes at the run"
        );
        assert_ne!(
            at((-160.0, 400.0), 6),
            Some(lured),
            "and back to the low line's ways at the fifth"
        );
    }

    #[test]
    fn the_fire_after_card_169_is_waited_out_left_of_the_middle_under_the_nearest_place_its_next_row_is_far_above()
     {
        let floor = 432.0;
        let on = |at_x: f32, hazards: Vec<Hazard>| {
            let frame = Frame {
                field: FIELD,
                at: (at_x, floor),
                hazards,
                ..Frame::default()
            };
            super::between_the_long_rows(&frame, floor)
        };
        let row = |from: f32, y: f32, first: usize| {
            (0..7).map(move |k| a_trails_bullet(first + k, (from + 8.0 * k as f32, y)))
        };
        let (left, _, width, _) = FIELD;
        let middle = left + width / 2.0;
        // A row coming down 52 above the left of the places, and one 132 above further right.
        let rows =
            || -> Vec<Hazard> { row(-120.0, 380.0, 0).chain(row(-56.0, 300.0, 10)).collect() };

        let x = on(-100.0, rows()).expect("a place across");
        assert!(x > -60.0, "{x} is not under the row 52 above");
        assert!(x <= middle, "{x} is left of the middle");
        assert_eq!(
            on(-100.0, rows()),
            on(-60.0, rows()),
            "the nearest of the clear ones either way"
        );
        assert_eq!(
            on(150.0, rows()),
            Some(middle),
            "from the right, the middle"
        );
        assert_eq!(on(-100.0, vec![]), None, "with nothing in the field");
    }

    #[test]
    fn the_stage_before_the_extras_midboss_is_waited_out_under_the_red_fairies_going_up_then_the_side_gathered_more()
     {
        let on = |stage_frames: i64, bodies: &[(f32, f32)]| {
            let frame = Frame {
                field: FIELD,
                at: (0.0, 432.0),
                hazards: bodies
                    .iter()
                    .enumerate()
                    .map(|(slot, at)| a_flare(slot, *at, 8.0))
                    .collect(),
                ..Frame::default()
            };
            super::under_the_red_fairies(&frame, stage_frames)
        };
        let (left, _, width, _) = FIELD;
        let off = super::UNDER_THE_RED_FAIRIES_OFF_THE_WALL_BY;
        assert_eq!(on(180, &[]), None, "with no body up yet");
        assert_eq!(
            on(200, &[(-156.0, 323.0), (154.0, 395.0)]),
            Some(left + width - off),
            "under the lowest going up, kept off the wall"
        );
        let gathered = [
            (-117.0, 30.0),
            (-100.0, 36.0),
            (-89.0, 37.0),
            (140.0, 31.0),
            (60.0, 300.0),
        ];
        let x = on(400, &gathered).expect("a place across");
        assert!(
            (x - (-102.0)).abs() < 1.0,
            "{x} is under the three gathered at the top left"
        );
        assert_eq!(
            on(700, &[(0.0, 80.0)]),
            Some(x),
            "where it last was once the red fairies are gone"
        );
    }

    #[test]
    fn card_163_is_waited_out_a_little_above_the_floor_and_on_it_while_rice_comes_down_on_the_run()
    {
        let (_, top, _, height) = FIELD;
        let floor = top + height;
        let above = floor - super::THE_FRUIT_IS_WAITED_FOR_ABOVE;
        let on = |hazards: Vec<Hazard>| {
            let frame = Frame {
                field: FIELD,
                at: (-30.0, above),
                hazards,
                ..Frame::default()
            };
            super::under_the_miracle_fruit(&frame)
        };
        assert_eq!(
            on(vec![]),
            (-30.0, above),
            "where it is across, a little above the floor, with nothing coming"
        );
        assert_eq!(
            on(vec![a_trails_bullet(1, (-20.0, above - 60.0))]),
            (-30.0, floor),
            "down to the floor with rice coming down on it"
        );
        assert_eq!(
            on(vec![
                a_trails_bullet(1, (60.0, above - 60.0)),
                a_trails_bullet(2, (-30.0, above - 200.0))
            ]),
            (-30.0, above),
            "rice off to the side or far above it sends it nowhere"
        );
    }

    #[test]
    fn card_15_is_waited_out_going_round_the_boss_clockwise_from_where_the_run_is_round_her() {
        let boss = (0.0, 160.0);
        let at = |run: (f32, f32), boss: Option<(f32, f32)>, stage_frames: i64| {
            let frame = Frame {
                field: FIELD,
                at: run,
                boss,
                ..Frame::default()
            };
            super::round_the_jealous_boss(&frame, stage_frames)
        };
        let start = 60_000;
        let first = at((0.0, 400.0), Some(boss), start).expect("a place round her");
        assert!(
            super::distance(first, (0.0, 256.0)) < 0.01,
            "under her where the run is round her, at {first:?}"
        );
        let half_way = (180.0 / super::ROUND_THE_JEALOUS_BOSS_BY) as i64;
        let mut place = first;
        for frame in 1..=half_way {
            place = at(place, Some(boss), start + frame).expect("a place round her");
        }
        assert!(
            super::distance(place, (0.0, 64.0)) < 0.5,
            "half way round, over her, after a half turn's frames, at {place:?}"
        );
        let near_the_top =
            at(place, Some((0.0, 120.0)), start + half_way + 1).expect("a place round her");
        assert!(
            near_the_top.1 >= FIELD.1 + 24.0 - 0.5,
            "kept under the top with her near it, at {near_the_top:?}"
        );
        let (a, b) = (
            at(place, Some(boss), start + half_way + 2).expect("a place"),
            at(place, Some(boss), start + half_way + 3).expect("a place"),
        );
        let turned = (a.0 - boss.0) * (b.1 - boss.1) - (a.1 - boss.1) * (b.0 - boss.0);
        assert!(
            turned > 0.0,
            "not gone round clockwise on the screen, down being further"
        );
        let near_the_wall = (FIELD.0 + 60.0, 200.0);
        let past_it = at((FIELD.0 + 10.0, 200.0), Some(near_the_wall), start + 1_000)
            .expect("a place round her");
        assert!(
            past_it.1 < near_the_wall.1 - 48.0,
            "gone on over her rather than between her and the wall 60 off her, at {past_it:?}"
        );
        assert_eq!(
            at(place, None, start + 1_001),
            None,
            "nowhere with no boss read"
        );
    }

    #[test]
    fn card_135_is_waited_out_under_the_boss_and_up_and_down_the_field_each_time_a_body_comes_near_where_it_waits()
     {
        let (_, top, _, height) = FIELD;
        let boss = (0.0, 224.0);
        let body = |at: (f32, f32)| Hazard {
            kind: Kind::Body { solid: true },
            slot: 300,
            frames: None,
            at,
            step: None,
            half: (8.0, 8.0),
            grows: 0.0,
        };
        let at = |run: (f32, f32), bodies: Vec<Hazard>, stage_frames: i64| {
            let frame = Frame {
                field: FIELD,
                at: run,
                boss: Some(boss),
                hazards: bodies,
                ..Frame::default()
            };
            super::up_and_down_from_the_rekindled(&frame, stage_frames)
        };
        let (low, high) = ((0.0, top + height - 48.0), (-32.0, top + 48.0));
        let start = 70_000;
        assert_eq!(at(low, vec![], start), low, "under her to start with");
        assert_eq!(
            at(low, vec![body((0.0, 340.0))], start + 1),
            high,
            "up as a body comes near"
        );
        assert_eq!(
            at(low, vec![body((0.0, 340.0))], start + 2),
            high,
            "and up while it stays near"
        );
        assert_eq!(at(high, vec![], start + 3), high, "and up once it has gone");
        assert_eq!(
            at(high, vec![body((-30.0, 120.0))], start + 4),
            low,
            "down as the next comes near"
        );
        assert_eq!(at(low, vec![], start + 5), low, "and down once it has gone");
        assert_eq!(
            at(low, vec![body((0.0, 340.0))], start + 6),
            (32.0, top + 48.0),
            "and up again as the next comes, a little right of her this time"
        );

        // **And not turned by a body that comes near the run on its way**, in the same test as the turns above since
        // the way keeps its turns in one static: a card begun afresh.
        let start = start + 10_000;
        assert_eq!(at(low, vec![], start), low);
        assert_eq!(at(low, vec![body((0.0, 340.0))], start + 1), high);
        let halfway = (-16.0, top + height / 2.0);
        assert_eq!(at(halfway, vec![], start + 2), high);
        assert_eq!(
            at(halfway, vec![body((-16.0, 300.0))], start + 3),
            high,
            "up still, the next body having come near the run on its way"
        );
        assert_eq!(
            at(high, vec![body((-30.0, 120.0))], start + 4),
            low,
            "and down as it comes near the run up there"
        );
    }

    #[test]
    fn card_131_is_waited_out_on_the_floor_aside_of_the_way_a_circle_of_spirits_comes_down_on() {
        let (_, top, _, height) = FIELD;
        let floor = top + height;
        let boss = (0.0, 128.0);
        // Five spirits 47 off a middle going down from `22,104` at 2 a frame, round once every 32 frames.
        let spirits = |frame: i64| -> Vec<Hazard> {
            let middle = (22.0, 254.0 + 2.0 * frame as f32);
            (0..5)
                .map(|n| {
                    let angle = std::f32::consts::TAU * (frame as f32 / 32.0 + n as f32 / 5.0);
                    Hazard {
                        kind: Kind::Body { solid: true },
                        slot: 4000 + n,
                        frames: None,
                        at: (middle.0 + 47.0 * angle.cos(), middle.1 + 47.0 * angle.sin()),
                        step: None,
                        half: (7.2, 7.2),
                        grows: 0.0,
                    }
                })
                .collect()
        };
        let at = |run: (f32, f32), frame: i64| {
            let frame_seen = Frame {
                field: FIELD,
                at: run,
                boss: Some(boss),
                hazards: spirits(frame),
                ..Frame::default()
            };
            super::between_the_spinning_spirits(&frame_seen, 90_000 + frame)
        };
        assert_eq!(
            at((0.0, floor), 0),
            Some((0.0, floor)),
            "where the run is before a round of the circle is seen"
        );
        for frame in 1..31 {
            at((0.0, floor), frame);
        }
        let (x, y) = at((0.0, floor), 31).unwrap();
        assert_eq!(y, floor, "on the floor");
        assert!(
            (x - (22.0 - 64.0 - 40.0)).abs() < 1.0,
            "left of the way down at 22 across, nearer the run: {x}"
        );
        let (x, _) = at((-110.0, floor), 32).unwrap();
        assert_eq!(
            x, -110.0,
            "and where the run is when it is well aside of it already"
        );
        let (x, _) = at((150.0, floor), 33).unwrap();
        assert!(
            (x - 128.0).abs() < 0.5,
            "and 40 in from the gap's edge, between the way and the wall: {x}"
        );
    }

    #[test]
    fn card_151_is_waited_out_in_front_of_the_boss_356_down_and_card_159_on_the_floor_under_the_sun()
     {
        let (left, top, width, height) = FIELD;
        let frame = Frame {
            field: FIELD,
            at: (0.0, 432.0),
            ..Frame::default()
        };
        let with_her = Frame {
            field: FIELD,
            at: (0.0, 432.0),
            boss: Some((-60.0, 120.0)),
            ..Frame::default()
        };
        assert_eq!(
            super::among_the_fixed_stars(&with_her),
            (-60.0, top + 356.0),
            "in front of her"
        );
        assert_eq!(
            super::among_the_fixed_stars(&frame),
            (left + width / 2.0, top + 356.0),
            "and in the middle with no boss read"
        );
        assert_eq!(
            super::after_the_yin_yangs(&frame),
            (left + width / 2.0, top + 300.0),
            "stage 5 after its yin-yangs a little below the middle"
        );
        assert_eq!(
            super::under_the_artificial_sun(&frame),
            (left + width / 2.0, top + height),
            "on the floor under the sun"
        );
    }

    #[test]
    fn stage_5_after_its_midboss_is_waited_out_streaming_across_the_floor_from_the_runs_side_and_never_back()
     {
        let (left, _, width, _) = FIELD;
        let at = |x: f32, stage_frames: i64| {
            let frame = Frame {
                field: FIELD,
                at: (x, 432.0),
                ..Frame::default()
            };
            super::across_the_yin_yangs(&frame, stage_frames)
        };
        let start = 80_000;
        assert_eq!(
            at(100.0, start),
            left + width - 24.0,
            "from the side the run is on"
        );
        let mut x = left + width - 24.0;
        for frame in 1..=100 {
            x = at(x, start + frame);
        }
        assert!(
            (x - (left + width - 24.0 - 48.0)).abs() < 0.01,
            "on across slowly, at {x}"
        );
        for frame in 101..=1000 {
            x = at(x, start + frame);
        }
        assert!(
            (x - (left + 24.0)).abs() < 0.01,
            "to the other side and no further, at {x}"
        );
    }

    #[test]
    fn card_167_is_waited_out_in_front_of_the_boss_a_little_below_the_middle() {
        let (_, top, _, _) = FIELD;
        let below_the_middle = top + 326.0;
        let frame = |boss: Option<(f32, f32)>| Frame {
            field: FIELD,
            at: (-50.0, 400.0),
            boss,
            ..Frame::default()
        };
        assert_eq!(
            super::in_front_of_the_id(&frame(Some((70.0, 120.0)))),
            (70.0, below_the_middle),
            "under the boss, a little below the middle"
        );
        assert_eq!(
            super::in_front_of_the_id(&frame(None)),
            (-50.0, below_the_middle),
            "where the run is across with no boss read"
        );
    }

    #[test]
    fn the_stage_after_the_extras_midboss_is_waited_out_left_then_along_to_the_right_wall_up_it_and_back_twice()
     {
        let (left, top, width, _) = FIELD;
        let (low, up) = (top + 368.0, top + 208.0);
        let beaten = 9248;
        let frame = Frame {
            field: FIELD,
            ..Frame::default()
        };
        let played: Vec<_> = (0..=2200)
            .map(|into| super::from_the_left_wall_to_the_top_right(&frame, beaten + into))
            .collect();
        let at = |into: usize| played[into];
        assert_eq!(
            at(0),
            Some((left, low)),
            "against the left wall as the midboss is beaten"
        );
        assert_eq!(at(104), Some((left, low)), "through the first set");
        let along = top + super::AFTER_THE_MIDBOSS_ALONG;
        assert_eq!(
            at(105),
            Some((left, along)),
            "off the left wall as the second set comes"
        );
        let half_way = at(252).expect("along the low line");
        assert!(
            half_way.0 > left + width * 0.45
                && half_way.0 < left + width * 0.55
                && half_way.1 == along,
            "half way along, above the floor, at {half_way:?}"
        );
        let coming_up = at(399).expect("up towards the right wall");
        assert!(
            coming_up.0 > left + width * 0.99 && (coming_up.1 - up).abs() < 1.0,
            "come up to the right wall's room by the end of the way along, at {coming_up:?}"
        );
        assert_eq!(at(400), Some((left + width, up)), "up the right wall");
        assert_eq!(at(599), Some((left + width, up)), "through the last set");
        assert_eq!(at(600), Some((left, low)), "back at the bottom left");
        assert_eq!(
            at(1500),
            Some((left, low)),
            "against the left wall for the second wave"
        );
        assert_eq!(
            at(1900),
            Some((left + width, up)),
            "up the right wall for its last set"
        );
        assert_eq!(
            at(2150),
            None,
            "the columns firing three ways left to the rules"
        );
        assert_eq!(
            super::from_the_left_wall_to_the_top_right(&frame, beaten),
            Some((left, low)),
            "the chapter taken again"
        );

        // Streams laid down the right wall from 208 to 300: the room up it is under them and no lower than the low line.
        let right = left + width;
        let streamed = Frame {
            field: FIELD,
            at: (right, 400.0),
            hazards: (0..12)
                .map(|k| a_trails_bullet(k, (right - 10.0, 208.0 + 8.0 * k as f32)))
                .collect(),
            ..Frame::default()
        };
        let again = 30000;
        let up_it: Vec<_> = (0..=450)
            .map(|into| super::from_the_left_wall_to_the_top_right(&streamed, again + into))
            .collect();
        assert_eq!(
            up_it[450],
            Some((right, low)),
            "up the right wall to where there is room"
        );
    }

    #[test]
    fn card_169_is_waited_out_in_under_the_boss_then_between_the_rays_at_a_distance_the_rice_ahead_keeps_off()
     {
        let boss: (f32, f32) = (0.0, 224.0);
        let round_her = |off: f32, degrees: f32| {
            let angle = degrees.to_radians();
            (boss.0 + off * angle.cos(), boss.1 + off * angle.sin())
        };
        // The rays' newest bullets at 100 off her, and older rice at the distances and angles given.
        let at = |run_degrees: f32, rays: &[f32], rice: &[(f32, f32)], stage_frames: i64| {
            let rays = rays
                .iter()
                .map(|degrees| (Some(2), round_her(100.0, *degrees)));
            let rice = rice
                .iter()
                .map(|(off, degrees)| (Some(150), round_her(*off, *degrees)));
            let frame = Frame {
                field: FIELD,
                at: round_her(150.0, run_degrees),
                boss: Some(boss),
                hazards: rays
                    .chain(rice)
                    .enumerate()
                    .map(|(slot, (frames, at))| Hazard {
                        frames,
                        at,
                        ..a_bullet_at_change(slot + 1, 0)
                    })
                    .collect(),
                ..Frame::default()
            };
            super::between_the_polygraphs_rays(&frame, stage_frames)
        };
        let near = |got: Option<(f32, f32)>, wanted: (f32, f32), why: &str| {
            let got = got.expect(why);
            assert!(
                (got.0 - wanted.0).hypot(got.1 - wanted.1) < 0.01,
                "{why}: {got:?} and not {wanted:?}"
            );
        };
        let off_her = |got: Option<(f32, f32)>| {
            let got = got.expect("a place");
            (got.0 - boss.0).hypot(got.1 - boss.1)
        };
        let out = super::UNDER_THE_POLYGRAPH_OUT;
        let out_at = super::THE_POLYGRAPHS_RAYS_GO_OUT_AT;
        let eight = [0.0, 45.0, 90.0, 135.0, 180.0, -135.0, -90.0, -45.0];
        for stage_frames in 0..out_at {
            assert_eq!(
                at(90.0, &eight, &[], stage_frames),
                Some((boss.0, boss.1 + super::UNDER_THE_POLYGRAPH_FIRST)),
                "in under her till the rays go out"
            );
        }
        near(
            at(80.0, &eight, &[], out_at),
            round_her(out, 67.5),
            "then out, between the rays at 45 and 90",
        );
        let twice = [eight, eight.map(|degrees| degrees + 3.0)].concat();
        near(
            at(100.0, &twice, &[], out_at + 1),
            round_her(out, 114.0),
            "between the rays at 90 and 135, each laid at two distances, the distance kept",
        );
        // The rays turn to the lesser angle, so rice at a lesser one is ahead of the run.
        let rice_ahead = [(out, 107.5)];
        let went_to = off_her(at(100.0, &eight, &rice_ahead, out_at + 2));
        assert!(
            (went_to - out).abs() >= super::A_DISTANCE_IS_CLEAR_BY,
            "away from the rice ahead at its own distance, to {went_to}"
        );
        assert_eq!(
            off_her(at(100.0, &eight, &rice_ahead, out_at + 3)),
            went_to,
            "and waited there while the rice keeps off it"
        );
        assert_eq!(
            off_her(at(100.0, &eight, &[(out, 117.5)], out_at + 4)),
            went_to,
            "with rice behind the run at its old one let be"
        );
        let shifted = eight.map(|degrees| degrees + 22.5);
        assert_eq!(
            off_her(at(5.0, &shifted, &[], 5)),
            super::UNDER_THE_POLYGRAPH_FIRST,
            "in on the card taken again"
        );
        for stage_frames in 6..5 + out_at {
            at(5.0, &shifted, &[], stage_frames);
        }
        let wall = off_her(at(5.0, &shifted, &[], 5 + out_at));
        assert!(
            (wall - (FIELD.0 + FIELD.2 - super::A_WALL_IS_KEPT_OFF_BY)).abs() < 0.01,
            "and no further out than the wall between the rays at -22.5 and 22.5, at {wall}"
        );
        let no_boss = Frame {
            field: FIELD,
            ..Frame::default()
        };
        assert_eq!(
            super::between_the_polygraphs_rays(&no_boss, 6),
            None,
            "with no boss"
        );
    }

    #[test]
    fn card_171s_hearts_are_handed_on_as_ground_laid_where_each_will_be_the_next_frames_and_their_trails_are_not()
     {
        let bullet = |at: (f32, f32), half: f32, step: (f32, f32), slot: usize| Hazard {
            kind: Kind::Bullet {
                hitting: true,
                state: 1,
                next_change: None,
            },
            slot,
            frames: None,
            at,
            step: Some(step),
            half: (half, half),
            grows: 0.0,
        };
        let frame = Frame {
            field: FIELD,
            hazards: vec![
                bullet((0.0, 200.0), 5.0, (2.0, 0.0), 40),
                bullet((50.0, 300.0), 3.0, (0.35, 0.0), 41),
            ],
            ..Frame::default()
        };
        let laid = super::the_embers_trails_to_come(&frame);
        let places: Vec<(f32, f32)> = laid.iter().map(|hazard| hazard.at).collect();
        assert_eq!(
            places,
            (1..=8)
                .map(|ahead| (2.0 * ahead as f32, 200.0))
                .collect::<Vec<_>>(),
            "the heart's way eight frames on, and nothing of the trail's bullet"
        );
        assert!(
            laid.iter()
                .all(|hazard| hazard.slot > 41 && hazard.half == (8.0, 8.0))
        );
    }

    #[test]
    fn card_169s_newest_grain_at_the_runs_distance_coming_on_it_is_handed_on_as_a_bullet_going_round_on_to_it()
     {
        let boss: (f32, f32) = (0.0, 224.0);
        let round_her = |off: f32, degrees: f32| {
            let angle = degrees.to_radians();
            (boss.0 + off * angle.cos(), boss.1 + off * angle.sin())
        };
        let grain = |slot: usize, age: i32, off: f32, degrees: f32| Hazard {
            frames: Some(age),
            at: round_her(off, degrees),
            ..a_bullet_at_change(slot, 0)
        };
        let run = round_her(139.0, 64.8);
        let frame = Frame {
            field: FIELD,
            at: run,
            boss: Some(boss),
            hazards: vec![
                grain(1, 2, 139.0, 73.7),
                grain(2, 150, 139.0, 70.0),
                grain(3, 2, 100.0, 70.0),
                grain(4, 2, 139.0, 60.0),
                grain(5, 2, 139.0, 90.0),
            ],
            ..Frame::default()
        };
        let to_come = super::the_polygraphs_grains_to_come(&frame);
        assert_eq!(
            to_come.len(),
            1,
            "only the newest grain at the run's distance within 12 ahead"
        );
        let coming = &to_come[0];
        let wanted = round_her(139.0, 73.7);
        assert!(
            (coming.at.0 - wanted.0).hypot(coming.at.1 - wanted.1) < 0.01,
            "where it was laid: {:?}",
            coming.at
        );
        assert_eq!(
            coming.frames, None,
            "with no age, so as to be none of the rays nor the rice"
        );
        assert_eq!(coming.slot, 6, "in the slot past the game's own");
        let step = coming.step.expect("a step");
        let towards = (run.0 - coming.at.0, run.1 - coming.at.1);
        assert!(
            step.0 * towards.0 + step.1 * towards.1 > 0.0,
            "going round on to the run: {step:?}"
        );
        let pace = step.0.hypot(step.1);
        assert!(
            (pace - 139.0 * super::A_RAY_TURNS_BY.to_radians()).abs() < 0.001,
            "at the rays' turn: {pace}"
        );
        assert!(
            super::the_polygraphs_grains_to_come(&Frame {
                boss: None,
                ..frame
            })
            .is_empty(),
            "and none with no boss"
        );
    }

    #[test]
    fn card_174_is_waited_out_below_the_boss_where_the_ring_comes_with_its_rose_gone_past_and_not_coming_back()
     {
        let boss: (f32, f32) = (0.0, 128.0);
        // A ring of 56 bullets `off` from her, the 50 degrees of it round `rose` left out for its rose.
        let ring = |off: f32, rose: f32| {
            (0..56)
                .map(|bullet| bullet as f32 * 360.0 / 56.0)
                .filter(|degrees| {
                    let off = (degrees - rose).rem_euclid(360.0);
                    off.min(360.0 - off) > 25.0
                })
                .enumerate()
                .map(|(slot, degrees)| Hazard {
                    at: (
                        boss.0 + off * degrees.to_radians().cos(),
                        boss.1 + off * degrees.to_radians().sin(),
                    ),
                    ..a_bullet_at_change(slot + 1, 0)
                })
                .collect::<Vec<_>>()
        };
        let at = |hazards: Vec<Hazard>| {
            let frame = Frame {
                field: (-184.0, 32.0, 368.0, 400.0),
                at: (0.0, 280.0),
                boss: Some(boss),
                hazards,
                ..Frame::default()
            };
            super::out_of_the_subterranean_roses_way(&frame)
        };
        let below_her = |degrees: f32, off: f32| {
            let angle = degrees.to_radians();
            (boss.0 + off * angle.cos(), boss.1 + off * angle.sin())
        };
        let near = |got: Option<(f32, f32)>, wanted: (f32, f32), why: &str| {
            let got = got.expect(why);
            assert!(
                (got.0 - wanted.0).hypot(got.1 - wanted.1) < 0.5,
                "{why}: {got:?} and not {wanted:?}"
            );
        };
        // Whether the rose of a ring `ring` out with its rose at `rose`, reaching `lead` round ahead of its middle and
        // the least behind it and in and out of its ring, keeps clear of `place` all the while it goes out over it — the
        // rose going round `turns` at 2.8 degrees a frame and the ring out at 1.0, and within the 6.4 the ring's bullets
        // are apart, the rose being read as the middle of the gap between two.
        let clear_reaching = |place: (f32, f32), ring: f32, rose: f32, turns: f32, lead: f32| {
            let off = (place.0 - boss.0).hypot(place.1 - boss.1);
            let angle = (place.1 - boss.1).atan2(place.0 - boss.0).to_degrees();
            let (thick, tail) = (super::A_ROSE_IS_THICK_BY, super::A_ROSE_REACHES_AT_LEAST);
            let kept = super::A_ROSE_IS_KEPT_OFF_BY - 6.4;
            (0..=((2.0 * thick) as usize)).all(|frames| {
                let frames = off - ring - thick + frames as f32;
                frames < 0.0 || {
                    let ahead = ((angle - rose - turns * super::A_ROSE_GOES_ROUND_BY * frames)
                        * turns
                        + 180.0)
                        .rem_euclid(360.0)
                        - 180.0;
                    ahead > lead + kept || ahead < -tail - kept
                }
            })
        };
        let clear = |place: (f32, f32), ring: f32, rose: f32, turns: f32| {
            clear_reaching(place, ring, rose, turns, super::A_ROSE_REACHES_AT_LEAST)
        };
        // The run 152 straight under her, the rose some 112 to 180 round as it would go out over it there, 40 to 64
        // frames on: under her where it has gone round by then.
        near(
            at(ring(100.0, 160.0)),
            below_her(90.0, 152.0),
            "where the run is, the rose going out over it long after it has gone round past",
        );
        let place = at(ring(92.0, 0.0)).expect("a place");
        assert!(
            clear(place, 92.0, 0.0, 1.0),
            "off where the rose goes round past as it goes out: {place:?}"
        );
        // A ring further in than the one before, so not taken for the same ring a frame on.
        // Its rose 57 to 79 round as the ring would come to the run, at 148 and 156 from her, coming onto it.
        let place = at(ring(84.0, -100.0)).expect("a place");
        assert!(
            clear(place, 84.0, -100.0, 1.0),
            "off where the rose would come round as the ring comes, to where it will have gone past: {place:?}"
        );
        assert!(
            (place.0 - boss.0).hypot(place.1 - (boss.1 + 152.0)) > 4.0,
            "and not where the run is: {place:?}"
        );
        // A ring whose rose goes round the other way, read off the frame before: less as the ring came 1 out, by more
        // than the 6.4 its bullets are apart so that its gap's middle is read as moved.
        at(ring(100.0, -93.0));
        let place = at(ring(101.0, -100.0)).expect("a place");
        assert!(
            clear(place, 101.0, -100.0, -1.0),
            "and so with the rose going round the other way: {place:?}"
        );
        // A rose whose bullets, 22 across, reach 70 ahead of its middle round the ring.
        let reaching = |rose: f32| {
            ring(112.0, rose)
                .into_iter()
                .chain((0..8).map(|bullet| {
                    let degrees = (rose + 70.0 * bullet as f32 / 7.0).to_radians();
                    Hazard {
                        at: (
                            boss.0 + 112.0 * degrees.cos(),
                            boss.1 + 112.0 * degrees.sin(),
                        ),
                        half: (11.2, 11.2),
                        ..a_bullet_at_change(100 + bullet, 0)
                    }
                }))
                .collect()
        };
        let place = at(reaching(-100.0)).expect("a place");
        assert!(
            clear_reaching(place, 112.0, -100.0, 1.0, 70.0),
            "with as far again to go as the rose reaches ahead of its middle: {place:?}"
        );
        let with_one_gone_past = |rose: f32| {
            ring(100.0, rose)
                .into_iter()
                .chain(ring(170.0, 0.0))
                .collect()
        };
        // A ring gone out past the run 170 from her: no further out than 170 less its rose's depth.
        let waiting = at(with_one_gone_past(-100.0)).expect("a place");
        let off = (waiting.0 - boss.0).hypot(waiting.1 - boss.1);
        assert!(
            off <= 170.0 - super::A_RING_GONE_PAST_IS_KEPT_FROM_BY + 0.5,
            "inside the ring gone past by its rose's depth: {waiting:?}"
        );
        assert!(clear(waiting, 100.0, -100.0, 1.0), "and so: {waiting:?}");
        assert_eq!(at(Vec::new()), None, "and nowhere with no ring coming");
    }

    #[test]
    fn card_173_is_waited_out_at_the_top_right_then_in_the_middle_of_the_bosss_round_then_just_off_it_away_from_her_roses()
     {
        // th11's field, whose top is 32 down, so the middle of the boss's round is at 0,244.
        let field = (-184.0, 32.0, 368.0, 400.0);
        let stage_frames = std::cell::Cell::new(0);
        let at = |boss: (f32, f32)| {
            let frame = Frame {
                field,
                boss: Some(boss),
                ..Frame::default()
            };
            stage_frames.set(stage_frames.get() + 1);
            super::away_from_the_hated(&frame, stage_frames.get()).expect("a place")
        };
        let near = |got: (f32, f32), wanted: (f32, f32), why: &str| {
            assert!(
                (got.0 - wanted.0).hypot(got.1 - wanted.1) < 0.01,
                "{why}: {got:?} and not {wanted:?}"
            );
        };
        for _ in 0..super::THE_BLUE_STAGE_LASTS {
            assert_eq!(
                at((-128.0, 360.0)),
                (112.0, 144.0),
                "the top right through the blue stage"
            );
        }
        for frames in super::THE_BLUE_STAGE_LASTS..super::THE_YELLOW_STAGE_ENDS - 2 {
            let boss = if frames % 2 == 0 {
                (-128.0, 360.0)
            } else {
                (128.0, 128.0)
            };
            assert_eq!(
                at(boss),
                (0.0, 244.0),
                "the middle of her round through the yellow stage"
            );
        }
        let reach = super::THE_HATEDS_ROSES_REACH_IN_YELLOW;
        let (x, y) = at((0.0, 160.0));
        near(
            (x, y),
            (0.0, 160.0 + reach),
            "and as far from her as her roses reach where she comes in near it",
        );
        let rose = |at: (f32, f32)| Hazard {
            at,
            half: (8.4, 8.4),
            ..a_bullet_at_change(0, 0)
        };
        let row = |at: (f32, f32)| Hazard {
            at,
            half: (3.0, 3.0),
            ..a_bullet_at_change(1, 0)
        };
        let with_roses = |boss: (f32, f32), hazards: Vec<Hazard>| {
            stage_frames.set(stage_frames.get() + 1);
            let frame = Frame {
                field,
                boss: Some(boss),
                hazards,
                ..Frame::default()
            };
            super::away_from_the_hated(&frame, stage_frames.get()).expect("a place")
        };
        near(
            with_roses(
                (128.0, 128.0),
                vec![
                    rose((-20.0, 330.0)),
                    rose((20.0, 350.0)),
                    row((150.0, 100.0)),
                ],
            ),
            (0.0, 340.0 - reach),
            "and as far from her roses as they reach where they come in near it, rather than from her",
        );
        let off = super::THE_HATED_IS_LEFT_OFF_THE_MIDDLE_BY;
        near(
            at((-128.0, 244.0)),
            (off, 244.0),
            "then right of the middle with her at the left of her round",
        );
        near(
            at((0.0, 360.0)),
            (0.0, 244.0 - off),
            "above it with her at its bottom",
        );
        near(
            at((10.0, 250.0)),
            (0.0, 244.0 - off),
            "and kept so as she crosses the middle",
        );
        near(
            at((128.0, 128.0)),
            (-off * 0.7409, 244.0 + off * 0.6716),
            "down and left of it with her at its top right",
        );
        near(
            with_roses(
                (128.0, 244.0),
                vec![
                    rose((-150.0, 300.0)),
                    rose((-110.0, 340.0)),
                    row((150.0, 100.0)),
                ],
            ),
            (off * 0.8633, 244.0 - off * 0.5047),
            "and off it away from her roses rather than from her, her rows not counted",
        );
        let no_boss = Frame {
            field,
            ..Frame::default()
        };
        assert_eq!(
            super::away_from_the_hated(&no_boss, stage_frames.get() + 1),
            None,
            "with no boss"
        );
    }
}
