//! The device the game shows its frames through, and the textures orb draws with.
//!
//! **A mirror of the slots and not an abstraction over them.** Every function here takes what the
//! vtable slot behind it takes: `set_render_state`, `capture_state_block`, `draw_primitive_up` and the
//! fifteen beside them. What must not cross is a *decision* — which render states the drawing sets, how
//! the state block brackets a draw, the FVF and the vertex layout are all above this line, because they
//! are what a scenario over the drawing is about. A seam that said *draw this text here* would take all
//! of that into [`crate::real::d3d8`], where no scenario reaches, and the failure it exists to prevent
//! is the game's own scene drawing wrong. See
//! [docs/adr/0009](../../../docs/adr/0009-orb-injects-and-nothing-else-and-every-com-object-is-behind-the-seam.md).
//!
//! **Eighteen of them, which is every slot this file types.** Direct3D 8 has no metadata in Windows'
//! own, so the vtables are declared here — and only the methods that get called are typed, the rest
//! being pointer-sized padding. So the count comes out of the declarations themselves:
//!
//! ```sh
//! $ grep -c 'offset_of!' crates/orb-api/src/d3d8.rs
//! 18
//! ```
//!
//! Counted off the slot asserts rather than off the declarations: the layouts carry a padding field per
//! run of slots nothing calls, and a count of the fields counts those too.
//!
//! Fifteen of `IDirect3DDevice8` and three of `IDirect3DTexture8`. Every typed slot's index is asserted
//! at compile time against the one it has in `d3d8.h`: a wrong index would be a call into an unrelated
//! method with the wrong signature.
//!
//! **The vtable layouts stay here and stay `pub`.** `orb-core` names none of them — the only code that
//! calls through one is `real` — but two things below the seam do: the `Present` slot orb patches to put
//! the back buffer in a letterbox, and a game laid out by hand, which writes a vtable of its own for
//! that patch to have something to write into.

use std::ffi::c_void;
use std::mem::offset_of;

use crate::{Device, Hresult, Locked, Texture, Viewport};

pub const D3DRS_ZENABLE: u32 = 7;
pub const D3DRS_ZWRITEENABLE: u32 = 14;
pub const D3DRS_ALPHATESTENABLE: u32 = 15;
pub const D3DRS_SRCBLEND: u32 = 19;
pub const D3DRS_DESTBLEND: u32 = 20;
pub const D3DRS_CULLMODE: u32 = 22;
pub const D3DRS_ALPHABLENDENABLE: u32 = 27;
pub const D3DRS_FOGENABLE: u32 = 28;
pub const D3DRS_SPECULARENABLE: u32 = 29;
pub const D3DRS_STENCILENABLE: u32 = 52;
pub const D3DRS_CLIPPING: u32 = 136;
pub const D3DRS_LIGHTING: u32 = 137;
pub const D3DRS_COLORVERTEX: u32 = 141;

pub const D3DTSS_COLOROP: u32 = 1;
pub const D3DTSS_COLORARG1: u32 = 2;
pub const D3DTSS_COLORARG2: u32 = 3;
pub const D3DTSS_ALPHAOP: u32 = 4;
pub const D3DTSS_ALPHAARG1: u32 = 5;
pub const D3DTSS_ALPHAARG2: u32 = 6;
pub const D3DTSS_ADDRESSU: u32 = 13;
pub const D3DTSS_ADDRESSV: u32 = 14;
pub const D3DTSS_MAGFILTER: u32 = 16;
pub const D3DTSS_MINFILTER: u32 = 17;
pub const D3DTSS_MIPFILTER: u32 = 18;
pub const D3DTSS_TEXTURETRANSFORMFLAGS: u32 = 24;

pub const D3DTA_DIFFUSE: u32 = 0;
pub const D3DTA_TEXTURE: u32 = 2;
pub const D3DTOP_MODULATE: u32 = 4;
pub const D3DBLEND_SRCALPHA: u32 = 5;
pub const D3DBLEND_INVSRCALPHA: u32 = 6;
pub const D3DCULL_NONE: u32 = 1;
pub const D3DTEXF_LINEAR: u32 = 2;
pub const D3DTEXF_NONE: u32 = 0;
pub const D3DTADDRESS_CLAMP: u32 = 3;
pub const D3DZB_FALSE: u32 = 0;

