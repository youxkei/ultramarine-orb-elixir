//! Which file a pointdevice run's scores go in.
//!
//! A run that has been rewound is not a run anyone played, so its score does not belong in the
//! game's ranking. Refusing the write would lose the record altogether, so the file is forked
//! instead: while orb is in pointdevice mode an open of `score.dat` becomes an open of
//! `pointdevice_score.dat`. Pointdevice runs are then ranked against each other, in the game's own
//! format and on its own screen, and the game's file comes out of such a run unchanged because it
//! is never opened.
//!
//! **One read is not the mode's, and it is the one that is not a record.** The game keeps four
//! things in the file, and the ranking and the spell cards a run has captured are the mode's own —
//! a capture in a chapter that can be played again is not the capture the game's record is a record
//! of. What the front end *offers*, though, is not a record: a stage that has been reached has been
//! reached, and answering that out of a new file locks the game back to stage 1 for want of
//! anything in it. So that read alone is left pointed at the game's own file — [`reading_unlocks`],
//! set from the one callback that fills the globals the front end lights its items from.
//!
//! **Half of what that read brings back is a record after all**, and it is the score beside each
//! practice stage: the callback fills what the menu offers and those scores out of the one file, and a
//! score practised in a chapter that can be played again is no more the game's record than a capture
//! is. One read cannot land in two files, so orb makes a fourth: `Game::read_what_the_modes_file_records` reads the
//! mode's own file wherever the mode is settled and puts its practice scores over the ones the front
//! end's read left — the unlocks beside them untouched.
//!
//! **Which file is open is a runtime switch, not a setting.** The mode is chosen inside the
//! game — at the item that starts a run, and at the one that shows the ranking — so the fork
//! follows it: a normal run and the ranking of normal runs are the game's own file, which is
//! where a run anybody could have played belongs.
//!
//! **Forking is as far as this goes, and orb's own format is the thing not to reach for.** Two files keep
//! runs that were rewound away from runs somebody played, and that is all they do: they do not make the
//! rewound ones comparable with each other, since a miss there costs a rewind rather than a life and the
//! number rewards grinding one chapter until it goes perfectly. What would make the file worth reading is
//! the retries beside the score — `RETRY` is counted already — and that means orb's own format, and with
//! it the game's own ranking screen no longer being where these are read. Nothing here is broken, so that
//! is a whole mechanism bought for a number nobody has asked for.
//!
//! **A window closed mid-run writes nothing**, and there is nowhere to put the fix: there is no front end
//! left to take the run through and the loop is on its way out. th06 loses that record too.
//!
//! **What is left in `orb::score` is the write over the exe's import of `CreateFileA`.** The replacement
//! that entry is pointed at is [`create_file_a`] and it is here, being a hook body like the eleven in
//! [`crate::runtime`]: nothing in it is per-game and nothing in it is Windows.
//!
//! A run with chapter retries is not a run anyone played, so its score does not belong in the game's
//! ranking — the same reason no replay is offered for one. Refusing the write would lose the record
//! altogether, so the file is forked: while orb is in pointdevice mode an open of `score.dat` becomes an
//! open of `pointdevice_score.dat`, those runs are ranked against each other in the game's own format and
//! on its own screen, and `score.dat` comes out of such a run unchanged because it is never opened.
//!
//! **Which file is open is a runtime switch, not a setting.** The mode is chosen inside the game — see
//! [`crate::mode`] — and the fork follows it, because a normal run and the ranking of normal runs are the
//! game's own file: that is where a run anybody could have played belongs. A launch starts in pointdevice,
//! which is what orb is for, and in normal with `--no-chapters`, where there is nothing to fork because
//! nothing can rewind. **And in normal in a game orb cannot rewind anything in**, which is `Game::rewinds`
//! and is asked at the attach rather than worked out from the run features that game declines: the fork has
//! to be settled before the front end's first read of the file, which is before there is a run for anything
//! to be declined about.
//!
//! **A rewind does not take back what the game counted about a spell card.** The record lives in the memory
//! a snapshot covers, so restoring a chapter would undo the attempt the game counted when the card started —
//! a chapter retried ten times would show one attempt — and undo a capture, which in a chapter that can be
//! played again is that chapter cleared. So the block is taken before the restore and put back after it, in
//! both of the paths that restore one. Both counters, not the attempt alone: capturing the card is what
//! clearing the chapter *is*, so a rewind that took one back and left the other would say a chapter had been
//! cleared none of the tries at it cleared.
//!
//! **A chapter is an attempt at a spell card only where one is up.** The card orb's own count goes against
//! comes out of `ds:0x5a5f98`, which holds the last card a boss was on and which **nothing clears** — not
//! the card ending, not the stage, not the run. So a chapter with no card in it reads whichever card came
//! before, and the nonspell that follows a spell reads the spell it follows: counted there, it would be an
//! attempt at a card nobody was fighting. Whether there is one to count is asked of
//! `g_EnemyManager.spellcardIsActive` instead, and asked *after* the snapshot is back rather than before — a
//! spell chapter's snapshot was taken at the card's own start, so what the restore puts the game back into
//! is that card. Where no card is up the log says so: `score: no spell card is up; no attempt counted`, said
//! rather than left as a missing line, since a line only written when the count was made would put the
//! meaning in the absence.
//!
//! **And only against a card the game has named.** th06 fills all 64 records from `Rng::GetRandomU16` before
//! it reads the file — 0x41bc87, the whole 0x40 of each of them — and writes back only the magic, the two
//! lengths, the version, the card's number and the two counts, 0x41bca0 to 0x41bcd5. So the name a record
//! carries is the generator's until the card itself starts and copies one in (0x409720), and `catk`'s parse
//! leaves every record the file has no entry for exactly as the fill left it. The ranking screen draws that
//! name for any card whose attempts are not zero and 「？？？？？」 for the ones whose are (0x42e265 and
//! 0x42e26e), which makes a count added to a record nobody named the generator's own bytes on the screen.
//! Refused by the game's own test for the same thing: the byte a record holds beside its name against the
//! sum of that name, which is the comparison at 0x4097e8. **A row already written wrong heals itself**, and
//! only by the card starting for real: that comparison then disagrees, both counts go, and the name is
//! written. A retry cannot do it — a chapter's snapshot is taken after the name was copied in, so putting it
//! back never starts the card again.
//!
//! **A run picked up keeps the names its playback learned.** The playback starts every card the run had
//! passed, which is why the counts are put back as they were before the buttons went in — a run picked up
//! would otherwise arrive having counted every card it passed. The names those starts wrote are not put back
//! with them: a name is what the playback *learned* about a card and not something it counted, and it is the
//! one thing about the card only the game knows. Putting it back leaves the record carrying the fill's own
//! bytes, which is a card orb then refuses to count against and a row the ranking never draws a name for
//! again — the landing being *inside* the card, nothing starts it a second time. So the block goes back with
//! each record's name, and the sum beside it, left as the playback left them. Which makes picking a run up
//! the second thing that heals a row already written wrong.
//!
//! **Where a session stops without the game writing, orb has the ranking built and taken down.** The write
//! is reached from one place, so a run given up or quit with `ESC` leaves what it counted about spell cards
//! in memory and nowhere else — th06 loses it, and orb does not. On the first frame the front end is up
//! after a run that ended anywhere else, orb writes `MainMenu.gameState` with what the `Score` item writes —
//! 0xa, from the handler at 0x437f56 — runs the updates that brings inside that one frame with nothing
//! drawn, and writes `curState` back to the front end the way the ranking leaves itself. The deleted
//! callback then writes the file. Bounded at 240 updates, against the 60 the front end waits and the one
//! each scene change costs; at 30µs an update the whole of it is under a frame's worth of time and none of
//! it is seen, the draw chain running once a frame after this.
//!
//! **th07 is written through its own save instead, and its ranking screen is never built.** The one write in
//! that exe is gated on a phase its ranking screen never reaches — the two kinds the screen is built with
//! skip it — so driving that screen wrote nothing however many updates it was given, and a launch faulted
//! inside those updates reading a vm the screen it had just asked to leave no longer had. What writes is
//! `0x44a302(2)`, the save that game makes on its own way out: it reads the file, lays every record the file
//! holds over the ones in memory, and writes memory back out. So what a run counted is written down on the
//! frame that run ends, and **put back** at the one moment inside the save that lies between its two halves:
//! the parse (0x445069) that lays the file's records down, which is hooked for exactly that. Put back rather
//! than added, because a run's records are the file's own with that run's counts already on them — the
//! build empties all 141 of them (0x42e657) and reads the file straight back in (0x42e735), and the front
//! end's own read does the same (0x45bff3). Added instead, a run's history would be counted twice. And only
//! inside the save orb asked for: the game saves at every run's build and on its way out, and a put-back at
//! a read that no write follows is records stranded in memory with the note spent.
//!
//! **None is built for a run that finished, or for one still inside its ending.** The screen a run finishes
//! at writes for itself, that deleted callback being the same one — so a run that reached
//! `STATE_RESULTSCREEN_FROMGAME` has nothing left waiting, and orb drops the request there. There is no
//! front end to act on 0xa on the result screen or over a staff roll, so all 240 updates would go on
//! whichever of those was running and the request then be undone — and undoing it writes
//! `RESULT_SCREEN_STATE_EXITING` into the screen that *was* up. One `ResultScreen` reached two ways is the
//! same field in the same object either way, so what that write sent away was the high-score name entry a
//! finished run had just arrived on: two frames from the run ending to the title menu, with the name entry
//! and the stats screen inside them and nobody having seen either. Over an ending it took 240 frames off the
//! staff roll instead, and wrote into a screen already freed. So the write is made only where a ranking orb
//! asked for is up, and the request is dropped where the run finished.
//!
//! **The record is put back between the two.** The added callback fills the captures out of the file it
//! read, which is what they were before the session counted anything, so orb puts back what it took before
//! asking the screen to leave. The ranking is the file's own and has to be: the write takes it out of the
//! table that read just filled — which is also why the record goes out through the game's own screen rather
//! than orb calling the write itself.
//!
//! Whether a run wrote on its own is asked rather than assumed: every open for writing is noted, and a run
//! that ended at the result screen has one. Both modes build one. A legacy session stopping partway keeping
//! its record is the one place orb is deliberately not the game as it was.
//!
//! **The captures in memory are emptied before a ranking is read.** th06 keeps the record of which spell
//! cards have been captured in one place, `g_GameManager.cardHistory` at 0x69bcd0 — 64 records of 0x40 — and
//! both reads that load it copy the records the file holds while leaving the rest as they were: 0x42b466
//! has no clear of its own, unlike `clrd`'s parse at 0x42b502, which memsets four records before it looks.
//! That is how a run's captures survive the reload the game does at every stage, and it is what the file is
//! written from: the write at 0x42b9ed copies `catk` out of that global, not out of what the screen read.
//!
//! With one file that is consistent. With two it is how one file's captures reach the other: a session that
//! looks at one ranking has that file's records in memory, and leaving the other one's screen writes them
//! into the other file. So orb empties the block before the ranking screen's read — at 0x42f060, before the
//! callback — and the read then defines the history rather than adding to it. Not in the two states that
//! screen is in on the way out of a run, 9 and 0x11 at `screen+0x8`, which are the ones the game itself
//! skips its parses in: there the record in memory is that run's own and the file is about to be written
//! from it. And not with `--no-chapters`, where there is one file and carrying the captures in memory is
//! right.
//!
//! **Every open of the file goes in the log, with which of the two it landed in and what it was for**:
//! `opened in place of the game's own` or `opened as the game's own`, and `write`, `read`, or `read for the
//! front end's unlocks`. Both sides rather than the redirected ones alone, because a line that is only
//! written when the file was swapped makes the absence of a line carry the meaning — and an absence is
//! equally what a read that never happened looks like. The reads cannot be told apart past the one that is
//! bracketed, so the line says which of them it is and no more. The one orb makes for itself says so on the
//! line after — `the practice scores read out of the mode's own file` — that being a read orb knows it is
//! making rather than one it is guessing at from a callback.
//!
//! **One read is not the mode's, and it is the one that is not a record.** th06 keeps four things in the one
//! file: `hscr`, the ranking; `catk`, which spell cards have been captured; `clrd`, how far each shot has got
//! on each difficulty, which is what the Extra and the practice stages are behind; `pscr`, what each of those
//! stages has been practised for. The ranking and the captures are the mode's own — a card captured in a
//! chapter that can be played again is not the capture the game's record is a record of, for the same
//! reason the score is not that score. What the front end *offers* is not a record of anything, though: a
//! stage that has been reached has been reached, and answering that question out of a new file locks the
//! game back to stage 1 for want of anything in it. So that one read is left pointed at `score.dat` whatever
//! the mode. The three reads are told apart by which callback is running:
//!
//! | read | what it is for | which file |
//! | --- | --- | --- |
//! | `MainMenu::AddedCallback`, 0x43a5c0 | `clrd` and `pscr` into `g_GameManager` at 0x69ccd0 and 0x69cd30, the only place the front end's `Extra Start`, its practice stages and their scores come from | the game's own, always |
//! | `GameManager::AddedCallback`, 0x41bcdc, once per stage | the run's own record: the score to beat, and the spell cards it can add a capture to | the mode's |
//! | the ranking screen's added callback, 0x42f47f | the ranking on screen, and the record a score is entered into | the mode's |
//! | **orb's own**, wherever the mode is settled | `pscr` alone, over what the read above left | the mode's |
//!
//! **The fourth read is there because half of the first one is a record after all.** `pscr` is a score, one
//! per shot, difficulty and stage, and a stage practised in a chapter that can be played again was not
//! practised for that score — so what Pointdevice mode shows beside a practice stage belongs in orb's file
//! with the rest of that mode's runs, and it was `score.dat`'s number, the front end reading the scores in
//! the very read its items are lit from. One read lands in one file, so the score is read again: at the
//! answer to the mode question, orb calls the game's own read helper for the file the fork now points at,
//! parses `pscr` out of it into 0x69cd30, and gives the buffer back — 0x42b0d9, 0x42b65e and 0x42b7dc, the
//! three calls `MainMenu::AddedCallback` makes in that order. `clrd` is not parsed with it: that is what the
//! front end is lit from, and a read of orb's landing there would lock what an installation has earned.
//!
//! **At the answer, because that is the last thing before the screen those scores are drawn on.** The
//! practice stage select is `MainMenu`'s state 17, which the shot type select goes to instead of starting a
//! run where the run is a practice one (0x436dc5), and its rows are `STAGE %d  %.9d` — the score out of
//! `pscr` at 0x439a9b, for the shot and the difficulty already answered for. Nothing between that answer and
//! those rows reads the file again, and every item the screen is behind asks the question first. Which
//! stages are on it is not `pscr` at all: the count comes out of `clrd`, read at 0x4399e8 and held under six,
//! five of them on Easy. So what a mode decides here is the number on a row and never which rows there are.
//!
//! **A `clrd` record holds how far a shot has got, twice over, and the two are read by different screens.**
//! Per shot and difficulty there are two arrays of five: `GameManager::AddedCallback` raises the second at
//! every stage (0x41bf01) and the first only where the run has used no continue (0x41beb8, on the count at
//! `g_GameManager+0x1818` that a continue puts up at 0x402f0f), and a clear writes 99 — no stage's number —
//! into the first under that same condition (0x410c41) and into the second whatever happened (0x410c7d). The
//! Extra is behind the first: `GameManager::HasReachedMaxClears` at 0x43a736 compares Normal, Hard and
//! Lunatic against that 99, which is why clearing th06 on a continue earns no Extra. The practice stage
//! select counts its rows out of the second, so a stage reached on a continue can be practised.
//! `ParseClrd`'s fixup leaves every entry of both at 1 where it found no chunk, which is stage 1 reached:
//! one practice stage, and no Extra.
//!
//! The one write needs no telling apart: the exe reaches it from a single place, the ranking screen's
//! deleted callback on its way out, so a write while pointdevice mode is on is that screen's — a score
//! entered into it, or a ranking that was only looked at and written back as it was read, which is what
//! brings `pointdevice_score.dat` into existence before any run has finished. It carries `clrd` along with
//! the rest, because the game writes the file whole — so a pointdevice clear is recorded in orb's file and
//! **unlocks nothing**, the front end being lit from `score.dat`. That is the shape and not a gap in it: what
//! the front end offers is what runs somebody could have played have earned, and a run orb could rewind is
//! not one of those. Making a clear in the mode count would mean the union of the two files' `clrd`, and
//! with it orb knowing a format and an encryption it stays out of.
//!
//! **orb's file is a new one, not a copy of the game's.** Nothing seeds it: until a pointdevice run has
//! entered a score there is no `pointdevice_score.dat` at all, the open of one that is not there fails, and
//! the game takes that the way it takes a first launch. A copy would put the game's whole record at the top
//! of a ranking none of it belongs to — what a `score.dat` says has been cleared was cleared by runs nobody
//! could rewind, which is the one thing keeping these two files apart. The file being new costs a ranking
//! its history and the practice rows their scores, which is what a mode nobody has played in has to say, and
//! nothing else.
//!
//! **At the exe's import of `CreateFileA`**, not at the game's own score code, because that import is where
//! both of the game's own paths to the file end up. `score.dat` is one string, at 0x46af94, pushed at four
//! places: three reads through the helper at 0x42b0d9, which decrypts what it read (from 0x41bcdc, 0x42f47f
//! and 0x43a5c0), and one write through `FileSystem::WriteDataToFile` at 0x41e460, which encrypts before it
//! writes (from 0x42bc15). Both of those open with the CRT's `fopen` at 0x45ca0b — `"rb"` and `"wb"` — and
//! the CRT is statically linked, so the open reaches the OS at the exe's own import, IAT slot 0x46a150.
//! That slot is called from exactly two places: the CRT's open at 0x4677fa, and a file class at 0x43ceea
//! that the score file never goes through.
//!
//! orb's own read of the practice scores goes through that import with the rest, and it asks by the same
//! name — bytes of orb's rather than the string at 0x46af94, since which of the two files an open lands in
//! is decided from the name and the mode. So there is one place that decision is taken and the read orb
//! makes is forked the way the game's three are.
//!
//! So the redirect itself knows nothing about the game: no offset, and nothing about the format or the
//! encryption. It is the seam `memtrack` hooks for the heap calls, and d3d8's and dsound's own opens go
//! through their own imports and are not in the path.
//!
//! **One address is needed to tell the front end's read from the others**: `MainMenu::AddedCallback` at
//! 0x43a464, hooked so that the game's own file answers the read made while it runs and the mode's file
//! answers the rest. It is registered at 0x43a3c4 as the `+0x8` of the chain element at `menu+0x8234`, and
//! it is the callback that starts the title theme: `bgm/th06_01.mid` at 0x46c3a4, pushed at 0x43a475, which
//! is what identifies it. The parses identify each read: `hscr` at 0x42b280, `catk` at 0x42b466, `clrd` at
//! 0x42b502, `pscr` at 0x42b65e; this callback calls only the last two, and the ranking screen picks `hscr`
//! into a table of five difficulties by four shots at screen+0x3ab0. Nothing but which file an open lands in
//! rides on that address, and a game orb has no such address for lights its front end from whichever file
//! the mode points at — which for a mode whose file is new is nothing at all, and is the reason it is
//! hooked at all.
//!
//! **And three more to read the practice scores back**, which are the calls that read rather than a hook
//! over one: the helper at 0x42b0d9, `pscr`'s parse at 0x42b65e and the free at 0x42b7dc, called in that
//! order with the destination 0x69cd30 — where a record is 0x14 bytes and sits at `shot * 0x1e0 + stage *
//! 0x50 + difficulty * 0x14` (the arithmetic the parse itself indexes with at 0x42b783). A file that is not
//! there needs nothing extra: the helper answers a header with no chunks in it, allocated at 0x42b0f4, and
//! the parse clears every record it was going to fill — so a mode nobody has practised in shows zero rather
//! than the other mode's numbers. A game orb has none of these addresses for keeps whatever its own front
//! end read.
//!
//! **Only the open is redirected, which is enough here and would not be everywhere.** That other file class
//! truncates a `"w"` by calling `DeleteFileA` on the name before creating it — at 0x43cea9, its only call
//! site in the exe — so a game whose score write went that way would have its own file deleted while orb's
//! was written. th06's does not go that way.
//!
//! The whole file name is compared, ignoring case, and the directory the game named is kept. So
//! `pointdevice_score.dat` is not itself taken for the game's file and forked again — a second pass over it
//! would open `pointdevice_pointdevice_score.dat` — and a relative name resolves where the game's own open
//! would have resolved it. The paths are handled as the bytes the game gave and handed to the game's own
//! `CreateFileA` as those bytes: a directory name in the game's code page is not necessarily UTF-8, so
//! nothing in the path converts one. They are converted to text in one place, the log. Making the game
//! itself run on a machine whose code page is not Japanese is not orb's — th06 opens its own data files
//! through `CreateFileA` with their names held in the exe as Shift-JIS bytes, `紅魔郷ST.dat` at file offset
//! 0x6af74 and five more like it — and this hook does one thing to that import, which is which file the
//! score goes in.
//!
//! With `--no-chapters` the hook is not installed at all: nothing can rewind that run, and its score belongs
//! in the game's own file.
//!
//! **With `--clear` nothing is written**, whichever mode the run is in — there the hook is in the path for
//! the refusing rather than for the forking, and the file that must not be written is whichever this run
//! would have written. A cheated clear is not a score: orb's file is where runs that cannot be compared with
//! the game's are kept, and a clear nobody could have played at the top of *that* ranking is the same
//! mistake one file further on. The refusal is decided before the fork is, so it covers both files. The open
//! for writing is refused rather than the write being sent elsewhere, which the game takes cleanly:
//! `WriteDataToFile` checks its `fopen`, returns -1, and its one caller — the `call` at 0x42bc1a — drops
//! that and frees the buffer either way. Reads go through, so the ranking screen and what the file has
//! unlocked are what they were.

