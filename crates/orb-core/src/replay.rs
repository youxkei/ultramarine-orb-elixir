//! **The replay of a pointdevice run: every stretch of it that was played, the attempts that were
//! rewound away included.**
//!
//! The game's own replay cannot hold one of these, and *Pointdevice and normal* says why: the record of
//! inputs lives in the heap a snapshot covers — `ReplayManager` is `new`ed at 0x42a27f and each stage's
//! record allocated per stage — so a retry rewinds the record along with the run and each attempt writes
//! over the last from the chapter's own frame. What the game would save is the path that survived, with
//! nothing in it of the attempts that did not, and a run nobody could tell from a flawless one is the one
//! thing a replay of *this* mode must not be.
//!
//! So orb keeps its own, and what it keeps is the thing the game's own cannot: the attempts. **An attempt
//! is a stretch the run played from one rewind point to the next**, filed under the chapter it started at.
//! Every chapter therefore has the attempts that were rewound away and, last, the one that survived — which
//! is not held anywhere of its own: it is the stage's own path from that chapter's frame onwards, and the
//! path a stage was left with *is* the attempts that survived, strung together. That is the same property
//! [`crate::resume`] leans on, and it is why a chapter retried costs this nothing either.
//!
//! **One file per run there can be one of**, named for the same slot the chapter that run was left in is
//! named for — see `Game::run_slot`. The two correspond by that name and by nothing else: a run picked up
//! in a later launch reads the chapter out of one and goes on writing the other, and neither had to be told
//! about the other to do it.
//!
//! **A run still being played is not a saved replay, and the extension says which is which.** `.pending` is
//! the run somebody is in the middle of, or one they walked away from; [`EXTENSION`] is one that has been
//! kept. So the list orb offers replays from holds what was finished with and not the run in progress, and
//! the two live in one directory because they are the same file at two moments of its life.
//!
//! **It grows as the run does**, for the reason the chapter file does: there is no moment a session ends at
//! — the window closed, the game killed and a crash all leave nothing to write from — so a chapter
//! beginning and an attempt being rewound each hand the whole of it to [`crate::writer`]. Whole rather than
//! appended to, because what an attempt does to it is not always at the end: a chapter played again adds an
//! attempt in the middle of a run's own history.
//!
//! Why this file exists at all rather than the game's own format or its own replay manager is
//! [docs/adr/0024](../../../docs/adr/0024-a-pointdevice-runs-replay-is-orbs-own-and-is-watched-through-the-resumes-machinery.md).
//!
//! **One shape for both games, unlike the chapter file.** That one is per-game because it shipped that way
//! — a field th07 needed could not be added without refusing every th06 file already on a disk, which is
//! written where that version number is. This file is new and has no installed base, so what each game
//! contributes to it travels as what it already is: the numbers a stage starts with are the game's own
//! bytes, opaque here, exactly as [`crate::resume::Kept::state`] carries them. What is checked instead is
//! [`Game::named`], so a file carried between two installations is refused rather than read as this game's.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use orb_api::clock::LocalTime;

use crate::game::{Game, RunStart};
use crate::resume::{Kept, MAX_STAGE_FRAMES};
use crate::sync::MainThread;
use crate::{detail, log, writer};

/// What every line about a replay starts with, the thread that writes one included — see
/// [`writer::Handed::about`].
const ABOUT: &str = "replay";

/// The directory beside the game, which holds both the run being played and the ones that were kept: they
/// are the same file at two moments, and a directory apiece would be two places to look for one run's.
const DIRECTORY: &str = "pointdevice_replay";
/// What a replay somebody kept is called. MessagePack with the field names in it, for the reasons the
/// chapter file's own extension gives: the file is the machine's own, nobody reads a button mask by eye, and
/// a file carrying its own field names needs nothing but itself and a msgpack-to-YAML converter to read.
const EXTENSION: &str = "msgpack";
/// And what the run somebody is in the middle of is called, which is what keeps it out of the list.
const WHILE_PLAYING: &str = "pending";

/// What this file's fields mean, and what a file carrying another number is refused for.
const VERSION: u32 = 1;

/// How long a name a replay carries, in characters.
///
/// The row the launcher's settings dialog types one into holds this many, and this is where a name from an
/// `orb.yaml` somebody wrote by hand is held to the same: what the list draws it on is one line that also
/// carries which run it was and when, and a name past the width is drawn off the edge of a 640-wide output
/// where nothing clips it.
const NAME_LIMIT: usize = 12;

/// Whether the run being played is one whose replay is being kept.
///
/// An atomic for the reason [`crate::resume`]'s is: the two hooks that ask — the game's input read, and the
/// moment a stage's numbers are put in place — run inside the game's own update, where the `Runtime` that
/// knows which mode orb is in is already borrowed.
static KEEPING: AtomicBool = AtomicBool::new(false);

static RECORD: MainThread<Record> = MainThread::new(Record::new());

