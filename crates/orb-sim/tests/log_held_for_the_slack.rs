//! Which lines the frame's own thread holds back, and which it writes where it stands.
//!
//! Everything written because time passed waits for the slack: what the pacing says about itself, the
//! summary a second brings and the detail a frame does. What is written because something happened
//! does not, a run that ends in a crash or a hang having to have its last lines in the file.
//!
//! Where the real loop drains is `orb-e2e`'s `pacing::log_deferral`, driven by a game whose own loop
//! calls `render`. What is here is which macro takes which way in, which no game decides.
//!
//! One test to a file — see `log_writes.rs`. This one needs a file of its own for the reason
//! `log_off_thread.rs` does: it claims a thread as the frame's, and a test in the same binary
//! claiming another would leave both of them asking about a thread that is not theirs.

use std::sync::Arc;

use orb_config::LogLevel;
use orb_sim::Sim;

#[test]
fn what_a_run_says_as_time_passes_waits_for_the_slack_and_a_fault_does_not() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    orb_core::log::open();
    orb_core::log::set_level(LogLevel::Verbose);
    orb_core::log::set_pacing(true);

    // Claimed by this thread, which is what the frame loop's first drain does.
    orb_core::log::drain();

    orb_core::summary!("f60 stage 1 clears=true");
    orb_core::detail!("state: cur=1 wanted=1");
    orb_core::pacing!("frame: 2 refreshes, 33000us");
    assert!(!sim.log().said("f60 stage 1 clears=true"));
    assert!(!sim.log().said("state: cur=1 wanted=1"));
    assert!(!sim.log().said("frame: 2 refreshes, 33000us"));

    orb_core::log!("retry chapter 7 (retry 1)");
    assert!(sim.log().said("retry chapter 7 (retry 1)"));

    // The far side of the next flush, where what is left of the turn is slack.
    orb_core::log::drain();
    for held in [
        "f60 stage 1 clears=true",
        "state: cur=1 wanted=1",
        "frame: 2 refreshes, 33000us",
    ] {
        assert!(
            sim.log().said(held),
            "wanted {held:?} among {:?}",
            sim.log().lines()
        );
    }

    // And what is still held when the log is closed goes in rather than being lost: the crash filter
    // closes it from the thread that faulted, which for a fault inside a frame is this one.
    orb_core::detail!("state: cur=2 wanted=2");
    assert!(!sim.log().said("state: cur=2 wanted=2"));
    orb_core::log::close();
    assert!(sim.log().said("state: cur=2 wanted=2"));
    assert!(sim.log().closed());
}
