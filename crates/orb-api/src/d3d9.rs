//! The Direct3D 9 device a game of that generation shows its frames through — th10's.
//!
//! **A mirror of the slots and not an abstraction over them**, for the reason [`crate::d3d8`] gives:
//! every function here takes what its vtable slot takes, and what the frame and the overlay do with them is
//! above the seam.
//!
//! **Every slot this file types, and nothing besides**, counted the way `d3d8` counts its own:
//!
//! ```sh
//! $ grep -c 'offset_of!' crates/orb-api/src/d3d9.rs
//! ```
//!
//! What orb's own frame loop makes of a frame it runs in the game's place — the scene either side of the
//! draw chain, the texture unbound after it and the viewport before the update — what the overlay draws
//! with: a texture made and written, a state block to put the game's states back with, the states it sets,
//! and one quad at a time — and what a picture of the frame is taken with: the back buffer, a surface in
//! system memory, the copy into it and a lock of it. Every typed slot's index is asserted at compile time against the one it has
//! in `d3d9.h`, and those th10's own code calls through against the offset it calls it at.
//!
//! **One device to a simulated Windows whichever version it is.** What a laid-out game's device records is
//! a scene begun and ended, a texture bound, a viewport set and a quad drawn, and none of that differs
//! between the two interfaces; the vtable index is the one thing that does, and that is the real host's. So
//! each call here reaches the same [`crate::Win`] method its `d3d8` namesake does — a sampler state the one
//! the texture stage's states go to, the two having been one in Direct3D 8, and `SetFVF` the one
//! `SetVertexShader` goes to, which is what a fixed-function vertex shader of Direct3D 8 is.

use std::ffi::c_void;
use std::mem::offset_of;

use crate::d3d8::RawLocked;
use crate::{Device, Hresult, Locked, Surface, Texture, Viewport};

/// **`IDirect3D9::CreateDevice`'s slot**, the seventeenth of `IDirect3D9`'s vtable in `d3d9.h` — the one th10 makes
/// its device through, `call [ecx+0x40]` (0x439a06).
pub const CREATE_DEVICE_SLOT: usize = 16;

/// **`D3DCREATE_NOWINDOWCHANGES`**, the behaviour flag that says Direct3D must not alter the focus window in any
/// way.
pub const D3DCREATE_NOWINDOWCHANGES: u32 = 0x800;
/// `D3DPRESENT_PARAMETERS::PresentationInterval`'s offset: the fourteenth of its fourteen `u32`-wide fields.
pub const PRESENTATION_INTERVAL: usize = 0x34;
/// `D3DPRESENT_INTERVAL_IMMEDIATE`, a present that does not wait for the display's blank.
pub const D3DPRESENT_INTERVAL_IMMEDIATE: u32 = 0x8000_0000;

/// `D3DPOOL_SYSTEMMEM`, the pool a surface the back buffer is copied into has to be in.
pub const D3DPOOL_SYSTEMMEM: u32 = 2;

/// `D3DSURFACE_DESC` as Direct3D 9 has it, field for field — not Direct3D 8's, which has a size where this has
/// a multisample quality, so the two are not one struct.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct SurfaceDesc9 {
    pub format: u32,
    pub kind: u32,
    pub usage: u32,
    pub pool: u32,
    pub multi_sample_type: u32,
    pub multi_sample_quality: u32,
    pub width: u32,
    pub height: u32,
}

/// `D3DSAMP_ADDRESSU` and its neighbours: the sampler states Direct3D 9 took out of the texture stage's.
pub const D3DSAMP_ADDRESSU: u32 = 1;
pub const D3DSAMP_ADDRESSV: u32 = 2;
pub const D3DSAMP_MAGFILTER: u32 = 5;
pub const D3DSAMP_MINFILTER: u32 = 6;
pub const D3DSAMP_MIPFILTER: u32 = 7;

/// `D3DSBT_ALL`, the same number as Direct3D 8's.
pub const D3DSBT_ALL: u32 = 1;

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

/// `SetTexture`. `None` unbinds the stage, which is the null the game's own frame leaves behind its draw.
pub fn set_texture(device: Device, stage: u32, texture: Option<Texture>) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_texture(device, stage, texture);
    }
    host::set_texture(device, stage, texture);
}

