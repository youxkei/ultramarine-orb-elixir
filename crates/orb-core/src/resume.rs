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
//!
//! A snapshot is only good inside the process that took it, so closing the game would lose where the run
//! was — which for the one thing orb exists to do, grinding a late chapter, is the wrong place to lose it.
//! What survives a launch is what the run *pressed*, and the chapter is reached again by being played to.
//!
//! `pointdevice_resume/` beside the game holds one `.msgpack` per run there is a chapter of, and each holds:
//! which run it is, the run's numbers as the stage it is in began, and the buttons of every frame from that
//! stage's first to the frame the chapter began on. One is written **every time a chapter begins**, because
//! there is no moment a session ends at — the window closed, the game killed and a crash all leave nothing
//! to write from.
//!
//! **And on the frame the retry menu sends the run back to a chapter behind the one it was in**, which is
//! the same thing said the other way: what the file holds is where the run is standing, and a run sent back
//! to a chapter is standing there. Read on that frame and not the one after it, because the song position
//! and the reproduction line that go in with it belong to the chapter's own frame — which the restore has
//! just put the game on. The buttons need nothing done to them: the record is one entry per stage frame, so
//! the entries under any chapter are the ones that reached it.
//!
//! **Read on the frame and written off it.** The frame's own part is the reading, which only the frame can
//! do; encoding and writing are nobody's frame's, and `WriteFile` takes what it takes — a disk, an antivirus
//! reading every write, a folder being synced; on one machine the boundaries stopped the game for about a
//! second each. So the frame hands the chapter to a thread of orb's own, which is the answer the joystick
//! read has: below the game's priority, holding one chapter and not a queue — what the file holds is where
//! the run is *now*, so a chapter still waiting when the next arrives is one nothing wants — and taking it
//! out from under its lock before touching the disk, so that handing the next one over never waits for a
//! write. The line it writes says how long the write took, that being the number this arrangement is about —
//! and where the write took longer than a frame it says so at the level a run is ordinarily logged at,
//! since a machine whose disk does that is one nobody knew to ask `verbose` of.
//!
//! **Not held for the frame's slack**, which is what the log used to do with its own lines: the slack is
//! what is left of one frame's turn, so a write of about a second put there misses the next frame's blank
//! instead of this one's — the same stutter a frame later.
//!
//! The thread is stopped where orb is taken out of a launch, having written what it holds: a line written
//! after the log is closed would go into whatever log the process opens next. **A real launch does not come
//! that way** — `DllMain`'s detach closes the log and the process ends — because waiting for a thread from
//! inside the loader lock is the deadlock that lock's every rule is about. What a session that ends with a
//! chapter still waiting loses is that chapter, and the file still holds the one before it, which is the
//! arrangement anyway.
//!
//! **MessagePack, through the same serde the settings go through, and with the field names in it** —
//! `to_vec_named`, not the positional arrays rmp-serde writes by default. Packed because the file is the
//! machine's own: written as text a chapter deep in a stage came to 30KB against 11KB here, and nobody
//! reads a button mask by eye. Named because a file that carries its own field names needs nothing but
//! itself to be read — any MessagePack-to-YAML converter reads one beside the log, which is what has caught
//! every fault in this machinery: the seed written 2048 draws early, the song position ignored, a landing
//! that agreed on every field it had. Not a flag on the launcher: it is for whoever is working on orb, not
//! for anybody playing.
//!
//! The buttons are a map keyed by the frame they changed on, rather than a list of pairs: a map of frames is
//! in order because it is a map, not because something checked, it is a byte smaller each, and the dump
//! prints one change per line where pairs are two. Everything anyone reads by eye is a string either way —
//! the seed as `0x1a2b`, since that is how every line of the log writes it, and the reproduction line as the
//! log's own text.
//!
//! A chapter is never written down part-way through: the resume point is a chapter's own first frame, which
//! is where dying sends the player anyway, and the frame a run was abandoned on is as likely as not the
//! frame they were hit on.
//!
//! **One per run there can be one of**, the way th15 keeps a pointdevice save per difficulty and character. A
//! run of another character is another run and its buttons would play somebody else's shot, so what two
//! runs have in common when a chapter of one can be picked up in the other is the difficulty, the character
//! and the shot. That answer is one string, `lunatic-marisa-b`, which is both the name of the file and the
//! test for whether two runs are the same run: a listing of the directory then reads as the runs somebody
//! has left unfinished. A directory rather than one file holding them all, because a file is written whole
//! at every chapter and with every run in it that write would be every run's buttons a few seconds apart.
//! Its file holds the newest chapter, and starting that same run from the beginning replaces it.
//!
//! **A practice run has no such name, and that is the whole of why none is kept.** It is one stage played
//! on its own — nothing carried in, nothing after it — so what a resume would save is the walk back through
//! a menu that starts the same stage again in a moment. Having no slot is also what keeps it from taking
//! the full run's: a practice run of the same shot would name that same file unless the stage went into
//! the name too. Nothing is written for one, nothing is offered, and nothing is marked, by there being
//! nothing to call it rather than by three separate checks.
//!
//! **The record is one entry per stage frame, kept by frame number**, where the game's own replay keeps a
//! `(frame, buttons)` pair per change. That is what makes a retried chapter cost nothing: an attempt that is
//! rewound plays the same frames over and writes over them, so the entries below any frame are always the
//! ones that survived to reach it, and a restore has nothing to drop. What goes in the file is the frames
//! the buttons changed on, which is a few hundred lines for a stage.
//!
//! The buttons are written down and handed back in `Controller::GetInput` (0x41d820), which
//! `Supervisor::OnUpdate` calls once a frame and is the one place every button the game acts on passes
//! through. They are keyed to `GameManager.gameFrames`, read in that same call: the supervisor is chain
//! priority 0 and `GameManager::OnUpdate` — which is what advances that counter, at its very end — is
//! priority 4, so the number read there is the frame the update about to run *is*. That counter is also the
//! clock a chapter's start is expressed in, which is what lets a chapter name a place in the record.
//!
//! Only the buttons the game's own replay records: `g_CurFrameInput & 0x1f7`, which is every button but
//! 0x8. That one opens the pause menu, and a resume that fed it back would put up a menu instead of playing.
//! A frame with a menu up is not in the record either, and does not have to be kept out of it by hand:
//! `GameManager::OnUpdate` returns `CHAIN_CALLBACK_RESULT_BREAK` there before it reaches the counter, so the
//! frame does not advance and the same entry is written again by the frame that eventually runs.
//!
//! **What the run's numbers are, and where they go back.** The state a stage needs to be played again is
//! what the game's own replay keeps per stage, which is `StageReplayData`'s eight fields:
//!
//! | | |
//! | --- | --- |
//! | score | `guiScore` at `GameManager+0x0` and `score` at `+0x4`, both, as the replay writes both |
//! | `randomSeed` | `g_Rng.seed` at 0x69d8f8, with the generation count cleared |
//! | `pointItemsCollected` | `+0x1816` |
//! | power | `+0x1810` |
//! | `livesRemaining`, `bombsRemaining` | `+0x181a`, `+0x181b` |
//! | rank | `+0x1a70` |
//! | `powerItemCountForScore` | `+0x1819` |
//!
//! and beside them the difficulty at `+0x10` and the character and shot at `+0x181d` and `+0x181e`, which
//! the replay keeps in its own header. orb writes two more. `extraLives` (`+0x181c`), because the replay's
//! path does not: `GameManager::AddedCallback` raises that count with a loop against the score thresholds
//! instead, and a count that can only rise means a resume writing the score without it hands out every
//! extra life the run has already had — a life the run never got, and with it the `IncreaseSubrank(200)`
//! that came with the last one. And the count of deaths (`+0x20`), which nothing but the result screen
//! reads, so that a resumed run's is the run's. The two counters beside that one are left as a fresh stage
//! leaves them, the game's own replay not carrying them either.
//!
//! **The seed is the whole of why the buttons alone are not enough.** `GameManager::AddedCallback` copies the
//! generator's seed into `randomSeed` as each stage begins and clears the generation count — it does not
//! reseed — so a stage played again from a different seed is a different stage however the buttons fall.
//!
//! **And it goes in at one instant, which is neither end of that callback.** It is written on the way into
//! `Stage::RegisterChain` (0x4044c0) — called from one place in the whole exe, 0x41c00d, between the numbers
//! being put in place and the stage being built out of them. Both ends of the callback are wrong:
//!
//! - **After it is two draws late.** Building a stage draws from the generator — two numbers on stage 1 —
//!   and `Stage::RegisterChain` is what draws them. The landing check said so in the one field it disagreed
//!   on: `randoms=2 against randoms=0`.
//! - **On the way in is 2048 draws early.** Before the callback copies the seed anywhere it fills a table of
//!   keys out of the generator: 0x41bc4f, 64 records of 32 `u16` at `manager+0x30`, one `Rng::GetRandomU16`
//!   (0x41e780) each, and every draw rewrites `g_Rng.seed`. The whole block is skipped when `curState` is 3,
//!   the state between two stages of a run — so a stage reached by playing draws none of them and a stage
//!   reached from the menu, which is every resume, draws all 2048: a stage resumed with `seed=0x789c` where
//!   `0xc381` was written down, and 2048 draws of that generator from `0xc381` are `0x789c` exactly.
//!
//! Written where the game's own replay effectively writes it, in other words: `AddedCallbackDemo` is called
//! from 0x41bf7e, after the 2048 and before `g_Rng.generationCount = 0` at 0x41bfec. The `GameManager` copy
//! of the seed goes in beside the generator, since the callback has already taken it by then and that copy
//! is what the *next* chapter of the resumed stage writes down.
//!
//! **The seed is in the landing line too**, and not only in the header, because it is the one number a
//! stage can start wrong in while every other field of that line is right: a resumed stage 2 agreed with
//! what was written down field for field with its seed 2048 draws out — the player's place being the
//! player's own inputs, and the bullets being the stage's script, which is deterministic. A landing that
//! agrees now agrees about the generator as well.
//!
//! **Which score is the run's depends on the frame it is read on.** The last thing that callback does is
//! `score = 0`, and `GameManager::OnUpdate` raises it back on the stage's first update — `if (score <
//! guiScore) score = guiScore`, which is what carries a run's total across a stage at all. So on the frame
//! orb reads, `score` is nothing and `guiScore` is the total, and it is `guiScore` that is read. The game's
//! own recording reads `score` there and is right to: it is registered from inside that callback, before the
//! zeroing. Reading the field the game reads, on a different frame from the game, is where this would have
//! gone wrong silently — every stage written down with a score of nothing.
//!
//! **They go back where the game puts them back**, which is that same callback: 0x41bb02, registered by
//! `GameManager::RegisterChain` as the added callback of the chain element at 0x69d720 and so run by
//! `Chain::AddToCalcChain` at the moment a stage is registered, before that stage's first update. It is
//! where `ReplayManager::AddedCallbackDemo` writes a replay's stage record from — called from inside it — and
//! it has to be after rather than before, because that callback is also what sets rank from a table indexed
//! by difficulty and what overwrites the power in practice mode. orb hooks it and writes the run's numbers
//! as it returns; on any other stage of any other run, that is where they are *read*, which is the only
//! moment they are the numbers the stage started with.
//!
//! **Starting the run is one write, because the game's own front end has done the rest.** The question is
//! put on the frame after the character select has taken its item, which is the one frame where
//! `g_Supervisor.curState` is already `GAMEMANAGER` while `wantedState` is still the front end:
//! `Supervisor::OnUpdate` is chain priority 0 and copies `wantedState = curState` as its last act, and the
//! front end is priority 2, so an item that starts a run writes the state after that copy and the supervisor
//! has not yet acted on it. By then the difficulty, the character and the shot are the ones just chosen, the
//! front end has removed its own job — its update returns `CHAIN_CALLBACK_RESULT_CONTINUE_AND_REMOVE_JOB`,
//! whose deleted callback releases the title's graphics and cuts its drawing — and nothing of the run
//! exists. So orb writes `currentStage`, as the menu counts it: `AddedCallback` raises the number by one, so
//! it is handed the stage before the one meant, which is what the character select writes too.
//!
//! The number is checked against what the game has, since it comes out of a text file and one that is
//! nonsense is `AddedCallback` indexing its stage table past the end — `[eax*8+0x4764ec]` is where it picks
//! a stage's data out of. Nothing else has to be checked or written: what the run *is* was chosen on the
//! game's own screens, and the file is only read at all when its name says it is that same run. The demo
//! and a replay reach those two states the same way, so both are refused: the demo starts from the title
//! screen itself.
//!
//! **The playback runs inside the frame that built the stage**, updating with nothing drawn until the frame
//! the chapter began on — the ending skip's mechanism aimed at a chapter. An update is tens of microseconds
//! and a stage is thousands of them. Every update is observed, so the run lands with the stage divided into
//! the same chapters and with the chapter's own snapshot to be sent back to, and the retry count carries on
//! from what the run had already spent.
//!
//! **And the playback is given up where the next stage is registered, rather than left as a bound to compare
//! against.** The frame it feeds to is a frame of a *stage*, and the stage the run walks into starts
//! counting again from nothing: left standing, that bound is one every frame of the new stage is below, and
//! the buttons of those frames are written down as they are played. A frame the game reads twice is then
//! one the playback still holds an answer for, so the music is held quiet on it and a chapter retried there
//! hands the run the stretch somebody has already played through — fed back a frame at a time with nothing
//! pressing anything, which is a replay of something else being watched.
//!
//! **A chapter the stage's own song plays through lands with that song where the chapter had it**, which is
//! what a retry of one does to the music — and a resume is the same chapter arrived at another way, so it
//! owes the same. Otherwise what plays is the track's opening milliseconds, the stage having been built a
//! frame ago.
//!
//! Which chapters those are is decided by the song and not by the chapter's kind: a midboss is a fight with
//! the stage's song still playing, so its chapters want the same thing done to them as the midstage's, and
//! it is the same question the retry answers from the same place. Asked of the kind instead, a resume into
//! `MIDBOSS NONSPELL 1` left the track at its opening milliseconds with the position that had been written
//! down ignored.
//!
//! What is written down for it is one number: the file offset of the sound that was audible when the
//! chapter began, which is where the streaming thread's next read was less everything sitting in the buffer
//! unplayed. The buffer's own contents are what a snapshot keeps for a rewind and are no use across a launch
//! — the buffer is an object of that process — where the wave file is the same file next time. Putting it
//! back seeks the file there, fills the buffer from it, and starts the buffer at nothing, which leaves the
//! stream exactly as a freshly loaded track leaves it: one buffer's worth from that offset, the play cursor
//! and the next write at nothing, the file at what follows. Filled rather than left to the streaming thread
//! so that nothing of the opening it already held is heard first.
//!
//! **And the countdown to the loop with it, which is the second half of that seek.** The game's wave file is
//! the DirectX SDK's `CWaveFile` with loop points added, and it keeps `m_ck.cksize` — the size `mmioDescend`
//! left, or the loop end where the track has one — as how much sound is left: every read is clamped to it
//! and subtracted from it, and the track is looped when a read comes up *short against it*. The file's own
//! end is never consulted. So the position and that countdown are a pair, and a seek that moves one alone
//! is a track that loops in the wrong place: seconds of the song repeating, once, near the end of a resumed
//! stage, because the stream believed it had as much left as it did before the seek, read past the end of
//! the sound, and got a failed read rather than a short one — no loop taken, and the buffer left going round
//! its own contents until the skipped bytes had been counted off. It is put back as the loop point less
//! where the file now is, and the loop point is taken from the pair as it stands before the seek rather than
//! from the header's fields, since which of them the track is going by is the game's business. The buffer
//! is stopped for both reads: the streaming thread moves both halves, on notifications a stopped buffer
//! does not raise.
//!
//! The chapter's own snapshot is then taken again, because the one from the playback holds the sound as the
//! playback had it: left alone, the first death in that chapter would rewind the music to the top and undo
//! this. It costs one copy of a chapter on a frame that has just run a stage's worth of updates. A chapter
//! of a boss's own theme is left alone in both respects, the way that fight's retry leaves it: starting a
//! long theme over every attempt is worse than the jump in it.
//!
//! `song=-1` in the file is a stream that would not say — `mmioSeek` refusing, or a song that had looped
//! within a buffer's length of the reading — and a resume then leaves the music as the stage started it. So
//! is a file written before the field existed.
//!
//! **The file goes when the run finishes**, which is the result screen: a run given up does not go through
//! one, so what is left holding a chapter is exactly a run that was left. A cleared run's file is taken away
//! — that run's own, which is the one it was being written to.
//!
//! **What settles whether this works is the landing.** The reproduction line — the replay clock, the
//! buttons, where the player is, how many numbers the generator has given out, the score, the rank — is
//! written into the file for the chapter's frame and read again at the frame the playback stops on, and the
//! log says whether the two agree field for field and which field they first differ on. That is the
//! instrument for the one thing the mechanism rests on: that the numbers a stage reads at its start are all
//! written down. A player hit during the playback is reported too, and at which frame: a path that survived
//! does not do that, every death a run had having been rewound away, so a hit is a playback that has come
//! out of step at or before that frame.
//!
//! `--no-resume` takes the whole of it out — the two hooks and the file — while leaving the chapters it is
//! built on, which is what a fault in a run that chapters alone do not explain wants.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::game::{Game, RunStart};
use crate::sync::MainThread;
use crate::{detail, log, writer};

