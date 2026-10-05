//! **What th06 answers a bot with**: its own names for its buttons, and one frame of a stage read off its
//! own arrays.
//!
//! Its own file and this game's own, for the reason th07's is — see `game::th07::sight`: which bit of a
//! word is this game's shot, where its player is and what its own state byte says has just happened are
//! questions about *this* game, and Touhou is not one kind of game to take a union of.
//!
//! **And none of what to do about any of it is here.** What is read goes across as [`Frame`] and the rules
//! are `orb-bot`'s: a bullet the game has already spent and a body with nothing solid on it are handed over
//! with what the game says of them rather than left out, and a beam is handed over with the frame it starts
//! hitting on rather than judged.
//!
//! **Nothing here is a hitbox of its own.** What a hit is is the game's own test at 0x426c40, two boxes
//! overlapping, and both boxes are read: the player's own corners at `[player+0x458]`..`+0x468`, and each
//! bullet's `+0x550` or each enemy's `+0xc78`.

use orb_api::mem;

use crate::game::{Beam, Frame, Hazard, Kind, Pickup, Worth};

use super::{
    BUTTON_BOMB, BUTTON_DOWN, BUTTON_FOCUS, BUTTON_LEFT, BUTTON_MENU, BUTTON_RIGHT, BUTTON_SHOOT,
    BUTTON_SKIP, BUTTON_UP, G_BULLET_MANAGER, G_ENEMY_MANAGER, G_GAME_MANAGER, G_ITEM_MANAGER,
    G_MOTION_SCALE, G_PLAYER, G_SUPERVISOR, MENU_CANCEL, MENU_DECIDE, PLAYER_DEAD, PLAYER_NORMAL,
    POWER_FULL, STATE_GAMEMANAGER, Th06, bullet_manager, enemy, enemy_manager, game_manager, item,
    player, supervisor,
};

/// **The bit a recipe's button name is**, or `None` for a name th06 has nothing for.
///
/// The names are this game's, which is the line every other per-game thing in this tree is drawn on: `decide`
/// is a shot and a return key here and something else elsewhere.
pub fn bit(name: &str) -> Option<u16> {
    Some(match name {
        "shot" => BUTTON_SHOOT,
        "bomb" => BUTTON_BOMB,
        "focus" => BUTTON_FOCUS,
        "skip" => BUTTON_SKIP,
        "up" => BUTTON_UP,
        "down" => BUTTON_DOWN,
        "left" => BUTTON_LEFT,
        "right" => BUTTON_RIGHT,
        // The two masks every screen of this game is driven with — see [`super::MENU_DECIDE`].
        "decide" => MENU_DECIDE,
        "cancel" => MENU_CANCEL,
        // **And the button that opens the game's own pause menu**, which is the one road a recipe otherwise
        // cannot take — the same name th07 answers, and the same reason: what is behind it is that menu's own
        // items and everything they lead to.
        //
        // Its items are the game's own and its cursor is the game's, so a recipe presses down them blind:
        // `answer` is what orb's own questions have instead, and a menu of the game's has no such thing.
        "menu" => BUTTON_MENU,
        _ => return None,
    })
}

/// How far a candidate move is followed for one frame where the game's own step cannot be read at all, a bot
/// that stood still being worse than one that moved by a guess.
const A_PROBE: f32 = 8.0;

