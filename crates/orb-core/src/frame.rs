//! orb's own frame loop, in place of the game's.
//!
//! The game's own loop draws and only then updates, so what reaches the screen is
//! always one update behind the input that shaped it. Running the update first is what
//! removes that frame of lag.
//!
//! The pacing is the compositor's. Each frame waits for as many vertical blanks as
//! make one sixtieth of a second — two of them on a 120Hz display — so every frame is
//! shown for exactly as long as the last. Waiting for a blank blocks without spending
//! any CPU and needs no guess about how long a frame will take.
//!
//! `DwmFlush` is what waits, and what it waits for is the compositor composing the next
//! frame rather than the next blank as such — so it returns at the blank *the frame just
//! handed over reached*. That is worth knowing twice over: it is why a frame handed over
//! too near a blank costs the following one a refresh as well, and it is the only thing
//! that will say whether a frame made its blank at all. The compositor's own answers to
//! that question all read zero.
//!
//! A refresh rate that is not a whole multiple of 60 has no one cadence to keep — at 144Hz a
//! frame is 2.4 refreshes — so each frame goes on whichever blank is nearest where a
//! sixtieth-of-a-second grid has got to: two refreshes, three, two, three, two. The rate is 60
//! over any length of time and every frame is still shown at a blank. Only a display whose
//! refresh rate is not known at all is paced by the clock.

use crate::{log, pacing};
use orb_api::{Hwnd, clock, display, process, window};

/// The rate the game's logic runs at, which is what its timers assume.
pub const LOGIC_HZ: u32 = 60;
/// How much of the wait to a frame's own deadline is spun instead of waited out.
///
/// Spinning is only worth it for the last stretch; before that, give the CPU up so
/// the sound and the rest of the system keep their share. Only the wait to the frame's
/// own deadline spins; waiting for a blank does not need it.
///
/// **The number is what the wait overshoots by**, because covering that is the whole of what the spin
/// does: a wait aimed at the deadline less this lands *on* the deadline whenever it overshoots by
/// less than this, and hands the frame over after its blank has gone whenever it overshoots by more —
/// one refresh lost, visibly.
///
/// Measured, and set a little above the worst overshoot seen. Which is why the wait is a
/// high-resolution timer and not `Sleep`: this figure covers the timer's worst excursions and does
/// *not* cover `Sleep`'s, and what margin the old call really had came from rounding its wait down to
/// a whole millisecond — gone at every exact multiple and never counted on. See
/// [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md).
///
/// The margin over the worst seen is not large, so whoever finds frames arriving late should suspect
/// this number before anything else, measure their own host's overshoot, and raise it rather than
/// shrink it: what a larger one costs is a busier core, and what a smaller one costs is refreshes.
const SPIN_US: i64 = 1500;
/// How often the display is asked what it is doing. Cheap, but there is no reason to
/// ask more than once a second: it only changes when the window moves to another
/// monitor or the mode is changed.
const RESYNC_FRAMES: usize = 60;

/// What a host that cannot make the timer is told, and what orb ends the process with.
///
/// Named in full, because this is orb saying which program is stopping the game to somebody who has
/// only seen the game. The version is named too: the flag the timer needs is Windows 10 1803's, so
/// the answer to "why here and not on the other machine" is in the message rather than in a log.
const NO_TIMER_TITLE: &str = "Ultramarine Orb Elixir";
const NO_TIMER_TEXT: &str = "This host cannot create the high-resolution timer that orb paces the \
     game's frames on, and orb does not pace them any other way.\n\nWindows 10 version 1803 or \
     later is needed. The game is being stopped.";
/// Not zero, so that whatever started the game can tell this from a run that ended.
const NO_TIMER_CODE: u32 = 1;

/// Everything the pacing keeps between frames.
///
/// One value rather than a page of statics, so that a test can pace two displays without
/// being two processes. Nothing here is atomic and nothing needs to be: every field is
/// touched only from the thread the frame loop runs on, which is the game's.
pub struct Pacing {
    frequency: i64,

    /// Blanks to wait for per game frame: 1 at 60Hz, 2 at 120Hz. Zero means the display
    /// does not divide into 60 and the clock is pacing instead.
    blanks_per_frame: u32,

    /// Whether the blanks can pace this display at all, which needs only their spacing to be
    /// known. A rate that does not divide into 60 is still paced by them — see `IDEAL_NEXT`.
    blank_paced: bool,

    /// Where the sixtieth-of-a-second grid has got to, as a moment rather than a count of
    /// refreshes, because on most displays it does not land on refreshes.
    ///
    /// A frame is shown at the blank nearest this, and the grid then advances by exactly one
    /// sixtieth regardless of which blank that was. At 120Hz the answer is two refreshes every
    /// time and nothing about the pacing changes. At 144Hz, where a frame is 2.4 refreshes, it
    /// comes out two, three, two, three, two — whatever keeps the average at 2.4 — so the rate
    /// is exactly 60 over any length of time and every frame is still shown at a blank.
    ///
    /// Being a moment and not an accumulated count is what makes that self-correcting: the grid
    /// is absolute, so a frame put on the nearer blank does not push the ones after it.
    ideal_next: i64,

    /// How many refreshes the frame now in flight was given, which is what its gap should come
    /// out as. Zero when the clock paced it and there was no blank to aim at.
    aimed_refreshes: i64,

    /// A blank, kept as the phase every aim is measured from, and moved on only by frames that
    /// landed where they were aimed.
    ///
    /// The aim used to be the last landing plus a count of refreshes, which cannot correct itself:
    /// a frame that lands a refresh late becomes the reference for the next aim, so the grid asks
    /// for one refresh fewer and the lateness is absorbed instead of undone. That settles into a
    /// fixed point — measured at 144Hz as an aim averaging 2.2 refreshes with a frame in five
    /// landing a refresh late, which added back to exactly the 2.4 the display wanted, so the rate
    /// looked right while a fifth of the frames were shown somewhere nobody had asked for.
    ///
    /// Left alone by a late landing, so the frame after one aims at the blank the grid always meant
    /// and the pattern comes straight again.
    phase: i64,

    /// One refresh in performance-counter ticks, used to say what the gaps between frames
    /// came out as.
    period: i64,

    /// Where the clock path is up to. Unused while the blanks are doing the pacing.
    next_present: i64,

    settle_age: usize,

    /// What the last log line about the display said, so the next one is only written when
    /// the answer has changed. `-1` because no real answer packs to it.
    reported: i64,

    /// How many times the spacing the compositor reports moved over the period, by however little.
    ///
    /// Every move and not only the ones `adopt` acts on, because how restless the reading is is a
    /// property of the host worth having in the log: measured at a move on half of every second's
    /// reading, on a panel that held 120Hz throughout. Neither the rate nor the allowance can say
    /// that. `grid` divides a whole second by the spacing in microseconds, so 120Hz — 8333.33µs, on
    /// the boundary — rounds to 120 or to 119 depending on a tick and `whole_multiple` answers two
    /// blanks a frame to both, which is a rate that flips while nothing has changed; and the spacing
    /// moves far more often than the rate flips at all.
    period_moves: usize,

    /// How many of those were a different spacing rather than a restless reading of one, and had a
    /// found compose time to throw away with it.
    ///
    /// Apart from the count of moves because the two answer different questions — that one is what
    /// the host's reading does, this one is what orb did about it — and a reset is expensive: the
    /// only thing that raises the allowance again is a frame missing its blank, so giving back 2900µs
    /// buys four of them on the way up. A period reporting moves and no resets is the reading being
    /// restless and orb holding still, which is what it should do.
    compose_resets: usize,

    /// How long before the blank a frame is shown at to start that frame's work, in
    /// microseconds.
    ///
    /// Every microsecond of it is input lag, and every microsecond too few is a frame
    /// handed over after the compositor has stopped taking it — which the display shows a
    /// refresh late. So it is measured rather than chosen: what the work has been taking
    /// lately, plus enough to reach the compositor in time.
    ///
    /// Which is two things and they move separately: `next_budget` follows the work, and
    /// [`Self::measure_compose`] raises this by whatever it raised the compositor's share by, that
    /// share being the other half of what is being budgeted for.
    prepare_us: i64,

    /// What it is being given at the moment.
    compose_us: i64,

    /// How long a frame has been on screen lately, so the rate can be shown alongside the
    /// lag it costs.
    interval_us: i64,

    /// How long it has been taking from the keyboard being read to that frame being on
    /// screen. Measured, not the `PREPARE_US` budget it is aimed at.
    ///
    /// Spans two of orb's frames, because that is where the two ends are. `Present` does not
    /// wait for anything — it queues the frame and returns in a tenth of a millisecond — so
    /// the moment a frame actually reaches the screen is the moment the next frame's
    /// `DwmFlush` comes back, that being the call that waits for the compositor.
    input_lag_us: i64,

    /// When the frame now in the compositor's hands read the keyboard.
    input_read_at: i64,

    /// The compositor's own count of frames it could not show at the refresh they were
    /// aimed at, as it stood when the log last mentioned it.
    ///
    /// Reported and not acted on. It read zero through every run whose cadence was broken, so
    /// what it means is not what its name suggests, and how long the compositor gets is driven
    /// by the flush's overshoot instead.
    was_late: i64,

    /// The blank the last frame's turn was counted from, so the next frame can say whether it
    /// reached the flush while the blank that was its own was still ahead. Zero after a frame
    /// paced by the clock, whose anchor is not a blank at all.
    last_blank: i64,

    /// When the flush was called and when it came back. The two apart are what say whether a
    /// late frame lost its refresh before it did anything or spent it on something.
    flush_called: i64,

    blank_at: i64,

    /// How far past the blank that was its turn the frame reached the flush. Negative is in
    /// time, and says how much of the compose time was still there.
    arrival_us: i64,

    /// When everything orb does after handing a frame over was done with. The near end of the
    /// span the next frame has to reach the flush inside, and the only part of a frame that
    /// nothing used to measure.
    accounted: i64,

    /// What asking the display what it is doing cost this frame, zero on the frames it was
    /// not asked. It happens before the flush, so it is spent out of the compose time.
    settle_us: i64,

    /// How long [`Pacing::hold_for_the_blank_before`] held this frame back — how much further before
    /// its blank the budget would have handed it over than its own work asked for. Zero on every
    /// frame whose drawing finished after that earlier blank, which is nearly all of them.
    held_us: i64,

    /// How far the anchor the flush returned at sits after the blank the compositor says was
    /// the last one. Near zero means the anchor is a blank; a whole refresh means it is not,
    /// and every arrival measured against it is out by that much.
    anchor_us: i64,

