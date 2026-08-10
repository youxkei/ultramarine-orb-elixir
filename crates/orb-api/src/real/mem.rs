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

/// The heap handles and the direct reservations orb's import hooks have noticed the game take, which is
/// what [`game_regions`] walks. Any game thread can allocate, so this needs a lock.
static TRACKED: Mutex<Tracked> = Mutex::new(Tracked {
    heaps: Vec::new(),
    reservations: Vec::new(),
});

struct Tracked {
    heaps: Vec<usize>,
    reservations: Vec<(usize, usize)>,
}

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

/// The word swapped with the page writable for exactly the length of the write, and put back whatever it
/// was: a vtable's page is read-only, and a range left writable is one the game can be made to execute
/// out of.
pub unsafe fn replace_word(address: usize, value: usize) -> Option<usize> {
    let previous = unsafe { unprotect(address, size_of::<usize>()) }?;
    let original = unsafe { read::<usize>(address) };
    unsafe {
        write(address, value);
        reprotect(address, size_of::<usize>(), previous);
    }
    Some(original)
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

pub fn note_heap(heap: usize) {
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

pub fn note_reservation(base: usize, len: usize) {
    let Ok(mut tracked) = TRACKED.lock() else {
        return;
    };
    // A commit inside an already-recorded reservation is not a new region.
    if tracked
        .reservations
        .iter()
        .any(|(at, len)| *at <= base && base < at + len)
    {
        return;
    }
    tracked.reservations.push((base, len));
}

pub fn forget_reservation(base: usize) {
    let Ok(mut tracked) = TRACKED.lock() else {
        return;
    };
    tracked.reservations.retain(|(at, _)| *at != base);
}

/// The data range, then the heaps and the reservations the import hooks noticed, as committed readable
/// ranges with no two of them covering the same pages.
///
/// Beside [`private_regions`] and [`process_heap_regions`], which are the same `HeapWalk` and
/// `VirtualQuery` over the same process asked two other questions. The three differ in what they are
/// walking *for*: this one is what a chapter is a copy of, and it covers only what the game itself took.
///
/// Walking a heap takes its lock, so no thread may be suspended when this is called.
pub fn game_regions(data: &std::ops::Range<usize>) -> Vec<(usize, usize)> {
    let mut regions = vec![(data.start, data.len())];

    let (heaps, reservations) = match TRACKED.lock() {
        Ok(tracked) => (tracked.heaps.clone(), tracked.reservations.clone()),
        Err(_) => return regions,
    };
    for heap in heaps {
        collect_heap(heap, &mut regions);
    }
    for (base, len) in reservations {
        collect_committed(base..base + len, &mut regions);
    }
    regions
}

fn collect_heap(heap: usize, out: &mut Vec<(usize, usize)>) {
    let heap = heap as *mut c_void;
    if unsafe { HeapLock(heap) } == 0 {
        return;
    }
    let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
    while unsafe { HeapWalk(heap, &mut entry) } != 0 {
        if entry.wFlags & PROCESS_HEAP_REGION as u16 == 0 {
            continue;
        }
        let region = unsafe { entry.Anonymous.Region };
        let base = entry.lpData as usize;
        let span = base..base + region.dwCommittedSize as usize + region.dwUnCommittedSize as usize;
        collect_committed(span, out);
    }
    unsafe { HeapUnlock(heap) };
}

/// Splits `span` into the parts that are committed and readable. Heap regions contain uncommitted holes
/// and no-access guard pages, and reading those would fault rather than return zeroes.
fn collect_committed(span: std::ops::Range<usize>, out: &mut Vec<(usize, usize)>) {
    let mut address = span.start;
    while address < span.end {
        let Some(info) = query(address) else {
            return;
        };
        let base = info.BaseAddress as usize;
        let next = base + info.RegionSize;
        // This file's own [`is_readable`] and not the looser test the walk arrived with, which was
        // not-guarded-and-not-no-access: that admits an execute-only page, and a page with no read
        // protection on it is one a copy faults on rather than reads.
        if info.State == MEM_COMMIT && is_readable(info.Protect) {
            let start = address.max(base);
            let end = next.min(span.end);
            if start < end {
                push_merged(out, (start, end - start));
            }
        }
        address = next.max(address + 1);
    }
}

/// Heap regions and direct reservations can name the same pages; saving them twice would make a
/// restore's write order decide the outcome.
///
/// Every entry the range touches and not the first of them: one that bridges two entries already apart —
/// a heap region and a reservation with a gap between — would otherwise grow the first across the second
/// and leave the pair overlapping, which is the duplicate this exists to prevent. One pass reaches them
/// all, because no two entries here ever touch each other: this is the only thing that adds one.
///
/// **Below the seam and not above it**, which is where it was and is the one thing about this that could
/// be got wrong quietly: it is a rule about real pages. A laid-out game answers with the objects an
/// e2e test put there, and two of those merged because they abut is one range nothing in that space can
/// read — measured, as 26 of `orb-core`'s own tests failing on a 61440-byte range that is two.
fn push_merged(out: &mut Vec<(usize, usize)>, (base, len): (usize, usize)) {
    let (mut base, mut end) = (base, base + len);
    out.retain(|(at, len)| {
        let touching = base <= at + len && *at <= end;
        if touching {
            base = base.min(*at);
            end = end.max(at + len);
        }
        !touching
    });
    out.push((base, end - base));
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

#[cfg(test)]
mod tests {
    use super::push_merged;

    /// Nothing in the set covers the same pages as anything else, whichever order the walk found
    /// them in — including where what arrives bridges two that were apart.
    #[test]
    fn a_region_bridging_two_entries_leaves_one() {
        for mut out in [
            vec![(0x1000, 0x1000), (0x3000, 0x1000)],
            vec![(0x3000, 0x1000), (0x1000, 0x1000)],
        ] {
            push_merged(&mut out, (0x1800, 0x2000));
            assert_eq!(out, [(0x1000, 0x3000)]);
        }
    }

    /// One already covered adds nothing, one that abuts an entry extends it, and one that
    /// touches nothing stands on its own.
    #[test]
    fn what_is_already_covered_is_not_saved_again() {
        let mut out = vec![(0x1000, 0x3000)];
        push_merged(&mut out, (0x2000, 0x1000));
        assert_eq!(out, [(0x1000, 0x3000)]);

        push_merged(&mut out, (0x4000, 0x1000));
        assert_eq!(out, [(0x1000, 0x4000)]);

        push_merged(&mut out, (0x8000, 0x1000));
        assert_eq!(out, [(0x1000, 0x4000), (0x8000, 0x1000)]);
    }
}
