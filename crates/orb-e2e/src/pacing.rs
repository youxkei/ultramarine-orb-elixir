//! **orb's own frame loop: the shape it has, the rate it holds, and the log it writes about itself.**
//!
//! Every scenario over the one subject is here, each in a process of its own —
//! [`fake::in_its_own_process`] spawns this binary again for every `#[test]`, so a scenario owns a
//! process wherever it is written and the file it is written in owns nothing of it. One file rather
//! than twelve is `fake` compiled once instead of twelve times, and the judging below with no
//! `dead_code` allow over it: a helper nothing calls reads as dead, which twelve binaries could not
//! see. See `docs/adr/0003`.
//!
//! **How many that is has no business being written here**, and was wrong twice over while it was:
//! `cargo xtask test -p orb-e2e pacing::` counts them, and a number in a comment is stale the moment
//! anybody adds one.
//!
//! Each section keeps its own numbers, because they do not share them: `WORK_US` is 700µs in most of
//! them, 1500µs in two and 4000µs in [`budget`]'s.

use orb_core::frame;

/// One `#[test]` per row of a table, each in a process of its own.
///
/// **A row is the unit of work, because `#[test]` is.** `fake::in_its_own_process` spawns a process per
/// test and the harness runs tests across the cores, so rows looped over *inside* one test are rows nothing
/// runs in parallel.
///
/// And it is why this exists. With the three tables below as three tests, one of them — forty
/// rate-and-compose-time pairs in one process — was very nearly the whole of the file's wall clock by
/// itself, a floor no number of cores goes below however much of the rest runs in parallel. Split into a
/// test per rate, the floor became the one rate whose ceiling admits the most compose times, and below
/// that nothing but less work helps. So the rows are not split further than the reading of a failure
/// wants. Whoever wants the seconds runs the file.
///
/// What the file costs stopped being the split's to decide once two other things moved: the wait to a
/// frame's deadline became exact, so the spin ran the whole of `SPIN_US` rather than part of it, and then
/// the spin's `pause` went behind the seam and was charged for, which took a hundredfold off its
/// iterations. See
/// [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md) and
/// [docs/adr/0007](../../../docs/adr/0007-the-spins-pause-is-behind-the-seam.md).
///
/// So the spin is no longer nearly all of what this file spends its time on, and what splitting the rows
/// further would buy is a measurement nobody has taken since.
///
/// The invocation is the table, which is what the sections below want anyway: somebody with a stutter reads
/// these to find out whether their own desktop is one of the ones that breaks, and a row that fails should
/// name itself. Where the rows of one row genuinely have to be read together they stay a loop inside it —
/// see [`converges::inside_the_ceiling`], whose failure has to say which compose times are stuck.
macro_rules! a_test_per_rate {
    ($($name:ident => $body:expr,)+) => {
        $(
            #[test]
            fn $name() {
                in_its_own_process(|| $body);
            }
        )+
    };
}

// ── How a run's rate is judged ───────────────────────────────────────────────────────────────────
//
// Two readings of the same run, and both are asserted on. The rate comes off the moments the game was
// handed its frames over at — `Fake::handovers_us`, which is where its own `Present` wrote them down —
// and beside it is the `frame:` line orb writes per reporting period, taken apart by `Reported`. The
// first is what somebody playing has; the second is what somebody reading a real run's log has to be
// able to believe.
//
// Roughly the rate rather than the exact turn. The host wakes the waiting thread when it gets round to
// it and its compositor is slow now and then — see `orb-sim/src/display.rs` — so a scenario that
// asserted a turn to the microsecond would be asserting that the host is a metronome, which none is.
//
// Every one of these takes values: microseconds, the log's own lines, and the message a failure
// carries. Not the game and not the host — starting a launch, running frames and reading the host are
// the fake game's, and a judgement that could begin a run would be a second game.

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
const NEAR_SIXTY: f64 = 6.0;

/// How near sixty a second has to be to *count* as sixty, in frames a second.
///
/// Tight, unlike [`NEAR_SIXTY`], because this is not a tolerance — it is the question "was this second
/// sixty frames a second or was it not". Half a frame either way is the game running at its own speed;
/// anything more is a second that lost a refresh or gained one. What the scenarios then assert is the
/// *proportion* of seconds that were, which is the shape the answer wants: a run is not "60fps ± 6", it
/// is 60fps for so much of its length.
const AT_SIXTY: f64 = 0.5;

/// How many hosts a scenario is held against.
///
/// One is not enough: the wake delays are drawn, so a run is one of many the machine could have given.
/// A scenario that holds for one seed and not another has found something a real machine can do, which
/// is a defect and not a flake — so every assertion names the seed it failed on.
const SEEDS: u64 = 4;

/// Runs `body` against each host in turn.
fn for_each_seed(mut body: impl FnMut(u64)) {
    for seed in 0..SEEDS {
        body(seed);
    }
}

/// A second of play, in frames, which is the rate the game's own logic runs at and its timers assume.
///
/// The unit the rate is judged in, because it is the unit somebody playing notices: one turn a refresh
/// long either way is nothing, and a second at half rate is the complaint.
const A_SECOND: usize = frame::LOGIC_HZ as usize;

/// How long the rate is given to settle, in seconds.
///
/// The allowance starts at 2500µs and climbs a hundred microseconds a miss, so a display whose
/// compositor wants more than that spends its first seconds below rate — that is the climb working, not
/// a fault. What would be a fault is it still being below after a few seconds of play, which is when
/// somebody has reached the title screen and would notice.
const GRACE_SECONDS: usize = 3;

/// The turns in microseconds: how long each frame's turn was, from being handed over to the one after.
fn turns(handovers_us: &[i64]) -> Vec<i64> {
    handovers_us
        .iter()
        .zip(handovers_us.iter().skip(1))
        .map(|(before, after)| after - before)
        .collect()
}

/// The turns as counts of the compositor's refreshes, rounded.
fn refreshes(handovers_us: &[i64], period_us: i64) -> Vec<i64> {
    turns(handovers_us)
        .into_iter()
        .map(|turn| (turn + period_us / 2) / period_us)
        .collect()
}

/// The rate the run came out at, over the turns after `warm_up`.
fn fps(handovers_us: &[i64], warm_up: usize) -> f64 {
    let turns = turns(handovers_us);
    let settled = &turns[warm_up..];
    1_000_000.0 / (settled.iter().sum::<i64>() as f64 / settled.len() as f64)
}

/// The rate over each second of the run from `warm_up` on, so that a stretch at the wrong rate cannot
/// be averaged out by a stretch at the right one.
///
/// An average over the whole run is not what a player has: a run that spends a second at 30 and a
/// second at 90 averages sixty and is unplayable.
fn rate_each_second(handovers_us: &[i64], warm_up: usize) -> Vec<f64> {
    turns(handovers_us)[warm_up..]
        .chunks(A_SECOND)
        .filter(|second| second.len() == A_SECOND)
        .map(|second| 1_000_000.0 / (second.iter().sum::<i64>() as f64 / second.len() as f64))
        .collect()
}

/// What fraction of the seconds from `warm_up` on were sixty frames a second, by [`AT_SIXTY`].
fn share_at_sixty(handovers_us: &[i64], warm_up: usize) -> f64 {
    let seconds = rate_each_second(handovers_us, warm_up);
    let at = seconds
        .iter()
        .filter(|rate| (**rate - 60.0).abs() < AT_SIXTY)
        .count();
    at as f64 / seconds.len() as f64
}

