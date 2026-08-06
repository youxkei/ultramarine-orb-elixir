//! **Sixty frames a second is held in every situation, and this is the table of what a situation is.**
//!
//! The one claim the pacing exists to make, in one place. Every other `pacing_*.rs` is about a mechanism
//! inside it — the budget's climb, the allowance's ratchet, the load guard, the grid's arithmetic — and
//! each of those can be right while the thing somebody playing cares about is wrong.
//!
//! # Why a table and not every combination
//!
//! Four things vary: the monitor's rate, the compositor's rate, how unevenly the compositor takes its
//! time, and how unevenly the game's own frame takes its. Crossing them is several hundred runs, and most
//! of the crossings ask the same question twice, because **only the first two decide which path the code
//! takes.** `adopt` chooses among five: no compositor to ask, a rate that is a whole multiple of sixty, a
//! rate that rounds to one, a rate that is not a multiple, and a rate under sixty. The other two are the
//! same arithmetic in every one of those — they widen what the allowance has to cover and nothing else.
//!
//! So the rows are the five paths, each at rest and each with everything uneven at once, and then the
//! unevenness taken apart one source at a time on the one path that has least room to absorb it. A row
//! that fails says which condition did it, which is what a table is for and what a cross product is not.
//!
//! # One test a row
//!
//! Not a loop over rows inside one test, because a row is a launch and a launch is a process — so the
//! harness runs them at once, sixteen at a time on this machine, and the whole table takes six seconds
//! where a loop would take a minute and a half. The seeds stay a loop inside a row: four hosts of the
//! same situation are the same question asked again, and naming which of them failed is what the
//! assertion messages do.
//!
//! # Where the floors come from
//!
//! Measured, per row, over the four hosts apiece, and each floor is set below the worst of them: what a
//! lost refresh costs turns on the display's rate and how often one is lost turns on how uneven the row
//! declared its desktop to be, so a single floor low enough for the worst row would say nothing about any
//! of the others. What the floor is for is a *stretch* of bad seconds — the mixed-rate desktop's own
//! defect was 0% of them — and not one more second than last time.
//!
//! | row | seconds untouched, per host | floor |
//! | --- | --- | --- |
//! | 120Hz, restless | 73 91 73 73 | 65% |
//! | 60Hz, restless | 55 64 64 64 | 45% |
//! | 144Hz, restless | 100 91 73 82 | 65% |
//! | 119Hz, restless | 73 91 73 82 | 65% |
//! | no compositor, restless | 100 91 73 91 | 65% |
//! | 60Hz, the compositor's ordinary cost wandering | 91 73 91 64 | 55% |
//! | 60Hz, the compositor spiking | 82 91 64 73 | 55% |
//! | 60Hz, the game's own frame wandering | 100 100 100 100 | 90% |
//! | 120Hz monitor, 144Hz compositor, restless | 100 91 73 82 | 65% |
//! | 120Hz monitor, 70Hz compositor, restless | 91 91 64 73 | 55% |
//! | silent monitor, 120Hz compositor, restless | 73 91 73 73 | 65% |
//!
//! # What is *not* here
//!
//! The three of these that are not sixty, each of which has its own reason and its own row below:
//! a display under sixty, which is deliberately the clock; the NTSC-derived rates, which get the
//! display's own rate and are right to; and a compositor wanting more than three quarters of a refresh,
//! which is geometry and lives in `pacing_converges.rs`.

mod fake;
mod pacing;

use fake::{Display, Work, in_its_own_process};
use orb_sim::Compose;
use pacing::{for_each_seed, launched_with};

/// Fifteen seconds: three for the allowance to finish climbing and a dozen judged a second at a time.
///
/// Which is also what makes "some proportion of the time" the strongest form of itself here. A run of
/// twelve judged seconds cannot measure 99% — eleven of twelve is 91.7% — so the assertion is *every*
/// second, and a row that wants a proportion instead says so and says why.
const FRAMES: u32 = 15 * pacing::A_SECOND as u32;

/// What the game's own frame costs when it is not the thing being varied: what a real run's report line
/// said, "(694us to draw + …)".
const WORK_US: i64 = 700;

/// Everything uneven at once, which is the row that says the sources do not compound into something none
/// of them does alone.
fn a_restless_desktop() -> Compose {
    Compose {
        jitter_us: 800,
        spike_one_in: 300,
        ..Compose::measured()
    }
}

/// And the game's own frame the same way: wandering by half again, with a stage load about twice in a
/// fifteen-second run.
fn a_restless_game() -> Work {
    Work::loading(WORK_US, 450)
}

