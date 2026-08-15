//! A device that keeps what it was asked to draw instead of drawing it.
//!
//! What is kept is the request — the quads with their rectangles and colours, in the order they were
//! drawn, and which texture each went through. Enough to say that the retry menu put its ways on the
//! screen with the cursor on one of them, or that the mark over the lives covers the row and nothing
//! beside it.
//!
//! **And which string went into each texture**, which is what makes [`Recording::says`] possible. A
//! device is handed a bitmap and never a string, so what a texture holds has to be read back out of the
//! bitmap — and the bitmap is [`Glyphs`](crate::Glyphs)'s own, whose pixels carry which string they were
//! baked from. That is the second half of the glyph seam: nothing bakes a comparison here, and no two
//! strings are alike by accident.
//!
//! **Per-simulated-Windows and not a static.** It was a static once, and had to be: the recording device
//! was a vtable of Rust functions, and a vtable's functions are plain `extern "system"` with nowhere to
//! carry a context. Behind the seam the device is a handle and the answer is a `&self`, so the record
//! goes where the rest of a simulated host's state is.

use std::collections::HashMap;
use std::sync::Mutex;

use orb_api::{Device, Locked, Texture, Viewport};

use crate::Glyphs;

/// What the game shows through, as an e2e test's game writes it into its own memory.
///
/// Any address: what it is for is that orb reads it out of the game's memory and hands it back to the
/// seam, so the only thing it has to be is a number that comes out as it went in. A laid-out game maps a
/// vtable at it all the same — see `orb-e2e`'s `finds_its_device` — because the `Present` slot orb
/// patches has to have something to be patched in.
pub const DEVICE: Device = Device(0x0d3d_0000);

/// One quad, as the vertices handed to `DrawPrimitiveUP` describe it.
///
/// The half-pixel shift the drawing applies is undone, so these are the coordinates the caller asked for
/// rather than the ones Direct3D wants.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: u32,
    /// Which texture was bound when it was drawn. Solid fills go through the overlay's one white
    /// texel, so this tells a filled rectangle from a glyph or a picture without holding either.
    pub texture: usize,
}

impl Quad {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Whether this quad covers the whole of `other`, which is how "the mark covers the row" is asked.
    pub fn covers(&self, other: &Quad) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.right() >= other.right()
            && self.bottom() >= other.bottom()
    }

    /// Whether the two overlap at all, for asking that something is *not* drawn over its neighbour.
    pub fn overlaps(&self, other: &Quad) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// What the device was asked to do, in order.
#[derive(Default)]
pub struct Drawn {
    pub quads: Vec<Quad>,
    /// Rectangles the device was asked to clear, with the colour, for the wash a chapter boundary puts
    /// over the play field.
    pub clears: Vec<u32>,
    pub scenes: u32,
    /// Viewports set, so that a frame drawn to the whole output rather than the play field is visible as
    /// such.
    pub viewports: Vec<Viewport>,
}

impl Drawn {
    /// The quads that went through a texture other than the white texel, which are the ones with a
    /// picture or a glyph in them.
    pub fn pictured(&self) -> Vec<Quad> {
        let white = self.quads.first().map(|quad| quad.texture);
        self.quads
            .iter()
            .copied()
            .filter(|quad| Some(quad.texture) != white)
            .collect()
    }
}

/// The vertex the drawing writes. Declared here rather than shared, because what this has to agree with
/// is the bytes on the wire: a change to the drawing's own struct that this did not follow is a change
/// this should stop decoding.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    rhw: f32,
    color: u32,
    u: f32,
    v: f32,
}

/// A texture the device has handed out: what it is, and what was uploaded into it.
///
/// Boxed, and never taken out of [`Recording::textures`] while the recording lives. A lock hands out the
/// address of these rows and the drawing writes through it, so the storage must not move — and a
/// release does not free it, both because the drawing may release twice and because what an e2e test
/// asks afterwards is what went in.
struct Held {
    width: u32,
    pixels: Vec<u32>,
}

