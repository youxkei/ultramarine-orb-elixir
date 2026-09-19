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
//!   gets one — and in every [`SHOT_LET_GO_EVERY_IN_A_CONVERSATION`] while a line is up, for the game whose
//!   skip key orb has no binding for — and the skip key while a dialogue is up
//! - the bomb the frame the player is hit and on no other frame at all, which is a deathbomb —
//!   [`Frame::hit`] stands for the window the game counts down, and a bomb inside it is a death taken back.
//!   A bomb spent before a hit buys nothing that one does and takes the stock the window needs, which is
//!   said against the record of the frames it used to be spent on
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
//! the screen of a th07 stage never being empty; and reading the bullets only once one is *near* is a bot that
//! walks into one on its way to an item, which two launches did. What the next frames' positions are for is
//! knowing which moves are open, and among those the one that gets on with it.
//!
//! Nothing here presses for a small enemy: the shot is held on every frame already, and what a killed one drops falls
//! to whoever is under it.
//!
//! **Which is not a claim that they clear the game.** Nothing laid out by hand can say that; a launch says it,
//! and what a launch that dies says is which rule to look at. So the rules go in one at a time, each with the
//! e2e test that pins it — `orb-e2e`'s `the_bot_playing_th07`.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};

use orb_config::recipe::Dodge;
use orb_core::detail;
use orb_core::game::{Beam, FIRES_WHEN_NOTHING_SAYS, Frame, Game, Hazard, Kind, Worth};

use crate::hits::Hazards;

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
///
/// **And a count of the fire by depth was written to choose this instead of fixing it, and taken out
/// again.** The ground from [`the_band`]'s top to here cut into three rows, the fire coming into the
/// player's own column counted by the row it arrives in, a shallower row wanted where the deep line held
/// clearly more of it — [`clearly_fuller`]'s own bar, so that one bullet moved nothing, and
/// `a_bullets_own_size_is_part_of_what_a_hit_is` and `a_chapter_drawn_to_be_hit_in_is_not_dodged_in` are
/// the two frames that set that bar and the gate that only what is above the player counts.
///
/// **It fires, and the attempt with it in was worse.** One th06c Extra attempt moved the wait `1,161`
/// times and was hit five times, four of them outside the band where three of five had been — one at
/// `49.3,16.0`, the very top of the field, where the worst before it was `192`; and the floor stayed where
/// it was, `8.0,432.0` being that attempt's first hit. **One attempt against one attempt is inside the
/// noise**, though, which a later pair measured: an attempt whose only change was a rule that then never
/// fired was hit six times against a four, so the spread on one build is at least `4` to `7`. What holds
/// this out is the arithmetic rather than that count — it moves the wait *shallower*, `368` to `331` on
/// that game's field, leaving `37` units between the wait and the band's own top where there were `74`, so
/// a climb out of the band is half as long to make.
///
/// **Which bounds the wanting the way the nine's own ranking is bounded.** Deeper is the clamp, shallower
/// is the band's top, and this sits between them: the hits are dodges ending at one end or the other, and
/// where a dodge goes is not what the wanting decides.
const ABOVE_THE_BOTTOM: f32 = 64.0;

/// And how far in from either side edge the x the rules want is held, in the same units and for the same
/// reason: a corner is where a dodge runs out of moves, three of the nine leading into the wall where the clamp
/// makes them the move the player is already making.
///
/// **Which a run paid for.** th07's stage 4 has three bosses and one of them stands at the left edge; a run that
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
/// th07's collection at the top of the field, which needs the whole climb; what it buys is a run that is
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
/// **The game's own test agrees**: th07 compares the edges inclusively at 0x43e2e7 where this one is strict,
/// so nought of room is already a hit there and never was room at all.
///
/// Two, which is more than the largest margin a hit has been taken on and half of one unfocused step — so a
/// lane the player can actually use is still a lane.
///
/// **And one of the things that projection was out by has since been named**: the player was projected where it
/// stands against a hazard a step on — see [`THE_NEXT_FRAME_AND_LOOK_AHEAD`] — which credits a move with a
/// whole step of room the frame does not have. Whether this can come down now that the two are paired is a run
/// nobody has made, and it is worth making: a slack larger than it needs to be is a lane a dense pattern leaves
/// that the rules read as shut, and th06c's Extra boss puts 207 bullets and eight beams up at once.
const ROOM_TO_SPARE: f32 = 2.0;

/// How many frames of holding a move are looked at before it is chosen.
///
/// One is not enough: a bullet a frame away is one there is nowhere left to be for, and what a move has to
/// answer is where holding it *leads*.
///
/// **And the number is set by the widest bullet in the game.** To be out from under one the player has to have
/// travelled that bullet's half and its own on top, which is `(12 + 2) / a step` frames — four at the step
/// th07 works out and eight at the halved one the focused step leaves, its widest bullet being 24 across. A
/// move first seen three frames out is therefore one there is no getting away from whatever is pressed, and
/// the aimed shot is what makes it: a launch spent stage 6's boss dying to the same aimed large bullet over
/// and over, 30 hits at one chapter. Twelve is those eight and room to pick a direction in rather than having
/// to move the frame the bullet appears; past that a bullet's own step stops describing where it goes, and
/// nothing here reads what turns one.
///
/// **And it is [`THE_FRAMES_AHEAD`] now, which is the only thing that has moved where a run is hit.** A move
/// clear for ten frames of thirteen is one that window cannot tell a dead end from a way through, and
/// th06c's Extra boss lays arms across the band that leave the three upward moves and nothing else: the frame
/// one attempt left the band on, dumped at `76.3,292.2`, has `left` clear for ten of the thirteen and hit on
/// the eleventh. Counting the hits outside [`the_band`] or at the clamp against every hit an attempt took,
/// two attempts a build: **`16` of twenty-one at twelve frames, over every other build of one session,
/// against `19` of forty-one at forty over five attempts** — and the hits it leaves inside are at the
/// waiting line rather than above the band — `342.3,367.4`,
/// `330.3,368.9`, `306.3,366.5`. The judgement those attempts were made against is an attempt with none
/// outside, and no attempt of either walk has reached it.
///
/// **What it changes besides is which branch the ranking spends its frames in.** A move is hit where its room
/// falls under [`ROOM_TO_SPARE`] on any frame looked at, and over forty frames of a dense screen something
/// always arrives: a hit at `8.0,421.7` has standing still `hit first on 16` with `1.2 of room`, where the
/// same frame at thirteen had a way out. So the branch where every one of the nine is hit becomes the
/// ordinary branch, and that is the branch [`where_to_be`]'s answer reaches only as a tie-break.
///
/// **And that is where the walk's gain comes from, which asking the three tiers of the first thirteen frames
/// instead is what measured.** [`Judged::clear`] over that window and the room it is reported with over the
/// same one — the two have to agree, or a move called open carries the margin of a frame thirty out — puts
/// every tier back where it was measured, and the run is worse for it: two Extra attempts took `5` hits
/// outside [`the_band`] or at the clamp of eight apiece, `10` of sixteen, against `12` of twenty-six for the
/// three attempts that asked the tiers of the whole walk. So the lengthening is not a longer look with the
/// old judgement behind it; the collapse of the three tiers into the last of them **is** the change, and the
/// order inside that branch — `first()`, then the hit count, then sideways and the room — is what a run of
/// this fight is now steered by.
///
/// **And what it cost to get here was two readings and a recipe's own window.** [`Judged::on`] was a `u32`
/// and the walk it holds a bit per frame of was thirteen, so forty shifted past the end of the mask —
/// silently in a release build, and the first launch of this was measured on a judgement whose thirty-third
/// frame read as its first. `the_fine_step_is_taken_where_the_whole_one_finds_nothing_open` is a ring that
/// has to surround the player inside the play field, which the waiting line leaves `64` of room beneath.
/// And `orb-real`'s `a_death_answered_by_the_chapter_again` ends at the chapter after the one its death falls
/// in, which is a thing of how well the rules play: a longer walk chooses a different nearest thing to be hit
/// by and carried the laid-out game's death from its second chapter to its third. The frame's own time is not
/// among the costs — a launch reads `on screen from 1648us to 4669us` against `1394us to 3555us` at twelve.
const LOOK_AHEAD: usize = THE_FRAMES_AHEAD - 1;

/// **What the rules made of a frame**: the d-pad to press, whether the step must not be halved, the room the
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
    /// **And half the player's own box**, which is the other half of every comparison the nine moves are judged
    /// by: a hit reported beside a hazard's own half says what that hazard covered and not what the thing it hit
    /// covers, so a box read too small is a run hit with room to spare on every line of its dump and nothing in
    /// them to say so.
    half: (f32, f32),
    /// **Whether the move it chose is hit at all inside the frames looked at**, which — the move chosen
    /// being the best of the nine — is nothing being open to it.
    ///
    /// **And its own next frame left less room than [`ROOM_A_BOMB_IS_OWED_AT`]**: those are the two
    /// halves of *cornered*.
    ///
    /// **Not an overlap, and not the slack either.** Nought of room on the next frame was tried and is
    /// too late — th06c's Extra boss took a run's first hit on a frame with three bombs in hand,
    /// `nowhere open to it` on all nine moves and the best of them `0.1 of room` on its own next frame,
    /// a tenth of a unit and a bomb never spent. [`ROOM_TO_SPARE`] was tried next and is too eager, for
    /// the reason and with the numbers that constant's own doc has.
    ///
    /// **Which is what the record of such a frame is for, and it is held apart by [`BOMBS_APART`] rather
    /// than by being rare.**
    /// Two narrower readings were tried and each was too late: the frame the boxes already overlap on is
    /// a frame the game's own test has run on, and three frames of warning did not save a player with
    /// seven bullets inside sixteen of it, each of them falling two a frame. **And the wider one spent the
    /// run's three bombs in its first two stages**, where a run has three for the whole of it and no more
    /// until a death: `nowhere open to it` alone is the judgement's own pessimism, two units of slack over
    /// thirteen frames, and stage 1 of Normal is full of frames that read that way and are nothing.
    ///
    /// **The next frame's own room and not the least of the thirteen**, which was the reading between
    /// those two and is why a stage 5 run was hit with two bombs in hand: every one of the nine moves was
    /// hit and the best of them read `0.1 of room`, that `0.1` being some frame ten ahead — while the
    /// move's own next frame overlapped by `0.03`, which is the hit. A box eleven frames out is a box
    /// eleven more frames of choosing can still be moved out of the way of.
    hit_at_once: bool,
    /// **And how much room that move had there**, which is what says whether the frame really was lost —
    /// see the line such a frame writes, and [`ROOM_TO_SPARE`], which is the slack a *clear* move is judged
    /// with rather than a hit.
    next_frame: f32,
    /// **And the step the twelve frames were projected with**, which is the other half of *where* they were
    /// projected to.
    ///
    /// **Because a hit with four of room is either that step or nothing.** Six dumps of one Extra run and two
    /// of a Lunatic one hold the same shape: a bullet four to one side of the player and seven above it,
    /// falling faster than the player's own diagonal, and the diagonal away from it judged clear for all
    /// twelve frames. That judgement comes out of the arithmetic only with the *unfocused* step — 2.83 a
    /// frame against the bullet's 2.9, which keeps the gap — and the hit comes out of it only with the
    /// focused one, 1.41, which loses 1.49 a frame and closes seven in five. So the two numbers are the
    /// question, and `Game::what_a_bot_sees` answers them off the focus bit of the word the run *last* acted
    /// on.
    step: ((f32, f32), (f32, f32)),
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
    /// **And every beam as the judgement read it** — see [`HELD_BEAMS`].
    beams: [Option<Beam>; HELD_BEAMS],
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
    /// **And the nearest thing that can hit the player**, which is a chapter drawn to be hit in and not hit
    /// yet — see [`crate::hits::Hazards::GoneTo`].
    SomethingThatCanHitIt,
    /// The x a recipe told the rules to stand at — see [`TOLD_TO_STAND_AT`].
    ThePlaceItWasTold,
    /// The place the way a recipe named for the card that is up puts the rules — see [`DODGE`].
    TheWayItWasToldToDodge,
    /// **The nearest way out of a beam the run is standing in**, which is the one errand that outranks
    /// every other — see [`the_way_out_of_a_beam`].
    OutOfABeam,
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
            Self::SomethingThatCanHitIt => "something that can hit it",
            Self::ThePlaceItWasTold => "the place it was told to stand",
            Self::TheWayItWasToldToDodge => "the place the card's own dodge puts it",
            Self::OutOfABeam => "the way out of the beam it is in",
        }
    }
}

/// **What the frame's model was made of, and what it left out** — bullets, bodies and beams, each as the
/// number judged against and the number passed over for not being able to hit anything.
///
/// Which is the one thing a hit saying `14.5 of room` needs and no other field of [`Steered`] can give: a model
/// that says a move is clear by fourteen units and is wrong is a model missing a hazard, and the passing over
/// rules are where a missing hazard would have gone. A run cleared the game with eight hits on stage 6's boss
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
    /// **And the widest body a frame held, as its slot and its own half** — which is what a hit with a
    /// room of `-1448.7` wants said: a box that size is a reading and not an enemy, and the slot is what
    /// a debugger needs to go and look at the one that was handed over. A card that stands a grid of cells up
    /// put one of `2408.0,2502.2` on the field, ten times the field across, and the rules had nowhere open
    /// for as long as it stood there.
    widest_body: Option<(usize, (f32, f32))>,
    /// **And how many body slots hold an enemy other than the one they held last frame**, which is the
    /// number that tells a wide box handed over apart from a wide box the model made of two enemies —
    /// see `game::Hazard::frames`. The same card's cells are born and killed every few frames, and the
    /// half a hit named was `8.0` grown over one lattice step of the grid rather than anything the game
    /// put on the field.
    refilled: u32,
    /// And the same for the bullet slots, whose own count is read off `Th08::BULLET_FRAMES`: a bullet is
    /// differenced against its slot as well, the rules reading one that moved otherwise than its step says
    /// as one a script is moving and fearing the box it swept.
    refilled_bullets: u32,
    lasers: u32,
    /// Of those, how many had not started hitting yet — a warning line, judged for where its beam will be.
    warnings: u32,
    /// And how many of them are fading. Judged like any other, and worth saying because what a game takes down
    /// while a beam goes out is that game's own — th06c's takes the length and not the width — so a beam left
    /// too eagerly or crossed too readily is this number standing where the room should be.
    ///
    /// It was the count of lasers *skipped* for their phase, and the skip was the wrong walk's gate — see the
    /// seam's `Beam::fading`.
    fading: u32,
    /// And how many bullets no step of their own explains, with the x they are spread about where those are
    /// enough to be a beam a script is sweeping — see [`a_sweep`].
    swept: u32,
    sweeping: Option<f32>,
    /// **And the widest of those disagreements, as what the bullet moved against what it said it would.**
    ///
    /// **Which is the one thing the count cannot say and the whole of what to do about it turns on.** A
    /// count of bullets that moved otherwise is the same number whether the seam's step is out by a
    /// factor, is being read a frame late, or describes a bullet a script re-aims — and those are three
    /// different repairs. A launch of th06c read `189 bullet(s) no step of their own explained` out of
    /// 189, so every one of them was feared over the box it had swept and not one was projected; what
    /// this says is which of the three that was.
    out_by: Option<OutBy>,
    /// And how many boxes the enemies' own trails put in, which is the beams that follow them.
    trail: u32,
    /// And the fire each box across the field was counted as holding, which is what the place to wait was chosen
    /// by — see [`the_fire_in_each_box`].
    fire: [f32; BOXES_ACROSS],
    /// And whether that fire was counted under a boss overhead, which is what it was counted by — see
    /// [`the_boss_is_overhead`].
    overhead: bool,
    /// **And the frame a beam will be lying across the band in each of those boxes on**, which the fire count
    /// has no shape for — [`THE_FRAMES_AHEAD`] for a box none of them reaches inside the window. See the loop
    /// in [`steer`] that answers it.
    closes: [u32; BOXES_ACROSS],
    /// **And the nearest beam's own box, as the slot, how far outside it the player is standing and how far
    /// along it that is.** Nought or less says the run was inside a beam the model was holding, which is a
    /// dodge the rules did not make rather than a reading they did not have; a large one says the beams had
    /// no part in the hit. The same job the nearest bodiless enemy does for the enemies.
    ///
    /// **And whether that beam was firing**, since a box the player is inside takes nothing where the lane
    /// has not gone live.
    beam_at: Option<(usize, f32, f32, bool)>,
}

