//! What a bot playing th11 is told of it, which is this game's own arrays read.
//!
//! **Every address here is in `th11/mod.rs` beside its own reading**, and `asm.md` is where each came
//! from. What this file is, is the shape those readings take for `orb_bot`: the field the player is held
//! inside, the player and its own box, its four speeds, one [`Hazard`] per live enemy and per live bullet,
//! one [`Beam`] per laser that can hit, and one [`Pickup`] per item.
//!
//! **The field is measured from its top middle**, as th10's is: a position is `-192` to `192` across and `0`
//! to `448` down, and the player is held inside `-184` to `184` and `32` to `432` — see [`PLAYER_BOUNDS`].
//!
//! **Two of this game's tests are circles where th10's are boxes** — a round bullet's and every enemy's body,
//! `0x431f70` — and what the rules are handed is a box, so each circle goes over as the square about it that
//! the circle test's own reach fits inside: see [`a_circles_reach`].

use std::sync::atomic::{AtomicBool, Ordering};

use orb_api::mem;

use super::{
    BULLET_AGE, BULLET_BYTES, BULLET_FLAGS, BULLET_IS_ROUND, BULLET_MAY_HIT, BULLET_POSITION,
    BULLET_SIZE, BULLET_SLOTS, BULLET_STATE, BULLET_STEP, BULLETS_AT, CARD_TIMER, ENEMY_BODY,
    ENEMY_BODY_OFF_FOR, ENEMY_FLAGS, ENEMY_NO_BODY, ENEMY_POSITION, G_BOMB_COSTS, G_BULLET_MANAGER,
    G_CARD, G_CHARACTER, G_GAME_SPEED, G_ITEM_MANAGER, G_LASER_MANAGER, G_PLAYER, G_POWER,
    G_POWER_AT_MOST, G_SHOT_TYPE, GAP_STATE_THROUGH, ITEM_BYTES, ITEM_POSITION, ITEM_SLOTS,
    ITEM_STATE, ITEM_TYPE, ITEMS_AT, LASER_ANGLE, LASER_FIRST, LASER_ID, LASER_KIND_ANCHORED,
    LASER_LENGTH, LASER_LENGTH_AT_MOST_ANCHORED, LASER_LENGTH_AT_MOST_TRAVELLING, LASER_NEXT,
    LASER_ORIGIN, LASER_PHASE, LASER_PHASE_FULL, LASER_PHASE_WARNING, LASER_PHASE_WIDENING,
    LASER_SPEED, LASER_TIMER, LASER_WARNING_FRAMES, LASER_WIDTH, LASER_WIDTH_WHOLE, LASERS_AT_MOST,
    PLAYER_BOUNDS, PLAYER_GAP_STATE, PLAYER_HALF, PLAYER_POSITION, PLAYER_SHT, PLAYER_SPEEDS,
    PLAYER_STATE, PLAYER_STATE_HIT, PLAYER_STATE_NORMAL, SHOT_TYPE_WITH_A_GAP,
    SHOT_TYPES_A_CHARACTER, SHT_WHOLE_HITBOX, SPEED_SCALE, Th11,
};
use crate::game::{Beam, Frame, Hazard, Kind, Pickup, Worth};
use crate::trace;

/// Whether the hit the player is in has been traced — see [`trace_what_stood_over_a_hit`].
static A_HIT_WAS_TRACED: AtomicBool = AtomicBool::new(false);

/// How far from the player an enemy's or a bullet's own box is traced on the frame a hit is seen: three of the
/// player's own whole steps, measured from the box's edge.
const TRACED_WITHIN: f32 = 12.0;

/// **What stood over the player on the frame a hit is seen**, once a hit and into the trace — see
/// [`trace::OverAHit`]: every enemy on the list and every bullet in any state within [`TRACED_WITHIN`] of the
/// player, with the words the game's own tests are gated on.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn trace_what_stood_over_a_hit(at: (f32, f32)) {
    if !trace::on() {
        return;
    }
    let near = |(x, y): (f32, f32), (w, h): (f32, f32)| {
        (x - at.0).abs() <= TRACED_WITHIN + w.abs() / 2.0
            && (y - at.1).abs() <= TRACED_WITHIN + h.abs() / 2.0
    };
    let mut over = trace::OverAHit {
        at,
        ..trace::OverAHit::default()
    };
    for (slot, enemy) in Th11.the_enemies().into_iter().enumerate() {
        let place = unsafe { position(enemy + ENEMY_POSITION) };
        let size = unsafe { position(enemy + ENEMY_BODY) };
        if near(place, size) {
            over.enemies.push(trace::AnEnemyOverAHit {
                slot: slot as u32,
                at: place,
                size,
                flags: mem::read_committed::<u32>(enemy + ENEMY_FLAGS).unwrap_or(0),
                familiar: 0,
                behaviour: None,
                anchor: None,
                angle: 0.0,
            });
        }
    }
    if let Some(manager) = mem::read_committed::<usize>(G_BULLET_MANAGER).filter(|it| *it != 0) {
        for slot in 0..BULLET_SLOTS {
            let bullet = manager + BULLETS_AT + slot * BULLET_BYTES;
            let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
            let place = unsafe { position(bullet + BULLET_POSITION) };
            // **A round one's size is its radius**, so it is measured from as twice that: one hit of a Normal scout on
            // card 31 had nothing traced over it at all.
            let size = match unsafe { mem::read::<u32>(bullet + BULLET_FLAGS) } & BULLET_IS_ROUND {
                0 => unsafe { position(bullet + BULLET_SIZE) },
                _ => {
                    let radius = unsafe { mem::read::<f32>(bullet + BULLET_SIZE) };
                    (radius * 2.0, radius * 2.0)
                }
            };
            if state == 0 || !near(place, size) {
                continue;
            }
            over.bullets.push(a_bullet_traced(bullet, slot, state));
        }
    }
    trace::over_a_hit(over);
}

