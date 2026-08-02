//! Chapter-based retry for 東方紅魔郷 1.02h, injected before the game starts.

mod audio;
mod chapter;
mod crash;
mod d3d8;
mod frame;
mod game;
mod hook;
mod input;
mod joystick;
mod log;
mod mem;
mod memtrack;
mod overlay;
mod pe;
mod profile;
mod retry_ui;
mod score;
mod snapshot;
mod sync;
mod text;
mod threads;
mod tuning;
mod window;

use std::ffi::c_void;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use orb_config::Config;
use windows_sys::Win32::Foundation::{BOOL, HANDLE, TRUE};
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use chapter::{Cause, Chapters, Judgement};
use game::th06::Th06;
use game::{Game, State};
use input::Keyboard;
use log::{detail, log, pacing, summary};
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
    /// Created on the first frame that has a Direct3D device; `None` for good
    /// once it has failed, so a broken overlay does not retry every frame.
    overlay: Option<Overlay>,
    overlay_ready: bool,
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
}

/// The one game orb knows how to run inside.
static GAME: Th06 = Th06;

static RUNTIME: MainThread<Option<Runtime>> = MainThread::new(None);
/// Set while orb's own per-frame work is running.
///
/// Win32 calls that move a window dispatch messages synchronously, and the game
/// draws from its window proc, so a hook can be entered again from inside itself.
/// Nested entries run the game and nothing of ours: `Runtime` is handed out as
/// `&mut`, and two of those at once is not a thing that can be reasoned about.
static IN_HOOK: AtomicBool = AtomicBool::new(false);

static RUN_CALC_CHAIN: AtomicUsize = AtomicUsize::new(0);
static RUN_DRAW_CHAIN: AtomicUsize = AtomicUsize::new(0);
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
    frame::configure();

    let mut config = match log::host_exe().map(|path| Config::load_beside(&path)) {
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
         block_replay_save={} skip_ending={} borderless={} during_replay={} \
         fast_clear={} speed={} stress_restore_frames={} chapters={} track_memory={} \
         frame_hooks={}",
        config.game_dir.display(),
        config.log_level,
        config.pacing_log,
        config.compose_us,
        config.self_check,
        config.chapter_tuning,
        config.block_replay_save,
        config.skip_ending,
        config.borderless,
        config.during_replay,
        config.fast_clear,
        config.speed,
        config.stress_restore_frames,
        config.chapters,
        config.track_memory,
        config.frame_hooks,
    );
    // Set after the line that says what it is, so the log always states the level it
    // is then written at.
    log::set_level(config.log_level);
    log::set_pacing(config.pacing_log);
    frame::pin_compose(config.compose_us);

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

    // Only where chapters are: `--no-chapters` leaves orb loaded with nothing of its own
    // happening, and a run nothing can rewind belongs in the game's own ranking. Loud rather
    // than fatal if the import is not there, since what it costs is scores in the game's file
    // and not a run that cannot be played.
    //
    // A clear run installs it whichever way those are set, because there the fork is not what it
    // is for: the file that must not be written is the game's own, and refusing the write means
    // being in the path of it.
    if config.fast_clear || (config.chapters && config.own_score_file) {
        if config.fast_clear {
            score::refuse_writes();
        }
        match unsafe { score::install(exe) } {
            Ok(()) if config.fast_clear => log!("score: no file is written this run"),
            Ok(()) => log!("score: score.dat is forked to orb_score.dat"),
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
            run_calc_chain as usize,
            &RUN_CALC_CHAIN,
        ));
        hooks.push((
            "draw",
            patches.draw,
            run_draw_chain as usize,
            &RUN_DRAW_CHAIN,
        ));
    }
    match patches.render {
        Some(patch) if config.frame_hooks && config.own_frame_loop => {
            hooks.push(("frame loop", patch, render as usize, &RENDER));
        }
        _ => {}
    }
    match patches.input {
        // Only worth hooking when orb keeps the game updating with the window in the
        // background; left alone otherwise, the game stops updating there by itself.
        Some(patch) if config.frame_hooks && config.own_frame_loop && config.always_draw => {
            hooks.push(("input", patch, get_input as usize, &GET_INPUT));
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
                get_controller_input as usize,
                &GET_CONTROLLER_INPUT,
            ));
        }
        _ => {}
    }
    match patches.save_replay {
        Some(patch) if config.block_replay_save => {
            hooks.push(("replay save", patch, save_replay as usize, &SAVE_REPLAY));
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
            stop_recording as usize,
            &STOP_RECORDING,
        ));
    }
    if config.borderless {
        match unsafe { window::install(exe, GAME.content_size()) } {
            Ok(()) => log!("borderless: window hooks installed"),
            Err(error) => log!("borderless: {error}; the window is left as the game makes it"),
        }
        if let Some(patch) = patches.create_window {
            FORCE_WINDOWED.store(true, Ordering::Relaxed);
            hooks.push((
                "window creation",
                patch,
                create_game_window as usize,
                &CREATE_GAME_WINDOW,
            ));
        }
        if let Some(patch) = patches.init_device {
            hooks.push((
                "device init",
                patch,
                init_d3d_device as usize,
                &INIT_D3D_DEVICE,
            ));
        }
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

    let tuning = config.chapter_tuning.then(|| config.base_dir.clone());
    let during_replay = config.during_replay;
    unsafe {
        *RUNTIME.get() = Some(Runtime {
            game: &GAME,
            config,
            data,
            frames: 0,
            previous: None,
            keyboard: Keyboard::new(),
            overlay: None,
            overlay_ready: false,
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
            held: false,
            walled: false,
            rolling: false,
        });
    }
}