/// `SetViewport`. `D3DVIEWPORT9` is `D3DVIEWPORT8` field for field, so it is the same [`Viewport`].
pub fn set_viewport(device: Device, viewport: Viewport) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_viewport(device, viewport);
    }
    host::set_viewport(device, viewport);
}

/// `GetViewport`, and what the game had set.
pub fn get_viewport(device: Device) -> Viewport {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.get_viewport(device);
    }
    host::get_viewport(device)
}

/// `CreateTexture`, with the shared handle Direct3D 9 added left null — and the result as well as the
/// texture, which failure it was being what orb writes into the log.
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

/// `LockRect` over the whole of a level of a texture, and `None` where it failed.
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

/// `Release` of a texture.
pub fn release_texture(texture: Texture) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.release_texture(texture);
    }
    host::release_texture(texture);
}

/// `CreateStateBlock`, and the block it made as a token — the `IDirect3DStateBlock9` itself, an address
/// that fits the token on the one architecture a game of this device runs on — nought where it would not.
pub fn create_state_block(device: Device, kind: u32) -> (Hresult, u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.create_state_block(device, kind);
    }
    host::create_state_block(device, kind)
}

/// `IDirect3DStateBlock9::Capture`, whose result says whether the block is still the device's.
pub fn capture_state_block(device: Device, token: u32) -> Hresult {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.capture_state_block(device, token);
    }
    host::capture_state_block(device, token)
}

/// `IDirect3DStateBlock9::Apply`.
pub fn apply_state_block(device: Device, token: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.apply_state_block(device, token);
    }
    host::apply_state_block(device, token);
}

/// `IDirect3DStateBlock9::Release`, which is what deleting one is in Direct3D 9.
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

/// `SetSamplerState` — see the module's own doc for where it goes in a simulated Windows.
pub fn set_sampler_state(device: Device, sampler: u32, kind: u32, value: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_texture_stage_state(device, sampler, kind, value);
    }
    host::set_sampler_state(device, sampler, kind, value);
}

/// `SetFVF` — see the module's own doc for where it goes in a simulated Windows.
pub fn set_fvf(device: Device, fvf: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.set_vertex_shader(device, fvf);
    }
    host::set_fvf(device, fvf);
}

/// `DrawPrimitiveUP`, the vertices handed over as bytes the way `d3d8`'s are.
pub fn draw_primitive_up(device: Device, kind: u32, count: u32, vertices: &[u8], stride: u32) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.draw_primitive_up(device, kind, count, vertices, stride);
    }
    host::draw_primitive_up(device, kind, count, vertices, stride);
}

/// `GetBackBuffer` of a swap chain, and the surface it handed over with a reference orb has to release.
pub fn get_back_buffer(
    device: Device,
    swap_chain: u32,
    index: u32,
    kind: u32,
) -> (Hresult, Option<Surface>) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        let _ = swap_chain;
        return win.get_back_buffer(device, index, kind);
    }
    host::get_back_buffer(device, swap_chain, index, kind)
}

/// `CreateOffscreenPlainSurface`, the shared handle left null — what a surface outside the device's own memory is
/// made by, which is the one kind a render target can be copied into.
pub fn create_offscreen_plain_surface(
    device: Device,
    width: u32,
    height: u32,
    format: u32,
    pool: u32,
) -> (Hresult, Option<Surface>) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        let _ = pool;
        return win.create_image_surface(device, width, height, format);
    }
    host::create_offscreen_plain_surface(device, width, height, format, pool)
}

/// `GetRenderTargetData`, a render target's contents copied into a surface in system memory — the `CopyRects`
/// of Direct3D 8, which a simulated Windows has one of.
pub fn get_render_target_data(device: Device, from: Surface, to: Surface) -> Hresult {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.copy_rects(device, from, to);
    }
    host::get_render_target_data(device, from, to)
}

