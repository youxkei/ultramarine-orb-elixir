//! **What th06c answers a bot with**: one frame of a stage read off its own statics.
//!
//! Its own file and this game's own, for the reason th06's and th07's are: where its player is, what its
//! own state byte says has just happened and which of seven numbers an item is are questions about
//! *this* game, and Touhou is not one kind of game to take a union of.
//!
//! **And none of what to do about any of it is here.** What is read goes across as [`Frame`] and the
//! rules are `orb-bot`'s: an enemy the game will not run its body against is handed over with what the
//! game says of it rather than left out.
//!
//! **Nothing here is a hitbox of its own.** What a hit is is the game's own test at `0x14004c5d0`, two
//! boxes overlapping, and both boxes are read: the player's own corners at `[player+0x410]`..`+0x424`,
//! each bullet's `+0x5a4`, and each enemy's `+0x2a4` cut to a third.
//!
//! **Every address it reads goes through [`at`]**, this exe being built with ASLR on.

use orb_api::mem;

use crate::game::{Beam, Frame, Hazard, Kind, Pickup, Worth};

use super::{
    A_REPLAY_IS_PLAYING, BOMBS, BULLET_MANAGER, ENEMY_MANAGER, EVERYTHING_HELD, FIELD_SIZE,
    GAME_MANAGER, INPUT_THE_RUN_ACTS_ON, ITEM_MANAGER, LASERS, LIVES, PLAYER, PLAYER_AREA, POWER,
    POWER_FULL, STAGE, THE_PLAYBACK_IS_THE_DEMO, at, bullet_manager, enemy, enemy_manager,
    game_manager, item, laser, player,
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
    into.half = unsafe { player_box() };
    (into.straight, into.diagonal) = unsafe { a_step(false) };
    (into.focused_straight, into.focused_diagonal) = unsafe { a_step(true) };
    let state = unsafe { mem::read::<u8>(at(PLAYER) + player::STATE) };
    into.hit = state == player::HIT;
    into.normal = state == player::NORMAL;
    // **And no border**, which is a thing th07 has and this game has not: nothing here takes a hit for
    // the player, so every hit it takes is one the deaths count.
    into.border = false;
    into.full_power = unsafe { mem::read::<u16>(at(POWER)) } >= POWER_FULL;
    // **And how many bombs there are to spend**, which this game keeps as a byte of its own — see
    // `Frame::bombs`, whose reason is that the rules wait after one whether it went off or not.
    into.bombs = unsafe { mem::read::<i8>(at(BOMBS)) };
    // **Unread.** Where this game keeps the line of dialogue that is up is one of the readings
    // `docs/th06c/asm.md` still lists as not established; a line is pressed past by the shot the rules
    // hold anyway.
    into.dialogue = false;
    into.boss = unsafe { the_boss_the_fight_is_with() };
    into.hazards.clear();
    into.beams.clear();
    into.items.clear();
    for slot in 0..bullet_manager::SLOTS {
        let bullet = at(BULLET_MANAGER) + bullet_manager::BULLETS + slot * bullet_manager::BYTES;
        // The walk's own gate at `0x14000d9f0`: a slot whose state is nought holds nothing at all.
        let state = unsafe { mem::read::<u16>(bullet + bullet_manager::STATE) };
        if state == 0 {
            continue;
        }
        let (wx, wy) = unsafe { position(bullet + bullet_manager::SIZE) };
        let (vx, vy) = unsafe { position(bullet + bullet_manager::STEP) };
        let scale = bullet_manager::a_steps_scale(state);
        into.hazards.push(Hazard {
            kind: Kind::Bullet {
                hitting: state == bullet_manager::FLYING,
                state,
            },
            slot,
            at: unsafe { position(bullet + bullet_manager::POSITION) },
            // Its own update adds its step, scaled by whatever its state scales one by, to its position
            // every frame — and that is an ordinary bullet's whole motion.
            step: Some((vx * scale, vy * scale)),
            half: (wx / 2.0, wy / 2.0),
        });
    }
    // **And the lasers, which are a line rather than a box**: the player's own test for one at
    // `0x14004c7d0` turns the player about the laser's own origin by its angle and compares it with a
    // box that is *along* and *across*. What that box reaches across is a quarter of the width the beam
    // carries — see [`laser::HALF_OF_ITS_WIDTH`], which is two halvings read off the exe.
    //
    // **A warning is handed over with the frames until it fires and not left out**: `0x14000e4d1` runs
    // that test only once the frames have reached [`laser::HITS_FROM`], so one still warning hits
    // nothing at all — and whether that is a frame to be elsewhere for is the bot's to decide.
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
        // **And what a fading one takes down is its own length and not its width**, which is the one
        // thing about this game's lasers no reading would have guessed: `0x14000e5ed` works out
        // `width × (1 − frames ÷` [`laser::NARROWS_OVER`]`)` and writes it into the *length's* slot of
        // the pair the test is handed — `[rsp+0x30]`, which `0x14004c89a` compares along the beam — so
        // what is left of a fading laser is a stub of its own line about the middle of it, at the width
        // it always had. `docs/th06c/asm.md` has the registers.
        let (near, far) = match phase >= laser::FADING {
            true => {
                let narrows = unsafe { mem::read::<i32>(beam + laser::NARROWS_OVER) };
                match narrows > 0 {
                    true => {
                        let left = half_across * (1.0 - frames as f32 / narrows as f32);
                        let middle = (near + far) / 2.0;
                        (middle - left.max(0.0), middle + left.max(0.0))
                    }
                    false => (near, far),
                }
            }
            false => (near, far),
        };
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
                true => (unsafe { mem::read::<i32>(beam + laser::HITS_FROM) }) - frames,
                false => 0,
            },
            fading: phase >= laser::FADING,
        });
    }
    // **And the enemies, whose bodies are as solid as a bullet in this game too**, with a third of
    // `+0x2a4` on each side and with no step, there being none on it: an enemy is moved by its own
    // script.
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
            at: unsafe { position(body + enemy::POSITION) },
            step: None,
            half: (
                sx / enemy::A_THIRD_OF_ITS_SIZE,
                sy / enemy::A_THIRD_OF_ITS_SIZE,
            ),
        });
    }
    // **Every slot walked and not the live count taken**, because the count says how many there are and
    // not which slots they are in — the same walk `0x14002aae0` makes.
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