/// Replaces `th06::Chain::RunCalcChain`. `__thiscall` with a single argument is
/// `fastcall` with nothing on the stack, which is an ABI Rust can spell.
extern "fastcall" fn run_calc_chain(chain: *mut c_void) -> i32 {
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
extern "fastcall" fn run_draw_chain(chain: *mut c_void) -> i32 {
    if IN_HOOK.swap(true, Ordering::Relaxed) {
        note_reentry();
        return unsafe { call_original(&RUN_DRAW_CHAIN, chain) };
    }
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
    // The game does nothing at all while its window is behind. Carrying on is what
    // makes coming back to it instant instead of a stale frame, and it is also what
    // keeps a replay or a stress run going while attention is elsewhere. The keys are
    // dealt with in the input hook, not by stopping.
    let window = unsafe { game.window() };
    if !runtime.config.always_draw && unsafe { GetForegroundWindow() } != window {
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
    frame::wait_for_slot(window);
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
    frame::finished(frame::Marks {
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
/// the one in front.
///
/// The keyboard is read globally, not per window, so a game that keeps updating in
/// the background would otherwise act on whatever is being typed elsewhere. Dropping
/// the buttons here rather than skipping the update is what lets a background window
/// go on being drawn.
extern "system" fn get_input() -> u16 {
    // Asked of the system rather than read from the game's own `WM_ACTIVATEAPP`
    // flag, which only says what the game was last told; this is the same question
    // orb asks for its own keys, so the two cannot disagree.
    let window = unsafe { GAME.window() };
    let active = !window.is_null() && unsafe { GetForegroundWindow() } == window;
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
        return 0;
    }
    // Back in front: get the device back before anyone reads it. Left to itself the
    // game would only try on the one `DIERR_INPUTLOST` that reports the loss, and
    // whether it ever sees that report depends on exactly when the frames fell —
    // which is not something to leave the whole keyboard resting on.
    if INPUT_LOST.load(Ordering::Relaxed) {
        if !unsafe { GAME.acquire_input() } {
            return 0;
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
    buttons
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

    if let Some(menu) = &mut runtime.retry {
        unsafe { hold_frame(runtime.game) };
        if let Some(choice) = menu.update(&runtime.keyboard) {
            let restored = match choice {
                Choice::Chapter => unsafe { runtime.chapters.retry_chapter(runtime.game) },
                Choice::Stage => unsafe { runtime.chapters.retry_stage(runtime.game) },
            };
            if restored {
                runtime.retry = None;
                // The restored state is not a continuation of the frame we froze
                // on, so nothing about it should be compared against that frame.
                runtime.previous = None;
            }
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
        if runtime.config.chapters {
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

    // Over a replay as well as over a run someone is playing, because a replay is
    // what plays a whole run for a tuning pass.
    if runtime.config.chapters && runtime.chapters.tracking(&state) {
        tune(runtime, &state);
        boundary_reached(runtime, &state);
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
    if died && runtime.config.chapters && runtime.chapters.can_retry() {
        log!("died in chapter {}", runtime.chapters.number());
        runtime.retry = Some(RetryMenu::new());
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
                pacing!("{}", frame::report());
                pacing!("{}", frame::worst());
                pacing!("{}", frame::shown());
            } else {
                summary!("{}", frame::report());
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
        state.boss_attack_frames.map_or(-1, |frames| frames),
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
    if runtime.stressed >= STRESS_PER_CHAPTER || runtime.frames % interval != 0 {
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
    if add || keep || drop || write {
        if let Some(tuning) = runtime.chapters.tuning() {
            tuning.write(&GAME);
        }
    }
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
    if !runtime.overlay_ready {
        let device = unsafe { runtime.game.d3d_device() };
        if device.is_null() {
            return;
        }
        runtime.overlay_ready = true;
        runtime.overlay = unsafe {
            Overlay::new(
                device,
                &runtime.config.game_dir.join("font.ttf"),
                FONT_HEIGHT,
            )
        };
        log!(
            "overlay: {}",
            if runtime.overlay.is_some() {
                "ready"
            } else {
                "unavailable"
            }
        );
    }
    let Some(overlay) = &runtime.overlay else {
        return;
    };

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
    if runtime.frames % HUD_NUMBER_INTERVAL == 0 {
        runtime.shown = frame::status();
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
    use super::{Cause, FLASH_JUDGING, FLASH_PLAYING, Watching, flash_for, moved_on};

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
