//! How a run's rate is judged, for the scenarios about orb's own frame loop.
//!
//! Two readings of the same run, and both are asserted on. The rate comes off the ticks the game was
//! handed its frames over at — `Fake::handovers`, which is where its own `Present` wrote them down —
//! and beside it is the `frame:` line orb writes per reporting period, taken apart by [`Reported`].
//! The first is what somebody playing has; the second is what somebody reading a real run's log has to
//! be able to believe.
//!
//! Roughly the rate rather than the exact turn. The host wakes the waiting thread when it gets round to
//! it and its compositor is slow now and then — see `orb-sim/src/display.rs` — so a scenario that
//! asserted a turn to the microsecond would be asserting that the host is a metronome, which none is.

// Compiled into each test binary that uses it, so every one sees the helpers the others needed.
#![allow(dead_code)]

use orb_core::game::RunStart;
use orb_core::profile;
use orb_sim::Clock;

use crate::fake::{Display, Fake, Work};

/// A launch on `display` whose every frame does the same `work_us` of the game's own work.
///
/// The even one, for a scenario about the arithmetic around it. What the frames really cost is not one
/// number — see [`launched_with`].
pub fn launched(display: Display, name: &str, work_us: i64) -> Box<Fake> {
    launched_with(display, name, Work::flat(work_us))
}

/// And one whose frames cost as unevenly as the scenario says — see [`Work`].
///
/// Both open the pacing log, so that what orb says about its own rate is written where a scenario can
/// read it back. The game sits on its title menu throughout: what these are about is the frame loop, and
/// a stage being played is a load and a draw rather than another question about the cadence — which is
/// what `work` stands for.
pub fn launched_with(display: Display, name: &str, work: Work) -> Box<Fake> {
    let game = Fake::attach_to_display(display, name, the_run(), |config| {
        config.pacing_log = true;
    });
    game.frame_takes(work);
    game
}

/// The run a launch is started for: Normal, Reimu A, from stage one. None of it is played here.
fn the_run() -> RunStart {
    RunStart {
        difficulty: 1,
        character: 0,
        shot_type: 0,
        practice: false,
        stage: 0,
    }
}

/// How near sixty every second of a run has to come, in frames a second.
///
/// Three, not a tenth, and judged a second at a time rather than over the run — which is what somebody
/// playing has. A turn a refresh long either way is nothing; a second at half rate is the complaint. An
/// average over three thousand frames is no use for saying so: a run that spends a second at 30 and a
/// second at 90 averages sixty and is unplayable.
///
/// Six because of what a good display measures at, and because of what one lost refresh costs. The host
/// is not a metronome — it wakes the waiting thread when it gets round to it, and its compositor is slow
/// now and then — so a second here and there loses a refresh. At 60Hz a refresh is a sixtieth, so *one*
/// lost in a second of sixty frames takes that second from 60 to 59.0, and a handful takes it to 55.3,
/// which is the worst measured over four hosts and the rates a display reports. With the compositor's
/// cost held flat instead, every second comes out within 0.08.
///
/// So the band is what the modelled host does, and it is nowhere near the rates a defect produces: 30
/// on a latched allowance, 48, 52, 71. What it does not distinguish is a second that lost a refresh
/// from one that did not — the `shown` line's own count is what says that, and the scenarios read it
/// where it matters.
pub const NEAR_SIXTY: f64 = 6.0;

/// How near sixty a second has to be to *count* as sixty, in frames a second.
///
/// Tight, unlike [`NEAR_SIXTY`], because this is not a tolerance — it is the question "was this second
/// sixty frames a second or was it not". Half a frame either way is the game running at its own speed;
/// anything more is a second that lost a refresh or gained one. What the scenarios then assert is the
/// *proportion* of seconds that were, which is the shape the answer wants: a run is not "60fps ± 6", it
/// is 60fps for so much of its length.
pub const AT_SIXTY: f64 = 0.5;

/// How many hosts a scenario is held against.
///
/// One is not enough: the wake delays are drawn, so a run is one of many the machine could have given.
/// A scenario that holds for one seed and not another has found something a real machine can do, which
/// is a defect and not a flake — so every assertion names the seed it failed on.
pub const SEEDS: u64 = 4;

