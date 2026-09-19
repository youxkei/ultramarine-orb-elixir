//! Drawing on top of the game's frame.
//!
//! Coordinates are in the game's own 640x480 output, so the overlay lands in the
//! same place whatever the window is scaled to.
//!
//! **Through either of two things**, which is what [`Backend`] is: the game's own Direct3D 8 device, or a
//! canvas over the swapchain a game that has no Direct3D 8 presents its frames through — see
//! [`crate::canvas`]. Everything above `Frame` is written once for both, because what the whole overlay
//! asks of a device is one alpha-blended textured quad and both can draw that.
//!
//! On the device, every draw is bracketed by a state block capture and apply. The game sets
//! render states once and assumes they stay set, so leaving so much as the
//! vertex shader changed shows up as the whole scene drawing wrong.

use std::path::Path;

use orb_api::d3d8::*;
use orb_api::text::Font;
use orb_api::{Device, Mask, Texture, Viewport};

use crate::canvas::{Canvas, Sheet};
use crate::game::Face;
use crate::log;

pub const SCREEN_WIDTH: f32 = 640.0;
pub const SCREEN_HEIGHT: f32 = 480.0;

pub const WHITE: u32 = 0xffff_ffff;

/// The size a line of orb's own is drawn at, in the game's 640x480 output.
///
/// **A size and not an em height any more**, which is what `Game::own_face` changed: a game that names the face
/// it draws its own text in also says what height it asks the host for, and th07 asks for a *cell* of twice
/// this less two and shows the result at half. So the glyphs come out around four fifths of this number, and
/// this is the number to turn when they read too small — which a th07 session asked for at 15, where the lines
/// were legible and not comfortable.
///
/// **And what stops it going higher is the longest line orb draws over a game's own screen**: the mark on the
/// shot select, whose lower line is a chapter — `STAGE 4 MIDBOSS NONSPELL 1 RETRY 14`, thirty-five characters of
/// a fixed-pitch face, so half this number apiece. From x=24 that reaches x=339 here and x=374 at 20, and the
/// game's own writing about the shot under the cursor begins around x=368. So 20 was tried and put back.
pub const FONT_HEIGHT: i32 = 18;
/// And the size the one word on the mark over the lives is written at, which is bigger because it
/// is a word on a brush stroke rather than a line to be scanned.
pub const MARK_FONT_HEIGHT: i32 = 22;

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

/// **What an overlay draws through.** The two are not alternatives orb chooses between: a game has one of
/// them and not the other, and which it is, is what `Game::d3d_device` answers against a swapchain being
/// there — see `runtime::draw_overlay`.
enum Backend {
    /// The game's own `IDirect3DDevice8`, and the state block that puts its render states back.
    TheDevice { device: Device, state_block: u32 },
    /// A canvas over the game's own swapchain, for a game that presents its frames through Direct3D 11 and
    /// has no Direct3D 8 device at all — see [`crate::canvas`], which is where every decision about that
    /// drawing is.
    TheCanvas(Canvas),
}

/// **A picture uploaded to whatever the overlay draws through**, which is the one thing that has a value
/// per backend rather than a call: a `Label` keeps one between frames, so the two cannot be told apart by
/// where the code is.
enum Uploaded {
    /// A texture of the device, rounded up to powers of two for old hardware's sake, with how much of it
    /// the picture fills.
    OnTheDevice { texture: Texture, u: f32, v: f32 },
    /// A sheet of the canvas, which needs no rounding: Direct3D 11 takes a texture of any size, so the
    /// picture fills the whole of it.
    OnTheCanvas(Sheet),
}

impl Uploaded {
    /// The piece of it the picture went into, as `Frame::quad` wants it.
    fn uv(&self) -> [f32; 4] {
        match self {
            Self::OnTheDevice { u, v, .. } => [0.0, 0.0, *u, *v],
            Self::OnTheCanvas(_) => [0.0, 0.0, 1.0, 1.0],
        }
    }