/// The device an e2e test's game shows through, and everything asked of it.
///
/// **Whatever keeps this installed has to be a field of it** rather than a second value handed back
/// beside it: a guard returned alongside drops *first*, so the installation goes away while the textures
/// this recording still holds are being released, and the next test finds them cleared under it.
pub struct Recording {
    drawn: Mutex<Drawn>,
    /// The textures handed out, by the handle each was given. The handle is an address of its own
    /// making — nothing dereferences it, the rows being reached through the lock instead.
    textures: Mutex<HashMap<usize, Box<Held>>>,
    /// The texture bound to stage 0, which the next quad is drawn with.
    bound: Mutex<usize>,
    /// The next handle to hand out. Well clear of [`DEVICE`], so that a device and a texture can never
    /// be mistaken for one another in a record an e2e test reads.
    next: Mutex<usize>,
}

/// Where the texture handles start. An address nothing dereferences, so any number will do — this one is
/// legible in a failure and cannot be confused with [`DEVICE`].
const FIRST_TEXTURE: usize = 0x7e00_0000;

impl Default for Recording {
    fn default() -> Self {
        Self::new()
    }
}

impl Recording {
    pub(crate) fn new() -> Self {
        Self {
            drawn: Mutex::new(Drawn::default()),
            textures: Mutex::new(HashMap::new()),
            bound: Mutex::new(0),
            next: Mutex::new(FIRST_TEXTURE),
        }
    }

    /// What has been asked of it since the last [`forget`](Self::forget).
    pub fn drawn(&self) -> Drawn {
        let drawn = self.drawn.lock().unwrap();
        Drawn {
            quads: drawn.quads.clone(),
            clears: drawn.clears.clone(),
            scenes: drawn.scenes,
            viewports: drawn.viewports.clone(),
        }
    }

    /// Forgets what has been drawn, so that what an e2e test asserts about is one frame rather than the
    /// textures the drawing uploaded while it was being built.
    pub fn forget(&self) {
        *self.drawn.lock().unwrap() = Drawn::default();
    }

    /// What was uploaded into a texture the device handed out, or `None` for one it did not.
    pub fn pixels_of(&self, texture: usize) -> Option<Vec<u32>> {
        self.textures
            .lock()
            .unwrap()
            .get(&texture)
            .map(|held| held.pixels.clone())
    }

    /// The quads that drew `text`, wherever on the screen it was drawn.
    ///
    /// **Asked of the bake rather than baked again.** A device is handed a bitmap and never a string, so
    /// what a quad says has to be read out of the bitmap it was drawn with — and the bitmaps here are
    /// [`Glyphs`]'s own, whose pixels carry which string they came from. Nothing is rasterised twice,
    /// nothing is compared by pixel equality, and a texture that holds no string at all — the drawing's
    /// white texel, the brush stroke over the lives, the game's own sheet — answers with nothing.
    ///
    /// The colour comes back with each quad, which is what says whether an item was the one under a
    /// menu's cursor: `menu_ui` draws that one in `SELECTED` and the rest in `NORMAL`.
    ///
    /// **The drop shadow under each label is left out**, or every line on the screen would come back
    /// twice: `Overlay::label` draws the same texture at one pixel down and across in the colour's own
    /// alpha over black, and then the text itself. What that leaves out is a label drawn in black, which
    /// nothing draws — the four colours in `menu_ui` and the word on the mark over the lives are all lit.
    pub fn says(&self, glyphs: &Glyphs, text: &str) -> Vec<Quad> {
        self.drawn()
            .quads
            .into_iter()
            .filter(|quad| quad.color & 0x00ff_ffff != 0)
            .filter(|quad| self.holds(glyphs, quad.texture).as_deref() == Some(text))
            .collect()
    }

    /// Which string went into a texture, and `None` for one that holds no string.
    fn holds(&self, glyphs: &Glyphs, texture: usize) -> Option<String> {
        let textures = self.textures.lock().unwrap();
        let first = *textures.get(&texture)?.pixels.first()?;
        glyphs.said(first)
    }

    pub(crate) fn create_texture(&self, width: u32, height: u32) -> Texture {
        let height = height.max(1);
        let mut next = self.next.lock().unwrap();
        let handle = *next;
        *next += 1;
        self.textures.lock().unwrap().insert(
            handle,
            Box::new(Held {
                width,
                pixels: vec![0; (width * height) as usize],
            }),
        );
        Texture(handle)
    }