/// The second that came out furthest from sixty, and how far.
fn worst_second(handovers_us: &[i64], warm_up: usize) -> (usize, f64) {
    rate_each_second(handovers_us, warm_up)
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
/// NTSC-derived ones: a display reporting 59 gets one blank a frame and runs at its own rate, which is
/// what a real NTSC-derived display did — its own rate halved, two refreshes a frame throughout, with
/// the compositor's share never once climbing off its start. That is the pacing working; a tenth of a
/// percent slow is a clock nobody can see.
///
/// All of the seconds, not most: measured over four hosts and thirty seconds apiece, every display
/// the pacing accepts holds every second — which is what the real runs show too, `gaps in
/// refreshes 2x600` with none lost. So a shortfall here is a finding rather than a flake, and the
/// seed in the message is how to go back to it.
fn assert_every_second_at(handovers_us: &[i64], wanted: f64, seed: u64, said: &str) {
    let warm_up = GRACE_SECONDS * A_SECOND;
    let seconds = rate_each_second(handovers_us, warm_up);
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
        "seed {seed}: {} of {} seconds were not {wanted} frames a second — {astray:?}\n  {said}",
        astray.len(),
        seconds.len(),
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
fn assert_mostly_sixty(handovers_us: &[i64], seed: u64, said: &str) {
    let warm_up = GRACE_SECONDS * A_SECOND;
    let share = share_at_sixty(handovers_us, warm_up);
    let over_the_run = fps(handovers_us, warm_up);
    assert!(
        share >= 0.8 && (over_the_run - 60.0).abs() < AT_SIXTY,
        "seed {seed}: {:.0}% of the seconds were sixty and the run came out at {over_the_run} — \
         \n  {said}",
        share * 100.0,
    );
}

/// A run came through the unevenness it was given: the rate itself, the share of seconds untouched, and
/// nothing collapsing further than a stage load's own arithmetic.
///
/// Three things and not one, because the unevenness is what the pacing has to survive rather than
/// something it can undo — see `Judged` in the [`holds`] section for what each of the sources costs by
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
/// - **no second collapsed further than a load's own arithmetic.** [`WORST_SECOND`] is what a
///   quarter-second frame makes of the second it lands in, and a load is the largest thing any row
///   declares, so a second below it is something else.
///
/// The share is answered rather than asserted, so that a row can hold the *worst* of its hosts to its own
/// floor rather than each host to it in turn.
fn assert_holds_through_it(handovers_us: &[i64], wanted: f64, seed: u64, said: &str) -> f64 {
    let warm_up = GRACE_SECONDS * A_SECOND;
    let seconds = rate_each_second(handovers_us, warm_up);
    let untouched: Vec<f64> = seconds
        .iter()
        .copied()
        .filter(|rate| (rate - wanted).abs() < AT_SIXTY)
        .collect();
    let worst = seconds.iter().copied().fold(f64::MAX, f64::min);
    assert!(
        !untouched.is_empty(),
        "seed {seed}: not one second was {wanted} frames a second — {seconds:?}\n  {said}",
    );
    let among_them = untouched.iter().sum::<f64>() / untouched.len() as f64;
    assert!(
        (among_them - wanted).abs() < AT_SIXTY,
        "seed {seed}: the seconds nothing touched average {among_them}, where {wanted} was asked for — \
         {seconds:?}\n  {said}",
    );
    assert!(
        worst >= WORST_SECOND,
        "seed {seed}: a second at {worst} frames, where a quarter of a second lost is {WORST_SECOND} — \
         {seconds:?}\n  {said}",
    );
    untouched.len() as f64 / seconds.len() as f64
}

/// The lowest a second may read, which is a stage load's own arithmetic: a quarter of a second spent in
/// one frame leaves the other sixty to fit in the 1.25 seconds that second became, and 60/1.25 is 48.
/// Measured at 47.8 to 48.8 over the loads the [`load`] section drives, so the bound is just under.
const WORST_SECOND: f64 = 47.0;

/// What one of orb's own `frame:` reporting lines says, taken apart.
///
/// Parsed rather than matched as text, so that a scenario says what the numbers have to be — and
/// strictly, because the shape of the line is as much of what is being held as the numbers are:
/// `Pacing::report` is what somebody looking into a stutter has in front of them, and a line that has
/// stopped saying one of these is a line they can no longer read.
#[derive(Debug)]
struct Reported {
    /// How many frames the period covered.
    frames: usize,
    /// How far apart they were handed over, smoothed 31 parts in 32 — the same reading the status
    /// line's `fps` is made of.
    interval_us: i64,
    /// How long before its blank a frame is started, in microseconds. Every one of them is input lag,
    /// which is why the line splits it into the two drawing times it is made of.
    prepare_us: i64,
    /// The game's own share of that: what its update and draw were last measured taking.
    draw_us: i64,
    /// And the compositor's: the allowance, which climbs a hundred microseconds every time a frame
    /// misses its blank and never comes back down.
    compose_us: i64,
    /// What the compositor said it could not show when it meant to, which orb's own gaps cannot see:
    /// they measure when a frame was handed over and not when it reached the screen.
    shown_late: usize,
    /// One entry per gap size that happened, in refreshes, with how many frames had it.
    gaps: Vec<(usize, usize)>,
}

impl Reported {
    /// # Panics
    /// On a line that is not one of those, naming what was missing from it.
    fn of(line: &str) -> Self {
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
    fn fps(&self) -> f64 {
        1_000_000.0 / self.interval_us as f64
    }

    /// How many frames the line accounted for in its buckets, which is every frame of the period.
    fn in_buckets(&self) -> usize {
        self.gaps.iter().map(|(_, count)| count).sum()
    }
}

/// What one of those lines is known by among the rest of the log, and the whole of how the line's own
/// spelling reaches a scenario: `Pacing::report` is the only writer of it.
const A_REPORT: &str = "us apart";

/// Every `frame:` reporting line orb has written, in the order it wrote them.
///
/// One per `profile::INTERVAL` frames, and only where the launch asked for the pacing log or is at
/// `normal` or above — which is what a scenario about these has to have set.
fn reports(lines: &[String]) -> Vec<Reported> {
    lines
        .iter()
        .filter(|line| line.contains(A_REPORT))
        .map(|line| Reported::of(line))
        .collect()
}

/// The last of them: the run once whatever had to settle had settled.
///
/// # Panics
/// Where none was written, since a scenario asserting on one has run far enough for one to exist.
fn reported(lines: &[String]) -> Reported {
    reports(lines).pop().unwrap_or_else(|| {
        panic!(
            "orb wrote no reporting line in this run:\n  {}",
            lines.join("\n  ")
        )
    })
}

/// What one of orb's own lines about which blanks the frames reached says, taken apart.
///
/// The other half of a period's account, and the half the ratchet is answerable to: [`Reported`]'s
/// `gaps` are the spacing of the hand-overs, and these are where each frame landed against the blank it
/// was aimed at. Parsed rather than matched as text for the same reason as that one — the line is what
/// somebody looking into a stutter has in front of them.
#[derive(Debug)]
struct Shown {
    /// One entry per count of refreshes past the blank aimed at that happened over the period, with how
    /// many frames were that far past it. Empty where the line said none was.
    past_the_aim: Vec<(usize, usize)>,
    /// The allowance: how far before its blank a frame is handed over, which climbs a step every time
    /// one misses a blank and never comes back down.
    allowance_us: i64,
}

impl Shown {
    /// # Panics
    /// On a line that is not one of those, naming what was missing from it.
    fn of(line: &str) -> Self {
        // The parts are semicolons: the blanks the frames reached with whatever was not charged against
        // the compositor, the frames shown early where there were any, the allowance, and the frames the
        // clock paced. `Pacing::shown` is the only writer, and a part that has moved fails here.
        let parts: Vec<&str> = line.split("; ").collect();
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
        let past = part("the blanks the frames reached", |part| {
            part.contains(PAST_THE_AIM)
        });
        let gets = part("the allowance", |part| part.starts_with(GETS));
        // Whatever the line was not charging against the compositor follows the buckets on a comma, and
        // the buckets themselves hold none.
        let buckets = past
            .split_once(PAST_THE_AIM)
            .expect("the marker just found")
            .1;
        let buckets = buckets
            .split_once(',')
            .map_or(buckets, |(before, _)| before);
        let allowance: String = gets[GETS.len()..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        Self {
            past_the_aim: buckets
                .split_whitespace()
                .filter(|bucket| *bucket != "none")
                .map(|bucket| {
                    let (refreshes, count_of) = bucket
                        .split_once('x')
                        .unwrap_or_else(|| panic!("{bucket:?} is not a bucket in {line:?}"));
                    (count(refreshes.trim_end_matches('+')), count(count_of))
                })
                .collect(),
            allowance_us: count(&allowance) as i64,
        }
    }

    /// How many frames of the period landed that many refreshes past the blank they were aimed at.
    fn past_by(&self, refreshes: usize) -> usize {
        self.past_the_aim
            .iter()
            .filter(|(past, _)| *past == refreshes)
            .map(|(_, count)| count)
            .sum()
    }
}

/// What one of those lines is known by among the rest of the log, and the whole of how its own spelling
/// reaches a scenario: `Pacing::shown` is the only writer of it.
const PAST_THE_AIM: &str = "refreshes past the blank aimed at";
/// And what the allowance is known by inside it.
const GETS: &str = "the compositor gets ";

/// Every line orb has written about which blanks the frames reached, in the order it wrote them.
///
/// One per `profile::INTERVAL` frames, and only where the launch asked for the pacing log: `report` is
/// written at `normal` as well and this one is not.
fn shown(lines: &[String]) -> Vec<Shown> {
    lines
        .iter()
        .filter(|line| line.contains(PAST_THE_AIM))
        .map(|line| Shown::of(line))
        .collect()
}

/// How many frames of the whole run landed each count of refreshes past the blank they were aimed at,
/// summed over every period orb reported: index 0 is the frames that landed on it.
///
/// Summed over the run because the allowance is not: it climbs across the periods and each line reports
/// only its own frames, so what a step is set against is every miss reported since the run began.
///
/// Frames of a period orb has not reported yet are not in here, and neither are their steps: a line
/// carries the buckets and the allowance together, so the last of them is a reading of the same moment.
fn frames_past_the_aim(lines: &[String]) -> Vec<usize> {
    let periods = shown(lines);
    let buckets = || periods.iter().flat_map(|period| period.past_the_aim.iter());
    let widest = buckets().map(|(past, _)| *past).max();
    let mut total = vec![0; widest.map_or(0, |widest| widest + 1)];
    for (past, count) in buckets() {
        total[*past] += count;
    }
    total
}

/// How long orb is giving the compositor, in microseconds, off the last of those lines.
///
/// Read out of the log rather than off the field it is kept in, for the same reason the report line is:
/// the log is where somebody looking into a stutter finds it, and a number orb has but never says is one
/// they cannot act on.
///
/// # Panics
/// Where orb has not written that line yet.
fn allowance_us(lines: &[String]) -> i64 {
    shown(lines)
        .pop()
        .unwrap_or_else(|| {
            panic!(
                "orb never said what the compositor gets:\n  {}",
                lines.join("\n  ")
            )
        })
        .allowance_us
}

/// The millisecond orb stamped a line with, which is the moment the line was *written* — not the moment
/// what it says was worked out. The two are a frame apart for everything the frame loop holds back.
///
/// # Panics
/// On a line with no stamp on it, every line orb writes having one.
fn stamped_at(line: &str) -> i64 {
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

/// What millisecond a moment of the run is, so that a hand-over can be put beside the stamp on a line.
fn millis_at(us: i64) -> i64 {
    us / 1_000
}

/// How many of orb's own lines about the pacing a failure carries.
const SAID_LINES: usize = 3;

/// The last of what orb said about its own pacing, for a failure message — the report, the worsts and
/// which blanks the frames reached, which is what somebody looking into a stutter has.
fn last_said(lines: &[String]) -> String {
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

/// A display whose rate is a whole multiple of sixty: the same number of blanks to every frame.
///
/// orb's own frame loop, driven by a game whose loop calls it — the real `configure`, `settle`,
/// `wait_for_slot` and `finished`, in the order `render` composes them, over a compositor a scenario
/// declares. What drove this before was a copy of that loop written in a harness, and nothing held the
/// copy to the original: the arithmetic was covered and the order it is asked for in was not.
mod blanks {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};

    const HZ: u32 = 120;
    /// What the game's own frame is declared to take: about what a real run's report line shows for the
    /// drawing, so the pacing is asked the question a run asks it.
    const WORK_US: i64 = 700;
    /// Long enough for the allowance to have finished climbing; the climb itself costs refreshes.
    const FRAMES: u32 = 3_000;
    const SETTLED: usize = 2_000;

    #[test]
    fn a_120hz_display_gets_two_blanks_a_frame_and_sixty_frames_a_second() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("pacing-blanks-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(FRAMES);
                let handovers = game.handovers_us();

                // What `settle` made of the display, which is the decision everything below rests on. The
                // compositor's own spacing, not the monitor's reported rate: that is the grid the frames go
                // on, and the two agree here.
                assert!(
                    game.log()
                        .said("120Hz compositor, one frame every 2 blank(s)"),
                    "seed {seed}: {:?}",
                    game.log().lines()
                );

                // Every frame reached the game's own `Present`, which is what makes the moments below a
                // reading of the whole run rather than of the part of it orb paced.
                assert_eq!(
                    handovers.len(),
                    FRAMES as usize,
                    "seed {seed}: frames handed over — {}",
                    last_said(&game.log().lines())
                );

                let rate = fps(&handovers, SETTLED);
                assert!(
                    (rate - 60.0).abs() < NEAR_SIXTY,
                    "seed {seed}: {rate} frames a second — {}",
                    last_said(&game.log().lines())
                );

                // Two blanks to a frame, near enough always. A real host loses one now and then to a late
                // wake or a slow composition; what must not happen is a turn the grid never asked for.
                let counts = refreshes(&handovers, game.refresh_period_us());
                let settled = &counts[SETTLED..];
                assert!(
                    settled.iter().all(|count| (2..=3).contains(count)),
                    "seed {seed}: turns of {settled:?} refreshes — {}",
                    last_said(&game.log().lines())
                );

                // And orb's own line about the run agreeing with the moments. Nobody debugging a stutter
                // has those; this line is what they have, so a run whose rate was right and whose account
                // of it was wrong is a run orb cannot be read about.
                let reported = reported(&game.log().lines());
                let said = reported.fps();
                assert!(
                    (said - 60.0).abs() < NEAR_SIXTY,
                    "seed {seed}: orb reported {said} frames a second over {} frames — {}",
                    reported.frames,
                    last_said(&game.log().lines())
                );
                assert_eq!(
                    reported.in_buckets(),
                    reported.frames,
                    "seed {seed}: gaps in refreshes account for {:?} of {} frames — {}",
                    reported.gaps,
                    reported.frames,
                    last_said(&game.log().lines())
                );
                // The buckets are the same claim the counts above are, made in refreshes: two to a frame,
                // with whatever the host lost put in the bucket above.
                assert!(
                    reported
                        .gaps
                        .iter()
                        .all(|(refreshes, _)| (2..=3).contains(refreshes)),
                    "seed {seed}: {:?} — {}",
                    reported.gaps,
                    last_said(&game.log().lines())
                );
                assert_eq!(
                    reported.shown_late,
                    0,
                    "seed {seed}: the compositor could not show {} of them when it meant to — {}",
                    reported.shown_late,
                    last_said(&game.log().lines())
                );
            });
        });
    }
}

/// A display nothing will say the rate of, which is the one case paced by the clock.
///
/// No compositor and no reported rate. The frames then have no blank to be put on, so the cadence is
/// kept by waiting to a grid — which is the path `wait_until` and `wait_by_clock` exist for and
/// which the blank path never reaches.
mod by_clock {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_core::profile;

    const WORK_US: i64 = 1_500;
    /// Past a reporting period, so that orb has written its own account of the run and a frame after it
    /// has drained the line: what the pacing says about itself waits for the slack on the far side of a
    /// flush, which is the `log_deferral` section's subject and the reason this is not exactly a period.
    const FRAMES: u32 = profile::INTERVAL + 1;

    #[test]
    fn a_display_that_will_not_say_its_rate_is_paced_by_the_clock() {
        in_its_own_process(|| {
            let game = Fake::attach_watching_the_pacing(
                Display::unknown(),
                "by-the-clock",
                Work::flat(WORK_US),
            );
            game.frames(FRAMES);
            let handovers = game.handovers_us();

            assert!(
                game.log().said(
                    "an unknown monitor and the compositor will not say; pacing by the clock"
                ),
                "{:?}",
                game.log().lines()
            );

            // Sixty frames a second, held by the clock rather than by the blanks. Not to the tick: the wait
            // ends by spinning, so it overshoots by however long one look at the counter takes.
            let rate = fps(&handovers, 0);
            assert!((rate - 60.0).abs() < 0.5, "{rate} frames a second");

            // And said to be the clock's, which is what stops somebody reading the gaps above as blanks:
            // every frame of the period, since there was never a blank for one of them to have missed.
            //
            // One more than the period's frames, and both counts are right. The line is written from inside
            // a frame's own update, so that frame has taken its turn by the clock and has not been handed
            // over yet: it is counted here and will be counted among the next period's frames.
            let reported = reported(&game.log().lines());
            assert!(
                game.log().said(&format!(
                    "{} frame(s) paced by the clock",
                    reported.frames + 1
                )),
                "{} frames reported — {}",
                reported.frames,
                last_said(&game.log().lines())
            );
        });
    }
}

/// A stage load, and what the allowance must not do about it.
///
/// A load is a frame that takes a quarter of a second, and the frames around it miss their blanks
/// because of it. The compositor had nothing to do with that, so none of it may reach the allowance —
/// `measure_compose`'s own comment records what happens when it does: a run "ratchets to seven
/// milliseconds of lag by the third stage", which is a quarter of a frame of input lag bought for
/// nothing and never given back.
///
/// So three loads here rather than one, since the third stage is where it was noticed.
mod load {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    const HZ: u32 = 120;
    const WORK_US: i64 = 700;
    /// What `budget_ceiling`'s comment calls the frame that must not be allowed to set anything: a stage
    /// load, a quarter of a second of it.
    const LOAD_US: i64 = 250_000;
    /// A compositor comfortably inside the 2500µs orb starts by allowing, so that any climb at all is the
    /// load's doing and not the compositor's.
    const QUICK_COMPOSE_US: i64 = 1_000;

    #[test]
    fn three_stage_loads_buy_the_compositor_nothing() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ);
                display.compose = Compose::flat(QUICK_COMPOSE_US);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("load-{seed}"),
                    Work::flat(WORK_US),
                );

                // Far enough in that orb has said what it is allowing, which is where the allowance is read
                // from. Two seconds was enough when a harness held the pacing itself; being told takes a
                // reporting period.
                game.frames_until_the_log_holds_another(A_REPORT);
                let before = allowance_us(&game.log().lines());
                // From the first load on: what the loads did to the rate is a claim about the seconds after
                // them, and the settling above is not part of it.
                let settling = game.handovers_us().len();

                for _ in 0..3 {
                    game.frame_takes(Work::flat(LOAD_US));
                    game.frame();
                    game.frame_takes(Work::flat(WORK_US));
                    game.frames(3 * A_SECOND as u32);
                }

                game.frames_until_the_log_holds_another(A_REPORT);
                let allowed = allowance_us(&game.log().lines());
                assert_eq!(
                    allowed,
                    before,
                    "seed {seed}: the allowance climbed from {before}us to {allowed}us over three loads, and \
                     the compositor was taking {QUICK_COMPOSE_US}us throughout — {}",
                    last_said(&game.log().lines())
                );

                // And the damage stops at the second the load is in. Measured: a load's own second reads 47.8
                // to 48.8 — sixty frames in the 1.25 seconds a quarter-second load makes of it, which is
                // arithmetic and not pacing — and every other second of the run is exactly sixty. So what is
                // asserted is that no more seconds than there were loads are off at all, and that the ones that
                // are lost the load and nothing besides.
                let handovers = game.handovers_us();
                let off: Vec<f64> = rate_each_second(&handovers[settling..], 0)
                    .into_iter()
                    .filter(|rate| (rate - 60.0).abs() >= AT_SIXTY)
                    .collect();
                assert!(
                    off.len() <= 3,
                    "seed {seed}: {} seconds off sixty for three loads — {off:?} — {}",
                    off.len(),
                    last_said(&game.log().lines())
                );
                for rate in &off {
                    assert!(
                        *rate >= 47.0,
                        "seed {seed}: a second at {rate} frames, where a quarter of a second lost is 48 — \
                         {off:?} — {}",
                        last_said(&game.log().lines())
                    );
                }
            });
        });
    }
}

