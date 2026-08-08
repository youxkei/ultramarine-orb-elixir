//! Drawing on top of the game's frame.
//!
//! Coordinates are in the game's own 640x480 output, so the overlay lands in the
//! same place whatever the window is scaled to.
//!
//! Every draw is bracketed by a state block capture and apply. The game sets
//! render states once and assumes they stay set, so leaving so much as the
//! vertex shader changed shows up as the whole scene drawing wrong.

use std::path::Path;

use orb_api::d3d8::*;
use orb_api::text::Font;
use orb_api::{Device, Mask, Texture, Viewport};

use crate::log;

pub const SCREEN_WIDTH: f32 = 640.0;
pub const SCREEN_HEIGHT: f32 = 480.0;

pub const WHITE: u32 = 0xffff_ffff;

/// Em height in the game's 640x480 output.
pub const FONT_HEIGHT: i32 = 15;
/// And the size the one word on the mark over the lives is written at, which is bigger because it
/// is a word on a brush stroke rather than a line to be scanned.
pub const MARK_FONT_HEIGHT: i32 = 19;

#[repr(C)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    rhw: f32,
    color: u32,
    u: f32,
    v: f32,
}

const FVF: u32 = D3DFVF_XYZRHW | D3DFVF_DIFFUSE | D3DFVF_TEX1;

pub struct Overlay {
    device: Device,
    state_block: u32,
    font: Font,
    /// A second size, for the one thing here that is not a line of status text: the word on the
    /// mark over the lives, which is a word on a brush stroke and is read at a glance rather than
    /// scanned.
    mark_font: Font,
    /// Stands in for "no texture" so solid quads go through the same modulate
    /// path as text instead of needing their own texture stage setup.
    white: Texture,
}

impl Overlay {
    /// # Safety
    /// `device` must be the game's live `IDirect3DDevice8`, and this must run on
    /// the thread that owns it.
    pub unsafe fn new(
        device: Device,
        font_path: &Path,
        font_height: i32,
        mark_height: i32,
    ) -> Option<Self> {
        if device.is_null() {
            return None;
        }
        let font = loaded(font_path, font_height)?;
        let mark_font = loaded(font_path, mark_height)?;

        let (created, state_block) = create_state_block(device, D3DSBT_ALL);
        if created < 0 {
            log!("overlay: CreateStateBlock failed ({created:#x})");
            return None;
        }

        let Some(white) = made(device, 1, 1) else {
            // Deleted, not abandoned in the device: the block is the device's to hold and
            // there is no `Overlay` left to be dropped, so nothing would ever come back for
            // it. The two fonts above have a drop of their own and go with the `None`.
            delete_state_block(device, state_block);
            return None;
        };
        upload(
            white,
            &Mask {
                width: 1,
                height: 1,
                pixels: vec![WHITE],
            },
            1,
            1,
        );

        Some(Self {
            device,
            state_block,
            font,
            mark_font,
            white,
        })
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn mark_font(&self) -> &Font {
        &self.mark_font
    }

    /// # Safety
    /// Must be called between the game's `BeginScene` and `EndScene`, on the
    /// device's thread.
    pub unsafe fn frame(&self) -> Option<Frame<'_>> {
        // A device reset invalidates the block; recreating it is cheaper than
        // tracking device loss, and skipping one frame of overlay is invisible.
        if capture_state_block(self.device, self.state_block) < 0 {
            return None;
        }

        let viewport = get_viewport(self.device);
        let full = Viewport {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH as u32,
            height: SCREEN_HEIGHT as u32,
            min_z: 0.0,
            max_z: 1.0,
        };
        {
            set_viewport(self.device, full);
            set_vertex_shader(self.device, FVF);
            for (state, value) in [
                (D3DRS_ZENABLE, D3DZB_FALSE),
                (D3DRS_ZWRITEENABLE, 0),
                (D3DRS_ALPHATESTENABLE, 0),
                (D3DRS_ALPHABLENDENABLE, 1),
                (D3DRS_SRCBLEND, D3DBLEND_SRCALPHA),
                (D3DRS_DESTBLEND, D3DBLEND_INVSRCALPHA),
                (D3DRS_CULLMODE, D3DCULL_NONE),
                (D3DRS_LIGHTING, 0),
                (D3DRS_SPECULARENABLE, 0),
                (D3DRS_FOGENABLE, 0),
                (D3DRS_STENCILENABLE, 0),
                (D3DRS_CLIPPING, 1),
                (D3DRS_COLORVERTEX, 1),
            ] {
                set_render_state(self.device, state, value);
            }
            for (state, value) in [
                (D3DTSS_COLOROP, D3DTOP_MODULATE),
                (D3DTSS_COLORARG1, D3DTA_TEXTURE),
                (D3DTSS_COLORARG2, D3DTA_DIFFUSE),
                (D3DTSS_ALPHAOP, D3DTOP_MODULATE),
                (D3DTSS_ALPHAARG1, D3DTA_TEXTURE),
                (D3DTSS_ALPHAARG2, D3DTA_DIFFUSE),
                (D3DTSS_ADDRESSU, D3DTADDRESS_CLAMP),
                (D3DTSS_ADDRESSV, D3DTADDRESS_CLAMP),
                (D3DTSS_MAGFILTER, D3DTEXF_LINEAR),
                (D3DTSS_MINFILTER, D3DTEXF_LINEAR),
                (D3DTSS_MIPFILTER, D3DTEXF_NONE),
                (D3DTSS_TEXTURETRANSFORMFLAGS, 0),
            ] {
                set_texture_stage_state(self.device, 0, state, value);
            }
        }
        Some(Frame {
            overlay: self,
            viewport,
        })
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        delete_state_block(self.device, self.state_block);
        release_texture(self.white);
    }
}