/// Runs `body` against each host in turn.
pub fn for_each_seed(mut body: impl FnMut(u64)) {
    for seed in 0..SEEDS {
        body(seed);
    }
}

/// A second of play, in frames. The unit the rate is judged in, because it is the unit somebody playing
/// notices: one turn a refresh long either way is nothing, and a second at half rate is the complaint.
pub const A_SECOND: usize = 60;

/// How long the rate is given to settle, in seconds.
///
/// The allowance starts at 2500µs and climbs a hundred microseconds a miss, so a display whose
/// compositor wants more than that spends its first seconds below rate — that is the climb working, not
/// a fault. What would be a fault is it still being below after a few seconds of play, which is when
/// somebody has reached the title screen and would notice.
pub const GRACE_SECONDS: usize = 3;

/// The turns in microseconds: how long each frame's turn was, from being handed over to the one after.
pub fn turns(handovers: &[i64]) -> Vec<i64> {
    handovers
        .iter()
        .zip(handovers.iter().skip(1))
        .map(|(before, after)| Clock::micros_for_ticks(after - before))
        .collect()
}

/// The turns as counts of the compositor's refreshes, rounded.
pub fn refreshes(game: &Fake, handovers: &[i64]) -> Vec<i64> {
    let period = game
        .sim()
        .display()
        .compositor_period()
        .expect("a display this scenario declared a compositor for");
    handovers
        .iter()
        .zip(handovers.iter().skip(1))
        .map(|(before, after)| (after - before + period / 2) / period)
        .collect()
}

/// The rate the run came out at, over the turns after `warm_up`.
pub fn fps(handovers: &[i64], warm_up: usize) -> f64 {
    let turns = turns(handovers);
    let settled = &turns[warm_up..];
    1_000_000.0 / (settled.iter().sum::<i64>() as f64 / settled.len() as f64)
}

/// The rate over each second of the run from `warm_up` on, so that a stretch at the wrong rate cannot
/// be averaged out by a stretch at the right one.
///
/// An average over the whole run is not what a player has: a run that spends a second at 30 and a
/// second at 90 averages sixty and is unplayable.
pub fn rate_each_second(handovers: &[i64], warm_up: usize) -> Vec<f64> {
    turns(handovers)[warm_up..]
        .chunks(A_SECOND)
        .filter(|second| second.len() == A_SECOND)
        .map(|second| 1_000_000.0 / (second.iter().sum::<i64>() as f64 / second.len() as f64))
        .collect()
}

/// What fraction of the seconds from `warm_up` on were sixty frames a second, by [`AT_SIXTY`].
pub fn share_at_sixty(handovers: &[i64], warm_up: usize) -> f64 {
    let seconds = rate_each_second(handovers, warm_up);
    let at = seconds
        .iter()
        .filter(|rate| (**rate - 60.0).abs() < AT_SIXTY)
        .count();
    at as f64 / seconds.len() as f64
}

/// The second that came out furthest from sixty, and how far.
pub fn worst_second(handovers: &[i64], warm_up: usize) -> (usize, f64) {
    rate_each_second(handovers, warm_up)
        .into_iter()
        .enumerate()
        .fold((0, 60.0), |worst, (at, rate)| {
            if (rate - 60.0).abs() > (worst.1 - 60.0).abs() {
                (at, rate)
            } else {
                worst
            }
        })
}

/// Every second after [`GRACE_SECONDS`] ran at `wanted` frames a second, by [`AT_SIXTY`].
///
/// `wanted` is sixty for every display that has a blank on a sixtieth of a second, which is every
/// whole multiple and every fractional rate the grid can chase. It is *not* sixty for the
/// NTSC-derived ones: a display reporting 59 gets one blank a frame and runs at its own rate, which
/// `DONE.md` records for the real 119.88Hz case — "59.94fps, the display's own rate halved". That is
/// the pacing working; a tenth of a percent slow is a clock nobody can see.
///
/// All of the seconds, not most: measured over four hosts and thirty seconds apiece, every display
/// the pacing accepts holds every second — which is what `DONE.md`'s real runs show too, `gaps in
/// refreshes 2x600` with none lost. So a shortfall here is a finding rather than a flake, and the
/// seed in the message is how to go back to it.
pub fn assert_every_second_at(game: &Fake, handovers: &[i64], wanted: f64, seed: u64) {
    let warm_up = GRACE_SECONDS * A_SECOND;
    let seconds = rate_each_second(handovers, warm_up);
    assert!(
        !seconds.is_empty(),
        "a run this short says nothing after {GRACE_SECONDS}s of grace"
    );
    let astray: Vec<(usize, f64)> = seconds
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, rate)| (rate - wanted).abs() >= AT_SIXTY)
        .collect();
    assert!(
        astray.is_empty(),
        "seed {seed}: {} of {} seconds were not {wanted} frames a second — {astray:?}\n  {}",
        astray.len(),
        seconds.len(),
        last_said(game),
    );
}