/// **One bullet's disagreement between what it moved and what its step said** — see [`Counted::out_by`], which
/// keeps the widest of them.
///
/// **With the game's own number for the state that step was scaled by**, which is what a scale is picked by:
/// that number is what says whether a factor is one of the game's own branches read wrongly or something no
/// branch accounts for. th06c's five are `1.0`, `1/2`, `1/2.5`, `1/3` and `1/2`, and no ratio between two of
/// them is the `1.4` one of its bullets moved by.
#[derive(Clone, Copy)]
struct OutBy {
    moved: (f32, f32),
    step: (f32, f32),
    state: u16,
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
    /// **And the half the frame it came nearest on was judged against, where the two differ** — which is the
    /// box feared rather than the box read, for a hazard a script is moving: see the `grows` argument of the
    /// judging closure in [`steer`].
    ///
    /// **Because the two read alike on one line and one of them is the game's.** Five hits of one th08 Normal
    /// run named `an enemy ... half of it 2408.0,4164.9` while the same dump's widest *read* body was
    /// `16.0,10.7`, and a box ten times the field is a thing to go and look for in a debugger where a box
    /// feared out to ten times the field is a thing to fix in the fear.
    feared: Option<(f32, f32)>,
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
/// th07's own bullet hell would put a hundred of these in it.
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

/// **How many beams the judgement's own reading of them is kept for**, which is [`Held`]'s reason applied to
/// the one hazard [`Held`] cannot carry: a beam is a place rather than a point, so it is never *within reach*
/// of the player and never held.
///
/// **And what the walk beside a hit shows is not what the judgement had.** A launch of th06c on Extra was hit
/// with `0.1 of room`, the nearest beam `12.0` across in the judgement and `12.0` wide in the walk that
/// followed — half of which is `6.0`. Two different repairs read the same: a width measured wrong, or a beam
/// whose width, origin or very existence moved between the read and the walk. th06's own lasers arrive one a
/// frame in a sweep, so the second is not a remote possibility, and no field of [`Steered`] told them apart.
///
/// Sixteen, against the nine th06c's Extra boss had up at once.
const HELD_BEAMS: usize = 16;

/// **What a move came to**: which of the frames looked at it is hit on, and the least room any of them had.
///
/// The frames as a set of bits rather than a count, so that two bullets reaching the same frame is one frame
/// hit and not two — what a move is ranked by is how many frames of it are lost, and the first one lost.
///
/// **The room between the boxes and not between the middles.** A hit is two boxes overlapping, so what says how
/// near a move came to being one is what was left on the tighter of its two axes; a distance between middles
/// puts a small bullet six away ahead of a large one seventeen away, when it is the large one that is one unit
/// from a hit. Which is not only a number in the log: it is what picks between moves that are all hit, and what
/// the log names as the hazard to look at.
///
/// **And which of them a beam will only be drawing its warning on**, which is a frame to be elsewhere for and
/// not a frame that is lost. The game's own test is gated on the beam's own countdown, so a warning cannot hit
/// anything at all — a move into one is worse than a move out of one and better than a move into a beam that is
/// live. Counted apart from [`Judged::on`] because a card that fans twenty-two warnings out of one point shuts
/// every one of the nine where the two are the same, and a bot with nothing open to it stands where it is until
/// the beams fire.
#[derive(Clone, Copy)]
struct Judged {
    /// **A bit per frame of [`THE_NEXT_FRAME_AND_LOOK_AHEAD`], so the width is what bounds the look-ahead.**
    /// At `u32` a walk of forty frames shifts past the end of it, which a release build does silently: the
    /// frames past the thirty-second come back as the low ones, so a move hit on the thirty-third reads as
    /// hit on the first. One launch was measured that way before a debug build's own overflow check said so.
    on: u64,
    warned: u64,
    room: f32,
    /// **The room left on the move's own next frame**, where [`Judged::room`] is the least of all the
    /// frames looked at.
    ///
    /// Kept apart because the two answer different questions: the least room over
    /// [`THE_NEXT_FRAME_AND_LOOK_AHEAD`] frames is how tight the way out is, and *twelve* frames of it
    /// is a way out the next eleven frames of choosing can still find — where a box that overlaps on
    /// the next frame is a hit no later frame can decide its way out of. That one is what such a frame's
    /// own record is of; see [`Steered::hit_at_once`].
    next_frame: f32,
}

impl Judged {
    const CLEAR: Self = Self {
        on: 0,
        warned: 0,
        room: f32::INFINITY,
        next_frame: f32::INFINITY,
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

    /// **The same judgement over its first `frames` frames alone** — see [`A_CROSSINGS_LOOK_AHEAD`].
    fn within(self, frames: usize) -> Self {
        let kept = (1u64 << frames) - 1;
        Self {
            on: self.on & kept,
            warned: self.warned & kept,
            ..self
        }
    }
}

/// **How many frames a move is judged over while `「レーヴァテイン」` is crossed**, where every other frame is
/// judged over [`THE_NEXT_FRAME_AND_LOOK_AHEAD`].
///
/// **Because the way across is a gap between two lines of bullets that rise to the left**, `11` over every
/// `64` across — see `crate::cards::the_lane` — and a move held for forty frames goes `160` across and out of
/// it whichever of the nine it is: straight left meets the lower line, up and to the left the upper one. So
/// every move towards the far edge was hit and standing in the gap was not, and the rules stood there at
/// `376,338` with `6 of the nine clear` from the swing fading to the next reaching them, on seventeen
/// attempts running.
///
/// **Six and not twelve, which was tried first**: the gap rises `0.18` for every unit across and a diagonal
/// rises one, so held for twelve frames up and to the left climbs `34` — through the upper line — and
/// straight left meets the lower one by then too, and the rules stood at `308,386` in
/// `the_rules_of_th06c_make_their_way_left_between_the_lines_laevateinn_leaves` with only standing and the
/// step back right clear. Six frames of either are `17` of climb or `24` across, inside the gap's `40`, so
/// the two take turns and the run goes along it.
///
/// **And while `「そして誰もいなくなるか？」`'s walk round the field is followed**, for the same kind of reason:
/// the card fills the field to `640` bullets going `0.5` a frame, and over forty frames every one of the nine
/// moves goes through one of them — the rules stood with `0 of the nine clear` for over `150` frames on the top
/// edge, `185,16`, while the card's chasers came up to them, and each of the card's hits was of a run standing
/// still like that. Over six frames a bullet going `0.5` moves `3`, and the run `24`.
const A_CROSSINGS_LOOK_AHEAD: usize = 6;

/// The nine one-frame moves, as the direction each goes in.
///
/// **Standing still is first**, which is what settles a tie: where two moves are equally safe the bot holds its
/// ground rather than stepping back and forth between them every frame.
/// **The direction is which way and not how far** — how far is the game's, and a diagonal's own share of a step
/// is its own number there, which is why [`Frame::diagonal`] is a pair of its own.
///
/// **Directions and not words**, which is what moved here from the game: a word is made of that game's own bits
/// and this is nine ways to go, the same nine in every game there is.
///
/// **And up before down, which is not a preference between the two.** A tie goes to whichever comes first, and
/// down first was tried for a run of th06c's Extra that climbed out of the band and was hit there — the
/// histories of its hits have the player at `y >= 360` for 68% of their 2,910 frames and above the band for 7%,
/// with every hit in the 7%. It is the wrong answer twice over: the climb is made of moves that win outright and
/// not of ties, so the order never reached it; and a tie broken downward walks the player at the wait line into
/// the last [`ABOVE_THE_BOTTOM`] units of the field, where the clamp takes `down` away and three of these are
/// the wall — which is the arithmetic that constant was raised for, at four hits.
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
/// th07 and a bit of another elsewhere, and the recipe a bot walks names its presses the same way.
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
///
/// **Eight was tried, for a game whose held shot is its focus key, and it buys nothing.** th06's own input read
/// counts the frames the shot has been held and sets the focus bit itself from the eighth — but that read is one
/// orb replaces, and a bot's word reaches the game's own movement without passing through it. Which a launch
/// says: at a cadence of eight the words still carried the bit on the frames a boss was up and not on the
/// others, so what was setting it was [`Keys::focus`] under a fight and never the holding.
const SHOT_LET_GO_EVERY: u32 = 60;

/// How often it is let go while a line of a conversation is up, for a game whose skip key orb has no
/// binding for — see [`Keys::skip`], which is nought where [`Game::button_bit`] finds no `"skip"`.
///
/// **Ten, because the line itself asks for eight.** th06nc advances a line on an edge of `0x301` — the
/// shot's own `0x1` among them — and only once `[dialogue + 0x3420]`, the characters of the line that are
/// out, has reached eight; a cadence under that would spend its edges on a line too young to take them.
/// Ten leaves the release, the press after it and the eight.
///
/// **And what it buys is half the conversation, measured either side of this line.** Two launches of
/// th06nc drove the same recipe into the same conversation — stage 1's, the one its script clock stops at
/// 5,093 for — and the line stood from stage frame 5,091 to 5,477 at sixty against 5,092 to 5,283 at ten:
/// **386 frames against 191**. The nineteen releases of the second are counted, ten frames apart in the
/// per-frame `resume:` words; the first's six are 386 frames at one in sixty and not a count, that launch
/// having been given `--no-resume` and so writing no word a frame. Half and not a sixth, because a line's
/// own typing is a character a frame up to its length and no cadence shortens that.
const SHOT_LET_GO_EVERY_IN_A_CONVERSATION: u32 = 10;

/// The frames these rules have been asked for, which is what [`SHOT_LET_GO_EVERY`] is counted in — set to the
/// stage's own frame wherever that clock has moved, see [`the_rules_count`].
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
    /// **How many of the nine were clear on that frame**, which is what says whether the move it made was
    /// the move it had: a climb out of [`THE_BAND`] is one thing where every other move was shut and
    /// another where three were open and this one was nearest to where it wanted to be.
    ///
    /// **Which the history could not say and is the question it is kept for.** One run's hit came at
    /// `224.2,186.5` — above the band, in the hub of two four-arm crosses — and the frames before it read
    /// `with something open` all the way up, meaning the *chosen* move was clear and nothing more.
    clear: u32,
    /// **And how many of them a beam was only warning over**, which is the one thing left that can send a
    /// move the other way from where the rules want to be.
    ///
    /// **Which reading the history needed and could not do.** Three of the nine frames of one Extra's climb
    /// read `9 of the nine clear` while the word went *left* from `169,394` with the wanting at `192,368`,
    /// and nothing in the line says whether that was a warning ranked above the wanting or a defect: the
    /// boxes read `0, 0, 0, 0, 0` on all three, a beam being no part of the counting.
    warned: u32,
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
             pressing {:#05x} with {} of the nine clear, {} of them only warned, and {} and the boxes {}",
            back - step,
            before.at.0,
            before.at.1,
            before.settled.said(),
            before.wanted.0,
            before.wanted.1,
            before.word,
            before.clear,
            before.warned,
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
    /// Where each bullet was, the step it claimed, and the count that says the bullet in that slot now is
    /// the same one — see [`Stayed`].
    bullets: Vec<Option<Stayed>>,
    /// Where each body was, how it was moving, and the count that says the body in that slot now is the
    /// same one — see [`Stayed`].
    bodies: Vec<Option<Stayed>>,
    /// Where each beam was fired from, the angle it pointed at, and how many frames it had been in its
    /// phase — see [`Aimed`].
    beams: Vec<Option<Aimed>>,
}

/// **What one slot held**: where its thing stood, the step or the motion it had there, and how many frames the
/// game said it had been there — the third being what says the thing there now is that same thing, for the
/// reason `game::Hazard::frames` gives.
type Stayed = ((f32, f32), (f32, f32), Option<i32>);

/// **What one beam slot held**: where the beam was fired from, which way it pointed, and how many frames it
/// had been in its phase — the third being what says the beam there now is that same beam, for the reason
/// `game::Beam::frames` gives.
type Aimed = ((f32, f32), f32, i32);

static REMEMBERED: Mutex<Option<Remembered>> = Mutex::new(None);

/// **What the frame's own call of [`steer`] read, waiting to become the row the next frame reads against.**
///
/// Held apart from [`REMEMBERED`] so that every call inside one frame reads the same motion — see the put at
/// the end of `steer`, which has what publishing there cost, and [`remembers_what_the_frame_read`], which is
/// what moves it.
static READ_THIS_FRAME: Mutex<Option<Remembered>> = Mutex::new(None);

/// Move what the frame's own call of [`steer`] read into the row the next frame reads against.
///
/// Nothing where this frame made no such call, which is a frame answered off [`a_tick_read_again`]: the row
/// then stays the one that reading was made against, the world not having moved.
fn remembers_what_the_frame_read() {
    let read = READ_THIS_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(read) = read {
        *REMEMBERED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(read);
    }
}

/// The frame last read out of the game, kept so that reading the next one allocates nothing.
static SEEN: Mutex<Option<Frame>> = Mutex::new(None);

/// How many of the frames these rules were asked for read a tick they had already read — see
/// [`a_tick_read_again`], and the hit's own line, which is where it is said against [`FRAMES`].
static AGAIN: AtomicU32 = AtomicU32::new(0);

/// **What the rules decided last frame, for a frame that read the same tick over again** — or nothing where
/// this reading is a new one.
///
/// **Such frames exist because the read that asks for a word is not the game's tick.** A game orb has no frame
/// loop in is asked off its own keyboard read, and th06c makes that read as often as it gets round to it: one
/// launch's play step was asked 62,400 times in 944.9 seconds — 66.0 a second, running from 61 to 76 as the load
/// moved — against a game that moves its bullets sixty times a second. So about one frame in ten is the frame
/// before it, read again.
///
/// **And on such a frame the model is a field of bullets standing still.** What a bullet moved is where it is
/// now less where it was, so a tick read twice makes that nought for every one of them while each still claims
/// a step — which is [`A_STEP_OF_ITS_OWN`]'s test for a bullet a script is moving, and a scripted bullet is
/// feared over what it *just* moved rather than projected along its step. Feared over nothing, here. Four hits
/// of one launch of th06c were dumped and three of them were decided on a frame like this, each saying every
/// bullet it held was explained by no step of its own — 100 of 100, 53 of 61, 195 of 195 — with the widest of
/// them having moved `0.00,0.00` while its step said `-4.82` down the screen.
///
/// **Read off the world and not off a clock of the game's**, because what is being asked is whether this
/// reading is a new one, and the reading itself answers that: the bullets that claim a step standing exactly
/// where they stood, and the player with them. Exact equality, these being the same bytes read twice.
///
/// **A field where nothing claims a step says nothing either way, and such a frame decides for itself.** There
/// is nothing to be wrong about where nothing is moving: the same decision comes out of the same reading.
fn a_tick_read_again(frame: &Frame) -> Option<Steered> {
    let was = (*LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner))?;
    if was.at != frame.at {
        return None;
    }
    let kept = REMEMBERED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let last = kept.as_ref()?;
    let mut claimed = 0;
    for hazard in &frame.hazards {
        if !matches!(hazard.kind, Kind::Bullet { .. }) {
            continue;
        }
        let Some((stood, step, _)) = what_a_slot_held(&last.bullets, hazard.slot) else {
            continue;
        };
        if step == (0.0, 0.0) {
            continue;
        }
        if stood != hazard.at {
            return None;
        }
        claimed += 1;
    }
    (claimed > 0).then_some(was)
}

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

/// **Whether the count a slot carried last frame and the count it carries now are one thing's and not two**,
/// which is what a difference across the two frames may be taken as motion on the strength of — see
/// `game::Hazard::frames`. The frame after is one further on, and nothing else is the same thing.
///
/// `true` where the game keeps no such count on either, which leaves the slot to say it: that is every game
/// but th08, and what it costs is in `game::Hazard::frames`.
fn the_same_thing(frames: Option<i32>, was: Option<i32>) -> bool {
    match (frames, was) {
        (Some(frames), Some(was)) => frames == was + 1,
        (None, None) => true,
        // A count on one frame and none on the next is one game answering the same question two ways, which
        // no sight does — and taken as the same thing it would be the reading above trusted for nothing.
        _ => false,
    }
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

/// How far a beam was carried since last frame, and nothing for one that was not there then.
///
/// **A difference and not a rate**, for the reason [`turned`] gives: what carries a beam is its boss's script,
/// and nothing on the beam says where it is going.
///
/// **And a beam that is carried is the one hit th06c's Extra boss took every attempt of.** Its wide beam is
/// fired from the boss and follows her while it charges: `114.0,58.1` one frame and `120.4,59.2` the next,
/// `6.4` across in one frame against the player's own `4.0`. Taken as standing still it left the move the
/// rules chose `0.0 of room` with every one of the nine hit, and the frame after it the player was inside the
/// beam — three attempts in one launch, each of them at that beam and none of them at anything else.
fn moved(was: Option<(f32, f32)>, (x, y): (f32, f32)) -> (f32, f32) {
    match was {
        Some((was_x, was_y)) => (x - was_x, y - was_y),
        None => (0.0, 0.0),
    }
}

/// How far what a hazard actually moved may differ from the step it claims, **or its step from the step it had
/// last frame**, before it counts as moved by something other than itself. A hair, an ordinary bullet's movement
/// *being* its step to the bit and its step never changing at all.
///
/// **The step's own change is the half that catches a beam.** Comparing only the movement against the step
/// misses a piece that turns gently: the disagreement in one frame is under any threshold worth having, and the
/// straight line it is then projected along is wrong by the whole of the turn twelve frames out — which is a hit
/// reading `12.2 of room` with nothing else near, and stage 6's turning card gave three of them.
const A_STEP_OF_ITS_OWN: f32 = 0.5;

/// **The move's own next frame, and then the [`LOOK_AHEAD`] after it.**
///
/// **The player and the hazard are projected by the same count, because one frame moves both.** th06c
/// advances a bullet's own position at `0x14000e377` and hands *that* position to the hit test at
/// `0x14000e490`, and the player's job is walked ahead of the bullets' — so the frame a hit happens on
/// is one where each has taken the same number of steps, and pairing the player where it stands with
/// the bullet a step on credits the move with a whole step of room the frame does not have. Measured:
/// a stage 4 hit was taken on a move judged to have `2.6` of room with a bullet stepping `2.7` a frame,
/// and the two boxes were `6.7` apart on the axis where their halves add to `6.7` — touching, and the
/// game's test takes that. See `orb-core`'s `game/th06c/asm.md`.
///
/// **So the frame the player is standing in is not judged at all**, and nothing is lost with it: it is
/// the frame whose test has already run, no move can change where the player already is, and the same
/// answer for all nine moves is one that decides nothing between them.
const THE_NEXT_FRAME_AND_LOOK_AHEAD: usize = LOOK_AHEAD + 1;

/// **How little room the move chosen has left on its own next frame for the frame to count as one that
/// was lost**, which is not [`ROOM_TO_SPARE`] and is a different question from it.
///
/// **Those two answer different things.** The slack is what a move has to clear a hazard by before the
/// rules will call it clear, and it is generous on purpose: a move judged clear has to *be* clear. What
/// this answers is whether the frame is a hit already, and a frame the rules read wrongly either way is
/// one somebody has to be able to read back afterwards — so it wants the smallest number the projection
/// can be trusted to, and the slack is not that number.
///
/// **Measured on the four bombs one th06c Extra launch spent inside Flandre's fight**, back when such a
/// frame spent one: `1.9`, `-3.2`, `0.0` and `-3.3` of room. The three at nought or below are hits the
/// bomb took back — the game's own test compares its edges inclusively, so nought is a hit there — and
/// `1.9` is a frame nothing was going to touch, one bomb of four spent on the slack rather than on a
/// corner. That launch was then hit on the corner after its last bomb, which is the reading that
/// eventually left the bomb to the deathbomb and this number to the record.
///
/// Half a unit: five times the `0.1` the one hit this threshold was first set from had, and a quarter of
/// the `1.9` a bomb was wasted at.
const ROOM_A_BOMB_IS_OWED_AT: f32 = 0.5;

/// **How long after one corner the next is worth writing down**, counted in the frames the rules are asked
/// for.
///
/// **Longer than a bomb of the game's own lasts**, which is the length it was chosen for while this held a
/// bomb apart from the next: th06's clears the screen and holds the player untouchable for a few seconds,
/// and a second bomb pressed inside that is one spent on a screen with nothing in it. Four seconds at sixty
/// a second. What it holds apart now is the dump, and a corner that lasts a hundred frames is one corner.
const BOMBS_APART: u32 = 240;

/// When the last corner was written down — see [`BOMBS_APART`].
static SAID_AT: AtomicU32 = AtomicU32::new(0);

/// Whether the player was above [`the_band`] on the frame before, so that a climb out of it is written down
/// on the frame it is made and not on every frame it lasts.
static WAS_ABOVE_THE_BAND: AtomicBool = AtomicBool::new(false);

/// And when the last such climb was written down, held apart by [`BOMBS_APART`] like the corner.
static LEFT_THE_BAND_AT: AtomicU32 = AtomicU32::new(0);

/// **How many frames running nothing has been open to the move chosen, which is a place lost rather than a
/// frame lost.**
///
/// **One such frame is the judgement's own pessimism and dozens of them running are not.** Nothing open means
/// every one of the nine is inside [`ROOM_TO_SPARE`] of something somewhere in the frames looked at, and a
/// screen of th06's stage 1 reads that way often enough to have spent a run's three bombs in two stages back
/// when such a frame spent one — see [`Steered::hit_at_once`], which is why a single frame of it is not a
/// stretch and is not written down as one. What it cannot see
/// is a *stretch* of them: the player is then somewhere every move is shut from, and each of those frames was
/// the best of its own nine, so nothing in the ranking gets it out. th06c's Extra boss took a run's second hit
/// after `169` frames of exactly that — `wanting the emptiest box at 192,368` on every one of them, and the
/// player at `x = 376` in the thickest box of the five the whole time.
///
/// A second of it, against a stretch six times that long; and one corner's record is held from the next by
/// [`BOMBS_APART`] whichever of the two asked for it, so what this can cost a launch is one dump in four
/// seconds.
const SHUT_IN_FOR: u32 = 60;

/// How many there have been in a row, counted from the last bomb — see [`SHUT_IN_FOR`].
static SHUT_IN: AtomicU32 = AtomicU32::new(0);

/// **How many of a card's frames the gauge may stay short of its human end with the shot unfocused before the
/// rest of the card is played focused** — see `Frame::at_the_gauges_human_end`.
///
/// **Four hundred, which is past every stretch a card that pays at that end had.** Six runs of stage 2 read off
/// their traces: the three that were at the human end on a third to a half of card 22 were short of it for
/// `188`, `264` and `704` frames running at most, and every other card of all six for no more than `328`;
/// the three that never reached it through card 22 were short of it from its third frame to its last,
/// `1524` to `1596` of them. So a card is taken for one grazed off its human end some four hundred frames in,
/// and one of the six besides — the `704` — in the last three hundred frames of its own card 22.
///
/// **Focused and not the rest of it thrown away**: focused, the grazes and the player lean the gauge the same
/// way, to the youkai end, where a graze pays a time orb of its own (0x44aab3 through 0x44aac9) — and played
/// focused from its first frame, card 22 carried two of those three runs over the norm and the third to
/// within `139` of it, while two of the three that had reached the human end fell short of it that way.
/// **Given over only once held off, five of the six met the norm** where three had: the three that reached the
/// human end met it as they had, the one whose card 22 was given over in its last frames with `7758` against
/// `8295`, and two of the three that never did met it with `7525` and `8097`, the third ending on `6756` against
/// the `6567` it had.
const HELD_OFF_THE_HUMAN_END_FOR: i32 = 400;

/// The card frame the gauge last went short of its human end on, the card frame last seen, and whether the
/// card up has been given over to the focused shot — see [`HELD_OFF_THE_HUMAN_END_FOR`].
static HELD_OFF: Mutex<(Option<i32>, i32, bool)> = Mutex::new((None, 0, false));

// **Nothing here is a dead zone round where it wants to be**, which the choice makes unnecessary: standing
// still is one of the [`WAYS`] and the one a tie goes to, so the frame the player is there is a frame nothing
// is pressed for.

/// The word the rules press this frame, and nothing at all off a stage.
///
/// **What it does about the hazards is told to it and not decided here**, which is a chapter's business rather
/// than a frame's: an attempt drawn to be hit in ignores them for the whole of itself — see [`crate::hits`].
///
/// **Nothing off a stage, the shot included.** The rules are asked for a word by the game's own keyboard read,
/// which every screen makes — and a menu of th07's is taken by the shot as much as by the return key, so a
/// word held on the title screen walks that menu once and then holds it, while the d-pad there moves a cursor
/// these rules cannot read. The front end is the recipe's to drive, one press to a screen; a rule pressing
/// alongside it is a press nobody wrote.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn word_to_press(game: &dyn Game, hazards: Hazards, bombs: bool) -> u16 {
    // **The one buffer, kept between frames**, so that a frame of a run costs no allocation once it has run
    // once: the three lists it holds are cleared and filled, and a stage of th07 fills the first of them six
    // hundred times.
    let mut seen = SEEN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let frame = seen.get_or_insert_with(Frame::default);
    if !unsafe { game.what_a_bot_sees(frame) } {
        return 0;
    }
    orb_core::trace::the_player_at(frame.at);
    // **And the hazards ignored while `「幻視調律」`'s lattice is where none of it can hit** — see
    // `crate::cards::visionary_tunings_lattice_cannot_hit`, which is the climb the rules dodging it never make.
    // All of them for those frames, and not the hidden bullets taken out of what the rules see: `th08::sight`
    // hands those over as hitting for the card it measured that on, with nothing on them to say they are
    // hidden, and this card's schedule says when none of its bullets can hit without asking which.
    let hazards = match hazards == Hazards::Dodged
        && the_dodge() == Some(Dodge::VisionaryTuning)
        && the_cards_frames().is_some_and(crate::cards::visionary_tunings_lattice_cannot_hit)
    {
        true => Hazards::Ignored,
        false => hazards,
    };
    let keys = Keys::of(game);
    // **The shot held, and let go for one frame in every [`SHOT_LET_GO_EVERY`].** A stage's own clear screen
    // goes on to the next stage the frame the shot goes *down* and otherwise waits out its own timer, and a run
    // that has held the shot since the stage began never gives it that frame: one launch spent 60 seconds
    // between stage 1's last chapter and stage 2's first. A shot in sixty is what that costs, th07's own shot firing
    // off the player's timer rather than off a press.
    let count = the_rules_count();
    let let_go_every = if frame.dialogue {
        SHOT_LET_GO_EVERY_IN_A_CONVERSATION
    } else {
        SHOT_LET_GO_EVERY
    };
    // **And held altogether while the familiars out are worth more alive** — see
    // `Frame::familiars_are_worth_sparing`. The fire and not the focus, the focused shot's hits being worth
    // nothing where the unfocused one's are.
    let spares_them = hazards == Hazards::Dodged && frame.familiars_are_worth_sparing;
    // **Never on two asks running**, which a count set back to the stage's frame would otherwise do: the
    // laid-out th07's clock appears at nought on the ask after one that counted nought, and let the shot go on
    // both — three over `120` frames where there are two.
    let lets_go = count.is_multiple_of(let_go_every) && !LET_GO_LAST_ASK.load(Ordering::Relaxed);
    LET_GO_LAST_ASK.store(lets_go, Ordering::Relaxed);
    let mut word = if lets_go || spares_them { 0 } else { keys.shot };
    // The frame the player is hit and not the frames after it — the window the game counts down is one this
    // presses into rather than a single frame, and its end takes the life.
    //
    // **And not at all in a chapter drawn to be hit in**, which is what the whole of that drawing is for: a
    // A deathbomb is the death taken back, so a bomb here is a chapter that was played to be died in and was
    // not.
    // **And not at all where the recipe forbade the bomb**, which is what a run held to no bomb is: the
    // hit stands and takes the life — see `recipe::Step::Bombs`.
    if frame.hit && hazards == Hazards::Dodged && bombs {
        word |= keys.bomb;
    }
    // A dialogue read at the game's own pace is the afternoon this was written to save.
    //
    // **And the skip is not every game's to press**, which is why the release above is the other half of
    // this: [`Keys::skip`] is nought where the game has no `"skip"` among its buttons — th06nc's has
    // none, its own skip bit being one no key orb presses makes — so what carries such a game's
    // conversation is the shot going down at [`SHOT_LET_GO_EVERY_IN_A_CONVERSATION`].
    if frame.dialogue {
        word |= keys.skip;
    }
    // **Focused from the first pass, for a card dodged [`Dodge::Focused`]**, and not the whole step with the
    // fine one asked only where nothing is open: one card of the kind — rings out of a boss at the top of the
    // middle, hidden and shown again — was lost at the one place `14.0,423.5` on six attempts running, every
    // one of them `the focus key not held` — and a path at the whole step crosses twice the ground the fine
    // step's does, which is the fine pass below's own reason. **The norm was met through all of it**, so what
    // kept the key up was not `Frame::shoots_a_boss_unfocused`: it is [`holds_the_focus`] asking for a frame
    // with nothing to dodge, which a screen of rings hardly ever is — the key was down on 993 of that card's
    // 5,878 frames, and the fine pass below is asked only where nothing at the whole step is open.
    //
    // **Not for [`Dodge::BehindTheLaser`], which was tried.** Its card's rice is `チョン避け` by the strategy
    // wiki's own words and the run is pushed to the floor at the whole step, so the fine one looked like the
    // answer; a pick-up of that card's chapter played it focused took four hits and a death where the whole
    // step's took two and none, at the floor on 47% of its wanted lines against 40%.
    //
    // **And for the rest of a card whose grazes have held the gauge short of its human end** — see
    // [`HELD_OFF_THE_HUMAN_END_FOR`].
    let held_off = held_off_the_human_end(frame);
    let focused_through =
        hazards == Hazards::Dodged && (the_dodge() == Some(Dodge::Focused) || held_off);
    let steered = match a_tick_read_again(frame) {
        Some(again) => {
            AGAIN.fetch_add(1, Ordering::Relaxed);
            again
        }
        None => steer(frame, &keys, hazards, focused_through, THE_FRAMES_OWN),
    };
    // **And the choice made again with the step the focus key it is about to press will take**, which is the
    // circle between the two closed rather than left open: the bit is decided from what steering answered —
    // see [`holds_the_focus`] — and the step a frame moves by is decided by the bit, so one pass is always a
    // projection of the other state's step on any frame that answer changes. th06c's are `4.00` and `2.83`
    // against `2.00` and `1.41`, which is a move that covers half what it planned or twice it.
    //
    // **Twice at most, and the unfocused answer is what a disagreement keeps.** Where the focused pass no
    // longer says the frame is one to hold the key on, the frame is played at the whole step it was found
    // clear at: that is the answer both passes agree about, where taking the focused one would be pressing a
    // key its own projection says not to.
    let (mut steered, mut focused) = (steered, focused_through);
    // **And never the calm frame's focus for a card gone round a small square**, whose whole point is being gone
    // from where the familiars fire: see [`round_a_small_square`], whose three hits came with the key down.
    // **Nor for `「レーヴァテイン」`**, whose swings across are got through by crossing the field ahead of
    // them, which the halved step does not do in time — see `crate::cards::laevateinn`. **Nor for
    // `「過去を刻む時計」`**, whose low clock is got past by crossing behind its centre — see
    // `crate::cards::the_clock`. **Nor for `「そして誰もいなくなるか？」`**, whose walk round the whole field has
    // to keep ahead of the fire piling up behind it.
    let the_whole_step_through = matches!(
        the_dodge(),
        Some(
            Dodge::RoundASmallSquare
                | Dodge::Laevateinn
                | Dodge::Clock
                | Dodge::AndThenThereWereNone
        )
    );
    if !focused && !the_whole_step_through && holds_the_focus(frame, hazards, &steered) {
        let held = steer(frame, &keys, hazards, true, ASKED_AGAIN);
        if holds_the_focus(frame, hazards, &held) {
            steered = held;
            focused = true;
        }
    }
    // **And once more with the fine step where nothing at all is open**, because what a move is judged on is
    // a *path* and not a landing: [`LOOK_AHEAD`] frames of the whole step travel `52` units and of the
    // focused one `26`, and a path that travels less crosses less.
    //
    // **Measured on the fan th06c's Extra boss lays.** Nine lanes from `192,80`, their angles `0.039` rad
    // apart and each `half_across` of `3.0` — which at the player's own radius from that origin, `266`,
    // is `0.023` rad of lane and `4.3` units of gap between two, against a player `2.4` across. So each
    // gap is passable and the trouble is getting *along* one: a fifty-two unit path crosses a dozen lanes
    // and is hit on one of them, where half of it crosses six. Three bombs of one launch went on that
    // pattern with `the model was holding 0 within reach` — no bullets at all, the lanes alone, and every
    // one of the nine shut.
    //
    // **Not the landing, which the whole step already reaches finely enough**: the nine land at `0`,
    // `±2.83` and `±4.00` across, five places inside four units, so a `4.3` gap within reach holds one of
    // them already. It is the frames in between that the coarse step spends crossing lanes.
    //
    // **Asked only where the whole step found nothing**, which is what keeps it from being the halving
    // [`holds_the_focus`] exists to refuse: a frame with room to spare is played at the step that has the
    // ground, and a frame with none has nothing left to lose.
    //
    // **And taken only where it found real room and not merely something not shut.** An answer with a
    // slack's worth of room is not an answer, and taking it reports the frame as one that had a way out:
    // `hit_at_once` reads the room this leaves, so the frame is neither steered out of nor written down.
    // One launch's hit was exactly that — every whole-step move shut, the fine pass answering
    // `2.2 of room`, and the player hit on the frame after. So the same margin [`holds_the_focus`] asks
    // for is asked here, of the step this move will actually take.
    if !focused && steered.room.is_none() {
        let fine = steer(frame, &keys, hazards, true, ASKED_AGAIN);
        let enough = ROOM_TO_SPARE + frame.focused_straight.0.max(frame.focused_straight.1);
        if fine.room.is_some_and(|room| room > enough) {
            steered = fine;
            focused = true;
        }
    }
    if focused {
        word |= keys.focus;
    }
    // **And now that every pass has been made, what the frame read becomes what the next one reads
    // against.** Any earlier and the passes disagree about what moved — see
    // [`remembers_what_the_frame_read`].
    remembers_what_the_frame_read();
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
    // Said once for the hit and not once for every frame of the window it opened: the deathbomb window stands for
    // as long as the game counts it down, and a line a frame turns one hit into a page of them.
    if WAS_HIT.swap(frame.hit, Ordering::Relaxed) != frame.hit && frame.hit {
        said_what_took_it(game);
        // And the frame itself, which is the one thing the arrays cannot be asked for: a hit whose dump
        // names nothing within reach of the player was still made by something that was drawn.
        orb_core::shot::wanted("hit");
    }
    // **And a border broken is a hit, which orb was not counting at all.** th07 deflects a hit taken on a
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
    // **And the frame nothing was open on is written down and no bomb is spent on it**, which is the whole
    // of what a bomb of these rules is for now: the one above, pressed into the window the game opens on a
    // hit it has already registered, and that one only.
    //
    // **Because a bomb spent before the hit buys nothing the deathbomb does not.** Both clear the screen
    // and both make the player untouchable while they do; what the earlier one adds is that the hit is
    // never registered, and what it costs is the stock. **Which is what one Extra attempt paid.** Eight of
    // its bombs went on frames the rules called lost — rooms of `-0.1`, `-2.9`, `-2.8`, `-0.3`, `0.2`,
    // `0.2` and `0.2` on the move's own next frame — and then both of its hits came with `bombs=0` in hand
    // and `6 frame(s) a bomb still takes it back in` on the dump, so both took a life a bomb would have
    // taken back. **And a prediction can be wrong where a hit cannot**: the frame above is a hit the game
    // has counted, and these are frames a model says are lost.
    //
    // **What is kept is the record and the picture**, which is what every reading in this change was made
    // of: the frames nothing was open on are where the rules are wrong or the place was lost, and
    // [`said_what_the_frame_had_nothing_open_on`] writes what a hit writes. [`ROOM_A_BOMB_IS_OWED_AT`] is
    // still the room that decides one, and [`SHUT_IN_FOR`] still the stretch.
    //
    // **And nothing in a chapter drawn to be hit in**, the same as the deathbomb: that drawing is the whole
    // of what a retry measurement is for.
    //
    // **And a stretch counted only over frames the hazards were dodged on.** A move chosen with them ignored
    // is the one nearest where the rules want to be, so the frames of a climb through a lattice that cannot
    // hit are all of them shut by the model's lights — and carried over, the climb's end would be a stretch a
    // second long on its first frame of dodging.
    let shut_in = match (hazards, steered.room) {
        (Hazards::Dodged, None) => SHUT_IN.fetch_add(1, Ordering::Relaxed) + 1,
        _ => {
            SHUT_IN.store(0, Ordering::Relaxed);
            0
        }
    };
    // **And held apart by [`BOMBS_APART`] the way the bomb was**, one line and one picture per corner
    // rather than one per frame of it: the frames a corner lasts are a corner and not dozens of them.
    let count = FRAMES.load(Ordering::Relaxed);
    let cornered = steered.hit_at_once || shut_in >= SHUT_IN_FOR;
    if hazards == Hazards::Dodged
        && cornered
        && !frame.hit
        && count.wrapping_sub(SAID_AT.load(Ordering::Relaxed)) >= BOMBS_APART
    {
        SAID_AT.store(count, Ordering::Relaxed);
        SHUT_IN.store(0, Ordering::Relaxed);
        // **And a bomb for the place and not for the frame**, which is the half of the older rule the
        // deathbomb does not already cover. A bomb pressed the frame before a hit buys nothing that one
        // does: the same hit, paid a frame earlier — see the comment above, and the eight of them one
        // attempt spent before both of its hits came with `bombs=0` in hand. **A stretch is the other
        // question.** What a bomb buys there is not the frame: it clears the screen and holds the player
        // untouchable across it, which is the one thing that puts a run somewhere it can play from — and
        // where nothing has been open for a second the run is not going to get there by steering, every
        // one of those frames having been the best of its own nine.
        //
        // **Which the frames of one climb say outright.** A run pinned at `x = 8` crawled from `y = 119`
        // to `96` with `0 of the nine clear` on every frame of it, was recorded as
        // `60 frame(s) running with nothing open` at `8.0,121.8` — and with nothing spent on it went on
        // crawling to be hit at `12.0,95.7`, two lives and a bomb in hand.
        //
        // **And not under `「幻視調律」`, whose lattice reads that way on most of its frames and is got
        // through all the same** — see `crate::cards::visionary_tunings_lattice_cannot_hit`. Two clears of
        // one session spent a bomb twice apiece on such stretches of it, after `60` to `89` frames, and the
        // no-bomb clear of the session after went through about 2,450 frames of the card with no hit.
        let got_through = the_dodge() == Some(Dodge::VisionaryTuning);
        let for_the_place = !steered.hit_at_once && frame.bombs > 0 && bombs && !got_through;
        if for_the_place {
            word |= keys.bomb;
        }
        match (steered.hit_at_once, for_the_place) {
            (true, _) => detail!(
                "autoplay: nothing open and {:.1} of room on the move's own next frame, which is a \
                 hit the window is for",
                steered.next_frame,
            ),
            (_, true) => detail!(
                "autoplay: a bomb spent after {shut_in} frame(s) running with nothing open, which is \
                 a place to be out of"
            ),
            _ if got_through => detail!(
                "autoplay: {shut_in} frame(s) running with nothing open under a lattice that reads \
                 so and is got through, and no bomb spent on it"
            ),
            _ => detail!(
                "autoplay: {shut_in} frame(s) running with nothing open and no bomb to spend on it, \
                 which is a place lost"
            ),
        }
        // **And the whole of what the rules made of the frame**, which the room alone is not: see
        // [`said_what_the_frame_had_nothing_open_on`], and the picture beside it for the one question the
        // arrays cannot answer — whether the screen the rules called shut had a way out on it.
        said_what_the_frame_had_nothing_open_on(game, &steered);
        orb_core::shot::wanted("shut");
    }
    // **And the frame the player leaves [`the_band`], which is the one frame of a climb anything can be
    // read off.** The dumps above are a hit and a second with nothing open, and a climb is neither: every
    // attempt of th06c's Extra puts two to four hits into the region above the band's own top, and read
    // back from each of them the run spends sixty frames going up with three or four of the nine open — so
    // the moves it *had* on those frames are what says why up was the one it took, and no line of any
    // launch carried them. The wanting is in the history and the nine judgements are not.
    //
    // **And what it answered is that the climb is forced.** One attempt's four, with the moves open on
    // each and what stood nearest:
    //
    //     76.3,292.2    up, up-left, up-right only    a laser at 80.5,314.5 stepping 1.2,0.0, 22 below
    //     314.2,291.3   still, up, right, up-left,    a bullet at 244.8,194.2 — the four downward hit
    //                   up-right
    //     116.4,293.1   none of the nine              a laser at 144.6,265.4 stepping -1.2,0.7, 28 above
    //     99.9,291.3    up, up-left, up-right only    a bullet at 148.0,223.6
    //
    // Three of the four have the three upward moves and nothing else, and the fourth is the run going for
    // an item above the band, whose place no clamp of [`where_to_be`]'s is applied to by design. On two of
    // them the nearest thing is a laser lying below or level with the player *inside* the band — this
    // boss lays arms across the strip the rules wait in, and `Counted::closes` counts all five boxes of it
    // closed on ten of nineteen frames one attempt dumped. So upward is the answer left on those frames
    // and a run that climbs is dodging, which is why nothing here refuses the climb.
    //
    // **Written on the crossing and not on every frame above**, so that one climb is one dump, and held
    // apart by [`BOMBS_APART`] besides for a run that hovers on the line.
    let above = steered.at.1 < the_band(frame.field);
    let was_in = !WAS_ABOVE_THE_BAND.swap(above, Ordering::Relaxed);
    if hazards == Hazards::Dodged
        && above
        && was_in
        && !frame.hit
        && count.wrapping_sub(LEFT_THE_BAND_AT.load(Ordering::Relaxed)) >= BOMBS_APART
    {
        LEFT_THE_BAND_AT.store(count, Ordering::Relaxed);
        said_what_the_frame_left_the_band_on(game, &steered);
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
        clear: steered
            .judgements
            .iter()
            .filter(|judged| judged.clear())
            .count() as u32,
        warned: steered
            .judgements
            .iter()
            .filter(|judged| judged.only_warned())
            .count() as u32,
        fire: steered.counted.fire,
    });
    // **And the same three said every frame where a launch asked for them**, which is the one instrument a
    // question about a whole fight has: the record above is kept for the seconds before a hit, and what a
    // fight's own damage wants is the *share* of its frames — how often the wanting was the boss, how often
    // the rules were in her column, and how many of the nine were clear when they were not.
    //
    // **Because the payout th08's Last Spells are due on turns on exactly that share.** A card there pays a
    // thousand time orbs where it is taken inside a seventh of its own time and nothing where it is not, and
    // a run's stage 1 read `2,094` of the `2,500` its norm wants with no payout in it at all; the frames of
    // that boss's fight have the rules within 16 of her column on **42%** of them. Which of the two that is —
    // a wanting that is not hers, or a wanting of hers the nine outrank — is what this line answers, and no
    // dump does: a dump is a frame and this is a fight.
    //
    // **And the boxes the wanting was chosen by**, which is what a walk to an edge is decided on and what no
    // dump of a fight without a hit carries: one run spent **3,400** of a survival card's 7,360 frames in the
    // leftmost box, wanting the emptiest box on 4,269 of them, and nothing it said could tell a box counted
    // full from a ring not read as one.
    detail!(
        "autoplay: wanted {} at {:.0},{:.0} from {:.0},{:.0} with {} of the nine clear, the boxes {}{}",
        steered.settled.said(),
        steered.wanted.0,
        steered.wanted.1,
        steered.at.0,
        steered.at.1,
        steered
            .judgements
            .iter()
            .filter(|judged| judged.clear())
            .count(),
        steered
            .counted
            .fire
            .iter()
            .map(|fire| format!("{fire:.0}"))
            .collect::<Vec<_>>()
            .join(", "),
        match steered.counted.overhead {
            true => " under a boss overhead",
            false => "",
        },
    );
    *LAST_FRAME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(steered);
    // **Whether the step the projection was made of is the step the frame will take**, which is a thing to
    // check and no longer a thing to expect: the two passes above are made for exactly this, so what is left
    // for this to catch is a move taken from a tick already read — see [`a_tick_read_again`], whose answer was
    // projected on some earlier frame and whose focus rule is re-asked here.
    //
    // **The pairs themselves and not the bit changing.** What matters is the number the twelve frames were
    // walked with, so that is what is compared: the bit going on or off between two frames is ordinary, and
    // says nothing about whether either frame was projected right.
    let focused = word & keys.focus != 0 && keys.focus != 0;
    WAS_FOCUSED.store(focused, Ordering::Relaxed);
    let would_take = match focused {
        true => (frame.focused_straight, frame.focused_diagonal),
        false => (frame.straight, frame.diagonal),
    };
    THE_STEP_WAS_STALE.store(steered.step != would_take, Ordering::Relaxed);
    word | steered.word
}

