//! Keeping a pointdevice run's scores out of the game's own file.
//!
//! A run that has been rewound is not a run anyone played, so its score does not belong in the
//! game's ranking. Refusing the write would lose the record altogether, so the file is forked
//! instead: while orb is in pointdevice mode every open of `score.dat` becomes an open of
//! `pointdevice_score.dat`. Pointdevice runs are then ranked against each other, in the game's
//! own format and on its own screen, and the game's file comes out of such a run unchanged
//! because it is never opened at all.
//!
//! **Which file is open is a runtime switch, not a setting.** The mode is chosen inside the
//! game — at the item that starts a run, and at the one that shows the ranking — so the fork
//! follows it: a normal run and the ranking of normal runs are the game's own file, which is
//! where a run anybody could have played belongs.
//!
//! Done at the exe's import of `CreateFileA` rather than at the game's own score code, which
//! is the seam `memtrack` uses and for the same reason: the game's statically linked CRT
//! reaches the OS through it. Both of 紅魔郷's own paths to the file end there — see
//! `SPEC.md` for the four places it names it and the call graph down to the import. So
//! nothing here is per-game: no address, no offset, and nothing about the file's format or
//! the encryption over it. d3d8's and dsound's own opens go through their own imports and are
//! not in the path.
//!
//! Only the open is redirected. A game that truncated a file by deleting it first — which is
//! what one of 紅魔郷's two file paths does, and not the one the score file takes — would have
//! its own file deleted while orb's was written, and would need `DeleteFileA` redirected
//! alongside this.

use std::borrow::Cow;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{
    GENERIC_WRITE, GetLastError, HANDLE, INVALID_HANDLE_VALUE, TRUE,
};
use windows_sys::Win32::Storage::FileSystem::CopyFileA;

use crate::hook;
use crate::log::log;

/// The game's file, as the game asks for it.
const THEIRS: &[u8] = b"score.dat";
/// orb's, beside it and named for the mode whose runs are in it, so that a directory listing
/// says which is which.
const OURS: &[u8] = b"pointdevice_score.dat";

/// Longer than any path the game opens, and a bound on the walk for the terminator: a
/// pointer that is not a string at all is handed back untouched rather than read until it
/// faults.
const LIMIT: usize = 1024;

static CREATE_FILE_A: AtomicUsize = AtomicUsize::new(0);
/// Whether the copy that starts orb's file off has been tried, whatever came of it.
static SEEDED: AtomicBool = AtomicBool::new(false);
/// Whether the file may be written at all.
static WRITTEN: AtomicBool = AtomicBool::new(true);
/// Whether opens of the score file go to orb's own, which is what pointdevice mode means for
/// this file. Off until orb says, so a run started before anything was chosen is the game's own
/// file — the same answer as no orb at all.
static FORKED: AtomicBool = AtomicBool::new(false);

/// Sends the score file to orb's own, or leaves it as the game's.
///
/// Called whenever the mode is settled, which is before anything opens the file: the ranking's
/// own screen opens it as it is built, and so does the title menu — which is where the mode is
/// chosen, one screen earlier.
pub fn fork(ours: bool) {
    FORKED.store(ours, Ordering::Relaxed);
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

/// # Safety
/// Must run before the game's entry point, or the game's first open of its score file is
/// the game's own. Patches `module`'s imports, so nothing else may be executing it.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "KERNEL32.dll",
            "CreateFileA",
            create_file_a as usize,
        )?
    };
    CREATE_FILE_A.store(previous, Ordering::Relaxed);
    Ok(())
}

type CreateFileA =
    unsafe extern "system" fn(*const u8, u32, u32, *const c_void, u32, u32, HANDLE) -> HANDLE;