/// Most of the seconds were sixty, and the run as a whole was.
///
/// For a scenario where something is *meant* to cost a frame its blank now and then: a spike does
/// that by definition, and the second it lands in comes out a frame short. What must not happen is
/// the run losing the rate — a dip is a dip, and a stretch of them is a slow game.
///
/// Four fifths, measured: a compositor spiking about one frame in three hundred put one second of
/// eleven off sixty. The share and the rate together are what say the dips are dips.
pub fn assert_mostly_sixty(game: &Fake, handovers: &[i64], seed: u64) {
    let warm_up = GRACE_SECONDS * A_SECOND;
    let share = share_at_sixty(handovers, warm_up);
    let over_the_run = fps(handovers, warm_up);
    assert!(
        share >= 0.8 && (over_the_run - 60.0).abs() < AT_SIXTY,
        "seed {seed}: {:.0}% of the seconds were sixty and the run came out at {over_the_run} — \
         \n  {}",
        share * 100.0,
        last_said(game),
    );
}

/// A run came through the unevenness it was given: the rate itself, the share of seconds untouched, and
/// nothing collapsing further than a stage load's own arithmetic.
///
/// Three things and not one, because the unevenness is what the pacing has to survive rather than
/// something it can undo — see `Judged` in `pacing_holds.rs` for what each of the sources costs by
/// arithmetic — and each of the three fails differently:
///
/// - **the seconds it did leave alone average exactly the rate asked for.** Which is the claim about the
///   cadence: outside the seconds a lost refresh or a load landed in, the pacing is not slow, not fast
///   and not drifting.
///
///   The rate over the *whole* run is not that claim and cannot be. A quarter-second load spends its
///   quarter second whatever the pacing does, so a fifteen-second run with two of them takes 15.5
///   seconds of wall clock, and 900 frames in that is 58.06 a second — arithmetic, measured at 58.5 to
///   58.8 over these rows, and the reason this reads the untouched seconds instead.
/// - **no second collapsed further than a load's own arithmetic.** `WORST_SECOND` is what a
///   quarter-second frame makes of the second it lands in, and a load is the largest thing any row
///   declares, so a second below it is something else.
///
/// The share is answered rather than asserted, so that a row can hold the *worst* of its hosts to its own
/// floor rather than each host to it in turn.
pub fn assert_holds_through_it(game: &Fake, handovers: &[i64], wanted: f64, seed: u64) -> f64 {
    let warm_up = GRACE_SECONDS * A_SECOND;
    let seconds = rate_each_second(handovers, warm_up);
    let untouched: Vec<f64> = seconds
        .iter()
        .copied()
        .filter(|rate| (rate - wanted).abs() < AT_SIXTY)
        .collect();
    let worst = seconds.iter().copied().fold(f64::MAX, f64::min);
    assert!(
        !untouched.is_empty(),
        "seed {seed}: not one second was {wanted} frames a second — {seconds:?}\n  {}",
        last_said(game),
    );
    let among_them = untouched.iter().sum::<f64>() / untouched.len() as f64;
    assert!(
        (among_them - wanted).abs() < AT_SIXTY,
        "seed {seed}: the seconds nothing touched average {among_them}, where {wanted} was asked for — \
         {seconds:?}\n  {}",
        last_said(game),
    );
    assert!(
        worst >= WORST_SECOND,
        "seed {seed}: a second at {worst} frames, where a quarter of a second lost is {WORST_SECOND} — \
         {seconds:?}\n  {}",
        last_said(game),
    );
    untouched.len() as f64 / seconds.len() as f64
}