    /// Given back to whichever made it, which is why this is one value and not a texture beside a flag.
    ///
    /// # Safety
    /// Must run on the thread that owns the backend it was made on.
    unsafe fn release(self) {
        match self {
            Self::OnTheDevice { texture, .. } => release_texture(texture),
            Self::OnTheCanvas(sheet) => unsafe { sheet.release() },
        }
    }
}

impl Backend {
    /// **A mask uploaded, as whichever of the two this is takes it.**
    ///
    /// # Safety
    /// Must run on the thread that owns this backend.
    unsafe fn upload(&self, mask: &Mask) -> Option<Uploaded> {
        match self {
            Self::TheDevice { device, .. } => {
                let (width, height) = (
                    mask.width.next_power_of_two(),
                    mask.height.next_power_of_two(),
                );
                let texture = made(*device, width, height)?;
                upload(texture, mask, width, height);
                Some(Uploaded::OnTheDevice {
                    texture,
                    u: mask.width as f32 / width as f32,
                    v: mask.height as f32 / height as f32,
                })
            }
            Self::TheCanvas(canvas) => unsafe {
                canvas
                    .sheet(mask.width, mask.height, &mask.pixels)
                    .map(Uploaded::OnTheCanvas)
            },
        }
    }
}

pub struct Overlay {
    backend: Backend,
    font: Font,
    /// A second size, for the one thing here that is not a line of status text: the word on the
    /// mark over the lives, which is a word on a brush stroke and is read at a glance rather than
    /// scanned.
    mark_font: Font,
    /// Stands in for "no texture" so solid quads go through the same modulate
    /// path as text instead of needing their own texture stage setup.
    ///
    /// An `Option` so that the drop can take it out and release it: there is no `Overlay` without one, so
    /// nothing but the drop ever finds it empty.
    white: Option<Uploaded>,
}

impl Overlay {
    /// # Safety
    /// `device` must be the game's live `IDirect3DDevice8`, and this must run on
    /// the thread that owns it.
    pub unsafe fn new(
        device: Device,
        font_path: &Path,
        own_face: Option<Face>,
        font_height: i32,
        mark_height: i32,
    ) -> Option<Self> {
        if device.is_null() {
            return None;
        }
        let font = loaded(font_path, own_face, font_height)?;
        let mark_font = loaded(font_path, own_face, mark_height)?;

        let (created, state_block) = create_state_block(device, D3DSBT_ALL);
        if created < 0 {
            log!("overlay: CreateStateBlock failed ({created:#x})");
            return None;
        }

        let backend = Backend::TheDevice {
            device,
            state_block,
        };
        let Some(white) = (unsafe { backend.upload(&one_white_pixel()) }) else {
            // Deleted, not abandoned in the device: the block is the device's to hold and
            // there is no `Overlay` left to be dropped, so nothing would ever come back for
            // it. The two fonts above have a drop of their own and go with the `None`.
            delete_state_block(device, state_block);
            return None;
        };

        Some(Self {
            backend,
            font,
            mark_font,
            white: Some(white),
        })
    }

    /// **The same overlay over a game's own swapchain**, for one that draws through Direct3D 11 and has no
    /// Direct3D 8 device for [`Overlay::new`] to take.
    ///
    /// The output is this module's own 640x480 and not a size read off the swapchain: those are the
    /// coordinates everything orb draws is placed in, and the canvas's viewport maps them onto whatever
    /// the back buffer measures.
    ///
    /// # Safety
    /// `swapchain` must be a live `IDXGISwapChain` of a Direct3D 11 device, and this must run on the
    /// thread that made it.
    pub unsafe fn over_a_swapchain(
        swapchain: usize,
        font_path: &Path,
        own_face: Option<Face>,
        font_height: i32,
        mark_height: i32,
    ) -> Option<Self> {
        if swapchain == 0 {
            return None;
        }
        let font = loaded(font_path, own_face, font_height)?;
        let mark_font = loaded(font_path, own_face, mark_height)?;
        let canvas =
            unsafe { Canvas::new(swapchain, (SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)) }?;
        let backend = Backend::TheCanvas(canvas);
        let white = unsafe { backend.upload(&one_white_pixel()) }?;
        Some(Self {
            backend,
            font,
            mark_font,
            white: Some(white),
        })
    }