/// **One frame of a run, out of the game's own arrays**, and `false` off a stage.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn what_a_bot_sees(into: &mut Frame) -> bool {
    if !unsafe { in_a_stage() } {
        return false;
    }
    into.field = unsafe { field() };
    into.at = unsafe { player() };
    into.half = unsafe { player_box() };
    (into.straight, into.diagonal) = unsafe { a_step(false) };
    (into.focused_straight, into.focused_diagonal) = unsafe { a_step(true) };
    // **The frames a bomb still takes the hit back in.** `Player::Die` at 0x427770 writes the state 2, and the
    // four bombs write 3 over it — so a bomb pressed while it stands is a deathbomb and takes the death back.
    // It stands for `respawnTimer` frames and not for one — the 6 the last respawn left there (0x428f9e), which
    // the dying branch takes one off a frame (0x428a9a) — and that is what makes a word of the rules' able to
    // reach one at all.
    let state = unsafe { mem::read::<i8>(G_PLAYER + player::PLAYER_STATE) };
    into.hit = state == PLAYER_DEAD;
    into.normal = state == PLAYER_NORMAL;
    // **And no border**, which is a thing th07 has and this game has not: nothing here takes a hit for the
    // player, so every hit it takes is one the deaths count.
    into.border = false;
    // **And whether the shot is full**, which this game keeps as an integer and stops adding to at
    // [`POWER_FULL`] — the number its own item handlers compare against before they step it up.
    into.full_power =
        unsafe { mem::read::<u16>(G_GAME_MANAGER + game_manager::CURRENT_POWER) } >= POWER_FULL;
    // **And how many bombs there are to spend**, which is the game manager's own byte — see
    // `Frame::bombs`, whose reason is that the rules wait after one whether it went off or not.
    into.bombs = unsafe { mem::read::<i8>(G_GAME_MANAGER + game_manager::BOMBS_REMAINING) };
    into.dialogue = unsafe { crate::game::Game::a_line_of_dialogue_is_shown(&Th06) };
    into.boss = unsafe { the_boss_the_fight_is_with() };
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    for slot in 0..bullet_manager::BULLET_SLOTS {
        let bullet =
            G_BULLET_MANAGER + bullet_manager::BULLETS + slot * bullet_manager::BULLET_BYTES;
        // **Every state but nothing**, which is the walk's own gate at 0x414a41: a slot whose state is nought
        // holds nothing at all, and the game's own five cases are what the rest are.
        let state = unsafe { mem::read::<u16>(bullet + bullet_manager::BULLET_STATE) };
        if state == 0 {
            continue;
        }
        // Its own update adds its velocity times the motion scale to its position every frame — 0x415986 — and
        // that is an ordinary bullet's whole motion. Its own half is half the size the hit test is given.
        let (wx, wy) = unsafe { position(bullet + bullet_manager::BULLET_SIZE) };
        let (vx, vy) = unsafe { position(bullet + bullet_manager::BULLET_VELOCITY) };
        let scale = mem::read_committed::<f32>(G_MOTION_SCALE).unwrap_or(1.0);
        into.hazards.push(Hazard {
            kind: Kind::Bullet {
                hitting: state == bullet_manager::BULLET_IN_FLIGHT,
                state,
                next_change: None,
            },
            slot,
            frames: None,
            at: unsafe { position(bullet + bullet_manager::BULLET_POSITION) },
            step: Some((vx * scale, vy * scale)),
            half: (wx / 2.0, wy / 2.0),
        });
    }
    // **And the enemies, whose bodies are as solid as a bullet in this game too.** Which is what a run on its
    // way to an item walks into: the item was dropped by something, and that something is still standing over
    // it.
    //
    // **With a third of `+0xc78` on each side**, which is what 0x4126e5 and the test it calls make of that
    // size between them, **and with no step**, there being none on it: an enemy is moved by its own script.
    //
    // **And only the ones the game would test at all.** 0x4126a6 and 0x4126c2 gate the enemy's own collision
    // on the two lowest bits of [`enemy::SOLID`], and 0x412660 and 0x41267d the whole of it on two more before
    // those, so one failing any of them cannot touch the player however wide it is — handed over all the same,
    // with what the bytes say, since a hit naming nothing near it needs to know what was passed over.
    for slot in 0..enemy_manager::ENEMY_SLOTS {
        let at = G_ENEMY_MANAGER + enemy_manager::ENEMIES + slot * enemy_manager::ENEMY_BYTES;
        if unsafe { mem::read::<u8>(at + enemy::ALIVE) } & enemy::ALIVE_BIT == 0 {
            continue;
        }
        let flags = unsafe { mem::read::<u8>(at + enemy::SOLID) };
        let solid = flags & enemy::SOLID_BITS == enemy::SOLID_BITS
            && flags & enemy::TESTED_AT_ALL_BIT != 0
            && unsafe { mem::read::<u8>(at + enemy::NOT_TESTED) } & enemy::NOT_TESTED_BIT == 0;
        let (sx, sy) = unsafe { position(at + enemy::SIZE) };
        into.hazards.push(Hazard {
            kind: Kind::Body { solid },
            slot,
            // **Nothing, nothing read off this game saying how long the enemy in this slot has been there** —
            // see `Hazard::frames`, whose note has what a slot taken as an identity without one costs.
            frames: None,
            at: unsafe { position(at + enemy::POSITION) },
            step: None,
            half: (sx / enemy::HALF_OF_ITS_SIZE, sy / enemy::HALF_OF_ITS_SIZE),
        });
    }
    // **And the lasers, which are a line rather than a box**: the player's own test for one at 0x427190 turns
    // the box into the beam's own frame, which is *along* and *across*.
    //
    // **A warning line is handed over with the frames until it hits and not left out**: the game's own test is
    // made in the warning only once [`bullet_manager::LASER_HITS_FROM`] frames of it have gone, so until then
    // it hits nothing — and whether that is a frame to be elsewhere for is the bot's to decide. And a fading
    // beam past [`bullet_manager::LASER_HITS_UNTIL`] is not handed over at all, the test no longer being made.
    for slot in 0..bullet_manager::LASER_COUNT {
        let laser = G_BULLET_MANAGER + bullet_manager::LASERS + slot * bullet_manager::LASER_STRIDE;
        if unsafe { mem::read::<u32>(laser + bullet_manager::LASER_IN_USE) } == 0 {
            continue;
        }
        let phase = unsafe { mem::read::<u8>(laser + bullet_manager::LASER_PHASE) };
        let gone = unsafe { mem::read::<i32>(laser + bullet_manager::LASER_PHASE_FRAMES) };
        if phase >= bullet_manager::LASER_FADING
            && gone >= unsafe { mem::read::<i32>(laser + bullet_manager::LASER_HITS_UNTIL) }
        {
            continue;
        }
        into.beams.push(Beam {
            slot,
            origin: unsafe { position(laser + bullet_manager::LASER_ORIGIN) },
            angle: unsafe { mem::read::<f32>(laser + bullet_manager::LASER_ANGLE) },
            frames: unsafe { mem::read::<i32>(laser + bullet_manager::LASER_PHASE_FRAMES) },
            near: unsafe { mem::read::<f32>(laser + bullet_manager::LASER_NEAR) },
            far: unsafe { mem::read::<f32>(laser + bullet_manager::LASER_FAR) },
            // What 0x415e35 adds to the far end every frame, which is the only thing about a beam here that
            // moves: the near end follows it once the length the laser keeps at `+0x238` is reached.
            grows: unsafe { mem::read::<f32>(laser + bullet_manager::LASER_GROWS) }
                * mem::read_committed::<f32>(G_MOTION_SCALE).unwrap_or(1.0),
            half_across: unsafe { mem::read::<f32>(laser + bullet_manager::LASER_WIDTH) } / 2.0,
            // Before the firing phase, the frames until its warning starts hitting; from then on, none.
            fires_in: if phase < bullet_manager::LASER_FIRING {
                let from = unsafe { mem::read::<i32>(laser + bullet_manager::LASER_HITS_FROM) };
                (from - gone).max(0)
            } else {
                0
            },
            fading: phase >= bullet_manager::LASER_FADING,
        });
    }
    // **Every slot walked and not the live count taken**, because the count says how many there are and not
    // which slots they are in: the manager clears [`item::ALIVE`] and leaves the rest of a collected item's
    // numbers where they were, which is the same walk 0x41f537 makes.
    for slot in 0..item::SLOTS {
        let at = G_ITEM_MANAGER + slot * item::BYTES;
        if unsafe { mem::read::<i8>(at + item::ALIVE) } == 0 {
            continue;
        }
        into.items.push(Pickup {
            worth: worth(unsafe { mem::read::<i8>(at + item::KIND) }),
            at: unsafe { position(at + item::POSITION) },
        });
    }
    true
}

