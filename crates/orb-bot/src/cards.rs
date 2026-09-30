//! **Ways of dodging one card of one game, each named for its card** — a card whose pattern is the same every
//! launch, so that what gets through it is its own schedule and nothing any other card shares. The ways that
//! are a mechanism a card may want, whichever card it is, are `steer`'s.

use std::f32::consts::FRAC_PI_2;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

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
        if river
            .chosen_on
            .is_none_or(|was| now - was >= A_PACE_IS_CHOSEN_EVERY)
        {
            river.chosen_on = Some(now);
            river.place = frame.at;
            river.pace = the_pace_through(frame, aside, card, &river.bodies_before);
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
}

/// **How a card's bullets at rest set off**: straight at the run, or each its own way.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SetsOff {
    AtTheRun,
    AnyWay,
}

const THE_RIVER: LookedAhead = LookedAhead {
    sets_off: SetsOff::AtTheRun,
    paces: &RIVER_PACES,
    looks_ahead: RIVER_LOOKS_AHEAD,
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
) -> (f32, f32) {
    let field = frame.field;
    let home = match card.home {
        Home::AboveTheFloor(above) => (field.0 + field.2 / 2.0, field.1 + field.3 - above),
        Home::WhereTheRiceIsYoungest => where_the_rice_is_youngest(frame),
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
            Fire::of(hazard, mine, card.sets_off, moved)
        })
        .collect();
    let paces = std::iter::once((0.0, 0.0)).chain((0..16).flat_map(|heading| {
        let angle = heading as f32 * std::f32::consts::PI / 8.0;
        card.paces
            .iter()
            .map(move |pace| (pace * angle.cos(), pace * angle.sin()))
    }));
    let mut played = Vec::with_capacity(fire.len());
    let mut best = ((0.0, 0.0), -1, f32::MIN);
    for pace in paces {
        played.clear();
        played.extend_from_slice(&fire);
        let (unhit, room) = played_on(&mut played, frame.at, pace, field, home, mine, card);
        if unhit > best.1 || (unhit == best.1 && room > best.2) {
            best = (pace, unhit, room);
        }
    }
    best.0
}

/// **How many frames a run going on at `pace` from `from` goes unhit by the card's fire**, the card's frames
/// looked ahead and one for all of them, and the room it is left: the least of it on the way, at most
/// [`ROOM_ENOUGH`], less how far from `home` it ends — or, for a run that is hit, how far into the fire.
fn played_on(
    fire: &mut Vec<Fire>,
    from: (f32, f32),
    pace: (f32, f32),
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
    ) -> Option<Fire> {
        let step = hazard.step.or(moved).unwrap_or((0.0, 0.0));
        let moves = match hazard.kind {
            Kind::Bullet { .. } => {
                let pace = step.0.hypot(step.1);
                let at_rest_until = match sets_off {
                    SetsOff::AtTheRun => RIVER_SETS_OFF_AT,
                    SetsOff::AnyWay => RED_FROG_BURSTS_AT,
                };
                let age = hazard.frames.unwrap_or(at_rest_until);
                if pace < AT_REST_UNDER && age < at_rest_until {
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
                };
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
}

impl River {
    const NONE: River = River {
        on: None,
        chosen_on: None,
        pace: (0.0, 0.0),
        place: (0.0, 0.0),
        bodies: Vec::new(),
        bodies_before: Vec::new(),
    };
}

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
}
