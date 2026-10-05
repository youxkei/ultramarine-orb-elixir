//! What a bot playing th10 is told of it, which is this game's own arrays read.
//!
//! **Every address here is in `th10/mod.rs` beside its own reading**, and `asm.md` is where each came
//! from. What this file is, is the shape those readings take for `orb_bot`: the field the player is held
//! inside, the player and its own box, its four speeds, one [`Hazard`] per live enemy and per live bullet,
//! one [`Beam`] per laser that can hit, and one [`Pickup`] per item.
//!
//! **The field is measured from its top middle**, which is this game's own: a position is `-192` to `192`
//! across and `0` to `448` down, and the player is held inside `-184` to `184` and `32` to `432` — see
//! [`PLAYER_BOUNDS`].

use std::sync::atomic::{AtomicBool, Ordering};

use orb_api::mem;

use super::{
    A_BULLET_COMES_OUT_FOR, BULLET_AGE, BULLET_BYTES, BULLET_FLAGS, BULLET_MAY_HIT,
    BULLET_NEXT_CHANGE, BULLET_POSITION, BULLET_SIZE, BULLET_SLOTS, BULLET_STATE, BULLET_STEP,
    BULLETS_AT, ELEMENT_NEXT, ENEMIES_AT_MOST, ENEMY_ATTACK_FRAMES, ENEMY_BODY, ENEMY_BODY_OFF_FOR,
    ENEMY_FLAGS, ENEMY_LIST, ENEMY_NO_BODY, ENEMY_POSITION, G_BULLET_MANAGER, G_ENEMY_MANAGER,
    G_GAME_SPEED, G_ITEM_MANAGER, G_LASER_MANAGER, G_PLAYER, G_POWER, ITEM_BYTES, ITEM_POSITION,
    ITEM_SLOTS, ITEM_STATE, ITEM_TYPE, ITEMS_AT, LASER_ANGLE, LASER_FIRST, LASER_ID,
    LASER_KIND_ANCHORED, LASER_LENGTH, LASER_LENGTH_AT_MOST_ANCHORED,
    LASER_LENGTH_AT_MOST_TRAVELLING, LASER_NEXT, LASER_ORIGIN, LASER_PHASE, LASER_PHASE_FULL,
    LASER_PHASE_WARNING, LASER_PHASE_WIDENING, LASER_SPEED, LASER_TIMER, LASER_WARNING_FRAMES,
    LASER_WIDTH, LASER_WIDTH_WHOLE, LASERS_AT_MOST, PLAYER_BOUNDS, PLAYER_HALF, PLAYER_POSITION,
    PLAYER_SPEEDS, PLAYER_STATE, PLAYER_STATE_HIT, PLAYER_STATE_NORMAL, POWER_A_BOMB_COSTS,
    POWER_FULL, SPEED_SCALE, Th10,
};
use crate::game::{Beam, Frame, Hazard, Kind, Pickup, Worth};
use crate::trace;

/// Whether the hit the player is in has been traced — see [`trace_what_stood_over_a_hit`].
static A_HIT_WAS_TRACED: AtomicBool = AtomicBool::new(false);

/// How far from the player an enemy's or a bullet's own box is traced on the frame a hit is seen: three of the
/// player's own whole steps, measured from the box's edge.
const TRACED_WITHIN: f32 = 12.0;

/// **How far up an enemy's address is shifted before it is added into its attack's count** — see [`enemies`]: two
/// enemies then count `0x400` times their distance apart, and an enemy is an object past `+0x2470` (its
/// `ENEMY_BODY_OFF_FOR`) — some 9.5 million counts, where an attack's own timer, one more a frame, would take some
/// 44 hours of one attack to cover them.
const ENEMY_APART_BY: u32 = 10;

