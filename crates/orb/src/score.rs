//! The open of the game's score file, stood in front of.
//!
//! Which file the open lands in is [`orb_core::score`] — this is the patch that gets in front of the
//! call, and the walk of the path the game handed over.
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

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};

use crate::hook;
use crate::log;
use orb_core::score::{asked, forked, redirected, refused, text, theirs};

/// The three switches the mode moves, under the names the install lists already call them by. Which
/// file an open lands in is decided above the seam and the switches are that decision being made, so
/// they are `orb-core`'s — and a caller here reaches them by the same path as before the split.
pub use orb_core::score::{fork, reading_unlocks, refuse_writes};

/// Longer than any path the game opens, and a bound on the walk for the terminator: a
/// pointer that is not a string at all is handed back untouched rather than read until it
/// faults.
const LIMIT: usize = 1024;

static CREATE_FILE_A: AtomicUsize = AtomicUsize::new(0);

/// # Safety
/// Must run before the game's entry point, or the game's first open of its score file is
/// the game's own. Patches `module`'s imports, so nothing else may be executing it.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    let previous = unsafe {
        hook::install_import(
            module,
            "KERNEL32.dll",
            "CreateFileA",
            hook::address(create_file_a as _),
        )?
    };
    CREATE_FILE_A.store(previous, Ordering::Relaxed);
    Ok(())
}

/// The same, for a game laid out by hand: its own `CreateFileA` in place of the import table there is
/// none of.
///
/// The same answer [`window::install_over`](crate::window::install_over) is, and for the same reason —
/// see [docs/adr/0002](../../../../docs/adr/0002-the-frame-loops-two-calls-into-the-game-are-addresses.md).
/// Which file an open lands in is what every scenario about this file reads, and the path and the access
/// are the whole of what has to cross for that: a laid-out game answers with a handle of its own and there
/// is no file on any disk.
///
/// # Safety
/// `original` must be the game's own `CreateFileA` and outlive the last open it makes.
pub unsafe fn install_over(original: CreateFileA) {
    CREATE_FILE_A.store(original as usize, Ordering::Relaxed);
}

/// The game's own `CreateFileA`, which [`create_file_a`] calls through with the name it decided. `pub`
/// because a game laid out by hand hands one of these over — see [`install_over`].
pub type CreateFileA =
    unsafe extern "system" fn(*const u8, u32, u32, *const c_void, u32, u32, HANDLE) -> HANDLE;

/// Creates the file the mode says, in place of the one the game asked for.
///
/// `pub` for the same reason the frame loop's hooks are: a game laid out by hand calls this where its own
/// `CreateFileA` call lands, there being no import table to reach it through.
///
/// # Safety
/// The arguments are `CreateFileA`'s own, and an [`install`] or an [`install_over`] must have run first —
/// without one the original this calls through is null.
pub unsafe extern "system" fn create_file_a(
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
        return INVALID_HANDLE_VALUE;
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