    /// **Whether this one is drawn from the present rather than from the draw hook**, which is what a
    /// canvas over a swapchain is: see `runtime::the_overlay_over_a_swapchain`, and th06c's own
    /// `end_drawing`, which composites the frame and hands it over in one call — so the only moment a
    /// finished frame of such a game can be drawn over is inside its present.
    pub fn draws_over_a_swapchain(&self) -> bool {
        matches!(self.backend, Backend::TheCanvas(_))
    }

    /// **The canvas's view of the frame's buffer given back**, once the frame is drawn over — see
    /// `canvas::Canvas::lets_go_of_this_frames_buffer`. Nothing for an overlay drawn with a device.
    ///
    /// # Safety
    /// Must run on the device's thread, after the frame's drawing.
    pub unsafe fn lets_go_of_this_frames_buffer(&self) {
        if let Backend::TheCanvas(canvas) = &self.backend {
            unsafe { canvas.lets_go_of_this_frames_buffer() };
        }
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
        let Backend::TheDevice {
            device,
            state_block,
        } = self.backend
        else {
            // **Nothing to take out here, the canvas doing it per quad instead.** Direct3D 11 has no
            // state block, so there is no one call to bracket a frame with: `canvas::Saved` reads the
            // game's own state and writes it back around each draw — see there, and the black window
            // that came of a canvas which set state and put none of it back.
            //
            // **What a frame does need is this frame's own buffer**, a swapchain that flips presenting
            // from a different one each time — see `canvas::Canvas::takes_this_frames_buffer`. Here and
            // not per quad because one view serves every quad of a frame, and this is the one place that
            // runs once a frame.
            if let Backend::TheCanvas(canvas) = &self.backend {
                unsafe { canvas.takes_this_frames_buffer() };
            }
            return Some(Frame {
                overlay: self,
                viewport: None,
            });
        };
        // A device reset invalidates the block; recreating it is cheaper than
        // tracking device loss, and skipping one frame of overlay is invisible.
        if capture_state_block(device, state_block) < 0 {
            return None;
        }

        let viewport = get_viewport(device);
        let full = Viewport {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH as u32,
            height: SCREEN_HEIGHT as u32,
            min_z: 0.0,
            max_z: 1.0,
        };
        {
            set_viewport(device, full);
            set_vertex_shader(device, FVF);
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
                set_render_state(device, state, value);
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
                set_texture_stage_state(device, 0, state, value);
            }
        }
        Some(Frame {
            overlay: self,
            viewport: Some(viewport),
        })
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        if let Backend::TheDevice {
            device,
            state_block,
        } = self.backend
        {
            delete_state_block(device, state_block);
        }
        if let Some(white) = self.white.take() {
            unsafe { white.release() };
        }
    }
}

/// The picture a solid fill is drawn with, which is what puts every quad through the same modulate path.
fn one_white_pixel() -> Mask {
    Mask {
        width: 1,
        height: 1,
        pixels: vec![WHITE],
    }
}

/// Restores the game's render state when it goes out of scope.
pub struct Frame<'a> {
    overlay: &'a Overlay,
    /// The viewport to put back, and `None` on a canvas, which took none out — see [`Overlay::frame`].
    viewport: Option<Viewport>,
}

