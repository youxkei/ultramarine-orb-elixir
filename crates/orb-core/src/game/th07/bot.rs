//! **What 妖々夢 answers a bot with**: its own names for its buttons, and the word one frame of a stage takes.
//!
//! Its own file and this game's own, for the reason each game's resume file is its own: which bit of a word is
//! this game's shot, where its player is, and what its own state byte says has just happened are questions
//! about *this* game. A shared answer could only be the union of every game's vocabulary, and 東方 is not one
//! kind of game to take a union of.
//!
//! **And nothing of a recipe is here**, `orb-bot` walking those: a bot is a harness rather than any part of
//! what orb does for somebody playing, so its own machinery is a crate outside the logic that ships. What is
//! left here is the two answers that need this game's addresses, which is what the game seam is for.
//!
//! **The rules are Normal's**, and dodging comes before all of them:
//!
//! - the shot held, let go for one frame in every [`SHOT_LET_GO_EVERY`] so that a screen waiting for a press
//!   gets one, and [`BUTTON_SKIP`] while a dialogue is up
//! - the bomb the frame the player is hit, which is 決死結界 — 0x440cf0 counts a window down while
//!   [`super::PLAYER_STATE`] stands at 2, and a bomb inside it is a death taken back
//! - **the nearest to where it wants to be, of the nine moves nothing will be at.** Holding each one is
//!   followed [`LOOK_AHEAD`] frames out, against every bullet's own step and every enemy where it stands, and a
//!   move whose path comes within [`HIT`] of one of them is not taken. An enemy is in that because its body is
//!   as solid as a bullet in this game, and because it is what a run on its way to an item walks into — the item
//!   was dropped by something, and that something is still standing over it
//! - and with nothing near, whichever item is worth most — a life, a bomb, the big power item, the small one,
//!   the point item, the cherry — or the boss's own x with the y low, or the middle of the field low
//! - **all of it inside [`THE_BAND`]**, the bottom third of the field, dodges included
//!
//! **Safety filters the choice rather than replacing it**, which is the third shape this took and the one the
//! others were wrong for. Fleeing — the move furthest from everything — is a bot that spends the stage leaving,
//! the screen of a 妖々夢 stage never being empty; and reading the bullets only once one is *near* is a bot that
//! walks into one on its way to an item, which two launches did. What the next frames' positions are for is
//! knowing which moves are open, and among those the one that gets on with it.
//!
//! Nothing here presses for a 雑魚: the shot is held on every frame already, and what a killed one drops falls
//! to whoever is under it.
//!
//! **Which is not a claim that they clear the game.** Nothing laid out by hand can say that; a launch says it,
//! and what a launch that dies says is which rule to look at. So the rules go in one at a time, each with the
//! e2e test that pins it — `orb-e2e`'s `the_bot_playing_th07`.

use std::sync::atomic::{AtomicU32, Ordering};

use orb_api::mem;

use crate::log;

