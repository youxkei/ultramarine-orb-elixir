//! **The grammar of a recipe orb plays a run to**: a list of steps carried out in order.
//!
//! ```yaml
//! - press: [decide]
//!   frames: 2
//! - play_until: { stage: 2, chapter: 3 }
//! - screenshot: the midboss
//!   frames: 30
//! - start_over: true
//! - take_hits: { chance: 0.2, times_each_retry: 0.5 }
//! - bombs: false
//! - stand_at: { x: 40 }
//! - name: ORB
//! - leave: true
//! - wait_until: { question: resume }
//! - quit: true
//! ```
//!
//! **The difference between the first two is the whole reason for a recipe.** A `press` holds a word for a
//! number of frames, which is what a screen takes: a screen answers the same way however long the run before
//! it took. A `play_until` hands the frames to the game's own rules until something is *reached*, which is
//! what a stage takes: how many frames stage 1 costs is not a number anybody can write down, so a recipe that
//! counted them would be a recipe for one run of one afternoon.
//!
//! `wait_until` is a `press` of nothing with an `until` — the frames a screen takes to come up are as
//! unwritable as a stage's. `quit` closes the game, which is the last step of a recipe nobody is watching.
//!
//! **And two of them ask for something rather than pressing at it.** A `screenshot` asks for a picture of the
//! frame being drawn, which is how a screen full of words is checked: whether the words on it are the ones
//! somebody reads is not a thing any line of the log can say. Its `frames` are how long to let the thing it is
//! of be drawn first, a screen and a card's plate each having an animation of their own. A `start_over` asks
//! the run to start over the way the game's own pause menu item does, which orb turns into its question of
//! where to start again — the one way to reach a chapter again without dying for it, so the one way a recipe
//! can ask for a retry. What answers that question is a `press` at `until: { question: retry }`, the same as a
//! player pressing at it.
//!
//! **And one of them says how the run is played rather than doing anything itself.** A `take_hits` hands the
//! rules a chance of being hit on purpose in each chapter, which is the one thing a run played by rules that
//! dodge everything cannot show: a death, the chapter retried, and the run carrying on from it. It stands from
//! the step it is written at until another one changes it, so a recipe can play the stages it wants clean and
//! take hits through the rest. A `bombs` is the other such step: `false` forbids the rules the bomb from there
//! on, which is what a run held to no bomb is written with, and `true` hands it back. **And a `stand_at` tells
//! the rules where to stand**, from there on until a `stand_at` with nothing in it frees them: an `x` across
//! the field, or `away_from_the_boss`, a distance to keep from the boss's own x on whichever side the run is
//! on with the whole distance to spare. The rules still dodge from that place, and the place is the recipe's
//! rather than theirs — which is what an attack the rules cannot read their way out of wants: a beam laid down
//! the field from where the boss stands is one a run kept a beam's width from her is never under, wherever
//! she goes. **And a `dodge` names a way through one card**, for a card no place gets through: the way is
//! the rules', carried out in `orb_bot::steer`, and what the recipe says is which card wants which — see
//! [`Dodge`].
//!
//! **Which is what the steps that say how a run is played are for: the chapter the rules cannot clear gets its
//! own way past it here, rather than a change to the rules that every game pays for.** A `play_until` naming
//! that chapter's card by its number — `spell: 94` — is what brackets it where it is a card, and one naming
//! the chapter's own boundary where it is not: stand where the chapter wants, free the rules on the one
//! after. The recipe's own comment is where which attack it is for goes, with the place and the count of what
//! the rules died to without it.
//!
//! **A reading first, where there is one to make.** A chapter lost to something orb never handed over is a
//! reading missing and not a chapter wanting a strategy: th08's stage 4 boss took 28, 15 and then 7 deaths at
//! a place apiece until her beam was read off the anchor its script writes, and neither told place dodged it,
//! that beam being aimed at the player rather than laid from her. What a told place is for is the attack that
//! *is* read and still not survived.
//!
//! **And `leave` gives the run up for the title, which `quit` cannot be.** Closing the process leaves a run
//! nobody left — no last frame and no line saying the run ended — where somebody playing goes back to the title
//! from the pause menu, and it is *that* interruption a resume is for. So a recipe that measures one says both:
//! leave the run, and then close the game.
//!
//! **And a recipe that leaves a run answers the resume question itself.** The chapter the run was in is kept
//! as it goes, so orb puts its own question up over the front end the give-up lands on — and every `press` a
//! recipe makes after that is read as the answer to it. A launch of th08 pressed `down` at what it took for
//! the title, the question took that as its answer, the playback of a chapter 38,564 frames deep began over a
//! run that had just been torn down, and the game went out with `0xc0000005` in the text element walk that
//! teardown had freed. Answer it with an `answer` step — `はじめから` where the recipe is about what comes
//! after the give-up — and the presses after it go where they were meant to.
//!
//! **And a `press` with an `until` of its own is a tap**: down for its frames, up for as many, over and over
//! until what it waits for is reached. Which is how a run is *started* without counting the screens on the way
//! — a menu answers the frame a button goes down, so a word held across four screens takes one of them, and
//! how many there are is not something a recipe can know: which item a title menu comes up on is whatever the
//! game's own score file last remembered.
//!
//! **What a tap waits for has to be something the tap cannot take away.** A tap of `decide` waiting on a
//! `question` answers that question the frame it comes up, so the frame the walker next looks at has no
//! question up and the step never ends — which is a launch that walked its whole front end and then tapped
//! through a stage without the rules ever getting a frame. A `stage` and a `chapter` are *reached*, so no press
//! can undo one.
//!
//! **The grammar is here and the names are the game's.** A list of button names, a stage and a chapter and a
//! question is the same list whatever game reads it, so it is read once — and what `decide` *is*, and which
//! chapter a stage's midboss falls in, differ from game to game, so nothing here resolves either.
//! `game::th07::bot` is where th07's are, and a name that game has nothing for is a recipe it refuses.
//!
//! Here rather than in `orb-core` because this is where the YAML reader is: `orb.yaml` goes through the same
//! one, and a second copy of it in the DLL for a diagnostic would be weight in what ships.

use std::ops::Range;

use serde::Deserialize;

