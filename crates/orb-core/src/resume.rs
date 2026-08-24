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
//! **One per run there can be one of**, the way th15 keeps a pointdevice save per difficulty and
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

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

use crate::game::{Game, RunStart};
use crate::sync::MainThread;
use crate::{detail, log};

/// A directory beside the game, and one file in it per run there is a chapter of — named for that
/// run, so a listing of it reads as the runs somebody has left unfinished.
///
/// A directory rather than one file holding them all, because the file is written whole every time a
/// chapter begins: with every run in it, that write would be every run's buttons a few seconds
/// apart, and what it is written for is one of them.
const DIRECTORY: &str = "pointdevice_resume";
/// MessagePack, with the field names in it — `to_vec_named`, not the positional arrays rmp-serde
/// writes by default. What that buys is that the file needs nothing but itself to be read, so **nothing here
/// reads one out for anybody**: a converter does, and reading one beside the log is what has caught every fault
/// in this machinery so far.
///
/// `msgpack2json -d -i <file>` is the reading, and the `-d` is not optional — its plain mode refuses a map whose
/// keys are numbers, which [`Written::buttons`] is. A launcher option for this was begun once and thrown away:
/// a self-describing file needs no reader of orb's, and one written anyway is a second thing to keep in step
/// with the first.
///
/// Packed rather than text because it is the machine's own: a stage's buttons written as lines came
/// to 30KB by a boss chapter deep in a stage, against 11KB here, and nobody reads a button mask by
/// eye. The two fields anybody does read — the seed and the reproduction line — are strings either
/// way.
const EXTENSION: &str = "msgpack";

/// Longer than any stage of any run: half an hour of one at 60 frames a second. A bound on what a
/// file can ask to be allocated for, and on the frame number a record will write down at all.
pub(crate) const MAX_STAGE_FRAMES: u32 = 60 * 60 * 30;

/// Whether a run's inputs are being kept at all.
///
/// An atomic because the two hooks that ask — the game's input read and the moment a stage's
/// numbers are put in place — run inside the game's own update, where the `Runtime` that knows
/// which mode orb is in is already borrowed.
static KEEPING: AtomicBool = AtomicBool::new(false);

static RECORD: MainThread<Record> = MainThread::new(Record::new());

/// **What putting a run back takes**: which run it was, the numbers the stage started with, the buttons of
/// every frame since and the generator each was pressed with, and where the chapter and its song are.
///
/// Not *the file* — the file is the game's, and what it holds is that game's business: see
/// [`Game::encode_resume`], and the two games' own `resume` modules for what each writes. This is the other
/// side of that seam, and it is the whole of what the machinery here knows about a kept run.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Kept {
    pub run: RunStart,
    /// The numbers that stage started with, **in the game's own bytes and opaque here** — see
    /// [`Game::run_state`], which is where what a run *is* stops being anything this module knows. They came
    /// out of the game and go back into it, and into its own file in between.
    pub state: Vec<u8>,
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
    /// And the generator's own state on each of those frames, which is what says whether a playback is still
    /// reproducing the run — see [`Record::feed`]. One entry per frame like the buttons, and no shorter, with
    /// nothing for a frame the run has no word for: a playback compares the frames that have one and says
    /// nothing about the rest.
    ///
    /// **Per frame and not all-or-none.** They were all-or-none, and a run picked up out of a file from before
    /// orb wrote them down — every file on a disk, [`VERSION`] having stayed at 2 for them — wrote a list with
    /// nothing in it at every chapter after that, because the frames it had played back had no word to carry
    /// forward. One such file made every file after it as blind as itself, and a th07 session was played
    /// through with the only check that names a parting frame switched off.
    pub generators: Vec<Option<u16>>,
}

