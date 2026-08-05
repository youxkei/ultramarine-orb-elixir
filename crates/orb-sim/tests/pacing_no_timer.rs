//! A host that will not give a millisecond timer.
//!
//! Without it `Sleep` is only accurate to the system's tick, some fifteen milliseconds — nearly two
//! refreshes at 120Hz, and exactly the size of the stutter measured whenever the pacing fell back to
//! the clock. orb carries on either way, so what has to hold is that it says so: a run that stutters
//! for this reason is one where the log is the only place the reason appears.

use orb_api::Hwnd;
use orb_sim::Sim;
use std::sync::Arc;

/// `TIMERR_NOCANDO`, which is what the call answers when it will not.
const NOCANDO: u32 = 97;

#[test]
fn a_refused_millisecond_timer_is_written_down() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    sim.display().set_foreground(Hwnd(1));
    sim.display().refuse_period(NOCANDO);

    orb_core::log::open();
    orb_core::frame::Pacing::new().configure();

    assert!(
        sim.log().said(&format!(
            "frame: the system will not give a 1ms timer ({NOCANDO}); waits will be coarse"
        )),
        "{:?}",
        sim.log().lines()
    );
    assert_eq!(
        sim.display().period_held(),
        None,
        "nothing to give back, since nothing was granted"
    );
}
