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
    MENU_DECIDE, PAD_DOWN, PAD_LEFT, PAD_RIGHT, PAD_UP, PLAYER_BOX_MAX, PLAYER_BOX_MIN, PLAYER_HIT,
    PLAYER_POSITION, PLAYER_SCALE_ACROSS, PLAYER_SCALE_DOWN, PLAYER_SPEEDS, PLAYER_STATE,
    SCENE_RUN, SPEED_DIAGONAL, SPEED_STRAIGHT, SUPERVISOR_SCENE, Th07,
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

/// And how far in from either side edge the x the rules want is held, in the same units and for the same
/// reason: a corner is where a dodge runs out of moves, three of the nine leading into the wall where the clamp
/// makes them the move the player is already making.
///
/// **Which a run paid for.** 妖々夢's 4面 has three bosses and one of them stands at the left edge; a run that
/// followed its x was hit at `27.8,432.0` with nowhere open to it — every one of the nine shut, in the corner it
/// had been led into.
///
/// Only where the bot *waits*, the same as [`THE_BAND`]: a dodge goes wherever the field allows.
const IN_FROM_THE_SIDE: f32 = 48.0;

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

/// **How much a move has to clear a hazard's box by before it counts as clearing it at all.**
///
/// Not a hitbox and not a distance standing in for one: the box test is the game's own and both boxes are read.
/// This is slack on top of it, and the launches are what set it. One stage produced twelve hits reading `0.6`,
/// `1.1`, `0.6`, `0.5` and more of the same — every one of them a hazard the model had, at a size the model
/// read, with the player put past it by less than a unit. So something in the projection is out by about a unit
/// and no reading has yet said what: the step is the game's own number times the player's own scale times
/// [`G_MOTION_SCALE`], any of which can move between the read and the frame, and a bullet's own script can
/// re-aim it mid-flight. Until a reading names it, room under a unit is not room.
///
/// **The game's own test agrees**: 0x43e2e7 compares the edges inclusively where this one is strict, so nought
/// of room is already a hit there and never was room at all.
///
/// Two, which is more than the largest margin a hit has been taken on and half of one unfocused step — so a
/// lane the player can actually use is still a lane.
const ROOM_TO_SPARE: f32 = 2.0;

/// How many frames of holding a move are looked at before it is chosen.
///
/// One is not enough: a bullet a frame away is one there is nowhere left to be for, and what a move has to
/// answer is where holding it *leads*.
///
/// **And the number is set by the widest bullet in the game.** To be out from under one the player has to have
/// travelled that bullet's half and its own on top, which is `(12 + 2) / a step` frames — four at the step
/// 0x43f0f5 works out and eight at the halved one 低速移動 leaves, 妖々夢's widest bullet being 24 across
/// (0x4275ca writes it). A move first seen three frames out is therefore one there is no getting away from
/// whatever is pressed, and 自機狙い is the shot that makes it: a launch spent 6面's boss dying to the same
/// aimed 大玉 over and over, 30 hits at one chapter. Twelve is those eight and room to pick a direction in
/// rather than having to move the frame the bullet appears; past that a bullet's own step stops describing
/// where it goes, and nothing here reads what turns one.
const LOOK_AHEAD: usize = 12;

/// **What the rules made of a frame**: the 十字キー to press, whether the step must not be halved, the room the
/// move chosen was left with where it was clear, and whether it is a move towards something to collect rather
/// than towards the fight.
///
/// With the player's own place and what came nearest to the move chosen, which are what a hit is read against:
/// a line saying only how much room there was says the model was wrong and not which part of it.
#[derive(Clone, Copy)]
struct Steered {
    word: u16,
    dodging: bool,
    room: Option<f32>,
    for_an_item: bool,
    at: (f32, f32),
    nearest: Option<Nearest>,
    counted: Counted,
    held: [Option<Held>; HELD],
    /// How many were within reach, which is not how many of them fit in [`Steered::held`].
    held_all: u32,
}

/// **What the frame's model was made of, and what it left out** — bullets, enemies and lasers, each as the
/// number judged against and the number passed over for not being able to hit anything.
///
/// Which is the one thing a hit saying `14.5 of room` needs and no other field of [`Steered`] can give: a model
/// that says a move is clear by fourteen units and is wrong is a model missing a hazard, and the three passing
/// over rules are where a missing hazard would have gone. A run cleared the game with eight hits on 6面's boss
/// reading between six and fourteen of room, every one of them after those three rules went in.
#[derive(Clone, Copy, Default)]
struct Counted {
    bullets: u32,
    /// Of those, how many were in one of the four states the collision was not found in — judged all the same,
    /// for the reason [`super::BULLET_STATE`] gives.
    another_state: u32,
    enemies: u32,
    /// And how many were passed over for having no body to run into, which is the one rule here that leaves a
    /// hazard out — with the nearest of them to the player, and how far that was, for a hit to name.
    no_body: u32,
    no_body_at: Option<((f32, f32), f32)>,
    lasers: u32,
    /// Of those, how many had not started hitting yet — a 予告線, judged for where its beam will be.
    warnings: u32,
    /// And how many slots held a laser that was passed over for its phase, which is the one gate here taken
    /// from a walk whose purpose was inferred rather than read: a card of beams that shows nought of them is
    /// this number standing where the lasers should be.
    phase_over: u32,
    /// And how many bullets no step of their own explains, with the x they are spread about where those are
    /// enough to be a beam a script is sweeping — see [`a_sweep`].
    swept: u32,
    sweeping: Option<f32>,
    /// And how many boxes the enemies' own trails put in, which is the beams that follow them — see
    /// [`the_trail`].
    trail: u32,
}

/// **What came nearest to a move**, as much of it as a log line can carry: which kind of hazard, where it was
/// read, where it was going, and the half of its own box the test is given.
///
/// The kind by name rather than by number, a line being read by whoever is looking at why a run was hit.
#[derive(Clone, Copy)]
struct Nearest {
    what: &'static str,
    at: (f32, f32),
    step: (f32, f32),
    half: (f32, f32),
}

/// **What the model was holding within reach of the player on the frame it chose**, kept so that a hit can be
/// read against it.
///
/// The arrays cannot be asked this afterwards. A hit is noticed a frame or two on — the state it is noticed by
/// is written inside the game's own update, which runs after orb reads the input — and the thing that did it has
/// been taken off them by then: a bullet's own state goes to 5 the moment the test comes back. So a walk after
/// the fact says what is there now, and a hit whose cause has already gone reads as a hit with nothing near it,
/// which is the one answer that says nothing. Held here, an empty list means the model never had it.
///
/// A fixed room and not a list that grows: [`Steered`] is copied out of a lock on every frame, and a frame of
/// 妖々夢's own bullet hell would put a hundred of these in it.
const HELD: usize = 12;

