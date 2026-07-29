//! Raw reads and writes against the game's own memory.
//!
//! Volatile throughout: the compiler has no idea the game writes these
//! locations every frame, so a plain read is free to be hoisted or reused.

use std::ffi::c_void;

use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_IMAGE, MEMORY_BASIC_INFORMATION, VirtualQuery,
};

pub unsafe fn read<T: Copy>(address: usize) -> T {
    unsafe { (address as *const T).read_volatile() }
}

pub unsafe fn write<T: Copy>(address: usize, value: T) {
    unsafe { (address as *mut T).write_volatile(value) }
}

/// Whether `address` holds a pointer to a mapped image, which is what a live COM
/// object's vtable pointer looks like.
///
/// The game's allocator does not scrub freed blocks, so a stale pointer to a
/// released object still reads back as its old contents. Following one into
/// DirectSound crashes, and this is the cheap check that catches most of them.
pub fn vtable_in_image(address: usize) -> bool {
    let Some(vtable) = read_committed::<usize>(address) else { return false };
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let queried = unsafe {
        VirtualQuery(vtable as *const c_void, &mut info, size_of::<MEMORY_BASIC_INFORMATION>())
    };
    queried != 0 && info.State == MEM_COMMIT && info.Type == MEM_IMAGE
}

/// Reads only if the whole value sits in committed memory, for chasing pointers
/// out of the game's structures that may not be valid yet or any more.
pub fn read_committed<T: Copy>(address: usize) -> Option<T> {
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let queried = unsafe {
        VirtualQuery(address as *const c_void, &mut info, size_of::<MEMORY_BASIC_INFORMATION>())
    };
    let end = info.BaseAddress as usize + info.RegionSize;
    if queried == 0 || info.State != MEM_COMMIT || address + size_of::<T>() > end {
        return None;
    }
    Some(unsafe { read(address) })
}