/// One step of a recipe, as [`read`] hands it over: one of these and never two of them.
///
/// **Compared and not ordered or hashed**, [`Step::TakeHits`] carrying a chance: a chance is a float, and what
/// a test asks of one is whether it read back as the number the file said.
#[derive(Debug, PartialEq)]
pub enum Step {
    /// A word held for a number of frames, which is what a screen takes — or **tapped** for as long as it
    /// takes, where an `until` says what it is waiting for: held for `frames`, released for as many, over and
    /// over.
    ///
    /// Tapped rather than held because a menu answers the frame a button goes down: a word held across four
    /// screens takes one of them. And for as long as it takes rather than a count of taps, because how many
    /// screens there are between the title and a stage is not something a recipe can know — which item a title
    /// menu comes up on is whatever the game's own score file last remembered.
    Press {
        buttons: Vec<String>,
        frames: u32,
        until: Option<Until>,
        give_up_after: Option<u32>,
    },
    /// The game played by its own rules until something is reached.
    Play {
        until: Until,
        give_up_after: Option<u32>,
    },
    /// And nothing held until something is reached, which is what waiting for a screen to come up is.
    Wait {
        until: Until,
        give_up_after: Option<u32>,
    },
    /// **The run given up for the title**, which is how a session is interrupted rather than killed: the run
    /// ends, and the file it was written down in is the one the next launch offers to pick up.
    ///
    /// Which is what [`Step::Quit`] cannot be. Closing the process leaves a run nobody left — no last frame, no
    /// line saying the run ended — and the interruption a resume is *for* is the other one, somebody going back
    /// to the title from the pause menu.
    Leave,
    /// The game closed, which is a recipe nobody is watching having finished.
    Quit,
    /// **A picture of the frame being drawn, named by what asked for it.** What a recipe is for here is a
    /// screen rather than a number: whether the words on it are the ones somebody reads is not a thing any
    /// line of the log can say, so the frame itself is the answer and a person or another tool reads it.
    ///
    /// Named rather than numbered, because a directory of pictures is read by what each one was taken of.
    ///
    /// **And `frames` of the game's own rules first, which is what a thing being drawn takes.** A screen and a
    /// card's plate each have an animation of their own, so the frame a condition became true is the frame
    /// before the thing it is about is on the screen — three pictures of three menus each came out mid-fade
    /// showing the menu before, and one asked for as a card started came out with no plate at all. The frames
    /// are the rules' to play rather than nobody's: a run whose player stands still for half a second is a run
    /// that gets shot.
    Screenshot { named: String, frames: u32 },
    /// **The run asked to start over, which is what the game's own pause menu item does** — and what orb
    /// turns into its question of where to start again. The one way to reach a chapter again without dying
    /// first, so it is the one way a recipe can ask for a retry.
    ///
    /// It raises the question and answers nothing: what to answer is a `press` at
    /// `until: { question: retry }`, the same as a player pressing at it.
    StartOver,
    /// **The rules handed a chance of being hit on purpose in each chapter**, and what that chance is
    /// multiplied by each time the same chapter is retried.
    ///
    /// **Which is the one thing rules that dodge everything cannot show.** Pointdevice mode's whole content is
    /// what happens after a death — the chapter put back, the run carried on — and a run that is never hit
    /// reaches none of it: every measurement of a retry so far has been a chapter asked to start over, which
    /// is the pause menu's ask and not a death at all.
    ///
    /// **A chance per chapter and not per frame**, because the chapter is what a retry puts back: the draw is
    /// made once as the chapter starts and says whether this attempt at it ends in a hit.
    ///
    /// **And lowered each retry, so that a run gets past the chapter it died in.** At the same chance every
    /// attempt a chapter is retried until the fire happens to miss, which is a run that spends its afternoon
    /// on one chapter; multiplied down, the attempts after the first are played more carefully than the one
    /// before — which is what somebody who died there does.
    ///
    /// A chance of nought is the rules dodging everything, which is what a recipe that says nothing gets.
    ///
    /// **And `gives_up` is the chance that a death is answered by going back to the title** rather than by
    /// taking the chapter again, which is the other thing somebody who has just died does — and the one that
    /// leaves a run in a file for the next launch to pick up.
    TakeHits {
        chance: f32,
        times_each_retry: f32,
        gives_up: f32,
    },
    /// **Whether the rules may press the bomb**, from the step it is written at onwards.
    ///
    /// **A run played without one is a thing a recipe has to be able to ask for**, a *no-bomb* clear being
    /// what a run is measured by where a run that bombs its way through is not the same measurement: the
    /// rules bomb where a hit has landed and the bomb takes it back, and where nothing has been open for a
    /// second — see `orb_bot::steer` — and both are a bomb spent that a run held to none cannot spend. What
    /// a recipe that says nothing gets is the rules bombing as they see fit, which is every recipe written
    /// before this.
    Bombs { allowed: bool },
    /// **Where the rules are told to stand**, from here on while they dodge — see the module's doc for what
    /// it is for. An `x` across the field, or a distance to keep from the boss's own x
    /// (`away_from_the_boss`), the side being whichever the run is on with the whole distance to spare;
    /// both `None` frees them.
    ///
    /// **Neither dodges a beam aimed at the player**, and a launch of each says so: th08's stage 4 boss
    /// lays one 160 wide along the angle she is facing, and a run held at the field's left edge died to it
    /// fifteen times while a run held 120 from her x died at `90.8` with her at `210.8` and the beam's own
    /// column at `128.5` — 38 units from its middle, the step having moved the run into it. What a told
    /// place is for is an attack whose ground does not follow the run.
    ///
    /// **And with a `spell` beside it the place is that card's alone**: it stands while the card of that
    /// number runs and lapses when it ends, the place written without one standing the rest of the time. So
    /// a told place needs no `play_until` either side of it, which is what a chapter's number could not give
    /// it — see [`Until::Spell`] for how a stage's own table moves those.
    StandAt {
        x: Option<f32>,
        y: Option<f32>,
        away_from_the_boss: Option<f32>,
        spell: Option<u32>,
    },
    /// **And a place that is one stretch of a stage's**, by the stage's own frames, standing while that stage's
    /// frame is inside them — the way [`Step::DodgeAStretch`] names its ways, for fire that is no card and
    /// wants a height as well as an x: th10's stage 4 midboss and the waterfall after her carried Normal runs
    /// down to the floor and along it into a corner, `y = 432` on every hit, with the way's own height the low
    /// line.
    StandAtThroughAStretch {
        x: Option<f32>,
        y: Option<f32>,
        away_from_the_boss: Option<f32>,
        stage: i32,
        frames: Range<u32>,
    },
    /// **A way of dodging one card, by its name**, which the rules take up while the card of that number runs
    /// and put down when it ends — see [`Dodge`].
    ///
    /// **A name and not numbers**, because what a way does is `orb_bot`'s and what a recipe says is which card
    /// wants which. Most of them are a way through and not somewhere to stand — the laser that goes live where
    /// the run stood is dodged by stepping behind it wherever the run is, and no one place is that — and the
    /// ones that are a place are a place `orb_bot` holds the measurement of, which a [`Step::StandAt`] of the
    /// same numbers would leave to every recipe to copy.
    Dodge { with: Dodge, spell: u32 },
    /// **And a way of dodging a stretch of one stage**, by the stage's own frames, which the rules take up while
    /// that stage's frame is inside them — for fire that is no card: th10's stage 4 midboss's first attack hit
    /// a Normal run in the bottom-left corner on five launches of six, between 800 and 1,350 frames into it,
    /// and its number is no card's to name.
    ///
    /// **The stage's frames and not a chapter's number**, because a chapter's number moves with the stage's
    /// own table, and the frame a stage's script brings a midboss on at does not.
    DodgeAStretch {
        with: Dodge,
        stage: i32,
        frames: Range<u32>,
    },
    /// **And a way of dodging what a boss fires between one card and the next**, named by the card before it,
    /// which the rules take up once that card has ended and put down as the next one starts — for fire that has
    /// neither a card's number nor stage frames of its own: th10's Suwako's nonspell after
    /// `蛙狩「蛙は口ゆえ蛇に呑まるる」` carried a whole Extra into the bottom-left corner and hit it there, and the
    /// stage frame it begins on moves with how long every card before it took.
    ///
    /// **The card before it and not the one after**, because the one after is not up until the fire is over.
    DodgeAfterACard { with: Dodge, after: u32 },
    /// **And a way of dodging one chapter of a stage**, by the stage and the chapter's own number as orb counts them,
    /// which the rules take up while that chapter is the run's — or, with `frames`, while the chapter's own frame is
    /// inside them, counted from the frame it began on. What the autoplay engine's ways are written by, a chapter
    /// being the one thing a stage of every difficulty is divided into that a recipe and the engine both see.
    DodgeAChapter {
        with: Dodge,
        stage: i32,
        chapter: u32,
        frames: Option<Range<u32>>,
    },
    /// **And a place to stand through one chapter of a stage**, by the stage and the chapter's number, or a stretch of
    /// its own frames — the way [`Step::DodgeAChapter`] names its ways: th06c's Extra holds the floor from 300 frames
    /// into its chapter 7 for 2,313 frames, which no card's number and no stage frame of a run that was slowed or
    /// sped somewhere before it would name.
    StandAtThroughAChapter {
        x: Option<f32>,
        y: Option<f32>,
        away_from_the_boss: Option<f32>,
        stage: i32,
        chapter: u32,
        frames: Option<Range<u32>>,
    },
    /// **A player's own path through one card, gone along again**, which the rules take up while the card of that
    /// number runs and put down when it ends: on the card's frame `every`, `2 × every` and on, the place written for
    /// it, between two of them the way between, and past the last the last — as `orb-real paths` writes it out of a
    /// trace of a replay played back.
    ///
    /// **A path and not a way of the rules' own**, for a card whose fire is all aimed at where the player is: gone
    /// where a player who went through it went, frame for frame, the card fires as it fired at them.
    Follow {
        spell: u32,
        every: u32,
        path: Vec<(f32, f32)>,
    },
    /// **An item of one of orb's own questions, taken outright rather than pressed for.**
    ///
    /// **Which is the one thing a press cannot do to those screens.** A question of orb's keeps its own cursor
    /// and nothing of it reaches a bot, so every press takes the item the cursor came up on: `つづきから` and
    /// never `はじめから`, `はい` and never `いいえ`, and the first stage, chapter and attempt of a replay and
    /// never any other. Three launches on the real game were written to press `down` and then `decide` at
    /// `どこから始める` and all three came back with the item the cursor was on.
    ///
    /// The item is its place in the list on the screen, counted from nought — which is what a recipe can say
    /// about a list whose length it cannot see. A step names one question, so the four lists a replay is
    /// chosen out of are four steps.
    ///
    /// It waits for that question to be up, the same as a `wait_until` for one, and ends when the answer has
    /// been taken — so it takes a bound the same way, a question that never comes up being a step that would
    /// otherwise never end.
    Answer {
        question: String,
        item: usize,
        give_up_after: Option<u32>,
    },
    /// **The name a run's ranking is registered under, typed into the game's own entry.**
    ///
    /// **Which a press cannot do.** That entry is a grid of characters with a cursor of its own, and a
    /// `decide` at it writes whatever character the cursor happens to be on and moves to the next place
    /// in the name — so a step that taps `decide` until the screen goes away registers eight of the
    /// letter the cursor started on, and `AAAAAAAA` is what every clear a recipe played was ranked as.
    /// Walking the grid instead is thirty-odd presses a name, each of them blind: the cursor is the
    /// screen's own and not the one an `until` can wait for.
    ///
    /// So the name is written where the screen's own keys write it and the entry left on its end item,
    /// which leaves one press to finish it — see `Game::types_its_name`.
    ///
    /// It waits for that entry the way a `wait_until` waits for a screen, and takes a bound for the same
    /// reason: a run that ends some other way never reaches one.
    Name {
        typed: String,
        give_up_after: Option<u32>,
    },
}