use std::borrow::Cow;
use std::ffi::{CStr, c_void};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::log;

/// The game's file, as the game asks for it.
///
/// Terminated, because one open of it is orb's own — see [`asked_for`] — and `CreateFileA` takes a
/// terminated string.
const THEIRS: &CStr = c"score.dat";
/// The same name as text, for a game that reads the file itself rather than asking the game's open for
/// it — `Game::read_what_the_modes_file_records` of a game whose one read is at startup.
pub const THEIRS_NAME: &str = "score.dat";
/// orb's, beside it and named for the mode whose runs are in it, so that a directory listing
/// says which is which.
///
/// An earlier version of orb called this `orb_score.dat`, and nothing reads or renames that name now: a
/// directory that has one has its old pointdevice scores in a file nothing opens. Renaming it on sight
/// would be orb writing to a file it did not create, over a name it cannot be sure of, so it is left
/// where it is.
const OURS: &[u8] = b"pointdevice_score.dat";

/// `GENERIC_WRITE`, which is Win32's own number for it.
///
/// Written out here rather than taken from `windows-sys`, and not because of the seam: what this bit is
/// read out of is the access mask the *game* passed to its own `CreateFileA`, which reaches this file as
/// one of the hook's arguments. So it is a number about the game's call and not a call of orb's, and the
/// half that reads it is this one.
const GENERIC_WRITE: u32 = 0x4000_0000;

