//! What a bot playing th08 is told of it, which is this game's own arrays read.
//!
//! **Every address here is in `th08/mod.rs` beside its own reading**, and `asm.md` is where each came
//! from. What this file is, is the shape those readings take for `orb_bot`: the field, the player, the
//! player's own box, the four speeds, one [`Hazard`] per live enemy — with one more per box a long
//! enemy's own trail is tested at — and per live bullet, one [`Beam`] per live laser and one [`Pickup`]
//! per item on the list.
//!
//! **The field is measured from its own corner and th07's is not**, which is the one thing here that
//! does not carry across: a position of this game is 0 to 384 across and 0 to 448 down — the player's
//! own start is `width / 2` and `height - 64` (0x44d745 and 0x44d767) — where th07's are screen
//! coordinates with the field's left at 32. So [`field`] answers a left and a top of nothing while
//! reading the same two of the game manager's four for the size.
//!
//! **What is not here**, and each is an answer rather than a gap:
//!
//! - **a laser's own turn**, there being no rate on its angle to read: a beam that sweeps is one whose
//!   next angle is the difference between the two frames the rules hold.

use std::sync::atomic::{AtomicBool, Ordering};

use orb_api::mem;

use super::{
    BEAM_ANCHOR_ALONG, BEAM_ANCHOR_SLACK, BEAM_BEHAVIOURS, BEAM_LENGTH, BEAM_NARROWEST, BOMBS,
    BULLET_ACCELERATES, BULLET_ACCELERATION, BULLET_ACCELERATION_FRAMES,
    BULLET_ACCELERATION_FRAMES_RUN, BULLET_BEHAVIOURS, BULLET_BYTES, BULLET_FRAMES, BULLET_HITS,
    BULLET_KIND, BULLET_POSITION, BULLET_SIZE, BULLET_SLOTS, BULLET_SPRITE, BULLET_STEP,
    BULLET_UNTESTED, BULLETS_AT, CARD_PAYS_ITS_LEAST_UNDER, CARD_TIME_LEFT, ENEMIES_AT,
    ENEMY_ALIVE, ENEMY_ANGLE, ENEMY_ATTACK_FRAMES, ENEMY_BOX_BYTES, ENEMY_BOX_EVERY,
    ENEMY_BOX_FLAGS, ENEMY_BOX_OF_ITS_SIZE, ENEMY_BOX_SKIPPED, ENEMY_BOXES, ENEMY_BOXES_AT,
    ENEMY_BOXES_TAPER, ENEMY_BYTES, ENEMY_FAMILIAR, ENEMY_FIRST_BOX, ENEMY_FRAMES, ENEMY_POSITION,
    ENEMY_SCRIPT, ENEMY_SIZE, ENEMY_SLOTS, ENEMY_SOLID, ENEMY_UNTESTED,
    FAMILIARS_PASSED_THROUGH_BY, G_BOXES_SKIPPED_WHILE, G_BULLET_MANAGER, G_CHARACTER,
    G_ENEMY_MANAGER, G_FRAME_STEP, G_GAME_MANAGER, G_ITEM_MANAGER, G_PLAYER, G_SPELL_CARD,
    G_TEAM_SPEC, GAUGE, GAUGE_HUMAN_END, ITEM_CLASS, ITEM_LIST, ITEM_NEXT, ITEM_POSITION,
    ITEM_REACH, ITEM_SLOTS, ITEM_UNREACHABLE, LASER_ALIVE, LASER_ANGLE, LASER_BYTES, LASER_FADING,
    LASER_FAR, LASER_GROWS, LASER_LIVE_AT, LASER_NEAR, LASER_ORIGIN, LASER_PHASE, LASER_SLOTS,
    LASER_TIMER, LASER_WIDTH, LASERS_AT, PLAYER_BOUNDS_HEIGHT, PLAYER_BOUNDS_LEFT,
    PLAYER_BOUNDS_TOP, PLAYER_BOUNDS_WIDTH, PLAYER_BOX_FAR, PLAYER_BOX_NEAR, PLAYER_HIT,
    PLAYER_NORMAL, PLAYER_POSITION, PLAYER_STATE, POWER, SCRIPT_BEAM_ANCHOR, SCRIPT_BEHAVIOUR,
    SPEC_POWER_FULL, SPEED_DIAGONAL, SPEED_FOCUSED_DIAGONAL, SPEED_FOCUSED_STRAIGHT,
    SPEED_STRAIGHT, SPRITE_CASE, SPRITE_ELAPSED, SPRITE_FLAGS, SPRITE_NEVER_ENDS, SPRITE_SCRIPT,
    SPRITE_WAIT, TIME_ORBS, TIME_ORBS_WANTED, Th08,
};
use crate::game::{Beam, FIRES_WHEN_NOTHING_SAYS, Frame, Hazard, Kind, Pickup, Worth};
use crate::trace;

