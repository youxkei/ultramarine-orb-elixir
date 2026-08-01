//! orb's own frame loop, in place of the game's.
//!
//! The game's own loop draws and only then updates, so what reaches the screen is
//! always one update behind the input that shaped it. Running the update first is what
//! removes that frame of lag.
//!
//! The pacing is the compositor's. Each frame waits for as many vertical blanks as
//! make one sixtieth of a second — two of them on a 120Hz display — so every frame is
//! shown for exactly as long as the last. Waiting for a blank blocks without spending
//! any CPU and needs no guess about how long a frame will take: the display says when,
//! and it is never wrong.
//!
//! A refresh rate that is not a whole multiple of 60 has no such cadence to keep — at
//! 144Hz a frame is 2.4 refreshes — so those fall back to pacing by the clock.

use std::sync::atomic::{AtomicI64, AtomicU32, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::{
    DWM_TIMING_INFO, DwmFlush, DwmGetCompositionTimingInfo,
};
use windows_sys::Win32::Media::{TIMERR_NOERROR, timeBeginPeriod, timeEndPeriod};
use windows_sys::Win32::Graphics::Gdi::{
    DEVMODEW, ENUM_CURRENT_SETTINGS, EnumDisplaySettingsW, GetDC, GetDeviceCaps, GetMonitorInfoW,
    MONITOR_DEFAULTTONEAREST, MONITORINFOEXW, MonitorFromWindow, ReleaseDC, VREFRESH,
};
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows_sys::Win32::System::Threading::Sleep;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use crate::log::{detail, log};

/// The rate the game's logic runs at, which is what its timers assume.
const LOGIC_HZ: u32 = 60;
/// Spinning is only worth it for the last stretch; before that, give the CPU up so
/// the sound and the rest of the system keep their share. Only the clock path spins;
/// waiting for a blank does not need it.
const SPIN_US: i64 = 1500;
/// How often the display is asked what it is doing. Cheap, but there is no reason to
/// ask more than once a second: it only changes when the window moves to another
/// monitor or the mode is changed.
const RESYNC_FRAMES: usize = 60;

static FREQUENCY: AtomicI64 = AtomicI64::new(0);
/// Blanks to wait for per game frame: 1 at 60Hz, 2 at 120Hz. Zero means the display
/// does not divide into 60 and the clock is pacing instead.
static BLANKS_PER_FRAME: AtomicU32 = AtomicU32::new(0);
/// One refresh in performance-counter ticks, used to say what the gaps between frames
/// came out as.
static PERIOD: AtomicI64 = AtomicI64::new(0);
/// Where the clock path is up to. Unused while the blanks are doing the pacing.
static NEXT_PRESENT: AtomicI64 = AtomicI64::new(0);
static SETTLE_AGE: AtomicUsize = AtomicUsize::new(RESYNC_FRAMES);
/// What the last log line about the display said, so the next one is only written when
/// the answer has changed. `-1` because no real answer packs to it.
static REPORTED: AtomicI64 = AtomicI64::new(-1);

/// How long before the blank a frame is shown at to start that frame's work, in
/// microseconds.
///
/// Every microsecond of it is input lag, and every microsecond too few is a frame
/// handed over after the compositor has stopped taking it — which the display shows a
/// refresh late. So it is measured rather than chosen: what the work has been taking
/// lately, plus enough to reach the compositor in time.
static PREPARE_US: AtomicI64 = AtomicI64::new(4000);
/// What handing the frame over needs on top of the work itself. The compositor wants
/// the frame some way before the blank it will be shown at, and how far is not
/// something it will say.
///
/// Only the floor of it, and the least that has ever been enough. What is actually used
/// is found by trying: a frame handed over too near the blank crosses it and is shown a
/// refresh late, which shows up as a gap of the wrong size, and the room given is raised
/// until that stops and then shaved back down. So the number below is a starting point
/// and a hard floor, not a claim about what the compositor wants.
const HANDOVER_FLOOR_US: i64 = 1200;
/// The room being left at the moment, over and above the work itself.
static HANDOVER_US: AtomicI64 = AtomicI64::new(HANDOVER_FLOOR_US);
/// How long a frame has been on screen lately, so the rate can be shown alongside the
/// lag it costs.
static INTERVAL_US: AtomicI64 = AtomicI64::new(0);
/// How long it has been taking from the keyboard being read to that frame being on
/// screen. Measured, not the `PREPARE_US` budget it is aimed at.
///
/// Spans two of orb's frames, because that is where the two ends are. `Present` does not
/// wait for anything — it queues the frame and returns in a tenth of a millisecond — so
/// the moment a frame actually reaches the screen is the moment the next frame's
/// `DwmFlush` comes back, that being the call that waits for the compositor.
static INPUT_LAG_US: AtomicI64 = AtomicI64::new(0);
/// When the frame now in the compositor's hands read the keyboard.
static INPUT_READ_AT: AtomicI64 = AtomicI64::new(0);
/// The compositor's own count of frames it could not show at the refresh they were
/// aimed at, as it stood when the log last mentioned it.
static WAS_LATE: AtomicI64 = AtomicI64::new(-1);
/// The same count, as the handover room last saw it. Kept apart from the log's copy so
/// that reading the log does not change what the pacing does.
static LATE_SEEN: AtomicI64 = AtomicI64::new(-1);
/// How often that count is asked for. A second: often enough to notice a display that
/// has started missing frames, seldom enough that the asking costs nothing.
const LATE_INTERVAL: usize = 60;
static LATE_AGE: AtomicUsize = AtomicUsize::new(0);

static FRAMES: AtomicUsize = AtomicUsize::new(0);
static LAST_PRESENT: AtomicI64 = AtomicI64::new(0);
/// How many frames may explain themselves between reports.
const LATE_LINES: usize = 6;
static SPOKEN: AtomicUsize = AtomicUsize::new(0);
/// How many refreshes apart the frames came out, counted. Says which pattern the
/// pacing is producing rather than only how often it was not the intended one, which
/// is the difference between a frame shown twice and one shown early. The last bucket
/// is everything longer.
static GAPS: [AtomicUsize; 6] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];

