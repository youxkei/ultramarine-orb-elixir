//! Chapter-based retry for 東方紅魔郷 1.02h, injected before the game starts.

// The one thing the linker has to say about this crate, and it is not a fault: `DllMain` is
// `extern "system"`, so mingw's linker looks for the decorated `_DllMain@12`, finds the
// undecorated symbol Rust exported, and resolves the one to the other. That fixup is what makes
// the DLL's entry point work, and there is no spelling of the export that avoids it.
#![allow(linker_messages)]

// `pub` on the ones a scenario in `tests/` drives, which is what an integration test needs of a
// crate: `chapter` and `resume` are what a pointdevice run *is*, and `retry_ui` is how a death gets
// to one. Nothing outside this crate and its own tests links it — the DLL exports `DllMain` and the
// four hooks and nothing else — so what these say is which parts have a scenario over them.
//
// The hooks below are `pub` for the same reason, and that is the whole of what a scenario needs of
// this file: a game laid out by hand calls them where the real game's own code would, and
// [`attach_to`] is how it is attached to with no process to patch.
pub mod chapter;
mod crash;
mod hook;
mod joystick;
mod lives_ui;
pub mod memtrack;
/// The list the three questions draw and the colours they draw it in.
///
/// `pub` for a scenario that reads the screen: which item a menu's cursor is on is a colour rather
/// than a position — `menu_ui::list` draws the chosen one in `SELECTED` and the rest in `NORMAL` —
/// so a test cannot say what a frame shows without naming those.
pub mod menu_ui;
pub mod mode_ui;
mod overlay;
mod pe;
/// A device that keeps what it was asked to draw, so a test can say what is on the screen. Beside
/// the drawing it stands in for rather than in `orb-core` with the vtable declarations, because its
/// `Screen` fixture builds a real `Overlay` — and that reaches `text.rs` and the GDI.
///
/// `pub` rather than `#[cfg(test)]` because a scenario in `tests/` needs one as well, and for the
/// same reason the drawing itself needs no seam: a device is a pointer to a vtable, so a game laid
/// out by hand hands orb one of these and orb's overlay is built and drawn through the calls it
/// makes against a real one.
pub mod recording;
pub mod resume;
mod resume_ui;
pub mod retry_ui;
pub mod score;
pub mod snapshot;
mod text;
mod threads;
mod tuning;
mod window;

use std::ffi::c_void;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering};

use orb_config::Config;

// What has gone behind the seam, under the names the rest of this crate already calls it by.
// `log` carries both the module and the macro of that name — they are in different namespaces —
// which is why every call site can say `use crate::{detail, log}` whether it lives here or in
// `orb-core`, and go on saying it as it moves between them.
pub(crate) use orb_core::{
    audio, d3d8, detail, frame, game, input, log, pacing, profile, summary, sync,
};
use windows_sys::Win32::Foundation::{BOOL, HANDLE, TRUE};
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_sys::Win32::System::Threading::GetCurrentProcessId;

/// The frame loop's state, made on the first ask.
///
/// Self-initialising rather than set up in `attach`, so that there is no order in which a caller
/// can reach it before it exists — it reads the counter's frequency when it is made, and a pacing
/// that does not know that yet can answer nothing.
///
/// # Safety
/// Must run on the game's main thread, and the caller must not already hold a reference this
/// returned.
#[allow(clippy::mut_from_ref)]
unsafe fn pacing() -> &'static mut frame::Pacing {
    unsafe { PACING.get() }.get_or_insert_with(frame::Pacing::new)
}

/// One brush stroke, as coverage: what the mark over the lives is drawn from, baked out of
/// `brush.png` by `build.rs` rather than kept here as four thousand numbers.
mod brush {
    include!(concat!(env!("OUT_DIR"), "/brush.rs"));
}

use chapter::{Cause, Chapters, Judgement};
use game::th06::Th06;
use game::{Game, Pad, RunStart, State};
use input::Keyboard;
use lives_ui::LivesMark;
use menu_ui::By;
use mode_ui::{Answer, Mode, ModeMenu};
use overlay::Overlay;
use retry_ui::{Choice, RetryMenu};
use sync::MainThread;

/// How often the state line goes to the log while a run is in progress.
const STATE_LOG_INTERVAL: u32 = 60;
/// How often the numbers on the HUD are refreshed. Often enough to watch, seldom enough
/// to read.
const HUD_NUMBER_INTERVAL: u32 = 30;

/// Em height in the game's 640x480 output.
const FONT_HEIGHT: i32 = 15;
/// And the size the one word on the mark over the lives is written at, which is bigger because it
/// is a word on a brush stroke rather than a line to be scanned.
const MARK_FONT_HEIGHT: i32 = 19;

/// How many frames to keep trying to build the overlay for.
///
/// More than one because what one failure costs is the whole run: no retry menu and none of the
/// three questions, each of them being skipped where there is no overlay. Not forever, because
/// the failure to expect is `font.ttf` missing, and that answer does not change however often it
/// is asked — so this is a handful of tries and then the line saying it is unavailable.
const OVERLAY_ATTEMPTS: u32 = 8;

/// `CHAIN_CALLBACK_RESULT_BREAK`: what `RunCalcChain` returns when a job asks it
/// to stop for this frame. Returning it instead of calling the original is how
/// the retry menu freezes the game while drawing carries on.
const CHAIN_BREAK: i32 = 1;
/// `th06::RenderResult`, which the loop the game calls this from expects back.
const RENDER_KEEP_RUNNING: i32 = 0;
const RENDER_EXIT_SUCCESS: i32 = 1;
const RENDER_EXIT_ERROR: i32 = 2;

/// `RunCalcChain`'s answers that mean the game wants to stop running.
const CHAIN_EXIT_SUCCESS: i32 = 0;
const CHAIN_EXIT_ERROR: i32 = -1;

/// A stop on the ending skip, in case an ending ever waits for something that
/// running the update alone will not deliver.
///
/// Above a whole ending, because the point is that no frame of one is drawn and this is a
/// limit per frame: the loop stops here, the frame it is in goes on to draw whatever the
/// ending is showing by then, and the next frame picks the skip up again. At 7200 — two
/// minutes of game time, which was taken for longer than any ending — stage 6's ending hit
/// it five times and put five frames of itself on the screen.
///
/// Fifteen minutes, against the 36,932 updates that ending took, staff roll included. An
/// update of it costs 13µs — 484ms for all 36,932 — so an ending that never ends is a
/// pause under a second rather than a game that never comes back.
const ENDING_SKIP_LIMIT: u32 = 60 * 60 * 15;

/// A stop on the trip through the ranking that writes what a run counted about spell cards: room for
/// the front end's wait on the item and the screen building itself, in updates run inside one frame
/// with nothing drawn.
const COMMIT_FRAME_LIMIT: u32 = 240;

/// How many times the stress mode restores the same chapter before letting the
/// run carry on. Without a limit it would rewind the first chapter for ever and
/// never reach a boss, which is where the paths worth exercising are.
const STRESS_PER_CHAPTER: u32 = 4;

/// Green rather than white, because the game itself flashes white — a bomb, a boss going
/// down — and a mark that means something orb decided should not look like something the
/// game did. Nothing in 紅魔郷 fills the play field with green.
const FLASH_COLOR: u32 = 0x0040_ff40;

/// The wash over the play field the moment a boundary is reached: how far it goes, how long
/// it stays at full before it starts to go, and how many drawn frames it lasts altogether.
///
/// It holds before it fades because a wash that starts fading at once reads as dim however
/// bright its first frame is.
#[derive(Clone, Copy)]
struct Flash {
    alpha: u32,
    hold: u32,
    frames: u32,
}

/// A judging pass, where the wash is the instrument the pass is run with: a boundary is a
/// frame among a stage's thousands, and what says one has been reached should not be a
/// number to read. Nobody is playing, so it can take the field for the sixth of a second it
/// has — one frame in twenty is all the frame underneath gets — and still be gone before the
/// next boundary.
const FLASH_JUDGING: Flash = Flash {
    alpha: 0xc0,
    hold: 5,
    frames: 16,
};

/// A run somebody is playing, where the same wash would be in the way rather than useful: a
/// chapter begins every few seconds through a fight, and the frame one lands on is a frame
/// being dodged on. So it says a chapter began — which is where dying now sends you back
/// to — through bullets that stay readable, and is gone inside a quarter of a second.
///
/// Dimmer and shorter than a judging pass's, but not by as much as it was: 0x40 held two
/// frames of ten went unnoticed in play, where attention is on the player and not on the
/// field. Somebody watching a pass is looking straight at the frame the wash is on.
const FLASH_PLAYING: Flash = Flash {
    alpha: 0x70,
    hold: 3,
    frames: 14,
};

/// This process's command line, which for the game is what the launcher wrote: the game's own
/// path and orb's options after it.
fn command_line() -> String {
    let mut wide = unsafe { GetCommandLineW() };
    if wide.is_null() {
        return String::new();
    }
    let mut characters = Vec::new();
    // Walked rather than measured first, since the only length there is to have is the
    // terminator's position.
    unsafe {
        while *wide != 0 {
            characters.push(*wide);
            wide = wide.add(1);
        }
    }
    String::from_utf16_lossy(&characters)
}

/// The keys pressed while a midstage table is being built, and the two that save and restore
/// by hand for exercising the snapshot engine.
///
/// Fixed rather than settings: whoever is building a table is the only person who presses any
/// of them, and a setting nobody changes is one more thing that can be wrong. The arrow keys
/// and the modifiers because stepping wants a hand that does not have to think; `a`, `s` and
/// `d` because they are under the other one.
mod keys {
    use orb_config::VirtualKey;
    use orb_config::keys::*;

    /// Puts a boundary at the frame the game is on, and writes both files.
    pub const ADD: VirtualKey = A;
    pub const WRITE: VirtualKey = D;
    /// Steps to the boundary either side of this frame, and holds or lets go.
    pub const NEXT: VirtualKey = RIGHT;
    pub const PREVIOUS: VirtualKey = LEFT;
    pub const HOLD: VirtualKey = SPACE;
    /// Judges the boundary the game is standing on, one step per press.
    pub const KEEP: VirtualKey = UP;
    pub const DROP: VirtualKey = DOWN;
    /// Held with a stepping key: across stages, or between the boundaries judged out.
    pub const ACROSS: VirtualKey = SHIFT;
    pub const DROPPED: VirtualKey = CTRL;
}

/// How long to wait for the game to build the stage a resume asked for before giving up on it.
///
/// Ten seconds of frames, against the second or so a stage takes to load: what this is a stop on
/// is the game having gone somewhere else with the run — back to its own menu, which is what a
/// stage it cannot load does.
const RESUME_START_FRAMES: u32 = 600;

/// How much longer than the frame it is aiming at a playback may run for, which is slack for a
/// clock that has stopped rather than room to reach anything: the game's own stage clock counts
/// every update of a stage, so one update is one frame of it.
const STALLED_FRAMES: u32 = 60;

/// A stop on a chapter step, for a target the run never reaches — a replay that
/// has ended, or a stage that has no further boundary.
///
/// Ten minutes of game time, which is longer than any stage: a step back replays a
/// stage from its start, so the limit has to be above a whole one, and an update is
/// tens of microseconds.
const STEP_LIMIT_FRAMES: u32 = 60 * 60 * 10;

struct Runtime {
    game: &'static dyn Game,
    config: Config,
    data: Range<usize>,
    frames: u32,
    previous: Option<State>,
    keyboard: Keyboard,
    /// Created on the first frame that has a Direct3D device.
    overlay: Option<Overlay>,
    /// Frames left to try building it on, so that a broken overlay is not retried for the whole
    /// run and a busy first frame is not the whole of the answer either.
    overlay_attempts: u32,
    /// The lag and the frame interval as the status line last showed them, in
    /// microseconds. Held between refreshes so the numbers can be read.
    shown: (i64, i64, i64),
    chapters: Chapters,
    /// How far the play cursor has run past the next write, at its best and worst
    /// over a reporting interval. Judging the music by ear needs someone listening
    /// at the moment it slips; these numbers do not.
    margin_worst: u32,
    margin_best: u32,
    /// Frames left to report the margin every frame, set by a restore so that what
    /// a restore does to the streaming is visible rather than averaged away.
    margin_trace: u32,
    /// The flash now running — its numbers and the drawn frames of it left — and the
    /// boundary it was started for.
    flash: Option<(Flash, u32)>,
    flashed: Option<(i32, u32)>,
    /// The stream the music was last seen playing through. The game replaces it
    /// when it changes track, which would leave a snapshot pointing at freed
    /// memory, so it is worth knowing exactly when that happens.
    stream: (usize, Option<u32>),
    /// Restores the stress mode has done in the chapter it is on.
    stressed: u32,
    stressing: (i32, u32),
    /// `Some` while the game is frozen on the retry menu.
    retry: Option<RetryMenu>,
    /// The mark over the game's own count of lives, in a run where dying costs a chapter
    /// rather than one of them.
    lives: LivesMark,
    /// Whether that mark is being drawn, as `before_draw` decided at the top of this frame: read
    /// again by the drawing itself, so that the row the game was asked to repaint and the row a
    /// mark goes over are the same row — and read by the next frame's decision, which is what
    /// keeps the mark on a panel the game is still painting after the run it belonged to has ended.
    marked: bool,
    /// How many frames it has been kept for on that second count, for the line that says so.
    marked_after: u32,
    /// Which of the two things a run is, and which of the two rankings is being looked at.
    mode: Mode,
    /// Whether anybody is there to be asked which. A pass over a replay is not, and a menu
    /// frozen on a question nobody answers is a pass that never ends.
    asks_mode: bool,
    /// `Some` while the game is frozen on that question.
    asking: Option<ModeMenu>,
    /// Whether the game is being held on a chapter boundary stepped to during a
    /// replay. Its update does not run while it is; the drawing carries on, which
    /// is what makes the frame the boundary falls on something to look at.
    held: bool,
    /// Whether the end of this stage has already held the run once. Once is enough:
    /// the hold says the stage is over and going on loses the way back into it, and
    /// after that the choice has been made.
    walled: bool,
    /// Whether the ending skip has reached the staff roll, which is where it stops and
    /// leaves the game to play the roll a frame at a time.
    rolling: bool,
    /// Whether a run was given up, the one way out of a run that writes nothing.
    given_up: bool,
    /// `Some` while the game is frozen on the question of where to start a run that has one left
    /// unfinished, holding the run that would be picked up.
    asking_resume: Option<(resume_ui::ResumeMenu, resume::Saved)>,
    /// The line drawn on the shot type select where the run under the cursor has a chapter written
    /// down. Kept across frames because it holds what it read, the screen being one somebody sits on.
    mark: resume_ui::Mark,
    /// What the question over the shot type select was answered with, waiting for the frame the run it
    /// was about is registered on. That frame is where a chapter can be put back — see
    /// [`Game::start_stage`] — and is also where the question would otherwise be asked a second time.
    started: Option<Started>,
    /// Which chapter this run's file describes — the stage and the frame it began on — so that it is
    /// written once per chapter rather than once per frame of one.
    kept: Option<(i32, u32)>,
    /// Frames spent waiting for the game to build the stage a resume asked for.
    starting: u32,
    /// Whether the game has been asked to read its keyboard the way it does without DirectInput,
    /// which is asked once and cannot be asked before the device exists.
    sent_keys: bool,
}

/// A run the resume question has been answered for, waiting for the game to get to it.
struct Started {
    /// Which run it was answered about. A run the game then registers that is not this one is a run
    /// nothing was answered about — the cursor moved between the answer and the press being handed
    /// over, which is not something the front end lets happen, but starting somebody else's chapter is
    /// not a thing to leave resting on that.
    slot: String,
    /// The chapter to put back, where that is what was asked for.
    saved: Option<resume::Saved>,
}

/// The one game orb knows how to run inside.
static GAME: Th06 = Th06;

