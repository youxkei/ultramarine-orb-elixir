//! Picking a run up again at the chapter it was left in, in a later launch of the game.
//!
//! A snapshot is only good inside the process that took it — it holds pointers to Direct3D and
//! DirectSound objects, and those addresses differ per launch — so closing the game loses where a
//! run was. What does survive a launch is what the run *pressed*: the buttons of the stage being
//! played, from its first frame to the frame the chapter began on, beside the run's own numbers as
//! that stage started. Playing those back into a stage the game has just built arrives at the same
//! frame, which is what the game's own replay does with the same numbers.
//!
//! Written down every time a chapter begins, so that whatever ends the session — the retry menu's
//! third item, the window closed, a crash — leaves the chapter to come back to. Never part of one:
//! the resume point is a chapter's own first frame, which is where dying sends the player anyway,
//! and the frame a run was abandoned on is as likely as not the frame they were hit on.
//!
//! **One per run there can be one of**, the way 紺珠伝 keeps a pointdevice save per difficulty and
//! character: a run of another character is another run, and its buttons would play somebody else's
//! shot with them. Which runs are the same run is the game's to say — see [`Game::run_slot`] — and
//! the answer doubles as the name of the file, so a listing of the directory reads as the runs
//! somebody has left unfinished. The question of picking one up is therefore asked after the
//! character select rather than at the mode question, which is a screen too early to know.
//!
//! **The record is one entry per stage frame, kept by frame number.** That is what makes a
//! retried chapter cost nothing here: an attempt that is rewound plays the same frames again and
//! writes over them, so the entries below any frame are always the ones that survived to reach it,
//! and nothing has to be dropped when a snapshot is restored.
//!
//! What is not written down is a chapter's *place*: a run is resumed by being played, so a chapter
//! a boss began — which is on no clock the table could name — is reached the same way as any other.
//! Writing down what a chapter *is*, the run's numbers plus the chapter's script frame with the
//! stage's script run forward to it, is the shape that suggests itself and it cannot reach half of
//! them: the boss's, which are the ones most worth grinding.
//!
//! **Not through the game's own replay**, which holds the same inputs and could reach the same
//! frame: `SaveReplay` writes the file, its playback and a jump to the stage reach the stage, and
//! the ending skip's loop would reach the frame. What that costs is the takeover afterwards — the
//! replay manager's high-priority job cut so it stops writing `g_CurFrameInput`, `isInReplay`
//! cleared, and recording started again into a buffer that already holds the path — and with it
//! two ways of allocating a record in the same heap, since a loaded `ReplayData` is freed as one
//! block where a recorded one is a block per stage. The run would also be a recording being
//! watched for as long as the playback lasted, which is a different thing to the game in every
//! place `isInReplay` is read: the score, the result screen, and what a stage's start puts back.
//! Written down here instead, the run stays an ordinary recording run throughout, and a second
//! quit is this machinery over again.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game::{Game, RunStart, RunState};
use crate::log::{detail, log};
use crate::sync::MainThread;

/// A directory beside the game, and one file in it per run there is a chapter of — named for that
/// run, so a listing of it reads as the runs somebody has left unfinished.
///
/// A directory rather than one file holding them all, because the file is written whole every time a
/// chapter begins: with every run in it, that write would be every run's buttons a few seconds
/// apart, and what it is written for is one of them.
const DIRECTORY: &str = "pointdevice_resume";
/// MessagePack, with the field names in it — `to_vec_named`, not the positional arrays rmp-serde
/// writes by default. What that buys is that the file needs nothing but itself to be read: any
/// converter prints one as YAML without being told the format, and reading one beside the log is what
/// has caught every fault in this machinery so far.
///
/// Packed rather than text because it is the machine's own: a stage's buttons written as lines came
/// to 30KB by a boss chapter deep in a stage, against 11KB here, and nobody reads a button mask by
/// eye. The two fields anybody does read — the seed and the reproduction line — are strings either
/// way.
const EXTENSION: &str = "msgpack";

/// What this file's fields mean. Written into it and refused on the way back in, so a file from
/// another version is a line in the log rather than a run started with fields read as each other.
const VERSION: u32 = 1;

/// Longer than any stage of any run: half an hour of one at 60 frames a second. A bound on what a
/// file can ask to be allocated for, and on the frame number a record will write down at all.
const MAX_STAGE_FRAMES: u32 = 60 * 60 * 30;

/// Whether a run's inputs are being kept at all.
///
/// An atomic because the two hooks that ask — the game's input read and the moment a stage's
/// numbers are put in place — run inside the game's own update, where the `Runtime` that knows
/// which mode orb is in is already borrowed.
static KEEPING: AtomicBool = AtomicBool::new(false);

