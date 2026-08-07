//! A display and a compositor a test declares.
//!
//! The blanks are a grid: `origin + n × period`. A frame handed over at `t` is composed by
//! `t + compose`, and the blank it reaches is the first one at or after that — which is what
//! `DwmFlush` returns at. Everything the pacing decides about whether a frame made its blank comes
//! out of those two numbers, so this is where a test sets them.
//!
//! Two things here are deliberately as unhelpful as the real compositor, because the code comments
//! in `frame.rs` record measuring them that way and a kinder simulation would let a test come to
//! rely on something false:
//!
//! - `cRefresh` counts *compositions*, one per frame handed over, and not refreshes of the display.
//!   A run where eight flushes waited out eight refreshes had it advance eight times, once a frame.
//! - `cFramesLate` stays at zero, through a broken cadence as much as an even one. The one test that
//!   may move it off zero is the one asserting nothing is judged on it — see
//!   [`says_every_composition_was_late`](Display::says_every_composition_was_late).
//!
//! # What this does not model, and what rests on measurement
//!
//! **One grid, not two.** A desktop where the compositor times one monitor and the game's window is
//! on another has two: the compositor's compositions, and the blanks of the panel the pixels appear
//! on. Only the first is here. That is enough for what the scenarios measure — the frame *rate* is
//! decided by when `DwmFlush` returns, which is the compositor's grid — and it is why nothing here
//! can say anything about how those frames land on the other panel.
//!
//! **The split case is measured, not assumed** — `scripts/compositor-probe.c` on a desktop of a 120Hz
//! primary, a second 120Hz monitor and a 144Hz one beside them. The compositor reported
//! `qpcRefreshPeriod=6944.4us` — 144.00Hz, the fastest monitor's rate — and 119 flushes apiece put a
//! window on each of the three at 143.97Hz, means of 6946.0, 6946.1 and 6945.5µs, while
//! `EnumDisplaySettingsW` answered 120, 144 and 120 about their own. So both of the things this simulates
//! hold: the flushes fall at the compositor's rate, and the window's monitor changes nothing about
//! them.
//!
//! **The wake delay is modelled, from its measured distribution** — see [`USUAL_US`]. Which makes the
//! simulator non-deterministic, on purpose: a host that woke a thread the instant a blank came is a
//! host nobody has, and a scenario that only holds against one is a scenario about arithmetic. A
//! scenario that fails for some seeds and not others has found something a real machine can do, and
//! the seed is in the assertion so the run can be replayed.
//!
//! **The compose time is a distribution too, and it is inferred rather than read** — see [`Compose`].
//! Nothing on the machine reports what the compositor takes over a frame; orb knows only what it
//! *allows* it, which is its own estimate and climbs on a miss. So [`Compose::measured`] is pinned
//! from three sides — the value that reproduces the real 120Hz run's `600 frames, 16650us apart, gaps
//! in refreshes 2x600`, the 3.5ms a session was
//! seen reaching, and how rare half an hour of play makes that — and remains the number to vary
//! rather than the number to trust. A scenario asking what the ratchet does sets its own rate, since
//! at the measured rarity a scenario sees no spike at all.

use std::sync::Mutex;

use orb_api::{Composition, Hwnd};

use crate::noise::Noise;