/// How far a candidate move is followed for one frame where a speed cannot be read at all, a bot that
/// stood still being worse than one that moved by a guess. th07's own answer, and the same number.
const A_PROBE: f32 = 8.0;

/// Whether the hit the player is in has been traced — see [`trace_what_stood_over_a_hit`].
static A_HIT_WAS_TRACED: AtomicBool = AtomicBool::new(false);

/// How far from the player an enemy's or a bullet's own box is traced on the frame a hit is seen: three of the
/// player's own whole steps, which covers what moved onto it over the frame between the game's test and this
/// read. **Measured from the box's edge and not its middle**, a body 48 by 80 reaching the player from a
/// middle thirty units off.
const TRACED_WITHIN: f32 = 12.0;

/// **What stood over the player on the frame a hit is seen**, once a hit and into the trace — see
/// [`trace::OverAHit`]: every live enemy and every bullet in any kind within [`TRACED_WITHIN`] of the player,
/// with the words the game's own four tests are gated on.
///
/// **Because a hit can come with nothing in the model.** `「蓬莱人形」` hit a run three times with `inf of room`
/// on all nine moves and `nothing was near it at all`, a bodiless enemy standing where the player stood each
/// time; which of the four ways the game takes a life it was is in that enemy's flags, its familiar word, its
/// script's behaviour and its anchor, and in the kind of any bullet there. **Fields of the trace and not
/// lines of the log**, which somebody watching asked for: a check reads them, where a line is read by a person.
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
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES_AT + slot * ENEMY_BYTES;
        let flags = unsafe { mem::read::<u32>(enemy + ENEMY_ALIVE) };
        let place = unsafe { position(enemy + ENEMY_POSITION) };
        let size = unsafe { position(enemy + ENEMY_SIZE) };
        if flags & 1 == 0 || !near(place, size) {
            continue;
        }
        let script = mem::read_committed::<usize>(enemy + ENEMY_SCRIPT).filter(|it| *it != 0);
        over.enemies.push(trace::AnEnemyOverAHit {
            slot: slot as u32,
            at: place,
            size,
            flags,
            familiar: unsafe { mem::read::<u32>(enemy + ENEMY_FAMILIAR) },
            behaviour: script
                .and_then(|it| mem::read_committed::<usize>(it + SCRIPT_BEHAVIOUR))
                .map(|it| it as u32),
            anchor: script.and_then(|it| {
                mem::read_committed::<f32>(it + SCRIPT_BEAM_ANCHOR).zip(mem::read_committed::<f32>(
                    it + SCRIPT_BEAM_ANCHOR + size_of::<f32>(),
                ))
            }),
            angle: unsafe { mem::read::<f32>(enemy + ENEMY_ANGLE) },
        });
    }
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS_AT + slot * BULLET_BYTES;
        let kind = unsafe { mem::read::<u16>(bullet + BULLET_KIND) };
        let place = unsafe { position(bullet + BULLET_POSITION) };
        let size = unsafe { position(bullet + BULLET_SIZE) };
        if kind == 0 || !near(place, size) {
            continue;
        }
        over.bullets.push(trace::ABulletOverAHit {
            slot: slot as u32,
            at: place,
            size,
            kind,
            hidden: unsafe { mem::read::<u8>(bullet + BULLET_UNTESTED) },
        });
    }
    trace::over_a_hit(over);
}