/// **Whether the frame steering has just answered is one to hold the focus key on**, which is where a shot does
/// its damage — and off again for either of the two things that want the whole step: anything that reaches the
/// move chosen inside the frames looked at, since the halving is paid for by the frame a bullet is a hit, and an
/// item to go for, which is most of a field of crossing at half the speed.
///
/// **And off in a chapter drawn to be hit in**, whatever is up: the halved step is what a player fighting a boss
/// holds, and that is a run on its way to a death — twice as many frames to cross the field in is twice as long
/// before the thing that is going to hit it does.
///
/// **And off unless the move chosen has a whole step of room beyond the slack**, which is stricter than being
/// clear and is what four hits of one Extra launch bought: each was a move the model called clear with `2.3`,
/// `2.4`, `2.7` and `4.2` of room, the focus key held. [`ROOM_TO_SPARE`] is two units, so those are frames
/// whose margin is a slack and no more — and a halved step covers half the ground over the frames looked at
/// while everything coming at it keeps its own speed, which is what turns such a margin into a hit.
///
/// **The room and not what is near**, which the case about a bullet twelve units off and going nowhere is
/// there to refuse: room is measured past both boxes, so one number means the same thing for a large bullet
/// and a small one where a distance means two different things. That bullet leaves `8` of room against the
/// `6` this asks for and the key stays on.
///
/// **It is also what the rule used to do without saying so.** Before the two passes above, the key was
/// decided off a projection made with the step the *last* frame took, so holding it made the next frame's
/// projection the halved one, which said `dodging` and took it off again: the launches before this show
/// `the focus key not held` on every dump, and the rule was calibrated against that oscillation rather than
/// against holding it.
///
/// Its own function because [`word_to_press`] asks it twice, of two passes of [`steer`].
fn holds_the_focus(frame: &Frame, hazards: Hazards, steered: &Steered) -> bool {
    let step = frame.straight.0.max(frame.straight.1);
    // **Standing still and not the move chosen**, which is the difference between *nothing is coming at the
    // player* and *the way out is wide*. A bullet falling onto the player's own column leaves every sideways
    // move a whole field of room while the frame is one to take at the whole step — and read off the move
    // chosen, the key goes down on it. `the_bot_shoots_a_boss_slowed_down` is that frame: a bullet `12` to
    // the side with no step of its own is a frame to hold the key on, and the same bullet `24` above and
    // closing `4` a frame is not, where both leave the sideways moves clear by the width of the field.
    //
    // **And not on a game whose boss is worth more shot unfocused**, which is a game saying what its shot
    // earns rather than how its player dodges: see `Frame::shoots_a_boss_unfocused`. th08's shots drop a
    // time orb with its gauge at the human end, which the unfocused shot leans it to, and a key held on calm
    // frames and let go to dodge holds the gauge in the middle — over two launches of its stage 1, 4,152
    // frames of the boss's fight were shot focused in her column and the stage ended at 1,447 and 1,626 time
    // orbs of a norm of 2,500, where the Last Spell wants the norm met. Only until it is met, which is the
    // game's own answer every frame. The fine step below is left alone: that is a dodge, and a dodge is not
    // what the key is held for here.
    hazards == Hazards::Dodged
        && !steered.dodging
        && !steered.for_an_item
        && steered.judgements[0].room > ROOM_TO_SPARE + step
        && frame.boss.is_some()
        && !frame.shoots_a_boss_unfocused
}

/// Whether the focus key was held on the frame just decided, and whether the step that frame's projection was
/// made with is the one its own word takes. Read by [`said_what_took_it`] and nothing else.
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
    said_what_the_frame_held(game, &was, "hit at");
}

/// **And the same dump for a frame the rules found nothing open on**, which is the frame being judged now
/// rather than the one before: that is decided on the frame itself, where a hit is only noticed on the frame
/// after the game took it. So this is handed [`steer`]'s own answer and [`said_what_took_it`] reads
/// [`LAST_FRAME`].
///
/// **Because such a frame with only its room beside it cannot be argued with.** One launch spent a bomb at
/// `-1.1` of room on a screen somebody watching called plainly open, and the line carried no position, no
/// nine judgements and nothing of what the model was made of — so which of *phantom hazard*, *wrong step*
/// and *nothing open after all* it was could not be told apart, and the frame is gone by the time anybody
/// asks.
fn said_what_the_frame_had_nothing_open_on(game: &dyn Game, steered: &Steered) {
    said_what_the_frame_held(game, steered, "nothing open at");
}

/// **And the same dump for the frame a climb out of [`the_band`] was made on**, which is the frame whose
/// nine judgements say why upward was the move it had — see the call, which has what nine attempts could
/// not answer without it.
fn said_what_the_frame_left_the_band_on(game: &dyn Game, steered: &Steered) {
    said_what_the_frame_held(game, steered, "left the band at");
}

