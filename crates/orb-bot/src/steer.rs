//! **The word one frame of a stage takes**, out of what [`Game::what_a_bot_sees`] hands over and nothing else.
//!
//! **No address of any game is in here**, which is the point of it being here: what a bullet's own step is kept
//! at and which byte says a body is solid are questions about one game, and `orb-core`'s game seam answers them.
//! What is left is the deciding, and the deciding is not any game's — it is a bot's, and a bot is a harness
//! rather than any part of what orb does for somebody playing.
//!
//! **The rules are Normal's**, and dodging comes before all of them:
//!
//! - the shot held, let go for one frame in every [`SHOT_LET_GO_EVERY`] so that a screen waiting for a press
//!   gets one, and the skip key while a dialogue is up
//! - the bomb the frame the player is hit, which is 決死結界 — [`Frame::hit`] stands for the window the game
//!   counts down, and a bomb inside it is a death taken back
//! - **the nearest to where it wants to be, of the nine moves nothing will be at.** Holding each one is
//!   followed [`LOOK_AHEAD`] frames out, against every bullet's own step and every body where it stands, and a
//!   move whose path comes within [`ROOM_TO_SPARE`] of one of them is not taken. A body is in that because it is
//!   as solid as a bullet, and because it is what a run on its way to an item walks into — the item was dropped
//!   by something, and that something is still standing over it
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

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use orb_core::detail;
use orb_core::game::{Beam, Frame, Game, Hazard, Kind, Worth};

/// How far off the field's bottom edge the player waits with nothing to go to, in the game's own units.
///
/// **A rule's own number and not a measurement of anything.** Against the edge outright is where the game's own
/// clamp holds the player with the whole of one side taken away from it, and this much up leaves the room a
/// dodge needs on every side of it.
///
/// **More than [`LOOK_AHEAD`] downward steps and not exactly that many.** It was 48, which twelve frames of a
/// step of four cover to the unit: a dodge that spends its whole look-ahead going down arrives at the clamp on
/// the last frame it can see, so the room ran out on the very frame the trouble was picked for. Four hits in one
/// session were that arithmetic — a run reached `222,432` with the emptiest box at `328` and was carried back to
/// `118,432`, since at the clamp there is no downwards and three of [`WAYS`] are the wall.
const ABOVE_THE_BOTTOM: f32 = 64.0;

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

/// **How much a move has to clear a hazard's box by before it counts as clearing it at all.**
///
/// Not a hitbox and not a distance standing in for one: the box test is the game's own and both boxes are read.
/// This is slack on top of it, and the launches are what set it. One stage produced twelve hits reading `0.6`,
/// `1.1`, `0.6`, `0.5` and more of the same — every one of them a hazard the model had, at a size the model
/// read, with the player put past it by less than a unit. So something in the projection is out by about a unit
/// and no reading has yet said what: the step is the game's own number times the player's own scale times a
/// motion scale, any of which can move between the read and the frame, and a bullet's own script can re-aim it
/// mid-flight. Until a reading names it, room under a unit is not room.
///
/// **The game's own test agrees**: 妖々夢 compares the edges inclusively at 0x43e2e7 where this one is strict,
/// so nought of room is already a hit there and never was room at all.
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
/// 妖々夢 works out and eight at the halved one 低速移動 leaves, its widest bullet being 24 across. A move first
/// seen three frames out is therefore one there is no getting away from whatever is pressed, and 自機狙い is the
/// shot that makes it: a launch spent 6面's boss dying to the same aimed 大玉 over and over, 30 hits at one
/// chapter. Twelve is those eight and room to pick a direction in rather than having to move the frame the
/// bullet appears; past that a bullet's own step stops describing where it goes, and nothing here reads what
/// turns one.
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
    /// **And where the rules wanted to be**, which is the one thing the boxes beside a hit cannot settle: a box
    /// climbing from nought to a hundred and eighty while the bot stands in it is either a rule that never fired
    /// or something else wanted more, and those are different repairs.
    wanted: (f32, f32),
    /// **And which of the four rules settled that place.** The pair alone does not say: an item's own x with the
    /// wait line for a y reads exactly like the emptiest box's middle with nothing to collect, and those are
    /// opposite findings — a rule outranked against a rule that fired and was not followed.
    settled: Settled,
    nearest: Option<Nearest>,
    counted: Counted,
    held: [Option<Held>; HELD],
    /// How many were within reach, which is not how many of them fit in [`Steered::held`].
    held_all: u32,
    /// **What every one of [`WAYS`] came to**, in that order, so that the move taken can be read against the
    /// ones left: the ranking is a comparison, and a hit that names only the winner says nothing about why it
    /// won. A run walked itself into the floor over fifteen frames, each step the best move of its own frame, and
    /// whether *down* won those comparisons or merely tied and came earlier in the order is two repairs.
    judgements: [Judged; WAYS.len()],
}

/// **Which rule settled where the bot wanted to be**, in the order [`where_to_be`] asks them.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Settled {
    /// Nothing yet, which is a frame kept before one was asked for.
    #[default]
    Nothing,
    /// An item worth going for, at the place it is — and which kind, since that is what decides whether the box
    /// rules are exempted at all: a point item or a cherry does not exempt them and a power item does.
    AnItem(Worth),
    /// The boss's own x, a fight being a thing to stand under.
    ABoss,
    /// Neither, which is the middle of the field low.
    TheMiddle,
    /// And the emptiest box, where one of the three box rules sent it there instead.
    TheEmptiestBox,
}

impl Settled {
    fn said(self) -> &'static str {
        match self {
            Self::AnItem(Worth::Life) => "a 1up",
            Self::AnItem(Worth::Bomb) => "a bomb item",
            Self::AnItem(Worth::BigPower) => "a big power item",
            Self::AnItem(Worth::Power) => "a power item",
            Self::AnItem(Worth::Point) => "a point item",
            Self::AnItem(Worth::Cherry) => "a cherry",
            Self::AnItem(Worth::Other) => "an item of no place of its own",
            Self::Nothing => "nothing yet",
            Self::ABoss => "a boss",
            Self::TheMiddle => "the middle",
            Self::TheEmptiestBox => "the emptiest box",
        }
    }
}