static RECORD: MainThread<Record> = MainThread::new(Record::new());

/// Everything the file holds: which run it was, the numbers the stage started with, and the
/// buttons pressed since.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Saved {
    pub run: RunStart,
    pub state: RunState,
    /// The stage frame the chapter began on, which is where the playback stops. The buttons below
    /// are the frames before it, since a chapter begins with its own first frame not yet run.
    pub at: u32,
    pub chapter: u32,
    /// What that chapter is called, which nothing could work out again without playing the stage.
    pub name: String,
    pub retries: u32,
    /// Where the song was when that chapter began, as a position in the track's own file — the one
    /// thing about the sound that means anything in another process, the buffer and its object being
    /// this one's. `None` where the stream would not say, and then a resume leaves the music as the
    /// stage started it.
    pub song: Option<i32>,
    /// What [`crate::game::Reproduction`] said at that frame, as it says it. Held against the same
    /// line when the playback lands: if the numbers the game reads at a stage's start are not all
    /// written down, this is the first thing that says so, and which field it was.
    pub landing: String,
    /// One entry per stage frame, `0..at`.
    pub buttons: Vec<u16>,
}

impl Saved {
    /// Where the run stopped, for the item that offers to pick it up. Not which run it is: the
    /// question is asked where the player has just chosen that themselves.
    pub fn describe(&self) -> String {
        format!(
            "STAGE {}  {}  RETRY {}",
            self.run.stage + 1,
            self.name,
            self.retries,
        )
    }
}

impl fmt::Display for Saved {
    /// For the log, where what is wanted is the numbers rather than the names: this is what a
    /// landing is held against.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "stage {} chapter {} ({}) at frame {}, {} frame(s) of buttons, \
             diff={} char={}-{} score={} seed={:#06x} lives={} bombs={} power={} rank={}",
            self.run.stage + 1,
            self.chapter,
            self.name,
            self.at,
            self.buttons.len(),
            self.run.difficulty,
            self.run.character,
            self.run.shot_type,
            self.state.score,
            self.state.seed,
            self.state.lives,
            self.state.bombs,
            self.state.power,
            self.state.rank,
        )
    }
}

/// Where a playback is going, and what the chapter it is going to was written down as. Kept for
/// the moment it gets there, which is where the two are held against each other.
pub struct Landing {
    /// The stage frame the chapter began on.
    pub at: u32,
    pub chapter: u32,
    pub name: String,
    /// What the run had been rewound this many times when it was left, which the resumed run
    /// carries on counting from.
    pub retries: u32,
    /// Where the song was, for the music to be put back to.
    pub song: Option<i32>,
    /// The reproduction line written down for that frame.
    pub reproduction: String,
}

/// The buttons of the stage being played, and whatever a resume is in the middle of.
struct Record {
    /// Which stage the buttons belong to, and `None` when nothing is being kept.
    stage: Option<i32>,
    /// The run's numbers as that stage began, read where the game itself puts them in place.
    start: RunState,
    /// One entry per stage frame: the buttons the update of that frame acted on.
    buttons: Vec<u16>,
    /// Set while a saved run is being played in, and taken when it arrives.
    feeding: Option<Landing>,
    /// A saved run whose stage the game has been asked to build, waiting for it to be built.
    starting: Option<Saved>,
}

impl Record {
    const fn new() -> Self {
        Self {
            stage: None,
            start: RunState {
                score: 0,
                seed: 0,
                point_items: 0,
                power: 0,
                lives: 0,
                bombs: 0,
                rank: 0,
                power_items: 0,
                extra_lives: 0,
                deaths: 0,
            },
            buttons: Vec::new(),
            feeding: None,
            starting: None,
        }
    }

    /// Writes down what the update of `frame` is about to act on. The same frame reached twice —
    /// a chapter retried, or a frame the game spent with a menu up — is one entry, the last write
    /// winning, which is the one that update sees.
    fn note(&mut self, frame: u32, buttons: u16) {
        if self.stage.is_none() || frame >= MAX_STAGE_FRAMES {
            return;
        }
        let at = frame as usize;
        if self.buttons.len() <= at {
            self.buttons.resize(at + 1, 0);
        }
        self.buttons[at] = buttons;
    }

    /// What the update of `frame` is to act on while a run is being played in, and `None` once the
    /// chapter is reached — from there the keyboard is the player's again.
    fn feed(&self, frame: u32) -> Option<u16> {
        let at = self.feeding.as_ref()?.at;
        (frame < at)
            .then(|| self.buttons.get(frame as usize).copied())
            .flatten()
    }

