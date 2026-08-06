//! A compositor that starts taking longer, and the time it is given following it.
//!
//! What the compositor needs is not a threshold but a distribution, and orb's answer is to ratchet: a
//! frame that missed its blank adds `MISS_STEP_US` to what the compositor is given, and nothing takes it
//! back down. A compositor that slows under load is the case that ratchet exists for, and it is one no
//! test could make happen — a real one is not asked to be slow.

mod fake;
mod pacing;

use fake::{Display, in_its_own_process};
use orb_sim::Compose;
use pacing::{A_SECOND, NEAR_SIXTY, for_each_seed, launched};

const HZ: u32 = 120;
/// What it takes once something else on the desktop wants the GPU: more than the 2500µs orb starts by
/// allowing, so frames begin missing until the allowance has caught up.
const SLOW_COMPOSE_US: i64 = 4_000;
const WORK_US: i64 = 700;
/// What orb starts by allowing the compositor, before any frame has missed a blank.
const ALLOWED_AT_FIRST: i64 = 2_500;

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
            let game = launched(display, &format!("spikes-{seed}"), WORK_US);
            game.frames(15 * A_SECOND as u32);

            // The spikes were seen: the allowance climbed off the 2500µs it starts at, which only a frame
            // missing its blank does.
            pacing::until_reported(&game);
            let allowed = pacing::allowance_us(&game);
            assert!(
                allowed > ALLOWED_AT_FIRST,
                "seed {seed}: the allowance never moved, so no spike cost a frame its blank — {}",
                pacing::last_said(&game)
            );

            // And the rate is sixty all the same, which is what having an allowance is for. Most of the
            // seconds rather than all: the second a spike lands in loses a refresh, and that is the dip the
            // allowance then stops happening again.
            pacing::assert_mostly_sixty(&game, &game.handovers(), seed);
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
/// | 60Hz | 58.07 | 1 to 4 | 2500–2700µs |
/// | 120Hz | 59.03 | 0 to 1 | 2600–3000µs |
/// | 144Hz, 165Hz | 59.93 | none | 2600–3000µs |
/// | 240Hz | 59.52 | none | 2600–3000µs |
///
/// **60Hz is the rate with no room.** A missed blank there costs a whole refresh, which is a whole frame,
/// and the frame after it cannot come early enough to make it back — so the second reads 59.02 and one
/// seed lost four of its eleven seconds. From 144Hz up the same spike is absorbed and no second is off at
/// all.
///
/// 60Hz is also the rate where the pacing cannot tell a miss from a stage load: a cadence is a whole game
/// turn however fast the display is, so at 60Hz one refresh *is* one turn and the smallest miss there is
/// lands on the load guard's boundary, with jitter deciding which of the two it gets called. That shows
/// here as the allowance — one of the four seeds never moved it off its 2500µs start, its single miss
/// having been charged to a load that never happened. See `measure_compose` in `frame.rs`, whose grace is
/// one frame for this reason.
#[test]
fn what_a_spike_costs_depends_on_the_rate_and_never_on_the_run() {
    in_its_own_process(|| {
        for hz in [60u32, 120, 144, 165, 240] {
            for_each_seed(|seed| {
                let mut display = Display::agreed(hz);
                display.compose = Compose::spiking();
                display.seed = seed;
                let game = launched(display, &format!("spike-cost-{hz}-{seed}"), WORK_US);
                game.frames(15 * A_SECOND as u32);
                let handovers = game.handovers();
                let warm_up = pacing::GRACE_SECONDS * A_SECOND;

                let rate = pacing::fps(&handovers, warm_up);
                assert!(
                    (rate - 60.0).abs() < 1.0,
                    "{hz}Hz, seed {seed}: {rate} frames a second over the run — {}",
                    pacing::last_said(&game)
                );

                let share = pacing::share_at_sixty(&handovers, warm_up);
                let (at, worst) = pacing::worst_second(&handovers, warm_up);
                assert!(
                    share >= 0.5,
                    "{hz}Hz, seed {seed}: only {share} of the seconds were sixty, worst {worst} at second \
                     {at} — {}",
                    pacing::last_said(&game)
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
            let game = launched(display, &format!("slows-{seed}"), WORK_US);

            // Nothing has missed, so the allowance is still what it started at.
            pacing::until_reported(&game);
            let allowed = pacing::allowance_us(&game);
            assert_eq!(
                allowed,
                ALLOWED_AT_FIRST,
                "seed {seed}: the compose time orb starts by allowing — {}",
                pacing::last_said(&game)
            );

            // The desktop gets busy.
            let quick = game.handovers().len();
            game.sim()
                .display()
                .set_compose(Compose::flat(SLOW_COMPOSE_US));
            game.frames(2_000);

            // The allowance was raised past what the compositor now really takes. Not to it exactly — the
            // ratchet is a hundred microseconds a miss and it stops as soon as the frames land — so what is
            // asserted is that it covers it.
            let allowed = pacing::allowance_us(&game);
            assert!(
                allowed >= SLOW_COMPOSE_US,
                "seed {seed}: {allowed}us allowed for a compositor taking {SLOW_COMPOSE_US}us — {}",
                pacing::last_said(&game)
            );

            // And having caught up, the rate is sixty again — the misses were the climb and not a rate.
            let handovers = game.handovers();
            let rate = pacing::fps(&handovers[quick..], 1_500);
            assert!(
                (rate - 60.0).abs() < NEAR_SIXTY,
                "seed {seed}: {rate} frames a second after the climb — {}",
                pacing::last_said(&game)
            );
        });
    });
}
