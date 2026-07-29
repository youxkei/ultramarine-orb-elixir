//! Where chapters begin, and the snapshot taken at each one.
//!
//! Two snapshots are kept: the current chapter and the start of the stage, which
//! are the two things the retry menu offers. Chapter 1 *is* the stage start, so
//! it has no snapshot of its own.
//!
//! Boundaries are found by watching the game rather than by consulting a script,
//! so difficulty differences need no separate handling: a boss with an extra
//! attack on Lunatic simply produces an extra chapter.

use std::ops::Range;
use std::path::PathBuf;

use crate::game::{Game, State};
use crate::log::log;
use crate::memtrack;
use crate::profile;
use crate::snapshot::{Audio, Music, Snapshot};
use crate::tuning::Tuning;

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
    /// How far through this stage's midstage table the chapter began.
    next_midstage: usize,
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
            mark: Mark { number: 0, started_at: 0, next_midstage: 0 },
            chapter: None,
            stage_start: None,
            seen: Seen { boss_present: false, boss_timer: None, spellcard: None },
            resync: false,
            settling_until: None,
            during_replay,
            stage_music: None,
        }
    }

    pub fn number(&self) -> u32 {
        self.mark.number
    }

    pub fn retries(&self) -> u32 {
        self.retries
    }

    pub fn can_retry(&self) -> bool {
        self.stage_start.is_some()
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
        self.seen = current;
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
        // A replay drives the same gameplay scene, which is what makes it usable
        // to build the midstage table or to exercise a snapshot repeatedly.
        let tracking = state.in_game || (self.during_replay && state.playing && !state.demo);
        if !tracking {
            if self.stage.is_some() {
                // The run is over; keep whatever the tuning pass found.
                self.save_tuning(game);
            }
            self.forget();
            return;
        }
        // Mid-death, mid-dialogue or paused is no place to restart from, and the
        // game is not really progressing either.
        if state.unsettled || state.in_dialogue || state.paused {
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
        if !self.resync && self.due(game, state, current) {
            unsafe { self.begin_chapter(game, state, data, self_check) };
        }
    }

    /// Advances the midstage cursor as a side effect: an entry is spent whether
    /// or not it turns out to start a chapter.
    fn due(&mut self, game: &dyn Game, state: &State, current: &Seen) -> bool {
        let elapsed = state.stage_frames.saturating_sub(self.mark.started_at);
        let midstage = match &mut self.midstage {
            Midstage::Table => {
                let table = game.midstage(state.stage);
                let mut due = false;
                while self.mark.next_midstage < table.len()
                    && table[self.mark.next_midstage] <= state.script_frames
                {
                    self.mark.next_midstage += 1;
                    due = true;
                }
                due
            }
            Midstage::Tuning(tuning) => tuning.is_boundary(state, elapsed),
        };

        if elapsed < MIN_CHAPTER_FRAMES {
            return false;
        }
        let boss_appeared = current.boss_present && !self.seen.boss_present;
        // Beating one ends a fight and hands the stage back, which is as much a
        // fresh start as the fight beginning was — and for a midboss it is the
        // only boundary between it and whatever the stage does next.
        let boss_beaten = !current.boss_present && self.seen.boss_present;
        // The boss timer is reset to zero every time the script moves to the
        // next attack, spell or not.
        let attack_changed = current.boss_present
            && matches!(
                (self.seen.boss_timer, current.boss_timer),
                (Some(before), Some(now)) if now < before
            );
        let spellcard_changed = current.spellcard != self.seen.spellcard;

        midstage || boss_appeared || boss_beaten || attack_changed || spellcard_changed
    }

    unsafe fn begin_stage(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
    ) {
        self.mark = Mark { number: 1, started_at: state.stage_frames, next_midstage: 0 };
        self.stage_music = game.music_identity();
        if let Midstage::Tuning(tuning) = &mut self.midstage {
            // The stage just finished is worth keeping even if the pass stops here.
            tuning.write(game);
            tuning.begin_stage(state.stage);
        }
        self.chapter = None;

        let snapshot =
            unsafe {
                take(
                    self.stage_start.take(),
                    data,
                    self_check,
                    Audio {
                        policy: Music::Rewind(game.music()),
                        identity: game.music_identity(),
                        state: game.audio_state(),
                        thread: game.audio_thread(),
                    },
                )
            };
        log!(
            "stage {} chapter 1 (stage start) at frame {}, music: {}",
            state.stage + 1,
            state.stage_frames,
            snapshot.has_music(),
        );
        self.stage_start = Some(Checkpoint { snapshot, mark: self.mark });
    }

    unsafe fn begin_chapter(
        &mut self,
        game: &dyn Game,
        state: &State,
        data: &Range<usize>,
        self_check: bool,
    ) {
        self.mark = Mark {
            number: self.mark.number + 1,
            started_at: state.stage_frames,
            next_midstage: self.mark.next_midstage,
        };
        let audio = self.audio_for(game, state);
        let keeps_music = matches!(audio.policy, Music::KeepPlaying);
        let snapshot = unsafe { take(self.chapter.take(), data, self_check, audio) };
        log!(
            "stage {} chapter {} at frame {} (script {}){}, music={}",
            state.stage + 1,
            self.mark.number,
            state.stage_frames,
            state.script_frames,
            if state.boss_present { ", boss" } else { "" },
            if keeps_music { "keep" } else { "rewind" },
        );
        self.chapter = Some(Checkpoint { snapshot, mark: self.mark });
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
        unsafe { checkpoint.snapshot.restore(game.music_identity()) };
        self.after_restore(mark);
        log!("retry chapter {} (retry {})", mark.number, self.retries);
        true
    }

    /// # Safety
    /// Must run on the game's main thread, between frames.
    pub unsafe fn retry_stage(&mut self, game: &dyn Game) -> bool {
        let Some(checkpoint) = self.stage_start.as_ref() else { return false };
        let mark = checkpoint.mark;
        unsafe { checkpoint.snapshot.restore(game.music_identity()) };
        self.chapter = None;
        self.after_restore(mark);
        log!("retry stage from chapter {} (retry {})", mark.number, self.retries);
        true
    }

    /// A boss's own music is left running across a retry: rewinding a long fight's
    /// theme every attempt is worse than the jump in it. A midboss plays the
    /// stage's music, and that is rewound like anything else midstage.
    fn audio_for(&self, game: &dyn Game, state: &State) -> Audio {
        let identity = game.music_identity();
        let boss_has_own_music = state.boss_present && identity != self.stage_music;
        let policy =
            if boss_has_own_music { Music::KeepPlaying } else { Music::Rewind(game.music()) };
        Audio { policy, identity, state: game.audio_state(), thread: game.audio_thread() }
    }

    fn after_restore(&mut self, mark: Mark) {
        self.mark = mark;
        self.retries += 1;
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
        self.mark = Mark { number: 0, started_at: 0, next_midstage: 0 };
    }
}

/// Reuses `previous`'s buffers when there is one, so a boundary costs a copy
/// rather than a copy plus several megabytes of fresh pages.
unsafe fn take(
    previous: Option<Checkpoint>,
    data: &Range<usize>,
    self_check: bool,
    audio: Audio,
) -> Snapshot {
    let started = profile::now();
    let regions = unsafe { memtrack::regions(data.clone()) };
    let snapshot = match previous {
        Some(checkpoint) => {
            let mut snapshot = checkpoint.snapshot;
            unsafe { snapshot.update(&regions, audio, self_check) };
            snapshot
        }
        None => unsafe { Snapshot::capture(&regions, audio, self_check) },
    };
    unsafe { profile::record(profile::Phase::Snapshot, started) };
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