/// **The ways of dodging a card the rules have by name**, which a [`Step::Dodge`] picks one of for a card and
/// `orb_bot::steer` carries out — named in a recipe the way it reads, `behind_the_laser`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dodge {
    /// A laser drawn off to one side of the run, turned onto where the run stood when it appeared and turned
    /// live there and on past: stepped the short way across towards the side it came from before it goes
    /// live, and stood behind as it sweeps.
    BehindTheLaser,
    /// A beam an enemy's script aims where the run is, holds there while it warns, fires wider than it can
    /// be read to be before it does and then turns after the run: kept clear of at the width it fires at,
    /// and led round the field away from it, corner by corner, as it turns.
    OutOfTheFollowingLaser,
    /// Lasers laid up the field from its floor with aimed fire between them: waited for in the field's
    /// middle, where there is room either side of each.
    InTheMiddle,
    /// th08's `狂符「幻視調律（ビジョナリチューニング）」`, whose lattice is hidden on the same frames of the card every
    /// launch: climbed back up through while it is hidden, and waited for in the middle below the band it is
    /// born in — see `orb_bot::cards`.
    VisionaryTuning,
    /// A boss that follows the run about and fires from where it is: never waited for at all, the run going
    /// on round the field corner by corner.
    RoundTheField,
    /// Familiars that stay on the run and fire from where they are: never still, the run going round a small
    /// square under the boss clockwise at the whole step.
    RoundASmallSquare,
    /// Lasers a boss aims at the run, with fire strewn between them: never still either, the run going from
    /// one side of the low line to the other and back, so that where each laser is aimed is where the run was.
    SideToSide,
    /// Fire whose boss lets items fall down the column it is thickest in: dodged where the rules' own boxes
    /// say, with no item gone for.
    WithoutTheItems,
    /// Fire whose gaps are narrower than the run's whole step: dodged focused from the card's first frame to
    /// its last, at the fine step, the place being the rules' own.
    Focused,
    /// th08's Extra Last Spell `「インペリシャブルシューティング」`, whose eight waves come at the same frames of
    /// the card every launch: dodged by that schedule — see `orb_bot::cards`.
    ImperishableShooting,
    /// th06c's `禁忌「レーヴァテイン」`, a sword that swings round the boss and then across the whole field
    /// from either edge by turns: waited out at the edge each swing across ends at, and at the full step —
    /// see `orb_bot::cards`.
    Laevateinn,
    /// th06c's `禁弾「過去を刻む時計」`, two clocks of four laser hands at once whose centres cross the field
    /// opposite ways: waited out at the bottom, under the high clock's reach and out of the low one's across
    /// from its centre — see `orb_bot::cards`.
    Clock,
    /// th06c's `秘弾「そして誰もいなくなるか？」`, fire that piles up behind a run that stays anywhere: walked round the
    /// whole field clockwise, a corner at a time — see `orb_bot::cards`.
    AndThenThereWereNone,
    /// Fire that fills the field and piles against its right: waited for left of the middle.
    LeftOfTheMiddle,
    /// Fire that fills the field and piles against its left: waited for right of the middle.
    RightOfTheMiddle,
    /// Fire that streams down across the field from fairies above it to the left and piles into the corner it goes
    /// to: waited for against the left wall, under where it sets off from.
    AgainstTheLeftWall,
    /// Fire that thins out towards the floor and carries a run along it into a corner: waited for on the floor
    /// beside the middle, and dodged without going far from it — see `orb_bot::steer`.
    OnTheFloorByTheMiddle,
    /// Fire that fills the band the rules wait in and leaves another part of the field empty: waited for
    /// wherever on the whole field it is emptiest, away from the walls.
    WhereverItIsEmptiest,
    /// Fire that streams across the field and carries the run with it: walked to the place of the lower field
    /// that will have the most room once the run is there, every bullet carried on along its own step.
    WhereThereWillBeRoom,
    /// Fire gathered and thrown at where the run stands: waited for low in the middle, and stepped aside from
    /// where it will come down as it falls.
    AwayFromTheFire,
    /// th10's `開宴「二拝二拍一拝」`, 120 bullets strewn over the top of the field and thrown at where the run is
    /// as they start down: waited out in the middle and gone aside from as each throw is let go — see
    /// `orb_bot::cards`.
    TwoBowsTwoClapsOneBow,
    /// th10's `源符「厭い川の翡翠」`, rivers of still bullets each let go from the top down straight at the run: gone
    /// through at whichever steady pace the card's own fire, played on, leaves unhit longest — see `orb_bot::cards`.
    JadeOfTheLoathsomeRiver,
    /// th10's `神符「杉で結ぶ古き縁」`, fire aimed at the run in two goes: walked up and down the middle of the field
    /// — see `orb_bot::cards`.
    OldTiesBoundByCedars,
    /// th10's `神具「洩矢の鉄の輪」`, rounds of iron rings let go at where the run is: waited under the boss or
    /// above her, and over to the other as the rings are about to reach it — see `orb_bot::cards`.
    MoriyasIronRings,
    /// th10's `土着神「宝永四年の赤蛙」`, clones of the boss that go straight at the run and lay rice bullets along
    /// the way: walked from one end of the low line to the other, tapping off what comes at it — see
    /// `orb_bot::steer`.
    TheRedFrogOfHoeiFour,
    /// th10's `蛙狩「蛙は口ゆえ蛇に呑まるる」`, knots of bullets laid at rest along a snake's way round the field, each
    /// bursting three ways at one age: gone on at whichever steady pace the knots, played on, leave unhit longest —
    /// see `orb_bot::cards`.
    TheSnakeEatsTheCroakingFrog,
    /// th10's `神符「神が歩かれた御神渡り」`, pillars of bullets at rest laid from the top of the field down at where
    /// the run stood: stepped aside of a pillar before it is laid down to the run — see `orb_bot::cards`.
    OmiwatariThatGodWalked,
    /// th10's `天流「お天水の奇跡」`, bullets that fall to rest across the field and set off at the run: dodged along the
    /// low line wherever the dodging carries the run, and cut back across past the middle before it is at a wall —
    /// see `orb_bot::cards`.
    TheMiracleOfOtensui,
    /// Fire that spreads wider the lower it gets: waited for a third of the way up the field from its floor,
    /// the x the rules' own.
    AThirdOfTheWayUp,
    /// Fire aimed at the run that carries it down to the floor wherever it waits: waited for a third of the way up
    /// the field in its middle, and dodged no lower than a step below that height and no further across than `40`
    /// from the middle — see `orb_bot::steer`.
    AThirdOfTheWayUpAndNoLower,
    /// Fire that piles against one side of the field and carries a run along the floor into a corner: waited for low
    /// in the field's middle, a little above the floor, and dodged no further across than `40` from the middle — see
    /// `orb_bot::steer`.
    LowInTheMiddle,
    /// th11's `恨霊「スプリーンイーター」`, rings of spirits that lay arms of still bullets round the run and close on
    /// where it stood, the arms after them: waited for at the floor in the middle and then in each ring's middle, and
    /// gone round the burst's edge as the arms close — see `orb_bot::cards`.
    SpleenEater,
    /// th11's `恨霊「スプリーンイーター」` the partner's way, Reimu with Yukari: waited for against the nearer side wall, and
    /// gone through the gap to the other one as a ring's spirits close in — see `orb_bot::cards`.
    ThroughTheGap,
    /// th11's `妬符「グリーンアイドモンスター」`, a monster that follows the run and lays a trail of still bullets behind
    /// it: waited for under the boss, led straight up under her and brought back down beside the trail — see
    /// `orb_bot::cards`.
    GreenEyedMonster,
    /// th10's `「諏訪大戦　～ 土着神話 vs 中央神話」`, slow bullets piling at the floor with streams through them, its
    /// boss's life held to its end: up and down the left wall by the card's own frames, no further out from it than
    /// `72` and no lower than the low line, and up the middle for its last frames — see `orb_bot::steer`.
    GreatSuwaWar,
    /// th10's `土着神「手長足長さま」`, arms swept across the field and a slow ring out of the boss each 360 frames: up
    /// and down the middle by the card's own frames, off the ring's way down as it comes — see `orb_bot::steer`.
    TenagaAshinaga,
    /// A boss whose fire thins out the further below her it is: waited for at her own x, the height the
    /// rules' own.
    UnderTheBoss,
    /// A fall of fire that swings from side to side, landing in one bottom corner and then the other: waited for
    /// at the low line on the other side of the middle from where it lands.
    AwayFromWhereTheFallLands,
    /// th11's `核熱「ニュークリアフュージョン」`, rings of eight suns born above the boss and flung out across the whole
    /// field, each ring turned its own way: waited out low on the field between the two suns of the ring coming that
    /// leave the gap nearest straight down — see `orb_bot::cards`.
    BetweenTheSuns,
    /// th11's `爆符「メガフレア」`, flares born across the top of the field that fall straight down and shrink as they
    /// fall: waited out on the floor, across from where the flares about to land will land — see `orb_bot::cards`.
    BetweenTheFlares,
    /// th11's `反応「妖怪ポリグラフ」`, a star of eight rays the boss lays still bullets along, out to as far as the
    /// run is from her and turning there: gone round her between two of the rays as they turn — see `orb_bot::steer`.
    YoukaiPolygraph,
    /// th11's `「嫌われ者のフィロソフィ」`, columns of big bullets across the whole field and roses coming up round the
    /// boss as she goes round its edge: waited for at the top right of her round through its blue stage and then just off
    /// its middle away from her, dodged no further from there than `40` — see `orb_bot::cards`.
    PhilosophyOfTheHated,
    /// th11's `「サブタレイニアンローズ」`, rings of bullets going out from the boss with roses going round between
    /// them: waited for half as far from the boss as the floor is, round her out of the way of the rose on the ring
    /// coming, dodged no further from there than `40` — see `orb_bot::cards`.
    SubterraneanRose,
    /// th11's `復燃「恋の埋火」`, hearts trailing bullets that come back off every wall: waited for on the low line in the
    /// field's middle, at the place across the hearts come onto last and nearer under the boss, dodged no further from
    /// there than `40` — see `orb_bot::cards`.
    EmbersOfLove,
    /// th11's `深層「無意識の遺伝子」`, a boss going from corner to corner laying bullets along her way: waited for across
    /// the field from her, on the low line while she is high and up while she is low, dodged no further from there
    /// than `40` — see `orb_bot::cards`.
    GeneticsOfTheSubconscious,
    /// th11's `神徳「五穀豊穣ライスシャワー」`, rice falling over the field and pellets scattered from the boss: waited for
    /// under her a third of the way up and dodged in small steps either way no further from there than `40` — see
    /// `orb_bot::steer`.
    RiceShower,
    /// The stage after th11's Extra midboss, rows of fairies coming in from either side and laying streams of rice:
    /// waited out against the left wall through the first set, along the low line to the right wall, straight up it
    /// through the last set and back to the bottom left, twice over, and to the rules' own from the columns firing three
    /// ways — see `orb_bot::cards`.
    FromTheLeftWallToTheTopRight,
    /// th11's Extra boss's fire after `反応「妖怪ポリグラフ」`, arms of bullets wound out from her lying as long rows by the
    /// floor: waited out on the floor left of the middle under the place whose next row is furthest above, at the whole
    /// step — see `orb_bot::cards`.
    BetweenTheLongRows,
    /// The stage before th11's Extra midboss: waited out on the floor under the red fairies, the lowest one going up
    /// and then the side of those gathered at the top that has more of them, and on that side after, dodged in small
    /// steps no further from there than `40` — see `orb_bot::cards`.
    WhereTheRedFairiesComeOut,
    /// th11's `奇跡「ミラクルフルーツ」`, fruits bursting into rows of rice: waited out where the run is across a little
    /// above the floor, and on the floor while rice is coming down on it — see `orb_bot::cards`.
    MiracleFruit,
    /// th11's `本能「イドの解放」`, hearts thrown out from the boss along curves: waited out in front of her a little below
    /// the middle of the field — see `orb_bot::cards`.
    ReleaseOfTheId,
    /// th11's `花咲爺「華やかなる仁者への嫉妬」`, big bullets aimed at the run laying lines of petals: waited out going round
    /// the boss clockwise, over her and down again — see `orb_bot::cards`.
    JealousyOfTheKind,
    /// th11's `「死灰復燃」`, bodies coming at the run firing as they come: waited out under the boss, and up to the top
    /// as one comes near and down again as the next does — see `orb_bot::cards`.
    Rekindled,
    /// th11's `焔星「フィクストスター」`, rings of stars about suns going round the boss: waited out in front of her at the
    /// height between the rings — see `orb_bot::cards`.
    FixedStar,
    /// th11's `「地獄の人工太陽」`, a sun at the field's middle sending rings out and pulling them and the run back in:
    /// waited out on the floor under it, away from its pull — see `orb_bot::cards`.
    ArtificialSun,
    /// th11's stage 5 after its midboss, yin-yangs thrown at the run: waited out on the floor streaming across from
    /// one side to the other and never back — see `orb_bot::cards`.
    AcrossTheYinYangs,
    /// th11's `贖罪「旧地獄の針山」`, spirits going round in circles sent out in a fan of seven: waited out on the floor
    /// in the gap between the circles' ways nearest the run — see `orb_bot::cards`.
    BetweenTheSpinningSpirits,
    /// th11's stage 5 after its yin-yangs, spirits aimed at the run and spirits firing back: waited out a little below
    /// the middle of the field — see `orb_bot::cards`.
    AfterTheYinYangs,
}