/// The game's window on one monitor while the compositor times another.
///
/// The worklist's *What the fixed stutter costs* named this case and said of it that "that refusal has
/// not been seen happen", and that the hazard beside it is one "nothing has re-checked since". It was
/// both: the refusal never fired, and what happened instead ran the game **fast** — measured on a real
/// mixed-rate desktop, period after period above sixty with `0 frame(s) paced by the clock` in the same
/// run, at the ratio the two rates come to — because the frames went
/// on the compositor's blanks while the
/// cadence was counted in the monitor's. Music and every timer in the game are counted in its own
/// frames, so a run like that is a run at the wrong speed.
///
/// It is not a limit and never was. `DwmFlush` returns at the compositor's blanks and at nobody
/// else's, whatever monitor the window is on — measured on a real mixed-rate desktop, and what it
/// decided is beside `frame::Pacing::grid` — so that grid is the one a frame can be put on, and counting the
/// cadence in anything else is the fault. The grid is the compositor's now, and a 144Hz compositor is
/// paced the way any 144Hz display is: one frame on whichever of its blanks is nearest each sixtieth.
mod disagrees {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};

    /// What the monitor the window is on reports, and a whole multiple of sixty — which is what used to
    /// decide the cadence and now decides nothing.
    const MONITOR_HZ: u32 = 120;
    /// What the compositor is timing, which is another monitor of the desktop, and not a whole multiple.
    const COMPOSITOR_HZ: u32 = 144;
    const WORK_US: i64 = 1_500;

    /// Fifteen seconds: a few for the allowance to settle and a dozen to hold the rate through.
    const FRAMES: u32 = 15 * A_SECOND as u32;

    #[test]
    fn a_monitor_the_compositor_is_not_timing_is_still_paced_at_sixty() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::split(MONITOR_HZ, COMPOSITOR_HZ);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("disagrees-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(FRAMES);

                // The grid taken is the compositor's, and named as the compositor's: 144Hz is not a
                // multiple of sixty, so it is the same fractional path a 144Hz monitor takes.
                assert!(
                    game.log().said(
                        "144Hz compositor is not a multiple of 60Hz; one frame on whichever blank is nearest each sixtieth"
                    ),
                    "seed {seed}: {:?}",
                    game.log().lines()
                );

                // And the desktop it is happening on is said once, because a frame shown on the
                // compositor's blank still has the window's own panel to reach — which is the part of this
                // no simulator can speak for, and the reason a run on the machine is still wanted.
                assert!(
                    game.log().said(
                        "the window's own monitor is 120Hz, which is not what the compositor is timing"
                    ),
                    "seed {seed}: {:?}",
                    game.log().lines()
                );

                // Sixty frames a second in every second past the grace, which is the whole of the fix: the
                // rate used to come out above sixty here and on a real desktop alike, the game running
                // fast by the ratio of the two rates.
                assert_every_second_at(
                    &game.handovers_us(),
                    60.0,
                    seed,
                    &last_said(&game.log().lines()),
                );

                // On the blanks throughout. The old code wrote `pacing by the clock` and then paced by the
                // blanks anyway — two lines of one run contradicting each other — and neither is written
                // now: the frames are on the blanks and the log says so.
                assert!(
                    !game.log().said("pacing by the clock"),
                    "seed {seed}: {}",
                    last_said(&game.log().lines())
                );
                assert!(
                    game.log().said("0 frame(s) paced by the clock"),
                    "seed {seed}: {}",
                    last_said(&game.log().lines())
                );
            });
        });
    }
}

/// A frame whose work does not fit the budget it was started against, and the budget finding its
/// size.
///
/// `prepare_us` is how long before its blank a frame starts, and every microsecond of it is input
/// lag. It is measured rather than chosen, so a frame that overran has to raise it and the cadence
/// has to come right and stay right. `next_budget` is tested on its own arithmetic; this is the loop
/// doing it.
mod budget {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    const HZ: u32 = 120;
    /// More than the 4000µs the budget starts at once the compositor's share is added, so the first
    /// frame cannot make the blank it was aimed at.
    const WORK_US: i64 = 4_000;
    /// What orb starts by allowing the compositor, before any frame has missed a blank.
    const ALLOWED_AT_FIRST: i64 = 2_500;
    /// How much longer a frame's own work reads back as than the game put into it.
    ///
    /// The frame loop reads the counter seven times between its marks and the simulated one moves a tick per
    /// read, so a span the game spent 4000µs in is measured at 4002. What must not drift is the span being
    /// the game's work and nothing else, which is what the bound is for rather than the exact microsecond.
    const READS_US: i64 = 10;

    #[test]
    fn a_frame_that_overran_raises_the_budget_and_the_cadence_comes_right() {
        in_its_own_process(|| {
            let mut display = Display::agreed(HZ);
            // Flat, so that what the budget is answering to is the frame's own work and not a spike.
            display.compose = Compose::flat(1_000);
            display.metronome = true;
            let game = Fake::attach_watching_the_pacing(display, "budget", Work::flat(WORK_US));
            game.frames(600);
            let handovers = game.handovers_us();
            let counts = refreshes(&handovers, game.refresh_period_us());

            // The first turn is three refreshes: the frame was started 4000µs before its blank, spent 4000µs of
            // that on itself, and the compositor still wanted its share — so it was shown at the refresh after
            // the one it asked for.
            assert_eq!(counts[0], 3, "{counts:?}");

            // And every turn after it is two, because the budget was raised to cover what the frame really
            // takes. One miss and no more, which is what makes a stutter a stutter rather than a rate.
            assert!(
                counts[1..].iter().all(|count| *count == 2),
                "{counts:?} — {}",
                last_said(&game.log().lines())
            );

            // What it settled at: the frame's own 4000µs plus what the compositor is being given. Read off the
            // line the run reports rather than the field, because that line is what somebody debugging a
            // stutter actually has.
            game.frames_until_the_log_holds_another(A_REPORT);
            let reported = reported(&game.log().lines());
            assert!(
                (WORK_US..=WORK_US + READS_US).contains(&reported.draw_us),
                "the frame's own work reads back as {}us against the {WORK_US}us it takes — {}",
                reported.draw_us,
                last_said(&game.log().lines())
            );
            assert_eq!(
                reported.compose_us,
                ALLOWED_AT_FIRST,
                "the compositor is being given {}us, and nothing here asked it for more — {}",
                reported.compose_us,
                last_said(&game.log().lines())
            );
            assert_eq!(
                reported.prepare_us,
                reported.draw_us + reported.compose_us,
                "the lag is not the two times it is said to be made of — {}",
                last_said(&game.log().lines())
            );

            // One frame late in six hundred, and named as one whose drawing overran rather than one the
            // compositor was short-changed on — different faults, and giving the compositor longer would not
            // have touched this one.
            assert!(
                game.log().said("1x1") && game.log().said("whose drawing overran"),
                "{}",
                last_said(&game.log().lines())
            );
        });
    }

    /// The usual frame beside the spike below: about what a real run's report line shows for the game's
    /// own drawing, so the spike is being set against an ordinary frame and not against nothing.
    const QUIET_WORK_US: i64 = 700;

    /// One frame's own work, where the sound it started took most of it.
    ///
    /// Declared from what a real run showed rather than invented: on the frame a spell card starts,
    /// `PLAY_SOUNDS.call()` is most of the frame's whole span — the `sound` figure on its own `--pacing`
    /// line — and that happens a handful of times in a session, the worst of them worse than this.
    ///
    /// Spent with the rest of that frame's work rather than through [`Work::sound_us`], because what it
    /// is here for is a spike of this size and what the budget does with the frames after one. Where
    /// inside the frame it falls is the [`sound`](super::sound) section's subject.
    const SOUND_SPIKE_US: i64 = 9_000;

    /// One frame in this many carries it, so a run of this length sees a handful.
    const SPIKE_ONE_IN: i64 = 300;

    /// A spike the ceiling admits must not leave the frames after it handed over before the blank
    /// *before* the one they are aimed at, because that is the game running at double speed.
    ///
    /// The chain, measured on a real run and the reason this exists: a frame whose sound took most of it
    /// came to a whole frame's worth of work, which was inside the ceiling — so `next_budget` believed it
    /// and `prepare_us` went to it. The frames after that one do almost no work, so each was handed over
    /// nearly the whole budget before its blank, and the blank one refresh earlier was nearer than the
    /// compositor's share: it took them *there*. The anchor moved a refresh early, the next frame went as
    /// early again, and the turns came out a refresh apart instead of two. orb's own line said so where a
    /// period ended inside one: `shown a refresh or more early, so the game ran fast for them`.
    ///
    /// **The claim is what the screen got, and not the spacing of the handovers.** A spike's own frame
    /// is shown a refresh late whatever is done — it started against a budget the frames before it set,
    /// so its drawing outgrew its turn and no pacing can undo that — and the frame after it comes back
    /// onto the grid. The gap between those two handovers is then a refresh shorter than the cadence by
    /// arithmetic: 5800µs late plus 16666µs of cadence is 10866, which is 1.3 refreshes. Both frames
    /// reached the blanks they belong to, so a `1x` turn there is not the game running fast, and
    /// asserting on the turns alone reads it as if it were.
    ///
    /// What says the game ran fast is orb's own count of frames the compositor took a refresh early,
    /// which is the fault this is about and is asserted at zero. Beside it, that the run neither gains
    /// nor loses frames: one turn over the cadence and one under it per spike, and sixty frames a
    /// second over the run.
    #[test]
    fn a_spike_the_ceiling_admits_does_not_hand_the_frames_after_it_over_early() {
        in_its_own_process(|| {
            let mut display = Display::agreed(HZ);
            display.compose = Compose::flat(1_000);
            display.metronome = true;
            let game = Fake::attach_watching_the_pacing(
                display,
                "budget-spike",
                Work {
                    spike_us: SOUND_SPIKE_US,
                    spike_one_in: SPIKE_ONE_IN,
                    ..Work::flat(QUIET_WORK_US)
                },
            );
            game.frames(1_200);
            game.frames_until_the_log_holds_another(A_REPORT);

            // The one that decides, and the one the fault showed up as: orb's own count of frames the
            // compositor took at the blank before the one they were aimed at.
            assert!(
                !game.log().said("shown a refresh or more early"),
                "{}",
                last_said(&game.log().lines())
            );

            let handovers = game.handovers_us();
            let counts = refreshes(&handovers, game.refresh_period_us());
            let under = counts.iter().filter(|count| **count < 2).count();
            let over = counts.iter().filter(|count| **count > 2).count();
            assert_eq!(
                under,
                over,
                "{under} turn(s) under the cadence against {over} over it, so the run gained or lost \
                 frames rather than paying a spike back — {}",
                last_said(&game.log().lines())
            );
            let rate = fps(&handovers, 0);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "{rate} frames a second — {}",
                last_said(&game.log().lines())
            );
            // The spike's own frame loses a refresh, and it did so before the ceiling was lowered too —
            // measured over these 1200 turns, `{1: 42, 2: 1155, 3: 3}` without it against `{2: 1197, 3:
            // 3}` with, the threes at the same three frames both times. A budget is a prediction and a
            // spike nothing has seen yet cannot be one, so that frame is not what the ceiling changed:
            // what it changed is the 42.
            let over: Vec<(usize, i64)> = counts
                .iter()
                .enumerate()
                .filter(|(_, count)| **count > 3)
                .map(|(at, count)| (at, *count))
                .collect();
            assert!(
                over.is_empty(),
                "turn(s) past a lost refresh, at {over:?} — {}",
                last_said(&game.log().lines())
            );

            // And orb charges it to the frame's own drawing rather than to the compositor, which is what
            // keeps the allowance where it is: giving the compositor longer would not have made a frame
            // whose work outgrew its budget arrive any earlier.
            assert!(
                game.log().said("whose drawing overran"),
                "{}",
                last_said(&game.log().lines())
            );
            assert_eq!(
                allowance_us(&game.log().lines()),
                ALLOWED_AT_FIRST,
                "a spike bought the compositor time — {}",
                last_said(&game.log().lines())
            );
        });
    }

    /// Work that is heavy on *every* frame is covered, up to most of a game frame.
    ///
    /// Which is the other half of the budget's job and the half a ceiling of one refresh gave up. The
    /// budget exists so that a frame whose work does not fit the tail of its turn is started earlier;
    /// held under a refresh, the most it can start a frame is a refresh before its blank, so work past
    /// a refresh less the compositor's share cannot be covered at all and every frame of it misses.
    /// Swept at 120Hz with 2500µs allowed, that put 5500µs a frame on the cadence and 6000µs on three
    /// refreshes apiece — 40 frames a second for the whole run, from work a fifth of a game frame long.
    ///
    /// The rows are the sweep, and they run either side of that edge on purpose: 4000µs is what
    /// `a_frame_that_overran_raises_the_budget_and_the_cadence_comes_right` already drives, and 9000µs
    /// is the sound spike's own size asked for every frame instead of one in three hundred. What holds
    /// them all is the budget being allowed to grow to what the work takes, which is what it is for.
    #[test]
    fn work_that_is_heavy_every_frame_is_covered() {
        in_its_own_process(|| {
            let mut missed = Vec::new();
            for work_us in [4_000i64, 5_500, 6_000, 7_000, 9_000] {
                let mut display = Display::agreed(HZ);
                display.compose = Compose::flat(1_000);
                display.metronome = true;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("budget-sustained-{work_us}"),
                    Work::flat(work_us),
                );
                game.frames(600);
                let counts = refreshes(&game.handovers_us(), game.refresh_period_us());
                // Past the first, which is the one the budget has not been raised for yet — the frame
                // that teaches it what the work costs, and the claim
                // `a_frame_that_overran_raises_the_budget_and_the_cadence_comes_right` makes on its own.
                let off = counts[1..].iter().filter(|count| **count != 2).count();
                if off > 0 {
                    missed.push(format!(
                        "{work_us}us: {off} of {} turns off the cadence\n    {}",
                        counts.len() - 1,
                        last_said(&game.log().lines())
                    ));
                }
            }
            assert!(
                missed.is_empty(),
                "work the budget should cover and does not:\n  {}",
                missed.join("\n  ")
            );
        });
    }

    /// And over a real host the rate still comes right, which is the part that matters to a run.
    #[test]
    fn the_rate_comes_right_over_a_real_host_too() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("budget-host-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(3_000);
                let rate = fps(&game.handovers_us(), 2_000);
                assert!(
                    (rate - 60.0).abs() < NEAR_SIXTY,
                    "seed {seed}: {rate} frames a second — {}",
                    last_said(&game.log().lines())
                );
            });
        });
    }
}