/// **Which of the shared kinds one of this game's items is**, as the table at 0x4200f8 indexes them.
///
/// **Both of the power items that fill it are the big one.** th06 has two on top of the ordinary power
/// item — one worth eight steps and one that fills the meter outright — and the shared vocabulary has a name
/// for neither of those separately, so both go across as the kind a bot crosses the field for. What is lost is
/// which of the two it was, and no rule of the bot's tells them apart.
///
/// And a bullet a bomb cleared is a point item: what it is worth is a score off the chain, which is what the
/// point item is worth.
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

/// The boss the fight is with, where there is one.
///
/// **One slot and not a walk**, which is the shape of this game: the manager keeps the one pointer at
/// [`enemy_manager::BOSSES`] and `read_state` reads the fight's life and timer through it already.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_boss_the_fight_is_with() -> Option<(f32, f32)> {
    let boss = unsafe { mem::read::<usize>(G_ENEMY_MANAGER + enemy_manager::BOSSES) };
    (boss != 0
        && mem::read_committed::<u8>(boss + enemy::ALIVE)
            .is_some_and(|alive| alive & enemy::ALIVE_BIT != 0))
    .then(|| unsafe { position(boss + enemy::POSITION) })
}

/// **Half the player's own box**, which is the player's half of what a hit is.
///
/// Off the two corners the game's own test is given rather than the half they are worked out from: those are
/// the numbers being compared, and a shot whose box the update has not filled in yet has nothing to be hit by
/// anyway.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player_box() -> (f32, f32) {
    let min = unsafe { position(G_PLAYER + player::BOX_MIN) };
    let max = unsafe { position(G_PLAYER + player::BOX_MAX) };
    (((max.0 - min.0) / 2.0).abs(), ((max.1 - min.1) / 2.0).abs())
}

