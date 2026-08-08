//! Everything that decides what happens to a run.
//!
//! **The rule is that the code a scenario drives cannot reach Windows except through the seam**, and
//! that is what this crate is: every hook body, the chapters, the snapshots, the drawing, the menus and
//! the frame loop, with everything they ask of the host going through [`orb_api`]. So the same code runs
//! against the real Windows the game is loaded into and against the simulated one `orb-sim` puts in front
//! of it — which is the whole of what makes a scenario evidence about a launch, `orb::attach_to` filling
//! the same statics `hook::install` fills.
//!
//! **Checked rather than kept by hand.** `cargo xtask seam` builds this crate and `orb-sim` for a host
//! with no Windows on it, so a `windows-sys` import here fails to compile. A grep for that name would not
//! do: a COM vtable is Windows reached through a pointer the game handed over, and no crate's name appears
//! in it — which is why the device, the sound buffer and the glyphs are all behind the seam as well. See
//! [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
//!
//! What is *not* here is `orb`, which injects this into the game's process and does nothing else:
//! `DllMain`, the trampolines, the import table, the PE headers, the crash handler and the install
//! lists.

pub mod audio;
pub mod chapter;
pub mod frame;
pub mod game;
pub mod input;
pub mod joystick;
pub mod lives_ui;
pub mod log;
pub mod memtrack;
pub mod menu;
pub mod menu_ui;
pub mod mode;
pub mod mode_ui;
pub mod overlay;
pub mod profile;
pub mod resume;
pub mod resume_ui;
pub mod retry_ui;
pub mod runtime;
pub mod score;
pub mod snapshot;
pub mod sync;
pub mod tuning;
pub mod window;

/// One brush stroke, as coverage: what the mark over the lives is drawn from, baked out of
/// `brush.png` by `build.rs` rather than kept here as four thousand numbers.
mod brush {
    include!(concat!(env!("OUT_DIR"), "/brush.rs"));
}