/// Where the frame loop calls `PlaySounds`, and what the frame that starts a spell card pays for it.
///
/// The sounds are the game's own call, and the frame loop makes it where the game's own loop does:
/// between the update and the draw, inside the span the frame has to reach its blank in. On the frame a
/// spell card starts that call is most of the frame's whole span — the `sound` figure of a `--pacing`
/// line — so what it costs is a question about the frame the card starts on and not about a rate.
///
/// Which is why the card is a frame the scenario names rather than one in three hundred: the sound is
/// declared for one frame, through [`Work::sound_us`], which the fake game spends inside `PlaySounds`
/// and nowhere else. The turns from that frame on are then read off directly, and which frame paid is
/// the answer rather than something averaged away.
///
/// **What it is for is the placement**, and `runtime::render` is where the reason that placement is
/// what it is stands: moving the call past `Present` moves the lost refresh from the card's own frame
/// to the frame after it, and the frame it moves to is one whose drawing did not overrun — so the
/// allowance climbs for it. Moving the call and running this is how that is measured again.
mod sound {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    const HZ: u32 = 120;
    /// What the frames either side of the card's do, which is what a real run's report line shows for
    /// the game's own drawing.
    const WORK_US: i64 = 700;
    /// What the sound the card starts costs, from the run in
    /// [docs/adr/0011](../../../docs/adr/0011-the-frame-is-held-for-the-blank-before-the-one-it-is-aimed-at.md):
    /// a `PLAY_SOUNDS` that ran 8438µs, and the worst of a session is worse.
    ///
    /// Inside `budget_ceiling`, which at 120Hz is 12500µs, so this is a frame the budget is allowed to
    /// answer to rather than one the ceiling refuses.
    const SOUND_US: i64 = 9_000;
    /// A second of play after the card, so the frames following it are judged over a stretch the
    /// scenario names rather than over whatever is left of orb's reporting period.
    const AFTER: u32 = A_SECOND as u32;

    /// The card's own frame loses its blank, and what the run pays for it stops there.
    ///
    /// **The frame cannot keep it wherever the call is made.** It began against a budget the frames
    /// before it left, a budget is a prediction, and a sound nothing has heard yet is not one it can
    /// have made — so the work outgrows the turn and the compositor takes the frame at the refresh
    /// after the one it asked for. What the placement decides is only which frame that is.
    ///
    /// So what is asserted is where the cost lands and that nothing follows it: the lost refresh is the
    /// card's own, the frame after it comes straight back onto the grid, and the two readings that
    /// would say the run itself had been made worse — the compositor's allowance, which never comes
    /// back down, and frames handed over before the blank before the one they are aimed at — say
    /// nothing.
    #[test]
    fn the_sound_a_frame_starts_costs_that_frame_its_blank_and_no_lag_after_it() {
        in_its_own_process(|| {
            let mut display = Display::agreed(HZ);
            // A metronome, and the compositor flat inside its allowance: the claim is about the turns of
            // named frames, and a host that wakes late loses a refresh of its own now and then, which
            // this would read as the sound's.
            display.compose = Compose::flat(1_000);
            display.metronome = true;
            let game = Fake::attach_watching_the_pacing(display, "sound", Work::flat(WORK_US));

            // Far enough in for the budget to have settled on what an ordinary frame takes, which is
            // what the card's frame then begins against, and for orb to have said what it is allowing
            // the compositor.
            game.frames_until_the_log_holds_another(A_REPORT);
            let quiet = game.handovers_us().len();
            let allowed_before = allowance_us(&game.log().lines());

            game.frame_takes(Work {
                sound_us: SOUND_US,
                ..Work::flat(WORK_US)
            });
            game.frame();
            game.frame_takes(Work::flat(WORK_US));
            game.frames(AFTER);
            game.frames_until_the_log_holds_another(A_REPORT);

            // The card's own turn is the first of these and the frames after it are the rest.
            let handovers = game.handovers_us();
            let counts = refreshes(&handovers[quiet - 1..], game.refresh_period_us());
            let said = last_said(&game.log().lines());
            assert_eq!(
                counts[0], 3,
                "the card's own frame took {} refreshes — {said}",
                counts[0]
            );

            // And the frame after it is back where the grid always meant it to be, which makes its turn
            // a refresh *shorter* than the cadence rather than longer: the card's frame was handed over
            // late and this one at its own blank, so the gap between the two is short by however late
            // the first was. The two turns come out 25666µs and 10229µs against a cadence of 16666,
            // which the report line carries as its worst and its best. Both frames reached the blank
            // they belong to, so a turn of one here is not the game running fast — see `docs/adr/0011`,
            // and the count of frames shown early below, which is the reading that would say otherwise.
            assert_eq!(
                counts[1], 1,
                "the frame after the card took {} refreshes — {said}",
                counts[1]
            );
            let off: Vec<(usize, i64)> = counts[2..]
                .iter()
                .enumerate()
                .filter(|(_, count)| **count != 2)
                .map(|(at, count)| (at + 2, *count))
                .collect();
            assert!(
                off.is_empty(),
                "turn(s) off the cadence past the card's own frame and the one after it: {off:?} — \
                 {said}"
            );

            // The miss is charged to the frame's own drawing, which is what keeps the allowance where it
            // is: the sound was inside the span that frame had to reach its blank in, so no amount of
            // time given to the compositor would have got it there any earlier.
            assert!(game.log().said("whose drawing overran"), "{said}");
            let allowed = allowance_us(&game.log().lines());
            assert_eq!(
                allowed, allowed_before,
                "the sound took the allowance from {allowed_before}us to {allowed}us, which is input \
                 lag for the rest of the run — {said}"
            );

            // Nor did the frames after it go early, which is what the budget the sound raised would
            // otherwise do to frames that start no sound of their own — a refresh or more before their
            // blank, where the compositor takes them at the blank before it and the game runs fast.
            assert!(!game.log().said("shown a refresh or more early"), "{said}");

            // And the budget it raised has come back down: the report line's own figure for the game's
            // drawing, against the 9000µs the sound put into one frame. Not to the microsecond, that
            // being however many frames of the period fell after the card, but nowhere near the sound.
            let reported = reported(&game.log().lines());
            assert!(
                reported.draw_us < 2 * WORK_US,
                "the lag still holds {}us of the drawing, where a quiet frame takes {WORK_US}us — \
                 {said}",
                reported.draw_us
            );
        });
    }
}

/// A display whose rate is not a whole multiple of sixty: two refreshes, three, two, three.
///
/// At 144Hz a frame is 2.4 refreshes, so there is no one count to keep. Each frame goes on whichever
/// blank is nearest where a sixtieth-of-a-second grid has got to.
///
/// The pattern is asserted on a metronome host, because it is a claim about the grid's arithmetic; that
/// the rate comes out at sixty over a real one is the `converges` section.
mod fractional {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    const HZ: u32 = 144;
    const WORK_US: i64 = 700;
    const FRAMES: u32 = 400;
    /// The grid takes a few frames to come up, and the budget a few more.
    const WARM_UP: usize = 100;

    #[test]
    fn the_grid_alternates_two_refreshes_and_three_in_the_pattern_that_averages_two_point_four() {
        in_its_own_process(|| {
            let mut display = Display::agreed(HZ);
            display.metronome = true;
            display.compose = Compose::flat(1_000);
            let game =
                Fake::attach_watching_the_pacing(display, "fractional-grid", Work::flat(WORK_US));
            game.frames(FRAMES);

            assert!(
                game.log().said(
                    "144Hz compositor is not a multiple of 60Hz; one frame on whichever blank is nearest each sixtieth"
                ),
                "{:?}",
                game.log().lines()
            );

            let counts = refreshes(&game.handovers_us(), game.refresh_period_us());
            let settled = &counts[WARM_UP..];
            assert!(
                settled.iter().all(|count| *count == 2 || *count == 3),
                "{settled:?}"
            );

            // Twelve refreshes to every five frames, wherever the five are taken from. Being true of *every*
            // window rather than of the average is the self-correcting part: the grid is a moment and not an
            // accumulated count, so a frame put on the nearer blank does not push the ones after it.
            for (at, window) in settled.windows(5).enumerate() {
                assert_eq!(
                    window.iter().sum::<i64>(),
                    12,
                    "five frames from {at} took {window:?} refreshes"
                );
            }
        });
    }

    /// And over a real host it is still sixty frames a second, and still only twos and threes — a turn of
    /// one or four would be a frame the grid never asked for, whatever a late wake did to it.
    #[test]
    fn a_late_wake_leaves_the_rate_at_sixty_and_the_counts_where_the_grid_put_them() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("fractional-host-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(3_000);
                let handovers = game.handovers_us();
                let rate = fps(&handovers, 2_000);
                assert!(
                    (rate - 60.0).abs() < NEAR_SIXTY,
                    "seed {seed}: {rate} frames a second — {}",
                    last_said(&game.log().lines())
                );
                let counts = refreshes(&handovers, game.refresh_period_us());
                let settled = &counts[2_000..];
                assert!(
                    settled.iter().all(|count| (2..=3).contains(count)),
                    "seed {seed}: {settled:?} — {}",
                    last_said(&game.log().lines())
                );
            });
        });
    }
}

/// A compositor that starts taking longer, and the time it is given following it.
///
/// What the compositor needs is not a threshold but a distribution, and orb's answer is to ratchet: a
/// frame a refresh past the blank it was aimed at adds `MISS_STEP_US` to what the compositor is given,
/// and nothing takes it back down. A compositor that slows under load is the case that ratchet exists
/// for, and it is one no test could make happen — a real one is not asked to be slow.
///
/// **Which frames it steps for is the second half of the section**, because a step is never given back:
/// the last three rows are a compositor with room to spare, one whose composition outlasts a refresh, and
/// one whose spikes are a refresh late, and between them they say the ratchet steps for the third of those
/// and for neither of the others.
mod compose {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    const HZ: u32 = 120;
    /// What it takes once something else on the desktop wants the GPU: more than the 2500µs orb starts by
    /// allowing, so frames begin missing until the allowance has caught up.
    const SLOW_COMPOSE_US: i64 = 4_000;
    const WORK_US: i64 = 700;
    /// What orb starts by allowing the compositor, before any frame has missed a blank.
    const ALLOWED_AT_FIRST: i64 = 2_500;
    /// And what it adds to that for each frame it saw miss its blank, which is `MISS_STEP_US`.
    ///
    /// Here because the three rows below are about *which* frames it adds it for, and a row saying so has
    /// to count the steps: the allowance a period reports is where it started plus one of these for every
    /// miss charged since, so the two together say which of the period's misses were charged and which
    /// were not.
    const A_STEP: i64 = 100;