/// One bullet as the trace keeps it.
fn a_bullet_traced(bullet: usize, slot: usize, state: u16) -> trace::ABulletOverAHit {
    trace::ABulletOverAHit {
        slot: slot as u32,
        at: unsafe { position(bullet + BULLET_POSITION) },
        size: unsafe { position(bullet + BULLET_SIZE) },
        kind: state,
        hidden: unsafe { mem::read::<u32>(bullet + BULLET_FLAGS) } as u8,
        age: Some(unsafe { mem::read::<i32>(bullet + BULLET_AGE) }),
        step: Some(unsafe { position(bullet + BULLET_STEP) }),
    }
}

/// **Every bullet on the field into the trace, on a frame of the card a launch named** — see
/// `trace::TheField`, the card's frames its own timer's count.
///
/// **And not the boss's main script's time**, which they were: that is set back by her script — through card 169 at
/// `hits-extra/s7-c11-f26723` it stood at 51 to 59 over and over, and through card 127 it went 75 to 120 and then
/// -219 upward — where the card's timer went 1 to 1,350 through card 169 at `hits-extra10/s7-c11-f27544` and 1 to
/// 1,500 through card 127 on each attempt at it (2026-10-08).
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn trace_the_field(frame: &Frame) {
    let up = Th11.the_card_that_is_up();
    if !trace::the_field_is_wanted_in(up) {
        return;
    }
    let card = up.unwrap_or(trace::NO_CARD);
    let boss = Th11.the_boss_enemy();
    let handed_over = trace::TheField::of(frame, card, 0);
    let mut field = trace::TheField {
        card,
        at: frame.at,
        card_frames: up
            .and_then(|_| mem::read_committed::<usize>(G_CARD).filter(|card| *card != 0))
            .and_then(|card| mem::read_committed::<i32>(card + CARD_TIMER))
            .unwrap_or(-1),
        bullets: Vec::new(),
        beams: handed_over.beams,
        boss: boss.map(|enemy| unsafe { position(enemy + ENEMY_POSITION) }),
        bodies: handed_over.bodies,
    };
    if let Some(manager) = mem::read_committed::<usize>(G_BULLET_MANAGER).filter(|it| *it != 0) {
        for slot in 0..BULLET_SLOTS {
            let bullet = manager + BULLETS_AT + slot * BULLET_BYTES;
            let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
            if state != 0 {
                field.bullets.push(a_bullet_traced(bullet, slot, state));
            }
        }
    }
    trace::the_field(field);
}

/// **The share of a laser's length the game tests**: both kinds hand their test an origin a tenth of the way
/// along and a length of eight tenths (`length / 10`, `length * 4 / 5` — 0x425ff0 through 0x426044, 0x4274da
/// through 0x42752e), th10's shares.
const LASER_TESTED_FROM: f32 = 0.1;
const LASER_TESTED_TO: f32 = 0.9;

/// **The length a laser has to reach before either kind is tested at all** — `length > 16` at 0x425f94 and
/// 0x42747e.
const LASER_TESTED_PAST: f32 = 16.0;

/// **How much of a bullet's own step its state moves it by**: the whole of it for an active one, and half
/// while it is coming out or going (0x408939, 0x4089be).
fn the_share_of_its_step(state: u16) -> f32 {
    match state {
        BULLET_ACTIVE => 1.0,
        _ => 0.5,
    }
}

/// **The updates until a bullet coming out is tested**, off its own animation: a bullet leaves state 2 on the update
/// that finds its animation's `$I0` set (`[bullet+0x3bc]`, 0x408a3e through 0x408a50), and is tested on that one, and
/// what sets it is the `iset $I0` its coming-out script runs (opcode 6, 0x44cd88) on the update its time comes up —
/// the animation running after the test (0x408ce5), its timer one on after it (0x44e1d9). So it is the updates to
/// that instruction's time from the timer's, and two more; one where `$I0` is already set; and nothing where the
/// script ends or jumps first.
///
/// **Read off the script and not one age for every bullet**, which th10's is: the time is `bullet.anm`'s, chosen per
/// bullet by the maker's coming-out label (0x4095bf through 0x409637), and the field traced turned every bullet of
/// cards 31, 171, 172 and 174 active at age 26 and every one of stage 5's cards 131 and 135 at 16 (2026-10-10).
fn the_updates_until_it_is_tested(bullet: usize) -> Option<i32> {
    if mem::read_committed::<i32>(bullet + BULLET_ANIMATION_DONE)? != 0 {
        return Some(1);
    }
    let timer = mem::read_committed::<i32>(bullet + BULLET_ANIMATION_TIMER)?;
    let mut at =
        mem::read_committed::<usize>(bullet + BULLET_ANIMATION_AT).filter(|it| *it != 0)?;
    for _ in 0..AN_ANIMATION_IS_READ_FOR_AT_MOST {
        let opcode = mem::read_committed::<u16>(at)?;
        let size = mem::read_committed::<u16>(at + 2)?;
        if matches!(opcode, ANIMATION_ENDS | ANIMATION_DELETES | ANIMATION_JUMPS) || size == 0 {
            return None;
        }
        if opcode == ANIMATION_SETS
            && mem::read_committed::<u16>(at + 6)? & 1 != 0
            && mem::read_committed::<i32>(at + 8)? == ANIMATION_I0
        {
            let time = i32::from(mem::read_committed::<i16>(at + 4)?);
            return Some((time - timer + 2).max(1));
        }
        at += usize::from(size);
    }
    None
}
/// The bullet's animation, `0x434` bytes from its `+0x8`: its timer's whole frames, where it is in its script, and its
/// `$I0` (the runner 0x44b4b0; the variables 0x44b080).
const BULLET_ANIMATION_TIMER: usize = 0x68;
const BULLET_ANIMATION_AT: usize = 0x3b0;
const BULLET_ANIMATION_DONE: usize = 0x3bc;
/// An animation's instructions: the number, the size and the time are a word, a word and a signed word (0x44b513), the
/// mask saying which arguments are variables a word at `+0x6`, and the arguments from `+0x8` — end `-1`, delete 1,
/// jump 4, set 6 (the table at 0x44e228); `$I0` is variable 10000.
const ANIMATION_ENDS: u16 = 0xffff;
const ANIMATION_DELETES: u16 = 1;
const ANIMATION_JUMPS: u16 = 4;
const ANIMATION_SETS: u16 = 6;
const ANIMATION_I0: i32 = 10000;
/// How many instructions a read goes through at most: a coming-out script is a handful, and a word read wrong is a
/// walk that never ends otherwise.
const AN_ANIMATION_IS_READ_FOR_AT_MOST: usize = 64;
/// **The age [`Frame::a_bullet_comes_out_for`] says a bullet coming out is tested at**, against which each one's own
/// age is handed over as this less [`the_updates_until_it_is_tested`] — so the rules count its own updates down.
const A_BULLET_IS_TESTED_AT: i32 = 26;