    fn stop(&mut self) {
        self.stage = None;
        self.buttons.clear();
        self.feeding = None;
        self.starting = None;
    }
}

/// Whether the run being played is one whose inputs are worth keeping: pointdevice, and orb
/// allowed to keep them at all.
pub fn keep(on: bool) {
    KEEPING.store(on, Ordering::Relaxed);
}

/// What the game is to see on the keyboard for the frame about to run, while a saved run is being
/// played in. `None` leaves the read to the game.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn fed(frame: u32) -> Option<u16> {
    unsafe { RECORD.get() }.feed(frame)
}

/// Writes down what the game did see.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn noted(frame: u32, buttons: u16) {
    unsafe { RECORD.get() }.note(frame, buttons);
}

/// Where a stage is about to be built and nothing of it has been drawn from the generator yet: the
/// one moment the seed of a resumed run goes in.
///
/// Neither end of the callback that puts a stage's numbers in place will do, and the landing check
/// caught both: after it is two draws late, building a stage being what draws them
/// (`randoms=2 against randoms=0`), and on the way in is 2048 draws early, that callback filling a
/// table of keys out of the generator first (`seed=0x789c` where `0xc381` was written down, which is
/// what 2048 draws from `0xc381` come to). See [`crate::game::th06`]'s `STAGE_REGISTER_CHAIN`.
///
/// # Safety
/// Must run on the game's main thread, from the hook over [`crate::game::Hooks::stage_building`],
/// before the original.
pub unsafe fn stage_building(game: &dyn Game) {
    let record = unsafe { RECORD.get() };
    if let Some(saved) = &record.starting {
        unsafe { game.set_run_seed(saved.state.seed) };
        detail!("resume: the generator seeded {:#06x}", saved.state.seed);
    }
}

/// Where a stage's numbers have just been put in place by the game and its first update has not
/// run: the moment to read them, and the only moment to write a resumed run's over them.
///
/// # Safety
/// Must run on the game's main thread, from the hook over [`crate::game::Hooks::stage_begun`].
pub unsafe fn stage_begun(game: &dyn Game) {
    let record = unsafe { RECORD.get() };
    let state = unsafe { game.read_state() };
    // A run there is a slot to keep as well as one somebody is playing, so that a practice run is not
    // recorded and then found to have nowhere to go. The reinit state counts as playing, since a stage
    // reached from the one before it is registered from there.
    let keeping = KEEPING.load(Ordering::Relaxed)
        && state.in_run
        && !state.demo
        && !state.replay
        && game.run_slot(&unsafe { game.run_start() }).is_some();
    if let Some(saved) = record.starting.take() {
        // Two ways this is not the stage it asked for, said apart: what came up is a run orb has no
        // business writing down, or it is the wrong stage of one.
        if !keeping {
            record.stop();
            return log!(
                "resume: not put back — what came up is a demo, a replay, or a run in a mode that \
                 keeps nothing"
            );
        }
        if saved.run.stage != state.stage {
            record.stop();
            return log!(
                "resume: not put back — stage {} came up where stage {} was asked for",
                state.stage + 1,
                saved.run.stage + 1,
            );
        }
        unsafe { game.set_run_state(&saved.state) };
        record.stage = Some(state.stage);
        record.start = saved.state;
        log!("resume: {saved}; playing its buttons in");
        record.feeding = Some(Landing {
            at: saved.at,
            chapter: saved.chapter,
            name: saved.name,
            retries: saved.retries,
            song: saved.song,
            reproduction: saved.landing,
        });
        record.buttons = saved.buttons;
        return;
    }
    if !keeping {
        record.stop();
        // Said, because this hook firing and keeping nothing is the whole of what a demo or a
        // replay should do to it — and a session that never plays one has this as the only line
        // saying the hook is in the path at all.
        return detail!(
            "resume: stage {} of a run orb is not keeping; nothing of it is written down",
            state.stage + 1,
        );
    }
    record.stage = Some(state.stage);
    record.start = unsafe { game.run_state() };
    record.buttons.clear();
    record.feeding = None;
    detail!(
        "resume: stage {} begins with score={} seed={:#06x} lives={} bombs={} power={} rank={}",
        state.stage + 1,
        record.start.score,
        record.start.seed,
        record.start.lives,
        record.start.bombs,
        record.start.power,
        record.start.rank,
    );
}

/// The frame a playback is running to, while one is running.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn landing() -> Option<u32> {
    unsafe { RECORD.get() }
        .feeding
        .as_ref()
        .map(|landing| landing.at)
}

