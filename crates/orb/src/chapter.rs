//! Where chapters begin, and the snapshot taken at each one.
//!
//! Two snapshots are kept: the current chapter and the start of the stage, which
//! are the two things the retry menu offers. Chapter 1 *is* the stage start, so
//! it has no snapshot of its own.
//!
//! Boundaries are found by watching the game rather than by consulting a script,
//! so difficulty differences need no separate handling: a boss with an extra
//! attack on Lunatic simply produces an extra chapter.

use std::fmt;
use std::ops::Range;
use std::path::PathBuf;

use crate::game::{Game, State};
use crate::log::log;
use crate::memtrack;
use crate::profile;
use crate::snapshot::{Audio, Music, Snapshot};
use crate::tuning::{Judged, Tuning, Verdict};

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
    /// `None` while chapter 1 is current, since that is the stage start.
    chapter: Option<Checkpoint>,
    stage_start: Option<Checkpoint>,
    seen: Seen,
    /// Set by a restore: the state jumped, so this frame's differences are not
    /// chapter boundaries.
    resync: bool,
    /// The frame count at which the stage just entered becomes snapshottable.
    settling_until: Option<u32>,
    /// Whether a replay playing back counts as a run to track chapters over.
    during_replay: bool,
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
}

/// Which of a stage's two kinds of fight is on.
#[derive(Clone, Copy, PartialEq)]
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
/// it. What that leans on is being able to go back further than one chapter, which stepping
/// does and the retry menu does not yet.
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
            chapter: None,
            stage_start: None,
            seen: Seen {
                boss_present: false,
                boss_timer: None,
                spellcard: None,
            },
            resync: false,
            settling_until: None,
            during_replay,
            stage_music: None,
            starts: Vec::new(),
            fighting: None,
        }
    }

    pub fn number(&self) -> u32 {
        self.mark.number
    }

    /// The stage frame the chapter now running began at. With the stage, this is what
    /// says which boundary the game is standing on rather than how many have gone by:
    /// numbering starts again at every stage, so a move between stages keeps the number
    /// and changes this.
    pub fn started_at(&self) -> u32 {
        self.mark.started_at
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
        if self.mark.number == 0 {
            return None;
        }
        let kind = Kind::of(self.mark.cause);
        // Counted out of the chapter starts this stage has recorded rather than kept in a
        // counter of its own, because a restore and a step back put the mark back and then
        // play the same chapters again: a counter would make the second time through the
        // same chapter a different one, while these are the frames they began at and the
        // same frame is the same chapter.
        let index = self
            .starts
            .iter()
            .filter(|(at, cause)| *at <= self.mark.started_at && Kind::of(*cause) == kind)
            .count();
        Some(Name { kind, index })
    }

    pub fn retries(&self) -> u32 {
        self.retries
    }

    pub fn can_retry(&self) -> bool {
        self.stage_start.is_some()
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

    /// Whether a chapter's start is still one, which for the table's own boundaries is
    /// whether the table still has it and still carries it.
    ///
    /// Gone counts as not one: refusing a boundary put there by hand takes it out
    /// altogether rather than remembering it as refused, and a chapter start that outlived
    /// the boundary it came from is a place the stepping would stop at for no reason —
    /// `chapter_dropped_key` cannot reach it either, since it is in no table to be found in.
    /// Whether this boundary of the current stage is one somebody put there.
    fn added_by_hand(&self, frame: i32) -> bool {
        let Midstage::Tuning(tuning) = &self.midstage else {
            return false;
        };
        let Some(stage) = self.stage else {
            return false;
        };
        tuning
            .judged(stage, frame)
            .is_some_and(|judged| judged.by_hand)
    }

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

        if self.stage != Some(state.stage) {
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
            let music_ready = game.music().is_some();
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
                .copied()
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
        // *midboss*, which called that fight the stage's own.
        let boss_appeared = current.boss_present && !self.seen.boss_present;
        if current.boss_present {
            let fight = self.fighting.get_or_insert(Fight::Midboss);
            if game.music_identity() != self.stage_music {
                *fight = Fight::Stage;
            }
        }
        // Beating a midboss hands the stage back, and the waves that follow want a
        // chapter of their own — it is the only boundary between the fight and them.
        // Beating the stage's own boss is not a boundary: the stage is over, and a
        // chapter whose first act is that the fight is already won has nothing to
        // retry. It carved two chapters a second long out of the teardown as well.
        let beaten = !current.boss_present && self.seen.boss_present;
        let midboss_beaten = beaten && self.fighting == Some(Fight::Midboss);
        if !current.boss_present {
            self.fighting = None;
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
                self.mark.cause = named;
                if let Ok(at) = self
                    .starts
                    .binary_search_by_key(&self.mark.started_at, |(at, _)| *at)
                {
                    self.starts[at].1 = named;
                }
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
        let by_hand = midstage.is_some_and(|frame| self.added_by_hand(frame));
        if elapsed < MIN_CHAPTER_FRAMES && !by_hand {
            return Due::No;
        }
        // The boss timer is reset to zero every time the script moves to the
        // next attack, spell or not.
        let attack_changed = current.boss_present
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
        self.starts.clear();
        self.starts.push((state.stage_frames, Cause::StageStart));
        if let Midstage::Tuning(tuning) = &mut self.midstage {
            // The stage just finished is worth keeping even if the pass stops here.
            tuning.write(game);
            tuning.begin_stage(state.stage);
        }
        self.chapter = None;

        let snapshot = unsafe {
            take(
                self.stage_start.take(),
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
        self.stage_start = Some(Checkpoint {
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
        let snapshot = unsafe { take(self.chapter.take(), game, data, self_check, audio, &live) };
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
        self.chapter = Some(Checkpoint {
            snapshot,
            mark: self.mark,
        });
    }

    /// Restores the start of the current chapter. Returns false when there is
    /// nothing to go back to.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retry_chapter(&mut self, game: &dyn Game) -> bool {
        let Some(checkpoint) = self.chapter.as_ref().or(self.stage_start.as_ref()) else {
            return false;
        };
        let mark = checkpoint.mark;
        unsafe { checkpoint.snapshot.restore(game) };
        self.after_restore(mark);
        self.retries += 1;
        log!("retry chapter {} (retry {})", mark.number, self.retries);
        true
    }

    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retry_stage(&mut self, game: &dyn Game) -> bool {
        if !unsafe { self.rewind_stage(game) } {
            return false;
        }
        self.retries += 1;
        log!(
            "retry stage from chapter {} (retry {})",
            self.mark.number,
            self.retries
        );
        true
    }

    /// Puts the stage back to its start without counting a retry, for stepping
    /// back through the chapters: replaying part of a stage to reach a boundary is
    /// not an attempt at the stage.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn rewind_stage(&mut self, game: &dyn Game) -> bool {
        let Some(checkpoint) = self.stage_start.as_ref() else {
            return false;
        };
        let mark = checkpoint.mark;
        unsafe { checkpoint.snapshot.restore(game) };
        self.chapter = None;
        self.after_restore(mark);
        true
    }

    /// A boss's own music is left running across a retry: rewinding a long fight's
    /// theme every attempt is worse than the jump in it. A midboss plays the
    /// stage's music, and that is rewound like anything else midstage.
    fn audio_for(&self, game: &dyn Game, state: &State) -> Audio {
        let identity = game.music_identity();
        let boss_has_own_music = state.boss_present && identity != self.stage_music;
        let policy = if boss_has_own_music {
            Music::KeepPlaying
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

    fn after_restore(&mut self, mark: Mark) {
        self.mark = mark;
        self.resync = true;
    }

    /// Writes out whatever the tuning pass has found, if one is running.
    pub fn save_tuning(&self, game: &dyn Game) {
        if let Midstage::Tuning(tuning) = &self.midstage {
            tuning.write(game);
        }
    }

    /// Drops everything on leaving the run: the snapshots describe a stage that
    /// is no longer loaded, and the retry count belongs to that attempt.
    pub fn forget(&mut self) {
        if self.stage.is_some() {
            log!("run ended after {} retries", self.retries);
        }
        self.stage = None;
        self.retries = 0;
        self.chapter = None;
        self.stage_start = None;
        self.settling_until = None;
        self.mark = Mark {
            number: 0,
            started_at: 0,
            midstage_upto: i32::MIN,
            cause: Cause::StageStart,
        };
        self.starts.clear();
    }
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

#[cfg(test)]
mod tests {
    use super::{Cause, Chapters, Due, Judgement, Mark, Seen};
    use crate::game::State;
    use crate::game::th06::Th06;
    use std::path::PathBuf;

    /// Judging happens with the game held on the boundary, which is the only way the keys
    /// work: a frame is a sixtieth of a second, and one pressed while the game runs would
    /// land on whichever frame it reached.
    const HELD: bool = true;

    /// Two hundred frames of enemies and two hundred without, over and over: a
    /// stage's waves in the only terms the detector reads. The stage's clock and the
    /// script's are held equal, since all it asks of them is that both advance.
    fn waves(frame: u32) -> State {
        State {
            enemy_count: if (frame / 200).is_multiple_of(2) {
                3
            } else {
                0
            },
            ..empty(frame)
        }
    }

    /// A frame with nothing on screen at all. The stage's clock and the script's are
    /// held equal, since all the boundary detection asks of them is that both
    /// advance.
    fn empty(frame: u32) -> State {
        State {
            scene: 2,
            playing: true,
            in_run: true,
            in_game: true,
            in_ending: false,
            ending_script: None,
            demo: false,
            replay: false,
            practice: false,
            paused: false,
            unsettled: false,
            bombing: false,
            in_dialogue: false,
            stage: 0,
            difficulty: 1,
            stage_frames: frame,
            script_frames: frame as i32,
            random_seed: 0,
            deaths: 0,
            lives: 2,
            bombs: 3,
            power: 128,
            enemy_count: 0,
            bullet_count: 0,
            laser_count: 0,
            boss_present: false,
            boss_life: None,
            boss_attack_frames: None,
            spellcard: None,
        }
    }

    fn tuning_chapters() -> Chapters {
        // The directory is only reached by `write` and by the read at startup, which
        // most tests here want no part of.
        Chapters::new(&Th06, Some(PathBuf::new()), false)
    }

    /// A directory of its own, so that tests writing files do not read each other's.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::remove_file(dir.join("tuning.txt")).ok();
        dir
    }

    /// What `begin_stage` does to the bookkeeping, without the snapshot it also
    /// takes: there is no game to capture the memory of.
    fn begin_stage(chapters: &mut Chapters, state: &State) {
        chapters.stage = Some(state.stage);
        chapters.mark = Mark {
            number: 1,
            started_at: state.stage_frames,
            midstage_upto: state.script_frames,
            cause: Cause::StageStart,
        };
        chapters.starts.clear();
        chapters
            .starts
            .push((state.stage_frames, Cause::StageStart));
        if let Some(tuning) = chapters.tuning() {
            tuning.begin_stage(state.stage);
        }
    }

    /// What `add_boundary` does, without the snapshot it also takes: the chapter begins on
    /// the boundary's own frame.
    fn add_boundary(chapters: &mut Chapters, state: &State) {
        assert!(chapters.note_boundary(state), "the stage takes boundaries");
        let cause = Cause::Boundary(state.script_frames);
        chapters.mark = Mark {
            number: chapters.mark.number + 1,
            started_at: state.stage_frames,
            midstage_upto: chapters.mark.midstage_upto,
            cause,
        };
        let at = chapters
            .starts
            .binary_search_by_key(&state.stage_frames, |(at, _)| *at);
        if let Err(at) = at {
            chapters.starts.insert(at, (state.stage_frames, cause));
        }
    }

    /// What `observe` does around the boundary detection, without the snapshot it also
    /// takes: the guard turns a frame away, and holds the boss being there while it does.
    fn watch(chapters: &mut Chapters, state: &State) -> Due {
        let current = Seen::of(state);
        if super::guarded(state) {
            return Due::No;
        }
        let due = chapters.due(&Th06, state, &current);
        chapters.seen = current;
        due
    }

    /// Plays a stage through the boundary detection as the frame hook does, and
    /// reports the script frame each chapter after the first began at.
    fn play(chapters: &mut Chapters, frames: u32) -> Vec<i32> {
        play_from(chapters, 0, frames)
    }

    fn play_from(chapters: &mut Chapters, from: u32, frames: u32) -> Vec<i32> {
        let mut begun = Vec::new();
        for frame in from..frames {
            let state = waves(frame);
            let Due::Yes(cause) = chapters.due(&Th06, &state, &Seen::of(&state)) else {
                continue;
            };
            chapters.mark = Mark {
                number: chapters.mark.number + 1,
                started_at: state.stage_frames,
                midstage_upto: chapters.mark.midstage_upto,
                cause,
            };
            let at = chapters
                .starts
                .binary_search_by_key(&state.stage_frames, |(at, _)| *at);
            if let Err(at) = at {
                chapters.starts.insert(at, (state.stage_frames, cause));
            }
            begun.push(state.script_frames);
        }
        begun
    }

    /// A frame of a fight, with the boss's timer and whichever attack it is on.
    fn fight(frame: u32, timer: i32, spellcard: Option<u32>) -> State {
        State {
            boss_present: true,
            boss_attack_frames: Some(timer),
            spellcard,
            ..empty(frame)
        }
    }

    /// A frame with enemies on it, for waiting out the floor on a chapter's length without
    /// the gap detector offering a boundary in the quiet.
    fn enemies(frame: u32) -> State {
        State {
            enemy_count: 3,
            ..empty(frame)
        }
    }

    /// One frame through the detection, with the bookkeeping a chapter beginning does and
    /// without the snapshot: what the frame hook does, as far as the name is concerned.
    fn step(chapters: &mut Chapters, state: &State) {
        let Due::Yes(cause) = watch(chapters, state) else {
            return;
        };
        chapters.mark = Mark {
            number: chapters.mark.number + 1,
            started_at: state.stage_frames,
            midstage_upto: chapters.mark.midstage_upto,
            cause,
        };
        if let Err(at) = chapters
            .starts
            .binary_search_by_key(&state.stage_frames, |(at, _)| *at)
        {
            chapters.starts.insert(at, (state.stage_frames, cause));
        }
    }

    fn named(chapters: &Chapters) -> String {
        chapters.name().expect("a stage is running").to_string()
    }

    /// What the status line and the retry menu call each chapter, in the order a stage
    /// reaches them: numbered inside the part of the stage it belongs to rather than counted
    /// straight through, and the waves picking their own count up again after the midboss.
    #[test]
    fn a_chapter_is_named_for_the_part_of_the_stage_it_is_in() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        assert_eq!(named(&chapters), "MIDSTAGE 1");

        // A gap in the waves, which the detector offers at 259.
        for frame in 0..300 {
            step(&mut chapters, &waves(frame));
        }
        assert_eq!(named(&chapters), "MIDSTAGE 2");

        // A midboss, whose first attack starts with it. There is no game here to play a
        // stage's track, so which fight it is comes from `stage_music` being the same
        // nothing that `music_identity` reports.
        step(&mut chapters, &fight(320, 0, None));
        assert_eq!(named(&chapters), "MIDBOSS NONSPELL 1");

        for frame in 321..380 {
            step(&mut chapters, &fight(frame, frame as i32 - 320, None));
        }
        step(&mut chapters, &fight(380, 0, Some(1)));
        assert_eq!(named(&chapters), "MIDBOSS SPELL 1");

        for frame in 381..441 {
            step(&mut chapters, &fight(frame, frame as i32 - 380, Some(1)));
        }
        // Beaten, which hands the stage back.
        step(&mut chapters, &enemies(441));
        assert_eq!(named(&chapters), "MIDSTAGE 3");

        // The stage's own boss brings the second of the two songs the stage names, which is
        // what tells it from a midboss.
        chapters.stage_music = Some(0x51a7);
        for frame in 442..511 {
            step(&mut chapters, &enemies(frame));
        }
        step(&mut chapters, &fight(511, 0, None));
        assert_eq!(named(&chapters), "BOSS NONSPELL 1");
    }

    /// A chapter reached twice is the same chapter. The count comes out of the frames this
    /// stage's chapters began at, not out of a counter per kind: a retry puts the mark back
    /// and the run then reaches the same chapters again, which a counter would name one
    /// further along every time.
    #[test]
    fn a_chapter_reached_again_keeps_its_name() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        for frame in 0..300 {
            step(&mut chapters, &waves(frame));
        }
        let boundary = chapters.mark;
        step(&mut chapters, &fight(320, 0, None));
        assert_eq!(named(&chapters), "MIDBOSS NONSPELL 1");

        // What a retry does to orb's own bookkeeping: the mark goes back with the memory.
        chapters.mark = boundary;
        chapters.seen = Seen::of(&waves(299));
        assert_eq!(named(&chapters), "MIDSTAGE 2");
        step(&mut chapters, &fight(320, 0, None));
        assert_eq!(named(&chapters), "MIDBOSS NONSPELL 1");
    }

    /// One put there by hand begins its chapter however close behind the last one it is. The
    /// shortest a chapter may be is there to stop a boss's flurry of script transitions
    /// carving out chapters a fraction of a second long, and a hand is not that: dropping it
    /// on the passes after the one it was added on would lose what somebody wrote down, and
    /// silently, since the boundary would still be in the table.
    #[test]
    fn one_added_by_hand_is_not_too_close_to_the_last() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        assert_eq!(play_from(&mut chapters, 0, 300), [259]);

        // Fifty-four frames after it, which is inside the floor.
        add_boundary(&mut chapters, &empty(313));
        // And it is still a boundary on the next pass over the same ground.
        chapters.mark = Mark {
            midstage_upto: 0,
            ..chapters.mark
        };
        let at = waves(313);
        assert!(matches!(
            chapters.due(&Th06, &at, &Seen::of(&at)),
            Due::Yes(Cause::Boundary(313))
        ));
    }

    /// A spellcard that names itself after the chapter its attack began takes that chapter's
    /// name rather than being lost: the boss's timer resets on one frame and the card is
    /// declared on the next, and the shortest a chapter may be would refuse a second chapter
    /// for the same attack.
    #[test]
    fn a_spellcard_named_late_still_names_its_chapter() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let fighting = |frame, timer, spell| State {
            boss_present: true,
            boss_attack_frames: Some(timer),
            spellcard: spell,
            ..empty(frame)
        };

        for frame in 0..200 {
            watch(&mut chapters, &fighting(frame, frame as i32, None));
        }
        // The timer starts again, which is the attack changing and all that is known of it.
        let due = watch(&mut chapters, &fighting(200, 0, None));
        assert!(matches!(
            due,
            Due::Yes(Cause::MidbossNonspell | Cause::BossNonspell)
        ));
        let Due::Yes(cause) = due else { unreachable!() };
        chapters.mark = Mark {
            number: 2,
            started_at: 200,
            cause,
            ..chapters.mark
        };

        // The card names itself on the next frame. No second chapter, and the one standing
        // there is a spellcard's.
        let due = watch(&mut chapters, &fighting(201, 1, Some(3)));
        assert!(matches!(due, Due::No));
        assert!(matches!(
            chapters.cause(),
            Cause::MidbossSpell | Cause::BossSpell
        ));
    }

    /// A boundary judged out of the table is not a chapter start, so the stepping passes
    /// over it — and it is not gone either, so `chapter_dropped_key` with a stepping key
    /// still reaches it. In script frames, since nothing has a stage frame for a boundary
    /// that begins no chapter.
    #[test]
    fn the_stepping_passes_over_a_boundary_judged_out_and_the_dropped_key_reaches_it() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        assert_eq!(play(&mut chapters, 1200), [259, 659, 1059]);
        chapters.tuning().unwrap().reject(0, 659);

        let at = empty(300);
        assert_eq!(chapters.next_start(&at), Some(1059));
        assert_eq!(chapters.previous_start(&empty(1200)), Some(1059));
        assert_eq!(chapters.next_dropped(&at), Some(659));
        assert_eq!(chapters.previous_dropped(&empty(1200)), Some(659));
        // Taken back, it is a chapter start again and no longer the dropped key's.
        chapters.judge(&empty(659), HELD, Judgement::Better);
        assert_eq!(chapters.next_start(&at), Some(659));
        assert_eq!(chapters.next_dropped(&at), None);
    }

    /// One put there by hand and then taken out is gone from the table altogether, so the
    /// chapter it began is no longer a place the stepping stops at either. Nothing can be
    /// judged there and `chapter_dropped_key` cannot find it, so stopping would be stopping
    /// at nothing.
    #[test]
    fn the_stepping_passes_over_a_boundary_taken_out_by_hand() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        play(&mut chapters, 1200);

        let at = empty(500);
        add_boundary(&mut chapters, &at);
        assert_eq!(chapters.next_start(&empty(300)), Some(500));

        chapters.judge(&at, HELD, Judgement::Worse);
        chapters.judge(&at, HELD, Judgement::Worse);
        assert!(chapters.judged(&at, HELD).is_none(), "taken out altogether");
        assert_eq!(chapters.next_start(&empty(300)), Some(659));
        assert_eq!(chapters.previous_start(&empty(600)), Some(259));
        assert_eq!(chapters.next_dropped(&empty(300)), None);
    }

    /// A fight ending on the frame the player is hit still ends, on that frame: stage 6's
    /// midboss went down as the player died, and dying is not a reason to pass a boundary
    /// over — the way out of a start that kills whoever restores it is the chapter before it.
    #[test]
    fn a_fight_that_ends_as_the_player_dies_still_ends() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));

        // A midboss, up long enough for a chapter to be allowed after it.
        for frame in 0..200 {
            watch(
                &mut chapters,
                &State {
                    boss_present: true,
                    ..empty(frame)
                },
            );
        }
        let hit = State {
            unsettled: true,
            ..empty(200)
        };
        assert!(matches!(
            watch(&mut chapters, &hit),
            Due::Yes(Cause::StageAfterMidboss)
        ));
    }

    /// An attack changing under a bomb is a boundary on the frame it changes: a bomb is two
    /// seconds long, and the frame the fight moved on is the one worth going back to.
    #[test]
    fn an_attack_that_changes_under_a_bomb_is_a_boundary_there() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let spell = |frame, timer| State {
            boss_present: true,
            boss_life: Some(3000),
            boss_attack_frames: Some(timer),
            spellcard: Some(7),
            ..empty(frame)
        };

        for frame in 0..200 {
            watch(&mut chapters, &spell(frame, frame as i32));
        }
        // The spellcard is bombed out and the next attack begins while the bomb is still
        // going: the boss's timer starts again from nothing.
        let bombed = State {
            bombing: true,
            ..spell(200, 0)
        };
        // Whose fight it is comes from the music, which there is no game here to play, so
        // either naming will do — what matters is that the change was taken.
        let due = watch(&mut chapters, &bombed);
        assert!(matches!(
            due,
            Due::Yes(Cause::BossSpell | Cause::MidbossSpell)
        ));
    }

    /// The judging keys do nothing unless the game is standing still on the boundary: a
    /// frame is a sixtieth of a second, and a key pressed while the game runs would land on
    /// whichever frame it reached.
    #[test]
    fn judging_needs_the_game_held_on_the_boundary() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        play(&mut chapters, 1200);

        let on_it = empty(659);
        assert!(chapters.judged(&on_it, false).is_none());
        chapters.judge(&on_it, false, Judgement::Worse);
        assert_eq!(
            chapters.judged(&on_it, HELD).unwrap().verdict.label(),
            "KEEP"
        );

        // Nor anywhere else, held or not: what is judged is the boundary underneath, and
        // inside a chapter there is none.
        let past_it = empty(700);
        assert!(chapters.judged(&past_it, HELD).is_none());
        chapters.judge(&past_it, HELD, Judgement::Worse);
        assert_eq!(
            chapters.judged(&on_it, HELD).unwrap().verdict.label(),
            "KEEP"
        );
    }

    /// A fight underway outranks the table: the enemy timeline runs on through a midboss,
    /// so a fight that drags reaches the boundaries of the waves after it, and a chapter
    /// beginning half way through a fight is a retry point for neither. The entry is spent
    /// where it falls, so the fight's end is the one boundary there.
    #[test]
    fn a_boundary_reached_during_a_fight_is_not_one() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        // The detector's boundary for the first gap, found in a run with no fight in it.
        assert_eq!(play(&mut chapters, 400), [259]);

        // The same stage again, with a midboss on screen from before that boundary until
        // after it. Nothing of the table's begins a chapter while it is up.
        let mut fought = tuning_chapters();
        begin_stage(&mut fought, &empty(0));
        fought.tuning().unwrap().add(0, 259);
        let fight = |frame| State {
            boss_present: true,
            boss_attack_frames: Some(frame as i32),
            ..empty(frame)
        };
        for frame in 0..400 {
            let state = fight(frame);
            let current = Seen::of(&state);
            let due = fought.due(&Th06, &state, &current);
            assert!(
                !matches!(due, Due::Yes(Cause::Boundary(_))),
                "the table began a chapter at frame {frame}, inside the fight",
            );
            fought.seen = current;
        }
        // And it is spent where it fell rather than saved up: what the frame the fight ends
        // on has is the fight's own end, not the table's entry firing as its double.
        let after = empty(400);
        assert!(matches!(
            fought.due(&Th06, &after, &Seen::of(&after)),
            Due::Yes(Cause::StageAfterMidboss)
        ));
    }

    /// A second into each gap, and once per gap however long it runs on: the wave
    /// leaves at 200 and the boundary falls at 259, the next at 659.
    #[test]
    fn a_gap_between_waves_becomes_one_chapter() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        assert_eq!(play(&mut chapters, 1200), [259, 659, 1059]);
    }

    #[test]
    fn a_stage_played_twice_divides_the_same_way() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let start = chapters.mark;
        let first = play(&mut chapters, 1200);

        // What stepping back through the chapters does: the stage's start comes
        // back and part of the stage is played again.
        chapters.mark = start;
        assert_eq!(play(&mut chapters, 1200), first);
        // And each boundary is written down once, however often it is played over.
        assert_eq!(chapters.tuning().unwrap().count(0), first.len());
    }

    #[test]
    fn a_boundary_removed_by_hand_does_not_come_back() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let start = chapters.mark;
        play(&mut chapters, 1200);

        chapters.tuning().unwrap().reject(0, 659);
        chapters.mark = start;
        assert_eq!(play(&mut chapters, 1200), [259, 1059]);
    }

    /// The detector's own proposals are kept when they are judged out, and kept as
    /// the detector's: forgetting one would have it proposed again the next time the
    /// stage was played, and the same boundary would have to be judged out every
    /// session.
    #[test]
    fn a_boundary_judged_out_stays_out_in_the_next_session() {
        let dir = scratch("orb-dropped-stays-dropped");
        let mut chapters = Chapters::new(&Th06, Some(dir.clone()), false);
        begin_stage(&mut chapters, &empty(0));
        assert_eq!(play(&mut chapters, 1200), [259, 659, 1059]);

        let tuning = chapters.tuning().unwrap();
        tuning.reject(0, 659);
        tuning.write(&Th06);

        let mut next = Chapters::new(&Th06, Some(dir), false);
        begin_stage(&mut next, &empty(0));
        // Not proposed again, and not because the detector was suppressed: the stage
        // is played from the top with the frames it covers all new to this pass.
        assert_eq!(play(&mut next, 1200), [259, 1059]);
        let judged = next.tuning().unwrap().judged(0, 659).expect("still known");
        assert_eq!(judged.verdict.label(), "DROP");
        assert!(!judged.by_hand);
    }

    /// Stepping onto a boundary judged out is how it gets taken back, so the frame the
    /// game is on has to be what says which boundary is being looked at: a refused one
    /// no longer begins the chapter standing there.
    #[test]
    fn a_boundary_judged_out_can_be_taken_back_from_where_it_is() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        play(&mut chapters, 1200);
        let at = empty(659);
        chapters.judge(&at, HELD, Judgement::Out);

        assert_eq!(chapters.judged(&at, HELD).unwrap().verdict.label(), "DROP");
        chapters.judge(&at, HELD, Judgement::Better);
        assert_eq!(
            chapters.judged(&at, HELD).unwrap().verdict.label(),
            "ADJUST"
        );
        chapters.judge(&at, HELD, Judgement::Better);
        assert_eq!(chapters.judged(&at, HELD).unwrap().verdict.label(), "KEEP");
    }

    /// One put there by hand goes altogether instead: nothing proposes it again, so
    /// there is nothing for a refusal to hold back, and `tuning_add_key` is the way
    /// back.
    #[test]
    fn a_boundary_added_by_hand_is_taken_out_altogether() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let start = chapters.mark;
        play(&mut chapters, 1200);

        let at = empty(500);
        add_boundary(&mut chapters, &at);
        let judged = chapters
            .judged(&at, HELD)
            .expect("the added boundary is the one to judge");
        assert_eq!(judged.verdict.label(), "KEEP");
        assert!(judged.by_hand);

        chapters.judge(&at, HELD, Judgement::Out);
        assert!(chapters.judged(&at, HELD).is_none());
        chapters.mark = start;
        assert_eq!(play(&mut chapters, 1200), [259, 659, 1059]);
    }

    /// A hand and the detector looking at the same lull must not leave two boundaries a
    /// frame apart. The hand goes down when the screen empties, the detector a second
    /// into it, so one put there by hand just before is the near miss to rule out.
    #[test]
    fn a_boundary_added_by_hand_leaves_no_room_for_one_beside_it() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        // The gap opens at 200 and the detector's own boundary for it falls at 259.
        assert_eq!(play_from(&mut chapters, 0, 259), []);
        add_boundary(&mut chapters, &waves(258));
        play_from(&mut chapters, 259, 1200);

        let tuning = chapters.tuning().unwrap();
        assert!(
            tuning
                .judged(0, 258)
                .expect("the one put there by hand")
                .by_hand
        );
        assert!(tuning.judged(0, 259).is_none());
        // The gaps after it are still the detector's to propose.
        assert_eq!(tuning.count(0), 3);
    }

    /// Two presses take one out; the third must find nothing rather than start taking out
    /// the boundary the chapter began at, which is frames back with nothing on screen to
    /// say it is what the key is about to change.
    #[test]
    fn taking_one_out_does_not_reach_the_boundary_behind_it() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        play(&mut chapters, 1200);

        let at = empty(700);
        add_boundary(&mut chapters, &at);
        for _ in 0..4 {
            chapters.judge(&at, HELD, Judgement::Worse);
        }

        let tuning = chapters.tuning().unwrap();
        assert!(tuning.judged(0, 700).is_none());
        assert_eq!(
            tuning.judged(0, 659).expect("still there").verdict.label(),
            "KEEP"
        );
    }

    /// Down to `Rejected` takes it out of the table, up brings it back, and neither
    /// end wraps round.
    #[test]
    fn a_verdict_steps_and_comes_back() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let start = chapters.mark;
        play(&mut chapters, 1200);

        let tuning = chapters.tuning().unwrap();
        tuning.judge_down(0, 659);
        assert_eq!(tuning.judged(0, 659).unwrap().verdict.label(), "ADJUST");
        tuning.judge_down(0, 659);
        tuning.judge_down(0, 659);
        assert_eq!(tuning.judged(0, 659).unwrap().verdict.label(), "DROP");
        assert_eq!(tuning.count(0), 2);

        tuning.judge_up(0, 659);
        assert_eq!(tuning.judged(0, 659).unwrap().verdict.label(), "ADJUST");
        tuning.judge_up(0, 659);
        tuning.judge_up(0, 659);
        assert_eq!(tuning.judged(0, 659).unwrap().verdict.label(), "KEEP");
        chapters.mark = start;
        assert_eq!(play(&mut chapters, 1200), [259, 659, 1059]);
    }

    /// A chapter a boss began has nothing of the table's behind it, and the judging
    /// keys must leave the table alone there rather than reach for the nearest one.
    #[test]
    fn a_chapter_a_boss_began_is_not_judged() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        play(&mut chapters, 1200);

        // Where the stepping leaves the game when it stops on a boss's boundary: on a
        // frame the table has nothing at, in a chapter the game began itself.
        chapters.mark.cause = Cause::BossNonspell;
        let at = empty(700);
        assert!(chapters.judged(&at, HELD).is_none());
        chapters.judge(&at, HELD, Judgement::Worse);
        chapters.judge(&at, HELD, Judgement::Out);

        let tuning = chapters.tuning().unwrap();
        assert_eq!(tuning.count(0), 3);
        for frame in [259, 659, 1059] {
            assert_eq!(tuning.judged(0, frame).unwrap().verdict.label(), "KEEP");
        }
    }

    #[test]
    fn a_boundary_added_by_hand_becomes_a_chapter() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        let start = chapters.mark;
        play(&mut chapters, 1200);

        add_boundary(&mut chapters, &empty(500));
        chapters.mark = start;
        assert_eq!(play(&mut chapters, 1200), [259, 500, 659, 1059]);
    }

    /// A boundary and the chapter it begins are one thing, so adding one has to leave one
    /// thing: the chapter begins on the boundary's own frame, and the update after it —
    /// which is where the ordinary path would notice the crossing — begins nothing.
    /// Otherwise the number on screen, the frame the flash goes off on and the frame a
    /// step lands on are all one past the number written down, and a later pass over the
    /// same table disagrees with all three.
    #[test]
    fn adding_one_begins_its_chapter_there_and_not_a_frame_later() {
        let mut chapters = tuning_chapters();
        begin_stage(&mut chapters, &empty(0));
        play_from(&mut chapters, 0, 500);

        let number = chapters.number();
        add_boundary(&mut chapters, &empty(499));
        assert_eq!(chapters.number(), number + 1);
        assert_eq!(chapters.started_at(), 499);
        assert_eq!(
            chapters
                .judged(&empty(499), HELD)
                .expect("judged where it is")
                .frame,
            499
        );
        assert_eq!(play_from(&mut chapters, 500, 660), [659]);
    }
}