    /// A compositor that spikes several times in the run, which is the case the allowance's ratchet is for.
    ///
    /// `Compose::spiking` and not the default: what the default's rarity is set from is half an hour of play,
    /// and over fifteen seconds that is no spikes at all — a path nothing reaches is a path nothing tests.
    /// Here it is about one in three hundred frames, so a run of nine hundred sees a handful.
    #[test]
    fn a_handful_of_spikes_raises_the_allowance_and_the_rate_still_holds() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ);
                display.compose = Compose::spiking();
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("spikes-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(15 * A_SECOND as u32);

                // The spikes were seen: the allowance climbed off the 2500µs it starts at, which only a frame
                // missing its blank does.
                game.frames_until_the_log_holds_another(A_REPORT);
                let allowed = allowance_us(&game.log().lines());
                assert!(
                    allowed > ALLOWED_AT_FIRST,
                    "seed {seed}: the allowance never moved, so no spike cost a frame its blank — {}",
                    last_said(&game.log().lines())
                );

                // And the rate is sixty all the same, which is what having an allowance is for. Most of the
                // seconds rather than all: the second a spike lands in loses a refresh, and that is the dip the
                // allowance then stops happening again.
                assert_mostly_sixty(&game.handovers_us(), seed, &last_said(&game.log().lines()));
            });
        });
    }

    /// The same spikes at every rate anyone plays at, because what a spike costs is not the same at all of
    /// them and the answer above is 120Hz's alone.
    ///
    /// **What is asserted is loose on purpose.** The spike rate here is three hundred times the measured
    /// one, so the damage is inflated by the same factor and any tight bound would be a fact about the
    /// inflation rather than about the pacing. What holds regardless is that a spike costs its own second and
    /// not the run: the rate stays inside a frame a second, and most seconds are untouched.
    ///
    /// Measured, over these four seeds — and the spread is the finding rather than the numbers:
    ///
    /// | | worst second | seconds off sixty, of eleven | the allowance |
    /// | --- | --- | --- | --- |
    /// | 60Hz | 58.08 | 1 to 4 | 2500–2900µs |
    /// | 120Hz | 59.03 | 0 to 1 | 2500–2900µs |
    /// | 144Hz, 165Hz | 59.93 | none | 2500–2900µs |
    /// | 240Hz | 59.52 | none | 2500–2900µs |
    ///
    /// **60Hz is the rate with no room.** A missed blank there costs a whole refresh, which is a whole frame,
    /// and the frame after it cannot come early enough to make it back — so the worst second reads 58.08 and
    /// one seed lost four of its eleven. From 144Hz up the same spike is absorbed and no second is off at all.
    ///
    /// **What it costs the allowance is the same at every one of them**, seed for seed: a spike is a
    /// composition the allowance was short for, wherever the refreshes fall, so each one buys exactly one
    /// step and the column above is four rates reporting the same four numbers. What a rate decides is what
    /// the screen made of the spike and not what orb was told about it. The seed that reports 2500 is a
    /// stream that drew no spike inside its fifteen seconds at all, which is what the rarity in
    /// `Compose::spiking` is for.
    #[test]
    fn what_a_spike_costs_depends_on_the_rate_and_never_on_the_run() {
        in_its_own_process(|| {
            for hz in [60u32, 120, 144, 165, 240] {
                for_each_seed(|seed| {
                    let mut display = Display::agreed(hz);
                    display.compose = Compose::spiking();
                    display.seed = seed;
                    let game = Fake::attach_watching_the_pacing(
                        display,
                        &format!("spike-cost-{hz}-{seed}"),
                        Work::flat(WORK_US),
                    );
                    game.frames(15 * A_SECOND as u32);
                    let handovers = game.handovers_us();
                    let warm_up = GRACE_SECONDS * A_SECOND;

                    let rate = fps(&handovers, warm_up);
                    assert!(
                        (rate - 60.0).abs() < 1.0,
                        "{hz}Hz, seed {seed}: {rate} frames a second over the run — {}",
                        last_said(&game.log().lines())
                    );

                    let share = share_at_sixty(&handovers, warm_up);
                    let (at, worst) = worst_second(&handovers, warm_up);
                    assert!(
                        share >= 0.5,
                        "{hz}Hz, seed {seed}: only {share} of the seconds were sixty, worst {worst} at second \
                         {at} — {}",
                        last_said(&game.log().lines())
                    );
                });
            }
        });
    }

    #[test]
    fn a_compositor_that_slows_is_given_longer_until_the_frames_land_again() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ);
                display.compose = Compose::flat(1_000);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("slows-{seed}"),
                    Work::flat(WORK_US),
                );

                // Nothing has missed, so the allowance is still what it started at.
                game.frames_until_the_log_holds_another(A_REPORT);
                let allowed = allowance_us(&game.log().lines());
                assert_eq!(
                    allowed,
                    ALLOWED_AT_FIRST,
                    "seed {seed}: the compose time orb starts by allowing — {}",
                    last_said(&game.log().lines())
                );

                // The desktop gets busy.
                let quick = game.handovers_us().len();
                game.sim()
                    .display()
                    .set_compose(Compose::flat(SLOW_COMPOSE_US));
                game.frames(2_000);

                // The allowance was raised past what the compositor now really takes. Not to it exactly — the
                // ratchet is a hundred microseconds a miss and it stops as soon as the frames land — so what is
                // asserted is that it covers it.
                let allowed = allowance_us(&game.log().lines());
                assert!(
                    allowed >= SLOW_COMPOSE_US,
                    "seed {seed}: {allowed}us allowed for a compositor taking {SLOW_COMPOSE_US}us — {}",
                    last_said(&game.log().lines())
                );

                // And having caught up, the rate is sixty again — the misses were the climb and not a rate.
                let handovers = game.handovers_us();
                let rate = fps(&handovers[quick..], 1_500);
                assert!(
                    (rate - 60.0).abs() < NEAR_SIXTY,
                    "seed {seed}: {rate} frames a second after the climb — {}",
                    last_said(&game.log().lines())
                );
            });
        });
    }

    /// What the compositor takes in the rows below where its cost is not the thing being varied: far
    /// enough inside the allowance that no wake delay the host draws can put a frame past its blank.
    ///
    /// The room is what matters and not the number. A frame is handed over [`ALLOWED_AT_FIRST`] before the
    /// blank it is aimed at and composed this long after that, so what is left over is the room, and a
    /// flush woken late moves the *anchor* the next aim is measured from by up to `orb_sim::SPIKE_US`. So a
    /// run whose room is wider than that worst wake cannot lose a refresh at all, whatever the host does.
    const ROOMY_COMPOSE_US: i64 = 1_000;
    /// Held rather than remembered, since either number could be changed without the other and the rows
    /// below would then be about a different question.
    const _: () = assert!(ALLOWED_AT_FIRST - ROOMY_COMPOSE_US > orb_sim::SPIKE_US);

    /// **A compositor with room to spare never moves the allowance**, which is the ratchet's own claim
    /// about itself: it steps for a frame that missed its blank and for nothing else.
    ///
    /// The form of that question a scenario can ask without the host having to say where each frame landed:
    /// a run with [`ROOMY_COMPOSE_US`]'s room in it has no frame that *can* miss, so any step at all is a
    /// step taken for a frame that landed where it was aimed. Which is worth being sure of rather than
    /// reasoning about, because a step is never given back — every wrong one is `MISS_STEP_US` of input lag
    /// for the rest of the run.
    ///
    /// Every rate a display reports, because the two cases are told apart by rounding the overshoot at half
    /// a refresh and a fast display is where that half-refresh is shortest: 8333µs at 60Hz against 2083µs at
    /// 240Hz, and the worst wake delay the host draws is 1300µs. So the fastest rate has the least room
    /// between a frame that landed and a frame a refresh late, and which rates leave enough is a table
    /// rather than an argument.
    #[test]
    fn a_compositor_with_room_to_spare_never_moves_the_allowance() {
        in_its_own_process(|| {
            for hz in [60u32, 120, 144, 165, 240] {
                for_each_seed(|seed| {
                    let mut display = Display::agreed(hz);
                    display.compose = Compose::flat(ROOMY_COMPOSE_US);
                    display.seed = seed;
                    let game = Fake::attach_watching_the_pacing(
                        display,
                        &format!("room-{hz}-{seed}"),
                        Work::flat(WORK_US),
                    );
                    game.frames(15 * A_SECOND as u32);
                    let lines = game.log().lines();
                    let said = last_said(&lines);

                    // No frame was anywhere but on the blank it was aimed at.
                    let past = frames_past_the_aim(&lines);
                    assert!(
                        past[1..].iter().all(|count| *count == 0),
                        "{hz}Hz, seed {seed}: {past:?} frames per count of refreshes past the blank they \
                         were aimed at, and this compositor has {}us of room — {said}",
                        ALLOWED_AT_FIRST - ROOMY_COMPOSE_US,
                    );
                    // So the allowance is where it started, to the microsecond.
                    let allowed = allowance_us(&lines);
                    assert_eq!(
                        allowed, ALLOWED_AT_FIRST,
                        "{hz}Hz, seed {seed}: {allowed}us allowed, and nothing here missed a blank — {said}",
                    );
                });
            }
        });
    }

    /// The allowance moved by one step for each frame that landed a refresh past the blank it was aimed
    /// at, and for nothing else — asked of each reported period against the one before it, and answering
    /// how many such frames the pairs held between them.
    ///
    /// **Period against period rather than over the run, because the first line holds the launch**: the
    /// frames that build the overlay and start a run draw more than a played one and outgrow their budget,
    /// which is a miss the ratchet is right not to step for, and `settle` finding the compositor's own
    /// spacing part-way through resets the allowance to where it started. Every pair after that first line
    /// is a run at a rate, and a step and the miss it was taken for are counted inside the same period
    /// whichever frame of it they fell on — `Pacing::shown` and the step are the same frame's work.
    fn steps_against_frames_a_refresh_late(lines: &[String], seed: u64, said: &str) -> usize {
        let periods = shown(lines);
        assert!(
            periods.len() > 1,
            "seed {seed}: orb reported {} period(s), so there is no pair of them to read a step off — \
             {said}",
            periods.len(),
        );
        let mut late = 0;
        for pair in periods.windows(2) {
            let (before, period) = (&pair[0], &pair[1]);
            let a_refresh_late = period.past_by(1);
            late += a_refresh_late;
            assert_eq!(
                period.allowance_us - before.allowance_us,
                A_STEP * a_refresh_late as i64,
                "seed {seed}: {}us allowed against the {}us of the period before it, over {a_refresh_late} \
                 frame(s) a refresh late and {:?} refreshes past the blank aimed at — {said}",
                period.allowance_us,
                before.allowance_us,
                period.past_the_aim,
            );
        }
        late
    }

    /// 240Hz, for the row below: the rate with the shortest refresh, which is where a composition can
    /// outlast one and still be a composition.
    ///
    /// A refresh is 4167µs there, so a desktop hiccup of a few milliseconds is past it while leaving its
    /// frame two refreshes out. At 60Hz the same sentence asks for a composition past 16667µs, which is a
    /// stage load in everything but name and lands its frame fifteen refreshes out — a different row's
    /// question. It is also the rate whose ceiling is shortest, 3124µs, so nothing about this run is a
    /// compositor the allowance was ever going to cover.
    const HZ_WITH_A_SHORT_REFRESH: u32 = 240;
    /// Longer than that display's 4167µs refresh, and a desktop hiccup rather than a load: one composition
    /// in three hundred taking as long as most of a game frame.
    const COMPOSE_PAST_A_REFRESH_US: i64 = 10_000;
    /// Long enough for a handful of those, at that rarity, in each of the reported periods.
    const FRAMES: u32 = 3_000;

    /// A composition longer than a whole refresh, which is the one miss no allowance can answer.
    ///
    /// **Where the ratchet may step is decided by what a step can do**, and what a step does is hand the
    /// frame over earlier. `compose_ceiling` holds the allowance under a refresh — hand a frame over
    /// further ahead than that and the compositor takes it at the blank *before* the one it was aimed at —
    /// so a composition longer than a refresh puts its frame two refreshes past its aim at every allowance
    /// the ceiling admits. A step there is bought for a frame it could not have saved, and it is never
    /// given back: measured before this at 200 to 400µs of input lag over fifteen seconds of play.
    #[test]
    fn a_composition_longer_than_a_refresh_does_not_move_the_allowance() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ_WITH_A_SHORT_REFRESH);
                display.compose = Compose {
                    usual_us: ROOMY_COMPOSE_US,
                    jitter_us: 0,
                    spike_us: COMPOSE_PAST_A_REFRESH_US,
                    spike_one_in: 300,
                };
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("past-a-refresh-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(FRAMES);
                // So that the frames of the run are all inside a period orb has reported one of these for:
                // the buckets and the allowance are read off those lines, and a run that stopped mid-period
                // would be asking about the frames it happened to have written down.
                game.frames_until_the_log_holds_another(PAST_THE_AIM);
                let lines = game.log().lines();
                let said = last_said(&lines);
                let past = frames_past_the_aim(&lines);

                // The hiccups happened and cost their frames two refreshes apiece, which is what makes an
                // allowance that never moved worth reading: a run that saw none would report one too.
                let beyond: usize = past.iter().skip(2).sum();
                assert!(
                    beyond > 0,
                    "seed {seed}: {past:?} frames per count of refreshes past the blank they were aimed \
                     at, so no composition here outlasted a refresh — {said}",
                );
                // And the allowance stepped for none of them, which is what the pairs say: a step is for a
                // frame a refresh late, and this run has no frame a refresh late in it to take one.
                let late = steps_against_frames_a_refresh_late(&lines, seed, &said);
                assert_eq!(
                    late, 0,
                    "seed {seed}: {past:?} — a frame a refresh late is the allowance's to answer, and this \
                     row is about the ones that are not — {said}",
                );
            });
        });
    }

    /// And every frame that *was* a refresh late moved it by one step, which at 60Hz they did not.
    ///
    /// **60Hz is where a refresh and a whole game turn are the same length**, so it is the rate at which a
    /// miss can be mistaken for the frame that takes a quarter of a second: the smallest miss there is is
    /// one refresh, which is one turn, and how late the two flushes either side of it were woken decided
    /// which of the two it was called. Measured before this: 131 of 160 misses uncharged in a run with no
    /// load anywhere in it, counting the frames those excused in turn, and each of them a refresh the player
    /// lost that the allowance was never told about.
    ///
    /// A compositor that spikes past the allowance rather than one flatly over it, so the misses arrive a
    /// few to a period instead of every frame until the climb is done: what the pairs are reading is a step
    /// against the miss it was taken for, and a climb finished inside one period would leave every pair
    /// after it with nothing in it.
    #[test]
    fn every_frame_a_refresh_late_moves_the_allowance_one_step() {
        const HZ_WHERE_A_REFRESH_IS_A_TURN: u32 = 60;
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::agreed(HZ_WHERE_A_REFRESH_IS_A_TURN);
                display.compose = Compose {
                    usual_us: ROOMY_COMPOSE_US,
                    jitter_us: 0,
                    spike_us: SLOW_COMPOSE_US,
                    spike_one_in: 300,
                };
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("no-room-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(FRAMES);
                game.frames_until_the_log_holds_another(PAST_THE_AIM);
                let lines = game.log().lines();
                let said = last_said(&lines);
                let past = frames_past_the_aim(&lines);

                // Nothing here landed further out than a refresh, so every miss of the run is one the
                // allowance answers for and the pairs are reading the case in question.
                assert!(
                    past.iter().skip(2).all(|count| *count == 0),
                    "seed {seed}: {past:?} frames per count of refreshes past the blank they were aimed \
                     at, against a compositor spiking to {SLOW_COMPOSE_US}us — {said}",
                );
                let late = steps_against_frames_a_refresh_late(&lines, seed, &said);
                assert!(
                    late > 0,
                    "seed {seed}: {past:?} — no frame of the pairs was a refresh late, so nothing here \
                     asked the allowance for a step — {said}",
                );
            });
        });
    }
}