/// Hands the keyboard back, the chapter having been reached, and gives up what that chapter was
/// written down as so the two can be held against each other.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn landed() -> Option<Landing> {
    unsafe { RECORD.get() }.feeding.take()
}

/// Whether a saved run is waiting for the game to build its stage.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn starting() -> bool {
    unsafe { RECORD.get() }.starting.is_some()
}

/// Gives up on a resume, wherever it had got to.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn abandon(why: &str) {
    let record = unsafe { RECORD.get() };
    record.starting = None;
    record.feeding = None;
    log!("resume: given up — {why}");
}

/// Drops what is being kept, for a run that has ended: the stage those buttons belong to is being
/// torn down.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn forget() {
    unsafe { RECORD.get() }.stop();
}

/// Points the run the game is about to build at the stage the saved run was in, and holds its
/// numbers for the moment that stage is ready for them.
///
/// Nothing else about the run is written: the difficulty, the character and the shot are the ones
/// just chosen, and this is only ever a run whose slot says they are the saved one's.
///
/// # Safety
/// Must run on the game's main thread, between frames, on a frame [`Game::run_chosen`] is true of.
pub unsafe fn begin(game: &dyn Game, saved: Saved) -> bool {
    if !unsafe { game.start_stage(saved.run.stage) } {
        log!("resume: the game did not take {saved}");
        return false;
    }
    let record = unsafe { RECORD.get() };
    record.stop();
    log!("resume: {saved}; the run is being built at that stage");
    record.starting = Some(saved);
    true
}

/// Writes down where the run is, which is what the next launch offers to pick up. Says whether it
/// wrote.
///
/// # Safety
/// Must run on the game's main thread, on the frame the chapter began on — `at` is what says
/// which frame that is, and the reproduction line read here has to be that frame's.
pub unsafe fn write(
    dir: &Path,
    game: &dyn Game,
    at: u32,
    chapter: u32,
    name: &str,
    retries: u32,
) -> bool {
    let record = unsafe { RECORD.get() };
    let run = unsafe { game.run_start() };
    // Nothing said where there is no slot: a practice run is the ordinary case for that, not a fault.
    let Some(path) = slot(dir, game, &run) else {
        return false;
    };
    // The numbers held are the ones read as *that* stage began. A record of another stage's would be
    // a run put back with the wrong lives and the wrong seed, and nothing about the file would say
    // so, so it is not written at all.
    if record.stage != Some(run.stage) {
        return false;
    }
    // Every frame before the chapter, and there is one for each of them: a chapter is reached by
    // playing the frames under it, so a record that is short is a record that missed some.
    if record.buttons.len() < at as usize {
        detail!(
            "resume: not written — {} frame(s) of buttons for a chapter at {at}",
            record.buttons.len(),
        );
        return false;
    }
    let saved = Saved {
        run,
        state: record.start,
        at,
        chapter,
        name: name.to_owned(),
        retries,
        // Where the song is at this instant, which for a chapter beginning is where it will be put
        // back to on a retry — and now on a resume as well.
        song: game
            .music()
            .and_then(|music| unsafe { music.audible_offset() }),
        landing: unsafe { game.reproduction() }.to_string(),
        buttons: record.buttons[..at as usize].to_vec(),
    };
    // The directory is made where it is written rather than at startup, so an installation nobody
    // has left a run in has nothing of orb's beside the game but the log.
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        log!("resume: cannot make {}: {error}", parent.display());
        return false;
    }
    match std::fs::write(&path, encode(&saved)) {
        Ok(()) => {
            detail!("resume: {saved} written to {}", path.display());
            true
        }
        Err(error) => {
            log!("resume: cannot write {}: {error}", path.display());
            false
        }
    }
}

/// Where this run's chapter is kept: one file per run there can be one of, named as the game names
/// that run. `None` for a run the game keeps no slot for, which is a run nothing here is about.
fn slot(dir: &Path, game: &dyn Game, run: &RunStart) -> Option<PathBuf> {
    Some(
        dir.join(DIRECTORY)
            .join(game.run_slot(run)?)
            .with_extension(EXTENSION),
    )
}