    /// Which blank the last frame reached, counted in refreshes from the one it was aimed at:
    /// 0 is the blank it was aimed at, 1 the refresh after. The last is everything further.
    ///
    /// `DwmFlush` waits for the compositor to compose the next frame rather than for the next
    /// blank, so it returns at the blank *our own frame* reached — which makes this the answer
    /// to whether the frame made its blank, and the thing the compose time has to be driven by.
    ///
    /// The compositor's own answer to the same question is not usable. `cFramesLate` stays at
    /// zero through a run whose cadence is visibly broken, and `qpcFrameDisplayed` with
    /// `cFrameDisplayed`, `cFramesDropped`, `cFramesMissed` and `cRefreshesDisplayed` all read
    /// zero while `cFrameSubmitted` and `cFrameConfirmed` in the same read moved 1211 over a
    /// period — so the call works and that family is simply not populated for the desktop
    /// query, which is the only one it accepts.
    missed: [usize; 4],

    /// How far after the blank it was aimed at the last frame reached the screen, for the
    /// per-frame line.
    overshoot_us: i64,

    /// Frames shown *before* the blank they were aimed at, which is the one thing the buckets above
    /// cannot say: they count refreshes past the aim and take `max(0)` of it, so a frame a refresh
    /// early reads as one that landed exactly where it asked to.
    ///
    /// Not a cosmetic matter, and the reason it is counted at all. A frame per refresh is an update
    /// per refresh — bullets, enemies and the music's own clock with it — so the whole game runs at
    /// double speed while the log says nothing is wrong. It was noticed on the status line and
    /// nowhere else.
    early: usize,

    /// Whether the last frame landed more than one refresh past the blank it was aimed at, so the
    /// next one is still picking itself up.
    ///
    /// A stage load takes a quarter of a second, and the frame after one misses its blank
    /// through no fault of the compositor — nothing about that says it wants longer. Measured:
    /// over a long replay, most of the climbs the allowance made happened in the reporting
    /// periods that carried a load, and the value each climbed from had sat through many quiet
    /// periods without missing once.
    ///
    /// Cleared by the first frame that makes its blank rather than after a fixed count, since
    /// what it is waiting for is the loop being back on its feet and that is the thing that
    /// says so.
    recovering: bool,

    /// Misses laid at that door rather than at the compositor's, so the exemption is visible
    /// instead of being a silent reason the value stopped climbing.
    after_a_frame_past_a_refresh: usize,

    /// Whether the last frame's drawing outgrew the budget it was started against.
    ///
    /// Such a frame reaches the compositor late whatever the compositor was given, so the miss is
    /// the drawing's and giving the compositor longer answers nothing. Left unsaid it answers
    /// something worse: at 144Hz a heavy frame at startup climbed the compositor's share to the
    /// ceiling, and once there the budget — drawing and composing together — was the ceiling too,
    /// so the drawing had no allowance at all, every frame landed late, and every one of those
    /// asked for a climb that could not happen. 120 frames of every 600, for the rest of the run.
    overran: bool,

    /// Misses that were, counted apart for the same reason as the ones the frame before them excused.
    overrun_drawing: usize,

    /// Frames the clock paced rather than the blanks, because there is no blank to have missed
    /// on those and a period of them would otherwise read as a period with nothing wrong.
    clock_frames: usize,

    /// Frames whose count of refreshes came from the sixtieth's grid rather than from a fixed
    /// number of blanks, counted for the opposite reason to the clock's: those are two refreshes
    /// and three by design, so a period of them reads as a period of stutters and is not one.
    ///
    /// The pacing falls to the grid whenever the reported rate divides into no whole number of
    /// blanks — which 120Hz does not, until the compositor answers 108 or 109 for a second, and
    /// then the frames of that second are counted here and in no other number the log carries.
    /// `clock_frames` does not hold them: a measured spacing at or above 60Hz is paced by the
    /// blanks whether or not it divides.
    grid_frames: usize,

    /// The most the compositor has ever been seen to want more than, which it is never given
    /// less than again.
    ///
    /// A value a frame has already missed its blank at is known not to be enough for this
    /// display, and going back to it buys a stutter that has been paid for once already. So the
    /// shaving only ever tries values never shown to be short, and a stutter costs at most one
    /// frame per value rather than one every couple of minutes for as long as the game runs.
    ///
    /// This only ratchets upward, so a miss that was not really about the compositor costs lag for the
    /// rest of the run. What is left of those is a frame that reached the flush after its own blank had
    /// gone for a reason other than its drawing — `measure_compose` excuses the drawing and the frame
    /// after a landing more than a refresh out, and nothing has produced one that is neither. It errs
    /// toward giving the compositor longer, which is the safe direction, and the clamp at three quarters
    /// of a refresh is what bounds it.
    proven_short_us: i64,

    /// A compose time given on the command line, which pins it: 0 to find it while running.
    pinned_compose_us: i64,

    frames: usize,

    last_present: i64,

    spoken: usize,

    /// Late frames the ration kept out of the log, so a bad patch says how bad rather than
    /// only that it happened.
    unspoken: usize,

    /// The worst of the period, for the things a rate does not describe: an average interval
    /// hides a single 25ms frame, and the `5+` bucket does not say how much more.
    ///
    /// Arrivals beyond a whole turn are left out of the worst and counted on their own. A
    /// stage load takes a third of a second, and the frame after it arrives that late through
    /// no fault of the compose time — one of those in a period would be the whole of the
    /// worst and would say nothing about the frames either side of it.
    worst_arrival: i64,

    late_arrivals: usize,

    overrun_arrivals: usize,

    worst_settle: i64,

    worst_gap: i64,

    /// The shortest a frame was on screen over the period, which the refresh buckets cannot say.
    ///
    /// They round: `2x600` means every gap fell between one and a half and two and a half
    /// refreshes, and at 120Hz that is a four-millisecond window reported as one number. A period
    /// of frames alternating 15ms and 18ms is judder anybody would see and reads as `2x600`, so
    /// the buckets alone were never going to settle whether the pacing is smooth.
    best_gap: i64,

    /// How far off the cadence the gaps fell, in bands of half a millisecond either side, so the
    /// spread inside a bucket is visible rather than rounded away. The ends are everything
    /// further.
    jitter: [usize; 9],

    /// What the log itself cost over the period, both threads that write one.
    log_us: [i64; 2],

    log_writes: [i64; 2],

    /// How many refreshes apart the frames came out, counted. Says which pattern the
    /// pacing is producing rather than only how often it was not the intended one, which
    /// is the difference between a frame shown twice and one shown early. The last bucket
    /// is everything longer.
    gaps: [usize; 6],
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
    /// After [`Pacing::hold_for_the_blank_before`], which is the same moment as `drawn` on every
    /// frame whose drawing finished after that blank had gone — nearly all of them.
    ///
    /// Its own mark rather than folded into `presented`, because the hold is time the frame spent
    /// doing nothing and the spans in the pacing log have to add up to the gap. Folded in, it would
    /// read as a `present` that took a refresh.
    pub held: i64,
    /// After handing the frame over.
    pub presented: i64,
}

impl Pacing {
    /// The pacing as the DLL starts it, on the counter's own frequency.
    ///
    /// Read once, here, rather than cached on first use: every span the pacing works in is measured
    /// in its ticks, so a pacing that does not know it yet is one that cannot answer anything.
    ///
    /// No `Default`, although this takes no arguments: making one would put a read of the host's
    /// clock behind `Pacing::default()`, where nothing about the name says the host is asked
    /// anything. A pacing cannot be made without asking, so the making says so.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::at(clock::frequency().max(1))
    }

    /// At a stated frequency, for the tests that work in microseconds as if they were ticks.
    fn at(frequency: i64) -> Self {
        Self {
            frequency,
            blanks_per_frame: 0,
            blank_paced: false,
            ideal_next: 0,
            aimed_refreshes: 0,
            phase: 0,
            period: 0,
            next_present: 0,
            settle_age: RESYNC_FRAMES,
            reported: -1,
            period_moves: 0,
            compose_resets: 0,
            prepare_us: 4000,
            compose_us: COMPOSE_START_US,
            interval_us: 0,
            input_lag_us: 0,
            input_read_at: 0,
            was_late: -1,
            last_blank: 0,
            flush_called: 0,
            blank_at: 0,
            arrival_us: 0,
            accounted: 0,
            settle_us: 0,
            held_us: 0,
            anchor_us: 0,
            missed: [0; 4],
            overshoot_us: 0,
            early: 0,
            recovering: false,
            after_a_frame_past_a_refresh: 0,
            overran: false,
            overrun_drawing: 0,
            clock_frames: 0,
            grid_frames: 0,
            proven_short_us: 0,
            pinned_compose_us: 0,
            frames: 0,
            last_present: 0,
            spoken: 0,
            unspoken: 0,
            worst_arrival: i64::MIN,
            late_arrivals: 0,
            overrun_arrivals: 0,
            worst_settle: 0,
            worst_gap: 0,
            best_gap: i64::MAX,
            jitter: [0; 9],
            log_us: [0; 2],
            log_writes: [0; 2],
            gaps: [0; 6],
        }
    }
}

/// How long the compositor is given to draw, over and above the frame's own drawing.
///
/// The window between `Present` and the blank is not idle: it is where the compositor
/// composes the desktop and gets it onto the screen for that blank. So a frame's turn holds
/// two drawing times, the game's and the compositor's, and both have to finish before the
/// blank or the frame is shown at the one after.
///
/// The least it may be given, which only a pinned sweep goes near.
const COMPOSE_FLOOR_US: i64 = 1000;
/// What it starts at, and it starts high on purpose.
///
/// What the compositor needs is not a threshold with a value below it that always fails and
/// above it that always works — it is a distribution, and more time only makes a miss rarer.
/// Measured by climbing in small steps over tens of thousands of frames: past a certain point
/// every value carried the frames that came after it, and every one of them still missed now
/// and then, so the climb stuttered its way up and there was no edge to arrive at.
///
/// So this is a margin past the knee rather than a reading of any machine, found by sweeping pinned
/// values until the misses stopped. Somebody who would rather have the microseconds back can find
/// their own floor with `--compose=N`.
///
/// **A run that never stutters says nothing about how far above the knee this is.** The climb only
/// goes up, so a display whose compositor is satisfied here reports this number back whatever it
/// would have settled for — the microseconds between the two are real input lag and invisible. What
/// finds them is `--compose=N` below this and a run long enough to stutter at it, which is the sweep
/// above rather than anything a paced run can be read for.
const COMPOSE_START_US: i64 = 2500;
/// What a frame that missed its blank adds to it. Enough to be worth the trip, since a miss
/// is one stutter and a hundred microseconds is not; small enough that a frame which missed
/// for some reason of its own does not cost half a millisecond of lag for the rest of the run.
const MISS_STEP_US: i64 = 100;

/// How many frames may explain themselves between reports.
const LATE_LINES: usize = 6;
/// Half a millisecond, and four bands either side of the cadence before the ends.
const JITTER_BAND_US: i64 = 500;

impl Pacing {
    fn micros(&self, ticks: i64) -> i64 {
        ticks * 1_000_000 / self.frequency
    }

