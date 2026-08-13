//! Noticing where the game's own allocations are.
//!
//! The game keeps most of its state in `.data`, but not all of it: `malloc` in
//! its statically linked MSVC6 CRT goes to a private heap, and a few things come
//! straight from `VirtualAlloc`. Both are reached through the exe's imports, so
//! hooking those imports catches every region the game owns without catching
//! d3d8's or dsound's, whose allocations must be left alone.
//!
//! Heap contents are saved together with the allocator's own bookkeeping, which
//! is what makes a restored snapshot hand back the identical addresses.
//!
//! **Import hooks and nothing else.** What a region is and how two of them are held apart is
//! [`orb_core::memtrack`], where a snapshot reads them; the walk of what these noticed is behind the
//! seam, `HeapLock`, `HeapWalk` and `VirtualQuery` being the host's and a laid-out game answering the
//! same question out of its own address space. So each hook hands the handle or the range over as it
//! sees it — see [`orb_api::mem::note_heap`].

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{BOOL, HANDLE};
use windows_sys::Win32::System::Memory::{
    HEAP_FLAGS, MEM_RELEASE, VIRTUAL_ALLOCATION_TYPE, VIRTUAL_FREE_TYPE,
};

use crate::hook;

static HEAP_CREATE: AtomicUsize = AtomicUsize::new(0);
static HEAP_ALLOC: AtomicUsize = AtomicUsize::new(0);
static HEAP_REALLOC: AtomicUsize = AtomicUsize::new(0);
static HEAP_FREE: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_ALLOC: AtomicUsize = AtomicUsize::new(0);
static VIRTUAL_FREE: AtomicUsize = AtomicUsize::new(0);

/// # Safety
/// Must run before the game's entry point, or allocations made in between go
/// unrecorded. Patches `module`'s imports, so nothing else may be executing it.
pub unsafe fn install(module: usize) -> Result<(), hook::Error> {
    unsafe {
        for (function, replacement, original) in [
            ("HeapCreate", hook::address(heap_create as _), &HEAP_CREATE),
            ("HeapAlloc", hook::address(heap_alloc as _), &HEAP_ALLOC),
            (
                "HeapReAlloc",
                hook::address(heap_realloc as _),
                &HEAP_REALLOC,
            ),
            ("HeapFree", hook::address(heap_free as _), &HEAP_FREE),
            (
                "VirtualAlloc",
                hook::address(virtual_alloc as _),
                &VIRTUAL_ALLOC,
            ),
            (
                "VirtualFree",
                hook::address(virtual_free as _),
                &VIRTUAL_FREE,
            ),
        ] {
            let previous = hook::install_import(module, "KERNEL32.dll", function, replacement)?;
            original.store(previous, Ordering::Relaxed);
        }
    }
    Ok(())
}

type HeapCreate = unsafe extern "system" fn(HEAP_FLAGS, usize, usize) -> HANDLE;
type HeapAlloc = unsafe extern "system" fn(HANDLE, HEAP_FLAGS, usize) -> *mut c_void;
type HeapReAlloc =
    unsafe extern "system" fn(HANDLE, HEAP_FLAGS, *const c_void, usize) -> *mut c_void;
type HeapFree = unsafe extern "system" fn(HANDLE, HEAP_FLAGS, *const c_void) -> BOOL;
type VirtualAlloc =
    unsafe extern "system" fn(*const c_void, usize, VIRTUAL_ALLOCATION_TYPE, u32) -> *mut c_void;
type VirtualFree = unsafe extern "system" fn(*mut c_void, usize, VIRTUAL_FREE_TYPE) -> BOOL;

unsafe extern "system" fn heap_create(flags: HEAP_FLAGS, initial: usize, max: usize) -> HANDLE {
    let original: HeapCreate = unsafe { std::mem::transmute(HEAP_CREATE.load(Ordering::Relaxed)) };
    let heap = unsafe { original(flags, initial, max) };
    if !heap.is_null() {
        orb_api::mem::note_heap(heap as usize);
    }
    heap
}

unsafe extern "system" fn heap_alloc(heap: HANDLE, flags: HEAP_FLAGS, bytes: usize) -> *mut c_void {
    let original: HeapAlloc = unsafe { std::mem::transmute(HEAP_ALLOC.load(Ordering::Relaxed)) };
    orb_api::mem::note_heap(heap as usize);
    unsafe { original(heap, flags, bytes) }
}

unsafe extern "system" fn heap_realloc(
    heap: HANDLE,
    flags: HEAP_FLAGS,
    memory: *const c_void,
    bytes: usize,
) -> *mut c_void {
    let original: HeapReAlloc =
        unsafe { std::mem::transmute(HEAP_REALLOC.load(Ordering::Relaxed)) };
    orb_api::mem::note_heap(heap as usize);
    unsafe { original(heap, flags, memory, bytes) }
}

unsafe extern "system" fn heap_free(
    heap: HANDLE,
    flags: HEAP_FLAGS,
    memory: *const c_void,
) -> BOOL {
    let original: HeapFree = unsafe { std::mem::transmute(HEAP_FREE.load(Ordering::Relaxed)) };
    orb_api::mem::note_heap(heap as usize);
    unsafe { original(heap, flags, memory) }
}

unsafe extern "system" fn virtual_alloc(
    address: *const c_void,
    size: usize,
    allocation_type: VIRTUAL_ALLOCATION_TYPE,
    protection: u32,
) -> *mut c_void {
    let original: VirtualAlloc =
        unsafe { std::mem::transmute(VIRTUAL_ALLOC.load(Ordering::Relaxed)) };
    let allocated = unsafe { original(address, size, allocation_type, protection) };
    if !allocated.is_null() {
        orb_api::mem::note_reservation(allocated as usize, size);
    }
    allocated
}

unsafe extern "system" fn virtual_free(
    address: *mut c_void,
    size: usize,
    free_type: VIRTUAL_FREE_TYPE,
) -> BOOL {
    let original: VirtualFree =
        unsafe { std::mem::transmute(VIRTUAL_FREE.load(Ordering::Relaxed)) };
    let freed = unsafe { original(address, size, free_type) };
    if freed != 0 && free_type & MEM_RELEASE != 0 {
        orb_api::mem::forget_reservation(address as usize);
    }
    freed
}
