//! Ending the process orb is inside.
//!
//! One call, for one case: a host that cannot make the timer the frame loop waits on is a host orb
//! does not run on, and what orb does about it is say so and stop — see
//! [docs/adr/0006](../../../docs/adr/0006-the-frame-loop-waits-on-a-high-resolution-timer.md).
//!
//! Behind the seam because a suite that really exited would take the harness's child with it, so
//! there is behaviour here no scenario could otherwise reach.

/// `ExitProcess`. Does not return on a real host.
///
/// A `panic!` would not do instead: the crate aborts on panic and the crash handler would write a
/// `module+offset` line, which says a fault in orb where what happened is a host orb does not
/// support.
pub fn exit(code: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.exit_process(code);
    }
    host::exit(code);
}

#[cfg(windows)]
use crate::real::process as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn exit(_code: u32) {
        no_windows("process::exit")
    }
}