impl Dodge {
    /// The name a recipe gives it, which is what the log says it by too.
    pub fn name(self) -> &'static str {
        match self {
            Dodge::BehindTheLaser => "behind_the_laser",
            Dodge::OutOfTheFollowingLaser => "out_of_the_following_laser",
            Dodge::InTheMiddle => "in_the_middle",
            Dodge::VisionaryTuning => "visionary_tuning",
            Dodge::RoundTheField => "round_the_field",
            Dodge::RoundASmallSquare => "round_a_small_square",
            Dodge::SideToSide => "side_to_side",
            Dodge::WithoutTheItems => "without_the_items",
            Dodge::Focused => "focused",
            Dodge::ImperishableShooting => "imperishable_shooting",
            Dodge::Laevateinn => "laevateinn",
            Dodge::Clock => "clock",
            Dodge::AndThenThereWereNone => "and_then_there_were_none",
            Dodge::LeftOfTheMiddle => "left_of_the_middle",
            Dodge::RightOfTheMiddle => "right_of_the_middle",
            Dodge::AgainstTheLeftWall => "against_the_left_wall",
            Dodge::OnTheFloorByTheMiddle => "on_the_floor_by_the_middle",
            Dodge::WhereverItIsEmptiest => "wherever_it_is_emptiest",
            Dodge::WhereThereWillBeRoom => "where_there_will_be_room",
            Dodge::AwayFromTheFire => "away_from_the_fire",
            Dodge::TwoBowsTwoClapsOneBow => "two_bows_two_claps_one_bow",
            Dodge::JadeOfTheLoathsomeRiver => "jade_of_the_loathsome_river",
            Dodge::OldTiesBoundByCedars => "old_ties_bound_by_cedars",
            Dodge::MoriyasIronRings => "moriyas_iron_rings",
            Dodge::TheRedFrogOfHoeiFour => "the_red_frog_of_hoei_four",
            Dodge::TheSnakeEatsTheCroakingFrog => "the_snake_eats_the_croaking_frog",
            Dodge::OmiwatariThatGodWalked => "omiwatari_that_god_walked",
            Dodge::TheMiracleOfOtensui => "the_miracle_of_otensui",
            Dodge::AThirdOfTheWayUp => "a_third_of_the_way_up",
            Dodge::AThirdOfTheWayUpAndNoLower => "a_third_of_the_way_up_and_no_lower",
            Dodge::LowInTheMiddle => "low_in_the_middle",
            Dodge::SpleenEater => "spleen_eater",
            Dodge::ThroughTheGap => "through_the_gap",
            Dodge::GreenEyedMonster => "green_eyed_monster",
            Dodge::GreatSuwaWar => "great_suwa_war",
            Dodge::TenagaAshinaga => "tenaga_ashinaga",
            Dodge::UnderTheBoss => "under_the_boss",
            Dodge::AwayFromWhereTheFallLands => "away_from_where_the_fall_lands",
            Dodge::BetweenTheSuns => "between_the_suns",
            Dodge::BetweenTheFlares => "between_the_flares",
            Dodge::YoukaiPolygraph => "youkai_polygraph",
            Dodge::PhilosophyOfTheHated => "philosophy_of_the_hated",
            Dodge::SubterraneanRose => "subterranean_rose",
            Dodge::EmbersOfLove => "embers_of_love",
            Dodge::GeneticsOfTheSubconscious => "genetics_of_the_subconscious",
            Dodge::RiceShower => "rice_shower",
            Dodge::FromTheLeftWallToTheTopRight => "from_the_left_wall_to_the_top_right",
            Dodge::BetweenTheLongRows => "between_the_long_rows",
            Dodge::WhereTheRedFairiesComeOut => "where_the_red_fairies_come_out",
            Dodge::MiracleFruit => "miracle_fruit",
            Dodge::ReleaseOfTheId => "release_of_the_id",
            Dodge::JealousyOfTheKind => "jealousy_of_the_kind",
            Dodge::Rekindled => "rekindled",
            Dodge::FixedStar => "fixed_star",
            Dodge::ArtificialSun => "artificial_sun",
            Dodge::AcrossTheYinYangs => "across_the_yin_yangs",
            Dodge::BetweenTheSpinningSpirits => "between_the_spinning_spirits",
            Dodge::AfterTheYinYangs => "after_the_yin_yangs",
        }
    }
}

/// What a `play` or a `wait` is waiting for: one of these and never two.
///
/// One at a time because a step waits for one thing and the next step waits for the next, which is what puts
/// them in an order. What each of them *means* is the game's — see `Game::settle_the_recipe`.
#[derive(Debug, PartialEq, Eq)]
pub enum Until {
    /// A chapter of a stage being the one the run is in, both counted the way the log counts them.
    Chapter { stage: i32, chapter: u32 },
    /// One of orb's own questions being up, by the name the log gives it.
    Question(String),
    /// **The game's own cursor being on the item at this place in the list**, counted from zero the way the
    /// game counts it.
    ///
    /// Which is how a screen this grammar cannot see the items of is walked: Practice Start is the third of
    /// the title menu's items and every screen after it is another list, so a recipe that counted taps instead
    /// would be a recipe written against one machine's unlocks.
    Cursor(i32),
    /// **That item of that screen**, which is what a list has to be named by: the cursor is one field whatever
    /// screen is up, so waiting for a number alone waits for whichever screen reaches it first. A launch
    /// walking into stage practice put the difficulty on Lunatic that way — the 3 it was waiting for was the
    /// difficulty's fourth item and not the stage's.
    Where { screen: i32, cursor: i32 },
    /// **Or that screen being up, whatever its cursor is on**, which is what a step *waiting* for one wants:
    /// an item is what an aim needs and a wait needs the screen.
    ///
    /// **And a walk of a front end cannot be written without it.** Which screen a launch is on when a recipe
    /// reaches a step is not always the same screen: th06c comes up in a demo whose length is the replay's,
    /// so the title arrives before some launches' first step and long after others'. A recipe that presses
    /// blind then walks a different list every launch — three launches written that way started Lunatic
    /// instead of Normal, the press meant for the demo having chosen the title's first item — where a step
    /// per screen waits for the screen it is about to press at and is over the frame that screen is up.
    ///
    /// The cursor is no use for that: it is one field whatever screen is up, and where it sits on a screen
    /// nobody has pressed at is whatever the game's own score file remembered.
    Screen(i32),
    /// **A boss being up**, which is what the part of a stage before one ends at.
    ///
    /// Practising that part was a thing this grammar could not say: bounded by a count of frames instead, a run
    /// guessed at a minute and ended with the midsection still going. Whether a boss is up is a thing orb reads
    /// already — the log's own `boss=` and the focused step the rules hold under one — so it is a thing to wait
    /// for.
    Boss,
    /// **A dialogue being up**, which is what a boss's fight begins after.
    ///
    /// The one thing in a stage a count of frames cannot reach: the rules press past a line as fast as the game
    /// takes presses, so a conversation is up for a handful of frames and shots at hundred-frame intervals
    /// through a midsection walked either side of it.
    Dialogue,
    /// **The game's own name entry being up**, which is the one thing a `play` step otherwise runs straight
    /// past: the entry opens inside the scene a run is played in and the run's own end is past it, so a step
    /// that waited for the run to be over would be waiting for the entry to have gone. Every frame in between
    /// is one the rules are asked for a word on, and the word they hold is a shot — which types whichever
    /// character the entry's cursor starts on, eight times over. See `Game::typing_a_name`.
    Name,
    /// **A spell card running**, which is the frames the plate its name is on is drawn over.
    ///
    /// Named this way rather than by the chapter it falls in: which chapter of a stage is a card is the game's
    /// own numbering, so a step naming one is written against a reading of one afternoon's logs.
    Card,
    /// **The plate a card's name is shown on being drawn**, which is what a picture of that name waits for and
    /// is not the same frame as the card starting: the card starts and the plate is put up after it.
    CardName,
    /// **That card running**, by the number the game gives it — the index of its own record in the score
    /// file, which is what `spell=` in a launch's state lines says.
    ///
    /// **Which is what a told place is bracketed by where a chapter number would move under it.** A stage's
    /// chapters are counted through its midstage table as well as its fight, so every boundary that table
    /// gains puts every chapter of the fight after it one further on: th08's table went from nothing to rows
    /// for six stages in one pass, and the four brackets its whole-run recipes had been written with named
    /// chapters that were then other chapters. The card is the card whatever the table holds.
    Spell(u32),
    /// Or that many frames having gone by.
    Frames(u32),
}

/// A step as it is *written*, which is a map whose keys say which of the four it is.
///
/// **Loose here and strict out of [`read`]**, rather than an enum the reader could take straight: an
/// externally tagged enum is a YAML *tag* to `serde_yaml_ng` — `- !press {…}` — and a recipe is written by
/// hand, so the file gets the plain keys and this pays for them by being checked once.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    press: Option<Vec<String>>,
    frames: Option<u32>,
    /// A `press`'s own, which is what turns it into a tap — beside `frames` rather than in place of it, those
    /// being how long each tap holds.
    until: Option<WrittenUntil>,
    /// How many frames a step is worth waiting through, for one whose `until` may never happen.
    ///
    /// **Which is what stops a recipe holding a launch for ever.** A `play_until` a stage the run died before
    /// reaching waits on a screen the rules do not press, and a launch nobody is watching sits there until
    /// somebody closes it by hand — so a step that can go wrong that way says how long it is worth waiting
    /// through, and the walker says in the log which of the two ended it.
    give_up_after: Option<u32>,
    play_until: Option<WrittenUntil>,
    wait_until: Option<WrittenUntil>,
    leave: Option<bool>,
    quit: Option<bool>,
    /// What the picture is to be called — see [`Step::Screenshot`].
    screenshot: Option<String>,
    /// And the pause menu's own ask, which orb answers with a question — see [`Step::StartOver`].
    start_over: Option<bool>,
    /// And the chance of being hit on purpose in a chapter — see [`Step::TakeHits`].
    take_hits: Option<WrittenHits>,
    /// And whether the rules may bomb — see [`Step::Bombs`].
    bombs: Option<bool>,
    /// And where the rules are told to stand — see [`Step::StandAt`].
    stand_at: Option<WrittenStand>,
    /// And a way of dodging one card — see [`Step::Dodge`].
    dodge: Option<WrittenDodge>,
    /// And an item of one of orb's own questions, taken outright — see [`Step::Answer`].
    answer: Option<WrittenAnswer>,
    /// And the name a run's ranking is registered under — see [`Step::Name`].
    name: Option<String>,
    /// And a player's own path through one card — see [`Step::Follow`].
    follow: Option<WrittenFollow>,
}

/// `follow: { spell: 107, every: 4, path: [[68.2, 432.0], [68.2, 432.0], …] }` — the card, how many of its frames
/// apart the places are, and the places, each `[x, y]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenFollow {
    spell: u32,
    every: u32,
    path: Vec<(f32, f32)>,
}

/// `dodge: { with: behind_the_laser, spell: 198 }` — the way, by its name, and the card it is for — or
/// `dodge: { with: in_the_middle, after_spell: 104 }`, the card the fire it is for comes after, or
/// `dodge: { with: focused, stage: 4, from: 3600, to: 5400 }`, the stretch of a stage it is for, or
/// `dodge: { with: focused, stage: 4, chapter: 3 }`, the chapter of one, `from` and `to` counted from its start.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenDodge {
    with: Dodge,
    spell: Option<u32>,
    after_spell: Option<u32>,
    stage: Option<i32>,
    chapter: Option<u32>,
    from: Option<u32>,
    to: Option<u32>,
}

/// The frames of a chapter a step is for: the whole of it with neither end written, a stretch of its own with
/// both, and nothing a step can be read as with one.
fn the_frames_of_a_chapter(from: Option<u32>, to: Option<u32>) -> Option<Option<Range<u32>>> {
    match (from, to) {
        (None, None) => Some(None),
        (Some(from), Some(to)) if from < to => Some(Some(from..to)),
        _ => None,
    }
}