/// Sixty frames a second, whatever the compositor takes.
///
/// This is the whole of what the pacing promises on a display it is not refusing: the allowance is
/// measured rather than chosen, so it climbs until the frames stop missing, and once they stop every
/// frame lands on the blank the sixtieth-of-a-second grid meant. What the compositor takes does not
/// come into it — that is what having an allowance is *for*.
///
/// So the invariant is not "sixty for the compose times we tried". It is **sixty, eventually, for any
/// of them**, and a compose time that does not reach sixty is a defect rather than a limit.
///
/// Held over enough frames for the climb to finish — the ratchet is a hundred microseconds a miss —
/// and read off the last thousand rather than the whole run, since the climb itself is allowed to cost
/// frames.
mod converges {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    /// What the game's own update and draw are declared to take: about what a real run's report line
    /// shows for the drawing.
    const WORK_US: i64 = 700;
    const FRAMES: u32 = 3_000;
    const SETTLED: usize = 2_000;

    /// What a compositor might take over a frame, in microseconds — the range these rows declare, not a
    /// reading of anywhere. It runs from well under the allowance's start to past the ceiling of the
    /// fastest rate covered, so that between them the rows ask what happens on both sides of every
    /// boundary the pacing has. What any one host takes is read off `--pacing` and is not a number the
    /// suite has an opinion about.
    const COMPOSE_US: [i64; 8] = [400, 1_000, 2_000, 2_500, 3_000, 3_200, 3_800, 4_000];

    fn settled(hz: u32, compose: Compose, name: &str, seed: u64) -> (Box<Fake>, f64) {
        let mut display = Display::agreed(hz);
        display.compose = compose;
        display.seed = seed;
        let game = Fake::attach_watching_the_pacing(display, name, Work::flat(WORK_US));
        game.frames(FRAMES);
        let rate = fps(&game.handovers_us(), SETTLED);
        (game, rate)
    }

    /// The one that holds: a compositor that is cheap most frames and occasionally several times that,
    /// which is the shape `Compose::spiking` declares. Every host, at each rate a display reports.
    fn spikes_and_settles(hz: u32) {
        for seed in 0..SEEDS {
            let name = format!("converges-{hz}-{seed}");
            let (game, rate) = settled(hz, Compose::ordinary(), &name, seed);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "{hz}Hz, seed {seed}: {rate} frames a second after {SETTLED} frames\n  {}",
                last_said(&game.log().lines())
            );
        }
    }

    // The rates a display reports itself as, over the range anyone plays at, are the rows of both tables
    // here and are written out in each: a `const RATES` read by two loops was what these were before, and a
    // rate cannot be a `#[test]`'s name and come out of an array at the same time. So the five are spelled
    // twice, and a rate added to one list and not the other is the thing to watch for.
    a_test_per_rate! {
        a_compositor_that_spikes_still_settles_at_sixty_on_a_60hz_display => spikes_and_settles(60),
        a_compositor_that_spikes_still_settles_at_sixty_on_a_120hz_display => spikes_and_settles(120),
        a_compositor_that_spikes_still_settles_at_sixty_on_a_144hz_display => spikes_and_settles(144),
        a_compositor_that_spikes_still_settles_at_sixty_on_a_165hz_display => spikes_and_settles(165),
        a_compositor_that_spikes_still_settles_at_sixty_on_a_240hz_display => spikes_and_settles(240),
    }

    /// **How long the compositor may take, and it is not "anything".** The frame is handed over
    /// `allowance` before the blank it is aimed at, so an allowance past a refresh hands it over before the
    /// blank *before* that one, and the compositor takes it there — the frame is shown a refresh early. So
    /// there is a ceiling of one refresh less a quarter, and a compositor wanting more than that cannot be
    /// covered by any amount of climbing. See `compose_ceiling` in `frame.rs`, which has the 144Hz
    /// measurement of what going past it does.
    ///
    /// Three quarters of a refresh is 12500µs at 60Hz and 3124µs at 240Hz, which is why the fast display is
    /// the one that runs out of room first.
    fn ceiling_us(hz: u32) -> i64 {
        (1_000_000 / hz as i64) * 3 / 4
    }

    /// The promise, for every compose time the pacing has the room to cover.
    ///
    /// This failed before `measure_compose` stopped leaving `recovering` set: seven of these forty pairs sat
    /// at half rate or thereabouts for the whole run — 60Hz at 3200µs and above, 144Hz and 165Hz at 3800 and
    /// above — with the allowance frozen and every miss charged to a stage load long over. It is not
    /// `#[ignore]`d and the bound is not the behaviour's: what a compositor takes has to be answerable up to
    /// the point where answering it is geometrically impossible, and that point is the ceiling below.
    ///
    /// The compose times of one rate are collected rather than asserted one at a time, so a failure says
    /// *which* of them are stuck and not merely that one is: the pattern above — everything from 3200µs up
    /// at 60Hz — is what named the cause, and a run that stopped at the first would have shown one pair.
    /// `tried` with it, because a ceiling that filtered every compose time out would leave nothing stuck
    /// and pass.
    fn inside_the_ceiling(hz: u32) {
        let mut stuck = Vec::new();
        let mut tried = 0;
        for compose_us in COMPOSE_US {
            if compose_us > ceiling_us(hz) {
                continue;
            }
            tried += 1;
            let name = format!("ceiling-{hz}-{compose_us}");
            let (game, rate) = settled(hz, Compose::flat(compose_us), &name, 0);
            if (rate - 60.0).abs() >= NEAR_SIXTY {
                stuck.push(format!(
                    "{hz}Hz at {compose_us}us: {rate:.2} fps\n    {}",
                    last_said(&game.log().lines())
                ));
            }
        }
        assert!(tried > 0, "{hz}Hz has no compose time inside its ceiling");
        assert!(
            stuck.is_empty(),
            "{} of {tried} compose times never reach sixty at {hz}Hz:\n  {}",
            stuck.len(),
            stuck.join("\n  ")
        );
    }

    a_test_per_rate! {
        any_compositor_inside_a_60hz_displays_ceiling_settles_at_sixty => inside_the_ceiling(60),
        any_compositor_inside_a_120hz_displays_ceiling_settles_at_sixty => inside_the_ceiling(120),
        any_compositor_inside_a_144hz_displays_ceiling_settles_at_sixty => inside_the_ceiling(144),
        any_compositor_inside_a_165hz_displays_ceiling_settles_at_sixty => inside_the_ceiling(165),
        any_compositor_inside_a_240hz_displays_ceiling_settles_at_sixty => inside_the_ceiling(240),
    }

    /// And past the ceiling it cannot, which is a limit rather than a defect.
    ///
    /// 240Hz is where the ceiling is shortest — 3124µs — so it is the rate at which a compositor wanting
    /// more than the geometry allows can be asked for at all. Whether a real desktop wants that much is a
    /// separate question and not this one's: the compose time here is pinned, and what a host actually
    /// takes is read off `--pacing` rather than asserted anywhere. What this holds is the pacing's
    /// behaviour when it is asked for more than there is room for: the allowance
    /// climbs to exactly the ceiling and stops, no miss is charged to a load, and the rate settles at 48.00
    /// for the rest of the run — five refreshes a frame against the four a sixtieth is at this rate, so
    /// every frame takes one extra and not every fifth.
    ///
    /// What is asserted is that shape and not a number to be satisfied with: the allowance ends *at* the
    /// ceiling, so what is missing is room and not a climb. A run reading 48fps with the allowance below the
    /// ceiling would be the latch back again, and this would catch it.
    #[test]
    fn past_the_ceiling_the_rate_is_not_sixty_and_the_allowance_is_out_of_room() {
        const HZ: u32 = 240;
        in_its_own_process(|| {
            for compose_us in COMPOSE_US {
                if compose_us <= ceiling_us(HZ) {
                    continue;
                }
                let name = format!("past-the-ceiling-{compose_us}");
                let (game, rate) = settled(HZ, Compose::flat(compose_us), &name, 0);
                let allowed = allowance_us(&game.log().lines());

                assert_eq!(
                    allowed,
                    ceiling_us(HZ),
                    "{HZ}Hz at {compose_us}us: the allowance stopped at {allowed}us, not at the ceiling — {}",
                    last_said(&game.log().lines())
                );
                assert!(
                    (rate - 60.0).abs() >= NEAR_SIXTY,
                    "{HZ}Hz at {compose_us}us: {rate} frames a second, which is sixty — so the ceiling was room \
                     enough after all and this test is the thing that is wrong"
                );
            }
        });
    }
}

