//! **What th06nc answers a bot with**: one frame of a stage read off its own statics.
//!
//! Its own file and this game's own, for the reason th06c's is: where its player is, what its own state
//! byte says has just happened and which of seven numbers an item is are questions about *this* game.
//!
//! **And none of what to do about any of it is here.** What is read goes across as [`Frame`] and the
//! rules are `orb-bot`'s.
//!
//! **What a hit is here is a circle and not a box.** The player's own test at `0x14006c090` takes a
//! hazard's position and size and, for a bullet, asks whether the distance between the two centres is
//! under the bullet's smaller half plus the player's own radius at `[player+0x774c]`; for an enemy it
//! asks whether the nearest point of the enemy's box is within that radius. Both go across as boxes:
//! the player's half is the radius on each axis and a bullet's is its smaller half, each a box that
//! holds the circle the game tests — so a move the model calls clear is clear of what the game tests,
//! and a move it calls shut may be clear by the corner of a box, which is the safer of the two errors.
//!
//! **Every address it reads goes through [`at`]**, this exe being built with ASLR on.

use orb_api::mem;

use crate::game::{Beam, Frame, Hazard, Kind, Pickup, Worth};

use super::{
    BOMBS, BULLET_MANAGER, ENEMY_MANAGER, INPUT_THE_RUN_ACTS_ON, ITEM_MANAGER, LASERS, LIVES,
    PLAYER, PLAYER_AREA, POWER, POWER_FULL, at, bullet_manager, enemy, enemy_manager, in_a_stage,
    item, laser, player, the_line_of_dialogue,
};

