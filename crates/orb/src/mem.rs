//! Raw reads and writes against the game's own memory.
//!
//! Volatile throughout: the compiler has no idea the game writes these
//! locations every frame, so a plain read is free to be hoisted or reused.

use std::ffi::c_void;

use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_IMAGE, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery,
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
    let Some(vtable) = read_committed::<usize>(address) else {
        return false;
    };
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let queried = unsafe {
        VirtualQuery(
            vtable as *const c_void,
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    queried != 0 && info.State == MEM_COMMIT && info.Type == MEM_IMAGE
}

/// Reads only where the whole value can be read at all, for chasing pointers out of the game's
/// structures that may not be valid yet or any more.
///
/// Committed is not enough on its own, and each of the three that are missing from it is a
/// process that dies rather than a read that misses:
///
/// - **Alignment.** Every one of these addresses is a field of a structure the game built, so an
///   address that is not aligned for what is being read is not one of them. Reading it anyway is
///   undefined behaviour, which a build with the checks on turns into a non-unwinding panic: one
///   test run in twenty aborted with `STATUS_STACK_BUFFER_OVERRUN` after every test in it had
///   passed, and the run that kept its output said `read_volatile requires that the pointer
///   argument is aligned`, out of the chase after a track's identity.
/// - **`PAGE_NOACCESS`**, which faults.
/// - **`PAGE_GUARD`**, which is every thread's stack guard. Reading one raises
///   `STATUS_GUARD_PAGE_VIOLATION` — fatal to a process not expecting it — and where that is
///   caught the page comes back without its guard, so the thread owning that stack stops growing
///   it and dies of an overflow later and somewhere else.
pub fn read_committed<T: Copy>(address: usize) -> Option<T> {
    if !address.is_multiple_of(align_of::<T>()) {
        return None;
    }
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let queried = unsafe {
        VirtualQuery(
            address as *const c_void,
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    let end = info.BaseAddress as usize + info.RegionSize;
    if queried == 0
        || info.State != MEM_COMMIT
        || info.Protect & (PAGE_NOACCESS | PAGE_GUARD) != 0
        || address + size_of::<T>() > end
    {
        return None;
    }
    Some(unsafe { read(address) })
}

#[cfg(test)]
mod tests {
    use super::read_committed;

    /// Memory this process owns, so the answer is about the rule and not about what happens to
    /// be mapped: aligned and committed is read, one byte along is not.
    #[test]
    fn an_unaligned_address_is_not_read() {
        let value: u64 = 0x0123_4567_89ab_cdef;
        let address = &raw const value as usize;
        assert_eq!(read_committed::<u64>(address), Some(value));
        assert_eq!(read_committed::<u64>(address + 1), None);
    }

    /// The bottom of the address space is never mapped, and asking about it is how every pointer
    /// out of a structure the game has not built yet comes back.
    #[test]
    fn an_address_that_is_not_mapped_is_not_read() {
        assert_eq!(read_committed::<u32>(0), None);
        assert_eq!(read_committed::<u32>(0x1000), None);
    }
}