/// `INVALID_HANDLE_VALUE`, which is what a refused open answers with.
///
/// Written out here for the same reason [`GENERIC_WRITE`] is, and held against Windows' own by a test in
/// `orb` — `the_refused_handle_is_windows_own` — rather than by a `const` assert beside it: a raw pointer
/// cannot be cast to an integer or compared in a constant, so this is one Windows number the compiler
/// cannot be made to check.
pub const INVALID_HANDLE: isize = -1;

/// Longer than any path the game opens, and a bound on the walk for the terminator: a
/// pointer that is not a string at all is handed back untouched rather than read until it
/// faults.
const LIMIT: usize = 1024;

/// The game's own `CreateFileA`, which the game's import table pointed at — or the function a game laid
/// out by hand handed over in place of it.
static CREATE_FILE_A: AtomicUsize = AtomicUsize::new(0);

static WRITTEN: AtomicBool = AtomicBool::new(true);

/// Whether what is being played is a run an attempt out of a replay is being watched in — see
/// [`watching`], which is where the reason is.
static WATCHED: AtomicBool = AtomicBool::new(false);
/// Whether opens of the score file go to orb's own, which is what pointdevice mode means for
/// this file. Off until orb says, so a run started before anything was chosen is the game's own
/// file — the same answer as no orb at all.
static FORKED: AtomicBool = AtomicBool::new(false);
/// Whether the read now happening is the one the front end's unlocks come out of, which is the one
/// read that is the game's own file whatever the mode.
///
/// One flag rather than anything per thread: the score file is opened on the game's own thread,
/// which is where the bracket is set and cleared, and nothing else in the process opens it.
static UNLOCKS: AtomicBool = AtomicBool::new(false);

