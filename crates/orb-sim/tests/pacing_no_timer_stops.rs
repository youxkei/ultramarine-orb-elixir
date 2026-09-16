//! A host that cannot make the timer every wait is made on: the frame's turn is not waited out.
//!
//! The other half of `pacing_no_timer.rs`, whose header has the whole of why: orb says so
//! and *stops*, rather than pacing on with a wait it does not have.
//!
//! **A file of its own rather than a second test beside that one**, which is what it was: orb's log is
//! process-global by design — the file handle, the level and the thread that writes it are statics — so two
//! tests in one binary are two runs writing each other's log. Which is what the thread showed: the writer
//! carries the simulated Windows installed where it was spawned, so the lines of whichever test started it
//! went into that test's log wherever they were said.
//!
//! Paced by the clock, which is the path with no flush in it, so the only thing that could move the
//! counter by a turn is the wait itself.

use orb_api::Hwnd;
use orb_core::frame::Pacing;
use orb_sim::Sim;
use std::sync::Arc;

const WINDOW: Hwnd = Hwnd(0x1234);

/// One turn of the game's logic, which is what a frame waits out and what a frame that gave up does
/// not.
const TURN_US: i64 = 1_000_000 / frame_hz();
const fn frame_hz() -> i64 {
    orb_core::frame::LOGIC_HZ as i64
}

#[test]
fn the_frames_turn_is_not_waited_out_after_that() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    sim.clock().refuse_the_timer();

    orb_core::log::open();
    // What the log says is read after what has been said is written: the frame loop hands its lines to a
    // thread of orb's own — see `orb_core::log` — and the loop here is driven a turn at a time.
    sim.log().waits_first(orb_core::log::written_out);
    let mut pacing = Pacing::new();
    pacing.configure();
    let before = sim.clock().peek();
    pacing.wait_for_slot(WINDOW);
    let waited = orb_sim::Clock::micros_for_ticks(sim.clock().peek() - before);

    assert!(
        waited < TURN_US / 2,
        "{waited}us of a {TURN_US}us turn was waited out on a host that cannot wait"
    );
    assert_eq!(sim.exited(), Some(1), "{:?}", sim.log().lines());
}
