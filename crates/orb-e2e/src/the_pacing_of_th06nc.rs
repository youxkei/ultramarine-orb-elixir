//! **orb's cadence over th06nc's own frame, with a run in it.**
//!
//! `pacing.rs` is the pacing itself — the budget, the allowance, the spin, the compositor's own lateness —
//! and all of that is orb's code, the same code whichever game is in front of it. What is the *game's* is
//! the frame that runs inside the loop, and this game's is a frame with no function of its own: `main`
//! (`0x1400454f0`) holds the whole of it inline between `0x140045ce0` and `0x140045ea5`, where th06c has
//! `0x1400256f0` to stand in for. So orb's jump stands on the fourteen bytes at `0x140045d7d` — the calc
//! chain's `lea`, `call`, `test` and `jne` — and the loop's eleven pieces are called from
//! `Th06nc::frame_in_the_loop` one at a time.
//!
//! **And this game's limiter is called rather than skipped, with its clock disarmed first.** orb calls
//! `0x14003c330` whole with the base at `0x140c22128` and the count at `0x140c22130` written to nought,
//! which puts every clock more than a period past `base + period × (count + 1)` and so starts the limiter
//! over without a wait: neither the sleep through `0x140c220f0` nor the `pause` spin at `0x14003c594` is
//! reached, and the whole of what the game used to spend waiting is orb's to spend. `Th06nc::end_drawing`
//! has why that and not the pieces of it, and `th06nc/asm.md`'s *The timing* has the reading.
//!
//! **What this game does more than once a frame it does for a replay and not for a run**: `0x140045d97`
//! walks the calc chain again up to seven more times, and its gate wants `[0x1404f278c]` set,
//! `[0x1404f27b0]` clear, the state at `[0x140c21d9c]` 2 and `[0x140a6ec60]` holding `0x8000` — a playback
//! being fast-forwarded. A run of this game is one walk a frame, so the rate a run comes out at is the rate
//! orb's loop hands frames over at and nothing else.
//!
//! So what this asks is the question a player asks — **does a run of this game come out at sixty frames a
//! second** — over the display shapes a machine really has, and with a run in progress rather than a launch
//! sitting at a menu.
//!
//! **Roughly the rate rather than the exact turn, and judged a second at a time**, for
//! `the_pacing_of_th07.rs`'s reasons: the host wakes a waiting thread when it gets round to it, and an
//! average over a whole run reads a second at 30 and a second at 90 as sixty.

use crate::fake::th06nc::Fake;
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

/// A Pointdevice mode run of this game on the display given, played for `SECONDS` of it.
///
/// Nothing fired at the player: a death in this mode is a retry offered, and a question nobody answers is a
/// game standing on one frame — which is every frame this is measuring.
fn a_run(display: Display, name: &str) -> Box<Fake> {
    let game = Fake::attach_configured(display, name, Work::wandering(WORK_US), |config| {
        config.log_level = LogLevel::Verbose;
    });
    game.fires_at_the_player(false);
    game.in_a_run_in_the_stage(0);
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
fn a_run_of_th06nc_on_an_ordinary_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(120), "th06nc-paced-120");
        holds_at_sixty(&game, "120Hz agreed");
    });
}

/// And on a display at the rate the game was written for, where every frame is one refresh.
#[test]
fn a_run_of_th06nc_on_a_sixty_hertz_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(60), "th06nc-paced-60");
        holds_at_sixty(&game, "60Hz agreed");
    });
}

/// And where the window is on one monitor while the compositor times another, which is the shape a second
/// screen makes: the rate is the compositor's to give and the run is still sixty.
#[test]
fn a_run_of_th06nc_on_a_split_display_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::split(143, 60), "th06nc-paced-split");
        holds_at_sixty(&game, "a 143Hz monitor a 60Hz compositor is timing");
    });
}

/// And where nothing will say the rate at all, which is the one case paced by the clock.
#[test]
fn a_run_of_th06nc_on_a_display_that_will_not_say_its_rate_holds_at_sixty() {
    in_its_own_process(|| {
        let game = a_run(Display::unknown(), "th06nc-paced-unknown");
        holds_at_sixty(&game, "a display that will not say its rate");
    });
}

/// And the report line orb writes about it is this run's: what somebody reading a real log has, over a game
/// whose every frame is orb's own loop's.
#[test]
fn the_report_line_of_a_run_of_th06nc_says_what_the_frames_did() {
    in_its_own_process(|| {
        let game = a_run(Display::agreed(120), "th06nc-paced-reported");
        let log = game.log();
        // Waited for rather than looked for: the report is written once a reporting period, and the line a
        // launch opens with is the cadence it settled on rather than a report of any frames.
        game.frames_until_the_log_holds_another("us apart,");
        let said = log
            .lines()
            .into_iter()
            .rev()
            .find(|line| line.contains("us apart,"))
            .expect("a report of the frames of a run of several seconds");

        // What the line is for: how many frames the period covered, how far apart they were handed over, how
        // much of the frame the drawing and the compositor were given, and how many reached the screen late.
        // Named rather than measured here — what each number should be is `pacing.rs`'s subject — because
        // what this says is that a run of *this* game produces them.
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
