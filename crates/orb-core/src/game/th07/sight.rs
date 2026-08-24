//! **What 妖々夢 answers a bot with**: its own names for its buttons, and one frame of a stage read off its own
//! arrays.
//!
//! Its own file and this game's own, for the reason each game's resume file is its own: which bit of a word is
//! this game's shot, where its player is, and what its own state byte says has just happened are questions
//! about *this* game. A shared answer could only be the union of every game's vocabulary, and 東方 is not one
//! kind of game to take a union of.
//!
//! **And none of what to do about any of it is here.** What is read is handed over as [`Frame`] and the rules
//! are `orb-bot`'s — a bullet still coming out, a body with nothing solid on it and a beam still drawing its
//! warning all go across with what the game says of them rather than being left out here, because which of them
//! can be ignored is a rule and a rule that has been wrong twice. Reading is what needs these addresses.
//!
//! **Nothing here is a hitbox of its own**, which two launches paid for: one number standing in for every
//! bullet's size was four, and hits came back saying the move they were taken on had 2.1 and 3.7 units of room —
//! so no single number was ever going to be it. What a hit is is the game's own test, two boxes overlapping, and
//! both boxes are read: `[player+0x948]`..`+0x954` and each bullet's own `+0xb7c`.

use orb_api::mem;

use crate::game::{Beam, Frame, Hazard, Kind, Pickup, Worth};

use super::{
    BOSS_SLOTS, BOSSES, BULLET_BYTES, BULLET_HITS, BULLET_POSITION, BULLET_SIZE, BULLET_SLOTS,
    BULLET_STATE, BULLET_STEP, BULLETS, BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP,
    ENEMIES, ENEMY_ALIVE, ENEMY_ALIVE_BIT, ENEMY_BOX_OF_ITS_SIZE, ENEMY_BYTES, ENEMY_POSITION,
    ENEMY_SIZE, ENEMY_SLOTS, ENEMY_SOLID, ENEMY_SOLID_BITS, ENEMY_TRAIL, ENEMY_TRAIL_BYTES,
    ENEMY_TRAIL_EVERY, ENEMY_TRAIL_FROM, ENEMY_TRAIL_NODES, FIELD_HEIGHT, FIELD_LEFT, FIELD_TOP,
    FIELD_WIDTH, G_BULLET_MANAGER, G_ENEMY_MANAGER, G_GAME_MANAGER, G_ITEM_MANAGER, G_LASERS,
    G_MOTION_SCALE, G_PLAYER, G_SUPERVISOR, GAME_MENU_UP, ITEM_ALIVE, ITEM_BIG_POWER, ITEM_BOMB,
    ITEM_BYTES, ITEM_CHERRY, ITEM_KIND, ITEM_LIFE, ITEM_POINT, ITEM_POSITION, ITEM_POWER,
    ITEM_SLOTS, LASER_ALIVE, LASER_ANGLE, LASER_BYTES, LASER_FADING, LASER_FAR, LASER_LIVE_AT,
    LASER_NEAR, LASER_ORIGIN, LASER_PHASE, LASER_SLOTS, LASER_TIMER, LASER_WIDTH, MENU_CANCEL,
    MENU_DECIDE, PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_BORDER, PLAYER_BOX_MAX,
    PLAYER_BOX_MIN, PLAYER_HIT, PLAYER_NORMAL, PLAYER_POSITION, PLAYER_SCALE_ACROSS,
    PLAYER_SCALE_DOWN, PLAYER_SPEEDS, PLAYER_STATE, SCENE_RUN, SPEED_DIAGONAL, SPEED_STRAIGHT,
    SUPERVISOR_SCENE, Th07,
};

