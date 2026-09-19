//! Where chapters begin, and the snapshot taken at each one.
//!
//! A stage's chapters are kept from its start up to the one being played, as many
//! of them as [`KEPT_CHAPTERS`], and every one of them is a place the retry menu
//! offers to go back to. Chapter 1 *is* the stage start, so the oldest of them is
//! that.
//!
//! Boundaries are found by watching the game rather than by consulting a script,
//! so difficulty differences need no separate handling: a boss with an extra
//! attack on Lunatic simply produces an extra chapter.
//!
//! A chapter begins at a stage's start, at each attack of a fight — the boss's timer being reset, which a
//! spellcard beginning does too — and when a midboss is beaten, since that hands the stage back and the
//! waves after it want a chapter of their own. Not when a spellcard ends: either the fight is moving on,
//! which the timer says on the same frame, or the boss has just been beaten. Not when the stage's own boss
//! is beaten either: the stage is over, and a chapter whose first act is that the fight is already won has
//! nothing to retry.
//!
//! Dialogue and a pause hold a boundary back, since neither is the run getting anywhere; a bomb and a death
//! do not, though a chapter beginning under either is a poor place to restart from. A boundary the fight
//! really has is worth more than one that is always comfortable, and the way out of a bad start is the
//! chapter before it. What the fight did while a boundary was held back is taken as soon as one is allowed,
//! so nothing is lost to the seconds a bomb or a death lasts.
//!
//! The log names what began each chapter, so one that turns up where none belongs says which signal
//! produced it. Midstage boundaries come from a compiled-in table of script frame numbers, because a
//! stage's waves run on a clock and are reproducible. Boss boundaries are detected as the game runs, so a
//! difficulty with an extra attack gets an extra chapter with no table.
//!
//! **A chapter beginning washes the play field green**, so that where dying will send you back to is
//! something seen rather than a number to read off the status line. Dim enough to read the bullets through
//! and gone inside a sixth of a second, because a chapter begins every few seconds through a fight and the
//! frame one lands on is a frame being dodged on. Not a stage's own start: that is already unmistakable
//! from the title, the music and an empty field, and a wash over its first frame says nothing that was not
//! obvious. Green because the game flashes white itself — a bomb, a boss going down — and a mark that means
//! something orb decided should not look like something the game did; nothing in th06 fills the play field
//! with green. It holds a moment before it fades, since a wash that starts fading at once reads as dim
//! however bright its first frame is. The play field and no further, because that is the part the game
//! repaints every frame: the panel beside it and the border around it are not, so a wash drawn there would
//! never be drawn over and would stay for good. The fade is counted in frames drawn rather than in the
//! game's, which is stopped wherever a step is holding the game still.
//!
//! `boundary_flash: false` leaves the screen with nothing of orb's over the game. Only a run somebody is
//! playing is marked, and a judging pass ([`crate::tuning`]) — never a replay running for anything else: a
//! collecting pass crosses a boundary every few drawn frames, and a green field for twenty minutes is a
//! mark to nobody.
//!
//! **The chapters of the stage are kept from its start up to the one being played**, since every one of
//! them is a place the retry menu offers to go back to. Eight of them at once, which reaches back across a
//! fight — th06's bosses have eight or nine attacks, so the chapter a fight began at is usually still there
//! to be gone back to from the one it is being lost in. When a ninth is due **the oldest above the stage's
//! own start goes**: that one stays whatever else does, being the way out of the whole stage, and what a
//! death wants is the chapters around the one it happened in.
//!
//! What eight cost is measured rather than guessed, and the log is the instrument: a chapter is written down
//! with the size of its own snapshot, and a session over stages 1 to 5 came out at **five to nine regions
//! and 4.2 to 7.2 megabytes** a chapter — so forty to fifty-five megabytes for the eight, against a
//! two-gigabyte address space. Those are the same numbers wherever it runs, what a snapshot covers being the
//! game's own memory. What a boundary costs the *frame* is not: the `snapshot` phase of the `perf:` line is
//! where that is asked, of the machine asking.
//!
//! Going back to a chapter drops the ones after it, the run not having played them yet — and their buffers
//! are kept to be written over rather than freed, because the run is about to play those frames again and
//! reach those boundaries again. Freed, each of those boundaries would have a chapter's worth of fresh pages
//! to fault in between two frames.
//!
//! **They are this stage's chapters only** — a snapshot of an earlier one would name Direct3D textures the
//! game released when it loaded this stage, and reloading them is not enough to make it whole: an `AnmVm`
//! holds its script as a raw pointer into the file's buffer, so a file loaded again at another address
//! leaves every live one of them pointing at nothing — a jump to 0x00000000 on the frame after such a
//! restore.
//!
//! They are kept across a stage transition, which is not the run ending however much it looks like one from
//! outside: the game leaves the gameplay scene for `GAMEMANAGER_REINIT` while it tears the last stage's
//! managers down and builds the next one's. Only leaving the run for good takes them.
//!
//! **Which fight it is comes from the music**, because that is where the game says it: a stage's data names
//! two songs, its own and its boss's, and the second is played for the fight the stage ends with and for
//! nothing else. The answer is watched for as long as the fight lasts and only ever raised — it starts at
//! the fight a stage runs on through, and the first frame the stage's own track is gone it becomes the
//! other for good. Both ends need that: the track changes when the fight begins, which is not always the
//! frame the boss arrives on, and by the frame the boss is beaten the sound may be being taken down again.
//! The defeat is where the answer matters most, a midboss going down being a boundary and the stage's own
//! boss going down being the stage ending.
//!
//! **And where the game names those two songs, the answer is which of them is playing rather than whether
//! the track has changed.** th07 does: its stage data carries both names, so the fight it ends with is the
//! one under the second of them and every other fight is a midboss, whatever was playing when the chapter
//! at the stage's start was taken. That last part is why the two readings are not the same — a stage's
//! first chapter falls frames into the stage, and the song the stage was come into with can still be
//! playing then, which made every fight of that stage a track change and its midboss the boss. th06 keeps
//! the comparison: its track is named by a path its stage data holds and what is playing is a wave file
//! with no name orb can read.

use std::fmt;
use std::ops::Range;
use std::path::PathBuf;

use crate::crash;
use crate::game::{Game, State};
use crate::memtrack;
use crate::profile;
use crate::snapshot::{Audio, Music, Snapshot};
use crate::tuning::{Judged, Tuning, Verdict};
use crate::{detail, log};

/// Which way a boundary is being judged.
pub enum Judgement {
    Better,
    Worse,
    /// Straight out of the table.
    ///
    /// Nothing in play produces this: the key that did was taken away, and what is left is
    /// `Chapters::judge` still knowing how to do it — `Tuning::reject` — and the tests that hold
    /// it to that. Kept rather than deleted because taking a boundary out of the table is a thing
    /// a judging pass has to be able to do, and the missing half is a key to bind, not this.
    ///
    /// So `Tuning::reject` is the one function of that file no e2e test enters, and it stays that way
    /// until there is a key: `orb-e2e`'s `a_chapter_table_collected` takes a boundary out with `DROP`
    /// twice, which is the way a judging pass has.
    #[allow(dead_code)]
    Out,
}

/// A floor on chapter length, in frames. Without it a boss's opening flurry of
/// script transitions would carve out chapters a fraction of a second long.
const MIN_CHAPTER_FRAMES: u32 = 60;

/// How long to let a stage run before snapshotting its start.
///
/// On the frame the game enters a stage none of it exists yet: the frame counter
/// still holds the previous scene's value, and the music is between tracks, with
/// `backgroundMusic` pointing at a freed object. Waiting a few frames means every
/// snapshot describes a stage that is actually loaded.
const STAGE_SETTLE_FRAMES: u32 = 8;

/// How many chapters of a stage there are snapshots of at once, the stage's own start among them.
///
/// **Eight because it reaches back across a fight**: th06's bosses have eight or nine attacks, so the
/// chapter a fight began at is usually still there to be gone back to from the one it is being lost in.
///
/// What that costs is measured rather than guessed, and the log is the instrument: every chapter is
/// written down with the size of its own snapshot — `{} region(s) of {} bytes` in
/// [`begin_chapter`](Chapters::begin_chapter) — and a session over stages 1 to 5 of th06 1.02h came
/// out at **five to nine regions and 4.2 to 7.2 megabytes** a chapter. So eight of them is forty to
/// fifty-five megabytes, which is nothing beside a two-gigabyte address space. The same numbers on any
/// machine: what a snapshot covers is the game's own memory, and the game is the same game.
///
/// What a boundary costs the *frame* is not, and is no number to write down here — it is the
/// `snapshot` phase of the log's own `perf:` line, on the machine being asked.
///
/// A cap all the same, and this is not the number to raise without a reason of its own: a stage
/// divides into a few dozen chapters, and what would be gained past a fight's worth of them is going
/// back further than anybody grinding a chapter reaches for.
const KEPT_CHAPTERS: usize = 8;

/// How much longer to wait for the music to come up before giving up on it.
///
/// A snapshot without the music still rewinds the streaming bookkeeping in the
/// game's memory, which leaves it disagreeing with a sound buffer that has moved
/// on — the short repeating loop. Waiting is much better than that.
const MUSIC_WAIT_FRAMES: u32 = 240;

/// Where midstage boundaries come from.
enum Midstage {
    /// The table baked into `chapters.rs`.
    Table,
    /// Detected as the stage is played, for building that table.
    Tuning(Tuning),
}

pub struct Chapters {
    midstage: Midstage,
    stage: Option<i32>,
    retries: u32,
    /// Describes the chapter now being played.
    mark: Mark,
    /// Where inside this stage the run can be sent back to.
    snapshots: Snapshots,
    seen: Seen,
    /// Set by a restore: the state jumped, so this frame's differences are not
    /// chapter boundaries.
    resync: bool,
    /// The frame count at which the stage just entered becomes snapshottable.
    settling_until: Option<u32>,
    /// Whether a replay playing back counts as a run to track chapters over.
    during_replay: bool,
    /// Whether the run these snapshots belong to is one somebody is playing, rather than a replay
    /// a pass over the table is reading. Settled with the stage's own snapshot, which is where
    /// [`Chapters::tracking`] has just told the two apart, and it stands for as long as the run
    /// does.
    somebody_playing: bool,
    /// The track playing when the stage began. A boss brings its own, and that is
    /// what tells a stage's own boss from a midboss, which keeps the stage's.
    stage_music: Option<u32>,
    /// Where each chapter of this stage has begun, in stage frames, for the stepping
    /// keys to move between.
    ///
    /// Stage frames rather than script frames, which is what the table is written in:
    /// a boss's boundaries are not in any table — they are found as the fight runs —
    /// and the script clock stands still during a fight, so it cannot name them at
    /// all. The stage's own clock counts every update, so it names all of them.
    starts: Vec<(u32, Cause)>,
    /// Which fight is on, where one is: watched for as long as it lasts and only ever
    /// raised, since the track it is read from changes later than the boss arrives and may
    /// be gone again by the frame the boss is beaten.
    fighting: Option<Fight>,
    /// Whether the fight this stage ends with has been won, which is the stage being over with
    /// every chapter of it still there to go back into. What a judging pass is walled by — see
    /// `step` — and the only thing that says so on a game whose transition spends no frame
    /// outside the scene a run is played in.
    won: bool,
}

/// Which of a stage's two kinds of fight is on.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Fight {
    /// The stage runs on past it, so the waves after it are the same stage carrying on.
    Midboss,
    /// The one a stage ends with, which brings the second of the two songs a stage's data
    /// names.
    Stage,
}

struct Checkpoint {
    snapshot: Snapshot,
    mark: Mark,
}

/// The snapshots a stage has: one per chapter, its own start first and the chapter being played last,
/// and behind them the buffers of the chapters a restore has gone back past.
///
/// **Those buffers are kept rather than freed**, because going back is what they are for: a restore
/// drops every chapter after the one it puts back, and the run then plays those frames again and
/// reaches those boundaries again. Freed, each of those boundaries would have a chapter's worth of
/// fresh pages to fault in between two frames — several megabytes, see [`KEPT_CHAPTERS`] for what one
/// measures — which is the cost [`take`] reuses a snapshot's buffers to avoid.
struct Snapshots {
    kept: Vec<Checkpoint>,
    /// How many of `kept` are chapters of the run as it stands. The rest are those buffers.
    depth: usize,
}

impl Snapshots {
    fn new() -> Self {
        Self {
            kept: Vec::new(),
            depth: 0,
        }
    }

    fn depth(&self) -> usize {
        self.depth
    }

    /// The chapter `at` places up from the stage's start, and `None` where the stage has no such
    /// chapter kept.
    fn at(&self, at: usize) -> Option<&Checkpoint> {
        self.kept.get(..self.depth)?.get(at)
    }

    /// Which of them the chapter being played is.
    fn newest(&self) -> Option<usize> {
        self.depth.checked_sub(1)
    }

    /// How many of them are still chapters of the run: a restore keeps the chapter it put back and
    /// everything under it, and a stage beginning keeps none of them. What is left above is the
    /// buffers the note on this struct is about.
    fn keeping(&mut self, chapters: usize) {
        self.depth = chapters.min(self.kept.len());
    }

    /// The buffers for the snapshot about to be taken, where there are any: the newest chapter gone
    /// back past, the oldest above the stage's start once the stage has as many chapters as it keeps,
    /// or nothing at all while it is still filling.
    fn spare(&mut self) -> Option<Checkpoint> {
        if self.depth < self.kept.len() {
            return Some(self.kept.remove(self.depth));
        }
        if self.depth < KEPT_CHAPTERS {
            return None;
        }
        // The stage's own start is the one that stays whatever else goes: it is the way out of the
        // whole stage, and what a death wants is the chapters around the one it happened in.
        self.depth -= 1;
        Some(self.kept.remove(1))
    }

    /// Puts the chapter that snapshot was taken for on top.
    fn push(&mut self, checkpoint: Checkpoint) {
        self.kept.insert(self.depth, checkpoint);
        self.depth += 1;
    }

    /// What the chapter being played was taken with, for the one thing about a chapter that is
    /// settled after it began: the name of the attack, where the game declares a spellcard a frame
    /// or two into the chapter its own timer began.
    fn newest_mark(&mut self) -> Option<&mut Mark> {
        let at = self.newest()?;
        Some(&mut self.kept[at].mark)
    }

    /// Takes the chapter being played out, for a snapshot about to be taken again over its own
    /// buffers and pushed back.
    fn take_newest(&mut self) -> Option<Checkpoint> {
        self.depth = self.newest()?;
        Some(self.kept.remove(self.depth))
    }

    fn forget(&mut self) {
        self.kept.clear();
        self.depth = 0;
    }
}

/// A chapter there is a snapshot to go back to, which is what the retry menu offers.
pub struct Offer {
    /// Which of the stage's snapshots it is, which is what asking for that one back names.
    pub at: usize,
    /// The chapter's own number, for a pass building the table: what a chapter is called there is
    /// settled by boundaries the pass is still judging.
    pub number: u32,
    pub name: Name,
    /// Whether it is the stage's own start.
    pub stage_start: bool,
}

/// Everything about a chapter that lives in `orb` rather than in the game, and
/// so has to be rewound by hand when its snapshot is restored.
#[derive(Clone, Copy)]
struct Mark {
    number: u32,
    started_at: u32,
    /// The newest midstage boundary the chapter has passed, as a script frame.
    ///
    /// A frame rather than an index into the table, because the tuning pass edits
    /// the table underneath it: removing an entry the stage has already gone by
    /// would leave an index pointing one past where it belongs and the next
    /// boundary would be missed.
    midstage_upto: i32,
    /// What began this chapter.
    cause: Cause,
}

