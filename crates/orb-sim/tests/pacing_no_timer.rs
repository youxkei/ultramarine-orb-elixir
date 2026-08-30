//! A host that cannot make the timer every wait is made on.
//!
//! The wait to a frame's own deadline is a `CREATE_WAITABLE_TIMER_HIGH_RESOLUTION` waitable timer,
//! which is Windows 10 1803's, and there is no second wait behind it: `Sleep` kept as a spare would
//! be a slow path that ships, and a launch that paced badly while its log said otherwise is what
//! `frame.rs` is written against. So what has to hold is that orb says so where somebody will read
//! it and *stops* — see
//! [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md),
//! decisions 3 to 5.
//!
//! Writable at all only because the dialog and the giving up are behind the seam: an e2e test that
//! raised a real `MessageBoxW` would wait for a click that is never coming, and one that really
//! exited would take the harness's child with it.
//!
//! One test to a file — see `log_writes.rs`, and `pacing_no_timer_stops.rs`, which is the other half of
//! this one and was a second test here until orb's log gained a thread of its own to write it on.

use orb_api::Hwnd;
use orb_core::frame::Pacing;
use orb_sim::{Compose, Sim};
use std::sync::Arc;

const WINDOW: Hwnd = Hwnd(0x1234);
const HZ: u32 = 120;

#[test]
fn a_host_that_cannot_make_the_timer_is_told_so_and_the_launch_ends() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    sim.attach_display(WINDOW, HZ, Compose::ordinary());
    sim.clock().refuse_the_timer();

    orb_core::log::open();
    // What the log says is read after what has been said is written: the frame loop hands its lines to a
    // thread of orb's own — see `orb_core::log` — and the loop here is driven a turn at a time.
    sim.log().waits_first(orb_core::log::written_out);
    let mut pacing = Pacing::new();
    pacing.configure();
    pacing.wait_for_slot(WINDOW);

    assert!(
        sim.log().said(
            "frame: the host cannot create a high-resolution timer, which every wait is made on; stopping"
        ),
        "{:?}",
        sim.log().lines()
    );

    // Said to somebody and not only to the log, because the log is inside a game that is about to
    // stop. Which version it needs is in the message: the answer to "why here and not on the other
    // machine" is a Windows this one is older than.
    let dialogs = sim.dialogs();
    assert_eq!(dialogs.len(), 1, "{dialogs:?}");
    let (title, text) = &dialogs[0];
    assert_eq!(title, "Ultramarine Orb Elixir");
    assert!(text.contains("1803"), "{text}");
    assert!(text.contains("high-resolution timer"), "{text}");

    assert_eq!(
        sim.exited(),
        Some(1),
        "the process is ended, and not with a code that reads as a run that finished"
    );
}
