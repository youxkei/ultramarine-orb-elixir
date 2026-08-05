//! Raw reads and writes against the game's own memory.
//!
//! Volatile throughout: the compiler has no idea the game writes these locations every frame,
//! so a plain read is free to be hoisted or reused.
//!
//! Every one of them goes through this module, which is what makes a test with no game in it
//! possible: a simulated Windows laid out by hand answers the same reads at the same addresses,
//! so the whole of `Th06` — the offsets, the structure walks, the patching — runs against it
//! unchanged. The alternative was a second implementation of `Game` for tests, which would
//! have left those two thousand lines the only part of orb nothing exercised.
//!
//! In a build without the `sim` feature the install point does not exist and none of these
//! functions branch.
//!
//! # Safety
//!
//! Every function here that is `unsafe` is so for one reason, and the contract is the same for all
//! of them: with no simulated Windows installed the address is dereferenced as it stands, so it
//! must be an address the game really keeps `T` at, aligned for `T`, and committed and readable —
//! or writable, where the function writes. An address that is none of those does not fail: it
//! faults, and the game goes down with orb inside it. [`read_committed`] is the one that asks
//! first, and it is what every chase after a pointer out of the game's own structures goes
//! through.
//!
//! `T` must also be a type all of whose bit patterns are values: an integer, a float, a pointer, or
//! an array of those. That is what the game's structures are made of, and it is what lets the same
//! call be answered out of bytes when a simulated Windows is in front.

/// Reinterprets what a seam read came back with.
///
/// Sound for what this seam carries, which is what the game's structures are made of: integers,
/// floats, pointers, and arrays of those. Nothing with a destructor, nothing holding a
/// reference, nothing whose bit patterns are not all values of the type.
#[cfg(feature = "sim")]
fn from_bytes<T: Copy>(bytes: &[u8]) -> T {
    assert_eq!(bytes.len(), size_of::<T>());
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), value.as_mut_ptr().cast::<u8>(), bytes.len());
        value.assume_init()
    }
}

#[cfg(feature = "sim")]
fn as_bytes<T: Copy>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) }
}

/// # Safety
/// As the module's, for a read of `T`.
pub unsafe fn read<T: Copy>(address: usize) -> T {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return from_bytes(&win.read_bytes(address, size_of::<T>()));
    }
    unsafe { host::read(address) }
}

/// # Safety
/// As the module's, for a write of `T`.
pub unsafe fn write<T: Copy>(address: usize, value: T) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.write_bytes(address, as_bytes(&value));
    }
    unsafe { host::write(address, value) }
}

/// A range of the game's memory as bytes, for what orb holds across something that would
/// replace it without having any use for the shape of it.
///
/// # Safety
/// As the module's, over the whole of `address..address + len`.
pub unsafe fn read_bytes(address: usize, len: usize) -> Vec<u8> {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.read_bytes(address, len);
    }
    unsafe { host::read_bytes(address, len) }
}

/// # Safety
/// `source` must be as long as the range it is being put back into.
pub unsafe fn write_bytes(address: usize, source: &[u8]) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.write_bytes(address, source);
    }
    unsafe { host::write_bytes(address, source) }
}

/// # Safety
/// As the module's, over the whole of `address..address + len`, which must be writable.
pub unsafe fn fill_bytes(address: usize, byte: u8, len: usize) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.fill_bytes(address, byte, len);
    }
    unsafe { host::fill_bytes(address, byte, len) }
}

/// Copies a range of the game's memory into a buffer of orb's own, which is what a snapshot is
/// made of. The destination stays a real pointer under a simulated Windows: the buffer belongs
/// to orb and is real memory in a test as much as in a game.
///
/// # Safety
/// `to` must be writable for `len` bytes.
pub unsafe fn copy_out(address: usize, to: *mut u8, len: usize) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        let bytes = win.read_bytes(address, len);
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), to, len) };
        return;
    }
    unsafe { host::copy_out(address, to, len) }
}

/// Copies a buffer of orb's own back over the game's memory, which is what a restore does.
///
/// # Safety
/// `from` must be readable for `len` bytes.
pub unsafe fn copy_in(address: usize, from: *const u8, len: usize) {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        let bytes = unsafe { std::slice::from_raw_parts(from, len) };
        win.write_bytes(address, bytes);
        return;
    }
    unsafe { host::copy_in(address, from, len) }
}

