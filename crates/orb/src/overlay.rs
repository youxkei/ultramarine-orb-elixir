//! Drawing on top of the game's frame.
//!
//! Coordinates are in the game's own 640x480 output, so the overlay lands in the
//! same place whatever the window is scaled to.
//!
//! Every draw is bracketed by a state block capture and apply. The game sets
//! render states once and assumes they stay set, so leaving so much as the
//! vertex shader changed shows up as the whole scene drawing wrong.

use std::ffi::c_void;
use std::path::Path;

use crate::d3d8::*;
use crate::log::log;
use crate::text::{Font, Mask};

pub const SCREEN_WIDTH: f32 = 640.0;
pub const SCREEN_HEIGHT: f32 = 480.0;

pub const WHITE: u32 = 0xffff_ffff;

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
    device: *mut Device,
    state_block: u32,
    font: Font,
    /// Stands in for "no texture" so solid quads go through the same modulate
    /// path as text instead of needing their own texture stage setup.
    white: *mut Texture,
}

impl Overlay {
    /// # Safety
    /// `device` must be the game's live `IDirect3DDevice8`, and this must run on
    /// the thread that owns it.
    pub unsafe fn new(device: *mut Device, font_path: &Path, font_height: i32) -> Option<Self> {
        if device.is_null() {
            return None;
        }
        let font = Font::load(font_path, font_height)?;

        let mut state_block = 0;
        let created = unsafe {
            ((*(*device).vtable).create_state_block)(device, D3DSBT_ALL, &mut state_block)
        };
        if created < 0 {
            log!("overlay: CreateStateBlock failed ({created:#x})");
            return None;
        }

        let white = unsafe { create_texture(device, 1, 1) }?;
        unsafe {
            upload(
                white,
                &Mask {
                    width: 1,
                    height: 1,
                    pixels: vec![WHITE],
                },
                1,
                1,
            )
        };

        Some(Self {
            device,
            state_block,
            font,
            white,
        })
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    /// # Safety
    /// Must be called between the game's `BeginScene` and `EndScene`, on the
    /// device's thread.
    pub unsafe fn frame(&self) -> Option<Frame<'_>> {
        let vtable = unsafe { &*(*self.device).vtable };
        // A device reset invalidates the block; recreating it is cheaper than
        // tracking device loss, and skipping one frame of overlay is invisible.
        if unsafe { (vtable.capture_state_block)(self.device, self.state_block) } < 0 {
            return None;
        }

        let mut viewport = Viewport::default();
        unsafe { (vtable.get_viewport)(self.device, &mut viewport) };
        let full = Viewport {
            x: 0,
            y: 0,
            width: SCREEN_WIDTH as u32,
            height: SCREEN_HEIGHT as u32,
            min_z: 0.0,
            max_z: 1.0,
        };
        unsafe {
            (vtable.set_viewport)(self.device, &full);
            (vtable.set_vertex_shader)(self.device, FVF);
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
                (vtable.set_render_state)(self.device, state, value);
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
                (vtable.set_texture_stage_state)(self.device, 0, state, value);
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
        unsafe {
            ((*(*self.device).vtable).delete_state_block)(self.device, self.state_block);
            ((*(*self.white).vtable).release)(self.white);
        }
    }
}

/// Restores the game's render state when it goes out of scope.
pub struct Frame<'a> {
    overlay: &'a Overlay,
    viewport: Viewport,
}

