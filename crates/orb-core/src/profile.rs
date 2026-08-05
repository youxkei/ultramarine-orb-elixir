//! Where orb's per-frame time goes.
//!
//! A constant cost added to every frame shows up as the heavier stages dropping
//! below full speed while the lighter ones look fine, which is indistinguishable
//! from several different causes by eye. These numbers tell them apart.

use orb_api::clock;

use crate::log;
use crate::sync::MainThread;

/// How often the summary is written, in frames. Shared, so that what else is written once
/// a report lands beside the same set of numbers rather than drifting against them.
pub const INTERVAL: u32 = 600;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// orb's own per-frame work, not counting the game's update.
    Update,
    /// The overlay, including the render state it saves and restores.
    Draw,
    /// The game's own `Present`, which orb redirects through a letterbox.
    Present,
    /// The game reading the keyboard, joystick included.
    Input,
    /// The joystick part of that, which is inside `Input` and not additional to it.
    Joystick,
    /// Taking a chapter snapshot.
    Snapshot,
    /// Finding the regions to save, which walks the game's heaps.
    Regions,
    /// Enumerating and suspending the game's other threads.
    Threads,
}

const PHASES: [(Phase, &str); 8] = [
    (Phase::Update, "update"),
    (Phase::Draw, "draw"),
    (Phase::Present, "present"),
    (Phase::Input, "input"),
    (Phase::Joystick, "joystick"),
    (Phase::Snapshot, "snapshot"),
    (Phase::Regions, "regions"),
    (Phase::Threads, "threads"),
];

#[derive(Clone, Copy, Default)]
struct Slot {
    ticks: i64,
    worst: i64,
    calls: u32,
}

static SLOTS: MainThread<[Slot; PHASES.len()]> = MainThread::new(
    [Slot {
        ticks: 0,
        worst: 0,
        calls: 0,
    }; PHASES.len()],
);
static FRAMES: MainThread<u32> = MainThread::new(0);
/// The whole frame, measured between calls to `frame`. Everything the game does
/// is in here too, so it is what says whether a slowdown is orb's at all.
static FRAME: MainThread<Slot> = MainThread::new(Slot {
    ticks: 0,
    worst: 0,
    calls: 0,
});
static LAST_FRAME: MainThread<i64> = MainThread::new(0);

pub fn now() -> i64 {
    clock::counter()
}

/// Microseconds since a `now()`. For what is measured off the game's thread, and so has
/// no place in a table of what a frame spent.
pub fn since(started: i64) -> i64 {
    microseconds(now() - started, frequency())
}

/// # Safety
/// Must run on the game's main thread.
pub unsafe fn record(phase: Phase, since: i64) {
    let elapsed = now() - since;
    let slot = &mut unsafe { SLOTS.get() }[index(phase)];
    slot.ticks += elapsed;
    slot.worst = slot.worst.max(elapsed);
    slot.calls += 1;
}

/// Returns true on the frames it reports, so callers can add their own line
/// alongside it.
///
/// # Safety
/// Must run on the game's main thread, once per frame.
pub unsafe fn frame() -> bool {
    let now = now();
    let last = unsafe { LAST_FRAME.get() };
    if *last != 0 {
        let elapsed = now - *last;
        let frame = unsafe { FRAME.get() };
        frame.ticks += elapsed;
        frame.worst = frame.worst.max(elapsed);
        frame.calls += 1;
    }
    *last = now;

    let frames = unsafe { FRAMES.get() };
    *frames += 1;
    if *frames < INTERVAL {
        return false;
    }
    let counted = std::mem::replace(frames, 0);

    let frequency = frequency();
    let whole = std::mem::take(unsafe { FRAME.get() });
    let slots = unsafe { SLOTS.get() };
    let mut report = format!(
        "perf: frame={}us worst={}us over {counted} frames;",
        microseconds(whole.ticks, frequency) / i64::from(whole.calls.max(1)),
        microseconds(whole.worst, frequency),
    );
    for (phase, name) in PHASES {
        let slot = std::mem::take(&mut slots[index(phase)]);
        if slot.calls == 0 {
            continue;
        }
        report += &format!(
            " {name}={}us/frame worst={}us calls={}",
            microseconds(slot.ticks, frequency) / i64::from(counted),
            microseconds(slot.worst, frequency),
            slot.calls,
        );
    }
    if log::wanted(log::NORMAL) {
        log::line(&report);
    }
    true
}

fn microseconds(ticks: i64, frequency: i64) -> i64 {
    if frequency == 0 {
        0
    } else {
        ticks * 1_000_000 / frequency
    }
}

fn frequency() -> i64 {
    clock::frequency()
}

fn index(phase: Phase) -> usize {
    PHASES
        .iter()
        .position(|(known, _)| *known == phase)
        .unwrap_or(0)
}
