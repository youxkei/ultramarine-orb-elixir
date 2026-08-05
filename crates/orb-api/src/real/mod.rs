//! The seam answered by the Windows that is actually there.
//!
//! Reached whenever no simulated Windows is installed, which in the DLL the game loads is
//! always. Nothing here is behind a runtime check in a build without the `sim` feature: the
//! facades in the parent modules compile down to these calls and nothing else.

pub mod clock;
pub mod display;
pub mod logfile;
pub mod mem;
pub mod module;
pub mod thread;
pub mod window;