/// How a row is judged, which is a column of the table and not a tolerance to be chosen.
///
/// Two, because the unevenness the rows declare is not something the pacing can undo — it is the thing
/// the pacing has to survive, and two of the three sources cost a second by arithmetic:
///
/// - a refresh lost to a late wake or a slow composition costs its second one frame, so that second reads
///   59.0 at 60Hz and 59.5 at 120Hz whatever orb does about it;
/// - a stage load is a quarter of a second in one frame, so its second holds sixty frames in 1.25
///   seconds, which is 48 and is not a rate anybody can pace out of.
///
/// So a row with nothing uneven in it is held to every second, and a row that declares unevenness is held
/// to what unevenness cannot excuse: the seconds it left alone averaging the rate asked for, the share it
/// left alone, and no second collapsing past what one load costs.
#[derive(Clone, Copy)]
enum Judged {
    /// Every second within half a frame. For the rows with nothing to absorb.
    EverySecond,
    /// The run itself within half a frame, no second below a load's own arithmetic, and at least this
    /// share of the seconds untouched — on the worst of the four hosts, not on each in turn.
    SecondsUntouched(f64),
}

/// Which host a row runs on, and it is a condition like the others: the third source of unevenness is
/// the machine itself, which wakes the waiting thread when it gets round to it.
#[derive(Clone, Copy, PartialEq)]
enum Host {
    /// No wake delay at all. One run, since there is no stream left to draw a second one from — and the
    /// only kind of host a claim about arithmetic can be made against, no machine being one.
    Metronome,
    /// The measured distribution, over [`pacing::SEEDS`] of them, each named in what it fails.
    Real,
}

/// One row: the display, the compositor's unevenness, the game's own, the host's, the rate the run has to
/// come out at, and how it is judged.
fn holds(
    name: &str,
    display: impl Fn(u64) -> Display,
    compose: Compose,
    work: Work,
    host: Host,
    wanted: f64,
    judged: Judged,
) {
    in_its_own_process(|| {
        let mut shares = Vec::new();
        let mut run = |seed: u64| {
            let mut display = display(seed);
            display.compose = compose;
            display.seed = seed;
            display.metronome = host == Host::Metronome;
            let game = launched_with(display, &format!("{name}-{seed}"), work);
            game.frames(FRAMES);
            let handovers = game.handovers();
            match judged {
                Judged::EverySecond => {
                    pacing::assert_every_second_at(&game, &handovers, wanted, seed);
                }
                Judged::SecondsUntouched(_) => {
                    shares.push((
                        seed,
                        pacing::assert_holds_through_it(&game, &handovers, wanted, seed),
                    ));
                }
            }
        };
        match host {
            Host::Metronome => run(0),
            Host::Real => for_each_seed(run),
        }
        // The worst host of the four, held to the row's own floor. Held here rather than inside the run
        // so that the message names every host: a row that has slipped has slipped by some amount on
        // some of them, and which is what says whether it is a stretch or one more second than before.
        if let Judged::SecondsUntouched(floor) = judged {
            let (seed, worst) =
                shares.iter().copied().fold(
                    (0, f64::MAX),
                    |worst, at| if at.1 < worst.1 { at } else { worst },
                );
            assert!(
                worst >= floor,
                "seed {seed} left {:.0}% of the seconds untouched, and {:.0}% is this row's floor — {:?}",
                worst * 100.0,
                floor * 100.0,
                shares
                    .iter()
                    .map(|(seed, share)| format!("seed {seed}: {:.0}%", share * 100.0))
                    .collect::<Vec<_>>(),
            );
        }
    });
}

fn agreed(hz: u32) -> impl Fn(u64) -> Display {
    move |_| Display::agreed(hz)
}

fn split(monitor_hz: u32, compositor_hz: u32) -> impl Fn(u64) -> Display {
    move |_| Display::split(monitor_hz, compositor_hz)
}

/// A monitor that will not report its rate, on a desktop whose compositor will.
fn silent_monitor(compositor_hz: u32) -> impl Fn(u64) -> Display {
    move |_| Display {
        compositor_hz: Some(compositor_hz),
        ..Display::unknown()
    }
}

// ── The five paths at rest ───────────────────────────────────────────────────────────────────────
//
// Even work and an even compositor: what the arithmetic does with nothing to absorb.

#[test]
fn a_whole_multiple_at_rest() {
    holds(
        "rest-120",
        agreed(120),
        Compose::flat(1_000),
        Work::flat(WORK_US),
        Host::Metronome,
        60.0,
        Judged::EverySecond,
    );
}

/// 60Hz is the whole multiple with no room in it: one refresh *is* one game turn, so a frame that misses
/// its blank loses a whole frame and the one after cannot come early enough to make it back.
#[test]
fn the_whole_multiple_with_no_room_at_rest() {
    holds(
        "rest-60",
        agreed(60),
        Compose::flat(1_000),
        Work::flat(WORK_US),
        Host::Metronome,
        60.0,
        Judged::EverySecond,
    );
}