/// **What stood over the player on the frame a hit is seen**, once a hit and into the trace — see
/// [`trace::OverAHit`]: every enemy on the list and every bullet in any state within [`TRACED_WITHIN`] of the
/// player, with the words the game's own tests are gated on — an enemy's flags, and a bullet's state and its
/// flags where the trace's `hidden` is.
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
    if let Some(manager) = mem::read_committed::<usize>(G_ENEMY_MANAGER).filter(|it| *it != 0) {
        let mut element = mem::read_committed::<usize>(manager + ENEMY_LIST).unwrap_or(0);
        for slot in 0..ENEMIES_AT_MOST {
            if element == 0 {
                break;
            }
            let next = mem::read_committed::<usize>(element + ELEMENT_NEXT).unwrap_or(0);
            if let Some(enemy) = mem::read_committed::<usize>(element).filter(|it| *it != 0) {
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
            element = next;
        }
    }
    if let Some(manager) = mem::read_committed::<usize>(G_BULLET_MANAGER).filter(|it| *it != 0) {
        for slot in 0..BULLET_SLOTS {
            let bullet = manager + BULLETS_AT + slot * BULLET_BYTES;
            let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
            let place = unsafe { position(bullet + BULLET_POSITION) };
            let size = unsafe { position(bullet + BULLET_SIZE) };
            if state == 0 || !near(place, size) {
                continue;
            }
            over.bullets.push(trace::ABulletOverAHit {
                slot: slot as u32,
                at: place,
                size,
                kind: state,
                hidden: unsafe { mem::read::<u32>(bullet + BULLET_FLAGS) } as u8,
                age: Some(unsafe { mem::read::<i32>(bullet + BULLET_AGE) }),
                step: Some(unsafe { position(bullet + BULLET_STEP) }),
            });
        }
    }
    trace::over_a_hit(over);
}

/// **Every bullet on the field into the trace, on a frame of the card a launch named** — see
/// `trace::TheField` — with its state and its flags, the boss, and how far into the card the boss's own
/// attack clock is. Or on every frame no card is up on, where `none` was named: stage 4's midboss's first
/// attack, which carried a Normal run into a corner launch after launch, is no card.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn trace_the_field(frame: &Frame) {
    let up = Th10.the_card_that_is_up();
    if !trace::the_field_is_wanted_in(up) {
        return;
    }
    let card = up.unwrap_or(trace::NO_CARD);
    let boss = Th10.the_boss_enemy();
    let handed_over = trace::TheField::of(frame, card, 0);
    let mut field = trace::TheField {
        card,
        at: frame.at,
        card_frames: boss
            .and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_ATTACK_FRAMES))
            .unwrap_or(-1),
        bullets: Vec::new(),
        // **The lasers as the bot is handed them**, which the field had none of: th10's `開宴「二拝二拍一拝」`
        // sweeps fans of travelling lasers between its throws, and a launch was lost to them with nothing of
        // them in its trace.
        beams: handed_over.beams,
        boss: boss.map(|enemy| unsafe { position(enemy + ENEMY_POSITION) }),
        // **And the bodies as the bot is handed them**: th10's `土着神「宝永四年の赤蛙」` sends Suwako's
        // doubles at the run in straight lines, and a whole Extra was hit on it at `-133.9,225.9` with
        // nothing of them in its trace.
        bodies: handed_over.bodies,
    };
    if let Some(manager) = mem::read_committed::<usize>(G_BULLET_MANAGER).filter(|it| *it != 0) {
        for slot in 0..BULLET_SLOTS {
            let bullet = manager + BULLETS_AT + slot * BULLET_BYTES;
            let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
            if state == 0 {
                continue;
            }
            field.bullets.push(trace::ABulletOverAHit {
                slot: slot as u32,
                at: unsafe { position(bullet + BULLET_POSITION) },
                size: unsafe { position(bullet + BULLET_SIZE) },
                kind: state,
                hidden: unsafe { mem::read::<u32>(bullet + BULLET_FLAGS) } as u8,
                age: Some(unsafe { mem::read::<i32>(bullet + BULLET_AGE) }),
                step: Some(unsafe { position(bullet + BULLET_STEP) }),
            });
        }
    }
    trace::the_field(field);
}