/// A bullet that can take a life — see [`BULLET_STATE`], tested in state 1 alone (0x408a57).
const BULLET_ACTIVE: u16 = 1;
/// One coming out, which moves at half its step and is tested the frame it turns active.
const BULLET_SPAWNING: u16 = 2;

/// **The half-width a laser's test is handed, out of the width it is drawn at** — th10's formulas, read again
/// here: `w / 2` below 32, and from 32 on `w - (w + 16) / 2` for the travelling kind (0x425ff0 through 0x426044)
/// and `w - (w + 16) / 3` for the anchored one (0x4274da through 0x42752e), which the test halves.
fn half_across(width: f32, anchored: bool) -> f32 {
    let handed = if width < 32.0 {
        width / 2.0
    } else if anchored {
        width - (width + 16.0) / 3.0
    } else {
        (width - 16.0) / 2.0
    };
    handed / 2.0
}

/// **The half of the square a circle test of radius `radius` reaches through**, against a player whose own half
/// is `half` and whose `.sht` names a whole hitbox of `whole`: the test hits where `d² < whole² + radius²`
/// (0x431f79 through 0x431fbf), so the centres can be `sqrt(whole² + radius²)` apart — and the rules test a box
/// against the player's own half, which is taken back off.
///
/// **The square and not the circle inside it**, the rules having boxes alone: a box this size reaches past the
/// circle at its corners, which is a little ground given up and never a hit missed.
fn a_circles_reach(radius: f32, whole: f32, half: f32) -> f32 {
    ((whole * whole + radius * radius).sqrt() - half).max(0.0)
}

/// **How far a circle reaches for each box it goes over as**: a circle of reach `R` is `R / 8` boxes, so that what the
/// boxes give up past the circle at their corners stays near a run's own step.
const A_BOX_FOR_EACH: f32 = 8.0;

/// **The boxes a circle of `reach` goes over as**, about its centre: one square for a small one, and for a large one
/// `n` whose union holds the circle — the quarter cut into `n` turns of `90° / n`, and box `k` as wide as the circle is
/// at the bottom of its turn and as tall as it is at the top, `(R·cos θ(k-1), R·sin θ(k))`, so that each holds the
/// circle's rows inside its turn.
///
/// **Because a square is wrong in the one place it matters, and three boxes were not enough.** th11's
/// `核熱「ニュークリアフュージョン」` sends bodies of radius 88 down the field, and the square about one took the floor's
/// whole width from a run it pressed down onto: six hits of a Normal scout of 2026-10-07 at the floor with
/// `nowhere open to it`, each some 87 from the body's centre — the circle's own edge. Three boxes, cut at 30° and
/// 60°, still reach `1.22 R` at their corners, some 19 past a sun and 10 past a flare of `爆符「メガフレア」` that the
/// rules read the floor as shut under; eleven reach `1.07 R`.
fn the_boxes_of_a_circle(reach: f32) -> Vec<(f32, f32)> {
    let boxes = ((reach / A_BOX_FOR_EACH).ceil() as usize).clamp(1, BOXES_A_THING);
    if boxes == 1 {
        return vec![(reach, reach)];
    }
    let turn = std::f32::consts::FRAC_PI_2 / boxes as f32;
    (1..=boxes)
        .map(|k| {
            (
                reach * (turn * (k - 1) as f32).cos(),
                reach * (turn * k as f32).sin(),
            )
        })
        .collect()
}

/// **How many slots one thing of the game's takes among the hazards**, a box of [`the_boxes_of_a_circle`] apiece and
/// that many at most, so that each box is a thing of its own the rules can follow — and for an enemy, [`A_RING_OF`]
/// more past those for the ring its hit callback may test in place of its body.
const BOXES_A_THING: usize = 12;
const BOXES_AN_ENEMY: usize = BOXES_A_THING + A_RING_OF;

/// **The hit callback that tests a ring in place of an enemy's body**, `0x418050`, which the stage's script puts at
/// the enemy's `+0x2674` out of the table at `0x4a3da0` (0x414c0a, 0x414c11) and the enemy's update calls in place of
/// the body's circle (0x411e19): 32 rectangles about the enemy (`mov iVar4,0x20`), each `12` long and
/// `life * 16 / life at the start + 4` wide, laid along an arc of the enemy's sprite (0x41807c through 0x418150).
///
/// **What `怪輪「地獄の苦輪」` hits with**, card 31 of stage 3: two Normal scouts of 2026-10-07 were hit at `0,368` with
/// no bullet on the field and nothing within twelve of the player, the rules reading `51.2` and `43.3` of room.
const RING_CALLBACK: usize = 0x0041_8050;
const A_RING_OF: usize = 32;
const ENEMY_HIT_CALLBACK: usize = 0x2674;
/// **The sprite the ring is laid along**, by its number at the enemy's `+0x111c` (the callback's `[esi+0xe0]`, its `esi`
/// being the enemy's `+0x103c`), looked up the way `0x4561e0` looks one up: two lists from the sprite manager's
/// `+0x7b562c` and `+0x7b5634`, each element the sprite and the next, the sprite's own number its first word.
const ENEMY_SPRITE: usize = 0x111c;
const SPRITE_LISTS: [usize; 2] = [0x007b_562c, 0x007b_5634];
/// The sprite's arc and its middle, and the ring's radius, three floats the callback reads (`+0x24`, `+0x2c`, `+0x40`).
const SPRITE_ARC: usize = 0x24;
const SPRITE_TURNED: usize = 0x2c;
const SPRITE_RADIUS: usize = 0x40;
/// And the enemy's life at the start, beside its life (0x4108f0).
const ENEMY_LIFE_AT_THE_START: usize = 0x2530;

