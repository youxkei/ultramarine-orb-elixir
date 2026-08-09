//! The machine's own code page, which is what winmm answers a device's name in.
//!
//! **A module of its own, and neither of the two it could have been put in.** `module.rs` opens *the
//! modules loaded into the process orb is injected into* and a code page is not one; `text.rs` opens
//! *the font a string is baked through* and a conversion is not that either. One function in a file
//! that says what it is beats a second subject in a file that already has one. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

/// `MultiByteToWideChar` with `CP_ACP`, for bytes a Win32 `-A` call answered in whatever code page
/// this machine is set to.
///
/// Lossy, and there is nothing else for it to be: the log is UTF-8 and a byte the code page has no
/// character for is a byte nobody can read either way.
pub fn text(bytes: &[u8]) -> String {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.codepage_text(bytes);
    }
    host::text(bytes)
}

#[cfg(windows)]
use crate::real::codepage as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub fn text(_bytes: &[u8]) -> String {
        no_windows("codepage::text")
    }
}
