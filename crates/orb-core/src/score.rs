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
//! **The open itself is `orb::score`**, which is the exe's import of `CreateFileA` hooked. This is the
//! half that decides *which* name that open is given, and it is the half a test can ask on its own:
//! nothing here is per-game and nothing here is Windows.

use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

/// The game's file, as the game asks for it.
const THEIRS: &[u8] = b"score.dat";
/// orb's, beside it and named for the mode whose runs are in it, so that a directory listing
/// says which is which.
const OURS: &[u8] = b"pointdevice_score.dat";

/// `GENERIC_WRITE`, which is Win32's own number for it.
///
/// Written out here rather than taken from `windows-sys`, and not because of the seam: what this bit is
/// read out of is the access mask the *game* passed to its own `CreateFileA`, which reaches this file as
/// one of the hook's arguments. So it is a number about the game's call and not a call of orb's, and the
/// half that reads it is this one.
const GENERIC_WRITE: u32 = 0x4000_0000;

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
