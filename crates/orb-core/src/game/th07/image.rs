//! A th07 process image laid out in an address space, so that the real [`Th07`](super::Th07) has
//! something to read in a test with no game running.
//!
//! Far less of one than [`th06::image`](super::super::th06::image), and for the reason the `Th07`
//! beside it declines almost everything: what a frame of orb's own reads out of this space is the device
//! it draws through, the window it asks about, the chain it calls the update and the draw at, the queue of
//! quads it empties and draws around them and the fog flag it writes — and the update, the draw and the
//! calls on that queue are functions an e2e test hands over rather than code that would have to be here.
//! Everything a run is made of is read by methods that answer without looking, so there is nothing of a
//! run to lay out.
//!
//! The addresses are the game's own, written from the same constants the reads use, so what this cannot
//! catch is a constant that is wrong: a wrong one is wrong on both sides at once, and only the real
//! game says otherwise. What it does catch is everything built on top of them, which is where the code
//! is.

use std::ops::Range;
use std::sync::Arc;

use orb_api::{Hwnd, Kind};
use orb_sim::{Sim, Space};

use orb_api::Device;

/// The game's static data, as the real one is: **one range**, `0x0049c000..0x01365258`, which is what
/// orb reads out of the PE and writes in the log every run. Six times 紅魔郷's, and the range a
/// snapshot of a chapter would be a copy of if 妖々夢 had chapters.
///
/// The section header's own virtual size, `0xec9258` — not its raw size, which is `0x3800`: the rest is
/// the globals the loader zeroes, and every address below is in it.
const DATA: Range<usize> = 0x0049_c000..0x0136_5258;

/// The object the drawing queues its quads in, which in the real game is 0x17e560 bytes off the
/// allocator and so is in none of [`DATA`]: a base of this file's choosing, the length the game's own.
///
/// The length is what makes it this block and not a smaller one — the two fields the frame's calls move
/// are at 0x17e534 and 0x17e538, near the end of it, and the buffer they point into is everything from
/// 0x2e534 up to them.
const QUAD_QUEUE: Range<usize> = 0x0300_0000..0x0317_E560;

/// Where the next quad goes, and where drawing the queue starts from: `0x17e534` and `0x17e538` of the
/// object.
///
/// Read off the three functions that move them — 0x44f580 points the first at the buffer and copies it
/// into the second, 0x44f690 adds [`QUAD_BYTES`] to the first, and 0x44f5c0 draws from the second and
/// then copies the first into it.
const QUEUE_WRITE_AT: usize = 0x0017_E534;
const QUEUE_DRAW_FROM: usize = 0x0017_E538;
/// The buffer those two point into, and the count of quads in it: `this + 0x2e534` as 0x44f580 writes
/// it, and the `0x2e530` it zeroes.
const QUEUE_BUFFER: usize = 0x0002_E534;
const QUEUED: usize = 0x0002_E530;
/// How many times the queue has been drawn: `[this+0x14]`, which 0x44f5c0 adds one to at 0x44f679 on the
/// calls that had something to draw.
///
/// The game's own count and not this file's, which is what makes it worth reading back: what an e2e
/// test asks of orb's frame is that the game's own idea of how many times it drew agrees with the
/// frames.
const QUEUE_DRAWS: usize = 0x14;
/// What one quad costs the buffer: six vertices of 0x1c bytes, which is what 0x44f690 copies in — the
/// four of a quad as two triangles, `v0 v1 v2 v1 v2 v3`, from its argument's 0x70 bytes.
const QUAD_BYTES: usize = 0xa8;

/// Which pad button is which as the game's own defaults have it, which is what a launch with nobody's
/// `th07.cfg` in it reads: the 18 bytes at 0x49ee40, copied to the mapping at 0x4399d3.
///
/// Laid out because zeros are not neutral here — a table of them says every bit of the word is button 0 —
/// and because this is the table a launch really has until somebody configures the game.
const PAD_MAPPING: [i16; 9] = [0, 1, 2, 4, -1, -1, -1, -1, 3];

pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    /// In a host whose non-determinism is drawn from `seed` — the wake delays and the compositor's
    /// spikes — which an e2e test names in its assertions so that a failure can be replayed. The one
    /// constructor, for the reason `th06`'s says.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        sim.space().map(DATA.start, DATA.len(), Kind::Private);
        sim.space()
            .map(QUAD_QUEUE.start, QUAD_QUEUE.len(), Kind::Private);
        sim.space()
            .write::<usize>(super::G_QUAD_QUEUE, QUAD_QUEUE.start);
        sim.space()
            .write::<[i16; 9]>(super::PAD_MAPPING, PAD_MAPPING);
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
    pub fn shows_through(&self, device: Device, window: Hwnd) {
        let space = self.space();
        space.write::<usize>(super::G_D3D_DEVICE, device.0);
        space.write::<Hwnd>(super::G_GAME_WINDOW, window);
    }

    /// The object the game calls its own whole frame on, which orb's loop is handed and hands the frame
    /// back to on each of the three ways out of it that return.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// The chain the update and the draw are called on. Nothing is laid out at it — an e2e test's own
    /// two functions are what orb calls, and this is the argument they are called with — so what it has
    /// to be is an address inside the space, which is what a real one is.
    pub fn chain_object(&self) -> usize {
        super::G_CHAIN
    }

    /// The whole-output viewport the game keeps in one struct and orb writes as it prepares a frame,
    /// for an e2e test that reads back what orb put there.
    pub fn viewport(&self) -> [u32; 4] {
        let space = self.space();
        [0, 1, 2, 3].map(|word| space.read::<u32>(super::VIEWPORT + word * 4))
    }

    /// The three things the game's own frame does to the queue of quads, as the exe does them: the
    /// queue emptied at 0x44f580, a quad added at 0x44f690, and the queue drawn at 0x44f5c0.
    ///
    /// **Here rather than in whatever drives the game, because the block is here.** They are the game's
    /// own code and a laid-out game has to be able to answer them — code being the one thing an address
    /// space cannot hold — and what each is made of is the fields above, which nothing outside this file
    /// knows the offsets of.
    ///
    /// A quad added is the whole of why any of this is laid out: it writes [`QUAD_BYTES`] where
    /// [`QUEUE_WRITE_AT`] points, so a frame whose queue was never emptied writes to address zero, and
    /// one that emptied it and never drew walks up the buffer until it runs off the end. Both are what
    /// the real game does, which is the point — the first of them is the fault that took 妖々夢 down.
    pub fn empties_the_queue(&self) {
        let space = self.space();
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, 0);
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_WRITE_AT,
            QUAD_QUEUE.start + QUEUE_BUFFER,
        );
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_DRAW_FROM,
            space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT),
        );
    }

    pub fn queues_a_quad(&self) {
        let space = self.space();
        let at = space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT);
        // The vertices themselves, which nothing reads: what a quad *is* is the bytes it costs the
        // buffer and the address they went to.
        space.fill_bytes(at, 0xff, QUAD_BYTES);
        space.write::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT, at + QUAD_BYTES);
        let queued = space.read::<u32>(QUAD_QUEUE.start + QUEUED);
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, queued + 1);
    }

    pub fn draws_the_queue(&self) {
        let space = self.space();
        // Nothing at all where there is nothing queued, which is what makes the second of the frame's two
        // calls the game's own no-op: the count is what 0x44f5d1 returns on.
        if space.read::<u32>(QUAD_QUEUE.start + QUEUED) == 0 {
            return;
        }
        space.write::<usize>(
            QUAD_QUEUE.start + QUEUE_DRAW_FROM,
            space.read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT),
        );
        space.write::<u32>(QUAD_QUEUE.start + QUEUED, 0);
        let drawn = space.read::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS);
        space.write::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS, drawn + 1);
    }

    /// Where the next quad would go, for an e2e test that reads it back: the start of the buffer on a
    /// frame whose queue was emptied, and one quad further on for every quad drawn since it was not.
    pub fn queue_writes_at(&self) -> usize {
        self.space()
            .read::<usize>(QUAD_QUEUE.start + QUEUE_WRITE_AT)
    }

    /// The start of that buffer, and what one quad costs it: what the answer above is worth reading
    /// against.
    pub fn queue_buffer(&self) -> usize {
        QUAD_QUEUE.start + QUEUE_BUFFER
    }

    pub fn queue_bytes_per_quad(&self) -> usize {
        QUAD_BYTES
    }

    /// And how many quads are in it now, and how many times the game has drawn it.
    pub fn queued(&self) -> u32 {
        self.space().read::<u32>(QUAD_QUEUE.start + QUEUED)
    }

    pub fn queue_draws(&self) -> u32 {
        self.space().read::<u32>(QUAD_QUEUE.start + QUEUE_DRAWS)
    }

    /// The pad's buttons as the game's own key config read would have answered them, and where that array
    /// is: 0x80 in the byte of each button held.
    ///
    /// The read itself is the game's own function, which a laid-out game hands over — this is the array it
    /// answers with, and what an e2e test reads back to see the buttons orb added.
    pub fn pad_buttons(&self) -> usize {
        super::PAD_BUTTONS
    }

    pub fn pad_button_held(&self, button: usize) -> bool {
        self.space().read::<u8>(super::PAD_BUTTONS + button) & 0x80 != 0
    }

    /// And the array emptied, which is what the game's own read does before it fills it — every byte of
    /// what its own screen walks.
    pub fn empties_the_pad_buttons(&self) {
        self.space()
            .fill_bytes(super::PAD_BUTTONS, 0, super::PAD_BUTTONS_READ);
    }

    /// The fog turned on, as a stage's own background drawing turns it on at 0x43a1bd: the field both of
    /// the game's fog calls compare and write, set to the 1 that call writes.
    pub fn turns_the_fog_on(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, 1);
    }

    pub fn fog_is_on(&self) -> bool {
        self.space()
            .read::<u32>(super::G_SUPERVISOR + super::FOG_ON)
            != 0
    }

    /// The same field written the way the game's own frame writes it at 0x434739, which is what makes
    /// the call after it reach the device: 0xff, and the call does nothing at all where this says the
    /// fog is already out.
    pub fn forces_the_fog_call_through(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, super::FOG_ON_FORCED);
    }

    /// And the field 0x43a207 clears, which is the half of that call this block holds: the
    /// `SetRenderState` beside it goes through the device, which is a real object in an e2e test and not
    /// laid out at all.
    pub fn puts_the_fog_out(&self) {
        self.space()
            .write::<u32>(super::G_SUPERVISOR + super::FOG_ON, 0);
    }

    /// Hands over the three calls the game's own frame makes on that queue and on the fog, in place of
    /// the addresses `Th07` holds: [`empties_the_queue`](Image::empties_the_queue),
    /// [`draws_the_queue`](Image::draws_the_queue) and the fog put out.
    ///
    /// All three together, because orb's frame makes all three: a game that handed over fewer would be
    /// jumping into memory nothing has mapped part way through one frame.
    pub fn hands_over_the_frames_own_calls(
        &self,
        empties_the_queue: usize,
        draws_the_queue: usize,
        puts_the_fog_out: usize,
    ) {
        super::set_empty_queue(empties_the_queue);
        super::set_draw_queue(draws_the_queue);
        super::set_fog_off(puts_the_fog_out);
    }

    pub fn data(&self) -> Range<usize> {
        DATA
    }

    /// The host this game is laid out in, for an e2e test that needs more of it than the memory — the
    /// display it declares, or which window is in front.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }
}