/// One attempt as somebody watches it: which frame of the stage it starts at, and what was pressed from
/// there.
///
/// **Its own frame and not the chapter's**, although the two are usually the same: what a rewind sends the
/// run back to is a snapshot's frame, and a stage played from further back arrives at its chapters a frame
/// or two off where the attempt before it did. So an attempt says where it starts and a playback feeds it
/// from there, and neither has to agree with the chapter it is filed under about anything but which stretch
/// of the stage it is.
///
/// **Not the generators**, which the frames *before* it carry. Reaching the chapter an attempt starts at is
/// a playback held to the run frame by frame — see [`crate::resume`] — and the attempt itself is watched
/// rather than checked, so there is nothing to hold it against.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Attempt {
    pub at: u32,
    pub buttons: Vec<u16>,
}

/// A chapter of a stage, as the replay of it holds one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Chapter {
    pub number: u32,
    pub name: String,
    /// The stage frame it began on, which is what identifies it: a chapter is where the run can be sent
    /// back to, and that is a frame.
    pub at: u32,
    /// Where the song was when it began, for a playback to put the music back the way a retry of it does.
    pub song: Option<i32>,
    /// And which track that position is in — see [`crate::resume::Kept::song_identity`], which is the same
    /// reading kept for the same reason: the file the game streams holds every track, so a position put into a
    /// stream playing another one is bytes of whatever lies there.
    pub song_identity: Option<u32>,
    /// And whether it was coming out, `None` for a file that does not say — see
    /// [`crate::resume::Kept::song_sounding`].
    pub song_sounding: Option<bool>,
    /// What [`crate::game::Reproduction`] said on that frame, so that a playback which reached it can be
    /// held against the run that recorded it.
    pub landing: String,
    /// The attempts that were rewound away, in the order they were made. The one that survived is not here
    /// — see the module's own note.
    pub lost: Vec<Attempt>,
}

/// One stage of the run: the numbers it began with, the path it was left with, and its chapters.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub stage: i32,
    /// The numbers that stage started with, **in the game's own bytes and opaque here** — the same bytes
    /// [`crate::resume::Kept::state`] carries, and the ones a playback seeds the stage from.
    pub state: Vec<u8>,
    /// One entry per stage frame: the path the stage was left with, which is the attempts that survived
    /// strung together.
    pub played: Vec<u16>,
    /// And the generator each of those frames was played with, which is what says whether a playback that
    /// is reaching a chapter is still reproducing the run.
    pub generators: Vec<Option<u16>>,
    /// In the order the stage plays them, which is by the frame each began on.
    pub chapters: Vec<Chapter>,
}

impl Stage {
    /// How many attempts a chapter has: the ones that were rewound away, and the one that survived.
    ///
    /// The survivor is counted even where the run was left in the middle of it — a stretch somebody
    /// walked away from is a stretch that was played — and it is not counted where the chapter is one the
    /// path no longer reaches at all, which is a chapter of an earlier attempt that a rewind wrote past.
    pub fn tries(&self, chapter: &Chapter) -> usize {
        chapter.lost.len() + usize::from(self.survived(chapter).is_some())
    }

    /// One of them, counted the way [`Stage::tries`] counts: the rewound ones first, in the order they were
    /// made, and the one that survived last.
    pub fn attempt(&self, chapter: &Chapter, which: usize) -> Option<Attempt> {
        match chapter.lost.get(which) {
            Some(lost) => Some(lost.clone()),
            None if which == chapter.lost.len() => Some(Attempt {
                at: chapter.at,
                buttons: self.survived(chapter)?,
            }),
            None => None,
        }
    }

    /// The stretch of the path from a chapter's own frame to the frame the chapter after it began on, and
    /// `None` where the path does not reach that frame.
    ///
    /// **The chapter and not the rest of what was played**, because an attempt at a chapter is that chapter:
    /// the frames past it are the next chapter's, and they were chosen out of no list. To the end of what was
    /// played instead, watching the attempt that survived the third chapter of a stage played the twelve
    /// after it as well.
    ///
    /// Where no chapter follows, the end of what was played is where this one ended: the run was left in it,
    /// or the boss died in it.
    fn survived(&self, chapter: &Chapter) -> Option<Vec<u16>> {
        let ended = self
            .chapters
            .iter()
            .find(|next| next.at > chapter.at)
            .map_or(self.played.len(), |next| next.at as usize);
        self.played
            .get(chapter.at as usize..ended.min(self.played.len()))
            .filter(|played| !played.is_empty())
            .map(<[u16]>::to_vec)
    }
}

/// A whole run, as a file of one holds it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Replay {
    /// Which run it was: the difficulty, the character and the shot, which are what a slot is named from.
    /// The stage in it is the one the run *began* at, the stages it played being below.
    pub run: RunStart,
    /// Whose it is, as `orb.yaml` says — empty where nobody has said, and then the list names it by which
    /// run it was and when.
    pub name: String,
    /// When the run began, as the host's calendar said: what a kept replay's own file name carries, so the
    /// name and the file agree about which evening this was.
    pub started: String,
    /// How the run ended, and `None` in a file written before this was recorded — see [`Ended`].
    pub ended: Option<Ended>,
    pub stages: Vec<Stage>,
}

/// How the run a replay is of came to an end.
///
/// **What it is for is the line the list shows.** A name is what a run ends *with* in this game — the
/// high-score entry a finished run reaches — so a name against a run somebody walked away from is a name
/// from nowhere: what orb has is the setting saying whose orb this is, which says nothing about which of
/// its runs were played out. So the list says whose run a finished one was, and *that it was interrupted*
/// about one that was not.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Ended {
    /// Played to its end: the boss died and the screens a run ends at came up.
    Finished,
    /// And left where it stood — given up at the retry menu, or written over by はじめから.
    Interrupted,
}

