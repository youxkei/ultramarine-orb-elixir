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

/// The game's static data, as the real one is: **one range**, `0x00476000..0x006e79fc`, which is what
/// orb reads out of the PE and writes in the log every run — `.data 0x00476000..0x006e79fc (2562556
/// bytes)`. This is the range a chapter is a copy of.
///
/// A set of windows around each global was tried and is why this is one range. Every structure the
/// reads reach — the enemy manager at 0x004b79c8, the cards at 0x005a5ff8, the lasers, the game
/// manager at 0x0069bca0, the supervisor, the player, the managers — is *inside* this on the machine,
/// so a snapshot there covers all of them. Windows covering each one separately left the rest outside
/// what `data()` reported, and a chapter restored in a scenario put back less than the same chapter
/// restores on the machine: measured as the script's clock coming back on hardware and not in a test,
/// which is a simulator disagreeing with the thing it stands for.
const DATA: Range<usize> = 0x0047_6000..0x006e_79fc;

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
        sim.space().map(DATA.start, DATA.len(), Kind::Private);
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

    /// Lays out the game's record of a spell card, with `card` the one a boss is on now.
    ///
    /// A record the game has written, which means the `CATK` magic at its head: `count_card_attempt`
    /// refuses one without it, because a zeroed record is a card this build does not have rather than
    /// a card nobody has tried. So a scenario that wants the attempt counted has to say the game had
    /// got as far as writing the record, which is what this says.
    pub fn spell_card(&self, card: i32, attempts: u16) {
        let space = self.space();
        space.write::<i32>(super::CURRENT_CARD, card);
        let record = super::CARD_HISTORY + card as usize * 0x40;
        space.write::<u32>(record, super::CATK_MAGIC);
        space.write::<u16>(record + super::CATK_ATTEMPTS, attempts);
    }

    /// How many attempts the game's record holds for `card`, which is the number the 完全無欠 ranking
    /// screen shows against it.
    pub fn card_attempts(&self, card: i32) -> u16 {
        self.space()
            .read::<u16>(super::CARD_HISTORY + card as usize * 0x40 + super::CATK_ATTEMPTS)
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

    /// The host this game is laid out in, for a scenario that needs more of it than the memory — the
    /// keyboard somebody presses at orb's own menus, or which window is in front.
    ///
    /// The same one, not another: a scenario pressing keys against one host while the chapters read
    /// the game through a second would be two processes agreeing about nothing.
    pub fn sim(&self) -> &Arc<Sim> {
        &self.sim
    }
}
