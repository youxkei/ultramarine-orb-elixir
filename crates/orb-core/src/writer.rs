//! The thread the files orb keeps of its own are written on, which is not the frame's.
//!
//! **Not in the frame**, where the first of them was: `WriteFile` takes what a disk, an antivirus reading
//! every write or a folder being synced decides, none of which orb can ask about beforehand, and a frame
//! that waits for one waits that long. On one machine a chapter's write stopped the game for about a
//! second each time. What only the frame can do is the *reading* — the run's numbers, the song's position
//! and the reproduction line are that frame's — so the frame reads, encodes and hands the bytes over, and
//! this is what puts them on the disk.
//!
//! **Not held for the frame's own slack** either, which is what [`crate::log`] does with its lines and is
//! the answer that suggests itself: the slack is what is left of one frame's turn, so a write of about a
//! second put there misses the next frame's blank instead of this one's — the same stutter, one frame
//! later.
//!
//! **One waiting write per file, and not a queue.** What a file holds is where the run is *now*, so a
//! write still waiting when the next one to the same file arrives is one nothing wants: the newer answers
//! the same question, and a queue would spend a slow disk's time writing places the run has already left.
//! Per file rather than one for everything, which is what it was: the chapter a run was left in and the
//! replay of that run are written at the same moment and are two files, so a single slot meant whichever
//! was handed over second threw the other away.
//!
//! Here rather than in [`crate::resume`], where it was written, because it is no longer that module's:
//! what a chapter and a replay share is the *thread*, and each of them says its own lines about its own
//! file — see [`Handed::about`].

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};

use crate::{detail, log};

/// How long [`stop`] waits for the writer to finish what it holds, in milliseconds.
///
/// Long enough for a write that is slow — which is what this whole arrangement is about — and not long
/// enough to hold a game that is closing behind a disk that has stopped answering. A file lost to that is
/// the version of it already on the disk still being there, which is the arrangement anyway.
const STOP_WAIT_MS: u64 = 2_000;

/// Where a write is slow enough to be worth a line at the level a run is ordinarily logged at, in
/// microseconds.
///
/// A frame at sixty a second, because that is the unit the thing being watched for is measured in: a
/// write that fits inside one would have cost the frame nothing even in the frame, and one that does not
/// is the reason the write is not in the frame any more.
const SLOW_WRITE_US: i64 = 16_666;

/// One file on its way to the disk: where it goes, the bytes to put there, and what the log calls it.
///
/// **The bytes and not the thing they are of**, because encoding is the frame's work and this thread has
/// nothing to encode with — a chapter's bytes are the game's own file format, and the thread has no game
/// to ask.
pub(crate) struct Handed {
    /// What the lines about this write start with, which is the word the rest of that mechanism's lines
    /// already start with: `resume` for a chapter a run was left in, `replay` for the run itself. So a
    /// log read for one of them holds every line about it, this thread's included.
    pub about: &'static str,
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    /// The line the write is said with, worked out where what is being written still is one.
    pub said: String,
    /// And what the line saying an older write never reached the disk calls the one it replaced.
    pub overtaken: String,
}

/// What the writer is waiting for.
struct Waiting {
    /// One entry per file — see the module's own note on why it is per file.
    next: Vec<Handed>,
    /// Whether the writer has been asked to stop, and whether it has — see [`stop`].
    stopping: bool,
    stopped: bool,
}

static WAITING: Mutex<Waiting> = Mutex::new(Waiting {
    next: Vec::new(),
    stopping: false,
    stopped: false,
});
/// What the writer waits on for something to write and the stopper waits on for the writer, so that
/// neither costs anything while nothing is happening.
static CHANGED: Condvar = Condvar::new();
/// Whether there is a writer. Cleared by [`stop`], so that a launch after one that was taken out gets a
/// thread of its own rather than handing files to a thread that has gone.
static WRITING: AtomicBool = AtomicBool::new(false);