/// The chapter left unfinished by a run of the same difficulty, character and shot as the one about
/// to be played, if there is one. A missing file is the ordinary case: no such run has been left.
///
/// The file's own idea of which run it is has to name the same slot. It always will, being what named
/// the file — so a mismatch is a file somebody moved or edited, and starting a run from it would be
/// starting the wrong one.
pub fn load(dir: &Path, game: &dyn Game, run: &RunStart) -> Option<Saved> {
    let path = slot(dir, game, run)?;
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            log!("resume: cannot read {}: {error}", path.display());
            return None;
        }
    };
    let saved = match decode(&bytes) {
        Some(saved) => saved,
        // Left where it is rather than deleted: a file that cannot be read is worth looking at, and
        // orb deleting it is what would stop anyone doing so.
        None => {
            log!(
                "resume: cannot make sense of {}; it is left alone",
                path.display()
            );
            return None;
        }
    };
    let (held, wanted) = (game.run_slot(&saved.run), game.run_slot(run));
    if held != wanted {
        log!(
            "resume: {} holds a run of {}, not {}; left alone",
            path.display(),
            held.unwrap_or_else(|| "no run with a slot".to_owned()),
            wanted.unwrap_or_else(|| "no run with a slot".to_owned()),
        );
        return None;
    }
    log!("resume: {} holds {saved}", path.display());
    Some(saved)
}

/// What a named slot holds, as the line a mark on the front end shows, and `None` where it holds
/// nothing or nothing that can be read.
///
/// By name rather than by run, because this is asked of a run being *pointed* at rather than one
/// being started, and there is nothing to check the file's own idea of itself against yet. Silent
/// about a file it cannot read, unlike [`load`]: the front end is not the place to say so, and the
/// answer is the same either way — nothing to offer.
pub fn peek(dir: &Path, slot: &str) -> Option<String> {
    let path = dir.join(DIRECTORY).join(slot).with_extension(EXTENSION);
    let bytes = std::fs::read(path).ok()?;
    Some(decode(&bytes)?.describe())
}

/// What runs have been left unfinished, as the names of their files, for the log at startup: which
/// of them the question will offer depends on what is chosen, and this is what says none of them was
/// there to be offered.
pub fn left(dir: &Path) -> Vec<String> {
    let mut slots: Vec<String> = std::fs::read_dir(dir.join(DIRECTORY))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|it| it == EXTENSION))
        .filter_map(|path| Some(path.file_stem()?.to_string_lossy().into_owned()))
        .collect();
    // In order, so that a session's line and a directory listing read the same way.
    slots.sort();
    slots
}

/// Takes it away, for a run that has finished rather than been left: there is no chapter to come
/// back to, and an offer to pick up a run that is over is worse than no offer.
pub fn discard(dir: &Path, game: &dyn Game, run: &RunStart) -> Option<PathBuf> {
    let path = slot(dir, game, run)?;
    match std::fs::remove_file(&path) {
        Ok(()) => {
            log!("resume: {} removed", path.display());
            Some(path)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            log!("resume: cannot remove {}: {error}", path.display());
            None
        }
    }
}

