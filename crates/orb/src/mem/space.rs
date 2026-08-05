//! An address space a test puts in front of the real one, so that the game's memory can be
//! read and written with no game there to own it.
//!
//! The addresses [`Th06`](crate::game::th06::Th06) reads are absolute and belong to a
//! process orb is not running in during a test. Two ways to make them readable, and this is
//! the second:
//!
//! - `VirtualAlloc` them in the test process at the bases the game uses. Ties every test to a
//!   host where 0x0069bca0 and its neighbours happen to be free — the test binary's own image
//!   is one bad link away from sitting there — and to a Windows that has `VirtualAlloc` at
//!   all, which is what keeps the suite off a Linux runner.
//! - Answer the reads from a map. The same bytes at the same addresses, with none of that,
//!   and the fields a structure has not been built with yet read as zero the way a freshly
//!   committed page does.
//!
//! Installed per thread. The test harness runs tests side by side in one process, so a space
//! in a static would be two tests writing each other's game; and it hands its threads out
//! again, so the installation has to come back off at the end of a test rather than be left
//! for whatever runs there next — which is what [`Installed`] is for.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// What a region is, as far as anything reading it can tell.
///
/// [`Space::vtable_in_image`] is the one question orb asks that tells them apart: a live COM
/// object's vtable pointer lands in a mapped image, and the stale pointer left in a block the
/// game's allocator did not scrub does not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A mapped executable — the game's own image, or a DLL it loaded.
    Image,
    /// Anything the game allocated.
    Private,
}

/// How a region answers a read, short of holding bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Access {
    /// Committed and readable.
    Open,
    /// Reserved and never committed, which is every pointer into a structure the game has not
    /// built yet.
    Uncommitted,
    /// `PAGE_GUARD`, which is every thread's stack guard: reading one raises
    /// `STATUS_GUARD_PAGE_VIOLATION` in a real process and takes the thread's stack with it.
    Guarded,
}

struct Region {
    base: usize,
    bytes: Vec<u8>,
    kind: Kind,
    access: Access,
}

impl Region {
    fn end(&self) -> usize {
        self.base + self.bytes.len()
    }

    fn holds(&self, address: usize, len: usize) -> bool {
        address >= self.base && address.saturating_add(len) <= self.end()
    }
}

/// The regions a test has laid out, and the bytes in them.
pub struct Space {
    regions: Mutex<Vec<Region>>,
}

impl Default for Space {
    fn default() -> Self {
        Self::new()
    }
}

impl Space {
    pub fn new() -> Self {
        Self {
            regions: Mutex::new(Vec::new()),
        }
    }

    /// Commits `len` zeroed bytes at `base`, as the game's static data and its allocations
    /// both read.
    pub fn map(&self, base: usize, len: usize, kind: Kind) {
        self.add(base, len, kind, Access::Open);
    }

    /// Reserves without committing, for the addresses a structure that has not been built yet
    /// is reached through.
    pub fn reserve(&self, base: usize, len: usize) {
        self.add(base, len, Kind::Private, Access::Uncommitted);
    }

    /// A stack guard, which is the page `read_committed` refuses on top of refusing what is
    /// not committed.
    pub fn guard(&self, base: usize, len: usize) {
        self.add(base, len, Kind::Private, Access::Guarded);
    }

    fn add(&self, base: usize, len: usize, kind: Kind, access: Access) {
        let mut regions = self.regions.lock().unwrap();
        assert!(
            !regions
                .iter()
                .any(|region| base < region.end() && region.base < base + len),
            "{base:#010x}..{:#010x} overlaps a region already laid out",
            base + len
        );
        regions.push(Region {
            base,
            bytes: vec![0; len],
            kind,
            access,
        });
    }

    /// Drops a region, for the frees a test needs to be seen: a region that has gone away
    /// between a snapshot and the restore of it is the case a restore has to survive.
    pub fn unmap(&self, base: usize) {
        let mut regions = self.regions.lock().unwrap();
        let at = regions
            .iter()
            .position(|region| region.base == base)
            .unwrap_or_else(|| panic!("no region is based at {base:#010x}"));
        regions.remove(at);
    }

