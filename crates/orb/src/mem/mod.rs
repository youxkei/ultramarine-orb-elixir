//! Raw reads and writes against the game's own memory.
//!
//! Volatile throughout: the compiler has no idea the game writes these
//! locations every frame, so a plain read is free to be hoisted or reused.
//!
//! Every one of them goes through this module, which is what makes a test with no game in it
//! possible: a [`space`] laid out by hand answers the same reads at the same addresses, so the
//! whole of [`Th06`](crate::game::th06::Th06) — the offsets, the structure walks, the
//! patching — runs against it unchanged. The alternative was a second implementation of
//! [`Game`](crate::game::Game) for tests, which would have left those two thousand lines the
//! only part of orb nothing exercised.
//!
//! In a build that is not a test the space does not exist and none of these functions branch.

use std::ffi::c_void;

use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_IMAGE, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery,
};

mod real;
#[cfg(test)]
pub mod space;

pub unsafe fn read<T: Copy>(address: usize) -> T {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.read(address);
    }
    unsafe { (address as *const T).read_volatile() }
}

pub unsafe fn write<T: Copy>(address: usize, value: T) {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.write(address, value);
    }
    unsafe { (address as *mut T).write_volatile(value) }
}

/// A range of the game's memory as bytes, for what orb holds across something that would
/// replace it without having any use for the shape of it.
pub unsafe fn read_bytes(address: usize, len: usize) -> Vec<u8> {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.read_bytes(address, len);
    }
    unsafe { std::slice::from_raw_parts(address as *const u8, len) }.to_vec()
}

/// # Safety
/// `source` must be as long as the range it is being put back into.
pub unsafe fn write_bytes(address: usize, source: &[u8]) {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.write_bytes(address, source);
    }
    unsafe { std::ptr::copy_nonoverlapping(source.as_ptr(), address as *mut u8, source.len()) };
}

pub unsafe fn fill_bytes(address: usize, byte: u8, len: usize) {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.fill_bytes(address, byte, len);
    }
    unsafe { std::ptr::write_bytes(address as *mut u8, byte, len) };
}

/// Copies a range of the game's memory into a buffer of orb's own, which is what a snapshot is
/// made of. The destination stays a real pointer under a space: the buffer belongs to orb and
/// is real memory in a test as much as in a game.
///
/// # Safety
/// `to` must be writable for `len` bytes.
pub unsafe fn copy_out(address: usize, to: *mut u8, len: usize) {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        let bytes = space.read_bytes(address, len);
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), to, len) };
        return;
    }
    unsafe { std::ptr::copy_nonoverlapping(address as *const u8, to, len) };
}

/// Copies a buffer of orb's own back over the game's memory, which is what a restore does.
///
/// # Safety
/// `from` must be readable for `len` bytes.
pub unsafe fn copy_in(address: usize, from: *const u8, len: usize) {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        let bytes = unsafe { std::slice::from_raw_parts(from, len) };
        space.write_bytes(address, bytes);
        return;
    }
    unsafe { std::ptr::copy_nonoverlapping(from, address as *mut u8, len) };
}

/// Puts back any page of `address..address + len` that has gone since a snapshot, so the copy
/// has somewhere to write, and says whether the whole of it is now there.
pub unsafe fn commit(address: usize, len: usize) -> bool {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.commit(address, len);
    }
    unsafe { real::commit(address, len) }
}

/// Makes a range writable and answers what its protection was, or `None` where it could not be
/// changed. A space has no protection to change, so nothing is asked of it and nothing has to be
/// put back.
pub unsafe fn unprotect(address: usize, len: usize) -> Option<u32> {
    #[cfg(test)]
    if space::installed().is_some() {
        return None;
    }
    unsafe { real::unprotect(address, len) }
}

/// Puts back what [`unprotect`] answered with.
pub unsafe fn reprotect(address: usize, len: usize, previous: u32) {
    #[cfg(test)]
    if space::installed().is_some() {
        return;
    }
    unsafe { real::reprotect(address, len, previous) };
}

/// Whether `address` holds a pointer to a mapped image, which is what a live COM
/// object's vtable pointer looks like.
///
/// The game's allocator does not scrub freed blocks, so a stale pointer to a
/// released object still reads back as its old contents. Following one into
/// DirectSound crashes, and this is the cheap check that catches most of them.
pub fn vtable_in_image(address: usize) -> bool {
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.vtable_in_image(address);
    }
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
    #[cfg(test)]
    if let Some(space) = space::installed() {
        return space.read_committed(address);
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