pub fn now() -> i64 {
    let mut counter = 0;
    unsafe { QueryPerformanceCounter(&mut counter) };
    counter
}

fn frequency() -> i64 {
    let cached = FREQUENCY.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    let mut frequency = 0;
    unsafe { QueryPerformanceFrequency(&mut frequency) };
    FREQUENCY.store(frequency.max(1), Ordering::Relaxed);
    frequency.max(1)
}

fn micros(ticks: i64) -> i64 {
    ticks * 1_000_000 / frequency()
}

fn ticks(micros: i64) -> i64 {
    micros * frequency() / 1_000_000
}

/// One sixtieth of a second, the rate the game's own timers assume.
fn frame_ticks() -> i64 {
    frequency() / i64::from(LOGIC_HZ)
}

/// Sets the cadence up before there is a window to ask about, from the desktop's own
/// refresh rate. `settle` replaces this with the game monitor's answer as soon as the
/// window exists.
pub fn configure() {
    NEXT_PRESENT.store(now(), Ordering::Relaxed);
    // Asked for once here rather than around each wait, as the game asked for it around
    // each of its own. Without it `Sleep` is only accurate to the system's tick, which
    // is some fifteen milliseconds — nearly two refreshes at 120Hz, and exactly the
    // size of the stutter measured whenever the pacing fell back to the clock.
    //
    // The game's own calls went with the loop that was replaced. vpatch does the same
    // thing for the same reason, and says so in its notes.
    let asked = unsafe { timeBeginPeriod(1) };
    if asked != TIMERR_NOERROR {
        log!("frame: the system will not give a 1ms timer ({asked}); waits will be coarse");
    }
    let screen = unsafe { GetDC(std::ptr::null_mut()) };
    let desktop = if screen.is_null() {
        None
    } else {
        let refresh = unsafe { GetDeviceCaps(screen, VREFRESH as i32) };
        unsafe { ReleaseDC(std::ptr::null_mut(), screen) };
        u32::try_from(refresh).ok().filter(|refresh| *refresh > 1)
    };
    adopt(desktop);
}