use super::{
    BULLET_ALIVE, BULLET_BYTES, BULLET_POSITION, BULLET_SIZE, BULLET_SLOTS, BULLET_STEP, BULLETS,
    BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP, ENEMIES, ENEMY_ALIVE, ENEMY_ALIVE_BIT,
    ENEMY_BOX_OF_ITS_SIZE, ENEMY_BYTES, ENEMY_POSITION, ENEMY_SIZE, ENEMY_SLOTS, FIELD_HEIGHT,
    FIELD_LEFT, FIELD_TOP, FIELD_WIDTH, G_BULLET_MANAGER, G_ENEMY_MANAGER, G_GAME_MANAGER,
    G_ITEM_MANAGER, G_LASERS, G_MOTION_SCALE, G_PLAYER, G_SUPERVISOR, GAME_MENU_UP, ITEM_ALIVE,
    ITEM_BIG_POWER, ITEM_BOMB, ITEM_BYTES, ITEM_CHERRY, ITEM_KIND, ITEM_LIFE, ITEM_POINT,
    ITEM_POSITION, ITEM_POWER, ITEM_SLOTS, LASER_ALIVE, LASER_ANGLE, LASER_BYTES, LASER_FADING,
    LASER_FAR, LASER_NEAR, LASER_ORIGIN, LASER_PHASE, LASER_SLOTS, LASER_WIDTH, MENU_CANCEL,
    MENU_DECIDE, PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_BOX_MAX, PLAYER_BOX_MIN, PLAYER_HIT,
    PLAYER_POSITION, PLAYER_SPEED_SCALE, PLAYER_SPEEDS, PLAYER_STATE, SCENE_RUN, SPEED_DIAGONAL,
    SPEED_STRAIGHT, SUPERVISOR_SCENE, Th07,
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

/// How far off the field's bottom edge the player waits with nothing to go to, in the game's own units.
///
/// **A rule's own number and not a measurement of anything.** Against the edge outright is where the clamp at
/// 0x43f474 holds the player with the whole of one side taken away from it, and this much up leaves the room a
/// dodge needs on every side of it.
const ABOVE_THE_BOTTOM: f32 = 48.0;

/// **How much of the field, measured up from its bottom edge, the bot moves in at all** — a third of it.
///
/// A bound and not a preference, and it is the third thing tried: three launches in a row went up the field for
/// something and stayed up, and each time the reason was a different one — an item's own place, then a screen
/// worth sweeping at full power, then the next item always being above the last. What a band gives up is
/// 妖々夢's collection at the top of the field, which needs the whole climb; what it buys is a run that is
/// never up among the bullets for anything, and one thing to change rather than a reason to chase each time.
const THE_BAND: f32 = 1.0 / 3.0;

// **Nothing here is a hitbox of its own**, which two launches paid for: one number standing in for every
// bullet's size was four, and hits came back saying the move they were taken on had 2.1 and 3.7 units of room —
// so no single number was ever going to be it. What a hit is is the game's own test, two boxes overlapping, and
// both boxes are read: `[player+0x948]`..`+0x954` and each bullet's own `+0xb7c`.

/// How far a candidate move is followed for one frame where the game's own step cannot be read at all, a bot
/// that stood still being worse than one that moved by a guess.
const A_PROBE: f32 = 8.0;

/// How many frames of holding a move are looked at before it is chosen.
///
/// One is not enough: a bullet a frame away is one there is nowhere left to be for, and what a move has to
/// answer is where holding it *leads*. Three is far enough to leave in and near enough that a bullet's own step
/// still describes where it goes — a bullet turns, and nothing here reads what turns it.
const LOOK_AHEAD: usize = 3;

/// **What a move came to**: which of the frames looked at it is hit on, and how near it got to anything.
///
/// The frames as a set of bits rather than a count, so that two bullets reaching the same frame is one frame
/// hit and not two — what a move is ranked by is how many frames of it are lost, and the first one lost.
#[derive(Clone, Copy)]
struct Judged {
    on: u32,
    near: f32,
}

impl Judged {
    const CLEAR: Self = Self {
        on: 0,
        near: f32::INFINITY,
    };

    fn hit(&mut self, frame: usize) {
        self.on |= 1 << frame;
    }

    fn clear(self) -> bool {
        self.on == 0
    }

    fn hits(self) -> u32 {
        self.on.count_ones()
    }

    /// The first frame it is hit on, and past every frame looked at for one that is not hit at all.
    fn first(self) -> u32 {
        if self.on == 0 {
            u32::MAX
        } else {
            self.on.trailing_zeros()
        }
    }
}

/// The nine one-frame moves, as the word each is and the direction it goes in.
///
/// **Standing still is first**, which is what settles a tie: where two moves are equally safe the bot holds its
/// ground rather than stepping back and forth between them every frame.
/// **The direction is which way and not how far** — how far is the game's, and a diagonal's own share of a step
/// is its own number there: 0x43f165 loads the speed's `+0x2c` where a straight move takes `+0x24`.
const MOVES: [(u16, f32, f32); 9] = [
    (0, 0.0, 0.0),
    (PAD_UP, 0.0, -1.0),
    (PAD_DOWN, 0.0, 1.0),
    (PAD_LEFT, -1.0, 0.0),
    (PAD_RIGHT, 1.0, 0.0),
    (PAD_UP | PAD_LEFT, -1.0, -1.0),
    (PAD_UP | PAD_RIGHT, 1.0, -1.0),
    (PAD_DOWN | PAD_LEFT, -1.0, 1.0),
    (PAD_DOWN | PAD_RIGHT, 1.0, 1.0),
];

/// How often the shot is let go for a frame, so that a screen waiting for one gets a press: a second of them.
const SHOT_LET_GO_EVERY: u32 = 60;

/// The frames these rules have been asked for, which is what [`SHOT_LET_GO_EVERY`] is counted in.
///
/// A clock of its own rather than the stage's, because the stage's stops: `STAGE_FRAMES` does not move under the
/// pause menu and it is the clear screen's own wait that this is for.
static FRAMES: AtomicU32 = AtomicU32::new(0);

/// Whether the player was hit last frame, so that a hit is said once rather than once a frame of the window it
/// opened.
static WAS_HIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// **The angle each laser slot had last frame**, which is the only way to know how fast one is turning: nothing
/// in a laser's own update moves its angle — 0x424f6e writes it where one is made and its boss's script moves it
/// after that — so there is no rate to read, only a difference to keep.
///
/// A quiet NaN for a slot that was not a laser last frame, so that one just made is not given the turn of
/// whatever used the slot before it.
static LAST_ANGLE: [AtomicU32; LASER_SLOTS] = [const { AtomicU32::new(NO_ANGLE) }; LASER_SLOTS];
const NO_ANGLE: u32 = 0x7fc0_0000;

/// How far a laser's beam turned since last frame, and nothing for one that was not there then.
///
/// Wrapped into half a turn either way, an angle being a thing that comes back round: the difference across
/// that seam is a whole turn of nonsense and the beam did not move at all.
fn turned(slot: usize, angle: f32) -> f32 {
    let was = f32::from_bits(LAST_ANGLE[slot].swap(angle.to_bits(), Ordering::Relaxed));
    if was.is_nan() {
        return 0.0;
    }
    let turn = angle - was;
    let round = std::f32::consts::TAU;
    turn - round * (turn / round).round()
}

/// Says a slot holds no laser, so the next one to use it starts without a turn.
fn turns_no_more(slot: usize) {
    LAST_ANGLE[slot].store(NO_ANGLE, Ordering::Relaxed);
}

// **Nothing here is a dead zone round where it wants to be**, which the choice makes unnecessary: standing
// still is one of the [`MOVES`] and the one a tie goes to, so the frame the player is there is a frame nothing
// is pressed for.

/// The word the rules press this frame.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn word_to_press() -> u16 {
    // **Nothing at all off a stage, the shot included.** The rules are asked for a word by the game's own
    // keyboard read, which every screen makes — and a menu of this game's is taken by the shot as much as by
    // the return key, so a word held on the title screen walks that menu once and then holds it, while
    // 十字キー there moves a cursor these rules cannot read. The front end is the recipe's to drive, one press
    // to a screen; a rule pressing alongside it is a press nobody wrote.
    if !unsafe { in_a_stage() } {
        return 0;
    }
    // **The shot held, and let go for one frame in every [`SHOT_LET_GO_EVERY`].** A stage's own clear screen
    // goes on to the next stage the frame the shot goes *down* and otherwise waits out its own timer, and a run
    // that has held the shot since the stage began never gives it that frame: one launch spent 60 seconds
    // between 1面's last chapter and 2面's first. A shot in sixty is what that costs, 妖々夢's own shot firing
    // off the player's timer rather than off a press.
    let frame = FRAMES.fetch_add(1, Ordering::Relaxed);
    let mut word = if frame.is_multiple_of(SHOT_LET_GO_EVERY) {
        0
    } else {
        BUTTON_SHOT
    };
    // **The frame the player is hit and not the frames after it.** 0x43edc0 leaves the state at 2 and zeroes
    // the timer, and 0x440cf0 counts `[player+0x23f8]` down from there while the state stands — a window this
    // presses into rather than a single frame, and one whose end takes the life.
    if unsafe { mem::read::<i8>(G_PLAYER + PLAYER_STATE) } == PLAYER_HIT {
        word |= BUTTON_BOMB;
    }
    // A dialogue read at the game's own pace is the afternoon this was written to save.
    if unsafe { Th07::dialogue_line() }.is_some() {
        word |= BUTTON_SKIP;
    }
    let (steer, dodging, room) = unsafe { steer() };
    // **What the rules thought of the move a hit was taken on.** A frame ahead is computed and a move nothing
    // will be at is chosen, so a run that is hit at all means one of three things and no others: something on
    // the screen is not in the model, or every one of the nine was shut, or the move chosen was not the move
    // made. Which of them it was is the one thing a launch can say and nothing here can.
    // Said once for the hit and not once for every frame of the window it opened: 決死結界 stands for as long as
    // the game counts it down, and a line a frame turns one hit into a page of them.
    if WAS_HIT.swap(word & BUTTON_BOMB != 0, Ordering::Relaxed) != (word & BUTTON_BOMB != 0)
        && word & BUTTON_BOMB != 0
    {
        match room {
            Some(room) => log!("autoplay: hit while pressing {steer:#05x}, with {room:.1} of room"),
            None => log!("autoplay: hit while pressing {steer:#05x}, with nowhere open to it"),
        }
    }
    // **低速移動 under a boss**, which is where this game's shots do their damage — and off again the moment
    // anything could reach the player at all, the same key halving the step a dodge is made of.
    if !dodging && unsafe { Th07::boss() }.is_some() {
        word |= BUTTON_FOCUS;
    }
    word | steer
}