static RUNTIME: MainThread<Option<Runtime>> = MainThread::new(None);
/// Everything the frame loop keeps between frames.
///
/// Beside the runtime rather than inside it, because the pacing is configured out of `DllMain` and
/// the runtime does not exist until the game has a device.
static PACING: MainThread<Option<frame::Pacing>> = MainThread::new(None);
/// Set while orb's own per-frame work is running.
///
/// Win32 calls that move a window dispatch messages synchronously, and the game
/// draws from its window proc, so a hook can be entered again from inside itself.
/// Nested entries run the game and nothing of ours: `Runtime` is handed out as
/// `&mut`, and two of those at once is not a thing that can be reasoned about.
static IN_HOOK: AtomicBool = AtomicBool::new(false);

static RUN_CALC_CHAIN: AtomicUsize = AtomicUsize::new(0);
static RUN_DRAW_CHAIN: AtomicUsize = AtomicUsize::new(0);
static UNLOCKS_READ: AtomicUsize = AtomicUsize::new(0);
static RANKING_READ: AtomicUsize = AtomicUsize::new(0);
static STAGE_BEGUN: AtomicUsize = AtomicUsize::new(0);
static STAGE_BUILDING: AtomicUsize = AtomicUsize::new(0);
static SAVE_REPLAY: AtomicUsize = AtomicUsize::new(0);
static STOP_RECORDING: AtomicUsize = AtomicUsize::new(0);
static CREATE_GAME_WINDOW: AtomicUsize = AtomicUsize::new(0);
static INIT_D3D_DEVICE: AtomicUsize = AtomicUsize::new(0);
static RENDER: AtomicUsize = AtomicUsize::new(0);
static GET_INPUT: AtomicUsize = AtomicUsize::new(0);
static GET_CONTROLLER_INPUT: AtomicUsize = AtomicUsize::new(0);
/// The game's chain functions, which orb has hooked, so calling them from its own
/// frame loop still runs everything orb does per frame.
static RUN_CALC_CHAIN_TARGET: AtomicUsize = AtomicUsize::new(0);
static RUN_DRAW_CHAIN_TARGET: AtomicUsize = AtomicUsize::new(0);
/// Whether the window-creation hook should override the game's display setting.
static FORCE_WINDOWED: AtomicBool = AtomicBool::new(false);
/// Whether the game was last given the keys, so the change can be logged once
/// rather than every frame.
static INPUT_ACTIVE: AtomicBool = AtomicBool::new(true);
/// Whether the keyboard has been out of reach since it was last read, and so needs
/// getting back before the next read.
static INPUT_LOST: AtomicBool = AtomicBool::new(false);
/// Whether the front end's own decide is being kept from the game — see [`held_back`]. Set for as
/// long as the screen orb has a question about is up with a run of a chapter under the cursor, and
/// left alone while the question itself is up: the frame that writes it ends in the question's own
/// branch and never reaches the write.
static HOLD_DECIDE: AtomicBool = AtomicBool::new(false);
/// Whether one of those presses has arrived and has not been asked about yet.
static DECIDE_PRESSED: AtomicBool = AtomicBool::new(false);
/// Whether it was down on the read before. What is held back is the press and not the holding, and
/// the game's own `g_LastFrameInput` cannot tell orb which this is: the bits were taken out of the
/// word it is assigned from.
static DECIDE_WAS_DOWN: AtomicBool = AtomicBool::new(false);
/// Whether the press held back is to be handed over on the next read, the question having been
/// answered with a run to start.
static FEED_DECIDE: AtomicBool = AtomicBool::new(false);
/// Whether the key a question was cancelled with is being kept from the game until it is let go. What
/// the screen underneath would do with it is go back — see [`Game::menu_cancel`].
static HOLD_CANCEL: AtomicBool = AtomicBool::new(false);
/// Whether the next read is the first after one of orb's questions came down, on which nothing the
/// game was not already holding is let through — see [`held_back`].
static SETTLE_KEYS: AtomicBool = AtomicBool::new(false);
/// The word the game was handed on the read before, which is what the game's own idea of the frame
/// before still says while a question of orb's is up: no frame of one runs its chain, so nothing
/// assigns `g_LastFrameInput`.
static LAST_WORD: AtomicU16 = AtomicU16::new(0);

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(_module: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => attach(),
        DLL_PROCESS_DETACH => {
            frame::release();
            log::close();
        }
        _ => {}
    }
    TRUE
}

/// Runs with the game still suspended and the loader lock held: no loader
/// calls, no game code, just patching bytes and allocating our own memory.
fn attach() {
    log::open();
    std::panic::set_hook(Box::new(|info| log::line(&format!("panic: {info}"))));
    crash::install();

    log!(
        "orb {} attached to pid {}",
        env!("CARGO_PKG_VERSION"),
        unsafe { GetCurrentProcessId() }
    );

    let exe = unsafe { GetModuleHandleW(std::ptr::null()) } as usize;
    log!("exe image base {exe:#010x}");
    let data = unsafe { pe::section(exe, b".data\0\0\0") };
    let Some(data) = data else {
        return log!("no .data section in the exe; orb is doing nothing this run");
    };
    log!(
        ".data {:#010x}..{:#010x} ({} bytes)",
        data.start,
        data.end,
        data.len()
    );
    unsafe { pacing() }.configure();

    let mut config = match orb_api::module::host_exe().map(|path| Config::load_beside(&path)) {
        Some(Ok(config)) => config,
        Some(Err(error)) => return log!("config: {error}; orb is doing nothing this run"),
        None => return log!("cannot locate the game exe; orb is doing nothing this run"),
    };
    // The rest comes off this process's own command line, which the launcher wrote when it
    // created the game: what belongs to building the midstage table or to looking into a fault
    // changes from one launch to the next, so it is said at the launch rather than kept in a
    // file. The launcher has read them once already and refused to start on anything it could
    // not; this is the same reading, and a second complaint if the game was started some other
    // way.
    let command_line = command_line();
    match orb_config::args::Options::from_command_line(&command_line) {
        Ok(options) => config.apply(&options),
        Err(error) => return log!("arguments: {error}; orb is doing nothing this run"),
    }
    log!(
        "config: game_dir={} log_level={} pacing_log={} compose_us={} self_check={} chapter_tuning={} \
         block_replay_save={} skip_ending={} screen={} during_replay={} \
         fast_clear={} speed={} stress_restore_frames={} chapters={} resume={} sent_keys={} \
         track_memory={} frame_hooks={} own_frame_loop={}",
        config.game_dir.display(),
        config.log_level,
        config.pacing_log,
        config.compose_us,
        config.self_check,
        config.chapter_tuning,
        config.block_replay_save,
        config.skip_ending,
        config.screen,
        config.during_replay,
        config.fast_clear,
        config.speed,
        config.stress_restore_frames,
        config.chapters,
        config.resume,
        config.sent_keys,
        config.track_memory,
        config.frame_hooks,
        config.own_frame_loop,
    );
    // Set after the line that says what it is, so the log always states the level it
    // is then written at.
    log::set_level(config.log_level);
    log::set_pacing(config.pacing_log);
    unsafe { pacing() }.pin_compose(config.compose_us);

    if config.track_memory {
        match unsafe { memtrack::install(exe) } {
            Ok(()) => log!("memory hooks installed"),
            Err(error) => return log!("memory hooks: {error}; orb is doing nothing this run"),
        }
        match unsafe { threads::install(exe) } {
            Ok(()) => log!("thread hook installed"),
            Err(error) => return log!("thread hook: {error}; orb is doing nothing this run"),
        }
    }

    // Only where the score file might have to be somewhere other than the game's own: with
    // `--no-chapters` nothing can rewind, so every run belongs in the game's own ranking and
    // there is nothing to fork. Loud rather than fatal if the import is not there, since what it
    // costs is which file scores land in and not a run that cannot be played.
    //
    // A clear installs it whichever way that is set, because there the fork is not what it is
    // for: the file that must not be written is whichever this run would write, and refusing the
    // write means being in the path of it.
    if config.chapters || config.fast_clear {
        if config.fast_clear {
            score::refuse_writes();
        }
        match unsafe { score::install(exe) } {
            Ok(()) if config.fast_clear => log!("score: no file is written this run"),
            Ok(()) => log!("score: score.dat is forked while orb is in pointdevice mode"),
            Err(error) => log!("score: {error}; the game will write its own score.dat"),
        }
    }

    let patches = GAME.hooks();
    // Remembered before the patches are consumed: orb's own frame loop calls the
    // game's chain functions at these addresses, which is what keeps its hooks in
    // the path.
    RUN_CALC_CHAIN_TARGET.store(patches.update.target, Ordering::Relaxed);
    RUN_DRAW_CHAIN_TARGET.store(patches.draw.target, Ordering::Relaxed);

    let mut hooks = Vec::new();
    if config.frame_hooks {
        hooks.push((
            "update",
            patches.update,
            hook::address(run_calc_chain as _),
            &RUN_CALC_CHAIN,
        ));
        hooks.push((
            "draw",
            patches.draw,
            hook::address(run_draw_chain as _),
            &RUN_DRAW_CHAIN,
        ));
    }
    match patches.render {
        Some(patch) if config.frame_hooks && config.own_frame_loop => {
            hooks.push(("frame loop", patch, hook::address(render as _), &RENDER));
        }
        _ => {}
    }
    match patches.input {
        // Two things want this one. Keeping the game updating with the window in the background
        // means the keys have to be dropped there, since the keyboard is read globally rather
        // than per window; and a run that can be picked up again is a run whose buttons are
        // written down here and handed back here.
        Some(patch) if config.frame_hooks => {
            hooks.push(("input", patch, hook::address(get_input as _), &GET_INPUT));
        }
        _ => {}
    }
    match patches.stage_begun {
        // Whatever mode the launch starts in, since the mode is answered per run: this is the
        // moment a stage's numbers are put in place, which is where they are read from and the
        // only place a resumed run's go back.
        Some(patch) if config.chapters && config.resume => {
            hooks.push((
                "stage start",
                patch,
                hook::address(stage_begun as _),
                &STAGE_BEGUN,
            ));
        }
        _ => {}
    }
    match patches.stage_building {
        // The other half of the same job: the numbers go in where the stage has just been built, and
        // the seed has to be in before it is.
        Some(patch) if config.chapters && config.resume => {
            hooks.push((
                "stage build",
                patch,
                hook::address(stage_building as _),
                &STAGE_BUILDING,
            ));
        }
        _ => {}
    }
    // Loud rather than fatal: what it costs is the frame paying for the read again, which is
    // what every run before this did.
    match unsafe { joystick::install(exe, GAME.joystick_calibration()) } {
        Ok(()) => log!("joystick: read on a thread of orb's, out of the game's frame"),
        Err(error) => log!("joystick: {error}; the read stays in the game's frame"),
    }
    match patches.joystick {
        // Only to time it, when the log is being read closely enough to want the split out
        // of `input`.
        Some(patch) if config.log_level >= orb_config::LogLevel::Verbose => {
            hooks.push((
                "joystick",
                patch,
                hook::address(get_controller_input as _),
                &GET_CONTROLLER_INPUT,
            ));
        }
        _ => {}
    }
    match patches.save_replay {
        Some(patch) if config.block_replay_save => {
            hooks.push((
                "replay save",
                patch,
                hook::address(save_replay as _),
                &SAVE_REPLAY,
            ));
        }
        _ => {}
    }
    // Not behind a setting: it takes nothing away from a run being recorded, and
    // without it moving between a replay's stages quietly damages the replay in
    // memory.
    if let Some(patch) = patches.stop_recording {
        hooks.push((
            "replay record end",
            patch,
            hook::address(stop_recording as _),
            &STOP_RECORDING,
        ));
    }
    // Always, because both of the answers are orb's: fullscreen is a borderless window covering
    // the monitor and a size is a window of that size, and either way it is a window rather than
    // the display taken exclusively — which is what leaves orb somewhere to draw the numbers
    // beside the game, and what the letterbox is presented into.
    match unsafe { window::install(exe, GAME.content_size(), config.screen) } {
        Ok(()) => log!("screen: window hooks installed"),
        Err(error) => log!("screen: {error}; the window is left as the game makes it"),
    }
    match patches.unlocks_read {
        // Only where a run can be rewound, which is the only way the mode is ever pointdevice and
        // so the only way any open goes anywhere but the game's own file. A clear does not need it:
        // there the score hook is in the path to refuse the write, which is decided before any of
        // this.
        Some(patch) if config.chapters => {
            hooks.push((
                "unlocks read",
                patch,
                hook::address(unlocks_read as _),
                &UNLOCKS_READ,
            ));
        }
        _ => {}
    }
    match patches.ranking_read {
        // Only where there are two score files to keep apart, which is where a run can be rewound.
        // With one file the game's own way of carrying the captures in memory is right, and orb
        // clearing them would be orb losing a record nothing else keeps.
        Some(patch) if config.chapters => {
            hooks.push((
                "ranking read",
                patch,
                hook::address(ranking_read as _),
                &RANKING_READ,
            ));
        }
        _ => {}
    }
    if let Some(patch) = patches.create_window {
        FORCE_WINDOWED.store(true, Ordering::Relaxed);
        hooks.push((
            "window creation",
            patch,
            hook::address(create_game_window as _),
            &CREATE_GAME_WINDOW,
        ));
    }
    if let Some(patch) = patches.init_device {
        hooks.push((
            "device init",
            patch,
            hook::address(init_d3d_device as _),
            &INIT_D3D_DEVICE,
        ));
    }
    for (name, patch, replacement, original) in hooks {
        match unsafe { hook::install(patch.target, patch.prologue, replacement) } {
            Ok(trampoline) => {
                original.store(trampoline, Ordering::Relaxed);
                log!("{name} hook installed, original at {trampoline:#010x}");
            }
            Err(error) => return log!("{name} hook: {error}; orb is doing nothing this run"),
        }
    }

    unsafe { attached(&GAME, config, data) };
}

/// The game's own functions orb's hooks call through, which in a real process are the trampolines
/// [`hook::install`] leaves behind and for a game that is not one are its own.
///
/// One per hook a game reaches orb through, which is what a frame of one is made of: its update and
/// its draw, the read every key it acts on passes through, the two moments a stage's numbers are put
/// in place, and the two reads of the score file orb has something to say about. Not the frame loop:
/// orb's own runs the game's `Present` and its sound by calling the exe's code at its own addresses,
/// so a game that is not a real process has nothing there to call — such a run is
/// `--no-frame-loop`, with the update and the draw still hooked, which is what these are.
pub struct Originals {
    pub update: extern "fastcall" fn(*mut c_void) -> i32,
    pub draw: extern "fastcall" fn(*mut c_void) -> i32,
    pub input: extern "system" fn() -> u16,
    pub stage_building: extern "C" fn(i32) -> i32,
    pub stage_begun: extern "C" fn(*mut c_void) -> i32,
    pub unlocks_read: extern "C" fn(*mut c_void) -> i32,
    pub ranking_read: extern "C" fn(*mut c_void) -> i32,
}

