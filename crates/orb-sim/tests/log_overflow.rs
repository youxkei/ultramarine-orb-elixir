//! What happens to held lines when the frame loop has stopped draining.
//!
//! A drain happens twice per frame's turn, so a great many lines held means no better moment is
//! coming — the frame loop has stopped. They are written where they stand rather than lost, which
//! for a run that ended in a fault is the difference between having the last frames and not.
//!
//! One test to a file — see `log_writes.rs`.

use std::sync::Arc;

use orb_sim::Sim;

#[test]
fn lines_held_with_no_drain_coming_are_written_where_they_stand() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    orb_core::log::open();
    orb_core::log::set_pacing(true);

    // Claims this thread as the frame's, so what follows is held rather than written.
    orb_core::log::drain();

    // Far more than a frame makes, and no drain between them. The ceiling is orb's to pick and this
    // test does not name it — only that there is one, that reaching it says so, and that nothing
    // held is lost.
    const DEFERRED: usize = 200;
    for frame in 0..DEFERRED {
        orb_core::pacing!("frame {frame}: held with nothing draining");
    }

    assert!(
        sim.log().said("the frame loop has stopped draining"),
        "the giving-up line is what says the log is no longer waiting for a frame"
    );
    assert!(
        sim.log().said("frame 0: held with nothing draining"),
        "reaching the ceiling writes what was held, without waiting for a drain that is not coming"
    );

    // What has been deferred since the ceiling was last reached is still held rather than written:
    // giving up is per batch, not once and for all, so the loop coming back to life is still the
    // cheap moment for the rest of them.
    orb_core::log::drain();
    for frame in 0..DEFERRED {
        let held = format!("frame {frame}: held with nothing draining");
        assert!(sim.log().said(&held), "{held:?} was dropped");
    }
}