/// How late a flush's return sits after the blank it is returning at, as measured.
///
/// `scripts/compositor-probe.c` at 144Hz, 119 gaps between flush returns against a 6944.4µs refresh:
/// mean 6945.2µs, least 5856.1, most 8048.8, and in 200µs bands about the mean
///
/// ```text
///  -1700                      mean                      +1700
///    0  0  0  1  0  4  4  3    96    1  5  4  0  0  1  0  0
/// ```
///
/// so **81% of returns are within 100µs of the refresh** and the rest are single excursions. The raw
/// run shows what those are: `... 6912 6964 7539 6405 6874 7392 6528 6918 ...` — one gap long and the
/// next short, the two summing back to twice the refresh. That is one late wake, not a drifting clock,
/// which is why the blanks here are an exact grid and only the *return* is delayed.
///
/// Read off that histogram: `USUAL_US` for the 90% within a band or two of nothing, `SPIKE_US` for the
/// 9% that are not, of which the largest measured was about 1300µs. Not a shape chosen for
/// convenience — a delay drawn evenly over the whole range, which is what this was before the
/// distribution was measured, makes a tenth of the frames miss where a real machine misses three per
/// hundred.
pub const USUAL_US: i64 = 100;
pub const SPIKE_US: i64 = 1_300;
/// How often the wake is one of the excursions rather than one of the 96, in parts per hundred.
pub const SPIKE_PERCENT: i64 = 9;

/// What a compositor is doing, when there is one.
#[derive(Clone, Copy)]
struct Compositor {
    /// How far apart the blanks are, in performance-counter ticks.
    ///
    /// Kept apart from the monitor's reported rate rather than derived from it, because the case
    /// the pacing refuses — and which `TODO.md` records as never having been seen happen — is
    /// exactly the two disagreeing: the compositor's clock follows one monitor of the desktop, and
    /// the game's window may be on another.
    period: i64,
    /// The tick the blank grid is counted from.
    origin: i64,
    /// How long composing one frame usually takes, in ticks.
    compose: i64,
    /// How far above that an ordinary frame may cost, drawn per frame.
    compose_jitter: i64,
    /// And how long it takes when it does not, in ticks.
    ///
    /// A compositor is not a constant: it shares a GPU with the rest of the desktop and now and then
    /// takes far longer over a frame than it usually does. That is the whole reason orb's allowance
    /// ratchets rather than settling, and a simulator with a fixed compose time cannot produce the
    /// behaviour the allowance exists for — it makes *every* frame expensive instead of a few, which
    /// costs refreshes a real machine does not lose.
    compose_spike: i64,
    /// How rare the spike is: one frame in this many.
    spike_one_in: i64,
    /// When the game last handed a frame over. `None` before it has handed over any, where a flush
    /// has nothing of ours to wait for and returns at the next blank.
    presented: Option<i64>,
    /// Compositions, not refreshes — see the note at the top.
    refresh: i64,
}

impl Compositor {
    /// The first blank at or after `at`.
    fn blank_at_or_after(&self, at: i64) -> i64 {
        let since = at - self.origin;
        let whole = since.div_euclid(self.period);
        let blank = self.origin + whole * self.period;
        if blank >= at {
            blank
        } else {
            blank + self.period
        }
    }

    /// The most recent blank at or before `at`.
    fn blank_at_or_before(&self, at: i64) -> i64 {
        self.origin + (at - self.origin).div_euclid(self.period) * self.period
    }
}

pub struct Display {
    /// What `EnumDisplaySettingsW` reports for the monitor the window is on, in whole Hz.
    monitor_hz: Mutex<Option<u32>>,
    /// What `GetDeviceCaps(VREFRESH)` reports for the desktop.
    desktop_hz: Mutex<Option<u32>>,
    compositor: Mutex<Option<Compositor>>,
    foreground: Mutex<Hwnd>,
    /// Whether the wake delay is modelled at all, and the stream of draws it comes from.
    wake: Mutex<(bool, Noise)>,
    /// Whether `cFramesLate` climbs with the compositions instead of reading zero — see
    /// [`says_every_composition_was_late`](Self::says_every_composition_was_late).
    every_composition_late: Mutex<bool>,
}

impl Default for Display {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Display {
    /// `seed` decides the wake delays. A scenario passes it in and names it in every assertion, so a
    /// failure can be replayed.
    pub fn new(seed: u64) -> Self {
        Self {
            monitor_hz: Mutex::new(None),
            desktop_hz: Mutex::new(None),
            compositor: Mutex::new(None),
            foreground: Mutex::new(Hwnd::NULL),
            wake: Mutex::new((true, Noise::seeded(seed))),
            every_composition_late: Mutex::new(false),
        }
    }

