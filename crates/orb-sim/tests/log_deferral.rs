//! Lines the frame loop holds back until writing one costs nothing.
//!
//! The moments between handing a frame over and the blank it is shown at are the ones a write must
//! stay out of, so what the pacing says about itself is held and written on the far side of the
//! wait. Nothing tested this before there was a host to run it without: the mechanism turns on
//! which thread is asking, and orb's own statics make that a question about the whole process.
//!
//! One test to a file — see `log_writes.rs`.

use std::sync::Arc;

use orb_sim::Sim;

#[test]
fn what_the_frame_loop_defers_is_written_when_it_drains() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    orb_core::log::open();
    orb_core::log::set_pacing(true);

    // The first drain claims this thread as the frame's. Nothing is held for it yet, which is why
    // it returns without writing.
    orb_core::log::drain();

    orb_core::pacing!("frame 1: waited 3ms for the blank");
    assert!(
        !sim.log().said("frame 1"),
        "held back, not written: writing it where it is worked out is what costs the frame"
    );

    orb_core::log::drain();
    assert!(sim.log().said("frame 1: waited 3ms for the blank"));

    // Held in the order they were deferred, so a run reads as a sequence of frames.
    orb_core::pacing!("frame 2: on the blank");
    orb_core::pacing!("frame 3: a refresh late");
    orb_core::log::drain();
    let lines = sim.log().lines();
    let at = |needle: &str| {
        lines
            .iter()
            .position(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("no line holds {needle:?} among {lines:?}"))
    };
    assert!(at("frame 2") < at("frame 3"));
}