    pub(crate) fn lock_rect(&self, texture: Texture) -> Option<Locked> {
        let mut textures = self.textures.lock().unwrap();
        let held = textures.get_mut(&texture.0)?;
        Some(Locked {
            pitch: (held.width * size_of::<u32>() as u32) as i32,
            bits: held.pixels.as_mut_ptr() as usize,
        })
    }

    pub(crate) fn set_texture(&self, texture: Option<Texture>) {
        *self.bound.lock().unwrap() = texture.map_or(0, |texture| texture.0);
    }

    pub(crate) fn scene_began(&self) {
        self.drawn.lock().unwrap().scenes += 1;
    }

    pub(crate) fn cleared(&self, color: u32) {
        self.drawn.lock().unwrap().clears.push(color);
    }

    pub(crate) fn viewport_set(&self, viewport: Viewport) {
        self.drawn.lock().unwrap().viewports.push(viewport);
    }

    /// The game's own, which the drawing puts back when its frame ends.
    pub(crate) fn viewport(&self) -> Viewport {
        Viewport {
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            min_z: 0.0,
            max_z: 1.0,
        }
    }

    pub(crate) fn drew(&self, count: u32, vertices: &[u8], stride: u32) {
        if stride as usize != size_of::<Vertex>() || count != 2 {
            // Not the two-triangle strip the drawing draws. Said rather than decoded as one, because a
            // silent misread here would be an e2e test asserting about quads that were never drawn.
            return;
        }
        if vertices.len() < 4 * size_of::<Vertex>() {
            return;
        }
        let corners = unsafe { std::slice::from_raw_parts(vertices.as_ptr() as *const Vertex, 4) };
        let texture = *self.bound.lock().unwrap();
        // The drawing shifts by half a pixel so texel centres land on pixel centres; undone here so the
        // record is what the caller asked for.
        let quad = Quad {
            x: corners[0].x + 0.5,
            y: corners[0].y + 0.5,
            width: corners[3].x - corners[0].x,
            height: corners[3].y - corners[0].y,
            color: corners[0].color,
            texture,
        };
        self.drawn.lock().unwrap().quads.push(quad);
    }
}

#[cfg(test)]
mod tests {
    use super::{DEVICE, Quad};
    use crate::Sim;
    use orb_api::d3d8::{D3DFMT_A8R8G8B8, D3DPOOL_MANAGED, D3DPT_TRIANGLESTRIP, D3DSBT_ALL};
    use orb_api::{Texture, Win};

    /// The vertex the drawing writes, written again here as the drawing writes it: four corners of a
    /// strip, the colour in each, and the half-pixel shift.
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Vertex {
        x: f32,
        y: f32,
        z: f32,
        rhw: f32,
        color: u32,
        u: f32,
        v: f32,
    }

    fn strip(x: f32, y: f32, width: f32, height: f32, color: u32) -> [Vertex; 4] {
        let (left, top) = (x - 0.5, y - 0.5);
        let (right, bottom) = (left + width, top + height);
        let corner = |x: f32, y: f32| Vertex {
            x,
            y,
            z: 0.0,
            rhw: 1.0,
            color,
            u: 0.0,
            v: 0.0,
        };
        [
            corner(left, top),
            corner(right, top),
            corner(left, bottom),
            corner(right, bottom),
        ]
    }