pub const D3DPT_TRIANGLESTRIP: u32 = 5;
pub const D3DPOOL_MANAGED: u32 = 1;
pub const D3DFMT_A8R8G8B8: u32 = 21;
pub const D3DSBT_ALL: u32 = 1;
pub const D3DCLEAR_TARGET: u32 = 0x1;
pub const D3DCLEAR_ZBUFFER: u32 = 0x2;

pub const D3DFVF_XYZRHW: u32 = 0x004;
pub const D3DFVF_DIFFUSE: u32 = 0x040;
pub const D3DFVF_TEX1: u32 = 0x100;

// --- the fifteen of `IDirect3DDevice8` ---------------------------------------

/// `CreateTexture`, and the result as well as the texture: which failure it was is what orb writes into
/// the log.
///
/// Eight arguments, which is that slot's own list — see [`crate::Win::create_texture`] for why none of
/// them is decided here.
#[allow(clippy::too_many_arguments)]
pub fn create_texture(
    device: Device,
    width: u32,
    height: u32,
    levels: u32,
    usage: u32,
    format: u32,
    pool: u32,
) -> (Hresult, Option<Texture>) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.create_texture(device, width, height, levels, usage, format, pool);
    }
    host::create_texture(device, width, height, levels, usage, format, pool)
}

/// `CreateStateBlock`, and the token it made — zero where it would not.
pub fn create_state_block(device: Device, kind: u32) -> (Hresult, u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.create_state_block(device, kind);
    }
    host::create_state_block(device, kind)
}

/// `CaptureStateBlock`. The result comes back because a device reset invalidates the block and the
/// drawing gives that frame up rather than tracking device loss.
pub fn capture_state_block(device: Device, token: u32) -> Hresult {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.capture_state_block(device, token);
    }
    host::capture_state_block(device, token)
}

// The four below answer a result nobody above this line reads, so nothing comes back from them. Which
// is the mirror and not a simplification of it: what orb does with the slot is make the call and carry
// on, and a value returned for the sake of the shape would be one every call site has to discard.

/// `ApplyStateBlock`.
pub fn apply_state_block(device: Device, token: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.apply_state_block(device, token);
    }
    host::apply_state_block(device, token);
}

/// `DeleteStateBlock`.
pub fn delete_state_block(device: Device, token: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.delete_state_block(device, token);
    }
    host::delete_state_block(device, token);
}

/// `SetRenderState`.
pub fn set_render_state(device: Device, state: u32, value: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_render_state(device, state, value);
    }
    host::set_render_state(device, state, value);
}

/// `SetTextureStageState`.
pub fn set_texture_stage_state(device: Device, stage: u32, kind: u32, value: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_texture_stage_state(device, stage, kind, value);
    }
    host::set_texture_stage_state(device, stage, kind, value);
}

/// `SetTexture`. `None` unbinds the stage, which is the null the drawing leaves behind it.
pub fn set_texture(device: Device, stage: u32, texture: Option<Texture>) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_texture(device, stage, texture);
    }
    host::set_texture(device, stage, texture);
}

/// `SetVertexShader`, which for a fixed-function draw is the FVF.
pub fn set_vertex_shader(device: Device, shader: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_vertex_shader(device, shader);
    }
    host::set_vertex_shader(device, shader);
}

/// `SetViewport`.
pub fn set_viewport(device: Device, viewport: Viewport) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_viewport(device, viewport);
    }
    host::set_viewport(device, viewport);
}

/// `GetViewport`, and what the game had set — the drawing puts it back when its frame ends.
///
/// A viewport rather than a result and one filled in: the call answers a failure by leaving the
/// caller's own value alone, and the caller's own value was `Viewport::default()`, so that is what
/// comes back.
pub fn get_viewport(device: Device) -> Viewport {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.get_viewport(device);
    }
    host::get_viewport(device)
}

/// `DrawPrimitiveUP`. `vertices` is the whole buffer as bytes and `stride` how long one of them is,
/// which is what the slot takes: what is in them is the drawing's own layout and stays above this line.
pub fn draw_primitive_up(device: Device, kind: u32, count: u32, vertices: &[u8], stride: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.draw_primitive_up(device, kind, count, vertices, stride);
    }
    host::draw_primitive_up(device, kind, count, vertices, stride);
}

/// `BeginScene`.
pub fn begin_scene(device: Device) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.begin_scene(device);
    }
    host::begin_scene(device);
}