/// The dump itself, `took` naming what it is of — see the two above.
///
/// **Never the words `hit at` for anything but a hit**, that being what a launch's hits are counted by: the
/// dump of a frame the rules called shut is read by the same eye, and some of those are frames nothing was
/// going to touch — a launch with three of them would otherwise read as a run that took three hits.
fn said_what_the_frame_held(game: &dyn Game, was: &Steered, took: &str) {
    let room = match was.room {
        Some(room) => format!("{room:.1} of room"),
        None => "nowhere open to it".to_owned(),
    };
    // **The hit itself at every level, and what it is diagnosed with only where somebody asked for detail.**
    // Where, what was pressed and how much room there was is the hit; which hazard was nearest and at what
    // size, what the model was made of, and whether the step was the frame before's are what a *calibration*
    // reads, and a run nobody is calibrating should not carry three lines' worth of them per hit.
    detail!(
        "autoplay: {took} {:.1},{:.1} having pressed {:#05x} with {room}, half of it {:.1},{:.1}, \
         projected at {:.2},{:.2} straight and {:.2},{:.2} diagonal, the focus key {}",
        was.at.0,
        was.at.1,
        was.word,
        was.half.0,
        was.half.1,
        was.step.0.0,
        was.step.0.1,
        was.step.1.0,
        was.step.1.1,
        // **What orb pressed on that frame, which the word beside it does not carry**: the word a hit line
        // holds is the direction chosen, and the focus bit is added after the choice — see the end of
        // [`word_to_press`]. So the two halves of the question are one line apart without this.
        match WAS_FOCUSED.load(Ordering::Relaxed) {
            true => "held",
            false => "not held",
        },
    );
    let of = was.counted;
    // **How many frames of this run were a tick already read**, which is the one thing in this dump that is
    // about the run rather than the frame — and it is here because it is what says whether the frame above is
    // one of them: see [`a_tick_read_again`].
    detail!(
        "autoplay: {} of the {} frame(s) asked for were a tick read again",
        AGAIN.load(Ordering::Relaxed),
        FRAMES.load(Ordering::Relaxed),
    );
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
    // **And which of them a beam will be lying across the band in**, which the numbers above cannot
    // hold: a beam is not one of the hazards they are counted from, and the frames a climb out of the band
    // is made on are frames an arm is across the strip the rules wait in.
    detail!(
        "autoplay: a beam lies across the band in each box from frame {}",
        of.closes
            .iter()
            .map(|closes| match *closes {
                never if never >= THE_FRAMES_AHEAD as u32 => "never".to_owned(),
                at => at.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", "),
    );
    detail!(
        "autoplay: {} bullet(s) no step of their own explained, {}{}",
        of.swept,
        match of.sweeping {
            Some(x) => format!("a beam being swept about x {x:.1}"),
            None => "too few to be a beam being swept".to_owned(),
        },
        match of.out_by {
            Some(out) => format!(
                "; the widest of them moved {:.2},{:.2} saying {:.2},{:.2}, in the game's own state {}",
                out.moved.0, out.moved.1, out.step.0, out.step.1, out.state,
            ),
            None => String::new(),
        },
    );
    detail!(
        "autoplay: the model had {} bullet(s), {} of them in another state and {} in a slot another one \
         had last frame; {} enem(ies), with {} left out for having no body and {} in a slot another one \
         had last frame; {} laser(s), {} of them still only a warning and {} of them fading{}{}{}{}",
        of.bullets,
        of.another_state,
        of.refilled_bullets,
        of.enemies,
        of.no_body,
        of.refilled,
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
        match of.widest_body {
            Some((slot, (x, y))) => {
                format!(", the widest body slot {slot}, half of it {x:.1},{y:.1}")
            }
            None => String::new(),
        },
        match of.beam_at {
            // **And whether that one was firing**, which a run inside a beam's box cannot be read without:
            // th08 arms four enemies with a lane apiece and fires them in turn, so a box the player stands
            // in says a life was taken only where the lane it belongs to was live.
            Some((slot, outside, along, firing)) => format!(
                ", the nearest beam slot {slot}, {outside:.1} outside its box and {along:.1} along it, {}",
                match firing {
                    true => "firing",
                    false => "not fired yet",
                },
            ),
            None => String::new(),
        },
        if THE_STEP_WAS_STALE.load(Ordering::Relaxed) {
            "; the focus bit changed on that frame, so the step was the one before's"
        } else {
            ""
        },
    );
    match was.nearest {
        Some(near) => detail!(
            "autoplay: nearest was {} at {:.1},{:.1} stepping {:.1},{:.1}, half of it {:.1},{:.1}{}",
            near.what,
            near.at.0,
            near.at.1,
            near.step.0,
            near.step.1,
            near.half.0,
            near.half.1,
            match near.feared {
                Some((x, y)) =>
                    format!(", feared out to {x:.1},{y:.1} by the frame it was nearest on"),
                None => String::new(),
            },
        ),
        None => detail!("autoplay: nothing was near it at all"),
    }
    let keys = Keys::of(game);
    for (way, judged) in WAYS.iter().zip(was.judgements) {
        detail!(
            "autoplay: the move {:#05x} was hit first on {}, on {} frame(s), warned on {}, \
             with {:.1} of room and {:.1} of it on its own next frame",
            keys.way(*way),
            match judged.clear() {
                true => "none".to_owned(),
                false => judged.first().to_string(),
            },
            judged.hits(),
            judged.warnings(),
            judged.room,
            judged.next_frame,
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
    for beam in was.beams.iter().flatten() {
        detail!(
            "autoplay: judged a laser {} from {:.1},{:.1} at {:.3} rad, half of it {:.1} across, \
             {:.1}..{:.1} along growing {:.1}, firing in {}, {} frame(s) into its phase{}",
            beam.slot,
            beam.origin.0,
            beam.origin.1,
            beam.angle,
            beam.half_across,
            beam.near,
            beam.far,
            beam.grows,
            beam.fires_in,
            beam.frames,
            match beam.fading {
                true => ", fading",
                false => "",
            },
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

/// **How far apart two lanes may stand, in halves of their own width, and still be one step of one sweep.**
///
/// **Because a beam a frame older than another is not always the frame before of it.** What the test below it
/// is for is a boss laying a row of lanes, one a frame; two lasers that have nothing to do with each other
/// can carry those same two ages, and a spacing read off such a pair is a band swept across the whole field.
///
/// **Four halves — twice a lane's own width — because that is what the two patterns measure.** th06c's Extra
/// boss lays them both ways, and the halves are `3.0` in each: crossing the field, one dump's origins were
/// `200.7`, `205.8`, `210.8`, `215.7`, `220.5`, `225.3`, `229.9`, `234.5`, all at `1.571` rad and
/// `4.8` apart, which is `1.6` halves; standing still, another's were one origin with the angles `1.296`
/// through `1.610` in steps of `0.039` rad, which at the player's own radius from it is `11.2` apart, or
/// `3.7` halves.
///
/// **And what the band this feeds cannot reach is the frame such a row is laid *over* the player on**,
/// which is worth having here before somebody writes a sixth rule for it. Those numbers are a front
/// covering `4.8` and `11.2` units a frame against the `4.0` a whole step does: a player the front has
/// reached cannot be got out from under it, so the frame is decided by where the run was standing dozens
/// of frames earlier — one dump of it has the player `1.373` rad from the origin against a front at
/// `1.453`, two lanes and so two frames, with every one of the nine hit on its own next frame and nothing
/// within reach but the lanes. **And the choice that would have saved it is not in those frames either**:
/// what a boss lays next is her script's, and nothing in the model reads a script. A person clears such a
/// pattern by having seen it; rules asked for a word off one frame at a time are playing it for the first
/// time every time.
///
/// **Five rules were written for that one frame and taken out again**, each with what it measured beside
/// the code it would go in: the lanes counted into `the_fire_in_each_box` by [`how_much_it_fills`] and
/// then by the share of a box they cover, the row's own x fed to [`a_sweep`]'s answer, the ground behind
/// the front wanted by [`where_to_be`], and the band ranked into the nine. So what is left for it is not
/// another weighing — it is the pattern known ahead of the frame, which is a change to what orb *reads*.
const A_SWEEPS_OWN_SPACING: f32 = 4.0;

/// **The lane laid the frame before this one, where one was** — the beam whose phase is one frame further on
/// than this beam's and whose origin is nearest to it.
///
/// **The phase and not the slot, which is what makes a sweep readable at all.** A boss laying a row of lanes
/// lays each in whatever slot is free, so nothing about the slots says which lane came before which; what
/// says it is `game::Beam::frames`, the count each carries of how long it has been in its phase.
///
/// **And nothing where this beam is the oldest of them**, which is the one lane with nothing behind it: a
/// sweep is read from the pair and a single lane says nothing about where the next one goes.
fn the_lane_laid_before(frame: &Frame, beam: &Beam) -> Option<Beam> {
    frame
        .beams
        .iter()
        .filter(|other| other.frames == beam.frames + 1)
        .min_by(|a, b| {
            let apart = |it: &Beam| {
                (it.origin.0 - beam.origin.0).powi(2) + (it.origin.1 - beam.origin.1).powi(2)
            };
            apart(a).total_cmp(&apart(b))
        })
        .copied()
}

/// **The d-pad of the safest of the nine one-frame moves**, or the one towards where the rules want to be
/// where nothing is near enough to dodge.
///
/// **Or the nearest to where it wants to be and nothing else, in a chapter drawn to be hit in.** Everything
/// below is still worked out there — the projection, the boxes, what came nearest — because that is what the
/// line beside a hit is made of, and a hit taken on purpose is read the same way as any other.
/// **The frame's own call of [`steer`], which is the one that puts what it read into [`REMEMBERED`].**
///
/// What a hazard moved is read against that row, so exactly one call a frame may replace it — see the put
/// at the end of `steer`, which has what a second one cost.
const THE_FRAMES_OWN: bool = true;

/// And a call asked again inside the same frame, which leaves the row where the frame's own call left it.
const ASKED_AGAIN: bool = false;

/// **Where a recipe tells the rules to stand** — see `recipe::Step::StandAt`. Either axis on its own or
/// both: the one left out stays the rules' own errand, so a `y` alone is a height to play an attack from
/// with where along it left to them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standing {
    pub across: Option<Across>,
    pub y: Option<f32>,
}

/// **And which way a told x is said**, there being two.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Across {
    /// An x across the field.
    At(f32),
    /// At least this far from the boss's own x, on the side of her the run is on where the whole distance
    /// fits inside the field and the other side where it does not. With no boss on the screen the rules
    /// run their own errands.
    AwayFromTheBoss(f32),
}

/// **The way a recipe named for the card that is up**, or nothing — see `recipe::Step::Dodge`. A static for
/// the reason [`TOLD_TO_STAND_AT`] is one.
static DODGE: Mutex<Option<Dodge>> = Mutex::new(None);

/// Tells the rules which way to dodge the card that is up, or none.
pub fn dodge_with(dodge: Option<Dodge>) {
    *DODGE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = dodge;
}

/// The way the rules were told to dodge the card that is up.
fn the_dodge() -> Option<Dodge> {
    *DODGE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// **How many frames the card that is up has run**, or nothing — see `Reached::card_frames`. A static for the
/// reason [`DODGE`] is one.
static CARD_FRAMES: Mutex<Option<i32>> = Mutex::new(None);

/// Tells the rules how far into the card that is up they are.
pub fn the_card_has_run(frames: Option<i32>) {
    *CARD_FRAMES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = frames;
}

/// **The stage's own frame, and the one the rules were last asked on**, -1 for none — see [`the_rules_count`].
/// Atomics rather than a lock, both being read and written on every frame a bot plays.
static STAGE_FRAMES: AtomicI64 = AtomicI64::new(-1);
static STAGE_FRAMES_BEFORE: AtomicI64 = AtomicI64::new(-1);

/// Whether the shot was let go on the ask before this one — see [`word_to_press`].
static LET_GO_LAST_ASK: AtomicBool = AtomicBool::new(false);

/// Tells the rules which frame of the stage this is, and nothing off a run.
pub fn the_stage_has_run(frames: Option<u32>) {
    STAGE_FRAMES.store(frames.map_or(-1, i64::from), Ordering::Relaxed);
}

/// **This ask's number in [`FRAMES`], the count the shot's let-go is timed by** — the stage's own frame on an
/// ask its clock has moved since, and one on from the ask before on one where it stands.
///
/// **The stage's frame where it moves**, so that the let-go falls on the same stage frames however long the
/// rules had been playing before: counted from the rules' own first frame, a run picked up at th06c's Extra
/// chapter 7 let the shot go on stage frame `8385` where the whole run it was written by let it go on `8411`,
/// and the two parted there — the game's own numbers agreeing up to it, and a shot that goes on another frame
/// being an enemy that falls on another, the generator's next draw with it (`8438`). What was measured off
/// such a pick-up was not the run it was cut from: the run was hit twice in chapter 8 where all four of the
/// pick-ups went through.
///
/// **And not set there only where the clock starts or goes back**, counting on by the asks between, which it
/// was: a run asks more often than its clock moves — a line of dialogue stands the clock for as long as it is
/// up — so that count was a history of the whole stage, and a pick-up shows the rules only the last frames of
/// its playback (`runtime::THE_FRAMES_THE_RULES_ARE_SHOWN`). Shown the last 600 of a pick-up of that chapter 7,
/// the rules let the shot go otherwise than the run on 20 of them and parted from it on the word at `8041`.
///
/// **And not the stage's clock alone**, which was tried: it stands still under the pause menu and at a stage's
/// own clear screen, the wait the let-go is for, and a game can ask the rules more than once in a frame — timed
/// by the stage's frame, the laid-out th07 let the shot go on sixty frames running with its clock standing at
/// nought. Counted on by the asks while it stands, those frames let it go as they did.
fn the_rules_count() -> u32 {
    let stage = STAGE_FRAMES.load(Ordering::Relaxed);
    let before = STAGE_FRAMES_BEFORE.swap(stage, Ordering::Relaxed);
    if stage >= 0 && stage != before {
        FRAMES.store(stage as u32, Ordering::Relaxed);
    }
    FRAMES.fetch_add(1, Ordering::Relaxed)
}

/// How far into the card that is up the rules are.
fn the_cards_frames() -> Option<i32> {
    *CARD_FRAMES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// **Whether the card that is up is one the rules play focused for the rest of**, the gauge having stayed short
/// of its human end for [`HELD_OFF_THE_HUMAN_END_FOR`] of its frames running while the stage's norm was unmet —
/// and nothing between cards, or past the norm, or on a game with no such gauge. Said as the card is given over.
fn held_off_the_human_end(frame: &Frame) -> bool {
    let mut held = HELD_OFF
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (since, last, given_over) = &mut *held;
    let Some(frames) = the_cards_frames() else {
        *held = (None, 0, false);
        return false;
    };
    // A card's clock going back is the next card.
    if frames < *last {
        (*since, *given_over) = (None, false);
    }
    *last = frames;
    if *given_over {
        return true;
    }
    match (frame.shoots_a_boss_unfocused, frame.at_the_gauges_human_end) {
        (true, Some(false)) => {
            let began = *since.get_or_insert(frames);
            if frames - began >= HELD_OFF_THE_HUMAN_END_FOR {
                *given_over = true;
                detail!(
                    "autoplay: the gauge has been short of its human end for {} frame(s) of this card, \
                     and the rest of it is played focused",
                    frames - began
                );
            }
        }
        _ => *since = None,
    }
    *given_over
}

/// **How far behind the lane a turning laser goes live in [`Dodge::BehindTheLaser`] stands**, square to it and
/// on the player's own row.
///
/// **Because that is how a laser turned onto the run while it is still drawn is got through, and the rules'
/// own dodging is how it is lost.** One card of the kind, read every frame of three attempts: each laser
/// appears `60` frames before it goes live — its first phase `60` long and live at `60` — pointing
/// `60 × π/400` to one side of the player, and turns `π/400` a frame onto where the player stood when it
/// appeared. It goes live there and turns on past for the `120` frames of its second phase, `54°` of sweep.
/// So the one thing a laser still being drawn says is the lane it goes live in, and the side of that lane it
/// has come from is the side it is turning away from: a short step towards the side it appears on is the way
/// through. The rules instead walked away from the laser being drawn at them, the way it looks to be coming,
/// and were ahead of it when it went live: one session lost that card `70` times over.
///
/// **Twelve**, which is the lane's own quarter-width of `4` and the player's half, and the rest a step and a
/// half of the rules' unfocused `4.0` — room to stand still in while the laser is drawn across the place and
/// passes on. The pick-up that took the card first time stood `9.6` to `22.7` behind each of its five lasers.
const BEHIND_THE_LASER_BY: f32 = 12.0;

/// **How wide [`Dodge::OutOfTheFollowingLaser`] keeps the run of a beam aimed and not yet firing**, half of it:
/// the width the beam fires at, where nothing else says how wide it will be.
///
/// **Because the width a beam is read at before it fires is the narrowest it could be, and this one is not
/// that.** One card of the kind, read every frame off the enemies from outside: the beam is aimed where the
/// player stood when it appeared and held there for `118` frames of warning, then fires `240` wide — its
/// behaviour's own width — and turns `0.0036` rad a frame onto the player until it points at them. The run
/// stood `83.7` off its middle, clear of the `80` it was read at while it warned and inside the `120` it hit
/// at, and was hit the frame it fired.
const A_FOLLOWING_LASERS_HALF_ACROSS: f32 = 120.0;

/// **How far left of the middle [`Dodge::LeftOfTheMiddle`] waits** — `115` on a field whose middle is `192`,
/// which is the second fifth of it — at the low line, the height being the rules' own.
///
/// **The emptier side, read off the fire itself.** One card of the kind sends its bullets out in rings that fill
/// the field, and fourteen attempts at it told nothing died at exactly `376.0,432.0`, the bottom-right corner,
/// with `0 of the nine clear` for the sixty frames before each hit — the errand's own `the emptiest box` having
/// walked them along the floor to the right wall. The five boxes across read `57.0, 76.9, 56.7, 60.8, 150.0`,
/// `62.0, 71.2, 117.0, 83.6, 161.0`, `54, 50, 125, 95, 174` and `49, 56, 96, 65, 166` on the hits: the rightmost
/// fifth carries the most on every one of them and the two on the left the least.
///
/// **No height of its own**, a told one being held no lower than the low line: the `380` the recipes told this
/// card and the one below with the x was the low line's `368` on every frame of the no-bomb clear that passed
/// both, `wanted the place it was told to stand at 115,368` on 1,576 of them.
const LEFT_OF_THE_MIDDLE_BY: f32 = 77.0;

/// **How far right of the middle [`Dodge::RightOfTheMiddle`] waits** — `288` on a field whose middle is `192` —
/// at the low line, for the reason [`LEFT_OF_THE_MIDDLE_BY`] is.
///
/// **The emptier side, read off the fire itself**, the way that one is. One card of the kind lays its bullets
/// across the field and fires them, and told nothing died fifty-five times along the floor, seven of them at
/// `63.0,432.0` with over a thousand bullets up. Eight of the hits through it read the five boxes across as `9.0,
/// 143.7, 33.3, 0.0, 0.0`, `451.2, 220.2, 155.7, 140.8, 104.0`, `661.5, 986.8, 307.4, 114.5, 155.5`, `60.8,
/// 142.5, 84.4, 0.0, 0.0` and more of the same: the two on the left carry the most on every one of them and the
/// two on the right the least, twice nothing at all. Told the middle the run died seven times at `202.6,432.0`,
/// and told `96` eight times at `127.3,432.0` and `103.7,428.0`, the densest fifth.
const RIGHT_OF_THE_MIDDLE_BY: f32 = 96.0;

/// **The place a way of dodging waits at**, for the ways that are one — and nothing for the ways that are not,
/// whose place is the rules' own errand.
///
/// **[`Dodge::InTheMiddle`] is the field's own middle, with the height left to the rules**: one card of the kind
/// lays lasers up the field from its floor with aimed fire between them, four of them `53` apart and `8`
/// across. Told nothing the rules walked the whole field across it, `77` to `227.3,366.6`, and died there with
/// 202 bullets up and `nowhere open to it` fourteen attempts running; told `40` a launch died eleven times at
/// `42.8,362.2`, and told `96` fourteen times at `90.4,379.0`, the boxes reading `16.0, 44.6, 8.1, 9.5, 5.0` on
/// every one of those — the fifth the run stood in carrying five times what the three to its right did. So both
/// sides have been tried and each ended at the place told, and the middle is the one place with room either
/// side of it.
///
/// **[`Dodge::BehindTheLaser`] waits in the middle too, between its lasers** — the place behind one being
/// wanted while it is drawn, see [`where_to_be`]. The lasers come from either side by turns and the side one
/// comes from is the side the run steps to: the strategy wiki's `奇数回目のレーザーは右から … 偶数回目のレーザー
/// は左から`, with `レーザーが来る方向に余裕を持っていれば大丈夫` beside it. The rules' own errand put the run at a
/// wall instead — one Extra wanted `the emptiest box at 56` on 2,726 of the card's frames and was hit at `x = 8`
/// five times and at `376` three, the place behind each laser clamped to the wall it came in from; and an Extra
/// that wanted the middle through the rule under a boss overhead flipped between it and the boxes beside it and
/// was pushed to the walls all the same. So the middle is a place the way holds, and not the boxes' answer.
fn the_place_it_holds(dodge: Dodge, field: (f32, f32, f32, f32)) -> Option<Standing> {
    let middle = field.0 + field.2 / 2.0;
    let (x, y) = match dodge {
        Dodge::BehindTheLaser => (middle, None),
        Dodge::OutOfTheFollowingLaser
        | Dodge::RoundTheField
        | Dodge::RoundASmallSquare
        | Dodge::SideToSide
        | Dodge::WithoutTheItems
        | Dodge::Focused
        | Dodge::ImperishableShooting
        | Dodge::Laevateinn
        | Dodge::Clock
        | Dodge::AndThenThereWereNone => return None,
        Dodge::InTheMiddle => (middle, None),
        Dodge::VisionaryTuning => (
            middle,
            Some(field.1 + field.3 - crate::cards::VISIONARY_TUNING_ABOVE_THE_FLOOR),
        ),
        Dodge::LeftOfTheMiddle => (middle - LEFT_OF_THE_MIDDLE_BY, None),
        Dodge::RightOfTheMiddle => (middle + RIGHT_OF_THE_MIDDLE_BY, None),
    };
    Some(Standing {
        across: Some(Across::At(x)),
        y,
    })
}

/// **How wide a beam is to be kept out of**: its own half-width, save for one aimed and not yet firing while
/// the rules dodge by keeping out of a following laser — see [`A_FOLLOWING_LASERS_HALF_ACROSS`].
fn the_half_across(beam: &Beam) -> f32 {
    match the_dodge() == Some(Dodge::OutOfTheFollowingLaser)
        && beam.fires_in == FIRES_WHEN_NOTHING_SAYS
    {
        true => beam.half_across.max(A_FOLLOWING_LASERS_HALF_ACROSS),
        false => beam.half_across,
    }
}

/// **The lane a turning laser still drawing its warning goes live in**, and which way it is turning — what
/// [`Dodge::BehindTheLaser`] stands behind.
#[derive(Clone, Copy, Debug)]
struct GoesLive {
    origin: (f32, f32),
    angle: f32,
    turn: f32,
    fires_in: i32,
}

impl GoesLive {
    /// **The x on the row at `y` that is `distance` behind the lane**, measured square to it, or nothing
    /// where the lane does not cross that row ahead of its origin.
    fn behind(self, y: f32, distance: f32) -> Option<f32> {
        let (sin, cos) = self.angle.sin_cos();
        let down = y - self.origin.1;
        let along = down / sin;
        if !along.is_finite() || along <= 0.0 {
            return None;
        }
        let leaving = self.turn.signum() * down.signum();
        Some(self.origin.0 + along * cos + leaving * distance / sin.abs())
    }
}

/// **Where a recipe told the rules to stand**, or nothing. Read by [`where_to_be`], which puts it in place
/// of every errand of the rules' own while it is set, the dodging left as it is. A static and not a field
/// of the frame, because it is a recipe's telling and not a reading of the game, and it stands across
/// frames the way the bomb's allowance does.
static TOLD_TO_STAND_AT: Mutex<Option<Standing>> = Mutex::new(None);

/// Tells the rules where to stand, or frees them of it — a recipe's `stand_at`.
pub fn stand_at(standing: Option<Standing>) {
    *TOLD_TO_STAND_AT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = standing;
}

/// **How many times the chapter in flight has been retried**, which moves where the rules wait — see
/// [`aside_on`].
static ATTEMPT: AtomicU32 = AtomicU32::new(0);

/// Tells the rules which attempt at its chapter the run is on, nought for the one that reached it.
pub fn this_attempt(attempt: u32) {
    ATTEMPT.store(attempt, Ordering::Relaxed);
}

/// **How far to the side of the boss and of the middle the rules wait on an attempt**: nothing on the
/// first, and a draw of up to [`ASIDE_AT_MOST`] either way on each retry of it.
///
/// **Because a retry is otherwise the attempt before it over again.** The chapter's snapshot puts the game
/// back as it was, and the rules are a function of what they read off it, so an attempt at a chapter the
/// rules lose is lost the same way: one card was lost eight attempts running at the same three hits,
/// `136.6,421.2`, `203.6,421.9` and `39.3,429.2`, where the card runs 4,200 frames and an attempt
/// lasted 1,230. Where they wait is what is moved, and not how they dodge — the dodge is what keeps an
/// attempt alive at all, and a place a recipe told them is somebody's judgement and not theirs to vary. The
/// place a way of dodging a card holds is theirs, and moves with the boss and the middle.
///
/// **A draw and not a launch's own number**, for the reason the hits are one: the same recipe plays the
/// same run every launch — see `hits::drawn`.
pub fn aside_on(attempt: u32) -> f32 {
    if attempt == 0 {
        return 0.0;
    }
    (crate::hits::drawn((0, 0), attempt, crate::hits::WAITING_ASIDE) * 2.0 - 1.0) * ASIDE_AT_MOST
}

/// **A quarter of the field either way**, which on a field 384 wide keeps a boss at its middle — where that
/// card is fired from — inside [`IN_FROM_THE_SIDE`] of both edges, the wait moved and never clamped.
const ASIDE_AT_MOST: f32 = 96.0;

fn steer(frame: &Frame, keys: &Keys, hazards: Hazards, focused: bool, remembers: bool) -> Steered {
    let field = frame.field;
    let at = frame.at;
    // **The step the frame this chooses for will actually take**, which is the pair for the focus key as
    // *this* answer will leave it and not as the last one did — see `Frame::focused_straight`, and
    // [`word_to_press`], which asks twice to close the circle between the two.
    let (the_straight, the_diagonal) = match focused {
        true => (frame.focused_straight, frame.focused_diagonal),
        false => (frame.straight, frame.diagonal),
    };
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
            the_diagonal
        } else {
            the_straight
        };
        // **Clamped to the field's own edges and not to the line the bot waits on.** Holding the projection off
        // the bottom by [`ABOVE_THE_BOTTOM`] was tried, so that a dodge could not spend the floor — two hits in
        // one run were the player walking from `420` down to the clamp at `432` and sitting pinned there until
        // every one of the nine was hit on the frame it was judged on. It cost four of these tests at once:
        // keeping the player low needs the moves that reach the low line to be moves, and the longest way out of
        // a pattern coming from above is sometimes downwards. The strip below the line is a bad place to be and
        // that is not the same as a place no move may land in.
        //
        // **And holding it under [`the_band`]'s own top while the player is inside the band was tried as
        // well, which is the same idea the other way up and the last thing about the place that is neither
        // a preference between the nine nor a wanting.** A move that leaves the band is then followed as a
        // move that stopped at the boundary, so climbing out counts as escaping nothing — and what that
        // costs is the escape itself. One Extra attempt with it in was hit six times against four and read
        // `14` frames with nothing open against `10`, four of the six still outside the band; the hits are
        // one sample each and inside the spread of `4` to `7` a build, where the shut count is not — a
        // model that will not follow a move upward has three fewer of them on the frames that need them
        // most, and upward is where a pattern that comes from above is left.
        std::array::from_fn::<_, THE_NEXT_FRAME_AND_LOOK_AHEAD, _>(|step| {
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
    let mut kept_beams = [None; HELD_BEAMS];
    // The bullets no step of their own explains, and where across the field they are — see [`a_sweep`].
    let mut swept = 0u32;
    let mut swept_x = 0.0;
    // **And the frame a beam will be lying across the band in each of the boxes on**, or
    // [`THE_FRAMES_AHEAD`] for a box none of them reaches inside the window — see the beam loop below,
    // which is where the motion it takes to answer that is already worked out.
    let mut closes = [THE_FRAMES_AHEAD as u32; BOXES_ACROSS];
    // **And the turning warning that goes live soonest**, for [`Dodge::BehindTheLaser`].
    let mut goes_live = None::<GoesLive>;
    let behind_a_turning_laser = the_dodge() == Some(Dodge::BehindTheLaser);
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
                       grows: (f32, f32),
                       (tx, ty): (f32, f32),
                       // **Whether what this box is takes a life yet**, which is a warning where it does
                       // not: a move is not shut by it, and a place to wait is not chosen inside it. See
                       // the harmless bullet's own arm — a mesh a game turns solid is ground to cross and
                       // no ground to stand on, and a run that stood on one died eleven times at
                       // `207.6,384.9` with `a bullet at 208.0,384.0 stepping 0.0,0.0` under it.
                       warning: bool| {
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
                let read = (wx, wy);
                let (wx, wy) = (wx + grows.0 * ahead, wy + grows.1 * ahead);
                // **And the step's own change carried with it, which a straight line is wrong by the whole
                // of.** A hazard whose step is the same every frame is where its step says it will be; one
                // whose step changes by a hair a frame is somewhere else entirely by the end of the frames
                // looked at, and that hair is measured already — see [`A_STEP_OF_ITS_OWN`], whose own note
                // says a turn under it is *projected straight and wrong by the whole of the turn twelve
                // frames out*, and names the three hits of stage 6's turning card that were read as `12.2 of
                // room` with nothing near. Each frame adds one more turn to the step, so what the frames add
                // up to is the turn over the triangle of them.
                let curve = ahead * (ahead - 1.0) / 2.0;
                let (dx, dy) = (
                    (x - (hx + sx * ahead + tx * curve)).abs(),
                    (y - (hy + sy * ahead + ty * curve)).abs(),
                );
                // What was left on the tighter of the two axes, which is negative on both where the boxes
                // overlap — so the larger of the two is the room, and a room below zero is the hit.
                let room = (dx - (mine_x + wx)).max(dy - (mine_y + wy));
                // **And the same against where it is now, on the move's own next frame.** A step is what a
                // hazard claims and not what the game is obliged to apply: the order inside one frame is the
                // player's job, then the hazard's, then the test — so a hazard that does not move is one the
                // test compares the *moved* player with where it already stands, and a landing has to be clear
                // of both places to be clear at all.
                //
                // **Which costs a lane only where a hazard is receding.** Coming at the player the projected
                // place is the tighter of the two and this changes nothing; going away it is the space just
                // vacated, and that space is exactly what a bullet that stops takes back. A Normal run of
                // th06c was hit at `138.5,381.1` on a move judged to have `2.2 of room`: bullet 461 stood at
                // `141.9,384.6` claiming `-1.0,3.0` a frame, did not move at all, and the run stepped `2.0`
                // right into where the projection had it leaving.
                // **On the move's own first landing.**
                let room = match step == 0 {
                    true => {
                        let still =
                            ((x - hx).abs() - (mine_x + wx)).max((y - hy).abs() - (mine_y + wy));
                        room.min(still)
                    }
                    false => room,
                };
                if room < judged.room {
                    judged.room = room;
                    *nearest = Some(Nearest {
                        what,
                        at: (hx, hy),
                        step: (sx, sy),
                        half: read,
                        feared: match (wx, wy) == read {
                            true => None,
                            false => Some((wx, wy)),
                        },
                    });
                }
                if step == 0 {
                    judged.next_frame = judged.next_frame.min(room);
                }
                // **[`ROOM_TO_SPARE`] and not nought**: a frame the move only just clears is a frame it is
                // treated as losing, which is what keeps it out of the choice as well as out of the test.
                if room < ROOM_TO_SPARE {
                    match warning {
                        true => judged.warn(step),
                        false => judged.hit(step),
                    }
                }
            }
        }
    };
    for hazard in &frame.hazards {
        let Hazard {
            kind,
            slot,
            frames,
            at: place,
            step,
            half,
        } = *hazard;
        match kind {
            // **And one out of its hitting state left out where the game says it is harmless** — see
            // `Frame::a_bullet_not_hitting_is_harmless`, which is th08 and no other game: there a bullet
            // coming out cannot take a life and one that comes out over the run is turned harmless for
            // good, so a mesh of them is ground to stand in rather than a wall. Kept as a hazard
            // everywhere else, a bullet that will hit in a moment being one to be gone from.
            Kind::Bullet { hitting: false, .. } if frame.a_bullet_not_hitting_is_harmless => {
                counted.another_state += 1;
                note(
                    &mut now.bullets,
                    slot,
                    (place, step.unwrap_or((0.0, 0.0)), frames),
                );
                // **Judged as a warning and not left out**, which is the whole of the difference between
                // ground to cross and ground to stand on: such a bullet takes no life this frame and
                // will take one the frame the game turns it solid, so a move may go through it and a
                // place to wait may not be inside it. **A run that treated them as nothing stood in one
                // and died eleven times at `207.6,384.9`**, the dump naming `a bullet at 208.0,384.0
                // stepping 0.0,0.0` — a bullet the card had placed there and then made solid.
                against(
                    "a bullet not hitting yet",
                    slot,
                    place,
                    step.unwrap_or((0.0, 0.0)),
                    half,
                    (0.0, 0.0),
                    (0.0, 0.0),
                    true,
                );
            }
            Kind::Bullet { hitting, state } => {
                if !hitting {
                    counted.another_state += 1;
                }
                counted.bullets += 1;
                // Its own update adds its step to its position every frame, so where it will be in a frame's
                // time is where it is plus as many steps as there are frames to look.
                let step = step.unwrap_or((0.0, 0.0));
                let (moved, turn) = match what_a_slot_held(&last.bullets, slot) {
                    Some((place_was, step_was, was)) if the_same_thing(frames, was) => (
                        (place.0 - place_was.0, place.1 - place_was.1),
                        (step.0 - step_was.0, step.1 - step_was.1),
                    ),
                    // **The step it claims for a bullet the slot was not holding last frame**, so that one
                    // just made is not read as having crossed the field in a frame — whether the slot was
                    // free then or held another bullet, which is what [`the_same_thing`] tells apart. A
                    // difference across two of them is the distance between them, and the fear below spreads
                    // a box over the whole of that on every frame looked at: a card that lays a lattice of
                    // bullets and cancels it again and again had a hit inside one read
                    // `a bullet at 383.4,296.9 stepping 0.0,0.0, feared out to 385.4,2.0` — the width of the
                    // field, laid across the row the run was standing in.
                    held => {
                        if held.is_some() {
                            counted.refilled_bullets += 1;
                        }
                        (step, (0.0, 0.0))
                    }
                };
                note(&mut now.bullets, slot, (place, step, frames));
                let scripted = (moved.0 - step.0).abs() > A_STEP_OF_ITS_OWN
                    || (moved.1 - step.1).abs() > A_STEP_OF_ITS_OWN
                    || turn.0.abs() > A_STEP_OF_ITS_OWN
                    || turn.1.abs() > A_STEP_OF_ITS_OWN;
                if scripted {
                    swept += 1;
                    swept_x += place.0;
                    // The widest of them kept, as the pair it disagreed on — see `Counted::out_by`.
                    let out = (moved.0 - step.0).abs().max((moved.1 - step.1).abs());
                    if counted.out_by.is_none_or(|was| {
                        out > (was.moved.0 - was.step.0)
                            .abs()
                            .max((was.moved.1 - was.step.1).abs())
                    }) {
                        counted.out_by = Some(OutBy { moved, step, state });
                    }
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
                    // **And the turn it is carrying, where its own step is what moves it.** A scripted one is
                    // feared over the box above instead, there being nothing to carry.
                    if scripted { (0.0, 0.0) } else { turn },
                    false,
                );
            }
            // **And the bodies, which are as solid as a bullet.** Which is what a run on its way to an item
            // walks into: the item was dropped by something, and that something is still standing over it.
            //
            // **And with where it is going**, which is the difference from last frame and not a field: nothing
            // on a body is a step, its script being what moves it. A body read as standing still is one the
            // model leaves room beside for as long as it takes to arrive — a launch was hit beside stage 2's
            // midboss with 4.7 of room, which is less than a body covers in a frame.
            //
            // **And only the ones with a body to run into.** The game gates its own collision on whether an
            // enemy is solid, so one with that down cannot touch the player at all — which th07's stage 6 boss
            // relies on: it fires an aimed shot off an enemy its script keeps *at* the player, and a model that made
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
                // **The widest of them kept for the hit line** — see [`Counted::widest_body`]: a body
                // wider than the field is the model and not the game, and its slot is the only thing a
                // reader can take to a debugger.
                if counted.widest_body.is_none_or(|(_, (was, _))| half.0 > was) {
                    counted.widest_body = Some((slot, half));
                }
                // **And feared rather than projected where a script is swinging it.** Stage 6's turning beam is a
                // line of these, and a straight line from last frame's motion is wrong by the whole of the
                // sweep — so the test is whether that motion is itself changing, and what is kept away from is
                // everywhere it could reach.
                //
                // **And nothing at all where the slot has been given to another enemy since** — see
                // [`the_same_thing`] and `game::Hazard::frames`: the difference is then the distance between
                // two enemies, which the fear below spreads a box over on every frame looked at.
                let (went, swung) = match what_a_slot_held(&last.bodies, slot) {
                    Some((place_was, went_was, was)) if the_same_thing(frames, was) => {
                        let went = (place.0 - place_was.0, place.1 - place_was.1);
                        (went, (went.0 - went_was.0, went.1 - went_was.1))
                    }
                    held => {
                        if held.is_some() {
                            counted.refilled += 1;
                        }
                        ((0.0, 0.0), (0.0, 0.0))
                    }
                };
                note(&mut now.bodies, slot, (place, went, frames));
                // **Projected along its own motion with the change in that motion carried, and feared over
                // the change and not over the motion.** All three are measured on a body — where it went
                // last frame, and by how much that differed from the frame before — so where it will be is
                // the first two and what is not known about that is the third.
                //
                // **Feared over how fast it is going, a body that is going anywhere shuts every move.** The
                // box was spread by `went` for each of the forty frames wherever `swung` cleared
                // [`A_STEP_OF_ITS_OWN`], which is a sixth of a unit — so an enemy moving `8.0,9.9` a frame
                // and turning at all reached the rules as a box `320.2,393.0` across, on a field 384 by 448.
                // A box that size holds all nine landings at once and holds them on the same frame, so the
                // ranking has nothing to choose between them and the run stands where it is: eight attempts
                // at one last stage's card died at `8.0,362.3`, the left wall, every one of them pressing
                // down into it with `nowhere open to it`.
                against(
                    "an enemy",
                    slot,
                    place,
                    went,
                    half,
                    (swung.0.abs(), swung.1.abs()),
                    swung,
                    false,
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
                    (0.0, 0.0),
                    false,
                );
            }
        }
    }
    // **And the beams, which are the one hazard a distance cannot stand in for**: a beam is a place, not a
    // point, so a move is judged inside the beam's own frame — turn the landing about the origin by the angle
    // and it is *along* and *across*, which is the shape the game's own test makes of it.
    //
    // **A warning line is left for the frames it will be firing on and not for being drawn**: the game's own test is
    // gated on the beam's own countdown, so one still drawing its warning hits nothing at all — and a bot that
    // made the warning a hazard would keep the player out of the one lane a pattern leaves open. The warning is
    // still where the beam *will* be, though, which is the whole reason anybody moves off one: the frames
    // looked at that fall after it fires are frames to be elsewhere for. So the count decides, frame by frame,
    // and neither answer has to be guessed.
    //
    // The angle and the origin are the ones it has now carried on by the turn and the distance it made last
    // frame — nothing in a beam's own update moves either, so one whose sweep speeds up or turns back is a
    // beam this is late about.
    for beam in &frame.beams {
        let Beam {
            slot,
            origin: (ox, oy),
            angle,
            near,
            far,
            grows,
            half_across: _,
            fires_in,
            fading,
            frames,
        } = *beam;
        let across = the_half_across(beam);
        counted.lasers += 1;
        // **And how far outside the nearest beam's own box the player is standing**, which is the number a
        // hit that names nothing near enough to be it wants: nought or less is a run inside a beam the
        // model was holding, and a large one is a hit the beams had no part in. Measured where the player
        // is and where the beam is now, the projection below being about the nine moves instead.
        {
            let (sin, cos) = angle.sin_cos();
            let (dx, dy) = (frame.at.0 - ox, frame.at.1 - oy);
            let along = dx * cos + dy * sin;
            let outside = ((-dx * sin + dy * cos).abs() - across)
                .max(near - along)
                .max(along - far);
            if counted.beam_at.is_none_or(|(_, was, ..)| outside < was) {
                counted.beam_at = Some((slot, outside, along, fires_in <= 0));
            }
        }
        if let Some(room) = kept_beams.iter_mut().find(|kept| kept.is_none()) {
            *room = Some(*beam);
        }
        if fires_in > 0 {
            counted.warnings += 1;
        }
        if fading {
            counted.fading += 1;
        }
        // **And the slot's own reading is the beam's only where the beam in it is the same one**, which is
        // one frame more of its phase and nothing else — see `game::Beam::frames`.
        let was = what_a_slot_held(&last.beams, slot).filter(|(.., then)| then + 1 == frames);
        let turn = turned(was.map(|(_, angle, _)| angle), angle);
        let went = moved(was.map(|(origin, ..)| origin), (ox, oy));
        let counted_down = fires_in > 0 && fires_in != FIRES_WHEN_NOTHING_SAYS;
        if counted_down && turn != 0.0 && goes_live.is_none_or(|it| fires_in < it.fires_in) {
            let by = fires_in as f32;
            goes_live = Some(GoesLive {
                origin: (ox + went.0 * by, oy + went.1 * by),
                angle: angle + turn * by,
                turn,
                fires_in,
            });
        }
        // **And the lane laid the frame before this one, where a boss is laying a row of them** — see
        // [`the_lane_laid_before`], which is what says another is coming and how far on it will be.
        let before = the_lane_laid_before(frame, beam).map(|it| (it.origin, it.angle.sin_cos()));
        // **And which of the boxes across the field this beam will be lying across the band in, at any frame
        // inside [`THE_FRAMES_AHEAD`].** Which is the one thing [`the_fire_in_each_box`] has no shape
        // for, and the shape it wants is this one: a truth per box and not an amount of anything. Both
        // measures tried instead were the wrong scale for a lane — [`how_much_it_fills`] squares, so a
        // warning `768` across reads as `36864` in the box its axis crosses, and the share of a box a lane
        // covers puts `134` in one against `clearly_fuller`'s bar of `9`, which makes any beam over the bot a
        // reason to cross the field. A truth has no scale to be wrong about.
        //
        // **And it answers the frames a climb out of the band is made on.** One of those, dumped at
        // `76.3,292.2`, has the three upward moves clear and the other six hit, with the nearest thing a
        // laser `25` below the player: an arm from `173.4,288.0` at `2.847` rad, `216` long and `12` across,
        // running down to the left across the strip the rules wait in at `1.2` units a frame. The band was
        // being closed from underneath, and nothing the rules read could see that the place they were
        // waiting in was about to have a beam laid across it.
        //
        // **And a beam closes a box across and not along, which is the whole of the measure.** The arm above
        // runs `17°` below the horizontal, so inside the band it spans `146` units of x — two boxes' worth of
        // width, with nowhere left in them to stand. A beam running *down* through the band spans a few units
        // of x however long it is, and this tree has already decided what to do about one of those: nothing.
        // `the_bot_leaves_a_laser_that_is_turning_onto_it` stands the player under a beam `19` away pointing
        // almost straight down and asks for no move at all, and it is the frame that says so — marking the
        // box its crossing falls in made the bot leave, a box being `77` across where that beam covers `7`.
        // So what is asked is how much of a box's own width the beam takes inside the band, against
        // [`A_BOX_IS_BLOCKED_AT`].
        //
        // **Sampled along the beam and not solved for the crossing**, so that the same arithmetic answers an
        // arm at any angle, and stepped by the beam's own half-width, which is as fine as the thing sampled.
        // **And the frame it arrives on and not only whether it does**, which is what the truth alone could
        // not say: eight arms of this boss close all five boxes on most of the frames one attempt dumped, so
        // *closed* tells them nothing apart where *closed in three frames* against *closed in thirty-nine*
        // is a place to wait and a place not to.
        let (field, band) = (frame.field, the_band(frame.field));
        let wide = field.2 / BOXES_ACROSS as f32 * A_BOX_IS_BLOCKED_AT;
        for ahead in 0..THE_FRAMES_AHEAD {
            let at_frame = ahead as u32;
            let ahead = ahead as f32;
            // A beam still only drawn hits nothing, and by this frame it either fires or it does not.
            if fires_in as f32 > ahead {
                continue;
            }
            let (at, (sin, cos)) = (
                (ox + went.0 * ahead, oy + went.1 * ahead),
                (angle + turn * ahead).sin_cos(),
            );
            let end = far + grows * ahead;
            let mut span = [(f32::MAX, f32::MIN); BOXES_ACROSS];
            let mut along = near;
            while along <= end {
                let (x, y) = (at.0 + cos * along, at.1 + sin * along);
                if y >= band - across {
                    let (from, to) = &mut span[which_box(field, x)];
                    *from = from.min(x);
                    *to = to.max(x);
                }
                along += across.max(A_SMALL_ONE);
            }
            for (box_, (from, to)) in span.iter().enumerate() {
                if to - from >= wide {
                    closes[box_] = closes[box_].min(at_frame);
                }
            }
        }
        note(&mut now.beams, slot, ((ox, oy), angle, frames));
        for ((judged, nearest), path) in judged.iter_mut().zip(&mut nearest).zip(&lands) {
            for (step, (x, y)) in path.iter().enumerate() {
                // Where the beam will be pointing by then, which is where it points now plus as many of its own
                // turns as there are frames to look — and where it will be fired from, which is the same
                // arithmetic on the origin.
                let ahead = angle + turn * (step + 1) as f32;
                let (cos, sin) = (ahead.cos(), ahead.sin());
                let (ox, oy) = (
                    ox + went.0 * (step + 1) as f32,
                    oy + went.1 * (step + 1) as f32,
                );
                let (dx, dy) = (x - ox, y - oy);
                let along = dx * cos + dy * sin;
                let off = -dx * sin + dy * cos;
                // **And the whole band the sweep is being laid across, not only the lane it stands in.**
                // Where a lane was laid the frame before this one, the two are one step of a sweep and the
                // step is how far apart they stand here: the lane after this one is that far on again, and
                // the one after that twice. So every across from this lane to where the sweep has reached
                // by then is a place to be out of — the room is the nearer of the two ends, and nought
                // where the player is between them.
                //
                // **Measured where it stands and not projected**, the lane before being a laid one: what
                // moves in this pattern is the laying and not what is laid, which every dump of it says —
                // `nearest was a laser ... stepping 0.0,0.0`, nine of them and not one carried.
                //
                // **And left alone where the two are too far apart to be one sweep**, which is
                // [`A_SWEEPS_OWN_SPACING`]: a lane a screen away that happens to be a frame older is not
                // the frame before of this one, and taking it for that is a band across the whole field.
                let swept = match before {
                    Some(((bx, by), (before_sin, before_cos))) => {
                        let behind = -(x - bx) * before_sin + (y - by) * before_cos;
                        let apart = behind - off;
                        match apart.abs() <= across * A_SWEEPS_OWN_SPACING {
                            true => off - apart * (step + 1) as f32,
                            false => off,
                        }
                    }
                    None => off,
                };
                let off = match off * swept > 0.0 {
                    true => off.abs().min(swept.abs()),
                    false => 0.0,
                };
                // **And how far it will reach by then**, which is the other thing a beam does with a frame:
                // th06's own extend as they fire, and a length taken as settled is a beam whose end the
                // player is standing just past for every frame looked at. A run stood five units under one and
                // died there nineteen times.
                let far = far + grows * (step + 1) as f32;
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
                        step: went,
                        half: (across, across),
                        // A beam's width is the game's on every frame looked at: what grows on one is its far
                        // end, which is in `far` above and not in the box.
                        feared: None,
                    });
                }
                // And the beam's own room on that frame, for the same reason a bullet's is kept — save
                // where it is a beam that has not fired, which reaches nothing whatever it is drawn
                // across.
                if step == 0 && fires_in <= 1 {
                    judged.next_frame = judged.next_frame.min(room);
                }
                if room < ROOM_TO_SPARE {
                    // **Live by then or still only drawn.** The game's own gate is the countdown running out, so
                    // a frame this many ahead is one the beam can hit on only once it has — before that the lane
                    // is a place to be out of rather than a frame lost.
                    //
                    // **And a turning one is not even that while the rules dodge by stepping behind it**,
                    // which is the whole of what that takes: the place is on the far side of the lane the
                    // laser is drawn in, and a warning on it shut every move that crosses to it — leaving the
                    // moves that run ahead of the laser, away from where it is drawn, as the ones clear. See
                    // [`BEHIND_THE_LASER_BY`].
                    if (step + 1) as i32 >= fires_in {
                        judged.hit(step);
                    } else if !(behind_a_turning_laser && counted_down && turn != 0.0) {
                        judged.warn(step);
                    }
                }
            }
        }
    }
    // **And the row this frame read against is left exactly as it was**, whichever call this is: what a
    // hazard *moved* is where it is now against where that row says it was, so a call that replaces the row
    // leaves every call after it inside the same frame reading every hazard as having moved nothing.
    //
    // **Which is what the two focus passes were deciding on.** A bullet that claims a step and moved nought
    // is one [`A_STEP_OF_ITS_OWN`] calls script-moved, and such a hazard is feared over what it just
    // moved — nothing at all. So the pass after the frame's own saw a field of points: a bullet `24` above
    // the player closing `4` a frame, feared over `(12,24)` by the frame's own call and judged a point by
    // the one after it, which answered `20 of room` where the first had `-160` and pressed the focus key on
    // the strength of it. `the_bot_shoots_a_boss_slowed_down` is the frame that says so.
    //
    // **So what the frame's own call read waits in [`READ_THIS_FRAME`]** until the word is decided, and
    // [`remembers_what_the_frame_read`] moves it across — see [`THE_FRAMES_OWN`].
    *kept = Some(last);
    drop(kept);
    if remembers {
        *READ_THIS_FRAME
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(now);
    }
    // **The nearest to where it wants to be, of the moves nothing will be at.** Which is not the same as the
    // one furthest from everything: the next frame's positions are known, so what a move has to be is *not
    // where a bullet lands* — and keeping away from bullets besides is a bot that spends the stage leaving,
    // since the screen of a th07 stage is never empty and being near one is the ordinary state of playing.
    // **Where to be, and only now that the bullets have been walked**: which box of the field a script is
    // sweeping a beam across is one of the things that decides it, and that is a thing the walk finds.
    let sweeping = a_sweep(swept, swept_x);
    counted.swept = swept;
    counted.sweeping = sweeping;
    counted.fire = the_fire_in_each_box(frame);
    counted.overhead = the_boss_is_overhead(frame);
    counted.closes = closes;
    // **Or the nearest thing that can hit the player, where the chapter is one to be hit in and has not been.**
    // The nearest to the move of standing still is that thing: it is the one whose box came closest to where the
    // player is over the frames looked at, past every rule that leaves a hazard out — so a body with nothing
    // solid on it is not something to walk at, and neither is a beam still drawing its warning.
    //
    // **And the place it was read at, which no clamp of [`where_to_be`]'s is applied to.** The fire a chapter
    // is to be hit by is wherever it is, and the band and the side edges are where the rules *wait* — a walk
    // held inside them is a walk that stops short of what it is for.
    let (wanted, settled) = match (hazards, nearest[0]) {
        (Hazards::GoneTo, Some(near)) => (near.at, Settled::SomethingThatCanHitIt),
        _ => where_to_be(frame, sweeping, goes_live),
    };
    let mut best: Option<(usize, Judged, f32, f32)> = None;
    let crossing = settled == Settled::TheWayItWasToldToDodge
        && match the_dodge() {
            Some(Dodge::Laevateinn) => (wanted.0 - at.0).abs() > the_straight.0,
            Some(Dodge::AndThenThereWereNone) => true,
            _ => false,
        };
    for ((judged, land), which) in judged.into_iter().zip(lands).zip(0..) {
        let judged = match crossing {
            true => judged.within(A_CROSSINGS_LOOK_AHEAD),
            false => judged,
        };
        // How near this frame's own step gets to where it wants to be, which is the first of the path: the
        // frames after it are what the move is judged *hit* by and not where it is going.
        //
        // **And the nearest that holding the move gets to where it wants to be was written, measured and
        // taken out.** Which is the one thing the frames of th06c's Extra point at: every frame of that
        // climb is a step that was locally the thing to do — the run goes up because up is what is open, one
        // step at a time, with the step itself never far from where the rules want to be — so a move judged
        // by its landing alone cannot tell a way down from a way out of the band. Two Extra attempts of it
        // took `10` hits outside [`the_band`] or at the clamp of the eighteen they took, against `12` of
        // twenty-six for the attempts judged by the landing, and the hub cluster is in both of them. So
        // where a move *leads* is not what the place a run loses turns on either.
        //
        // **Not the end of the path**, which is a different rule and a wrong one: held [`LOOK_AHEAD`] frames
        // the end is `160` units past a place `20` away, so a move that arrives reads as one that overshot,
        // and seventeen frames of this suite said so at once.
        //
        // **And stopped where the field's own edge stops it**, which is what the nearest along the whole
        // path gets wrong and what `the_bot_dodges_an_enemy_on_its_way_to_an_item` is the frame for. That
        // frame stands the player at `192,384` with an item at `100,384` and the field ending at `448`: up
        // and down are a tie, both nearest at the sixteenth step with `4234`, until the downward path is
        // clamped at the floor on the twenty-third and runs *left along it* — reaching the item's own column
        // `64` below it, `4096`, which is nearer than the tie and is a place the item cannot be taken from.
        // The clamp is right for judging what a move is hit by, a move into a wall being a move that does
        // not happen; carried into where it leads it turns holding a direction into being pinned against
        // that wall, so the path is read only as far as it is still going somewhere.
        //
        // **And it had to be read only as far as the field's own edge lets the move go**, which is where
        // the first form of it went wrong and is worth keeping: the nearest over the *whole* path cost
        // `the_bot_dodges_an_enemy_on_its_way_to_an_item`, whose player stands at `192,384` with an item at
        // `100,384` and the field ending at `448`. Up and down are a tie there, both nearest at the
        // sixteenth step with `4234`, until the downward path is clamped at the floor on the twenty-third
        // and runs *left along it* — reaching the item's own column `64` below it, `4096`, which is nearer
        // than the tie and is a place the item cannot be taken from. The clamp is right for judging what a
        // move is hit by, a move into a wall being a move that does not happen; carried into where a move
        // leads it turns holding a direction into being pinned against that wall. Stopped at the frame the
        // stored landing first differs from the step's own arithmetic, the tie holds and the suite passes —
        // which is how the measure above was run at all.
        //
        // **And the end of the path is a third form and a wrong one**: held [`LOOK_AHEAD`] frames it is
        // `160` units past a place `20` away, so a move that arrives reads as one that overshot, and
        // seventeen frames of this suite said so at once.
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
        // order was hit nine times before stage 3's boss where the one before it reached stage 4 without a scratch.
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
            // **With the hazards ignored, the nearest to where it wants to be and nothing else.** Which is the
            // whole of what being hit on purpose is: not a bullet picked out to be run into, but the frames
            // played as though none of them were there — and where that ends up is where a player who stopped
            // looking ends up.
            Some((.., was_towards)) if hazards != Hazards::Dodged => towards < was_towards,
            // **And the band was ranked in here three ways over and reaches none of it.** A move that stays
            // inside [`the_band`] over one that leaves it: under the depth of the next frame's overlap, then
            // under the frames a move buys as well, then over whether either is open at all. One Extra
            // attempt apiece, counting the hits dumped above the band's own top against the five and three
            // the ordering before them gave: `4`, `4` and `6` of six and seven. The last of those is the
            // whole trade — a hit taken at the boundary rather than a climb — and it left the run up there
            // more rather than less.
            //
            // **Which is the same answer the floor got**, and for the same reason: the climbs are made on
            // frames where nothing in the band is open to prefer. Three hits of one attempt read `1, 0, 0,
            // 1, 1, 0, 0, 0` of the nine clear over the last eight sampled frames of the climb they ended,
            // `4, 0, 0, 0, 0, 0, 0, 0` and `3, 1, 0, 0, 1, 0, 0, 0` — so what is left is a place decided
            // dozens of frames earlier, and `where_to_be` is where that lives.
            Some((_, was, ..)) if tier(judged) != tier(was) => tier(judged) < tier(was),
            Some((_, was, was_x, was_towards)) => match (was.clear(), judged.clear()) {
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
                //
                // **And holding it under the wanting unless the lane the player is already in is one of them
                // was tried and reaches nothing.** The thought was that a card fanning its warnings over the
                // field has the run leaving lanes it was never going to be in, which above the wanting reads as
                // a walk to wherever the fewest of them are — and the climb three attempts of th06c's Extra
                // made goes *left* from `165` to `135` on its first three frames, every one of them with `9 of
                // the nine clear`, while what it wants is `192,368`. Two attempts with the gate in ended above
                // the band in the same three places as the three without it: `167.5,192.0`, `125.5,270.6` and
                // `215.1,196.4` against `180.0,202.1`, `126.1,270.4` and `180.5,203.2`. So the climb is not
                // this term.
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
                // **And a move whose own next frame does not overlap, over one that does, before any of that.**
                // With nothing open the next frame is the only one the player is certain to reach: a move
                // already inside something on it is a hit taken now, where one clear of everything on it buys
                // another frame of choosing, and no frame further out can be worth more than that. Which
                // `first()` cannot say for it — every move is `hit first on 0` the moment the slack is
                // crossed, and the slack is crossed on every frame of a screen like this one.
                //
                // **A run died of that order**, on th06c's Extra boss with 207 bullets and eight beams up:
                // `0x0a0` was chosen `-3.2` into something on its own next frame because it was hit on four
                // of the thirteen, over `0x090` with `1.8` of room there and hit on all thirteen — and a
                // frame twelve out is one twelve more frames of choosing can be spent getting out of the way
                // of.
                (false, false) => {
                    // **And a move the field's own edge turns into standing still is not one of them.** The
                    // reason a sideways move is preferred here is that the fire's thickness is a thing of
                    // *x* and a move that does not change the x cannot get out of it — which a move into a
                    // wall does not, the clamp on [`lands`] holding it where it is. One hit of an Extra
                    // attempt is the whole of it, at `8.0,130.9` against a field that starts at `8`:
                    // `0x000` and `0x040` read `hit first on 6, on 7 frame(s), -0.8 of room and 3.3 on its
                    // own next frame`, the same numbers to the last digit, and the word that came out was
                    // `0x040` — the run pressing into the wall because pressing into it was ranked as
                    // getting out from under something. Which is how a run comes to spend thirty frames
                    // pinned at `x = 8`.
                    let sideways = land[0].0 != at.0;
                    let was_sideways = was_x != at.0;
                    let lives = judged.next_frame >= 0.0;
                    let was_lives = was.next_frame >= 0.0;
                    match (lives, was_lives) {
                        (true, false) => true,
                        (false, true) => false,
                        // **And where neither of them reaches it, the shallower of the two overlaps.** The
                        // frames after the next one are frames that never arrive, so what is left to choose
                        // by is how far into something the next frame puts the player — and the least of
                        // that is the move nearest to being no hit at all, [`ROOM_TO_SPARE`] being two units
                        // of slack the projection is kept with rather than a hit. A run stood still into a
                        // beam `-6.0` deep over a step aside `-2.0` deep, because standing still was hit on
                        // two of the thirteen frames where the step was hit on six.
                        //
                        // **And where two of them are as shallow as each other, the one nearer to where
                        // the rules want to be** — which is the one way the wanting reaches a screen the
                        // nine are never all clear on, and the place a run loses is lost on exactly those
                        // screens. What decided the corner one attempt was hit in was this comparison and
                        // then nothing: the nine read `-0.9, -2.9, -0.9, -0.9, -0.9, -2.9, -2.8, -0.9,
                        // -0.9` on their own next frames, six of them tied at `-0.9`, and the tie went to
                        // [`WAYS`]'s own order, which is standing still. The boxes on that frame read
                        // `173.2, 348.4, 257.2, 19.2, 11.2` with the player in the first of them, so the
                        // count that decides where to be had every number it needed and no way to be read:
                        // `towards` is looked at where both moves are clear and where the hazards are
                        // ignored, and was looked at nowhere else.
                        //
                        // **A tie-break and not a re-ordering**, which is what keeps every measured order
                        // above it intact: the depth of the overlap still decides wherever the two differ.
                        // And it cannot oscillate the way the comment above warns a tie-break can — what it
                        // aims at is one place per frame rather than the other move's own number, so two
                        // moves cannot each be the answer to the other.
                        //
                        // **What one Extra attempt apiece says**: four hits against five, the first of them
                        // at `f27480` against `f16860`, and ten frames with nothing open against twelve.
                        // What it does not do is put the hits back inside the band — all four are still
                        // outside it, one at the clamp and three above — so the place a dense screen loses
                        // is not this tie either.
                        //
                        // **And one attempt apiece is inside the noise**, which a later pair of them
                        // measured: an attempt whose only change was a rule that then never fired was hit
                        // six times against that four, so the spread on one build is at least `4` to `7`.
                        // What holds this in is the reasoning above and not that number. `steer`'s own comment records `towards` tried under and
                        // over `hits()` against a bot that predates the two focus passes, the
                        // deathbomb-only bomb, the swept band and the read that stopped a second pass
                        // seeing a world that had not moved, and found nothing then; this is the same
                        // question asked where the corner frame was actually decided.
                        (false, false) => match judged.next_frame == was.next_frame {
                            true => towards < was_towards,
                            false => judged.next_frame > was.next_frame,
                        },
                        // **And under the first hit, where the rules want to be — which is this branch
                        // being the ordinary one now and not the corner.** A move is hit where its room
                        // falls under [`ROOM_TO_SPARE`] on any frame looked at, and at
                        // [`LOOK_AHEAD`]'s forty frames something always arrives on a screen of this
                        // game's Extra boss: the hit at `8.0,421.7` has standing still `hit first on 16`
                        // with `1.2 of room`, a frame the thirteen-frame walk called open. So the branch
                        // every one of the nine being hit leads to is where a run now spends its frames,
                        // and the wanting was read in it only where two moves were equally shallow on
                        // their own next frame.
                        //
                        // **And the wanting ranked into this chain rather than left as the tie above was
                        // written and taken out again.** The reason to try it is that this branch is the
                        // ordinary one at [`LOOK_AHEAD`]'s forty frames — a move is hit where its room
                        // falls under [`ROOM_TO_SPARE`] on any frame looked at, and something always
                        // arrives on a screen of this game's Extra boss, so the hit at `8.0,421.7` has
                        // standing still `hit first on 16` with `1.2 of room`, a frame the thirteen-frame
                        // walk called open.
                        //
                        // **Over `hits()` it costs a case of this suite outright**:
                        // `a_warning_line_is_left_whatever_frame_its_beam_fires_on` has a beam two frames
                        // from firing, where the step aside and standing still are both hit from the
                        // second frame and what carries the step clear by the third is the count — ranked
                        // under the wanting, the player stayed in the lane. **And under `hits()` it
                        // measures worse**: two Extra attempts took `13` hits outside [`the_band`] or at
                        // the clamp of the eighteen they took, against `12` of twenty-six for the three
                        // attempts without it, and the hits it left inside were the two at the waiting
                        // line each attempt already had. So the wanting stays where it was measured to
                        // belong, which is the tie on the next frame's own depth.
                        // **And the count over the first hit was tried at this walk's own length and is
                        // refused by the frame that settled the order at the old one.** That order was
                        // measured when the walk was twelve frames, where *hit at all* and *hit next
                        // frame* are nearly the same thing, and at forty they are further apart than
                        // ever — `hit first on 32` against `hit first on 2` is thirty frames of choosing
                        // — so the question is worth asking again and
                        // `the_move_taken_is_the_one_whose_trouble_is_furthest_off` answers it: swapping
                        // the two costs that case outright.
                        (true, true) => {
                            judged.first() > was.first()
                                || (judged.first() == was.first()
                                    && (judged.hits() < was.hits()
                                        || (judged.hits() == was.hits()
                                            && ((sideways && !was_sideways)
                                                || (sideways == was_sideways
                                                    && (judged.room > was.room
                                                        // And the same tie-break at the bottom of this
                                                        // chain, for the same reason.
                                                        || (judged.room == was.room
                                                            && towards < was_towards)))))))
                        }
                    }
                }
            },
        };
        if better {
            best = Some((which, judged, land[0].0, towards));
        }
    }
    // **What the focused step under a boss is off for is anything that reaches the move chosen** inside the frames
    // looked at, and not only what would hit it next frame: halving the step doubles the frames it takes to be
    // out from under something, so by the frame a bullet is a hit the halving has already been paid for.
    //
    // **And not a distance beside it**, which is what this was: a distance is between two centres where a hit is
    // two boxes overlapping, so a bullet 12 away is inside a large bullet's own half and outside a small one's —
    // the same number meaning a hit and a clear frame. [`LOOK_AHEAD`] frames of the game's own test already say
    // whether anything reaches the player, and one more margin on top of it says nothing the test does not.
    // **Whether the player has to move at all**, which is [`WAYS`]'s own first entry and not the move chosen:
    // the move chosen is clear on almost every frame, that being what choosing it means, so a slowing taken off
    // by it is a slowing never taken off. Where holding still keeps the player out of everything for the frames
    // looked at, the step is not what the frame is for and the damage is.
    let (chosen, judged_it, ..) = best.unwrap_or((0, Judged::CLEAR, 0.0, 0.0));
    Steered {
        word: keys.way(WAYS[chosen]),
        dodging: !judged[0].clear(),
        room: judged_it.clear().then_some(judged_it.room),
        for_an_item: the_item_gone_for(frame).is_some(),
        // **Nothing open at all *and* the best of the nine short of the slack on its own next frame**,
        // which is the difference between a corner and thirteen frames of pessimism — see
        // [`Steered::hit_at_once`].
        hit_at_once: !judged_it.clear() && judged_it.next_frame < ROOM_A_BOMB_IS_OWED_AT,
        next_frame: judged_it.next_frame,
        at,
        half: frame.half,
        step: (the_straight, the_diagonal),
        wanted,
        settled,
        nearest: nearest[chosen],
        counted,
        held,
        held_all,
        beams: kept_beams,
        judgements: judged,
    }
}

