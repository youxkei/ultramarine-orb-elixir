//! th10, as a [`Game`] that answers what has been measured and declines the rest.
//!
//! `th10.exe`, 487,936 bytes, `md5 7dc488d82c81dd4aee4ba098b8804d83`, `ver 1.00a`. Every address below was
//! read off that exe with `i686-w64-mingw32-objdump -d -M intel` and nothing else: no table anybody
//! published and nothing carried across from th08's addresses, which are where the same things happen to
//! live in a different program. `asm.md` beside this file is that reading; the constants here are what orb
//! does with it.
//!
//! **What this game answers so far** is its frame's two walks, its scenes, its keys, the screens and cursor
//! of its front end, the run chosen and the run's own numbers, the generator and the stage's clocks, the
//! player with its box and its four speeds and its states, the enemies' bodies, the bullets with their
//! steps and boxes, both kinds of laser with the phase that says when each hits, the items, the boss and
//! the card up, and whether a conversation is on. So `Game::what_a_bot_sees` fills a whole frame, the rules
//! steer a run of it, and a recipe walks its front end.
//!
//! **And it rewinds**: [`Th10::rewinds`] answers `true`, so a launch in Pointdevice mode takes a snapshot at
//! each chapter, answers a hit with the retry menu, forks the score file to `pointdevice_scoreth10.dat` and
//! writes a run left unfinished to be picked up where it was left — see `resume.rs`.
#![doc = include_str!("asm.md")]

/// The midstage table, in a file of its own because a `--tune` pass writes one to paste over it.
pub mod chapters;
/// The address space laid out for a test with no game running — see `orb-e2e`'s `fake::th10`.
#[cfg(any(test, feature = "sim"))]
pub mod image;
/// The file a run left unfinished is written in — see `Game::encode_resume`.
pub mod resume;
/// The score file as bytes — see `Th10::read_what_the_modes_file_records`.
pub mod score_file;
/// What a bot playing this game is told of it — see `Game::what_a_bot_sees`.
mod sight;

use std::ffi::c_void;
use std::ops::Range;
// The slots behind `handed_over!` want this only where a laid-out game can fill them — see the macro,
// whose other half is the constant itself.
#[cfg(any(test, feature = "sim"))]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// **How many hits the run has taken, which is orb's count and not the game's** — see
/// [`Th10::read_state`], which moves it. This exe keeps no such count: no field of the run's numbers counts
/// a death. The death takes a life (0x426a15), a third of the faith above 5000 (0x426a22) and `0x400` off the
/// word at `0x474c98` (0x426cb6), and the dying state takes `0x40` off the power (0x425ab6) — none of them a
/// count.
static DEATHS: AtomicI32 = AtomicI32::new(0);
/// Whether the player was in [`PLAYER_STATE_HIT`] on the frame before, which is what makes the count above
/// move once per hit rather than once per frame of one.
static WAS_HIT: AtomicBool = AtomicBool::new(false);
/// **And the count each frame of the stage had**, by the stage's frame — see [`Th10::read_state`]. th08's
/// deaths are the game's own and come back with a chapter; these are orb's and would carry a death a retry
/// took back into every chapter after it.
static DEATHS_BY_FRAME: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());
/// **Whether the run's stage began at the end of the last one's fade**, out of the numbers a pick-up was put
/// back with, until the landing catches the player's shot up — see [`resume::Numbers::began_out_of_a_fade`] and
/// `Th10::starts_the_shot_a_stage_began_with`.
static THE_RUN_BEGAN_OUT_OF_A_FADE: AtomicBool = AtomicBool::new(false);

/// **The stage's script frames as orb keeps them**: the latest time any of the stage's own scripts has reached,
/// kept for every frame of the stage the clock has reached — see [`Th10::script_frames`].
///
/// **Kept by orb because nothing of the game's is that clock.** The stage is not always run by `main`: stage 2's
/// sat on one jump for the whole stage, its time 1, 2, 3 and round again, while the enemy it had spawned on the
/// first frame — flags `0x110`, no body and no life — ran the stage, its time 301, 601 and on to 5864, where it
/// stood on instruction 340 for the boss. And a script's time can go back: a jump sets it to the number the jump
/// carries (0x44e411), and a `call` starts the sub at nought (0x44e15a). The stage's frames at `0x474c88` go on
/// through a boss and a conversation alike (0x418774 is the one writer).
///
/// **The latest time and not the frames no script was held on**, which is what this counted first: stage 1's
/// `main` waits out its midboss with no instruction 340 under it — that count read `script=3354` at `frames=3354`
/// with the midboss up, and 4,881 where `main`'s own time stood at 4,370 for the boss — so what it waits on is a
/// loop of its own, whose jump back is the time standing. The latest time stands through that as through 340,
/// and climbs with the stage between.
///
/// **By the stage's frame, and not a running count**, because a chapter put back puts the stage's frames back
/// and nothing of orb's: the count a frame was given is what that frame reads again after a restore to it.
static SCRIPT_CLOCK: std::sync::Mutex<ScriptClock> = std::sync::Mutex::new(ScriptClock::EMPTY);

/// **The game object and the frame of it a stage was last said to begin on** — see
/// [`Th10::a_stage_has_just_been_built`] — put back to nothing by the next build.
static BEGUN: std::sync::Mutex<Option<(usize, i32)>> = std::sync::Mutex::new(None);
/// **Whether the last look [`Th10::a_stage_has_just_been_built`] took found the stage being built** — see
/// [`Th10::walks_a_frame_of_its_build`], which holds the walk after it to that look.
static A_BUILD_WAS_GOING_ON: AtomicBool = AtomicBool::new(false);
/// **The most frames the clock keeps of one stage**: the longest measured is the Extra's, 36,408 at the hit that
/// ended a launch on Suwako's last cards, and this is two hours and more at sixty.
const A_STAGE_RUNS_AT_MOST: usize = 1 << 19;

struct ScriptClock {
    /// The stage's own scripts, taken on the stage's first frame — see [`Th10::the_stage_scripts`].
    scripts: Vec<usize>,
    /// The count at each of the stage's frames, from nought at the stage's frame 0.
    by_frame: Vec<i32>,
    /// The frame the count was last asked for.
    last: usize,
}

impl ScriptClock {
    const EMPTY: Self = Self {
        scripts: Vec::new(),
        by_frame: Vec::new(),
        last: 0,
    };

    /// The clock at the stage's frame `at`, the stage's scripts taken with `scripts` on the stage's first frame
    /// and `time` asked of them on each frame counted for the first time: the latest time any of them that is
    /// still alive reads, if one is.
    fn at(
        &mut self,
        at: usize,
        scripts: impl FnOnce() -> Vec<usize>,
        time: impl FnOnce(&[usize]) -> Option<i32>,
    ) -> i32 {
        if at == 0 {
            *self = Self::EMPTY;
            return 0;
        }
        // **A frame past any stage's own answers what the clock last stood at, and is kept nowhere.** The stage's
        // frames are a word of the game's read inside hooks that cannot unwind, so a word read wrong — mid-build,
        // or off a game laid out by hand — would otherwise be a resize to the whole of it, and a `capacity
        // overflow` in a laid-out game's frame was that.
        if at > A_STAGE_RUNS_AT_MOST {
            return self.by_frame.get(self.last).copied().unwrap_or(0);
        }
        if self.by_frame.is_empty() {
            self.scripts = scripts();
            self.by_frame.push(0);
            self.last = 0;
        }
        // A frame read again, a frame gone back to or one read for the first time: the first two answer what
        // was counted then, whatever came after.
        if at <= self.last {
            self.last = at;
            return self.by_frame[at];
        }
        self.by_frame.truncate(self.last + 1);
        let before = self.by_frame[self.last];
        let now = time(&self.scripts).map_or(before, |time| time.max(before));
        // Frames no read was made on read what this one does.
        self.by_frame.resize(at + 1, now);
        self.last = at;
        self.by_frame[at]
    }
}

use orb_api::Device;
use orb_api::Hwnd;
use orb_api::mem;

use crate::audio::Music;
use crate::game::song::{name_at, same_song};
use crate::game::{
    Boundary, Building, Call, Face, FileOpen, FrameCalls, FrameEntered, Game, Hooks, Memory, Menu,
    Pad, PanelTile, Patch, Read, Reading, Rect, RunStart, State, Threads,
};
use crate::log;

/// th10 — 東方風神録, Mountain of Faith — as the [`Game`] whose addresses are in this module.
pub struct Th10;

/// The `IDirect3DDevice9` this game's one `CreateDevice` made — see *The window, the device and the
/// frame* in `asm.md`, where the twelve slots it is called through are.
const G_D3D_DEVICE: usize = 0x0049_1c30;

/// The window object the frame is called on, whose `+0x0` is the HWND.
const G_GAME_WINDOW: usize = 0x0049_24f0;

/// The frame, `stdcall` on [`G_GAME_WINDOW`] — see `asm.md`, which has it in order.
///
/// **orb's own frame leaves out the `0x44c150` this one makes on its way out**, and needs nothing in its place:
/// WinMain's teardown makes the same call on the same thread's object before the process goes, so the thread
/// is stopped whichever frame left.
const RENDER: usize = 0x0043_9390;
/// `push ebp; mov ebp,[esp+0x8]`, which is the five bytes the jump orb writes needs and ends on an
/// instruction.
const RENDER_PROLOGUE: &[u8] = &[0x55, 0x8b, 0x6c, 0x24, 0x08];

/// The chain, whose calc list the frame walks through [`RUN_CALC_CHAIN`] and whose draw list through
/// [`RUN_DRAW_CHAIN`] — see *The chain* in `asm.md`.
const G_CHAIN: usize = 0x0049_1be4;
const RUN_CALC_CHAIN: usize = 0x0044_9c00;
const RUN_DRAW_CHAIN: usize = 0x0044_9d40;
/// `sub esp,0x8; push ebx; mov ebx,[0x4660b0]` — the calc walk's first nine bytes.
const RUN_CALC_CHAIN_PROLOGUE: &[u8] =
    &[0x83, 0xec, 0x08, 0x53, 0x8b, 0x1d, 0xb0, 0x60, 0x46, 0x00];
/// `sub esp,0x8; push ebx; push ebp` — the draw walk's first five.
const RUN_DRAW_CHAIN_PROLOGUE: &[u8] = &[0x83, 0xec, 0x08, 0x53, 0x55];

/// The sound manager, and its call once a frame after the update — `stdcall` on it, see *The sound*.
const G_SOUND_MANAGER: usize = 0x0049_2590;
const PLAY_SOUNDS: usize = 0x0043_ddf0;
/// The id of the thread the music is streamed on, which `CreateThread` writes here as it makes it (0x43d6fa).
const SOUND_MUSIC_THREAD: usize = 0x614;
/// **The manager's extent**, the `0x14b4` words its constructor zeroes (0x43cbf3), with the event the music's
/// thread waits on (`+0x520c`) and that thread's id and handle (`+0x614`, `+0x618`) inside it.
const SOUND_MANAGER_BYTES: usize = 0x52d0;
/// **The wrapper DirectSound is kept in**, a `new(4)` holding the `IDirectSound8` (0x43cda4 through 0x43cdd9).
const SOUND_DEVICE: usize = 0x610;
const SOUND_DEVICE_BYTES: usize = 4;
/// **The queue of the sound's commands**, whose head is the command being carried out and nought for none
/// (0x43de0d through 0x43de18).
const SOUND_COMMANDS: usize = 0x1f88;
/// **Its 31 slots of `0x10c` bytes**, each `{kind, argument, phase, name}` (0x43e460 through 0x43e4c5), and the kind
/// 0x420b10 queues a song's play as, its argument the song's slot.
const SOUND_COMMAND_SLOTS: usize = 31;
const SOUND_COMMAND_BYTES: usize = 0x10c;
const SOUND_COMMAND_ARGUMENT: usize = 0x4;
const SOUND_A_PLAY: u32 = 2;
/// **The music's stream**, a `new(0x78)` 0x44d6a0 builds (0x44cb2d, 0x44cb51): the `new(4)` block holding its
/// `IDirectSoundBuffer` at `+0x4` (0x44d6a3 through 0x44d6b9), and the source it reads through at `+0xc`, a
/// `new(0x94)` whose `+0x90` is the record of `thbgm.fmt` being read (0x44c969, 0x44c984).
const MUSIC_STREAM: usize = 0x5208;
const MUSIC_STREAM_BYTES: usize = 0x78;
const STREAM_BUFFER: usize = 0x4;
const STREAM_BUFFER_BYTES: usize = 4;
const STREAM_SOURCE: usize = 0xc;
const MUSIC_SOURCE_BYTES: usize = 0x94;
const SOURCE_RECORD: usize = 0x90;
/// **A record of `thbgm.fmt`**: the track's file name at its start, which a play looks the record up by — 0x43d2a0
/// cuts the path it is handed past its last `/` (else its last `\`) and holds what is left against the start of
/// each `0x34`-byte record, a byte at a time to the terminating zero (0x43d30b through 0x43d339) — and after the
/// name's 16 bytes where its track is in `thbgm.dat` and how long it is (0x43d8b0, 0x43d8c6).
const RECORD_NAME: usize = 0x0;
const RECORD_NAME_BYTES: usize = 0x10;
const RECORD_AT: usize = 0x10;
const RECORD_BYTES: usize = 0x14;
/// **The manager's two songs a stage loads**, `0x100` bytes of path apiece from `+0x4108`: the build asks for the
/// stage record's `+0x10` into slot 0 and its `+0x14` into slot 1 (0x417bb4 through 0x417bd1, by 0x420a90, which
/// writes `wav` over the extension), the manager's load copies the path into the slot (0x43d824 through
/// 0x43d83b), and a play by slot hands that path to the lookup above (0x43e13e through 0x43e155). The stage's
/// start plays slot 0 (0x4183d6 through 0x4183e1) and the conversation before the stage's own boss slot 1
/// (0x416a91 through 0x416a9c) — so slot 1 is the fight's song.
const SOUND_SONGS: usize = 0x4108;
const SONG_PATH_BYTES: usize = 0x100;
const THE_FIGHTS_SONG: usize = 1;

/// **The supervisor's seven locks**, `CRITICAL_SECTION`s of `0x18` from `0x492274` with a byte of depth apiece
/// after them to `0x492323` (0x421420, 0x421c50 through 0x421c88) — the chain's, the file loader's, the log's,
/// the scene switch's and the thread start's among them, which another thread may be holding as a snapshot is
/// taken.
const G_LOCKS: usize = 0x0049_2274;
const LOCKS_BYTES: usize = 0x0049_2324 - 0x0049_2274;

/// The present, which takes nothing and pops nothing — see `asm.md`.
const PRESENT: usize = 0x0043_91f0;

/// **The sprite manager**, whose queue of quads the frame draws out either side of its draw chain.
const G_SPRITES: usize = 0x0049_1c10;
/// `0x442f50`, entered with the manager in `esi`: whatever the queue holds drawn and the queue emptied.
const DRAW_THE_QUEUE: usize = 0x0044_2f50;
// Handed over for a laid-out game to answer with a function of its own — see `handed_over!` — as are the
// camera's, the fog's, the options' and the pause menu's resume below: a frame that jumped into memory nothing
// has mapped is one no e2e test could reach at all.
handed_over!(
    DRAW_THE_QUEUE_AT,
    the_queues_draw,
    set_the_queues_draw,
    DRAW_THE_QUEUE
);
/// The queue's count, and its two cursors, which the frame puts back at the queue's start as its scene
/// begins (0x4394a7 through 0x4394b7).
const SPRITES_QUEUED: usize = 0x003a_dac8;
const SPRITES_QUEUE_STARTS: usize = 0x003a_dacc;
const SPRITES_WRITE_AT: usize = 0x0072_dacc;
const SPRITES_DRAWN_FROM: usize = 0x0072_dad0;
/// **The manager's 33 `.anm` files**, a pointer apiece from `+0x3ad06c` (`cmp 0x21` at 0x44775b): a file is a
/// `new(0x130)` (0x44712f) whose textures are `+0x10c` entries of `0x10` from `+0x120`, an `IDirect3DTexture9`
/// first in each (0x4471b8 through 0x4471dc), and whose sprites are `+0x114` of `0x44` from `+0x118`, each with
/// its texture's pointer copied in at `+0x4` (0x4475f0 through 0x447602). Every load and every free of one is
/// a build's or a destructor's, so none changes inside a stage.
const ANM_FILES_AT: usize = 0x003a_d06c;
const ANM_FILES: usize = 0x21;
const ANM_TEXTURES: usize = 0x10c;
const ANM_TEXTURES_AT: usize = 0x120;
const ANM_TEXTURE_BYTES: usize = 0x10;
const ANM_SPRITES: usize = 0x114;
const ANM_SPRITES_AT: usize = 0x118;
const ANM_SPRITE_BYTES: usize = 0x44;
/// How many of each a file is believed to hold at most, which is the bound on a count read off memory.
const ANM_TEXTURES_AT_MOST: u32 = 0x100;
const ANM_SPRITES_AT_MOST: u32 = 0x1000;
/// **The surfaces the screen's captures are made in**, four arrays of 32 pointers from `+0x3ad4e0` and their
/// records after them to `+0x3ad6e0`, made and released by the captures 0x448270 and 0x448450.
const SURFACES_AT: usize = 0x003a_d4e0;
const SURFACES_END: usize = 0x003a_d6e0;

/// **The camera the frame's update is set up under**: `0x421480`, entered with the camera in `edi`, sets
/// its two transforms — and the frame then sets the viewport the camera keeps at `+0xcc`, the camera having
/// been named in [`G_THE_CAMERA`] first (0x4393f4 through 0x439419).
const SET_THE_CAMERA: usize = 0x0042_1480;
handed_over!(
    SET_THE_CAMERA_AT,
    the_cameras_set,
    set_the_cameras_set,
    SET_THE_CAMERA
);
const THE_CAMERA: usize = 0x0049_1e94;
const G_THE_CAMERA: usize = 0x0049_1fac;
const CAMERA_VIEWPORT: usize = 0xcc;
/// And the word the frame writes `1` into after it (0x439425).
const G_A_CAMERA_IS_SET: usize = 0x0049_1fb0;

/// **The fog**, which the frame sets off under its scene: `0x420cd0` entered with the supervisor in `edi`,
/// and the dword [`G_FOG_FORCED`] forced to `0xff` before it (0x4394c2, 0x4394cc), so that it draws the queue,
/// puts that dword at nought and sets the fog off on every frame (0x420cd6 through 0x420cfa).
const SET_THE_FOG: usize = 0x0042_0cd0;
handed_over!(SET_THE_FOG_AT, the_fogs_set, set_the_fogs_set, SET_THE_FOG);
const G_SUPERVISOR: usize = 0x0049_1c28;
const G_FOG_FORCED: usize = 0x0049_2378;