impl Replay {
    /// The line the list of replays shows for it: whose run it was, which run, and when.
    ///
    /// Nothing about how far it got, which is the next screen's: choosing a replay is followed by choosing a
    /// stage out of it, and the stages it holds are that list.
    ///
    /// `whose` is the first column, which the caller chooses because it is the caller that has the words: the
    /// name in `orb.yaml` for a run that finished, and for one that did not, what the list says instead of a
    /// name — see [`Ended`]. Empty leaves the column out altogether, which is a file whose launch had no name
    /// set.
    pub fn describe(&self, slot: &str, whose: &str) -> String {
        match whose.is_empty() {
            true => format!("{slot}  {}", self.started),
            false => format!("{whose}  {slot}  {}", self.started),
        }
    }
}

/// What is being kept of the run being played.
struct Record {
    /// Which run it is, and `None` while nothing is being kept.
    run: Option<RunStart>,
    slot: String,
    name: String,
    started: String,
    stages: Vec<Stage>,
}

impl Record {
    /// **`const fn` because [`RECORD`] is a static**, which is also why no e2e test can enter it: what
    /// happens here is const evaluation and not execution, so a coverage run reports it never run however
    /// many replays were written down. See `resume::Record::new`, which is the same zero.
    const fn new() -> Self {
        Self {
            run: None,
            slot: String::new(),
            name: String::new(),
            started: String::new(),
            stages: Vec::new(),
        }
    }

    fn stop(&mut self) {
        self.run = None;
        self.slot.clear();
        self.name.clear();
        self.started.clear();
        self.stages.clear();
    }

    /// The stage being played, which is the last one: a run reaches its stages in order, and a stage
    /// re-entered by a resume is the one it was already writing.
    fn stage(&mut self) -> Option<&mut Stage> {
        self.stages.last_mut()
    }

    /// Writes down what the update of `frame` is about to act on, the way the chapter file's own record
    /// does: the same frame reached twice — a chapter retried, or one the game spent with a menu up — is one
    /// entry, and the last write wins because that is the one the update sees.
    fn note(&mut self, frame: u32, buttons: u16, generator: u16) {
        if frame >= MAX_STAGE_FRAMES {
            return;
        }
        let Some(stage) = self.stage() else {
            return;
        };
        let at = frame as usize;
        if stage.played.len() <= at {
            stage.played.resize(at + 1, 0);
            stage.generators.resize(at + 1, None);
        }
        stage.played[at] = buttons;
        stage.generators[at] = Some(generator);
    }

    /// The replay as a file of it, which is what the thread that writes them is handed.
    fn written(&self, game: &dyn Game) -> Option<Vec<u8>> {
        let run = self.run?;
        Some(encode(
            game,
            &Replay {
                run,
                name: self.name.clone(),
                started: self.started.clone(),
                // Nothing yet: this is the run being played, and how it ends is settled where it is kept.
                ended: None,
                stages: self.stages.clone(),
            },
        ))
    }
}

/// Whether the run being played is one whose replay is worth keeping: pointdevice, orb keeping what it
/// pressed at all, and a launch that does not refuse replays outright.
///
/// **Every entry point here reads it, [`begun`] and [`picked_up`] included.** Those two are called where a
/// run is registered, which is a frame that says nothing about whether this launch keeps replays — so a
/// record started there and never written into is a file with a run's name and nothing in it, and the offer
/// to keep one would go up over the screen a run somebody could not have recorded ended at.
pub fn keep(on: bool) {
    KEEPING.store(on, Ordering::Relaxed);
}

/// Writes down what the game saw on the frame it saw it, beside the chapter file's own record of the same
/// read.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn noted(frame: u32, buttons: u16, generator: u16) {
    if !KEEPING.load(Ordering::Relaxed) {
        return;
    }
    unsafe { RECORD.get() }.note(frame, buttons, generator);
}

/// Whether a run's replay is being written down now, which is what says there is one to keep.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn keeping() -> bool {
    unsafe { RECORD.get() }.run.is_some()
}

/// What becomes of the replay that was being written when a run starts over it.
///
/// Two, because the same run starting from the beginning is two different things depending on whether
/// anybody is being asked: see [`begun`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TheOneBefore {
    /// Kept, and put where the list offers it.
    Kept,
    /// Somebody is being asked about it, and their answer is what keeps it or takes it away — so nothing
    /// here touches the file, and this run's own record starts beside it. Which costs nothing: the first
    /// thing that writes is this run's first chapter, and the question is answered before a stage exists.
    Asked,
}

