//! The seam between orb and the game it is running inside.
//!
//! Everything above this module — chapters, snapshots, the retry menu, the
//! overlay — is written against [`Game`] and [`State`] rather than against
//! 東方紅魔郷. Only [`th06`] implements it; the point of the seam is that porting
//! to another Touhou game means writing addresses and offsets, not reworking how
//! chapters or retries behave.

pub mod th06;

use std::ffi::c_void;
use std::fmt;
use std::ops::Range;

use windows_sys::Win32::Foundation::HWND;

use crate::audio::Music;
use crate::d3d8::Device;

/// A function to hook and the bytes expected at it, so a mismatch is caught
/// rather than relocating an instruction that cannot be relocated.
pub struct Patch {
    pub target: usize,
    pub prologue: &'static [u8],
}

pub struct Hooks {
    /// One call per logic frame. Not calling through freezes the game.
    pub update: Patch,
    /// Called inside the scene the game draws into, the only place an overlay
    /// may draw.
    pub draw: Patch,
    /// Writes a replay file. `None` if the game has nothing to suppress.
    pub save_replay: Option<Patch>,
    /// Finishes off the record of inputs a run is being recorded into. `None` if the
    /// game does not also run it while playing a replay back, where the record it
    /// would write into is the replay's own.
    pub stop_recording: Option<Patch>,
    /// Runs after the config is read and before the window and device exist, which
    /// is the only moment `force_windowed` can still take effect.
    pub create_window: Option<Patch>,
    /// Runs just after the device is created, and again after every reset: the
    /// first moment its vtable can be redirected.
    pub init_device: Option<Patch>,
    /// The game's whole frame: its draw, its update, its pacing and its present.
    /// Replacing it is what lets orb pace the game and drop a frame of input lag.
    pub render: Option<Patch>,
    /// Everything the game reads from the keyboard, once a frame. Needed because
    /// orb keeps updating while the window is not in front, and a game that carried
    /// on reading the keyboard then would act on keys meant for something else.
    pub input: Option<Patch>,
    /// The part of the input read that goes to a joystick, if the game keeps it
    /// separate. Hooked only to find out what it costs.
    pub joystick: Option<Patch>,
}

/// A rectangle in the game's own output resolution.
#[derive(Clone, Copy)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn center_x(&self) -> f32 {
        self.left + self.width / 2.0
    }

    pub fn center_y(&self) -> f32 {
        self.top + self.height / 2.0
    }
}

pub trait Game {
    fn hooks(&self) -> Hooks;

    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn read_state(&self) -> State;

    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn window(&self) -> HWND;

    /// Null until the game has finished setting Direct3D up.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn d3d_device(&self) -> *mut Device;

    /// The music stream, if one is playing and its objects look intact.
    fn music(&self) -> Option<Music>;

    /// Identifies the track playing, changing when the game switches tracks. Used to tell a
    /// stage's own music from a boss's, which is what tells a midboss from the boss a stage
    /// ends with: a stage's data names two songs and the game plays the second for that
    /// fight alone.
    fn music_identity(&self) -> Option<u32>;

    /// The thread that keeps the music's buffer topped up, which is left running
    /// while the game is held still: its pauses are the ones that are heard.
    fn audio_thread(&self) -> Option<u32>;

    /// Stops the music through the game's own code, so that the stream, its sound
    /// buffer and its streaming thread all go away the way the game expects.
    ///
    /// For a restore that cannot put the sound back: what the snapshot holds of it
    /// went with the track, and the stream playing now was allocated after the
    /// snapshot. Called while the game's memory is still its own, because that is
    /// the only moment freeing that stream is bookkeeping the allocator agrees with.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with nothing suspended.
    unsafe fn stop_music(&self);

    /// Starts the stage's own track again after such a restore, and returns whether
    /// there was one to name. The restored state names a stream that no longer
    /// exists, so that has to be cleared before the game is asked for anything.
    ///
    /// # Safety
    /// Must run on the game's main thread, after the memory has been restored.
    unsafe fn restart_stage_music(&self) -> bool;

    /// The memory holding the state of the sound system. A restore that leaves
    /// these alone leaves the music playing, instead of rewinding a stream whose
    /// sound buffer is not being rewound with it.
    fn audio_state(&self) -> Vec<Range<usize>>;

