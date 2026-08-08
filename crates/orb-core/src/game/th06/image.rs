//! A th06 process image laid out in an address space, so that the real
//! [`Th06`] has something to read in a test with no game running.
//!
//! The space itself is `orb_sim::Space`, which this crate cannot link to: it is a dev-dependency and
//! the seam runs the other way round — the simulator implements `orb_api::Win` and knows nothing about
//! this.
//!
//! The regions are the game's own, at the game's own addresses, and they are laid out from the
//! same constants the reads use — so what this cannot catch is a constant that is wrong. A wrong
//! offset makes the writer and the reader wrong together, and only the real game says otherwise —
//! every one of these was read against 東方紅魔郷 1.02h running, with what the screen showed held
//! against what came back. What this does catch is everything built on
//! top of the offsets, which is where the code is.
//!
//! Zeroed, not filled with plausible values. A field the game has not written is zero in a
//! freshly committed page too, and a `Game` method that has to survive reading one is a method
//! whose pointer chases go through `read_committed` — which is the property worth holding it to.

use std::ops::Range;
use std::sync::Arc;

use orb_api::{Hwnd, Kind};
use orb_sim::{Sim, Sound, Space};

use crate::d3d8::Device;
use crate::game::RunStart;

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

/// Two of the blocks the game takes from its allocator: the boss of the fight on now, and the screen
/// a ranking is shown on. Both are reached only through a pointer the game keeps in [`DATA`], which
/// is what makes "no boss" the enemy manager holding none rather than the memory going away.
///
/// Outside `DATA` because that is where they are in the real game, and a snapshot of a laid-out game
/// should have the same work to do as one of a running game: `game_regions` finds these for itself
/// and copies them beside the static data, rather than getting them for free as fields of a global.
const BOSS: Range<usize> = 0x0300_0000..0x0300_1000;
const RANKING_SCREEN: Range<usize> = 0x0300_1000..0x0300_2000;
/// And the screen a run's result is shown on, which is reached the hardest of the three ways: through
/// the chain element it registered, by the callback that element holds. There is nothing in [`DATA`]
/// pointing at it at all — `chain_argument` is the whole of how orb finds it — so the element goes in
/// the chain's calc list with this block hanging off it.
const RESULT_SCREEN: Range<usize> = 0x0300_3000..0x0300_4000;
/// The chain element that screen registers, in a block of its own so that a snapshot has the same walk
/// to do over it as over a real one.
const RESULT_SCREEN_ELEM: Range<usize> = 0x0300_4000..0x0300_5000;
/// And a third, for the controller the game polls: the object at its foot and the vtable it is
/// reached through above it, since a COM interface is a pointer to a pointer to functions.
///
/// The functions themselves are not in here — they cannot be, being code — so what goes in the vtable
/// is the address of a real one, which is the same thing a Direct3D device made of Rust functions
/// does: see `Image::controller`.
const CONTROLLER: Range<usize> = 0x0300_2000..0x0300_3000;
const CONTROLLER_VTABLE: usize = CONTROLLER.start + 0x100;
/// And a fourth, for the keyboard device the game takes exclusively: the same shape as the controller, and
/// the two slots orb calls to let it go are the whole of why it is here — see
/// [`keyboard_device`](Image::keyboard_device).
const KEYBOARD: Range<usize> = 0x0300_5000..0x0300_6000;
const KEYBOARD_VTABLE: usize = KEYBOARD.start + 0x100;

/// The ending's own object and the chain element it registers, in blocks of their own and for the same
/// reason the result screen's two are: the object is nowhere in the game's static data, and
/// `chain_argument` finding it by the callback of the element it registered is the whole of how orb
/// reaches it.
const ENDING: Range<usize> = 0x0300_6000..0x0300_7000;
const ENDING_ELEM: Range<usize> = 0x0300_7000..0x0300_8000;
/// The two `.end` scripts an ending reads, as the addresses `Ending::LoadEndingFile` (0x4106d0) leaves
/// in the object: its own, and the staff roll's, which it reads over the one running.
///
/// Two addresses is the whole of what tells the ending from the roll — the scene stays 10 across both
/// and `isInEnding` stays set through both — and they differ because that function reads the new file
/// before it frees the one it replaces.
const ENDING_SCRIPT: usize = ENDING.start + 0x400;
const ROLL_SCRIPT: usize = ENDING.start + 0x800;

/// The two heap objects a track is streamed through: the streaming sound the sound player keeps, and
/// the wave file under it.
const MUSIC: Range<usize> = 0x0300_8000..0x0300_9000;
const STREAM: usize = MUSIC.start;
const WAVE: usize = MUSIC.start + 0x100;
/// And the array of buffer pointers the stream keeps, which is one long: the game's own is an array
/// however many buffers there are, and orb refuses a stream that says it has any number but one.
const BUFFERS: usize = MUSIC.start + 0x200;
/// And the vtable the streaming sound is reached through, which has to be inside [`CODE`]: a pointer
/// into the image is what orb takes for the difference between a live COM object and the stale one left
/// in a block the allocator did not scrub, and it asks that of the stream before it reads anything
/// through it.
const AUDIO_VTABLE: usize = CODE.start + 0x100;

/// The chain element a screen shake registers, in a block of its own so that orb's walk for it has the
/// same work to do as over a real one. The shake has no object of its own worth laying out: what orb
/// matches on is the element's callback, and what it does with the element is cut it.
const SHAKE_ELEM: Range<usize> = 0x0300_a000..0x0300_b000;

/// The one page of the anm manager orb reads: its array of 264 texture pointers, at the 0x1c110 they
/// start at. What it reads there is the array's bounds, as handles a restore must leave alone, and the
/// one slot the panel is painted from.
///
/// **A page and not the block, and the manager's own base is not mapped at all.** Nothing reads that
/// base — orb reads the *pointer* to it out of [`DATA`] and adds the offset — and every page laid out
/// here is a page this process's own heap cannot then be allowed to land in: `Sound::install` maps the
/// address of a real object, and `Space::map` panics where the two meet. Which the suite has seen
/// happen. So what is laid out is what is read, and it goes in the gap the blocks above leave rather
/// than past them.
const ANM_TEXTURES: Range<usize> = 0x0300_b000..0x0300_c000;
/// Which puts the manager itself below the space's own first block, where nothing reads it.
const ANM_MANAGER: usize = ANM_TEXTURES.start - super::anm_manager::TEXTURES;

/// And one page for the head of the stage's own `.std` file, reached through the pointer `g_Stage` keeps.
/// What orb reads out of it is one string at +0x290: the path of the track the stage names first, which
/// is what it hands back to the game to start that track again.
const STAGE_DATA: Range<usize> = 0x0300_c000..0x0300_d000;

/// How long one of that header's `songPaths` entries is — `char[128]`, inline, so the address of the
/// entry is the string.
const SONG_PATH_BYTES: usize = 128;

/// The replay manager, the replay it has loaded, and one record of inputs per stage of that replay:
/// three more blocks off the heap, reached through the pointer at `g_ReplayManager`.
const REPLAY_MANAGER: Range<usize> = 0x0300_9000..0x0300_a000;
const REPLAY: usize = REPLAY_MANAGER.start + 0x100;
const RECORDS: Range<usize> = 0x0301_0000..0x0302_0000;
/// How much of that block each stage's record gets, and how many entries fit in one.
const RECORD_BYTES: usize = (RECORDS.end - RECORDS.start) / super::replay_data::STAGES as usize;

/// A record of inputs as this file lays one out: how many entries it holds, how far playback has got
/// through it, and then the entries — the frame each change of what was held happened on, and what was
/// held from then on.
///
/// **This layout is not the game's**, and it is here rather than in whatever drives the game for the same
/// reason everything else is: orb reads none of it. What it reads is the *pointers* — the manager, the
/// replay, the per-stage entry — and what it does about the record is refuse to let a teardown write one
/// while it is being played back. So what a record has to be is something a write of that shape can be
/// seen in.
mod record {
    pub const ENTRIES: usize = 0x0;
    pub const PLAYBACK_AT: usize = 0x4;
    /// The seed the stage was drawn with, which is one of the eight fields
    /// `ReplayManager::AddedCallbackDemo` writes out of a stage's record — and the one two passes over
    /// one stage could not agree without: it goes in where a played stage would have had whatever the
    /// generator was left on.
    pub const SEED: usize = 0x8;
    pub const FIRST: usize = 0x10;
    /// A frame and a word, padded to keep the frames aligned.
    pub const STRIDE: usize = 0x8;
}

