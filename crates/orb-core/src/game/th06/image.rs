//! A th06 process image laid out in an address space, so that the real
//! [`Th06`] has something to read in a test with no game running.
//!
//! The space itself is `orb_sim::Space`, which this crate cannot link to: it is a dev-dependency and
//! the seam runs the other way round — the simulator implements `orb_api::Win` and knows nothing about
//! this.
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

use orb_api::Kind;
use orb_sim::{Sim, Space};

/// The game's static data: the game manager, the spell card history 0x30 into it, and the job
/// chain past that. This is the range a chapter is a copy of.
const DATA: Range<usize> = 0x0069_b000..0x0069_e000;

/// The supervisor, and the window handle beside it.
const SUPERVISOR: Range<usize> = 0x006c_6000..0x006c_7000;

/// The replay manager, the sound player, the graphics manager and the front end, which sit within
/// a couple of pages of each other.
const MANAGERS: Range<usize> = 0x006d_3000..0x006d_5000;

/// The player, which is 0x98f0 bytes of it and reaches almost to the managers above: `g_Player` is
/// at 0x006ca628 and `g_ReplayManager` at 0x006d3f18 is what comes after it. Stopped short of
/// [`MANAGERS`] rather than run right up to it, since two regions that touch are two regions and
/// laying them out overlapping is refused.
const PLAYER: Range<usize> = 0x006c_a000..0x006d_3000;

/// The enemy manager, which is where the script's clock, the enemy count and the spell card flag
/// are read from.
const ENEMIES: Range<usize> = 0x004b_7000..0x004b_9000;

/// Which spell card is up, and the head of the bullet manager beside it — `g_BulletManager` is at
/// 0x005a5ff8, which is why this runs a page past where the cards are.
const CARDS: Range<usize> = 0x005a_5000..0x005a_7000;

/// The bullet manager's laser array, which is 0xec000 into it: 0x005a5ff8 + 0xec000 puts laser zero
/// at 0x00691ff8, and sixty-four of them at 0x270 apiece reach 0x0069bbf8. So the tail of the array
/// and the bullet count past it are inside [`DATA`] already, and what is missing is the run up to
/// it. Not the whole megabyte between the manager's head and here: nothing reads the bullets
/// themselves, only the count and the lasers' in-use flags.
const LASERS: Range<usize> = 0x0069_1000..0x0069_b000;

/// Stands for the game's own code, so that a pointer into it reads as a live COM object's vtable
/// and one anywhere else reads as the stale pointer left in a block the allocator did not scrub.
const CODE: Range<usize> = 0x0040_1000..0x0040_2000;

/// A stage in progress, as a scenario says it.
///
/// Every field is one `read_state` parses back out of the game's memory, which is what makes a test
/// that writes these and reads a `State` a test of the parse rather than of itself.
#[derive(Clone, Copy)]
pub struct Playing {
    /// Counted from zero, as everything above `Game` counts them.
    pub stage: i32,
    pub difficulty: i32,
    pub frames: u32,
    pub script_frames: i32,
    pub seed: u16,
    pub deaths: i32,
    pub lives: i8,
    pub bombs: i8,
    pub power: u16,
    pub enemies: i32,
}

impl Default for Playing {
    /// Stage one on Normal, on its first frame, with the lives and bombs a run starts with.
    fn default() -> Self {
        Self {
            stage: 0,
            difficulty: 1,
            frames: 0,
            script_frames: 0,
            seed: 0,
            deaths: 0,
            lives: 2,
            bombs: 3,
            power: 0,
            enemies: 0,
        }
    }
}

/// The game's memory, laid out. Reading it goes through [`enter`](Self::enter).
pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    pub fn laid_out() -> Self {
        let sim = Arc::new(Sim::new());
        for region in [
            &DATA,
            &SUPERVISOR,
            &MANAGERS,
            &CARDS,
            &PLAYER,
            &ENEMIES,
            &LASERS,
        ] {
            sim.space().map(region.start, region.len(), Kind::Private);
        }
        sim.space().map(CODE.start, CODE.len(), Kind::Image);
        Self { sim }
    }

    /// Puts this image in front of the real address space for as long as the answer is held, which
    /// is what makes `Th06`'s reads land in it.
    ///
    /// Scoped rather than held for the image's whole life, so that a test can lay out two games and
    /// each read the one it is asking about: the one entered is the one in front.
    pub fn enter(&self) -> orb_api::Installed {
        self.sim.enter()
    }

    pub fn space(&self) -> &Space {
        self.sim.space()
    }

    /// Lays a stage in progress over the game's globals.
    ///
    /// In the game's own terms rather than as addresses, so that a scenario says what the game is
    /// doing and this file stays the only place that knows where any of it is kept. The offsets are
    /// the same constants [`Th06`](super::Th06) reads through, reached as a child module of it —
    /// which is what keeps the writer and the reader from drifting apart.
    pub fn playing(&self, run: Playing) {
        use super::{game_manager, supervisor};
        let space = self.space();
        space.write::<i32>(
            super::G_SUPERVISOR + supervisor::CUR_STATE,
            super::STATE_GAMEMANAGER,
        );
        space.write::<i32>(
            super::G_SUPERVISOR + supervisor::WANTED_STATE,
            super::STATE_GAMEMANAGER,
        );
        // The game counts stages from one while one is running; `read_state` takes the one back off.
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::CURRENT_STAGE,
            run.stage + 1,
        );
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::DIFFICULTY,
            run.difficulty,
        );
        space.write::<u32>(
            super::G_GAME_MANAGER + game_manager::GAME_FRAMES,
            run.frames,
        );
        space.write::<u16>(super::G_GAME_MANAGER + game_manager::RANDOM_SEED, run.seed);
        space.write::<i32>(super::G_GAME_MANAGER + game_manager::DEATHS, run.deaths);
        space.write::<i8>(
            super::G_GAME_MANAGER + game_manager::LIVES_REMAINING,
            run.lives,
        );
        space.write::<i8>(
            super::G_GAME_MANAGER + game_manager::BOMBS_REMAINING,
            run.bombs,
        );
        space.write::<u16>(
            super::G_GAME_MANAGER + game_manager::CURRENT_POWER,
            run.power,
        );
        space.write::<i32>(
            super::G_ENEMY_MANAGER + super::enemy_manager::TIMELINE_TIME_CURRENT,
            run.script_frames,
        );
        space.write::<i32>(
            super::G_ENEMY_MANAGER + super::enemy_manager::ENEMY_COUNT,
            run.enemies,
        );
    }

    /// Says the run on screen is a replay being watched rather than one somebody is playing.
    ///
    /// Its own method rather than a field of [`Playing`], because it is not a number a stage is
    /// played with: it is what decides whether orb acts on the run at all.
    pub fn watching_a_replay(&self) {
        self.space()
            .write::<u32>(super::G_GAME_MANAGER + super::game_manager::IS_IN_REPLAY, 1);
    }

    /// What a snapshot covers, which orb reads out of the PE in a real game.
    pub fn data(&self) -> Range<usize> {
        DATA
    }
}