/// What every line about a chapter written down starts with, the thread that writes one included: a log
/// read for this machinery holds all of it, and the writer's own lines say which of the files it writes
/// they are about — see [`writer::Handed::about`].
const ABOUT: &str = "resume";

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
    /// Where the song was when that chapter began, as a position in the file the game streams from — the one
    /// thing about the sound that means anything in another process, the buffer and its object being
    /// this one's. `None` where the stream would not say, and then a resume leaves the music as the
    /// stage started it.
    pub song: Option<i32>,
    /// **And which track that position is in**, since the file holds every one of them: the position means the
    /// sound it points at only while the track playing is the one it was read from, and a stage's first chapter
    /// is picked up into the front end's own song — the build has loaded the stage's slots and its script has
    /// asked for neither. `None` where the stream would not say, and for a file written before this was kept;
    /// either way a resume leaves the music alone rather than seeking a stream it cannot vouch for.
    ///
    /// [`Game::music_identity`] and not a name, which is what the restore holds a chapter's sound to as well —
    /// and it is a number the track's own record makes, so the same track answers with it in any process.
    pub song_identity: Option<u32>,
    /// **And whether that sound was coming out**, which the position does not say: a stream sitting at an
    /// offset with its buffer stopped is silence, and a chapter taken in that is a chapter whose sound was
    /// nothing.
    ///
    /// Which is what a stage's own first chapter is taken in: it falls frames into the stage, where the song
    /// of the scene before it has been taken down and the stage's own has not been asked for — so a resume
    /// that put the position back *and played it* made a silence audible. A launch picked stage 6's first
    /// chapter up and heard the title screen's own theme for the five seconds before that stage's script
    /// started its song.
    ///
    /// The restore holds a chapter to the same thing — see [`crate::audio::Saved::was_playing`] — and this is
    /// that reading kept in a file, so that a resume owes what a retry owes.
    ///
    /// **`None` is a file that does not say**, which is every th06 file written before this was kept — that
    /// game's own file is read both with the field and without it, see `game::th06::resume`. Not the same as
    /// `Some(false)`: one is a chapter whose sound was nothing and the other is a chapter nothing is known
    /// about, and a line saying the first of a file holding the second would be orb saying what it was never
    /// told. Either leaves the music alone.
    pub song_sounding: Option<bool>,
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
    /// And which track that position is in — see [`Kept::song_identity`].
    pub song_identity: Option<u32>,
    /// And whether it was coming out, `None` for a file that does not say — see [`Kept::song_sounding`].
    pub song_sounding: Option<bool>,
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
    /// The frame the buttons stop being fed on, which is not always the frame the playback is *going* to.
    ///
    /// **The two are the same for a run being picked up and not for a replay being watched.** Picking one up
    /// feeds every frame below the chapter inside the frame that built the stage and hands the keyboard back
    /// there — see [`landing`]. Watching an attempt feeds those same frames and then goes on, one frame of
    /// the run to one frame of the game, for as long as the attempt lasted: so the landing is where the
    /// *fast* part stops and this is where the feeding does. See [`watch`].
    until: Option<u32>,
    /// A saved run whose stage the game has been asked to build, waiting for it to be built.
    starting: Option<Kept>,
    /// How many frames past the chapter the playback of `starting` goes on feeding for, and `None` for a run
    /// being picked up — which is the ordinary case: the chapter is where the keyboard becomes the player's.
    watch: Option<u32>,
    /// Whether the stage a watched attempt is in has been written into the run the front end started — see
    /// [`the_stage_of_a_watch`], which is asked on every frame the run is registered on and there is more
    /// than one of those on th07.
    watching_that_stage: bool,
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
            until: None,
            starting: None,
            watch: None,
            watching_that_stage: false,
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
        let until = self.until?;
        if frame >= until {
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

    /// Gives up the playback and keeps everything the run is written down as: the bound it fed to and the
    /// three things held about how it went, none of which means anything to the playback after it.
    ///
    /// **Not [`Record::stop`]**, which is for a run that has ended and throws its buttons away with the rest.
    /// A playback is given up under a run that goes on being played and written down.
    fn the_playback_is_over(&mut self) {
        self.until = None;
        self.fed = None;
        self.skipped = false;
        self.parted = None;
    }

    fn stop(&mut self) {
        self.stage = None;
        self.played.clear();
        self.parted = None;
        self.fed = None;
        self.skipped = false;
        self.feeding = None;
        self.until = None;
        self.starting = None;
        self.watch = None;
        WATCHING.store(false, Ordering::Relaxed);
    }
}