/// Starts writing a run down from its beginning, whatever was there before.
///
/// **Whatever was there before is finished with**, which is the whole of what はじめから does to a replay:
/// the chapter file this run is about to write over describes a run somebody played, and the replay of that
/// run is the replay of what they played up to there — nothing after it ever will be. So it is given the
/// day and the time and put where the list offers it, and this run starts a file of its own under the same
/// name the one it replaced had. Where somebody is being asked about it instead, `before` says so and their
/// answer is what does it.
///
/// # Safety
/// Must run on the game's main thread, between frames.
pub unsafe fn begun(
    dir: &Path,
    game: &dyn Game,
    run: &RunStart,
    slot: &str,
    name: &str,
    now: LocalTime,
    before: TheOneBefore,
) {
    if !KEEPING.load(Ordering::Relaxed) {
        return;
    }
    // Interrupted, whatever it was: this is a run being written over by the one starting here, so what the
    // file holds is a run somebody walked away from and not one played out.
    if before == TheOneBefore::Kept {
        keep_what_was_being_played(dir, game, slot, name, now, Ended::Interrupted);
    }
    let record = unsafe { RECORD.get() };
    record.stop();
    record.run = Some(*run);
    record.slot = slot.to_owned();
    record.name = shortened(name);
    record.started = now.to_string();
    log!("{ABOUT}: {slot} begun at {now}; what it plays is being written down");
}

/// And goes on writing one that was picked up where an earlier launch left it, which is the other half of
/// the chapter file and the replay corresponding: the run carries on, so the replay it was being written
/// into carries on too.
///
/// `saved` is the chapter that run was picked up at, and it is what a replay is *built* from where there is
/// none to carry on: it holds the stage's own numbers, the buttons of every frame up to that chapter and the
/// generator each was played with, which is one stage with one chapter in it. So a launch that has the
/// chapter and not the replay — the file deleted, or a run left before orb kept replays at all — is offered
/// the run from where it was picked up rather than nothing at all.
///
/// # Safety
/// Must run on the game's main thread, between frames.
pub unsafe fn picked_up(
    dir: &Path,
    game: &dyn Game,
    slot: &str,
    name: &str,
    saved: &Kept,
    now: LocalTime,
) {
    if !KEEPING.load(Ordering::Relaxed) {
        return;
    }
    let record = unsafe { RECORD.get() };
    record.stop();
    let carried = read(&being_played_at(dir, slot), game)
        .filter(|replay| game.run_slot(&replay.run).as_deref() == Some(slot));
    let replay = match carried {
        Some(replay) => {
            log!(
                "{ABOUT}: {slot} was begun at {}; the run picked up goes on being written into it",
                replay.started,
            );
            replay
        }
        None => {
            log!(
                "{ABOUT}: nothing of {slot} was being written down; what the chapter holds is where \
                 its replay starts"
            );
            Replay {
                run: saved.run,
                name: shortened(name),
                started: now.to_string(),
                ended: None,
                stages: Vec::new(),
            }
        }
    };
    record.run = Some(replay.run);
    record.slot = slot.to_owned();
    // The name as `orb.yaml` says it now, and not the one the file was begun with: whose run it is is a
    // setting, so a name changed between two sessions of one run is a name changed for that run.
    record.name = shortened(name);
    record.started = replay.started;
    record.stages = replay.stages;
    // The stage the chapter is in, where the file being carried on has nothing of it: the frames below that
    // chapter are what the playback is about to run, and nothing else has them.
    if record
        .stages
        .iter()
        .all(|stage| stage.stage != saved.run.stage)
    {
        record.stages.push(the_stage_a_chapter_is_in(saved));
    }
}

/// One stage as a chapter of it describes one: the numbers, the path up to that chapter, and the chapter
/// itself.
fn the_stage_a_chapter_is_in(saved: &Kept) -> Stage {
    Stage {
        stage: saved.run.stage,
        state: saved.state.clone(),
        played: saved.buttons.clone(),
        generators: saved.generators.clone(),
        chapters: vec![Chapter {
            number: saved.chapter,
            name: saved.name.clone(),
            at: saved.at,
            song: saved.song,
            song_identity: saved.song_identity,
            song_sounding: saved.song_sounding,
            landing: saved.landing.clone(),
            lost: Vec::new(),
        }],
    }
}

/// Drops what is being kept without writing it, for a run that is no longer one whose replay is kept at
/// all — the mode answered the other way, which is a run with nothing of orb's happening to it.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn forget() {
    unsafe { RECORD.get() }.stop();
}

/// Writes what is being kept and then drops it, for a run that has ended however it ended.
///
/// **The last stretch is in the file because of this and nothing else.** What hands the file over otherwise
/// is a chapter beginning and an attempt being rewound away, and on the frame a chapter begins the run has
/// played none of it yet: the read for that frame has not happened, so the newest chapter's own attempt is
/// nought frames long until something writes again. A run given up at the retry menu, one whose last life
/// went, one that reached its ending — each of them ends without another chapter, and this is what leaves
/// the stretch it ended in watchable.
///
/// The file stays where it is: a run given up is one somebody can pick up again, and its replay is what the
/// next session goes on writing.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn left(dir: &Path, game: &dyn Game) {
    let record = unsafe { RECORD.get() };
    if record.run.is_some() {
        unsafe { written_down(dir, game) };
    }
    record.stop();
}

