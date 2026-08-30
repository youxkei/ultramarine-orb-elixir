//! A line from a thread that is not the frame's, which reaches the file the same way.
//!
//! Every line is handed to the thread that writes the log — see `log_written_off_the_frame.rs` — and that
//! is not a thing the frame's own thread gets and the others do not: a thread of orb's own (the joystick
//! poller is one) says its line, hands it over and goes back to what it was doing.
//!
//! What is asserted here is that a line said off the frame's thread is not lost, and that the frame's own
//! accounting stays the frame's: what another thread said is counted as another thread's, so a run whose
//! frames are late cannot be read as the log's fault when the log was written elsewhere entirely.
//!
//! One test to a file — see `log_writes.rs`. This one needs a file of its own more than most: it
//! claims a thread as the frame's, and a test in the same binary claiming another would leave both
//! of them asking about a thread that is not theirs.

use std::sync::Arc;

use orb_sim::Sim;

/// How long a line is waited for, in tries of a millisecond each — see `log_written_off_the_frame.rs`.
const WAITS_FOR: usize = 2_000;

#[test]
fn a_line_from_another_thread_reaches_the_file_and_is_counted_as_that_threads() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    orb_core::log::open();
    orb_core::log::set_pacing(true);

    // Claimed by *this* thread first, so that the one spawned below is genuinely not the frame's — and
    // the writer started, which is the same call.
    orb_core::log::off_the_frame();
    let _opening = orb_core::log::spent();

    let sim_for_thread = Arc::clone(&sim);
    std::thread::spawn(move || {
        let _installed = sim_for_thread.enter();
        orb_core::pacing!("joystick: no pad answered");
    })
    .join()
    .unwrap();

    for _ in 0..WAITS_FOR {
        if sim.log().said("joystick: no pad answered") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(
        sim.log().said("joystick: no pad answered"),
        "a line said off the frame's thread never reached the file: {:?}",
        sim.log().lines()
    );

    // And the frame's own thread wrote none of it, that line having been neither said nor written here.
    let cost = orb_core::log::spent();
    assert_eq!(
        cost.writes, 0,
        "the frame's thread was charged for a line another thread said",
    );
}
