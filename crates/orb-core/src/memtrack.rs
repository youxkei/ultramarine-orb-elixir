//! The set of regions the game owns.
//!
//! The game keeps most of its state in `.data`, but not all of it: `malloc` in its statically linked
//! MSVC6 CRT goes to a private heap, and a few things come straight from `VirtualAlloc`. What a chapter
//! is a copy of is that whole set, and this is the half of it that is arithmetic — a region, and the
//! rule that no two of them ever cover the same pages.
//!
//! **The noticing and the walk are `orb::memtrack`**, which hooks the exe's imports for both allocators
//! and then walks what they handed out. That is `HeapWalk` and `VirtualQuery` over a real process, so it
//! cannot live here — see [`hands_over_the_walk`].

use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// The walk of a real process's heaps, as `orb` hands it over.
pub type Walk = unsafe fn(Range<usize>) -> Vec<Region>;

/// Where that walk is, or zero before anything has handed one over.
///
/// **The one thing this crate is handed the other way round**, and the reason is the reason the three
/// hook bodies that patch memory are handed over too: what is on the other side of it is Windows, and
/// this crate may not name Windows. A simulated host never reaches it — `orb_api::mem::game_regions`
/// answers first — so a scenario drives everything above this line and nothing below it.
static WALK: AtomicUsize = AtomicUsize::new(0);

/// Says where the walk is, which `orb::memtrack::install` does as it patches the imports it walks the
/// results of.
pub fn hands_over_the_walk(walk: Walk) {
    WALK.store(walk as usize, Ordering::Relaxed);
}

/// Everything worth saving, as committed readable ranges.
///
/// Walking the heaps takes the heap lock, so this must be called before any thread is suspended, and its
/// result treated as a plan rather than a fact: `snapshot` re-checks each range before touching it.
///
/// # Safety
/// `data` must be the exe's `.data` range.
pub unsafe fn regions(data: Range<usize>) -> Vec<Region> {
    let started = profile::now();
    let regions = unsafe { asked(data) };
    unsafe { profile::record(profile::Phase::Regions, started) };
    regions
}

/// # Safety
/// As [`regions`].
unsafe fn asked(data: Range<usize>) -> Vec<Region> {
    // A laid-out simulated Windows *is* the game's memory, so what it holds is the whole answer: there
    // are no heaps to walk and no reservations to have been told about. The data range still leads, as it
    // does in a real game, and the rest is whatever else the test put there.
    //
    // Asked through the seam rather than behind a `cfg(test)`, which is where it was: `cfg(test)` is
    // false in a crate compiled as a dependency of a test binary, so the scenario that drives a whole run
    // reached the walk below with no heaps tracked — a chapter that copied `.data` and nothing else. What
    // that lost was the fight's own block: the clock of the attack a chapter began on did not come back
    // with the chapter.
    if let Some(regions) = orb_api::mem::game_regions(&data) {
        return regions
            .into_iter()
            .map(|(base, len)| Region { base, len })
            .collect();
    }
    let handed_over = WALK.load(Ordering::Relaxed);
    // Nothing has handed a walk over, which is a launch with the memory hooks turned off: the data range
    // is then the whole of what a chapter is a copy of, and everything the game allocated is outside it.
    if handed_over == 0 {
        return vec![Region {
            base: data.start,
            len: data.len(),
        }];
    }
    let walk: Walk = unsafe { std::mem::transmute(handed_over) };
    unsafe { walk(data) }
}

/// Heap regions and direct reservations can name the same pages; saving them
/// twice would make a restore's write order decide the outcome.
///
/// Every entry the region touches and not the first of them: one that bridges two entries
/// already apart — a heap region and a reservation with a gap between — would otherwise grow
/// the first across the second and leave the pair overlapping, which is the duplicate this
/// exists to prevent. One pass reaches them all, because no two entries here ever touch each
/// other: this is the only thing that adds one.
pub fn push_merged(out: &mut Vec<Region>, region: Region) {
    let mut base = region.base;
    let mut end = region.end();
    out.retain(|existing| {
        let touching = base <= existing.end() && existing.base <= end;
        if touching {
            base = base.min(existing.base);
            end = end.max(existing.end());
        }
        !touching
    });
    out.push(Region {
        base,
        len: end - base,
    });
}

#[cfg(test)]
mod tests {
    use super::{Region, push_merged};

    fn region(base: usize, end: usize) -> Region {
        Region {
            base,
            len: end - base,
        }
    }

    /// Nothing here covers the same pages as anything else, whichever order the walk found
    /// them in — including where what arrives bridges two that were apart.
    #[test]
    fn a_region_bridging_two_entries_leaves_one() {
        for mut out in [
            vec![region(0x1000, 0x2000), region(0x3000, 0x4000)],
            vec![region(0x3000, 0x4000), region(0x1000, 0x2000)],
        ] {
            push_merged(&mut out, region(0x1800, 0x3800));
            assert_eq!(out, [region(0x1000, 0x4000)]);
        }
    }

    /// One already covered adds nothing, one that abuts an entry extends it, and one that
    /// touches nothing stands on its own.
    #[test]
    fn what_is_already_covered_is_not_saved_again() {
        let mut out = vec![region(0x1000, 0x4000)];
        push_merged(&mut out, region(0x2000, 0x3000));
        assert_eq!(out, [region(0x1000, 0x4000)]);

        push_merged(&mut out, region(0x4000, 0x5000));
        assert_eq!(out, [region(0x1000, 0x5000)]);

        push_merged(&mut out, region(0x8000, 0x9000));
        assert_eq!(out, [region(0x1000, 0x5000), region(0x8000, 0x9000)]);
    }
}