/// `IDirect3DSurface9::GetDesc`, and `None` where it failed.
pub fn surface_desc(surface: Surface) -> Option<SurfaceDesc9> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.surface_desc(surface).map(|desc| SurfaceDesc9 {
            format: desc.format,
            kind: desc.kind,
            usage: desc.usage,
            pool: desc.pool,
            multi_sample_type: desc.multi_sample_type,
            multi_sample_quality: 0,
            width: desc.width,
            height: desc.height,
        });
    }
    host::surface_desc(surface)
}

/// `IDirect3DSurface9::LockRect` over the whole surface, and `None` where it failed or handed back nothing.
pub fn lock_surface(surface: Surface, flags: u32) -> Option<Locked> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.lock_surface(surface, flags);
    }
    host::lock_surface(surface, flags)
}

/// `IDirect3DSurface9::UnlockRect`.
pub fn unlock_surface(surface: Surface) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.unlock_surface(surface);
    }
    host::unlock_surface(surface);
}

/// `Release` of a surface, for the back buffer's own reference and for the surface orb made.
pub fn release_surface(surface: Surface) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.release_surface(surface);
    }
    host::release_surface(surface);
}

// --- the layouts -------------------------------------------------------------

/// `IDirect3DDevice9`'s vtable, as far as the slots orb calls are concerned.
#[repr(C)]
pub struct DeviceVtable {
    pub _iunknown: [usize; 3],
    pub _slot_3_to_17: [usize; 15],
    pub get_back_buffer:
        unsafe extern "system" fn(*mut c_void, u32, u32, u32, *mut *mut c_void) -> Hresult,
    pub _slot_19_to_22: [usize; 4],
    pub create_texture: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        *mut *mut c_void,
        *mut c_void,
    ) -> Hresult,
    pub _slot_24_to_31: [usize; 8],
    pub get_render_target_data:
        unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> Hresult,
    pub _slot_33_to_35: [usize; 3],
    pub create_offscreen_plain_surface: unsafe extern "system" fn(
        *mut c_void,
        u32,
        u32,
        u32,
        u32,
        *mut *mut c_void,
        *mut c_void,
    ) -> Hresult,
    pub _slot_37_to_40: [usize; 4],
    pub begin_scene: unsafe extern "system" fn(*mut c_void) -> Hresult,
    pub end_scene: unsafe extern "system" fn(*mut c_void) -> Hresult,
    pub _slot_43_to_46: [usize; 4],
    pub set_viewport: unsafe extern "system" fn(*mut c_void, *const Viewport) -> Hresult,
    pub get_viewport: unsafe extern "system" fn(*mut c_void, *mut Viewport) -> Hresult,
    pub _slot_49_to_56: [usize; 8],
    pub set_render_state: unsafe extern "system" fn(*mut c_void, u32, u32) -> Hresult,
    pub _slot_58: [usize; 1],
    pub create_state_block:
        unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> Hresult,
    pub _slot_60_to_64: [usize; 5],
    pub set_texture: unsafe extern "system" fn(*mut c_void, u32, *mut c_void) -> Hresult,
    pub _slot_66: [usize; 1],
    pub set_texture_stage_state: unsafe extern "system" fn(*mut c_void, u32, u32, u32) -> Hresult,
    pub _slot_68: [usize; 1],
    pub set_sampler_state: unsafe extern "system" fn(*mut c_void, u32, u32, u32) -> Hresult,
    pub _slot_70_to_82: [usize; 13],
    pub draw_primitive_up:
        unsafe extern "system" fn(*mut c_void, u32, u32, *const c_void, u32) -> Hresult,
    pub _slot_84_to_88: [usize; 5],
    pub set_fvf: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
}

/// And `IDirect3DTexture9`'s.
#[repr(C)]
pub struct TextureVtable {
    pub _query_interface: usize,
    pub _add_ref: usize,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub _slot_3_to_18: [usize; 16],
    pub lock_rect:
        unsafe extern "system" fn(*mut c_void, u32, *mut RawLocked, *const c_void, u32) -> Hresult,
    pub unlock_rect: unsafe extern "system" fn(*mut c_void, u32) -> Hresult,
}

