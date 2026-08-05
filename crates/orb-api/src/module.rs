//! The modules loaded into the process orb is injected into.

use std::path::PathBuf;

/// The path of the exe this is running inside — the game's, since that is the process orb is
/// injected into. `orb.yaml` and the log go in its directory, which is where the launcher is
/// installed too.
///
/// Not this DLL's own directory: the launcher carries the DLL inside itself and unpacks it to the
/// temp directory, so asking where the DLL is would find neither the config nor a sensible place
/// for the log.
pub fn host_exe() -> Option<PathBuf> {
    path(None)
}

/// Where a loaded module's file is. `None` for the exe of this process.
///
/// Used by the crash handler to say which module an address is in, which is worth having: most of
/// what has gone wrong here was found by reading `module+offset` out of the log.
pub fn path(module: Option<usize>) -> Option<PathBuf> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.module_path(module);
    }
    host::path(module)
}

/// Whether a module is loaded into this process at all.
///
/// Asked apart from [`proc_address`] because the two failures are different findings and the log
/// says so: a winmm that is not there means the music cannot be rewound exactly, and a winmm that is
/// there without the symbol asked for means something has replaced it.
pub fn loaded(module: &str) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.module_loaded(module);
    }
    host::loaded(module)
}

/// The address of a function exported by an already-loaded module, or `None` where either is not
/// there. The module is not loaded if it is not already: what is wanted is the winmm the *game*
/// has open, and loading a second copy would answer about the wrong one.
pub fn proc_address(module: &str, name: &str) -> Option<usize> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.proc_address(module, name);
    }
    host::proc_address(module, name)
}

#[cfg(windows)]
use crate::real::module as host;

#[cfg(not(windows))]
mod host {
    use crate::no_windows;
    use std::path::PathBuf;

    pub fn path(_module: Option<usize>) -> Option<PathBuf> {
        no_windows("module::path")
    }
    pub fn loaded(_module: &str) -> bool {
        no_windows("module::loaded")
    }
    pub fn proc_address(_module: &str, _name: &str) -> Option<usize> {
        no_windows("module::proc_address")
    }
}