impl Frame<'_> {
    pub fn fill(&self, x: f32, y: f32, width: f32, height: f32, color: u32) {
        let Some(white) = &self.overlay.white else {
            return;
        };
        self.quad(white, x, y, width, height, color, [0.0, 0.0, 1.0, 1.0]);
    }

    /// A piece of a texture over a rectangle, which is how the panel's own background is put
    /// back: the game's sheet, the tile of it that is the background, and where that tile goes.
    ///
    /// **The one thing here that is the device's alone**, and nothing at all on a canvas: what it draws
    /// with is a texture of the game's own Direct3D 8 device, which a game that has no such device hands
    /// over none of — `Game::panel_tile` answers `None` there, so nothing reaches this.
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
        if !matches!(self.overlay.backend, Backend::TheDevice { .. }) {
            return;
        }
        self.quad(
            &Uploaded::OnTheDevice {
                texture,
                u: 0.0,
                v: 0.0,
            },
            x,
            y,
            width,
            height,
            color,
            uv,
        );
    }

    /// A baked picture, drawn without the drop shadow a label gets: what this is for is a brush
    /// stroke, and a shadow under one would be a second stroke.
    pub fn picture(&self, picture: &Picture, x: f32, y: f32, color: u32) {
        let Some(uploaded) = &picture.uploaded else {
            return;
        };
        self.quad(
            uploaded,
            x,
            y,
            picture.width(),
            picture.height(),
            color,
            uploaded.uv(),
        );
    }

    /// The same picture drawn into a box of its own rather than at its own size, for a mark whose box
    /// the game's panel bounds: th07's rows are as far apart as they are tall, so the stroke over its
    /// count of lives has that row and nothing either side of it — see `Game::room_around_the_lives_row`.
    pub fn picture_in(
        &self,
        picture: &Picture,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
    ) {
        let Some(uploaded) = &picture.uploaded else {
            return;
        };
        self.quad(uploaded, x, y, width, height, color, uploaded.uv());
    }

    /// Draws `label` with a one-pixel drop shadow, which is what makes text
    /// readable over a screen full of bullets.
    pub fn label(&self, label: &Label, x: f32, y: f32, color: u32) {
        self.label_scaled(label, x, y, 1.0, color);
    }

    /// And the same drawn `scale` of its own size, for a word that has to fit a box the game's own panel
    /// bounds rather than the size its face was baked at — see [`Frame::picture_in`].
    pub fn label_scaled(&self, label: &Label, x: f32, y: f32, scale: f32, color: u32) {
        let Some(uploaded) = &label.uploaded else {
            return;
        };
        let shadow = color & 0xff00_0000;
        for (offset, color) in [(1.0, shadow), (0.0, color)] {
            self.quad(
                uploaded,
                x + offset,
                y + offset,
                label.width() * scale,
                label.height() * scale,
                color,
                uploaded.uv(),
            );
        }
    }

    /// **The one draw the whole overlay is made of**, sent to whichever backend the overlay holds.
    #[allow(clippy::too_many_arguments)]
    fn quad(
        &self,
        uploaded: &Uploaded,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
        uv: [f32; 4],
    ) {
        let (device, texture) = match (&self.overlay.backend, uploaded) {
            (Backend::TheCanvas(canvas), Uploaded::OnTheCanvas(sheet)) => {
                // Safety: a `Frame` is only made inside the game's own frame on the thread that owns the
                // backend, and the sheet came out of this canvas — see `Overlay::frame`.
                unsafe { canvas.quad(sheet, x, y, width, height, color, uv) };
                return;
            }
            (Backend::TheDevice { device, .. }, Uploaded::OnTheDevice { texture, .. }) => {
                (*device, *texture)
            }
            // A picture of the other backend, which is a `Label` drawn against an overlay that did not bake
            // it. Nothing arranges it — there is one overlay in a launch, rebuilt with everything it holds
            // — and drawing it would hand one API's handle to the other.
            _ => return,
        };
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
        let (
            Backend::TheDevice {
                device,
                state_block,
            },
            Some(viewport),
        ) = (&self.overlay.backend, self.viewport)
        else {
            return;
        };
        set_texture(*device, 0, None);
        set_viewport(*device, viewport);
        apply_state_block(*device, *state_block);
    }
}