/// **The bit a recipe's button name is**, or `None` for a name 妖々夢 has nothing for.
///
/// The names are this game's, which is the line every other per-game thing in this tree is drawn on:
/// `Numbers` and `Reproduction` are 妖々夢's because their fields are, and these are because `decide` is a
/// shot and a return key here and something else elsewhere.
pub fn bit(name: &str) -> Option<u16> {
    Some(match name {
        "shot" => BUTTON_SHOT,
        "bomb" => BUTTON_BOMB,
        "focus" => BUTTON_FOCUS,
        "skip" => BUTTON_SKIP,
        "up" => PAD_UP,
        "down" => PAD_DOWN,
        "left" => PAD_LEFT,
        "right" => PAD_RIGHT,
        // The two masks every screen of this game is driven with — see [`Th07::menu_decide`].
        "decide" => MENU_DECIDE,
        "cancel" => MENU_CANCEL,
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
    (into.straight, into.diagonal) = unsafe { a_step() };
    // **The frame the player is hit and not the frames after it.** 0x43edc0 leaves the state at 2 and zeroes
    // the timer, and 0x440cf0 counts `[player+0x23f8]` down from there while the state stands — a window a bomb
    // can be pressed into rather than a single frame, and one whose end takes the life.
    let state = unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) };
    into.hit = state == PLAYER_HIT;
    into.normal = state == PLAYER_NORMAL;
    // **The border, which is what this game takes a hit on instead.** 0x441bd0 deflects a hit taken on one
    // rather than putting the player into the state above, so a run kept alive by borders is a run the deaths
    // never counted — see [`super::PLAYER_BORDER`].
    into.border = unsafe { mem::read::<usize>(G_PLAYER + PLAYER_BORDER) } != 0;
    into.dialogue = unsafe { Th07::dialogue_line() }.is_some();
    into.boss = unsafe { the_boss_the_fight_is_with(into.field) };
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        // **Every state but nothing, and the reading that said otherwise was wrong.** The dispatch at 0x425b82
        // reaches the player's own tests from the first of its five cases only, so leaving the other four out
        // looked safe — and a launch answered: it cleared the game with nine hits where the one before it had
        // four, eight of them on 6面's boss reading between six and fourteen units of room, which is the log's
        // shape for a hazard the model does not have.
        //
        // What the reading missed is that the state is not a phase a bullet passes through. 0x423a5d, 0x423e66,
        // 0x423f83 and 0x4240e7 write 1, 2, 3 and 4 from four different places in the spawn, so it is a kind of
        // bullet — and only one of them, at 0x425ff8, ever becomes the first. Whatever the other cases do reach,
        // this is not the reading to leave a bullet out on. Which state it was in goes across, so that a hit can
        // still say how many were of one the collision was not found in.
        let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
        if state == 0 {
            continue;
        }
        // Its own update adds its step to its position every frame — 0x425ccd adds `+0xb98` whole, and that is
        // an ordinary bullet's whole motion. And its own half, which is half of the size the hit test is given.
        let (wx, wy) = unsafe { position(bullet + BULLET_SIZE) };
        into.hazards.push(Hazard {
            kind: Kind::Bullet {
                hitting: state == BULLET_HITS,
            },
            slot,
            at: unsafe { position(bullet + BULLET_POSITION) },
            step: Some(unsafe { position(bullet + BULLET_STEP) }),
            half: (wx / 2.0, wy / 2.0),
        });
    }
    // **And the enemies, whose bodies are as solid as a bullet in this game.** Which is what a run on its way
    // to an item walks into: the item was dropped by something, and that something is still standing over it.
    //
    // **With its own body**, a third of `+0x2b3c` on each side, which is what 0x420490 and the box test it calls
    // make of that size between them: a boss taken for a point is a boss the model says there is room under
    // while its body reaches.
    //
    // **And with no step**, there being none on it: an enemy is moved by its own script and nothing anywhere on
    // it is a rate, so where one is going can only be the difference from where it was.
    //
    // **And whether it has a body to run into at all.** 0x420e86 and 0x420e9f gate the enemy's own collision on
    // the two bits of [`ENEMY_SOLID`], so an enemy with either down cannot touch the player — which 妖々夢's
    // 6面 boss relies on: it fires 自機狙い off an enemy its script keeps *at* the player. Handed over all the
    // same, with what the byte says, since a hit naming nothing near it needs to know what was passed over.
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES + slot * ENEMY_BYTES;
        if unsafe { mem::read::<u8>(enemy + ENEMY_ALIVE) } & ENEMY_ALIVE_BIT == 0 {
            continue;
        }
        let solid =
            unsafe { mem::read::<u8>(enemy + ENEMY_SOLID) } & ENEMY_SOLID_BITS == ENEMY_SOLID_BITS;
        let (sx, sy) = unsafe { position(enemy + ENEMY_SIZE) };
        let half = (
            sx / ENEMY_BOX_OF_ITS_SIZE / 2.0,
            sy / ENEMY_BOX_OF_ITS_SIZE / 2.0,
        );
        into.hazards.push(Hazard {
            kind: Kind::Body { solid },
            slot,
            at: unsafe { position(enemy + ENEMY_POSITION) },
            step: None,
            half,
        });
        if !solid {
            continue;
        }
        // **And every place it has been, each its own box.** A beam that follows an enemy is its trail, tested
        // box by box against the player at 0x4210cd with the enemy's own size — so each of them is a hazard of
        // exactly the kind a bullet is, and one enemy can carry a screen's worth of them.
        //
        // **Standing still, because a place already gone to does not move.** The trail holds where the enemy
        // *was*, so the shape of it is fixed in space and only its ends come and go.
        for node in unsafe { the_trail(enemy) } {
            into.hazards.push(Hazard {
                kind: Kind::Trail,
                slot,
                at: unsafe { position(node) },
                step: None,
                half,
            });
        }
    }
    // **And the lasers, which are the one hazard a point cannot stand in for**: a beam is a place rather than a
    // point, and the shape the game's own test at 0x43e6b0 makes of it is *along* and *across*.
    for slot in 0..LASER_SLOTS {
        let laser = G_LASERS + slot * LASER_BYTES;
        // **`+0x4d4` is the aliveness after all**, and the walk that owns the collision says so: 0x4263f9
        // compares it with nothing and skips the slot where it is, from a base of `[this]+0x366628` — which is
        // this same array, the constructor at 0x4233aa laying out its 0x40 slots of 0x4ec there.
        //
        // **So the beams that keep hitting the bot are not lasers.** 1面's last card and 6面's turning one both
        // read every one of these slots as empty while they are on the screen, and taking the gate away instead
        // put 57 of them in — stale geometry in slots holding nothing. Whatever those beams are, they are in the
        // bullet manager, which the model already has.
        if unsafe { mem::read::<u32>(laser + LASER_ALIVE) } == 0 {
            continue;
        }
        // **And no gate on the phase**, which is the wrong walk's gate: the two that stop at [`LASER_FADING`]
        // are the *bullet* ones, 0x4188eb and 0x424864, deciding whether a beam deletes a bullet. The player's
        // own test is called from every branch of the phase switch at 0x42657a — 0x426727 for phase 0, behind
        // the 予告線 timer, and 0x4267ec and 0x4269a5 for phases 1 and 2 with no gate before them at all. A
        // fading beam narrows and goes on hitting while it does, and a run was hit at `201.1,356.0` with the
        // model holding no lasers and twenty of them thrown away here.
        //
        // **And the frame it starts hitting on rather than whether it is drawn.** 0x4266dc gates the game's own
        // test on [`LASER_TIMER`] reaching [`LASER_LIVE_AT`], so a beam still drawing its 予告線 hits nothing at
        // all — which is a thing to be out of the way of and not a frame lost, and telling those apart is the
        // bot's.
        into.beams.push(Beam {
            slot,
            origin: unsafe { position(laser + LASER_ORIGIN) },
            angle: unsafe { mem::read::<f32>(laser + LASER_ANGLE) },
            near: unsafe { mem::read::<f32>(laser + LASER_NEAR) },
            far: unsafe { mem::read::<f32>(laser + LASER_FAR) },
            // **Nothing, this game's own growth not having been read.** th06 keeps a beam's own rate beside
            // its ends and th07 may as well; what is established here is the two ends, and a rate written
            // down on the strength of the other game's layout would be a number nothing measured.
            grows: 0.0,
            half_across: unsafe { mem::read::<f32>(laser + LASER_WIDTH) } / 2.0,
            fires_in: unsafe { mem::read::<i32>(laser + LASER_LIVE_AT) }
                - unsafe { mem::read::<i32>(laser + LASER_TIMER) },
            fading: unsafe { mem::read::<u8>(laser + LASER_PHASE) } >= LASER_FADING,
        });
    }
    // **Every slot walked and not the live count taken**, because the count says how many there are and not
    // which slots they are in: the manager leaves a collected item's own numbers where they were and clears
    // [`ITEM_ALIVE`], which is the same walk 0x432a3f makes.
    for slot in 0..ITEM_SLOTS {
        let item = G_ITEM_MANAGER + slot * ITEM_BYTES;
        if unsafe { mem::read::<i8>(item + ITEM_ALIVE) } == 0 {
            continue;
        }
        into.items.push(Pickup {
            worth: worth(unsafe { mem::read::<i8>(item + ITEM_KIND) }),
            at: unsafe { position(item + ITEM_POSITION) },
        });
    }
    true
}