/// The file's own shape, which is not the game's structs: the names in it are the file's business,
/// so that a field renamed in [`RunStart`] is not a file that reads differently afterwards.
///
/// `deny_unknown_fields` throughout, because a field nothing knows is a file from another version
/// or another program, and reading one of those halfway is worse than not reading it.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    version: u32,
    run: WrittenRun,
    chapter: WrittenChapter,
    start: WrittenStart,
    /// The reproduction line exactly as the log prints it, kept as the one string it is: a landing is
    /// held against it by field name — see [`differs`] — so what is compared is this text either way,
    /// and the file shows the line somebody can hold against the log by eye.
    landing: String,
    /// The buttons held from each frame they changed on, keyed by that frame.
    ///
    /// Only the changes, which is how the game writes its own record: everything between two of them
    /// is the first one's, and a stage's worth of frames comes to a few hundred entries. A map rather
    /// than a list of pairs, so that they are in order because a map of frames is, rather than because
    /// something checked; it is also a byte less each, and `--dump` prints one per line where a list
    /// of pairs is two.
    buttons: BTreeMap<u32, u16>,
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
struct WrittenChapter {
    number: u32,
    frame: u32,
    retries: u32,
    name: String,
    /// Where the song was in the track's own file, and `null` where the stream would not say — which
    /// is a resume that leaves the music as the stage started it.
    song: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenStart {
    score: u32,
    #[serde(with = "hex")]
    seed: u16,
    points: u16,
    power: u16,
    lives: i8,
    bombs: i8,
    rank: i32,
    power_items: i8,
    extra_lives: i8,
    deaths: i32,
}

/// A number written the way every line of the log that mentions this one writes it. The seed is the
/// generator's state rather than a quantity, and `0x1a2b` is what it is called everywhere else.
mod hex {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(value: &u16, into: S) -> Result<S::Ok, S::Error> {
        into.serialize_str(&format!("{value:#06x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<u16, D::Error> {
        let text = String::deserialize(from)?;
        super::number(&text)
            .and_then(|value| u16::try_from(value).ok())
            .ok_or_else(|| D::Error::custom(format!("not a number: {text}")))
    }
}

/// The file, whole. Written every time rather than appended to: it describes one chapter of one
/// run, and the one it describes is the newest.
fn encode(saved: &Saved) -> Vec<u8> {
    let written = Written {
        version: VERSION,
        run: WrittenRun {
            difficulty: saved.run.difficulty,
            character: saved.run.character,
            shot: saved.run.shot_type,
            stage: saved.run.stage,
        },
        chapter: WrittenChapter {
            number: saved.chapter,
            frame: saved.at,
            retries: saved.retries,
            name: saved.name.clone(),
            song: saved.song,
        },
        start: WrittenStart {
            score: saved.state.score,
            seed: saved.state.seed,
            points: saved.state.point_items,
            power: saved.state.power,
            lives: saved.state.lives,
            bombs: saved.state.bombs,
            rank: saved.state.rank,
            power_items: saved.state.power_items,
            extra_lives: saved.state.extra_lives,
            deaths: saved.state.deaths,
        },
        landing: saved.landing.clone(),
        buttons: changes(&saved.buttons),
    };
    // With the names in it, so that the file says what its own fields are: see [`EXTENSION`]. Nothing
    // in it can fail to serialize — every field is a number, a string or a pair of numbers — so a
    // failure here is a file that would say nothing about the run, which is what no bytes read as.
    rmp_serde::to_vec_named(&written).unwrap_or_default()
}

/// Reads it back, and `None` for anything that is not a file this version wrote: a field missing or
/// unknown, a number that is not one, a version that is not this one.
fn decode(bytes: &[u8]) -> Option<Saved> {
    let written: Written = rmp_serde::from_slice(bytes).ok()?;
    if written.version != VERSION {
        return None;
    }
    let changes = written.buttons;
    let at = written.chapter.frame;
    // Nothing at or past the frame the buttons are meant to reach, that being a record which does not
    // describe the chapter beside it. They are in order already, a map of frames being kept that way.
    if at > MAX_STAGE_FRAMES || changes.keys().last().is_some_and(|frame| *frame >= at) {
        return None;
    }
    Some(Saved {
        run: RunStart {
            difficulty: written.run.difficulty,
            character: written.run.character,
            shot_type: written.run.shot,
            // Not in the file, there being nothing to say: a practice run has no slot to be written
            // in — see `Game::run_slot` — so every file is a full run's.
            practice: false,
            stage: written.run.stage,
        },
        state: RunState {
            score: written.start.score,
            seed: written.start.seed,
            point_items: written.start.points,
            power: written.start.power,
            lives: written.start.lives,
            bombs: written.start.bombs,
            rank: written.start.rank,
            power_items: written.start.power_items,
            extra_lives: written.start.extra_lives,
            deaths: written.start.deaths,
        },
        at,
        chapter: written.chapter.number,
        name: written.chapter.name,
        retries: written.chapter.retries,
        song: written.chapter.song,
        landing: written.landing,
        buttons: dense(&changes, at),
    })
}

/// The frames the buttons changed on, out of one entry per frame: everything between two changes is
/// the first one's, which is what a run with a key held down is.
fn changes(buttons: &[u16]) -> BTreeMap<u32, u16> {
    let mut changes = BTreeMap::new();
    let mut held = 0;
    for (frame, pressed) in buttons.iter().enumerate() {
        if *pressed != held {
            changes.insert(frame as u32, *pressed);
            held = *pressed;
        }
    }
    changes
}

/// One entry per frame again, out of the frames the buttons changed on.
fn dense(changes: &BTreeMap<u32, u16>, upto: u32) -> Vec<u16> {
    let mut buttons = vec![0; upto as usize];
    let mut frames = changes.keys().copied().skip(1);
    for (frame, held) in changes {
        let until = frames.next().unwrap_or(upto).min(upto);
        for slot in &mut buttons[*frame as usize..until as usize] {
            *slot = *held;
        }
    }
    buttons
}

/// The first field two reproduction lines disagree on, and `None` where they agree.
///
/// Field by field rather than line against line, because what a resume that landed somewhere else
/// needs said is which number came out differently: that names what a stage reads at its start and
/// is not written down. By name and not by position, which is how the file's own fields are read.
///
/// Two lines of different shapes are said whole, there being no field to point at: a field one of
/// them has not is a line something other than this version wrote, and nothing of it is to be read as
/// agreeing.
pub fn differs(written: &str, landed: &str) -> Option<String> {
    /// Into a set and not a list: what makes two lines the same shape is which fields they
    /// carry, and holding them in the order they were written would call a line whose fields
    /// come in another order a line of some other version.
    fn keys(line: &str) -> BTreeSet<&str> {
        line.split_whitespace()
            .filter_map(|field| field.split_once('=').map(|(key, _)| key))
            .collect()
    }
    if keys(written) != keys(landed) {
        return Some(format!("{written} against {landed}"));
    }
    let written = Fields(written);
    landed.split_whitespace().find_map(|field| {
        let (key, value) = field.split_once('=')?;
        let theirs = written.text(key)?;
        (theirs != value).then(|| format!("{key}={theirs} against {key}={value}"))
    })
}

/// The `key=value` pairs of a reproduction line, which is the one thing here that is a line rather
/// than a field of the file: it is the log's own, held against another of the log's own by name.
struct Fields<'a>(&'a str);

impl<'a> Fields<'a> {
    fn text(&self, key: &str) -> Option<&'a str> {
        self.0.split_whitespace().find_map(|field| {
            let (name, value) = field.split_once('=')?;
            (name == key).then_some(value)
        })
    }
}

/// A number as this file writes them: decimal, or hexadecimal where it is one to hold against the
/// log, which writes the seed and a frame's buttons that way.
fn number(text: &str) -> Option<i64> {
    let text = text.trim();
    match text.strip_prefix("0x") {
        Some(hex) => i64::from_str_radix(hex, 16).ok(),
        None => text.parse().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_STAGE_FRAMES, Record, Saved, VERSION, decode, encode};
    use crate::game::{RunStart, RunState};

    pub fn saved() -> Saved {
        Saved {
            run: RunStart {
                difficulty: 3,
                character: 1,
                shot_type: 0,
                practice: false,
                stage: 3,
            },
            state: RunState {
                score: 7417420,
                seed: 0x1a2b,
                point_items: 328,
                power: 128,
                lives: 2,
                bombs: 3,
                rank: 21,
                power_items: 44,
                extra_lives: 1,
                deaths: 6,
            },
            at: 12,
            chapter: 17,
            name: "BOSS SPELL 2".to_owned(),
            retries: 42,
            song: Some(1234567),
            landing: "replay_frame=-1 input=0x0000 randoms=1234".to_owned(),
            buttons: vec![0, 0, 1, 1, 1, 0x41, 0x41, 0, 0, 0, 0x101, 0x101],
        }
    }

    /// Every field of it, since a resume is a run started from these numbers: one read back as
    /// another is a run that plays out differently and says nothing about why.
    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(decode(&encode(&saved())), Some(saved()));
    }