/// **The share of a laser's length the game tests**, from its tenth to its ninth: both kinds hand their test
/// an origin a tenth of the way along and a length of eight tenths (0x41d635 through 0x41d6ab, 0x41e8ac
/// through 0x41e925), so the ends a player can stand on are ends the drawing reaches.
const LASER_TESTED_FROM: f32 = 0.1;
const LASER_TESTED_TO: f32 = 0.9;

/// **The length a laser has to reach before either kind is tested at all** — `length > 16` at 0x41d60d and
/// 0x41e886.
const LASER_TESTED_PAST: f32 = 16.0;

/// **How much of a bullet's own step its state moves it by**: the whole of it for an active one, and half
/// while it is coming out or going (0x4062f0 through 0x40635e, 0x40627d through 0x4062eb).
fn the_share_of_its_step(state: u16) -> f32 {
    match state {
        BULLET_ACTIVE => 1.0,
        _ => 0.5,
    }
}

/// A bullet that can take a life — see [`BULLET_STATE`].
const BULLET_ACTIVE: u16 = 1;
/// One coming out, which moves at half its step and is tested the frame it turns active.
const BULLET_SPAWNING: u16 = 2;

/// **The half-width a laser's test is handed, out of the width it is drawn at** — `kind` being which of the
/// two, see [`LASER_KIND_ANCHORED`].
///
/// The test builds its box as `± width / 2` across (0x4268d3 through 0x426922), and what it is handed is
/// not the drawn width: a width below 32 goes over halved, and from 32 on the travelling kind hands
/// `(width - 16) / 2` (0x41d670 through 0x41d697) and the anchored kind `width - (width + 16) / 3`
/// (0x41e8e7 through 0x41e90e) — `fcomp` against 32 and `jp` past the halving where it is not below, so 32
/// itself takes the second answer.
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
    // **A bullet coming out takes no life, and turns active on a frame of its own** — see [`BULLET_STATE`]:
    // state 2's branch moves it at half its step and tests nothing, and the frame it turns 1 runs the active
    // branch, test and all. So such a bullet is ground a move may cross and not ground to wait in, which is
    // what this answer asks the rules for.
    into.a_bullet_not_hitting_is_harmless = true;
    into.a_bullet_comes_out_for = Some(A_BULLET_COMES_OUT_FOR);
    into.shoots_a_boss_unfocused = false;
    into.at_the_gauges_human_end = None;
    into.familiars_are_worth_sparing = false;
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    let power = unsafe { mem::read::<u16>(G_POWER) };
    into.full_power = power >= POWER_FULL;
    into.bombs = (power / POWER_A_BOMB_COSTS) as i8;
    into.dialogue = Th10.a_conversation_is_on();
    into.boss = Th10
        .the_boss_enemy()
        .map(|enemy| unsafe { position(enemy + ENEMY_POSITION) });
    let game_speed = unsafe { mem::read::<f32>(G_GAME_SPEED) };
    unsafe { enemies(into) };
    unsafe { bullets(into, game_speed) };
    unsafe { lasers(into, game_speed) };
    unsafe { items(into) };
    unsafe { trace_the_field(into) };
    true
}