/// **Which of the shared kinds one of this game's items is**, as the table at 0x433a5a indexes them: 0 steps the
/// power up by one and 2 by eight — 大パワーアイテム — 5 is the life and 3 the bomb, 1 is worth a score off the
/// chain and 7 fills the cherry.
///
/// The numbering is this game's and the names are not, which is the whole of why this mapping is here: what any
/// of them is *worth going for* is a rule, and a rule written against 妖々夢's own indices would be a rule this
/// file had to hold.
const fn worth(kind: i8) -> Worth {
    match kind {
        ITEM_LIFE => Worth::Life,
        ITEM_BOMB => Worth::Bomb,
        ITEM_BIG_POWER => Worth::BigPower,
        ITEM_POWER => Worth::Power,
        ITEM_POINT => Worth::Point,
        ITEM_CHERRY => Worth::Cherry,
        _ => Worth::Other,
    }
}

/// The boss the fight is with, where there is one on the screen and inside the play field.
///
/// **Every slot and not the first that is alive.** 妖々夢's 4面 is three bosses in the eight, and the ones the
/// fight is not with wait off the side of the field — alive, so no aliveness test passes them over, and outside
/// it, so nothing about their place is a place. Taking the first alive one and then finding it unusable is
/// answering "no boss": a launch stood in the middle of the field for the whole of that fight.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_boss_the_fight_is_with(field: (f32, f32, f32, f32)) -> Option<(f32, f32)> {
    (0..BOSS_SLOTS)
        .filter_map(|slot| {
            let boss =
                unsafe { mem::read::<usize>(G_ENEMY_MANAGER + BOSSES + slot * size_of::<usize>()) };
            (boss != 0
                && mem::read_committed::<u8>(boss + ENEMY_ALIVE)
                    .is_some_and(|alive| alive & ENEMY_ALIVE_BIT != 0))
            .then(|| unsafe { position(boss + ENEMY_POSITION) })
        })
        .find(|(x, _)| *x >= field.0 && *x <= field.0 + field.2)
}