    /// What a snapshot of this space covers: the data range first, as a real game's does, and then
    /// every other region the game allocated. A region of the game's code is left out — a chapter
    /// is a copy of its state, not of itself — and so is anything overlapping the data range, which
    /// a snapshot saving the same bytes twice would be.
    pub fn game_regions(&self, data: &std::ops::Range<usize>) -> Vec<(usize, usize)> {
        let regions = self.regions.lock().unwrap();
        std::iter::once((data.start, data.len()))
            .chain(
                regions
                    .iter()
                    .filter(|region| region.access == Access::Open && region.kind == Kind::Private)
                    .filter(|region| data.start >= region.end() || region.base >= data.end)
                    .map(|region| (region.base, region.bytes.len())),
            )
            .collect()
    }

    /// The regions as bases and lengths, for a test that wants to hand orb what to snapshot.
    pub fn mapped(&self) -> Vec<(usize, usize)> {
        let regions = self.regions.lock().unwrap();
        regions
            .iter()
            .filter(|region| region.access == Access::Open)
            .map(|region| (region.base, region.bytes.len()))
            .collect()
    }

    /// # Panics
    /// Where no region holds the whole of `address..address + size_of::<T>()`, or the one that
    /// does is not readable. A real read there takes the process down, so a test reaching one
    /// has a wrong address in it and saying which is the whole of the help there is to give.
    pub fn read<T: Copy>(&self, address: usize) -> T {
        let bytes = self.bytes(address, size_of::<T>());
        // Sound for what this seam carries, which is what the game's structures are made of:
        // integers, floats, pointers, and arrays of those. Nothing with a destructor, nothing
        // holding a reference, nothing whose bit patterns are not all values of the type.
        let mut value = std::mem::MaybeUninit::<T>::uninit();
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                value.as_mut_ptr() as *mut u8,
                size_of::<T>(),
            );
            value.assume_init()
        }
    }

    /// # Panics
    /// As [`read`](Self::read).
    pub fn write<T: Copy>(&self, address: usize, value: T) {
        let bytes =
            unsafe { std::slice::from_raw_parts(&raw const value as *const u8, size_of::<T>()) };
        self.write_bytes(address, bytes);
    }

    pub fn read_committed<T: Copy>(&self, address: usize) -> Option<T> {
        if !address.is_multiple_of(align_of::<T>()) {
            return None;
        }
        let regions = self.regions.lock().unwrap();
        let region = regions
            .iter()
            .find(|region| region.holds(address, size_of::<T>()))?;
        if region.access != Access::Open {
            return None;
        }
        drop(regions);
        Some(self.read(address))
    }

    pub fn vtable_in_image(&self, address: usize) -> bool {
        let Some(vtable) = self.read_committed::<usize>(address) else {
            return false;
        };
        let regions = self.regions.lock().unwrap();
        regions
            .iter()
            .any(|region| region.holds(vtable, 1) && region.kind == Kind::Image)
    }

    pub fn read_bytes(&self, address: usize, len: usize) -> Vec<u8> {
        self.bytes(address, len)
    }

    /// Puts back whatever of `address..address + len` is not there, so a restore has somewhere to
    /// write: a region reserved without being committed is committed, and one that has gone
    /// altogether comes back zeroed. Which is what the game freeing a few megabytes between a
    /// snapshot and the restore of it leaves behind, and the case the restore has to survive.
    ///
    /// Run by run, because a range can be partly there: the hole is not always at the front.
    pub fn commit(&self, address: usize, len: usize) -> bool {
        let mut regions = self.regions.lock().unwrap();
        let end = address + len;
        let mut at = address;
        while at < end {
            match regions
                .iter_mut()
                .find(|region| at >= region.base && at < region.end())
            {
                Some(region) => {
                    region.access = Access::Open;
                    at = region.end().min(end);
                }
                None => {
                    // Nothing is mapped here. The run to fill is up to whatever the next region
                    // begins at, so that filling a hole does not swallow what is past it.
                    let next = regions
                        .iter()
                        .filter(|region| region.base > at)
                        .map(|region| region.base)
                        .min()
                        .unwrap_or(end)
                        .min(end);
                    regions.push(Region {
                        base: at,
                        bytes: vec![0; next - at],
                        kind: Kind::Private,
                        access: Access::Open,
                    });
                    at = next;
                }
            }
        }
        true
    }

    pub fn write_bytes(&self, address: usize, source: &[u8]) {
        let mut regions = self.regions.lock().unwrap();
        let region = Self::find_mut(&mut regions, address, source.len());
        let at = address - region.base;
        region.bytes[at..at + source.len()].copy_from_slice(source);
    }

    pub fn fill_bytes(&self, address: usize, byte: u8, len: usize) {
        let mut regions = self.regions.lock().unwrap();
        let region = Self::find_mut(&mut regions, address, len);
        let at = address - region.base;
        region.bytes[at..at + len].fill(byte);
    }

    fn bytes(&self, address: usize, len: usize) -> Vec<u8> {
        let regions = self.regions.lock().unwrap();
        let region = Self::find(&regions, address, len);
        let at = address - region.base;
        region.bytes[at..at + len].to_vec()
    }

    fn find(regions: &[Region], address: usize, len: usize) -> &Region {
        let region = regions
            .iter()
            .find(|region| region.holds(address, len))
            .unwrap_or_else(|| {
                panic!("{address:#010x} for {len} bytes is not mapped in this space")
            });
        assert!(
            region.access == Access::Open,
            "{address:#010x} is {:?}, which a read does not come back from",
            region.access
        );
        region
    }

    fn find_mut(regions: &mut [Region], address: usize, len: usize) -> &mut Region {
        let at = regions
            .iter()
            .position(|region| region.holds(address, len))
            .unwrap_or_else(|| {
                panic!("{address:#010x} for {len} bytes is not mapped in this space")
            });
        let region = &mut regions[at];
        assert!(
            region.access == Access::Open,
            "{address:#010x} is {:?}, which a write does not reach",
            region.access
        );
        region
    }
}