/// **One of the game's own functions whose argument arrives in a register it chose**, called with that
/// register holding it.
///
/// **Saved and put back inside the call and not handed to the compiler as an operand**, because on x86 the
/// compiler keeps `esi` for itself — it is the base pointer of a frame it realigns — and an operand in it is
/// refused. `edi` goes the same way, so that the two read as one shape.
///
/// # Safety
/// `function` must be one of this game's that reads its argument out of that register, run on the game's
/// main thread with whatever it wants of the frame.
unsafe fn with_esi(function: usize, esi: usize) {
    unsafe {
        std::arch::asm!(
            "push esi",
            "mov esi, {esi}",
            "call {function}",
            "pop esi",
            esi = in(reg) esi,
            function = in(reg) function,
            clobber_abi("C"),
        )
    };
}

/// See [`with_esi`] — **and handed to the compiler as an operand**, `eax` being a register it lets go of, and
/// one the call is free to leave anything in.
///
/// # Safety
/// As [`with_esi`].
unsafe fn with_eax(function: usize, eax: usize) {
    unsafe {
        std::arch::asm!(
            "call {function}",
            function = in(reg) function,
            inout("eax") eax => _,
            clobber_abi("C"),
        )
    };
}

/// See [`with_esi`].
///
/// # Safety
/// As [`with_esi`].
unsafe fn with_edi(function: usize, edi: usize) {
    unsafe {
        std::arch::asm!(
            "push edi",
            "mov edi, {edi}",
            "call {function}",
            "pop edi",
            edi = in(reg) edi,
            function = in(reg) function,
            clobber_abi("C"),
        )
    };
}

/// **The float every step of a frame is scaled by**, the game's own speed: a bullet's move is its velocity
/// times this (0x40643e), a laser's growth (0x41d4d8) and the player's own step (0x425404) the same.
const G_GAME_SPEED: usize = 0x0047_6f78;

/// **The player**, a `0x4478`-byte object 0x425020 allocates and 0x4246c0 builds, which stores itself here
/// at 0x4247aa. Its calc job is 0x425730 at priority `0x10`, ahead of the lasers' `0x13` and the bullets'
/// `0x14`, so a frame's bullets are tested against the player where the frame's own move put it.
const G_PLAYER: usize = 0x0047_7834;
/// Where it is, in the field's own coordinates: two floats, written as the whole hundredths it is kept in
/// at `+0x3cc` times `0.01` (0x42549e through 0x4254c2).
const PLAYER_POSITION: usize = 0x3c0;
const PLAYER_HUNDREDTHS: usize = 0x3cc;
/// **What a stage's replay record keeps of the player besides where it is** — see `resume::Numbers`: its past
/// places (0x42a512 through 0x42a520), whether it is focused (0x42a58f, written at 0x42518a), and the four
/// options' numbers, four pairs apiece from `+0x32d4` in steps of `0x98` (0x42a528 through 0x42a56a).
const PLAYER_PLACES: usize = 0x436c;
const PLAYER_FOCUSED: usize = 0x4474;
const PLAYER_OPTIONS: usize = 0x32d4;
const PLAYER_OPTION_BYTES: usize = 0x98;
/// **An option's flag the playback leaves at nought**: 0x428e10 sets it to 1 for the four as it puts the place
/// back (0x428e4a through 0x428e5c), and 0x42a66f through 0x42a681 puts it at nought once the options are in.
const OPTION_MOVED: usize = 0x58;
/// **The player's options made again for the power it has**, `stdcall` on the player (`push eax; call` at
/// 0x42a5ef): the count of them is the power over twenty (0x426f73 through 0x426f92), and the playback calls it
/// between the places and the options' own numbers.
const OPTIONS_FOR_THE_POWER: usize = 0x0042_6f70;
handed_over!(
    OPTIONS_FOR_THE_POWER_AT,
    the_options_made,
    set_the_options_made,
    OPTIONS_FOR_THE_POWER
);
/// **What a place in hundredths is multiplied by to be a place**, the float at `0x470b00` 0x428e2c reads.
const HUNDREDTH: f32 = 0.01;
/// **The half of its box on each axis**, which is what both tests build the box it is tested with from
/// (0x425f00 through 0x425f97): half of `[0x476fb8 + character * 4]` (0x4249eb through 0x424a0b), which holds
/// `2.0` for Reimu and `3.5` for Marisa, so the half is `1.0` and `1.75`.
const PLAYER_HALF: usize = 0x41c;
/// **Its four speeds, in hundredths of a unit a frame**: straight and focused straight, then the diagonal
/// and the focused diagonal per axis — `+0x3d4`, `+0x3d8`, `+0x3dc`, `+0x3e0`, each the `.sht` file's own
/// float times a hundred (0x4248dc through 0x424926). The diagonal is the straight speed times
/// `sin(π/4)`, worked out as the file is loaded (0x426537 through 0x42656e).
const PLAYER_SPEEDS: usize = 0x3d4;
const SPEED_SCALE: f32 = 100.0;
/// **Its state**, the switch the calc job dispatches on (0x425739, the table at 0x426344): 0 coming back in,
/// `y` walked from `480` to `400` over sixty frames (0x425751 through 0x42590f); 1 playing; 2 dying, the life
/// gone (0x4269d0 decrements it at 0x426a15) and thirty frames on the run over or the player back; and 4 hit.
const PLAYER_STATE: usize = 0x458;
/// Playing: the bomb asked for, and the move (0x425964 through 0x425a8c).
const PLAYER_STATE_NORMAL: i32 = 1;
/// **Hit, and inside the eight frames a bomb takes it back in** — `cmp [player+0x478], 8` at 0x425a96,
/// set by the hit at 0x426cff.
const PLAYER_STATE_HIT: i32 = 4;
/// **The frames nothing can hit the player for**, the current count of a timer that counts down (0x425e46
/// through 0x425eaf) with its float after it: six as a hit is taken, 180 at a death, 280 as the player comes
/// back in, 270 for a bomb (0x426ea5, 0x426ac4, 0x425ceb, 0x4259be).
const PLAYER_UNHITTABLE_FOR: usize = 0x4310;
/// Its previous count and its float, which the constructor writes beside it as `119` and `120.0` (0x424d14
/// through 0x424d28).
const PLAYER_UNHITTABLE_BEFORE: usize = 0x430c;
const PLAYER_UNHITTABLE_AS_A_FLOAT: usize = 0x4314;
/// **The current count of the player's three timers a frame steps**: the shot's at `+0x464` — held at -1 while
/// no enemy is up or a conversation is (0x4262ce through 0x42631f), 0 on the first frame the shot is held and
/// round 0 to 15 while it stays held (0x4281d0) — the state's at `+0x478`, and the one at `+0x48c` every update
/// steps (0x4261f2), which holds the focus off for its first four frames (`cmp [edi+0x48c],0x4` at 0x425177).
/// Each is a timer of the game's whose previous count is the word before and whose float is the word after —
/// `+0x490` is the last one's float, which an `int` read of it gives the bits of.
const PLAYER_SHOT_TIMER: usize = 0x464;
const PLAYER_STATE_TIMER: usize = 0x478;
const PLAYER_UPDATE_TIMER: usize = 0x48c;
/// **The player's shots in flight**, `0x80` of `0x5c` bytes from `+0x49c`, each free while its `+0x40` is
/// nought (0x427ebe). Its first `0x44` bytes are its own — its age, where it is, its speed and angle, its
/// state — and past them two sprites and a homing target, which are pointers into the heap.
const PLAYER_SHOTS: usize = 0x49c;
const PLAYER_SHOT_BYTES: usize = 0x5c;
const PLAYER_SHOT_SLOTS: usize = 0x80;
const PLAYER_SHOT_STATE: usize = 0x40;
const PLAYER_SHOTS_OWN_BYTES: usize = 0x44;
/// **The shot's step, `0x4281d0`, entered with the player in `eax`**: the timer started on a frame the shot is
/// held, the frame's shots fired at its count and the count stepped — reading the shot off the run's own word
/// [`G_RUN_INPUT`] — which the player's calc calls only with an enemy up and no conversation (0x4262b4 through
/// 0x4262c7), and **`0x428280`, `stdcall` on the player**, every shot in flight moved, which it calls straight
/// after either way (0x42632d).
const SHOT_STEP: usize = 0x0042_81d0;
handed_over!(SHOT_STEP_AT, the_shot_step, set_the_shot_step, SHOT_STEP);
const SHOTS_MOVED: usize = 0x0042_8280;
handed_over!(
    SHOTS_MOVED_AT,
    the_shots_moved,
    set_the_shots_moved,
    SHOTS_MOVED
);
/// **The stage's background, its `.std`**: the object at `0x4776e8` (the one the last stage's is kept at
/// [`G_KEPT_STD`] through a transition, 0x417f10), and its calc job `0x402720`, entered with the object in
/// `eax` (0x403050), which steps its script and copies its camera over camera 0 at [`G_CAMERA`] every frame
/// (0x402835) — the camera an enemy made by instructions 270 and 271 is placed by and every enemy of flag
/// `0x40000` rides along with (0x40e9e7, 0x40de91).
const G_STAGE_BACKGROUND: usize = 0x0047_76e8;
const BACKGROUND_CALC: usize = 0x0040_2720;
handed_over!(
    BACKGROUND_CALC_AT,
    the_backgrounds_calc,
    set_the_backgrounds_calc,
    BACKGROUND_CALC
);
const G_CAMERA: usize = 0x0049_1d7c;
/// **How many frames of its background a stage begun out of a fade has run ahead of one built from the menu**:
/// the new `.std`'s jobs go on at the fade's first frame (0x4181f0 → 0x404530) and its calc skips that frame
/// alone (0x402738), and `main` is spawned at the fade's thirtieth (0x41840b), where a stage built fresh switches
/// its `.std` on and spawns `main` on the one frame (0x41821c through 0x418299). Measured as what it does: a
/// pick-up of stage 4 had the enemy of flag `0x40000` that comes on at frame 31 at y 176.0 where the run had it
/// at 190.5 — 29 frames of a camera moving 0.5 a frame.
const A_FADE_RUNS_THE_BACKGROUND_AHEAD_BY: usize = 29;
/// **The word the run acts on**, the copy the replay's calc job makes of [`G_INPUT`] at priority `0xb`, ahead of
/// the player's `0x10` (0x42a3d0 → 0x4297d0) — `[0x474e5c] = [0x474e30] & 0x1f7` recording.
const G_RUN_INPUT: usize = 0x0047_4e5c;

/// **The rectangle the player is held inside**, as a left, a top and a size: `x` in `-184` to `184` and `y`
/// in `32` to `432`, the clamp of the hundredths at 0x425446 through 0x425494.
const PLAYER_BOUNDS: (f32, f32, f32, f32) = (-184.0, 32.0, 368.0, 400.0);

/// **The bullet manager**, a `0x3e0b54`-byte object 0x406060 allocates, whose pointer is stored at
/// 0x4060bf.
const G_BULLET_MANAGER: usize = 0x0047_76f0;
/// Its array of bullets: `0x7d1` slots of `0x7f0` bytes from `+0x60`, built by the vector constructor at
/// 0x406097, of which the walk visits the first `0x7d0` (0x4065f6) — the last is a slot whose state is 5
/// and which ends the free-slot search.
const BULLETS_AT: usize = 0x60;
/// How many survived the manager's last walk, set to nought as the walk starts and counted up (0x4065c9,
/// 0x40664c).
const BULLETS_ALIVE: usize = 0x5c;
const BULLET_BYTES: usize = 0x7f0;
const BULLET_SLOTS: usize = 0x7d0;
/// Its flags, of which bit 1 is the one the active branch asks before it tests the player (0x406486) and
/// the spawn sets (0x406b8b).
const BULLET_FLAGS: usize = 0x0;
const BULLET_MAY_HIT: u32 = 1 << 1;
/// Where it is and its step a frame, three floats each; the step times [`G_GAME_SPEED`] is what its own
/// update adds (0x40643e through 0x406480).
const BULLET_POSITION: usize = 0x3b4;
const BULLET_STEP: usize = 0x3c0;
/// **Its box**, a width and a height the test builds `± half` from (0x4266b3 through 0x4266dc), written at
/// the spawn off the table at `0x4741e0` by the bullet's sprite (0x406c3d through 0x406c4d).
const BULLET_SIZE: usize = 0x3f0;
/// **Its state, a word**: 0 a free slot, 1 active and tested, 2 coming out, 3 cancelled and going — the
/// switch at 0x40625f.
const BULLET_STATE: usize = 0x446;
/// **Its age**, the frame of a timer the manager's update steps for every bullet it keeps (0x40664f through
/// 0x406655) and the spawn starts at nought (0x406a8c) — what tells the bullet a slot holds from the one it held
/// the frame before.
const BULLET_AGE: usize = 0x3fc;
/// **The [`BULLET_AGE`] a bullet coming out turns active at**, the same for every bullet: read off
/// `--trace-the-world` on 2026-10-04 over the bullets of the Extra's cards 99, 108 and 101, 867, 4,218, 4,304
/// and 3,328 of them, each first read in state 1 at age `9` and in state 2 at every age before it. Card 99's
/// run was hit at `158.3,373.7` by one of them, the rules having waited inside it as ground that never turns.
const A_BULLET_COMES_OUT_FOR: i32 = 9;
/// **Which of its changes the bullet is at**: the index into its list of up to `0x12` changes its script laid
/// on it — a speed, an aim, a sprite — each `0x18` bytes from `+0x464` with its kind at `+0x10` of it; the
/// update runs the one indexed and steps past it once it has run (0x406da2 through 0x407329), and the bits of
/// the ones still running are at `+0x43c`. `神符「杉で結ぶ古き縁」`'s fire goes `2`, `4`, `5`: thrown out, aimed
/// at the run, and aimed at it again from where it is at a slow `1` a frame (read off `--trace-the-world` on
/// 2026-10-02, `the_nearest_bullets`).
const BULLET_NEXT_CHANGE: usize = 0x45c;
/// **How many of the bullets nearest the run the world's record carries, and which words of each**: from the
/// place to past the state, as raw words.
const NEAREST_BULLETS: usize = 24;
const NEAREST_BULLETS_FROM: usize = BULLET_POSITION;
const NEAREST_BULLETS_TO: usize = 0x460;

/// **The laser manager**, a `0x45c`-byte object 0x41c290 allocates and stores at 0x41c2b8.
const G_LASER_MANAGER: usize = 0x0047_781c;
/// The first laser of its list — the list's head is a laser of its own embedded at `+0x10`, so the first
/// real one is that one's next (0x41c331) — and each laser's next at `+0x8` (0x41c345).
const LASER_FIRST: usize = 0x18;
const LASER_NEXT: usize = 0x8;
/// How many lasers are on the list (0x41c57f through 0x41c591).
const LASERS_ALIVE: usize = 0x438;
/// **How many the manager holds at most**, the create refusing one past `0x100` (0x41c510 through
/// 0x41c51e).
const LASERS_AT_MOST: usize = 0x100;
/// **Which of the two kinds it is, by its vtable**: `0x46da60` travels and has no warning, and `0x46da10`
/// stays anchored and goes through a warning line, a widening, its whole width and a shrinking — no other
/// laser vtable is stored anywhere in the exe (0x41c5b9, 0x41c689).
const LASER_KIND_ANCHORED: usize = 0x0046_da10;
/// Its phase, which for the anchored kind is 3 the warning line, 4 widening, 2 whole and 5 shrinking —
/// the table at 0x41ea30 — and is tested in 4 and 2 alone (0x41e886 through 0x41e8a6).
const LASER_PHASE: usize = 0xc;
const LASER_PHASE_FULL: i32 = 2;
const LASER_PHASE_WARNING: i32 = 3;
const LASER_PHASE_WIDENING: i32 = 4;
/// The frames it has been in its phase, the timer's own count, which each change of phase starts again.
const LASER_TIMER: usize = 0x14;
/// Where it starts, three floats, which way it points, how long it is now and how wide it is drawn.
const LASER_ORIGIN: usize = 0x24;
const LASER_ANGLE: usize = 0x3c;
const LASER_LENGTH: usize = 0x40;
const LASER_WIDTH: usize = 0x44;
/// Its speed, which is what the length grows by a frame until it reaches its most (0x41d4cf, 0x41e706).
const LASER_SPEED: usize = 0x48;
/// Its own number, counted up by the manager as each is made (0x41c521 through 0x41c57c).
const LASER_ID: usize = 0x54;
/// **The anchored kind's own**: the width it fires at, how long it grows to, and how many frames its
/// warning line stands (0x41e819, 0x41e706, 0x41e7de).
const LASER_WIDTH_WHOLE: usize = 0x44c;
const LASER_WARNING_FRAMES: usize = 0x454;
const LASER_LENGTH_AT_MOST_ANCHORED: usize = 0x444;
/// **And the travelling kind's most**, which it grows to before its origin starts moving (0x41d4cf).
const LASER_LENGTH_AT_MOST_TRAVELLING: usize = 0x434;

/// **The sounds the frame hands over, called the way the game calls them**: the manager pushed.
///
/// A function of orb's own because the frame's call is `stdcall` and `FrameCall` calls in the convention
/// the rest of the frame's calls are in, the `this` in `ecx` — see `runtime::OnAnAddress`. Not
/// `FrameCalls::orbs_own`: a laid-out game hands its own sound over in its place, as it does every other
/// game's.
extern "fastcall" fn plays_the_sounds(manager: usize) {
    let sounds: extern "stdcall" fn(usize) -> i32 = unsafe { std::mem::transmute(PLAY_SOUNDS) };
    sounds(manager);
}

/// **The pad's half of the keyboard's read**, whose answer is the word the game stores — see
/// `game::Read::PadHalf`. `push ebp; mov ebp,esp; and esp,-8`, six bytes ending on an instruction; entered
/// with the keyboard's word in `ecx`, the player and the pad pushed, and left through `ret 0x8` (0x44a32b).
/// Its one caller that runs is the read at 0x44a8b8, which the supervisor's calc job makes first thing in
/// every walk (0x41ffad, priority 1).
const INPUT_PAD_HALF: usize = 0x0044_a190;
const INPUT_PAD_HALF_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xe4, 0xf8];

/// **The replay's object built for a stage built fresh**, `0x429610`, `stdcall` on two (`ret 0x8` at 0x4296b2)
/// and called from one place, the build's 0x417a86 on the loader's thread — which is the seam a resumed run's
/// seed goes in at (`Hooks::stage_building`): past the run's defaults (0x4178ab through 0x417a6d), where nothing
/// has drawn from the generator, and before the replay reads the seed as a recording's (0x429081) or writes one
/// back as a playback's (0x4292d2) and before any of the stage's objects is built (0x417a9e on).
///
/// **A fresh build is every stage a resume starts**: the stage after one won goes through `0x42a6a0` instead
/// (0x417afb), and a run's start and a retry both build fresh (0x421a0a, 0x421a80).
const REPLAY_BUILT: usize = 0x0042_9610;
/// `push -1; push 0x46566b` — seven bytes, the second an absolute address and not a relative one.
const REPLAY_BUILT_PROLOGUE: &[u8] = &[0x6a, 0xff, 0x68, 0x6b, 0x56, 0x46, 0x00];