/// `stand_at: { x: 40 }` or `stand_at: { away_from_the_boss: 120 }`, a `y` beside either or on its own, and
/// `stand_at: {}` to free the rules of the place — and a `spell` beside any of them for a place one card's
/// alone.
///
/// **A `y` on its own leaves the x to the rules' own errand**, which is what an attack whose ground is a
/// height and not a place wants: a mesh of bullets that is harmless until it turns solid is stood in from
/// above the band, and where along it the run stands is where the gap is. What that does not buy is the
/// sides — the dodging can still be pushed into a corner, and a launch of th08's stage 5 card died six
/// times at `8.0,432.0` with the height told and the x free.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenStand {
    x: Option<f32>,
    y: Option<f32>,
    away_from_the_boss: Option<f32>,
    spell: Option<u32>,
    stage: Option<i32>,
    chapter: Option<u32>,
    from: Option<u32>,
    to: Option<u32>,
}

/// The question a step answers and which of its items.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenAnswer {
    /// By the name the log gives it, the same names an `until` waits for one by.
    question: String,
    /// Its place in the list on the screen, counted from nought. Nothing said is the first, which is what a
    /// press would have taken.
    item: Option<usize>,
}

/// The two numbers a `take_hits` is written with.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenHits {
    chance: f32,
    /// **Nothing said is a chance that does not change**, which is what a reader of one number expects of it.
    /// A chance of one that never changes is a chapter hit in for as long as the fire keeps arriving, and that
    /// is a recipe saying so rather than a mistake this can tell from one.
    times_each_retry: Option<f32>,
    /// And nothing said is a run that always takes the chapter again, which is what every recipe written
    /// before this got and what a bot that never gave up did.
    gives_up: Option<f32>,
}