unsafe extern "system" fn create_file_a(
    name: *const u8,
    access: u32,
    share: u32,
    security: *const c_void,
    disposition: u32,
    flags: u32,
    template: HANDLE,
) -> HANDLE {
    let original: CreateFileA =
        unsafe { std::mem::transmute(CREATE_FILE_A.load(Ordering::Relaxed)) };
    // Every archive, stage script and replay the game reads comes through here, so a path
    // that is not the score file costs one walk and one comparison.
    let Some(directory) = unsafe { given(name) }.and_then(theirs) else {
        return unsafe { original(name, access, share, security, disposition, flags, template) };
    };
    // The open is refused rather than the write being sent somewhere else, because the game asks
    // for the file once, checks the open, and drops what its write returned — see `SPEC.md` for
    // the two calls that is. So a refusal is a file that stays as it was, and a game that carries
    // on as if it had saved. Before the fork, since what must not be written is whichever file
    // this run would have written.
    if refused(access) {
        log!("score: nothing written, this run had nothing able to hit the player");
        return INVALID_HANDLE_VALUE;
    }
    if !FORKED.load(Ordering::Relaxed) {
        return unsafe { original(name, access, share, security, disposition, flags, template) };
    }
    let ours = forked(directory);
    if !SEEDED.swap(true, Ordering::Relaxed) {
        unsafe { seed(name, &ours) };
    }
    log!("score: {} opened in place of the game's own", text(&ours));
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

/// Whether an open of the score file is one to refuse. Only a write: a read is where the ranking
/// screen and what the file has unlocked come from, and a session showing none of those would
/// look like the file had been lost rather than left alone.
fn refused(access: u32) -> bool {
    !WRITTEN.load(Ordering::Relaxed) && access & GENERIC_WRITE != 0
}

/// The directory the game named its score file in, and `None` for every path that is not the
/// game's score file.
///
/// The whole file name is compared rather than its end, which is what keeps
/// `pointdevice_score.dat` from being taken for the game's file and forked again. The directory
/// is kept because that is what the game's own open would have resolved against — its own
/// directory for a relative name, wherever it says for an absolute one.
fn theirs(path: &[u8]) -> Option<&[u8]> {
    let cut = path
        .iter()
        .rposition(|byte| *byte == b'\\' || *byte == b'/')
        .map_or(0, |at| at + 1);
    let (directory, name) = path.split_at(cut);
    name.eq_ignore_ascii_case(THEIRS).then_some(directory)
}

/// orb's file in that directory, terminated for the open it is about to be handed to.
fn forked(directory: &[u8]) -> Vec<u8> {
    let mut ours = Vec::with_capacity(directory.len() + OURS.len() + 1);
    ours.extend_from_slice(directory);
    ours.extend_from_slice(OURS);
    ours.push(0);
    ours
}

/// Starts orb's file off as a copy of the game's, so that what a `score.dat` has already
/// unlocked — practice on a stage that has been reached, the Extra stage — is not locked
/// again by playing through orb. Only where there is nothing there yet, which `CopyFileA`
/// decides for itself; after that the two are separate records and the game's is never read
/// again.
///
/// Copied with the bytes the game gave rather than through `std::fs`: a path in the game's
/// own code page is not necessarily UTF-8, and `CopyFileA` takes the same ANSI string the
/// game's open would have.
///
/// # Safety
/// `theirs` must be the NUL-terminated path the game asked for, and `ours` the terminated
/// one it is being turned into.
unsafe fn seed(theirs: *const u8, ours: &[u8]) {
    if unsafe { CopyFileA(theirs, ours.as_ptr(), TRUE) } != 0 {
        return log!("score: {} started as a copy of the game's", text(ours));
    }
    // The two ordinary answers — orb's file is already there, and there is no `score.dat` to
    // copy because nothing has been played yet — are both errors to `CopyFileA`, so the code
    // is said rather than the line reading as a fault: 80 is one, 2 the other.
    log!(
        "score: {} not copied from the game's, GetLastError {}",
        text(ours),
        unsafe { GetLastError() }
    );
}

/// The path the game gave, as its own bytes without the terminator, and `None` for a null
/// pointer or a string with no terminator inside [`LIMIT`].
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

/// A path for the log, and the only place these bytes are read as text: they are in the
/// game's code page, which is not UTF-8, so nothing that opens a file converts them at all.
fn text(path: &[u8]) -> Cow<'_, str> {
    let end = path.len() - usize::from(path.last() == Some(&0));
    String::from_utf8_lossy(&path[..end])
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};

    use super::{forked, refuse_writes, refused, text, theirs};

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