/// Works out the cadence from the monitor the game's window is on, and says so in the
/// log. Called once a second, so a window dragged to another monitor is followed.
fn settle(window: HWND) {
    let hz = monitor_refresh(window);
    adopt(hz);
    let blanks = BLANKS_PER_FRAME.load(Ordering::Relaxed);

    // The blanks come from the compositor, whose clock follows one monitor of the
    // desktop. Where that is not the monitor being drawn to, waiting on it paces the
    // game to the wrong rate — 144 read for a 120Hz display ran the game at 72 frames
    // a second — so it is checked rather than assumed.
    let composited = composition().map(|(period, _)| 1_000_000 / micros(period).max(1));
    let agrees = match (hz, composited) {
        (Some(hz), Some(composited)) => composited == i64::from(hz),
        _ => false,
    };

    let signature = i64::from(hz.unwrap_or(0)) << 8 | i64::from(blanks) << 1 | i64::from(agrees);
    if REPORTED.swap(signature, Ordering::Relaxed) == signature {
        return;
    }
    let reported = hz.map_or_else(|| "an unknown".to_string(), |hz| format!("{hz}Hz"));
    match (blanks, agrees) {
        (0, _) => {
            log!("frame: {reported} monitor is not a multiple of {LOGIC_HZ}Hz; pacing by the clock")
        }
        (blanks, true) => log!("frame: {reported} monitor, one frame every {blanks} blank(s)"),
        (_, false) => {
            let composited = composited.map_or_else(|| "nothing".to_string(), |hz| format!("{hz}Hz"));
            log!("frame: {reported} monitor but the compositor is timing {composited}; pacing by the clock");
        }
    }
}

/// Settles on how many blanks make a game frame, from what the monitor reports.
fn adopt(hz: Option<u32>) {
    let (blanks, period) = match hz {
        Some(hz) if hz >= LOGIC_HZ && hz % LOGIC_HZ == 0 => {
            (hz / LOGIC_HZ, frequency() / i64::from(hz))
        }
        Some(hz) => (0, frequency() / i64::from(hz)),
        None => (0, frame_ticks()),
    };
    BLANKS_PER_FRAME.store(blanks, Ordering::Relaxed);
    PERIOD.store(period, Ordering::Relaxed);
}

/// The refresh rate of the monitor the game's window is on, in whole Hz.
///
/// Asked of that monitor rather than of the desktop, because a second monitor at a
/// different rate is not this game's business.
fn monitor_refresh(window: HWND) -> Option<u32> {
    if window.is_null() {
        return None;
    }
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    if monitor.is_null() {
        return None;
    }
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    if unsafe { GetMonitorInfoW(monitor, (&raw mut info).cast()) } == 0 {
        return None;
    }
    let mut mode: DEVMODEW = unsafe { std::mem::zeroed() };
    mode.dmSize = size_of::<DEVMODEW>() as u16;
    let read =
        unsafe { EnumDisplaySettingsW(info.szDevice.as_ptr(), ENUM_CURRENT_SETTINGS, &mut mode) };
    (read != 0 && mode.dmDisplayFrequency > 1).then_some(mode.dmDisplayFrequency)
}

/// How many frames the compositor could not show at the refresh they were aimed at.
/// `None` when it will not say.
fn frames_late() -> Option<i64> {
    let mut info: DWM_TIMING_INFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<DWM_TIMING_INFO>() as u32;
    let asked = unsafe { DwmGetCompositionTimingInfo(std::ptr::null_mut(), &mut info) };
    (asked >= 0).then(|| info.cFramesLate as i64)
}

/// What the pacing is costing and what it is buying, for the screen: how long a frame
/// takes from reading the keyboard to being handed over, and how long a frame stays up.
/// Both measured, both microseconds.
pub fn status() -> (i64, i64) {
    (INPUT_LAG_US.load(Ordering::Relaxed), INTERVAL_US.load(Ordering::Relaxed))
}