/// **The 十字キー of the safest of the nine one-frame moves**, or the one towards where the rules want to be
/// where nothing is near enough to dodge.
///
/// The safety of a move is how near the nearest bullet gets to where that move lands — so the choice covers
/// every direction at once, and up and down are dodges as much as left and right. Two regimes and not a blend,
/// because a blend is a bot that walks into a bullet for an item: inside [`DANGER`] the frame is the dodge, and
/// outside it the bullets are not read at all.
///
/// **Lasers are not in this.** They are their own manager's slots — [`super::G_LASERS`] — and a length and an
/// angle rather than a point, so nothing here would make them a distance; a launch that dies on one is what
/// says whether that matters.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn steer() -> (u16, bool, Option<f32>) {
    let at = unsafe { player() };
    let field = unsafe { field() };
    let wanted = unsafe { where_to_be(field) };
    // **Where holding each move leads over the next [`LOOK_AHEAD`] frames**, held inside the field the way the
    // clamp at 0x43f416 would hold it: a move into a wall is a move that does not happen, and scoring it as
    // though it did would pick it. Not held inside [`THE_BAND`] — that is where it *wants* to be, and a probe
    // clamped into the band is a player above the band with nine moves that all land in the same place.
    //
    // More than the next frame, because one frame of warning is not enough to be anywhere: a bullet three
    // frames off where the player stands is one to be somewhere else for, and it is *this* frame's move that
    // gets it there.
    let (straight, diagonal) = unsafe { a_step() };
    let lands = MOVES.map(|(_, dx, dy)| {
        let step = if dx != 0.0 && dy != 0.0 {
            diagonal
        } else {
            straight
        };
        std::array::from_fn::<_, LOOK_AHEAD, _>(|frame| {
            let far = step * (frame + 1) as f32;
            (
                (at.0 + dx * far).clamp(field.0, field.0 + field.2),
                (at.1 + dy * far).clamp(field.1, field.1 + field.3),
            )
        })
    });
    // **Two boxes overlapping, which is the game's own test and not a distance**: 0x43e294 makes a box of a
    // bullet's place and its own size and compares it with the player's, and no two kinds of bullet are the
    // same size — so one number standing in for every one of them is a different test rather than an
    // approximation of that one. What each move is judged by is therefore *which frame it is first hit on*, and
    // the room left over is only for the log and for choosing between moves that are all hit.
    // **How many of the frames it is hit on and which is the first**, rather than whether any of them is: a
    // player already inside something is hit on the first frame whatever it presses, so a move that has to be
    // clear all the way is a bot with nothing open to it — which falls through to the most room, and the most
    // room is the fleeing all this exists instead of. Fewest hit is a move that gets out; latest first is a move
    // that has longer before it has to.
    let (mine_x, mine_y) = unsafe { player_box() };
    let mut judged = [Judged::CLEAR; MOVES.len()];
    let mut against = |(hx, hy): (f32, f32), (sx, sy): (f32, f32), (wx, wy): (f32, f32)| {
        for (judged, path) in judged.iter_mut().zip(&lands) {
            for (frame, (x, y)) in path.iter().enumerate() {
                let ahead = (frame + 1) as f32;
                let (dx, dy) = ((x - (hx + sx * ahead)).abs(), (y - (hy + sy * ahead)).abs());
                judged.near = judged.near.min(dx * dx + dy * dy);
                if dx < mine_x + wx && dy < mine_y + wy {
                    judged.hit(frame);
                }
            }
        }
    };
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        if unsafe { mem::read::<u16>(bullet + BULLET_ALIVE) } == 0 {
            continue;
        }
        // Its own update adds its step to its position every frame, so where it will be in a frame's time is
        // where it is plus as many steps as there are frames to look — and its own half, which is half of the
        // size the hit test is given.
        let (wx, wy) = unsafe { position(bullet + BULLET_SIZE) };
        against(
            unsafe { position(bullet + BULLET_POSITION) },
            unsafe { position(bullet + BULLET_STEP) },
            (wx / 2.0, wy / 2.0),
        );
    }
    // **And the enemies, whose bodies are as solid as a bullet in this game.** Which is what a run on its way
    // to an item walks into: the item was dropped by something, and that something is still standing over it.
    //
    // **With its own body**, a third of `+0x2b3c` on each side, which is what 0x420490 and the box test it calls
    // make of that size between them: a boss taken for a point is a boss the model says there is room under
    // while its body reaches. Where it is and not where it is going, an enemy being moved by its own script
    // rather than by a step of its own that could be read.
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES + slot * ENEMY_BYTES;
        if unsafe { mem::read::<u8>(enemy + ENEMY_ALIVE) } & ENEMY_ALIVE_BIT == 0 {
            continue;
        }
        let (wx, wy) = unsafe { position(enemy + ENEMY_SIZE) };
        against(
            unsafe { position(enemy + ENEMY_POSITION) },
            (0.0, 0.0),
            (
                wx / ENEMY_BOX_OF_ITS_SIZE / 2.0,
                wy / ENEMY_BOX_OF_ITS_SIZE / 2.0,
            ),
        );
    }
    // **And the lasers, which are the one hazard a distance cannot stand in for**: a beam is a place, not a
    // point, so a move is judged inside the beam's own frame — turn the landing about the origin by the angle
    // and it is *along* and *across*, which is the shape the game's own test at 0x43e6b0 makes of it.
    //
    // The width it has *now*, which is what makes the warning line a hazard the size it looks: a beam about to
    // fire is thin and the one firing is not. And the angle it has now, which a turning beam will not have in
    // the frames looked at — nothing here reads what turns one, so a card of those is a card this understates.
    for slot in 0..LASER_SLOTS {
        let laser = G_LASERS + slot * LASER_BYTES;
        if unsafe { mem::read::<u32>(laser + LASER_ALIVE) } == 0
            || unsafe { mem::read::<u8>(laser + LASER_PHASE) } >= LASER_FADING
        {
            turns_no_more(slot);
            continue;
        }
        let (ox, oy) = unsafe { position(laser + LASER_ORIGIN) };
        let angle = unsafe { mem::read::<f32>(laser + LASER_ANGLE) };
        let turn = turned(slot, angle);
        let (near, far) = (unsafe { mem::read::<f32>(laser + LASER_NEAR) }, unsafe {
            mem::read::<f32>(laser + LASER_FAR)
        });
        let across = unsafe { mem::read::<f32>(laser + LASER_WIDTH) } / 2.0;
        for (judged, path) in judged.iter_mut().zip(&lands) {
            for (frame, (x, y)) in path.iter().enumerate() {
                // Where the beam will be pointing by then, which is where it points now plus as many of its own
                // turns as there are frames to look.
                let ahead = angle + turn * (frame + 1) as f32;
                let (cos, sin) = (ahead.cos(), ahead.sin());
                let (dx, dy) = (x - ox, y - oy);
                let along = dx * cos + dy * sin;
                let off = (-dx * sin + dy * cos).abs();
                if along > near - mine_x && along < far + mine_x && off < across + mine_y {
                    judged.hit(frame);
                }
            }
        }
    }
    // **The nearest to where it wants to be, of the moves nothing will be at.** Which is not the same as the
    // one furthest from everything: the next frame's positions are known, so what a move has to be is *not
    // where a bullet lands* — and keeping away from bullets besides is a bot that spends the stage leaving,
    // since the screen of a 妖々夢 stage is never empty and being near one is the ordinary state of playing.
    let mut best: Option<(u16, Judged, f32)> = None;
    for (((word, _, _), judged), land) in MOVES.iter().zip(judged).zip(lands) {
        // How near this frame's own step gets to where it wants to be, which is the first of the path: the
        // frames after it are what the move is judged *hit* by and not where it is going.
        let (x, y) = land[0];
        let towards = (x - wanted.0).powi(2) + (y - wanted.1).powi(2);
        // Clear first, then nearest to where it wants to be. And where nothing is clear: **fewest frames hit,
        // then the latest first hit, then the most room** — a move that gets out over a move that has longer
        // before it has to, and either over a move that merely has space around it, which is the fleeing all
        // this exists instead of. Strictly better only, so a tie goes to the earlier of [`MOVES`]: standing
        // still is first, and a bot that took the last of a tie would step between two of them every frame.
        let better = match best {
            None => true,
            Some((_, was, was_towards)) => match (was.clear(), judged.clear()) {
                (false, true) => true,
                (true, false) => false,
                (true, true) => towards < was_towards,
                (false, false) => {
                    judged.hits() < was.hits()
                        || (judged.hits() == was.hits()
                            && (judged.first() > was.first()
                                || (judged.first() == was.first() && judged.near > was.near)))
                }
            },
        };
        if better {
            best = Some((*word, judged, towards));
        }
    }
    // **What the 低速移動 under a boss is off for is anything that could reach the player at all**, and not
    // only what would hit it this frame: slowing down halves the step a dodge is made of, so by the frame a
    // bullet is a hit the halving has already been paid for. The reach is as far as anything gets in the frames
    // looked at, so nothing here is a margin of its own.
    let reach = straight * LOOK_AHEAD as f32;
    let (word, judged_it, _) = best.unwrap_or((0, Judged::CLEAR, 0.0));
    (
        word,
        !judged_it.clear() || judged[0].near <= reach * reach,
        judged_it.clear().then(|| judged_it.near.sqrt()),
    )
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

