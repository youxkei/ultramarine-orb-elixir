//! The parts of Direct3D 8 the overlay needs.
//!
//! Windows' own metadata does not describe D3D8, so the vtables are declared
//! here. Only the methods that get called are typed; the rest are pointer-sized
//! padding, and every typed method's slot is asserted at compile time against
//! the index it has in `d3d8.h`. A wrong index would be a call into an unrelated
//! method with the wrong signature.

use std::ffi::c_void;
use std::mem::offset_of;

#[cfg(test)]
pub mod recording;

pub type Hresult = i32;

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

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub min_z: f32,
    pub max_z: f32,
}

#[repr(C)]
pub struct LockedRect {
    pub pitch: i32,
    pub bits: *mut c_void,
}

#[repr(C)]
pub struct Device {
    pub vtable: *const DeviceVtable,
}

#[repr(C)]
pub struct DeviceVtable {
    _iunknown: [usize; 3],
    _slot_3_to_19: [usize; 17],
    pub create_texture: unsafe extern "system" fn(
        *mut Device,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        *mut *mut Texture,
    ) -> Hresult,
    _slot_21_to_33: [usize; 13],
    pub begin_scene: unsafe extern "system" fn(*mut Device) -> Hresult,
    pub end_scene: unsafe extern "system" fn(*mut Device) -> Hresult,
    pub clear:
        unsafe extern "system" fn(*mut Device, u32, *const c_void, u32, u32, f32, u32) -> Hresult,
    _slot_37_to_39: [usize; 3],
    pub set_viewport: unsafe extern "system" fn(*mut Device, *const Viewport) -> Hresult,
    pub get_viewport: unsafe extern "system" fn(*mut Device, *mut Viewport) -> Hresult,
    _slot_42_to_49: [usize; 8],
    pub set_render_state: unsafe extern "system" fn(*mut Device, u32, u32) -> Hresult,
    _slot_51_to_53: [usize; 3],
    pub apply_state_block: unsafe extern "system" fn(*mut Device, u32) -> Hresult,
    pub capture_state_block: unsafe extern "system" fn(*mut Device, u32) -> Hresult,
    pub delete_state_block: unsafe extern "system" fn(*mut Device, u32) -> Hresult,
    pub create_state_block: unsafe extern "system" fn(*mut Device, u32, *mut u32) -> Hresult,
    _slot_58_to_60: [usize; 3],
    pub set_texture: unsafe extern "system" fn(*mut Device, u32, *mut Texture) -> Hresult,
    _slot_62: [usize; 1],
    pub set_texture_stage_state: unsafe extern "system" fn(*mut Device, u32, u32, u32) -> Hresult,
    _slot_64_to_71: [usize; 8],
    pub draw_primitive_up:
        unsafe extern "system" fn(*mut Device, u32, u32, *const c_void, u32) -> Hresult,
    _slot_73_to_75: [usize; 3],
    pub set_vertex_shader: unsafe extern "system" fn(*mut Device, u32) -> Hresult,
}

#[repr(C)]
pub struct Texture {
    pub vtable: *const TextureVtable,
}

#[repr(C)]
pub struct TextureVtable {
    _query_interface: usize,
    _add_ref: usize,
    pub release: unsafe extern "system" fn(*mut Texture) -> u32,
    _slot_3_to_15: [usize; 13],
    pub lock_rect: unsafe extern "system" fn(
        *mut Texture,
        u32,
        *mut LockedRect,
        *const c_void,
        u32,
    ) -> Hresult,
    pub unlock_rect: unsafe extern "system" fn(*mut Texture, u32) -> Hresult,
}

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
