//! Everything that decides what happens to a run.
//!
//! **The rule is that the code an e2e test drives cannot reach Windows except through the seam**, and
//! that is what this crate is: every hook body, the chapters, the snapshots, the drawing, the menus and
//! the frame loop, with everything they ask of the host going through [`orb_api`]. So the same code runs
//! against the real Windows the game is loaded into and against the simulated one `orb-sim` puts in front
//! of it — which is the whole of what makes an e2e test evidence about a launch, [`runtime::attach_to`]
//! filling the same statics `orb`'s own install lists fill.
//!
//! **Checked rather than kept by hand.** `cargo xtask seam` builds this crate and `orb-sim` for a host
//! with no Windows on it, so a `windows-sys` import here fails to compile. A grep for that name would not
//! do: a COM vtable is Windows reached through a pointer the game handed over, and no crate's name appears
//! in it — which is why the device, the sound buffer and the glyphs are all behind the seam as well. See
//! [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
//!
//! What is *not* here is `orb`, which is code whose subject is another module's memory and nothing else:
//! `DllMain`, the jump written over a prologue, the import table entry swapped, the PE headers those are
//! read out of, the crash filter and the install lists. Nothing outside that crate names it, which is what
//! says the line is where it claims to be — see
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

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
pub mod mouse;
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