/// **One frame of a run, out of the game's own arrays**, and `false` where no player is up.
///
/// # Safety
/// Must run on the game's main thread, between frames.
pub unsafe fn what_a_bot_sees(into: &mut Frame) -> bool {
    let Some(player) = mem::read_committed::<usize>(G_PLAYER).filter(|it| *it != 0) else {
        return false;
    };
    into.field = PLAYER_BOUNDS;
    into.at = unsafe { position(player + PLAYER_POSITION) };
    into.half = unsafe { position(player + PLAYER_HALF) };
    let speed = |at: usize| unsafe { mem::read::<i32>(player + PLAYER_SPEEDS + at * 4) } as f32 / SPEED_SCALE;
    let (straight, focused_straight, diagonal, focused_diagonal) =
        (speed(0), speed(1), speed(2), speed(3));
    into.straight = (straight, straight);
    into.diagonal = (diagonal, diagonal);
    into.focused_straight = (focused_straight, focused_straight);
    into.focused_diagonal = (focused_diagonal, focused_diagonal);
    let state = unsafe { mem::read::<i32>(player + PLAYER_STATE) };
    into.hit = state == PLAYER_STATE_HIT;
    if into.hit != A_HIT_WAS_TRACED.swap(into.hit, Ordering::Relaxed) && into.hit {
        unsafe { trace_what_stood_over_a_hit(into.at) };
    }
    into.normal = state == PLAYER_STATE_NORMAL;
    into.border = false;
    // **A bullet coming out takes no life, and turns active on a frame of its own**, as th10's: state 2 moves
    // it at half its step and tests nothing (0x408939 through 0x4089be), the test being state 1's alone.
    into.a_bullet_not_hitting_is_harmless = true;
    // **One age for every bullet, each one's own handed over against it** — see [`A_BULLET_IS_TESTED_AT`]: left
    // `None`, a bullet coming out was ground to cross for good, and the Extra's card 172 was hit at `-88.9,334.9` by
    // one coming out 4.2 off the run for 20 frames, turned active at age 26 with the run waiting beside it, twice on
    // `hits/s7-c14-f42507` and twice more on another kept chapter (2026-10-10).
    into.a_bullet_comes_out_for = Some(A_BULLET_IS_TESTED_AT);
    into.shoots_a_boss_unfocused = false;
    into.at_the_gauges_human_end = None;
    into.familiars_are_worth_sparing = false;
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    // **The bombs a run has are its power over what one costs**, both the shot type's `.sht`'s (0x4311ac through
    // 0x4311f9), and full power the most it can hold (0x420a09 through 0x420a29).
    let power = unsafe { mem::read::<i32>(G_POWER) };
    let costs = unsafe { mem::read::<i32>(G_BOMB_COSTS) };
    let at_most = unsafe { mem::read::<i32>(G_POWER_AT_MOST) };
    into.full_power = at_most > 0 && power >= at_most;
    into.bombs = match costs {
        1.. => (power / costs).clamp(0, i32::from(i8::MAX)) as i8,
        _ => 0,
    };
    into.dialogue = Th11.a_conversation_is_on();
    let shot_type = unsafe {
        mem::read::<i32>(G_CHARACTER) * SHOT_TYPES_A_CHARACTER + mem::read::<i32>(G_SHOT_TYPE)
    };
    into.warps_across = shot_type == SHOT_TYPE_WITH_A_GAP;
    into.warping = unsafe { mem::read::<i32>(player + PLAYER_GAP_STATE) } >= GAP_STATE_THROUGH;
    into.boss = Th11
        .the_boss_enemy()
        .map(|enemy| unsafe { position(enemy + ENEMY_POSITION) });
    let whole = mem::read_committed::<usize>(player + PLAYER_SHT)
        .filter(|it| *it != 0)
        .and_then(|sht| mem::read_committed::<f32>(sht + SHT_WHOLE_HITBOX))
        .unwrap_or(into.half.0 * 2.0);
    let game_speed = unsafe { mem::read::<f32>(G_GAME_SPEED) };
    unsafe { enemies(into, whole) };
    unsafe { bullets(into, game_speed, whole) };
    unsafe { lasers(into, game_speed) };
    unsafe { items(into) };
    unsafe { trace_the_field(into) };
    true
}

/// **Every enemy on the list, with the body the player is tested against**: solid where neither of the two
/// bits that skip the test is set and the frames it is skipped for have run out (0x411df9 through 0x411e0e),
/// tested as a circle of half its body's size (0x411e2c through 0x411e48).
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn enemies(into: &mut Frame, whole: f32) {
    let half = into.half.0;
    // Taken out and the lock let go of at once: [`Sizes`] puts this frame's back as it drops, under the same lock.
    let before = std::mem::take(
        &mut *SIZES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    );
    let mut sizes = Sizes {
        before,
        now: Vec::new(),
    };
    let enemies = Th11.the_enemies();
    let numbers: Vec<Option<u32>> = enemies
        .iter()
        .map(|enemy| unsafe { the_ring_sprites_number(*enemy) })
        .collect();
    let wanted: Vec<u32> = numbers.iter().flatten().copied().collect();
    let found = unsafe { the_sprites_numbered(&wanted) };
    let sprite_of = |number: u32| {
        wanted
            .iter()
            .zip(&found)
            .find_map(|(wanted, sprite)| (*wanted == number).then_some(*sprite)?)
    };
    for ((enemy, (slot, held_for)), number) in enemies
        .iter()
        .copied()
        .zip(places_kept(&enemies))
        .zip(numbers)
    {
        let flags = mem::read_committed::<u32>(enemy + ENEMY_FLAGS).unwrap_or(0);
        let off_for = mem::read_committed::<i32>(enemy + ENEMY_BODY_OFF_FOR).unwrap_or(0);
        let size = mem::read_committed::<f32>(enemy + ENEMY_BODY).unwrap_or(0.0);
        let reach = a_circles_reach(size.abs() / 2.0, whole, half);
        // **How many reads the enemy has held its slot for**, which is one more each frame while it is the same enemy and
        // nought for the next one given the slot — what tells the enemy in a place from the one before it, see
        // [`places_kept`].
        let frames = Some(held_for);
        let at = unsafe { position(enemy + ENEMY_POSITION) };
        let solid = flags & ENEMY_NO_BODY == 0 && off_for <= 0;
        let grows = it_grows_by(&mut sizes, enemy, size.abs());
        let mut push = |part: usize, at: (f32, f32), half: (f32, f32)| {
            into.hazards.push(Hazard {
                kind: Kind::Body { solid },
                slot: slot * BOXES_AN_ENEMY + part,
                frames,
                at,
                step: None,
                half,
                grows,
            });
        };
        let ring = number
            .and_then(sprite_of)
            .and_then(|sprite| unsafe { the_ring_of(enemy, sprite) });
        match ring {
            Some(ring) => {
                for (part, (place, half)) in ring.into_iter().enumerate() {
                    push(
                        BOXES_A_THING + part,
                        (at.0 + place.0, at.1 + place.1),
                        (half, half),
                    );
                }
            }
            None => {
                for (part, half) in the_boxes_of_a_circle(reach).into_iter().enumerate() {
                    push(part, at, half);
                }
            }
        }
    }
}