/// **Every bullet on the field into the trace, on a frame of the card a launch named** — see
/// `trace::TheField` — with the words [`trace_what_stood_over_a_hit`] reads of each, and how far into the card
/// the boss's own attack clock is.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn trace_the_field(at: (f32, f32)) {
    let card = unsafe { Th08.the_card_that_is_up() }.and_then(|card| u32::try_from(card).ok());
    let Some(card) = card.filter(|_| trace::the_field_is_wanted_in(card)) else {
        return;
    };
    let card_frames = unsafe { Th08.the_boss_enemy() }
        .and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_ATTACK_FRAMES))
        .unwrap_or(-1);
    let mut field = trace::TheField {
        card,
        at,
        card_frames,
        bullets: Vec::new(),
        beams: Vec::new(),
        boss: None,
        bodies: Vec::new(),
    };
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS_AT + slot * BULLET_BYTES;
        let kind = unsafe { mem::read::<u16>(bullet + BULLET_KIND) };
        if kind == 0 {
            continue;
        }
        field.bullets.push(trace::ABulletOverAHit {
            slot: slot as u32,
            at: unsafe { position(bullet + BULLET_POSITION) },
            size: unsafe { position(bullet + BULLET_SIZE) },
            kind,
            hidden: unsafe { mem::read::<u8>(bullet + BULLET_UNTESTED) },
        });
    }
    trace::the_field(field);
}