/// One of them, with which slot of which array it came out of — the slot being what makes it findable again in
/// a dump of the same frame.
#[derive(Clone, Copy)]
struct Held {
    what: &'static str,
    slot: usize,
    at: (f32, f32),
    step: (f32, f32),
    half: (f32, f32),
}

/// **What a move came to**: which of the frames looked at it is hit on, and the least room any of them had.
///
/// The frames as a set of bits rather than a count, so that two bullets reaching the same frame is one frame
/// hit and not two — what a move is ranked by is how many frames of it are lost, and the first one lost.
///
/// **The room between the boxes and not between the middles.** A hit is two boxes overlapping, so what says how
/// near a move came to being one is what was left on the tighter of its two axes; a distance between middles
/// puts a 米弾 six away ahead of a 大玉 seventeen away, when it is the 大玉 that is one unit from a hit. Which
/// is not only a number in the log: it is what picks between moves that are all hit, and what the log names as
/// the hazard to look at.
/// **And which of them a beam will only be drawing its warning on**, which is a frame to be elsewhere for and
/// not a frame that is lost. 0x4266dc gates the game's own test on [`super::LASER_TIMER`] reaching
/// [`super::LASER_LIVE_AT`], so a warning cannot hit anything at all — a move into one is worse than a move out
/// of one and better than a move into a beam that is live. Counted apart from [`Judged::on`] because a card that
/// fans twenty-two warnings out of one point shuts every one of the nine where the two are the same, and a bot
/// with nothing open to it stands where it is until the beams fire.
#[derive(Clone, Copy)]
struct Judged {
    on: u32,
    warned: u32,
    room: f32,
}

impl Judged {
    const CLEAR: Self = Self {
        on: 0,
        warned: 0,
        room: f32::INFINITY,
    };

    fn hit(&mut self, frame: usize) {
        self.on |= 1 << frame;
    }

    fn warn(&mut self, frame: usize) {
        self.warned |= 1 << frame;
    }

    fn clear(self) -> bool {
        self.on == 0
    }

    /// Whether nothing will be at it and a warning is drawn across it, which is the middle of the three.
    fn only_warned(self) -> bool {
        self.on == 0 && self.warned != 0
    }