/// **The slot each enemy is handed over in, and how many reads it has held it for**: the slot it had on the read before
/// while it is still on the list, and otherwise the first slot no enemy on the list now holds, its count started at
/// nought.
///
/// **And not its place on the list**, which is th10's and an identity there only because its enemies join at the tail:
/// on th11 the place moved under nearly every enemy every frame — read with `--log=verbose` on a hit of card 143 from
/// `hits8/s6-c5-f9287` (2026-10-07), `80 enem(ies) … 77 in a slot another one had last frame`, so that nearly every
/// enemy was read as standing still, the suns of radius 88 coming down on the run among them. **Nor the enemy's main
/// script's time for the count**, which the scripts set back as they loop, and a count that does not go one on a frame
/// is read as another enemy.
fn places_kept(enemies: &[usize]) -> Vec<(usize, i32)> {
    let mut held = PLACES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut now: Vec<Option<(usize, i32)>> = vec![None; held.len()];
    let mut places = vec![(0, 0); enemies.len()];
    let mut new = Vec::new();
    for (at, enemy) in enemies.iter().enumerate() {
        match held
            .iter()
            .position(|it| it.is_some_and(|(was, _)| was == *enemy))
        {
            Some(slot) => {
                let count = held[slot].map_or(0, |(_, count)| count.wrapping_add(1));
                now[slot] = Some((*enemy, count));
                places[at] = (slot, count);
            }
            None => new.push(at),
        }
    }
    for at in new {
        let slot = match now.iter().position(Option::is_none) {
            Some(slot) => slot,
            None => {
                now.push(None);
                now.len() - 1
            }
        };
        now[slot] = Some((enemies[at], 0));
        places[at] = (slot, 0);
    }
    *held = now;
    places
}
/// What [`places_kept`] read on the read before.
static PLACES: std::sync::Mutex<Vec<Option<(usize, i32)>>> = std::sync::Mutex::new(Vec::new());

/// **Each enemy's body's size as it was read the frame before**, by the enemy — see [`it_grows_by`].
static SIZES: std::sync::Mutex<Vec<(usize, f32)>> = std::sync::Mutex::new(Vec::new());

/// The sizes read the frame before, and the ones read this frame, which are the next frame's before.
struct Sizes {
    before: Vec<(usize, f32)>,
    now: Vec<(usize, f32)>,
}

impl Drop for Sizes {
    fn drop(&mut self) {
        *SIZES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = std::mem::take(&mut self.now);
    }
}

/// **How much an enemy's reach changed since the frame before**, half of what its size did — its body being tested
/// as a circle of half its size — and nought for one not read then.
///
/// **Because th11's bodies change size as they go**: `爆符「メガフレア」`'s flares swell to a reach of 88 and give up
/// some 0.26 of it for each unit they fall, landing some 20 to 30 across, and carried down at the reach they had
/// higher up they read the floor as shut — five to seven hits on the floor of each stage 6 practice of 2026-10-07.
///
/// **And never so fast that the rules' look ahead takes it under the least a body is seen at**: a flare stops shrinking
/// at a size of 33.3, and carried on down at the pace it had, the rules read it gone forty frames on — hit with `5.1`,
/// `2.8` and `3.1 of room` by flares of that size on the floor of the practice after it went in (2026-10-07).
fn it_grows_by(sizes: &mut Sizes, enemy: usize, size: f32) -> f32 {
    sizes.now.push((enemy, size));
    let changed = sizes
        .before
        .iter()
        .find(|(was, _)| *was == enemy)
        .map_or(0.0, |(_, was)| (size - was) / 2.0);
    changed.max(((A_BODY_IS_SEEN_AT_LEAST - size) / 2.0 / THE_RULES_LOOK_AHEAD_BY).min(0.0))
}

/// **The least size a body shrinks to**, `爆符「メガフレア」`'s flares on the floor.
const A_BODY_IS_SEEN_AT_LEAST: f32 = 33.3;
/// **How many frames the rules look ahead**, which is what a shrink is spread over so as not to pass the least.
const THE_RULES_LOOK_AHEAD_BY: f32 = 40.0;

/// **The number of the sprite an enemy's ring is laid along**, where its hit callback is [`RING_CALLBACK`].
///
/// # Safety
/// Must run on the game's main thread, with `enemy` one of the manager's list.
unsafe fn the_ring_sprites_number(enemy: usize) -> Option<u32> {
    if mem::read_committed::<usize>(enemy + ENEMY_HIT_CALLBACK)? != RING_CALLBACK {
        return None;
    }
    mem::read_committed::<u32>(enemy + ENEMY_SPRITE)
}

