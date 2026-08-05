//! orb's logic, with no Windows in it.
//!
//! What the DLL does to a run is decided here and asked of the host through
//! [`orb_api`] — so the same code runs against the real Windows the game is loaded into
//! and against the simulated one `orb-sim` puts in front of it. Nothing in this crate names
//! `windows-sys`, which is what a test on a host that is not Windows depends on.

pub mod audio;
pub mod d3d8;
pub mod frame;
pub mod game;
pub mod log;
pub mod profile;
pub mod sync;