/// What the compositor says about the blanks: how far apart they are in ticks, and
/// which one was the last, counted from when it started. `None` when it will not say.
fn composition() -> Option<(i64, i64)> {
    let mut info: DWM_TIMING_INFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<DWM_TIMING_INFO>() as u32;
    let asked = unsafe { DwmGetCompositionTimingInfo(std::ptr::null_mut(), &mut info) };
    (asked >= 0 && info.qpcRefreshPeriod != 0)
        .then(|| (info.qpcRefreshPeriod as i64, info.cRefresh as i64))
}

/// Waits until it is this frame's turn: the blank a whole frame's worth of refreshes
/// after the one the last frame began on.
///
/// Counted out by refresh number rather than by waiting for that many more blanks.
/// The two are the same only while a frame's work fits inside a single refresh, and the
/// game's update runs to ten milliseconds, which at 120Hz does not: asking for two more
/// blanks from a frame that has already run past one of them spends three refreshes on
/// a frame that had room in two, which is forty frames a second with a third of the
/// budget unspent.
///
/// The turns are refresh numbers and not moments on the clock, because it is being on
/// the blanks that makes the frames look even. A grid in clock time is exact to the
/// microsecond and still lands the frames wherever it likes across the refreshes, which
/// is visibly worse than being a refresh late now and then.
pub fn wait_for_slot(window: HWND) {
    if SETTLE_AGE.fetch_add(1, Ordering::Relaxed) >= RESYNC_FRAMES {
        SETTLE_AGE.store(0, Ordering::Relaxed);
        settle(window);
    }

    let blanks = i64::from(BLANKS_PER_FRAME.load(Ordering::Relaxed));
    let in_front = !window.is_null() && unsafe { GetForegroundWindow() } == window;
    // A window that is not in front has a cadence to keep, but counting refreshes
    // against it comes out wrong, and the clock will do until that is understood
    // rather than guessed at.
    //
    // A replay being run fast keeps the cadence like anything else: `replay_speed` is
    // updates per drawn frame, so the frames still come one per turn and only carry
    // more of the game with them.
    if blanks == 0 || !in_front {
        return wait_by_clock(blanks);
    }

    // One flush, as an anchor on a real blank, and the clock for the rest of the way.
    //
    // Not one flush per refresh the frame occupies. The frame is handed over just before
    // the blank it is to be shown at, so by the time the next frame asks, a blank is
    // moments away and its flush returns at once — a second flush would then wait out a
    // whole further refresh and the cadence would come out three refreshes instead of
    // two.
    //
    // Counting the compositor's refresh numbers was tried as well, so that a frame
    // running long would not cost the next one a refresh too. It cannot work: `cRefresh`
    // counts compositions of this window rather than refreshes of the display, and the
    // log had it advancing once per frame while eight flushes waited out eight
    // refreshes.
    if unsafe { DwmFlush() } < 0 {
        // The compositor has gone — turned off, or a session change. The clock will do
        // until `settle` notices.
        return wait_by_clock(blanks);
    }
    let blank = now();

    // The flush has just waited out the composition that put the last frame on screen, so
    // this is the far end of that frame's input lag and the near end was its own keyboard
    // read.
    let read_at = INPUT_READ_AT.swap(0, Ordering::Relaxed);
    if read_at != 0 {
        let shown = micros(blank - read_at);
        let lag = INPUT_LAG_US.load(Ordering::Relaxed);
        INPUT_LAG_US.store(if lag == 0 { shown } else { (lag * 7 + shown) / 8 }, Ordering::Relaxed);
    }

    // Then wait out almost all of the frame's turn, and do the work at the end of it.
    //
    // This is the whole of the input lag that is left to win. The work takes a fraction
    // of a refresh, so doing it at the start of the turn — where waiting for the blanks
    // leaves it — reads the keyboard a refresh and a half before the frame it appears
    // in. Doing it at the end reads the keyboard just before, which is as late as it can
    // be and still be that frame.
    let cadence = PERIOD.load(Ordering::Relaxed).max(1) * blanks;
    sleep_until(blank + cadence - ticks(PREPARE_US.load(Ordering::Relaxed)));
}

