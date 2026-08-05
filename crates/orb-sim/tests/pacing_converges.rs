//! Sixty frames a second, whatever the compositor takes.
//!
//! This is the whole of what the pacing promises on a display it is not refusing: the allowance is
//! measured rather than chosen, so it climbs until the frames stop missing, and once they stop every
//! frame lands on the blank the sixtieth-of-a-second grid meant. What the compositor takes does not
//! come into it — that is what having an allowance is *for*.
//!
//! So the invariant is not "sixty for the compose times we tried". It is **sixty, eventually, for any
//! of them**, and a compose time that does not reach sixty is a defect rather than a limit.
//!
//! Held over enough frames for the climb to finish — the ratchet is a hundred microseconds a miss —
//! and read off the last thousand rather than the whole run, since the climb itself is allowed to cost
//! frames.

mod pacing;

use orb_sim::Compose;
use pacing::{Display, NEAR_SIXTY, Run};

/// What the game's own update and draw take, read off a real run's `report()`: "(694us to draw + …)".
const WORK_US: i64 = 700;
const FRAMES: usize = 3_000;
const SETTLED: usize = 2_000;

/// The rates a display reports itself as, over the range anyone plays at.
const RATES: [u32; 5] = [60, 120, 144, 165, 240];

/// What a compositor might take over a frame, in microseconds. The upper end is not invented: a real
/// session was seen reaching about 3.5ms, and `DONE.md`'s mixed-rate run had orb's own allowance climb
/// to 3900µs chasing it.
const COMPOSE_US: [i64; 8] = [400, 1_000, 2_000, 2_500, 3_000, 3_200, 3_800, 4_000];

fn settled_rate(hz: u32, compose: Compose, seed: u64) -> (f64, String) {
    let mut display = Display::agreed(hz);
    display.compose = compose;
    display.seed = seed;
    let mut run = Run::started(display);
    let waits = run.frames(FRAMES, WORK_US);
    let rate = run.fps(&waits, SETTLED);
    let shown = run.pacing().shown();
    (rate, shown)
}

/// The one that holds: a compositor whose cost is what this machine's appears to be — usually a
/// millisecond, occasionally three and a half.
#[test]
fn a_compositor_that_spikes_still_settles_at_sixty() {
    for hz in RATES {
        for seed in 0..4u64 {
            let (rate, shown) = settled_rate(hz, Compose::measured(), seed);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "{hz}Hz, seed {seed}: {rate} frames a second after {SETTLED} frames\n  {shown}"
            );
        }
    }
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
#[test]
fn any_compositor_inside_the_ceiling_settles_at_sixty() {
    let mut stuck = Vec::new();
    let mut tried = 0;
    for hz in RATES {
        for compose_us in COMPOSE_US {
            if compose_us > ceiling_us(hz) {
                continue;
            }
            tried += 1;
            let (rate, shown) = settled_rate(hz, Compose::flat(compose_us), 0);
            if (rate - 60.0).abs() >= NEAR_SIXTY {
                stuck.push(format!(
                    "{hz}Hz at {compose_us}us: {rate:.2} fps\n    {shown}"
                ));
            }
        }
    }
    assert!(
        stuck.is_empty(),
        "{} of {tried} compose times never reach sixty:\n  {}",
        stuck.len(),
        stuck.join("\n  ")
    );
}

/// And past the ceiling it cannot, which is a limit rather than a defect — but nothing in orb says so.
///
/// Of the rates anyone plays at only 240Hz has a refresh short enough for a plausible compositor to want
/// three quarters of it: 3124µs against the 3200µs and up that a busy desktop reaches. There the
/// allowance climbs to exactly the ceiling and stops, no miss is charged to a load, and the rate settles
/// at 48.00 — every fifth frame taking an extra refresh, for the rest of the run.
///
/// What is asserted is that shape and not a number to be satisfied with: the allowance ends *at* the
/// ceiling, so what is missing is room and not a climb. A run reading 48fps with the allowance below the
/// ceiling would be the latch back again, and this would catch it.
#[test]
fn past_the_ceiling_the_rate_is_not_sixty_and_the_allowance_is_out_of_room() {
    const HZ: u32 = 240;
    for compose_us in COMPOSE_US {
        if compose_us <= ceiling_us(HZ) {
            continue;
        }
        let mut display = Display::agreed(HZ);
        display.compose = Compose::flat(compose_us);
        display.seed = 0;
        let mut run = Run::started(display);
        let waits = run.frames(FRAMES, WORK_US);
        let rate = run.fps(&waits, SETTLED);
        let (_, _, allowed) = run.pacing().status();

        assert_eq!(
            allowed,
            ceiling_us(HZ),
            "{HZ}Hz at {compose_us}us: the allowance stopped at {allowed}us, not at the ceiling — {}",
            run.pacing().shown()
        );
        assert!(
            (rate - 60.0).abs() >= NEAR_SIXTY,
            "{HZ}Hz at {compose_us}us: {rate} frames a second, which is sixty — so the ceiling was room \
             enough after all and this test is the thing that is wrong"
        );
    }
}