/// **Every enemy on the list, with the body the player is tested against**: solid where neither of the two
/// bits that skip the test is set and the frames it is skipped for have run out (0x40e2bd through 0x40e2ce).
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn enemies(into: &mut Frame) {
    let Some(manager) = mem::read_committed::<usize>(G_ENEMY_MANAGER).filter(|it| *it != 0) else {
        return;
    };
    let mut element = mem::read_committed::<usize>(manager + ENEMY_LIST).unwrap_or(0);
    for slot in 0..ENEMIES_AT_MOST {
        if element == 0 {
            break;
        }
        let next = mem::read_committed::<usize>(element + ELEMENT_NEXT).unwrap_or(0);
        if let Some(enemy) = mem::read_committed::<usize>(element).filter(|it| *it != 0) {
            let flags = mem::read_committed::<u32>(enemy + ENEMY_FLAGS).unwrap_or(0);
            let off_for = mem::read_committed::<i32>(enemy + ENEMY_BODY_OFF_FOR).unwrap_or(0);
            let (width, height) = unsafe { position(enemy + ENEMY_BODY) };
            into.hazards.push(Hazard {
                kind: Kind::Body {
                    solid: flags & ENEMY_NO_BODY == 0 && off_for <= 0,
                },
                // **Its place on the list**, which is an identity for as long as nothing ahead of it is taken
                // off — enemies join at the tail — **and the timer its attack is counted on, which is what tells
                // the enemy in a place from the one before it**: stepped once a frame by the enemy's update
                // (0x40e57e) and started over only at a change of attack (0x41284c). Without it an enemy taken
                // off ahead moved every one behind it into its neighbour's place, the difference was read as the
                // enemy's motion, and the fear spread over it covered the field — the Extra's stretch before
                // Suwako read `an enemy at -218.0,224.0 stepping 3.0,0.0, feared out to 456.0,200.0` over a hit
                // with every one of the nine moves at `-100 of room`.
                //
                // **And the enemy's own address added in, at a step no timer reaches in a run**: a wave's enemies
                // are laid on one frame and count their attacks alike, so the one behind an enemy shot down took
                // its place with the count it would have had — read as the same enemy a frame on, the step between
                // their places its motion. The same stretch read `an enemy at -143.0,224.0 stepping 3.0,0.0,
                // feared out to 306.0,200.0` over a hit with every move at some `-70 of room` once the timer was in.
                // **Added and not set beside it**, the count being one thing's only while it is one more each
                // frame — which an enemy's own count, shifted by its own address, still is.
                slot,
                frames: mem::read_committed::<i32>(enemy + ENEMY_ATTACK_FRAMES)
                    .map(|timer| timer.wrapping_add((enemy as i32).wrapping_shl(ENEMY_APART_BY))),
                at: unsafe { position(enemy + ENEMY_POSITION) },
                step: None,
                half: ((width / 2.0).abs(), (height / 2.0).abs()),
            });
        }
        element = next;
    }
}

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
        if state != ITEM_FALLING && state != ITEM_FALLING_BELOW_THE_LINE {
            continue;
        }
        into.items.push(Pickup {
            worth: worth(unsafe { mem::read::<i32>(item + ITEM_TYPE) }),
            at: unsafe { position(item + ITEM_POSITION) },
        });
    }
}

/// An item falling, and one falling that the line across the top does not take (0x41b010 through
/// 0x41b2a4).
const ITEM_FALLING: i32 = 1;
const ITEM_FALLING_BELOW_THE_LINE: i32 = 2;

/// **Which of the shared kinds one of this game's items is**, out of what collecting its type does (the
/// table at 0x41b8a0): 1 and 10 a power, 4 and 11 a big one, 2 and 5 a point, 7 a life, and 3, 8 and 9 faith,
/// which the shared vocabulary has no name for.
fn worth(kind: i32) -> Worth {
    match kind {
        1 | 10 => Worth::Power,
        4 | 11 => Worth::BigPower,
        2 | 5 => Worth::Point,
        7 => Worth::Life,
        _ => Worth::Other,
    }
}

