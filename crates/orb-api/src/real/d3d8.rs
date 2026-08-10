//! The device and its textures, called through the vtable the game handed over.
//!
//! **The only code in the workspace that calls a Direct3D vtable.** Every slot here is one line: read
//! the vtable out of the object, call the slot, hand back what it answered. Nothing decides anything —
//! which render states the drawing sets and how it brackets a draw are above the seam, where an e2e test
//! reaches them.

use std::ffi::c_void;

use crate::d3d8::{DeviceVtable, RawLocked, TextureVtable};
use crate::{Device, Hresult, Locked, Texture, Viewport};

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

fn object(address: usize) -> *mut c_void {
    address as *mut c_void
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
        )
    };
    if created < 0 || texture.is_null() {
        return (created, None);
    }
    (created, Some(Texture(texture as usize)))
}

pub fn create_state_block(device: Device, kind: u32) -> (Hresult, u32) {
    let mut token = 0;
    let created =
        unsafe { (device_vtable(device).create_state_block)(object(device.0), kind, &mut token) };
    (created, token)
}

pub fn capture_state_block(device: Device, token: u32) -> Hresult {
    unsafe { (device_vtable(device).capture_state_block)(object(device.0), token) }
}

pub fn apply_state_block(device: Device, token: u32) {
    unsafe { (device_vtable(device).apply_state_block)(object(device.0), token) };
}

pub fn delete_state_block(device: Device, token: u32) {
    unsafe { (device_vtable(device).delete_state_block)(object(device.0), token) };
}

pub fn set_render_state(device: Device, state: u32, value: u32) {
    unsafe { (device_vtable(device).set_render_state)(object(device.0), state, value) };
}

pub fn set_texture_stage_state(device: Device, stage: u32, kind: u32, value: u32) {
    unsafe {
        (device_vtable(device).set_texture_stage_state)(object(device.0), stage, kind, value)
    };
}

pub fn set_texture(device: Device, stage: u32, texture: Option<Texture>) {
    let texture = texture.map_or(std::ptr::null_mut(), |texture| object(texture.0));
    unsafe { (device_vtable(device).set_texture)(object(device.0), stage, texture) };
}

pub fn set_vertex_shader(device: Device, shader: u32) {
    unsafe { (device_vtable(device).set_vertex_shader)(object(device.0), shader) };
}

pub fn set_viewport(device: Device, viewport: Viewport) {
    unsafe { (device_vtable(device).set_viewport)(object(device.0), &viewport) };
}

pub fn get_viewport(device: Device) -> Viewport {
    let mut viewport = Viewport::default();
    unsafe { (device_vtable(device).get_viewport)(object(device.0), &mut viewport) };
    viewport
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

pub fn begin_scene(device: Device) {
    unsafe { (device_vtable(device).begin_scene)(object(device.0)) };
}

pub fn end_scene(device: Device) {
    unsafe { (device_vtable(device).end_scene)(object(device.0)) };
}

pub fn clear(device: Device, flags: u32, color: u32, z: f32, stencil: u32) {
    unsafe {
        (device_vtable(device).clear)(
            object(device.0),
            0,
            std::ptr::null(),
            flags,
            color,
            z,
            stencil,
        )
    };
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