/// **One frame of a run, out of the game's own arrays**, and `false` off a stage.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn what_a_bot_sees(into: &mut Frame) -> bool {
    if !unsafe { in_a_stage() } {
        return false;
    }
    into.field = unsafe { field() };
    into.at = unsafe { position(at(PLAYER) + player::POSITION) };
    into.half = unsafe { player_half() };
    (into.straight, into.diagonal) = unsafe { a_step(false) };
    (into.focused_straight, into.focused_diagonal) = unsafe { a_step(true) };
    let state = unsafe { mem::read::<u8>(at(PLAYER) + player::STATE) };
    into.hit = state == player::HIT;
    into.normal = state == player::NORMAL;
    into.border = false;
    into.full_power = unsafe { mem::read::<u16>(at(POWER)) } >= POWER_FULL;
    into.bombs = unsafe { mem::read::<i8>(at(BOMBS)) };
    // **And the line of a conversation, which this stood at `false` while where the game kept it was
    // unread.** It is read now — see [`the_line_of_dialogue`] — and what stood on the reading is the
    // shot's own cadence: `orb_bot::steer` lets the shot go every
    // `SHOT_LET_GO_EVERY_IN_A_CONVERSATION` frames while a line is up and every `SHOT_LET_GO_EVERY`
    // otherwise, and the game wants one edge of `0x301` a line. **Not the skip key, which this game
    // gives orb none of**: its own skip is `0x8` of the word, built out of a binding filled per
    // keyboard layout, so `Keys::skip` is nought here and the shot is the whole of it.
    into.dialogue = unsafe { the_line_of_dialogue() } >= 0;
    into.boss = unsafe { the_boss_the_fights_own_slot() }
        .map(|body| unsafe { position(body + enemy::POSITION) });
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    for slot in 0..bullet_manager::SLOTS {
        let bullet = at(BULLET_MANAGER) + bullet_manager::BULLETS + slot * bullet_manager::BYTES;
        // The walk's own gate at `0x1400110a0`: a slot whose state is nought holds nothing at all.
        let state = unsafe { mem::read::<u16>(bullet + bullet_manager::STATE) };
        if state == 0 {
            continue;
        }
        let (wx, wy) = unsafe { position(bullet + bullet_manager::SIZE) };
        let (vx, vy) = unsafe { position(bullet + bullet_manager::STEP) };
        let scale = bullet_manager::a_steps_scale(state);
        // **The smaller half on both axes**, because that is the circle the game tests a bullet as —
        // `minss` at `0x14006c122` — see the module's own note.
        let half = wx.min(wy) / 2.0;
        into.hazards.push(Hazard {
            kind: Kind::Bullet {
                hitting: state == bullet_manager::FLYING,
                state,
                next_change: None,
            },
            slot,
            frames: None,
            at: unsafe { position(bullet + bullet_manager::POSITION) },
            step: Some((vx * scale, vy * scale)),
            half: (half, half),
        });
    }
    // **And the lasers, which are a line rather than a box**: the player's own test for one at
    // `0x14006c2b0` turns the player about the laser's own origin by its angle and compares it with a
    // box that is along and across, and what that box reaches across is a quarter of the width the
    // beam carries — see [`laser::HALF_OF_ITS_WIDTH`].
    //
    // **Only a firing laser hits.** The update's warning phase never calls the test and neither does
    // its fading phase — the one call is at `0x140011fb3`, in the firing phase — so a warning goes
    // across with the frames until it fires and a fading one is handed over as fading.
    for slot in 0..laser::SLOTS {
        let beam = at(LASERS) + slot * laser::BYTES;
        if unsafe { mem::read::<u8>(beam + laser::IN_USE) } == 0 {
            continue;
        }
        let phase = unsafe { mem::read::<u8>(beam + laser::PHASE) };
        let frames = unsafe { mem::read::<i32>(beam + laser::FRAMES) };
        let half_across =
            unsafe { mem::read::<f32>(beam + laser::WIDTH) } / laser::HALF_OF_ITS_WIDTH;
        let near = unsafe { mem::read::<f32>(beam + laser::NEAR) };
        let far = unsafe { mem::read::<f32>(beam + laser::FAR) };
        into.beams.push(Beam {
            slot,
            origin: unsafe { position(beam + laser::ORIGIN) },
            angle: unsafe { mem::read::<f32>(beam + laser::ANGLE) },
            frames,
            near,
            far,
            grows: unsafe { mem::read::<f32>(beam + laser::GROWS) },
            half_across,
            fires_in: match phase < laser::FIRING {
                true => (unsafe { mem::read::<i32>(beam + laser::WARNS_FOR) }) - frames,
                false => 0,
            },
            fading: phase >= laser::FADING,
        });
    }
    // **And the enemies, whose bodies are as solid as a bullet in this game too**, with a third of
    // `+0x2c0` on each side — two thirds at `0x14003933c` and then the test's own half — and with no
    // step, there being none on it: an enemy is moved by its own script.
    for slot in 0..enemy_manager::SLOTS {
        let body = at(ENEMY_MANAGER) + enemy_manager::ENEMIES + slot * enemy_manager::BYTES;
        if unsafe { mem::read::<u8>(body + enemy::FLAGS) } & enemy::IN_USE == 0 {
            continue;
        }
        let solidity = unsafe { mem::read::<u8>(body + enemy::SOLIDITY) };
        let solid = solidity & enemy::TESTED != 0
            && solidity & enemy::SOLID_BITS == enemy::SOLID_BITS
            && unsafe { mem::read::<u8>(body + enemy::MORE_FLAGS) } & enemy::PASSES_THROUGH == 0;
        let (sx, sy) = unsafe { position(body + enemy::SIZE) };
        into.hazards.push(Hazard {
            kind: Kind::Body { solid },
            slot,
            // **Nothing, nothing read off this game saying how long the enemy in this slot has been there** —
            // see `Hazard::frames`, whose note has what a slot taken as an identity without one costs.
            frames: None,
            at: unsafe { position(body + enemy::POSITION) },
            step: None,
            half: (
                sx / enemy::A_THIRD_OF_ITS_SIZE,
                sy / enemy::A_THIRD_OF_ITS_SIZE,
            ),
        });
    }
    // **Every slot walked and not the live count taken**, the same walk `0x1400443a0` makes.
    for slot in 0..item::SLOTS {
        let pickup = at(ITEM_MANAGER) + item::ITEMS + slot * item::BYTES;
        if unsafe { mem::read::<u8>(pickup + item::ALIVE) } == 0 {
            continue;
        }
        into.items.push(Pickup {
            worth: worth(unsafe { mem::read::<i8>(pickup + item::KIND) }),
            at: unsafe { position(pickup + item::POSITION) },
        });
    }
    true
}

/// **Which of the shared kinds one of this game's items is**, as the table at `0x140045168` indexes
/// them — the same seven th06c has, in the same order.
const fn worth(kind: i8) -> Worth {
    match kind {
        item::LIFE => Worth::Life,
        item::BOMB => Worth::Bomb,
        item::BIG_POWER | item::FULL_POWER => Worth::BigPower,
        item::POWER => Worth::Power,
        item::POINT | item::CANCELLED => Worth::Point,
        _ => Worth::Other,
    }
}