    /// Turns the wake delay off, making the host a metronome — which no host is. Only for a scenario
    /// about arithmetic, and one that says so.
    pub fn as_a_metronome(&self) {
        self.wake.lock().unwrap().0 = false;
    }

    /// Has `cFramesLate` climb one per composition, which is a host saying every frame it was handed
    /// missed the refresh it was aimed at.
    ///
    /// The real one reads zero throughout — see the note at the top — so this is not a machine and
    /// nothing may be asserted *from* it. What it is for is the opposite claim: a scenario runs the
    /// same frames twice, once against a host reading zero and once against this, and holds every
    /// number orb decided to be the same across the two. The loudest answer a compositor could give
    /// rather than a plausible middle, because what is being shown is that no answer reaches a
    /// decision.
    pub fn says_every_composition_was_late(&self) {
        *self.every_composition_late.lock().unwrap() = true;
    }

    /// How late this flush is woken, drawn from the measured distribution. See [`USUAL_US`].
    fn wake_delay(&self) -> i64 {
        let (modelled, noise) = &mut *self.wake.lock().unwrap();
        if !*modelled {
            return 0;
        }
        let us = if noise.up_to(99) < SPIKE_PERCENT {
            USUAL_US + noise.up_to(SPIKE_US - USUAL_US)
        } else {
            noise.up_to(USUAL_US)
        };
        crate::Clock::ticks_for_micros(us)
    }

    /// What the monitor the window is on reports, in whole Hz. `None` is a monitor that will not
    /// say, which is what paces the game by the clock.
    pub fn set_monitor_hz(&self, hz: Option<u32>) {
        *self.monitor_hz.lock().unwrap() = hz;
    }

    /// What the desktop reports, which the pacing reads once at startup before there is a window.
    pub fn set_desktop_hz(&self, hz: Option<u32>) {
        *self.desktop_hz.lock().unwrap() = hz;
    }

    /// Puts a compositor there, timing blanks `period` ticks apart from `now`, and taking `compose`
    /// ticks over each frame.
    pub fn attach_compositor(&self, now: i64, period: i64, compose: Compose) {
        *self.compositor.lock().unwrap() = Some(Compositor {
            period,
            origin: now,
            compose: crate::Clock::ticks_for_micros(compose.usual_us),
            compose_jitter: crate::Clock::ticks_for_micros(compose.jitter_us),
            compose_spike: crate::Clock::ticks_for_micros(compose.spike_us),
            spike_one_in: compose.spike_one_in,
            presented: None,
            refresh: 0,
        });
    }

    /// Takes the compositor away — turned off, or a session change — after which a flush fails and
    /// the pacing falls back to the clock.
    pub fn detach_compositor(&self) {
        *self.compositor.lock().unwrap() = None;
    }

    /// Changes what composing a frame takes, which a test does mid-run to see the pacing follow it.
    pub fn set_compose(&self, compose: Compose) {
        if let Some(compositor) = self.compositor.lock().unwrap().as_mut() {
            compositor.compose = crate::Clock::ticks_for_micros(compose.usual_us);
            compositor.compose_jitter = crate::Clock::ticks_for_micros(compose.jitter_us);
            compositor.compose_spike = crate::Clock::ticks_for_micros(compose.spike_us);
            compositor.spike_one_in = compose.spike_one_in;
        }
    }

    /// How far apart the blanks are, for a test working out where one should have fallen.
    pub fn compositor_period(&self) -> Option<i64> {
        self.compositor.lock().unwrap().map(|it| it.period)
    }

    pub fn set_foreground(&self, window: Hwnd) {
        *self.foreground.lock().unwrap() = window;
    }

