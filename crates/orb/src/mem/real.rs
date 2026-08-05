//! The page operations against the real address space, behind the seam in the parent module.
//!
//! Here rather than beside the snapshot that asks for them, because a test asks for the same
//! things of a [`space`](super::space) and the two answers belong side by side.

use std::ffi::c_void;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
    VirtualAlloc, VirtualProtect, VirtualQuery,
};

/// Walked run by run rather than tested at the range's first page. `VirtualQuery` describes only
/// the run of pages that begins where it is asked, so a range whose head is still committed
/// answers "committed" while a hole further in has been handed back to the OS — which the game
/// freeing a few megabytes does. That hole was a fault inside the copy's own `memcpy`, and
/// `VirtualProtect` over the whole range had quietly failed for the same reason.
pub unsafe fn commit(address: usize, len: usize) -> bool {
    let end = address + len;
    let mut at = address;
    while at < end {
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        let queried = unsafe {
            VirtualQuery(
                at as *const c_void,
                &mut info,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        if queried == 0 {
            return false;
        }
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
