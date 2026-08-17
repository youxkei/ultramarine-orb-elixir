//! **orb's cadence over 妖々夢's own frame, with a run in it.**
//!
//! `pacing.rs` is the pacing itself — the budget, the allowance, the spin, the compositor's own lateness — and
//! all of that is orb's code, the same code whichever game is in front of it. What is the *game's* is the frame
//! that runs inside the loop: 妖々夢's own update, its draw chain with the queue of quads either side of it, and
//! its present. So what this asks is the question a player asks of that pair — **does a run of this game come out
//! at sixty frames a second** — over the display shapes a machine really has, and with a run in progress rather
//! than a launch sitting at a menu.
//!
//! `th07.rs`'s own launch is the same question with nothing being played; this one has chapters being taken,
//! snapshots being copied and the mark being drawn, which is the frame a run really spends.
//!
//! **Roughly the rate rather than the exact turn, and judged a second at a time.** The host wakes the waiting
//! thread when it gets round to it and its compositor is slow now and then — see `orb-sim/src/display.rs` — so an
//! e2e test that asserted a turn to the microsecond would be asserting that the host is a metronome, which none
//! is. And an average over the whole run is no use: a run that spends a second at 30 and a second at 90 averages
//! sixty and is unplayable.

use crate::fake::th07::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::frame;

/// What the game's own frame is declared to take: about what a real run's report line shows for the drawing, so
/// that the pacing is being asked the question a run asks it.
const WORK_US: i64 = 700;

/// How long each of these runs is: several seconds, which is long enough for the pacing to settle and for every
/// second of it to be judged on its own.
const SECONDS: u32 = 4;

/// The rate the game was written for, which is what every one of these has to come out at whatever the display
/// reports.
const SIXTY: f64 = frame::LOGIC_HZ as f64;

/// How near sixty every second of a run has to come, in frames a second.
///
/// Six, for `pacing.rs`'s own reason: the host is not a metronome, so a second here and there loses a refresh —
/// one lost in a second of sixty takes that second to 59.0 and a handful to 55.3 — and the band is nowhere near
/// the rates a defect produces, which are 30 on a latched allowance, 48, 52, 71.
const NEAR_SIXTY: f64 = 6.0;

/// A 完全無欠モード run of this game on the display given, played for `SECONDS` of it.
///
/// The song is streaming and the fight is left alone: what this is about is the frames, and a chapter at every
/// boundary of a fight would be snapshots taken on frames this is measuring.
fn a_run(display: Display, name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(display, name, Work::wandering(WORK_US), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.streams_its_song(0);
    game.in_a_pointdevice_run();
    game.frames(SECONDS * frame::LOGIC_HZ);
    game
}

/// The rate of each second of the run, out of the moments the game was handed its frames over at — which is where
/// its own present wrote them down, and so what somebody playing had.
///
/// The first second is left out: the frames before the pacing has settled are the ones orb spends finding the
/// display's own rhythm, and a launch is judged on what it settles at.
fn each_second(handovers_us: &[i64]) -> Vec<f64> {
    let Some(first) = handovers_us.first().copied() else {
        return Vec::new();
    };
    let mut seconds = Vec::new();
    for second in 1..u64::from(SECONDS) {
        let from = first + (second as i64) * 1_000_000;
        let to = from + 1_000_000;
        let frames = handovers_us
            .iter()
            .filter(|at| **at >= from && **at < to)
            .count();
        seconds.push(frames as f64);
    }
    seconds
}

/// Every second of the run within [`NEAR_SIXTY`] of sixty, naming the second that was not.
fn holds_at_sixty(game: &Fake, said: &str) {
    let seconds = each_second(&game.handovers_us());
    assert!(
        !seconds.is_empty(),
        "{said}: the run was too short to judge a second of",
    );
    for (second, rate) in seconds.iter().enumerate() {
        assert!(
            (rate - SIXTY).abs() <= NEAR_SIXTY,
            "{said}: second {second} of the run came out at {rate} frames a second, not {SIXTY}\n  \
             every second: {seconds:?}",
        );
    }
}

/// A display whose monitor and compositor agree on a whole multiple of sixty, which is what almost every machine
/// has: the frames go on its blanks.
#[test]
fn a_run_of_th07_on_an_ordinary_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(120), "th07-paced-120");
        holds_at_sixty(&game, "120Hz agreed");
    });
}

/// And on a display at the rate the game was written for, where every frame is one refresh.
#[test]
fn a_run_of_th07_on_a_sixty_hertz_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(60), "th07-paced-60");
        holds_at_sixty(&game, "60Hz agreed");
    });
}

/// And where the window is on one monitor while the compositor times another, which is the shape a second screen
/// makes: the rate is the compositor's to give and the run is still sixty.
#[test]
fn a_run_of_th07_on_a_split_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::split(143, 60), "th07-paced-split");
        holds_at_sixty(&game, "a 143Hz monitor a 60Hz compositor is timing");
    });
}

/// And where nothing will say the rate at all, which is the one case paced by the clock.
#[test]
fn a_run_of_th07_on_a_display_that_will_not_say_its_rate_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::unknown(), "th07-paced-unknown");
        holds_at_sixty(&game, "a display that will not say its rate");
    });
}

/// And the report line orb writes about it is the same run's: what somebody reading a real log has, over this
/// game's own frame.
#[test]
fn the_report_line_of_a_run_of_th07_says_what_the_frames_did() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(120), "th07-paced-reported");
        let log = game.log();
        // Waited for rather than looked for: the report is written once a reporting period, and the line a launch
        // opens with is the cadence it settled on rather than a report of any frames.
        game.frames_until_the_log_holds_another("us apart,");
        let said = log
            .lines()
            .into_iter()
            .rev()
            .find(|line| line.contains("us apart,"))
            .expect("a report of the frames of a run of several seconds");

        // What the line is for: how many frames the period covered, how far apart they were handed over, how much
        // of the frame the drawing and the compositor were given, and how many reached the screen late. Named
        // rather than measured here — what each number should be is `pacing.rs`'s subject — because what this says
        // is that a run of *this* game produces them.
        for part in [
            "frames,",
            "us apart,",
            "before the blank",
            "to draw",
            "for the compositor",
            "shown late",
        ] {
            assert!(
                said.contains(part),
                "the report of the frames does not say the {part:?}: {said}",
            );
        }
        // And the frames it counted are this run's: a period that reported none would be a loop that handed none
        // over, whatever the rest of the line said.
        let frames: u32 = said
            .split("frame: ")
            .nth(1)
            .and_then(|rest| rest.split(' ').next())
            .and_then(|count| count.parse().ok())
            .unwrap_or_else(|| panic!("no count of frames at the head of {said}"));
        assert!(
            frames >= SECONDS * frame::LOGIC_HZ,
            "the report covered {frames} frames, fewer than the {} the run had run",
            SECONDS * frame::LOGIC_HZ,
        );
    });
}