impl Mark {
    /// The midstage boundary this chapter began at, where one did. `None` for the
    /// stage's start and for every chapter a boss began: those are settled by the
    /// game as it runs, are in no table, and so are nothing for the judging keys to
    /// act on.
    fn began_at(&self) -> Option<i32> {
        match self.cause {
            Cause::Boundary(frame) => Some(frame),
            _ => None,
        }
    }
}

/// What begins a chapter. Of three kinds where what can be done with one is concerned —
/// the game's own, settled as the run goes and in no table; the table's, found by the
/// detector; and the table's, put there by hand — and told apart more finely than that on
/// screen, because which fight and which sort of attack is what says where in a stage the
/// game is standing.
///
/// A fight's chapters are named by whose fight it is and by whether the attack it begins
/// has a name: a spellcard, or the nonspell between two of them. A boss arriving is one of
/// those too — its first attack starts with it — so there is nothing else to call that.
#[derive(Clone, Copy, PartialEq)]
pub enum Cause {
    StageStart,
    MidbossNonspell,
    MidbossSpell,
    /// The stage's waves again, the midboss having gone down: that hands the stage back,
    /// and what comes next wants a chapter of its own. Named for the part of the stage it
    /// begins rather than for the defeat that ends the fight, which is the same way the
    /// stage's own start is named — the stage's own boss going down is not a boundary at
    /// all, since the stage is over and a chapter whose first act is that the fight is
    /// already won has nothing to retry.
    StageAfterMidboss,
    BossNonspell,
    BossSpell,
    /// A midstage boundary of the table, at this script frame.
    Boundary(i32),
}

impl fmt::Display for Cause {
    /// For the log, where a chapter turning up in the wrong place has to say which signal
    /// produced it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StageStart => f.write_str("stage start"),
            Self::MidbossNonspell => f.write_str("a midboss nonspell"),
            Self::MidbossSpell => f.write_str("a midboss spellcard"),
            Self::StageAfterMidboss => f.write_str("the midboss was beaten"),
            Self::BossNonspell => f.write_str("a boss nonspell"),
            Self::BossSpell => f.write_str("a boss spellcard"),
            Self::Boundary(frame) => write!(f, "tl {frame}"),
        }
    }
}

impl Cause {
    /// For the status line of a pass building the table, where what is wanted is not what the
    /// chapter is but why the chapter changed at all: which signal produced the boundary, and
    /// so whether it is one to keep. The table's own boundaries are named there by their frame
    /// and their verdict instead, so this is only ever asked of the game's.
    pub fn label(self) -> &'static str {
        match self {
            Self::StageStart => "STAGE",
            Self::MidbossNonspell => "MIDBOSS_NONSPELL",
            Self::MidbossSpell => "MIDBOSS_SPELL",
            Self::StageAfterMidboss => "STAGE_AFTER_MIDBOSS",
            Self::BossNonspell => "BOSS_NONSPELL",
            Self::BossSpell => "BOSS_SPELL",
            Self::Boundary(_) => "TABLE",
        }
    }

    /// Which fight a chapter that began for this reason was under, where it was under one at all.
    fn fight(self) -> Option<Fight> {
        match self {
            Self::MidbossNonspell | Self::MidbossSpell => Some(Fight::Midboss),
            Self::BossNonspell | Self::BossSpell => Some(Fight::Stage),
            Self::StageStart | Self::StageAfterMidboss | Self::Boundary(_) => None,
        }
    }
}

/// The part of a stage a chapter belongs to, which is what it is named after.
///
/// *Midstage* for the waves, which is what they are called everywhere else here and in the
/// table the boundaries between them come from. Not "stage", which beside a number reads as
/// the stage's own.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// The waves: the stage's own start, each midstage boundary of the table, and the waves
    /// handed back when a midboss goes down. One run of chapters, because they are all the
    /// same part of the stage — a midboss interrupts the waves rather than starting a new
    /// set of them.
    Midstage,
    MidbossNonspell,
    MidbossSpell,
    BossNonspell,
    BossSpell,
}

impl Kind {
    fn of(cause: Cause) -> Self {
        match cause {
            Cause::StageStart | Cause::StageAfterMidboss | Cause::Boundary(_) => Self::Midstage,
            Cause::MidbossNonspell => Self::MidbossNonspell,
            Cause::MidbossSpell => Self::MidbossSpell,
            Cause::BossNonspell => Self::BossNonspell,
            Cause::BossSpell => Self::BossSpell,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Midstage => "MIDSTAGE",
            Self::MidbossNonspell => "MIDBOSS NONSPELL",
            Self::MidbossSpell => "MIDBOSS SPELL",
            Self::BossNonspell => "BOSS NONSPELL",
            Self::BossSpell => "BOSS SPELL",
        }
    }
}

/// What a chapter is called on screen: which part of the stage it belongs to, and which one
/// of those it is.
///
/// Counted per kind and from the stage's start, because a count of every chapter gone by says
/// nothing about where the game is standing, while `BOSS SPELL 2` is the fight's second
/// spellcard — which is how a fight is talked about, and how a chapter worth grinding is
/// named to somebody else.
pub struct Name {
    kind: Kind,
    index: usize,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.kind.label(), self.index)
    }
}

/// Whether a chapter begins on this frame, and what began it.
enum Due {
    No,
    Yes(Cause),
}

/// Whether this frame is one to keep away from: no place to start a chapter, and nothing to
/// read a chapter's cause from either.
///
/// Dying and bombing are not among them, though a chapter beginning under either is a poor
/// retry point — one kills whoever restores it and the other hands them a cleared screen and
/// two seconds of invulnerability. A boundary the fight really has is worth more than a
/// boundary that is always comfortable, and the way out of a bad start is the chapter before
/// it. What that leans on is being able to go back further than one chapter, which the retry
/// menu offers every chapter the stage has kept for.
///
/// A bomb was guarded against on the strength of a boundary seen where a bomb had cleared a
/// boss's bullets, taken at the time for one the detector had proposed. It cannot have been:
/// the detector asks for no enemies *and* no boss, so a fight is the one place it never
/// speaks. Nothing else was ever pinned on a bomb, and there is nothing here to put back.
/// Should a boundary no attack accounts for turn up under one, the `sync:` lines say what the
/// timer and the spellcard were doing, and the answer is to disbelieve that one signal rather
/// than the whole frame.
fn guarded(state: &State) -> bool {
    state.in_dialogue || state.paused
}

/// The signals a boundary is derived from, as of the previous frame.
#[derive(Clone, Copy, PartialEq)]
struct Seen {
    boss_present: bool,
    boss_timer: Option<i32>,
    spellcard: Option<u32>,
}

impl Chapters {
    pub fn new(game: &dyn Game, tuning: Option<PathBuf>, during_replay: bool) -> Self {
        Self {
            midstage: match tuning {
                Some(path) => Midstage::Tuning(Tuning::new(game, path)),
                None => Midstage::Table,
            },
            stage: None,
            retries: 0,
            mark: Mark {
                number: 0,
                started_at: 0,
                midstage_upto: i32::MIN,
                cause: Cause::StageStart,
            },
            snapshots: Snapshots::new(),
            seen: Seen {
                boss_present: false,
                boss_timer: None,
                spellcard: None,
            },
            resync: false,
            settling_until: None,
            during_replay,
            somebody_playing: false,
            stage_music: None,
            starts: Vec::new(),
            fighting: None,
            won: false,
        }
    }

    /// Whether the fight this stage ends with has been won, which is this stage being over.
    ///
    /// **Read a frame later than it is set**, which is what puts a defeat under a dialogue after that
    /// dialogue: [`Chapters::observe`] keeps what a guarded frame turned away and acts on it once the
    /// guard lifts, so this answers on the first frame that could have carried a chapter.
    pub fn stage_won(&self) -> bool {
        self.won
    }

    pub fn number(&self) -> u32 {
        self.mark.number
    }

    /// **What a stage's own first chapter is called, before it has begun.**
    ///
    /// Which is knowable without it having begun: a stage's start is a `Midstage` chapter and the first of
    /// them, whatever the stage turns out to hold. [`Chapters::name`] answers nothing until the stage has
    /// settled and the music is ready — and a session interrupted inside that wait is one whose run has to be
    /// written down under some name, so this is that name and it is the same one.
    pub fn what_a_stages_start_is_called() -> Name {
        Name {
            kind: Kind::Midstage,
            index: 1,
        }
    }

    /// And whether a stage of this run has begun with its first chapter not yet announced, which is that wait.
    ///
    /// **The stage rather than the chapter**, because what is being asked is whether there is a stage to write
    /// down at all: a frame between two stages has neither, and a frame inside the wait has the stage and no
    /// chapter of it.
    pub fn settling(&self) -> bool {
        self.stage.is_some() && self.settling_until.is_some()
    }

    /// The stage frame the chapter now running began at. With the stage, this is what
    /// says which boundary the game is standing on rather than how many have gone by:
    /// numbering starts again at every stage, so a move between stages keeps the number
    /// and changes this.
    pub fn started_at(&self) -> u32 {
        self.mark.started_at
    }

    /// **Which stage the chapters kept are of**, and `None` before a stage's start has settled into one.
    ///
    /// Which is not always `State::stage`: a game that leaves the gameplay scene to build its next stage
    /// says no stage at all on the frames of the transition — th06c's stage word reads nought there — and
    /// what the chapters are still of is the stage just played.
    pub fn stage(&self) -> Option<i32> {
        self.stage
    }

    /// What began the chapter now running, which is what says which of the three kinds of
    /// boundary the game is standing in: one the game settled as it ran, or one of the
    /// table's — and for those the status line goes on to say whether a person put it there.
    pub fn cause(&self) -> Cause {
        self.mark.cause
    }

    /// What to call the chapter now running, for a run somebody is playing. `None` where there
    /// is no chapter: a menu, or the frames of a stage before its own start has been marked.
    ///
    /// A pass building the table shows [`Cause::label`] instead, which answers a different
    /// question — why the chapter changed here — and is the one worth having while what is
    /// being decided is whether the boundary belongs in the table at all.
    pub fn name(&self) -> Option<Name> {
        (self.mark.number > 0).then(|| self.named(&self.mark))
    }

    /// And what to call the chapter that began at a mark, which is what the chapters kept beside the
    /// one being played are listed by.
    fn named(&self, mark: &Mark) -> Name {
        let kind = Kind::of(mark.cause);
        // Counted out of the chapter starts this stage has recorded rather than kept in a
        // counter of its own, because a restore and a step back put the mark back and then
        // play the same chapters again: a counter would make the second time through the
        // same chapter a different one, while these are the frames they began at.
        //
        // And two starts of one kind closer together than [`MIN_CHAPTER_FRAMES`] are that same chapter
        // twice, which is the shape a chapter played again arrives in: the frame a restore lands on is one
        // nothing is read on — `resync` — so an attack seen at 251 the first time through is seen at 252
        // the second, and both are recorded. Both have to be: the stepping keys read this list for where
        // the stage's boundaries are, and the ones ahead of a restore are still the stage's. Counted
        // straight, th07's own `MIDBOSS SPELL 1` came back from a retry as `MIDBOSS SPELL 2`.
        let mut index = 0;
        let mut last: Option<u32> = None;
        for (at, _) in self
            .starts
            .iter()
            .filter(|(at, cause)| *at <= mark.started_at && Kind::of(*cause) == kind)
        {
            if last.is_none_or(|last| at - last >= MIN_CHAPTER_FRAMES) {
                index += 1;
            }
            last = Some(*at);
        }
        Name { kind, index }
    }

    pub fn retries(&self) -> u32 {
        self.retries
    }

    /// Puts the count back, for a run picked up where an earlier session left it: the rewinds it
    /// had already cost are part of what that run is, and a clear with none of them and a clear
    /// with sixty are not the same clear.
    pub fn carry_retries(&mut self, retries: u32) {
        self.retries = retries;
    }

    pub fn can_retry(&self) -> bool {
        self.snapshots.depth() > 0
    }

    /// The chapters there is a snapshot to go back to, the one being played first: what the retry menu
    /// lists, in the order it lists them.
    ///
    /// Newest first because that is the order they are wanted in: the chapter just lost is what a death
    /// is usually answered with, so it is the item the cursor starts on and each press down goes one
    /// chapter further back.
    pub fn offers(&self) -> Vec<Offer> {
        (0..self.snapshots.depth())
            .rev()
            .filter_map(|at| {
                let mark = self.snapshots.at(at)?.mark;
                Some(Offer {
                    at,
                    number: mark.number,
                    name: self.named(&mark),
                    stage_start: at == 0,
                })
            })
            .collect()
    }

    /// Whether what is being tracked is a run somebody is playing.
    ///
    /// A property of the run and not of the frame, which is what asking about the panel wants: the
    /// frames a run passes through outside the gameplay scene are still that run's — the one a
    /// stage transition is built in, and the one a chapter is put back on.
    pub fn somebody_playing(&self) -> bool {
        self.somebody_playing
    }

    /// Whether this state is a run to track chapters over: one someone is playing,
    /// or a replay when `during_replay` says a replay counts as one. A replay drives
    /// the same gameplay scene, which is what makes it usable to build the midstage
    /// table or to exercise a snapshot repeatedly.
    pub fn tracking(&self, state: &State) -> bool {
        state.in_game || (self.during_replay && state.playing && !state.demo)
    }

    /// The stage frames of the chapters either side of this point, which is where the
    /// stepping keys go. Strictly either side, so that a step lands on a boundary and
    /// the next press moves on rather than staying put — and so that a frame or two
    /// past a boundary steps back onto it, which is what that key is for.
    ///
    /// A boundary judged out of the table is not one of them. It is judged out, not
    /// gone — `chapter_dropped_key` with either key is how it is reached, and taking that
    /// back means being able to get to it — but it begins no chapter, and a step that
    /// stopped at it would be stopping at nothing.
    pub fn previous_start(&self, state: &State) -> Option<u32> {
        self.in_table()
            .filter(|start| *start < state.stage_frames)
            .max()
    }

    /// And the chapter after it. Neither leaves the stage: what lies outside it is the
    /// game's to load, not a snapshot's to put back.
    pub fn next_start(&self, state: &State) -> Option<u32> {
        self.in_table()
            .filter(|start| *start > state.stage_frames)
            .min()
    }

    /// The stage frames of the chapters this stage has begun, minus the ones a boundary
    /// since judged out of the table began.
    fn in_table(&self) -> impl Iterator<Item = u32> {
        self.starts
            .iter()
            .filter(|(_, cause)| self.kept(*cause))
            .map(|(frame, _)| *frame)
    }

    /// Calls the chapter being played something else, everywhere its name is kept: the mark it is being
    /// played under, the record its start is in — which is what the count in its name comes out of and what
    /// [`Chapters::recorded_fight`] answers from — and the mark inside the snapshot it began with.
    ///
    /// That last one because a restore puts the mark back: left as it was, going back to this chapter would
    /// take the name away again and the menu offering it would call it what it arrived as.
    fn rename_chapter(&mut self, named: Cause) {
        self.mark.cause = named;
        if let Ok(at) = self
            .starts
            .binary_search_by_key(&self.mark.started_at, |(at, _)| *at)
        {
            self.starts[at].1 = named;
        }
        if let Some(mark) = self.snapshots.newest_mark() {
            mark.cause = named;
        }
    }