/// The lowest a second may read, which is a stage load's own arithmetic: a quarter of a second spent in
/// one frame leaves the other sixty to fit in the 1.25 seconds that second became, and 60/1.25 is 48.
/// Measured at 47.8 to 48.8 over the loads `pacing_load.rs` drives, so the bound is just under.
pub const WORST_SECOND: f64 = 47.0;

/// What one of orb's own `frame:` reporting lines says, taken apart.
///
/// Parsed rather than matched as text, so that a scenario says what the numbers have to be — and
/// strictly, because the shape of the line is as much of what is being held as the numbers are:
/// `Pacing::report` is what somebody looking into a stutter has in front of them, and a line that has
/// stopped saying one of these is a line they can no longer read.
#[derive(Debug)]
pub struct Reported {
    /// How many frames the period covered.
    pub frames: usize,
    /// How far apart they were handed over, smoothed 31 parts in 32 — the same reading the status
    /// line's `fps` is made of.
    pub interval_us: i64,
    /// How long before its blank a frame is started, in microseconds. Every one of them is input lag,
    /// which is why the line splits it into the two drawing times it is made of.
    pub prepare_us: i64,
    /// The game's own share of that: what its update and draw were last measured taking.
    pub draw_us: i64,
    /// And the compositor's: the allowance, which climbs a hundred microseconds every time a frame
    /// misses its blank and never comes back down.
    pub compose_us: i64,
    /// What the compositor said it could not show when it meant to, which orb's own gaps cannot see:
    /// they measure when a frame was handed over and not when it reached the screen.
    pub shown_late: usize,
    /// One entry per gap size that happened, in refreshes, with how many frames had it.
    pub gaps: Vec<(usize, usize)>,
}

impl Reported {
    /// # Panics
    /// On a line that is not one of those, naming what was missing from it.
    pub fn of(line: &str) -> Self {
        // Past the timestamp orb stamps every line with, whose own digits would otherwise be the
        // first number here.
        let said = line
            .split_once("frame: ")
            .unwrap_or_else(|| panic!("no {:?} in {line:?}", "frame: "))
            .1;
        // The line is commas between its parts and has none inside a part, so this is the whole of
        // taking it apart. `report` is the only writer, and a part that has moved fails here.
        let parts: Vec<&str> = said.split(", ").collect();
        let part = |what: &str, is: fn(&str) -> bool| -> &str {
            parts
                .iter()
                .copied()
                .find(|part| is(part))
                .unwrap_or_else(|| panic!("no {what} in {line:?}"))
        };
        let count = |text: &str| -> usize {
            text.parse()
                .unwrap_or_else(|_| panic!("{text:?} is not a count in {line:?}"))
        };
        let frames = part("frame count", |part| part.ends_with(" frames"));
        let interval = part("interval", |part| part.ends_with("us apart"));
        let late = part("frames shown late", |part| part.ends_with(" shown late"));
        let gaps = part("gaps", |part| part.starts_with("gaps in refreshes"));
        // The one part with a shape of its own inside it: `4200us before the blank (1700us to draw +
        // 2500us for the compositor)`, which is the lag and the two times it is made of.
        let lag = part("the lag before the blank", |part| {
            part.contains("us before the blank (")
        });
        let us_before = |mark: &str| -> i64 {
            let (before, _) = lag
                .split_once(mark)
                .unwrap_or_else(|| panic!("no {mark:?} in {lag:?}"));
            let digits: String = before
                .chars()
                .rev()
                .take_while(char::is_ascii_digit)
                .collect();
            count(&digits.chars().rev().collect::<String>()) as i64
        };
        Self {
            frames: count(frames.trim_end_matches(" frames")),
            interval_us: count(interval.trim_end_matches("us apart")) as i64,
            prepare_us: us_before("us before the blank"),
            draw_us: us_before("us to draw"),
            compose_us: us_before("us for the compositor"),
            shown_late: count(late.trim_end_matches(" shown late")),
            gaps: gaps["gaps in refreshes".len()..]
                .split_whitespace()
                .map(|bucket| {
                    let (refreshes, count_of) = bucket
                        .split_once('x')
                        .unwrap_or_else(|| panic!("{bucket:?} is not a bucket in {line:?}"));
                    (count(refreshes.trim_end_matches('+')), count(count_of))
                })
                .collect(),
        }
    }

    /// The rate the line itself says the run came out at, which is the one somebody reads off the log
    /// rather than off a scenario's own arithmetic.
    pub fn fps(&self) -> f64 {
        1_000_000.0 / self.interval_us as f64
    }

