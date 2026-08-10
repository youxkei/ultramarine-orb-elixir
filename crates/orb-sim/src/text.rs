//! The glyphs, which a simulated Windows has none of.
//!
//! **A declared metric rather than a rasteriser, and the reason is that the real one was already not
//! the game's.** An e2e test's directory used to hold Windows' own Arial under the name 紅魔郷 installs
//! its font as, because `AddFontResourceExW` takes a path and a path that is not a font is not
//! something `Font::load` survives — so the pixels an e2e test matched against were Arial's. Keeping
//! them real bought a fidelity that was never there, at the price of an answer that varied with which
//! fonts the machine happened to have.
//!
//! What an e2e test really asks of a baked string is two things, and both of them are here: **which
//! string went into a texture**, and **how big the quad round it came out**. The first is the whole of
//! what `says` is for; the second is what the drawing centres and lays out against. So the mask
//! carries the one and [`Metric`] declares the other — the same shape everything else in this crate
//! answers with, `Panel::scaled`'s two sizes and `Display::agreed`'s hertz.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use orb_api::{Face, Mask};

/// How wide and how tall a string comes out at, as an e2e test declares it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Metric {
    /// How many pixels wide one character is.
    pub advance: u32,
    /// How many pixels tall a line of them is.
    pub line: u32,
}

impl Metric {
    /// What an em height comes out as where an e2e test has said nothing: half the em wide per
    /// character, and two pixels over the em tall.
    ///
    /// About what a proportional face measures at that size — GDI's own answer for Arial at
    /// `lfHeight: -15` is 58 pixels for the seven of `DISABLE` against this one's 49 — and the
    /// numbers themselves are what nothing in the suite has an opinion about. What every assertion
    /// over a baked string needs of them is only that they are not nothing: the three that read the
    /// geometry all say the same thing, which is that the word on the mark over the lives overlaps
    /// the row the game counts them in, and `lives_ui::word_at` centres the word on that row — so a
    /// word of any size at all overlaps it and a word of none would not be drawn.
    pub fn for_em(height: i32) -> Self {
        let em = height.unsigned_abs();
        Self {
            advance: (em / 2).max(1),
            line: em + 2,
        }
    }
}

/// The fonts an e2e test says are there, the faces made of them, and every string baked through one.
///
/// **What this cannot hold is the rasterising**, and nothing else does either: a bake here answers from
/// the metric an e2e test declared, so an e2e test can ask where the drawing put a string and tell one
/// string from another, and nothing tells a right metric from a wrong one. What a string actually comes
/// out as is something only a launch says, in `overlay: font.ttf loaded, GDI is using …`.
pub struct Glyphs {
    /// The font files an e2e test says are beside the game. A path that is not one of these is one
    /// [`Glyphs::load_face`] refuses — which is 妖々夢, whose fonts are inside `th07.dat` and whose
    /// directory holds no `font.ttf` at all, and is the launch orb says `overlay: unavailable` for.
    installed: Mutex<Vec<PathBuf>>,
    /// The em height each face was made at, the index being its [`Face`] — which is the whole of what
    /// a face is where there are no glyphs, the metric being read off that height.
    ///
    /// Never emptied: a face given back stays here so that a mask baked through it is still readable,
    /// and one launch makes four of them.
    faces: Mutex<Vec<i32>>,
    /// What a string comes out as at an em height, where an e2e test has said other than
    /// [`Metric::for_em`].
    declared: Mutex<HashMap<i32, Metric>>,
    /// Every string baked, in the order it was first asked for. The index is what the mask's own
    /// pixels carry, so that a texture it was uploaded into can be read back as the string that went
    /// into it — see [`Glyphs::said`].
    baked: Mutex<Vec<String>>,
}

/// What a mask's pixels are: full coverage, with which string it is in the colour.
///
/// A real mask is `0x00ffffff` throughout with the glyph's shape in the alpha channel, and the colour
/// is applied by the vertex colour at draw time — so its own RGB carries nothing and is free to carry
/// this instead. Which is the one property a simulated bake has to have: two bakes of one string
/// identical, and two strings never alike, because what an e2e test asks of a texture is which string
/// went into it.
fn carrying(baked: usize) -> u32 {
    0xff00_0000 | (baked as u32 & 0x00ff_ffff)
}

impl Glyphs {
    pub(crate) fn new() -> Self {
        Self {
            installed: Mutex::new(Vec::new()),
            faces: Mutex::new(Vec::new()),
            declared: Mutex::new(HashMap::new()),
            baked: Mutex::new(Vec::new()),
        }
    }

    /// Says a font file is there, which is what makes an overlay over it possible.
    ///
    /// Declared rather than read off the disk, so that what an e2e test asserts about the overlay does
    /// not depend on a file a directory happens to hold — and so that the launch with no font beside
    /// its exe is an e2e test saying nothing rather than an e2e test deleting something.
    pub fn install_font(&self, path: impl Into<PathBuf>) {
        self.installed.lock().unwrap().push(path.into());
    }

    /// And says it is not there any more, leaving the faces already made of it readable.
    ///
    /// Which is a launch of 妖々夢: the harness's own device bakes what a game draws through the font,
    /// and orb then reaches for one and finds none — the game keeping its fonts inside `th07.dat`. A
    /// face outliving the file is the real thing too, `AddFontResourceExW` having already been made.
    pub fn remove_font(&self, path: &Path) {
        self.installed
            .lock()
            .unwrap()
            .retain(|installed| installed != path);
    }

    /// Says what a string comes out as at an em height, for an e2e test with an opinion about the size
    /// of a baked one.
    pub fn measures(&self, height: i32, metric: Metric) {
        self.declared.lock().unwrap().insert(height, metric);
    }