/// The frame number `ReplayManager::StopRecording` terminates a record at, which is 紅魔郷's own: no run
/// reaches it, so playback that walked into it would stand still for ever rather than end.
pub const RECORD_ENDS_AT: i32 = 9_999_999;

/// Where `clrd` is parsed into, as an offset in the game manager: 0x69ccd0, which is what the front
/// end lights `Extra Start` and its practice stages from. `pscr`'s destination is the 0x69cd30 after
/// it, so what lies between the two is `clrd`'s four records at 0x18 apiece.
///
/// Here rather than beside the offsets [`Th06`](super::Th06) reads through, because orb reads
/// neither of them: what reads them is the game's own front end, and this file is where the game's
/// part is laid out. Which of the three reads of the score file fills them is `MAIN_MENU_ADDED`'s
/// table.
const CLEARED: usize = 0x1030;
const CLEARED_BYTES: usize = 0x60;

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

/// The numbers a stage is played with that no [`State`](crate::game::State) holds: what the run has
/// scored, where its generator has got to, and where the player is standing.
///
/// Apart from [`Playing`] because these are not what a frame is judged by — they are the fields of
/// [`Reproduction`](crate::game::Reproduction), which is the line a run played back into a chapter is
/// held against. A resume that arrives with any of them different is a resume that has come out of
/// step, and these are what let a scenario say so.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Reproducing {
    pub score: u32,
    /// The generator itself, which is not [`Playing::seed`]: that one is the `GameManager`'s copy of
    /// what the stage was seeded with, and this is what the next number out of it will be made from.
    pub seed: u16,
    /// How many numbers have come out of it since the stage seeded it.
    pub randoms: u32,
    pub player: (f32, f32),
}

/// What a controller reports, in the terms the game's own read of one takes it in: which of its
/// buttons are down, where the stick's Y axis is in the ±1000 the game gave every axis, and which way
/// the hat points in hundredths of a degree.
///
/// The buttons are a mask in the numbering the game's own mapping names them by, which is what
/// `SetButtonFromDirectInputJoystate` indexes the device's array with.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Pushed {
    pub buttons: u32,
    pub y: i32,
    /// A full circle is 36000; anything above it is the hat at rest, which is what a real one reports.
    pub hat: u32,
}

impl Pushed {
    /// Nothing pushed, with the hat where a device that has one leaves it when it is not being pushed.
    pub fn none() -> Self {
        Self {
            hat: HAT_AT_REST,
            ..Self::default()
        }
    }

    /// That button down, and nothing else.
    pub fn button(button: i16) -> Self {
        Self {
            buttons: 1 << button,
            ..Self::none()
        }
    }
}

/// What a hat reports while nobody is pushing it: past a full circle, which is what the game's own
/// read takes as no direction at all.
pub const HAT_AT_REST: u32 = 0xffff;

/// How much of a `DIJOYSTATE2` the game asks its controller for, which is the whole of one: its size
/// is what `GetDeviceState` is told, and a device that wrote less would be one the game reads past.
pub const JOY_STATE_BYTES: usize = 272;

/// Writes `pushed` into the `DIJOYSTATE2` a controller is asked to fill.
///
/// The game's own struct, at the game's own offsets: six axes, two sliders, four hats, and then a byte
/// per button with the top bit set on the ones that are down. Here rather than in whatever is standing
/// in for the device, for the same reason every other offset in this file is here.
///
/// # Safety
/// `state` must be [`JOY_STATE_BYTES`] of writable memory — which is what the game's own read hands
/// its device, a buffer of its own on the stack.
pub unsafe fn joy_state(state: *mut u8, pushed: Pushed) {
    /// Where each half of it starts, in bytes: the axes lead, the two sliders and the four hats follow,
    /// and the buttons are after those.
    const HATS: usize = 6 * 4 + 2 * 4;
    const BUTTONS: usize = HATS + 4 * 4;
    unsafe {
        std::ptr::write_bytes(state, 0, JOY_STATE_BYTES);
        // The Y axis, which is the second of the six and is measured downwards.
        state
            .add(super::AXIS_Y * size_of::<i32>())
            .cast::<i32>()
            .write_unaligned(pushed.y);
        state.add(HATS).cast::<u32>().write_unaligned(pushed.hat);
        for button in 0..u32::BITS {
            if pushed.buttons & (1 << button) != 0 {
                state.add(BUTTONS + button as usize).write(0x80);
            }
        }
    }
}

/// Which of the game's buttons is which, as its own configuration names them by number — the mapping
/// `Controller::GetControllerInput` reads every frame and the one orb reads a pad's buttons through.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mapping {
    pub shoot: i16,
    pub bomb: i16,
    pub menu: i16,
    pub up: i16,
    pub down: i16,
    /// How far the stick has to go before the game counts it as pushed, in the ±1000 it gave its axes.
    pub y_axis: i16,
}

/// One of the game's own screens, as its front end's `gameState` names them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    /// The title menu, whose cursor is one of [`item`]'s.
    Title,
    /// The shot type select, where the cursor is the shot itself.
    ShotType,
    /// The ranking, which is what orb asks the front end for on the way out of a run so that what
    /// the run counted is written — see [`Game::show_ranking`](crate::game::Game::show_ranking).
    Ranking,
    /// Any other of its screens, as the number it is: the difficulty and the character select are
    /// two, and orb has nothing to ask over either.
    Other(i32),
}

/// Which item of the title menu a cursor is on, for the two orb has a question about — and for the
/// one the score file decides, which is [`EXTRA`].
pub mod item {
    pub const GAME_START: i32 = super::super::TITLE_ITEM_START;
    pub const EXTRA: i32 = super::super::TITLE_ITEM_EXTRA;
    pub const SCORE: i32 = super::super::TITLE_ITEM_SCORE;
}

/// Where the game's own front end is: which screen, what its cursor is on, and how many frames it
/// has been there.
///
/// The last of those is `stateTimer`, and it is here because it is what decides whether the screen
/// would act on a press at all — see [`acts_on_a_press`](FrontEnd::acts_on_a_press).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrontEnd {
    pub screen: Screen,
    pub cursor: i32,
    pub frames: i32,
}

impl FrontEnd {
    /// Whether this screen is past the frames it ignores its own decide for.
    ///
    /// Here rather than in whatever is driving the game, because the numbers are the game's own —
    /// see `MENU_TITLE_GRACE_FRAMES` — and orb holds a press back over exactly these frames.
    pub fn acts_on_a_press(&self) -> bool {
        self.frames
            >= match self.screen {
                Screen::Title => super::MENU_TITLE_GRACE_FRAMES,
                Screen::ShotType => super::MENU_SHOT_TYPE_GRACE_FRAMES,
                // Neither has a decide of its own that orb has anything to say about.
                _ => return false,
            }
    }
}

/// Which of its own scenes the game is running.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    FrontEnd,
    /// A stage being played.
    Playing,
    /// The screen a run ends into, which is what tells a run that finished from one that was left —
    /// see [`Game::run_finished`](crate::game::Game::run_finished).
    Result,
    /// The ranking, which is a scene of the supervisor's as well as a state of the front end's:
    /// asking for it is one and building it is the other.
    Ranking,
    /// The scene a stage transition passes through, which is the game rebuilding its game manager for
    /// the stage after: one frame of it, `f44096 scene=3 stage=2` and then `f44097 scene=2 stage=3` a
    /// quarter of a second later, because the next stage is built inside that frame.
    Rebuilding,
    /// The ending, which is where a cleared run goes before its result screen. Named because
    /// [`read_state`](crate::game::Game::read_state) reads it: a frame of an ending is not a frame of a
    /// run somebody is playing.
    Ending,
    Other(i32),
}

/// Its supervisor's two states: what is running, and what has been asked for.
///
/// Both, because every one-frame window orb watches for is the two disagreeing —
/// [`Game::run_chosen`](crate::game::Game::run_chosen) is that frame and nothing else is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Supervising {
    pub running: Scene,
    pub wanted: Scene,
}

/// The boss a fight is against: what is left of it, and how long the attack it is on has been
/// running.
///
/// That second count going back to nothing is what says the fight has moved to its next attack,
/// spell card or not, which is where a chapter begins.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Boss {
    pub life: i32,
    pub attack_frames: i32,
}

/// A track, as the three numbers of its header that tell one from another: how long its wave file is,
/// and where it loops — see `Th06::music_identity`, which is what reads them.
///
/// A scenario's own numbers. Which they are does not matter and what matters is that two tracks are two
/// sets of them: a snapshot's music belongs to the track that was playing, and a track that has changed
/// under it is one to start again rather than copy back.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Track {
    pub length: u32,
    pub loop_start: u32,
    pub loop_end: u32,
}