/// Where the rules want the player: the x whatever is worth being under has, and the y always low.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn where_to_be(field: (f32, f32, f32, f32)) -> (f32, f32) {
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    // The item worth going for, at the place it is; under a boss with nothing to collect the boss's own x with
    // the y low, a fight being a thing to stand under rather than to reach; and with neither the middle, low,
    // which is the shortest way to wherever the next item falls.
    let (x, y) = match unsafe { the_item_worth_going_for() } {
        Some((_, at)) => at,
        None => match unsafe { Th07::boss() } {
            Some(boss) => (unsafe { mem::read::<f32>(boss + ENEMY_POSITION) }, low),
            None => (field.0 + field.2 / 2.0, low),
        },
    };
    // **Inside the band or waited for at the low line**, and not held at the band's own top edge: an item is
    // almost always above, so clamping its y into the band put the bot to live on that edge — the middle of the
    // field, moving only sideways, with the dodge it wanted to make upwards pulled straight back. Waiting low is
    // where the item arrives anyway.
    (x, if y >= the_band(field) { y } else { low })
}

/// The top of the band the bot moves in, which is the bottom [`THE_BAND`] of the field.
fn the_band(field: (f32, f32, f32, f32)) -> f32 {
    field.1 + field.3 * (1.0 - THE_BAND)
}

