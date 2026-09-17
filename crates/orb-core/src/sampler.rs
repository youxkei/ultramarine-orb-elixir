//! Where the game was when its own update ran long.
//!
//! `profile` accounts for orb's phases and the frame line for the frame's, and between them a frame whose
//! time went inside the game's own update is a frame neither can say anything about: the update is one
//! call, and what it spent the time on is the game's business. This is what answers that question, and it
//! is the only part of orb that asks where the game is *executing* rather than what it holds: a thread of
//! orb's own watches how long the frame's thread has been inside the update and, once it has been in there
//! longer than a frame, reads where it is — its instruction pointer and the calls it is under, held still
//! for the read — a few times, and the frame writes those out with the update's own length when it comes
//! back.
//!
//! A thread rather than a hook inside the update, because the update is the game's: nothing of orb's runs
//! while the game is in it, so nothing on the frame's thread can take the sample. And samples rather than
//! a per-job timing of the calc chain, because the chain is walked by the game's own function and the
//! jobs in it are the game's — a walk of orb's own would be a second implementation of the game's, and one
//! that a laid-out game would have to be taught. Where the pointer lands is read off the exe's listing by
//! whoever reads the log, the way every other address in it is.
//!
//! Written at every level `pacing!` writes at, and nothing otherwise: the thread is not even started.
//!
//! **No e2e test reaches the line, and none is written for it.** `thread::where_it_is` answers `None`
//! under a simulated host on purpose: a laid-out game's update is a function of the test's own that
//! returns while the clock it advanced has not moved in real time, so a watching thread never catches one
//! in progress, and a simulated thread has no instruction pointer for a sample to be of anyway. A test
//! asserting that nothing is written would pass on the sim answering nothing and say nothing about orb, so
//! what this is checked by is the real game: a launch of th06c with the pacing on writes these lines, and
//! `Th06c::live_handles` carries what they said.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use orb_api::{clock, module, thread};

use crate::log;
use crate::profile;

/// When the frame's thread went into the game's update, as `profile::now()` has it, and zero while it is
/// out of it.
static UPDATE_BEGUN: AtomicI64 = AtomicI64::new(0);
/// Whether the watching thread has been started.
static WATCHING: AtomicBool = AtomicBool::new(false);
/// One sample of the frame's thread.
struct Sample {
    /// Which update it is of: [`UPDATE_BEGUN`]'s value at the time, which no two updates share.
    ///
    /// **Carried because a sample must not be written against another update.** The frame takes the lock
    /// with `try_lock` and so may leave its samples behind, and the watching thread's read of the frame's
    /// thread may finish after the update it was sampling has ended; either way what is left is a sample
    /// of an update whose line has gone, and a line saying a frame's thread was somewhere it never was is
    /// worse than no line.
    update: i64,
    /// How far into that update it was taken, in microseconds.
    at_us: i64,
    /// Where the thread was, as the module and offset of the instruction and of the calls it was under.
    /// Named on the watching thread, so that the frame does nothing but write the line.
    where_it_was: String,
}

static SAMPLES: Mutex<Vec<Sample>> = Mutex::new(Vec::new());

/// How long an update runs before it is worth sampling, in microseconds: two frames of sixty. One frame
/// is what a stage's heaviest moment costs on its own, and a sample of that says nothing a frame line
/// does not.
const LONG_US: i64 = 33_333;
/// How far apart two samples are, in microseconds, and how many are taken of one update: the first at
/// [`LONG_US`], then one every so often up to a bound, so that an update ten times too long is described
/// in a line and not a page.
const BETWEEN_SAMPLES_US: i64 = 16_667;
const SAMPLES_AN_UPDATE: usize = 8;
/// How long the watching thread sleeps between looks, in milliseconds.
const A_LOOK_MS: u32 = 4;

/// The frame's thread is going into the game's update, at `at`. Starts the watching thread the first
/// time, where the pacing is being written at all.
///
/// **The moment is the caller's own read of the counter and not one of this function's.** A read costs
/// the frame the read, and a simulated one charges the frame for it — `orb_sim::Clock::counter` — so a
/// sampler that took its own would move the numbers every pacing e2e test is about. It did: two of them
/// failed on 68us of work that was not the frame's.
pub fn the_update_begins(at: i64) {
    if !log::pacing_wanted() {
        return;
    }
    UPDATE_BEGUN.store(at, Ordering::Release);
    if WATCHING
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok()
        && thread::spawn(watch).is_err()
    {
        WATCHING.store(false, Ordering::Relaxed);
        log!("sampler: cannot start a thread; a long update goes unsampled");
    }
}

