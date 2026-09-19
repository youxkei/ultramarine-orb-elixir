//! The memory operations against the real address space.

use std::ffi::c_void;

use std::sync::Mutex;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Memory::{
    GetProcessHeap, HeapLock, HeapUnlock, HeapValidate, HeapWalk, MEM_COMMIT, MEM_FREE, MEM_IMAGE,
    MEM_PRIVATE, MEM_RESERVE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ,
    PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS,
    PAGE_PROTECTION_FLAGS, PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY, PROCESS_HEAP_ENTRY,
    VirtualAlloc, VirtualProtect, VirtualQuery,
};
use windows_sys::Win32::System::SystemServices::PROCESS_HEAP_REGION;

use crate::mem::Source;

/// The ranges orb has said are its own, so that [`private_regions`] can leave them out: they hold
/// copies of the game's memory, and a walk that counted them would report orb's own copies as memory
/// something changed behind a snapshot's back.
static OURS: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

/// The ranges the snapshots orb is still holding would put back, one entry per snapshot that names
/// one. What they are for is [`hold_what_a_restore_needs`]: a large block the game hands back to the
/// OS takes the *address space* a chapter needs with it, and whoever is given it next is who a restore
/// would write over.
static NEEDED: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

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

/// Whether `address` holds a pointer to a page a module can be executed out of, which is what a
/// return address on a stack looks like.
///
/// **Executable and not merely inside an image**, which is [`vtable_in_image`]'s question: a stack
/// also carries pointers to a module's own data, and a run of those among the return addresses is a
/// call chain nobody can read. Used by `thread::where_it_is` and nothing above the seam.
pub fn points_at_code(address: usize) -> bool {
    let Some(pointer) = read_committed::<usize>(address) else {
        return false;
    };
    query(pointer).is_some_and(|info| {
        info.State == MEM_COMMIT
            && info.Type == MEM_IMAGE
            && info.Protect
                & (PAGE_EXECUTE
                    | PAGE_EXECUTE_READ
                    | PAGE_EXECUTE_READWRITE
                    | PAGE_EXECUTE_WRITECOPY)
                != 0
    })
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

pub fn orbs_own() -> Vec<(usize, usize)> {
    OURS.lock().map(|ours| ours.clone()).unwrap_or_default()
}

pub fn what_a_restore_needs() -> Vec<(usize, usize)> {
    NEEDED
        .lock()
        .map(|needed| needed.clone())
        .unwrap_or_default()
}

pub fn heaps() -> Vec<(usize, bool)> {
    let mut heaps = vec![unsafe { GetProcessHeap() } as usize];
    if let Ok(tracked) = TRACKED.lock() {
        for heap in &tracked.heaps {
            if !heaps.contains(heap) {
                heaps.push(*heap);
            }
        }
    }
    heaps
        .into_iter()
        .filter(|heap| *heap != 0)
        .map(|heap| {
            let sound = unsafe { HeapValidate(heap as *mut c_void, 0, std::ptr::null()) } != 0;
            (heap, sound)
        })
        .collect()
}

pub fn allocation_at(address: usize) -> Option<usize> {
    let info = query(address)?;
    (info.State != MEM_FREE).then_some(info.AllocationBase as usize)
}

pub fn a_restore_needs(base: usize, len: usize) {
    if let Ok(mut needed) = NEEDED.lock() {
        needed.push((base, len));
    }
}

pub fn a_restore_is_done_with(base: usize, len: usize) {
    if let Ok(mut needed) = NEEDED.lock()
        && let Some(at) = needed.iter().position(|entry| *entry == (base, len))
    {
        // One entry and not every match: snapshots name the same ranges as each other, and what says a
        // range still matters is that one of them is still there.
        needed.remove(at);
    }
}

pub fn hold_what_a_restore_needs(address: usize) -> Option<(usize, usize)> {
    let needed = NEEDED.lock().ok()?;
    if !needed
        .iter()
        .any(|(base, len)| address >= *base && address < base + len)
    {
        return None;
    }
    drop(needed);
    let info = query(address)?;
    if info.State != MEM_FREE {
        return None;
    }
    // The whole free span, and not the part a snapshot named: a reservation is granted in units of the
    // allocation granularity, so asking for less than what is free is asking for a base Windows would
    // have to round outside it.
    let base = info.BaseAddress as usize;
    let reserved = unsafe {
        VirtualAlloc(
            base as *const c_void,
            info.RegionSize,
            MEM_RESERVE,
            PAGE_NOACCESS,
        )
    };
    (!reserved.is_null()).then_some((reserved as usize, info.RegionSize))
}

/// The data range, then the heaps and the reservations the import hooks noticed, as committed readable
/// ranges with no two of them covering the same pages.
///
/// Beside [`private_regions`] and [`process_heap_regions`], which are the same `HeapWalk` and
/// `VirtualQuery` over the same process asked two other questions. The three differ in what they are
/// walking *for*: this one is what a chapter is a copy of, and it covers only what the game itself took.
///
/// Walking a heap takes its lock, so no thread may be suspended when this is called.
pub fn game_regions(data: &std::ops::Range<usize>) -> Vec<(usize, usize, Source)> {
    let mut regions = vec![(data.start, data.len(), Source::Data)];

    let (heaps, reservations) = match TRACKED.lock() {
        Ok(tracked) => (tracked.heaps.clone(), tracked.reservations.clone()),
        Err(_) => return regions,
    };
    // Counted from one in the order the hooks noticed them, which is what makes the name mean the same
    // pages in the next launch: `TRACKED` is only ever pushed to, so the `n`th heap of one launch is the
    // `n`th of the next wherever the OS put its handle. For a game whose allocator is the UCRT's there is
    // exactly one, `orb::memtrack::install_process_heap`'s own `HeapCreate`, so `heap1.N` names a region
    // of the game's heap and nothing about which heap has to be worked out to bisect it.
    for (which, heap) in heaps.into_iter().enumerate() {
        collect_heap(heap, which + 1, &mut regions);
    }
    for (which, (base, len)) in reservations.into_iter().enumerate() {
        collect_committed(
            base..base + len,
            Source::Reservation(which + 1),
            &mut regions,
        );
    }
    regions
}

/// **The regions counted as the walk hands them over**, which is the ordinal a bisect names one by: the
/// heap manager's own list is appended to as the heap grows rather than reordered, so the `n`th region of
/// one launch is the `n`th of the next — measured, as five launches of th06nc printing the same seven
/// sizes in the same order.
fn collect_heap(heap: usize, which: usize, out: &mut Vec<(usize, usize, Source)>) {
    let heap = heap as *mut c_void;
    if unsafe { HeapLock(heap) } == 0 {
        return;
    }
    let mut entry: PROCESS_HEAP_ENTRY = unsafe { std::mem::zeroed() };
    let mut regions = 0;
    while unsafe { HeapWalk(heap, &mut entry) } != 0 {
        if entry.wFlags & PROCESS_HEAP_REGION as u16 == 0 {
            continue;
        }
        regions += 1;
        let region = unsafe { entry.Anonymous.Region };
        let base = entry.lpData as usize;
        let span = base..base + region.dwCommittedSize as usize + region.dwUnCommittedSize as usize;
        collect_committed(
            span,
            Source::Heap {
                which,
                region: regions,
            },
            out,
        );
    }
    unsafe { HeapUnlock(heap) };
}

/// Splits `span` into the parts that are committed and readable. Heap regions contain uncommitted holes
/// and no-access guard pages, and reading those would fault rather than return zeroes.
fn collect_committed(
    span: std::ops::Range<usize>,
    from: Source,
    out: &mut Vec<(usize, usize, Source)>,
) {
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
                push_merged(out, (start, end - start), from);
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
/// **The merged entry is named after whichever of them starts lowest**, and where two start together
/// after the one already in the list. That is a property of the pages and not of the order the walk
/// happened to find them in, which the name has to be: the list is unordered, so a rule reading "the one
/// that was already here" would answer differently for the same three ranges arriving the other way
/// round. The tie goes to the standing entry because the walk goes `.data`, then the heaps as they were
/// noticed, then the reservations — so it is the earlier source, and for the pair the merging exists for
/// (a range the game reserved from the OS that its heap manager has since taken over) the earlier source
/// is the heap that hands those pages out.
fn push_merged(out: &mut Vec<(usize, usize, Source)>, (base, len): (usize, usize), from: Source) {
    let (mut base, mut end) = (base, base + len);
    let mut lowest = (base, from);
    out.retain(|(at, len, theirs)| {
        let touching = base <= at + len && *at <= end;
        if touching {
            if *at <= lowest.0 {
                lowest = (*at, *theirs);
            }
            base = base.min(*at);
            end = end.max(at + len);
        }
        !touching
    });
    out.push((base, end - base, lowest.1));
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
    use super::{Source, push_merged};

    /// The first region of the game's first heap, which is what the walk's own entries are named.
    const A_REGION: Source = Source::Heap {
        which: 1,
        region: 1,
    };

    /// And the one after it, to say that two regions of one heap are two names.
    const THE_NEXT_REGION: Source = Source::Heap {
        which: 1,
        region: 2,
    };

    /// Nothing in the set covers the same pages as anything else, whichever order the walk found
    /// them in — including where what arrives bridges two that were apart.
    #[test]
    fn a_region_bridging_two_entries_leaves_one() {
        for mut out in [
            vec![(0x1000, 0x1000, A_REGION), (0x3000, 0x1000, A_REGION)],
            vec![(0x3000, 0x1000, A_REGION), (0x1000, 0x1000, A_REGION)],
        ] {
            push_merged(&mut out, (0x1800, 0x2000), A_REGION);
            assert_eq!(out, [(0x1000, 0x3000, A_REGION)]);
        }
    }

    /// One already covered adds nothing, one that abuts an entry extends it, and one that
    /// touches nothing stands on its own.
    #[test]
    fn what_is_already_covered_is_not_saved_again() {
        let mut out = vec![(0x1000, 0x3000, A_REGION)];
        push_merged(&mut out, (0x2000, 0x1000), A_REGION);
        assert_eq!(out, [(0x1000, 0x3000, A_REGION)]);

        push_merged(&mut out, (0x4000, 0x1000), A_REGION);
        assert_eq!(out, [(0x1000, 0x4000, A_REGION)]);

        push_merged(&mut out, (0x8000, 0x1000), A_REGION);
        assert_eq!(
            out,
            [(0x1000, 0x4000, A_REGION), (0x8000, 0x1000, A_REGION)],
        );
    }

    /// A merged range is named after whichever of them starts lowest, and after the standing entry
    /// where they start together — so what a bisect is given for a reservation the heap manager has
    /// taken over is the heap's own region, whichever order the walk met the two in.
    #[test]
    fn a_merge_is_named_after_what_starts_lowest() {
        let mut out = vec![(0x2000, 0x1000, A_REGION)];
        push_merged(&mut out, (0x1000, 0x1000), Source::Reservation(1));
        assert_eq!(out, [(0x1000, 0x2000, Source::Reservation(1))]);

        let mut out = vec![(0x1000, 0x1000, A_REGION)];
        push_merged(&mut out, (0x2000, 0x1000), Source::Reservation(1));
        assert_eq!(out, [(0x1000, 0x2000, A_REGION)]);

        let mut out = vec![(0x1000, 0x1000, THE_NEXT_REGION)];
        push_merged(&mut out, (0x1000, 0x2000), Source::Reservation(1));
        assert_eq!(out, [(0x1000, 0x2000, THE_NEXT_REGION)]);
    }
}
