//! The font a string is baked through.
//!
//! Everything the drawing asks of a rasteriser: a face at a size, a string baked to a coverage mask,
//! and the face given back. What is on the far side is the GDI — see [`crate::real::text`] — or a
//! simulated Windows answering from a metric a scenario declared.
//!
//! **The mask and not the glyphs is what a scenario is about.** A real rasteriser's answer depends on
//! which fonts the machine has, and the one a scenario used to run against was Windows' own Arial
//! standing in for the game's `font.ttf` — so what was being matched was never 紅魔郷's glyphs anyway.
//! What a scenario does ask is which string went into a texture and how big the quad round it came
//! out, and both of those a declared metric answers exactly.

use std::path::Path;

use crate::{Face, Mask};

/// A face the host has made, held for as long as anything bakes through it.
///
/// Owned rather than a handle the caller closes, the way [`crate::LogFile`] is: the host counts the
/// adds behind it, and an overlay makes one per size every time it is built — so a face let go of
/// without the add being taken back out is a font that stays loaded for the rest of the process.
pub struct Font(Face);

impl Font {
    /// `path` is the `.ttf` to load; `height` is the em height in pixels of the game's 640x480
    /// output.
    pub fn load(path: &Path, height: i32) -> Option<Self> {
        load_face(path, height).map(Self)
    }

    /// What the host actually selected, which is the font asked for only if it loaded.
    pub fn face_name(&self) -> Option<String> {
        face_name(self.0)
    }

    /// The coverage mask for `text`, or `None` for a string that measures to nothing.
    pub fn render(&self, text: &str) -> Option<Mask> {
        bake(self.0, text)
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        drop_face(self.0);
    }
}

fn load_face(path: &Path, height: i32) -> Option<Face> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.load_face(path, height);
    }
    host::load_face(path, height)
}

fn face_name(face: Face) -> Option<String> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.face_name(face);
    }
    host::face_name(face)
}

fn bake(face: Face, text: &str) -> Option<Mask> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.bake(face, text);
    }
    host::bake(face, text)
}

fn drop_face(face: Face) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.drop_face(face);
    }
    host::drop_face(face);
}

#[cfg(windows)]
use crate::real::text as host;

#[cfg(not(windows))]
mod host {
    use crate::{Face, Mask, no_windows};
    use std::path::Path;

    pub fn load_face(_path: &Path, _height: i32) -> Option<Face> {
        no_windows("text::load_face")
    }
    pub fn face_name(_face: Face) -> Option<String> {
        no_windows("text::face_name")
    }
    pub fn bake(_face: Face, _text: &str) -> Option<Mask> {
        no_windows("text::bake")
    }
    pub fn drop_face(_face: Face) {
        no_windows("text::drop_face")
    }
}