/// **Which of the shared kinds one of this game's items is**, as the table at `0x14002ba1c` indexes them.
///
/// **Both of the power items that fill it are the big one.** This game has two on top of the ordinary
/// power item — one worth eight steps and one that fills the meter outright — and the shared vocabulary
/// has a name for neither of those separately, so both go across as the kind a bot crosses the field for.
///
/// And the item a bomb leaves behind is a point item: what it is worth is a score off the graze, which is
/// what the point item is worth.
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

/// **The boss the fight is with**, where there is one.
///
/// **A walk and not a slot**, which is the shape of this game: nothing keeps a pointer to it, and what
/// marks it is a bit on the enemy itself — [`enemy::IS_THE_BOSS`], which `0x140020f62` is the only test
/// of and which decides whose life the panel's own bar is drawn from.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_boss_the_fight_is_with() -> Option<(f32, f32)> {
    unsafe { the_boss_the_fights_own_slot() }
        .map(|body| unsafe { position(body + enemy::POSITION) })
}

/// And the slot it is in, for what wants its life rather than where it is.
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

/// **Half the player's own box**, which is the player's half of what a hit is.
///
/// Off the two corners the game's own test is given rather than the half they are worked out from: those
/// are the numbers being compared at `0x14004c692` onwards.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn player_box() -> (f32, f32) {
    let min = unsafe { position(at(PLAYER) + player::BOX_MIN) };
    let max = unsafe { position(at(PLAYER) + player::BOX_MAX) };
    (((max.0 - min.0) / 2.0).abs(), ((max.1 - min.1) / 2.0).abs())
}