    /// The buttons are written as the frames they changed on and come back one per frame, holding
    /// the last change through to the chapter — which is what a run with a key held down is.
    #[test]
    fn the_buttons_come_back_one_per_frame() {
        let mut held = saved();
        held.at = 6;
        held.buttons = vec![0, 0, 1, 1, 1, 1];
        // One entry, not six frames.
        assert_eq!(
            super::changes(&held.buttons),
            std::collections::BTreeMap::from([(2, 1)]),
        );
        assert_eq!(decode(&encode(&held)), Some(held));
    }

    /// A stream that would not say where the song was is written as nothing rather than as a number
    /// standing for nothing, and read back as a resume that leaves the music as the stage started it.
    #[test]
    fn a_chapter_with_no_song_position_is_read_as_having_none() {
        let mut quiet = saved();
        quiet.song = None;
        assert_eq!(decode(&encode(&quiet)), Some(quiet));
    }

    /// A chapter at the stage's own start has none of them, which is a run resumed by building
    /// the stage and playing nothing.
    #[test]
    fn a_stage_start_holds_no_buttons() {
        let mut start = saved();
        start.at = 0;
        start.chapter = 1;
        start.name = "MIDSTAGE 1".to_owned();
        start.buttons = Vec::new();
        assert_eq!(decode(&encode(&start)), Some(start));
    }