    /// Which fight this stage recorded the chapter holding a frame under, for a frame it has been
    /// through before.
    ///
    /// The newest boundary at or before the frame, a chapter running until the next one begins. A frame
    /// past everything recorded is answered from the chapter it is in all the same: the stage's own boss
    /// going down ends the stage rather than beginning a chapter, so the last thing recorded under a fight
    /// is the attack it is beaten during.
    fn recorded_fight(&self, frame: u32) -> Option<Fight> {
        self.starts
            .iter()
            .rfind(|(at, _)| *at <= frame)
            .and_then(|(_, cause)| cause.fight())
    }

    /// Whether this boundary of the current stage is one somebody put there.
    ///
    /// Asked of the baked table as well as of a tuning pass, because what turns on the answer —
    /// the exemption from the shortest a chapter may be — has to come out the same in play as in
    /// the pass that chose the number. While only a pass could answer it, a stage divided one way
    /// under `--judge` and another way in the run that used the table it wrote.
    fn added_by_hand(&self, game: &dyn Game, frame: i32) -> bool {
        let Some(stage) = self.stage else {
            return false;
        };
        match &self.midstage {
            Midstage::Tuning(tuning) => tuning
                .judged(stage, frame)
                .is_some_and(|judged| judged.by_hand),
            Midstage::Table => game
                .midstage(stage)
                .iter()
                .any(|entry| entry.frame == frame && entry.by_hand),
        }
    }

    /// Whether a chapter's start is still one, which for the table's own boundaries is
    /// whether the table still has it and still carries it.
    ///
    /// Gone counts as not one: refusing a boundary put there by hand takes it out
    /// altogether rather than remembering it as refused, and a chapter start that outlived
    /// the boundary it came from is a place the stepping would stop at for no reason —
    /// `chapter_dropped_key` cannot reach it either, since it is in no table to be found in.
    fn kept(&self, cause: Cause) -> bool {
        let Cause::Boundary(frame) = cause else {
            return true;
        };
        let Midstage::Tuning(tuning) = &self.midstage else {
            return true;
        };
        let Some(stage) = self.stage else { return true };
        tuning
            .judged(stage, frame)
            .is_some_and(|judged| judged.verdict != Verdict::Rejected)
    }

    /// The script frames of the boundaries judged out of the table, either side of where
    /// the game is, which is where `chapter_dropped_key` with a stepping key goes.
    ///
    /// Script frames, and not the stage frames the other stepping works in, because one
    /// judged out begins no chapter: nothing has recorded a stage frame for it, and the
    /// clock the table is written in is the only thing that names it at all.
    pub fn previous_dropped(&self, state: &State) -> Option<i32> {
        self.dropped()
            .filter(|frame| *frame < state.script_frames)
            .max()
    }

    pub fn next_dropped(&self, state: &State) -> Option<i32> {
        self.dropped()
            .filter(|frame| *frame > state.script_frames)
            .min()
    }

    fn dropped(&self) -> impl Iterator<Item = i32> {
        let stage = self.stage;
        let tuning = match &self.midstage {
            Midstage::Tuning(tuning) => Some(tuning),
            Midstage::Table => None,
        };
        stage
            .zip(tuning)
            .into_iter()
            .flat_map(|(stage, tuning)| tuning.rejected(stage))
    }

    /// Which boundary the judging keys act on: the one at the very frame the game is
    /// standing on, and only while it is standing still.
    ///
    /// Only while held, because a frame is a sixtieth of a second: a key pressed while the
    /// game runs would land on whichever frame it happened to reach, which is no way to
    /// aim at a boundary. Only the frame itself, because a chapter judged from anywhere
    /// inside it is a boundary edited from where it cannot be seen — and where the game
    /// stands on a boss's boundary or a stage's start, the table has nothing there and
    /// there is nothing to judge.
    fn judging(&self, state: &State, held: bool) -> Option<(i32, i32)> {
        let Midstage::Tuning(tuning) = &self.midstage else {
            return None;
        };
        let stage = self.stage?;
        let here = state.script_frames;
        (held && tuning.judged(stage, here).is_some()).then_some((stage, here))
    }

    /// What is known about that boundary. `None` where there is none — the stage's
    /// start, a boss appearing, an attack changing — which the game settles as the run
    /// goes and is not anyone's to judge.
    pub fn judged(&self, state: &State, held: bool) -> Option<Judged> {
        let Midstage::Tuning(tuning) = &self.midstage else {
            return None;
        };
        let (stage, boundary) = self.judging(state, held)?;
        tuning.judged(stage, boundary)
    }

    /// The boundary the chapter now running began at, for the status line: it names the
    /// chapter being played for as long as it is being played, where `judged` names only
    /// what a key would change and only while the game is still.
    pub fn chapter_boundary(&self) -> Option<Judged> {
        let Midstage::Tuning(tuning) = &self.midstage else {
            return None;
        };
        tuning.judged(self.stage?, self.mark.began_at()?)
    }

    /// Judges it one step better, or one step worse. Does nothing where there is
    /// nothing of the table's to judge.
    pub fn judge(&mut self, state: &State, held: bool, judgement: Judgement) {
        let Some((stage, boundary)) = self.judging(state, held) else {
            return;
        };
        let Midstage::Tuning(tuning) = &mut self.midstage else {
            return;
        };
        match judgement {
            Judgement::Better => tuning.judge_up(stage, boundary),
            Judgement::Worse => tuning.judge_down(stage, boundary),
            Judgement::Out => tuning.reject(stage, boundary),
        }
    }