/// **The nearest place outside a beam the run is standing inside**, or nothing where it is in none.
///
/// **Which the nine moves cannot answer.** A move is judged by whether it is hit over
/// [`THE_FRAMES_AHEAD`], and a beam 160 across is 80 units from its middle to its edge — twenty frames of
/// a whole step. So every one of the nine is hit, the judgement has nothing to choose between them, and a
/// run four units inside one stands there: `nowhere open to it` with `the nearest beam slot 2, -4.2
/// outside its box`, which is th08's stage 4 boss taking a life through a beam the model was holding.
///
/// The way out is across the beam and not along it, so the place wanted is the run's own, moved
/// perpendicular to the beam far enough to clear its edge and [`A_STEP_OUT_OF_A_BEAM`] more. The side is
/// whichever is nearer, and the other where the nearer one is off the field — a run pushed against an edge
/// has the long way out and no other.
///
/// **The beam still only drawn is left alone**, its own frames being what a warning is for: a lane that
/// has not fired is ground to be off before it does, which the moves can see.
///
/// **Save the one nothing says the frame of** — [`FIRES_WHEN_NOTHING_SAYS`] — which the moves cannot get
/// off on their own: they are judged over [`THE_FRAMES_AHEAD`], a lane 160 across takes twenty frames of
/// stepping to leave, and a beam that may fire on any of them warns every one of the nine alike. So the
/// nine have nothing to choose between and the place to wait is the whole of the answer: th08 arms four
/// enemies at once in stage 4's fight, and a run that read their lanes as ground stood in one until it
/// fired.
fn the_way_out_of_a_beam(frame: &Frame) -> Option<(f32, f32)> {
    let field = frame.field;
    let (left, right) = (field.0, field.0 + field.2);
    let (top, bottom) = (field.1, field.1 + field.3);
    let at = frame.at;
    // The deepest one it is inside, that being the one it has least time to get out of — and a lane
    // already firing before any depth of one only armed, a beam testing the player now being the one that
    // takes the life if neither is left.
    let mut deepest = None::<(bool, f32, (f32, f32))>;
    for beam in &frame.beams {
        let firing = beam.fires_in <= 0;
        if !firing && beam.fires_in != FIRES_WHEN_NOTHING_SAYS {
            continue;
        }
        let (sin, cos) = beam.angle.sin_cos();
        let (dx, dy) = (at.0 - beam.origin.0, at.1 - beam.origin.1);
        let along = dx * cos + dy * sin;
        if along < beam.near || along > beam.far {
            continue;
        }
        let off = -dx * sin + dy * cos;
        let half_across = the_half_across(beam);
        let inside = half_across - off.abs();
        if inside <= 0.0 {
            continue;
        }
        // Across the beam, the way it is already leaning, or the other way — and the two walks are not the
        // same length where the run is off the middle of the lane. **Which is a place inside the beam
        // where the far walk was the near one's distance the other way**: a run 34 off the middle of a
        // 160-wide lane is 46 from one edge and 114 from the other, and told the 50 of the nearer one
        // leftwards it stops 30 short of leaving — `nowhere open to it` at `91.4,432.0`, the same frame of
        // the same chapter 95 times over.
        let away = match off < 0.0 {
            true => -1.0,
            false => 1.0,
        };
        let (nx, ny) = (-sin, cos);
        let out = inside + A_STEP_OUT_OF_A_BEAM;
        let across = half_across + off.abs() + A_STEP_OUT_OF_A_BEAM;
        let nearer = (at.0 + nx * away * out, at.1 + ny * away * out);
        let other = (at.0 - nx * away * across, at.1 - ny * away * across);
        let fits = |(x, y): (f32, f32)| x >= left && x <= right && y >= top && y <= bottom;
        // **And where both sides fit, the one towards the middle of the field.** The near side is the
        // shorter walk and the corner is where it leads: a run 8 units inside a beam at the field's own
        // right edge was put *to* that edge, and died there with 147 bullets and `nowhere open to it` —
        // which is what [`IN_FROM_THE_SIDE`] keeps the rules off for every other errand. The longer walk
        // across a beam is the one with somewhere to go when it is over.
        let middle = field.0 + field.2 / 2.0;
        let towards_the_middle = match (fits(nearer), fits(other)) {
            (true, false) => nearer,
            (false, true) => other,
            _ => match (nearer.0 - middle).abs() <= (other.0 - middle).abs() {
                true => nearer,
                false => other,
            },
        };
        let to = towards_the_middle;
        if deepest.is_none_or(|(was_firing, was, _)| (firing, inside) > (was_firing, was)) {
            deepest = Some((firing, inside, to));
        }
    }
    deepest.map(|(.., to)| (to.0.clamp(left, right), to.1.clamp(top, bottom)))
}