/// Whether a stage is running to be steered in.
///
/// **The stage the run has reached, the state the game manager is in, and the byte that holds
/// everything.** [`STAGE`] is nought until a run starts; [`A_STAGE`] is the one state of the twenty-one
/// a stage is played in; and a frame with [`EVERYTHING_HELD`] set is a frame nothing of the game moved
/// in.
///
/// **The state is there because the stage alone kept a launch pressing after its run was over**, which
/// is a number a run leaves behind it: the front end that came up after a game over still read
/// `stage 4`, so the rules went on pressing there and walked the title into a second run the recipe's
/// own `play_until` was still counting the first one's frames of.
///
/// **And the stage is there because the state alone is not the whole of it**: [`A_STAGE`] is the state
/// the game manager sits in for a stage, and `0x14002f95d`'s table of six — reached from twelve to
/// seventeen — is not a stage's phases at all. A launch that took those six for one never steered:
/// nothing pressed, and the run was over at `得点 000001800` with the panel's lives row empty and the
/// game asking 「コンティニュ〜？」.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn in_a_stage() -> bool {
    let held = unsafe { mem::read::<u8>(at(EVERYTHING_HELD)) } != 0;
    let running = unsafe { in_a_run() };
    running && !held
}

/// **Whether there is a run at all**, which is wider than a frame to steer in: a frame the game is
/// holding everything on is a frame of a run all the same, and [`EVERYTHING_HELD`] is what a death's own
/// animation sets.
///
/// **The two are separate because a `play_until` ends where the run does.** Asked of the narrower one, a
/// launch's own step ended on stage 5 with three lives left and the recipe closed the game after it: the
/// `run:` line either side of each death read `stage 0` — no run — and the frames between them were the
/// death being played out. Measured across four deaths of one launch.
///
/// **And a playback is none of it, however much of a stage it is.** This game plays its attract demo
/// through the same stage the player's own run uses: the state was 11, the stage 7 and the difficulty 4
/// while the screen said `DEMO PLAY` over stage 4's library, so a step that had played its run to the
/// end waited another 34,200 frames for a run that was over to be over. [`A_REPLAY_IS_PLAYING`] is what
/// tells the two apart, and what said so at the time was a picture: every line of the log was that same
/// reading.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn in_a_run() -> bool {
    unsafe { a_stage_is_the_scene() && !a_replay_is_playing() }
}

/// **Whether a stage is what the game is showing**, whoever or whatever is playing it: the stage the run
/// has reached and the state the game manager is in, which the demo holds exactly as a run does.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_stage_is_the_scene() -> bool {
    let scene = unsafe { mem::read::<i32>(at(GAME_MANAGER + game_manager::STATE)) };
    let stage = unsafe { mem::read::<i32>(at(STAGE)) };
    (1..=STAGES).contains(&stage) && scene == A_STAGE
}

/// **Whether the replay in memory is one being played back**, which is [`A_REPLAY_IS_PLAYING`] and a
/// question about that record rather than about the screen: the run's own start is what clears the byte
/// and the demo's own hand-over is not, so it stands through the title the demo hands back to.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn a_replay_is_playing() -> bool {
    unsafe { mem::read::<u8>(at(A_REPLAY_IS_PLAYING)) != 0 }
}

/// **And whether that playback is the attract demo**, which is the one nobody asked for. See
/// [`THE_PLAYBACK_IS_THE_DEMO`].
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn the_playback_is_the_demo() -> bool {
    unsafe { mem::read::<u8>(at(THE_PLAYBACK_IS_THE_DEMO)) != 0 }
}

/// The stages a run of this game has, the Extra counted: `0x14002712e` treats six as the last of the
/// numbered ones and the Extra is the seventh.
const STAGES: i32 = 7;

/// **The state of the game manager's twenty-one that a stage is played in**, read off the `run:` line a
/// launch writes each time its stage, difficulty or state changes. Walking a fresh launch into a run
/// said 0, 1, 2, 6, 7 and 9 for the screens of the front end, then `run: stage 1 … scene 11` for the
/// whole of the first stage and `run: stage 2 … scene 11` for the second.
const A_STAGE: i32 = 11;