    fn ticks(&self, micros: i64) -> i64 {
        micros * self.frequency / 1_000_000
    }

    /// One sixtieth of a second, the rate the game's own timers assume.
    fn frame_ticks(&self) -> i64 {
        self.frequency / i64::from(LOGIC_HZ)
    }

    /// Holds the compositor's drawing time at what the command line said, instead of letting it
    /// be found while running. 0 leaves it to be found.
    ///
    /// For sweeping it: the compositor's own count of frames it could not show stays at zero
    /// through runs whose cadence is broken, so what it needs is established by pinning it too
    /// small until frames are known to miss and walking it up until they stop.
    /// The least the compositor may be given.
    ///
    /// A pinned value is the whole of it, because pinning is for measuring one value and a floor
    /// that overrode it would make every reading below `COMPOSE_FLOOR_US` the same reading.
    ///
    /// Otherwise it is what has been shown to be too little, which the frame's own drawing time
    /// is not allowed to be clamped under either.
    fn compose_floor(&self) -> i64 {
        let pinned = self.pinned_compose_us;
        if pinned > 0 {
            return pinned;
        }
        COMPOSE_FLOOR_US.max(self.proven_short_us)
    }

    pub fn pin_compose(&mut self, us: u32) {
        if us == 0 {
            return;
        }
        let us = i64::from(us);
        self.pinned_compose_us = us;
        self.compose_us = us;
        log!("frame: the compositor's drawing time is pinned at {us}us rather than found");
    }

    /// Sets the cadence up before there is a window to ask about, from the desktop's own
    /// refresh rate. `settle` replaces this with the game monitor's answer as soon as the
    /// window exists.
    pub fn configure(&mut self) {
        self.next_present = now();
        // Nothing is asked of the system's timer resolution here, and nothing needs to be: the wait
        // to a frame's deadline is a high-resolution waitable timer, which is not tied to the system
        // tick. `timeBeginPeriod(1)` was asked for here as the game asked for it around each of its
        // own waits, and it bought `Sleep` a millisecond it is no longer waiting through.
        //
        // The timer itself is not made here either: `configure` runs inside `DllMain`, and the one
        // thing a host that cannot make it gets is a `MessageBoxW` — which under the loader lock is
        // the textbook deadlock. So it is made on first use from the frame hook; see `no_timer`.
        let (hz, measured) = self.grid(display::desktop_refresh());
        self.adopt(hz, measured);
    }

    /// The grid the frames are put on, as a rate in whole Hz and the spacing of its blanks in ticks.
    ///
    /// **The compositor's whenever there is one**, because that is the only grid a frame can be put on:
    /// `DwmFlush` returns at its blanks and at nobody else's. Both halves of that are measured.
    ///
    /// Measured on a mixed-rate desktop: the compositor reports one period for the whole desktop and it
    /// is the *fastest* monitor's, neither the primary's nor the window's, while `EnumDisplaySettingsW`
    /// answers about each panel's own. A window on any of the monitors flushes at that one compositor
    /// rate. So the two numbers `adopt` compares really can disagree, and only the first of them moves
    /// with the window.
    ///
    /// And the frames really are composed against that grid: measured by handing a frame over a chosen
    /// lead before a blank, a frame handed over too near one never makes it and a frame handed over far
    /// enough ahead always does, with the same thresholds for a window covered by a full-screen one as
    /// for a window in front.
    ///
    /// So `fallback`, which is what a monitor or the desktop reports for itself, is only the rate to
    /// count in where the compositor will not say. Counting in it while flushing on the compositor's
    /// blanks is what ran a 120Hz window fast on a desktop the compositor timed at 144: the frames went
    /// on 6944µs blanks while the cadence asked for two of the monitor's 8333µs ones.
    fn grid(&self, fallback: Option<u32>) -> (Option<u32>, Option<i64>) {
        match composition() {
            Some((period, _)) => (
                u32::try_from(1_000_000 / self.micros(period).max(1)).ok(),
                Some(period),
            ),
            None => (fallback, None),
        }
    }

    /// Works out the cadence from the grid the frames are going on, and says so in the
    /// log. Called once a second, so a window dragged to another monitor is followed.
    fn settle(&mut self, window: Hwnd) {
        let monitor = display::monitor_refresh(window);
        let (hz, measured) = self.grid(monitor);
        self.adopt(hz, measured);
        let blanks = self.blanks_per_frame;

        // Whether the monitor the window is on is the one the compositor's clock follows. It decides
        // nothing any more — the cadence is counted in the compositor's own spacing either way — and it
        // is worth a line of its own, because a desktop where the two disagree is one where a frame
        // shown on the compositor's blank still has the panel's own to reach.
        let agrees = match (monitor, hz) {
            (Some(monitor), Some(hz)) => same_rate(i64::from(hz), i64::from(monitor)),
            _ => false,
        };
        let signature =
            i64::from(hz.unwrap_or(0)) << 8 | i64::from(blanks) << 1 | i64::from(agrees);
        if std::mem::replace(&mut self.reported, signature) == signature {
            return;
        }
        let named = hz.map_or_else(|| "an unknown".to_string(), |hz| format!("{hz}Hz"));
        let whose = if measured.is_some() {
            "compositor"
        } else {
            "monitor"
        };
        match blanks {
            0 if self.blank_paced => log!(
                "frame: {named} {whose} is not a multiple of {LOGIC_HZ}Hz; one frame on whichever blank is nearest each sixtieth"
            ),
            0 => {
                log!("frame: {named} {whose} and the compositor will not say; pacing by the clock")
            }
            blanks => log!("frame: {named} {whose}, one frame every {blanks} blank(s)"),
        }
        // And the desktop it is happening on, once, where the panel is not what is being timed.
        if !agrees {
            let panel = monitor.map_or_else(|| "an unknown".to_string(), |hz| format!("{hz}Hz"));
            log!(
                "frame: the window's own monitor is {panel}, which is not what the compositor is timing"
            );
        }
    }

    /// Settles on how the blanks are spaced, and whether they are known well enough to pace by.
    ///
    /// A rate that does not divide into 60 is paced by them too, on the nearest blank to each
    /// sixtieth — the count per frame is not a constant there, so it is worked out per frame
    /// rather than settled here. Only a display whose rate is not known at all falls back to the
    /// clock.
    fn adopt(&mut self, hz: Option<u32>, measured: Option<i64>) {
        let (blanks, period) = match hz {
            // The measured spacing where there is one, and the nominal multiple's where there is not:
            // for a rate that reports short the nominal is the nearer of the two, a 119.88Hz refresh
            // being 8341µs, which 120 puts at 8333 and 119 at 8403.
            Some(hz) => match whole_multiple(hz) {
                Some(multiple) => (
                    multiple,
                    measured.unwrap_or(self.frequency / i64::from(multiple * LOGIC_HZ)),
                ),
                None => (0, measured.unwrap_or(self.frequency / i64::from(hz))),
            },
            None => (0, self.frame_ticks()),
        };
        // A whole multiple is paced by the blanks on that alone, as it always was. A rate that is not
        // one is too, where the compositor measured the spacing rather than it being derived from a
        // rounded rate: the count per frame is then worked out per frame against a real period.
        //
        // Only at or above 60Hz. Below it there is no blank to put a sixtieth of a second on — a 50Hz
        // display would get one frame per blank and run the game at 50, seventeen percent slow with the
        // music to match. The clock at least keeps the game's own speed there and leaves the unevenness
        // to the display.
        //
        // A grid was the obvious alternative and is not wanted. A 50Hz display could take two blanks
        // for some frames and one for others, the way the fractional path does above 60, and land every
        // frame on a blank at an average of 1.2 — sixty frames a second on a 50Hz panel. What it buys
        // that with is judder by construction: five frames of every six shown for one refresh and the
        // sixth for two, every second of the run. A display under 60Hz is outside what orb paces at
        // all, so what it gets here is the game's own speed on a clock that lands anywhere, and not an
        // even speed traded for uneven frames.
        self.blank_paced =
            blanks > 0 || (measured.is_some() && hz.is_some_and(|hz| hz >= LOGIC_HZ));
        // A compose time shown to be short is shown so of the display it was measured on. The window
        // moving to another monitor, or the mode changing under it, makes it a claim about
        // something else — and since it only ever ratchets upward, carrying it over would hold
        // lag on the new display for a fault that belonged to the old one.
        //
        // Both swaps, and then the test: `||` would skip the second one and leave the period
        // unstored on every change of the blanks.
        let was_blanks = std::mem::replace(&mut self.blanks_per_frame, blanks);
        let was_period = std::mem::replace(&mut self.period, period);
        let blanks_changed = was_blanks != blanks;
        // Not the first adoption, which moves the period off zero and is the value being found
        // rather than moving. `configure` makes that one, from inside `DllMain`, where there is
        // nothing to count and nothing to have thrown away yet.
        if was_period != period && was_period != 0 {
            self.period_moves += 1;
        }
        // **Whether it is a different spacing and not whether it is a different number**, which is
        // the whole of what the reset can be asked. `qpcRefreshPeriod` is the compositor's own
        // measurement and it moves like one: measured over five minutes on a 120Hz panel, it moved
        // on half of every second's reading and 71% of those moves did not change even the
        // microsecond it rounds to — 8333µs to 8333µs, a handful of the counter's ticks apart, since
        // the counter runs at ten to the microsecond. Read as a different display, each of those
        // threw away an allowance of 2600µs to 3200µs that had been climbed to, and the only thing
        // that climbs it is a frame missing its blank: 65 resets in the run, every one of them
        // re-bought in stutters. So the same two per cent that `same_rate` holds two *rates* to,
        // held here to the spacings — a spacing is the rate upside down and the band reads the same
        // on either.
        //
        // It earns its place on a display whose rate is no whole multiple, and nowhere else: for one
        // that is, `whole_multiple`'s own tolerance is two per cent of the same nominal, so any move
        // wide enough to fail this fails that too and arrives as `blanks_changed`. A fractional
        // display has no such second answer — 100Hz to 110Hz is nine per cent with no blank count to
        // change — and that is the move this is here to catch.
        let respaced = was_period != 0 && !same_rate(was_period, period);
        if (blanks_changed || respaced) && self.pinned_compose_us == 0 {
            // What the reset costs, said where it happens, because the report's own numbers cannot
            // carry it: `compose_us` is written once per period and a reset can fall anywhere in
            // between, so a period's one reading of it is no account of what happened to it.
            //
            // Only where there was something to lose. A reset of a value already at
            // `COMPOSE_START_US` with nothing proven short costs nothing.
            if self.compose_us > COMPOSE_START_US || self.proven_short_us > 0 {
                self.compose_resets += 1;
                pacing!(
                    "frame: the compositor's blanks moved from {}us to {}us apart, {} to {} a frame; \
                     giving back the {}us found for it and the {}us proven short",
                    self.micros(was_period),
                    self.micros(period),
                    was_blanks,
                    blanks,
                    self.compose_us,
                    self.proven_short_us,
                );
            }
            self.proven_short_us = 0;
            self.compose_us = COMPOSE_START_US;
        }
    }