/// **What one frame of holding a direction moves the player by**, straight and diagonal, as the game works it
/// out at 0x43f0f5 and 0x43f378: the speed off the struct the player points at, times the player's own scale,
/// times [`G_MOTION_SCALE`].
///
/// [`A_PROBE`] where any of it cannot be read or comes out as no step at all — a bot that stood still would be
/// worse than one that moved by a guess, and outside a run there is nothing there to read.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn a_step() -> (f32, f32) {
    let scale = mem::read_committed::<f32>(G_PLAYER + PLAYER_SPEED_SCALE)
        .zip(mem::read_committed::<f32>(G_MOTION_SCALE))
        .map(|(own, motion)| own * motion);
    let speeds = mem::read_committed::<usize>(G_PLAYER + PLAYER_SPEEDS).filter(|at| *at != 0);
    let step = |of: usize| {
        speeds
            .zip(scale)
            .and_then(|(speeds, scale)| {
                mem::read_committed::<f32>(speeds + of).map(|speed| speed * scale)
            })
            .filter(|step| step.is_finite() && *step > 0.0)
            .unwrap_or(A_PROBE)
    };
    (step(SPEED_STRAIGHT), step(SPEED_DIAGONAL))
}

/// **What each kind of item is worth going for**, largest first — a life, a bomb, the big power item, the small
/// one, the point item, the cherry, and last anything the six of them do not name.
///
/// Which is 妖々夢's own worth of them: a life is a life, a bomb is a life this bot's 決死結界 will spend, and
/// [`ITEM_BIG_POWER`] is eight steps of power against [`ITEM_POWER`]'s one. **Nothing scores zero**, so no item
/// on the screen is walked past — a kind with no place of its own is still worth the step it costs.
const fn worth(kind: i8) -> u8 {
    match kind {
        ITEM_LIFE => 6,
        ITEM_BOMB => 5,
        ITEM_BIG_POWER => 4,
        ITEM_POWER => 3,
        ITEM_POINT => 2,
        ITEM_CHERRY => 1,
        _ => 0,
    }
}

