//! th11, as a [`Game`] that answers what has been measured and declines the rest.
//!
//! `th11.exe`, 688,128 bytes, `md5 6af1e3b37f28293f4fc127ee01d2632c`, `ver 1.00a`. Every address below was
//! read off that exe — Ghidra's decompilation to find a thing and `i686-w64-mingw32-objdump -d -M intel` to
//! say it — and nothing carried across from th10's, which is the same engine with every address and most
//! offsets its own. `asm.md` beside this file is that reading; the constants here are what orb does with it.
//!
//! **What this game answers so far** is its frame's two walks and its frame, its scenes, its keys, the screens
//! and cursor of its front end, the run chosen and the run's own numbers, the generator and the stage's clocks,
//! the player with its box and its four speeds and its states, the enemies' bodies, the bullets with their
//! steps and boxes, both kinds of laser with the phase that says when each hits, the items, the boss and the
//! card up, and whether a conversation is on. So `Game::what_a_bot_sees` fills a whole frame, the rules steer
//! a run of it, and a recipe walks its front end.
//!
//! **And it rewinds** — see [`Th11::rewinds`]: a chapter is cut at each stage's first frame and at each attack, a
//! death answered by putting the chapter back, and a run left unfinished written down and picked up again out of
//! the stage's own replay record.
#![doc = include_str!("asm.md")]

/// The midstage table, in a file of its own because a `--tune` pass writes one to paste over it.
pub mod chapters;
/// The address space laid out for a test with no game running — see `orb-e2e`'s `fake::th11`.
#[cfg(any(test, feature = "sim"))]
pub mod image;
/// The file a run left unfinished is written in — see `Game::encode_resume`.
pub mod resume;
/// The score file as bytes — see `Th11::read_what_the_modes_file_records`.
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

/// **How many hits the run has taken, which is orb's count** — see [`Th11::read_state`], which moves it. The
/// game keeps a count of its own as well, the enemy manager's `+0x10` one higher at every death (0x431247), but
/// that manager is built again with every stage and its count with it.
static DEATHS: AtomicI32 = AtomicI32::new(0);
/// Whether the player was in [`PLAYER_STATE_HIT`] on the frame before, which is what makes the count above
/// move once per hit rather than once per frame of one.
static WAS_HIT: AtomicBool = AtomicBool::new(false);
/// **And the count each frame of the stage had**, by the stage's frame — th10's reason: the count is orb's and
/// would carry a death a retry took back into every chapter after it.
static DEATHS_BY_FRAME: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());
/// **Whether the run's stage began at the end of the last one's fade**, out of the numbers a pick-up was put back
/// with — see [`resume::Numbers::began_out_of_a_fade`].
static THE_RUN_BEGAN_OUT_OF_A_FADE: AtomicBool = AtomicBool::new(false);
/// **The game object and the frame of it a stage was last said to begin on** — see
/// [`Th11::a_stage_has_just_been_built`] — put back to nothing by the next build.
static BEGUN: std::sync::Mutex<Option<(usize, i32)>> = std::sync::Mutex::new(None);
/// **Whether the last look [`Th11::a_stage_has_just_been_built`] took found the stage being built** — see
/// [`Th11::walks_a_frame_of_its_build`].
static A_BUILD_WAS_GOING_ON: AtomicBool = AtomicBool::new(false);

/// **The stage's script frames as orb keeps them**: the frames of the stage on which none of its own scripts was held
/// on an instruction that waits, kept for every frame of the stage the clock has reached — see
/// [`Th11::script_frames`].
///
/// **The frames the scripts were not held on, and not the latest time they reached**, which is th10's clock and was
/// this game's first: a `call` here runs the sub in the caller's own context with its time at nought (0x45b80f, the
/// caller's pushed at 0x45b78f), so a stage whose `main` calls one sub after another read `script=60` through all of
/// stage 1 to its boss, 160 through stages 2 and 3 and 0 through stage 4, and a `--tune` pass of a whole Normal
/// proposed one boundary (`tune-n1`, 2026-10-10). **Nor the frames their time moved on**, which was the second: the
/// stage's first frame has two scripts beside `main` that jump back round 1, 2, 3 for the whole stage, and that clock
/// read the stage's own frames to the frame. What stands still is `main` held on `0x154`, which waits while a boss is
/// up (0x415b1e through 0x415b30), or on `0x153` — stage 3 practised alone had it on `0x154` with its time at 160 from
/// the midboss's arrival to its end and through the boss — and the stage waits its midboss out so, measured: stage 3
/// practised alone three times brought its first wave after the midboss 121 frames after the midboss was beaten each
/// time, at stage frames 5611, 4933 and 5052.
static SCRIPT_CLOCK: std::sync::Mutex<ScriptClock> = std::sync::Mutex::new(ScriptClock::EMPTY);

/// **The most frames the clock keeps of one stage**: two hours and more at sixty, th10's bound.
const A_STAGE_RUNS_AT_MOST: usize = 1 << 19;