/// Puts back any page of `address..address + len` that has gone since a snapshot, so the copy
/// has somewhere to write, and says whether the whole of it is now there.
///
/// # Safety
/// The range must be one the game owns. Committing pages in the middle of somebody else's
/// reservation is how a process ends up with the wrong thing at an address it trusted.
pub unsafe fn commit(address: usize, len: usize) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.commit(address, len);
    }
    unsafe { host::commit(address, len) }
}

/// Makes a range writable and answers what its protection was, or `None` where it could not be
/// changed. A simulated Windows has no protection to change, so nothing is asked of it and
/// nothing has to be put back.
///
/// # Safety
/// The range must be one the game owns, and what this answers with must be handed back to
/// [`reprotect`]: a range left writable is one the game can be made to execute out of.
pub unsafe fn unprotect(address: usize, len: usize) -> Option<u32> {
    #[cfg(feature = "sim")]
    if crate::installed().is_some() {
        return None;
    }
    unsafe { host::unprotect(address, len) }
}

/// Puts back what [`unprotect`] answered with.
///
/// # Safety
/// `previous` must be what [`unprotect`] answered for this same range, and nothing may still be
/// relying on the range being writable.
pub unsafe fn reprotect(address: usize, len: usize, previous: u32) {
    #[cfg(feature = "sim")]
    if crate::installed().is_some() {
        return;
    }
    unsafe { host::reprotect(address, len, previous) }
}

/// Whether `address` holds a pointer to a mapped image, which is what a live COM object's
/// vtable pointer looks like.
///
/// The game's allocator does not scrub freed blocks, so a stale pointer to a released object
/// still reads back as its old contents. Following one into DirectSound crashes, and this is
/// the cheap check that catches most of them.
pub fn vtable_in_image(address: usize) -> bool {
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win.vtable_in_image(address);
    }
    host::vtable_in_image(address)
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
///
/// The alignment rule is answered here rather than behind the seam because it is a property of
/// the type being read and not of the address space, so a simulated Windows cannot pass a test
/// on a read the real one would have refused.
pub fn read_committed<T: Copy>(address: usize) -> Option<T> {
    if !address.is_multiple_of(align_of::<T>()) {
        return None;
    }
    #[cfg(feature = "sim")]
    if let Some(win) = crate::installed() {
        return win
            .read_committed_bytes(address, size_of::<T>())
            .map(|bytes| from_bytes(&bytes));
    }
    host::read_committed(address)
}

#[cfg(windows)]
use crate::real::mem as host;

/// What the rules are against the address space that is really there.
///
/// Only on Windows, and deliberately so: these ask what `VirtualQuery` says about this process's
/// own pages, so a host without one has nothing for them to be about. They are the half of the
/// memory seam a simulated Windows cannot answer for — see `orb-sim` for the other half.
#[cfg(all(test, windows))]
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

/// What the memory seam does on a host that has no Windows behind it.
///
/// Every one of these is reached only by a test that has not installed a simulated Windows.
/// There is nothing to read on this host and no third answer to give — answering zero would
/// let a test pass on a read that never happened.
#[cfg(not(windows))]
mod host {
    use crate::no_windows;

    pub unsafe fn read<T: Copy>(_address: usize) -> T {
        no_windows("mem::read")
    }
    pub unsafe fn write<T: Copy>(_address: usize, _value: T) {
        no_windows("mem::write")
    }
    pub unsafe fn read_bytes(_address: usize, _len: usize) -> Vec<u8> {
        no_windows("mem::read_bytes")
    }
    pub unsafe fn write_bytes(_address: usize, _source: &[u8]) {
        no_windows("mem::write_bytes")
    }
    pub unsafe fn fill_bytes(_address: usize, _byte: u8, _len: usize) {
        no_windows("mem::fill_bytes")
    }
    pub unsafe fn copy_out(_address: usize, _to: *mut u8, _len: usize) {
        no_windows("mem::copy_out")
    }
    pub unsafe fn copy_in(_address: usize, _from: *const u8, _len: usize) {
        no_windows("mem::copy_in")
    }
    pub unsafe fn commit(_address: usize, _len: usize) -> bool {
        no_windows("mem::commit")
    }
    pub unsafe fn unprotect(_address: usize, _len: usize) -> Option<u32> {
        no_windows("mem::unprotect")
    }
    pub unsafe fn reprotect(_address: usize, _len: usize, _previous: u32) {
        no_windows("mem::reprotect")
    }
    pub fn vtable_in_image(_address: usize) -> bool {
        no_windows("mem::vtable_in_image")
    }
    pub fn read_committed<T: Copy>(_address: usize) -> Option<T> {
        no_windows("mem::read_committed")
    }
}