/// **The sprite numbered each of `numbers`**, looked up the way `0x4561e0` looks one up, the first of the two lists
/// before the second.
///
/// **In one walk for every ring up, and not a walk each**: a read of the lists asks Windows whether each word is
/// there to be read, and card 31's seven rings walked a ring at a time took the frame's sight from some 0.1 ms to
/// 23 ms at `--pace=16` (stage 3 practised on its own, 2026-10-10), slower than the game's own rate.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn the_sprites_numbered(numbers: &[u32]) -> Vec<Option<usize>> {
    let mut found = vec![None; numbers.len()];
    if numbers.is_empty() {
        return found;
    }
    let Some(manager) = mem::read_committed::<usize>(super::G_SPRITES).filter(|it| *it != 0) else {
        return found;
    };
    let mut spans = Readable::default();
    for list in SPRITE_LISTS {
        let mut element = spans.read::<usize>(manager + list).unwrap_or(0);
        for _ in 0..A_LIST_RUNS_AT_MOST {
            if element == 0 || found.iter().all(Option::is_some) {
                break;
            }
            let Some([sprite, next]) = spans.read::<[usize; 2]>(element) else {
                break;
            };
            if let Some(number) = (sprite != 0).then(|| spans.read::<u32>(sprite)).flatten() {
                for (wanted, at) in numbers.iter().zip(found.iter_mut()) {
                    if at.is_none() && *wanted == number {
                        *at = Some(sprite);
                    }
                }
            }
            element = next;
        }
    }
    found
}

/// **Reads by [`mem::read_committed`]'s rule, asking the host once a region**: the sprites live in a few regions of
/// the heap, and a walk of them asking about each word took card 31's sight 2.5 ms a frame where one walk was left.
/// Held for one walk between frames only, nothing then freeing the pages it holds.
#[derive(Default)]
struct Readable(Vec<std::ops::Range<usize>>);

impl Readable {
    fn read<T: Copy>(&mut self, address: usize) -> Option<T> {
        if !address.is_multiple_of(align_of::<T>()) {
            return None;
        }
        let end = address.checked_add(size_of::<T>())?;
        let held = |span: &std::ops::Range<usize>| span.start <= address && end <= span.end;
        if !self.0.iter().any(held) {
            let span = mem::readable_at(address)?;
            if !held(&span) {
                return mem::read_committed(address);
            }
            self.0.push(span);
        }
        Some(unsafe { mem::read(address) })
    }
}

/// **The ring an enemy's hit callback tests in place of its body**, laid along `sprite`: each of its 32 rectangles'
/// centres about the enemy, and the half of the square any turn of the rectangle fits inside.
///
/// # Safety
/// Must run on the game's main thread, with `enemy` one of the manager's list.
unsafe fn the_ring_of(enemy: usize, sprite: usize) -> Option<Vec<((f32, f32), f32)>> {
    let arc = mem::read_committed::<f32>(sprite + SPRITE_ARC)?;
    let turned = mem::read_committed::<f32>(sprite + SPRITE_TURNED)?;
    let radius = mem::read_committed::<f32>(sprite + SPRITE_RADIUS)?;
    let life = mem::read_committed::<i32>(enemy + super::ENEMY_LIFE)? as f32;
    let at_the_start = mem::read_committed::<i32>(enemy + ENEMY_LIFE_AT_THE_START)?.max(1) as f32;
    let wide = life * 16.0 / at_the_start + 4.0;
    // **The square any turn of a `12` by `wide` rectangle fits inside**, its half the rectangle's half-diagonal: what the
    // game tests is the rectangle turned to the ring's angle (0x432520), and the rules have boxes only.
    let half = (wide.max(12.0) / 2.0) * std::f32::consts::SQRT_2;
    let first = arc / 64.0 + (turned - arc / 2.0);
    Some(
        (0..A_RING_OF)
            .map(|at| {
                let angle = first + arc / 32.0 * at as f32;
                ((angle.cos() * radius, angle.sin() * radius), half)
            })
            .collect(),
    )
}

/// **How many elements a walk of a sprite list goes through at most**: a next pointer read wrong is a walk that never
/// ends otherwise.
const A_LIST_RUNS_AT_MOST: usize = 0x2000;

/// **Every item still falling**, with what picking it up does — the ones flying to the player on their own
/// are left out, nothing about where the player goes changing where they end.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn items(into: &mut Frame) {
    let Some(manager) = mem::read_committed::<usize>(G_ITEM_MANAGER).filter(|it| *it != 0) else {
        return;
    };
    for slot in 0..ITEM_SLOTS {
        let item = manager + ITEMS_AT + slot * ITEM_BYTES;
        let state = unsafe { mem::read::<i32>(item + ITEM_STATE) };
        if state != ITEM_FALLING && state != ITEM_THROWN_UP {
            continue;
        }
        into.items.push(Pickup {
            worth: worth(unsafe { mem::read::<i32>(item + ITEM_TYPE) }),
            at: unsafe { position(item + ITEM_POSITION) },
        });
    }
}

/// An item falling, and one thrown up and falling (the item manager's state switch, 0x42361a on).
const ITEM_FALLING: i32 = 1;
const ITEM_THROWN_UP: i32 = 2;

/// **Which of the shared kinds one of this game's items is**, out of what collecting its type does: 1 and 10 a
/// power, 4 and 11 a big one, 2 a point, 7 a life, and 3, 5, 6, 8 and 9 a score, a piece of a life, full power and
/// the small ones, which the shared vocabulary has no name for.
fn worth(kind: i32) -> Worth {
    match kind {
        1 | 10 => Worth::Power,
        4 | 11 => Worth::BigPower,
        2 => Worth::Point,
        7 => Worth::Life,
        _ => Worth::Other,
    }
}