    /// How many frames the line accounted for in its buckets, which is every frame of the period.
    pub fn in_buckets(&self) -> usize {
        self.gaps.iter().map(|(_, count)| count).sum()
    }
}

/// Every `frame:` reporting line orb has written, in the order it wrote them.
///
/// One per `profile::INTERVAL` frames, and only where the launch asked for the pacing log or is at
/// `normal` or above — which is what a scenario about these has to have set.
pub fn reports(game: &Fake) -> Vec<Reported> {
    game.log()
        .lines()
        .iter()
        .filter(|line| line.contains(" frames, ") && line.contains("us apart"))
        .map(|line| Reported::of(line))
        .collect()
}

/// The last of them: the run once whatever had to settle had settled.
///
/// # Panics
/// Where none was written, since a scenario asserting on one has run far enough for one to exist.
pub fn reported(game: &Fake) -> Reported {
    reports(game).pop().unwrap_or_else(|| {
        panic!(
            "orb wrote no reporting line in this run:\n  {}",
            game.log().lines().join("\n  ")
        )
    })
}

/// Runs frames until orb has written its next account of the pacing.
///
/// What a scenario does instead of asking the pacing for a number, and the reason the numbers here are
/// read off the log at all: the `Pacing` a run is paced by is the frame loop's own, and nothing outside
/// orb holds it. So the account is waited for, one reporting period at the most, and every frame of the
/// waiting is an ordinary frame of the run.
pub fn until_reported(game: &Fake) {
    let before = reports(game).len();
    for _ in 0..=profile::INTERVAL {
        game.frame();
        if reports(game).len() > before {
            return;
        }
    }
    panic!(
        "orb wrote no account of the pacing in {} frames:\n  {}",
        profile::INTERVAL,
        game.log().lines().join("\n  ")
    );
}

/// How long orb is giving the compositor, in microseconds, off its own line about which blanks the
/// frames reached.
///
/// The allowance: how far before its blank a frame is handed over, which climbs a hundred microseconds
/// every time one misses a blank and never comes back down. Read out of the log rather than off the
/// field it is kept in, for the same reason the report line is: the log is where somebody looking into a
/// stutter finds it, and a number orb has but never says is one they cannot act on.
///
/// # Panics
/// Where orb has not written that line yet.
pub fn allowance_us(game: &Fake) -> i64 {
    const GETS: &str = "the compositor gets ";
    let lines = game.log().lines();
    let said = lines
        .iter()
        .rev()
        .find_map(|line| line.split_once(GETS))
        .unwrap_or_else(|| {
            panic!(
                "orb never said what the compositor gets:\n  {}",
                lines.join("\n  ")
            )
        })
        .1;
    let digits: String = said.chars().take_while(char::is_ascii_digit).collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("{said:?} does not start with a number of microseconds"))
}

/// The millisecond orb stamped a line with, which is the moment the line was *written* — not the moment
/// what it says was worked out. The two are a frame apart for everything the frame loop holds back.
///
/// # Panics
/// On a line with no stamp on it, every line orb writes having one.
pub fn stamped_at(line: &str) -> i64 {
    let stamp = line
        .split_once("ms] ")
        .unwrap_or_else(|| panic!("no timestamp in {line:?}"))
        .0
        .trim_start_matches('[')
        .trim();
    stamp
        .parse()
        .unwrap_or_else(|_| panic!("{stamp:?} is not a millisecond in {line:?}"))
}

/// What millisecond a tick of the simulated clock is, so that a hand-over can be put beside the stamp on
/// a line.
pub fn millis_at(tick: i64) -> i64 {
    Clock::micros_for_ticks(tick) / 1_000
}

/// How many of orb's own lines about the pacing a failure carries.
const SAID_LINES: usize = 3;

/// The last of what orb said about its own pacing, for a failure message — the report, the worsts and
/// which blanks the frames reached, which is what somebody looking into a stutter has.
pub fn last_said(game: &Fake) -> String {
    let lines = game.log().lines();
    let said: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("frame: "))
        .rev()
        .take(SAID_LINES)
        .collect();
    said.into_iter()
        .rev()
        .map(String::as_str)
        .collect::<Vec<&str>>()
        .join("\n  ")
}
