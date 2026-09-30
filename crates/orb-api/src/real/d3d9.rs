//! The Direct3D 9 device, its textures and its state blocks, called through the vtables the game handed
//! over — the same one line a slot [`super::d3d8`] is.

use std::ffi::c_void;

use crate::d3d8::RawLocked;
use crate::d3d9::{DeviceVtable, StateBlockVtable, SurfaceDesc9, SurfaceVtable, TextureVtable};
use crate::{Device, Hresult, Locked, Surface, Texture, Viewport};

/// The vtable of an object at an address: its first field is the pointer to it, which is what a COM
/// object is.
///
/// # Safety
/// `object` must be a live COM object whose vtable is laid out as `V` says.
unsafe fn vtable<V>(object: usize) -> &'static V {
    unsafe { &*(*(object as *const *const V)) }
}

fn device_vtable(device: Device) -> &'static DeviceVtable {
    unsafe { vtable(device.0) }
}

fn texture_vtable(texture: Texture) -> &'static TextureVtable {
    unsafe { vtable(texture.0) }
}

fn surface_vtable(surface: Surface) -> &'static SurfaceVtable {
    unsafe { vtable(surface.0) }
}

fn state_block_vtable(token: u32) -> &'static StateBlockVtable {
    unsafe { vtable(token as usize) }
}

fn object(address: usize) -> *mut c_void {
    address as *mut c_void
}

pub fn begin_scene(device: Device) {
    unsafe { (device_vtable(device).begin_scene)(object(device.0)) };
}

pub fn end_scene(device: Device) {
    unsafe { (device_vtable(device).end_scene)(object(device.0)) };
}

pub fn set_texture(device: Device, stage: u32, texture: Option<Texture>) {
    let texture = texture.map_or(std::ptr::null_mut(), |texture| object(texture.0));
    unsafe { (device_vtable(device).set_texture)(object(device.0), stage, texture) };
}

pub fn set_viewport(device: Device, viewport: Viewport) {
    unsafe { (device_vtable(device).set_viewport)(object(device.0), &viewport) };
}

pub fn get_viewport(device: Device) -> Viewport {
    let mut viewport = Viewport::default();
    unsafe { (device_vtable(device).get_viewport)(object(device.0), &mut viewport) };
    viewport
}

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
    let mut texture: *mut c_void = std::ptr::null_mut();
    let created = unsafe {
        (device_vtable(device).create_texture)(
            object(device.0),
            width,
            height,
            levels,
            usage,
            format,
            pool,
            &mut texture,
            std::ptr::null_mut(),
        )
    };
    if created < 0 || texture.is_null() {
        return (created, None);
    }
    (created, Some(Texture(texture as usize)))
}

pub fn lock_rect(texture: Texture, level: u32, flags: u32) -> Option<Locked> {
    let mut locked = RawLocked {
        pitch: 0,
        bits: std::ptr::null_mut(),
    };
    let result = unsafe {
        (texture_vtable(texture).lock_rect)(
            object(texture.0),
            level,
            &mut locked,
            std::ptr::null(),
            flags,
        )
    };
    if result < 0 || locked.bits.is_null() {
        return None;
    }
    Some(Locked {
        pitch: locked.pitch,
        bits: locked.bits as usize,
    })
}

pub fn unlock_rect(texture: Texture, level: u32) {
    unsafe { (texture_vtable(texture).unlock_rect)(object(texture.0), level) };
}

pub fn release_texture(texture: Texture) {
    unsafe { (texture_vtable(texture).release)(object(texture.0)) };
}

pub fn create_state_block(device: Device, kind: u32) -> (Hresult, u32) {
    let mut block: *mut c_void = std::ptr::null_mut();
    let created =
        unsafe { (device_vtable(device).create_state_block)(object(device.0), kind, &mut block) };
    (created, block as usize as u32)
}