/// Attaches orb to a game that is not a real process: `originals` in place of the trampolines, and
/// then the same runtime [`attach`] leaves behind.
///
/// What [`attach`] does above this and a scenario cannot: read a `.data` section out of the PE, load
/// `orb.yaml` from beside an exe, and patch a call site in the game's code. A game laid out by hand
/// has none of those — it hands over its own memory's bounds, a `Config` written in the open, and its
/// own functions where the patches would have pointed.
///
/// # Safety
/// Must run on the thread the game's frames will run on, with a simulated Windows installed there,
/// and every function in `originals` must outlive the last frame orb's hooks are reached on.
pub unsafe fn attach_to(
    game: &'static dyn Game,
    config: Config,
    data: Range<usize>,
    originals: Originals,
) {
    log::open();
    log!(
        "orb {} attached to a game laid out in this process",
        env!("CARGO_PKG_VERSION")
    );
    log::set_level(config.log_level);
    log::set_pacing(config.pacing_log);
    // The flags a fresh process would have brought, since a game that is not one does not bring it: a
    // launch is the moment nothing is being held back, nobody has pressed anything, and the keyboard
    // has not been lost. Left standing, one game's would be the next game's opening state — a press
    // held back on the screen a game before ended on, and a question up over the one that follows.
    IN_HOOK.store(false, Ordering::Relaxed);
    INPUT_ACTIVE.store(true, Ordering::Relaxed);
    INPUT_LOST.store(false, Ordering::Relaxed);
    HOLD_DECIDE.store(false, Ordering::Relaxed);
    DECIDE_PRESSED.store(false, Ordering::Relaxed);
    DECIDE_WAS_DOWN.store(false, Ordering::Relaxed);
    FEED_DECIDE.store(false, Ordering::Relaxed);
    HOLD_CANCEL.store(false, Ordering::Relaxed);
    SETTLE_KEYS.store(false, Ordering::Relaxed);
    LAST_WORD.store(0, Ordering::Relaxed);
    // And nothing of a run written down, which is the one piece of that state a `Runtime` does not
    // hold: the record lives beside it, for the two hooks that reach it from inside the game's update.
    unsafe { resume::forget() };
    for (slot, original) in [
        (&RUN_CALC_CHAIN, originals.update as usize),
        (&RUN_DRAW_CHAIN, originals.draw as usize),
        (&GET_INPUT, originals.input as usize),
        (&STAGE_BUILDING, originals.stage_building as usize),
        (&STAGE_BEGUN, originals.stage_begun as usize),
        (&UNLOCKS_READ, originals.unlocks_read as usize),
        (&RANKING_READ, originals.ranking_read as usize),
    ] {
        slot.store(original, Ordering::Relaxed);
    }
    unsafe { attached(game, config, data) };
}

/// Takes the runtime down, which is what closing the game does to it.
///
/// For a scenario that plays one game and then another in the same process: a runtime left standing
/// would hold the first game's chapters and draw its overlay through a device that has gone. Dropped
/// here rather than left to `DllMain`'s own detach, where the process is going away and an overlay
/// released through a device Direct3D has already torn down is a fault on the way out.
///
/// # Safety
/// Must run on the game's main thread, with no frame in progress and the game it was attached to —
/// its memory and the device it draws through — still there.
pub unsafe fn detached() {
    unsafe { *RUNTIME.get() = None };
}

/// Which of the two a launch starts in and what that decides, and the runtime the hooks then find.
///
/// Apart from [`attach`] because it is the whole of what a game has to have done to it, with nothing
/// in it about a process: see [`attach_to`].
///
/// # Safety
/// Must run on the game's main thread, before any of orb's hooks are reached.
unsafe fn attached(game: &'static dyn Game, config: Config, data: Range<usize>) {
    // Which mode a launch starts in, before anybody has been asked: pointdevice, since that is
    // what orb is for, and normal where `--no-chapters` has left it nothing to be.
    let mode = if config.chapters {
        Mode::Pointdevice
    } else {
        Mode::Normal
    };
    // The question is only put to somebody who is there to answer it. A pass over a replay has
    // nobody at the keyboard, and neither has a clear: those take the mode they are given.
    let asks_mode =
        config.chapters && !config.during_replay && !config.chapter_tuning && !config.fast_clear;
    log!(
        "mode: {mode} to start with; {}",
        if asks_mode {
            "the menu asks which"
        } else {
            "nobody is asked"
        }
    );

    // Whether this launch keeps what it plays, and which runs an earlier one left unfinished. The
    // names only: which of them is offered depends on what is chosen at the character select, and
    // this is the line that says whether there was anything there to offer at all.
    resume::keep(config.chapters && config.resume && mode == Mode::Pointdevice);
    if config.resume {
        let left = resume::left(&config.base_dir);
        log!(
            "resume: {} run(s) left unfinished{}{}",
            left.len(),
            if left.is_empty() { "" } else { ": " },
            left.join(", "),
        );
    }
    // Which of the two rankings this run is headed for, said whether or not the hook that acts on it
    // went in: with nothing able to rewind, the mode is normal and this is the game's own file, which
    // is where it was anyway.
    score::fork(mode == Mode::Pointdevice);

    let tuning = config.chapter_tuning.then(|| config.base_dir.clone());
    let during_replay = config.during_replay;
    unsafe {
        *RUNTIME.get() = Some(Runtime {
            game,
            config,
            data,
            frames: 0,
            previous: None,
            keyboard: Keyboard::new(),
            overlay: None,
            overlay_attempts: OVERLAY_ATTEMPTS,
            shown: (0, 0, 0),
            chapters: Chapters::new(&GAME, tuning, during_replay),
            margin_worst: 0,
            margin_best: u32::MAX,
            margin_trace: 0,
            flash: None,
            flashed: None,
            stream: (0, None),
            stressed: 0,
            stressing: (-1, 0),
            retry: None,
            lives: LivesMark::new(),
            marked: false,
            marked_after: 0,
            mode,
            asks_mode,
            asking: None,
            held: false,
            walled: false,
            rolling: false,
            given_up: false,
            asking_resume: None,
            mark: resume_ui::Mark::new(),
            started: None,
            kept: None,
            starting: 0,
            sent_keys: false,
        });
    }
}

impl Runtime {
    /// Whether this run has chapters at all: the mode says so, and `--no-chapters` says not.
    fn chaptering(&self) -> bool {
        self.config.chapters && self.mode == Mode::Pointdevice
    }

    /// And whether what it presses is being written down, so that the chapter it is left in can be
    /// played again in a later launch.
    fn keeping(&self) -> bool {
        self.chaptering() && self.config.resume
    }
}

/// What the pad is doing, for a menu of orb's own.
///
/// Asked of the game, and read as *its* buttons, because the frames these menus are up are frames
/// the game's own input is not running on — so a pad does nothing on them unless orb does this.
/// Which of a game's ways of reaching a pad it is read through is the game's business too: see
/// `Game::pad`.
///
/// # Safety
/// Must run on the game's main thread.
unsafe fn pad(game: &dyn Game) -> Pad {
    unsafe { game.pad(joystick::reading()) }
}

/// Takes the answer to the mode question: what a run does, and which of the two files the
/// scores are kept in.
///
/// One answer for both, because they are one thing: the ranking of pointdevice runs is the file
/// pointdevice runs are written to. What the game has unlocked is not part of that answer — the
/// menu is lit from `score.dat` whichever mode is chosen, since a stage reached is a stage
/// reached.
fn choose(runtime: &mut Runtime, mode: Mode) {
    let was = std::mem::replace(&mut runtime.mode, mode);
    score::fork(mode == Mode::Pointdevice);
    resume::keep(runtime.keeping());
    // Nothing kept of a run that will not be rewound: a stage's snapshots are forty megabytes or
    // so, and normal mode is the game as it was. Its buttons go with them — a run that cannot be
    // rewound has nothing a chapter would be resumed into.
    if mode == Mode::Normal {
        runtime.chapters.forget();
        unsafe { resume::forget() };
    }
    log!("mode: {mode}, was {was}");
}

/// Takes the game through the screen a ranking is shown on, with nothing drawn, that being the one
/// place it writes the records its score file holds.
///
/// A run given up writes nothing, so what it counted about spell cards would go with the process. The
/// screen is asked for the way the front end's item asks, the updates that brings run inside this one
/// frame, and then the screen is told to leave the way its own menu tells it to — it puts the scene
/// back itself, and going down is what writes. Nothing is seen: drawing happens once a frame, after.
///
/// **The record goes back in the middle**: building that screen is also the game loading the record
/// out of the file, which is what it held before this session counted anything.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread, with no run in progress and the
/// game settled where it is.
unsafe fn commit_records(runtime: &mut Runtime, chain: *mut c_void, result: &mut i32) {
    let captures = unsafe { runtime.game.captures() };
    if captures.is_empty() {
        return log!("score: this game keeps no record of captures; nothing to write");
    }
    let mut frames = 0;
    // Only the frame limit stops this. Not the chain's answer: `CHAIN_EXIT_SUCCESS` is 0, which is
    // also what a chain walk returns when it simply carried on, so guarding on it ended both loops
    // after one update — the screen was left standing where the player then met it, and the write came
    // from whatever the screen itself did later.
    let running = |result: &mut i32, frames: &mut u32| {
        *result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        *frames += 1;
        *frames < COMMIT_FRAME_LIMIT
    };
    unsafe { runtime.game.show_ranking() };
    while !unsafe { runtime.game.showing_ranking() } && running(result, &mut frames) {}
    if !unsafe { runtime.game.showing_ranking() } {
        // The request undone as well: asking is a reservation the front end acts on later, so leaving
        // one behind is a ranking that comes up by itself — which is what a give-up landing on the
        // score screen was.
        unsafe { runtime.game.leave_ranking() };
        return log!("score: the ranking was not built after {frames} update(s); nothing written");
    }
    log!("score: the ranking is up — {}", unsafe {
        runtime.game.ranking_state()
    });
    unsafe { runtime.game.set_captures(&captures) };
    unsafe { runtime.game.leave_ranking() };
    log!("score: asked it to leave — {}", unsafe {
        runtime.game.ranking_state()
    });
    // Until the scene is no longer the ranking's, not until the screen's own state changes: the state
    // changes on the update it is asked, and the scene goes back several updates later. Stopping at the
    // first left those updates to be drawn — the ranking, visible for a second.
    while unsafe { runtime.game.ranking_scene() } && running(result, &mut frames) {}
    // A couple more so the front end is built before its cursor is put back, and then the cursor: the
    // one that comes back is on the item the game thinks was left, which is the ranking orb asked for.
    for _ in 0..2 {
        running(result, &mut frames);
    }
    unsafe { runtime.game.restore_menu_cursor() };
    log!(
        "score: taken through the ranking in {frames} update(s) — {}",
        unsafe { runtime.game.ranking_state() }
    );
}

/// Gives the run up, which is the retry menu's third choice: the game is put on its way to the
/// title menu, and reports whether it had a run to leave.
///
/// Nothing of orb's is dropped here. The snapshots describe a stage the game is about to tear
/// down, and what notices that is the run leaving `in_run` a frame or two later — the same path
/// that ends any other run, which is also what writes the line saying how many retries it took.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn give_up(game: &dyn Game) -> bool {
    if !unsafe { game.leave_run() } {
        log!("retry: there is no run to give up");
        return false;
    }
    // The key that answered orb's question is not one the game's own screens should act on.
    // Needed here where it is not needed after a retry: a retry puts the whole of `.data` back
    // from a snapshot, and this leaves the game to build the title menu — which reads the same
    // keyboard, and would take the `z` still held from this answer as an item chosen.
    unsafe { game.swallow_input() };
    log!("retry: the run is given up; the game is on its way to the title");
    true
}

/// Replaces `th06::Chain::RunCalcChain`. `__thiscall` with a single argument is
/// `fastcall` with nothing on the stack, which is an ABI Rust can spell.
///
/// # Safety
/// Must run on the game's main thread, and `chain` must be the chain object the game calls this on:
/// it is handed to the game's own update, which reads the frame's jobs out of it.
pub unsafe extern "fastcall" fn run_calc_chain(chain: *mut c_void) -> i32 {
    if IN_HOOK.swap(true, Ordering::Relaxed) {
        note_reentry();
        return unsafe { call_original(&RUN_CALC_CHAIN, chain) };
    }
    let result = unsafe { on_update(chain) };
    IN_HOOK.store(false, Ordering::Relaxed);
    result
}

/// Once, so a hook that turns out to nest every frame does not fill the log.
fn note_reentry() {
    static REPORTED: AtomicBool = AtomicBool::new(false);
    if !REPORTED.swap(true, Ordering::Relaxed) {
        log!("hook re-entered from inside itself; the nested frame runs the game only");
    }
}

/// Replaces `th06::Chain::RunDrawChain`, so the overlay draws after the game's
/// own drawing and inside the same scene.
///
/// # Safety
/// As [`run_calc_chain`], and inside the scene the game draws into: what orb draws here goes through
/// the device it is already drawing with.
pub unsafe extern "fastcall" fn run_draw_chain(chain: *mut c_void) -> i32 {
    if IN_HOOK.swap(true, Ordering::Relaxed) {
        note_reentry();
        return unsafe { call_original(&RUN_DRAW_CHAIN, chain) };
    }
    unsafe { before_draw() };
    let result = unsafe { call_original(&RUN_DRAW_CHAIN, chain) };
    unsafe { after_draw() };
    IN_HOOK.store(false, Ordering::Relaxed);
    result
}

/// Replaces the game's window creation, to overrule its display setting first.
/// Borderless mode needs a window; a game that has taken the display exclusively
/// has none to resize, and by the time anything of ours runs per frame the device
/// already exists.
extern "C" fn create_game_window(instance: *mut c_void) {
    if FORCE_WINDOWED.swap(false, Ordering::Relaxed) && !GAME.windowed() {
        unsafe { GAME.force_windowed() };
        log!("borderless: overrode the game's fullscreen setting");
    }
    let original: extern "C" fn(*mut c_void) =
        unsafe { std::mem::transmute(CREATE_GAME_WINDOW.load(Ordering::Relaxed)) };
    original(instance)
}

/// Replaces `th06::GameWindow::Render` with orb's own frame: update first, then
/// draw, then present on the display's cadence.
///
/// The game's own order is draw-then-update, which puts everything on screen one
/// update behind the input that produced it. Doing the update first is the whole of
/// that fix; the pacing around it is what keeps the result smooth.
extern "fastcall" fn render(_window: *mut c_void) -> i32 {
    let runtime = unsafe { RUNTIME.get() }.as_mut();
    let Some(runtime) = runtime else {
        return unsafe { call_render(_window) };
    };
    let game = runtime.game;
    let device = unsafe { game.d3d_device() };
    // Nothing to pace or draw with; let the game have its own loop back.
    if device.is_null() {
        return unsafe { call_render(_window) };
    }
    // The game does nothing at all while its window is behind, and orb carries on: that is what
    // makes coming back to it instant instead of a stale frame, and what keeps a replay or a
    // stress run going while attention is elsewhere. The keys are dealt with in the input hook,
    // not by stopping. `always_draw: false` is for somebody who wants the game's own behaviour
    // back, and below is where they get it.
    let window = unsafe { game.window() };
    if !runtime.config.always_draw && orb_api::window::foreground() != window {
        // Paced even with nothing drawn. The wait this frame loop runs on is inside this
        // function, and the game's own loop calls it straight back, so a return that waits for
        // nothing spins a core for as long as the window stays behind. Not being in front is
        // exactly the case `wait_for_slot` paces by the clock, which is what is wanted here:
        // the cadence without a blank to count against.
        unsafe { pacing() }.wait_for_slot(window);
        return RENDER_KEEP_RUNNING;
    }

    let chain = game.chain();
    let (update, draw) = (
        RUN_CALC_CHAIN_TARGET.load(Ordering::Relaxed),
        RUN_DRAW_CHAIN_TARGET.load(Ordering::Relaxed),
    );
    // Calling a null function pointer is undefined, and the compiler turns it into
    // an instruction that only crashes. Handing the frame back is the honest answer.
    if update == 0 || draw == 0 {
        return unsafe { call_render(_window) };
    }
    let update: extern "fastcall" fn(*mut c_void) -> i32 = unsafe { std::mem::transmute(update) };
    let draw: extern "fastcall" fn(*mut c_void) -> i32 = unsafe { std::mem::transmute(draw) };

    // Before the update as the game does it: it leaves the viewport covering the whole
    // output, and the update is what narrows it to the playfield for the drawing that
    // follows. Doing it after the update overwrites that and the bullets end up outside
    // the playfield.
    //
    // Before the wait, too, because it ends in a `Clear` — a driver call that blocks
    // for as long as the display still holds a buffer of ours. Waiting here costs
    // nothing, since waiting is what comes next anyway; on the far side of the wait it
    // would push the rest of the frame past a blank, and the frame after that would sit
    // out an extra one.
    let started = frame::now();
    unsafe { game.prepare_frame(device) };
    let cleared = frame::now();
    // The frame's turn. What follows runs in one go, so the input the update reads is
    // as recent as the frame it appears in.
    unsafe { pacing() }.wait_for_slot(window);
    let waited = frame::now();
    let updated = update(chain);
    let ran = frame::now();
    unsafe { game.play_sounds() };
    if updated == CHAIN_EXIT_SUCCESS {
        return RENDER_EXIT_SUCCESS;
    }
    if updated == CHAIN_EXIT_ERROR {
        return RENDER_EXIT_ERROR;
    }
    let sounded = frame::now();

    let drawn = unsafe {
        let vtable = &*(*device).vtable;
        (vtable.begin_scene)(device);
        draw(chain);
        (vtable.end_scene)(device);
        (vtable.set_texture)(device, 0, std::ptr::null_mut());
        let drawn = frame::now();
        game.present();
        drawn
    };
    unsafe { pacing() }.finished(frame::Marks {
        started,
        cleared,
        waited,
        updated: ran,
        sounded,
        drawn,
        presented: frame::now(),
    });
    RENDER_KEEP_RUNNING
}