/// **Every bullet that can hit or is about to**, off the manager's array — a round one at the square its circle
/// test reaches through, see [`a_circles_reach`].
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn bullets(into: &mut Frame, game_speed: f32, whole: f32) {
    let Some(manager) = mem::read_committed::<usize>(G_BULLET_MANAGER).filter(|it| *it != 0) else {
        return;
    };
    let half = into.half.0;
    for slot in 0..BULLET_SLOTS {
        let bullet = manager + BULLETS_AT + slot * BULLET_BYTES;
        let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
        if state != BULLET_ACTIVE && state != BULLET_SPAWNING {
            continue;
        }
        let flags = unsafe { mem::read::<u32>(bullet + BULLET_FLAGS) };
        if flags & BULLET_MAY_HIT == 0 {
            continue;
        }
        let (vx, vy) = unsafe { position(bullet + BULLET_STEP) };
        let share = the_share_of_its_step(state) * game_speed;
        let (width, height) = unsafe { position(bullet + BULLET_SIZE) };
        let boxes = match flags & BULLET_IS_ROUND {
            0 => vec![((width / 2.0).abs(), (height / 2.0).abs())],
            _ => the_boxes_of_a_circle(a_circles_reach(width.abs(), whole, half)),
        };
        let age = unsafe { mem::read::<i32>(bullet + BULLET_AGE) };
        // **And its own age where its script says nothing of when**, which is the age most of this game's bullets come
        // out at — see [`A_BULLET_IS_TESTED_AT`].
        let frames = Some(match state {
            BULLET_SPAWNING => the_updates_until_it_is_tested(bullet)
                .map_or(age, |left| A_BULLET_IS_TESTED_AT - left),
            _ => age,
        });
        let at = unsafe { position(bullet + BULLET_POSITION) };
        for (part, half) in boxes.into_iter().enumerate() {
            into.hazards.push(Hazard {
                kind: Kind::Bullet {
                    hitting: state == BULLET_ACTIVE,
                    state,
                    next_change: None,
                },
                slot: slot * BOXES_A_THING + part,
                frames,
                at,
                step: Some((vx * share, vy * share)),
                half,
                grows: 0.0,
            });
        }
    }
}

/// **Every laser the game will test**, off the manager's list — the warning line of an anchored one handed
/// over at the width it will fire at, with the frames until it does.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn lasers(into: &mut Frame, game_speed: f32) {
    let Some(manager) = mem::read_committed::<usize>(G_LASER_MANAGER).filter(|it| *it != 0) else {
        return;
    };
    let mut laser = mem::read_committed::<usize>(manager + LASER_FIRST).unwrap_or(0);
    for _ in 0..LASERS_AT_MOST {
        if laser == 0 {
            break;
        }
        let next = mem::read_committed::<usize>(laser + LASER_NEXT).unwrap_or(0);
        if let Some(beam) = unsafe { a_beam(laser, game_speed) } {
            into.beams.push(beam);
        }
        laser = next;
    }
}

/// One laser as a beam, or `None` for one the game will not test again.
///
/// # Safety
/// Must run on the game's main thread, with `laser` one of the manager's list.
unsafe fn a_beam(laser: usize, game_speed: f32) -> Option<Beam> {
    let anchored = mem::read_committed::<usize>(laser)? == LASER_KIND_ANCHORED;
    let word = |at: usize| unsafe { mem::read::<i32>(laser + at) };
    let float = |at: usize| unsafe { mem::read::<f32>(laser + at) };
    let phase = word(LASER_PHASE);
    let frames = word(LASER_TIMER);
    let length = float(LASER_LENGTH);
    let (fires_in, width, grows) = if anchored {
        let whole = float(LASER_WIDTH_WHOLE);
        let at_most = float(LASER_LENGTH_AT_MOST_ANCHORED);
        let growing = match length < at_most {
            true => float(LASER_SPEED) * game_speed,
            false => 0.0,
        };
        match phase {
            LASER_PHASE_WARNING => (word(LASER_WARNING_FRAMES) - frames, whole, growing),
            LASER_PHASE_WIDENING | LASER_PHASE_FULL => (0, whole, growing),
            _ => return None,
        }
    } else {
        let width = float(LASER_WIDTH);
        let at_most = float(LASER_LENGTH_AT_MOST_TRAVELLING);
        let growing = match length < at_most {
            true => float(LASER_SPEED) * game_speed,
            false => 0.0,
        };
        (0, width, growing)
    };
    let fires_in = match length > LASER_TESTED_PAST {
        true => fires_in,
        false => fires_in.max(1),
    };
    Some(Beam {
        slot: word(LASER_ID) as u32 as usize,
        origin: unsafe { position(laser + LASER_ORIGIN) },
        angle: float(LASER_ANGLE),
        frames,
        near: length * LASER_TESTED_FROM,
        far: length * LASER_TESTED_TO,
        grows: grows * LASER_TESTED_TO,
        half_across: half_across(width, anchored),
        fires_in,
        fading: false,
    })
}

/// Two floats read as a place.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn position(at: usize) -> (f32, f32) {
    (unsafe { mem::read::<f32>(at) }, unsafe {
        mem::read::<f32>(at + size_of::<f32>())
    })
}

#[cfg(test)]
mod tests {
    use super::{a_circles_reach, half_across};
    use orb_api::Kind;
    use orb_sim::Sim;

    /// **Each sprite asked for is found, the first list's before the second's, with the host asked once a region and
    /// not once a word**: two lists of 200 elements apiece out of one region of the heap, the sprites in another.
    #[test]
    fn the_sprites_asked_for_are_found_in_one_walk_asking_once_a_region() {
        const DATA: usize = 0x004c_3000;
        const MANAGER: usize = 0x0100_0000;
        const ELEMENTS: usize = 0x0200_0000;
        const SPRITES: usize = 0x0300_0000;
        const EACH: usize = 200;
        let sim = std::sync::Arc::new(Sim::new());
        let space = sim.space();
        space.map(DATA, 0x1000, Kind::Private);
        space.map(MANAGER, 0x0080_0000, Kind::Private);
        space.map(ELEMENTS, 0x10000, Kind::Private);
        space.map(SPRITES, 0x10000, Kind::Private);
        let _installed = sim.enter();
        space.write::<usize>(super::super::G_SPRITES, MANAGER);
        for (list, head) in super::SPRITE_LISTS.into_iter().enumerate() {
            for at in 0..EACH {
                let element = ELEMENTS + (list * EACH + at) * 8;
                let sprite = SPRITES + (list * EACH + at) * 0x40;
                let next = if at + 1 < EACH { element + 8 } else { 0 };
                space.write::<[usize; 2]>(element, [sprite, next]);
                // The number 7 on one sprite of each list, which is the first list's to answer.
                let number = if at == EACH - 1 {
                    7
                } else {
                    (list * EACH + at) as u32 + 100
                };
                space.write::<u32>(sprite, number);
            }
            space.write::<usize>(MANAGER + head, ELEMENTS + list * EACH * 8);
        }
        let asked = space.asked_about_an_address();
        let found = unsafe { super::the_sprites_numbered(&[7, 100 + EACH as u32 + 3]) };
        assert_eq!(
            found,
            vec![
                Some(SPRITES + (EACH - 1) * 0x40),
                Some(SPRITES + (EACH + 3) * 0x40)
            ]
        );
        assert!(space.asked_about_an_address() - asked < 10);
    }