/// Which displays the game runs at sixty frames a second on, and which it does not.
///
/// One case a test, because the question it answers is a table: somebody with a stutter wants to know
/// whether their desktop is one of the ones that breaks, and reading the arithmetic will not tell them.
///
/// Roughly the rate and not the exact turn: the host is not a metronome. Long enough runs that the
/// allowance has finished climbing, since the climb itself costs frames.
///
/// **What the split rows do and do not establish.** That a compositor can be timing a rate the game's
/// monitor is not, and that a flush follows the compositor whatever monitor the window is on, are measured
/// on real hardware, and what that decided is beside `frame::Pacing::grid`. What these rows add is orb's
/// own arithmetic over it.
mod rates {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};

    const WORK_US: i64 = 700;
    /// Fifteen seconds of play: a few for the allowance to settle in and a dozen to hold it.
    const FRAMES: u32 = 15 * A_SECOND as u32;

    /// Sixty frames a second in every second of the run past the grace — which is what "the game runs at the
    /// right speed" means to somebody playing it. The music and every timer in it are counted in the game's
    /// own frames, so a second at the wrong rate is a second of the game at the wrong speed however the
    /// average over the run reads.
    fn assert_rate(display: Display, name: &str, wanted: f64, seed: u64) {
        let game = Fake::attach_watching_the_pacing(display, name, Work::flat(WORK_US));
        game.frames(FRAMES);
        assert_every_second_at(
            &game.handovers_us(),
            wanted,
            seed,
            &last_said(&game.log().lines()),
        );
    }

    fn assert_sixty(display: Display, name: &str, seed: u64) {
        assert_rate(display, name, 60.0, seed);
    }

    fn agreed(hz: u32, seed: u64) -> Display {
        let mut display = Display::agreed(hz);
        display.seed = seed;
        display
    }

    fn split(monitor_hz: u32, compositor_hz: u32, seed: u64) -> Display {
        let mut display = Display::split(monitor_hz, compositor_hz);
        display.seed = seed;
        display
    }

    // ── The displays that work: one monitor, or several the compositor agrees with ──────────────────

    #[test]
    fn every_rate_a_display_reports_gets_sixty_frames_a_second() {
        in_its_own_process(|| {
            for hz in [60u32, 120, 144, 165, 240] {
                for_each_seed(|seed| {
                    assert_sixty(agreed(hz, seed), &format!("rate-{hz}-{seed}"), seed);
                });
            }
        });
    }

    /// And the NTSC-derived rates get the display's own, which is a tenth of a percent short of sixty.
    ///
    /// `dmDisplayFrequency` is whole Hz, so a 59.94Hz display reports 59 and a 119.88Hz one reports 119.
    /// `whole_multiple` reads those as one blank a frame and two, which is right — and the rate that comes
    /// out is then the display's, not sixty. A real NTSC-derived display came out at its own rate halved,
    /// two refreshes a frame throughout, which is the same thing.
    ///
    /// Here the compositor really is 59 and 119 rather than 59.94 and 119.88, since those are the numbers
    /// the scenario declares, so the rates to expect are 59 and 59.5.
    #[test]
    fn an_ntsc_rate_gets_the_displays_own_rate() {
        in_its_own_process(|| {
            for (hz, wanted) in [(59u32, 59.0), (119, 59.5)] {
                for_each_seed(|seed| {
                    assert_rate(agreed(hz, seed), &format!("ntsc-{hz}-{seed}"), wanted, seed);
                });
            }
        });
    }

    #[test]
    fn a_display_that_will_not_say_its_rate_is_paced_by_the_clock_at_sixty() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::unknown();
                display.seed = seed;
                assert_sixty(display, &format!("unknown-{seed}"), seed);
            });
        });
    }

    // ── A mixed-rate desktop: the compositor times one monitor and the game is on another ───────────
    //
    // What decides the outcome is whether the rate the *compositor* is timing has a blank where a sixtieth of
    // a second falls, because the frames go on its blanks whatever the monitor says. Where it has one, the
    // disagreement costs nothing however large it is.

    #[test]
    fn a_disagreement_the_compositor_can_still_land_a_sixtieth_on_is_harmless() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                assert_sixty(
                    split(120, 240, seed),
                    &format!("split-120-240-{seed}"),
                    seed,
                );
                assert_sixty(split(240, 60, seed), &format!("split-240-60-{seed}"), seed);
            });
        });
    }

    /// And the game's own monitor not being a whole multiple is harmless too, because the rate the cadence
    /// is counted in is the compositor's either way — here a 120Hz one, which is a whole multiple.
    #[test]
    fn a_fractional_monitor_a_whole_multiple_compositor_is_timing_gets_sixty() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let game = Fake::attach_watching_the_pacing(
                    split(144, 120, seed),
                    &format!("split-144-120-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(FRAMES);
                assert_every_second_at(
                    &game.handovers_us(),
                    60.0,
                    seed,
                    &last_said(&game.log().lines()),
                );
                // On the blanks, and on the compositor's count of them rather than the monitor's.
                assert!(
                    game.log()
                        .said("120Hz compositor, one frame every 2 blank(s)"),
                    "seed {seed}: {:?}",
                    game.log().lines()
                );
                assert!(
                    !game.log().said("pacing by the clock"),
                    "seed {seed}: {}",
                    last_said(&game.log().lines())
                );
            });
        });
    }

    /// **The ones that used to be broken.** A monitor that *is* a whole multiple used to keep the cadence in
    /// its own refreshes while the frames went on the compositor's blanks, and the rate came out wrong at
    /// every one of these: the frames came too quickly and the game ran *fast*, with the music and every
    /// timer in it counted in them.
    ///
    /// The cadence is counted in the compositor's spacing now, so each of these is the same fractional grid a
    /// display of that rate would get: one frame on whichever blank is nearest each sixtieth. Every second of
    /// every one of them is sixty.
    fn a_fractional_compositor(compositor_hz: u32) {
        for_each_seed(|seed| {
            let game = Fake::attach_watching_the_pacing(
                split(120, compositor_hz, seed),
                &format!("was-broken-{compositor_hz}-{seed}"),
                Work::flat(WORK_US),
            );
            game.frames(FRAMES);
            assert_every_second_at(
                &game.handovers_us(),
                60.0,
                seed,
                &last_said(&game.log().lines()),
            );
        });
    }

    a_test_per_rate! {
        a_fractional_70hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(70),
        a_fractional_75hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(75),
        a_fractional_90hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(90),
        a_fractional_100hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(100),
        a_fractional_110hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(110),
        a_fractional_144hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(144),
        a_fractional_150hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(150),
        a_fractional_165hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(165),
        a_fractional_200hz_compositor_gets_sixty_whatever_the_monitor_says => a_fractional_compositor(200),
    }

    /// A monitor that will not say its rate, on a desktop whose compositor will.
    ///
    /// Which is the other case the old cadence lost: with nothing to take a whole multiple of, the blanks
    /// were refused and the frames were paced by the clock — while a compositor sat there timing them the
    /// whole time. The rate is taken from the compositor now, so this is an ordinary 120Hz run.
    #[test]
    fn a_monitor_that_will_not_say_is_paced_on_the_compositors_blanks() {
        in_its_own_process(|| {
            for_each_seed(|seed| {
                let mut display = Display::unknown();
                display.compositor_hz = Some(120);
                display.seed = seed;
                let game = Fake::attach_watching_the_pacing(
                    display,
                    &format!("silent-monitor-{seed}"),
                    Work::flat(WORK_US),
                );
                game.frames(FRAMES);
                assert_every_second_at(
                    &game.handovers_us(),
                    60.0,
                    seed,
                    &last_said(&game.log().lines()),
                );
                assert!(
                    game.log()
                        .said("120Hz compositor, one frame every 2 blank(s)"),
                    "seed {seed}: {:?}",
                    game.log().lines()
                );
                assert!(
                    game.log().said("0 frame(s) paced by the clock"),
                    "seed {seed}: {}",
                    last_said(&game.log().lines())
                );
            });
        });
    }
}

/// **Sixty frames a second is held in every situation, and this is the table of what a situation is.**
///
/// The one claim the pacing exists to make, in one place. Every other section here is about a mechanism
/// inside it — the budget's climb, the allowance's ratchet, what a landing more than a refresh out
/// excuses, the grid's arithmetic — and each of those can be right while the thing somebody playing cares
/// about is wrong.
///
/// # Why a table and not every combination
///
/// Four things vary: the monitor's rate, the compositor's rate, how unevenly the compositor takes its
/// time, and how unevenly the game's own frame takes its. Crossing them is several hundred runs, and most
/// of the crossings ask the same question twice, because **only the first two decide which path the code
/// takes.** `adopt` chooses among five: no compositor to ask, a rate that is a whole multiple of sixty, a
/// rate that rounds to one, a rate that is not a multiple, and a rate under sixty. The other two are the
/// same arithmetic in every one of those — they widen what the allowance has to cover and nothing else.
///
/// So the rows are the five paths, each at rest and each with everything uneven at once, and then the
/// unevenness taken apart one source at a time on the one path that has least room to absorb it. A row
/// that fails says which condition did it, which is what a table is for and what a cross product is not.
///
/// # One test a row
///
/// Not a loop over rows inside one test, because a row is a launch and a launch is a process — so the
/// harness runs them across the cores at once, where a loop would run them one after another. The seeds
/// stay a loop inside a row: four hosts of the same situation are the same question asked again, and
/// naming which of them failed is what the assertion messages do.
///
/// # Where the floors come from
///
/// Measured, per row, over the four hosts apiece, and each floor is set below the worst of them: what a
/// lost refresh costs turns on the display's rate and how often one is lost turns on how uneven the row
/// declared its desktop to be, so a single floor low enough for the worst row would say nothing about any
/// of the others. What the floor is for is a *stretch* of bad seconds — the mixed-rate desktop's own
/// defect was 0% of them — and not one more second than last time.
///
/// | row | seconds untouched, per host | floor |
/// | --- | --- | --- |
/// | 120Hz, restless | 73 91 73 73 | 65% |
/// | 60Hz, restless | 55 64 64 64 | 45% |
/// | 144Hz, restless | 100 91 73 82 | 65% |
/// | 119Hz, restless | 73 91 73 82 | 65% |
/// | no compositor, restless | 100 91 73 91 | 65% |
/// | 60Hz, the compositor's ordinary cost wandering | 91 73 91 64 | 55% |
/// | 60Hz, the compositor spiking | 82 91 64 73 | 55% |
/// | 60Hz, the game's own frame wandering | 100 100 100 100 | 90% |
/// | 120Hz monitor, 144Hz compositor, restless | 100 91 73 82 | 65% |
/// | 120Hz monitor, 70Hz compositor, restless | 91 91 64 73 | 55% |
/// | silent monitor, 120Hz compositor, restless | 73 91 73 73 | 65% |
///
/// # What is *not* here
///
/// The three of these that are not sixty, each of which has its own reason and its own row below:
/// a display under sixty, which is deliberately the clock; the NTSC-derived rates, which get the
/// display's own rate and are right to; and a compositor wanting more than three quarters of a refresh,
/// which is geometry and lives in the `converges` section.
mod holds {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    /// Fifteen seconds: three for the allowance to finish climbing and a dozen judged a second at a time.
    ///
    /// Which is also what makes "some proportion of the time" the strongest form of itself here. A run of
    /// twelve judged seconds cannot measure 99% — eleven of twelve is 91.7% — so the assertion is *every*
    /// second, and a row that wants a proportion instead says so and says why.
    const FRAMES: u32 = 15 * A_SECOND as u32;

    /// What the game's own frame costs when it is not the thing being varied: about what a real run's
    /// report line shows for the drawing.
    const WORK_US: i64 = 700;