/// **The next corner round the field away from a following laser**, and the nearer upper corner where none is
/// firing — what [`Dodge::OutOfTheFollowingLaser`] walks to once it is out of the beam.
///
/// **Round and not across**, because the laser turns after the run about its own origin and never catches it
/// while the run has angle left to take: `0.0036` rad a frame is about a unit a frame at the low line's
/// distance from the boss, against the `4.0` of a whole step. What runs out is the field. Out of the beam
/// the short way and then along the floor, one run of such a card was put against the left wall and up it
/// on one wave and into the bottom-left corner on the next, with `0 of the nine clear` for the forty frames
/// before it was hit there. So the corners are waypoints — the two at the low line and the two
/// [`UP_THE_SIDE_ABOVE_THE_FLOOR`] up, each [`IN_FROM_THE_SIDE`] in — taken in the order their angle from
/// the laser's origin goes, the way the run is already going from the laser: up a side, down it, along the
/// floor and up the other, which is the strategy wiki's own `左上、左下、右上、左下` for leading it.
///
/// **The laser nearest the run's own angle** where several are firing, that being the one closing on it.
///
/// **And an upper corner to be aimed at while none is firing**, the nearer of the two: the laser is aimed
/// where the run stands when it appears, so where the run waits between waves is where the next one starts
/// from, and from an upper corner the whole of the walk round is ahead of it. Out of the beam the short way
/// and round from wherever the wave found it, one run of such a card had half the walk at most, and the
/// laser outlasted it: it went on past the corner `200` up the right side and was hit at `376.0,16.0`, the
/// field's own top-right corner, still wanting `328,200`.
fn ahead_of_a_following_laser(frame: &Frame) -> Option<(f32, f32)> {
    let field = frame.field;
    let at = frame.at;
    let (inside, outside) = (
        field.0 + IN_FROM_THE_SIDE,
        field.0 + field.2 - IN_FROM_THE_SIDE,
    );
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    let up = field.1 + field.3 - UP_THE_SIDE_ABOVE_THE_FLOOR;
    let around = |(x, y): (f32, f32), origin: (f32, f32)| (y - origin.1).atan2(x - origin.0);
    let apart = |a: f32, b: f32| {
        let d = (a - b).rem_euclid(std::f32::consts::TAU);
        if d > std::f32::consts::PI {
            d - std::f32::consts::TAU
        } else {
            d
        }
    };
    let firing = frame
        .beams
        .iter()
        .filter(|beam| beam.fires_in <= 0)
        .min_by(|a, b| {
            let off = |beam: &Beam| apart(around(at, beam.origin), beam.angle).abs();
            off(a).total_cmp(&off(b))
        });
    // **Beam or none**: between two waves the enemies that lay them are gone and nothing is aimed, and a run
    // that went back to its own errand then — `a boss at 79` to `115` on one Final B clear — was where the next
    // laser was aimed, `226,382`, and was pushed along the floor into the bottom-left corner and hit there.
    let Some(beam) = firing else {
        let nearer = match (at.0 - inside).abs() <= (at.0 - outside).abs() {
            true => inside,
            false => outside,
        };
        return Some((nearer, up));
    };
    let from = around(at, beam.origin);
    let way = apart(from, beam.angle).signum();
    if way == 0.0 {
        return None;
    }
    let corners = [(inside, up), (inside, low), (outside, low), (outside, up)];
    let ahead = |corner: (f32, f32)| apart(around(corner, beam.origin), from) * way;
    corners
        .into_iter()
        .filter(|corner| ahead(*corner) > A_CORNER_PASSED_AT)
        .min_by(|a, b| ahead(*a).total_cmp(&ahead(*b)))
        .or_else(|| {
            corners
                .into_iter()
                .max_by(|a, b| ahead(*a).total_cmp(&ahead(*b)))
        })
}

/// **The next corner clockwise round the field from where the run is** — what [`Dodge::RoundTheField`] walks
/// to, the same four corners [`ahead_of_a_following_laser`] takes, measured about their own middle.
///
/// **Because a boss that follows the run is under it wherever it waits.** One card of the kind takes the boss
/// off the field as a spirit that tracks the run and fires from where it is: the rules' errand was `a boss at
/// 101,368` from `103,366`, the boss's own x being the run's, so they stood still where her fire comes from and
/// were hit there with `122.0 of room` on the moves — nine hits on one Extra's attempts. The strategy wiki's
/// way through each of its phases is round the whole field (`画面全体を大回転`), and that is a place that moves
/// on.
fn round_the_field(frame: &Frame) -> (f32, f32) {
    let field = frame.field;
    let at = frame.at;
    let (inside, outside) = (
        field.0 + IN_FROM_THE_SIDE,
        field.0 + field.2 - IN_FROM_THE_SIDE,
    );
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    let up = field.1 + field.3 - UP_THE_SIDE_ABOVE_THE_FLOOR;
    the_next_corner_clockwise(at, (inside, up), (outside, low))
}

