//! **The clock the game paces itself by, answered running faster than it does.**
//!
//! For a game orb has no frame of: `frame::Pacing` shortens the wait between one frame and the next for
//! a game whose frames orb is *inside*, and there is nothing to shorten for one that waits on its own.
//! What such a game has instead is a clock it asks the host for, and the seam is the ask — the same
//! `GetProcAddress` that answers th06c's keyboard read answers its clocks, so a launch that wants the
//! game to run at twice its own rate gets it by handing back a clock that runs at twice the host's.
//!
//! **Three of them, because a game picks whichever the host has**: th06c holds the strings `timeGetTime`,
//! `GetTickCount`, `QueryPerformanceCounter` and `QueryPerformanceFrequency`, and asks for them by name.
//! All three counters are scaled together and the frequency is left alone: a frequency scaled as well
//! would say ticks are worth less and cancel the whole of it, and two clocks scaled by different amounts
//! are a game whose sleeps and whose pacing disagree.
//!
//! **And every answer is measured from that clock's own first reading**, rather than multiplying what the
//! host says: a counter multiplied outright jumps decades into the future on its first answer, and a
//! game that holds a startup timestamp against it waits out the difference.
//!
//! **What stays at the host's own rate is everything of orb's and everything on another thread.** orb's
//! own reading of time is `orb_api::clock`, which goes to the host through orb's own imports and not
//! through the game's table; the music is Media Foundation's own thread and its own clock, so a launch
//! run faster than the game is a launch whose music is wrong. A check of the sound is a launch of this
//! at one.

use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

/// **How many times its own rate the game is being run at**, one being a launch nothing here changes —
/// see `orb_config::Config::pace`.
static TIMES: AtomicI64 = AtomicI64::new(1);

/// What a clock had not answered yet, which no reading of one is: `timeGetTime` and `GetTickCount` are
/// unsigned milliseconds and a performance counter is positive, so a negative stands for no first
/// reading taken.
const NOT_YET: i64 = -1;

/// The host's own three, kept as they were answered — see [`the_millis_are`].
static MILLIS: AtomicUsize = AtomicUsize::new(0);
static TICKS: AtomicUsize = AtomicUsize::new(0);
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// The first reading each of them gave, which every later one of that clock is measured from.
static FROM_MILLIS: AtomicI64 = AtomicI64::new(NOT_YET);
static FROM_TICKS: AtomicI64 = AtomicI64::new(NOT_YET);
static FROM_COUNTER: AtomicI64 = AtomicI64::new(NOT_YET);

/// How much faster than the host the clocks orb answers with run.
///
/// # Safety
/// Must run at the attach, before the game has looked a clock up.
pub unsafe fn runs_at(times: u32) {
    TIMES.store(i64::from(times.max(1)), Ordering::Relaxed);
}

/// The host's own `timeGetTime`, which orb's answer calls through to.
///
/// # Safety
/// Must be what `GetProcAddress` answered for that name.
pub unsafe fn the_millis_are(host: usize) {
    MILLIS.store(host, Ordering::Relaxed);
}

/// The host's own `GetTickCount`.
///
/// # Safety
/// Must be what `GetProcAddress` answered for that name.
pub unsafe fn the_ticks_are(host: usize) {
    TICKS.store(host, Ordering::Relaxed);
}

/// The host's own `QueryPerformanceCounter`.
///
/// # Safety
/// Must be what `GetProcAddress` answered for that name.
pub unsafe fn the_counter_is(host: usize) {
    COUNTER.store(host, Ordering::Relaxed);
}

/// One reading scaled: as far past that clock's first reading as the time since it, times over.
fn scaled(from: &AtomicI64, now: i64) -> i64 {
    let first = match from.compare_exchange(NOT_YET, now, Ordering::Relaxed, Ordering::Relaxed) {
        Ok(_) => now,
        Err(first) => first,
    };
    first + (now - first) * TIMES.load(Ordering::Relaxed)
}

/// **`DWORD timeGetTime(void)`, answered faster.**
///
/// # Safety
/// [`the_millis_are`] must have been told what the host's own is.
pub unsafe extern "system" fn millis() -> u32 {
    let host: extern "system" fn() -> u32 =
        unsafe { std::mem::transmute(MILLIS.load(Ordering::Relaxed)) };
    scaled(&FROM_MILLIS, i64::from(host())) as u32
}

/// **`DWORD GetTickCount(void)`, answered faster.**
///
/// # Safety
/// [`the_ticks_are`] must have been told what the host's own is.
pub unsafe extern "system" fn ticks() -> u32 {
    let host: extern "system" fn() -> u32 =
        unsafe { std::mem::transmute(TICKS.load(Ordering::Relaxed)) };
    scaled(&FROM_TICKS, i64::from(host())) as u32
}

/// **`BOOL QueryPerformanceCounter(LARGE_INTEGER *count)`, answered faster.**
///
/// The host's own is called with the game's own pointer and what it wrote there is scaled in place, so a
/// host that refuses leaves the game with whatever it would have had.
///
/// # Safety
/// `count` must be the eight bytes the host's own read is given, which is what the game passes, and
/// [`the_counter_is`] must have been told what the host's own is.
pub unsafe extern "system" fn counter(count: *mut i64) -> i32 {
    let host: unsafe extern "system" fn(*mut i64) -> i32 =
        unsafe { std::mem::transmute(COUNTER.load(Ordering::Relaxed)) };
    let answer = unsafe { host(count) };
    if answer == 0 || count.is_null() {
        return answer;
    }
    unsafe { *count = scaled(&FROM_COUNTER, *count) };
    answer
}