/// **One frame of a run, out of the game's own arrays**, and `false` off a stage.
///
/// # Safety
/// Must run on the game's main thread, between frames.
pub unsafe fn what_a_bot_sees(into: &mut Frame) -> bool {
    let Some(run) = (unsafe { Th08.run_numbers() }) else {
        return false;
    };
    into.field = unsafe { field() };
    into.at = unsafe { position(G_PLAYER + PLAYER_POSITION) };
    unsafe { trace_the_field(into.at) };
    into.half = unsafe { player_box() };
    let (straight, diagonal, focused_straight, focused_diagonal) = unsafe { steps() };
    into.straight = (straight, straight);
    into.diagonal = (diagonal, diagonal);
    into.focused_straight = (focused_straight, focused_straight);
    into.focused_diagonal = (focused_diagonal, focused_diagonal);
    let state = unsafe { mem::read::<u8>(G_PLAYER + PLAYER_STATE) };
    into.hit = state == PLAYER_HIT;
    if into.hit != A_HIT_WAS_TRACED.swap(into.hit, Ordering::Relaxed) && into.hit {
        unsafe { trace_what_stood_over_a_hit(into.at) };
    }
    into.normal = state == PLAYER_NORMAL;
    into.border = false;
    into.full_power = unsafe { full_power(run) };
    into.bombs = mem::read_committed::<f32>(run + BOMBS).map_or(0, |bombs| bombs as i8);
    into.dialogue = unsafe { Th08.a_line_is_up() };
    // **A bullet out of its hitting state is nothing to keep away from on this game** — see
    // [`BULLET_HITS`] for the walk that says so, and `Frame::a_bullet_not_hitting_is_harmless` for what
    // the rules do with it.
    into.a_bullet_not_hitting_is_harmless = true;
    // **And a boss shot unfocused for as long as the stage's norm is not met**, the gauge's end being what
    // makes a hit worth a time orb — see `Frame::shoots_a_boss_unfocused`, and `TIME_ORBS` for what the
    // orbs decide. Past the norm the orbs have bought what they buy, the Last Spell, and the focused shot's
    // concentrated damage is what the fight wants; asked every frame, so a miss that takes the count back
    // under the norm puts the unfocused shot back.
    let under_the_norm = match (
        mem::read_committed::<u32>(run + TIME_ORBS),
        mem::read_committed::<u32>(run + TIME_ORBS_WANTED),
    ) {
        (Some(orbs), Some(norm)) => orbs < norm,
        _ => true,
    };
    into.shoots_a_boss_unfocused = under_the_norm;
    // **And whether the gauge is at the end that pays for a hit** — see `Frame::at_the_gauges_human_end`.
    into.at_the_gauges_human_end = mem::read_committed::<i16>(run + GAUGE)
        .zip(mem::read_committed::<i16>(G_GAME_MANAGER + GAUGE_HUMAN_END))
        .map(|(gauge, end)| gauge <= end);
    let mut a_familiar_is_out = false;
    into.boss = unsafe { Th08.the_boss() };
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    // **Plain reads down the three arrays, and not committed ones.** The enemy, bullet and laser pools
    // are inside their managers, which are statics of the exe — `.data` from the attach to the exit — so
    // there is nothing for a committed read to refuse; what one costs is a `VirtualQuery` a slot, and
    // 0x1e0 + 0x600 + 0x100 of those a frame was a launch running at sixteen frames a second whatever
    // `--pace` said, half of it here and half in the reproduction line's own walk. The item list is
    // the exception below: a chain of pointers, and read committed as one.
    let character = unsafe { mem::read::<u8>(G_CHARACTER) };
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES_AT + slot * ENEMY_BYTES;
        let flags = unsafe { mem::read::<u32>(enemy + ENEMY_ALIVE) };
        if flags & 1 == 0 {
            continue;
        }
        // **And the beam an enemy's own script lays**, body or no body — see [`ENEMY_SCRIPT`]: a 590-long
        // box along the enemy's angle, and a player inside it is a player the bullets and the bodies never
        // touched. The context is the game's own allocation, so it is read committed; the enemies are few.
        //
        // **What says there is one is the anchor and not the behaviour** — see [`BEAM_ANCHOR_ALONG`]: the
        // behaviour field turns on the frame the box first tests the player, which is the frame a run
        // standing in it dies, so the rules would get no frame to move. The anchor is the beam's own half
        // length along the angle and the game writes it hundreds of frames earlier, which is the warning.
        // **The width the behaviour running tests, where one is** — which of the three it is says that —
        // and [`BEAM_NARROWEST`] where none is yet.
        //
        // **And the behaviour is also what says the beam is live**, the anchor saying only that this enemy is
        // one that lays one: stage 4's fight arms four enemies with an anchor apiece and fires them in turn,
        // so a lane read live for as long as it is armed is four of them at once — the rules died 95 times at
        // `91.4,432.0` inside one with `nowhere open to it`, every retry of the chapter the same frame of it.
        // So an armed one is handed over as a beam [`FIRES_WHEN_NOTHING_SAYS`] frames off, which the rules
        // read as a warning.
        //
        // **And a behaviour running is a beam whatever the anchor is.** The anchor along the angle is how an
        // armed enemy is told from one that is not, and the behaviour asks nothing of it: 0x424730 lays its
        // box from `position - anchor` along `[enemy+0x14]` and tests it with the anchor it finds (0x42474a
        // through 0x424770), so a beam fired from any other one is a beam the game tests the player against.
        let firing = |context: usize| {
            mem::read_committed::<usize>(context + SCRIPT_BEHAVIOUR).and_then(|behaviour| {
                BEAM_BEHAVIOURS
                    .iter()
                    .find(|(it, _)| *it == behaviour)
                    .map(|(_, width)| *width)
            })
        };
        if let Some(context) =
            mem::read_committed::<usize>(enemy + ENEMY_SCRIPT).filter(|it| *it != 0)
            && let Some((ax, ay)) =
                mem::read_committed::<f32>(context + SCRIPT_BEAM_ANCHOR).zip(mem::read_committed::<
                    f32,
                >(
                    context + SCRIPT_BEAM_ANCHOR + size_of::<f32>(),
                ))
            && let angle = unsafe { mem::read::<f32>(enemy + ENEMY_ANGLE) }
            && let firing = firing(context)
            && (firing.is_some()
                || (ax - angle.cos() * BEAM_ANCHOR_ALONG).abs() < BEAM_ANCHOR_SLACK
                    && (ay - angle.sin() * BEAM_ANCHOR_ALONG).abs() < BEAM_ANCHOR_SLACK)
        {
            let width = firing.unwrap_or(BEAM_NARROWEST);
            let (x, y) = unsafe { position(enemy + ENEMY_POSITION) };
            into.beams.push(Beam {
                slot,
                origin: (x - ax, y - ay),
                angle,
                // **The enemy's own count**, which is what says the beam in this slot next frame is this
                // one and so what carries its turn — see `Beam::frames`. A nought here made every frame's
                // beam one of its own to the rules, which a beam that turned after the run was not: stage
                // 4B's Last Spell fires one that turns `0.0036` rad a frame onto the player until it points
                // at them, and a run that read it standing still stood where it arrived.
                frames: unsafe { mem::read::<i32>(enemy + ENEMY_FRAMES) },
                near: 0.0,
                far: BEAM_LENGTH,
                grows: 0.0,
                // **Half the width tested every frame**, and not the quarter a laser is handed: the test
                // halves what it is given once (0x44a793), and what it is given here is the width whole.
                half_across: width / 2.0,
                fires_in: match firing.is_some() {
                    true => 0,
                    false => FIRES_WHEN_NOTHING_SAYS,
                },
                fading: false,
            });
        }
        // **And whether it has a body to run into**, which is [`ENEMY_SOLID`]: handed over either way,
        // since a hit naming nothing near it needs to know what was passed over, and only the solid ones
        // are a lane to keep out of.
        //
        // **Its own half is a third of its size**, which is what 0x42c290 and the box test it calls make
        // of that size between them — see [`ENEMY_BOX_OF_ITS_SIZE`]. Half of it is what orb handed over
        // before this reading, and a bot given a body half again as wide as the one that kills it is a
        // bot that will not fly under a fairy it has to shoot.
        let (sx, sy) = unsafe { position(enemy + ENEMY_SIZE) };
        // **And the gate in front of that bit, which the game's own walk has and orb had not** — see
        // [`ENEMY_BOX_SKIPPED`]: an enemy whose flags carry that bit is passed over entirely, body and
        // trail alike, while [`G_BOXES_SKIPPED_WHILE`] is not nothing. A run of stage 6's
        // `蘇活「生命遊戯」` was handed a solid body `2408.0,2502.2` across — ten times the field — and had
        // `-1448.7 of room` on all nine moves for as long as it stood there.
        //
        // **And the three bits in front of that gate**, which pass an enemy over with no static asked — see
        // [`ENEMY_UNTESTED`]: stage 1 keeps one alive through its whole midsection with its solid bit set.
        let skipped = flags & ENEMY_UNTESTED != 0
            || flags & ENEMY_BOX_SKIPPED != 0
                && unsafe { mem::read::<u32>(G_BOXES_SKIPPED_WHILE) } != 0;
        // **And a familiar, for the two runs whose player the body test never takes a life of on one** — see
        // [`FAMILIARS_PASSED_THROUGH_BY`]. The body is handed over the way the others are, and not as solid.
        let a_familiar = unsafe { mem::read::<u32>(enemy + ENEMY_FAMILIAR) } != 0;
        a_familiar_is_out |= a_familiar;
        let passed_through = FAMILIARS_PASSED_THROUGH_BY.contains(&character) && a_familiar;
        let solid = flags & ENEMY_SOLID != 0 && !skipped && !passed_through;
        let half = |(x, y): (f32, f32)| {
            (
                (x / ENEMY_BOX_OF_ITS_SIZE / 2.0).abs(),
                (y / ENEMY_BOX_OF_ITS_SIZE / 2.0).abs(),
            )
        };
        // **And how long this enemy has been in the slot**, which is what says the body there next frame is
        // this one and not the next enemy the slot is given — see [`ENEMY_FRAMES`] and `Hazard::frames`.
        let frames = Some(unsafe { mem::read::<i32>(enemy + ENEMY_FRAMES) });
        into.hazards.push(Hazard {
            kind: Kind::Body { solid },
            slot,
            frames,
            at: unsafe { position(enemy + ENEMY_POSITION) },
            step: None,
            half: half((sx, sy)),
        });
        // **And the boxes it has past that one, where it is one of the long enemies** — see
        // [`ENEMY_BOXES_AT`], which has the walk. A [`Kind::Trail`] apiece and not a body: each is a
        // place the enemy has already been, so nothing will move it and only its ends come and go.
        //
        // **Behind the same two gates the game's own walk is behind**: nothing where the enemy has no
        // body to run into, and nothing where [`ENEMY_BOX_FLAGS`] is nothing at all, which is every
        // enemy but the long ones.
        let flagged = unsafe { mem::read::<u8>(enemy + ENEMY_BOX_FLAGS) };
        if !solid || flagged == 0 {
            continue;
        }
        let boxes = unsafe { mem::read::<i16>(enemy + ENEMY_BOXES) };
        let boxes = usize::try_from(boxes).unwrap_or(0);
        for along in (ENEMY_FIRST_BOX..boxes).step_by(ENEMY_BOX_EVERY) {
            let at = enemy + ENEMY_BOXES_AT + along * ENEMY_BOX_BYTES;
            // **Tapered where that byte says so**, which is `size * along / boxes`: the far end of a
            // long enemy is narrower than its head, and a bot told the head's width all the way down
            // would keep out of a lane the game would have let it through.
            let taper = match flagged & ENEMY_BOXES_TAPER != 0 {
                true => along as f32 / boxes as f32,
                false => 1.0,
            };
            into.hazards.push(Hazard {
                kind: Kind::Trail,
                slot,
                frames,
                at: unsafe { position(at) },
                step: None,
                half: half((sx * taper, sy * taper)),
            });
        }
    }
    // **And the familiars spared through a card's last frames while the norm is unmet**, which is what their
    // being alive pays at the end of the attack they belong to — see `Frame::familiars_are_worth_sparing`.
    // Past the norm the orbs have bought the Last Spell, and the fight wants whatever ends it soonest.
    let a_cards_last_frames = unsafe { Th08.the_card_that_is_up() }.is_some()
        && mem::read_committed::<i32>(G_SPELL_CARD + CARD_TIME_LEFT)
            .is_some_and(|left| left < CARD_PAYS_ITS_LEAST_UNDER);
    into.familiars_are_worth_sparing = under_the_norm && a_familiar_is_out && a_cards_last_frames;
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS_AT + slot * BULLET_BYTES;
        let state = unsafe { mem::read::<u16>(bullet + BULLET_KIND) };
        if state == 0 {
            continue;
        }
        // **Its own step, which is its velocity over what its kind divides that by** — see
        // [`BULLET_STEP`]. Handed over rather than left to the rules to difference: a launch with it
        // missing said `22 bullet(s) no step of their own explained` on every hit it reported, and a
        // difference is a frame behind where the field is the frame itself.
        let (mut vx, mut vy) = unsafe { position(bullet + BULLET_STEP) };
        // **And the step the next frame adds, where a behaviour is changing it** — see [`BULLET_BEHAVIOURS`]:
        // the acceleration goes into the step before the kind's own add, so the step read after a frame is
        // one acceleration short of the one about to be taken while the timer runs.
        let behaviours = unsafe { mem::read::<u32>(bullet + BULLET_BEHAVIOURS) };
        if behaviours & BULLET_ACCELERATES != 0
            && unsafe { mem::read::<i32>(bullet + BULLET_ACCELERATION_FRAMES_RUN) }
                < unsafe { mem::read::<i32>(bullet + BULLET_ACCELERATION_FRAMES) }
        {
            let (ax, ay) = unsafe { position(bullet + BULLET_ACCELERATION) };
            let frame_step = unsafe { mem::read::<f32>(G_FRAME_STEP) };
            vx += ax * frame_step;
            vy += ay * frame_step;
        }
        // **And how many of them the kind takes**, which for a bullet read in kinds 2 to 5 is more than one:
        // the frame after adds the divided share and the whole step both — see
        // `Th08::how_much_of_its_step_a_bullet_takes`. Read as the divided share alone, a fresh bullet was
        // three times under and a launch lost one fight fifty times over at one hit, the dump each time
        // naming three bullets that had moved `2.99` saying `2.00` in the game's own state 1.
        //
        // **And the whole step back off it on a frame the spawn cannot end**, which is every frame of one
        // but its last — see [`BULLET_SPRITE`]: there the bullet moves the divided share and nothing more,
        // and read at the ending frame's multiple instead a wall of them is three times too fast.
        let of_it = Th08.how_much_of_its_step_a_bullet_takes(state);
        let of_it = match (2..=4).contains(&state) && !unsafe { a_spawn_may_end(bullet) } {
            true => of_it - 1.0,
            false => of_it,
        };
        into.hazards.push(Hazard {
            // **Hitting in one of the five kinds and in none of the others** — see [`BULLET_HITS`]: the
            // box test that takes a life has one caller and it is kind 1's, the spawn kinds asking only
            // whether the boxes overlap and kind 5 asking nothing. Called dangerous whichever kind it
            // was in before this reading, which is a bot that will not fly through a mesh of them
            // coming out.
            //
            // **And hitting while the script has it hidden too, which the game's own tests are not** — see
            // `BULLET_UNTESTED`. A hidden bullet is shown again where it has got to and tested from that
            // frame, and handed over as harmless the rules would walk through it and be standing in it
            // then: stage 5's `散符「真実の月（インビジブルフルムーン）」` hid about 650 of its 900 bullets
            // for `97` to `101` frames of every `206`, read from outside through one whole card, and they
            // drift `0.3` a frame towards the run while hidden, so the nearest of them was `7` to `25` off
            // the run on each of the thirteen frames they were shown again — a run the rules kept off them.
            kind: Kind::Bullet {
                hitting: state == BULLET_HITS,
                state,
            },
            slot,
            // **And how long this bullet has been in the slot** — see [`BULLET_FRAMES`]: a step of its own is
            // not enough, the rules reading a bullet that moved otherwise than its step says as one a script
            // is moving, and across two bullets of one slot that difference is the distance between them.
            frames: Some(unsafe { mem::read::<i32>(bullet + BULLET_FRAMES) }),
            at: unsafe { position(bullet + BULLET_POSITION) },
            step: Some((vx * of_it, vy * of_it)),
            half: unsafe { half(bullet + BULLET_SIZE) },
        });
    }
    // **And the lasers, which live in the bullet manager in this game** — see [`LASERS_AT`]. A beam is a
    // place rather than a point, so it goes over as its own shape and not as a hazard: an origin, an
    // angle, and how far along it it reaches.
    for slot in 0..LASER_SLOTS {
        let laser = G_BULLET_MANAGER + LASERS_AT + slot * LASER_BYTES;
        if unsafe { mem::read::<u32>(laser + LASER_ALIVE) } == 0 {
            continue;
        }
        let phase = unsafe { mem::read::<u8>(laser + LASER_PHASE) };
        let frames = unsafe { mem::read::<i32>(laser + LASER_TIMER) };
        into.beams.push(Beam {
            slot,
            origin: unsafe { position(laser + LASER_ORIGIN) },
            angle: unsafe { mem::read::<f32>(laser + LASER_ANGLE) },
            frames,
            near: unsafe { mem::read::<f32>(laser + LASER_NEAR) },
            far: unsafe { mem::read::<f32>(laser + LASER_FAR) },
            // **The rate and not the field**, this game scaling the reach it works out per frame by
            // [`G_FRAME_STEP`] — 0x431bcf.
            grows: unsafe { mem::read::<f32>(laser + LASER_GROWS) }
                * unsafe { mem::read::<f32>(G_FRAME_STEP) },
            // **A quarter of the width**: 0x431c47 halves it into the box's own height and the test at
            // 0x44a7b6 halves that again, so a quarter each side is what the game tests.
            half_across: unsafe { mem::read::<f32>(laser + LASER_WIDTH) } / 2.0 / 2.0,
            // **Nothing to wait for outside the first phase.** [`LASER_LIVE_AT`] is asked only while the
            // warning line is growing (0x431e75); once it fires, and while it fades, the game tests the
            // player every frame.
            fires_in: if phase == 0 {
                (unsafe { mem::read::<i32>(laser + LASER_LIVE_AT) }) - frames
            } else {
                0
            },
            fading: phase >= LASER_FADING,
        });
    }
    // **And the items, off the list the manager keeps them in** — see [`ITEM_LIST`], and [`ITEM_CLASS`]
    // for what each is worth. Bounded by the pool's own size, a list orb has read wrong being a frame
    // that never ends otherwise.
    let mut item = mem::read_committed::<usize>(G_ITEM_MANAGER + ITEM_LIST).unwrap_or(0);
    for _ in 0..ITEM_SLOTS {
        if item == 0 {
            break;
        }
        // **Left out where the game's own test is skipped on it**, which is what `+0x2d7` of 3 means: an
        // item nothing the player does reaches is one a bot steered towards would sit under.
        if mem::read_committed::<u8>(item + ITEM_REACH) != Some(ITEM_UNREACHABLE)
            && let Some(class) = mem::read_committed::<u8>(item + ITEM_CLASS)
        {
            into.items.push(Pickup {
                worth: worth(class),
                at: unsafe { position(item + ITEM_POSITION) },
            });
        }
        item = mem::read_committed::<usize>(item + ITEM_NEXT).unwrap_or(0);
    }
    true
}