/// **The next corner clockwise round a box from where the run is**, measured about the box's own middle — the
/// walk [`round_the_field`] and [`round_a_small_square`] both take, from the top-left corner to the
/// bottom-right.
fn the_next_corner_clockwise(
    at: (f32, f32),
    (left, top): (f32, f32),
    (right, bottom): (f32, f32),
) -> (f32, f32) {
    let middle = ((left + right) / 2.0, (top + bottom) / 2.0);
    let around = |(x, y): (f32, f32)| (y - middle.1).atan2(x - middle.0);
    let from = around(at);
    let ahead = |corner: (f32, f32)| {
        let d = (around(corner) - from).rem_euclid(std::f32::consts::TAU);
        let reached = (corner.0 - at.0).hypot(corner.1 - at.1) <= A_CORNER_REACHED_WITHIN;
        if d <= A_CORNER_PASSED_AT || reached {
            d + std::f32::consts::TAU
        } else {
            d
        }
    };
    [(left, top), (right, top), (right, bottom), (left, bottom)]
        .into_iter()
        .min_by(|a, b| ahead(*a).total_cmp(&ahead(*b)))
        .unwrap_or(middle)
}

/// **The next corner of a small square under the boss, clockwise** — what [`Dodge::RoundASmallSquare`] walks to,
/// its bottom edge on the low line under the boss's own x, or the field's middle where there is no boss.
///
/// **Because the familiars of that kind of card stay on the run and fire from where they are.** One Extra's
/// `「蓬莱人形」` hit a run three times with `inf of room` on all nine moves, and what stood over each hit, read
/// out of the trace, was the same: eight familiars stacked seven to fourteen units off the run, and a ring of
/// eight bullets `24` across about them, every one in kind 5 by the frame the hit was seen — a kind nothing is
/// tested in, so what hit the run is gone or turned by then, and those rings, laid within reach of it, are what
/// was there. The focus key was down on all three, the run moving `2` a frame. The strategy wiki's way through
/// that card is `妹紅の前で小さく時計回りにちょん避け(四角く→↓←↑)` — never still, so that where the familiars fire
/// from is where the run was.
///
/// **And moved aside on a retry, the way every other way's place is** — see [`aside_on`]. A square that does not
/// move is an attempt played the way the one before it was: that Extra's four attempts were each hit at the same
/// place, `2,916` or `2,917` frames in.
fn round_a_small_square(frame: &Frame) -> (f32, f32) {
    let field = frame.field;
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    let aside = aside_on(ATTEMPT.load(Ordering::Relaxed));
    let x = (frame.boss.map_or(field.0 + field.2 / 2.0, |(x, _)| x) + aside).clamp(
        field.0 + IN_FROM_THE_SIDE + A_SMALL_SQUARES_HALF,
        field.0 + field.2 - IN_FROM_THE_SIDE - A_SMALL_SQUARES_HALF,
    );
    the_next_corner_clockwise(
        frame.at,
        (x - A_SMALL_SQUARES_HALF, low - 2.0 * A_SMALL_SQUARES_HALF),
        (x + A_SMALL_SQUARES_HALF, low),
    )
}

/// **Half a side of the square [`round_a_small_square`] walks**: six of the run's whole steps across, which is
/// `48` — the bullets its card fires from on the run being `24` across, so a side is two of them.
const A_SMALL_SQUARES_HALF: f32 = 24.0;

/// **The end of the low line the run is going to** — what [`Dodge::SideToSide`] walks to: from as far in as a
/// way's place is held on one side to as far in on the other, and back once there.
///
/// **Because a laser aimed at the run is aimed at where it stands.** th08's `難題「龍の頸の玉　-五色の弾丸-」` fires
/// lasers at the run with fire strewn between them, and the rules dodging each one stepped the same way as the
/// last until there was no more field that way: nine Kaguya clears in nine were hit on it, eight of them
/// within `47` of a wall, five 355 frames into the card at about `45,369` with a laser `6` off standing still
/// that the moves read `2.2` to `3.6` of room from — and the ninth at `8.0,391.8` wanting the middle, the box it
/// was in no longer an edge it had chosen. The strategy wiki has it `自機狙いレーザー+放物線軌道の弾のばら撒き`
/// and `まともに避けようとすると詰む事もある`, and its way through is from side to side between the lasers: a run
/// that is always going somewhere is not where the next one is aimed, and one that turns before the wall has
/// the field the other way to go on into.
///
/// **Which way the run is going kept, and not read off where it is.** Walked as a loop 24 high round the low line
/// by [`the_next_corner_clockwise`], the way on was the angle about the loop's middle — and a run the dodging had
/// taken outside the loop past a corner was still short of it by that angle: Kaguya's clear wanted `56,368` from
/// `x = 8` to `64` at the left wall for the card's last 1,100 frames, and stood there.
fn side_to_side(frame: &Frame) -> (f32, f32) {
    let field = frame.field;
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    let (inside, outside) = (
        field.0 + IN_FROM_THE_SIDE,
        field.0 + field.2 - IN_FROM_THE_SIDE,
    );
    let at = frame.at.0;
    let now = the_cards_frames().unwrap_or(0);
    let mut going = SIDE_TO_SIDE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (going_left, from, since) = *going;
    let (from, since) = match (at - from).abs() < A_STEP_TAKEN && now >= since {
        true => (from, since),
        false => (at, now),
    };
    let stood = now - since >= SIDE_TO_SIDE_TURNS_AFTER;
    let turned = match going_left {
        true => at > inside + A_CORNER_REACHED_WITHIN,
        false => at >= outside - A_CORNER_REACHED_WITHIN,
    } != stood;
    *going = match stood {
        true => (turned, at, now),
        false => (turned, from, since),
    };
    match turned {
        true => (inside, low),
        false => (outside, low),
    }
}

/// **Which end of the low line [`side_to_side`] is going to** — the right one to start with — **and where the run
/// has stood since which of the card's frames.** By the card's frames and not by the calls, the rules being asked
/// where to be more than once a frame.
///
/// **And turned back where it has stood for [`SIDE_TO_SIDE_TURNS_AFTER`]**, which is the way on being shut rather
/// than the run being there: the two hits a Kaguya clear took on the card with the ends alone to turn at were
/// each at a place it had stood `91` and `109` frames at, wanting the far end with no step towards it open and
/// pressing the shot alone — still in front of lasers aimed at it, which is what the way is for never being.
static SIDE_TO_SIDE: Mutex<(bool, f32, i32)> = Mutex::new((false, 0.0, 0));

/// **How many of its frames [`side_to_side`] stands still before it turns back**: a third of a second, well inside
/// the ninety-one the first of those hits stood.
const SIDE_TO_SIDE_TURNS_AFTER: i32 = 20;

/// **Less than this across between two calls is standing still** — half of th08's focused step, so that a focused
/// step taken towards the end is going somewhere.
const A_STEP_TAKEN: f32 = 1.0;

/// **How far up the side [`ahead_of_a_following_laser`] takes the run** — `200` on a field whose floor is
/// `432`, about half way up it.
///
/// Up because that is angle the laser has to turn through: from a boss at `161,103`, the corner at the low line
/// is `0.56` rad round from straight down and this one `1.05`, `134` frames more of the laser's turn. And not
/// higher, where the run is beside the boss's own fire as it leaves her.
const UP_THE_SIDE_ABOVE_THE_FLOOR: f32 = 232.0;

/// **How near in angle a corner has to come before the run is past it**: a twentieth of a radian, which at the
/// low line's distance from the boss is a dozen units — three steps.
const A_CORNER_PASSED_AT: f32 = 0.05;

/// **And how near in distance**, which is a whole unfocused step of th08's: a corner no step brings the run any
/// nearer to is one it has reached, whatever the angle says. **Not the angle alone**, which is a twentieth of a
/// radian about the box's own middle and so shrinks with the box: [`round_a_small_square`]'s corners are `34`
/// from theirs, where it is `1.7` — less than any step. One Extra's `「蓬莱人形」` stood at `70.8,321.4` wanting
/// the corner at `72,320` for the last `63` frames of each of four attempts running, every step open to it
/// landing further from the corner than standing did, and was hit there each time.
const A_CORNER_REACHED_WITHIN: f32 = 4.0;

/// How far past a beam's own edge the way out aims, so that a run leaving one does not stop on the line.
/// A whole unfocused step of th08's, which is the widest of the games' — a place a step short of clear is
/// a place the next frame's judgement puts back inside.
const A_STEP_OUT_OF_A_BEAM: f32 = 4.0;

/// **A beam a script is sweeping**, as the x it is spread about — or nothing where the bullets no step explains
/// are too few to be one.
///
/// A turning beam in this game is a line of bullets, and a script rather than any field of theirs is what swings
/// it: stage 6's is a dozen and more of them at once. So what tells one from a bullet whose own curve the reading
/// happens to miss is how many there are, and the floor is the one the thickest box already uses — more than
/// the nine moves, which is more than a rounding difference in either.
///
/// **The count is what a launch says.** A hit's own line carries it, so a card whose every bullet reads as
/// scripted is a card this names and not one it hides.
fn a_sweep(swept: u32, swept_x: f32) -> Option<f32> {
    (swept as usize > WAYS.len()).then(|| swept_x / swept as f32)
}

// **And the x just behind a row of lanes being laid was answered here and wanted by [`where_to_be`], and
// what it reached is nothing.** The reading behind it holds: such a row cannot be outrun — a lane a frame,
// `4.8` units apart crossing the field in one of the two patterns and `0.039` rad apart from one origin in
// the other, which is `11.2` units a frame at the player's own radius against the `4.0` a whole step
// covers — and the ground behind the front is the ground that opens, the lanes fading after about seven
// frames with every dump of the pattern carrying one `fading` with no length left.
//
// **What it cannot do is get there.** The frame it is for has the player *ahead* of the front — one dump
// reads `1.373` rad from the origin against a front at `1.453`, two lanes and so two frames, with every
// one of the nine hit on its own next frame at `-7.1`, `-6.3`, `-6.6`, `-3.5`, `-3.2`, `-5.2`, `-3.8`,
// `-4.1` and `-4.9` of room and `the model was holding 0 within reach` — and behind the front, from there,
// is the far side of seven laid lanes. The attempt with it in read the same one frame of `nowhere open to
// it` on the fan as the attempt without and put all four of its hits above the band, where three of five
// had been.
//
// **And the gate it was written with is worth keeping in mind**, being the thing the try before it could
// not say: more than four lanes with one before them and at least one of those laid this frame or last. A
// pattern that stopped sweeping ten frames ago still has every lane a frame apart in its phases, and the
// two static crosses that boss puts up are eight lanes `147` frames into their own.

// **A row of lanes being laid was fed to that rule as well and taken out again.** A box a row is crossing
// is a box the nine moves can do nothing about once the front is on them — `4.8` units a frame in one of
// the patterns and `11.2` at the player's own radius in the other, against the `4.0` a whole step
// covers — so the frame is decided dozens of frames earlier by which box the player is standing in, and
// `the_lane_laid_before` already says which lanes are a row.
//
// **What it did not do is move the number it was for.** An Extra attempt with it in read one frame of
// `nowhere open to it` on the row and the fan, which is exactly what the attempt without it read; what
// moved was the rest — eight frames on the two crosses against seven, six with no beam on the screen
// against four, and seven hits against five. It fires on a pattern that is not being laid at all, two
// static lanes a frame apart in their phases being all it asks for, and `counted.sweeping` is printed on
// the *bullets*' own line, so fifteen dumps read `0 bullet(s) no step of their own explained, a beam being
// swept about x …`.

/// Where the rules want the player: the x whatever is worth being under has, and the y always low.
fn where_to_be(
    frame: &Frame,
    sweep: Option<f32>,
    goes_live: Option<GoesLive>,
) -> ((f32, f32), Settled) {
    let field = frame.field;
    let at = frame.at;
    let low = field.1 + field.3 - ABOVE_THE_BOTTOM;
    // The item worth going for, at the place it is; under a boss with nothing to collect the boss's own x with
    // the y low, a fight being a thing to stand under rather than to reach; and with neither the middle, low,
    // which is the shortest way to wherever the next item falls.
    // **Or the place a recipe told the rules to stand**, which is the whole errand while it is told: the
    // items, the boss, the middle and the three arms below that leave a box are the rules' own judgement,
    // and what a `stand_at` says is that somebody's is better here — an attack the rules cannot read, a
    // beam laid down the field, is dodged by standing where it is not. The dodging itself goes on from
    // that place as from any other.
    let (left, right) = (field.0, field.0 + field.2);
    // **Or behind a laser the card that is up is dodged by stepping behind**, which outranks a told place
    // for the frames it is drawn — see [`BEHIND_THE_LASER_BY`]. On the row the run is on, and not the low
    // one every told place is on: the step the card wants is short and sideways, and a lane leaning off the
    // upright crosses a row `64` higher a dozen units further along — the whole of the distance being kept
    // behind it.
    if the_dodge() == Some(Dodge::BehindTheLaser)
        && let Some(x) = goes_live.and_then(|it| it.behind(at.1, BEHIND_THE_LASER_BY))
    {
        return (
            (x.clamp(left, right), at.1),
            Settled::TheWayItWasToldToDodge,
        );
    }
    // **Or the place the way the card is dodged is**, where the recipe told the rules no place of its own.
    let told = *TOLD_TO_STAND_AT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (told, settled_there) = match told {
        Some(told) => (Some(told), Settled::ThePlaceItWasTold),
        None => (
            the_dodge().and_then(|it| the_place_it_holds(it, field)),
            Settled::TheWayItWasToldToDodge,
        ),
    };
    let told_y = told.and_then(|it| it.y);
    // **A way's own place moved on a retry, where a recipe's is not**: a place a recipe told is somebody's
    // judgement and not the rules' to vary, but a way's is the rules' own measurement, and held still it
    // makes a retry the attempt before it over again — one Extra was hit at `198.3,428.0` attempt after
    // attempt with the way's middle held through each. See [`aside_on`].
    let aside = match settled_there {
        Settled::TheWayItWasToldToDodge => aside_on(ATTEMPT.load(Ordering::Relaxed)),
        _ => 0.0,
    };
    let told_x = match told.and_then(|it| it.across) {
        Some(Across::At(x)) => Some(x + aside),
        // **The side of the boss the run is already on**, so that the run is not sent across her to
        // reach the other — crossing a boss is crossing what she fires — unless the distance does not
        // fit on this side, where the other side is the only place that far from her.
        Some(Across::AwayFromTheBoss(distance)) => frame.boss.map(|(boss_x, _)| {
            let fits_left = boss_x - distance >= left;
            let fits_right = boss_x + distance <= right;
            let to_the_left = match (fits_left, fits_right) {
                (true, false) => true,
                (false, true) => false,
                _ => at.0 < boss_x,
            };
            if to_the_left {
                boss_x - distance
            } else {
                boss_x + distance
            }
        }),
        None => None,
    };
    if let Some(x) = told_x {
        // Held inside the field and not [`IN_FROM_THE_SIDE`] of it: a recipe that names the edge means
        // the edge, that being where a beam down the middle is not. **And a told y as deep as the floor**,
        // for the same reason, where it was held at the low line: th06c's Extra after its midboss is fairies
        // firing fans of three at the run, the gaps between a fan's lines wider the further they have come,
        // and the strategy wiki's way through it is `最下部から…ちょん避けし続ける`, `64` below the low line.
        return (
            (
                x.clamp(left, right),
                told_y.unwrap_or(low).clamp(field.1, field.1 + field.3),
            ),
            settled_there,
        );
    }
    // **Or out of a beam the run is standing in, which outranks every errand below.** A beam's box is
    // wider than the frames the moves are looked over: inside one, all nine are hit whichever way they
    // go, so the nine say nothing and the run stays where it is — `nowhere open to it` with the dump's
    // own `the nearest beam slot 2, -4.2 outside its box`, four units in and dead there. What gets a
    // person out is walking the short way across it over as many frames as it takes, and that is an
    // errand and not a move.
    if let Some(out) = the_way_out_of_a_beam(frame) {
        return (out, Settled::OutOfABeam);
    }
    // **Or on round the field ahead of a following laser**, for a card dodged that way — see
    // [`ahead_of_a_following_laser`].
    if the_dodge() == Some(Dodge::OutOfTheFollowingLaser)
        && let Some(ahead) = ahead_of_a_following_laser(frame)
    {
        return (ahead, Settled::TheWayItWasToldToDodge);
    }
    // **Or on round the field whatever is up**, for a card dodged that way — see [`round_the_field`].
    if the_dodge() == Some(Dodge::RoundTheField) {
        return (round_the_field(frame), Settled::TheWayItWasToldToDodge);
    }
    // **Or on round a small square under the boss** — see [`round_a_small_square`].
    if the_dodge() == Some(Dodge::RoundASmallSquare) {
        return (round_a_small_square(frame), Settled::TheWayItWasToldToDodge);
    }
    // **Or on from one side of the low line to the other** — see [`side_to_side`].
    if the_dodge() == Some(Dodge::SideToSide) {
        return (side_to_side(frame), Settled::TheWayItWasToldToDodge);
    }
    // **Or where one card's own schedule has the run at this frame of it** — see `crate::cards`.
    if the_dodge() == Some(Dodge::ImperishableShooting)
        && let Some(frames) = the_cards_frames()
        && let Some(there) = crate::cards::imperishable_shooting(frame, frames)
    {
        return (there, Settled::TheWayItWasToldToDodge);
    }
    // **Or at the edge the sword's swing across ends at** — see `crate::cards::laevateinn`.
    if the_dodge() == Some(Dodge::Laevateinn)
        && let Some(there) = crate::cards::laevateinn(frame)
    {
        return (there, Settled::TheWayItWasToldToDodge);
    }
    // **Or round the whole field** — see `crate::cards::and_then_there_were_none`.
    if the_dodge() == Some(Dodge::AndThenThereWereNone) {
        return (
            crate::cards::and_then_there_were_none(frame, the_cards_frames()),
            Settled::TheWayItWasToldToDodge,
        );
    }
    // **Or at the bottom out of the clock's reach** — see `crate::cards::the_clock`.
    if the_dodge() == Some(Dodge::Clock)
        && let Some(there) = crate::cards::the_clock(frame, the_cards_frames())
    {
        return (there, Settled::TheWayItWasToldToDodge);
    }
    let going_for = the_item_gone_for(frame);
    let aside = aside_on(ATTEMPT.load(Ordering::Relaxed));
    let (settled, (x, y)) = match going_for {
        Some((worth, at)) => (Settled::AnItem(worth), at),
        None => match frame.boss {
            Some((x, _)) => (Settled::ABoss, (x + aside, low)),
            None => (Settled::TheMiddle, (field.0 + field.2 / 2.0 + aside, low)),
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
    // **The nearest to the middle and not to the run, under a boss overhead**, which is where her fire opens
    // round the run rather than carrying it out: under a ring out of her the rules stood in the rightmost box
    // for 2,284 of a card's 3,060 frames, the middle the emptiest box on the count and the box they wanted
    // the one beside the edge they stood at — the next one in from where they were, and the dodging of each
    // ring going out past them taking them back to the edge.
    let near = match the_boss_is_overhead(frame) {
        true => BOXES_ACROSS / 2,
        false => mine,
    };
    let elsewhere = || {
        let empty = the_emptiest_box(&fire, near, swept);
        field.0 + field.2 * (empty as f32 + 0.5) / BOXES_ACROSS as f32
    };
    // **And waiting out of a box a beam will be lying across the band in was written and taken out again,
    // which is what the measure itself says is the wrong use of it.** The same two questions the thickness
    // is asked — the box it is heading for and the box it is standing in — with a box it will not be lying
    // across to go to, and the emptiest of those preferred over one it will. Two Extra attempts of it took
    // `6` hits and `8`, every one of the fourteen outside the band, which is the worst pair of any build
    // measured; and the dumps say why. Of nineteen frames one attempt dumped, **ten read `a beam will lie
    // across the band in box(es) 0, 1, 2, 3, 4` and six read `0, 1, 3, 4`** — so this boss closes the whole
    // width of the band most of the time, and where it leaves one box it is the middle one, which is where
    // the two crosses converge and where every hit above the band already is. A rule that leaves the closed
    // boxes therefore aims the run at the hub.
    //
    // **And graded by the frame the arm arrives on rather than left a truth, it is out too.** Which is the
    // answer to what that pair of dumps says — *closed* tells five boxes nothing apart where *closed in
    // three frames* against *closed in thirty-nine* is a place to wait and a place not to — so the measure
    // carries the frame now and `the_emptiest_box` preferred the box whose arm arrives latest among the ones
    // that are not piled. Two Extra attempts took `10` hits outside [`the_band`] or at the clamp of the
    // twenty-one they took, against `12` of twenty-six for the attempts without it: the same attempt twice
    // over, with the first five hits of each at the waiting line and the hub cluster after them.
    //
    // **The measure is kept for what it says and not for what it chose**, which is the dump line beside the
    // fire counts: see `Counted::closes`.
    let (settled, x) = match () {
        // **And the ground behind a row of lanes being laid was wanted here and is not any more.** That row
        // cannot be outrun and what it has laid goes out behind it, so past its back lane is the one place
        // it leaves — and from *ahead* of the front, which is where the frames that need it are, that place
        // is on the far side of seven laid lanes and forty-seven units of wall. An attempt with it in read
        // the same one frame of `nowhere open to it` on the fan as the attempt without, and put all four of
        // its hits above the band where three of five had been. See [`behind_a_sweep`], which is kept for
        // the reading in it.
        _ if swept.is_some_and(|swept| swept == heading || swept == mine) => {
            (Settled::TheEmptiestBox, elsewhere())
        }
        _ if badly => (Settled::TheEmptiestBox, elsewhere()),
        // **And exempting the box it is already standing in from this arm was tried and taken out
        // again**, which is the narrower thing th08's own dump suggests: [`MUCH_MORE_THAN_A_HANDFUL`] says
        // what it takes to leave the box the player is in, *because leaving that one costs the damage it
        // is there for*, and this arm asks the lower [`MORE_THAN_A_HANDFUL`] of that same box whenever the
        // errand happens to be inside it — which under a boss is most of a fight.
        //
        // **What the one launch of it did was lose the stage earlier.** With `heading != mine` added, a
        // pointdevice run of th08's stage 1 never reached the boss it had reached without it: it died in
        // the stage's own midsection twice, at frames 1712 and 1453, both times crossing for a big power
        // item — which this arm never rules on, `worth_the_crossing` being ahead of it. So the run
        // diverged from the first frame the exemption fired and the comparison says nothing about the
        // fight it was made for.
        //
        // **What a fair measurement of it would be** is the boss's own chapter played from the same
        // snapshot with and without, which wants a run that reaches it either way, and the hit counts of
        // the four other games' cases beside it: this is a rule all five share.
        // **And what this arm costs a fight is measured now, which is what that fair measurement wants
        // first.** The per-frame line at the end of [`word_to_press`] says what the rules wanted on every
        // frame of one, and over th08's stage 1 — 10,364 frames — they wanted **a boss on 3,381 of them, the
        // emptiest box on 2,837, a power item on 2,558**, the middle on 679, a point item on 467 and a big
        // power item on 315. So this arm is what displaces the boss a *quarter* of a fight, `going_for`
        // being `None` there and `worth_the_crossing` false with it — and the rules are within 16 units of
        // that boss's own column on 42% of the frames of her fight, which is what every card payout scaling
        // to nothing is made of.
        //
        // **And exempting a boss from it was tried and is not measurable yet.** With `frame.boss.is_none()`
        // added, that stage's own time orbs read `1,986` and the boss was wanted on 3,680 frames against
        // 3,381 — but the five launches of that stage measured over this change read **2,094, 1,986, 1,977,
        // 1,872 and 1,262**, and each drew a stage seed of its own. The spread is wider than the effect, so
        // one launch a candidate settles nothing and the exemption is out until the fixture below exists.
        //
        // **And the fixture exists now, and the exemption measured nothing through it either.** A pick-up of a
        // kept chapter of stage 2 plays the same fight every launch, so the same five chapters ahead of that
        // boss were played with and without a boss exempted while the stage's norm was unmet: card 26 — the
        // last before the norm is asked — ended on `6918`, `6239`, `7442`, `6752` and `6513` without it and on
        // `6766`, `7345`, `6824`, `6448` and `6945` with it, the Last Spell up on three of the five against four,
        // and no hit in any of the ten. A mean `93` up against a chapter's own swing of `-618` to `+1106` is one
        // fight's chaos and not a lever, so the arm is left as it was.
        _ if !worth_the_crossing && the_fire_is_piled_into(&fire, heading) => {
            (Settled::TheEmptiestBox, elsewhere())
        }
        _ => (settled, x),
    };
    // **And keeping the place until the run is there was written and taken out again.** Every one of the
    // three arms above is a threshold on a count of a screen in motion or a detection that comes and goes
    // with what the script does, and a crossing of two boxes is most of a second at the game's own step — so
    // the errand alternates and the run is never at either end of it. One walk's wanted x, a sample in six
    // frames:
    //
    //     118 192 192 192  56  56  56  56  56  56 192 192  56 192 192 192 118 192 192 192 266 266
    //
    // `192` is the boss's own x and `56` the emptiest box, two places `136` apart with the run forty frames
    // from both, and that walk ended above the band at `194.7,214.7`. Held until the run arrived — the box
    // it then stands in being what the arms are asked about next, with a lapse for a place that cannot be
    // reached and a drop for one the fire has piled into since — the flip goes: the next attempt's walk
    // reads `the emptiest box 118` on twenty samples running.
    //
    // **And it made the place worse both times it was run.** Two Extra attempts apiece, counting the hits
    // outside [`the_band`] and the clamp: `5` of six and `6` of seven with it in, against `4` of six and `4`
    // of five without, and its own pair is the highest of every build measured — `6` and `7` hits where the
    // spread of one build is `4` to `7`. Which has a mechanism as plausible as the one it was written for: a
    // place kept is a place crossed to through fire that arrived while the crossing was being made, and
    // nothing measured bounds how long it is worth keeping. So the errand is asked afresh each frame.
    //
    // **And what the flip costs has one measurement now, off th08's stage 1 boss.** A pointdevice run with
    // the bomb forbidden lost that fight three times running, and the dump of the hit — one sample in six
    // frames over the 175 before it, the wanted x alone — reads:
    //
    //     192 192 192 192 192 269 269 269 269  48  48 192 192 192 192  48  48  48  48  48  48  48  48 192
    //     192 192 192 192 192  48
    //
    // Fifteen samples of `a boss at 192` against fifteen of `the emptiest box`, at 269 and then 48, on a
    // field 384 wide. The run's own x follows them across the whole of it — 191, 195, 206, 218, back to
    // 194, then 182, 170, 152, 138, 118, 102, 86, 74 — and the hit lands at `79,417` with **3** of the nine
    // boxes clear, where the first hundred frames of that walk had nine. Which is this comment's own
    // mechanism costing a fight: the errand is 136 to 221 units away and changes before the run has
    // covered a third of it.
    //
    // **Which is not on its own a reason to put the hold back**, the two attempts that measured the hold
    // having made th07's Extra worse — but it is the first number on this side of the trade, and that
    // chapter is reproducible to measure a change against: the same run retried it three times and died at
    // frame 9397 with `seed=0xea6a` every one of them.
    // **Inside the band or waited for at the low line**, and not held at the band's own top edge: an item is
    // almost always above, so clamping its y into the band put the bot to live on that edge — the middle of the
    // field, moving only sideways, with the dodge it wanted to make upwards pulled straight back. Waiting low is
    // where the item arrives anyway.
    //
    // **And the x held [`IN_FROM_THE_SIDE`] off either edge**, which the y has needed all along and the x turned
    // out to need too: a corner is a place with nothing open in it. A field too narrow to keep that off both
    // sides has only its middle to offer, which is where a run with no field read at all ends up.
    // **And the player's own x while it is above the band was tried and taken out again.** The errand is what
    // keeps a climb going — `towards` is one distance over both axes, so a wanted x a hundred units off swamps
    // a y twenty out, and at `77,366` wanting `a boss at 192,368` standing still measures `13229` against
    // `13261` for going up, a quarter of a percent, where wanting `77,384` makes the same two `324` and `484`.
    // What it contradicts is `the_bot_holds_the_bosss_x_and_stays_low`, which is this tree's own answer to the
    // frame it fires on: a player above the band with a boss across the field goes under it diagonally, and
    // wanting its own x makes that move straight down. The narrower form — the band's x only where the errand
    // is the middle or the emptiest box, and the boss's and an item's left alone — is untried.
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
            // **And no lower than the low line**, whatever is below it: an item there is a frame or two from
            // leaving the field, and a run that follows it down is a run at the bottom edge with half its
            // moves gone. Four of th08's stage 1 deaths were that, at `115,432`, `81.7,432`, `75.5,423.8` and
            // `103,391` on a field 448 deep, the last of them `wanting a power item at 214,441` — and the
            // one at `81.7,432` seventeen times over, `nowhere open to it`.
            // **Or the y a recipe told them**, with the x left to the errand: an attack whose ground is a
            // height and not a place wants that — a mesh coming out is stood in from above the band, and
            // where along it is the rules' own business.
            //
            // **Down to the floor for a 1up or a bomb, though**, which is what a run is worth the bottom
            // edge for: a boss that drops one mid-stage drops it onto the player, and one below the low line
            // is one the rules would stand above and watch fall out of the field. A Normal clear wanted
            // `a 1up at 192,368` from `191,368` — the low line itself, a frame's step from the item's x — for
            // 270 frames of one stage and never took it.
            match told_y {
                Some(told) => told.clamp(field.1, low),
                None if y >= the_band(field) => match settled {
                    Settled::AnItem(Worth::Life | Worth::Bomb) => y.min(field.1 + field.3),
                    _ => y.min(low),
                },
                None => low,
            },
        ),
        settled,
    )
}