/// The fallback for a display that cannot be divided into 60.
fn wait_by_clock(blanks: i64) {
    let period = PERIOD.load(Ordering::Relaxed).max(1);
    let cadence = if blanks > 0 { period * blanks } else { frame_ticks() };

    let current = now();
    let mut target = NEXT_PRESENT.load(Ordering::Relaxed);
    // Nowhere near the plan — after a load, a snapshot, or a window that was not being
    // drawn. Start again from here rather than racing to catch up.
    if target < current - cadence * 4 || target > current + cadence * 4 {
        target = current + cadence;
    }
    // A frame that ran long is late, and the next slot is the one just ahead — not a
    // whole cadence from now.
    while target <= current {
        target += cadence;
    }
    NEXT_PRESENT.store(target + cadence, Ordering::Relaxed);
    sleep_until(target);
}

/// The moments a frame passed through, so a frame that took too long can say where it
/// went rather than leaving it to be guessed at.
pub struct Marks {
    /// Entering the frame, before anything has been done.
    pub started: i64,
    /// After the viewport and the background clear.
    pub cleared: i64,
    /// After waiting for the blanks.
    pub waited: i64,
    /// After the game's update.
    pub updated: i64,
    /// After the game's sounds, which is a separate mark because it talks to the
    /// sound device and the update does not.
    pub sounded: i64,
    /// After the draw and the overlay.
    pub drawn: i64,
    /// After handing the frame over.
    pub presented: i64,
}

/// Called once the frame has been handed over, to see what the display is getting.
pub fn finished(marks: Marks) {
    let previous = LAST_PRESENT.swap(marks.presented, Ordering::Relaxed);
    let period = PERIOD.load(Ordering::Relaxed).max(1);
    let blanks = BLANKS_PER_FRAME.load(Ordering::Relaxed);

    // What this frame took from reading the keyboard to being handed over, which is what
    // the next one has to leave room for.
    //
    // Rises at once and falls slowly, so it sits near the worst of the recent frames
    // rather than their average. Aiming at the average means missing the handover on
    // every frame heavier than it, and one frame handed over late is shown a whole
    // refresh late — far more visible than the microseconds of lag that aiming high
    // costs.
    let turn = micros(period * i64::from(blanks.max(1)));
    // Left for the next frame's flush to close off, which is when this frame is on screen.
    INPUT_READ_AT.store(marks.waited, Ordering::Relaxed);
    let took = micros(marks.presented - marks.waited) + HANDOVER_US.load(Ordering::Relaxed);
    let prepare = PREPARE_US.load(Ordering::Relaxed);
    let next = if took > prepare { took } else { prepare - (prepare - took) / 64 };
    // Never more than half the turn: past that, the lag is worse than the stutter it is
    // avoiding, and the fault is elsewhere.
    PREPARE_US.store(next.clamp(HANDOVER_FLOOR_US, turn / 2), Ordering::Relaxed);

    FRAMES.fetch_add(1, Ordering::Relaxed);
    if previous == 0 {
        return;
    }
    let gap = marks.presented - previous;
    let refreshes = (gap + period / 2) / period;
    let bucket = refreshes.clamp(0, GAPS.len() as i64 - 1) as usize;
    GAPS[bucket].fetch_add(1, Ordering::Relaxed);

    let interval = INTERVAL_US.load(Ordering::Relaxed);
    let smoothed = if interval == 0 { micros(gap) } else { (interval * 31 + micros(gap)) / 32 };
    INTERVAL_US.store(smoothed, Ordering::Relaxed);

    // How near the blank a frame may be handed over, found by trying rather than assumed.
    //
    // Judged on the compositor's own count of frames it could not show when they were
    // meant to be shown, which is the thing this room exists to prevent. The gaps between
    // our presents are the wrong signal: they go wrong for reasons that have nothing to do
    // with the handover — a stage loading, the process descheduled — and driving the room
    // from them ratcheted the lag up to seven milliseconds while the compositor was
    // reporting that not one frame had been late.
    if blanks > 0 && LATE_AGE.fetch_add(1, Ordering::Relaxed) >= LATE_INTERVAL {
        LATE_AGE.store(0, Ordering::Relaxed);
        if let Some(total) = frames_late() {
            let seen = LATE_SEEN.swap(total, Ordering::Relaxed);
            let room = HANDOVER_US.load(Ordering::Relaxed);
            // Up quickly, down slowly: a late frame is seen, and a millisecond of lag is
            // not.
            let next = if seen >= 0 && total > seen { room + 1000 } else { room - 250 };
            HANDOVER_US.store(next.clamp(HANDOVER_FLOOR_US, turn / 2), Ordering::Relaxed);
        }
    }

    // The breakdown, for the frames that did not come out on the cadence. Rationed,
    // because a bad patch would otherwise fill the log with the same line.
    if blanks == 0 || refreshes == i64::from(blanks) {
        return;
    }
    let spoken = SPOKEN.fetch_add(1, Ordering::Relaxed);
    if spoken >= LATE_LINES {
        return;
    }
    let us = |from: i64, to: i64| micros(to - from);
    detail!(
        "frame: {refreshes} refreshes — clear {}us wait {}us update {}us sound {}us draw {}us present {}us",
        us(marks.started, marks.cleared),
        us(marks.cleared, marks.waited),
        us(marks.waited, marks.updated),
        us(marks.updated, marks.sounded),
        us(marks.sounded, marks.drawn),
        us(marks.drawn, marks.presented),
    );
}