unsafe fn call_render(window: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(RENDER.load(Ordering::Relaxed)) };
    original(window)
}

/// Replaces the game's device setup, to redirect the device's `Present` before
/// anything is presented through it. Runs again after every device reset.
extern "C" fn init_d3d_device() {
    let device = unsafe { GAME.d3d_device() };
    if !device.is_null() {
        unsafe { window::hook_device(device) };
    }
    let original: extern "C" fn() =
        unsafe { std::mem::transmute(INIT_D3D_DEVICE.load(Ordering::Relaxed)) };
    original()
}

/// What the game sees on the keyboard this frame — nothing, while its window is not
/// the one in front, and the buttons written down for this frame while a run is being
/// played back into the chapter it was left in.
///
/// The keyboard is read globally, not per window, so a game that keeps updating in
/// the background would otherwise act on whatever is being typed elsewhere. Dropping
/// the buttons here rather than skipping the update is what lets a background window
/// go on being drawn.
///
/// This is also the one place every button the game acts on passes through, which is what makes it
/// where a run's own buttons are written down: against the frame of the stage about to run, since
/// the read happens before the counter moves. See [`resume`].
pub extern "system" fn get_input() -> u16 {
    // Which frame of the stage this read is for. Before anything else, and before the foreground
    // check: a run being played back into place is being played with the window wherever it is,
    // and the frames it runs through are not frames anybody is at the keyboard for.
    let frame = unsafe { GAME.stage_frame() };
    if let Some(buttons) = frame.and_then(|frame| unsafe { resume::fed(frame) }) {
        return buttons;
    }

    // Asked of the system rather than read from the game's own `WM_ACTIVATEAPP`
    // flag, which only says what the game was last told; this is the same question
    // orb asks for its own keys, so the two cannot disagree.
    let window = unsafe { GAME.window() };
    let active = !window.is_null() && orb_api::window::foreground() == window;
    if INPUT_ACTIVE.swap(active, Ordering::Relaxed) != active {
        log!(
            "input: window {}",
            if active {
                "in front"
            } else {
                "behind, keys not read"
            }
        );
    }

    // Not read at all while behind, rather than read and thrown away. The game's
    // keyboard is a foreground DirectInput device, so the system unacquires it the
    // moment the window goes behind and every read then fails. The game checks only
    // for `DIERR_INPUTLOST` and treats anything else as a success, so a failed read
    // hands it an uninitialised stack buffer as the key state.
    if !active {
        INPUT_LOST.store(true, Ordering::Relaxed);
        // Nothing is being held back on the frames the game is being given nothing, and what it was
        // holding is forgotten with them: a button let go of behind the window would otherwise still
        // read as held on the frame it comes back on, and a press then is a press swallowed rather than
        // held back. A button still down on that frame reads as a fresh press instead, which puts the
        // question up — the same trade the game makes across its own scene changes, and the harmless
        // way round. See [`held_back`].
        DECIDE_WAS_DOWN.store(false, Ordering::Relaxed);
        return unsafe { noted(frame, 0) };
    }
    // Back in front: get the device back before anyone reads it. Left to itself the
    // game would only try on the one `DIERR_INPUTLOST` that reports the loss, and
    // whether it ever sees that report depends on exactly when the frames fell —
    // which is not something to leave the whole keyboard resting on.
    if INPUT_LOST.load(Ordering::Relaxed) {
        if !unsafe { GAME.acquire_input() } {
            return unsafe { noted(frame, 0) };
        }
        INPUT_LOST.store(false, Ordering::Relaxed);
        log!("input: keyboard re-acquired");
    }

    let original: extern "system" fn() -> u16 =
        unsafe { std::mem::transmute(GET_INPUT.load(Ordering::Relaxed)) };
    // Timed because it is the largest single thing in a frame — most of a refresh at
    // 120Hz — and none of it is orb's.
    let started = profile::now();
    let buttons = original();
    unsafe { profile::record(profile::Phase::Input, started) };
    unsafe { noted(frame, held_back(buttons)) }
}

/// The word the game reads, with the front end's own decide taken out of it while orb has a question
/// to ask on that press, and put back for the one read that hands the press over.
///
/// Done to the word the read hands back rather than to `g_CurFrameInput` afterwards, because that is
/// what `Supervisor::OnUpdate` assigns from and what every `WAS_PRESSED` in the frame is worked out
/// against — see [`Game::swallow_input`]. A press taken out here is one no screen in that frame ever
/// saw, so the screen orb asked over is still standing, on the item it was asked about, and answering
/// "neither" is nothing at all rather than a scene put back by hand.
///
/// The edge is orb's own to keep, for the same reason: the game's copy of the frame before has the
/// bits missing too, so it would call a press that has been held back for ten frames a fresh one on
/// the frame the holding stopped.
///
/// And with nothing new in it at all on the first read after a question came down, which is what
/// [`SETTLE_KEYS`] asks for.
fn held_back(buttons: u16) -> u16 {
    let decide = GAME.menu_decide();
    let cancel = GAME.menu_cancel();

    // The key a question was cancelled with, until it is let go: it is still down on the frame the
    // game carries on into, and going back is what the screen underneath does with it.
    //
    // Or until the screen it was cancelled on is gone, which is the same holding as the decide's. A
    // hold that outlived it would take the bomb and the pause button out of a run: answer a question
    // with the cancel key still down — a run is started with the other hand — and every read of the
    // stage would be short of them until it was let go, including the reads a resumed run writes down
    // as its own.
    let buttons = if !HOLD_CANCEL.load(Ordering::Relaxed) {
        buttons
    } else if buttons & cancel == 0 || !HOLD_DECIDE.load(Ordering::Relaxed) {
        HOLD_CANCEL.store(false, Ordering::Relaxed);
        buttons
    } else {
        buttons & !cancel
    };

    // And every other key that was pressed while the question was up, on the one read the game
    // carries on with. Its own idea of the frame before is the frame the question went up on — no
    // frame of one runs the chain, so nothing assigns `g_LastFrameInput` — so a key held on orb's
    // menu is a key the screen underneath reads as a fresh press. The directions are what that costs:
    // orb's menus and the game's read the same ones, and both of these screens move their cursor
    // before they read their decide, so the item handed the press would not be the item asked about.
    //
    // The decide is left in, being the one bit whose edge orb keeps itself: masked out here it would
    // read as let go, and the read after as pressed again — which is the question coming straight back
    // up on the frame it was cancelled.
    let buttons = if SETTLE_KEYS.swap(false, Ordering::Relaxed) {
        buttons & (LAST_WORD.load(Ordering::Relaxed) | decide)
    } else {
        buttons
    };

    let word = if FEED_DECIDE.swap(false, Ordering::Relaxed) {
        buttons | decide
    } else if !HOLD_DECIDE.load(Ordering::Relaxed) {
        // Written down on these reads too, so that what is already down as the holding starts is not
        // taken for a press made under it. It is the opposite: the press that reached the screen before
        // is the one that got the game *to* this screen, and the key is still down on its first frames.
        // Read as a press, that put the question up before anybody had asked for a run.
        DECIDE_WAS_DOWN.store(buttons & decide != 0, Ordering::Relaxed);
        buttons
    } else {
        let down = buttons & decide != 0;
        let was = DECIDE_WAS_DOWN.swap(down, Ordering::Relaxed);
        if down && !was {
            DECIDE_PRESSED.store(true, Ordering::Relaxed);
        }
        buttons & !decide
    };
    LAST_WORD.store(word, Ordering::Relaxed);
    word
}

/// Writes down what the update about to run will act on, and hands it back to the game.
///
/// The frames the window was behind are written down as well, as the nothing the game is given
/// there: what has to be reproduced is what the update saw, and a run left with the window behind
/// has frames of exactly that.
///
/// # Safety
/// Only ever called from the input hook, on the game's main thread.
unsafe fn noted(frame: Option<u32>, buttons: u16) -> u16 {
    if let Some(frame) = frame {
        unsafe { resume::noted(frame, buttons & GAME.run_input()) };
    }
    buttons
}

/// Runs after `th06::GameManager::AddedCallback`, where the game has just put a stage's numbers in
/// place and that stage has not been updated yet.
///
/// Nothing of `Runtime` is touched here: this is called from inside the game's own update, where
/// the frame hook is already holding it. What it needs to know — whether this run is being kept —
/// is [`resume`]'s own flag.
pub extern "C" fn stage_begun(manager: *mut c_void) -> i32 {
    let original: extern "C" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(STAGE_BEGUN.load(Ordering::Relaxed)) };
    let result = original(manager);
    // Its own answer first: anything but nothing is a stage it could not build, and the game is
    // on its way back to its menu with no stage for any of this to be about.
    if result == 0 {
        unsafe { resume::stage_begun(&GAME) };
    }
    result
}

/// Runs around the callback that takes what the front end offers out of the score file, which is the
/// one read of that file whose answer is the game's own whichever mode is on.
///
/// Bracketed rather than the mode's file being chosen once: everything else the file holds is a
/// record of runs played one way or the other, and what the front end offers is not a record —
/// see [`score`].
pub extern "C" fn unlocks_read(menu: *mut c_void) -> i32 {
    let original: extern "C" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(UNLOCKS_READ.load(Ordering::Relaxed)) };
    score::reading_unlocks(true);
    let result = original(menu);
    score::reading_unlocks(false);
    result
}

/// Runs before the callback that reads a ranking out of the score file, which is where the record of
/// captures that read is about to fill is emptied. See [`Game::forget_captures`] for why it has to be
/// emptied at all, and why here.
///
/// # Safety
/// Must run on the game's main thread, and `screen` must be the ranking screen's own object as the
/// callback this replaces is handed it: the state of that screen is read through it, and it is what
/// decides whether the record in memory is that read's to fill.
pub unsafe extern "C" fn ranking_read(screen: *mut c_void) -> i32 {
    let original: extern "C" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(RANKING_READ.load(Ordering::Relaxed)) };
    unsafe { GAME.forget_captures(screen) };
    original(screen)
}

/// Runs before `th06::Stage::RegisterChain`, where the stage's numbers are in place and the stage
/// itself is about to be built out of them — including out of the generator, which is why a resumed
/// run's seed goes in here and nowhere else.
pub extern "C" fn stage_building(stage: i32) -> i32 {
    let original: extern "C" fn(i32) -> i32 =
        unsafe { std::mem::transmute(STAGE_BUILDING.load(Ordering::Relaxed)) };
    unsafe { resume::stage_building(&GAME) };
    original(stage)
}

/// The joystick half of the input read, which is a tail call inside the keyboard half and so
/// cannot be told apart from outside it. Hooked to time it, and for nothing else: what made
/// it worth nine milliseconds a frame is answered from a sample of orb's own now — see
/// [`joystick`] — and this is how the perf line says so.
extern "C" fn get_controller_input(buttons: u32) -> u16 {
    let original: extern "C" fn(u32) -> u16 =
        unsafe { std::mem::transmute(GET_CONTROLLER_INPUT.load(Ordering::Relaxed)) };
    let started = profile::now();
    let all = original(buttons);
    unsafe { profile::record(profile::Phase::Joystick, started) };
    all
}

/// Replaces `th06::ReplayManager::SaveReplay`, dropping the write while leaving
/// the teardown the game does through the same function.
extern "C" fn save_replay(path: *const u8, name: *const u8) {
    if !path.is_null() {
        log!("replay save blocked");
        return;
    }
    let original: extern "C" fn(*const u8, *const u8) =
        unsafe { std::mem::transmute(SAVE_REPLAY.load(Ordering::Relaxed)) };
    original(path, name)
}

/// Replaces `th06::ReplayManager::StopRecording`, which finishes an input record off
/// with a blank entry and a frame number no run reaches. Right for a recording, and
/// wrong for a replay: the game runs it from `GameManager::DeletedCallback` at every
/// stage teardown, and during playback the record it writes into is the replay's own,
/// at wherever playback has reached.
///
/// So leaving a stage part way — which is what moving between a replay's stages
/// does — leaves that stage terminated at the frame it was left on. Play it again and
/// the player takes no input from there: it stands still and is hit. Measured at
/// 297414375ms in `orb.log`, jumping out of stage 1 around script frame 250 and
/// straight back, three lives gone by frame 1027.
extern "C" fn stop_recording() {
    if unsafe { GAME.replaying() } {
        detail!("replay: the record is being watched, not written; its terminator dropped");
        return;
    }
    let original: extern "C" fn() =
        unsafe { std::mem::transmute(STOP_RECORDING.load(Ordering::Relaxed)) };
    original()
}

unsafe fn call_original(trampoline: &AtomicUsize, chain: *mut c_void) -> i32 {
    let original: extern "fastcall" fn(*mut c_void) -> i32 =
        unsafe { std::mem::transmute(trampoline.load(Ordering::Relaxed)) };
    original(chain)
}