/// **The addresses of the boxes an enemy's trail is**, which is the beam that follows it.
///
/// From [`super::ENEMY_TRAIL_FROM`], every [`super::ENEMY_TRAIL_EVERY`], as far as the count at
/// [`super::ENEMY_TRAIL_NODES`] — the walk 0x420eed to 0x420f0c makes, and no more of the trail than the game
/// itself tests.
///
/// # Safety
/// Must run on the game's main thread, on a live enemy.
unsafe fn the_trail(enemy: usize) -> impl Iterator<Item = usize> {
    let nodes = unsafe { mem::read::<i16>(enemy + ENEMY_TRAIL_NODES) }.max(0) as usize;
    (ENEMY_TRAIL_FROM..nodes)
        .step_by(ENEMY_TRAIL_EVERY)
        .map(move |node| enemy + ENEMY_TRAIL + node * ENEMY_TRAIL_BYTES)
}

/// **Half the player's own box**, which is the player's half of what a hit is.
///
/// Read rather than written down because it is the shot's own box and differs by character — and where it cannot
/// be read there is nothing to be hit anyway, a launch outside a run having no player.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player_box() -> (f32, f32) {
    let min = unsafe { position(G_PLAYER + PLAYER_BOX_MIN) };
    let max = unsafe { position(G_PLAYER + PLAYER_BOX_MAX) };
    (((max.0 - min.0) / 2.0).abs(), ((max.1 - min.1) / 2.0).abs())
}