/// How the pacing is doing, for the log. With the blanks pacing, every gap should be
/// in one bucket; anything else means frames are reaching the display unevenly.
pub fn report() -> String {
    let frames = FRAMES.swap(0, Ordering::Relaxed);
    SPOKEN.store(0, Ordering::Relaxed);
    let mut gaps = String::new();
    for (refreshes, count) in GAPS.iter().enumerate() {
        let count = count.swap(0, Ordering::Relaxed);
        if count > 0 {
            let more = if refreshes == GAPS.len() - 1 { "+" } else { "" };
            gaps.push_str(&format!(" {refreshes}{more}x{count}"));
        }
    }
    // The compositor's own tally of frames it could not show when they were meant to be
    // shown. Our own gaps cannot see those: they measure when the frame was handed over,
    // not when it reached the screen. This is what says whether handing it over as late
    // as we do is late enough.
    let late = match frames_late() {
        Some(total) => {
            let previous = WAS_LATE.swap(total, Ordering::Relaxed);
            if previous < 0 { 0 } else { total - previous }
        }
        None => 0,
    };
    // The lag split into the two things it is made of, because they answer to different
    // causes: the work grows with what the game is drawing and is nobody's fault, while
    // the room is orb's own margin and should sit at its floor unless frames are actually
    // being shown late.
    let room = HANDOVER_US.load(Ordering::Relaxed);
    let prepare = PREPARE_US.load(Ordering::Relaxed);
    format!(
        "frame: {frames} frames, {}us apart, {prepare}us before the blank ({}us work + {room}us room), {late} shown late, gaps in refreshes{gaps}",
        INTERVAL_US.load(Ordering::Relaxed),
        (prepare - room).max(0),
    )
}

/// Gives the 1ms timer back, as its documentation asks.
pub fn release() {
    unsafe { timeEndPeriod(1) };
}

/// Sleeps most of the way there and spins the rest, because `Sleep` is only accurate
/// to about a millisecond and the last millisecond is the one that decides whether the
/// frame makes its slot.
fn sleep_until(deadline: i64) {
    loop {
        let remaining = micros(deadline - now());
        if remaining <= 0 {
            return;
        }
        if remaining > SPIN_US {
            unsafe { Sleep(((remaining - SPIN_US) / 1000).max(1) as u32) };
        } else {
            std::hint::spin_loop();
        }
    }
}