/// **The input**, `0x6a` bytes a player the read fills: the word this frame at `+0x0`, the frame before's at
/// `+0x2`, the buttons gone down this frame at `+0x6` and those let go at `+0x8` (0x44a8bd through 0x44a926),
/// and an auto-repeat word at `+0x4` off sixteen counters from `+0xa`, a button held firing in it on its 26th
/// frame and every eighth after (0x44a8d2 through 0x44a90a). The menus take decide and cancel off the buttons
/// gone down and move their cursors on those and the repeat together (0x422d1c, 0x42d52b); the run reads a
/// copy of the first that the replay's own job makes at priority `0xb`
/// (0x4297d0), after this read and before the player's move at `0x10` — so the player moves on the frame's
/// own keys.
const G_INPUT: usize = 0x0047_4e30;
const G_INPUT_LAST: usize = 0x0047_4e32;

/// The buttons, as the read folds the keyboard's keys and the pad's buttons into them (0x44a63c through
/// 0x44a8b1, 0x44a205 through 0x44a291).
const BUTTON_SHOT: u16 = 0x0001;
const BUTTON_BOMB: u16 = 0x0002;
const BUTTON_FOCUS: u16 = 0x0004;
const BUTTON_MENU: u16 = 0x0008;
const PAD_UP: u16 = 0x0010;
const PAD_DOWN: u16 = 0x0020;
const PAD_LEFT: u16 = 0x0040;
const PAD_RIGHT: u16 = 0x0080;
/// Ctrl, which runs a conversation on where the conversation lets it (0x416a3a).
const BUTTON_SKIP: u16 = 0x0100;
/// **What every menu of this game takes as decide and as cancel**: Z or Enter, and X or Esc
/// (`[0x474e36] & 0x1001` at 0x422d78 and 0x42d5ff, `& 0xa` at 0x42d59c and 0x43049c).
const MENU_DECIDE: u16 = 0x1001;
const MENU_CANCEL: u16 = 0x000a;
/// **What the run's copy keeps of the word**: `& 0x1f7` at 0x4297e9, which is every button but Esc and
/// nothing past the skip — and so what a replay of this game records.
const RUN_INPUT: u16 = 0x01f7;

/// **The pad's mapping**, the nine `i16` `th10.cfg` holds from its `+0x4`, laid at `+0x58` of the player's
/// input (0x4208ff through 0x420934) — a button's number, negative for none (0x44a209) — with the exe's own
/// `.data` holding `0, 1, 2, 3, -1, -1, -1, -1, 4` there for a configuration the file does not answer
/// (0x418c7a). **The read takes five of them**, each into a bit of the word (0x44a205 through 0x44a291).
const G_PAD_MAPPING: usize = 0x0047_4e88;
const PAD_MAPPED: [(usize, u16); 5] = [
    (0, BUTTON_SHOT),
    (1, BUTTON_BOMB),
    (2, BUTTON_FOCUS),
    (3, BUTTON_MENU),
    (8, BUTTON_SKIP),
];

/// **The pad DirectInput found**, joystick 0's at `[0x491c3c]`, with bit `0x400` of the input's flags set where
/// it is there (0x43bac5 through 0x43badd) — read in the `DIJOYSTATE2` format (`GetDeviceState` of `0x110` at
/// 0x44a3bb) with every axis' range ±1000 (0x43bb30), its two axes against the thresholds at `+0x16` and `+0x18`
/// of the configuration, 600 apiece where the file does not answer (0x418c4d, 0x44a457 through 0x44a4b4). And the
/// slots of `IDirectInputDevice8` the read calls: `Acquire` (0x44a351), `GetDeviceState` (0x44a3bb) and `Poll`
/// (0x44a33b).
const G_PAD_DEVICE: usize = 0x0049_1c3c;
/// The `JOYCAPSA` of joystick 0 the startup check fills (0x44a11b) — `x_min` at `+0x24`, as the read takes it
/// (0x44a2a2).
#[cfg(any(test, feature = "sim"))]
const G_JOYSTICK_CAPS: usize = 0x0049_18b8;
const PAD_THROUGH_DIRECTINPUT: u32 = 0x400;
const CFG_PAD_ACROSS: usize = 0x0049_1d5e;
const CFG_PAD_DOWN: usize = 0x0049_1d60;
#[cfg(any(test, feature = "sim"))]
const DEVICE_ACQUIRE: usize = 7;
#[cfg(any(test, feature = "sim"))]
const DEVICE_GET_STATE: usize = 9;
#[cfg(any(test, feature = "sim"))]
const DEVICE_POLL: usize = 25;
const PAD_RANGE: i32 = 1000;

/// **The supervisor**, whose `+0x0` is the `hInstance` (0x438b51): the scene it is on and the scene wanted,
/// which the switch at 0x4218d0 moves the one to where they differ.
const G_SCENE: usize = 0x0049_1fb4;
const G_SCENE_WANTED: usize = 0x0049_1fb8;
/// The title, which 0x42cd50 builds (0x42197b).
const SCENE_TITLE: i32 = 4;
/// **A run**, which 0x4180e0 builds the game object of (0x4219f8 through 0x421a14).
const SCENE_RUN: i32 = 7;
/// **The run started over at its starting stage**, which the pause menu's item for it writes as the menu closes
/// (0x42316c) and the switch turns into [`SCENE_RUN`] built fresh (0x421a80 through 0x421abd).
const SCENE_START_OVER: i32 = 10;
/// **The stage after the one just won**, which the switch turns straight into [`SCENE_RUN`] with the run's
/// numbers carried (0x421a4e through 0x421a76) — so a run is still on while it is the scene.
const SCENE_NEXT_STAGE: i32 = 11;
/// The ending, and the staff roll inside it (0x40b940, 0x40b973).
const SCENE_ENDING: i32 = 14;

/// **The configuration**, `0x34` bytes read out of `th10.cfg` at 0x420870: the byte that says the game
/// makes a window, `1` for one — 0x420999 says `ウィンドウモードで起動します` where it is — and the dialog
/// that asks is answered into the same byte (0x438b92).
const CFG_WINDOWED: usize = 0x0049_1d65;

/// **The input's flags, and the bit that says the keyboard is read through DirectInput** — cleared with 0x400 at
/// the input's set-up (0x43ba90) and the device's own made at 0x43b8d0 into `[0x491c38]`; and the slot of
/// `IDirectInputDevice8::Unacquire`, eighth after `IUnknown`'s three and `GetCapabilities`, `EnumObjects`,
/// `GetProperty`, `SetProperty` and `Acquire`.
const G_INPUT_FLAGS: usize = 0x0049_1ff4;
const INPUT_THROUGH_DIRECTINPUT: u32 = 0x200;
const G_KEYBOARD_DEVICE: usize = 0x0049_1c38;
const DEVICE_UNACQUIRE: usize = 8;

/// **The title**, a `0x5acc`-byte object 0x42c920 builds and stores here at 0x42c9e2: which of its screens
/// is up (the switch at 0x42ce37), where that screen is in its own steps, the cursor, and the frames since the
/// screen or its step was set.
const G_TITLE: usize = 0x0047_784c;
const TITLE_SCREEN: usize = 0x1c;
const TITLE_STEP: usize = 0x20;
const TITLE_CURSOR: usize = 0x24;
// Read by nothing of orb's — whether a screen takes a press is its step, see [`TITLE_STEP_READING`] — and
// kept for the laid-out title, which counts it as the game does.
#[cfg(any(test, feature = "sim"))]
const TITLE_FRAMES: usize = 0x2b4;
/// **The main menu**, eight items the cursor wraps over (0x42d43b), each read off what it does on decide
/// (the table at 0x42d900): a run, the Extra — with the difficulty put at 4 (0x42d790) — practice, the
/// replays and the player's data.
const SCREEN_MAIN_MENU: i32 = 2;
const TITLE_ITEM_START: i32 = 0;
const TITLE_ITEM_EXTRA: i32 = 1;
const TITLE_ITEM_PRACTICE: i32 = 2;
const TITLE_ITEM_REPLAY: i32 = 3;
const TITLE_ITEM_PLAYER_DATA: i32 = 4;
/// **The shot select**, screen 8 (0x430a60), whose cursor is the shot and whose decide asks for the run
/// (0x430f5a); and the difficulty the Extra is, 4 (0x42d790).
const SCREEN_SHOT_SELECT: i32 = 8;
const DIFFICULTY_EXTRA: i32 = 4;
/// **The step of a screen that reads a key**, after the one that waits out the frames it ignores them for:
/// the main menu's first step runs until the title's frames at `+0x2b4` are past ten (0x42d4ca), and the second reads the keys
/// from its first frame (0x42d516) — entering it zeroes those frames (0x42c633), so they say nothing of it.
const TITLE_STEP_READING: i32 = 2;

/// **The game object**, `0x60` bytes 0x4180e0 builds for each stage: its flags — `0x10` from the pause
/// menu's opening (0x422af4), the game over (0x423244) and a run ended by its clear (0x4233e4), which only the
/// pause menu's resume takes off (0x422c38) — and `+0x5c`, `1` for a replay being played (0x41811d).
const G_GAME: usize = 0x0047_7810;
const GAME_FLAGS: usize = 0x58;
const GAME_PAUSED: u32 = 0x10;
/// And `0x4` while the stage's build runs, which the loader thread sets as it starts (0x41787a) and clears
/// as it ends (0x417c30), and which the game's own job answers by stepping nothing (0x41864b).
const GAME_BEING_BUILT: u32 = 0x4;
const GAME_REPLAYING: usize = 0x5c;
/// **The frames the game object has run**, the frame of its timer at `+0x10` (0x418784), and the flag it sets
/// while the stage before this one fades out under it: a stage after one won comes up with the old stage's
/// `.std` kept at `0x4776e4` (0x417f0b), and its first frame fades the two with `+0x58 |= 0x800` and nothing
/// of the stage begun (0x4181f2 through 0x418217), which starts thirty frames later with the flag taken off
/// (0x41840b through 0x418426). A stage built fresh has no `.std` kept and begins on its first frame.
const GAME_FRAMES: usize = 0x14;
const GAME_FADING_IN: u32 = 0x800;
const GAME_BEGINS_A_FADE_AT: i32 = 0x1e;
const G_KEPT_STD: usize = 0x0047_76e4;

/// **The pause object**, which runs the pause menu, the end of a run and its name entry: its state at
/// `+0x4` is 1 to 5 for the pause menu and 6 to 13 for the end of a run (0x4223f0, the tables at 0x4224b0
/// and 0x42438c).
const G_PAUSE: usize = 0x0047_7830;
const PAUSE_STATE: usize = 0x4;
/// **The pause menu's resume**, entered with the pause object in `esi` (0x4231a9): the game object's pause flag
/// taken off, the music let go on (the sound's command 7), and the game's speed put back from the pause
/// object's `+0x2c0` (0x422c30 through 0x422c73).
const UNPAUSE: usize = 0x0042_2c30;
handed_over!(UNPAUSE_AT, the_unpause, set_the_unpause, UNPAUSE);
const PAUSE_THE_RUN_IS_OVER: std::ops::RangeInclusive<i32> = 6..=13;
/// **The menu a run ends on and the state that leaves it**: 8 offers playing on, the replay's save and the title,
/// and 13 carries out the item chosen twelve frames after its timer starts over — the timer at `+0x10`, whose
/// frame is `+0x14` and which 0x405410 starts at nought (0x40543c through 0x405442). The cursor is `+0x24`.
const PAUSE_THE_RUN_ENDS_ON_A_MENU: i32 = 8;
const PAUSE_LEAVING_THE_END: i32 = 13;
const PAUSE_CURSOR: usize = 0x24;
const PAUSE_FRAMES: usize = 0x14;
const PAUSE_ITEM_THE_TITLE: i32 = 2;
/// **The name entry**, the replay's at state 11 and the ranking's at 12 over the same fields: the name, eight
/// characters and a nought at `+0x2b4`; the place in it at `+0x1e0`; and the grid's cursor at `+0xfc`, whose
/// cell 90 ends the entry (0x423b28 through 0x423ec8).
const PAUSE_NAMES_A_REPLAY: i32 = 11;
const PAUSE_NAMES_A_RANKING: i32 = 12;
const PAUSE_NAME: usize = 0x2b4;
const PAUSE_NAME_BYTES: usize = 8;
const PAUSE_NAME_TYPED: usize = 0x1e0;
const PAUSE_NAME_GRID: usize = 0xfc;
const PAUSE_THE_END_CELL: i32 = 90;
/// **And the clear's own name entry, which is the title's screen 15** (0x432cb0) after the ending: the name at
/// `+0x58dc`, the place in it at `+0x58e8` and the grid's cursor at `+0x58f4`, read in its second step
/// (0x432eda) over the same grid the pause object's entry walks — `[0x4746d8]`, both at 0x423dbb and 0x432fee —
/// so its end cell is the same 90; a decide there copies the name into the ranking and the score object's
/// `+0x1d878` (0x4330b1 through 0x43310c).
const SCREEN_NAME_ENTRY: i32 = 15;
const TITLE_NAME: usize = 0x58dc;
const TITLE_NAME_TYPED: usize = 0x58e8;
const TITLE_NAME_GRID: usize = 0x58f4;
/// **Whether the clear is ranked at all**, nought where it is: the first step sets it to 1 for a score the
/// ranking has no place for (0x432ed0), and the second then takes a decide straight to the third with no name
/// typed (0x432fe0).
const TITLE_NAME_NOT_ASKED: usize = 0x58ec;
/// **The replay's slots after a clear**, the title's screen 16 (0x433570), and the step its cancel takes
/// (0x4336af); and the frames the step setter puts back as it sets one, `+0x2b0` to -1 and `+0x2b4` and `+0x2b8`
/// to nought (0x42c659 through 0x42c665).
const SCREEN_REPLAYS_SLOTS: i32 = 16;
const SLOTS_STEP_CANCELLED: i32 = 4;
const TITLE_STEPS_FRAMES: usize = 0x2b4;

/// **Where the entry up keeps the name, the place in it and the grid's cursor**, which are the same three
/// fields at the pause object's places or the title's.
struct NameEntry {
    name: usize,
    typed: usize,
    grid: usize,
}

/// **The run's numbers**, a block the code hands over as `0x474c40`: the score over ten, the power from
/// nought to `100` — a bomb costing `20`, and `100` shown as `5.00` (0x418930, 0x4259d4) — the faith over
/// ten, the character, the shot, the lives, the difficulty with the Extra as 4, the stage from 1 with the
/// Extra as 7, the stage the run started at, and the frames since the stage began (0x418769).
const G_SCORE: usize = 0x0047_4c44;
/// The block itself, and as far as the trace's world takes it: past the run's flags at `0x474ca0`.
const G_RUN_NUMBERS: usize = 0x0047_4c40;
const RUN_NUMBERS_BYTES: usize = 0x80;
const G_POWER: usize = 0x0047_4c48;
const POWER_FULL: u16 = 100;
const POWER_A_BOMB_COSTS: u16 = 20;
const G_CHARACTER: usize = 0x0047_4c68;
const G_SHOT_TYPE: usize = 0x0047_4c6c;
const G_LIVES: usize = 0x0047_4c70;
const G_DIFFICULTY: usize = 0x0047_4c74;
const G_STAGE: usize = 0x0047_4c7c;
const G_STARTING_STAGE: usize = 0x0047_4c80;
const G_STAGE_FRAMES: usize = 0x0047_4c88;
/// **The rest of what a stage's replay record keeps of the run** — see `resume::Numbers`: the faith, and the
/// timer the frames it stands before it falls are counted on — its frame before, its frame and the same as a
/// float, which is what 0x42a930 writes (0x42a95d through 0x42a964); the continues, `0x474c98`, and the extends.
const G_FAITH: usize = 0x0047_4c4c;
const G_FAITH_TIMER_BEFORE: usize = 0x0047_4c54;
const G_FAITH_TIMER: usize = 0x0047_4c58;
const G_FAITH_TIMER_AS_A_FLOAT: usize = 0x0047_4c5c;
const G_CONTINUES: usize = 0x0047_4c90;
const G_474C98: usize = 0x0047_4c98;
const G_EXTENDS: usize = 0x0047_4c9c;
/// **The stage's record of what it loads**, which the build reads the stage's files through (0x417a58,
/// 0x417a93): `0x474788` on by `0x30` a stage, which the shot select writes as it asks for a run (0x430f16
/// through 0x430f4f) and a retry as it builds the starting stage again (0x421a9f through 0x421ab7).
const G_STAGE_LOADS: usize = 0x0047_7848;
const STAGE_LOADS: usize = 0x0047_4788;
const STAGE_LOADS_BYTES: usize = 0x30;
/// **The run's flags**: `0x10` a practice run (0x42d7d4), `0x20` the attract demo (0x42ce58).
const G_RUN_FLAGS: usize = 0x0047_4ca0;
const RUN_IS_A_PRACTICE: u32 = 0x10;
const RUN_IS_THE_DEMO: u32 = 0x20;
/// How many stages there are to count, the Extra the seventh.
const STAGES: i32 = 7;
const DIFFICULTIES: i32 = 5;

/// **The generator**, a word the step at 0x44b9b0 turns over, seeded once from `timeGetTime` as the game
/// starts (0x4201d0) and carried from one stage into the next — and the count of its draws beside it, which
/// every step adds to and a playback's seed starts at nought (0x4292d8).
const G_RANDOM: usize = 0x0049_18b0;
const G_RANDOM_DRAWS: usize = 0x0049_18b4;

/// **The enemy manager**, `0x68` bytes 0x40d6b0 builds with the stage's `.ecl`: the boss the fight is with,
/// slot 0 of its eight (ECL instruction 332 writes it at 0x40fffe), the list of enemies and how many are on
/// it. An enemy is alive exactly while it is on the list.
const G_ENEMY_MANAGER: usize = 0x0047_7704;
const ENEMY_BOSS: usize = 0x10;
const ENEMY_LIST: usize = 0x58;
const ENEMY_COUNT: usize = 0x60;
/// **The frames the enemy manager has run**, which go on through a boss and a conversation and take one step
/// more at every conversation's start (0x40e707 through 0x40e753).
const ENEMY_MANAGER_FRAMES: usize = 0x44;
/// And the whole of it, which 0x40d6b0 builds.
const ENEMY_MANAGER_BYTES: usize = 0x68;
/// An element of the list: the enemy, and the next element (0x40d751 through 0x40d79f).
const ELEMENT_NEXT: usize = 0x4;
/// **An enemy**, `0x2518` bytes: the time of its main script as a float (0x44fb96), where it is, the size of the
/// body the player is tested against (0x40e2d6), its life, the frames its current attack has run (0x40e572; set
/// to nought at each change of attack, 0x41284c), the frames its body is not tested for (0x40e2c6), and its flags.
const ENEMY_SCRIPT_TIME: usize = 0x8;
const ENEMY_POSITION: usize = 0x1068;
const ENEMY_BODY: usize = 0x10f4;
const ENEMY_ATTACK_FRAMES: usize = 0x115c;
const ENEMY_LIFE: usize = 0x23fc;
const ENEMY_BODY_OFF_FOR: usize = 0x2470;
const ENEMY_FLAGS: usize = 0x2480;
/// **The two bits that skip the body's test**, `test …,0x12` at 0x40e2bd.
const ENEMY_NO_BODY: u32 = 0x12;
/// **The one of them the stage's own scripts carry**: stage 2's `main` read flags `0x10` and the enemy it spawns
/// to run the stage `0x110`, and every enemy that came on after them with a body read `0x1120`, `0x3120` and
/// the like.
const ENEMY_HAS_NO_BODY: u32 = 0x10;
/// **How many enemies a walk of the list goes through at most**: a next pointer read wrong is a walk that
/// never ends otherwise, and a stage has never been measured holding this many.
const ENEMIES_AT_MOST: usize = 0x400;