/// Sends the score file to orb's own, or leaves it as the game's.
///
/// Called whenever the mode is settled, which is before anything opens the file: the ranking's
/// own screen opens it as it is built, and so does the title menu — which is where the mode is
/// chosen, one screen earlier.
pub fn fork(ours: bool) {
    FORKED.store(ours, Ordering::Relaxed);
}

/// Says that the read about to happen is the one the front end's unlocks come out of, or that it is
/// over.
///
/// Called around that read and nowhere else, so that it alone is answered from the game's own file
/// while everything else the file holds follows the mode.
pub fn reading_unlocks(inside: bool) {
    UNLOCKS.store(inside, Ordering::Relaxed);
}

/// Keeps the run being played out of the file altogether, rather than out of the game's.
///
/// For a run cleared with nothing able to hit the player: that is not a score, and orb's file
/// exists to keep runs that are not comparable apart from each other — a clear nobody could have
/// played at the top of it would be the same mistake one file further on. Reads are left alone,
/// so the session still shows the ranking it had.
pub fn refuse_writes() {
    WRITTEN.store(false, Ordering::Relaxed);
}

/// And keeps a run an attempt is being *watched* in out of the file, for as long as that lasts.
///
/// **What such a run counted was counted once already**, by the run the attempt came out of: an attempt that
/// happens to be the one the boss died on clears the run again, and a clear watched three times would be
/// three clears in the ranking. Nobody played it.
///
/// Its own flag rather than [`refuse_writes`], which is the launch's answer for a clear and holds for the
/// whole of one: this is set as an attempt starts and cleared when the front end is back, because the write
/// comes as the screen a finished run ends at goes down — which is after the run itself has gone.
pub fn watching(on: bool) {
    WATCHED.store(on, Ordering::Relaxed);
}