/// **What the frame's model was made of, and what it left out** — bullets, bodies and beams, each as the
/// number judged against and the number passed over for not being able to hit anything.
///
/// Which is the one thing a hit saying `14.5 of room` needs and no other field of [`Steered`] can give: a model
/// that says a move is clear by fourteen units and is wrong is a model missing a hazard, and the passing over
/// rules are where a missing hazard would have gone. A run cleared the game with eight hits on 6面's boss
/// reading between six and fourteen of room, every one of them after those rules went in.
#[derive(Clone, Copy, Default)]
struct Counted {
    bullets: u32,
    /// Of those, how many were in a state the game's own collision was not found in — judged all the same, for
    /// the reason the seam's `Kind::Bullet` gives.
    another_state: u32,
    enemies: u32,
    /// And how many were passed over for having no body to run into, which is the one rule here that leaves a
    /// hazard out — with the nearest of them to the player, and how far that was, for a hit to name.
    no_body: u32,
    no_body_at: Option<((f32, f32), f32)>,
    lasers: u32,
    /// Of those, how many had not started hitting yet — a 予告線, judged for where its beam will be.
    warnings: u32,
    /// And how many of them are fading. Judged like any other, and worth saying because the width read for one
    /// is the width it had while what the game is testing against by then is narrower — so a beam left too
    /// eagerly is this number standing where the room should be.
    ///
    /// It was the count of lasers *skipped* for their phase, and the skip was the wrong walk's gate — see the
    /// seam's `Beam::fading`.
    fading: u32,
    /// And how many bullets no step of their own explains, with the x they are spread about where those are
    /// enough to be a beam a script is sweeping — see [`a_sweep`].
    swept: u32,
    sweeping: Option<f32>,
    /// And how many boxes the enemies' own trails put in, which is the beams that follow them.
    trail: u32,
    /// And the fire each box across the field was counted as holding, which is what the place to wait was chosen
    /// by — see [`the_fire_in_each_box`].
    fire: [f32; BOXES_ACROSS],
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
/// not a frame that is lost. The game's own test is gated on the beam's own countdown, so a warning cannot hit
/// anything at all — a move into one is worse than a move out of one and better than a move into a beam that is
/// live. Counted apart from [`Judged::on`] because a card that fans twenty-two warnings out of one point shuts
/// every one of the nine where the two are the same, and a bot with nothing open to it stands where it is until
/// the beams fire.
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

/// The nine one-frame moves, as the direction each goes in.
///
/// **Standing still is first**, which is what settles a tie: where two moves are equally safe the bot holds its
/// ground rather than stepping back and forth between them every frame.
/// **The direction is which way and not how far** — how far is the game's, and a diagonal's own share of a step
/// is its own number there, which is why [`Frame::diagonal`] is a pair of its own.
///
/// **Directions and not words**, which is what moved here from the game: a word is made of that game's own bits
/// and this is nine ways to go, the same nine in every game there is.
const WAYS: [(f32, f32); 9] = [
    (0.0, 0.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (-1.0, 0.0),
    (1.0, 0.0),
    (-1.0, -1.0),
    (1.0, -1.0),
    (-1.0, 1.0),
    (1.0, 1.0),
];

/// **The bits this game puts the buttons a bot presses in**, asked for by name once a frame.
///
/// By name and through the seam, because that is where a name is resolved: `left` is a bit of one word in
/// 妖々夢 and a bit of another elsewhere, and the recipe a bot walks names its presses the same way.
struct Keys {
    shot: u16,
    bomb: u16,
    focus: u16,
    skip: u16,
    up: u16,
    down: u16,
    left: u16,
    right: u16,
}

impl Keys {
    fn of(game: &dyn Game) -> Self {
        let bit = |name| game.button_bit(name).unwrap_or(0);
        Self {
            shot: bit("shot"),
            bomb: bit("bomb"),
            focus: bit("focus"),
            skip: bit("skip"),
            up: bit("up"),
            down: bit("down"),
            left: bit("left"),
            right: bit("right"),
        }
    }

    /// The word one of [`WAYS`] is pressed as.
    fn way(&self, (dx, dy): (f32, f32)) -> u16 {
        let across = match dx {
            dx if dx < 0.0 => self.left,
            dx if dx > 0.0 => self.right,
            _ => 0,
        };
        let down = match dy {
            dy if dy < 0.0 => self.up,
            dy if dy > 0.0 => self.down,
            _ => 0,
        };
        across | down
    }
}

/// How often the shot is let go for a frame, so that a screen waiting for one gets a press: a second of them.
const SHOT_LET_GO_EVERY: u32 = 60;

/// The frames these rules have been asked for, which is what [`SHOT_LET_GO_EVERY`] is counted in.
///
/// A clock of its own rather than the stage's, because the stage's stops: it does not move under the pause menu
/// and it is the clear screen's own wait that this is for.
static FRAMES: AtomicU32 = AtomicU32::new(0);

/// Whether the player was hit last frame, so that a hit is said once rather than once a frame of the window it
/// opened.
static WAS_HIT: AtomicBool = AtomicBool::new(false);

/// And whether the player had a border up on the frame before, so that its going down is said once.
static HAD_A_BORDER: AtomicBool = AtomicBool::new(false);

/// **How many of the frames before a hit are kept to be said with it**, which is a few seconds of them.
///
/// Every hit of one session was `nowhere open to it` with all of the moves hit on the frame they were judged
/// on — a place lost before its frame began — and the frame's own dump cannot say how the player came to be
/// standing there. Four of them turned out to be the x not moving for thirty frames at an edge, and that was
/// found by reading the raw `resume:` lines by hand afterwards rather than from anything orb said.
const THE_FRAMES_BEFORE: usize = 180;

/// And one in this many of them is written out, so that a few seconds is a few dozen lines rather than a page a
/// hit.
const ONE_IN: usize = 6;

/// What one of those frames is kept as: where the player was, what it pressed, whether anything was open to it,
/// and what the boxes across the field read.
#[derive(Clone, Copy, Default)]
struct Before {
    at: (f32, f32),
    wanted: (f32, f32),
    settled: Settled,
    word: u16,
    open: bool,
    fire: [f32; BOXES_ACROSS],
}

/// The frames before, oldest first once it has wrapped — see [`THE_FRAMES_BEFORE`].
static BEFORE: Mutex<(usize, [Option<Before>; THE_FRAMES_BEFORE])> =
    Mutex::new((0, [None; THE_FRAMES_BEFORE]));

/// Keep one frame, and forget the one from three seconds ago.
fn keep(before: Before) {
    let mut kept = BEFORE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let at = kept.0;
    kept.1[at] = Some(before);
    kept.0 = (at + 1) % THE_FRAMES_BEFORE;
}

/// Say the frames before a hit, oldest first, one in [`ONE_IN`].
fn say_the_frames_before() {
    let kept = BEFORE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (next, frames) = (kept.0, kept.1);
    // **Stepped over the frames there are and not over the room for them.** A run only a second old has filled a
    // handful of the slots, and a stride over the whole ring lands on the empty ones.
    let kept: Vec<_> = (0..THE_FRAMES_BEFORE)
        .filter_map(|step| frames[(next + step) % THE_FRAMES_BEFORE])
        .collect();
    let back = kept.len();
    for (step, before) in kept.into_iter().enumerate().filter(|(step, _)| {
        // Every [`ONE_IN`] of them, counted from the newest so that the frame before the hit is always one.
        (back - 1 - step).is_multiple_of(ONE_IN)
    }) {
        detail!(
            "autoplay: before it, {} frame(s) back, at {:.0},{:.0} wanting {} at {:.0},{:.0}, \
             pressing {:#05x} with {} and the boxes {}",
            back - step,
            before.at.0,
            before.at.1,
            before.settled.said(),
            before.wanted.0,
            before.wanted.1,
            before.word,
            match before.open {
                true => "something open",
                false => "nothing open",
            },
            before
                .fire
                .iter()
                .map(|fire| format!("{fire:.0}"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
}

/// **What the rules made of the last frame**, which is what a hit is reported against: the hit happened between
/// that frame's word going in and this one being asked for.
///
/// Behind a lock and not in atomics, there being more of it than a word now: it is written once and read once
/// per frame on the game's own thread, and nothing else takes this lock.
static LAST_FRAME: Mutex<Option<Steered>> = Mutex::new(None);

/// **What each slot held last frame**, which is the only reading there is of where a thing with no step is
/// going.
///
/// **A row per kind, grown to whatever the game turns out to have**, rather than an array of the game's own slot
/// count: how many slots a manager has is a fact about that game, and a bot that had to be told it would be a
/// bot with one game's numbers in it. It grows once and never again, a stage's slots being the same slots all
/// the way through.
///
/// **And the row is rebuilt each frame rather than cleared slot by slot.** A slot the game stops handing over is
/// a slot that holds nothing, and the next thing to use it must not inherit what stood there before — an enemy
/// given the motion of its predecessor is a hazard feared over a box the width of the field, and a bullet given
/// one is the same thing in the bullets. Building the new row from the slots seen this frame is that forgetting,
/// with nothing to remember to call.
#[derive(Default)]
struct Remembered {
    /// Where each bullet was, and the step it claimed.
    bullets: Vec<Option<Stood>>,
    /// Where each body was, and how it was moving.
    bodies: Vec<Option<Stood>>,
    /// The angle each beam pointed at.
    beams: Vec<Option<f32>>,
}

/// What one slot held: where its thing stood, and the step or the motion it had there.
type Stood = ((f32, f32), (f32, f32));

static REMEMBERED: Mutex<Option<Remembered>> = Mutex::new(None);

/// The frame last read out of the game, kept so that reading the next one allocates nothing.
static SEEN: Mutex<Option<Frame>> = Mutex::new(None);

/// What one row says of a slot, and nothing for a slot it does not reach.
fn what_a_slot_held<T: Copy>(row: &[Option<T>], slot: usize) -> Option<T> {
    row.get(slot).copied().flatten()
}

/// Put what a slot holds now into a row being built, growing it to reach.
fn note<T: Copy>(row: &mut Vec<Option<T>>, slot: usize, now: T) {
    if row.len() <= slot {
        row.resize(slot + 1, None);
    }
    row[slot] = Some(now);
}

/// How far a beam turned since last frame, and nothing for one that was not there then.
///
/// **A difference and not a rate**, there being no rate on a beam to read: what turns one is its boss's script.
///
/// Wrapped into half a turn either way, an angle being a thing that comes back round: the difference across
/// that seam is a whole turn of nonsense and the beam did not move at all.
fn turned(was: Option<f32>, angle: f32) -> f32 {
    let Some(was) = was else {
        return 0.0;
    };
    let turn = angle - was;
    let round = std::f32::consts::TAU;
    turn - round * (turn / round).round()
}

/// How far what a hazard actually moved may differ from the step it claims, **or its step from the step it had
/// last frame**, before it counts as moved by something other than itself. A hair, an ordinary bullet's movement
/// *being* its step to the bit and its step never changing at all.
///
/// **The step's own change is the half that catches a beam.** Comparing only the movement against the step
/// misses a piece that turns gently: the disagreement in one frame is under any threshold worth having, and the
/// straight line it is then projected along is wrong by the whole of the turn twelve frames out — which is a hit
/// reading `12.2 of room` with nothing else near, and 6面's turning card gave three of them.
const A_STEP_OF_ITS_OWN: f32 = 0.5;

// **Nothing here is a dead zone round where it wants to be**, which the choice makes unnecessary: standing
// still is one of the [`WAYS`] and the one a tie goes to, so the frame the player is there is a frame nothing
// is pressed for.

/// The word the rules press this frame, and nothing at all off a stage.
///
/// **Nothing off a stage, the shot included.** The rules are asked for a word by the game's own keyboard read,
/// which every screen makes — and a menu of 妖々夢's is taken by the shot as much as by the return key, so a
/// word held on the title screen walks that menu once and then holds it, while 十字キー there moves a cursor
/// these rules cannot read. The front end is the recipe's to drive, one press to a screen; a rule pressing
/// alongside it is a press nobody wrote.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn word_to_press(game: &dyn Game) -> u16 {
    // **The one buffer, kept between frames**, so that a frame of a run costs no allocation once it has run
    // once: the three lists it holds are cleared and filled, and a stage of 妖々夢 fills the first of them six
    // hundred times.
    let mut seen = SEEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let frame = seen.get_or_insert_with(Frame::default);
    if !unsafe { game.what_a_bot_sees(frame) } {
        return 0;
    }
    let keys = Keys::of(game);
    // **The shot held, and let go for one frame in every [`SHOT_LET_GO_EVERY`].** A stage's own clear screen
    // goes on to the next stage the frame the shot goes *down* and otherwise waits out its own timer, and a run
    // that has held the shot since the stage began never gives it that frame: one launch spent 60 seconds
    // between 1面's last chapter and 2面's first. A shot in sixty is what that costs, 妖々夢's own shot firing
    // off the player's timer rather than off a press.
    let count = FRAMES.fetch_add(1, Ordering::Relaxed);
    let mut word = if count.is_multiple_of(SHOT_LET_GO_EVERY) {
        0
    } else {
        keys.shot
    };
    // The frame the player is hit and not the frames after it — the window the game counts down is one this
    // presses into rather than a single frame, and its end takes the life.
    if frame.hit {
        word |= keys.bomb;
    }
    // A dialogue read at the game's own pace is the afternoon this was written to save.
    if frame.dialogue {
        word |= keys.skip;
    }
    let steered = steer(frame, &keys);
    // **What the rules thought of the move a hit was taken on — and it is the frame before's that they thought
    // it of.** A frame ahead is computed and a move nothing will be at is chosen, so a run that is hit at all
    // means one of three things and no others: something on the screen is not in the model, or every one of the
    // nine was shut, or the move chosen was not the move made. Which of them it was is the one thing a launch
    // can say and nothing here can.
    //
    // **This frame's judgement is of a screen the hit has already been taken off.** The bullet that did it is
    // gone — the game puts its state to spent the moment the test comes back — so the room this frame has is the
    // room to some *other* bullet, and the figure means nothing: three launches were read as 20 and 29 and 12.5
    // of room, and every one of those numbers was measured after the thing being measured had been removed.
    // What was decided on the frame the hit came out of is [`LAST_FRAME`].
    //
    // Said once for the hit and not once for every frame of the window it opened: 決死結界 stands for as long as
    // the game counts it down, and a line a frame turns one hit into a page of them.
    if WAS_HIT.swap(frame.hit, Ordering::Relaxed) != frame.hit && frame.hit {
        said_what_took_it(game);
        // And the frame itself, which is the one thing the arrays cannot be asked for: a hit whose dump
        // names nothing within reach of the player was still made by something that was drawn.
        orb_core::shot::wanted("hit");
    }
    // **And a border broken is a hit, which orb was not counting at all.** 妖々夢 deflects a hit taken on a
    // border rather than putting the player into the state above, so a run kept alive by borders read as a run
    // that was never hit — every `0 hit(s)` reported before this was reported without them.
    //
    // **A break and not any border going away**, which is what the player's own state tells apart: one in the
    // middle of dying or coming back is not one whose border was broken.
    if HAD_A_BORDER.swap(frame.border, Ordering::Relaxed) && !frame.border && frame.normal {
        detail!("autoplay: a border was broken, which is a hit");
        said_what_took_it(game);
        orb_core::shot::wanted("hit");
    }
    // **Kept every frame, because a hit takes the screen it happened on away.** What a hit needs is the seconds
    // before it, and by the frame it is noticed the bullets have moved on and the place is a place nothing is at
    // any more — see [`THE_FRAMES_BEFORE`].
    keep(Before {
        at: steered.at,
        wanted: steered.wanted,
        settled: steered.settled,
        word: word | steered.word,
        open: steered.room.is_some(),
        fire: steered.counted.fire,
    });
    *LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(steered);
    // **低速移動 under a boss**, which is where this game's shots do their damage — and off again for either of
    // the two things that want the whole step: anything that reaches the move chosen inside the frames looked
    // at, since the halving is paid for by the frame a bullet is a hit, and an item to go for, which is most of
    // a field of crossing at half the speed.
    if !steered.dodging && !steered.for_an_item && frame.boss.is_some() {
        word |= keys.focus;
    }
    // **Whether the step the projection was made of is the step the frame will take.** The step handed over is
    // what the player's own update left last frame, and that update is what the focus key moves — so on any
    // frame this changes its mind about 低速 the projection was made with the step of the other state, which for
    // 1.0 against 0.4 is two and a half times out. Whether that is what a hit is is a thing only a launch can
    // say, so the hit line says it.
    let focused = word & keys.focus != 0 && keys.focus != 0;
    let changed = WAS_FOCUSED.swap(focused, Ordering::Relaxed) != focused;
    THE_STEP_WAS_STALE.store(changed, Ordering::Relaxed);
    word | steered.word
}

/// Whether the focus key was held on the frame before, and whether that answer had just changed — which is the
/// frame the step a move was chosen with is the wrong one. Read by [`said_what_took_it`] and nothing else.
static WAS_FOCUSED: AtomicBool = AtomicBool::new(false);
static THE_STEP_WAS_STALE: AtomicBool = AtomicBool::new(false);

/// **Says of the frame the hit came out of what the rules made of it**: where the player was, what it pressed,
/// how much room the model left it, and what came nearest to the move it made.
///
/// All four, because between them they say which part of the model was wrong and one of them alone does not.
/// Three launches read `with 20 of room`, `29` and `12.5` and every one of those was a hazard the model did not
/// have; three more read `4.7`, `5.2` and `4.4`, which is the same model missing by less than a step — and no
/// line said what the thing it had measured *was*, so neither run could be taken further.
fn said_what_took_it(game: &dyn Game) {
    let was = LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(was) = *was else {
        detail!("autoplay: hit on the first frame the rules were asked for");
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
    detail!(
        "autoplay: hit at {:.1},{:.1} having pressed {:#05x} with {room}",
        was.at.0,
        was.at.1,
        was.word,
    );
    let of = was.counted;
    detail!(
        "autoplay: the enemies' own trails put {} box(es) in",
        of.trail,
    );
    // **The five numbers the place to wait was chosen by.** Whether a box is left is two comparisons against
    // the emptiest of them, and a hit boxed in at the bottom right with the bottom left plainly empty is either
    // a count that did not see it or a threshold that did not clear — which are different repairs, and nothing
    // printed tells them apart.
    detail!(
        "autoplay: the fire in each box was {}",
        of.fire
            .iter()
            .map(|fire| format!("{fire:.1}"))
            .collect::<Vec<_>>()
            .join(", "),
    );
    detail!(
        "autoplay: {} bullet(s) no step of their own explained, {}",
        of.swept,
        match of.sweeping {
            Some(x) => format!("a beam being swept about x {x:.1}"),
            None => "too few to be a beam being swept".to_owned(),
        },
    );
    detail!(
        "autoplay: the model had {} bullet(s), {} of them in another state; {} enem(ies), with {} \
         left out for having no body; {} laser(s), {} of them still only a warning and {} of them \
         fading{}{}",
        of.bullets,
        of.another_state,
        of.enemies,
        of.no_body,
        of.lasers,
        of.warnings,
        of.fading,
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
        Some(near) => detail!(
            "autoplay: nearest was {} at {:.1},{:.1} stepping {:.1},{:.1}, half of it {:.1},{:.1}",
            near.what,
            near.at.0,
            near.at.1,
            near.step.0,
            near.step.1,
            near.half.0,
            near.half.1,
        ),
        None => detail!("autoplay: nothing was near it at all"),
    }
    let keys = Keys::of(game);
    for (way, judged) in WAYS.iter().zip(was.judgements) {
        detail!(
            "autoplay: the move {:#05x} was hit first on {}, on {} frame(s), warned on {}, \
             with {:.1} of room",
            keys.way(*way),
            match judged.clear() {
                true => "none".to_owned(),
                false => judged.first().to_string(),
            },
            judged.hits(),
            judged.warnings(),
            judged.room,
        );
    }
    detail!(
        "autoplay: the model was holding {} within reach when it chose",
        was.held_all,
    );
    for held in was.held.iter().flatten() {
        detail!(
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
    say_the_frames_before();
    // And what the game's own arrays hold now, past every rule that leaves one out, which is the one question
    // this side cannot answer: the frame handed over is what the rules were given, so a hazard missing from it
    // is missing from this too.
    unsafe { game.say_what_was_near() };
}

/// How near a thing has to be to the player for a hit to name it, whatever rule the model judged it by.
const NEAR_ENOUGH_TO_BE_IT: f32 = 16.0;

/// **The 十字キー of the safest of the nine one-frame moves**, or the one towards where the rules want to be
/// where nothing is near enough to dodge.
fn steer(frame: &Frame, keys: &Keys) -> Steered {
    let at = frame.at;
    let field = frame.field;
    // **Where holding each move leads over the next [`LOOK_AHEAD`] frames**, held inside the field the way the
    // game's own clamp would hold it: a move into a wall is a move that does not happen, and scoring it as
    // though it did would pick it. Not held inside [`THE_BAND`] — that is where it *wants* to be, and a probe
    // clamped into the band is a player above the band with nine moves that all land in the same place.
    //
    // More than the next frame, because one frame of warning is not enough to be anywhere: a bullet three
    // frames off where the player stands is one to be somewhere else for, and it is *this* frame's move that
    // gets it there.
    let lands = WAYS.map(|(dx, dy)| {
        let (across, down) = if dx != 0.0 && dy != 0.0 {
            frame.diagonal
        } else {
            frame.straight
        };
        // **Clamped to the field's own edges and not to the line the bot waits on.** Holding the projection off
        // the bottom by [`ABOVE_THE_BOTTOM`] was tried, so that a dodge could not spend the floor — two hits in
        // one run were the player walking from `420` down to the clamp at `432` and sitting pinned there until
        // every one of the nine was hit on the frame it was judged on. It cost four of these tests at once:
        // keeping the player low needs the moves that reach the low line to be moves, and the longest way out of
        // a pattern coming from above is sometimes downwards. The strip below the line is a bad place to be and
        // that is not the same as a place no move may land in.
        std::array::from_fn::<_, LOOK_AHEAD, _>(|step| {
            let frames = (step + 1) as f32;
            (
                (at.0 + dx * across * frames).clamp(field.0, field.0 + field.2),
                (at.1 + dy * down * frames).clamp(field.1, field.1 + field.3),
            )
        })
    });
    // **Two boxes overlapping, which is the game's own test and not a distance**: the game makes a box of a
    // bullet's place and its own size and compares it with the player's, and no two kinds of bullet are the
    // same size — so one number standing in for every one of them is a different test rather than an
    // approximation of that one. What each move is judged by is therefore *which frame it is first hit on*, and
    // the room left over is only for the log and for choosing between moves that are all hit.
    // **How many of the frames it is hit on and which is the first**, rather than whether any of them is: a
    // player already inside something is hit on the first frame whatever it presses, so a move that has to be
    // clear all the way is a bot with nothing open to it — which falls through to the most room, and the most
    // room is the fleeing all this exists instead of. Fewest hit is a move that gets out; latest first is a move
    // that has longer before it has to.
    // **How many ways out of where a move lands there are was tried here and taken out again.** A place is
    // survivable because the frame after it has somewhere to go, so counting the moves open from each landing
    // looked like the answer to being pinned — three hits in one session were that, at the floor twice and at the
    // left wall once. As a tie-break above the room it changed which of two equally-judged moves wins, and
    // `the_bot_steps_out_of_the_way_of_a_bullet_a_step_wide_of_it` said so at once: the bot took a diagonal where
    // a step aside was what the frame was for. Whatever reaches the pinning has to leave the ordinary frame's
    // choice alone, and this did not.
    let (mine_x, mine_y) = frame.half;
    let mut counted = Counted::default();
    let mut judged = [Judged::CLEAR; WAYS.len()];
    let mut nearest = [None; WAYS.len()];
    let mut held = [None; HELD];
    let mut held_all = 0;
    // The bullets no step of their own explains, and where across the field they are — see [`a_sweep`].
    let mut swept = 0u32;
    let mut swept_x = 0.0;
    let mut kept = REMEMBERED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let last = kept.take().unwrap_or_default();
    let mut now = Remembered::default();
    // `grows` is how much a hazard's own half is taken to spread each frame, which is nought for everything a
    // step describes and its own speed for the one thing no reading does.
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
        for ((judged, nearest), path) in judged.iter_mut().zip(&mut nearest).zip(&lands) {
            for (step, (x, y)) in path.iter().enumerate() {
                let ahead = (step + 1) as f32;
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
                    judged.hit(step);
                }
            }
        }
    };
    for hazard in &frame.hazards {
        let Hazard {
            kind,
            slot,
            at: place,
            step,
            half,
        } = *hazard;
        match kind {
            Kind::Bullet { hitting } => {
                if !hitting {
                    counted.another_state += 1;
                }
                counted.bullets += 1;
                // Its own update adds its step to its position every frame, so where it will be in a frame's
                // time is where it is plus as many steps as there are frames to look.
                let step = step.unwrap_or((0.0, 0.0));
                let (moved, turn) = match what_a_slot_held(&last.bullets, slot) {
                    // **The step it claims for a bullet that was not there last frame**, so that one just made
                    // is not read as having crossed the field in a frame.
                    None => (step, (0.0, 0.0)),
                    Some((place_was, step_was)) => (
                        (place.0 - place_was.0, place.1 - place_was.1),
                        (step.0 - step_was.0, step.1 - step_was.1),
                    ),
                };
                note(&mut now.bullets, slot, (place, step));
                let scripted = (moved.0 - step.0).abs() > A_STEP_OF_ITS_OWN
                    || (moved.1 - step.1).abs() > A_STEP_OF_ITS_OWN
                    || turn.0.abs() > A_STEP_OF_ITS_OWN
                    || turn.1.abs() > A_STEP_OF_ITS_OWN;
                if scripted {
                    swept += 1;
                    swept_x += place.0;
                }
                against(
                    "a bullet",
                    slot,
                    place,
                    if scripted { (0.0, 0.0) } else { step },
                    half,
                    // **Feared and not projected.** Where a script is moving it there is nothing to project, so
                    // what is kept away from is everywhere it could reach: its box spread by the speed it just
                    // showed, for every frame looked at.
                    if scripted {
                        (moved.0.abs(), moved.1.abs())
                    } else {
                        (0.0, 0.0)
                    },
                );
            }
            // **And the bodies, which are as solid as a bullet.** Which is what a run on its way to an item
            // walks into: the item was dropped by something, and that something is still standing over it.
            //
            // **And with where it is going**, which is the difference from last frame and not a field: nothing
            // on a body is a step, its script being what moves it. A body read as standing still is one the
            // model leaves room beside for as long as it takes to arrive — a launch was hit beside 2面's
            // midboss with 4.7 of room, which is less than a body covers in a frame.
            //
            // **And only the ones with a body to run into.** The game gates its own collision on whether an
            // enemy is solid, so one with that down cannot touch the player at all — which 妖々夢's 6面 boss
            // relies on: it fires 自機狙い off an enemy its script keeps *at* the player, and a model that made
            // that solid had a hazard nothing could get away from.
            Kind::Body { solid: false } => {
                counted.no_body += 1;
                // **The nearest of them kept for the hit line**, this being the one rule here that leaves a
                // hazard out: a hit reading twelve units of room with nothing near enough to be it is a hit by
                // something the model does not have, and this is the only thing it throws away.
                let apart = (place.0 - at.0).powi(2) + (place.1 - at.1).powi(2);
                if counted.no_body_at.is_none_or(|(_, was)| apart < was) {
                    counted.no_body_at = Some((place, apart));
                }
            }
            Kind::Body { solid: true } => {
                counted.enemies += 1;
                // **And feared rather than projected where a script is swinging it.** 6面's turning beam is a
                // line of these, and a straight line from last frame's motion is wrong by the whole of the
                // sweep — so the test is whether that motion is itself changing, and what is kept away from is
                // everywhere it could reach.
                let (went, swung) = match what_a_slot_held(&last.bodies, slot) {
                    None => ((0.0, 0.0), (0.0, 0.0)),
                    Some((place_was, went_was)) => {
                        let went = (place.0 - place_was.0, place.1 - place_was.1);
                        (went, (went.0 - went_was.0, went.1 - went_was.1))
                    }
                };
                note(&mut now.bodies, slot, (place, went));
                let scripted =
                    swung.0.abs() > A_STEP_OF_ITS_OWN || swung.1.abs() > A_STEP_OF_ITS_OWN;
                against(
                    "an enemy",
                    slot,
                    place,
                    if scripted { (0.0, 0.0) } else { went },
                    half,
                    if scripted {
                        (went.0.abs(), went.1.abs())
                    } else {
                        (0.0, 0.0)
                    },
                );
            }
            // **And every place a body has been, each its own box, with [`CLEAR_OF_A_BEAM`] kept round it.** A
            // beam that follows a body is its trail, tested box by box against the player exactly as a bullet
            // is — so each of them is a hazard of that kind, and one enemy can carry a screen's worth of them.
            //
            // **Standing still, because a place already gone to does not move.** The trail holds where the
            // enemy *was*, so the shape of it is fixed in space and only its ends come and go: the projection is
            // the position itself, and nothing here has to be feared for want of a step. Nothing of it is
            // remembered either — a node is a place rather than a thing, and the slot it carries is the body's.
            Kind::Trail => {
                counted.trail += 1;
                against(
                    "a beam's node",
                    slot,
                    place,
                    (0.0, 0.0),
                    (half.0 + CLEAR_OF_A_BEAM, half.1 + CLEAR_OF_A_BEAM),
                    (0.0, 0.0),
                );
            }
        }
    }
    // **And the beams, which are the one hazard a distance cannot stand in for**: a beam is a place, not a
    // point, so a move is judged inside the beam's own frame — turn the landing about the origin by the angle
    // and it is *along* and *across*, which is the shape the game's own test makes of it.
    //
    // **A 予告線 is left for the frames it will be firing on and not for being drawn**: the game's own test is
    // gated on the beam's own countdown, so one still drawing its warning hits nothing at all — and a bot that
    // made the warning a hazard would keep the player out of the one lane a pattern leaves open. The warning is
    // still where the beam *will* be, though, which is the whole reason anybody moves off one: the frames
    // looked at that fall after it fires are frames to be elsewhere for. So the count decides, frame by frame,
    // and neither answer has to be guessed.
    //
    // The angle is the one it has now carried on by the turn it made last frame — nothing in a beam's own update
    // moves it, so one whose sweep speeds up or turns back is a beam this is late about.
    for beam in &frame.beams {
        let Beam {
            slot,
            origin: (ox, oy),
            angle,
            near,
            far,
            half_across: across,
            fires_in,
            fading,
        } = *beam;
        counted.lasers += 1;
        if fires_in > 0 {
            counted.warnings += 1;
        }
        if fading {
            counted.fading += 1;
        }
        let turn = turned(what_a_slot_held(&last.beams, slot), angle);
        note(&mut now.beams, slot, angle);
        for ((judged, nearest), path) in judged.iter_mut().zip(&mut nearest).zip(&lands) {
            for (step, (x, y)) in path.iter().enumerate() {
                // Where the beam will be pointing by then, which is where it points now plus as many of its own
                // turns as there are frames to look.
                let ahead = angle + turn * (step + 1) as f32;
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
                    // **Live by then or still only drawn.** The game's own gate is the countdown running out, so
                    // a frame this many ahead is one the beam can hit on only once it has — before that the lane
                    // is a place to be out of rather than a frame lost.
                    if (step + 1) as i32 >= fires_in {
                        judged.hit(step);
                    } else {
                        judged.warn(step);
                    }
                }
            }
        }
    }
    *kept = Some(now);
    drop(kept);
    // **The nearest to where it wants to be, of the moves nothing will be at.** Which is not the same as the
    // one furthest from everything: the next frame's positions are known, so what a move has to be is *not
    // where a bullet lands* — and keeping away from bullets besides is a bot that spends the stage leaving,
    // since the screen of a 妖々夢 stage is never empty and being near one is the ordinary state of playing.
    // **Where to be, and only now that the bullets have been walked**: which box of the field a script is
    // sweeping a beam across is one of the things that decides it, and that is a thing the walk finds.
    let sweeping = a_sweep(swept, swept_x);
    counted.swept = swept;
    counted.sweeping = sweeping;
    counted.fire = the_fire_in_each_box(frame);
    let (wanted, settled) = where_to_be(frame, sweeping);
    let mut best: Option<(usize, Judged, f32, f32)> = None;
    for (((way, judged), land), which) in WAYS.iter().zip(judged).zip(lands).zip(0..) {
        // How near this frame's own step gets to where it wants to be, which is the first of the path: the
        // frames after it are what the move is judged *hit* by and not where it is going.
        let (x, y) = land[0];
        let towards = (x - wanted.0).powi(2) + (y - wanted.1).powi(2);
        // Clear first, then nearest to where it wants to be. And where nothing is clear: **the latest first
        // hit, then fewest frames hit, then the most room** — and either over a move that merely has space
        // around it, which is the fleeing all this exists instead of. Strictly better only, so a tie goes to the
        // earlier of [`WAYS`]: standing still is first, and a bot that took the last of a tie would step
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
                // whichever comes first in [`WAYS`], a fixed lean to one side of the screen with nothing
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
                //
                // **A place lost before its frame began is not reachable from here.** Two hits in one run were
                // the player walking from `420` to the clamp at `432` and sitting there until all nine were hit
                // *on the frame they were judged on*, the room negative on every one; a third was thirty frames
                // pinned at `x = 8`. No order between the nine reaches back to the frames that chose the place,
                // and the two things tried from here — the floor outranking the frames, and the ways out of each
                // landing — each cost an existing test. What is left is where the bot *waits*, which is
                // [`where_to_be`]'s business and not this comparison's.
                //
                // **Where the rules want to be does not reach in here, and three placements said so.** The box
                // counting moves that place to the emptiest box and this branch never reads it, which is why
                // four hits in one session have the x not moving for thirty frames at an edge. But the move that
                // does the pinning is the one that wins on *when it is first hit* — going down at the floor is
                // clamped, so it stays where the fire has not arrived — and nothing ranked below that reaches
                // it. Tried under `hits()`, over `hits()`, and as the ways out of each landing: the first two
                // changed nothing and the third cost
                // `the_bot_steps_out_of_the_way_of_a_bullet_a_step_wide_of_it`. What is left is to stop the
                // place being wanted at all, which is [`where_to_be`]'s business.
                (false, false) => {
                    let sideways = way.0 != 0.0;
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
            best = Some((which, judged, way.0, towards));
        }
    }
    // **What the 低速移動 under a boss is off for is anything that reaches the move chosen** inside the frames
    // looked at, and not only what would hit it next frame: halving the step doubles the frames it takes to be
    // out from under something, so by the frame a bullet is a hit the halving has already been paid for.
    //
    // **And not a distance beside it**, which is what this was: a distance is between two centres where a hit is
    // two boxes overlapping, so a bullet 12 away is inside a 大玉's own half and outside a 米弾's — the same
    // number meaning a hit and a clear frame. [`LOOK_AHEAD`] frames of the game's own test already say whether
    // anything reaches the player, and one more margin on top of it says nothing the test does not.
    // **Whether the player has to move at all**, which is [`WAYS`]'s own first entry and not the move chosen:
    // the move chosen is clear on almost every frame, that being what choosing it means, so a slowing taken off
    // by it is a slowing never taken off. Where holding still keeps the player out of everything for the frames
    // looked at, the step is not what the frame is for and the damage is.
    let (chosen, judged_it, ..) = best.unwrap_or((0, Judged::CLEAR, 0.0, 0.0));
    Steered {
        word: keys.way(WAYS[chosen]),
        dodging: !judged[0].clear(),
        room: judged_it.clear().then_some(judged_it.room),
        for_an_item: the_item_worth_going_for(frame).is_some(),
        at,
        wanted,
        settled,
        nearest: nearest[chosen],
        counted,
        held,
        held_all,
        judgements: judged,
    }
}

/// **A beam a script is sweeping**, as the x it is spread about — or nothing where the bullets no step explains
/// are too few to be one.
///
/// A turning beam in this game is a line of bullets, and a script rather than any field of theirs is what swings
/// it: 6面's is a dozen and more of them at once. So what tells one from a bullet whose own curve the reading
/// happens to miss is how many there are, and the floor is the one the thickest box already uses — more than
/// the nine moves, which is more than a rounding difference in either.
///
/// **The count is what a launch says.** A hit's own line carries it, so a card whose every bullet reads as
/// scripted is a card this names and not one it hides.
fn a_sweep(swept: u32, swept_x: f32) -> Option<f32> {
    (swept as usize > WAYS.len()).then(|| swept_x / swept as f32)
}

/// Where the rules want the player: the x whatever is worth being under has, and the y always low.
fn where_to_be(frame: &Frame, sweep: Option<f32>) -> ((f32, f32), Settled) {
    let field = frame.field;
    let at = frame.at;
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    // The item worth going for, at the place it is; under a boss with nothing to collect the boss's own x with
    // the y low, a fight being a thing to stand under rather than to reach; and with neither the middle, low,
    // which is the shortest way to wherever the next item falls.
    let going_for = the_item_worth_going_for(frame);
    let (settled, (x, y)) = match going_for {
        Some((worth, at)) => (Settled::AnItem(worth), at),
        None => match frame.boss {
            Some((x, _)) => (Settled::ABoss, (x, low)),
            None => (Settled::TheMiddle, (field.0 + field.2 / 2.0, low)),
        },
    };
    // **And out of a box of the field the fire is clearly piled into**, whichever of them that x is in.
    //
    // Which is the one thing the nine moves cannot say between them: each asks whether *it* is hit over the
    // frames looked at, so the bot will stand in the thick of a pattern for as long as the next twelve frames
    // happen to be survivable and be somewhere with nothing open the moment they are not. That is what its hits
    // are — `nowhere open to it` with a model holding 123 bullets and nine bodies is a place chosen and not a
    // hazard missed.
    //
    // **Unless what it is going for is worth more than a point item.** Being crushed costs a bomb at best and
    // the run at worst, which a point or a cherry is worth neither of; a 1up, a bomb, or the eight steps of
    // power a big power item carries is worth the crossing. So the two rules meet at [`worth`], which is the
    // one place a pickup is ranked.
    let worth_the_crossing = going_for.is_some_and(|(it, _)| worth(it) > worth(Worth::Point));
    // **And out of the box a script is sweeping a beam across, whatever is in it.** Which the thickness cannot
    // stand in for: a beam is a dozen bullets rather than a hundred, so the box it is crossing is rarely the
    // thickest — and nothing about a swept bullet says where it will be, so the frames looked at are no answer
    // either. **Not weighed against an item the way the thickness is**, because the two are not the same bet: a
    // box full of fire is a place a run of good frames gets through, and a beam arriving is arriving.
    // **And the box the player is standing in, not only the one it is heading for.** Following a boss across a
    // box is how it gets into one: the question was put about the wanted x alone, so a boss drifting over the
    // pile was a boss the bot followed in with the ground under its own feet never asked about. A practice was
    // hit at `138.0,409.5` with the boxes holding `15.0, 218.7, 58.0, 13.5, 17.5` — forty frames walking from
    // `164` to `132`, out of the box the boss was in and into the fullest on the screen, the emptiest the other
    // way.
    let swept = sweep.map(|it| which_box(field, it));
    let mine = which_box(field, at.0);
    let heading = which_box(field, x);
    let fire = the_fire_in_each_box(frame);
    // **And out of the one it is standing in where that one is badly worse**, which takes more than not walking
    // into a pile does: see [`MUCH_MORE_THAN_A_HANDFUL`].
    let badly = clearly_fuller(&fire, mine, MUCH_MORE_THAN_A_HANDFUL);
    let elsewhere = || {
        let empty = the_emptiest_box(&fire);
        field.0 + field.2 * (empty as f32 + 0.5) / BOXES_ACROSS as f32
    };
    let (settled, x) = match the_thickest_box(&fire) {
        _ if swept.is_some_and(|swept| swept == heading || swept == mine) => {
            (Settled::TheEmptiestBox, elsewhere())
        }
        _ if badly => (Settled::TheEmptiestBox, elsewhere()),
        Some(thick) if !worth_the_crossing && thick == heading => {
            (Settled::TheEmptiestBox, elsewhere())
        }
        _ => (settled, x),
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
        (
            if inside <= outside {
                x.clamp(inside, outside)
            } else {
                field.0 + field.2 / 2.0
            },
            if y >= the_band(field) { y } else { low },
        ),
        settled,
    )
}

/// How many pieces the field is cut into across its width for the fire in each to be counted.
///
/// **An odd cut, so that the middle has a box of its own.** The place a pattern converges on is the middle —
/// 自機狙い goes there and so does most of what a boss fires — and it is discounted for that by
/// [`IN_THE_MIDDLE_ANYWAY`]; an even cut puts that place on a boundary, where the discount lands on neither
/// side of it. Five rather than three is somebody who plays this game asking for a finer answer to *where to
/// wait*: at a third of a 384-wide field, the emptiest box is 128 across and standing in the middle of it is
/// standing 64 from fire either side.
const BOXES_ACROSS: usize = 5;

/// **The room the bot keeps on every side of each place a beam-carrying body has been**, on top of the box that
/// place is tested as.
///
/// The box itself is the body's own size as the game hands it to the test, which for 6面's spiral is a couple of
/// units — so a beam a few units off the player is one every move reads as clear on the frame before it is not,
/// and the trail's own boxes are the only hazard in this game a *line* is made of. Grazing a line is not the
/// same trade as grazing a bullet: a bullet is gone next frame and the line is where the body will lay the next
/// node.
///
/// **A judgement, and what it replaced.** The trail used to be weighted heavily in [`the_fire_in_each_box`]
/// instead, at twelve bullets a node — which answers which of the five boxes to *wait* in and says nothing about
/// how near the beam the bot may stand inside one. Six hits across four practices were inside a spiral that the
/// counting had already sent it away from.
///
/// **Doubled from twelve**, which was about the spacing the tested nodes have — the game tests every sixth of
/// them — and so the distance that makes the line solid rather than a row of dots with gaps to squeeze through.
/// That much is what the line *is*; this much is room to be wrong about where it will be next, which a line laid
/// down by something moving needs and a bullet does not.
const CLEAR_OF_A_BEAM: f32 = 24.0;

/// **The smallest bullet 妖々夢 fires, which is the unit the fire in a box is counted in.**
///
/// The sizes are written at spawn and there are six of them, from 4 up to 24 — so counting bullets one apiece
/// asks how many heads are in a box where what decides whether there is a way through is how much of its width
/// they take. A 大玉 is six of these across, and three of them across a box is a wall where three 米弾 is a gap.
const A_SMALL_ONE: f32 = 4.0;

/// **How much of a box one hazard of a given size fills**, in [`A_SMALL_ONE`]s squared.
///
/// **Squared, because what a box holds is an area.** A bullet twice as wide takes four times the room to weave
/// past, not twice: it blocks that much of the box in both directions at once, and a 大玉 is six of the smallest
/// across. Counted by the width alone, a handful of the big ones read as a handful and the box they filled read
/// as passable.
///
/// **And never less than one of them**, which is not a guard against a bad reading: nothing this game fires is
/// narrower than the smallest it fires, so a size below that is a size that has not been written yet — a slot
/// filled in the frame it spawns on. Weighted as it stands, such a hazard would count for nothing at all, and a
/// pattern read as it arrives is the one worth leaving the box for.
fn how_much_it_fills(half: f32) -> f32 {
    let across = (half * 2.0).max(A_SMALL_ONE) / A_SMALL_ONE;
    across * across
}

/// **What the middle box's count is multiplied by before the boxes are compared.**
///
/// The middle of the field is where a pattern is thickest by nature — 自機狙い converges there and so does most
/// of what a boss fires — so counting it against the outer boxes as it stands is a bot that leaves the middle for
/// an imbalance that was always going to be there.
///
/// **Raised once, from four fifths.** At that, the discount was doing more than answering the converging: the
/// middle read as emptier than boxes holding less, and the bot went back to it over places that were genuinely
/// clearer. Nine tenths is what somebody who plays this game asked for instead.
const IN_THE_MIDDLE_ANYWAY: f32 = 0.9;

/// And the two boxes either side of it, by the same reasoning and less of it: the converging falls off with the
/// distance from the middle rather than stopping where a box does. Raised with the one above, from nine tenths.
const BESIDE_THE_MIDDLE_ANYWAY: f32 = 0.95;

/// Which of the [`BOXES_ACROSS`] a place across the field is in.
fn which_box(field: (f32, f32, f32, f32), x: f32) -> usize {
    let across = ((x - field.0) / field.2 * BOXES_ACROSS as f32).floor();
    (across.max(0.0) as usize).min(BOXES_ACROSS - 1)
}

/// **How much fire each of the [`BOXES_ACROSS`] holds**, the middle one's counted at [`IN_THE_MIDDLE_ANYWAY`].
fn the_fire_in_each_box(frame: &Frame) -> [f32; BOXES_ACROSS] {
    let field = frame.field;
    // **And nothing below the player**, whatever box it is in: what a box is counted for is where to wait, and
    // what makes a box a bad place to wait is the fire that can still arrive. Fire below has gone past, so
    // counting it fills the box the bot is standing in with what has already missed — the box reads worst just as
    // it becomes safe, and the box that is filling up stands out only once the fire is level with the player.
    // Three hits in one run were pinned at an edge with every other box reading `0.0`.
    let above = frame.at.1;
    let mut fire = [0.0; BOXES_ACROSS];
    for hazard in &frame.hazards {
        // **The bullets and the beams' own nodes, and nothing that is only a body.** A body is a thing to dodge
        // where it stands rather than fire filling a place, and a boss is one of them.
        let counts = match hazard.kind {
            Kind::Bullet { .. } | Kind::Trail => true,
            Kind::Body { .. } => false,
        };
        if !counts {
            continue;
        }
        // **Only what is inside the band the player can be in.** The bot never rises above [`THE_BAND`], so
        // fire above it is fire nothing it does can put it among — counting the whole column of each box
        // mixed that in and buried the difference the boxes are compared for: a run was hit in the middle of
        // the field with the middle thick, and the counting could not see it.
        //
        // **Where it is going and not only where it is, for anything with a step.** A node of a trail is a place
        // the enemy has already been and it stays there; what moves is the enemy writing new ones, and the box
        // it will write into next is the box it is over.
        let Some((x, _)) = where_it_comes_into(hazard, field, above) else {
            continue;
        };
        fire[which_box(field, x)] += how_much_it_fills(hazard.half.0);
    }
    // **The middle discounted, and the two beside it discounted less.** The converging a pattern does is a thing
    // of the middle of the field, and it falls off either side of it rather than stopping at a box's edge.
    let middle = BOXES_ACROSS / 2;
    fire[middle] *= IN_THE_MIDDLE_ANYWAY;
    fire[middle - 1] *= BESIDE_THE_MIDDLE_ANYWAY;
    fire[middle + 1] *= BESIDE_THE_MIDDLE_ANYWAY;
    fire
}

/// **How far ahead the boxes are counted**, in frames.
///
/// **Because going to a box costs frames, and the count is what decides to go.** The crossing the box rule asks
/// for is up to two boxes — 154 of the game's units across a field of 384 — which at a step of a little over four
/// is most of a second. Counted at this instant the answer describes the field being left rather than the one
/// being arrived in: a wall a third of a second above the band reads as an empty box to stand under, and by the
/// time it reads as fire there is no longer room for the crossing that would have answered it. Four hits in one
/// session were that lag, each one a rule firing correctly and too late.
///
/// **Not the seconds it takes a pattern to develop**, which is what it would be if the number were about the
/// game: at two seconds every bullet on the screen has left the field and the boxes all read empty.
const THE_FRAMES_AHEAD: usize = 40;

/// **Where a hazard first comes into the window the boxes are counted over**, within [`THE_FRAMES_AHEAD`], or
/// nothing where it never does.
///
/// The window is [`the_counted_band`] down to `above`, the player's own row. Fire already inside it comes back at
/// the place it is now, so what this adds is the fire on its way in — counted in the box it will be in when it
/// gets there rather than the box it is over on the way.
///
/// **Below the player is out and stays out.** That is fire which has gone past, and a step carried forward does
/// not bring it back: a bullet under the player is one whose whole future is further under it.
fn where_it_comes_into(
    hazard: &Hazard,
    field: (f32, f32, f32, f32),
    above: f32,
) -> Option<(f32, f32)> {
    let band = the_counted_band(field);
    let (at, step) = (hazard.at, hazard.step.unwrap_or((0.0, 0.0)));
    (0..=THE_FRAMES_AHEAD).find_map(|frame| {
        let (x, y) = (at.0 + step.0 * frame as f32, at.1 + step.1 * frame as f32);
        (y >= band && y <= above).then_some((x, y))
    })
}

/// **The box of the field the fire is clearly piled into**, or nothing where none of them is.
///
/// **Measured against the emptiest of them and not against the screen.** Asking whether one box holds
/// more than half of everything is asking the wrong question: a run was hit in the middle with the left
/// obviously empty, and the middle still held under half because the right held the rest. What matters is that
/// there is somewhere clearly better to be.
///
/// Clearly, and said in two numbers rather than a taste: **twice the emptiest**, which is a factor and so means
/// the same at thirty bullets as at three hundred; and **more of a difference than there are moves to weave
/// between**, so that a handful is not a pile. An ordinary imbalance is not a reason to leave a fight.
fn the_thickest_box(fire: &[f32; BOXES_ACROSS]) -> Option<usize> {
    (0..BOXES_ACROSS).find(|box_| clearly_fuller(fire, *box_, MORE_THAN_A_HANDFUL))
}

/// **How much more than the emptiest box a box has to hold to be one worth leaving**, in [`A_SMALL_ONE`]s.
///
/// Enough of a difference that a handful is not a pile: an ordinary imbalance is not a reason to leave a fight,
/// and every box but one is fuller than the emptiest by something.
///
/// **Lowered once, from the whole of [`WAYS`].** Half of them is what somebody who plays this game asked for
/// after a run left too late twice over.
const MORE_THAN_A_HANDFUL: f32 = WAYS.len() as f32 / 2.0;

/// And how much more it takes to leave the box the player is already standing in.
///
/// **Because leaving that one costs the damage it is there for.** Not walking into a pile is free; walking out of
/// one is walking out from under the boss, and a fight nobody shoots is a fight that does not end — 6面's own
/// practice runs for twenty-seven thousand frames as it is. So the bar is the one above raised, and dodging comes
/// before shooting only where the box is worse than somewhere else by more than an imbalance.
///
/// **A judgement, and it has been lowered once**: at four times a handful the bot was read as leaving too late,
/// and twice is what somebody who plays this game asked for instead.
const MUCH_MORE_THAN_A_HANDFUL: f32 = 2.0 * MORE_THAN_A_HANDFUL;

/// **And the factor beside the difference: how many times the emptiest a box has to hold.**
///
/// A factor means the same at thirty bullets as at three hundred, where the difference alone would leave a
/// crowded screen with no box worth leaving. **Lowered twice**, from two and then from one and a half: a run sat
/// in a box holding well over what the emptiest did and left only when it was much too late to matter.
const TIMES_THE_EMPTIEST: f32 = 1.25;

/// Whether one box holds clearly more than the emptiest of them: [`TIMES_THE_EMPTIEST`] of it, and more than
/// `enough` beyond it.
///
/// Both, because either alone is wrong at one end of the range: the factor says nothing where the emptiest is
/// empty, and the difference says nothing where every box is full.
fn clearly_fuller(fire: &[f32; BOXES_ACROSS], box_: usize, enough: f32) -> bool {
    let empty = fire.iter().copied().fold(f32::INFINITY, f32::min);
    fire[box_] > empty * TIMES_THE_EMPTIEST && fire[box_] - empty > enough
}

/// And the emptiest of them, which is where the bot waits instead.
fn the_emptiest_box(fire: &[f32; BOXES_ACROSS]) -> usize {
    (0..BOXES_ACROSS)
        .min_by(|a, b| fire[*a].total_cmp(&fire[*b]))
        .unwrap_or(BOXES_ACROSS / 2)
}

/// The top of the band the bot moves in, which is the bottom [`THE_BAND`] of the field.
fn the_band(field: (f32, f32, f32, f32)) -> f32 {
    field.1 + field.3 * (1.0 - THE_BAND)
}

/// **How much of the field, up from its bottom edge, the fire in a box is counted over** — three fifths.
///
/// Deeper than [`THE_BAND`] the bot moves in, and for a reason of its own: what decides where to *wait* is what
/// is coming, and it comes from above the band. **Raised twice**, from a third and then from a half: a run was
/// hit boxed in at the bottom right with the bottom left plainly empty, and what closed on it was a band of fire
/// that had not reached the bottom third yet — counted only there, the box it stood in read as clear enough to
/// wait in until the fire arrived.
const COUNTED_BAND: f32 = 3.0 / 5.0;

/// The y above which fire is not counted towards a box.
fn the_counted_band(field: (f32, f32, f32, f32)) -> f32 {
    field.1 + field.3 * (1.0 - COUNTED_BAND)
}

/// **What each kind of pickup is worth going for**, largest first — a life, a bomb, the big power item, the small
/// one, the point item, the cherry, and last anything the six of them do not name.
///
/// Which is 妖々夢's own worth of them and near enough every game's: a life is a life, a bomb is a life this
/// bot's 決死結界 will spend, and a big power item is eight steps of power against a small one's one. **Nothing
/// scores zero**, so no item on the screen is walked past — a kind with no place of its own is still worth the
/// step it costs.
const fn worth(kind: Worth) -> u8 {
    match kind {
        Worth::Life => 6,
        Worth::Bomb => 5,
        Worth::BigPower => 4,
        Worth::Power => 3,
        Worth::Point => 2,
        Worth::Cherry => 1,
        Worth::Other => 0,
    }
}

/// The item worth going for, or `None` where the screen is empty.
///
/// **The worth first and the distance only inside it**, which is what makes a big power item across the field
/// beat a small one under the player's hand. Between two of a kind it is the nearer.
fn the_item_worth_going_for(frame: &Frame) -> Option<(Worth, (f32, f32))> {
    let (at_x, at_y) = frame.at;
    frame
        .items
        .iter()
        .map(|item| {
            (
                item.worth,
                (item.at.0 - at_x).powi(2) + (item.at.1 - at_y).powi(2),
                item.at,
            )
        })
        // **The kind and not the worth**, so that [`worth`] is the one place a pickup is ranked and a hit's own
        // line can say which item a run was going for.
        .max_by(|(ours, ourm, _), (theirs, theirm, _)| {
            worth(*ours)
                .cmp(&worth(*theirs))
                .then(theirm.total_cmp(ourm))
        })
        .map(|(kind, _, at)| (kind, at))
}