/// And `IDirect3DSurface9`'s.
#[repr(C)]
pub struct SurfaceVtable {
    pub _query_interface: usize,
    pub _add_ref: usize,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub _slot_3_to_11: [usize; 9],
    pub get_desc: unsafe extern "system" fn(*mut c_void, *mut SurfaceDesc9) -> Hresult,
    pub lock_rect:
        unsafe extern "system" fn(*mut c_void, *mut RawLocked, *const c_void, u32) -> Hresult,
    pub unlock_rect: unsafe extern "system" fn(*mut c_void) -> Hresult,
}

/// And `IDirect3DStateBlock9`'s.
#[repr(C)]
pub struct StateBlockVtable {
    pub _query_interface: usize,
    pub _add_ref: usize,
    pub release: unsafe extern "system" fn(*mut c_void) -> u32,
    pub _get_device: usize,
    pub capture: unsafe extern "system" fn(*mut c_void) -> Hresult,
    pub apply: unsafe extern "system" fn(*mut c_void) -> Hresult,
}

const fn slot(index: usize) -> usize {
    index * size_of::<usize>()
}

// Each against `d3d9.h`.
const _: () = {
    assert!(offset_of!(DeviceVtable, get_back_buffer) == slot(18));
    assert!(offset_of!(DeviceVtable, create_texture) == slot(23));
    assert!(offset_of!(DeviceVtable, get_render_target_data) == slot(32));
    assert!(offset_of!(DeviceVtable, create_offscreen_plain_surface) == slot(36));
    assert!(offset_of!(DeviceVtable, begin_scene) == slot(41));
    assert!(offset_of!(DeviceVtable, end_scene) == slot(42));
    assert!(offset_of!(DeviceVtable, set_viewport) == slot(47));
    assert!(offset_of!(DeviceVtable, get_viewport) == slot(48));
    assert!(offset_of!(DeviceVtable, set_render_state) == slot(57));
    assert!(offset_of!(DeviceVtable, create_state_block) == slot(59));
    assert!(offset_of!(DeviceVtable, set_texture) == slot(65));
    assert!(offset_of!(DeviceVtable, set_texture_stage_state) == slot(67));
    assert!(offset_of!(DeviceVtable, set_sampler_state) == slot(69));
    assert!(offset_of!(DeviceVtable, draw_primitive_up) == slot(83));
    assert!(offset_of!(DeviceVtable, set_fvf) == slot(89));
    assert!(offset_of!(TextureVtable, release) == slot(2));
    assert!(offset_of!(TextureVtable, lock_rect) == slot(19));
    assert!(offset_of!(TextureVtable, unlock_rect) == slot(20));
    assert!(offset_of!(SurfaceVtable, release) == slot(2));
    assert!(offset_of!(SurfaceVtable, get_desc) == slot(12));
    assert!(offset_of!(SurfaceVtable, lock_rect) == slot(13));
    assert!(offset_of!(SurfaceVtable, unlock_rect) == slot(14));
    assert!(offset_of!(StateBlockVtable, release) == slot(2));
    assert!(offset_of!(StateBlockVtable, capture) == slot(4));
    assert!(offset_of!(StateBlockVtable, apply) == slot(5));
};

// And those th10 calls against the offset it calls them through, which are a 32-bit game's: `[ecx+0xa4]` at
// 0x439496, `[edx+0xa8]` at 0x439502, `[ecx+0xbc]` at 0x439419, `[ecx+0xe4]` at 0x420cfa, `[ecx+0x104]` at
// 0x4394f4, `[ecx+0x10c]` at 0x442f68, `[edx+0x14c]` at 0x442fb1 and `[ecx+0x164]` at 0x442f8f.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(offset_of!(DeviceVtable, begin_scene) == 0xa4);
    assert!(offset_of!(DeviceVtable, end_scene) == 0xa8);
    assert!(offset_of!(DeviceVtable, set_viewport) == 0xbc);
    assert!(offset_of!(DeviceVtable, set_render_state) == 0xe4);
    assert!(offset_of!(DeviceVtable, set_texture) == 0x104);
    assert!(offset_of!(DeviceVtable, set_texture_stage_state) == 0x10c);
    assert!(offset_of!(DeviceVtable, draw_primitive_up) == 0x14c);
    assert!(offset_of!(DeviceVtable, set_fvf) == 0x164);
};