thread_local! {
    static INSTALLED: RefCell<Option<Arc<Space>>> = const { RefCell::new(None) };
}

/// Puts whatever was installed before back, when it goes.
///
/// The harness hands its threads out again, so an installation left behind is the next test on
/// that thread reading a game it did not lay out. Nested rather than refused, so that a test can
/// have two games laid out at once and each read the one it is asking about — the innermost is the
/// one in front.
#[must_use = "the space comes off the thread the moment this is dropped"]
pub struct Installed(Option<Arc<Space>>);

impl Drop for Installed {
    fn drop(&mut self) {
        let previous = self.0.take();
        INSTALLED.with(|installed| *installed.borrow_mut() = previous);
    }
}

/// Puts `space` in front of the real address space, for this thread, until the return value is
/// dropped.
///
/// Takes an `Arc` so that the threads a simulated game runs on can be given the same space
/// rather than one each — an audio thread reading its own copy of the game would agree with
/// nothing.
pub fn install(space: &Arc<Space>) -> Installed {
    INSTALLED.with(|installed| {
        let mut installed = installed.borrow_mut();
        Installed(installed.replace(Arc::clone(space)))
    })
}

/// The space this thread reads through, if any.
pub fn installed() -> Option<Arc<Space>> {
    INSTALLED.with(|installed| installed.borrow().clone())
}

#[cfg(test)]
mod tests {
    use super::{Kind, Space, install, installed};
    use std::sync::Arc;

    /// Inside the game's static data, and inside this test binary's own image as well — which is
    /// the point. Every address orb reads the game at is one this binary already keeps something
    /// at, so a read that is not answered by the space is answered by whatever is there.
    const DATA: usize = 0x0069_b000;
    const CODE: usize = 0x0040_1000;

    /// What the space is for: the game's own address answers with the game's own number, in a
    /// process where no game is running.
    #[test]
    fn a_mapped_address_reads_back_what_was_put_there() {
        let space = Space::new();
        space.map(DATA, 0x100, Kind::Private);
        space.write::<u32>(DATA + 0x40, 0x1234_5678);
        assert_eq!(space.read::<u32>(DATA + 0x40), 0x1234_5678);
        // The bytes either side are the zeroes a freshly committed page reads as, and not this
        // binary's.
        assert_eq!(space.read::<u32>(DATA + 0x3c), 0);
        assert_eq!(space.read::<u32>(DATA + 0x44), 0);
    }