/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn on_update(chain: *mut c_void) -> i32 {
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return unsafe { call_original(&RUN_CALC_CHAIN, chain) };
    };
    runtime.keyboard.poll(unsafe { runtime.game.window() });

    // Asked once, and here rather than at startup because the device does not exist until the game
    // has made one — which it does after `DllMain` has been and gone.
    if runtime.config.sent_keys && !runtime.sent_keys {
        runtime.sent_keys = true;
        log!(
            "input: {}",
            if unsafe { runtime.game.take_sent_keys() } {
                "the game's own keyboard device let go of; keys sent to it are read the other way"
            } else {
                "the game had no keyboard device to let go of; it was already reading the other way"
            }
        );
    }

    if let Some(menu) = &mut runtime.retry {
        unsafe { hold_frame(runtime.game) };
        let pad = unsafe { pad(runtime.game) };
        if let Some((choice, by)) = menu.update(&runtime.keyboard, pad) {
            log!("retry: {} on the {by}", choice.label());
            let acted = match choice {
                Choice::Chapter => unsafe { runtime.chapters.retry_chapter(runtime.game) },
                Choice::Stage => unsafe { runtime.chapters.retry_stage(runtime.game) },
                Choice::Quit => {
                    let left = unsafe { give_up(runtime.game) };
                    runtime.given_up |= left;
                    left
                }
            };
            if acted {
                runtime.retry = None;
                // What the game carries on into is not a continuation of the frame we
                // froze on — a chapter put back, or the front end being built where the
                // run was given up — so nothing about it should be compared against that
                // frame.
                runtime.previous = None;
            }
        }
        return CHAIN_BREAK;
    }

    // No `hold_frame` with this one: it is over the game's own menu, and the frame that menu
    // draws into wants the whole output — which `prepare_frame` has already given it — rather
    // than the play field's viewport.
    if let Some(asking) = &mut runtime.asking {
        let pad = unsafe { pad(runtime.game) };
        if let Some((answer, by)) = asking.update(&runtime.keyboard, pad) {
            runtime.asking = None;
            // Whichever way it was answered: the keys that answered it are the title menu's own keys
            // too, and the frame it carries on into is one where nothing of orb's has moved its
            // cursor. See `held_back`.
            SETTLE_KEYS.store(true, Ordering::Relaxed);
            match answer {
                // The mode goes in before the item is acted on, and then the press is handed over so
                // that the menu chooses the item itself.
                Answer::Chosen(mode) => {
                    log!("mode: answered on the {by}");
                    choose(runtime, mode);
                    FEED_DECIDE.store(true, Ordering::Relaxed);
                }
                // The item is not chosen: the press that would have chosen it was held back, so the
                // menu never left the item it is on and there is no animation to put back. The mode is
                // left as it was, nothing having been answered.
                //
                // The key that cancelled is held back until it is let go: it is still down, and what
                // the title menu does with it is put its own cursor on `Quit`.
                Answer::Cancelled => {
                    log!("mode: not chosen on the {by}; the menu is where it was");
                    HOLD_CANCEL.store(true, Ordering::Relaxed);
                }
            }
        }
        return CHAIN_BREAK;
    }

    // Where to start a run that has a chapter of its own left unfinished. Over the game's own shot
    // type select, whose decide is being held back for exactly this — so no `hold_frame` here either.
    if let Some((menu, ..)) = &mut runtime.asking_resume {
        let pad = unsafe { pad(runtime.game) };
        if let Some((answer, by)) = menu.update(&runtime.keyboard, pad) {
            let (_, saved) = runtime.asking_resume.take().expect("the menu is up");
            SETTLE_KEYS.store(true, Ordering::Relaxed);
            unsafe { answered(runtime, answer, saved, by) };
        }
        return CHAIN_BREAK;
    }

    // A replay can be run faster than it was recorded, which is what makes a pass
    // over a full run to collect chapter boundaries take minutes. Every update is
    // still observed, so nothing is missed by going quickly.
    //
    // A run being cleared to reach an ending goes the same way. It is somebody at the
    // keyboard rather than a replay, but they are not dodging anything — nothing can hit
    // them — so the frames are there to see the run go by and not to play on.
    let fast_forward = runtime.previous.is_some_and(|previous| {
        previous.replay && previous.playing && !previous.demo
            || runtime.config.fast_clear && previous.in_game
    });
    let mut repeats = if fast_forward {
        runtime.config.speed
    } else {
        1
    };

    let mut result = CHAIN_BREAK;
    let mut state = runtime
        .previous
        .unwrap_or(unsafe { runtime.game.read_state() });
    // Stepping between chapter boundaries, and the hold that lets the frame one
    // falls on be looked at. Before the frame's own updates, and instead of them.
    if let Step::Held { reached, ran } = unsafe { step(runtime, chain, &state) } {
        state = reached;
        result = ran;
        // The game's own frame setup is part of the chain being held back, so the
        // viewport it would have set has to be set here instead.
        unsafe { hold_frame(runtime.game) };
        repeats = 0;
    }

    // Timed around orb's own work only: the game's update runs in between, and
    // counting it made orb look like it was costing milliseconds a frame.
    let mut started = profile::now();
    for _ in 0..repeats {
        // Before the update, because what reads it is the hit test inside the update. Not
        // during a replay or the attract demo, where the run is a record of one somebody
        // played and the player not dying would be a different run from the one recorded.
        if runtime.config.fast_clear && state.in_game {
            unsafe { runtime.game.make_invulnerable() };
        }
        unsafe { profile::record(profile::Phase::Update, started) };
        result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        started = profile::now();
        state = unsafe { runtime.game.read_state() };
        if runtime.chaptering() {
            unsafe {
                runtime.chapters.observe(
                    runtime.game,
                    &state,
                    &runtime.data,
                    runtime.config.self_check,
                )
            };
        }
        unsafe { reproduced(runtime, &state) };
        if result == CHAIN_EXIT_SUCCESS || result == CHAIN_EXIT_ERROR {
            break;
        }
    }

    // A run being picked up where an earlier session left it: the buttons it has already pressed,
    // played into the stage the game has just built. Inside the frame that built it, so nothing of
    // the run being played forward is ever drawn.
    if let Some(at) = unsafe { resume::landing() }
        && state.playing
    {
        let (reached, ran) = unsafe { play_in(runtime, chain, at) };
        state = reached;
        result = ran;
    }
    // Waiting for the stage one asked for. A stage takes a second to build; ten seconds of frames
    // is the game having gone somewhere else with the run, which is what a stage it cannot load
    // does.
    if unsafe { resume::starting() } {
        runtime.starting += 1;
        if runtime.starting > RESUME_START_FRAMES {
            unsafe { resume::abandon("the stage never came up") };
        }
    }

    // Runs the ending out inside the frame it starts on, which is what keeps it off the screen
    // entirely: drawing happens once per frame, and by the next one the game is on the frame
    // the skip stopped at. Jumping the scene instead would be simpler, but the ending is also
    // where the game sets the clear flag and enters the score, and those have to happen.
    //
    // Stops where the ending hands over to its staff roll rather than at the scene change,
    // which is what keeps the roll: the roll is the last of an ending's scripts and the scene
    // is the same across all of them, so stopping at the scene ran the roll out too.
    //
    // Never during a demo or a replay. Only a run someone played reaches an ending worth
    // skipping, and a demo that looked like one is what turned this into a hundred
    // thousand updates a frame.
    if runtime.config.skip_ending
        && state.in_ending
        && !state.demo
        && !state.replay
        // Not once the roll is what is running: the ending's flag stays set through it and the
        // scene stays 10, so this would start again on the next frame and run out the one part
        // of an ending that was worth keeping.
        && !runtime.rolling
    {
        let scene = state.scene;
        let script = state.ending_script;
        let track = runtime.game.music_identity();
        let mut frames = 0;
        let mut rolling = false;
        // Stops at the scene change as well as at the flag, so that whatever follows the
        // ending is reached and then left alone.
        while state.in_ending
            && state.scene == scene
            && !rolling
            && frames < ENDING_SKIP_LIMIT
            && result != CHAIN_EXIT_SUCCESS
            && result != CHAIN_EXIT_ERROR
        {
            result = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
            state = unsafe { runtime.game.read_state() };
            frames += 1;
            rolling = moved_on(script, state.ending_script);
        }
        runtime.rolling = rolling;
        if rolling {
            // The track with it, because the roll's script starts one of its own — the ending
            // plays `bgm/th06_16` and the roll `bgm/th06_17` — so both changing on the same
            // update is a second thing saying this is where one ends and the other begins.
            log!(
                "ending run out in {frames} update(s), where its staff roll begins, \
                 track {track:?} -> {:?}",
                runtime.game.music_identity(),
            );
        } else {
            log!(
                "ending skipped, {frames} frames run, scene {scene} -> {}",
                state.scene
            );
        }
    }
    // The roll is inside the ending as far as the game is concerned, so what says there is no
    // longer one to keep out of the way of is the ending's flag going out.
    if !state.in_ending {
        runtime.rolling = false;
    }

    // The screen that offers to save a replay of the run just finished, which a pointdevice run
    // has nothing to put in one — see `Game::skip_replay_prompt`. Costs one read on every other
    // frame there is.
    if runtime.mode == Mode::Pointdevice && unsafe { runtime.game.skip_replay_prompt() } {
        log!("result: no replay is offered for a run with chapters");
    }

    // Over a replay as well as over a run someone is playing, because a replay is
    // what plays a whole run for a tuning pass.
    if runtime.chaptering() && runtime.chapters.tracking(&state) {
        tune(runtime, &state);
        boundary_reached(runtime, &state);
        unsafe { keep_chapter(runtime, &state) };
    }
    // The run over, whichever way: what was written down is a stage the game is taking down, and
    // the file beside it is left holding the chapter unless the run *finished* — the result screen
    // is what a run reaches and what giving one up does not go through.
    if runtime.previous.is_some_and(|previous| previous.in_run) && !state.in_run {
        unsafe { resume::forget() };
        // Any run that ends, not only one given up through orb's own menu: legacy mode has no such
        // menu, so the game's own quit is the only way out of one and it writes nothing either. A demo
        // is nobody's run and has nothing to write.
        if runtime
            .previous
            .is_some_and(|previous| !previous.demo && !previous.replay)
        {
            runtime.given_up = true;
            log!("score: a run ended; what it counted waits for the trip through the ranking");
        }
    }
    // The first frame after a run was given up on which the game has arrived somewhere: asking for a
    // screen while a change was in flight is what broke the front end twice.
    // The first frame after a run was given up on which the game has arrived somewhere.
    if runtime.given_up && !state.in_run && state.scene == state.wanted {
        runtime.given_up = false;
        unsafe { commit_records(runtime, chain, &mut result) };
        state = unsafe { runtime.game.read_state() };
    }
    // The mode as well as the chapter, because the result screen is a screen every run ends at: a
    // normal run's game over would otherwise take away the chapter a pointdevice run was left in.
    // And this run's own file, which is the one the run just finished was written to.
    if runtime.kept.is_some() && runtime.keeping() && unsafe { runtime.game.run_finished() } {
        let run = unsafe { runtime.game.run_start() };
        resume::discard(&runtime.config.base_dir, runtime.game, &run);
        runtime.kept = None;
    }
    unsafe { stress(runtime, &state) };
    // Polling DirectSound every frame is diagnostic work, so it only happens when
    // something is being diagnosed.
    if runtime.config.stress_restore_frames > 0 || runtime.config.self_check {
        unsafe { watch_music(runtime, &state) };
    }

    // The miss is only certain once `deaths` moves, which is after the death bomb
    // window has closed; a successful death bomb never gets here.
    let died = state.in_game
        && runtime
            .previous
            .is_some_and(|previous| state.deaths > previous.deaths);
    if died && runtime.chaptering() && runtime.chapters.can_retry() {
        log!("died in chapter {}", runtime.chapters.number());
        runtime.retry = Some(RetryMenu::new());
    }

    // The question after the character select, where a run has been chosen and a chapter of that
    // same run — the same difficulty, character and shot — was left unfinished. Read off the disk
    // here rather than kept from startup: what it offers is the chapter the last session stopped at,
    // which moves with every chapter a run reaches.
    //
    // Before the frame that builds the run, which is what makes both answers cost nothing: the game
    // has settled what the run is and has not acted on it.
    if runtime.keeping() && runtime.asking_resume.is_none() && unsafe { runtime.game.run_chosen() }
    {
        let run = unsafe { runtime.game.run_start() };
        // A run the game keeps no slot for is a practice run: nothing was written down for it and
        // nothing is asked about it, and that ends here rather than in a line of the log.
        if let Some(slot) = runtime.game.run_slot(&run) {
            match runtime.started.take() {
                // Answered a screen earlier, on the press that started this run. The chapter goes in
                // here and not there because this is the one frame the game will take a stage for a
                // run it has not built yet — see [`Game::start_stage`].
                Some(started) if started.slot == slot => {
                    if let Some(saved) = started.saved {
                        runtime.starting = 0;
                        unsafe { resume::begin(runtime.game, saved) };
                    }
                }
                answered => {
                    if let Some(started) = answered {
                        log!(
                            "resume: {} was answered for, and {slot} is what the game registered",
                            started.slot
                        );
                    }
                    ask_where_to_start(runtime, &run, &slot, resume_ui::Cancels::Nothing);
                }
            }
        }
    }

    // And the same thing said a screen earlier, over the game's own shot type select, where it costs
    // nothing and freezes nothing: which run is under the cursor, and whether that one has a chapter
    // written down. Every frame, because the cursor moves between the two shots on the screen — the
    // file is only read where the answer would be about another run.
    let keeping = runtime.keeping();
    let pointing = keeping
        .then(|| unsafe { runtime.game.run_pointed_at() })
        .flatten();
    let pointed = pointing.as_ref().and_then(|run| runtime.game.run_slot(run));
    let base = &runtime.config.base_dir;
    runtime
        .mark
        .pointing(pointed.as_deref(), |slot| resume::peek(base, slot));

    // And the press that would choose an item is kept from the game on both of the screens orb has a
    // question about — the title menu and the shot type select — so that a question goes on the press
    // rather than after it. See `held_back`, and `Game::menu_pointed_at` for what that buys: a screen
    // whose press never arrived is a screen a cancel leaves exactly where it was, with no state to put
    // back and no animation to sit through.
    //
    // Whichever item the cursor is on, and not only the ones orb would ask about, although that is a
    // press held back on every run started and one file read on each. The frame the cursor arrives on
    // the item is why: this is written after the game's update, so it is the *next* frame's read the
    // holding starts on, and both screens move their cursor before they read their decide — 0x436a88
    // against 0x436d79 on the one, 0x437b5c against 0x437c1b on the other. So a direction and the shot
    // button on one frame choose the item the cursor has just reached, and holding only under the
    // cursor's own answer would leave that one frame able to start a run nobody was asked about.
    //
    // Not without an overlay to draw the question with: a press held back for a question nobody can
    // see is a screen that has stopped working.
    // Flattened, and not left as the `Option<Option<_>>` `then` hands back: the outer one says only
    // that somebody is there to be asked, which is a property of the launch and not of the screen —
    // read for the hold it would have been on every frame of every run, with the shot button taken
    // out of the word the game reads.
    let asked_about = runtime
        .asks_mode
        .then(|| unsafe { runtime.game.menu_pointed_at() })
        .flatten();
    HOLD_DECIDE.store(
        (pointing.is_some() || asked_about.is_some())
            && runtime.overlay.is_some()
            && unsafe { runtime.game.menu_takes_a_press() },
        Ordering::Relaxed,
    );
    if DECIDE_PRESSED.swap(false, Ordering::Relaxed) {
        match (pointing, pointed, asked_about) {
            (Some(run), Some(slot), _) => {
                ask_where_to_start(runtime, &run, &slot, resume_ui::Cancels::TheRun);
            }
            (_, _, Some(menu)) => {
                log!("menu: {menu:?} is under the cursor, asking which mode");
                runtime.asking = Some(ModeMenu::new(menu, runtime.mode));
            }
            // An item orb has nothing to ask about, which is four of the title menu's eight and every
            // frame the cursor has moved off one it does ask about. The press goes to the front end
            // unanswered, a frame late and otherwise as it was made.
            _ => {
                detail!(
                    "menu: nothing to ask about the item under the cursor; the press is handed over"
                );
                FEED_DECIDE.store(true, Ordering::Relaxed);
            }
        }
    }

    // Every frame while nothing is being played, not only where the scene changed: what a trip through
    // the game's own ranking has to fit into is the frames *inside* a transition, and a line per
    // change showed neither how many there are nor what the two fields do across them.
    if !state.in_run {
        detail!("state: cur={} wanted={}", state.scene, state.wanted);
    }
    let scene_changed = runtime
        .previous
        .is_none_or(|previous| previous.scene != state.scene);
    let due = state.in_game && runtime.frames % STATE_LOG_INTERVAL == 0;
    if scene_changed || due {
        summary!("f{} {state} clears={}", runtime.frames, unsafe {
            runtime.game.clears_back_buffer()
        },);
    }
    // `quiet` writes none of the above, and the compose time the pacing settles at is a property of
    // what the game was doing while it settled — a stage being played and a menu are not the
    // same load. Without this a pacing run's numbers cannot be attributed to anything, which
    // is how a sweep came to be written up against a scene nobody had recorded.
    //
    // Once per report rather than per second, so there is one of these against each set of
    // numbers and no more.
    if log::pacing_wanted() && (scene_changed || runtime.frames % profile::INTERVAL == 0) {
        pacing!("f{} {state}", runtime.frames);
    }

    runtime.previous = Some(state);
    runtime.frames += 1;
    // Here rather than in the draw hook: this writes on the window itself, which is not
    // something to do in the middle of a scene the game is drawing into.
    unsafe { write_status(runtime) };
    unsafe {
        profile::record(profile::Phase::Update, started);
        if profile::frame() {
            // Both lines through the held queue when the pacing is what is being watched:
            // the buckets and the interval say as much about it as the worsts do, `quiet`
            // has nothing else to write them, and this runs inside the frame's work — the
            // one place a write costs the frame it is describing.
            if log::pacing_wanted() {
                pacing!("{}", pacing().report());
                pacing!("{}", pacing().worst());
                pacing!("{}", pacing().shown());
            } else {
                summary!("{}", pacing().report());
            }
            summary!(
                "audio: behind {}..{} bytes",
                if runtime.margin_best == u32::MAX {
                    0
                } else {
                    runtime.margin_best
                },
                runtime.margin_worst,
            );
            runtime.margin_best = u32::MAX;
            runtime.margin_worst = 0;
        }
    }
    result
}