/// Whether an open of the score file is one to refuse. Only a write: a read is where the ranking
/// screen and what the file has unlocked come from, and a session showing none of those would
/// look like the file had been lost rather than left alone.
pub fn refused(access: u32) -> bool {
    (!WRITTEN.load(Ordering::Relaxed) || WATCHED.load(Ordering::Relaxed))
        && access & GENERIC_WRITE != 0
}

/// What the open now happening is for, as far as anything here can tell: a write, the read the front
/// end's unlocks come out of, or one of the two reads that cannot be told from each other —
/// `GameManager::AddedCallback`'s at every stage and the ranking screen's.
///
/// For the log, so that a line says which read it was and a reader is not left counting opens
/// against scenes to work it out.
pub fn asked(access: u32) -> &'static str {
    if access & GENERIC_WRITE != 0 {
        "write"
    } else if UNLOCKS.load(Ordering::Relaxed) {
        "read for the front end's unlocks"
    } else {
        "read"
    }
}

/// Whether the open now happening is one to send to orb's file.
///
/// Everything the file holds is the mode's own — the ranking, and beside it which spell cards a run
/// has captured, since a capture in a chapter that can be played again is not the capture the
/// game's record is a record of. **With one exception**, and it is not a record of anything: what
/// the front end offers. A stage that has been reached has been reached, and a first pointdevice
/// session answering that question out of its own new file locked the Extra stage and every
/// practice stage a `score.dat` had already earned — the module doc above has the three reads and what
/// each is for.
///
/// Reads and writes alike, which is why this asks nothing about the access: the exception is a read
/// the game makes nowhere near a write, and the exe reaches its score write from one place only —
/// the ranking screen on its way out. So a write while pointdevice mode is on is that screen's, a
/// score entered into it or a ranking that was only looked at and written back as it was read.
///
/// **The read that exception is made for brings the practice scores back with the unlocks**, and those
/// are the mode's: orb reads them again out of the file this answers with, which is the one open of it
/// that is orb's own rather than the game's — see [`asked_for`].
pub fn redirected() -> bool {
    FORKED.load(Ordering::Relaxed) && !UNLOCKS.load(Ordering::Relaxed)
}

