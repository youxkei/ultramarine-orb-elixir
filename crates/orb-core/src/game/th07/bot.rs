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
    BOSS_SLOTS, BOSSES, BULLET_BYTES, BULLET_HITS, BULLET_POSITION, BULLET_SIZE, BULLET_SLOTS,
    BULLET_STATE, BULLET_STEP, BULLETS, BUTTON_BOMB, BUTTON_FOCUS, BUTTON_SHOT, BUTTON_SKIP,
    ENEMIES, ENEMY_ALIVE, ENEMY_ALIVE_BIT, ENEMY_BOX_OF_ITS_SIZE, ENEMY_BYTES, ENEMY_POSITION,
    ENEMY_SIZE, ENEMY_SLOTS, ENEMY_SOLID, ENEMY_SOLID_BITS, FIELD_HEIGHT, FIELD_LEFT, FIELD_TOP,
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
    bullets_left_out: u32,
    enemies: u32,
    enemies_left_out: u32,
    lasers: u32,
    lasers_left_out: u32,
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
#[derive(Clone, Copy)]
struct Judged {
    on: u32,
    room: f32,
}

impl Judged {
    const CLEAR: Self = Self {
        on: 0,
        room: f32::INFINITY,
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

/// Says a slot holds no enemy, so the next one to use it starts without a motion.
fn moves_no_more(slot: usize) {
    LAST_PLACE[slot].0.store(NOWHERE, Ordering::Relaxed);
    LAST_PLACE[slot].1.store(NOWHERE, Ordering::Relaxed);
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
        log!("autoplay: hit on the first frame the rules were asked for");
        return;
    };
    let room = match was.room {
        Some(room) => format!("{room:.1} of room"),
        None => "nowhere open to it".to_owned(),
    };
    // Said only where it is true, a line saying nothing about the step being a line whose step was the right
    // one — and where it is true it is the first thing to look at, the model having moved the player somewhere
    // it was never going to be.
    let room = if THE_STEP_WAS_STALE.load(Ordering::Relaxed) {
        format!("{room}, on the frame 低速 changed and the step was last frame's")
    } else {
        room
    };
    let of = was.counted;
    let model = format!(
        "the model had {} bullet(s), {} enem(ies), {} laser(s), and left out {}, {}, {}",
        of.bullets,
        of.enemies,
        of.lasers,
        of.bullets_left_out,
        of.enemies_left_out,
        of.lasers_left_out,
    );
    match was.nearest {
        Some(near) => log!(
            "autoplay: hit at {:.1},{:.1} having pressed {:#05x} with {room} — nearest was {} at \
             {:.1},{:.1} stepping {:.1},{:.1}, half of it {:.1},{:.1}; {model}",
            was.at.0,
            was.at.1,
            was.word,
            near.what,
            near.at.0,
            near.at.1,
            near.step.0,
            near.step.1,
            near.half.0,
            near.half.1,
        ),
        None => log!(
            "autoplay: hit at {:.1},{:.1} having pressed {:#05x} with {room} — and nothing near it \
             at all; {model}",
            was.at.0,
            was.at.1,
            was.word,
        ),
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
    let mut against =
        |what: &'static str, (hx, hy): (f32, f32), (sx, sy): (f32, f32), (wx, wy): (f32, f32)| {
            for ((judged, nearest), path) in judged.iter_mut().zip(&mut nearest).zip(&lands) {
                for (frame, (x, y)) in path.iter().enumerate() {
                    let ahead = (frame + 1) as f32;
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
        };
    for slot in 0..BULLET_SLOTS {
        let bullet = G_BULLET_MANAGER + BULLETS + slot * BULLET_BYTES;
        // **Only the state the collision is in.** The other four the dispatch at 0x425b82 knows move a bullet
        // by a fraction of its step and reach neither of the player's tests, so a bullet in one of them is on
        // its way into existence and cannot touch anything yet — which is most of a wall of bullets on the
        // frame a boss fires it.
        match unsafe { mem::read::<u16>(bullet + BULLET_STATE) } {
            0 => continue,
            state if state != BULLET_HITS => {
                counted.bullets_left_out += 1;
                continue;
            }
            _ => counted.bullets += 1,
        }
        // Its own update adds its step to its position every frame, so where it will be in a frame's time is
        // where it is plus as many steps as there are frames to look — and its own half, which is half of the
        // size the hit test is given.
        let (wx, wy) = unsafe { position(bullet + BULLET_SIZE) };
        against(
            "a bullet",
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
            counted.enemies_left_out += 1;
            moves_no_more(slot);
            continue;
        }
        counted.enemies += 1;
        let (wx, wy) = unsafe { position(enemy + ENEMY_POSITION) };
        let (sx, sy) = unsafe { position(enemy + ENEMY_SIZE) };
        against(
            "an enemy",
            (wx, wy),
            moved(slot, (wx, wy)),
            (
                sx / ENEMY_BOX_OF_ITS_SIZE / 2.0,
                sy / ENEMY_BOX_OF_ITS_SIZE / 2.0,
            ),
        );
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
        if unsafe { mem::read::<u32>(laser + LASER_ALIVE) } == 0
            || unsafe { mem::read::<u8>(laser + LASER_PHASE) } >= LASER_FADING
        {
            turns_no_more(slot);
            continue;
        }
        let fires_in = unsafe { mem::read::<i32>(laser + LASER_LIVE_AT) }
            - unsafe { mem::read::<i32>(laser + LASER_TIMER) };
        if fires_in > LOOK_AHEAD as i32 {
            counted.lasers_left_out += 1;
        } else {
            counted.lasers += 1;
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
                // Nothing to be anywhere else for on a frame the beam is still only drawn.
                if fires_in > frame as i32 + 1 {
                    continue;
                }
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
        let better = match best {
            None => true,
            Some((_, was, was_towards)) => match (was.clear(), judged.clear()) {
                (false, true) => true,
                (true, false) => false,
                (true, true) => towards < was_towards,
                (false, false) => {
                    judged.first() > was.first()
                        || (judged.first() == was.first()
                            && (judged.hits() < was.hits()
                                || (judged.hits() == was.hits() && judged.room > was.room)))
                }
            },
        };
        if better {
            best = Some((*word, judged, towards));
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
    let (word, judged_it, _) = best.unwrap_or((0, Judged::CLEAR, 0.0));
    let chosen = MOVES.iter().position(|(it, _, _)| *it == word);
    Steered {
        word,
        dodging: !judged[0].clear(),
        room: judged_it.clear().then_some(judged_it.room),
        for_an_item: unsafe { the_item_worth_going_for() }.is_some(),
        at,
        nearest: chosen.and_then(|chosen| nearest[chosen]),
        counted,
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
unsafe fn where_to_be(field: (f32, f32, f32, f32)) -> (f32, f32) {
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    // The item worth going for, at the place it is; under a boss with nothing to collect the boss's own x with
    // the y low, a fight being a thing to stand under rather than to reach; and with neither the middle, low,
    // which is the shortest way to wherever the next item falls.
    let (x, y) = match unsafe { the_item_worth_going_for() } {
        Some((_, at)) => at,
        None => match unsafe { the_boss_to_stand_under(field) } {
            Some(x) => (x, low),
            None => (field.0 + field.2 / 2.0, low),
        },
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