/// Restores the game's render state when it goes out of scope.
pub struct Frame<'a> {
    overlay: &'a Overlay,
    viewport: Viewport,
}

impl Frame<'_> {
    pub fn fill(&self, x: f32, y: f32, width: f32, height: f32, color: u32) {
        self.quad(
            self.overlay.white,
            x,
            y,
            width,
            height,
            color,
            [0.0, 0.0, 1.0, 1.0],
        );
    }

    /// A piece of a texture over a rectangle, which is how the panel's own background is put
    /// back: the game's sheet, the tile of it that is the background, and where that tile goes.
    ///
    /// # Safety
    /// `texture` must be a live `IDirect3DTexture8` of the game's device.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn piece(
        &self,
        texture: Texture,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        uv: [f32; 4],
        color: u32,
    ) {
        self.quad(texture, x, y, width, height, color, uv);
    }

    /// A baked picture, drawn without the drop shadow a label gets: what this is for is a brush
    /// stroke, and a shadow under one would be a second stroke.
    pub fn picture(&self, picture: &Picture, x: f32, y: f32, color: u32) {
        let Some(texture) = picture.texture else {
            return;
        };
        self.quad(
            texture,
            x,
            y,
            picture.width(),
            picture.height(),
            color,
            [0.0, 0.0, picture.u, picture.v],
        );
    }

    /// Draws `label` with a one-pixel drop shadow, which is what makes text
    /// readable over a screen full of bullets.
    pub fn label(&self, label: &Label, x: f32, y: f32, color: u32) {
        let Some(texture) = label.texture else { return };
        let shadow = color & 0xff00_0000;
        for (offset, color) in [(1.0, shadow), (0.0, color)] {
            self.quad(
                texture,
                x + offset,
                y + offset,
                label.width(),
                label.height(),
                color,
                [0.0, 0.0, label.u, label.v],
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn quad(
        &self,
        texture: Texture,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
        uv: [f32; 4],
    ) {
        // Half-pixel shift so texel centres land on pixel centres.
        let (left, top) = (x - 0.5, y - 0.5);
        let (right, bottom) = (left + width, top + height);
        let [u0, v0, u, v] = uv;
        let vertices = [
            Vertex {
                x: left,
                y: top,
                z: 0.0,
                rhw: 1.0,
                color,
                u: u0,
                v: v0,
            },
            Vertex {
                x: right,
                y: top,
                z: 0.0,
                rhw: 1.0,
                color,
                u,
                v: v0,
            },
            Vertex {
                x: left,
                y: bottom,
                z: 0.0,
                rhw: 1.0,
                color,
                u: u0,
                v,
            },
            Vertex {
                x: right,
                y: bottom,
                z: 0.0,
                rhw: 1.0,
                color,
                u,
                v,
            },
        ];
        let device = self.overlay.device;
        set_texture(device, 0, Some(texture));
        draw_primitive_up(
            device,
            D3DPT_TRIANGLESTRIP,
            2,
            // The vertices as the slot takes them: a run of bytes and how long one is. What is in
            // them is this file's own layout, which is why the struct is declared here.
            unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    std::mem::size_of_val(&vertices),
                )
            },
            size_of::<Vertex>() as u32,
        );
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        let device = self.overlay.device;
        set_texture(device, 0, None);
        set_viewport(device, self.viewport);
        apply_state_block(device, self.overlay.state_block);
    }
}