/// The field the player may move in, as its left, top, width and height.
///
/// **[`PLAYER_AREA`] where it reads as a rectangle and the field's own size where it does not.** Nothing
/// in the exe writes those four floats — see [`PLAYER_AREA`] — so whether the game fills them is a
/// question this reading cannot answer off the exe, and a bot given a rectangle of nought would have
/// nowhere to move. The fallback is the size every other comparison of a coordinate is made against,
/// at the origin the player's own spawn is measured from.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn field() -> (f32, f32, f32, f32) {
    let (left, top) = unsafe { position(at(PLAYER_AREA)) };
    let (width, height) = unsafe { position(at(PLAYER_AREA) + 2 * size_of::<f32>()) };
    if width > 0.0 && height > 0.0 {
        return (left, top, width, height);
    }
    let (width, height) = unsafe { position(at(FIELD_SIZE)) };
    (0.0, 0.0, width, height)
}

/// **What one frame of holding a direction moves the player by**, as `0x14004af14` works it out: the
/// speed for that kind of move, times the player's own scale for the axis.
///
/// **Whichever pair the caller asks for**, and both are asked for: the four speeds are the player's own
/// fields and the switch at `0x14004acbf` picks between them by the focus bit, so which pair a frame
/// takes is decided by a key the *rules* press — see `Frame::focused_straight`, which is why the answer
/// is not this reading's to choose.
///
/// [`A_PROBE`] where any of it cannot be read or comes out as no step at all — a bot that stood still
/// would be worse than one that moved by a guess, and outside a run there is nothing there to read.
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