    /// A file this version did not write is refused rather than read with its fields taken for each
    /// other, and so is one that has lost a field, gained one nothing knows, or holds a field of
    /// another kind than it should.
    ///
    /// Written as MessagePack maps here rather than by editing bytes, since what has to be refused is
    /// a *file* of that shape however it came to be one.
    #[test]
    fn a_file_this_version_did_not_write_is_refused() {
        use rmpv::Value;

        let whole = || match rmpv::decode::read_value(&mut encode(&saved()).as_slice()) {
            Ok(Value::Map(fields)) => fields,
            other => panic!("not a map: {other:?}"),
        };
        let refused = |fields: Vec<(Value, Value)>| {
            let mut bytes = Vec::new();
            rmpv::encode::write_value(&mut bytes, &Value::Map(fields)).expect("written");
            assert!(decode(&bytes).is_none());
        };
        let named = |name: &str| Value::String(name.into());
        let replacing = |name: &str, with: Value| {
            let mut fields = whole();
            let at = fields
                .iter()
                .position(|(key, _)| *key == named(name))
                .expect("a field of that name");
            fields[at].1 = with;
            fields
        };

        // The whole of it, untouched, is the file that is read — everything below is that file with
        // one thing wrong with it.
        assert_eq!(decode(&encode(&saved())), Some(saved()));

        refused(replacing("version", Value::from(VERSION + 1)));
        refused(replacing("landing", Value::from(0)));
        refused(
            whole()
                .into_iter()
                .filter(|(key, _)| *key != named("start"))
                .collect(),
        );
        refused({
            let mut fields = whole();
            fields.push((named("elsewhere"), Value::from(1)));
            fields
        });
        // A frame at or past the one the buttons are meant to reach is a record that does not
        // describe the chapter beside it.
        let held_from = |frames: [u32; 2]| {
            Value::Map(
                frames
                    .iter()
                    .map(|frame| (Value::from(*frame), Value::from(1)))
                    .collect(),
            )
        };
        refused(replacing("buttons", held_from([2, 12])));
        refused(replacing("buttons", held_from([2, saved().at])));
    }

    /// A chapter played again writes over the frames it plays again, so what a chapter's own save
    /// holds is the attempt that survived to reach it. Which is why a restore drops nothing here.
    #[test]
    fn an_attempt_played_again_writes_over_the_one_before_it() {
        let mut record = Record::new();
        record.stage = Some(3);
        for frame in 0..10 {
            record.note(frame, 0x001);
        }
        // Back to the chapter's own frame, and played differently from there.
        for frame in 4..10 {
            record.note(frame, 0x041);
        }
        assert_eq!(record.buttons[..4], [0x001; 4]);
        assert_eq!(record.buttons[4..], [0x041; 6]);
    }

    /// Nothing is kept while nothing is being played, so a session spent in the menus does not
    /// grow a record of what was pressed there.
    #[test]
    fn nothing_is_kept_between_runs() {
        let mut record = Record::new();
        record.note(0, 0x001);
        assert!(record.buttons.is_empty());
        record.stage = Some(0);
        record.note(0, 0x001);
        record.stop();
        assert!(record.buttons.is_empty());
    }

    /// A landing that is not the one written down says which number it was, since that is what
    /// names the thing a stage reads at its start and orb does not write down.
    #[test]
    fn a_landing_that_differs_says_which_field_it_was() {
        let written = "replay_frame=-1 randoms=1234 score=7417420";
        assert_eq!(super::differs(written, written), None);
        assert_eq!(
            super::differs(written, "replay_frame=-1 randoms=1200 score=7417420"),
            Some("randoms=1234 against randoms=1200".to_owned()),
        );
        // The same fields in another order still agree, being read by name.
        assert_eq!(
            super::differs(written, "score=7417420 replay_frame=-1 randoms=1234"),
            None,
        );
        // And one of them differing is still named, wherever in the line it stands.
        assert_eq!(
            super::differs(written, "score=7417420 randoms=1200 replay_frame=-1"),
            Some("randoms=1234 against randoms=1200".to_owned()),
        );
        // Two lines of different shapes have no field to point at, so both are said whole.
        assert!(
            super::differs(written, "replay_frame=-1")
                .is_some_and(|said| said.contains("against replay_frame=-1")),
        );
    }

    /// A frame number no stage reaches is not one to allocate for, whatever asked.
    #[test]
    fn a_frame_no_stage_reaches_is_refused() {
        let mut record = Record::new();
        record.stage = Some(0);
        record.note(MAX_STAGE_FRAMES, 0x001);
        assert!(record.buttons.is_empty());

        let mut far = saved();
        far.at = MAX_STAGE_FRAMES + 1;
        assert!(decode(&encode(&far)).is_none());
    }

    /// The buttons are handed back for the frames before the chapter and not for the chapter's
    /// own frame: from there the keyboard is the player's again.
    #[test]
    fn the_buttons_are_fed_up_to_the_chapter_and_no_further() {
        let mut record = Record::new();
        record.stage = Some(3);
        record.buttons = vec![0, 1, 2, 3];
        record.feeding = Some(super::Landing {
            at: 3,
            chapter: 17,
            name: String::new(),
            retries: 0,
            song: None,
            reproduction: String::new(),
        });
        assert_eq!(record.feed(0), Some(0));
        assert_eq!(record.feed(2), Some(2));
        assert_eq!(record.feed(3), None);
        record.feeding = None;
        assert_eq!(record.feed(0), None);
    }
}