/// Whether a stage is running to be steered in.
///
/// **The supervisor's own scene and the two menu flags**, which is what `read_state` answers `playing` and
/// `paused` out of: the rules press nothing off a stage, and a frame under the pause or retry menu is a frame
/// the game is not updating.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn in_a_stage() -> bool {
    let scene = unsafe { mem::read::<i32>(G_SUPERVISOR + supervisor::CUR_STATE) };
    let paused = unsafe { mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_GAME_MENU) } != 0
        || unsafe { mem::read::<u8>(G_GAME_MANAGER + game_manager::IS_IN_RETRY_MENU) } != 0;
    scene == STATE_GAMEMANAGER && !paused
}

/// **What one frame of holding a direction moves the player by**, as 0x427da5 works it out: the speed for that
/// kind of move, times the player's own scale for the axis, times [`G_MOTION_SCALE`].
///
/// **Whichever pair the caller asks for**, and both are asked for: the four speeds are the player's own fields
/// and the switch at 0x427a09 picks between them by the focus key, so which pair a frame takes is decided by a
/// key the *rules* press — see `Frame::focused_straight`, which is why the answer is not this reading's to
/// choose.
///
/// [`A_PROBE`] where any of it cannot be read or comes out as no step at all — a bot that stood still would be
/// worse than one that moved by a guess, and outside a run there is nothing there to read.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_step(focused: bool) -> ((f32, f32), (f32, f32)) {
    let motion = mem::read_committed::<f32>(G_MOTION_SCALE);
    let scale = |of: usize| {
        mem::read_committed::<f32>(G_PLAYER + of)
            .zip(motion)
            .map(|(own, motion)| own * motion)
    };
    let (across, down) = (scale(player::SCALE_ACROSS), scale(player::SCALE_DOWN));
    let step = |of: usize, scale: Option<f32>| {
        mem::read_committed::<f32>(G_PLAYER + of)
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

/// How near a thing has to be to the player for a hit to name it, whatever rule the model judged it by.
const NEAR_ENOUGH_TO_BE_IT: f32 = 16.0;

/// **Everything within [`NEAR_ENOUGH_TO_BE_IT`] of where the player was hit, out of the game's own arrays and
/// past every rule that leaves one out.**
///
/// Which is the question [`what_a_bot_sees`] cannot answer for a bot that was hit: what it was given is what
/// it decided from, so a hazard missing from that is missing from any reading of it. **Where the player is now
/// and not where the move was decided** — the state a hit is noticed by is written inside the game's own
/// update, so this walk happens a frame or two on and every distance measured from the older place carries
/// those frames as error.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn say_what_was_near() {
    let at = unsafe { player() };
    crate::detail!(
        "autoplay: what was near it is measured from {:.1},{:.1}, where the player is now",
        at.0,
        at.1,
    );
    // **The boxes and not the middles**, which is what a laser-shaped thing needs: it sits at its own pivot
    // with a size the length of the screen, so its middle is nowhere near a player its box covers.
    let near = |(x, y): (f32, f32), (wx, wy): (f32, f32)| {
        (x - at.0).abs() < NEAR_ENOUGH_TO_BE_IT + wx && (y - at.1).abs() < NEAR_ENOUGH_TO_BE_IT + wy
    };
    let mut nearest_bullet: Option<(usize, (f32, f32), f32, u16)> = None;
    for slot in 0..bullet_manager::BULLET_SLOTS {
        let bullet =
            G_BULLET_MANAGER + bullet_manager::BULLETS + slot * bullet_manager::BULLET_BYTES;
        let state = unsafe { mem::read::<u16>(bullet + bullet_manager::BULLET_STATE) };
        if state == 0 {
            continue;
        }
        let place = unsafe { position(bullet + bullet_manager::BULLET_POSITION) };
        let size = unsafe { position(bullet + bullet_manager::BULLET_SIZE) };
        let apart = (place.0 - at.0).hypot(place.1 - at.1);
        if nearest_bullet.is_none_or(|(.., was, _)| apart < was) {
            nearest_bullet = Some((slot, place, apart, state));
        }
        if near(place, (size.0 / 2.0, size.1 / 2.0)) {
            crate::detail!(
                "autoplay: near it, bullet {slot} state {state} at {:.1},{:.1} half {:.1},{:.1} \
                 velocity {:.1},{:.1}",
                place.0,
                place.1,
                size.0 / 2.0,
                size.1 / 2.0,
                unsafe { mem::read::<f32>(bullet + bullet_manager::BULLET_VELOCITY) },
                unsafe {
                    mem::read::<f32>(bullet + bullet_manager::BULLET_VELOCITY + size_of::<f32>())
                },
            );
        }
    }
    // **And the nearest of them whatever the reach**, which is the one thing a dump that answered nothing can
    // still be asked: a nearest bullet twenty units off is a model whose boxes are too small, and one two
    // hundred off is something that is in no array at all.
    match nearest_bullet {
        Some((slot, place, apart, state)) => crate::detail!(
            "autoplay: the nearest bullet is {slot}, {apart:.1} off at {:.1},{:.1} state {state}",
            place.0,
            place.1,
        ),
        None => crate::detail!("autoplay: there is no bullet on the screen at all"),
    }
    for slot in 0..enemy_manager::ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + enemy_manager::ENEMIES + slot * enemy_manager::ENEMY_BYTES;
        if unsafe { mem::read::<u8>(enemy + enemy::ALIVE) } & enemy::ALIVE_BIT == 0 {
            continue;
        }
        let place = unsafe { position(enemy + enemy::POSITION) };
        let (sx, sy) = unsafe { position(enemy + enemy::SIZE) };
        let half = (sx / enemy::HALF_OF_ITS_SIZE, sy / enemy::HALF_OF_ITS_SIZE);
        if near(place, half) {
            crate::detail!(
                "autoplay: near it, enemy {slot} at {:.1},{:.1} half {:.1},{:.1} solidity {:#04x}",
                place.0,
                place.1,
                half.0,
                half.1,
                unsafe { mem::read::<u8>(enemy + enemy::SOLID) },
            );
        }
    }
    for slot in 0..bullet_manager::LASER_COUNT {
        let laser = G_BULLET_MANAGER + bullet_manager::LASERS + slot * bullet_manager::LASER_STRIDE;
        if unsafe { mem::read::<u32>(laser + bullet_manager::LASER_IN_USE) } == 0 {
            continue;
        }
        crate::detail!(
            "autoplay: laser slot {slot} phase {} from {:.1},{:.1} angle {:.2} wide {:.1} \
             along {:.1}..{:.1}, {} frame(s) of its phase gone of {}",
            unsafe { mem::read::<u8>(laser + bullet_manager::LASER_PHASE) },
            unsafe { mem::read::<f32>(laser + bullet_manager::LASER_ORIGIN) },
            unsafe { mem::read::<f32>(laser + bullet_manager::LASER_ORIGIN + size_of::<f32>()) },
            unsafe { mem::read::<f32>(laser + bullet_manager::LASER_ANGLE) },
            unsafe { mem::read::<f32>(laser + bullet_manager::LASER_WIDTH) },
            unsafe { mem::read::<f32>(laser + bullet_manager::LASER_NEAR) },
            unsafe { mem::read::<f32>(laser + bullet_manager::LASER_FAR) },
            unsafe { mem::read::<i32>(laser + bullet_manager::LASER_PHASE_FRAMES) },
            unsafe { mem::read::<i32>(laser + bullet_manager::LASER_PHASE_LASTS) },
        );
    }
}

/// Where the player is.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player() -> (f32, f32) {
    unsafe { position(G_PLAYER + player::POSITION_CENTER) }
}

/// The two floats a position is, wherever one is kept.
///
/// # Safety
/// Must run on the game's main thread, with `at` the address of a position.
unsafe fn position(at: usize) -> (f32, f32) {
    (unsafe { mem::read::<f32>(at) }, unsafe {
        mem::read::<f32>(at + size_of::<f32>())
    })
}

/// The field the player may move in, as its left, top, width and height.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn field() -> (f32, f32, f32, f32) {
    let left_top = unsafe { position(G_GAME_MANAGER + game_manager::PLAYER_AREA_TOP_LEFT) };
    let size = unsafe { position(G_GAME_MANAGER + game_manager::PLAYER_AREA_SIZE) };
    (left_top.0, left_top.1, size.0, size.1)
}