/// The frame's thread is back from the game's update, at `at`. Writes the samples the watching thread
/// took of it, where it took any — and `at` is the caller's own read of the counter, for the reason
/// [`the_update_begins`] gives.
pub fn the_update_ends(at: i64) {
    let begun = UPDATE_BEGUN.swap(0, Ordering::AcqRel);
    if begun == 0 {
        return;
    }
    let elapsed_us = profile::span_us(begun, at);
    // `try_lock` and not `lock`: the frame does not wait on the watching thread, which may be holding
    // the lock while it pushes a sample of an update that has just ended.
    let Ok(mut samples) = SAMPLES.try_lock() else {
        return;
    };
    let mine: Vec<String> = samples
        .iter()
        .filter(|sample| sample.update == begun)
        .map(|sample| format!("{}us in {}", sample.at_us, sample.where_it_was))
        .collect();
    samples.clear();
    if mine.is_empty() {
        return;
    }
    crate::pacing!(
        "update: {elapsed_us}us inside the game's own, where the frame's thread was {}",
        mine.join("; then "),
    );
}

/// The watching thread: looks every [`A_LOOK_MS`] at how long the frame's thread has been in the update,
/// and samples where it is once that is long.
fn watch() {
    // Below the game's, the way the joystick poller says of itself: this thread is not the frame's, and a
    // simulated host charges the frame for a counter read on any thread that has not said so — see
    // `orb_sim::Clock::counter`. Unsaid, its reads moved the frames a pacing e2e test asserts about.
    thread::below_normal();
    // **And gone at once where the host will not say where a thread is**, which is every e2e test: a
    // simulated one answers `None` by design, and the wait below is written down rather than waited out
    // there — so a thread that stayed would go round as fast as a core will let it for the whole of a
    // suite that runs its tests across the cores. It did: `orb-e2e` went from 193s to 250s.
    let frame = log::frame_thread();
    if frame == 0 || thread::where_it_is(frame).is_none() {
        log!("sampler: this host will not say where a thread is, so a long update goes unsampled");
        return;
    }
    let mut watching = 0;
    let mut taken_at_us = 0;
    let mut taken = 0;
    loop {
        clock::sleep(A_LOOK_MS);
        let begun = UPDATE_BEGUN.load(Ordering::Acquire);
        if begun == 0 {
            continue;
        }
        if begun != watching {
            watching = begun;
            taken_at_us = 0;
            taken = 0;
        }
        let elapsed_us = profile::since(begun);
        if elapsed_us < LONG_US
            || taken >= SAMPLES_AN_UPDATE
            || (taken > 0 && elapsed_us - taken_at_us < BETWEEN_SAMPLES_US)
        {
            continue;
        }
        let Some(executing) = thread::where_it_is(log::frame_thread()) else {
            continue;
        };
        // The update may have ended while the thread was being read: a sample of an update that is over
        // is dropped rather than written against the next one.
        if UPDATE_BEGUN.load(Ordering::Acquire) != begun {
            continue;
        }
        taken_at_us = elapsed_us;
        taken += 1;
        let mut where_it_was = format!("at {}", named(executing.at));
        for (call, address) in executing.under.iter().enumerate() {
            where_it_was += if call == 0 { ", under " } else { " " };
            where_it_was += &named(*address);
        }
        if let Ok(mut samples) = SAMPLES.lock() {
            samples.push(Sample {
                update: begun,
                at_us: elapsed_us,
                where_it_was,
            });
        }
    }
}

/// An address as the module it is in and how far into it, which is how it is looked up again — and as
/// itself where it is in no module, which a sampled address should never be and a stack word may be.
fn named(address: usize) -> String {
    match module::holding(address) {
        Some((path, base)) => format!(
            "{}+{:#x}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            address - base,
        ),
        None => format!("{address:#x}"),
    }
}