    pub fn foreground(&self) -> Hwnd {
        *self.foreground.lock().unwrap()
    }

    pub fn monitor_refresh(&self, window: Hwnd) -> Option<u32> {
        // A window that is not there has no monitor, as the real call has none to find.
        if window.is_null() {
            return None;
        }
        *self.monitor_hz.lock().unwrap()
    }

    pub fn desktop_refresh(&self) -> Option<u32> {
        *self.desktop_hz.lock().unwrap()
    }

    pub fn composition(&self, now: i64) -> Option<Composition> {
        let compositor = (*self.compositor.lock().unwrap())?;
        Some(Composition {
            refresh_period: compositor.period,
            refresh: compositor.refresh,
            vblank: compositor.blank_at_or_before(now),
            // Zero, as the real one reads, unless a scenario asked for the other answer. See the note
            // at the top of this module.
            frames_late: if *self.every_composition_late.lock().unwrap() {
                compositor.refresh
            } else {
                0
            },
        })
    }

    /// Says the game has handed a frame over, which is what a flush then waits to be composed.
    pub fn presented(&self, now: i64) {
        if let Some(compositor) = self.compositor.lock().unwrap().as_mut() {
            compositor.presented = Some(now);
            compositor.refresh += 1;
        }
    }

    /// Which tick a flush returns at, or `None` where there is no compositor to flush.
    ///
    /// The blank the frame in the compositor's hands reached — the first one at or after it was
    /// composed — plus however late the waiting thread is woken. Never earlier than `now`, since a
    /// flush cannot return in the past.
    ///
    /// The wake delay is what makes this non-deterministic, and deliberately: a host that woke a
    /// thread the instant a blank came would be a host nobody has. See [`USUAL_US`].
    pub fn flush(&self, now: i64) -> Option<i64> {
        let compositor = (*self.compositor.lock().unwrap())?;
        let took = {
            let (_, noise) = &mut *self.wake.lock().unwrap();
            if compositor.spike_one_in > 0 && noise.up_to(compositor.spike_one_in - 1) == 0 {
                compositor.compose_spike
            } else if compositor.compose_jitter > 0 {
                compositor.compose + noise.up_to(compositor.compose_jitter)
            } else {
                compositor.compose
            }
        };
        let composed = match compositor.presented {
            Some(presented) => presented + took,
            // Nothing of ours is in its hands, so there is nothing of ours to be shown: the next
            // blank is all a flush can wait for.
            None => now,
        };
        Some((compositor.blank_at_or_after(composed) + self.wake_delay()).max(now))
    }
}

/// What the compositor takes over a frame: what it usually takes, what it takes when it does not,
/// and how often that is.
///
/// Two numbers and a rate rather than one, because a compositor is not a constant and orb's allowance
/// exists for the times it is not. A fixed compose time makes every frame expensive, which costs
/// refreshes a real machine does not lose — measured: with a fixed 2000µs a simulated 120Hz display
/// lost 35 refreshes in 1592 frames, where the real 120Hz run is `600 frames, 16650us apart, gaps in
/// refreshes 2x600` over seven periods in a row, none lost at all.
/// Three sources of unevenness rather than two, and the third is what a scenario about the *rate* wants:
/// the usual cost is not one number either, it wanders. `jitter_us` is how far above `usual_us` it may
/// wander, drawn per frame from the same seeded stream the wake delays come from, so a scenario declares
/// how uneven this compositor is and a deterministic one is that spread set to nothing.
#[derive(Clone, Copy)]
pub struct Compose {
    pub usual_us: i64,
    /// How far over `usual_us` an ordinary frame may cost, drawn per frame. Zero is a compositor that
    /// takes exactly `usual_us` every time, which no machine does and every claim about arithmetic
    /// wants.
    pub jitter_us: i64,
    pub spike_us: i64,
    /// One frame in this many. Zero for a compositor that never spikes.
    pub spike_one_in: i64,
}

impl Compose {
    /// What this machine appears to do, from three observations rather than one:
    ///
    /// - `usual_us` — anything up to about 1200µs reproduces the real 120Hz run exactly:
    ///   `600 frames, 16650us apart, gaps in refreshes 2x600` with the allowance never climbing off its
    ///   2500µs start. Above it the simulated allowance climbs where the real one did not. And
    ///   `scripts/background-flush-probe.c` puts this machine's compose time between 1000 and 2000µs
    ///   from the other side: a frame handed over 1000µs before a blank made it 54 times in 60, and one
    ///   handed over 2000µs before made it 60 times in 60.
    /// - `spike_us` — the compositor is reported to reach about this in **half an hour** of play.
    /// - `spike_one_in` — which is what half an hour says about how rare it is: thirty minutes at sixty
    ///   frames a second is 108,000 frames, and reaching 3500µs about once in that is one frame in a
    ///   hundred thousand. A floor on the rarity rather than a count of it, since nobody counted — but
    ///   the order of it is the point, and it is the thing this was wrong about. It was nine in six
    ///   hundred, taken from a mixed-rate run's miss accounting, and that run was mispaced: its misses
    ///   were the pacing's and not the compositor's. Three orders of magnitude too often, and it cost
    ///   the simulated 60Hz display two seconds in five that a real one holds.
    ///
    /// So a scenario of a few thousand frames sees no spike at all, which is what the real 600-frame
    /// runs show — `gaps in refreshes 2x600`, none lost. The spike is what the allowance's
    /// ratchet exists for over a *session*, and a scenario about it says so and sets its own.
    /// `jitter_us` is what is left over from the same reasoning: the 1200µs that still reproduces the
    /// real run is the ceiling of the ordinary cost, not its middle, so an ordinary frame is put between
    /// 1000 and 1200µs rather than at either end.
    pub fn measured() -> Self {
        Self {
            usual_us: 1_000,
            jitter_us: 200,
            spike_us: 3_500,
            spike_one_in: 100_000,
        }
    }