    /// Puts a boundary where the game is now, and begins its chapter there.
    ///
    /// A boundary is a chapter's start; those are one thing and adding one has to leave
    /// them one thing. Left to the ordinary path, the crossing of a boundary is noticed on
    /// the update where the script clock has reached it — which for a frame the game is
    /// already standing on is the update after this one, so the chapter would begin a
    /// frame past the number written down. That frame is the number on screen, the frame
    /// the flash goes off on and the frame a step lands on, and the same table read on a
    /// later pass would begin the chapter at the boundary itself: two answers for one
    /// thing, and the one on screen the wrong one.
    ///
    /// So the chapter begins here, and the cursor moves past the boundary with it: what
    /// the next update would otherwise see is a boundary it has not crossed yet.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn add_boundary(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
    ) {
        if !self.note_boundary(state) {
            // Said rather than passed over: a key that does nothing at all is the hardest
            // kind of nothing to look into.
            return log!(
                "tuning: nothing to add tl {} to — stage {} is not being tuned",
                state.script_frames,
                state.stage + 1,
            );
        }
        unsafe {
            self.begin_chapter(
                game,
                state,
                data,
                self_check,
                Cause::Boundary(state.script_frames),
            )
        };
    }

    /// Writes the boundary down and moves the cursor past it: everything about adding one
    /// except the chapter it begins.
    fn note_boundary(&mut self, state: &State) -> bool {
        let Midstage::Tuning(tuning) = &mut self.midstage else {
            return false;
        };
        if !tuning.add(state.stage, state.script_frames) {
            return false;
        }
        self.mark.midstage_upto = state.script_frames;
        true
    }

    /// `None` unless `chapter_tuning` is on.
    pub fn tuning(&mut self) -> Option<&mut Tuning> {
        match &mut self.midstage {
            Midstage::Table => None,
            Midstage::Tuning(tuning) => Some(tuning),
        }
    }

    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn observe(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
    ) {
        let current = Seen::of(state);
        unsafe { self.step(game, state, &current, data, self_check) };
        // A frame the guard turned away is no frame to read a fight from either, so what
        // the fight looked like before it is what the frame after it is compared against.
        // Whatever happened under the guard is then acted on once, on the first frame that
        // can carry a chapter.
        //
        // Tracking it through the guard instead loses every transition that happens under
        // one, and both are seconds long. A bomb between the last two of Cirno's spellcards
        // swallowed the nonspell between them — chapters 9 and 10 of stage 2 at frames 9554
        // and 11088, 1534 frames apart with one bomb spent, and nothing in between — and
        // the player dying as stage 6's midboss went down swallowed the boundary the waves
        // after a midboss get.
        if !guarded(state) {
            self.seen = current;
        }
        self.resync = false;
    }

    unsafe fn step(
        &mut self,
        game: &dyn Game,
        state: &State,
        current: &Seen,
        data: &Range<usize>,
        self_check: bool,
    ) {
        if !self.tracking(state) {
            if self.stage.is_some() {
                // Keep whatever the tuning pass has found, here as well as at a stage's
                // end: this is where a run that is being left goes through.
                self.save_tuning(game);
            }
            // A stage transition leaves the gameplay scene for a moment while the game
            // builds the next stage, and the run is the same run. What is kept of the
            // stages already played is what a step back into one needs, so it goes when
            // the run does and not before.
            if !state.in_run {
                self.forget();
            }
            return;
        }
        // Mid-dialogue or paused is not the run getting anywhere. What the fight did while
        // the guard was up is not lost, though — `observe` holds what it was compared
        // against — so it is acted on as soon as a chapter is allowed again, which for the
        // dialogue a boss arrives with is the frame the fight starts on.
        if guarded(state) {
            return;
        }

        // **A frame between two stages is a frame of neither**, and this is not the frame to give one up on:
        // th06 and th06c both leave the gameplay scene while the next stage is built, and this game's stage
        // word reads nought there — `State::stage` being that word less one — so a frame of the rebuild
        // announces a stage that is no stage. Left to the line below it, that frame dropped everything kept
        // of the stage just played: a `--judge` pass stepping forward off the last boundary of a th06c stage
        // came to rest there and then answered `cannot go back — no stage start kept`, which is the wall
        // being a place a pass loses the stage from rather than one to look around in.
        // **And only where the chapters are of a stage already**: a run that has not been in one has not
        // left one, and a frame before its first stage is built says the same nought — see the wall in
        // `runtime::step`, which is the same distinction.
        if state.stage < 0 && self.stage.is_some() {
            return;
        }

        if self.stage != Some(state.stage) {
            // **Nothing of the stage before this one is this one's**, from the frame the game is in it: no
            // chapter has begun here, so there is no boundary to have been reached, nothing to step to and
            // nothing to go back into. What is kept until the stage's own start settles is otherwise the
            // stage that has gone — and th06 washed the play field green over the opening of every stage
            // for it, the wash a boundary gets going by the *cause* of the chapter the game has come to
            // rest on and that cause being the last fight's of the stage before.
            self.the_stage_was_left();
            // The frame counter restarting is what says the stage has replaced
            // the scene before it, rather than merely being announced.
            if state.stage_frames <= 1 {
                self.stage = Some(state.stage);
                self.settling_until = Some(state.stage_frames + STAGE_SETTLE_FRAMES);
            }
            return;
        }
        if let Some(until) = self.settling_until {
            let settled = state.stage_frames >= until;
            // A game whose snapshots carry no music has nothing here to wait for, and counts as ready —
            // see [`Game::a_snapshot_waits_for_its_music`], which measures what the wait cost th06c.
            let music_ready = !game.a_snapshot_waits_for_its_music() || game.music().is_some();
            if settled && (music_ready || state.stage_frames >= until + MUSIC_WAIT_FRAMES) {
                self.settling_until = None;
                unsafe { self.begin_stage(game, state, data, self_check) };
            }
            return;
        }
        if self.resync {
            return;
        }
        let Due::Yes(cause) = self.due(game, state, current) else {
            return;
        };
        unsafe { self.begin_chapter(game, state, data, self_check, cause) };
    }

    /// Advances the midstage cursor as a side effect: an entry is spent whether
    /// or not it turns out to start a chapter.
    fn due(&mut self, game: &dyn Game, state: &State, current: &Seen) -> Due {
        let elapsed = state.stage_frames.saturating_sub(self.mark.started_at);
        // The tuning pass grows the list it is then read from, so that a stage
        // played twice — which is what stepping back through the chapters does —
        // divides the same way both times.
        if let Midstage::Tuning(tuning) = &mut self.midstage {
            tuning.propose(state, elapsed);
        }
        // The newest of the boundaries passed, not the first: the order a hand-written
        // table happens to be in cannot then leave an entry to fire again next frame.
        let midstage = match &self.midstage {
            Midstage::Table => game
                .midstage(state.stage)
                .iter()
                .map(|entry| entry.frame)
                .filter(|frame| *frame > self.mark.midstage_upto && *frame <= state.script_frames)
                .max(),
            Midstage::Tuning(tuning) => {
                tuning.passed(state.stage, self.mark.midstage_upto, state.script_frames)
            }
        };
        if let Some(frame) = midstage {
            self.mark.midstage_upto = frame;
        }

        // Which fight this is, watched for as long as it lasts and only ever raised: the
        // track changes when the fight the stage ends with begins, which is not always the
        // frame the boss arrives on, and by the frame it is beaten the sound may be being
        // taken down again. So the answer starts at the fight a stage runs on through, and
        // the first frame the stage's own track is gone it becomes the other for good.
        //
        // The music says which fight it is because the stage's data says so: an STD names
        // two songs, the stage's and the boss's, and the game plays the second one for the
        // boss it ends with. What the timeline is doing does not say it — the wait it parks
        // on for a boss, `RunEclTimeline`'s `case 0xc`, is also how stage 3 waits for its
        // *midboss*, which called that fight the stage's own. Neither does anything a boss
        // carries: both games register a midboss and the boss they end with alike, so what says which
        // fight is on is the music, and the stage's own record where it has been through the frame before.
        //
        // The game's own answer where it has one, and that comparison where it has not: what a
        // stage's start was snapshotted under is not always the stage's own song — see
        // `Game::the_fights_own_song_is_playing`.
        let boss_appeared = current.boss_present && !self.seen.boss_present;
        if current.boss_present {
            let its_own_song = game.the_fights_own_song_is_playing();
            let identity = game.music_identity();
            let stage_music = self.stage_music;
            // **And what this stage already recorded about this frame, where it has been through it
            // before.** A step back replays part of a stage over frames the game's own sound never runs
            // on — `runtime::run_to` walks the calc chain and nothing else — and the restore has asked the
            // game for the track the chapter it put back was taken under. So the music over a replayed
            // fight is the stage's own whatever fight it is, and read off it the stage's own boss is a
            // midboss: whose going down hands the stage back and takes a chapter, the one boundary a stage
            // does not have. A launch of th07 stepping about in its second stage had the whole of that
            // stage's fight as `a midboss nonspell` and a chapter for the boss going down.
            //
            // It can only raise the answer, the same way the music can: a boss recorded as a midboss is
            // this misreading itself, and a fight the stage has no boundary under the frame of — the
            // midboss's defeat lost to a death, and the boss after it read against the midboss's last
            // spellcard — is one the music is still right about.
            let recorded = self.recorded_fight(state.stage_frames);
            let fight = self.fighting.get_or_insert(Fight::Midboss);
            let was = *fight;
            if recorded == Some(Fight::Stage)
                || its_own_song
                    .unwrap_or_else(|| a_track_other_than_the_stages(identity, stage_music))
            {
                *fight = Fight::Stage;
            }
            // What the answer was read off, on the frame a fight arrives and on any frame it changes:
            // which of these three has gone stale is the whole question a fight read as the wrong one
            // turns on, and a launch cannot be asked it afterwards.
            let became_the_stages = *fight != was;
            if boss_appeared || became_the_stages {
                detail!(
                    "fight: {} — recorded {recorded:?}, its own song {its_own_song:?}, \
                     playing {identity:?}, the stage's {stage_music:?}",
                    if *fight == Fight::Stage {
                        "the stage's own"
                    } else {
                        "a midboss"
                    },
                );
            }
            // **And the chapter the fight arrived in called what the fight turned out to be.** The song the
            // stage ends with does not always start on the frame its boss arrives — a launch of th07 read
            // `its own song Some(false)` on the frame stage 2's boss took a chapter and `Some(true)` on the
            // next one — so the chapter for that arrival is there before the answer is. Left as it arrived it
            // is a midboss chapter for the fight the stage ends with, and it is what `recorded_fight` answers
            // with for those frames ever after.
            //
            // The same shape as the spellcard declared after the chapter its attack began, and bounded the
            // same way: what arrives late is the name of the fight rather than another fight.
            let named = match self.mark.cause {
                Cause::MidbossNonspell => Some(Cause::BossNonspell),
                Cause::MidbossSpell => Some(Cause::BossSpell),
                _ => None,
            };
            if became_the_stages
                && elapsed < MIN_CHAPTER_FRAMES
                && let Some(named) = named
            {
                self.rename_chapter(named);
                log!(
                    "stage {} chapter {} at frame {}: the fight it began is the stage's own after all",
                    state.stage + 1,
                    self.mark.number,
                    self.mark.started_at,
                );
            }
        }
        // Beating a midboss hands the stage back, and the waves that follow want a
        // chapter of their own — it is the only boundary between the fight and them.
        // Beating the stage's own boss is not a boundary: the stage is over, and a
        // chapter whose first act is that the fight is already won has nothing to
        // retry. It carved two chapters a second long out of the teardown as well.
        let beaten = !current.boss_present && self.seen.boss_present;
        let midboss_beaten = beaten && self.fighting == Some(Fight::Midboss);
        self.won |= beaten && self.fighting == Some(Fight::Stage);
        if !current.boss_present {
            self.fighting = None;
        }
        // **And a chapter the defeat closes goes with the fight**, which is the same rule read from the
        // other end: an attack declared with the boss all but dead is a chapter whose whole content is the
        // defeat, and a hit taken in the frames it is going down offers it. A launch of th07 was left
        // there — `boss_life=46`, a card declared at frame 13761, the boss gone 42 frames later — and going
        // back to that chapter is going back to the defeat with nothing to fight.
        //
        // Dropped at the defeat rather than refused at the declaration, because what says it is the defeat
        // is the defeat: the life a boss has left is what it is *given* for an attack, and the attack that
        // was the last one is not told from the one before it until the fight ends.
        if beaten
            && elapsed < MIN_CHAPTER_FRAMES
            && self.mark.number > 1
            && let Some(closed) = self.snapshots.take_newest()
        {
            log!(
                "stage {} chapter {} lasted {elapsed} frame(s) and the fight ended in it; \
                 the chapter before it is what a retry goes back to",
                state.stage + 1,
                self.mark.number,
            );
            drop(closed);
            self.starts.pop();
            if let Some(mark) = self.snapshots.newest_mark() {
                self.mark = *mark;
            }
        }
        let spellcard_started =
            current.spellcard.is_some() && current.spellcard != self.seen.spellcard;

        // A spellcard that names itself a frame or two after the chapter its attack began.
        // The boss's timer resets first and the spellcard is declared after, and where those
        // fall on different frames the chapter is already there and called a nonspell —
        // Patchouli's first card and Flandre's last, in stage 7. What arrives late is the name
        // of the attack, not another attack, so the chapter takes the name rather than a
        // second chapter starting for it, which the shortest a chapter may be would refuse
        // anyway.
        if spellcard_started && elapsed < MIN_CHAPTER_FRAMES {
            let named = match self.mark.cause {
                Cause::MidbossNonspell => Some(Cause::MidbossSpell),
                Cause::BossNonspell => Some(Cause::BossSpell),
                _ => None,
            };
            if let Some(named) = named {
                self.rename_chapter(named);
                log!(
                    "stage {} chapter {} at frame {}: the attack it began has a name after all",
                    state.stage + 1,
                    self.mark.number,
                    self.mark.started_at,
                );
                return Due::No;
            }
        }

        // The shortest a chapter may be is a rule against a boss's opening flurry of script
        // transitions, not against a hand: somebody who put a boundary 54 frames after the
        // last one meant it, and dropping it on every pass after the one it was added on
        // would quietly lose what they wrote down. Stage 5's 2363, added by hand 54 frames
        // after the boundary at 2309, is the one that showed it.
        let by_hand = midstage.is_some_and(|frame| self.added_by_hand(game, frame));
        if elapsed < MIN_CHAPTER_FRAMES && !by_hand {
            return Due::No;
        }
        // The boss timer is reset to zero every time the script moves to the
        // next attack, spell or not.
        //
        // **Except while one card stays declared, where the reset is the defeat.** A card is one attack with
        // one timer, so the fight moving on shows as the card changing as well; the boss's life running out
        // during its last card resets that timer with the card still up and the game plays the death out
        // underneath it. Read as an attack, that is a chapter whose whole content is the defeat — a fight
        // with nothing left to fight, which is what stepping back into it goes to.
        //
        // Measured on th07: stage 2's card 25 began at frame 13346 and its timer started again at 14983
        // with `boss_life=1`, the boss gone 60 frames later. The chapter the defeat closes is dropped for
        // being shorter than [`MIN_CHAPTER_FRAMES`], and 60 is exactly that — so the drop cannot be what
        // covers this, and the reset is refused here instead.
        let a_card_still_up =
            current.spellcard.is_some() && current.spellcard == self.seen.spellcard;
        let attack_changed = current.boss_present
            && !a_card_still_up
            && matches!(
                (self.seen.boss_timer, current.boss_timer),
                (Some(before), Some(now)) if now < before
            );
        // A spellcard *beginning* is a fresh thing to fight. One ending is not: either
        // the fight is moving on to another attack, which the timer reset above says
        // on the same frame, or the boss has just been beaten — and a chapter that
        // begins at a defeat has nothing in it to retry. The defeat happens during the
        // last attack, so the spellcard clearing is the only signal it gives here.
        // A fight underway outranks the table. The enemy timeline runs on through a
        // midboss, so a slow fight reaches the boundaries of the waves that come after
        // it — and a chapter beginning inside a fight that is already half fought is a
        // retry point for neither the fight nor the waves. The same frame in a run that
        // killed the midboss quickly is past the fight and is a boundary; that is the
        // clock's doing and not the table's, and the table cannot say which run it is in.
        //
        // The entry is spent rather than held back: the fight's own end is the boundary
        // the waves after it want, and an entry held over it would fire a frame later as
        // that boundary's double.
        if let Some(frame) = midstage.filter(|_| !current.boss_present) {
            return Due::Yes(Cause::Boundary(frame));
        }
        if midboss_beaten {
            return Due::Yes(Cause::StageAfterMidboss);
        }
        // A boss arriving is its first attack starting, so it is named as one. A spellcard
        // beginning is the attack changing — the timer resets for both — and which of the
        // two it is says what is on screen, so it is worth saying.
        if boss_appeared || attack_changed || spellcard_started {
            let midboss = self.fighting != Some(Fight::Stage);
            return Due::Yes(match (midboss, current.spellcard.is_some()) {
                (true, false) => Cause::MidbossNonspell,
                (true, true) => Cause::MidbossSpell,
                (false, false) => Cause::BossNonspell,
                (false, true) => Cause::BossSpell,
            });
        }
        Due::No
    }

    unsafe fn begin_stage(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
    ) {
        // Nothing before the stage's own start counts, whatever the table says: the
        // first chapter is the stage start itself.
        self.mark = Mark {
            number: 1,
            started_at: state.stage_frames,
            midstage_upto: state.script_frames,
            cause: Cause::StageStart,
        };
        self.stage_music = game.music_identity();
        // The fight this stage ends with is ahead of it, whatever the stage before it ended in.
        self.won = false;
        // `tracking` let this frame through, so the run is either one somebody is playing or a
        // replay a pass is reading, and `in_game` is which. Kept for the whole run rather than
        // asked of each frame: what it answers is whether the panel says the lives are disabled,
        // and the frames that would be read wrong are the ones inside a run that are not gameplay
        // frames.
        self.somebody_playing = state.in_game;
        self.starts.clear();
        self.starts.push((state.stage_frames, Cause::StageStart));
        if let Midstage::Tuning(tuning) = &mut self.midstage {
            // The stage just finished is worth keeping even if the pass stops here.
            tuning.write(game);
            tuning.begin_stage(state.stage);
        }
        // Nothing of the stage before this one is a chapter any more, its start included: what a
        // snapshot of it names was released when this stage loaded. The buffers stay to be written
        // over, this one first.
        self.snapshots.keeping(0);

        let snapshot = unsafe {
            take(
                self.snapshots.spare(),
                game,
                data,
                self_check,
                Audio {
                    policy: Music::Rewind(game.music()),
                    identity: game.music_identity(),
                    state: game.audio_state(),
                    thread: game.audio_thread(),
                },
                &game.live_handles(),
            )
        };
        log!(
            "stage {} chapter 1 (stage start) at frame {}, music: {}, \
             lives={} bombs={} power={} deaths={} seed={:#06x}",
            state.stage + 1,
            state.stage_frames,
            snapshot.has_music(),
            state.lives,
            state.bombs,
            state.power,
            state.deaths,
            state.random_seed,
        );
        self.snapshots.push(Checkpoint {
            snapshot,
            mark: self.mark,
        });
    }

    unsafe fn begin_chapter(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
        cause: Cause,
    ) {
        self.mark = Mark {
            number: self.mark.number + 1,
            started_at: state.stage_frames,
            midstage_upto: self.mark.midstage_upto,
            cause,
        };
        // Sorted and without repeats: stepping back replays part of the stage, and
        // the chapters it passes through again are the same ones.
        if let Err(at) = self
            .starts
            .binary_search_by_key(&state.stage_frames, |(frame, _)| *frame)
        {
            self.starts.insert(at, (state.stage_frames, cause));
        }
        let audio = self.audio_for(game, state);
        let keeps_music = matches!(audio.policy, Music::KeepPlaying);
        let live = unsafe { game.live_handles() };
        let snapshot =
            unsafe { take(self.snapshots.spare(), game, data, self_check, audio, &live) };
        // The run's own numbers with every boundary: a replay that has come out of step
        // says so here, by reaching the same frame with different lives or power than
        // the pass before it.
        log!(
            "stage {} chapter {} at frame {} (script {}): {cause}{}, music={}, \
             lives={} bombs={} power={} deaths={}, {} region(s) of {} bytes",
            state.stage + 1,
            self.mark.number,
            state.stage_frames,
            state.script_frames,
            if state.boss_present { ", boss" } else { "" },
            if keeps_music { "keep" } else { "rewind" },
            state.lives,
            state.bombs,
            state.power,
            state.deaths,
            // What one chapter costs to keep, which is what says how many of them can be
            // kept at once.
            snapshot.regions(),
            snapshot.bytes(),
        );
        // **And which regions those are**, at the level a launch being read into is given: the line above
        // says how many and how large, and two launches that differ only in this shape cannot be told
        // apart by a count — see `Snapshot::covers` for the pair that made this worth writing.
        detail!("snapshot: covering {}", snapshot.covers());
        self.snapshots.push(Checkpoint {
            snapshot,
            mark: self.mark,
        });
    }

    /// Takes the current chapter's snapshot again, for a moment when what it holds of the sound is
    /// no longer what is playing: a resume has just put the song where that chapter had it, and the
    /// snapshot was taken with the song wherever the playback left it. Everything else in it is the
    /// same memory it already held.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retake(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
    ) {
        let Some(checkpoint) = self.snapshots.take_newest() else {
            return;
        };
        let mark = checkpoint.mark;
        let audio = self.audio_for(game, state);
        let live = unsafe { game.live_handles() };
        let snapshot = unsafe { take(Some(checkpoint), game, data, self_check, audio, &live) };
        detail!(
            "chapter {} taken again with the sound as it is now",
            mark.number
        );
        self.snapshots.push(Checkpoint { snapshot, mark });
    }

    /// Restores the start of the chapter being played. Returns false when there is
    /// nothing to go back to.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retry_chapter(&mut self, game: &dyn Game) -> bool {
        let Some(at) = self.snapshots.newest() else {
            return false;
        };
        unsafe { self.retry_kept(game, at) }
    }

    /// And the start of any of the chapters this stage has kept, which is what the retry menu asks
    /// for: the chapters after it are no longer chapters of the run, since the run has not played
    /// them yet.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retry_kept(&mut self, game: &dyn Game, at: usize) -> bool {
        let Some(checkpoint) = self.snapshots.at(at) else {
            return false;
        };
        let mark = checkpoint.mark;
        unsafe { put_back(game, checkpoint) };
        // After the restore, against the record that survived it. Only for a run somebody is playing:
        // a pass over a replay retries nothing anybody attempted.
        if self.somebody_playing
            && let Some(attempts) = unsafe { game.count_card_attempt() }
        {
            log!("retry: attempt {attempts} at this spell card");
        }
        self.snapshots.keeping(at + 1);
        self.after_restore(mark);
        unsafe { self.count_a_retry(game) };
        log!("retry chapter {} (retry {})", mark.number, self.retries);
        true
    }

    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retry_stage(&mut self, game: &dyn Game) -> bool {
        if !unsafe { self.rewind_stage(game) } {
            return false;
        }
        unsafe { self.count_a_retry(game) };
        log!(
            "retry stage from chapter {} (retry {})",
            self.mark.number,
            self.retries
        );
        true
    }

    /// One more retry: orb's own count, which the panel and the resume file carry, and the game's own records
    /// where they have a field for it.
    ///
    /// **Only for a run somebody is playing**, which is the same rule the card attempt above goes by: a pass
    /// stepping about in a replay retries nothing anybody attempted, and a number in その他の記録 that a pass
    /// put there is a record of nobody's play.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn count_a_retry(&mut self, game: &dyn Game) {
        self.retries += 1;
        if self.somebody_playing {
            unsafe { game.count_a_retry() };
        }
    }

    /// Puts the stage back to its start without counting a retry, for stepping
    /// back through the chapters: replaying part of a stage to reach a boundary is
    /// not an attempt at the stage.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn rewind_stage(&mut self, game: &dyn Game) -> bool {
        let Some(checkpoint) = self.snapshots.at(0) else {
            return false;
        };
        let mark = checkpoint.mark;
        unsafe { put_back(game, checkpoint) };
        self.snapshots.keeping(1);
        self.after_restore(mark);
        true
    }

    /// A boss's own music is left running across a retry: rewinding a long fight's
    /// theme every attempt is worse than the jump in it. A midboss plays the
    /// stage's music, and that is rewound like anything else midstage.
    fn audio_for(&self, game: &dyn Game, state: &State) -> Audio {
        let identity = game.music_identity();
        let policy = if self.boss_has_own_music(game, state) {
            Music::KeepPlaying
        } else if game.a_sound_command_is_in_flight() {
            // **Nothing of the sound is read while the game's own sound has an ask outstanding.** What the
            // player's stream field names then is not a stream: a restore takes the music down, asks for the
            // track the chapter it puts back had, and copies the game's memory — so the field comes back
            // pointing at the object the take-down deleted, and the ask that would replace it is a command in
            // the game's own queue. The queue is walked by the frame's own sound, which `runtime::run_to`
            // skips, so through a whole replay that field names a released COM object whose leftover vtable
            // pointer still satisfies `mem::vtable_in_image`.
            //
            // **And what such an object does is not defined.** A launch of th07 stepping back in stage 2
            // wrote `taking the music down` and `music: restarting slot 0`, and the replay after it faulted
            // taking a chapter: `Snapshot::update+0x143c`, reading `0x60` with `edi` zero, which is
            // `self.untracked = Vec::new()` with `self` read as null. The report's `esp` was four bytes below
            // where that frame's prologue puts it, so one of the calls made on the dead buffer had returned
            // with the stack short and every `[esp+X]` after it read one slot along.
            detail!("snapshot: the game's own sound is mid-ask; this chapter keeps no music");
            Music::Rewind(None)
        } else {
            Music::Rewind(game.music())
        };
        Audio {
            policy,
            identity,
            state: game.audio_state(),
            thread: game.audio_thread(),
        }
    }

    /// Whether the track playing is a boss's own rather than the stage's, which is what decides
    /// whether the music is put back with a chapter — here and wherever else a chapter's music is
    /// restored, so that the two cannot disagree about one.
    ///
    /// Not the chapter's kind, which is the test that suggests itself and is wrong: a midboss is a
    /// fight with the stage's song still playing, and its chapters want that song put back like any
    /// other of the stage's. Measured as a resume landing in `MIDBOSS NONSPELL 1` with the track at
    /// its opening milliseconds and the position that was written down ignored.
    pub fn stage_song_playing(&self, game: &dyn Game, state: &State) -> bool {
        !self.boss_has_own_music(game, state)
    }

    /// **Whether the fight up is the one the stage ends with**, which is what a recipe's `boss` waits for.
    ///
    /// The boss slot being filled is not that question: a midboss fills it too, so a step waiting on that ended
    /// a stage 4 midsection at its midboss. The song is what tells them apart, and it is the same test as the one
    /// above — a fight with the stage's own song still playing is a midboss.
    pub fn the_stages_own_boss_is_up(&self, game: &dyn Game, state: &State) -> bool {
        self.boss_has_own_music(game, state)
    }

    /// **The game's own answer where it has one**, and the track having changed since the stage began where it
    /// has not: what a stage's start was snapshotted under is not always the stage's own song — see
    /// [`Game::the_fights_own_song_is_playing`], which th07 answers out of the two names its stage data
    /// carries.
    fn boss_has_own_music(&self, game: &dyn Game, state: &State) -> bool {
        state.boss_present
            && game.the_fights_own_song_is_playing().unwrap_or_else(|| {
                a_track_other_than_the_stages(game.music_identity(), self.stage_music)
            })
    }

    fn after_restore(&mut self, mark: Mark) {
        self.mark = mark;
        self.resync = true;
        // The run is back before wherever it had got to, so the stage's own fight is ahead of it again —
        // whatever it was when the snapshot was put back. What this answers is the run as it stands, and a
        // step back past a defeat is a defeat that has not happened.
        self.won = false;
    }

    /// Writes out whatever the tuning pass has found, if one is running.
    pub fn save_tuning(&self, game: &dyn Game) {
        if let Midstage::Tuning(tuning) = &self.midstage {
            tuning.write(game);
        }
    }

    /// Drops what a stage being left holds, on the frame the game is asked to load another one.
    ///
    /// **What a snapshot of a stage names is released as the next one loads** — which is why
    /// [`begin_stage`](Chapters::begin_stage) drops them — and a stage *begins* a few frames after it is
    /// built, once its own start has settled. Between the ask and that beginning, everything kept is the
    /// stage that has gone: a launch of th07 stepped back in those frames and put the first stage's own
    /// start into a process that had loaded the second, which drew over green and faulted inside the game
    /// an update later. The same frames had the fight of the stage left behind still won, so the pass was
    /// walled in the stage it had just arrived in, and its chapter's cause was that fight's — a wash over
    /// a stage's own start, which is the one boundary that gets none.
    ///
    /// The run itself carries on: its retries and what a resume has written down are the run's rather than
    /// this stage's.
    pub fn the_stage_was_left(&mut self) {
        self.stage = None;
        self.settling_until = None;
        self.snapshots.keeping(0);
        self.starts.clear();
        self.won = false;
        self.mark = Mark {
            number: 0,
            started_at: 0,
            midstage_upto: i32::MIN,
            cause: Cause::StageStart,
        };
    }

    /// Drops everything on leaving the run: the snapshots describe a stage that
    /// is no longer loaded, and the retry count belongs to that attempt.
    pub fn forget(&mut self) {
        if self.stage.is_some() {
            log!("run ended after {} retries", self.retries);
        }
        self.stage = None;
        self.retries = 0;
        self.snapshots.forget();
        self.settling_until = None;
        self.somebody_playing = false;
        self.won = false;
        self.mark = Mark {
            number: 0,
            started_at: 0,
            midstage_upto: i32::MIN,
            cause: Cause::StageStart,
        };
        self.starts.clear();
    }
}