/// `EndScene`.
pub fn end_scene(device: Device) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.end_scene(device);
    }
    host::end_scene(device);
}

/// `Clear`, over the whole of the current viewport.
///
/// No rectangle count and no rectangles: every clear orb makes is `(0, NULL)`, which is the slot's own
/// spelling of *all of it*, and a list of rectangles is the one argument here that could not cross as
/// plain data.
pub fn clear(device: Device, flags: u32, color: u32, z: f32, stencil: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.clear(device, flags, color, z, stencil);
    }
    host::clear(device, flags, color, z, stencil);
}

// --- the three of `IDirect3DTexture8` ----------------------------------------

/// `LockRect` over the whole surface, and `None` where it failed or handed back nothing to write to.
pub fn lock_rect(texture: Texture, level: u32, flags: u32) -> Option<Locked> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.lock_rect(texture, level, flags);
    }
    host::lock_rect(texture, level, flags)
}

/// `UnlockRect`.
pub fn unlock_rect(texture: Texture, level: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.unlock_rect(texture, level);
    }
    host::unlock_rect(texture, level);
}

/// `Release`, which for a texture orb made is the last of its references.
pub fn release_texture(texture: Texture) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.release_texture(texture);
    }
    host::release_texture(texture);
}

// --- the layouts -------------------------------------------------------------

/// `IDirect3DDevice8`'s vtable, as far as the slots orb calls are concerned.
///
/// The object each slot is called on is a `*mut c_void`: what it really is is the game's device, and
/// nothing above the seam ever looks inside one.
#[repr(C)]
pub struct DeviceVtable {
    pub _iunknown: [usize; 3],
    pub _slot_3_to_19: [usize; 17],
    pub create_texture: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        *mut *mut c_void,
    ) -> Hresult,
    pub _slot_21_to_33: [usize; 13],
    pub begin_scene: unsafe extern "system" fn(*mut c_void) -> Hresult,
    pub end_scene: unsafe extern "system" fn(*mut c_void) -> Hresult,
    pub clear:
        unsafe extern "system" fn(*mut c_void, u32, *const c_void, u32, u32, f32, u32) -> Hresult,
    pub _slot_37_to_39: [usize; 3],
    pub set_viewport: unsafe extern "system" fn(*mut c_void, *const Viewport) -> Hresult,
    pub get_viewport: unsafe extern "system" fn(*mut c_void, *mut Viewport) -> Hresult,
    pub _slot_42_to_49: [usize; 8],
    pub set_render_state: unsafe extern "system" fn(*mut c_void, u32, u32) -> Hresult,
    pub _slot_51_to_53: [usize; 3],
    pub apply_state_block: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
    pub capture_state_block: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
    pub delete_state_block: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
    pub create_state_block: unsafe extern "system" fn(*mut c_void, u32, *mut u32) -> Hresult,
    pub _slot_58_to_60: [usize; 3],
    pub set_texture: unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> Hresult,
    pub _slot_62: [usize; 1],
    pub set_texture_stage_state: unsafe extern "system" fn(*mut c_void, u32, u32, u32) -> Hresult,
    pub _slot_64_to_71: [usize; 8],
    pub draw_primitive_up:
        unsafe extern "system" fn(*mut c_void, u32, u32, *const c_void, u32) -> Hresult,
    pub _slot_73_to_75: [usize; 3],
    pub set_vertex_shader: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
}

/// And `IDirect3DTexture8`'s.
#[repr(C)]
pub struct TextureVtable {
    pub _query_interface: usize,
    pub _add_ref: usize,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub _slot_3_to_15: [usize; 13],
    pub lock_rect:
        unsafe extern "system" fn(*mut c_void, u32, *mut RawLocked, *const c_void, u32) -> Hresult,
    pub unlock_rect: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
}

/// `D3DLOCKED_RECT`, which is what the slot fills in — [`Locked`] is the same two numbers with the
/// pointer as an address, so that nothing across the seam traffics in one.
#[repr(C)]
pub struct RawLocked {
    pub pitch: i32,
    pub bits: *mut c_void,
}

/// Which slot the `Present` orb patches is, so that the back buffer lands in a rectangle of the game's
/// aspect ratio rather than being stretched over the whole window.
///
/// Not typed above because orb never calls it — it replaces it, and calls what was there.
pub const PRESENT_SLOT: usize = 15;

