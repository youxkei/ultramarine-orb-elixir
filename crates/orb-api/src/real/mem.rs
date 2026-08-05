//! The memory operations against the real address space.

use std::ffi::c_void;

use std::sync::Mutex;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Memory::{
    GetProcessHeap, HeapLock, HeapUnlock, HeapWalk, MEM_COMMIT, MEM_IMAGE, MEM_PRIVATE,
    MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE,
    PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS, PAGE_PROTECTION_FLAGS, PAGE_READONLY,
    PAGE_READWRITE, PAGE_WRITECOPY, PROCESS_HEAP_ENTRY, VirtualAlloc, VirtualProtect, VirtualQuery,
};
use windows_sys::Win32::System::SystemServices::PROCESS_HEAP_REGION;

/// The ranges orb has said are its own, so that [`private_regions`] can leave them out: they hold
/// copies of the game's memory, and a walk that counted them would report orb's own copies as memory
/// something changed behind a snapshot's back.
static OURS: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

pub unsafe fn read<T: Copy>(address: usize) -> T {
    unsafe { (address as *const T).read_volatile() }
}

pub unsafe fn write<T: Copy>(address: usize, value: T) {
    unsafe { (address as *mut T).write_volatile(value) }
}

pub unsafe fn read_bytes(address: usize, len: usize) -> Vec<u8> {
    unsafe { std::slice::from_raw_parts(address as *const u8, len) }.to_vec()
}

pub unsafe fn write_bytes(address: usize, source: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(source.as_ptr(), address as *mut u8, source.len()) };
}

pub unsafe fn fill_bytes(address: usize, byte: u8, len: usize) {
    unsafe { std::ptr::write_bytes(address as *mut u8, byte, len) };
}

pub unsafe fn copy_out(address: usize, to: *mut u8, len: usize) {
    unsafe { std::ptr::copy_nonoverlapping(address as *const u8, to, len) };
}

pub unsafe fn copy_in(address: usize, from: *const u8, len: usize) {
    unsafe { std::ptr::copy_nonoverlapping(from, address as *mut u8, len) };
}

/// Walked run by run rather than tested at the range's first page. `VirtualQuery` describes only
/// the run of pages that begins where it is asked, so a range whose head is still committed
/// answers "committed" while a hole further in has been handed back to the OS — which the game
/// freeing a few megabytes does. That hole was a fault inside the copy's own `memcpy`, and
/// `VirtualProtect` over the whole range had quietly failed for the same reason.
pub unsafe fn commit(address: usize, len: usize) -> bool {
    let end = address + len;
    let mut at = address;
    while at < end {
        let Some(info) = query(at) else {
            return false;
        };
        let run = (info.BaseAddress as usize + info.RegionSize).min(end);
        if run <= at {
            return false;
        }
        if info.State != MEM_COMMIT {
            let span = run - at;
            let committed =
                unsafe { VirtualAlloc(at as *const c_void, span, MEM_COMMIT, PAGE_READWRITE) };
            if committed.is_null() {
                let reserved = unsafe {
                    VirtualAlloc(
                        at as *const c_void,
                        span,
                        MEM_COMMIT | MEM_RESERVE,
                        PAGE_READWRITE,
                    )
                };
                if reserved.is_null() {
                    return false;
                }
            }
        }
        at = run;
    }
    true
}

pub unsafe fn unprotect(address: usize, len: usize) -> Option<u32> {
    let mut previous: PAGE_PROTECTION_FLAGS = 0;
    let changed =
        unsafe { VirtualProtect(address as *const c_void, len, PAGE_READWRITE, &mut previous) }
            != FALSE;
    changed.then_some(previous)
}

pub unsafe fn reprotect(address: usize, len: usize, previous: u32) {
    let mut discarded: PAGE_PROTECTION_FLAGS = 0;
    unsafe {
        VirtualProtect(address as *const c_void, len, previous, &mut discarded);
    }
}