struct ScriptClock {
    /// The stage's own scripts, taken on the stage's first frame — see [`Th11::the_stage_scripts`].
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
    /// and `held` asked of them on each frame counted for the first time: the frames since the frame asked before,
    /// where none of them is held, and none where one is.
    fn at(
        &mut self,
        at: usize,
        scripts: impl FnOnce() -> Vec<usize>,
        held: impl FnOnce(&[usize]) -> bool,
    ) -> i32 {
        if at == 0 {
            *self = Self::EMPTY;
            return 0;
        }
        // A frame past any stage's own answers what the clock last stood at — th10's clock has why.
        if at > A_STAGE_RUNS_AT_MOST {
            return self.by_frame.get(self.last).copied().unwrap_or(0);
        }
        if self.by_frame.is_empty() {
            self.scripts = scripts();
            self.by_frame.push(0);
            self.last = 0;
        }
        if at <= self.last {
            self.last = at;
            return self.by_frame[at];
        }
        self.by_frame.truncate(self.last + 1);
        let before = self.by_frame[self.last];
        let count = match held(&self.scripts) {
            true => before,
            false => before + i32::try_from(at - self.last).unwrap_or(i32::MAX),
        };
        self.by_frame.resize(at + 1, count);
        self.last = at;
        count
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

/// th11 — 東方地霊殿, Subterranean Animism — as the [`Game`] whose addresses are in this module.
pub struct Th11;

/// The `IDirect3DDevice9` the game's `CreateDevice` made (`&0x4c3288` pushed at 0x446f5d).
const G_D3D_DEVICE: usize = 0x004c_3288;

/// The window object the frame is called on, whose `+0x0` is the HWND `CreateWindowExA` answered (0x446cc3).
const G_GAME_WINDOW: usize = 0x004c_3d88;

/// **The frame**, `stdcall` on [`G_GAME_WINDOW`] (`ret 0x4` at 0x4464fd): the one of the three WinMain chooses
/// between that is th10's frame in its order, limiter and all — see `asm.md`. Its trampoline is what a frame orb
/// hands back reaches, whichever of the three the pump called.
const RENDER: usize = 0x0044_6420;
/// `push ebx; push ebp; mov ebp,[esp+0xc]`, six bytes ending on an instruction.
const RENDER_PROLOGUE: &[u8] = &[0x53, 0x55, 0x8b, 0x6c, 0x24, 0x0c];
/// **The other two frames the pump picks between** (0x445874 through 0x4458a7): `0x446080` where bit `0x10` of
/// `[0x4c3dc0]` stands — the default, which paces in its present — and `0x446650` where the present waits for the
/// blank. `test byte [0x4c3dc0],0x10`, seven bytes holding an absolute address and not a relative one; and the
/// same six bytes as [`RENDER_PROLOGUE`].
const OTHER_RENDERS: &[Patch] = &[
    Patch {
        target: 0x0044_6080,
        prologue: &[0xf6, 0x05, 0xc0, 0x3d, 0x4c, 0x00, 0x10],
    },
    Patch {
        target: 0x0044_6650,
        prologue: &[0x53, 0x55, 0x8b, 0x6c, 0x24, 0x0c],
    },
];

/// The chain, whose calc list the frame walks through [`RUN_CALC_CHAIN`] — the chain in `ebx` — and whose draw
/// list through [`RUN_DRAW_CHAIN`], which takes nothing (see `FrameEntered::ChainInEbx`).
const G_CHAIN: usize = 0x004c_3234;
const RUN_CALC_CHAIN: usize = 0x0045_6cb0;
const RUN_DRAW_CHAIN: usize = 0x0045_6e10;
/// `sub esp,0x8; test dword [0x4c3810],0x8000` — each walk's first thirteen bytes, the same in both, the second
/// an absolute address and an immediate.
const A_WALKS_PROLOGUE: &[u8] = &[
    0x83, 0xec, 0x08, 0xf7, 0x05, 0x10, 0x38, 0x4c, 0x00, 0x00, 0x80, 0x00, 0x00,
];

/// **The sound manager**, `0x52f4` bytes `0x448ff0` zeroes: the id of its music's thread at `+0x614`, which
/// `CreateThread` writes as it makes it (`lea eax,[esi+0x614]` pushed at 0x449b9f).
const G_SOUND_MANAGER: usize = 0x004c_3e80;
const SOUND_MUSIC_THREAD: usize = 0x614;
const SOUND_MANAGER_BYTES: usize = 0x52f4;
/// **The queue of the sound's commands**, whose head is the command being carried out and nought for none: 31
/// slots of `0x10c` bytes, `{kind, argument, step, name}` (0x44a9c0), and the kind a song's play is queued as.
const SOUND_COMMANDS: usize = 0x1f88;
const SOUND_COMMAND_SLOTS: usize = 31;
const SOUND_COMMAND_BYTES: usize = 0x10c;
const SOUND_COMMAND_ARGUMENT: usize = 0x4;
const SOUND_A_PLAY: u32 = 2;
/// **The music's stream**, and the source it reads through at `+0xc`, whose `+0x90` is the record of
/// `thbgm.fmt` it was last put on — `0x45b2d0` writes the record's address there (0x45b31d).
const MUSIC_STREAM: usize = 0x5208;
const STREAM_SOURCE: usize = 0xc;
const SOURCE_RECORD: usize = 0x90;
/// **A record of `thbgm.fmt`**: the track's file name at its start, which `0x4496b0` looks the record up by
/// (0x449725 through 0x44973f), and after the name's 16 bytes where its track is in `thbgm.dat` and how long it is
/// (0x449db5, 0x449dc6).
/// **The heap objects the sound is beside the manager**: the wrapper `+0x610` keeps DirectSound in, `new(4)`
/// (0x4491a6); the stream, `new(0x7c)` (0x45a0fb); the holder of its buffer at `+0x4`, `new(4)` (0x45a46d); and its
/// source, `new(0x9c)` (0x459f28). The music's thread writes the stream and the source as it refills (0x44a900,
/// 0x45aee0, 0x45b480), so a snapshot's copy of either is one the thread has gone past.
const SOUND_DEVICE: usize = 0x610;
const SOUND_DEVICE_BYTES: usize = 4;
const MUSIC_STREAM_BYTES: usize = 0x7c;
const STREAM_BUFFER: usize = 0x4;
const STREAM_BUFFER_BYTES: usize = 4;
const MUSIC_SOURCE_BYTES: usize = 0x9c;
const RECORD_NAME: usize = 0x0;
const RECORD_NAME_BYTES: usize = 0x10;
const RECORD_AT: usize = 0x10;
const RECORD_BYTES: usize = 0x14;
/// **The manager's two songs a stage loads**, `0x100` bytes of path apiece from `+0x4108`: the build loads the
/// stage record's `+0x10` into slot 0 and `+0x14` into slot 1 (0x41fd57, 0x41fd68), the stage's first frame plays
/// slot 0 (0x420525) and the conversation's script slot 1 (0x41e9d7) — so slot 1 is the fight's song.
const SOUND_SONGS: usize = 0x4108;
const SONG_PATH_BYTES: usize = 0x100;
const THE_FIGHTS_SONG: usize = 1;

/// The present the frame at [`RENDER`] makes, which takes nothing and pops nothing (`push ecx` … `pop ecx; ret`
/// at 0x446645) — the screenshot, `Present`, and a reset where that fails.
const PRESENT: usize = 0x0044_65d0;

/// **The sprite manager**, whose queue of quads the frame draws out either side of its draw chain.
const G_SPRITES: usize = 0x004c_3268;
/// `0x44fd10`, entered with the manager in `esi`: whatever the queue holds drawn and the queue emptied.
const DRAW_THE_QUEUE: usize = 0x0044_fd10;
// Handed over for a laid-out game to answer with a function of its own — see `handed_over!` — as are the
// camera's, the fog's, the options', the pause menu's resume, a record made empty and the shot's two below: a
// frame that jumped into memory nothing has mapped is one no e2e test could reach at all.
handed_over!(
    DRAW_THE_QUEUE_AT,
    the_queues_draw,
    set_the_queues_draw,
    DRAW_THE_QUEUE
);
/// The queue's count, and its two cursors, which the frame puts back at the queue's start as its scene
/// begins (0x446540 through 0x446557).
const SPRITES_QUEUED: usize = 0x0043_5620;
const SPRITES_QUEUE_STARTS: usize = 0x0043_5624;
const SPRITES_WRITE_AT: usize = 0x007b_5624;
const SPRITES_DRAWN_FROM: usize = 0x007b_5628;
/// **The manager's 32 `.anm` files**, a pointer apiece from `+0x4350c0` (`cmp ebp,0x20` at 0x4541c8): a file is a
/// `new(0x134)` (0x454208) whose textures are `+0x10c` entries of `0x14` from `+0x120`, an `IDirect3DTexture9` first
/// in each (0x45429e, 0x445374), and whose sprites are `+0x114` of `0x48` from `+0x118`, each holding the address of
/// its texture's entry at `+0x8` (0x454787 through 0x454797). **No surfaces of the captures beside them**, th10's
/// `+0x3ad4e0`: the render targets the captures copy into are textures of these files (0x445320, 0x4453b0).
const ANM_FILES_AT: usize = 0x0043_50c0;
const ANM_FILES: usize = 0x20;
const ANM_TEXTURES: usize = 0x10c;
const ANM_TEXTURES_AT: usize = 0x120;
const ANM_TEXTURE_BYTES: usize = 0x14;
const ANM_SPRITES: usize = 0x114;
const ANM_SPRITES_AT: usize = 0x118;
const ANM_SPRITE_BYTES: usize = 0x48;
/// How many of each a file is believed to hold at most, th10's bound on a count read off memory.
const ANM_TEXTURES_AT_MOST: u32 = 0x100;
const ANM_SPRITES_AT_MOST: u32 = 0x1000;
/// **The supervisor's three surfaces**, `+0x1ac` through `+0x1b4`: two levels of its own `.anm`'s textures and the
/// back buffer, which 0x42b530 gets and 0x42b600 releases around a reset.
const G_SURFACES: usize = 0x004c_342c;
const SURFACES_BYTES: usize = 0xc;

/// **The supervisor's twelve locks**, `CRITICAL_SECTION`s of `0x18` from `0x4c3a90` with a byte of depth apiece
/// after them to `0x4c3bbb` (0x44556f) — the chain's, the file loader's, the log's, the scene switch's, the build
/// thread's start's, the generator's and the sound's among them, which another thread may be holding as a snapshot
/// is taken.
const G_LOCKS: usize = 0x004c_3a90;
const LOCKS_BYTES: usize = 0x004c_3bbc - 0x004c_3a90;

/// **The replay's object built for a stage built fresh**, `0x435e00`, `stdcall` on the game object's `+0x74`
/// (`ret 0x4` at 0x435eaf) and called from one place, the build's 0x41fc2b on the loader's thread — the seam a
/// resumed run's seed goes in at (`Hooks::stage_building`): past the run's numbers and the player (0x41fa5b,
/// 0x41fa9f), where nothing has drawn from the generator, and before the replay reads the seed as a recording's
/// (0x435826) or writes one back as a playback's (0x435a91) and before any of the stage's objects is built.
///
/// **A fresh build is every stage a resume starts**: the stage after one won goes through `0x436f30` instead
/// (0x41fc98), and a run's start and a retry both build fresh.
const REPLAY_BUILT: usize = 0x0043_5e00;
/// `push -1; push 0x48a8ab` — seven bytes, the second an absolute address and not a relative one.
const REPLAY_BUILT_PROLOGUE: &[u8] = &[0x6a, 0xff, 0x68, 0xab, 0xa8, 0x48, 0x00];

/// **The `.anm` file's unload, `0x4549e0`, entered with the file in `edi`**: its VMs flagged for deletion through
/// 0x4564c0, which walks both of the sprite manager's VM lists (`+0x7b562c`, `+0x7b5634`), its textures released
/// (`[vtbl+0x8]` at 0x454a25) and its five buffers freed — the scripts at `+0x11c` among them — all under no lock.
///
/// **Which the title's thread calls while the frames go on**: it unloads the loading screen's file, slot 1
/// (`[sprite manager+0x4350c4]`, 0x439564), once that screen's counter reaches 180, and the loading screen's two
/// VMs of it on layer 0x16 are still being run and drawn — the update's VM walk tests the deletion flag before it
/// runs a VM (0x4554d7, 0x4554fe) and has no way to see it set halfway. A launch held for a debugger had them made
/// out of that file (`0x03207f08`) and unloaded through this call from 0x439569, and every launch that stood
/// spinning at the title or crashed did so on the frame the unload is at, f215: the interpreter on an instruction of
/// size nought in script memory that held no script, or an access violation inside the driver (`nvd3dum.dll`,
/// `0xc0000005`, 2026-10-12 13:48:29).
const UNLOAD: usize = 0x0045_49e0;
/// `push ebx; xor ebx,ebx; cmp [edi+0x108],ebx` — nine bytes, none of them relative.
const UNLOAD_PROLOGUE: &[u8] = &[0x53, 0x33, 0xdb, 0x39, 0x9f, 0x08, 0x01, 0x00, 0x00];

/// **The LZSS decode, `0x4426c0`, and its encode, `0x4423a0`, over the one window the process has**: the 8KB at
/// `0x4c0ee8` (0x442631, 0x442794 with `and ecx,0x1fff`), which neither takes a lock around. The decode is
/// entered with its output in `eax` and three arguments pushed and the encode with three pushed, both `ret 0xc`.
///
/// **Which two threads reach at once at the title**: the loading object's thread decodes `scoreth11.dat` (0x437952)
/// outside the archive's lock `0x4c3ac0`, which the archive's own decodes are inside (0x441865, 0x4419f2), and a
/// launch held for a debugger logged that decode still running on its thread as the title's thread began loading
/// `title.anm` (0x439660). A decode whose window another one writes over copies noughts wherever it copies from
/// the window, and every script of `title.anm` in a hung launch's dump was that — each instruction's opcode and
/// floats kept and its length halfword and its repeated noughts nought — so the main menu's VM of script 0 runs an
/// instruction of size nought for ever (0x43a1d4 into 0x455a00, the VM run as it is made).
const DECODE: usize = 0x0044_26c0;
/// `sub esp,0xc; cmp dword [esp+0x18],0` — eight bytes, relative to `esp` and not to the instruction.
const DECODE_PROLOGUE: &[u8] = &[0x83, 0xec, 0x0c, 0x83, 0x7c, 0x24, 0x18, 0x00];
const ENCODE: usize = 0x0044_23a0;
/// `sub esp,0x24; push ebx; push esi` — five bytes.
const ENCODE_PROLOGUE: &[u8] = &[0x83, 0xec, 0x24, 0x53, 0x56];

/// **The camera the frame's update is set up under**: `0x42a810`, entered with the camera in `edi`, sets its two
/// transforms — and the frame then sets the viewport the camera keeps at `+0xcc`, the camera having been named in
/// [`G_THE_CAMERA`] first (0x4464a9 through 0x4464cf).
const SET_THE_CAMERA: usize = 0x0042_a810;
handed_over!(
    SET_THE_CAMERA_AT,
    the_cameras_set,
    set_the_cameras_set,
    SET_THE_CAMERA
);
const THE_CAMERA: usize = 0x004c_359c;
const G_THE_CAMERA: usize = 0x004c_37cc;
const CAMERA_VIEWPORT: usize = 0xcc;
/// And the word the frame writes `1` into after it (0x4464d7).
const G_A_CAMERA_IS_SET: usize = 0x004c_37d0;

/// **The fog**, which the frame sets off under its scene: `0x42a300` entered with the supervisor in `edi`, and the
/// dword [`G_FOG_FORCED`] forced to `0xff` before it (0x446562, 0x44656c).
const SET_THE_FOG: usize = 0x0042_a300;
handed_over!(SET_THE_FOG_AT, the_fogs_set, set_the_fogs_set, SET_THE_FOG);
const G_SUPERVISOR: usize = 0x004c_3280;
const G_FOG_FORCED: usize = 0x004c_3c10;

/// **One of the game's own functions whose argument arrives in a register it chose**, called with that register
/// holding it — th10's `with_esi` has why the register is saved inside the asm.
///
/// # Safety
/// `function` must be one of this game's that reads its argument out of that register, run on the game's main
/// thread with whatever it wants of the frame.
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

/// See [`with_esi`] — `ebx` being the other register the compiler refuses as an operand.
///
/// # Safety
/// As [`with_esi`].
unsafe fn with_ebx(function: usize, ebx: usize) {
    unsafe {
        std::arch::asm!(
            "push ebx",
            "mov ebx, {ebx}",
            "call {function}",
            "pop ebx",
            ebx = in(reg) ebx,
            function = in(reg) function,
            clobber_abi("C"),
        )
    };
}

/// See [`with_esi`] — **and handed to the compiler as an operand**, `eax` being a register it lets go of.
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

/// **The float every step of a frame is scaled by**, the game's own speed, read directly by every move and every
/// timer (0x408b4c, 0x430290).
const G_GAME_SPEED: usize = 0x004a_7948;

/// **The player**, a `0x8d24`-byte object 0x4301b0 allocates and 0x42f760 stores here (0x42f852). Its calc job
/// is at priority `0x10` (0x42f95d), ahead of the lasers' `0x13` and the bullets' `0x14`, so a frame's fire is
/// tested against the player where the frame's own move put it.
const G_PLAYER: usize = 0x004a_8eb4;
/// Where it is, in the field's own coordinates: three floats, written as the whole 128ths it is kept in at
/// `+0x888` times `0.0078125` (0x430859 through 0x430870).
const PLAYER_POSITION: usize = 0x87c;
const PLAYER_128THS: usize = 0x888;
/// **The half of its box on each axis**, half of `[0x4a7aa8 + character * 4]` — `2.0` and `3.5` there, so
/// `1.0` for Reimu and `1.75` for Marisa whoever the partner (0x42fadc, 0x42fb21 through 0x42fb45).
const PLAYER_HALF: usize = 0x8e4;
/// **The shot type's record of the `.sht`**, whose `+0x4` is the whole hitbox the circle test measures from — the
/// same `2.0` and `3.5` (0x42fae9, 0x431f79 through 0x431fbf).
const PLAYER_SHT: usize = 0x92c;
const SHT_WHOLE_HITBOX: usize = 0x4;
/// **Its four speeds, in 128ths of a unit a frame**: straight, focused straight, diagonal, focused diagonal —
/// the `.sht`'s floats times `128.0` (0x42fa20 through 0x42fa60).
const PLAYER_SPEEDS: usize = 0x890;
const SPEED_SCALE: f32 = 128.0;
/// **Its state**, the switch the calc job dispatches on (0x43108e): 0 coming back in, 1 playing, 2 dying and 4 hit.
const PLAYER_STATE: usize = 0x928;
/// **Where Yukari's gap is** (0x430e50): 0 nothing, 1 and 3 against the left and the right wall (set by the clamp,
/// 0x4307de and 0x43080f, for shot type 0 alone), 2 and 4 the direction let go there, and from
/// [`GAP_STATE_THROUGH`] carried through it — 99 out of the left, 100 out of the right — for 45 frames.
const PLAYER_GAP_STATE: usize = 0x920;
const GAP_STATE_THROUGH: i32 = 99;
/// **The shot type whose player has the gap**, Reimu and Yukari — `[0x4a5714] + 3 * [0x4a5710] == 0` at the clamp
/// (0x4307d1 through 0x4307dc).
const SHOT_TYPE_WITH_A_GAP: i32 = 0;
/// Playing: the bomb asked for, and the move (0x4311ac through 0x43121b).
const PLAYER_STATE_NORMAL: i32 = 1;
/// **Hit, and inside the eight frames a bomb takes it back in** — the death at `+0x948 >= 8` (0x43122d), set by
/// the hit at 0x432a9e.
const PLAYER_STATE_HIT: i32 = 4;
/// **The frames nothing can hit the player for**, the current count of a timer that counts down (0x431658
/// through 0x4316c2) with its float after it: six as a hit is taken, 180 at a death, 280 as the player comes back
/// in and 40 under every frame of a bomb (0x432c29, 0x4328ac, 0x431506, 0x406860).
const PLAYER_UNHITTABLE_FOR: usize = 0x8bb4;
/// And that timer's count the frame before, and its float.
const PLAYER_UNHITTABLE_BEFORE: usize = 0x8bb0;
const PLAYER_UNHITTABLE_AS_A_FLOAT: usize = 0x8bb8;
/// **The element Patchouli's options are on**, which her `0x400` steps through five (0x4308fe through 0x430921).
const PLAYER_ELEMENT: usize = 0x8bac;
/// **How many options the player has**, the power over a bomb's cost (0x432cee, 0x43361d).
const PLAYER_OPTION_COUNT: usize = 0x7c90;
/// **The player's options made again for the power it has**, entered with the player in `ebx` (callers at
/// 0x41fa99, 0x435601, 0x436ee8), plain `ret`. The playback calls it twice with the count at nought where the
/// element is not nought (0x436f02 through 0x436f13), the count being what it makes options past.
const OPTIONS_FOR_THE_POWER: usize = 0x0043_2cc0;
handed_over!(
    OPTIONS_FOR_THE_POWER_AT,
    the_options_made,
    set_the_options_made,
    OPTIONS_FOR_THE_POWER
);
/// **What a 128th of the player's position is as a float** — 0x435630 turning the record's 128ths into `+0x87c`.
const A_128TH: f32 = 1.0 / 128.0;
/// **The current counts of the player's three timers a frame steps**: the shot's, held at -1 while it is off; the
/// state's; and the one every update steps, which holds the focus off for its first four frames (0x430290).
const PLAYER_SHOT_TIMER: usize = 0x934;
const PLAYER_STATE_TIMER: usize = 0x948;
const PLAYER_UPDATE_TIMER: usize = 0x95c;
/// **Whether it is focused**, `([0x4c93c0] >> 3) & 1` while an enemy is up (0x430373).
const PLAYER_FOCUSED: usize = 0x8d20;
/// **The word the player is played with**, the run's copy of [`G_INPUT`]'s low half that a recording writes and a
/// playback reads back (0x43600d) — what the shot's step reads its press out of (0x4343a7).
const G_RUN_INPUT: usize = 0x004c_93c0;
/// **The shot's step**, entered with the player in `eax`, plain `ret`: in the player's state 1 it starts the timer
/// on a press and fires at each new count (0x434380 through 0x43440b). And **the shots moved**, `stdcall` on the
/// player (`ret 0x4` at 0x4347bd), which the player's update calls straight after it (0x431b1b).
const SHOT_STEP: usize = 0x0043_4380;
handed_over!(SHOT_STEP_AT, the_shot_step, set_the_shot_step, SHOT_STEP);
const SHOTS_MOVED: usize = 0x0043_4440;
handed_over!(
    SHOTS_MOVED_AT,
    the_shots_moved,
    set_the_shots_moved,
    SHOTS_MOVED
);
/// **The panel's flags**, whose `0x10` holds the shot's step back as a conversation's does (0x431b0b through
/// 0x431b15).
const PANEL_FLAGS: usize = 0x4420;
const PANEL_HOLDS_THE_SHOT: u32 = 0x10;

/// **The rectangle the player is held inside**, as a left, a top and a size: `x` in `-184` to `184` and `y` in
/// `32` to `432`, the clamp of the 128ths at 0x4307c5 through 0x430849.
const PLAYER_BOUNDS: (f32, f32, f32, f32) = (-184.0, 32.0, 368.0, 400.0);

/// **The bullet manager**, a `0x46d678`-byte object 0x408710 allocates and stores at 0x40877f.
const G_BULLET_MANAGER: usize = 0x004a_8d68;
/// Its array of bullets: `0x7d1` slots of `0x910` bytes from `+0x64`, of which the walk visits the first `0x7d0`
/// (0x408d38), and how many it kept at `+0x5c` (0x408d0b).
const BULLETS_AT: usize = 0x64;
const BULLETS_ALIVE: usize = 0x5c;
const BULLET_BYTES: usize = 0x910;
const BULLET_SLOTS: usize = 0x7d0;
/// Its flags: `0x2` tested against the player (0x408b9c), and `0x10` round, tested as a circle whose radius is
/// [`BULLET_SIZE`]'s first word (0x408ba4).
const BULLET_FLAGS: usize = 0x0;
const BULLET_MAY_HIT: u32 = 0x2;
const BULLET_IS_ROUND: u32 = 0x10;
/// Where it is and its step a frame, three floats each; the step times [`G_GAME_SPEED`] is what its own update
/// adds (0x408b4c through 0x408b97).
const BULLET_POSITION: usize = 0x43c;
const BULLET_STEP: usize = 0x448;
/// **Its box**, a width and a height the box test halves (0x408ba8), or the round one's radius.
const BULLET_SIZE: usize = 0x45c;
/// **Its state, a word**: 0 a free slot, 1 active and tested, 2 coming out, 3 cancelled (0x408916 through
/// 0x408933, 0x408bda).
const BULLET_STATE: usize = 0x4b2;
/// **Its age**, the count of the timer from `+0x464` (0x408d90 through 0x408db7).
const BULLET_AGE: usize = 0x468;

/// **The laser manager**, a `0x47c`-byte object 0x424940 allocates and stores at 0x424963.
const G_LASER_MANAGER: usize = 0x004a_8e94;
/// The first laser of its list — the head is a laser of its own at `+0x10`, so the first real one is that one's
/// next — each laser's next at `+0x8`, and the count at `+0x454`, at most `0x100` (0x424df0).
const LASER_FIRST: usize = 0x18;
const LASER_NEXT: usize = 0x8;
const LASERS_ALIVE: usize = 0x454;
const LASERS_AT_MOST: usize = 0x100;
/// **Which of the two kinds it is, by its vtable**: `LaserInfiniteInf`, `0x494abc`, stays where it is fired from
/// and goes through a warning line, a widening, its whole width and a shrinking; `LaserLineInf`, `0x494a64`,
/// travels. The RTTI names no third.
const LASER_KIND_ANCHORED: usize = 0x0049_4abc;
/// Its phase — 3 the warning line, 4 widening, 2 whole, 5 shrinking (the switch at 0x4273bf) — tested in 4 and 2.
const LASER_PHASE: usize = 0xc;
const LASER_PHASE_FULL: i32 = 2;
const LASER_PHASE_WARNING: i32 = 3;
const LASER_PHASE_WIDENING: i32 = 4;
/// The frames it has been in its phase, the count of the timer at `+0x14`.
const LASER_TIMER: usize = 0x18;
/// Where it starts, which way it points, how long it is now, how wide it is drawn, the speed its length grows
/// at, and its own number (counted up from `0x10000`, `+0x458` of the manager).
const LASER_ORIGIN: usize = 0x3c;
const LASER_ANGLE: usize = 0x54;
const LASER_LENGTH: usize = 0x58;
const LASER_WIDTH: usize = 0x5c;
const LASER_SPEED: usize = 0x60;
const LASER_ID: usize = 0x6c;
/// **The anchored kind's own**: how long it grows to, the width it fires at and how many frames its warning line
/// stands (0x4272c0, 0x4273c9).
const LASER_LENGTH_AT_MOST_ANCHORED: usize = 0x460;
const LASER_WIDTH_WHOLE: usize = 0x468;
const LASER_WARNING_FRAMES: usize = 0x470;
/// **And the travelling kind's most**, which it grows to before its origin starts moving (0x425cc0).
const LASER_LENGTH_AT_MOST_TRAVELLING: usize = 0x450;

/// **The sounds a frame of th10 makes and th11's does not**: this game carries its sound's commands out inside the
/// calc walk, from the supervisor's own job (`call 0x44a340` at 0x429058), so the frame has no call on the sound
/// manager and orb's frame makes none either — a second one a walk would carry a queued play out twice as fast.
extern "fastcall" fn plays_no_sounds(_manager: usize) {}

/// **The pad's half of the keyboard's read**, whose answer is the word the game stores — see
/// `game::Read::PadHalfInEcx`. `push ebp; mov ebp,esp; and esp,-8`, six bytes ending on an instruction.
const INPUT_PAD_HALF: usize = 0x0045_7270;
const INPUT_PAD_HALF_PROLOGUE: &[u8] = &[0x55, 0x8b, 0xec, 0x83, 0xe4, 0xf8];

/// **The input**, `0x148` bytes the read fills, every word of it a `u32`: this frame's at `+0x0`, the frame
/// before's at `+0x4` (0x4579f2 through 0x457a05), then the repeat, the buttons gone down and those let go.
const G_INPUT: usize = 0x004c_92a8;
const G_INPUT_LAST: usize = 0x004c_92ac;

/// The buttons, as the read folds the keyboard's keys and the pad's buttons into them (0x4576ee through 0x4579e9,
/// 0x4572e8 through 0x457382).
const BUTTON_SHOT: u16 = 0x0001;
const BUTTON_BOMB: u16 = 0x0002;
/// Shift, which th10 has at `0x4`: no key and no button sets `0x4` here.
const BUTTON_FOCUS: u16 = 0x0008;
const PAD_UP: u16 = 0x0010;
const PAD_DOWN: u16 = 0x0020;
const PAD_LEFT: u16 = 0x0040;
const PAD_RIGHT: u16 = 0x0080;
/// Esc, the pause (`[0x4c92b4] & 0x100` at 0x42bf53).
const BUTTON_MENU: u16 = 0x0100;
/// Ctrl, which runs a conversation on where the conversation lets it (0x41d3ad).
const BUTTON_SKIP: u16 = 0x0200;
/// **What every menu of this game takes as decide and as cancel, inside the low half orb's word is** — see
/// `runtime::get_input_with_the_keys_in_ecx`: the menus decide on `& 0x80001` (0x42ca55), Z and Enter, and Enter is
/// `0x80000`, past a `u16`; Z alone is the half of it a word of orb's can press. Cancel is X or Esc, `& 0x102`.
const MENU_DECIDE: u16 = 0x0001;
const MENU_CANCEL: u16 = 0x0102;
/// **What the run's copy keeps of the word**: its low half (0x43600d) — every key a run is played with, Esc and
/// Ctrl among them, which th10's copy leaves out.
const RUN_INPUT: u16 = 0x03fb;

/// **The pad's mapping**, the nine `i16` `th11.cfg` holds from its `+0x4`, at `+0x134` of the input (0x429f41
/// through 0x429f76) — a button's number, negative for none — the read taking five of them (0x4572e8 through
/// 0x457382).
const G_PAD_MAPPING: usize = 0x004c_93dc;
const PAD_MAPPED: [(usize, u16); 5] = [
    (0, BUTTON_SHOT),
    (1, BUTTON_BOMB),
    (2, BUTTON_FOCUS),
    (3, BUTTON_MENU),
    (8, BUTTON_SKIP),
];

/// **The pad DirectInput found**, at `[0x4c32a4]`, with bit `0x800` of the input's flags set where it is there
/// (0x4456c8 through 0x445702) — read as a `DIJOYSTATE2` with every axis ranged ±1000 (0x447d23 through 0x447d3b),
/// its two axes against the thresholds at `+0x16` and `+0x18` of the configuration (0x45752b through 0x457582).
const G_PAD_DEVICE: usize = 0x004c_32a4;
const PAD_THROUGH_DIRECTINPUT: u32 = 0x800;
const CFG_PAD_ACROSS: usize = 0x004c_345e;
const CFG_PAD_DOWN: usize = 0x004c_3460;
const PAD_RANGE: i32 = 1000;

/// **The supervisor's scene and the scene wanted**, `+0x554` and `+0x558` of `0x4c3280`, which the switch at
/// 0x42b0e0 moves the one to where they differ.
const G_SCENE: usize = 0x004c_37d4;
const G_SCENE_WANTED: usize = 0x004c_37d8;
/// The title, which 0x439900 builds (0x42b1ac).
const SCENE_TITLE: i32 = 4;
/// **A run**, which `0x4202d0` builds the game object of (0x42b220).
const SCENE_RUN: i32 = 7;
/// **The run started over at its starting stage**, which the pause menu's retry writes (0x42d4b5) and the switch
/// turns into [`SCENE_RUN`] built fresh (0x42b29f).
const SCENE_START_OVER: i32 = 10;
/// **The stage after the one just won**, which the switch turns straight into [`SCENE_RUN`] with the run's numbers
/// carried (0x42b269) — so a run is still on while it is the scene.
const SCENE_NEXT_STAGE: i32 = 12;
/// **A continue**, the run torn down and built again where it stands (0x42b30e).
const SCENE_CONTINUE: i32 = 14;
/// The ending (0x40f080 at 0x42b32f).
const SCENE_ENDING: i32 = 15;

/// **The configuration's window size**, `th11.cfg`'s `+0x1d`: 0 the whole screen and 1 to 3 a window of 640x480,
/// 960x720 and 1280x960 (0x446b60 through 0x446b79). The startup dialog writes it (0x447984 through 0x4479c8).
const CFG_WINDOWED: usize = 0x004c_3465;

/// **The input's flags, and the bit that says the keyboard is read through DirectInput** — set at the set-up where
/// the device at `[0x4c32a0]` was made (0x4456c8 through 0x445702) and tested by the read (0x4576d3); and the slot
/// of `IDirectInputDevice8::Unacquire`.
const G_INPUT_FLAGS: usize = 0x004c_3810;
const INPUT_THROUGH_DIRECTINPUT: u32 = 0x400;
const G_KEYBOARD_DEVICE: usize = 0x004c_32a0;
const DEVICE_UNACQUIRE: usize = 8;

/// **The title**, a `0x5b94`-byte object 0x439900 builds: which of its screens is up (the switch at 0x439a6e),
/// that screen's step, and the cursor.
const G_TITLE: usize = 0x004a_8ecc;
const TITLE_SCREEN: usize = 0x1c;
const TITLE_STEP: usize = 0x20;
const TITLE_CURSOR: usize = 0x24;
/// **The main menu**, screen 1 (0x43a0a0), eight items: a run, the Extra, practice, the replays and the player's
/// data among them.
const SCREEN_MAIN_MENU: i32 = 1;
const TITLE_ITEM_START: i32 = 0;
const TITLE_ITEM_EXTRA: i32 = 1;
const TITLE_ITEM_PRACTICE: i32 = 2;
const TITLE_ITEM_REPLAY: i32 = 3;
const TITLE_ITEM_PLAYER_DATA: i32 = 4;
/// **The partner's select**, screen 7 (0x43d0b0), whose cursor is the shot and whose decide asks for the run
/// (0x43d5fc); and the difficulty the Extra is, 4 (0x43a539).
const SCREEN_SHOT_SELECT: i32 = 7;
const DIFFICULTY_EXTRA: i32 = 4;
/// **The step of a screen that reads a key**, after the one that waits out the frames it ignores them for.
const TITLE_STEP_READING: i32 = 2;

/// **The game object**, `0x78` bytes `0x4202d0` builds for each stage: its flags — `0x10` from the pause menu's
/// opening, the game over and a run ended by its clear, which only the pause menu's resume takes off (0x42c885) —
/// and `+0x74`, the argument it was built with, `1` for a replay being played (0x420325).
const G_GAME: usize = 0x004a_8e88;
const GAME_FLAGS: usize = 0x60;
const GAME_PAUSED: u32 = 0x10;
/// And `0x4` while the stage's build runs on its thread (0x41f903, 0x41fdbe).
const GAME_BEING_BUILT: u32 = 0x4;
const GAME_REPLAYING: usize = 0x74;
/// **The frames the game object has run** (0x4207fd), and the flag a stage built inside the game object before it
/// is set up under, taken off at its thirtieth frame (0x420424, 0x42043a).
const GAME_FRAMES: usize = 0x14;
const GAME_FADING_IN: u32 = 0x800;
/// **The frame of such a game object its stage begins on**, and the `.std` kept from the stage before that the
/// frame-0 path tells it by (0x420424, 0x4205b1).
const GAME_BEGINS_A_FADE_AT: i32 = 0x1e;
const G_KEPT_STD: usize = 0x004a_8d5c;
/// **The stage's background, its `.std`**, `new(0x3158)` (0x4029c0), with its fade's flags at `+0x2ff0` and its
/// fade's timer from `+0x2ff4` — the count the frame before, the count, its float, the game's speed it steps by and
/// the timer's own flags (0x4051d0). A stage begun out of a fade has its `.std` put at `0x4` and a count of 60
/// (0x405215 through 0x40522f), the draw counts it down a frame at a time, and the calc steps no script while
/// `0x4` stands with the count at 30 or over (0x402b1c through 0x402b36) — so on the stage's first frame, the
/// fade's thirtieth, the run's background steps nothing.
const G_STD: usize = 0x004a_8d60;
const STD_FADE_FLAGS: usize = 0x2ff0;
const STD_IS_ON: u32 = 0x1;
const STD_FADING_OUT: u32 = 0x2;
const STD_FADING_IN: u32 = 0x4;
const STD_FADE_BEFORE: usize = 0x2ff4;
const STD_FADE_COUNT: usize = 0x2ff8;
const STD_FADE_AS_A_FLOAT: usize = 0x2ffc;
const STD_FADE_STEPS_BY: usize = 0x3000;
const STD_FADE_TIMER_FLAGS: usize = 0x3004;
const TIMER_IS_SET_UP: u32 = 0x1;
/// The count the run's fade stands at as its stage's first frame begins: 60 less the thirty frames it has drawn.
const STD_FADE_AT_THE_STAGES_FIRST_FRAME: i32 = 30;

/// **The pause object**, which runs the pause menu, the end of a run and its name entries: its state at `+0x4`,
/// the pause menu 1 to 10, a run lost 12 to 19 and a run won 20 to 26, and 27 once either has carried its item out
/// (0x42bf30, the table at 0x42bfb4).
const G_PAUSE: usize = 0x004a_8eb0;
const PAUSE_STATE: usize = 0x4;
/// **The pause menu's resume**, `0x42c880`, which takes nothing: the game object's pause flag taken off, the music
/// let go on and the game's speed put back (0x42c880 through 0x42c8cd).
const UNPAUSE: usize = 0x0042_c880;
handed_over!(UNPAUSE_AT, the_unpause, set_the_unpause, UNPAUSE);
const PAUSE_THE_RUN_IS_OVER: std::ops::RangeInclusive<i32> = 12..=26;
/// **The menus a run ends on and the states that carry their items out**: a run lost's at 14, its item 1 the title,
/// carried out at 19; a run won's at 22, its item 0 the title, carried out at 26 — each twelve frames after its timer
/// starts over, the timer at `+0x10` whose count is `+0x14`. The cursor is `+0x38`.
const PAUSE_A_LOST_RUN_ENDS_ON_A_MENU: i32 = 14;
const PAUSE_A_WON_RUN_ENDS_ON_A_MENU: i32 = 22;
const PAUSE_LEAVING_A_LOST_RUN: i32 = 19;
const PAUSE_LEAVING_A_WON_RUN: i32 = 26;
const PAUSE_CURSOR: usize = 0x38;
const PAUSE_FRAMES: usize = 0x14;
const PAUSE_A_LOST_RUNS_TITLE: i32 = 1;
const PAUSE_A_WON_RUNS_TITLE: i32 = 0;
/// **The name entries**, the replay's at 10, 17 and 24 and the ranking's at 18 and 25, over the same fields: the
/// name, eight characters at `+0x2cc`; how many are typed at `+0x1f0`; and the grid's place at `+0x110`, whose
/// place `0x5a` ends the entry (`+0x118 = 0x5b` places, the last three a space, back and the end).
const PAUSE_NAMES: [i32; 5] = [10, 17, 18, 24, 25];
const PAUSE_NAME: usize = 0x2cc;
const PAUSE_NAME_BYTES: usize = 8;
const PAUSE_NAME_TYPED: usize = 0x1f0;
const PAUSE_NAME_GRID: usize = 0x110;
const PAUSE_THE_END_CELL: i32 = 0x5a;
/// **And the clear's own name entry, which is the title's screen 14** (0x43f6b0) after the ending: the name at
/// `+0x58d4`, how many are typed at `+0x58e0` and the grid's place at `+0x58ec`, read in its second step over the
/// same grid and end place `0x5a` — whose decide copies the name into the ranking and puts the screen's third step
/// up, which five frames on is screen 15.
const SCREEN_NAME_ENTRY: i32 = 14;
const TITLE_NAME: usize = 0x58d4;
const TITLE_NAME_TYPED: usize = 0x58e0;
const TITLE_NAME_GRID: usize = 0x58ec;
/// **Whether the clear is ranked at all**, nought where it is: the first step sets it to 1 for a score the ranking
/// has no place for, and the second then takes a decide straight to the third with no name typed.
const TITLE_NAME_NOT_ASKED: usize = 0x58e4;
/// **The replay's slots after a clear**, the title's screen 15 (0x43ffa0), and the step its cancel takes, which
/// six frames on puts the main menu up (`FUN_004390e0(1)` past `+0x2b4 > 5`); and the frames the step setter
/// 0x439140 puts back as it sets one, the timer from `+0x2b0`.
const SCREEN_REPLAYS_SLOTS: i32 = 15;
const SLOTS_STEP_CANCELLED: i32 = 4;
const TITLE_STEPS_FRAMES: usize = 0x2b4;

/// **Where the entry up keeps the name, how many are typed and the grid's place**, which are the same three
/// fields at the pause object's places or the title's.
struct NameEntry {
    name: usize,
    typed: usize,
    grid: usize,
}

/// **The run's numbers**, a block of `0x90` bytes from `0x4a56e0`: the score over ten, the power, the character,
/// the partner, the lives, the difficulty with the Extra as 4, the stage from 1 with the Extra as 7, the stage the
/// run started at, and the frames since the stage began.
const G_SCORE: usize = 0x004a_56e4;
const G_POWER: usize = 0x004a_56e8;
const G_CHARACTER: usize = 0x004a_5710;
const G_SHOT_TYPE: usize = 0x004a_5714;
const G_LIVES: usize = 0x004a_5718;
const G_DIFFICULTY: usize = 0x004a_5720;
const G_STAGE: usize = 0x004a_5728;
const G_STARTING_STAGE: usize = 0x004a_572c;
const G_STAGE_FRAMES: usize = 0x004a_5734;
/// **The most power the shot type holds and what a bomb costs**, both out of its `.sht` (0x42fa66 through
/// 0x42fa7c): there is no stock of bombs, a bomb being power.
const G_POWER_AT_MOST: usize = 0x004a_5748;
const G_BOMB_COSTS: usize = 0x004a_574c;
/// **The stage's record of what it loads**, which the build reads the stage's files through: `0x4a3828` on by
/// `0x40` a stage from stage 0, which the partner's select writes as it asks for a run (0x43d5b3 through 0x43d5ec).
const G_STAGE_LOADS: usize = 0x004a_8ec8;
const STAGE_LOADS: usize = 0x004a_3828;
const STAGE_LOADS_BYTES: usize = 0x40;
/// **The run's flags**: `0x10` a practice run (0x43a57e), `0x20` the attract demo (0x439aa3).
const G_RUN_FLAGS: usize = 0x004a_5758;
const RUN_IS_A_PRACTICE: u32 = 0x10;
const RUN_IS_THE_DEMO: u32 = 0x20;
/// How many stages there are to count, the Extra the seventh.
const STAGES: i32 = 7;
const DIFFICULTIES: i32 = 5;
/// **The rest of the run's numbers a stage's replay record keeps** — see [`resume::Numbers`]: the point items'
/// value, the pieces of the next life, the word th10 keeps at `0x474c98`, the continues used, the extends the score
/// has given, and the graze.
const G_POINT_VALUE: usize = 0x004a_56f0;
const G_LIFE_PIECES: usize = 0x004a_571c;
const G_4A5744: usize = 0x004a_5744;
const G_CONTINUES: usize = 0x004a_573c;
const G_EXTENDS: usize = 0x004a_5750;
const G_GRAZE: usize = 0x004a_5754;

/// **The score object**, `0x2e21c` bytes 0x437800 builds at the startup: seven records of `0x68d4` from `+0x8`, one
/// a shot type and the seventh every shot type's, each with its 175 cards of `0x90` from `+0x664` — the name in the
/// first `0x80`, the captures and the attempts after it, each held at 99,999 (0x40c650, 0x40cd60).
const G_SCORE_OBJECT: usize = 0x004a_8ebc;
const RECORDS_AT: usize = 0x8;
const SCORE_RECORD_BYTES: usize = 0x68d4;
const RECORDS: usize = 7;
const RECORD_OF_EVERY_SHOT_TYPE: usize = 6;
const SHOT_TYPES_A_CHARACTER: i32 = 3;
const CARDS_AT: usize = 0x664;
const CARDS: usize = 175;
const CARD_RECORD_BYTES: usize = 0x90;
const CARD_NAME_BYTES: usize = 0x80;
const CARD_CAPTURES: usize = 0x80;
const CARD_ATTEMPTS: usize = 0x84;
const A_CARDS_COUNT_STOPS_AT: u32 = 99_999;
/// **Each shot type's practice**, th10's entries of 8 bytes, `difficulty * 6 + stage` with the stage counted from 1,
/// from `+0x59c`: at `+0x5` whether it is offered, set as a run wins the stage (0x41eb61) and read by the practice
/// select (0x43d80a).
const PRACTICE_AT: usize = 0x59c;
const PRACTICE_BYTES: usize = 8;
const PRACTICES: usize = 4 * 6 + 1;
const PRACTICE_OFFERED: usize = 0x5;
/// **The settings and the endings seen** at `+0x2ddd4`, the file's `ST` record (0x437766) — see [`score_file`].
const SETTINGS_AT: usize = 0x2_ddd4;
/// **The endings seen inside them**, a byte an ending from `+0x2ddea` — `character * 3 + partner`, and six more for
/// a run with no continue — that the ending's set-up marks (0x40edb5 through 0x40edfa), the main menu lighting its
/// Extra from the second six (0x439350).
const CLEARS: std::ops::Range<usize> = 0x2_ddea..0x2_ddf6;
/// **A shot type's record made empty**, entered with the record in `eax` (called for each of the seven at 0x437793
/// through 0x4377bc): zeroed, headed, the ranking's defaults and each card numbered.
const A_RECORD_EMPTIED: usize = 0x0043_7550;
handed_over!(
    A_RECORD_EMPTIED_AT,
    a_record_emptied,
    set_a_record_emptied,
    A_RECORD_EMPTIED
);
/// **The replay**, `0x2dc` bytes 0x435e00 builds, whose `+0x10` is 1 where it is played back and 0 where it is
/// recorded (0x4356e0) — what a card's start asks before it counts the attempt.
const G_REPLAY_MANAGER: usize = 0x004a_8eb8;
const REPLAY_MANAGER_PLAYS_BACK: usize = 0x10;

/// **The generator**, a word `0x458bc0` turns over, seeded once from `timeGetTime` as the supervisor's job is added
/// (0x429582) — and the count of its draws beside it.
const G_RANDOM: usize = 0x004c_2f00;
const G_RANDOM_DRAWS: usize = 0x004c_2f04;

/// **The enemy manager**, `0x78` bytes 0x411070 builds with the stage's `.ecl`: the boss the fight is with in its
/// first slot, the list of enemies and how many are on it.
const G_ENEMY_MANAGER: usize = 0x004a_8d7c;
const ENEMY_BOSS: usize = 0x1c;
const ENEMY_LIST: usize = 0x68;
const ENEMY_COUNT: usize = 0x70;
/// An element of the list: the enemy, the next element and the one before (0x4111c0).
const ELEMENT_NEXT: usize = 0x4;
/// **An enemy**, `0x2678` bytes: the time of its main script as a float, where it is, the size of the body the
/// player is tested against as a circle of half of it (0x411e2c through 0x411e48), its life, the frames its body is
/// not tested for, and its flags.
const ENEMY_SCRIPT_TIME: usize = 0x8;
/// **The instruction the main script stands on**, its context's `+0x4` (0x45b860 keeping the context in `edi`), and an
/// instruction's opcode its `+0x4` (`movsx eax,WORD PTR [esi+0x4]` at 0x45b8a2).
const ENEMY_SCRIPT_AT: usize = 0xc;
const INSTRUCTION_OPCODE: usize = 0x4;
/// **The instructions that hold a script where they stand**, by returning -1 to the runner (0x45d275): `0x154` while
/// the enemy manager's boss slot holds one (0x415b1e through 0x415b30), and `0x153` while `[[0x4a8d84]+0x4438]` is
/// set and `0x410720` answers nought (0x415af6 through 0x415b19).
const WAITS: [u16; 2] = [0x153, 0x154];
/// **The enemy's contexts past its main one**, the list `callAsync` links each into after the head at `+0x1030`
/// (0x45d7a0), its first element at the head's next.
const ENEMY_ASYNC_CONTEXTS: usize = 0x1034;
const ENEMY_POSITION: usize = 0x1070;
const ENEMY_BODY: usize = 0x1114;
/// **The frames its current attack has run**, th10's `+0x115c` here: the count of the timer whose frame before is at
/// `+0x1190`, a frame on at the end of every update of the enemy's (0x41278c through 0x412793), and back at nought at
/// each change of attack: 0x417b40, which the update asks for the script of the enemy's next attack, sets it there
/// both where the life is down to the next attack's mark (0x417bf8) and where the attack's time has run out against
/// it (0x417c84, 0x417cf2) — the count the panel's seconds are that time less.
const ENEMY_ATTACK_FRAMES: usize = 0x1194;
const ENEMY_LIFE: usize = 0x252c;
const ENEMY_BODY_OFF_FOR: usize = 0x25ac;
const ENEMY_FLAGS: usize = 0x25bc;
/// **The two bits that skip the body's test**, `test al,0x22` at 0x411df9 — and what tells the stage's own scripts
/// from the enemies they spawn, which have a body to shoot.
const ENEMY_NO_BODY: u32 = 0x22;
/// **How many enemies a walk of the list goes through at most**: a next pointer read wrong is a walk that never
/// ends otherwise.
const ENEMIES_AT_MOST: usize = 0x400;

/// **The spell card**, a `0x924`-byte object 0x40bcc0 builds: the card's number, its flags, bit 0 of which is a
/// card running (0x40c6cc on), and the count of its timer — the frames it has run, from nought at its start
/// (0x40c674) and a frame on at each of its calc job's while it runs (0x40c147).
const G_CARD: usize = 0x004a_8d6c;
const CARD_NUMBER: usize = 0x8dc;
const CARD_TIMER: usize = 0x88c;
const CARD_FLAGS: usize = 0x8e0;
const CARD_IS_UP: u32 = 0x1;

/// **The item manager**, `0x265e70` bytes 0x4231e0 allocates: its items from `+0x14`, `0x478` bytes each, of which
/// the first `0x96` take every type but the small one fire leaves.
const G_ITEM_MANAGER: usize = 0x004a_8e90;
const ITEMS_AT: usize = 0x14;
const ITEM_BYTES: usize = 0x478;
const ITEM_SLOTS: usize = 0x96;
/// Where it is, its state and its type.
const ITEM_POSITION: usize = 0x434;
const ITEM_STATE: usize = 0x464;
const ITEM_TYPE: usize = 0x468;
/// **The slot the next small item fire leaves goes in**, in the `0x800` after those (0x424230's first half).
const SMALL_ITEMS_RING_AT: usize = 0x265e68;

/// **The object a stage's conversations are run from**, whose `+0x4438` is the conversation running and nought
/// where none is (0x415af6, 0x431e90).
const G_PANEL: usize = 0x004a_8d84;
const PANEL_CONVERSATION: usize = 0x4438;

/// **The bomb**, whose `+0x3c` is one going off — what the player's own bomb asks to be nought before it lets one go
/// (0x40651b).
const G_BOMB: usize = 0x004a_8d64;
const BOMB_GOING_OFF: usize = 0x3c;

/// **The play area in the game's own 640x480 output**, th10's: the field `384` wide from 32 across and `448` tall
/// from 16 down.
const PLAY_AREA: Rect = Rect {
    left: 32.0,
    top: 16.0,
    width: 384.0,
    height: 448.0,
};
const BACK_BUFFER: (u32, u32) = (640, 480);

impl Th11 {
    /// **The buttons held in `buttons` — a bit per pad button — as the bits the mapping makes them**.
    ///
    /// # Safety
    /// Must run on the game's main thread, the mapping being read out of the game's memory.
    unsafe fn the_mapped_buttons(&self, buttons: u32) -> u16 {
        let mapping = unsafe { mem::read::<[i16; 9]>(G_PAD_MAPPING) };
        PAD_MAPPED
            .iter()
            .filter(|(at, _)| {
                u32::try_from(mapping[*at])
                    .ok()
                    .and_then(|button| 1u32.checked_shl(button))
                    .is_some_and(|bit| buttons & bit != 0)
            })
            .fold(0, |word, (_, bit)| word | bit)
    }

    /// **One read of the pad DirectInput found**, and `None` where the input's flags say there is none.
    ///
    /// # Safety
    /// Must run on the game's main thread.
    unsafe fn the_pads_own_state(&self) -> Option<crate::joystick::DeviceState> {
        let (device, _) = unsafe { self.its_pad_device() }?;
        unsafe { crate::joystick::the_state_of(device) }
    }

    /// **The word the read makes of the pad DirectInput found**: the buttons by the mapping and the two axes past
    /// the thresholds the configuration holds (0x4574a9 through 0x457582), and the hat where `hats`.
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

    /// **The word the read makes of a pad orb sampled**, which is the read's winmm half (0x4572bf through 0x4573e8):
    /// the buttons by the mapping and the axes past a quarter of their travel, and the hat where `hats`.
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
        Self::object(G_TITLE)
    }

    /// The object behind a pointer of the game's own, where there is one.
    fn object(at: usize) -> Option<usize> {
        mem::read_committed::<usize>(at).filter(|it| *it != 0)
    }

    /// **The record of `thbgm.fmt` the stream is reading**: the manager's stream, its source and the record the
    /// source was last put on (0x45b31d).
    fn the_record_playing() -> Option<usize> {
        let stream = Self::object(G_SOUND_MANAGER + MUSIC_STREAM)?;
        Self::object(Self::object(stream + STREAM_SOURCE)? + SOURCE_RECORD)
    }

    /// **The boss the fight is with**, off the manager's first slot — whichever enemy last ran 332 there (0x414ed0,
    /// 0x414ee0), until its death puts nought back (0x4114bc).
    ///
    /// **Which is a boss between her fights as well, and not left over**: stage 3's boss comes back on the frame after
    /// its midboss is beaten and stays up through the waves, at `0,128` with 100,000 life and `0x10` in her flags — on
    /// stage 3 practised alone (`20261010-140418-practice-stage3`, 2026-10-10) from frame 5779 to her fight at 8333,
    /// where she reads 1,339 and the bit is off — so the slot holds her the whole way, as the game has her.
    fn the_boss_enemy(&self) -> Option<usize> {
        Self::object(Self::object(G_ENEMY_MANAGER)? + ENEMY_BOSS)
    }

    /// **How many contexts `callAsync` has the enemy running, and the latest time any of them reads** — the list at
    /// [`ENEMY_ASYNC_CONTEXTS`], each element a context and the next (0x45d7f7 through 0x45d81b).
    fn the_async_contexts_times(enemy: usize) -> (usize, f64) {
        let (mut count, mut latest) = (0, f64::NAN);
        let mut element = Self::object(enemy + ENEMY_ASYNC_CONTEXTS);
        while let Some(at) = element
            && count < ENEMIES_AT_MOST
        {
            if let Some(time) = Self::object(at).and_then(mem::read_committed::<f32>) {
                latest = latest.max(f64::from(time));
            }
            count += 1;
            element = Self::object(at + ELEMENT_NEXT);
        }
        (count, latest)
    }

    /// **Whether any of the stage's own scripts still alive stands on an instruction that waits** — see
    /// [`SCRIPT_CLOCK`]. Read between frames, a script on one is a script it held: one that does not wait is run
    /// past in the frame it is reached on.
    fn the_stage_scripts_are_held(&self, scripts: &[usize]) -> bool {
        self.the_enemies()
            .into_iter()
            .filter(|enemy| scripts.contains(enemy))
            .filter_map(|enemy| Self::object(enemy + ENEMY_SCRIPT_AT))
            .filter_map(|at| mem::read_committed::<u16>(at + INSTRUCTION_OPCODE))
            .any(|opcode| WAITS.contains(&opcode))
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

    /// **The stage's own scripts**: the enemies on the list with no body — `main`, and on a stage where `main`
    /// does nothing but loop, the enemy it spawns to run the stage — taken on the stage's first frame.
    fn the_stage_scripts(&self) -> Vec<usize> {
        self.the_enemies()
            .into_iter()
            .filter(|enemy| {
                mem::read_committed::<u32>(enemy + ENEMY_FLAGS)
                    .is_some_and(|flags| flags & ENEMY_NO_BODY != 0)
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
                |scripts| self.the_stage_scripts_are_held(scripts),
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

    /// The name entry up: one of the pause object's, or the title's after a clear.
    fn the_entry_a_name_is_typed_into(&self) -> Option<NameEntry> {
        let in_the_pause_object = Self::object(G_PAUSE).filter(|pause| {
            mem::read_committed::<i32>(pause + PAUSE_STATE)
                .is_some_and(|state| PAUSE_NAMES.contains(&state))
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
/// `character * 3 + partner`, or [`RECORD_OF_EVERY_SHOT_TYPE`] — at `+0x8` in steps of `0x68d4`, and the card's
/// `0x90` bytes from `+0x664` of it (0x43799e; 0x40c650).
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

impl Game for Th11 {
    /// **The seams this game's own frame is reached through**: the two walks, which put orb inside every frame; the
    /// frame, all three of the ones the pump picks between; and the pad's half of the read, which is what a recipe
    /// presses through.
    fn hooks(&self) -> Hooks {
        Hooks {
            update: Some(Patch {
                target: RUN_CALC_CHAIN,
                prologue: A_WALKS_PROLOGUE,
            }),
            draw: Some(Patch {
                target: RUN_DRAW_CHAIN,
                prologue: A_WALKS_PROLOGUE,
            }),
            walks: None,
            frame_entered: FrameEntered::ChainInEbx,
            runs_its_frame: None,
            render: Some(Patch {
                target: RENDER,
                prologue: RENDER_PROLOGUE,
            }),
            renders_too: OTHER_RENDERS,
            save_replay: None,
            stop_recording: None,
            stage_begun: None,
            stage_building: Some(Building::StdcallOfOne(Patch {
                target: REPLAY_BUILT,
                prologue: REPLAY_BUILT_PROLOGUE,
            })),
            unlocks_read: None,
            ranking_read: None,
            card_records_read: None,
            create_window: None,
            init_device: None,
            input: Some(Read::PadHalfInEcx(Patch {
                target: INPUT_PAD_HALF,
                prologue: INPUT_PAD_HALF_PROLOGUE,
            })),
            joystick: None,
            tops_up_the_music: None,
            stops_the_music: None,
            pad_buttons: None,
            // **The window size's 640x480 and the start button** of `DialogBoxParamA(hInstance, 0xcb, …)`, which WinMain
            // puts up where bit `0x100` of `th11.cfg`'s `+0x38` is set or Shift is held (0x445606 through 0x445626) — and
            // the configuration's defaults set the bit (0x420d8c). The procedure 0x447910 reads the radio buttons `0xcc`
            // to `0xcf` into the window size as `0xd0` is pressed (0x44793e through 0x4479c8), and its close sets the bit
            // WinMain leaves on (0x4479e3, 0x44562c).
            startup_dialog: Some(crate::game::Dialog {
                clicks: &[0xcd, 0xd0],
            }),
            makes_its_window: None,
            moves_its_window: None,
            shows_its_window: None,
            places_its_window: None,
            restyles_its_window: None,
            reads_a_pad: None,
            makes_its_renderer: None,
            // `HeapCreate`, `HeapAlloc`, `HeapReAlloc`, `HeapFree`, `VirtualAlloc` and `VirtualFree` are all in the
            // exe's own `KERNEL32.dll` imports, the CRT linked statically.
            takes_its_memory: Memory::Kernel32Imports,
            makes_its_threads: Threads::CreateThread,
            unloads_beside_its_frame: Some(Patch {
                target: UNLOAD,
                prologue: UNLOAD_PROLOGUE,
            }),
            decodes_through_one_window: Some(crate::game::OneWindow {
                decode: Patch {
                    target: DECODE,
                    prologue: DECODE_PROLOGUE,
                },
                encode: Patch {
                    target: ENCODE,
                    prologue: ENCODE_PROLOGUE,
                },
            }),
            opens_its_files: FileOpen::CreateFileA,
        }
    }

    /// **A run of this game is rewound**, with what a chapter is made of read: the stage's first frame after a
    /// build on a loader thread (0x41f8f0) — [`Th11::a_stage_has_just_been_built`] — and the replay's own seam for
    /// its seed ([`REPLAY_BUILT`]); the two lists a snapshot leaves out, [`Th11::audio_state`] and
    /// [`Th11::live_handles`]; and the numbers a stage starts from, field for field out of the replay's record
    /// ([`resume`]).
    fn rewinds(&self) -> bool {
        true
    }

    /// `scoreth11.dat` (`0x4957b8`), read through `CreateFileA`.
    fn score_files(&self) -> crate::score::Files {
        crate::score::Files {
            theirs: c"scoreth11.dat",
            ours: c"pointdevice_scoreth11.dat",
        }
    }

    unsafe fn read_state(&self) -> State {
        let (scene, wanted) = unsafe { self.scenes() };
        let flags = unsafe { mem::read::<u32>(G_RUN_FLAGS) };
        let demo = flags & RUN_IS_THE_DEMO != 0;
        let replay = self.a_replay_is_played() && !demo;
        // **Not the end of a run, which this game keeps inside the scene a run is played in**, as th10's does.
        let playing = scene == SCENE_RUN && !self.the_end_of_a_run_is_up();
        let boss = self.the_boss_enemy();
        let player_state = Self::object(G_PLAYER)
            .and_then(|player| mem::read_committed::<i32>(player + PLAYER_STATE));
        let power = unsafe { mem::read::<i32>(G_POWER) };
        let costs = unsafe { mem::read::<i32>(G_BOMB_COSTS) };
        let stage_frames = unsafe { mem::read::<u32>(G_STAGE_FRAMES) };
        let hit = player_state == Some(PLAYER_STATE_HIT);
        if playing && !replay && !demo {
            let mut kept = DEATHS_BY_FRAME
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let at = stage_frames as usize;
            if at == 0 {
                kept.clear();
            }
            // **A frame gone back to is the count that frame had**, th10's way of putting a chapter's deaths back.
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
            in_run: playing || scene == SCENE_NEXT_STAGE || scene == SCENE_CONTINUE,
            in_game: playing && !replay && !demo,
            in_ending: scene == SCENE_ENDING,
            ending_script: None,
            demo,
            replay,
            practice: flags & RUN_IS_A_PRACTICE != 0,
            paused: Self::object(G_GAME)
                .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
                .is_some_and(|it| it & GAME_PAUSED != 0),
            unsettled: playing && player_state.is_some_and(|it| it != PLAYER_STATE_NORMAL),
            bombing: Self::object(G_BOMB)
                .and_then(|bomb| mem::read_committed::<i32>(bomb + BOMB_GOING_OFF))
                .is_some_and(|it| it != 0),
            in_dialogue: self.a_conversation_is_on(),
            // Counted from nought, the way orb counts them: this game's own count is from 1.
            stage: unsafe { mem::read::<i32>(G_STAGE) } - 1,
            difficulty: unsafe { mem::read(G_DIFFICULTY) },
            stage_frames,
            script_frames: self.script_frames(match playing {
                true => unsafe { self.stage_frame() }.unwrap_or(0),
                false => 0,
            }),
            random_seed: unsafe { mem::read(G_RANDOM) },
            deaths: DEATHS.load(Ordering::Relaxed),
            lives: unsafe { mem::read::<i32>(G_LIVES) } as i8,
            // **The bombs a run has are its power over what one costs** — see [`G_BOMB_COSTS`].
            bombs: match costs {
                1.. => (power / costs).clamp(0, i32::from(i8::MAX)) as i8,
                _ => 0,
            },
            power: u16::try_from(power.max(0)).unwrap_or(u16::MAX),
            // **Less the stage's own scripts**, which are on the list for the whole stage with no body to shoot —
            // th10's reason.
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
            // **The card's own timer while one is up, and the boss's attack's between cards** — so the nonspell after
            // a card is a new attack, as th10's boss's clock makes it. **And not the boss's clock throughout**, which
            // th10 reads: the frames into a card that `orb_bot::cards` dodges by were measured on the card's timer.
            boss_attack_frames: match self.the_card_that_is_up() {
                Some(_) => Self::object(G_CARD)
                    .and_then(|card| mem::read_committed::<i32>(card + CARD_TIMER)),
                None => {
                    boss.and_then(|enemy| mem::read_committed::<i32>(enemy + ENEMY_ATTACK_FRAMES))
                }
            },
            spellcard: self.the_card_that_is_up(),
        }
    }

    /// **A hit counted inside the eight frames a bomb takes it back in**, and taken once the player is dying —
    /// th10's reading, held again here: back to playing through a bomb (0x43124f through 0x4312aa), or on to dying
    /// (0x4327d0).
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

    /// **Nothing, which is a device orb draws nothing through**: this one is Direct3D 9's — see
    /// [`Th11::draws_through_its_d3d_device`].
    unsafe fn d3d_device(&self) -> Device {
        Device::NULL
    }

    fn draws_through_its_d3d_device(&self) -> bool {
        false
    }

    unsafe fn direct3d_9_device(&self) -> Device {
        Self::object(G_D3D_DEVICE).map_or(Device::NULL, Device)
    }

    /// The last of the frame's quads, which is what the panel and the text over the field are drawn with.
    unsafe fn draw_what_is_queued(&self) {
        if let Some(sprites) = Self::object(G_SPRITES) {
            unsafe { with_esi(the_queues_draw(), sprites) };
        }
    }

    /// **The family the game's own text is asked for in**, th10's `ＭＳ ゴシック` — the face this engine asks
    /// `CreateFontA` for, its size the cell orb asks for.
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

    /// **The record of `thbgm.fmt` the stream is reading**, by where its track is in `thbgm.dat` and how long it is.
    fn music_identity(&self) -> Option<u32> {
        let record = Self::the_record_playing()?;
        let at = mem::read_committed::<u32>(record + RECORD_AT)?;
        let bytes = mem::read_committed::<u32>(record + RECORD_BYTES)?;
        Some(bytes ^ at.rotate_left(11))
    }

    /// **The name at the start of that record**, which is the file's own name for the track.
    fn the_track_heard(&self) -> Option<String> {
        let record = Self::the_record_playing()?;
        let name = name_at(record + RECORD_NAME, RECORD_NAME_BYTES)?;
        Some(String::from_utf8_lossy(&name).into_owned())
    }

    /// **Whether that record is the one the stage loaded as its fight's song**, slot 1 — and a play still in the
    /// manager's queue answers before the record does, th10's reasons both.
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

    fn audio_thread(&self) -> Option<u32> {
        mem::read_committed::<u32>(G_SOUND_MANAGER + SOUND_MUSIC_THREAD).filter(|it| *it != 0)
    }

    unsafe fn stop_music(&self) {}

    unsafe fn restart_stage_music(&self) -> bool {
        false
    }

    /// **The objects the sound is, which a restore leaves alone**: the manager, the wrapper it keeps DirectSound
    /// in, the stream and the holder of its buffer, and the source the stream reads through — see
    /// [`SOUND_DEVICE`] and the constants beside it.
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

    /// **What names an object a restore cannot copy**: every `.anm` file's textures and the sprites that point
    /// into them, the supervisor's three surfaces, and its twelve locks — see [`ANM_FILES_AT`] and the constants
    /// beside it.
    unsafe fn live_handles(&self) -> Vec<Range<usize>> {
        let mut ranges: Vec<Range<usize>> = vec![
            G_LOCKS..G_LOCKS + LOCKS_BYTES,
            G_SURFACES..G_SURFACES + SURFACES_BYTES,
        ];
        let Some(sprites) = Self::object(G_SPRITES) else {
            return ranges;
        };
        for slot in 0..ANM_FILES {
            let Some(file) = Self::object(sprites + ANM_FILES_AT + slot * 4) else {
                continue;
            };
            // **Bounded rather than believed**, th10's reason.
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

    fn a_snapshot_waits_for_its_music(&self) -> bool {
        false
    }

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

    /// **The window of 640x480**, size 1 of the configuration's three.
    unsafe fn force_windowed(&self) {
        unsafe { mem::write::<u8>(CFG_WINDOWED, 1) };
    }

    unsafe fn set_play_viewport(&self, _device: Device) {}

    /// **The chain the two walks are called on**, which this game allocates rather than keeping it as a static.
    fn chain(&self) -> *mut c_void {
        Self::object(G_CHAIN).unwrap_or(0) as *mut c_void
    }

    unsafe fn clears_back_buffer(&self) -> bool {
        false
    }

    /// **Nothing done**: the read acquires the DirectInput keyboard again on any failed `GetDeviceState`
    /// (0x457864 through 0x45787e), th10's shape.
    unsafe fn acquire_input(&self) -> bool {
        true
    }

    /// **The input flags' bit that says the keyboard is DirectInput's let go of, and the device unacquired**: the
    /// read takes `GetKeyboardState` (0x4576e8) wherever bit `0x400` of `[0x4c3810]` is clear (0x4576d3).
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

    /// **What the game's frame does between its limiter and its calc walk**, in its order: the queue drawn out,
    /// the camera set and its viewport, and the word after it (0x44649f through 0x4464d7).
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

    /// **The scene begun, and what the frame does inside it before its draw chain**: the queue put back at its
    /// start, the fog's dword forced and the fog set (0x44653e through 0x44656c).
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

    /// **And after the draw chain**: the queue drawn out, the texture unbound, and the scene ended (0x44657c
    /// through 0x4465a1).
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
                function: plays_no_sounds as *const () as usize,
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

    /// **The player's own countdown put back up**, the one every test asks is nought before it takes the hit
    /// (0x431ed4, 0x43223d).
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
        "the ranking of th11 is not read".to_owned()
    }

    unsafe fn leave_ranking(&self) {}

    /// **One more attempt at the card that is up, where the game's own card start counts one**: in the run's shot
    /// type's record and in the total's, each held at [`A_CARDS_COUNT_STOPS_AT`], and not in a replay (0x40c650) —
    /// told the way the card start tells it, by the replay's own word.
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

    /// **Every record's cards**, the seven records' `0x90`-byte entries for all 175 cards — see [`card_record`] —
    /// which is what a card's attempts and captures are counted into.
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

    /// **The counts put back and every name left as it stands**, the name being the first `0x80` bytes of an entry
    /// and copied in from the script at each card's start (0x40c650).
    unsafe fn set_captures_keeping_names(&self, saved: &[u8]) {
        unsafe { self.put_the_cards_back(saved, true) };
    }

    unsafe fn forget_captures(&self, _screen: *mut c_void) {}

    /// **The mode's own file read into the score object, but for what the front end offers**, th10's way: each
    /// shot type's record out of the file the mode points at — or made empty by the game's own code where that file
    /// holds none — and out of the game's own file the settings with the endings the Extra is lit from, and each
    /// practice stage's being offered. A stage reached is a stage reached whichever mode reached it, where a
    /// ranking, a card's count and a practice score are the mode's.
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
            // **Each one offered where the file offers it or memory does**, th10's reason: the game marks a stage
            // reached in memory, and writes it only as a run is torn down and as it leaves.
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
                None => unsafe { with_eax(a_record_emptied(), record) },
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
        // **And the endings the file holds with the ones memory does**, th10's reason: the ending's set-up marks
        // one after the run's teardown has written the file.
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
             the endings out of {own_name}"
        );
    }

    /// **Yes**: `scoreth11.dat` is read once, by the loading object as the game starts (0x428b28 into 0x437800),
    /// into the object every screen reads from memory — the main menu steps over its Extra on it (0x439350 at
    /// 0x43a0c3). Read through the mode's fork, the real case `the_extra_played_to_its_clear` (2026-10-07) found
    /// the menu's cursor going from item 0 to 2, the game's own file holding the clear's ending at `0x11` and orb's
    /// own none.
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

    /// **Both pads there are, as the read would have made them, with the hat as well** — th10's reasoning.
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

    /// **The hat and nothing else, where `hats`**: the read that makes the rest of a pad's word is the game's own
    /// and called through, and never looks at the hat.
    unsafe fn pad_word(&self, pad: Option<Reading>, hats: bool) -> u16 {
        match (pad, hats) {
            (Some(reading), true) => directions(crate::joystick::hat(reading.pov)),
            _ => 0,
        }
    }

    unsafe fn add_pad_buttons(&self, _into: *mut c_void, _pad: Option<Reading>) {}

    /// **The mapping read backwards**, th10's way.
    unsafe fn pushed_for(&self, word: u16) -> Option<crate::joystick::Pushed> {
        let mapping = unsafe { mem::read::<[i16; 9]>(G_PAD_MAPPING) };
        Some(crate::joystick::pushed_by(
            word,
            PAD_MAPPED.iter().map(|(at, bit)| (*bit, mapping[*at])),
            (PAD_UP, PAD_DOWN, PAD_LEFT, PAD_RIGHT),
        ))
    }

    unsafe fn its_pad_device(&self) -> Option<(usize, i32)> {
        if mem::read_committed::<u32>(G_INPUT_FLAGS)? & PAD_THROUGH_DIRECTINPUT == 0 {
            return None;
        }
        mem::read_committed::<usize>(G_PAD_DEVICE)
            .filter(|device| *device != 0)
            .map(|device| (device, PAD_RANGE))
    }

    /// **What the pause menu's own item for the title writes**: the scene wanted put at the title (`0x40dea0` with 4
    /// at 0x42d4ec), which the supervisor answers by taking the game down and building the title.
    unsafe fn leave_run(&self) -> bool {
        unsafe { mem::write::<i32>(G_SCENE_WANTED, SCENE_TITLE) };
        true
    }

    /// **The pause menu's item for starting over, taken**: the scene wanted at [`SCENE_START_OVER`] in a run still
    /// being played.
    unsafe fn asked_to_start_the_run_over(&self) -> bool {
        let (scene, wanted) = unsafe { self.scenes() };
        scene == SCENE_RUN && wanted == SCENE_START_OVER && !self.the_end_of_a_run_is_up()
    }

    /// **The scene the game is in wanted again, and the pause menu's own resume made** — th10's shape: the retry
    /// leaves the run paused for the build it asked for, and only the resume takes the pause off.
    unsafe fn keep_the_run_where_it_is(&self) {
        let (scene, wanted) = unsafe { self.scenes() };
        unsafe { mem::write::<i32>(G_SCENE_WANTED, scene) };
        let paused = Self::object(G_GAME)
            .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
            .is_some_and(|flags| flags & GAME_PAUSED != 0);
        if wanted == SCENE_START_OVER && paused {
            let unpause: extern "C" fn() = unsafe { std::mem::transmute(the_unpause()) };
            unpause();
        }
    }

    unsafe fn swallow_input(&self) {
        unsafe {
            mem::write::<u32>(G_INPUT, u32::MAX);
            mem::write::<u32>(G_INPUT_LAST, u32::MAX);
        }
    }

    unsafe fn press_into(&self, _state: *mut u8, _word: u16) {}

    fn menu_decide(&self) -> u16 {
        MENU_DECIDE
    }

    /// **Yes**, as th10's ending does: its script's lines wait on a press.
    fn an_ending_waits_for_a_press(&self) -> bool {
        true
    }

    fn menu_cancel(&self) -> u16 {
        MENU_CANCEL
    }

    unsafe fn the_name_its_own_entry_holds(&self) -> Option<String> {
        let entry = self.the_entry_a_name_is_typed_into()?;
        crate::game::the_name_held_at(entry.name, PAUSE_NAME_BYTES)
    }

    /// **Into the field the entry's own grid writes**, with the count typed at the full eight and the grid's place on
    /// the one that ends the entry — so the press after it leaves the entry the way somebody's would.
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

    /// **The end of a run's menu answered with its item for the title**, the way its own decide answers it: the
    /// pause object at the state that carries an item out, with its timer started over.
    unsafe fn skip_replay_prompt(&self) -> bool {
        // **And after a clear, the title's screen 15**, answered with its cancel's own step set the way the step
        // setter sets one.
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
        let (title, leaving) = match mem::read_committed::<i32>(pause + PAUSE_STATE) {
            Some(PAUSE_A_LOST_RUN_ENDS_ON_A_MENU) => {
                (PAUSE_A_LOST_RUNS_TITLE, PAUSE_LEAVING_A_LOST_RUN)
            }
            Some(PAUSE_A_WON_RUN_ENDS_ON_A_MENU) => {
                (PAUSE_A_WON_RUNS_TITLE, PAUSE_LEAVING_A_WON_RUN)
            }
            _ => return false,
        };
        unsafe {
            mem::write::<i32>(pause + PAUSE_CURSOR, title);
            mem::write::<i32>(pause + PAUSE_STATE, leaving);
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

    /// **Nothing while the stage is built or the stage before it fades out**, whose frames are none of this stage's.
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
    /// stage on (0x420424 through 0x420495, 0x4205b1), putting the stage's frames at nought, spawning `main` and
    /// taking the player's part of the replay's record. Asked at the top of the frame, before that update, which is
    /// where the playback puts a stage's numbers back as well.
    unsafe fn a_stage_has_just_been_built(&self) -> bool {
        let (scene, _) = unsafe { self.scenes() };
        let Some(game) = Self::object(G_GAME) else {
            return false;
        };
        let flags = mem::read_committed::<u32>(game + GAME_FLAGS).unwrap_or(GAME_BEING_BUILT);
        let frames = mem::read_committed::<i32>(game + GAME_FRAMES).unwrap_or(-1);
        let nothing_kept = Self::object(G_KEPT_STD).is_none();
        // **Once for each stage**, th10's reason: a frame orb holds before its update is the same frame asked about
        // again.
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

    /// **Nothing walked on the frame a build ended between orb's look and the walk**, th10's reason: the build is a
    /// thread of its own (0x41f8f0) and takes its flag off at 0x41fdbe whatever the main thread is doing, and a walk
    /// that finds it off runs the stage's first update on a frame [`Th11::a_stage_has_just_been_built`] read as still
    /// building, so with nothing put back.
    ///
    /// # Safety
    /// Must run on the game's main thread, after [`Th11::a_stage_has_just_been_built`] this frame.
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

    /// **The run the partner's select's cursor is on**: the difficulty and the character already where the two
    /// screens before it wrote them (0x43cb11, 0x43cfb6), and the partner under the cursor.
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
        "th11"
    }

    fn encode_resume(&self, saved: &crate::resume::Kept) -> Vec<u8> {
        resume::encode(saved)
    }

    fn decode_resume(&self, bytes: &[u8]) -> Option<crate::resume::Kept> {
        resume::decode(bytes)
    }

    /// **The stage's replay record, field for field** — see [`resume`]: read on the frame whose update is the
    /// stage's first, which is where the game's own record takes the player's part of it (0x436da0 from 0x420480
    /// and 0x4205bd) and after the build has taken the rest (0x4356e0, 0x436f30).
    ///
    /// **The point items' value as it stands and not over a hundred** as the record keeps it: the record rounds it
    /// down to tens of hundreds, and a value put back rounded is a run that scores differently from the one kept.
    unsafe fn run_state(&self) -> Vec<u8> {
        let player = Self::object(G_PLAYER).unwrap_or(0);
        let read = |at: usize| mem::read_committed::<i32>(at).unwrap_or(0);
        resume::encode_state(&resume::Numbers {
            score: unsafe { mem::read(G_SCORE) },
            power: unsafe { mem::read(G_POWER) },
            point_value: unsafe { mem::read(G_POINT_VALUE) },
            lives: unsafe { mem::read(G_LIVES) },
            life_pieces: unsafe { mem::read(G_LIFE_PIECES) },
            the_number_at_4a5744: unsafe { mem::read(G_4A5744) },
            continues: unsafe { mem::read(G_CONTINUES) },
            extends: unsafe { mem::read(G_EXTENDS) },
            graze: unsafe { mem::read(G_GRAZE) },
            seed: unsafe { mem::read(G_RANDOM) },
            at: (
                read(player + PLAYER_128THS),
                read(player + PLAYER_128THS + 4),
            ),
            focused: read(player + PLAYER_FOCUSED),
            element: read(player + PLAYER_ELEMENT),
            deaths: DEATHS.load(Ordering::Relaxed),
            began_out_of_a_fade: Self::object(G_GAME)
                .and_then(|game| mem::read_committed::<u32>(game + GAME_FLAGS))
                .is_some_and(|flags| flags & GAME_FADING_IN != 0),
            unhittable_for: read(player + PLAYER_UNHITTABLE_FOR),
            small_items_ring_at: Self::object(G_ITEM_MANAGER)
                .map_or(0, |manager| read(manager + SMALL_ITEMS_RING_AT)),
        })
    }

    /// **Put back the way the game's own playback puts a stage back** — the numbers as 0x4356e0's playback and
    /// 0x436f30's second half do, and the player's as 0x436da0's second half does: where it is in 128ths and as the
    /// floats 0x435630 makes of them, whether it is focused and the element.
    unsafe fn set_run_state(&self, state: &[u8]) {
        let Some(numbers) = resume::decode_state(state) else {
            return;
        };
        unsafe {
            mem::write(G_SCORE, numbers.score);
            mem::write(G_POWER, numbers.power);
            mem::write(G_POINT_VALUE, numbers.point_value);
            mem::write(G_LIVES, numbers.lives);
            mem::write(G_LIFE_PIECES, numbers.life_pieces);
            mem::write(G_4A5744, numbers.the_number_at_4a5744);
            mem::write(G_CONTINUES, numbers.continues);
            mem::write(G_EXTENDS, numbers.extends);
            mem::write(G_GRAZE, numbers.graze);
        }
        DEATHS.store(numbers.deaths, Ordering::Relaxed);
        let Some(player) = Self::object(G_PLAYER) else {
            return;
        };
        unsafe {
            mem::write(player + PLAYER_128THS, numbers.at.0);
            mem::write(player + PLAYER_128THS + 4, numbers.at.1);
            mem::write(player + PLAYER_POSITION, numbers.at.0 as f32 * A_128TH);
            mem::write(player + PLAYER_POSITION + 4, numbers.at.1 as f32 * A_128TH);
            mem::write(player + PLAYER_FOCUSED, numbers.focused);
            mem::write(player + PLAYER_ELEMENT, numbers.element);
            if numbers.element != 0 {
                mem::write::<i32>(player + PLAYER_OPTION_COUNT, 0);
                with_ebx(the_options_made(), player);
                with_ebx(the_options_made(), player);
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
                mem::write(manager + SMALL_ITEMS_RING_AT, numbers.small_items_ring_at);
            }
            // **The run's background as its fade left it on this frame**, so that it steps no script on it either
            // — see [`G_STD`]. Measured on a pick-up of stage 6 (2026-10-07): with the shot caught up, the
            // generator parted from the run at frame 6953 one draw ahead, everything else of the state agreeing.
            if numbers.began_out_of_a_fade
                && let Some(background) = Self::object(G_STD)
            {
                let flags = mem::read::<u32>(background + STD_FADE_TIMER_FLAGS);
                mem::write(background + STD_FADE_TIMER_FLAGS, flags | TIMER_IS_SET_UP);
                mem::write(background + STD_FADE_STEPS_BY, G_GAME_SPEED);
                mem::write(
                    background + STD_FADE_BEFORE,
                    STD_FADE_AT_THE_STAGES_FIRST_FRAME + 1,
                );
                mem::write(
                    background + STD_FADE_COUNT,
                    STD_FADE_AT_THE_STAGES_FIRST_FRAME,
                );
                mem::write(
                    background + STD_FADE_AS_A_FLOAT,
                    STD_FADE_AT_THE_STAGES_FIRST_FRAME as f32,
                );
                let fade = mem::read::<u32>(background + STD_FADE_FLAGS);
                mem::write(background + STD_FADE_FLAGS, fade | STD_FADING_IN);
            }
        }
        THE_RUN_BEGAN_OUT_OF_A_FADE.store(numbers.began_out_of_a_fade, Ordering::Relaxed);
    }

    /// **The update the run's stage spent firing before a pick-up's stage could**, run here: the shot's step and
    /// the shots' move, once, on the run's own first word — th10's.
    ///
    /// **Because the run's player was in its state 1 on that update and the pick-up's not yet**: a stage entered
    /// from the one before keeps the player, whose state the stage's first frame puts at 1 (0x42fec0), and the
    /// step fires only in that state (0x434389); a stage built fresh has a player made that build. Measured on a
    /// pair of stage 2 (fade-record.yaml and fade-pick-up.yaml, 2026-10-07): the pick-up's shot timer one behind the
    /// run's from frame 2, its score one over the run's at frame 202 and the generator parting at 221.
    ///
    /// **Only where the run's stage began out of a fade and the pick-up's timer is still held**, and under the
    /// conditions the player's update puts in front of the step (0x431af9 through 0x431b15): no conversation, an
    /// enemy up, and the panel not holding the shot.
    ///
    /// **And the background's fade ended where the run's draw ended it**: see [`G_STD`] for the fade the stage's
    /// first update held its script back under. The run's draw of that frame and the next counts the fade down and
    /// takes its `0x4` off (0x403348 through 0x40336d, 0x403835 through 0x403874), and a playback draws nothing — a
    /// pick-up of stage 2 (2026-10-07) held the count at 30 through all 2,520 updates of its playback, the
    /// background stepping no script the whole way, and one of stage 6 parted from its run on the generator the
    /// background draws from. So it is ended here, after the update the script was held back on, as the run's stood
    /// on its next frame: the jobs' bit on, the fade's two off and its count at nought. **And without the screen's
    /// fade the run's draw puts up there (0x40335b)**, an object that draw makes for the screen and nothing an update
    /// reads.
    unsafe fn starts_the_shot_a_stage_began_with(&self, word: u16) {
        if !THE_RUN_BEGAN_OUT_OF_A_FADE.swap(false, Ordering::Relaxed) {
            return;
        }
        if let Some(background) = Self::object(G_STD) {
            unsafe {
                let flags = mem::read::<u32>(background + STD_FADE_FLAGS);
                mem::write(
                    background + STD_FADE_FLAGS,
                    flags & !(STD_FADING_IN | STD_FADING_OUT) | STD_IS_ON,
                );
                mem::write::<i32>(background + STD_FADE_BEFORE, 1);
                mem::write::<i32>(background + STD_FADE_COUNT, 0);
                mem::write::<f32>(background + STD_FADE_AS_A_FLOAT, 0.0);
            }
        }
        let Some(player) = Self::object(G_PLAYER) else {
            return;
        };
        let enemies = Self::object(G_ENEMY_MANAGER)
            .and_then(|manager| mem::read_committed::<i32>(manager + ENEMY_COUNT))
            .unwrap_or(0);
        let panel = Self::object(G_PANEL);
        let talking = panel
            .and_then(|panel| mem::read_committed::<usize>(panel + PANEL_CONVERSATION))
            .unwrap_or(0);
        let held = panel
            .and_then(|panel| mem::read_committed::<u32>(panel + PANEL_FLAGS))
            .is_some_and(|flags| flags & PANEL_HOLDS_THE_SHOT != 0);
        let the_pick_ups = mem::read_committed::<i32>(player + PLAYER_SHOT_TIMER).unwrap_or(-1);
        if the_pick_ups >= 0 || enemies <= 0 || talking != 0 || held {
            return;
        }
        unsafe {
            let read = mem::read::<u32>(G_RUN_INPUT);
            mem::write::<u32>(G_RUN_INPUT, u32::from(word));
            with_eax(the_shot_step(), player);
            let moved: extern "stdcall" fn(usize) = std::mem::transmute(the_shots_moved());
            moved(player);
            mem::write(G_RUN_INPUT, read);
        }
        log!(
            "resume: the shot caught up to a stage begun out of a fade: its timer at {}",
            unsafe { mem::read::<i32>(player + PLAYER_SHOT_TIMER) },
        );
    }

    /// **The seed, where the game's own playback writes it**, with the draws counted from nought — the record's
    /// `+0x2` (0x4356e0, 0x436f30).
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
                "score={} power={} point_value={} lives={} life_pieces={} 4a5744={} continues={} extends={} \
                 graze={} seed={:#06x} at={},{} element={} deaths={}",
                numbers.score,
                numbers.power,
                numbers.point_value,
                numbers.lives,
                numbers.life_pieces,
                numbers.the_number_at_4a5744,
                numbers.continues,
                numbers.extends,
                numbers.graze,
                numbers.seed,
                numbers.at.0,
                numbers.at.1,
                numbers.element,
                numbers.deaths,
            ),
            None => "no numbers of a run of th11".to_owned(),
        }
    }

    /// **The frame a run has been asked for and nothing of it built**: the title up and the run wanted — the
    /// partner's select writes the scene wanted as 7 (0x43d5fc).
    unsafe fn run_chosen(&self) -> bool {
        let (scene, wanted) = unsafe { self.scenes() };
        scene == SCENE_TITLE && wanted == SCENE_RUN
    }

    /// **The stage, the stage the run started at, and the record the build reads the stage's files through**, which
    /// is what the partner's select writes for stage 1 or the Extra (0x43d5b3 through 0x43d5ec).
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

    /// **A run of these numbers, written where the title's own screens write them**: the difficulty (0x43cb11), the
    /// character (0x43cfb6), the partner (0x43d58d), the stage and its record ([`Th11::start_stage`]), the practice
    /// and the demo's bits taken off, and the scene wanted at a run (0x43d5fc).
    unsafe fn start_a_run(&self, run: &RunStart) -> bool {
        if !(0..STAGES).contains(&run.stage) || !(0..DIFFICULTIES).contains(&run.difficulty) {
            log!(
                "replay: there is no stage {} of difficulty {} in th11",
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

    /// **The end of a run, whichever way it ended**: the pause object's states 12 to 26, and the ending.
    unsafe fn run_finished(&self) -> bool {
        unsafe { self.scenes() }.0 == SCENE_ENDING || self.the_end_of_a_run_is_up()
    }

    unsafe fn reproduction(&self) -> String {
        let player = Self::object(G_PLAYER).unwrap_or(0);
        let read = |at: usize| mem::read_committed::<i32>(at).unwrap_or(0);
        format!(
            "frames={} seed={:#06x} draws={} score={} power={} lives={} at={},{}",
            unsafe { mem::read::<u32>(G_STAGE_FRAMES) },
            unsafe { mem::read::<u16>(G_RANDOM) },
            unsafe { mem::read::<u32>(G_RANDOM_DRAWS) },
            unsafe { mem::read::<u32>(G_SCORE) },
            unsafe { mem::read::<i32>(G_POWER) },
            unsafe { mem::read::<i32>(G_LIVES) },
            read(player + PLAYER_128THS),
            read(player + PLAYER_128THS + 4),
        )
    }

    /// **The player's clocks and the run's place**, as their fields are.
    unsafe fn the_world(&self) -> Option<crate::trace::TheWorld> {
        let player = Self::object(G_PLAYER)?;
        let int = |at: usize| f64::from(mem::read_committed::<i32>(at).unwrap_or(0));
        let float = |at: usize| f64::from(mem::read_committed::<f32>(at).unwrap_or(f32::NAN));
        // **Each enemy by where it is, its life, its main script's time and its flags**, which is what says which of
        // them are the stage's own scripts — the ones [`Th11::the_stage_scripts`] takes for the clock.
        let enemies = self
            .the_enemies()
            .into_iter()
            .map(|enemy| {
                let (asyncs, latest) = Self::the_async_contexts_times(enemy);
                vec![
                    float(enemy + ENEMY_POSITION),
                    float(enemy + ENEMY_POSITION + 4),
                    int(enemy + ENEMY_LIFE),
                    float(enemy + ENEMY_SCRIPT_TIME),
                    int(enemy + ENEMY_FLAGS),
                    Self::object(enemy + ENEMY_SCRIPT_AT)
                        .and_then(|at| mem::read_committed::<u16>(at + INSTRUCTION_OPCODE))
                        .map_or(f64::NAN, f64::from),
                    asyncs as f64,
                    latest,
                ]
            })
            .collect();
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
                    "draws".to_owned(),
                    f64::from(unsafe { mem::read::<u32>(G_RANDOM_DRAWS) }),
                ),
                (
                    "seed".to_owned(),
                    f64::from(unsafe { mem::read::<u16>(G_RANDOM) }),
                ),
                (
                    "game_frames".to_owned(),
                    Self::object(G_GAME).map_or(f64::NAN, |game| int(game + GAME_FRAMES)),
                ),
                // **And the background's fade**, which a stage begun out of the last one's runs through — see
                // [`G_STD`].
                (
                    "std_fade_flags".to_owned(),
                    Self::object(G_STD).map_or(f64::NAN, |std| int(std + STD_FADE_FLAGS)),
                ),
                (
                    "std_fade_count".to_owned(),
                    Self::object(G_STD).map_or(f64::NAN, |std| int(std + STD_FADE_COUNT)),
                ),
            ],
            tables: vec![("enemies".to_owned(), enemies)],
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

    /// **The frames the stage is built on**, which the loader thread sets bit 4 of the game object's flags for.
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

    /// **The clock goes on with each frame no script is held on, and stands on each frame one is**, as `main` held on
    /// its wait for a boss.
    #[test]
    fn the_clock_counts_the_frames_no_script_was_held_on() {
        let mut clock = ScriptClock::EMPTY;
        let read: Vec<i32> = [false, false, true, true, false]
            .iter()
            .enumerate()
            .map(|(frame, held)| clock.at(frame + 1, || vec![0x1000], |_| *held))
            .collect();
        assert_eq!(read, [1, 2, 2, 2, 3]);
    }

    /// **A frame gone back to reads what it read before, and the frames after it are counted again from there.**
    #[test]
    fn a_frame_gone_back_to_reads_its_count_and_the_clock_goes_on_from_it() {
        let mut clock = ScriptClock::EMPTY;
        for (frame, held) in [false, true, false, false].iter().enumerate() {
            clock.at(frame + 1, || vec![0x1000], |_| *held);
        }
        assert_eq!(clock.at(2, || vec![0x1000], |_| false), 1);
        assert_eq!(clock.at(3, || vec![0x1000], |_| false), 2);
    }

    /// **Frames run past unread are counted where no script was held as they were read**, as at four updates a frame.
    #[test]
    fn frames_run_past_unread_are_counted_where_no_script_was_held() {
        let mut clock = ScriptClock::EMPTY;
        clock.at(1, || vec![0x1000], |_| false);
        assert_eq!(clock.at(5, || vec![0x1000], |_| false), 5);
        assert_eq!(clock.at(9, || vec![0x1000], |_| true), 5);
    }
}