    /// What the pacing is costing and what it is buying, for the screen: how long a frame
    /// takes from reading the keyboard to being handed over, how long a frame stays up, and
    /// and how long the compositor is being given to draw. All measured, all microseconds.
    ///
    /// The compositor's time is worth a line of its own next to the lag rather than folded in. It is
    /// the part of the lag orb chose, it is the part that moves while the game is running, and
    /// it is the one that says whether the cadence is being held cheaply or expensively.
    pub fn status(&self) -> (i64, i64, i64) {
        (self.input_lag_us, self.interval_us, self.compose_us)
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
    pub fn wait_for_slot(&mut self, window: Hwnd) {
        // Measured rather than assumed cheap: `EnumDisplaySettingsW` and the compositor are
        // both asked, and this runs before the flush — so whatever it takes comes out of the
        // millisecond the frame has to reach that flush in.
        let age = self.settle_age;
        self.settle_age += 1;
        if age >= RESYNC_FRAMES {
            self.settle_age = 0;
            let asked = now();
            self.settle(window);
            let cost = self.micros(now() - asked);
            self.settle_us = cost;
            self.worst_settle = self.worst_settle.max(cost);
        } else {
            self.settle_us = 0;
        }

        let blanks = i64::from(self.blanks_per_frame);
        // The window being behind is not one of the reasons. It was: "counting refreshes against it
        // comes out wrong, and the clock will do until that is understood rather than guessed at" — and
        // measuring it says nothing about a window behind comes out wrong.
        // Measured, over every state a window can be in: in front, behind, covered by a full-screen
        // window and minimised all flush at the compositor's rate with every gap one refresh, `Present`
        // never answers `S_PRESENT_OCCLUDED`, and the lead a frame needs to make its blank is the same
        // whether anybody can see it or not. So a background frame is paced on the blanks like
        // any other, which is what `always_draw` — on by default — asked for all along.
        //
        // A replay being run fast keeps the cadence like anything else: `speed` is
        // updates per drawn frame, so the frames still come one per turn and only carry
        // more of the game with them.
        if !self.blank_paced {
            self.last_blank = 0;
            // So that a frame paced this way says so rather than reporting spans off the last
            // blank there was, which is somewhere in the past and belongs to another frame.
            self.flush_called = 0;
            self.aimed_refreshes = 0;
            return self.wait_by_clock(blanks);
        }
        let period = self.period.max(1);
        // What the frame already handed over was aimed at, which is the blank this flush is about
        // to wait out. Not this frame's own count: on a display that is not a whole multiple they
        // differ every other frame, and it is the one already in the compositor's hands that the
        // arrival and the overshoot are about.
        let borne = period * self.aimed_refreshes.max(1);

        // How much of the compositor's drawing time was still ahead when the frame got here,
        // which is the other way a refresh goes missing: that time is the compositor's, and a
        // frame that has spent it before even asking has none left to be composed in.
        //
        // Kept apart from the overshoot the compose time is driven by, because they are
        // different faults with different answers — this one is orb or the game's loop being
        // slow between frames, and giving the compositor longer would not touch it.
        let called = now();
        let last = self.last_blank;
        let arrival = if last == 0 {
            0
        } else {
            self.micros(called - (last + borne))
        };
        self.arrival_us = arrival;
        self.flush_called = called;
        if last != 0 {
            if arrival > 0 {
                self.late_arrivals += 1;
            }
            if arrival > self.micros(borne) {
                self.overrun_arrivals += 1;
            } else {
                self.worst_arrival = self.worst_arrival.max(arrival);
            }
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
        if !display::flush() {
            // The compositor has gone — turned off, or a session change. The clock will do
            // until `settle` notices.
            self.last_blank = 0;
            self.flush_called = 0;
            return self.wait_by_clock(blanks);
        }
        let blank = now();
        self.blank_at = blank;
        self.last_blank = blank;
        // Which blank the frame just handed over reached, which is the blank this flush has
        // this moment returned at, against the one it was aimed at. Everything the compose time is
        // driven by comes from those two numbers and neither costs a call.
        if last != 0 {
            self.measure_compose(period, last + borne, blank);
        }
        // This frame's own turn, worked out now that there is a blank to count it from. The order
        // matters: the count is the blank the grid means minus the blank in hand, so it cannot be
        // had before the flush returns.
        let refreshes = self.refreshes_this_frame(period, blank);
        let cadence = period * refreshes;
        self.aimed_refreshes = refreshes;
        // Whether the anchor is a blank at all, asked of the compositor rather than assumed
        // from the flush having returned. After the flush, so the asking is spent out of the
        // slack and not out of the compose time before it.
        self.anchor_us = if crate::log::pacing_wanted() {
            vblank().map_or(0, |vblank| self.micros(blank - vblank))
        } else {
            0
        };

        // The flush has just waited out the composition that put the last frame on screen, so
        // this is the far end of that frame's input lag and the near end was its own keyboard
        // read.
        let read_at = std::mem::replace(&mut self.input_read_at, 0);
        if read_at != 0 {
            let shown = self.micros(blank - read_at);
            let lag = self.input_lag_us;
            self.input_lag_us = if lag == 0 {
                shown
            } else {
                (lag * 7 + shown) / 8
            };
        }

        // Everything the run has said since the last frame goes to the log here, on the far
        // side of the flush. What is left of the turn is slack — the work needs the last
        // couple of milliseconds of it and nothing needs the rest — so a write that takes a
        // millisecond takes it out of nothing. On the near side of the flush the same write
        // costs a refresh.
        crate::log::drain();
        // And the numbers orb paints beside the game, which cost milliseconds of GDI for the same
        // couple of microseconds' worth of slack. Safe here for the reason the drain is: this runs on
        // the game's own frame, which is the thread that owns the window, and before the scene.
        unsafe { crate::window::paint_held() };

        // Then wait out almost all of the frame's turn, and do the work at the end of it.
        //
        // This is the whole of the input lag that is left to win. The work takes a fraction
        // of a refresh, so doing it at the start of the turn — where waiting for the blanks
        // leaves it — reads the keyboard a refresh and a half before the frame it appears
        // in. Doing it at the end reads the keyboard just before, which is as late as it can
        // be and still be that frame.
        self.wait_until(blank + cadence - self.ticks(self.prepare_us));
    }

    /// Which blank the frame just gone reached, and what that says the compose time should be.
    ///
    /// The frame reached the blank this flush returned at, so `blank` against the blank it was
    /// aimed at is the whole answer: nothing to average, and nothing to infer from a gap that
    /// goes wrong for half a dozen unrelated reasons. One frame decides it, because the two
    /// cases are a refresh apart while the aim is good to a few hundred microseconds.
    ///
    /// **A frame is a refresh late or it is not, and the half-refresh boundary is the whole of what
    /// says which.** Both moments are blanks the flush returned at, so the overshoot is a whole
    /// number of refreshes plus however differently the two returns were woken — and that wake is
    /// measured in hundreds of microseconds against a half-refresh of 2083µs at 240Hz, the shortest
    /// any rate anybody plays at has. A tighter window than half was tried on paper and is worse: a
    /// quarter-refresh at 240Hz is 1042µs, under the worst wake seen, so a real miss would fall
    /// outside it and go unanswered — which is a stutter left in place to avoid a step, the wrong way
    /// round. `orb-e2e`'s `pacing`'s `a_compositor_with_room_to_spare_never_moves_the_allowance` is
    /// the boundary held from the other side, over every rate.
    fn measure_compose(&mut self, period: i64, aimed: i64, blank: i64) {
        let overshoot = blank - aimed;
        self.overshoot_us = self.micros(overshoot);
        // A frame that landed where it was aimed is a blank worth measuring the next aim from, and
        // one that did not is not. Keeping the phase off the late ones is what stops a run settling
        // into a fifth of its frames arriving a refresh after the blank they asked for.
        if overshoot.abs() < period / 4 {
            self.phase = blank;
        }
        let refreshes = (overshoot.max(0) + period / 2) / period;
        self.missed[refreshes.clamp(0, self.missed.len() as i64 - 1) as usize] += 1;
        // Half a refresh either way is the boundary here as it is everywhere else, so the other side
        // of it is a frame the compositor took at a blank before the one it was aimed at.
        if overshoot < -period / 2 {
            self.early += 1;
        }

        if self.pinned_compose_us > 0 {
            return;
        }
        let compose = self.compose_us;
        // A step hands the frame over earlier and that is the whole of what it does, so more than one
        // refresh out is past anything a step could have reached: `compose_ceiling` holds the allowance
        // under a refresh — further ahead than that and the compositor takes the frame at the blank
        // *before* the one it was aimed at — so a composition longer than a refresh puts its frame two
        // refreshes past its aim at every allowance there is. A stage load is that case many times over,
        // which is what this branch was written for.
        //
        // **Not the whole turn**, which is the tempting way to say the same thing and is a different
        // number at every rate: a turn is one refresh at 60Hz and four at 240. At 60Hz the smallest miss
        // there is lands on that boundary and how late the two flushes either side of it were woken
        // decides which side — measured as 131 of 160 misses in a run with no load in it going uncharged,
        // 78 of them here where nothing counts them and 53 in the branch below. At 240Hz it is the other
        // way round: a composition of 10000µs costs its frame two refreshes, well inside the turn, and
        // bought a step for a frame no allowance could have saved. `orb-e2e`'s `pacing`'s
        // `every_frame_a_refresh_late_moves_the_allowance_one_step` and
        // `a_composition_longer_than_a_refresh_does_not_move_the_allowance` are those two.
        //
        // Without a guard here at all a run ratchets to seven milliseconds of lag by the third stage,
        // which is what the first attempt at driving this from our own gaps did.
        if refreshes > 1 {
            self.recovering = true;
            return;
        }
        // Taken rather than left set, so it excuses one frame and not every frame until one lands.
        //
        // Held as a state it deadlocks: a compositor slow enough that no frame lands is a compositor no
        // frame ever climbs for, since the line that cleared this sat past the return below and only a
        // landing frame reached it. Measured before this — 60Hz with the compositor at 3200µs, 20,000
        // frames — the rate sat at 30.00 with the allowance frozen at 2800µs and 10,121 of the misses
        // excused by a flag one frame had set and nothing since had cleared.
        //
        // What it is for is the frame after one that landed far off: the aim is measured from where the
        // last flush returned, and a flush called after its blank has gone returns at once rather than at
        // a blank, so the next frame is aimed off the grid and misses for that and not for the
        // compositor. Measured at 60, 120 and 144Hz: a quarter-second frame costs exactly one frame its
        // blank and the frame after that one lands.
        let after_a_frame_past_a_refresh = std::mem::take(&mut self.recovering);
        if refreshes > 0 {
            if after_a_frame_past_a_refresh {
                self.after_a_frame_past_a_refresh += 1;
                return;
            }
            // Nor is a frame whose own drawing outgrew its budget. It would have been late whatever
            // the compositor had been given, and climbing for it is how the climb ate the drawing's
            // allowance and then could not stop.
            if self.overran {
                self.overrun_drawing += 1;
                return;
            }
            self.proven_short_us = self.proven_short_us.max(compose);
            // `max` and not a plain assignment because the one thing the climb must never do is go
            // down, and saying so outright beats leaving it to whatever the ceiling works out as. It
            // never comes back down at all, which is the whole of why a run does not stutter.
            //
            // It used to: a clean stretch shaved it, on the reasoning that the least that works is
            // worth having since every microsecond of it is input lag. But the only thing that can
            // say a value is too little is a frame missing its blank at it, so every step downward is
            // a wager, and every lost wager is a stutter in the middle of a run. Measured: a walk
            // downward passes the real edge without dwelling anywhere long enough to catch a value that
            // only fails sometimes, and the miss that says so then sets a floor which says nothing about
            // that floor being enough — so it climbs back a stutter a step, for as long as the walk lasts.
            //
            // Climbing from below has none of that to do. It starts under anything a display has
            // wanted and rises until the misses stop, which at fifty microseconds a frame is over
            // before a title screen is up, and where it stops is the answer. Going down again could
            // only re-ask a question already answered.
            self.compose_us = self
                .compose_us
                .max((compose + MISS_STEP_US).min(self.compose_ceiling(period)));
            // And the budget with it, by however much the step really came to — nothing, where the
            // ceiling refused it. The budget is the drawing's time and the compositor's together, so a
            // step in the one is a step in the other: left where it was, the frame the step was taken for
            // is started exactly as late as the frame that missed and hands over exactly as near its
            // blank, so the step does nothing at all until `next_budget` catches up a frame later.
            //
            // It also read as the drawing having overrun. `account` measures the frame against the
            // allowance now in force while the budget was built from the one before it, so every frame
            // that followed a step came out over its budget by exactly the step — and the next miss was
            // then charged to a drawing that had not grown by a microsecond, which is a miss the ratchet
            // never heard about. Measured at 120Hz against a compositor wanting 4000µs: 10 of the climb's
            // 38 misses.
            self.prepare_us =
                (self.prepare_us + self.compose_us - compose).min(self.budget_ceiling());
        }
    }

    /// The most the compositor may be given: one refresh, less a quarter of it.
    ///
    /// A frame is handed over `compose` before the blank it is aimed at — that is the whole of what
    /// decides where the handover lands, since the drawing happens before it and only moves when
    /// the drawing starts. So this and not the budget is what has to stay inside a refresh: hand
    /// over earlier than the blank before the aimed one and the compositor takes it at that earlier
    /// blank, which shows the frame a refresh early and brings the flush back there with it,
    /// carrying the whole reckoning a refresh with it.
    ///
    /// At 120Hz a refresh is 8333µs and the old ceiling — half a game frame — was exactly that, so
    /// this was never visibly wrong there. At 144Hz a refresh is 6944 and half a game frame is
    /// 8333, and the moment the compositor's share passed a refresh the gaps collapsed to one
    /// apiece: `gaps in refreshes 1x418 2x179`, a hundred frames a second.
    ///
    /// **The fastest rate makes this the shortest, and there is no line here saying the climb was
    /// refused.** A display fast enough for [`COMPOSE_START_US`] to sit near this leaves the
    /// allowance with nowhere to go, and a log that only counts the misses reads the same as a
    /// compositor that is merely slow. What has kept that line from being worth writing is that
    /// nothing has been shown to reach the state: the allowance is a ratchet driven by missed
    /// blanks, so a run that was mispaced for any other reason inflates it directly, and a figure
    /// read off such a run is not a claim about what a compositor wanted. Anyone who suspects the
    /// ceiling has the pacing, whose `frame:` line carries the allowance beside the gaps.
    fn compose_ceiling(&self, period: i64) -> i64 {
        self.micros(period) * 3 / 4
    }

    /// The most the drawing and the compositor may be given between them, which is the input lag
    /// this pacing can cost.
    ///
    /// A whole game frame less a quarter, because the budget only decides how early the drawing
    /// starts and the drawing has the frame's own turn to happen in. So work heavy on every frame is
    /// covered up to most of a frame, which is what a budget is for —
    /// `orb-e2e`'s `pacing`'s `work_that_is_heavy_every_frame_is_covered` is that claim.
    ///
    /// Tying it to three quarters of a *refresh* was a mistake: at 144Hz that came out the same
    /// 5208µs as the compositor's share, so the drawing had no allowance at all, every frame
    /// reached the compositor late, and the input lag and the compositor's share read as the same
    /// number on screen — which is what gave it away, since the one is the other plus the drawing.
    ///
    /// See `next_budget`, and what happened when a 252ms frame was allowed to set this.
    ///
    /// **Nothing here keeps the handover off the blank before the one the frame is aimed at**, and
    /// nothing here can: how early a frame really goes is this less that frame's own work, and the work
    /// is not known until it is done. [`Self::hold_for_the_blank_before`] is where that is enforced.
    fn budget_ceiling(&self) -> i64 {
        self.micros(self.frame_ticks()) * 3 / 4
    }

    /// Holds the drawn frame until the blank before the one it is aimed at has gone, so that the
    /// compositor cannot take it there.
    ///
    /// **After the drawing, because that is the only place the question has an answer.** How early the
    /// frame is going is the budget less the work this frame actually did, and before the drawing
    /// finishes that number does not exist. The budget is a prediction tracked near the worst of the
    /// recent frames, so the frame after a spike does almost none of the work the budget was set from
    /// and goes nearly the whole of it early.
    ///
    /// What it is answering, measured on a real run: one heavy frame — a spell card starting, and its
    /// sounds with it — set the budget high enough that the frames after it, doing almost none of that
    /// work, were handed over further ahead than a refresh. The blank one refresh earlier was then nearer
    /// than the compositor's share, so it composed them *there*; the flush came back at that earlier
    /// blank, the anchor moved a refresh with it, and the next frame went as early again. The turns came
    /// out a refresh apart instead of two and the log said so:
    /// `shown a refresh or more early, so the game ran fast for them`.
    ///
    /// Bounding the budget was tried instead and is why this exists rather than that: held under a
    /// refresh, the most the budget could start a frame was a refresh before its blank, so work heavier
    /// than a refresh less the compositor's share could not be covered at all — the cadence held up to
    /// that point and broke to a third of the rate past it, from work a fraction of a game frame long. See
    /// `orb-e2e`'s `pacing`'s `work_that_is_heavy_every_frame_is_covered`, which is that headroom
    /// asserted, and `a_spike_the_ceiling_admits_does_not_hand_the_frames_after_it_over_early`, which
    /// is this.
    ///
    /// **The target is that earlier blank itself and not a margin before it.** A frame handed over at a
    /// blank cannot have been composed for it, the composing wanting time it did not have. What is left
    /// afterwards is a whole refresh in which to be composed for the aimed blank, and `compose_ceiling`
    /// already holds the share under three quarters of one, so this can never eat the compositor's own
    /// time however long it waits.
    ///
    /// Nothing to hold for where the frame is aimed at one refresh or none: the blank before it is the
    /// one already in hand, so there is nothing between them, and a frame the clock paced has no blank
    /// this could be measured from at all — `wait_for_slot` stores zeroes there so they say so.
    pub fn hold_for_the_blank_before(&mut self) {
        self.held_us = 0;
        if self.blank_at == 0 || self.aimed_refreshes < 2 {
            return;
        }
        let earlier = self.blank_at + self.period * (self.aimed_refreshes - 1);
        let left = earlier - now();
        if left <= 0 {
            return;
        }
        self.held_us = self.micros(left);
        // The same wait the frame's own deadline uses, spin and all, although nothing here needs that
        // precision: overshooting this blank costs nothing, there being a whole refresh behind it and
        // the compositor wanting at most three quarters of one. A second, cheaper wait for the sake of
        // the `SPIN_US` this burns is a slow path that ships to save microseconds on the few frames
        // that reach here at all — and a wait nothing else uses is a wait nothing else tests.
        self.wait_until(earlier);
    }

    /// How many refreshes to give the frame about to start.
    ///
    /// A rate that divides into 60 gets the same number every time and no grid to chase. That is
    /// deliberate: a display sold as 120Hz is often 119.88, and following an exact sixtieth there
    /// would mean a frame taking three refreshes every few minutes to make up the difference. Two
    /// every time is 59.94fps, a tenth of a percent slow on a clock nobody can see, against a
    /// hitch anybody can.
    ///
    /// A rate that does not divide takes the blank nearest where the sixtieth-of-a-second grid has
    /// got to. At 144Hz, where a frame is 2.4 refreshes, that comes out two, three, two, three,
    /// two — and the average is 2.4 exactly, so the rate is 60 over any length of time and every
    /// frame is still shown at a blank. The grid is an absolute moment rather than a running total
    /// of refreshes spent, which is what makes it self-correcting: a frame put on the nearer blank
    /// does not push the ones after it.
    fn refreshes_this_frame(&mut self, period: i64, blank: i64) -> i64 {
        let blanks = i64::from(self.blanks_per_frame);
        if blanks > 0 {
            return blanks;
        }
        self.grid_frames += 1;
        // Measured from the phase rather than from where the last frame landed, so that a frame
        // which landed late does not become the thing the next aim is built on. The count is then
        // whatever it takes to reach the blank the grid meant, counted from the blank in hand.
        let phase = self.phase;
        if phase == 0 || (blank - phase).abs() > self.frame_ticks() * 4 {
            self.phase = blank;
            self.ideal_next = blank + self.frame_ticks();
            return ((self.frame_ticks() + period / 2) / period).max(1);
        }
        let (refreshes, next) = grid_aim(period, self.frame_ticks(), phase, self.ideal_next, blank);
        self.ideal_next = next;
        refreshes
    }

    /// The fallback for a display whose refresh rate is not known at all.
    fn wait_by_clock(&mut self, blanks: i64) {
        self.clock_frames += 1;
        let period = self.period.max(1);
        let cadence = if blanks > 0 {
            period * blanks
        } else {
            self.frame_ticks()
        };

        let current = now();
        let mut target = self.next_present;
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
        self.next_present = target + cadence;
        // Here too, and for the same reason: the target is a moment on the clock, so a write
        // before the wait only shortens the wait. A frame paced this way still has a whole
        // turn of slack, and lines held for a drain that never comes are lines lost.
        crate::log::drain();
        // The paint too, and for that same last reason: a display paced this way is one the numbers
        // would otherwise never reach, there being no flush on this path to reach them on.
        unsafe { crate::window::paint_held() };
        self.wait_until(target);
    }

    /// Called once the frame has been handed over, to see what the display is getting.
    pub fn finished(&mut self, marks: Marks) {
        self.account(&marks);
        // Last of all, and unconditionally, because this is the near end of the span the next
        // frame has to reach the flush inside: everything after it belongs to the game's own
        // loop rather than to orb.
        self.accounted = now();
    }

    fn account(&mut self, marks: &Marks) {
        let previous = std::mem::replace(&mut self.last_present, marks.presented);
        let accounted = self.accounted;
        let cost = crate::log::spent();
        self.log_us[0] += cost.us;
        self.log_writes[0] += cost.writes;
        self.log_us[1] += cost.other_us;
        self.log_writes[1] += cost.other_writes;
        let period = self.period.max(1);
        // What the frame just handed over was aimed at, not what a display of this rate nominally
        // gets: on one that is not a whole multiple those differ every other frame.
        let aimed_refreshes = self.aimed_refreshes;

        // What this frame took from reading the keyboard to being handed over, which is what
        // the next one has to leave room for.
        let turn = self.budget_ceiling();
        // Left for the next frame's flush to close off, which is when this frame is on screen.
        self.input_read_at = marks.waited;
        // The hold is not work and must not be counted as any: it is time this frame spent waiting
        // *because* the budget was above what the frame needed, so a budget that grew to include it
        // would start the next frame earlier still, hold it longer, and grow again — the estimate
        // chasing its own wait until it reached the ceiling. See `hold_for_the_blank_before`.
        let took = self.micros((marks.presented - marks.waited) - (marks.held - marks.drawn))
            + self.compose_us;
        let prepare = self.prepare_us;
        // Whether this frame's drawing outgrew what it was started against. If it did, it reached
        // the compositor late for a reason the compositor cannot be given more time to fix, and the
        // next frame's reckoning must not read the resulting miss as one it can.
        self.overran = took > prepare;
        // The floor held under the ceiling rather than trusted to be: `clamp` panics when they
        // cross, and a pinned compose time larger than a refresh would cross them — inside the
        // frame loop, so the game would go down with it.
        let floor = self.compose_floor().min(turn);
        self.prepare_us = next_budget(prepare, took, turn).clamp(floor, turn);

        self.frames += 1;
        if previous == 0 {
            return;
        }
        let gap = marks.presented - previous;
        let refreshes = (gap + period / 2) / period;
        let bucket = refreshes.clamp(0, self.gaps.len() as i64 - 1) as usize;
        self.gaps[bucket] += 1;
        self.worst_gap = self.worst_gap.max(self.micros(gap));
        self.best_gap = self.best_gap.min(self.micros(gap));
        // Against the cadence the frame was aimed at rather than against the average, so a period
        // that is evenly wrong and one that is unevenly right do not come out the same.
        //
        // The clock's cadence where there are no blanks to count, not skipped: a display that is
        // not a multiple of 60 is paced entirely by the spacing of these presents, so it is the
        // one path where their spacing is the whole of what orb controls. Measuring it only where
        // the blanks do the work left the other case with nothing to look at.
        let aimed = if aimed_refreshes > 0 {
            period * aimed_refreshes
        } else {
            self.frame_ticks()
        };
        let off = self.micros(gap - aimed);
        let band = off.div_euclid(JITTER_BAND_US) + (self.jitter.len() as i64) / 2;
        self.jitter[band.clamp(0, self.jitter.len() as i64 - 1) as usize] += 1;

        // Smoothed over 32 frames, and the status line then holds each value for 30 more — so the rate
        // on screen is what a run settled at and cannot show the second that lost four frames. That is
        // what it is for, being read while playing rather than afterwards, and it is why the pacing's
        // own account is the log's: every frame that missed the cadence is written there with where its
        // time went, and nothing on screen is a substitute for reading it.
        let interval = self.interval_us;
        let smoothed = if interval == 0 {
            self.micros(gap)
        } else {
            (interval * 31 + self.micros(gap)) / 32
        };
        self.interval_us = smoothed;

        // How long the compositor needs is not judged here. It was, from
        // `DWM_TIMING_INFO.cFramesLate` — its own count of frames it could not show when they
        // were meant to be shown — and that count stayed at zero through runs where three to
        // five frames of every six hundred were a refresh late, so the value never moved off
        // its floor. It is driven from the flush's own overshoot instead, in `measure_compose`,
        // which is the same question asked of something that answers.

        // The breakdown, for the frames that did not come out on the cadence. Rationed,
        // because a bad patch would otherwise fill the log with the same line.
        if on_cadence(gap, aimed, period) {
            return;
        }
        let spoken = self.spoken;
        self.spoken += 1;
        if spoken >= LATE_LINES {
            self.unspoken += 1;
            return;
        }
        let us = |from: i64, to: i64| self.micros(to - from);
        let called = self.flush_called;
        // A frame is late for one of two reasons, and which is which is the first thing to
        // know. Either it reached the flush after the blank that was its turn had gone — the
        // refresh was lost before the frame did anything, and the spans up to `flush` say what
        // spent it — or it arrived in time and something after that overran, which the spans
        // from `wait` on say.
        let (how, waiting) = if called == 0 {
            (
                "paced by the clock".to_string(),
                format!("wait {}us", us(marks.cleared, marks.waited)),
            )
        } else {
            let arrival = self.arrival_us;
            let blank = self.blank_at;
            (
                if arrival > 0 {
                    format!("reached the flush {arrival}us after the blank it was aiming at")
                } else {
                    format!(
                        "reached the flush {}us before the blank it was aiming at",
                        -arrival
                    )
                },
                format!(
                    "pace {}us of which settle {}us, flush {}us to an anchor {}us after the compositor's blank, wait {}us (the frame before reached the screen {}us off its own blank)",
                    us(marks.cleared, called),
                    self.settle_us,
                    us(called, blank),
                    self.anchor_us,
                    us(blank, marks.waited),
                    self.overshoot_us,
                ),
            )
        };
        // The whole gap, from the last frame being handed over to this one, in spans that add
        // up to it. The first two are outside the marks and were what had to be guessed at:
        // what orb did after handing the last frame over, and what the game's own loop did
        // before calling this one back. Between them they are the millisecond a frame has to
        // reach the flush in.
        pacing!(
            "frame: {refreshes} refreshes, {}us — {how}; after present {}us, loop {}us, clear {}us, {waiting}, update {}us, sound {}us, draw {}us, hold {}us, present {}us; log {}us in {} writes here, {}us in {} elsewhere",
            self.micros(gap),
            us(previous, accounted),
            us(accounted, marks.started),
            us(marks.started, marks.cleared),
            us(marks.waited, marks.updated),
            us(marks.updated, marks.sounded),
            us(marks.sounded, marks.drawn),
            us(marks.drawn, marks.held),
            us(marks.held, marks.presented),
            cost.us,
            cost.writes,
            cost.other_us,
            cost.other_writes,
        );
    }

    /// How the pacing is doing, for the log. With the blanks pacing, every gap should be
    /// in one bucket; anything else means frames are reaching the display unevenly.
    pub fn report(&mut self) -> String {
        let frames = std::mem::replace(&mut self.frames, 0);
        let mut gaps = String::new();
        for (refreshes, count) in std::mem::take(&mut self.gaps).iter().enumerate() {
            let count = *count;
            if count > 0 {
                let more = if refreshes == self.gaps.len() - 1 {
                    "+"
                } else {
                    ""
                };
                gaps.push_str(&format!(" {refreshes}{more}x{count}"));
            }
        }
        // The compositor's own tally of frames it could not show when they were meant to be
        // shown. Our own gaps cannot see those: they measure when the frame was handed over,
        // not when it reached the screen. This is what says whether handing it over as late
        // as we do is late enough.
        let late = match frames_late() {
            Some(total) => {
                let previous = std::mem::replace(&mut self.was_late, total);
                if previous < 0 { 0 } else { total - previous }
            }
            None => 0,
        };
        // The lag split into the two drawing times it is made of, because they answer to
        // different causes: the game's grows with what it is drawing and is nobody's fault,
        // while the compositor's is a property of the display and should not move once found.
        let compose = self.compose_us;
        let prepare = self.prepare_us;
        format!(
            "frame: {frames} frames, {}us apart, {prepare}us before the blank ({}us to draw + {compose}us for the compositor), {late} shown late, gaps in refreshes{gaps}",
            self.interval_us,
            (prepare - compose).max(0),
        )
    }

    /// The worst of the period rather than the shape of it, for the things a rate cannot
    /// show: an average interval hides one frame in six hundred, which is exactly the kind
    /// that gets noticed while playing.
    ///
    /// Written where the pacing is, so this is also where the ration on the per-frame lines
    /// is given back — `report` is not written at all at `quiet`, which is the level a sweep
    /// is read at.
    pub fn worst(&mut self) -> String {
        let arrival = std::mem::replace(&mut self.worst_arrival, i64::MIN);
        let unspoken = std::mem::replace(&mut self.unspoken, 0);
        self.spoken = 0;
        // The spread of the gaps, which is what the refresh buckets round away and what judder
        // actually is: bands of half a millisecond off the cadence, counted from four bands below.
        let mut spread = String::new();
        for (band, count) in std::mem::take(&mut self.jitter).iter().enumerate() {
            let count = *count;
            if count > 0 {
                let low = (band as i64 - (self.jitter.len() as i64) / 2) * JITTER_BAND_US;
                let edge = if band == 0 {
                    format!("under {}", low + JITTER_BAND_US)
                } else if band == self.jitter.len() - 1 {
                    format!("over {low}")
                } else {
                    format!("{low}")
                };
                spread.push_str(&format!(" {edge}us:{count}"));
            }
        }
        let best = std::mem::replace(&mut self.best_gap, i64::MAX);
        let mut line = format!(
            "frame: on screen from {}us to {}us, off the cadence by{spread}; arrival at worst {}, {} past it, {} of those beyond a whole turn; settle at worst {}us",
            if best == i64::MAX { 0 } else { best },
            std::mem::replace(&mut self.worst_gap, 0),
            if arrival == i64::MIN {
                "unmeasured".to_string()
            } else {
                format!("{arrival:+}us against the blank")
            },
            std::mem::replace(&mut self.late_arrivals, 0),
            std::mem::replace(&mut self.overrun_arrivals, 0),
            std::mem::replace(&mut self.worst_settle, 0),
        );
        // What the log itself cost. It is the one thing in here that is only there because
        // something is being looked into, so a run whose late frames line up with its writes
        // is being told about its own instrument and not about the pacing.
        line += &format!(
            "; log {}us in {} writes on the frame's thread, {}us in {} on orb's own",
            std::mem::take(&mut self.log_us[0]),
            std::mem::take(&mut self.log_writes[0]),
            std::mem::take(&mut self.log_us[1]),
            std::mem::take(&mut self.log_writes[1]),
        );
        if unspoken > 0 {
            line += &format!("; {unspoken} late frame(s) not written, the ration was full");
        }
        line
    }

    /// Which blank the frames actually reached, and what the compositor's own counters did
    /// while they did — the two halves of the same question, which is what the compose time
    /// has to answer to.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    pub fn shown(&mut self) -> String {
        let mut blanks = String::new();
        for (refreshes, count) in std::mem::take(&mut self.missed).iter().enumerate() {
            let count = *count;
            if count > 0 {
                let more = if refreshes == self.missed.len() - 1 {
                    "+"
                } else {
                    ""
                };
                blanks.push_str(&format!(" {refreshes}{more}x{count}"));
            }
        }
        if blanks.is_empty() {
            blanks.push_str(" none");
        }
        let clock = std::mem::replace(&mut self.clock_frames, 0);
        let grid = std::mem::replace(&mut self.grid_frames, 0);
        let short = self.proven_short_us;
        // How often the spacing moved under the allowance, beside the allowance it moved. Said even
        // at zero: a period that held still is the answer to whether the display is the reason the
        // frames are uneven, and a clause that only appears when it did not hold still cannot give
        // it.
        let moved = match (
            std::mem::replace(&mut self.period_moves, 0),
            std::mem::replace(&mut self.compose_resets, 0),
        ) {
            (0, _) => "; the blanks kept their spacing".to_string(),
            (moves, 0) => format!("; the blanks changed spacing {moves} time(s), costing nothing"),
            (moves, resets) => format!(
                "; the blanks changed spacing {moves} time(s), {resets} of which gave back what had been found"
            ),
        };
        // Frames the grid counted, which are two refreshes and three by design. Only where there
        // were any: on a display that divides into 60 there never are, and a nought in the line
        // every period would be a number nobody has a question about.
        let grid = match grid {
            0 => String::new(),
            grid => format!(
                "; {grid} frame(s) counted off the sixtieth's grid, which take two refreshes and three by design"
            ),
        };
        let excused = match (
            std::mem::replace(&mut self.after_a_frame_past_a_refresh, 0),
            std::mem::replace(&mut self.overrun_drawing, 0),
        ) {
            (0, 0) => String::new(),
            (after, 0) => format!(", {after} of them the frame after one more than a refresh out"),
            (0, drawing) => format!(", {drawing} of them a frame whose drawing overran"),
            (after, drawing) => {
                format!(
                    ", {after} after a frame more than a refresh out and {drawing} whose drawing overran, neither counted against the compositor"
                )
            }
        };
        let early = match std::mem::replace(&mut self.early, 0) {
            0 => String::new(),
            early => {
                format!("; {early} shown a refresh or more early, so the game ran fast for them")
            }
        };
        format!(
            "frame: refreshes past the blank aimed at{blanks}{excused}{early}; the compositor gets {}us{}, never shaved below {}us{}{moved}; {clock} frame(s) paced by the clock, which have no blank to have missed{grid}",
            self.compose_us,
            if self.pinned_compose_us > 0 {
                " pinned"
            } else {
                " found"
            },
            self.compose_floor(),
            if short > 0 {
                format!(" since a frame missed its blank at {short}us")
            } else {
                String::new()
            },
        )
    }

    /// Waits most of the way there and spins the rest, because the wait overshoots by up to
    /// [`SPIN_US`] and the last stretch is the one that decides whether the frame makes its slot.
    fn wait_until(&mut self, deadline: i64) {
        loop {
            let left = deadline - now();
            if left <= 0 {
                return;
            }
            if self.micros(left) > SPIN_US {
                // In the counter's own ticks, so the wait is aimed exactly where the spin picks it
                // up. The call this replaced took whole milliseconds and rounded them down, which
                // handed the spin up to a millisecond more than it was asked to cover — margin that
                // was never counted on and that a wait of an exact number of milliseconds did not
                // have at all.
                if !clock::wait(left - self.ticks(SPIN_US)) {
                    return self.no_timer();
                }
            } else {
                // `pause`, and nothing given up. Yielding here instead — `SwitchToThread` or
                // `Sleep(0)`, so that the sound and the rest of the system get the core back for the
                // last stretch — is the obvious improvement, and it was measured and rejected. On an
                // idle machine it saves nothing, because a yield with nobody waiting returns at once
                // and the loop goes round as often. With every core loaded it made the rest of the
                // system worse off rather than better, and it cost the landings: frames that had been
                // arriving just past the deadline began missing whole refreshes.
                clock::spin_once();
            }
        }
    }

    /// Says the host cannot make the timer the waits are made on, and ends the launch.
    ///
    /// There is no second wait to fall back to and that is deliberate: `Sleep` kept as a spare would
    /// be a slow path that ships, and a launch that paced badly while its log said it was pacing
    /// well is what the whole of this file is written against. So a host that cannot do this is a
    /// host orb does not run on, and it is told so where somebody will read it — see
    /// [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md).
    ///
    /// From here rather than from [`configure`](Self::configure) because this runs on the frame
    /// hook's thread, which is the game's main one with a window and a message pump. `configure`
    /// runs inside `DllMain`.
    ///
    /// The launcher asks the same question before it injects anything, so the ordinary way an
    /// unsupported host is turned away is with the game never started. This is for the case the
    /// launcher was not the way in.
    ///
    /// **No e2e test enters it**, which is what that makes it: a host with no high-resolution timer is one
    /// no game runs on, so what drives this is a test with no game in it — `orb-sim`'s `pacing_no_timer`,
    /// which reaches it through the timer that will not be made rather than by name, and whose own header
    /// says the dialog and the exit being behind the seam is what makes such a test writable at all.
    fn no_timer(&self) {
        log!(
            "frame: the host cannot create a high-resolution timer, which every wait is made on; stopping"
        );
        window::message_box(NO_TIMER_TITLE, NO_TIMER_TEXT);
        process::exit(NO_TIMER_CODE);
    }
}

pub fn now() -> i64 {
    clock::counter()
}

/// The multiple of 60 a reported rate really is, when it is one.
///
/// `dmDisplayFrequency` is whole Hz, so the NTSC-derived rates come back short: 119.88 reports
/// as 119 and 59.94 as 59. Those are multiples of 60 in every sense that matters to a frame
/// loop and not one in arithmetic, and taking them for fractional rates is worse than useless —
/// the grid then chases an exact sixtieth against a rate 0.8% away from one and pays the
/// difference in a one-refresh frame about once a second.
///
/// Two per cent, which reaches 119 from 120 and 59 from 60 while leaving 165 to be what it is:
/// the nearest multiple to that is 180, eight per cent off.
fn whole_multiple(hz: u32) -> Option<u32> {
    let multiple = (hz + LOGIC_HZ / 2) / LOGIC_HZ;
    let nominal = multiple * LOGIC_HZ;
    (multiple >= 1 && nominal.abs_diff(hz) * 100 <= nominal * 2).then_some(multiple)
}

/// Whether two rates named by different means are the same display's.
///
/// Within two per cent, not equal. One comes from `dmDisplayFrequency` in whole Hz and the other
/// from the compositor's refresh period in whole microseconds, so a 119.88Hz display is 119 by
/// one and 120 by the other and they never were going to match. What this is for is catching the
/// compositor timing an altogether different monitor — 144 against 120, which ran the game at 72
/// frames a second — and that is not a rounding's worth apart.
fn same_rate(a: i64, b: i64) -> bool {
    (a - b).abs() * 100 <= a.max(b) * 2
}

/// The refresh rate of the monitor the game's window is on, in whole Hz.
///
/// Asked of that monitor rather than of the desktop, because a second monitor at a
/// different rate is not this game's business.
/// How many frames the compositor could not show at the refresh they were aimed at.
/// `None` when it will not say.
fn frames_late() -> Option<i64> {
    display::composition().map(|it| it.frames_late)
}

/// When the compositor says the last blank was. `None` when it will not say.
///
/// Only for checking the anchor the pacing keeps, which is when `DwmFlush` came back and
/// is assumed to be a blank. A flush that returns a refresh late and a flush that returns
/// on time against an anchor which was itself a refresh early produce the same gap, and
/// the compositor's own timestamp is what tells them apart. Asked for only while the pacing
/// is being written, since nothing but that question needs it.
fn vblank() -> Option<i64> {
    display::composition()
        .filter(|it| it.vblank != 0)
        .map(|it| it.vblank)
}

/// What the compositor says about the blanks: how far apart they are in ticks, and
/// which one was the last, counted from when it started. `None` when it will not say.
fn composition() -> Option<(i64, i64)> {
    display::composition()
        .filter(|it| it.refresh_period != 0)
        .map(|it| (it.refresh_period, it.refresh))
}

/// What to start the next frame's work against, from what this one's took.
///
/// Rises at once and falls slowly, so it sits near the worst of the recent frames rather than
/// their average: aiming at the average means missing the handover on every frame heavier than
/// it, and one frame handed over late is shown a whole refresh late — far more visible than the
/// microseconds of lag that aiming high costs.
///
/// A frame that wanted more than the whole budget is left out of that, because it is a scene
/// being built rather than a heavy frame. `RunCalcChain` takes 252ms where a run ends and the
/// next scene is made, and nothing about that says what the frame after it will take. Believed,
/// it pinned the budget to the ceiling — and the frames that followed, two milliseconds of work
/// apiece, were then started 12.5ms before a blank 8.3ms away. Handed over that early they were
/// composed for the blank *before* the one they were aimed at, and `DwmFlush` returned there with
/// them, so the anchor everything is measured from moved a refresh early and the next frame
/// handed over just as early again. One frame per refresh, which is one update per refresh, for
/// the thirty frames the budget took to decay back: the game at double speed for a third of a
/// second after every stage load, and every counter reading clean while it happened.
fn next_budget(prepare: i64, took: i64, ceiling: i64) -> i64 {
    if took > ceiling {
        return prepare;
    }
    if took > prepare {
        return took;
    }
    prepare - (prepare - took) / 64
}

/// The blank on the phase's grid nearest where the sixtieth-of-a-second grid has got to, in
/// refreshes from the blank in hand, and where the grid stands once this frame has had its turn.
///
/// Both grids are absolute, so neither carries an error forward: the answer is the same whatever
/// the last frame did.
///
/// A grid moment the blank in hand has already passed is a frame that has been missed, and it is
/// dropped rather than caught up. Left where it was, the count comes out at one refresh and stays
/// there until the debt is spent — and one refresh a frame is one update per refresh, so the debt
/// is spent running the game at double speed. Nothing is won by that: the frames that were missed
/// are gone either way, and what the grid is for is where the *next* one goes.
fn grid_aim(period: i64, frame: i64, phase: i64, ideal: i64, blank: i64) -> (i64, i64) {
    let ideal = if ideal <= blank { blank + frame } else { ideal };
    let steps = ((ideal - phase) + period / 2) / period;
    let target = phase + steps * period;
    // Counted from the blank in hand rather than from the phase, since that is where the wait
    // starts. At least one, because a target already behind us is one this frame cannot have.
    (
        (((target - blank) + period / 2) / period).max(1),
        ideal + frame,
    )
}

/// Whether a frame's gap came out on the cadence it was aimed at, both read as a number of the
/// display's refreshes.
///
/// Asked of the cadence and not of `AIMED_REFRESHES`: a frame paced by the clock counted no
/// refreshes of its own, and `wait_for_slot` stores a zero there so that it says so — so holding
/// that zero against a count threw every one of those frames out of the pacing log before the
/// line was formatted, which is the run whose cadence there is a question about. For a frame that
/// did count them, the two are the same test.
fn on_cadence(gap: i64, aimed: i64, period: i64) -> bool {
    let refreshes = |ticks: i64| (ticks + period / 2) / period;
    refreshes(gap) == refreshes(aimed)
}

#[cfg(test)]
mod tests {
    use super::{
        COMPOSE_FLOOR_US, LOGIC_HZ, Pacing, grid_aim, next_budget, on_cadence, same_rate,
        whole_multiple,
    };