    /// **A bullet coming out is tested two updates past its script's `iset $I0` to the timer, and on the next where
    /// `$I0` is set**; a script that ends before setting it says nothing.
    #[test]
    fn a_bullet_coming_out_is_tested_by_the_time_its_script_sets_i0_at() {
        const BULLET: usize = 0x0100_0000;
        const SCRIPT: usize = 0x0200_0000;
        let sim = std::sync::Arc::new(Sim::new());
        let space = sim.space();
        space.map(BULLET, 0x1000, Kind::Private);
        space.map(SCRIPT, 0x1000, Kind::Private);
        let _installed = sim.enter();
        // An instruction of 0x10 bytes: number, size, time, mask, two arguments.
        let instruction = |at: usize, opcode: u16, time: i16, mask: u16, first: i32| {
            space.write::<u16>(at, opcode);
            space.write::<u16>(at + 2, 0x10);
            space.write::<i16>(at + 4, time);
            space.write::<u16>(at + 6, mask);
            space.write::<i32>(at + 8, first);
            space.write::<i32>(at + 12, 1);
        };
        instruction(SCRIPT, 3, 0, 0, 0);
        instruction(SCRIPT + 0x10, 6, 26, 1, 10001);
        instruction(SCRIPT + 0x20, 6, 26, 1, 10000);
        instruction(SCRIPT + 0x30, 0xffff, 26, 0, 0);
        space.write::<usize>(BULLET + super::BULLET_ANIMATION_AT, SCRIPT);
        space.write::<i32>(BULLET + super::BULLET_ANIMATION_TIMER, 20);
        assert_eq!(super::the_updates_until_it_is_tested(BULLET), Some(8));
        space.write::<i32>(BULLET + super::BULLET_ANIMATION_DONE, 1);
        assert_eq!(super::the_updates_until_it_is_tested(BULLET), Some(1));

        space.write::<i32>(BULLET + super::BULLET_ANIMATION_DONE, 0);
        instruction(SCRIPT + 0x20, 0xffff, 26, 0, 0);
        assert_eq!(super::the_updates_until_it_is_tested(BULLET), None);
    }

    /// **An enemy keeps its slot while it stays on the list, its count one on each read**, whatever its place on the
    /// list; one taken off leaves its slot to the next one put on, whose count starts again at nought.
    #[test]
    fn an_enemy_keeps_its_slot_and_counts_on_while_it_stays_on_the_list() {
        *super::PLACES.lock().unwrap() = Vec::new();
        assert_eq!(
            super::places_kept(&[0x100, 0x200, 0x300]),
            vec![(0, 0), (1, 0), (2, 0)]
        );
        // The list put the other way round, and one more enemy at its head.
        assert_eq!(
            super::places_kept(&[0x400, 0x300, 0x200, 0x100]),
            vec![(3, 0), (2, 1), (1, 1), (0, 1)]
        );
        // `0x200` taken off and `0x500` put on: it is given the slot `0x200` had, and counts from nought.
        assert_eq!(
            super::places_kept(&[0x500, 0x100, 0x300, 0x400]),
            vec![(1, 0), (0, 2), (2, 2), (3, 1)]
        );
    }

    /// **A laser 32 wide is handed over by the formula and not halved**, as th10's.
    #[test]
    fn a_laser_32_wide_is_handed_over_by_its_kinds_formula() {
        assert_eq!(half_across(32.0, false), (32.0 - 16.0) / 2.0 / 2.0);
        assert_eq!(half_across(32.0, true), (32.0 - (32.0 + 16.0) / 3.0) / 2.0);
        assert_eq!(half_across(31.0, false), 31.0 / 2.0 / 2.0);
    }

    /// **A circle reaches as far as its test lets the centres come**, less the player's own half: Reimu's whole
    /// hitbox of 2 against a bullet of radius 4 lets them come `sqrt(20)` apart.
    /// **A large circle goes over as the boxes that hold it**, every point of its edge inside one of them, and a
    /// small one as one square.
    #[test]
    fn a_large_circle_is_held_by_its_boxes() {
        assert_eq!(super::the_boxes_of_a_circle(4.0), vec![(4.0, 4.0)]);
        let boxes = super::the_boxes_of_a_circle(88.0);
        assert_eq!(boxes.len(), 11);
        // And none of them reaches much past the circle at its corner.
        assert!(
            boxes
                .iter()
                .all(|(across, down)| across.hypot(*down) < 88.0 * 1.08)
        );
        for step in 0..360 {
            let (x, y) = (
                (step as f32).to_radians().cos() * 87.9,
                (step as f32).to_radians().sin() * 87.9,
            );
            assert!(
                boxes
                    .iter()
                    .any(|(across, down)| x.abs() <= *across && y.abs() <= *down),
                "{x},{y} is in none of {boxes:?}"
            );
        }
    }

    #[test]
    fn a_circle_reaches_by_its_test() {
        assert!((a_circles_reach(4.0, 2.0, 1.0) - (20f32.sqrt() - 1.0)).abs() < 1e-6);
        assert_eq!(a_circles_reach(0.0, 1.0, 1.0), 0.0);
    }
}
