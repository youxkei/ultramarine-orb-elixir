//! Where the game's own allocations live.
//!
//! The game keeps most of its state in `.data`, but not all of it: `malloc` in
//! its statically linked MSVC6 CRT goes to a private heap, and a few things come
//! straight from `VirtualAlloc`. Both are reached through the exe's imports, so
//! hooking those imports catches every region the game owns without catching
//! d3d8's or dsound's, whose allocations must be left alone.
//!
//! Heap contents are saved together with the allocator's own bookkeeping, which
//! is what makes a restored snapshot hand back the identical addresses.

use std::ffi::c_void;
use std::ops::Range;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{BOOL, HANDLE};
use windows_sys::Win32::System::Memory::{
    HEAP_FLAGS, HeapLock, HeapUnlock, HeapWalk, MEM_COMMIT, MEM_RELEASE, MEMORY_BASIC_INFORMATION,
    PAGE_GUARD, PAGE_NOACCESS, PROCESS_HEAP_ENTRY, VIRTUAL_ALLOCATION_TYPE, VIRTUAL_FREE_TYPE,
    VirtualQuery,
};
use windows_sys::Win32::System::SystemServices::PROCESS_HEAP_REGION;

use crate::hook;
use crate::profile;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Region {
    pub base: usize,
    pub len: usize,
}

impl Region {
    pub fn end(&self) -> usize {
        self.base + self.len
    }
}

/// Heap handles and direct reservations seen so far. Any game thread can
/// allocate, so this is the one place in `orb` that needs a lock.
static TRACKED: Mutex<Tracked> = Mutex::new(Tracked {
    heaps: Vec::new(),
    reservations: Vec::new(),
});

struct Tracked {
    heaps: Vec<usize>,
    reservations: Vec<Region>,
}

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
            ("HeapCreate", heap_create as usize, &HEAP_CREATE),
            ("HeapAlloc", heap_alloc as usize, &HEAP_ALLOC),
            ("HeapReAlloc", heap_realloc as usize, &HEAP_REALLOC),
            ("HeapFree", heap_free as usize, &HEAP_FREE),
            ("VirtualAlloc", virtual_alloc as usize, &VIRTUAL_ALLOC),
            ("VirtualFree", virtual_free as usize, &VIRTUAL_FREE),
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
        note_heap(heap as usize);
    }
    heap
}

unsafe extern "system" fn heap_alloc(heap: HANDLE, flags: HEAP_FLAGS, bytes: usize) -> *mut c_void {
    let original: HeapAlloc = unsafe { std::mem::transmute(HEAP_ALLOC.load(Ordering::Relaxed)) };
    note_heap(heap as usize);
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
    note_heap(heap as usize);
    unsafe { original(heap, flags, memory, bytes) }
}

unsafe extern "system" fn heap_free(
    heap: HANDLE,
    flags: HEAP_FLAGS,
    memory: *const c_void,
) -> BOOL {
    let original: HeapFree = unsafe { std::mem::transmute(HEAP_FREE.load(Ordering::Relaxed)) };
    note_heap(heap as usize);
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
        note_reservation(allocated as usize, size);
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
        forget_reservation(address as usize);
    }
    freed
}

fn note_heap(heap: usize) {
    if heap == 0 {
        return;
    }
    let Ok(mut tracked) = TRACKED.lock() else {
        return;
    };
    if !tracked.heaps.contains(&heap) {
        tracked.heaps.push(heap);
    }
}

fn note_reservation(base: usize, len: usize) {
    let Ok(mut tracked) = TRACKED.lock() else {
        return;
    };
    // A commit inside an already-recorded reservation is not a new region.
    if tracked
        .reservations
        .iter()
        .any(|region| region.base <= base && base < region.end())
    {
        return;
    }
    tracked.reservations.push(Region { base, len });
}

fn forget_reservation(base: usize) {
    let Ok(mut tracked) = TRACKED.lock() else {
        return;
    };
    tracked.reservations.retain(|region| region.base != base);
}

/// Everything worth saving, as committed readable ranges.
///
/// Walking the heaps takes the heap lock, so this must be called before any
/// thread is suspended, and its result treated as a plan rather than a fact:
/// `snapshot` re-checks each range before touching it.
///
/// # Safety
/// `data` must be the exe's `.data` range.
pub unsafe fn regions(data: Range<usize>) -> Vec<Region> {
    let started = profile::now();
    let mut regions = vec![Region {
        base: data.start,
        len: data.len(),
    }];

    let (heaps, reservations) = match TRACKED.lock() {
        Ok(tracked) => (tracked.heaps.clone(), tracked.reservations.clone()),
        Err(_) => {
            unsafe { profile::record(profile::Phase::Regions, started) };
            return regions;
        }
    };
    for heap in heaps {
        unsafe { collect_heap(heap, &mut regions) };
    }
    for reservation in reservations {
        unsafe { collect_committed(reservation.base..reservation.end(), &mut regions) };
    }
    unsafe { profile::record(profile::Phase::Regions, started) };
    regions
}

unsafe fn collect_heap(heap: usize, out: &mut Vec<Region>) {
    unsafe {
        let heap = heap as HANDLE;
        if HeapLock(heap) == 0 {
            return;
        }
        let mut entry: PROCESS_HEAP_ENTRY = std::mem::zeroed();
        while HeapWalk(heap, &mut entry) != 0 {
            if entry.wFlags & PROCESS_HEAP_REGION as u16 == 0 {
                continue;
            }
            let region = entry.Anonymous.Region;
            let base = entry.lpData as usize;
            let span =
                base..base + region.dwCommittedSize as usize + region.dwUnCommittedSize as usize;
            collect_committed(span, out);
        }
        HeapUnlock(heap);
    }
}

/// Splits `span` into the parts that are committed and readable. Heap regions
/// contain uncommitted holes and no-access guard pages, and reading those would
/// fault rather than return zeroes.
unsafe fn collect_committed(span: Range<usize>, out: &mut Vec<Region>) {
    let mut address = span.start;
    while address < span.end {
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let queried = unsafe {
            VirtualQuery(
                address as *const c_void,
                &mut info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if queried == 0 {
            return;
        }
        let base = info.BaseAddress as usize;
        let next = base + info.RegionSize;
        if info.State == MEM_COMMIT && is_readable(info.Protect) {
            let start = address.max(base);
            let end = next.min(span.end);
            if start < end {
                push_merged(
                    out,
                    Region {
                        base: start,
                        len: end - start,
                    },
                );
            }
        }
        address = next.max(address + 1);
    }
}

pub fn is_readable(protection: u32) -> bool {
    protection & PAGE_GUARD == 0 && protection & PAGE_NOACCESS == 0
}

/// Heap regions and direct reservations can name the same pages; saving them
/// twice would make a restore's write order decide the outcome.
fn push_merged(out: &mut Vec<Region>, region: Region) {
    for existing in out.iter_mut() {
        if region.base >= existing.base && region.end() <= existing.end() {
            return;
        }
        if region.base <= existing.end() && existing.base <= region.end() {
            let base = existing.base.min(region.base);
            let end = existing.end().max(region.end());
            *existing = Region {
                base,
                len: end - base,
            };
            return;
        }
    }
    out.push(region);
}
