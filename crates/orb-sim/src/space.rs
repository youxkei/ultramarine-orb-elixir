//! An address space put in front of the real one, so that the game's memory can be read and
//! written with no game there to own it.
//!
//! The addresses `Th06` reads are absolute and belong to a process orb is not running in during
//! a test. Two ways to make them readable, and this is the second:
//!
//! - `VirtualAlloc` them in the test process at the bases the game uses. Ties every test to a
//!   host where 0x0069bca0 and its neighbours happen to be free — the test binary's own image
//!   is one bad link away from sitting there — and to a Windows that has `VirtualAlloc` at
//!   all, which is what keeps the suite off a Linux runner.
//! - Answer the reads from a map. The same bytes at the same addresses, with none of that,
//!   and the fields a structure has not been built with yet read as zero the way a freshly
//!   committed page does.

use std::ops::Range;
use std::sync::Mutex;

use orb_api::Kind;

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

/// Reinterprets bytes out of the space as what was asked for.
///
/// Sound for what this seam carries, which is what the game's structures are made of: integers,
/// floats, pointers, and arrays of those. Nothing with a destructor, nothing holding a reference,
/// nothing whose bit patterns are not all values of the type.
fn reinterpret<T: Copy>(bytes: &[u8]) -> T {
    assert_eq!(bytes.len(), size_of::<T>());
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), value.as_mut_ptr().cast::<u8>(), bytes.len());
        value.assume_init()
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

    /// Whether nothing is laid out over `base..base + len`, which is what a real object of this process has
    /// to be true of before the space can be told where it is.
    ///
    /// **Which is a hazard rather than a theory.** A laid-out game claims the addresses the real one's
    /// `.data` and heap blocks are at, and this is a 32-bit process whose own heap reaches both — so an
    /// allocation can land at an address already laid out here, and [`map`](Space::map) stops rather than
    /// shadowing the game's own memory. Watched in `orb-e2e`'s `the_music_across_a_restore` at 0x6ce8f0,
    /// 0x301df98 and 0x651398, when the sound buffer was a real object of this crate's whose address the
    /// space was told. It is not any more — the buffer is answered through the seam — and what asks now is
    /// `orb-e2e`'s `vtable_for`, over the address it lays the device's vtable at: that one is not a real
    /// object either, the slot orb patches being swapped through `orb_api::mem::replace_word`, but the
    /// address is chosen rather than worked out and a scenario that laid a game out over it should hear so
    /// there rather than inside [`map`](Space::map).
    pub fn has_room(&self, base: usize, len: usize) -> bool {
        let regions = self.regions.lock().unwrap();
        !regions
            .iter()
            .any(|region| base < region.end() && region.base < base + len)
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
    pub fn game_regions(&self, data: &Range<usize>) -> Vec<(usize, usize)> {
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
        reinterpret(&self.bytes(address, size_of::<T>()))
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
        Some(reinterpret(
            &self.read_committed_bytes(address, size_of::<T>())?,
        ))
    }

    /// The bytes of `address..address + len`, or `None` where the region holding them is not
    /// readable — reserved without being committed, guarded, or not laid out at all.
    ///
    /// Alignment is not asked about here. It is a property of the type being read and not of the
    /// address space, so it is refused on the near side of the seam, in `orb_api::mem`.
    pub fn read_committed_bytes(&self, address: usize, len: usize) -> Option<Vec<u8>> {
        let regions = self.regions.lock().unwrap();
        let region = regions.iter().find(|region| region.holds(address, len))?;
        if region.access != Access::Open {
            return None;
        }
        let at = address - region.base;
        Some(region.bytes[at..at + len].to_vec())
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