/// What the player is doing, which is what says whether a frame is one a chapter may begin on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Player {
    Normal,
    /// Coming back after a death.
    Spawning,
    /// Hit, and the frames of dying that follow.
    Dying,
    /// The seconds after a bomb or a respawn, where nothing can hit the player.
    Invulnerable,
}

/// The game's memory, laid out. Reading it goes through [`enter`](Self::enter).
pub struct Image {
    sim: Arc<Sim>,
}

impl Image {
    /// In a host whose non-determinism is drawn from `seed` — the wake delays and the compositor's
    /// spikes — which a scenario names in its assertions so that a failure can be replayed.
    ///
    /// The seed is always said, there being no second constructor that leaves it out: one that did was
    /// `Sim::seeded(0)` under another name, and a test which does not care what the host does still has
    /// to be readable as having chosen.
    pub fn laid_out_seeded(seed: u64) -> Self {
        Self::mapped(Arc::new(Sim::seeded(seed)))
    }

    fn mapped(sim: Arc<Sim>) -> Self {
        sim.space().map(DATA.start, DATA.len(), Kind::Private);
        sim.space().map(CODE.start, CODE.len(), Kind::Image);
        sim.space().map(BOSS.start, BOSS.len(), Kind::Private);
        sim.space()
            .map(RANKING_SCREEN.start, RANKING_SCREEN.len(), Kind::Private);
        sim.space()
            .map(CONTROLLER.start, CONTROLLER.len(), Kind::Private);
        sim.space()
            .map(RESULT_SCREEN.start, RESULT_SCREEN.len(), Kind::Private);
        sim.space().map(
            RESULT_SCREEN_ELEM.start,
            RESULT_SCREEN_ELEM.len(),
            Kind::Private,
        );
        sim.space()
            .map(KEYBOARD.start, KEYBOARD.len(), Kind::Private);
        sim.space().map(ENDING.start, ENDING.len(), Kind::Private);
        sim.space()
            .map(ENDING_ELEM.start, ENDING_ELEM.len(), Kind::Private);
        sim.space().map(MUSIC.start, MUSIC.len(), Kind::Private);
        sim.space()
            .map(REPLAY_MANAGER.start, REPLAY_MANAGER.len(), Kind::Private);
        sim.space().map(RECORDS.start, RECORDS.len(), Kind::Private);
        sim.space()
            .map(SHAKE_ELEM.start, SHAKE_ELEM.len(), Kind::Private);
        sim.space()
            .map(ANM_TEXTURES.start, ANM_TEXTURES.len(), Kind::Private);
        sim.space()
            .map(STAGE_DATA.start, STAGE_DATA.len(), Kind::Private);
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

    /// The numbers of [`Reproducing`], which a stage moves as it is played.
    pub fn reproducing(&self, run: Reproducing) {
        use super::game_manager;
        let space = self.space();
        // Both, because the panel's number chases the run's and a score put in one alone would show
        // counting up to it.
        space.write::<u32>(super::G_GAME_MANAGER + game_manager::GUI_SCORE, run.score);
        space.write::<u32>(super::G_GAME_MANAGER + game_manager::SCORE, run.score);
        space.write::<u16>(super::G_RNG, run.seed);
        space.write::<u32>(super::G_RNG + super::RNG_GENERATION_COUNT, run.randoms);
        space.write::<f32>(
            super::G_PLAYER + super::player::POSITION_CENTER,
            run.player.0,
        );
        space.write::<f32>(
            super::G_PLAYER + super::player::POSITION_CENTER + size_of::<f32>(),
            run.player.1,
        );
    }

    /// What they are now, which is what the next frame moves on from.
    pub fn reproducing_now(&self) -> Reproducing {
        use super::game_manager;
        let space = self.space();
        Reproducing {
            score: space.read(super::G_GAME_MANAGER + game_manager::GUI_SCORE),
            seed: space.read(super::G_RNG),
            randoms: space.read(super::G_RNG + super::RNG_GENERATION_COUNT),
            player: (
                space.read(super::G_PLAYER + super::player::POSITION_CENTER),
                space.read(super::G_PLAYER + super::player::POSITION_CENTER + size_of::<f32>()),
            ),
        }
    }

    /// Hands over the game's own `Chain::Cut`: code is the one thing an address space laid out by hand
    /// cannot hold, and a shake still running at a stage move is taken down through that call.
    pub fn hands_over_chain_cut(&self, address: usize) {
        super::set_chain_cut(address);
    }

    /// And its own `SoundPlayer::StopBGM` and `Supervisor::PlayAudio`, which are the two calls a restore
    /// makes into the game where the track has been replaced since the chapter was taken: the sound is
    /// torn down through the first and started again through the second.
    ///
    /// Both together, because neither is reached without the other on that path — a scenario that handed
    /// over one would find the other's address under it.
    pub fn hands_over_the_music_calls(&self, stop_bgm: usize, play_audio: usize) {
        super::set_stop_bgm(stop_bgm);
        super::set_play_audio(play_audio);
    }

    /// `g_Stage.stdData->songPaths[0]`: the head of the stage's own `.std` file, with the path of the
    /// track it names first written into it.
    ///
    /// Which is the string orb hands back to `PlayAudio` — so what it has to be is a path that reads as
    /// one, and `Th06::stage_song` is what decides that.
    ///
    /// # Panics
    /// Where the path does not fit the `char[128]` the game keeps it in, with room for its terminator.
    pub fn names_its_song(&self, path: &str) {
        assert!(
            path.len() < SONG_PATH_BYTES,
            "{path:?} is longer than the {SONG_PATH_BYTES} bytes the game keeps a song path in",
        );
        let mut entry = [0u8; SONG_PATH_BYTES];
        entry[..path.len()].copy_from_slice(path.as_bytes());
        let space = self.space();
        space.write::<usize>(super::G_STAGE + super::stage::STD_DATA, STAGE_DATA.start);
        assert!(
            super::stage_header::SONG_PATHS + SONG_PATH_BYTES <= STAGE_DATA.len(),
            "the song path does not fit the one page the stage's data is laid out in",
        );
        space.write::<[u8; SONG_PATH_BYTES]>(
            STAGE_DATA.start + super::stage_header::SONG_PATHS,
            entry,
        );
    }

    /// `ScreenEffect::ShakeScreen` registered as a job of the chain's, which is what a bomb leaves
    /// running: what orb matches on is the callback, and what it does with the element is cut it.
    pub fn shakes_the_screen(&self) {
        let space = self.space();
        let elem = SHAKE_ELEM.start;
        space.write::<usize>(elem + super::chain_elem::CALLBACK, super::SHAKE_SCREEN);
        space.write::<usize>(elem + super::chain_elem::ARG, elem);
        let head = super::G_CHAIN + super::chain_elem::NEXT;
        let after: usize = space.read(head);
        space.write::<usize>(elem + super::chain_elem::NEXT, after);
        space.write::<usize>(head, elem);
    }

    /// Whether that job is still in the chain, which is what says a shake is still running.
    pub fn shaking_the_screen(&self) -> bool {
        let mut at: usize = self.space().read(super::G_CHAIN + super::chain_elem::NEXT);
        while at != 0 {
            if at == SHAKE_ELEM.start {
                return true;
            }
            at = self.space().read(at + super::chain_elem::NEXT);
        }
        false
    }

    /// And the shake taking itself down, which it does on the frame its own frames run out — the frame a
    /// shake cut early never reaches.
    pub fn cuts_the_shake_from_the_chain(&self) {
        self.cuts_from_the_chain(SHAKE_ELEM.start);
    }

    /// `Chain::Cut`: the element unlinked from the calc chain, which is what the game's own call does and
    /// what a game laid out by hand has to do in its place — see
    /// [`hands_over_chain_cut`](Self::hands_over_chain_cut).
    pub fn cuts_from_the_chain(&self, elem: usize) {
        let space = self.space();
        let mut at = super::G_CHAIN + super::chain_elem::NEXT;
        loop {
            let next: usize = space.read(at);
            if next == 0 {
                return;
            }
            if next == elem {
                space.write::<usize>(at, space.read::<usize>(elem + super::chain_elem::NEXT));
                space.write::<usize>(elem + super::chain_elem::NEXT, 0);
                space.write::<usize>(elem + super::chain_elem::CALLBACK, 0);
                return;
            }
            at = next + super::chain_elem::NEXT;
        }
    }

    /// The arcade region, which is not the box the player is held inside: it is what a screen shake
    /// writes every frame from the generator, and what `Player::AddedCallback` measures a stage's first
    /// position from.
    ///
    /// Written once where the game sets its screen up and by nothing per stage, which is the whole reason
    /// a shake can reach the stage after the one that started it: nothing on the way into a stage puts it
    /// back.
    pub fn sets_the_arcade_region(&self, top_left: (f32, f32), size: (f32, f32)) {
        let space = self.space();
        space.write::<[f32; 2]>(
            super::G_GAME_MANAGER + super::game_manager::ARCADE_REGION_TOP_LEFT,
            [top_left.0, top_left.1],
        );
        space.write::<[f32; 2]>(
            super::G_GAME_MANAGER + super::game_manager::ARCADE_REGION_SIZE,
            [size.0, size.1],
        );
    }

    /// How big it is now, which is what a stage's first position comes out of.
    pub fn arcade_region_size(&self) -> (f32, f32) {
        let size: [f32; 2] = self
            .space()
            .read(super::G_GAME_MANAGER + super::game_manager::ARCADE_REGION_SIZE);
        (size[0], size[1])
    }

    /// How many extra lives the run's score has paid for, which is the count a stage's own loop only
    /// ever raises: it can bring the number up to what the score has earned and cannot bring it down
    /// from what a later stage reached, which is why starting a replay at a stage writes it.
    pub fn extra_lives(&self) -> i8 {
        self.space()
            .read(super::G_GAME_MANAGER + super::game_manager::EXTRA_LIVES)
    }

    pub fn set_extra_lives(&self, lives: i8) {
        self.space().write::<i8>(
            super::G_GAME_MANAGER + super::game_manager::EXTRA_LIVES,
            lives,
        );
    }

    /// The box the player is held inside, which a stage's build puts in place and nothing else moves.
    pub fn play_field(&self, top: f32, height: f32) {
        use super::game_manager;
        let space = self.space();
        let y = size_of::<f32>();
        space.write::<f32>(
            super::G_GAME_MANAGER + game_manager::PLAYER_AREA_TOP_LEFT + y,
            top,
        );
        space.write::<f32>(
            super::G_GAME_MANAGER + game_manager::PLAYER_AREA_SIZE + y,
            height,
        );
    }

    /// The numbers of [`Playing`] as they are now.
    ///
    /// What makes a game driving itself out of this image a game with one state rather than two: the
    /// frame it plays next is worked out from the memory it is in, so a chapter put back underneath it
    /// takes the run back with it.
    pub fn playing_now(&self) -> Playing {
        use super::{enemy_manager, game_manager};
        let space = self.space();
        Playing {
            stage: space.read::<i32>(super::G_GAME_MANAGER + game_manager::CURRENT_STAGE) - 1,
            difficulty: space.read(super::G_GAME_MANAGER + game_manager::DIFFICULTY),
            frames: space.read(super::G_GAME_MANAGER + game_manager::GAME_FRAMES),
            script_frames: space
                .read(super::G_ENEMY_MANAGER + enemy_manager::TIMELINE_TIME_CURRENT),
            seed: space.read(super::G_GAME_MANAGER + game_manager::RANDOM_SEED),
            deaths: space.read(super::G_GAME_MANAGER + game_manager::DEATHS),
            lives: space.read(super::G_GAME_MANAGER + game_manager::LIVES_REMAINING),
            bombs: space.read(super::G_GAME_MANAGER + game_manager::BOMBS_REMAINING),
            power: space.read(super::G_GAME_MANAGER + game_manager::CURRENT_POWER),
            enemies: space.read(super::G_ENEMY_MANAGER + enemy_manager::ENEMY_COUNT),
        }
    }

    /// Which run the front end has been answered for: the difficulty, the character and the shot, and
    /// the stage as its own screens count it — one less than a stage in progress, since building one
    /// is what raises the number.
    pub fn chose(&self, run: &RunStart) {
        use super::game_manager;
        let space = self.space();
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::DIFFICULTY,
            run.difficulty,
        );
        space.write::<u8>(
            super::G_GAME_MANAGER + game_manager::CHARACTER,
            run.character as u8,
        );
        space.write::<u8>(
            super::G_GAME_MANAGER + game_manager::SHOT_TYPE,
            run.shot_type as u8,
        );
        space.write::<u8>(
            super::G_GAME_MANAGER + game_manager::IS_IN_PRACTICE_MODE,
            u8::from(run.practice),
        );
        space.write::<i32>(
            super::G_GAME_MANAGER + game_manager::CURRENT_STAGE,
            run.stage,
        );
    }