    /// Where the game keeps handles to things that are not its own memory — Direct3D's
    /// objects — which a snapshot cannot copy and a restore therefore must not rewind. Put
    /// back, a handle names something released long ago, and the next release of it is a
    /// use-after-free inside the game.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn live_handles(&self) -> Vec<Range<usize>>;

    fn play_area(&self) -> Rect;

    /// The size the game renders at, whose aspect ratio a borderless window has
    /// to preserve.
    fn content_size(&self) -> (u32, u32);

    /// False when the game is configured to take the display exclusively.
    fn windowed(&self) -> bool;

    /// Makes the game create a window instead of taking the display, so that
    /// borderless mode has a window to work with.
    ///
    /// # Safety
    /// Must run before the game creates its window.
    unsafe fn force_windowed(&self);

    /// Puts back the per-frame device setup the game itself does, for frames
    /// where its update is being held back.
    ///
    /// # Safety
    /// Must run on the game's main thread with a live device, before the game
    /// draws.
    unsafe fn set_play_viewport(&self, device: *mut Device);

    /// The chain object the update and draw functions are called on.
    fn chain(&self) -> *mut c_void;

    /// Whether the game wipes the back buffer before drawing each frame. What decides
    /// whether anything orb draws in a corner the game does not repaint stays there.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn clears_back_buffer(&self) -> bool;

    /// Puts the game's keyboard back in a state it can be read from, returning
    /// whether it now can be. `true` when the game reads the keyboard in a way that
    /// needs nothing done to it.
    ///
    /// Needed because a game may read the keyboard through a device the system takes
    /// away while the window is behind, and getting it back is not something the
    /// game's own code can be relied on to do.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the window in front.
    unsafe fn acquire_input(&self) -> bool;

    /// The device setup the game does before its update: the full-output viewport,
    /// and the background clear its options may ask for.
    ///
    /// # Safety
    /// Must run on the game's main thread with a live device.
    unsafe fn prepare_frame(&self, device: *mut Device);

    /// Hands queued sound effects to the sound system, as the game does once per
    /// frame after its update.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn play_sounds(&self);

    /// The game's own present, which also handles a lost device and its own
    /// screenshot key.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the frame drawn.
    unsafe fn present(&self);

    /// Starts the replay being watched at another of its stages, the way the game's own
    /// replay menu does: it tears this stage down, builds that one, and the replay's
    /// record of that stage puts the run's state back — the seed, the lives, the power,
    /// the score so far. Nothing of orb's is involved, which is what makes this the way
    /// to move between stages rather than restoring a snapshot of one.
    ///
    /// False where the replay has no such stage, so that nothing is asked for that
    /// would drop the game back to its menu.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn jump_to_stage(&self, stage: i32) -> bool;

    /// Whether the replay held in memory is one being played back, whose record of
    /// inputs nothing may write into.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn replaying(&self) -> bool;

    /// # Safety
    /// Must run on the game's main thread, with a stage running.
    unsafe fn reproduction(&self) -> Reproduction;

    /// Midstage chapter boundaries per stage, as script frame numbers.
    fn midstage_table(&self) -> &'static [&'static [i32]];

    fn midstage(&self, stage: i32) -> &'static [i32] {
        usize::try_from(stage)
            .ok()
            .and_then(|stage| self.midstage_table().get(stage).copied())
            .unwrap_or(&[])
    }
}

/// What says whether a stage is playing out the same way it did the last time it was
/// played: the clock the replay feeds its inputs on and the buttons it fed, where the
/// player is, and how many numbers the game has drawn from its generator.
///
/// Two passes over one stage of one replay have to agree on all of it, frame for frame.
/// That is the premise chapters rest on — a boundary is a frame number, and a step back
/// replays the stage to reach it — so when they disagree, the first frame they disagree
/// on and which of these numbers it was are the whole of the diagnosis.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Reproduction {
    /// `ReplayManager::frameId`, which the replay walks its record of inputs by. Its
    /// own count, not the stage's: a frame the game does not update does not advance it.
    pub replay_frame: i32,
    /// The buttons the update acted on, after the replay has put its own in.
    pub input: u16,
    pub player: (f32, f32),
    /// The top and the height of the box the player is held inside, which is also what
    /// the player's place at a stage's start is measured from. Nothing on the way into a
    /// stage puts it back where the stage was reached from another one.
    pub player_area: (f32, f32),
    /// How many numbers have come out of the generator since the stage seeded it.
    pub randoms: u32,
    /// Items in the air, which nothing about a stage's start puts back to none.
    pub items: u32,
    /// The score as shown, which is what an extra life is measured against.
    pub score: u32,
    /// How many extra lives the score has already paid for, so that the next one is
    /// not paid for twice.
    pub extra_lives: i8,
    /// What the game makes of how the run is going, which enemies read.
    pub rank: i32,
    pub sub_rank: i32,
}