    /// A pacing whose counter ticks once a microsecond, so that every number below reads as the
    /// microseconds it stands for. The real one runs at ten ticks to the microsecond; nothing here
    /// turns on which, and this way the arithmetic is legible.
    fn pacing() -> Pacing {
        Pacing::at(1_000_000)
    }

    /// A 144Hz refresh and a sixtieth of a second, in microseconds standing in for the
    /// performance counter's ticks. 2.4 refreshes to the frame, so the count has to vary.
    const REFRESH: i64 = 6944;
    const FRAME: i64 = 16_666;
    /// Three quarters of a frame, which is `budget_ceiling`.
    const CEILING: i64 = 12_500;

    /// The rates a display actually reports itself as, and which of them a frame loop should
    /// treat as a whole number of refreshes to the game's sixtieth.
    ///
    /// The NTSC-derived ones are the point: `dmDisplayFrequency` is whole Hz, so 119.88Hz says
    /// 119 and 59.94Hz says 59, and reading those as fractional rates is how a display that is
    /// two refreshes a frame in every practical sense ends up chasing a sixtieth it cannot hold.
    #[test]
    fn a_rate_that_reports_short_is_still_a_multiple() {
        assert_eq!(whole_multiple(60), Some(1));
        assert_eq!(whole_multiple(59), Some(1), "59.94Hz");
        assert_eq!(whole_multiple(120), Some(2));
        assert_eq!(whole_multiple(119), Some(2), "119.88Hz");
        assert_eq!(whole_multiple(239), Some(4), "239.76Hz");
    }