pub fn vtable_in_image(address: usize) -> bool {
    let Some(vtable) = read_committed::<usize>(address) else {
        return false;
    };
    query(vtable).is_some_and(|info| info.State == MEM_COMMIT && info.Type == MEM_IMAGE)
}

/// Alignment is refused here as well as in the facade, and not redundantly: the facade checks it
/// so that the byte-level seam — which is handed a length and knows nothing of `T` — cannot answer
/// a read the real one would have refused, and this checks it so that `vtable_in_image` below,
/// which reaches this directly, is held to the same rule.
pub fn read_committed<T: Copy>(address: usize) -> Option<T> {
    if !address.is_multiple_of(align_of::<T>()) {
        return None;
    }
    let info = query(address)?;
    let end = info.BaseAddress as usize + info.RegionSize;
    if info.State != MEM_COMMIT
        || info.Protect & (PAGE_NOACCESS | PAGE_GUARD) != 0
        || address + size_of::<T>() > end
    {
        return None;
    }
    Some(unsafe { read(address) })
}

fn query(address: usize) -> Option<MEMORY_BASIC_INFORMATION> {
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let queried = unsafe {
        VirtualQuery(
            address as *const c_void,
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    (queried != 0).then_some(info)
}

pub fn keep_out_of_private_regions(base: usize, len: usize) {
    if let Ok(mut ours) = OURS.lock() {
        ours.push((base, len));
    }
}

pub fn count_private_region_again(base: usize) {
    if let Ok(mut ours) = OURS.lock() {
        ours.retain(|(at, _)| *at != base);
    }
}

pub fn private_regions() -> Vec<(usize, usize)> {
    let ours = OURS.lock().map(|ours| ours.clone()).unwrap_or_default();
    let mut regions = Vec::new();
    let mut address = 0usize;
    while let Some(info) = query(address) {
        let base = info.BaseAddress as usize;
        let next = base + info.RegionSize;
        let interesting = info.State == MEM_COMMIT
            && info.Type == MEM_PRIVATE
            && is_readable(info.Protect)
            && !ours
                .iter()
                .any(|(at, len)| base < at + len && *at < base + info.RegionSize);
        if interesting {
            regions.push((base, info.RegionSize));
        }
        if next <= address {
            break;
        }
        address = next;
    }
    regions
}

/// A page that can be read at all. `PAGE_GUARD` and `PAGE_NOACCESS` cannot, and reading one is how a
/// walk of the whole address space takes the process down instead of describing it.
fn is_readable(protect: PAGE_PROTECTION_FLAGS) -> bool {
    if protect & (PAGE_GUARD | PAGE_NOACCESS) != 0 {
        return false;
    }
    protect
        & (PAGE_READONLY
            | PAGE_READWRITE
            | PAGE_WRITECOPY
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_EXECUTE_WRITECOPY)
        != 0
}

pub fn process_heap_regions() -> Vec<(usize, usize)> {
    let heap = unsafe { GetProcessHeap() };
    let mut regions = Vec::new();
    if heap.is_null() || unsafe { HeapLock(heap) } == 0 {
        return regions;
    }
    let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
    while unsafe { HeapWalk(heap, &mut entry) } != 0 {
        if entry.wFlags & PROCESS_HEAP_REGION as u16 == 0 {
            continue;
        }
        let region = unsafe { entry.Anonymous.Region };
        regions.push((
            entry.lpData as usize,
            region.dwCommittedSize as usize + region.dwUnCommittedSize as usize,
        ));
    }
    unsafe { HeapUnlock(heap) };
    regions
}

pub fn read_committed_bytes(address: usize, len: usize) -> Option<Vec<u8>> {
    let info = query(address)?;
    let end = info.BaseAddress as usize + info.RegionSize;
    if info.State != MEM_COMMIT
        || info.Protect & (PAGE_NOACCESS | PAGE_GUARD) != 0
        || address + len > end
    {
        return None;
    }
    Some(unsafe { read_bytes(address, len) })
}