/// **Whether the spawn of this bullet can end on this frame**, off the sprite it draws itself with — see
/// [`BULLET_SPRITE`], which has what each of these answers is read off.
///
/// `false` only where the animation's own wait is still ahead of the frames it has waited, which is the
/// one thing 0x45ea00 can be asked without walking the animation's script. Everything else answers `true`,
/// which is the frame a spawning bullet adds its whole step as well as its share.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_spawn_may_end(bullet: usize) -> bool {
    let sprite = bullet + BULLET_SPRITE;
    let Some(script) = mem::read_committed::<usize>(sprite + SPRITE_SCRIPT).filter(|it| *it != 0)
    else {
        return true;
    };
    if unsafe { mem::read::<u32>(sprite + SPRITE_FLAGS) } & SPRITE_NEVER_ENDS != 0 {
        return false;
    }
    if unsafe { mem::read::<u16>(sprite + SPRITE_CASE) } != 0 {
        return true;
    }
    let Some(wait) = mem::read_committed::<i16>(script + SPRITE_WAIT) else {
        return true;
    };
    let elapsed = unsafe { mem::read::<i32>(sprite + SPRITE_ELAPSED) };
    i32::from(wait) <= elapsed
}

/// **Which of the shared kinds one of this game's items is**, out of the class [`ITEM_CLASS`] holds.
///
/// **The two the shared vocabulary has no name for go over as [`Worth::Other`]**, which ranks them below
/// everything named: class 6 is worth score off the graze and class 7 is what a cancelled bullet leaves,
/// three at a time, so a bot that crossed a screen for one of those would be crossing it for a thousandth
/// of what a life is worth. `Other` is what `orb_bot::steer` goes for only where it costs nothing.
fn worth(class: u8) -> Worth {
    match class {
        0 => Worth::Power,
        2 | 4 => Worth::BigPower,
        3 => Worth::Bomb,
        5 => Worth::Life,
        1 | 8 => Worth::Point,
        _ => Worth::Other,
    }
}