/// One of those two numbers, or what is wrong with it: a chance is a chance, and a factor a chance is
/// multiplied by every retry is one as well — above one it climbs, which is a chapter that gets likelier to be
/// died in the more often it has been.
fn a_chance(named: &str, chance: f32, at: usize) -> Result<f32, String> {
    match (0.0..=1.0).contains(&chance) {
        true => Ok(chance),
        false => Err(format!(
            "step {at} says {named} of {chance}, which is not a chance"
        )),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenUntil {
    stage: Option<i32>,
    chapter: Option<u32>,
    question: Option<String>,
    screen: Option<i32>,
    cursor: Option<i32>,
    boss: Option<bool>,
    dialogue: Option<bool>,
    card: Option<bool>,
    card_name: Option<bool>,
    name: Option<bool>,
    frames: Option<u32>,
    spell: Option<u32>,
}

impl WrittenUntil {
    /// **The conditions written as a bare `true`**: the word each is written under, whether this step wrote it,
    /// what it means, and what a step that wrote `false` should have waited for instead.
    ///
    /// A list rather than four more places in the tuple below. Each of these is one field that is either `true`
    /// or absent, so an arm per combination of them grows by a factor for every condition added — and the
    /// `None`s a new one puts into every arm already there are that many chances to put one in the wrong place.
    fn flags(&self) -> [(&'static str, Option<bool>, Until, &'static str); 5] {
        [
            // **Only `true` throughout.** What a step could do with `boss: false` is wait for a fight to be
            // over, and a fight ends by the stage ending — which is a stage and a chapter to wait for and
            // already written. The others are refused for the same reason: what follows a conversation is the
            // fight, and what follows a card is the rest of the fight.
            (
                "boss",
                self.boss,
                Until::Boss,
                "a stage and a chapter to wait for",
            ),
            (
                "dialogue",
                self.dialogue,
                Until::Dialogue,
                "a boss being up to wait for",
            ),
            (
                "card",
                self.card,
                Until::Card,
                "most of a fight, and a fight is a boss being up to wait for",
            ),
            (
                "card_name",
                self.card_name,
                Until::CardName,
                "most of a card, and a card is a card running to wait for",
            ),
            (
                "name",
                self.name,
                Until::Name,
                "the run being over, which is what a play step ends at anyway",
            ),
        ]
    }

    fn read(&self, at: usize) -> Result<Until, String> {
        let mut asked = None;
        for (word, written, until, instead) in self.flags() {
            match written {
                Some(false) => {
                    return Err(format!("step {at} waits for no {word}, which is {instead}"));
                }
                Some(true) if asked.is_some() => return Err(Self::one_of_them(at)),
                Some(true) => asked = Some(until),
                None => {}
            }
        }
        let shaped = (
            self.stage,
            self.chapter,
            &self.question,
            self.screen,
            self.cursor,
            self.frames,
            self.spell,
        );
        // **And nothing beside it**, a flag being a whole condition: anything written with one is a step
        // waiting for two things.
        if let Some(until) = asked {
            return match shaped {
                (None, None, None, None, None, None, None) => Ok(until),
                _ => Err(Self::one_of_them(at)),
            };
        }
        match shaped {
            (Some(stage), Some(chapter), None, None, None, None, None) => {
                Ok(Until::Chapter { stage, chapter })
            }
            (None, None, Some(question), None, None, None, None) => {
                Ok(Until::Question(question.clone()))
            }
            (None, None, None, Some(screen), Some(cursor), None, None) => {
                Ok(Until::Where { screen, cursor })
            }
            (None, None, None, Some(screen), None, None, None) => Ok(Until::Screen(screen)),
            (None, None, None, None, Some(cursor), None, None) => Ok(Until::Cursor(cursor)),
            (None, None, None, None, None, Some(0), None) => {
                Err(format!("step {at} is over before it starts"))
            }
            (None, None, None, None, None, Some(frames), None) => Ok(Until::Frames(frames)),
            (None, None, None, None, None, None, Some(spell)) => Ok(Until::Spell(spell)),
            _ => Err(Self::one_of_them(at)),
        }
    }

    fn one_of_them(at: usize) -> String {
        format!(
            "step {at} waits for a stage and a chapter, a question, an item the cursor is on with \
             or without the screen it is a list of, a boss being up, a dialogue being up, a card \
             running, a card's name being shown, a card by its number, or a count of frames, and \
             for one of those"
        )
    }
}

/// A bound checked the way every other count of frames in here is: nothing to wait through is a step that
/// never happens.
fn gives_up(frames: Option<u32>, at: usize) -> Result<Option<u32>, String> {
    match frames {
        Some(0) => Err(format!("step {at} gives up before it starts")),
        held => Ok(held),
    }
}

/// Reads one, or says what is wrong with it.
///
/// A whole-list parse and `deny_unknown_fields` throughout, for the resume file's reason: a key nothing knows
/// is a recipe written for something else, and carrying out half of one is worse than carrying out none. And
/// every step checked before any is handed back, for the same reason.
pub fn read(text: &str) -> Result<Vec<Step>, String> {
    let written: Vec<Written> = serde_yaml_ng::from_str(text).map_err(|why| why.to_string())?;
    let mut steps = Vec::with_capacity(written.len());
    for (index, step) in written.iter().enumerate() {
        let at = index + 1;
        let step = match step {
            Written {
                press: Some(buttons),
                frames,
                until,
                give_up_after,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => match frames {
                None => return Err(format!("step {at} presses without saying for how long")),
                Some(0) => return Err(format!("step {at} is over before it starts")),
                Some(frames) => Step::Press {
                    buttons: buttons.clone(),
                    frames: *frames,
                    until: until.as_ref().map(|until| until.read(at)).transpose()?,
                    give_up_after: gives_up(*give_up_after, at)?,
                },
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after,
                play_until: Some(until),
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::Play {
                until: until.read(at)?,
                give_up_after: gives_up(*give_up_after, at)?,
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after,
                play_until: None,
                wait_until: Some(until),
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::Wait {
                until: until.read(at)?,
                give_up_after: gives_up(*give_up_after, at)?,
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: Some(true),
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::Leave,
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: Some(true),
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::Quit,
            Written {
                press: None,
                frames,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: Some(named),
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => match named.trim() {
                "" => return Err(format!("step {at} asks for a picture of nothing")),
                named => Step::Screenshot {
                    named: named.to_owned(),
                    // **Nothing said is the frame the step begins on**, which is what every picture asked for
                    // before this was and what a screen already up wants.
                    frames: frames.unwrap_or(0),
                },
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: Some(true),
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::StartOver,
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: Some(hits),
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::TakeHits {
                chance: a_chance("a chance", hits.chance, at)?,
                // **Nothing said is a chance that does not change** — see [`WrittenHits`].
                times_each_retry: a_chance(
                    "a chance each retry",
                    hits.times_each_retry.unwrap_or(1.0),
                    at,
                )?,
                gives_up: a_chance("a chance of giving up", hits.gives_up.unwrap_or(0.0), at)?,
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: Some(allowed),
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => Step::Bombs { allowed: *allowed },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: Some(stand),
                dodge: None,
                answer: None,
                name: None,
                follow: None,
            } => {
                if stand.x.is_some() && stand.away_from_the_boss.is_some() {
                    return Err(format!(
                        "step {at} tells the rules an x and a distance from the boss both, which is two \
                         places"
                    ));
                }
                let no_place =
                    stand.x.is_none() && stand.y.is_none() && stand.away_from_the_boss.is_none();
                if let Some(spell) = stand.spell
                    && no_place
                {
                    return Err(format!(
                        "step {at} gives card {spell} no place to stand, which is the place written \
                         without a card standing through it anyway"
                    ));
                }
                match (
                    stand.spell,
                    stand.stage,
                    stand.chapter,
                    stand.from,
                    stand.to,
                ) {
                    (spell, None, None, None, None) => Step::StandAt {
                        x: stand.x,
                        y: stand.y,
                        away_from_the_boss: stand.away_from_the_boss,
                        spell,
                    },
                    (None, Some(stage), None, Some(from), Some(to)) if from < to && !no_place => {
                        Step::StandAtThroughAStretch {
                            x: stand.x,
                            y: stand.y,
                            away_from_the_boss: stand.away_from_the_boss,
                            stage,
                            frames: from..to,
                        }
                    }
                    (None, Some(stage), Some(chapter), from, to)
                        if !no_place && let Some(frames) = the_frames_of_a_chapter(from, to) =>
                    {
                        Step::StandAtThroughAChapter {
                            x: stand.x,
                            y: stand.y,
                            away_from_the_boss: stand.away_from_the_boss,
                            stage,
                            chapter,
                            frames,
                        }
                    }
                    _ => {
                        return Err(format!(
                            "step {at} tells the rules a place for neither one card, a stretch of one \
                             stage nor a chapter of one: `spell`, `stage` with `from` and `to`, or \
                             `stage` and `chapter` with both or neither, `from` before `to`, and a place \
                             beside any of them"
                        ));
                    }
                }
            }
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: Some(dodge),
                answer: None,
                name: None,
                follow: None,
            } => match *dodge {
                WrittenDodge {
                    with,
                    spell: Some(spell),
                    after_spell: None,
                    stage: None,
                    chapter: None,
                    from: None,
                    to: None,
                } => Step::Dodge { with, spell },
                WrittenDodge {
                    with,
                    spell: None,
                    after_spell: Some(after),
                    stage: None,
                    chapter: None,
                    from: None,
                    to: None,
                } => Step::DodgeAfterACard { with, after },
                WrittenDodge {
                    with,
                    spell: None,
                    after_spell: None,
                    stage: Some(stage),
                    chapter: None,
                    from: Some(from),
                    to: Some(to),
                } if from < to => Step::DodgeAStretch {
                    with,
                    stage,
                    frames: from..to,
                },
                WrittenDodge {
                    with,
                    spell: None,
                    after_spell: None,
                    stage: Some(stage),
                    chapter: Some(chapter),
                    from,
                    to,
                } if let Some(frames) = the_frames_of_a_chapter(from, to) => Step::DodgeAChapter {
                    with,
                    stage,
                    chapter,
                    frames,
                },
                _ => {
                    return Err(format!(
                        "step {at} dodges neither one card, the fire after one, a stretch of one stage nor \
                         a chapter of one: `spell`, `after_spell`, `stage` with `from` and `to`, or `stage` \
                         and `chapter` with both or neither, `from` before `to`"
                    ));
                }
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: Some(answer),
                name: None,
                follow: None,
            } => match answer.question.trim() {
                "" => return Err(format!("step {at} answers a question with no name")),
                question => Step::Answer {
                    question: question.to_owned(),
                    // **Nothing said is the first item**, which is what a press would have taken: a step
                    // that names no item is a step asking for the answer pressing would give.
                    item: answer.item.unwrap_or(0),
                    give_up_after: gives_up(*give_up_after, at)?,
                },
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: Some(typed),
                follow: None,
            } => match typed.trim() {
                "" => return Err(format!("step {at} types a name of nothing")),
                typed => Step::Name {
                    typed: typed.to_owned(),
                    give_up_after: gives_up(*give_up_after, at)?,
                },
            },
            Written {
                press: None,
                frames: None,
                until: None,
                give_up_after: None,
                play_until: None,
                wait_until: None,
                leave: None,
                quit: None,
                screenshot: None,
                start_over: None,
                take_hits: None,
                bombs: None,
                stand_at: None,
                dodge: None,
                answer: None,
                name: None,
                follow: Some(follow),
            } => match (follow.every, follow.path.is_empty()) {
                (0, _) => {
                    return Err(format!(
                        "step {at} follows a path whose places are no frames apart"
                    ));
                }
                (_, true) => return Err(format!("step {at} follows a path with no place on it")),
                (every, false) => Step::Follow {
                    spell: follow.spell,
                    every,
                    path: follow.path.clone(),
                },
            },
            // **Only `true`.** What a step could do with `start_over: false` is nothing: the question is
            // raised or it is not, and a step that raises nothing is a step to leave out.
            Written {
                start_over: Some(false),
                ..
            } => {
                return Err(format!(
                    "step {at} starts the run over as far as `false`, which is not a step"
                ));
            }
            _ => {
                return Err(format!(
                    "step {at} is a press, a play_until, a wait_until, a screenshot, a start_over, \
                     a take_hits, a bombs, a stand_at, a dodge, a follow, an answer, a name, a leave or a \
                     quit, and one of those and not two"
                ));
            }
        };
        steps.push(step);
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::{Dodge, Step, Until, read};

    /// A recipe of every step there is, which is what one looks like — and what the scenario this was
    /// written for is: th07 played into stage 2 as far as its midboss, left, picked up again, and closed.
    #[test]
    fn a_recipe_is_a_list_of_steps_carried_out_in_order() {
        assert_eq!(
            read(
                "- press: [decide]\n  frames: 2\n\
                 - play_until: { stage: 2, chapter: 3 }\n\
                 - leave: true\n\
                 - wait_until: { question: resume }\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                    until: None,
                    give_up_after: None,
                },
                Step::Play {
                    until: Until::Chapter {
                        stage: 2,
                        chapter: 3,
                    },
                    give_up_after: None,
                },
                Step::Leave,
                Step::Wait {
                    until: Until::Question("resume".to_owned()),
                    give_up_after: None,
                },
                Step::Quit,
            ]),
        );
    }

    /// **A picture is asked for by the name it is to be filed under**, and a run of them is what a screen
    /// full of words is checked from: nothing in the log can say whether the words on a screen are the ones
    /// somebody reads.
    #[test]
    fn a_step_can_ask_for_a_picture_of_the_frame() {
        assert_eq!(
            read("- screenshot: the title menu\n- screenshot: orbs-question\n"),
            Ok(vec![
                Step::Screenshot {
                    named: "the title menu".to_owned(),
                    frames: 0,
                },
                Step::Screenshot {
                    named: "orbs-question".to_owned(),
                    frames: 0,
                },
            ]),
        );
    }

    /// **And it can say how long to let the thing it is of be drawn**, which is what a screen and a card's
    /// plate each take: the frame a condition became true is the frame before what it is about is on the
    /// screen.
    #[test]
    fn a_picture_can_say_how_long_to_let_what_it_is_of_be_drawn() {
        assert_eq!(
            read("- screenshot: the card's own plate\n  frames: 30\n"),
            Ok(vec![Step::Screenshot {
                named: "the card's own plate".to_owned(),
                frames: 30,
            }]),
        );
    }

    /// And a picture of nothing is refused, a file with no name being a file nobody can read back.
    #[test]
    fn a_picture_wants_a_name() {
        assert!(read("- screenshot: \"  \"\n").is_err());
    }

    /// **The name a run's ranking is registered under is a step of its own**, and it takes a bound the
    /// way anything waiting for a screen does.
    #[test]
    fn a_step_can_type_the_name_a_ranking_is_registered_under() {
        assert_eq!(
            read("- name: ORB\n- name: A B\n  give_up_after: 600\n"),
            Ok(vec![
                Step::Name {
                    typed: "ORB".to_owned(),
                    give_up_after: None,
                },
                Step::Name {
                    typed: "A B".to_owned(),
                    give_up_after: Some(600),
                },
            ]),
        );
    }

    /// And a name of nothing is refused: a ranking under a field of spaces is the ranking the entry came
    /// up holding, which is what naming one is instead of.
    #[test]
    fn a_name_wants_characters() {
        assert!(read("- name: \"   \"\n").is_err());
    }

    /// **The run asked to start over is a step of its own**, and what it answers with is a press at orb's
    /// question: raising it and answering it are two things, the way a player meets them.
    #[test]
    fn a_step_can_ask_the_run_to_start_over() {
        assert_eq!(
            read(
                "- start_over: true\n\
                 - press: [decide]\n  frames: 2\n  until: { question: retry }\n"
            ),
            Ok(vec![
                Step::StartOver,
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                    until: Some(Until::Question("retry".to_owned())),
                    give_up_after: None,
                },
            ]),
        );
    }

    /// Only `true`: what a step that starts the run over as far as `false` would do is nothing.
    #[test]
    fn starting_over_as_far_as_false_is_refused() {
        assert!(read("- start_over: false\n").is_err());
    }

    /// **A step hands the rules a chance of being hit on purpose in a chapter**, and what that chance is
    /// multiplied by each time the chapter is retried — which is what a run has to do to reach what happens
    /// after a death at all.
    #[test]
    fn a_step_can_hand_the_rules_a_chance_of_being_hit() {
        assert_eq!(
            read("- take_hits: { chance: 0.25, times_each_retry: 0.5, gives_up: 0.1 }\n"),
            Ok(vec![Step::TakeHits {
                chance: 0.25,
                times_each_retry: 0.5,
                gives_up: 0.1,
            }]),
        );
    }

    /// And nothing said about the retries is a chance that does not change, which is what one number on its
    /// own reads as — with a run that always takes the chapter again rather than the title.
    #[test]
    fn a_chance_with_nothing_said_about_the_retries_does_not_change() {
        assert_eq!(
            read("- take_hits: { chance: 0.1 }\n"),
            Ok(vec![Step::TakeHits {
                chance: 0.1,
                times_each_retry: 1.0,
                gives_up: 0.0,
            }]),
        );
    }

    /// **A step can tell the rules where to stand**, and free them of the place again.
    #[test]
    fn a_step_can_tell_the_rules_where_to_stand() {
        assert_eq!(
            read(
                "- stand_at: { x: 40 }\n- stand_at: { away_from_the_boss: 120 }\n\
                 - stand_at: { y: 240 }\n- stand_at: {}\n",
            ),
            Ok(vec![
                Step::StandAt {
                    x: Some(40.0),
                    y: None,
                    away_from_the_boss: None,
                    spell: None
                },
                Step::StandAt {
                    x: None,
                    y: None,
                    away_from_the_boss: Some(120.0),
                    spell: None
                },
                Step::StandAt {
                    x: None,
                    y: Some(240.0),
                    away_from_the_boss: None,
                    spell: None
                },
                Step::StandAt {
                    x: None,
                    y: None,
                    away_from_the_boss: None,
                    spell: None
                },
            ]),
        );
    }

    /// **A step cannot tell the rules an x and a distance from the boss both**, which would be two places.
    #[test]
    fn a_step_cannot_tell_the_rules_two_places_to_stand() {
        assert!(read("- stand_at: { x: 40, away_from_the_boss: 120 }\n").is_err());
    }

    /// **A step can name the way one card is dodged**, by the way's own name and the card's number — and a
    /// name the rules have no way of is refused, as is a way named for no card.
    #[test]
    fn a_step_can_name_the_way_one_card_is_dodged() {
        assert_eq!(
            read("- dodge: { with: behind_the_laser, spell: 198 }\n"),
            Ok(vec![Step::Dodge {
                with: Dodge::BehindTheLaser,
                spell: 198,
            }]),
        );
        assert_eq!(
            read("- dodge: { with: out_of_the_following_laser, spell: 97 }\n"),
            Ok(vec![Step::Dodge {
                with: Dodge::OutOfTheFollowingLaser,
                spell: 97,
            }]),
        );
        for (name, with) in [
            ("in_the_middle", Dodge::InTheMiddle),
            ("visionary_tuning", Dodge::VisionaryTuning),
            ("round_the_field", Dodge::RoundTheField),
            ("round_a_small_square", Dodge::RoundASmallSquare),
            ("side_to_side", Dodge::SideToSide),
            ("without_the_items", Dodge::WithoutTheItems),
            ("focused", Dodge::Focused),
            ("imperishable_shooting", Dodge::ImperishableShooting),
            ("laevateinn", Dodge::Laevateinn),
            ("clock", Dodge::Clock),
            ("and_then_there_were_none", Dodge::AndThenThereWereNone),
            ("left_of_the_middle", Dodge::LeftOfTheMiddle),
            ("right_of_the_middle", Dodge::RightOfTheMiddle),
            ("against_the_left_wall", Dodge::AgainstTheLeftWall),
            ("on_the_floor_by_the_middle", Dodge::OnTheFloorByTheMiddle),
            ("wherever_it_is_emptiest", Dodge::WhereverItIsEmptiest),
            ("where_there_will_be_room", Dodge::WhereThereWillBeRoom),
            ("away_from_the_fire", Dodge::AwayFromTheFire),
            ("two_bows_two_claps_one_bow", Dodge::TwoBowsTwoClapsOneBow),
            (
                "jade_of_the_loathsome_river",
                Dodge::JadeOfTheLoathsomeRiver,
            ),
            ("old_ties_bound_by_cedars", Dodge::OldTiesBoundByCedars),
            ("moriyas_iron_rings", Dodge::MoriyasIronRings),
            ("the_red_frog_of_hoei_four", Dodge::TheRedFrogOfHoeiFour),
            (
                "the_snake_eats_the_croaking_frog",
                Dodge::TheSnakeEatsTheCroakingFrog,
            ),
            ("omiwatari_that_god_walked", Dodge::OmiwatariThatGodWalked),
            ("the_miracle_of_otensui", Dodge::TheMiracleOfOtensui),
            ("a_third_of_the_way_up", Dodge::AThirdOfTheWayUp),
            (
                "a_third_of_the_way_up_and_no_lower",
                Dodge::AThirdOfTheWayUpAndNoLower,
            ),
            ("low_in_the_middle", Dodge::LowInTheMiddle),
            ("spleen_eater", Dodge::SpleenEater),
            ("through_the_gap", Dodge::ThroughTheGap),
            ("green_eyed_monster", Dodge::GreenEyedMonster),
            ("great_suwa_war", Dodge::GreatSuwaWar),
            ("tenaga_ashinaga", Dodge::TenagaAshinaga),
            ("under_the_boss", Dodge::UnderTheBoss),
            (
                "away_from_where_the_fall_lands",
                Dodge::AwayFromWhereTheFallLands,
            ),
            ("between_the_suns", Dodge::BetweenTheSuns),
            ("between_the_flares", Dodge::BetweenTheFlares),
            ("youkai_polygraph", Dodge::YoukaiPolygraph),
            ("philosophy_of_the_hated", Dodge::PhilosophyOfTheHated),
            ("subterranean_rose", Dodge::SubterraneanRose),
            ("embers_of_love", Dodge::EmbersOfLove),
            (
                "genetics_of_the_subconscious",
                Dodge::GeneticsOfTheSubconscious,
            ),
            ("rice_shower", Dodge::RiceShower),
            (
                "from_the_left_wall_to_the_top_right",
                Dodge::FromTheLeftWallToTheTopRight,
            ),
            ("between_the_long_rows", Dodge::BetweenTheLongRows),
            (
                "where_the_red_fairies_come_out",
                Dodge::WhereTheRedFairiesComeOut,
            ),
            ("miracle_fruit", Dodge::MiracleFruit),
            ("release_of_the_id", Dodge::ReleaseOfTheId),
            ("jealousy_of_the_kind", Dodge::JealousyOfTheKind),
            ("rekindled", Dodge::Rekindled),
            ("fixed_star", Dodge::FixedStar),
            ("artificial_sun", Dodge::ArtificialSun),
            ("across_the_yin_yangs", Dodge::AcrossTheYinYangs),
            (
                "between_the_spinning_spirits",
                Dodge::BetweenTheSpinningSpirits,
            ),
            ("after_the_yin_yangs", Dodge::AfterTheYinYangs),
        ] {
            assert_eq!(
                read(&format!("- dodge: {{ with: {name}, spell: 1 }}\n")),
                Ok(vec![Step::Dodge { with, spell: 1 }]),
            );
            assert_eq!(with.name(), name);
        }
        assert!(read("- dodge: { with: standing_still, spell: 198 }\n").is_err());
        assert!(read("- dodge: { with: behind_the_laser }\n").is_err());
    }

    /// **And the way a stretch of one stage is dodged**, by the stage and the frames of it the way is for —
    /// and a stretch with an end missing, ending before it starts or with a card beside it is refused.
    #[test]
    fn a_step_can_name_the_way_a_stretch_of_a_stage_is_dodged() {
        assert_eq!(
            read("- dodge: { with: focused, stage: 4, from: 3600, to: 5400 }\n"),
            Ok(vec![Step::DodgeAStretch {
                with: Dodge::Focused,
                stage: 4,
                frames: 3600..5400,
            }]),
        );
        assert!(read("- dodge: { with: focused, stage: 4, from: 3600 }\n").is_err());
        assert!(read("- dodge: { with: focused, stage: 4, from: 5400, to: 3600 }\n").is_err());
        assert!(
            read("- dodge: { with: focused, spell: 1, stage: 4, from: 3600, to: 5400 }\n").is_err()
        );
    }

    /// **And the way the fire between one card and the next is dodged**, by the card before it — and a card
    /// named both as the one and as the one before is refused, being two fights.
    #[test]
    fn a_step_can_name_the_way_the_fire_after_a_card_is_dodged() {
        assert_eq!(
            read("- dodge: { with: in_the_middle, after_spell: 104 }\n"),
            Ok(vec![Step::DodgeAfterACard {
                with: Dodge::InTheMiddle,
                after: 104,
            }]),
        );
        assert!(read("- dodge: { with: in_the_middle, spell: 104, after_spell: 104 }\n").is_err());
        assert!(
            read("- dodge: { with: focused, after_spell: 104, stage: 4, from: 3600, to: 5400 }\n")
                .is_err()
        );
    }

    /// **And a player's own path through one card**, the card, how many of its frames apart the places are and the
    /// places — and a path whose places are no frames apart, or that has none, is refused, and so is one beside
    /// another kind of step.
    #[test]
    fn a_step_can_have_the_rules_go_along_a_players_path_through_one_card() {
        assert_eq!(
            read("- follow: { spell: 107, every: 4, path: [[68.2, 432.0], [72.2, 432.0]] }\n"),
            Ok(vec![Step::Follow {
                spell: 107,
                every: 4,
                path: vec![(68.2, 432.0), (72.2, 432.0)],
            }]),
        );
        assert!(read("- follow: { spell: 107, every: 0, path: [[68.2, 432.0]] }\n").is_err());
        assert!(read("- follow: { spell: 107, every: 4, path: [] }\n").is_err());
        assert!(
            read("- follow: { spell: 107, every: 4, path: [[68.2, 432.0]] }\n  bombs: false\n")
                .is_err()
        );
    }

    /// **And a place can be one card's**, which is a place with the card's number beside it — and a card with
    /// no place beside it is refused, being no telling at all.
    #[test]
    fn a_step_can_tell_the_rules_where_to_stand_through_one_card() {
        assert_eq!(
            read("- stand_at: { x: 192, y: 260, spell: 105 }\n"),
            Ok(vec![Step::StandAt {
                x: Some(192.0),
                y: Some(260.0),
                away_from_the_boss: None,
                spell: Some(105),
            }]),
        );
        assert!(read("- stand_at: { spell: 105 }\n").is_err());
    }

    /// **And the way one chapter of a stage is dodged**, by the stage and the chapter's number — the whole of it, or
    /// the frames of it counted from its start — and a chapter with no stage, with one end of its frames missing or
    /// with a card beside it is refused.
    #[test]
    fn a_step_can_name_the_way_a_chapter_of_a_stage_is_dodged() {
        assert_eq!(
            read(
                "- dodge: { with: a_third_of_the_way_up, stage: 6, chapter: 9 }\n\
                 - dodge: { with: on_the_floor_by_the_middle, stage: 4, chapter: 3, from: 60, to: 260 }\n"
            ),
            Ok(vec![
                Step::DodgeAChapter {
                    with: Dodge::AThirdOfTheWayUp,
                    stage: 6,
                    chapter: 9,
                    frames: None,
                },
                Step::DodgeAChapter {
                    with: Dodge::OnTheFloorByTheMiddle,
                    stage: 4,
                    chapter: 3,
                    frames: Some(60..260),
                },
            ]),
        );
        assert!(read("- dodge: { with: focused, chapter: 9 }\n").is_err());
        assert!(read("- dodge: { with: focused, stage: 6, chapter: 9, from: 60 }\n").is_err());
        assert!(read("- dodge: { with: focused, spell: 90, stage: 6, chapter: 9 }\n").is_err());
    }

    /// **And a place to stand through one chapter of a stage**, the same way — and one with no place is refused.
    #[test]
    fn a_step_can_tell_the_rules_where_to_stand_through_a_chapter_of_a_stage() {
        assert_eq!(
            read("- stand_at: { y: 432, stage: 7, chapter: 7, from: 300, to: 2613 }\n"),
            Ok(vec![Step::StandAtThroughAChapter {
                x: None,
                y: Some(432.0),
                away_from_the_boss: None,
                stage: 7,
                chapter: 7,
                frames: Some(300..2613),
            }]),
        );
        assert!(read("- stand_at: { stage: 7, chapter: 7 }\n").is_err());
        assert!(read("- stand_at: { y: 432, spell: 1, stage: 7, chapter: 7 }\n").is_err());
    }

    /// **And one stretch of a stage's**, by the stage and its frames the way a stretch is dodged — and a stretch
    /// with a card beside it, an end missing or no place is refused.
    #[test]
    fn a_step_can_tell_the_rules_where_to_stand_through_a_stretch_of_a_stage() {
        assert_eq!(
            read(
                "- stand_at: { away_from_the_boss: 96, y: 300, stage: 4, from: 3600, to: 4660 }\n"
            ),
            Ok(vec![Step::StandAtThroughAStretch {
                x: None,
                y: Some(300.0),
                away_from_the_boss: Some(96.0),
                stage: 4,
                frames: 3600..4660,
            }]),
        );
        assert!(read("- stand_at: { y: 300, stage: 4, from: 3600 }\n").is_err());
        assert!(
            read("- stand_at: { y: 300, spell: 1, stage: 4, from: 3600, to: 4660 }\n").is_err()
        );
        assert!(read("- stand_at: { stage: 4, from: 3600, to: 4660 }\n").is_err());
    }

    /// **A step can forbid the rules the bomb**, and allow it again, which is what a run held to no bomb is
    /// written with.
    #[test]
    fn a_step_can_forbid_the_rules_the_bomb() {
        assert_eq!(
            read("- bombs: false\n- bombs: true\n"),
            Ok(vec![
                Step::Bombs { allowed: false },
                Step::Bombs { allowed: true },
            ]),
        );
    }

    /// **And a chance outside nought to one is refused, both of them.** Above one a chance is not one, and a
    /// factor above one is a chapter that gets likelier to be died in the more often it has been.
    #[test]
    fn a_chance_that_is_not_a_chance_is_refused() {
        assert_eq!(
            read("- take_hits: { chance: 1.5 }\n"),
            Err("step 1 says a chance of 1.5, which is not a chance".to_owned()),
        );
        assert_eq!(
            read("- take_hits: { chance: -0.5 }\n"),
            Err("step 1 says a chance of -0.5, which is not a chance".to_owned()),
        );
        assert_eq!(
            read("- take_hits: { chance: 0.5, times_each_retry: 2 }\n"),
            Err("step 1 says a chance each retry of 2, which is not a chance".to_owned()),
        );
        assert_eq!(
            read("- take_hits: { chance: 0.5, gives_up: 3 }\n"),
            Err("step 1 says a chance of giving up of 3, which is not a chance".to_owned()),
        );
    }

    /// A press with an `until` is one step and not two, and it keeps its frames: those are how long each tap
    /// holds, not how long the step lasts.
    #[test]
    fn a_press_can_be_tapped_until_something_is_reached() {
        assert_eq!(
            read("- press: [decide]\n  frames: 10\n  until: { question: mode }\n"),
            Ok(vec![Step::Press {
                buttons: vec!["decide".to_owned()],
                frames: 10,
                until: Some(Until::Question("mode".to_owned())),
                give_up_after: None,
            }]),
        );
    }

    /// **A press can be tapped until the game's own cursor is on an item**, which is what makes a screen this
    /// grammar cannot see the items of walkable: the item is named by where it sits, and the game keeps that
    /// number itself.
    ///
    /// Which is how stage practice is reached at all — Practice Start is the third of the title menu's items
    /// and each screen after it is another list, so a recipe that could only count taps would be a recipe
    /// written against somebody's unlocks.
    #[test]
    fn a_press_can_be_tapped_until_the_cursor_is_on_an_item() {
        assert_eq!(
            read("- press: [down]\n  frames: 10\n  until: { cursor: 2 }\n"),
            Ok(vec![Step::Press {
                buttons: vec!["down".to_owned()],
                frames: 10,
                until: Some(Until::Cursor(2)),
                give_up_after: None,
            }]),
        );
        // And not with anything else beside it, an `until` being one of the four and not a set of conditions.
        assert!(read("- press: [down]\n  until: { cursor: 2, stage: 1 }\n").is_err());
    }

    /// **And the screen the list belongs to beside the item**, since the cursor is one field whatever screen is
    /// up: a launch waiting for item 3 of the stage list got item 3 of the difficulty list and put the run on
    /// Lunatic.
    #[test]
    fn a_press_can_be_tapped_until_the_cursor_is_on_an_item_of_a_screen() {
        assert_eq!(
            read("- press: [down]\n  frames: 10\n  until: { screen: 11, cursor: 3 }\n"),
            Ok(vec![Step::Press {
                buttons: vec!["down".to_owned()],
                frames: 10,
                until: Some(Until::Where {
                    screen: 11,
                    cursor: 3,
                }),
                give_up_after: None,
            }]),
        );
    }

    /// **And the screen on its own is a step waiting for that screen**, whatever its cursor is on — which
    /// is what a walk of a front end whose screens do not always arrive in the same order needs.
    #[test]
    fn a_step_can_wait_for_a_screen_whatever_its_cursor_is_on() {
        assert_eq!(
            read("- wait_until: { screen: 6 }\n  give_up_after: 900\n"),
            Ok(vec![Step::Wait {
                until: Until::Screen(6),
                give_up_after: Some(900),
            }]),
        );
    }

    /// **A step can say how long it is worth waiting through**, which is what stops a recipe holding a launch
    /// for ever: a `play_until` a stage the run died before reaching waits on a screen the rules do not press.
    #[test]
    fn a_step_can_say_how_long_it_is_worth_waiting_through() {
        assert_eq!(
            read("- play_until: { stage: 2, chapter: 3 }\n  give_up_after: 30000\n"),
            Ok(vec![Step::Play {
                until: Until::Chapter {
                    stage: 2,
                    chapter: 3,
                },
                give_up_after: Some(30000),
            }]),
        );
        assert_eq!(
            read("- wait_until: { question: resume }\n  give_up_after: 0\n"),
            Err("step 1 gives up before it starts".to_owned()),
        );
    }

    /// And a pair of recipes of the shape the resume's own parting is measured with: one that plays to a point
    /// and closes the game there, one that picks the run back up. A recipe itself lives beside the copy of the
    /// install it is played against, the game's own process being what reads one, so nothing in the tree holds
    /// the ones a measurement was actually taken with — which is why this pair is here at all. A grammar this
    /// stopped reading is a measurement nobody can take again, and this test is what would notice.
    ///
    /// **Two launches and no pause menu.** A run wants closing in the middle for a resume to have anything to
    /// pick up, and `quit` closes the game where it stands — where leaving through the pause menu would be a
    /// cursor to walk down a screen this grammar cannot see the items of.
    ///
    /// **And the tap waits for the first chapter and not for the mode question**, which is what a launch had to
    /// say: a tap of `decide` answers that question the frame it comes up, so what it was waiting for was gone
    /// before the frame the walker next looked at — the step never ended, and the rules never got a frame. A
    /// chapter is reached rather than offered, so nothing the tap presses can take it away again.
    /// **And every step that could wait for something that will not happen says how long it is worth waiting
    /// through**, so that a launch nobody is watching closes itself: a run that died before stage 2 leaves orb's
    /// own retry menu up, which the rules do not press, and one that was left there ran for an hour before
    /// anybody noticed.
    #[test]
    fn the_recipes_the_partings_measurement_is_taken_with_read() {
        assert_eq!(
            read(
                "- press: [decide]\n  frames: 10\n  until: { stage: 1, chapter: 1 }\n\
                 \x20 give_up_after: 3600\n\
                 - play_until: { stage: 2, chapter: 3 }\n  give_up_after: 36000\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 10,
                    until: Some(Until::Chapter {
                        stage: 1,
                        chapter: 1,
                    }),
                    give_up_after: Some(3600),
                },
                Step::Play {
                    until: Until::Chapter {
                        stage: 2,
                        chapter: 3,
                    },
                    give_up_after: Some(36000),
                },
                Step::Quit,
            ]),
        );
        assert_eq!(
            read(
                "- wait_until: { question: resume }\n  give_up_after: 1800\n\
                 - press: [decide]\n  frames: 2\n\
                 - play_until: { frames: 600 }\n\
                 - quit: true\n"
            ),
            Ok(vec![
                Step::Wait {
                    until: Until::Question("resume".to_owned()),
                    give_up_after: Some(1800),
                },
                Step::Press {
                    buttons: vec!["decide".to_owned()],
                    frames: 2,
                    until: None,
                    give_up_after: None,
                },
                Step::Play {
                    until: Until::Frames(600),
                    give_up_after: None,
                },
                Step::Quit,
            ]),
        );
    }

    /// **A step that is over before it starts is refused**, rather than skipped: a recipe is carried out a
    /// step at a time and counted in the log, so one whose steps do not all happen is one nobody can hold
    /// against the file it came from.
    #[test]
    fn a_step_that_is_over_before_it_starts_is_refused() {
        assert_eq!(
            read("- press: [shot]\n  frames: 0\n"),
            Err("step 1 is over before it starts".to_owned()),
        );
        assert_eq!(
            read("- press: [shot]\n  frames: 1\n- play_until: { frames: 0 }\n"),
            Err("step 2 is over before it starts".to_owned()),
        );
    }

    /// A press without its frames, which is a step nothing says the length of.
    #[test]
    fn a_press_without_its_frames_is_refused() {
        assert_eq!(
            read("- press: [shot]\n"),
            Err("step 1 presses without saying for how long".to_owned()),
        );
    }

    /// Two steps in one, and a step that is none: both are a recipe nobody can carry out in an order.
    #[test]
    fn a_step_that_is_two_things_or_nothing_is_refused() {
        assert!(read("- press: [shot]\n  frames: 2\n  quit: true\n").is_err());
        assert!(read("- {}\n").is_err());
    }

    /// A dialogue is a thing to wait for, and no dialogue is not: what follows a conversation is the fight, and
    /// that is a boss being up.
    #[test]
    fn a_step_can_wait_for_a_dialogue_and_not_for_none() {
        assert_eq!(
            read("- play_until: { dialogue: true }\n"),
            Ok(vec![Step::Play {
                until: Until::Dialogue,
                give_up_after: None,
            }]),
        );
        assert!(read("- play_until: { dialogue: false }\n").is_err());
    }

    /// And a card is one too, which is what a picture of the plate its name is on has to be asked on.
    #[test]
    fn a_step_can_wait_for_a_card_and_not_for_none() {
        assert_eq!(
            read("- play_until: { card: true }\n"),
            Ok(vec![Step::Play {
                until: Until::Card,
                give_up_after: None,
            }]),
        );
        assert!(read("- play_until: { card: false }\n").is_err());
    }

    /// And the plate that card's name is on is a condition of its own, the card starting and its name being on
    /// the screen being two frames.
    #[test]
    fn a_step_can_wait_for_a_cards_name_and_not_for_none() {
        assert_eq!(
            read("- play_until: { card_name: true }\n"),
            Ok(vec![Step::Play {
                until: Until::CardName,
                give_up_after: None,
            }]),
        );
        assert!(read("- play_until: { card_name: false }\n").is_err());
    }

    /// **And one card by its number**, which is what brackets a told place against a table that moves the
    /// chapters — alone, a card and a chapter being two things to wait for.
    #[test]
    fn a_step_can_wait_for_a_card_by_its_number_and_for_nothing_beside_it() {
        assert_eq!(
            read("- play_until: { spell: 94 }\n"),
            Ok(vec![Step::Play {
                until: Until::Spell(94),
                give_up_after: None,
            }]),
        );
        assert!(read("- play_until: { spell: 94, stage: 5 }\n").is_err());
        assert!(read("- play_until: { spell: 94, card: true }\n").is_err());
    }

    /// And two of them at once is a step waiting for two things, which no step does.
    #[test]
    fn a_wait_for_two_of_the_flags_is_refused() {
        assert!(read("- play_until: { card: true, boss: true }\n").is_err());
        assert!(read("- play_until: { card: true, frames: 30 }\n").is_err());
    }

    /// And a wait for two things at once, or for nothing.
    #[test]
    fn a_wait_for_more_than_one_thing_is_refused() {
        assert!(read("- wait_until: { stage: 2, question: resume }\n").is_err());
        assert!(read("- wait_until: { stage: 2 }\n").is_err());
    }

    /// And a key nothing knows, which is a recipe written for something else.
    #[test]
    fn a_step_with_a_key_nothing_knows_is_refused() {
        assert!(read("- press: [shot]\n  frames: 2\n  hold: yes\n").is_err());
        assert!(read("- dance: [shot]\n").is_err());
    }
}