    fn warnings(self) -> u32 {
        self.warned.count_ones()
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

/// **What the rules made of the last frame**, which is what a hit is reported against: the hit happened between
/// that frame's word going in and this one being asked for.
///
/// Behind a lock and not in atomics, there being more of it than a word now: it is written once and read once
/// per frame on the game's own thread, and nothing else takes this lock.
static LAST_FRAME: std::sync::Mutex<Option<Steered>> = std::sync::Mutex::new(None);

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

/// **Where each enemy slot was last frame**, which is the only way to know where one is going: an enemy is
/// moved by its own script and there is no step of its own to read anywhere on it.
///
/// A quiet NaN for a slot that held nothing last frame, so that an enemy just made is not given the motion of
/// whatever stood in that slot before it.
static LAST_PLACE: [(AtomicU32, AtomicU32); ENEMY_SLOTS] =
    [const { (AtomicU32::new(NOWHERE), AtomicU32::new(NOWHERE)) }; ENEMY_SLOTS];
const NOWHERE: u32 = 0x7fc0_0000;

/// **How each enemy was moving last frame**, so that one being swept by a script can be told from one drifting:
/// 6面's turning beam is a line of small solid enemies, size 8 and solidity `0x27`, and the boss's script whirls
/// them. A straight line drawn from where such a thing went last frame is wrong by the whole of the turn, which
/// is a hit reading `6.4 of room` with one of them four units from the player.
static LAST_ENEMY_MOVE: [(AtomicU32, AtomicU32); ENEMY_SLOTS] =
    [const { (AtomicU32::new(NOWHERE), AtomicU32::new(NOWHERE)) }; ENEMY_SLOTS];

/// How much an enemy's own motion changed since last frame, and nothing for one that was not there then.
fn an_enemy_turned(slot: usize, moved: (f32, f32)) -> (f32, f32) {
    let was = (
        f32::from_bits(
            LAST_ENEMY_MOVE[slot]
                .0
                .swap(moved.0.to_bits(), Ordering::Relaxed),
        ),
        f32::from_bits(
            LAST_ENEMY_MOVE[slot]
                .1
                .swap(moved.1.to_bits(), Ordering::Relaxed),
        ),
    );
    if was.0.is_nan() || was.1.is_nan() {
        return (0.0, 0.0);
    }
    (moved.0 - was.0, moved.1 - was.1)
}

/// How far an enemy moved since last frame, and nothing for one that was not there then.
fn moved(slot: usize, (x, y): (f32, f32)) -> (f32, f32) {
    let was = (
        f32::from_bits(LAST_PLACE[slot].0.swap(x.to_bits(), Ordering::Relaxed)),
        f32::from_bits(LAST_PLACE[slot].1.swap(y.to_bits(), Ordering::Relaxed)),
    );
    if was.0.is_nan() || was.1.is_nan() {
        return (0.0, 0.0);
    }
    (x - was.0, y - was.1)
}

/// **Where each bullet slot was last frame**, so that a bullet moved by something other than its own step can
/// be told from one moved by it: 0x425ccd adds `+0xb98` and that is an ordinary bullet's whole motion, so any
/// disagreement is a script.
static LAST_BULLET: [(AtomicU32, AtomicU32); BULLET_SLOTS] =
    [const { (AtomicU32::new(NOWHERE), AtomicU32::new(NOWHERE)) }; BULLET_SLOTS];

/// And the step each held, so that a piece being turned is caught by its step changing even where the turn in
/// any one frame is too small to tell from its own motion.
static LAST_STEP: [(AtomicU32, AtomicU32); BULLET_SLOTS] =
    [const { (AtomicU32::new(NOWHERE), AtomicU32::new(NOWHERE)) }; BULLET_SLOTS];

/// How far what a bullet actually moved may differ from the step it claims, **or its step from the step it had
/// last frame**, before it counts as moved by something other than itself. A hair, an ordinary bullet's movement
/// *being* its step to the bit and its step never changing at all.
///
/// **The step's own change is the half that catches a beam.** Comparing only the movement against the step
/// misses a piece that turns gently: the disagreement in one frame is under any threshold worth having, and the
/// straight line it is then projected along is wrong by the whole of the turn twelve frames out — which is a hit
/// reading `12.2 of room` with nothing else near, and 6面's turning card gave three of them.
const A_STEP_OF_ITS_OWN: f32 = 0.5;

/// What a bullet actually moved since last frame, and its own step for one that was not there then.
fn a_bullet_moved(slot: usize, (x, y): (f32, f32), step: (f32, f32)) -> (f32, f32) {
    let was = (
        f32::from_bits(LAST_BULLET[slot].0.swap(x.to_bits(), Ordering::Relaxed)),
        f32::from_bits(LAST_BULLET[slot].1.swap(y.to_bits(), Ordering::Relaxed)),
    );
    if was.0.is_nan() || was.1.is_nan() {
        return step;
    }
    (x - was.0, y - was.1)
}

/// Says a slot holds no bullet, so the next one to use it is not given the last one's place or step.
fn a_bullet_is_gone(slot: usize) {
    LAST_BULLET[slot].0.store(NOWHERE, Ordering::Relaxed);
    LAST_BULLET[slot].1.store(NOWHERE, Ordering::Relaxed);
    LAST_STEP[slot].0.store(NOWHERE, Ordering::Relaxed);
    LAST_STEP[slot].1.store(NOWHERE, Ordering::Relaxed);
}

/// How much a bullet's step changed since last frame, and nothing for one that was not there then.
fn a_step_turned(slot: usize, step: (f32, f32)) -> (f32, f32) {
    let was = (
        f32::from_bits(LAST_STEP[slot].0.swap(step.0.to_bits(), Ordering::Relaxed)),
        f32::from_bits(LAST_STEP[slot].1.swap(step.1.to_bits(), Ordering::Relaxed)),
    );
    if was.0.is_nan() || was.1.is_nan() {
        return (0.0, 0.0);
    }
    (step.0 - was.0, step.1 - was.1)
}

/// Says a slot holds no enemy, so the next one to use it starts without a motion.
fn moves_no_more(slot: usize) {
    LAST_PLACE[slot].0.store(NOWHERE, Ordering::Relaxed);
    LAST_PLACE[slot].1.store(NOWHERE, Ordering::Relaxed);
    LAST_ENEMY_MOVE[slot].0.store(NOWHERE, Ordering::Relaxed);
    LAST_ENEMY_MOVE[slot].1.store(NOWHERE, Ordering::Relaxed);
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
    let steered = unsafe { steer() };
    // **What the rules thought of the move a hit was taken on — and it is the frame before's that they thought
    // it of.** A frame ahead is computed and a move nothing will be at is chosen, so a run that is hit at all
    // means one of three things and no others: something on the screen is not in the model, or every one of the
    // nine was shut, or the move chosen was not the move made. Which of them it was is the one thing a launch
    // can say and nothing here can.
    //
    // **This frame's judgement is of a screen the hit has already been taken off.** The bullet that did it is
    // gone — 0x425eb7 puts its state to 5 the moment the test comes back — so the room this frame has is the
    // room to some *other* bullet, and the figure means nothing: three launches were read as 20 and 29 and 12.5
    // of room, and every one of those numbers was measured after the thing being measured had been removed.
    // What was decided on the frame the hit came out of is [`LAST_MOVE`].
    //
    // Said once for the hit and not once for every frame of the window it opened: 決死結界 stands for as long as
    // the game counts it down, and a line a frame turns one hit into a page of them.
    if WAS_HIT.swap(word & BUTTON_BOMB != 0, Ordering::Relaxed) != (word & BUTTON_BOMB != 0)
        && word & BUTTON_BOMB != 0
    {
        said_what_took_it();
        // And the frame itself, which is the one thing the arrays cannot be asked for: a hit whose dump
        // names nothing within reach of the player was still made by something that was drawn.
        crate::shot::wanted("hit");
    }
    *LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(steered);
    // **低速移動 under a boss**, which is where this game's shots do their damage — and off again for either of
    // the two things that want the whole step: anything that reaches the move chosen inside the frames looked
    // at, since the halving is paid for by the frame a bullet is a hit, and an item to go for, which is most of
    // a field of crossing at half the speed.
    if !steered.dodging && !steered.for_an_item && unsafe { Th07::boss() }.is_some() {
        word |= BUTTON_FOCUS;
    }
    // **Whether the step the projection was made of is the step the frame will take.** The scale
    // [`a_step`] reads is what the player's own update left last frame, and that update is what the focus key
    // moves — so on any frame this changes its mind about 低速 the projection was made with the step of the
    // other state, which for 1.0 against 0.4 is two and a half times out. Whether that is what a hit is is a
    // thing only a launch can say, so the hit line says it.
    let focused = word & BUTTON_FOCUS != 0;
    let changed = WAS_FOCUSED.swap(focused, Ordering::Relaxed) != focused;
    THE_STEP_WAS_STALE.store(changed, Ordering::Relaxed);
    word | steered.word
}

/// Whether the focus key was held on the frame before, and whether that answer had just changed — which is the
/// frame the step a move was chosen with is the wrong one. Read by [`said_what_took_it`] and nothing else.
static WAS_FOCUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static THE_STEP_WAS_STALE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// **Says of the frame the hit came out of what the rules made of it**: where the player was, what it pressed,
/// how much room the model left it, and what came nearest to the move it made.
///
/// All four, because between them they say which part of the model was wrong and one of them alone does not.
/// Three launches read `with 20 of room`, `29` and `12.5` and every one of those was a hazard the model did not
/// have; three more read `4.7`, `5.2` and `4.4`, which is the same model missing by less than a step — and no
/// line said what the thing it had measured *was*, so neither run could be taken further.
fn said_what_took_it() {
    let was = LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(was) = *was else {
        crate::detail!("autoplay: hit on the first frame the rules were asked for");
        return;
    };
    let room = match was.room {
        Some(room) => format!("{room:.1} of room"),
        None => "nowhere open to it".to_owned(),
    };
    // **The hit itself at every level, and what it is diagnosed with only where somebody asked for detail.**
    // Where, what was pressed and how much room there was is the hit; which hazard was nearest and at what
    // size, what the model was made of, and whether the step was the frame before's are what a *calibration*
    // reads, and a run nobody is calibrating should not carry three lines' worth of them per hit.
    crate::detail!(
        "autoplay: hit at {:.1},{:.1} having pressed {:#05x} with {room}",
        was.at.0,
        was.at.1,
        was.word,
    );
    let of = was.counted;
    crate::detail!(
        "autoplay: the enemies' own trails put {} box(es) in",
        of.trail,
    );
    crate::detail!(
        "autoplay: {} bullet(s) no step of their own explained, {}",
        of.swept,
        match of.sweeping {
            Some(x) => format!("a beam being swept about x {x:.1}"),
            None => "too few to be a beam being swept".to_owned(),
        },
    );
    crate::detail!(
        "autoplay: the model had {} bullet(s), {} of them in another state; {} enem(ies), with {} \
         left out for having no body; {} laser(s), {} of them still only a warning, with {} passed \
         over for their phase{}{}",
        of.bullets,
        of.another_state,
        of.enemies,
        of.no_body,
        of.lasers,
        of.warnings,
        of.phase_over,
        match of.no_body_at {
            Some((at, apart)) => {
                format!(
                    ", the nearest bodiless one at {:.1},{:.1}, {:.1} off",
                    at.0,
                    at.1,
                    apart.sqrt()
                )
            }
            None => String::new(),
        },
        if THE_STEP_WAS_STALE.load(Ordering::Relaxed) {
            "; 低速 changed on that frame, so the step was the one before's"
        } else {
            ""
        },
    );
    match was.nearest {
        Some(near) => crate::detail!(
            "autoplay: nearest was {} at {:.1},{:.1} stepping {:.1},{:.1}, half of it {:.1},{:.1}",
            near.what,
            near.at.0,
            near.at.1,
            near.step.0,
            near.step.1,
            near.half.0,
            near.half.1,
        ),
        None => crate::detail!("autoplay: nothing was near it at all"),
    }
    crate::detail!(
        "autoplay: the model was holding {} within reach when it chose",
        was.held_all,
    );
    for held in was.held.iter().flatten() {
        crate::detail!(
            "autoplay: held {} {} at {:.1},{:.1} stepping {:.1},{:.1}, half of it {:.1},{:.1}",
            held.what,
            held.slot,
            held.at.0,
            held.at.1,
            held.step.0,
            held.step.1,
            held.half.0,
            held.half.1,
        );
    }
    unsafe { say_what_was_near() };
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
/// # Safety
/// Must run on the game's main thread.
unsafe fn say_what_was_near() {
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
unsafe fn steer() -> Steered {
    let at = unsafe { player() };
    let field = unsafe { field() };
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
        let (across, down) = if dx != 0.0 && dy != 0.0 {
            diagonal
        } else {
            straight
        };
        std::array::from_fn::<_, LOOK_AHEAD, _>(|frame| {
            let frames = (frame + 1) as f32;
            (
                (at.0 + dx * across * frames).clamp(field.0, field.0 + field.2),
                (at.1 + dy * down * frames).clamp(field.1, field.1 + field.3),
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
    let mut counted = Counted::default();
    let mut judged = [Judged::CLEAR; MOVES.len()];
    let mut nearest = [None; MOVES.len()];
    let mut held = [None; HELD];
    let mut held_all = 0;
    // The bullets no step of their own explains, and where across the field they are — see [`a_sweep`].
    let mut swept = 0u32;
    let mut swept_x = 0.0;
    // `grows` is how much a hazard's own half is taken to spread each frame, which is nought for everything a
    // step describes and its own speed for the one thing no reading does — see [`a_bullet_moved`].
    let mut against = |what: &'static str,
                       slot: usize,
                       (hx, hy): (f32, f32),
                       (sx, sy): (f32, f32),
                       (wx, wy): (f32, f32),
                       grows: (f32, f32)| {
        // Held before it is judged, and by where it is rather than by where it is projected: what a hit is read
        // against is what the model had, and a projection is the thing being called into question.
        if (hx - at.0).abs() < NEAR_ENOUGH_TO_BE_IT + wx
            && (hy - at.1).abs() < NEAR_ENOUGH_TO_BE_IT + wy
        {
            held_all += 1;
            if let Some(room) = held.iter_mut().find(|held| held.is_none()) {
                *room = Some(Held {
                    what,
                    slot,
                    at: (hx, hy),
                    step: (sx, sy),
                    half: (wx, wy),
                });
            }
        }
        {
            for ((judged, nearest), path) in judged.iter_mut().zip(&mut nearest).zip(&lands) {
                for (frame, (x, y)) in path.iter().enumerate() {
                    let ahead = (frame + 1) as f32;
                    let (wx, wy) = (wx + grows.0 * ahead, wy + grows.1 * ahead);
                    let (dx, dy) = ((x - (hx + sx * ahead)).abs(), (y - (hy + sy * ahead)).abs());
                    // What was left on the tighter of the two axes, which is negative on both where the boxes
                    // overlap — so the larger of the two is the room, and a room below zero is the hit.
                    let room = (dx - (mine_x + wx)).max(dy - (mine_y + wy));
                    if room < judged.room {
                        judged.room = room;
                        *nearest = Some(Nearest {
                            what,
                            at: (hx, hy),
                            step: (sx, sy),
                            half: (wx, wy),
                        });
                    }
                    // **[`ROOM_TO_SPARE`] and not nought**: a frame the move only just clears is a frame it is
                    // treated as losing, which is what keeps it out of the choice as well as out of the test.
                    if room < ROOM_TO_SPARE {
                        judged.hit(frame);
                    }
                }
            }
        }
    };
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
        // this is not the reading to leave a bullet out on. The count is kept so a hit still says how many were
        // of a state the collision was not found in.
        match unsafe { mem::read::<u16>(bullet + BULLET_STATE) } {
            0 => {
                // **Forgotten, so the next bullet in this slot is not given the last one's place.** The slots
                // recycle constantly, and a new bullet inheriting where the old one stood looks like something
                // that moved the width of the field in a frame — which is a scripted bullet by the test below,
                // feared over a box that size, and a bot cornered in the bottom left of 1面's opening.
                a_bullet_is_gone(slot);
                continue;
            }
            state if state != BULLET_HITS => counted.another_state += 1,
            _ => {}
        }
        counted.bullets += 1;
        // Its own update adds its step to its position every frame, so where it will be in a frame's time is
        // where it is plus as many steps as there are frames to look — and its own half, which is half of the
        // size the hit test is given.
        let (wx, wy) = unsafe { position(bullet + BULLET_SIZE) };
        let at = unsafe { position(bullet + BULLET_POSITION) };
        let step = unsafe { position(bullet + BULLET_STEP) };
        let moved = a_bullet_moved(slot, at, step);
        let turned = a_step_turned(slot, step);
        let scripted = (moved.0 - step.0).abs() > A_STEP_OF_ITS_OWN
            || (moved.1 - step.1).abs() > A_STEP_OF_ITS_OWN
            || turned.0.abs() > A_STEP_OF_ITS_OWN
            || turned.1.abs() > A_STEP_OF_ITS_OWN;
        if scripted {
            swept += 1;
            swept_x += at.0;
        }
        against(
            "a bullet",
            slot,
            at,
            if scripted { (0.0, 0.0) } else { step },
            (wx / 2.0, wy / 2.0),
            // **Feared and not projected.** Where a script is moving it there is nothing to project, so what
            // is kept away from is everywhere it could reach: its box spread by the speed it just showed, for
            // every frame looked at.
            if scripted {
                (moved.0.abs(), moved.1.abs())
            } else {
                (0.0, 0.0)
            },
        );
    }
    // **And the enemies, whose bodies are as solid as a bullet in this game.** Which is what a run on its way
    // to an item walks into: the item was dropped by something, and that something is still standing over it.
    //
    // **With its own body**, a third of `+0x2b3c` on each side, which is what 0x420490 and the box test it calls
    // make of that size between them: a boss taken for a point is a boss the model says there is room under
    // while its body reaches.
    //
    // **And with where it is going**, which is [`moved`] and not a field: nothing on an enemy is a step, its
    // script being what moves it, so the motion is the difference from last frame the way a laser's turn is. A
    // body read as standing still is one the model leaves room beside for as long as it takes to arrive — a
    // launch was hit beside 2面's midboss with 4.7 of room, which is less than a body covers in a frame.
    // **And only the ones with a body to run into.** 0x420e86 and 0x420e9f gate the enemy's own collision on
    // the two bits of [`ENEMY_SOLID`], so an enemy with either down cannot touch the player at all — which
    // 妖々夢's 6面 boss relies on: it fires 自機狙い off an enemy its script keeps *at* the player, and a model
    // that made that solid had a hazard nothing could get away from.
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES + slot * ENEMY_BYTES;
        if unsafe { mem::read::<u8>(enemy + ENEMY_ALIVE) } & ENEMY_ALIVE_BIT == 0 {
            moves_no_more(slot);
            continue;
        }
        if unsafe { mem::read::<u8>(enemy + ENEMY_SOLID) } & ENEMY_SOLID_BITS != ENEMY_SOLID_BITS {
            counted.no_body += 1;
            // **The nearest of them kept for the hit line**, this being the one rule here that leaves a hazard
            // out: a hit reading twelve units of room with no bullet, no laser and no enemy near enough to be
            // it is a hit by something the model does not have, and this is the only thing it throws away.
            let there = unsafe { position(enemy + ENEMY_POSITION) };
            let apart = (there.0 - at.0).powi(2) + (there.1 - at.1).powi(2);
            if counted.no_body_at.is_none_or(|(_, was)| apart < was) {
                counted.no_body_at = Some((there, apart));
            }
            moves_no_more(slot);
            continue;
        }
        counted.enemies += 1;
        let (wx, wy) = unsafe { position(enemy + ENEMY_POSITION) };
        let (sx, sy) = unsafe { position(enemy + ENEMY_SIZE) };
        // **And feared rather than projected where a script is swinging it.** 6面's turning beam is a line of
        // these, and a straight line from last frame's motion is wrong by the whole of the sweep — so the test
        // is whether that motion is itself changing, and what is kept away from is everywhere it could reach.
        let went = moved(slot, (wx, wy));
        let swung = an_enemy_turned(slot, went);
        let scripted = swung.0.abs() > A_STEP_OF_ITS_OWN || swung.1.abs() > A_STEP_OF_ITS_OWN;
        against(
            "an enemy",
            slot,
            (wx, wy),
            if scripted { (0.0, 0.0) } else { went },
            (
                sx / ENEMY_BOX_OF_ITS_SIZE / 2.0,
                sy / ENEMY_BOX_OF_ITS_SIZE / 2.0,
            ),
            if scripted {
                (went.0.abs(), went.1.abs())
            } else {
                (0.0, 0.0)
            },
        );
        // **And every place it has been, each its own box.** A beam that follows an enemy is its trail, tested
        // box by box against the player at 0x4210cd with the enemy's own size — so each of them is a hazard of
        // exactly the kind a bullet is, and one enemy can carry a screen's worth of them.
        //
        // **Standing still, because a place already gone to does not move.** The trail holds where the enemy
        // *was*, so the shape of it is fixed in space and only its ends come and go: the projection is the
        // position itself, and nothing here has to be feared for want of a step.
        for node in unsafe { the_trail(enemy) } {
            counted.trail += 1;
            against(
                "a beam's node",
                slot,
                unsafe { position(node) },
                (0.0, 0.0),
                (
                    sx / ENEMY_BOX_OF_ITS_SIZE / 2.0,
                    sy / ENEMY_BOX_OF_ITS_SIZE / 2.0,
                ),
                (0.0, 0.0),
            );
        }
    }
    // **And the lasers, which are the one hazard a distance cannot stand in for**: a beam is a place, not a
    // point, so a move is judged inside the beam's own frame — turn the landing about the origin by the angle
    // and it is *along* and *across*, which is the shape the game's own test at 0x43e6b0 makes of it.
    //
    // **A 予告線 is left for the frames it will be firing on and not for being drawn**: 0x4266dc gates the
    // game's own test on [`LASER_TIMER`] reaching [`LASER_LIVE_AT`], so a beam still drawing its warning hits
    // nothing at all — and a bot that made the warning a hazard would keep the player out of the one lane a
    // pattern leaves open. The warning is still where the beam *will* be, though, which is the whole reason
    // anybody moves off one: the frames looked at that fall after it fires are frames to be elsewhere for. So
    // the count decides, frame by frame, and neither answer has to be guessed.
    //
    // The angle is the one it has now carried on by the turn it made last frame — nothing in a laser's update
    // moves it, so a beam whose sweep speeds up or turns back is a beam this is late about.
    for slot in 0..LASER_SLOTS {
        let laser = G_LASERS + slot * LASER_BYTES;
        // **`+0x4d4` is the aliveness after all**, and the walk that owns the collision says so: 0x4263f9
        // compares it with nothing and skips the slot where it is, from a base of `[this]+0x366628` — which is
        // this same array, the constructor at 0x4233aa laying out its 0x40 slots of 0x4ec there.
        //
        // **So the beams that keep hitting the bot are not lasers.** 1面's last card and 6面's turning one both
        // read every one of these slots as empty while they are on the screen, and taking the gate away instead
        // put 57 of them in — stale geometry in slots holding nothing. Whatever those beams are, they are in the
        // bullet manager, which the model already has; what it does not have is a way to say where a bullet goes
        // when a script rather than its own step is moving it.
        if unsafe { mem::read::<u32>(laser + LASER_ALIVE) } == 0 {
            turns_no_more(slot);
            continue;
        }
        if unsafe { mem::read::<u8>(laser + LASER_PHASE) } >= LASER_FADING {
            counted.phase_over += 1;
            turns_no_more(slot);
            continue;
        }
        let fires_in = unsafe { mem::read::<i32>(laser + LASER_LIVE_AT) }
            - unsafe { mem::read::<i32>(laser + LASER_TIMER) };
        // **Judged whatever frame it fires on, and counted so a hit says how many were only warnings.** The
        // timer used to skip the frames before firing, and the frames looked at being twelve, a beam firing in
        // twenty of them was nothing at all to the model: the bot stood inside a lattice of warning lines until
        // twelve frames before they went live, and two hits came back saying `2.3 of room` and `3.3` with twenty
        // lasers left out. What the timer says is that a warning cannot *hit* — which is why a beam is judged by
        // the room it leaves rather than shutting a move outright, the ranking preferring a move hit late over
        // one hit now. Where a beam will be is where the player should not be.
        counted.lasers += 1;
        if fires_in > 0 {
            counted.warnings += 1;
        }
        let (ox, oy) = unsafe { position(laser + LASER_ORIGIN) };
        let angle = unsafe { mem::read::<f32>(laser + LASER_ANGLE) };
        let turn = turned(slot, angle);
        let (near, far) = (unsafe { mem::read::<f32>(laser + LASER_NEAR) }, unsafe {
            mem::read::<f32>(laser + LASER_FAR)
        });
        let across = unsafe { mem::read::<f32>(laser + LASER_WIDTH) } / 2.0;
        for ((judged, nearest), path) in judged.iter_mut().zip(&mut nearest).zip(&lands) {
            for (frame, (x, y)) in path.iter().enumerate() {
                // Where the beam will be pointing by then, which is where it points now plus as many of its own
                // turns as there are frames to look.
                let ahead = angle + turn * (frame + 1) as f32;
                let (cos, sin) = (ahead.cos(), ahead.sin());
                let (dx, dy) = (x - ox, y - oy);
                let along = dx * cos + dy * sin;
                let off = (-dx * sin + dy * cos).abs();
                // **How much room a beam left, kept the way a bullet's is**: measured in the beam's own frame,
                // where *across* it is what the width stands against and *along* it what its two ends do.
                // Without it a hit taken on a laser reports the room to whatever bullet happened to be nearest,
                // and a line that names the wrong hazard is worse than one that names none.
                let room = (off - (across + mine_y))
                    .max((near - mine_x - along).max(along - (far + mine_x)));
                if room < judged.room {
                    judged.room = room;
                    let on = along.clamp(near, far);
                    *nearest = Some(Nearest {
                        what: "a laser",
                        at: (ox + on * cos, oy + on * sin),
                        step: (0.0, 0.0),
                        half: (across, across),
                    });
                }
                if room < ROOM_TO_SPARE {
                    // **Live by then or still only drawn.** The game's own gate is the timer reaching the frame
                    // it fires on, so a frame this many ahead is one the beam can hit on only once the countdown
                    // has run out — before that the lane is a place to be out of rather than a frame lost.
                    if (frame + 1) as i32 >= fires_in {
                        judged.hit(frame);
                    } else {
                        judged.warn(frame);
                    }
                }
            }
        }
    }
    // **The nearest to where it wants to be, of the moves nothing will be at.** Which is not the same as the
    // one furthest from everything: the next frame's positions are known, so what a move has to be is *not
    // where a bullet lands* — and keeping away from bullets besides is a bot that spends the stage leaving,
    // since the screen of a 妖々夢 stage is never empty and being near one is the ordinary state of playing.
    // **Where to be, and only now that the bullets have been walked**: which third of the field a script is
    // sweeping a beam across is one of the things that decides it, and that is a thing the walk finds.
    let sweeping = a_sweep(swept, swept_x);
    counted.swept = swept;
    counted.sweeping = sweeping;
    let wanted = unsafe { where_to_be(field, sweeping) };
    let mut best: Option<(u16, Judged, f32, f32)> = None;
    for (((word, dx, _), judged), land) in MOVES.iter().zip(judged).zip(lands) {
        // How near this frame's own step gets to where it wants to be, which is the first of the path: the
        // frames after it are what the move is judged *hit* by and not where it is going.
        let (x, y) = land[0];
        let towards = (x - wanted.0).powi(2) + (y - wanted.1).powi(2);
        // Clear first, then nearest to where it wants to be. And where nothing is clear: **the latest first
        // hit, then fewest frames hit, then the most room** — and either over a move that merely has space
        // around it, which is the fleeing all this exists instead of. Strictly better only, so a tie goes to the
        // earlier of [`MOVES`]: standing still is first, and a bot that took the last of a tie would step
        // between two of them every frame.
        //
        // **When before how much**, because the move is made again next frame: what a move has to buy is frames
        // to decide in, and a move clear for nine of them and hit on two after that is worth more than one hit
        // on the very next frame. The count came first while [`LOOK_AHEAD`] was three, where "hit at all" and
        // "hit next frame" are nearly the same thing; at twelve they are not, and a launch that kept the old
        // order was hit nine times before 3面's boss where the one before it reached 4面 without a scratch.
        // **Three and not two: nothing will be at it, a warning is drawn across it, something will.** Which is
        // what lets a card that fans its warnings over the whole field be crossed — every move being warned, the
        // three collapse to one and the wanting decides, where shutting a move per warning left all nine shut.
        let tier = |judged: Judged| {
            if !judged.clear() {
                2
            } else if judged.only_warned() {
                1
            } else {
                0
            }
        };
        let better = match best {
            None => true,
            Some((_, was, ..)) if tier(judged) != tier(was) => tier(judged) < tier(was),
            Some((_, was, was_dx, was_towards)) => match (was.clear(), judged.clear()) {
                (false, true) => true,
                (true, false) => false,
                // **And the emptier of two the wanting cannot tell apart.** Every symmetric pair is such a
                // tie, and so is every frame the bot is standing where it wanted to be — which today went to
                // whichever comes first in [`MOVES`], a fixed lean to one side of the screen with nothing
                // behind it. The room is already measured, so this costs nothing and overrides nothing: above
                // the wanting the same number makes a bot that steps whenever a neighbour is a shade freer and
                // never arrives, and as a threshold on how cramped is too cramped it has to be guessed.
                // **And the one crossing fewer of the warnings first, where both are crossing some.** Above the
                // wanting because the frames a warning covers are frames the beam will be firing on: a lane
                // left is a lane left before it matters, and by the frame it matters the move is the same move.
                (true, true) => {
                    judged.warnings() < was.warnings()
                        || (judged.warnings() == was.warnings()
                            && (towards < was_towards
                                || (towards == was_towards && judged.room > was.room)))
                }
                // **And with nothing open, sideways before anything else that lasts as long.** The fire's
                // thickness is a thing of *x*: it comes from above and follows the player down, so a move that
                // does not change the x cannot get out of it — down buys frames and arrives at the wall, which
                // is where three runs were hit at `y = 432` with nowhere open to them. Only after the sideways
                // ones does the room decide.
                (false, false) => {
                    let sideways = *dx != 0.0;
                    let was_sideways = was_dx != 0.0;
                    judged.first() > was.first()
                        || (judged.first() == was.first()
                            && (judged.hits() < was.hits()
                                || (judged.hits() == was.hits()
                                    && ((sideways && !was_sideways)
                                        || (sideways == was_sideways && judged.room > was.room)))))
                }
            },
        };
        if better {
            best = Some((*word, judged, *dx, towards));
        }
    }
    // **What the 低速移動 under a boss is off for is anything that reaches the move chosen** inside the frames
    // looked at, and not only what would hit it next frame: halving the step doubles the frames it takes to be
    // out from under something, so by the frame a bullet is a hit the halving has already been paid for.
    //
    // **And not a distance beside it**, which is what this was: `near` is between two centres where a hit is
    // two boxes overlapping, so a bullet 12 away is inside a 大玉's own half and outside a 米弾's — the same
    // number meaning a hit and a clear frame. [`LOOK_AHEAD`] frames of the game's own test already say whether
    // anything reaches the player, and one more margin on top of it says nothing the test does not.
    // **Whether the player has to move at all**, which is [`MOVES`]'s own first entry and not the move chosen:
    // the move chosen is clear on almost every frame, that being what choosing it means, so a slowing taken off
    // by it is a slowing never taken off. Where holding still keeps the player out of everything for the frames
    // looked at, the step is not what the frame is for and the damage is.
    let (word, judged_it, _, _) = best.unwrap_or((0, Judged::CLEAR, 0.0, 0.0));
    let chosen = MOVES.iter().position(|(it, _, _)| *it == word);
    Steered {
        word,
        dodging: !judged[0].clear(),
        room: judged_it.clear().then_some(judged_it.room),
        for_an_item: unsafe { the_item_worth_going_for() }.is_some(),
        at,
        nearest: chosen.and_then(|chosen| nearest[chosen]),
        counted,
        held,
        held_all,
    }
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
/// **A beam a script is sweeping**, as the x it is spread about — or nothing where the bullets no step explains
/// are too few to be one.
///
/// A turning beam in this game is a line of bullets, and a script rather than any field of theirs is what swings
/// it: 6面's is a dozen and more of them at once. So what tells one from a bullet whose own curve the reading
/// happens to miss is how many there are, and the floor is the one the thickest third already uses — more than
/// the nine moves, which is more than a rounding difference in either.
///
/// **The count is what a launch says.** A hit's own line carries it, so a card whose every bullet reads as
/// scripted is a card this names and not one it hides.
fn a_sweep(swept: u32, swept_x: f32) -> Option<f32> {
    (swept as usize > MOVES.len()).then(|| swept_x / swept as f32)
}

unsafe fn where_to_be(field: (f32, f32, f32, f32), sweep: Option<f32>) -> (f32, f32) {
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    // The item worth going for, at the place it is; under a boss with nothing to collect the boss's own x with
    // the y low, a fight being a thing to stand under rather than to reach; and with neither the middle, low,
    // which is the shortest way to wherever the next item falls.
    let going_for = unsafe { the_item_worth_going_for() };
    let (x, y) = match going_for {
        Some((_, at)) => at,
        None => match unsafe { the_boss_to_stand_under(field) } {
            Some(x) => (x, low),
            None => (field.0 + field.2 / 2.0, low),
        },
    };
    // **And out of a third of the field the fire is clearly piled into**, whichever of the three that x is in.
    //
    // Which is the one thing the nine moves cannot say between them: each asks whether *it* is hit over the
    // frames looked at, so the bot will stand in the thick of a pattern for as long as the next twelve frames
    // happen to be survivable and be somewhere with nothing open the moment they are not. That is what its hits
    // are — `nowhere open to it` with a model holding 123 bullets and nine bodies is a place chosen and not a
    // hazard missed.
    //
    // **Unless what it is going for is worth more than a point item.** Being crushed costs a bomb at best and
    // the run at worst, which a point or a cherry is worth neither of; a 1up, a bomb, or the eight steps of
    // power [`ITEM_BIG_POWER`] carries is worth the crossing. So the two rules meet at [`worth`], which is the
    // one place this game's items are ranked.
    let worth_the_crossing = going_for.is_some_and(|(it, _)| it > worth(ITEM_POINT));
    // **And out of the third a script is sweeping a beam across, whatever is in it.** Which the thickness cannot
    // stand in for: a beam is a dozen bullets rather than a hundred, so the third it is crossing is rarely the
    // thickest — and nothing about a swept bullet says where it will be, so the frames looked at are no answer
    // either. **Not weighed against an item the way the thickness is**, because the two are not the same bet: a
    // third full of fire is a place a run of good frames gets through, and a beam arriving is arriving.
    let swept = sweep.map(|x| a_third(field, x));
    let x = match unsafe { the_thickest_third(field) } {
        _ if swept.is_some_and(|swept| swept == a_third(field, x)) => {
            let empty = unsafe { the_emptiest_third(field) };
            field.0 + field.2 * (empty as f32 + 0.5) / THIRDS as f32
        }
        Some(thick) if !worth_the_crossing && thick == a_third(field, x) => {
            let empty = unsafe { the_emptiest_third(field) };
            field.0 + field.2 * (empty as f32 + 0.5) / THIRDS as f32
        }
        _ => x,
    };
    // **Inside the band or waited for at the low line**, and not held at the band's own top edge: an item is
    // almost always above, so clamping its y into the band put the bot to live on that edge — the middle of the
    // field, moving only sideways, with the dodge it wanted to make upwards pulled straight back. Waiting low is
    // where the item arrives anyway.
    //
    // **And the x held [`IN_FROM_THE_SIDE`] off either edge**, which the y has needed all along and the x turned
    // out to need too: a corner is a place with nothing open in it. A field too narrow to keep that off both
    // sides has only its middle to offer, which is where a run with no field read at all ends up.
    let (inside, outside) = (
        field.0 + IN_FROM_THE_SIDE,
        field.0 + field.2 - IN_FROM_THE_SIDE,
    );
    (
        if inside <= outside {
            x.clamp(inside, outside)
        } else {
            field.0 + field.2 / 2.0
        },
        if y >= the_band(field) { y } else { low },
    )
}

/// The x of the boss the fight is with, where there is one worth standing under.
///
/// **Inside the play field**, since the x of something the player cannot get under is an x it holds itself
/// against a wall for — and a wall is where a dodge runs out of moves, three of the nine leading back into it.
/// A boss that has left the field is one to stop following, and the middle of the field is a better place to
/// wait than the corner it would otherwise be waited out in.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_boss_to_stand_under(field: (f32, f32, f32, f32)) -> Option<f32> {
    // **Every slot and not the first that is alive.** 妖々夢's 4面 is three bosses in the eight, and the ones
    // the fight is not with wait off the side of the field — alive, so no aliveness test passes them over, and
    // outside it, so their x is nothing to stand under. Taking the first alive one and then finding it unusable
    // is answering "no boss": a launch stood in the middle of the field for the whole of that fight.
    (0..BOSS_SLOTS)
        .filter_map(|slot| {
            let boss =
                unsafe { mem::read::<usize>(G_ENEMY_MANAGER + BOSSES + slot * size_of::<usize>()) };
            (boss != 0
                && mem::read_committed::<u8>(boss + ENEMY_ALIVE)
                    .is_some_and(|alive| alive & ENEMY_ALIVE_BIT != 0))
            .then(|| unsafe { mem::read::<f32>(boss + ENEMY_POSITION) })
        })
        .find(|x| *x >= field.0 && *x <= field.0 + field.2)
}

/// How many pieces the field is cut into across its width for the fire in each to be counted.
const THIRDS: usize = 3;

/// **The smallest bullet this game fires, which is the unit the fire in a third is counted in.**
///
/// The sizes are written at spawn and there are six of them — 4 at 0x423a4e up to 24 at 0x4275ca — so counting
/// bullets one apiece asks how many heads are in a third where what decides whether there is a way through is
/// how much of its width they take. A 大玉 is six of these across, and three of them across a third is a wall
/// where three 米弾 is a gap.
const A_SMALL_ONE: f32 = 4.0;

/// **And what a beam's own box is worth against a bullet of the same width.**
///
/// A bullet crosses the band and is gone, so a third full of them is a third with a run of good frames coming;
/// the places an enemy has been do not move at all, so a third a beam lies across has none. **A judgement and
/// not a measurement** — what a launch can say is whether the bot still walks into 6面's spiral at this, and the
/// practice is where that is read.
const A_BEAM_OVER_A_BULLET: f32 = 3.0;

/// **How much of a third's fire one box of a given size is**, in [`A_SMALL_ONE`]s.
///
/// **And never less than one of them**, which is not a guard against a bad reading: nothing this game fires is
/// narrower than the smallest it fires, so a size below that is a size that has not been written yet — a slot
/// filled in the frame it spawns on. Weighted as it stands, such a box would count for nothing at all, and a
/// pattern read as it arrives is the one worth leaving the third for.
fn how_wide(size: f32) -> f32 {
    size.max(A_SMALL_ONE) / A_SMALL_ONE
}

/// **What the middle one's count is multiplied by before the three are compared.**
///
/// The middle of the field is where a pattern is thickest by nature — 自機狙い converges there and so does most
/// of what a boss fires — so counting it against the outer two as it stands is a bot that leaves the middle for
/// an imbalance that was always going to be there. Four fifths is what somebody who plays this game put it at
/// once the counting was over the [`THE_BAND`] the player can actually be in rather than the whole column.
const IN_THE_MIDDLE_ANYWAY: f32 = 0.8;

/// Which of the [`THIRDS`] a place across the field is in.
fn a_third(field: (f32, f32, f32, f32), x: f32) -> usize {
    let across = ((x - field.0) / field.2 * THIRDS as f32).floor();
    (across.max(0.0) as usize).min(THIRDS - 1)
}

/// **How much fire each of the [`THIRDS`] holds**, the middle one's counted at [`IN_THE_MIDDLE_ANYWAY`].
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_fire_in_each_third(field: (f32, f32, f32, f32)) -> [f32; THIRDS] {
    let mut fire = [0.0; THIRDS];
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        if unsafe { mem::read::<u16>(bullet + BULLET_STATE) } == 0 {
            continue;
        }
        // **Only what is inside the band the player can be in.** The bot never rises above [`THE_BAND`], so
        // fire above it is fire nothing it does can put it among — counting the whole column of each third
        // mixed that in and buried the difference the three are compared for: a run was hit in the middle of
        // the field with the middle thick, and the counting could not see it.
        let (x, y) = unsafe { position(bullet + BULLET_POSITION) };
        if y < the_band(field) {
            continue;
        }
        fire[a_third(field, x)] += how_wide(unsafe { mem::read::<f32>(bullet + BULLET_SIZE) });
    }
    // **And every place a beam-carrying enemy has been counts as one of them.** A beam that follows an enemy is
    // its trail, and the game tests the player against each of those boxes exactly as it does a bullet — so a
    // third with a beam laid across it is a third full of fire, and one that counted only the bullets read the
    // spiral 6面 fires as an empty place to stand.
    for slot in 0..ENEMY_SLOTS {
        let enemy = G_ENEMY_MANAGER + ENEMIES + slot * ENEMY_BYTES;
        if unsafe { mem::read::<u8>(enemy + ENEMY_ALIVE) } & ENEMY_ALIVE_BIT == 0 {
            continue;
        }
        if unsafe { mem::read::<u8>(enemy + ENEMY_SOLID) } & ENEMY_SOLID_BITS != ENEMY_SOLID_BITS {
            continue;
        }
        let wide = how_wide(unsafe { mem::read::<f32>(enemy + ENEMY_SIZE) });
        for node in unsafe { the_trail(enemy) } {
            let (x, y) = unsafe { position(node) };
            if y < the_band(field) {
                continue;
            }
            fire[a_third(field, x)] += wide * A_BEAM_OVER_A_BULLET;
        }
    }
    fire[THIRDS / 2] *= IN_THE_MIDDLE_ANYWAY;
    fire
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

/// **The third of the field the fire is clearly piled into**, or nothing where none of them is.
///
/// **Measured against the emptiest of the three and not against the screen.** Asking whether one third holds
/// more than half of everything is asking the wrong question: a run was hit in the middle with the left
/// obviously empty, and the middle still held under half because the right held the rest. What matters is that
/// there is somewhere clearly better to be.
///
/// Clearly, and said in two numbers rather than a taste: **twice the emptiest**, which is a factor and so means
/// the same at thirty bullets as at three hundred; and **more of a difference than there are moves to weave
/// between**, so that a handful is not a pile. An ordinary imbalance is not a reason to leave a fight.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_thickest_third(field: (f32, f32, f32, f32)) -> Option<usize> {
    let fire = unsafe { the_fire_in_each_third(field) };
    let empty = fire.iter().copied().fold(f32::INFINITY, f32::min);
    (0..THIRDS)
        .find(|third| fire[*third] > empty * 2.0 && fire[*third] - empty > MOVES.len() as f32)
}

/// And the emptiest of them, which is where the bot waits instead.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn the_emptiest_third(field: (f32, f32, f32, f32)) -> usize {
    let fire = unsafe { the_fire_in_each_third(field) };
    (0..THIRDS)
        .min_by(|a, b| fire[*a].total_cmp(&fire[*b]))
        .unwrap_or(THIRDS / 2)
}

/// The top of the band the bot moves in, which is the bottom [`THE_BAND`] of the field.
fn the_band(field: (f32, f32, f32, f32)) -> f32 {
    field.1 + field.3 * (1.0 - THE_BAND)
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