    /// Raises the stage number the way the callback that puts a stage's numbers in place does, and
    /// answers the stage that came up.
    ///
    /// Its own step because orb reads a run's stage back through the same `-1` — see
    /// `game_manager::CURRENT_STAGE` — so a game that built a stage without doing this would be a
    /// stage off in everything above it.
    pub fn stage_built(&self) -> i32 {
        use super::game_manager;
        let at = super::G_GAME_MANAGER + game_manager::CURRENT_STAGE;
        let stage: i32 = self.space().read(at);
        self.space().write::<i32>(at, stage + 1);
        stage
    }

    /// Puts the front end's own three fields where `front` says.
    ///
    /// Not the supervisor with them: whether the front end is what is *running* is the supervisor's to
    /// say — see [`supervising`](Self::supervising) — and the frames field is a timer any screen of the
    /// game can be counting, since a game laid out by hand has one screen at a time.
    pub fn front_end(&self, front: FrontEnd) {
        use super::main_menu;
        let space = self.space();
        space.write::<i32>(
            super::G_MAIN_MENU + main_menu::GAME_STATE,
            screen_of(front.screen),
        );
        space.write::<i32>(super::G_MAIN_MENU + main_menu::CURSOR, front.cursor);
        space.write::<i32>(super::G_MAIN_MENU + main_menu::STATE_TIMER, front.frames);
    }

    /// What the front end's own three fields hold, whatever the supervisor says is running.
    ///
    /// Not `None` off the front end, because the game's is a global that keeps whatever it was left
    /// holding — which is the very thing `menu_pointed_at` guards against and so has to be readable
    /// here. [`scene`](Self::scene) is what says whether the front end is what is running.
    pub fn front_end_now(&self) -> FrontEnd {
        use super::main_menu;
        let space = self.space();
        FrontEnd {
            screen: screen_from(space.read(super::G_MAIN_MENU + main_menu::GAME_STATE)),
            cursor: space.read(super::G_MAIN_MENU + main_menu::CURSOR),
            frames: space.read(super::G_MAIN_MENU + main_menu::STATE_TIMER),
        }
    }

    pub fn supervising(&self, supervisor: Supervising) {
        let space = self.space();
        space.write::<i32>(
            super::G_SUPERVISOR + super::supervisor::CUR_STATE,
            scene_of(supervisor.running),
        );
        space.write::<i32>(
            super::G_SUPERVISOR + super::supervisor::WANTED_STATE,
            scene_of(supervisor.wanted),
        );
    }

    pub fn supervising_now(&self) -> Supervising {
        let space = self.space();
        Supervising {
            running: scene_from(space.read(super::G_SUPERVISOR + super::supervisor::CUR_STATE)),
            wanted: scene_from(space.read(super::G_SUPERVISOR + super::supervisor::WANTED_STATE)),
        }
    }

    /// What is running, which is the half of [`supervising_now`](Self::supervising_now) that most
    /// questions are about.
    pub fn scene(&self) -> Scene {
        self.supervising_now().running
    }

    /// The word the game's own read handed to this frame's update, which is what every one of its
    /// `WAS_PRESSED` tests is worked out against — see
    /// [`Game::swallow_input`](crate::game::Game::swallow_input).
    pub fn input(&self, word: u16) {
        self.space().write::<u16>(super::G_CUR_FRAME_INPUT, word);
    }

    /// And what the frame before was handed, which is what it still holds until the next update
    /// assigns over it.
    pub fn input_now(&self) -> u16 {
        self.space().read(super::G_CUR_FRAME_INPUT)
    }