/// How far a candidate move is followed for one frame where the game's own step cannot be read at all, a
/// bot that stood still being worse than one that moved by a guess.
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
/// arrays and past every rule that leaves one out.**
///
/// Which is the question [`what_a_bot_sees`] cannot answer for a bot that was hit: what it was given is
/// what it decided from, so a hazard missing from that is missing from any reading of it.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn say_what_was_near() {
    let player = unsafe { position(at(PLAYER) + player::POSITION) };
    // **What the player is in the middle of, beside what was near it.** A run kept reporting hits with
    // nothing at all within sixteen of the player, and what tells a hit from a state the player is in for
    // some other reason is the frames a bomb still takes it back in: that counter is what a hit starts and
    // what the death is made final at the end of. See `player::A_BOMB_STILL_TAKES_IT_BACK`.
    crate::detail!(
        "autoplay: the player is in state {}, with {} frame(s) a bomb still takes it back in, {} of \
         being untouchable, and {} live",
        unsafe { mem::read::<u8>(at(PLAYER) + player::STATE) },
        unsafe { mem::read::<i32>(at(PLAYER) + player::A_BOMB_STILL_TAKES_IT_BACK) },
        unsafe { mem::read::<i32>(at(PLAYER) + player::UNTOUCHABLE) },
        unsafe { mem::read::<i8>(at(LIVES)) },
    );
    crate::detail!(
        "autoplay: what was near it is measured from {:.1},{:.1}, where the player is now",
        player.0,
        player.1,
    );
    // **The two corners the game's own test is handed, beside the position the model reads.** Those
    // corners are the whole of the player's half of a hit — `0x14004c692`–`0x14004c6c8` compares them
    // and nothing else — so a hit reported with room to spare on every one of the nine moves is either
    // a hazard the model never had or a box the model put somewhere the game did not.
    let min = unsafe { position(at(PLAYER) + player::BOX_MIN) };
    let max = unsafe { position(at(PLAYER) + player::BOX_MAX) };
    crate::detail!(
        "autoplay: the box the game tests is {:.1},{:.1}..{:.1},{:.1}, middle {:.1},{:.1}, against \
         the {:.1},{:.1} the model reads",
        min.0,
        min.1,
        max.0,
        max.1,
        (min.0 + max.0) / 2.0,
        (min.1 + max.1) / 2.0,
        player.0,
        player.1,
    );
    // **And the word the run acted on, which is the game's own and not the one the rules chose.** A
    // word orb writes reaches the run through this game's keys and its own read of them, so a move
    // taken in a direction nothing was pressed in is a word that did not arrive. The dword after it is
    // the frame before — the frame a hit happened on, this dump being written on the one after it.
    crate::detail!(
        "autoplay: the word the run acts on is {:#05x}, and {:#05x} on the frame before it",
        unsafe { mem::read::<u32>(at(INPUT_THE_RUN_ACTS_ON)) },
        unsafe { mem::read::<u32>(at(INPUT_THE_RUN_ACTS_ON) + size_of::<u32>()) },
    );
    // **And every laser that is in use, with the fields a lane is worked out of.** Which of them hit is
    // the question a room of `4.5` and a nearest bullet `15.1` away leaves open: the reach across is
    // measured rather than read — see [`laser::HALF_OF_ITS_WIDTH`] — so the numbers it is measured from
    // belong in the dump beside it.
    for slot in 0..laser::SLOTS {
        let beam = at(LASERS) + slot * laser::BYTES;
        if unsafe { mem::read::<u8>(beam + laser::IN_USE) } == 0 {
            continue;
        }
        let origin = unsafe { position(beam + laser::ORIGIN) };
        crate::detail!(
            "autoplay: laser {slot} from {:.1},{:.1} at {:.3} rad, {:.1} wide, {:.1}..{:.1} along \
             growing {:.1}, phase {} on frame {} of it, hitting from {}",
            origin.0,
            origin.1,
            unsafe { mem::read::<f32>(beam + laser::ANGLE) },
            unsafe { mem::read::<f32>(beam + laser::WIDTH) },
            unsafe { mem::read::<f32>(beam + laser::NEAR) },
            unsafe { mem::read::<f32>(beam + laser::FAR) },
            unsafe { mem::read::<f32>(beam + laser::GROWS) },
            unsafe { mem::read::<u8>(beam + laser::PHASE) },
            unsafe { mem::read::<i32>(beam + laser::FRAMES) },
            unsafe { mem::read::<i32>(beam + laser::HITS_FROM) },
        );
    }
    // **And the four speeds a move is projected with, as they are read.** A step projected twice the
    // length the game takes is a dodge that does not arrive, and which of the four pairs a frame uses
    // is the focus bit in the word above.
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
        // **A slot of nought is looked at too, which is the whole point of looking now.** The game puts
        // the bullet that hit to spent the moment its test comes back, so the one thing this dump is for
        // is in a slot the model no longer has — every hit of several runs was reported with nothing
        // within sixteen of the player and the bullets it *did* name clear of it on one axis each.
        //
        // What a spent slot's own position and size still hold is where that bullet was, which is the
        // reading a hit has to be held against; its step means nothing once it is spent.
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
    // **And the nearest of them whatever the reach**, which is the one thing a dump that answered
    // nothing can still be asked: a nearest bullet twenty units off is a model whose boxes are too
    // small, and one two hundred off is something that is in no array at all.
    match nearest {
        Some((slot, place, apart, state)) => {
            crate::detail!(
                "autoplay: the nearest bullet is {slot}, {apart:.1} off at {:.1},{:.1} state {state}",
                place.0,
                place.1,
            );
            // **And its own first bytes, read as floats**, which is the one thing a dump of what was near
            // cannot be replaced by: every offset this module has for a bullet was read off the exe, and a
            // field that reads as nought while the bullet plainly moves is a field read in the wrong place.
            // Measured: six hits of one launch each named a bullet `stepping 0.0,0.0`.
            let bullet =
                at(BULLET_MANAGER) + bullet_manager::BULLETS + slot * bullet_manager::BYTES;
            let floats = (0..24)
                .map(|word| {
                    let of = bullet + word * size_of::<f32>();
                    format!("{:.2}", unsafe { mem::read::<f32>(of) })
                })
                .collect::<Vec<_>>()
                .join(" ");
            crate::detail!("autoplay: bullet {slot}'s own first 24 words are {floats}");
        }
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