#[cfg(windows)]
use crate::real::d3d9 as host;

#[cfg(not(windows))]
mod host {
    use super::SurfaceDesc9;
    use crate::{Device, Hresult, Locked, Surface, Texture, Viewport, no_windows};

    pub fn begin_scene(_device: Device) {
        no_windows("d3d9::begin_scene")
    }
    pub fn end_scene(_device: Device) {
        no_windows("d3d9::end_scene")
    }
    pub fn set_texture(_device: Device, _stage: u32, _texture: Option<Texture>) {
        no_windows("d3d9::set_texture")
    }
    pub fn set_viewport(_device: Device, _viewport: Viewport) {
        no_windows("d3d9::set_viewport")
    }
    pub fn get_viewport(_device: Device) -> Viewport {
        no_windows("d3d9::get_viewport")
    }
    pub fn create_texture(
        _device: Device,
        _width: u32,
        _height: u32,
        _levels: u32,
        _usage: u32,
        _format: u32,
        _pool: u32,
    ) -> (Hresult, Option<Texture>) {
        no_windows("d3d9::create_texture")
    }
    pub fn lock_rect(_texture: Texture, _level: u32, _flags: u32) -> Option<Locked> {
        no_windows("d3d9::lock_rect")
    }
    pub fn unlock_rect(_texture: Texture, _level: u32) {
        no_windows("d3d9::unlock_rect")
    }
    pub fn release_texture(_texture: Texture) {
        no_windows("d3d9::release_texture")
    }
    pub fn create_state_block(_device: Device, _kind: u32) -> (Hresult, u32) {
        no_windows("d3d9::create_state_block")
    }
    pub fn capture_state_block(_device: Device, _token: u32) -> Hresult {
        no_windows("d3d9::capture_state_block")
    }
    pub fn apply_state_block(_device: Device, _token: u32) {
        no_windows("d3d9::apply_state_block")
    }
    pub fn delete_state_block(_device: Device, _token: u32) {
        no_windows("d3d9::delete_state_block")
    }
    pub fn set_render_state(_device: Device, _state: u32, _value: u32) {
        no_windows("d3d9::set_render_state")
    }
    pub fn set_texture_stage_state(_device: Device, _stage: u32, _kind: u32, _value: u32) {
        no_windows("d3d9::set_texture_stage_state")
    }
    pub fn set_sampler_state(_device: Device, _sampler: u32, _kind: u32, _value: u32) {
        no_windows("d3d9::set_sampler_state")
    }
    pub fn set_fvf(_device: Device, _fvf: u32) {
        no_windows("d3d9::set_fvf")
    }
    pub fn draw_primitive_up(
        _device: Device,
        _kind: u32,
        _count: u32,
        _vertices: &[u8],
        _stride: u32,
    ) {
        no_windows("d3d9::draw_primitive_up")
    }
    pub fn get_back_buffer(
        _device: Device,
        _swap_chain: u32,
        _index: u32,
        _kind: u32,
    ) -> (Hresult, Option<Surface>) {
        no_windows("d3d9::get_back_buffer")
    }
    pub fn create_offscreen_plain_surface(
        _device: Device,
        _width: u32,
        _height: u32,
        _format: u32,
        _pool: u32,
    ) -> (Hresult, Option<Surface>) {
        no_windows("d3d9::create_offscreen_plain_surface")
    }
    pub fn get_render_target_data(_device: Device, _from: Surface, _to: Surface) -> Hresult {
        no_windows("d3d9::get_render_target_data")
    }
    pub fn surface_desc(_surface: Surface) -> Option<SurfaceDesc9> {
        no_windows("d3d9::surface_desc")
    }
    pub fn lock_surface(_surface: Surface, _flags: u32) -> Option<Locked> {
        no_windows("d3d9::lock_surface")
    }
    pub fn unlock_surface(_surface: Surface) {
        no_windows("d3d9::unlock_surface")
    }
    pub fn release_surface(_surface: Surface) {
        no_windows("d3d9::release_surface")
    }
}