/// A stage of the run has just had its numbers put in place and has not been updated yet, which is where
/// the numbers a playback would seed it from are readable.
///
/// A stage already in the record is the one being re-entered rather than a second of it — which is what a
/// run picked up does, the resume playing the same stage in again — so its chapters and the attempts under
/// them stay where they are.
///
/// # Safety
/// Must run on the game's main thread, after [`crate::resume::stage_begun`] has put a resumed run's own
/// numbers back: what goes in here is the numbers the stage is going to be played with.
pub unsafe fn stage_begun(game: &dyn Game) {
    if !KEEPING.load(Ordering::Relaxed) {
        return;
    }
    // A run of somebody's own and not the attract demo or a replay being watched, which reach this the same
    // way — see `resume::stage_begun`, where the same three say whether a stage is one to write down.
    let state = unsafe { game.read_state() };
    if !state.in_run || state.demo || state.replay {
        return;
    }
    let stage = state.stage;
    let started = unsafe { game.run_start() };
    let state = unsafe { game.run_state() };
    let record = unsafe { RECORD.get() };
    if record.run.is_none() {
        return;
    }
    // **Which run this is, as the game says it now.** A stage with its numbers in place is the first moment
    // *both* games say: th07 keeps the difficulty a build writes where a run in progress keeps it, so a
    // record begun on the frame the run was registered was begun under a slot a build away from this run's —
    // `easy-reimu-a` for a Normal run, and a file of that name. Said when it moves, because the name it moves
    // to is the file every line after this one is about.
    if let Some(slot) = game.run_slot(&started).filter(|slot| *slot != record.slot) {
        log!(
            "{ABOUT}: {} is {slot} now that its stage has its numbers",
            record.slot,
        );
        record.slot = slot;
        record.run = Some(started);
    }
    if let Some(already) = record.stages.iter_mut().find(|held| held.stage == stage) {
        // The numbers all the same, since this entry into the stage is the one a playback would reproduce.
        already.state = state;
        return detail!("{ABOUT}: stage {} again; its chapters are kept", stage + 1);
    }
    record.stages.push(Stage {
        stage,
        state,
        played: Vec::new(),
        generators: Vec::new(),
        chapters: Vec::new(),
    });
    detail!("{ABOUT}: stage {} begins", stage + 1);
}

/// A chapter has begun, on the frame it began on: the one frame the song's position and the reproduction
/// line that go in with it are that chapter's.
///
/// **A chapter is its number, and the frame is wherever it was last announced at.** A stage announces one
/// twice on the way in — the stage's own start, written before the first chapter has settled, and then that
/// chapter itself — and a stage played again from further back arrives at its boundaries a frame or two off
/// where the attempt before it did. Two entries for one chapter would be a list with the same chapter on it
/// twice, so the later announcement moves the one that is there: what the path now reaches that chapter at
/// is where somebody watching it starts. The attempts under it keep their own frames — see [`Attempt`].
///
/// # Safety
/// Must run on the game's main thread, on the frame the chapter began on.
pub unsafe fn chapter(dir: &Path, game: &dyn Game, saved: &crate::resume::Kept) {
    if !KEEPING.load(Ordering::Relaxed) {
        return;
    }
    let record = unsafe { RECORD.get() };
    let Some(stage) = record.stage() else {
        return;
    };
    if let Some(already) = stage
        .chapters
        .iter_mut()
        .find(|chapter| chapter.number == saved.chapter)
    {
        if already.at == saved.at {
            return;
        }
        already.at = saved.at;
        already.name = saved.name.clone();
        already.song = saved.song;
        already.song_identity = saved.song_identity;
        already.song_sounding = saved.song_sounding;
        already.landing = saved.landing.clone();
        stage.chapters.sort_by_key(|chapter| chapter.at);
        return unsafe { written_down(dir, game) };
    }
    stage.chapters.push(Chapter {
        number: saved.chapter,
        name: saved.name.clone(),
        at: saved.at,
        song: saved.song,
        song_identity: saved.song_identity,
        song_sounding: saved.song_sounding,
        landing: saved.landing.clone(),
        lost: Vec::new(),
    });
    // By the frame each began on, which is the order the stage plays them: a chapter whose boundary landed
    // somewhere else on a later attempt goes where its own frame puts it rather than where it was found.
    stage.chapters.sort_by_key(|chapter| chapter.at);
    unsafe { written_down(dir, game) };
}

/// An attempt has been rewound away: the run was at `ended` and has been sent back to the chapter that
/// begins at `to`.
///
/// What is kept is the path from `to` to `ended` as it stands now, because that is what the attempt was —
/// and it has to be kept *here*, before the frames from `to` onwards are played again and written over.
///
/// **Filed under the chapter that frame falls in**, which is the last one to begin at or before it rather
/// than one that begins on it exactly: a rewind goes back to a snapshot, and a stage's own start is a
/// snapshot at a frame no chapter's boundary is on. The attempt keeps the frame it really started at, so
/// what is watched is the stretch that was played and not the chapter's own opening.
///
/// # Safety
/// Must run on the game's main thread, on the frame the restore put the run on.
pub unsafe fn rewound(dir: &Path, game: &dyn Game, to: u32, ended: u32) {
    if !KEEPING.load(Ordering::Relaxed) {
        return;
    }
    let record = unsafe { RECORD.get() };
    let Some(stage) = record.stage() else {
        return;
    };
    let Some(played) = stage.played.get(to as usize..ended as usize) else {
        return detail!(
            "{ABOUT}: an attempt from frame {to} to frame {ended} is not in a path of {} frame(s)",
            stage.played.len(),
        );
    };
    let attempt = Attempt {
        at: to,
        buttons: played.to_vec(),
    };
    let Some(chapter) = stage.chapters.iter_mut().rfind(|chapter| chapter.at <= to) else {
        return detail!("{ABOUT}: no chapter of this stage begins at or before frame {to}");
    };
    chapter.lost.push(attempt);
    detail!(
        "{ABOUT}: attempt {} at chapter {} was rewound away after {} frame(s)",
        chapter.lost.len(),
        chapter.number,
        ended - to,
    );
    unsafe { written_down(dir, game) };
}