    fn bytes(vertices: &[Vertex; 4]) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                std::mem::size_of_val(vertices),
            )
        }
    }

    fn drew(sim: &Sim, vertices: &[Vertex; 4]) {
        sim.draw_primitive_up(
            DEVICE,
            D3DPT_TRIANGLESTRIP,
            2,
            bytes(vertices),
            size_of::<Vertex>() as u32,
        );
    }

    fn made(sim: &Sim, width: u32, height: u32) -> Texture {
        let (result, texture) = sim.create_texture(
            DEVICE,
            width,
            height,
            1,
            0,
            D3DFMT_A8R8G8B8,
            D3DPOOL_MANAGED,
        );
        assert_eq!(result, 0);
        texture.expect("a texture a simulated device always makes")
    }

    /// What the recording is for: a quad drawn through the device comes back as the rectangle and the
    /// colour that were asked for, with the half-pixel shift undone.
    #[test]
    fn a_quad_drawn_comes_back_as_the_rectangle_that_was_asked_for() {
        let sim = Sim::new();
        drew(&sim, &strip(496.0, 122.0, 144.0, 16.0, 0xff00_ff00));

        let drawn = sim.drawing().drawn();
        assert_eq!(drawn.quads.len(), 1);
        let quad = drawn.quads[0];
        assert_eq!((quad.x, quad.y), (496.0, 122.0));
        assert_eq!((quad.width, quad.height), (144.0, 16.0));
        assert_eq!(quad.color, 0xff00_ff00);
    }

    /// Which texture a quad went through is kept, because that is what tells a solid fill from a glyph:
    /// the drawing binds its one white texel for the first and a real texture for the second.
    #[test]
    fn a_quad_remembers_which_texture_it_was_drawn_with() {
        let sim = Sim::new();
        let white = made(&sim, 1, 1);
        let picture = made(&sim, 64, 32);
        assert_ne!(white, picture);

        let vertices = strip(0.0, 0.0, 8.0, 8.0, 0xffff_ffff);
        for texture in [white, picture] {
            sim.set_texture(DEVICE, 0, Some(texture));
            drew(&sim, &vertices);
        }

        let drawn = sim.drawing().drawn();
        assert_eq!(drawn.quads.len(), 2);
        assert_eq!(drawn.quads[0].texture, white.0);
        // The white one is drawn first, which is how `pictured` tells the rest from it.
        assert_eq!(drawn.pictured().len(), 1);
        assert_eq!(drawn.pictured()[0].texture, picture.0);
    }

    /// A texture the drawing locks has somewhere to write, and the pitch it is told is the pitch it
    /// gets: an upload that walked off the end of a row would be writing over the row below.
    #[test]
    fn a_locked_texture_hands_out_a_row_of_its_own_width() {
        let sim = Sim::new();
        let texture = made(&sim, 64, 32);
        let locked = sim.lock_rect(texture, 0, 0).expect("a texture it made");
        assert_eq!(locked.pitch, 64 * 4);
        assert_ne!(locked.bits, 0);
        sim.unlock_rect(texture, 0);
        assert!(
            sim.lock_rect(Texture(locked.bits), 0, 0).is_none(),
            "a handle no simulated device handed out was locked",
        );
    }

    /// The state block the drawing takes out and puts back, and the scene it draws inside. Neither does
    /// anything here; what a test needs of them is that they answer, since the drawing gives up on a
    /// device whose `CreateStateBlock` fails.
    #[test]
    fn a_state_block_and_a_scene_are_answered() {
        let sim = Sim::new();
        let (result, token) = sim.create_state_block(DEVICE, D3DSBT_ALL);
        assert_eq!(result, 0);
        assert_ne!(token, 0, "a block the drawing can apply and delete");
        assert_eq!(sim.capture_state_block(DEVICE, token), 0);
        sim.begin_scene(DEVICE);
        sim.end_scene(DEVICE);
        assert_eq!(sim.drawing().drawn().scenes, 1);
    }

    /// A quad covers the row it is meant to and misses the rows either side, which is the shape of every
    /// assertion the mark over the lives wants to make.
    #[test]
    fn a_quad_says_what_it_covers_and_what_it_misses() {
        let row = Quad {
            x: 496.0,
            y: 122.0,
            width: 144.0,
            height: 16.0,
            color: 0,
            texture: 0,
        };
        let over = Quad {
            x: 496.0,
            y: 106.0,
            width: 144.0,
            height: 40.0,
            ..row
        };
        let bombs = Quad {
            x: 496.0,
            y: 146.0,
            width: 144.0,
            height: 16.0,
            ..row
        };
        assert!(over.covers(&row));
        assert!(!row.covers(&over));
        assert!(over.overlaps(&row));
        assert!(!over.overlaps(&bombs), "the row below is left alone");
    }
}