/// **The slot the boss the fight is with is in**, where there is one — the enemy carrying
/// [`enemy::IS_THE_BOSS`], which `0x1400396b0` and `0x1400398f0` are the two tests of and whose life
/// they divide into the bar the panel draws.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn the_boss_the_fights_own_slot() -> Option<usize> {
    (0..enemy_manager::SLOTS)
        .map(|slot| at(ENEMY_MANAGER) + enemy_manager::ENEMIES + slot * enemy_manager::BYTES)
        .find(|body| {
            let flags = unsafe { mem::read::<u8>(*body + enemy::FLAGS) };
            let solidity = unsafe { mem::read::<u8>(*body + enemy::SOLIDITY) };
            flags & enemy::IN_USE != 0 && solidity & enemy::IS_THE_BOSS != 0
        })
}

/// **The player's half, which is its own radius on each axis** — see the module's own note.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player_half() -> (f32, f32) {
    let radius = unsafe { mem::read::<f32>(at(PLAYER) + player::RADIUS) }.abs();
    (radius, radius)
}

/// The field the player may move in, as its left, top, width and height — [`PLAYER_AREA`] where it
/// reads as a rectangle, and the 384 by 448 every comparison in this exe is made against where it does
/// not.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn field() -> (f32, f32, f32, f32) {
    let (left, top) = unsafe { position(at(PLAYER_AREA)) };
    let (width, height) = unsafe { position(at(PLAYER_AREA) + 2 * size_of::<f32>()) };
    if width > 0.0 && height > 0.0 {
        return (left, top, width, height);
    }
    (0.0, 0.0, super::FIELD_SIZE.0, super::FIELD_SIZE.1)
}

/// **What one frame of holding a direction moves the player by**: the speed for that kind of move,
/// times the player's own scale for the axis — `0x14006aaa6` and `0x14006aab6`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_step(focused: bool) -> ((f32, f32), (f32, f32)) {
    let scale = |of: usize| mem::read_committed::<f32>(at(PLAYER) + of);
    let (across, down) = (scale(player::SCALE_ACROSS), scale(player::SCALE_DOWN));
    let step = |of: usize, scale: Option<f32>| {
        mem::read_committed::<f32>(at(PLAYER) + of)
            .zip(scale)
            .map(|(speed, scale)| speed * scale)
            .filter(|step| step.is_finite() && *step > 0.0)
            .unwrap_or(A_PROBE)
    };
    let (straight, diagonal) = match focused {
        true => (
            player::SPEED_STRAIGHT_FOCUSED,
            player::SPEED_DIAGONAL_FOCUSED,
        ),
        false => (player::SPEED_STRAIGHT, player::SPEED_DIAGONAL),
    };
    (
        (step(straight, across), step(straight, down)),
        (step(diagonal, across), step(diagonal, down)),
    )
}

/// How far a candidate move is followed for one frame where the game's own step cannot be read at all.
const A_PROBE: f32 = 8.0;

/// The two floats a position is, wherever one is kept.
///
/// # Safety
/// Must run on the game's main thread, with `of` the address of a position.
unsafe fn position(of: usize) -> (f32, f32) {
    (unsafe { mem::read::<f32>(of) }, unsafe {
        mem::read::<f32>(of + size_of::<f32>())
    })
}