    /// The boss of the fight on now, or none.
    ///
    /// Both halves of what orb reads: the pointer the enemy manager keeps, which is where the life
    /// and the attack's clock are read through, and the panel's own flag, which is what
    /// `boss_present` is.
    pub fn boss(&self, boss: Option<Boss>) {
        use super::{enemy, enemy_manager, gui};
        let space = self.space();
        space.write::<u8>(super::G_GUI + gui::BOSS_PRESENT, u8::from(boss.is_some()));
        space.write::<usize>(
            super::G_ENEMY_MANAGER + enemy_manager::BOSSES,
            boss.map_or(0, |_| BOSS.start),
        );
        if let Some(boss) = boss {
            space.write::<i32>(BOSS.start + enemy::LIFE, boss.life);
            space.write::<i32>(BOSS.start + enemy::BOSS_TIMER_CURRENT, boss.attack_frames);
        }
    }

    /// The boss of that fight as the game's memory holds it now, or none.
    ///
    /// Read back rather than kept beside the memory, so that a game moving an attack's clock on moves
    /// the clock a chapter restored underneath it takes back.
    pub fn boss_now(&self) -> Option<Boss> {
        use super::{enemy, enemy_manager};
        let space = self.space();
        let at = space.read::<usize>(super::G_ENEMY_MANAGER + enemy_manager::BOSSES);
        (at != 0).then(|| Boss {
            life: space.read(at + enemy::LIFE),
            attack_frames: space.read(at + enemy::BOSS_TIMER_CURRENT),
        })
    }

    /// The spell card that boss is on now, or none.
    ///
    /// Which card it is goes in both of the places the game keeps it: the enemy manager's, which is
    /// what a `State` reads, and the one the count of attempts is indexed by — see
    /// [`card_record`](Self::card_record).
    pub fn card(&self, card: Option<i32>) {
        use super::enemy_manager;
        let space = self.space();
        space.write::<i32>(
            super::G_ENEMY_MANAGER + enemy_manager::SPELLCARD_IS_ACTIVE,
            i32::from(card.is_some()),
        );
        if let Some(card) = card {
            space.write::<u32>(
                super::G_ENEMY_MANAGER + enemy_manager::SPELLCARD_IDX,
                card as u32,
            );
            space.write::<i32>(super::CURRENT_CARD, card);
        }
    }

    /// Lays out the game's own record of a spell card.
    ///
    /// A record the game has written, which means the `CATK` magic at its head: `count_card_attempt`
    /// refuses one without it, because a zeroed record is a card this build does not have rather than
    /// a card nobody has tried. So a scenario that wants the attempt counted has to say the game had
    /// got as far as writing the record, which is what this says.
    pub fn card_record(&self, card: i32, attempts: u16) {
        let space = self.space();
        let record = super::CARD_HISTORY + card as usize * 0x40;
        space.write::<u32>(record, super::CATK_MAGIC);
        space.write::<u16>(record + super::CATK_ATTEMPTS, attempts);
    }

    /// `clrd`'s parse at 0x42b502: the destination cleared before the chunk is looked for — four
    /// records memset at 0x42b535 — and then whatever the read found written into it.
    ///
    /// **The clear is the half worth laying out.** A read that failed leaves the front end nothing to
    /// light, so a file that is not there locks what an earlier one had earned rather than leaving it
    /// as it was — which is why the one read the menu's items come out of has to be pointed at the
    /// game's own file whichever mode orb is in. `catk`'s parse at 0x42b466 has no clear of its own,
    /// which is the difference [`set_captures`](crate::game::Game::set_captures) is read against.
    pub fn parses_the_unlocks(&self, chunk: &[u8]) {
        let space = self.space();
        let at = super::G_GAME_MANAGER + CLEARED;
        space.fill_bytes(at, 0, CLEARED_BYTES);
        space.write_bytes(at, &chunk[..chunk.len().min(CLEARED_BYTES)]);
    }

    /// What that parse left the front end to light its items from.
    ///
    /// Bytes rather than records: what one holds per shot is not something orb reads, and the whole of
    /// what anything above asks is whether the read left the menu anything at all.
    pub fn unlocks(&self) -> Vec<u8> {
        self.space()
            .read_bytes(super::G_GAME_MANAGER + CLEARED, CLEARED_BYTES)
    }

    /// What the player is doing.
    pub fn player(&self, player: Player) {
        self.space().write::<i8>(
            super::G_PLAYER + super::player::PLAYER_STATE,
            match player {
                Player::Normal => super::PLAYER_NORMAL,
                Player::Spawning => super::PLAYER_SPAWNING,
                Player::Dying => super::PLAYER_DEAD,
                Player::Invulnerable => super::PLAYER_INVULNERABLE,
            },
        );
    }

    /// What the player is doing, read back — which is how a game that models `Player::OnUpdate` knows
    /// whether this update's hit test has a player it can kill.
    pub fn player_now(&self) -> Player {
        match self
            .space()
            .read::<i8>(super::G_PLAYER + super::player::PLAYER_STATE)
        {
            super::PLAYER_SPAWNING => Player::Spawning,
            super::PLAYER_DEAD => Player::Dying,
            super::PLAYER_INVULNERABLE => Player::Invulnerable,
            _ => Player::Normal,
        }
    }

    /// The frames of invulnerability left, which `Player::OnUpdate` ticks down and puts the state back
    /// to normal at the end of.
    ///
    /// Apart from [`player`](Self::player) because the two are what a restore has to write *together*:
    /// a state written with the frames left where the last respawn put them is a player whose
    /// invulnerability expires inside the update it was written for. See
    /// `Th06::make_invulnerable`.
    pub fn invulnerable_frames(&self) -> i32 {
        self.space()
            .read(super::G_PLAYER + super::player::INVULNERABLE_FRAMES)
    }

    pub fn set_invulnerable_frames(&self, frames: i32) {
        self.space()
            .write::<i32>(super::G_PLAYER + super::player::INVULNERABLE_FRAMES, frames);
    }

    /// `Gui::RegisterChain`: the static element `g_Gui`'s draw job is, linked into the chain's draw
    /// list — which is what `Th06::draws_lives_row` walks for.
    ///
    /// The element itself and nothing else, since that is the whole of what the walk asks: whether
    /// 0x69bc5c is in that list. What it is *for* is that a run being left is not a run whose panel has
    /// gone — the game paints that row once more after the run ends, and the mark has to reach it.
    pub fn registers_gui_in_the_draw_chain(&self) {
        let space = self.space();
        let head = super::G_CHAIN + super::CHAIN_DRAW_LIST + super::chain_elem::NEXT;
        let after: usize = space.read(head);
        space.write::<usize>(super::GUI_DRAW_ELEM + super::chain_elem::NEXT, after);
        space.write::<usize>(head, super::GUI_DRAW_ELEM);
    }

    /// And `Chain::Cut` taking it out again, which is what the front end drawing its own screen does.
    pub fn cuts_gui_from_the_draw_chain(&self) {
        let space = self.space();
        let head = super::G_CHAIN + super::CHAIN_DRAW_LIST + super::chain_elem::NEXT;
        if space.read::<usize>(head) == super::GUI_DRAW_ELEM {
            let after: usize = space.read(super::GUI_DRAW_ELEM + super::chain_elem::NEXT);
            space.write::<usize>(head, after);
        }
        space.write::<usize>(super::GUI_DRAW_ELEM + super::chain_elem::NEXT, 0);
    }

    /// Whether it is in there, for a scenario reading the same list orb reads.
    pub fn gui_in_the_draw_chain(&self) -> bool {
        self.space()
            .read::<usize>(super::G_CHAIN + super::CHAIN_DRAW_LIST + super::chain_elem::NEXT)
            == super::GUI_DRAW_ELEM
    }

    /// The panel being laid over a stage's first frames, which sets all five of `GuiFlags`' two-bit
    /// fields to 2 itself — 0x41a2b6, inside the vm's script, until it reaches `ExitHide` 250 frames in.
    ///
    /// Which is why the field orb writes decides nothing over those frames, and the reason one of the
    /// two fields in `repaint_lives_row`'s ask was tried and left: see that method.
    pub fn repaints_the_whole_panel(&self) {
        self.space()
            .write::<u32>(super::G_GUI + super::gui::FLAGS, 0b10_10_10_10_10);
    }

    /// `GuiFlags` as it stands, which is what says whether the game will repaint the row the lives are
    /// counted in — the lowest pair.
    pub fn gui_flags(&self) -> u32 {
        self.space().read(super::G_GUI + super::gui::FLAGS)
    }

    /// And the word after `Gui::OnDraw` has spent what it drew from it, which is one off each field.
    pub fn sets_gui_flags(&self, flags: u32) {
        self.space()
            .write::<u32>(super::G_GUI + super::gui::FLAGS, flags);
    }