/// **The spell card**, a `0x37b0`-byte object 0x408910 builds: the card's number, and its flags, bit 0 of
/// which is a card running (0x4092fa, 0x409c40).
const G_CARD: usize = 0x0047_76f4;
const CARD_NUMBER: usize = 0x3788;
const CARD_FLAGS: usize = 0x378c;

/// **The score object**, `0x1dcb4` bytes (0x42af36) read out of `scoreth10.dat` once as the game starts
/// (0x42ae7d): seven records of `0x437c` from `+0x8` (0x42aeb1), one for each shot type — `character * 3 + shot`
/// — and the seventh for all of them, each with an entry of `0x90` bytes for every one of the 110 cards from
/// `+0x59c` (0x432767): the card's name in the first `0x80`, then how many times it was captured and how many
/// attempted, each counted at the card's start and at its capture into the shot type's record and the
/// seventh's, held at 99,999 (0x409389 through 0x4093b8, 0x409d0e through 0x409d36).
const G_SCORE_OBJECT: usize = 0x0047_783c;
const RECORDS_AT: usize = 0x8;
const SCORE_RECORD_BYTES: usize = 0x437c;
const RECORDS: usize = 7;
const RECORD_OF_EVERY_SHOT_TYPE: usize = 6;
const SHOT_TYPES_A_CHARACTER: i32 = 3;
const CARDS_AT: usize = 0x59c;
const CARDS: usize = 110;
const CARD_RECORD_BYTES: usize = 0x90;
const CARD_NAME_BYTES: usize = 0x80;
const CARD_CAPTURES: usize = 0x80;
const CARD_ATTEMPTS: usize = 0x84;
const A_CARDS_COUNT_STOPS_AT: u32 = 99_999;
/// **The replay manager**, whose `+0x10` is 1 while a replay plays back: the card start counts nothing then
/// (`cmp DWORD PTR [edx+0x10],0x1` at 0x409312).
const G_REPLAY_MANAGER: usize = 0x0047_7838;
const REPLAY_MANAGER_PLAYS_BACK: usize = 0x10;
/// **Each shot type's practice**, an entry of 8 bytes for each difficulty but the Extra and each stage —
/// `difficulty * 6 + stage` — from `+0x4dc`: the stage's best practice score in the first four (0x4237d4), and
/// at `+0x5` whether it is offered, set as a run clears the stage (0x416c08) and read by the practice select
/// (0x4314be). **The stage is counted from 1 there**, the game's own `[0x474c7c]` at both, so entry 0 is no
/// stage's and Lunatic's sixth is entry 24 — which a count of `4 * 6` from nought left behind.
const PRACTICE_AT: usize = 0x4dc;
const PRACTICE_BYTES: usize = 8;
const PRACTICES: usize = 4 * 6 + 1;
const PRACTICE_OFFERED: usize = 0x5;
/// **The settings and the clears** at `+0x1d86c`, the file's `ST` record (0x42b152) — see [`score_file`].
const SETTINGS_AT: usize = 0x1_d86c;
/// **The clears inside them**, a byte a shot type from `+0x1d888` — `character * 3 + shot` — that the ending's
/// set-up marks (0x40b684 through 0x40b6e2) and the main menu lights its Extra from (0x42c850).
const CLEARS: std::ops::Range<usize> = 0x1_d888..0x1_d88e;
/// **A shot type's record made empty**, `stdcall` on the record (0x42aeb1): zeroed, headed, ten names and
/// scores for each difficulty and each card numbered — what a record the file does not hold starts as.
const A_RECORD_EMPTIED: usize = 0x0042_acb0;
handed_over!(
    A_RECORD_EMPTIED_AT,
    a_record_emptied,
    set_a_record_emptied,
    A_RECORD_EMPTIED
);
const CARD_IS_UP: u32 = 0x1;

/// **The item manager**, `0x21cec0` bytes 0x41aee7 allocates: its items from `+0x14`, `0x3f0` bytes each, of
/// which the first `0x96` take every type and the rest the small faith of type 8 alone (0x41bb15, 0x41bccd).
const G_ITEM_MANAGER: usize = 0x0047_7818;
const ITEMS_AT: usize = 0x14;
const ITEM_BYTES: usize = 0x3f0;
const ITEM_SLOTS: usize = 0x96;
/// **And every slot, the small faith's after them**: those are taken round as a ring from the cursor at
/// `+0x21ceb8` (0x41bccd), which is where the slots end — `(0x21ceb8 - 0x14) / 0x3f0`, `0x896` of them.
const ALL_ITEM_SLOTS: usize = 0x896;
/// **The slot of that ring the next small faith goes in** — see `resume::Numbers::small_faith_ring_at`.
const SMALL_FAITH_RING_AT: usize = 0x21_ceb8;
/// Where it is, its state — 1 and 2 falling, 3 and 4 flying to the player on their own (0x41b010 through
/// 0x41b2a4) — and its type, which the collection dispatches on (0x41b3c1).
const ITEM_POSITION: usize = 0x3ac;
const ITEM_STATE: usize = 0x3dc;
const ITEM_TYPE: usize = 0x3e0;

/// **The panel**, whose `+0x9eb8` is the conversation running and nought where none is (0x415df9,
/// 0x415490).
const G_PANEL: usize = 0x0047_770c;
const PANEL_CONVERSATION: usize = 0x9eb8;

/// **The bomb**, whose `+0x28` is what the player's own bomb asks to be nought before it lets one go
/// (0x425998) — one running.
const G_BOMB: usize = 0x0047_76ec;
const BOMB_GOING_OFF: usize = 0x28;

/// **The play area in the game's own 640x480 output**, which is the field put where its drawing puts it: a
/// sprite of the field is drawn at its position plus `224` across and `16` down (0x4066f0 through
/// 0x40670e), and the field is `384` wide — so its left edge is at 32.
const PLAY_AREA: Rect = Rect {
    left: 32.0,
    top: 16.0,
    width: 384.0,
    height: 448.0,
};
const BACK_BUFFER: (u32, u32) = (640, 480);

impl Th10 {
    /// **The buttons held in `buttons` — a bit per pad button — as the bits the mapping makes them**, which is
    /// what the read does with either device's buttons (0x44a205 through 0x44a291, 0x44a3dc through 0x44a455).
    ///
    /// # Safety
    /// Must run on the game's main thread, the mapping being read out of the game's memory.
    unsafe fn the_mapped_buttons(&self, buttons: u32) -> u16 {
        let mapping = unsafe { mem::read::<[i16; 9]>(G_PAD_MAPPING) };
        PAD_MAPPED
            .iter()
            .filter(|(at, _)| {
                // `checked_shl` and not `1 << button`, a number past the width of a mask being a file somebody
                // edited rather than a button, and negative naming none — the read's own `jl` at 0x44a210.
                u32::try_from(mapping[*at])
                    .ok()
                    .and_then(|button| 1u32.checked_shl(button))
                    .is_some_and(|bit| buttons & bit != 0)
            })
            .fold(0, |word, (_, bit)| word | bit)
    }

    /// **One read of the pad DirectInput found** — see `joystick::the_state_of` — and `None` where the input's
    /// flags say there is none.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_pads_own_state(&self) -> Option<crate::joystick::DeviceState> {
        let (device, _) = unsafe { self.its_pad_device() }?;
        unsafe { crate::joystick::the_state_of(device) }
    }

    /// **The word the read makes of the pad DirectInput found**: the buttons by the mapping, a button held
    /// where its byte has its top bit, and the two axes past the thresholds the configuration holds
    /// (0x44a3dc through 0x44a4b4). And the hat where `hats`, which the read has no look at.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_word_of_the_pads_own(
        &self,
        state: &crate::joystick::DeviceState,
        hats: bool,
    ) -> u16 {
        let buttons = state.buttons_held();
        let threshold = |at: usize| i32::from(unsafe { mem::read::<i16>(at) });
        let (across, down) = (threshold(CFG_PAD_ACROSS), threshold(CFG_PAD_DOWN));
        let (x, y) = (state.axes[0], state.axes[1]);
        let stick = directions((y < -down, y > down, x < -across, x > across));
        let hat = match hats {
            true => directions(crate::joystick::hat(state.hats[0])),
            false => 0,
        };
        let buttons = unsafe { self.the_mapped_buttons(buttons) };
        buttons | stick | hat
    }

    /// **The word the read makes of a pad orb sampled**, which is the read's winmm half (0x44a1f7 through
    /// 0x44a315): the buttons by the mapping and the axes past a quarter of their travel from its middle — the
    /// rule `joystick::axis` holds — and the hat where `hats`, which the read has no look at.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_word_of_a_sampled_pad(&self, reading: &Reading, hats: bool) -> u16 {
        let (up, down) = crate::joystick::axis(reading.y);
        let (left, right) = crate::joystick::axis(reading.x);
        let hat = match hats {
            true => directions(crate::joystick::hat(reading.pov)),
            false => 0,
        };
        let buttons = unsafe { self.the_mapped_buttons(reading.buttons) };
        buttons | directions((up, down, left, right)) | hat
    }

    /// The scene the supervisor is on, and the one it wants.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn scenes(&self) -> (i32, i32) {
        unsafe { (mem::read(G_SCENE), mem::read(G_SCENE_WANTED)) }
    }

    /// The title, while the front end has one.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn title(&self) -> Option<usize> {
        if unsafe { self.scenes() }.0 != SCENE_TITLE {
            return None;
        }
        mem::read_committed::<usize>(G_TITLE).filter(|it| *it != 0)
    }

    /// The object behind a pointer of the game's own, where there is one.
    fn object(at: usize) -> Option<usize> {
        mem::read_committed::<usize>(at).filter(|it| *it != 0)
    }

    /// **The record of `thbgm.fmt` the stream is reading**: the manager's stream, its source and the record the
    /// source was last put on (0x44dcba).
    fn the_record_playing() -> Option<usize> {
        let stream = Self::object(G_SOUND_MANAGER + MUSIC_STREAM)?;
        Self::object(Self::object(stream + STREAM_SOURCE)? + SOURCE_RECORD)
    }

    /// **The boss the fight is with**, off the manager's first slot.
    fn the_boss_enemy(&self) -> Option<usize> {
        Self::object(Self::object(G_ENEMY_MANAGER)? + ENEMY_BOSS)
    }

    /// **The latest time any of the stage's own scripts still alive reads**, in whole frames — see
    /// [`SCRIPT_CLOCK`] for why the latest.
    fn the_stage_scripts_time(&self, scripts: &[usize]) -> Option<i32> {
        self.the_enemies()
            .into_iter()
            .filter(|enemy| scripts.contains(enemy))
            .filter_map(|enemy| mem::read_committed::<f32>(enemy + ENEMY_SCRIPT_TIME))
            .map(|time| time as i32)
            .max()
    }

    /// **The enemies on the list**, in its order, [`ENEMIES_AT_MOST`] at most.
    fn the_enemies(&self) -> Vec<usize> {
        let mut enemies = Vec::new();
        let mut element =
            Self::object(G_ENEMY_MANAGER).and_then(|manager| Self::object(manager + ENEMY_LIST));
        while let Some(at) = element
            && enemies.len() < ENEMIES_AT_MOST
        {
            let Some(enemy) = Self::object(at) else {
                break;
            };
            enemies.push(enemy);
            element = Self::object(at + ELEMENT_NEXT);
        }
        enemies
    }

    /// **The stage's own scripts**: the enemies on the list with no body — `main`, and on stages where `main`
    /// does nothing but loop, the enemy it spawns to run the stage — taken on the stage's first frame, when
    /// nothing else has come on yet.
    fn the_stage_scripts(&self) -> Vec<usize> {
        self.the_enemies()
            .into_iter()
            .filter(|enemy| {
                mem::read_committed::<u32>(enemy + ENEMY_FLAGS)
                    .is_some_and(|flags| flags & ENEMY_HAS_NO_BODY != 0)
            })
            .collect()
    }

    /// **The stage's script frames**, off [`SCRIPT_CLOCK`], which this keeps one frame at a time.
    fn script_frames(&self, stage_frames: u32) -> i32 {
        SCRIPT_CLOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .at(
                stage_frames as usize,
                || self.the_stage_scripts(),
                |scripts| self.the_stage_scripts_time(scripts),
            )
    }

    /// The card that is up, by its own number, and nothing between cards.
    fn the_card_that_is_up(&self) -> Option<u32> {
        let card = Self::object(G_CARD)?;
        let flags = mem::read_committed::<u32>(card + CARD_FLAGS)?;
        if flags & CARD_IS_UP == 0 {
            return None;
        }
        u32::try_from(mem::read_committed::<i32>(card + CARD_NUMBER)?).ok()
    }

    /// Whether a conversation is on.
    fn a_conversation_is_on(&self) -> bool {
        Self::object(G_PANEL)
            .and_then(|panel| mem::read_committed::<usize>(panel + PANEL_CONVERSATION))
            .is_some_and(|it| it != 0)
    }

    /// **Whether the pause object is on the end of a run**, whichever way it ended — see
    /// [`PAUSE_THE_RUN_IS_OVER`].
    fn the_end_of_a_run_is_up(&self) -> bool {
        Self::object(G_PAUSE)
            .and_then(|pause| mem::read_committed::<i32>(pause + PAUSE_STATE))
            .is_some_and(|state| PAUSE_THE_RUN_IS_OVER.contains(&state))
    }

    /// Whether the run on is a replay being played, the attract demo's included.
    fn a_replay_is_played(&self) -> bool {
        Self::object(G_GAME)
            .and_then(|game| mem::read_committed::<i32>(game + GAME_REPLAYING))
            .is_some_and(|it| it == 1)
    }

    /// The name entry up: the pause object's, the replay's or the ranking's, or the title's after a clear.
    fn the_entry_a_name_is_typed_into(&self) -> Option<NameEntry> {
        let in_the_pause_object = Self::object(G_PAUSE).filter(|pause| {
            mem::read_committed::<i32>(pause + PAUSE_STATE)
                .is_some_and(|state| [PAUSE_NAMES_A_REPLAY, PAUSE_NAMES_A_RANKING].contains(&state))
        });
        if let Some(pause) = in_the_pause_object {
            return Some(NameEntry {
                name: pause + PAUSE_NAME,
                typed: pause + PAUSE_NAME_TYPED,
                grid: pause + PAUSE_NAME_GRID,
            });
        }
        let title = unsafe { self.title() }?;
        (mem::read_committed::<i32>(title + TITLE_SCREEN)? == SCREEN_NAME_ENTRY
            && mem::read_committed::<i32>(title + TITLE_STEP)? == TITLE_STEP_READING
            && mem::read_committed::<i32>(title + TITLE_NAME_NOT_ASKED)? == 0)
            .then_some(NameEntry {
                name: title + TITLE_NAME,
                typed: title + TITLE_NAME_TYPED,
                grid: title + TITLE_NAME_GRID,
            })
    }

    /// Bytes [`Game::captures`] took put back over every record's cards, or all but each entry's name.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn put_the_cards_back(&self, saved: &[u8], keeping_names: bool) {
        let Some(records) = Self::object(G_SCORE_OBJECT) else {
            return;
        };
        for (record, cards) in saved
            .chunks(CARDS * CARD_RECORD_BYTES)
            .take(RECORDS)
            .enumerate()
        {
            let at = card_record(records, record, 0);
            for (byte, value) in cards.iter().enumerate() {
                if keeping_names && byte % CARD_RECORD_BYTES < CARD_NAME_BYTES {
                    continue;
                }
                unsafe { mem::write::<u8>(at + byte, *value) };
            }
        }
    }
}

/// **Where a card's entry is in the score object's records**: record `record` — a shot type,
/// `character * 3 + shot`, or [`RECORD_OF_EVERY_SHOT_TYPE`] — at `+0x8` in steps of `0x437c`, and the card's
/// `0x90` bytes from `+0x59c` of it (0x42aeb1; 0x409350, 0x4093a0).
fn card_record(records: usize, record: usize, card: usize) -> usize {
    records + RECORDS_AT + record * SCORE_RECORD_BYTES + CARDS_AT + card * CARD_RECORD_BYTES
}

/// The four directions as bits of the word the read builds — see [`PAD_UP`] and the three beside it.
fn directions((up, down, left, right): (bool, bool, bool, bool)) -> u16 {
    [
        (up, PAD_UP),
        (down, PAD_DOWN),
        (left, PAD_LEFT),
        (right, PAD_RIGHT),
    ]
    .into_iter()
    .filter(|(pushed, _)| *pushed)
    .fold(0, |word, (_, bit)| word | bit)
}

impl Game for Th10 {
    /// **The seams this game's own frame is reached through, and no more**: the two walks, which put orb
    /// inside every frame, and the pad's half of the read, which is what a recipe presses through.
    ///
    /// **The frame itself is the game's still** — see `asm.md`, which has it in order. What is declined is
    /// each a reading not yet made rather than a decision: the window's making is reached through the
    /// import table, which orb rewrites for every game, and a function entered with the instance in `ebx`
    /// (0x439730, `mov ebx,esi; call` at 0x438be8) is not one a hook in Rust can stand in front of and call
    /// through with that register intact.
    fn hooks(&self) -> Hooks {
        Hooks {
            update: Some(Patch {
                target: RUN_CALC_CHAIN,
                prologue: RUN_CALC_CHAIN_PROLOGUE,
            }),
            draw: Some(Patch {
                target: RUN_DRAW_CHAIN,
                prologue: RUN_DRAW_CHAIN_PROLOGUE,
            }),
            walks: None,
            // **Pushed, all three** — see `FrameEntered::OnTheStack`.
            frame_entered: FrameEntered::OnTheStack,
            runs_its_frame: None,
            // **orb's own cadence**, one update and one draw a frame in place of the limiter and the frame
            // skip [`RENDER`] is made of — see `Th10::prepare_frame` and its two neighbours, which are the rest
            // of it in its own order.
            render: Some(Patch {
                target: RENDER,
                prologue: RENDER_PROLOGUE,
            }),
            save_replay: None,
            stop_recording: None,
            stage_begun: None,
            stage_building: Some(Building::StdcallOfTwo(Patch {
                target: REPLAY_BUILT,
                prologue: REPLAY_BUILT_PROLOGUE,
            })),
            unlocks_read: None,
            ranking_read: None,
            card_records_read: None,
            create_window: None,
            init_device: None,
            input: Some(Read::PadHalf(Patch {
                target: INPUT_PAD_HALF,
                prologue: INPUT_PAD_HALF_PROLOGUE,
            })),
            joystick: None,
            tops_up_the_music: None,
            stops_the_music: None,
            pad_buttons: None,
            // **Nothing, though the game has one**: `DialogBoxParamA` with the resource `0xcb` asks which of
            // a window and the whole screen to start in (0x438b8c), and it is put up only where bit `0x100`
            // of the configuration's flags is set (0x438b73) — which the dialog's own checkbox writes. What
            // orb answers dialogs through is the `DialogBoxIndirectParamW` th06c builds its own with, and the
            // copies of this game orb plays have the bit clear.
            startup_dialog: None,
            makes_its_window: None,
            moves_its_window: None,
            shows_its_window: None,
            places_its_window: None,
            restyles_its_window: None,
            reads_a_pad: None,
            makes_its_renderer: None,
            // `HeapCreate`, `HeapAlloc`, `HeapReAlloc`, `HeapFree`, `VirtualAlloc` and `VirtualFree` are all in
            // the exe's own `KERNEL32.dll` imports, the CRT linked statically.
            takes_its_memory: Memory::Kernel32Imports,
            // `CreateThread` of the same table, which its CRT's `_beginthreadex` (0x453683) ends in too.
            makes_its_threads: Threads::CreateThread,
            opens_its_files: FileOpen::CreateFileA,
        }
    }

