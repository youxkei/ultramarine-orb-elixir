//! **Ways of dodging one card of one game, each named for its card** — a card whose pattern is the same every
//! launch, so that what gets through it is its own schedule and nothing any other card shares. The ways that
//! are a mechanism a card may want, whichever card it is, are `steer`'s.

use std::f32::consts::FRAC_PI_2;
use std::sync::Mutex;

use orb_core::game::{Frame, Kind};

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