    /// The device orb draws its overlay through and the window it reads the keyboard against, which
    /// are what the game has once it has finished setting Direct3D up.
    pub fn shows_through(&self, device: *mut Device, window: Hwnd) {
        let space = self.space();
        space.write::<*mut Device>(super::G_SUPERVISOR + super::supervisor::D3D_DEVICE, device);
        space.write::<Hwnd>(
            super::G_SUPERVISOR + super::supervisor::HWND_GAME_WINDOW,
            window,
        );
    }

    /// The controller the game polls, reached the way a COM interface is: a pointer to the object, and
    /// the object's own first word pointing at the vtable.
    ///
    /// The three functions are the ones the game's read calls through — poll, acquire, and the read
    /// itself — and they are addresses of real ones, because code is the one thing a laid-out address
    /// space cannot hold. Which is the same shape as the Direct3D device orb draws through.
    pub fn controller(&self, poll: usize, acquire: usize, read_state: usize) {
        let space = self.space();
        space.write::<usize>(super::G_CONTROLLER, CONTROLLER.start);
        space.write::<usize>(CONTROLLER.start, CONTROLLER_VTABLE);
        for (slot, function) in [
            (super::dinput_device::POLL, poll),
            (super::dinput_device::ACQUIRE, acquire),
            (super::dinput_device::GET_DEVICE_STATE, read_state),
        ] {
            space.write::<usize>(CONTROLLER_VTABLE + slot * size_of::<usize>(), function);
        }
    }

    /// And no controller at all, which is what the game has where its enumeration found none — the
    /// branch its own read asks winmm on.
    pub fn no_controller(&self) {
        self.space().write::<usize>(super::G_CONTROLLER, 0);
    }

    /// Whether it has one, which is what the game's own read branches on: with a controller it polls that
    /// through DirectInput and never asks winmm at all.
    pub fn holds_a_controller(&self) -> bool {
        self.space().read::<usize>(super::G_CONTROLLER) != 0
    }

    /// Whether the game's own configuration says it runs in a window, which is the setting orb overrules
    /// before the window is made: a game that has taken the display exclusively has no window to resize,
    /// and by the time anything of orb's runs per frame the device already exists.
    ///
    /// Zero to begin with, which is a game configured for full screen — the case the override is for.
    pub fn windowed(&self) -> bool {
        self.space()
            .read::<u8>(super::G_SUPERVISOR + super::supervisor::CFG_WINDOWED)
            != 0
    }

    pub fn set_windowed(&self, windowed: bool) {
        self.space().write::<u8>(
            super::G_SUPERVISOR + super::supervisor::CFG_WINDOWED,
            windowed.into(),
        );
    }

    /// `AnmManager::LoadAnm` having put `data/front.anm`'s sheet in the manager's own array — slot 13,
    /// `ANM_FILE_FRONT` — with the manager where the game keeps the pointer to it.
    ///
    /// **A plain address and not an object**, which is the difference from the controller and the sound
    /// buffer: orb never reads a word through this pointer. It binds it — `SetTexture` and then a quad —
    /// and hands the array's bounds to a snapshot as a range to leave alone, so what it has to be is a
    /// number that comes back out as it went in.
    pub fn loads_the_front_sheet(&self, texture: usize) {
        let space = self.space();
        space.write::<usize>(super::G_ANM_MANAGER, ANM_MANAGER);
        space.write::<usize>(self.front_sheet_at(), texture);
    }

    /// What is in that slot now, which is what a restore must not have put back: a handle to a texture
    /// the game may have released since is one the game would release a second time.
    pub fn front_sheet(&self) -> usize {
        self.space().read(self.front_sheet_at())
    }

    fn front_sheet_at(&self) -> usize {
        ANM_TEXTURES.start + super::FRONT_TEXTURE * size_of::<usize>()
    }

    /// A word of that manager just past its texture array, which orb reads nothing of.
    ///
    /// Here for the other half of the claim about the handles: the array is the only part of the block a
    /// restore leaves alone, so a scenario asking whether it was left alone has to be able to ask whether
    /// anything else in the same page came back.
    pub fn anm_manager_word(&self) -> usize {
        self.space().read(self.beside_the_textures())
    }

    pub fn set_anm_manager_word(&self, word: usize) {
        self.space()
            .write::<usize>(self.beside_the_textures(), word);
    }

    fn beside_the_textures(&self) -> usize {
        ANM_TEXTURES.start + super::anm_manager::TEXTURE_COUNT * size_of::<usize>()
    }

    /// The keyboard device the game takes `DISCL_EXCLUSIVE | DISCL_FOREGROUND`, reached the same way the
    /// controller is: a pointer to the object, and the object's first word pointing at the vtable.
    ///
    /// `unacquire` and `release` are the two slots orb calls to let it go — `Th06::take_sent_keys` — and
    /// `acquire` the one it calls to get it back after the window has been away. Addresses of real
    /// functions, because code is the one thing a laid-out address space cannot hold.
    pub fn keyboard_device(&self, acquire: usize, unacquire: usize, release: usize) {
        let space = self.space();
        space.write::<usize>(
            super::G_SUPERVISOR + super::supervisor::KEYBOARD,
            KEYBOARD.start,
        );
        space.write::<usize>(KEYBOARD.start, KEYBOARD_VTABLE);
        for (slot, function) in [
            (super::dinput_device::ACQUIRE, acquire),
            (super::dinput_device::UNACQUIRE, unacquire),
            (super::dinput_device::RELEASE, release),
        ] {
            space.write::<usize>(KEYBOARD_VTABLE + slot * size_of::<usize>(), function);
        }
    }

    /// Whether the game still holds one, which is what orb clears when it lets it go: the pointer being
    /// nothing is what sends `Controller::GetInput` down its `GetKeyboardState` branch.
    pub fn holds_a_keyboard_device(&self) -> bool {
        self.space()
            .read::<usize>(super::G_SUPERVISOR + super::supervisor::KEYBOARD)
            != 0
    }

    /// Whether the run on screen is the attract demo, which the game starts from its title screen when
    /// nobody has pressed anything.
    ///
    /// A run in every other respect — it goes through the same two states a played one does — and this flag
    /// is the whole of what tells it apart, which is what `read_state` reads it for: a demo is not a run
    /// somebody is playing, so nothing of the mode is offered over one.
    pub fn demo_mode(&self, demo: bool) {
        self.space().write::<u8>(
            super::G_GAME_MANAGER + super::game_manager::DEMO_MODE,
            u8::from(demo),
        );
    }

    /// The mapping that says which of its buttons the game reads as what.
    pub fn maps_the_pad(&self, mapping: Mapping) {
        use super::supervisor;
        let space = self.space();
        for (at, button) in [
            (supervisor::CFG_SHOOT_BUTTON, mapping.shoot),
            (supervisor::CFG_BOMB_BUTTON, mapping.bomb),
            (supervisor::CFG_MENU_BUTTON, mapping.menu),
            (supervisor::CFG_UP_BUTTON, mapping.up),
            (supervisor::CFG_DOWN_BUTTON, mapping.down),
            (supervisor::CFG_PAD_Y_AXIS, mapping.y_axis),
        ] {
            space.write::<i16>(super::G_SUPERVISOR + at, button);
        }
    }

    /// The three objects the game's own chain hands to a callback, which is what orb's hooks over
    /// those callbacks are given: the front end, the game manager, and the screen a ranking is shown
    /// on.
    ///
    /// Here rather than as addresses in whatever is driving the game, for the same reason everything
    /// else in this file is: the offsets are th06's, and this is the one place that knows them.
    pub fn front_end_object(&self) -> usize {
        super::G_MAIN_MENU
    }

    pub fn game_manager_object(&self) -> usize {
        super::G_GAME_MANAGER
    }

    pub fn ranking_screen(&self) -> usize {
        RANKING_SCREEN.start
    }

    /// The window object the game's whole frame is a method on, which is what a frame loop — the
    /// game's own or orb's in its place — is called on.
    pub fn game_window_object(&self) -> usize {
        super::G_GAME_WINDOW
    }

    /// `ResultScreen::RegisterChain`: the screen's own job in the chain's *calc* list, carrying
    /// `ResultScreen::OnUpdate` as its callback and the screen itself as what to call it on.
    ///
    /// Which is the only way to that screen at all — nothing in the game's static data points at it, and
    /// `chain_argument` finding it by its callback is what orb does instead. So a game that wrote the
    /// screen's state into a global would be answering the question orb's walk is being asked.
    pub fn registers_the_result_screen(&self, state: i32) {
        let space = self.space();
        let elem = RESULT_SCREEN_ELEM.start;
        space.write::<usize>(
            elem + super::chain_elem::CALLBACK,
            super::RESULT_SCREEN_ON_UPDATE,
        );
        space.write::<usize>(elem + super::chain_elem::ARG, RESULT_SCREEN.start);
        let head = super::G_CHAIN + super::chain_elem::NEXT;
        let after: usize = space.read(head);
        space.write::<usize>(elem + super::chain_elem::NEXT, after);
        space.write::<usize>(head, elem);
        self.set_result_screen_state(state);
    }