/// The play field, as a left, a top and a size — the left and the top being nothing, for the reason the
/// module's own doc gives.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn field() -> (f32, f32, f32, f32) {
    // **The rectangle the player is held inside, and not the play area** — see `PLAYER_BOUNDS_LEFT`: a
    // move is judged by where it lands, and a landing past the floor at 432 is one the game does not
    // make. Read rather than written down, the way the area's own two are.
    (
        unsafe { mem::read::<f32>(G_GAME_MANAGER + PLAYER_BOUNDS_LEFT) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + PLAYER_BOUNDS_TOP) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + PLAYER_BOUNDS_WIDTH) },
        unsafe { mem::read::<f32>(G_GAME_MANAGER + PLAYER_BOUNDS_HEIGHT) },
    )
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

/// A size read as a half of itself, which is what a hazard's own is: the game's tests build their boxes
/// as `position ± size / 2` — see `ENEMY_SIZE` in `th08/mod.rs`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn half(at: usize) -> (f32, f32) {
    let (width, height) = unsafe { position(at) };
    ((width / 2.0).abs(), (height / 2.0).abs())
}

/// The player's own half, out of the box the game keeps ready: the far corner less the near one, halved.
///
/// **Read off the box rather than off `PLAYER_HALF` itself**, because the box is what the game's own
/// tests compare against — so a frame where the two disagree is a frame the box wins.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player_box() -> (f32, f32) {
    let near = unsafe { position(G_PLAYER + PLAYER_BOX_NEAR) };
    let far = unsafe { position(G_PLAYER + PLAYER_BOX_FAR) };
    (
        ((far.0 - near.0) / 2.0).abs(),
        ((far.1 - near.1) / 2.0).abs(),
    )
}