    /// A frame paced by the clock is held to its cadence like any other, so that the run whose
    /// cadence there is a question about is the one the pacing log has lines for.
    #[test]
    fn a_frame_paced_by_the_clock_is_still_held_to_its_cadence() {
        // 50Hz, which is no whole multiple of the game's sixtieth: nothing counts refreshes,
        // and what the wait aims at is a sixtieth on the clock.
        const FIFTY: i64 = 20_000;
        assert!(on_cadence(FRAME, FRAME, FIFTY));
        assert!(on_cadence(FRAME + 2_000, FRAME, FIFTY), "within a refresh");
        assert!(!on_cadence(FRAME * 2, FRAME, FIFTY), "a whole frame late");

        // And a frame that did count them: two refreshes of 144Hz aimed at, three taken.
        assert!(on_cadence(REFRESH * 2, REFRESH * 2, REFRESH));
        assert!(!on_cadence(REFRESH * 3, REFRESH * 2, REFRESH));
    }

    /// And the ones that are genuinely not, which have to stay that way or the frames get put on
    /// a cadence the display cannot show them at.
    #[test]
    fn a_rate_that_is_not_a_multiple_is_left_alone() {
        for hz in [75, 100, 144, 143, 165] {
            assert_eq!(whole_multiple(hz), None, "{hz}Hz");
        }
    }