/// The directory the game named its score file in, and `None` for every path that is not the
/// game's score file.
///
/// The whole file name is compared rather than its end, which is what keeps
/// `pointdevice_score.dat` from being taken for the game's file and forked again. The directory
/// is kept because that is what the game's own open would have resolved against — its own
/// directory for a relative name, wherever it says for an absolute one.
pub fn theirs(path: &[u8]) -> Option<&[u8]> {
    let cut = path
        .iter()
        .rposition(|byte| *byte == b'\\' || *byte == b'/')
        .map_or(0, |at| at + 1);
    let (directory, name) = path.split_at(cut);
    name.eq_ignore_ascii_case(THEIRS.to_bytes())
        .then_some(directory)
}

/// The name to ask for the score file by, for the one open of it orb makes itself: a game filling its
/// practice scores out of whichever file the mode points at — see `Game::read_what_the_modes_file_records`.
///
/// The game's own name and not orb's, because which of the two files an open lands in is decided from
/// the name and the mode by [`create_file_a`] — so an open orb makes is forked the same way the game's
/// three are, and there is one place that decision is taken.
pub fn asked_for() -> *const u8 {
    THEIRS.as_ptr().cast()
}

/// And which of the two an open of the game's own name lands in now, by that name alone: what the fork
/// decides, said to a game laid out by hand — whose reads of the file do not go through `CreateFileA` at all
/// and which therefore has no other way to know which file it is being asked for.
///
/// The unlocks bracket is not in it: what that read lands in is decided per read and this is asked outside
/// one.
pub fn the_file_now_opened() -> &'static str {
    if FORKED.load(Ordering::Relaxed) {
        // Its own bytes rather than `OURS`, which carries no terminator to strip.
        "pointdevice_score.dat"
    } else {
        "score.dat"
    }
}

/// orb's file in that directory, terminated for the open it is about to be handed to.
pub fn forked(directory: &[u8]) -> Vec<u8> {
    let mut ours = Vec::with_capacity(directory.len() + OURS.len() + 1);
    ours.extend_from_slice(directory);
    ours.extend_from_slice(OURS);
    ours.push(0);
    ours
}

/// A path for the log, and the only place these bytes are read as text: they are in the
/// game's code page, which is not UTF-8, so nothing that opens a file converts them at all.
pub fn text(path: &[u8]) -> Cow<'_, str> {
    let end = path.len() - usize::from(path.last() == Some(&0));
    String::from_utf8_lossy(&path[..end])
}

/// The game's own `CreateFileA`, which [`create_file_a`] calls through with the name it decided.
pub type CreateFileA = unsafe extern "system" fn(
    *const u8,
    u32,
    u32,
    *const c_void,
    u32,
    u32,
    *mut c_void,
) -> *mut c_void;