/// The item worth going for, or `None` where the screen is empty.
///
/// **The worth first and the distance only inside it**, which is what makes a big power item across the field
/// beat a small one under the player's hand. Between two of a kind it is the nearer.
///
/// **Every slot walked and not the live count taken**, because the count says how many there are and not
/// which slots they are in: the manager leaves a collected item's own numbers where they were and clears
/// [`ITEM_ALIVE`], which is the same walk 0x432a3f makes.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_item_worth_going_for() -> Option<(u8, (f32, f32))> {
    let (at_x, at_y) = unsafe { player() };
    (0..ITEM_SLOTS)
        .map(|slot| G_ITEM_MANAGER + slot * ITEM_BYTES)
        .filter(|item| unsafe { mem::read::<i8>(item + ITEM_ALIVE) } != 0)
        .map(|item| {
            let worth = worth(unsafe { mem::read::<i8>(item + ITEM_KIND) });
            let at = unsafe { position(item + ITEM_POSITION) };
            (worth, (at.0 - at_x).powi(2) + (at.1 - at_y).powi(2), at)
        })
        .max_by(|(ours, ourm, _), (theirs, theirm, _)| {
            ours.cmp(theirs).then(theirm.total_cmp(ourm))
        })
        .map(|(worth, _, at)| (worth, at))
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