/// Whether a track is playing and is not the one the stage began under, which is how a game that cannot name
/// its own two songs tells the fight it ends with from a midboss.
///
/// **Nothing playing is not another track.** The identity is read through the stream, so a player holding none
/// answers `None` — and compared straight, that is unequal to the stage's own song and every fight comes out the
/// stage's. A launch of th07 was left there by a restore that could not put the music back: its second stage's
/// midboss was read as the fight the stage ends with, so beating it set the stage won and a judging pass could
/// not step past that defeat.
fn a_track_other_than_the_stages(playing: Option<u32>, the_stages: Option<u32>) -> bool {
    playing.is_some_and(|playing| Some(playing) != the_stages)
}

/// Puts a checkpoint back, keeping across it the one thing a rewind must not take back.
///
/// The game's record of which spell cards have been captured is in the memory a snapshot covers, so
/// a restore would undo what the game counted about the attempt that just failed: the attempt
/// itself, which is the thing a chapter retried ten times should have ten of, and a capture, which
/// in a chapter that can be played again means that chapter was cleared. Neither is part of what a
/// rewind is for — the stage goes back, what the player has done to the game's own records does not.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn put_back(game: &dyn Game, checkpoint: &Checkpoint) {
    let captures = unsafe { game.captures() };
    unsafe { checkpoint.snapshot.restore(game) };
    unsafe { game.set_captures(&captures) };
    // And the name of the card this chapter is inside put back on the plate it is shown on, which the
    // game baked into a sprite where that card was declared and no snapshot holds. Left as it was, a
    // chapter reached from a *later* card — which is what going back more than one chapter reaches —
    // would be fought under the later card's name.
    //
    // After the records are back, since the name is baked out of the one the game copied into this
    // card's own record: the records above are what carry it across the restore.
    unsafe { game.redraw_card_name() };
}

/// Reuses `previous`'s buffers when there is one, so a boundary costs a copy
/// rather than a copy plus several megabytes of fresh pages.
unsafe fn take(
    previous: Option<Checkpoint>,
    game: &dyn Game,
    data: &Range<usize>,
    self_check: bool,
    audio: Audio,
    live: &[Range<usize>],
) -> Snapshot {
    let started = profile::now();
    let regions = unsafe { memtrack::regions(data.clone()) };
    crash::doing(crash::Doing::TakingASnapshot, 0, 0);
    let snapshot = match previous {
        Some(checkpoint) => {
            let mut snapshot = checkpoint.snapshot;
            unsafe { snapshot.update(&regions, audio, live, self_check) };
            snapshot
        }
        None => unsafe { Snapshot::capture(&regions, audio, live, self_check) },
    };
    unsafe { profile::record(profile::Phase::Snapshot, started) };
    if self_check {
        // Restored into the state it was just taken from, which is the one moment a restore
        // can be held against what it should have produced without disturbing anything.
        unsafe { snapshot.restore(game) };
        let check = unsafe { snapshot.check() };
        log!(
            "self_check: {} saved region(s) did not restore, {} untracked region(s) changed, \
             {} change(s) in the process heap",
            check.unrestored.len(),
            check.changed_untracked.len(),
            check.changed_in_process_heap,
        );
        for region in check
            .unrestored
            .iter()
            .chain(&check.changed_untracked)
            .take(32)
        {
            log!("self_check:   {:#010x}+{:#x}", region.base, region.len);
        }
    }
    // Nothing of orb's in flight any more: what a fault says about that has to be about the frame it
    // happened on, so a note left standing would name work that finished seconds ago.
    crash::doing(crash::Doing::Nothing, 0, 0);
    snapshot
}

impl Seen {
    fn of(state: &State) -> Self {
        Self {
            boss_present: state.boss_present,
            boss_timer: state.boss_attack_frames,
            spellcard: state.spellcard,
        }
    }
}

// Only on the target th06 and th07 are built for — see `game::th06c::tests`, which is what the other
// target's run of these is for.
#[cfg(all(test, target_arch = "x86"))]
mod tests {
    use super::{
        Cause, Chapters, Due, Judgement, KEPT_CHAPTERS, MUSIC_WAIT_FRAMES, Mark,
        STAGE_SETTLE_FRAMES, Seen,
    };
    use crate::game::th06::Th06;
    use crate::game::th06::image::{Boss, Image, Player, Playing, Track};
    use crate::game::{Game, State};
    use std::path::PathBuf;

    /// Judging happens with the game held on the boundary, which is the only way the keys
    /// work: a frame is a sixtieth of a second, and one pressed while the game runs would
    /// land on whichever frame it reached.
    const HELD: bool = true;

    /// The stage every e2e test below plays, and the difficulty it is played at: stage one on Normal,
    /// counted from zero the way everything above `Game` counts stages.
    ///
    /// Laid into the game rather than said to the detector, which is what holds the two to the same
    /// stage: the number the game keeps is one greater — see `game_manager::CURRENT_STAGE` — and the
    /// stage is what the table is looked up by and what a chapter's log line names.
    const STAGE: i32 = 0;
    const DIFFICULTY: i32 = 1;

    /// What the run carries, which is th06's own for one that has just started.
    ///
    /// Nothing here is decided by any of them: they go into the line a chapter is written down with,
    /// which is what says a chapter began with the run in the state it was in.
    const LIVES: i8 = 2;
    const BOMBS: i8 = 3;
    const POWER: u16 = 128;

    /// What the boss of a fight has left. No boundary is derived from a boss's life — what the
    /// detector reads is the clock of the attack it is on — and a fight laid out has to have one all
    /// the same, the life being read through the pointer that says there is a boss at all.
    const BOSS_LIFE: i32 = 1500;

    /// The stage's own song, which a stage whose music has started is playing and which the fight it ends
    /// with is told from.
    const STAGE_TRACK: Track = Track {
        length: 0x38f2,
        loop_start: 0x800,
        loop_end: 0x3800,
    };

    /// The track the fight a stage *ends* with brings, which is the second of the two songs a stage's
    /// data names.
    ///
    /// Its three numbers are this file's own: what tells one track from another is that two tracks are
    /// not the same three numbers, and which of a stage's two fights is on is read off exactly that.
    const BOSS_TRACK: Track = Track {
        length: 0x51a7,
        loop_start: 0x1000,
        loop_end: 0x5000,
    };

    /// A frame of the stage, as it is laid into the game's own memory.
    ///
    /// Every field is one `Th06::read_state` parses back out, which is the whole of why an e2e test
    /// says one of these rather than a `State`: the frame the detector is stepped with is read out of
    /// the game the way production reads it — see [`Stage::standing_on`].
    #[derive(Clone, Copy)]
    struct Frame {
        /// How far into the stage this is, in the game's own clocks. The stage's clock and the
        /// script's are held equal, since all the boundary detection asks of them is that both
        /// advance.
        at: u32,
        enemies: i32,
        /// The boss of the fight on, where one is: what is left of it, and how long the attack it is
        /// on has been running.
        boss: Option<Boss>,
        /// The spell card that boss is on.
        card: Option<i32>,
        player: Player,
        /// Whether a bomb is going off, which clears the screen and takes the danger away for a
        /// couple of seconds.
        bombing: bool,
    }

    /// Two hundred frames of enemies and two hundred without, over and over: a
    /// stage's waves in the only terms the detector reads.
    fn waves(since: u32) -> Frame {
        Frame {
            enemies: if (since / 200).is_multiple_of(2) {
                3
            } else {
                0
            },
            ..at(since)
        }
    }

    /// A frame with nothing on screen at all.
    fn empty(frame: u32) -> Frame {
        Frame {
            at: frame,
            enemies: 0,
            boss: None,
            card: None,
            player: Player::Normal,
            bombing: false,
        }
    }

