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
//! left to take the run through and the loop is on its way out. 紅魔郷 loses that record too.
//!
//! **What is left in `orb::score` is the write over the exe's import of `CreateFileA`.** The replacement
//! that entry is pointed at is [`create_file_a`] and it is here, being a hook body like the eleven in
//! [`crate::runtime`]: nothing in it is per-game and nothing in it is Windows. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use std::borrow::Cow;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::log;

/// The game's file, as the game asks for it.
const THEIRS: &[u8] = b"score.dat";
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

/// Whether an open of the score file is one to refuse. Only a write: a read is where the ranking
/// screen and what the file has unlocked come from, and a session showing none of those would
/// look like the file had been lost rather than left alone.
pub fn refused(access: u32) -> bool {
    !WRITTEN.load(Ordering::Relaxed) && access & GENERIC_WRITE != 0
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
/// practice stage a `score.dat` had already earned — see `SPEC.md` for the three reads and what
/// each is for.
///
/// Reads and writes alike, which is why this asks nothing about the access: the exception is a read
/// the game makes nowhere near a write, and the exe reaches its score write from one place only —
/// the ranking screen on its way out. So a write while pointdevice mode is on is that screen's, a
/// score entered into it or a ranking that was only looked at and written back as it was read.
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
    name.eq_ignore_ascii_case(THEIRS).then_some(directory)
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
/// The same answer [`crate::window::install_over`] is, and for the same reason — see
/// [docs/adr/0002](../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
/// Which file an open lands in is what every scenario about this file reads, and the path and the access
/// are the whole of what has to cross for that: a laid-out game answers with a handle of its own and there
/// is no file on any disk.
///
/// # Safety
/// `original` must be the game's own `CreateFileA` and outlive the last open it makes.
pub unsafe fn install_over(original: CreateFileA) {
    CREATE_FILE_A.store(original as usize, Ordering::Relaxed);
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
    // for the file once, checks the open, and drops what its write returned — see `SPEC.md` for
    // the two calls that is. So a refusal is a file that stays as it was, and a game that carries
    // on as if it had saved. Before the fork, since what must not be written is whichever file
    // this run would have written.
    if refused(access) {
        log!("score: nothing written, this run had nothing able to hit the player");
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
        GENERIC_WRITE, fork, forked, reading_unlocks, redirected, refuse_writes, refused, text,
        theirs,
    };

    /// `GENERIC_READ`, the other of the two the game asks with.
    const GENERIC_READ: u32 = 0x8000_0000;

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
