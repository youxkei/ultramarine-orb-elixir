//! **orb's cadence over th08's own frame, with a run in it.**
//!
//! `pacing.rs` is the pacing itself — the budget, the allowance, the spin, the compositor's own lateness —
//! and all of that is orb's code, the same code whichever game is in front of it. What is the *game's* is
//! the frame that runs inside the loop, and this game's is one whose own limiter and frame-skip loop orb has
//! patched out: orb's loop calls the frame's pieces instead — see `Th08::frame_calls`.
//!
//! So what this asks is the question a player asks of that pair — **does a run of this game come out at
//! sixty frames a second** — over the display shapes a machine really has, and with a run in progress
//! rather than a launch sitting at a menu: `th08.rs`'s own pacing test is a launch with no run in it.
//!
//! **Roughly the rate rather than the exact turn, and judged a second at a time**, for
//! `the_pacing_of_th07.rs`'s reasons: the host wakes a waiting thread when it gets round to it, and an
//! average over a whole run reads a second at 30 and a second at 90 as sixty.

use crate::fake::th08::Fake;
use crate::fake::{Display, Launched, Work, in_its_own_process};
use orb_config::LogLevel;
use orb_core::frame;

/// What the game's own frame is declared to take, as every other e2e test over this game declares it.
const WORK_US: i64 = 700;

/// How long each of these runs is: several seconds, which is long enough for the pacing to settle and for
/// every second of it to be judged on its own.
const SECONDS: u32 = 4;

/// The rate the game was written for, which is what every one of these has to come out at whatever the
/// display reports.
const SIXTY: f64 = frame::LOGIC_HZ as f64;

/// How near sixty every second of a run has to come, in frames a second — `the_pacing_of_th07.rs`'s own six,
/// and for the same reason.
const NEAR_SIXTY: f64 = 6.0;

/// A Pointdevice mode run of this game on the display given, played for `SECONDS` of it — placed, which is a
/// run with nothing fired at the player: a death in this mode is a retry offered, and a question nobody
/// answers is a game standing on one frame, which is every frame this is measuring.
fn a_run(display: Display, name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(display, name, Work::wandering(WORK_US), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.its_title_menu_is_up();
    game.is_in_a_run();
    game.frames(SECONDS * frame::LOGIC_HZ);
    game
}

/// The rate of each second of the run, out of the moments the game was handed its frames over at — which is
/// where its own present wrote them down, and so what somebody playing had.
///
/// The first second is left out: the frames before the pacing has settled are the ones orb spends finding
/// the display's own rhythm, and a launch is judged on what it settles at.
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

/// A display whose monitor and compositor agree on a whole multiple of sixty, which is what almost every
/// machine has: the frames go on its blanks.
#[test]
fn a_run_of_th08_on_an_ordinary_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(120), "th08-paced-120");
        holds_at_sixty(&game, "120Hz agreed");
    });
}

/// And on a display at the rate the game was written for, where every frame is one refresh.
#[test]
fn a_run_of_th08_on_a_sixty_hertz_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(60), "th08-paced-60");
        holds_at_sixty(&game, "60Hz agreed");
    });
}

/// And where the window is on one monitor while the compositor times another, which is the shape a second
/// screen makes: the rate is the compositor's to give and the run is still sixty.
#[test]
fn a_run_of_th08_on_a_split_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::split(143, 60), "th08-paced-split");
        holds_at_sixty(&game, "a 143Hz monitor a 60Hz compositor is timing");
    });
}

/// And where nothing will say the rate at all, which is the one case paced by the clock.
#[test]
fn a_run_of_th08_on_a_display_that_will_not_say_its_rate_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::unknown(), "th08-paced-unknown");
        holds_at_sixty(&game, "a display that will not say its rate");
    });
}

/// And the report line orb writes about it is this run's: what somebody reading a real log has, over a game
/// whose every frame is orb's own loop's.
#[test]
fn the_report_line_of_a_run_of_th08_says_what_the_frames_did() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(120), "th08-paced-reported");
        let log = game.log();
        game.frames_until_the_log_holds_another("us apart,");
        let said = log
            .lines()
            .into_iter()
            .rev()
            .find(|line| line.contains("us apart,"))
            .expect("a report of the frames of a run of several seconds");
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
    });
}