    /// Two ways of naming one display's rate round differently and must still agree; two
    /// different displays must not. The second is what the check is for: 144 read against a
    /// 120Hz display ran the game at 72 frames a second.
    #[test]
    fn rates_a_rounding_apart_are_the_same_display() {
        assert!(same_rate(119, 120), "one rounds down, the other up");
        assert!(same_rate(60, 59));
        assert!(!same_rate(144, 120));
        assert!(!same_rate(60, 120));
    }

    /// What a display of 2.4 refreshes a frame gets: two refreshes and three in whatever order
    /// keeps the average at 2.4, so the rate is exactly 60 over any length of time.
    #[test]
    fn a_fractional_rate_takes_two_refreshes_and_three() {
        let (mut phase, mut ideal, mut blank) = (0, FRAME, 0);
        let mut counts = Vec::new();
        for _ in 0..10 {
            let (refreshes, next) = grid_aim(REFRESH, FRAME, phase, ideal, blank);
            counts.push(refreshes);
            ideal = next;
            // The frame lands where it was aimed, which is what moves the phase on.
            blank += refreshes * REFRESH;
            phase = blank;
        }
        assert!(
            counts.iter().all(|refreshes| (2..=3).contains(refreshes)),
            "{counts:?}"
        );
        assert_eq!(
            counts.iter().sum::<i64>(),
            24,
            "2.4 refreshes a frame over ten frames: {counts:?}"
        );
    }

