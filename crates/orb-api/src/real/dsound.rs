//! The sound buffer, called through the vtable the game handed over.
//!
//! **The only code in the workspace that calls a DirectSound vtable**, the way `real::d3d8` is the only
//! one that calls a Direct3D one. Every slot here is one line and decides nothing.

use std::ffi::c_void;

use crate::dsound::SoundBufferVtable;
use crate::{Hresult, LockedBuffer, SoundBuffer};

fn vtable(buffer: SoundBuffer) -> &'static SoundBufferVtable {
    unsafe { &*(*(buffer.0 as *const *const SoundBufferVtable)) }
}

fn object(buffer: SoundBuffer) -> *mut c_void {
    buffer.0 as *mut c_void
}

pub fn get_current_position(buffer: SoundBuffer) -> (Hresult, u32, u32) {
    let mut play = 0;
    let mut write = 0;
    let result =
        unsafe { (vtable(buffer).get_current_position)(object(buffer), &mut play, &mut write) };
    (result, play, write)
}

pub fn get_status(buffer: SoundBuffer) -> (Hresult, u32) {
    let mut status = 0;
    let result = unsafe { (vtable(buffer).get_status)(object(buffer), &mut status) };
    (result, status)
}

pub fn lock(buffer: SoundBuffer, offset: u32, bytes: u32, flags: u32) -> (Hresult, LockedBuffer) {
    let mut first: *mut c_void = std::ptr::null_mut();
    let mut first_bytes = 0;
    let mut second: *mut c_void = std::ptr::null_mut();
    let mut second_bytes = 0;
    let result = unsafe {
        (vtable(buffer).lock)(
            object(buffer),
            offset,
            bytes,
            &mut first,
            &mut first_bytes,
            &mut second,
            &mut second_bytes,
            flags,
        )
    };
    (
        result,
        LockedBuffer {
            first: first as usize,
            first_bytes,
            second: second as usize,
            second_bytes,
        },
    )
}

pub fn unlock(buffer: SoundBuffer, locked: LockedBuffer) {
    unsafe {
        (vtable(buffer).unlock)(
            object(buffer),
            locked.first as *mut c_void,
            locked.first_bytes,
            locked.second as *mut c_void,
            locked.second_bytes,
        )
    };
}

pub fn play(buffer: SoundBuffer, reserved: u32, priority: u32, flags: u32) {
    unsafe { (vtable(buffer).play)(object(buffer), reserved, priority, flags) };
}

pub fn stop(buffer: SoundBuffer) {
    unsafe { (vtable(buffer).stop)(object(buffer)) };
}

pub fn set_current_position(buffer: SoundBuffer, position: u32) {
    unsafe { (vtable(buffer).set_current_position)(object(buffer), position) };
}

pub fn restore(buffer: SoundBuffer) {
    unsafe { (vtable(buffer).restore)(object(buffer)) };
}