    /// Everything uneven at once, which is the row that says the sources do not compound into something none
    /// of them does alone.
    fn a_restless_desktop() -> Compose {
        Compose {
            jitter_us: 800,
            spike_one_in: 300,
            ..Compose::ordinary()
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
        /// The measured distribution, over [`SEEDS`] of them, each named in what it fails.
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
                let game =
                    Fake::attach_watching_the_pacing(display, &format!("{name}-{seed}"), work);
                game.frames(FRAMES);
                let handovers = game.handovers_us();
                let said = last_said(&game.log().lines());
                match judged {
                    Judged::EverySecond => {
                        assert_every_second_at(&handovers, wanted, seed, &said);
                    }
                    Judged::SecondsUntouched(_) => {
                        shares.push((
                            seed,
                            assert_holds_through_it(&handovers, wanted, seed, &said),
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
                let (seed, worst) = shares.iter().copied().fold((0, f64::MAX), |worst, at| {
                    if at.1 < worst.1 { at } else { worst }
                });
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
    /// period that rounds to 119, which is two blanks a frame and 59.5 frames a second — and the real one
    /// came out at 59.94, its own rate halved. A tenth of a per cent is a clock nobody sees.
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
    /// of it is 48 — so what this holds is that the seconds either side of it are untouched, which is the
    /// `load` section's claim about the allowance seen from the rate's end.
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
}

/// What orb's own frame loop asks of the game, in what order, and the ways it declines to run one.
///
/// The rate is every other section's subject. This is the loop's shape: the update before the draw, the
/// sounds between them, the present at the end, and the frame handed back to the game's own loop where
/// there is nothing to pace or draw with. None of it could be driven before the two calls into the game
/// became addresses the game hands over — see `docs/adr/0002`.
mod frame_loop {
    use super::*;
    use crate::fake::th06::{
        CHAIN_FAILED, CHAIN_LEFT, FRAME_FAILED, FRAME_KEPT_RUNNING, FRAME_LEFT, Fake, the_run,
    };
    use crate::fake::{DRAW, Launched, PRESENT, SOUND, UPDATE, WINDOW, in_its_own_process};
    use orb_api::Hwnd;

    /// A launch, run far enough in that the frames below are ordinary ones — the overlay built, the title
    /// menu up — and then asked to forget what it was asked for while getting there.
    fn settled(name: &str, settings: impl FnOnce(&mut orb_config::Config)) -> Box<Fake> {
        let game = Fake::attach(name, the_run(), settings);
        game.at_the_title_menu();
        game.forget_asked();
        game
    }

    /// The same, with the frame loop's own log on so that what it says about the pacing can be read back.
    fn settled_with_the_pacing_log(
        name: &str,
        settings: impl FnOnce(&mut orb_config::Config),
    ) -> Box<Fake> {
        settled(name, |config| {
            config.pacing_log = true;
            settings(config);
        })
    }

    #[test]
    fn the_frame_loop_asks_for_the_update_first_and_the_draw_after_the_sounds() {
        in_its_own_process(|| {
            let game = settled("frame-loop-order", |_| {});
            game.frame();
            // The whole of what a frame asks of the game, in orb's order. The update before the draw is the
            // frame of input lag removed; the sounds go where the game's own loop put them, after the update.
            assert_eq!(game.asked(), [UPDATE, SOUND, DRAW, PRESENT]);
        });
    }

    /// And a launch that turned it off gets 紅魔郷's own order back, which is what `--no-frame-loop` is for.
    #[test]
    fn the_games_own_loop_asks_for_the_draw_first() {
        in_its_own_process(|| {
            let game = settled("frame-loop-off", |config| config.own_frame_loop = false);
            game.frame();
            assert_eq!(game.asked(), [DRAW, UPDATE, SOUND, PRESENT]);
        });
    }

    /// The chain's two exits are the frame's two, and a frame that is leaving is not drawn or handed over.
    ///
    /// Which is what the game's own loop above `Render` reads to stop: a walk that answered nothing has asked
    /// the game to stop, and one that answered `-1` has failed. Nothing of orb's decides that — it is passed
    /// on, and the frame stops where it was told to.
    #[test]
    fn the_chains_two_exits_become_the_frames_two() {
        in_its_own_process(|| {
            let game = settled("frame-loop-exits", |_| {});
            for (walked, answered) in [(CHAIN_LEFT, FRAME_LEFT), (CHAIN_FAILED, FRAME_FAILED)] {
                game.chain_answers(walked);
                game.forget_asked();
                assert_eq!(game.frame(), answered, "the chain answered {walked}");
                assert_eq!(
                    game.asked(),
                    [UPDATE, SOUND],
                    "the chain answered {walked} and the frame carried on",
                );
            }
        });
    }

    /// With no runtime the frame is the game's own again, which is the state a process is in before
    /// `DllMain` has left one behind and the state a closed game is left in.
    #[test]
    fn a_frame_with_no_runtime_is_handed_back_to_the_game() {
        in_its_own_process(|| {
            let game = settled("frame-loop-no-runtime", |_| {});
            unsafe { orb_core::runtime::detached() };
            assert_eq!(game.frame(), FRAME_KEPT_RUNNING);
            assert_eq!(game.asked(), [DRAW, UPDATE, SOUND, PRESENT]);
        });
    }

    /// And with no device, which is every frame before the game has finished setting Direct3D up: there is
    /// nothing to draw the overlay through and nothing to pace against.
    #[test]
    fn a_frame_with_no_device_is_handed_back_to_the_game() {
        in_its_own_process(|| {
            let game = settled("frame-loop-no-device", |_| {});
            game.image().shows_through(orb_api::Device::NULL, WINDOW);
            assert_eq!(game.frame(), FRAME_KEPT_RUNNING);
            assert_eq!(game.asked(), [DRAW, UPDATE, SOUND, PRESENT]);
        });
    }

    /// The window behind is the one way out that paces instead of returning.
    ///
    /// The game's own loop calls straight back, so a frame that returned without waiting would spin a core
    /// for as long as the window stayed behind. Nothing is asked of the game — that is what `always_draw` off
    /// means — and the turn is still taken.
    ///
    /// **On the blanks, and not by the clock.** That is measured rather than assumed: a window behind, one
    /// covered by a full-screen window and one minimised all flush at the compositor's own rate with every
    /// gap one refresh, and the lead a frame needs to make its blank is the same whether anybody can see
    /// it. So a background frame is paced like any other, which is what the run below reads back out of
    /// the log.
    #[test]
    fn a_frame_with_the_window_behind_takes_its_turn_on_the_blanks() {
        in_its_own_process(|| {
            let game = settled_with_the_pacing_log("frame-loop-behind", |config| {
                config.always_draw = false;
            });
            let before = game.sim().clock().peek();
            game.sim().display().set_foreground(Hwnd(WINDOW.0 + 1));

            assert_eq!(game.frame(), FRAME_KEPT_RUNNING);
            assert_eq!(game.asked(), Vec::<&str>::new());

            // A whole turn of the clock went by all the same, which is the difference between pacing and
            // returning: a sixtieth of a second is 16666µs, and the wait is the only thing here that takes
            // any.
            let took = orb_sim::Clock::micros_for_ticks(game.sim().clock().peek() - before);
            assert!(
                took > 10_000,
                "the frame took {took}us, which is no turn at all"
            );

            // And it was the compositor's blank it waited for, which orb's own count of the other kind is
            // what says: a frame paced by the clock is counted there and never taken back.
            //
            // Read after the window comes forward again, because that count is written from inside the
            // update — and a frame behind asks the game for nothing, so a run that stayed behind writes
            // nothing at all to read.
            const BEHIND: u32 = 300;
            game.frames(BEHIND);
            game.sim().display().set_foreground(WINDOW);
            game.frames_until_the_log_holds_another(A_REPORT);

            assert!(
                game.log().said("0 frame(s) paced by the clock"),
                "the {} frame(s) behind were paced by the clock — {}",
                BEHIND + 1,
                last_said(&game.log().lines())
            );
        });
    }
}

/// Lines the frame loop holds back until writing one costs nothing.
///
/// The moments between handing a frame over and the blank it is shown at are the ones a write must stay
/// out of: the next frame has about a millisecond to reach `DwmFlush`, and one that arrives after the
/// blank has gone waits out another refresh and is shown late. So what the pacing says about itself is
/// held, and written on the far side of that flush, where what is left of the turn is slack.
///
/// **Which is a claim about where the real loop drains**, and that is why this is driven by a game whose
/// own loop calls `render`: a harness that deferred and drained in an order of its own would be saying
/// where *it* drains. The two lines below are read off the log — the moment orb stamped the line with,
/// against the moments the game was handed its frames over at.
///
/// What covers the writing itself is `log_writes.rs` in `orb-sim`.
mod log_deferral {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_core::profile;

    /// A display with room for a frame that takes its time: at 60Hz a turn is 16.6ms, and the work below is
    /// eight of them.
    const HZ: u32 = 60;

    /// What the game's own work takes. Far larger than any other pacing scenario asks for, and that is the
    /// point: the stamps here are milliseconds, so the moment the loop drains and the moment it does the
    /// frame's work have to be milliseconds apart for the log to say which of them the line was written in.
    const WORK_US: i64 = 8_000;
    const WORK_MS: i64 = WORK_US / 1_000;

    #[test]
    fn what_the_frame_loop_defers_is_written_on_the_far_side_of_the_next_flush() {
        in_its_own_process(|| {
            let game = Fake::attach_watching_the_pacing(
                Display::agreed(HZ),
                "log-deferral",
                Work::flat(WORK_US),
            );
            // Up to and including the frame whose own update works the pacing's account of the period out.
            game.frames(profile::INTERVAL);
            let over = millis_at(*game.handovers_us().last().expect("a frame handed over"));

            // Held back, not written: writing it where it was worked out is what would cost the next frame
            // its blank, and that frame is the one the account is about.
            assert!(
                reports(&game.log().lines()).is_empty(),
                "written where it was worked out:\n  {}",
                game.log().lines().join("\n  ")
            );

            // One more frame, which is the one with the slack in it.
            game.frame();
            let next = millis_at(*game.handovers_us().last().expect("a frame handed over"));
            let lines = game.log().lines();
            let written = lines
                .iter()
                .find(|line| line.contains(A_REPORT))
                .unwrap_or_else(|| panic!("nothing was drained:\n  {}", lines.join("\n  ")));
            let held = stamped_at(written);

            // After the frame it is about was handed over, which is the holding back.
            assert!(
                held > over,
                "stamped {held}ms, and the frame it is about went out at {over}ms — {written}"
            );
            // And before that next frame did its own work, which is where the slack is: the flush returned,
            // the line was written, and the turn's remaining fourteen milliseconds were still ahead. A line
            // written beside the work instead would be inside the millisecond the frame after it has to reach
            // the flush in.
            assert!(
                next - held >= WORK_MS,
                "stamped {held}ms against a frame handed over at {next}ms, which is inside its own \
                 {WORK_MS}ms of work — {written}"
            );

            // And the three lines of a period are written in the order they were deferred, so that a run
            // reads as a sequence of frames rather than as whatever came out of the buffer first.
            let at = |needle: &str| {
                lines
                    .iter()
                    .position(|line| line.contains(needle))
                    .unwrap_or_else(|| {
                        panic!("no line holds {needle:?} among\n  {}", lines.join("\n  "))
                    })
            };
            assert!(at(A_REPORT) < at("on screen from"));
            assert!(at("on screen from") < at("refreshes past the blank aimed at"));
        });
    }
}

// ── What the compositor's own counters are worth ─────────────────────────────────────────────────
//
// What this holds is that orb reports one of these numbers and that reporting it has to keep saying
// nothing, which is why it is a scenario rather than a note about what a host was once seen doing.
//
// The claim is a negative one, so it is asserted the only way a negative can be: the same frames are run
// twice, against two hosts that differ in this one answer and in nothing else, and every number orb
// decided is held to be the same across the two. A host reading zero while frames plainly miss says the
// number is worthless; a host reading the loudest answer it could give and changing nothing says orb
// agrees.
mod counters {
    use super::*;
    use crate::fake::{Display, Launched, Work, in_its_own_process, th06::Fake};
    use orb_sim::Compose;

    /// The display, and a compositor wanting more of each refresh than there is room to give it: past
    /// `ceiling_us(240)`'s 3124µs, which is the one rate anyone plays at where a plausible compositor
    /// reaches it. See [`converges::past_the_ceiling_the_rate_is_not_sixty_and_the_allowance_is_out_of_room`],
    /// which is the same host and asserts what it does to the rate.
    ///
    /// Chosen because frames have to be *plainly* missing for a count of zero to be worth anything: here
    /// every fifth frame takes a fifth refresh for the rest of the run, and the run settles at 48fps.
    const HZ: u32 = 240;
    const COMPOSE_US: i64 = 4_000;

    /// What the game's own update and draw are declared to take: about what a real run's report line
    /// shows for the drawing.
    const WORK_US: i64 = 700;

    /// Long enough that the allowance has finished climbing and the misses that are left are the
    /// geometric ones, since a climb still under way is a run whose two halves are not the same run.
    const FRAMES: u32 = 3_000;

    /// How many of this display's refreshes a sixtieth of a second is, which is the gap every frame
    /// should have and the one the missing frames are not in.
    const GRID_REFRESHES: usize = (HZ / frame::LOGIC_HZ) as usize;

    /// Runs those frames against a host `answers` has had its say about, and hands back the moments the
    /// game was handed them over at beside the last reporting line orb wrote.
    fn run(name: &str, answers: impl FnOnce(&Fake)) -> (Vec<i64>, Reported) {
        let mut display = Display::agreed(HZ);
        display.compose = Compose::flat(COMPOSE_US);
        let game = Fake::attach_watching_the_pacing(display, name, Work::flat(WORK_US));
        answers(&game);
        game.frames(FRAMES);
        (game.handovers_us(), reported(&game.log().lines()))
    }

    /// `cFramesLate` is not evidence, and neither is the family around it.
    ///
    /// The compositor's own count of frames it could not show at the refresh they were aimed at read
    /// `0 shown late` through every run the pacing work was developed against, *including* runs where a
    /// tenth of the frames in a period plainly missed their blank. It is still reported, as a number whose
    /// meaning is not what its name says.
    ///
    /// `qpcFrameDisplayed`, `cFrameDisplayed`, `cFramesDropped`, `cFramesMissed` and `cRefreshesDisplayed`
    /// are worse: all zero, while `cFrameSubmitted` and `cFrameConfirmed` in the same read moved with the
    /// frames. So the call works and that family is not populated for the desktop query, which is the only
    /// one `DwmGetCompositionTimingInfo` accepts. None of those five is in [`Composition`] at all, which
    /// is the whole of what that leaves behind: a field orb does not read is a field no scenario can say
    /// anything about.
    ///
    /// And `cRefresh` was measured advancing by exactly one per read whether a run of frames had been
    /// handed over or none at all — so it is neither refreshes nor our compositions, and the first probe
    /// written for it read it twice and got its own calls back as the answer.
    ///
    /// [`Composition`]: orb_api::Composition
    #[test]
    fn nothing_is_judged_on_the_compositors_own_count_of_late_frames() {
        in_its_own_process(|| {
            let (silent_handovers, silent) = run("counters-zero", |_| {});
            let (loud_handovers, loud) = run("counters-late", |game| {
                game.sim().display().says_every_composition_was_late();
            });

            // The frames were plainly missing, which is what makes a count of zero worth reading: a gap
            // that is not the grid's own is a frame that waited out a refresh it was not aimed at.
            let missed: usize = silent
                .gaps
                .iter()
                .filter(|(refreshes, _)| *refreshes != GRID_REFRESHES)
                .map(|(_, count)| count)
                .sum();
            assert!(
                missed > 0,
                "no frame of the period missed its blank, so this host says nothing about a count of \
                 them: {silent:?}"
            );
            // And the compositor said none of them had. Which is the measurement, and the whole of what
            // the number is worth.
            assert_eq!(
                silent.shown_late, 0,
                "{missed} of {} frames missed their blank and the host was asked to say none was late: \
                 {silent:?}",
                silent.frames,
            );
            // The other host said every frame it was handed was late, and orb repeated it — reported,
            // which is the half of this that has to keep working.
            assert_eq!(
                loud.shown_late, loud.frames,
                "the host said every composition was late and the line does not say so: {loud:?}"
            );

            // And decided nothing by it. The same frames went out at the same moments, to the tick, so
            // the count reached no wait and no allowance — the two runs are one run whose host answered
            // one question differently.
            assert_eq!(
                silent_handovers, loud_handovers,
                "the frames moved when the host changed its count of late ones"
            );
            // Which the line agrees about in every part of itself but that count, including the
            // allowance: it climbs on a frame *orb* saw miss, and never on one the compositor claims.
            assert_eq!(silent.frames, loud.frames);
            assert_eq!(silent.interval_us, loud.interval_us);
            assert_eq!(silent.prepare_us, loud.prepare_us);
            assert_eq!(silent.draw_us, loud.draw_us);
            assert_eq!(
                silent.compose_us, loud.compose_us,
                "the allowance followed the compositor's own count: {silent:?} against {loud:?}"
            );
            assert_eq!(silent.gaps, loud.gaps);
        });
    }
}