/// Whether a stage is running to be steered in.
///
/// **The supervisor's own scene and not the run's numbers being there**, which is what this was and what it
/// cost: a launch reached the title screen with that pointer still set, so the rules ran with the play field
/// never filled in — a field of zeroes puts the y they want above its own top, and the [`PAD_UP`] that went in
/// wrapped 妖々夢's title menu from its first item round to 終了, which the shot beside it then took. The scene
/// is the same thing `Th07::read_state` answers `playing` out of, and it is a run's on no other screen.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn in_a_stage() -> bool {
    let scene = unsafe { mem::read::<i32>(G_SUPERVISOR + SUPERVISOR_SCENE) };
    let paused = unsafe { mem::read::<u8>(G_GAME_MANAGER + GAME_MENU_UP) };
    scene == SCENE_RUN && paused == 0
}

/// **What one frame of holding a direction moves the player by**, as the game works it out at 0x43f0f5 and
/// 0x43f378: the speed off the struct the player points at, times the player's own scale, times
/// [`G_MOTION_SCALE`] — and it comes back as the pair for a straight move and the pair for a diagonal one,
/// across and down each time.
///
/// **A pair each way and not one number**, because the scale is per axis: 0x43f37b multiplies by
/// [`PLAYER_SCALE_ACROSS`] and 0x43f39c by [`PLAYER_SCALE_DOWN`], and nothing says the two are equal. One of
/// them standing in for both is a projection whose y is out by their ratio every frame the player moves.
///
/// [`A_PROBE`] where any of it cannot be read or comes out as no step at all — a bot that stood still would be
/// worse than one that moved by a guess, and outside a run there is nothing there to read.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_step() -> ((f32, f32), (f32, f32)) {
    let motion = mem::read_committed::<f32>(G_MOTION_SCALE);
    let scale = |of: usize| {
        mem::read_committed::<f32>(G_PLAYER + of)
            .zip(motion)
            .map(|(own, motion)| own * motion)
    };
    let (across, down) = (scale(PLAYER_SCALE_ACROSS), scale(PLAYER_SCALE_DOWN));
    let speeds = mem::read_committed::<usize>(G_PLAYER + PLAYER_SPEEDS).filter(|at| *at != 0);
    let step = |of: usize, scale: Option<f32>| {
        speeds
            .zip(scale)
            .and_then(|(speeds, scale)| {
                mem::read_committed::<f32>(speeds + of).map(|speed| speed * scale)
            })
            .filter(|step| step.is_finite() && *step > 0.0)
            .unwrap_or(A_PROBE)
    };
    (
        (step(SPEED_STRAIGHT, across), step(SPEED_STRAIGHT, down)),
        (step(SPEED_DIAGONAL, across), step(SPEED_DIAGONAL, down)),
    )
}

/// How near a thing has to be to the player for a hit to name it, whatever rule the model judged it by.
const NEAR_ENOUGH_TO_BE_IT: f32 = 16.0;