/// One line of text, baked to a texture and re-baked only when it changes.
///
/// **The picture it keeps belongs to the backend it was baked on**, and nothing here says which backend
/// that was. So the same `Label` drawn against a *second* device hands over the first one's texture,
/// which is a use-after-free rather than a wrong picture. A run never arranges it — there is one device,
/// and the overlay is rebuilt with the new one when it is lost — so only a test can, and a test that
/// draws should make one device of its own and draw every frame of that test on it. Across the *two*
/// backends `Frame::quad` does refuse it, that being the one mismatch it can see.
pub struct Label {
    uploaded: Option<Uploaded>,
    baked: String,
    size: (u32, u32),
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
            uploaded: None,
            baked: String::new(),
            size: (0, 0),
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
        if self.uploaded.is_some() && self.baked == text {
            return;
        }
        self.release();
        self.baked = text.to_owned();
        self.size = (0, 0);

        let Some(mask) = font.render(text) else {
            return;
        };
        let Some(uploaded) = (unsafe { overlay.backend.upload(&mask) }) else {
            return;
        };

        self.uploaded = Some(uploaded);
        // **The quad is the size the string is drawn at and the mask can be bigger**, which is what a face baked
        // at a multiple of its size means: the texture holds that many times the pixels and the quad round it is
        // that many times smaller, so the device's own filter does the scaling down — see `Font::baked_at` and
        // `Game::own_face`. A quad left at the mask's size would be text at twice the size instead.
        let baked_at = font.baked_at();
        self.size = (mask.width / baked_at, mask.height / baked_at);
    }

    fn release(&mut self) {
        if let Some(uploaded) = self.uploaded.take() {
            unsafe { uploaded.release() };
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
    uploaded: Option<Uploaded>,
    size: (u32, u32),
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
            uploaded: None,
            size: (0, 0),
        }
    }

    pub fn baked(&self) -> bool {
        self.uploaded.is_some()
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
        let Some(uploaded) = (unsafe { overlay.backend.upload(mask) }) else {
            return;
        };
        self.uploaded = Some(uploaded);
        self.size = (mask.width, mask.height);
    }

    fn release(&mut self) {
        if let Some(uploaded) = self.uploaded.take() {
            unsafe { uploaded.release() };
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
fn loaded(path: &Path, own_face: Option<Face>, height: i32) -> Option<Font> {
    let Some(font) = Font::load(path, height) else {
        // The machine's own face rather than nothing, which is the whole difference between orb having
        // screens in th07 and having none.
        // The line says which of the two this launch is, because what the glyphs on screen *are* is the one
        // thing a substituted face changes.
        log!("overlay: no font beside the game; using the machine's own face");
        // Asked for by the family the game itself asks the host for, where it names one: what orb's screens
        // are then in is the face the game's own text is in, rather than whatever the charset maps to. See
        // `Game::own_face`.
        let face = own_face.unwrap_or(Face::AT_ITS_OWN_SIZE);
        // At the height the game itself asks for, out of the size orb draws at, and baked at whatever that
        // comes out as: the family alone is the same face and not the same glyphs — see `Game::own_face`.
        let font =
            Font::of_the_machine((face.height)(height), face.family, face.bold, face.baked_at);
        match &font {
            // Named because a family the machine does not have is a substituted face, and then the glyphs
            // are neither orb's choice nor the game's.
            Some(font) => log!(
                "overlay: the machine's own face is {:?}, asked for as {:?}",
                font.face_name().as_deref(),
                face.family,
            ),
            None => log!(
                "overlay: cannot load {} and the machine has no face",
                path.display()
            ),
        }
        return font;
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
        // With a font beside the game, so no face of the machine's is asked for and the family a game
        // would name is not part of what this harness is for.
        let overlay =
            unsafe { Overlay::new(orb_sim::DEVICE, &font, None, FONT_HEIGHT, MARK_FONT_HEIGHT) }
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