/// Hands the run being played to the thread that writes orb's own files.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn written_down(dir: &Path, game: &dyn Game) {
    let record = unsafe { RECORD.get() };
    let Some(bytes) = record.written(game) else {
        return;
    };
    let stages: usize = record.stages.len();
    let chapters: usize = record.stages.iter().map(|stage| stage.chapters.len()).sum();
    let attempts: usize = record
        .stages
        .iter()
        .flat_map(|stage| stage.chapters.iter().map(|chapter| stage.tries(chapter)))
        .sum();
    writer::hand_over(writer::Handed {
        about: ABOUT,
        path: being_played_at(dir, &record.slot),
        bytes,
        said: format!(
            "{}, {stages} stage(s), {chapters} chapter(s), {attempts} attempt(s)",
            record.slot,
        ),
        overtaken: format!("{}, {chapters} chapter(s)", record.slot),
    });
}

/// Keeps the run being written down as a replay somebody can watch, and says what it is called.
///
/// Called where a run is finished with: the screen it ended at, or the answer that writes over the chapter
/// it was left in. Nothing is kept where nothing was being written — a run in normal mode, a launch with no
/// replays being kept — and nothing is kept twice, the pending file going as this one appears.
/// `ended` is how the run came to be finished with, which is the caller's to say: the two moments a replay
/// is kept at are a run that reached the screens it ends at and a run being written over — see [`Ended`].
pub fn keep_what_was_being_played(
    dir: &Path,
    game: &dyn Game,
    slot: &str,
    name: &str,
    now: LocalTime,
    ended: Ended,
) -> Option<PathBuf> {
    let from = being_played_at(dir, slot);
    let mut replay = read(&from, game)?;
    replay.ended = Some(ended);
    // **And whose it is, settled here rather than left as the writes left it.** The name comes from the game's
    // own name entry, which for a run that reached the end is a screen *after* the last chapter was written —
    // so a file sealed with the name its last write carried is a file named by the run before this one.
    replay.name = shortened(name);
    // The day it was kept on and not the day it began, where the file cannot say: what somebody looks for
    // in a list is the evening they played, and a run played across two sessions is the later one.
    let stamp = match replay.started.is_empty() {
        true => now.to_string(),
        false => replay.started.clone(),
    };
    let into = dir
        .join(DIRECTORY)
        .join(format!("{slot}-{stamp}"))
        .with_extension(EXTENSION);
    // Written where it stands rather than handed to the writer's thread: this is the one moment there is to
    // keep it, and a run given up here is a replay lost. The bytes are the ones already on the disk.
    if let Err(error) = orb_api::fs::write(&into, &encode(game, &replay)) {
        log!(
            "{ABOUT}: cannot keep {} as {}: {error}",
            from.display(),
            into.display()
        );
        return None;
    }
    log!(
        "{ABOUT}: kept as {}",
        // Beside the game rather than as a whole path, which is what somebody reading the log wants: the
        // directory is the one every other line about this file names.
        display_beside(dir, &into),
    );
    match orb_api::fs::remove_file(&from) {
        Ok(()) => detail!("{ABOUT}: {} taken away, having been kept", from.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => log!("{ABOUT}: cannot take {} away: {error}", from.display()),
    }
    Some(into)
}

/// Takes the run being written down away, for a replay somebody was offered and declined.
pub fn declined(dir: &Path, slot: &str) -> Option<PathBuf> {
    let path = being_played_at(dir, slot);
    match orb_api::fs::remove_file(&path) {
        Ok(()) => {
            log!("{ABOUT}: not kept; {} removed", path.display());
            Some(path)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            log!("{ABOUT}: cannot remove {}: {error}", path.display());
            None
        }
    }
}

/// A path under the directory these live in, as the log says it: the directory and the file, and not the
/// whole of wherever the game is installed.
fn display_beside(dir: &Path, path: &Path) -> String {
    path.strip_prefix(dir)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

/// Where the run of a slot that is being played is written.
fn being_played_at(dir: &Path, slot: &str) -> PathBuf {
    dir.join(DIRECTORY).join(slot).with_extension(WHILE_PLAYING)
}

/// The runs there is a replay of in progress, as the slots naming them.
pub fn being_played(dir: &Path) -> Vec<String> {
    stems(dir, WHILE_PLAYING)
}

/// Whether there is one for a named run, which is what says there is something to ask about keeping.
///
/// By name rather than by what is being kept in memory, because the two moments this is asked at are
/// different: the screen a run ended at has that run still in memory, and a run about to be started over
/// has only the file the last session left.
pub fn is_being_played(dir: &Path, slot: &str) -> bool {
    orb_api::fs::read(&being_played_at(dir, slot)).is_ok()
}

/// And the replays somebody kept, as the names of their files — which is a slot and the day it was kept on.
pub fn saved(dir: &Path) -> Vec<String> {
    stems(dir, EXTENSION)
}

fn stems(dir: &Path, extension: &str) -> Vec<String> {
    let mut names: Vec<String> = orb_api::fs::files_in(&dir.join(DIRECTORY))
        .into_iter()
        .flatten()
        .filter(|path| path.extension().is_some_and(|it| it == extension))
        .filter_map(|path| Some(path.file_stem()?.to_string_lossy().into_owned()))
        .collect();
    // In order, so that a listing of the directory and a list on the screen read the same way.
    names.sort();
    names
}

/// One of the replays somebody kept, whole.
///
/// The game is handed over rather than read out of the static the hooks read, the way `resume::peek` takes
/// one: what a replay holds per stage is that game's own bytes, so reading one is asking a game about a
/// file rather than asking orb about the launch it is in.
pub fn read_saved(dir: &Path, name: &str, game: &dyn Game) -> Option<Replay> {
    read(
        &dir.join(DIRECTORY).join(name).with_extension(EXTENSION),
        game,
    )
}

/// And the run of a slot that is being played, for whoever wants to see how far it has got.
pub fn read_being_played(dir: &Path, slot: &str, game: &dyn Game) -> Option<Replay> {
    read(&being_played_at(dir, slot), game)
}

/// Reads one, and `None` for a file that is not there or is not a replay of this game's runs.
fn read(path: &Path, game: &dyn Game) -> Option<Replay> {
    let bytes = match orb_api::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            log!("{ABOUT}: cannot read {}: {error}", path.display());
            return None;
        }
    };
    match decode(game, &bytes) {
        Some(replay) => Some(replay),
        // Left where it is rather than deleted, for the reason a chapter file that cannot be read is: a
        // file nobody can read is worth looking at, and orb deleting it is what would stop anybody doing
        // so.
        None => {
            log!(
                "{ABOUT}: cannot make sense of {}; it is left alone",
                path.display()
            );
            None
        }
    }
}