/// One line of text, baked to a texture and re-baked only when it changes.
pub struct Label {
    texture: Option<Texture>,
    baked: String,
    size: (u32, u32),
    /// Where the used part of the texture ends, since the texture itself is
    /// rounded up to powers of two for old hardware's sake.
    u: f32,
    v: f32,
}

/// Beside `new` because the module is public now, and one with nothing baked or chosen yet is exactly
/// what `new` makes.
impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

impl Label {
    pub const fn new() -> Self {
        Self {
            texture: None,
            baked: String::new(),
            size: (0, 0),
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn width(&self) -> f32 {
        self.size.0 as f32
    }

    pub fn height(&self) -> f32 {
        self.size.1 as f32
    }

    /// # Safety
    /// Must run on the device's thread.
    pub unsafe fn set(&mut self, overlay: &Overlay, text: &str) {
        unsafe { self.set_in(overlay, overlay.font(), text) };
    }

    /// The same, in a font of the overlay's other than its usual one.
    ///
    /// # Safety
    /// Must run on the device's thread.
    pub unsafe fn set_in(&mut self, overlay: &Overlay, font: &Font, text: &str) {
        if self.texture.is_some() && self.baked == text {
            return;
        }
        self.release();
        self.baked = text.to_owned();
        self.size = (0, 0);

        let Some(mask) = font.render(text) else {
            return;
        };
        let (width, height) = (
            mask.width.next_power_of_two(),
            mask.height.next_power_of_two(),
        );
        let Some(texture) = made(overlay.device, width, height) else {
            return;
        };
        upload(texture, &mask, width, height);

        self.texture = Some(texture);
        self.size = (mask.width, mask.height);
        self.u = mask.width as f32 / width as f32;
        self.v = mask.height as f32 / height as f32;
    }

    fn release(&mut self) {
        if let Some(texture) = self.texture.take() {
            release_texture(texture);
        }
    }
}

impl Drop for Label {
    fn drop(&mut self) {
        self.release();
    }
}

/// A coverage map baked to a texture once, for something that did not come from a font: the
/// brush stroke over the lives. What `Label` is to a line of text, without the caching, since
/// what goes in here does not change.
pub struct Picture {
    texture: Option<Texture>,
    size: (u32, u32),
    u: f32,
    v: f32,
}

/// Beside `new` because the module is public now, and one with nothing baked or chosen yet is exactly
/// what `new` makes.
impl Default for Picture {
    fn default() -> Self {
        Self::new()
    }
}

impl Picture {
    pub const fn new() -> Self {
        Self {
            texture: None,
            size: (0, 0),
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn baked(&self) -> bool {
        self.texture.is_some()
    }

    pub fn width(&self) -> f32 {
        self.size.0 as f32
    }

    pub fn height(&self) -> f32 {
        self.size.1 as f32
    }

    /// # Safety
    /// Must run on the device's thread.
    pub unsafe fn bake(&mut self, overlay: &Overlay, mask: &Mask) {
        self.release();
        let (width, height) = (
            mask.width.next_power_of_two(),
            mask.height.next_power_of_two(),
        );
        let Some(texture) = made(overlay.device, width, height) else {
            return;
        };
        upload(texture, mask, width, height);
        self.texture = Some(texture);
        self.size = (mask.width, mask.height);
        self.u = mask.width as f32 / width as f32;
        self.v = mask.height as f32 / height as f32;
    }

    fn release(&mut self) {
        if let Some(texture) = self.texture.take() {
            release_texture(texture);
        }
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        self.release();
    }
}

/// A face of the font at that height, with what came of the asking said in the log.
///
/// **The saying is here and not behind the seam.** `orb-api` is under the log, so a failure comes back
/// from it as nothing at all — and what a substituted face means, which is that the glyphs on screen are
/// not the game's, is the drawing's business rather than a rasteriser's.
///
/// One line for both of the ways it can fail — a path that is not a font, and a host that would not make
/// a face of one that is — because from here they are the same answer, and the path is the half worth
/// printing.
fn loaded(path: &Path, height: i32) -> Option<Font> {
    let Some(font) = Font::load(path, height) else {
        log!("overlay: cannot load {}", path.display());
        return None;
    };
    log!(
        "overlay: font.ttf loaded, GDI is using {:?}",
        font.face_name().as_deref()
    );
    Some(font)
}

/// A texture of orb's own: one level, no usage flags, `A8R8G8B8` and managed, which is what every
/// texture the drawing makes is. The four the slot asks for that are always these are decided here
/// rather than behind the seam, that being where a decision belongs.
fn made(device: Device, width: u32, height: u32) -> Option<Texture> {
    let (created, texture) = create_texture(
        device,
        width,
        height,
        1,
        0,
        D3DFMT_A8R8G8B8,
        D3DPOOL_MANAGED,
    );
    if texture.is_none() {
        log!("overlay: CreateTexture {width}x{height} failed ({created:#x})");
    }
    texture
}

/// Copies `mask` into the top-left of the texture, leaving the padding that
/// rounding up to a power of two added fully transparent.
fn upload(texture: Texture, mask: &Mask, width: u32, height: u32) {
    let Some(locked) = lock_rect(texture, 0, 0) else {
        return;
    };
    for row in 0..height {
        let destination = locked.bits + row as usize * locked.pitch as usize;
        // The rows the lock handed over, one at a time. What the seam gives back is where they are
        // and how far apart they are, because *which* of them the mask fills and what the padding a
        // power-of-two texture added is left as is this function's decision and not the host's.
        let destination =
            unsafe { std::slice::from_raw_parts_mut(destination as *mut u32, width as usize) };
        destination.fill(0);
        if row < mask.height {
            let start = (row * mask.width) as usize;
            let source = &mask.pixels[start..start + mask.width as usize];
            destination[..source.len()].copy_from_slice(source);
        }
    }
    unlock_rect(texture, 0);
}

/// An overlay drawing onto a simulated Windows, for the tests of what a menu or a mark puts on the
/// screen.
///
/// Here rather than in each of the four test modules that draw, because what it holds together is the
/// order the three things go in and come apart in: the simulated Windows installed, an overlay built
/// through it, and the overlay dropped again while that host is still in front. An overlay outliving it
/// would release its state block and its texture through the real Windows, on handles no real device
/// ever handed out.
///
/// **What it replaces held three traps and this has none of them.** It stood an `Overlay` on a device
/// made of Rust functions and asked what the drawing had done by baking every string a second time
/// through the same font at the same size — so it needed one device per test, the overlay torn down
/// before that device, and a font every Windows has, which was Arial standing in for the game's own.
/// The seam answers all three: the record is the simulator's, the strings come back out of the masks it
/// baked, and there is no font.
#[cfg(test)]
pub(crate) struct Drawing {
    /// Declared first so it is dropped first, while the host below is still installed.
    overlay: Overlay,
    /// Held for as long as this is, which is what makes the drawing land in the simulated host rather
    /// than in the real one.
    _installed: orb_api::Installed,
    sim: std::sync::Arc<orb_sim::Sim>,
}

#[cfg(test)]
impl Drawing {
    /// The overlay orb builds, at the two sizes orb builds it at.
    ///
    /// # Panics
    /// If the overlay cannot be built, which here would mean the seam is not answering: there is no
    /// font to fail to load and no device to refuse a state block.
    pub(crate) fn new() -> Self {
        let sim = std::sync::Arc::new(orb_sim::Sim::new());
        let installed = sim.enter();
        let font = std::path::Path::new("game").join("font.ttf");
        sim.text().install_font(&font);
        let overlay =
            unsafe { Overlay::new(orb_sim::DEVICE, &font, FONT_HEIGHT, MARK_FONT_HEIGHT) }
                .expect("an overlay on a simulated Windows");
        // The textures the overlay uploaded building itself are not a frame.
        sim.drawing().forget();
        Self {
            overlay,
            _installed: installed,
            sim,
        }
    }

    /// Draws one frame and answers everything the device was asked for.
    pub(crate) fn drawn(&self, with: impl FnOnce(&Overlay)) -> orb_sim::Drawn {
        with(&self.overlay);
        let drawn = self.sim.drawing().drawn();
        self.sim.drawing().forget();
        drawn
    }

    /// The quads of one frame, which is what most of these ask about.
    pub(crate) fn frame(&self, with: impl FnOnce(&Overlay)) -> Vec<orb_sim::Quad> {
        self.drawn(with).quads
    }
}