/// **Everything within [`NEAR_ENOUGH_TO_BE_IT`] of where the player was hit, out of all three of the game's own
/// arrays and past every rule that leaves one out.**
///
/// Which is the question a hit reading `6.4 of room` with the nearest thing eight units off cannot answer: the
/// player was hit by something, and if it is in none of the three then none of the three is where it lives. Said
/// with the fields that would tell one kind from another, so that whatever it is can be singled out.
///
/// **Where the player is and not where it was when the move was decided.** The state the hit is noticed by is
/// written inside the game's own update and orb reads before that update runs, so this walk happens a frame or
/// two on: the player has moved and so has everything in the arrays. Measured against the older of the two
/// places, every distance carries those frames as error — eight units of it for a bullet crossing at four a
/// frame — which is enough for the thing that did it to read as nowhere near. The hit's own line says where the
/// player was; this walk is the only self-consistent reading of one frame.
///
/// **A dump and not a judgement**, which is why it is here and not with the rules: what it says is what the
/// arrays hold, past the rules, in the words this game's own fields are kept in.
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
    // **The boxes and not the middles.** A thing shaped like a beam sits at its pivot with a size the length of
    // the screen, so its middle is nowhere near the player while its box covers it — and a dump that asked about
    // middles said nothing was near a hit at all, which is how this came to be written.
    let near = |(x, y): (f32, f32), (wx, wy): (f32, f32)| {
        (x - at.0).abs() < NEAR_ENOUGH_TO_BE_IT + wx && (y - at.1).abs() < NEAR_ENOUGH_TO_BE_IT + wy
    };
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
        let place = unsafe { position(bullet + BULLET_POSITION) };
        let size = unsafe { position(bullet + BULLET_SIZE) };
        if state != 0 && near(place, (size.0 / 2.0, size.1 / 2.0)) {
            crate::detail!(
                "autoplay: near it, bullet {slot} state {state} at {:.1},{:.1} half {:.1},{:.1} \
                 step {:.1},{:.1}",
                place.0,
                place.1,
                size.0 / 2.0,
                size.1 / 2.0,
                unsafe { mem::read::<f32>(bullet + BULLET_STEP) },
                unsafe { mem::read::<f32>(bullet + BULLET_STEP + size_of::<f32>()) },
            );
        }
    }
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES + slot * ENEMY_BYTES;
        let place = unsafe { position(enemy + ENEMY_POSITION) };
        let (sx, sy) = unsafe { position(enemy + ENEMY_SIZE) };
        let half = (
            sx / ENEMY_BOX_OF_ITS_SIZE / 2.0,
            sy / ENEMY_BOX_OF_ITS_SIZE / 2.0,
        );
        if unsafe { mem::read::<u8>(enemy + ENEMY_ALIVE) } & ENEMY_ALIVE_BIT != 0
            && near(place, half)
        {
            crate::detail!(
                "autoplay: near it, enemy {slot} at {:.1},{:.1} half {:.1},{:.1} \
                 solidity {:#04x}",
                place.0,
                place.1,
                half.0,
                half.1,
                unsafe { mem::read::<u8>(enemy + ENEMY_SOLID) },
            );
        }
    }
    // **Every slot that holds anything, and not every slot the walk would visit.** `+0x4d4` is the word the
    // game's own walk skips on, so a beam kept some other way is a beam orb has never once seen — and a card
    // whose beam is plainly across the player while the model says nought of them is either that or nothing at
    // all. The width is what says a slot holds geometry rather than zeroes.
    for slot in 0..LASER_SLOTS {
        let laser = G_LASERS + slot * LASER_BYTES;
        let alive = unsafe { mem::read::<u32>(laser + LASER_ALIVE) };
        let width = unsafe { mem::read::<f32>(laser + LASER_WIDTH) };
        if alive == 0 && width == 0.0 {
            continue;
        }
        crate::detail!(
            "autoplay: laser slot {slot} holds alive {alive} phase {} from {:.1},{:.1} angle {:.2} \
             wide {width:.1} along {:.1}..{:.1} timer {} of {}",
            unsafe { mem::read::<u8>(laser + LASER_PHASE) },
            unsafe { mem::read::<f32>(laser + LASER_ORIGIN) },
            unsafe { mem::read::<f32>(laser + LASER_ORIGIN + size_of::<f32>()) },
            unsafe { mem::read::<f32>(laser + LASER_ANGLE) },
            unsafe { mem::read::<f32>(laser + LASER_NEAR) },
            unsafe { mem::read::<f32>(laser + LASER_FAR) },
            unsafe { mem::read::<i32>(laser + LASER_TIMER) },
            unsafe { mem::read::<i32>(laser + LASER_LIVE_AT) },
        );
    }
    unsafe { say_the_nearest_of_each(at) };
}

