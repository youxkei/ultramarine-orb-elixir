//! What orb writes to its log, driven with no Windows under it.
//!
//! The real `orb_core::log` throughout — its statics, its formatting, its level filtering — with
//! only the file and the clock behind it answered by a simulated Windows. That is the whole shape
//! of these suites: orb's code, a host that is not there.
//!
//! **A unit test of the log, and not a scenario**, which is why it is still here and not among the
//! runs in `orb/tests`: what `log!` formats and which level keeps which line is not something a game
//! decides, so there is nothing for a game to drive. What *is* a scenario is where the file goes —
//! beside the game — and that is asserted over a whole run as well.
//!
//! One test to a file, because orb's log is process-global by design: the file handle, the level
//! and the frame's thread are statics, and two tests in one binary would be writing each other's
//! log. A file is a binary of its own, so each of these owns its process — which `orb-core`'s own
//! `#[cfg(test)]` cannot give, every one of its tests sharing a binary.

use std::path::PathBuf;
use std::sync::Arc;

use orb_config::LogLevel;
use orb_sim::Sim;

#[test]
fn the_log_is_written_beside_the_game_through_the_seam() {
    let sim = Arc::new(Sim::new());
    let _installed = sim.enter();

    // Not the directory this test runs in, and not orb's own: the exe orb is injected into is the
    // game's, and the log belongs beside it because that is where `orb.yaml` and the launcher are.
    let installed_at = PathBuf::from("install");
    sim.set_host_exe(installed_at.join("th06.exe"));

    orb_core::log::open();
    assert_eq!(sim.log().path(), Some(installed_at.join("orb.log")));

    // The line every run opens with. A log without it cannot say whether the run even happened.
    assert!(sim.log().said("---- new run ----"));
    // Appended to rather than started over: a run worth looking at is often over by the time
    // anyone looks.
    assert!(!sim.log().restarted());

    orb_core::log!("startup: orb {}", 42);
    assert!(sim.log().said("startup: orb 42"));

    // Every line carries the host's own millisecond count, so what a run did can be lined up
    // against when it did it.
    sim.clock().advance_micros(5_000_000);
    orb_core::log!("five seconds in");
    let stamped = format!("[{:>8}ms] five seconds in", sim.clock().ticks());
    assert!(
        sim.log().said(&stamped),
        "wanted {stamped:?} among {:?}",
        sim.log().lines()
    );

    // At `quiet` the file holds the startup lines and the faults and nothing else — which is the
    // level a run is played at when the log itself is among the suspects.
    orb_core::log::set_level(LogLevel::Quiet);
    orb_core::summary!("a summary nobody asked for");
    orb_core::detail!("a detail nobody asked for");
    orb_core::log!("a fault, which every level keeps");
    assert!(!sim.log().said("nobody asked for"));
    assert!(sim.log().said("a fault, which every level keeps"));

    // And at `verbose` the per-frame writers come back on.
    orb_core::log::set_level(LogLevel::Verbose);
    orb_core::detail!("frame 1886 was late by 2ms");
    assert!(sim.log().said("frame 1886 was late by 2ms"));

    orb_core::log::close();
    assert!(sim.log().closed());
}