/// **How much of a box's own width a beam has to lie across for that box to be no place to wait** — half of
/// it.
///
/// Half, because what the measure is for is a box with nowhere left in it to stand: a beam across half of one
/// leaves the other half, which at a fifth of a 384-wide field is `38` units and five steps of room. And
/// because the two things it has to tell apart are `146` units of one arm inside the band against `7` of a
/// beam running down through it — a gap no number between them has to be chosen carefully inside.
const A_BOX_IS_BLOCKED_AT: f32 = 0.5;

/// How many pieces the field is cut into across its width for the fire in each to be counted.
///
/// **An odd cut, so that the middle has a box of its own.** The place a pattern converges on is the middle —
/// an aimed shot goes there and so does most of what a boss fires — and it is discounted for that by
/// [`IN_THE_MIDDLE_ANYWAY`]; an even cut puts that place on a boundary, where the discount lands on neither
/// side of it. Five rather than three is somebody who plays this game asking for a finer answer to *where to
/// wait*: at a third of a 384-wide field, the emptiest box is 128 across and standing in the middle of it is
/// standing 64 from fire either side.
const BOXES_ACROSS: usize = 5;

/// **The room the bot keeps on every side of each place a beam-carrying body has been**, on top of the box that
/// place is tested as.
///
/// The box itself is the body's own size as the game hands it to the test, which for stage 6's spiral is a couple of
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

/// **The smallest bullet th07 fires, which is the unit the fire in a box is counted in.**
///
/// The sizes are written at spawn and there are six of them, from 4 up to 24 — so counting bullets one apiece
/// asks how many heads are in a box where what decides whether there is a way through is how much of its width
/// they take. A large bullet is six of these across, and three of them across a box is a wall where three of
/// these is a gap.
const A_SMALL_ONE: f32 = 4.0;

/// **How much of a box one hazard of a given size fills**, in [`A_SMALL_ONE`]s squared.
///
/// **Squared, because what a box holds is an area.** A bullet twice as wide takes four times the room to weave
/// past, not twice: it blocks that much of the box in both directions at once, and a large bullet is six of the
/// smallest across. Counted by the width alone, a handful of the big ones read as a handful and the box they
/// filled read as passable.
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
/// The middle of the field is where a pattern is thickest by nature — an aimed shot converges there and so does most
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

/// **And the middle's count under a boss overhead**, in place of the two above — see
/// [`the_boss_is_overhead`].
///
/// **Because fire leaving a boss overhead, counted as it stands, reads the middle as the pile it is not.** A
/// ring goes out at evenly spaced angles, so where it comes down into the band is spaced by the angle each box
/// subtends from her: from the middle of the top at about `y = 100`, over the top of the counted band at
/// `y = 179` of a field 384 wide, the middle box takes `51°` of it, each box beside it `30°` and each edge box
/// `12°`. So a ring alone reads about **4.2 times** as full in the middle and **2.4** beside it as at an edge,
/// and the discount above, a tenth and a twentieth, left the edge box the emptiest: the rules walked out to the
/// edge of the field under every ring, where the next one closes the corner. **A fifth and two fifths** is a
/// little under what cancels that shape out, so that the middle — where somebody who plays this game waits for
/// a ring to open round them, and asked for the rules to — is the box chosen.
///
/// **Under any fire leaving her and not a ring's alone**, which a ring's own reading was tried as and what
/// it missed says why not: a survival card whose fire leaves the boss in every direction and at every speed
/// at once — random, a winder, aimed three-ways, crossing — was read as a ring on **none** of its 7,592
/// frames in one run, the boxes averaged `36, 49, 156, 50, 37` and the rules stood in the leftmost box for
/// 3,600 of them. What shapes the count is where she is and not which pattern she fires.
const IN_THE_MIDDLE_UNDER_HER: f32 = 0.2;
const BESIDE_THE_MIDDLE_UNDER_HER: f32 = 0.4;

/// **Whether the boss is overhead**: in the middle box of the field, which is where the fire she sends out comes
/// into the band converging on the middle — see [`IN_THE_MIDDLE_UNDER_HER`].
///
/// **At any height**, which is what somebody watching the rules asked for once the top third alone was tried:
/// a boss lower down sends her fire into the band from nearer, steeper and so more converging on the middle
/// still, and the rules went to an edge under a ring out of a boss in the middle with the rule in.
fn the_boss_is_overhead(frame: &Frame) -> bool {
    let field = frame.field;
    frame
        .boss
        .is_some_and(|(boss_x, _)| which_box(field, boss_x) == BOXES_ACROSS / 2)
}

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
    // **And the lanes the beams are laid along are not counted here, which two measures of them say is
    // right for this count rather than for the beams.** A beam is its own array and not one of the
    // hazards, so a screen with ten lanes down one side of the field and nothing down the other reads as
    // empty on both — and that is one Extra pattern outright: its boss crosses laying one lane a frame,
    // `4.8` units apart and `6.0` wide, and a player caught in front of that front cannot be got out of
    // the way of it at `4.0` a frame.
    //
    // **[`how_much_it_fills`] is the wrong shape for a lane.** It squares, a bullet twice as wide taking
    // four times the room to weave past, so a warning `768` across read as `36864` in the box its axis
    // crossed, made the middle of the field the thickest box there is, and sent the bot to the edge on the
    // very frame `a_warning_line_is_crossed_where_they_leave_nowhere_to_go` asks it to cross.
    //
    // **And the share of a box a lane covers is the right shape and the wrong scale.** A lane blocks
    // across and runs the whole way along, so what it takes out of a box is the width it covers for the
    // whole of the band this count is over — `covered × the band / A_SMALL_ONE²`, which is `100` for one
    // `6`-wide lane inside one box against a box that holds `1290`, and `1290` in every one of the five
    // for a lane as wide as the field, so that one tells the boxes nothing apart and the warning case
    // passes. What it runs into is [`MUCH_MORE_THAN_A_HANDFUL`]: `clearly_fuller` asks for `9` and one
    // ordinary lane crossing the waiting depth puts `134` in a box, so *any* beam over the bot becomes a
    // reason to cross the field — and `the_bot_leaves_a_laser_that_is_turning_onto_it` says a beam
    // pointing wide of the player is nothing to move for at all.
    //
    // So what a beam wants is an answer of its own rather than a share of this one, whose thresholds are
    // the bullets' — and `steer`'s own worklist is where that is written down.
    // **The middle discounted, and the two beside it discounted less.** The converging a pattern does is a thing
    // of the middle of the field, and it falls off either side of it rather than stopping at a box's edge —
    // and much more of it under a boss overhead, see [`IN_THE_MIDDLE_UNDER_HER`].
    let overhead = the_boss_is_overhead(frame);
    let (in_the_middle, beside_it) = match overhead {
        true => (IN_THE_MIDDLE_UNDER_HER, BESIDE_THE_MIDDLE_UNDER_HER),
        false => (IN_THE_MIDDLE_ANYWAY, BESIDE_THE_MIDDLE_ANYWAY),
    };
    let middle = BOXES_ACROSS / 2;
    fire[middle] *= in_the_middle;
    fire[middle - 1] *= beside_it;
    fire[middle + 1] *= beside_it;
    // **And an edge no emptier than the box beside it**, the discount alone not reaching the frames a boss's
    // fire is densest on. Fire leaving a boss comes to an edge box last, it being the furthest from her, so
    // over the forty frames counted the edge reads empty where what is on its way to it has simply not
    // arrived — and the edge is the place with one way out. A survival card read `7, 81, 212, 85, 7` on the
    // frame a run left for `x = 56`, which the discount makes `7, 32, 42, 34, 7` and still an edge; and th08's
    // card 152 read `0, 11, 10, 9, 0` 175 frames before a hit at `23.1,383.1` with nowhere open, the fire
    // having piled into the box beside the edge by then.
    //
    // **Under her or not**, which is where this rule began and what the same card said was too narrow: five
    // Kaguya clears were hit on card 152 355 frames into it at `x = 44.6` to `47.3`, each after wanting the
    // emptiest box at `56` with the boss at `x = 80` — the box beside the edge, so not overhead — one of them
    // off `0, 6, 0, 4, 2`, where the edge and the middle both read empty, stood equally near, and the edge
    // came first.
    fire[0] = fire[0].max(fire[1]);
    fire[BOXES_ACROSS - 1] = fire[BOXES_ACROSS - 1].max(fire[BOXES_ACROSS - 2]);
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

/// **Whether the fire is clearly piled into one box of the field.**
///
/// **Measured against the emptiest of them and not against the screen.** Asking whether one box holds
/// more than half of everything is asking the wrong question: a run was hit in the middle with the left
/// obviously empty, and the middle still held under half because the right held the rest. What matters is that
/// there is somewhere clearly better to be.
///
/// Clearly, and said in two numbers rather than a taste: **twice the emptiest**, which is a factor and so means
/// the same at thirty bullets as at three hundred; and **more of a difference than there are moves to weave
/// between**, so that a handful is not a pile. An ordinary imbalance is not a reason to leave a fight.
///
/// **Asked of the box itself, and not answered as the first box of the five that is piled.** An edge is counted
/// no emptier than the box beside it — see [`the_fire_in_each_box`] — so wherever the box beside an edge is piled
/// the edge is too, and comes first: asked which box is the thick one, the answer was the edge, and a run heading
/// for a boss in the box beside it was left heading into the pile.
fn the_fire_is_piled_into(fire: &[f32; BOXES_ACROSS], box_: usize) -> bool {
    clearly_fuller(fire, box_, MORE_THAN_A_HANDFUL)
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
/// one is walking out from under the boss, and a fight nobody shoots is a fight that does not end — stage 6's own
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
///
/// **The nearest of the boxes as empty as that one, and not the bare least of the five.** What the count
/// measures is a screen in motion, and the least of five floats is settled by a tenth of a bullet: six
/// consecutive samples of one th06c Extra's walk wanted `56`, `56`, `266`, `56`, `118` and `56`, the two
/// that flipped reading `11, 13, 16, 11, 11` and `16, 13, 26, 13, 16` — a place to wait thrown 210 units
/// across the field and back while the boxes either side of it were inside a unit of each other. A run
/// given that spends the pattern crossing, and it is crossing that takes it up out of [`THE_BAND`]: the
/// same walk goes on to `313`, `280`, `263` and the hit at `176,202`.
///
/// So a box that is not [`clearly_fuller`] than the emptiest is as good as the emptiest, and of those the
/// nearest is the one to want — the crossing itself being the cost this is weighed with. Equally near and
/// equally empty, the emptier by the float decides, so that nothing here leans on [`BOXES_ACROSS`]'s own
/// order.
///
/// **And the box a beam is being swept across is no candidate however little it holds**, which is the whole
/// of why [`where_to_be`] asks about a sweep separately: a dozen swept bullets are not a pile, so the
/// counting cannot be left to rule that box out.
///
/// **And a beam's own arrival was ranked in here twice and is out both times** — see [`where_to_be`]'s note,
/// which has what the measure says about itself: as a truth, this boss closes the whole width of the band
/// most of the time and the one box it leaves is the middle, where the hits already are; as the frame it
/// arrives on, so that a box an arm reaches in thirty-nine frames beats one it reaches in three, two Extra
/// attempts took `10` hits outside [`the_band`] or at the clamp of twenty-one against `12` of twenty-six
/// without it, which is the same attempt twice over. What the measure is kept for is the dump line.
///
/// **And *as empty as the emptiest* is twice it, not [`TIMES_THE_EMPTIEST`].** That factor is tuned for
/// *leaving*, where a quarter over the emptiest is a reason to go; measured here it made every box between
/// the run and one empty box at the far edge count as piled, and the nearest box that was not piled was
/// that far one. th08's stage 1 boss is where that read as a walk across the whole field: the boxes
/// `7, 48, 28, 36, 14` with the run in the fourth wanted the first, 234 units away, past the fifth at 14 —
/// twice the emptiest, and no pile. A box holding under twice what the emptiest holds is a place to wait
/// beside, and the nearest such box is the one to cross to.
///
/// **Measured on the one fight, from one file.** The rules picked that boss's chapter up out of
/// `pointdevice_resume/` and lost it seven times over at one hit with the boss at 5335; with the far box
/// no longer outranking the near one the same pick-up beat that attack and lost the spell card after it
/// with the boss at 1125. `the_thick_of_it_is_crossed_for_a_big_power_item_and_not_for_a_point` is the
/// other side of the line: six over nothing, two boxes off, is still a box to leave for.
fn the_emptiest_box(fire: &[f32; BOXES_ACROSS], near: usize, swept: Option<usize>) -> usize {
    let empty = fire.iter().copied().fold(f32::INFINITY, f32::min);
    let piled = |box_: usize| {
        fire[box_] > empty * TWICE_THE_EMPTIEST && fire[box_] - empty > MORE_THAN_A_HANDFUL
    };
    (0..BOXES_ACROSS)
        .filter(|box_| Some(*box_) != swept)
        .min_by(|a, b| {
            piled(*a)
                .cmp(&piled(*b))
                .then_with(|| a.abs_diff(near).cmp(&b.abs_diff(near)))
                .then_with(|| fire[*a].total_cmp(&fire[*b]))
        })
        .unwrap_or(near)
}

/// How many times the emptiest box a box has to hold before it is no longer a place as good as the emptiest
/// to wait in — see [`the_emptiest_box`], which has why this is not [`TIMES_THE_EMPTIEST`].
const TWICE_THE_EMPTIEST: f32 = 2.0;

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
///
/// **And asking [`THE_BAND`] of it once the counting projects was tried, which is the same question with the
/// projection behind it.** `where_it_comes_into` walks each hazard [`THE_FRAMES_AHEAD`] along its own step and
/// counts it where it first comes into this band, so *will it be somewhere to be by then* is already the
/// question being asked — and narrowing the band to the one the bot moves in makes that *will it reach the
/// bottom third in forty frames*, which two frames of the suite refuse:
/// `a_box_is_counted_by_the_fire_in_the_bottom_half_of_the_field` is the run above, whose fire never reached
/// the third at all, and `fire_on_its_way_into_the_band_is_counted_where_it_will_be` has a wall descending
/// from `100` at three a frame, which is `220` by the fortieth — inside this band and short of that one. So
/// the depth is not what the counting is missing, and the projection did not change the answer.
const COUNTED_BAND: f32 = 3.0 / 5.0;

/// The y above which fire is not counted towards a box.
fn the_counted_band(field: (f32, f32, f32, f32)) -> f32 {
    field.1 + field.3 * (1.0 - COUNTED_BAND)
}

/// **What each kind of pickup is worth going for**, largest first — a life, a bomb, the big power item, the small
/// one, the point item, the cherry, and last anything the six of them do not name.
///
/// Which is th07's own worth of them and near enough every game's: a life is a life, a bomb is a life this
/// bot's deathbomb will spend, and a big power item is eight steps of power against a small one's one. **A kind
/// with no place of its own scores nothing**, and [`the_item_worth_going_for`] goes for one only where it costs
/// nothing — a step or two — rather than as an errand.
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

/// **The item the rules go for**: [`the_item_worth_going_for`], and none at all through a card dodged
/// [`Dodge::WithoutTheItems`], whose boss lets them fall down the column her fire is thickest in.
///
/// th08's `禁薬「蓬莱の薬」` was hit on eighteen of the twenty-three whole runs that reached it, seventeen of them in
/// its last stretch and sixteen at the floor between `x = 143` and `212`; a Normal clear through that stretch wanted
/// a point item on 22% of its frames, `1,048` of the `1,114` at Eirin's own `x = 192`, falling from `y = 288`. Two
/// practices of the card, the rules otherwise the same, wanted one on 7% and were hit on neither — the boxes' own
/// `118`, `192` and `266` being where they waited instead.
fn the_item_gone_for(frame: &Frame) -> Option<(Worth, (f32, f32))> {
    match the_dodge() == Some(Dodge::WithoutTheItems) {
        true => None,
        false => the_item_worth_going_for(frame),
    }
}

/// The item worth going for, or `None` where the screen is empty.
///
/// **The worth first and the distance only inside it**, which is what makes a big power item across the field
/// beat a small one under the player's hand. Between two of a kind it is the nearer.
///
/// **And no power item at all where the shot is already full**, whichever kind: what one is worth is the power
/// it adds, and at the top there is none to add. th07 turns one into something else there and hands it over
/// as that, so this is th06's rule — its own stay power items worth a scrap of score, and a run that crossed
/// the field for one crossed it for nothing and dodged nothing on the way.
fn the_item_worth_going_for(frame: &Frame) -> Option<(Worth, (f32, f32))> {
    let (at_x, at_y) = frame.at;
    // **And under a boss, a kind worth nothing only where it costs nothing**: within a couple of steps on
    // both axes, which is a pickup made on the way rather than an errand. th08's graze scraps and the three
    // an erased bullet leaves are that kind, and a spell card is where they fall thickest — a run there
    // wanted `an item of no place of its own at 335,384` from `234,387` under 390 bullets, and died crossing
    // to it thirteen times running, `nowhere open to it`. **Under a boss and not everywhere**, because a
    // stage with nothing else on the screen is where such a kind is worth the walk it always was —
    // `the_bot_takes_the_items_in_their_own_order` is that walk — and a fight is where the walk is paid for
    // twice, in the crossing and in the damage not done under the boss.
    let (reach_x, reach_y) = (2.0 * frame.straight.0, 2.0 * frame.straight.1);
    // **And sweeping a field a captured card has cleared was tried for th08's own norm and taken out again.**
    // Those items are that game's **time orbs** — a card taken there cancels the screen and leaves one per
    // bullet cancelled — and the count a stage's Last Spell is due on is counted in them, which is what the
    // route to its own last stage turns on. One clear's stage 1 read **2,094 of the 2,500** its norm wants
    // with the count climbing in steps of 21 to 25 and no card payout in any of them, so the orbs looked like
    // the half to buy. Told to go for such an item wherever nothing was inside 96 units — twenty-four frames
    // of a bullet's own step — the same stage read **1,872**, and told the field's middle through that boss's
    // whole fight it read **1,977**: both *below* the run that was told nothing. What the sweep costs is the
    // damage not done while the shot is off the boss, which is the payout the norm is really short of.
    frame
        .items
        .iter()
        .filter(|item| {
            worth(item.worth) > 0
                || frame.boss.is_none()
                || ((item.at.0 - at_x).abs() <= reach_x && (item.at.1 - at_y).abs() <= reach_y)
        })
        .filter(|item| !frame.full_power || !matches!(item.worth, Worth::Power | Worth::BigPower))
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