/// Says which function [`create_file_a`] calls through, which `orb::score::install` does with what it
/// took out of the import table — and a game laid out by hand with its own `CreateFileA`, there being no
/// import table to take one out of.
///
/// The same answer [`crate::window::install_over`] is, and for the same reason.
/// Which file an open lands in is what every e2e test about this file reads, and the path and the access
/// are the whole of what has to cross for that: a laid-out game answers with a handle of its own and there
/// is no file on any disk.
///
/// # Safety
/// `original` must be the game's own `CreateFileA` and outlive the last open it makes.
pub unsafe fn install_over(original: CreateFileA) {
    CREATE_FILE_A.store(original as usize, Ordering::Relaxed);
}

/// Whether orb is in the path of the game's own opens at all, which is whether an [`install_over`] has
/// happened: a launch with nothing to fork installs nothing, and the game's `CreateFileA` entry is the one
/// it always was.
///
/// **For a game laid out by hand**, which is the one thing that cannot answer this for itself: a real
/// launch's game reaches its own import entry whether or not it was patched, and a laid-out one has no
/// import table — it calls [`create_file_a`] where the entry was patched and its own where it was not, and
/// this is how it knows which. Without it a game whose launch forked nothing would be calling through a
/// null original.
pub fn installed() -> bool {
    CREATE_FILE_A.load(Ordering::Relaxed) != 0
}

/// Creates the file the mode says, in place of the one the game asked for.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// `CreateFileA` call lands, there being no import table to reach it through.
///
/// # Safety
/// The arguments are `CreateFileA`'s own, and an [`install_over`] must have run first — without one the
/// original this calls through is null.
pub unsafe extern "system" fn create_file_a(
    name: *const u8,
    access: u32,
    share: u32,
    security: *const c_void,
    disposition: u32,
    flags: u32,
    template: *mut c_void,
) -> *mut c_void {
    let original: CreateFileA =
        unsafe { std::mem::transmute(CREATE_FILE_A.load(Ordering::Relaxed)) };
    // Every archive, stage script and replay the game reads comes through here, so a path
    // that is not the score file costs one walk and one comparison.
    let Some(path) = (unsafe { given(name) }) else {
        return unsafe { original(name, access, share, security, disposition, flags, template) };
    };
    let Some(directory) = theirs(path) else {
        return unsafe { original(name, access, share, security, disposition, flags, template) };
    };
    // The open is refused rather than the write being sent somewhere else, because the game asks
    // for the file once, checks the open, and drops what its write returned — `WriteDataToFile` and
    // its one caller at 0x42bc1a, as the module doc says. So a refusal is a file that stays as it was, and a game that carries
    // on as if it had saved. Before the fork, since what must not be written is whichever file
    // this run would have written.
    if refused(access) {
        // Which of the two refusals it is, because they are different runs: one is a clear nothing could
        // hit the player in, the other an attempt out of a replay being watched again.
        log!(
            "score: nothing written, {}",
            if WATCHED.load(Ordering::Relaxed) {
                "this run is one an attempt is being watched in"
            } else {
                "this run had nothing able to hit the player"
            },
        );
        return INVALID_HANDLE as *mut c_void;
    }
    // Written whichever file the open lands in, so that no fact about this file has to be read out
    // of a line that is not there: an open that was left alone and an open that never happened look
    // the same in a log that only says when one was swapped, and telling those two apart is most of
    // what anybody reads these lines for.
    if !redirected() {
        log!(
            "score: {} opened as the game's own, {}",
            text(path),
            asked(access)
        );
        return unsafe { original(name, access, share, security, disposition, flags, template) };
    }
    // orb's file is made by the game writing it — which the ranking screen does on its way out,
    // whether a score was entered into it or the ranking was only looked at — and until then there
    // is nothing there: the open of a file that is not there fails, and the game takes that the way
    // it takes a first launch. It is not started as a copy of `score.dat`,
    // which would carry that file's record into a ranking none of it belongs to: what a
    // `score.dat` says has been cleared was cleared by runs nobody could rewind. The cost of
    // not copying is what such a file has unlocked — practice on a stage that has been reached,
    // the Extra stage — since the menu lights those from the file the mode points at.
    let ours = forked(directory);
    log!(
        "score: {} opened in place of the game's own, {}",
        text(&ours),
        asked(access)
    );
    unsafe {
        original(
            ours.as_ptr(),
            access,
            share,
            security,
            disposition,
            flags,
            template,
        )
    }
}

/// The game's own `fopen`, for a game whose opens are the universal CRT's — see
/// `crate::game::FileOpen::Fopen`.
static FOPEN: AtomicUsize = AtomicUsize::new(0);

/// `fopen(path, mode)`, answering a `FILE*` and null where it opened nothing.
pub type Fopen = unsafe extern "C" fn(*const u8, *const u8) -> *mut c_void;

/// Says which function [`fopen`] calls through — the same answer [`install_over`] is, for the other
/// import.
///
/// # Safety
/// `original` must be the game's own `fopen` and outlive the last open it makes.
pub unsafe fn install_over_fopen(original: Fopen) {
    FOPEN.store(original as usize, Ordering::Relaxed);
}