impl Frame<'_> {
    pub fn fill(&self, x: f32, y: f32, width: f32, height: f32, color: u32) {
        unsafe { self.quad(self.overlay.white, x, y, width, height, color, 1.0, 1.0) };
    }

    /// Draws `label` with a one-pixel drop shadow, which is what makes text
    /// readable over a screen full of bullets.
    pub fn label(&self, label: &Label, x: f32, y: f32, color: u32) {
        let Some(texture) = label.texture else { return };
        let shadow = color & 0xff00_0000;
        for (offset, color) in [(1.0, shadow), (0.0, color)] {
            unsafe {
                self.quad(
                    texture,
                    x + offset,
                    y + offset,
                    label.width(),
                    label.height(),
                    color,
                    label.u,
                    label.v,
                )
            };
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn quad(
        &self,
        texture: *mut Texture,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
        u: f32,
        v: f32,
    ) {
        // Half-pixel shift so texel centres land on pixel centres.
        let (left, top) = (x - 0.5, y - 0.5);
        let (right, bottom) = (left + width, top + height);
        let vertices = [
            Vertex {
                x: left,
                y: top,
                z: 0.0,
                rhw: 1.0,
                color,
                u: 0.0,
                v: 0.0,
            },
            Vertex {
                x: right,
                y: top,
                z: 0.0,
                rhw: 1.0,
                color,
                u,
                v: 0.0,
            },
            Vertex {
                x: left,
                y: bottom,
                z: 0.0,
                rhw: 1.0,
                color,
                u: 0.0,
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
        let vtable = unsafe { &*(*device).vtable };
        unsafe {
            (vtable.set_texture)(device, 0, texture);
            (vtable.draw_primitive_up)(
                device,
                D3DPT_TRIANGLESTRIP,
                2,
                vertices.as_ptr() as *const c_void,
                size_of::<Vertex>() as u32,
            );
        }
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        let device = self.overlay.device;
        unsafe {
            let vtable = &*(*device).vtable;
            (vtable.set_texture)(device, 0, std::ptr::null_mut());
            (vtable.set_viewport)(device, &self.viewport);
            (vtable.apply_state_block)(device, self.overlay.state_block);
        }
    }
}

/// One line of text, baked to a texture and re-baked only when it changes.
pub struct Label {
    texture: Option<*mut Texture>,
    baked: String,
    size: (u32, u32),
    /// Where the used part of the texture ends, since the texture itself is
    /// rounded up to powers of two for old hardware's sake.
    u: f32,
    v: f32,
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
        if self.texture.is_some() && self.baked == text {
            return;
        }
        self.release();
        self.baked = text.to_owned();
        self.size = (0, 0);

        let Some(mask) = overlay.font().render(text) else {
            return;
        };
        let (width, height) = (
            mask.width.next_power_of_two(),
            mask.height.next_power_of_two(),
        );
        let texture = unsafe { create_texture(overlay.device, width, height) };
        let Some(texture) = texture else { return };
        unsafe { upload(texture, &mask, width, height) };

        self.texture = Some(texture);
        self.size = (mask.width, mask.height);
        self.u = mask.width as f32 / width as f32;
        self.v = mask.height as f32 / height as f32;
    }

    fn release(&mut self) {
        if let Some(texture) = self.texture.take() {
            unsafe { ((*(*texture).vtable).release)(texture) };
        }
    }
}

impl Drop for Label {
    fn drop(&mut self) {
        self.release();
    }
}

unsafe fn create_texture(device: *mut Device, width: u32, height: u32) -> Option<*mut Texture> {
    let mut texture = std::ptr::null_mut();
    let created = unsafe {
        ((*(*device).vtable).create_texture)(
            device,
            width,
            height,
            1,
            0,
            D3DFMT_A8R8G8B8,
            D3DPOOL_MANAGED,
            &mut texture,
        )
    };
    if created < 0 || texture.is_null() {
        log!("overlay: CreateTexture {width}x{height} failed ({created:#x})");
        return None;
    }
    Some(texture)
}

/// Copies `mask` into the top-left of the texture, leaving the padding that
/// rounding up to a power of two added fully transparent.
unsafe fn upload(texture: *mut Texture, mask: &Mask, width: u32, height: u32) {
    let mut locked = LockedRect {
        pitch: 0,
        bits: std::ptr::null_mut(),
    };
    let result =
        unsafe { ((*(*texture).vtable).lock_rect)(texture, 0, &mut locked, std::ptr::null(), 0) };
    if result < 0 || locked.bits.is_null() {
        return;
    }
    for row in 0..height {
        let destination = unsafe { locked.bits.byte_add(row as usize * locked.pitch as usize) };
        let destination =
            unsafe { std::slice::from_raw_parts_mut(destination as *mut u32, width as usize) };
        destination.fill(0);
        if row < mask.height {
            let start = (row * mask.width) as usize;
            let source = &mask.pixels[start..start + mask.width as usize];
            destination[..source.len()].copy_from_slice(source);
        }
    }
    unsafe { ((*(*texture).vtable).unlock_rect)(texture, 0) };
}