/// **Every bullet that can hit or is about to**, off the manager's array.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn bullets(into: &mut Frame, game_speed: f32) {
    let Some(manager) = mem::read_committed::<usize>(G_BULLET_MANAGER).filter(|it| *it != 0) else {
        return;
    };
    for slot in 0..BULLET_SLOTS {
        let bullet = manager + BULLETS_AT + slot * BULLET_BYTES;
        let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
        // **Left out where it can never hit again**: a free slot, and one cancelled, which goes on drawing
        // until its animation ends and is tested by nothing.
        if state != BULLET_ACTIVE && state != BULLET_SPAWNING {
            continue;
        }
        // **And one whose own flag has the test off** — see [`BULLET_MAY_HIT`], which the spawn sets and
        // the active branch asks before it tests.
        if unsafe { mem::read::<u32>(bullet + BULLET_FLAGS) } & BULLET_MAY_HIT == 0 {
            continue;
        }
        let (vx, vy) = unsafe { position(bullet + BULLET_STEP) };
        let share = the_share_of_its_step(state) * game_speed;
        let (width, height) = unsafe { position(bullet + BULLET_SIZE) };
        // **Read as the state above is, and not through `mem::read_committed`**: every word here is in the slot the
        // state was read out of, in the manager's own array, so a word that could not be read would have been the
        // state first — and a `VirtualQuery` apiece, two a bullet, was most of the rules' steering, `6,849` of its
        // `9,153` microseconds a frame with 500 bullets and more up on card 101 (c101g, 2026-10-03).
        into.hazards.push(Hazard {
            kind: Kind::Bullet {
                hitting: state == BULLET_ACTIVE,
                state,
                next_change: Some(unsafe { mem::read::<u32>(bullet + BULLET_NEXT_CHANGE) }),
            },
            slot,
            frames: Some(unsafe { mem::read::<i32>(bullet + BULLET_AGE) }),
            at: unsafe { position(bullet + BULLET_POSITION) },
            step: Some((vx * share, vy * share)),
            half: ((width / 2.0).abs(), (height / 2.0).abs()),
        });
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
    // Bounded by the manager's own capacity: a list read wrong is a frame that never ends otherwise.
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
    // **The laser's first word through `mem::read_committed`, and the rest of it read as that word was found
    // readable**: they are one object of the game's, which its own list led to, so a word of it past the first
    // that could not be read is an object that could not be either — and at a `VirtualQuery` a word, some ten a
    // laser, card 101's hundred and forty lasers cost the steering more than its bullets did.
    let anchored = mem::read_committed::<usize>(laser)? == LASER_KIND_ANCHORED;
    let word = |at: usize| unsafe { mem::read::<i32>(laser + at) };
    let float = |at: usize| unsafe { mem::read::<f32>(laser + at) };
    let phase = word(LASER_PHASE);
    let frames = word(LASER_TIMER);
    let length = float(LASER_LENGTH);
    let (fires_in, width, grows) = if anchored {
        // **The phases, of which two are tested** — see [`LASER_PHASE`]: the warning line is drawn two wide
        // and hits nothing, so it goes over at the width it fires at and with the frames it waits out; the
        // shrinking one is left out, nothing testing it.
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
        // **The travelling kind has no warning**: it is tested once it is longer than sixteen and wider than
        // three (0x41d60d through 0x41d62f), and grows at its own speed until it reaches its length.
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
        // **The laser's own number and not its place in the list**, which is what says the beam of this slot
        // next frame is this one — see `Beam::frames`. The list is re-linked as lasers come and go, so a place
        // in it is another laser's the frame one ahead of it is freed.
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
    use super::half_across;

    /// **A laser 32 wide is handed over by the formula and not halved**, the game's branch being taken where
    /// the width is not below 32 — and one just under it halved, before the test halves either again.
    #[test]
    fn a_laser_32_wide_is_handed_over_by_its_kinds_formula() {
        assert_eq!(half_across(32.0, false), (32.0 - 16.0) / 2.0 / 2.0);
        assert_eq!(half_across(32.0, true), (32.0 - (32.0 + 16.0) / 3.0) / 2.0);
        assert_eq!(half_across(31.0, false), 31.0 / 2.0 / 2.0);
    }
}
