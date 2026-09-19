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
//! laid-out game's regions are the objects an e2e test put there, and two of those merged because they
//! happen to abut is one range that nothing in that space can read.
//!
//! **And each region arrives with the source that handed it out**, `orb_api::mem::Source` — the game's
//! `n`th heap, its `n`th reservation, or `.data` — because a place in this list is not a name: a heap's
//! committed runs are as many regions as its manager currently holds them in, and one launch of th06nc
//! listed the game's heap as 8 in one chapter and 9 in another. What that costs where it is missing is a
//! bisect: `--hold-back-at` can only be pointed at a span of the game's heap by something that means the
//! same pages twice.

use std::ops::Range;

use orb_api::mem::Source;

use crate::{crash, detail, profile};

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

/// One region of the set, and which of the things the game took handed it out.
///
/// **The source is here and not in [`Region`]**, which is the type every other list of ranges in
/// `snapshot` is made of — the sound's, the live handles', the holes a restore leaves. Those are spans
/// orb named itself and have no source to answer with, and a field they would all have to fill with
/// something is a field that says nothing wherever it is read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Owned {
    pub region: Region,
    /// What a launch bisecting a restore names this region by — see [`orb_api::mem::Source`].
    pub from: Source,
}

/// Everything worth saving, as committed readable ranges, with the data range first.
///
/// Walking the heaps takes the heap lock, so this must be called before any thread is suspended, and its
/// result treated as a plan rather than a fact: `snapshot` re-checks each range before touching it.
///
/// Asked through the seam rather than from behind a `cfg(test)`, which is where the choice used to be
/// made: `cfg(test)` is false in a crate compiled as a dependency of a test binary, so the e2e test that
/// drives a whole run reached the walk with no heaps tracked — a chapter that copied `.data` and nothing
/// else. What that lost was the fight's own block: the clock of the attack a chapter began on did not
/// come back with the chapter.
///
/// # Safety
/// `data` must be the exe's `.data` range.
pub unsafe fn regions(data: Range<usize>) -> Vec<Owned> {
    let started = profile::now();
    // The walk takes the heaps' own locks and reads their structures, so a fault inside it is a heap that
    // has been written over rather than anything of orb's — see `crash::Doing`.
    crash::doing(crash::Doing::WalkingTheHeaps, 0, 0);
    let regions: Vec<Owned> = orb_api::mem::game_regions(&data)
        .into_iter()
        .map(|(base, len, from)| Owned {
            region: Region { base, len },
            from,
        })
        .collect();
    crash::doing(crash::Doing::Nothing, 0, 0);
    unsafe { profile::record(profile::Phase::Regions, started) };
    // Which pages a chapter is a copy of, said in full: what a restore writes over is exactly this, so a
    // range in it that turns out not to be the game's is read off here and nowhere else.
    //
    // **With the source in front of each**, which is what makes the line worth reading twice in one
    // launch: the same heap walks to a different number of regions as its manager grows and joins them, so
    // a list of nine bare ranges and a list of eight say nothing about which of them are the same pages.
    detail!(
        "memory: saving {}",
        regions
            .iter()
            .map(|owned| format!(
                "{} {:#010x}+{:#x}",
                owned.from, owned.region.base, owned.region.len
            ))
            .collect::<Vec<_>>()
            .join(" "),
    );
    regions
}
