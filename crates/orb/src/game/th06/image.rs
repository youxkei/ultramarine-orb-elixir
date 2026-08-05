//! A th06 process image laid out in a [`Space`], so that the real [`Th06`](super::Th06) has
//! something to read in a test with no game running.
//!
//! The regions are the game's own, at the game's own addresses, and they are laid out from the
//! same constants the reads use — so what this cannot catch is a constant that is wrong. A wrong
//! offset makes the writer and the reader wrong together, and only the real game says otherwise;
//! that is what the measurements in `DONE.md` are for. What it does catch is everything built on
//! top of the offsets, which is where the code is.
//!
//! Zeroed, not filled with plausible values. A field the game has not written is zero in a
//! freshly committed page too, and a `Game` method that has to survive reading one is a method
//! whose pointer chases go through `read_committed` — which is the property worth holding it to.

use std::ops::Range;
use std::sync::Arc;

use crate::mem::space::{self, Kind, Space};

/// The game's static data: the game manager, the spell card history 0x30 into it, and the job
/// chain past that. This is the range a chapter is a copy of.
const DATA: Range<usize> = 0x0069_b000..0x0069_e000;

/// The supervisor, and the window handle beside it.
const SUPERVISOR: Range<usize> = 0x006c_6000..0x006c_7000;

/// The replay manager, the sound player, the graphics manager and the front end, which sit within
/// a couple of pages of each other.
const MANAGERS: Range<usize> = 0x006d_3000..0x006d_5000;

/// Which spell card is up, which the game keeps well away from the rest.
const CARDS: Range<usize> = 0x005a_5000..0x005a_6000;

/// Stands for the game's own code, so that a pointer into it reads as a live COM object's vtable
/// and one anywhere else reads as the stale pointer left in a block the allocator did not scrub.
const CODE: Range<usize> = 0x0040_1000..0x0040_2000;

/// The game's memory, laid out. Reading it goes through [`enter`](Self::enter).
pub struct Image {
    space: Arc<Space>,
}

impl Image {
    pub fn laid_out() -> Self {
        let space = Arc::new(Space::new());
        for region in [&DATA, &SUPERVISOR, &MANAGERS, &CARDS] {
            space.map(region.start, region.len(), Kind::Private);
        }
        space.map(CODE.start, CODE.len(), Kind::Image);
        Self { space }
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which
    /// is what makes `Th06`'s reads land in it.
    ///
    /// Scoped rather than held for the image's whole life, so that a test can lay out two games and
    /// each read the one it is asking about: the one entered is the one in front.
    pub fn enter(&self) -> space::Installed {
        space::install(&self.space)
    }

    pub fn space(&self) -> &Arc<Space> {
        &self.space
    }

    /// What a snapshot covers, which orb reads out of the PE in a real game.
    pub fn data(&self) -> Range<usize> {
        DATA
    }
}
