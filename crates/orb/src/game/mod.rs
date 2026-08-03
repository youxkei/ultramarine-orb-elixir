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
use crate::d3d8::{Device, Texture};

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
    /// Runs where the game has just put a stage's numbers in place and before that
    /// stage's first update: the one moment a run's state can be written without
    /// something of the stage having already been decided from the state it replaces.
    ///
    /// `None` for a game with no such point, which leaves a run unable to be picked up
    /// again — see [`crate::resume`].
    pub stage_begun: Option<Patch>,
    /// Runs after those numbers are in place and before the stage is built out of them,
    /// which is the last moment nothing of the stage has been drawn from the generator
    /// yet: the only place a resumed run's seed can go.
    ///
    /// A separate seam from [`stage_begun`](Hooks::stage_begun) because the two moments are
    /// not the same one, and what the seed has to be in before is this one.
    ///
    /// `None` where the game has no such call, which costs a resumed run the seed and with
    /// it every enemy the seed decides.
    pub stage_building: Option<Patch>,
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

/// What the game's own front end has just been asked for.
///
/// Only the two moments orb has a question of its own to put over: a run is one thing in
/// pointdevice mode and another in normal mode, and so is a ranking. Everything else the front
/// end does is [`Elsewhere`](Menu::Elsewhere), including being nowhere near it.
///
/// The moment rather than the choice: what a game is left holding after the front end has acted
/// on a keypress differs per game, and every game has a frame on which a run has been chosen and
/// not yet built.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Menu {
    /// Anywhere else: a run in progress, a replay, the options, or no front end at all.
    Elsewhere,
    /// A run has been chosen and the game is on its way into it. Practice as well as a full
    /// run and the Extra stage: each is a run, and each is one thing with chapters and another
    /// without.
    Run,
    /// The ranking has been chosen and the game is on its way into it.
    Scores,
}

/// What a pad is doing, in the terms a menu of orb's needs.
///
/// Asked of the game because every one of these is its own mapping: which button decides and which
/// cancels, and how far a stick goes before it counts. Answered from a reading orb took itself,
/// because a menu of orb's has the game frozen and the game is not reading the pad on those frames.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Pad {
    pub up: bool,
    pub down: bool,
    pub decide: bool,
    pub cancel: bool,
}

/// The tile the game paints its status panel's background with: a texture it has already loaded,
/// the piece of that texture which is one tile, and the grid the tiles are laid on.
///
/// Asked for rather than carried, so that orb ships no art and a modified sheet is honoured —
/// and so that what is left on the screen where orb stops drawing is the panel the game would
/// have painted there anyway.
pub struct PanelTile {
    pub texture: *mut Texture,
    /// The piece of the sheet, as `[left, top, right, bottom]` in texture coordinates.
    pub uv: [f32; 4],
    /// Where the grid starts in the game's output, and how far apart its lines are.
    pub origin: (f32, f32),
    pub pitch: f32,
}

/// Which run this is: everything the game's own front end is asked for before one starts.
///
/// What says whether a chapter written down belongs to the run about to be played. A run of another
/// character is another run — its buttons would play somebody else's shot — which is why 紺珠伝 keeps
/// a pointdevice save per difficulty and character, and why orb keeps one per [`Game::run_slot`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RunStart {
    pub difficulty: i32,
    /// Whose run it is, and which of that character's two shots. Two numbers because that is
    /// how the game keeps them, and every table it indexes by them is `character * 2 + shot`.
    pub character: i32,
    pub shot_type: i32,
    /// One stage rather than a whole run.
    pub practice: bool,
    /// Counted from zero, the way orb counts stages everywhere.
    pub stage: i32,
}