/// Whether an ending has gone from one of its scripts to the next, which is where the ending
/// itself ends and its staff roll begins.
///
/// Both sides have to be known for a change to say that. With no script to compare — a game
/// whose ending orb cannot find, or an ending already torn down — the answer is no, and the
/// skip runs the scene out the way it did before there was a script to read.
fn moved_on(from: Option<usize>, to: Option<usize>) -> bool {
    matches!((from, to), (Some(from), Some(to)) if from != to)
}

/// Puts the resume question up, where the run has a chapter of its own to ask about.
///
/// `cancels` says what a cancel does, which is what differs between the two frames this is asked on.
/// [`resume_ui::Cancels::TheRun`] is also what says a press is being held back for the question, so
/// where there turns out to be nothing to ask it is that press which has to be handed over: one held
/// back for a question that never appears is a screen that has stopped working.
fn ask_where_to_start(
    runtime: &mut Runtime,
    run: &RunStart,
    slot: &str,
    cancels: resume_ui::Cancels,
) {
    let asked = match resume::load(&runtime.config.base_dir, runtime.game, run) {
        // Not without the overlay, which is what would draw it: a frozen game with an invisible
        // question over it is a game that looks broken, and what is lost by not asking is the run
        // starting where the game was going to start it anyway.
        Some(_) if runtime.overlay.is_none() => {
            log!("resume: {slot} was left, but there is no overlay to ask over the game with");
            false
        }
        Some(saved) => {
            log!("resume: {slot} was left; asking where to start");
            let menu = resume_ui::ResumeMenu::new(saved.describe(), cancels);
            runtime.asking_resume = Some((menu, saved));
            true
        }
        None => {
            detail!("resume: nothing was left of {slot}");
            false
        }
    };
    if !asked && cancels == resume_ui::Cancels::TheRun {
        FEED_DECIDE.store(true, Ordering::Relaxed);
    }
}

/// What answering it does, which is not the same at the two frames it can be asked on. The game is
/// what says which of them this is: on the frame a run was chosen a chapter can go in at once, and on
/// the press there is no run to put one into yet.
///
/// # Safety
/// Must run on the game's main thread, between frames.
unsafe fn answered(runtime: &mut Runtime, answer: resume_ui::Answer, saved: resume::Saved, by: By) {
    let slot = runtime.game.run_slot(&saved.run);
    let picked_up = match answer {
        resume_ui::Answer::Continue => {
            log!("resume: from where it stopped, answered on the {by}");
            Some(saved)
        }
        // The file is left where it is, and the first chapter this run reaches writes over it.
        resume_ui::Answer::Beginning => {
            log!("resume: from the beginning, answered on the {by}; {saved} is left behind");
            None
        }
        // Nothing to hand over and nothing to put back: the press that would have started the run is
        // the one this was asked on, and holding it back was the whole of it. What carries on
        // underneath is the shot type select, on the shot the question was about.
        //
        // Except for the key that cancelled, which is held back until it is let go: it is still down,
        // and what the shot type select does with it is go back to the character select — which is not
        // what somebody answering "neither" about this screen asked for.
        resume_ui::Answer::Cancelled => {
            log!("resume: neither, answered on the {by}; no run is started");
            HOLD_CANCEL.store(true, Ordering::Relaxed);
            return;
        }
    };
    if unsafe { runtime.game.run_chosen() } {
        if let Some(saved) = picked_up {
            runtime.starting = 0;
            unsafe { resume::begin(runtime.game, saved) };
        }
        // The key that answered is not one the stage about to be built should act on — the same reason
        // as the mode question's, and here the frame the game carries on into is the one that
        // registers the run.
        unsafe { runtime.game.swallow_input() };
        return;
    }
    match slot {
        Some(slot) => {
            runtime.started = Some(Started {
                slot,
                saved: picked_up,
            });
        }
        // Not reachable through the question, `resume::load` having found the file under that very
        // name; said rather than unwrapped, an answer dropped being a run that starts from the
        // beginning.
        None => log!("resume: the run answered about has no slot; the answer is dropped"),
    }
    FEED_DECIDE.store(true, Ordering::Relaxed);
}

/// Where a step left the game, when one happened.
enum Step {
    /// Not stepping: the frame runs as it would have.
    Carry,
    /// Held on a boundary. The game's update does not run this frame, and the draw
    /// puts up the frame it stopped on again.
    Held { reached: State, ran: i32 },
}

/// Stepping between chapter boundaries while a replay plays back.
///
/// What a midstage boundary has to be judged on is the frame it falls on, and at
/// eight updates to a drawn frame nothing drawn is within eight updates of one. A
/// step runs the updates with nothing drawn and holds the game on the frame it
/// arrives at, so that frame is the one on screen.
///
/// Only over a replay: a replay is what plays a whole run for a tuning pass, and
/// holding a run someone is playing is what the retry menu is for.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn step(runtime: &mut Runtime, chain: *mut c_void, state: &State) -> Step {
    // The run rather than the gameplay scene: a stage's end leaves that scene while the
    // game builds the next stage, and stopping there is the point.
    let watching = runtime.config.chapters
        && runtime.config.chapter_stepping
        && runtime.config.during_replay
        && state.replay
        && state.in_run
        && !state.demo
        && !state.paused;
    if !watching {
        runtime.held = false;
        runtime.walled = false;
        return Step::Carry;
    }
    let next = runtime.keyboard.pressed(keys::NEXT.0);
    let previous = runtime.keyboard.pressed(keys::PREVIOUS.0);
    let hold = runtime.keyboard.pressed(keys::HOLD.0);
    // Held rather than pressed: they say what the stepping keys mean, like a shift.
    let across = runtime.keyboard.held(keys::ACROSS.0);
    let dropped = runtime.keyboard.held(keys::DROPPED.0);
    if next || previous {
        detail!(
            "step: {} pressed, across {}, dropped {}",
            if next { "next" } else { "back" },
            if across { "held" } else { "not held" },
            if dropped { "held" } else { "not held" },
        );
    }

    // The end of the stage: the game leaving the gameplay scene to build the next one,
    // which is well after the boss went down. Held there and kept held — carrying on is
    // what loses the stage, so neither the hold key nor a step forward moves — while a
    // step back still goes where it always goes. Nothing has been torn down yet: the
    // hold is what stops the update that would do it.
    let ended = !state.playing;
    if ended {
        runtime.held = true;
        if !runtime.walled {
            runtime.walled = true;
            log!(
                "step: stage {} has ended; the across key with next or back leaves it",
                state.stage + 1,
            );
        }
    } else {
        runtime.walled = false;
    }

    // Moving between stages, which nothing else does.
    if across && (next || previous) {
        if let Some(step) = unsafe { leave(runtime, state, next) } {
            return step;
        }
        return if runtime.held {
            Step::Held {
                reached: *state,
                ran: CHAIN_BREAK,
            }
        } else {
            Step::Carry
        };
    }

    // A toggle rather than a resume, and not only on boundaries: stopping wherever
    // something looks like a gap is what puts `tuning_add_key` on an exact frame.
    if hold && !ended {
        runtime.held = !runtime.held;
        log!(
            "step: {} at chapter {} (script {})",
            if runtime.held { "held" } else { "playing on" },
            runtime.chapters.number(),
            state.script_frames,
        );
        if !runtime.held {
            return Step::Carry;
        }
    }
    // A boundary judged out of the table, which begins no chapter and so is nowhere in
    // the chapter starts the ordinary stepping moves between. Aimed at by the script
    // clock, the only thing that names it.
    if dropped && (next || previous) && !ended {
        let script = if next {
            runtime.chapters.next_dropped(state)
        } else {
            runtime.chapters.previous_dropped(state)
        };
        let Some(script) = script else {
            log!(
                "step: no boundary judged out {} script {}",
                if next { "after" } else { "before" },
                state.script_frames,
            );
            return if runtime.held {
                Step::Held {
                    reached: *state,
                    ran: CHAIN_BREAK,
                }
            } else {
                Step::Carry
            };
        };
        // Back the same way as any other back: the stage's start comes back and the
        // replay comes back with it, and the run forward from there lands on the frame.
        if previous && !unsafe { runtime.chapters.rewind_stage(runtime.game) } {
            log!("step: cannot go back — no stage start kept");
            return Step::Carry;
        }
        return unsafe {
            run_to(
                runtime,
                chain,
                Aim {
                    script: Some(script),
                    ..Aim::none()
                },
            )
        };
    }
    if next && !ended {
        let from = runtime.chapters.number();
        let at = runtime.chapters.next_start(state);
        return unsafe {
            run_to(
                runtime,
                chain,
                Aim {
                    at,
                    from: Some(from),
                    ..Aim::none()
                },
            )
        };
    }
    // Back is a restore and then the same thing again: a restore rewinds the replay
    // along with everything else, so the stage's start and a run forward from it land
    // exactly on the boundary asked for.
    if previous {
        // Read before the restore, which is what takes the stage's clock back.
        let behind = runtime.chapters.previous_start(state);
        if !unsafe { runtime.chapters.rewind_stage(runtime.game) } {
            log!("step: cannot go back — no stage start kept");
        } else if let Some(frame) = behind {
            let aim = Aim {
                at: Some(frame),
                ..Aim::none()
            };
            return unsafe { run_to(runtime, chain, aim) };
        } else {
            // The stage's own start is what lies before its first boundary, and the
            // restore has already arrived there.
            let reached = unsafe { runtime.game.read_state() };
            runtime.held = true;
            log!(
                "step: held at the stage's start (script {})",
                reached.script_frames
            );
            return Step::Held {
                reached,
                ran: CHAIN_BREAK,
            };
        }
    }
    if runtime.held {
        Step::Held {
            reached: *state,
            ran: CHAIN_BREAK,
        }
    } else {
        Step::Carry
    }
}

/// Asks the game to start the replay at the stage either side of this one, and reports
/// the frame to run when it will. `None` when there is no such stage, which leaves
/// whatever was going on alone.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn leave(runtime: &mut Runtime, state: &State, forward: bool) -> Option<Step> {
    let stage = if forward {
        state.stage + 1
    } else {
        state.stage - 1
    };
    if !unsafe { runtime.game.jump_to_stage(stage) } {
        log!("step: the replay has no stage {}", stage + 1);
        return None;
    }
    // The game has to run for it to build that stage, so nothing is held.
    runtime.held = false;
    runtime.walled = false;
    Some(Step::Carry)
}

/// What ends a step: whichever of these comes first.
struct Aim {
    /// A chapter this stage has begun once already, by the stage frame it began at.
    /// Set going either way, so that a boundary judged out of the table — which no
    /// longer begins a chapter, and so no longer moves the number — is still a place
    /// both keys stop at.
    at: Option<u32>,
    /// The chapter the step began in, for a boundary nothing has recorded yet: a stage
    /// the first time through has none of them, and a boss's are never in a table.
    /// `None` going back, which has to run past every boundary between the stage's
    /// start and the one asked for.
    from: Option<u32>,
    /// A frame of the enemy timeline, for a boundary of the table that begins no chapter:
    /// one judged out is in no chapter start, and the clock it is written in is the only
    /// thing that names it.
    script: Option<i32>,
}

impl Aim {
    /// Nothing asked for, to be filled in with the one thing that is.
    fn none() -> Self {
        Self {
            at: None,
            from: None,
            script: None,
        }
    }

    fn reached(&self, state: &State, number: u32) -> bool {
        self.at.is_some_and(|frame| state.stage_frames >= frame)
            || self
                .script
                .is_some_and(|frame| state.script_frames >= frame)
            || self.from.is_some_and(|from| number != from)
    }
}

/// Runs updates until the aim is reached, then holds the game there. The ending
/// skip's mechanism aimed at a boundary rather than at the end of a scene: drawing
/// happens once per frame, so however many updates this runs, not one of them is
/// seen.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn run_to(runtime: &mut Runtime, chain: *mut c_void, aim: Aim) -> Step {
    let mut reached = unsafe { runtime.game.read_state() };
    let mut ran = CHAIN_BREAK;
    let mut frames = 0;
    while !aim.reached(&reached, runtime.chapters.number())
        && reached.playing
        && !reached.paused
        && frames < STEP_LIMIT_FRAMES
        && ran != CHAIN_EXIT_SUCCESS
        && ran != CHAIN_EXIT_ERROR
    {
        ran = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        reached = unsafe { runtime.game.read_state() };
        unsafe {
            runtime.chapters.observe(
                runtime.game,
                &reached,
                &runtime.data,
                runtime.config.self_check,
            )
        };
        unsafe { reproduced(runtime, &reached) };
        frames += 1;
    }
    runtime.held = true;
    log!(
        "step: held in stage {} chapter {} at frame {} (script {}), {frames} update(s) run",
        reached.stage + 1,
        runtime.chapters.number(),
        reached.stage_frames,
        reached.script_frames,
    );
    Step::Held { reached, ran }
}

