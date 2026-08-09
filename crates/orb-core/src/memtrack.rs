//! The set of regions the game owns.
//!
//! The game keeps most of its state in `.data`, but not all of it: `malloc` in its statically linked
//! MSVC6 CRT goes to a private heap, and a few things come straight from `VirtualAlloc`. What a chapter
//! is a copy of is that whole set, and this is where a snapshot asks for it.
//!
//! **The noticing is `orb::memtrack`** — the exe's imports for both allocators hooked — and **the walk
//! is behind the seam**, `orb_api::mem::game_regions` being `HeapLock`, `HeapWalk` and `VirtualQuery`
//! over a real process, and laid-out memory answering for itself. What each hook noticed goes over as
//! `mem::note_heap` and `mem::note_reservation`.
//!
//! **The rule that no two entries cover the same pages went with the walk**, and that is not tidying:
//! it is a rule about real pages, a heap region and a reservation being able to name the same ones. A
//! laid-out game's regions are the objects a scenario put there, and two of those merged because they
//! happen to abut is one range that nothing in that space can read. See
//! [docs/adr/0010](../../../docs/adr/0010-orb-is-the-patched-bytes-and-everything-else-has-one-of-two-other-homes.md).

use std::ops::Range;

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

/// Everything worth saving, as committed readable ranges, with the data range first.
///
/// Walking the heaps takes the heap lock, so this must be called before any thread is suspended, and its
/// result treated as a plan rather than a fact: `snapshot` re-checks each range before touching it.
///
/// Asked through the seam rather than from behind a `cfg(test)`, which is where the choice used to be
/// made: `cfg(test)` is false in a crate compiled as a dependency of a test binary, so the scenario that
/// drives a whole run reached the walk with no heaps tracked — a chapter that copied `.data` and nothing
/// else. What that lost was the fight's own block: the clock of the attack a chapter began on did not
/// come back with the chapter.
///
/// # Safety
/// `data` must be the exe's `.data` range.
pub unsafe fn regions(data: Range<usize>) -> Vec<Region> {
    let started = profile::now();
    let regions = orb_api::mem::game_regions(&data)
        .into_iter()
        .map(|(base, len)| Region { base, len })
        .collect();
    unsafe { profile::record(profile::Phase::Regions, started) };
    regions
}