    /// And `Chain::Cut` taking it out, which the screen's deleted callback does on the way to the title.
    pub fn cuts_the_result_screen(&self) {
        let space = self.space();
        let elem = RESULT_SCREEN_ELEM.start;
        let head = super::G_CHAIN + super::chain_elem::NEXT;
        if space.read::<usize>(head) == elem {
            let after: usize = space.read(elem + super::chain_elem::NEXT);
            space.write::<usize>(head, after);
        }
        space.write::<usize>(elem + super::chain_elem::NEXT, 0);
        space.write::<usize>(elem + super::chain_elem::CALLBACK, 0);
    }

    /// Which of its states that screen is in, and the two orb has anything to do with — the question it
    /// asks about saving a replay, and the way out it writes in place of answering one.
    pub fn result_screen_state(&self) -> i32 {
        self.space()
            .read(RESULT_SCREEN.start + super::result_screen::STATE)
    }

    pub fn set_result_screen_state(&self, state: i32) {
        self.space()
            .write::<i32>(RESULT_SCREEN.start + super::result_screen::STATE, state);
    }

    /// `Ending::RegisterChain` (0x4107b0): the ending's own job in the chain's calc list, carrying
    /// `Ending::OnUpdate` as its callback and the ending itself as what to call it on, with the `.end`
    /// script it has read in the object.
    pub fn registers_the_ending(&self) {
        let space = self.space();
        let elem = ENDING_ELEM.start;
        space.write::<usize>(elem + super::chain_elem::CALLBACK, super::ENDING_ON_UPDATE);
        space.write::<usize>(elem + super::chain_elem::ARG, ENDING.start);
        let head = super::G_CHAIN + super::chain_elem::NEXT;
        let after: usize = space.read(head);
        space.write::<usize>(elem + super::chain_elem::NEXT, after);
        space.write::<usize>(head, elem);
        space.write::<usize>(ENDING.start + super::ending::SCRIPT, ENDING_SCRIPT);
    }

    /// `Ending::LoadEndingFile` reading the staff roll's script over the one running, which is every
    /// ending's last act: all six of 紅魔郷's finish on `@Fdata/staff00.end`, so the script changing is
    /// where the ending itself ends and the roll begins.
    pub fn hands_over_to_the_roll(&self) {
        self.space()
            .write::<usize>(ENDING.start + super::ending::SCRIPT, ROLL_SCRIPT);
    }

    /// And `Chain::Cut` taking that job out, which is the scene being taken down.
    pub fn cuts_the_ending(&self) {
        let space = self.space();
        let elem = ENDING_ELEM.start;
        let head = super::G_CHAIN + super::chain_elem::NEXT;
        if space.read::<usize>(head) == elem {
            let after: usize = space.read(elem + super::chain_elem::NEXT);
            space.write::<usize>(head, after);
        }
        space.write::<usize>(elem + super::chain_elem::NEXT, 0);
        space.write::<usize>(elem + super::chain_elem::CALLBACK, 0);
    }

    /// The track the sound player is streaming, through the two objects it is reached by: the streaming
    /// sound the player keeps, and the wave file under it holding [`Track`]'s three numbers.
    ///
    /// A vtable in the image at the head of the stream, because that is what orb asks before it reads
    /// anything through the pointer — see [`AUDIO_VTABLE`].
    pub fn plays_a_track(&self, track: Track) {
        let space = self.space();
        space.write::<usize>(STREAM, AUDIO_VTABLE);
        space.write::<usize>(STREAM + super::streaming_sound::WAVE_FILE, WAVE);
        space.write::<u32>(WAVE + super::wave_file::SIZE_OF_FILE, track.length);
        space.write::<u32>(WAVE + super::wave_file::LOOP_START, track.loop_start);
        space.write::<u32>(WAVE + super::wave_file::LOOP_END, track.loop_end);
        space.write::<usize>(
            super::G_SOUND_PLAYER + super::sound_player::BACKGROUND_MUSIC,
            STREAM,
        );
    }

    /// And the whole of the sound that track is streamed through, for a scenario about the stream itself:
    /// the buffer the game plays and the file handle it reads it out of, beside the wave file's own
    /// numbers.
    ///
    /// `left` is the countdown the track's loop is taken on — how many bytes of sound the stream believes
    /// are left before it — which with the file's position is the pair a loop is decided by: the game
    /// subtracts every byte it reads from it and starts the track over when a read comes up short against
    /// it. So the two are put in together, and a scenario that moved one without the other would be laying
    /// out the very fault this is about.
    pub fn streams_a_track(&self, track: Track, sound: &Sound, left: u32) {
        self.plays_a_track(track);
        sound.install(&self.sim);
        let space = self.space();
        space.write::<usize>(STREAM + super::streaming_sound::BUFFERS, BUFFERS);
        space.write::<usize>(BUFFERS, sound.buffer_object());
        space.write::<u32>(
            STREAM + super::streaming_sound::BUFFER_SIZE,
            sound.buffer_size(),
        );
        space.write::<u32>(STREAM + super::streaming_sound::BUFFER_COUNT, 1);
        space.write::<u32>(
            STREAM + super::streaming_sound::NOTIFY_SIZE,
            sound.notify_size(),
        );
        space.write::<u32>(STREAM + super::streaming_sound::NEXT_WRITE_OFFSET, 0);
        space.write::<usize>(WAVE + super::wave_file::MMIO, sound.mmio());
        space.write::<u32>(WAVE + super::wave_file::BYTES_LEFT, left);
    }

    /// How many bytes of sound the stream believes are left before the track loops, which is what a seek
    /// has to move with the file.
    pub fn bytes_left(&self) -> u32 {
        self.space().read(WAVE + super::wave_file::BYTES_LEFT)
    }

    /// And the same wave file with no handle in it, which is a stream orb cannot move the file of: both
    /// winmm calls refuse a handle of nothing, and every read of the position answers that it will not say.
    ///
    /// Which is a thing that happens — `Th06::music` takes the handle as nothing where the read of it does
    /// not come off — and what orb does about it is give up on the file and put the rest back.
    pub fn forgets_the_file_handle(&self) {
        self.space()
            .write::<usize>(WAVE + super::wave_file::MMIO, 0);
    }

    pub fn set_bytes_left(&self, left: u32) {
        self.space()
            .write::<u32>(WAVE + super::wave_file::BYTES_LEFT, left);
    }

    /// The offset the streaming thread will write the next chunk at, which orb reads as the token for
    /// "has the stream moved since I looked" — and which a restore puts back with the rest of the
    /// game's memory rather than through the buffer.
    pub fn next_write_offset(&self) -> u32 {
        self.space()
            .read(STREAM + super::streaming_sound::NEXT_WRITE_OFFSET)
    }

    pub fn set_next_write_offset(&self, at: u32) {
        self.space()
            .write::<u32>(STREAM + super::streaming_sound::NEXT_WRITE_OFFSET, at);
    }

    /// And no track at all, which is what a game with nothing playing has: the pointer the sound player
    /// keeps is what `StopBGM` clears, and every read of the music goes through it.
    pub fn takes_the_music_down(&self) {
        self.space().write::<usize>(
            super::G_SOUND_PLAYER + super::sound_player::BACKGROUND_MUSIC,
            0,
        );
    }

    /// Says that screen is up with its records read in, which is the state orb waits for before it
    /// puts back what this session counted.
    pub fn ranking_screen_shown(&self) {
        self.space()
            .write::<i32>(RANKING_SCREEN.start + super::RANKING_STATE, RANKING_SHOWN);
    }

    /// Whether it has been told to leave, which is what asks the game to put the scene back.
    pub fn ranking_screen_leaving(&self) -> bool {
        self.space()
            .read::<i32>(RANKING_SCREEN.start + super::RANKING_STATE)
            == super::RESULT_SCREEN_STATE_EXITING
    }