/// The run's numbers as a stage begins, which is the state a stage needs to be played again and
/// nothing else.
///
/// These are the fields the game's own replay keeps per stage — see `SPEC.md` for the record they
/// are read out of — plus the count of deaths, which the replay leaves behind because the result
/// screen is the only thing that reads it. Everything else a stage starts with is either loaded
/// with the stage or set the same way every time, so it is the same on both sides of a resume.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RunState {
    pub score: u32,
    /// What the generator was seeded with as the stage began. The whole of why the buttons alone
    /// are not enough: the game takes this from wherever the generator had got to, so a stage
    /// played again from a different seed is a different stage.
    pub seed: u16,
    pub point_items: u16,
    pub power: u16,
    pub lives: i8,
    pub bombs: i8,
    /// What the game makes of how the run is going, which the enemies read.
    pub rank: i32,
    pub power_items: i8,
    /// How many extra lives the score has already paid for, so the next one is not paid twice.
    pub extra_lives: i8,
    pub deaths: i32,
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

    /// How to put the status panel's own background back over a piece of it — for the parts of a
    /// mark that fall outside a row the game repaints. `None` before the sheet is loaded, and for a
    /// game whose panel is not a tiled one.
    ///
    /// # Safety
    /// Must run on the game's main thread, with the game past initialisation.
    unsafe fn panel_tile(&self) -> Option<PanelTile>;

    /// Asks the game to repaint the row it shows the lives in, this frame — the background and the
    /// count both.
    ///
    /// For a mark drawn over that row, and it does two things at once. A panel that is not
    /// repainted is one where a mark blended over what the last frame left hardens into its own
    /// edges; and the count showing faintly through where the ink of the mark is dry is only true
    /// if the count is drawn again underneath it. Which is worth having: the lives are disabled,
    /// not gone, and one gained still shows.
    ///
    /// # Safety
    /// Must run on the game's main thread, before the game draws.
    unsafe fn repaint_lives_row(&self);

    /// Where the game shows how many lives are left, in its own 640x480 output: the part of
    /// the status panel holding the count itself, which is what a mark saying the count
    /// decides nothing goes over.
    ///
    /// The count and not the label beside it. This is a rectangle orb paints over every
    /// frame, so it has to be one the game paints too: a count that changes is erased and
    /// redrawn, and the label beside it is drawn when the stage begins and never again.
    fn lives_row(&self) -> Rect;

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

    /// Makes the game read its keyboard the way it does when DirectInput has no device to read,
    /// and says whether it had one to let go of.
    ///
    /// For a session driven by another program: a device taken `DISCL_EXCLUSIVE | DISCL_FOREGROUND`
    /// does not see keys sent with `SendInput`, and every screen orb has a question over is the
    /// game's own screen answered on the game's own keyboard. What is left is the path the game
    /// takes when it never got a device, which reads the state the system keeps — and that does see
    /// them. Nothing else about the run changes: the same code turns those keys into the same
    /// buttons.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the game past initialisation.
    unsafe fn take_sent_keys(&self) -> bool;

    /// Where the game keeps the `JOYCAPSA` it measures a joystick's axes against, if it
    /// keeps one at all.
    ///
    /// Needed because a game may read those caps once and never again — 紅魔郷 reads them
    /// at startup, and only if a joystick answered then — while an axis it never calibrated
    /// against is one whose centre reads as far over. `None` for a game that asks the
    /// device every time, which has no such window to be wrong in.
    fn joystick_calibration(&self) -> Option<usize>;

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

    /// Puts the player where nothing can hit them for the update about to run, using
    /// whatever the game itself does for the seconds after a bomb or a respawn.
    ///
    /// For one update, because that state is the game's own and the game takes it back:
    /// a clear that reaches an ending is the only way to see one, and half an hour of
    /// playing well is not how a boundary in the ending gets looked at.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a stage running.
    unsafe fn make_invulnerable(&self);

    /// Whether the replay held in memory is one being played back, whose record of
    /// inputs nothing may write into.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn replaying(&self) -> bool;

    /// What the game's own front end has just been asked for, so orb can put its question over
    /// the moment a run or the ranking is chosen.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn menu(&self) -> Menu;

    /// What the pad is doing, in the terms a menu of orb's needs: which way it is being pushed,
    /// and whether it is deciding or cancelling.
    ///
    /// Asked of the game rather than worked out from a reading, because *where* a pad is read is
    /// the game's business too. A game may reach one some way orb's own sampling does not — and
    /// then a menu of orb's driven from the sample answers to a pad the game has not got, which
    /// looks exactly like orb's menus ignoring a pad that plainly works. Which is what it was.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn pad(&self) -> Pad;

    /// Puts the game's front end back on its way to the title menu, the way its own back button
    /// does, and says whether there was a front end to do it to.
    ///
    /// For a question orb asked over a choice the game has already acted on: the answer to it can
    /// be "neither", and then what the player asked for is the menu they came from. Doing it the
    /// game's own way rather than undoing what the choice did is what keeps orb out of the business
    /// of putting a screen's worth of animation back.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with the front end running.
    unsafe fn leave_menu(&self) -> bool;

    /// Gives the run up, and says whether there was one to give up.
    ///
    /// For the retry menu's third choice. Asked of the game rather than done by hand, because
    /// every game has its own way out of a run and each of them already works: what a pause
    /// menu's quit does is what orb should do, so that whatever a run has to be taken down is
    /// taken down by the code that knows about it.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, with a run in progress.
    unsafe fn leave_run(&self) -> bool;

    /// Leaves the game's own idea of the buttons so that the frame it carries on into has no new
    /// press on it, whatever is being held.
    ///
    /// For the frames orb froze the game for: the key that answered orb's question is not a key
    /// the game's own menu should act on, and the keyboard is read globally rather than per
    /// window, so there is nothing else to tell one from the other. Without it, whether the
    /// answer also reaches the game depends on whether the screen behind the question happens to
    /// read input on the frame it resumes on, which is a property of that screen and not
    /// something orb decided.
    ///
    /// Not needed after a retry, where the snapshot restore puts the same state back with
    /// everything else in `.data`.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn swallow_input(&self);

    /// Takes the game off the screen that offers to save a replay of the run just finished, if
    /// that is where it is, and says whether it did.
    ///
    /// For a pointdevice run, where the offer is one nothing could accept: a replay is the
    /// inputs and nothing about the rewinds between them, so one recorded here plays back as a
    /// run that dies where this one carried on. Refusing the write instead would leave somebody
    /// naming a file that never appears.
    ///
    /// Asked every frame, so it must cost nothing on the frames where there is no such screen.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames.
    unsafe fn skip_replay_prompt(&self) -> bool;

    /// The frame the stage now running is on, or `None` where no stage is running.
    ///
    /// Its own accessor rather than a whole [`State`] because the input read asks for it on every
    /// frame there is, including the frames of a menu: what a frame's buttons are written down
    /// against is this number, and reading the rest of the state to get it would put a job-chain
    /// walk inside the game's keyboard read.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn stage_frame(&self) -> Option<u32>;

    /// Which bits of a frame's input decide how a run goes, and so are the ones worth writing
    /// down and handing back.
    ///
    /// 紅魔郷's own replay records exactly these, which is what says the rest do not change a run.
    /// The pause button is outside them, which is the one that matters: a resume that fed it back
    /// would open the menu instead of playing.
    fn run_input(&self) -> u16;

    /// Which run this is, as the game's own front end left it.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn run_start(&self) -> RunStart;

    /// Which run the front end is *pointing* at, where it is on a screen that knows: the difficulty
    /// and the character already chosen, and the last of the three under the cursor.
    ///
    /// For the mark that says a run of it was left unfinished, which is worth putting where the run is
    /// being chosen rather than only offering it a screen later — nothing on the game's own screens
    /// remembers which run somebody was in the middle of.
    ///
    /// `None` anywhere the answer would be a guess: any other screen, and a practice run, whose stage
    /// is part of which run it is and is not chosen until afterwards.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_pointed_at(&self) -> Option<RunStart>;

    /// Which run this is one of, as a name: what two runs have in common when a chapter of one can
    /// be picked up in the other.
    ///
    /// Asked of the game because the characters are its own. It is also the name of the file that
    /// chapter is kept in, so it has to be one a directory listing can be read — and two runs are
    /// the same run exactly when this is the same, which is what keeps the answer in one place.
    ///
    /// `None` for a run there is no slot for, which is what stops one being kept at all: nothing to
    /// name is nothing to write, nothing to offer and nothing to mark, in one place rather than as a
    /// check three callers have to remember.
    fn run_slot(&self, run: &RunStart) -> Option<String>;

    /// The run's numbers, which at a stage's own first frame are the numbers that stage started
    /// with.
    ///
    /// # Safety
    /// Must run on the game's main thread, with a run in progress.
    unsafe fn run_state(&self) -> RunState;

    /// Puts them back, the way the game's own replay puts a stage's numbers back — all but the seed,
    /// which is [`Game::set_run_seed`]'s and has to be earlier.
    ///
    /// # Safety
    /// Must run on the game's main thread, where the game has a stage built and has not yet
    /// updated it — see [`Hooks::stage_begun`].
    unsafe fn set_run_state(&self, state: &RunState);

    /// Seeds the generator for the stage the game is about to build, and puts its own copy of the
    /// seed beside it.
    ///
    /// Apart from the rest of a stage's numbers because it goes in at another moment: building a
    /// stage draws from the generator, so the seed has to be in before that, where the numbers cannot
    /// go in until the stage exists. Both moments were measured wrong before they were measured
    /// right — see the game's own account of the seam, `th06`'s `STAGE_REGISTER_CHAIN`.
    ///
    /// # Safety
    /// Must run on the game's main thread, from the hook over [`Hooks::stage_building`], before the
    /// original.
    unsafe fn set_run_seed(&self, seed: u16);

    /// Whether the game's front end has asked for a run and nothing of that run has been built yet.
    ///
    /// The one frame on which what the run is — its difficulty, its character, its shot — is settled
    /// and none of it has been acted on, which is where the question of picking one up belongs: the
    /// answer costs nothing either way there, and the front end has already taken itself down. Never
    /// true of the attract demo or a replay, which ask for a run the same way.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_chosen(&self) -> bool;

    /// Points the run being built at another of its stages, and says whether the game has one.
    ///
    /// The whole of what picking a run up asks of the front end: the difficulty, character and shot
    /// are the ones just chosen, and [`Hooks::stage_begun`] is where the run's own numbers go in.
    ///
    /// # Safety
    /// Must run on the game's main thread, between frames, on a frame [`Game::run_chosen`] is true
    /// of.
    unsafe fn start_stage(&self, stage: i32) -> bool;

    /// Whether the run has finished into the game's own result screen.
    ///
    /// What says a run is over rather than left: a run given up does not go through one — see
    /// [`Game::leave_run`] — so this is what tells the chapter worth coming back to from the run
    /// that has none.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn run_finished(&self) -> bool;

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
    /// The generator itself, which is the one number a wrong stage start can be wrong in while
    /// every other field of this line is right: a resumed stage 2 agreed on all of them with the
    /// seed 2048 draws out, the player's place being the player's own inputs and the bullets being
    /// the stage's script. Not the `GameManager` copy the file's own header carries — this is what
    /// the next number out of it will be made from.
    pub seed: u16,
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
             rng={:#06x} items={} score={} extras={} rank={} subrank={}",
            self.replay_frame,
            self.input,
            self.player.0,
            self.player.1,
            self.player_area.0,
            self.player_area.1,
            self.randoms,
            self.seed,
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
    /// Which script the ending is running, as the address of the file it was read from.
    /// `None` when no ending is running.
    ///
    /// An ending is one script per part of itself and moves on by reading the next file over
    /// the one it is running, so this changing is the one thing that marks a part of an ending
    /// ending: the scene does not change with it and no flag says which part is running. What
    /// the ending skip stops on, so that the part it hands over to — the staff roll — is
    /// played instead of run out with the rest.
    pub ending_script: Option<usize>,
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