/// Plays a saved run's buttons into the stage the game has just built, until the frame the chapter
/// it was left in began on, and reports where that left the game.
///
/// The ending skip's mechanism aimed at a chapter: drawing happens once per frame, so however many
/// updates this runs, not one of them is seen. Every one of them is observed, though, which is
/// what leaves the run with the chapter's own snapshot to be sent back to and the stage divided
/// the way it was.
///
/// What the run lands on is then held against what was written down for that frame. Both lines go
/// to the log whatever comes of it: what a resume rests on is that the numbers a stage reads at its
/// start are all written down, and this is the instrument for that — see [`game::Reproduction`].
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn play_in(runtime: &mut Runtime, chain: *mut c_void, at: u32) -> (State, i32) {
    let mut reached = unsafe { runtime.game.read_state() };
    let mut ran = CHAIN_BREAK;
    let mut frames = 0;
    let deaths = reached.deaths;
    // The first frame the player was hit, where one was. The buttons written down are a path that
    // survived — every death a run had was rewound away by the retry menu, and the frames of the
    // attempt that did not survive were written over by the one that did — so a death here is a
    // playback that has come out of step, and the frame it happened on is where to look.
    let mut hit = None;
    // A bound on a clock that has stopped rather than on how far there is to go: an update of a
    // stage is one of its frames, so a playback that has run more updates than the frame it aims at
    // is one whose updates are going somewhere else.
    let limit = at + STALLED_FRAMES;
    while reached.stage_frames < at
        && reached.playing
        && frames < limit
        && ran != CHAIN_EXIT_SUCCESS
        && ran != CHAIN_EXIT_ERROR
    {
        ran = unsafe { call_original(&RUN_CALC_CHAIN, chain) };
        reached = unsafe { runtime.game.read_state() };
        unsafe {
            runtime.chapters.observe(
                runtime.game,
                &reached,
                &runtime.data,
                runtime.config.self_check,
            )
        };
        if hit.is_none() && reached.deaths > deaths {
            hit = Some(reached.stage_frames);
        }
        frames += 1;
    }
    if let Some(frame) = hit {
        log!(
            "resume: the player was hit at frame {frame}, which a path that survived does not do; \
             the playback has come out of step there or before it"
        );
    }
    let arrived = unsafe { resume::landed() };
    // The captures back as they were before the buttons went in: playing a stage again starts every
    // card the run had passed, and the game counts an attempt — and a capture — at each of them.
    if arrived.is_some() {
        let bytes = unsafe { resume::landed_captures(runtime.game) };
        if bytes != 0 {
            log!("resume: {bytes} byte(s) of captures put back; the playback counts none of them");
        }
        // And one attempt at the card the chapter is on, because picking a run up where it was left is
        // an attempt at that chapter — the same as the retry menu's, and the game counts neither: it
        // counts where a card starts, and this landing is inside one.
        if let Some(attempts) = unsafe { runtime.game.count_card_attempt() } {
            log!("resume: attempt {attempts} at this spell card");
        }
    }
    let Some(landing) = arrived else {
        return (reached, ran);
    };
    // The rewinds the run had already cost are part of what it is, and the status line's count
    // carries on from them rather than starting again at none.
    runtime.chapters.carry_retries(landing.retries);
    // The file already describes the chapter the run is standing in, so nothing writes it again —
    // and a run that goes on to finish from here still has that file taken away.
    runtime.kept = Some((reached.stage, landing.at));
    // Nothing about the frame the question was frozen on is worth comparing against this one. The
    // count of deaths above all: the run's own is in place now, and against the count the front end
    // was left holding it reads as a death on the frame the resume landed.
    runtime.previous = None;
    let landed = unsafe { runtime.game.reproduction() };
    log!(
        "resume: {frames} update(s) run; stage {} frame {} (script {}) in chapter {} ({}), \
         written down as chapter {} ({}) at frame {}; lives={} bombs={} power={} score={}",
        reached.stage + 1,
        reached.stage_frames,
        reached.script_frames,
        runtime.chapters.number(),
        runtime
            .chapters
            .name()
            .map_or_else(|| "none yet".to_owned(), |name| name.to_string()),
        landing.chapter,
        landing.name,
        landing.at,
        reached.lives,
        reached.bombs,
        reached.power,
        landed.score,
    );
    let landed = landed.to_string();
    match resume::differs(&landing.reproduction, &landed) {
        None => log!("resume: the landing is the frame that was written down, field for field"),
        Some(field) => {
            log!("resume: the landing is not the frame written down: {field}");
            log!("resume:   written {}", landing.reproduction);
            log!("resume:   landed  {landed}");
        }
    }

    // The song put where that chapter had it, which is what a retry of a chapter of the stage does
    // to the music — so a resume, which is the same chapter arrived at another way, owes the same.
    // What is playing at the landing is otherwise the track's opening milliseconds, the stage having
    // been built a frame ago.
    //
    // A boss's own theme is left alone, the way a fight's retry leaves it: starting it over every
    // attempt is worse than the jump in it. Which chapters those are is the same question the retry
    // asks, asked the same way — of the song playing and not of the chapter's kind, a midboss being a
    // fight the stage's song plays through.
    if runtime.chapters.stage_song_playing(runtime.game, &reached) {
        let moved = landing
            .song
            .zip(runtime.game.music())
            .is_some_and(|(song, music)| {
                let moved = unsafe { music.play_from(song) };
                log!(
                    "resume: the song {} at {song} in the track's own file",
                    if moved {
                        "picked up"
                    } else {
                        "could not be put"
                    },
                );
                moved
            });
        // The chapter's snapshot was taken during the playback, with the song where the playback had
        // it — nothing, near enough. Left at that, the first death in this chapter would rewind the
        // music to the top and undo what was just put right, so the snapshot is taken again now that
        // the sound is where it belongs. It costs one copy of a chapter, on a frame that has just
        // run a stage's worth of updates.
        if moved {
            unsafe {
                runtime.chapters.retake(
                    runtime.game,
                    &reached,
                    &runtime.data,
                    runtime.config.self_check,
                )
            };
        }
    }
    (reached, ran)
}

/// Writes down where the run is at every chapter it reaches, which is what a later launch offers to
/// pick up.
///
/// At every one of them rather than where a session ends, because there is no moment a session ends
/// at: closing the window, the game being killed and a crash all leave nothing to write from. What
/// this costs is one file of a few tens of kilobytes per chapter, beside a snapshot of several
/// megabytes taken on the same frame.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn keep_chapter(runtime: &mut Runtime, state: &State) {
    // A run somebody is playing, and not one being played back into place: the chapters a playback
    // passes through are behind the one it is going to, and writing one of those down would move
    // the resume point backwards.
    if !runtime.keeping() || !state.in_game || unsafe { resume::landing() }.is_some() {
        return;
    }
    let Some(name) = runtime.chapters.name() else {
        return;
    };
    let at = runtime.chapters.started_at();
    let chapter = (state.stage, at);
    if runtime.kept == Some(chapter) {
        return;
    }
    // Only from the frame the chapter began on, because what goes in the file with it is the
    // reproduction line for that frame and a frame further on is not that frame. A run at more
    // than one update to a drawn frame — a clear — passes chapters without ever standing on one.
    if state.stage_frames != at {
        return detail!(
            "resume: chapter {} began at frame {at} and the game is at {}; not written",
            runtime.chapters.number(),
            state.stage_frames,
        );
    }
    // Set whether or not the write comes off: what it would do again next frame is fail again,
    // and the line saying why has been written once.
    runtime.kept = Some(chapter);
    unsafe {
        resume::write(
            &runtime.config.base_dir,
            runtime.game,
            at,
            runtime.chapters.number(),
            &name.to_string(),
            runtime.chapters.retries(),
        )
    };
}

/// Starts the flash when the game comes to rest on a boundary it was not on before.
///
/// The boundary rather than the chapter number, because numbering starts again at every
/// stage: moving between stages lands on chapter 1 of the next one, which is a boundary
/// reached and would otherwise pass unmarked. Set here rather than where a step ends, so
/// that a boundary the game runs past while it is being held under `space` is marked as
/// well as one a step stops on.
fn boundary_reached(runtime: &mut Runtime, state: &State) {
    let boundary = (state.stage, runtime.chapters.started_at());
    if runtime.flashed == Some(boundary) {
        return;
    }
    runtime.flashed = Some(boundary);
    let watching = if runtime.config.chapter_stepping {
        Watching::Judge
    } else if state.in_game {
        Watching::Player
    } else {
        Watching::Nobody
    };
    if let Some(flash) = flash_for(
        watching,
        runtime.config.boundary_flash,
        runtime.chapters.cause(),
    ) {
        runtime.flash = Some((flash, flash.frames));
    }
}

/// Who is waiting to see a boundary land, which is what decides the wash it gets.
#[derive(Clone, Copy, PartialEq)]
enum Watching {
    /// A judging pass, where the wash is what the pass is run with.
    Judge,
    /// A run somebody is playing, where a boundary says where dying will send them.
    Player,
    /// A collecting pass, or a replay being tracked for anything else: sixty-four updates to
    /// a drawn frame and nobody at the keyboard.
    Nobody,
}

/// Which wash a boundary gets, and whether it gets one at all.
fn flash_for(watching: Watching, wanted: bool, cause: Cause) -> Option<Flash> {
    // Not the stage's own start. A stage beginning is already unmistakable — the title, the
    // music, the field empty — and a wash over the first frame of one says nothing that was
    // not already obvious.
    if cause == Cause::StageStart {
        return None;
    }
    match watching {
        // Whichever way the setting is left: it is about a run somebody is playing, and a
        // pass with no wash is a pass with nothing to judge a boundary by.
        Watching::Judge => Some(FLASH_JUDGING),
        Watching::Player => wanted.then_some(FLASH_PLAYING),
        // A collecting pass crosses a boundary every few drawn frames, so a wash there is a
        // green field for twenty minutes and a mark to nobody.
        Watching::Nobody => None,
    }
}

/// A line per update of a replay's stage, so that two passes over one stage can be held
/// against each other frame for frame. What a desynchronised replay looks like is the
/// player being hit where the recording was not, and the first frame these numbers
/// differ on says which of the replay's clock, its inputs, the player and the game's
/// generator went first.
///
/// Only while stepping, since that is the pass with somebody watching: a collecting pass
/// over a whole run would write a line per update of it.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn reproduced(runtime: &Runtime, state: &State) {
    // Nothing while the game is not moving: a menu holds one frame for as long as it is
    // open, and a line per drawn frame of it would be the same line hundreds of times —
    // which is not only noise but pushes everything after it out of step with the pass
    // being compared against.
    if !runtime.config.chapter_stepping
        || !state.replay
        || state.demo
        || !state.in_run
        || state.paused
        || !log::wanted(log::VERBOSE)
    {
        return;
    }
    let game = runtime.game;
    let reproduction = unsafe { game.reproduction() };
    // The update that runs while the game is building a stage, where the replay's own
    // clock does not exist yet and the player and the play field read as zeroes. There is
    // nothing there to hold against another pass.
    if reproduction.replay_frame < 0 {
        return;
    }
    detail!(
        "sync: stage={} frame={} script={} {} lives={} bombs={} power={} deaths={} \
         enemies={} bullets={} lasers={} attack={} spell={}{}{}",
        state.stage + 1,
        state.stage_frames,
        state.script_frames,
        reproduction,
        state.lives,
        state.bombs,
        state.power,
        state.deaths,
        state.enemy_count,
        state.bullet_count,
        state.laser_count,
        // What a chapter of a fight is derived from, next to the two things that stop one
        // being started: whether a bomb moves either of these is the question of whether a
        // bomb has to stop one at all.
        state.boss_attack_frames.unwrap_or(-1),
        state.spellcard.map_or(-1, |spell| spell as i32),
        if state.bombing { " bombing" } else { "" },
        if state.unsettled { " unsettled" } else { "" },
    );
}

/// The device setup the game does inside its own update, for a frame where that
/// update is being held back.
///
/// # Safety
/// Only ever called from the frame hook, on the game's main thread.
unsafe fn hold_frame(game: &dyn Game) {
    let device = unsafe { game.d3d_device() };
    if !device.is_null() {
        unsafe { game.set_play_viewport(device) };
    }
}

/// Watches the streaming margin, which is what a listener hears as the music
/// breaking up once it runs out.
unsafe fn watch_music(runtime: &mut Runtime, state: &State) {
    if !state.playing {
        return;
    }
    let Some(music) = runtime.game.music() else {
        return;
    };
    let identity = runtime.game.music_identity();
    if runtime.stream != (music.stream, identity) {
        log!(
            "audio: stream {:#010x} track {:?} (was {:#010x} track {:?})",
            music.stream,
            identity,
            runtime.stream.0,
            runtime.stream.1,
        );
        runtime.stream = (music.stream, identity);
    }
    let margin = unsafe { music.margin() };
    let Some(margin) = margin else { return };
    runtime.margin_worst = runtime.margin_worst.max(margin.behind);
    runtime.margin_best = runtime.margin_best.min(margin.behind);
    if runtime.margin_trace > 0 {
        runtime.margin_trace -= 1;
        log!(
            "audio: behind={} of chunk {} buffer {}",
            margin.behind,
            margin.notify_size,
            music.buffer_size,
        );
    }
}

/// Restores the current chapter on a timer, so the snapshot and the music get
/// exercised as many times as a long session would without anyone playing one.
unsafe fn stress(runtime: &mut Runtime, state: &State) {
    let interval = runtime.config.stress_restore_frames;
    // Not while the game is held on a boundary: a restore on a timer would take it
    // off the frame someone is looking at.
    if interval == 0 || !state.playing || state.unsettled || state.paused || runtime.held {
        return;
    }
    // Each chapter gets a few goes and is then left alone, so the run walks on
    // through the midstage, the midboss and every one of a boss's attacks.
    let chapter = (state.stage, runtime.chapters.number());
    if runtime.stressing != chapter {
        runtime.stressing = chapter;
        runtime.stressed = 0;
    }
    if runtime.stressed >= STRESS_PER_CHAPTER || !runtime.frames.is_multiple_of(interval) {
        return;
    }
    runtime.stressed += 1;
    if unsafe { runtime.chapters.retry_chapter(runtime.game) } {
        runtime.previous = None;
        // Watch what the restore did to the streaming, frame by frame.
        runtime.margin_trace = 12;
    }
}

/// Hand corrections to the midstage table while playing. The table is written out
/// by itself at the end of every stage; these are for a boundary the detector puts
/// in the wrong place, or misses.
fn tune(runtime: &mut Runtime, state: &State) {
    let add = runtime.keyboard.pressed(keys::ADD.0);
    let write = runtime.keyboard.pressed(keys::WRITE.0);
    let keep = runtime.keyboard.pressed(keys::KEEP.0);
    let drop = runtime.keyboard.pressed(keys::DROP.0);

    // These go through `Chapters`, which is what knows whether the frame the game is
    // standing on has a boundary of the table's: a boss's boundaries are the game's own
    // and there is nothing about them to write down.
    //
    // The hold goes with them because judging is only allowed while the game is standing
    // on the boundary. Adding is not: a gap the detector missed is caught by pressing at
    // the moment the stage passes through it, which is with the game running.
    let held = runtime.held;
    if add {
        unsafe {
            runtime.chapters.add_boundary(
                runtime.game,
                state,
                &runtime.data,
                runtime.config.self_check,
            )
        };
    }
    if keep {
        runtime.chapters.judge(state, held, Judgement::Better);
    }
    if drop {
        runtime.chapters.judge(state, held, Judgement::Worse);
    }

    // Written out as soon as anything is decided, not only at the end of the stage:
    // a session that is closed in the middle of one — which is how looking at a few
    // boundaries and stopping goes — would otherwise lose everything it decided.
    // Two files of a few hundred bytes, and only on a keypress.
    if (add || keep || drop || write)
        && let Some(tuning) = runtime.chapters.tuning()
    {
        tuning.write(&GAME);
    }
}

/// What has to be asked of the game before it draws, rather than after: the row the mark over the
/// lives goes on, repainted with its count, so that the stars show through where the ink is dry and
/// nothing of orb's accumulates on a panel the game otherwise leaves alone.
///
/// # Safety
/// Only ever called from the draw hook, before the game's own drawing, on its main thread.
unsafe fn before_draw() {
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return;
    };
    // Decided here and kept, so the ask and the mark cannot disagree about a frame — and so that
    // the answer is the one the next frame's decision is carried past the end of a run by.
    let marking = marking(runtime);
    runtime.marked = marking != Marking::No;
    if runtime.marked {
        unsafe { runtime.game.repaint_lives_row() };
    }
    // How long a panel outlives its run, said where it ends: that number is the whole of whether
    // the mark stops a frame before the painting does, and half a second of fade to the title is
    // not something to count by eye.
    if marking == Marking::PanelLeft {
        runtime.marked_after += 1;
    } else if runtime.marked_after > 0 {
        log!(
            "lives: the mark stayed on the panel for {} frame(s) after the run ended",
            std::mem::replace(&mut runtime.marked_after, 0)
        );
    }
}

/// Whether the lives are being marked as disabled this frame.
///
/// Three things: the mode, a chapter there is a snapshot to go back to, and a run somebody is
/// playing — not a demo or a replay, which have no menu offered to them. On the frames before the
/// stage's own snapshot exists a death does cost a life, and a mark saying otherwise would be
/// wrong about exactly the frames it matters.
///
/// **Asked of the run and not of the frame**, which is `runtime.previous`, and which was wrong
/// about the mark twice over. It is the gameplay scene alone, so it does not hold the frame a stage
/// transition is built in — one frame of the log and a quarter of a second of screen, `f44096
/// scene=3 stage=2` and then `f44097 scene=2 stage=3 frames=1` 265ms later, every transition of
/// that run between 250 and 265ms. And it is dropped outright wherever an update has put the game
/// somewhere that has nothing to do with the frame it froze on — a chapter put back, a resume
/// landed — which are frames in the middle of a run and drawn like any other. On both the mark went
/// off, and the game paints that row from its own "this row changed" bits without asking orb, so
/// what stood there instead was the count.
///
/// **And then the frames the run's panel outlives the run**, for as long as the game is still
/// painting that row: leaving a run ends it on one frame and leaves its panel on the screen until
/// the front end has drawn its own, and the row the game paints last is the one left standing
/// there. `Game::draws_lives_row` is that question, and it is what ends the mark rather than the
/// run ending — so the mark cannot stop a frame before the painting does, and cannot go on to be
/// drawn over a screen that is no longer the panel.
///
/// The death itself is still answered on `in_game`, which is where it belongs: it is a comparison
/// against the frame before, and nothing can be hit on a frame the game spends building a stage or
/// walking back to its title.
fn marking(runtime: &Runtime) -> Marking {
    if runtime.chaptering() && runtime.chapters.can_retry() && runtime.chapters.somebody_playing() {
        return Marking::Run;
    }
    if runtime.marked && unsafe { runtime.game.draws_lives_row() } {
        return Marking::PanelLeft;
    }
    Marking::No
}