/// A name held to what the list has room to draw, by characters and not by bytes: every name here is
/// somebody's own text, and half of a Japanese one is not a name.
fn shortened(name: &str) -> String {
    name.chars().take(NAME_LIMIT).collect()
}

/// The file, whole.
fn encode(game: &dyn Game, replay: &Replay) -> Vec<u8> {
    let written = Written {
        version: VERSION,
        game: game.named().to_owned(),
        name: replay.name.clone(),
        started: replay.started.clone(),
        ended: replay.ended,
        run: WrittenRun {
            difficulty: replay.run.difficulty,
            character: replay.run.character,
            shot: replay.run.shot_type,
            // **The first stage this replay holds, and not the stage the run was registered with.** That
            // one is what the character select left — the stage before the one meant, which for a run
            // starting at the first is *minus one* — and a launch on the real game wrote `stage: -1` into
            // every file. Nothing reads the field; what it can honestly say is where the replay starts.
            stage: replay
                .stages
                .first()
                .map_or(replay.run.stage, |stage| stage.stage),
        },
        stages: replay
            .stages
            .iter()
            .map(|stage| WrittenStage {
                stage: stage.stage,
                state: stage.state.clone(),
                frames: stage.played.len() as u32,
                buttons: changes(&stage.played, 0),
                generators: stage.generators.clone(),
                chapters: stage
                    .chapters
                    .iter()
                    .map(|chapter| WrittenChapter {
                        number: chapter.number,
                        frame: chapter.at,
                        name: chapter.name.clone(),
                        song: chapter.song,
                        song_identity: chapter.song_identity,
                        song_sounding: chapter.song_sounding,
                        landing: chapter.landing.clone(),
                        lost: chapter
                            .lost
                            .iter()
                            .map(|attempt| WrittenAttempt {
                                from: attempt.at,
                                frames: attempt.buttons.len() as u32,
                                buttons: changes(&attempt.buttons, attempt.at),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    // With the names in it, so that the file says what its own fields are — see [`EXTENSION`]. Nothing in
    // it can fail to serialize, so a failure would be a file that says nothing about the run, which is
    // what no bytes read as.
    rmp_serde::to_vec_named(&written).unwrap_or_default()
}

/// And back. `None` for anything that is not a replay this version of orb wrote for this game.
fn decode(game: &dyn Game, bytes: &[u8]) -> Option<Replay> {
    let written: Written = rmp_serde::from_slice(bytes).ok()?;
    if written.version != VERSION || written.game != game.named() {
        return None;
    }
    let mut stages = Vec::new();
    for stage in written.stages {
        // A path no stage reaches is a file this cannot allocate for, and one whose changes fall outside it
        // is a file that does not describe its own path.
        if stage.frames > MAX_STAGE_FRAMES
            || stage
                .buttons
                .keys()
                .last()
                .is_some_and(|frame| *frame >= stage.frames)
        {
            return None;
        }
        // An entry for every frame, or none at all: the two are held together by frame while a playback
        // reaches a chapter, so a shorter list is one whose frames do not line up.
        if !stage.generators.is_empty() && stage.generators.len() != stage.frames as usize {
            return None;
        }
        let mut chapters = Vec::new();
        for chapter in stage.chapters {
            if chapter.frame > stage.frames {
                return None;
            }
            chapters.push(Chapter {
                number: chapter.number,
                name: chapter.name,
                at: chapter.frame,
                song: chapter.song,
                song_identity: chapter.song_identity,
                song_sounding: chapter.song_sounding,
                landing: chapter.landing,
                lost: chapter
                    .lost
                    .iter()
                    .map(|attempt| Attempt {
                        at: attempt.from,
                        buttons: dense(
                            &attempt.buttons,
                            attempt.from,
                            attempt.frames.min(MAX_STAGE_FRAMES),
                        ),
                    })
                    .collect(),
            });
        }
        chapters.sort_by_key(|chapter| chapter.at);
        stages.push(Stage {
            stage: stage.stage,
            state: stage.state,
            played: dense(&stage.buttons, 0, stage.frames),
            generators: stage.generators,
            chapters,
        });
    }
    Some(Replay {
        run: RunStart {
            difficulty: written.run.difficulty,
            character: written.run.character,
            shot_type: written.run.shot,
            // Not in the file, there being nothing to say: a practice run has no slot to be kept in — see
            // `Game::run_slot` — so every replay here is a full run's.
            practice: false,
            stage: written.run.stage,
        },
        name: written.name,
        started: written.started,
        ended: written.ended,
        stages,
    })
}

/// The file's own shape, which is not the structs above: the names in it are the file's business, so that a
/// field renamed in [`Replay`] is not a file that reads differently afterwards.
///
/// `deny_unknown_fields` throughout, because a field nothing knows is a file from another version or another
/// program, and reading one of those halfway is worse than not reading it.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    version: u32,
    /// How the run ended, and **`default` rather than a version of its own**: a file written before this was
    /// recorded says nothing about it, and the list has a name to show for such a file the way it always
    /// did. Bumping the version instead would refuse every replay somebody had already kept.
    #[serde(default)]
    ended: Option<Ended>,
    /// Which game's runs these are, as that game calls itself — see [`Game::named`]. A replay carried into
    /// another game's installation is refused by it rather than read as that game's, the numbers a stage
    /// starts with being bytes only the game that wrote them can put back.
    game: String,
    name: String,
    started: String,
    run: WrittenRun,
    stages: Vec<WrittenStage>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenRun {
    difficulty: i32,
    character: i32,
    shot: i32,
    stage: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenStage {
    stage: i32,
    /// The numbers the stage started with, as the game's own bytes — its own file format inside this one,
    /// which is what lets one shape of replay hold either game's runs. See `Game::run_state`.
    state: Vec<u8>,
    /// How many frames the path below covers, which a map of the frames it changed on cannot say.
    frames: u32,
    /// The buttons held from each frame they changed on, keyed by that frame — the same shape the chapter
    /// file writes its own for, and for the same reasons: everything between two changes is the first one's,
    /// a map of frames is in order because it is a map, and a converter prints one change per line.
    buttons: BTreeMap<u32, u16>,
    /// The generator's own state on every one of those frames, as a list — a generator moves on nearly every
    /// frame, so a map of the changes would be every frame with a key in front of it.
    generators: Vec<Option<u16>>,
    chapters: Vec<WrittenChapter>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenChapter {
    number: u32,
    frame: u32,
    name: String,
    song: Option<i32>,
    /// Which track that position is in, `null` where the stream would not say — see
    /// [`crate::resume::Kept::song_identity`].
    song_identity: Option<u32>,
    /// And whether any of it was coming out, `null` for a file that does not say — see
    /// [`crate::resume::Kept::song_sounding`].
    #[serde(default)]
    song_sounding: Option<bool>,
    landing: String,
    /// The attempts that were rewound away, in the order they were made.
    lost: Vec<WrittenAttempt>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenAttempt {
    /// The stage frame it started at, which is where a rewind sent the run back to and need not be the
    /// chapter's own — see [`Attempt`].
    from: u32,
    /// How many frames it lasted.
    frames: u32,
    /// Keyed by the stage frame, like the path's own: what somebody reading the file by eye holds an
    /// attempt against is the path, and two frame numberings would make that arithmetic.
    buttons: BTreeMap<u32, u16>,
}

/// The frames the buttons changed on, out of one entry per frame: everything between two of them is the
/// first one's, which is what a run with a key held down is.
///
/// **Keyed by the stage frame**, `from` being the frame the stretch starts on: a stage's path starts at
/// nought and an attempt's starts at its chapter's own frame, and what somebody reading the file by eye
/// holds an attempt against is the path. Two numberings would make that arithmetic.
fn changes(buttons: &[u16], from: u32) -> BTreeMap<u32, u16> {
    let mut changes = BTreeMap::new();
    let mut held = 0;
    for (frame, pressed) in buttons.iter().enumerate() {
        if *pressed != held {
            changes.insert(from + frame as u32, *pressed);
            held = *pressed;
        }
    }
    changes
}

/// One entry per frame again, out of the frames the buttons changed on.
fn dense(changes: &BTreeMap<u32, u16>, from: u32, frames: u32) -> Vec<u16> {
    let mut buttons = vec![0; frames as usize];
    let mut held = 0;
    for frame in 0..frames {
        if let Some(pressed) = changes.get(&(from + frame)) {
            held = *pressed;
        }
        buttons[frame as usize] = held;
    }
    buttons
}