impl Kept {
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

impl fmt::Display for Kept {
    /// For the log, where what is wanted is the numbers rather than the names: this is what a
    /// landing is held against.
    ///
    /// **And whether the generators are in it**, which is the one thing about a file that decides whether
    /// anything can say where a playback of it parted. A session was played out of a file that had none and
    /// nothing in the log mentioned it, so the parting line never came and its absence read as a playback
    /// that had not parted. Said at both ends, this being the line the writer hands a file over with and the
    /// line a launch picks one up with.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let with_a_word = self.generators.iter().flatten().count();
        // **The run's own numbers are not here.** They are the game's bytes — see [`Kept::state`] — so what
        // says them is the game, and the two lines this one is written into say it after this: `Game::run_state`
        // for why, and `Game::say_run_state` for the words.
        write!(
            f,
            "stage {} chapter {} ({}) at frame {}, {} frame(s) of buttons {}, diff={} char={}-{}",
            self.run.stage + 1,
            self.chapter,
            self.name,
            self.at,
            self.buttons.len(),
            if with_a_word == self.buttons.len() {
                "and the generator each was played with".to_owned()
            } else if with_a_word == 0 {
                "and no generator for any of them".to_owned()
            } else {
                format!("and the generator on {with_a_word} of them")
            },
            self.run.difficulty,
            self.run.character,
            self.run.shot_type,
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

/// What one frame of a stage was played with: the buttons its update acted on, and the generator's own state as
/// it did.
///
/// **One entry and not two lists**, because two lists are two lengths. They were two, and a playback that ended
/// early left the buttons at the length its file had and the generators empty — the first frame written down
/// after that indexed the short one, and a th07 launch crashed where a run was picked up:
/// `index out of bounds: the len is 0 but the index is 3609`.
#[derive(Clone, Copy, Default)]
struct Played {
    buttons: u16,
    /// Held against what the generator is on that frame while a run is played back — see [`Record::feed`].
    ///
    /// `None` for a frame out of a file written before orb wrote them down: such a playback is fed its buttons
    /// and says nothing about where it parted, which is what a file with nothing in it to compare deserves.
    generator: Option<u16>,
}

/// The buttons of the stage being played, and whatever a resume is in the middle of.
struct Record {
    /// Which stage the buttons belong to, and `None` when nothing is being kept.
    stage: Option<i32>,
    /// The run's numbers as that stage began, read where the game itself puts them in place — the game's own
    /// bytes, and empty while nothing is being kept.
    start: Vec<u8>,
    /// One entry per stage frame — see [`Played`].
    played: Vec<Played>,
    /// The frame a playback was first found to have parted from the run it is reproducing, so that it is said
    /// once rather than for every frame after it.
    parted: Option<u32>,
    /// The last frame a playback was asked for buttons for, and whether the frames it has been asked for went
    /// in order — see [`Record::feed`], which is where a gap in them is said.
    fed: Option<u32>,
    skipped: bool,
    /// Set while a saved run is being played in, and taken when it arrives.
    feeding: Option<Landing>,
    /// A saved run whose stage the game has been asked to build, waiting for it to be built.
    starting: Option<Kept>,
}

impl Record {
    /// **`const fn` because [`RECORD`] is a static**, which is also why no e2e test can enter it: what
    /// happens here is const evaluation and not execution, so a coverage run reports it never run however
    /// many runs were written down. See `game::proposed`, which is the same zero.
    const fn new() -> Self {
        Self {
            stage: None,
            start: Vec::new(),
            played: Vec::new(),
            parted: None,
            fed: None,
            skipped: false,
            feeding: None,
            starting: None,
        }
    }

    /// Writes down what the update of `frame` is about to act on. The same frame reached twice —
    /// a chapter retried, a frame the game spent with a menu up, or one th07's own slowdown decided
    /// not to advance the run on — is one entry, the last write winning, which is the one that update
    /// sees.
    fn note(&mut self, frame: u32, buttons: u16, generator: u16) {
        if self.stage.is_none() || frame >= MAX_STAGE_FRAMES {
            return;
        }
        let at = frame as usize;
        if self.played.len() <= at {
            self.played.resize(at + 1, Played::default());
        }
        self.played[at] = Played {
            buttons,
            generator: Some(generator),
        };
    }

    /// What the update of `frame` is to act on while a run is being played in, and `None` once the
    /// chapter is reached — from there the keyboard is the player's again.
    ///
    /// **And the generator held against what it was on that frame**, which is the only thing that says a
    /// playback has stopped reproducing the run *and where*. The buttons cannot: a run that has parted plays on
    /// with the same presses landing on a different stage, and what shows up is a death thousands of frames
    /// later or a landing check at the end — neither of which says which frame it went wrong on. A th07 run
    /// picked up out of its file was hit at frame 1206 of 8553 and had died seven times by the chapter, and the
    /// generator at the stage's ninth frame was already `0x3874` where the run had `0x4289`.
    ///
    /// Said once, the first frame being the one with the cause in it, and the playback carried on so that the
    /// landing check still gets its say.
    fn feed(&mut self, frame: u32, generator: u16) -> Option<u16> {
        let at = self.feeding.as_ref()?.at;
        if frame >= at {
            return None;
        }
        // **The frames a playback is asked for go one at a time**, the same frame twice being ordinary — the
        // game reads the keyboard more than once on a frame it spends with a menu up, and on one its own
        // slowdown does not advance the run on. One that *jumps* is not: the buttons of the frames stepped over
        // are never fed to anything, so the update that would have acted on them acts on the frame before's,
        // and a run played with a press missing looks like a run that simply went differently. Said once, this
        // being a thing about the playback rather than about a frame of it, and needing nothing written down
        // about the run to catch — which is what the generator below needs.
        if let Some(last) = self.fed
            && frame != last
            && frame != last + 1
            && !self.skipped
        {
            self.skipped = true;
            log!(
                "resume: the playback was fed frame {frame} straight after frame {last}, which is not \
                 the next frame of the run"
            );
        }
        self.fed = Some(frame);
        let played = self.played.get(frame as usize).copied()?;
        let word = played.buttons;
        // **Not written down as this playback's where the run has none.** A fed frame is one nothing reads the
        // keyboard for, so [`Record::note`] never sees it again and a word put here would be carried into every
        // file after it as the run's own — and the frames with none are exactly the frames a playback is
        // suspected of parting on. What the file would then hold against the next playback is the parting
        // itself, which is a check that agrees with whatever went wrong.
        if self.parted.is_none()
            && let Some(was) = played.generator
            && was != generator
        {
            self.parted = Some(frame);
            log!(
                "resume: the playback parted from the run at frame {frame}: the generator is \
                 {generator:#06x} where the run had {was:#06x}"
            );
        }
        Some(word)
    }

    fn stop(&mut self) {
        self.stage = None;
        self.played.clear();
        self.parted = None;
        self.fed = None;
        self.skipped = false;
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
/// `generator` is the game's own generator now, which is held against what it was on that frame of the run
/// being played back — see [`Record::feed`].
pub unsafe fn fed(frame: u32, generator: u16) -> Option<u16> {
    unsafe { RECORD.get() }.feed(frame, generator)
}

/// Writes down what the game did see, and the generator it saw it with.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn noted(frame: u32, buttons: u16, generator: u16) {
    unsafe { RECORD.get() }.note(frame, buttons, generator);
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
        unsafe { game.seed_the_stage(&saved.state) };
        detail!(
            "resume: the stage seeded from {}",
            game.say_run_state(&saved.state),
        );
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
        log!(
            "resume: {saved} {}; playing its buttons in",
            game.say_run_state(&saved.state),
        );
        record.start = saved.state;
        // **Said on this way in as well as the other**, and after the numbers have gone in, so that the two
        // entries into a stage can be held against each other at all: the line below is what a launch that
        // *walked* into the stage says, and a comparison needs both halves in the same shape. See
        // [`Game::the_runs_own_bytes`].
        detail!(
            "resume: stage {} begins with {} rng={:#06x}{}",
            state.stage + 1,
            game.say_run_state(&record.start),
            unsafe { game.generator() },
            match unsafe { game.the_runs_own_bytes() } {
                Some(said) => format!(" {said}"),
                None => String::new(),
            },
        );
        unsafe { hold_captures(game) };
        record.feeding = Some(Landing {
            at: saved.at,
            chapter: saved.chapter,
            name: saved.name,
            retries: saved.retries,
            song: saved.song,
            reproduction: saved.landing,
        });
        // Both halves of every frame, which is what the playback is held to: the buttons to feed and the
        // generator each of them was pressed with — see [`Played`] and [`Record::feed`]. The file's own two
        // lists are the same length, [`decode`] refusing one where they are not.
        record.played = saved
            .buttons
            .iter()
            .enumerate()
            .map(|(frame, buttons)| Played {
                buttons: *buttons,
                generator: saved.generators.get(frame).copied().flatten(),
            })
            .collect();
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
    record.played.clear();
    record.parted = None;
    record.feeding = None;
    // **And the generator as the build leaves it**, which is the one point of the three a parting needs that
    // nothing was saying. A stage picked up is seeded at [`stage_building`] with the word the run recorded and
    // reaches its first frame drawing the same *count* of numbers as the run did and landing on a different one:
    // two of the three points are said — the seeding, and the frame — and this is the middle. What it settles is
    // whether the build itself draws differently on the two ways into a stage, a run's own being a transition
    // and a resumed one's being a stage built as a run's first.
    detail!(
        "resume: stage {} begins with {} rng={:#06x}{}",
        state.stage + 1,
        game.say_run_state(&record.start),
        unsafe { game.generator() },
        match unsafe { game.the_runs_own_bytes() } {
            Some(said) => format!(" {said}"),
            None => String::new(),
        },
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

/// What the game's record of captured spell cards was before a run's buttons were played back into
/// it, held so that the playback leaves it alone.
///
/// Playing a stage in again starts every card the run had passed, and the game counts an attempt at
/// each — a resumed run would arrive with an attempt, and a capture, against cards nobody had played
/// this session. So it is taken at [`begin`] and put back at [`landed_captures`].
///
/// The counts alone, not the names the playback wrote with them: see
/// [`Game::set_captures_keeping_names`](crate::game::Game::set_captures_keeping_names).
static CAPTURES: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

/// Holds the record while the buttons go in.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn hold_captures(game: &dyn Game) {
    if let Ok(mut held) = CAPTURES.lock() {
        *held = unsafe { game.captures() };
    }
}

/// Puts back what the playback counted, the playback being over, and says how many bytes went back.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn landed_captures(game: &dyn Game) -> usize {
    let Ok(mut held) = CAPTURES.lock() else {
        return 0;
    };
    let held = std::mem::take(&mut *held);
    if held.is_empty() {
        return 0;
    }
    unsafe { game.set_captures_keeping_names(&held) };
    held.len()
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
pub unsafe fn begin(game: &dyn Game, saved: Kept) -> bool {
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
    if record.played.len() < at as usize {
        detail!(
            "resume: not written — {} frame(s) of buttons for a chapter at {at}",
            record.played.len(),
        );
        return false;
    }
    let saved = Kept {
        run,
        // Cloned rather than taken: the record goes on being the record of this stage, and the next chapter
        // of it is written from the same numbers.
        state: record.start.clone(),
        at,
        chapter,
        name: name.to_owned(),
        retries,
        // Where the song is at this instant, which for a chapter beginning is where it will be put
        // back to on a retry — and now on a resume as well.
        song: game
            .music()
            .and_then(|music| unsafe { music.audible_offset() }),
        landing: unsafe { game.reproduction() },
        buttons: record.played[..at as usize]
            .iter()
            .map(|played| played.buttons)
            .collect(),
        // One per frame, and nothing for a frame the run has no word for. A frame with none is one played
        // back out of a file that had none for it, and what the frames beside it hold is what says where a
        // later playback of *them* parts — see the game's own file for whether it will carry such a frame at
        // all: `th07::resume` refuses a file with any frame unaccounted for, `th06::resume` cannot.
        generators: record.played[..at as usize]
            .iter()
            .map(|played| played.generator)
            .collect(),
    };
    // The game's own file, asked for here where the game is at hand: the thread that writes it has none.
    handed_over(Handed {
        path,
        bytes: game.encode_resume(&saved),
        said: format!("{saved} {}", game.say_run_state(&saved.state)),
        chapter: saved.chapter,
    });
    true
}

/// A chapter on its way to the disk: where it goes, the bytes to put there, and what the log calls it.
///
/// **The bytes and not the chapter itself**, because the bytes are the *game's* — see
/// `Game::encode_resume`, which is where a file's shape lives now — and the thread that writes them has no
/// game to ask. Reading the run and asking its game for its file are both the frame's work; what is left
/// for the thread is the write, which is the part that takes a disk's time.
struct Handed {
    path: PathBuf,
    bytes: Vec<u8>,
    /// The line the write is said with, worked out where the chapter still is one.
    said: String,
    /// And its number, for the line saying a chapter was overtaken before it reached the disk.
    chapter: u32,
}

/// What the thread that writes chapters down is waiting for.
///
/// **One chapter and not a queue.** What the file holds is where the run is *now*, so a chapter still
/// waiting when the next one arrives is one nothing wants: the newer answers the same question, and a
/// queue would spend a slow disk's time writing places the run has already left.
struct Waiting {
    next: Option<Handed>,
    /// Whether the writer has been asked to stop, and whether it has — see [`stop_writing`].
    stopping: bool,
    stopped: bool,
}

static WAITING: Mutex<Waiting> = Mutex::new(Waiting {
    next: None,
    stopping: false,
    stopped: false,
});
/// What the writer waits on for a chapter and the stopper waits on for the writer, so that neither
/// costs anything while nothing is happening.
static CHANGED: Condvar = Condvar::new();
/// Whether there is a writer. Cleared by [`stop_writing`], so that a launch after one that was taken
/// out gets a thread of its own rather than handing chapters to a thread that has gone.
static WRITING: AtomicBool = AtomicBool::new(false);

/// How long [`stop_writing`] waits for the writer to finish what it holds, in milliseconds.
///
/// Long enough for a write that is slow — which is what this whole arrangement is about — and not long
/// enough to hold a game that is closing behind a disk that has stopped answering. A chapter lost to
/// that is the chapter before it still being on the disk, which is the arrangement anyway.
const STOP_WAIT_MS: u64 = 2_000;

/// Where a write is slow enough to be worth a line at the level a run is ordinarily logged at, in
/// microseconds.
///
/// A frame at sixty a second, because that is the unit the thing being watched for is measured in: a
/// write that fits inside one would have cost the frame nothing even in the frame, and one that does not
/// is the reason the write is not in the frame any more.
const SLOW_WRITE_US: i64 = 16_666;

/// Hands the chapter to the thread that writes it, and starts that thread the first time.
///
/// **Not in the frame**, where it was: `WriteFile` takes what a disk, an antivirus reading every write or
/// a folder being synced decides, none of which orb can ask about beforehand, and a frame that waits for
/// one waits that long. What only the frame can do is the *reading* — the numbers, the song's position
/// and the reproduction line are that frame's — so that is the frame's part and no more of it.
///
/// **Not held for the frame's own slack** either, which is what [`crate::log`] does with its lines and
/// is the answer that suggests itself here: `defer` keeps them and `drain` writes them where what is left
/// of the turn is nobody's. The slack is what remains of one frame's turn, so a write of about a second
/// put there misses the next frame's blank instead of this one's — the same stutter, one frame later.
///
/// Where no thread can be made, the write happens here instead: a chapter written down in the frame is
/// worse than a frame, and a chapter not written down at all is a run nobody can pick up.
fn handed_over(handed: Handed) {
    if WRITING
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
        // Through the seam, which carries whatever simulated Windows this thread reads through onto
        // the one being made — see `orb_api::thread::spawn`.
        && orb_api::thread::spawn(writing).is_err()
    {
        log!("resume: cannot start a thread; chapters are written down in the frame");
        WRITING.store(false, Ordering::Relaxed);
        written(&handed);
        return;
    }
    let Ok(mut waiting) = WAITING.lock() else {
        return log!("resume: the writer left its own state locked; nothing more is written down");
    };
    let overwritten = waiting.next.replace(handed);
    drop(waiting);
    CHANGED.notify_all();
    if let Some(overwritten) = overwritten {
        detail!(
            "resume: chapter {} was still waiting to be written; the newer one is what the file wants",
            overwritten.chapter,
        );
    }
}

/// The thread that writes them, one at a time, until it is stopped.
///
/// Below the game's priority, because a chapter written a millisecond later costs nobody anything and
/// a late frame costs the thing orb was written for.
///
/// Nothing is held while the write happens: the chapter is taken out from under the lock and the lock
/// let go of before the disk is touched, so a frame handing the next one over never waits for a write.
fn writing() {
    orb_api::thread::below_normal();
    loop {
        let Ok(mut waiting) = WAITING.lock() else {
            return;
        };
        while waiting.next.is_none() && !waiting.stopping {
            match CHANGED.wait(waiting) {
                Ok(next) => waiting = next,
                Err(_) => return,
            }
        }
        // Whatever is waiting goes first, stopping or not: a chapter handed over and then dropped is a
        // run somebody would be offered the chapter before.
        if let Some(handed) = waiting.next.take() {
            drop(waiting);
            written(&handed);
            continue;
        }
        waiting.stopped = true;
        drop(waiting);
        CHANGED.notify_all();
        return;
    }
}

/// Stops the writer once it has written what it holds, and waits for it.
///
/// Called where orb is going away — see [`crate::runtime::detached`] — for two reasons. A line written
/// after the log has been closed goes into whatever log this process opens next, and a thread carrying
/// the simulated Windows of a launch that has gone would write the next launch's chapters into it. What
/// is waiting is written first, this being the one moment there is to write it.
///
/// **The way out of a real launch does not come through here**: `DllMain`'s `DLL_PROCESS_DETACH` closes
/// the log and lets the process end, and waiting for a thread from inside the loader lock is the
/// deadlock every note about that lock is about. What waits here is a launch being taken out by hand,
/// which is what an e2e test does at the end of one.
pub fn stop_writing() {
    if !WRITING.swap(false, Ordering::Relaxed) {
        return;
    }
    let Ok(mut waiting) = WAITING.lock() else {
        return log!("resume: the writer left its own state locked; it is not being waited for");
    };
    waiting.stopping = true;
    CHANGED.notify_all();
    let waited = CHANGED.wait_timeout_while(
        waiting,
        std::time::Duration::from_millis(STOP_WAIT_MS),
        |waiting| !waiting.stopped,
    );
    let Ok((mut waiting, timed_out)) = waited else {
        return log!("resume: the writer left its own state locked; it is not being waited for");
    };
    if timed_out.timed_out() {
        log!(
            "resume: the writer has not finished in {STOP_WAIT_MS}ms; what it holds goes unwritten"
        );
    }
    // Whatever came of it, the next launch in this process starts a writer of its own.
    *waiting = Waiting {
        next: None,
        stopping: false,
        stopped: false,
    };
}

/// Writes one of them where it stands, and says what it cost.
///
/// The microseconds are the point of the line as much as the chapter is: the write being slow enough
/// to stop a frame is what took it off the frame, and this is where that is read.
fn written(handed: &Handed) -> bool {
    let Handed {
        path, bytes, said, ..
    } = handed;
    let started = crate::profile::now();
    // The directory is made where it is written rather than at startup, so an installation nobody
    // has left a run in has nothing of orb's beside the game but the log.
    if let Some(parent) = path.parent()
        && let Err(error) = orb_api::fs::create_dir_all(parent)
    {
        log!("resume: cannot make {}: {error}", parent.display());
        return false;
    }
    match orb_api::fs::write(path, bytes) {
        Ok(()) => {
            let us = crate::profile::since(started);
            // Not left to `verbose` alone, where this line was: a session on the machine a slow write
            // is about was played through at the ordinary level and said nothing about the disk,
            // because nobody knew there was a level to ask for. Nor said at the ordinary level every
            // time, which is a line a chapter about a disk that is behaving.
            if us >= SLOW_WRITE_US {
                log!(
                    "resume: writing {} took longer than a frame: {us}us",
                    path.display(),
                );
            }
            detail!("resume: {said} written to {} in {us}us", path.display(),);
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
pub fn load(dir: &Path, game: &dyn Game, run: &RunStart) -> Option<Kept> {
    let path = slot(dir, game, run)?;
    let bytes = match orb_api::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            log!("resume: cannot read {}: {error}", path.display());
            return None;
        }
    };
    let saved = match game.decode_resume(&bytes) {
        Some(saved) => saved,
        // Left where it is rather than deleted: a file that cannot be read is worth looking at, and
        // orb deleting it is what would stop anyone doing so. A file this game's own reader does not
        // know is one of those: another game's, another version of this game's own, or a run left
        // before the file carried what putting it back takes.
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
    log!(
        "resume: {} holds {saved} {}",
        path.display(),
        game.say_run_state(&saved.state),
    );
    Some(saved)
}

/// What a named slot holds, as the line a mark on the front end shows, and `None` where it holds
/// nothing or nothing that can be read.
///
/// By name rather than by run, because this is asked of a run being *pointed* at rather than one
/// being started, and there is nothing to check the file's own idea of itself against yet. Silent
/// about a file it cannot read, unlike [`load`]: the front end is not the place to say so, and the
/// answer is the same either way — nothing to offer.
pub fn peek(dir: &Path, game: &dyn Game, slot: &str) -> Option<String> {
    let path = dir.join(DIRECTORY).join(slot).with_extension(EXTENSION);
    let bytes = orb_api::fs::read(&path).ok()?;
    Some(game.decode_resume(&bytes)?.describe())
}

/// What runs have been left unfinished, as the names of their files, for the log at startup: which
/// of them the question will offer depends on what is chosen, and this is what says none of them was
/// there to be offered.
pub fn left(dir: &Path) -> Vec<String> {
    let mut slots: Vec<String> = orb_api::fs::files_in(&dir.join(DIRECTORY))
        .into_iter()
        .flatten()
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
    match orb_api::fs::remove_file(&path) {
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

#[cfg(test)]
mod tests {
    use super::{MAX_STAGE_FRAMES, Played, Record};

    /// A chapter played again writes over the frames it plays again, so what a chapter's own save
    /// holds is the attempt that survived to reach it. Which is why a restore drops nothing here.
    #[test]
    fn an_attempt_played_again_writes_over_the_one_before_it() {
        let mut record = Record::new();
        record.stage = Some(3);
        for frame in 0..10 {
            record.note(frame, 0x001, frame as u16);
        }
        // Back to the chapter's own frame, and played differently from there.
        for frame in 4..10 {
            record.note(frame, 0x041, frame as u16);
        }
        let pressed: Vec<u16> = record.played.iter().map(|played| played.buttons).collect();
        assert_eq!(pressed[..4], [0x001; 4]);
        assert_eq!(pressed[4..], [0x041; 6]);
    }

    /// Nothing is kept while nothing is being played, so a session spent in the menus does not
    /// grow a record of what was pressed there.
    #[test]
    fn nothing_is_kept_between_runs() {
        let mut record = Record::new();
        record.note(0, 0x001, 0x1234);
        assert!(record.played.is_empty());
        record.stage = Some(0);
        record.note(0, 0x001, 0x1234);
        record.stop();
        assert!(record.played.is_empty());
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

    /// A frame number no stage reaches is not one to write down, whatever asked. What a *file* saying such a
    /// frame is refused for is each game's own — see `th06::resume` and `th07::resume`.
    #[test]
    fn a_frame_no_stage_reaches_is_not_written_down() {
        let mut record = Record::new();
        record.stage = Some(0);
        record.note(MAX_STAGE_FRAMES, 0x001, 0x1234);
        assert!(record.played.is_empty());
    }

    /// The buttons are handed back for the frames before the chapter and not for the chapter's
    /// own frame: from there the keyboard is the player's again.
    #[test]
    fn the_buttons_are_fed_up_to_the_chapter_and_no_further() {
        let mut record = feeding_a_run();
        assert_eq!(record.feed(0, 0xa0), Some(0));
        assert_eq!(record.feed(2, 0xa2), Some(2));
        assert_eq!(record.feed(3, 0xa3), None);
        record.feeding = None;
        assert_eq!(record.feed(0, 0xa0), None);
    }

    /// **The generator on each fed frame is held against the one the run had**, which is the only thing that
    /// says a playback has stopped reproducing that run and on which frame: the same buttons carry on being fed
    /// into a stage that is no longer the same stage, and what comes of that is a death thousands of frames
    /// later. Said once — the first frame is the one with the cause in it — and the buttons keep coming, so the
    /// landing check at the end still gets its say.
    #[test]
    fn a_playback_that_parts_from_the_run_says_which_frame_it_was() {
        let mut record = feeding_a_run();
        assert_eq!(record.feed(0, 0xa0), Some(0));
        assert_eq!(record.parted, None, "frame 0 was the run's own");

        // Frame 1 with the generator somewhere else: the frame is named and the button still handed over.
        assert_eq!(record.feed(1, 0xbb), Some(1));
        assert_eq!(record.parted, Some(1));

        // And the frame after it, also out of step, does not take the name off the first.
        assert_eq!(record.feed(2, 0xcc), Some(2));
        assert_eq!(record.parted, Some(1));
    }

    /// **A fed frame the run has no word for is left with none**, and the frames beside it are held to theirs:
    /// what such a frame came out of is a file from before orb wrote the generators down, and this playback's
    /// own generator is not the run's. Written down here it would go into every file after this one as the
    /// run's own, and the next playback of those frames would be held against a parting rather than a run.
    #[test]
    fn a_fed_frame_the_run_has_no_generator_for_is_neither_held_nor_written_down() {
        let mut record = feeding_a_run();
        record.played[1].generator = None;

        assert_eq!(record.feed(1, 0xbb), Some(1));
        assert_eq!(
            record.parted, None,
            "a frame with no word was called a parting"
        );
        assert_eq!(
            record.played[1].generator, None,
            "the playback's own generator was written down as the run's",
        );

        // And the frame after it, which does have one, is still held to it.
        assert_eq!(record.feed(2, 0xcc), Some(2));
        assert_eq!(record.parted, Some(2));
    }

    /// **A playback asked for a frame that is not the next one is said out loud**, and the same frame asked for
    /// twice is not: the game reads the keyboard more than once on a frame it spends with a menu up, where a
    /// jump is a frame of the run whose buttons nothing was ever fed.
    ///
    /// What this catches needs nothing written down about the run, which is what the generator check needs —
    /// so it catches it in a run picked up out of a file from before orb wrote the generators down.
    #[test]
    fn a_playback_asked_for_a_frame_that_is_not_the_next_one_says_so() {
        let mut record = feeding_a_run();
        assert_eq!(record.feed(0, 0xa0), Some(0));
        assert_eq!(record.feed(0, 0xa0), Some(0));
        assert_eq!(record.feed(1, 0xa1), Some(1));
        assert_eq!(record.feed(2, 0xa2), Some(2));
        assert!(!record.skipped, "frames in order were called a jump");

        let mut jumped = feeding_a_run();
        assert_eq!(jumped.feed(0, 0xa0), Some(0));
        assert_eq!(jumped.feed(2, 0xa2), Some(2));
        assert!(
            jumped.skipped,
            "frame 2 fed straight after frame 0 was not said",
        );
    }

    /// **A frame written down while a record already holds a run played back is one entry and not two**, which
    /// is what a playback that ended early made of two lists: `resume.rs` kept the buttons and the generators
    /// side by side, a playback filled the buttons out of its file and the generators stayed empty, and the
    /// first frame written down after it indexed the short one — `index out of bounds: the len is 0 but the
    /// index is 3609`, in a th07 launch that crashed where a run was picked up.
    #[test]
    fn a_frame_written_down_over_a_playback_grows_the_whole_of_it() {
        let mut record = feeding_a_run();
        record.feeding = None;
        // Inside what the playback loaded, which is where the two lengths used to part company.
        record.note(1, 0x101, 0xbeef);
        assert_eq!(record.played[1].buttons, 0x101);
        assert_eq!(record.played[1].generator, Some(0xbeef));
        // And past it, which is the run carrying on from where the playback left off.
        record.note(9, 0x041, 0xf00d);
        assert_eq!(record.played[9].buttons, 0x041);
        assert_eq!(record.played[9].generator, Some(0xf00d));
        assert_eq!(record.played.len(), 10);
    }

    /// A record in the middle of playing a run back: four frames of buttons, the generator each of them was
    /// pressed with, and the chapter three frames in.
    fn feeding_a_run() -> Record {
        let mut record = Record::new();
        record.stage = Some(3);
        record.played = [(0, 0xa0), (1, 0xa1), (2, 0xa2), (3, 0xa3)]
            .map(|(buttons, generator)| Played {
                buttons,
                generator: Some(generator),
            })
            .to_vec();
        record.feeding = Some(super::Landing {
            at: 3,
            chapter: 17,
            name: String::new(),
            retries: 0,
            song: None,
            reproduction: String::new(),
        });
        record
    }
}
