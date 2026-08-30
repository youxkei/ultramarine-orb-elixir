//! Who writes the log: not the thread the frame runs on.
//!
//! A `WriteFile` takes what a disk, an antivirus reading every write or a folder being synced decides.
//! Measured on th07: one of them took **751621µs** while that frame's own work was under a millisecond,
//! and two more took 230ms and 178ms — three of the stutters somebody watching the run noticed. Holding
//! the line for the frame's own slack, which is what this used to do, only moves such a write into the
//! next frame's turn; so the frame's thread formats its line and hands it over, and a thread of orb's own
//! puts it on the disk.
//!
//! What is asserted here is the frame's side of that: it says a line, it writes nothing, and the line is
//! in the file. The count is the instrument — `log::spent`, which the frame loop's own line reports as
//! `log Nus in M writes here` — so a launch that went back to writing in the frame would say so.
//!
//! One test to a file — see `log_writes.rs`. This one needs a file of its own for the reason
//! `log_off_thread.rs` does: it claims a thread as the frame's, and a test in the same binary claiming
//! another would leave both of them asking about a thread that is not theirs.

use std::sync::Arc;

use orb_config::LogLevel;
use orb_sim::Sim;

/// How long a line is waited for, in tries of a millisecond each. Far longer than a hand-over and a write
/// of a few bytes takes, and a bound all the same: a test that waits forever for a thread that never came
/// says nothing about why.
const WAITS_FOR: usize = 2_000;

#[test]
fn the_frames_own_thread_says_its_lines_and_writes_none_of_them() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();
    orb_core::log::open();
    orb_core::log::set_level(LogLevel::Verbose);
    orb_core::log::set_pacing(true);

    // The opening line was written before there was a thread to hand it to, which is `DllMain`'s case:
    // the loader lock is held there and a thread made under it waits for the caller to let go.
    assert!(sim.log().said("---- new run ----"));
    let before = orb_core::log::spent();
    assert_eq!(
        before.other_writes, 1,
        "the line every run opens with was not written where it was said"
    );
    // Counted as another thread's because there was no frame's thread yet — the claim below is what makes
    // one — which is what every launch's first lines are: `DllMain`'s.
    assert_eq!(before.writes, 0);

    // Claimed by this thread and the writer started, which is what the frame loop does once a frame.
    orb_core::log::off_the_frame();

    for said in [
        "retry chapter 7 (retry 1)",
        "f60 stage 1 clears=true",
        "state: cur=1 wanted=1",
        "frame: 2 refreshes, 33000us",
    ] {
        // Every way in, because they are one way in now: what a level or a switch decides is whether the
        // line is said at all, and a line said is a line handed over.
        match said {
            "retry chapter 7 (retry 1)" => orb_core::log!("{said}"),
            "f60 stage 1 clears=true" => orb_core::summary!("{said}"),
            "state: cur=1 wanted=1" => orb_core::detail!("{said}"),
            _ => orb_core::pacing!("{said}"),
        }
    }

    // **Nothing written here**, which is the whole of it: whatever the disk is doing, it is not this
    // thread's to wait for.
    let cost = orb_core::log::spent();
    assert_eq!(
        cost.writes, 0,
        "the frame's own thread wrote {} line(s) of the log, taking {}us",
        cost.writes, cost.us
    );

    // And in the file, in the order they were said: a log is read as a sequence of moments, so a queue
    // that came out in another order would be a run that reads as one that did not happen.
    for said in [
        "retry chapter 7 (retry 1)",
        "f60 stage 1 clears=true",
        "state: cur=1 wanted=1",
        "frame: 2 refreshes, 33000us",
    ] {
        waits_for(&sim, said);
    }
    let lines = sim.log().lines();
    let at = |text: &str| {
        lines
            .iter()
            .position(|line| line.contains(text))
            .unwrap_or_else(|| panic!("{text:?} among {lines:?}"))
    };
    assert!(at("retry chapter 7") < at("f60 stage 1"));
    assert!(at("f60 stage 1") < at("state: cur=1"));
    assert!(at("state: cur=1") < at("frame: 2 refreshes"));

    // What the thread wrote is counted as its own, which is the other half of the same instrument: the
    // writing did not stop happening, it stopped happening here.
    //
    // Summed over both reads, `spent` owning the interval since the last one: the thread writes while this
    // one goes on asking, so which of the two reads a line falls in is the scheduler's to decide.
    let by_the_thread = orb_core::log::spent();
    assert_eq!(
        cost.other_writes + by_the_thread.other_writes,
        4,
        "the thread that writes the log is not counted as having written it"
    );

    // And what is still waiting when the log closes goes in rather than being lost, wherever that call
    // comes from: the crash filter reaches it on the thread that faulted.
    orb_core::detail!("state: cur=2 wanted=2");
    orb_core::log::close();
    assert!(
        sim.log().said("state: cur=2 wanted=2"),
        "a line still waiting when the log closed was lost: {:?}",
        sim.log().lines()
    );
    assert!(sim.log().closed());
}

/// Waits for a line to reach the file, the thread that writes it being another one.
fn waits_for(sim: &Sim, said: &str) {
    for _ in 0..WAITS_FOR {
        if sim.log().said(said) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("{said:?} never reached the file: {:?}", sim.log().lines());
}