/// **The same fork over `fopen`**, for a game that opens its score file through the universal CRT.
///
/// **The mode string stands in for the access mask.** What [`create_file_a`] reads `GENERIC_WRITE` off is
/// its access argument; `fopen` says the same thing in its mode — `w`, `a` or a `+` is an open that writes
/// and `r` alone is one that reads — so the mode is read into the mask the rest of this file decides on, and
/// nothing below the decision is written twice. A refused open answers null, which is what the CRT answers
/// for a file it could not open and what a game reads as a first launch.
///
/// # Safety
/// The arguments are `fopen`'s own, and an [`install_over_fopen`] must have run first — without one the
/// original this calls through is null.
pub unsafe extern "C" fn fopen(name: *const u8, mode: *const u8) -> *mut c_void {
    let original: Fopen = unsafe { std::mem::transmute(FOPEN.load(Ordering::Relaxed)) };
    let Some(path) = (unsafe { given(name) }) else {
        return unsafe { original(name, mode) };
    };
    let Some(directory) = theirs(path) else {
        return unsafe { original(name, mode) };
    };
    let access = match unsafe { given(mode) } {
        Some(mode) if mode.iter().any(|byte| matches!(byte, b'w' | b'a' | b'+')) => GENERIC_WRITE,
        _ => GENERIC_READ,
    };
    if refused(access) {
        log!(
            "score: nothing written, {}",
            if WATCHED.load(Ordering::Relaxed) {
                "this run is one an attempt is being watched in"
            } else {
                "this run had nothing able to hit the player"
            },
        );
        return std::ptr::null_mut();
    }
    if !redirected() {
        log!(
            "score: {} opened as the game's own, {}",
            text(path),
            asked(access)
        );
        return unsafe { original(name, mode) };
    }
    let ours = forked(directory);
    log!(
        "score: {} opened in place of the game's own, {}",
        text(&ours),
        asked(access)
    );
    unsafe { original(ours.as_ptr(), mode) }
}

/// `GENERIC_READ`, the other of the two the game asks with — what an `fopen` for reading is read as.
const GENERIC_READ: u32 = 0x8000_0000;

/// The path the game gave, as its own bytes without the terminator, and `None` for a null
/// pointer or a string with no terminator inside [`LIMIT`].
///
/// **A raw pointer into the game's memory, walked here.** Which is the one thing about this file that
/// reads as though it were on the wrong side of the seam and is not: the pointer is the hook's own
/// argument, brought by the call, and the hook body is this crate's.
///
/// # Safety
/// `name` must be null or a readable NUL-terminated string.
unsafe fn given(name: *const u8) -> Option<&'static [u8]> {
    if name.is_null() {
        return None;
    }
    // Walked a byte at a time rather than taking `LIMIT` of them and looking for the
    // terminator: a short string near the end of a page has no such bytes to read.
    for length in 0..LIMIT {
        if unsafe { *name.add(length) } == 0 {
            return Some(unsafe { std::slice::from_raw_parts(name, length) });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        GENERIC_READ, GENERIC_WRITE, fork, forked, reading_unlocks, redirected, refuse_writes,
        refused, text, theirs,
    };

    /// Pointdevice mode sends the whole file to orb's — the ranking, and the spell cards a run has
    /// captured beside it — except the one read the front end's unlocks come out of, which is the
    /// game's own file however a run is played.
    #[test]
    fn everything_but_the_unlocks_follows_the_mode() {
        assert!(!redirected(), "nothing until the mode says so");
        fork(true);
        assert!(redirected());
        reading_unlocks(true);
        assert!(!redirected(), "what the front end offers");
        reading_unlocks(false);
        assert!(redirected());
        fork(false);
        assert!(!redirected(), "a normal run is the game's own file");
    }

    /// The path orb's file is opened at, for a path the game asked for.
    fn ours(path: &[u8]) -> Option<Vec<u8>> {
        theirs(path).map(forked)
    }

    /// Nothing is refused until a run says so, and then it is the write and only the write.
    #[test]
    fn a_run_whose_score_is_not_one_writes_nothing_and_still_reads() {
        assert!(!refused(GENERIC_WRITE));
        refuse_writes();
        assert!(refused(GENERIC_WRITE));
        assert!(!refused(GENERIC_READ));
    }

    #[test]
    fn the_game_score_file_is_forked_where_the_game_asked_for_it() {
        assert_eq!(
            ours(b"score.dat").as_deref(),
            Some(&b"pointdevice_score.dat\0"[..])
        );
        assert_eq!(
            ours(br"C:\game\score.dat").as_deref(),
            Some(&b"C:\\game\\pointdevice_score.dat\0"[..]),
        );
        assert_eq!(
            ours(b"data/score.dat").as_deref(),
            Some(&b"data/pointdevice_score.dat\0"[..])
        );
    }

    /// Every other path the game opens goes through untouched. The whole file name is what is
    /// compared, so a name that merely contains the game's, or extends it, is not it.
    #[test]
    fn nothing_else_is_forked() {
        for path in [
            &b"scoreboard.dat"[..],
            b"score.dat.bak",
            b"data/scores.dat",
            b"score.da",
            // A name in the game's own code page, which is not UTF-8: Shift-JIS 東.
            b"\x93\x8c.dat",
            b"",
        ] {
            assert!(theirs(path).is_none(), "{}", text(path));
        }
    }

    /// orb's own file is not itself forked: what is compared is the whole file name, and a
    /// second pass over it would open `pointdevice_pointdevice_score.dat`.
    #[test]
    fn orbs_own_file_is_left_alone() {
        assert!(theirs(b"pointdevice_score.dat").is_none());
        assert!(theirs(br"C:\game\pointdevice_score.dat").is_none());
    }

    /// The game asks in whatever case its own source spells it, and Windows does not care.
    #[test]
    fn the_case_it_is_asked_for_in_does_not_matter() {
        assert_eq!(
            ours(b"SCORE.DAT").as_deref(),
            Some(&b"pointdevice_score.dat\0"[..])
        );
    }
}