/// Whether the run being played is one whose inputs are worth keeping: pointdevice, and orb
/// allowed to keep them at all.
pub fn keep(on: bool) {
    KEEPING.store(on, Ordering::Relaxed);
}

/// Whether what is being put back is an attempt somebody is watching rather than a run they are picking up.
///
/// **A flag of its own beside [`KEEPING`], because the two say different things and a watched attempt needs
/// one of each answered the other way.** Putting a run back is what both do — the same numbers into the same
/// stage, the same buttons fed in — so a stage begun while this is set is one to write over. Writing the run
/// down is not: what a watched attempt would write is a replay's own frames into the file of the run
/// somebody left, which is that run overwritten by a recording of itself. So this puts the numbers back and
/// keeps nothing, and `KEEPING` keeps and puts back.
static WATCHING: AtomicBool = AtomicBool::new(false);

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
    let a_run_of_somebodys_own = state.in_run
        && !state.demo
        && !state.replay
        && game.run_slot(&unsafe { game.run_start() }).is_some();
    let keeping = KEEPING.load(Ordering::Relaxed) && a_run_of_somebodys_own;
    // And a run to *put back*, which is the same run with one more way of being one: an attempt somebody is
    // watching is put back and written down nowhere — see [`WATCHING`].
    let putting_back = keeping || (WATCHING.load(Ordering::Relaxed) && a_run_of_somebodys_own);
    if let Some(saved) = record.starting.take() {
        // Two ways this is not the stage it asked for, said apart: what came up is a run orb has no
        // business writing down, or it is the wrong stage of one.
        if !putting_back {
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
        // **Nothing written down where this is an attempt being watched**, which is what an unnamed stage
        // makes of [`Record::note`]: the frames fed are not noted anyway, and the frames after them are the
        // replay's own rather than a run's — see [`WATCHING`].
        record.stage = keeping.then_some(state.stage);
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
        // Where the feeding stops, which is the chapter for a run being picked up and the end of the
        // attempt for one being watched — see [`Record::until`]. The playback of a watched attempt has
        // buttons for the frames past the chapter, and [`Record::played`] below is where they are.
        record.until = Some(saved.at + record.watch.take().unwrap_or(0));
        record.feeding = Some(Landing {
            at: saved.at,
            chapter: saved.chapter,
            name: saved.name,
            retries: saved.retries,
            song: saved.song,
            song_identity: saved.song_identity,
            song_sounding: saved.song_sounding,
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
    record.feeding = None;
    // **And the playback of the stage before this one given up.** Its bound is a frame of *that* stage and
    // this one starts counting again from nothing, so a bound left standing is one every frame of this stage
    // is below — and the buttons of those frames are written down as they are played, which makes a frame the
    // game reads twice one the playback has an answer for by its second read. A th07 session picked a run up
    // in stage 5 and walked into stage 6, whose frame 1 the playback was asked for straight after frame 17863;
    // the music was then held quiet on every frame it answered for, and the chapter retried there handed the
    // run the stretch somebody had already played through, which looked like a replay of something else.
    //
    // **The frame the playback runs out on is not where this belongs**, though it is where the playback
    // *ends*: nothing inside the stage it fed reaches a frame below the bound, the landing chapter's snapshot
    // being taken again at the landing frame and being the only one there is — so the retry menu of a run just
    // picked up offers that chapter alone, with no item behind it.
    record.the_playback_is_over();
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

/// Puts an attempt out of a replay in the way of the run the game is about to build, to be watched rather
/// than picked up: the frames below the chapter played in as fast as a run being picked up, and then the
/// attempt's own frames one to a frame of the game.
///
/// `frames` is how many of them the attempt lasted. Everything else is the machinery a run being picked up
/// goes through, and that is the point of it being this function's neighbour: the stage is seeded from the
/// numbers written down, the buttons go in through the same read, and the landing is held against the same
/// reproduction line — so an attempt that plays back somewhere else says so in the same words a resume that
/// lands somewhere else does.
///
/// **Nothing is written down while it runs**, which is [`WATCHING`]'s whole job: the run this looks like to
/// the game is not one somebody is playing, and the files of the run it came out of are not to be touched.
///
/// # Safety
/// Must run on the game's main thread, between frames, with the game about to build the run whose numbers
/// this attempt belongs to — see [`Game::start_a_run`].
pub unsafe fn watch(game: &dyn Game, saved: Kept, frames: u32) -> bool {
    if !unsafe { game.start_stage(saved.run.stage) } {
        log!(
            "replay: the game did not take stage {}",
            saved.run.stage + 1
        );
        return false;
    }
    let record = unsafe { RECORD.get() };
    record.stop();
    log!("replay: {saved}; {frames} frame(s) of it to watch from there");
    record.starting = Some(saved);
    record.watch = Some(frames);
    record.watching_that_stage = false;
    WATCHING.store(true, Ordering::Relaxed);
    true
}

/// **The stage a watched attempt is in, written on the frame the run it is watched in is registered.**
///
/// The item that starts a run writes the stage its own menu asked for — nothing, a full run starting at the
/// first — so an attempt made in any other stage has to have its own put in after that. This frame is where:
/// it is the one frame the game will take a stage for a run it has not built yet, and a run being *picked up*
/// has the same thing done for it by [`begin`], which is called on it too.
///
/// Answers whether it wrote, which is only where an attempt is what is starting.
///
/// # Safety
/// Must run on the game's main thread, on the frame the run was registered.
pub unsafe fn the_stage_of_a_watch(game: &dyn Game) -> bool {
    if !WATCHING.load(Ordering::Relaxed) {
        return false;
    }
    let record = unsafe { RECORD.get() };
    // Once, however many frames the run is registered on: th07's own registration stands for several, and a
    // stage written on each of them is that many lines saying the same thing.
    if record.watching_that_stage {
        return false;
    }
    let Some(stage) = record.starting.as_ref().map(|saved| saved.run.stage) else {
        return false;
    };
    if !unsafe { game.start_stage(stage) } {
        return false;
    }
    record.watching_that_stage = true;
    log!(
        "resume: stage {} is put in for the attempt being watched",
        stage + 1
    );
    true
}

/// The frame the buttons stop being fed on while an attempt is being watched, and `None` where nothing is —
/// which is what says a watched attempt is over and the run it was in is orb's to end.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn watching() -> Option<u32> {
    WATCHING
        .load(Ordering::Relaxed)
        .then(|| unsafe { RECORD.get() }.until)
        .flatten()
}

/// Writes down where the run is, which is what the next launch offers to pick up. Answers with the chapter
/// it wrote, and `None` where it wrote nothing.
///
/// **The chapter itself and not whether it wrote**, because the run's replay wants the same chapter and the
/// numbers in it have to be the ones that went in the file: the song's position and the reproduction line
/// are that frame's, and read a second time they are read off a game that has moved. See
/// [`crate::replay::chapter`], which is handed what this answers.
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
) -> Option<Kept> {
    let record = unsafe { RECORD.get() };
    let run = unsafe { game.run_start() };
    // Nothing said where there is no slot: a practice run is the ordinary case for that, not a fault.
    let path = slot(dir, game, &run)?;
    // The numbers held are the ones read as *that* stage began. A record of another stage's would be
    // a run put back with the wrong lives and the wrong seed, and nothing about the file would say
    // so, so it is not written at all.
    if record.stage != Some(run.stage) {
        return None;
    }
    // Every frame before the chapter, and there is one for each of them: a chapter is reached by
    // playing the frames under it, so a record that is short is a record that missed some.
    if record.played.len() < at as usize {
        detail!(
            "resume: not written — {} frame(s) of buttons for a chapter at {at}",
            record.played.len(),
        );
        return None;
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
        // And the track it is a position in, read on the same frame: the two are one reading, and a position
        // held without it is one nothing can be seeked by.
        song_identity: game.music_identity(),
        // And whether any of it was audible, which is the third part of that one reading: a position and a
        // track say where the sound was and not that there was one.
        song_sounding: Some(
            game.music()
                .is_some_and(|music| unsafe { music.sounding() }),
        ),
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
    writer::hand_over(writer::Handed {
        about: ABOUT,
        path,
        bytes: game.encode_resume(&saved),
        said: format!("{saved} {}", game.say_run_state(&saved.state)),
        overtaken: format!("chapter {}", saved.chapter),
    });
    Some(saved)
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
        // And nothing at all once the playback is over, which is what the end of the feeding says: the
        // landing being taken is what hands the run to whoever is playing it, and that is a frame later —
        // see [`Record::until`].
        record.until = None;
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
            song_identity: None,
            song_sounding: None,
            reproduction: String::new(),
        });
        // Where the feeding stops, which for a run being picked up is the chapter it is going to.
        record.until = Some(3);
        record
    }

    /// **An attempt being watched is fed past the chapter it starts at**, which is the one thing a playback
    /// of one does that a run being picked up does not: the frames below the chapter go in as fast as they
    /// do for a resume, and the attempt's own go in after it.
    #[test]
    fn an_attempt_being_watched_is_fed_past_the_chapter_and_stops_where_it_ended() {
        let mut record = feeding_a_run();
        record.until = Some(4);
        assert_eq!(record.feed(2, 0xa2), Some(2));
        // The frame the chapter began on, which a resume hands back to the keyboard and this does not.
        assert_eq!(record.feed(3, 0xa3), Some(3));
        assert_eq!(record.feed(4, 0xa4), None, "the attempt ran past its end");
    }

    /// And the landing is still the chapter, whichever of the two this is: what the playback is *aimed* at
    /// is where the fast part stops, and for an attempt being watched the rest is played a frame at a time.
    #[test]
    fn the_landing_is_the_chapter_and_not_where_the_feeding_stops() {
        let mut record = feeding_a_run();
        record.until = Some(4);
        assert_eq!(
            record.feeding.as_ref().map(|landing| landing.at),
            Some(3),
            "the frame the playback is aimed at moved with the feeding's own end",
        );
    }
}
