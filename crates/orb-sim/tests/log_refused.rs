//! A launch that never started, and what it leaves behind.
//!
//! Here rather than in `orb-e2e` for the same reason as the rest of these: there is no game to drive
//! it. A refusal happens in the launcher, before a process has been created — so what a scenario would
//! have to stand up is the one thing that never exists on this path.
//!
//! One test to a file, because the log's handle and level are statics and two tests in one binary
//! would be writing each other's log.

use std::path::PathBuf;
use std::sync::Arc;

use orb_sim::Sim;

#[test]
fn a_refused_launch_says_so_in_the_log_and_opens_no_run() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();

    // The launcher's own exe, this being the one write to the log that happens outside a game: with no
    // game started there is no game's exe for the file to be beside, and `orb.exe` is installed in that
    // same directory.
    let installed_at = PathBuf::from("install");
    sim.set_host_exe(installed_at.join("orb.exe"));

    orb_core::log::refused("unexpected argument '--pacing' found, and the game was not started");

    // The file a run would have written to, so that a launch that failed and one that worked are read
    // in one place.
    assert_eq!(sim.log().path(), Some(installed_at.join("orb.log")));
    assert!(sim.log().said(
        "launch refused: unexpected argument '--pacing' found, and the game was not started"
    ));

    // And nothing saying a run began, which is what the file would claim if this went through the
    // opening every run does.
    assert!(!sim.log().said("---- new run ----"));

    // Appended rather than started over: a refusal is not a reason to throw away the run before it.
    assert!(!sim.log().restarted());

    // Closed, because the launcher is about to end and a line still held is a line nobody reads.
    assert!(sim.log().closed());
}