/// Hands a file to the thread that writes it, and starts that thread the first time.
///
/// Where no thread can be made, the write happens here instead: a file written in the frame is worse
/// than a frame, and a file not written at all is a run nobody can pick up.
pub(crate) fn hand_over(handed: Handed) {
    if WRITING
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
        // Through the seam, which carries whatever simulated Windows this thread reads through onto
        // the one being made — see `orb_api::thread::spawn`.
        && orb_api::thread::spawn(writing).is_err()
    {
        log!("writer: cannot start a thread; what orb keeps of its own is written in the frame");
        WRITING.store(false, Ordering::Relaxed);
        write(&handed);
        return;
    }
    let Ok(mut waiting) = WAITING.lock() else {
        return log!("writer: the writer left its own state locked; nothing more is written");
    };
    // The same file answered again, which is what supersedes: the newer bytes are what that file wants,
    // and the older ones describe somewhere the run has already left.
    let overtaken = match waiting
        .next
        .iter()
        .position(|already| already.path == handed.path)
    {
        Some(at) => Some(std::mem::replace(&mut waiting.next[at], handed)),
        None => {
            waiting.next.push(handed);
            None
        }
    };
    drop(waiting);
    CHANGED.notify_all();
    if let Some(overtaken) = overtaken {
        detail!(
            "{}: {} was still waiting to be written; the newer one is what the file wants",
            overtaken.about,
            overtaken.overtaken,
        );
    }
}

/// The thread that writes them, one at a time, until it is stopped.
///
/// Below the game's priority, because a file written a millisecond later costs nobody anything and a late
/// frame costs the thing orb was written for.
///
/// Nothing is held while a write happens: the file is taken out from under the lock and the lock let go
/// of before the disk is touched, so a frame handing the next one over never waits for a write.
fn writing() {
    orb_api::thread::below_normal();
    loop {
        let Ok(mut waiting) = WAITING.lock() else {
            return;
        };
        while waiting.next.is_empty() && !waiting.stopping {
            match CHANGED.wait(waiting) {
                Ok(next) => waiting = next,
                Err(_) => return,
            }
        }
        // Whatever is waiting goes first, stopping or not: a file handed over and then dropped is a run
        // somebody would be offered the chapter before. The oldest of them first, so that two files
        // handed over on one frame reach the disk in the order the frame asked for.
        if !waiting.next.is_empty() {
            let handed = waiting.next.remove(0);
            drop(waiting);
            write(&handed);
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
/// the simulated Windows of a launch that has gone would write the next launch's files into it. What is
/// waiting is written first, this being the one moment there is to write it.
///
/// **The way out of a real launch does not come through here**: `DllMain`'s `DLL_PROCESS_DETACH` closes
/// the log and lets the process end, and waiting for a thread from inside the loader lock is the deadlock
/// every note about that lock is about. What waits here is a launch being taken out by hand, which is
/// what an e2e test does at the end of one.
pub fn stop() {
    if !WRITING.swap(false, Ordering::Relaxed) {
        return;
    }
    let Ok(mut waiting) = WAITING.lock() else {
        return log!("writer: the writer left its own state locked; it is not being waited for");
    };
    waiting.stopping = true;
    CHANGED.notify_all();
    let waited = CHANGED.wait_timeout_while(
        waiting,
        std::time::Duration::from_millis(STOP_WAIT_MS),
        |waiting| !waiting.stopped,
    );
    let Ok((mut waiting, timed_out)) = waited else {
        return log!("writer: the writer left its own state locked; it is not being waited for");
    };
    if timed_out.timed_out() {
        log!(
            "writer: the writer has not finished in {STOP_WAIT_MS}ms; what it holds goes unwritten"
        );
    }
    // Whatever came of it, the next launch in this process starts a writer of its own.
    *waiting = Waiting {
        next: Vec::new(),
        stopping: false,
        stopped: false,
    };
}

/// Writes one of them where it stands, and says what it cost.
///
/// The microseconds are the point of the line as much as the file is: the write being slow enough to stop
/// a frame is what took it off the frame, and this is where that is read.
fn write(handed: &Handed) {
    let Handed {
        about,
        path,
        bytes,
        said,
        ..
    } = handed;
    let started = crate::profile::now();
    // The directory is made where the file is written rather than at startup, so an installation nobody
    // has played a run in has nothing of orb's beside the game but the log.
    if let Some(parent) = path.parent()
        && let Err(error) = orb_api::fs::create_dir_all(parent)
    {
        return log!("{about}: cannot make {}: {error}", parent.display());
    }
    match orb_api::fs::write(path, bytes) {
        Ok(()) => {
            let us = crate::profile::since(started);
            // Not left to `verbose` alone, where this line was: a session on the machine a slow write
            // is about was played through at the ordinary level and said nothing about the disk,
            // because nobody knew there was a level to ask for. Nor said at the ordinary level every
            // time, which is a line a file about a disk that is behaving.
            if us >= SLOW_WRITE_US {
                log!(
                    "{about}: writing {} took longer than a frame: {us}us",
                    path.display(),
                );
            }
            detail!("{about}: {said} written to {} in {us}us", path.display());
        }
        Err(error) => log!("{about}: cannot write {}: {error}", path.display()),
    }
}
