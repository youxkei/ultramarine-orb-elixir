//! The seam answered by the Windows that is actually there.
//!
//! Reached whenever no simulated Windows is installed, which in the DLL the game loads is
//! always. Nothing here is behind a runtime check in a build without the `sim` feature: the
//! facades in the parent modules compile down to these calls and nothing else.

pub mod clock;
pub mod codepage;
pub mod d3d8;
pub mod display;
pub mod dsound;
pub mod joystick;
pub mod keyboard;
pub mod logfile;
pub mod mem;
pub mod module;
pub mod mouse;
pub mod process;
pub mod text;
pub mod thread;
pub mod window;
pub mod xinput;