    /// As [`measured`](Self::measured), with the ordinary cost wandering as far as a scenario says.
    ///
    /// For the rows of a decision table that vary the compositor's unevenness on its own, since what the
    /// spike answers for and what a restless ordinary cost answers for are not the same thing: the
    /// allowance ratchets past a spike once and stays there, where a cost that wanders has to be covered
    /// every frame.
    pub fn wandering(jitter_us: i64) -> Self {
        Self {
            jitter_us,
            ..Self::measured()
        }
    }

    /// As [`measured`](Self::measured), but spiking often enough that a scenario a few hundred frames
    /// long sees several.
    ///
    /// One in a hundred thousand is what half an hour of play suggests, and over a fifteen-second
    /// scenario that is no spikes at all — a path nothing reaches is a path nothing tests. So a scenario
    /// about the spike says how often, and this is the rate that puts a handful inside one: one in three
    /// hundred is about three a run, which is what the allowance's ratchet has to answer for.
    ///
    /// Deliberately not the default. What the default is for is asking whether a display holds sixty on
    /// the machine this was measured from, and a compositor spiking three hundred times as often is not
    /// that machine.
    ///
    /// And with the ordinary cost held still, unlike [`measured`](Self::measured): the allowance answers
    /// a spike and a restless ordinary cost with the same ratchet, so a scenario that moved both could not
    /// say which of them moved it. A row about the wander says so with
    /// [`wandering`](Self::wandering).
    pub fn spiking() -> Self {
        Self {
            jitter_us: 0,
            spike_one_in: 300,
            ..Self::measured()
        }
    }

    /// One value every time, for a scenario about arithmetic rather than about a machine.
    pub fn flat(us: i64) -> Self {
        Self {
            usual_us: us,
            jitter_us: 0,
            spike_us: us,
            spike_one_in: 0,
        }
    }
}