impl fmt::Display for Reproduction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "replay_frame={} input={:#06x} player={:.2},{:.2} area={:.2}+{:.2} randoms={} \
             items={} score={} extras={} rank={} subrank={}",
            self.replay_frame,
            self.input,
            self.player.0,
            self.player.1,
            self.player_area.0,
            self.player_area.1,
            self.randoms,
            self.items,
            self.score,
            self.extra_lives,
            self.rank,
            self.sub_rank,
        )
    }
}

/// The game's state for one frame, read all at once so that everything in it
/// describes the same instant.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct State {
    /// The game's own scene identifier. Only for the log and for noticing that
    /// the scene changed.
    pub scene: i32,
    /// The gameplay scene, whoever is driving it.
    pub playing: bool,
    /// That scene, or the step between two of its stages. Everything a run's
    /// snapshots reach through is still alive here, which is what says whether they
    /// are worth keeping: a stage transition leaves the gameplay scene for a moment
    /// and is not the run ending.
    pub in_run: bool,
    /// A run orb should act on: not a menu, and not the attract demo or a replay,
    /// which look like play but have no player to offer a retry to.
    pub in_game: bool,
    pub in_ending: bool,
    pub demo: bool,
    pub replay: bool,
    pub practice: bool,
    /// A menu is open and the game has stopped updating itself.
    pub paused: bool,
    /// Dying or respawning: no place to start a chapter from, and no place to
    /// return to either.
    pub unsettled: bool,
    /// A bomb is going off, which clears the screen and makes the player
    /// invulnerable for a couple of seconds. Also no place to start a chapter: the
    /// danger has been taken away and comes straight back.
    pub bombing: bool,
    pub in_dialogue: bool,
    pub stage: i32,
    pub difficulty: i32,
    /// Frames since this stage began.
    pub stage_frames: u32,
    /// The clock the stage's enemy script runs on, which is what midstage
    /// boundaries are expressed in: it advances the same way however the player
    /// is doing, and on every difficulty.
    pub script_frames: i32,
    pub random_seed: u16,
    pub deaths: i32,
    pub lives: i8,
    pub bombs: i8,
    pub power: u16,
    pub enemy_count: i32,
    pub bullet_count: i32,
    pub laser_count: i32,
    pub boss_present: bool,
    pub boss_life: Option<i32>,
    /// Frames the boss's current attack has been running. Resetting is what marks
    /// the move to the next attack, spellcard or not.
    pub boss_attack_frames: Option<i32>,
    /// The spellcard being fought, if any.
    pub spellcard: Option<u32>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scene={} stage={} diff={} frames={} script={} seed={:#06x} \
             deaths={} lives={} bombs={} power={} enemies={} bullets={} lasers={} boss={}",
            self.scene,
            self.stage,
            self.difficulty,
            self.stage_frames,
            self.script_frames,
            self.random_seed,
            self.deaths,
            self.lives,
            self.bombs,
            self.power,
            self.enemy_count,
            self.bullet_count,
            self.laser_count,
            self.boss_present,
        )?;
        if let (Some(life), Some(frames)) = (self.boss_life, self.boss_attack_frames) {
            write!(f, " boss_life={life} attack_frames={frames}")?;
        }
        if let Some(spellcard) = self.spellcard {
            write!(f, " spell={spellcard}")?;
        }
        for (flag, name) in [
            (self.unsettled, "unsettled"),
            (self.bombing, "bombing"),
            (self.in_dialogue, "dialogue"),
            (self.paused, "paused"),
            (self.practice, "practice"),
            (self.replay, "replay"),
            (self.demo, "demo"),
            (self.in_ending, "ending"),
        ] {
            if flag {
                write!(f, " {name}")?;
            }
        }
        Ok(())
    }
}