const fn slot(index: usize) -> usize {
    index * size_of::<usize>()
}

const _: () = {
    assert!(offset_of!(DeviceVtable, create_texture) == slot(20));
    assert!(offset_of!(DeviceVtable, begin_scene) == slot(34));
    assert!(offset_of!(DeviceVtable, end_scene) == slot(35));
    assert!(offset_of!(DeviceVtable, clear) == slot(36));
    assert!(offset_of!(DeviceVtable, set_viewport) == slot(40));
    assert!(offset_of!(DeviceVtable, get_viewport) == slot(41));
    assert!(offset_of!(DeviceVtable, set_render_state) == slot(50));
    assert!(offset_of!(DeviceVtable, apply_state_block) == slot(54));
    assert!(offset_of!(DeviceVtable, capture_state_block) == slot(55));
    assert!(offset_of!(DeviceVtable, delete_state_block) == slot(56));
    assert!(offset_of!(DeviceVtable, create_state_block) == slot(57));
    assert!(offset_of!(DeviceVtable, set_texture) == slot(61));
    assert!(offset_of!(DeviceVtable, set_texture_stage_state) == slot(63));
    assert!(offset_of!(DeviceVtable, draw_primitive_up) == slot(72));
    assert!(offset_of!(DeviceVtable, set_vertex_shader) == slot(76));
    assert!(offset_of!(TextureVtable, release) == slot(2));
    assert!(offset_of!(TextureVtable, lock_rect) == slot(16));
    assert!(offset_of!(TextureVtable, unlock_rect) == slot(17));
};

#[cfg(windows)]
use crate::real::d3d8 as host;

#[cfg(not(windows))]
mod host {
    use crate::{Device, Hresult, Locked, Texture, Viewport, no_windows};

    pub fn create_texture(
        _device: Device,
        _width: u32,
        _height: u32,
        _levels: u32,
        _usage: u32,
        _format: u32,
        _pool: u32,
    ) -> (Hresult, Option<Texture>) {
        no_windows("d3d8::create_texture")
    }
    pub fn create_state_block(_device: Device, _kind: u32) -> (Hresult, u32) {
        no_windows("d3d8::create_state_block")
    }
    pub fn capture_state_block(_device: Device, _token: u32) -> Hresult {
        no_windows("d3d8::capture_state_block")
    }
    pub fn apply_state_block(_device: Device, _token: u32) {
        no_windows("d3d8::apply_state_block")
    }
    pub fn delete_state_block(_device: Device, _token: u32) {
        no_windows("d3d8::delete_state_block")
    }
    pub fn set_render_state(_device: Device, _state: u32, _value: u32) {
        no_windows("d3d8::set_render_state")
    }
    pub fn set_texture_stage_state(_device: Device, _stage: u32, _kind: u32, _value: u32) {
        no_windows("d3d8::set_texture_stage_state")
    }
    pub fn set_texture(_device: Device, _stage: u32, _texture: Option<Texture>) {
        no_windows("d3d8::set_texture")
    }
    pub fn set_vertex_shader(_device: Device, _shader: u32) {
        no_windows("d3d8::set_vertex_shader")
    }
    pub fn set_viewport(_device: Device, _viewport: Viewport) {
        no_windows("d3d8::set_viewport")
    }
    pub fn get_viewport(_device: Device) -> Viewport {
        no_windows("d3d8::get_viewport")
    }
    pub fn draw_primitive_up(
        _device: Device,
        _kind: u32,
        _count: u32,
        _vertices: &[u8],
        _stride: u32,
    ) {
        no_windows("d3d8::draw_primitive_up")
    }
    pub fn begin_scene(_device: Device) {
        no_windows("d3d8::begin_scene")
    }
    pub fn end_scene(_device: Device) {
        no_windows("d3d8::end_scene")
    }
    pub fn clear(_device: Device, _flags: u32, _color: u32, _z: f32, _stencil: u32) {
        no_windows("d3d8::clear")
    }
    pub fn lock_rect(_texture: Texture, _level: u32, _flags: u32) -> Option<Locked> {
        no_windows("d3d8::lock_rect")
    }
    pub fn unlock_rect(_texture: Texture, _level: u32) {
        no_windows("d3d8::unlock_rect")
    }
    pub fn release_texture(_texture: Texture) {
        no_windows("d3d8::release_texture")
    }
}