#[test]
fn a_fractional_rate_at_rest() {
    holds(
        "rest-144",
        agreed(144),
        Compose::flat(1_000),
        Work::flat(WORK_US),
        Host::Metronome,
        60.0,
        Judged::EverySecond,
    );
}

/// The NTSC-derived rates, whose target is the display's own and not sixty. A 119.88Hz panel reports a
/// period that rounds to 119, which is two blanks a frame and 59.5 frames a second — and `DONE.md` has
/// the real one at 59.94, "the display's own rate halved". A tenth of a per cent is a clock nobody sees.
#[test]
fn an_ntsc_rate_at_rest_gets_the_displays_own() {
    holds(
        "rest-119",
        agreed(119),
        Compose::flat(1_000),
        Work::flat(WORK_US),
        Host::Metronome,
        59.5,
        Judged::EverySecond,
    );
}

/// No compositor at all, which is the one path with no blank to put a frame on. Sixty by the clock.
#[test]
fn no_compositor_at_rest() {
    holds(
        "rest-clock",
        |_| Display::unknown(),
        Compose::flat(1_000),
        Work::flat(WORK_US),
        Host::Metronome,
        60.0,
        Judged::EverySecond,
    );
}

// ── The same five with everything uneven at once ─────────────────────────────────────────────────

#[test]
fn a_whole_multiple_on_a_restless_desktop() {
    holds(
        "restless-120",
        agreed(120),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.65),
    );
}

#[test]
fn the_whole_multiple_with_no_room_on_a_restless_desktop() {
    holds(
        "restless-60",
        agreed(60),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.45),
    );
}

#[test]
fn a_fractional_rate_on_a_restless_desktop() {
    holds(
        "restless-144",
        agreed(144),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.65),
    );
}

#[test]
fn an_ntsc_rate_on_a_restless_desktop() {
    holds(
        "restless-119",
        agreed(119),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        59.5,
        Judged::SecondsUntouched(0.65),
    );
}

#[test]
fn no_compositor_on_a_restless_desktop() {
    holds(
        "restless-clock",
        |_| Display::unknown(),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.65),
    );
}

// ── The unevenness taken apart, on the path with least room to absorb it ─────────────────────────
//
// 60Hz, because a missed blank costs a whole game frame there and nowhere else. A row here failing while
// the row above it passes says which of the four sources did it.

#[test]
fn a_compositor_whose_ordinary_cost_wanders() {
    holds(
        "apart-compose-wander",
        agreed(60),
        Compose::wandering(800),
        Work::flat(WORK_US),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.55),
    );
}

#[test]
fn a_compositor_that_spikes() {
    holds(
        "apart-compose-spike",
        agreed(60),
        Compose::spiking(),
        Work::flat(WORK_US),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.55),
    );
}

#[test]
fn a_game_whose_own_frame_wanders() {
    holds(
        "apart-work-wander",
        agreed(60),
        Compose::flat(1_000),
        Work::wandering(WORK_US),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.9),
    );
}

/// A frame that costs a quarter of a second now and then, which is a stage load. The second it lands in
/// is one the arithmetic cannot give back — sixty frames in the 1.25 seconds a quarter-second load makes
/// of it is 48 — so what this holds is that the seconds either side of it are untouched, which is
/// `pacing_load.rs`'s claim about the allowance seen from the rate's end.
#[test]
fn a_game_that_loads_a_stage_now_and_then() {
    holds(
        "apart-work-load",
        agreed(60),
        Compose::flat(1_000),
        Work::loading(WORK_US, 450),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.65),
    );
}

/// And the host's own unevenness alone, which is what every row above already carries: the wake delays
/// are drawn per flush from the seed. This is that with nothing else moving, so a failure here is the
/// host and not the desktop.
#[test]
fn a_host_that_wakes_the_waiting_thread_late() {
    holds(
        "apart-host",
        agreed(60),
        Compose::flat(1_000),
        Work::flat(WORK_US),
        Host::Real,
        60.0,
        Judged::EverySecond,
    );
}

// ── The desktop the two rates disagree on, which decides no path and is here to prove it ─────────

#[test]
fn a_compositor_timing_another_monitor_on_a_restless_desktop() {
    holds(
        "restless-split-144",
        split(120, 144),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.65),
    );
}

/// The worst fractional rate anyone has: 70Hz is 1.167 refreshes to a frame, so the grid alternates one
/// blank and two and no frame is evenly spaced against its neighbours. The rate is still sixty.
#[test]
fn the_least_even_fractional_compositor_on_a_restless_desktop() {
    holds(
        "restless-split-70",
        split(120, 70),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.55),
    );
}

#[test]
fn a_monitor_that_will_not_say_on_a_restless_desktop() {
    holds(
        "restless-silent",
        silent_monitor(120),
        a_restless_desktop(),
        a_restless_game(),
        Host::Real,
        60.0,
        Judged::SecondsUntouched(0.65),
    );
}