/// **The nearest thing of each of the three kinds, with no reach to fall outside of and no rule applied.**
///
/// Which is the last question the arrays can be asked. [`say_what_was_near`] asks what is within a reach, and
/// six hits in three practices answered nothing at all with three hundred bullets on the screen — an answer that
/// is the same whether the thing that did it is a little further off than the reach or in no array at all. These
/// three lines tell those apart: a nearest bullet twenty units off is a model whose boxes are too small, and a
/// nearest bullet two hundred off is a beam that is in none of them.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn say_the_nearest_of_each(at: (f32, f32)) {
    /// The nearest of one kind: which slot, where, how far off, what its own state or solidity byte says, and
    /// the size its box is made of.
    struct Nearest {
        slot: usize,
        at: (f32, f32),
        apart: f32,
        says: u32,
        size: f32,
    }
    let mut bullet_at: Option<Nearest> = None;
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        let state = unsafe { mem::read::<u16>(bullet + BULLET_STATE) };
        if state == 0 {
            continue;
        }
        let place = unsafe { position(bullet + BULLET_POSITION) };
        let apart = (place.0 - at.0).hypot(place.1 - at.1);
        if bullet_at.as_ref().is_none_or(|was| apart < was.apart) {
            bullet_at = Some(Nearest {
                slot,
                at: place,
                apart,
                says: state.into(),
                size: unsafe { mem::read::<f32>(bullet + BULLET_SIZE) },
            });
        }
    }
    match bullet_at {
        Some(near) => crate::detail!(
            "autoplay: the nearest bullet is {}, {:.1} off at {:.1},{:.1} state {} size {:.1}",
            near.slot,
            near.apart,
            near.at.0,
            near.at.1,
            near.says,
            near.size,
        ),
        None => crate::detail!("autoplay: there is no bullet on the screen at all"),
    }
    let mut enemy_at: Option<Nearest> = None;
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES + slot * ENEMY_BYTES;
        if unsafe { mem::read::<u8>(enemy + ENEMY_ALIVE) } & ENEMY_ALIVE_BIT == 0 {
            continue;
        }
        let place = unsafe { position(enemy + ENEMY_POSITION) };
        let apart = (place.0 - at.0).hypot(place.1 - at.1);
        if enemy_at.as_ref().is_none_or(|was| apart < was.apart) {
            enemy_at = Some(Nearest {
                slot,
                at: place,
                apart,
                says: unsafe { mem::read::<u8>(enemy + ENEMY_SOLID) }.into(),
                size: unsafe { mem::read::<f32>(enemy + ENEMY_SIZE) },
            });
        }
    }
    match enemy_at {
        Some(near) => crate::detail!(
            "autoplay: the nearest enemy is {}, {:.1} off at {:.1},{:.1} solidity {:#04x} \
             size {:.1}",
            near.slot,
            near.apart,
            near.at.0,
            near.at.1,
            near.says,
            near.size,
        ),
        None => crate::detail!("autoplay: there is no enemy on the screen at all"),
    }
}

/// Where the player is.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player() -> (f32, f32) {
    unsafe { position(G_PLAYER + PLAYER_POSITION) }
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

/// The play field, as its left, top, width and height.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn field() -> (f32, f32, f32, f32) {
    (
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_LEFT) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_TOP) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_WIDTH) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + FIELD_HEIGHT) },
    )
}