    /// **A run of this game is rewound**, with what a chapter is made of read: the stage's first frame after a
    /// build on a loader thread (0x417870) — [`Th10::a_stage_has_just_been_built`] — and the replay's own seam
    /// for its seed ([`REPLAY_BUILT`]); the stage's own clock ([`SCRIPT_CLOCK`]); the two lists a snapshot
    /// leaves out, [`Th10::audio_state`] and [`Th10::live_handles`]; the numbers a stage starts from, field for
    /// field out of the replay's record ([`resume`]); and the pause menu's retry and the end of a run's menu,
    /// which the mode answers with its own.
    fn rewinds(&self) -> bool {
        true
    }

    /// `scoreth10.dat` (`0x46e848`), which the read `0x44b360` (0x42ae7d) and the write `0x44b620` (0x42b33f)
    /// both open through `CreateFileA` (0x44b432, 0x44b570).
    fn score_files(&self) -> crate::score::Files {
        crate::score::Files {
            theirs: c"scoreth10.dat",
            ours: c"pointdevice_scoreth10.dat",
        }
    }

    unsafe fn read_state(&self) -> State {
        let (scene, wanted) = unsafe { self.scenes() };
        let flags = unsafe { mem::read::<u32>(G_RUN_FLAGS) };
        let demo = flags & RUN_IS_THE_DEMO != 0;
        let replay = self.a_replay_is_played() && !demo;
        // **Not the end of a run, which this game keeps inside the scene a run is played in** — see
        // [`PAUSE_THE_RUN_IS_OVER`]: a game over and a clear put the pause object at 6 (0x4231f7) with the
        // scene still 7, so a scene alone reads the continue prompt as a run still being played, and a `play`
        // step waited out its whole bound at one.
        let playing = scene == SCENE_RUN && !self.the_end_of_a_run_is_up();
        let boss = self.the_boss_enemy();
        let player_state = Self::object(G_PLAYER)
            .and_then(|player| mem::read_committed::<i32>(player + PLAYER_STATE));
        let power = unsafe { mem::read::<u16>(G_POWER) };
        let stage_frames = unsafe { mem::read::<u32>(G_STAGE_FRAMES) };
        // **Counted on the frame the state becomes the hit's and on no other**, and for a run of somebody's
        // own: the hits of a replay are those of the run it was recorded from. Put back to nought at the
        // title, so that a launch's second run counts from where the game starts its own.
        let hit = player_state == Some(PLAYER_STATE_HIT);
        if playing && !replay && !demo {
            let mut kept = DEATHS_BY_FRAME
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let at = stage_frames as usize;
            if at == 0 {
                kept.clear();
            }
            // **A frame gone back to is the count that frame had**, which is how a chapter put back puts its
            // deaths back: nothing of orb's is in the snapshot, the count being orb's own.
            if let Some(then) = kept.get(at).copied() {
                DEATHS.store(then, Ordering::Relaxed);
                kept.truncate(at);
            }
            if hit && !WAS_HIT.swap(true, Ordering::Relaxed) {
                DEATHS.fetch_add(1, Ordering::Relaxed);
            } else if !hit {
                WAS_HIT.store(false, Ordering::Relaxed);
            }
            if at <= A_STAGE_RUNS_AT_MOST {
                kept.resize(at + 1, DEATHS.load(Ordering::Relaxed));
            }
        } else {
            WAS_HIT.store(false, Ordering::Relaxed);
            if scene == SCENE_TITLE {
                DEATHS.store(0, Ordering::Relaxed);
            }
        }
        State {
            scene,
            wanted,
            playing,
            in_run: playing || scene == SCENE_NEXT_STAGE,
            in_game: playing && !replay && !demo,
            in_ending: scene == SCENE_ENDING,
            ending_script: None,
            demo,
            replay,
            practice: flags & RUN_IS_A_PRACTICE != 0,
            paused: Self::object(G_GAME)
                .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
                .is_some_and(|it| it & GAME_PAUSED != 0),
            // **Anything but the player's ordinary state**: coming back in, dying, and the eight frames a hit
            // waits for a bomb in — see [`PLAYER_STATE`].
            unsettled: playing && player_state.is_some_and(|it| it != PLAYER_STATE_NORMAL),
            bombing: Self::object(G_BOMB)
                .and_then(|bomb| mem::read_committed::<i32>(bomb + BOMB_GOING_OFF))
                .is_some_and(|it| it != 0),
            in_dialogue: self.a_conversation_is_on(),
            // Counted from nought, the way orb counts them: this game's own count is from 1.
            stage: unsafe { mem::read::<i32>(G_STAGE) } - 1,
            difficulty: unsafe { mem::read(G_DIFFICULTY) },
            stage_frames,
            // **At the stage's frame 0 while it is built or the stage before it fades out**, which is what
            // [`Th10::stage_frame`] answers nothing for: a fade counts 1 to 30 with nothing of the stage on the list,
            // and the stage's own frame 0 is inside the update that begins it, where no read is.
            script_frames: self.script_frames(match playing {
                true => unsafe { self.stage_frame() }.unwrap_or(0),
                false => 0,
            }),
            random_seed: unsafe { mem::read(G_RANDOM) },
            // **orb's own count of the hits** — see [`DEATHS`].
            deaths: DEATHS.load(Ordering::Relaxed),
            lives: unsafe { mem::read::<i32>(G_LIVES) } as i8,
            // **The bombs a run has are its power, twenty of it apiece** — see [`POWER_A_BOMB_COSTS`].
            bombs: (power / POWER_A_BOMB_COSTS) as i8,
            power,
            // **Less the stage's own scripts**, which are on the list for the whole stage with no body to
            // shoot: counted in, a Normal run under `--tune` read two enemies or more on every frame from the
            // first to the ending, and the tuning pass proposed not one boundary.
            enemy_count: Self::object(G_ENEMY_MANAGER)
                .and_then(|manager| mem::read_committed::<i32>(manager + ENEMY_COUNT))
                .map_or(0, |count| {
                    count - i32::try_from(self.the_stage_scripts().len()).unwrap_or(0)
                }),
            bullet_count: Self::object(G_BULLET_MANAGER)
                .and_then(|manager| mem::read_committed::<i32>(manager + BULLETS_ALIVE))
                .unwrap_or(0),
            laser_count: Self::object(G_LASER_MANAGER)
                .and_then(|manager| mem::read_committed::<i32>(manager + LASERS_ALIVE))
                .unwrap_or(0),
            boss_present: boss.is_some(),
            boss_life: boss.and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_LIFE)),
            boss_attack_frames: boss
                .and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_ATTACK_FRAMES)),
            spellcard: self.the_card_that_is_up(),
        }
    }

    /// **A hit counted inside the eight frames a bomb takes it back in**, and taken once the player is dying:
    /// the count moves on the hit (see [`DEATHS`]), and the state says which way it went — back to playing
    /// through a bomb (0x425bd9 through 0x425c84), or on to dying (0x4269d0).
    unsafe fn where_a_counted_death_stands(&self) -> crate::game::CountedDeath {
        match Self::object(G_PLAYER)
            .and_then(|player| mem::read_committed::<i32>(player + PLAYER_STATE))
        {
            Some(PLAYER_STATE_HIT) => crate::game::CountedDeath::Open,
            Some(PLAYER_STATE_NORMAL) => crate::game::CountedDeath::Bombed,
            _ => crate::game::CountedDeath::Taken,
        }
    }

    unsafe fn a_line_of_dialogue_is_shown(&self) -> bool {
        self.a_conversation_is_on()
    }

    unsafe fn window(&self) -> Hwnd {
        unsafe { mem::read(G_GAME_WINDOW) }
    }

    /// **Nothing, which is a device orb draws nothing through**: this one is Direct3D 9's, where every
    /// drawing of orb's is Direct3D 8's or DXGI's — see [`Th10::draws_through_its_d3d_device`].
    unsafe fn d3d_device(&self) -> Device {
        Device::NULL
    }

    fn draws_through_its_d3d_device(&self) -> bool {
        false
    }

    unsafe fn direct3d_9_device(&self) -> Device {
        Self::object(G_D3D_DEVICE).map_or(Device::NULL, Device)
    }

    /// The last of the frame's quads, which is what the panel and the text over the field are drawn with,
    /// so an overlay drawn ahead of it would be under them.
    unsafe fn draw_what_is_queued(&self) {
        if let Some(sprites) = Self::object(G_SPRITES) {
            unsafe { with_esi(the_queues_draw(), sprites) };
        }
    }

    /// **The family the game's own text is asked for in**: `CreateFontA` at 0x437a9c with `ＭＳ ゴシック`
    /// (0x46f36c), a cell 32 tall, the weight 0x190 and Shift JIS — the one ask of the three in this exe that
    /// names its face itself (0x4369b2 and 0x437316 take theirs from the object they are made for). **The height
    /// is the size orb asks for, as a cell**, which is what that ask is: how big the game then draws what it
    /// bakes has not been read.
    fn own_face(&self) -> Option<Face> {
        Some(Face {
            family: "ＭＳ ゴシック",
            bold: false,
            height: |size| size,
            baked_at: 1,
        })
    }

    fn music(&self) -> Option<Music> {
        None
    }

    /// **The record of `thbgm.fmt` the stream is reading**, by where its track is in `thbgm.dat` and how long it
    /// is: a stage's own song and its boss's are two records, and the boss's is put on by the conversation before
    /// the fight (0x416a91 through 0x416a9c).
    fn music_identity(&self) -> Option<u32> {
        let record = Self::the_record_playing()?;
        let at = mem::read_committed::<u32>(record + RECORD_AT)?;
        let bytes = mem::read_committed::<u32>(record + RECORD_BYTES)?;
        Some(bytes ^ at.rotate_left(11))
    }

    /// **The name at the start of that record**, which is the file's own name for the track and the one a play
    /// looked the record up by — see [`RECORD_NAME`]: 0x43d2a0 cuts the path at `/` (0x43d2b1) or `\` (0x43d2c9)
    /// and compares what is left with each record's first bytes from `[0x492590+0x1f84]` (0x43d305 through
    /// 0x43d355).
    fn the_track_heard(&self) -> Option<String> {
        let record = Self::the_record_playing()?;
        let name = name_at(record + RECORD_NAME, RECORD_NAME_BYTES)?;
        Some(String::from_utf8_lossy(&name).into_owned())
    }

    /// **Whether that record is the one the stage loaded as its fight's song**, slot 1 of the manager's two — see
    /// [`SOUND_SONGS`] for the stage record's `+0x14` loaded there and the conversation before the stage's own
    /// boss playing it.
    ///
    /// By name rather than by the track having changed since the stage began, for th08's reason: the stage's song
    /// is queued at its start (0x4183e1) and put on the stream by the manager's own frame call (0x43e170), so the
    /// track a chapter taken at the stage's start reads can be the song before — and every midboss would then be
    /// a change. Nothing in the attract demo, whose build loads neither slot (0x417ba6), and nothing before a
    /// stage has loaded one.
    ///
    /// **And a play still in the manager's queue answers before the record does**: both songs are asked for as
    /// plays (0x420b10 queues kind 2, at the stage's start 0x4183e1 and before the stage's boss 0x416a9c), which
    /// the manager's own frame call walks a phase a run — and a playback runs no frame of the game's, so
    /// through one the record stays on whatever song was playing before. The last play queued is what the
    /// game has asked to hear, its argument the slot.
    fn the_fights_own_song_is_playing(&self) -> Option<bool> {
        let flags = mem::read_committed::<u32>(G_RUN_FLAGS)?;
        if flags & RUN_IS_THE_DEMO != 0 {
            return None;
        }
        let queued = (0..SOUND_COMMAND_SLOTS)
            .map(|slot| G_SOUND_MANAGER + SOUND_COMMANDS + slot * SOUND_COMMAND_BYTES)
            .map_while(|at| {
                mem::read_committed::<u32>(at)
                    .filter(|kind| *kind != 0)
                    .map(|kind| (kind, at))
            })
            .filter(|(kind, _)| *kind == SOUND_A_PLAY)
            .last();
        if let Some((_, at)) = queued {
            let slot = mem::read_committed::<u32>(at + SOUND_COMMAND_ARGUMENT)?;
            return Some(slot as usize == THE_FIGHTS_SONG);
        }
        let fights = name_at(
            G_SOUND_MANAGER + SOUND_SONGS + THE_FIGHTS_SONG * SONG_PATH_BYTES,
            SONG_PATH_BYTES,
        )
        .filter(|path| !path.is_empty())?;
        let record = Self::the_record_playing()?;
        let track = name_at(record + RECORD_NAME, RECORD_NAME_BYTES)?;
        Some(same_song(&fights, &track))
    }

    /// The music's own thread — see *The sound* in `asm.md`, whose manager keeps the id at `+0x614`.
    fn audio_thread(&self) -> Option<u32> {
        mem::read_committed::<u32>(G_SOUND_MANAGER + SOUND_MUSIC_THREAD).filter(|it| *it != 0)
    }

    unsafe fn stop_music(&self) {}

    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    /// **The objects the sound is, which a restore leaves alone**: the manager, the wrapper it keeps DirectSound
    /// in, the stream and the block its buffer is held in, and the source the stream reads through — see
    /// [`SOUND_MANAGER_BYTES`] and the four beside it. The music's thread writes the stream and the source as it
    /// refills (0x44d850), so a snapshot's copy of either is one the thread has gone past.
    fn audio_state(&self) -> Vec<Range<usize>> {
        let mut ranges: Vec<Range<usize>> =
            std::iter::once(G_SOUND_MANAGER..G_SOUND_MANAGER + SOUND_MANAGER_BYTES).collect();
        let held = |at: usize, bytes: usize| Self::object(at).map(|object| object..object + bytes);
        ranges.extend(held(G_SOUND_MANAGER + SOUND_DEVICE, SOUND_DEVICE_BYTES));
        if let Some(stream) = Self::object(G_SOUND_MANAGER + MUSIC_STREAM) {
            ranges.push(stream..stream + MUSIC_STREAM_BYTES);
            ranges.extend(held(stream + STREAM_BUFFER, STREAM_BUFFER_BYTES));
            ranges.extend(held(stream + STREAM_SOURCE, MUSIC_SOURCE_BYTES));
        }
        ranges
    }

    /// **What names an object a restore cannot copy**: every `.anm` file's textures and the sprites that copy
    /// their pointers, the surfaces the screen's captures are made in, and the supervisor's seven locks — see
    /// [`ANM_FILES_AT`] and the constants beside it.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        let mut ranges: Vec<Range<usize>> =
            std::iter::once(G_LOCKS..G_LOCKS + LOCKS_BYTES).collect();
        let Some(sprites) = Self::object(G_SPRITES) else {
            return ranges;
        };
        ranges.push(sprites + SURFACES_AT..sprites + SURFACES_END);
        for slot in 0..ANM_FILES {
            let Some(file) = Self::object(sprites + ANM_FILES_AT + slot * 4) else {
                continue;
            };
            // **Bounded rather than believed**, a count a restore is halfway through putting back being a range
            // that would hold megabytes of somebody else's.
            let mut list = |count: usize, at: usize, bytes: usize, most: u32| {
                let count = mem::read_committed::<u32>(file + count).map(|it| it.clamp(1, most));
                if let (Some(count), Some(at)) = (count, Self::object(file + at)) {
                    ranges.push(at..at + count as usize * bytes);
                }
            };
            list(
                ANM_TEXTURES,
                ANM_TEXTURES_AT,
                ANM_TEXTURE_BYTES,
                ANM_TEXTURES_AT_MOST,
            );
            list(
                ANM_SPRITES,
                ANM_SPRITES_AT,
                ANM_SPRITE_BYTES,
                ANM_SPRITES_AT_MOST,
            );
        }
        ranges
    }

    /// **What a restore waits for a song for is a song it could put back**, and this game gives it none — see
    /// [`Th10::music`] — so a stage's first chapter is taken on its first frame.
    fn a_snapshot_waits_for_its_music(&self) -> bool {
        false
    }

    /// The head of the sound's queue of commands, which the frame's sound carries out a phase at a time
    /// (0x43de0d through 0x43de18).
    fn a_sound_command_is_in_flight(&self) -> bool {
        mem::read_committed::<u32>(G_SOUND_MANAGER + SOUND_COMMANDS).is_some_and(|it| it != 0)
    }

    fn play_area(&self) -> Rect {
        PLAY_AREA
    }

    unsafe fn panel_tile(&self) -> Option<PanelTile> {
        None
    }

    unsafe fn repaint_lives_row(&self) {}

    unsafe fn draws_lives_row(&self) -> bool {
        false
    }

    /// **Nothing read**, the panel's own layout not being — and nothing of orb's draws over it here.
    fn lives_row(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn content_size(&self) -> (u32, u32) {
        BACK_BUFFER
    }

    fn windowed(&self) -> bool {
        unsafe { mem::read::<u8>(CFG_WINDOWED) != 0 }
    }

    unsafe fn force_windowed(&self) {
        unsafe { mem::write::<u8>(CFG_WINDOWED, 1) };
    }

    unsafe fn set_play_viewport(&self, _device: Device) {}

    /// **The chain the two walks are called on**, which this game allocates rather than keeping it as a
    /// static — see *The chain* in `asm.md`.
    fn chain(&self) -> *mut c_void {
        Self::object(G_CHAIN).unwrap_or(0) as *mut c_void
    }

    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    /// **Nothing done, and not because there is no device**: on the settings th10 comes with it reads its keys
    /// through a DirectInput keyboard held `DISCL_FOREGROUND` (0x43b8d0), which the system takes away while the
    /// window is behind — but the read acquires it again on *any* failed `GetDeviceState` (0x44a766, then 0x44a77a
    /// or 0x44a78e), not only the one report of the loss. And orb's hook is the pad's half of that read (0x44a190),
    /// called after the keyboard's: an `Acquire` from here would come after the game's own, on the frame already
    /// lost to it.
    unsafe fn acquire_input(&self) -> bool {
        true
    }

    /// **The input flags' bit that says the keyboard is DirectInput's let go of, and the device unacquired**: the
    /// read takes `GetKeyboardState` (0x44a636) wherever bit 0x200 of `[0x491ff4]` is clear (0x44a628), and every
    /// read of the device at `[0x491c38]` is behind that same test (0x44a754, 0x44ab31, 0x44ae85) — so the device
    /// is left in place rather than released, nothing of the game's testing its pointer.
    unsafe fn take_sent_keys(&self) -> bool {
        let flags = unsafe { mem::read::<u32>(G_INPUT_FLAGS) };
        if flags & INPUT_THROUGH_DIRECTINPUT == 0 {
            return false;
        }
        let device = unsafe { mem::read::<usize>(G_KEYBOARD_DEVICE) };
        if device != 0 {
            let vtable = unsafe { mem::read::<usize>(device) };
            let unacquire: unsafe extern "system" fn(usize) -> i32 = unsafe {
                std::mem::transmute(mem::read::<usize>(
                    vtable + DEVICE_UNACQUIRE * size_of::<usize>(),
                ))
            };
            unsafe { unacquire(device) };
        }
        unsafe { mem::write::<u32>(G_INPUT_FLAGS, flags & !INPUT_THROUGH_DIRECTINPUT) };
        true
    }

    /// **What the game's frame does between its limiter and its calc walk**, in its order: the queue drawn
    /// out, the camera set and its viewport, and the word after it — see *The frame* in `asm.md`.
    ///
    /// `device` is nought, [`Th10::d3d_device`] answering none — the device every call here is made on is
    /// the game's own, read where the game keeps it.
    unsafe fn prepare_frame(&self, _device: Device) {
        let Some(device) = Self::object(G_D3D_DEVICE) else {
            return;
        };
        if let Some(sprites) = Self::object(G_SPRITES) {
            unsafe { with_esi(the_queues_draw(), sprites) };
        }
        unsafe {
            mem::write::<usize>(G_THE_CAMERA, THE_CAMERA);
            with_edi(the_cameras_set(), THE_CAMERA);
            orb_api::d3d9::set_viewport(
                Device(device),
                mem::read::<orb_api::Viewport>(THE_CAMERA + CAMERA_VIEWPORT),
            );
            mem::write::<i32>(G_A_CAMERA_IS_SET, 1);
        }
    }

    /// **The scene begun, and what the frame does inside it before its draw chain**: the queue put back at
    /// its start, the fog's dword forced and the fog set.
    unsafe fn begin_drawing(&self, _device: Device) {
        let Some(device) = Self::object(G_D3D_DEVICE) else {
            return;
        };
        orb_api::d3d9::begin_scene(Device(device));
        if let Some(sprites) = Self::object(G_SPRITES) {
            unsafe {
                mem::write::<u32>(sprites + SPRITES_QUEUED, 0);
                mem::write::<usize>(sprites + SPRITES_WRITE_AT, sprites + SPRITES_QUEUE_STARTS);
                mem::write::<usize>(sprites + SPRITES_DRAWN_FROM, sprites + SPRITES_QUEUE_STARTS);
            }
        }
        unsafe {
            mem::write::<u32>(G_FOG_FORCED, 0xff);
            with_edi(the_fogs_set(), G_SUPERVISOR);
        }
    }

    /// **And after the draw chain**: the queue drawn out, the texture unbound, and the scene ended.
    unsafe fn end_drawing(&self, _device: Device) {
        let Some(device) = Self::object(G_D3D_DEVICE) else {
            return;
        };
        if let Some(sprites) = Self::object(G_SPRITES) {
            unsafe { with_esi(the_queues_draw(), sprites) };
        }
        orb_api::d3d9::set_texture(Device(device), 0, None);
        orb_api::d3d9::end_scene(Device(device));
    }

    fn frame_calls(&self) -> FrameCalls {
        FrameCalls {
            play_sounds: Call {
                function: plays_the_sounds as *const () as usize,
                this: G_SOUND_MANAGER,
            },
            present: Call {
                function: PRESENT,
                this: 0,
            },
            orbs_own: false,
        }
    }

    unsafe fn jump_to_stage(&self, _stage: i32) -> bool {
        false
    }

    /// **The player's own countdown put back up**, the one both of the player's tests ask is nought before they
    /// take the hit (0x426756, 0x426950) — see [`PLAYER_UNHITTABLE_FOR`]. The lasers' test refuses outright while
    /// it is above nought; the bullets' still answers 1, so a bullet that touches the player is cancelled all
    /// the same (0x426756 through 0x426766, 0x4064ad).
    unsafe fn make_invulnerable(&self) {
        let Some(player) = Self::object(G_PLAYER) else {
            return;
        };
        let at = player + PLAYER_UNHITTABLE_FOR;
        if unsafe { mem::read::<i32>(at) } < 2 {
            unsafe {
                mem::write::<i32>(at, 2);
                mem::write::<f32>(at + size_of::<i32>(), 2.0);
            }
        }
    }

    unsafe fn replaying(&self) -> bool {
        self.a_replay_is_played()
    }

    unsafe fn show_ranking(&self) {}

    unsafe fn showing_ranking(&self) -> bool {
        false
    }

    unsafe fn ranking_scene(&self) -> bool {
        false
    }

    unsafe fn restore_menu_cursor(&self) {}

    unsafe fn ranking_state(&self) -> String {
        "the ranking of th10 is not read".to_owned()
    }

    unsafe fn leave_ranking(&self) {}

    /// **One more attempt at the card that is up, where the game's own card start counts one**: in the run's
    /// shot type's record and in the total's, each held at [`A_CARDS_COUNT_STOPS_AT`], and not in a replay
    /// (0x409312 through 0x4093b8) — told the way the card start tells it, by the replay manager's own word.
    unsafe fn count_card_attempt(&self) -> Option<u16> {
        let card = self.the_card_that_is_up()? as usize;
        let played_back = Self::object(G_REPLAY_MANAGER)
            .and_then(|manager| mem::read_committed::<i32>(manager + REPLAY_MANAGER_PLAYS_BACK))
            == Some(1);
        if played_back || card >= CARDS {
            return None;
        }
        let records = Self::object(G_SCORE_OBJECT)?;
        let shot_type = unsafe {
            mem::read::<i32>(G_CHARACTER) * SHOT_TYPES_A_CHARACTER + mem::read::<i32>(G_SHOT_TYPE)
        };
        let shot_type = usize::try_from(shot_type)
            .ok()
            .filter(|it| *it < RECORD_OF_EVERY_SHOT_TYPE)?;
        let mut counted = None;
        for record in [shot_type, RECORD_OF_EVERY_SHOT_TYPE] {
            let at = card_record(records, record, card) + CARD_ATTEMPTS;
            let count = mem::read_committed::<u32>(at)?;
            let count = (count + 1).min(A_CARDS_COUNT_STOPS_AT);
            unsafe { mem::write::<u32>(at, count) };
            counted.get_or_insert(count);
        }
        counted.and_then(|it| u16::try_from(it).ok())
    }

    unsafe fn redraw_card_name(&self) {}

    /// **Every record's cards**, the seven records' `0x90`-byte entries for all 110 cards — see
    /// [`card_record`] — which is what a card's attempts and captures are counted into.
    unsafe fn captures(&self) -> Vec<u8> {
        let Some(records) = Self::object(G_SCORE_OBJECT) else {
            return Vec::new();
        };
        (0..RECORDS)
            .flat_map(|record| {
                let at = card_record(records, record, 0);
                (0..CARDS * CARD_RECORD_BYTES)
                    .map(move |byte| mem::read_committed::<u8>(at + byte).unwrap_or(0))
            })
            .collect()
    }

    /// **Whether any of them holds a count**, the captures and the attempts at `+0x80` and `+0x84`: every entry
    /// holds a name as soon as the card has been met.
    fn these_records_hold_a_count(&self, records: &[u8]) -> bool {
        records.chunks(CARD_RECORD_BYTES).any(|card| {
            [CARD_CAPTURES, CARD_ATTEMPTS].into_iter().any(|at| {
                card.get(at..at + size_of::<u32>())
                    .is_some_and(|word| word.iter().any(|byte| *byte != 0))
            })
        })
    }

    unsafe fn set_captures(&self, saved: &[u8]) {
        unsafe { self.put_the_cards_back(saved, false) };
    }

    /// **The counts put back and every name left as it stands**, the name being the first `0x80` bytes of an
    /// entry and copied in from the script at each card's start (0x409350).
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]) {
        unsafe { self.put_the_cards_back(saved, true) };
    }

    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// **The mode's own file read into the score object, but for what the front end offers**: each shot type's
    /// record out of the file the mode points at — or made empty by the game's own code where that file holds
    /// none — and out of the game's own file the settings with the clears the Extra is lit from, and each
    /// practice stage's being offered. A stage reached is a stage reached whichever mode reached it, where a
    /// ranking, a card's count and a practice score are the mode's. [`score_file`] has why it is read here.
    ///
    /// Where the game's own file has no record the object's own stands, being what startup read out of it.
    unsafe fn read_what_the_modes_file_records(&self) {
        let Some(object) = mem::read_committed::<usize>(G_SCORE_OBJECT).filter(|at| *at != 0)
        else {
            return;
        };
        let read = |name: &str| {
            orb_api::fs::read(std::path::Path::new(name))
                .ok()
                .and_then(|file| score_file::decode(&file))
        };
        let own_name = self.score_files().theirs.to_str().unwrap_or_default();
        let own = read(own_name);
        let modes_name = crate::score::the_file_now_opened();
        let modes = read(modes_name);
        let emptied: extern "stdcall" fn(usize) =
            unsafe { std::mem::transmute(a_record_emptied()) };
        let mut held = 0;
        for number in 0..RECORDS {
            let record = object + RECORDS_AT + number * SCORE_RECORD_BYTES;
            let offered_in = |bytes: &[u8]| -> Vec<u8> {
                (0..PRACTICES)
                    .map(|practice| {
                        bytes[PRACTICE_AT + practice * PRACTICE_BYTES + PRACTICE_OFFERED]
                    })
                    .collect()
            };
            // **Each one offered where the file offers it or memory does**: the game marks a stage reached in
            // memory, and writes it only as a run is torn down and as it leaves.
            let in_memory = offered_in(&unsafe { mem::read_bytes(record, SCORE_RECORD_BYTES) });
            let offered: Vec<u8> = match own
                .as_ref()
                .and_then(|own| own.shot_types[number].as_deref())
            {
                Some(bytes) => offered_in(bytes)
                    .into_iter()
                    .zip(&in_memory)
                    .map(|(file, memory)| file.max(*memory))
                    .collect(),
                None => in_memory,
            };
            match modes
                .as_ref()
                .and_then(|modes| modes.shot_types[number].as_deref())
            {
                Some(bytes) => {
                    unsafe { mem::write_bytes(record, bytes) };
                    held += 1;
                }
                None => emptied(record),
            }
            for (practice, offered) in offered.into_iter().enumerate() {
                unsafe {
                    mem::write::<u8>(
                        record + PRACTICE_AT + practice * PRACTICE_BYTES + PRACTICE_OFFERED,
                        offered,
                    )
                };
            }
        }
        // **And the clears the file holds with the ones memory does**, for the same reason: the clear is marked
        // after the teardown on the way to the ending (0x418150 at 0x421afd, then 0x40b684), so a mode question
        // after it in the same launch reads a file without it.
        if let Some(settings) = own.as_ref().and_then(|own| own.settings.as_deref()) {
            let cleared = unsafe { mem::read_bytes(object + CLEARS.start, CLEARS.len()) };
            unsafe { mem::write_bytes(object + SETTINGS_AT, settings) };
            for (at, memory) in CLEARS.zip(cleared) {
                let file = mem::read_committed::<u8>(object + at).unwrap_or(0);
                unsafe { mem::write::<u8>(object + at, file.max(memory)) };
            }
        }
        log!(
            "score: {held} of {RECORDS} shot types' records read out of {modes_name}, the practices offered and \
             the clears out of {own_name}"
        );
    }

    /// **Yes**: `scoreth10.dat` is read once, by the loading object as the game starts (0x42ae7d, the one call
    /// of 0x42ae60), into the object every screen reads from memory — the main menu takes its Extra off it
    /// (0x42c850 at 0x42d442). Read through the mode's fork, a launch in Pointdevice mode came up with the
    /// Extra gone from the menu, orb's own file holding no clear.
    fn reads_its_file_once_at_startup(&self) -> bool {
        true
    }

    unsafe fn menu_pointed_at(&self) -> Option<Menu> {
        let title = unsafe { self.title() }?;
        if mem::read_committed::<i32>(title + TITLE_SCREEN)? != SCREEN_MAIN_MENU {
            return None;
        }
        match mem::read_committed::<i32>(title + TITLE_CURSOR)? {
            TITLE_ITEM_START | TITLE_ITEM_EXTRA | TITLE_ITEM_PRACTICE => Some(Menu::Run),
            TITLE_ITEM_REPLAY => Some(Menu::Replays),
            TITLE_ITEM_PLAYER_DATA => Some(Menu::Scores),
            _ => None,
        }
    }

    unsafe fn menu_takes_a_press(&self) -> bool {
        let Some(title) = (unsafe { self.title() }) else {
            return false;
        };
        mem::read_committed::<i32>(title + TITLE_SCREEN) == Some(SCREEN_MAIN_MENU)
            && mem::read_committed::<i32>(title + TITLE_STEP) == Some(TITLE_STEP_READING)
    }

    /// **Both pads there are, as the read would have made them, with the hat as well**: the one DirectInput
    /// found, read here because the game's read is frozen on the frames a menu of orb's is up, and the one orb
    /// sampled — its own pad where the game found none, and the same pad again where it did, which merges to
    /// the same answer. Decide and cancel are what every menu of the game takes them as, the shot and the bomb
    /// or the pause (`0x1001` and `0xa` at 0x42d5ff and 0x42d59c, the pad's half of them being `0x1` and `0xa`).
    unsafe fn pad(&self, pad: Option<Reading>) -> Pad {
        let own = unsafe { self.the_pads_own_state() }.map_or(0, |state| unsafe {
            self.the_word_of_the_pads_own(&state, true)
        });
        let sampled = pad.map_or(0, |reading| unsafe {
            self.the_word_of_a_sampled_pad(&reading, true)
        });
        let word = own | sampled;
        Pad {
            up: word & PAD_UP != 0,
            down: word & PAD_DOWN != 0,
            decide: word & MENU_DECIDE != 0,
            cancel: word & MENU_CANCEL != 0,
            left: word & PAD_LEFT != 0,
            right: word & PAD_RIGHT != 0,
            shot: word & BUTTON_SHOT != 0,
            bomb: word & BUTTON_BOMB != 0,
            focus: word & BUTTON_FOCUS != 0,
            skip: word & BUTTON_SKIP != 0,
        }
    }

    /// **The hat and nothing else, where `hats`**: the read that makes the rest of a pad's word is the game's
    /// own and called through — `Read::PadHalf` — and the one field it never looks at is the hat a d-pad reports
    /// in. Off the pad orb sampled and not off the device the game holds, which the game's read has just polled
    /// this frame: a second `Poll` and `GetDeviceState` a frame is the cost the contract warns of.
    ///
    /// **Not the sampled pad's buttons and stick as well**, which would play a pad the game has no device for:
    /// where it is the game's own pad, its stick would then count a push from a quarter of its travel through
    /// winmm where the game's DirectInput read counts one from six tenths, and the feel of the stick would be
    /// orb's.
    unsafe fn pad_word(&self, pad: Option<Reading>, hats: bool) -> u16 {
        match (pad, hats) {
            (Some(reading), true) => directions(crate::joystick::hat(reading.pov)),
            _ => 0,
        }
    }

    unsafe fn add_pad_buttons(&self, _into: *mut c_void, _pad: Option<Reading>) {}

    /// **The mapping read backwards**: each of the five bits the read makes of a pad's buttons held on the
    /// button the mapping names for it, and the four directions as the stick pushed to the end of its travel,
    /// past the dead zone of either branch of the read. The menus' Enter (`0x1000`) has no button — every menu
    /// takes the shot's `0x1` beside it — and nor have the pause menu's Q and R or the screenshot.
    unsafe fn pushed_for(&self, word: u16) -> Option<crate::joystick::Pushed> {
        let mapping = unsafe { mem::read::<[i16; 9]>(G_PAD_MAPPING) };
        Some(crate::joystick::pushed_by(
            word,
            PAD_MAPPED.iter().map(|(at, bit)| (*bit, mapping[*at])),
            (PAD_UP, PAD_DOWN, PAD_LEFT, PAD_RIGHT),
        ))
    }

    /// **The pad DirectInput found**, where the input's flags say there is one — its axes ranged ±1000 (0x43bb30).
    unsafe fn its_pad_device(&self) -> Option<(usize, i32)> {
        if mem::read_committed::<u32>(G_INPUT_FLAGS)? & PAD_THROUGH_DIRECTINPUT == 0 {
            return None;
        }
        mem::read_committed::<usize>(G_PAD_DEVICE)
            .filter(|device| *device != 0)
            .map(|device| (device, PAD_RANGE))
    }

    /// **What the pause menu's own item for the title writes**: the scene wanted put at the title
    /// (0x40ac90, called with 4 at 0x42319a), which the supervisor answers by taking the game down and
    /// building the title (0x42197b).
    unsafe fn leave_run(&self) -> bool {
        unsafe { mem::write::<i32>(G_SCENE_WANTED, SCENE_TITLE) };
        true
    }

    /// **The pause menu's item for starting over, taken**: the scene wanted at [`SCENE_START_OVER`] in a run
    /// still being played — the end of a run writes the same scene for a cleared run's retry (0x424360 through
    /// 0x424370), which is no ask of this kind.
    unsafe fn asked_to_start_the_run_over(&self) -> bool {
        let (scene, wanted) = unsafe { self.scenes() };
        scene == SCENE_RUN && wanted == SCENE_START_OVER && !self.the_end_of_a_run_is_up()
    }

    /// **The scene the game is in wanted again, and the pause menu's own resume made**: the item for starting
    /// over closes the menu and leaves the run paused — the game object's flag `0x10` set and the game's speed
    /// held at the pause's — for the build it asked for (0x42315e through 0x42316c), and only the item for
    /// resuming takes both back (`0x422c30`, entered with the pause object in `esi`, 0x4231ab).
    unsafe fn keep_the_run_where_it_is(&self) {
        let (scene, wanted) = unsafe { self.scenes() };
        unsafe { mem::write::<i32>(G_SCENE_WANTED, scene) };
        let paused = Self::object(G_GAME)
            .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
            .is_some_and(|flags| flags & GAME_PAUSED != 0);
        if wanted == SCENE_START_OVER
            && paused
            && let Some(pause) = Self::object(G_PAUSE)
        {
            unsafe { with_esi(the_unpause(), pause) };
        }
    }

    unsafe fn swallow_input(&self) {
        unsafe {
            mem::write::<u16>(G_INPUT, u16::MAX);
            mem::write::<u16>(G_INPUT_LAST, u16::MAX);
        }
    }

    unsafe fn press_into(&self, _state: *mut u8, _word: u16) {}

    fn menu_decide(&self) -> u16 {
        MENU_DECIDE
    }

    /// **Yes**: the ending's script waits on its lines with a wait of `-1`, which puts its timer back at 999 on
    /// every update (0x40befa through 0x40bf2b) and goes on only on a press of `0x1001` (0x40bf30). A Pointdevice
    /// Normal cleared on the real game stood in its ending for good under the skip — `ending skipped, 54000 frames
    /// run, scene 14 -> 14` on every frame, the ending script's timer (`[[0x477700]+0x18]+0x30`) at 999 — where
    /// the same ending with presses in it went on to scene 15 (2026-10-02).
    fn an_ending_waits_for_a_press(&self) -> bool {
        true
    }

    fn menu_cancel(&self) -> u16 {
        MENU_CANCEL
    }

    /// **What the name entry holds**, the replay's (state 11) or the ranking's (12), which share the field: the
    /// eight characters at `+0x2b4`, space-padded (0x423b28 through 0x423bf4).
    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        let entry = self.the_entry_a_name_is_typed_into()?;
        crate::game::the_name_held_at(entry.name, PAUSE_NAME_BYTES)
    }

    /// **Into the field the entry's own grid writes**, with the place in it at the full eight and the grid's
    /// cursor on the cell that ends the entry (90, 0x423dcd through 0x423ec8) — so the press after it leaves the
    /// entry the way somebody's would.
    unsafe fn types_its_name(&self, name: &str) -> bool {
        let Some(entry) = self.the_entry_a_name_is_typed_into() else {
            return false;
        };
        if !crate::game::the_name_typed_into(entry.name, PAUSE_NAME_BYTES, name) {
            return false;
        }
        unsafe {
            mem::write::<i32>(entry.typed, PAUSE_NAME_BYTES as i32);
            mem::write::<i32>(entry.grid, PAUSE_THE_END_CELL);
        }
        true
    }

    unsafe fn typing_a_name(&self) -> bool {
        self.the_entry_a_name_is_typed_into().is_some()
    }

    /// **The end of a run's menu answered with its last item**, the title, the way its own decide answers it: the
    /// pause object at 13 with its timer started over (0x423948 through 0x42394f), which twelve frames on sends
    /// items 1 and 2 to the title (0x42431d through 0x424370). The menu is state 8 and its item 1 is the replay's
    /// save (0x423991), so the item taken is the one that saves nothing and plays nothing on.
    ///
    /// **And after a clear, the title's screen 16** (0x433570), the replay's 25 slots, answered with its cancel's
    /// own step 4 (0x4336af) set the way the step setter 0x42c620 sets one — which six frames on puts the main
    /// menu up (0x433a33 through 0x433a7e).
    unsafe fn skip_replay_prompt(&self) -> bool {
        if let Some(title) = unsafe { self.title() }
            && mem::read_committed::<i32>(title + TITLE_SCREEN) == Some(SCREEN_REPLAYS_SLOTS)
            && mem::read_committed::<i32>(title + TITLE_STEP) == Some(TITLE_STEP_READING)
        {
            unsafe {
                mem::write::<i32>(title + TITLE_STEP, SLOTS_STEP_CANCELLED);
                mem::write::<i32>(title + TITLE_STEPS_FRAMES - 4, -1);
                mem::write::<i32>(title + TITLE_STEPS_FRAMES, 0);
                mem::write::<i32>(title + TITLE_STEPS_FRAMES + 4, 0);
            }
            log!("replay: the offer to save one was taken off the screen a run ended on");
            return true;
        }
        let Some(pause) = Self::object(G_PAUSE) else {
            return false;
        };
        if mem::read_committed::<i32>(pause + PAUSE_STATE) != Some(PAUSE_THE_RUN_ENDS_ON_A_MENU) {
            return false;
        }
        unsafe {
            mem::write::<i32>(pause + PAUSE_CURSOR, PAUSE_ITEM_THE_TITLE);
            mem::write::<i32>(pause + PAUSE_STATE, PAUSE_LEAVING_THE_END);
            mem::write::<i32>(pause + PAUSE_FRAMES - 4, -1);
            mem::write::<i32>(pause + PAUSE_FRAMES, 0);
            mem::write::<f32>(pause + PAUSE_FRAMES + 4, 0.0);
        }
        log!("replay: the offer to save one was taken off the screen a run ended on");
        true
    }

    unsafe fn leave_the_screen_a_run_ends_at(&self) -> bool {
        unsafe { mem::write::<i32>(G_SCENE_WANTED, SCENE_TITLE) };
        true
    }

    unsafe fn the_screen_a_run_ends_at_is_up(&self) -> bool {
        unsafe { self.run_finished() }
    }

    /// **Nothing while the stage is built or the stage before it fades out**, whose frames are none of this
    /// stage's: the build's are all read at nought (0x4178bb), and the fade's count 1 to 30 and are counted again
    /// from nought as the stage begins (0x41846b).
    unsafe fn stage_frame(&self) -> Option<u32> {
        let (scene, _) = unsafe { self.scenes() };
        let flags = Self::object(G_GAME)
            .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
            .unwrap_or(GAME_BEING_BUILT);
        (scene == SCENE_RUN && flags & (GAME_BEING_BUILT | GAME_FADING_IN) == 0)
            .then(|| unsafe { mem::read::<u32>(G_STAGE_FRAMES) })
    }

    /// **The frame whose update begins the stage**: the build over and the game object at its first frame with
    /// nothing kept to fade out, or at the thirtieth of a fade — the two frames the game object's job begins a
    /// stage on (0x41821c, 0x41840b), putting the stage's frames at nought, spawning `main` and taking the
    /// player's part of the replay's record. Asked at the top of the frame, before that update, which is where
    /// the playback puts a stage's numbers back as well.
    unsafe fn a_stage_has_just_been_built(&self) -> bool {
        let (scene, _) = unsafe { self.scenes() };
        let Some(game) = Self::object(G_GAME) else {
            return false;
        };
        let flags = mem::read_committed::<u32>(game + GAME_FLAGS).unwrap_or(GAME_BEING_BUILT);
        let frames = mem::read_committed::<i32>(game + GAME_FRAMES).unwrap_or(-1);
        let nothing_kept = Self::object(G_KEPT_STD).is_none();
        // **Once for each stage**, a frame orb holds before its update — the mode question's, the retry menu's —
        // being the same frame asked about again.
        let mut begun = BEGUN
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if flags & GAME_BEING_BUILT != 0 {
            *begun = None;
        }
        A_BUILD_WAS_GOING_ON.store(flags & GAME_BEING_BUILT != 0, Ordering::Relaxed);
        let begins = scene == SCENE_RUN
            && flags & GAME_BEING_BUILT == 0
            && match flags & GAME_FADING_IN {
                0 => frames == 0 && nothing_kept,
                _ => frames == GAME_BEGINS_A_FADE_AT,
            };
        if !begins || *begun == Some((game, frames)) {
            return false;
        }
        *begun = Some((game, frames));
        true
    }

    /// **Nothing walked on the frame a build ended between orb's look and the walk**, so that the stage's first
    /// update is on a frame whose look read the build over. The loader is a thread of its own (0x417870) and takes
    /// its flag off at 0x417c30 whatever the main thread is doing; a walk that finds it off runs the game object's
    /// frame 0 and the stage's first update (0x41821c) — on a frame [`Th10::a_stage_has_just_been_built`] read as
    /// still building, so with nothing put back. A pick-up of 2026-10-04 02:27 was that frame: no playback, the
    /// stage's own frame 1 at `updates=1` and stage 7 played from its start.
    ///
    /// **One walk left out, and not th08's supervisor walked on every frame of the build**: the game object's own
    /// job steps nothing while its flag is on (0x41864b), so the walks of the build's frames run no stage; and the
    /// build being over by this one, nothing of it waits on the main thread. **Nor every walk of the build left
    /// out**, which would close the window inside the walk as well — the supervisor's, the title's and the pause
    /// object's jobs ahead of the game object's — because the build's last wait (0x417c02 through 0x417c10) sleeps
    /// until `[0x494518]`, beside the music's `thbgm.fmt` at `[0x494514]`, goes back to nought, and nothing in the
    /// exe writes that word by its address: who clears it, and whether a walk has to run for it, is unread.
    ///
    /// **No e2e test holds this**, the fake being one thread: its build ends at the present or inside a walk, and
    /// orb's look and the walk are one call of the chain's hook apart with no call of the fake's between them for a
    /// build to end in. What says it on the real game is the line below in a launch's log.
    ///
    /// # Safety
    /// Must run on the game's main thread, after [`Th10::a_stage_has_just_been_built`] this frame.
    unsafe fn walks_a_frame_of_its_build(&self) -> Option<i32> {
        if !A_BUILD_WAS_GOING_ON.load(Ordering::Relaxed) {
            return None;
        }
        let game = Self::object(G_GAME)?;
        let flags = mem::read_committed::<u32>(game + GAME_FLAGS)?;
        if flags & GAME_BEING_BUILT != 0 {
            return None;
        }
        log!("frame: the stage's build ended after this frame was looked at; its walk is left out");
        Some(crate::runtime::CHAIN_BREAK)
    }

    unsafe fn generator(&self) -> u16 {
        unsafe { mem::read(G_RANDOM) }
    }

    fn run_input(&self) -> u16 {
        RUN_INPUT
    }

    unsafe fn run_start(&self) -> RunStart {
        RunStart {
            difficulty: unsafe { mem::read(G_DIFFICULTY) },
            character: unsafe { mem::read(G_CHARACTER) },
            shot_type: unsafe { mem::read(G_SHOT_TYPE) },
            practice: unsafe { mem::read::<u32>(G_RUN_FLAGS) } & RUN_IS_A_PRACTICE != 0,
            stage: unsafe { mem::read::<i32>(G_STAGE) } - 1,
            mode: 0,
        }
    }

    /// **The run the shot select's cursor is on**: the difficulty and the character already where the two
    /// screens before it wrote them (0x4305cb, 0x430997), and the shot under the cursor, which the select writes
    /// only as it is decided (0x430ec0) — the stage 1, or the Extra's for its difficulty (0x430f16 through
    /// 0x430f4f), and a practice by the main menu's own bit (0x42d7d4).
    unsafe fn run_pointed_at(&self) -> Option<RunStart> {
        let title = Self::object(G_TITLE)?;
        if mem::read_committed::<i32>(title + TITLE_SCREEN)? != SCREEN_SHOT_SELECT {
            return None;
        }
        let difficulty = unsafe { mem::read::<i32>(G_DIFFICULTY) };
        Some(RunStart {
            difficulty,
            character: unsafe { mem::read(G_CHARACTER) },
            shot_type: mem::read_committed::<i32>(title + TITLE_CURSOR)?,
            practice: unsafe { mem::read::<u32>(G_RUN_FLAGS) } & RUN_IS_A_PRACTICE != 0,
            stage: match difficulty {
                DIFFICULTY_EXTRA => STAGES - 1,
                _ => 0,
            },
            mode: 0,
        })
    }

    fn run_slot(&self, run: &RunStart) -> Option<String> {
        if run.practice {
            return None;
        }
        const DIFFICULTIES_NAMED: [&str; 5] = ["easy", "normal", "hard", "lunatic", "extra"];
        const CHARACTERS: [&str; 2] = ["reimu", "marisa"];
        const SHOTS: [&str; 3] = ["a", "b", "c"];
        let named = |names: &[&'static str], at: i32| {
            usize::try_from(at)
                .ok()
                .and_then(|at| names.get(at).copied())
        };
        Some(
            match (
                named(&DIFFICULTIES_NAMED, run.difficulty),
                named(&CHARACTERS, run.character),
                named(&SHOTS, run.shot_type),
            ) {
                (Some(difficulty), Some(character), Some(shot)) => {
                    format!("{difficulty}-{character}{shot}")
                }
                _ => format!(
                    "difficulty{}-character{}-shot{}",
                    run.difficulty, run.character, run.shot_type
                ),
            },
        )
    }

    fn named(&self) -> &'static str {
        "th10"
    }

    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8> {
        resume::encode(saved)
    }

    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept> {
        resume::decode(bytes)
    }

    /// **The stage's replay record, field for field** — see [`resume`]: read on the frame whose update is the
    /// stage's first, which is where the game's own record takes the player's part of it (0x42a450 from
    /// 0x418278) and after the build has taken the rest (0x4290bf through 0x42910d).
    unsafe fn run_state(&self) -> Vec<u8> {
        let player = Self::object(G_PLAYER).unwrap_or(0);
        let read = |at: usize| mem::read_committed::<i32>(at).unwrap_or(0);
        resume::encode_state(&resume::Numbers {
            score: unsafe { mem::read(G_SCORE) },
            power: unsafe { mem::read(G_POWER) },
            faith: unsafe { mem::read(G_FAITH) },
            faith_stands_for: unsafe { mem::read(G_FAITH_TIMER) },
            lives: unsafe { mem::read(G_LIVES) },
            the_number_at_474c98: unsafe { mem::read(G_474C98) },
            continues: unsafe { mem::read(G_CONTINUES) },
            extends: unsafe { mem::read(G_EXTENDS) },
            seed: unsafe { mem::read(G_RANDOM) },
            at: (
                read(player + PLAYER_HUNDREDTHS),
                read(player + PLAYER_HUNDREDTHS + 4),
            ),
            places: (0..resume::PLACES_KEPT)
                .map(|place| {
                    let at = player + PLAYER_PLACES + place * 8;
                    (read(at), read(at + 4))
                })
                .collect(),
            focused: read(player + PLAYER_FOCUSED),
            options: (0..resume::OPTION_NUMBERS)
                .map(|number| {
                    read(
                        player + PLAYER_OPTIONS + number / 8 * PLAYER_OPTION_BYTES + number % 8 * 4,
                    )
                })
                .collect(),
            deaths: DEATHS.load(Ordering::Relaxed),
            began_out_of_a_fade: Self::object(G_GAME)
                .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
                .is_some_and(|flags| flags & GAME_FADING_IN != 0),
            unhittable_for: read(player + PLAYER_UNHITTABLE_FOR),
            small_faith_ring_at: Self::object(G_ITEM_MANAGER)
                .map_or(0, |manager| read(manager + SMALL_FAITH_RING_AT)),
        })
    }

    /// **Put back the way the game's own playback puts a stage back** — the numbers as 0x4292de through
    /// 0x429347 does, and the player's as 0x42a5a8 through 0x42a681 does: where it is in hundredths and as the
    /// floats 0x428e10 makes of them, its past places, the options rebuilt for the power through `0x426f70`,
    /// and their numbers and flags after.
    unsafe fn set_run_state(&self, state: &[u8]) {
        let Some(numbers) = resume::decode_state(state) else {
            return;
        };
        unsafe {
            mem::write(G_SCORE, numbers.score);
            mem::write(G_POWER, numbers.power);
            mem::write(G_FAITH, numbers.faith);
            mem::write(G_FAITH_TIMER_BEFORE, numbers.faith_stands_for - 1);
            mem::write(G_FAITH_TIMER, numbers.faith_stands_for);
            mem::write(G_FAITH_TIMER_AS_A_FLOAT, numbers.faith_stands_for as f32);
            mem::write(G_LIVES, numbers.lives);
            mem::write(G_474C98, numbers.the_number_at_474c98);
            mem::write(G_CONTINUES, numbers.continues);
            mem::write(G_EXTENDS, numbers.extends);
        }
        DEATHS.store(numbers.deaths, Ordering::Relaxed);
        let Some(player) = Self::object(G_PLAYER) else {
            return;
        };
        unsafe {
            mem::write(player + PLAYER_HUNDREDTHS, numbers.at.0);
            mem::write(player + PLAYER_HUNDREDTHS + 4, numbers.at.1);
            mem::write(player + PLAYER_POSITION, numbers.at.0 as f32 * HUNDREDTH);
            mem::write(
                player + PLAYER_POSITION + 4,
                numbers.at.1 as f32 * HUNDREDTH,
            );
            for (place, (x, y)) in numbers.places.iter().enumerate() {
                mem::write(player + PLAYER_PLACES + place * 8, *x);
                mem::write(player + PLAYER_PLACES + place * 8 + 4, *y);
            }
            mem::write(player + PLAYER_FOCUSED, numbers.focused);
            let options_for_the_power: extern "stdcall" fn(usize) =
                std::mem::transmute(the_options_made());
            options_for_the_power(player);
            for (number, value) in numbers.options.iter().enumerate() {
                mem::write(
                    player + PLAYER_OPTIONS + number / 8 * PLAYER_OPTION_BYTES + number % 8 * 4,
                    *value,
                );
            }
            for option in 0..4 {
                mem::write::<u32>(
                    player + PLAYER_OPTIONS + option * PLAYER_OPTION_BYTES + OPTION_MOVED,
                    0,
                );
            }
            mem::write(
                player + PLAYER_UNHITTABLE_BEFORE,
                numbers.unhittable_for - 1,
            );
            mem::write(player + PLAYER_UNHITTABLE_FOR, numbers.unhittable_for);
            mem::write(
                player + PLAYER_UNHITTABLE_AS_A_FLOAT,
                numbers.unhittable_for as f32,
            );
            if let Some(manager) = Self::object(G_ITEM_MANAGER) {
                mem::write(manager + SMALL_FAITH_RING_AT, numbers.small_faith_ring_at);
            }
        }
        THE_RUN_BEGAN_OUT_OF_A_FADE.store(numbers.began_out_of_a_fade, Ordering::Relaxed);
    }

    /// **The update the run's stage spent firing before a pick-up's stage could**, run here: the shot's step
    /// and the shots' move, once, on the run's own first word — see [`resume::Numbers::began_out_of_a_fade`] for
    /// the pair that measured the update missing.
    ///
    /// **Only where the run's stage began out of a fade and the pick-up's timer is still held**: a stage the run
    /// built from the menu — stage 1, the Extra, a practice — did not fire on that update either, and those
    /// land field for field as they are.
    ///
    /// **By the game's own two calls and not by writing the timer forward**, the timer being the smaller half
    /// of what that update did: it fired the shots at the count it stood at, and those six are in flight on
    /// the run's frame 2 and not on the pick-up's. And under the same two conditions the player's calc puts in
    /// front of the step (0x4262b4 through 0x4262c3), an enemy up and no conversation.
    ///
    /// **And the background the run's stage had running through its fade**, caught up here at the same moment
    /// for the same reason — see [`A_FADE_RUNS_THE_BACKGROUND_AHEAD_BY`]: its calc run as many more times, with
    /// the generator kept as it stood, the record's seed being the one the stage began on whatever the
    /// background drew on the way to it.
    unsafe fn starts_the_shot_a_stage_began_with(&self, word: u16) {
        let faded = THE_RUN_BEGAN_OUT_OF_A_FADE.swap(false, Ordering::Relaxed);
        if faded && let Some(background) = Self::object(G_STAGE_BACKGROUND) {
            unsafe {
                let generator = mem::read::<u16>(G_RANDOM);
                let draws = mem::read::<u32>(G_RANDOM_DRAWS);
                for _ in 0..A_FADE_RUNS_THE_BACKGROUND_AHEAD_BY {
                    with_eax(the_backgrounds_calc(), background);
                }
                mem::write(G_RANDOM, generator);
                mem::write(G_RANDOM_DRAWS, draws);
            }
            crate::log!(
                "resume: the background caught up {A_FADE_RUNS_THE_BACKGROUND_AHEAD_BY} frames to a stage begun out of a \
                 fade: the camera at {:.2},{:.2}",
                unsafe { mem::read::<f32>(G_CAMERA) },
                unsafe { mem::read::<f32>(G_CAMERA + 4) },
            );
        }
        let Some(player) = Self::object(G_PLAYER) else {
            return;
        };
        let enemies = Self::object(G_ENEMY_MANAGER)
            .and_then(|manager| mem::read_committed::<i32>(manager + ENEMY_COUNT))
            .unwrap_or(0);
        let talking = Self::object(G_PANEL)
            .and_then(|panel| mem::read_committed::<usize>(panel + PANEL_CONVERSATION))
            .unwrap_or(0);
        let the_pick_ups = mem::read_committed::<i32>(player + PLAYER_SHOT_TIMER).unwrap_or(-1);
        if !faded || the_pick_ups >= 0 || enemies <= 0 || talking != 0 {
            return;
        }
        unsafe {
            let read = mem::read::<u16>(G_RUN_INPUT);
            mem::write(G_RUN_INPUT, word);
            with_eax(the_shot_step(), player);
            let moved: extern "stdcall" fn(usize) = std::mem::transmute(the_shots_moved());
            moved(player);
            mem::write(G_RUN_INPUT, read);
        }
        crate::log!(
            "resume: the shot caught up to a stage begun out of a fade: its timer at {}",
            unsafe { mem::read::<i32>(player + PLAYER_SHOT_TIMER) },
        );
    }

    /// **The seed, where the game's own playback writes it**, with the draws counted from nought (0x4292d2
    /// through 0x4292d8) — from the build's own thread, at the entry of [`REPLAY_BUILT`].
    unsafe fn seed_the_stage(&self, state: &[u8]) {
        let Some(numbers) = resume::decode_state(state) else {
            return;
        };
        unsafe {
            mem::write(G_RANDOM, numbers.seed);
            mem::write::<u32>(G_RANDOM_DRAWS, 0);
        }
    }

    fn say_run_state(&self, state: &[u8]) -> String {
        match resume::decode_state(state) {
            Some(numbers) => format!(
                "score={} power={} faith={} faith_stands_for={} lives={} 474c98={} continues={} extends={} \
                 seed={:#06x} at={},{} deaths={}",
                numbers.score,
                numbers.power,
                numbers.faith,
                numbers.faith_stands_for,
                numbers.lives,
                numbers.the_number_at_474c98,
                numbers.continues,
                numbers.extends,
                numbers.seed,
                numbers.at.0,
                numbers.at.1,
                numbers.deaths,
            ),
            None => "no numbers of a run of th10".to_owned(),
        }
    }

    /// **The frame a run has been asked for and nothing of it built**: the title up, the run wanted, and no
    /// replay asked — the shot select writes the scene wanted as 7 (0x430f5a) and the supervisor builds the
    /// run on its next walk, before the title's own job runs again.
    unsafe fn run_chosen(&self) -> bool {
        let (scene, wanted) = unsafe { self.scenes() };
        scene == SCENE_TITLE && wanted == SCENE_RUN
    }

    /// **The stage, the stage the run started at, and the record the build reads the stage's files through**,
    /// which is what the shot select writes for stage 1 or the Extra (0x430f16 through 0x430f4f) and what the
    /// practice's select writes for the stage it offers (0x431394 on): the build loads whatever `0x477848`
    /// names, whatever the stage says.
    unsafe fn start_stage(&self, stage: i32) -> bool {
        if !(0..STAGES).contains(&stage) {
            return false;
        }
        unsafe {
            mem::write::<i32>(G_STAGE, stage + 1);
            mem::write::<i32>(G_STARTING_STAGE, stage + 1);
            mem::write::<usize>(
                G_STAGE_LOADS,
                STAGE_LOADS + (stage as usize + 1) * STAGE_LOADS_BYTES,
            );
        }
        true
    }

    /// **A run of these numbers, written where the title's own screens write them**: the difficulty (0x4305cb),
    /// the character (0x430997), the shot (0x430ec0), the stage and its record ([`Th10::start_stage`]), the
    /// practice and the demo's bits taken off (0x42d7d4, 0x42ce55), and the scene wanted at a run (0x430f5a).
    ///
    /// **And the title left to the scene switch**, which takes it down itself on its way into a run (0x421a03)
    /// — where th08's front end, run from its own job, stays on the chain under a scene written from outside.
    unsafe fn start_a_run(&self, run: &RunStart) -> bool {
        if !(0..STAGES).contains(&run.stage) || !(0..DIFFICULTIES).contains(&run.difficulty) {
            log!(
                "replay: there is no stage {} of difficulty {} in th10",
                run.stage + 1,
                run.difficulty
            );
            return false;
        }
        unsafe {
            mem::write::<i32>(G_DIFFICULTY, run.difficulty);
            mem::write::<i32>(G_CHARACTER, run.character);
            mem::write::<i32>(G_SHOT_TYPE, run.shot_type);
            let flags = mem::read::<u32>(G_RUN_FLAGS);
            mem::write::<u32>(G_RUN_FLAGS, flags & !(RUN_IS_A_PRACTICE | RUN_IS_THE_DEMO));
            self.start_stage(run.stage);
            mem::write::<i32>(G_SCENE_WANTED, SCENE_RUN);
        }
        true
    }

    /// **The end of a run, whichever way it ended**: the pause object's states 6 to 13 — the game over and
    /// the clear alike, with its name entry and its replay offer (0x4231d0, 0x423370) — and the ending.
    unsafe fn run_finished(&self) -> bool {
        unsafe { self.scenes() }.0 == SCENE_ENDING || self.the_end_of_a_run_is_up()
    }

    /// **The run's numbers the replay's record keeps, and where the player is**, which is what two passes over a
    /// stage that parted would part on first: the generator and the player's place are what the fire is aimed and
    /// drawn from, and `0x474c98` is what the enemies' script reads.
    unsafe fn reproduction(&self) -> String {
        let player = Self::object(G_PLAYER).unwrap_or(0);
        let read = |at: usize| mem::read_committed::<i32>(at).unwrap_or(0);
        format!(
            "frames={} seed={:#06x} draws={} score={} power={} faith={} lives={} 474c98={} at={},{}",
            unsafe { mem::read::<u32>(G_STAGE_FRAMES) },
            unsafe { mem::read::<u16>(G_RANDOM) },
            unsafe { mem::read::<u32>(G_RANDOM_DRAWS) },
            unsafe { mem::read::<u32>(G_SCORE) },
            unsafe { mem::read::<u16>(G_POWER) },
            unsafe { mem::read::<u32>(G_FAITH) },
            unsafe { mem::read::<i32>(G_LIVES) },
            unsafe { mem::read::<i32>(G_474C98) },
            read(player + PLAYER_HUNDREDTHS),
            read(player + PLAYER_HUNDREDTHS + 4),
        )
    }

    /// **The player's clocks and its shots, the enemies and the items, as their fields are** — which is what a
    /// pick-up of stage 2, 4 or 5 parting from its run is looked for in: the generator parted on the stage's
    /// frame 202, 147 and 197 with every field of the reproduction line agreeing, the playback's score five
    /// frames ahead of the run's from frame 132.
    ///
    /// **A shot by its own bytes and not the two sprites and the target past them**, which are pointers into a
    /// heap every launch lays out again. An enemy by where it is, its life, its script's time and its flags, and
    /// not by its address, for the same reason; each shot and item with its slot first, the slot a free one is
    /// found by being part of what a run does.
    unsafe fn the_world(&self) -> Option<crate::trace::TheWorld> {
        let player = Self::object(G_PLAYER)?;
        let int = |at: usize| f64::from(mem::read_committed::<i32>(at).unwrap_or(0));
        let float = |at: usize| f64::from(mem::read_committed::<f32>(at).unwrap_or(f32::NAN));
        let shots = (0..PLAYER_SHOT_SLOTS)
            .map(|slot| (slot, player + PLAYER_SHOTS + slot * PLAYER_SHOT_BYTES))
            .filter(|(_, shot)| int(shot + PLAYER_SHOT_STATE) != 0.0)
            .map(|(slot, shot)| {
                std::iter::once(slot as f64)
                    .chain(
                        (0..PLAYER_SHOTS_OWN_BYTES)
                            .step_by(size_of::<u32>())
                            .map(|at| int(shot + at)),
                    )
                    .collect()
            })
            .collect();
        let enemies = self
            .the_enemies()
            .into_iter()
            .map(|enemy| {
                vec![
                    float(enemy + ENEMY_POSITION),
                    float(enemy + ENEMY_POSITION + 4),
                    int(enemy + ENEMY_LIFE),
                    float(enemy + ENEMY_SCRIPT_TIME),
                    int(enemy + ENEMY_FLAGS),
                ]
            })
            .collect();
        let items = Self::object(G_ITEM_MANAGER)
            .map(|manager| {
                (0..ALL_ITEM_SLOTS)
                    .map(|slot| (slot, manager + ITEMS_AT + slot * ITEM_BYTES))
                    .filter(|(_, item)| int(item + ITEM_STATE) != 0.0)
                    .map(|(slot, item)| {
                        vec![
                            slot as f64,
                            int(item + ITEM_STATE),
                            int(item + ITEM_TYPE),
                            float(item + ITEM_POSITION),
                            float(item + ITEM_POSITION + 4),
                        ]
                    })
                    .collect()
            })
            .unwrap_or_default();
        let manager = Self::object(G_ENEMY_MANAGER);
        // **The run's whole block of numbers and the enemy manager's object, word for word**, and not only the
        // fields of them the record carries: a pick-up of stage 4 had an enemy come on at frame 31 fourteen and a
        // half higher than the run's, with no draw of the generator and nothing else of the world apart — so
        // something its script read was carried out of the stage before, and these are where to look first.
        let words = |from: usize, bytes: usize| -> Vec<Vec<f64>> {
            (0..bytes / size_of::<u32>())
                .map(|word| {
                    let at = from + word * size_of::<u32>();
                    vec![at as f64, int(at), float(at)]
                })
                .collect()
        };
        let the_runs_numbers = words(G_RUN_NUMBERS, RUN_NUMBERS_BYTES);
        let the_managers_own = manager.map_or_else(Vec::new, |at| words(at, ENEMY_MANAGER_BYTES));
        // **The bullets nearest the run, from their place to their state word for word**, and not every live
        // one: a card holds five hundred and more, and the stretch of the bullet past its sprite's own block is
        // where what it looks like is kept — `神符「杉で結ぶ古き縁」` fires its aimed rings green and turns them
        // white, and the strategy wiki's way through it turns back only once every one has turned.
        let (x, y) = (
            float(player + PLAYER_POSITION),
            float(player + PLAYER_POSITION + 4),
        );
        let mut bullets: Vec<Vec<f64>> = Self::object(G_BULLET_MANAGER)
            .map(|manager| {
                (0..BULLET_SLOTS)
                    .map(|slot| (slot, manager + BULLETS_AT + slot * BULLET_BYTES))
                    .filter(|(_, bullet)| {
                        mem::read_committed::<u16>(bullet + BULLET_STATE).unwrap_or(0) != 0
                    })
                    .map(|(slot, bullet)| {
                        let (bx, by) = (
                            float(bullet + BULLET_POSITION),
                            float(bullet + BULLET_POSITION + 4),
                        );
                        [(bx - x).hypot(by - y), slot as f64]
                            .into_iter()
                            .chain(
                                (NEAREST_BULLETS_FROM..NEAREST_BULLETS_TO)
                                    .step_by(size_of::<u32>())
                                    .map(|at| int(bullet + at)),
                            )
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default();
        bullets.sort_by(|a, b| a[0].total_cmp(&b[0]));
        bullets.truncate(NEAREST_BULLETS);
        Some(crate::trace::TheWorld {
            numbers: vec![
                ("state".to_owned(), int(player + PLAYER_STATE)),
                ("shot_timer".to_owned(), int(player + PLAYER_SHOT_TIMER)),
                ("state_timer".to_owned(), int(player + PLAYER_STATE_TIMER)),
                ("update_timer".to_owned(), int(player + PLAYER_UPDATE_TIMER)),
                ("unhittable".to_owned(), int(player + PLAYER_UNHITTABLE_FOR)),
                ("focused".to_owned(), int(player + PLAYER_FOCUSED)),
                ("x".to_owned(), float(player + PLAYER_POSITION)),
                ("y".to_owned(), float(player + PLAYER_POSITION + 4)),
                (
                    "manager_frames".to_owned(),
                    manager.map_or(f64::NAN, |at| int(at + ENEMY_MANAGER_FRAMES)),
                ),
                (
                    "bullets".to_owned(),
                    Self::object(G_BULLET_MANAGER).map_or(f64::NAN, |at| int(at + BULLETS_ALIVE)),
                ),
                (
                    "draws".to_owned(),
                    f64::from(unsafe { mem::read::<u32>(G_RANDOM_DRAWS) }),
                ),
                (
                    "seed".to_owned(),
                    f64::from(unsafe { mem::read::<u16>(G_RANDOM) }),
                ),
                ("camera_x".to_owned(), float(G_CAMERA)),
                ("camera_y".to_owned(), float(G_CAMERA + 4)),
                ("camera_z".to_owned(), float(G_CAMERA + 8)),
                (
                    "game_frames".to_owned(),
                    Self::object(G_GAME).map_or(f64::NAN, |game| int(game + GAME_FRAMES)),
                ),
            ],
            tables: vec![
                ("shots".to_owned(), shots),
                ("enemies".to_owned(), enemies),
                ("items".to_owned(), items),
                ("the_runs_numbers".to_owned(), the_runs_numbers),
                ("the_enemy_managers_own".to_owned(), the_managers_own),
                ("the_nearest_bullets".to_owned(), bullets),
            ],
        })
    }

    fn button_bit(&self, name: &str) -> Option<u16> {
        Some(match name {
            "shot" => BUTTON_SHOT,
            "bomb" => BUTTON_BOMB,
            "focus" => BUTTON_FOCUS,
            "skip" => BUTTON_SKIP,
            "up" => PAD_UP,
            "down" => PAD_DOWN,
            "left" => PAD_LEFT,
            "right" => PAD_RIGHT,
            "decide" => MENU_DECIDE,
            "cancel" => MENU_CANCEL,
            "menu" => BUTTON_MENU,
            _ => return None,
        })
    }

    unsafe fn what_a_bot_sees(&self, into: &mut crate::game::Frame) -> bool {
        unsafe { sight::what_a_bot_sees(into) }
    }

    unsafe fn front_end_cursor(&self) -> Option<i32> {
        mem::read_committed::<i32>(unsafe { self.title() }? + TITLE_CURSOR)
    }

    unsafe fn front_end_screen(&self) -> Option<i32> {
        mem::read_committed::<i32>(unsafe { self.title() }? + TITLE_SCREEN)
    }

    /// **The frames the stage is built on**, which the loader thread sets bit 4 of the game object's flags
    /// for (0x417876) and clears as it ends (0x417c30).
    unsafe fn mid_build(&self) -> bool {
        Self::object(G_GAME)
            .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
            .is_some_and(|it| it & GAME_BEING_BUILT != 0)
    }

    fn midstage_table(&self) -> &'static [&'static [Boundary]] {
        &chapters::MIDSTAGE
    }

    fn stages_past_the_numbered(&self) -> &'static [&'static str] {
        &["extra"]
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptClock;

    /// The frames of a stage, each with the latest time its scripts read on it, from its frame 1 on.
    fn kept(times: &[Option<i32>]) -> (ScriptClock, Vec<i32>) {
        let mut clock = ScriptClock::EMPTY;
        let read = times
            .iter()
            .enumerate()
            .map(|(frame, time)| clock.at(frame + 1, || vec![0x1000], |_| *time))
            .collect();
        (clock, read)
    }

    /// The clock climbs with the scripts and stands where their time goes back, which is a loop waiting on a
    /// fight or a sub called from the start.
    #[test]
    fn the_clock_is_the_latest_time_the_scripts_reached() {
        let (_, read) = kept(&[
            Some(1),
            Some(2),
            Some(1),
            Some(2),
            Some(3),
            Some(0),
            Some(4),
        ]);
        assert_eq!(read, [1, 2, 2, 2, 3, 3, 4]);
    }

    /// And stands where no script of the stage is left to read.
    #[test]
    fn the_clock_stands_with_no_script_left() {
        let (_, read) = kept(&[Some(5), None, None, Some(6)]);
        assert_eq!(read, [5, 5, 5, 6]);
    }

    /// A frame gone back to reads what it read the first time, however the frames after it were kept, and the
    /// frames after it are kept again from there.
    #[test]
    fn a_frame_gone_back_to_reads_its_own_time() {
        let (mut clock, _) = kept(&[Some(10), Some(20), Some(30), Some(40)]);
        assert_eq!(clock.at(2, || vec![0x2000], |_| Some(99)), 20);
        assert_eq!(clock.at(3, || vec![0x2000], |_| Some(21)), 21);
        assert_eq!(clock.at(4, || vec![0x2000], |_| Some(22)), 22);
    }

    /// And a frame asked for twice is read once.
    #[test]
    fn a_frame_asked_for_twice_is_read_once() {
        let mut clock = ScriptClock::EMPTY;
        assert_eq!(clock.at(1, || vec![0x1000], |_| Some(1)), 1);
        assert_eq!(clock.at(1, || vec![0x1000], |_| Some(7)), 1);
        assert_eq!(clock.at(2, || vec![0x1000], |_| Some(2)), 2);
    }

    /// The stage's frame 0 is a stage being built, which starts the clock and the stage's scripts over.
    #[test]
    fn a_stage_built_starts_the_clock_over() {
        let (mut clock, _) = kept(&[Some(100), Some(200)]);
        assert_eq!(clock.at(0, || vec![0x1000], |_| Some(300)), 0);
        let mut asked = Vec::new();
        assert_eq!(
            clock.at(
                1,
                || vec![0x3000, 0x4000],
                |scripts| {
                    asked.extend_from_slice(scripts);
                    Some(1)
                }
            ),
            1
        );
        assert_eq!(asked, [0x3000, 0x4000]);
    }
}