    /// And what it gets after a stall in the game's own update has left the grid behind: the
    /// frames that were missed are dropped. Given to the frames after it instead, the missed time
    /// buys one refresh a frame — an update per refresh, which is the game at double speed.
    #[test]
    fn a_grid_left_behind_drops_the_frames_it_missed() {
        let stalled = 50_000;
        let (refreshes, next) = grid_aim(REFRESH, FRAME, 0, FRAME, stalled);
        assert!(
            refreshes >= 2,
            "{refreshes} refresh(es) is the game run fast"
        );
        assert!(
            next > stalled + FRAME,
            "the grid starts again from the blank in hand"
        );
    }

    /// The budget is what the next frame's work will be started against, and the 252ms
    /// `RunCalcChain` that builds the next scene predicts nothing about it. Believed, it pins the
    /// budget to the ceiling, from where the frames after it are handed over more than a refresh
    /// before the blank they are aimed at and shown at the blank before it.
    #[test]
    fn a_frame_that_ran_past_the_whole_budget_does_not_set_it() {
        assert_eq!(next_budget(3700, 254_800, CEILING), 3700);
    }

    /// A heavier frame is the thing the budget must not be caught out by, since a frame handed
    /// over late is shown a whole refresh late, so it is believed at once.
    #[test]
    fn a_heavier_frame_raises_the_budget_at_once() {
        assert_eq!(next_budget(3700, 5000, CEILING), 5000);
    }

    /// A lighter one only says the budget may be higher than it needs to be, which is lag rather
    /// than a lost refresh, so it comes down slowly enough to stay near the worst of the frames
    /// around it.
    #[test]
    fn a_lighter_frame_brings_the_budget_down_slowly() {
        let next = next_budget(3700, 2900, CEILING);
        assert!((2900..3700).contains(&next), "{next}");
        assert!(3700 - next < (3700 - 2900) / 8, "{next} is most of the way");
    }

    /// The whole budget is the game's own frame's, and does not shrink as the display gets faster.
    ///
    /// Which is the fix, and the thing that broke: tied to a *refresh* instead, at 144Hz it came out
    /// the same 5208µs as the compositor's share, so the drawing had no allowance inside it, every
    /// frame reached the compositor late, and the input lag and the compositor's share read as the
    /// same number on screen — which is what gave it away, the one being the other plus the drawing.
    /// See `budget_ceiling`.
    #[test]
    fn the_whole_budget_is_the_games_frame_and_not_the_displays() {
        let pacing = pacing();
        let whole = pacing.budget_ceiling();
        // The rates a display reports itself as, over the range anyone plays at.
        for hz in [60u32, 75, 90, 100, 120, 144, 165, 240, 360] {
            let period = pacing.ticks(1_000_000 / i64::from(hz));
            assert_eq!(
                pacing.budget_ceiling(),
                whole,
                "{hz}Hz moved the whole budget"
            );
            // And a display faster than the game leaves the drawing an allowance inside it.
            if hz > LOGIC_HZ {
                assert!(
                    pacing.compose_ceiling(period) < whole,
                    "{hz}Hz leaves the drawing nothing",
                );
            }
        }
    }

    /// The compositor's share is three quarters of a *refresh*, since what it is being given is time
    /// inside the refresh the frame is aimed at. A faster display gives it less, which is the whole
    /// reason it is not a constant.
    #[test]
    fn the_compositors_share_is_measured_against_a_refresh() {
        let pacing = pacing();
        let refresh = pacing.ticks(REFRESH);
        assert_eq!(
            pacing.compose_ceiling(refresh),
            pacing.micros(refresh) * 3 / 4
        );
        assert!(pacing.compose_ceiling(refresh / 2) < pacing.compose_ceiling(refresh));
    }

    /// A pinned drawing time is the whole of the answer, floor and all: pinning is for measuring one
    /// value, and a floor that overrode it would make every reading below the floor the same
    /// reading — which is exactly the range a sweep starts in.
    #[test]
    fn a_pinned_drawing_time_is_not_raised_to_the_floor() {
        let mut pacing = pacing();
        let under = COMPOSE_FLOOR_US / 2;
        pacing.proven_short_us = COMPOSE_FLOOR_US * 4;
        pacing.pinned_compose_us = under;
        assert_eq!(pacing.compose_floor(), under);

        // Unpinned, the floor is the greater of the constant and what a run has shown to be short.
        pacing.pinned_compose_us = 0;
        assert_eq!(pacing.compose_floor(), COMPOSE_FLOOR_US * 4);
        pacing.proven_short_us = 0;
        assert_eq!(pacing.compose_floor(), COMPOSE_FLOOR_US);
    }

    /// A rate that divides into 60 is told how many refreshes a frame gets and never chases a grid:
    /// the same number every time, whatever the blank in hand is.
    ///
    /// Which is the deliberate part — a display sold as 120Hz is often 119.88, and following an
    /// exact sixtieth there would spend a third refresh every few minutes to make up the difference.
    #[test]
    fn a_rate_that_divides_gets_the_same_count_every_frame() {
        // No putting it back afterwards: this pacing is this test's own, where the static it
        // replaced was the whole process's and had to be swept up or the next test read it.
        let mut pacing = pacing();
        pacing.blanks_per_frame = 2;
        let refresh = pacing.ticks(REFRESH);
        let counts: Vec<i64> = [0, 1, 7, 12_345, -9]
            .into_iter()
            .map(|blank| pacing.refreshes_this_frame(refresh, blank))
            .collect();
        assert_eq!(counts, [2, 2, 2, 2, 2]);
    }
}
