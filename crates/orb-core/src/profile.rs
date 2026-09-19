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
    /// **The part of [`Phase::Update`] that falls before the game's update**, which is inside `Update`
    /// and not additional to it: orb's frame beginning to the moment the game's own update is called.
    ///
    /// **Its own phase because `Update` is two spans and the log gave one number for both.** `update`
    /// reports `calls=1200` over 600 frames — the span before the game's update and the span after it —
    /// so fifteen milliseconds a frame said nothing about which of the two holds them. This one is the
    /// first, and `update` minus it is the second.
    Ahead,
    /// **What the chaptering does on a frame that reaches no boundary**, which is
    /// [`Chapters::observe`] and nothing else — the walk that decides whether this frame is a chapter's
    /// and keeps what a retry would need.
    ///
    /// **Its own phase because [`Phase::Update`] hid it.** A run of th06nc in Pointdevice mode plays at
    /// about half the frame rate a legacy-mode one does — 31.9 against 58.9 frames a second, measured off
    /// two green launches of the real game — and `update`, which is orb's own per-frame work, is
    /// 16,231–23,168us of it against 1,800–6,820us for the same case driven `--no-chapters`. Neither
    /// [`Phase::Snapshot`] nor [`Phase::Regions`] covers any of that: both are recorded at a boundary,
    /// where the cost is flat across every frame. So the difference is here, and this is what says how
    /// much of it.
    Chaptering,
    /// **Keeping the chapter a frame belongs to** — `tune`, `boundary_reached` and `keep_chapter`
    /// together, which `runtime` runs on every frame of a run the chaptering is tracking.
    ///
    /// **The other half of [`Phase::Chaptering`], and separate because that one came back at 1us a
    /// frame** while taking the chaptering out entirely took fifteen milliseconds off — so the cost is
    /// here, in a boundary being looked for and the chapter behind it being held, rather than in the
    /// walk that decides whether to look.
    Keeping,
    /// **The line every frame of a run writes about how it reproduces** — `Game::reproduction`,
    /// `Game::the_worlds_fingerprint` and the `resume: frame … played with …` the two go into.
    ///
    /// **Its own phase because the fingerprint is a read per bullet slot.** `Th06nc`'s walks
    /// `bullet_manager::SLOTS` of them with `mem::read_committed`, which is one `VirtualQuery` apiece,
    /// and four more for every slot in use — on a frame of a run that is the largest count of reads orb
    /// makes anywhere. It is what is left of [`Phase::Update`] once [`Phase::Ahead`] came back at nought
    /// and [`Phase::State`] at three milliseconds of the twenty-one.
    Reproduction,
    /// **Reading the run's own numbers out of the game**, `Game::read_state`, which orb does once for
    /// every update the game makes and once more for a frame with no frame before it.
    ///
    /// **Its own phase because it is the largest unnamed thing inside [`Phase::Update`].** On th06nc in
    /// Pointdevice mode `update` is fifteen milliseconds a frame where the same case driven
    /// `--no-chapters` pays 1,800–6,820us, the chaptering's own work is measured and small
    /// ([`Phase::Chaptering`] and [`Phase::Keeping`], 1–20us), and the bot's steering is ruled out by a
    /// control that took no hits and paid the same. `Th06nc::read_state` is seven instruments and a walk
    /// of the fight's own slots, every read of it going through `mem::read_committed`.
    State,
    /// Taking a chapter snapshot.
    Snapshot,
    /// Finding the regions to save, which walks the game's heaps.
    Regions,
    /// Enumerating and suspending the game's other threads.
    Threads,
}

const PHASES: [(Phase, &str); 13] = [
    (Phase::Update, "update"),
    (Phase::Draw, "draw"),
    (Phase::Present, "present"),
    (Phase::Input, "input"),
    (Phase::Joystick, "joystick"),
    (Phase::Ahead, "ahead"),
    (Phase::Reproduction, "reproduction"),
    (Phase::Chaptering, "chaptering"),
    (Phase::Keeping, "keeping"),
    (Phase::State, "state"),
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

/// Microseconds between two `now()`s, for a span whose ends the caller already read.
pub fn span_us(from: i64, to: i64) -> i64 {
    microseconds(to - from, frequency())
}

/// # Safety
/// Must run on the game's main thread.
pub unsafe fn record(phase: Phase, since: i64) {
    unsafe { record_until(phase, since, now()) };
}

/// A phase whose end the caller has already read the counter for.
///
/// **Which is the whole reason this exists**: a read of the counter costs the frame the read, and a
/// simulated one charges the frame for it too — `orb_sim::Clock::counter`, whose cost every pacing e2e
/// test's numbers are derived from. So where a caller takes a `now()` of its own for the moment a phase
/// ends, that read is the one to charge the phase with, and [`record`]'s own would be a second.
///
/// # Safety
/// Must run on the game's main thread.
pub unsafe fn record_until(phase: Phase, since: i64, until: i64) {
    let elapsed = until - since;
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
