//! A display whose rate is a whole multiple of sixty: the same number of blanks to every frame.
//!
//! The real `configure`, `settle`, `wait_for_slot` and `finished` over a compositor a test declares.
//! Nothing drove this loop before — `frame.rs`'s own tests are all arithmetic, `grid_aim` and
//! `whole_multiple` and the budget, handed numbers rather than run.
//!
//! Roughly the right rate rather than exact ticks. The host wakes the waiting thread when it gets round
//! to it and its compositor is slow now and then — see `orb-sim/src/display.rs` — so a scenario that
//! asserted a turn to the microsecond would be asserting that the host is a metronome, which none is.

mod pacing;

use pacing::{Display, NEAR_SIXTY, Run, for_each_seed};

const HZ: u32 = 120;
const WORK_US: i64 = 700;
/// Long enough for the allowance to have finished climbing; the climb itself costs refreshes.
const FRAMES: usize = 3_000;
const SETTLED: usize = 2_000;

#[test]
fn a_120hz_display_gets_two_blanks_a_frame_and_sixty_frames_a_second() {
    for_each_seed(|seed| {
        let mut display = Display::agreed(HZ);
        display.seed = seed;
        let mut run = Run::started(display);
        let waits = run.frames(FRAMES, WORK_US);

        // What `settle` made of the display, which is the decision everything below rests on.
        assert!(
            run.said("120Hz monitor, one frame every 2 blank(s)"),
            "seed {seed}: {:?}",
            run.lines()
        );

        let rate = run.fps(&waits, SETTLED);
        assert!(
            (rate - 60.0).abs() < NEAR_SIXTY,
            "seed {seed}: {rate} frames a second — {}",
            run.pacing().report()
        );

        // Two blanks to a frame, near enough always. A real host loses one now and then to a late wake
        // or a slow composition; what must not happen is a turn the grid never asked for.
        let counts = run.refreshes(&waits);
        let settled = &counts[SETTLED..];
        assert!(
            settled.iter().all(|count| (2..=3).contains(count)),
            "seed {seed}: turns of {settled:?} refreshes — {}",
            run.pacing().shown()
        );
    });
}