    /// Every card the game holds a record of, and the attempts against each — which is what its
    /// ranking screen has to show and what its score file is written from.
    ///
    /// A record with no `CATK` at its head is not a card this build has, so it is not a row: that is
    /// the same test `count_card_attempt` refuses on.
    pub fn card_records(&self) -> Vec<(i32, u16)> {
        let cards = super::CARD_HISTORY_BYTES / 0x40;
        (0..cards as i32)
            .filter(|card| {
                self.space()
                    .read::<u32>(super::CARD_HISTORY + *card as usize * 0x40)
                    == super::CATK_MAGIC
            })
            .map(|card| (card, self.card_attempts(card)))
            .collect()
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

    /// `ReplayManager::RegisterChain` with a replay loaded: the manager, the replay under it, and a
    /// record for each of the stages that replay covers.
    ///
    /// A stage the replay does not cover keeps the null pointer it was laid out with, which is what
    /// `jump_to_stage` asks the replay about before it moves anywhere.
    pub fn loads_a_replay(&self, stages: &[i32]) {
        let space = self.space();
        space.write::<usize>(super::G_REPLAY_MANAGER, REPLAY_MANAGER.start);
        space.write::<usize>(
            REPLAY_MANAGER.start + super::replay_manager::REPLAY_DATA,
            REPLAY,
        );
        // `isDemo`, which is the game's own name for playback: the attract demo and a replay somebody
        // chose are the same thing to it, and it is what orb reads to know the record is being watched
        // rather than written.
        space.write::<i32>(REPLAY_MANAGER.start + super::replay_manager::IS_DEMO, 1);
        for stage in stages {
            space.write::<usize>(self.stage_entry(*stage), record_of(*stage));
        }
    }

    /// The inputs one of those stages holds, as the pairs a recording is made of: the frame each change
    /// of what was held happened on, and what was held from then on — and the seed that stage was drawn
    /// with, since a replay puts that back too.
    pub fn records_the_inputs(&self, stage: i32, seed: u16, entries: &[(i32, u16)]) {
        let space = self.space();
        let record = record_of(stage);
        space.write::<i32>(record + record::ENTRIES, entries.len() as i32);
        space.write::<i32>(record + record::PLAYBACK_AT, 0);
        space.write::<u16>(record + record::SEED, seed);
        for (index, (frame, input)) in entries.iter().enumerate() {
            let at = record + record::FIRST + index * record::STRIDE;
            space.write::<i32>(at, *frame);
            space.write::<u16>(at + size_of::<i32>(), *input);
        }
    }

    /// And what it holds now, which is what says whether a teardown wrote over it.
    pub fn recorded_inputs(&self, stage: i32) -> Vec<(i32, u16)> {
        let space = self.space();
        let record = record_of(stage);
        let entries: i32 = space.read(record + record::ENTRIES);
        (0..entries.max(0) as usize)
            .map(|index| {
                let at = record + record::FIRST + index * record::STRIDE;
                (space.read(at), space.read(at + size_of::<i32>()))
            })
            .collect()
    }

    /// What the record says was held on `frame`, and how far playback has reached in saying so.
    ///
    /// The entries are walked from the start, as playback walks them: nothing else says which entry is
    /// current, and the count is what a teardown writing over the record moves.
    pub fn plays_back_the_inputs(&self, stage: i32, frame: i32) -> u16 {
        let entries = self.recorded_inputs(stage);
        let reached = entries
            .iter()
            .rposition(|(at, _)| *at <= frame)
            .unwrap_or(0);
        self.space()
            .write::<i32>(record_of(stage) + record::PLAYBACK_AT, reached as i32);
        entries.get(reached).map_or(0, |(_, input)| *input)
    }

    /// `ReplayManager::StopRecording`: a blank input at the frame the run stopped, and a terminator
    /// after it, written into the record at the entry playback has reached.
    ///
    /// Which is right for a recording and wrong for a replay — the record it lands in during playback is
    /// the replay's own — and that is the whole of what `orb::stop_recording` holds back.
    pub fn stops_recording(&self, stage: i32, frame: i32) {
        let space = self.space();
        let record = record_of(stage);
        let at: i32 = space.read(record + record::PLAYBACK_AT);
        for (index, entry) in [(at, (frame, 0)), (at + 1, (RECORD_ENDS_AT, 0))] {
            let index = index.max(0) as usize;
            if record::FIRST + (index + 1) * record::STRIDE > RECORD_BYTES {
                continue;
            }
            let (frame, input) = entry;
            let written = record + record::FIRST + index * record::STRIDE;
            space.write::<i32>(written, frame);
            space.write::<u16>(written + size_of::<i32>(), input);
        }
        let entries: i32 = space.read(record + record::ENTRIES);
        space.write::<i32>(record + record::ENTRIES, entries.max(at + 2));
    }

    /// The seed a stage's record says that stage was drawn with, which is what a replay writes where a
    /// played stage would have had whatever the generator was left on.
    pub fn recorded_seed(&self, stage: i32) -> u16 {
        self.space().read(record_of(stage) + record::SEED)
    }

    /// The clock the replay is at, which is the first field of the manager and what
    /// [`Reproduction`](crate::game::Reproduction) reads as `replay_frame`.
    pub fn set_replay_clock(&self, frame: i32) {
        self.space().write::<i32>(REPLAY_MANAGER.start, frame);
    }

    /// Where the pointer to `stage`'s record goes in the replay it belongs to.
    fn stage_entry(&self, stage: i32) -> usize {
        REPLAY + super::replay_data::STAGE_DATA + stage as usize * size_of::<usize>()
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

/// Where a stage's record of inputs is laid out, one block of [`RECORD_BYTES`] per stage.
fn record_of(stage: i32) -> usize {
    RECORDS.start + stage.max(0) as usize * RECORD_BYTES
}

/// How many attempts a saved record of captures holds against `card`.
///
/// The same 0x40-byte records [`Image::card_attempts`] reads in memory, read out of the bytes a score
/// file was written from instead: what the file holds is what the memory held when the ranking screen
/// went down, so this is what says the trip through that screen wrote what a session counted.
pub fn attempts_in(saved: &[u8], card: i32) -> u16 {
    let at = card as usize * 0x40 + super::CATK_ATTEMPTS;
    saved
        .get(at..at + size_of::<u16>())
        .map_or(0, |bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
}

/// The state the ranking screen is in once it is up with its records read in. The first of the states
/// orb reads as "showing", so a game laid out here and orb cannot disagree about which those are.
const RANKING_SHOWN: i32 = super::RESULT_SCREEN_SHOWING[0];

/// The two states of the result screen orb has anything to do with, under the names the exe's own
/// `ResultScreenState` gives them: the question about saving a replay, and the way out the game itself
/// puts a practice run's result screen into.
///
/// Named here rather than left as numbers in whatever drives the game, for the same reason every other
/// offset is: they are th06's, and this is the one place that knows them.
pub mod result_state {
    pub const SAVE_REPLAY_QUESTION: i32 = super::super::RESULT_STATE_SAVE_REPLAY_QUESTION;
    pub const EXIT: i32 = super::super::RESULT_STATE_EXIT;
}

fn scene_of(scene: Scene) -> i32 {
    match scene {
        Scene::FrontEnd => super::STATE_MAINMENU,
        Scene::Playing => super::STATE_GAMEMANAGER,
        Scene::Result => super::STATE_RESULTSCREEN_FROMGAME,
        Scene::Ranking => super::STATE_SCORE,
        Scene::Rebuilding => super::STATE_GAMEMANAGER_REINIT,
        Scene::Ending => super::STATE_ENDING,
        Scene::Other(state) => state,
    }
}

fn scene_from(state: i32) -> Scene {
    match state {
        super::STATE_MAINMENU => Scene::FrontEnd,
        super::STATE_GAMEMANAGER => Scene::Playing,
        super::STATE_RESULTSCREEN_FROMGAME => Scene::Result,
        super::STATE_SCORE => Scene::Ranking,
        super::STATE_GAMEMANAGER_REINIT => Scene::Rebuilding,
        super::STATE_ENDING => Scene::Ending,
        state => Scene::Other(state),
    }
}

fn screen_of(screen: Screen) -> i32 {
    match screen {
        Screen::Title => super::MENU_STATE_TITLE,
        Screen::ShotType => super::MENU_STATE_SHOT_TYPE,
        Screen::Ranking => super::MENU_STATE_SCORE,
        Screen::Other(state) => state,
    }
}

fn screen_from(state: i32) -> Screen {
    match state {
        super::MENU_STATE_TITLE => Screen::Title,
        super::MENU_STATE_SHOT_TYPE => Screen::ShotType,
        super::MENU_STATE_SCORE => Screen::Ranking,
        state => Screen::Other(state),
    }
}