    /// The metric a string at this em height is baked to.
    pub fn metric(&self, height: i32) -> Metric {
        self.declared
            .lock()
            .unwrap()
            .get(&height)
            .copied()
            .unwrap_or_else(|| Metric::for_em(height))
    }

    /// Which string a mask's pixels carry, and `None` for pixels that were not baked here — the
    /// overlay's own white texel, or the brush stroke, which is a picture and not a string.
    ///
    /// What the drawing seam asks to say which string a texture holds.
    pub fn said(&self, pixel: u32) -> Option<String> {
        if pixel >> 24 != 0xff {
            return None;
        }
        let baked = self.baked.lock().unwrap();
        baked.get((pixel & 0x00ff_ffff) as usize).cloned()
    }

    /// Every string baked so far, in the order it was first asked for.
    pub fn asked_for(&self) -> Vec<String> {
        self.baked.lock().unwrap().clone()
    }

    pub(crate) fn load_face(&self, path: &Path, height: i32) -> Option<Face> {
        if !self
            .installed
            .lock()
            .unwrap()
            .iter()
            .any(|installed| installed == path)
        {
            return None;
        }
        let mut faces = self.faces.lock().unwrap();
        faces.push(height);
        Some(Face(faces.len() - 1))
    }

    /// What was selected, which here is nothing: a metric at an em height, which is what a face is
    /// when there are no glyphs.
    ///
    /// The height and not the path, though the path is what a face was made from. orb writes this
    /// answer into its log, and an e2e test's font sits in a directory named after the process — so a
    /// path here would be a log line that reads differently every run, which is exactly the kind of
    /// answer a simulated host is for not having.
    pub(crate) fn face_name(&self, face: Face) -> Option<String> {
        let height = *self.faces.lock().unwrap().get(face.0)?;
        Some(format!("a metric declared at {height}"))
    }

    pub(crate) fn bake(&self, face: Face, text: &str) -> Option<Mask> {
        if text.is_empty() {
            return None;
        }
        let height = *self.faces.lock().unwrap().get(face.0)?;
        let metric = self.metric(height);
        let width = metric.advance * text.chars().count() as u32;
        let pixel = {
            let mut baked = self.baked.lock().unwrap();
            let at = baked.iter().position(|held| held == text);
            carrying(at.unwrap_or_else(|| {
                baked.push(text.to_owned());
                baked.len() - 1
            }))
        };
        Some(Mask {
            width,
            height: metric.line,
            pixels: vec![pixel; (width * metric.line) as usize],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Glyphs, Metric};
    use std::path::Path;

    fn over(path: &Path, height: i32) -> (Glyphs, orb_api::Face) {
        let glyphs = Glyphs::new();
        glyphs.install_font(path);
        let face = glyphs.load_face(path, height).expect("a font declared");
        (glyphs, face)
    }

    /// A path no e2e test declared a font at is a path there is no face of, which is the launch with
    /// no `font.ttf` beside its exe.
    #[test]
    fn a_font_no_e2e_test_declared_cannot_be_loaded() {
        let glyphs = Glyphs::new();
        assert!(glyphs.load_face(Path::new("game/font.ttf"), 15).is_none());
        glyphs.install_font("game/font.ttf");
        assert!(glyphs.load_face(Path::new("game/font.ttf"), 15).is_some());
        assert!(
            glyphs.load_face(Path::new("game/other.ttf"), 15).is_none(),
            "a face was made of a file nothing said was there",
        );
    }

    /// The two things a baked string has to be: the same twice, and not the same as another string.
    #[test]
    fn a_string_bakes_the_same_twice_and_two_strings_never_alike() {
        let (glyphs, face) = over(Path::new("game/font.ttf"), 15);
        let of = |text: &str| {
            glyphs
                .bake(face, text)
                .expect("a string with characters in it")
        };
        assert_eq!(of("DISABLE").pixels, of("DISABLE").pixels);
        assert_ne!(of("DISABLE").pixels, of("DISABLED").pixels);
        assert!(glyphs.bake(face, "").is_none(), "an empty string measured");
    }

    /// And which string a mask carries comes back out of it, which is how a texture is read as the
    /// string that went into it.
    #[test]
    fn a_mask_says_which_string_it_was_baked_from() {
        let (glyphs, face) = over(Path::new("game/font.ttf"), 15);
        let mask = glyphs.bake(face, "やめる").expect("a string");
        assert_eq!(glyphs.said(mask.pixels[0]).as_deref(), Some("やめる"));
        assert_eq!(
            glyphs.said(0x00ff_ffff),
            None,
            "a pixel with no coverage read as a string",
        );
        assert_eq!(
            glyphs.said(carrying_past_the_end()),
            None,
            "a string nothing baked was named",
        );
    }

    fn carrying_past_the_end() -> u32 {
        super::carrying(usize::from(u16::MAX))
    }

    /// The size a string comes out at is the metric's, and an e2e test may say what that is.
    #[test]
    fn a_strings_size_is_the_declared_metric() {
        let (glyphs, face) = over(Path::new("game/font.ttf"), 15);
        let mask = glyphs.bake(face, "DISABLE").expect("a string");
        let usual = Metric::for_em(15);
        assert_eq!((mask.width, mask.height), (usual.advance * 7, usual.line));
        assert_eq!(mask.pixels.len(), (mask.width * mask.height) as usize);

        glyphs.measures(
            15,
            Metric {
                advance: 10,
                line: 30,
            },
        );
        let wider = glyphs.bake(face, "DISABLE").expect("a string");
        assert_eq!((wider.width, wider.height), (70, 30));
    }
}