    /// A directory of its own, so that tests writing files do not read each other's.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join("tuning.txt")).ok();
        dir
    }

    /// The frame the stage's first chapter begins on.
    ///
    /// The stage is latched on the frame its own clock restarts, settles for
    /// `STAGE_SETTLE_FRAMES`, and — with no track playing, which is what a laid-out image with its
    /// sound structures left zeroed reads as — waits out `MUSIC_WAIT_FRAMES` on top of that. Every
    /// frame a test below plays comes after this one, which is why the boundaries are written as
    /// offsets from it rather than as the round numbers the wave pattern alone would put them at.
    const STAGE_BEGINS: u32 = STAGE_SETTLE_FRAMES + MUSIC_WAIT_FRAMES;

    /// A stage running under the real `observe`, and the game whose memory its frames are laid into
    /// and its chapters are copies of.
    ///
    /// Everything here goes through the code the frame hook runs, from the same end: a frame is
    /// written into the game's memory, read back out of it with `Th06::read_state`, and the detector
    /// stepped with what came out. What a chapter beginning looks like from outside is the number
    /// changing, which is what the status line and the retry menu read — so that is what these
    /// report, rather than a verdict read out of the detector.
    ///
    /// **A stage laid out frame by frame rather than one played**, which is what these reach and a run
    /// over the game in `orb-e2e` does not: a thousand frames of waves for the floor on how short a
    /// chapter may be, a boundary judged out and taken back between two of them, and a fight bombed or
    /// died in on a chosen frame. A whole run through the same code is `orb-e2e`'s `pointdevice_run`.
    struct Stage {
        // Declared first so it is dropped first: the image has to still be installed while the
        // chapters it belongs to are being taken down.
        chapters: Chapters,
        image: Image,
    }

    impl Stage {
        /// `tuning` is the directory a pass writes its table into, or `None` for the baked table. A
        /// pass reaches that directory as soon as a stage is left or a boundary is added, so a test
        /// wanting one hands over a scratch of its own — an empty path is the directory the tests
        /// themselves run in.
        fn begun(tuning: Option<PathBuf>, during_replay: bool, of: impl Fn(u32) -> Frame) -> Self {
            let mut stage = Self::laid_out(tuning, during_replay);
            stage.settle(of);
            stage
        }

        /// The game laid out with its stage not yet begun, for a test that has something to put in
        /// its memory first: the chapter is a copy of what is there when the stage settles.
        fn laid_out(tuning: Option<PathBuf>, during_replay: bool) -> Self {
            Self {
                chapters: Chapters::new(&Th06, tuning, during_replay),
                // Nothing here reads the host's own unevenness, so any seed will do and this one says
                // so.
                image: Image::laid_out_seeded(0),
            }
        }

        fn settle(&mut self, of: impl Fn(u32) -> Frame) {
            for frame in 0..=STAGE_BEGINS {
                self.frame(of(frame));
            }
            assert_eq!(
                self.chapters.number(),
                1,
                "the stage's first chapter is taken",
            );
        }

        /// A stage of waves, with the baked table for its midstage boundaries.
        fn waves() -> Self {
            Self::begun(None, false, empty)
        }

        /// A stage of waves with a tuning pass over it, writing into a scratch of its own.
        fn tuned(name: &str) -> Self {
            Self::begun(Some(scratch(name)), false, empty)
        }

        /// A stage of a replay playing back, which a pass building the table follows like a run with
        /// nobody playing it.
        ///
        /// The flag the game sets is laid out before the stage begins, because that is where it
        /// decides something: whether these frames are a run to track chapters over at all.
        fn replayed() -> Self {
            let mut stage = Self::laid_out(None, true);
            stage.image.watching_a_replay();
            stage.settle(empty);
            stage
        }

        /// One frame through the real `observe`, and whether a chapter began on it.
        fn frame(&mut self, frame: Frame) -> bool {
            let state = self.standing_on(frame);
            let _entered = self.image.enter();
            let before = self.chapters.number();
            unsafe {
                self.chapters
                    .observe(&Th06, &state, &self.image.data(), false)
            };
            self.chapters.number() != before
        }

        /// The game standing on that frame, the way production has one: the frame written into the
        /// game's own memory, and `Th06::read_state` reading it back out through every offset and
        /// pointer chase of it.
        ///
        /// Which is what the detector and the judging keys below are handed, and the reason an e2e test
        /// says a [`Frame`] rather than a `State`. A `State` written beside the memory is a second
        /// answer to what the game is doing, and the one thing two answers cannot show is them
        /// disagreeing — `observe` reading a different stage from the one the image holds is a defect
        /// no e2e test built that way can fail for.
        fn standing_on(&self, frame: Frame) -> State {
            let _entered = self.image.enter();
            self.image.playing(Playing {
                stage: STAGE,
                difficulty: DIFFICULTY,
                frames: frame.at,
                script_frames: frame.at as i32,
                seed: 0,
                deaths: 0,
                lives: LIVES,
                bombs: BOMBS,
                power: POWER,
                enemies: frame.enemies,
            });
            self.image.boss(frame.boss);
            self.image.card(frame.card);
            self.image.player(frame.player);
            self.image.bombing(frame.bombing);
            unsafe { Th06.read_state() }
        }

        /// Plays `frames` frames from the stage's start, and reports how far into the stage each
        /// chapter after the first began — not the script frame itself, which carries the settle in
        /// front of it and says nothing about the rule being measured.
        fn play(&mut self, frames: u32) -> Vec<u32> {
            self.play_from(0, frames)
        }

        /// From `from` frames after the stage began, up to `upto`. The wave pattern is counted from
        /// the stage's own start, so where a gap falls is the same question it was.
        fn play_from(&mut self, from: u32, upto: u32) -> Vec<u32> {
            let mut begun = Vec::new();
            for since in from..upto {
                if self.frame(waves(since)) {
                    begun.push(since);
                }
            }
            begun
        }

        /// The real `add_boundary`, which is what the key that adds one by hand reaches — with the
        /// game standing on that frame, since where a boundary goes is where the game is.
        fn add_boundary(&mut self, frame: Frame) {
            let state = self.standing_on(frame);
            let _entered = self.image.enter();
            unsafe {
                self.chapters
                    .add_boundary(&Th06, &state, &self.image.data(), false)
            };
        }

        /// The real `retry_chapter`, with the image in front so the copy lands in it.
        fn retry_chapter(&mut self) -> bool {
            let _entered = self.image.enter();
            unsafe { self.chapters.retry_chapter(&Th06) }
        }

        /// The real `retry_stage`.
        fn retry_stage(&mut self) -> bool {
            let _entered = self.image.enter();
            unsafe { self.chapters.retry_stage(&Th06) }
        }

        /// And the real `retry_kept`, which is the chapter an item of the retry menu asks for.
        fn goes_back_to(&mut self, at: usize) -> bool {
            let _entered = self.image.enter();
            unsafe { self.chapters.retry_kept(&Th06, at) }
        }

        /// Whether the byte written all over `at` before a chapter was taken is what stands there now.
        fn holds(&self, at: usize, byte: u8) -> bool {
            self.image
                .space()
                .read_bytes(at, 0x100)
                .iter()
                .all(|read| *read == byte)
        }
    }

    /// What the retry menu would list, in the order it lists them: what each chapter is called, and
    /// which of the stage's snapshots puts it back.
    fn offered(chapters: &Chapters) -> Vec<(String, usize)> {
        chapters
            .offers()
            .iter()
            .map(|offer| (offer.name.to_string(), offer.at))
            .collect()
    }

    /// The frame `n` frames into the stage, as the game's own clocks read it.
    ///
    /// Every number the table and the stepping are asked about is one of these, because they are
    /// what the game has. What a test writes is how far into the stage it means, which is what the
    /// rules are about: the settle in front of it belongs to no chapter.
    fn since(n: u32) -> u32 {
        STAGE_BEGINS + n
    }

    /// A frame that far into the stage with nothing on screen.
    fn at(n: u32) -> Frame {
        empty(since(n))
    }

    /// The whole of a chapter, through the code that does it: `observe` decides the stage has
    /// begun, takes a snapshot of the game's memory, and `retry_chapter` puts that memory back.
    ///
    /// The bookkeeping and the snapshot together, which is what the helpers below cannot do —
    /// every one of them says "without the snapshot it also takes", and what that cost was the
    /// only assertion that matters: that going back to a chapter puts the game where it was.
    #[test]
    fn going_back_to_a_chapter_puts_the_games_memory_back() {
        let mut stage = Stage::laid_out(None, false);
        // Away from every field `Th06` reads, so that what is being watched is the copy and not
        // the game manager being scribbled on.
        let at = stage.image.data().start + 0x2f00;

        stage.image.space().fill_bytes(at, 0xa1, 0x100);
        stage.settle(empty);

        // The stage running on: whatever the game does to its own memory after the chapter began.
        stage.image.space().fill_bytes(at, 0xc3, 0x100);
        assert!(stage.retry_chapter());

        assert!(
            stage
                .image
                .space()
                .read_bytes(at, 0x100)
                .iter()
                .all(|byte| *byte == 0xa1),
            "the chapter's memory came back",
        );
    }

    /// The stage start is a chapter like any other and going back to it is the same mechanism, so
    /// the one thing worth pinning apart from the chapter case is that it is still there to go back
    /// to after the run has moved on from it.
    #[test]
    fn going_back_to_the_stage_puts_the_games_memory_back() {
        let mut stage = Stage::laid_out(None, false);
        let at = stage.image.data().start + 0x2f00;

        stage.image.space().fill_bytes(at, 0xa1, 0x100);
        stage.settle(empty);

        stage.image.space().fill_bytes(at, 0xc3, 0x100);
        assert!(stage.retry_stage());

        assert!(stage.holds(at, 0xa1));
    }

    /// Every chapter the stage has reached is one to go back to, the one being played first and the
    /// stage's own start last — which is the order the retry menu lists them in.
    #[test]
    fn every_chapter_of_the_stage_is_one_to_go_back_to() {
        let mut stage = Stage::tuned("orb-chapter-offered");
        assert_eq!(stage.play(1200), [259, 659, 1059]);

        assert_eq!(
            offered(&stage.chapters),
            [
                ("MIDSTAGE 4".to_owned(), 3),
                ("MIDSTAGE 3".to_owned(), 2),
                ("MIDSTAGE 2".to_owned(), 1),
                ("MIDSTAGE 1".to_owned(), 0),
            ],
        );
        // And the stage's own start is the one of them said to be that, which is what the menu puts a
        // question in front of.
        let offers = stage.chapters.offers();
        assert!(offers.last().expect("the stage has chapters").stage_start);
        assert!(!offers.iter().rev().skip(1).any(|offer| offer.stage_start));
    }

    /// Going back to one leaves the chapters after it behind: the run has not played them, so they are
    /// nothing to go back to until it reaches them again — which it then does, and they are offered
    /// again.
    #[test]
    fn going_back_to_a_chapter_leaves_the_ones_after_it_behind() {
        let mut stage = Stage::tuned("orb-chapter-back-drops-the-rest");
        assert_eq!(stage.play(1200), [259, 659, 1059]);

        assert!(stage.goes_back_to(1));
        assert_eq!(
            offered(&stage.chapters),
            [("MIDSTAGE 2".to_owned(), 1), ("MIDSTAGE 1".to_owned(), 0)],
        );
        // Which is the chapter the run is in again, and the retry it cost.
        assert_eq!(stage.chapters.number(), 2);
        assert_eq!(stage.chapters.retries(), 1);

        assert_eq!(stage.play_from(300, 1200), [659, 1059]);
        assert_eq!(
            offered(&stage.chapters),
            [
                ("MIDSTAGE 4".to_owned(), 3),
                ("MIDSTAGE 3".to_owned(), 2),
                ("MIDSTAGE 2".to_owned(), 1),
                ("MIDSTAGE 1".to_owned(), 0),
            ],
        );
    }

    /// And a chapter left standing still holds its own memory once the run has played the chapters
    /// above it again: what a later snapshot is written over is the memory of a chapter gone back
    /// past, never one that is still there to go back to.
    #[test]
    fn a_chapter_left_standing_still_holds_the_memory_it_was_taken_with() {
        // Laid out rather than begun, so that the stage's own memory is what this test says it is
        // before the first chapter is taken — and tuned, the gaps between waves being what divides a
        // stage this far into it.
        let mut stage = Stage::laid_out(Some(scratch("orb-chapter-standing")), false);
        let at = stage.image.data().start + 0x2f00;

        // A byte apiece: what the stage's memory held as each of its first four chapters was taken.
        stage.image.space().fill_bytes(at, 0xa1, 0x100);
        stage.settle(empty);
        stage.image.space().fill_bytes(at, 0xb2, 0x100);
        assert_eq!(stage.play_from(0, 300), [259]);
        stage.image.space().fill_bytes(at, 0xc3, 0x100);
        assert_eq!(stage.play_from(300, 700), [659]);
        stage.image.space().fill_bytes(at, 0xd4, 0x100);
        assert_eq!(stage.play_from(700, 1100), [1059]);

        // Back to the second of them, and the stage played forward from there with its memory holding
        // something else: the two chapters it reaches again are taken over the buffers of the two it
        // went back past.
        assert!(stage.goes_back_to(1));
        assert!(stage.holds(at, 0xb2));
        stage.image.space().fill_bytes(at, 0xe5, 0x100);
        assert_eq!(stage.play_from(300, 1100), [659, 1059]);

        // Every chapter still there is the memory it was taken with, the two that were taken again
        // included. Newest first, because going back to one drops the chapters after it: each is asked
        // for from under the last, which is the run walking down what the stage kept.
        for (chapter, byte) in [(3, 0xe5), (2, 0xe5), (1, 0xb2), (0, 0xa1)] {
            assert!(stage.goes_back_to(chapter));
            assert!(
                stage.holds(at, byte),
                "chapter {chapter} does not hold {byte:#04x}",
            );
        }
    }

    /// A stage with more chapters than there are snapshots for drops the oldest above its own start,
    /// which stays: it is the way out of the whole stage, and what a death wants is the chapters
    /// around the one it happened in.
    #[test]
    fn a_stage_with_more_chapters_than_it_keeps_drops_the_oldest_above_its_start() {
        let mut stage = Stage::tuned("orb-chapter-cap");
        // A gap for every chapter the stage keeps, which with its own start is one chapter more than
        // that: the first gap falls at 259 and the rest 400 frames apart.
        let gaps = KEPT_CHAPTERS;
        let played = stage.play(260 + 400 * (gaps as u32 - 1));
        assert_eq!(played.len(), gaps);

        // The chapter being played and the ones behind it, and then the stage's own start: what went is
        // the chapters between.
        let newest = gaps + 1;
        let mut expected: Vec<(String, usize)> = (0..KEPT_CHAPTERS - 1)
            .map(|back| {
                (
                    format!("MIDSTAGE {}", newest - back),
                    KEPT_CHAPTERS - 1 - back,
                )
            })
            .collect();
        expected.push(("MIDSTAGE 1".to_owned(), 0));
        assert_eq!(offered(&stage.chapters), expected);
    }

    /// A frame of a fight, with the boss's timer and whichever attack it is on.
    fn fight(since: u32, timer: i32, card: Option<i32>) -> Frame {
        Frame {
            boss: Some(Boss {
                life: BOSS_LIFE,
                attack_frames: timer,
            }),
            card,
            ..at(since)
        }
    }

    /// A frame with enemies on it, for waiting out the floor on a chapter's length without
    /// the gap detector offering a boundary in the quiet.
    fn enemies(since: u32) -> Frame {
        Frame {
            enemies: 3,
            ..at(since)
        }
    }

    fn named(chapters: &Chapters) -> String {
        chapters.name().expect("a stage is running").to_string()
    }

    /// Whether somebody is playing is settled by the frame the stage began on and then stands for
    /// the whole run, which is what the mark over the lives is drawn on — and it goes when the run
    /// does, since a front end still carrying the mark would be the panel of a run that is over.
    ///
    /// It stands for the run because the run passes through frames that are not gameplay frames and
    /// are drawn like any other: the one a stage transition is built in, and the one a chapter is
    /// put back on. Asking those frames would read them as nobody playing.
    #[test]
    fn a_run_somebody_is_playing_is_settled_where_the_stage_began() {
        let mut stage = Stage::waves();
        assert!(stage.chapters.somebody_playing());
        stage.chapters.forget();
        assert!(!stage.chapters.somebody_playing());
    }

    /// And a replay is not one, whatever else is tracked over it: it drives the same scene with
    /// nobody to offer a chapter to, and dying in one costs the life it costs.
    ///
    /// Followed here as a pass building the table follows it, which is the only thing that follows
    /// a replay at all: `tracking` lets a run somebody is playing through on `in_game`, and a
    /// replay only on `during_replay`. Without that this stage never begins, so what the answer is
    /// about is a stage that did.
    #[test]
    fn a_replay_is_not_a_run_somebody_is_playing() {
        let stage = Stage::replayed();
        assert!(!stage.chapters.somebody_playing());
    }

    /// What the status line and the retry menu call each chapter, in the order a stage
    /// reaches them: numbered inside the part of the stage it belongs to rather than counted
    /// straight through, and the waves picking their own count up again after the midboss.
    #[test]
    fn a_chapter_is_named_for_the_part_of_the_stage_it_is_in() {
        let mut stage = Stage::tuned("orb-chapter-named");
        assert_eq!(named(&stage.chapters), "MIDSTAGE 1");

        // A gap in the waves, which the detector offers at 259.
        for frame in 0..300 {
            stage.frame(waves(frame));
        }
        assert_eq!(named(&stage.chapters), "MIDSTAGE 2");

        // A midboss, whose first attack starts with it. The stage began with no track under it, so
        // which fight this is comes from `music_identity` still reporting that same nothing.
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "MIDBOSS NONSPELL 1");

        for frame in 321..380 {
            stage.frame(fight(frame, frame as i32 - 320, None));
        }
        stage.frame(fight(380, 0, Some(1)));
        assert_eq!(named(&stage.chapters), "MIDBOSS SPELL 1");

        for frame in 381..441 {
            stage.frame(fight(frame, frame as i32 - 380, Some(1)));
        }
        // Beaten, which hands the stage back.
        stage.frame(enemies(441));
        assert_eq!(named(&stage.chapters), "MIDSTAGE 3");

        // The stage's own boss brings the second of the two songs the stage names, which is
        // what tells it from a midboss — the track laid out where the game would start it.
        stage.image.plays_a_track(BOSS_TRACK);
        for frame in 442..511 {
            stage.frame(enemies(frame));
        }
        stage.frame(fight(511, 0, None));
        assert_eq!(named(&stage.chapters), "BOSS NONSPELL 1");
    }

    /// **A fight replayed is the fight the stage recorded**, whatever the music says by then.
    ///
    /// Which of a stage's two fights is on is read off the music, the stage's data naming the second song for
    /// the boss it ends with. **A step back replays part of the stage over frames where the game's own sound
    /// never runs** — `runtime::run_to` walks the calc chain and nothing else — and the restore has asked the
    /// game for the track the chapter it put back was taken under. So the track over a replayed fight is the
    /// stage's own, and read off it the stage's boss is a midboss: whose going down hands the stage back and
    /// takes a chapter, which is the one boundary a stage does not have.
    ///
    /// A launch of th07 found it. Stepping about in its second stage left the whole of that stage's fight
    /// logged as `a midboss nonspell` and `a midboss spellcard`, and the boss going down took a chapter of its
    /// own. What answers instead is what the stage already knows: the chapter it divided this frame into the
    /// first time through, which is the same list the stepping keys move between.
    #[test]
    fn a_fight_replayed_is_the_fight_the_stage_recorded() {
        let mut stage = Stage::tuned("orb-chapter-fight-replayed");
        for frame in 0..300 {
            stage.frame(waves(frame));
        }
        let before_the_fight = stage.chapters.mark;

        // The stage's own boss, under the second of the two songs the stage names: the track laid out where
        // the game would start it, which is what tells this fight from a midboss.
        stage.image.plays_a_track(BOSS_TRACK);
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "BOSS NONSPELL 1");

        // What a step back leaves: the mark and the signals back with the memory, and the music the track
        // that chapter had — which for a chapter of the stage's own waves is no track at all.
        stage.chapters.mark = before_the_fight;
        stage.chapters.seen = Seen::of(&stage.standing_on(waves(299)));
        stage.chapters.fighting = None;
        stage.image.plays_no_track();

        // And the fight again, which the replay brings: the same frame, and nothing about the sound to read
        // it off.
        stage.frame(fight(320, 0, None));
        assert_eq!(
            named(&stage.chapters),
            "BOSS NONSPELL 1",
            "the fight the stage ends with was read as a midboss on the way through it again",
        );

        // And the boss going down through it: the stage is over, so the defeat is no boundary and the
        // stage is won.
        for frame in 321..700 {
            stage.frame(fight(frame, (frame - 320) as i32, None));
        }
        stage.frame(empty(since(700)));
        assert_eq!(
            named(&stage.chapters),
            "BOSS NONSPELL 1",
            "the stage's own boss going down took a chapter",
        );
        assert!(
            stage.chapters.stage_won(),
            "the stage was not won by its own boss going down",
        );
    }

    /// The card a boss is beaten during, by its own record.
    const A_CARD: i32 = 25;

    /// **A card whose timer starts again is not another attack, and the defeat is what it is.**
    ///
    /// The timer resetting is how the fight moving on to its next attack is read, and the defeat resets it
    /// too: the boss's life runs out during its last card, the card stays declared, and the timer goes back to
    /// nothing while the game plays the death out. Read as an attack that is a chapter offered for the
    /// defeat — a fight with nothing left to fight, one bomb from over.
    ///
    /// A launch of th07 is the measurement: stage 2's chapter 11 began card 25 at frame 13346, and chapter
    /// 12 began at 14983 on the *same* card 25 with `attack_frames=1` and `boss_life=1`, with the boss gone 60
    /// frames later. **Sixty**, which is why the chapter the defeat closes was not dropped either — that rule
    /// asks for shorter than [`MIN_CHAPTER_FRAMES`], and this was exactly it.
    #[test]
    fn the_same_card_whose_timer_starts_again_is_not_another_attack() {
        let mut stage = Stage::tuned("orb-chapter-card-timer-again");
        for frame in 0..300 {
            stage.frame(waves(frame));
        }

        // The stage's own boss, under the second of the two songs its stage names, and the card it puts up.
        stage.image.plays_a_track(BOSS_TRACK);
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "BOSS NONSPELL 1");
        for frame in 321..500 {
            stage.frame(fight(frame, (frame - 320) as i32, None));
        }
        stage.frame(fight(500, 0, Some(A_CARD)));
        assert_eq!(named(&stage.chapters), "BOSS SPELL 1");
        for frame in 501..800 {
            stage.frame(fight(frame, (frame - 500) as i32, Some(A_CARD)));
        }
        let the_cards_chapter = stage.chapters.number();

        // The boss beaten during it: the same card still declared and its own timer started again.
        stage.frame(fight(800, 1, Some(A_CARD)));
        assert_eq!(
            stage.chapters.number(),
            the_cards_chapter,
            "the defeat inside the card began a chapter of its own",
        );

        // And the death played out, which is the stage over with the card's chapter the last one there is.
        for frame in 801..860 {
            stage.frame(fight(frame, (frame - 800) as i32, Some(A_CARD)));
        }
        stage.frame(empty(since(860)));
        assert!(
            stage.chapters.stage_won(),
            "the stage was not won by its own boss going down",
        );
        assert_eq!(
            named(&stage.chapters),
            "BOSS SPELL 1",
            "the chapter left standing at the stage's end is not the card that was being fought",
        );
    }

    /// **No music at all says nothing about which fight is on.**
    ///
    /// Where the game cannot name its own two songs, which fight it is comes from the track having changed since
    /// the stage began. A track that has *gone* is not a track that has changed: the stream is what the identity
    /// is read through, and a stage playing nothing reads as `None` — which compares unequal to the stage's own
    /// song and so answers every fight the stage's own.
    ///
    /// A launch of th07 is the measurement, by way of a restore that left the player holding no stream: from
    /// there its second stage's *midboss* was read as the fight the stage ends with, so beating it set the stage
    /// won and the judging pass could not step past that defeat.
    #[test]
    fn a_fight_with_nothing_playing_is_no_fight_of_the_stages_own() {
        let mut stage = Stage::laid_out(Some(scratch("orb-chapter-no-music")), false);
        // The stage's own song from before its start, so what the stage began under is a track and not nothing.
        stage.image.plays_a_track(STAGE_TRACK);
        stage.settle(empty);
        for frame in 0..300 {
            stage.frame(waves(frame));
        }

        // And the sound gone: a stream the game has taken down and not made again, which is what a restore that
        // could not put the music back leaves.
        stage.image.plays_no_track();
        stage.frame(fight(320, 0, None));
        assert_eq!(
            named(&stage.chapters),
            "MIDBOSS NONSPELL 1",
            "a fight with nothing playing was read as the fight the stage ends with",
        );

        // And its defeat hands the stage back rather than winning it.
        for frame in 321..500 {
            stage.frame(fight(frame, (frame - 320) as i32, None));
        }
        stage.frame(empty(since(500)));
        assert!(
            !stage.chapters.stage_won(),
            "beating that fight was taken for the stage being over",
        );
        assert!(matches!(stage.chapters.cause(), Cause::StageAfterMidboss));
    }

    /// **The chapter a fight arrived in takes the name its song gives it.**
    ///
    /// The song the stage ends with does not always start on the frame its boss arrives. A launch of th07
    /// read `its own song Some(false)` on the frame stage 2's boss took a chapter — frame 8099 — and
    /// `Some(true)` on the frame after it. Left as it arrived, that chapter is a midboss nonspell for the
    /// fight the stage ends with, and it is what [`Chapters::recorded_fight`] answers with for those frames
    /// ever after: a step back into it replays the fight as a midboss, whose going down takes a chapter.
    ///
    /// The same shape as the spellcard that names itself after the chapter its attack began — what arrives
    /// late is the name of the fight rather than another fight — so the chapter takes it while it is younger
    /// than [`MIN_CHAPTER_FRAMES`], in the record and in the snapshot a retry offers it out of as well.
    #[test]
    fn the_chapter_a_fight_arrived_in_takes_the_name_its_song_gives_it() {
        let mut stage = Stage::laid_out(Some(scratch("orb-chapter-fight-named-late")), false);
        // The stage's own song from before its start, which is what a fight arriving under it is read
        // against.
        stage.image.plays_a_track(STAGE_TRACK);
        stage.settle(empty);
        for frame in 0..300 {
            stage.frame(waves(frame));
        }

        // The boss, arriving with the stage's own song still playing: nothing about the sound tells this
        // fight from a midboss yet.
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "MIDBOSS NONSPELL 1");

        // And its own song a frame later, which is the stage's data saying which fight it is.
        stage.image.plays_a_track(BOSS_TRACK);
        stage.frame(fight(321, 1, None));
        assert_eq!(
            named(&stage.chapters),
            "BOSS NONSPELL 1",
            "the chapter the fight arrived in kept the name it arrived under",
        );
        assert!(
            offered(&stage.chapters)
                .iter()
                .any(|(name, _)| name == "BOSS NONSPELL 1"),
            "a retry offers the chapter under the name it arrived under: {:?}",
            offered(&stage.chapters),
        );
    }

    /// A chapter reached twice is the same chapter. The count comes out of the frames this
    /// stage's chapters began at, not out of a counter per kind: a retry puts the mark back
    /// and the run then reaches the same chapters again, which a counter would name one
    /// further along every time.
    #[test]
    fn a_chapter_reached_again_keeps_its_name() {
        let mut stage = Stage::tuned("orb-chapter-named-again");
        for frame in 0..300 {
            stage.frame(waves(frame));
        }
        let boundary = stage.chapters.mark;
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "MIDBOSS NONSPELL 1");

        // What a retry does to orb's own bookkeeping: the mark goes back with the memory.
        stage.chapters.mark = boundary;
        stage.chapters.seen = Seen::of(&stage.standing_on(waves(299)));
        assert_eq!(named(&stage.chapters), "MIDSTAGE 2");
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "MIDBOSS NONSPELL 1");
    }

    /// **And it keeps it where the second time through begins a frame later, which is the only way a
    /// restore ever lands.** The frame the run comes back on is one nothing is read on — `resync` — so the
    /// attack that was seen at 320 is seen at 321 the next time, and both frames are recorded.
    ///
    /// What makes them the same chapter is how far apart they are: two starts of one kind closer together
    /// than the shortest a chapter may be cannot be two chapters, since a boundary that soon after the mark
    /// is one the detector refuses.
    #[test]
    fn a_chapter_reached_again_a_frame_later_keeps_its_name() {
        let mut stage = Stage::tuned("orb-chapter-named-a-frame-later");
        for frame in 0..300 {
            stage.frame(waves(frame));
        }
        let boundary = stage.chapters.mark;
        stage.frame(fight(320, 0, None));
        assert_eq!(named(&stage.chapters), "MIDBOSS NONSPELL 1");

        // What a retry does to orb's own bookkeeping, through the call that does it rather than by
        // putting the mark back by hand: what is being asked about is the rest of what that call moves.
        stage.chapters.after_restore(boundary);
        stage.chapters.seen = Seen::of(&stage.standing_on(waves(299)));
        assert_eq!(named(&stage.chapters), "MIDSTAGE 2");
        // The frame the run lands on, which begins nothing, and the midboss arriving on the next one.
        stage.frame(waves(320));
        stage.frame(fight(321, 0, None));
        assert_eq!(named(&stage.chapters), "MIDBOSS NONSPELL 1");
    }

    /// One put there by hand begins its chapter however close behind the last one it is. The
    /// shortest a chapter may be is there to stop a boss's flurry of script transitions
    /// carving out chapters a fraction of a second long, and a hand is not that: dropping it
    /// on the passes after the one it was added on would lose what somebody wrote down, and
    /// silently, since the boundary would still be in the table.
    #[test]
    fn one_added_by_hand_is_not_too_close_to_the_last() {
        let mut stage = Stage::tuned("orb-chapter-hand-close");
        assert_eq!(stage.play_from(0, 300), [259]);

        // Fifty-four frames after it, which is inside the floor.
        stage.add_boundary(at(313));
        // And it is still a boundary on the next pass over the same ground.
        stage.chapters.mark = Mark {
            midstage_upto: 0,
            ..stage.chapters.mark
        };
        let on_it = stage.standing_on(waves(313));
        // With the image in front, `due` being one of the calls that asks the game what is playing.
        let _entered = stage.image.enter();
        assert!(matches!(
            stage.chapters.due(&Th06, &on_it, &Seen::of(&on_it)),
            Due::Yes(Cause::Boundary(frame)) if frame == since(313) as i32,
        ));
    }

    /// And the same question is answerable of the baked table, which is what makes that exemption
    /// the same in play as in the `--judge` pass that chose the numbers. While only a tuning pass
    /// could answer it, a table entry inside the floor began a chapter under `--judge` and was
    /// silently dropped in the run that used the table it wrote.
    #[test]
    fn the_baked_table_says_which_boundaries_a_hand_put_there() {
        let mut chapters = Chapters::new(&Th06, None, false);
        // Stage 5, whose 2363 somebody put there and whose 6827 the detector proposed.
        chapters.stage = Some(4);
        assert!(chapters.added_by_hand(&Th06, 2363));
        assert!(!chapters.added_by_hand(&Th06, 6827));
        // A frame no boundary of that stage falls on is nobody's, and neither is one of another
        // stage's.
        assert!(!chapters.added_by_hand(&Th06, 2364));
        assert!(!chapters.added_by_hand(&Th06, 4472));
    }

    /// A spellcard that names itself after the chapter its attack began takes that chapter's
    /// name rather than being lost: the boss's timer resets on one frame and the card is
    /// declared on the next, and the shortest a chapter may be would refuse a second chapter
    /// for the same attack.
    #[test]
    fn a_spellcard_named_late_still_names_its_chapter() {
        let mut stage = Stage::tuned("orb-chapter-spellcard-late");

        for frame in 0..200 {
            stage.frame(fight(frame, frame as i32, None));
        }
        // The timer starts again, which is the attack changing and all that is known of it.
        assert!(stage.frame(fight(200, 0, None)));
        assert_eq!(stage.chapters.number(), 2);
        assert!(matches!(
            stage.chapters.cause(),
            Cause::MidbossNonspell | Cause::BossNonspell
        ));

        // The card names itself on the next frame. No second chapter, and the one standing
        // there is a spellcard's.
        assert!(!stage.frame(fight(201, 1, Some(3))));
        assert!(matches!(
            stage.chapters.cause(),
            Cause::MidbossSpell | Cause::BossSpell
        ));
    }

    /// A boundary judged out of the table is not a chapter start, so the stepping passes
    /// over it — and it is not gone either, so `chapter_dropped_key` with a stepping key
    /// still reaches it. In script frames, since nothing has a stage frame for a boundary
    /// that begins no chapter.
    #[test]
    fn the_stepping_passes_over_a_boundary_judged_out_and_the_dropped_key_reaches_it() {
        let mut stage = Stage::tuned("orb-chapter-stepping-judged-out");
        assert_eq!(stage.play(1200), [259, 659, 1059]);
        stage
            .chapters
            .tuning()
            .unwrap()
            .reject(0, since(659) as i32);

        let from = stage.standing_on(at(300));
        let last = stage.standing_on(at(1200));
        assert_eq!(stage.chapters.next_start(&from), Some(since(1059)));
        assert_eq!(stage.chapters.previous_start(&last), Some(since(1059)));
        assert_eq!(stage.chapters.next_dropped(&from), Some(since(659) as i32));
        assert_eq!(
            stage.chapters.previous_dropped(&last),
            Some(since(659) as i32),
        );
        // Taken back, it is a chapter start again and no longer the dropped key's.
        let on_it = stage.standing_on(at(659));
        stage.chapters.judge(&on_it, HELD, Judgement::Better);
        assert_eq!(stage.chapters.next_start(&from), Some(since(659)));
        assert_eq!(stage.chapters.next_dropped(&from), None);
    }

    /// One put there by hand and then taken out is gone from the table altogether, so the
    /// chapter it began is no longer a place the stepping stops at either. Nothing can be
    /// judged there and `chapter_dropped_key` cannot find it, so stopping would be stopping
    /// at nothing.
    #[test]
    fn the_stepping_passes_over_a_boundary_taken_out_by_hand() {
        let mut stage = Stage::tuned("orb-chapter-stepping-hand-out");
        stage.play(1200);

        stage.add_boundary(at(500));
        let on_it = stage.standing_on(at(500));
        let from = stage.standing_on(at(300));
        assert_eq!(stage.chapters.next_start(&from), Some(since(500)));

        stage.chapters.judge(&on_it, HELD, Judgement::Worse);
        stage.chapters.judge(&on_it, HELD, Judgement::Worse);
        assert!(
            stage.chapters.judged(&on_it, HELD).is_none(),
            "taken out altogether",
        );
        assert_eq!(stage.chapters.next_start(&from), Some(since(659)));
        assert_eq!(
            stage.chapters.previous_start(&stage.standing_on(at(600))),
            Some(since(259)),
        );
        assert_eq!(stage.chapters.next_dropped(&from), None);
    }

    /// A fight ending on the frame the player is hit still ends, on that frame: stage 6's
    /// midboss went down as the player died, and dying is not a reason to pass a boundary
    /// over — the way out of a start that kills whoever restores it is the chapter before it.
    #[test]
    fn a_fight_that_ends_as_the_player_dies_still_ends() {
        let mut stage = Stage::tuned("orb-chapter-fight-ends-on-death");

        // A midboss, up long enough for a chapter to be allowed after it. Its attack's clock stands
        // still, since a reset is the fight moving on and this fight does not.
        for frame in 0..200 {
            stage.frame(fight(frame, 0, None));
        }
        let hit = Frame {
            player: Player::Dying,
            ..at(200)
        };
        assert!(stage.frame(hit));
        assert!(matches!(stage.chapters.cause(), Cause::StageAfterMidboss));
    }

    /// An attack changing under a bomb is a boundary on the frame it changes: a bomb is two
    /// seconds long, and the frame the fight moved on is the one worth going back to.
    #[test]
    fn an_attack_that_changes_under_a_bomb_is_a_boundary_there() {
        let mut stage = Stage::tuned("orb-chapter-bombed-attack-change");
        let spell = |since, timer| fight(since, timer, Some(7));

        for frame in 0..200 {
            stage.frame(spell(frame, frame as i32));
        }
        // The spellcard is bombed out and the next attack begins while the bomb is still going: the card is
        // no longer declared and the boss's timer starts again from nothing.
        //
        // **The card gone rather than still up**, which is what the fight moving on looks like — the event
        // behind this is the bomb between the last two of Cirno's spellcards, whose nonspell it swallowed, so
        // what the guard lifts on is another attack than the one it came down over. A card still declared
        // with its timer started again is the *defeat* instead, and no boundary at all — see
        // `the_same_card_whose_timer_starts_again_is_not_another_attack`.
        let bombed = Frame {
            bombing: true,
            ..fight(200, 0, None)
        };
        // Whose fight it is comes from the music, which no track is laid out for here, so
        // either naming will do — what matters is that the change was taken.
        assert!(stage.frame(bombed));
        assert!(matches!(
            stage.chapters.cause(),
            Cause::BossNonspell | Cause::MidbossNonspell
        ));
    }

    /// The judging keys do nothing unless the game is standing still on the boundary: a
    /// frame is a sixtieth of a second, and a key pressed while the game runs would land on
    /// whichever frame it reached.
    #[test]
    fn judging_needs_the_game_held_on_the_boundary() {
        let mut stage = Stage::tuned("orb-chapter-judging-held");
        stage.play(1200);

        let on_it = stage.standing_on(at(659));
        assert!(stage.chapters.judged(&on_it, false).is_none());
        stage.chapters.judge(&on_it, false, Judgement::Worse);
        assert_eq!(
            stage.chapters.judged(&on_it, HELD).unwrap().verdict.label(),
            "KEEP",
        );

        // Nor anywhere else, held or not: what is judged is the boundary underneath, and
        // inside a chapter there is none.
        let past_it = stage.standing_on(at(700));
        assert!(stage.chapters.judged(&past_it, HELD).is_none());
        stage.chapters.judge(&past_it, HELD, Judgement::Worse);
        assert_eq!(
            stage.chapters.judged(&on_it, HELD).unwrap().verdict.label(),
            "KEEP",
        );
    }

    /// A fight underway outranks the table: the enemy timeline runs on through a midboss,
    /// so a fight that drags reaches the boundaries of the waves after it, and a chapter
    /// beginning half way through a fight is a retry point for neither. The entry is spent
    /// where it falls, so the fight's end is the one boundary there.
    #[test]
    fn a_boundary_reached_during_a_fight_is_not_one() {
        let mut plain = Stage::tuned("orb-chapter-fight-outranks-plain");
        // The detector's boundary for the first gap, found in a run with no fight in it.
        assert_eq!(plain.play(400), [259]);

        // The same stage again, with a midboss on screen from before that boundary until
        // after it. Nothing of the table's begins a chapter while it is up.
        let mut fought = Stage::tuned("orb-chapter-fight-outranks-fought");
        fought.chapters.tuning().unwrap().add(0, since(259) as i32);
        for frame in 0..400 {
            fought.frame(fight(frame, frame as i32, None));
            assert!(
                !matches!(fought.chapters.cause(), Cause::Boundary(_)),
                "the table began a chapter at frame {frame}, inside the fight",
            );
        }
        // And it is spent where it fell rather than saved up: what the frame the fight ends
        // on has is the fight's own end, not the table's entry firing as its double.
        assert!(fought.frame(at(400)));
        assert!(matches!(fought.chapters.cause(), Cause::StageAfterMidboss));
    }

    /// A second into each gap, and once per gap however long it runs on: the wave
    /// leaves at 200 and the boundary falls at 259, the next at 659.
    #[test]
    fn a_gap_between_waves_becomes_one_chapter() {
        let mut stage = Stage::tuned("orb-chapter-gap");
        assert_eq!(stage.play(1200), [259, 659, 1059]);
    }

    #[test]
    fn a_stage_played_twice_divides_the_same_way() {
        let mut stage = Stage::tuned("orb-chapter-twice");
        let start = stage.chapters.mark;
        let first = stage.play(1200);

        // What stepping back through the chapters does: the stage's start comes
        // back and part of the stage is played again.
        stage.chapters.mark = start;
        assert_eq!(stage.play(1200), first);
        // And each boundary is written down once, however often it is played over.
        assert_eq!(stage.chapters.tuning().unwrap().count(0), first.len(),);
    }

    #[test]
    fn a_boundary_removed_by_hand_does_not_come_back() {
        let mut stage = Stage::tuned("orb-chapter-removed-stays-out");
        let start = stage.chapters.mark;
        stage.play(1200);

        stage
            .chapters
            .tuning()
            .unwrap()
            .reject(0, since(659) as i32);
        stage.chapters.mark = start;
        assert_eq!(stage.play(1200), [259, 1059]);
    }

    /// The detector's own proposals are kept when they are judged out, and kept as
    /// the detector's: forgetting one would have it proposed again the next time the
    /// stage was played, and the same boundary would have to be judged out every
    /// session.
    #[test]
    fn a_boundary_judged_out_stays_out_in_the_next_session() {
        let dir = scratch("orb-dropped-stays-dropped");
        let mut stage = Stage::begun(Some(dir.clone()), false, empty);
        assert_eq!(stage.play(1200), [259, 659, 1059]);

        let tuning = stage.chapters.tuning().unwrap();
        tuning.reject(0, since(659) as i32);
        tuning.write(&Th06);
        drop(stage);

        let mut next = Stage::begun(Some(dir), false, empty);
        // Not proposed again, and not because the detector was suppressed: the stage
        // is played from the top with the frames it covers all new to this pass.
        assert_eq!(next.play(1200), [259, 1059]);
        let judged = next
            .chapters
            .tuning()
            .unwrap()
            .judged(0, since(659) as i32)
            .expect("still known");
        assert_eq!(judged.verdict.label(), "DROP");
        assert!(!judged.by_hand);
    }

    /// Stepping onto a boundary judged out is how it gets taken back, so the frame the
    /// game is on has to be what says which boundary is being looked at: a refused one
    /// no longer begins the chapter standing there.
    #[test]
    fn a_boundary_judged_out_can_be_taken_back_from_where_it_is() {
        let mut stage = Stage::tuned("orb-chapter-taken-back");
        stage.play(1200);
        let on_it = stage.standing_on(at(659));
        stage.chapters.judge(&on_it, HELD, Judgement::Out);

        let label = |stage: &Stage| stage.chapters.judged(&on_it, HELD).unwrap().verdict.label();
        assert_eq!(label(&stage), "DROP");
        stage.chapters.judge(&on_it, HELD, Judgement::Better);
        assert_eq!(label(&stage), "ADJUST");
        stage.chapters.judge(&on_it, HELD, Judgement::Better);
        assert_eq!(label(&stage), "KEEP");
    }

    /// One put there by hand goes altogether instead: nothing proposes it again, so
    /// there is nothing for a refusal to hold back, and `tuning_add_key` is the way
    /// back.
    #[test]
    fn a_boundary_added_by_hand_is_taken_out_altogether() {
        let mut stage = Stage::tuned("orb-chapter-hand-out-altogether");
        let start = stage.chapters.mark;
        stage.play(1200);

        stage.add_boundary(at(500));
        let on_it = stage.standing_on(at(500));
        let judged = stage
            .chapters
            .judged(&on_it, HELD)
            .expect("the added boundary is the one to judge");
        assert_eq!(judged.verdict.label(), "KEEP");
        assert!(judged.by_hand);

        stage.chapters.judge(&on_it, HELD, Judgement::Out);
        assert!(stage.chapters.judged(&on_it, HELD).is_none());
        stage.chapters.mark = start;
        assert_eq!(stage.play(1200), [259, 659, 1059]);
    }

    /// A hand and the detector looking at the same lull must not leave two boundaries a
    /// frame apart. The hand goes down when the screen empties, the detector a second
    /// into it, so one put there by hand just before is the near miss to rule out.
    #[test]
    fn a_boundary_added_by_hand_leaves_no_room_for_one_beside_it() {
        let mut stage = Stage::tuned("orb-chapter-hand-no-room");
        // The gap opens at 200 and the detector's own boundary for it falls at 259.
        assert_eq!(stage.play_from(0, 259), []);
        stage.add_boundary(waves(258));
        stage.play_from(259, 1200);

        let tuning = stage.chapters.tuning().unwrap();
        assert!(
            tuning
                .judged(0, since(258) as i32)
                .expect("the one put there by hand")
                .by_hand
        );
        assert!(tuning.judged(0, since(259) as i32).is_none());
        // The gaps after it are still the detector's to propose.
        assert_eq!(tuning.count(0), 3);
    }

    /// Two presses take one out; the third must find nothing rather than start taking out
    /// the boundary the chapter began at, which is frames back with nothing on screen to
    /// say it is what the key is about to change.
    #[test]
    fn taking_one_out_does_not_reach_the_boundary_behind_it() {
        let mut stage = Stage::tuned("orb-chapter-taking-out-reaches-no-further");
        stage.play(1200);

        stage.add_boundary(at(700));
        let on_it = stage.standing_on(at(700));
        for _ in 0..4 {
            stage.chapters.judge(&on_it, HELD, Judgement::Worse);
        }

        let tuning = stage.chapters.tuning().unwrap();
        assert!(tuning.judged(0, since(700) as i32).is_none());
        assert_eq!(
            tuning
                .judged(0, since(659) as i32)
                .expect("still there")
                .verdict
                .label(),
            "KEEP",
        );
    }

    /// Down to `Rejected` takes it out of the table, up brings it back, and neither
    /// end wraps round.
    #[test]
    fn a_verdict_steps_and_comes_back() {
        let mut stage = Stage::tuned("orb-chapter-verdict-steps");
        let start = stage.chapters.mark;
        stage.play(1200);
        let tl = since(659) as i32;

        let tuning = stage.chapters.tuning().unwrap();
        tuning.judge_down(0, tl);
        assert_eq!(tuning.judged(0, tl).unwrap().verdict.label(), "ADJUST");
        tuning.judge_down(0, tl);
        tuning.judge_down(0, tl);
        assert_eq!(tuning.judged(0, tl).unwrap().verdict.label(), "DROP");
        assert_eq!(tuning.count(0), 2);

        tuning.judge_up(0, tl);
        assert_eq!(tuning.judged(0, tl).unwrap().verdict.label(), "ADJUST");
        tuning.judge_up(0, tl);
        tuning.judge_up(0, tl);
        assert_eq!(tuning.judged(0, tl).unwrap().verdict.label(), "KEEP");
        stage.chapters.mark = start;
        assert_eq!(stage.play(1200), [259, 659, 1059]);
    }

    /// A chapter a boss began has nothing of the table's behind it, and the judging
    /// keys must leave the table alone there rather than reach for the nearest one.
    #[test]
    fn a_chapter_a_boss_began_is_not_judged() {
        let mut stage = Stage::tuned("orb-chapter-boss-not-judged");
        stage.play(1200);

        // Where the stepping leaves the game when it stops on a boss's boundary: on a
        // frame the table has nothing at, in a chapter the game began itself.
        stage.chapters.mark.cause = Cause::BossNonspell;
        let on_it = stage.standing_on(at(700));
        assert!(stage.chapters.judged(&on_it, HELD).is_none());
        stage.chapters.judge(&on_it, HELD, Judgement::Worse);
        stage.chapters.judge(&on_it, HELD, Judgement::Out);

        let tuning = stage.chapters.tuning().unwrap();
        assert_eq!(tuning.count(0), 3);
        for frame in [259, 659, 1059] {
            assert_eq!(
                tuning
                    .judged(0, since(frame) as i32)
                    .unwrap()
                    .verdict
                    .label(),
                "KEEP",
            );
        }
    }

    #[test]
    fn a_boundary_added_by_hand_becomes_a_chapter() {
        let mut stage = Stage::tuned("orb-chapter-hand-becomes-chapter");
        let start = stage.chapters.mark;
        stage.play(1200);

        stage.add_boundary(at(500));
        stage.chapters.mark = start;
        assert_eq!(stage.play(1200), [259, 500, 659, 1059]);
    }

    /// A boundary and the chapter it begins are one thing, so adding one has to leave one
    /// thing: the chapter begins on the boundary's own frame, and the update after it —
    /// which is where the ordinary path would notice the crossing — begins nothing.
    /// Otherwise the number on screen, the frame the flash goes off on and the frame a
    /// step lands on are all one past the number written down, and a later pass over the
    /// same table disagrees with all three.
    #[test]
    fn adding_one_begins_its_chapter_there_and_not_a_frame_later() {
        let mut stage = Stage::tuned("orb-chapter-hand-begins-there");
        stage.play_from(0, 500);

        let number = stage.chapters.number();
        stage.add_boundary(at(499));
        assert_eq!(stage.chapters.number(), number + 1);
        assert_eq!(stage.chapters.started_at(), since(499));
        let on_it = stage.standing_on(at(499));
        assert_eq!(
            stage
                .chapters
                .judged(&on_it, HELD)
                .expect("judged where it is")
                .frame,
            since(499) as i32,
        );
        assert_eq!(stage.play_from(500, 660), [659]);
    }
}
