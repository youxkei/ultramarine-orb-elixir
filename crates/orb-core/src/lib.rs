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
//! in it — which is why the device, the sound buffer and the glyphs are all behind the seam as well.
//!
//! What is *not* here is `orb`, which is code whose subject is another module's memory and nothing else:
//! `DllMain`, the jump written over a prologue, the import table entry swapped, the PE headers those are
//! read out of, the crash filter and the install lists. Nothing outside that crate names it, which is what
//! says the line is where it claims to be.
//!
//! **What orb does not do:**
//!
//! - Restore a snapshot across launches: it holds pointers to Direct3D and DirectSound objects, and those
//!   addresses differ per launch. A chapter is reached again by the run being played to it instead — see
//!   [`resume`].
//! - Keep more than one chapter of the same run written down. Its file holds the newest, and starting that
//!   same run from the beginning replaces it. Runs of different difficulties, characters and shots each have
//!   their own.
//! - Return the music to where it was across a track change: the track starts again from its beginning
//!   instead — see [`audio`].
//! - Keep a replay *of the game's own* for a pointdevice run, because the game's record of one is rewound
//!   with the run: the screen that offers to save one is skipped rather than the write being refused. What is
//!   kept instead is a replay of orb's own, which holds the attempts as well as the path — see [`replay`] —
//!   and it is watched on orb's own screens rather than the game's, since the game has none for it.
//! - Watch a pointdevice replay at anything but one attempt at a time, or fast-forward or pause an attempt
//!   being watched: the frames go by at the cadence the run was played at, and `x` stops it.
//! - Rewind sound effects on a restore. Only the music is restored.
//! - Accept an exe orb's addresses have not been read against, whatever put it there: a translation patch
//!   that rewrote the exe, a build of the game orb has not been taught, another game under a name orb reads.
//!   The launch is refused naming every game and build orb knows — see [`game::KNOWN`]. A patch that leaves
//!   the exe alone is not this: what orb reads off the file is its md5, and a patch applied while the game
//!   runs does not change one.
//! - Make the game itself run on a machine whose code page is not Japanese. That is what a locale emulator
//!   or thcrap's own locale independence is for — see [`score`] for the one thing orb does to the import the
//!   game opens its data files through.
//! - Find a thcrap that left no launcher in the game's directory, on the desktop or under the start menu:
//!   the game starts untranslated and the line says none is installed where the game is.

pub mod audio;
pub mod chapter;
pub mod crash;
pub mod dxgi;
pub mod faster;
pub mod find;
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
pub mod replay;
pub mod replay_ui;
pub mod resume;
pub mod resume_ui;
pub mod retry_ui;
pub mod runtime;
pub mod score;
pub mod shot;
pub mod snapshot;
pub mod sync;
pub mod trace;
pub mod tuning;
pub mod window;
pub mod writer;

/// One brush stroke, as coverage: what the mark over the lives is drawn from, baked out of
/// `brush.png` by `build.rs` rather than kept here as four thousand numbers.
mod brush {
    include!(concat!(env!("OUT_DIR"), "/brush.rs"));
}