    /// Reserved and never committed is where every pointer into a structure the game has not
    /// built yet lands, and the answer is that there is nothing to read — which is the answer
    /// orb's pointer chases are written against.
    #[test]
    fn an_uncommitted_region_is_not_read() {
        let space = Space::new();
        space.reserve(DATA, 0x100);
        assert_eq!(space.read_committed::<u32>(DATA), None);
    }

    /// The page every thread's stack ends with. Refused separately from what is uncommitted
    /// because reading one in a real process does not merely fail: the guard comes off and the
    /// thread owning that stack stops growing it.
    #[test]
    fn a_guard_page_is_not_read() {
        let space = Space::new();
        space.guard(DATA, 0x1000);
        assert_eq!(space.read_committed::<u32>(DATA), None);
    }

    /// The alignment rule holds in the space as well, so that a test cannot pass on a read the
    /// real one would have refused. Every one of these addresses is a field of a structure the
    /// game built, so an unaligned one is not one of them.
    #[test]
    fn an_unaligned_address_is_not_read() {
        let space = Space::new();
        space.map(DATA, 0x100, Kind::Private);
        assert_eq!(space.read_committed::<u32>(DATA + 4), Some(0));
        assert_eq!(space.read_committed::<u32>(DATA + 5), None);
    }

    /// A read that would have taken the process down is a test with a wrong address in it, so it
    /// says so rather than answering. The alternative — falling through to the real address space
    /// — is the failure this whole module exists to remove: it would read this binary's image and
    /// call it the game's.
    #[test]
    #[should_panic(expected = "is not mapped in this space")]
    fn reading_where_nothing_is_mapped_says_so() {
        let space = Space::new();
        space.map(DATA, 0x100, Kind::Private);
        space.read::<u32>(DATA + 0x200);
    }

    /// How orb tells a live COM object from the stale pointer left in a block the game's
    /// allocator did not scrub: the live one's vtable is in a mapped image.
    #[test]
    fn a_vtable_is_live_only_where_it_points_into_an_image() {
        let space = Space::new();
        space.map(CODE, 0x100, Kind::Image);
        space.map(DATA, 0x100, Kind::Private);

        space.write::<usize>(DATA, CODE + 0x10);
        assert!(space.vtable_in_image(DATA));

        // Into the game's own data rather than its code: allocated, readable, and not a vtable.
        space.write::<usize>(DATA, DATA + 0x80);
        assert!(!space.vtable_in_image(DATA));

        // And nowhere at all, which is the freed object whose block still holds its old bytes.
        space.write::<usize>(DATA, 0x0dead000);
        assert!(!space.vtable_in_image(DATA));
    }

    /// What a snapshot is handed, and what it is handed after a free: a region that has gone away
    /// between the snapshot and the restore of it is the case a restore has to survive, so the
    /// space has to be able to stop having one.
    #[test]
    fn what_is_mapped_is_what_has_not_been_unmapped() {
        let space = Space::new();
        space.map(DATA, 0x100, Kind::Private);
        space.map(CODE, 0x40, Kind::Image);
        // Reserved without being committed is not memory anything can be read out of, so it is not
        // memory a snapshot has anything to save.
        space.reserve(DATA + 0x1000, 0x100);

        let mut mapped = space.mapped();
        mapped.sort_unstable();
        assert_eq!(mapped, vec![(CODE, 0x40), (DATA, 0x100)]);

        space.unmap(DATA);
        assert_eq!(space.mapped(), vec![(CODE, 0x40)]);
    }

    /// The harness hands its threads out again, so an installation left behind would be the next
    /// test on that thread reading a game it did not lay out.
    #[test]
    fn a_space_comes_off_the_thread_when_its_installation_goes() {
        assert!(installed().is_none());
        let space = Arc::new(Space::new());
        {
            let _installed = install(&space);
            assert!(installed().is_some());
        }
        assert!(installed().is_none());
    }
}