/// **The four speeds, out of the team's own `.sht`** — straight, diagonal, and the two focused.
///
/// A launch measured them as 4.00, 2.83, 2.00 and 1.41 and the same launch found all four in this
/// object; see `SPEED_STRAIGHT` in `th08/mod.rs`. [`A_PROBE`] where one cannot be read, a bot that
/// stood still being worse than one that moved by a guess.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn steps() -> (f32, f32, f32, f32) {
    let spec = mem::read_committed::<usize>(G_TEAM_SPEC).filter(|it| *it != 0);
    let step = |at: usize| {
        spec.and_then(|spec| mem::read_committed::<f32>(spec + at))
            .filter(|speed| speed.is_finite() && *speed > 0.0)
            .unwrap_or(A_PROBE)
    };
    (
        step(SPEED_STRAIGHT),
        step(SPEED_DIAGONAL),
        step(SPEED_FOCUSED_STRAIGHT),
        step(SPEED_FOCUSED_DIAGONAL),
    )
}

/// Whether the shot is at the power it stops adding to, which this game keeps in the team's own `.sht`
/// as well as in the panel's own compare — see `SPEC_POWER_FULL`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn full_power(run: usize) -> bool {
    let spec = mem::read_committed::<usize>(G_TEAM_SPEC).filter(|it| *it != 0);
    let full = spec
        .and_then(|spec| mem::read_committed::<f32>(spec + SPEC_POWER_FULL))
        .filter(|full| full.is_finite() && *full > 0.0);
    match (mem::read_committed::<f32>(run + POWER), full) {
        (Some(power), Some(full)) => power >= full,
        _ => false,
    }
}