// The device beside the token only because Direct3D 8's three take it: a Direct3D 9 block is an object of
// its own and is called on itself.
pub fn capture_state_block(_device: Device, token: u32) -> Hresult {
    unsafe { (state_block_vtable(token).capture)(object(token as usize)) }
}

pub fn apply_state_block(_device: Device, token: u32) {
    unsafe { (state_block_vtable(token).apply)(object(token as usize)) };
}

pub fn delete_state_block(_device: Device, token: u32) {
    unsafe { (state_block_vtable(token).release)(object(token as usize)) };
}

pub fn set_render_state(device: Device, state: u32, value: u32) {
    unsafe { (device_vtable(device).set_render_state)(object(device.0), state, value) };
}

pub fn set_texture_stage_state(device: Device, stage: u32, kind: u32, value: u32) {
    unsafe {
        (device_vtable(device).set_texture_stage_state)(object(device.0), stage, kind, value)
    };
}

pub fn set_sampler_state(device: Device, sampler: u32, kind: u32, value: u32) {
    unsafe { (device_vtable(device).set_sampler_state)(object(device.0), sampler, kind, value) };
}

pub fn set_fvf(device: Device, fvf: u32) {
    unsafe { (device_vtable(device).set_fvf)(object(device.0), fvf) };
}

pub fn draw_primitive_up(device: Device, kind: u32, count: u32, vertices: &[u8], stride: u32) {
    unsafe {
        (device_vtable(device).draw_primitive_up)(
            object(device.0),
            kind,
            count,
            vertices.as_ptr() as *const c_void,
            stride,
        )
    };
}

pub fn get_back_buffer(
    device: Device,
    swap_chain: u32,
    index: u32,
    kind: u32,
) -> (Hresult, Option<Surface>) {
    let mut back: *mut c_void = std::ptr::null_mut();
    let got = unsafe {
        (device_vtable(device).get_back_buffer)(
            object(device.0),
            swap_chain,
            index,
            kind,
            &mut back,
        )
    };
    if got < 0 || back.is_null() {
        return (got, None);
    }
    (got, Some(Surface(back as usize)))
}

pub fn create_offscreen_plain_surface(
    device: Device,
    width: u32,
    height: u32,
    format: u32,
    pool: u32,
) -> (Hresult, Option<Surface>) {
    let mut surface: *mut c_void = std::ptr::null_mut();
    let created = unsafe {
        (device_vtable(device).create_offscreen_plain_surface)(
            object(device.0),
            width,
            height,
            format,
            pool,
            &mut surface,
            std::ptr::null_mut(),
        )
    };
    if created < 0 || surface.is_null() {
        return (created, None);
    }
    (created, Some(Surface(surface as usize)))
}

pub fn get_render_target_data(device: Device, from: Surface, to: Surface) -> Hresult {
    unsafe {
        (device_vtable(device).get_render_target_data)(
            object(device.0),
            object(from.0),
            object(to.0),
        )
    }
}

pub fn surface_desc(surface: Surface) -> Option<SurfaceDesc9> {
    let mut desc = SurfaceDesc9::default();
    let got = unsafe { (surface_vtable(surface).get_desc)(object(surface.0), &mut desc) };
    (got >= 0).then_some(desc)
}

pub fn lock_surface(surface: Surface, flags: u32) -> Option<Locked> {
    let mut locked = RawLocked {
        pitch: 0,
        bits: std::ptr::null_mut(),
    };
    let result = unsafe {
        (surface_vtable(surface).lock_rect)(object(surface.0), &mut locked, std::ptr::null(), flags)
    };
    if result < 0 || locked.bits.is_null() {
        return None;
    }
    Some(Locked {
        pitch: locked.pitch,
        bits: locked.bits as usize,
    })
}

pub fn unlock_surface(surface: Surface) {
    unsafe { (surface_vtable(surface).unlock_rect)(object(surface.0)) };
}

pub fn release_surface(surface: Surface) {
    unsafe { (surface_vtable(surface).release)(object(surface.0)) };
}