/// **Everything within [`NEAR_ENOUGH_TO_BE_IT`] of where the player was hit, out of the game's own
/// arrays and past every rule that leaves one out** — the dump a hit writes.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn say_what_was_near() {
    let player = unsafe { position(at(PLAYER) + player::POSITION) };
    crate::detail!(
        "autoplay: the player is in state {}, with {} frame(s) a bomb still takes it back in, {} live, \
         and a radius of {:.2}",
        unsafe { mem::read::<u8>(at(PLAYER) + player::STATE) },
        unsafe { mem::read::<i32>(at(PLAYER) + player::A_BOMB_STILL_TAKES_IT_BACK) },
        unsafe { mem::read::<i8>(at(LIVES)) },
        unsafe { mem::read::<f32>(at(PLAYER) + player::RADIUS) },
    );
    crate::detail!(
        "autoplay: what was near it is measured from {:.1},{:.1}, where the player is now",
        player.0,
        player.1,
    );
    crate::detail!(
        "autoplay: the word the run acts on is {:#05x}, and {:#05x} on the frame before it",
        unsafe { mem::read::<u32>(at(INPUT_THE_RUN_ACTS_ON)) },
        unsafe { mem::read::<u32>(at(INPUT_THE_RUN_ACTS_ON) + size_of::<u32>()) },
    );
    for slot in 0..laser::SLOTS {
        let beam = at(LASERS) + slot * laser::BYTES;
        if unsafe { mem::read::<u8>(beam + laser::IN_USE) } == 0 {
            continue;
        }
        let origin = unsafe { position(beam + laser::ORIGIN) };
        crate::detail!(
            "autoplay: laser {slot} from {:.1},{:.1} at {:.3} rad, {:.1} wide, {:.1}..{:.1} along \
             growing {:.1}, phase {} on frame {} of it, warning for {}",
            origin.0,
            origin.1,
            unsafe { mem::read::<f32>(beam + laser::ANGLE) },
            unsafe { mem::read::<f32>(beam + laser::WIDTH) },
            unsafe { mem::read::<f32>(beam + laser::NEAR) },
            unsafe { mem::read::<f32>(beam + laser::FAR) },
            unsafe { mem::read::<f32>(beam + laser::GROWS) },
            unsafe { mem::read::<u8>(beam + laser::PHASE) },
            unsafe { mem::read::<i32>(beam + laser::FRAMES) },
            unsafe { mem::read::<i32>(beam + laser::WARNS_FOR) },
        );
    }
    let read = |of: usize| unsafe { mem::read::<f32>(at(PLAYER) + of) };
    crate::detail!(
        "autoplay: its speeds are {:.2} straight and {:.2} diagonal, {:.2} and {:.2} focused, \
         scaled by {:.2},{:.2}",
        read(player::SPEED_STRAIGHT),
        read(player::SPEED_DIAGONAL),
        read(player::SPEED_STRAIGHT_FOCUSED),
        read(player::SPEED_DIAGONAL_FOCUSED),
        read(player::SCALE_ACROSS),
        read(player::SCALE_DOWN),
    );
    let near = |(x, y): (f32, f32), (wx, wy): (f32, f32)| {
        (x - player.0).abs() < NEAR_ENOUGH_TO_BE_IT + wx
            && (y - player.1).abs() < NEAR_ENOUGH_TO_BE_IT + wy
    };
    let mut nearest: Option<(usize, (f32, f32), f32, u16)> = None;
    for slot in 0..bullet_manager::SLOTS {
        let bullet = at(BULLET_MANAGER) + bullet_manager::BULLETS + slot * bullet_manager::BYTES;
        let state = unsafe { mem::read::<u16>(bullet + bullet_manager::STATE) };
        let place = unsafe { position(bullet + bullet_manager::POSITION) };
        let size = unsafe { position(bullet + bullet_manager::SIZE) };
        if state != 0 {
            let apart = (place.0 - player.0).hypot(place.1 - player.1);
            if nearest.is_none_or(|(.., was, _)| apart < was) {
                nearest = Some((slot, place, apart, state));
            }
        }
        if near(place, (size.0 / 2.0, size.1 / 2.0)) {
            let scale = bullet_manager::a_steps_scale(state);
            crate::detail!(
                "autoplay: near it, bullet {slot} state {state} at {:.1},{:.1} half {:.1},{:.1} \
                 step {:.1},{:.1}",
                place.0,
                place.1,
                size.0 / 2.0,
                size.1 / 2.0,
                unsafe { mem::read::<f32>(bullet + bullet_manager::STEP) } * scale,
                unsafe { mem::read::<f32>(bullet + bullet_manager::STEP + size_of::<f32>()) }
                    * scale,
            );
        }
    }
    match nearest {
        Some((slot, place, apart, state)) => crate::detail!(
            "autoplay: the nearest bullet is {slot}, {apart:.1} off at {:.1},{:.1} state {state}",
            place.0,
            place.1,
        ),
        None => crate::detail!("autoplay: there is no bullet on the screen at all"),
    }
    for slot in 0..enemy_manager::SLOTS {
        let body = at(ENEMY_MANAGER) + enemy_manager::ENEMIES + slot * enemy_manager::BYTES;
        if unsafe { mem::read::<u8>(body + enemy::FLAGS) } & enemy::IN_USE == 0 {
            continue;
        }
        let place = unsafe { position(body + enemy::POSITION) };
        let (sx, sy) = unsafe { position(body + enemy::SIZE) };
        let half = (
            sx / enemy::A_THIRD_OF_ITS_SIZE,
            sy / enemy::A_THIRD_OF_ITS_SIZE,
        );
        if near(place, half) {
            crate::detail!(
                "autoplay: near it, enemy {slot} at {:.1},{:.1} half {:.1},{:.1} solidity {:#04x}",
                place.0,
                place.1,
                half.0,
                half.1,
                unsafe { mem::read::<u8>(body + enemy::SOLIDITY) },
            );
        }
    }
}

/// How near a thing has to be to the player for a hit to name it, whatever rule the model judged it by.
const NEAR_ENOUGH_TO_BE_IT: f32 = 16.0;
