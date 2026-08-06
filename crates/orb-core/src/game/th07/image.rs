//! A th07 process image laid out in an address space, so that the real [`Th07`](super::Th07) has
//! something to read in a test with no game running.
//!
//! Far less of one than [`th06::image`](super::super::th06::image), and for the reason the `Th07`
//! beside it declines almost everything: a frame of orb's own reads three things out of this space —
//! the device it draws through, the window it asks about, and the chain it calls the update and the
//! draw at — and the update and the draw themselves are functions a scenario hands over rather than
//! code that would have to be here. Everything a run is made of is read by methods that answer without
//! looking, so there is nothing of a run to lay out.
//!
//! The addresses are the game's own, written from the same constants the reads use, so what this cannot
//! catch is a constant that is wrong: a wrong one is wrong on both sides at once, and only the real
//! game says otherwise. What it does catch is everything built on top of them, which is where the code
//! is.

use std::ops::Range;
use std::sync::Arc;

use orb_api::{Hwnd, Kind};
use orb_sim::{Sim, Space};

use crate::d3d8::Device;

/// The game's static data, as the real one is: **one range**, `0x0049c000..0x01365258`, which is what
/// orb reads out of the PE and writes in the log every run. Six times 紅魔郷's, and the range a
/// snapshot of a chapter would be a copy of if 妖々夢 had chapters.
///
/// The section header's own virtual size, `0xec9258` — not its raw size, which is `0x3800`: the rest is
/// the globals the loader zeroes, and every address below is in it.
const DATA: Range<usize> = 0x0049_c000..0x0136_5258;

pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    pub fn laid_out() -> Self {
        Self::mapped(Arc::new(Sim::new()))
    }

    /// And in a host whose non-determinism is drawn from `seed` — the wake delays and the compositor's
    /// spikes — for a scenario that names the seed in its assertions so that a failure can be replayed.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        sim.space().map(DATA.start, DATA.len(), Kind::Private);
        Self { sim }
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which is
    /// what makes `Th07`'s reads land in it.
    pub fn enter(&self) -> orb_api::Installed {
        self.sim.enter()
    }

    pub fn space(&self) -> &Space {
        self.sim.space()
    }

    /// The device orb draws through and the window it asks about, where the game's own
    /// `IDirect3D8::CreateDevice` left them.
    ///
    /// A real pointer for the device, code being the one thing an address space cannot hold: orb reads
    /// the pointer out of here and then goes through its vtable, which is a vtable of Rust functions.
    pub fn shows_through(&self, device: *mut Device, window: Hwnd) {
        let space = self.space();
        space.write::<*mut Device>(super::G_D3D_DEVICE, device);
        space.write::<Hwnd>(super::G_GAME_WINDOW, window);
    }

    /// The object the game calls its own whole frame on, which orb's loop is handed and hands the frame
    /// back to on each of the three ways out of it that return.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// The chain the update and the draw are called on. Nothing is laid out at it — a scenario's own
    /// two functions are what orb calls, and this is the argument they are called with — so what it has
    /// to be is an address inside the space, which is what a real one is.
    pub fn chain_object(&self) -> usize {
        super::G_CHAIN
    }

    /// The whole-output viewport the game keeps in one struct and orb writes as it prepares a frame,
    /// for a scenario that reads back what orb put there.
    pub fn viewport(&self) -> [u32; 4] {
        let space = self.space();
        [0, 1, 2, 3].map(|word| space.read::<u32>(super::VIEWPORT + word * 4))
    }

    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// The host this game is laid out in, for a scenario that needs more of it than the memory — the
    /// display it declares, or which window is in front.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }
}