/// Which of the two counts the mark over the lives is drawn on, where it is drawn at all.
#[derive(Clone, Copy, PartialEq)]
enum Marking {
    /// A frame of the run itself.
    Run,
    /// A frame after the run has ended, with the game still painting that row: the panel a run
    /// leaves on the screen for as long as the front end takes to draw over it.
    PanelLeft,
    No,
}

/// # Safety
/// Only ever called from the draw hook, between the game's `BeginScene` and
/// `EndScene`, on the game's main thread.
unsafe fn after_draw() {
    let started = profile::now();
    unsafe { draw_overlay() };
    unsafe { profile::record(profile::Phase::Draw, started) };
}

/// # Safety
/// Only ever called from the draw hook, between the game's `BeginScene` and
/// `EndScene`, on the game's main thread.
unsafe fn draw_overlay() {
    let Some(runtime) = unsafe { RUNTIME.get() }.as_mut() else {
        return;
    };
    if runtime.overlay.is_none() && runtime.overlay_attempts > 0 {
        let device = unsafe { runtime.game.d3d_device() };
        if device.is_null() {
            return;
        }
        runtime.overlay_attempts -= 1;
        runtime.overlay = unsafe {
            Overlay::new(
                device,
                &runtime.config.game_dir.join("font.ttf"),
                FONT_HEIGHT,
                MARK_FONT_HEIGHT,
            )
        };
        // Said when it is there and when the last try has gone, and nothing in between: what
        // each try itself could not do is already a line of its own.
        match (runtime.overlay.is_some(), runtime.overlay_attempts) {
            (true, _) => log!("overlay: ready"),
            (false, 0) => log!("overlay: unavailable"),
            (false, _) => {}
        }
    }
    let Some(overlay) = &runtime.overlay else {
        return;
    };

    // The game's count of lives painted over, where dying costs the chapter and not one of them.
    // Before the menus, because the mark belongs to the panel and a menu goes up over that.
    //
    // What `before_draw` decided for this frame, not the question asked again: the ask it made of
    // the game has already been spent by the game's own drawing, and a frame where one of the two
    // happened without the other is either a count with nothing over it or ink over a row nobody
    // repainted.
    if runtime.marked {
        let row = runtime.game.lives_row();
        let panel = unsafe { runtime.game.panel_tile() };
        unsafe { runtime.lives.draw(overlay, row, panel) };
    }

    if let Some(menu) = &mut runtime.retry {
        let area = runtime.game.play_area();
        // The chapter by name, which is what the menu is offering to put the player back at —
        // and by number in a pass building the table, where every number on screen is one to
        // hold against the log.
        let chapter = match runtime
            .chapters
            .name()
            .filter(|_| !runtime.config.chapter_tuning)
        {
            Some(name) => name.to_string(),
            None => format!("CHAPTER {}", runtime.chapters.number()),
        };
        unsafe { menu.draw(overlay, area, &chapter, runtime.chapters.retries()) };
        return;
    }

    if let Some(asking) = &mut runtime.asking {
        unsafe { asking.draw(overlay) };
        return;
    }

    if let Some((asking, _)) = &mut runtime.asking_resume {
        unsafe { asking.draw(overlay) };
        return;
    }

    // Over the game's own screen and not instead of it: what it says is about the item the game's
    // cursor is on, and the screen carries on running underneath.
    unsafe { runtime.mark.draw(overlay) };

    // Over the play field and no further. The rest of the game's output — the panel beside
    // it, the border around it — is not repainted every frame, so a wash drawn there is
    // not drawn over again and stays on the screen for good.
    //
    // Counted down in drawn frames rather than in the game's, because a step holds the game
    // on the frame a boundary falls on: its clock stops there and the flash would stop with
    // it.
    if let Some((flash, left)) = runtime.flash.take() {
        let area = runtime.game.play_area();
        let fading = flash.frames - flash.hold;
        let alpha = if left > fading {
            flash.alpha
        } else {
            flash.alpha * left / fading
        };
        if let Some(frame) = unsafe { overlay.frame() } {
            frame.fill(
                area.left,
                area.top,
                area.width,
                area.height,
                alpha << 24 | FLASH_COLOR,
            );
        }
        if left > 1 {
            runtime.flash = Some((flash, left - 1));
        }
    }
}

/// The line of numbers written below the game, in the black beside it.
///
/// Not part of the overlay, which draws into the game's own 640x480 back buffer — all of
/// that is shown inside the letterbox, so anything put there is over the game, and the
/// game does not clear it between frames so it also stays there. The black around the
/// game belongs to the window, and to orb.
///
/// # Safety
/// Must run on the game's main thread, outside a scene.
unsafe fn write_status(runtime: &mut Runtime) {
    let Some(state) = runtime.previous else {
        return;
    };
    // The lag is what the pacing costs, and it is only meaningful next to the rate it
    // buys: a low number with an uneven rate is not an improvement.
    //
    // Held still between refreshes so the numbers can be read at all.
    if runtime.frames.is_multiple_of(HUD_NUMBER_INTERVAL) {
        runtime.shown = unsafe { pacing() }.status();
    }
    let (lag, interval, compose) = runtime.shown;
    // A line each, because the black is usually bars down the sides of a widescreen
    // monitor rather than a strip under the game, and a strip that narrow takes one
    // short line at a time.
    //
    // Two different questions, and which is being asked depends on what the session is for.
    //
    // A run wants the chapter by name — which part of the stage it is, rather than a count of
    // the chapters gone by — since that is what says where dying will send the player and
    // whether the flash that just went off was expected.
    //
    // A pass building the table wants why the chapter changed *here*: the signal that produced
    // the boundary, and for the table's own the frame it is written down as and what has been
    // decided about it. Which is a name to nobody, and is what says whether the boundary
    // belongs in the table at all.
    let tuning = runtime.config.chapter_tuning;
    let mut lines = Vec::new();
    // Neither in normal mode, where there are no chapters: an empty name and a retry count that
    // cannot move are two lines saying that the run is not the one they describe.
    if runtime.chaptering() {
        if tuning {
            lines.push(format!(
                "CH {:02}  RETRY {}",
                runtime.chapters.number(),
                runtime.chapters.retries(),
            ));
        } else {
            if let Some(name) = runtime.chapters.name() {
                lines.push(name.to_string());
            }
            lines.push(format!("RETRY {}", runtime.chapters.retries()));
        }
    }
    lines.push(format!("INPUT LAG {}.{}ms", lag / 1000, lag % 1000 / 100));
    // Beside the lag rather than folded into it: this is the part of the lag orb chose, it is
    // the part that moves while the game runs, and watching it settle is watching the pacing
    // find how near the blank this display will take a frame.
    lines.push(format!(
        "COMPOSE {}.{}ms",
        compose / 1000,
        compose % 1000 / 100
    ));
    lines.push(format!(
        "{}.{}fps",
        1_000_000 / interval.max(1),
        10_000_000 / interval.max(1) % 10,
    ));
    // `HAND` marks the boundaries a person put there, those being the numbers nothing would
    // find again.
    //
    // The boundary a key would change comes first, and the chapter's own after it: the two
    // are the same thing wherever a step has stopped, and where they are not, what a key
    // is about to do outranks what is being played.
    if tuning {
        let judged = runtime
            .chapters
            .judged(&state, runtime.held)
            .or_else(|| runtime.chapters.chapter_boundary());
        lines.push(match judged {
            Some(judged) => format!(
                "{} {} {}",
                if judged.by_hand { "HAND" } else { "AUTO" },
                judged.frame,
                judged.verdict.label(),
            ),
            None => runtime.chapters.cause().label().to_owned(),
        });
    }
    // The script frame it would be written down as, and how many the stage's table has.
    if let Some(tuning) = runtime.chapters.tuning() {
        lines.push(format!("SCRIPT {}", state.script_frames));
        lines.push(format!("TABLE {}", tuning.count(state.stage)));
    }
    if runtime.held {
        lines.push("HOLD".to_owned());
    }
    unsafe { window::write_beside(&lines) };
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{
        Cause, DECIDE_PRESSED, DECIDE_WAS_DOWN, FEED_DECIDE, FLASH_JUDGING, FLASH_PLAYING, GAME,
        Game, HOLD_CANCEL, HOLD_DECIDE, SETTLE_KEYS, Watching, flash_for, held_back, moved_on,
    };

    /// What the game reads while orb is holding a press of its own back: the buttons it does read are
    /// its own, the decide is gone from every frame of the holding, it is the press that is reported
    /// and not each frame the button is down, the key a question was cancelled with reaches nothing
    /// until it is let go, and no other key held on the question reaches the read the game carries on
    /// with.
    ///
    /// One test for the sequence rather than one per step. Each read's answer depends on the reads
    /// before it, and what they are kept in is statics of the process — so two tests of this would be
    /// two tests writing each other's state as the harness ran them side by side. Every one of them is
    /// put where this test wants it first, for the same reason.
    #[test]
    fn a_press_held_back_is_reported_once_and_reaches_the_game_only_when_handed_over() {
        let decide = GAME.menu_decide();
        let other = 0x20;
        HOLD_DECIDE.store(false, Ordering::Relaxed);
        DECIDE_PRESSED.store(false, Ordering::Relaxed);
        DECIDE_WAS_DOWN.store(false, Ordering::Relaxed);
        FEED_DECIDE.store(false, Ordering::Relaxed);
        HOLD_CANCEL.store(false, Ordering::Relaxed);
        SETTLE_KEYS.store(false, Ordering::Relaxed);

        // Nothing held back: the word is the game's own, whatever is in it.
        assert_eq!(held_back(decide | other), decide | other);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // Held back, with the button still down from the press that got the game to this screen. Not a
        // press: nobody has asked for anything here yet, and reading it as one puts the question up on
        // the screen's own first frames.
        HOLD_DECIDE.store(true, Ordering::Relaxed);
        assert_eq!(held_back(decide), 0);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // Still down, and still nothing.
        assert_eq!(held_back(decide | other), other);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // Let go and pressed: that is the press the question goes on.
        assert_eq!(held_back(0), 0);
        assert_eq!(held_back(decide), 0);
        assert!(DECIDE_PRESSED.swap(false, Ordering::Relaxed));

        // Let go and pressed again, which is a second question.
        assert_eq!(held_back(0), 0);
        assert_eq!(held_back(decide), 0);
        assert!(DECIDE_PRESSED.swap(false, Ordering::Relaxed));

        // Handed over on one read, and on that read only: the screen decides on the edge, and a second
        // read with the bits in would be a second decide.
        FEED_DECIDE.store(true, Ordering::Relaxed);
        assert_eq!(held_back(other), decide | other);
        assert_eq!(held_back(other), other);
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));

        // A key pushed while the question was up does not reach the read the game carries on with: the
        // game's own frame before is the frame the question went up on, so a direction still down there
        // is a direction it reads as pressed — and it moves its cursor before it reads its decide.
        let direction = 0x40;
        assert_eq!(held_back(decide | other), other);
        assert!(DECIDE_PRESSED.swap(false, Ordering::Relaxed));
        SETTLE_KEYS.store(true, Ordering::Relaxed);
        assert_eq!(held_back(decide | other | direction), other);
        // And the decide is not settled with the rest of them: read as let go there, it would be a
        // press again on the read after, which is the question coming back up on its own cancel.
        assert!(!DECIDE_PRESSED.load(Ordering::Relaxed));
        // One read only. What is still down after it is down against a frame the game was given, so it
        // is the game's own to read — by which time the item it asked about has already had the press.
        assert_eq!(held_back(decide | other | direction), other | direction);

        // And the key a question was cancelled with: kept from the screen underneath, whose own back it
        // is, for as long as it is down. One of the two buttons rather than both, that being the
        // ordinary case and the one a faked previous frame got wrong.
        let cancel = GAME.menu_cancel();
        let one = cancel.isolate_lowest_one();
        HOLD_CANCEL.store(true, Ordering::Relaxed);
        assert_eq!(held_back(one | other), other);
        assert_eq!(held_back(cancel | other), other);

        // Let go, and the next press of it is the screen's own again: cancelling a question and asking
        // to go back are two different things to ask for.
        assert_eq!(held_back(other), other);
        assert!(!HOLD_CANCEL.load(Ordering::Relaxed));
        assert_eq!(held_back(one | other), one | other);

        // And it ends with the screen it was cancelled on whether or not the key has been let go: held
        // past that, it would be taking the bomb out of whatever was started next.
        HOLD_CANCEL.store(true, Ordering::Relaxed);
        assert_eq!(held_back(one | other), other);
        HOLD_DECIDE.store(false, Ordering::Relaxed);
        assert_eq!(held_back(one | other), one | other);
        assert!(!HOLD_CANCEL.load(Ordering::Relaxed));
    }

    /// The address is the file the script was read from, so the same address is the same part
    /// of the ending still running and any other address is the next part of it.
    #[test]
    fn an_ending_moves_on_when_its_script_does() {
        assert!(!moved_on(Some(0x1234), Some(0x1234)));
        assert!(moved_on(Some(0x1234), Some(0x5678)));
    }

    /// A script on one side only says nothing about the other: an ending that has just been
    /// torn down has no script, and reading one is a walk of the game's job chain that can
    /// come back with nothing.
    #[test]
    fn a_script_that_is_not_there_is_not_an_ending_moving_on() {
        assert!(!moved_on(None, Some(0x1234)));
        assert!(!moved_on(Some(0x1234), None));
        assert!(!moved_on(None, None));
    }

    #[test]
    fn a_stage_start_is_never_washed() {
        for watching in [Watching::Judge, Watching::Player, Watching::Nobody] {
            assert!(flash_for(watching, true, Cause::StageStart).is_none());
        }
    }

    #[test]
    fn a_run_is_washed_unless_the_setting_says_not_to() {
        let flash = flash_for(Watching::Player, true, Cause::BossSpell);
        assert_eq!(flash.map(|flash| flash.alpha), Some(FLASH_PLAYING.alpha));
        assert!(flash_for(Watching::Player, false, Cause::BossSpell).is_none());
    }

    /// The wash is the instrument a judging pass is run with, so the setting — which is
    /// about a run being played — does not reach it.
    #[test]
    fn a_judging_pass_is_washed_whatever_the_setting_says() {
        for wanted in [false, true] {
            let flash = flash_for(Watching::Judge, wanted, Cause::Boundary(1886));
            assert_eq!(flash.map(|flash| flash.alpha), Some(FLASH_JUDGING.alpha));
        }
    }

    /// A collecting pass runs a replay with nobody at the keyboard, and crosses a boundary
    /// every few drawn frames.
    #[test]
    fn a_pass_nobody_is_watching_is_not_washed() {
        assert!(flash_for(Watching::Nobody, true, Cause::Boundary(1886)).is_none());
    }

    /// Brighter and longer for the pass nobody is playing, since there nothing is being
    /// dodged on the frame underneath it.
    // Constants either side of every assertion here, which is the point of it: what is being
    // pinned is the relationship between two of them, and a run cannot tell you whether that has
    // been edited apart.
    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn a_judging_pass_is_washed_harder_than_a_run() {
        assert!(FLASH_JUDGING.alpha > FLASH_PLAYING.alpha);
        assert!(FLASH_JUDGING.frames > FLASH_PLAYING.frames);
        // Both fade rather than cutting out, which is what the hold being short of the
        // whole is: `frames - hold` is the divisor the fade is worked out over.
        for flash in [FLASH_JUDGING, FLASH_PLAYING] {
            assert!(flash.hold < flash.frames);
        }
    }
}
