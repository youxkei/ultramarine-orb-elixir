//! A line from a thread that is not the frame's, which has no frame to wait for.
//!
//! One test to a file — see `log_writes.rs`. This one needs a file of its own more than most: it
//! claims a thread as the frame's, and a test in the same binary claiming another would leave both
//! of them asking about a thread that is not theirs.

use std::sync::Arc;

use orb_sim::Sim;

#[test]
fn a_line_from_another_thread_is_written_where_it_stands() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    orb_core::log::open();
    orb_core::log::set_pacing(true);

    // Claimed by *this* thread first, so that the one spawned below is genuinely not the frame's.
    orb_core::log::drain();

    // A thread of orb's own — the joystick poller is one — has no frame to wait for, so holding
    // its line back would only make it late.
    let sim_for_thread = Arc::clone(&sim);
    std::thread::spawn(move || {
        let _installed = sim_for_thread.enter();
        orb_core::pacing!("joystick: no pad answered");
    })
    .join()
    .unwrap();

    assert!(
        sim.log().said("joystick: no pad answered"),
        "written without a drain, because no drain of this thread's is coming"
    );
}
